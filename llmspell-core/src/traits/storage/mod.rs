//! Storage trait abstractions
//!
//! Provides trait abstractions for persistent storage components including
//! workflow state, session data, artifact storage, and core storage backends.
//!
//! These traits enable pluggable storage backends (SQLite, PostgreSQL, etc.)
//! while keeping domain logic backend-agnostic.
//!
//! # Core Storage Traits (Phase 13c.3)
//!
//! The following traits were migrated from scattered crates to provide
//! a single source of truth for all storage abstractions:
//!
//! - `StorageBackend`: Low-level key-value storage operations
//! - `VectorStorage`: Vector embedding storage with HNSW indexing
//! - `KnowledgeGraph`: Bi-temporal knowledge graph operations
//! - `ProceduralMemory`: State transition pattern learning

pub mod artifact;
pub mod backend;
pub mod graph;
pub mod procedural;
pub mod session;
pub mod vector;
pub mod workflow;

pub use artifact::ArtifactStorage;
pub use backend::StorageBackend;
pub use graph::KnowledgeGraph;
// Day 2 exports - remaining traits
// pub use procedural::ProceduralMemory;
pub use session::SessionStorage;
pub use vector::{HNSWStorage, VectorStorage};
pub use workflow::WorkflowStateStorage;
