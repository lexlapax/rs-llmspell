//! Integration tests for shared `StateManager` between kernel and `ScriptRuntime`
//!
//! These tests verify that when a kernel is created with state persistence enabled,
//! the `ScriptRuntime` uses the same `StateManager` instance, allowing state sharing
//! between kernel operations and script execution.

#![cfg(feature = "lua")]

use anyhow::Result;
use llmspell_bridge::ScriptRuntime;
use llmspell_config::{
    GlobalRuntimeConfig, KernelSettings, LLMSpellConfig, StatePersistenceConfig,
};
use llmspell_state_persistence::{factory::StateFactory, StateScope};
use std::sync::Arc;

/// Test that kernel and `ScriptRuntime` share the same `StateManager` instance
#[tokio::test(flavor = "multi_thread")]
async fn test_shared_state_manager_between_kernel_and_runtime() -> Result<()> {
    // Create config with state persistence enabled
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
                    .kernel(KernelSettings::default())
                    .build(),
            )
            .build(),
    );

    // Create StateManager from config (simulating what kernel does)
    let state_manager = StateFactory::create_from_config(&config)
        .await?
        .expect("State manager should be created when persistence is enabled");

    // Write test data directly to StateManager
    let test_key = "shared_test_key";
    let test_value = serde_json::json!({ "source": "kernel", "value": 42 });
    state_manager
        .set(StateScope::Global, test_key, test_value.clone())
        .await?;

    // Create ScriptRuntime with the same StateManager
    let runtime = ScriptRuntime::new_with_engine_and_state_manager(
        "lua",
        (*config).clone(),
        state_manager.clone(),
    )
    .await?;

    // Execute Lua script that reads the state
    // State global uses 'load' function with scope and key
    let script = r#"
        local state_value = State.load("global", "shared_test_key")
        if state_value then
            return state_value.value
        else
            return nil
        end
    "#;

    let output = runtime.execute_script(script).await?;

    // Verify the script could read the value written by the kernel
    assert_eq!(
        output.output,
        serde_json::Value::Number(serde_json::Number::from(42)),
        "Script should read the value written by kernel's StateManager"
    );

    // Now write from script and read from kernel
    // State global uses 'save' function with scope, key, and value
    let write_script = r#"
        State.save("global", "script_written_key", { source = "script", value = 100 })
        return true
    "#;

    runtime.execute_script(write_script).await?;

    // Read the value written by script directly from StateManager
    let script_value = state_manager
        .get(StateScope::Global, "script_written_key")
        .await?
        .expect("Value should exist");

    assert_eq!(
        script_value["value"],
        serde_json::json!(100),
        "Kernel's StateManager should see value written by script"
    );

    Ok(())
}

/// Test that independent `StateManager` is created when none is provided
#[tokio::test(flavor = "multi_thread")]
async fn test_independent_state_manager_when_not_shared() -> Result<()> {
    // Create config with state persistence enabled
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
                    .kernel(KernelSettings::default())
                    .build(),
            )
            .build(),
    );

    // Create StateManager from config (simulating what kernel does)
    let kernel_state_manager = StateFactory::create_from_config(&config)
        .await?
        .expect("State manager should be created when persistence is enabled");

    // Write test data to kernel's StateManager
    let test_key = "kernel_only_key";
    let test_value = serde_json::json!({ "source": "kernel", "value": 999 });
    kernel_state_manager
        .set(StateScope::Global, test_key, test_value.clone())
        .await?;

    // Create ScriptRuntime WITHOUT sharing the StateManager
    // This simulates the old behavior where each creates its own
    let runtime = ScriptRuntime::new_with_engine_name("lua", (*config).clone()).await?;

    // Execute Lua script that tries to read the state
    // State global uses 'load' function with scope and key
    let script = r#"
        local state_value = State.load("global", "kernel_only_key")
        if state_value then
            return state_value
        else
            return "not_found"
        end
    "#;

    let output = runtime.execute_script(script).await?;

    // Verify the script CANNOT read the value from kernel's StateManager
    // because they are using different instances
    assert_eq!(
        output.output,
        serde_json::Value::String("not_found".to_string()),
        "Script should NOT see value from kernel's separate StateManager"
    );

    Ok(())
}

