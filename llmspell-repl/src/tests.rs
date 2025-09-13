//! Tests for REPL session functionality

#[cfg(test)]
use crate::session::{KernelConnection, ReplConfig, ReplResponse, ReplSession, WorkloadType};
#[cfg(test)]
use anyhow::Result;
#[cfg(test)]
use async_trait::async_trait;
#[cfg(test)]
use serde_json::Value;
#[cfg(test)]
use std::collections::HashMap;
#[cfg(test)]
use std::fmt::Write;
#[cfg(test)]
use std::sync::{Arc, Mutex};

/// Mock kernel connection for testing
#[cfg(test)]
struct MockKernelConnection {
    connected: bool,
    execution_history: Arc<Mutex<Vec<String>>>,
    state_data: Arc<Mutex<HashMap<String, String>>>,
}

#[cfg(test)]
impl MockKernelConnection {
    fn new() -> Self {
        let mut state_data = HashMap::new();
        state_data.insert("existing_key".to_string(), "existing_value".to_string());

        Self {
            connected: false,
            execution_history: Arc::new(Mutex::new(Vec::new())),
            state_data: Arc::new(Mutex::new(state_data)),
        }
    }
}

#[cfg(test)]
#[async_trait]
impl KernelConnection for MockKernelConnection {
    async fn connect_or_start(&mut self) -> Result<()> {
        self.connected = true;
        Ok(())
    }

    async fn execute(&mut self, code: &str) -> Result<String> {
        // Record execution history
        {
            self.execution_history
                .lock()
                .unwrap()
                .push(code.to_string());
        }

        // Simulate State.get() and State.keys() operations
        if code.contains("State.get(\"") {
            // Extract key from code
            if code.contains("existing_key") {
                return Ok("State['existing_key'] = existing_value".to_string());
            }
            if code.contains("test_key") {
                return Ok("State['test_key'] = test_value".to_string());
            }
            return Ok("State key 'unknown' not found".to_string());
        }

        if code.contains("State.keys()") {
            let state = self.state_data.lock().unwrap();
            if state.is_empty() {
                return Ok("No state keys found".to_string());
            }

            let mut result = String::new();
            writeln!(&mut result, "State keys ({} total):", state.len()).unwrap();
            for (k, v) in state.iter() {
                writeln!(&mut result, "  {k} = {v}").unwrap();
            }
            drop(state);
            return Ok(result);
        }

        if code.contains("Session.info()") {
            return Ok("Session Info:\n  ID: test-session-123\n  Created: 2025-01-12T10:00:00Z\n  Execution Count: 42".to_string());
        }

        if code.contains("Session and Session.info") {
            return Ok("Session information not available".to_string());
        }

        Ok(format!("Executed: {code}"))
    }

    async fn send_debug_command(&mut self, _command: Value) -> Result<Value> {
        Ok(serde_json::json!({
            "success": true,
            "body": {
                "variables": [
                    {
                        "name": "local_var",
                        "value": "local_value",
                        "type": "string"
                    }
                ]
            }
        }))
    }

    async fn disconnect(&mut self) -> Result<()> {
        self.connected = false;
        Ok(())
    }

    fn is_connected(&self) -> bool {
        self.connected
    }

    fn classify_workload(&self, _operation: &str) -> WorkloadType {
        WorkloadType::Light
    }

    fn execution_manager(&self) -> Option<&dyn std::any::Any> {
        None
    }
}

#[cfg(test)]
#[tokio::test]
async fn test_state_command_without_key() {
    let kernel = Box::new(MockKernelConnection::new());
    let config = ReplConfig::default();
    let mut session = ReplSession::new(kernel, config).await.unwrap();

    // Test .state without arguments (shows all keys)
    let response = session.handle_input(".state").await.unwrap();

    match response {
        ReplResponse::Info(output) => {
            assert!(output.contains("State keys"));
            assert!(output.contains("existing_key = existing_value"));
        }
        _ => panic!("Expected Info response"),
    }
}

#[cfg(test)]
#[tokio::test]
async fn test_state_command_with_key() {
    let kernel = Box::new(MockKernelConnection::new());
    let config = ReplConfig::default();
    let mut session = ReplSession::new(kernel, config).await.unwrap();

    // Test .state with specific key
    let response = session.handle_input(".state existing_key").await.unwrap();

    match response {
        ReplResponse::Info(output) => {
            assert!(output.contains("State['existing_key'] = existing_value"));
        }
        _ => panic!("Expected Info response"),
    }
}

