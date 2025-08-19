//! ABOUTME: EventEmitter trait for universal event emission across all components
//! ABOUTME: Provides abstraction for lifecycle events, observability, and component communication

use crate::{ComponentId, Result};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::fmt::Debug;

/// Universal event emission trait for all components
///
/// This trait provides a clean abstraction for event operations that can be
/// implemented by various backends (EventBus, logging, webhooks, etc.).
/// It's designed to provide observability and coordination for all component
/// interactions, following the same pattern as StateAccess.
///
/// # Key Design Decisions
///
/// - **Optional Implementation**: Components work with `Option<Arc<dyn EventEmitter>>`
/// - **Fire-and-Forget**: Events are non-blocking and errors are logged but not propagated
/// - **Structured Data**: Events use JSON values for maximum flexibility
/// - **Configuration-Driven**: Can be enabled/disabled via config
///
/// # Usage Patterns
///
/// ```ignore
/// // Components emit lifecycle events automatically
/// if let Some(events) = &context.events {
///     events.emit("agent.started", json!({ "agent_id": id })).await;
/// }
///
/// // Workflows emit step events
/// events.emit("workflow.step.completed", json!({
///     "step": step_name,
///     "duration_ms": elapsed
/// })).await;
/// ```
#[async_trait]
pub trait EventEmitter: Send + Sync + Debug {
    /// Emit a simple event with type and data
    ///
    /// This is the primary method for emitting events. The event type
    /// should follow a dot-notation convention (e.g., "component.action").
    ///
    /// # Arguments
    /// * `event_type` - The event type/name (e.g., "agent.started")
    /// * `data` - The event payload as JSON value
    ///
    /// # Example
    /// ```ignore
    /// events.emit("tool.executed", json!({
    ///     "tool_name": "calculator",
    ///     "duration_ms": 42
    /// })).await?;
    /// ```
    async fn emit(&self, event_type: &str, data: Value) -> Result<()>;

    /// Emit with full event structure
    ///
    /// For more complex events that need additional metadata,
    /// correlation IDs, or parent event references.
    ///
    /// # Arguments
    /// * `event` - Full event data structure
    ///
    /// # Example
    /// ```ignore
    /// let event = EventData {
    ///     event_type: "workflow.step.started".to_string(),
    ///     component_id: ComponentId::new(),
    ///     data: json!({ "step": "validation" }),
    ///     correlation_id: Some(workflow_id),
    ///     ..Default::default()
    /// };
    /// events.emit_structured(event).await?;
    /// ```
    async fn emit_structured(&self, event: EventData) -> Result<()>;

    /// Check if events are enabled
    ///
    /// Components can use this to skip event preparation if events
    /// are disabled, improving performance.
    ///
    /// # Example
    /// ```ignore
    /// if events.is_enabled() {
    ///     let expensive_data = calculate_metrics();
    ///     events.emit("metrics.calculated", expensive_data).await?;
    /// }
    /// ```
    fn is_enabled(&self) -> bool {
        true
    }

    /// Get event configuration
    ///
    /// Returns the current event configuration for filtering
    /// and processing decisions.
    fn config(&self) -> EventConfig {
        EventConfig::default()
    }
}

/// Full event data structure for structured events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventData {
    /// Event type/name (e.g., "agent.completed")
    pub event_type: String,

    /// Component that emitted the event
    pub component_id: ComponentId,

    /// Event payload data
    pub data: Value,

    /// Additional metadata for the event
    #[serde(default)]
    pub metadata: HashMap<String, Value>,

    /// Correlation ID for tracing related events
    #[serde(skip_serializing_if = "Option::is_none")]
    pub correlation_id: Option<String>,

    /// Parent event ID for event chains
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent_event_id: Option<String>,

    /// Session ID for grouping events
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session_id: Option<String>,

    /// User ID for multi-tenant scenarios
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_id: Option<String>,
}

impl Default for EventData {
    fn default() -> Self {
        Self {
            event_type: String::new(),
            component_id: ComponentId::new(),
            data: Value::Null,
            metadata: HashMap::new(),
            correlation_id: None,
            parent_event_id: None,
            session_id: None,
            user_id: None,
        }
    }
}

