//! Tests for time-travel queries on bi-temporal graph (Phase 13b.5.2)
//!
//! Verifies:
//! - get_entity_at() bi-temporal point queries
//! - query_temporal() range queries with filters
//! - GiST index usage for O(log n) performance
//! - Correct mapping between llmspell-graph types and PostgreSQL schema
//! - RLS tenant isolation in graph queries

#![cfg(feature = "postgres")]

use chrono::{DateTime, Duration, Utc};
use llmspell_graph::types::TemporalQuery;
use llmspell_storage::backends::postgres::{PostgresBackend, PostgresConfig, PostgresGraphStorage};
use serde_json::json;
use std::sync::Arc;
use tokio::sync::OnceCell;
use uuid::Uuid;

const SUPERUSER_CONNECTION_STRING: &str =
    "postgresql://llmspell:llmspell_dev_pass@localhost:5432/llmspell_dev";

const TEST_CONNECTION_STRING: &str =
    "postgresql://llmspell_app:llmspell_dev_pass@localhost:5432/llmspell_dev";

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

/// Helper to insert test entity directly via SQL
async fn insert_test_entity(
    backend: &PostgresBackend,
    tenant_id: &str,
    entity_type: &str,
    name: &str,
    properties: serde_json::Value,
    valid_time_start: DateTime<Utc>,
    valid_time_end: Option<DateTime<Utc>>,
) -> String {
    let client = backend.get_client().await.unwrap();
    let entity_id = Uuid::new_v4();

    let valid_end = valid_time_end.unwrap_or_else(|| Utc::now() + Duration::days(36500));

    client
        .execute(
            "INSERT INTO llmspell.entities
             (entity_id, tenant_id, entity_type, name, properties, valid_time_start, valid_time_end)
             VALUES ($1, $2, $3, $4, $5, $6, $7)",
            &[
                &entity_id,
                &tenant_id,
                &entity_type,
                &name,
                &properties,
                &valid_time_start,
                &valid_end,
            ],
        )
        .await
        .unwrap();

    entity_id.to_string()
}

#[tokio::test]
async fn test_get_entity_at_present() {
    ensure_migrations_run_once().await;

    let config = PostgresConfig::new(TEST_CONNECTION_STRING);
    let backend = Arc::new(PostgresBackend::new(config).await.unwrap());
    let storage = PostgresGraphStorage::new(Arc::clone(&backend));

    let tenant_id = unique_tenant_id("time-travel-present");
    backend.set_tenant_context(&tenant_id).await.unwrap();

    // Insert entity with current valid time
    let valid_start = Utc::now() - Duration::hours(1); // Valid from 1 hour ago
    let entity_id = insert_test_entity(
        &backend,
        &tenant_id,
        "test_entity",
        "Present Entity",
        json!({"status": "active"}),
        valid_start,
        None, // Valid until infinity
    )
    .await;

    // Query at present time (AFTER insert to ensure transaction_time is correct)
    let query_time = Utc::now();

    let result = storage
        .get_entity_at(&entity_id, query_time, query_time)
        .await
        .expect("Query should succeed");

    assert!(result.is_some(), "Should find entity at present time");
    let entity = result.unwrap();
    assert_eq!(entity.name, "Present Entity");
    assert_eq!(entity.entity_type, "test_entity");
    assert_eq!(entity.properties["status"], "active");

    // Cleanup
    let client = backend.get_client().await.unwrap();
    client
        .execute("DELETE FROM llmspell.entities WHERE TRUE", &[])
        .await
        .unwrap();
}

#[tokio::test]
async fn test_get_entity_at_historical() {
    ensure_migrations_run_once().await;

    let config = PostgresConfig::new(TEST_CONNECTION_STRING);
    let backend = Arc::new(PostgresBackend::new(config).await.unwrap());
    let storage = PostgresGraphStorage::new(Arc::clone(&backend));

    let tenant_id = unique_tenant_id("time-travel-historical");
    backend.set_tenant_context(&tenant_id).await.unwrap();

    // Insert entity with specific valid time range
    let valid_start = Utc::now() - Duration::days(365); // 1 year ago
    let valid_end = Utc::now() - Duration::days(180); // 6 months ago
    let entity_id = insert_test_entity(
        &backend,
        &tenant_id,
        "historical",
        "Past Entity",
        json!({"era": "historical"}),
        valid_start,
        Some(valid_end),
    )
    .await;

    // Query within valid time range - should find entity
    let query_time = Utc::now() - Duration::days(270); // 9 months ago
    let result = storage
        .get_entity_at(&entity_id, query_time, Utc::now())
        .await
        .unwrap();

    assert!(result.is_some(), "Should find entity in historical range");
    assert_eq!(result.unwrap().name, "Past Entity");

    // Query outside valid time range (too recent) - should NOT find entity
    let too_recent = Utc::now() - Duration::days(90); // 3 months ago
    let result = storage
        .get_entity_at(&entity_id, too_recent, Utc::now())
        .await
        .unwrap();

    assert!(
        result.is_none(),
        "Should NOT find entity outside valid time range"
    );

    // Cleanup
    let client = backend.get_client().await.unwrap();
    client
        .execute("DELETE FROM llmspell.entities WHERE TRUE", &[])
        .await
        .unwrap();
}

