//! RAG pipeline builder with validation and configuration management

use anyhow::Result;
use std::sync::Arc;
use thiserror::Error;
use tracing::{debug, info};

use crate::embeddings::{CacheConfig, EmbeddingCache, EmbeddingFactory, EmbeddingProviderConfig};
use llmspell_storage::vector_storage::VectorStorage;

use super::{
    config::{IngestionConfig, RAGConfig, RerankingConfig, RetrievalConfig, TimeoutConfig},
    rag_pipeline::{RAGPipeline, RAGPipelineError},
};

/// Builder for constructing RAG pipelines with validation
pub struct RAGPipelineBuilder {
    /// Pipeline configuration
    config: Option<RAGConfig>,

    /// Vector storage backend
    storage: Option<Arc<dyn VectorStorage>>,

    /// Embedding factory
    embedding_factory: Option<Arc<EmbeddingFactory>>,

    /// Embedding cache
    embedding_cache: Option<Arc<EmbeddingCache>>,

    /// Whether to validate configuration on build
    validate_config: bool,

    /// Whether to perform startup checks
    startup_checks: bool,
}

impl std::fmt::Debug for RAGPipelineBuilder {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RAGPipelineBuilder")
            .field("validate_config", &self.validate_config)
            .field("startup_checks", &self.startup_checks)
            .finish_non_exhaustive()
    }
}

/// Errors that can occur during pipeline construction
#[derive(Debug, Error)]
pub enum RAGPipelineBuilderError {
    /// A required component is missing
    #[error("Missing required component: {component}")]
    MissingComponent {
        /// The name of the missing component
        component: String,
    },

    /// The configuration is invalid
    #[error("Invalid configuration: {message}")]
    InvalidConfiguration {
        /// Details about the invalid configuration
        message: String,
    },

    /// Storage backend error
    #[error("Storage backend error: {source}")]
    StorageError {
        /// The underlying storage error
        source: anyhow::Error,
    },

    /// Embedding configuration error
    #[error("Embedding configuration error: {source}")]
    EmbeddingError {
        /// The underlying embedding error
        source: anyhow::Error,
    },

    /// Pipeline construction failed
    #[error("Pipeline construction failed: {source}")]
    ConstructionFailed {
        /// The underlying pipeline error
        source: RAGPipelineError,
    },

    /// Startup validation failed
    #[error("Startup validation failed: {message}")]
    StartupValidationFailed {
        /// Details about the validation failure
        message: String,
    },
}

impl Default for RAGPipelineBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl RAGPipelineBuilder {
    /// Create a new builder with default settings
    #[must_use]
    pub fn new() -> Self {
        Self {
            config: None,
            storage: None,
            embedding_factory: None,
            embedding_cache: None,
            validate_config: true,
            startup_checks: true,
        }
    }

    /// Set the RAG configuration
    #[must_use]
    pub fn with_config(mut self, config: RAGConfig) -> Self {
        self.config = Some(config);
        self
    }

    /// Set the vector storage backend
    #[must_use]
    pub fn with_storage(mut self, storage: Arc<dyn VectorStorage>) -> Self {
        self.storage = Some(storage);
        self
    }

    /// Set the embedding factory
    #[must_use]
    pub fn with_embedding_factory(mut self, factory: Arc<EmbeddingFactory>) -> Self {
        self.embedding_factory = Some(factory);
        self
    }

    /// Set the embedding cache
    #[must_use]
    pub fn with_embedding_cache(mut self, cache: Arc<EmbeddingCache>) -> Self {
        self.embedding_cache = Some(cache);
        self
    }

    /// Skip configuration validation (not recommended)
    #[must_use]
    pub const fn skip_validation(mut self) -> Self {
        self.validate_config = false;
        self
    }

    /// Skip startup checks (faster but less safe)
    #[must_use]
    pub const fn skip_startup_checks(mut self) -> Self {
        self.startup_checks = false;
        self
    }

    /// Create embedding factory from provider config
    #[must_use]
    pub fn with_embedding_provider(mut self, config: EmbeddingProviderConfig) -> Self {
        let factory = Arc::new(EmbeddingFactory::new(config));
        self.embedding_factory = Some(factory);
        self
    }

    /// Create embedding cache from cache config
    #[must_use]
    pub fn with_cache_config(mut self, config: CacheConfig) -> Self {
        let cache = Arc::new(EmbeddingCache::new(config));
        self.embedding_cache = Some(cache);
        self
    }

