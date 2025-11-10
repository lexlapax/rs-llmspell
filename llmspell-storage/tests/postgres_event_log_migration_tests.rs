//! Integration tests for Event Log migration (Phase 13b.11.1)
//!
//! Tests V11__event_log.sql migration:
//! - Partitioned table creation
//! - Automatic partition creation trigger
//! - RLS policies for tenant isolation
//! - Indexes for EventStorage queries
//! - Partition management functions

#![cfg(feature = "postgres")]

use llmspell_storage::{PostgresBackend, PostgresConfig};
use tokio::sync::OnceCell;
use uuid::Uuid;

// Admin connection for migrations (llmspell user has CREATE TABLE privileges)
const ADMIN_CONNECTION_STRING: &str =
    "postgresql://llmspell:llmspell_dev_pass@localhost:5432/llmspell_dev";

// Application connection for queries (llmspell_app enforces RLS, no schema modification)
const APP_CONNECTION_STRING: &str =
    "postgresql://llmspell_app:llmspell_app_pass@localhost:5432/llmspell_dev";

static MIGRATION_INIT: OnceCell<()> = OnceCell::const_new();

/// Ensure migrations run once before all tests (uses admin user for DDL operations)
async fn ensure_migrations_run_once() {
    MIGRATION_INIT
        .get_or_init(|| async {
            let config = PostgresConfig::new(ADMIN_CONNECTION_STRING);
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

#[tokio::test]
async fn test_event_log_table_exists() {
    ensure_migrations_run_once().await;

    let config = PostgresConfig::new(APP_CONNECTION_STRING);
    let backend = PostgresBackend::new(config).await.unwrap();
    let client = backend.get_client().await.unwrap();

    let row = client
        .query_one(
            "SELECT EXISTS (
                SELECT 1 FROM information_schema.tables
                WHERE table_schema = 'llmspell'
                AND table_name = 'event_log'
            )",
            &[],
        )
        .await
        .unwrap();

    let exists: bool = row.get(0);
    assert!(exists, "event_log table should exist");
}

#[tokio::test]
async fn test_event_log_initial_partitions_created() {
    ensure_migrations_run_once().await;

    let config = PostgresConfig::new(APP_CONNECTION_STRING);
    let backend = PostgresBackend::new(config).await.unwrap();
    let client = backend.get_client().await.unwrap();

    let rows = client
        .query(
            "SELECT tablename FROM pg_tables
             WHERE schemaname = 'llmspell'
             AND tablename LIKE 'event_log_%'
             ORDER BY tablename",
            &[],
        )
        .await
        .unwrap();

    assert!(
        rows.len() >= 4,
        "Should have at least 4 event_log partitions, found {}",
        rows.len()
    );

    for row in rows {
        let tablename: String = row.get(0);
        assert!(
            tablename.starts_with("event_log_"),
            "Partition name should start with event_log_"
        );
        assert!(
            tablename.len() == 17,
            "Partition name should be in format event_log_YYYY_MM"
        );
    }
}

#[tokio::test]
async fn test_event_log_indexes_created() {
    ensure_migrations_run_once().await;

    let config = PostgresConfig::new(APP_CONNECTION_STRING);
    let backend = PostgresBackend::new(config).await.unwrap();
    let client = backend.get_client().await.unwrap();

    let expected_indexes = vec![
        "idx_event_log_correlation",
        "idx_event_log_type",
        "idx_event_log_sequence",
        "idx_event_log_tenant_time",
        "idx_event_log_payload",
    ];

    for index_name in expected_indexes {
        let row = client
            .query_one(
                "SELECT EXISTS (
                    SELECT 1 FROM pg_indexes
                    WHERE schemaname = 'llmspell'
                    AND indexname = $1
                )",
                &[&index_name],
            )
            .await
            .unwrap();

        let exists: bool = row.get(0);
        assert!(exists, "Index {} should exist", index_name);
    }
}

#[tokio::test]
async fn test_event_log_rls_policies_enabled() {
    ensure_migrations_run_once().await;

    let config = PostgresConfig::new(APP_CONNECTION_STRING);
    let backend = PostgresBackend::new(config).await.unwrap();
    let client = backend.get_client().await.unwrap();

    let row = client
        .query_one(
            "SELECT relrowsecurity
             FROM pg_class c
             JOIN pg_namespace n ON n.oid = c.relnamespace
             WHERE n.nspname = 'llmspell'
             AND c.relname = 'event_log'",
            &[],
        )
        .await
        .unwrap();

    let rls_enabled: bool = row.get(0);
    assert!(rls_enabled, "RLS should be enabled on event_log table");

    let expected_policies = vec![
        "tenant_isolation_select",
        "tenant_isolation_insert",
        "tenant_isolation_update",
        "tenant_isolation_delete",
    ];

    for policy_name in expected_policies {
        let row = client
            .query_one(
                "SELECT EXISTS (
                    SELECT 1 FROM pg_policies
                    WHERE schemaname = 'llmspell'
                    AND tablename = 'event_log'
                    AND policyname = $1
                )",
                &[&policy_name],
            )
            .await
            .unwrap();

        let exists: bool = row.get(0);
        assert!(exists, "Policy {} should exist", policy_name);
    }
}

