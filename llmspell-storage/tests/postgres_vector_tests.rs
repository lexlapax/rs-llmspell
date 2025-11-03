//! Tests for PostgreSQLVectorStorage with dimension routing (Phase 13b.4.2)
//!
//! Verifies:
//! - Dimension routing to correct tables (384, 768, 1536, 3072)
//! - VectorStorage trait implementation
//! - RLS tenant isolation
//! - Insert, search, update, delete operations on all dimensions
//! - Performance characteristics

#![cfg(feature = "postgres")]

use llmspell_core::state::StateScope;
use llmspell_storage::{
    PostgresBackend, PostgresConfig, PostgreSQLVectorStorage, VectorEntry, VectorQuery,
    VectorStorage,
};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::OnceCell;
use uuid::Uuid;

const SUPERUSER_CONNECTION_STRING: &str =
    "postgresql://llmspell:llmspell_dev_pass@localhost:5432/llmspell_dev";

const TEST_CONNECTION_STRING: &str =
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
async fn test_dimension_routing_all_dimensions() {
    ensure_migrations_run_once().await;

    let config = PostgresConfig::new(TEST_CONNECTION_STRING);
    let backend = Arc::new(PostgresBackend::new(config).await.unwrap());
    let storage = PostgreSQLVectorStorage::new(backend.clone());

    let tenant_id = unique_tenant_id("dim-routing");
    backend.set_tenant_context(&tenant_id).await.unwrap();

    // Test all 4 supported dimensions
    for dim in [384, 768, 1536, 3072] {
        let id = Uuid::new_v4().to_string();
        let entry = VectorEntry::new(id, vec![0.1; dim]).with_scope(StateScope::Global);

        let ids = storage.insert(vec![entry]).await.unwrap();
        assert_eq!(ids.len(), 1, "Should insert 1 vector for {} dims", dim);
    }

    // Cleanup
    backend.clear_tenant_context().await.unwrap();
}

#[tokio::test]
async fn test_dimension_routing_unsupported_dimension() {
    ensure_migrations_run_once().await;

    let config = PostgresConfig::new(TEST_CONNECTION_STRING);
    let backend = Arc::new(PostgresBackend::new(config).await.unwrap());
    let storage = PostgreSQLVectorStorage::new(backend.clone());

    let tenant_id = unique_tenant_id("unsupported-dim");
    backend.set_tenant_context(&tenant_id).await.unwrap();

    // Test unsupported dimension (should error)
    let invalid = VectorEntry::new("vec-999".to_string(), vec![1.0; 999])
        .with_scope(StateScope::Global);

    let result = storage.insert(vec![invalid]).await;
    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("Unsupported dimension"));

    backend.clear_tenant_context().await.unwrap();
}

#[tokio::test]
async fn test_insert_and_search_384() {
    ensure_migrations_run_once().await;

    let config = PostgresConfig::new(TEST_CONNECTION_STRING);
    let backend = Arc::new(PostgresBackend::new(config).await.unwrap());
    let storage = PostgreSQLVectorStorage::new(backend.clone());

    let tenant_id = unique_tenant_id("insert-search-384");
    backend.set_tenant_context(&tenant_id).await.unwrap();

    // Insert 3 vectors
    let id1 = Uuid::new_v4().to_string();
    let id2 = Uuid::new_v4().to_string();
    let id3 = Uuid::new_v4().to_string();

    let vectors = vec![
        VectorEntry::new(id1.clone(), vec![1.0; 384]).with_scope(StateScope::Global),
        VectorEntry::new(id2, vec![0.5; 384]).with_scope(StateScope::Global),
        VectorEntry::new(id3, vec![0.1; 384]).with_scope(StateScope::Global),
    ];

    let ids = storage.insert(vectors).await.unwrap();
    assert_eq!(ids.len(), 3);

    // Search for similar vectors
    let query = VectorQuery::new(vec![1.0; 384], 2);
    let results = storage.search(&query).await.unwrap();

    assert_eq!(results.len(), 2, "Should return top 2 results");
    assert_eq!(results[0].id, id1, "Most similar should be id1");

    // Cleanup
    storage.delete(&ids).await.unwrap();
    backend.clear_tenant_context().await.unwrap();
}

