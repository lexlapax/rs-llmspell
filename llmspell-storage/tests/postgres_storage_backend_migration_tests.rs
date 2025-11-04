//! Tests for PostgreSQL Storage Backend Migrations (Phase 13b.7.1)
//!
//! Verifies V6__agent_state.sql and V7__kv_store.sql migrations:
//! - Table creation with correct schemas
//! - Index creation and optimization
//! - RLS policy enforcement
//! - Trigger functionality (auto-update timestamps, version incrementing)
//! - Constraint enforcement (unique keys, check constraints)
//! - Tenant isolation

#![cfg(feature = "postgres")]

use llmspell_storage::backends::postgres::{PostgresBackend, PostgresConfig};
use std::error::Error;
use std::sync::Arc;
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

// ============================================================================
// V6: Agent States Table Tests
// ============================================================================

#[tokio::test]
async fn test_agent_states_table_created() {
    ensure_migrations_run_once().await;

    let config = PostgresConfig::new(SUPERUSER_CONNECTION_STRING);
    let backend = PostgresBackend::new(config).await.unwrap();
    let client = backend.get_client().await.unwrap();

    // Verify table exists
    let row = client
        .query_one(
            "SELECT COUNT(*) FROM pg_tables
             WHERE schemaname = 'llmspell' AND tablename = 'agent_states'",
            &[],
        )
        .await
        .unwrap();
    let count: i64 = row.get(0);
    assert_eq!(count, 1, "agent_states table should exist");

    // Verify RLS is enabled
    let row = client
        .query_one(
            "SELECT relrowsecurity FROM pg_class
             WHERE relname = 'agent_states' AND relnamespace = 'llmspell'::regnamespace",
            &[],
        )
        .await
        .unwrap();
    let rls_enabled: bool = row.get(0);
    assert!(rls_enabled, "RLS should be enabled on agent_states");
}

#[tokio::test]
async fn test_agent_states_indexes_created() {
    ensure_migrations_run_once().await;

    let config = PostgresConfig::new(SUPERUSER_CONNECTION_STRING);
    let backend = PostgresBackend::new(config).await.unwrap();
    let client = backend.get_client().await.unwrap();

    // Get all indexes for agent_states
    let rows = client
        .query(
            "SELECT indexname
             FROM pg_indexes
             WHERE schemaname = 'llmspell'
               AND tablename = 'agent_states'
               AND indexname LIKE 'idx_%'
             ORDER BY indexname",
            &[],
        )
        .await
        .unwrap();

    // Should have 6 indexes
    assert_eq!(rows.len(), 6, "Should have 6 indexes");

    let expected_indexes = [
        "idx_agent_states_data_gin",
        "idx_agent_states_execution_state",
        "idx_agent_states_metadata_name",
        "idx_agent_states_tenant",
        "idx_agent_states_type",
        "idx_agent_states_updated",
    ];

    let index_names: Vec<String> = rows.iter().map(|r| r.get("indexname")).collect();

    // Check all expected indexes exist (order may vary)
    for expected in &expected_indexes {
        assert!(
            index_names.contains(&expected.to_string()),
            "Index {} should exist",
            expected
        );
    }
}

