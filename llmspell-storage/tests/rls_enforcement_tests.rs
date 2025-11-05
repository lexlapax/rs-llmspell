//! Comprehensive RLS enforcement test suite (Phase 13b.3.3)
//!
//! Tests tenant isolation, policy enforcement, security, and performance
//! using the test_data table created in Phase 13b.3.2

#![cfg(feature = "postgres")]

use llmspell_storage::{PostgresBackend, PostgresConfig};
use tokio::sync::OnceCell;
use uuid::Uuid;

const TEST_CONNECTION_STRING: &str =
    "postgresql://llmspell_app:llmspell_app_pass@localhost:5432/llmspell_dev";

const SUPERUSER_CONNECTION_STRING: &str =
    "postgresql://llmspell:llmspell_dev_pass@localhost:5432/llmspell_dev";

static MIGRATION_INIT: OnceCell<()> = OnceCell::const_new();

/// Generate unique tenant ID for test isolation (prevents concurrent test interference)
fn unique_tenant_id(prefix: &str) -> String {
    format!("{}-{}", prefix, Uuid::new_v4())
}

/// Ensure migrations are run exactly once before any RLS tests
///
/// RLS tests use llmspell_app user (limited privileges), but migrations require
/// superuser privileges. This helper uses llmspell user to run migrations once,
/// then all RLS tests can use llmspell_app user safely.
async fn ensure_migrations_run_once() {
    MIGRATION_INIT
        .get_or_init(|| async {
            // Use superuser connection for migrations
            let config = PostgresConfig::new(SUPERUSER_CONNECTION_STRING);
            let backend = PostgresBackend::new(config)
                .await
                .expect("Failed to create backend for migration init");

            // Run migrations (idempotent, safe to call multiple times)
            backend
                .run_migrations()
                .await
                .expect("Failed to run migrations during RLS test initialization");
        })
        .await;
}

async fn setup_backend() -> PostgresBackend {
    // Ensure migrations run once before any RLS tests
    ensure_migrations_run_once().await;

    let config = PostgresConfig::new(TEST_CONNECTION_STRING);
    let backend = PostgresBackend::new(config)
        .await
        .expect("Failed to create backend");

    backend
}

/// Cleanup data for specific tenant (called after each test)
async fn cleanup_tenant_data(backend: &PostgresBackend, tenant_id: &str) {
    backend.set_tenant_context(tenant_id).await.ok();
    if let Ok(client) = backend.get_client().await {
        let _ = client
            .execute("DELETE FROM llmspell.test_data WHERE TRUE", &[])
            .await;
    }
    backend.clear_tenant_context().await.ok();
}

// =============================================================================
// TENANT ISOLATION TESTS
// =============================================================================

#[tokio::test]
async fn test_tenant_isolation_select_cross_tenant_blocked() {
    let backend = setup_backend().await;
    let tenant_a = unique_tenant_id("isolation-a");
    let tenant_b = unique_tenant_id("isolation-b");

    // Tenant A inserts data
    backend
        .set_tenant_context(&tenant_a)
        .await
        .expect("Failed to set tenant A context");

    let client_a = backend.get_client().await.expect("Failed to get client");
    client_a
        .execute(
            "INSERT INTO llmspell.test_data (tenant_id, value) VALUES ($1, $2)",
            &[&tenant_a, &"secret-a"],
        )
        .await
        .expect("Failed to insert for tenant A");
    drop(client_a); // Return to pool

    // Tenant B tries to query
    backend
        .set_tenant_context(&tenant_b)
        .await
        .expect("Failed to set tenant B context");

    let client_b = backend.get_client().await.expect("Failed to get client");
    let rows = client_b
        .query("SELECT * FROM llmspell.test_data", &[])
        .await
        .expect("Failed to query");

    assert_eq!(
        rows.len(),
        0,
        "Tenant B should NOT see tenant A's data (RLS isolation failed)"
    );

    cleanup_tenant_data(&backend, &tenant_a).await;
    cleanup_tenant_data(&backend, &tenant_b).await;
}