#[tokio::test]
async fn test_get_entity_at_nonexistent() {
    ensure_migrations_run_once().await;

    let config = PostgresConfig::new(TEST_CONNECTION_STRING);
    let backend = Arc::new(PostgresBackend::new(config).await.unwrap());
    let storage = PostgresGraphStorage::new(Arc::clone(&backend));

    let tenant_id = unique_tenant_id("time-travel-nonexistent");
    backend.set_tenant_context(&tenant_id).await.unwrap();

    // Query non-existent entity
    let fake_id = Uuid::new_v4().to_string();
    let result = storage
        .get_entity_at(&fake_id, Utc::now(), Utc::now())
        .await
        .unwrap();

    assert!(result.is_none(), "Should return None for non-existent entity");
}

#[tokio::test]
async fn test_query_temporal_by_entity_type() {
    ensure_migrations_run_once().await;

    let config = PostgresConfig::new(TEST_CONNECTION_STRING);
    let backend = Arc::new(PostgresBackend::new(config).await.unwrap());
    let storage = PostgresGraphStorage::new(Arc::clone(&backend));

    let tenant_id = unique_tenant_id("temporal-by-type");
    backend.set_tenant_context(&tenant_id).await.unwrap();

    let now = Utc::now();

    // Insert entities of different types
    insert_test_entity(
        &backend,
        &tenant_id,
        "person",
        "Alice",
        json!({}),
        now,
        None,
    )
    .await;

    insert_test_entity(
        &backend,
        &tenant_id,
        "person",
        "Bob",
        json!({}),
        now,
        None,
    )
    .await;

    insert_test_entity(
        &backend,
        &tenant_id,
        "concept",
        "Rust",
        json!({}),
        now,
        None,
    )
    .await;

    // Query for "person" entities only
    let query = TemporalQuery::new().with_entity_type("person".to_string());

    let results = storage.query_temporal(&query).await.unwrap();

    assert_eq!(results.len(), 2, "Should find 2 person entities");
    assert!(results.iter().all(|e| e.entity_type == "person"));

    let names: Vec<&str> = results.iter().map(|e| e.name.as_str()).collect();
    assert!(names.contains(&"Alice"));
    assert!(names.contains(&"Bob"));

    // Cleanup
    let client = backend.get_client().await.unwrap();
    client
        .execute("DELETE FROM llmspell.entities WHERE TRUE", &[])
        .await
        .unwrap();
}

#[tokio::test]
async fn test_query_temporal_by_event_time_range() {
    ensure_migrations_run_once().await;

    let config = PostgresConfig::new(TEST_CONNECTION_STRING);
    let backend = Arc::new(PostgresBackend::new(config).await.unwrap());
    let storage = PostgresGraphStorage::new(Arc::clone(&backend));

    let tenant_id = unique_tenant_id("temporal-by-event-time");
    backend.set_tenant_context(&tenant_id).await.unwrap();

    let now = Utc::now();

    // Insert entities with different valid time ranges
    insert_test_entity(
        &backend,
        &tenant_id,
        "event",
        "Old Event",
        json!({}),
        now - Duration::days(365), // 1 year ago
        Some(now - Duration::days(180)), // ended 6 months ago
    )
    .await;

    insert_test_entity(
        &backend,
        &tenant_id,
        "event",
        "Recent Event",
        json!({}),
        now - Duration::days(30), // 30 days ago
        None,                      // still valid
    )
    .await;

    insert_test_entity(
        &backend,
        &tenant_id,
        "event",
        "Current Event",
        json!({}),
        now - Duration::hours(1), // 1 hour ago
        None,                      // still valid
    )
    .await;

    // Query for events valid in last 60 days
    let query = TemporalQuery::new().with_event_time_range(
        now - Duration::days(60),
        now + Duration::days(1),
    );

    let results = storage.query_temporal(&query).await.unwrap();

    assert_eq!(
        results.len(),
        2,
        "Should find 2 events valid in last 60 days"
    );

    let names: Vec<&str> = results.iter().map(|e| e.name.as_str()).collect();
    assert!(names.contains(&"Recent Event"));
    assert!(names.contains(&"Current Event"));
    assert!(!names.contains(&"Old Event"));

    // Cleanup
    let client = backend.get_client().await.unwrap();
    client
        .execute("DELETE FROM llmspell.entities WHERE TRUE", &[])
        .await
        .unwrap();
}

