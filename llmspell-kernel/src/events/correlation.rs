//! Event Correlation System for Kernel Operations
//!
//! Provides distributed tracing, event correlation, and `IOPub` broadcasting
//! for comprehensive kernel event management across multiple clients.

use crate::debug::coordinator::DebugEvent;
use crate::io::manager::IOPubMessage;
use crate::io::router::{MessageDestination, MessageRouter};
use crate::sessions::events::SessionEvent as KernelSessionEvent;
use crate::sessions::events::SessionEvent as SessionsSessionEvent;
use anyhow::Result;
use llmspell_events::{
    correlation::{CorrelationContext, EventCorrelationTracker, EventLink, EventRelationship},
    universal_event::{Language, UniversalEvent},
    EventBus,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, instrument, warn};
use uuid::Uuid;

/// Execution status for execute requests/replies
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ExecutionStatus {
    /// Execution completed successfully
    Ok,
    /// Execution failed with error
    Error,
    /// Execution was aborted
    Aborted,
}

/// Kernel-wide event types encompassing all kernel operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum KernelEvent {
    /// Code execution request received
    ExecuteRequest {
        /// Code to execute
        code: String,
        /// Message ID for correlation
        msg_id: String,
        /// Session ID
        session_id: String,
        /// Silent execution flag
        silent: bool,
        /// User expressions to evaluate
        user_expressions: HashMap<String, String>,
    },
    /// Code execution reply sent
    ExecuteReply {
        /// Execution status
        status: ExecutionStatus,
        /// Message ID for correlation
        msg_id: String,
        /// Session ID
        session_id: String,
        /// Execution count
        execution_count: u64,
        /// Error information if status is Error
        error_info: Option<ErrorInfo>,
    },
    /// Debug event from debug coordinator
    DebugEvent(DebugEvent),
    /// Session event from sessions crate
    SessionEvent(Box<SessionsSessionEvent>),
    /// Kernel session event (local to kernel)
    KernelSessionEvent(Box<KernelSessionEvent>),
    /// Kernel startup event
    KernelStartup {
        /// Kernel ID
        kernel_id: String,
        /// Protocol version
        protocol_version: String,
        /// Language information
        language_info: LanguageInfo,
    },
    /// Kernel shutdown event
    KernelShutdown {
        /// Restart flag
        restart: bool,
        /// Reason for shutdown
        reason: String,
    },
    /// Kernel status change
    StatusChange {
        /// New execution state
        execution_state: ExecutionState,
        /// Previous state
        previous_state: Option<ExecutionState>,
    },
    /// Custom kernel event
    Custom {
        /// Event name
        name: String,
        /// Event data
        data: Value,
        /// Event source component
        source: String,
    },
    /// Raw `IOPub` message for bridging
    IOPubMessage(IOPubMessage),
}

/// Error information for failed executions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorInfo {
    /// Error name
    pub ename: String,
    /// Error value/message
    pub evalue: String,
    /// Traceback lines
    pub traceback: Vec<String>,
}

/// Language information for kernel
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LanguageInfo {
    /// Language name
    pub name: String,
    /// Language version
    pub version: String,
    /// Mimetype for code
    pub mimetype: String,
    /// File extension
    pub file_extension: String,
    /// Pygments lexer
    pub pygments_lexer: String,
    /// Code mirror mode
    pub codemirror_mode: String,
}

/// Kernel execution state
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ExecutionState {
    /// Kernel is idle
    Idle,
    /// Kernel is busy executing
    Busy,
    /// Kernel is starting up
    Starting,
    /// Kernel is shutting down
    Terminating,
}

impl KernelEvent {
    /// Get the event type as a string for correlation
    pub fn event_type(&self) -> String {
        match self {
            Self::ExecuteRequest { .. } => "kernel.execute_request".to_string(),
            Self::ExecuteReply { .. } => "kernel.execute_reply".to_string(),
            Self::DebugEvent(_debug_event) => "kernel.debug.event".to_string(),
            Self::SessionEvent(session_event) => {
                format!("kernel.session.{}", session_event.event.event_type)
            }
            Self::KernelSessionEvent(kernel_event) => {
                format!("kernel.kernel_session.{:?}", kernel_event.event_type).to_lowercase()
            }
            Self::KernelStartup { .. } => "kernel.startup".to_string(),
            Self::KernelShutdown { .. } => "kernel.shutdown".to_string(),
            Self::StatusChange { .. } => "kernel.status_change".to_string(),
            Self::Custom { name, .. } => format!("kernel.custom.{name}"),
            Self::IOPubMessage(msg) => format!("kernel.iopub.{}", msg.header.msg_type),
        }
    }

