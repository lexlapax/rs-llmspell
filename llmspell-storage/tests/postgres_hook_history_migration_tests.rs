//! Migration tests for PostgreSQL Hook History Schema (Phase 13b.12.1)
//!
//! Tests V13__hook_history.sql migration:
//! - Table and column creation
//! - Index creation
//! - RLS policies
//! - Cleanup functions

#![cfg(feature = "postgres")]

use llmspell_storage::backends::postgres::{PostgresBackend, PostgresConfig};
use tokio::sync::OnceCell;
use uuid::Uuid;

// Admin connection for migrations (llmspell user has CREATE TABLE privileges)
const ADMIN_CONNECTION_STRING: &str =
    "postgresql://llmspell:llmspell_dev_pass@localhost:5432/llmspell_dev";

// Application connection for queries (llmspell_app enforces RLS)
const APP_CONNECTION_STRING: &str =
    "postgresql://llmspell_app:llmspell_app_pass@localhost:5432/llmspell_dev";

static MIGRATION_INIT: OnceCell<()> = OnceCell::const_new();

/// Ensure migrations run once before all tests
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
async fn test_hook_history_table_exists() {
    ensure_migrations_run_once().await;

    let config = PostgresConfig::new(ADMIN_CONNECTION_STRING);
    let backend = PostgresBackend::new(config).await.unwrap();
    let client = backend.get_client().await.unwrap();

    let row = client
        .query_one(
            "SELECT EXISTS (
                SELECT 1 FROM information_schema.tables
                WHERE table_schema = 'llmspell'
                AND table_name = 'hook_history'
            )",
            &[],
        )
        .await
        .unwrap();

    let exists: bool = row.get(0);
    assert!(exists, "hook_history table should exist");
}

#[tokio::test]
async fn test_hook_history_table_schema() {
    ensure_migrations_run_once().await;

    let config = PostgresConfig::new(ADMIN_CONNECTION_STRING);
    let backend = PostgresBackend::new(config).await.unwrap();
    let client = backend.get_client().await.unwrap();

    let rows = client
        .query(
            "SELECT column_name, data_type
             FROM information_schema.columns
             WHERE table_schema = 'llmspell'
             AND table_name = 'hook_history'
             ORDER BY column_name",
            &[],
        )
        .await
        .unwrap();

    // Expected columns (in alphabetical order)
    let expected_columns = vec![
        ("component_id", "character varying"),
        ("contains_sensitive_data", "boolean"),
        ("context_size", "integer"),
        ("correlation_id", "uuid"),
        ("created_at", "timestamp with time zone"),
        ("duration_ms", "integer"),
        ("execution_id", "uuid"),
        ("hook_context", "bytea"),
        ("hook_id", "character varying"),
        ("hook_type", "character varying"),
        ("metadata", "jsonb"),
        ("modified_operation", "boolean"),
        ("result_data", "jsonb"),
        ("retention_priority", "integer"),
        ("tags", "ARRAY"),
        ("tenant_id", "character varying"),
        ("timestamp", "timestamp with time zone"),
        ("triggering_component", "character varying"),
    ];

    assert_eq!(rows.len(), expected_columns.len());

    for (i, row) in rows.iter().enumerate() {
        let column_name: String = row.get("column_name");
        let data_type: String = row.get("data_type");

        assert_eq!(
            (column_name.as_str(), data_type.as_str()),
            expected_columns[i],
            "Column {} schema mismatch",
            i
        );
    }
}

#[tokio::test]
async fn test_hook_history_indexes_created() {
    ensure_migrations_run_once().await;

    let config = PostgresConfig::new(ADMIN_CONNECTION_STRING);
    let backend = PostgresBackend::new(config).await.unwrap();
    let client = backend.get_client().await.unwrap();

    let rows = client
        .query(
            "SELECT indexname FROM pg_indexes
             WHERE schemaname = 'llmspell'
             AND tablename = 'hook_history'
             ORDER BY indexname",
            &[],
        )
        .await
        .unwrap();

    let index_names: Vec<String> = rows.iter().map(|row| row.get("indexname")).collect();

    // Expected indexes
    assert!(index_names.contains(&"hook_history_pkey".to_string()));
    assert!(index_names.contains(&"idx_hook_history_hook_time".to_string()));
    assert!(index_names.contains(&"idx_hook_history_correlation".to_string()));
    assert!(index_names.contains(&"idx_hook_history_type".to_string()));
    assert!(index_names.contains(&"idx_hook_history_tenant_time".to_string()));
    assert!(index_names.contains(&"idx_hook_history_retention".to_string()));
    assert!(index_names.contains(&"idx_hook_history_metadata".to_string()));
    assert!(index_names.contains(&"idx_hook_history_tags".to_string()));

    assert!(index_names.len() >= 8, "Should have at least 8 indexes");
}

