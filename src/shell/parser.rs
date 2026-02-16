//! Разбор командной строки (без подстановок и пайпов).
//!
//! Поддерживает:
//! - разделение на аргументы по пробелам
//! - одинарные и двойные кавычки (кавычки убираются)
//! - присваивания окружения `NAME=value` (в начале строки, в любом количестве)

use std::fmt;

use super::types::CommandSpec;

/// Результат парсинга одной строки.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ParsedLine {
    pub(crate) assignments: Vec<(String, String)>,
    pub(crate) command: Option<CommandSpec>,
}

/// Ошибка парсинга.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum ParseError {
    /// В строке есть незакрытая кавычка.
    UnclosedQuote(char),
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ParseError::UnclosedQuote(q) => write!(f, "unclosed quote: {q}"),
        }
    }
}

impl std::error::Error for ParseError {}

/// Парсит одну строку пользовательского ввода.
pub(crate) fn parse_line(line: &str) -> Result<ParsedLine, ParseError> {
    let tokens = tokenize(line)?;
    if tokens.is_empty() {
        return Ok(ParsedLine {
            assignments: Vec::new(),
            command: None,
        });
    }

    let mut assignments = Vec::new();
    let mut idx = 0;
    while idx < tokens.len() {
        if let Some((k, v)) = parse_assignment(&tokens[idx]) {
            assignments.push((k, v));
            idx += 1;
            continue;
        }
        break;
    }

    if idx >= tokens.len() {
        return Ok(ParsedLine {
            assignments,
            command: None,
        });
    }

    let name = tokens[idx].clone();
    let args = tokens[idx + 1..].to_vec();
    Ok(ParsedLine {
        assignments,
        command: Some(CommandSpec { name, args }),
    })
}

/// Пытается распарсить токен как присваивание окружения `NAME=value`.
///
/// Возвращает `None`, если токен не является присваиванием или имя переменной невалидно.
fn parse_assignment(token: &str) -> Option<(String, String)> {
    let (name, value) = token.split_once('=')?;
    if name.is_empty() {
        return None;
    }

    let mut chars = name.chars();
    let first = chars.next()?;
    if !(first == '_' || first.is_ascii_alphabetic()) {
        return None;
    }
    if !chars.all(|c| c == '_' || c.is_ascii_alphanumeric()) {
        return None;
    }

    Some((name.to_string(), value.to_string()))
}

/// Превращает входную строку в список аргументов.
///
/// Особенности:
/// - разделитель: пробелы/табы (последовательности разделителей схлопываются)
/// - одинарные/двойные кавычки группируют пробелы внутри аргумента
/// - кавычки удаляются (quote removal)
/// - пустые кавычки (`""` или `''`) создают пустой аргумент
fn tokenize(input: &str) -> Result<Vec<String>, ParseError> {
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    enum Mode {
        Normal,
        InQuote(char),
    }

    let mut tokens = Vec::new();
    let mut current = String::new();
    let mut mode = Mode::Normal;
    let mut token_started = false;

    let mut chars = input.chars().peekable();
    while let Some(ch) = chars.next() {
        match mode {
            Mode::Normal => match ch {
                ' ' | '\t' => {
                    if token_started {
                        tokens.push(std::mem::take(&mut current));
                        token_started = false;
                    }
                    while matches!(chars.peek(), Some(' ' | '\t')) {
                        let _ = chars.next();
                    }
                }
                '\'' | '"' => {
                    mode = Mode::InQuote(ch);
                    token_started = true;
                }
                _ => {
                    current.push(ch);
                    token_started = true;
                }
            },
            Mode::InQuote(q) => {
                if ch == q {
                    mode = Mode::Normal;
                } else {
                    current.push(ch);
                    token_started = true;
                }
            }
        }
    }

    match mode {
        Mode::Normal => {
            if token_started {
                tokens.push(current);
            }
            Ok(tokens)
        }
        Mode::InQuote(q) => Err(ParseError::UnclosedQuote(q)),
    }
}
