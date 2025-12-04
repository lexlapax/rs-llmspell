//! Tests for recursive graph traversal with CTEs (Phase 13b.5.3)
//!
//! Verifies:
//! - get_related() 1-4 hop traversals
//! - Cycle prevention with path tracking
//! - Relationship type filtering
//! - Performance <50ms for typical graphs
//! - Tenant isolation in graph traversals

#![cfg(feature = "postgres")]

use chrono::{DateTime, Utc};
use llmspell_storage::backends::postgres::{PostgresBackend, PostgresConfig, PostgresGraphStorage};
use serde_json::json;
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

/// Helper to insert test entity directly via SQL
async fn insert_test_entity(
    backend: &PostgresBackend,
    tenant_id: &str,
    entity_id: Uuid,
    entity_type: &str,
    name: &str,
    valid_time: DateTime<Utc>,
) {
    let client = backend.get_client().await.expect("get client");

    client
        .execute(
            "INSERT INTO llmspell.entities
             (tenant_id, entity_id, entity_type, name, properties, valid_time_start, valid_time_end, transaction_time_start)
             VALUES ($1, $2, $3, $4, $5, $6, 'infinity', CURRENT_TIMESTAMP)",
            &[&tenant_id, &entity_id, &entity_type, &name, &json!({}), &valid_time],
        )
        .await
        .expect("insert entity");
}

/// Helper to insert test relationship directly via SQL
async fn insert_test_relationship(
    backend: &PostgresBackend,
    tenant_id: &str,
    from_entity: Uuid,
    to_entity: Uuid,
    relationship_type: &str,
    valid_time: DateTime<Utc>,
) {
    let client = backend.get_client().await.expect("get client");

    client
        .execute(
            "INSERT INTO llmspell.relationships
             (tenant_id, from_entity, to_entity, relationship_type, properties, valid_time_start, valid_time_end, transaction_time_start)
             VALUES ($1, $2, $3, $4, $5, $6, 'infinity', CURRENT_TIMESTAMP)",
            &[&tenant_id, &from_entity, &to_entity, &relationship_type, &json!({}), &valid_time],
        )
        .await
        .expect("insert relationship");
}

#[tokio::test]
async fn test_graph_traversal_1_hop() {
    ensure_migrations_run_once().await;

    let tenant_id = unique_tenant_id("traversal-1hop");
    let config = PostgresConfig::new(APP_CONNECTION_STRING);
    let backend = Arc::new(PostgresBackend::new(config).await.expect("create backend"));
    backend
        .set_tenant_context(&tenant_id)
        .await
        .expect("set tenant context");

    let graph = PostgresGraphStorage::new(Arc::clone(&backend));
    let now = Utc::now();

    // Create simple graph: A -> B -> C
    let entity_a = Uuid::new_v4();
    let entity_b = Uuid::new_v4();
    let entity_c = Uuid::new_v4();

    insert_test_entity(&backend, &tenant_id, entity_a, "Node", "A", now).await;
    insert_test_entity(&backend, &tenant_id, entity_b, "Node", "B", now).await;
    insert_test_entity(&backend, &tenant_id, entity_c, "Node", "C", now).await;

    insert_test_relationship(&backend, &tenant_id, entity_a, entity_b, "links_to", now).await;
    insert_test_relationship(&backend, &tenant_id, entity_b, entity_c, "links_to", now).await;

    // Query 1-hop from A
    let results = graph
        .get_related(&entity_a.to_string(), None, 1, now)
        .await
        .expect("get_related");

    assert_eq!(results.len(), 1, "Should find exactly 1 entity at 1-hop");
    assert_eq!(results[0].0.name, "B");
    assert_eq!(results[0].1, 1, "Depth should be 1");
    assert_eq!(results[0].2.len(), 2, "Path should contain 2 entities");
}

