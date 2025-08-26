//! Trait definitions for vector storage and retrieval

pub mod hnsw;
pub mod hybrid;
pub mod storage;

// Re-export commonly used traits
pub use hnsw::{DistanceMetric, HNSWConfig, HNSWStorage, NamespaceStats};
pub use hybrid::{
    ComponentScores, HybridQuery, HybridResult, HybridStorage, RerankingStrategy,
    RetrievalStrategy, RetrievalWeights,
};
pub use storage::{
    ScopedStats, StorageStats, VectorEntry, VectorQuery, VectorResult, VectorStorage,
};
