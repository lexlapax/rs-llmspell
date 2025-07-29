//! ABOUTME: Hook integration module for session management
//! ABOUTME: Provides artifact collectors and other session-specific hooks

pub mod collectors;

pub use collectors::{
    process_collected_artifact, register_artifact_collectors, ArtifactCollectionProcessor,
    CollectorConfig,
};
