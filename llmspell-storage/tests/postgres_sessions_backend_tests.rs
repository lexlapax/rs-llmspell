//! Tests for PostgreSQL session backend (Phase 13b.9.2)
//!
//! Verifies:
//! - Session routing (session:{uuid} → sessions table, session:{uuid}:* → kv_store)
//! - CRUD operations (get, set, delete, exists, list)
//! - Batch operations (get_batch, set_batch)
//! - Tenant isolation (RLS enforcement)
//! - Field extraction (status, artifact_count, timestamps, expires_at)
//! - UUID detection and parsing

#![cfg(feature = "postgres")]

use llmspell_storage::traits::StorageBackend;
use llmspell_storage::{PostgresBackend, PostgresConfig};
use std::collections::HashMap;
use tokio::sync::OnceCell;
use uuid::Uuid;

const SUPERUSER_CONNECTION_STRING: &str =
    "postgresql://llmspell:llmspell_dev_pass@localhost:5432/llmspell_dev";

const APP_CONNECTION_STRING: &str =
    "postgresql://llmspell_app:llmspell_dev_pass@localhost:5432/llmspell_dev";

static MIGRATION_INIT: OnceCell<()> = OnceCell::const_new();

/// Ensure migrations run once before all tests
async fn ensure_migrations_run_once() {
    MIGRATION_INIT
        .get_or_init(|| async {
            let config = PostgresConfig::new(SUPERUSER_CONNECTION_STRING);
            let backend = PostgresBackend::new(config)
                .await
                .expect("Failed to create backend for migration init");

            backend
                .run_migrations()
                .await
                .expect("Failed to run migrations during test initialization");
        })
        .await;
}

fn unique_tenant_id(prefix: &str) -> String {
    format!("{}-{}", prefix, Uuid::new_v4())
}

/// Create a SessionSnapshot-like JSON structure for testing
fn create_session_snapshot(
    session_id: &Uuid,
    status: &str,
    artifact_count: usize,
    retention_days: Option<i64>,
) -> Vec<u8> {
    let now = chrono::Utc::now();
    let snapshot = serde_json::json!({
        "metadata": {
            "id": session_id.to_string(),
            "status": status,
            "created_at": now.to_rfc3339(),
            "updated_at": now.to_rfc3339(),
            "started_at": now.to_rfc3339(),
            "ended_at": null,
            "created_by": "test-user",
            "name": "test-session",
            "description": "Test session for backend tests",
            "artifact_count": artifact_count,
            "total_artifact_size": 0,
            "operation_count": 0,
            "tags": ["test"],
            "parent_session_id": null,
            "custom_metadata": {}
        },
        "config": {
            "max_duration_secs": 86400,
            "auto_save_interval_secs": 300,
            "max_artifacts": 1000,
            "auto_collect_artifacts": true,
            "enable_replay": true,
            "retention_days": retention_days,
            "metadata": {},
            "resource_limits": {
                "max_memory_bytes": 1073741824_i64,
                "max_cpu_seconds": 3600,
                "max_operations": 100000,
                "max_storage_bytes": 10737418240_i64
            },
            "hook_config": {
                "on_start": true,
                "on_end": true,
                "on_suspend": true,
                "on_resume": true,
                "on_artifact_create": true,
                "timeout_ms": 5000
            }
        },
        "state": {},
        "artifact_ids": [],
        "snapshot_at": now.to_rfc3339(),
        "version": 1
    });

    serde_json::to_vec(&snapshot).unwrap()
}

