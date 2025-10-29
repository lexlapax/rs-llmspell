//! Document chunking strategies for RAG

pub mod strategies;
pub mod tokenizer;

#[cfg(feature = "memory-aware")]
pub mod memory_aware;

pub use strategies::{
    ChunkingConfig, ChunkingStrategy, DocumentChunk, SemanticChunker, SlidingWindowChunker,
};
pub use tokenizer::{TiktokenCounter, TokenCounter};

#[cfg(feature = "memory-aware")]
pub use memory_aware::MemoryAwareChunker;
