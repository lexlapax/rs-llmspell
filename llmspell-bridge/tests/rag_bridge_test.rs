//! ABOUTME: Integration tests for RAG bridge functionality
//! ABOUTME: Tests RAG operations with real vector storage

use llmspell_bridge::rag_bridge::{ChunkingConfig, RAGBridge, RAGDocument};
use llmspell_bridge::ProviderManager;
use llmspell_config::providers::ProviderManagerConfig;
use llmspell_kernel::sessions::{SessionManager, SessionManagerConfig};
use llmspell_kernel::state::StateManager;
use llmspell_rag::multi_tenant_integration::MultiTenantRAG;
use llmspell_storage::backends::sqlite::{SqliteBackend, SqliteConfig, SqliteVectorStorage};
use llmspell_storage::MemoryBackend;
use llmspell_tenancy::manager::MultiTenantVectorManager;
use std::collections::HashMap;
use std::sync::Arc;

async fn setup_test_bridge() -> Arc<RAGBridge> {
    // Setup state manager
    let state_manager = Arc::new(StateManager::new(None).await.unwrap());

    // Setup session manager
    let storage_backend = Arc::new(MemoryBackend::new());
    let hook_registry = Arc::new(llmspell_hooks::HookRegistry::new());
    let hook_executor = Arc::new(llmspell_hooks::HookExecutor::new());
    let event_bus = Arc::new(llmspell_events::bus::EventBus::new());
    let session_config = SessionManagerConfig::default();

    let session_manager = Arc::new(
        SessionManager::new(
            state_manager.clone(),
            storage_backend,
            hook_registry,
            hook_executor,
            &event_bus,
            session_config,
        )
        .unwrap(),
    );

    // Setup SQLite vector storage (in-memory for testing)
    let config = SqliteConfig::in_memory();
    let backend = Arc::new(SqliteBackend::new(config).await.unwrap());
    backend.run_migrations().await.unwrap(); // Run migrations to create vector_metadata table
    let vector_storage = Arc::new(SqliteVectorStorage::new(backend, 384).await.unwrap()); // 384 dimensions

    // Setup multi-tenant infrastructure
    let tenant_manager = Arc::new(MultiTenantVectorManager::new(vector_storage.clone()));
    let multi_tenant_rag = Arc::new(MultiTenantRAG::new(tenant_manager));

    // Create state-aware storage with real HNSW storage
    let state_aware_storage = Arc::new(
        llmspell_rag::state_integration::StateAwareVectorStorage::new(
            vector_storage,
            state_manager,
            multi_tenant_rag.clone(),
        ),
    );

    // Setup provider manager
    let config = ProviderManagerConfig::default();
    let providers = Arc::new(ProviderManager::new(config).await.unwrap());
    let core_providers = providers.create_core_manager_arc().await.unwrap();

    Arc::new(RAGBridge::from_components(
        state_aware_storage,
        session_manager,
        multi_tenant_rag,
        core_providers,
    ))
}

#[tokio::test]
async fn test_rag_bridge_search_basic() {
    let bridge = setup_test_bridge().await;

    // First ingest some documents
    let mut metadata1 = HashMap::new();
    metadata1.insert("source".to_string(), serde_json::json!("test1.txt"));

    let mut metadata2 = HashMap::new();
    metadata2.insert("source".to_string(), serde_json::json!("test2.txt"));

    let documents = vec![
        RAGDocument {
            id: "doc1".to_string(),
            text: "The quick brown fox jumps over the lazy dog".to_string(),
            metadata: Some(metadata1),
        },
        RAGDocument {
            id: "doc2".to_string(),
            text: "Machine learning is a subset of artificial intelligence".to_string(),
            metadata: Some(metadata2),
        },
    ];

    let ingest_response = bridge
        .ingest(documents, None, None, None, None, None)
        .await
        .unwrap();
    assert_eq!(ingest_response.documents_processed, 2);

    // Now search
    let search_params = llmspell_bridge::rag_bridge::RAGSearchParams {
        query: "fox jumps".to_string(),
        k: Some(5),
        scope: None,
        scope_id: None,
        filters: None,
        threshold: None,
        context: None,
    };
    let search_response = bridge.search(search_params).await.unwrap();
    assert!(!search_response.results.is_empty());
}

#[tokio::test]
async fn test_rag_bridge_ingest_with_chunking() {
    let bridge = setup_test_bridge().await;

    let long_text = "This is a very long document. ".repeat(100);

    let documents = vec![RAGDocument {
        id: "long_doc".to_string(),
        text: long_text,
        metadata: None,
    }];

    let chunking = ChunkingConfig {
        chunk_size: Some(100),
        overlap: Some(20),
        strategy: Some("sliding_window".to_string()),
    };

    let response = bridge
        .ingest(documents, None, None, None, Some(chunking), None)
        .await
        .unwrap();
    assert_eq!(response.documents_processed, 1);
    // Should create multiple vectors due to chunking
    assert!(response.vectors_created >= response.documents_processed);
}

