//! Tests for V8__workflow_states migration (Phase 13b.8.1)
//!
//! Verifies:
//! - workflow_states table created with correct schema
//! - 9 indexes created (tenant, tenant_workflow, status, started, completed, data_gin, execution_stats, tenant_status)
//! - RLS policies applied (4 policies: SELECT, INSERT, UPDATE, DELETE)
//! - Unique constraint on (tenant_id, workflow_id)
//! - Status constraint (valid enum values)
//! - Step index constraint (non-negative)
//! - Trigger auto-updates last_updated timestamp
//! - Lifecycle trigger auto-updates started_at and completed_at
//! - Tenant isolation verification

#![cfg(feature = "postgres")]

use llmspell_storage::{PostgresBackend, PostgresConfig};
use std::error::Error;
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

#[tokio::test]
async fn test_workflow_states_table_created() {
    ensure_migrations_run_once().await;

    let config = PostgresConfig::new(SUPERUSER_CONNECTION_STRING);
    let backend = PostgresBackend::new(config).await.unwrap();
    let client = backend.get_client().await.unwrap();

    // Verify table exists
    let row = client
        .query_one(
            "SELECT COUNT(*) FROM pg_tables
             WHERE schemaname = 'llmspell' AND tablename = 'workflow_states'",
            &[],
        )
        .await
        .unwrap();
    let count: i64 = row.get(0);
    assert_eq!(count, 1, "workflow_states table should exist");

    // Verify RLS is enabled
    let row = client
        .query_one(
            "SELECT relrowsecurity FROM pg_class
             WHERE relname = 'workflow_states' AND relnamespace = 'llmspell'::regnamespace",
            &[],
        )
        .await
        .unwrap();
    let rls_enabled: bool = row.get(0);
    assert!(rls_enabled, "RLS should be enabled on workflow_states");
}

#[tokio::test]
async fn test_workflow_states_indexes_created() {
    ensure_migrations_run_once().await;

    let config = PostgresConfig::new(SUPERUSER_CONNECTION_STRING);
    let backend = PostgresBackend::new(config).await.unwrap();
    let client = backend.get_client().await.unwrap();

    // Get all indexes for workflow_states (excluding primary key and unique constraint indexes)
    let rows = client
        .query(
            "SELECT indexname
             FROM pg_indexes
             WHERE schemaname = 'llmspell'
               AND tablename = 'workflow_states'
               AND indexname LIKE 'idx_%'
             ORDER BY indexname",
            &[],
        )
        .await
        .unwrap();

    // Should have 8 indexes (tenant, tenant_workflow, status, started, completed, data_gin, execution_stats, tenant_status)
    assert_eq!(rows.len(), 8, "Should have 8 indexes");

    let expected_indexes = [
        "idx_workflow_states_completed",
        "idx_workflow_states_data_gin",
        "idx_workflow_states_execution_stats",
        "idx_workflow_states_started",
        "idx_workflow_states_status",
        "idx_workflow_states_tenant",
        "idx_workflow_states_tenant_status",
        "idx_workflow_states_tenant_workflow",
    ];

    for (i, row) in rows.iter().enumerate() {
        let indexname: String = row.get("indexname");
        assert_eq!(indexname, expected_indexes[i]);
    }
}

#[tokio::test]
async fn test_workflow_states_rls_policies() {
    ensure_migrations_run_once().await;

    let config = PostgresConfig::new(SUPERUSER_CONNECTION_STRING);
    let backend = PostgresBackend::new(config).await.unwrap();
    let client = backend.get_client().await.unwrap();

    // Verify all 4 RLS policies exist
    let rows = client
        .query(
            "SELECT policyname, cmd
             FROM pg_policies
             WHERE schemaname = 'llmspell' AND tablename = 'workflow_states'
             ORDER BY policyname",
            &[],
        )
        .await
        .unwrap();

    assert_eq!(
        rows.len(),
        4,
        "Should have 4 RLS policies (SELECT, INSERT, UPDATE, DELETE)"
    );

    let expected_policies = [
        ("tenant_isolation_delete", "DELETE"),
        ("tenant_isolation_insert", "INSERT"),
        ("tenant_isolation_select", "SELECT"),
        ("tenant_isolation_update", "UPDATE"),
    ];

    for (i, row) in rows.iter().enumerate() {
        let policyname: String = row.get("policyname");
        let cmd: String = row.get("cmd");
        assert_eq!(policyname, expected_policies[i].0);
        assert_eq!(cmd, expected_policies[i].1);
    }
}

