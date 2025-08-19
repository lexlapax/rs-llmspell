// ABOUTME: Test utilities for event system testing including event creation and fixtures
// ABOUTME: Provides reusable helpers for event bus, correlation, and stream tests

use async_trait::async_trait;
use llmspell_core::{
    traits::event::{EventConfig, EventData, EventEmitter},
    Result,
};
use llmspell_events::{
    bus::EventBus, correlation::EventCorrelationTracker, EventMetadata, Language, UniversalEvent,
};
use serde_json::Value;
use std::sync::{Arc, RwLock};
use uuid::Uuid;

/// Create a test event with default values
pub fn create_test_event(event_type: &str) -> UniversalEvent {
    UniversalEvent::new(event_type, Value::Null, Language::Rust)
}

/// Create a test event with data
pub fn create_test_event_with_data(event_type: &str, data: Value) -> UniversalEvent {
    UniversalEvent::new(event_type, data, Language::Rust)
}

/// Create a test event with correlation ID
pub fn create_test_event_with_correlation(
    event_type: &str,
    correlation_id: Uuid,
) -> UniversalEvent {
    UniversalEvent::new(event_type, Value::Null, Language::Rust).with_correlation_id(correlation_id)
}

/// Create a test event with custom language
pub fn create_test_event_with_language(
    event_type: &str,
    data: Value,
    language: Language,
) -> UniversalEvent {
    UniversalEvent::new(event_type, data, language)
}

/// Create a test event with metadata
pub fn create_test_event_with_metadata(
    event_type: &str,
    metadata: EventMetadata,
) -> UniversalEvent {
    let mut event = UniversalEvent::new(event_type, Value::Null, Language::Rust);
    event.metadata = metadata;
    event
}

/// Create a test event bus
pub fn create_test_event_bus() -> Arc<EventBus> {
    Arc::new(EventBus::new())
}

/// Create a test correlation tracker
pub fn create_test_correlation_tracker() -> Arc<EventCorrelationTracker> {
    Arc::new(EventCorrelationTracker::default())
}

/// Create a sequence of correlated test events
pub fn create_correlated_event_sequence(
    event_types: Vec<&str>,
    correlation_id: Uuid,
) -> Vec<UniversalEvent> {
    event_types
        .into_iter()
        .map(|event_type| create_test_event_with_correlation(event_type, correlation_id))
        .collect()
}

/// Create test event data for different scenarios
pub mod event_data {
    use serde_json::json;

    pub fn agent_execution_data(agent_id: &str, input: &str) -> serde_json::Value {
        json!({
            "agent_id": agent_id,
            "input": input,
            "timestamp": chrono::Utc::now()
        })
    }

    pub fn tool_execution_data(tool_name: &str, params: serde_json::Value) -> serde_json::Value {
        json!({
            "tool": tool_name,
            "parameters": params,
            "timestamp": chrono::Utc::now()
        })
    }

    pub fn workflow_step_data(workflow_id: &str, step: &str) -> serde_json::Value {
        json!({
            "workflow_id": workflow_id,
            "step": step,
            "timestamp": chrono::Utc::now()
        })
    }

    pub fn error_data(error_type: &str, message: &str) -> serde_json::Value {
        json!({
            "error_type": error_type,
            "message": message,
            "timestamp": chrono::Utc::now()
        })
    }
}

/// Test event collector for testing EventEmitter implementations
///
/// This collector implements the EventEmitter trait and stores all emitted events
/// in memory for later assertion. It's designed to be used in unit and integration
/// tests to verify that components are correctly emitting events.
///
/// # Example
/// ```rust
/// use llmspell_testing::event_helpers::TestEventCollector;
/// use serde_json::json;
///
/// #[tokio::test]
/// async fn test_component_emits_events() {
///     let collector = TestEventCollector::new();
///     
///     // Emit some test events
///     collector.emit("test.started", json!({"id": "test-123"})).await.unwrap();
///     collector.emit("test.completed", json!({"result": "success"})).await.unwrap();
///     
///     // Assert events were captured
///     assert_event_emitted(&collector, "test.started");
///     assert_event_count(&collector, 2);
///     assert_event_data_contains(&collector, "test.started", "id", &json!("test-123"));
/// }
/// ```
#[derive(Debug, Default)]
pub struct TestEventCollector {
    /// All events captured by this collector
    events: Arc<RwLock<Vec<EventData>>>,
    /// Configuration for this test emitter
    config: EventConfig,
    /// Whether events are enabled (for testing is_enabled behavior)
    enabled: bool,
}

