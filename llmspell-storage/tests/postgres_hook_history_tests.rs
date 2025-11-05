//! Integration tests for PostgreSQL Hook History Storage (Phase 13b.12.2)
//!
//! Tests PostgresHookHistoryStorage backend:
//! - Store and load executions
//! - Query by correlation_id, hook_id, hook_type
//! - Archive old executions
//! - Get storage statistics

#![cfg(feature = "postgres")]

use chrono::{Duration, Utc};
use llmspell_storage::backends::postgres::{
    HookHistoryStats, PostgresBackend, PostgresConfig, PostgresHookHistoryStorage,
    SerializedHookExecution,
};
use serde_json::json;
use std::sync::Arc;
use uuid::Uuid;

// Application connection (enforces RLS)
const APP_CONNECTION_STRING: &str =
    "postgresql://llmspell_app:llmspell_app_pass@localhost:5432/llmspell_dev";

#[tokio::test]
async fn test_hook_history_store_and_load() {
    let config = PostgresConfig::new(APP_CONNECTION_STRING);
    let backend = Arc::new(PostgresBackend::new(config).await.unwrap());
    let storage = PostgresHookHistoryStorage::new(backend.clone());

    let tenant_id = format!("test_hook_store_{}", Uuid::new_v4());
    backend.set_tenant_context(&tenant_id).await.unwrap();

    // Create test execution
    let execution_id = Uuid::new_v4();
    let correlation_id = Uuid::new_v4();
    let hook_context = vec![1u8, 2u8, 3u8, 4u8, 5u8]; // Simulated compressed data

    let execution = SerializedHookExecution {
        execution_id,
        hook_id: "test_rate_limiter".to_string(),
        hook_type: "rate_limit".to_string(),
        correlation_id,
        hook_context: hook_context.clone(),
        result_data: json!({
            "success": true,
            "message": "Rate limit check passed"
        }),
        timestamp: Utc::now(),
        duration_ms: 25,
        triggering_component: "Agent".to_string(),
        component_id: "agent-123".to_string(),
        modified_operation: false,
        tags: vec!["production".to_string(), "critical".to_string()],
        retention_priority: 10,
        context_size: hook_context.len() as i32,
        contains_sensitive_data: false,
        metadata: json!({"env": "prod"}),
    };

    // Store execution
    storage.store_execution(&execution).await.unwrap();

    // Load execution
    let loaded = storage
        .load_execution(&execution_id)
        .await
        .unwrap()
        .expect("Execution should exist");

    assert_eq!(loaded.execution_id, execution_id);
    assert_eq!(loaded.hook_id, "test_rate_limiter");
    assert_eq!(loaded.hook_type, "rate_limit");
    assert_eq!(loaded.correlation_id, correlation_id);
    assert_eq!(loaded.hook_context, hook_context);
    assert_eq!(loaded.duration_ms, 25);
    assert_eq!(loaded.tags, vec!["production", "critical"]);
    assert_eq!(loaded.retention_priority, 10);

    // Cleanup
    let client = backend.get_client().await.unwrap();
    client
        .execute(
            "DELETE FROM llmspell.hook_history WHERE execution_id = $1",
            &[&execution_id],
        )
        .await
        .unwrap();
}