    /// Get the message ID for correlation if available
    pub fn message_id(&self) -> Option<String> {
        match self {
            Self::ExecuteRequest { msg_id, .. } | Self::ExecuteReply { msg_id, .. } => {
                Some(msg_id.clone())
            }
            Self::DebugEvent(_debug_event) => None,
            Self::SessionEvent(session_event) => Some(session_event.event.id.to_string()),
            Self::KernelSessionEvent(kernel_event) => Some(kernel_event.event.id.to_string()),
            _ => None,
        }
    }

    /// Get the session ID if available
    pub fn session_id(&self) -> Option<String> {
        match self {
            Self::ExecuteRequest { session_id, .. } | Self::ExecuteReply { session_id, .. } => {
                Some(session_id.clone())
            }
            Self::DebugEvent(_debug_event) => None,
            Self::SessionEvent(session_event) => Some(session_event.session_id.to_string()),
            Self::KernelSessionEvent(kernel_event) => Some(kernel_event.session_id.to_string()),
            Self::IOPubMessage(msg) => Some(msg.header.session.clone()),
            _ => None,
        }
    }

    /// Convert to `UniversalEvent` for correlation tracking
    pub fn to_universal_event(&self, correlation_id: Uuid) -> UniversalEvent {
        let event_data = match self {
            Self::ExecuteRequest {
                code,
                msg_id,
                session_id,
                silent,
                user_expressions,
            } => json!({
                "code": code,
                "msg_id": msg_id,
                "session_id": session_id,
                "silent": silent,
                "user_expressions": user_expressions
            }),
            Self::ExecuteReply {
                status,
                msg_id,
                session_id,
                execution_count,
                error_info,
            } => json!({
                "status": status,
                "msg_id": msg_id,
                "session_id": session_id,
                "execution_count": execution_count,
                "error_info": error_info
            }),
            Self::DebugEvent(debug_event) => {
                // Convert debug event to JSON
                serde_json::to_value(debug_event).unwrap_or(Value::Null)
            }
            Self::SessionEvent(session_event) => {
                // Use the session event's data
                session_event.event.data.clone()
            }
            Self::KernelSessionEvent(kernel_event) => json!({
                "event_type": kernel_event.event_type,
                "session_id": kernel_event.session_id.to_string(),
                "metadata": kernel_event.event.metadata
            }),
            Self::KernelStartup {
                kernel_id,
                protocol_version,
                language_info,
            } => json!({
                "kernel_id": kernel_id,
                "protocol_version": protocol_version,
                "language_info": language_info
            }),
            Self::KernelShutdown { restart, reason } => json!({
                "restart": restart,
                "reason": reason
            }),
            Self::StatusChange {
                execution_state,
                previous_state,
            } => json!({
                "execution_state": execution_state,
                "previous_state": previous_state
            }),
            Self::Custom { name, data, source } => json!({
                "name": name,
                "data": data,
                "source": source
            }),
            Self::IOPubMessage(msg) => json!({
                "header": msg.header,
                "parent_header": msg.parent_header,
                "metadata": msg.metadata,
                "content": msg.content
            }),
        };

        UniversalEvent::new(self.event_type(), event_data, Language::Rust)
            .with_correlation_id(correlation_id)
            .with_source("kernel")
    }

    /// Convert to `IOPub` message for broadcasting
    pub fn to_iopub_message(&self, session: Option<&str>) -> IOPubMessage {
        use crate::io::manager::MessageHeader;

        let msg_type = match self {
            Self::ExecuteRequest { .. } => "execute_request",
            Self::ExecuteReply { .. } => "execute_reply",
            Self::DebugEvent(_) => "debug_event",
            Self::SessionEvent(_) => "session_event",
            Self::KernelSessionEvent(_) => "kernel_session_event",
            Self::KernelStartup { .. } => "kernel_started",
            Self::KernelShutdown { .. } => "kernel_shutdown",
            Self::StatusChange { .. } => "status",
            Self::Custom { .. } => "kernel_custom",
            Self::IOPubMessage(msg) => return msg.clone(),
        };

        let session_str = session.unwrap_or("default");
        let header = MessageHeader::new(msg_type, session_str);
        let mut content = HashMap::new();

        match self {
            Self::ExecuteRequest {
                code,
                msg_id,
                silent,
                user_expressions,
                ..
            } => {
                content.insert("code".to_string(), json!(code));
                content.insert("msg_id".to_string(), json!(msg_id));
                content.insert("silent".to_string(), json!(silent));
                content.insert("user_expressions".to_string(), json!(user_expressions));
            }
            Self::ExecuteReply {
                status,
                msg_id,
                execution_count,
                error_info,
                ..
            } => {
                content.insert("status".to_string(), json!(status));
                content.insert("msg_id".to_string(), json!(msg_id));
                content.insert("execution_count".to_string(), json!(execution_count));
                if let Some(error) = error_info {
                    content.insert("error_info".to_string(), json!(error));
                }
            }
            Self::StatusChange {
                execution_state, ..
            } => {
                content.insert("execution_state".to_string(), json!(execution_state));
            }
            Self::KernelStartup {
                kernel_id,
                protocol_version,
                language_info,
            } => {
                content.insert("kernel_id".to_string(), json!(kernel_id));
                content.insert("protocol_version".to_string(), json!(protocol_version));
                content.insert("language_info".to_string(), json!(language_info));
            }
            Self::KernelShutdown { restart, reason } => {
                content.insert("restart".to_string(), json!(restart));
                content.insert("reason".to_string(), json!(reason));
            }
            _ => {
                // For other events, use generic serialization
                if let Ok(serialized) = serde_json::to_value(self) {
                    content.insert("event".to_string(), serialized);
                }
            }
        }

        IOPubMessage {
            parent_header: None,
            header,
            metadata: HashMap::new(),
            content,
        }
    }
}

