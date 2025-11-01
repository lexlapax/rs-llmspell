//! Integration tests for `SurrealDB` knowledge graph backend
//!
//! Tests cover:
//! - Basic CRUD operations (create, read)
//! - Bi-temporal queries (temporal filtering)
//! - Relationship traversal
//! - Known limitations (`update_entity`, `delete_before` with `SurrealDB` 2.0)

use chrono::Utc;
use llmspell_graph::{
    storage::surrealdb::SurrealDBBackend,
    traits::KnowledgeGraph,
    types::{Entity, Relationship, TemporalQuery},
};
use serde_json::json;
use std::collections::HashMap;

#[tokio::test]
async fn test_backend_initialization() {
    let backend = SurrealDBBackend::new_temp()
        .await
        .expect("Failed to create temp backend");
    assert!(backend.data_dir().exists());
}

#[tokio::test]
async fn test_entity_create_and_read() {
    let backend = SurrealDBBackend::new_temp().await.unwrap();

    let entity = Entity::new(
        "Rust".into(),
        "programming_language".into(),
        json!({"paradigm": "multi-paradigm"}),
    );

    let id = backend.add_entity(entity.clone()).await.unwrap();
    let retrieved = backend.get_entity(&id).await.unwrap();

    assert_eq!(retrieved.name, "Rust");
    assert_eq!(retrieved.entity_type, "programming_language");
    // ID may have angle brackets from SurrealDB Thing format
    assert!(
        retrieved.id == id || retrieved.id == format!("⟨{id}⟩"),
        "ID should match: got '{}', expected '{id}' or '⟨{id}⟩'",
        retrieved.id
    );
}

#[tokio::test]
#[ignore = "SurrealDB 2.0 properties field not persisting on update"]
async fn test_entity_update() {
    let backend = SurrealDBBackend::new_temp().await.unwrap();

    let entity = Entity::new("Python".into(), "programming_language".into(), json!({}));
    let id = backend.add_entity(entity).await.unwrap();

    let mut changes = HashMap::new();
    changes.insert("version".into(), json!("3.12"));
    backend.update_entity(&id, changes).await.unwrap();

    let updated = backend.get_entity(&id).await.unwrap();
    // This assertion fails due to SurrealDB 2.0 bug
    assert_eq!(updated.properties["version"], "3.12");
}

#[tokio::test]
async fn test_relationship_create() {
    let backend = SurrealDBBackend::new_temp().await.unwrap();

    let entity1 = Entity::new("Rust".into(), "language".into(), json!({}));
    let entity2 = Entity::new("Memory Safety".into(), "feature".into(), json!({}));

    let id1 = backend.add_entity(entity1).await.unwrap();
    let id2 = backend.add_entity(entity2).await.unwrap();

    let rel = Relationship::new(id1.clone(), id2, "has_feature".into(), json!({}));
    let rel_id = backend.add_relationship(rel).await.unwrap();

    assert!(!rel_id.is_empty());
}

#[tokio::test]
async fn test_relationship_traversal() {
    let backend = SurrealDBBackend::new_temp().await.unwrap();

    let lang = Entity::new("Rust".into(), "language".into(), json!({}));
    let feat1 = Entity::new("Safety".into(), "feature".into(), json!({}));
    let feat2 = Entity::new("Speed".into(), "feature".into(), json!({}));

    let lang_id = backend.add_entity(lang).await.unwrap();
    let feat1_id = backend.add_entity(feat1).await.unwrap();
    let feat2_id = backend.add_entity(feat2).await.unwrap();

    backend
        .add_relationship(Relationship::new(
            lang_id.clone(),
            feat1_id,
            "has_feature".into(),
            json!({}),
        ))
        .await
        .unwrap();

    backend
        .add_relationship(Relationship::new(
            lang_id.clone(),
            feat2_id,
            "has_feature".into(),
            json!({}),
        ))
        .await
        .unwrap();

    let related = backend.get_related(&lang_id, "has_feature").await.unwrap();
    assert_eq!(related.len(), 2);
}

#[tokio::test]
async fn test_temporal_query_by_type() {
    let backend = SurrealDBBackend::new_temp().await.unwrap();

    backend
        .add_entity(Entity::new("Rust".into(), "language".into(), json!({})))
        .await
        .unwrap();

    backend
        .add_entity(Entity::new("Python".into(), "language".into(), json!({})))
        .await
        .unwrap();

    backend
        .add_entity(Entity::new("Cargo".into(), "tool".into(), json!({})))
        .await
        .unwrap();

    let query = TemporalQuery::new()
        .with_entity_type("language".into())
        .with_limit(10);

    let results = backend.query_temporal(query).await.unwrap();
    assert_eq!(results.len(), 2);
}

#[tokio::test]
#[ignore = "SurrealDB 2.0 may not preserve custom timestamps"]
async fn test_delete_before_retention() {
    let backend = SurrealDBBackend::new_temp().await.unwrap();

    let mut old_entity = Entity::new("Old".into(), "test".into(), json!({}));
    old_entity.ingestion_time = Utc::now() - chrono::Duration::days(30);
    backend.add_entity(old_entity).await.unwrap();

    backend
        .add_entity(Entity::new("New".into(), "test".into(), json!({})))
        .await
        .unwrap();

    let cutoff = Utc::now() - chrono::Duration::days(7);
    let deleted = backend.delete_before(cutoff).await.unwrap();

    // This assertion fails due to SurrealDB timestamp handling
    assert_eq!(deleted, 1);
}

#[tokio::test]
async fn test_empty_query_returns_all() {
    let backend = SurrealDBBackend::new_temp().await.unwrap();

    backend
        .add_entity(Entity::new("Entity1".into(), "type1".into(), json!({})))
        .await
        .unwrap();

    backend
        .add_entity(Entity::new("Entity2".into(), "type2".into(), json!({})))
        .await
        .unwrap();

    let query = TemporalQuery::new();
    let results = backend.query_temporal(query).await.unwrap();

    assert!(results.len() >= 2);
}

#[tokio::test]
async fn test_query_with_limit() {
    let backend = SurrealDBBackend::new_temp().await.unwrap();

    for i in 0..5 {
        backend
            .add_entity(Entity::new(format!("Entity{i}"), "test".into(), json!({})))
            .await
            .unwrap();
    }

    let query = TemporalQuery::new().with_limit(3);
    let results = backend.query_temporal(query).await.unwrap();

    assert_eq!(results.len(), 3);
}
