//! Tests for PostgreSQL workflow state backend operations (Phase 13b.8.2)
//!
//! Verifies:
//! - Workflow state CRUD operations (get, set, delete, exists)
//! - Key parsing and format validation
//! - Status and current_step extraction from PersistentWorkflowState
//! - List operations with prefix filtering
//! - Batch operations (get_batch, set_batch)
//! - Tenant isolation (RLS enforcement)
//! - Clear operations

#![cfg(feature = "postgres")]

use llmspell_storage::traits::StorageBackend;
use llmspell_storage::{PostgresBackend, PostgresConfig};
use std::collections::HashMap;
use tokio::sync::OnceCell;
use uuid::Uuid;

const SUPERUSER_CONNECTION_STRING: &str =
    "postgresql://llmspell:llmspell_dev_pass@localhost:5432/llmspell_dev";

const APP_CONNECTION_STRING: &str =
    "postgresql://llmspell_app:llmspell_app_pass@localhost:5432/llmspell_dev";

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

fn create_workflow_state_json(workflow_id: &str, status: &str, current_step: i32) -> Vec<u8> {
    let state = serde_json::json!({
        "workflow_id": workflow_id,
        "config": {
            "max_execution_time": null,
            "default_step_timeout": {"secs": 30, "nanos": 0},
            "max_retry_attempts": 3,
            "retry_delay_ms": 1000,
            "exponential_backoff": true,
            "continue_on_error": false,
            "default_error_strategy": "Fail"
        },
        "workflow_state": {
            "execution_id": {"id": "test-exec-123"},
            "current_step": current_step,
            "shared_data": {},
            "step_outputs": {},
            "start_time": null
        },
        "status": status,
        "execution_history": [],
        "metadata": {
            "id": {"id": workflow_id},
            "name": "Test Workflow",
            "description": "Test workflow state",
            "version": "0.1.0",
            "created_at": {"secs_since_epoch": 1700000000, "nanos_since_epoch": 0},
            "updated_at": {"secs_since_epoch": 1700000000, "nanos_since_epoch": 0}
        },
        "execution_stats": {
            "total_executions": 1,
            "successful_executions": 0,
            "failed_executions": 0,
            "average_duration_ms": 0.0,
            "last_execution_duration_ms": null,
            "total_retry_attempts": 0
        },
        "checkpoints": {},
        "last_updated": {"secs_since_epoch": 1700000000, "nanos_since_epoch": 0},
        "custom_properties": {}
    });

    serde_json::to_vec(&state).unwrap()
}

#[tokio::test]
async fn test_workflow_state_set_and_get() {
    ensure_migrations_run_once().await;

    let config = PostgresConfig::new(APP_CONNECTION_STRING);
    let backend = PostgresBackend::new(config).await.unwrap();
    let tenant_id = unique_tenant_id("workflow-set-get");

    backend.set_tenant_context(tenant_id.clone()).await.unwrap();

    let workflow_id = "test-workflow-1";
    let key = format!("custom:workflow_{}:state", workflow_id);
    let state = create_workflow_state_json(workflow_id, "running", 2);

    // Set workflow state
    backend.set(&key, state.clone()).await.unwrap();

    // Get workflow state
    let result = backend.get(&key).await.unwrap();
    assert!(result.is_some(), "Workflow state should exist");

    let retrieved = result.unwrap();
    let retrieved_json: serde_json::Value = serde_json::from_slice(&retrieved).unwrap();
    let original_json: serde_json::Value = serde_json::from_slice(&state).unwrap();

    assert_eq!(
        retrieved_json["workflow_id"], original_json["workflow_id"],
        "Workflow ID should match"
    );
    assert_eq!(
        retrieved_json["status"], original_json["status"],
        "Status should match"
    );
    assert_eq!(
        retrieved_json["workflow_state"]["current_step"],
        original_json["workflow_state"]["current_step"],
        "Current step should match"
    );
}

#[tokio::test]
async fn test_workflow_state_update() {
    ensure_migrations_run_once().await;

    let config = PostgresConfig::new(APP_CONNECTION_STRING);
    let backend = PostgresBackend::new(config).await.unwrap();
    let tenant_id = unique_tenant_id("workflow-update");

    backend.set_tenant_context(tenant_id.clone()).await.unwrap();

    let workflow_id = "test-workflow-2";
    let key = format!("custom:workflow_{}:state", workflow_id);

    // Set initial state
    let initial_state = create_workflow_state_json(workflow_id, "pending", 0);
    backend.set(&key, initial_state).await.unwrap();

    // Update state (different status and step)
    let updated_state = create_workflow_state_json(workflow_id, "completed", 5);
    backend.set(&key, updated_state.clone()).await.unwrap();

    // Verify update
    let result = backend.get(&key).await.unwrap().unwrap();
    let retrieved: serde_json::Value = serde_json::from_slice(&result).unwrap();

    assert_eq!(retrieved["status"], "completed", "Status should be updated");
    assert_eq!(
        retrieved["workflow_state"]["current_step"], 5,
        "Current step should be updated"
    );
}