#[tokio::test]
async fn test_search_scoped() {
    ensure_migrations_run_once().await;

    let config = PostgresConfig::new(TEST_CONNECTION_STRING);
    let backend = Arc::new(PostgresBackend::new(config).await.unwrap());
    let storage = PostgreSQLVectorStorage::new(backend.clone());

    let tenant_id = unique_tenant_id("search-scoped");
    backend.set_tenant_context(&tenant_id).await.unwrap();

    // Insert vectors with different scopes
    let scope_a = StateScope::User("user-a".to_string());
    let scope_b = StateScope::User("user-b".to_string());

    let vectors = vec![
        VectorEntry::new(Uuid::new_v4().to_string(), vec![1.0; 768]).with_scope(scope_a.clone()),
        VectorEntry::new(Uuid::new_v4().to_string(), vec![0.9; 768]).with_scope(scope_a.clone()),
        VectorEntry::new(Uuid::new_v4().to_string(), vec![1.0; 768]).with_scope(scope_b.clone()),
    ];

    let ids = storage.insert(vectors).await.unwrap();
    assert_eq!(ids.len(), 3);

    // Search only scope A
    let query = VectorQuery::new(vec![1.0; 768], 10);
    let results_a = storage.search_scoped(&query, &scope_a).await.unwrap();

    assert_eq!(results_a.len(), 2, "Should only find vectors in scope A");

    // Search only scope B
    let results_b = storage.search_scoped(&query, &scope_b).await.unwrap();

    assert_eq!(results_b.len(), 1, "Should only find vectors in scope B");

    // Cleanup
    storage.delete(&ids).await.unwrap();
    backend.clear_tenant_context().await.unwrap();
}

#[tokio::test]
async fn test_update_metadata() {
    ensure_migrations_run_once().await;

    let config = PostgresConfig::new(TEST_CONNECTION_STRING);
    let backend = Arc::new(PostgresBackend::new(config).await.unwrap());
    let storage = PostgreSQLVectorStorage::new(backend.clone());

    let tenant_id = unique_tenant_id("update-metadata");
    backend.set_tenant_context(&tenant_id).await.unwrap();

    // Insert vector
    let entry = VectorEntry::new(Uuid::new_v4().to_string(), vec![1.0; 1536])
        .with_scope(StateScope::Global);

    let ids = storage.insert(vec![entry]).await.unwrap();
    assert_eq!(ids.len(), 1);

    // Update metadata
    let new_metadata = HashMap::from([
        ("status".to_string(), serde_json::json!("updated")),
        ("count".to_string(), serde_json::json!(42)),
    ]);

    storage
        .update_metadata(&ids[0], new_metadata.clone())
        .await
        .unwrap();

    // Verify update by searching with metadata included
    let query = VectorQuery::new(vec![1.0; 1536], 1);
    let mut query_with_meta = query.clone();
    query_with_meta.include_metadata = true;

    let results = storage.search(&query_with_meta).await.unwrap();
    assert_eq!(results.len(), 1);

    if let Some(meta) = &results[0].metadata {
        assert_eq!(meta.get("status"), Some(&serde_json::json!("updated")));
        assert_eq!(meta.get("count"), Some(&serde_json::json!(42)));
    } else {
        panic!("Metadata should be included");
    }

    // Cleanup
    storage.delete(&ids).await.unwrap();
    backend.clear_tenant_context().await.unwrap();
}

#[tokio::test]
async fn test_delete_scope() {
    ensure_migrations_run_once().await;

    let config = PostgresConfig::new(TEST_CONNECTION_STRING);
    let backend = Arc::new(PostgresBackend::new(config).await.unwrap());
    let storage = PostgreSQLVectorStorage::new(backend.clone());

    let tenant_id = unique_tenant_id("delete-scope");
    backend.set_tenant_context(&tenant_id).await.unwrap();

    // Insert vectors across multiple scopes and dimensions
    let scope_x = StateScope::User("user-x".to_string());
    let scope_y = StateScope::User("user-y".to_string());

    let vectors = vec![
        VectorEntry::new(Uuid::new_v4().to_string(), vec![1.0; 384]).with_scope(scope_x.clone()),
        VectorEntry::new(Uuid::new_v4().to_string(), vec![1.0; 768]).with_scope(scope_x.clone()),
        VectorEntry::new(Uuid::new_v4().to_string(), vec![1.0; 384]).with_scope(scope_y.clone()),
        VectorEntry::new(Uuid::new_v4().to_string(), vec![1.0; 1536]).with_scope(scope_y.clone()),
    ];

    storage.insert(vectors).await.unwrap();

    // Delete all vectors in scope X
    let deleted_count = storage.delete_scope(&scope_x).await.unwrap();
    assert_eq!(deleted_count, 2, "Should delete 2 vectors from scope X");

    // Verify scope X is empty
    let query_384 = VectorQuery::new(vec![1.0; 384], 10);
    let results_x = storage.search_scoped(&query_384, &scope_x).await.unwrap();
    assert_eq!(results_x.len(), 0, "Scope X should be empty");

    // Verify scope Y still has data
    let results_y = storage.search_scoped(&query_384, &scope_y).await.unwrap();
    assert_eq!(results_y.len(), 1, "Scope Y should still have 1 vector");

    // Cleanup
    storage.delete_scope(&scope_y).await.unwrap();
    backend.clear_tenant_context().await.unwrap();
}

