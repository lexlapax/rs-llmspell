//! Integration tests for PostgreSQL backend (Phase 13b.2)
//!
//! These tests require a running PostgreSQL instance with VectorChord.
//! Run: `cd docker/postgres && docker compose up -d`
//!
//! Connection: postgresql://llmspell:llmspell_dev_pass@localhost:5432/llmspell_dev

#![cfg(feature = "postgres")]

use llmspell_storage::{PostgresBackend, PostgresConfig};
use tokio::sync::OnceCell;

static MIGRATION_INIT: OnceCell<()> = OnceCell::const_new();

/// Test database connection string
// Admin connection for migrations (llmspell user has CREATE TABLE privileges)
const ADMIN_CONNECTION_STRING: &str =
    "postgresql://llmspell:llmspell_dev_pass@localhost:5432/llmspell_dev";

// Application connection for queries (llmspell_app enforces RLS, no schema modification)
const APP_CONNECTION_STRING: &str =
    "postgresql://llmspell_app:llmspell_app_pass@localhost:5432/llmspell_dev";

#[tokio::test]
async fn test_postgres_backend_creation() {
    let config = PostgresConfig::new(APP_CONNECTION_STRING);
    let backend = PostgresBackend::new(config).await;

    assert!(
        backend.is_ok(),
        "Failed to create PostgreSQL backend: {:?}",
        backend.err()
    );
}

#[tokio::test]
async fn test_postgres_backend_health_check() {
    let config = PostgresConfig::new(APP_CONNECTION_STRING);
    let backend = PostgresBackend::new(config)
        .await
        .expect("Failed to create backend");

    let is_healthy = backend.is_healthy().await;
    assert!(is_healthy, "PostgreSQL backend should be healthy");
}

#[tokio::test]
async fn test_postgres_pool_status() {
    let config = PostgresConfig::new(APP_CONNECTION_STRING).with_max_pool_size(10);
    let backend = PostgresBackend::new(config)
        .await
        .expect("Failed to create backend");

    let status = backend.pool_status();
    assert_eq!(status.max_size, 10, "Max pool size should be 10");
    assert!(
        status.available <= status.max_size,
        "Available connections should not exceed max size"
    );
}

#[tokio::test]
async fn test_tenant_context_management() {
    let config = PostgresConfig::new(APP_CONNECTION_STRING);
    let backend = PostgresBackend::new(config)
        .await
        .expect("Failed to create backend");

    // Initially no tenant context
    assert_eq!(
        backend.get_tenant_context().await,
        None,
        "Initial tenant context should be None"
    );

    // Set tenant context
    backend
        .set_tenant_context("tenant_123")
        .await
        .expect("Failed to set tenant context");

    assert_eq!(
        backend.get_tenant_context().await,
        Some("tenant_123".to_string()),
        "Tenant context should be set"
    );

    // Clear tenant context
    backend
        .clear_tenant_context()
        .await
        .expect("Failed to clear tenant context");

    assert_eq!(
        backend.get_tenant_context().await,
        None,
        "Tenant context should be cleared"
    );
}

#[tokio::test]
async fn test_tenant_context_with_multiple_tenants() {
    let config = PostgresConfig::new(APP_CONNECTION_STRING);
    let backend = PostgresBackend::new(config)
        .await
        .expect("Failed to create backend");

    // Set tenant A
    backend
        .set_tenant_context("tenant_a")
        .await
        .expect("Failed to set tenant A");
    assert_eq!(
        backend.get_tenant_context().await,
        Some("tenant_a".to_string())
    );

    // Switch to tenant B
    backend
        .set_tenant_context("tenant_b")
        .await
        .expect("Failed to set tenant B");
    assert_eq!(
        backend.get_tenant_context().await,
        Some("tenant_b".to_string())
    );

    // Switch to tenant C
    backend
        .set_tenant_context("tenant_c")
        .await
        .expect("Failed to set tenant C");
    assert_eq!(
        backend.get_tenant_context().await,
        Some("tenant_c".to_string())
    );
}

#[tokio::test]
async fn test_postgres_config_validation() {
    // Empty connection string should fail validation
    let config = PostgresConfig::new("");
    assert!(
        config.validate().is_err(),
        "Empty connection string should fail validation"
    );

    // Valid connection string should pass
    let config = PostgresConfig::new(APP_CONNECTION_STRING);
    assert!(
        config.validate().is_ok(),
        "Valid connection string should pass validation"
    );

    // Zero pool size should fail
    let config = PostgresConfig::new(APP_CONNECTION_STRING).with_max_pool_size(0);
    assert!(
        config.validate().is_err(),
        "Zero pool size should fail validation"
    );

    // Excessive pool size should fail
    let config = PostgresConfig::new(APP_CONNECTION_STRING).with_max_pool_size(101);
    assert!(
        config.validate().is_err(),
        "Pool size > 100 should fail validation"
    );

    // Valid pool size should pass
    let config = PostgresConfig::new(APP_CONNECTION_STRING).with_max_pool_size(50);
    assert!(
        config.validate().is_ok(),
        "Valid pool size should pass validation"
    );
}