#[tokio::test]
async fn test_session_routing_primary_key() {
    ensure_migrations_run_once().await;

    let config = PostgresConfig::new(APP_CONNECTION_STRING);
    let backend = PostgresBackend::new(config).await.unwrap();
    let tenant_id = unique_tenant_id("routing-primary");

    backend.set_tenant_context(&tenant_id).await.unwrap();

    let session_id = Uuid::new_v4();
    let key = format!("session:{}", session_id);
    let value = create_session_snapshot(&session_id, "Active", 0, Some(30));

    // Set session (should go to sessions table)
    backend.set(&key, value.clone()).await.unwrap();

    // Verify it exists
    assert!(backend.exists(&key).await.unwrap());

    // Get session back
    let retrieved = backend.get(&key).await.unwrap();
    assert!(retrieved.is_some());

    // Verify stored in sessions table (not kv_store)
    let client = backend.get_client().await.unwrap();
    let row = client
        .query_opt(
            "SELECT session_id FROM llmspell.sessions
             WHERE tenant_id = $1 AND session_id = $2",
            &[&tenant_id, &session_id],
        )
        .await
        .unwrap();
    assert!(row.is_some(), "Session should be in sessions table");

    // Verify NOT in kv_store
    let kv_row = client
        .query_opt(
            "SELECT key FROM llmspell.kv_store
             WHERE tenant_id = $1 AND key = $2",
            &[&tenant_id, &key],
        )
        .await
        .unwrap();
    assert!(kv_row.is_none(), "Session should NOT be in kv_store");
}

#[tokio::test]
async fn test_session_routing_state_items() {
    ensure_migrations_run_once().await;

    let config = PostgresConfig::new(APP_CONNECTION_STRING);
    let backend = PostgresBackend::new(config).await.unwrap();
    let tenant_id = unique_tenant_id("routing-state");

    backend.set_tenant_context(&tenant_id).await.unwrap();

    let session_id = Uuid::new_v4();
    let state_key = format!("session:{}:my_state_key", session_id);
    let state_value = b"test state data".to_vec();

    // Set state item (should go to kv_store)
    backend.set(&state_key, state_value.clone()).await.unwrap();

    // Verify it exists
    assert!(backend.exists(&state_key).await.unwrap());

    // Get state item back
    let retrieved = backend.get(&state_key).await.unwrap();
    assert_eq!(retrieved.unwrap(), state_value);

    // Verify stored in kv_store (not sessions table)
    let client = backend.get_client().await.unwrap();
    let kv_row = client
        .query_opt(
            "SELECT key FROM llmspell.kv_store
             WHERE tenant_id = $1 AND key = $2",
            &[&tenant_id, &state_key],
        )
        .await
        .unwrap();
    assert!(kv_row.is_some(), "State item should be in kv_store");

    // Verify NOT in sessions table (won't be valid UUID in second part)
    let sessions_count: i64 = client
        .query_one(
            "SELECT COUNT(*) FROM llmspell.sessions
             WHERE tenant_id = $1",
            &[&tenant_id],
        )
        .await
        .unwrap()
        .get(0);
    assert_eq!(sessions_count, 0, "No sessions should be in sessions table");
}

#[tokio::test]
async fn test_session_crud_operations() {
    ensure_migrations_run_once().await;

    let config = PostgresConfig::new(APP_CONNECTION_STRING);
    let backend = PostgresBackend::new(config).await.unwrap();
    let tenant_id = unique_tenant_id("crud");

    backend.set_tenant_context(&tenant_id).await.unwrap();

    let session_id = Uuid::new_v4();
    let key = format!("session:{}", session_id);

    // Test exists (should be false initially)
    assert!(!backend.exists(&key).await.unwrap());

    // Test set
    let value = create_session_snapshot(&session_id, "Active", 5, Some(30));
    backend.set(&key, value.clone()).await.unwrap();

    // Test exists (should be true now)
    assert!(backend.exists(&key).await.unwrap());

    // Test get
    let retrieved = backend.get(&key).await.unwrap();
    assert!(retrieved.is_some());
    let retrieved_json: serde_json::Value = serde_json::from_slice(&retrieved.unwrap()).unwrap();
    assert_eq!(
        retrieved_json["metadata"]["status"].as_str().unwrap(),
        "Active"
    );

    // Test update (set again with different status)
    let updated_value = create_session_snapshot(&session_id, "Completed", 10, Some(30));
    backend.set(&key, updated_value).await.unwrap();

    // Verify updated
    let retrieved = backend.get(&key).await.unwrap();
    let retrieved_json: serde_json::Value = serde_json::from_slice(&retrieved.unwrap()).unwrap();
    assert_eq!(
        retrieved_json["metadata"]["status"].as_str().unwrap(),
        "Completed"
    );

    // Verify artifact_count was updated in database
    let client = backend.get_client().await.unwrap();
    let row = client
        .query_one(
            "SELECT artifact_count FROM llmspell.sessions
             WHERE tenant_id = $1 AND session_id = $2",
            &[&tenant_id, &session_id],
        )
        .await
        .unwrap();
    let artifact_count: i32 = row.get(0);
    assert_eq!(artifact_count, 10);

    // Test delete
    backend.delete(&key).await.unwrap();

    // Verify deleted
    assert!(!backend.exists(&key).await.unwrap());
}

