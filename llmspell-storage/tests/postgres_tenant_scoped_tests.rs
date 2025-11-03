//! Integration tests for TenantScoped trait implementation on PostgresBackend (Phase 13b.3.4)
//!
//! These tests validate that PostgresBackend correctly implements the TenantScoped trait,
//! providing a standard interface for tenant context management across all backends.

#![cfg(feature = "postgres")]

use llmspell_core::{state::StateScope, TenantScoped};
use llmspell_storage::{PostgresBackend, PostgresConfig};

const TEST_CONNECTION_STRING: &str =
    "postgresql://llmspell:llmspell_dev_pass@localhost:5432/llmspell_dev";

async fn setup_backend() -> PostgresBackend {
    let config = PostgresConfig::new(TEST_CONNECTION_STRING);
    PostgresBackend::new(config)
        .await
        .expect("Failed to create backend")
}

#[tokio::test]
async fn test_tenant_scoped_trait_tenant_id() {
    let backend = setup_backend().await;

    // Initially no tenant context
    assert_eq!(
        backend.tenant_id().await,
        None,
        "Should have no tenant context initially"
    );

    // Set tenant context using TenantScoped trait
    TenantScoped::set_tenant_context(&backend, "test-tenant-1".to_string(), StateScope::Global)
        .await
        .expect("Failed to set tenant context");

    // Verify tenant ID via TenantScoped trait
    assert_eq!(
        backend.tenant_id().await,
        Some("test-tenant-1".to_string()),
        "Should return tenant ID via TenantScoped trait"
    );
}

#[tokio::test]
async fn test_tenant_scoped_scope_returns_global() {
    let backend = setup_backend().await;

    // PostgreSQL backend uses Global scope
    assert_eq!(
        backend.scope(),
        &StateScope::Global,
        "PostgreSQL backend should use Global scope"
    );
}

#[tokio::test]
async fn test_tenant_scoped_set_tenant_context_multiple_tenants() {
    let backend = setup_backend().await;

    // Set tenant A
    TenantScoped::set_tenant_context(&backend, "tenant-a".to_string(), StateScope::Global)
        .await
        .expect("Failed to set tenant A");
    assert_eq!(backend.tenant_id().await, Some("tenant-a".to_string()));

    // Switch to tenant B
    TenantScoped::set_tenant_context(&backend, "tenant-b".to_string(), StateScope::Global)
        .await
        .expect("Failed to set tenant B");
    assert_eq!(backend.tenant_id().await, Some("tenant-b".to_string()));

    // Switch back to tenant A
    TenantScoped::set_tenant_context(&backend, "tenant-a".to_string(), StateScope::Global)
        .await
        .expect("Failed to set tenant A again");
    assert_eq!(backend.tenant_id().await, Some("tenant-a".to_string()));
}

#[tokio::test]
async fn test_tenant_scoped_ignores_scope_parameter() {
    let backend = setup_backend().await;

    // PostgreSQL backend ignores the scope parameter (always uses session variables)
    // Test with different StateScope values to ensure they don't cause errors

    TenantScoped::set_tenant_context(&backend, "tenant-1".to_string(), StateScope::Global)
        .await
        .expect("Should work with Global scope");
    assert_eq!(backend.tenant_id().await, Some("tenant-1".to_string()));

    TenantScoped::set_tenant_context(
        &backend,
        "tenant-2".to_string(),
        StateScope::User("user1".to_string()),
    )
    .await
    .expect("Should work with User scope");
    assert_eq!(backend.tenant_id().await, Some("tenant-2".to_string()));

    TenantScoped::set_tenant_context(
        &backend,
        "tenant-3".to_string(),
        StateScope::Session("session1".to_string()),
    )
    .await
    .expect("Should work with Session scope");
    assert_eq!(backend.tenant_id().await, Some("tenant-3".to_string()));
}

#[tokio::test]
async fn test_tenant_scoped_error_handling() {
    let backend = setup_backend().await;

    // Valid tenant context should succeed
    let result =
        TenantScoped::set_tenant_context(&backend, "valid-tenant".to_string(), StateScope::Global)
            .await;
    assert!(
        result.is_ok(),
        "Setting valid tenant context should succeed"
    );

    // TenantScoped trait returns Result, allowing proper error propagation
    // (Specific error conditions tested in RLS enforcement tests)
}

#[tokio::test]
async fn test_tenant_scoped_trait_as_dyn_trait_object() {
    let backend = setup_backend().await;
    let tenant_scoped: &dyn TenantScoped = &backend;

    // Verify trait object works correctly
    tenant_scoped
        .set_tenant_context("dyn-tenant".to_string(), StateScope::Global)
        .await
        .expect("Failed to set via trait object");

    assert_eq!(
        tenant_scoped.tenant_id().await,
        Some("dyn-tenant".to_string()),
        "Should work via trait object"
    );

    assert_eq!(
        tenant_scoped.scope(),
        &StateScope::Global,
        "Should return Global scope via trait object"
    );
}

#[tokio::test]
async fn test_tenant_scoped_async_trait_send_sync() {
    // This test verifies that TenantScoped is Send + Sync, allowing it to be used
    // across thread boundaries and in async contexts

    let backend = setup_backend().await;

    // Spawn task that uses TenantScoped trait
    let handle = tokio::spawn(async move {
        TenantScoped::set_tenant_context(&backend, "async-tenant".to_string(), StateScope::Global)
            .await
            .expect("Failed to set tenant in spawned task");

        backend.tenant_id().await
    });

    let tenant_id = handle.await.expect("Task panicked");
    assert_eq!(
        tenant_id,
        Some("async-tenant".to_string()),
        "TenantScoped trait should work across async boundaries"
    );
}
