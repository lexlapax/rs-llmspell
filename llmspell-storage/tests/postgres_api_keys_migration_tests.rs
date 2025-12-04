//! Migration tests for PostgreSQL API Keys Schema (Phase 13b.13.1)
//!
//! Tests V14__api_keys.sql migration:
//! - Table and column creation
//! - Index creation
//! - RLS policies
//! - pgcrypto extension
//! - Helper functions (cleanup, stats, rotate)
//! - Encryption/decryption

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
async fn test_api_keys_table_exists() {
    ensure_migrations_run_once().await;

    let config = PostgresConfig::new(ADMIN_CONNECTION_STRING);
    let backend = PostgresBackend::new(config).await.unwrap();
    let client = backend.get_client().await.unwrap();

    let row = client
        .query_one(
            "SELECT EXISTS (
                SELECT 1 FROM information_schema.tables
                WHERE table_schema = 'llmspell'
                AND table_name = 'api_keys'
            )",
            &[],
        )
        .await
        .unwrap();

    let exists: bool = row.get(0);
    assert!(exists, "api_keys table should exist");
}

#[tokio::test]
async fn test_api_keys_table_schema() {
    ensure_migrations_run_once().await;

    let config = PostgresConfig::new(ADMIN_CONNECTION_STRING);
    let backend = PostgresBackend::new(config).await.unwrap();
    let client = backend.get_client().await.unwrap();

    let rows = client
        .query(
            "SELECT column_name, data_type
             FROM information_schema.columns
             WHERE table_schema = 'llmspell'
             AND table_name = 'api_keys'
             ORDER BY column_name",
            &[],
        )
        .await
        .unwrap();

    // Expected columns (in alphabetical order)
    let expected_columns = vec![
        ("created_at", "timestamp with time zone"),
        ("deactivated_at", "timestamp with time zone"),
        ("encrypted_key", "bytea"),
        ("expires_at", "timestamp with time zone"),
        ("is_active", "boolean"),
        ("key_id", "character varying"),
        ("key_metadata", "jsonb"),
        ("last_used_at", "timestamp with time zone"),
        ("rotated_from", "character varying"),
        ("service", "character varying"),
        ("tenant_id", "character varying"),
        ("usage_count", "bigint"),
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
async fn test_api_keys_indexes_created() {
    ensure_migrations_run_once().await;

    let config = PostgresConfig::new(ADMIN_CONNECTION_STRING);
    let backend = PostgresBackend::new(config).await.unwrap();
    let client = backend.get_client().await.unwrap();

    let rows = client
        .query(
            "SELECT indexname FROM pg_indexes
             WHERE schemaname = 'llmspell'
             AND tablename = 'api_keys'
             ORDER BY indexname",
            &[],
        )
        .await
        .unwrap();

    let index_names: Vec<String> = rows.iter().map(|row| row.get("indexname")).collect();

    // Expected indexes
    assert!(index_names.contains(&"api_keys_pkey".to_string()));
    assert!(index_names.contains(&"idx_api_keys_tenant_service".to_string()));
    assert!(index_names.contains(&"idx_api_keys_expiration".to_string()));
    assert!(index_names.contains(&"idx_api_keys_active".to_string()));
    assert!(index_names.contains(&"idx_api_keys_metadata".to_string()));

    assert!(index_names.len() >= 5, "Should have at least 5 indexes");
}

#[tokio::test]
async fn test_api_keys_rls_policies_enabled() {
    ensure_migrations_run_once().await;

    let config = PostgresConfig::new(ADMIN_CONNECTION_STRING);
    let backend = PostgresBackend::new(config).await.unwrap();
    let client = backend.get_client().await.unwrap();

    // Check RLS is enabled
    let row = client
        .query_one(
            "SELECT relrowsecurity, relforcerowsecurity FROM pg_class
             WHERE oid = 'llmspell.api_keys'::regclass",
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
             AND tablename = 'api_keys'
             ORDER BY policyname",
            &[],
        )
        .await
        .unwrap();

    let policy_names: Vec<String> = rows.iter().map(|row| row.get("policyname")).collect();

    assert!(policy_names.contains(&"api_keys_tenant_select".to_string()));
    assert!(policy_names.contains(&"api_keys_tenant_insert".to_string()));
    assert!(policy_names.contains(&"api_keys_tenant_update".to_string()));
    assert!(policy_names.contains(&"api_keys_tenant_delete".to_string()));

    assert_eq!(policy_names.len(), 4, "Should have exactly 4 RLS policies");
}

#[tokio::test]
async fn test_pgcrypto_extension_exists() {
    ensure_migrations_run_once().await;

    let config = PostgresConfig::new(ADMIN_CONNECTION_STRING);
    let backend = PostgresBackend::new(config).await.unwrap();
    let client = backend.get_client().await.unwrap();

    let row = client
        .query_one(
            "SELECT EXISTS (
                SELECT 1 FROM pg_extension
                WHERE extname = 'pgcrypto'
            )",
            &[],
        )
        .await
        .unwrap();

    let exists: bool = row.get(0);
    assert!(exists, "pgcrypto extension should exist");
}

#[tokio::test]
async fn test_api_keys_cleanup_function_exists() {
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
                AND p.proname = 'cleanup_expired_api_keys'
            )",
            &[],
        )
        .await
        .unwrap();

    let exists: bool = row.get(0);
    assert!(exists, "cleanup_expired_api_keys function should exist");
}

