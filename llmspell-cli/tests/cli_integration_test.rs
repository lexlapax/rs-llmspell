//! ABOUTME: Integration tests for the CLI
//! ABOUTME: Tests end-to-end CLI functionality

use assert_cmd::Command;
use predicates::prelude::*;
use serial_test::serial;
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
#[serial]
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
#[serial]
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
#[serial]
fn test_run_missing_file() {
    let mut cmd = Command::cargo_bin("llmspell").unwrap();
    cmd.arg("run")
        .arg("nonexistent.lua")
        .assert()
        .failure()
        .stderr(predicate::str::contains("Script file not found"));
}
#[test]
#[serial]
fn test_run_simple_lua_script() {
    let dir = tempdir().unwrap();
    let script_path = dir.path().join("test.lua");
    fs::write(&script_path, "print('Hello from test!')").unwrap();

    let mut cmd = Command::cargo_bin("llmspell").unwrap();
    cmd.arg("run").arg(&script_path).assert().success();
}
#[test]
#[serial]
fn test_exec_inline_code() {
    let mut cmd = Command::cargo_bin("llmspell").unwrap();
    cmd.arg("exec")
        .arg("print('Inline execution works!')")
        .assert()
        .success()
        .stdout(predicate::str::contains("Inline execution works!"));
}
#[test]
#[serial]
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

// Memory command tests
#[test]
fn test_memory_help() {
    let mut cmd = Command::cargo_bin("llmspell").unwrap();
    cmd.arg("memory")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("episodic and semantic memory"));
}

#[test]
fn test_memory_add_help() {
    let mut cmd = Command::cargo_bin("llmspell").unwrap();
    cmd.arg("memory")
        .arg("add")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("episodic memory"));
}

#[test]
fn test_memory_search_help() {
    let mut cmd = Command::cargo_bin("llmspell").unwrap();
    cmd.arg("memory")
        .arg("search")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Search episodic memory"));
}

#[test]
fn test_memory_query_help() {
    let mut cmd = Command::cargo_bin("llmspell").unwrap();
    cmd.arg("memory")
        .arg("query")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("semantic knowledge graph"));
}

#[test]
fn test_memory_stats_help() {
    let mut cmd = Command::cargo_bin("llmspell").unwrap();
    cmd.arg("memory")
        .arg("stats")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("memory system statistics"));
}

#[test]
fn test_memory_consolidate_help() {
    let mut cmd = Command::cargo_bin("llmspell").unwrap();
    cmd.arg("memory")
        .arg("consolidate")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Trigger consolidation"));
}

// Context command tests
#[test]
fn test_context_help() {
    let mut cmd = Command::cargo_bin("llmspell").unwrap();
    cmd.arg("context")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Assemble context"));
}

#[test]
fn test_context_assemble_help() {
    let mut cmd = Command::cargo_bin("llmspell").unwrap();
    cmd.arg("context")
        .arg("assemble")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("retrieval strategy"));
}

#[test]
fn test_context_strategies_help() {
    let mut cmd = Command::cargo_bin("llmspell").unwrap();
    cmd.arg("context")
        .arg("strategies")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "available context assembly strategies",
        ));
}

#[test]
fn test_context_analyze_help() {
    let mut cmd = Command::cargo_bin("llmspell").unwrap();
    cmd.arg("context")
        .arg("analyze")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("estimated token usage"));
}
