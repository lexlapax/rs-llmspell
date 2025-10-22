//! Episodic memory implementations (vector-indexed interaction history)
//!
//! This module provides multiple storage backends for episodic memory:
//! - `InMemoryEpisodicMemory` (default, simple and fast for <10k entries)
//! - `HnswEpisodicMemory` (future, using llmspell-storage for >10k entries)
//! - `ChromaDBEpisodicMemory` (future, optional external service)
//! - `QdrantEpisodicMemory` (future, optional external service)

pub mod in_memory;

pub use in_memory::InMemoryEpisodicMemory;
