//! ABOUTME: Session and artifact management for rs-llmspell, providing persistent session state and artifact storage
//! ABOUTME: Integrates with Phase 5 state persistence, Phase 4 hooks, and Phase 3.3 storage infrastructure

#![warn(missing_docs)]
#![warn(clippy::all)]
#![warn(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]
#![allow(clippy::must_use_candidate)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::unused_self)]
#![allow(clippy::redundant_closure_for_method_calls)]
#![allow(clippy::uninlined_format_args)]
#![allow(clippy::map_unwrap_or)]
#![allow(clippy::unnecessary_wraps)]

//! # Session and Artifact Management
//!
//! This crate provides comprehensive session management capabilities for rs-llmspell,
//! including:
//!
//! - Session lifecycle management (create, save, restore, replay)
//! - Artifact storage with content-addressed deduplication
//! - Hook integration for session events
//! - Event correlation across session activities
//! - Lua bridge for script access to sessions
//!
//! ## Architecture
//!
//! The crate is organized into the following modules:
//!
//! - `manager`: Core session management orchestration
//! - `session`: Session types and state management
//! - `artifact`: Artifact storage and retrieval
//! - `replay`: Session replay engine
//! - `bridge`: Script language integration (Lua)
//! - `error`: Error types and handling
//! - `types`: Core type definitions

/// Error types for session operations
pub mod error;

/// Core type definitions
pub mod types;

/// Session management
pub mod session;

/// Configuration types
pub mod config;

/// Session manager orchestration
pub mod manager;

/// Artifact storage system
pub mod artifact;

/// Session replay engine
pub mod replay;

/// Script bridge implementations
pub mod bridge;

/// Hook integration
pub mod hooks;

/// Event system integration
pub mod events;

/// Session policies using hook patterns
pub mod policies;

/// Session middleware using pattern hooks
pub mod middleware;

/// Session analytics using MetricsHook
pub mod analytics;

/// Session security and isolation
pub mod security;

// Re-export commonly used types
pub use artifact::{
    ArtifactId, ArtifactQuery, ArtifactStorage, ArtifactStorageConfig, ArtifactStorageOps,
    ArtifactType, SessionArtifact,
};
pub use config::{SessionManagerConfig, SessionManagerConfigBuilder};
pub use error::{Result, SessionError};
pub use manager::SessionManager;
pub use session::Session;
pub use types::{
    CreateSessionOptions, CreateSessionOptionsBuilder, SessionConfig, SessionId, SessionMetadata,
    SessionQuery, SessionStatus,
};

/// Prelude module for convenient imports
pub mod prelude {
    pub use crate::{
        ArtifactId, ArtifactType, Result, Session, SessionArtifact, SessionConfig, SessionError,
        SessionId, SessionManager, SessionMetadata, SessionStatus,
    };
}
