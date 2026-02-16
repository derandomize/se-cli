//! Исполнение команд и цикл REPL.

mod builtins;
mod executor;
mod parser;
mod types;

#[cfg(test)]
mod tests;

use std::collections::HashMap;
use std::io::BufRead;
use std::io::Read;
use std::io::Write;
use std::process::Stdio;
use std::sync::Arc;

use builtins::Builtin;
use executor::StdProcessExecutor;
use parser::parse_line;
use types::{CommandSpec, IoStreams, Pipeline, ShellControl, ShellError, ShellResult};

/// Состояние интерпретатора.
///
/// Содержит набор переменных окружения, которые будут передаваться внешним процессам.
struct ShellState {
    env: HashMap<String, String>,
}

impl ShellState {
    /// Инициализирует состояние окружением текущего процесса.
    fn new_from_process_env() -> Self {
        let mut env = HashMap::new();
        for (k, v) in std::env::vars() {
            env.insert(k, v);
        }
        Self { env }
    }

    /// Применяет список присваиваний `NAME=value` к окружению интерпретатора.
    fn apply_assignments(&mut self, assignments: &[(String, String)]) {
        for (k, v) in assignments {
            self.env.insert(k.clone(), v.clone());
        }
    }
}

/// Запускает REPL поверх заданных потоков ввода/вывода.
pub(crate) fn run_repl<R: std::io::Read, W1: std::io::Write, W2: std::io::Write>(
    input: R,
    mut output: W1,
    mut error: W2,
) -> i32 {
    let mut state = ShellState::new_from_process_env();
    let executor = StdProcessExecutor::new();
    let mut io = IoStreams {
        stdout: &mut output,
        stderr: &mut error,
    };

    let reader = std::io::BufReader::new(input);
    for line in reader.lines() {
        let line = match line {
            Ok(l) => l,
            Err(e) => {
                let _ = writeln!(io.stderr, "I/O error: {e}");
                return 1;
            }
        };

        match run_single_line(&executor, &mut state, &line, &mut io) {
            Ok(ShellControl::Continue(_code)) => {
                // На этом этапе не ведём глобальный "$?".
            }
            Ok(ShellControl::Exit(code)) => return code,
            Err(e) => {
                let _ = writeln!(io.stderr, "{e}");
            }
        }
    }

    0
}

/// Обрабатывает одну строку ввода: trim → parse → apply env → execute.
///
/// Возвращает управляющее действие (продолжить или выйти) либо ошибку,
/// которую REPL напечатает в stderr.
fn run_single_line(
    executor: &StdProcessExecutor,
    state: &mut ShellState,
    line: &str,
    io: &mut IoStreams<'_>,
) -> ShellResult<ShellControl> {
    // Выполняет одну введенную строку: trim -> parse -> apply env -> builtin/external.
    let trimmed = line.trim();
    if trimmed.is_empty() {
        return Ok(ShellControl::Continue(0));
    }

    let parsed = parse_line(trimmed, &state.env).map_err(ShellError::Parse)?;
    state.apply_assignments(&parsed.assignments);

    let Some(pipeline) = parsed.pipeline else {
        return Ok(ShellControl::Continue(0));
    };

    run_pipeline(executor, state, pipeline, io)
}

/// Выполняет распарсенный pipeline.
///
/// `stdout` конвейера — это stdout последней команды. `stderr` каждой команды
/// выводится напрямую в `io.stderr` (не является частью пайпа).
fn run_pipeline(
    executor: &StdProcessExecutor,
    state: &mut ShellState,
    pipeline: Pipeline,
    io: &mut IoStreams<'_>,
) -> ShellResult<ShellControl> {
    if pipeline.commands.len() == 1 {
        return run_single_command(
            executor,
            state,
            pipeline.commands.into_iter().next().unwrap(),
            io,
        );
    }

    // `exit` внутри пайпа считаем ошибкой: не завершаем REPL внезапно.
    if pipeline.commands.iter().any(|c| c.name == "exit") {
        writeln!(io.stderr, "exit: cannot be used in pipeline").map_err(ShellError::Io)?;
        return Ok(ShellControl::Continue(2));
    }

    run_pipeline_with_os_pipes(state, pipeline, io)
}

struct StageResult {
    exit_code: i32,
    stderr: Vec<u8>,
}