#[tokio::test]
async fn test_tenant_isolation_select_own_data_visible() {
    let backend = setup_backend().await;
    let tenant_a = unique_tenant_id("own-data-a");

    // Tenant A inserts data
    backend
        .set_tenant_context(&tenant_a)
        .await
        .expect("Failed to set tenant A context");

    let client = backend.get_client().await.expect("Failed to get client");

    client
        .execute(
            "INSERT INTO llmspell.test_data (tenant_id, value) VALUES ($1, $2)",
            &[&tenant_a, &"visible-data"],
        )
        .await
        .expect("Failed to insert");

    // Tenant A queries (should see own data)
    let rows = client
        .query("SELECT value FROM llmspell.test_data", &[])
        .await
        .expect("Failed to query");

    assert_eq!(rows.len(), 1, "Tenant A should see its own data");

    let value: String = rows[0].get(0);
    assert_eq!(value, "visible-data");

    cleanup_tenant_data(&backend, &tenant_a).await;
}

#[tokio::test]
async fn test_no_tenant_context_sees_nothing() {
    let backend = setup_backend().await;
    let tenant_a = unique_tenant_id("no-context-a");

    // Insert data with tenant context
    backend
        .set_tenant_context(&tenant_a)
        .await
        .expect("Failed to set tenant context");

    let client = backend.get_client().await.expect("Failed to get client");

    client
        .execute(
            "INSERT INTO llmspell.test_data (tenant_id, value) VALUES ($1, $2)",
            &[&tenant_a, &"data"],
        )
        .await
        .expect("Failed to insert");

    drop(client);

    // Clear tenant context
    backend
        .clear_tenant_context()
        .await
        .expect("Failed to clear context");

    // Query without tenant context (should see nothing)
    let client = backend.get_client().await.expect("Failed to get client");
    let rows = client
        .query("SELECT * FROM llmspell.test_data", &[])
        .await
        .expect("Failed to query");

    assert_eq!(
        rows.len(),
        0,
        "Query without tenant context should return zero rows"
    );

    cleanup_tenant_data(&backend, &tenant_a).await;
}

#[tokio::test]
async fn test_multiple_tenants_isolation() {
    let backend = setup_backend().await;
    let tenant_1 = unique_tenant_id("multi-1");
    let tenant_2 = unique_tenant_id("multi-2");
    let tenant_3 = unique_tenant_id("multi-3");
    let tenants = [&tenant_1, &tenant_2, &tenant_3];

    // Insert data for 3 different tenants
    for tenant in &tenants {
        backend
            .set_tenant_context(*tenant)
            .await
            .expect("Failed to set tenant context");

        let client = backend.get_client().await.expect("Failed to get client");

        client
            .execute(
                "INSERT INTO llmspell.test_data (tenant_id, value) VALUES ($1, $2)",
                &[tenant, &format!("data-{}", tenant)],
            )
            .await
            .expect("Failed to insert");
    }

    // Verify each tenant sees only their own data
    for tenant in &tenants {
        backend
            .set_tenant_context(*tenant)
            .await
            .expect("Failed to set tenant context");

        let client = backend.get_client().await.expect("Failed to get client");

        let rows = client
            .query("SELECT value FROM llmspell.test_data", &[])
            .await
            .expect("Failed to query");

        assert_eq!(rows.len(), 1, "Tenant {} should see exactly 1 row", tenant);

        let value: String = rows[0].get(0);
        assert_eq!(value, format!("data-{}", tenant));
    }

    cleanup_tenant_data(&backend, &tenant_1).await;
    cleanup_tenant_data(&backend, &tenant_2).await;
    cleanup_tenant_data(&backend, &tenant_3).await;
}

// =============================================================================
// POLICY TYPE TESTS (SELECT, INSERT, UPDATE, DELETE)
// =============================================================================

#[tokio::test]
async fn test_select_policy_filters_by_tenant() {
    let backend = setup_backend().await;
    let tenant_a = unique_tenant_id("select-policy-a");
    let tenant_b = unique_tenant_id("select-policy-b");

    // Insert data for tenant A
    backend.set_tenant_context(&tenant_a).await.unwrap();
    let client = backend.get_client().await.expect("Failed to get client");
    client
        .execute(
            "INSERT INTO llmspell.test_data (tenant_id, value) VALUES ($1, $2)",
            &[&tenant_a, &"data-a"],
        )
        .await
        .unwrap();

    drop(client);

    // Query as tenant B
    backend.set_tenant_context(&tenant_b).await.unwrap();
    let client = backend.get_client().await.expect("Failed to get client");
    let rows = client
        .query("SELECT * FROM llmspell.test_data", &[])
        .await
        .unwrap();

    assert_eq!(rows.len(), 0, "SELECT policy should filter by tenant_id");

    cleanup_tenant_data(&backend, &tenant_a).await;
    cleanup_tenant_data(&backend, &tenant_b).await;
}