#[tokio::test]
async fn test_hook_history_correlation_query() {
    let config = PostgresConfig::new(APP_CONNECTION_STRING);
    let backend = Arc::new(PostgresBackend::new(config).await.unwrap());
    let storage = PostgresHookHistoryStorage::new(backend.clone());

    let tenant_id = format!("test_hook_correlation_{}", Uuid::new_v4());
    backend.set_tenant_context(&tenant_id).await.unwrap();

    let correlation_id = Uuid::new_v4();
    let mut execution_ids = Vec::new();

    // Store 3 executions with same correlation_id
    for i in 0..3 {
        let execution_id = Uuid::new_v4();
        execution_ids.push(execution_id);

        let execution = SerializedHookExecution {
            execution_id,
            hook_id: format!("hook_{}", i),
            hook_type: "test_hook".to_string(),
            correlation_id,
            hook_context: vec![i as u8],
            result_data: json!({"step": i}),
            timestamp: Utc::now() + Duration::milliseconds(i as i64 * 100),
            duration_ms: 10 + i as i32,
            triggering_component: "TestComponent".to_string(),
            component_id: format!("component_{}", i),
            modified_operation: false,
            tags: vec![],
            retention_priority: 0,
            context_size: 1,
            contains_sensitive_data: false,
            metadata: json!({}),
        };

        storage.store_execution(&execution).await.unwrap();
    }

    // Query by correlation_id
    let executions = storage
        .get_executions_by_correlation_id(&correlation_id)
        .await
        .unwrap();

    assert_eq!(executions.len(), 3);
    // Should be ordered by timestamp ascending
    assert_eq!(executions[0].hook_id, "hook_0");
    assert_eq!(executions[1].hook_id, "hook_1");
    assert_eq!(executions[2].hook_id, "hook_2");

    // Cleanup
    let client = backend.get_client().await.unwrap();
    for execution_id in execution_ids {
        client
            .execute(
                "DELETE FROM llmspell.hook_history WHERE execution_id = $1",
                &[&execution_id],
            )
            .await
            .unwrap();
    }
}

#[tokio::test]
async fn test_hook_history_hook_id_query() {
    let config = PostgresConfig::new(APP_CONNECTION_STRING);
    let backend = Arc::new(PostgresBackend::new(config).await.unwrap());
    let storage = PostgresHookHistoryStorage::new(backend.clone());

    let tenant_id = format!("test_hook_id_query_{}", Uuid::new_v4());
    backend.set_tenant_context(&tenant_id).await.unwrap();

    let hook_id = "rate_limiter_v2";
    let mut execution_ids = Vec::new();

    // Store 5 executions for same hook_id
    for i in 0..5 {
        let execution_id = Uuid::new_v4();
        execution_ids.push(execution_id);

        let execution = SerializedHookExecution {
            execution_id,
            hook_id: hook_id.to_string(),
            hook_type: "rate_limit".to_string(),
            correlation_id: Uuid::new_v4(),
            hook_context: vec![i as u8],
            result_data: json!({"iteration": i}),
            timestamp: Utc::now() + Duration::seconds(i as i64),
            duration_ms: 15 + i as i32,
            triggering_component: "Agent".to_string(),
            component_id: "agent-1".to_string(),
            modified_operation: false,
            tags: vec![],
            retention_priority: 0,
            context_size: 1,
            contains_sensitive_data: false,
            metadata: json!({}),
        };

        storage.store_execution(&execution).await.unwrap();
    }

    // Query all executions for hook_id
    let all_executions = storage
        .get_executions_by_hook_id(hook_id, None)
        .await
        .unwrap();
    assert_eq!(all_executions.len(), 5);
    // Should be ordered by timestamp DESC (newest first)
    assert_eq!(all_executions[0].result_data["iteration"], 4);
    assert_eq!(all_executions[4].result_data["iteration"], 0);

    // Query with limit
    let limited_executions = storage
        .get_executions_by_hook_id(hook_id, Some(2))
        .await
        .unwrap();
    assert_eq!(limited_executions.len(), 2);
    assert_eq!(limited_executions[0].result_data["iteration"], 4);
    assert_eq!(limited_executions[1].result_data["iteration"], 3);

    // Cleanup
    let client = backend.get_client().await.unwrap();
    for execution_id in execution_ids {
        client
            .execute(
                "DELETE FROM llmspell.hook_history WHERE execution_id = $1",
                &[&execution_id],
            )
            .await
            .unwrap();
    }
}

