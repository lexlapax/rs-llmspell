//! Tests for PostgreSQL artifact backend (Phase 13b.10.3)
//!
//! Verifies:
//! - BYTEA storage for small artifacts (<1MB)
//! - Large Object storage for large artifacts (>=1MB)
//! - Automatic storage type selection
//! - Content deduplication
//! - Metadata storage and retrieval
//! - Session artifact listing
//! - Reference counting
//! - Cascade deletion
//! - Statistics collection
//! - Compressed content handling

#![cfg(feature = "postgres")]

use chrono::Utc;
use llmspell_storage::{PostgresBackend, PostgresConfig};
use serde_json::json;
use std::sync::atomic::{AtomicU64, Ordering};
use tokio::sync::OnceCell;
use uuid::Uuid;

const SUPERUSER_CONNECTION_STRING: &str =
    "postgresql://llmspell:llmspell_dev_pass@localhost:5432/llmspell_dev";

static MIGRATION_INIT: OnceCell<()> = OnceCell::const_new();
static TENANT_COUNTER: AtomicU64 = AtomicU64::new(1);

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

/// Generate unique tenant ID for test isolation
fn unique_tenant_id() -> String {
    let counter = TENANT_COUNTER.fetch_add(1, Ordering::SeqCst);
    format!("test_tenant_{}", counter)
}

/// Helper to create a session for artifact testing
async fn create_session(backend: &PostgresBackend, tenant_id: &str, session_id: Uuid) {
    let client = backend.get_client().await.unwrap();
    client
        .execute(
            "INSERT INTO llmspell.sessions (tenant_id, session_id, session_data, status)
             VALUES ($1, $2, $3, $4)",
            &[&tenant_id, &session_id, &json!({}), &"active"],
        )
        .await
        .unwrap();
}

/// Helper to cleanup all data for a tenant (for test isolation)
async fn cleanup_tenant_data(backend: &PostgresBackend, tenant_id: &str) {
    let client = backend.get_client().await.unwrap();
    // Delete artifacts first (due to foreign keys)
    let _ = client
        .execute(
            "DELETE FROM llmspell.artifacts WHERE tenant_id = $1",
            &[&tenant_id],
        )
        .await;
    // Delete sessions
    let _ = client
        .execute(
            "DELETE FROM llmspell.sessions WHERE tenant_id = $1",
            &[&tenant_id],
        )
        .await;
    // Delete content
    let _ = client
        .execute(
            "DELETE FROM llmspell.artifact_content WHERE tenant_id = $1",
            &[&tenant_id],
        )
        .await;
}

#[tokio::test]
async fn test_store_and_retrieve_small_content() {
    ensure_migrations_run_once().await;

    let config = PostgresConfig::new(SUPERUSER_CONNECTION_STRING);
    let backend = PostgresBackend::new(config).await.unwrap();
    let tenant_id = unique_tenant_id();

    // Small content (should use BYTEA)
    let content = b"Hello, World! This is a small artifact.";
    let content_hash = blake3::hash(content).to_hex().to_string();

    // Store content
    backend
        .store_artifact_content(&tenant_id, &content_hash, content, false)
        .await
        .unwrap();

    // Retrieve content
    let retrieved = backend
        .retrieve_artifact_content(&tenant_id, &content_hash)
        .await
        .unwrap();

    assert_eq!(retrieved, content, "Retrieved content should match");

    // Verify storage type is BYTEA
    let client = backend.get_client().await.unwrap();
    let row = client
        .query_one(
            "SELECT storage_type FROM llmspell.artifact_content
             WHERE tenant_id = $1 AND content_hash = $2",
            &[&tenant_id, &content_hash],
        )
        .await
        .unwrap();
    let storage_type: String = row.get(0);
    assert_eq!(storage_type, "bytea", "Should use BYTEA storage");
}

