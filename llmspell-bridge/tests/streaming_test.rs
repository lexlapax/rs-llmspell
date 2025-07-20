//! ABOUTME: Integration tests for Lua streaming support
//! ABOUTME: Validates coroutine-based streaming and API functionality

#[cfg(feature = "lua")]
mod tests {
    use llmspell_bridge::{
        engine::factory::{EngineFactory, LuaConfig},
        providers::{ProviderManager, ProviderManagerConfig},
        registry::ComponentRegistry,
    };
    use std::sync::Arc;

    #[tokio::test]
    async fn test_lua_streaming_api_injection() {
        let config = LuaConfig::default();
        let mut engine = EngineFactory::create_lua_engine(&config).unwrap();

        // Create mock registry and provider manager
        let registry = Arc::new(ComponentRegistry::new());
        let provider_config = ProviderManagerConfig::default();
        let providers = Arc::new(ProviderManager::new(provider_config).await.unwrap());

        // Inject APIs including streaming
        let result = engine.inject_apis(&registry, &providers);
        assert!(result.is_ok(), "Failed to inject APIs with streaming");

        // Test that Streaming global exists
        let script = "return Streaming ~= nil";
        let output = engine.execute_script(script).await;

        match output {
            Ok(result) => {
                assert_eq!(
                    result.output.as_bool(),
                    Some(true),
                    "Streaming global not found"
                );
            }
            Err(e) => panic!("Script execution failed: {:?}", e),
        }
    }

    #[tokio::test]
    async fn test_lua_coroutine_streaming() {
        let config = LuaConfig::default();
        let mut engine = EngineFactory::create_lua_engine(&config).unwrap();

        // Create mock registry and provider manager
        let registry = Arc::new(ComponentRegistry::new());
        let provider_config = ProviderManagerConfig::default();
        let providers = Arc::new(ProviderManager::new(provider_config).await.unwrap());

        // Inject APIs
        engine.inject_apis(&registry, &providers).unwrap();

        // Test creating a stream with coroutines
        let script = r#"
            local stream = Streaming.create(function()
                coroutine.yield("chunk1")
                coroutine.yield("chunk2")
                coroutine.yield("chunk3")
            end)
            
            return {
                exists = stream ~= nil,
                hasNext = type(stream.next) == "function",
                hasIsDone = type(stream.isDone) == "function",
                hasCollect = type(stream.collect) == "function"
            }
        "#;

        let output = engine.execute_script(script).await;

        match output {
            Ok(result) => {
                let obj = result.output.as_object().expect("Expected object result");
                assert_eq!(obj.get("exists").and_then(|v| v.as_bool()), Some(true));
                assert_eq!(obj.get("hasNext").and_then(|v| v.as_bool()), Some(true));
                assert_eq!(obj.get("hasIsDone").and_then(|v| v.as_bool()), Some(true));
                assert_eq!(obj.get("hasCollect").and_then(|v| v.as_bool()), Some(true));
            }
            Err(e) => panic!("Script execution failed: {:?}", e),
        }
    }

    #[tokio::test]
    async fn test_lua_tool_api() {
        let config = LuaConfig::default();
        let mut engine = EngineFactory::create_lua_engine(&config).unwrap();

        // Create mock registry and provider manager
        let registry = Arc::new(ComponentRegistry::new());
        let provider_config = ProviderManagerConfig::default();
        let providers = Arc::new(ProviderManager::new(provider_config).await.unwrap());

        // Register tools with the registry
        llmspell_bridge::tools::register_all_tools(registry.clone()).unwrap();

        // Inject APIs
        engine.inject_apis(&registry, &providers).unwrap();

        // Test Tool API
        let script = r#"
            -- Check Tool global exists
            local toolExists = Tool ~= nil
            
            -- Get list of tools
            local tools = Tool.list()
            local hasTools = #tools > 0
            
            -- Try to get a tool
            local calc = Tool.get("calculator")
            local toolWorks = calc ~= nil and calc.name == "calculator"
            
            return {
                toolExists = toolExists,
                hasTools = hasTools,
                toolWorks = toolWorks
            }
        "#;

        let output = engine.execute_script(script).await;

        match output {
            Ok(result) => {
                let obj = result.output.as_object().expect("Expected object result");
                assert_eq!(obj.get("toolExists").and_then(|v| v.as_bool()), Some(true));
                assert_eq!(obj.get("hasTools").and_then(|v| v.as_bool()), Some(true));
                assert_eq!(obj.get("toolWorks").and_then(|v| v.as_bool()), Some(true));
            }
            Err(e) => panic!("Script execution failed: {:?}", e),
        }
    }

    #[tokio::test]
    async fn test_lua_workflow_api() {
        let config = LuaConfig::default();
        let mut engine = EngineFactory::create_lua_engine(&config).unwrap();

        // Create mock registry and provider manager
        let registry = Arc::new(ComponentRegistry::new());
        let provider_config = ProviderManagerConfig::default();
        let providers = Arc::new(ProviderManager::new(provider_config).await.unwrap());

        // Inject APIs
        engine.inject_apis(&registry, &providers).unwrap();

        // Test Workflow API
        let script = r#"
            -- Check Workflow global exists
            local workflowExists = Workflow ~= nil
            
            -- Create sequential workflow
            local seq = Workflow.sequential({
                {type = "step1"},
                {type = "step2"}
            })
            
            -- Create parallel workflow
            local par = Workflow.parallel({
                {type = "task1"},
                {type = "task2"}
            })
            
            return {
                workflowExists = workflowExists,
                seqType = seq and seq.type or nil,
                parType = par and par.type or nil,
                seqHasExecute = type(Workflow.execute) == "function",
                parHasExecute = type(Workflow.execute) == "function"
            }
        "#;

        let output = engine.execute_script(script).await;

        match output {
            Ok(result) => {
                let obj = result.output.as_object().expect("Expected object result");
                assert_eq!(
                    obj.get("workflowExists").and_then(|v| v.as_bool()),
                    Some(true)
                );
                assert_eq!(
                    obj.get("seqType").and_then(|v| v.as_str()),
                    Some("sequential")
                );
                assert_eq!(
                    obj.get("parType").and_then(|v| v.as_str()),
                    Some("parallel")
                );
                assert_eq!(
                    obj.get("seqHasExecute").and_then(|v| v.as_bool()),
                    Some(true)
                );
                assert_eq!(
                    obj.get("parHasExecute").and_then(|v| v.as_bool()),
                    Some(true)
                );
            }
            Err(e) => panic!("Script execution failed: {:?}", e),
        }
    }

    #[tokio::test]
    async fn test_lua_streaming_execution() {
        let config = LuaConfig::default();
        let mut engine = EngineFactory::create_lua_engine(&config).unwrap();

        // Create mock registry and provider manager
        let registry = Arc::new(ComponentRegistry::new());
        let provider_config = ProviderManagerConfig::default();
        let providers = Arc::new(ProviderManager::new(provider_config).await.unwrap());

        // Inject APIs first
        engine.inject_apis(&registry, &providers).unwrap();

        // Test that streaming execution returns appropriate error for now
        let script = "return 'test'";
        let stream_result = engine.execute_script_streaming(script).await;

        // For now, this should work as we have a basic implementation
        match stream_result {
            Ok(_) => println!("Streaming execution succeeded"),
            Err(e) => panic!("Streaming execution failed: {:?}", e),
        }
    }
}