#[tokio::test]
async fn test_hook_history_type_query() {
    let config = PostgresConfig::new(APP_CONNECTION_STRING);
    let backend = Arc::new(PostgresBackend::new(config).await.unwrap());
    let storage = PostgresHookHistoryStorage::new(backend.clone());

    let tenant_id = format!("test_hook_type_{}", Uuid::new_v4());
    backend.set_tenant_context(&tenant_id).await.unwrap();

    let hook_type = "authentication";
    let mut execution_ids = Vec::new();

    // Store 3 executions of same type
    for i in 0..3 {
        let execution_id = Uuid::new_v4();
        execution_ids.push(execution_id);

        let execution = SerializedHookExecution {
            execution_id,
            hook_id: format!("auth_hook_{}", i),
            hook_type: hook_type.to_string(),
            correlation_id: Uuid::new_v4(),
            hook_context: vec![i as u8],
            result_data: json!({"user": format!("user_{}", i)}),
            timestamp: Utc::now() + Duration::seconds(i as i64),
            duration_ms: 20,
            triggering_component: "API".to_string(),
            component_id: "api-1".to_string(),
            modified_operation: false,
            tags: vec![],
            retention_priority: 0,
            context_size: 1,
            contains_sensitive_data: true,
            metadata: json!({}),
        };

        storage.store_execution(&execution).await.unwrap();
    }

    // Query by hook type
    let executions = storage
        .get_executions_by_type(hook_type, None)
        .await
        .unwrap();
    assert_eq!(executions.len(), 3);
    assert!(executions.iter().all(|e| e.hook_type == hook_type));
    assert!(executions.iter().all(|e| e.contains_sensitive_data == true));

    // Cleanup
    let client = backend.get_client().await.unwrap();
    for execution_id in execution_ids {
        client
            .execute(
                "DELETE FROM llmspell.hook_history WHERE execution_id = $1",
                &[&execution_id],
            )
            .await
            .unwrap();
    }
}

#[tokio::test]
async fn test_hook_history_archive_executions() {
    let config = PostgresConfig::new(APP_CONNECTION_STRING);
    let backend = Arc::new(PostgresBackend::new(config).await.unwrap());
    let storage = PostgresHookHistoryStorage::new(backend.clone());

    let tenant_id = format!("test_hook_archive_{}", Uuid::new_v4());
    backend.set_tenant_context(&tenant_id).await.unwrap();

    let now = Utc::now();
    let old_timestamp = now - Duration::days(100);
    let mut execution_ids = Vec::new();

    // Store 2 old executions with low priority
    for i in 0..2 {
        let execution_id = Uuid::new_v4();
        execution_ids.push(execution_id);

        let execution = SerializedHookExecution {
            execution_id,
            hook_id: "old_hook".to_string(),
            hook_type: "test".to_string(),
            correlation_id: Uuid::new_v4(),
            hook_context: vec![i as u8],
            result_data: json!({"old": true}),
            timestamp: old_timestamp,
            duration_ms: 10,
            triggering_component: "Test".to_string(),
            component_id: "test-1".to_string(),
            modified_operation: false,
            tags: vec![],
            retention_priority: 0, // Low priority
            context_size: 1,
            contains_sensitive_data: false,
            metadata: json!({}),
        };

        storage.store_execution(&execution).await.unwrap();
    }

    // Store 1 old execution with high priority (should be preserved)
    let high_priority_id = Uuid::new_v4();
    execution_ids.push(high_priority_id);

    let high_priority_execution = SerializedHookExecution {
        execution_id: high_priority_id,
        hook_id: "important_hook".to_string(),
        hook_type: "critical".to_string(),
        correlation_id: Uuid::new_v4(),
        hook_context: vec![99u8],
        result_data: json!({"important": true}),
        timestamp: old_timestamp,
        duration_ms: 10,
        triggering_component: "CriticalSystem".to_string(),
        component_id: "critical-1".to_string(),
        modified_operation: false,
        tags: vec!["critical".to_string()],
        retention_priority: 100, // High priority - should be preserved
        context_size: 1,
        contains_sensitive_data: false,
        metadata: json!({}),
    };

    storage
        .store_execution(&high_priority_execution)
        .await
        .unwrap();

    // Archive old executions with priority <= 50
    let before_date = now - Duration::days(90);
    let deleted_count = storage.archive_executions(before_date, 50).await.unwrap();

    // Should have deleted the 2 low-priority executions, preserved the high-priority one
    assert_eq!(deleted_count, 2);

    // Verify high-priority execution still exists
    let preserved = storage.load_execution(&high_priority_id).await.unwrap();
    assert!(preserved.is_some());
    assert_eq!(preserved.unwrap().retention_priority, 100);

    // Cleanup remaining execution
    let client = backend.get_client().await.unwrap();
    client
        .execute(
            "DELETE FROM llmspell.hook_history WHERE execution_id = $1",
            &[&high_priority_id],
        )
        .await
        .unwrap();
}