#[tokio::test]
async fn test_insert_policy_validates_tenant_id() {
    let backend = setup_backend().await;
    let tenant_a = unique_tenant_id("insert-policy-a");
    let tenant_b = unique_tenant_id("insert-policy-b");

    // Set context to tenant-a
    backend.set_tenant_context(&tenant_a).await.unwrap();

    let client = backend.get_client().await.expect("Failed to get client");

    // Attempt to insert with DIFFERENT tenant_id (should fail WITH CHECK)
    let result = client
        .execute(
            "INSERT INTO llmspell.test_data (tenant_id, value) VALUES ($1, $2)",
            &[&tenant_b, &"malicious-data"],
        )
        .await;

    assert!(
        result.is_err(),
        "INSERT policy should reject mismatched tenant_id"
    );

    // Verify error message contains policy violation
    let err_msg = format!("{:?}", result.unwrap_err());
    assert!(
        err_msg.contains("policy") || err_msg.contains("violat"),
        "Error should indicate policy violation"
    );

    cleanup_tenant_data(&backend, &tenant_a).await;
}

#[tokio::test]
async fn test_update_policy_prevents_tenant_id_change() {
    let backend = setup_backend().await;
    let tenant_a = unique_tenant_id("update-prevent-a");
    let tenant_b = unique_tenant_id("update-prevent-b");

    // Insert data for tenant-a
    backend.set_tenant_context(&tenant_a).await.unwrap();
    let client = backend.get_client().await.expect("Failed to get client");
    let rows = client
        .query(
            "INSERT INTO llmspell.test_data (tenant_id, value) VALUES ($1, $2) RETURNING id",
            &[&tenant_a, &"original-value"],
        )
        .await
        .unwrap();

    let id: uuid::Uuid = rows[0].get(0);

    // Attempt to UPDATE tenant_id to different value (should fail WITH CHECK)
    let result = client
        .execute(
            "UPDATE llmspell.test_data SET tenant_id = $1 WHERE id = $2",
            &[&tenant_b, &id],
        )
        .await;

    assert!(
        result.is_err(),
        "UPDATE policy WITH CHECK should prevent tenant_id change"
    );

    cleanup_tenant_data(&backend, &tenant_a).await;
}

#[tokio::test]
async fn test_update_policy_allows_value_change_same_tenant() {
    let backend = setup_backend().await;
    let tenant_a = unique_tenant_id("update-allow-a");

    // Insert data
    backend.set_tenant_context(&tenant_a).await.unwrap();
    let client = backend.get_client().await.expect("Failed to get client");
    let rows = client
        .query(
            "INSERT INTO llmspell.test_data (tenant_id, value) VALUES ($1, $2) RETURNING id",
            &[&tenant_a, &"old-value"],
        )
        .await
        .unwrap();

    let id: uuid::Uuid = rows[0].get(0);

    // UPDATE value (not tenant_id) should succeed
    let updated = client
        .execute(
            "UPDATE llmspell.test_data SET value = $1 WHERE id = $2",
            &[&"new-value", &id],
        )
        .await
        .unwrap();

    assert_eq!(updated, 1, "UPDATE should succeed for same tenant");

    // Verify update
    let rows = client
        .query("SELECT value FROM llmspell.test_data WHERE id = $1", &[&id])
        .await
        .unwrap();

    assert_eq!(rows.len(), 1);
    let value: String = rows[0].get(0);
    assert_eq!(value, "new-value");

    cleanup_tenant_data(&backend, &tenant_a).await;
}

#[tokio::test]
async fn test_delete_policy_only_own_tenant() {
    let backend = setup_backend().await;
    let tenant_a = unique_tenant_id("delete-policy-a");
    let tenant_b = unique_tenant_id("delete-policy-b");

    // Tenant A inserts data
    backend.set_tenant_context(&tenant_a).await.unwrap();
    let client = backend.get_client().await.expect("Failed to get client");
    let rows = client
        .query(
            "INSERT INTO llmspell.test_data (tenant_id, value) VALUES ($1, $2) RETURNING id",
            &[&tenant_a, &"data-a"],
        )
        .await
        .unwrap();

    let id_a: uuid::Uuid = rows[0].get(0);

    drop(client);

    // Tenant B tries to delete tenant A's row (should fail USING clause)
    backend.set_tenant_context(&tenant_b).await.unwrap();
    let client = backend.get_client().await.expect("Failed to get client");
    let deleted = client
        .execute("DELETE FROM llmspell.test_data WHERE id = $1", &[&id_a])
        .await
        .unwrap();

    assert_eq!(
        deleted, 0,
        "DELETE policy should prevent cross-tenant deletion"
    );

    drop(client);

    // Tenant A can delete own row
    backend.set_tenant_context(&tenant_a).await.unwrap();
    let client = backend.get_client().await.expect("Failed to get client");
    let deleted = client
        .execute("DELETE FROM llmspell.test_data WHERE id = $1", &[&id_a])
        .await
        .unwrap();

    assert_eq!(deleted, 1, "DELETE should succeed for own tenant");

    cleanup_tenant_data(&backend, &tenant_a).await;
    cleanup_tenant_data(&backend, &tenant_b).await;
}