#[tokio::test]
async fn test_event_log_partition_management_functions_exist() {
    ensure_migrations_run_once().await;

    let config = PostgresConfig::new(APP_CONNECTION_STRING);
    let backend = PostgresBackend::new(config).await.unwrap();
    let client = backend.get_client().await.unwrap();

    let expected_functions = vec![
        "create_event_log_partition",
        "ensure_future_event_log_partitions",
        "cleanup_old_event_log_partitions",
    ];

    for function_name in expected_functions {
        let row = client
            .query_one(
                "SELECT EXISTS (
                    SELECT 1 FROM pg_proc p
                    JOIN pg_namespace n ON n.oid = p.pronamespace
                    WHERE n.nspname = 'llmspell'
                    AND p.proname = $1
                )",
                &[&function_name],
            )
            .await
            .unwrap();

        let exists: bool = row.get(0);
        assert!(exists, "Function {} should exist", function_name);
    }
}

#[tokio::test]
async fn test_event_log_partition_maintenance_workflow() {
    ensure_migrations_run_once().await;

    let config = PostgresConfig::new(APP_CONNECTION_STRING);
    let backend = PostgresBackend::new(config).await.unwrap();
    let client = backend.get_client().await.unwrap();

    // Call ensure_future_event_log_partitions() to create more partitions
    let row = client
        .query_one("SELECT llmspell.ensure_future_event_log_partitions()", &[])
        .await
        .unwrap();

    let results: Vec<String> = row.get(0);

    // Should return results for 4 partitions (current + 3 months)
    assert_eq!(results.len(), 4, "Should create 4 partition entries");

    // Calling again should skip existing partitions
    let row = client
        .query_one("SELECT llmspell.ensure_future_event_log_partitions()", &[])
        .await
        .unwrap();

    let results: Vec<String> = row.get(0);

    // All should be SKIPPED since they already exist
    for result in &results {
        assert!(
            result.contains("SKIPPED"),
            "Should skip existing partitions: {}",
            result
        );
    }
}

#[tokio::test]
async fn test_event_log_create_partition_function() {
    ensure_migrations_run_once().await;

    let config = PostgresConfig::new(APP_CONNECTION_STRING);
    let backend = PostgresBackend::new(config).await.unwrap();
    let client = backend.get_client().await.unwrap();

    let row = client
        .query_one(
            "SELECT llmspell.create_event_log_partition(
                '2026-01-01'::TIMESTAMPTZ,
                '2026-02-01'::TIMESTAMPTZ
            )",
            &[],
        )
        .await
        .unwrap();

    let result: String = row.get(0);
    assert!(
        result.contains("CREATED") || result.contains("SKIPPED"),
        "Should create or skip partition: {}",
        result
    );

    let row = client
        .query_one(
            "SELECT EXISTS (
                SELECT 1 FROM pg_tables
                WHERE schemaname = 'llmspell'
                AND tablename = 'event_log_2026_01'
            )",
            &[],
        )
        .await
        .unwrap();

    let exists: bool = row.get(0);
    assert!(exists, "event_log_2026_01 partition should exist");
}

