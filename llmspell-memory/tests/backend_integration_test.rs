//! Integration tests for episodic backend abstraction
//!
//! Validates that `InMemory` and `Sqlite` backends implement the `EpisodicMemory` trait
//! correctly through the `EpisodicBackend` enum.
//!
//! ## Sqlite Backend Architecture
//!
//! The Sqlite backend uses vectorlite-rs for vector search:
//! - **`vectorlite` HNSW**: O(log n) vector similarity search via `SQLite` extension
//! - **`SQLite`**: ACID-compliant persistent storage with full SQL metadata queries
//!
//! This provides complete `EpisodicMemory` trait implementation with optimal performance
//! for both vector search (primary use case) and metadata operations.

use async_trait::async_trait;
use llmspell_core::traits::embedding::EmbeddingProvider;
use llmspell_core::LLMSpellError;
use llmspell_memory::prelude::*;
use llmspell_memory::{embeddings::EmbeddingService, EpisodicBackend, MemoryConfig};
use std::sync::Arc;

/// Test embedding provider (generates deterministic embeddings)
struct TestEmbeddingProvider;

#[async_trait]
impl EmbeddingProvider for TestEmbeddingProvider {
    fn name(&self) -> &'static str {
        "test-backend-provider"
    }

    #[allow(clippy::cast_precision_loss)]
    async fn embed(&self, texts: &[String]) -> std::result::Result<Vec<Vec<f32>>, LLMSpellError> {
        // Generate simple deterministic embeddings based on text length
        Ok(texts
            .iter()
            .map(|text| {
                let base = text.len() as f32 / 100.0;
                (0..384).map(|i| base + (i as f32 / 1000.0)).collect()
            })
            .collect())
    }

    fn embedding_dimensions(&self) -> usize {
        384
    }

    fn embedding_model(&self) -> Option<&str> {
        Some("test-model")
    }
}

/// Test helper that runs the same test logic against both backends
async fn run_on_both_backends<F, Fut>(test_fn: F)
where
    F: Fn(Arc<dyn EpisodicMemory>) -> Fut,
    Fut: std::future::Future<Output = ()>,
{
    // Test with InMemory backend
    let config = MemoryConfig::for_testing();
    let backend = EpisodicBackend::from_config(&config).expect("InMemory backend creation failed");
    let backend_arc: Arc<dyn EpisodicMemory> = Arc::new(backend.clone());
    test_fn(backend_arc).await;

    // Test with Sqlite backend
    let provider: Arc<dyn EmbeddingProvider> = Arc::new(TestEmbeddingProvider);
    let embedding_service = Arc::new(EmbeddingService::new(provider));

    // Create temporary SQLite database for testing
    let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
    let db_path = temp_dir.path().join("test_episodic.db");
    let sqlite_backend = Arc::new(
        llmspell_storage::backends::sqlite::SqliteBackend::new(
            llmspell_storage::backends::sqlite::SqliteConfig::new(&db_path),
        )
        .await
        .expect("Failed to create SqliteBackend"),
    );

    let config =
        MemoryConfig::for_production(embedding_service).with_sqlite_backend(sqlite_backend);
    let backend = EpisodicBackend::from_config(&config).expect("Sqlite backend creation failed");
    let backend_arc: Arc<dyn EpisodicMemory> = Arc::new(backend);
    test_fn(backend_arc).await;
}

// ============================================================================
// Backend Abstraction Tests
// ============================================================================

#[tokio::test]
async fn test_backend_add() {
    run_on_both_backends(|backend| async move {
        let entry = EpisodicEntry::new("session-1".into(), "user".into(), "Test message".into());

        let id = backend.add(entry).await.expect("add failed");
        assert!(!id.is_empty(), "Entry ID should not be empty");
    })
    .await;
}

#[tokio::test]
async fn test_backend_get() {
    run_on_both_backends(|backend| async move {
        let entry = EpisodicEntry::new("session-1".into(), "user".into(), "Test message".into());

        let id = backend.add(entry).await.expect("add failed");
        let retrieved = backend.get(&id).await.expect("get failed");

        assert_eq!(retrieved.session_id, "session-1");
        assert_eq!(retrieved.role, "user");
        assert_eq!(retrieved.content, "Test message");
    })
    .await;
}

#[tokio::test]
async fn test_backend_get_session() {
    run_on_both_backends(|backend| async move {
        // Add entries to two different sessions
        for i in 0..5 {
            let entry =
                EpisodicEntry::new("session-1".into(), "user".into(), format!("Message {i}"));
            backend.add(entry).await.expect("add failed");
        }

        for i in 0..3 {
            let entry = EpisodicEntry::new(
                "session-2".into(),
                "user".into(),
                format!("Other message {i}"),
            );
            backend.add(entry).await.expect("add failed");
        }

        // Get session-1 entries
        let entries = backend
            .get_session("session-1")
            .await
            .expect("get_session failed");

        assert_eq!(entries.len(), 5);
        for entry in &entries {
            assert_eq!(entry.session_id, "session-1");
        }

        // Get session-2 entries
        let entries = backend
            .get_session("session-2")
            .await
            .expect("get_session failed");

        assert_eq!(entries.len(), 3);
        for entry in &entries {
            assert_eq!(entry.session_id, "session-2");
        }
    })
    .await;
}