#[tokio::test]
async fn test_graph_traversal_2_hop() {
    ensure_migrations_run_once().await;

    let tenant_id = unique_tenant_id("traversal-2hop");
    let config = PostgresConfig::new(APP_CONNECTION_STRING);
    let backend = Arc::new(PostgresBackend::new(config).await.expect("create backend"));
    backend
        .set_tenant_context(&tenant_id)
        .await
        .expect("set tenant context");

    let graph = PostgresGraphStorage::new(Arc::clone(&backend));
    let now = Utc::now();

    // Create graph: A -> B -> C
    let entity_a = Uuid::new_v4();
    let entity_b = Uuid::new_v4();
    let entity_c = Uuid::new_v4();

    insert_test_entity(&backend, &tenant_id, entity_a, "Node", "A", now).await;
    insert_test_entity(&backend, &tenant_id, entity_b, "Node", "B", now).await;
    insert_test_entity(&backend, &tenant_id, entity_c, "Node", "C", now).await;

    insert_test_relationship(&backend, &tenant_id, entity_a, entity_b, "links_to", now).await;
    insert_test_relationship(&backend, &tenant_id, entity_b, entity_c, "links_to", now).await;

    // Query 2-hop from A
    let results = graph
        .get_related(&entity_a.to_string(), None, 2, now)
        .await
        .expect("get_related");

    assert_eq!(results.len(), 2, "Should find 2 entities at 1-2 hops");

    // Check we got both B (1-hop) and C (2-hop)
    let names: Vec<&str> = results.iter().map(|(e, _, _)| e.name.as_str()).collect();
    assert!(names.contains(&"B"));
    assert!(names.contains(&"C"));

    // Verify depths
    for (entity, depth, path) in &results {
        if entity.name == "B" {
            assert_eq!(*depth, 1);
            assert_eq!(path.len(), 2);
        } else if entity.name == "C" {
            assert_eq!(*depth, 2);
            assert_eq!(path.len(), 3);
        }
    }
}

#[tokio::test]
async fn test_graph_traversal_4_hop() {
    ensure_migrations_run_once().await;

    let tenant_id = unique_tenant_id("traversal-4hop");
    let config = PostgresConfig::new(APP_CONNECTION_STRING);
    let backend = Arc::new(PostgresBackend::new(config).await.expect("create backend"));
    backend
        .set_tenant_context(&tenant_id)
        .await
        .expect("set tenant context");

    let graph = PostgresGraphStorage::new(Arc::clone(&backend));
    let now = Utc::now();

    // Create chain: A -> B -> C -> D -> E
    let entity_a = Uuid::new_v4();
    let entity_b = Uuid::new_v4();
    let entity_c = Uuid::new_v4();
    let entity_d = Uuid::new_v4();
    let entity_e = Uuid::new_v4();

    insert_test_entity(&backend, &tenant_id, entity_a, "Node", "A", now).await;
    insert_test_entity(&backend, &tenant_id, entity_b, "Node", "B", now).await;
    insert_test_entity(&backend, &tenant_id, entity_c, "Node", "C", now).await;
    insert_test_entity(&backend, &tenant_id, entity_d, "Node", "D", now).await;
    insert_test_entity(&backend, &tenant_id, entity_e, "Node", "E", now).await;

    insert_test_relationship(&backend, &tenant_id, entity_a, entity_b, "links_to", now).await;
    insert_test_relationship(&backend, &tenant_id, entity_b, entity_c, "links_to", now).await;
    insert_test_relationship(&backend, &tenant_id, entity_c, entity_d, "links_to", now).await;
    insert_test_relationship(&backend, &tenant_id, entity_d, entity_e, "links_to", now).await;

    // Query 4-hop from A
    let results = graph
        .get_related(&entity_a.to_string(), None, 4, now)
        .await
        .expect("get_related");

    assert_eq!(results.len(), 4, "Should find 4 entities at 1-4 hops");

    // Verify all entities except A
    let names: Vec<&str> = results.iter().map(|(e, _, _)| e.name.as_str()).collect();
    assert!(names.contains(&"B"));
    assert!(names.contains(&"C"));
    assert!(names.contains(&"D"));
    assert!(names.contains(&"E"));

    // Verify depths
    for (entity, depth, path) in &results {
        match entity.name.as_str() {
            "B" => {
                assert_eq!(*depth, 1);
                assert_eq!(path.len(), 2);
            }
            "C" => {
                assert_eq!(*depth, 2);
                assert_eq!(path.len(), 3);
            }
            "D" => {
                assert_eq!(*depth, 3);
                assert_eq!(path.len(), 4);
            }
            "E" => {
                assert_eq!(*depth, 4);
                assert_eq!(path.len(), 5);
            }
            _ => panic!("Unexpected entity: {}", entity.name),
        }
    }
}