#[tokio::test]
async fn test_event_log_insert_with_manual_partition_creation() {
    ensure_migrations_run_once().await;

    let config = PostgresConfig::new(APP_CONNECTION_STRING);
    let backend = PostgresBackend::new(config).await.unwrap();
    backend
        .set_tenant_context("test_tenant_event_log")
        .await
        .unwrap();

    let client = backend.get_client().await.unwrap();

    // Manually create partition for 2027-01
    client
        .query_one(
            "SELECT llmspell.create_event_log_partition(
                '2027-01-01'::TIMESTAMPTZ,
                '2027-02-01'::TIMESTAMPTZ
            )",
            &[],
        )
        .await
        .unwrap();

    let event_id = Uuid::new_v4();
    let correlation_id = Uuid::new_v4();
    let timestamp = "2027-01-15T12:00:00Z"
        .parse::<chrono::DateTime<chrono::Utc>>()
        .unwrap();
    let payload = serde_json::json!({
        "id": event_id,
        "event_type": "test.event",
        "data": {"message": "test"},
        "language": "rust",
        "timestamp": timestamp.to_rfc3339(),
        "sequence": 1,
        "metadata": {
            "correlation_id": correlation_id,
            "source": "test",
            "tags": [],
            "priority": 0
        },
        "schema_version": "1.0"
    });

    let result = client
        .execute(
            "INSERT INTO llmspell.event_log
             (tenant_id, event_id, event_type, correlation_id, timestamp, sequence, language, payload)
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8)",
            &[
                &"test_tenant_event_log",
                &event_id,
                &"test.event",
                &correlation_id,
                &timestamp,
                &1i64,
                &"rust",
                &payload,
            ],
        )
        .await;

    assert!(result.is_ok(), "Insert should succeed: {:?}", result.err());

    let row = client
        .query_one(
            "SELECT EXISTS (
                SELECT 1 FROM pg_tables
                WHERE schemaname = 'llmspell'
                AND tablename = 'event_log_2027_01'
            )",
            &[],
        )
        .await
        .unwrap();

    let exists: bool = row.get(0);
    assert!(exists, "event_log_2027_01 partition should exist");

    client
        .execute(
            "DELETE FROM llmspell.event_log WHERE tenant_id = $1 AND event_id = $2",
            &[&"test_tenant_event_log", &event_id],
        )
        .await
        .unwrap();
}