#[tokio::test]
async fn test_store_and_retrieve_large_content() {
    ensure_migrations_run_once().await;

    let config = PostgresConfig::new(SUPERUSER_CONNECTION_STRING);
    let backend = PostgresBackend::new(config).await.unwrap();
    let tenant_id = unique_tenant_id();

    // Large content (2MB - should use Large Object)
    let content = vec![42u8; 2 * 1024 * 1024];
    let content_hash = blake3::hash(&content).to_hex().to_string();

    // Store content
    backend
        .store_artifact_content(&tenant_id, &content_hash, &content, false)
        .await
        .unwrap();

    // Retrieve content
    let retrieved = backend
        .retrieve_artifact_content(&tenant_id, &content_hash)
        .await
        .unwrap();

    assert_eq!(
        retrieved.len(),
        content.len(),
        "Retrieved size should match"
    );
    assert_eq!(retrieved, content, "Retrieved content should match");

    // Verify storage type is Large Object
    let client = backend.get_client().await.unwrap();
    let row = client
        .query_one(
            "SELECT storage_type, large_object_oid FROM llmspell.artifact_content
             WHERE tenant_id = $1 AND content_hash = $2",
            &[&tenant_id, &content_hash],
        )
        .await
        .unwrap();
    let storage_type: String = row.get(0);
    let oid: Option<tokio_postgres::types::Oid> = row.get(1);
    assert_eq!(
        storage_type, "large_object",
        "Should use Large Object storage"
    );
    assert!(oid.is_some(), "Should have Large Object OID");
}

#[tokio::test]
async fn test_content_deduplication() {
    ensure_migrations_run_once().await;

    let config = PostgresConfig::new(SUPERUSER_CONNECTION_STRING);
    let backend = PostgresBackend::new(config).await.unwrap();
    let tenant_id = unique_tenant_id();

    let content = b"Duplicate content test";
    let content_hash = blake3::hash(content).to_hex().to_string();

    // Store content first time
    backend
        .store_artifact_content(&tenant_id, &content_hash, content, false)
        .await
        .unwrap();

    // Store same content again (should be no-op)
    backend
        .store_artifact_content(&tenant_id, &content_hash, content, false)
        .await
        .unwrap();

    // Verify only one entry exists
    let client = backend.get_client().await.unwrap();
    let row = client
        .query_one(
            "SELECT COUNT(*) FROM llmspell.artifact_content
             WHERE tenant_id = $1 AND content_hash = $2",
            &[&tenant_id, &content_hash],
        )
        .await
        .unwrap();
    let count: i64 = row.get(0);
    assert_eq!(count, 1, "Should only have one content entry");
}

#[tokio::test]
async fn test_store_and_retrieve_metadata() {
    ensure_migrations_run_once().await;

    let config = PostgresConfig::new(SUPERUSER_CONNECTION_STRING);
    let backend = PostgresBackend::new(config).await.unwrap();
    let tenant_id = unique_tenant_id();

    let session_id = Uuid::new_v4();
    let content = b"Test artifact content";
    let content_hash = blake3::hash(content).to_hex().to_string();
    let artifact_id = format!("{}:1:{}", session_id, content_hash);

    // Create session first (required by foreign key)
    create_session(&backend, &tenant_id, session_id).await;

    // Store content first
    backend
        .store_artifact_content(&tenant_id, &content_hash, content, false)
        .await
        .unwrap();

    // Store metadata
    let metadata = json!({
        "description": "Test artifact",
        "version": "1.0"
    });
    let created_at = Utc::now();

    backend
        .store_artifact_metadata(
            &tenant_id,
            &artifact_id,
            session_id,
            1,
            &content_hash,
            &metadata,
            "test.txt",
            "text",
            "text/plain",
            content.len() as i64,
            created_at,
            Some("test_user"),
            Some(vec!["test".to_string(), "example".to_string()]),
        )
        .await
        .unwrap();

    // Retrieve metadata
    let retrieved_metadata = backend
        .retrieve_artifact_metadata(&tenant_id, &artifact_id)
        .await
        .unwrap();

    assert_eq!(
        retrieved_metadata["description"], "Test artifact",
        "Metadata should match"
    );
    assert_eq!(
        retrieved_metadata["version"], "1.0",
        "Metadata should match"
    );
}

