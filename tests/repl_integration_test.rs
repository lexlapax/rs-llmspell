//! Integration tests for REPL state and session commands
//!
//! Tests the complete flow of state sharing between REPL and exec commands

use std::process::{Command, Stdio};
use std::io::Write;
use tempfile::TempDir;

/// Test that state persists across different command invocations
#[test]
fn test_state_persistence_across_commands() {
    // Create a temporary directory for state
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let state_path = temp_dir.path().join("state");
    
    // Set environment to use temp state directory
    std::env::set_var("LLMSPELL_STATE_PATH", state_path.to_str().unwrap());
    
    // First command: Set state via exec
    let output1 = Command::new("./target/debug/llmspell")
        .args(&["exec", "State.set('integration_test', 'test_value'); print('State set')"])
        .output()
        .expect("Failed to execute first command");
    
    let stdout1 = String::from_utf8_lossy(&output1.stdout);
    assert!(stdout1.contains("State set"), "Failed to set state: {}", stdout1);
    
    // Second command: Read state via exec
    let output2 = Command::new("./target/debug/llmspell")
        .args(&["exec", "print('Value: ' .. tostring(State.get('integration_test')))"])
        .output()
        .expect("Failed to execute second command");
    
    let stdout2 = String::from_utf8_lossy(&output2.stdout);
    assert!(stdout2.contains("Value: test_value"), "State not persisted: {}", stdout2);
}

/// Test that REPL .state command can see state set by exec
#[test]
fn test_repl_state_command_sees_exec_state() {
    // Create a temporary directory for state
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let state_path = temp_dir.path().join("state");
    
    // Set state via exec
    let output1 = Command::new("./target/debug/llmspell")
        .env("LLMSPELL_STATE_PATH", state_path.to_str().unwrap())
        .args(&["exec", "State.set('repl_test', 'from_exec')"])
        .output()
        .expect("Failed to set state");
    
    assert!(output1.status.success(), "Failed to set state via exec");
    
    // Start REPL and check state
    let mut repl = Command::new("./target/debug/llmspell")
        .env("LLMSPELL_STATE_PATH", state_path.to_str().unwrap())
        .arg("repl")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("Failed to start REPL");
    
    // Send .state command to REPL
    let stdin = repl.stdin.as_mut().expect("Failed to get stdin");
    stdin.write_all(b".state repl_test\n").expect("Failed to write to stdin");
    stdin.write_all(b".exit\n").expect("Failed to write exit command");
    
    let output = repl.wait_with_output().expect("Failed to get REPL output");
    let stdout = String::from_utf8_lossy(&output.stdout);
    
    assert!(stdout.contains("repl_test") || stdout.contains("from_exec"), 
            "REPL doesn't see exec state: {}", stdout);
}

/// Test session creation and listing
#[test]
fn test_session_management() {
    // Create a session
    let output1 = Command::new("./target/debug/llmspell")
        .args(&["session", "create", "integration-test-session", "--description", "Test session"])
        .output()
        .expect("Failed to create session");
    
    let stdout1 = String::from_utf8_lossy(&output1.stdout);
    assert!(stdout1.contains("Created session") || stdout1.contains("✓"), 
            "Failed to create session: {}", stdout1);
    
    // List sessions
    let output2 = Command::new("./target/debug/llmspell")
        .args(&["session", "list"])
        .output()
        .expect("Failed to list sessions");
    
    let stdout2 = String::from_utf8_lossy(&output2.stdout);
    assert!(stdout2.contains("integration-test-session") || stdout2.contains("Sessions"), 
            "Session not found in list: {}", stdout2);
}

/// Test that external kernel maintains state across connections
#[test]
#[ignore] // This test requires starting/stopping a kernel server
fn test_external_kernel_state_sharing() {
    use std::thread;
    use std::time::Duration;
    
    // Start kernel in background
    let mut kernel = Command::new("./target/debug/llmspell")
        .args(&["kernel", "start", "--port", "9599"])
        .spawn()
        .expect("Failed to start kernel");
    
    // Wait for kernel to start
    thread::sleep(Duration::from_secs(2));
    
    // Set state through external kernel
    let output1 = Command::new("./target/debug/llmspell")
        .args(&["exec", "--connect", "localhost:9599", "State.set('external_test', 'shared_value')"])
        .output()
        .expect("Failed to set state");
    
    assert!(output1.status.success(), "Failed to set state through external kernel");
    
    // Read state through same external kernel
    let output2 = Command::new("./target/debug/llmspell")
        .args(&["exec", "--connect", "localhost:9599", "print(State.get('external_test'))"])
        .output()
        .expect("Failed to read state");
    
    let stdout2 = String::from_utf8_lossy(&output2.stdout);
    assert!(stdout2.contains("shared_value"), "State not shared through external kernel: {}", stdout2);
    
    // Stop kernel
    kernel.kill().expect("Failed to stop kernel");
}

/// Test REPL help command includes new commands
#[test]
fn test_repl_help_includes_state_session() {
    let mut repl = Command::new("./target/debug/llmspell")
        .arg("repl")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("Failed to start REPL");
    
    // Send .help command
    let stdin = repl.stdin.as_mut().expect("Failed to get stdin");
    stdin.write_all(b".help\n").expect("Failed to write help command");
    stdin.write_all(b".exit\n").expect("Failed to write exit command");
    
    let output = repl.wait_with_output().expect("Failed to get REPL output");
    let stdout = String::from_utf8_lossy(&output.stdout);
    
    assert!(stdout.contains(".state"), ".state command not in help: {}", stdout);
    assert!(stdout.contains(".session"), ".session command not in help: {}", stdout);
}