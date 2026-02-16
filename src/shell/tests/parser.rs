//! Unit-тесты для парсера командной строки.

use super::super::parser::{ParseError, parse_line};
use std::collections::HashMap;

#[test]
fn tokenizes_basic_words() {
    let env = HashMap::new();
    let parsed = parse_line("echo hello world", &env).unwrap();
    let pipeline = parsed.pipeline.unwrap();
    assert_eq!(pipeline.commands.len(), 1);
    assert_eq!(pipeline.commands[0].name, "echo");
    assert_eq!(pipeline.commands[0].args, vec!["hello", "world"]);
}

#[test]
fn tokenizes_quotes_as_single_arg() {
    let env = HashMap::new();
    let parsed = parse_line("echo \"Hello, world!\"", &env).unwrap();
    let cmd = &parsed.pipeline.unwrap().commands[0];
    assert_eq!(cmd.args, vec!["Hello, world!"]);
}

#[test]
fn tokenizes_single_quotes_as_single_arg() {
    let env = HashMap::new();
    let parsed = parse_line("echo 'a b'", &env).unwrap();
    let cmd = &parsed.pipeline.unwrap().commands[0];
    assert_eq!(cmd.args, vec!["a b"]);
}

#[test]
fn preserves_empty_quoted_argument() {
    let env = HashMap::new();
    let parsed = parse_line("echo \"\" x", &env).unwrap();
    let cmd = &parsed.pipeline.unwrap().commands[0];
    assert_eq!(cmd.args, vec!["", "x"]);
}

#[test]
fn parses_assignments_only() {
    let env = HashMap::new();
    let parsed = parse_line("FILE=example.txt", &env).unwrap();
    assert_eq!(
        parsed.assignments,
        vec![("FILE".into(), "example.txt".into())]
    );
    assert!(parsed.pipeline.is_none());
}

#[test]
fn parses_assignments_before_command() {
    let env = HashMap::new();
    let parsed = parse_line("x=ex y=it echo ok", &env).unwrap();
    assert_eq!(parsed.assignments.len(), 2);
    let cmd = &parsed.pipeline.unwrap().commands[0];
    assert_eq!(cmd.name, "echo");
    assert_eq!(cmd.args, vec!["ok"]);
}

#[test]
fn stops_parsing_assignments_on_invalid_name() {
    // `1x=...` невалидно как имя переменной => это команда, а не assignment.
    let env = HashMap::new();
    let parsed = parse_line("1x=bad echo ok", &env).unwrap();
    assert!(parsed.assignments.is_empty());
    let cmd = &parsed.pipeline.unwrap().commands[0];
    assert_eq!(cmd.name, "1x=bad");
    assert_eq!(cmd.args, vec!["echo", "ok"]);
}

#[test]
fn errors_on_unclosed_quote_double() {
    let env = HashMap::new();
    let err = parse_line("echo \"oops", &env).unwrap_err();
    assert_eq!(err, ParseError::UnclosedQuote('"'));
}

#[test]
fn errors_on_unclosed_quote_single() {
    let env = HashMap::new();
    let err = parse_line("echo 'oops", &env).unwrap_err();
    assert_eq!(err, ParseError::UnclosedQuote('\''));
}

#[test]
fn expands_vars_outside_single_quotes() {
    let mut env = HashMap::new();
    env.insert("FOO".to_string(), "bar".to_string());

    let parsed = parse_line("echo $FOO \"$FOO\"", &env).unwrap();
    let cmd = &parsed.pipeline.unwrap().commands[0];
    assert_eq!(cmd.args, vec!["bar", "bar"]);
}

#[test]
fn expansion_outside_quotes_splits_on_whitespace_from_value() {
    let mut env = HashMap::new();
    env.insert("FOO".to_string(), "a b".to_string());

    let parsed = parse_line("echo $FOO", &env).unwrap();
    let cmd = &parsed.pipeline.unwrap().commands[0];
    assert_eq!(cmd.args, vec!["a", "b"]);
}

#[test]
fn expansion_inside_double_quotes_does_not_split_on_whitespace_from_value() {
    let mut env = HashMap::new();
    env.insert("FOO".to_string(), "a b".to_string());

    let parsed = parse_line("echo \"$FOO\"", &env).unwrap();
    let cmd = &parsed.pipeline.unwrap().commands[0];
    assert_eq!(cmd.args, vec!["a b"]);
}

#[test]
fn does_not_expand_in_single_quotes() {
    let mut env = HashMap::new();
    env.insert("FOO".to_string(), "bar".to_string());

    let parsed = parse_line("echo '$FOO'", &env).unwrap();
    let cmd = &parsed.pipeline.unwrap().commands[0];
    assert_eq!(cmd.args, vec!["$FOO"]);
}

#[test]
fn parses_pipelines() {
    let env = HashMap::new();
    let parsed = parse_line("echo hi | wc", &env).unwrap();
    let pipeline = parsed.pipeline.unwrap();
    assert_eq!(pipeline.commands.len(), 2);
    assert_eq!(pipeline.commands[0].name, "echo");
    assert_eq!(pipeline.commands[1].name, "wc");
}

#[test]
fn assignments_affect_expansion_later_in_line() {
    let env = HashMap::new();
    let parsed = parse_line("x=ex y=it echo $x$y", &env).unwrap();
    let cmd = &parsed.pipeline.unwrap().commands[0];
    assert_eq!(cmd.name, "echo");
    assert_eq!(cmd.args, vec!["exit"]);
}

#[test]
fn errors_on_empty_pipeline_segment() {
    let env = HashMap::new();
    let err = parse_line("echo hi | | wc", &env).unwrap_err();
    assert_eq!(err, ParseError::EmptyPipelineSegment);
}