#[tokio::test]
async fn test_workflow_state_delete() {
    ensure_migrations_run_once().await;

    let config = PostgresConfig::new(APP_CONNECTION_STRING);
    let backend = PostgresBackend::new(config).await.unwrap();
    let tenant_id = unique_tenant_id("workflow-delete");

    backend.set_tenant_context(tenant_id.clone()).await.unwrap();

    let workflow_id = "test-workflow-3";
    let key = format!("custom:workflow_{}:state", workflow_id);
    let state = create_workflow_state_json(workflow_id, "running", 1);

    // Set state
    backend.set(&key, state).await.unwrap();
    assert!(
        backend.exists(&key).await.unwrap(),
        "Should exist after set"
    );

    // Delete state
    backend.delete(&key).await.unwrap();
    assert!(
        !backend.exists(&key).await.unwrap(),
        "Should not exist after delete"
    );

    // Verify get returns None
    let result = backend.get(&key).await.unwrap();
    assert!(result.is_none(), "Get should return None after delete");
}

#[tokio::test]
async fn test_workflow_state_exists() {
    ensure_migrations_run_once().await;

    let config = PostgresConfig::new(APP_CONNECTION_STRING);
    let backend = PostgresBackend::new(config).await.unwrap();
    let tenant_id = unique_tenant_id("workflow-exists");

    backend.set_tenant_context(tenant_id.clone()).await.unwrap();

    let workflow_id = "test-workflow-4";
    let key = format!("custom:workflow_{}:state", workflow_id);

    // Check non-existent
    assert!(
        !backend.exists(&key).await.unwrap(),
        "Should not exist initially"
    );

    // Set state
    let state = create_workflow_state_json(workflow_id, "pending", 0);
    backend.set(&key, state).await.unwrap();

    // Check exists
    assert!(
        backend.exists(&key).await.unwrap(),
        "Should exist after set"
    );
}

#[tokio::test]
async fn test_workflow_state_list_keys() {
    ensure_migrations_run_once().await;

    let config = PostgresConfig::new(APP_CONNECTION_STRING);
    let backend = PostgresBackend::new(config).await.unwrap();
    let tenant_id = unique_tenant_id("workflow-list");

    backend.set_tenant_context(tenant_id.clone()).await.unwrap();

    // Create multiple workflow states
    let workflow_ids = vec!["workflow-alpha", "workflow-beta", "workflow-gamma"];
    for (i, wf_id) in workflow_ids.iter().enumerate() {
        let key = format!("custom:workflow_{}:state", wf_id);
        let state = create_workflow_state_json(wf_id, "running", i as i32);
        backend.set(&key, state).await.unwrap();
    }

    // List all workflow keys
    let keys = backend.list_keys("custom:workflow_").await.unwrap();

    assert_eq!(keys.len(), 3, "Should have 3 workflow states");

    // Verify all keys are present
    for wf_id in workflow_ids {
        let expected_key = format!("custom:workflow_{}:state", wf_id);
        assert!(
            keys.contains(&expected_key),
            "Should contain key for {}",
            wf_id
        );
    }

    // List with prefix filter
    let filtered_keys = backend
        .list_keys("custom:workflow_workflow-a")
        .await
        .unwrap();

    assert_eq!(
        filtered_keys.len(),
        1,
        "Should have 1 workflow matching prefix"
    );
    assert_eq!(
        filtered_keys[0], "custom:workflow_workflow-alpha:state",
        "Should match alpha workflow"
    );
}

#[tokio::test]
async fn test_workflow_state_batch_get() {
    ensure_migrations_run_once().await;

    let config = PostgresConfig::new(APP_CONNECTION_STRING);
    let backend = PostgresBackend::new(config).await.unwrap();
    let tenant_id = unique_tenant_id("workflow-batch-get");

    backend.set_tenant_context(tenant_id.clone()).await.unwrap();

    // Create workflow states
    let workflow_ids = ["batch-workflow-1", "batch-workflow-2", "batch-workflow-3"];
    let mut expected_keys = Vec::new();
    for (i, wf_id) in workflow_ids.iter().enumerate() {
        let key = format!("custom:workflow_{}:state", wf_id);
        let state = create_workflow_state_json(wf_id, "running", i as i32);
        backend.set(&key, state).await.unwrap();
        expected_keys.push(key);
    }

    // Batch get
    let results = backend
        .get_batch(
            &expected_keys
                .iter()
                .map(|s| s.to_string())
                .collect::<Vec<_>>(),
        )
        .await
        .unwrap();

    assert_eq!(results.len(), 3, "Should retrieve all 3 workflow states");

    for key in expected_keys {
        assert!(
            results.contains_key(&key),
            "Batch get should contain key: {}",
            key
        );
    }
}