#[tokio::test]
async fn test_hook_history_statistics() {
    let config = PostgresConfig::new(APP_CONNECTION_STRING);
    let backend = Arc::new(PostgresBackend::new(config).await.unwrap());
    let storage = PostgresHookHistoryStorage::new(backend.clone());

    let tenant_id = format!("test_hook_stats_{}", Uuid::new_v4());
    backend.set_tenant_context(&tenant_id).await.unwrap();

    let mut execution_ids = Vec::new();

    // Store 5 executions with different hooks and types
    let hooks = vec![
        ("rate_limiter", "rate_limit", 10),
        ("rate_limiter", "rate_limit", 15),
        ("auth_checker", "authentication", 20),
        ("auth_checker", "authentication", 25),
        ("validator", "validation", 30),
    ];

    for (hook_id, hook_type, duration_ms) in hooks {
        let execution_id = Uuid::new_v4();
        execution_ids.push(execution_id);

        let execution = SerializedHookExecution {
            execution_id,
            hook_id: hook_id.to_string(),
            hook_type: hook_type.to_string(),
            correlation_id: Uuid::new_v4(),
            hook_context: vec![1u8],
            result_data: json!({}),
            timestamp: Utc::now(),
            duration_ms,
            triggering_component: "Test".to_string(),
            component_id: "test-1".to_string(),
            modified_operation: false,
            tags: vec![],
            retention_priority: 0,
            context_size: 1,
            contains_sensitive_data: false,
            metadata: json!({}),
        };

        storage.store_execution(&execution).await.unwrap();
    }

    // Get statistics
    let stats: HookHistoryStats = storage.get_statistics().await.unwrap();

    assert_eq!(stats.total_executions, 5);
    assert!(stats.storage_size_bytes > 0);
    assert!(stats.oldest_execution.is_some());
    assert!(stats.newest_execution.is_some());

    // Check executions by hook
    assert_eq!(stats.executions_by_hook.get("rate_limiter"), Some(&2));
    assert_eq!(stats.executions_by_hook.get("auth_checker"), Some(&2));
    assert_eq!(stats.executions_by_hook.get("validator"), Some(&1));

    // Check executions by type
    assert_eq!(stats.executions_by_type.get("rate_limit"), Some(&2));
    assert_eq!(stats.executions_by_type.get("authentication"), Some(&2));
    assert_eq!(stats.executions_by_type.get("validation"), Some(&1));

    // Check average duration (10+15+20+25+30)/5 = 20
    assert_eq!(stats.avg_duration_ms, 20.0);

    // Cleanup
    let client = backend.get_client().await.unwrap();
    for execution_id in execution_ids {
        client
            .execute(
                "DELETE FROM llmspell.hook_history WHERE execution_id = $1",
                &[&execution_id],
            )
            .await
            .unwrap();
    }
}