#[tokio::test]
async fn test_rag_bridge_search_with_filters() {
    let bridge = setup_test_bridge().await;

    // Ingest documents with metadata
    let mut metadata = HashMap::new();
    metadata.insert("category".to_string(), serde_json::json!("science"));
    metadata.insert("year".to_string(), serde_json::json!(2024));

    let documents = vec![RAGDocument {
        id: "sci_doc".to_string(),
        text: "Quantum computing breakthrough announced".to_string(),
        metadata: Some(metadata.clone()),
    }];

    bridge
        .ingest(documents, None, None, None, None, None)
        .await
        .unwrap();

    // Search with filters
    let mut filters = HashMap::new();
    filters.insert("category".to_string(), serde_json::json!("science"));

    let search_params = llmspell_bridge::rag_bridge::RAGSearchParams {
        query: "quantum".to_string(),
        k: Some(5),
        scope: None,
        scope_id: None,
        filters: Some(filters),
        threshold: Some(0.5),
        context: None,
    };
    let response = bridge.search(search_params).await.unwrap();
    assert!(!response.results.is_empty());
}

#[tokio::test]
async fn test_rag_bridge_cleanup_scope() {
    let bridge = setup_test_bridge().await;

    // Ingest documents
    let documents = vec![RAGDocument {
        id: "cleanup_doc".to_string(),
        text: "Document to be cleaned up".to_string(),
        metadata: None,
    }];

    bridge
        .ingest(
            documents,
            Some("test".to_string()),
            Some("test_cleanup".to_string()),
            None,
            None,
            None,
        )
        .await
        .unwrap();

    // Clean up the scope
    let deleted = bridge.cleanup_scope("test", "test_cleanup").await.unwrap();
    assert!(deleted > 0);

    // Verify documents are gone
    let search_params = llmspell_bridge::rag_bridge::RAGSearchParams {
        query: "cleanup".to_string(),
        k: Some(5),
        scope: Some("test".to_string()),
        scope_id: Some("test_cleanup".to_string()),
        filters: None,
        threshold: None,
        context: None,
    };
    let response = bridge.search(search_params).await.unwrap();
    assert_eq!(response.total, 0);
    assert!(response.results.is_empty());
}

#[tokio::test]
async fn test_rag_bridge_get_stats() {
    let bridge = setup_test_bridge().await;

    // Ingest some documents
    let documents = vec![
        RAGDocument {
            id: "stats_doc1".to_string(),
            text: "First document for stats".to_string(),
            metadata: None,
        },
        RAGDocument {
            id: "stats_doc2".to_string(),
            text: "Second document for stats".to_string(),
            metadata: None,
        },
    ];

    bridge
        .ingest(
            documents,
            Some("test".to_string()),
            Some("test_stats".to_string()),
            None,
            None,
            None,
        )
        .await
        .unwrap();

    // Get stats
    let stats = bridge.get_stats("test", Some("test_stats")).await.unwrap();
    assert!(!stats.is_empty());
    assert!(stats.contains_key("total_vectors"));
}

#[tokio::test]
async fn test_rag_bridge_configure() {
    let bridge = setup_test_bridge().await;

    // Configure RAG settings
    // Should not error
    bridge
        .configure(
            Some(3600),
            Some("openai".to_string()),
            Some(true),
            Some(1800),
        )
        .unwrap();
}

#[tokio::test]
async fn test_rag_bridge_list_providers() {
    let bridge = setup_test_bridge().await;

    let providers = bridge.list_providers().unwrap();
    assert!(!providers.is_empty());
    // Mock provider should be available
    assert!(providers.contains(&"mock".to_string()));
}

#[tokio::test]
async fn test_rag_bridge_concurrent_operations() {
    let bridge = setup_test_bridge().await;

    // Test concurrent ingestion and search
    let bridge1 = bridge.clone();
    let bridge2 = bridge.clone();

    let ingest_handle = tokio::spawn(async move {
        let documents = vec![RAGDocument {
            id: "concurrent_doc".to_string(),
            text: "Testing concurrent operations".to_string(),
            metadata: None,
        }];
        bridge1
            .ingest(
                documents,
                Some("test".to_string()),
                Some("test_concurrent".to_string()),
                None,
                None,
                None,
            )
            .await
    });

    let search_handle = tokio::spawn(async move {
        // Small delay to let ingest start
        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;

        bridge2
            .search(llmspell_bridge::rag_bridge::RAGSearchParams {
                query: "concurrent".to_string(),
                k: Some(5),
                scope: Some("test".to_string()),
                scope_id: Some("test_concurrent".to_string()),
                filters: None,
                threshold: None,
                context: None,
            })
            .await
    });

    // Both should complete successfully
    let ingest_result = ingest_handle.await.unwrap();
    assert!(ingest_result.is_ok());

    let search_result = search_handle.await.unwrap();
    assert!(search_result.is_ok());
}

#[tokio::test]
async fn test_rag_bridge_error_handling() {
    let bridge = setup_test_bridge().await;

    // Test empty document list
    let response = bridge
        .ingest(vec![], None, None, None, None, None)
        .await
        .unwrap();
    assert_eq!(response.documents_processed, 0);

    // Test search with invalid scope (should return empty)
    let search_params = llmspell_bridge::rag_bridge::RAGSearchParams {
        query: "test".to_string(),
        k: Some(5),
        scope: Some("nonexistent".to_string()),
        scope_id: Some("nonexistent".to_string()),
        filters: None,
        threshold: None,
        context: None,
    };
    let response = bridge.search(search_params).await.unwrap();
    assert_eq!(response.total, 0);
}
