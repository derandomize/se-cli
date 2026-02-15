//! Интеграционные тесты логики шелла (без пайпов и подстановок).

use std::io::Cursor;

use super::run_repl;

use tempfile::NamedTempFile;

fn run_with_input(input: &str) -> (i32, String, String) {
    let mut out = Vec::new();
    let mut err = Vec::new();
    let code = run_repl(Cursor::new(input.as_bytes()), &mut out, &mut err);
    (
        code,
        String::from_utf8_lossy(&out).to_string(),
        String::from_utf8_lossy(&err).to_string(),
    )
}

#[test]
fn repl_exit_works() {
    let (code, _out, _err) = run_with_input("exit 7\n");
    assert_eq!(code, 7);
}

#[test]
fn echo_prints_arguments() {
    let (code, out, err) = run_with_input("echo hello world\nexit\n");
    assert_eq!(code, 0);
    assert_eq!(out.lines().next().unwrap(), "hello world");
    assert!(err.is_empty());
}

#[test]
fn cat_prints_file_contents() {
    let mut tmp = NamedTempFile::new().unwrap();
    std::io::Write::write_all(&mut tmp, b"hello\n").unwrap();
    let path = tmp.path().to_string_lossy();

    let (_code, out, err) = run_with_input(&format!("cat {path}\nexit\n"));
    assert_eq!(out, "hello\n");
    assert!(err.is_empty());
}

#[test]
fn wc_counts_lines_words_bytes() {
    let mut tmp = NamedTempFile::new().unwrap();
    std::io::Write::write_all(&mut tmp, b"a b\nc\n").unwrap();
    let path = tmp.path().to_string_lossy();

    let (_code, out, err) = run_with_input(&format!("wc {path}\nexit\n"));
    assert_eq!(out.lines().next().unwrap(), "2 3 6");
    assert!(err.is_empty());
}

#[cfg(windows)]
#[test]
fn env_is_passed_to_external_process_windows() {
    let (_code, out, _err) = run_with_input("FOO=bar cmd /C echo %FOO%\nexit\n");
    assert!(out.to_lowercase().contains("bar"));
}

#[cfg(not(windows))]
#[test]
fn env_is_passed_to_external_process_unix() {
    let (_code, out, _err) = run_with_input("FOO=bar sh -c 'echo $FOO'\nexit\n");
    assert!(out.contains("bar"));
}

#[cfg(windows)]
#[test]
fn external_command_runs_on_windows() {
    let (code, out, _err) = run_with_input("cmd /C echo hi\nexit\n");
    assert_eq!(code, 0);
    assert!(out.to_lowercase().contains("hi"));
}

#[cfg(not(windows))]
#[test]
fn external_command_runs_on_unix() {
    let (code, out, _err) = run_with_input("sh -c 'echo hi'\nexit\n");
    assert_eq!(code, 0);
    assert!(out.contains("hi"));
}