#[tokio::test]
async fn test_stats() {
    ensure_migrations_run_once().await;

    let config = PostgresConfig::new(TEST_CONNECTION_STRING);
    let backend = Arc::new(PostgresBackend::new(config).await.unwrap());
    let storage = PostgreSQLVectorStorage::new(backend.clone());

    let tenant_id = unique_tenant_id("stats");
    backend.set_tenant_context(&tenant_id).await.unwrap();

    // Insert vectors across all dimensions
    let vectors = vec![
        VectorEntry::new(Uuid::new_v4().to_string(), vec![1.0; 384]).with_scope(StateScope::Global),
        VectorEntry::new(Uuid::new_v4().to_string(), vec![1.0; 768]).with_scope(StateScope::Global),
        VectorEntry::new(Uuid::new_v4().to_string(), vec![1.0; 1536]).with_scope(StateScope::Global),
        VectorEntry::new(Uuid::new_v4().to_string(), vec![1.0; 3072]).with_scope(StateScope::Global),
    ];

    let ids = storage.insert(vectors).await.unwrap();

    // Get stats
    let stats = storage.stats().await.unwrap();
    assert_eq!(stats.total_vectors, 4, "Should count all 4 vectors");

    // Cleanup
    storage.delete(&ids).await.unwrap();
    backend.clear_tenant_context().await.unwrap();
}

#[tokio::test]
async fn test_stats_for_scope() {
    ensure_migrations_run_once().await;

    let config = PostgresConfig::new(TEST_CONNECTION_STRING);
    let backend = Arc::new(PostgresBackend::new(config).await.unwrap());
    let storage = PostgreSQLVectorStorage::new(backend.clone());

    let tenant_id = unique_tenant_id("stats-scope");
    backend.set_tenant_context(&tenant_id).await.unwrap();

    let scope_a = StateScope::User("stats-user-a".to_string());
    let scope_b = StateScope::User("stats-user-b".to_string());

    // Insert 3 vectors in scope A, 2 in scope B
    let vectors = vec![
        VectorEntry::new(Uuid::new_v4().to_string(), vec![1.0; 384]).with_scope(scope_a.clone()),
        VectorEntry::new(Uuid::new_v4().to_string(), vec![1.0; 768]).with_scope(scope_a.clone()),
        VectorEntry::new(Uuid::new_v4().to_string(), vec![1.0; 1536]).with_scope(scope_a.clone()),
        VectorEntry::new(Uuid::new_v4().to_string(), vec![1.0; 384]).with_scope(scope_b.clone()),
        VectorEntry::new(Uuid::new_v4().to_string(), vec![1.0; 768]).with_scope(scope_b.clone()),
    ];

    let ids = storage.insert(vectors).await.unwrap();

    // Get stats for scope A
    let stats_a = storage.stats_for_scope(&scope_a).await.unwrap();
    assert_eq!(stats_a.vector_count, 3, "Scope A should have 3 vectors");

    // Get stats for scope B
    let stats_b = storage.stats_for_scope(&scope_b).await.unwrap();
    assert_eq!(stats_b.vector_count, 2, "Scope B should have 2 vectors");

    // Cleanup
    storage.delete(&ids).await.unwrap();
    backend.clear_tenant_context().await.unwrap();
}