#[tokio::test]
async fn test_workflow_states_unique_constraint() {
    ensure_migrations_run_once().await;

    let config = PostgresConfig::new(APP_CONNECTION_STRING);
    let backend = PostgresBackend::new(config).await.unwrap();
    let tenant_id = unique_tenant_id("unique-test");

    backend.set_tenant_context(&tenant_id).await.unwrap();
    let client = backend.get_client().await.unwrap();

    let state_data = serde_json::json!({
        "workflow_id": "test-workflow-1",
        "config": {"max_retries": 3},
        "status": "pending",
        "execution_history": [],
        "execution_stats": {"total_executions": 0}
    });

    // Insert first workflow state
    client
        .execute(
            "INSERT INTO llmspell.workflow_states
             (tenant_id, workflow_id, workflow_name, state_data, current_step, status)
             VALUES ($1, $2, $3, $4, $5, $6)",
            &[
                &tenant_id,
                &"test-workflow-1",
                &"Test Workflow",
                &state_data,
                &0i32,
                &"pending",
            ],
        )
        .await
        .unwrap();

    // Try to insert duplicate workflow_id for same tenant (should fail)
    let result = client
        .execute(
            "INSERT INTO llmspell.workflow_states
             (tenant_id, workflow_id, workflow_name, state_data, current_step, status)
             VALUES ($1, $2, $3, $4, $5, $6)",
            &[
                &tenant_id,
                &"test-workflow-1",
                &"Different Name",
                &state_data,
                &1i32,
                &"running",
            ],
        )
        .await;

    assert!(
        result.is_err(),
        "Duplicate workflow_id should violate unique constraint"
    );

    let error = result.unwrap_err();
    let error_msg = if let Some(source) = error.source() {
        source.to_string()
    } else {
        error.to_string()
    };

    assert!(
        error_msg.contains("unique")
            || error_msg.contains("duplicate key")
            || error_msg.contains("already exists"),
        "Error should indicate unique constraint violation, got: {}",
        error_msg
    );
}

#[tokio::test]
async fn test_workflow_states_status_constraint() {
    ensure_migrations_run_once().await;

    let config = PostgresConfig::new(APP_CONNECTION_STRING);
    let backend = PostgresBackend::new(config).await.unwrap();
    let tenant_id = unique_tenant_id("status-test");

    backend.set_tenant_context(&tenant_id).await.unwrap();
    let client = backend.get_client().await.unwrap();

    let state_data = serde_json::json!({
        "workflow_id": "test-workflow-2",
        "config": {},
        "status": "invalid",
        "execution_history": []
    });

    // Try to insert workflow with invalid status (should fail)
    let result = client
        .execute(
            "INSERT INTO llmspell.workflow_states
             (tenant_id, workflow_id, state_data, current_step, status)
             VALUES ($1, $2, $3, $4, $5)",
            &[
                &tenant_id,
                &"test-workflow-2",
                &state_data,
                &0i32,
                &"invalid",
            ],
        )
        .await;

    assert!(
        result.is_err(),
        "Invalid status should violate valid_workflow_status constraint"
    );

    let error = result.unwrap_err();
    let error_msg = if let Some(source) = error.source() {
        source.to_string()
    } else {
        error.to_string()
    };

    assert!(
        error_msg.contains("valid_workflow_status") || error_msg.contains("check constraint"),
        "Error should indicate status constraint violation, got: {}",
        error_msg
    );
}

