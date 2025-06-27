//! ABOUTME: End-to-end integration tests for the script engine bridge
//! ABOUTME: Validates complete workflows from script execution to provider calls

use llmspell_bridge::{
    engine::factory::{EngineFactory, LuaConfig},
    providers::{ProviderConfig, ProviderManager, ProviderManagerConfig},
    ComponentRegistry,
};
use llmspell_core::{
    traits::agent::{AgentConfig, ConversationMessage},
    Agent,
};
use std::collections::HashMap;
use std::sync::Arc;

/// Test complete script execution through bridge abstraction
#[tokio::test]
async fn test_script_execution_through_bridge() {
    // Create engine through factory
    let lua_config = LuaConfig::default();
    let mut engine = EngineFactory::create_lua_engine(&lua_config).unwrap();

    // Set up dependencies
    let registry = Arc::new(ComponentRegistry::new());
    let provider_config = ProviderManagerConfig::default();
    let providers = Arc::new(ProviderManager::new(provider_config).await.unwrap());

    // Inject APIs
    engine.inject_apis(&registry, &providers).unwrap();

    // Execute a simple script
    let script = r#"
        local result = {}
        result.engine = "lua"
        result.version = _VERSION
        result.apis = {
            agent = type(Agent) == 'table',
            tool = type(Tool) == 'table',
            workflow = type(Workflow) == 'table'
        }
        return result
    "#;

    let output = engine.execute_script(script).await.unwrap();

    // Verify output
    let result = output.output.as_object().unwrap();
    assert_eq!(result.get("engine").unwrap().as_str().unwrap(), "lua");
    assert!(result
        .get("version")
        .unwrap()
        .as_str()
        .unwrap()
        .contains("Lua"));

    let apis = result.get("apis").unwrap().as_object().unwrap();
    assert_eq!(apis.get("agent").unwrap().as_bool().unwrap(), true);
    assert_eq!(apis.get("tool").unwrap().as_bool().unwrap(), true);
    assert_eq!(apis.get("workflow").unwrap().as_bool().unwrap(), true);
}

/// Test engine switching capability (even with only Lua)
#[tokio::test]
async fn test_engine_switching_integration() {
    // Test that we can create engines by name
    let config = serde_json::json!({
        "stdlib": "safe",
        "max_memory": 50000000,
        "debug": false,
        "package_paths": []
    });

    // Create Lua engine by name
    let engine = EngineFactory::create_from_name("lua", &config);
    assert!(engine.is_ok(), "Should create Lua engine by name");

    // Try to create non-existent engine
    let unknown = EngineFactory::create_from_name("python", &config);
    assert!(unknown.is_err(), "Should fail for unknown engine");

    // List available engines
    let engines = EngineFactory::list_available_engines();
    assert!(
        engines.iter().any(|e| e.name == "lua"),
        "Lua should be in available engines"
    );
}

/// Test streaming capabilities through bridge
#[tokio::test]
async fn test_streaming_through_bridge() {
    let lua_config = LuaConfig::default();
    let mut engine = EngineFactory::create_lua_engine(&lua_config).unwrap();

    // Verify streaming support
    assert!(engine.supports_streaming(), "Lua should support streaming");

    // Set up dependencies
    let registry = Arc::new(ComponentRegistry::new());
    let provider_config = ProviderManagerConfig::default();
    let providers = Arc::new(ProviderManager::new(provider_config).await.unwrap());

    engine.inject_apis(&registry, &providers).unwrap();

    // Try streaming execution (stub for now)
    let result = engine
        .execute_script_streaming("return 'streaming test'")
        .await;
    // Streaming returns a stub implementation or error
    match result {
        Err(e) => {
            // Expected for now - streaming not fully implemented
            println!("Streaming returned error as expected: {}", e);
        }
        Ok(stream) => {
            // If it succeeds, it should return a valid stream
            assert_eq!(stream.metadata.engine, "lua");
        }
    }
}

