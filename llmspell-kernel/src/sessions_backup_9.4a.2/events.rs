//! Session Event System
//!
//! Event tracking and correlation for session lifecycle and operations.
//! Supports event streaming, filtering, and analysis.

use super::SessionId;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::SystemTime;
use tracing::{debug, error, info, trace, warn, Level};

/// Session event types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum SessionEventType {
    /// Session was created
    Created,
    /// Session was paused
    Paused,
    /// Session was resumed
    Resumed,
    /// Session was archived
    Archived,
    /// Session expired due to TTL
    Expired,
    /// Message received by session
    MessageReceived,
    /// Code executed in session
    CodeExecuted,
    /// Artifact created
    ArtifactCreated,
    /// Artifact accessed
    ArtifactAccessed,
    /// Policy violation occurred
    PolicyViolation,
    /// Security event occurred
    SecurityEvent,
    /// Error occurred in session
    Error,
    /// Custom event
    Custom(String),
}

/// Session event with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionEvent {
    /// Event ID
    pub id: String,
    /// Session this event belongs to
    pub session_id: SessionId,
    /// Event type
    pub event_type: SessionEventType,
    /// Event timestamp
    pub timestamp: SystemTime,
    /// Event metadata
    pub metadata: HashMap<String, String>,
    /// Correlation ID for tracking related events
    pub correlation_id: Option<String>,
    /// User who triggered the event
    pub user_id: Option<String>,
}

impl SessionEvent {
    /// Create a new session event
    pub fn new(session_id: SessionId, event_type: SessionEventType) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            session_id,
            event_type,
            timestamp: SystemTime::now(),
            metadata: HashMap::new(),
            correlation_id: None,
            user_id: None,
        }
    }

    /// Add metadata to the event
    #[must_use]
    pub fn with_metadata(mut self, key: String, value: String) -> Self {
        self.metadata.insert(key, value);
        self
    }

    /// Set correlation ID for event tracking
    #[must_use]
    pub fn with_correlation_id(mut self, correlation_id: String) -> Self {
        self.correlation_id = Some(correlation_id);
        self
    }

    /// Set user ID for the event
    #[must_use]
    pub fn with_user_id(mut self, user_id: String) -> Self {
        self.user_id = Some(user_id);
        self
    }
}

/// Event handler trait
pub trait EventHandler: Send + Sync {
    /// Handle a session event
    fn handle(&self, event: &SessionEvent);

    /// Get handler name
    fn name(&self) -> &str;
}

/// Event bus for session events
pub struct SessionEventBus {
    /// Registered event handlers
    handlers: Vec<Arc<dyn EventHandler>>,
    /// Event filters
    filters: Vec<Arc<dyn EventFilter>>,
}

impl Default for SessionEventBus {
    fn default() -> Self {
        Self::new()
    }
}

impl SessionEventBus {
    /// Create a new event bus
    pub fn new() -> Self {
        Self {
            handlers: Vec::new(),
            filters: Vec::new(),
        }
    }

    /// Register an event handler
    pub fn register_handler(&mut self, handler: Arc<dyn EventHandler>) {
        debug!("Registering event handler: {}", handler.name());
        self.handlers.push(handler);
    }

    /// Register an event filter
    pub fn register_filter(&mut self, filter: Arc<dyn EventFilter>) {
        debug!("Registering event filter: {}", filter.name());
        self.filters.push(filter);
    }

    /// Emit an event to all handlers
    pub fn emit(&self, event: &SessionEvent) {
        // Apply filters
        for filter in &self.filters {
            if !filter.should_process(event) {
                debug!(
                    "Event filtered out by {}: {:?}",
                    filter.name(),
                    event.event_type
                );
                return;
            }
        }

        // Send to handlers
        for handler in &self.handlers {
            handler.handle(event);
        }

        debug!(
            "Event emitted: {:?} for session {}",
            event.event_type, event.session_id
        );
    }
}

/// Event filter trait
pub trait EventFilter: Send + Sync {
    /// Check if event should be processed
    fn should_process(&self, event: &SessionEvent) -> bool;

    /// Get filter name
    fn name(&self) -> &str;
}

/// Event type filter
pub struct EventTypeFilter {
    /// Allowed event types
    allowed_types: Vec<SessionEventType>,
}

impl EventTypeFilter {
    /// Create new event type filter
    pub fn new(allowed_types: Vec<SessionEventType>) -> Self {
        Self { allowed_types }
    }
}

impl EventFilter for EventTypeFilter {
    fn should_process(&self, event: &SessionEvent) -> bool {
        self.allowed_types.contains(&event.event_type)
    }

    fn name(&self) -> &'static str {
        "EventTypeFilter"
    }
}

/// Session filter - only events for specific sessions
pub struct SessionFilter {
    /// Allowed session IDs
    allowed_sessions: Vec<SessionId>,
}

impl SessionFilter {
    /// Create new session filter
    pub fn new(allowed_sessions: Vec<SessionId>) -> Self {
        Self { allowed_sessions }
    }
}

impl EventFilter for SessionFilter {
    fn should_process(&self, event: &SessionEvent) -> bool {
        self.allowed_sessions.contains(&event.session_id)
    }