#[tokio::test]
async fn test_backend_search() {
    run_on_both_backends(|backend| async move {
        // Add entries with different content
        let contents = vec![
            "Rust programming language",
            "Python web development",
            "Rust async programming",
            "JavaScript frontend",
            "Rust systems programming",
        ];

        for content in contents {
            let entry = EpisodicEntry::new("session-1".into(), "user".into(), content.into());
            backend.add(entry).await.expect("add failed");
        }

        // Search for "Rust programming"
        let results = backend
            .search("Rust programming", 10)
            .await
            .expect("search failed");

        // Should find all 3 Rust entries (or at least some results)
        assert!(!results.is_empty(), "Expected at least some search results");
        assert!(results.len() <= 10, "Results should respect top_k limit");
    })
    .await;
}

#[tokio::test]
async fn test_backend_list_unprocessed() {
    run_on_both_backends(|backend| async move {
        // Add entries
        let mut ids = Vec::new();
        for i in 0..5 {
            let entry =
                EpisodicEntry::new("session-1".into(), "user".into(), format!("Message {i}"));
            let id = backend.add(entry).await.expect("add failed");
            ids.push(id);
        }

        // All should be unprocessed initially
        let unprocessed = backend
            .list_unprocessed("session-1")
            .await
            .expect("list_unprocessed failed");
        assert_eq!(unprocessed.len(), 5);

        // Mark first 3 as processed
        backend
            .mark_processed(&ids[0..3])
            .await
            .expect("mark_processed failed");

        // Should have 2 unprocessed remaining
        let unprocessed = backend
            .list_unprocessed("session-1")
            .await
            .expect("list_unprocessed failed");
        assert_eq!(unprocessed.len(), 2);
    })
    .await;
}

#[tokio::test]
async fn test_backend_delete_before() {
    use chrono::{Duration, Utc};

    run_on_both_backends(|backend| async move {
        // Add entries
        for i in 0..5 {
            let entry =
                EpisodicEntry::new("session-1".into(), "user".into(), format!("Message {i}"));
            backend.add(entry).await.expect("add failed");
        }

        // Wait a bit
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        // Add one more recent entry
        let entry = EpisodicEntry::new("session-1".into(), "user".into(), "Recent message".into());
        backend.add(entry).await.expect("add failed");

        // Delete entries before "now + 1 hour" (should delete all)
        let future_time = Utc::now() + Duration::hours(1);
        let deleted = backend
            .delete_before(future_time)
            .await
            .expect("delete_before failed");

        assert_eq!(deleted, 6, "Should have deleted all 6 entries");

        // Verify session is empty
        let entries = backend
            .get_session("session-1")
            .await
            .expect("get_session failed");
        assert_eq!(entries.len(), 0);
    })
    .await;
}

#[tokio::test]
async fn test_backend_name() {
    // Test InMemory backend name
    let config = MemoryConfig::for_testing();
    let backend = EpisodicBackend::from_config(&config).expect("backend creation failed");
    assert_eq!(backend.backend_name(), "InMemory");

    // Test Sqlite backend name
    let provider: Arc<dyn EmbeddingProvider> = Arc::new(TestEmbeddingProvider);
    let embedding_service = Arc::new(EmbeddingService::new(provider));

    let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
    let db_path = temp_dir.path().join("test_backend_name.db");
    let sqlite_backend = Arc::new(
        llmspell_storage::backends::sqlite::SqliteBackend::new(
            llmspell_storage::backends::sqlite::SqliteConfig::new(&db_path),
        )
        .await
        .expect("Failed to create SqliteBackend"),
    );

    let config =
        MemoryConfig::for_production(embedding_service).with_sqlite_backend(sqlite_backend);
    let backend = EpisodicBackend::from_config(&config).expect("backend creation failed");
    assert_eq!(backend.backend_name(), "Sqlite");
}

// ============================================================================
// Configuration Tests
// ============================================================================

#[tokio::test]
async fn test_config_for_testing() {
    let config = MemoryConfig::for_testing();
    let backend = EpisodicBackend::from_config(&config).expect("backend creation failed");
    assert_eq!(backend.backend_name(), "InMemory");
}

#[tokio::test]
async fn test_config_for_production() {
    let provider: Arc<dyn EmbeddingProvider> = Arc::new(TestEmbeddingProvider);
    let embedding_service = Arc::new(EmbeddingService::new(provider));

    let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
    let db_path = temp_dir.path().join("test_production.db");
    let sqlite_backend = Arc::new(
        llmspell_storage::backends::sqlite::SqliteBackend::new(
            llmspell_storage::backends::sqlite::SqliteConfig::new(&db_path),
        )
        .await
        .expect("Failed to create SqliteBackend"),
    );

    let config =
        MemoryConfig::for_production(embedding_service).with_sqlite_backend(sqlite_backend);
    let backend = EpisodicBackend::from_config(&config).expect("backend creation failed");
    assert_eq!(backend.backend_name(), "Sqlite");
}

#[tokio::test]
async fn test_config_sqlite_without_embedding_service() {
    use llmspell_memory::EpisodicBackendType;

    let config = MemoryConfig::default().with_backend(EpisodicBackendType::Sqlite);
    let result = EpisodicBackend::from_config(&config);
    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("SQLite backend requires embedding service"));
}
