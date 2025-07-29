// ABOUTME: Comprehensive tests for hook persistence and replay functionality
// ABOUTME: Tests storage, replay, and inspection capabilities

#[cfg(test)]
mod tests {
    use crate::context::HookContext;
    use crate::persistence::*;
    use crate::result::HookResult;
    use crate::traits::{Hook, ReplayableHook};
    use crate::types::{ComponentId, ComponentType, HookPoint};
    use anyhow::Result;
    use async_trait::async_trait;
    use std::sync::Arc;
    use std::time::{Duration, SystemTime};
    use uuid::Uuid;

    /// Mock replayable hook for testing
    struct MockReplayableHook {
        id: String,
        result: HookResult,
    }

    impl MockReplayableHook {
        fn new(id: String, result: HookResult) -> Self {
            Self { id, result }
        }
    }

    #[async_trait]
    impl Hook for MockReplayableHook {
        async fn execute(&self, _context: &mut HookContext) -> Result<HookResult> {
            Ok(self.result.clone())
        }

        fn metadata(&self) -> crate::types::HookMetadata {
            crate::types::HookMetadata {
                name: self.id.clone(),
                description: Some("Mock hook for testing".to_string()),
                priority: crate::types::Priority::NORMAL,
                language: crate::types::Language::Native,
                tags: vec!["test".to_string()],
                version: "1.0.0".to_string(),
            }
        }

        fn should_execute(&self, _context: &HookContext) -> bool {
            true
        }

        fn as_any(&self) -> &dyn std::any::Any {
            self
        }
    }

    #[async_trait]
    impl ReplayableHook for MockReplayableHook {
        fn replay_id(&self) -> String {
            self.id.clone()
        }

        fn serialize_context(&self, context: &HookContext) -> Result<Vec<u8>> {
            Ok(serde_json::to_vec(context)?)
        }

        fn deserialize_context(&self, data: &[u8]) -> Result<HookContext> {
            Ok(serde_json::from_slice(data)?)
        }
    }

    fn create_test_context() -> HookContext {
        let component_id = ComponentId::new(ComponentType::Tool, "test_tool".to_string());
        HookContext::new(HookPoint::BeforeToolExecution, component_id)
    }

    fn create_test_execution(hook_id: String, correlation_id: Uuid) -> SerializedHookExecution {
        let context = create_test_context();
        let context_bytes = serde_json::to_vec(&context).unwrap();

        SerializedHookExecution {
            hook_id,
            execution_id: Uuid::new_v4(),
            correlation_id,
            hook_context: context_bytes,
            result: serde_json::to_string(&HookResult::Continue).unwrap(),
            timestamp: SystemTime::now(),
            duration: Duration::from_millis(50),
            metadata: std::collections::HashMap::new(),
        }
    }

    #[tokio::test]
    async fn test_storage_backend_lifecycle() {
        let backend = InMemoryStorageBackend::new();
        let correlation_id = Uuid::new_v4();
        let execution = create_test_execution("test_hook".to_string(), correlation_id);
        let metadata = HookMetadata::new(
            "test_hook".to_string(),
            ComponentType::Tool,
            "test_tool".to_string(),
        );

        // Store execution
        backend
            .store_execution(&execution, &metadata)
            .await
            .unwrap();

        // Load by ID
        let loaded = backend
            .load_execution(&execution.execution_id)
            .await
            .unwrap();
        assert!(loaded.is_some());
        assert_eq!(loaded.unwrap().hook_id, "test_hook");

        // Load by correlation
        let correlated = backend
            .load_executions_by_correlation(&correlation_id)
            .await
            .unwrap();
        assert_eq!(correlated.len(), 1);

        // Check statistics
        let stats = backend.get_statistics().await.unwrap();
        assert_eq!(stats.total_executions, 1);
        assert!(stats.compression_ratio < 1.0);
    }

