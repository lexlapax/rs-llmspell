//! Storage types
//!
//! Types for persistent storage including workflow state, session data,
//! and artifact storage structures.
//!
//! # Core Storage Types (Phase 13c.3)
//!
//! Domain types for storage backends migrated from scattered crates:
//! - Backend types: StorageBackendType, StorageCharacteristics
//! - Vector types: VectorEntry, VectorQuery, VectorResult, DistanceMetric, ScoringMethod
//! - Graph types: Entity, Relationship, TemporalQuery
//! - Procedural types: Pattern

pub mod artifact;
pub mod backend;
pub mod graph;
pub mod procedural;
pub mod session;
pub mod vector;
pub mod workflow;

// Re-export artifact types
pub use artifact::{Artifact, ArtifactId, ArtifactType, ContentHash, SessionStorageStats};

// Re-export backend types (Day 3)
// pub use backend::{StorageBackendType, StorageCharacteristics};

// Re-export graph types (Day 3)
// pub use graph::{Entity, Relationship, TemporalQuery};

// Re-export procedural types (Day 3)
// pub use procedural::Pattern;

// Re-export session types
pub use session::{SessionData, SessionStatus};

// Re-export vector types (Day 3)
// pub use vector::{VectorEntry, VectorQuery, VectorResult, DistanceMetric, ScoringMethod};

// Re-export workflow types
pub use workflow::{WorkflowState, WorkflowStatus};