#[tokio::test]
async fn test_workflow_state_batch_set() {
    ensure_migrations_run_once().await;

    let config = PostgresConfig::new(APP_CONNECTION_STRING);
    let backend = PostgresBackend::new(config).await.unwrap();
    let tenant_id = unique_tenant_id("workflow-batch-set");

    backend.set_tenant_context(tenant_id.clone()).await.unwrap();

    // Prepare batch items
    let mut items = HashMap::new();
    let workflow_ids = vec!["batch-set-1", "batch-set-2", "batch-set-3"];

    for (i, wf_id) in workflow_ids.iter().enumerate() {
        let key = format!("custom:workflow_{}:state", wf_id);
        let state = create_workflow_state_json(wf_id, "pending", i as i32);
        items.insert(key, state);
    }

    // Batch set
    backend.set_batch(items).await.unwrap();

    // Verify all were set
    for wf_id in workflow_ids {
        let key = format!("custom:workflow_{}:state", wf_id);
        assert!(
            backend.exists(&key).await.unwrap(),
            "Workflow {} should exist after batch set",
            wf_id
        );
    }
}

#[tokio::test]
async fn test_workflow_state_tenant_isolation() {
    ensure_migrations_run_once().await;

    let config = PostgresConfig::new(APP_CONNECTION_STRING);
    let backend = PostgresBackend::new(config).await.unwrap();

    let tenant_a = unique_tenant_id("tenant-a");
    let tenant_b = unique_tenant_id("tenant-b");

    let workflow_id = "shared-workflow-id";
    let key = format!("custom:workflow_{}:state", workflow_id);

    // Set state for tenant A
    backend.set_tenant_context(tenant_a.clone()).await.unwrap();
    let state_a = create_workflow_state_json(workflow_id, "running", 5);
    backend.set(&key, state_a.clone()).await.unwrap();

    // Set different state for tenant B (same workflow_id)
    backend.set_tenant_context(tenant_b.clone()).await.unwrap();
    let state_b = create_workflow_state_json(workflow_id, "completed", 10);
    backend.set(&key, state_b.clone()).await.unwrap();

    // Verify tenant A sees only their state
    backend.set_tenant_context(tenant_a.clone()).await.unwrap();
    let result_a = backend.get(&key).await.unwrap().unwrap();
    let json_a: serde_json::Value = serde_json::from_slice(&result_a).unwrap();
    assert_eq!(json_a["status"], "running", "Tenant A should see running");
    assert_eq!(
        json_a["workflow_state"]["current_step"], 5,
        "Tenant A should see step 5"
    );

    // Verify tenant B sees only their state
    backend.set_tenant_context(tenant_b.clone()).await.unwrap();
    let result_b = backend.get(&key).await.unwrap().unwrap();
    let json_b: serde_json::Value = serde_json::from_slice(&result_b).unwrap();
    assert_eq!(
        json_b["status"], "completed",
        "Tenant B should see completed"
    );
    assert_eq!(
        json_b["workflow_state"]["current_step"], 10,
        "Tenant B should see step 10"
    );
}

#[tokio::test]
async fn test_workflow_state_clear() {
    ensure_migrations_run_once().await;

    let config = PostgresConfig::new(APP_CONNECTION_STRING);
    let backend = PostgresBackend::new(config).await.unwrap();
    let tenant_id = unique_tenant_id("workflow-clear");

    backend.set_tenant_context(tenant_id.clone()).await.unwrap();

    // Create multiple workflow states
    let workflow_ids = vec!["clear-workflow-1", "clear-workflow-2", "clear-workflow-3"];
    for (i, wf_id) in workflow_ids.iter().enumerate() {
        let key = format!("custom:workflow_{}:state", wf_id);
        let state = create_workflow_state_json(wf_id, "running", i as i32);
        backend.set(&key, state).await.unwrap();
    }

    // Verify all exist
    for wf_id in &workflow_ids {
        let key = format!("custom:workflow_{}:state", wf_id);
        assert!(
            backend.exists(&key).await.unwrap(),
            "Workflow {} should exist before clear",
            wf_id
        );
    }

    // Clear all
    backend.clear().await.unwrap();

    // Verify all cleared
    for wf_id in workflow_ids {
        let key = format!("custom:workflow_{}:state", wf_id);
        assert!(
            !backend.exists(&key).await.unwrap(),
            "Workflow {} should not exist after clear",
            wf_id
        );
    }
}

