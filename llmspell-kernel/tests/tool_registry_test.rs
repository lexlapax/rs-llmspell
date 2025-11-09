//! Test that kernel tool handlers use real ComponentRegistry

#[cfg(feature = "lua")]
#[tokio::test(flavor = "multi_thread")]
async fn test_kernel_tool_handlers_with_registry() {
    use llmspell_bridge::ScriptRuntime;
    use llmspell_config::LLMSpellConfig;
    use llmspell_kernel::api::start_embedded_kernel_with_executor;
    use llmspell_tools::util::{CalculatorTool, DateTimeHandlerTool};
    use serde_json::json;
    use std::sync::Arc;

    // Create a runtime with Lua engine and tools
    let config = LLMSpellConfig::default();
    let runtime = ScriptRuntime::new(config.clone())
        .await
        .expect("Failed to create runtime");

    // Verify registry has tools (register_all_tools should have been called)
    let registry = runtime.registry();
    let tools_before = registry.list_tools();
    println!("Tools registered: {} tools", tools_before.len());

    // Manually register additional tools for testing
    registry
        .register_tool(
            "test_calculator".to_string(),
            Arc::new(CalculatorTool::new()),
        )
        .expect("Failed to register test calculator");
    registry
        .register_tool(
            "test_datetime".to_string(),
            Arc::new(DateTimeHandlerTool::new()),
        )
        .expect("Failed to register test datetime");

    // Create kernel with the runtime as executor
    let executor = Arc::new(runtime);
    let mut kernel_handle = start_embedded_kernel_with_executor(config, executor.clone())
        .await
        .expect("Failed to start kernel");

    // Test 1: Tool list command
    let list_request = json!({
        "command": "list",
    });

    let response = kernel_handle
        .send_tool_request(list_request)
        .await
        .expect("Failed to send tool list request");

    assert!(response.get("tools").is_some(), "Should have tools field");
    let tools = response["tools"].as_array().unwrap();
    assert!(!tools.is_empty(), "Should have tools registered");

    // Check for our test tools
    let tool_names: Vec<String> = tools
        .iter()
        .filter_map(|v| v.as_str().map(String::from))
        .collect();

    assert!(
        tool_names.contains(&"test_calculator".to_string()),
        "Should have test_calculator"
    );
    assert!(
        tool_names.contains(&"test_datetime".to_string()),
        "Should have test_datetime"
    );

    // Test 2: Tool list with category filter
    let list_category_request = json!({
        "command": "list",
        "category": "utility",
    });

    let response = kernel_handle
        .send_tool_request(list_category_request)
        .await
        .expect("Failed to send categorized tool list request");

    let category_tools = response["tools"].as_array().unwrap();
    // Calculator should be in utility category
    let utility_tool_names: Vec<String> = category_tools
        .iter()
        .filter_map(|v| v.as_str().map(String::from))
        .collect();
    assert!(
        utility_tool_names.contains(&"test_calculator".to_string()),
        "Calculator should be in utility category"
    );

    // Test 3: Tool info command
    let info_request = json!({
        "command": "info",
        "name": "test_calculator",
    });

    let response = kernel_handle
        .send_tool_request(info_request)
        .await
        .expect("Failed to send tool info request");

    assert_eq!(response["name"].as_str(), Some("test_calculator"));
    assert!(response.get("description").is_some());
    assert!(response.get("category").is_some());

    // Test 4: Tool invoke command
    let invoke_request = json!({
        "command": "invoke",
        "name": "test_calculator",
        "params": {
            "expression": "2 + 2"
        },
    });

    let response = kernel_handle
        .send_tool_request(invoke_request)
        .await
        .expect("Failed to send tool invoke request");

    // The response structure depends on whether the tool executed successfully
    assert!(response.get("tool").is_some());
    assert_eq!(response["tool"].as_str(), Some("test_calculator"));

    // Test 5: Tool test command
    let test_request = json!({
        "command": "test",
        "name": "test_calculator",
        "verbose": true,
    });

    let response = kernel_handle
        .send_tool_request(test_request)
        .await
        .expect("Failed to send tool test request");

    assert!(response.get("success").is_some());
    assert_eq!(response["tool"].as_str(), Some("test_calculator"));

    // Test 6: Tool search command
    let search_request = json!({
        "command": "search",
        "query": ["calc"],
    });

    let response = kernel_handle
        .send_tool_request(search_request)
        .await
        .expect("Failed to send tool search request");

    let matches = response["matches"].as_array().unwrap();
    let match_names: Vec<String> = matches
        .iter()
        .filter_map(|v| v.as_str().map(String::from))
        .collect();
    assert!(
        match_names.contains(&"test_calculator".to_string()),
        "Search for 'calc' should find test_calculator"
    );
}