#[tokio::test]
async fn test_workflow_states_step_index_constraint() {
    ensure_migrations_run_once().await;

    let config = PostgresConfig::new(APP_CONNECTION_STRING);
    let backend = PostgresBackend::new(config).await.unwrap();
    let tenant_id = unique_tenant_id("step-test");

    backend.set_tenant_context(&tenant_id).await.unwrap();
    let client = backend.get_client().await.unwrap();

    let state_data = serde_json::json!({
        "workflow_id": "test-workflow-3",
        "config": {},
        "status": "pending",
        "execution_history": []
    });

    // Try to insert workflow with negative step index (should fail)
    let result = client
        .execute(
            "INSERT INTO llmspell.workflow_states
             (tenant_id, workflow_id, state_data, current_step, status)
             VALUES ($1, $2, $3, $4, $5)",
            &[
                &tenant_id,
                &"test-workflow-3",
                &state_data,
                &-1i32,
                &"pending",
            ],
        )
        .await;

    assert!(
        result.is_err(),
        "Negative step index should violate positive_step_index constraint"
    );

    let error = result.unwrap_err();
    let error_msg = if let Some(source) = error.source() {
        source.to_string()
    } else {
        error.to_string()
    };

    assert!(
        error_msg.contains("positive_step_index") || error_msg.contains("check constraint"),
        "Error should indicate step index constraint violation, got: {}",
        error_msg
    );
}

#[tokio::test]
async fn test_workflow_states_updated_at_trigger() {
    ensure_migrations_run_once().await;

    let config = PostgresConfig::new(APP_CONNECTION_STRING);
    let backend = PostgresBackend::new(config).await.unwrap();
    let tenant_id = unique_tenant_id("trigger-test");

    backend.set_tenant_context(&tenant_id).await.unwrap();
    let client = backend.get_client().await.unwrap();

    let state_data = serde_json::json!({
        "workflow_id": "test-workflow-4",
        "config": {},
        "status": "pending",
        "execution_history": []
    });

    // Insert workflow state
    client
        .execute(
            "INSERT INTO llmspell.workflow_states
             (tenant_id, workflow_id, state_data, current_step, status)
             VALUES ($1, $2, $3, $4, $5)",
            &[
                &tenant_id,
                &"test-workflow-4",
                &state_data,
                &0i32,
                &"pending",
            ],
        )
        .await
        .unwrap();

    // Get initial last_updated
    let row = client
        .query_one(
            "SELECT last_updated FROM llmspell.workflow_states
             WHERE tenant_id = $1 AND workflow_id = $2",
            &[&tenant_id, &"test-workflow-4"],
        )
        .await
        .unwrap();
    let initial_updated_at: chrono::DateTime<chrono::Utc> = row.get(0);

    // Wait a bit to ensure timestamp difference
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    // Update the workflow state
    client
        .execute(
            "UPDATE llmspell.workflow_states
             SET current_step = current_step + 1
             WHERE tenant_id = $1 AND workflow_id = $2",
            &[&tenant_id, &"test-workflow-4"],
        )
        .await
        .unwrap();

    // Get updated last_updated
    let row = client
        .query_one(
            "SELECT last_updated FROM llmspell.workflow_states
             WHERE tenant_id = $1 AND workflow_id = $2",
            &[&tenant_id, &"test-workflow-4"],
        )
        .await
        .unwrap();
    let updated_updated_at: chrono::DateTime<chrono::Utc> = row.get(0);

    assert!(
        updated_updated_at > initial_updated_at,
        "last_updated should be automatically updated by trigger"
    );
}

#[tokio::test]
async fn test_workflow_states_lifecycle_trigger_started_at() {
    ensure_migrations_run_once().await;

    let config = PostgresConfig::new(APP_CONNECTION_STRING);
    let backend = PostgresBackend::new(config).await.unwrap();
    let tenant_id = unique_tenant_id("lifecycle-started-test");

    backend.set_tenant_context(&tenant_id).await.unwrap();
    let client = backend.get_client().await.unwrap();

    let state_data = serde_json::json!({
        "workflow_id": "test-workflow-5",
        "config": {},
        "status": "pending",
        "execution_history": []
    });

    // Insert workflow in pending state
    client
        .execute(
            "INSERT INTO llmspell.workflow_states
             (tenant_id, workflow_id, state_data, current_step, status)
             VALUES ($1, $2, $3, $4, $5)",
            &[
                &tenant_id,
                &"test-workflow-5",
                &state_data,
                &0i32,
                &"pending",
            ],
        )
        .await
        .unwrap();

    // Verify started_at is NULL
    let row = client
        .query_one(
            "SELECT started_at FROM llmspell.workflow_states
             WHERE tenant_id = $1 AND workflow_id = $2",
            &[&tenant_id, &"test-workflow-5"],
        )
        .await
        .unwrap();
    let started_at: Option<chrono::DateTime<chrono::Utc>> = row.get(0);
    assert!(started_at.is_none(), "started_at should be NULL initially");

    // Update status to running
    client
        .execute(
            "UPDATE llmspell.workflow_states
             SET status = 'running'
             WHERE tenant_id = $1 AND workflow_id = $2",
            &[&tenant_id, &"test-workflow-5"],
        )
        .await
        .unwrap();

    // Verify started_at is now set
    let row = client
        .query_one(
            "SELECT started_at FROM llmspell.workflow_states
             WHERE tenant_id = $1 AND workflow_id = $2",
            &[&tenant_id, &"test-workflow-5"],
        )
        .await
        .unwrap();
    let started_at: Option<chrono::DateTime<chrono::Utc>> = row.get(0);
    assert!(
        started_at.is_some(),
        "started_at should be set when status changes to running"
    );
}

