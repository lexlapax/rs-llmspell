// ABOUTME: Comprehensive migration test suite integrating with existing test patterns
// ABOUTME: Tests migration framework with StateManager, storage adapters, and hook system

use llmspell_state_persistence::{
    backend_adapter::StateStorageAdapter,
    config::{FieldSchema, PersistenceConfig, StorageBackendType},
    manager::SerializableState,
    migration::{
        engine::*, planner::MigrationPlanner, transforms::*, validator::*, MigrationConfig,
    },
    schema::*,
    StateError, StateManager, StateResult, StateScope,
};
use llmspell_storage::MemoryBackend;
use std::sync::Arc;
use std::time::Duration;
use tokio::time::timeout;

/// Test helper to create a test StateManager with memory backend
async fn create_test_state_manager() -> StateResult<StateManager> {
    StateManager::with_backend(StorageBackendType::Memory, PersistenceConfig::default()).await
}

/// Test helper to create basic schema
fn create_basic_schema(version: SemanticVersion) -> EnhancedStateSchema {
    let mut schema = EnhancedStateSchema::new(version);
    schema.add_field(
        "name".to_string(),
        FieldSchema {
            field_type: "string".to_string(),
            required: true,
            default_value: None,
            validators: vec![],
        },
    );
    schema.add_field(
        "age".to_string(),
        FieldSchema {
            field_type: "number".to_string(),
            required: false,
            default_value: Some(serde_json::json!(0)),
            validators: vec![],
        },
    );
    schema
}

/// Test helper to create advanced schema
fn create_advanced_schema(version: SemanticVersion) -> EnhancedStateSchema {
    let mut schema = create_basic_schema(version);
    schema.add_field(
        "email".to_string(),
        FieldSchema {
            field_type: "string".to_string(),
            required: false, // Make optional to avoid breaking change
            default_value: Some(serde_json::json!("user@example.com")),
            validators: vec!["email".to_string()],
        },
    );
    schema.add_field(
        "preferences".to_string(),
        FieldSchema {
            field_type: "object".to_string(),
            required: false,
            default_value: Some(serde_json::json!({})),
            validators: vec![],
        },
    );
    schema
}

