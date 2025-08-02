//! Simple integration test for Phase 2 tools

#[tokio::test(flavor = "multi_thread")]
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
        
        -- Test if we can call Tool.invoke directly
        -- Tool.invoke should wrap parameters correctly
        print("Testing Tool.invoke...")
        
        -- Create a coroutine to call Tool.invoke (which is async)
        local co = coroutine.create(function()
            return Tool.invoke("base64_encoder", {
                operation = "encode",
                input = "Hello, World!"
            })
        end)
        
        -- Execute the coroutine
        local success, result = coroutine.resume(co)
        
        -- Handle async operations that yield
        while success and coroutine.status(co) ~= "dead" do
            success, result = coroutine.resume(co, result)
        end
        
        if not success then
            error("Tool execution failed: " .. tostring(result))
        end
        
        if not result then
            error("Tool execution returned nil")
        end
        
        if not result then
            error("Tool execution failed: result is nil")
        end
        
        -- Debug: print what we got
        local result_type = type(result)
        if result_type ~= "table" then
            error("Tool execution failed: result is " .. result_type .. ", not a table")
        end
        
        -- Check what fields are in result
        local fields = {}
        for k, v in pairs(result) do
            table.insert(fields, k .. "=" .. type(v))
        end
        
        -- Check if execution failed
        if result.success == false then
            error("Tool execution failed: " .. (result.error or "unknown error"))
        end
        
        -- The result from executeAsync should have text field with JSON
        if not result.text then
            error("Tool execution failed: no text in result. Fields: " .. table.concat(fields, ", "))
        end
        
        -- Parse the JSON output from the text field
        local parsed = JSON.parse(result.text)
        if not parsed then
            error("Failed to parse tool output: " .. tostring(result.text))
        end
        
        -- Tool returns {success: true, result: {output: "...", variant: "standard", binary: false}}
        if not parsed.success then
            error("Tool returned failure: " .. tostring(parsed.message or "unknown"))
        end
        
        return {
            encoded = parsed.result.output,
            success = true
        }
    "#;

    match engine.execute_script(test_script).await {
        Ok(result) => {
            let obj = result.output.as_object().expect("Expected object result");
            assert!(obj.get("success").unwrap().as_bool().unwrap());
            assert_eq!(
                obj.get("encoded").unwrap().as_str().unwrap(),
                "SGVsbG8sIFdvcmxkIQ=="
            );
            println!("✅ Base64 encoder test passed");
        }
        Err(e) => panic!("Base64 test failed: {e}"),
    }

    // Test 2: Calculator tool
    let calc_test = r#"
        -- Use coroutine for async Tool.invoke
        local co = coroutine.create(function()
            return Tool.invoke("calculator", {
                operation = "evaluate",
                input = "2 + 3 * 4"
            })
        end)
        
        local success, result = coroutine.resume(co)
        while success and coroutine.status(co) ~= "dead" do
            success, result = coroutine.resume(co, result)
        end
        
        if not success then
            error("Calculator execution failed: " .. tostring(result))
        end
        
        if not result then
            error("Calculator failed: result is nil")
        end
        
        if not result.text then
            error("Calculator failed: no text in result")
        end
        
        local parsed = JSON.parse(result.text)
        if not parsed.success then
            error("Calculator tool failed: " .. tostring(parsed.message or "unknown"))
        end
        
        return {
            result = parsed.result.result, -- The actual numeric result is nested
            success = true
        }
    "#;

    match engine.execute_script(calc_test).await {
        Ok(result) => {
            let obj = result.output.as_object().expect("Expected object");
            assert_eq!(obj.get("result").unwrap().as_f64().unwrap(), 14.0);
            println!("✅ Calculator test passed");
        }
        Err(e) => panic!("Calculator test failed: {e}"),
    }

    // Test 3: Tool chaining
    let chain_test = r#"
        -- Debug: Check what tools are available
        local tools = Tool.list()
        print("Available tools:")
        for i, tool in ipairs(tools) do
            print("  " .. i .. ": " .. tool.name)
        end
        
        -- Generate UUID (using Tool instance methods)
        local uuid_tool = Tool.get("uuid_generator")
        if not uuid_tool then
            error("Could not get uuid_generator tool")
        end
        
        -- Create coroutine for async execution
        local co = coroutine.create(function()
            return uuid_tool:execute({
                operation = "generate",
                version = "v4"
            })
        end)
        
        local success, uuid_result = coroutine.resume(co)
        while success and coroutine.status(co) ~= "dead" do
            success, uuid_result = coroutine.resume(co, uuid_result)
        end
        
        if not success then
            error("UUID generation failed: " .. tostring(uuid_result))
        end
        
        if not uuid_result.text then
            error("UUID generation failed: no text in result")
        end
        
        local uuid_parsed = JSON.parse(uuid_result.text)
        if not uuid_parsed.success then
            error("UUID generation failed: " .. tostring(uuid_parsed.message or "unknown"))
        end
        
        local uuid = uuid_parsed.result.uuid
        
        -- Hash the UUID (using Tool instance methods)
        local hash_tool = Tool.get("hash_calculator")
        if not hash_tool then
            error("Could not get hash_calculator tool")
        end
        
        -- Create coroutine for async execution
        local co2 = coroutine.create(function()
            return hash_tool:execute({
                operation = "hash",
                algorithm = "md5",
                input = uuid
            })
        end)
        
        local success2, hash_result = coroutine.resume(co2)
        while success2 and coroutine.status(co2) ~= "dead" do
            success2, hash_result = coroutine.resume(co2, hash_result)
        end
        
        if not success2 then
            error("Hash calculation failed: " .. tostring(hash_result))
        end
        
        if not hash_result.text then
            error("Hash calculation failed: no text in result")
        end
        
        local hash_parsed = JSON.parse(hash_result.text)
        if not hash_parsed.success then
            error("Hash calculation failed: " .. tostring(hash_parsed.message or "unknown"))
        end
        
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
        Err(e) => panic!("Tool chaining test failed: {e}"),
    }

    // Test 4: List all available tools
    let list_tools_test = r"
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
    ";

    match engine.execute_script(list_tools_test).await {
        Ok(result) => {
            let obj = result.output.as_object().expect("Expected object");
            let count = obj.get("count").unwrap().as_i64().unwrap();
            println!("✅ Found {count} tools registered");
            assert!(count >= 25, "Expected at least 25 tools, found {count}");
        }
        Err(e) => panic!("Tool listing test failed: {e}"),
    }

    // Test 5: System → Data → Utility chain (Env → JSON → Template)
    let system_chain_test = r#"
        -- System → Data → Utility chain
        -- Step 1: Read environment variable (using Tool instance)
        local env_tool = Tool.get("environment_reader")
        if not env_tool then
            error("Could not get environment_reader tool")
        end
        
        -- Create coroutine for async execution
        local co = coroutine.create(function()
            return env_tool:execute({
                operation = "get",
                variable_name = "PATH"
            })
        end)
        
        local success, env_result = coroutine.resume(co)
        while success and coroutine.status(co) ~= "dead" do
            success, env_result = coroutine.resume(co, env_result)
        end
        
        if not success then
            error("Environment read failed: " .. tostring(env_result))
        end
        
        if not env_result.text then
            error("Environment read failed: no text in result")
        end
        
        -- Parse result and check for success
        local env_parsed = JSON.parse(env_result.text)
        local has_env_output = env_parsed and env_parsed.success
        
        -- Step 2: Test JSON processor with simple data
        local test_data = {
            users = {
                {name = "Alice", age = 30},
                {name = "Bob", age = 25}
            }
        }
        
        local json_tool = Tool.get("json_processor")
        if not json_tool then
            error("Could not get json_processor tool")
        end
        
        -- Create coroutine for async execution
        local co2 = coroutine.create(function()
            return json_tool:execute({
                operation = "query",
                input = JSON.stringify(test_data),
                query = ".users | length"
            })
        end)
        
        local success2, json_result = coroutine.resume(co2)
        while success2 and coroutine.status(co2) ~= "dead" do
            success2, json_result = coroutine.resume(co2, json_result)
        end
        
        if not success2 then
            error("JSON processing failed: " .. tostring(json_result))
        end
        
        if not json_result.text then
            error("JSON processing failed: no text in result")
        end
        
        local json_parsed = JSON.parse(json_result.text)
        
        -- Debug: Print what we actually got
        print("DEBUG json_parsed type:", type(json_parsed))
        print("DEBUG json_parsed value:", tostring(json_parsed))
        if type(json_parsed) == "table" then
            print("DEBUG json_parsed.success:", json_parsed.success)
            print("DEBUG json_parsed.result type:", type(json_parsed.result))
            print("DEBUG json_parsed.result value:", tostring(json_parsed.result))
        end
        
        -- Handle if json_parsed is directly the number result
        local user_count = nil
        if type(json_parsed) == "number" then
            user_count = json_parsed
        elseif type(json_parsed) == "table" and json_parsed.success then
            if type(json_parsed.result) == "number" then
                user_count = json_parsed.result
            elseif json_parsed.result and type(json_parsed.result.result) == "number" then
                user_count = json_parsed.result.result
            end
        else
            error("JSON processing failed: " .. tostring(json_parsed.message or "unknown"))
        end
        user_count = user_count or 0
        
        -- Step 3: Use the result in a template
        local template_tool = Tool.get("template_engine")
        if not template_tool then
            error("Could not get template_engine tool")
        end
        
        -- Create coroutine for async execution
        local co3 = coroutine.create(function()
            return template_tool:execute({
                input = "Found {{count}} users in the system",
                context = {
                    count = user_count
                },
                engine = "handlebars"
            })
        end)
        
        local success3, template_result = coroutine.resume(co3)
        while success3 and coroutine.status(co3) ~= "dead" do
            success3, template_result = coroutine.resume(co3, template_result)
        end
        
        if not success3 then
            error("Template rendering failed: " .. tostring(template_result))
        end
        
        if not template_result.text then
            error("Template rendering failed: no text in result")
        end
        
        local template_parsed = JSON.parse(template_result.text)
        local template_output = nil
        if template_parsed.success and template_parsed.result then
            template_output = template_parsed.result.output or template_parsed.result.rendered or template_parsed.result
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
        Err(e) => panic!("System chain test failed: {e}"),
    }

    // Test 6: File → Data → File chain (Write → CSV → Read)
    let file_chain_test = r#"
        -- File → Data → File chain
        -- Step 1: Write initial data
        local test_data = "name,age,city\nAlice,30,NYC\nBob,25,LA\nCharlie,35,Chicago"
        
        local file_tool = Tool.get("file_operations")
        if not file_tool then
            error("Could not get file_operations tool")
        end
        
        -- Create coroutine for async execution
        local co = coroutine.create(function()
            return file_tool:execute({
                operation = "write",
                path = "/tmp/llmspell_test_data.csv",
                input = test_data
            })
        end)
        
        local success, write_result = coroutine.resume(co)
        while success and coroutine.status(co) ~= "dead" do
            success, write_result = coroutine.resume(co, write_result)
        end
        
        if not success then
            error("File write failed: " .. tostring(write_result))
        end
        
        if not write_result.text then
            error("File write failed: no text in result")
        end
        
        -- Handle file operations result - it returns plain text, not JSON
        if not string.find(write_result.text, "Wrote .* bytes") then
            error("File write failed: unexpected response format")
        end
        
        -- Step 2: Analyze the CSV data
        local csv_tool = Tool.get("csv_analyzer")
        if not csv_tool then
            error("Could not get csv_analyzer tool")
        end
        
        -- Create coroutine for async execution
        local co2 = coroutine.create(function()
            return csv_tool:execute({
                operation = "analyze",
                input = test_data
            })
        end)
        
        local success2, csv_result = coroutine.resume(co2)
        while success2 and coroutine.status(co2) ~= "dead" do
            success2, csv_result = coroutine.resume(co2, csv_result)
        end
        
        if not success2 then
            error("CSV analysis failed: " .. tostring(csv_result))
        end
        
        if not csv_result.text then
            error("CSV analysis failed: no text in result")
        end
        
        local csv_parsed = JSON.parse(csv_result.text)
        
        -- CSV analyzer returns data directly, not wrapped in success/result structure
        local row_count = csv_parsed.row_count or 0
        local column_count = csv_parsed.column_count or 0
        local headers = {}
        
        -- Extract header names from columns array
        if csv_parsed.columns and type(csv_parsed.columns) == "table" then
            for _, col in ipairs(csv_parsed.columns) do
                if col.name then
                    table.insert(headers, col.name)
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
        
        -- Create coroutine for async execution
        local co3 = coroutine.create(function()
            return file_tool:execute({
                operation = "write",
                path = "/tmp/llmspell_analysis.txt",
                input = analysis_content
            })
        end)
        
        local success3, write_analysis = coroutine.resume(co3)
        while success3 and coroutine.status(co3) ~= "dead" do
            success3, write_analysis = coroutine.resume(co3, write_analysis)
        end
        
        if not success3 then
            error("Analysis write failed: " .. tostring(write_analysis))
        end
        
        if not write_analysis.text then
            error("Analysis write failed: no text in result")
        end
        
        -- Handle file operations result - it returns plain text, not JSON
        if not string.find(write_analysis.text, "Wrote .* bytes") then
            error("Analysis write failed: unexpected response format")
        end
        
        -- Cleanup (using synchronous pattern since we don't check results)
        local co4 = coroutine.create(function()
            return file_tool:execute({
                operation = "delete",
                path = "/tmp/llmspell_test_data.csv"
            })
        end)
        coroutine.resume(co4)
        
        local co5 = coroutine.create(function()
            return file_tool:execute({
                operation = "delete",
                path = "/tmp/llmspell_analysis.txt"
            })
        end)
        coroutine.resume(co5)
        
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
        Err(e) => panic!("File chain test failed: {e}"),
    }

    // Test 7: Data → System → File chain (HTTP simulation → Process → Save)
    let data_system_file_test = r#"
        -- Data → System → File chain
        -- Step 1: Simulate HTTP data (would normally be http_request)
        local mock_data = {
            users = {
                {name = "Alice", score = 95},
                {name = "Bob", score = 87},
                {name = "Charlie", score = 92}
            }
        }
        
        -- Use json_processor to query the data
        local json_tool = Tool.get("json_processor")
        if not json_tool then
            error("Could not get json_processor tool")
        end
        
        -- Create coroutine for async execution
        local co = coroutine.create(function()
            return json_tool:execute({
                operation = "query",
                input = JSON.stringify(mock_data),
                query = ".users | length"
            })
        end)
        
        local success, json_result = coroutine.resume(co)
        while success and coroutine.status(co) ~= "dead" do
            success, json_result = coroutine.resume(co, json_result)
        end
        
        if not success then
            error("JSON query failed: " .. tostring(json_result))
        end
        
        if not json_result.text then
            error("JSON query failed: no text in result")
        end
        
        local json_parsed = JSON.parse(json_result.text)
        
        -- Handle JSON processor returning direct number result  
        local user_count = nil
        if type(json_parsed) == "number" then
            user_count = json_parsed
        elseif type(json_parsed) == "table" and json_parsed.success then
            if type(json_parsed.result) == "number" then
                user_count = json_parsed.result
            elseif json_parsed.result and type(json_parsed.result.result) == "number" then
                user_count = json_parsed.result.result
            end
        else
            error("JSON query failed: " .. tostring(json_parsed.message or "unknown"))
        end
        user_count = user_count or 0
        
        -- Step 2: Use environment reader
        local env_tool = Tool.get("environment_reader")
        if not env_tool then
            error("Could not get environment_reader tool")
        end
        
        -- Create coroutine for async execution
        local co2 = coroutine.create(function()
            return env_tool:execute({
                operation = "list",
                pattern = "PATH"
            })
        end)
        
        local success2, env_result = coroutine.resume(co2)
        while success2 and coroutine.status(co2) ~= "dead" do
            success2, env_result = coroutine.resume(co2, env_result)
        end
        
        if not success2 then
            error("Environment read failed: " .. tostring(env_result))
        end
        
        if not env_result.text then
            error("Environment read failed: no text in result")
        end
        
        local env_parsed = JSON.parse(env_result.text)
        local system_checked = env_parsed and env_parsed.success
        
        -- Step 3: Save the results to a file
        local file_tool = Tool.get("file_operations")
        if not file_tool then
            error("Could not get file_operations tool")
        end
        
        -- Create coroutine for async execution
        local co3 = coroutine.create(function()
            return file_tool:execute({
                operation = "write",
                path = "/tmp/llmspell_data_chain.txt",
                input = string.format("Data Chain Results:\nUser count: %d\nEnvironment checked: true\nTimestamp: %s", 
                    user_count, os.date())
            })
        end)
        
        local success3, save_result = coroutine.resume(co3)
        while success3 and coroutine.status(co3) ~= "dead" do
            success3, save_result = coroutine.resume(co3, save_result)
        end
        
        if not success3 then
            error("File save failed: " .. tostring(save_result))
        end
        
        if not save_result.text then
            error("File save failed: no text in result")
        end
        
        -- Handle file operations result - it returns plain text, not JSON
        if not string.find(save_result.text, "Wrote .* bytes") then
            error("File save failed: unexpected response format")
        end
        
        -- Cleanup (using synchronous pattern since we don't check results)
        local co4 = coroutine.create(function()
            return file_tool:execute({
                operation = "delete",
                path = "/tmp/llmspell_data_chain.txt"
            })
        end)
        coroutine.resume(co4)
        
        return {
            data_queried = user_count == 3,
            system_checked = system_checked,
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
        Err(e) => panic!("Data → System → File chain test failed: {e}"),
    }

    // Test 8: Error propagation through chains
    let error_chain_test = r#"
        -- Test error propagation in tool chains
        local chain_errors = {}
        
        -- Chain with invalid tool
        local success1 = pcall(function()
            local result = Tool.get("nonexistent_tool")
            if result then
                error("Should not get nonexistent tool")
            end
        end)
        chain_errors.invalid_tool = not success1 or Tool.get("nonexistent_tool") == nil
        
        -- Chain with invalid parameters
        local success2 = pcall(function()
            local tool = Tool.get("base64_encoder")
            if not tool then
                error("Could not get base64_encoder tool")
            end
            
            -- Create coroutine for async execution with invalid operation
            local co = coroutine.create(function()
                return tool:execute({
                    operation = "invalid_op",
                    input = "test"
                })
            end)
            
            local success, result = coroutine.resume(co)
            while success and coroutine.status(co) ~= "dead" do
                success, result = coroutine.resume(co, result)
            end
            
            if not success then
                error("Tool execution failed: " .. tostring(result))
            end
            
            if not result.text then
                error("Tool execution failed: no text in result")
            end
            
            local parsed = JSON.parse(result.text)
            if not parsed.success then
                error("Tool execution failed: " .. tostring(parsed.message or "unknown"))
            end
        end)
        chain_errors.invalid_params = not success2
        
        -- Chain where middle step fails
        local success3, err3 = pcall(function()
            -- Step 1: Generate UUID (should work)
            local uuid_tool = Tool.get("uuid_generator")
            if not uuid_tool then
                error("Could not get uuid_generator tool")
            end
            
            local co = coroutine.create(function()
                return uuid_tool:execute({
                    operation = "generate",
                    version = "v4"
                })
            end)
            
            local success, uuid_result = coroutine.resume(co)
            while success and coroutine.status(co) ~= "dead" do
                success, uuid_result = coroutine.resume(co, uuid_result)
            end
            
            if not success then
                error("UUID generation failed: " .. tostring(uuid_result))
            end
            
            -- Step 2: Try to hash with missing required parameter
            local hash_tool = Tool.get("hash_calculator")
            if not hash_tool then
                error("Could not get hash_calculator tool")
            end
            
            local co2 = coroutine.create(function()
                return hash_tool:execute({
                    operation = "hash",
                    algorithm = "md5"
                    -- Missing required 'input' parameter
                })
            end)
            
            local success2, hash_result = coroutine.resume(co2)
            while success2 and coroutine.status(co2) ~= "dead" do
                success2, hash_result = coroutine.resume(co2, hash_result)
            end
            
            if not success2 then
                error("Chain failed at hash step: " .. tostring(hash_result))
            end
            
            if not hash_result.text then
                error("Chain failed at hash step: no text in result")
            end
            
            local hash_parsed = JSON.parse(hash_result.text)
            if not hash_parsed.success then
                error("Chain failed at hash step: " .. tostring(hash_parsed.message or "unknown"))
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
        Err(e) => panic!("Error propagation test failed: {e}"),
    }

    println!("\n✅ All integration tests passed!");
}

#[tokio::test(flavor = "multi_thread")]
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
            r#"Tool.get("hash_calculator"):execute({operation = "hash", algorithm = "md5", input = "test"})"#,
        ),
        (
            "calculator",
            r#"Tool.get("calculator"):execute({operation = "evaluate", input = "1+1"})"#,
        ),
    ];

    println!("\n=== Tool Performance Benchmarks ===");

    for (name, script) in benchmarks {
        let full_script = format!("return {script}");
        let iterations = 50;

        let start = Instant::now();
        for _ in 0..iterations {
            let _ = engine.execute_script(&full_script).await;
        }
        let elapsed = start.elapsed();

        let per_op_ms = elapsed.as_micros() as f64 / f64::from(iterations) / 1000.0;

        println!("{name:<20} {per_op_ms:>8.3} ms/op");

        // Check <10ms requirement
        if per_op_ms > 10.0 {
            println!("  ⚠️  WARNING: Exceeds 10ms target!");
        }
    }
}
