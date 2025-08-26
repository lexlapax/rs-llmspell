// ABOUTME: UniversalEvent type for cross-language event propagation
// ABOUTME: Provides language-agnostic event format with metadata and sequencing

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::sync::atomic::{AtomicU64, Ordering};
use uuid::Uuid;

/// Global sequence counter for event ordering
static SEQUENCE_COUNTER: AtomicU64 = AtomicU64::new(0);

/// Language identifier for event source
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Language {
    /// Rust native events
    Rust,
    /// Lua script events
    Lua,
    /// JavaScript events
    JavaScript,
    /// Python events
    Python,
    /// Unknown or external source
    Unknown,
}

impl Language {
    /// Get the language name as a string
    pub fn as_str(&self) -> &'static str {
        match self {
            Language::Rust => "rust",
            Language::Lua => "lua",
            Language::JavaScript => "javascript",
            Language::Python => "python",
            Language::Unknown => "unknown",
        }
    }
}

impl std::fmt::Display for Language {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Event metadata for tracking and debugging
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventMetadata {
    /// Correlation ID for tracing related events
    pub correlation_id: Uuid,
    /// Source component ID
    pub source: Option<String>,
    /// Target component ID (for directed events)
    pub target: Option<String>,
    /// Custom tags for filtering
    pub tags: Vec<String>,
    /// Priority level (lower is higher priority)
    pub priority: i32,
    /// Time-to-live in seconds
    pub ttl: Option<u64>,
}

impl Default for EventMetadata {
    fn default() -> Self {
        Self {
            correlation_id: Uuid::new_v4(),
            source: None,
            target: None,
            tags: Vec::new(),
            priority: 0,
            ttl: None,
        }
    }
}

/// Universal event format for cross-language compatibility
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UniversalEvent {
    /// Unique event ID
    pub id: Uuid,
    /// Event type/name (e.g., "agent.state_changed")
    pub event_type: String,
    /// Event payload data
    pub data: Value,
    /// Source language
    pub language: Language,
    /// Event timestamp
    pub timestamp: DateTime<Utc>,
    /// Sequence number for ordering
    pub sequence: u64,
    /// Event metadata
    pub metadata: EventMetadata,
    /// Schema version for compatibility
    pub schema_version: String,
}

impl UniversalEvent {
    /// Create a new universal event
    pub fn new(event_type: impl Into<String>, data: Value, language: Language) -> Self {
        Self {
            id: Uuid::new_v4(),
            event_type: event_type.into(),
            data,
            language,
            timestamp: Utc::now(),
            sequence: SEQUENCE_COUNTER.fetch_add(1, Ordering::SeqCst),
            metadata: EventMetadata::default(),
            schema_version: "1.0".to_string(),
        }
    }

    /// Create an event with custom metadata
    pub fn with_metadata(
        event_type: impl Into<String>,
        data: Value,
        language: Language,
        metadata: EventMetadata,
    ) -> Self {
        let mut event = Self::new(event_type, data, language);
        event.metadata = metadata;
        event
    }

    /// Set the correlation ID
    pub fn with_correlation_id(mut self, correlation_id: Uuid) -> Self {
        self.metadata.correlation_id = correlation_id;
        self
    }

    /// Set the source component
    pub fn with_source(mut self, source: impl Into<String>) -> Self {
        self.metadata.source = Some(source.into());
        self
    }

    /// Set the target component
    pub fn with_target(mut self, target: impl Into<String>) -> Self {
        self.metadata.target = Some(target.into());
        self
    }

    /// Add a tag
    pub fn with_tag(mut self, tag: impl Into<String>) -> Self {
        self.metadata.tags.push(tag.into());
        self
    }

    /// Set priority
    pub fn with_priority(mut self, priority: i32) -> Self {
        self.metadata.priority = priority;
        self
    }

    /// Set TTL in seconds
    pub fn with_ttl(mut self, ttl: u64) -> Self {
        self.metadata.ttl = Some(ttl);
        self
    }

    /// Check if event has expired
    pub fn is_expired(&self) -> bool {
        if let Some(ttl) = self.metadata.ttl {
            #[allow(clippy::cast_sign_loss)]
            let elapsed = Utc::now()
                .signed_duration_since(self.timestamp)
                .num_seconds() as u64;
            elapsed > ttl
        } else {
            false
        }
    }

    /// Get event age in seconds
    pub fn age_seconds(&self) -> i64 {
        Utc::now()
            .signed_duration_since(self.timestamp)
            .num_seconds()
    }

    /// Convert to JSON for cross-language serialization
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }

