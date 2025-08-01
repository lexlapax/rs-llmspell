// ABOUTME: Rust test runner for Lua tool integration tests
// ABOUTME: Executes Lua scripts to validate tool functionality from scripts

use llmspell_bridge::runtime::{RuntimeConfig, ScriptRuntime};
use llmspell_tools::registry::ToolRegistry;
use std::path::PathBuf;

#[cfg_attr(test_category = "integration")]
#[cfg_attr(test_category = "tool")]
#[tokio::test]
async fn test_lua_tool_integration() {
    // Initialize tool registry
    let registry = ToolRegistry::new();
    registry.register_default_tools().expect("Failed to register tools");
    
    // Create runtime with Lua
    let config = RuntimeConfig::default();
    let runtime = ScriptRuntime::new_with_lua(config)
        .await
        .expect("Failed to create Lua runtime");
    
    // Load and execute the integration test script
    let test_script_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .join("llmspell-testing/fixtures/lua/lua_tool_integration.lua");
    
    let script_content = std::fs::read_to_string(&test_script_path)
        .expect("Failed to read test script");
    
    // Execute the script
    let result = runtime.execute_script(&script_content).await;
    
    match result {
        Ok(output) => {
            println!("Test output:\n{}", output.output);
            assert!(
                !output.output.as_str().unwrap_or("").contains("failed"),
                "Tests should not fail"
            );
        }
        Err(e) => {
            panic!("Script execution failed: {:?}", e);
        }
    }
}

#[cfg_attr(test_category = "integration")]
#[cfg_attr(test_category = "tool")]
#[tokio::test]
async fn test_tool_response_format_consistency() {
    use llmspell_core::traits::base_agent::BaseAgent;
    use llmspell_core::types::AgentInput;
    use llmspell_core::ExecutionContext;
    use serde_json::json;
    
    // Test that all refactored tools return consistent response format
    let tools = vec![
        "hash-calculator",
        "base64-encoder",
        "uuid-generator",
        "text-manipulator",
        "calculator",
        "datetime-handler",
        "diff-calculator",
        "data-validation",
        "template-engine",
    ];
    
    let registry = ToolRegistry::new();
    registry.register_default_tools().unwrap();
    
    for tool_name in tools {
        let tool = registry.get_tool(tool_name)
            .unwrap_or_else(|| panic!("Tool {} not found", tool_name));
        
        // Create a valid input for each tool
        let input = match tool_name {
            "hash-calculator" => AgentInput::text("test").with_parameter(
                "parameters",
                json!({
                    "algorithm": "md5",
                    "input": "test"
                })
            ),
            "base64-encoder" => AgentInput::text("test").with_parameter(
                "parameters",
                json!({
                    "operation": "encode",
                    "input": "test"
                })
            ),
            "uuid-generator" => AgentInput::text("test").with_parameter(
                "parameters",
                json!({
                    "version": "v4",
                    "count": 1
                })
            ),
            "text-manipulator" => AgentInput::text("test").with_parameter(
                "parameters",
                json!({
                    "operation": "case",
                    "input": "test",
                    "format": "upper"
                })
            ),
            "calculator" => AgentInput::text("test").with_parameter(
                "parameters",
                json!({
                    "operation": "evaluate",
                    "expression": "1 + 1"
                })
            ),
            "datetime-handler" => AgentInput::text("test").with_parameter(
                "parameters",
                json!({
                    "operation": "now"
                })
            ),
            "diff-calculator" => AgentInput::text("test").with_parameter(
                "parameters",
                json!({
                    "left": "a",
                    "right": "b",
                    "format": "unified"
                })
            ),
            "data-validation" => AgentInput::text("test").with_parameter(
                "parameters",
                json!({
                    "data": {"test": "value"},
                    "schema": {
                        "type": "object",
                        "properties": {
                            "test": {"type": "string"}
                        }
                    }
                })
            ),
            "template-engine" => AgentInput::text("test").with_parameter(
                "parameters",
                json!({
                    "template": "Hello {{name}}",
                    "data": {"name": "World"},
                    "engine": "handlebars"
                })
            ),
            _ => panic!("Unknown tool: {}", tool_name),
        };
        
        let result = tool.execute(input, ExecutionContext::default()).await;
        
        // All tools should succeed with valid input
        assert!(result.is_ok(), "Tool {} failed: {:?}", tool_name, result);
        
        let output = result.unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&output.text)
            .unwrap_or_else(|e| panic!("Tool {} returned invalid JSON: {}", tool_name, e));
        
        // Check consistent response structure
        assert!(parsed.get("success").is_some(), 
                "Tool {} missing 'success' field", tool_name);
        assert!(parsed.get("operation").is_some(), 
                "Tool {} missing 'operation' field", tool_name);
        assert!(parsed.get("result").is_some(), 
                "Tool {} missing 'result' field", tool_name);
        
        // Success should be true for valid inputs
        assert_eq!(parsed["success"], true, 
                   "Tool {} should succeed with valid input", tool_name);
    }
}

#[cfg_attr(test_category = "integration")]
#[cfg_attr(test_category = "tool")]
#[tokio::test]
async fn test_tool_error_handling_consistency() {
    use llmspell_core::traits::base_agent::BaseAgent;
    use llmspell_core::types::AgentInput;
    use llmspell_core::ExecutionContext;
    use serde_json::json;
    
    let registry = ToolRegistry::new();
    registry.register_default_tools().unwrap();
    
    // Test missing required parameters
    let test_cases = vec![
        ("hash-calculator", json!({"algorithm": "md5"})), // missing input
        ("base64-encoder", json!({"operation": "encode"})), // missing input
        ("text-manipulator", json!({"operation": "case", "format": "upper"})), // missing input
        ("calculator", json!({"operation": "evaluate"})), // missing expression
        ("diff-calculator", json!({"format": "unified"})), // missing left/right
        ("data-validation", json!({"data": {}})), // missing schema
        ("template-engine", json!({"template": "test"})), // missing data
    ];
    
    for (tool_name, params) in test_cases {
        let tool = registry.get_tool(tool_name).unwrap();
        let input = AgentInput::text("test").with_parameter("parameters", params);
        
        let result = tool.execute(input, ExecutionContext::default()).await;
        
        // Should fail with missing parameters
        assert!(result.is_err(), 
                "Tool {} should fail with missing parameters", tool_name);
        
        let err = result.unwrap_err();
        let err_str = err.to_string();
        
        // Error should mention the missing field
        assert!(err_str.contains("required") || err_str.contains("missing") || err_str.contains("Missing"),
                "Tool {} error should indicate missing parameter: {}", tool_name, err_str);
    }
}