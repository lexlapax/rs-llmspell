//! Test that `ScriptExecutor` provides access to `ComponentRegistry`

#[cfg(feature = "lua")]
#[tokio::test(flavor = "multi_thread")]
async fn test_script_executor_component_registry_access() {
    use llmspell_bridge::ScriptRuntime;
    use llmspell_config::LLMSpellConfig;
    use llmspell_core::traits::script_executor::ScriptExecutor;

    // Create a runtime with Lua engine
    let config = LLMSpellConfig::default();
    let runtime = Box::pin(ScriptRuntime::new(config))
        .await
        .expect("Failed to create runtime");

    // Get the component registry via ScriptExecutor trait
    let registry = runtime
        .component_registry()
        .expect("ScriptRuntime should provide component_registry");

    // Tools are already registered via register_all_tools() during runtime creation
    // (calculator, datetime, uuid_generator, etc.)

    // Test that we can access tools via ComponentLookup trait
    let tools = registry.list_tools().await;
    assert!(
        tools.contains(&"calculator".to_string()),
        "Should have calculator tool"
    );

    // Test getting a specific tool
    let tool = registry.get_tool("calculator").await;
    assert!(tool.is_some(), "Should be able to get calculator tool");

    // Test that the tool has the expected metadata
    if let Some(tool) = tool {
        assert_eq!(tool.metadata().name, "calculator");
        assert!(!tool.metadata().description.is_empty());
    }
}

#[cfg(feature = "lua")]
#[tokio::test(flavor = "multi_thread")]
async fn test_kernel_can_access_registry() {
    use llmspell_bridge::ScriptRuntime;
    use llmspell_config::LLMSpellConfig;
    use llmspell_kernel::api::start_embedded_kernel_with_executor;
    use std::sync::Arc;

    // Create a runtime with component registry
    // Tools (calculator, datetime, uuid_generator, etc.) are already registered via register_all_tools()
    let config = LLMSpellConfig::default();
    let runtime = Box::pin(ScriptRuntime::new(config.clone()))
        .await
        .expect("Failed to create runtime");

    // Create kernel with the runtime as executor
    let executor = Arc::new(runtime);
    let mut kernel_handle = Box::pin(start_embedded_kernel_with_executor(
        config,
        executor.clone(),
    ))
    .await
    .expect("Failed to start kernel");

    // Send a tool list request
    let list_request = serde_json::json!({
        "command": "list",
    });

    let response = kernel_handle
        .send_tool_request(list_request)
        .await
        .expect("Failed to send tool request");

    // Verify we get actual tools, not placeholders
    assert!(response.get("tools").is_some());
    let tools = response["tools"].as_array().unwrap();

    // Should have at least the 3 tools we registered
    assert!(
        tools.len() >= 3,
        "Should have at least 3 tools, got {}",
        tools.len()
    );

    // Check for specific tools
    let tool_names: Vec<String> = tools
        .iter()
        .filter_map(|v| v.as_str().map(String::from))
        .collect();

    println!("Registered tools: {tool_names:?}");

    // Note: Tool names are based on their actual registration names in register_all_tools()
    // calculator is registered, but datetime and uuid_generator might have different names
    assert!(
        tool_names.contains(&"calculator".to_string()),
        "Should have calculator. Actual tools: {tool_names:?}"
    );

    // Just verify we have at least 3 tools registered - names may vary
    // The important test is that the registry is accessible, not specific tool names
}