impl TestEventCollector {
    /// Create a new test event collector with default configuration
    pub fn new() -> Self {
        Self {
            events: Arc::new(RwLock::new(Vec::new())),
            config: EventConfig::default(),
            enabled: true,
        }
    }

    /// Create a test event collector with custom configuration
    pub fn with_config(config: EventConfig) -> Self {
        Self {
            events: Arc::new(RwLock::new(Vec::new())),
            enabled: config.enabled,
            config,
        }
    }

    /// Create a disabled test event collector (for testing disabled behavior)
    pub fn disabled() -> Self {
        let config = EventConfig {
            enabled: false,
            ..EventConfig::default()
        };
        Self {
            events: Arc::new(RwLock::new(Vec::new())),
            enabled: false,
            config,
        }
    }

    /// Get all captured events
    pub fn get_events(&self) -> Vec<EventData> {
        self.events.read().unwrap().clone()
    }

    /// Get the number of captured events
    pub fn event_count(&self) -> usize {
        self.events.read().unwrap().len()
    }

    /// Clear all captured events
    pub fn clear(&self) {
        self.events.write().unwrap().clear();
    }

    /// Check if a specific event type was emitted
    pub fn has_event_type(&self, event_type: &str) -> bool {
        self.events
            .read()
            .unwrap()
            .iter()
            .any(|event| event.event_type == event_type)
    }

    /// Get all events of a specific type
    pub fn get_events_of_type(&self, event_type: &str) -> Vec<EventData> {
        self.events
            .read()
            .unwrap()
            .iter()
            .filter(|event| event.event_type == event_type)
            .cloned()
            .collect()
    }

    /// Get the latest event
    pub fn latest_event(&self) -> Option<EventData> {
        self.events.read().unwrap().last().cloned()
    }

    /// Get events with a specific correlation ID
    pub fn get_correlated_events(&self, correlation_id: &str) -> Vec<EventData> {
        self.events
            .read()
            .unwrap()
            .iter()
            .filter(|event| event.correlation_id.as_ref() == Some(&correlation_id.to_string()))
            .cloned()
            .collect()
    }

    /// Add an event directly to the collector (for testing purposes)
    pub fn add_event(&self, event: EventData) {
        self.events.write().unwrap().push(event);
    }
}

#[async_trait]
impl EventEmitter for TestEventCollector {
    async fn emit(&self, event_type: &str, data: Value) -> Result<()> {
        if !self.enabled {
            return Ok(());
        }

        let event = EventData {
            event_type: event_type.to_string(),
            component_id: llmspell_core::ComponentId::new(),
            data,
            ..Default::default()
        };

        self.events.write().unwrap().push(event);
        Ok(())
    }

    async fn emit_structured(&self, event: EventData) -> Result<()> {
        if !self.enabled {
            return Ok(());
        }

        self.events.write().unwrap().push(event);
        Ok(())
    }

    fn is_enabled(&self) -> bool {
        self.enabled
    }

    fn config(&self) -> EventConfig {
        self.config.clone()
    }
}

/// Helper function to assert that an event of a specific type was emitted
pub fn assert_event_emitted(collector: &TestEventCollector, event_type: &str) {
    assert!(
        collector.has_event_type(event_type),
        "Expected event '{}' to be emitted, but it was not found. Emitted events: {:?}",
        event_type,
        collector
            .get_events()
            .iter()
            .map(|e| &e.event_type)
            .collect::<Vec<_>>()
    );
}

/// Helper function to assert the number of events emitted
pub fn assert_event_count(collector: &TestEventCollector, expected_count: usize) {
    let actual_count = collector.event_count();
    assert_eq!(
        actual_count,
        expected_count,
        "Expected {} events, but found {}. Events: {:?}",
        expected_count,
        actual_count,
        collector
            .get_events()
            .iter()
            .map(|e| &e.event_type)
            .collect::<Vec<_>>()
    );
}

/// Helper function to assert that an event contains specific data
pub fn assert_event_data_contains(
    collector: &TestEventCollector,
    event_type: &str,
    key: &str,
    expected_value: &Value,
) {
    let events = collector.get_events_of_type(event_type);
    assert!(
        !events.is_empty(),
        "No events of type '{}' found",
        event_type
    );

    let found = events
        .iter()
        .any(|event| event.data.get(key) == Some(expected_value));

    assert!(
        found,
        "Expected event '{}' to contain '{}': {:?}, but it was not found. Event data: {:?}",
        event_type,
        key,
        expected_value,
        events.iter().map(|e| &e.data).collect::<Vec<_>>()
    );
}

