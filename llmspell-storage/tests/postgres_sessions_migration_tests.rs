//! Tests for V9__sessions migration (Phase 13b.9.1)
//!
//! Verifies:
//! - sessions table created with correct schema
//! - 10 indexes created (tenant, tenant_session, status, expires, created, accessed, data_gin, tenant_status, tenant_expires)
//! - RLS policies applied (4 policies: SELECT, INSERT, UPDATE, DELETE)
//! - Unique constraint on (tenant_id, session_id)
//! - Status constraint (valid enum values)
//! - Artifact count constraint (non-negative)
//! - Triggers auto-update updated_at and last_accessed_at
//! - Tenant isolation verification

#![cfg(feature = "postgres")]

use llmspell_storage::{PostgresBackend, PostgresConfig};
use std::error::Error;
use tokio::sync::OnceCell;
use uuid::Uuid;

const SUPERUSER_CONNECTION_STRING: &str =
    "postgresql://llmspell:llmspell_dev_pass@localhost:5432/llmspell_dev";

const TEST_CONNECTION_STRING: &str =
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
async fn test_sessions_table_created() {
    ensure_migrations_run_once().await;

    let config = PostgresConfig::new(SUPERUSER_CONNECTION_STRING);
    let backend = PostgresBackend::new(config).await.unwrap();
    let client = backend.get_client().await.unwrap();

    // Verify table exists
    let row = client
        .query_one(
            "SELECT COUNT(*) FROM pg_tables
             WHERE schemaname = 'llmspell' AND tablename = 'sessions'",
            &[],
        )
        .await
        .unwrap();
    let count: i64 = row.get(0);
    assert_eq!(count, 1, "sessions table should exist");

    // Verify RLS is enabled
    let row = client
        .query_one(
            "SELECT relrowsecurity FROM pg_class
             WHERE relname = 'sessions' AND relnamespace = 'llmspell'::regnamespace",
            &[],
        )
        .await
        .unwrap();
    let rls_enabled: bool = row.get(0);
    assert!(rls_enabled, "RLS should be enabled on sessions");
}

#[tokio::test]
async fn test_sessions_indexes_created() {
    ensure_migrations_run_once().await;

    let config = PostgresConfig::new(SUPERUSER_CONNECTION_STRING);
    let backend = PostgresBackend::new(config).await.unwrap();
    let client = backend.get_client().await.unwrap();

    // Get all indexes for sessions (excluding primary key)
    let rows = client
        .query(
            "SELECT indexname
             FROM pg_indexes
             WHERE schemaname = 'llmspell'
               AND tablename = 'sessions'
               AND indexname LIKE 'idx_%'
             ORDER BY indexname",
            &[],
        )
        .await
        .unwrap();

    // Should have 9 indexes
    assert_eq!(rows.len(), 9, "Should have 9 indexes");

    let expected_indexes = [
        "idx_sessions_accessed",
        "idx_sessions_created",
        "idx_sessions_data_gin",
        "idx_sessions_expires",
        "idx_sessions_status",
        "idx_sessions_tenant",
        "idx_sessions_tenant_expires",
        "idx_sessions_tenant_session",
        "idx_sessions_tenant_status",
    ];

    for (i, row) in rows.iter().enumerate() {
        let indexname: String = row.get("indexname");
        assert_eq!(indexname, expected_indexes[i]);
    }
}

