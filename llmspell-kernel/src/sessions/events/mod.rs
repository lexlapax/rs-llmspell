//! ABOUTME: Session event system with correlation tracking integration
//! ABOUTME: Provides session-specific event types and correlation utilities

pub mod session_events;

pub use session_events::{
    create_correlated_event, create_session_event, SessionEvent, SessionEventType,
    SESSION_EVENT_PREFIX,
};