    #[tokio::test]
    async fn test_hook_persistence_manager() {
        // Create mock replay manager (simplified)
        struct MockReplayManager;

        #[async_trait]
        impl HookReplayManager for MockReplayManager {
            async fn persist_hook_execution(
                &self,
                _hook: &dyn ReplayableHook,
                _context: &HookContext,
                _result: &HookResult,
                _duration: Duration,
            ) -> Result<()> {
                Ok(())
            }

            async fn get_hook_executions_by_correlation(
                &self,
                _correlation_id: Uuid,
            ) -> Result<Vec<SerializedHookExecution>> {
                Ok(vec![])
            }
        }

        let replay_manager = Arc::new(MockReplayManager);
        let storage_backend = Arc::new(InMemoryStorageBackend::new());
        let persistence_manager =
            HookPersistenceManager::with_storage_backend(replay_manager, storage_backend.clone());

        // Test storage statistics
        let stats = persistence_manager.get_storage_statistics().await.unwrap();
        assert_eq!(stats.total_executions, 0);
    }

    #[tokio::test]
    async fn test_replay_engine() {
        let mut engine = HookReplayEngine::new();
        let hook = MockReplayableHook::new("test_hook".to_string(), HookResult::Continue);
        let execution = create_test_execution("test_hook".to_string(), Uuid::new_v4());
        let options = ReplayOptions::default();

        // Replay execution
        let result = engine
            .replay_execution(&hook, &execution, &options)
            .await
            .unwrap();
        assert!(matches!(result, HookResult::Continue));

        // Check statistics
        let (total, success, failed, _) = engine.get_statistics();
        assert_eq!(total, 1);
        assert_eq!(success, 1);
        assert_eq!(failed, 0);
    }

    #[tokio::test]
    async fn test_replay_manager_session() {
        let replay_manager = Arc::new(MockReplayManager);
        let storage_backend = Arc::new(InMemoryStorageBackend::new());
        let persistence_manager = Arc::new(HookPersistenceManager::with_storage_backend(
            replay_manager,
            storage_backend.clone(),
        ));

        let manager = ReplayManager::new(persistence_manager, storage_backend);

        // Register hook
        let hook = Arc::new(MockReplayableHook::new(
            "test_hook".to_string(),
            HookResult::Continue,
        ));
        manager.register_hook("test_hook".to_string(), hook);

        // Start session
        let config = ReplaySessionConfig::default();
        let session_name = manager.start_replay_session(config).await.unwrap();

        // Check session exists
        let session = manager.get_session(&session_name);
        assert!(session.is_some());

        // Add breakpoint
        let breakpoint = ReplayBreakpoint {
            condition: BreakpointCondition::HookId("test_hook".to_string()),
            action: BreakpointAction::Pause,
        };
        manager.add_breakpoint(&session_name, breakpoint).unwrap();

        // End session
        let ended_session = manager.end_session(&session_name).unwrap();
        assert_eq!(ended_session.executions_replayed, 0);
    }

    #[tokio::test]
    async fn test_hook_inspector_analysis() {
        let storage_backend = Arc::new(InMemoryStorageBackend::new());
        let inspector = HookInspector::new(storage_backend.clone());

        // Create test executions
        let correlation_id = Uuid::new_v4();
        let executions = vec![
            create_test_execution("hook1".to_string(), correlation_id),
            create_test_execution("hook2".to_string(), correlation_id),
            create_test_execution("hook1".to_string(), correlation_id),
        ];

        // Store executions
        for execution in &executions {
            let metadata = HookMetadata::new(
                execution.hook_id.clone(),
                ComponentType::Tool,
                "test_tool".to_string(),
            );
            storage_backend
                .store_execution(execution, &metadata)
                .await
                .unwrap();
        }

        // Analyze executions
        let analysis = inspector.analyze_executions(&executions).await.unwrap();
        assert_eq!(analysis.total_executions, 3);
        assert_eq!(analysis.unique_hooks.len(), 2);
        assert_eq!(analysis.execution_by_hook.get("hook1"), Some(&2));
        assert_eq!(analysis.execution_by_hook.get("hook2"), Some(&1));
    }

