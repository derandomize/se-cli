//! Общие типы для исполнения команд.

use std::fmt;

use super::parser::ParseError;

/// Спецификация команды после разбора строки.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct CommandSpec {
    /// Имя команды (builtin или внешняя).
    pub(crate) name: String,
    /// Аргументы команды (без имени).
    pub(crate) args: Vec<String>,
}

/// Результат исполнения внешней команды.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct RunResult {
    /// Код возврата процесса.
    pub(crate) exit_code: i32,
    /// Содержимое stdout процесса.
    pub(crate) stdout: Vec<u8>,
    /// Содержимое stderr процесса.
    pub(crate) stderr: Vec<u8>,
}

/// Потоки вывода интерпретатора.
pub(crate) struct IoStreams<'a> {
    /// Поток stdout интерпретатора.
    pub(crate) stdout: &'a mut dyn std::io::Write,
    /// Поток stderr интерпретатора.
    pub(crate) stderr: &'a mut dyn std::io::Write,
}

/// Управляющий результат исполнения команды.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ShellControl {
    /// Продолжить работу REPL.
    Continue(i32),
    /// Завершить REPL.
    Exit(i32),
}

/// Ошибки интерпретатора.
#[derive(Debug)]
pub(crate) enum ShellError {
    /// Ошибка парсинга командной строки.
    Parse(ParseError),
    /// Ошибка ввода/вывода.
    Io(std::io::Error),
    /// Ошибка запуска внешнего процесса.
    Process(String),
}

impl fmt::Display for ShellError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ShellError::Parse(e) => write!(f, "Parse error: {e}"),
            ShellError::Io(e) => write!(f, "I/O error: {e}"),
            ShellError::Process(msg) => write!(f, "Process error: {msg}"),
        }
    }
}

impl std::error::Error for ShellError {}

/// Удобный alias для результатов функций шелла.
pub(crate) type ShellResult<T> = Result<T, ShellError>;
