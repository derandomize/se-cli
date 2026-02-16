//! Unit-тесты для внутренних функций цикла REPL.

use super::super::executor::StdProcessExecutor;
use super::super::types::{IoStreams, ShellControl, ShellError};
use super::super::{ShellState, run_single_line};

#[test]
fn empty_or_whitespace_line_is_noop() {
    let executor = StdProcessExecutor::new();
    let mut state = ShellState::new_from_process_env();
    let mut out = Vec::new();
    let mut err = Vec::new();
    let mut io = IoStreams {
        stdout: &mut out,
        stderr: &mut err,
    };

    let control = run_single_line(&executor, &mut state, "   ", &mut io).unwrap();
    assert_eq!(control, ShellControl::Continue(0));
    assert!(out.is_empty());
    assert!(err.is_empty());
}

#[test]
fn assignments_only_update_env_and_do_not_execute() {
    let executor = StdProcessExecutor::new();
    let mut state = ShellState::new_from_process_env();
    let mut out = Vec::new();
    let mut err = Vec::new();
    let mut io = IoStreams {
        stdout: &mut out,
        stderr: &mut err,
    };

    let control = run_single_line(&executor, &mut state, "FOO=bar", &mut io).unwrap();
    assert_eq!(control, ShellControl::Continue(0));
    assert_eq!(state.env.get("FOO").map(|s| s.as_str()), Some("bar"));
    assert!(out.is_empty());
    assert!(err.is_empty());
}

#[test]
fn parse_error_is_returned_from_run_single_line() {
    let executor = StdProcessExecutor::new();
    let mut state = ShellState::new_from_process_env();
    let mut out = Vec::new();
    let mut err = Vec::new();
    let mut io = IoStreams {
        stdout: &mut out,
        stderr: &mut err,
    };

    let e = run_single_line(&executor, &mut state, "echo \"oops", &mut io).unwrap_err();
    match e {
        ShellError::Parse(_) => {}
        other => panic!("expected parse error, got: {other}"),
    }
}
