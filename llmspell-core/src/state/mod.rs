//! ABOUTME: State management types and structures for llmspell-core
//! ABOUTME: Provides artifact correlation and state-related core types

pub mod artifact_correlation;

pub use artifact_correlation::{
    ArtifactCorrelation, ArtifactCorrelationManager, ArtifactId, ArtifactMetadata,
    ArtifactRelationship, StateOperation,
};
