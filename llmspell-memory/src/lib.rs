//! # Adaptive Memory System for `LLMSpell`
//!
//! This crate provides a production-ready memory architecture with three types of memory:
//! - **Episodic**: Vector-indexed interaction history (recent conversations)
//! - **Semantic**: Bi-temporal knowledge graph (facts and relationships)
//! - **Procedural**: Learned patterns and skills (state transitions)
//!
//! ## Architecture
//!
//! ```text
//! MemoryManager
//! ├── EpisodicMemory (vector search via HNSW/ChromaDB/Qdrant)
//! ├── SemanticMemory (knowledge graph via SurrealDB/Neo4j)
//! └── ProceduralMemory (pattern storage)
//! ```
//!
//! ## Hot-Swappable Storage Backends
//!
//! All memory types support multiple storage backends through trait abstraction:
//! - HNSW (default, from llmspell-kernel)
//! - `ChromaDB` (external service)
//! - Qdrant (external service)
//! - `InMemory` (testing/development)
//!
//! ## Usage
//!
//! ```rust,no_run
//! use llmspell_memory::prelude::*;
//! use llmspell_memory::DefaultMemoryManager;
//!
//! #[tokio::main]
//! async fn main() -> Result<()> {
//!     // Create memory manager
//!     let memory = DefaultMemoryManager::new_in_memory().await?;
//!
//!     // Add episodic memory
//!     let entry = EpisodicEntry::new(
//!         "session-1".into(),
//!         "user".into(),
//!         "What is Rust?".into(),
//!     );
//!     memory.episodic().add(entry).await?;
//!
//!     // Search episodic memories
//!     let results = memory.episodic().search("Rust", 5).await?;
//!     Ok(())
//! }
//! ```

pub mod config;
pub mod consolidation;
pub mod embeddings;
pub mod episodic;
pub mod error;
pub mod manager;
pub mod prelude;
pub mod procedural;
pub mod semantic;
pub mod traits;
pub mod types;

// Re-exports for convenience
pub use config::{EpisodicBackendType, MemoryConfig};
pub use episodic::{EpisodicBackend, HNSWEpisodicMemory, InMemoryEpisodicMemory};
pub use error::{MemoryError, Result};
pub use manager::DefaultMemoryManager;
pub use procedural::{InMemoryPatternTracker, NoopProceduralMemory};
pub use semantic::GraphSemanticMemory;
pub use traits::{
    ConsolidationDecision, Entity, EpisodicMemory, MemoryManager, Pattern, ProceduralMemory,
    Relationship, SemanticMemory,
};
pub use types::{ConsolidationMode, ConsolidationResult, EpisodicEntry};
