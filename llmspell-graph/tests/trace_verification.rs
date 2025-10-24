//! Trace verification tests for llmspell-graph
//!
//! Verifies that tracing instrumentation produces correct log output.

use llmspell_graph::storage::surrealdb::SurrealDBBackend;
use llmspell_graph::traits::KnowledgeGraph;
use llmspell_graph::types::{Entity, TemporalQuery};
use serde_json::json;
use std::sync::{Arc, Mutex};
use tempfile::TempDir;
use tracing::Level;
use tracing_subscriber::layer::{Context, SubscriberExt};
use tracing_subscriber::Layer;

/// Captured log event
#[derive(Debug, Clone)]
struct LogEvent {
    level: Level,
    message: String,
}

/// Test layer that captures log events
struct TestLayer {
    events: Arc<Mutex<Vec<LogEvent>>>,
}

impl TestLayer {
    fn new() -> (Self, Arc<Mutex<Vec<LogEvent>>>) {
        let events = Arc::new(Mutex::new(Vec::new()));
        (
            Self {
                events: Arc::clone(&events),
            },
            events,
        )
    }
}

impl<S> Layer<S> for TestLayer
where
    S: tracing::Subscriber,
{
    fn on_event(&self, event: &tracing::Event<'_>, _ctx: Context<'_, S>) {
        let mut visitor = MessageVisitor::default();
        event.record(&mut visitor);

        if let Some(message) = visitor.message {
            let log_event = LogEvent {
                level: *event.metadata().level(),
                message,
            };
            self.events.lock().unwrap().push(log_event);
        }
    }
}

#[derive(Default)]
struct MessageVisitor {
    message: Option<String>,
}

impl tracing::field::Visit for MessageVisitor {
    fn record_debug(&mut self, field: &tracing::field::Field, value: &dyn std::fmt::Debug) {
        if field.name() == "message" {
            self.message = Some(format!("{value:?}"));
        }
    }

    fn record_str(&mut self, field: &tracing::field::Field, value: &str) {
        if field.name() == "message" {
            self.message = Some(value.to_string());
        }
    }
}

fn setup_tracing() -> (Arc<Mutex<Vec<LogEvent>>>, tracing::subscriber::DefaultGuard) {
    let (layer, events) = TestLayer::new();
    let subscriber = tracing_subscriber::registry().with(layer);
    let guard = tracing::subscriber::set_default(subscriber);
    (events, guard)
}

#[tokio::test]
async fn test_surrealdb_init_produces_info_log() {
    let (events, _guard) = setup_tracing();

    let temp_dir = TempDir::new().unwrap();
    let _backend = SurrealDBBackend::new(temp_dir.path()).await.unwrap();

    let info_logs: Vec<_> = events
        .lock()
        .unwrap()
        .iter()
        .filter(|e| e.level == Level::INFO)
        .cloned()
        .collect();

    assert!(
        info_logs
            .iter()
            .any(|e| e.message.contains("Initializing SurrealDB backend")),
        "Expected info! log for SurrealDB initialization, got: {info_logs:?}"
    );
}

#[tokio::test]
async fn test_entity_creation_produces_debug_log() {
    let (events, _guard) = setup_tracing();

    let temp_dir = TempDir::new().unwrap();
    let backend = SurrealDBBackend::new(temp_dir.path()).await.unwrap();

    let entity = Entity::new(
        "Rust".to_string(),
        "programming_language".to_string(),
        json!({"paradigm": "multi-paradigm"}),
    );

    backend.add_entity(entity).await.unwrap();

    let debug_logs: Vec<_> = events
        .lock()
        .unwrap()
        .iter()
        .filter(|e| e.level == Level::DEBUG)
        .cloned()
        .collect();

    assert!(
        debug_logs
            .iter()
            .any(|e| e.message.contains("entity_type") || e.message.contains("Adding entity")),
        "Expected debug! log for entity creation with entity_type, got: {debug_logs:?}"
    );
}

#[tokio::test]
async fn test_temporal_query_produces_trace_log() {
    let (events, _guard) = setup_tracing();

    let temp_dir = TempDir::new().unwrap();
    let backend = SurrealDBBackend::new(temp_dir.path()).await.unwrap();

    // Add an entity first
    let entity = Entity::new("Python".to_string(), "language".to_string(), json!({}));
    backend.add_entity(entity).await.unwrap();

    // Now query
    let query = TemporalQuery::new().with_entity_type("language".to_string());
    let _ = backend.query_temporal(query).await;

    let trace_logs: Vec<_> = events
        .lock()
        .unwrap()
        .iter()
        .filter(|e| e.level == Level::TRACE)
        .cloned()
        .collect();

    assert!(
        trace_logs
            .iter()
            .any(|e| e.message.contains("query") || e.message.contains("Temporal")),
        "Expected trace! log for temporal query, got: {trace_logs:?}"
    );
}

#[tokio::test]
async fn test_connection_failure_produces_error_log() {
    let (events, _guard) = setup_tracing();

    // Try to create backend with invalid path (permission denied or non-existent parent)
    let result = SurrealDBBackend::new("/nonexistent/invalid/path/db").await;

    assert!(result.is_err(), "Expected connection to fail");

    let error_logs: Vec<_> = events
        .lock()
        .unwrap()
        .iter()
        .filter(|e| e.level == Level::ERROR)
        .cloned()
        .collect();

    assert!(
        error_logs
            .iter()
            .any(|e| e.message.contains("Failed") || e.message.contains("error")),
        "Expected error! log for connection failure, got: {error_logs:?}"
    );
}
