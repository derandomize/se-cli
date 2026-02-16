//! Исполнение команд и цикл REPL.

mod builtins;
mod executor;
mod parser;
mod types;

#[cfg(test)]
mod tests;

use std::collections::HashMap;
use std::io::BufRead;

use builtins::Builtin;
use executor::StdProcessExecutor;
use parser::parse_line;
use types::{CommandSpec, IoStreams, ShellControl, ShellError, ShellResult};

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

    let parsed = parse_line(trimmed).map_err(ShellError::Parse)?;
    state.apply_assignments(&parsed.assignments);

    let Some(command) = parsed.command else {
        return Ok(ShellControl::Continue(0));
    };

    run_command(executor, state, command, io)
}

/// Выполняет распарсенную команду.
///
/// Если имя совпадает с builtin — вызывает builtin-реализацию.
/// Иначе запускает внешний процесс и прокидывает его stdout/stderr наружу.
fn run_command(
    executor: &StdProcessExecutor,
    state: &mut ShellState,
    command: CommandSpec,
    io: &mut IoStreams<'_>,
) -> ShellResult<ShellControl> {
    // Выполняет уже распарсенную команду (builtin или внешний процесс).
    if let Some(builtin) = Builtin::from_name(&command.name) {
        return builtins::run_builtin(builtin, &command.args, io);
    }

    let result = executor.run_external(&command.name, &command.args, &state.env)?;
    io.stdout
        .write_all(&result.stdout)
        .map_err(ShellError::Io)?;
    io.stderr
        .write_all(&result.stderr)
        .map_err(ShellError::Io)?;
    Ok(ShellControl::Continue(result.exit_code))
}