#[tokio::test]
async fn test_event_log_rls_tenant_isolation() {
    ensure_migrations_run_once().await;

    let config = PostgresConfig::new(APP_CONNECTION_STRING);
    let backend = PostgresBackend::new(config).await.unwrap();

    // Use unique tenant IDs per test run to avoid pollution
    let tenant1 = format!("test_rls_{}", Uuid::new_v4());
    let tenant2 = format!("test_rls_{}", Uuid::new_v4());

    let event_id_1 = Uuid::new_v4();
    let event_id_2 = Uuid::new_v4();
    let correlation_id = Uuid::new_v4();
    let timestamp = chrono::Utc::now();

    // Insert event for tenant1
    backend.set_tenant_context(&tenant1).await.unwrap();
    let client1 = backend.get_client().await.unwrap();

    let payload = serde_json::json!({
        "id": event_id_1,
        "event_type": "test.event",
        "data": {"message": "test"},
        "language": "rust",
        "timestamp": timestamp.to_rfc3339(),
        "sequence": 1,
        "metadata": {
            "correlation_id": correlation_id,
            "source": "test",
            "tags": [],
            "priority": 0
        },
        "schema_version": "1.0"
    });

    client1
        .execute(
            "INSERT INTO llmspell.event_log
             (tenant_id, event_id, event_type, correlation_id, timestamp, sequence, language, payload)
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8)",
            &[
                &tenant1,
                &event_id_1,
                &"test.event",
                &correlation_id,
                &timestamp,
                &1i64,
                &"rust",
                &payload,
            ],
        )
        .await
        .unwrap();

    // Insert event for tenant2
    backend.set_tenant_context(&tenant2).await.unwrap();
    let client2 = backend.get_client().await.unwrap();

    let payload2 = serde_json::json!({
        "id": event_id_2,
        "event_type": "test.event",
        "data": {"message": "test"},
        "language": "rust",
        "timestamp": timestamp.to_rfc3339(),
        "sequence": 2,
        "metadata": {
            "correlation_id": correlation_id,
            "source": "test",
            "tags": [],
            "priority": 0
        },
        "schema_version": "1.0"
    });

    client2
        .execute(
            "INSERT INTO llmspell.event_log
             (tenant_id, event_id, event_type, correlation_id, timestamp, sequence, language, payload)
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8)",
            &[
                &tenant2,
                &event_id_2,
                &"test.event",
                &correlation_id,
                &timestamp,
                &2i64,
                &"rust",
                &payload2,
            ],
        )
        .await
        .unwrap();

    // Query as tenant1 - should only see tenant1's event
    backend.set_tenant_context(&tenant1).await.unwrap();
    let client3 = backend.get_client().await.unwrap();

    let rows = client3
        .query(
            "SELECT event_id FROM llmspell.event_log WHERE correlation_id = $1",
            &[&correlation_id],
        )
        .await
        .unwrap();

    assert_eq!(rows.len(), 1, "Should only see tenant1's event");
    let retrieved_event_id: Uuid = rows[0].get(0);
    assert_eq!(
        retrieved_event_id, event_id_1,
        "Should retrieve tenant1's event"
    );

    // Query as tenant2 - should only see tenant2's event
    backend.set_tenant_context(&tenant2).await.unwrap();
    let client4 = backend.get_client().await.unwrap();

    let rows = client4
        .query(
            "SELECT event_id FROM llmspell.event_log WHERE correlation_id = $1",
            &[&correlation_id],
        )
        .await
        .unwrap();

    assert_eq!(rows.len(), 1, "Should only see tenant2's event");
    let retrieved_event_id: Uuid = rows[0].get(0);
    assert_eq!(
        retrieved_event_id, event_id_2,
        "Should retrieve tenant2's event"
    );

    // Cleanup - delete as tenant1, then as tenant2
    backend.set_tenant_context(&tenant1).await.unwrap();
    let client5 = backend.get_client().await.unwrap();
    let _ = client5
        .execute(
            "DELETE FROM llmspell.event_log WHERE event_id = $1",
            &[&event_id_1],
        )
        .await;

    backend.set_tenant_context(&tenant2).await.unwrap();
    let client6 = backend.get_client().await.unwrap();
    let _ = client6
        .execute(
            "DELETE FROM llmspell.event_log WHERE event_id = $1",
            &[&event_id_2],
        )
        .await;
}

#[tokio::test]
async fn test_event_log_table_schema() {
    ensure_migrations_run_once().await;

    let config = PostgresConfig::new(APP_CONNECTION_STRING);
    let backend = PostgresBackend::new(config).await.unwrap();
    let client = backend.get_client().await.unwrap();

    let rows = client
        .query(
            "SELECT column_name, data_type, is_nullable
             FROM information_schema.columns
             WHERE table_schema = 'llmspell'
             AND table_name = 'event_log'
             ORDER BY ordinal_position",
            &[],
        )
        .await
        .unwrap();

    let expected_columns = [
        ("tenant_id", "character varying", "NO"),
        ("event_id", "uuid", "NO"),
        ("event_type", "character varying", "NO"),
        ("correlation_id", "uuid", "NO"),
        ("timestamp", "timestamp with time zone", "NO"),
        ("sequence", "bigint", "NO"),
        ("language", "character varying", "NO"),
        ("payload", "jsonb", "NO"),
    ];

    assert_eq!(
        rows.len(),
        expected_columns.len(),
        "Should have {} columns",
        expected_columns.len()
    );

    for (i, (expected_name, expected_type, expected_nullable)) in
        expected_columns.iter().enumerate()
    {
        let column_name: String = rows[i].get(0);
        let data_type: String = rows[i].get(1);
        let is_nullable: String = rows[i].get(2);

        assert_eq!(&column_name, expected_name, "Column {} name mismatch", i);
        assert_eq!(&data_type, expected_type, "Column {} type mismatch", i);
        assert_eq!(
            &is_nullable, expected_nullable,
            "Column {} nullable mismatch",
            i
        );
    }
}