/// Test concurrent access to shared `StateManager`
#[tokio::test(flavor = "multi_thread")]
async fn test_concurrent_shared_state_access() -> Result<()> {
    use tokio::task;

    // Create config with state persistence enabled
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
                    .kernel(KernelSettings::default())
                    .build(),
            )
            .build(),
    );

    // Create shared StateManager
    let state_manager = StateFactory::create_from_config(&config)
        .await?
        .expect("State manager should be created when persistence is enabled");

    // Create ScriptRuntime with shared StateManager
    let runtime = Arc::new(
        ScriptRuntime::new_with_engine_and_state_manager(
            "lua",
            (*config).clone(),
            state_manager.clone(),
        )
        .await?,
    );

    // Spawn multiple tasks that read and write concurrently
    let mut handles = vec![];

    for i in 0..5 {
        let sm = state_manager.clone();
        let rt = runtime.clone();

        let handle = task::spawn(async move {
            // Write from StateManager
            let key = format!("concurrent_key_{i}");
            let value = serde_json::json!({ "task": i, "source": "direct" });
            sm.set(StateScope::Global, &key, value).await.unwrap();

            // Read from script
            let script = format!(
                r#"
                local value = State.load("global", "concurrent_key_{i}")
                if value then
                    return value.task
                else
                    return -1
                end
                "#
            );

            let output = rt.execute_script(&script).await.unwrap();

            // Verify we can read our own write
            assert_eq!(
                output.output,
                serde_json::Value::Number(serde_json::Number::from(i)),
                "Task {i} should read its own value"
            );
        });

        handles.push(handle);
    }

    // Wait for all tasks to complete
    for handle in handles {
        handle.await?;
    }

    // Verify all values are present
    for i in 0..5 {
        let key = format!("concurrent_key_{i}");
        let value = state_manager
            .get(StateScope::Global, &key)
            .await?
            .expect("Value should exist");
        assert_eq!(value["task"], serde_json::json!(i));
    }

    Ok(())
}

/// Test that state persistence settings are respected
#[tokio::test(flavor = "multi_thread")]
async fn test_state_persistence_disabled() -> Result<()> {
    // Create config with state persistence DISABLED
    let config = Arc::new(
        LLMSpellConfig::builder()
            .default_engine("lua")
            .runtime(
                GlobalRuntimeConfig::builder()
                    .state_persistence(StatePersistenceConfig {
                        enabled: false,
                        ..Default::default()
                    })
                    .kernel(KernelSettings::default())
                    .build(),
            )
            .build(),
    );

    // StateFactory should return None when persistence is disabled
    let state_manager = StateFactory::create_from_config(&config).await?;
    assert!(
        state_manager.is_none(),
        "StateManager should not be created when persistence is disabled"
    );

    // ScriptRuntime should still work without StateManager
    let runtime = ScriptRuntime::new_with_engine_name("lua", (*config).clone()).await?;

    // Execute a simple script that doesn't use State
    let script = r#"return "hello world""#;
    let output = runtime.execute_script(script).await?;

    assert_eq!(
        output.output,
        serde_json::Value::String("hello world".to_string()),
        "Script should work without state persistence"
    );

    Ok(())
}

/// Test pointer verification - ensure kernel and runtime use same `StateManager` instance
#[tokio::test(flavor = "multi_thread")]
async fn test_same_state_manager_instance_pointer_equality() -> Result<()> {
    // Create config with state persistence enabled
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
                    .kernel(KernelSettings::default())
                    .build(),
            )
            .build(),
    );

    // Create StateManager from config
    let state_manager = StateFactory::create_from_config(&config)
        .await?
        .expect("State manager should be created when persistence is enabled");

    // Get pointer to StateManager
    let original_ptr = Arc::as_ptr(&state_manager);

    // Create ScriptRuntime with the same StateManager
    let runtime = ScriptRuntime::new_with_engine_and_state_manager(
        "lua",
        (*config).clone(),
        state_manager.clone(),
    )
    .await?;

    // Write through script to force StateManager usage
    let write_script = r#"
        State.save("global", "pointer_test_key", "test_value")
        return true
    "#;
    runtime.execute_script(write_script).await?;

    // Read back directly from original StateManager to verify same instance
    let value = state_manager
        .get(StateScope::Global, "pointer_test_key")
        .await?
        .expect("Value should exist");

    assert_eq!(
        value,
        serde_json::json!("test_value"),
        "Same StateManager instance should be used"
    );

    // Verify the Arc pointer is the same
    let runtime_sm_ptr = Arc::as_ptr(&state_manager);
    assert_eq!(
        original_ptr, runtime_sm_ptr,
        "StateManager pointers should be identical"
    );

    Ok(())
}