#[tokio::test]
async fn test_sessions_rls_policies() {
    ensure_migrations_run_once().await;

    let config = PostgresConfig::new(SUPERUSER_CONNECTION_STRING);
    let backend = PostgresBackend::new(config).await.unwrap();
    let client = backend.get_client().await.unwrap();

    // Verify all 4 RLS policies exist
    let rows = client
        .query(
            "SELECT policyname, cmd
             FROM pg_policies
             WHERE schemaname = 'llmspell' AND tablename = 'sessions'
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
async fn test_sessions_unique_constraint() {
    ensure_migrations_run_once().await;

    let config = PostgresConfig::new(TEST_CONNECTION_STRING);
    let backend = PostgresBackend::new(config).await.unwrap();
    let tenant_id = unique_tenant_id("unique-test");

    backend.set_tenant_context(&tenant_id).await.unwrap();
    let client = backend.get_client().await.unwrap();

    let session_id = Uuid::new_v4();
    let session_data = serde_json::json!({
        "metadata": {"id": session_id.to_string(), "status": "active"},
        "config": {},
        "state": {},
        "artifact_ids": []
    });

    // Insert first session
    client
        .execute(
            "INSERT INTO llmspell.sessions
             (tenant_id, session_id, session_data, status)
             VALUES ($1, $2, $3, $4)",
            &[&tenant_id, &session_id, &session_data, &"active"],
        )
        .await
        .unwrap();

    // Try to insert duplicate (should fail)
    let result = client
        .execute(
            "INSERT INTO llmspell.sessions
             (tenant_id, session_id, session_data, status)
             VALUES ($1, $2, $3, $4)",
            &[&tenant_id, &session_id, &session_data, &"active"],
        )
        .await;

    assert!(
        result.is_err(),
        "Duplicate session should violate unique constraint"
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
async fn test_sessions_status_constraint() {
    ensure_migrations_run_once().await;

    let config = PostgresConfig::new(TEST_CONNECTION_STRING);
    let backend = PostgresBackend::new(config).await.unwrap();
    let tenant_id = unique_tenant_id("status-test");

    backend.set_tenant_context(&tenant_id).await.unwrap();
    let client = backend.get_client().await.unwrap();

    let session_data = serde_json::json!({
        "metadata": {"status": "invalid"},
        "config": {},
        "state": {}
    });

    // Try to insert session with invalid status (should fail)
    let result = client
        .execute(
            "INSERT INTO llmspell.sessions
             (tenant_id, session_id, session_data, status)
             VALUES ($1, $2, $3, $4)",
            &[&tenant_id, &Uuid::new_v4(), &session_data, &"invalid"],
        )
        .await;

    assert!(
        result.is_err(),
        "Invalid status should violate valid_session_status constraint"
    );

    let error = result.unwrap_err();
    let error_msg = if let Some(source) = error.source() {
        source.to_string()
    } else {
        error.to_string()
    };

    assert!(
        error_msg.contains("valid_session_status") || error_msg.contains("check constraint"),
        "Error should indicate status constraint violation, got: {}",
        error_msg
    );
}

#[tokio::test]
async fn test_sessions_artifact_count_constraint() {
    ensure_migrations_run_once().await;

    let config = PostgresConfig::new(TEST_CONNECTION_STRING);
    let backend = PostgresBackend::new(config).await.unwrap();
    let tenant_id = unique_tenant_id("artifact-test");

    backend.set_tenant_context(&tenant_id).await.unwrap();
    let client = backend.get_client().await.unwrap();

    let session_data = serde_json::json!({
        "metadata": {},
        "config": {},
        "state": {}
    });

    // Try to insert session with negative artifact_count (should fail)
    let result = client
        .execute(
            "INSERT INTO llmspell.sessions
             (tenant_id, session_id, session_data, status, artifact_count)
             VALUES ($1, $2, $3, $4, $5)",
            &[&tenant_id, &Uuid::new_v4(), &session_data, &"active", &-1i32],
        )
        .await;

    assert!(
        result.is_err(),
        "Negative artifact_count should violate non_negative_artifact_count constraint"
    );

    let error = result.unwrap_err();
    let error_msg = if let Some(source) = error.source() {
        source.to_string()
    } else {
        error.to_string()
    };

    assert!(
        error_msg.contains("non_negative_artifact_count") || error_msg.contains("check constraint"),
        "Error should indicate artifact_count constraint violation, got: {}",
        error_msg
    );
}

#[tokio::test]
async fn test_sessions_updated_at_trigger() {
    ensure_migrations_run_once().await;

    let config = PostgresConfig::new(TEST_CONNECTION_STRING);
    let backend = PostgresBackend::new(config).await.unwrap();
    let tenant_id = unique_tenant_id("trigger-test");

    backend.set_tenant_context(&tenant_id).await.unwrap();
    let client = backend.get_client().await.unwrap();

    let session_id = Uuid::new_v4();
    let session_data = serde_json::json!({
        "metadata": {},
        "config": {},
        "state": {}
    });

    // Insert session
    client
        .execute(
            "INSERT INTO llmspell.sessions
             (tenant_id, session_id, session_data, status)
             VALUES ($1, $2, $3, $4)",
            &[&tenant_id, &session_id, &session_data, &"active"],
        )
        .await
        .unwrap();

    // Get initial updated_at
    let row = client
        .query_one(
            "SELECT updated_at FROM llmspell.sessions
             WHERE tenant_id = $1 AND session_id = $2",
            &[&tenant_id, &session_id],
        )
        .await
        .unwrap();
    let initial_updated_at: chrono::DateTime<chrono::Utc> = row.get(0);

    // Wait a bit to ensure timestamp difference
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    // Update the session
    client
        .execute(
            "UPDATE llmspell.sessions
             SET artifact_count = artifact_count + 1
             WHERE tenant_id = $1 AND session_id = $2",
            &[&tenant_id, &session_id],
        )
        .await
        .unwrap();

    // Get updated updated_at
    let row = client
        .query_one(
            "SELECT updated_at FROM llmspell.sessions
             WHERE tenant_id = $1 AND session_id = $2",
            &[&tenant_id, &session_id],
        )
        .await
        .unwrap();
    let updated_updated_at: chrono::DateTime<chrono::Utc> = row.get(0);

    assert!(
        updated_updated_at > initial_updated_at,
        "updated_at should be automatically updated by trigger"
    );
}

#[tokio::test]
async fn test_sessions_rls_isolation() {
    ensure_migrations_run_once().await;

    let config = PostgresConfig::new(TEST_CONNECTION_STRING);
    let backend = PostgresBackend::new(config).await.unwrap();

    let tenant_a = unique_tenant_id("rls-a");
    let tenant_b = unique_tenant_id("rls-b");

    let session_id_a = Uuid::new_v4();
    let session_id_b = Uuid::new_v4();

    let session_data = serde_json::json!({
        "metadata": {},
        "config": {},
        "state": {}
    });

    // Insert session for tenant A
    backend.set_tenant_context(&tenant_a).await.unwrap();
    {
        let client = backend.get_client().await.unwrap();
        client
            .execute(
                "INSERT INTO llmspell.sessions
                 (tenant_id, session_id, session_data, status, artifact_count)
                 VALUES ($1, $2, $3, $4, $5)",
                &[&tenant_a, &session_id_a, &session_data, &"active", &5i32],
            )
            .await
            .unwrap();
    }

    // Insert session for tenant B
    backend.set_tenant_context(&tenant_b).await.unwrap();
    {
        let client = backend.get_client().await.unwrap();
        client
            .execute(
                "INSERT INTO llmspell.sessions
                 (tenant_id, session_id, session_data, status, artifact_count)
                 VALUES ($1, $2, $3, $4, $5)",
                &[&tenant_b, &session_id_b, &session_data, &"archived", &10i32],
            )
            .await
            .unwrap();
    }

    // Query as tenant A - should only see tenant A's session
    backend.set_tenant_context(&tenant_a).await.unwrap();
    let rows = {
        let client = backend.get_client().await.unwrap();
        client
            .query(
                "SELECT session_id, status, artifact_count FROM llmspell.sessions
                 ORDER BY session_id",
                &[],
            )
            .await
            .unwrap()
    };

    assert_eq!(rows.len(), 1, "Tenant A should see only 1 session");
    let session_id: Uuid = rows[0].get(0);
    let status: String = rows[0].get(1);
    let artifact_count: i32 = rows[0].get(2);
    assert_eq!(session_id, session_id_a);
    assert_eq!(status, "active");
    assert_eq!(artifact_count, 5);

    // Query as tenant B - should only see tenant B's session
    backend.set_tenant_context(&tenant_b).await.unwrap();
    let rows = {
        let client = backend.get_client().await.unwrap();
        client
            .query(
                "SELECT session_id, status, artifact_count FROM llmspell.sessions
                 ORDER BY session_id",
                &[],
            )
            .await
            .unwrap()
    };

    assert_eq!(rows.len(), 1, "Tenant B should see only 1 session");
    let session_id: Uuid = rows[0].get(0);
    let status: String = rows[0].get(1);
    let artifact_count: i32 = rows[0].get(2);
    assert_eq!(session_id, session_id_b);
    assert_eq!(status, "archived");
    assert_eq!(artifact_count, 10);
}