#[tokio::test]
async fn test_workflow_states_lifecycle_trigger_completed_at() {
    ensure_migrations_run_once().await;

    let config = PostgresConfig::new(APP_CONNECTION_STRING);
    let backend = PostgresBackend::new(config).await.unwrap();
    let tenant_id = unique_tenant_id("lifecycle-completed-test");

    backend.set_tenant_context(&tenant_id).await.unwrap();
    let client = backend.get_client().await.unwrap();

    let state_data = serde_json::json!({
        "workflow_id": "test-workflow-6",
        "config": {},
        "status": "running",
        "execution_history": []
    });

    // Insert workflow in running state
    client
        .execute(
            "INSERT INTO llmspell.workflow_states
             (tenant_id, workflow_id, state_data, current_step, status)
             VALUES ($1, $2, $3, $4, $5)",
            &[
                &tenant_id,
                &"test-workflow-6",
                &state_data,
                &0i32,
                &"running",
            ],
        )
        .await
        .unwrap();

    // Verify completed_at is NULL
    let row = client
        .query_one(
            "SELECT completed_at FROM llmspell.workflow_states
             WHERE tenant_id = $1 AND workflow_id = $2",
            &[&tenant_id, &"test-workflow-6"],
        )
        .await
        .unwrap();
    let completed_at: Option<chrono::DateTime<chrono::Utc>> = row.get(0);
    assert!(
        completed_at.is_none(),
        "completed_at should be NULL initially"
    );

    // Update status to completed
    client
        .execute(
            "UPDATE llmspell.workflow_states
             SET status = 'completed'
             WHERE tenant_id = $1 AND workflow_id = $2",
            &[&tenant_id, &"test-workflow-6"],
        )
        .await
        .unwrap();

    // Verify completed_at is now set
    let row = client
        .query_one(
            "SELECT completed_at FROM llmspell.workflow_states
             WHERE tenant_id = $1 AND workflow_id = $2",
            &[&tenant_id, &"test-workflow-6"],
        )
        .await
        .unwrap();
    let completed_at: Option<chrono::DateTime<chrono::Utc>> = row.get(0);
    assert!(
        completed_at.is_some(),
        "completed_at should be set when status changes to completed"
    );
}

#[tokio::test]
async fn test_workflow_states_lifecycle_trigger_failed() {
    ensure_migrations_run_once().await;

    let config = PostgresConfig::new(APP_CONNECTION_STRING);
    let backend = PostgresBackend::new(config).await.unwrap();
    let tenant_id = unique_tenant_id("lifecycle-failed-test");

    backend.set_tenant_context(&tenant_id).await.unwrap();
    let client = backend.get_client().await.unwrap();

    let state_data = serde_json::json!({
        "workflow_id": "test-workflow-7",
        "config": {},
        "status": "running",
        "execution_history": []
    });

    // Insert workflow in running state
    client
        .execute(
            "INSERT INTO llmspell.workflow_states
             (tenant_id, workflow_id, state_data, current_step, status)
             VALUES ($1, $2, $3, $4, $5)",
            &[
                &tenant_id,
                &"test-workflow-7",
                &state_data,
                &0i32,
                &"running",
            ],
        )
        .await
        .unwrap();

    // Update status to failed
    client
        .execute(
            "UPDATE llmspell.workflow_states
             SET status = 'failed'
             WHERE tenant_id = $1 AND workflow_id = $2",
            &[&tenant_id, &"test-workflow-7"],
        )
        .await
        .unwrap();

    // Verify completed_at is set for failed status
    let row = client
        .query_one(
            "SELECT completed_at FROM llmspell.workflow_states
             WHERE tenant_id = $1 AND workflow_id = $2",
            &[&tenant_id, &"test-workflow-7"],
        )
        .await
        .unwrap();
    let completed_at: Option<chrono::DateTime<chrono::Utc>> = row.get(0);
    assert!(
        completed_at.is_some(),
        "completed_at should be set when status changes to failed"
    );
}

