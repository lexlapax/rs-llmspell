//! # llmspell-rag
//!
//! Vector storage and RAG (Retrieval-Augmented Generation) infrastructure for llmspell.
//!
//! This crate provides:
//! - Multi-tenant vector storage with namespace isolation
//! - Multi-provider embedding support (256-4096 dimensions)
//! - HNSW-based vector search with <10ms retrieval for 1M vectors
//! - State and session integration with proper scoping
//! - Security policies with RLS-style access control
//! - Hybrid retrieval combining vector, keyword, and metadata search
//!
//! ## Architecture
//!
//! The crate is organized into the following modules:
//! - `traits`: Core trait definitions for vector storage and retrieval
//! - `storage`: Vector storage implementations (HNSW, dimension routing)
//! - `embeddings`: Embedding generation with provider integration
//! - `pipeline`: RAG pipeline orchestration
//! - `multi_tenant`: Multi-tenant isolation and usage tracking
//! - `state_integration`: Integration with `StateManager` for scope-aware storage
//! - `session_integration`: Session-aware RAG with artifact storage
//! - `security`: Access policies and RLS enforcement
//! - `chunking`: Document chunking strategies
//! - `retrieval`: Hybrid retrieval implementations
//!
//! ## Usage
//!
//! ```rust,no_run
//! use llmspell_rag::prelude::*;
//! use std::collections::HashMap;
//!
//! # async fn example() -> anyhow::Result<()> {
//! // Create a vector entry with multi-tenant support
//! let entry = VectorEntry::new("doc-1".to_string(), vec![0.1, 0.2, 0.3])
//!     .with_scope(StateScope::Custom("tenant:tenant-123".to_string()))
//!     .with_metadata(HashMap::from([(
//!         "source".to_string(),
//!         serde_json::Value::String("document.txt".to_string()),
//!     )]));
//!
//! // Create a scoped vector query
//! let query = VectorQuery::new(vec![0.1, 0.2, 0.3], 10)
//!     .with_scope(StateScope::Custom("tenant:tenant-123".to_string()))
//!     .with_threshold(0.8);
//!
//! // Configure HNSW for optimal performance
//! let hnsw_config = HNSWConfig::balanced();
//!
//! // Create embedding factory for provider integration
//! let factory = EmbeddingFactoryBuilder::new()
//!     .with_config(EmbeddingProviderConfig::default())
//!     .build();
//! # Ok(())
//! # }
//! ```

#![warn(missing_docs)]
#![warn(missing_debug_implementations)]

/// Core trait definitions for hybrid retrieval
pub mod traits;

/// Embedding generation and management
pub mod embeddings;

/// RAG pipeline orchestration
pub mod pipeline;

// Multi-tenant functionality moved to llmspell-tenancy crate

/// State persistence integration
pub mod state_integration;

/// Session management integration
pub mod session_integration;

// Security functionality moved to llmspell-security crate

/// Document chunking strategies
pub mod chunking;

/// Retrieval implementations
pub mod retrieval;

/// Multi-tenant RAG integration
pub mod multi_tenant_integration;

/// Prelude for convenient imports
pub mod prelude {

    // Hybrid retrieval traits
    pub use crate::traits::hybrid::{
        ComponentScores, HybridQuery, HybridResult, HybridStorage, RerankingStrategy,
        RetrievalStrategy, RetrievalWeights,
    };

    // Re-export commonly used types from llmspell-core
    pub use llmspell_core::traits::storage::{HNSWStorage, VectorStorage};
    pub use llmspell_core::types::storage::{
        DistanceMetric, HNSWConfig, NamespaceStats, ScopedStats, StorageStats, VectorEntry,
        VectorQuery, VectorResult,
    };

    // Pipeline exports will be added when implemented
    // pub use crate::pipeline::{
    //     RAGPipeline, RAGPipelineBuilder, IngestOptions, SearchOptions,
    // };

    // Embeddings exports are now available
    pub use crate::embeddings::{
        CacheConfig, DimensionConfig, DimensionMapper, EmbeddingCache, EmbeddingFactory,
        EmbeddingFactoryBuilder, EmbeddingModel, EmbeddingProvider, EmbeddingProviderConfig,
        EmbeddingProviderType, LateInteractionModel, TokenEmbeddings,
    };

    // Multi-tenant RAG integration
    pub use crate::multi_tenant_integration::{
        MultiTenantRAG, TenantUsageMetrics, TenantVectorConfig,
    };

    // State integration
    pub use crate::state_integration::{
        StateAwareVectorStorage, TenantVectorStats, VectorOperationMetadata,
    };

    // Session integration
    pub use crate::session_integration::{
        scope_helpers, IngestStats, SessionAwareRAGPipeline, SessionVectorCollection,
        SessionVectorResult,
    };

    pub use llmspell_core::state::StateScope;
}

/// Library version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version() {
        // VERSION is a compile-time constant from Cargo.toml, always non-empty
        assert_eq!(VERSION, env!("CARGO_PKG_VERSION"));
    }
}