#[tokio::test]
async fn test_graph_traversal_cycle_prevention() {
    ensure_migrations_run_once().await;

    let tenant_id = unique_tenant_id("traversal-cycle");
    let config = PostgresConfig::new(APP_CONNECTION_STRING);
    let backend = Arc::new(PostgresBackend::new(config).await.expect("create backend"));
    backend
        .set_tenant_context(&tenant_id)
        .await
        .expect("set tenant context");

    let graph = PostgresGraphStorage::new(Arc::clone(&backend));
    let now = Utc::now();

    // Create circular graph: A -> B -> C -> A
    let entity_a = Uuid::new_v4();
    let entity_b = Uuid::new_v4();
    let entity_c = Uuid::new_v4();

    insert_test_entity(&backend, &tenant_id, entity_a, "Node", "A", now).await;
    insert_test_entity(&backend, &tenant_id, entity_b, "Node", "B", now).await;
    insert_test_entity(&backend, &tenant_id, entity_c, "Node", "C", now).await;

    insert_test_relationship(&backend, &tenant_id, entity_a, entity_b, "links_to", now).await;
    insert_test_relationship(&backend, &tenant_id, entity_b, entity_c, "links_to", now).await;
    insert_test_relationship(&backend, &tenant_id, entity_c, entity_a, "links_to", now).await; // Cycle!

    // Query 4-hop from A - should not loop infinitely
    let results = graph
        .get_related(&entity_a.to_string(), None, 4, now)
        .await
        .expect("get_related");

    // Should find B and C, but not re-visit A (cycle prevention)
    assert_eq!(
        results.len(),
        2,
        "Should find 2 entities (B, C) without cycles"
    );

    let names: Vec<&str> = results.iter().map(|(e, _, _)| e.name.as_str()).collect();
    assert!(names.contains(&"B"));
    assert!(names.contains(&"C"));
    assert!(!names.contains(&"A"), "Should not re-visit starting node A");

    // Verify paths don't contain duplicates
    for (entity, _depth, path) in &results {
        let unique_ids: std::collections::HashSet<_> = path.iter().collect();
        assert_eq!(
            unique_ids.len(),
            path.len(),
            "Path for {} should not contain duplicate IDs (cycle detected)",
            entity.name
        );
    }
}

#[tokio::test]
async fn test_graph_traversal_relationship_type_filter() {
    ensure_migrations_run_once().await;

    let tenant_id = unique_tenant_id("traversal-filter");
    let config = PostgresConfig::new(APP_CONNECTION_STRING);
    let backend = Arc::new(PostgresBackend::new(config).await.expect("create backend"));
    backend
        .set_tenant_context(&tenant_id)
        .await
        .expect("set tenant context");

    let graph = PostgresGraphStorage::new(Arc::clone(&backend));
    let now = Utc::now();

    // Create graph with mixed relationship types:
    // A --owns--> B --owns--> C
    // A --likes--> D
    let entity_a = Uuid::new_v4();
    let entity_b = Uuid::new_v4();
    let entity_c = Uuid::new_v4();
    let entity_d = Uuid::new_v4();

    insert_test_entity(&backend, &tenant_id, entity_a, "Node", "A", now).await;
    insert_test_entity(&backend, &tenant_id, entity_b, "Node", "B", now).await;
    insert_test_entity(&backend, &tenant_id, entity_c, "Node", "C", now).await;
    insert_test_entity(&backend, &tenant_id, entity_d, "Node", "D", now).await;

    insert_test_relationship(&backend, &tenant_id, entity_a, entity_b, "owns", now).await;
    insert_test_relationship(&backend, &tenant_id, entity_b, entity_c, "owns", now).await;
    insert_test_relationship(&backend, &tenant_id, entity_a, entity_d, "likes", now).await;

    // Query with "owns" filter - should only find B and C
    let results = graph
        .get_related(&entity_a.to_string(), Some("owns"), 2, now)
        .await
        .expect("get_related");

    assert_eq!(
        results.len(),
        2,
        "Should find 2 entities via 'owns' relationships"
    );

    let names: Vec<&str> = results.iter().map(|(e, _, _)| e.name.as_str()).collect();
    assert!(names.contains(&"B"));
    assert!(names.contains(&"C"));
    assert!(
        !names.contains(&"D"),
        "Should not find D (connected via 'likes')"
    );

    // Query with "likes" filter - should only find D
    let results = graph
        .get_related(&entity_a.to_string(), Some("likes"), 2, now)
        .await
        .expect("get_related");

    assert_eq!(
        results.len(),
        1,
        "Should find 1 entity via 'likes' relationships"
    );
    assert_eq!(results[0].0.name, "D");
}

