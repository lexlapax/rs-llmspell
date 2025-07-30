//! ABOUTME: Session-specific event types with built-in correlation support
//! ABOUTME: Defines standardized events for session lifecycle and operations

use crate::SessionId;
use llmspell_events::{
    correlation::{CorrelationContext, EventLink, EventRelationship},
    universal_event::{Language, UniversalEvent},
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;

/// Prefix for all session-related events
pub const SESSION_EVENT_PREFIX: &str = "session";

/// Session event types
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SessionEventType {
    /// Session created
    Created,
    /// Session started
    Started,
    /// Session suspended/checkpointed
    Suspended,
    /// Session resumed
    Resumed,
    /// Session completed
    Completed,
    /// Session failed
    Failed,
    /// Session saved
    Saved,
    /// Session loaded
    Loaded,
    /// Session archived
    Archived,
    /// Artifact stored
    ArtifactStored,
    /// Artifact retrieved
    ArtifactRetrieved,
    /// Artifact deleted
    ArtifactDeleted,
    /// State changed
    StateChanged,
    /// Hook executed
    HookExecuted,
    /// Custom event
    Custom(String),
}

impl SessionEventType {
    /// Convert to event type string
    pub fn to_event_type(&self) -> String {
        match self {
            Self::Created => format!("{SESSION_EVENT_PREFIX}.created"),
            Self::Started => format!("{SESSION_EVENT_PREFIX}.started"),
            Self::Suspended => format!("{SESSION_EVENT_PREFIX}.suspended"),
            Self::Resumed => format!("{SESSION_EVENT_PREFIX}.resumed"),
            Self::Completed => format!("{SESSION_EVENT_PREFIX}.completed"),
            Self::Failed => format!("{SESSION_EVENT_PREFIX}.failed"),
            Self::Saved => format!("{SESSION_EVENT_PREFIX}.saved"),
            Self::Loaded => format!("{SESSION_EVENT_PREFIX}.loaded"),
            Self::Archived => format!("{SESSION_EVENT_PREFIX}.archived"),
            Self::ArtifactStored => format!("{SESSION_EVENT_PREFIX}.artifact.stored"),
            Self::ArtifactRetrieved => format!("{SESSION_EVENT_PREFIX}.artifact.retrieved"),
            Self::ArtifactDeleted => format!("{SESSION_EVENT_PREFIX}.artifact.deleted"),
            Self::StateChanged => format!("{SESSION_EVENT_PREFIX}.state.changed"),
            Self::HookExecuted => format!("{SESSION_EVENT_PREFIX}.hook.executed"),
            Self::Custom(name) => format!("{SESSION_EVENT_PREFIX}.custom.{name}"),
        }
    }
}

/// Session event with correlation context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionEvent {
    /// The underlying universal event
    pub event: UniversalEvent,
    /// Associated correlation context
    pub correlation_context: CorrelationContext,
    /// Session ID this event belongs to
    pub session_id: SessionId,
    /// Event type
    pub event_type: SessionEventType,
}

impl SessionEvent {
    /// Create a new session event
    pub fn new(
        session_id: SessionId,
        event_type: SessionEventType,
        data: Value,
        correlation_context: CorrelationContext,
    ) -> Self {
        let mut event = UniversalEvent::new(event_type.to_event_type(), data, Language::Rust);

        // Set correlation ID in event metadata
        event.metadata.correlation_id = correlation_context.correlation_id;

        // Add session ID to event data
        if let Value::Object(ref mut map) = event.data {
            map.insert("session_id".to_string(), json!(session_id.to_string()));
        }

        Self {
            event,
            correlation_context,
            session_id,
            event_type,
        }
    }

    /// Create with parent correlation
    pub fn with_parent(
        session_id: SessionId,
        event_type: SessionEventType,
        data: Value,
        parent_context: &CorrelationContext,
    ) -> Self {
        let correlation_context = parent_context.create_child();
        Self::new(session_id, event_type, data, correlation_context)
    }

    /// Add metadata to the event
    #[must_use]
    pub fn with_metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.correlation_context = self.correlation_context.with_metadata(key, value);
        self
    }

    /// Add a tag to the event
    #[must_use]
    pub fn with_tag(mut self, tag: impl Into<String>) -> Self {
        self.correlation_context = self.correlation_context.with_tag(tag);
        self
    }

    /// Set event source
    #[must_use]
    pub fn with_source(mut self, source: impl Into<String>) -> Self {
        self.event.metadata.source = Some(source.into());
        self
    }

    /// Set event target
    #[must_use]
    pub fn with_target(mut self, target: impl Into<String>) -> Self {
        self.event.metadata.target = Some(target.into());
        self
    }

    /// Create a link to another event
    pub fn link_to(&self, other: &SessionEvent, relationship: EventRelationship) -> EventLink {
        EventLink::new(self.event.id, other.event.id, relationship)
            .with_metadata("from_session", self.session_id.to_string())
            .with_metadata("to_session", other.session_id.to_string())
    }
}

/// Create a session event with a new correlation root
pub fn create_session_event(
    session_id: SessionId,
    event_type: SessionEventType,
    data: Value,
) -> SessionEvent {
    let correlation_context = CorrelationContext::new_root()
        .with_metadata("session_id", session_id.to_string())
        .with_tag("session_lifecycle");

    SessionEvent::new(session_id, event_type, data, correlation_context)
}

