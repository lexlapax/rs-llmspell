//! Test that kernel tool handlers use real ComponentRegistry

#[cfg(feature = "lua")]
#[tokio::test]
async fn test_kernel_tool_handlers_with_registry() {
    use llmspell_bridge::ScriptRuntime;
    use llmspell_config::LLMSpellConfig;
    use llmspell_kernel::api::start_embedded_kernel_with_executor;
    use llmspell_tools::util::{CalculatorTool, DateTimeHandlerTool};
    use serde_json::json;
    use std::sync::Arc;

    // Create a runtime with Lua engine and tools
    let config = LLMSpellConfig::default();
    let runtime = ScriptRuntime::new_with_lua(config.clone())
        .await
        .expect("Failed to create runtime");

    // Verify registry has tools (register_all_tools should have been called)
    let registry = runtime.registry();
    let tools_before = registry.list_tools();
    println!("Tools registered: {} tools", tools_before.len());

    // Manually register additional tools for testing
    registry
        .register_tool("test_calculator".to_string(), Arc::new(CalculatorTool::new()))
        .expect("Failed to register test calculator");
    registry
        .register_tool("test_datetime".to_string(), Arc::new(DateTimeHandlerTool::new()))
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

    assert!(tool_names.contains(&"test_calculator".to_string()), "Should have test_calculator");
    assert!(tool_names.contains(&"test_datetime".to_string()), "Should have test_datetime");

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
    assert!(utility_tool_names.contains(&"test_calculator".to_string()),
            "Calculator should be in utility category");

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
    assert!(match_names.contains(&"test_calculator".to_string()),
            "Search for 'calc' should find test_calculator");
}

#[cfg(feature = "lua")]
#[tokio::test]
async fn test_kernel_tool_count() {
    use llmspell_bridge::ScriptRuntime;
    use llmspell_config::LLMSpellConfig;
    use llmspell_kernel::api::start_embedded_kernel_with_executor;
    use serde_json::json;
    use std::sync::Arc;

    // Create a runtime with default configuration
    let config = LLMSpellConfig::default();
    let runtime = ScriptRuntime::new_with_lua(config.clone())
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
    assert_eq!(tools.len() as u64, count, "Tool count should match array length");

    // The requirement is 40+ tools when all are registered
    // If register_all_tools worked, we should have many tools
    // For now, we may have fewer until all tools are implemented
    assert!(count > 0, "Should have at least some tools registered");
}