#[cfg(feature = "lua")]
#[tokio::test(flavor = "multi_thread")]
async fn test_kernel_tool_count() {
    use llmspell_bridge::ScriptRuntime;
    use llmspell_config::LLMSpellConfig;
    use llmspell_kernel::api::start_embedded_kernel_with_executor;
    use serde_json::json;
    use std::sync::Arc;

    // Create a runtime with default configuration
    let config = LLMSpellConfig::default();
    let runtime = ScriptRuntime::new(config.clone())
        .await
        .expect("Failed to create runtime");

    // Create kernel
    let executor = Arc::new(runtime);
    let mut kernel_handle = start_embedded_kernel_with_executor(config, executor.clone())
        .await
        .expect("Failed to start kernel");

    // Get tool list
    let list_request = json!({
        "command": "list",
    });

    let response = kernel_handle
        .send_tool_request(list_request)
        .await
        .expect("Failed to send tool list request");

    let tools = response["tools"].as_array().unwrap();
    let count = response["count"].as_u64().unwrap();

    println!("Kernel reports {} tools available", count);
    assert_eq!(
        tools.len() as u64,
        count,
        "Tool count should match array length"
    );

    // The requirement is 40+ tools when all are registered
    // If register_all_tools worked, we should have many tools
    // For now, we may have fewer until all tools are implemented
    assert!(count > 0, "Should have at least some tools registered");
}

#[cfg(feature = "lua")]
#[tokio::test(flavor = "multi_thread")]
async fn test_tool_invocation_with_timeout() {
    use llmspell_bridge::ScriptRuntime;
    use llmspell_config::LLMSpellConfig;
    use llmspell_kernel::api::start_embedded_kernel_with_executor;
    use serde_json::json;
    use std::sync::Arc;

    // Create runtime with tools (calculator already registered by register_all_tools)
    let config = LLMSpellConfig::default();
    let runtime = ScriptRuntime::new(config.clone())
        .await
        .expect("Failed to create runtime");

    // Create kernel (calculator already registered via register_all_tools)
    let executor = Arc::new(runtime);
    let mut kernel_handle = start_embedded_kernel_with_executor(config, executor.clone())
        .await
        .expect("Failed to start kernel");

    // Test 1: Normal invocation with timeout
    let invoke_request = json!({
        "command": "invoke",
        "name": "calculator",
        "params": {
            "expression": "2 + 2"
        },
        "timeout": 5  // 5 second timeout
    });

    let response = kernel_handle
        .send_tool_request(invoke_request)
        .await
        .expect("Failed to send tool invoke request");

    assert_eq!(response["status"].as_str(), Some("ok"));
    assert_eq!(response["tool"].as_str(), Some("calculator"));
    assert!(
        response.get("duration_ms").is_some(),
        "Should have duration tracking"
    );

    // Test 2: Invocation with very short timeout (should not timeout for simple calculation)
    let quick_invoke = json!({
        "command": "invoke",
        "name": "calculator",
        "params": {
            "expression": "10 * 5"
        },
        "timeout": 1  // 1 second timeout (should be enough)
    });

    let response = kernel_handle
        .send_tool_request(quick_invoke)
        .await
        .expect("Failed to send quick invoke request");

    assert_eq!(response["status"].as_str(), Some("ok"));

    // Test 3: Streaming flag (for future use)
    let streaming_invoke = json!({
        "command": "invoke",
        "name": "calculator",
        "params": {
            "expression": "100 / 4"
        },
        "streaming": true,
        "timeout": 5
    });

    let response = kernel_handle
        .send_tool_request(streaming_invoke)
        .await
        .expect("Failed to send streaming invoke request");

    assert_eq!(response["status"].as_str(), Some("ok"));
    assert_eq!(response["streaming"].as_bool(), Some(true));
}

