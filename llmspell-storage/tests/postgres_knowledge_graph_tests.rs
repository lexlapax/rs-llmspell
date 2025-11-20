//! Tests for KnowledgeGraph trait implementation (Phase 13b.5.4)
//!
//! Verifies:
//! - add_entity() and get_entity() CRUD operations
//! - update_entity() bi-temporal versioning
//! - add_relationship() and get_related()
//! - delete_before() data retention
//! - Trait method delegation to existing implementations

#![cfg(feature = "postgres")]

use chrono::{Duration, Utc};
use llmspell_graph::traits::KnowledgeGraph;
use llmspell_graph::{Entity, Relationship};
use llmspell_storage::backends::postgres::{PostgresBackend, PostgresConfig, PostgresGraphStorage};
use serde_json::json;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::OnceCell;
use uuid::Uuid;

const SUPERUSER_CONNECTION_STRING: &str =
    "postgresql://llmspell:llmspell_dev_pass@localhost:5432/llmspell_dev";

const APP_CONNECTION_STRING: &str =
    "postgresql://llmspell_app:llmspell_app_pass@localhost:5432/llmspell_dev";

static MIGRATION_INIT: OnceCell<()> = OnceCell::const_new();

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

fn unique_tenant_id(prefix: &str) -> String {
    format!("{}-{}", prefix, Uuid::new_v4())
}

#[tokio::test]
async fn test_add_and_get_entity() {
    ensure_migrations_run_once().await;

    let tenant_id = unique_tenant_id("kg-add-get");
    let config = PostgresConfig::new(APP_CONNECTION_STRING);
    let backend = Arc::new(PostgresBackend::new(config).await.expect("create backend"));
    backend
        .set_tenant_context(&tenant_id)
        .await
        .expect("set tenant");

    let graph = PostgresGraphStorage::new(Arc::clone(&backend));
    let now = Utc::now();

    // Create entity
    let entity = Entity {
        id: "".to_string(), // Will be generated
        name: "Test Entity".to_string(),
        entity_type: "Person".to_string(),
        properties: json!({"age": 30, "city": "SF"}),
        event_time: Some(now),
        ingestion_time: now,
    };

    let entity_id = graph.add_entity(entity).await.expect("add_entity");

    // Get entity back
    let retrieved = graph.get_entity(&entity_id).await.expect("get_entity");

    assert_eq!(retrieved.name, "Test Entity");
    assert_eq!(retrieved.entity_type, "Person");
    assert_eq!(retrieved.properties["age"], 30);
    assert_eq!(retrieved.properties["city"], "SF");
}

#[tokio::test]
async fn test_update_entity_bi_temporal() {
    ensure_migrations_run_once().await;

    let tenant_id = unique_tenant_id("kg-update");
    let config = PostgresConfig::new(APP_CONNECTION_STRING);
    let backend = Arc::new(PostgresBackend::new(config).await.expect("create backend"));
    backend
        .set_tenant_context(&tenant_id)
        .await
        .expect("set tenant");

    let graph = PostgresGraphStorage::new(Arc::clone(&backend));
    let now = Utc::now();

    // Create entity
    let entity = Entity {
        id: "".to_string(),
        name: "John Doe".to_string(),
        entity_type: "Person".to_string(),
        properties: json!({"age": 30, "city": "SF"}),
        event_time: Some(now),
        ingestion_time: now,
    };

    let entity_id = graph.add_entity(entity).await.expect("add_entity");

    // Update entity
    let mut changes = HashMap::new();
    changes.insert("age".to_string(), json!(31));
    changes.insert("updated".to_string(), json!(true));

    graph
        .update_entity(&entity_id, changes)
        .await
        .expect("update_entity");

    // Get updated entity
    let updated = graph.get_entity(&entity_id).await.expect("get_entity");

    assert_eq!(updated.properties["age"], 31);
    assert_eq!(updated.properties["city"], "SF"); // Unchanged
    assert_eq!(updated.properties["updated"], true); // New field
}