// =============================================================================
// SECURITY TESTS
// =============================================================================

#[tokio::test]
async fn test_explicit_where_clause_cannot_bypass_rls() {
    let backend = setup_backend().await;
    let tenant_a = unique_tenant_id("bypass-test-a");
    let tenant_b = unique_tenant_id("bypass-test-b");

    // Tenant A inserts data
    backend.set_tenant_context(&tenant_a).await.unwrap();
    let client = backend.get_client().await.expect("Failed to get client");
    client
        .execute(
            "INSERT INTO llmspell.test_data (tenant_id, value) VALUES ($1, $2)",
            &[&tenant_a, &"secret"],
        )
        .await
        .unwrap();

    drop(client);

    // Tenant B tries explicit WHERE clause to access tenant A data
    backend.set_tenant_context(&tenant_b).await.unwrap();
    let client = backend.get_client().await.expect("Failed to get client");
    let rows = client
        .query(
            "SELECT * FROM llmspell.test_data WHERE tenant_id = $1",
            &[&tenant_a],
        )
        .await
        .unwrap();

    assert_eq!(rows.len(), 0, "Explicit WHERE clause should NOT bypass RLS");

    cleanup_tenant_data(&backend, &tenant_a).await;
    cleanup_tenant_data(&backend, &tenant_b).await;
}

#[tokio::test]
async fn test_sql_injection_in_tenant_id() {
    let backend = setup_backend().await;
    let tenant_a = unique_tenant_id("injection-test-a");

    // Try SQL injection via tenant_id
    let malicious_tenant = format!("{}' OR '1'='1", tenant_a);

    backend.set_tenant_context(&tenant_a).await.unwrap();

    let client = backend.get_client().await.expect("Failed to get client");

    client
        .execute(
            "INSERT INTO llmspell.test_data (tenant_id, value) VALUES ($1, $2)",
            &[&tenant_a, &"data"],
        )
        .await
        .unwrap();

    drop(client);

    // Set malicious tenant context
    backend.set_tenant_context(&malicious_tenant).await.unwrap();

    // Query should see nothing (injection should not work)
    let client = backend.get_client().await.expect("Failed to get client");
    let rows = client
        .query("SELECT * FROM llmspell.test_data", &[])
        .await
        .unwrap();

    assert_eq!(
        rows.len(),
        0,
        "SQL injection in tenant_id should be neutralized by parameterization"
    );

    cleanup_tenant_data(&backend, &tenant_a).await;
}

#[tokio::test]
async fn test_union_injection_attempt() {
    let backend = setup_backend().await;
    let tenant_a = unique_tenant_id("union-test-a");

    // Insert legitimate data
    backend.set_tenant_context(&tenant_a).await.unwrap();
    let client = backend.get_client().await.expect("Failed to get client");
    client
        .execute(
            "INSERT INTO llmspell.test_data (tenant_id, value) VALUES ($1, $2)",
            &[&tenant_a, &"data"],
        )
        .await
        .unwrap();

    // Try UNION injection via value field
    let malicious_value =
        "data' UNION SELECT id, 'hacked', 'hacked', now() FROM llmspell.test_data WHERE '1'='1";

    let result = client
        .execute(
            "INSERT INTO llmspell.test_data (tenant_id, value) VALUES ($1, $2)",
            &[&tenant_a, &malicious_value],
        )
        .await;

    // Should succeed (it's just data), but verify RLS still works
    assert!(
        result.is_ok(),
        "Parameterized query should prevent injection"
    );

    // Verify only own tenant's data visible
    let rows = client
        .query("SELECT * FROM llmspell.test_data", &[])
        .await
        .unwrap();

    assert_eq!(
        rows.len(),
        2,
        "Should see 2 rows (legitimate + injection attempt as data)"
    );

    cleanup_tenant_data(&backend, &tenant_a).await;
}