#[tokio::test]
async fn test_postgres_config_builder_pattern() {
    let config = PostgresConfig::new(APP_CONNECTION_STRING)
        .with_max_pool_size(15)
        .with_connection_timeout(10000)
        .with_rls(false);

    assert_eq!(config.connection_string, APP_CONNECTION_STRING);
    assert_eq!(config.max_pool_size, 15);
    assert_eq!(config.connection_timeout_ms, 10000);
    assert!(!config.enable_rls);
}

#[tokio::test]
async fn test_postgres_config_defaults() {
    let config = PostgresConfig::new(APP_CONNECTION_STRING);

    assert_eq!(config.max_pool_size, 20, "Default pool size should be 20");
    assert_eq!(
        config.connection_timeout_ms, 5000,
        "Default timeout should be 5000ms"
    );
    assert!(config.enable_rls, "RLS should be enabled by default");
}

#[tokio::test]
async fn test_invalid_connection_string() {
    let config = PostgresConfig::new("invalid://connection/string");
    let result = PostgresBackend::new(config).await;

    assert!(
        result.is_err(),
        "Invalid connection string should return error"
    );
}

#[tokio::test]
async fn test_connection_to_nonexistent_database() {
    let config = PostgresConfig::new(
        "postgresql://llmspell:llmspell_dev_pass@localhost:5432/nonexistent_db",
    );

    // Pool creation succeeds (lazy connection), but health check should fail
    let backend = PostgresBackend::new(config).await;

    if let Ok(backend) = backend {
        let is_healthy = backend.is_healthy().await;
        assert!(
            !is_healthy,
            "Health check should fail for nonexistent database"
        );
    }
    // If backend creation fails immediately, that's also acceptable
}

#[tokio::test]
async fn test_multiple_backends_same_database() {
    let config = PostgresConfig::new(APP_CONNECTION_STRING).with_max_pool_size(5);

    // Create multiple backends to same database
    let backend1 = PostgresBackend::new(config.clone())
        .await
        .expect("Failed to create backend 1");
    let backend2 = PostgresBackend::new(config.clone())
        .await
        .expect("Failed to create backend 2");
    let backend3 = PostgresBackend::new(config)
        .await
        .expect("Failed to create backend 3");

    // All should be healthy
    assert!(backend1.is_healthy().await);
    assert!(backend2.is_healthy().await);
    assert!(backend3.is_healthy().await);

    // Set different tenant contexts
    backend1.set_tenant_context("tenant_1").await.unwrap();
    backend2.set_tenant_context("tenant_2").await.unwrap();
    backend3.set_tenant_context("tenant_3").await.unwrap();

    // Each backend should maintain its own tenant context
    assert_eq!(
        backend1.get_tenant_context().await,
        Some("tenant_1".to_string())
    );
    assert_eq!(
        backend2.get_tenant_context().await,
        Some("tenant_2".to_string())
    );
    assert_eq!(
        backend3.get_tenant_context().await,
        Some("tenant_3".to_string())
    );
}

#[tokio::test]
async fn test_rls_disabled_tenant_context() {
    let config = PostgresConfig::new(APP_CONNECTION_STRING).with_rls(false);
    let backend = PostgresBackend::new(config)
        .await
        .expect("Failed to create backend");

    // Setting tenant context with RLS disabled should still work
    // (just won't execute SET LOCAL command)
    backend
        .set_tenant_context("tenant_123")
        .await
        .expect("Setting tenant context should work even with RLS disabled");

    assert_eq!(
        backend.get_tenant_context().await,
        Some("tenant_123".to_string())
    );
}

#[tokio::test]
async fn test_concurrent_pool_access() {
    use std::sync::Arc;
    use tokio::task::JoinSet;

    let config = PostgresConfig::new(APP_CONNECTION_STRING).with_max_pool_size(10);
    let backend = Arc::new(
        PostgresBackend::new(config)
            .await
            .expect("Failed to create backend"),
    );

    // Spawn 20 concurrent tasks (more than pool size)
    let mut tasks = JoinSet::new();
    for i in 0..20 {
        let backend_clone = Arc::clone(&backend);
        tasks.spawn(async move {
            let tenant_id = format!("tenant_{}", i);
            backend_clone
                .set_tenant_context(&tenant_id)
                .await
                .expect("Failed to set tenant context");
            backend_clone.is_healthy().await
        });
    }

    // All tasks should complete successfully
    let mut success_count = 0;
    while let Some(result) = tasks.join_next().await {
        assert!(result.is_ok(), "Task should not panic");
        assert!(result.unwrap(), "Health check should pass");
        success_count += 1;
    }

    assert_eq!(success_count, 20, "All 20 tasks should complete");
}

