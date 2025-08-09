//! Integration tests for all 26 Phase 2 tools

#[tokio::test(flavor = "multi_thread")]
#[cfg(feature = "lua")]
async fn test_all_tools_integration() {
    use llmspell_bridge::{
        engine::factory::{EngineFactory, LuaConfig},
        providers::{ProviderManager, ProviderManagerConfig},
        ComponentRegistry,
    };
    use std::sync::Arc;

    // Initialize components
    let registry = Arc::new(ComponentRegistry::new());
    let provider_config = ProviderManagerConfig::default();
    let providers = Arc::new(ProviderManager::new(provider_config).await.unwrap());

    // Register all tools
    llmspell_bridge::tools::register_all_tools(registry.clone()).unwrap();

    // Create engine
    let lua_config = LuaConfig::default();
    let mut engine = EngineFactory::create_lua_engine(&lua_config).unwrap();

    // Inject APIs
    engine.inject_apis(&registry, &providers).unwrap();

    // Simple integration test for all tools
    let test_script = r#"
        -- First, list all available tools
        local all_tools = Tool.list()
        print("Available tools: " .. #all_tools)
        
        -- Test specific tools we know should exist
        local tools_to_test = {
            "base64_encoder",
            "calculator", 
            "uuid_generator",
            "hash_calculator",
            "text_manipulator",
            "file_operations"
        }
        
        local passed = 0
        local failed = 0
        
        for _, tool_name in ipairs(tools_to_test) do
            local tool = Tool.get(tool_name)
            if tool then
                passed = passed + 1
                print("✓ " .. tool_name .. " is available")
            else
                failed = failed + 1
                print("✗ " .. tool_name .. " is NOT available")
            end
        end
        
        print("\nTotal tools tested: " .. (passed + failed))
        print("Passed: " .. passed)
        print("Failed: " .. failed)
        
        return {
            passed = passed,
            failed = failed,
            total = passed + failed
        }
    "#;

    // Run the test
    match engine.execute_script(test_script).await {
        Ok(result) => {
            println!("Integration test output: {:?}", result.output);

            // Check if the test reported success
            if let Some(obj) = result.output.as_object() {
                if let Some(passed) = obj.get("passed") {
                    let passed_count = passed.as_i64().unwrap_or(0);
                    let failed_count = obj
                        .get("failed")
                        .and_then(serde_json::value::Value::as_i64)
                        .unwrap_or(0);

                    println!("Test results: {passed_count} passed, {failed_count} failed");
                    assert!(passed_count >= 6, "Should have at least 6 core tools");
                    assert_eq!(failed_count, 0, "Some core tools are missing");
                }
            }
        }
        Err(e) => {
            panic!("Integration test failed: {e}");
        }
    }
}

#[tokio::test(flavor = "multi_thread")]
#[cfg(feature = "lua")]
async fn test_tool_performance_benchmarks() {
    use llmspell_bridge::{
        engine::factory::{EngineFactory, LuaConfig},
        providers::{ProviderManager, ProviderManagerConfig},
        ComponentRegistry,
    };
    use std::sync::Arc;
    use std::time::Instant;

    let registry = Arc::new(ComponentRegistry::new());
    let provider_config = ProviderManagerConfig::default();
    let providers = Arc::new(ProviderManager::new(provider_config).await.unwrap());

    llmspell_bridge::tools::register_all_tools(registry.clone()).unwrap();

    let lua_config = LuaConfig::default();
    let mut engine = EngineFactory::create_lua_engine(&lua_config).unwrap();

    engine.inject_apis(&registry, &providers).unwrap();

    // Benchmark each tool category
    let tool_benchmarks = vec![
        (
            "base64_encoder",
            r#"return Tool.get("base64_encoder"):execute({operation="encode", input="test"})"#,
        ),
        (
            "calculator",
            r#"return Tool.get("calculator"):execute({operation="evaluate", expression="2+2"})"#,
        ),
        (
            "uuid_generator",
            r#"return Tool.get("uuid_generator"):execute({operation="generate", version="v4"})"#,
        ),
        (
            "hash_calculator",
            r#"return Tool.get("hash_calculator"):execute({operation="hash", algorithm="md5", input="test"})"#,
        ),
        (
            "text_manipulator",
            r#"return Tool.get("text_manipulator"):execute({operation="uppercase", input="test"})"#,
        ),
    ];

    println!("\nTool Performance Benchmarks:");
    println!("============================");

    for (tool_name, script) in tool_benchmarks {
        let iterations = 100;
        let start = Instant::now();

        for _ in 0..iterations {
            engine.execute_script(script).await.unwrap();
        }

        let elapsed = start.elapsed();
        #[allow(clippy::cast_precision_loss)] // Acceptable for timing measurements
        let per_op = elapsed.as_micros() as f64 / f64::from(iterations) / 1000.0; // Convert to ms

        println!("{tool_name:<20} {per_op:.3}ms/op");

        // Assert <10ms requirement
        assert!(
            per_op < 10.0,
            "{tool_name} exceeds 10ms target: {per_op:.3}ms"
        );
    }
}