/// Выполняет пайплайн через реальные OS-pipe'ы.
///
/// Стадии запускаются параллельно, чтобы избежать блокировок при заполнении буферов.
fn run_pipeline_with_os_pipes(
    state: &ShellState,
    pipeline: Pipeline,
    io: &mut IoStreams<'_>,
) -> ShellResult<ShellControl> {
    let n = pipeline.commands.len();
    debug_assert!(n >= 2);

    let env = Arc::new(state.env.clone());

    // Между стадиями: N-1 pipe'ов stdout->stdin.
    let mut readers: Vec<Option<os_pipe::PipeReader>> = Vec::with_capacity(n - 1);
    let mut writers: Vec<Option<os_pipe::PipeWriter>> = Vec::with_capacity(n - 1);
    for _ in 0..(n - 1) {
        let (r, w) = os_pipe::pipe().map_err(ShellError::Io)?;
        readers.push(Some(r));
        writers.push(Some(w));
    }

    // stdout последней стадии собираем через pipe в родителе, чтобы писать в `io.stdout`.
    let (mut final_out_reader, final_out_writer) = os_pipe::pipe().map_err(ShellError::Io)?;
    let mut final_out_writer = Some(final_out_writer);

    let mut handles = Vec::with_capacity(n);

    for (idx, command) in pipeline.commands.into_iter().enumerate() {
        let stdin_pipe = if idx == 0 {
            None
        } else {
            readers[idx - 1].take()
        };
        let stdout_pipe = if idx + 1 == n {
            final_out_writer
                .take()
                .expect("final_out_writer taken exactly once")
        } else {
            writers[idx]
                .take()
                .expect("writer for stage taken exactly once")
        };

        let env = Arc::clone(&env);
        handles.push(std::thread::spawn(move || -> ShellResult<StageResult> {
            if let Some(builtin) = Builtin::from_name(&command.name) {
                // Builtin запускаем в потоке. stdin читаем из pipe целиком.
                let input = if let Some(mut r) = stdin_pipe {
                    let mut buf = Vec::new();
                    r.read_to_end(&mut buf).map_err(ShellError::Io)?;
                    Some(buf)
                } else {
                    None
                };

                let mut out = Vec::new();
                let mut err = Vec::new();
                {
                    let mut local_io = IoStreams {
                        stdout: &mut out,
                        stderr: &mut err,
                    };
                    let control = builtins::run_builtin_with_input(
                        builtin,
                        &command.args,
                        input.as_deref(),
                        &mut local_io,
                    )?;
                    let exit_code = match control {
                        ShellControl::Continue(code) => code,
                        ShellControl::Exit(code) => code,
                    };

                    // stdout builtin'а — в stdout pipe.
                    let mut w = stdout_pipe;
                    w.write_all(&out).map_err(ShellError::Io)?;
                    drop(w);

                    return Ok(StageResult {
                        exit_code,
                        stderr: err,
                    });
                }
            }

            // External stage.
            let mut cmd = std::process::Command::new(&command.name);
            cmd.args(&command.args);
            cmd.env_clear();
            cmd.envs(env.iter());

            if let Some(r) = stdin_pipe {
                cmd.stdin(Stdio::from(r));
            } else {
                // В первом элементе пайплайна stdin пока не поддерживаем (нет редиректов),
                // чтобы REPL-ввод не смешивался с stdin команды.
                cmd.stdin(Stdio::null());
            }
            cmd.stdout(Stdio::from(stdout_pipe));
            cmd.stderr(Stdio::piped());

            let mut child = cmd.spawn().map_err(|e| {
                if e.kind() == std::io::ErrorKind::NotFound {
                    ShellError::Process(format!("command not found: {}", command.name))
                } else {
                    ShellError::Process(format!("failed to spawn {}: {e}", command.name))
                }
            })?;

            let mut child_stderr = child
                .stderr
                .take()
                .ok_or_else(|| ShellError::Process("failed to capture stderr".to_string()))?;

            let stderr_handle = std::thread::spawn(move || -> std::io::Result<Vec<u8>> {
                let mut buf = Vec::new();
                child_stderr.read_to_end(&mut buf)?;
                Ok(buf)
            });

            let status = child.wait().map_err(ShellError::Io)?;
            let exit_code = status.code().unwrap_or(1);

            let stderr = match stderr_handle.join() {
                Ok(Ok(buf)) => buf,
                Ok(Err(e)) => return Err(ShellError::Io(e)),
                Err(_) => {
                    return Err(ShellError::Process(
                        "stderr reader thread panicked".to_string(),
                    ));
                }
            };

            Ok(StageResult { exit_code, stderr })
        }));
    }

    // Собираем stdout последней стадии.
    let mut final_stdout = Vec::new();
    final_out_reader
        .read_to_end(&mut final_stdout)
        .map_err(ShellError::Io)?;

    let mut results = Vec::with_capacity(n);
    for h in handles {
        let res = h
            .join()
            .map_err(|_| ShellError::Process("pipeline stage panicked".to_string()))?;
        results.push(res?);
    }

    // stderr стадий печатаем в порядке команд (детерминированно для тестов).
    for r in &results {
        if !r.stderr.is_empty() {
            io.stderr.write_all(&r.stderr).map_err(ShellError::Io)?;
        }
    }
    io.stdout.write_all(&final_stdout).map_err(ShellError::Io)?;

    let last_exit = results.last().map(|r| r.exit_code).unwrap_or(0);
    Ok(ShellControl::Continue(last_exit))
}

fn run_single_command(
    executor: &StdProcessExecutor,
    state: &mut ShellState,
    command: CommandSpec,
    io: &mut IoStreams<'_>,
) -> ShellResult<ShellControl> {
    if let Some(builtin) = Builtin::from_name(&command.name) {
        return builtins::run_builtin(builtin, &command.args, io);
    }

    let result = executor.run_external(&command.name, &command.args, &state.env, None)?;
    io.stdout
        .write_all(&result.stdout)
        .map_err(ShellError::Io)?;
    io.stderr
        .write_all(&result.stderr)
        .map_err(ShellError::Io)?;
    Ok(ShellControl::Continue(result.exit_code))
}