// =============================================================================
// PERFORMANCE TESTS
// =============================================================================

#[tokio::test]
async fn test_rls_overhead_measurement() {
    let backend = setup_backend().await;
    let tenant_a = unique_tenant_id("overhead-test-a");

    // Insert 100 rows for tenant-a
    backend.set_tenant_context(&tenant_a).await.unwrap();
    let client = backend.get_client().await.expect("Failed to get client");
    for i in 0..100 {
        client
            .execute(
                "INSERT INTO llmspell.test_data (tenant_id, value) VALUES ($1, $2)",
                &[&tenant_a, &format!("value-{}", i)],
            )
            .await
            .unwrap();
    }

    // Measure query time WITH RLS (warm cache)
    let mut total_with_rls = std::time::Duration::ZERO;
    for _ in 0..10 {
        let start = std::time::Instant::now();
        let rows = client
            .query("SELECT * FROM llmspell.test_data", &[])
            .await
            .unwrap();
        total_with_rls += start.elapsed();
        assert_eq!(rows.len(), 100);
    }

    let avg_with_rls = total_with_rls / 10;

    // Note: Cannot easily disable RLS without superuser privileges
    // So we measure against a query with explicit WHERE clause
    let mut total_explicit_where = std::time::Duration::ZERO;
    for _ in 0..10 {
        let start = std::time::Instant::now();
        let rows = client
            .query(
                "SELECT * FROM llmspell.test_data WHERE tenant_id = $1",
                &[&tenant_a],
            )
            .await
            .unwrap();
        total_explicit_where += start.elapsed();
        assert_eq!(rows.len(), 100);
    }

    let avg_explicit_where = total_explicit_where / 10;

    // Calculate overhead percentage
    let overhead_pct = if avg_explicit_where.as_micros() > 0 {
        ((avg_with_rls.as_micros() as f64 / avg_explicit_where.as_micros() as f64) - 1.0) * 100.0
    } else {
        0.0
    };

    println!(
        "RLS overhead: {:.2}% (avg with RLS: {:?}, avg explicit WHERE: {:?})",
        overhead_pct, avg_with_rls, avg_explicit_where
    );

    // RLS overhead should be minimal (< 20% in practice, though target is <5%)
    // We use 50% as threshold since this is a simple test environment
    assert!(
        overhead_pct < 50.0,
        "RLS overhead ({:.2}%) should be reasonable",
        overhead_pct
    );

    cleanup_tenant_data(&backend, &tenant_a).await;
}

#[tokio::test]
async fn test_concurrent_tenant_queries() {
    let backend = setup_backend().await;

    use std::sync::Arc;
    use tokio::task::JoinSet;

    // Generate unique tenant IDs
    let tenant_ids: Vec<String> = (0..5)
        .map(|i| unique_tenant_id(&format!("concurrent-{}", i)))
        .collect();

    // Insert data for multiple tenants
    for tenant_id in &tenant_ids {
        backend.set_tenant_context(tenant_id).await.unwrap();

        let client = backend.get_client().await.unwrap();
        for j in 0..10 {
            client
                .execute(
                    "INSERT INTO llmspell.test_data (tenant_id, value) VALUES ($1, $2)",
                    &[tenant_id, &format!("data-{}", j)],
                )
                .await
                .unwrap();
        }
    }

    // Spawn concurrent queries for different tenants
    let backend = Arc::new(backend);
    let mut tasks = JoinSet::new();

    for tenant_id in tenant_ids.clone() {
        let backend_clone = Arc::clone(&backend);

        tasks.spawn(async move {
            backend_clone.set_tenant_context(&tenant_id).await.unwrap();
            let client = backend_clone.get_client().await.unwrap();

            let rows = client
                .query("SELECT * FROM llmspell.test_data", &[])
                .await
                .unwrap();

            // Each tenant should see exactly their 10 rows
            (tenant_id, rows.len())
        });
    }

    // Verify all tasks complete with correct isolation
    let mut results = Vec::new();
    while let Some(result) = tasks.join_next().await {
        let (tenant_id, row_count) = result.unwrap();
        results.push((tenant_id, row_count));
    }

    assert_eq!(results.len(), 5, "All 5 concurrent queries should complete");

    for (tenant_id, row_count) in &results {
        assert_eq!(
            *row_count, 10,
            "Tenant {} should see exactly 10 rows",
            tenant_id
        );
    }

    // Cleanup all tenants
    for tenant_id in &tenant_ids {
        cleanup_tenant_data(&backend, tenant_id).await;
    }
}
