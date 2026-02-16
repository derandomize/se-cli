//! Unit-тесты для запуска внешних команд.

use std::collections::HashMap;

use super::super::executor::StdProcessExecutor;

fn process_env_map() -> HashMap<String, String> {
    std::env::vars().collect()
}

#[cfg(windows)]
#[test]
fn run_external_captures_stdout_and_exit_code_windows() {
    let executor = StdProcessExecutor::new();
    let env = process_env_map();
    let args = vec!["/C".to_string(), "echo hi".to_string()];

    let result = executor.run_external("cmd", &args, &env).unwrap();
    assert_eq!(result.exit_code, 0);
    let out = String::from_utf8_lossy(&result.stdout).to_string();
    assert!(out.to_lowercase().contains("hi"));
}

#[cfg(not(windows))]
#[test]
fn run_external_captures_stdout_and_exit_code_unix() {
    let executor = StdProcessExecutor::new();
    let env = process_env_map();
    let args = vec!["-c".to_string(), "echo hi".to_string()];

    let result = executor.run_external("sh", &args, &env).unwrap();
    assert_eq!(result.exit_code, 0);
    let out = String::from_utf8_lossy(&result.stdout).to_string();
    assert!(out.contains("hi"));
}

#[test]
fn run_external_returns_command_not_found_for_missing_program() {
    let executor = StdProcessExecutor::new();
    let env = process_env_map();

    let err = executor
        .run_external("definitely-not-a-command-xyz-12345", &[], &env)
        .unwrap_err();
    let msg = err.to_string().to_lowercase();
    assert!(msg.contains("command not found"));
}