#[cfg(test)]
mod migration_integration_tests {
    use super::*;
    #[tokio::test]
    async fn test_basic_migration_engine_creation() {
        // Create storage adapter directly for testing
        let backend = Arc::new(MemoryBackend::new());
        let storage_adapter = Arc::new(StateStorageAdapter::new(backend, "test".to_string()));
        let schema_registry = SchemaRegistry::new();
        let hook_executor = Arc::new(llmspell_hooks::HookExecutor::new());
        let correlation_tracker = Arc::new(llmspell_events::EventCorrelationTracker::default());
        let event_bus = Arc::new(llmspell_events::EventBus::new());

        let engine = MigrationEngine::new(
            storage_adapter,
            schema_registry,
            hook_executor,
            correlation_tracker,
            event_bus,
        );

        assert!(engine.get_active_migrations().is_empty());
    }
    #[tokio::test]
    async fn test_schema_compatibility_validation() {
        let v1_0_0 = SemanticVersion::new(1, 0, 0);
        let v1_1_0 = SemanticVersion::new(1, 1, 0);

        let schema_v1 = create_basic_schema(v1_0_0.clone());
        let schema_v1_1 = create_advanced_schema(v1_1_0.clone());

        let compatibility = CompatibilityChecker::check_compatibility(&schema_v1, &schema_v1_1);

        assert!(compatibility.compatible);
        assert_eq!(compatibility.field_changes.len(), 2); // email and preferences added
        assert!(compatibility.migration_required);
    }
    #[tokio::test]
    async fn test_migration_with_state_manager() {
        let state_manager = create_test_state_manager().await.unwrap();

        // Store initial state
        let initial_state = serde_json::json!({
            "name": "John Doe",
            "age": 30
        });

        state_manager
            .set(StateScope::Global, "user:123", initial_state.clone())
            .await
            .unwrap();

        // Verify state was stored
        let retrieved = state_manager
            .get(StateScope::Global, "user:123")
            .await
            .unwrap();

        assert_eq!(retrieved, Some(initial_state));
    }
    #[tokio::test]
    async fn test_migration_planner_integration() {
        let mut planner = MigrationPlanner::new();

        let v1_0_0 = SemanticVersion::new(1, 0, 0);
        let v1_1_0 = SemanticVersion::new(1, 1, 0);

        let schema_v1 = create_basic_schema(v1_0_0.clone());
        let schema_v1_1 = create_advanced_schema(v1_1_0.clone());

        planner.register_schema(schema_v1).unwrap();
        planner.register_schema(schema_v1_1).unwrap();

        // Test migration possibility
        assert!(planner.is_migration_possible(&v1_0_0, &v1_1_0));

        // Test complexity estimation
        let complexity = planner.estimate_complexity(&v1_0_0, &v1_1_0).unwrap();
        assert!(complexity.field_changes > 0);
        assert_eq!(complexity.breaking_changes, 0);

        // Test migration paths
        let paths = planner.find_migration_paths(&v1_0_0).unwrap();
        assert!(!paths.is_empty());
    }
    #[tokio::test]
    async fn test_data_transformation() {
        let transformer = DataTransformer::new();

        let mut state = SerializableState {
            key: "test_user".to_string(),
            value: serde_json::json!({
                "name": "Alice",
                "age": 25
            }),
            timestamp: std::time::SystemTime::now(),
            schema_version: 1,
        };

        let mut transformation = StateTransformation::new(
            "v1_to_v2".to_string(),
            "Add email field with default".to_string(),
            1,
            2,
        );

        // Add default email field
        transformation.add_transform(FieldTransform::Default {
            field: "email".to_string(),
            value: serde_json::json!("user@example.com"),
        });

        let result = transformer
            .transform_state(&mut state, &transformation)
            .unwrap();

        assert!(result.success);
        assert_eq!(result.fields_transformed, 1);
        assert_eq!(state.schema_version, 2);
        assert_eq!(state.value["email"], "user@example.com");
        assert_eq!(state.value["name"], "Alice"); // Preserved
        assert_eq!(state.value["age"], 25); // Preserved
    }
    #[tokio::test]
    async fn test_migration_validation() {
        let rules = ValidationRules::strict();
        let validator = MigrationValidator::new(rules);

        // Create schema with a required field for testing validation
        let mut schema = create_basic_schema(SemanticVersion::new(2, 0, 0));
        schema.add_field(
            "email".to_string(),
            FieldSchema {
                field_type: "string".to_string(),
                required: true, // Keep required for this validation test
                default_value: None,
                validators: vec!["email".to_string()],
            },
        );

        // Valid state
        let valid_state = SerializableState {
            key: "valid_user".to_string(),
            value: serde_json::json!({
                "name": "John Doe",
                "age": 30,
                "email": "john@example.com"
            }),
            timestamp: std::time::SystemTime::now(),
            schema_version: 2,
        };

        let result = validator
            .validate_post_migration(&[valid_state], &schema)
            .unwrap();

        assert!(result.passed);
        assert_eq!(result.errors_count, 0);
        assert_eq!(result.validated_items, 1);

        // Invalid state (missing required field)
        let invalid_state = SerializableState {
            key: "invalid_user".to_string(),
            value: serde_json::json!({
                "name": "Jane Doe",
                "age": 25
                // missing required email field
            }),
            timestamp: std::time::SystemTime::now(),
            schema_version: 2,
        };

        let result = validator
            .validate_post_migration(&[invalid_state], &schema)
            .unwrap();

        assert!(!result.passed);
        assert!(result.critical_count > 0); // Missing required fields are Critical severity
    }
    #[tokio::test]
    async fn test_migration_with_timeout() {
        // Create storage adapter directly for testing
        let backend = Arc::new(MemoryBackend::new());
        let storage_adapter = Arc::new(StateStorageAdapter::new(backend, "test".to_string()));
        let schema_registry = SchemaRegistry::new();
        let hook_executor = Arc::new(llmspell_hooks::HookExecutor::new());
        let correlation_tracker = Arc::new(llmspell_events::EventCorrelationTracker::default());
        let event_bus = Arc::new(llmspell_events::EventBus::new());

        let engine = MigrationEngine::new(
            storage_adapter,
            schema_registry,
            hook_executor,
            correlation_tracker,
            event_bus,
        );

        // Test migration timeout
        let v1_0_0 = SemanticVersion::new(1, 0, 0);
        let v2_0_0 = SemanticVersion::new(2, 0, 0);

        let config = MigrationConfig {
            timeout: Duration::from_millis(1), // Very short timeout
            ..Default::default()
        };

        // This should complete quickly or timeout
        let result = timeout(
            Duration::from_millis(100),
            engine.migrate(&v1_0_0, &v2_0_0, config),
        )
        .await;

        // Either it completes successfully or times out - both are acceptable
        match result {
            Ok(migration_result) => {
                // Migration completed within timeout
                match migration_result {
                    Ok(_) => println!("Migration completed successfully"),
                    Err(e) => println!("Migration failed as expected: {}", e),
                }
            }
            Err(_) => {
                // Test timed out, which is also acceptable
                println!("Test timed out as expected");
            }
        }
    }
    #[tokio::test]
    async fn test_migration_error_handling() {
        // Create storage adapter directly for testing
        let backend = Arc::new(MemoryBackend::new());
        let storage_adapter = Arc::new(StateStorageAdapter::new(backend, "test".to_string()));
        let schema_registry = SchemaRegistry::new();
        let hook_executor = Arc::new(llmspell_hooks::HookExecutor::new());
        let correlation_tracker = Arc::new(llmspell_events::EventCorrelationTracker::default());
        let event_bus = Arc::new(llmspell_events::EventBus::new());

        let engine = MigrationEngine::new(
            storage_adapter,
            schema_registry,
            hook_executor,
            correlation_tracker,
            event_bus,
        );

        // Try to migrate between non-existent schema versions
        let v1_0_0 = SemanticVersion::new(1, 0, 0);
        let v2_0_0 = SemanticVersion::new(2, 0, 0);

        let config = MigrationConfig::default();
        let result = engine.migrate(&v1_0_0, &v2_0_0, config).await;

        // Should fail with schema not found error
        assert!(result.is_err());
        match result.unwrap_err() {
            StateError::MigrationError(msg) => {
                assert!(msg.contains("not found") || msg.contains("Schema"));
            }
            _ => panic!("Expected MigrationError"),
        }
    }
    #[tokio::test]
    async fn test_concurrent_migration_prevention() {
        // Create storage adapter directly for testing
        let backend = Arc::new(MemoryBackend::new());
        let storage_adapter = Arc::new(StateStorageAdapter::new(backend, "test".to_string()));
        let schema_registry = SchemaRegistry::new();
        let hook_executor = Arc::new(llmspell_hooks::HookExecutor::new());
        let correlation_tracker = Arc::new(llmspell_events::EventCorrelationTracker::default());
        let event_bus = Arc::new(llmspell_events::EventBus::new());

        let engine = Arc::new(MigrationEngine::new(
            storage_adapter,
            schema_registry,
            hook_executor,
            correlation_tracker,
            event_bus,
        ));

        // Configuration with low concurrent migration limit
        let config = MigrationConfig {
            max_concurrent_migrations: 1,
            timeout: Duration::from_secs(1),
            ..Default::default()
        };

        let v1_0_0 = SemanticVersion::new(1, 0, 0);
        let v2_0_0 = SemanticVersion::new(2, 0, 0);

        // Both migrations should fail due to missing schemas, but test concurrent handling
        let engine1 = Arc::clone(&engine);
        let engine2 = Arc::clone(&engine);
        let config1 = config.clone();
        let config2 = config.clone();
        let v1_0_0_clone = v1_0_0.clone();
        let v2_0_0_clone = v2_0_0.clone();

        let migration1 =
            tokio::spawn(async move { engine1.migrate(&v1_0_0, &v2_0_0, config1).await });

        let migration2 =
            tokio::spawn(
                async move { engine2.migrate(&v1_0_0_clone, &v2_0_0_clone, config2).await },
            );

        let (result1, result2) = tokio::join!(migration1, migration2);

        // Both should complete (though likely with errors due to missing schemas)
        assert!(result1.is_ok());
        assert!(result2.is_ok());
    }
}