#[tokio::test]
async fn test_hook_history_rls_policies_enabled() {
    ensure_migrations_run_once().await;

    let config = PostgresConfig::new(ADMIN_CONNECTION_STRING);
    let backend = PostgresBackend::new(config).await.unwrap();
    let client = backend.get_client().await.unwrap();

    // Check RLS is enabled
    let row = client
        .query_one(
            "SELECT relrowsecurity, relforcerowsecurity FROM pg_class
             WHERE oid = 'llmspell.hook_history'::regclass",
            &[],
        )
        .await
        .unwrap();

    let rls_enabled: bool = row.get(0);
    let force_rls: bool = row.get(1);

    assert!(rls_enabled, "RLS should be enabled");
    assert!(force_rls, "FORCE RLS should be enabled");

    // Check policies exist
    let rows = client
        .query(
            "SELECT policyname FROM pg_policies
             WHERE schemaname = 'llmspell'
             AND tablename = 'hook_history'
             ORDER BY policyname",
            &[],
        )
        .await
        .unwrap();

    let policy_names: Vec<String> = rows.iter().map(|row| row.get("policyname")).collect();

    assert!(policy_names.contains(&"hook_history_tenant_select".to_string()));
    assert!(policy_names.contains(&"hook_history_tenant_insert".to_string()));
    assert!(policy_names.contains(&"hook_history_tenant_update".to_string()));
    assert!(policy_names.contains(&"hook_history_tenant_delete".to_string()));

    assert_eq!(policy_names.len(), 4, "Should have exactly 4 RLS policies");
}

#[tokio::test]
async fn test_hook_history_cleanup_function_exists() {
    ensure_migrations_run_once().await;

    let config = PostgresConfig::new(ADMIN_CONNECTION_STRING);
    let backend = PostgresBackend::new(config).await.unwrap();
    let client = backend.get_client().await.unwrap();

    let row = client
        .query_one(
            "SELECT EXISTS (
                SELECT 1 FROM pg_proc p
                JOIN pg_namespace n ON p.pronamespace = n.oid
                WHERE n.nspname = 'llmspell'
                AND p.proname = 'cleanup_old_hook_executions'
            )",
            &[],
        )
        .await
        .unwrap();

    let exists: bool = row.get(0);
    assert!(exists, "cleanup_old_hook_executions function should exist");
}

#[tokio::test]
async fn test_hook_history_stats_function_exists() {
    ensure_migrations_run_once().await;

    let config = PostgresConfig::new(ADMIN_CONNECTION_STRING);
    let backend = PostgresBackend::new(config).await.unwrap();
    let client = backend.get_client().await.unwrap();

    let row = client
        .query_one(
            "SELECT EXISTS (
                SELECT 1 FROM pg_proc p
                JOIN pg_namespace n ON p.pronamespace = n.oid
                WHERE n.nspname = 'llmspell'
                AND p.proname = 'get_hook_history_stats'
            )",
            &[],
        )
        .await
        .unwrap();

    let exists: bool = row.get(0);
    assert!(exists, "get_hook_history_stats function should exist");
}

