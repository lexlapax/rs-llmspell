//! Common imports for knowledge graph

// Re-export error types
pub use crate::error::{GraphError, Result};

// Re-export traits
pub use crate::traits::KnowledgeGraph;

// Re-export types
pub use crate::types::{Entity, Relationship, TemporalQuery};

// Re-export storage backends
pub use crate::storage::surrealdb::SurrealDBBackend;
pub use crate::storage::GraphBackend;
