//! Запуск внешних команд.

use std::collections::HashMap;
use std::process::{Command, Stdio};

use super::types::{RunResult, ShellError, ShellResult};

/// Исполнитель внешних процессов через `std::process::Command`.
pub(crate) struct StdProcessExecutor;

impl StdProcessExecutor {
    /// Создает новый исполнитель внешних команд.
    pub(crate) fn new() -> Self {
        Self
    }

    /// Запускает внешнюю команду и возвращает ее stdout/stderr и код возврата.
    pub(crate) fn run_external(
        &self,
        program: &str,
        args: &[String],
        env: &HashMap<String, String>,
    ) -> ShellResult<RunResult> {
        // Очищаем env и передаем ровно то окружение, которое хранит ShellState.
        // Так тесты и поведение шелла остаются детерминированными.
        let mut cmd = Command::new(program);
        cmd.args(args);
        cmd.env_clear();
        cmd.envs(env);
        cmd.stdin(Stdio::inherit());
        cmd.stdout(Stdio::piped());
        cmd.stderr(Stdio::piped());

        let output = cmd.output().map_err(|e| {
            if e.kind() == std::io::ErrorKind::NotFound {
                ShellError::Process(format!("command not found: {program}"))
            } else {
                ShellError::Process(format!("failed to spawn {program}: {e}"))
            }
        })?;

        let exit_code = output.status.code().unwrap_or(1);
        Ok(RunResult {
            exit_code,
            stdout: output.stdout,
            stderr: output.stderr,
        })
    }
}