#[tokio::test]
async fn test_workflow_states_rls_isolation() {
    ensure_migrations_run_once().await;

    let config = PostgresConfig::new(APP_CONNECTION_STRING);
    let backend = PostgresBackend::new(config).await.unwrap();

    let tenant_a = unique_tenant_id("rls-a");
    let tenant_b = unique_tenant_id("rls-b");

    let state_data_a = serde_json::json!({
        "workflow_id": "workflow-a",
        "config": {},
        "status": "running",
        "execution_history": []
    });

    let state_data_b = serde_json::json!({
        "workflow_id": "workflow-b",
        "config": {},
        "status": "completed",
        "execution_history": []
    });

    // Insert workflow for tenant A
    backend.set_tenant_context(&tenant_a).await.unwrap();
    {
        let client = backend.get_client().await.unwrap();
        client
            .execute(
                "INSERT INTO llmspell.workflow_states
                 (tenant_id, workflow_id, workflow_name, state_data, current_step, status)
                 VALUES ($1, $2, $3, $4, $5, $6)",
                &[
                    &tenant_a,
                    &"workflow-a",
                    &"Workflow A",
                    &state_data_a,
                    &5i32,
                    &"running",
                ],
            )
            .await
            .unwrap();
    }

    // Insert workflow for tenant B
    backend.set_tenant_context(&tenant_b).await.unwrap();
    {
        let client = backend.get_client().await.unwrap();
        client
            .execute(
                "INSERT INTO llmspell.workflow_states
                 (tenant_id, workflow_id, workflow_name, state_data, current_step, status)
                 VALUES ($1, $2, $3, $4, $5, $6)",
                &[
                    &tenant_b,
                    &"workflow-b",
                    &"Workflow B",
                    &state_data_b,
                    &10i32,
                    &"completed",
                ],
            )
            .await
            .unwrap();
    }

    // Query as tenant A - should only see tenant A's workflow
    backend.set_tenant_context(&tenant_a).await.unwrap();
    let rows = {
        let client = backend.get_client().await.unwrap();
        client
            .query(
                "SELECT workflow_id, workflow_name, current_step, status
                 FROM llmspell.workflow_states
                 ORDER BY workflow_id",
                &[],
            )
            .await
            .unwrap()
    };

    assert_eq!(rows.len(), 1, "Tenant A should see only 1 workflow");
    let workflow_id: String = rows[0].get(0);
    let workflow_name: Option<String> = rows[0].get(1);
    let current_step: i32 = rows[0].get(2);
    let status: String = rows[0].get(3);
    assert_eq!(workflow_id, "workflow-a");
    assert_eq!(workflow_name, Some("Workflow A".to_string()));
    assert_eq!(current_step, 5);
    assert_eq!(status, "running");

    // Query as tenant B - should only see tenant B's workflow
    backend.set_tenant_context(&tenant_b).await.unwrap();
    let rows = {
        let client = backend.get_client().await.unwrap();
        client
            .query(
                "SELECT workflow_id, workflow_name, current_step, status
                 FROM llmspell.workflow_states
                 ORDER BY workflow_id",
                &[],
            )
            .await
            .unwrap()
    };

    assert_eq!(rows.len(), 1, "Tenant B should see only 1 workflow");
    let workflow_id: String = rows[0].get(0);
    let workflow_name: Option<String> = rows[0].get(1);
    let current_step: i32 = rows[0].get(2);
    let status: String = rows[0].get(3);
    assert_eq!(workflow_id, "workflow-b");
    assert_eq!(workflow_name, Some("Workflow B".to_string()));
    assert_eq!(current_step, 10);
    assert_eq!(status, "completed");
}
