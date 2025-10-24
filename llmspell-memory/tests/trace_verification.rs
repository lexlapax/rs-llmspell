//! Trace verification tests for llmspell-memory
//!
//! Verifies that tracing instrumentation produces correct log output.

use llmspell_memory::consolidation::metrics::ConsolidationMetrics;
use llmspell_memory::consolidation::prompt_schema::ConsolidationResponse;
use llmspell_memory::consolidation::PromptVersion;
use llmspell_memory::manager::DefaultMemoryManager;
use llmspell_memory::prelude::*;
use llmspell_memory::types::{ConsolidationResult, EpisodicEntry};
use std::sync::{Arc, Mutex};
use std::time::Duration;
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
    fn on_event(
        &self,
        event: &tracing::Event<'_>,
        _ctx: Context<'_, S>,
    ) {
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
async fn test_manager_init_produces_info_log() {
    let (events, _guard) = setup_tracing();

    let _manager = DefaultMemoryManager::new_in_memory().await.unwrap();

    let info_logs: Vec<_> = {
        let logs = events.lock().unwrap();
        logs.iter()
            .filter(|e| e.level == Level::INFO)
            .cloned()
            .collect()
    };

    assert!(
        info_logs.iter().any(|e| e.message.contains("Initializing DefaultMemoryManager") || e.message.contains("in-memory")),
        "Expected info! log for manager initialization, got: {info_logs:?}"
    );
}

#[tokio::test]
async fn test_episodic_add_produces_info_log() {
    let (events, _guard) = setup_tracing();

    let manager = DefaultMemoryManager::new_in_memory().await.unwrap();
    let entry = EpisodicEntry::new(
        "test-session".to_string(),
        "user".to_string(),
        "Hello world".to_string(),
    );

    manager.episodic().add(entry).await.unwrap();

    let info_logs: Vec<_> = {
        let logs = events.lock().unwrap();
        logs.iter()
            .filter(|e| e.level == Level::INFO)
            .cloned()
            .collect()
    };

    assert!(
        info_logs.iter().any(|e| e.message.contains("Adding episodic entry") && e.message.contains("session_id")),
        "Expected info! log for episodic add with session_id, got: {info_logs:?}"
    );
}

#[tokio::test]
async fn test_vector_search_produces_debug_log() {
    let (events, _guard) = setup_tracing();

    let manager = DefaultMemoryManager::new_in_memory().await.unwrap();

    // Add an entry first
    let entry = EpisodicEntry::new(
        "test-session".to_string(),
        "user".to_string(),
        "Rust programming language".to_string(),
    );
    manager.episodic().add(entry).await.unwrap();

    // Now search
    manager.episodic().search("Rust", 5).await.unwrap();

    let debug_logs: Vec<_> = {
        let logs = events.lock().unwrap();
        logs.iter()
            .filter(|e| e.level == Level::DEBUG)
            .cloned()
            .collect()
    };

    assert!(
        debug_logs.iter().any(|e| e.message.contains("Searching episodic memory") && e.message.contains("top_k")),
        "Expected debug! log for vector search with params, got: {debug_logs:?}"
    );
}

#[tokio::test]
async fn test_consolidation_metrics_produces_info_log() {
    let (events, _guard) = setup_tracing();

    let metrics = ConsolidationMetrics::new();

    let result = ConsolidationResult {
        entries_processed: 10,
        entities_added: 5,
        entities_updated: 2,
        entities_deleted: 0,
        entries_skipped: 3,
        entries_failed: 0,
        duration_ms: 100,
    };

    metrics.record_consolidation(
        &result,
        &[],
        Duration::from_millis(100),
        PromptVersion::V1,
        true,
        None,
        None,
        &[],
    ).await;

    let info_logs: Vec<_> = {
        let logs = events.lock().unwrap();
        logs.iter()
            .filter(|e| e.level == Level::INFO)
            .cloned()
            .collect()
    };

    assert!(
        info_logs.iter().any(|e| e.message.contains("Recording consolidation") && e.message.contains("entries_processed")),
        "Expected info! log for consolidation metrics, got: {info_logs:?}"
    );
}

#[tokio::test]
async fn test_json_parse_error_produces_warn_log() {
    let (events, _guard) = setup_tracing();

    // Try to parse invalid JSON
    let invalid_json = "{invalid json";
    let _result = ConsolidationResponse::from_json(invalid_json);

    let warn_logs: Vec<_> = {
        let logs = events.lock().unwrap();
        logs.iter()
            .filter(|e| e.level == Level::WARN)
            .cloned()
            .collect()
    };

    assert!(
        warn_logs.iter().any(|e| e.message.contains("Full JSON parsing failed") || e.message.contains("Partial parse failed")),
        "Expected warn! log for JSON parse error, got: {warn_logs:?}"
    );
}
