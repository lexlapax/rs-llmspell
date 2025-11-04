//! Tests for V3__vector_embeddings migration (Phase 13b.4.1)
//!
//! Verifies:
//! - All 4 dimension tables created (384, 768, 1536, 3072)
//! - HNSW indices functional
//! - RLS policies applied (4 per table = 16 total)
//! - Privilege grants to llmspell_app user

#![cfg(feature = "postgres")]

use llmspell_storage::{PostgresBackend, PostgresConfig};
use tokio::sync::OnceCell;
use uuid::Uuid;

const SUPERUSER_CONNECTION_STRING: &str =
    "postgresql://llmspell:llmspell_dev_pass@localhost:5432/llmspell_dev";

const APP_CONNECTION_STRING: &str =
    "postgresql://llmspell_app:llmspell_dev_pass@localhost:5432/llmspell_dev";

static MIGRATION_INIT: OnceCell<()> = OnceCell::const_new();

/// Ensure migrations run once before all tests (Phase 13b.3 pattern)
async fn ensure_migrations_run_once() {
    MIGRATION_INIT
        .get_or_init(|| async {
            let config = PostgresConfig::new(SUPERUSER_CONNECTION_STRING);
            let backend = PostgresBackend::new(config)
                .await
                .expect("Failed to create backend for migration init");

            // Run migrations (V1, V2, V3)
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
async fn test_vector_embeddings_tables_created() {
    ensure_migrations_run_once().await;

    let config = PostgresConfig::new(SUPERUSER_CONNECTION_STRING);
    let backend = PostgresBackend::new(config).await.unwrap();
    let client = backend.get_client().await.unwrap();

    // Verify all 4 tables exist
    let tables = vec![
        "vector_embeddings_384",
        "vector_embeddings_768",
        "vector_embeddings_1536",
        "vector_embeddings_3072",
    ];

    for table in tables {
        let row = client
            .query_one(
                "SELECT COUNT(*) FROM pg_tables WHERE schemaname = 'llmspell' AND tablename = $1",
                &[&table],
            )
            .await
            .unwrap();
        let count: i64 = row.get(0);
        assert_eq!(count, 1, "Table {} should exist", table);
    }
}

#[tokio::test]
async fn test_vector_embeddings_hnsw_indices() {
    ensure_migrations_run_once().await;

    let config = PostgresConfig::new(SUPERUSER_CONNECTION_STRING);
    let backend = PostgresBackend::new(config).await.unwrap();
    let client = backend.get_client().await.unwrap();

    // Verify HNSW indices exist (sorted by dimension not by name)
    let rows = client
        .query(
            "SELECT tablename, indexname
             FROM pg_indexes
             WHERE schemaname = 'llmspell'
               AND indexname LIKE '%hnsw%'
             ORDER BY tablename",
            &[],
        )
        .await
        .unwrap();

    assert_eq!(
        rows.len(),
        3,
        "Should have 3 HNSW indices (384, 768, 1536 dimensions)"
    );

    // Indices ordered alphabetically by table name (1536, 384, 768)
    let expected_indices = [
        "idx_vector_1536_hnsw",
        "idx_vector_384_hnsw",
        "idx_vector_768_hnsw",
    ];

    for (i, row) in rows.iter().enumerate() {
        let indexname: String = row.get("indexname");
        assert_eq!(indexname, expected_indices[i]);
    }

    // Note: 3072-dimensional table has no vector similarity index
    // Both HNSW and IVFFlat have 2000-dimension max in pgvector
}

#[tokio::test]
async fn test_vector_embeddings_rls_policies() {
    ensure_migrations_run_once().await;

    let config = PostgresConfig::new(SUPERUSER_CONNECTION_STRING);
    let backend = PostgresBackend::new(config).await.unwrap();
    let client = backend.get_client().await.unwrap();

    // Verify RLS policies exist (4 policies × 4 tables = 16 total)
    let rows = client
        .query(
            "SELECT tablename, policyname, cmd
             FROM pg_policies
             WHERE schemaname = 'llmspell'
               AND tablename LIKE 'vector_embeddings_%'
             ORDER BY tablename, cmd",
            &[],
        )
        .await
        .unwrap();

    assert_eq!(
        rows.len(),
        16,
        "Should have 16 RLS policies (4 tables × 4 policies)"
    );

    // Verify each table has all 4 policy types
    let tables = vec![
        "vector_embeddings_384",
        "vector_embeddings_768",
        "vector_embeddings_1536",
        "vector_embeddings_3072",
    ];

    for table in tables {
        let table_policies: Vec<_> = rows
            .iter()
            .filter(|row| {
                let t: String = row.get("tablename");
                t == table
            })
            .collect();

        assert_eq!(
            table_policies.len(),
            4,
            "Table {} should have 4 RLS policies",
            table
        );

        // Verify policy names
        let policy_names: Vec<String> = table_policies
            .iter()
            .map(|row| row.get("policyname"))
            .collect();

        assert!(policy_names.contains(&"tenant_isolation_select".to_string()));
        assert!(policy_names.contains(&"tenant_isolation_insert".to_string()));
        assert!(policy_names.contains(&"tenant_isolation_update".to_string()));
        assert!(policy_names.contains(&"tenant_isolation_delete".to_string()));
    }
}

#[tokio::test]
async fn test_vector_embeddings_rls_enabled() {
    ensure_migrations_run_once().await;

    let config = PostgresConfig::new(SUPERUSER_CONNECTION_STRING);
    let backend = PostgresBackend::new(config).await.unwrap();
    let client = backend.get_client().await.unwrap();

    // Verify RLS is enabled on all tables
    let tables = vec![
        "vector_embeddings_384",
        "vector_embeddings_768",
        "vector_embeddings_1536",
        "vector_embeddings_3072",
    ];

    for table in tables {
        let row = client
            .query_one(
                "SELECT relrowsecurity FROM pg_class WHERE relname = $1",
                &[&table],
            )
            .await
            .unwrap();
        let rls_enabled: bool = row.get(0);
        assert!(rls_enabled, "RLS should be enabled on {}", table);
    }
}

#[tokio::test]
async fn test_vector_embeddings_app_user_permissions() {
    ensure_migrations_run_once().await;

    // Use llmspell_app user (non-superuser)
    let config = PostgresConfig::new(APP_CONNECTION_STRING);
    let backend = PostgresBackend::new(config).await.unwrap();

    // Set tenant context
    let tenant_id = unique_tenant_id("perm-test");
    backend
        .set_tenant_context(&tenant_id)
        .await
        .expect("Should be able to set tenant context");

    let client = backend.get_client().await.unwrap();

    // Test INSERT permission on each table
    let tables = vec![
        ("vector_embeddings_384", 384),
        ("vector_embeddings_768", 768),
        ("vector_embeddings_1536", 1536),
        ("vector_embeddings_3072", 3072),
    ];

    for (table, dim) in tables {
        let embedding = vec![0.1; dim];

        // Should be able to INSERT
        let result = client
            .execute(
                &format!(
                    "INSERT INTO llmspell.{} (tenant_id, scope, embedding, metadata)
                     VALUES ($1, 'test', $2, '{{}}')",
                    table
                ),
                &[&tenant_id, &pgvector::Vector::from(embedding)],
            )
            .await;

        assert!(
            result.is_ok(),
            "llmspell_app should have INSERT permission on {}",
            table
        );

        // Should be able to SELECT
        let rows = client
            .query(&format!("SELECT COUNT(*) FROM llmspell.{}", table), &[])
            .await;

        assert!(
            rows.is_ok(),
            "llmspell_app should have SELECT permission on {}",
            table
        );

        // Cleanup
        client
            .execute(&format!("DELETE FROM llmspell.{} WHERE TRUE", table), &[])
            .await
            .expect("Should be able to DELETE");
    }
}

#[tokio::test]
async fn test_vector_embeddings_rls_isolation() {
    ensure_migrations_run_once().await;

    let config = PostgresConfig::new(APP_CONNECTION_STRING);
    let backend = PostgresBackend::new(config).await.unwrap();

    // Test RLS isolation on all 4 tables
    for (table, dim) in [
        ("vector_embeddings_384", 384),
        ("vector_embeddings_768", 768),
        ("vector_embeddings_1536", 1536),
        ("vector_embeddings_3072", 3072),
    ] {
        let tenant_a = unique_tenant_id(&format!("{}-a", table));
        let tenant_b = unique_tenant_id(&format!("{}-b", table));

        // Tenant A inserts data
        backend.set_tenant_context(&tenant_a).await.unwrap();
        let client = backend.get_client().await.unwrap();

        let embedding = vec![0.1; dim];
        client
            .execute(
                &format!(
                    "INSERT INTO llmspell.{} (tenant_id, scope, embedding, metadata)
                     VALUES ($1, 'test', $2, '{{}}')",
                    table
                ),
                &[&tenant_a, &pgvector::Vector::from(embedding)],
            )
            .await
            .unwrap();

        // Tenant A should see 1 row
        let rows = client
            .query(&format!("SELECT COUNT(*) FROM llmspell.{}", table), &[])
            .await
            .unwrap();
        let count: i64 = rows[0].get(0);
        assert_eq!(count, 1, "Tenant A should see its own data in {}", table);

        // Switch to Tenant B
        backend.clear_tenant_context().await.unwrap();
        backend.set_tenant_context(&tenant_b).await.unwrap();
        let client = backend.get_client().await.unwrap();

        // Tenant B should see 0 rows
        let rows = client
            .query(&format!("SELECT COUNT(*) FROM llmspell.{}", table), &[])
            .await
            .unwrap();
        let count: i64 = rows[0].get(0);
        assert_eq!(
            count, 0,
            "Tenant B should NOT see tenant A data in {}",
            table
        );

        // Cleanup
        backend.clear_tenant_context().await.unwrap();
        backend.set_tenant_context(&tenant_a).await.unwrap();
        let client = backend.get_client().await.unwrap();
        client
            .execute(&format!("DELETE FROM llmspell.{} WHERE TRUE", table), &[])
            .await
            .unwrap();
    }
}

#[tokio::test]
async fn test_vector_embeddings_migration_idempotent() {
    ensure_migrations_run_once().await;

    let config = PostgresConfig::new(SUPERUSER_CONNECTION_STRING);
    let backend = PostgresBackend::new(config).await.unwrap();

    // Run migrations again - should be idempotent
    backend
        .run_migrations()
        .await
        .expect("Migrations should be idempotent");

    // Verify version is still 3
    let version = backend
        .migration_version()
        .await
        .expect("Should be able to get migration version");

    assert!(
        version >= 3,
        "Migration version should be at least 3 after V3"
    );
}
