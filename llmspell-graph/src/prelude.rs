//! Common imports for knowledge graph

// Re-export error types
pub use crate::error::{GraphError, Result};

// Re-export traits from llmspell-core
pub use llmspell_core::traits::storage::KnowledgeGraph;

// Re-export types from llmspell-core
pub use llmspell_core::types::storage::{Entity, Relationship, TemporalQuery};

// Re-export domain-specific storage backend trait
pub use crate::storage::GraphBackend;
// Note: SQLite-based graph storage implemented - use SQLite or PostgreSQL storage via llmspell-storage
