//! ABOUTME: Integration tests for `ScriptRuntime` with multiple engines
//! ABOUTME: Validates language-agnostic runtime and engine switching

use llmspell_bridge::{engine::factory::EngineFactory, runtime::ScriptRuntime};
use llmspell_config::LLMSpellConfig;

#[tokio::test(flavor = "multi_thread")]
async fn test_runtime_with_lua_engine() {
    let config = LLMSpellConfig::default();
    assert_eq!(config.default_engine, "lua");

    let runtime = ScriptRuntime::new_with_lua(config).await;
    assert!(runtime.is_ok(), "Failed to create runtime with Lua engine");

    let runtime = runtime.unwrap();
    assert_eq!(runtime.get_engine_name(), "lua");
    assert!(runtime.supports_streaming());
    assert!(runtime.supports_multimodal());
}

#[tokio::test(flavor = "multi_thread")]
async fn test_runtime_with_engine_name() {
    let config = LLMSpellConfig::default();

    // Test creating with Lua by name
    let runtime = ScriptRuntime::new_with_engine_name("lua", config.clone()).await;
    assert!(
        runtime.is_ok(),
        "Failed to create runtime with engine name 'lua'"
    );

    let runtime = runtime.unwrap();
    assert_eq!(runtime.get_engine_name(), "lua");
}

#[tokio::test(flavor = "multi_thread")]
async fn test_runtime_execute_script() {
    let config = LLMSpellConfig::default();
    let runtime = ScriptRuntime::new_with_lua(config).await.unwrap();

    // Execute a simple script
    let result = runtime.execute_script("return 1 + 1").await;

    match result {
        Ok(output) => {
            assert_eq!(output.output.as_i64(), Some(2));
        }
        Err(e) => panic!("Script execution failed: {e:?}"),
    }
}

#[tokio::test(flavor = "multi_thread")]
async fn test_runtime_capability_detection() {
    let config = LLMSpellConfig::default();
    let runtime = ScriptRuntime::new_with_lua(config).await.unwrap();

    // Test capability detection
    let features = runtime.get_engine_features();
    assert!(features.async_execution);
    assert!(features.streaming);
    assert!(features.multimodal);
    assert!(features.modules);

    // Test individual capability methods
    assert!(runtime.supports_streaming());
    assert!(runtime.supports_multimodal());
}

#[tokio::test(flavor = "multi_thread")]
async fn test_runtime_configuration() {
    let mut config = LLMSpellConfig::default();

    // Test that configuration supports multiple engines
    assert!(config.supports_engine("lua"));
    assert!(config.supports_engine("javascript"));
    assert!(!config.supports_engine("python")); // Not configured

    // Test engine-specific configuration
    config.engines.lua.enable_debug = true;
    config.engines.javascript.strict_mode = false;

    let runtime = ScriptRuntime::new_with_lua(config).await.unwrap();
    assert_eq!(runtime.get_engine_name(), "lua");
}

#[tokio::test(flavor = "multi_thread")]
async fn test_runtime_execution_context() {
    let config = LLMSpellConfig::default();
    let runtime = ScriptRuntime::new_with_lua(config).await.unwrap();

    // Get initial context
    let context = runtime.get_execution_context();
    assert!(!context.working_directory.is_empty());

    // Update context
    let mut new_context = context;
    new_context.state = serde_json::json!({ "test": "value" });

    runtime.set_execution_context(new_context).unwrap();

    // Verify update
    let updated = runtime.get_execution_context();
    assert_eq!(updated.state, serde_json::json!({ "test": "value" }));
}

#[tokio::test(flavor = "multi_thread")]
async fn test_runtime_engine_switching_placeholder() {
    // This test demonstrates the architecture supports engine switching
    // even though JavaScript engine is not yet implemented

    let config = LLMSpellConfig::default();

    // Create with Lua
    let lua_runtime = ScriptRuntime::new_with_lua(config.clone()).await.unwrap();
    assert_eq!(lua_runtime.get_engine_name(), "lua");

    // Attempt to create with JavaScript
    let js_runtime = ScriptRuntime::new_with_javascript(config).await;

    #[cfg(feature = "javascript")]
    {
        // When JavaScript feature is enabled, it should create successfully
        assert!(js_runtime.is_ok());
        if let Ok(runtime) = js_runtime {
            assert_eq!(runtime.get_engine_name(), "javascript");
        }
    }

    #[cfg(not(feature = "javascript"))]
    {
        // When JavaScript feature is not enabled, it should fail
        assert!(js_runtime.is_err());
        if let Err(e) = js_runtime {
            let error_msg = format!("{e:?}");
            assert!(error_msg.contains("JavaScript") || error_msg.contains("not enabled"));
        }
    }
}
#[tokio::test]
async fn test_runtime_with_custom_engine_name() {
    let config = LLMSpellConfig::default();

    // Test unknown engine
    let result = ScriptRuntime::new_with_engine_name("unknown", config).await;
    assert!(result.is_err());

    if let Err(e) = result {
        match e {
            llmspell_core::error::LLMSpellError::Validation { field, .. } => {
                assert_eq!(field, Some("engine".to_string()));
            }
            _ => panic!("Expected validation error for unknown engine"),
        }
    }
}
#[tokio::test]
async fn test_available_engines() {
    let engines = EngineFactory::list_available_engines();

    // At least Lua should be available
    assert!(!engines.is_empty());

    let lua_engine = engines.iter().find(|e| e.name == "lua");
    assert!(lua_engine.is_some());

    let lua = lua_engine.unwrap();
    assert_eq!(lua.name, "lua");
    assert!(lua.features.streaming);
    assert!(lua.features.multimodal);
}
