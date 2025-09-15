//! ABOUTME: Integration tests for the CLI
//! ABOUTME: Tests end-to-end CLI functionality

use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::tempdir;
#[test]
fn test_cli_help() {
    let mut cmd = Command::cargo_bin("llmspell").unwrap();
    cmd.arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "LLMSpell provides scriptable LLM interactions",
        ));
}
#[test]
fn test_cli_version() {
    let mut cmd = Command::cargo_bin("llmspell").unwrap();
    cmd.arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::contains("llmspell"));
}
#[test]
fn test_run_command_help() {
    let mut cmd = Command::cargo_bin("llmspell").unwrap();
    cmd.arg("run")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Execute a script file"));
}
#[test]
fn test_invalid_engine() {
    let mut cmd = Command::cargo_bin("llmspell").unwrap();
    cmd.arg("run")
        .arg("--engine")
        .arg("ruby")
        .arg("test.rb")
        .assert()
        .failure()
        .stderr(predicate::str::contains("invalid value 'ruby'"));
}
#[test]
fn test_javascript_not_implemented() {
    let dir = tempdir().unwrap();
    let script_path = dir.path().join("test.js");
    fs::write(&script_path, "console.log('test')").unwrap();

    let mut cmd = Command::cargo_bin("llmspell").unwrap();
    cmd.arg("run")
        .arg("--engine")
        .arg("javascript")
        .arg(&script_path)
        .assert()
        .failure()
        .stderr(predicate::str::contains("not available yet"));
}
#[test]
fn test_python_not_implemented() {
    let dir = tempdir().unwrap();
    let script_path = dir.path().join("test.py");
    fs::write(&script_path, "print('test')").unwrap();

    let mut cmd = Command::cargo_bin("llmspell").unwrap();
    cmd.arg("run")
        .arg("--engine")
        .arg("python")
        .arg(&script_path)
        .assert()
        .failure()
        .stderr(predicate::str::contains("not available yet"));
}
#[test]
fn test_run_missing_file() {
    let mut cmd = Command::cargo_bin("llmspell").unwrap();
    cmd.arg("run")
        .arg("nonexistent.lua")
        .assert()
        .failure()
        .stderr(predicate::str::contains("Script file not found"));
}
#[test]
fn test_run_simple_lua_script() {
    let dir = tempdir().unwrap();
    let script_path = dir.path().join("test.lua");
    fs::write(&script_path, "print('Hello from test!')").unwrap();

    let mut cmd = Command::cargo_bin("llmspell").unwrap();
    cmd.arg("run").arg(&script_path).assert().success();
}
#[test]
fn test_exec_inline_code() {
    let mut cmd = Command::cargo_bin("llmspell").unwrap();
    cmd.arg("exec")
        .arg("print('Inline execution works!')")
        .assert()
        .success()
        .stdout(predicate::str::contains("Inline execution works!"));
}
#[test]
fn test_output_format_json() {
    let mut cmd = Command::cargo_bin("llmspell").unwrap();
    cmd.arg("--output")
        .arg("json")
        .arg("exec")
        .arg("print('test')")
        .assert()
        .success()
        .stdout(predicate::str::contains("\"status\":\"executed\""));
}
#[test]
fn test_validate_missing_config() {
    let mut cmd = Command::cargo_bin("llmspell").unwrap();
    cmd.arg("config")
        .arg("validate")
        .arg("--file")
        .arg("nonexistent.toml")
        .assert()
        .failure();
}
