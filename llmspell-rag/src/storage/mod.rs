//! Vector storage implementations
//!
//! Re-exports from llmspell-storage for backward compatibility

pub use llmspell_storage::backends::vector::{
    DimensionInfo, DimensionRouter, HNSWVectorStorage, IndexType, MetadataIndex,
    MetadataQueryOptimizer, QueryStats,
};