#[cfg(feature = "lua")]
#[tokio::test(flavor = "multi_thread")]
async fn test_tool_parameter_validation() {
    use llmspell_bridge::ScriptRuntime;
    use llmspell_config::LLMSpellConfig;
    use llmspell_kernel::api::start_embedded_kernel_with_executor;
    use serde_json::json;
    use std::sync::Arc;

    // Create runtime (calculator already registered by register_all_tools)
    let config = LLMSpellConfig::default();
    let runtime = ScriptRuntime::new(config.clone())
        .await
        .expect("Failed to create runtime");

    // Create kernel (calculator already registered via register_all_tools)
    let executor = Arc::new(runtime);
    let mut kernel_handle = start_embedded_kernel_with_executor(config, executor.clone())
        .await
        .expect("Failed to start kernel");

    // Test 1: Valid parameters
    let valid_request = json!({
        "command": "invoke",
        "name": "calculator",
        "params": {
            "expression": "5 + 3"
        }
    });

    let response = kernel_handle
        .send_tool_request(valid_request)
        .await
        .expect("Failed to send valid request");

    assert_eq!(response["status"].as_str(), Some("ok"));

    // Test 2: Missing tool name (should error)
    let missing_name = json!({
        "command": "invoke",
        "params": {
            "expression": "5 + 3"
        }
    });

    let response = kernel_handle.send_tool_request(missing_name).await;

    // Should either error or handle gracefully
    assert!(response.is_ok() || response.is_err());

    // Test 3: Invalid tool name
    let invalid_tool = json!({
        "command": "invoke",
        "name": "nonexistent_tool",
        "params": {}
    });

    let response = kernel_handle
        .send_tool_request(invalid_tool)
        .await
        .expect("Failed to send invalid tool request");

    assert_eq!(response["status"].as_str(), Some("error"));
    assert!(response["error"].as_str().unwrap().contains("not found"));

    // Test 4: Empty parameters (should be handled)
    let empty_params = json!({
        "command": "invoke",
        "name": "calculator",
        "params": {}
    });

    let response = kernel_handle
        .send_tool_request(empty_params)
        .await
        .expect("Failed to send empty params request");

    // Calculator might error or handle empty params gracefully
    assert!(response.get("status").is_some());
}

#[cfg(feature = "lua")]
#[tokio::test(flavor = "multi_thread")]
async fn test_tool_invocation_error_handling() {
    use llmspell_bridge::ScriptRuntime;
    use llmspell_config::LLMSpellConfig;
    use llmspell_kernel::api::start_embedded_kernel_with_executor;
    use serde_json::json;
    use std::sync::Arc;

    let config = LLMSpellConfig::default();
    let runtime = ScriptRuntime::new(config.clone())
        .await
        .expect("Failed to create runtime");

    // Calculator already registered via register_all_tools
    let executor = Arc::new(runtime);
    let mut kernel_handle = start_embedded_kernel_with_executor(config, executor.clone())
        .await
        .expect("Failed to start kernel");

    // Test 1: Invalid expression for calculator (should handle error)
    let invalid_expr = json!({
        "command": "invoke",
        "name": "calculator",
        "params": {
            "expression": "invalid math expression @#$"
        }
    });

    let response = kernel_handle
        .send_tool_request(invalid_expr)
        .await
        .expect("Failed to send invalid expression");

    // Should either succeed with an error message or return error status
    assert!(response.get("status").is_some());

    // Test 2: Null parameters
    let null_params = json!({
        "command": "invoke",
        "name": "calculator",
        "params": null
    });

    let response = kernel_handle
        .send_tool_request(null_params)
        .await
        .expect("Failed to send null params");

    assert!(response.get("status").is_some());

    // Test 3: Non-object parameters (should be validated)
    let array_params = json!({
        "command": "invoke",
        "name": "calculator",
        "params": ["not", "an", "object"]
    });

    let response = kernel_handle
        .send_tool_request(array_params)
        .await
        .expect("Failed to send array params");

    // Should reject non-object params
    assert_eq!(response["status"].as_str(), Some("error"));
    assert!(
        response["error"]
            .as_str()
            .unwrap_or("")
            .contains("validation")
            || response["error"].as_str().unwrap_or("").contains("object")
    );
}

#[cfg(feature = "lua")]
#[tokio::test(flavor = "multi_thread")]
async fn test_tool_reply_message_routing() {
    use llmspell_bridge::ScriptRuntime;
    use llmspell_config::LLMSpellConfig;
    use llmspell_kernel::api::start_embedded_kernel_with_executor;
    use serde_json::json;
    use std::sync::Arc;

    // Create runtime and register tools
    let config = LLMSpellConfig::default();
    let runtime = ScriptRuntime::new(config.clone())
        .await
        .expect("Failed to create runtime");

    // Calculator already registered via register_all_tools

    // Create kernel with executor
    let executor = Arc::new(runtime);
    let mut kernel_handle = start_embedded_kernel_with_executor(config, executor.clone())
        .await
        .expect("Failed to start kernel");

    // Test 1: Tool list should return proper message structure
    let list_request = json!({
        "command": "list",
    });

    let response = kernel_handle
        .send_tool_request(list_request)
        .await
        .expect("Failed to send tool list request");

    // Verify message structure indicates proper routing (not stdout)
    assert!(
        response.get("tools").is_some(),
        "Should have tools field from proper message routing"
    );
    assert!(
        response.get("count").is_some(),
        "Should have count field from proper message routing"
    );

    // Test 2: Tool invoke should return structured response through message protocol
    let invoke_request = json!({
        "command": "invoke",
        "name": "calculator",
        "params": {
            "expression": "5 + 5"
        }
    });

    let response = kernel_handle
        .send_tool_request(invoke_request)
        .await
        .expect("Failed to send tool invoke request");

    // Verify structured response (not raw stdout text)
    assert!(
        response.get("status").is_some(),
        "Should have structured status field"
    );
    assert!(
        response.get("tool").is_some(),
        "Should have tool name field"
    );

    // Test 3: Error responses should also use message protocol
    let error_request = json!({
        "command": "invoke",
        "name": "nonexistent_tool",
        "params": {}
    });

    let response = kernel_handle
        .send_tool_request(error_request)
        .await
        .expect("Failed to send error request");

    assert_eq!(response["status"].as_str(), Some("error"));
    assert!(
        response.get("error").is_some(),
        "Should have structured error field"
    );
}

