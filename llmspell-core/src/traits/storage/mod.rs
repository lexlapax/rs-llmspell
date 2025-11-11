//! Storage trait abstractions
//!
//! Provides trait abstractions for persistent storage components including
//! workflow state, session data, and artifact storage.
//!
//! These traits enable pluggable storage backends (SQLite, PostgreSQL, etc.)
//! while keeping domain logic backend-agnostic.

pub mod artifact;
pub mod session;
pub mod workflow;

pub use artifact::ArtifactStorage;
pub use session::SessionStorage;
pub use workflow::WorkflowStateStorage;