#[tokio::test]
async fn test_api_keys_stats_function_exists() {
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
                AND p.proname = 'get_api_key_stats'
            )",
            &[],
        )
        .await
        .unwrap();

    let exists: bool = row.get(0);
    assert!(exists, "get_api_key_stats function should exist");
}

#[tokio::test]
async fn test_api_keys_rotate_function_exists() {
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
                AND p.proname = 'rotate_api_key'
            )",
            &[],
        )
        .await
        .unwrap();

    let exists: bool = row.get(0);
    assert!(exists, "rotate_api_key function should exist");
}

#[tokio::test]
async fn test_api_keys_encryption_decryption() {
    ensure_migrations_run_once().await;

    let config = PostgresConfig::new(APP_CONNECTION_STRING);
    let backend = PostgresBackend::new(config).await.unwrap();

    let tenant_id = format!("test_encryption_{}", Uuid::new_v4());
    backend.set_tenant_context(&tenant_id).await.unwrap();

    let client = backend.get_client().await.unwrap();

    let key_id = format!("test_key_{}", Uuid::new_v4());
    let plaintext_key = "sk-1234567890abcdef";
    let encryption_passphrase = "test_passphrase_12345";

    // Encrypt and store (schema-qualified pgp_sym_encrypt)
    client
        .execute(
            "INSERT INTO llmspell.api_keys
             (key_id, tenant_id, service, encrypted_key, key_metadata)
             VALUES ($1, $2, 'openai', pgp_sym_encrypt($3::TEXT, $4::TEXT), '{}'::jsonb)",
            &[&key_id, &tenant_id, &plaintext_key, &encryption_passphrase],
        )
        .await
        .unwrap();

    // Retrieve and decrypt (schema-qualified)
    let row = client
        .query_one(
            "SELECT pgp_sym_decrypt(encrypted_key, $2::TEXT) as decrypted_key
             FROM llmspell.api_keys
             WHERE key_id = $1",
            &[&key_id, &encryption_passphrase],
        )
        .await
        .unwrap();

    let decrypted_key: String = row.get("decrypted_key");

    assert_eq!(decrypted_key, plaintext_key);

    // Cleanup
    client
        .execute(
            "DELETE FROM llmspell.api_keys WHERE key_id = $1",
            &[&key_id],
        )
        .await
        .unwrap();
}