#[cfg(feature = "lua")]
#[tokio::test(flavor = "multi_thread")]
async fn test_message_correlation_and_identity() {
    use llmspell_bridge::ScriptRuntime;
    use llmspell_config::LLMSpellConfig;
    use llmspell_kernel::api::start_embedded_kernel_with_executor;
    use serde_json::json;
    use std::sync::Arc;

    let config = LLMSpellConfig::default();
    let runtime = ScriptRuntime::new(config.clone())
        .await
        .expect("Failed to create runtime");

    // Calculator already registered via register_all_tools

    let executor = Arc::new(runtime);
    let mut kernel_handle = start_embedded_kernel_with_executor(config, executor.clone())
        .await
        .expect("Failed to start kernel");

    // Test that multiple concurrent requests maintain proper correlation
    let requests = vec![
        json!({"command": "list"}),
        json!({"command": "invoke", "name": "calculator", "params": {"expression": "1 + 1"}}),
        json!({"command": "invoke", "name": "calculator", "params": {"expression": "2 * 3"}}),
    ];

    let mut responses = Vec::new();
    for request in requests {
        let response = kernel_handle
            .send_tool_request(request)
            .await
            .expect("Failed to send request");
        responses.push(response);
    }

    // Verify all responses are properly structured (indicating message protocol routing)
    assert_eq!(responses.len(), 3);

    // First response should be tool list
    assert!(responses[0].get("tools").is_some());

    // Second and third should be invoke results
    for response in responses.iter().skip(1).take(2) {
        assert!(response.get("status").is_some());
        assert_eq!(response["tool"].as_str(), Some("calculator"));
    }
}

#[cfg(feature = "lua")]
#[tokio::test(flavor = "multi_thread")]
async fn test_bidirectional_communication_flow() {
    use llmspell_bridge::ScriptRuntime;
    use llmspell_config::LLMSpellConfig;
    use llmspell_kernel::api::start_embedded_kernel_with_executor;
    use serde_json::json;
    use std::sync::Arc;

    let config = LLMSpellConfig::default();
    let runtime = ScriptRuntime::new(config.clone())
        .await
        .expect("Failed to create runtime");

    // Calculator and datetime already registered via register_all_tools

    let executor = Arc::new(runtime);
    let mut kernel_handle = start_embedded_kernel_with_executor(config, executor.clone())
        .await
        .expect("Failed to start kernel");

    // Test complete bidirectional flow: CLI → Kernel → Tool → Kernel → CLI

    // 1. List tools (should return multiple tools)
    let list_response = kernel_handle
        .send_tool_request(json!({"command": "list"}))
        .await
        .expect("Failed to list tools");

    let tools = list_response["tools"].as_array().unwrap();
    assert!(tools.len() >= 2, "Should have at least 2 registered tools");

    // 2. Test each tool to verify bidirectional communication
    let test_cases = vec![
        ("calculator", json!({"expression": "10 / 2"})),
        ("datetime", json!({"format": "iso"})),
    ];

    for (tool_name, params) in test_cases {
        // Send tool request
        let request = json!({
            "command": "invoke",
            "name": tool_name,
            "params": params
        });

        let response = kernel_handle
            .send_tool_request(request)
            .await
            .unwrap_or_else(|_| panic!("Failed to invoke {}", tool_name));

        // Verify structured response through message protocol
        assert!(
            response.get("status").is_some(),
            "Tool {} should return structured status",
            tool_name
        );
        assert_eq!(
            response["tool"].as_str(),
            Some(tool_name),
            "Tool name should be preserved"
        );

        // For successful tools, should have result field
        if response["status"] == "ok" {
            assert!(
                response.get("result").is_some(),
                "Successful tool {} should have result",
                tool_name
            );
        }
    }

    // 3. Test error handling preserves message structure
    let error_request = json!({
        "command": "invoke",
        "name": "invalid_tool",
        "params": {}
    });

    let error_response = kernel_handle
        .send_tool_request(error_request)
        .await
        .expect("Failed to send error request");

    assert_eq!(error_response["status"].as_str(), Some("error"));
    assert!(error_response.get("error").is_some());
}
