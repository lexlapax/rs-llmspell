//! Tests for V5__procedural_memory migration (Phase 13b.6.1)
//!
//! Verifies:
//! - procedural_patterns table created with correct schema
//! - 6 indexes created (tenant, scope_key, frequency, last_seen, lookup, unique)
//! - RLS policies applied (4 policies: SELECT, INSERT, UPDATE, DELETE)
//! - Unique constraint on (tenant_id, scope, key, value)
//! - Trigger auto-updates updated_at timestamp
//! - Frequency constraint (must be > 0)

#![cfg(feature = "postgres")]

use llmspell_storage::{PostgresBackend, PostgresConfig};
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
async fn test_procedural_patterns_table_created() {
    ensure_migrations_run_once().await;

    let config = PostgresConfig::new(SUPERUSER_CONNECTION_STRING);
    let backend = PostgresBackend::new(config).await.unwrap();
    let client = backend.get_client().await.unwrap();

    // Verify table exists
    let row = client
        .query_one(
            "SELECT COUNT(*) FROM pg_tables
             WHERE schemaname = 'llmspell' AND tablename = 'procedural_patterns'",
            &[],
        )
        .await
        .unwrap();
    let count: i64 = row.get(0);
    assert_eq!(count, 1, "procedural_patterns table should exist");

    // Verify RLS is enabled
    let row = client
        .query_one(
            "SELECT relrowsecurity FROM pg_class
             WHERE relname = 'procedural_patterns' AND relnamespace = 'llmspell'::regnamespace",
            &[],
        )
        .await
        .unwrap();
    let rls_enabled: bool = row.get(0);
    assert!(rls_enabled, "RLS should be enabled on procedural_patterns");
}

#[tokio::test]
async fn test_procedural_patterns_indexes_created() {
    ensure_migrations_run_once().await;

    let config = PostgresConfig::new(SUPERUSER_CONNECTION_STRING);
    let backend = PostgresBackend::new(config).await.unwrap();
    let client = backend.get_client().await.unwrap();

    // Get all indexes for procedural_patterns (excluding primary key and unique constraint indexes)
    let rows = client
        .query(
            "SELECT indexname
             FROM pg_indexes
             WHERE schemaname = 'llmspell'
               AND tablename = 'procedural_patterns'
               AND indexname LIKE 'idx_%'
             ORDER BY indexname",
            &[],
        )
        .await
        .unwrap();

    // Should have 5 indexes (tenant, scope_key, frequency, last_seen, lookup)
    assert_eq!(rows.len(), 5, "Should have 5 indexes");

    let expected_indexes = [
        "idx_procedural_patterns_frequency",
        "idx_procedural_patterns_last_seen",
        "idx_procedural_patterns_lookup",
        "idx_procedural_patterns_scope_key",
        "idx_procedural_patterns_tenant",
    ];

    for (i, row) in rows.iter().enumerate() {
        let indexname: String = row.get("indexname");
        assert_eq!(indexname, expected_indexes[i]);
    }
}

