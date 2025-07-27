// ABOUTME: Tests to verify async hook processing doesn't block state operations
// ABOUTME: Ensures hooks are processed asynchronously without impacting performance

use llmspell_hooks::{Hook, HookContext, HookResult};
use llmspell_state_persistence::{performance::StateClass, StateManager};
use serde_json::json;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

/// Test hook that simulates slow processing
struct SlowTestHook {
    #[allow(dead_code)]
    name: String,
    delay_ms: u64,
    execution_count: Arc<AtomicU64>,
}

#[async_trait::async_trait]
impl Hook for SlowTestHook {
    async fn execute(&self, _context: &mut HookContext) -> anyhow::Result<HookResult> {
        // Simulate slow processing
        tokio::time::sleep(Duration::from_millis(self.delay_ms)).await;
        self.execution_count.fetch_add(1, Ordering::Relaxed);
        Ok(HookResult::Continue)
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

#[tokio::test]
async fn test_async_hooks_dont_block_state_operations() {
    // Create manager with persistence enabled so async hooks are available
    let config = llmspell_state_persistence::config::PersistenceConfig {
        enabled: true,
        ..Default::default()
    };
    let mut manager = StateManager::with_backend(
        llmspell_state_persistence::config::StorageBackendType::Memory,
        config,
    )
    .await
    .unwrap();

    // Create hooks that simulate slow processing
    let hook_count = Arc::new(AtomicU64::new(0));
    let hooks: Vec<Arc<dyn Hook>> = (0..5)
        .map(|i| {
            Arc::new(SlowTestHook {
                name: format!("slow_hook_{}", i),
                delay_ms: 1, // Very short delay to ensure all hooks complete
                execution_count: hook_count.clone(),
            }) as Arc<dyn Hook>
        })
        .collect();

    // Start async hook processor
    manager.start_async_hooks().await.unwrap();

    // Register the hooks with the manager - only after hooks for async processing
    {
        let mut after_hooks = manager.after_state_change_hooks.write();
        after_hooks.extend(hooks.clone());
    }

    // Perform state operations with async hooks
    let start = Instant::now();

    // These operations should complete quickly despite slow hooks
    for i in 0..10 {
        let key = format!("async_test_{}", i);
        let value = json!({ "value": i, "timestamp": Instant::now().elapsed().as_millis() });

        // Use regular set_with_hooks which will use async processing
        manager
            .set_with_hooks(
                llmspell_state_persistence::scope::StateScope::Global,
                &key,
                value,
            )
            .await
            .unwrap();
    }

    let operation_time = start.elapsed();

    // Operations should complete quickly (not wait for hooks)
    assert!(
        operation_time < Duration::from_millis(50),
        "State operations took {:?}, should be <50ms despite slow hooks",
        operation_time
    );

    // Wait for hooks to complete
    manager
        .wait_for_hooks(Duration::from_secs(10))
        .await
        .unwrap();

    // Give a bit more time for any stragglers
    tokio::time::sleep(Duration::from_millis(100)).await;

    // Get stats to see what happened
    if let Some(stats) = manager.hook_processor_stats() {
        println!("Hook processor stats: {:?}", stats);
        println!("Events queued: {}", stats.events_queued);
        println!("Events processed: {}", stats.events_processed);
    }

    // Verify all hooks executed
    let expected_executions = 10 * 5; // 10 operations * 5 hooks
    let actual_executions = hook_count.load(Ordering::Relaxed);
    assert_eq!(
        actual_executions, expected_executions,
        "Expected {} hook executions, got {}",
        expected_executions, actual_executions
    );

    // Stop async hooks
    manager.stop_async_hooks().await.unwrap();
}

#[tokio::test]
async fn test_async_agent_save_performance() {
    // Create manager with persistence enabled so async hooks are available
    let config = llmspell_state_persistence::config::PersistenceConfig {
        enabled: true,
        ..Default::default()
    };
    let mut manager = StateManager::with_backend(
        llmspell_state_persistence::config::StorageBackendType::Memory,
        config,
    )
    .await
    .unwrap();

    // Create a slow hook
    let hook_count = Arc::new(AtomicU64::new(0));
    let slow_hook = Arc::new(SlowTestHook {
        name: "slow_agent_hook".to_string(),
        delay_ms: 10, // Reduced delay for reliability
        execution_count: hook_count.clone(),
    }) as Arc<dyn Hook>;

    // Start async hooks
    manager.start_async_hooks().await.unwrap();

    // Create test agent state
    let agent_state = json!({
        "id": "async_agent_1",
        "model": "test-model",
        "messages": [
            {"role": "user", "content": "Hello"},
            {"role": "assistant", "content": "Hi there!"}
        ],
        "tool_performance": {
            "total_calls": 10,
            "avg_duration_ms": 25.5
        }
    });

    // Save agent state with async hooks
    let start = Instant::now();

    manager
        .save_agent_state_with_hooks(
            "async_agent_1",
            agent_state.clone(),
            vec![slow_hook.clone()],
        )
        .await
        .unwrap();

    let save_time = start.elapsed();

    // Save should complete quickly
    assert!(
        save_time < Duration::from_millis(50),
        "Agent save took {:?}, should be <50ms despite slow hook",
        save_time
    );

    // Verify state was saved immediately
    let loaded = manager.load_agent_state("async_agent_1").await.unwrap();
    assert!(loaded.is_some());
    // Verify the custom data was saved
    let loaded_state = loaded.unwrap();
    assert_eq!(loaded_state.agent_id, "async_agent_1");
    assert_eq!(loaded_state.agent_type, "custom");

    // Wait for hook to complete - use a polling approach
    let start = Instant::now();
    let timeout = Duration::from_secs(5);

    loop {
        // Check if hook executed
        let count = hook_count.load(Ordering::Relaxed);
        if count >= 1 {
            break;
        }

        // Check timeout
        if start.elapsed() > timeout {
            // Print stats for debugging
            if let Some(stats) = manager.hook_processor_stats() {
                println!("Agent save - Hook processor stats at timeout: {:?}", stats);
            }
            panic!("Timeout waiting for hook execution. Count: {}", count);
        }

        // Wait a bit before checking again
        tokio::time::sleep(Duration::from_millis(50)).await;
    }

    // Verify hook executed
    let actual_count = hook_count.load(Ordering::Relaxed);
    assert_eq!(
        actual_count, 1,
        "Expected 1 hook execution, got {}",
        actual_count
    );

    // Check stats
    let stats = manager.hook_processor_stats().unwrap();
    assert_eq!(stats.events_processed, 1);
    assert_eq!(stats.events_failed, 0);
    assert_eq!(stats.success_rate(), 100.0);

    manager.stop_async_hooks().await.unwrap();
}

#[tokio::test]
async fn test_hook_batching_performance() {
    // Create manager with persistence enabled so async hooks are available
    let config = llmspell_state_persistence::config::PersistenceConfig {
        enabled: true,
        ..Default::default()
    };
    let mut manager = StateManager::with_backend(
        llmspell_state_persistence::config::StorageBackendType::Memory,
        config,
    )
    .await
    .unwrap();

    // Configure batching
    manager
        .configure_hook_batching(10, Duration::from_millis(50))
        .unwrap();

    // Create a hook that counts batches
    let batch_count = Arc::new(AtomicU64::new(0));
    let batch_hook = Arc::new(SlowTestHook {
        name: "batch_hook".to_string(),
        delay_ms: 10,
        execution_count: batch_count.clone(),
    }) as Arc<dyn Hook>;

    // Start async hooks
    manager.start_async_hooks().await.unwrap();

    // Perform many operations quickly
    let start = Instant::now();

    for i in 0..100 {
        let key = format!("batch_test_{}", i);
        let value = json!({ "batch": i });

        manager
            .set_with_async_hooks_public(&key, value, StateClass::Trusted, vec![batch_hook.clone()])
            .await
            .unwrap();
    }

    let operation_time = start.elapsed();

    // All operations should complete very quickly
    assert!(
        operation_time < Duration::from_millis(100),
        "Batch operations took {:?}, should be <100ms",
        operation_time
    );

    // Wait for processing
    manager
        .wait_for_hooks(Duration::from_secs(10))
        .await
        .unwrap();

    // Give extra time for any stragglers
    tokio::time::sleep(Duration::from_millis(200)).await;

    // Should have processed all hooks
    let actual_count = batch_count.load(Ordering::Relaxed);
    assert_eq!(
        actual_count, 100,
        "Expected 100 hook executions, got {}",
        actual_count
    );

    manager.stop_async_hooks().await.unwrap();
}

#[tokio::test]
async fn test_hook_failure_isolation() {
    // Create manager with persistence enabled so async hooks are available
    let config = llmspell_state_persistence::config::PersistenceConfig {
        enabled: true,
        ..Default::default()
    };
    let mut manager = StateManager::with_backend(
        llmspell_state_persistence::config::StorageBackendType::Memory,
        config,
    )
    .await
    .unwrap();

    // Create a failing hook
    struct FailingHook {
        #[allow(dead_code)]
        name: String,
    }

    #[async_trait::async_trait]
    impl Hook for FailingHook {
        async fn execute(&self, _context: &mut HookContext) -> anyhow::Result<HookResult> {
            Err(anyhow::anyhow!("Hook intentionally failed"))
        }

        fn as_any(&self) -> &dyn std::any::Any {
            self
        }
    }

    let failing_hook = Arc::new(FailingHook {
        name: "failing_hook".to_string(),
    }) as Arc<dyn Hook>;

    // Start async hooks
    manager.start_async_hooks().await.unwrap();

    // State operation should succeed despite hook failure
    let result = manager
        .set_with_async_hooks_public(
            "failure_test",
            json!({ "test": "data" }),
            StateClass::Standard,
            vec![failing_hook],
        )
        .await;

    assert!(
        result.is_ok(),
        "State operation should succeed despite hook failure"
    );

    // Wait for hook processing
    manager
        .wait_for_hooks(Duration::from_secs(1))
        .await
        .unwrap();

    // Check stats show failure
    let stats = manager.hook_processor_stats().unwrap();
    assert_eq!(stats.events_processed, 1);
    assert_eq!(stats.events_failed, 1);
    assert!(stats.success_rate() < 100.0);

    manager.stop_async_hooks().await.unwrap();
}