/// Helper function to assert that events were emitted in a specific order
pub fn assert_event_sequence(collector: &TestEventCollector, expected_sequence: &[&str]) {
    let events = collector.get_events();
    let actual_sequence: Vec<&str> = events.iter().map(|e| e.event_type.as_str()).collect();

    assert_eq!(
        actual_sequence, expected_sequence,
        "Event sequence mismatch. Expected: {:?}, Actual: {:?}",
        expected_sequence, actual_sequence
    );
}

/// Helper function to assert that correlated events exist
pub fn assert_correlated_events(
    collector: &TestEventCollector,
    correlation_id: &str,
    expected_count: usize,
) {
    let correlated = collector.get_correlated_events(correlation_id);
    assert_eq!(
        correlated.len(),
        expected_count,
        "Expected {} correlated events for ID '{}', but found {}. Events: {:?}",
        expected_count,
        correlation_id,
        correlated.len(),
        correlated.iter().map(|e| &e.event_type).collect::<Vec<_>>()
    );
}

/// Helper function to create a test EventData with minimal setup
pub fn create_test_event_data(event_type: &str, data: Value) -> EventData {
    EventData {
        event_type: event_type.to_string(),
        component_id: llmspell_core::ComponentId::new(),
        data,
        ..Default::default()
    }
}

