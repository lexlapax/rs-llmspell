//! ABOUTME: Artifact storage system for managing session outputs and data
//! ABOUTME: Provides content-addressed storage with metadata and versioning

pub mod access;
pub mod metadata;
pub mod search;
pub mod session_artifact;
pub mod storage;
pub mod types;
pub mod versioning;

pub use search::{ArtifactSearch, ArtifactSearchQuery, SearchResult, SortOrder};
pub use session_artifact::SessionArtifact;
pub use storage::{
    ArtifactQuery, ArtifactStorage, ArtifactStorageConfig, ArtifactStorageOps, SessionStorageStats,
};
pub use types::{
    ArtifactId, ArtifactMetadata, ArtifactType, ArtifactVersion, ContentHash,
    MAX_ARTIFACT_NAME_LENGTH, MAX_ARTIFACT_SIZE, MAX_TAG_LENGTH,
};
