//! Разбор командной строки.
//!
//! Поддерживает:
//! - разделение на аргументы по пробелам
//! - одинарные и двойные кавычки (кавычки убираются)
//! - присваивания окружения `NAME=value` (в начале строки, в любом количестве)
//! - подстановки `$NAME` (в обычном режиме и в двойных кавычках)
//! - пайпы `|` (вне кавычек)

use std::fmt;

use std::collections::HashMap;

use super::types::{CommandSpec, Pipeline};

/// Результат парсинга одной строки.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ParsedLine {
    pub(crate) assignments: Vec<(String, String)>,
    pub(crate) pipeline: Option<Pipeline>,
}

/// Ошибка парсинга.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum ParseError {
    /// В строке есть незакрытая кавычка.
    UnclosedQuote(char),
    /// Пайп встречен там, где ожидается команда.
    EmptyPipelineSegment,
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ParseError::UnclosedQuote(q) => write!(f, "unclosed quote: {q}"),
            ParseError::EmptyPipelineSegment => write!(f, "empty pipeline segment"),
        }
    }
}

impl std::error::Error for ParseError {}

/// Парсит одну строку пользовательского ввода.
///
/// `base_env` используется для подстановок `$NAME`. Присваивания `NAME=value`
/// в начале строки влияют на подстановки далее по этой же строке.
pub(crate) fn parse_line(
    line: &str,
    base_env: &HashMap<String, String>,
) -> Result<ParsedLine, ParseError> {
    let expanded = expand_line(line, base_env)?;
    let tokens = tokenize_with_pipes_and_quotes(&expanded)?;

    let (assignments, tokens) = split_assignments_prefix(tokens);

    if tokens.is_empty() {
        return Ok(ParsedLine {
            assignments,
            pipeline: None,
        });
    }

    let pipeline = parse_pipeline(tokens)?;
    Ok(ParsedLine {
        assignments,
        pipeline: Some(pipeline),
    })
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum Token {
    Word(String),
    Pipe,
}

type Assignments = Vec<(String, String)>;
type Tokens = Vec<Token>;

fn parse_pipeline(tokens: Vec<Token>) -> Result<Pipeline, ParseError> {
    let mut commands = Vec::new();
    let mut current: Vec<String> = Vec::new();

    for tok in tokens {
        match tok {
            Token::Word(w) => current.push(w),
            Token::Pipe => {
                if current.is_empty() {
                    return Err(ParseError::EmptyPipelineSegment);
                }
                let name = current.remove(0);
                let args = current;
                commands.push(CommandSpec { name, args });
                current = Vec::new();
            }
        }
    }

    if current.is_empty() {
        return Err(ParseError::EmptyPipelineSegment);
    }
    let name = current.remove(0);
    let args = current;
    commands.push(CommandSpec { name, args });

    Ok(Pipeline { commands })
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

/// Выполняет подстановки `$NAME` по строке, сохраняя кавычки.
///
/// Подстановка выполняется:
/// - в обычном режиме и в двойных кавычках
/// - не выполняется в одинарных кавычках
///
/// Присваивания `NAME=value` в начале строки влияют на подстановки дальше
/// в этой же строке (обрабатываются слева направо).
fn expand_line(input: &str, base_env: &HashMap<String, String>) -> Result<String, ParseError> {
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    enum Mode {
        Normal,
        InSingleQuote,
        InDoubleQuote,
    }

    let mut out = String::new();
    let mut env: HashMap<String, String> = base_env.clone();
    let mut in_assignment_prefix = true;

    // Для распознавания присваиваний нужен текущий "word" без кавычек.
    let mut current_assignment_word = String::new();
    let mut assignment_word_started = false;

    let mut mode = Mode::Normal;

    let mut chars = input.chars().peekable();

    let finish_assignment_word =
        |env: &mut HashMap<String, String>,
         in_assignment_prefix: &mut bool,
         current_assignment_word: &mut String,
         assignment_word_started: &mut bool| {
            if !*assignment_word_started {
                return;
            }
            let word = std::mem::take(current_assignment_word);
            *assignment_word_started = false;

            if *in_assignment_prefix {
                if let Some((k, v)) = parse_assignment(&word) {
                    env.insert(k, v);
                } else {
                    *in_assignment_prefix = false;
                }
            }
        };
    while let Some(ch) = chars.next() {
        match mode {
            Mode::Normal => match ch {
                ' ' | '\t' => {
                    finish_assignment_word(
                        &mut env,
                        &mut in_assignment_prefix,
                        &mut current_assignment_word,
                        &mut assignment_word_started,
                    );

                    out.push(ch);
                    while matches!(chars.peek(), Some(' ' | '\t')) {
                        let _ = chars.next();
                        out.push(ch);
                    }
                }
                '|' => {
                    finish_assignment_word(
                        &mut env,
                        &mut in_assignment_prefix,
                        &mut current_assignment_word,
                        &mut assignment_word_started,
                    );
                    in_assignment_prefix = false;
                    out.push('|');
                }
                '\'' => {
                    mode = Mode::InSingleQuote;
                    out.push('\'');
                    assignment_word_started = true;
                }
                '"' => {
                    mode = Mode::InDoubleQuote;
                    out.push('"');
                    assignment_word_started = true;
                }
                '$' => {
                    if let Some(name) = try_read_var_name(&mut chars) {
                        let val = env.get(&name).map(|s| s.as_str()).unwrap_or("");
                        out.push_str(val);
                        current_assignment_word.push_str(val);
                        assignment_word_started = true;
                    } else {
                        out.push('$');
                        current_assignment_word.push('$');
                        assignment_word_started = true;
                    }
                }
                _ => {
                    out.push(ch);
                    current_assignment_word.push(ch);
                    assignment_word_started = true;
                }
            },
            Mode::InSingleQuote => {
                if ch == '\'' {
                    mode = Mode::Normal;
                    out.push('\'');
                } else {
                    out.push(ch);
                    current_assignment_word.push(ch);
                    assignment_word_started = true;
                }
            }
            Mode::InDoubleQuote => {
                if ch == '"' {
                    mode = Mode::Normal;
                    out.push('"');
                } else if ch == '$' {
                    if let Some(name) = try_read_var_name(&mut chars) {
                        let val = env.get(&name).map(|s| s.as_str()).unwrap_or("");
                        out.push_str(val);
                        current_assignment_word.push_str(val);
                        assignment_word_started = true;
                    } else {
                        out.push('$');
                        current_assignment_word.push('$');
                        assignment_word_started = true;
                    }
                } else {
                    out.push(ch);
                    current_assignment_word.push(ch);
                    assignment_word_started = true;
                }
            }
        }
    }

    match mode {
        Mode::Normal => {
            finish_assignment_word(
                &mut env,
                &mut in_assignment_prefix,
                &mut current_assignment_word,
                &mut assignment_word_started,
            );
            Ok(out)
        }
        Mode::InSingleQuote => Err(ParseError::UnclosedQuote('\'')),
        Mode::InDoubleQuote => Err(ParseError::UnclosedQuote('"')),
    }
}

/// Превращает строку (уже после expand) в токены с учетом кавычек и `|`.
///
/// Кавычки удаляются (quote removal), как описано в архитектуре.
fn tokenize_with_pipes_and_quotes(input: &str) -> Result<Tokens, ParseError> {
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    enum Mode {
        Normal,
        InSingleQuote,
        InDoubleQuote,
    }

    let mut tokens: Tokens = Vec::new();
    let mut current = String::new();
    let mut mode = Mode::Normal;
    let mut token_started = false;

    let mut chars = input.chars().peekable();
    while let Some(ch) = chars.next() {
        match mode {
            Mode::Normal => match ch {
                ' ' | '\t' => {
                    if token_started {
                        tokens.push(Token::Word(std::mem::take(&mut current)));
                        token_started = false;
                    }
                    while matches!(chars.peek(), Some(' ' | '\t')) {
                        let _ = chars.next();
                    }
                }
                '|' => {
                    if token_started {
                        tokens.push(Token::Word(std::mem::take(&mut current)));
                        token_started = false;
                    }
                    tokens.push(Token::Pipe);
                }
                '\'' => {
                    mode = Mode::InSingleQuote;
                    token_started = true;
                }
                '"' => {
                    mode = Mode::InDoubleQuote;
                    token_started = true;
                }
                _ => {
                    current.push(ch);
                    token_started = true;
                }
            },
            Mode::InSingleQuote => {
                if ch == '\'' {
                    mode = Mode::Normal;
                } else {
                    current.push(ch);
                    token_started = true;
                }
            }
            Mode::InDoubleQuote => {
                if ch == '"' {
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
                tokens.push(Token::Word(current));
            }
            Ok(tokens)
        }
        Mode::InSingleQuote => Err(ParseError::UnclosedQuote('\'')),
        Mode::InDoubleQuote => Err(ParseError::UnclosedQuote('"')),
    }
}

fn split_assignments_prefix(tokens: Tokens) -> (Assignments, Tokens) {
    let mut assignments = Vec::new();
    let mut idx = 0;
    while idx < tokens.len() {
        match &tokens[idx] {
            Token::Word(w) => {
                if let Some((k, v)) = parse_assignment(w) {
                    assignments.push((k, v));
                    idx += 1;
                    continue;
                }
                break;
            }
            Token::Pipe => break,
        }
    }

    (assignments, tokens.into_iter().skip(idx).collect())
}

fn try_read_var_name<I>(chars: &mut std::iter::Peekable<I>) -> Option<String>
where
    I: Iterator<Item = char>,
{
    let first = chars.peek().copied()?;
    if !(first == '_' || first.is_ascii_alphabetic()) {
        return None;
    }

    let mut name = String::new();
    name.push(chars.next().unwrap());
    while let Some(c) = chars.peek().copied() {
        if c == '_' || c.is_ascii_alphanumeric() {
            name.push(chars.next().unwrap());
        } else {
            break;
        }
    }
    Some(name)
}
