// ABOUTME: Simple test to verify async hook processing works
// ABOUTME: Focuses on the core async hook functionality

use llmspell_hooks::{
    ComponentId, ComponentType, Hook, HookContext, HookExecutor, HookPoint, HookResult,
};
use llmspell_state_persistence::{
    performance::{AsyncHookProcessor, HookEvent, HookEventType},
    StateManager,
};
use llmspell_state_traits::StateScope;
use serde_json::json;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use uuid::Uuid;

/// Test hook that counts executions
struct CountingHook {
    count: Arc<AtomicU64>,
}

#[async_trait::async_trait]
impl Hook for CountingHook {
    async fn execute(&self, _context: &mut HookContext) -> anyhow::Result<HookResult> {
        self.count.fetch_add(1, Ordering::Relaxed);
        Ok(HookResult::Continue)
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

#[tokio::test]
async fn test_async_hook_processor_directly() {
    // Create hook executor
    let hook_executor = Arc::new(HookExecutor::new());

    // Create async processor
    let mut processor = AsyncHookProcessor::new(hook_executor);

    // Start processor
    processor.start().unwrap();

    // Create counting hook
    let count = Arc::new(AtomicU64::new(0));
    let hook = Arc::new(CountingHook {
        count: count.clone(),
    }) as Arc<dyn Hook>;

    // Queue 10 events
    for i in 0..10 {
        let context = HookContext::new(
            HookPoint::Custom(format!("test_{}", i)),
            ComponentId::new(
                ComponentType::Custom("test".to_string()),
                "test".to_string(),
            ),
        );

        let event = HookEvent {
            hook_type: HookEventType::AfterStateChange,
            context,
            hooks: vec![hook.clone()],
            correlation_id: Uuid::new_v4(),
            timestamp: Instant::now(),
        };

        processor.queue_hook_event(event).unwrap();
    }

    // Check initial queue depth
    assert_eq!(processor.queue_depth(), 10);

    // Wait for processing
    processor
        .wait_for_drain(Duration::from_secs(2))
        .await
        .unwrap();

    // Verify all hooks executed
    assert_eq!(count.load(Ordering::Relaxed), 10);
    assert_eq!(processor.queue_depth(), 0);

    // Check stats
    let stats = processor.stats();
    assert_eq!(stats.events_queued, 10);
    assert_eq!(stats.events_processed, 10);
    assert_eq!(stats.events_failed, 0);

    // Stop processor
    processor.stop().await.unwrap();
}

#[tokio::test]
async fn test_state_manager_with_only_after_hooks() {
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

    // Create counting hook
    let count = Arc::new(AtomicU64::new(0));
    let hook = Arc::new(CountingHook {
        count: count.clone(),
    }) as Arc<dyn Hook>;

    // Start async hooks
    manager.start_async_hooks().await.unwrap();

    // Only register after hooks (these should be async)
    {
        let mut after_hooks = manager.after_state_change_hooks.write();
        after_hooks.push(hook);
    }

    // Perform state operations
    let start = Instant::now();

    for i in 0..10 {
        manager
            .set(
                StateScope::Global,
                &format!("key_{}", i),
                json!({ "value": i }),
            )
            .await
            .unwrap();
    }

    let operation_time = start.elapsed();

    // Operations should be fast since after hooks are async
    println!("Operation time: {:?}", operation_time);

    // Count should still be 0 immediately
    let immediate_count = count.load(Ordering::Relaxed);
    println!("Immediate hook count: {}", immediate_count);

    // Wait for hooks to process
    manager
        .wait_for_hooks(Duration::from_secs(2))
        .await
        .unwrap();

    // Now count should be 10
    let final_count = count.load(Ordering::Relaxed);
    println!("Final hook count: {}", final_count);
    assert_eq!(final_count, 10);

    // Get stats
    if let Some(stats) = manager.hook_processor_stats() {
        println!("Hook processor stats: {:?}", stats);
        assert!(stats.events_processed > 0);
    }

    manager.stop_async_hooks().await.unwrap();
}
