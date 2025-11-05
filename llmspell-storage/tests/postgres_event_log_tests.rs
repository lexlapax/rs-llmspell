//! Integration tests for PostgreSQL Event Log Storage (Phase 13b.11.2)
//!
//! Tests PostgresEventLogStorage backend operations:
//! - Event storage and retrieval
//! - Pattern matching queries
//! - Time range queries with partition pruning
//! - Correlation ID queries
//! - Storage statistics

#![cfg(feature = "postgres")]

use chrono::Utc;
use llmspell_storage::backends::postgres::{PostgresBackend, PostgresConfig, PostgresEventLogStorage};
use serde_json::Value;
use std::sync::Arc;
use tokio::sync::OnceCell;
use uuid::Uuid;

// Admin connection for migrations (llmspell user has CREATE TABLE privileges)
const ADMIN_CONNECTION_STRING: &str =
    "postgresql://llmspell:llmspell_dev_pass@localhost:5432/llmspell_dev";

// Application connection for queries (llmspell_app enforces RLS, no schema modification)
const APP_CONNECTION_STRING: &str =
    "postgresql://llmspell_app:llmspell_app_pass@localhost:5432/llmspell_dev";

static MIGRATION_INIT: OnceCell<()> = OnceCell::const_new();

/// Ensure migrations run once before all tests (uses admin user for DDL operations)
async fn ensure_migrations_run_once() {
    MIGRATION_INIT
        .get_or_init(|| async {
            let config = PostgresConfig::new(ADMIN_CONNECTION_STRING);
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

fn create_test_event(event_type: &str) -> Value {
    let event_id = Uuid::new_v4();
    let correlation_id = Uuid::new_v4();
    let timestamp = Utc::now();
    let sequence = chrono::Utc::now().timestamp_millis();

    serde_json::json!({
        "id": event_id.to_string(),
        "event_type": event_type,
        "data": {"test": "data"},
        "language": "rust",
        "timestamp": timestamp.to_rfc3339(),
        "sequence": sequence,
        "metadata": {
            "correlation_id": correlation_id.to_string(),
            "source": null,
            "target": null,
            "tags": [],
            "priority": 0,
            "ttl": null
        },
        "schema_version": "1.0"
    })
}

#[tokio::test]
async fn test_event_log_storage_basic_operations() {
    ensure_migrations_run_once().await;

    let config = PostgresConfig::new(APP_CONNECTION_STRING);
    let backend = Arc::new(PostgresBackend::new(config).await.unwrap());
    backend
        .set_tenant_context(&format!("test_event_log_{}", Uuid::new_v4()))
        .await
        .unwrap();

    let storage = PostgresEventLogStorage::new(backend.clone());

    let event = create_test_event("test.event");
    let event_id = Uuid::parse_str(event["id"].as_str().unwrap()).unwrap();
    let correlation_id =
        Uuid::parse_str(event["metadata"]["correlation_id"].as_str().unwrap()).unwrap();

    // Store event
    storage.store_event(&event).await.unwrap();

    // Retrieve by correlation ID
    let events = storage
        .get_events_by_correlation_id(correlation_id)
        .await
        .unwrap();
    assert_eq!(events.len(), 1);
    assert_eq!(events[0]["id"].as_str().unwrap(), event_id.to_string());
    assert_eq!(events[0]["event_type"].as_str().unwrap(), "test.event");

    // Cleanup
    let client = backend.get_client().await.unwrap();
    client
        .execute(
            "DELETE FROM llmspell.event_log WHERE event_id = $1",
            &[&event_id],
        )
        .await
        .unwrap();
}

#[tokio::test]
async fn test_event_log_pattern_matching() {
    ensure_migrations_run_once().await;

    let config = PostgresConfig::new(APP_CONNECTION_STRING);
    let backend = Arc::new(PostgresBackend::new(config).await.unwrap());
    backend
        .set_tenant_context(&format!("test_pattern_{}", Uuid::new_v4()))
        .await
        .unwrap();

    let storage = PostgresEventLogStorage::new(backend.clone());

    let event1 = create_test_event("agent.state_changed");
    let event2 = create_test_event("agent.action_taken");
    let event3 = create_test_event("system.startup");

    storage.store_event(&event1).await.unwrap();
    storage.store_event(&event2).await.unwrap();
    storage.store_event(&event3).await.unwrap();

    // Pattern: agent.*
    let events = storage.get_events_by_pattern("agent.*").await.unwrap();
    assert_eq!(events.len(), 2);

    // Pattern: system.*
    let events = storage.get_events_by_pattern("system.*").await.unwrap();
    assert_eq!(events.len(), 1);

    // Cleanup
    let client = backend.get_client().await.unwrap();
    for event in [event1, event2, event3] {
        let event_id = Uuid::parse_str(event["id"].as_str().unwrap()).unwrap();
        client
            .execute(
                "DELETE FROM llmspell.event_log WHERE event_id = $1",
                &[&event_id],
            )
            .await
            .unwrap();
    }
}

#[tokio::test]
async fn test_event_log_time_range_query() {
    ensure_migrations_run_once().await;

    let config = PostgresConfig::new(APP_CONNECTION_STRING);
    let backend = Arc::new(PostgresBackend::new(config).await.unwrap());
    backend
        .set_tenant_context(&format!("test_time_range_{}", Uuid::new_v4()))
        .await
        .unwrap();

    let storage = PostgresEventLogStorage::new(backend.clone());

    let event1 = create_test_event("test.event1");
    let event2 = create_test_event("test.event2");

    let start = Utc::now() - chrono::Duration::seconds(10);
    storage.store_event(&event1).await.unwrap();
    storage.store_event(&event2).await.unwrap();
    let end = Utc::now() + chrono::Duration::seconds(10);

    let events = storage.get_events_by_time_range(start, end).await.unwrap();
    assert!(events.len() >= 2);

    // Cleanup
    let client = backend.get_client().await.unwrap();
    for event in [event1, event2] {
        let event_id = Uuid::parse_str(event["id"].as_str().unwrap()).unwrap();
        client
            .execute(
                "DELETE FROM llmspell.event_log WHERE event_id = $1",
                &[&event_id],
            )
            .await
            .unwrap();
    }
}

#[tokio::test]
async fn test_event_log_storage_stats() {
    ensure_migrations_run_once().await;

    let config = PostgresConfig::new(APP_CONNECTION_STRING);
    let backend = Arc::new(PostgresBackend::new(config).await.unwrap());
    backend
        .set_tenant_context(&format!("test_stats_{}", Uuid::new_v4()))
        .await
        .unwrap();

    let storage = PostgresEventLogStorage::new(backend.clone());

    let event1 = create_test_event("test.event1");
    let event2 = create_test_event("test.event2");

    storage.store_event(&event1).await.unwrap();
    storage.store_event(&event2).await.unwrap();

    let stats = storage.get_storage_stats().await.unwrap();
    assert_eq!(stats.total_events, 2);
    assert!(stats.oldest_event.is_some());
    assert!(stats.newest_event.is_some());
    assert_eq!(stats.events_by_type["test.event1"], 1);
    assert_eq!(stats.events_by_type["test.event2"], 1);

    // Cleanup
    let client = backend.get_client().await.unwrap();
    for event in [event1, event2] {
        let event_id = Uuid::parse_str(event["id"].as_str().unwrap()).unwrap();
        client
            .execute(
                "DELETE FROM llmspell.event_log WHERE event_id = $1",
                &[&event_id],
            )
            .await
            .unwrap();
    }
}
