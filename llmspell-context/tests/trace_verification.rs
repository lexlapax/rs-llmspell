//! Trace verification tests for llmspell-context
//!
//! Verifies that tracing instrumentation produces correct log output.

use llmspell_context::query::analyzer::RegexQueryAnalyzer;
use llmspell_context::retrieval::strategy::StrategySelector;
use llmspell_context::traits::QueryAnalyzer;
use llmspell_context::types::{QueryIntent, QueryUnderstanding, RetrievalStrategy};
use std::sync::{Arc, Mutex};
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
async fn test_query_analysis_produces_debug_log() {
    let (events, _guard) = setup_tracing();

    let analyzer = RegexQueryAnalyzer::new();
    let _ = analyzer.understand("How do I use HashMap?").await.unwrap();

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
            .any(|e| e.message.contains("Intent classified") || e.message.contains("Extracted")),
        "Expected debug! log for query analysis with intent/extraction info, got: {debug_logs:?}"
    );
}

#[tokio::test]
async fn test_strategy_selection_produces_info_log() {
    let (events, _guard) = setup_tracing();

    let selector = StrategySelector::new();
    let understanding = QueryUnderstanding {
        intent: QueryIntent::HowTo,
        entities: vec![],
        keywords: vec!["implement".to_string()],
    };

    let strategy = selector.select(&understanding);

    assert_eq!(strategy, RetrievalStrategy::Episodic);

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
            .any(|e| e.message.contains("Selecting retrieval strategy")
                && e.message.contains("intent")),
        "Expected info! log for strategy selection with intent, got: {info_logs:?}"
    );
}