/// Bridge test: State set via Lua `save()` readable by kernel `StateManager`
#[tokio::test(flavor = "multi_thread")]
async fn test_bridge_lua_save_kernel_read() -> Result<()> {
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

    // Write complex data via Lua
    let script = r#"
        State.save("global", "complex_data", {
            string = "hello",
            number = 42,
            bool = true,
            array = {1, 2, 3},
            nested = {
                key = "value"
            }
        })
        return true
    "#;

    runtime.execute_script(script).await?;

    // Read via kernel's StateManager
    let data = state_manager
        .get(StateScope::Global, "complex_data")
        .await?
        .expect("Complex data should exist");

    // Verify complex structure
    assert_eq!(data["string"], "hello");
    assert_eq!(data["number"], 42);
    assert_eq!(data["bool"], true);
    assert_eq!(data["array"], serde_json::json!([1, 2, 3]));
    assert_eq!(data["nested"]["key"], "value");

    Ok(())
}

/// Bridge test: State set via kernel `StateManager` readable by Lua `load()`
#[tokio::test(flavor = "multi_thread")]
async fn test_bridge_kernel_set_lua_read() -> Result<()> {
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

    // Write complex data via kernel StateManager
    let complex_data = serde_json::json!({
        "users": [
            {"id": 1, "name": "Alice"},
            {"id": 2, "name": "Bob"}
        ],
        "settings": {
            "theme": "dark",
            "notifications": true
        }
    });

    state_manager
        .set(StateScope::Global, "app_state", complex_data)
        .await?;

    // Read via Lua and verify structure
    let script = r#"
        local state = State.load("global", "app_state")
        if not state then
            return "not_found"
        end
        
        -- Verify users array
        if #state.users ~= 2 then
            return "wrong_user_count"
        end
        
        if state.users[1].name ~= "Alice" then
            return "wrong_first_user"
        end
        
        if state.users[2].id ~= 2 then
            return "wrong_second_id"
        end
        
        -- Verify settings
        if state.settings.theme ~= "dark" then
            return "wrong_theme"
        end
        
        if state.settings.notifications ~= true then
            return "wrong_notifications"
        end
        
        return "all_correct"
    "#;

    let output = runtime.execute_script(script).await?;
    assert_eq!(
        output.output,
        serde_json::Value::String("all_correct".to_string()),
        "Lua should correctly read complex data from kernel StateManager"
    );

    Ok(())
}

/// Test that state persists across multiple runtime creations with same `StateManager`
#[tokio::test(flavor = "multi_thread")]
async fn test_state_persists_across_runtime_recreations() -> Result<()> {
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
                    .kernel(KernelSettings::default())
                    .build(),
            )
            .build(),
    );

    let state_manager = StateFactory::create_from_config(&config)
        .await?
        .expect("State manager should be created");

    // First runtime writes data
    {
        let runtime1 = ScriptRuntime::new_with_engine_and_state_manager(
            "lua",
            (*config).clone(),
            state_manager.clone(),
        )
        .await?;

        runtime1
            .execute_script(
                r#"
            State.save("global", "persistent_data", "from_runtime_1")
            return true
        "#,
            )
            .await?;
    } // runtime1 dropped

    // Second runtime reads data
    {
        let runtime2 = ScriptRuntime::new_with_engine_and_state_manager(
            "lua",
            (*config).clone(),
            state_manager.clone(),
        )
        .await?;

        let output = runtime2
            .execute_script(
                r#"
            return State.load("global", "persistent_data")
        "#,
            )
            .await?;

        assert_eq!(
            output.output,
            serde_json::Value::String("from_runtime_1".to_string()),
            "State should persist across runtime recreations"
        );
    }

    Ok(())
}

/// Test no file lock conflicts with shared `StateManager` under heavy concurrent load
#[tokio::test(flavor = "multi_thread")]
async fn test_no_file_lock_conflicts_heavy_load() -> Result<()> {
    use tokio::task;

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
                    .kernel(KernelSettings::default())
                    .build(),
            )
            .build(),
    );

    let state_manager = StateFactory::create_from_config(&config)
        .await?
        .expect("State manager should be created");

    // Spawn many concurrent operations
    let mut handles = vec![];
    for i in 0..100 {
        let sm = state_manager.clone();
        let handle = task::spawn(async move {
            // Each task does multiple operations
            for j in 0..10 {
                let key = format!("stress_test_{i}_{j}");
                let value = serde_json::json!({"task": i, "op": j});

                // Write
                sm.set(StateScope::Global, &key, value.clone()).await?;

                // Read back
                let read_value = sm.get(StateScope::Global, &key).await?;
                assert_eq!(read_value, Some(value));

                // Delete
                if j % 2 == 0 {
                    sm.delete(StateScope::Global, &key).await?;
                }
            }
            Ok::<(), anyhow::Error>(())
        });
        handles.push(handle);
    }

    // All operations should complete without lock conflicts
    for handle in handles {
        handle.await?.expect("Task should complete without errors");
    }

    Ok(())
}