#[tokio::test]
async fn test_procedural_patterns_rls_policies() {
    ensure_migrations_run_once().await;

    let config = PostgresConfig::new(SUPERUSER_CONNECTION_STRING);
    let backend = PostgresBackend::new(config).await.unwrap();
    let client = backend.get_client().await.unwrap();

    // Verify all 4 RLS policies exist
    let rows = client
        .query(
            "SELECT policyname, cmd
             FROM pg_policies
             WHERE schemaname = 'llmspell' AND tablename = 'procedural_patterns'
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
async fn test_procedural_patterns_unique_constraint() {
    ensure_migrations_run_once().await;

    let config = PostgresConfig::new(TEST_CONNECTION_STRING);
    let backend = PostgresBackend::new(config).await.unwrap();
    let tenant_id = unique_tenant_id("unique-test");

    backend.set_tenant_context(&tenant_id).await.unwrap();
    let client = backend.get_client().await.unwrap();

    // Insert first pattern
    client
        .execute(
            "INSERT INTO llmspell.procedural_patterns
             (tenant_id, scope, key, value, frequency)
             VALUES ($1, $2, $3, $4, $5)",
            &[&tenant_id, &"global", &"theme", &"dark", &1i32],
        )
        .await
        .unwrap();

    // Try to insert duplicate (should fail)
    let result = client
        .execute(
            "INSERT INTO llmspell.procedural_patterns
             (tenant_id, scope, key, value, frequency)
             VALUES ($1, $2, $3, $4, $5)",
            &[&tenant_id, &"global", &"theme", &"dark", &2i32],
        )
        .await;

    assert!(
        result.is_err(),
        "Duplicate pattern should violate unique constraint"
    );

    let error_msg = result.unwrap_err().to_string();
    assert!(
        error_msg.contains("unique") || error_msg.contains("duplicate key") || error_msg.contains("already exists"),
        "Error should indicate unique constraint violation, got: {}",
        error_msg
    );
}

#[tokio::test]
async fn test_procedural_patterns_frequency_constraint() {
    ensure_migrations_run_once().await;

    let config = PostgresConfig::new(TEST_CONNECTION_STRING);
    let backend = PostgresBackend::new(config).await.unwrap();
    let tenant_id = unique_tenant_id("frequency-test");

    backend.set_tenant_context(&tenant_id).await.unwrap();
    let client = backend.get_client().await.unwrap();

    // Try to insert pattern with frequency 0 (should fail)
    let result = client
        .execute(
            "INSERT INTO llmspell.procedural_patterns
             (tenant_id, scope, key, value, frequency)
             VALUES ($1, $2, $3, $4, $5)",
            &[&tenant_id, &"global", &"theme", &"dark", &0i32],
        )
        .await;

    assert!(
        result.is_err(),
        "Frequency 0 should violate positive_frequency constraint"
    );

    // Try to insert pattern with negative frequency (should fail)
    let result = client
        .execute(
            "INSERT INTO llmspell.procedural_patterns
             (tenant_id, scope, key, value, frequency)
             VALUES ($1, $2, $3, $4, $5)",
            &[&tenant_id, &"global", &"theme", &"light", &-1i32],
        )
        .await;

    assert!(
        result.is_err(),
        "Negative frequency should violate positive_frequency constraint"
    );
}

#[tokio::test]
async fn test_procedural_patterns_updated_at_trigger() {
    ensure_migrations_run_once().await;

    let config = PostgresConfig::new(TEST_CONNECTION_STRING);
    let backend = PostgresBackend::new(config).await.unwrap();
    let tenant_id = unique_tenant_id("trigger-test");

    backend.set_tenant_context(&tenant_id).await.unwrap();
    let client = backend.get_client().await.unwrap();

    // Insert pattern
    client
        .execute(
            "INSERT INTO llmspell.procedural_patterns
             (tenant_id, scope, key, value, frequency)
             VALUES ($1, $2, $3, $4, $5)",
            &[&tenant_id, &"global", &"theme", &"dark", &1i32],
        )
        .await
        .unwrap();

    // Get initial updated_at
    let row = client
        .query_one(
            "SELECT updated_at FROM llmspell.procedural_patterns
             WHERE tenant_id = $1 AND scope = $2 AND key = $3 AND value = $4",
            &[&tenant_id, &"global", &"theme", &"dark"],
        )
        .await
        .unwrap();
    let initial_updated_at: chrono::DateTime<chrono::Utc> = row.get(0);

    // Wait a bit to ensure timestamp difference
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    // Update the pattern
    client
        .execute(
            "UPDATE llmspell.procedural_patterns
             SET frequency = frequency + 1
             WHERE tenant_id = $1 AND scope = $2 AND key = $3 AND value = $4",
            &[&tenant_id, &"global", &"theme", &"dark"],
        )
        .await
        .unwrap();

    // Get updated updated_at
    let row = client
        .query_one(
            "SELECT updated_at FROM llmspell.procedural_patterns
             WHERE tenant_id = $1 AND scope = $2 AND key = $3 AND value = $4",
            &[&tenant_id, &"global", &"theme", &"dark"],
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
async fn test_procedural_patterns_rls_isolation() {
    ensure_migrations_run_once().await;

    let config = PostgresConfig::new(TEST_CONNECTION_STRING);
    let backend = PostgresBackend::new(config).await.unwrap();

    let tenant_a = unique_tenant_id("rls-a");
    let tenant_b = unique_tenant_id("rls-b");

    // Insert pattern for tenant A
    backend.set_tenant_context(&tenant_a).await.unwrap();
    {
        let client = backend.get_client().await.unwrap();
        client
            .execute(
                "INSERT INTO llmspell.procedural_patterns
                 (tenant_id, scope, key, value, frequency)
                 VALUES ($1, $2, $3, $4, $5)",
                &[&tenant_a, &"global", &"theme", &"dark", &5i32],
            )
            .await
            .unwrap();
    }

    // Insert pattern for tenant B
    backend.set_tenant_context(&tenant_b).await.unwrap();
    {
        let client = backend.get_client().await.unwrap();
        client
            .execute(
                "INSERT INTO llmspell.procedural_patterns
                 (tenant_id, scope, key, value, frequency)
                 VALUES ($1, $2, $3, $4, $5)",
                &[&tenant_b, &"global", &"theme", &"light", &3i32],
            )
            .await
            .unwrap();
    }

    // Query as tenant A - should only see tenant A's pattern
    backend.set_tenant_context(&tenant_a).await.unwrap();
    let rows = {
        let client = backend.get_client().await.unwrap();
        client
            .query(
                "SELECT value, frequency FROM llmspell.procedural_patterns
                 WHERE scope = 'global' AND key = 'theme'",
                &[],
            )
            .await
            .unwrap()
    };

    assert_eq!(rows.len(), 1, "Tenant A should see only 1 pattern");
    let value: String = rows[0].get(0);
    let frequency: i32 = rows[0].get(1);
    assert_eq!(value, "dark");
    assert_eq!(frequency, 5);

    // Query as tenant B - should only see tenant B's pattern
    backend.set_tenant_context(&tenant_b).await.unwrap();
    let rows = {
        let client = backend.get_client().await.unwrap();
        client
            .query(
                "SELECT value, frequency FROM llmspell.procedural_patterns
                 WHERE scope = 'global' AND key = 'theme'",
                &[],
            )
            .await
            .unwrap()
    };

    assert_eq!(rows.len(), 1, "Tenant B should see only 1 pattern");
    let value: String = rows[0].get(0);
    let frequency: i32 = rows[0].get(1);
    assert_eq!(value, "light");
    assert_eq!(frequency, 3);
}