#[tokio::test]
async fn test_list_session_artifacts() {
    ensure_migrations_run_once().await;

    let config = PostgresConfig::new(SUPERUSER_CONNECTION_STRING);
    let backend = PostgresBackend::new(config).await.unwrap();
    let tenant_id = unique_tenant_id();

    let session_id = Uuid::new_v4();

    // Create session first (required by foreign key)
    create_session(&backend, &tenant_id, session_id).await;

    // Create 3 artifacts with different sequences
    for seq in 1..=3 {
        let content = format!("Artifact {}", seq);
        let content_hash = blake3::hash(content.as_bytes()).to_hex().to_string();
        let artifact_id = format!("{}:{}:{}", session_id, seq, content_hash);

        backend
            .store_artifact_content(&tenant_id, &content_hash, content.as_bytes(), false)
            .await
            .unwrap();

        backend
            .store_artifact_metadata(
                &tenant_id,
                &artifact_id,
                session_id,
                seq,
                &content_hash,
                &json!({}),
                &format!("artifact{}.txt", seq),
                "text",
                "text/plain",
                content.len() as i64,
                Utc::now(),
                None,
                None,
            )
            .await
            .unwrap();
    }

    // List artifacts
    let artifacts = backend
        .list_session_artifacts(&tenant_id, session_id)
        .await
        .unwrap();

    assert_eq!(artifacts.len(), 3, "Should have 3 artifacts");

    // Verify order (should be by sequence)
    for (i, artifact_id) in artifacts.iter().enumerate() {
        let expected_seq = i + 1;
        assert!(
            artifact_id.contains(&format!(":{}:", expected_seq)),
            "Artifacts should be ordered by sequence"
        );
    }
}

#[tokio::test]
async fn test_delete_artifact_with_unique_content() {
    ensure_migrations_run_once().await;

    let config = PostgresConfig::new(SUPERUSER_CONNECTION_STRING);
    let backend = PostgresBackend::new(config).await.unwrap();
    let tenant_id = unique_tenant_id();

    let session_id = Uuid::new_v4();
    let content = b"Unique content for deletion";
    let content_hash = blake3::hash(content).to_hex().to_string();
    let artifact_id = format!("{}:1:{}", session_id, content_hash);

    // Create session first (required by foreign key)
    create_session(&backend, &tenant_id, session_id).await;

    // Store content and metadata
    backend
        .store_artifact_content(&tenant_id, &content_hash, content, false)
        .await
        .unwrap();

    backend
        .store_artifact_metadata(
            &tenant_id,
            &artifact_id,
            session_id,
            1,
            &content_hash,
            &json!({}),
            "delete_test.txt",
            "text",
            "text/plain",
            content.len() as i64,
            Utc::now(),
            None,
            None,
        )
        .await
        .unwrap();

    // Verify content exists
    let client = backend.get_client().await.unwrap();
    let row = client
        .query_opt(
            "SELECT 1 FROM llmspell.artifact_content
             WHERE tenant_id = $1 AND content_hash = $2",
            &[&tenant_id, &content_hash],
        )
        .await
        .unwrap();
    assert!(row.is_some(), "Content should exist before deletion");

    // Delete artifact
    backend
        .delete_artifact(&tenant_id, &artifact_id)
        .await
        .unwrap();

    // Verify metadata deleted
    let row = client
        .query_opt(
            "SELECT 1 FROM llmspell.artifacts
             WHERE tenant_id = $1 AND artifact_id = $2",
            &[&tenant_id, &artifact_id],
        )
        .await
        .unwrap();
    assert!(row.is_none(), "Metadata should be deleted");

    // Verify content deleted (since it was unique)
    let row = client
        .query_opt(
            "SELECT 1 FROM llmspell.artifact_content
             WHERE tenant_id = $1 AND content_hash = $2",
            &[&tenant_id, &content_hash],
        )
        .await
        .unwrap();
    assert!(row.is_none(), "Content should be deleted when unique");
}