/// Event broadcaster for multi-client `IOPub` support
pub struct EventBroadcaster {
    /// Message router for multi-client support
    message_router: Arc<MessageRouter>,
    /// Default session for messages
    default_session: String,
}

impl EventBroadcaster {
    /// Create new event broadcaster
    pub fn new(message_router: Arc<MessageRouter>, default_session: String) -> Self {
        Self {
            message_router,
            default_session,
        }
    }

    /// Broadcast a kernel event to all connected clients
    ///
    /// # Errors
    ///
    /// Returns an error if message routing fails
    #[instrument(level = "debug", skip(self))]
    pub async fn broadcast(&self, event: &KernelEvent) -> Result<()> {
        let session_opt = event.session_id();
        let session = session_opt.as_deref().unwrap_or(&self.default_session);
        let iopub_msg = event.to_iopub_message(Some(session));

        debug!("Broadcasting event {} to all clients", event.event_type());

        self.message_router
            .route_message(iopub_msg, MessageDestination::Broadcast)
            .await?;

        Ok(())
    }

    /// Send event to specific client
    ///
    /// # Errors
    ///
    /// Returns an error if message routing fails
    #[instrument(level = "debug", skip(self))]
    pub async fn send_to_client(&self, event: &KernelEvent, client_id: &str) -> Result<()> {
        let session_opt = event.session_id();
        let session = session_opt.as_deref().unwrap_or(&self.default_session);
        let iopub_msg = event.to_iopub_message(Some(session));

        debug!(
            "Sending event {} to client {}",
            event.event_type(),
            client_id
        );

        self.message_router
            .route_message(iopub_msg, MessageDestination::Client(client_id.to_string()))
            .await?;

        Ok(())
    }
}

/// Kernel event correlator combining event tracking and broadcasting
pub struct KernelEventCorrelator {
    /// Event correlation tracker from llmspell-events
    correlation_tracker: EventCorrelationTracker,
    /// Event bus for pattern-based subscriptions
    event_bus: EventBus,
    /// Event broadcaster for `IOPub` support
    broadcaster: EventBroadcaster,
    /// Current execution context for correlation
    execution_context: Arc<RwLock<Option<CorrelationContext>>>,
    /// Event sequence counter
    sequence_counter: Arc<RwLock<u64>>,
}

impl KernelEventCorrelator {
    /// Create new kernel event correlator
    pub fn new(
        message_router: Arc<MessageRouter>,
        default_session: String,
        event_bus: Option<EventBus>,
    ) -> Self {
        let correlation_tracker = EventCorrelationTracker::with_default_config();
        let event_bus = event_bus.unwrap_or_default();
        let broadcaster = EventBroadcaster::new(message_router, default_session);

        Self {
            correlation_tracker,
            event_bus,
            broadcaster,
            execution_context: Arc::new(RwLock::new(None)),
            sequence_counter: Arc::new(RwLock::new(0)),
        }
    }

    /// Start a new execution context
    #[instrument(level = "debug", skip(self))]
    pub async fn start_execution_context(&self, msg_id: &str, session_id: &str) {
        let context = CorrelationContext::new_root()
            .with_metadata("msg_id", msg_id)
            .with_metadata("session_id", session_id)
            .with_tag("execution");

        *self.execution_context.write().await = Some(context.clone());
        self.correlation_tracker.add_context(context);

        info!(
            "Started execution context for msg_id={}, session_id={}",
            msg_id, session_id
        );
    }

    /// End the current execution context
    #[instrument(level = "debug", skip(self))]
    pub async fn end_execution_context(&self) {
        *self.execution_context.write().await = None;
        debug!("Ended execution context");
    }