    /// Create from JSON
    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }

    /// Get a reference to the event data
    pub fn data(&self) -> &Value {
        &self.data
    }

    /// Get mutable reference to event data
    pub fn data_mut(&mut self) -> &mut Value {
        &mut self.data
    }

    /// Extract typed data from the event
    pub fn extract_data<T: for<'de> Deserialize<'de>>(&self) -> Result<T, serde_json::Error> {
        serde_json::from_value(self.data.clone())
    }

    /// Check if event matches a pattern
    pub fn matches_pattern(&self, pattern: &str) -> bool {
        if pattern == "*" {
            return true;
        }

        if let Some(prefix) = pattern.strip_suffix('*') {
            self.event_type.starts_with(prefix)
        } else {
            self.event_type == pattern
        }
    }
}

/// Builder for UniversalEvent
pub struct UniversalEventBuilder {
    event_type: String,
    data: Value,
    language: Language,
    metadata: EventMetadata,
}

impl UniversalEventBuilder {
    /// Create a new event builder
    pub fn new(event_type: impl Into<String>) -> Self {
        Self {
            event_type: event_type.into(),
            data: Value::Null,
            language: Language::Rust,
            metadata: EventMetadata::default(),
        }
    }

    /// Set event data
    pub fn data(mut self, data: Value) -> Self {
        self.data = data;
        self
    }

    /// Set source language
    pub fn language(mut self, language: Language) -> Self {
        self.language = language;
        self
    }

    /// Set correlation ID
    pub fn correlation_id(mut self, id: Uuid) -> Self {
        self.metadata.correlation_id = id;
        self
    }

    /// Set source component
    pub fn source(mut self, source: impl Into<String>) -> Self {
        self.metadata.source = Some(source.into());
        self
    }

    /// Set target component
    pub fn target(mut self, target: impl Into<String>) -> Self {
        self.metadata.target = Some(target.into());
        self
    }

    /// Add a tag
    pub fn tag(mut self, tag: impl Into<String>) -> Self {
        self.metadata.tags.push(tag.into());
        self
    }

    /// Set priority
    pub fn priority(mut self, priority: i32) -> Self {
        self.metadata.priority = priority;
        self
    }

    /// Set TTL
    pub fn ttl(mut self, ttl: u64) -> Self {
        self.metadata.ttl = Some(ttl);
        self
    }

    /// Build the event
    pub fn build(self) -> UniversalEvent {
        UniversalEvent::with_metadata(self.event_type, self.data, self.language, self.metadata)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_event_creation() {
        let event = UniversalEvent::new(
            "test.event",
            serde_json::json!({"key": "value"}),
            Language::Rust,
        );

        assert_eq!(event.event_type, "test.event");
        assert_eq!(event.language, Language::Rust);
        assert!(!event.is_expired());
    }
    #[test]
    fn test_event_builder() {
        let event = UniversalEventBuilder::new("test.built")
            .data(serde_json::json!({"built": true}))
            .language(Language::Lua)
            .source("test-component")
            .tag("test")
            .priority(-1)
            .ttl(300)
            .build();

        assert_eq!(event.event_type, "test.built");
        assert_eq!(event.language, Language::Lua);
        assert_eq!(event.metadata.source, Some("test-component".to_string()));
        assert_eq!(event.metadata.priority, -1);
        assert_eq!(event.metadata.ttl, Some(300));
    }
    #[test]
    fn test_event_serialization() {
        let event = UniversalEvent::new(
            "test.serialization",
            serde_json::json!({"test": true}),
            Language::JavaScript,
        );

        let json = event.to_json().unwrap();
        let deserialized = UniversalEvent::from_json(&json).unwrap();

        assert_eq!(deserialized.event_type, event.event_type);
        assert_eq!(deserialized.data, event.data);
        assert_eq!(deserialized.sequence, event.sequence);
    }
    #[test]
    fn test_pattern_matching() {
        let event = UniversalEvent::new("system.startup", Value::Null, Language::Rust);

        assert!(event.matches_pattern("*"));
        assert!(event.matches_pattern("system.*"));
        assert!(event.matches_pattern("system.startup"));
        assert!(!event.matches_pattern("agent.*"));
        assert!(!event.matches_pattern("system.shutdown"));
    }
    #[test]
    fn test_sequence_ordering() {
        let event1 = UniversalEvent::new("event1", Value::Null, Language::Rust);
        let event2 = UniversalEvent::new("event2", Value::Null, Language::Rust);
        let event3 = UniversalEvent::new("event3", Value::Null, Language::Rust);

        assert!(event1.sequence < event2.sequence);
        assert!(event2.sequence < event3.sequence);
    }
    #[test]
    fn test_ttl_expiration() {
        let mut event = UniversalEvent::new("expiring", Value::Null, Language::Rust).with_ttl(0); // Expire immediately

        // Manually set timestamp to past
        event.timestamp = Utc::now() - chrono::Duration::seconds(10);

        assert!(event.is_expired());
        assert!(event.age_seconds() >= 10);
    }
}