#[tokio::test]
async fn test_graph_traversal_path_tracking() {
    ensure_migrations_run_once().await;

    let tenant_id = unique_tenant_id("traversal-path");
    let config = PostgresConfig::new(APP_CONNECTION_STRING);
    let backend = Arc::new(PostgresBackend::new(config).await.expect("create backend"));
    backend
        .set_tenant_context(&tenant_id)
        .await
        .expect("set tenant context");

    let graph = PostgresGraphStorage::new(Arc::clone(&backend));
    let now = Utc::now();

    // Create diamond graph to test multiple paths:
    //     A
    //    / \
    //   B   C
    //    \ /
    //     D
    let entity_a = Uuid::new_v4();
    let entity_b = Uuid::new_v4();
    let entity_c = Uuid::new_v4();
    let entity_d = Uuid::new_v4();

    insert_test_entity(&backend, &tenant_id, entity_a, "Node", "A", now).await;
    insert_test_entity(&backend, &tenant_id, entity_b, "Node", "B", now).await;
    insert_test_entity(&backend, &tenant_id, entity_c, "Node", "C", now).await;
    insert_test_entity(&backend, &tenant_id, entity_d, "Node", "D", now).await;

    insert_test_relationship(&backend, &tenant_id, entity_a, entity_b, "links_to", now).await;
    insert_test_relationship(&backend, &tenant_id, entity_a, entity_c, "links_to", now).await;
    insert_test_relationship(&backend, &tenant_id, entity_b, entity_d, "links_to", now).await;
    insert_test_relationship(&backend, &tenant_id, entity_c, entity_d, "links_to", now).await;

    // Query 2-hop from A
    let results = graph
        .get_related(&entity_a.to_string(), None, 2, now)
        .await
        .expect("get_related");

    // Should find B, C (1-hop) and D (2-hop via shortest path)
    assert_eq!(results.len(), 3, "Should find B, C, D");

    // Find D's result
    let d_result = results
        .iter()
        .find(|(e, _, _)| e.name == "D")
        .expect("Should find entity D");

    // D should be at depth 2 (shortest path)
    assert_eq!(d_result.1, 2, "D should be at depth 2");

    // Path should be A -> B/C -> D (3 entities)
    assert_eq!(d_result.2.len(), 3, "Path to D should have 3 entities");

    // First entity in path should be A
    assert_eq!(
        d_result.2[0],
        entity_a.to_string(),
        "Path should start with A"
    );

    // Last entity in path should be D
    assert_eq!(
        d_result.2[2],
        entity_d.to_string(),
        "Path should end with D"
    );
}

#[tokio::test]
async fn test_graph_traversal_performance() {
    ensure_migrations_run_once().await;

    let tenant_id = unique_tenant_id("traversal-perf");
    let config = PostgresConfig::new(APP_CONNECTION_STRING);
    let backend = Arc::new(PostgresBackend::new(config).await.expect("create backend"));
    backend
        .set_tenant_context(&tenant_id)
        .await
        .expect("set tenant context");

    let graph = PostgresGraphStorage::new(Arc::clone(&backend));
    let now = Utc::now();

    // Create a larger graph: 1 root -> 5 children -> 10 grandchildren
    let root = Uuid::new_v4();
    insert_test_entity(&backend, &tenant_id, root, "Root", "Root", now).await;

    let mut children = Vec::new();
    for i in 0..5 {
        let child = Uuid::new_v4();
        insert_test_entity(
            &backend,
            &tenant_id,
            child,
            "Child",
            &format!("Child{}", i),
            now,
        )
        .await;
        insert_test_relationship(&backend, &tenant_id, root, child, "parent_of", now).await;
        children.push(child);
    }

    for (i, child) in children.iter().enumerate() {
        for j in 0..2 {
            let grandchild = Uuid::new_v4();
            insert_test_entity(
                &backend,
                &tenant_id,
                grandchild,
                "GrandChild",
                &format!("GrandChild{}_{}", i, j),
                now,
            )
            .await;
            insert_test_relationship(&backend, &tenant_id, *child, grandchild, "parent_of", now)
                .await;
        }
    }

    // Benchmark 2-hop traversal (should find 5 children + 10 grandchildren = 15 entities)
    let start = std::time::Instant::now();
    let results = graph
        .get_related(&root.to_string(), None, 2, now)
        .await
        .expect("get_related");
    let duration = start.elapsed();

    assert_eq!(
        results.len(),
        15,
        "Should find 15 entities (5 children + 10 grandchildren)"
    );

    // Performance target: <50ms
    assert!(
        duration.as_millis() < 50,
        "Graph traversal should complete in <50ms, took {}ms",
        duration.as_millis()
    );

    println!(
        "Graph traversal performance: {}ms for 15 entities across 2 hops",
        duration.as_millis()
    );
}

