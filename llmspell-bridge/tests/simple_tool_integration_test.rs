//! Simple integration test for Phase 2 tools

#[tokio::test]
#[cfg(feature = "lua")]
async fn test_simple_tool_integration() {
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

    // Test 1: Basic tool execution with JSON parsing
    let test_script = r#"
        -- Test base64 encoder
        local tool = Tool.get("base64_encoder")
        if not tool then
            error("Could not get base64_encoder tool")
        end
        
        -- Debug: print what we're sending
        local params = {
            operation = "encode",
            input = "Hello, World!"
        }
        
        -- Debug: check if params is a valid table
        if type(params) ~= "table" then
            error("params is not a table, it's a " .. type(params))
        end
        
        local result = tool:execute(params)
        
        if not result then
            error("Tool execution returned nil")
        end
        
        if not result.success then
            error("Tool execution failed: " .. (result.error or "unknown error"))
        end
        
        -- Parse the JSON output
        local parsed = JSON.parse(result.output)
        if not parsed or not parsed.result then
            error("Failed to parse tool output: " .. tostring(result.output))
        end
        
        return {
            encoded = parsed.result.output,
            success = true
        }
    "#;

    match engine.execute_script(test_script).await {
        Ok(result) => {
            let obj = result.output.as_object().expect("Expected object result");
            assert_eq!(obj.get("success").unwrap().as_bool().unwrap(), true);
            assert_eq!(
                obj.get("encoded").unwrap().as_str().unwrap(),
                "SGVsbG8sIFdvcmxkIQ=="
            );
            println!("✅ Base64 encoder test passed");
        }
        Err(e) => panic!("Base64 test failed: {}", e),
    }

    // Test 2: Calculator tool
    let calc_test = r#"
        local result = Tool.get("calculator"):execute({
            operation = "evaluate",
            expression = "2 + 3 * 4"
        })
        
        if not result.success then
            error("Calculator failed: " .. (result.error or "unknown"))
        end
        
        local parsed = JSON.parse(result.output)
        return {
            result = parsed.result.result,
            success = true
        }
    "#;

    match engine.execute_script(calc_test).await {
        Ok(result) => {
            let obj = result.output.as_object().expect("Expected object");
            assert_eq!(obj.get("result").unwrap().as_f64().unwrap(), 14.0);
            println!("✅ Calculator test passed");
        }
        Err(e) => panic!("Calculator test failed: {}", e),
    }

    // Test 3: Tool chaining
    let chain_test = r#"
        -- Generate UUID
        local uuid_result = Tool.get("uuid_generator"):execute({
            operation = "generate",
            version = "v4"
        })
        
        if not uuid_result.success then
            error("UUID generation failed")
        end
        
        local uuid_parsed = JSON.parse(uuid_result.output)
        local uuid = uuid_parsed.result.uuid
        
        -- Hash the UUID
        local hash_result = Tool.get("hash_calculator"):execute({
            operation = "hash",
            algorithm = "md5",
            data = uuid
        })
        
        if not hash_result.success then
            error("Hash calculation failed")
        end
        
        local hash_parsed = JSON.parse(hash_result.output)
        
        return {
            uuid = uuid,
            hash = hash_parsed.result.hash,
            success = true
        }
    "#;

    match engine.execute_script(chain_test).await {
        Ok(result) => {
            let obj = result.output.as_object().expect("Expected object");
            assert!(obj.get("uuid").unwrap().as_str().unwrap().len() == 36);
            assert!(obj.get("hash").unwrap().as_str().unwrap().len() == 32);
            println!("✅ Tool chaining test passed");
        }
        Err(e) => panic!("Tool chaining test failed: {}", e),
    }

    // Test 4: List all available tools
    let list_tools_test = r#"
        local tools = Tool.list()
        local count = 0
        local tool_names = {}
        
        for _, tool in ipairs(tools) do
            count = count + 1
            table.insert(tool_names, tool)
        end
        
        return {
            count = count,
            tools = tool_names,
            has_calculator = false,
            has_base64 = false
        }
    "#;

    match engine.execute_script(list_tools_test).await {
        Ok(result) => {
            let obj = result.output.as_object().expect("Expected object");
            let count = obj.get("count").unwrap().as_i64().unwrap();
            println!("✅ Found {} tools registered", count);
            assert!(count >= 25, "Expected at least 25 tools, found {}", count);
        }
        Err(e) => panic!("Tool listing test failed: {}", e),
    }

    println!("\n✅ All integration tests passed!");
}

#[tokio::test]
#[cfg(feature = "lua")]
async fn test_tool_performance() {
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

    // Warm up
    let warmup_script = r#"
        Tool.get("uuid_generator"):execute({operation = "generate", version = "v4"})
        return true
    "#;
    let _ = engine.execute_script(warmup_script).await;

    // Benchmark different tools
    let benchmarks = vec![
        (
            "uuid_generator",
            r#"Tool.get("uuid_generator"):execute({operation = "generate", version = "v4"})"#,
        ),
        (
            "base64_encoder",
            r#"Tool.get("base64_encoder"):execute({operation = "encode", input = "test"})"#,
        ),
        (
            "hash_calculator",
            r#"Tool.get("hash_calculator"):execute({operation = "hash", algorithm = "md5", data = "test"})"#,
        ),
        (
            "calculator",
            r#"Tool.get("calculator"):execute({operation = "evaluate", expression = "1+1"})"#,
        ),
    ];

    println!("\n=== Tool Performance Benchmarks ===");

    for (name, script) in benchmarks {
        let full_script = format!("return {}", script);
        let iterations = 50;

        let start = Instant::now();
        for _ in 0..iterations {
            let _ = engine.execute_script(&full_script).await;
        }
        let elapsed = start.elapsed();

        let per_op_ms = elapsed.as_micros() as f64 / iterations as f64 / 1000.0;

        println!("{:<20} {:>8.3} ms/op", name, per_op_ms);

        // Check <10ms requirement
        if per_op_ms > 10.0 {
            println!("  ⚠️  WARNING: Exceeds 10ms target!");
        }
    }
}