/// Helper function to create a test EventData with correlation
pub fn create_correlated_event_data(
    event_type: &str,
    data: Value,
    correlation_id: &str,
) -> EventData {
    EventData {
        event_type: event_type.to_string(),
        component_id: llmspell_core::ComponentId::new(),
        data,
        correlation_id: Some(correlation_id.to_string()),
        ..Default::default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_event() {
        let event = create_test_event("test.event");
        assert_eq!(event.event_type, "test.event");
        assert_eq!(event.language, Language::Rust);
        assert_eq!(event.data, Value::Null);
    }

    #[test]
    fn test_create_event_with_data() {
        let data = serde_json::json!({"key": "value"});
        let event = create_test_event_with_data("test.event", data.clone());
        assert_eq!(event.event_type, "test.event");
        assert_eq!(event.data, data);
    }

    #[test]
    fn test_create_event_with_correlation() {
        let correlation_id = Uuid::new_v4();
        let event = create_test_event_with_correlation("test.event", correlation_id);
        assert_eq!(event.metadata.correlation_id, correlation_id);
    }

    #[test]
    fn test_create_event_with_language() {
        let data = serde_json::json!({"test": true});
        let event =
            create_test_event_with_language("test.event", data.clone(), Language::JavaScript);
        assert_eq!(event.language, Language::JavaScript);
        assert_eq!(event.data, data);
    }

    #[test]
    fn test_create_correlated_sequence() {
        let correlation_id = Uuid::new_v4();
        let events =
            create_correlated_event_sequence(vec!["start", "process", "complete"], correlation_id);

        assert_eq!(events.len(), 3);
        for event in &events {
            assert_eq!(event.metadata.correlation_id, correlation_id);
        }
        assert_eq!(events[0].event_type, "start");
        assert_eq!(events[1].event_type, "process");
        assert_eq!(events[2].event_type, "complete");
    }

    #[test]
    fn test_event_data_generators() {
        let agent_data = event_data::agent_execution_data("test-agent", "hello");
        assert_eq!(agent_data["agent_id"], "test-agent");
        assert_eq!(agent_data["input"], "hello");

        let tool_data = event_data::tool_execution_data(
            "calculator",
            serde_json::json!({"operation": "add", "a": 1, "b": 2}),
        );
        assert_eq!(tool_data["tool"], "calculator");
        assert_eq!(tool_data["parameters"]["operation"], "add");

        let workflow_data = event_data::workflow_step_data("workflow-123", "validation");
        assert_eq!(workflow_data["workflow_id"], "workflow-123");
        assert_eq!(workflow_data["step"], "validation");

        let error_data = event_data::error_data("ValidationError", "Invalid input");
        assert_eq!(error_data["error_type"], "ValidationError");
        assert_eq!(error_data["message"], "Invalid input");
    }

    #[tokio::test]
    async fn test_event_collector_basic_functionality() {
        let collector = TestEventCollector::new();

        // Initially empty
        assert_eq!(collector.event_count(), 0);
        assert!(!collector.has_event_type("test.event"));

        // Emit an event
        collector
            .emit("test.started", serde_json::json!({"id": "test-123"}))
            .await
            .unwrap();

        // Verify it was captured
        assert_eq!(collector.event_count(), 1);
        assert!(collector.has_event_type("test.started"));
        assert_event_emitted(&collector, "test.started");

        // Emit another event
        collector
            .emit("test.completed", serde_json::json!({"result": "success"}))
            .await
            .unwrap();

        // Verify both events
        assert_eq!(collector.event_count(), 2);
        assert_event_count(&collector, 2);
        assert_event_emitted(&collector, "test.completed");
    }

    #[tokio::test]
    async fn test_event_collector_structured_events() {
        let collector = TestEventCollector::new();

        let event = create_test_event_data(
            "structured.test",
            serde_json::json!({"complex": {"nested": "data"}}),
        );

        collector.emit_structured(event).await.unwrap();

        assert_eq!(collector.event_count(), 1);
        assert_event_emitted(&collector, "structured.test");

        let captured = collector.get_events_of_type("structured.test");
        assert_eq!(captured.len(), 1);
        assert_eq!(captured[0].data["complex"]["nested"], "data");
    }

    #[tokio::test]
    async fn test_event_collector_disabled() {
        let collector = TestEventCollector::disabled();

        assert!(!collector.is_enabled());

        // Events should not be captured when disabled
        collector
            .emit("test.event", serde_json::json!({}))
            .await
            .unwrap();
        assert_eq!(collector.event_count(), 0);

        let event = create_test_event_data("structured.event", serde_json::json!({}));
        collector.emit_structured(event).await.unwrap();
        assert_eq!(collector.event_count(), 0);
    }

    #[tokio::test]
    async fn test_event_collector_correlation() {
        let collector = TestEventCollector::new();
        let correlation_id = "test-correlation-123";

        // Emit correlated events
        let event1 = create_correlated_event_data(
            "process.started",
            serde_json::json!({"step": 1}),
            correlation_id,
        );
        let event2 = create_correlated_event_data(
            "process.completed",
            serde_json::json!({"step": 2}),
            correlation_id,
        );

        collector.emit_structured(event1).await.unwrap();
        collector.emit_structured(event2).await.unwrap();

        // Also emit a non-correlated event
        collector
            .emit("other.event", serde_json::json!({}))
            .await
            .unwrap();

        // Verify correlation queries
        let correlated = collector.get_correlated_events(correlation_id);
        assert_eq!(correlated.len(), 2);
        assert_correlated_events(&collector, correlation_id, 2);

        // Verify sequence
        assert_eq!(correlated[0].event_type, "process.started");
        assert_eq!(correlated[1].event_type, "process.completed");
    }

    #[test]
    fn test_event_helper_functions() {
        let collector = TestEventCollector::new();

        // Test assertion helpers with mock data
        let events = vec![
            create_test_event_data("first", serde_json::json!({"key": "value1"})),
            create_test_event_data("second", serde_json::json!({"key": "value2"})),
            create_test_event_data("first", serde_json::json!({"key": "value3"})),
        ];

        // Manually add events for testing helper functions
        for event in events {
            collector.events.write().unwrap().push(event);
        }

        // Test helper functions
        assert_event_count(&collector, 3);
        assert_event_emitted(&collector, "first");
        assert_event_emitted(&collector, "second");
        assert_event_data_contains(&collector, "first", "key", &serde_json::json!("value1"));
        assert_event_data_contains(&collector, "second", "key", &serde_json::json!("value2"));

        // Test sequence assertion
        assert_event_sequence(&collector, &["first", "second", "first"]);

        // Test type filtering
        let first_events = collector.get_events_of_type("first");
        assert_eq!(first_events.len(), 2);

        let second_events = collector.get_events_of_type("second");
        assert_eq!(second_events.len(), 1);
    }

    #[test]
    fn test_event_collector_clear() {
        let collector = TestEventCollector::new();

        // Add some events manually
        collector
            .events
            .write()
            .unwrap()
            .push(create_test_event_data("test", serde_json::json!({})));
        collector
            .events
            .write()
            .unwrap()
            .push(create_test_event_data("test2", serde_json::json!({})));

        assert_eq!(collector.event_count(), 2);

        // Clear and verify
        collector.clear();
        assert_eq!(collector.event_count(), 0);
        assert!(!collector.has_event_type("test"));
    }

    #[test]
    fn test_create_event_data_helpers() {
        let basic_event = create_test_event_data("test.basic", serde_json::json!({"key": "value"}));
        assert_eq!(basic_event.event_type, "test.basic");
        assert_eq!(basic_event.data["key"], "value");
        assert!(basic_event.correlation_id.is_none());

        let correlated_event = create_correlated_event_data(
            "test.correlated",
            serde_json::json!({"key": "value"}),
            "corr-123",
        );
        assert_eq!(correlated_event.event_type, "test.correlated");
        assert_eq!(correlated_event.data["key"], "value");
        assert_eq!(
            correlated_event.correlation_id,
            Some("corr-123".to_string())
        );
    }
}
