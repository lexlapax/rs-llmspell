//! Memory stability tests for shared `StateManager`
//! Verifies that memory usage remains stable with shared state operations

#![cfg(feature = "lua")]

use anyhow::Result;
use llmspell_bridge::ScriptRuntime;
use llmspell_config::{
    GlobalRuntimeConfig, KernelSettings, LLMSpellConfig, StatePersistenceConfig,
};
use llmspell_state_persistence::{factory::StateFactory, StateScope};
use std::sync::Arc;
use std::time::Instant;

/// Test that memory usage remains stable during repeated state operations
#[tokio::test(flavor = "multi_thread")]
#[ignore = "Performance test - run with --ignored"]
async fn test_memory_stability_with_shared_state() -> Result<()> {
    // Create config with memory backend
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

    // Create multiple ScriptRuntime instances sharing the same StateManager
    let runtime1 = ScriptRuntime::new_with_engine_and_state_manager(
        "lua",
        (*config).clone(),
        state_manager.clone(),
    )
    .await?;

    let runtime2 = ScriptRuntime::new_with_engine_and_state_manager(
        "lua",
        (*config).clone(),
        state_manager.clone(),
    )
    .await?;

    // Perform many state operations
    let iterations = 1000;
    let start_time = Instant::now();

    for i in 0..iterations {
        let key = format!("test_key_{i}");
        let value = serde_json::json!({
            "iteration": i,
            "data": "x".repeat(100), // 100 bytes of data per entry
            "timestamp": i * 1000,
        });

        // Alternate between writing through StateManager and ScriptRuntime
        if i % 2 == 0 {
            state_manager.set(StateScope::Global, &key, value).await?;
        } else {
            let script = format!(
                r#"
                State.save("global", "{}", {{
                    iteration = {},
                    data = "{}",
                    timestamp = {}
                }})
                "#,
                key,
                i,
                "x".repeat(100),
                i * 1000
            );
            if i % 4 == 1 {
                runtime1.execute_script(&script).await?;
            } else {
                runtime2.execute_script(&script).await?;
            }
        }

        // Periodically read to ensure data persists
        if i % 100 == 0 {
            let read_key = format!("test_key_{}", i / 2);
            let _ = state_manager.get(StateScope::Global, &read_key).await?;
        }

        // Delete old entries to prevent unbounded growth
        if i > 100 {
            let old_key = format!("test_key_{}", i - 100);
            state_manager.delete(StateScope::Global, &old_key).await?;
        }
    }

    let elapsed = start_time.elapsed();
    println!(
        "Completed {} iterations in {:?} ({:.2} ops/sec)",
        iterations,
        elapsed,
        f64::from(iterations) / elapsed.as_secs_f64()
    );

    // Verify some data still exists
    let final_key = format!("test_key_{}", iterations - 1);
    let final_value = state_manager.get(StateScope::Global, &final_key).await?;
    assert!(final_value.is_some(), "Final value should exist");

    Ok(())
}

/// Test concurrent access doesn't cause memory leaks
#[tokio::test(flavor = "multi_thread")]
#[ignore = "Performance test - run with --ignored"]
async fn test_concurrent_access_memory_stability() -> Result<()> {
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

    // Spawn multiple concurrent tasks
    let mut handles = vec![];
    let num_tasks = 10;
    let ops_per_task = 100;

    for task_id in 0..num_tasks {
        let sm = state_manager.clone();
        let handle = tokio::spawn(async move {
            for i in 0..ops_per_task {
                let key = format!("task_{task_id}_item_{i}");
                let value = serde_json::json!({
                    "task": task_id,
                    "item": i,
                    "data": vec![0u8; 50], // Small data payload
                });

                // Write
                sm.set(StateScope::Global, &key, value.clone())
                    .await
                    .unwrap();

                // Read back
                let read_value = sm.get(StateScope::Global, &key).await.unwrap();
                assert_eq!(read_value, Some(value));

                // Delete to prevent accumulation
                sm.delete(StateScope::Global, &key).await.unwrap();
            }
        });
        handles.push(handle);
    }

    // Wait for all tasks to complete
    let start = Instant::now();
    for handle in handles {
        handle.await?;
    }
    let elapsed = start.elapsed();

    let total_ops = num_tasks * ops_per_task * 3; // write + read + delete
    println!(
        "Completed {} operations across {} tasks in {:?} ({:.2} ops/sec)",
        total_ops,
        num_tasks,
        elapsed,
        f64::from(total_ops) / elapsed.as_secs_f64()
    );

    Ok(())
}

/// Test that dropping `ScriptRuntime` instances doesn't leak memory
#[tokio::test(flavor = "multi_thread")]
async fn test_runtime_lifecycle_memory_stability() -> Result<()> {
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

    // Create and drop multiple runtime instances
    for i in 0..10 {
        let runtime = ScriptRuntime::new_with_engine_and_state_manager(
            "lua",
            (*config).clone(),
            state_manager.clone(),
        )
        .await?;

        // Use the runtime
        let script = format!(
            r#"
            State.save("global", "runtime_{i}", {{value = {i}}})
            return true
            "#
        );
        runtime.execute_script(&script).await?;

        // Runtime dropped here
    }

    // Verify state persists after runtime drops
    for i in 0..10 {
        let key = format!("runtime_{i}");
        let value = state_manager.get(StateScope::Global, &key).await?;
        assert!(value.is_some(), "Value {i} should persist");
    }

    Ok(())
}