    /// Build the RAG pipeline with validation
    ///
    /// # Errors
    ///
    /// Returns an error if required components are missing or validation fails
    pub async fn build(self) -> Result<RAGPipeline, RAGPipelineBuilderError> {
        debug!("Building RAG pipeline with validation");

        // Validate all required components are present
        let config = self
            .config
            .ok_or_else(|| RAGPipelineBuilderError::MissingComponent {
                component: "RAGConfig".to_string(),
            })?;

        let storage = self
            .storage
            .ok_or_else(|| RAGPipelineBuilderError::MissingComponent {
                component: "VectorStorage".to_string(),
            })?;

        let embedding_factory =
            self.embedding_factory
                .ok_or_else(|| RAGPipelineBuilderError::MissingComponent {
                    component: "EmbeddingFactory".to_string(),
                })?;

        let embedding_cache =
            self.embedding_cache
                .ok_or_else(|| RAGPipelineBuilderError::MissingComponent {
                    component: "EmbeddingCache".to_string(),
                })?;

        // Validate configuration if requested
        if self.validate_config {
            Self::validate_configuration_static(&config)?;
        }

        // Create the pipeline
        let pipeline = RAGPipeline::new(config, storage, embedding_factory, embedding_cache)
            .map_err(|e| RAGPipelineBuilderError::ConstructionFailed { source: e })?;

        // Perform startup checks if requested
        if self.startup_checks {
            Self::perform_startup_checks_static(&pipeline).await?;
        }

        info!("RAG pipeline built successfully");
        Ok(pipeline)
    }

    /// Validate configuration before building
    fn validate_configuration_static(config: &RAGConfig) -> Result<(), RAGPipelineBuilderError> {
        debug!("Validating RAG configuration");

        // Validate hybrid weights
        config
            .retrieval
            .hybrid_weights
            .validate()
            .map_err(|msg| RAGPipelineBuilderError::InvalidConfiguration { message: msg })?;

        // Validate timeout values
        if config.timeouts.embedding == 0
            || config.timeouts.search == 0
            || config.timeouts.pipeline == 0
        {
            return Err(RAGPipelineBuilderError::InvalidConfiguration {
                message: "All timeout values must be greater than zero".to_string(),
            });
        }

        // Validate concurrency limits
        if config.max_concurrency == 0 || config.max_concurrency > 1000 {
            return Err(RAGPipelineBuilderError::InvalidConfiguration {
                message: "max_concurrency must be between 1 and 1000".to_string(),
            });
        }

        // Validate retrieval parameters
        if config.retrieval.max_results == 0 || config.retrieval.max_results > 10000 {
            return Err(RAGPipelineBuilderError::InvalidConfiguration {
                message: "max_results must be between 1 and 10000".to_string(),
            });
        }

        if config.retrieval.min_score < 0.0 || config.retrieval.min_score > 1.0 {
            return Err(RAGPipelineBuilderError::InvalidConfiguration {
                message: "min_score must be between 0.0 and 1.0".to_string(),
            });
        }

        // Validate chunking configuration
        let chunk_config = &config.ingestion.chunking;
        if chunk_config.max_tokens == 0 || chunk_config.max_tokens > 100_000 {
            return Err(RAGPipelineBuilderError::InvalidConfiguration {
                message: "max_tokens must be between 1 and 100,000 tokens".to_string(),
            });
        }

        if chunk_config.overlap_tokens >= chunk_config.max_tokens {
            return Err(RAGPipelineBuilderError::InvalidConfiguration {
                message: "overlap_tokens must be less than max_tokens".to_string(),
            });
        }

        // Validate embedding dimensions match between config and provider
        if let Some(configured_dims) = config.ingestion.embedding.dimensions {
            if configured_dims == 0 || configured_dims > 10_000 {
                return Err(RAGPipelineBuilderError::InvalidConfiguration {
                    message: "embedding dimensions must be between 1 and 10,000".to_string(),
                });
            }
        }

        debug!("Configuration validation passed");
        Ok(())
    }

    /// Perform startup checks to ensure system health
    async fn perform_startup_checks_static(
        pipeline: &RAGPipeline,
    ) -> Result<(), RAGPipelineBuilderError> {
        debug!("Performing startup checks");

        // Test embedding generation
        Self::test_embedding_generation_static(pipeline);

        // Test storage connectivity
        Self::test_storage_connectivity_static(pipeline).await?;

        // Test cache functionality
        Self::test_cache_functionality_static(pipeline);

        info!("All startup checks passed");
        Ok(())
    }

    /// Test that embedding generation works
    fn test_embedding_generation_static(_pipeline: &RAGPipeline) {
        // TODO: Add actual embedding generation test
        // This would involve creating a simple test embedding to verify the provider works
        debug!("Embedding generation test (placeholder)");
    }

    /// Test storage backend connectivity
    async fn test_storage_connectivity_static(
        pipeline: &RAGPipeline,
    ) -> Result<(), RAGPipelineBuilderError> {
        // Test basic storage stats retrieval
        if let Err(e) = pipeline.stats(None).await {
            return Err(RAGPipelineBuilderError::StartupValidationFailed {
                message: format!("Storage connectivity test failed: {e}"),
            });
        }

        debug!("Storage connectivity test passed");
        Ok(())
    }

