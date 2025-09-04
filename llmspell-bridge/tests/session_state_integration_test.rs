//! Integration tests for Session operations with shared `StateManager`
//! Verifies that sessions created in kernel are visible in `ScriptRuntime` and vice versa

#![cfg(feature = "lua")]

use anyhow::Result;
use llmspell_bridge::ScriptRuntime;
use llmspell_config::{
    GlobalRuntimeConfig, KernelSettings, LLMSpellConfig, StatePersistenceConfig,
};
use llmspell_state_persistence::{factory::StateFactory, StateScope};
use std::sync::Arc;

/// Test that data written through kernel's `StateManager` is visible in `ScriptRuntime`
#[tokio::test(flavor = "multi_thread")]
async fn test_kernel_state_visible_in_runtime() -> Result<()> {
    // Create config with state persistence and sessions enabled
    let config = Arc::new(
        LLMSpellConfig::builder()
            .default_engine("lua")
            .runtime(
                GlobalRuntimeConfig::builder()
                    .state_persistence(StatePersistenceConfig {
                        enabled: true,
                        backend_type: "memory".to_string(),
                        ..Default::default()
                    })
                    .sessions(llmspell_config::SessionConfig {
                        enabled: true,
                        ..Default::default()
                    })
                    .kernel(KernelSettings::default())
                    .build(),
            )
            .build(),
    );

    // Create shared StateManager
    let state_manager = StateFactory::create_from_config(&config)
        .await?
        .expect("State manager should be created");

    // Write data directly through the shared StateManager (as kernel would)
    let test_key = "test_kernel_data";
    let test_data = serde_json::json!({
        "source": "kernel",
        "user": "test-user",
        "timestamp": "2024-01-01T00:00:00Z",
        "metadata": {
            "kernel": "created"
        }
    });

    state_manager
        .set(StateScope::Global, test_key, test_data.clone())
        .await?;

    // Create ScriptRuntime with the same StateManager
    let runtime = ScriptRuntime::new_with_engine_and_state_manager(
        "lua",
        (*config).clone(),
        state_manager.clone(),
    )
    .await?;

    // Read the same data through ScriptRuntime's State global
    let script = r#"
        -- Read data from global state
        local test_data = State.load("global", "test_kernel_data")
        if test_data then
            return {
                found = true,
                source = test_data.source,
                user = test_data.user,
                metadata_kernel = test_data.metadata and test_data.metadata.kernel
            }
        else
            return {found = false}
        end
    "#;

    let output = runtime.execute_script(script).await?;

    // Verify data is visible
    assert_eq!(output.output["found"], true, "Data should be found");
    assert_eq!(output.output["source"], "kernel");
    assert_eq!(output.output["user"], "test-user");
    assert_eq!(output.output["metadata_kernel"], "created");

    Ok(())
}

/// Test that data written through `ScriptRuntime` is visible in kernel's `StateManager`
#[tokio::test(flavor = "multi_thread")]
async fn test_runtime_state_visible_in_kernel() -> Result<()> {
    let config = Arc::new(
        LLMSpellConfig::builder()
            .default_engine("lua")
            .runtime(
                GlobalRuntimeConfig::builder()
                    .state_persistence(StatePersistenceConfig {
                        enabled: true,
                        backend_type: "memory".to_string(),
                        ..Default::default()
                    })
                    .sessions(llmspell_config::SessionConfig {
                        enabled: true,
                        ..Default::default()
                    })
                    .kernel(KernelSettings::default())
                    .build(),
            )
            .build(),
    );

    let state_manager = StateFactory::create_from_config(&config)
        .await?
        .expect("State manager should be created");

    let runtime = ScriptRuntime::new_with_engine_and_state_manager(
        "lua",
        (*config).clone(),
        state_manager.clone(),
    )
    .await?;

    // Write data through ScriptRuntime
    let script = r#"
        -- Create test data structure
        local test_data = {
            id = "runtime-123",
            created_by = "script-user",
            source = "runtime",
            metadata = {
                source = "runtime",
                timestamp = os.date()
            }
        }
        
        -- Store it in global state
        State.save("global", "test_runtime_data", test_data)
        return test_data.id
    "#;

    let output = runtime.execute_script(script).await?;
    assert_eq!(output.output, serde_json::json!("runtime-123"));

    // Read data directly from kernel's StateManager
    let runtime_data = state_manager
        .get(StateScope::Global, "test_runtime_data")
        .await?
        .expect("Data should exist in state");

    // Verify data written by runtime is visible
    assert_eq!(runtime_data["id"], "runtime-123");
    assert_eq!(runtime_data["created_by"], "script-user");
    assert_eq!(runtime_data["source"], "runtime");
    assert_eq!(runtime_data["metadata"]["source"], "runtime");

    Ok(())
}