#[tokio::test]
async fn test_api_keys_rls_tenant_isolation() {
    ensure_migrations_run_once().await;

    let config = PostgresConfig::new(APP_CONNECTION_STRING);
    let backend = PostgresBackend::new(config).await.unwrap();

    let tenant1 = format!("test_tenant1_{}", Uuid::new_v4());
    let tenant2 = format!("test_tenant2_{}", Uuid::new_v4());

    backend.set_tenant_context(&tenant1).await.unwrap();

    let key_id1 = format!("key_tenant1_{}", Uuid::new_v4());
    let encryption_passphrase = "test_passphrase";

    // Insert key for tenant1 (schema-qualified)
    let client = backend.get_client().await.unwrap();
    client
        .execute(
            "INSERT INTO llmspell.api_keys
             (key_id, tenant_id, service, encrypted_key)
             VALUES ($1, $2, 'anthropic', pgp_sym_encrypt('sk-ant-test'::TEXT, $3::TEXT))",
            &[&key_id1, &tenant1, &encryption_passphrase],
        )
        .await
        .unwrap();

    // Verify tenant1 can see their key
    let row = client
        .query_opt(
            "SELECT key_id FROM llmspell.api_keys WHERE key_id = $1",
            &[&key_id1],
        )
        .await
        .unwrap();
    assert!(row.is_some(), "Tenant1 should see their own key");

    // Switch to tenant2
    backend.set_tenant_context(&tenant2).await.unwrap();
    let client2 = backend.get_client().await.unwrap();

    // Verify tenant2 cannot see tenant1's key
    let row = client2
        .query_opt(
            "SELECT key_id FROM llmspell.api_keys WHERE key_id = $1",
            &[&key_id1],
        )
        .await
        .unwrap();
    assert!(
        row.is_none(),
        "Tenant2 should NOT see tenant1's key (RLS isolation)"
    );

    // Cleanup
    backend.set_tenant_context(&tenant1).await.unwrap();
    let client = backend.get_client().await.unwrap();
    client
        .execute(
            "DELETE FROM llmspell.api_keys WHERE key_id = $1",
            &[&key_id1],
        )
        .await
        .unwrap();
}

#[tokio::test]
async fn test_api_keys_unique_constraint() {
    ensure_migrations_run_once().await;

    let config = PostgresConfig::new(APP_CONNECTION_STRING);
    let backend = PostgresBackend::new(config).await.unwrap();

    let tenant_id = format!("test_unique_{}", Uuid::new_v4());
    backend.set_tenant_context(&tenant_id).await.unwrap();

    let encryption_passphrase = "test_passphrase";
    let service = "google_search";

    let client = backend.get_client().await.unwrap();

    // Insert first active key for service (schema-qualified)
    let key_id1 = format!("key1_{}", Uuid::new_v4());
    client
        .execute(
            "INSERT INTO llmspell.api_keys
             (key_id, tenant_id, service, encrypted_key, is_active)
             VALUES ($1, $2, $3, pgp_sym_encrypt('key1'::TEXT, $4::TEXT), true)",
            &[&key_id1, &tenant_id, &service, &encryption_passphrase],
        )
        .await
        .unwrap();

    // Try to insert second active key for same tenant/service (should fail)
    let key_id2 = format!("key2_{}", Uuid::new_v4());
    let result = client
        .execute(
            "INSERT INTO llmspell.api_keys
             (key_id, tenant_id, service, encrypted_key, is_active)
             VALUES ($1, $2, $3, pgp_sym_encrypt('key2'::TEXT, $4::TEXT), true)",
            &[&key_id2, &tenant_id, &service, &encryption_passphrase],
        )
        .await;

    assert!(
        result.is_err(),
        "Should fail to insert duplicate active key for same tenant/service"
    );

    // Deactivate first key
    client
        .execute(
            "UPDATE llmspell.api_keys SET is_active = false WHERE key_id = $1",
            &[&key_id1],
        )
        .await
        .unwrap();

    // Now inserting second active key should succeed (schema-qualified)
    client
        .execute(
            "INSERT INTO llmspell.api_keys
             (key_id, tenant_id, service, encrypted_key, is_active)
             VALUES ($1, $2, $3, pgp_sym_encrypt('key2'::TEXT, $4::TEXT), true)",
            &[&key_id2, &tenant_id, &service, &encryption_passphrase],
        )
        .await
        .unwrap();

    // Cleanup
    client
        .execute(
            "DELETE FROM llmspell.api_keys WHERE key_id IN ($1, $2)",
            &[&key_id1, &key_id2],
        )
        .await
        .unwrap();
}