impl EventData {
    /// Create a new event with the given type
    pub fn new(event_type: impl Into<String>) -> Self {
        Self {
            event_type: event_type.into(),
            ..Default::default()
        }
    }

    /// Builder-style method to set component ID
    pub fn with_component(mut self, component_id: ComponentId) -> Self {
        self.component_id = component_id;
        self
    }

    /// Builder-style method to set data
    pub fn with_data(mut self, data: Value) -> Self {
        self.data = data;
        self
    }

    /// Builder-style method to set correlation ID
    pub fn with_correlation(mut self, correlation_id: impl Into<String>) -> Self {
        self.correlation_id = Some(correlation_id.into());
        self
    }

    /// Builder-style method to add metadata
    pub fn with_metadata(mut self, key: impl Into<String>, value: Value) -> Self {
        self.metadata.insert(key.into(), value);
        self
    }
}

/// Event configuration for filtering and processing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventConfig {
    /// Whether events are enabled globally
    #[serde(default = "default_true")]
    pub enabled: bool,

    /// Event types to include (glob patterns)
    #[serde(default)]
    pub include_types: Vec<String>,

    /// Event types to exclude (glob patterns)
    #[serde(default)]
    pub exclude_types: Vec<String>,

    /// Whether to emit timing/performance events
    #[serde(default = "default_true")]
    pub emit_timing_events: bool,

    /// Whether to emit state change events
    #[serde(default)]
    pub emit_state_events: bool,

    /// Whether to emit debug-level events
    #[serde(default)]
    pub emit_debug_events: bool,

    /// Maximum events per second (rate limiting)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_events_per_second: Option<u32>,
}

fn default_true() -> bool {
    true
}

impl Default for EventConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            include_types: Vec::new(),
            exclude_types: Vec::new(),
            emit_timing_events: true,
            emit_state_events: false,
            emit_debug_events: false,
            max_events_per_second: None,
        }
    }
}

impl EventConfig {
    /// Check if a given event type should be emitted
    pub fn should_emit(&self, event_type: &str) -> bool {
        if !self.enabled {
            return false;
        }

        // Check excludes first (they take precedence)
        for pattern in &self.exclude_types {
            if Self::matches_pattern(pattern, event_type) {
                return false;
            }
        }

        // If includes are specified, event must match one
        if !self.include_types.is_empty() {
            for pattern in &self.include_types {
                if Self::matches_pattern(pattern, event_type) {
                    return true;
                }
            }
            return false;
        }

        // Default to true if no includes specified
        true
    }

    /// Simple glob pattern matching (supports * wildcard)
    fn matches_pattern(pattern: &str, text: &str) -> bool {
        if pattern == "*" {
            return true;
        }

        if let Some(prefix) = pattern.strip_suffix('*') {
            return text.starts_with(prefix);
        }

        if let Some(suffix) = pattern.strip_prefix('*') {
            return text.ends_with(suffix);
        }

        pattern == text
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_event_config_should_emit() {
        let config = EventConfig {
            enabled: true,
            include_types: vec!["agent.*".to_string(), "tool.*".to_string()],
            exclude_types: vec!["*.debug".to_string()],
            ..EventConfig::default()
        };

        assert!(config.should_emit("agent.started"));
        assert!(config.should_emit("tool.executed"));
        assert!(!config.should_emit("workflow.started")); // Not in includes
        assert!(!config.should_emit("agent.debug")); // Excluded
        assert!(!config.should_emit("tool.debug")); // Excluded
    }

    #[test]
    fn test_event_data_builder() {
        let event = EventData::new("test.event")
            .with_component(ComponentId::new())
            .with_data(serde_json::json!({ "key": "value" }))
            .with_correlation("test-correlation-id")
            .with_metadata("custom", serde_json::json!("metadata"));

        assert_eq!(event.event_type, "test.event");
        assert_eq!(
            event.correlation_id,
            Some("test-correlation-id".to_string())
        );
        assert_eq!(
            event.metadata.get("custom"),
            Some(&serde_json::json!("metadata"))
        );
    }
}