    fn name(&self) -> &'static str {
        "SessionFilter"
    }
}

/// Logging event handler
pub struct LoggingEventHandler {
    /// Log level for events
    log_level: tracing::Level,
}

impl LoggingEventHandler {
    /// Create new logging handler
    pub fn new(log_level: tracing::Level) -> Self {
        Self { log_level }
    }
}

impl EventHandler for LoggingEventHandler {
    fn handle(&self, event: &SessionEvent) {
        match self.log_level {
            Level::ERROR => error!("Session event: {:?}", event),
            Level::WARN => warn!("Session event: {:?}", event),
            Level::INFO => info!("Session event: {:?}", event),
            Level::DEBUG => debug!("Session event: {:?}", event),
            Level::TRACE => trace!("Session event: {:?}", event),
        }
    }

    fn name(&self) -> &'static str {
        "LoggingEventHandler"
    }
}

/// Event store for persistence
pub struct EventStore {
    /// Stored events (in-memory for now)
    events: Arc<parking_lot::RwLock<Vec<SessionEvent>>>,
    /// Maximum events to store
    max_events: usize,
}

impl EventStore {
    /// Create new event store
    pub fn new(max_events: usize) -> Self {
        Self {
            events: Arc::new(parking_lot::RwLock::new(Vec::new())),
            max_events,
        }
    }

    /// Store an event
    pub fn store(&self, event: SessionEvent) {
        let mut events = self.events.write();
        events.push(event);

        // Limit size
        let len = events.len();
        if len > self.max_events {
            events.drain(0..len - self.max_events);
        }
    }

    /// Get events for a session
    pub fn get_events(&self, session_id: &SessionId) -> Vec<SessionEvent> {
        self.events
            .read()
            .iter()
            .filter(|e| &e.session_id == session_id)
            .cloned()
            .collect()
    }

    /// Get events by type
    pub fn get_events_by_type(&self, event_type: &SessionEventType) -> Vec<SessionEvent> {
        self.events
            .read()
            .iter()
            .filter(|e| &e.event_type == event_type)
            .cloned()
            .collect()
    }

    /// Get all events
    pub fn get_all_events(&self) -> Vec<SessionEvent> {
        self.events.read().clone()
    }

    /// Clear all events
    pub fn clear(&self) {
        self.events.write().clear();
    }
}

impl EventHandler for EventStore {
    fn handle(&self, event: &SessionEvent) {
        self.store(event.clone());
    }

    fn name(&self) -> &'static str {
        "EventStore"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_session_event_creation() {
        let session_id = SessionId::new();
        let event = SessionEvent::new(session_id.clone(), SessionEventType::Created)
            .with_metadata("test_key".to_string(), "test_value".to_string())
            .with_correlation_id("test-correlation".to_string())
            .with_user_id("user123".to_string());

        assert_eq!(event.session_id, session_id);
        assert_eq!(event.event_type, SessionEventType::Created);
        assert_eq!(
            event.metadata.get("test_key"),
            Some(&"test_value".to_string())
        );
        assert_eq!(event.correlation_id, Some("test-correlation".to_string()));
        assert_eq!(event.user_id, Some("user123".to_string()));
    }

    #[test]
    fn test_event_bus() {
        let mut bus = SessionEventBus::new();
        let store = Arc::new(EventStore::new(100));
        bus.register_handler(store.clone());

        let session_id = SessionId::new();
        let event = SessionEvent::new(session_id.clone(), SessionEventType::Created);

        bus.emit(&event);

        let stored_events = store.get_events(&session_id);
        assert_eq!(stored_events.len(), 1);
        assert_eq!(stored_events[0].event_type, SessionEventType::Created);
    }

    #[test]
    fn test_event_filters() {
        let mut bus = SessionEventBus::new();
        let store = Arc::new(EventStore::new(100));

        // Only allow Created events
        let filter = Arc::new(EventTypeFilter::new(vec![SessionEventType::Created]));
        bus.register_filter(filter);
        bus.register_handler(store.clone());

        let session_id = SessionId::new();

        // This should be stored
        let created_event = SessionEvent::new(session_id.clone(), SessionEventType::Created);
        bus.emit(&created_event);

        // This should be filtered out
        let paused_event = SessionEvent::new(session_id.clone(), SessionEventType::Paused);
        bus.emit(&paused_event);

        let stored_events = store.get_events(&session_id);
        assert_eq!(stored_events.len(), 1);
        assert_eq!(stored_events[0].event_type, SessionEventType::Created);
    }

    #[test]
    fn test_event_store() {
        let store = EventStore::new(2);
        let session_id = SessionId::new();

        // Add 3 events, should only keep last 2
        store.store(SessionEvent::new(
            session_id.clone(),
            SessionEventType::Created,
        ));
        store.store(SessionEvent::new(
            session_id.clone(),
            SessionEventType::Paused,
        ));
        store.store(SessionEvent::new(
            session_id.clone(),
            SessionEventType::Resumed,
        ));

        let events = store.get_all_events();
        assert_eq!(events.len(), 2);
        assert_eq!(events[0].event_type, SessionEventType::Paused);
        assert_eq!(events[1].event_type, SessionEventType::Resumed);
    }
}