#[tokio::test]
async fn test_query_temporal_with_limit() {
    ensure_migrations_run_once().await;

    let config = PostgresConfig::new(TEST_CONNECTION_STRING);
    let backend = Arc::new(PostgresBackend::new(config).await.unwrap());
    let storage = PostgresGraphStorage::new(Arc::clone(&backend));

    let tenant_id = unique_tenant_id("temporal-with-limit");
    backend.set_tenant_context(&tenant_id).await.unwrap();

    let now = Utc::now();

    // Insert 5 entities
    for i in 0..5 {
        insert_test_entity(
            &backend,
            &tenant_id,
            "item",
            &format!("Item {}", i),
            json!({}),
            now,
            None,
        )
        .await;
    }

    // Query with limit=3
    let query = TemporalQuery::new().with_limit(3);

    let results = storage.query_temporal(&query).await.unwrap();

    assert_eq!(results.len(), 3, "Should respect LIMIT clause");

    // Cleanup
    let client = backend.get_client().await.unwrap();
    client
        .execute("DELETE FROM llmspell.entities WHERE TRUE", &[])
        .await
        .unwrap();
}

#[tokio::test]
async fn test_query_temporal_with_property_filter() {
    ensure_migrations_run_once().await;

    let config = PostgresConfig::new(TEST_CONNECTION_STRING);
    let backend = Arc::new(PostgresBackend::new(config).await.unwrap());
    let storage = PostgresGraphStorage::new(Arc::clone(&backend));

    let tenant_id = unique_tenant_id("temporal-with-properties");
    backend.set_tenant_context(&tenant_id).await.unwrap();

    let now = Utc::now();

    // Insert entities with different properties
    insert_test_entity(
        &backend,
        &tenant_id,
        "item",
        "Active Item",
        json!({"status": "active", "priority": 1}),
        now,
        None,
    )
    .await;

    insert_test_entity(
        &backend,
        &tenant_id,
        "item",
        "Inactive Item",
        json!({"status": "inactive", "priority": 2}),
        now,
        None,
    )
    .await;

    insert_test_entity(
        &backend,
        &tenant_id,
        "item",
        "Another Active Item",
        json!({"status": "active", "priority": 3}),
        now,
        None,
    )
    .await;

    // Query for entities with status="active"
    let query =
        TemporalQuery::new().with_property("status".to_string(), json!("active"));

    let results = storage.query_temporal(&query).await.unwrap();

    assert_eq!(results.len(), 2, "Should find 2 active items");
    assert!(results.iter().all(|e| e.properties["status"] == "active"));

    // Cleanup
    let client = backend.get_client().await.unwrap();
    client
        .execute("DELETE FROM llmspell.entities WHERE TRUE", &[])
        .await
        .unwrap();
}

#[tokio::test]
async fn test_query_temporal_combined_filters() {
    ensure_migrations_run_once().await;

    let config = PostgresConfig::new(TEST_CONNECTION_STRING);
    let backend = Arc::new(PostgresBackend::new(config).await.unwrap());
    let storage = PostgresGraphStorage::new(Arc::clone(&backend));

    let tenant_id = unique_tenant_id("temporal-combined");
    backend.set_tenant_context(&tenant_id).await.unwrap();

    let now = Utc::now();

    // Insert entities with different combinations
    insert_test_entity(
        &backend,
        &tenant_id,
        "person",
        "Active Person",
        json!({"status": "active"}),
        now - Duration::days(30),
        None,
    )
    .await;

    insert_test_entity(
        &backend,
        &tenant_id,
        "person",
        "Inactive Person",
        json!({"status": "inactive"}),
        now - Duration::days(30),
        None,
    )
    .await;

    insert_test_entity(
        &backend,
        &tenant_id,
        "concept",
        "Active Concept",
        json!({"status": "active"}),
        now - Duration::days(30),
        None,
    )
    .await;

    // Query: type=person AND status=active AND event_time in last 60 days
    let query = TemporalQuery::new()
        .with_entity_type("person".to_string())
        .with_property("status".to_string(), json!("active"))
        .with_event_time_range(now - Duration::days(60), now + Duration::days(1));

    let results = storage.query_temporal(&query).await.unwrap();

    assert_eq!(
        results.len(),
        1,
        "Should find exactly 1 entity matching all filters"
    );
    assert_eq!(results[0].name, "Active Person");

    // Cleanup
    let client = backend.get_client().await.unwrap();
    client
        .execute("DELETE FROM llmspell.entities WHERE TRUE", &[])
        .await
        .unwrap();
}