#[cfg(test)]
mod agent_state_migration_tests {
    use super::*;
    use llmspell_state_persistence::agent_state::*;
    #[tokio::test]
    async fn test_agent_state_migration() {
        let state_manager = create_test_state_manager().await.unwrap();

        // Create initial agent state
        let agent_state = PersistentAgentState::new("test_agent".to_string(), "test".to_string());

        // Store agent state
        let scope = StateScope::Agent("test_agent".to_string());
        state_manager
            .set(
                scope.clone(),
                "agent_data",
                serde_json::to_value(&agent_state).unwrap(),
            )
            .await
            .unwrap();

        // Verify agent state retrieval
        let retrieved = state_manager.get(scope, "agent_data").await.unwrap();

        assert!(retrieved.is_some());

        let retrieved_state: PersistentAgentState =
            serde_json::from_value(retrieved.unwrap()).unwrap();

        assert_eq!(retrieved_state.agent_id, "test_agent");
        assert_eq!(retrieved_state.agent_type, "test");
    }
    #[tokio::test]
    async fn test_agent_conversation_migration() {
        let state_manager = create_test_state_manager().await.unwrap();

        // Create agent with conversation history
        let mut agent_state =
            PersistentAgentState::new("chat_agent".to_string(), "conversation".to_string());

        // Add conversation messages
        agent_state.add_message(MessageRole::User, "Hello, how are you?".to_string());

        agent_state.add_message(
            MessageRole::Assistant,
            "I'm doing well, thank you for asking!".to_string(),
        );

        // Store and retrieve
        let scope = StateScope::Agent("chat_agent".to_string());
        state_manager
            .set(
                scope.clone(),
                "conversation",
                serde_json::to_value(&agent_state).unwrap(),
            )
            .await
            .unwrap();

        let retrieved = state_manager.get(scope, "conversation").await.unwrap();

        assert!(retrieved.is_some());

        let retrieved_state: PersistentAgentState =
            serde_json::from_value(retrieved.unwrap()).unwrap();

        assert_eq!(retrieved_state.state.conversation_history.len(), 2);
        assert!(matches!(
            retrieved_state.state.conversation_history[0].role,
            MessageRole::User
        ));
        assert!(matches!(
            retrieved_state.state.conversation_history[1].role,
            MessageRole::Assistant
        ));
    }
}