// Migration Tests (Phase 13b.2.6)

/// Ensure migrations are run exactly once for all tests
///
/// During active development, we modify migration files (e.g., fixing Phase 13b.3 RLS policies).
/// Refinery stores migration hashes and throws errors when files change. This helper resets
/// migration state ONCE at test suite startup, then all tests share the migrated schema.
///
/// IMPORTANT: This is dev/test-only behavior. Production migrations should never be modified
/// after deployment - create new migrations instead (e.g., V3__fix_rls_policies.sql).
async fn ensure_migrations_run_once() {
    MIGRATION_INIT.get_or_init(|| async {
        let config = PostgresConfig::new(ADMIN_CONNECTION_STRING);
        let backend = PostgresBackend::new(config)
            .await
            .expect("Failed to create backend for migration init");

        let client = backend.get_client().await.expect("Failed to get client");

        // Drop refinery_schema_history to allow re-running migrations with modified files
        let _ = client
            .execute("DROP TABLE IF EXISTS refinery_schema_history", &[])
            .await;

        // Drop llmspell schema to ensure clean state
        let _ = client
            .execute("DROP SCHEMA IF EXISTS llmspell CASCADE", &[])
            .await;

        // Recreate llmspell schema (required by V1__initial_setup.sql)
        let _ = client
            .execute("CREATE SCHEMA IF NOT EXISTS llmspell", &[])
            .await;

        // Grant privileges to llmspell_app user (for RLS tests)
        let _ = client
            .execute("GRANT ALL PRIVILEGES ON SCHEMA llmspell TO llmspell_app", &[])
            .await;
        let _ = client
            .execute("GRANT ALL PRIVILEGES ON ALL TABLES IN SCHEMA llmspell TO llmspell_app", &[])
            .await;
        let _ = client
            .execute("GRANT ALL PRIVILEGES ON ALL SEQUENCES IN SCHEMA llmspell TO llmspell_app", &[])
            .await;
        let _ = client
            .execute("ALTER DEFAULT PRIVILEGES IN SCHEMA llmspell GRANT ALL ON TABLES TO llmspell_app", &[])
            .await;
        let _ = client
            .execute("ALTER DEFAULT PRIVILEGES IN SCHEMA llmspell GRANT ALL ON SEQUENCES TO llmspell_app", &[])
            .await;

        // Run migrations
        backend
            .run_migrations()
            .await
            .expect("Failed to run migrations during test initialization");
    }).await;
}

#[tokio::test]
async fn test_run_migrations() {
    // Ensure migrations run once before all tests
    ensure_migrations_run_once().await;

    let config = PostgresConfig::new(APP_CONNECTION_STRING);
    let backend = PostgresBackend::new(config)
        .await
        .expect("Failed to create backend");

    // Migrations should already be applied
    let version = backend
        .migration_version()
        .await
        .expect("Failed to get migration version");

    assert!(version >= 2, "Migrations should already be applied");
}

#[tokio::test]
async fn test_migration_version() {
    // Ensure migrations run once before all tests
    ensure_migrations_run_once().await;

    let config = PostgresConfig::new(APP_CONNECTION_STRING);
    let backend = PostgresBackend::new(config)
        .await
        .expect("Failed to create backend");

    // Check migration version (should be 2 after V1 + V2)
    let version = backend
        .migration_version()
        .await
        .expect("Failed to get migration version");

    assert!(
        version >= 2,
        "Migration version should be at least 2 after running V1 and V2 migrations"
    );
}

#[tokio::test]
async fn test_migrations_idempotent() {
    // Ensure migrations run once before all tests
    ensure_migrations_run_once().await;

    let config = PostgresConfig::new(APP_CONNECTION_STRING);
    let backend = PostgresBackend::new(config)
        .await
        .expect("Failed to create backend");

    // Run migrations multiple times - should be idempotent
    // (First run was in ensure_migrations_run_once)
    backend
        .run_migrations()
        .await
        .expect("Second migration run failed (should be idempotent)");

    backend
        .run_migrations()
        .await
        .expect("Third migration run failed (should be idempotent)");

    // Version should still be consistent
    let version = backend
        .migration_version()
        .await
        .expect("Failed to get migration version");

    assert!(
        version >= 2,
        "Migration version should be at least 2 after V1 and V2"
    );
}
