//! Integration tests for all 26 Phase 2 tools

#[tokio::test]
#[cfg(feature = "lua")]
#[ignore = "Requires external test-helpers.lua file"]
async fn test_all_tools_integration() {
    use llmspell_bridge::{
        engine::factory::{EngineFactory, LuaConfig},
        providers::{ProviderManager, ProviderManagerConfig},
        ComponentRegistry,
    };
    use std::path::PathBuf;
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

    // Load and run the integration test script
    let test_script_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .join("tests")
        .join("integration")
        .join("test_all_tools.lua");

    let test_script =
        std::fs::read_to_string(&test_script_path).expect("Failed to read test script");

    // Run the test
    match engine.execute_script(&test_script).await {
        Ok(result) => {
            println!("Integration test output: {:?}", result.output);

            // Check if the test reported success
            if let Some(obj) = result.output.as_object() {
                if let Some(passed) = obj.get("passed") {
                    let passed_count = passed.as_i64().unwrap_or(0);
                    let failed_count = obj.get("failed").and_then(|f| f.as_i64()).unwrap_or(0);

                    println!(
                        "Test results: {} passed, {} failed",
                        passed_count, failed_count
                    );
                    assert_eq!(failed_count, 0, "Some tests failed");
                }
            }
        }
        Err(e) => {
            panic!("Integration test failed: {}", e);
        }
    }
}

#[tokio::test]
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
            r#"return Tool.get("hash_calculator"):execute({operation="hash", algorithm="md5", data="test"})"#,
        ),
        (
            "text_manipulator",
            r#"return Tool.get("text_manipulator"):execute({operation="uppercase", text="test"})"#,
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
        let per_op = elapsed.as_micros() as f64 / iterations as f64 / 1000.0; // Convert to ms

        println!("{:<20} {:.3}ms/op", tool_name, per_op);

        // Assert <10ms requirement
        assert!(
            per_op < 10.0,
            "{} exceeds 10ms target: {:.3}ms",
            tool_name,
            per_op
        );
    }
}