#[tokio::test]
async fn test_workflow_state_mixed_routing() {
    ensure_migrations_run_once().await;

    let config = PostgresConfig::new(APP_CONNECTION_STRING);
    let backend = PostgresBackend::new(config).await.unwrap();
    let tenant_id = unique_tenant_id("workflow-mixed");

    backend.set_tenant_context(tenant_id.clone()).await.unwrap();

    // Set workflow state (routes to workflow_states table)
    let workflow_key = "custom:workflow_test-wf:state";
    let workflow_state = create_workflow_state_json("test-wf", "running", 3);
    backend
        .set(workflow_key, workflow_state.clone())
        .await
        .unwrap();

    // Set KV data (routes to kv_store table)
    let kv_key = "custom:some_other_data";
    let kv_value = b"test value".to_vec();
    backend.set(kv_key, kv_value.clone()).await.unwrap();

    // Verify both exist
    assert!(backend.exists(workflow_key).await.unwrap());
    assert!(backend.exists(kv_key).await.unwrap());

    // Verify retrieval works correctly (compare JSON, not raw bytes due to field ordering)
    let wf_result = backend.get(workflow_key).await.unwrap().unwrap();
    let wf_result_json: serde_json::Value = serde_json::from_slice(&wf_result).unwrap();
    let wf_expected_json: serde_json::Value = serde_json::from_slice(&workflow_state).unwrap();
    assert_eq!(
        wf_result_json, wf_expected_json,
        "Workflow state JSON should match"
    );

    let kv_result = backend.get(kv_key).await.unwrap().unwrap();
    assert_eq!(kv_result, kv_value);

    // Clear should remove both
    backend.clear().await.unwrap();
    assert!(!backend.exists(workflow_key).await.unwrap());
    assert!(!backend.exists(kv_key).await.unwrap());
}

#[tokio::test]
async fn test_workflow_state_status_extraction() {
    ensure_migrations_run_once().await;

    let config = PostgresConfig::new(APP_CONNECTION_STRING);
    let backend = PostgresBackend::new(config).await.unwrap();
    let tenant_id = unique_tenant_id("workflow-status");

    backend.set_tenant_context(tenant_id.clone()).await.unwrap();

    let workflow_id = "status-workflow";
    let key = format!("custom:workflow_{}:state", workflow_id);

    // Test each status
    let statuses = ["pending", "running", "completed", "failed", "cancelled"];
    for (i, status) in statuses.iter().enumerate() {
        let state = create_workflow_state_json(workflow_id, status, i as i32);
        backend.set(&key, state).await.unwrap();

        // Verify status was extracted and stored correctly by reading directly from DB
        let client = backend.get_client().await.unwrap();
        let row = client
            .query_one(
                "SELECT status, current_step FROM llmspell.workflow_states
                 WHERE tenant_id = $1 AND workflow_id = $2",
                &[&tenant_id, &workflow_id],
            )
            .await
            .unwrap();

        let stored_status: String = row.get(0);
        let stored_step: i32 = row.get(1);

        assert_eq!(stored_status, *status, "Extracted status should match");
        assert_eq!(stored_step, i as i32, "Extracted current_step should match");
    }
}

#[tokio::test]
async fn test_workflow_state_invalid_key_format() {
    ensure_migrations_run_once().await;

    let config = PostgresConfig::new(APP_CONNECTION_STRING);
    let backend = PostgresBackend::new(config).await.unwrap();
    let tenant_id = unique_tenant_id("workflow-invalid-key");

    backend.set_tenant_context(tenant_id.clone()).await.unwrap();

    // Test invalid key formats
    let invalid_keys = vec![
        "workflow:test:state",  // Missing "custom:" prefix
        "custom:test:state",    // Missing "workflow_" prefix
        "custom:workflow_test", // Missing ":state" suffix
        "agent:some:agent",     // Wrong routing (agent, not workflow)
        "custom:workflow_",     // Empty workflow_id
    ];

    let state = create_workflow_state_json("test", "running", 0);

    for invalid_key in invalid_keys {
        // Workflow routing should fail for invalid keys (or route to kv_store)
        // We expect either an error or it goes to kv_store (not workflow_states)

        // For keys that don't start with "custom:workflow_", they won't route to workflow backend
        // For keys that do start with "custom:workflow_" but are malformed, parse_workflow_key will error

        if invalid_key.starts_with("custom:workflow_") {
            // These should error during parse
            let result = backend.set(invalid_key, state.clone()).await;
            assert!(
                result.is_err(),
                "Invalid workflow key should error: {}",
                invalid_key
            );
        }
        // Other keys will just go to KV store, which is fine
    }
}