#[tokio::test]
async fn test_graph_traversal_tenant_isolation() {
    ensure_migrations_run_once().await;

    let tenant_a = unique_tenant_id("traversal-tenant-a");
    let tenant_b = unique_tenant_id("traversal-tenant-b");

    let config = PostgresConfig::new(APP_CONNECTION_STRING);

    // Setup graph for tenant A
    let backend_a = Arc::new(
        PostgresBackend::new(config.clone())
            .await
            .expect("create backend"),
    );
    backend_a
        .set_tenant_context(&tenant_a)
        .await
        .expect("set tenant context");
    let graph_a = PostgresGraphStorage::new(Arc::clone(&backend_a));
    let now = Utc::now();

    let entity_a1 = Uuid::new_v4();
    let entity_a2 = Uuid::new_v4();
    insert_test_entity(&backend_a, &tenant_a, entity_a1, "Node", "A1", now).await;
    insert_test_entity(&backend_a, &tenant_a, entity_a2, "Node", "A2", now).await;
    insert_test_relationship(&backend_a, &tenant_a, entity_a1, entity_a2, "links_to", now).await;

    // Setup graph for tenant B
    let backend_b = Arc::new(PostgresBackend::new(config).await.expect("create backend"));
    backend_b
        .set_tenant_context(&tenant_b)
        .await
        .expect("set tenant context");
    let graph_b = PostgresGraphStorage::new(Arc::clone(&backend_b));

    let entity_b1 = Uuid::new_v4();
    let entity_b2 = Uuid::new_v4();
    insert_test_entity(&backend_b, &tenant_b, entity_b1, "Node", "B1", now).await;
    insert_test_entity(&backend_b, &tenant_b, entity_b2, "Node", "B2", now).await;
    insert_test_relationship(&backend_b, &tenant_b, entity_b1, entity_b2, "links_to", now).await;

    // Query from tenant A - should only see tenant A entities
    let results_a = graph_a
        .get_related(&entity_a1.to_string(), None, 2, now)
        .await
        .expect("get_related");

    assert_eq!(results_a.len(), 1, "Tenant A should only see A2");
    assert_eq!(results_a[0].0.name, "A2");

    // Query from tenant B - should only see tenant B entities
    let results_b = graph_b
        .get_related(&entity_b1.to_string(), None, 2, now)
        .await
        .expect("get_related");

    assert_eq!(results_b.len(), 1, "Tenant B should only see B2");
    assert_eq!(results_b[0].0.name, "B2");
}

// --- New tests for KnowledgeGraph::traverse() method (Task 13c.2.8.4) ---

use llmspell_graph::traits::KnowledgeGraph;

#[tokio::test]
async fn test_kg_traverse_1_hop() {
    ensure_migrations_run_once().await;

    let tenant_id = unique_tenant_id("kg-traverse-1hop");
    let config = PostgresConfig::new(APP_CONNECTION_STRING);
    let backend = Arc::new(PostgresBackend::new(config).await.expect("create backend"));
    backend
        .set_tenant_context(&tenant_id)
        .await
        .expect("set tenant context");

    let graph = PostgresGraphStorage::new(Arc::clone(&backend));
    let now = Utc::now();

    // Create graph: A -> B, A -> C
    let entity_a = Uuid::new_v4();
    let entity_b = Uuid::new_v4();
    let entity_c = Uuid::new_v4();

    insert_test_entity(&backend, &tenant_id, entity_a, "node", "A", now).await;
    insert_test_entity(&backend, &tenant_id, entity_b, "node", "B", now).await;
    insert_test_entity(&backend, &tenant_id, entity_c, "node", "C", now).await;

    insert_test_relationship(&backend, &tenant_id, entity_a, entity_b, "knows", now).await;
    insert_test_relationship(&backend, &tenant_id, entity_a, entity_c, "knows", now).await;

    // Traverse 1 hop from A using KnowledgeGraph trait
    let results = graph
        .traverse(&entity_a.to_string(), None, 1, None)
        .await
        .expect("traverse");

    assert_eq!(results.len(), 2);
    assert!(results.iter().any(|(e, d, _)| e.name == "B" && *d == 1));
    assert!(results.iter().any(|(e, d, _)| e.name == "C" && *d == 1));

    // Verify paths contain starting entity
    for (_, _, path_json) in &results {
        assert!(path_json.contains(&entity_a.to_string()));
    }
}

