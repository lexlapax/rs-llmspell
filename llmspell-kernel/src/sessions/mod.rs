//! ABOUTME: Session and artifact management within kernel, providing persistent session state and artifact storage
//! ABOUTME: Integrates with Phase 5 state persistence, Phase 4 hooks, and Phase 3.3 storage infrastructure

//! # Session and Artifact Management
//!
//! This module provides comprehensive session management capabilities for rs-llmspell,
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
//! The module is organized into the following submodules:
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
    SessionStatus,
};

/// Prelude module for convenient imports
pub mod prelude {
    pub use crate::sessions::{
        ArtifactId, ArtifactType, Result, Session, SessionArtifact, SessionConfig, SessionError,
        SessionId, SessionManager, SessionMetadata, SessionStatus,
    };
}
