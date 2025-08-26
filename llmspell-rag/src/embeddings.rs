//! Embedding generation and management with provider integration

pub mod cache;
pub mod dimensions;
pub mod factory;
pub mod provider;

// Re-export main types
pub use cache::{CacheConfig, EmbeddingCache};
pub use dimensions::{DimensionConfig, DimensionMapper};
pub use factory::{EmbeddingFactory, EmbeddingFactoryBuilder};
pub use provider::{
    EmbeddingModel, EmbeddingProvider, EmbeddingProviderConfig, EmbeddingProviderType,
    LateInteractionModel, TokenEmbeddings,
};