#[tokio::test]
async fn test_kg_traverse_4_hops_linear() {
    ensure_migrations_run_once().await;

    let tenant_id = unique_tenant_id("kg-traverse-4hop");
    let config = PostgresConfig::new(APP_CONNECTION_STRING);
    let backend = Arc::new(PostgresBackend::new(config).await.expect("create backend"));
    backend
        .set_tenant_context(&tenant_id)
        .await
        .expect("set tenant context");

    let graph = PostgresGraphStorage::new(Arc::clone(&backend));
    let now = Utc::now();

    // Create linear graph: A -> B -> C -> D -> E
    let entity_a = Uuid::new_v4();
    let entity_b = Uuid::new_v4();
    let entity_c = Uuid::new_v4();
    let entity_d = Uuid::new_v4();
    let entity_e = Uuid::new_v4();

    insert_test_entity(&backend, &tenant_id, entity_a, "node", "A", now).await;
    insert_test_entity(&backend, &tenant_id, entity_b, "node", "B", now).await;
    insert_test_entity(&backend, &tenant_id, entity_c, "node", "C", now).await;
    insert_test_entity(&backend, &tenant_id, entity_d, "node", "D", now).await;
    insert_test_entity(&backend, &tenant_id, entity_e, "node", "E", now).await;

    insert_test_relationship(&backend, &tenant_id, entity_a, entity_b, "next", now).await;
    insert_test_relationship(&backend, &tenant_id, entity_b, entity_c, "next", now).await;
    insert_test_relationship(&backend, &tenant_id, entity_c, entity_d, "next", now).await;
    insert_test_relationship(&backend, &tenant_id, entity_d, entity_e, "next", now).await;

    // Traverse 4 hops from A
    let results = graph
        .traverse(&entity_a.to_string(), None, 4, None)
        .await
        .expect("traverse");

    assert_eq!(results.len(), 4);
    assert!(results.iter().any(|(e, d, _)| e.name == "B" && *d == 1));
    assert!(results.iter().any(|(e, d, _)| e.name == "C" && *d == 2));
    assert!(results.iter().any(|(e, d, _)| e.name == "D" && *d == 3));
    assert!(results.iter().any(|(e, d, _)| e.name == "E" && *d == 4));

    // Verify path grows with depth
    let e_result = results
        .iter()
        .find(|(e, _, _)| e.name == "E")
        .expect("Should find E");
    let path: Vec<String> = serde_json::from_str(&e_result.2).unwrap();
    assert_eq!(path.len(), 5); // A, B, C, D, E
}

#[tokio::test]
async fn test_kg_traverse_with_cycles() {
    ensure_migrations_run_once().await;

    let tenant_id = unique_tenant_id("kg-traverse-cycle");
    let config = PostgresConfig::new(APP_CONNECTION_STRING);
    let backend = Arc::new(PostgresBackend::new(config).await.expect("create backend"));
    backend
        .set_tenant_context(&tenant_id)
        .await
        .expect("set tenant context");

    let graph = PostgresGraphStorage::new(Arc::clone(&backend));
    let now = Utc::now();

    // Create cyclic graph: A -> B -> C -> A
    let entity_a = Uuid::new_v4();
    let entity_b = Uuid::new_v4();
    let entity_c = Uuid::new_v4();

    insert_test_entity(&backend, &tenant_id, entity_a, "node", "A", now).await;
    insert_test_entity(&backend, &tenant_id, entity_b, "node", "B", now).await;
    insert_test_entity(&backend, &tenant_id, entity_c, "node", "C", now).await;

    insert_test_relationship(&backend, &tenant_id, entity_a, entity_b, "next", now).await;
    insert_test_relationship(&backend, &tenant_id, entity_b, entity_c, "next", now).await;
    insert_test_relationship(&backend, &tenant_id, entity_c, entity_a, "next", now).await;

    // Traverse 5 hops (should not revisit A due to cycle prevention)
    let results = graph
        .traverse(&entity_a.to_string(), None, 5, None)
        .await
        .expect("traverse");

    // Should find B and C only (A is excluded via cycle prevention)
    assert_eq!(results.len(), 2);
    assert!(results.iter().any(|(e, d, _)| e.name == "B" && *d == 1));
    assert!(results.iter().any(|(e, d, _)| e.name == "C" && *d == 2));
    assert!(!results.iter().any(|(e, _, _)| e.name == "A")); // A not revisited
}

