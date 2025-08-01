//! ABOUTME: Integration tests for the CLI
//! ABOUTME: Tests end-to-end CLI functionality

use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::tempdir;

#[cfg_attr(test_category = "integration")]
#[test]
fn test_cli_help() {
    let mut cmd = Command::cargo_bin("llmspell").unwrap();
    cmd.arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "LLMSpell - Scriptable LLM interactions",
        ));
}

#[cfg_attr(test_category = "integration")]
#[test]
fn test_cli_version() {
    let mut cmd = Command::cargo_bin("llmspell").unwrap();
    cmd.arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::contains("llmspell"));
}

#[cfg_attr(test_category = "integration")]
#[test]
fn test_run_command_help() {
    let mut cmd = Command::cargo_bin("llmspell").unwrap();
    cmd.arg("run")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Execute a script file"));
}

#[cfg_attr(test_category = "integration")]
#[test]
fn test_invalid_engine() {
    let mut cmd = Command::cargo_bin("llmspell").unwrap();
    cmd.arg("--engine")
        .arg("ruby")
        .arg("run")
        .arg("test.rb")
        .assert()
        .failure()
        .stderr(predicate::str::contains("invalid value 'ruby'"));
}

#[cfg_attr(test_category = "integration")]
#[test]
fn test_javascript_not_implemented() {
    let mut cmd = Command::cargo_bin("llmspell").unwrap();
    cmd.arg("--engine")
        .arg("javascript")
        .arg("run")
        .arg("test.js")
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "Script engine 'javascript' is not available yet",
        ));
}

#[cfg_attr(test_category = "integration")]
#[test]
fn test_python_not_implemented() {
    let mut cmd = Command::cargo_bin("llmspell").unwrap();
    cmd.arg("--engine")
        .arg("python")
        .arg("run")
        .arg("test.py")
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "Script engine 'python' is not available yet",
        ));
}

#[cfg_attr(test_category = "integration")]
#[test]
fn test_run_missing_file() {
    let mut cmd = Command::cargo_bin("llmspell").unwrap();
    cmd.arg("run")
        .arg("nonexistent.lua")
        .assert()
        .failure()
        .stderr(predicate::str::contains("Script file not found"));
}

#[cfg_attr(test_category = "integration")]
#[test]
fn test_run_simple_lua_script() {
    let dir = tempdir().unwrap();
    let script_path = dir.path().join("test.lua");
    fs::write(&script_path, "print('Hello from test!')").unwrap();

    let mut cmd = Command::cargo_bin("llmspell").unwrap();
    cmd.arg("run")
        .arg(&script_path)
        .assert()
        .success()
        .stdout(predicate::str::contains("Hello from test!"));
}

#[cfg_attr(test_category = "integration")]
#[test]
fn test_exec_inline_code() {
    let mut cmd = Command::cargo_bin("llmspell").unwrap();
    cmd.arg("exec")
        .arg("print('Inline execution works!')")
        .assert()
        .success()
        .stdout(predicate::str::contains("Inline execution works!"));
}

#[cfg_attr(test_category = "integration")]
#[test]
fn test_output_format_json() {
    let mut cmd = Command::cargo_bin("llmspell").unwrap();
    cmd.arg("--output")
        .arg("json")
        .arg("exec")
        .arg("return {result = 42}")
        .assert()
        .success()
        .stdout(predicate::str::contains("\"result\": 42"));
}

#[cfg_attr(test_category = "integration")]
#[test]
fn test_providers_command() {
    let mut cmd = Command::cargo_bin("llmspell").unwrap();
    cmd.arg("providers")
        .assert()
        .success()
        .stdout(predicate::str::contains("Available Providers"));
}

#[cfg_attr(test_category = "integration")]
#[test]
fn test_info_command() {
    let mut cmd = Command::cargo_bin("llmspell").unwrap();
    cmd.arg("info")
        .assert()
        .success()
        .stdout(predicate::str::contains("lua - Available"));
}

#[cfg_attr(test_category = "integration")]
#[test]
fn test_repl_not_implemented() {
    let mut cmd = Command::cargo_bin("llmspell").unwrap();
    cmd.arg("repl")
        .assert()
        .failure()
        .stderr(predicate::str::contains("REPL mode not implemented"));
}

#[cfg_attr(test_category = "integration")]
#[test]
fn test_validate_missing_config() {
    let mut cmd = Command::cargo_bin("llmspell").unwrap();
    cmd.arg("validate")
        .arg("nonexistent.toml")
        .assert()
        .failure();
}