#[tokio::test]
async fn test_reference_counting() {
    ensure_migrations_run_once().await;

    let config = PostgresConfig::new(SUPERUSER_CONNECTION_STRING);
    let backend = PostgresBackend::new(config).await.unwrap();
    let tenant_id = unique_tenant_id();

    let session_id = Uuid::new_v4();
    let content = b"Shared content";
    let content_hash = blake3::hash(content).to_hex().to_string();

    // Create session first (required by foreign key)
    create_session(&backend, &tenant_id, session_id).await;

    // Store content once
    backend
        .store_artifact_content(&tenant_id, &content_hash, content, false)
        .await
        .unwrap();

    // Create 2 artifacts sharing the same content
    let artifact_id_1 = format!("{}:1:{}", session_id, content_hash);
    let artifact_id_2 = format!("{}:2:{}", session_id, content_hash);

    backend
        .store_artifact_metadata(
            &tenant_id,
            &artifact_id_1,
            session_id,
            1,
            &content_hash,
            &json!({}),
            "shared1.txt",
            "text",
            "text/plain",
            content.len() as i64,
            Utc::now(),
            None,
            None,
        )
        .await
        .unwrap();

    backend
        .store_artifact_metadata(
            &tenant_id,
            &artifact_id_2,
            session_id,
            2,
            &content_hash,
            &json!({}),
            "shared2.txt",
            "text",
            "text/plain",
            content.len() as i64,
            Utc::now(),
            None,
            None,
        )
        .await
        .unwrap();

    // Verify reference count is 3 (1 initial + 2 increments from artifacts)
    let client = backend.get_client().await.unwrap();
    let row = client
        .query_one(
            "SELECT reference_count FROM llmspell.artifact_content
             WHERE tenant_id = $1 AND content_hash = $2",
            &[&tenant_id, &content_hash],
        )
        .await
        .unwrap();
    let ref_count: i32 = row.get(0);
    assert_eq!(
        ref_count, 3,
        "Reference count should be 3 (1 initial + 2 artifact inserts)"
    );

    // Delete first artifact
    backend
        .delete_artifact(&tenant_id, &artifact_id_1)
        .await
        .unwrap();

    // Verify reference count is now 2 (3 - 1 decrement)
    let row = client
        .query_one(
            "SELECT reference_count FROM llmspell.artifact_content
             WHERE tenant_id = $1 AND content_hash = $2",
            &[&tenant_id, &content_hash],
        )
        .await
        .unwrap();
    let ref_count: i32 = row.get(0);
    assert_eq!(
        ref_count, 2,
        "Reference count should be 2 after deleting one artifact"
    );

    // Verify content still exists
    let row = client
        .query_opt(
            "SELECT 1 FROM llmspell.artifact_content
             WHERE tenant_id = $1 AND content_hash = $2",
            &[&tenant_id, &content_hash],
        )
        .await
        .unwrap();
    assert!(row.is_some(), "Content should still exist");

    // Delete second artifact
    backend
        .delete_artifact(&tenant_id, &artifact_id_2)
        .await
        .unwrap();

    // Verify content is now deleted
    let row = client
        .query_opt(
            "SELECT 1 FROM llmspell.artifact_content
             WHERE tenant_id = $1 AND content_hash = $2",
            &[&tenant_id, &content_hash],
        )
        .await
        .unwrap();
    assert!(
        row.is_none(),
        "Content should be deleted when reference count reaches 0"
    );
}

#[tokio::test]
async fn test_get_artifact_stats() {
    ensure_migrations_run_once().await;

    let config = PostgresConfig::new(SUPERUSER_CONNECTION_STRING);
    let backend = PostgresBackend::new(config).await.unwrap();
    let tenant_id = unique_tenant_id();

    // Cleanup any existing data for this tenant (test isolation)
    cleanup_tenant_data(&backend, &tenant_id).await;

    let session_id = Uuid::new_v4();

    // Create session first (required by foreign key)
    create_session(&backend, &tenant_id, session_id).await;

    // Create 3 artifacts: 2 small (BYTEA), 1 large (Large Object)
    // First 2 share same content (deduplication)
    let small_content = b"Small shared content";
    let small_hash = blake3::hash(small_content).to_hex().to_string();

    backend
        .store_artifact_content(&tenant_id, &small_hash, small_content, false)
        .await
        .unwrap();

    // First artifact with small content
    backend
        .store_artifact_metadata(
            &tenant_id,
            &format!("{}:1:{}", session_id, small_hash),
            session_id,
            1,
            &small_hash,
            &json!({}),
            "small1.txt",
            "text",
            "text/plain",
            small_content.len() as i64,
            Utc::now(),
            None,
            None,
        )
        .await
        .unwrap();

    // Second artifact with same small content
    backend
        .store_artifact_metadata(
            &tenant_id,
            &format!("{}:2:{}", session_id, small_hash),
            session_id,
            2,
            &small_hash,
            &json!({}),
            "small2.txt",
            "text",
            "text/plain",
            small_content.len() as i64,
            Utc::now(),
            None,
            None,
        )
        .await
        .unwrap();

    // Third artifact with large content
    let large_content = vec![42u8; 2 * 1024 * 1024];
    let large_hash = blake3::hash(&large_content).to_hex().to_string();

    backend
        .store_artifact_content(&tenant_id, &large_hash, &large_content, false)
        .await
        .unwrap();

    backend
        .store_artifact_metadata(
            &tenant_id,
            &format!("{}:3:{}", session_id, large_hash),
            session_id,
            3,
            &large_hash,
            &json!({}),
            "large.bin",
            "binary",
            "application/octet-stream",
            large_content.len() as i64,
            Utc::now(),
            None,
            None,
        )
        .await
        .unwrap();

    // Get statistics
    let stats = backend.get_artifact_stats(&tenant_id).await.unwrap();

    assert_eq!(stats.total_artifacts, 3, "Should have 3 total artifacts");
    assert_eq!(
        stats.unique_contents, 2,
        "Should have 2 unique content hashes"
    );
    assert_eq!(stats.bytea_count, 1, "Should have 1 BYTEA storage entry");
    assert_eq!(
        stats.large_object_count, 1,
        "Should have 1 Large Object storage entry"
    );

    let expected_total_size = (small_content.len() * 2) + large_content.len();
    assert_eq!(
        stats.total_size_bytes, expected_total_size,
        "Total size should match"
    );
}