#[tokio::test]
async fn test_add_relationship_and_get_related() {
    ensure_migrations_run_once().await;

    let tenant_id = unique_tenant_id("kg-relationship");
    let config = PostgresConfig::new(APP_CONNECTION_STRING);
    let backend = Arc::new(PostgresBackend::new(config).await.expect("create backend"));
    backend
        .set_tenant_context(&tenant_id)
        .await
        .expect("set tenant");

    let graph = PostgresGraphStorage::new(Arc::clone(&backend));
    let now = Utc::now();

    // Create entities
    let person = Entity {
        id: "".to_string(),
        name: "Alice".to_string(),
        entity_type: "Person".to_string(),
        properties: json!({}),
        event_time: Some(now),
        ingestion_time: now,
    };

    let company = Entity {
        id: "".to_string(),
        name: "Acme Corp".to_string(),
        entity_type: "Company".to_string(),
        properties: json!({}),
        event_time: Some(now),
        ingestion_time: now,
    };

    let person_id = graph.add_entity(person).await.expect("add person");
    let company_id = graph.add_entity(company).await.expect("add company");

    // Create relationship
    let relationship = Relationship {
        id: "".to_string(),
        from_entity: person_id.clone(),
        to_entity: company_id.clone(),
        relationship_type: "works_at".to_string(),
        properties: json!({"since": "2020"}),
        event_time: Some(now),
        ingestion_time: now,
    };

    graph
        .add_relationship(relationship)
        .await
        .expect("add_relationship");

    // Get related entities (using KnowledgeGraph trait method)
    let related = KnowledgeGraph::get_related(&graph, &person_id, "works_at")
        .await
        .expect("get_related");

    assert_eq!(related.len(), 1);
    assert_eq!(related[0].name, "Acme Corp");
    assert_eq!(related[0].entity_type, "Company");
}

#[tokio::test]
async fn test_delete_before() {
    ensure_migrations_run_once().await;

    let tenant_id = unique_tenant_id("kg-delete");
    let config = PostgresConfig::new(APP_CONNECTION_STRING);
    let backend = Arc::new(PostgresBackend::new(config).await.expect("create backend"));
    backend
        .set_tenant_context(&tenant_id)
        .await
        .expect("set tenant");

    let graph = PostgresGraphStorage::new(Arc::clone(&backend));
    let now = Utc::now();

    // Create old entity (simulate by backdating ingestion_time)
    let old_time = now - Duration::hours(2);
    let entity = Entity {
        id: "".to_string(),
        name: "Old Entity".to_string(),
        entity_type: "Test".to_string(),
        properties: json!({"version": 1}),
        event_time: Some(old_time),
        ingestion_time: old_time,
    };

    let entity_id = graph.add_entity(entity).await.expect("add_entity");

    // Update entity to create a new version (with current transaction_time)
    let mut changes = HashMap::new();
    changes.insert("version".to_string(), json!(2));
    changes.insert("updated".to_string(), json!(true));
    graph
        .update_entity(&entity_id, changes)
        .await
        .expect("update_entity");

    // Verify current entity exists and has updated properties
    let retrieved = graph.get_entity(&entity_id).await.expect("get_entity");
    assert_eq!(retrieved.properties["version"], 2);
    assert_eq!(retrieved.properties["updated"], true);

    // Delete entity versions older than 1 hour ago
    // This should delete the old version (2 hours ago) but preserve the current version
    let cutoff = now - Duration::hours(1);
    let deleted_count = graph.delete_before(cutoff).await.expect("delete_before");

    assert!(
        deleted_count > 0,
        "Should have deleted at least 1 old version"
    );

    // Current version should still be retrievable
    // (delete_before only removes old versions based on transaction_time)
    let after_delete = graph.get_entity(&entity_id).await;
    assert!(after_delete.is_ok(), "Current version should still exist");
    let current = after_delete.unwrap();
    assert_eq!(
        current.properties["version"], 2,
        "Should retrieve current version with updated properties"
    );
}

#[tokio::test]
async fn test_tenant_isolation() {
    ensure_migrations_run_once().await;

    let tenant_a = unique_tenant_id("kg-tenant-a");
    let tenant_b = unique_tenant_id("kg-tenant-b");
    let config = PostgresConfig::new(APP_CONNECTION_STRING);

    // Setup for tenant A
    let backend_a = Arc::new(
        PostgresBackend::new(config.clone())
            .await
            .expect("create backend"),
    );
    backend_a
        .set_tenant_context(&tenant_a)
        .await
        .expect("set tenant");
    let graph_a = PostgresGraphStorage::new(Arc::clone(&backend_a));
    let now = Utc::now();

    let entity_a = Entity {
        id: "".to_string(),
        name: "Tenant A Entity".to_string(),
        entity_type: "Test".to_string(),
        properties: json!({}),
        event_time: Some(now),
        ingestion_time: now,
    };

    let id_a = graph_a.add_entity(entity_a).await.expect("add entity A");

    // Setup for tenant B
    let backend_b = Arc::new(PostgresBackend::new(config).await.expect("create backend"));
    backend_b
        .set_tenant_context(&tenant_b)
        .await
        .expect("set tenant");
    let graph_b = PostgresGraphStorage::new(Arc::clone(&backend_b));

    // Tenant B should not see Tenant A's entity
    let result = graph_b.get_entity(&id_a).await;
    assert!(result.is_err(), "Tenant B should not see Tenant A's entity");
}