#[tokio::test]
async fn test_kg_traverse_relationship_filter() {
    ensure_migrations_run_once().await;

    let tenant_id = unique_tenant_id("kg-traverse-filter");
    let config = PostgresConfig::new(APP_CONNECTION_STRING);
    let backend = Arc::new(PostgresBackend::new(config).await.expect("create backend"));
    backend
        .set_tenant_context(&tenant_id)
        .await
        .expect("set tenant context");

    let graph = PostgresGraphStorage::new(Arc::clone(&backend));
    let now = Utc::now();

    // Create multi-type graph: A -knows-> B, A -works_with-> C
    let entity_a = Uuid::new_v4();
    let entity_b = Uuid::new_v4();
    let entity_c = Uuid::new_v4();

    insert_test_entity(&backend, &tenant_id, entity_a, "person", "A", now).await;
    insert_test_entity(&backend, &tenant_id, entity_b, "person", "B", now).await;
    insert_test_entity(&backend, &tenant_id, entity_c, "person", "C", now).await;

    insert_test_relationship(&backend, &tenant_id, entity_a, entity_b, "knows", now).await;
    insert_test_relationship(&backend, &tenant_id, entity_a, entity_c, "works_with", now).await;

    // Traverse with "knows" filter (should only find B)
    let results = graph
        .traverse(&entity_a.to_string(), Some("knows"), 2, None)
        .await
        .expect("traverse");

    assert_eq!(results.len(), 1);
    assert_eq!(results[0].0.name, "B");

    // Traverse with "works_with" filter (should only find C)
    let results = graph
        .traverse(&entity_a.to_string(), Some("works_with"), 2, None)
        .await
        .expect("traverse");

    assert_eq!(results.len(), 1);
    assert_eq!(results[0].0.name, "C");

    // Traverse with no filter (should find both)
    let results = graph
        .traverse(&entity_a.to_string(), None, 2, None)
        .await
        .expect("traverse");

    assert_eq!(results.len(), 2);
}

#[tokio::test]
async fn test_kg_traverse_temporal() {
    ensure_migrations_run_once().await;

    let tenant_id = unique_tenant_id("kg-traverse-temporal");
    let config = PostgresConfig::new(APP_CONNECTION_STRING);
    let backend = Arc::new(PostgresBackend::new(config).await.expect("create backend"));
    backend
        .set_tenant_context(&tenant_id)
        .await
        .expect("set tenant context");

    let graph = PostgresGraphStorage::new(Arc::clone(&backend));

    let past = Utc::now() - chrono::Duration::days(10);
    let present = Utc::now();
    let future = Utc::now() + chrono::Duration::days(10);

    // Create entity A (exists now)
    let entity_a = Uuid::new_v4();
    insert_test_entity(&backend, &tenant_id, entity_a, "node", "A", present).await;

    // Create entity B with past event time
    let entity_b = Uuid::new_v4();
    insert_test_entity(&backend, &tenant_id, entity_b, "node", "B", past).await;

    // Create entity C with future event time
    let entity_c = Uuid::new_v4();
    insert_test_entity(&backend, &tenant_id, entity_c, "node", "C", future).await;

    // Add relationships
    insert_test_relationship(&backend, &tenant_id, entity_a, entity_b, "links", present).await;
    insert_test_relationship(&backend, &tenant_id, entity_a, entity_c, "links", future).await;

    // Query at present time (should see A and B, not C)
    let results = graph
        .traverse(&entity_a.to_string(), None, 2, Some(present))
        .await
        .expect("traverse");

    assert_eq!(results.len(), 1);
    assert_eq!(results[0].0.name, "B");

    // Query at future time (should see A, B, and C)
    let results = graph
        .traverse(&entity_a.to_string(), None, 2, Some(future))
        .await
        .expect("traverse");

    assert_eq!(results.len(), 2);
    assert!(results.iter().any(|(e, _, _)| e.name == "B"));
    assert!(results.iter().any(|(e, _, _)| e.name == "C"));
}