#[tokio::test]
async fn test_session_status_mapping() {
    ensure_migrations_run_once().await;

    let config = PostgresConfig::new(APP_CONNECTION_STRING);
    let backend = PostgresBackend::new(config).await.unwrap();
    let tenant_id = unique_tenant_id("status");

    backend.set_tenant_context(&tenant_id).await.unwrap();
    let client = backend.get_client().await.unwrap();

    // Test Active → "active"
    let session_id_1 = Uuid::new_v4();
    let key_1 = format!("session:{}", session_id_1);
    let value_1 = create_session_snapshot(&session_id_1, "Active", 0, Some(30));
    backend.set(&key_1, value_1).await.unwrap();

    let row = client
        .query_one(
            "SELECT status FROM llmspell.sessions
             WHERE tenant_id = $1 AND session_id = $2",
            &[&tenant_id, &session_id_1],
        )
        .await
        .unwrap();
    let status: String = row.get(0);
    assert_eq!(status, "active");

    // Test Suspended → "active" (still active, just paused)
    let session_id_2 = Uuid::new_v4();
    let key_2 = format!("session:{}", session_id_2);
    let value_2 = create_session_snapshot(&session_id_2, "Suspended", 0, Some(30));
    backend.set(&key_2, value_2).await.unwrap();

    let row = client
        .query_one(
            "SELECT status FROM llmspell.sessions
             WHERE tenant_id = $1 AND session_id = $2",
            &[&tenant_id, &session_id_2],
        )
        .await
        .unwrap();
    let status: String = row.get(0);
    assert_eq!(status, "active");

    // Test Completed → "archived"
    let session_id_3 = Uuid::new_v4();
    let key_3 = format!("session:{}", session_id_3);
    let value_3 = create_session_snapshot(&session_id_3, "Completed", 0, Some(30));
    backend.set(&key_3, value_3).await.unwrap();

    let row = client
        .query_one(
            "SELECT status FROM llmspell.sessions
             WHERE tenant_id = $1 AND session_id = $2",
            &[&tenant_id, &session_id_3],
        )
        .await
        .unwrap();
    let status: String = row.get(0);
    assert_eq!(status, "archived");

    // Test Archived → "archived"
    let session_id_4 = Uuid::new_v4();
    let key_4 = format!("session:{}", session_id_4);
    let value_4 = create_session_snapshot(&session_id_4, "Archived", 0, Some(30));
    backend.set(&key_4, value_4).await.unwrap();

    let row = client
        .query_one(
            "SELECT status FROM llmspell.sessions
             WHERE tenant_id = $1 AND session_id = $2",
            &[&tenant_id, &session_id_4],
        )
        .await
        .unwrap();
    let status: String = row.get(0);
    assert_eq!(status, "archived");
}