/// Create a correlated session event
pub fn create_correlated_event(
    session_id: SessionId,
    event_type: SessionEventType,
    data: Value,
    parent_event: &SessionEvent,
) -> SessionEvent {
    SessionEvent::with_parent(
        session_id,
        event_type,
        data,
        &parent_event.correlation_context,
    )
}

/// Builder for complex session events
pub struct SessionEventBuilder {
    session_id: SessionId,
    event_type: SessionEventType,
    data: HashMap<String, Value>,
    correlation_context: Option<CorrelationContext>,
    source: Option<String>,
    target: Option<String>,
    tags: Vec<String>,
    metadata: HashMap<String, String>,
}

impl SessionEventBuilder {
    /// Create a new builder
    pub fn new(session_id: SessionId, event_type: SessionEventType) -> Self {
        Self {
            session_id,
            event_type,
            data: HashMap::new(),
            correlation_context: None,
            source: None,
            target: None,
            tags: Vec::new(),
            metadata: HashMap::new(),
        }
    }

    /// Add data field
    #[must_use]
    pub fn data(mut self, key: impl Into<String>, value: Value) -> Self {
        self.data.insert(key.into(), value);
        self
    }

    /// Set correlation context
    #[must_use]
    pub fn correlation(mut self, context: CorrelationContext) -> Self {
        self.correlation_context = Some(context);
        self
    }

    /// Set parent correlation
    #[must_use]
    pub fn parent(mut self, parent: &CorrelationContext) -> Self {
        self.correlation_context = Some(parent.create_child());
        self
    }

    /// Set event source
    #[must_use]
    pub fn source(mut self, source: impl Into<String>) -> Self {
        self.source = Some(source.into());
        self
    }

    /// Set event target
    #[must_use]
    pub fn target(mut self, target: impl Into<String>) -> Self {
        self.target = Some(target.into());
        self
    }

    /// Add a tag
    #[must_use]
    pub fn tag(mut self, tag: impl Into<String>) -> Self {
        self.tags.push(tag.into());
        self
    }

    /// Add metadata
    #[must_use]
    pub fn metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }

    /// Build the event
    pub fn build(self) -> SessionEvent {
        let correlation_context = self.correlation_context.unwrap_or_else(|| {
            let mut ctx = CorrelationContext::new_root()
                .with_metadata("session_id", self.session_id.to_string());

            for tag in self.tags {
                ctx = ctx.with_tag(tag);
            }

            for (key, value) in self.metadata {
                ctx = ctx.with_metadata(key, value);
            }

            ctx
        });

        let mut event = SessionEvent::new(
            self.session_id,
            self.event_type,
            json!(self.data),
            correlation_context,
        );

        if let Some(source) = self.source {
            event = event.with_source(source);
        }

        if let Some(target) = self.target {
            event = event.with_target(target);
        }

        event
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_session_event_creation() {
        let session_id = SessionId::new();
        let event = create_session_event(
            session_id,
            SessionEventType::Created,
            json!({
                "name": "Test Session",
                "config": {"auto_save": true}
            }),
        );

        assert_eq!(event.session_id, session_id);
        assert_eq!(event.event_type, SessionEventType::Created);
        assert_eq!(event.event.event_type, "session.created");
        assert!(event.correlation_context.parent_id.is_none());
    }

    #[test]
    fn test_correlated_events() {
        let session_id = SessionId::new();

        let parent_event = create_session_event(session_id, SessionEventType::Started, json!({}));

        let child_event = create_correlated_event(
            session_id,
            SessionEventType::ArtifactStored,
            json!({"artifact_id": "test-artifact"}),
            &parent_event,
        );

        assert_eq!(
            child_event.correlation_context.parent_id,
            Some(parent_event.correlation_context.correlation_id)
        );
        assert_eq!(
            child_event.correlation_context.root_id,
            parent_event.correlation_context.root_id
        );
    }

    #[test]
    fn test_event_linking() {
        let session_id = SessionId::new();

        let event1 = create_session_event(session_id, SessionEventType::Started, json!({}));

        let event2 = create_session_event(session_id, SessionEventType::Completed, json!({}));

        let link = event1.link_to(&event2, EventRelationship::CausedBy);

        assert_eq!(link.from_event_id, event1.event.id);
        assert_eq!(link.to_event_id, event2.event.id);
        assert_eq!(link.relationship, EventRelationship::CausedBy);
        assert_eq!(
            link.metadata.get("from_session"),
            Some(&session_id.to_string())
        );
    }

    #[test]
    fn test_event_builder() {
        let session_id = SessionId::new();

        let event = SessionEventBuilder::new(session_id, SessionEventType::StateChanged)
            .data("key", json!("test_key"))
            .data("old_value", json!(null))
            .data("new_value", json!("test_value"))
            .source("session-manager")
            .target("state-store")
            .tag("state_update")
            .metadata("operation", "set_state")
            .build();

        assert_eq!(event.event_type, SessionEventType::StateChanged);
        assert_eq!(
            event.event.metadata.source,
            Some("session-manager".to_string())
        );
        assert_eq!(event.event.metadata.target, Some("state-store".to_string()));
        assert!(event.correlation_context.has_tag("state_update"));
        assert_eq!(
            event.correlation_context.get_metadata("operation"),
            Some(&"set_state".to_string())
        );
    }

    #[test]
    fn test_event_type_strings() {
        assert_eq!(SessionEventType::Created.to_event_type(), "session.created");
        assert_eq!(
            SessionEventType::ArtifactStored.to_event_type(),
            "session.artifact.stored"
        );
        assert_eq!(
            SessionEventType::Custom("test".to_string()).to_event_type(),
            "session.custom.test"
        );
    }
}