/// Test provider integration through scripts
#[tokio::test]
async fn test_provider_integration() {
    let lua_config = LuaConfig::default();
    let mut engine = EngineFactory::create_lua_engine(&lua_config).unwrap();

    // Set up provider configuration
    let mut provider_config = ProviderManagerConfig::default();

    // Note: This test will fail if OPENAI_API_KEY is not set
    // In a real test, we'd use a mock provider
    provider_config.providers.insert(
        "test-openai".to_string(),
        ProviderConfig {
            provider_type: "openai".to_string(),
            api_key_env: Some("OPENAI_API_KEY".to_string()),
            base_url: None,
            model: Some("gpt-3.5-turbo".to_string()),
            max_tokens: Some(100),
            extra: HashMap::new(),
        },
    );

    let registry = Arc::new(ComponentRegistry::new());
    let providers = Arc::new(match ProviderManager::new(provider_config).await {
        Ok(manager) => manager,
        Err(_) => {
            // If provider creation fails (no API key), create empty manager
            ProviderManager::new(ProviderManagerConfig::default())
                .await
                .unwrap()
        }
    });

    engine.inject_apis(&registry, &providers).unwrap();

    // Test that we can access provider functionality
    let script = r#"
        -- Test that Agent API is available
        return Agent ~= nil and type(Agent.create) == 'function'
    "#;

    let output = engine.execute_script(script).await.unwrap();
    assert_eq!(output.output.as_bool(), Some(true));
}

/// Test error propagation from scripts
#[tokio::test]
async fn test_error_propagation() {
    let lua_config = LuaConfig::default();
    let mut engine = EngineFactory::create_lua_engine(&lua_config).unwrap();

    let registry = Arc::new(ComponentRegistry::new());
    let provider_config = ProviderManagerConfig::default();
    let providers = Arc::new(ProviderManager::new(provider_config).await.unwrap());

    engine.inject_apis(&registry, &providers).unwrap();

    // Test various error scenarios
    let error_cases = vec![
        ("syntax error {{", "syntax"),
        ("error('runtime error')", "runtime"),
        ("nil + 1", "type"),
        ("unknown_function()", "undefined"),
    ];

    for (script, error_type) in error_cases {
        let result = engine.execute_script(script).await;
        assert!(result.is_err(), "Script '{}' should fail", script);

        let error_msg = result.unwrap_err().to_string();
        println!("Error for {}: {}", error_type, error_msg);
    }
}

/// Test multimodal type access from scripts
#[tokio::test]
async fn test_multimodal_types_access() {
    let lua_config = LuaConfig::default();
    let mut engine = EngineFactory::create_lua_engine(&lua_config).unwrap();

    assert!(
        engine.supports_multimodal(),
        "Lua should support multimodal"
    );

    let registry = Arc::new(ComponentRegistry::new());
    let provider_config = ProviderManagerConfig::default();
    let providers = Arc::new(ProviderManager::new(provider_config).await.unwrap());

    engine.inject_apis(&registry, &providers).unwrap();

    // Test creating multimodal content (when API is available)
    let script = r#"
        -- For now, just verify APIs are injected
        return {
            agent_available = Agent ~= nil,
            tool_available = Tool ~= nil,
            workflow_available = Workflow ~= nil
        }
    "#;

    let output = engine.execute_script(script).await.unwrap();
    let result = output.output.as_object().unwrap();

    assert_eq!(result.get("agent_available").unwrap().as_bool(), Some(true));
    assert_eq!(result.get("tool_available").unwrap().as_bool(), Some(true));
    assert_eq!(
        result.get("workflow_available").unwrap().as_bool(),
        Some(true)
    );
}

/// Test execution context management
#[tokio::test]
async fn test_execution_context_integration() {
    let lua_config = LuaConfig::default();
    let mut engine = EngineFactory::create_lua_engine(&lua_config).unwrap();

    let registry = Arc::new(ComponentRegistry::new());
    let provider_config = ProviderManagerConfig::default();
    let providers = Arc::new(ProviderManager::new(provider_config).await.unwrap());

    engine.inject_apis(&registry, &providers).unwrap();

    // Set custom execution context
    let mut context = llmspell_bridge::engine::bridge::ExecutionContext::default();
    context.working_directory = "/test/dir".to_string();
    context
        .environment
        .insert("TEST_VAR".to_string(), "test_value".to_string());
    context.state = serde_json::json!({"custom": "state"});

    engine.set_execution_context(context.clone()).unwrap();

    // Verify context was set
    let retrieved = engine.get_execution_context().unwrap();
    assert_eq!(retrieved.working_directory, "/test/dir");
    assert_eq!(
        retrieved.environment.get("TEST_VAR"),
        Some(&"test_value".to_string())
    );
}