    #[tokio::test]
    async fn test_execution_pattern_detection() {
        let storage_backend = Arc::new(InMemoryStorageBackend::new());
        let inspector = HookInspector::new(storage_backend);

        // Create pattern: hook1 -> hook2 -> hook3 (repeated)
        let correlation_id = Uuid::new_v4();
        let mut executions = Vec::new();

        for i in 0..6 {
            let hook_id = format!("hook{}", (i % 3) + 1);
            executions.push(create_test_execution(hook_id, correlation_id));
        }

        // Detect patterns
        let patterns = inspector.detect_patterns(&executions).await.unwrap();

        // Should detect sequential pattern
        let sequential_pattern = patterns
            .iter()
            .find(|p| matches!(p.pattern_type, PatternType::Sequential));
        assert!(sequential_pattern.is_some());
    }

    #[tokio::test]
    async fn test_timeline_construction() {
        let replay_manager = Arc::new(MockReplayManager);
        let storage_backend = Arc::new(InMemoryStorageBackend::new());
        let persistence_manager = Arc::new(HookPersistenceManager::with_storage_backend(
            replay_manager,
            storage_backend.clone(),
        ));

        let manager = ReplayManager::new(persistence_manager, storage_backend.clone());

        // Register hooks
        let hook1 = Arc::new(MockReplayableHook::new(
            "hook1".to_string(),
            HookResult::Continue,
        ));
        let hook2 = Arc::new(MockReplayableHook::new(
            "hook2".to_string(),
            HookResult::Continue,
        ));

        manager.register_hook("hook1".to_string(), hook1);
        manager.register_hook("hook2".to_string(), hook2);

        // Create executions
        let correlation_id = Uuid::new_v4();
        let executions = vec![
            create_test_execution("hook1".to_string(), correlation_id),
            create_test_execution("hook2".to_string(), correlation_id),
        ];

        // Store executions
        for execution in &executions {
            let metadata = HookMetadata::new(
                execution.hook_id.clone(),
                ComponentType::Tool,
                "test_tool".to_string(),
            );
            storage_backend
                .store_execution(execution, &metadata)
                .await
                .unwrap();
        }

        // Build timeline
        let timeline = manager.build_timeline(correlation_id).await.unwrap();
        assert_eq!(timeline.entries.len(), 2);
        assert!(timeline.total_duration >= Duration::from_secs(0));
    }

    #[tokio::test]
    async fn test_replay_options_modification() {
        let mut engine = HookReplayEngine::new();
        let hook = MockReplayableHook::new("test_hook".to_string(), HookResult::Continue);
        let execution = create_test_execution("test_hook".to_string(), Uuid::new_v4());

        // Test with parameter modification
        let mut options = ReplayOptions::default();
        options.modify_parameters = true;
        options.custom_parameters = Some(serde_json::json!({"test": "value"}));
        options.dry_run = false; // Actually execute

        let result = engine
            .replay_execution(&hook, &execution, &options)
            .await
            .unwrap();
        assert!(matches!(result, HookResult::Continue));
    }

    #[tokio::test]
    async fn test_inspection_query_filtering() {
        let storage_backend = Arc::new(InMemoryStorageBackend::new());
        let inspector = HookInspector::new(storage_backend.clone());

        let correlation_id = Uuid::new_v4();

        // Create query with filters
        let query = InspectionQuery {
            correlation_id: Some(correlation_id),
            hook_ids: Some(vec!["hook1".to_string()]),
            hook_points: None,
            time_range: Some(TimeRange {
                start: SystemTime::now() - Duration::from_secs(3600),
                end: SystemTime::now(),
            }),
            component_pattern: None,
            result_type: Some(ResultTypeFilter::Continue),
            limit: Some(10),
        };

        // Query executions (will be empty since we didn't store any)
        let results = inspector.query_executions(query).await.unwrap();
        assert_eq!(results.len(), 0);
    }

    struct MockReplayManager;

    #[async_trait]
    impl HookReplayManager for MockReplayManager {
        async fn persist_hook_execution(
            &self,
            _hook: &dyn ReplayableHook,
            _context: &HookContext,
            _result: &HookResult,
            _duration: Duration,
        ) -> Result<()> {
            Ok(())
        }

        async fn get_hook_executions_by_correlation(
            &self,
            _correlation_id: Uuid,
        ) -> Result<Vec<SerializedHookExecution>> {
            Ok(vec![])
        }
    }
}