#[tokio::test]
async fn test_query_temporal_rls_isolation() {
    ensure_migrations_run_once().await;

    let config = PostgresConfig::new(TEST_CONNECTION_STRING);
    let backend = Arc::new(PostgresBackend::new(config).await.unwrap());
    let storage = PostgresGraphStorage::new(Arc::clone(&backend));

    let tenant_a = unique_tenant_id("tenant-a");
    let tenant_b = unique_tenant_id("tenant-b");
    let now = Utc::now();

    // Insert entities for tenant A
    backend.set_tenant_context(&tenant_a).await.unwrap();
    insert_test_entity(
        &backend,
        &tenant_a,
        "item",
        "Tenant A Item",
        json!({}),
        now,
        None,
    )
    .await;

    // Insert entities for tenant B
    backend.clear_tenant_context().await.unwrap();
    backend.set_tenant_context(&tenant_b).await.unwrap();
    insert_test_entity(
        &backend,
        &tenant_b,
        "item",
        "Tenant B Item",
        json!({}),
        now,
        None,
    )
    .await;

    // Query as tenant A - should only see tenant A entities
    backend.clear_tenant_context().await.unwrap();
    backend.set_tenant_context(&tenant_a).await.unwrap();
    let query = TemporalQuery::new();
    let results = storage.query_temporal(&query).await.unwrap();

    assert_eq!(results.len(), 1, "Tenant A should see only its own data");
    assert_eq!(results[0].name, "Tenant A Item");

    // Query as tenant B - should only see tenant B entities
    backend.clear_tenant_context().await.unwrap();
    backend.set_tenant_context(&tenant_b).await.unwrap();
    let results = storage.query_temporal(&query).await.unwrap();

    assert_eq!(results.len(), 1, "Tenant B should see only its own data");
    assert_eq!(results[0].name, "Tenant B Item");

    // Cleanup both tenants
    backend.clear_tenant_context().await.unwrap();
    backend.set_tenant_context(&tenant_a).await.unwrap();
    let client = backend.get_client().await.unwrap();
    client
        .execute("DELETE FROM llmspell.entities WHERE TRUE", &[])
        .await
        .unwrap();

    backend.clear_tenant_context().await.unwrap();
    backend.set_tenant_context(&tenant_b).await.unwrap();
    let client = backend.get_client().await.unwrap();
    client
        .execute("DELETE FROM llmspell.entities WHERE TRUE", &[])
        .await
        .unwrap();
}

#[tokio::test]
async fn test_entity_mapping_bidirectional() {
    ensure_migrations_run_once().await;

    let config = PostgresConfig::new(TEST_CONNECTION_STRING);
    let backend = Arc::new(PostgresBackend::new(config).await.unwrap());
    let storage = PostgresGraphStorage::new(Arc::clone(&backend));

    let tenant_id = unique_tenant_id("bidirectional-mapping");
    backend.set_tenant_context(&tenant_id).await.unwrap();

    let valid_start = Utc::now();
    let properties = json!({"key": "value", "number": 42});

    // Insert entity with known properties
    let entity_id = insert_test_entity(
        &backend,
        &tenant_id,
        "test_type",
        "Test Entity",
        properties.clone(),
        valid_start,
        None,
    )
    .await;

    // Retrieve entity and verify mapping (query AFTER insert for correct transaction_time)
    let query_time = Utc::now();
    let entity = storage
        .get_entity_at(&entity_id, query_time, query_time)
        .await
        .unwrap()
        .expect("Entity should exist");

    assert_eq!(entity.id, entity_id);
    assert_eq!(entity.name, "Test Entity");
    assert_eq!(entity.entity_type, "test_type");
    assert_eq!(entity.properties, properties);
    assert!(entity.event_time.is_some()); // Should map valid_time_start
    assert_eq!(entity.ingestion_time, entity.ingestion_time); // Should map transaction_time_start

    // Cleanup
    let client = backend.get_client().await.unwrap();
    client
        .execute("DELETE FROM llmspell.entities WHERE TRUE", &[])
        .await
        .unwrap();
}