#[tokio::test]
async fn test_automatic_storage_type_selection() {
    ensure_migrations_run_once().await;

    let config = PostgresConfig::new(SUPERUSER_CONNECTION_STRING);
    let backend = PostgresBackend::new(config).await.unwrap();
    let tenant_id = unique_tenant_id();

    // Test at boundary: 1MB - 1 byte (should be BYTEA)
    let just_under_threshold = vec![0u8; (1024 * 1024) - 1];
    let hash_under = blake3::hash(&just_under_threshold).to_hex().to_string();

    backend
        .store_artifact_content(&tenant_id, &hash_under, &just_under_threshold, false)
        .await
        .unwrap();

    let client = backend.get_client().await.unwrap();
    let row = client
        .query_one(
            "SELECT storage_type FROM llmspell.artifact_content
             WHERE tenant_id = $1 AND content_hash = $2",
            &[&tenant_id, &hash_under],
        )
        .await
        .unwrap();
    let storage_type: String = row.get(0);
    assert_eq!(storage_type, "bytea", "Just under 1MB should use BYTEA");

    // Test at boundary: 1MB exactly (should be Large Object)
    let exactly_threshold = vec![0u8; 1024 * 1024];
    let hash_exact = blake3::hash(&exactly_threshold).to_hex().to_string();

    backend
        .store_artifact_content(&tenant_id, &hash_exact, &exactly_threshold, false)
        .await
        .unwrap();

    let row = client
        .query_one(
            "SELECT storage_type FROM llmspell.artifact_content
             WHERE tenant_id = $1 AND content_hash = $2",
            &[&tenant_id, &hash_exact],
        )
        .await
        .unwrap();
    let storage_type: String = row.get(0);
    assert_eq!(
        storage_type, "large_object",
        "Exactly 1MB should use Large Object"
    );
}

#[tokio::test]
async fn test_compressed_content_storage() {
    ensure_migrations_run_once().await;

    let config = PostgresConfig::new(SUPERUSER_CONNECTION_STRING);
    let backend = PostgresBackend::new(config).await.unwrap();
    let tenant_id = unique_tenant_id();

    let content = b"This content is compressed";
    let content_hash = blake3::hash(content).to_hex().to_string();

    // Store as compressed
    backend
        .store_artifact_content(&tenant_id, &content_hash, content, true)
        .await
        .unwrap();

    // Verify is_compressed flag
    let client = backend.get_client().await.unwrap();
    let row = client
        .query_one(
            "SELECT is_compressed FROM llmspell.artifact_content
             WHERE tenant_id = $1 AND content_hash = $2",
            &[&tenant_id, &content_hash],
        )
        .await
        .unwrap();
    let is_compressed: bool = row.get(0);
    assert!(is_compressed, "Should be marked as compressed");

    // Retrieve content (raw, backend doesn't decompress)
    let retrieved = backend
        .retrieve_artifact_content(&tenant_id, &content_hash)
        .await
        .unwrap();

    assert_eq!(
        retrieved, content,
        "Backend returns raw content (decompression handled at higher level)"
    );
}

#[tokio::test]
async fn test_delete_nonexistent_artifact() {
    ensure_migrations_run_once().await;

    let config = PostgresConfig::new(SUPERUSER_CONNECTION_STRING);
    let backend = PostgresBackend::new(config).await.unwrap();
    let tenant_id = unique_tenant_id();

    // Try to delete non-existent artifact (should be no-op)
    let result = backend
        .delete_artifact(&tenant_id, "nonexistent_artifact")
        .await;

    assert!(
        result.is_ok(),
        "Deleting non-existent artifact should succeed (no-op)"
    );
}