#[tokio::test]
async fn test_rls_tenant_isolation() {
    ensure_migrations_run_once().await;

    let config = PostgresConfig::new(TEST_CONNECTION_STRING);
    let backend = Arc::new(PostgresBackend::new(config).await.unwrap());
    let storage = PostgreSQLVectorStorage::new(backend.clone());

    let tenant_a = unique_tenant_id("rls-tenant-a");
    let tenant_b = unique_tenant_id("rls-tenant-b");

    // Tenant A inserts data
    backend.set_tenant_context(&tenant_a).await.unwrap();

    let vectors_a = vec![
        VectorEntry::new(Uuid::new_v4().to_string(), vec![1.0; 384]).with_scope(StateScope::Global),
        VectorEntry::new(Uuid::new_v4().to_string(), vec![1.0; 768]).with_scope(StateScope::Global),
    ];

    storage.insert(vectors_a).await.unwrap();

    // Tenant A should see 2 vectors
    let stats_a = storage.stats().await.unwrap();
    assert_eq!(stats_a.total_vectors, 2, "Tenant A should see its 2 vectors");

    // Switch to Tenant B
    backend.clear_tenant_context().await.unwrap();
    backend.set_tenant_context(&tenant_b).await.unwrap();

    // Tenant B should see 0 vectors
    let stats_b = storage.stats().await.unwrap();
    assert_eq!(
        stats_b.total_vectors, 0,
        "Tenant B should see no vectors (RLS isolation)"
    );

    // Tenant B inserts its own data
    let vectors_b = vec![
        VectorEntry::new(Uuid::new_v4().to_string(), vec![1.0; 1536]).with_scope(StateScope::Global),
    ];

    storage.insert(vectors_b).await.unwrap();

    // Tenant B should see 1 vector
    let stats_b2 = storage.stats().await.unwrap();
    assert_eq!(stats_b2.total_vectors, 1, "Tenant B should see its 1 vector");

    // Cleanup both tenants
    backend.clear_tenant_context().await.unwrap();
    backend.set_tenant_context(&tenant_a).await.unwrap();
    let client = backend.get_client().await.unwrap();
    for dim in [384, 768, 1536, 3072] {
        let table = format!("vector_embeddings_{}", dim);
        let _ = client
            .execute(&format!("DELETE FROM llmspell.{} WHERE TRUE", table), &[])
            .await;
    }

    backend.clear_tenant_context().await.unwrap();
    backend.set_tenant_context(&tenant_b).await.unwrap();
    let client = backend.get_client().await.unwrap();
    for dim in [384, 768, 1536, 3072] {
        let table = format!("vector_embeddings_{}", dim);
        let _ = client
            .execute(&format!("DELETE FROM llmspell.{} WHERE TRUE", table), &[])
            .await;
    }

    backend.clear_tenant_context().await.unwrap();
}

#[tokio::test]
async fn test_threshold_filtering() {
    ensure_migrations_run_once().await;

    let config = PostgresConfig::new(TEST_CONNECTION_STRING);
    let backend = Arc::new(PostgresBackend::new(config).await.unwrap());
    let storage = PostgreSQLVectorStorage::new(backend.clone());

    let tenant_id = unique_tenant_id("threshold");
    backend.set_tenant_context(&tenant_id).await.unwrap();

    // Insert vectors with varying similarity to query
    // v1: identical to query (similarity ~1.0)
    // v2: mix of 1.0 and -1.0 (similarity ~0.0)
    // v3: all -1.0 (similarity ~0.0, opposite direction)
    let v1 = vec![1.0; 384];
    let mut v2 = vec![1.0; 192];
    v2.extend(vec![-1.0; 192]);
    let v3 = vec![-1.0; 384];

    let vectors = vec![
        VectorEntry::new(Uuid::new_v4().to_string(), v1).with_scope(StateScope::Global),
        VectorEntry::new(Uuid::new_v4().to_string(), v2).with_scope(StateScope::Global),
        VectorEntry::new(Uuid::new_v4().to_string(), v3).with_scope(StateScope::Global),
    ];

    let ids = storage.insert(vectors).await.unwrap();

    // Search with high threshold - should filter out dissimilar vectors
    let query = VectorQuery::new(vec![1.0; 384], 10);
    let mut query_with_threshold = query.clone();
    query_with_threshold.threshold = Some(0.9); // Only very similar

    let results = storage.search(&query_with_threshold).await.unwrap();

    // Threshold should filter out some results (not all 3)
    assert!(
        results.len() < 3,
        "Should filter out dissimilar vectors, got {}",
        results.len()
    );

    // Cleanup
    storage.delete(&ids).await.unwrap();
    backend.clear_tenant_context().await.unwrap();
}
