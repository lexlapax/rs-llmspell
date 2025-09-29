//! Test that ScriptExecutor provides access to ComponentRegistry

#[cfg(feature = "lua")]
#[tokio::test]
async fn test_script_executor_component_registry_access() {
    use llmspell_bridge::ScriptRuntime;
    use llmspell_config::LLMSpellConfig;
    use llmspell_core::traits::script_executor::ScriptExecutor;
    use llmspell_tools::util::CalculatorTool;
    use std::sync::Arc;

    // Create a runtime with Lua engine
    let config = LLMSpellConfig::default();
    let runtime = ScriptRuntime::new_with_lua(config)
        .await
        .expect("Failed to create runtime");

    // Get the component registry via ScriptExecutor trait
    let registry = runtime
        .component_registry()
        .expect("ScriptRuntime should provide component_registry");

    // Register a test tool
    let calc_tool = Arc::new(CalculatorTool::new());

    // Get the actual ComponentRegistry to register tools
    // (In real usage, tools would be pre-registered)
    let runtime_registry = runtime.registry();
    runtime_registry
        .register_tool("calculator".to_string(), calc_tool.clone())
        .expect("Failed to register calculator tool");

    // Now test that we can access it via ComponentLookup trait
    let tools = registry.list_tools().await;
    assert!(tools.contains(&"calculator".to_string()), "Should have calculator tool");

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
#[tokio::test]
async fn test_kernel_can_access_registry() {
    use llmspell_bridge::ScriptRuntime;
    use llmspell_config::LLMSpellConfig;
    use llmspell_kernel::api::start_embedded_kernel_with_executor;
    use std::sync::Arc;

    // Create a runtime with component registry
    let config = LLMSpellConfig::default();
    let runtime = ScriptRuntime::new_with_lua(config.clone())
        .await
        .expect("Failed to create runtime");

    // Register some tools in the runtime's registry
    use llmspell_tools::util::{CalculatorTool, DateTimeHandlerTool, UuidGeneratorTool};

    let registry = runtime.registry();
    registry
        .register_tool("calculator".to_string(), Arc::new(CalculatorTool::new()))
        .expect("Failed to register calculator");
    registry
        .register_tool("datetime".to_string(), Arc::new(DateTimeHandlerTool::new()))
        .expect("Failed to register datetime handler");
    registry
        .register_tool("uuid_generator".to_string(), Arc::new(UuidGeneratorTool::default()))
        .expect("Failed to register uuid generator");

    // Create kernel with the runtime as executor
    let executor = Arc::new(runtime);
    let mut kernel_handle = start_embedded_kernel_with_executor(config, executor.clone())
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
    assert!(tools.len() >= 3, "Should have at least 3 tools, got {}", tools.len());

    // Check for specific tools
    let tool_names: Vec<String> = tools
        .iter()
        .filter_map(|v| v.as_str().map(String::from))
        .collect();

    assert!(tool_names.contains(&"calculator".to_string()), "Should have calculator");
    assert!(tool_names.contains(&"datetime".to_string()), "Should have datetime");
    assert!(tool_names.contains(&"uuid_generator".to_string()), "Should have uuid_generator");
}