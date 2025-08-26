//! Core trait definitions for vector storage and retrieval

pub mod hnsw;
pub mod hybrid;
pub mod storage;

// Re-export main traits
pub use hnsw::{HNSWConfig, HNSWStorage};
pub use hybrid::{HybridQuery, HybridResult, HybridStorage, RetrievalWeights};
pub use storage::{
    ScopedStats, StorageStats, VectorEntry, VectorQuery, VectorResult, VectorStorage,
};