#[tokio::test]
async fn test_session_expires_at_computation() {
    ensure_migrations_run_once().await;

    let config = PostgresConfig::new(APP_CONNECTION_STRING);
    let backend = PostgresBackend::new(config).await.unwrap();
    let tenant_id = unique_tenant_id("expires");

    backend.set_tenant_context(&tenant_id).await.unwrap();
    let client = backend.get_client().await.unwrap();

    // Test session with retention_days
    let session_id_1 = Uuid::new_v4();
    let key_1 = format!("session:{}", session_id_1);
    let value_1 = create_session_snapshot(&session_id_1, "Active", 0, Some(30));
    backend.set(&key_1, value_1).await.unwrap();

    let row = client
        .query_one(
            "SELECT expires_at, created_at FROM llmspell.sessions
             WHERE tenant_id = $1 AND session_id = $2",
            &[&tenant_id, &session_id_1],
        )
        .await
        .unwrap();
    let expires_at: Option<chrono::DateTime<chrono::Utc>> = row.get(0);
    let created_at: chrono::DateTime<chrono::Utc> = row.get(1);

    assert!(expires_at.is_some(), "expires_at should be set");
    let expires = expires_at.unwrap();
    let expected_expires = created_at + chrono::Duration::days(30);

    // Allow 1-minute tolerance for test timing
    let diff = (expires - expected_expires).num_seconds().abs();
    assert!(
        diff < 60,
        "expires_at should be ~30 days after created_at, diff: {} seconds",
        diff
    );

    // Test session without retention_days (None)
    let session_id_2 = Uuid::new_v4();
    let key_2 = format!("session:{}", session_id_2);
    let value_2 = create_session_snapshot(&session_id_2, "Active", 0, None);
    backend.set(&key_2, value_2).await.unwrap();

    let row = client
        .query_one(
            "SELECT expires_at FROM llmspell.sessions
             WHERE tenant_id = $1 AND session_id = $2",
            &[&tenant_id, &session_id_2],
        )
        .await
        .unwrap();
    let expires_at: Option<chrono::DateTime<chrono::Utc>> = row.get(0);

    assert!(
        expires_at.is_none(),
        "expires_at should be NULL when retention_days is None"
    );
}

#[tokio::test]
async fn test_session_list_keys() {
    ensure_migrations_run_once().await;

    let config = PostgresConfig::new(APP_CONNECTION_STRING);
    let backend = PostgresBackend::new(config).await.unwrap();
    let tenant_id = unique_tenant_id("list");

    backend.set_tenant_context(&tenant_id).await.unwrap();

    // Create 3 sessions
    let session_ids: Vec<Uuid> = (0..3).map(|_| Uuid::new_v4()).collect();
    for session_id in &session_ids {
        let key = format!("session:{}", session_id);
        let value = create_session_snapshot(session_id, "Active", 0, Some(30));
        backend.set(&key, value).await.unwrap();
    }

    // List all session keys
    let keys = backend.list_keys("session:").await.unwrap();

    assert_eq!(keys.len(), 3, "Should have 3 sessions");

    // Verify all session IDs are present
    for session_id in &session_ids {
        let expected_key = format!("session:{}", session_id);
        assert!(
            keys.contains(&expected_key),
            "Session key {} should be in list",
            expected_key
        );
    }

    // Verify keys are in descending created_at order (most recent first)
    // Since we created them in sequence, they should be in reverse order
    for i in 1..keys.len() {
        let prev_session_id = keys[i - 1]
            .strip_prefix("session:")
            .and_then(|s| Uuid::parse_str(s).ok())
            .unwrap();
        let curr_session_id = keys[i]
            .strip_prefix("session:")
            .and_then(|s| Uuid::parse_str(s).ok())
            .unwrap();

        // Get created_at timestamps
        let client = backend.get_client().await.unwrap();
        let prev_created: chrono::DateTime<chrono::Utc> = client
            .query_one(
                "SELECT created_at FROM llmspell.sessions WHERE session_id = $1",
                &[&prev_session_id],
            )
            .await
            .unwrap()
            .get(0);
        let curr_created: chrono::DateTime<chrono::Utc> = client
            .query_one(
                "SELECT created_at FROM llmspell.sessions WHERE session_id = $1",
                &[&curr_session_id],
            )
            .await
            .unwrap()
            .get(0);

        assert!(
            prev_created >= curr_created,
            "Sessions should be ordered by created_at DESC"
        );
    }
}

