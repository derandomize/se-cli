//! Unit-тесты для builtin-команд.

use std::io::Write;

use super::super::builtins::{Builtin, run_builtin};
use super::super::types::{IoStreams, ShellControl};

fn run(builtin: Builtin, args: &[&str]) -> (ShellControl, String, String) {
    let args: Vec<String> = args.iter().map(|s| s.to_string()).collect();
    let mut out = Vec::new();
    let mut err = Vec::new();
    let mut io = IoStreams {
        stdout: &mut out,
        stderr: &mut err,
    };

    let control = run_builtin(builtin, &args, &mut io).unwrap();
    (
        control,
        String::from_utf8_lossy(&out).to_string(),
        String::from_utf8_lossy(&err).to_string(),
    )
}

#[test]
fn echo_with_args_prints_joined() {
    let (control, out, err) = run(Builtin::Echo, &["hello", "world"]);
    assert_eq!(control, ShellControl::Continue(0));
    assert_eq!(out, "hello world\n");
    assert!(err.is_empty());
}

#[test]
fn echo_without_args_prints_newline() {
    let (control, out, err) = run(Builtin::Echo, &[]);
    assert_eq!(control, ShellControl::Continue(0));
    assert_eq!(out, "\n");
    assert!(err.is_empty());
}

#[test]
fn pwd_prints_current_dir() {
    let cwd = std::env::current_dir().unwrap();
    let (control, out, err) = run(Builtin::Pwd, &[]);
    assert_eq!(control, ShellControl::Continue(0));
    assert!(out.contains(cwd.to_string_lossy().as_ref()));
    assert!(out.ends_with('\n'));
    assert!(err.is_empty());
}

#[test]
fn exit_without_args_is_zero() {
    let (control, out, err) = run(Builtin::Exit, &[]);
    assert_eq!(control, ShellControl::Exit(0));
    assert!(out.is_empty());
    assert!(err.is_empty());
}

#[test]
fn exit_with_number_uses_it() {
    let (control, out, err) = run(Builtin::Exit, &["7"]);
    assert_eq!(control, ShellControl::Exit(7));
    assert!(out.is_empty());
    assert!(err.is_empty());
}

#[test]
fn exit_with_invalid_arg_defaults_to_zero() {
    let (control, _out, _err) = run(Builtin::Exit, &["nope"]);
    assert_eq!(control, ShellControl::Exit(0));
}

#[test]
fn cat_missing_operand_is_error() {
    let (control, out, err) = run(Builtin::Cat, &[]);
    assert_eq!(control, ShellControl::Continue(2));
    assert!(out.is_empty());
    assert!(err.to_lowercase().contains("missing"));
}

#[test]
fn cat_prints_file_contents() {
    let mut tmp = tempfile::NamedTempFile::new().unwrap();
    tmp.write_all(b"hello\n").unwrap();
    let path = tmp.path().to_string_lossy().to_string();

    let (control, out, err) = run(Builtin::Cat, &[&path]);
    assert_eq!(control, ShellControl::Continue(0));
    assert_eq!(out, "hello\n");
    assert!(err.is_empty());
}

#[test]
fn cat_nonexistent_file_sets_exit_code_1() {
    let (control, out, err) = run(Builtin::Cat, &["definitely-not-a-real-file-12345.txt"]);
    assert_eq!(control, ShellControl::Continue(1));
    assert!(out.is_empty());
    assert!(err.starts_with("cat:"));
    assert!(!err.to_lowercase().contains("os error"));
}

#[test]
fn wc_requires_exactly_one_arg() {
    let (control, out, err) = run(Builtin::Wc, &[]);
    assert_eq!(control, ShellControl::Continue(2));
    assert!(out.is_empty());
    assert!(err.contains("wc: missing file operand"));

    let (control, out, err) = run(Builtin::Wc, &["a", "b"]);
    assert_eq!(control, ShellControl::Continue(2));
    assert!(out.is_empty());
    assert!(err.contains("wc: expected exactly one file path"));
}

#[test]
fn wc_counts_lines_words_bytes() {
    let mut tmp = tempfile::NamedTempFile::new().unwrap();
    tmp.write_all(b"a b\nc\n").unwrap();
    let path = tmp.path().to_string_lossy().to_string();

    let (control, out, err) = run(Builtin::Wc, &[&path]);
    assert_eq!(control, ShellControl::Continue(0));
    assert_eq!(out.trim_end(), "2 3 6");
    assert!(err.is_empty());
}

#[test]
fn wc_nonexistent_file_sets_exit_code_1() {
    let (control, out, err) = run(Builtin::Wc, &["definitely-not-a-real-file-12345.txt"]);
    assert_eq!(control, ShellControl::Continue(1));
    assert!(out.is_empty());
    assert!(err.starts_with("wc:"));
    assert!(!err.to_lowercase().contains("os error"));
}