#[tokio::test]
async fn test_agent_states_rls_policies() {
    ensure_migrations_run_once().await;

    let config = PostgresConfig::new(SUPERUSER_CONNECTION_STRING);
    let backend = PostgresBackend::new(config).await.unwrap();
    let client = backend.get_client().await.unwrap();

    // Verify all 4 RLS policies exist
    let rows = client
        .query(
            "SELECT policyname, cmd
             FROM pg_policies
             WHERE schemaname = 'llmspell' AND tablename = 'agent_states'
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
async fn test_agent_states_unique_constraint() {
    ensure_migrations_run_once().await;

    let config = PostgresConfig::new(TEST_CONNECTION_STRING);
    let backend = Arc::new(PostgresBackend::new(config).await.unwrap());
    let tenant_id = unique_tenant_id("unique-agent");

    backend.set_tenant_context(&tenant_id).await.unwrap();
    let client = backend.get_client().await.unwrap();

    let agent_id = "test-agent-123";
    let state_data = serde_json::json!({"test": "data"});

    // Insert first agent state
    client
        .execute(
            "INSERT INTO llmspell.agent_states
             (tenant_id, agent_id, agent_type, state_data, checksum)
             VALUES ($1, $2, $3, $4, $5)",
            &[
                &tenant_id,
                &agent_id,
                &"assistant",
                &state_data,
                &"checksum1",
            ],
        )
        .await
        .unwrap();

    // Try to insert duplicate (should fail)
    let result = client
        .execute(
            "INSERT INTO llmspell.agent_states
             (tenant_id, agent_id, agent_type, state_data, checksum)
             VALUES ($1, $2, $3, $4, $5)",
            &[
                &tenant_id,
                &agent_id,
                &"assistant",
                &state_data,
                &"checksum2",
            ],
        )
        .await;

    assert!(
        result.is_err(),
        "Duplicate agent state should violate unique constraint"
    );

    let error = result.unwrap_err();
    let error_msg = if let Some(source) = error.source() {
        source.to_string()
    } else {
        error.to_string()
    };

    assert!(
        error_msg.contains("unique") || error_msg.contains("duplicate key"),
        "Error should mention unique constraint, got: {}",
        error_msg
    );
}

#[tokio::test]
async fn test_agent_states_version_increment_trigger() {
    ensure_migrations_run_once().await;

    let config = PostgresConfig::new(TEST_CONNECTION_STRING);
    let backend = Arc::new(PostgresBackend::new(config).await.unwrap());
    let tenant_id = unique_tenant_id("version-test");

    backend.set_tenant_context(&tenant_id).await.unwrap();
    let client = backend.get_client().await.unwrap();

    let agent_id = "version-agent";
    let state_data = serde_json::json!({"version": 1});

    // Insert agent state
    client
        .execute(
            "INSERT INTO llmspell.agent_states
             (tenant_id, agent_id, agent_type, state_data, checksum)
             VALUES ($1, $2, $3, $4, $5)",
            &[
                &tenant_id,
                &agent_id,
                &"assistant",
                &state_data,
                &"checksum",
            ],
        )
        .await
        .unwrap();

    // Get initial version
    let row = client
        .query_one(
            "SELECT data_version FROM llmspell.agent_states
             WHERE tenant_id = $1 AND agent_id = $2",
            &[&tenant_id, &agent_id],
        )
        .await
        .unwrap();
    let initial_version: i32 = row.get(0);
    assert_eq!(initial_version, 1, "Initial data_version should be 1");

    // Update state_data
    let new_state_data = serde_json::json!({"version": 2});
    client
        .execute(
            "UPDATE llmspell.agent_states
             SET state_data = $1
             WHERE tenant_id = $2 AND agent_id = $3",
            &[&new_state_data, &tenant_id, &agent_id],
        )
        .await
        .unwrap();

    // Verify version incremented
    let row = client
        .query_one(
            "SELECT data_version FROM llmspell.agent_states
             WHERE tenant_id = $1 AND agent_id = $2",
            &[&tenant_id, &agent_id],
        )
        .await
        .unwrap();
    let new_version: i32 = row.get(0);
    assert_eq!(
        new_version, 2,
        "data_version should auto-increment on state_data change"
    );
}

#[tokio::test]
async fn test_agent_states_updated_at_trigger() {
    ensure_migrations_run_once().await;

    let config = PostgresConfig::new(TEST_CONNECTION_STRING);
    let backend = Arc::new(PostgresBackend::new(config).await.unwrap());
    let tenant_id = unique_tenant_id("updated-at-test");

    backend.set_tenant_context(&tenant_id).await.unwrap();
    let client = backend.get_client().await.unwrap();

    let agent_id = "timestamp-agent";
    let state_data = serde_json::json!({"test": "data"});

    // Insert agent state
    client
        .execute(
            "INSERT INTO llmspell.agent_states
             (tenant_id, agent_id, agent_type, state_data, checksum)
             VALUES ($1, $2, $3, $4, $5)",
            &[
                &tenant_id,
                &agent_id,
                &"assistant",
                &state_data,
                &"checksum",
            ],
        )
        .await
        .unwrap();

    // Get initial updated_at
    let row = client
        .query_one(
            "SELECT updated_at FROM llmspell.agent_states
             WHERE tenant_id = $1 AND agent_id = $2",
            &[&tenant_id, &agent_id],
        )
        .await
        .unwrap();
    let initial_updated_at: chrono::DateTime<chrono::Utc> = row.get(0);

    // Wait a bit
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    // Update the state
    client
        .execute(
            "UPDATE llmspell.agent_states
             SET checksum = 'new-checksum'
             WHERE tenant_id = $1 AND agent_id = $2",
            &[&tenant_id, &agent_id],
        )
        .await
        .unwrap();

    // Get updated updated_at
    let row = client
        .query_one(
            "SELECT updated_at FROM llmspell.agent_states
             WHERE tenant_id = $1 AND agent_id = $2",
            &[&tenant_id, &agent_id],
        )
        .await
        .unwrap();
    let new_updated_at: chrono::DateTime<chrono::Utc> = row.get(0);

    assert!(
        new_updated_at > initial_updated_at,
        "updated_at should be automatically updated by trigger"
    );
}

// ============================================================================
// V7: KV Store Table Tests
// ============================================================================

#[tokio::test]
async fn test_kv_store_table_created() {
    ensure_migrations_run_once().await;

    let config = PostgresConfig::new(SUPERUSER_CONNECTION_STRING);
    let backend = PostgresBackend::new(config).await.unwrap();
    let client = backend.get_client().await.unwrap();

    // Verify table exists
    let row = client
        .query_one(
            "SELECT COUNT(*) FROM pg_tables
             WHERE schemaname = 'llmspell' AND tablename = 'kv_store'",
            &[],
        )
        .await
        .unwrap();
    let count: i64 = row.get(0);
    assert_eq!(count, 1, "kv_store table should exist");

    // Verify RLS is enabled
    let row = client
        .query_one(
            "SELECT relrowsecurity FROM pg_class
             WHERE relname = 'kv_store' AND relnamespace = 'llmspell'::regnamespace",
            &[],
        )
        .await
        .unwrap();
    let rls_enabled: bool = row.get(0);
    assert!(rls_enabled, "RLS should be enabled on kv_store");
}

#[tokio::test]
async fn test_kv_store_indexes_created() {
    ensure_migrations_run_once().await;

    let config = PostgresConfig::new(SUPERUSER_CONNECTION_STRING);
    let backend = PostgresBackend::new(config).await.unwrap();
    let client = backend.get_client().await.unwrap();

    // Get all indexes for kv_store
    let rows = client
        .query(
            "SELECT indexname
             FROM pg_indexes
             WHERE schemaname = 'llmspell'
               AND tablename = 'kv_store'
               AND indexname LIKE 'idx_%'
             ORDER BY indexname",
            &[],
        )
        .await
        .unwrap();

    // Should have 4 indexes
    assert_eq!(rows.len(), 4, "Should have 4 indexes");

    let expected_indexes = [
        "idx_kv_store_key_prefix",
        "idx_kv_store_metadata_gin",
        "idx_kv_store_tenant",
        "idx_kv_store_updated",
    ];

    for (i, row) in rows.iter().enumerate() {
        let indexname: String = row.get("indexname");
        assert_eq!(indexname, expected_indexes[i]);
    }
}

#[tokio::test]
async fn test_kv_store_rls_policies() {
    ensure_migrations_run_once().await;

    let config = PostgresConfig::new(SUPERUSER_CONNECTION_STRING);
    let backend = PostgresBackend::new(config).await.unwrap();
    let client = backend.get_client().await.unwrap();

    // Verify all 4 RLS policies exist
    let rows = client
        .query(
            "SELECT policyname, cmd
             FROM pg_policies
             WHERE schemaname = 'llmspell' AND tablename = 'kv_store'
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
async fn test_kv_store_unique_constraint() {
    ensure_migrations_run_once().await;

    let config = PostgresConfig::new(TEST_CONNECTION_STRING);
    let backend = Arc::new(PostgresBackend::new(config).await.unwrap());
    let tenant_id = unique_tenant_id("unique-kv");

    backend.set_tenant_context(&tenant_id).await.unwrap();
    let client = backend.get_client().await.unwrap();

    let key = "test:key:123";
    let value = b"test value";

    // Insert first key-value pair
    client
        .execute(
            "INSERT INTO llmspell.kv_store (tenant_id, key, value)
             VALUES ($1, $2, $3)",
            &[&tenant_id, &key, &&value[..]],
        )
        .await
        .unwrap();

    // Try to insert duplicate key (should fail)
    let result = client
        .execute(
            "INSERT INTO llmspell.kv_store (tenant_id, key, value)
             VALUES ($1, $2, $3)",
            &[&tenant_id, &key, &&b"different value"[..]],
        )
        .await;

    assert!(
        result.is_err(),
        "Duplicate key should violate unique constraint"
    );

    let error = result.unwrap_err();
    let error_msg = if let Some(source) = error.source() {
        source.to_string()
    } else {
        error.to_string()
    };

    assert!(
        error_msg.contains("unique") || error_msg.contains("duplicate key"),
        "Error should mention unique constraint, got: {}",
        error_msg
    );
}

#[tokio::test]
async fn test_kv_store_updated_at_trigger() {
    ensure_migrations_run_once().await;

    let config = PostgresConfig::new(TEST_CONNECTION_STRING);
    let backend = Arc::new(PostgresBackend::new(config).await.unwrap());
    let tenant_id = unique_tenant_id("kv-updated-at");

    backend.set_tenant_context(&tenant_id).await.unwrap();
    let client = backend.get_client().await.unwrap();

    let key = "timestamp:test";
    let value = b"initial value";

    // Insert key-value pair
    client
        .execute(
            "INSERT INTO llmspell.kv_store (tenant_id, key, value)
             VALUES ($1, $2, $3)",
            &[&tenant_id, &key, &&value[..]],
        )
        .await
        .unwrap();

    // Get initial updated_at
    let row = client
        .query_one(
            "SELECT updated_at FROM llmspell.kv_store
             WHERE tenant_id = $1 AND key = $2",
            &[&tenant_id, &key],
        )
        .await
        .unwrap();
    let initial_updated_at: chrono::DateTime<chrono::Utc> = row.get(0);

    // Wait a bit
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    // Update the value
    client
        .execute(
            "UPDATE llmspell.kv_store
             SET value = $1
             WHERE tenant_id = $2 AND key = $3",
            &[&&b"updated value"[..], &tenant_id, &key],
        )
        .await
        .unwrap();

    // Get updated updated_at
    let row = client
        .query_one(
            "SELECT updated_at FROM llmspell.kv_store
             WHERE tenant_id = $1 AND key = $2",
            &[&tenant_id, &key],
        )
        .await
        .unwrap();
    let new_updated_at: chrono::DateTime<chrono::Utc> = row.get(0);

    assert!(
        new_updated_at > initial_updated_at,
        "updated_at should be automatically updated by trigger"
    );
}

#[tokio::test]
async fn test_kv_store_metadata_support() {
    ensure_migrations_run_once().await;

    let config = PostgresConfig::new(TEST_CONNECTION_STRING);
    let backend = Arc::new(PostgresBackend::new(config).await.unwrap());
    let tenant_id = unique_tenant_id("kv-metadata");

    backend.set_tenant_context(&tenant_id).await.unwrap();
    let client = backend.get_client().await.unwrap();

    let key = "meta:test";
    let value = b"test value";
    let metadata = serde_json::json!({
        "content_type": "application/octet-stream",
        "size": 10,
        "custom": {
            "field1": "value1",
            "field2": 42
        }
    });

    // Insert with metadata
    client
        .execute(
            "INSERT INTO llmspell.kv_store (tenant_id, key, value, metadata)
             VALUES ($1, $2, $3, $4)",
            &[&tenant_id, &key, &&value[..], &metadata],
        )
        .await
        .unwrap();

    // Retrieve and verify metadata
    let row = client
        .query_one(
            "SELECT metadata FROM llmspell.kv_store
             WHERE tenant_id = $1 AND key = $2",
            &[&tenant_id, &key],
        )
        .await
        .unwrap();
    let retrieved_metadata: serde_json::Value = row.get(0);

    assert_eq!(retrieved_metadata, metadata);
}

#[tokio::test]
async fn test_tenant_isolation_across_both_tables() {
    ensure_migrations_run_once().await;

    let config = PostgresConfig::new(TEST_CONNECTION_STRING);
    let backend = Arc::new(PostgresBackend::new(config).await.unwrap());

    let tenant_a = unique_tenant_id("isolation-a");
    let tenant_b = unique_tenant_id("isolation-b");

    // Insert agent state for tenant A
    backend.set_tenant_context(&tenant_a).await.unwrap();
    {
        let client = backend.get_client().await.unwrap();
        client
            .execute(
                "INSERT INTO llmspell.agent_states
                 (tenant_id, agent_id, agent_type, state_data, checksum)
                 VALUES ($1, $2, $3, $4, $5)",
                &[
                    &tenant_a,
                    &"agent-a",
                    &"assistant",
                    &serde_json::json!({}),
                    &"check-a",
                ],
            )
            .await
            .unwrap();

        client
            .execute(
                "INSERT INTO llmspell.kv_store (tenant_id, key, value)
                 VALUES ($1, $2, $3)",
                &[&tenant_a, &"key-a", &&b"value-a"[..]],
            )
            .await
            .unwrap();
    }

    // Insert for tenant B
    backend.set_tenant_context(&tenant_b).await.unwrap();
    {
        let client = backend.get_client().await.unwrap();
        client
            .execute(
                "INSERT INTO llmspell.agent_states
                 (tenant_id, agent_id, agent_type, state_data, checksum)
                 VALUES ($1, $2, $3, $4, $5)",
                &[
                    &tenant_b,
                    &"agent-b",
                    &"reviewer",
                    &serde_json::json!({}),
                    &"check-b",
                ],
            )
            .await
            .unwrap();

        client
            .execute(
                "INSERT INTO llmspell.kv_store (tenant_id, key, value)
                 VALUES ($1, $2, $3)",
                &[&tenant_b, &"key-b", &&b"value-b"[..]],
            )
            .await
            .unwrap();
    }

    // Query as tenant A - should only see tenant A's data
    backend.set_tenant_context(&tenant_a).await.unwrap();
    {
        let client = backend.get_client().await.unwrap();

        let agent_rows = client
            .query("SELECT agent_id FROM llmspell.agent_states", &[])
            .await
            .unwrap();
        assert_eq!(agent_rows.len(), 1);
        let agent_id: String = agent_rows[0].get(0);
        assert_eq!(agent_id, "agent-a");

        let kv_rows = client
            .query("SELECT key FROM llmspell.kv_store", &[])
            .await
            .unwrap();
        assert_eq!(kv_rows.len(), 1);
        let key: String = kv_rows[0].get(0);
        assert_eq!(key, "key-a");
    }

    // Query as tenant B - should only see tenant B's data
    backend.set_tenant_context(&tenant_b).await.unwrap();
    {
        let client = backend.get_client().await.unwrap();

        let agent_rows = client
            .query("SELECT agent_id FROM llmspell.agent_states", &[])
            .await
            .unwrap();
        assert_eq!(agent_rows.len(), 1);
        let agent_id: String = agent_rows[0].get(0);
        assert_eq!(agent_id, "agent-b");

        let kv_rows = client
            .query("SELECT key FROM llmspell.kv_store", &[])
            .await
            .unwrap();
        assert_eq!(kv_rows.len(), 1);
        let key: String = kv_rows[0].get(0);
        assert_eq!(key, "key-b");
    }
}