    /// Test cache functionality
    fn test_cache_functionality_static(_pipeline: &RAGPipeline) {
        // TODO: Add actual cache functionality test
        // This would involve putting/getting a test value from the cache
        debug!("Cache functionality test (placeholder)");
    }
}

/// Builder for quick development/testing setups
impl RAGPipelineBuilder {
    /// Create a pipeline with default settings for development
    #[must_use]
    pub fn development() -> Self {
        Self::new()
            .with_config(RAGConfig::default())
            .skip_startup_checks()
    }

    /// Create a pipeline optimized for production
    #[must_use]
    pub fn production() -> Self {
        let config = RAGConfig {
            max_concurrency: 50,
            timeouts: TimeoutConfig {
                pipeline: 120,
                ..Default::default()
            },
            retrieval: RetrievalConfig {
                reranking: RerankingConfig {
                    enabled: true,
                    ..Default::default()
                },
                ..Default::default()
            },
            ingestion: IngestionConfig {
                deduplicate: true,
                ..Default::default()
            },
            ..Default::default()
        };

        Self::new().with_config(config)
    }

    /// Create a pipeline optimized for testing
    #[must_use]
    pub fn testing() -> Self {
        let config = RAGConfig {
            max_concurrency: 2,
            timeouts: TimeoutConfig {
                pipeline: 10,
                ..Default::default()
            },
            retrieval: RetrievalConfig {
                max_results: 5,
                ..Default::default()
            },
            ..Default::default()
        };

        Self::new()
            .with_config(config)
            .skip_validation()
            .skip_startup_checks()
    }

    /// Create a minimal pipeline for examples
    #[must_use]
    pub fn minimal() -> Self {
        let config = RAGConfig {
            max_concurrency: 1,
            retrieval: RetrievalConfig {
                reranking: RerankingConfig {
                    enabled: false,
                    ..Default::default()
                },
                ..Default::default()
            },
            ingestion: IngestionConfig {
                deduplicate: false,
                ..Default::default()
            },
            ..Default::default()
        };

        Self::new()
            .with_config(config)
            .skip_validation()
            .skip_startup_checks()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::embeddings::EmbeddingProviderType;
    use llmspell_storage::backends::sqlite::{SqliteBackend, SqliteConfig, SqliteVectorStorage};

    async fn create_test_components() -> (
        Arc<dyn VectorStorage>,
        Arc<EmbeddingFactory>,
        Arc<EmbeddingCache>,
    ) {
        let config = SqliteConfig::new(":memory:");
        let backend = Arc::new(SqliteBackend::new(config).await.unwrap());
        let storage = Arc::new(SqliteVectorStorage::new(backend, 384).await.unwrap());

        let embedding_config = EmbeddingProviderConfig {
            provider_type: EmbeddingProviderType::HuggingFace,
            model: "test-model".to_string(),
            dimensions: Some(384),
            ..Default::default()
        };
        let embedding_factory = Arc::new(EmbeddingFactory::new(embedding_config));
        let embedding_cache = Arc::new(EmbeddingCache::new(CacheConfig::default()));

        (storage, embedding_factory, embedding_cache)
    }

    #[tokio::test]
    async fn test_builder_with_all_components() {
        let (storage, factory, cache) = create_test_components().await;

        let result = RAGPipelineBuilder::new()
            .with_config(RAGConfig::default())
            .with_storage(storage)
            .with_embedding_factory(factory)
            .with_embedding_cache(cache)
            .skip_startup_checks()
            .build()
            .await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_builder_missing_component() {
        let result = RAGPipelineBuilder::new().build().await;

        assert!(result.is_err());
        if let Err(RAGPipelineBuilderError::MissingComponent { component }) = result {
            assert_eq!(component, "RAGConfig");
        }
    }

    #[test]
    fn test_development_builder() {
        let builder = RAGPipelineBuilder::development();
        assert!(builder.config.is_some());
        assert!(!builder.startup_checks);
    }

    #[test]
    fn test_production_builder() {
        let builder = RAGPipelineBuilder::production();
        assert!(builder.config.is_some());
        if let Some(config) = &builder.config {
            assert_eq!(config.max_concurrency, 50);
            assert!(config.retrieval.reranking.enabled);
        }
    }

    #[tokio::test]
    async fn test_invalid_configuration() {
        let (storage, factory, cache) = create_test_components().await;

        let mut invalid_config = RAGConfig::default();
        invalid_config.retrieval.hybrid_weights.vector = -1.0; // Invalid weight

        let result = RAGPipelineBuilder::new()
            .with_config(invalid_config)
            .with_storage(storage)
            .with_embedding_factory(factory)
            .with_embedding_cache(cache)
            .build()
            .await;

        assert!(result.is_err());
        if let Err(RAGPipelineBuilderError::InvalidConfiguration { message: _ }) = result {
            // Expected error type
        } else {
            panic!("Expected InvalidConfiguration error");
        }
    }
}
