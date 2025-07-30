//! ABOUTME: Hook integration module for session management
//! ABOUTME: Provides artifact collectors and session-specific hook context extensions

pub mod collectors;
pub mod context_extensions;

pub use collectors::{
    process_collected_artifact, register_artifact_collectors, ArtifactCollectionProcessor,
    CollectorConfig,
};
pub use context_extensions::{SessionHookContextExt, SessionHookContextHelper};
