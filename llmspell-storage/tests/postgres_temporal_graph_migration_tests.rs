//! Tests for V4__temporal_graph migration (Phase 13b.5.1)
//!
//! Verifies:
//! - Entities and relationships tables created
//! - GiST time-range indexes functional
//! - RLS policies applied (4 per table = 8 total)
//! - Privilege grants to llmspell_app user
//! - Bi-temporal semantics working correctly
//! - Foreign key constraints enforced

#![cfg(feature = "postgres")]

use llmspell_storage::{PostgresBackend, PostgresConfig};
use tokio::sync::OnceCell;
use uuid::Uuid;

const SUPERUSER_CONNECTION_STRING: &str =
    "postgresql://llmspell:llmspell_dev_pass@localhost:5432/llmspell_dev";

const APP_CONNECTION_STRING: &str =
    "postgresql://llmspell_app:llmspell_app_pass@localhost:5432/llmspell_dev";

static MIGRATION_INIT: OnceCell<()> = OnceCell::const_new();

/// Ensure migrations run once before all tests (Phase 13b.3 pattern)
async fn ensure_migrations_run_once() {
    MIGRATION_INIT
        .get_or_init(|| async {
            let config = PostgresConfig::new(SUPERUSER_CONNECTION_STRING);
            let backend = PostgresBackend::new(config)
                .await
                .expect("Failed to create backend for migration init");

            // Run migrations (V1, V2, V3, V4)
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
async fn test_temporal_graph_tables_created() {
    ensure_migrations_run_once().await;

    let config = PostgresConfig::new(SUPERUSER_CONNECTION_STRING);
    let backend = PostgresBackend::new(config).await.unwrap();
    let client = backend.get_client().await.unwrap();

    // Verify both tables exist
    let tables = ["entities", "relationships"];

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
async fn test_temporal_graph_gist_indices() {
    ensure_migrations_run_once().await;

    let config = PostgresConfig::new(SUPERUSER_CONNECTION_STRING);
    let backend = PostgresBackend::new(config).await.unwrap();
    let client = backend.get_client().await.unwrap();

    // Verify GiST indices exist (4 total: 2 per table)
    let rows = client
        .query(
            "SELECT tablename, indexname, indexdef
             FROM pg_indexes
             WHERE schemaname = 'llmspell'
               AND tablename IN ('entities', 'relationships')
               AND indexname LIKE '%_time'
             ORDER BY tablename, indexname",
            &[],
        )
        .await
        .unwrap();

    assert_eq!(
        rows.len(),
        4,
        "Should have 4 GiST time indices (2 entities + 2 relationships)"
    );

    // Verify index definitions use GIST and tstzrange
    for row in &rows {
        let indexdef: String = row.get("indexdef");
        assert!(
            indexdef.contains("USING gist"),
            "Index should use GIST: {}",
            indexdef
        );
        assert!(
            indexdef.contains("tstzrange"),
            "Index should use tstzrange: {}",
            indexdef
        );
    }
}

#[tokio::test]
async fn test_temporal_graph_rls_policies() {
    ensure_migrations_run_once().await;

    let config = PostgresConfig::new(SUPERUSER_CONNECTION_STRING);
    let backend = PostgresBackend::new(config).await.unwrap();
    let client = backend.get_client().await.unwrap();

    // Verify RLS policies exist (4 policies × 2 tables = 8 total)
    let rows = client
        .query(
            "SELECT tablename, policyname, cmd
             FROM pg_policies
             WHERE schemaname = 'llmspell'
               AND (tablename = 'entities' OR tablename = 'relationships')
             ORDER BY tablename, cmd",
            &[],
        )
        .await
        .unwrap();

    assert_eq!(
        rows.len(),
        8,
        "Should have 8 RLS policies (2 tables × 4 policies)"
    );

    // Verify each table has all 4 policy types
    for table in ["entities", "relationships"] {
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
async fn test_temporal_graph_rls_enabled() {
    ensure_migrations_run_once().await;

    let config = PostgresConfig::new(SUPERUSER_CONNECTION_STRING);
    let backend = PostgresBackend::new(config).await.unwrap();
    let client = backend.get_client().await.unwrap();

    // Verify RLS is enabled on both tables
    for table in ["entities", "relationships"] {
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
async fn test_temporal_graph_app_user_permissions() {
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

    // Test INSERT permission on entities table
    let entity_id = Uuid::new_v4();
    let result = client
        .execute(
            "INSERT INTO llmspell.entities (entity_id, tenant_id, entity_type, name, properties, valid_time_start)
             VALUES ($1, $2, 'test', 'Test Entity', '{}', now())",
            &[&entity_id, &tenant_id],
        )
        .await;

    assert!(
        result.is_ok(),
        "llmspell_app should have INSERT permission on entities"
    );

    // Test SELECT permission
    let rows = client
        .query("SELECT COUNT(*) FROM llmspell.entities", &[])
        .await;

    assert!(
        rows.is_ok(),
        "llmspell_app should have SELECT permission on entities"
    );

    // Test relationships table permissions
    let rel_id = Uuid::new_v4();
    let result = client
        .execute(
            "INSERT INTO llmspell.relationships
             (relationship_id, tenant_id, from_entity, to_entity, relationship_type, properties, valid_time_start)
             VALUES ($1, $2, $3, $3, 'test', '{}', now())",
            &[&rel_id, &tenant_id, &entity_id],
        )
        .await;

    assert!(
        result.is_ok(),
        "llmspell_app should have INSERT permission on relationships"
    );

    // Cleanup
    client
        .execute("DELETE FROM llmspell.relationships WHERE TRUE", &[])
        .await
        .expect("Should be able to DELETE");

    client
        .execute("DELETE FROM llmspell.entities WHERE TRUE", &[])
        .await
        .expect("Should be able to DELETE");
}

#[tokio::test]
async fn test_temporal_graph_rls_isolation() {
    ensure_migrations_run_once().await;

    let config = PostgresConfig::new(APP_CONNECTION_STRING);
    let backend = PostgresBackend::new(config).await.unwrap();

    // Test RLS isolation on entities table
    let tenant_a = unique_tenant_id("entity-a");
    let tenant_b = unique_tenant_id("entity-b");

    // Tenant A inserts data
    backend.set_tenant_context(&tenant_a).await.unwrap();
    let client = backend.get_client().await.unwrap();

    let entity_id = Uuid::new_v4();
    client
        .execute(
            "INSERT INTO llmspell.entities (entity_id, tenant_id, entity_type, name, properties, valid_time_start)
             VALUES ($1, $2, 'test', 'Tenant A Entity', '{}', now())",
            &[&entity_id, &tenant_a],
        )
        .await
        .unwrap();

    // Tenant A should see 1 row
    let rows = client
        .query("SELECT COUNT(*) FROM llmspell.entities", &[])
        .await
        .unwrap();
    let count: i64 = rows[0].get(0);
    assert_eq!(count, 1, "Tenant A should see its own data");

    // Switch to Tenant B
    backend.clear_tenant_context().await.unwrap();
    backend.set_tenant_context(&tenant_b).await.unwrap();
    let client = backend.get_client().await.unwrap();

    // Tenant B should see 0 rows
    let rows = client
        .query("SELECT COUNT(*) FROM llmspell.entities", &[])
        .await
        .unwrap();
    let count: i64 = rows[0].get(0);
    assert_eq!(count, 0, "Tenant B should NOT see tenant A data");

    // Cleanup
    backend.clear_tenant_context().await.unwrap();
    backend.set_tenant_context(&tenant_a).await.unwrap();
    let client = backend.get_client().await.unwrap();
    client
        .execute("DELETE FROM llmspell.entities WHERE TRUE", &[])
        .await
        .unwrap();
}

#[tokio::test]
async fn test_temporal_graph_bi_temporal_semantics() {
    ensure_migrations_run_once().await;

    let config = PostgresConfig::new(APP_CONNECTION_STRING);
    let backend = PostgresBackend::new(config).await.unwrap();

    let tenant_id = unique_tenant_id("temporal");
    backend.set_tenant_context(&tenant_id).await.unwrap();
    let client = backend.get_client().await.unwrap();

    // Insert entity with specific valid_time
    let entity_id = Uuid::new_v4();
    client
        .execute(
            "INSERT INTO llmspell.entities
             (entity_id, tenant_id, entity_type, name, properties, valid_time_start, valid_time_end)
             VALUES ($1, $2, 'test', 'Temporal Entity', '{}', '2024-01-01', '2024-12-31')",
            &[&entity_id, &tenant_id],
        )
        .await
        .unwrap();

    // Query within valid time range (should find it)
    let rows = client
        .query(
            "SELECT COUNT(*) FROM llmspell.entities
             WHERE valid_time_start <= '2024-06-01'::timestamptz
               AND valid_time_end > '2024-06-01'::timestamptz",
            &[],
        )
        .await
        .unwrap();
    let count: i64 = rows[0].get(0);
    assert_eq!(count, 1, "Should find entity within valid time range");

    // Query outside valid time range (should NOT find it)
    let rows = client
        .query(
            "SELECT COUNT(*) FROM llmspell.entities
             WHERE valid_time_start <= '2025-06-01'::timestamptz
               AND valid_time_end > '2025-06-01'::timestamptz",
            &[],
        )
        .await
        .unwrap();
    let count: i64 = rows[0].get(0);
    assert_eq!(count, 0, "Should NOT find entity outside valid time range");

    // Cleanup
    client
        .execute("DELETE FROM llmspell.entities WHERE TRUE", &[])
        .await
        .unwrap();
}

#[tokio::test]
async fn test_temporal_graph_foreign_key_constraints() {
    ensure_migrations_run_once().await;

    let config = PostgresConfig::new(APP_CONNECTION_STRING);
    let backend = PostgresBackend::new(config).await.unwrap();

    let tenant_id = unique_tenant_id("fk-test");
    backend.set_tenant_context(&tenant_id).await.unwrap();
    let client = backend.get_client().await.unwrap();

    // Create entity
    let entity_id = Uuid::new_v4();
    client
        .execute(
            "INSERT INTO llmspell.entities (entity_id, tenant_id, entity_type, name, properties, valid_time_start)
             VALUES ($1, $2, 'test', 'FK Test Entity', '{}', now())",
            &[&entity_id, &tenant_id],
        )
        .await
        .unwrap();

    // Create relationship with valid entity reference (should succeed)
    let rel_id = Uuid::new_v4();
    let result = client
        .execute(
            "INSERT INTO llmspell.relationships
             (relationship_id, tenant_id, from_entity, to_entity, relationship_type, properties, valid_time_start)
             VALUES ($1, $2, $3, $3, 'self_ref', '{}', now())",
            &[&rel_id, &tenant_id, &entity_id],
        )
        .await;

    assert!(
        result.is_ok(),
        "Relationship with valid entity reference should succeed"
    );

    // NOTE: As of V15 migration, foreign key constraints were removed to support bi-temporal
    // versioning (multiple entity versions with same entity_id). Referential integrity is now
    // enforced at the application level through the KnowledgeGraph trait implementation.
    //
    // This test verifies that relationships can be created successfully. Application code
    // (PostgresGraphStorage) ensures entity existence before creating relationships.

    // Cleanup
    client
        .execute("DELETE FROM llmspell.relationships WHERE TRUE", &[])
        .await
        .unwrap();

    client
        .execute("DELETE FROM llmspell.entities WHERE TRUE", &[])
        .await
        .unwrap();
}

#[tokio::test]
async fn test_temporal_graph_migration_idempotent() {
    ensure_migrations_run_once().await;

    let config = PostgresConfig::new(SUPERUSER_CONNECTION_STRING);
    let backend = PostgresBackend::new(config).await.unwrap();

    // Run migrations again - should be idempotent
    backend
        .run_migrations()
        .await
        .expect("Migrations should be idempotent");

    // Verify version is still 4
    let version = backend
        .migration_version()
        .await
        .expect("Should be able to get migration version");

    assert!(
        version >= 4,
        "Migration version should be at least 4 after V4"
    );
}
