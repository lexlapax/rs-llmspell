//! Tests for RLS test table creation and policy enforcement (Phase 13b.3.2)

#![cfg(feature = "postgres")]

use llmspell_storage::{PostgresBackend, PostgresConfig};
use tokio::sync::OnceCell;

const TEST_CONNECTION_STRING: &str =
    "postgresql://llmspell:llmspell_dev_pass@localhost:5432/llmspell_dev";

static MIGRATION_INIT: OnceCell<()> = OnceCell::const_new();

/// Ensure migrations are run exactly once before any tests
async fn ensure_migrations_run_once() {
    MIGRATION_INIT
        .get_or_init(|| async {
            let config = PostgresConfig::new(TEST_CONNECTION_STRING);
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
async fn test_migration_creates_test_data_table() {
    ensure_migrations_run_once().await;

    let config = PostgresConfig::new(TEST_CONNECTION_STRING);
    let backend = PostgresBackend::new(config)
        .await
        .expect("Failed to create backend");

    // Verify migration version is at least 2
    let version = backend
        .migration_version()
        .await
        .expect("Failed to get migration version");

    assert!(
        version >= 2,
        "Migration version should be at least 2 after running V2 migration"
    );

    // Query to check if table exists and has RLS enabled
    let client = backend.get_client().await.expect("Failed to get client");

    let row = client
        .query_one(
            "SELECT relrowsecurity FROM pg_class WHERE relname = 'test_data'",
            &[],
        )
        .await
        .expect("Failed to query table RLS status");

    let rls_enabled: bool = row.get(0);
    assert!(rls_enabled, "RLS should be enabled on test_data table");
}

#[tokio::test]
async fn test_test_data_table_has_four_policies() {
    ensure_migrations_run_once().await;

    let config = PostgresConfig::new(TEST_CONNECTION_STRING);
    let backend = PostgresBackend::new(config)
        .await
        .expect("Failed to create backend");

    // Query policies for test_data table
    let client = backend.get_client().await.expect("Failed to get client");

    let rows = client
        .query(
            "SELECT policyname, cmd FROM pg_policies WHERE tablename = 'test_data' ORDER BY policyname",
            &[],
        )
        .await
        .expect("Failed to query policies");

    assert_eq!(rows.len(), 4, "test_data table should have 4 RLS policies");

    // Verify all four policy types exist
    let policy_names: Vec<String> = rows.iter().map(|r| r.get(0)).collect();
    assert!(
        policy_names.contains(&"tenant_isolation_select".to_string()),
        "Should have SELECT policy"
    );
    assert!(
        policy_names.contains(&"tenant_isolation_insert".to_string()),
        "Should have INSERT policy"
    );
    assert!(
        policy_names.contains(&"tenant_isolation_update".to_string()),
        "Should have UPDATE policy"
    );
    assert!(
        policy_names.contains(&"tenant_isolation_delete".to_string()),
        "Should have DELETE policy"
    );
}

#[tokio::test]
async fn test_test_data_table_has_tenant_id_index() {
    ensure_migrations_run_once().await;

    let config = PostgresConfig::new(TEST_CONNECTION_STRING);
    let backend = PostgresBackend::new(config)
        .await
        .expect("Failed to create backend");

    // Query indexes for test_data table
    let client = backend.get_client().await.expect("Failed to get client");

    let rows = client
        .query(
            "SELECT indexname FROM pg_indexes WHERE tablename = 'test_data' AND indexname = 'idx_test_data_tenant'",
            &[],
        )
        .await
        .expect("Failed to query indexes");

    assert_eq!(rows.len(), 1, "test_data table should have tenant_id index");
}

#[tokio::test]
async fn test_can_insert_and_query_test_data() {
    ensure_migrations_run_once().await;

    let config = PostgresConfig::new(TEST_CONNECTION_STRING);
    let backend = PostgresBackend::new(config)
        .await
        .expect("Failed to create backend");

    // Set tenant context
    backend
        .set_tenant_context("test-tenant-123")
        .await
        .expect("Failed to set tenant context");

    // Insert test data
    let client = backend.get_client().await.expect("Failed to get client");

    client
        .execute(
            "INSERT INTO llmspell.test_data (tenant_id, value) VALUES ($1, $2)",
            &[&"test-tenant-123", &"test value"],
        )
        .await
        .expect("Failed to insert test data");

    // Query back (should only see rows for current tenant due to RLS)
    let rows = client
        .query("SELECT value FROM llmspell.test_data", &[])
        .await
        .expect("Failed to query test data");

    assert_eq!(rows.len(), 1, "Should see 1 row for current tenant");
    let value: String = rows[0].get(0);
    assert_eq!(value, "test value");

    // Clean up
    client
        .execute(
            "DELETE FROM llmspell.test_data WHERE tenant_id = $1",
            &[&"test-tenant-123"],
        )
        .await
        .expect("Failed to clean up test data");
}