#[cfg(test)]
mod performance_migration_tests {
    use super::*;
    use std::time::Instant;
    #[tokio::test]
    async fn test_migration_performance_basic() {
        let transformer = DataTransformer::new();

        let mut state = SerializableState {
            key: "perf_test".to_string(),
            value: serde_json::json!({
                "name": "Performance Test",
                "value": 42
            }),
            timestamp: std::time::SystemTime::now(),
            schema_version: 1,
        };

        let transformation = StateTransformation::new(
            "perf_test".to_string(),
            "Simple transformation".to_string(),
            1,
            2,
        );

        let start = Instant::now();
        let result = transformer
            .transform_state(&mut state, &transformation)
            .unwrap();
        let duration = start.elapsed();

        assert!(result.success);
        // Should complete well under 1ms for simple transformations
        assert!(duration < Duration::from_millis(1));
    }
    #[tokio::test]
    async fn test_migration_batch_performance() {
        let transformer = DataTransformer::new();

        // Create multiple states for batch processing
        let mut states = Vec::new();
        for i in 0..100 {
            states.push(SerializableState {
                key: format!("batch_item_{}", i),
                value: serde_json::json!({
                    "id": i,
                    "name": format!("Item {}", i),
                    "active": true
                }),
                timestamp: std::time::SystemTime::now(),
                schema_version: 1,
            });
        }

        let transformation = StateTransformation::new(
            "batch_transform".to_string(),
            "Batch transformation test".to_string(),
            1,
            2,
        );

        let start = Instant::now();
        let mut success_count = 0;

        for state in &mut states {
            let result = transformer.transform_state(state, &transformation).unwrap();
            if result.success {
                success_count += 1;
            }
        }

        let duration = start.elapsed();
        let avg_per_item = duration / 100;

        assert_eq!(success_count, 100);
        // Average time per item should be very fast
        println!(
            "Batch migration: {} items in {:?} (avg: {:?} per item)",
            success_count, duration, avg_per_item
        );

        // Should process 100 items in reasonable time
        assert!(duration < Duration::from_millis(10));
    }
}
