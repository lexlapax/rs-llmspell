//! Core RAG pipeline orchestrator

use anyhow::Result;
use llmspell_core::state::StateScope;
use llmspell_storage::vector_storage::VectorStorage;
use std::sync::Arc;
use thiserror::Error;
use tokio::time::{timeout, Duration};
use tracing::{debug, error, info};

use crate::{
    chunking::{ChunkingStrategy, SlidingWindowChunker},
    embeddings::{EmbeddingCache, EmbeddingFactory},
};

use super::{
    config::{QueryConfig, RAGConfig},
    ingestion::{DocumentProcessor, IngestionFlow, IngestionResult},
    retrieval_flow::{RetrievalFlow, RetrievalResult},
};

/// Core RAG pipeline that orchestrates document processing and retrieval
pub struct RAGPipeline {
    /// Pipeline configuration
    config: RAGConfig,

    /// Vector storage backend
    storage: Arc<dyn VectorStorage>,

    /// Embedding generation and caching
    embedding_factory: Arc<EmbeddingFactory>,
    embedding_cache: Arc<EmbeddingCache>,

    /// Document processing components
    document_processor: DocumentProcessor,
    ingestion_flow: IngestionFlow,

    /// Retrieval components  
    retrieval_flow: RetrievalFlow,

    /// Default chunking strategy
    #[allow(dead_code)]
    chunker: Box<dyn ChunkingStrategy>,
}

impl std::fmt::Debug for RAGPipeline {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RAGPipeline")
            .field("config", &self.config)
            .finish_non_exhaustive()
    }
}

/// Errors that can occur in RAG pipeline operations
#[derive(Debug, Error)]
pub enum RAGPipelineError {
    /// Configuration is invalid
    #[error("Configuration error: {message}")]
    Configuration {
        /// The configuration error message
        message: String,
    },

    /// Document ingestion failed
    #[error("Ingestion failed: {source}")]
    Ingestion {
        /// The underlying ingestion error
        source: anyhow::Error,
    },

    /// Retrieval operation failed
    #[error("Retrieval failed: {source}")]
    Retrieval {
        /// The underlying retrieval error
        source: anyhow::Error,
    },

    /// Embedding generation failed
    #[error("Embedding generation failed: {source}")]
    Embedding {
        /// The underlying embedding error
        source: anyhow::Error,
    },

    /// Storage backend error occurred
    #[error("Storage operation failed: {source}")]
    Storage {
        /// The underlying storage error
        source: anyhow::Error,
    },

    /// Operation exceeded timeout
    #[error("Operation timeout: {operation} took longer than {timeout_secs}s")]
    Timeout {
        /// The operation that timed out
        operation: String,
        /// The timeout duration in seconds
        timeout_secs: u64,
    },

    /// Invalid scope was provided
    #[error("Invalid scope: {scope}")]
    InvalidScope {
        /// The invalid scope that was provided
        scope: String,
    },
}

impl RAGPipeline {
    /// Create a new RAG pipeline
    ///
    /// # Errors
    ///
    /// Returns an error if configuration validation fails
    pub fn new(
        config: RAGConfig,
        storage: Arc<dyn VectorStorage>,
        embedding_factory: Arc<EmbeddingFactory>,
        embedding_cache: Arc<EmbeddingCache>,
    ) -> Result<Self, RAGPipelineError> {
        // Validate configuration
        config
            .retrieval
            .hybrid_weights
            .validate()
            .map_err(|msg| RAGPipelineError::Configuration { message: msg })?;

        // Create chunking strategy
        let chunker: Box<dyn ChunkingStrategy> = Box::new(SlidingWindowChunker::new());

        // Create processing components
        let document_processor =
            DocumentProcessor::new(config.ingestion.clone(), embedding_factory.clone());

        let ingestion_flow = IngestionFlow::new(
            storage.clone(),
            embedding_cache.clone(),
            config.ingestion.clone(),
        );

        let retrieval_flow = RetrievalFlow::new(
            storage.clone(),
            embedding_factory.clone(),
            embedding_cache.clone(),
            config.retrieval.clone(),
        );

        Ok(Self {
            config,
            storage,
            embedding_factory,
            embedding_cache,
            document_processor,
            ingestion_flow,
            retrieval_flow,
            chunker,
        })
    }

