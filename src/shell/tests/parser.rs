//! Unit-тесты для парсера командной строки.

use super::super::parser::{ParseError, parse_line};

#[test]
fn tokenizes_basic_words() {
    let parsed = parse_line("echo hello world").unwrap();
    let cmd = parsed.command.unwrap();
    assert_eq!(cmd.name, "echo");
    assert_eq!(cmd.args, vec!["hello", "world"]);
}

#[test]
fn tokenizes_quotes_as_single_arg() {
    let parsed = parse_line("echo \"Hello, world!\"").unwrap();
    let cmd = parsed.command.unwrap();
    assert_eq!(cmd.args, vec!["Hello, world!"]);
}

#[test]
fn tokenizes_single_quotes_as_single_arg() {
    let parsed = parse_line("echo 'a b'").unwrap();
    let cmd = parsed.command.unwrap();
    assert_eq!(cmd.args, vec!["a b"]);
}

#[test]
fn preserves_empty_quoted_argument() {
    let parsed = parse_line("echo \"\" x").unwrap();
    let cmd = parsed.command.unwrap();
    assert_eq!(cmd.args, vec!["", "x"]);
}

#[test]
fn parses_assignments_only() {
    let parsed = parse_line("FILE=example.txt").unwrap();
    assert_eq!(
        parsed.assignments,
        vec![("FILE".into(), "example.txt".into())]
    );
    assert!(parsed.command.is_none());
}

#[test]
fn parses_assignments_before_command() {
    let parsed = parse_line("x=ex y=it echo ok").unwrap();
    assert_eq!(parsed.assignments.len(), 2);
    let cmd = parsed.command.unwrap();
    assert_eq!(cmd.name, "echo");
    assert_eq!(cmd.args, vec!["ok"]);
}

#[test]
fn stops_parsing_assignments_on_invalid_name() {
    // `1x=...` невалидно как имя переменной => это команда, а не assignment.
    let parsed = parse_line("1x=bad echo ok").unwrap();
    assert!(parsed.assignments.is_empty());
    let cmd = parsed.command.unwrap();
    assert_eq!(cmd.name, "1x=bad");
    assert_eq!(cmd.args, vec!["echo", "ok"]);
}

#[test]
fn errors_on_unclosed_quote_double() {
    let err = parse_line("echo \"oops").unwrap_err();
    assert_eq!(err, ParseError::UnclosedQuote('"'));
}

#[test]
fn errors_on_unclosed_quote_single() {
    let err = parse_line("echo 'oops").unwrap_err();
    assert_eq!(err, ParseError::UnclosedQuote('\''));
}
