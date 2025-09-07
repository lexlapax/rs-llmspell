//! Unit tests for DAP Bridge

use llmspell_bridge::debug_state_cache::SharedDebugStateCache;
use llmspell_bridge::execution_bridge::{ExecutionManager, Variable};
use llmspell_kernel::dap_bridge::DAPBridge;
use serde_json::json;
use std::sync::Arc;

/// Create a test `ExecutionManager`
fn create_test_execution_manager() -> Arc<ExecutionManager> {
    let debug_cache = Arc::new(SharedDebugStateCache::new());
    Arc::new(ExecutionManager::new(debug_cache))
}

#[tokio::test]
async fn test_dap_initialize() {
    let execution_manager = create_test_execution_manager();
    let bridge = DAPBridge::new(execution_manager);

    let response = bridge
        .handle_request(json!({
            "seq": 1,
            "type": "request",
            "command": "initialize",
            "arguments": { "adapterId": "llmspell" }
        }))
        .await
        .unwrap();

    assert_eq!(response["success"], true);
    assert_eq!(response["command"], "initialize");
    assert!(response["body"].is_object());

    // Check some capabilities
    let body = &response["body"];
    assert_eq!(body["supportsEvaluateForHovers"], true);
    assert_eq!(body["supportsTerminateRequest"], true);
    assert_eq!(body["supportsStepBack"], false);
}

#[tokio::test]
async fn test_dap_variables_request() {
    let execution_manager = create_test_execution_manager();

    // Set up some test variables
    let test_var = Variable {
        name: "test_var".to_string(),
        value: "42".to_string(),
        var_type: "number".to_string(),
        has_children: false,
        reference: None,
    };

    execution_manager
        .cache_variables("current".to_string(), vec![test_var])
        .await;

    let bridge = DAPBridge::new(execution_manager);

    let response = bridge
        .handle_request(json!({
            "seq": 1,
            "type": "request",
            "command": "variables",
            "arguments": { "variablesReference": 1000 }
        }))
        .await
        .unwrap();

    assert_eq!(response["success"], true);
    assert_eq!(response["command"], "variables");

    let variables = response["body"]["variables"].as_array().unwrap();
    assert_eq!(variables.len(), 1);
    assert_eq!(variables[0]["name"], "test_var");
    assert_eq!(variables[0]["value"], "42");
    assert_eq!(variables[0]["type"], "number");
}

#[tokio::test]
async fn test_dap_breakpoint_commands() {
    let execution_manager = create_test_execution_manager();
    let bridge = DAPBridge::new(execution_manager.clone());

    // Test setting breakpoints
    let response = bridge
        .handle_request(json!({
            "seq": 1,
            "type": "request",
            "command": "setBreakpoints",
            "arguments": {
                "source": {
                    "path": "test.lua"
                },
                "breakpoints": [
                    { "line": 10 },
                    { "line": 20 }
                ]
            }
        }))
        .await
        .unwrap();

    assert_eq!(response["success"], true);
    assert_eq!(response["command"], "setBreakpoints");

    let breakpoints = response["body"]["breakpoints"].as_array().unwrap();
    assert_eq!(breakpoints.len(), 2);
    assert_eq!(breakpoints[0]["verified"], true);
    assert_eq!(breakpoints[0]["line"], 10);
    assert_eq!(breakpoints[1]["line"], 20);

    // Verify breakpoints were added to ExecutionManager
    let actual_breakpoints = execution_manager.get_breakpoints().await;
    assert_eq!(actual_breakpoints.len(), 2);
}