    /// Ingest a document into the RAG system
    ///
    /// # Errors
    ///
    /// Returns an error if document processing or storage fails
    pub async fn ingest_document(
        &self,
        document_id: String,
        content: String,
        metadata: Option<serde_json::Value>,
        scope: Option<StateScope>,
    ) -> Result<IngestionResult, RAGPipelineError> {
        let timeout_duration = Duration::from_secs(self.config.timeouts.pipeline);
        self.ingest_document_internal(document_id, content, metadata, scope, timeout_duration)
            .await
    }

    /// Search for relevant documents
    ///
    /// # Errors
    ///
    /// Returns an error if search or embedding generation fails
    pub async fn search(
        &self,
        query: String,
        scope: Option<StateScope>,
        query_config: Option<QueryConfig>,
    ) -> Result<RetrievalResult, RAGPipelineError> {
        let operation = "search";
        let timeout_duration = Duration::from_secs(self.config.timeouts.pipeline);

        let result = timeout(timeout_duration, async {
            debug!("Starting search for query: {}", query);

            let result = self
                .retrieval_flow
                .search(query, scope, query_config)
                .await
                .map_err(|e| RAGPipelineError::Retrieval { source: e })?;

            info!("Search completed with {} results", result.results.len());

            Ok(result)
        })
        .await
        .map_err(|_| RAGPipelineError::Timeout {
            operation: operation.to_string(),
            timeout_secs: self.config.timeouts.pipeline,
        })?;

        result
    }

    /// Batch ingest multiple documents
    ///
    /// # Errors
    ///
    /// Returns an error if semaphore acquisition fails
    pub async fn batch_ingest(
        &self,
        documents: Vec<(String, String, Option<serde_json::Value>)>,
        scope: Option<StateScope>,
    ) -> Result<Vec<Result<IngestionResult, RAGPipelineError>>, RAGPipelineError> {
        info!("Starting batch ingestion of {} documents", documents.len());

        let mut results = Vec::with_capacity(documents.len());
        let semaphore = Arc::new(tokio::sync::Semaphore::new(self.config.max_concurrency));
        let timeout_duration = Duration::from_secs(self.config.timeouts.pipeline);

        for (doc_id, content, metadata) in documents {
            let permit = semaphore.clone().acquire_owned().await.map_err(|e| {
                RAGPipelineError::Configuration {
                    message: format!("Semaphore error: {e}"),
                }
            })?;

            let scope_clone = scope.clone();

            // Process document directly without spawning tasks to avoid lifetime issues
            let result = self
                .ingest_document_internal(doc_id, content, metadata, scope_clone, timeout_duration)
                .await;

            results.push(result);
            drop(permit); // Release semaphore
        }

        let successful = results.iter().filter(|r| r.is_ok()).count();
        info!(
            "Batch ingestion completed: {}/{} successful",
            successful,
            results.len()
        );

        Ok(results)
    }

    /// Internal document ingestion method
    async fn ingest_document_internal(
        &self,
        document_id: String,
        content: String,
        metadata: Option<serde_json::Value>,
        scope: Option<StateScope>,
        timeout_duration: Duration,
    ) -> Result<IngestionResult, RAGPipelineError> {
        let operation = "document_ingestion";

        let result = timeout(timeout_duration, async {
            debug!("Starting document ingestion for ID: {}", document_id);

            // Process document (chunking + embedding)
            let processed_doc = self
                .document_processor
                .process_document(document_id.clone(), content, metadata, scope.clone())
                .await
                .map_err(|e| RAGPipelineError::Ingestion { source: e })?;

            // Store in vector database
            let ingestion_result = self
                .ingestion_flow
                .ingest_processed_document(processed_doc)
                .await
                .map_err(|e| RAGPipelineError::Storage { source: e })?;

            info!(
                "Successfully ingested document {} with {} chunks",
                document_id, ingestion_result.chunks_stored
            );

            Ok(ingestion_result)
        })
        .await
        .map_err(|_| RAGPipelineError::Timeout {
            operation: operation.to_string(),
            timeout_secs: timeout_duration.as_secs(),
        })?;

        result
    }

