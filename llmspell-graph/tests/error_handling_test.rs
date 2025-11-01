//! Error handling tests for knowledge graph operations
//!
//! Tests cover:
//! - Entity not found scenarios
//! - Invalid relationship references
//! - Empty query results

use llmspell_graph::{
    storage::surrealdb::SurrealDBBackend,
    traits::KnowledgeGraph,
    types::{Entity, Relationship, TemporalQuery},
};
use serde_json::json;

#[tokio::test]
async fn test_get_nonexistent_entity() {
    let backend = SurrealDBBackend::new_temp().await.unwrap();

    let result = backend.get_entity("nonexistent-id").await;

    assert!(
        result.is_err(),
        "Should return error for nonexistent entity"
    );
    let err_msg = result.unwrap_err().to_string();
    assert!(
        err_msg.contains("not found") || err_msg.contains("No entity"),
        "Error should indicate entity not found: {err_msg}"
    );
}

#[tokio::test]
async fn test_get_related_nonexistent_entity() {
    let backend = SurrealDBBackend::new_temp().await.unwrap();

    let result = backend.get_related("nonexistent-id", "any_relation").await;

    // get_related returns empty vec for nonexistent entities (not an error)
    assert!(result.is_ok());
    assert_eq!(result.unwrap().len(), 0);
}

#[tokio::test]
async fn test_relationship_with_invalid_entity_references() {
    let backend = SurrealDBBackend::new_temp().await.unwrap();

    // Create relationship with both entities missing
    let rel = Relationship::new(
        "invalid-from".into(),
        "invalid-to".into(),
        "test_relation".into(),
        json!({}),
    );

    // SurrealDB allows relationships without enforcing foreign keys by default
    // This test verifies behavior - it may succeed or fail depending on config
    let result = backend.add_relationship(rel).await;

    // Document actual behavior: SurrealDB 2.0 allows dangling relationships
    if result.is_ok() {
        // Verify the relationship was created (SurrealDB allows it)
        let rel_id = result.unwrap();
        assert!(!rel_id.is_empty());
    } else {
        // If enforced, should get foreign key error
        let err_msg = result.unwrap_err().to_string();
        assert!(
            err_msg.contains("not found") || err_msg.contains("foreign key"),
            "Error should indicate invalid reference: {err_msg}"
        );
    }
}

#[tokio::test]
async fn test_query_with_no_matches() {
    let backend = SurrealDBBackend::new_temp().await.unwrap();

    // Add some entities
    backend
        .add_entity(Entity::new("Test".into(), "type1".into(), json!({})))
        .await
        .unwrap();

    // Query for non-matching type
    let query = TemporalQuery::new().with_entity_type("nonexistent_type".into());
    let results = backend.query_temporal(query).await.unwrap();

    assert_eq!(
        results.len(),
        0,
        "Should return empty results for no matches"
    );
}

#[tokio::test]
async fn test_empty_database_query() {
    let backend = SurrealDBBackend::new_temp().await.unwrap();

    // Query empty database
    let query = TemporalQuery::new();
    let results = backend.query_temporal(query).await.unwrap();

    assert_eq!(results.len(), 0, "Empty database should return 0 results");
}