#[tokio::test]
async fn test_dap_stack_trace() {
    let execution_manager = create_test_execution_manager();
    let bridge = DAPBridge::new(execution_manager.clone());

    // Set up a test stack frame
    let frame = llmspell_bridge::execution_bridge::StackFrame {
        id: "frame1".to_string(),
        name: "test_function".to_string(),
        source: "test.lua".to_string(),
        line: 15,
        column: Some(5),
        is_user_code: true,
        locals: Vec::new(),
    };

    execution_manager.set_stack_trace(vec![frame]).await;

    let response = bridge
        .handle_request(json!({
            "seq": 1,
            "type": "request",
            "command": "stackTrace",
            "arguments": {
                "threadId": 1
            }
        }))
        .await
        .unwrap();

    assert_eq!(response["success"], true);
    assert_eq!(response["command"], "stackTrace");

    let stack_frames = response["body"]["stackFrames"].as_array().unwrap();
    assert_eq!(stack_frames.len(), 1);
    assert_eq!(stack_frames[0]["name"], "test_function");
    assert_eq!(stack_frames[0]["line"], 15);
    assert_eq!(stack_frames[0]["column"], 5);
}

#[tokio::test]
async fn test_dap_step_commands() {
    let execution_manager = create_test_execution_manager();
    let bridge = DAPBridge::new(execution_manager.clone());

    // Test continue
    let response = bridge
        .handle_request(json!({
            "seq": 1,
            "type": "request",
            "command": "continue",
            "arguments": {}
        }))
        .await
        .unwrap();

    assert_eq!(response["success"], true);
    assert_eq!(response["command"], "continue");
    assert_eq!(response["body"]["allThreadsContinued"], true);

    // Test step in
    let response = bridge
        .handle_request(json!({
            "seq": 2,
            "type": "request",
            "command": "stepIn",
            "arguments": {}
        }))
        .await
        .unwrap();

    assert_eq!(response["success"], true);
    assert_eq!(response["command"], "stepIn");

    // Test step over (next)
    let response = bridge
        .handle_request(json!({
            "seq": 3,
            "type": "request",
            "command": "next",
            "arguments": {}
        }))
        .await
        .unwrap();

    assert_eq!(response["success"], true);
    assert_eq!(response["command"], "next");

    // Test step out
    let response = bridge
        .handle_request(json!({
            "seq": 4,
            "type": "request",
            "command": "stepOut",
            "arguments": {}
        }))
        .await
        .unwrap();

    assert_eq!(response["success"], true);
    assert_eq!(response["command"], "stepOut");
}

#[tokio::test]
async fn test_dap_pause_and_terminate() {
    let execution_manager = create_test_execution_manager();
    let bridge = DAPBridge::new(execution_manager.clone());

    // Test pause
    let response = bridge
        .handle_request(json!({
            "seq": 1,
            "type": "request",
            "command": "pause",
            "arguments": {}
        }))
        .await
        .unwrap();

    assert_eq!(response["success"], true);
    assert_eq!(response["command"], "pause");

    // Test terminate
    let response = bridge
        .handle_request(json!({
            "seq": 2,
            "type": "request",
            "command": "terminate",
            "arguments": {}
        }))
        .await
        .unwrap();

    assert_eq!(response["success"], true);
    assert_eq!(response["command"], "terminate");
}

#[tokio::test]
async fn test_dap_unsupported_command() {
    let execution_manager = create_test_execution_manager();
    let bridge = DAPBridge::new(execution_manager);

    let response = bridge
        .handle_request(json!({
            "seq": 1,
            "type": "request",
            "command": "unsupportedCommand",
            "arguments": {}
        }))
        .await
        .unwrap();

    assert_eq!(response["success"], false);
    assert_eq!(response["command"], "unsupportedCommand");
    assert!(response["message"]
        .as_str()
        .unwrap()
        .contains("Unsupported"));
}

#[tokio::test]
async fn test_dap_empty_variables() {
    let execution_manager = create_test_execution_manager();
    // Don't cache any variables

    let bridge = DAPBridge::new(execution_manager);

    let response = bridge
        .handle_request(json!({
            "seq": 1,
            "type": "request",
            "command": "variables",
            "arguments": { "variablesReference": 1000 }
        }))
        .await
        .unwrap();

    assert_eq!(response["success"], true);
    assert_eq!(response["command"], "variables");

    let variables = response["body"]["variables"].as_array().unwrap();
    assert_eq!(variables.len(), 0);
}