    /// Get pipeline statistics
    ///
    /// # Errors
    ///
    /// Returns an error if storage stats retrieval fails
    pub async fn stats(
        &self,
        scope: Option<StateScope>,
    ) -> Result<PipelineStats, RAGPipelineError> {
        let storage_stats = if let Some(ref scope) = scope {
            self.storage
                .stats_for_scope(scope)
                .await
                .map_err(|e| RAGPipelineError::Storage { source: e })?
        } else {
            let stats = self
                .storage
                .stats()
                .await
                .map_err(|e| RAGPipelineError::Storage { source: e })?;
            // Convert to scoped stats format
            llmspell_storage::vector_storage::ScopedStats {
                scope: llmspell_core::state::StateScope::Global,
                vector_count: stats.total_vectors,
                storage_bytes: stats.storage_bytes,
                query_count: 0,      // Global stats don't track queries
                tokens_processed: 0, // Not tracked at global level
                estimated_cost: 0.0, // Not tracked at global level
            }
        };

        let cache_stats = self.embedding_cache.stats();
        let embedding_cost = self.embedding_factory.estimated_cost().unwrap_or(0.0);

        Ok(PipelineStats {
            vectors_stored: storage_stats.vector_count,
            memory_usage_bytes: storage_stats.storage_bytes,
            avg_query_time_ms: 0.0, // TODO: Calculate from storage stats
            cache_hits: cache_stats.0,
            cache_misses: cache_stats.1,
            cache_hit_rate: cache_stats.2,
            estimated_cost_usd: embedding_cost,
        })
    }

    /// Clear all data for a scope (tenant cleanup)
    ///
    /// # Errors
    ///
    /// Returns an error if storage deletion fails
    pub async fn clear_scope(&self, scope: &StateScope) -> Result<usize, RAGPipelineError> {
        debug!("Clearing scope: {:?}", scope);

        let deleted_count = self
            .storage
            .delete_scope(scope)
            .await
            .map_err(|e| RAGPipelineError::Storage { source: e })?;

        // Also clear cache entries (this is approximate since cache keys are hashed)
        self.embedding_cache.clear();

        info!("Cleared {} vectors for scope: {:?}", deleted_count, scope);
        Ok(deleted_count)
    }
}

/// Pipeline statistics
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PipelineStats {
    /// Number of vectors stored
    pub vectors_stored: usize,

    /// Memory usage in bytes
    pub memory_usage_bytes: usize,

    /// Average query time in milliseconds
    pub avg_query_time_ms: f32,

    /// Embedding cache hits
    pub cache_hits: usize,

    /// Embedding cache misses
    pub cache_misses: usize,

    /// Cache hit rate (0.0 to 1.0)
    pub cache_hit_rate: f64,

    /// Estimated cost in USD
    pub estimated_cost_usd: f64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::embeddings::{EmbeddingProviderConfig, EmbeddingProviderType};
    use llmspell_storage::backends::vector::HNSWVectorStorage;
    use llmspell_storage::vector_storage::HNSWConfig;

    fn create_test_pipeline() -> RAGPipeline {
        let storage = Arc::new(HNSWVectorStorage::new(384, HNSWConfig::default()));
        let embedding_config = EmbeddingProviderConfig {
            provider_type: EmbeddingProviderType::HuggingFace,
            model: "test-model".to_string(),
            dimensions: Some(384),
            ..Default::default()
        };
        let embedding_factory = Arc::new(EmbeddingFactory::new(embedding_config));
        let embedding_cache = Arc::new(EmbeddingCache::new(
            crate::embeddings::CacheConfig::default(),
        ));

        RAGPipeline::new(
            RAGConfig::default(),
            storage,
            embedding_factory,
            embedding_cache,
        )
        .unwrap()
    }

    #[tokio::test]
    async fn test_pipeline_creation() {
        let _pipeline = create_test_pipeline();
        // If we get here without panicking, creation succeeded
    }

    #[tokio::test]
    async fn test_invalid_config() {
        let storage = Arc::new(HNSWVectorStorage::new(384, HNSWConfig::default()));
        let embedding_factory = Arc::new(EmbeddingFactory::new(EmbeddingProviderConfig::default()));
        let embedding_cache = Arc::new(EmbeddingCache::new(
            crate::embeddings::CacheConfig::default(),
        ));

        let mut invalid_config = RAGConfig::default();
        invalid_config.retrieval.hybrid_weights.vector = -0.5; // Invalid weight

        let result = RAGPipeline::new(invalid_config, storage, embedding_factory, embedding_cache);
        assert!(result.is_err());
    }
}
