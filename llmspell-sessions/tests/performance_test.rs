//! Performance tests for session operations

#[cfg(test)]
mod tests {
    use llmspell_events::bus::EventBus;
    use llmspell_hooks::{HookExecutor, HookRegistry};
    use llmspell_sessions::{
        config::SessionManagerConfig, types::CreateSessionOptions, SessionManager,
    };
    use llmspell_state_persistence::StateManager;
    use llmspell_storage::MemoryBackend;
    use std::sync::Arc;
    use std::time::Instant;

    async fn create_test_manager() -> SessionManager {
        let state_manager = Arc::new(StateManager::new().await.unwrap());
        let storage_backend = Arc::new(MemoryBackend::new());
        let hook_registry = Arc::new(HookRegistry::new());
        let hook_executor = Arc::new(HookExecutor::new());
        let event_bus = Arc::new(EventBus::new());
        let config = SessionManagerConfig::default();

        SessionManager::new(
            state_manager,
            storage_backend,
            hook_registry,
            hook_executor,
            &event_bus,
            config,
        )
        .unwrap()
    }

    #[tokio::test]
    async fn test_session_create_performance() {
        let manager = create_test_manager().await;
        let options = CreateSessionOptions::default();

        // Warm up
        let _ = manager.create_session(options.clone()).await.unwrap();

        // Measure create_session
        let start = Instant::now();
        let session_id = manager.create_session(options).await.unwrap();
        let elapsed = start.elapsed();

        println!("Session creation took: {:?}", elapsed);
        assert!(
            elapsed.as_millis() < 50,
            "Session creation took {:?}, expected <50ms",
            elapsed
        );

        // Clean up
        let _ = manager.delete_session(&session_id).await;
    }

    #[tokio::test]
    async fn test_session_save_performance() {
        let manager = create_test_manager().await;
        let options = CreateSessionOptions::default();
        let session_id = manager.create_session(options).await.unwrap();
        let session = manager.get_session(&session_id).await.unwrap();

        // Warm up
        manager.save_session(&session).await.unwrap();

        // Measure save_session
        let start = Instant::now();
        manager.save_session(&session).await.unwrap();
        let elapsed = start.elapsed();

        println!("Session save took: {:?}", elapsed);
        assert!(
            elapsed.as_millis() < 50,
            "Session save took {:?}, expected <50ms",
            elapsed
        );

        // Clean up
        let _ = manager.delete_session(&session_id).await;
    }

    #[tokio::test]
    async fn test_session_load_performance() {
        let manager = create_test_manager().await;

        // Create multiple sessions for testing
        let options = CreateSessionOptions::default();
        let session_id1 = manager.create_session(options.clone()).await.unwrap();
        let session1 = manager.get_session(&session_id1).await.unwrap();
        manager.save_session(&session1).await.unwrap();

        let session_id2 = manager.create_session(options).await.unwrap();
        let session2 = manager.get_session(&session_id2).await.unwrap();
        manager.save_session(&session2).await.unwrap();

        // Complete sessions to remove from active cache
        manager.complete_session(&session_id1).await.unwrap();
        manager.complete_session(&session_id2).await.unwrap();

        // Warm up
        let _ = manager.load_session(&session_id1).await.unwrap();

        // Measure load_session
        let start = Instant::now();
        let _ = manager.load_session(&session_id2).await.unwrap();
        let elapsed = start.elapsed();

        println!("Session load took: {:?}", elapsed);
        assert!(
            elapsed.as_millis() < 50,
            "Session load took {:?}, expected <50ms",
            elapsed
        );

        // Clean up
        let _ = manager.delete_session(&session_id1).await;
        let _ = manager.delete_session(&session_id2).await;
    }

    #[tokio::test]
    async fn test_hook_overhead() {
        // Test with hooks enabled
        let manager_with_hooks = create_test_manager().await;

        // Test without hooks
        let state_manager = Arc::new(StateManager::new().await.unwrap());
        let storage_backend = Arc::new(MemoryBackend::new());
        let hook_registry = Arc::new(HookRegistry::new());
        let hook_executor = Arc::new(HookExecutor::new());
        let event_bus = Arc::new(EventBus::new());
        let mut config = SessionManagerConfig::default();
        config.hook_config.enable_lifecycle_hooks = false;

        let manager_no_hooks = SessionManager::new(
            state_manager,
            storage_backend,
            hook_registry,
            hook_executor,
            &event_bus,
            config,
        )
        .unwrap();

        let options = CreateSessionOptions::default();

        // Warm up both
        let _ = manager_with_hooks
            .create_session(options.clone())
            .await
            .unwrap();
        let _ = manager_no_hooks
            .create_session(options.clone())
            .await
            .unwrap();

        // Measure with hooks
        let start = Instant::now();
        let session_id_hooks = manager_with_hooks
            .create_session(options.clone())
            .await
            .unwrap();
        let with_hooks_time = start.elapsed();

        // Measure without hooks
        let start = Instant::now();
        let session_id_no_hooks = manager_no_hooks.create_session(options).await.unwrap();
        let without_hooks_time = start.elapsed();

        let overhead_micros =
            with_hooks_time.as_micros() as i64 - without_hooks_time.as_micros() as i64;
        let overhead_percent = ((with_hooks_time.as_nanos() as f64
            - without_hooks_time.as_nanos() as f64)
            / without_hooks_time.as_nanos() as f64)
            * 100.0;

        println!("With hooks: {:?}", with_hooks_time);
        println!("Without hooks: {:?}", without_hooks_time);
        println!(
            "Hook overhead: {:.2}% ({} µs)",
            overhead_percent, overhead_micros
        );

        // Check absolute overhead is less than 1ms (1000µs)
        // This is more meaningful than percentage for fast operations
        assert!(
            overhead_micros < 1000,
            "Hook overhead is {} µs, expected <1000µs (1ms)",
            overhead_micros
        );

        // Also ensure total time is still under 50ms target
        assert!(
            with_hooks_time.as_millis() < 50,
            "Total time with hooks is {:?}, expected <50ms",
            with_hooks_time
        );

        // Clean up
        let _ = manager_with_hooks.delete_session(&session_id_hooks).await;
        let _ = manager_no_hooks.delete_session(&session_id_no_hooks).await;
    }
}
