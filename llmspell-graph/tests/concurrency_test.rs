//! Concurrency tests for knowledge graph operations
//!
//! Tests cover:
//! - Concurrent entity creation
//! - Concurrent relationship traversal
//! - Thread-safe backend access

use llmspell_graph::{
    storage::surrealdb::SurrealDBBackend, traits::KnowledgeGraph, types::Entity,
};
use serde_json::json;
use std::sync::Arc;

#[tokio::test]
async fn test_concurrent_entity_creation() {
    let backend = Arc::new(SurrealDBBackend::new_temp().await.unwrap());

    // Create 10 entities concurrently
    let mut handles = vec![];

    for i in 0..10 {
        let backend_clone = Arc::clone(&backend);
        let handle = tokio::spawn(async move {
            let entity = Entity::new(
                format!("Entity{}", i),
                "concurrent_test".into(),
                json!({"index": i}),
            );
            backend_clone.add_entity(entity).await
        });
        handles.push(handle);
    }

    // Wait for all to complete
    let mut results = vec![];
    for handle in handles {
        let result = handle.await.unwrap();
        assert!(result.is_ok(), "Concurrent entity creation should succeed");
        results.push(result.unwrap());
    }

    // Verify all entities were created with unique IDs
    assert_eq!(results.len(), 10);
    let unique_ids: std::collections::HashSet<_> = results.into_iter().collect();
    assert_eq!(unique_ids.len(), 10, "All entities should have unique IDs");
}

#[tokio::test]
async fn test_concurrent_relationship_traversal() {
    let backend = Arc::new(SurrealDBBackend::new_temp().await.unwrap());

    // Create a central entity with multiple relationships
    let central = Entity::new("Central".into(), "hub".into(), json!({}));
    let central_id = backend.add_entity(central).await.unwrap();

    // Add 5 related entities
    for i in 0..5 {
        let entity = Entity::new(format!("Related{}", i), "spoke".into(), json!({}));
        let entity_id = backend.add_entity(entity).await.unwrap();

        let rel = llmspell_graph::types::Relationship::new(
            central_id.clone(),
            entity_id,
            "connects_to".into(),
            json!({}),
        );
        backend.add_relationship(rel).await.unwrap();
    }

    // Concurrently query relationships from multiple tasks
    let mut handles = vec![];

    for _ in 0..10 {
        let backend_clone = Arc::clone(&backend);
        let id_clone = central_id.clone();
        let handle = tokio::spawn(async move {
            backend_clone.get_related(&id_clone, "connects_to").await
        });
        handles.push(handle);
    }

    // Verify all concurrent reads succeed and return correct count
    for handle in handles {
        let result = handle.await.unwrap();
        assert!(result.is_ok(), "Concurrent relationship reads should succeed");
        let related = result.unwrap();
        assert_eq!(
            related.len(),
            5,
            "All concurrent reads should see 5 relationships"
        );
    }
}

#[tokio::test]
async fn test_concurrent_mixed_operations() {
    let backend = Arc::new(SurrealDBBackend::new_temp().await.unwrap());

    // Create initial entity
    let entity = Entity::new("Initial".into(), "test".into(), json!({}));
    let entity_id = backend.add_entity(entity).await.unwrap();

    // Mix reads and writes concurrently
    let mut read_handles = vec![];
    let mut write_handles = vec![];

    // 5 concurrent reads
    for _ in 0..5 {
        let backend_clone = Arc::clone(&backend);
        let id_clone = entity_id.clone();
        let handle = tokio::spawn(async move { backend_clone.get_entity(&id_clone).await });
        read_handles.push(handle);
    }

    // 5 concurrent writes
    for i in 0..5 {
        let backend_clone = Arc::clone(&backend);
        let handle = tokio::spawn(async move {
            let entity = Entity::new(format!("Concurrent{}", i), "test".into(), json!({}));
            backend_clone.add_entity(entity).await
        });
        write_handles.push(handle);
    }

    // Verify all read operations complete successfully
    for handle in read_handles {
        let result = handle.await.unwrap();
        assert!(result.is_ok(), "Concurrent reads should succeed");
        let entity = result.unwrap();
        assert_eq!(entity.name, "Initial");
    }

    // Verify all write operations complete successfully
    for handle in write_handles {
        let result = handle.await.unwrap();
        assert!(result.is_ok(), "Concurrent writes should succeed");
        let id = result.unwrap();
        assert!(!id.is_empty());
    }
}