/// Test performance benchmarks with bridge overhead
#[tokio::test]
async fn test_bridge_performance_overhead() {
    use std::time::Instant;

    let lua_config = LuaConfig::default();
    let mut engine = EngineFactory::create_lua_engine(&lua_config).unwrap();

    let registry = Arc::new(ComponentRegistry::new());
    let provider_config = ProviderManagerConfig::default();
    let providers = Arc::new(ProviderManager::new(provider_config).await.unwrap());

    engine.inject_apis(&registry, &providers).unwrap();

    // Benchmark simple script execution
    let script = "return 1 + 1";
    let iterations = 100;

    let start = Instant::now();
    for _ in 0..iterations {
        let _ = engine.execute_script(script).await.unwrap();
    }
    let duration = start.elapsed();

    let avg_time = duration.as_micros() / iterations;
    println!("Average execution time: {}μs", avg_time);

    // Bridge overhead should be minimal (< 1ms per execution)
    assert!(avg_time < 1000, "Bridge overhead should be < 1ms");
}

/// Test component registration and access
#[tokio::test]
async fn test_component_registration_integration() {
    use async_trait::async_trait;
    use llmspell_core::error::LLMSpellError;
    use llmspell_core::types::{AgentInput, AgentOutput, ExecutionContext};
    use llmspell_core::{BaseAgent, ComponentMetadata};

    // Create a mock agent
    struct MockAgent {
        metadata: ComponentMetadata,
        config: AgentConfig,
    }

    #[async_trait]
    impl BaseAgent for MockAgent {
        fn metadata(&self) -> &ComponentMetadata {
            &self.metadata
        }

        async fn execute(
            &self,
            _input: AgentInput,
            _context: ExecutionContext,
        ) -> Result<AgentOutput, LLMSpellError> {
            Ok(AgentOutput::text("Mock response"))
        }

        async fn validate_input(&self, _input: &AgentInput) -> Result<(), LLMSpellError> {
            Ok(())
        }

        async fn handle_error(&self, error: LLMSpellError) -> Result<AgentOutput, LLMSpellError> {
            Ok(AgentOutput::text(format!("Error: {}", error)))
        }
    }

    #[async_trait]
    impl Agent for MockAgent {
        fn config(&self) -> &AgentConfig {
            &self.config
        }

        async fn get_conversation(&self) -> Result<Vec<ConversationMessage>, LLMSpellError> {
            Ok(vec![])
        }

        async fn add_message(
            &mut self,
            _message: ConversationMessage,
        ) -> Result<(), LLMSpellError> {
            Ok(())
        }

        async fn clear_conversation(&mut self) -> Result<(), LLMSpellError> {
            Ok(())
        }
    }

    // Set up engine and registry
    let lua_config = LuaConfig::default();
    let mut engine = EngineFactory::create_lua_engine(&lua_config).unwrap();

    let registry = Arc::new(ComponentRegistry::new());

    // Register mock agent
    let mock_agent = Arc::new(MockAgent {
        metadata: ComponentMetadata::new("mock-agent".to_string(), "A mock agent".to_string()),
        config: AgentConfig::default(),
    });

    registry
        .register_agent("mock-agent".to_string(), mock_agent)
        .unwrap();

    let provider_config = ProviderManagerConfig::default();
    let providers = Arc::new(ProviderManager::new(provider_config).await.unwrap());

    engine.inject_apis(&registry, &providers).unwrap();

    // Verify registry works
    assert_eq!(registry.list_agents(), vec!["mock-agent"]);
    assert!(registry.get_agent("mock-agent").is_some());
}

/// Test concurrent script execution
#[tokio::test]
async fn test_concurrent_script_execution() {
    let lua_config = LuaConfig::default();
    let mut engine = EngineFactory::create_lua_engine(&lua_config).unwrap();

    let registry = Arc::new(ComponentRegistry::new());
    let provider_config = ProviderManagerConfig::default();
    let providers = Arc::new(ProviderManager::new(provider_config).await.unwrap());

    engine.inject_apis(&registry, &providers).unwrap();

    let engine = Arc::new(engine);

    // Run multiple scripts concurrently
    let mut handles = vec![];

    for i in 0..10 {
        let engine_clone = engine.clone();
        let handle = tokio::spawn(async move {
            let script = format!("return {}", i * i);
            engine_clone.execute_script(&script).await
        });
        handles.push(handle);
    }

    // All should complete successfully
    for (i, handle) in handles.into_iter().enumerate() {
        let result = handle.await.unwrap();
        assert!(result.is_ok());
        let output = result.unwrap();
        assert_eq!(output.output.as_i64(), Some((i * i) as i64));
    }
}
