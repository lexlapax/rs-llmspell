//! ABOUTME: Integration tests for Lua streaming support
//! ABOUTME: Validates coroutine-based streaming and API functionality

mod test_helpers;

#[cfg(feature = "lua")]
mod tests {
    use crate::test_helpers::create_test_infrastructure;
    use llmspell_bridge::{
        engine::factory::{EngineFactory, LuaConfig},
        providers::ProviderManager,
        registry::ComponentRegistry,
    };
    use llmspell_config::providers::ProviderManagerConfig;
    use std::sync::Arc;

    #[tokio::test(flavor = "multi_thread")]
    async fn test_lua_streaming_api_injection() {
        let config = LuaConfig::default();
        let mut engine = EngineFactory::create_lua_engine(&config).unwrap();

        // Create mock registry and provider manager
        let registry = Arc::new(ComponentRegistry::new());
        let provider_config = ProviderManagerConfig::default();
        let providers = Arc::new(ProviderManager::new(provider_config).await.unwrap());

        // Inject APIs including streaming
        let (tool_registry, agent_registry, workflow_factory) = create_test_infrastructure();

        let result = engine.inject_apis(
            &registry,
            &providers,
            &tool_registry,
            &agent_registry,
            &workflow_factory,
            None,
        );
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
            Err(e) => panic!("Script execution failed: {e:?}"),
        }
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_lua_coroutine_streaming() {
        let config = LuaConfig::default();
        let mut engine = EngineFactory::create_lua_engine(&config).unwrap();

        // Create mock registry and provider manager
        let registry = Arc::new(ComponentRegistry::new());
        let provider_config = ProviderManagerConfig::default();
        let providers = Arc::new(ProviderManager::new(provider_config).await.unwrap());

        // Inject APIs
        let (tool_registry, agent_registry, workflow_factory) = create_test_infrastructure();

        engine
            .inject_apis(
                &registry,
                &providers,
                &tool_registry,
                &agent_registry,
                &workflow_factory,
                None,
            )
            .unwrap();

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
                assert_eq!(
                    obj.get("exists")
                        .and_then(serde_json::value::Value::as_bool),
                    Some(true)
                );
                assert_eq!(
                    obj.get("hasNext")
                        .and_then(serde_json::value::Value::as_bool),
                    Some(true)
                );
                assert_eq!(
                    obj.get("hasIsDone")
                        .and_then(serde_json::value::Value::as_bool),
                    Some(true)
                );
                assert_eq!(
                    obj.get("hasCollect")
                        .and_then(serde_json::value::Value::as_bool),
                    Some(true)
                );
            }
            Err(e) => panic!("Script execution failed: {e:?}"),
        }
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_lua_tool_api() {
        let config = LuaConfig::default();
        let mut engine = EngineFactory::create_lua_engine(&config).unwrap();

        // Create mock registry and provider manager
        let registry = Arc::new(ComponentRegistry::new());
        let tool_registry = Arc::new(llmspell_tools::ToolRegistry::new());
        let provider_config = ProviderManagerConfig::default();
        let providers = Arc::new(ProviderManager::new(provider_config).await.unwrap());

        // Register tools with the registry
        let tools_config = llmspell_config::tools::ToolsConfig::default();
        llmspell_bridge::tools::register_all_tools(&registry, &tool_registry, &tools_config)
            .await
            .unwrap();

        // Inject APIs
        let (tool_registry, agent_registry, workflow_factory) = create_test_infrastructure();

        engine
            .inject_apis(
                &registry,
                &providers,
                &tool_registry,
                &agent_registry,
                &workflow_factory,
                None,
            )
            .unwrap();

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
                assert_eq!(
                    obj.get("toolExists")
                        .and_then(serde_json::value::Value::as_bool),
                    Some(true)
                );
                assert_eq!(
                    obj.get("hasTools")
                        .and_then(serde_json::value::Value::as_bool),
                    Some(true)
                );
                assert_eq!(
                    obj.get("toolWorks")
                        .and_then(serde_json::value::Value::as_bool),
                    Some(true)
                );
            }
            Err(e) => panic!("Script execution failed: {e:?}"),
        }
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_lua_workflow_api() {
        let config = LuaConfig::default();
        let mut engine = EngineFactory::create_lua_engine(&config).unwrap();

        // Create mock registry and provider manager
        let registry = Arc::new(ComponentRegistry::new());
        let tool_registry = Arc::new(llmspell_tools::ToolRegistry::new());
        let provider_config = ProviderManagerConfig::default();
        let providers = Arc::new(ProviderManager::new(provider_config).await.unwrap());

        // Register tools with the registry
        let tools_config = llmspell_config::tools::ToolsConfig::default();
        llmspell_bridge::tools::register_all_tools(&registry, &tool_registry, &tools_config)
            .await
            .unwrap();

        // Inject APIs
        let (tool_registry, agent_registry, workflow_factory) = create_test_infrastructure();

        engine
            .inject_apis(
                &registry,
                &providers,
                &tool_registry,
                &agent_registry,
                &workflow_factory,
                None,
            )
            .unwrap();

        // Test Workflow API
        let script = r#"
            -- Check Workflow global exists
            local workflowExists = Workflow ~= nil
            
            -- Create sequential workflow
            local seq = Workflow.sequential({
                name = "test_seq",
                description = "Test sequential workflow",
                steps = {
                    {name = "step1", type = "tool", tool = "uuid-generator", input = {}},
                    {name = "step2", type = "tool", tool = "hash-calculator", input = {algorithm = "sha256", input = "test"}}
                }
            })
            
            -- Create parallel workflow
            local par = Workflow.parallel({
                name = "test_par",
                description = "Test parallel workflow",
                steps = {
                    {name = "task1", type = "tool", tool = "uuid-generator", input = {}},
                    {name = "task2", type = "tool", tool = "date_time_handler", input = {operation = "now"}}
                }
            })
            
            return {
                workflowExists = workflowExists,
                seqType = seq and seq.type or nil,
                parType = par and par.type or nil,
                seqHasExecute = seq and type(seq.execute) == "function" or false,
                parHasExecute = par and type(par.execute) == "function" or false
            }
        "#;

        let output = engine.execute_script(script).await;

        match output {
            Ok(result) => {
                let obj = result.output.as_object().expect("Expected object result");
                assert_eq!(
                    obj.get("workflowExists")
                        .and_then(serde_json::value::Value::as_bool),
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
                    obj.get("seqHasExecute")
                        .and_then(serde_json::value::Value::as_bool),
                    Some(true)
                );
                assert_eq!(
                    obj.get("parHasExecute")
                        .and_then(serde_json::value::Value::as_bool),
                    Some(true)
                );
            }
            Err(e) => panic!("Script execution failed: {e:?}"),
        }
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_lua_streaming_execution() {
        let config = LuaConfig::default();
        let mut engine = EngineFactory::create_lua_engine(&config).unwrap();

        // Create mock registry and provider manager
        let registry = Arc::new(ComponentRegistry::new());
        let provider_config = ProviderManagerConfig::default();
        let providers = Arc::new(ProviderManager::new(provider_config).await.unwrap());

        // Inject APIs first
        let (tool_registry, agent_registry, workflow_factory) = create_test_infrastructure();

        engine
            .inject_apis(
                &registry,
                &providers,
                &tool_registry,
                &agent_registry,
                &workflow_factory,
                None,
            )
            .unwrap();

        // Test that streaming execution returns appropriate error for now
        let script = "return 'test'";
        let stream_result = engine.execute_script_streaming(script).await;

        // For now, this should work as we have a basic implementation
        match stream_result {
            Ok(_) => println!("Streaming execution succeeded"),
            Err(e) => panic!("Streaming execution failed: {e:?}"),
        }
    }
}
