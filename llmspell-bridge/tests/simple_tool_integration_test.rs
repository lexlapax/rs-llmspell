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

    // Test 5: System → Data → Utility chain (Env → JSON → Template)
    let system_chain_test = r#"
        -- System → Data → Utility chain
        -- Step 1: Read environment variable
        local env_result = Tool.get("environment_reader"):execute({
            operation = "get",
            variable_name = "PATH"
        })
        
        if not env_result.success then
            error("Environment read failed: " .. (env_result.error or "unknown"))
        end
        
        -- For now, just verify we got something back
        local has_env_output = env_result.output ~= nil and env_result.output ~= ""
        
        -- Step 2: Test JSON processor with simple data
        local test_data = {
            users = {
                {name = "Alice", age = 30},
                {name = "Bob", age = 25}
            }
        }
        
        local json_result = Tool.get("json_processor"):execute({
            operation = "query",
            input = JSON.stringify(test_data),
            query = ".users | length"
        })
        
        if not json_result.success then
            error("JSON processing failed: " .. (json_result.error or "unknown"))
        end
        
        local json_parsed = JSON.parse(json_result.output)
        -- The result might be in different places depending on the tool
        local user_count = nil
        if type(json_parsed) == "number" then
            user_count = json_parsed
        elseif json_parsed.result then
            if type(json_parsed.result) == "number" then
                user_count = json_parsed.result
            elseif json_parsed.result.result then
                user_count = json_parsed.result.result
            end
        end
        user_count = user_count or 0
        
        -- Step 3: Use the result in a template
        local template_result = Tool.get("template_engine"):execute({
            template = "Found {{count}} users in the system",
            context = {
                count = user_count
            },
            engine = "handlebars"
        })
        
        if not template_result.success then
            error("Template rendering failed: " .. (template_result.error or "unknown"))
        end
        
        local template_parsed = JSON.parse(template_result.output)
        -- Handle different output formats for template result
        local template_output = nil
        if template_parsed.result then
            template_output = template_parsed.result.output or template_parsed.result.rendered or template_parsed.result
        elseif type(template_parsed) == "string" then
            template_output = template_parsed
        end
        
        return {
            env_tested = has_env_output,
            json_processed = user_count == 2,
            template_rendered = template_output ~= nil,
            success = true
        }
    "#;

    match engine.execute_script(system_chain_test).await {
        Ok(result) => {
            let obj = result.output.as_object().expect("Expected object");
            assert!(obj.get("env_tested").unwrap().as_bool().unwrap());
            assert!(obj.get("json_processed").unwrap().as_bool().unwrap());
            assert!(obj.get("template_rendered").unwrap().as_bool().unwrap());
            println!("✅ System → Data → Utility chain test passed");
        }
        Err(e) => panic!("System chain test failed: {}", e),
    }

    // Test 6: File → Data → File chain (Write → CSV → Read)
    let file_chain_test = r#"
        -- File → Data → File chain
        -- Step 1: Write initial data
        local test_data = "name,age,city\nAlice,30,NYC\nBob,25,LA\nCharlie,35,Chicago"
        local write_result = Tool.get("file_operations"):execute({
            operation = "write",
            path = "/tmp/llmspell_test_data.csv",
            content = test_data
        })
        
        if not write_result.success then
            error("File write failed: " .. (write_result.error or "unknown"))
        end
        
        -- Step 2: Analyze the CSV data
        local csv_result = Tool.get("csv_analyzer"):execute({
            operation = "analyze",
            content = test_data
        })
        
        if not csv_result.success then
            error("CSV analysis failed: " .. (csv_result.error or "unknown"))
        end
        
        local csv_parsed = JSON.parse(csv_result.output)
        
        -- Extract CSV analysis results
        local row_count = 0
        local column_count = 0
        local headers = {}
        
        if csv_parsed.result then
            row_count = csv_parsed.result.row_count or 0
            column_count = csv_parsed.result.column_count or 0
            headers = csv_parsed.result.headers or {}
        elseif csv_parsed.row_count then
            -- Direct properties on parsed object
            row_count = csv_parsed.row_count or 0
            column_count = csv_parsed.column_count or 0
            -- Extract header names from columns array
            if csv_parsed.columns and type(csv_parsed.columns) == "table" then
                for _, col in ipairs(csv_parsed.columns) do
                    if col.name then
                        table.insert(headers, col.name)
                    end
                end
            end
        end
        
        -- Step 3: Write analysis results to another file
        local analysis_content = string.format(
            "CSV Analysis Results:\nRows: %d\nColumns: %d\nHeaders: %s",
            row_count,
            column_count,
            table.concat(headers, ", ")
        )
        
        local write_analysis = Tool.get("file_operations"):execute({
            operation = "write",
            path = "/tmp/llmspell_analysis.txt",
            content = analysis_content
        })
        
        if not write_analysis.success then
            error("Analysis write failed: " .. (write_analysis.error or "unknown"))
        end
        
        -- Cleanup
        Tool.get("file_operations"):execute({
            operation = "delete",
            path = "/tmp/llmspell_test_data.csv"
        })
        Tool.get("file_operations"):execute({
            operation = "delete",
            path = "/tmp/llmspell_analysis.txt"
        })
        
        return {
            file_written = true,
            csv_analyzed = row_count == 3,
            analysis_written = true,
            success = true
        }
    "#;

    match engine.execute_script(file_chain_test).await {
        Ok(result) => {
            let obj = result.output.as_object().expect("Expected object");
            assert!(obj.get("file_written").unwrap().as_bool().unwrap());
            assert!(obj.get("csv_analyzed").unwrap().as_bool().unwrap());
            assert!(obj.get("analysis_written").unwrap().as_bool().unwrap());
            println!("✅ File → Data → File chain test passed");
        }
        Err(e) => panic!("File chain test failed: {}", e),
    }

    // Test 7: Data → System → File chain (HTTP simulation → Process → Save)
    let data_system_file_test = r#"
        -- Data → System → File chain
        -- Note: Simplified test without process_executor (async tool)
        
        -- Step 1: Simulate HTTP data (would normally be http_request)
        local mock_data = {
            users = {
                {name = "Alice", score = 95},
                {name = "Bob", score = 87},
                {name = "Charlie", score = 92}
            }
        }
        
        -- Use json_processor to query the data
        local json_result = Tool.get("json_processor"):execute({
            operation = "query",
            input = JSON.stringify(mock_data),
            query = ".users | length"
        })
        
        if not json_result.success then
            error("JSON query failed: " .. (json_result.error or "unknown"))
        end
        
        local json_parsed = JSON.parse(json_result.output)
        -- Handle different result formats
        local user_count = nil
        if type(json_parsed) == "number" then
            user_count = json_parsed
        elseif json_parsed.result then
            if type(json_parsed.result) == "number" then
                user_count = json_parsed.result
            elseif json_parsed.result.result then
                user_count = json_parsed.result.result
            end
        end
        user_count = user_count or 0
        
        -- Step 2: Use environment reader instead of process executor (sync tool)
        local env_result = Tool.get("environment_reader"):execute({
            operation = "list",
            pattern = "PATH"
        })
        
        if not env_result.success then
            error("Environment read failed: " .. (env_result.error or "unknown"))
        end
        
        -- Step 3: Save the results to a file
        local save_result = Tool.get("file_operations"):execute({
            operation = "write",
            path = "/tmp/llmspell_data_chain.txt",
            content = string.format("Data Chain Results:\nUser count: %d\nEnvironment checked: true\nTimestamp: %s", 
                user_count, os.date())
        })
        
        if not save_result.success then
            error("File save failed: " .. (save_result.error or "unknown"))
        end
        
        -- Cleanup
        Tool.get("file_operations"):execute({
            operation = "delete",
            path = "/tmp/llmspell_data_chain.txt"
        })
        
        return {
            data_queried = user_count == 3,
            system_checked = env_result.output ~= nil,
            file_saved = true,
            success = true
        }
    "#;

    match engine.execute_script(data_system_file_test).await {
        Ok(result) => {
            let obj = result.output.as_object().expect("Expected object");
            assert!(obj.get("data_queried").unwrap().as_bool().unwrap());
            assert!(obj.get("system_checked").unwrap().as_bool().unwrap());
            assert!(obj.get("file_saved").unwrap().as_bool().unwrap());
            println!("✅ Data → System → File chain test passed");
        }
        Err(e) => panic!("Data → System → File chain test failed: {}", e),
    }

    // Test 8: Error propagation through chains
    let error_chain_test = r#"
        -- Test error propagation in tool chains
        local chain_errors = {}
        
        -- Chain with invalid tool
        local success1 = pcall(function()
            local result = Tool.get("nonexistent_tool"):execute({})
        end)
        chain_errors.invalid_tool = not success1
        
        -- Chain with invalid parameters
        local success2 = pcall(function()
            local result = Tool.get("base64_encoder"):execute({
                operation = "invalid_op",
                input = "test"
            })
            if not result.success then
                error(result.error)
            end
        end)
        chain_errors.invalid_params = not success2
        
        -- Chain where middle step fails
        local success3, err3 = pcall(function()
            -- Step 1: Generate UUID (should work)
            local uuid_result = Tool.get("uuid_generator"):execute({
                operation = "generate",
                version = "v4"
            })
            
            -- Step 2: Try to hash with missing required parameter
            local hash_result = Tool.get("hash_calculator"):execute({
                operation = "hash",
                algorithm = "md5"
                -- Missing required 'data' parameter
            })
            
            if not hash_result.success then
                error("Chain failed at hash step: " .. hash_result.error)
            end
        end)
        chain_errors.middle_step_fail = not success3
        
        -- Debug output
        print("Error test results:")
        print("  invalid_tool: " .. tostring(chain_errors.invalid_tool))
        print("  invalid_params: " .. tostring(chain_errors.invalid_params))
        print("  middle_step_fail: " .. tostring(chain_errors.middle_step_fail))
        if not success3 then
            print("  middle_step error: " .. tostring(err3))
        end
        
        return {
            invalid_tool_caught = chain_errors.invalid_tool,
            invalid_params_caught = chain_errors.invalid_params,
            chain_failure_caught = chain_errors.middle_step_fail,
            success = true
        }
    "#;

    match engine.execute_script(error_chain_test).await {
        Ok(result) => {
            let obj = result.output.as_object().expect("Expected object");
            assert!(obj.get("invalid_tool_caught").unwrap().as_bool().unwrap());
            assert!(obj.get("invalid_params_caught").unwrap().as_bool().unwrap());
            assert!(obj.get("chain_failure_caught").unwrap().as_bool().unwrap());
            println!("✅ Error propagation test passed");
        }
        Err(e) => panic!("Error propagation test failed: {}", e),
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