/// Test that `StateManager` instance is the same (pointer verification)
#[tokio::test(flavor = "multi_thread")]
async fn test_state_manager_same_pointer() -> Result<()> {
    let config = Arc::new(
        LLMSpellConfig::builder()
            .default_engine("lua")
            .runtime(
                GlobalRuntimeConfig::builder()
                    .state_persistence(StatePersistenceConfig {
                        enabled: true,
                        backend_type: "memory".to_string(),
                        ..Default::default()
                    })
                    .sessions(llmspell_config::SessionConfig {
                        enabled: true,
                        ..Default::default()
                    })
                    .kernel(KernelSettings::default())
                    .build(),
            )
            .build(),
    );

    // Create shared StateManager
    let state_manager = StateFactory::create_from_config(&config)
        .await?
        .expect("State manager should be created");

    let original_ptr = Arc::as_ptr(&state_manager);

    // Write through original StateManager
    state_manager
        .set(
            StateScope::Global,
            "pointer_test",
            serde_json::json!({"test": "data"}),
        )
        .await?;

    // Create a ScriptRuntime with the same StateManager
    let runtime = ScriptRuntime::new_with_engine_and_state_manager(
        "lua",
        (*config).clone(),
        state_manager.clone(),
    )
    .await?;

    // Read through ScriptRuntime to verify same instance
    let script = r#"
        local data = State.load("global", "pointer_test")
        if data then
            return {found = true, test = data.test}
        else
            return {found = false}
        end
    "#;

    let output = runtime.execute_script(script).await?;
    assert_eq!(output.output["found"], true, "Data should be found");
    assert_eq!(output.output["test"], "data");

    // Verify pointer equality
    let session_sm_ptr = Arc::as_ptr(&state_manager);
    assert_eq!(
        original_ptr, session_sm_ptr,
        "StateManager pointers should be identical"
    );

    Ok(())
}

/// Test that complex data structures are shared via `StateManager`
#[tokio::test(flavor = "multi_thread")]
async fn test_complex_data_via_shared_state_manager() -> Result<()> {
    let config = Arc::new(
        LLMSpellConfig::builder()
            .default_engine("lua")
            .runtime(
                GlobalRuntimeConfig::builder()
                    .state_persistence(StatePersistenceConfig {
                        enabled: true,
                        backend_type: "memory".to_string(),
                        ..Default::default()
                    })
                    .sessions(llmspell_config::SessionConfig {
                        enabled: true,
                        ..Default::default()
                    })
                    .kernel(KernelSettings::default())
                    .build(),
            )
            .build(),
    );

    let state_manager = StateFactory::create_from_config(&config)
        .await?
        .expect("State manager should be created");

    let runtime = ScriptRuntime::new_with_engine_and_state_manager(
        "lua",
        (*config).clone(),
        state_manager.clone(),
    )
    .await?;

    // Store complex data through ScriptRuntime
    let script = r#"
        -- Store complex nested data
        local complex_data = {
            id = "complex-001",
            type = "document",
            content = "This is test content",
            metadata = {
                created_by = "runtime",
                format = "text",
                nested = {
                    level2 = {
                        value = 42
                    }
                }
            },
            tags = {"test", "complex", "nested"}
        }
        
        State.save("global", "complex_data", complex_data)
        return complex_data.id
    "#;

    let output = runtime.execute_script(script).await?;
    assert_eq!(output.output, serde_json::json!("complex-001"));

    // Read complex data directly from StateManager
    let complex_data = state_manager
        .get(StateScope::Global, "complex_data")
        .await?
        .expect("Complex data should exist");

    // Verify complex structure
    assert_eq!(complex_data["id"], "complex-001");
    assert_eq!(complex_data["type"], "document");
    assert_eq!(complex_data["content"], "This is test content");
    assert_eq!(complex_data["metadata"]["created_by"], "runtime");
    assert_eq!(complex_data["metadata"]["format"], "text");
    assert_eq!(complex_data["metadata"]["nested"]["level2"]["value"], 42);
    assert_eq!(complex_data["tags"][0], "test");
    assert_eq!(complex_data["tags"][1], "complex");
    assert_eq!(complex_data["tags"][2], "nested");

    Ok(())
}
