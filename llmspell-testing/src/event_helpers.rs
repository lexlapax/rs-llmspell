// ABOUTME: Test utilities for event system testing including event creation and fixtures
// ABOUTME: Provides reusable helpers for event bus, correlation, and stream tests

use llmspell_events::{
    bus::EventBus, correlation::EventCorrelationTracker, EventMetadata, Language, UniversalEvent,
};
use serde_json::Value;
use std::sync::Arc;
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
}