#[tokio::test]
async fn test_session_batch_operations() {
    ensure_migrations_run_once().await;

    let config = PostgresConfig::new(APP_CONNECTION_STRING);
    let backend = PostgresBackend::new(config).await.unwrap();
    let tenant_id = unique_tenant_id("batch");

    backend.set_tenant_context(&tenant_id).await.unwrap();

    // Create batch items (mix of sessions, state items, and other keys)
    let session_id_1 = Uuid::new_v4();
    let session_id_2 = Uuid::new_v4();

    let mut items = HashMap::new();
    items.insert(
        format!("session:{}", session_id_1),
        create_session_snapshot(&session_id_1, "Active", 5, Some(30)),
    );
    items.insert(
        format!("session:{}", session_id_2),
        create_session_snapshot(&session_id_2, "Completed", 10, Some(30)),
    );
    items.insert(
        format!("session:{}:state1", session_id_1),
        b"state data 1".to_vec(),
    );
    items.insert(
        format!("session:{}:state2", session_id_2),
        b"state data 2".to_vec(),
    );
    items.insert("other:key".to_string(), b"other data".to_vec());

    // Set batch
    backend.set_batch(items.clone()).await.unwrap();

    // Get batch
    let keys: Vec<String> = items.keys().cloned().collect();
    let retrieved = backend.get_batch(&keys).await.unwrap();

    assert_eq!(retrieved.len(), 5, "Should retrieve all 5 items");

    // Verify sessions went to sessions table
    let client = backend.get_client().await.unwrap();
    let sessions_count: i64 = client
        .query_one(
            "SELECT COUNT(*) FROM llmspell.sessions WHERE tenant_id = $1",
            &[&tenant_id],
        )
        .await
        .unwrap()
        .get(0);
    assert_eq!(
        sessions_count, 2,
        "Should have 2 sessions in sessions table"
    );

    // Verify state items went to kv_store
    let kv_count: i64 = client
        .query_one(
            "SELECT COUNT(*) FROM llmspell.kv_store WHERE tenant_id = $1 AND key LIKE 'session:%:%'",
            &[&tenant_id],
        )
        .await
        .unwrap()
        .get(0);
    assert_eq!(kv_count, 2, "Should have 2 state items in kv_store");
}

#[tokio::test]
async fn test_session_rls_tenant_isolation() {
    ensure_migrations_run_once().await;

    let config = PostgresConfig::new(APP_CONNECTION_STRING);
    let backend = PostgresBackend::new(config).await.unwrap();

    let tenant_a = unique_tenant_id("rls-a");
    let tenant_b = unique_tenant_id("rls-b");

    let session_id_a = Uuid::new_v4();
    let session_id_b = Uuid::new_v4();

    let key_a = format!("session:{}", session_id_a);
    let key_b = format!("session:{}", session_id_b);

    // Create session for tenant A
    backend.set_tenant_context(&tenant_a).await.unwrap();
    let value_a = create_session_snapshot(&session_id_a, "Active", 5, Some(30));
    backend.set(&key_a, value_a).await.unwrap();

    // Create session for tenant B
    backend.set_tenant_context(&tenant_b).await.unwrap();
    let value_b = create_session_snapshot(&session_id_b, "Completed", 10, Some(60));
    backend.set(&key_b, value_b).await.unwrap();

    // Query as tenant A - should only see A's session
    backend.set_tenant_context(&tenant_a).await.unwrap();
    assert!(backend.exists(&key_a).await.unwrap());
    assert!(!backend.exists(&key_b).await.unwrap());

    let keys_a = backend.list_keys("session:").await.unwrap();
    assert_eq!(keys_a.len(), 1, "Tenant A should see only 1 session");
    assert!(keys_a.contains(&key_a));

    // Query as tenant B - should only see B's session
    backend.set_tenant_context(&tenant_b).await.unwrap();
    assert!(!backend.exists(&key_a).await.unwrap());
    assert!(backend.exists(&key_b).await.unwrap());

    let keys_b = backend.list_keys("session:").await.unwrap();
    assert_eq!(keys_b.len(), 1, "Tenant B should see only 1 session");
    assert!(keys_b.contains(&key_b));
}