#[tokio::test]
async fn test_hook_history_rls_tenant_isolation() {
    ensure_migrations_run_once().await;

    let config = PostgresConfig::new(APP_CONNECTION_STRING);
    let backend = PostgresBackend::new(config).await.unwrap();

    let tenant1 = format!("test_tenant_{}", Uuid::new_v4());
    let tenant2 = format!("test_tenant_{}", Uuid::new_v4());

    backend.set_tenant_context(&tenant1).await.unwrap();

    let execution_id1 = Uuid::new_v4();
    let correlation_id1 = Uuid::new_v4();

    // Insert execution for tenant1
    let client = backend.get_client().await.unwrap();
    client
        .execute(
            "INSERT INTO llmspell.hook_history
             (execution_id, tenant_id, hook_id, hook_type, correlation_id, hook_context, result_data,
              timestamp, duration_ms, triggering_component, component_id, context_size)
             VALUES ($1, $2, 'test_hook', 'rate_limit', $3, $4, $5, now(), 100, 'Agent', 'agent-1', 100)",
            &[
                &execution_id1,
                &tenant1,
                &correlation_id1,
                &vec![1u8, 2u8, 3u8],
                &serde_json::json!({"success": true}),
            ],
        )
        .await
        .unwrap();

    // Verify tenant1 can see their execution
    let row = client
        .query_opt(
            "SELECT execution_id FROM llmspell.hook_history WHERE execution_id = $1",
            &[&execution_id1],
        )
        .await
        .unwrap();
    assert!(row.is_some(), "Tenant1 should see their own execution");

    // Switch to tenant2
    backend.set_tenant_context(&tenant2).await.unwrap();
    let client2 = backend.get_client().await.unwrap();

    // Verify tenant2 cannot see tenant1's execution
    let row = client2
        .query_opt(
            "SELECT execution_id FROM llmspell.hook_history WHERE execution_id = $1",
            &[&execution_id1],
        )
        .await
        .unwrap();
    assert!(
        row.is_none(),
        "Tenant2 should NOT see tenant1's execution (RLS isolation)"
    );

    // Cleanup
    backend.set_tenant_context(&tenant1).await.unwrap();
    let client = backend.get_client().await.unwrap();
    client
        .execute(
            "DELETE FROM llmspell.hook_history WHERE execution_id = $1",
            &[&execution_id1],
        )
        .await
        .unwrap();
}

#[tokio::test]
async fn test_hook_history_insert_and_query() {
    ensure_migrations_run_once().await;

    let config = PostgresConfig::new(APP_CONNECTION_STRING);
    let backend = PostgresBackend::new(config).await.unwrap();

    let tenant_id = format!("test_insert_{}", Uuid::new_v4());
    backend.set_tenant_context(&tenant_id).await.unwrap();

    let execution_id = Uuid::new_v4();
    let correlation_id = Uuid::new_v4();
    let hook_context = vec![1u8, 2u8, 3u8, 4u8, 5u8]; // Simulated compressed data
    let result_data = serde_json::json!({
        "success": true,
        "message": "Hook executed successfully"
    });

    let client = backend.get_client().await.unwrap();

    // Insert hook execution
    client
        .execute(
            "INSERT INTO llmspell.hook_history
             (execution_id, tenant_id, hook_id, hook_type, correlation_id,
              hook_context, result_data, timestamp, duration_ms,
              triggering_component, component_id, context_size, tags, retention_priority)
             VALUES ($1, $2, 'rate_limiter', 'rate_limit', $3, $4, $5, now(), 150,
                     'Agent', 'agent-123', 5, ARRAY['production', 'important'], 10)",
            &[
                &execution_id,
                &tenant_id,
                &correlation_id,
                &hook_context,
                &result_data,
            ],
        )
        .await
        .unwrap();

    // Query by execution_id
    let row = client
        .query_one(
            "SELECT hook_id, hook_type, duration_ms, tags, retention_priority
             FROM llmspell.hook_history
             WHERE execution_id = $1",
            &[&execution_id],
        )
        .await
        .unwrap();

    let hook_id: String = row.get("hook_id");
    let hook_type: String = row.get("hook_type");
    let duration_ms: i32 = row.get("duration_ms");
    let tags: Vec<String> = row.get("tags");
    let retention_priority: i32 = row.get("retention_priority");

    assert_eq!(hook_id, "rate_limiter");
    assert_eq!(hook_type, "rate_limit");
    assert_eq!(duration_ms, 150);
    assert_eq!(tags, vec!["production", "important"]);
    assert_eq!(retention_priority, 10);

    // Cleanup
    client
        .execute(
            "DELETE FROM llmspell.hook_history WHERE execution_id = $1",
            &[&execution_id],
        )
        .await
        .unwrap();
}