#[tokio::test]
async fn test_get_relationships() {
    ensure_migrations_run_once().await;

    let tenant_id = unique_tenant_id("kg-get-relationships");
    let config = PostgresConfig::new(APP_CONNECTION_STRING);
    let backend = Arc::new(PostgresBackend::new(config).await.expect("create backend"));
    backend
        .set_tenant_context(&tenant_id)
        .await
        .expect("set tenant");

    let graph = PostgresGraphStorage::new(Arc::clone(&backend));
    let now = Utc::now();

    // Create entities
    let person = Entity {
        id: "".to_string(),
        name: "Bob".to_string(),
        entity_type: "Person".to_string(),
        properties: json!({}),
        event_time: Some(now),
        ingestion_time: now,
    };

    let company1 = Entity {
        id: "".to_string(),
        name: "TechCorp".to_string(),
        entity_type: "Company".to_string(),
        properties: json!({}),
        event_time: Some(now),
        ingestion_time: now,
    };

    let company2 = Entity {
        id: "".to_string(),
        name: "StartupInc".to_string(),
        entity_type: "Company".to_string(),
        properties: json!({}),
        event_time: Some(now),
        ingestion_time: now,
    };

    let person_id = graph.add_entity(person).await.expect("add person");
    let company1_id = graph.add_entity(company1).await.expect("add company1");
    let company2_id = graph.add_entity(company2).await.expect("add company2");

    // Create outgoing relationship (person -> company1)
    let relationship1 = Relationship {
        id: "".to_string(),
        from_entity: person_id.clone(),
        to_entity: company1_id.clone(),
        relationship_type: "works_at".to_string(),
        properties: json!({"since": "2020"}),
        event_time: Some(now),
        ingestion_time: now,
    };

    // Create incoming relationship (company2 -> person)
    let relationship2 = Relationship {
        id: "".to_string(),
        from_entity: company2_id.clone(),
        to_entity: person_id.clone(),
        relationship_type: "contacted".to_string(),
        properties: json!({"date": "2024-01-15"}),
        event_time: Some(now),
        ingestion_time: now,
    };

    let rel1_id = graph
        .add_relationship(relationship1)
        .await
        .expect("add relationship1");

    let rel2_id = graph
        .add_relationship(relationship2)
        .await
        .expect("add relationship2");

    // Get all relationships for person (should return both outgoing and incoming)
    let relationships = graph
        .get_relationships(&person_id)
        .await
        .expect("get_relationships");

    assert_eq!(
        relationships.len(),
        2,
        "Should find 2 relationships (1 outgoing, 1 incoming)"
    );

    // Verify relationship 1 (outgoing)
    let rel1 = relationships
        .iter()
        .find(|r| r.id == rel1_id)
        .expect("find relationship 1");
    assert_eq!(rel1.from_entity, person_id);
    assert_eq!(rel1.to_entity, company1_id);
    assert_eq!(rel1.relationship_type, "works_at");
    assert_eq!(rel1.properties["since"], "2020");

    // Verify relationship 2 (incoming)
    let rel2 = relationships
        .iter()
        .find(|r| r.id == rel2_id)
        .expect("find relationship 2");
    assert_eq!(rel2.from_entity, company2_id);
    assert_eq!(rel2.to_entity, person_id);
    assert_eq!(rel2.relationship_type, "contacted");
    assert_eq!(rel2.properties["date"], "2024-01-15");

    // Verify company1 has only 1 relationship (incoming from person)
    let company1_rels = graph
        .get_relationships(&company1_id)
        .await
        .expect("get company1 relationships");
    assert_eq!(company1_rels.len(), 1);
    assert_eq!(company1_rels[0].relationship_type, "works_at");
}
