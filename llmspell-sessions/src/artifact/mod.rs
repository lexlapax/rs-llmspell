//! ABOUTME: Artifact storage system for managing session outputs and data
//! ABOUTME: Provides content-addressed storage with metadata and versioning

pub mod session_artifact;
pub mod types;

pub use session_artifact::SessionArtifact;
pub use types::{
    ArtifactId, ArtifactMetadata, ArtifactType, ArtifactVersion, ContentHash,
    MAX_ARTIFACT_NAME_LENGTH, MAX_ARTIFACT_SIZE, MAX_TAG_LENGTH,
};

/// Artifact storage system for managing session artifacts
///
/// This is a placeholder that will be fully implemented in TASK-6.2.3
#[derive(Debug, Clone)]
pub struct ArtifactStorage;

impl ArtifactStorage {
    /// Create a new artifact storage instance
    pub fn new() -> Self {
        Self
    }
}

impl Default for ArtifactStorage {
    fn default() -> Self {
        Self::new()
    }
}
