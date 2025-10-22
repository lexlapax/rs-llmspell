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
//! ```rust,ignore
//! use llmspell_memory::prelude::*;
//!
//! // Create memory manager
//! let memory = DefaultMemoryManager::new_in_memory().await?;
//!
//! // Add episodic memory
//! memory.episodic().add(EpisodicEntry {
//!     session_id: "session-1".into(),
//!     role: "user".into(),
//!     content: "What is Rust?".into(),
//!     timestamp: Utc::now(),
//!     metadata: json!({}),
//! }).await?;
//!
//! // Search episodic memories
//! let results = memory.episodic().search("Rust", 5).await?;
//! ```

pub mod traits;
pub mod episodic;
pub mod semantic;
pub mod procedural;
pub mod consolidation;
pub mod manager;
pub mod types;
pub mod error;
pub mod prelude;

// Re-exports for convenience
pub use episodic::InMemoryEpisodicMemory;
pub use error::{MemoryError, Result};
pub use traits::{
    ConsolidationDecision, Entity, EpisodicMemory, MemoryManager, ProceduralMemory,
    Relationship, SemanticMemory,
};
pub use types::{ConsolidationMode, ConsolidationResult, EpisodicEntry};