#[tokio::test]
async fn test_session_clear_operations() {
    ensure_migrations_run_once().await;

    let config = PostgresConfig::new(APP_CONNECTION_STRING);
    let backend = PostgresBackend::new(config).await.unwrap();
    let tenant_id = unique_tenant_id("clear");

    backend.set_tenant_context(&tenant_id).await.unwrap();

    // Create multiple sessions and state items
    let session_id = Uuid::new_v4();
    let key = format!("session:{}", session_id);
    let value = create_session_snapshot(&session_id, "Active", 0, Some(30));
    backend.set(&key, value).await.unwrap();

    let state_key = format!("session:{}:state", session_id);
    backend
        .set(&state_key, b"state data".to_vec())
        .await
        .unwrap();

    // Verify they exist
    assert!(backend.exists(&key).await.unwrap());
    assert!(backend.exists(&state_key).await.unwrap());

    // Clear all
    backend.clear().await.unwrap();

    // Verify both are gone
    assert!(!backend.exists(&key).await.unwrap());
    assert!(!backend.exists(&state_key).await.unwrap());

    // Verify sessions table is empty for this tenant
    let client = backend.get_client().await.unwrap();
    let sessions_count: i64 = client
        .query_one(
            "SELECT COUNT(*) FROM llmspell.sessions WHERE tenant_id = $1",
            &[&tenant_id],
        )
        .await
        .unwrap()
        .get(0);
    assert_eq!(sessions_count, 0, "Sessions table should be empty");

    // Verify kv_store is empty for this tenant
    let kv_count: i64 = client
        .query_one(
            "SELECT COUNT(*) FROM llmspell.kv_store WHERE tenant_id = $1",
            &[&tenant_id],
        )
        .await
        .unwrap()
        .get(0);
    assert_eq!(kv_count, 0, "kv_store table should be empty");
}

#[tokio::test]
async fn test_invalid_session_keys() {
    ensure_migrations_run_once().await;

    let config = PostgresConfig::new(APP_CONNECTION_STRING);
    let backend = PostgresBackend::new(config).await.unwrap();
    let tenant_id = unique_tenant_id("invalid");

    backend.set_tenant_context(&tenant_id).await.unwrap();

    // Test invalid UUID format (should route to kv_store, not fail)
    let invalid_key = "session:not-a-uuid";
    let value = b"test data".to_vec();

    // Should not panic, just route to kv_store
    backend.set(invalid_key, value.clone()).await.unwrap();
    assert!(backend.exists(invalid_key).await.unwrap());

    // Verify it went to kv_store (not sessions table)
    let client = backend.get_client().await.unwrap();
    let kv_row = client
        .query_opt(
            "SELECT key FROM llmspell.kv_store WHERE tenant_id = $1 AND key = $2",
            &[&tenant_id, &invalid_key],
        )
        .await
        .unwrap();
    assert!(kv_row.is_some(), "Invalid UUID key should go to kv_store");

    // Verify NOT in sessions table
    let sessions_count: i64 = client
        .query_one(
            "SELECT COUNT(*) FROM llmspell.sessions WHERE tenant_id = $1",
            &[&tenant_id],
        )
        .await
        .unwrap()
        .get(0);
    assert_eq!(sessions_count, 0, "No sessions should be in sessions table");
}