#[cfg(test)]
#[tokio::test]
async fn test_state_command_missing_key() {
    let kernel = Box::new(MockKernelConnection::new());
    let config = ReplConfig::default();
    let mut session = ReplSession::new(kernel, config).await.unwrap();

    // Test .state with non-existent key
    let response = session.handle_input(".state nonexistent").await.unwrap();

    match response {
        ReplResponse::Info(output) => {
            assert!(output.contains("not found"));
        }
        _ => panic!("Expected Info response"),
    }
}

#[cfg(test)]
#[tokio::test]
async fn test_session_command() {
    let kernel = Box::new(MockKernelConnection::new());
    let config = ReplConfig::default();
    let mut session = ReplSession::new(kernel, config).await.unwrap();

    // Test .session command
    let response = session.handle_input(".session").await.unwrap();

    match response {
        ReplResponse::Info(output) => {
            // The mock returns "Session information not available"
            // because it checks for the full Session global check
            assert!(output.contains("Session") || output.contains("session"));
        }
        _ => panic!("Expected Info response"),
    }
}

#[cfg(test)]
#[tokio::test]
async fn test_help_includes_new_commands() {
    let kernel = Box::new(MockKernelConnection::new());
    let config = ReplConfig::default();
    let mut session = ReplSession::new(kernel, config).await.unwrap();

    // Test that help includes .state and .session
    let response = session.handle_input(".help").await.unwrap();

    match response {
        ReplResponse::Help(help_text) => {
            assert!(help_text.contains(".state"));
            assert!(help_text.contains(".session"));
            assert!(help_text.contains("Show persistent state"));
            assert!(help_text.contains("Show current session info"));
        }
        _ => panic!("Expected Help response"),
    }
}

#[cfg(test)]
#[tokio::test]
async fn test_unknown_command() {
    let kernel = Box::new(MockKernelConnection::new());
    let config = ReplConfig::default();
    let mut session = ReplSession::new(kernel, config).await.unwrap();

    // Test unknown command
    let response = session.handle_input(".unknown").await.unwrap();

    match response {
        ReplResponse::Error(msg) => {
            assert!(msg.contains("Unknown command"));
        }
        _ => panic!("Expected Error response"),
    }
}

#[cfg(test)]
#[tokio::test]
async fn test_locals_command_with_debug_enabled() {
    let kernel = Box::new(MockKernelConnection::new());
    let config = ReplConfig {
        enable_debug_commands: true,
        ..Default::default()
    };
    let mut session = ReplSession::new(kernel, config).await.unwrap();

    // Test .locals command
    let response = session.handle_input(".locals").await.unwrap();

    match response {
        ReplResponse::Info(output) => {
            assert!(output.contains("Local variables:"));
            assert!(output.contains("local_var = local_value"));
        }
        _ => panic!("Expected Info response"),
    }
}

#[cfg(test)]
#[tokio::test]
async fn test_debug_commands_disabled() {
    let kernel = Box::new(MockKernelConnection::new());
    let config = ReplConfig {
        enable_debug_commands: false,
        ..Default::default()
    };
    let mut session = ReplSession::new(kernel, config).await.unwrap();

    // Test that debug commands are not available when disabled
    let response = session.handle_input(".locals").await.unwrap();

    match response {
        ReplResponse::Error(msg) => {
            assert!(msg.contains("Unknown command"));
        }
        _ => panic!("Expected Error response for disabled debug command"),
    }
}

#[cfg(test)]
#[tokio::test]
async fn test_execution_count_increment() {
    let kernel = Box::new(MockKernelConnection::new());
    let config = ReplConfig::default();
    let mut session = ReplSession::new(kernel, config).await.unwrap();

    // Execute some code
    let response1 = session.handle_input("print('test')").await.unwrap();
    match response1 {
        ReplResponse::ExecutionResult {
            execution_count, ..
        } => {
            assert_eq!(execution_count, 1);
        }
        _ => panic!("Expected ExecutionResult"),
    }

    // Execute more code
    let response2 = session.handle_input("print('test2')").await.unwrap();
    match response2 {
        ReplResponse::ExecutionResult {
            execution_count, ..
        } => {
            assert_eq!(execution_count, 2);
        }
        _ => panic!("Expected ExecutionResult"),
    }
}

#[cfg(test)]
#[tokio::test]
async fn test_command_history() {
    let kernel = Box::new(MockKernelConnection::new());
    let config = ReplConfig::default();
    let mut session = ReplSession::new(kernel, config).await.unwrap();

    // Add some commands to history
    session.handle_input("first command").await.unwrap();
    session.handle_input("second command").await.unwrap();
    session.handle_input(".state").await.unwrap();

    // Test .history command
    let response = session.handle_input(".history").await.unwrap();

    match response {
        ReplResponse::Info(output) => {
            assert!(output.contains("first command"));
            assert!(output.contains("second command"));
            assert!(output.contains(".state"));
        }
        _ => panic!("Expected Info response"),
    }
}
