// ABOUTME: EventBusAdapter implements EventEmitter trait for llmspell-events integration
// ABOUTME: Bridges the core EventEmitter trait with the actual EventBus implementation

use async_trait::async_trait;
use llmspell_core::traits::event::{EventConfig, EventData, EventEmitter};
use llmspell_core::{LLMSpellError, Result};
use llmspell_events::{EventBus, Language, UniversalEvent};
use serde_json::Value;
use std::fmt::Debug;
use std::sync::Arc;

/// Adapter that implements `EventEmitter` trait using `EventBus`
pub struct EventBusAdapter {
    /// The underlying event bus
    event_bus: Arc<EventBus>,
    /// Configuration for event emission
    config: EventConfig,
    /// Language context for events
    language: Language,
}

impl EventBusAdapter {
    /// Create a new `EventBusAdapter` with the given `EventBus`
    #[must_use]
    pub fn new(event_bus: Arc<EventBus>) -> Self {
        Self {
            event_bus,
            config: EventConfig::default(),
            language: Language::Rust,
        }
    }

    /// Create with custom configuration
    #[must_use]
    pub const fn with_config(event_bus: Arc<EventBus>, config: EventConfig) -> Self {
        Self {
            event_bus,
            config,
            language: Language::Rust,
        }
    }

    /// Set the language context
    #[must_use]
    pub const fn with_language(mut self, language: Language) -> Self {
        self.language = language;
        self
    }

    /// Convert `EventData` to `UniversalEvent`
    fn to_universal_event(&self, event: EventData) -> UniversalEvent {
        let mut universal = UniversalEvent::new(&event.event_type, event.data, self.language);

        // Set metadata fields
        if let Some(correlation_id) = event.correlation_id {
            // Parse UUID from string
            if let Ok(uuid) = correlation_id.parse::<uuid::Uuid>() {
                universal.metadata.correlation_id = uuid;
            }
        }

        // Set source as component_id
        universal.metadata.source = Some(event.component_id.to_string());

        // Build tags list from metadata
        let mut tags = Vec::new();

        // Add component_id as a tag
        tags.push(format!("component:{}", event.component_id));

        // Store parent_event_id in tags
        if let Some(parent_event_id) = event.parent_event_id {
            tags.push(format!("parent_event:{parent_event_id}"));
        }

        // Store session_id in tags
        if let Some(session_id) = event.session_id {
            tags.push(format!("session:{session_id}"));
        }

        // Store user_id in tags
        if let Some(user_id) = event.user_id {
            tags.push(format!("user:{user_id}"));
        }

        // Add any additional metadata as tags (key:value format)
        for (key, value) in event.metadata {
            // Convert Value to String for tags
            let value_str = match value {
                Value::String(s) => s,
                other => other.to_string(),
            };
            tags.push(format!("{key}:{value_str}"));
        }

        universal.metadata.tags = tags;

        universal
    }
}

impl Debug for EventBusAdapter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("EventBusAdapter")
            .field("language", &self.language)
            .field("config", &self.config)
            .finish_non_exhaustive()
    }
}

#[async_trait]
impl EventEmitter for EventBusAdapter {
    /// Emit a simple event with type and data
    async fn emit(&self, event_type: &str, data: Value) -> Result<()> {
        // Check if this event should be emitted based on config
        if !self.config.should_emit(event_type) {
            return Ok(());
        }

        let event = UniversalEvent::new(event_type, data, self.language);
        self.event_bus
            .publish(event)
            .await
            .map_err(|e| LLMSpellError::Component {
                message: format!("Failed to emit event: {e}"),
                source: Some(Box::new(e)),
            })?;
        Ok(())
    }

    /// Emit a structured event with full metadata
    async fn emit_structured(&self, event: EventData) -> Result<()> {
        // Check if this event should be emitted based on config
        if !self.config.should_emit(&event.event_type) {
            return Ok(());
        }

        let universal_event = self.to_universal_event(event);
        self.event_bus
            .publish(universal_event)
            .await
            .map_err(|e| LLMSpellError::Component {
                message: format!("Failed to emit structured event: {e}"),
                source: Some(Box::new(e)),
            })?;
        Ok(())
    }

    /// Check if events are enabled
    fn is_enabled(&self) -> bool {
        self.config.enabled
    }

    /// Get the event configuration
    fn config(&self) -> EventConfig {
        self.config.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use llmspell_core::ComponentId;

    #[tokio::test]
    async fn test_event_bus_adapter_creation() {
        let event_bus = Arc::new(EventBus::new());
        let adapter = EventBusAdapter::new(event_bus);

        assert!(adapter.is_enabled());
    }

    #[tokio::test]
    async fn test_event_emission() {
        let event_bus = Arc::new(EventBus::new());
        let adapter = EventBusAdapter::new(event_bus.clone());

        // Subscribe to events
        let mut receiver = event_bus.subscribe("test.*").await.unwrap();

        // Emit an event through the adapter
        adapter
            .emit("test.event", serde_json::json!({"message": "hello"}))
            .await
            .unwrap();

        // Verify the event was received
        let event = tokio::time::timeout(std::time::Duration::from_millis(100), receiver.recv())
            .await
            .unwrap()
            .unwrap();

        assert_eq!(event.event_type, "test.event");
    }

    #[tokio::test]
    async fn test_structured_event_emission() {
        let event_bus = Arc::new(EventBus::new());
        let adapter = EventBusAdapter::new(event_bus.clone());

        // Subscribe to events
        let mut receiver = event_bus.subscribe("component.*").await.unwrap();

        // Create a structured event
        let event_data = EventData::new("component.started")
            .with_component(ComponentId::from_name("test-component"))
            .with_data(serde_json::json!({"status": "running"}))
            .with_correlation("test-correlation-123");

        // Emit the structured event
        adapter.emit_structured(event_data).await.unwrap();

        // Verify the event was received with metadata
        let event = tokio::time::timeout(std::time::Duration::from_millis(100), receiver.recv())
            .await
            .unwrap()
            .unwrap();

        assert_eq!(event.event_type, "component.started");
        assert!(event.metadata.source.is_some());
    }

    #[tokio::test]
    async fn test_config_filtering() {
        let event_bus = Arc::new(EventBus::new());

        // Create adapter with custom config that excludes debug events
        let config = EventConfig {
            exclude_types: vec!["*.debug".to_string()],
            ..EventConfig::default()
        };

        let adapter = EventBusAdapter::with_config(event_bus.clone(), config);

        // Subscribe to all events
        let mut receiver = event_bus.subscribe("*").await.unwrap();

        // Try to emit a debug event (should be filtered)
        adapter
            .emit("test.debug", serde_json::json!({"level": "debug"}))
            .await
            .unwrap();

        // Emit a non-debug event (should pass)
        adapter
            .emit("test.info", serde_json::json!({"level": "info"}))
            .await
            .unwrap();

        // Should only receive the non-debug event
        let event = tokio::time::timeout(std::time::Duration::from_millis(100), receiver.recv())
            .await
            .unwrap()
            .unwrap();

        assert_eq!(event.event_type, "test.info");

        // Should not receive another event
        let timeout_result =
            tokio::time::timeout(std::time::Duration::from_millis(100), receiver.recv()).await;

        assert!(timeout_result.is_err());
    }
}