    /// Track and broadcast a kernel event
    ///
    /// # Errors
    ///
    /// Returns an error if event tracking, publishing, or broadcasting fails
    #[instrument(level = "debug", skip(self, event))]
    pub async fn track_event(&self, event: KernelEvent) -> Result<()> {
        // Get or create correlation context
        let correlation_context = {
            let current_context = self.execution_context.read().await;
            if let Some(context) = current_context.as_ref() {
                context.create_child()
            } else {
                // Create new root context for events outside execution
                CorrelationContext::new_root()
                    .with_metadata("event_type", event.event_type())
                    .with_tag("kernel_event")
            }
        };

        // Convert to UniversalEvent and track
        let universal_event = event.to_universal_event(correlation_context.correlation_id);

        self.correlation_tracker
            .track_event(universal_event.clone());

        // Publish to event bus
        self.event_bus
            .publish(universal_event)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to publish event: {e}"))?;

        // Broadcast to IOPub
        self.broadcaster.broadcast(&event).await?;

        // Increment sequence counter
        {
            let mut counter = self.sequence_counter.write().await;
            *counter += 1;
        }

        debug!(
            "Tracked event: {} with correlation_id={}",
            event.event_type(),
            correlation_context.correlation_id
        );

        Ok(())
    }

    /// Link two events with a relationship
    #[instrument(level = "debug", skip(self))]
    pub fn link_events(
        &self,
        from_event_id: Uuid,
        to_event_id: Uuid,
        relationship: &EventRelationship,
    ) {
        let link = EventLink::new(from_event_id, to_event_id, relationship.clone())
            .with_metadata("source", "kernel_correlator");

        self.correlation_tracker.add_link(link);

        debug!(
            "Linked events: {} -> {} ({:?})",
            from_event_id, to_event_id, relationship
        );
    }

    /// Get events for a correlation ID
    pub fn get_correlated_events(&self, correlation_id: &Uuid) -> Vec<UniversalEvent> {
        self.correlation_tracker.get_events(correlation_id)
    }

    /// Get correlation statistics
    pub fn get_correlation_stats(&self) -> llmspell_events::correlation::CorrelationStats {
        self.correlation_tracker.get_stats()
    }

    /// Subscribe to events with pattern
    ///
    /// # Errors
    ///
    /// Returns an error if event subscription fails
    pub async fn subscribe_to_events(
        &self,
        pattern: &str,
    ) -> Result<tokio::sync::mpsc::UnboundedReceiver<UniversalEvent>> {
        self.event_bus
            .subscribe(pattern)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to subscribe to events: {e}"))
    }

    /// Clear all correlation data
    pub fn clear_correlations(&self) {
        self.correlation_tracker.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::io::router::MessageRouter;

    #[tokio::test]
    async fn test_kernel_event_creation() {
        let event = KernelEvent::ExecuteRequest {
            code: "print('hello')".to_string(),
            msg_id: "test-msg-123".to_string(),
            session_id: "test-session".to_string(),
            silent: false,
            user_expressions: HashMap::new(),
        };

        assert_eq!(event.event_type(), "kernel.execute_request");
        assert_eq!(event.message_id(), Some("test-msg-123".to_string()));
        assert_eq!(event.session_id(), Some("test-session".to_string()));
    }

    #[tokio::test]
    async fn test_event_correlator() {
        let router = Arc::new(MessageRouter::new(100));
        let correlator = KernelEventCorrelator::new(router, "test-session".to_string(), None);

        // Start execution context
        correlator
            .start_execution_context("msg-123", "session-456")
            .await;

        // Track an event
        let event = KernelEvent::StatusChange {
            execution_state: ExecutionState::Busy,
            previous_state: Some(ExecutionState::Idle),
        };

        correlator.track_event(event).await.unwrap();

        // Check stats
        let stats = correlator.get_correlation_stats();
        assert!(stats.total_events > 0);

        // End execution context
        correlator.end_execution_context().await;
    }

    #[test]
    fn test_universal_event_conversion() {
        let event = KernelEvent::ExecuteRequest {
            code: "test_code".to_string(),
            msg_id: "msg-123".to_string(),
            session_id: "session-456".to_string(),
            silent: false,
            user_expressions: HashMap::new(),
        };

        let correlation_id = Uuid::new_v4();
        let universal_event = event.to_universal_event(correlation_id);

        assert_eq!(universal_event.event_type, "kernel.execute_request");
        assert_eq!(universal_event.metadata.correlation_id, correlation_id);
        assert_eq!(universal_event.metadata.source, Some("kernel".to_string()));
    }

    #[test]
    fn test_iopub_message_conversion() {
        let event = KernelEvent::StatusChange {
            execution_state: ExecutionState::Busy,
            previous_state: Some(ExecutionState::Idle),
        };

        let iopub_msg = event.to_iopub_message(Some("test-session"));

        assert_eq!(iopub_msg.header.msg_type, "status");
        assert_eq!(iopub_msg.header.session, "test-session");
        assert!(iopub_msg.content.contains_key("execution_state"));
    }
}
