# Phase 8: Vector Storage and RAG Foundation - Design Document

**Version**: 1.0  
**Date**: August 2025  
**Status**: Implementation Specification  
**Phase**: 8 (Vector Storage and RAG Foundation)  
**Timeline**: Weeks 28-29 (2 weeks)  
**Priority**: HIGH (Foundation for Memory System)  
**Dependencies**: Phase 7 Infrastructure Consolidation ✅  
**Crate Structure**: New `llmspell-rag` crate, enhanced `llmspell-tools`, bridge integration

> **📋 Vector Storage Foundation**: This phase establishes the essential vector storage and retrieval infrastructure that will serve as the foundation for Phase 9's Adaptive Temporal Knowledge Graph memory system. Focus on production-ready HNSW indexing with BGE-M3 embeddings and ColBERT v2 late interaction.

---

## Phase Overview

### Goal
Implement production-ready vector storage and retrieval infrastructure that seamlessly integrates with rs-llmspell's existing multi-provider architecture. Support dynamic dimensions, provider-specific embeddings, and cost-aware routing while maintaining performance targets for Phase 9's Adaptive Memory System.

### Core Principles
- **Provider-First Architecture**: Leverage existing ProviderConfig infrastructure for embeddings
- **Dynamic Dimensions**: Support 256-4096 dimensional vectors across providers
- **Cost-Aware Routing**: Intelligent selection between API and local embeddings
- **Multi-Tenant by Design**: Scope-aware storage with tenant isolation and cost tracking
- **State & Session Integration**: Seamless binding with StateScope and SessionManager
- **Security-First**: PostgreSQL RLS-inspired policies with strict data isolation
- **Bridge-First Design**: Leverage mature Rust crates rather than reimplementing
- **Performance Critical**: Sub-millisecond retrieval for 1M+ vectors per tenant
- **Memory Efficient**: Streaming processing to handle large documents
- **Type Safe**: Leverage Rust's type system for compile-time guarantees
- **Hook Integration**: Full event emission and hook support for vector operations
- **Script Exposure**: Consistent API across Lua/JavaScript/Python bridges

### Critical Design Updates (Provider Impact)
**Research revealed** that different LLM providers offer vastly different embedding models:
- **OpenAI**: 256-3072 dims with Matryoshka Representation Learning
- **Google**: 768-3072 dims with multiple models
- **Cohere**: 1024 dims with multimodal support
- **Anthropic**: Partners with Voyage AI (no native embeddings)
- **Open Source**: BGE-M3, E5, ColBERT with 384-4096 dims

This necessitated fundamental architecture changes:
1. **Dynamic Storage**: Multiple HNSW indices for different dimensions
2. **Provider Integration**: Extend existing ProviderConfig instead of parallel system
3. **Cost Routing**: Select embeddings based on cost/performance tradeoffs
4. **Dimension Flexibility**: Support Matryoshka dimension reduction

### Success Criteria
- [ ] HNSW index supports 1M+ vectors with <10ms retrieval
- [ ] BGE-M3 embeddings generate 1024-dim vectors for 8192 token contexts
- [ ] ColBERT v2 provides token-level late interaction retrieval
- [ ] Hybrid retrieval combines vector, keyword, and graph traversal
- [ ] All vector operations emit events and support hooks
- [ ] Lua/JS scripts can perform vector search and RAG operations
- [ ] Configuration supports multiple embedding models and vector stores
- [ ] Integration tests validate end-to-end RAG pipeline

---

## 1. Vector Storage Architecture

### 1.1 Trait Hierarchy Design

The vector storage system uses a trait-based architecture for maximum flexibility:

```rust
// llmspell-rag/src/traits/storage.rs
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use llmspell_state_traits::StateScope;
use std::time::SystemTime;

/// Core vector storage trait with multi-tenant support
#[async_trait]
pub trait VectorStorage: Send + Sync {
    /// Insert vectors with metadata and scope
    async fn insert(&self, vectors: Vec<VectorEntry>) -> Result<Vec<String>>;
    
    /// Search for similar vectors within a scope
    async fn search(&self, query: &VectorQuery) -> Result<Vec<VectorResult>>;
    
    /// Scoped search with tenant isolation
    async fn search_scoped(
        &self,
        query: &VectorQuery,
        scope: &StateScope,
    ) -> Result<Vec<VectorResult>>;
    
    /// Update vector metadata
    async fn update_metadata(&self, id: &str, metadata: HashMap<String, Value>) -> Result<()>;
    
    /// Delete vectors by ID
    async fn delete(&self, ids: &[String]) -> Result<()>;
    
    /// Delete all vectors for a scope (tenant cleanup)
    async fn delete_scope(&self, scope: &StateScope) -> Result<usize>;
    
    /// Get storage statistics
    async fn stats(&self) -> Result<StorageStats>;
    
    /// Get tenant-specific statistics
    async fn stats_for_scope(&self, scope: &StateScope) -> Result<ScopedStats>;
}

/// Multi-tenant aware vector entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VectorEntry {
    pub id: String,
    pub embedding: Vec<f32>,
    pub metadata: HashMap<String, Value>,
    pub scope: StateScope,  // Tenant/user/session binding
    pub created_at: SystemTime,
    pub expires_at: Option<SystemTime>,  // TTL for session vectors
    pub tenant_id: Option<String>,  // Explicit tenant for billing
}

/// HNSW-specific storage trait with namespace support
#[async_trait]
pub trait HNSWStorage: VectorStorage {
    /// Configure HNSW parameters
    fn configure_hnsw(&mut self, config: HNSWConfig);
    
    /// Build or rebuild the HNSW index
    async fn build_index(&self) -> Result<()>;
    
    /// Create tenant-specific namespace/index
    async fn create_namespace(&self, namespace: &str) -> Result<()>;
    
    /// Get current HNSW parameters
    fn hnsw_params(&self) -> &HNSWConfig;
}

/// Hybrid storage supporting multiple retrieval methods
#[async_trait]
pub trait HybridStorage: VectorStorage {
    /// Perform hybrid search (vector + keyword + metadata)
    async fn hybrid_search(&self, query: &HybridQuery) -> Result<Vec<HybridResult>>;
    
    /// Configure retrieval weights
    fn set_weights(&mut self, weights: RetrievalWeights);
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VectorEntry {
    pub id: String,
    pub vector: Vec<f32>,
    pub metadata: HashMap<String, Value>,
    pub timestamp: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HNSWConfig {
    pub m: usize,              // Number of bi-directional links (16-64)
    pub ef_construction: usize, // Size of dynamic candidate list (200)
    pub ef_search: usize,      // Size of search candidate list (50-200)
    pub max_elements: usize,   // Maximum number of elements
}
```

### 1.2 Dynamic Dimension-Aware Storage

The storage layer MUST handle variable dimensions from different providers:

```rust
// llmspell-rag/src/storage/dynamic.rs

/// Multi-collection storage for different embedding models
pub struct DynamicVectorStorage {
    collections: HashMap<String, Box<dyn VectorStorage>>,
    dimension_map: HashMap<String, usize>,
    default_collection: String,
}

impl DynamicVectorStorage {
    /// Create or get collection for specific dimensions
    pub async fn get_or_create_collection(
        &mut self,
        name: &str,
        dimensions: usize,
    ) -> Result<&mut Box<dyn VectorStorage>> {
        if !self.collections.contains_key(name) {
            let storage = self.create_storage_for_dimensions(dimensions).await?;
            self.collections.insert(name.to_string(), storage);
            self.dimension_map.insert(name.to_string(), dimensions);
        }
        
        Ok(self.collections.get_mut(name).unwrap())
    }
    
    /// Route queries to appropriate collections based on embedding dimensions
    pub async fn search_multi_dimension(
        &self,
        query_vector: &[f32],
    ) -> Result<Vec<VectorResult>> {
        let query_dims = query_vector.len();
        
        // Find compatible collections (same or reducible dimensions)
        let compatible_collections: Vec<_> = self.dimension_map
            .iter()
            .filter(|(_, &dims)| {
                dims == query_dims || 
                (dims > query_dims && dims % query_dims == 0)  // Matryoshka compatibility
            })
            .collect();
        
        if compatible_collections.is_empty() {
            return Err(anyhow!("No collection found for {} dimensions", query_dims));
        }
        
        // Search across compatible collections
        let mut all_results = Vec::new();
        for (name, _) in compatible_collections {
            if let Some(storage) = self.collections.get(name) {
                let results = storage.search(&VectorQuery {
                    vector: query_vector.to_vec(),
                    k: 10,
                    filter: None,
                }).await?;
                all_results.extend(results);
            }
        }
        
        // Sort by score and dedup
        all_results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());
        all_results.dedup_by_key(|r| r.id.clone());
        all_results.truncate(10);
        
        Ok(all_results)
    }
}

// llmspell-rag/src/storage/memvdb.rs
use memvdb::MemVDB;

/// In-memory vector database using MemVDB with dynamic dimensions
pub struct MemVDBStorage {
    db: Arc<Mutex<MemVDB>>,
    config: HNSWConfig,
    dimensions: usize,  // Dynamic based on embedding model
}

impl MemVDBStorage {
    pub fn new(config: HNSWConfig, dimensions: usize) -> Result<Self> {
        let db = MemVDB::new()
            .dim(dimensions)  // Dynamic dimensions
            .hnsw_m(config.m)
            .hnsw_ef_construction(config.ef_construction)
            .build()?;
        
        Ok(Self {
            db: Arc::new(Mutex::new(db)),
            config,
            dimensions,
        })
    }
    
    pub fn dimensions(&self) -> usize {
        self.dimensions
    }
}

// llmspell-rag/src/storage/qdrant.rs
use qdrant_client::prelude::*;

/// Qdrant vector database client with collection per dimension
pub struct QdrantStorage {
    client: QdrantClient,
    collections: HashMap<usize, String>,  // dimension -> collection name
    config: HNSWConfig,
}

impl QdrantStorage {
    pub async fn ensure_collection(&mut self, dimensions: usize) -> Result<String> {
        let collection_name = format!("vectors_{}d", dimensions);
        
        if !self.collections.contains_key(&dimensions) {
            // Create collection with specific dimensions
            self.client.create_collection(&CreateCollection {
                collection_name: collection_name.clone(),
                vectors_config: Some(VectorsConfig {
                    size: dimensions as u64,
                    distance: Distance::Cosine,
                    hnsw_config: Some(HnswConfig {
                        m: self.config.m as u64,
                        ef_construct: self.config.ef_construction as u64,
                        ..Default::default()
                    }),
                    ..Default::default()
                }),
                ..Default::default()
            }).await?;
            
            self.collections.insert(dimensions, collection_name.clone());
        }
        
        Ok(self.collections[&dimensions].clone())
    }
}

// llmspell-rag/src/storage/embedded.rs
use hnswlib_rs::hnsw::Hnsw;

/// Embedded HNSW with support for multiple dimension indices
pub struct EmbeddedHNSW {
    indices: HashMap<usize, Arc<RwLock<Hnsw<f32, DistCosine>>>>,
    metadata: Arc<RwLock<HashMap<(usize, usize), VectorMetadata>>>,  // (dims, id) -> metadata
    config: HNSWConfig,
}

impl EmbeddedHNSW {
    pub fn get_or_create_index(&mut self, dimensions: usize) -> Result<Arc<RwLock<Hnsw<f32, DistCosine>>>> {
        if !self.indices.contains_key(&dimensions) {
            let index = Hnsw::<f32, DistCosine>::new(
                self.config.m,
                self.config.max_elements,
                dimensions,
                self.config.ef_construction,
                DistCosine,
            );
            self.indices.insert(dimensions, Arc::new(RwLock::new(index)));
        }
        
        Ok(self.indices[&dimensions].clone())
    }
}
```

---

## 2. Embedding Pipeline Architecture

### 2.1 Leveraging Existing Provider Abstraction

**CRITICAL UPDATE**: rs-llmspell already has `llmspell-providers` crate with provider abstractions using `rig-core`. The embedding system MUST extend this existing infrastructure, not create a parallel system.

The provider crate currently uses:
- **rig-core**: For API-based providers (OpenAI, Cohere, Anthropic)
- **Future**: candle-core, candle-transformers, tokenizers for local models

We need to extend the existing `ProviderInstance` trait to support embeddings:

```rust
// llmspell-providers/src/abstraction.rs (EXTENSION)
use rig::embeddings::EmbeddingModel as RigEmbeddingModel;

/// Extended trait for providers that support embeddings
#[async_trait]
pub trait EmbeddingProvider: ProviderInstance {
    /// Generate embeddings for text
    async fn embed(&self, texts: &[String]) -> Result<Vec<Vec<f32>>, LLMSpellError>;
    
    /// Get embedding dimensions
    fn embedding_dimensions(&self) -> usize;
    
    /// Check if dimensions are configurable (e.g., OpenAI's Matryoshka)
    fn supports_dimension_reduction(&self) -> bool {
        false
    }
    
    /// Configure output dimensions if supported
    fn set_embedding_dimensions(&mut self, dims: usize) -> Result<(), LLMSpellError> {
        Err(LLMSpellError::Provider {
            message: "Dimension configuration not supported".to_string(),
            provider: Some(self.name().to_string()),
            source: None,
        })
    }
    
    /// Get embedding model name
    fn embedding_model(&self) -> Option<&str>;
    
    /// Estimated cost per token for embeddings
    fn embedding_cost_per_token(&self) -> Option<f64> {
        None
    }
}

/// Provider-specific embedding configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct EmbeddingProviderConfig {
    pub provider_type: EmbeddingProviderType,
    pub model: String,
    pub dimensions: Option<usize>,  // None = use model default
    pub api_key_env: Option<String>,
    pub base_url: Option<String>,
    pub max_batch_size: usize,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum EmbeddingProviderType {
    OpenAI,           // text-embedding-3-large/small, ada-002
    Google,           // text-embedding-004, gecko, gemini-embedding-001
    Cohere,           // embed-v3-english, embed-v3-multilingual
    VoyageAI,         // voyage-2, voyage-large-2, voyage-code-2
    AWSBedrock,       // Various models via Bedrock
    HuggingFace,      // BGE-M3, E5, etc via local inference
    FastEmbed,        // ONNX-optimized models
    Custom(String),   // User-provided implementation
}

/// Late interaction model trait (ColBERT)
#[async_trait]
pub trait LateInteractionModel: EmbeddingModel {
    /// Generate token-level embeddings
    async fn embed_tokens(&self, texts: &[String]) -> Result<Vec<TokenEmbeddings>>;
    
    /// Compute late interaction score
    fn late_interaction_score(&self, query: &TokenEmbeddings, doc: &TokenEmbeddings) -> f32;
}

#[derive(Debug, Clone)]
pub struct TokenEmbeddings {
    pub token_ids: Vec<u32>,
    pub embeddings: Vec<Vec<f32>>,  // One embedding per token
    pub dimensions: usize,           // Actual dimensions (dynamic)
}
```

### 2.2 Extending RigProvider with Embeddings

We extend the existing `RigProvider` to support embeddings using rig-core's `EmbeddingModel` trait:

```rust
// llmspell-providers/src/rig.rs (EXTENSION)
use rig::embeddings::{EmbeddingModel as RigEmbeddingModel, Embedding};
use rig::providers;

/// Extended enum to hold both completion and embedding models
enum RigModelType {
    Completion(RigCompletionModel),
    Embedding(RigEmbeddingModelWrapper),
    Both {
        completion: RigCompletionModel,
        embedding: RigEmbeddingModelWrapper,
    },
}

/// Wrapper for different embedding models from rig
enum RigEmbeddingModelWrapper {
    OpenAI(providers::openai::EmbeddingModel),
    Cohere(providers::cohere::EmbeddingModel),
    // Future: Candle for local models
    Local(Box<dyn LocalEmbeddingModel>),
}

/// Extended RigProvider with embedding support
pub struct RigProvider {
    config: ProviderConfig,
    capabilities: ProviderCapabilities,
    model: RigModelType,
    max_tokens: u64,
    embedding_config: Option<EmbeddingConfig>,
}

#[derive(Clone, Debug)]
struct EmbeddingConfig {
    model_name: String,
    dimensions: usize,
    supports_dimension_reduction: bool,
    cost_per_token: Option<f64>,
}

impl RigProvider {
    /// Create provider with embedding support
    pub fn new_with_embeddings(config: ProviderConfig) -> Result<Self, LLMSpellError> {
        let (completion_model, embedding_model) = match config.provider_type.as_str() {
            "openai" => {
                let api_key = config.api_key.as_ref()
                    .ok_or_else(|| LLMSpellError::Configuration {
                        message: "OpenAI API key required".to_string(),
                        source: None,
                    })?;
                
                let client = providers::openai::Client::new(api_key);
                
                // Create completion model
                let completion = RigCompletionModel::OpenAI(
                    client.completion_model(&config.model)
                );
                
                // Create embedding model
                let embedding_model_name = config.custom_config
                    .get("embedding_model")
                    .and_then(|v| v.as_str())
                    .unwrap_or("text-embedding-3-small");
                
                let embedding = RigEmbeddingModelWrapper::OpenAI(
                    client.embedding_model(embedding_model_name)
                );
                
                let embedding_config = Some(EmbeddingConfig {
                    model_name: embedding_model_name.to_string(),
                    dimensions: match embedding_model_name {
                        "text-embedding-3-large" => 3072,
                        "text-embedding-3-small" => 1536,
                        "text-embedding-ada-002" => 1536,
                        _ => 1536,
                    },
                    supports_dimension_reduction: embedding_model_name.starts_with("text-embedding-3"),
                    cost_per_token: match embedding_model_name {
                        "text-embedding-3-large" => Some(0.00000013),
                        "text-embedding-3-small" => Some(0.00000002),
                        "text-embedding-ada-002" => Some(0.00000010),
                        _ => None,
                    },
                });
                
                (Some(completion), Some(embedding), embedding_config)
            }
            "cohere" => {
                let api_key = config.api_key.as_ref()
                    .ok_or_else(|| LLMSpellError::Configuration {
                        message: "Cohere API key required".to_string(),
                        source: None,
                    })?;
                
                let client = providers::cohere::Client::new(api_key);
                
                let completion = RigCompletionModel::Cohere(
                    client.completion_model(&config.model)
                );
                
                let embedding_model_name = config.custom_config
                    .get("embedding_model")
                    .and_then(|v| v.as_str())
                    .unwrap_or("embed-v3");
                
                let embedding = RigEmbeddingModelWrapper::Cohere(
                    client.embedding_model(embedding_model_name)
                );
                
                let embedding_config = Some(EmbeddingConfig {
                    model_name: embedding_model_name.to_string(),
                    dimensions: 1024,
                    supports_dimension_reduction: false,
                    cost_per_token: Some(0.00000010),
                });
                
                (Some(completion), Some(embedding), embedding_config)
            }
            "local" => {
                // Future: Use candle for local models
                let embedding_model_name = config.custom_config
                    .get("embedding_model")
                    .and_then(|v| v.as_str())
                    .unwrap_or("BAAI/bge-m3");
                
                let embedding = RigEmbeddingModelWrapper::Local(
                    Box::new(CandleEmbeddingModel::new(embedding_model_name)?)
                );
                
                let embedding_config = Some(EmbeddingConfig {
                    model_name: embedding_model_name.to_string(),
                    dimensions: 1024,  // BGE-M3 default
                    supports_dimension_reduction: false,
                    cost_per_token: None,  // Free local execution
                });
                
                (None, Some(embedding), embedding_config)
            }
            _ => return Err(LLMSpellError::Configuration {
                message: format!("Unsupported provider: {}", config.provider_type),
                source: None,
            }),
        };
        
        // Build the model type
        let model = match (completion_model, embedding_model) {
            (Some(c), Some(e)) => RigModelType::Both {
                completion: c,
                embedding: e,
            },
            (Some(c), None) => RigModelType::Completion(c),
            (None, Some(e)) => RigModelType::Embedding(e),
            (None, None) => return Err(LLMSpellError::Configuration {
                message: "No models configured".to_string(),
                source: None,
            }),
        };
        
        Ok(Self {
            config,
            capabilities,
            model,
            max_tokens,
            embedding_config,
        })
    }
}

### 2.3 Future Local Model Support with Candle

For local model execution, we'll implement a trait that can be fulfilled by candle-core/candle-transformers:

```rust
// llmspell-providers/src/local/mod.rs
use async_trait::async_trait;

/// Trait for local embedding models (to be implemented with candle)
#[async_trait]
pub trait LocalEmbeddingModel: Send + Sync {
    async fn embed(&self, texts: &[String]) -> Result<Vec<Vec<f32>>, LLMSpellError>;
    fn dimensions(&self) -> usize;
    fn model_id(&self) -> &str;
}

// llmspell-providers/src/local/candle_embeddings.rs (FUTURE)
use candle_core::{Device, Tensor};
use candle_nn::VarBuilder;
use candle_transformers::models::bert::{BertModel, Config};
use tokenizers::Tokenizer;

pub struct CandleEmbeddingModel {
    model: BertModel,
    tokenizer: Tokenizer,
    device: Device,
    model_id: String,
    dimensions: usize,
}

impl CandleEmbeddingModel {
    pub async fn new(model_id: &str) -> Result<Self, LLMSpellError> {
        // Determine model architecture and dimensions
        let (architecture, dimensions) = match model_id {
            "BAAI/bge-m3" => ("bert", 1024),
            "sentence-transformers/all-MiniLM-L6-v2" => ("bert", 384),
            "jinaai/jina-embeddings-v2-base-en" => ("bert", 768),
            _ => ("bert", 768), // Default
        };
        
        // Load model from HuggingFace or local cache
        let device = Device::cuda_if_available(0)
            .unwrap_or(Device::Cpu);
        
        // This would load the actual model weights
        // let model = load_model_from_huggingface(model_id, &device).await?;
        // let tokenizer = load_tokenizer(model_id).await?;
        
        Ok(Self {
            model,
            tokenizer,
            device,
            model_id: model_id.to_string(),
            dimensions,
        })
    }
}

#[async_trait]
impl LocalEmbeddingModel for CandleEmbeddingModel {
    async fn embed(&self, texts: &[String]) -> Result<Vec<Vec<f32>>, LLMSpellError> {
        // Tokenize and encode texts
        // Run through transformer model
        // Return embeddings
        todo!("Implement with candle")
    }
    
    fn dimensions(&self) -> usize {
        self.dimensions
    }
    
    fn model_id(&self) -> &str {
        &self.model_id
    }
}

#[async_trait]
impl EmbeddingModel for OpenAIEmbedder {
    async fn embed(&self, texts: &[String]) -> Result<Vec<Vec<f32>>> {
        let mut request = EmbeddingRequest {
            model: self.model.clone(),
            input: texts.to_vec(),
            ..Default::default()
        };
        
        // Add dimension parameter if configured and supported
        if let Some(dims) = self.dimensions {
            if self.model.starts_with("text-embedding-3") {
                request.dimensions = Some(dims);
            }
        }
        
        let response = self.client.embeddings(request).await?;
        Ok(response.data.into_iter()
            .map(|e| e.embedding)
            .collect())
    }
    
    fn dimensions(&self) -> usize {
        self.dimensions.unwrap_or(1536)
    }
    
    fn supports_dimension_reduction(&self) -> bool {
        self.model.starts_with("text-embedding-3")
    }
    
    fn set_dimensions(&mut self, dims: usize) -> Result<()> {
        if !self.supports_dimension_reduction() {
            return Err(anyhow!("{} doesn't support dimension configuration", self.model));
        }
        
        // Validate dimension constraints
        match self.model.as_str() {
            "text-embedding-3-large" if dims >= 256 && dims <= 3072 => {
                self.dimensions = Some(dims);
                Ok(())
            }
            "text-embedding-3-small" if dims >= 512 && dims <= 1536 => {
                self.dimensions = Some(dims);
                Ok(())
            }
            _ => Err(anyhow!("Invalid dimensions {} for model {}", dims, self.model))
        }
    }
    
    fn cost_per_token(&self) -> Option<f64> {
        match self.model.as_str() {
            "text-embedding-3-large" => Some(0.00000013),  // $0.00013/1K tokens
            "text-embedding-3-small" => Some(0.00000002),  // $0.00002/1K tokens
            "text-embedding-ada-002" => Some(0.00000010),  // $0.00010/1K tokens
            _ => None,
        }
    }
}

// llmspell-rag/src/embeddings/local.rs
use candle_core::{Device, Tensor};
use embed_anything::{EmbeddingModel as EmbedAnythingModel, ModelType};

pub struct LocalEmbedder {
    model: EmbedAnythingModel,
    model_id: String,
    dimensions: usize,
}

impl LocalEmbedder {
    pub async fn from_huggingface(model_id: &str) -> Result<Self> {
        let (model_type, dims) = match model_id {
            "BAAI/bge-m3" => (ModelType::BGE, 1024),
            "jinaai/jina-colbert-v2" => (ModelType::ColBERT, 128),
            _ => (ModelType::Generic, 768),  // Default
        };
        
        let model = EmbedAnythingModel::new(model_type, model_id).await?;
        
        Ok(Self {
            model,
            model_id: model_id.to_string(),
            dimensions: dims,
        })
    }
}
```

---

## 3. RAG Pipeline Components

### 3.1 Document Processing

```rust
// llmspell-rag/src/pipeline/chunking.rs
pub struct ChunkingStrategy {
    pub max_tokens: usize,
    pub overlap_tokens: usize,
    pub chunking_method: ChunkingMethod,
}

pub enum ChunkingMethod {
    /// Simple sliding window
    SlidingWindow,
    /// Semantic boundary detection
    Semantic,
    /// Preserve document structure
    Structural,
}

pub struct DocumentChunker {
    strategy: ChunkingStrategy,
    tokenizer: Arc<Tokenizer>,
}

impl DocumentChunker {
    pub async fn chunk(&self, document: &Document) -> Result<Vec<Chunk>> {
        match self.strategy.chunking_method {
            ChunkingMethod::SlidingWindow => self.sliding_window_chunk(document).await,
            ChunkingMethod::Semantic => self.semantic_chunk(document).await,
            ChunkingMethod::Structural => self.structural_chunk(document).await,
        }
    }
}
```

### 3.2 Provider-Integrated Retrieval Pipeline

The RAG pipeline leverages the extended provider abstraction:

```rust
// llmspell-rag/src/pipeline/retrieval.rs
use llmspell_providers::{ProviderManager, EmbeddingProvider};

pub struct RAGPipeline {
    chunker: DocumentChunker,
    provider_manager: Arc<ProviderManager>,
    current_provider: String,
    vector_store: DynamicVectorStorage,  // Handles multiple dimensions
    reranker: Option<Box<dyn Reranker>>,
}

impl RAGPipeline {
    pub fn builder() -> RAGPipelineBuilder {
        RAGPipelineBuilder::new()
    }
    
    /// Create pipeline from existing provider manager
    pub async fn from_provider_manager(
        provider_manager: Arc<ProviderManager>,
    ) -> Result<Self> {
        // Verify at least one provider supports embeddings
        let embedding_providers: Vec<String> = provider_manager
            .list_providers()
            .into_iter()
            .filter(|name| {
                provider_manager
                    .get_provider(name)
                    .and_then(|p| p.as_any().downcast_ref::<dyn EmbeddingProvider>())
                    .is_some()
            })
            .collect();
        
        if embedding_providers.is_empty() {
            // Auto-configure a local provider if none available
            let local_config = ProviderConfig::new("local", "BAAI/bge-m3")
                .with_custom("embedding_model", json!("BAAI/bge-m3"));
            provider_manager.register_provider("local", local_config)?;
        }
        
        let current_provider = provider_manager
            .default_provider()
            .unwrap_or_else(|| embedding_providers.first().unwrap().clone());
        
        Ok(Self {
            chunker: DocumentChunker::new(ChunkingStrategy::default()),
            provider_manager,
            current_provider,
            vector_store: DynamicVectorStorage::new(),
            reranker: None,
        })
    }
    
    /// Switch to a different embedding provider
    pub fn set_provider(&mut self, provider: &str) -> Result<()> {
        // Verify provider exists and supports embeddings
        let provider_instance = self.provider_manager
            .get_provider(provider)
            .ok_or_else(|| anyhow!("Provider {} not found", provider))?;
        
        provider_instance
            .as_any()
            .downcast_ref::<dyn EmbeddingProvider>()
            .ok_or_else(|| anyhow!("Provider {} doesn't support embeddings", provider))?;
        
        self.current_provider = provider.to_string();
        Ok(())
    }
    
    /// Get embedding provider with cost consideration
    fn select_provider(&self, consider_cost: bool) -> Result<Arc<dyn EmbeddingProvider>> {
        if consider_cost {
            // Select cheapest available embedding provider
            let cheapest = self.provider_manager
                .list_providers()
                .into_iter()
                .filter_map(|name| {
                    self.provider_manager.get_provider(&name)
                        .and_then(|p| p.as_any().downcast_ref::<dyn EmbeddingProvider>())
                        .and_then(|ep| ep.embedding_cost_per_token()
                            .map(|cost| (name, cost)))
                })
                .min_by_key(|(_, cost)| (*cost * 1_000_000.0) as u64)
                .map(|(name, _)| name)
                .unwrap_or(self.current_provider.clone());
            
            self.provider_manager.get_provider(&cheapest)
                .and_then(|p| p.as_any().downcast_ref::<dyn EmbeddingProvider>())
                .map(Arc::from)
                .ok_or_else(|| anyhow!("Failed to get embedding provider"))
        } else {
            self.provider_manager.get_provider(&self.current_provider)
                .and_then(|p| p.as_any().downcast_ref::<dyn EmbeddingProvider>())
                .map(Arc::from)
                .ok_or_else(|| anyhow!("Current provider doesn't support embeddings"))
        }
    }
    
    pub async fn ingest(&mut self, documents: Vec<Document>) -> Result<IngestStats> {
        let mut stats = IngestStats::default();
        
        // Select provider (could use cost-aware routing)
        let provider = self.select_provider(true)?;  // Use cheapest for bulk ingestion
        let dimensions = provider.embedding_dimensions();
        
        // Get or create collection for these dimensions
        let collection_name = format!("{}_{}d", provider.name(), dimensions);
        let storage = self.vector_store
            .get_or_create_collection(&collection_name, dimensions)
            .await?;
        
        for document in documents {
            // Chunk document
            let chunks = self.chunker.chunk(&document).await?;
            stats.total_chunks += chunks.len();
            
            // Generate embeddings
            let texts: Vec<String> = chunks.iter().map(|c| c.text.clone()).collect();
            let embeddings = provider.embed(&texts).await?;
            
            // Track costs if available
            if let Some(cost_per_token) = provider.embedding_cost_per_token() {
                let total_tokens: usize = texts.iter().map(|t| t.len() / 4).sum();  // Rough estimate
                stats.embedding_cost += cost_per_token * total_tokens as f64;
            }
            
            // Create vector entries with provider metadata
            let entries: Vec<VectorEntry> = chunks.into_iter()
                .zip(embeddings.into_iter())
                .map(|(chunk, embedding)| {
                    let mut metadata = chunk.metadata;
                    metadata.insert("embedding_model".to_string(), 
                        json!(provider.embedding_model().unwrap_or("default")));
                    metadata.insert("embedding_provider".to_string(), json!(provider.name()));
                    metadata.insert("dimensions".to_string(), json!(dimensions));
                    
                    VectorEntry {
                        id: chunk.id,
                        vector: embedding,
                        metadata,
                        timestamp: chrono::Utc::now().timestamp(),
                    }
                })
                .collect();
            
            // Store in appropriate collection
            let ids = storage.insert(entries).await?;
            stats.vectors_stored += ids.len();
        }
        
        Ok(stats)
    }
    
    pub async fn retrieve(&self, query: &str, k: usize) -> Result<Vec<RetrievedChunk>> {
        // Embed query
        let query_embedding = self.embedder.embed(&[query.to_string()]).await?[0].clone();
        
        // Search vector store
        let vector_query = VectorQuery {
            vector: query_embedding,
            k,
            filter: None,
        };
        
        let results = self.vector_store.search(&vector_query).await?;
        
        // Optional reranking
        let final_results = if let Some(reranker) = &self.reranker {
            reranker.rerank(query, results).await?
        } else {
            results
        };
        
        Ok(final_results.into_iter()
            .map(|r| RetrievedChunk::from(r))
            .collect())
    }
}
```

---

## 4. Component Integration

### 4.1 Agent-Vector Integration

Agents can leverage vector storage for memory and context:

```rust
// llmspell-agents/src/vector_enhanced.rs
use llmspell_rag::prelude::*;

/// Vector-enhanced agent with semantic memory
pub struct VectorEnhancedAgent {
    base_agent: Box<dyn BaseAgent>,
    vector_memory: Arc<RAGPipeline>,
    context_builder: ContextBuilder,
}

#[async_trait]
impl BaseAgent for VectorEnhancedAgent {
    async fn execute_impl(
        &self,
        input: AgentInput,
        context: ExecutionContext,
    ) -> Result<AgentOutput> {
        // Retrieve relevant context from vector memory
        let retrieved_context = self.vector_memory
            .retrieve(&input.text, 5)
            .await?;
        
        // Build enhanced context
        let enhanced_input = self.context_builder
            .with_base_input(input)
            .with_retrieved_context(retrieved_context)
            .with_session_context(&context)
            .build()?;
        
        // Execute with enhanced context
        let output = self.base_agent.execute_impl(enhanced_input, context).await?;
        
        // Store interaction in vector memory (async, non-blocking)
        let document = Document::from_interaction(&input, &output);
        tokio::spawn({
            let memory = self.vector_memory.clone();
            async move {
                let _ = memory.ingest(vec![document]).await;
            }
        });
        
        Ok(output)
    }
}
```

### 4.2 Tool-RAG Integration

New semantic search tools leveraging the RAG infrastructure:

```rust
// llmspell-tools/src/search/semantic_search.rs
use llmspell_rag::prelude::*;
use llmspell_core::traits::Tool;

pub struct SemanticSearchTool {
    pipeline: Arc<RAGPipeline>,
    config: SearchConfig,
}

#[async_trait]
impl Tool for SemanticSearchTool {
    fn metadata(&self) -> &ToolMetadata {
        &self.metadata
    }
    
    async fn execute(&self, input: ToolInput) -> Result<ToolOutput> {
        let query = input.get_required::<String>("query")?;
        let k = input.get_optional::<usize>("k").unwrap_or(10);
        let filters = input.get_optional::<HashMap<String, Value>>("filters");
        
        // Perform semantic search
        let results = self.pipeline.retrieve(&query, k).await?;
        
        // Format results
        let output = ToolOutput::new()
            .with_data("results", results)
            .with_metadata("search_type", "semantic")
            .with_metadata("model", self.pipeline.embedder.model_id());
        
        Ok(output)
    }
}

// llmspell-tools/src/search/code_search.rs
pub struct CodeSearchTool {
    pipeline: Arc<RAGPipeline>,
    language_parsers: HashMap<String, Box<dyn LanguageParser>>,
}

impl CodeSearchTool {
    pub async fn index_repository(&self, repo_path: &Path) -> Result<IndexStats> {
        let mut documents = Vec::new();
        
        // Walk repository and parse code files
        for entry in walkdir::WalkDir::new(repo_path) {
            let entry = entry?;
            if let Some(lang) = self.detect_language(&entry.path()) {
                if let Some(parser) = self.language_parsers.get(&lang) {
                    let symbols = parser.parse_file(&entry.path()).await?;
                    documents.push(Document::from_code(entry.path(), symbols));
                }
            }
        }
        
        // Ingest into vector store
        self.pipeline.ingest(documents).await
    }
}
```

### 4.3 Workflow Orchestration

Workflows can coordinate complex RAG operations:

```rust
// llmspell-workflows/src/rag_workflow.rs
use llmspell_rag::prelude::*;
use llmspell_workflows::{Workflow, WorkflowStep};

pub struct RAGWorkflow {
    steps: Vec<WorkflowStep>,
    pipeline: Arc<RAGPipeline>,
}

impl RAGWorkflow {
    pub fn builder() -> RAGWorkflowBuilder {
        RAGWorkflowBuilder::new()
    }
}

pub struct RAGWorkflowBuilder {
    steps: Vec<WorkflowStep>,
}

impl RAGWorkflowBuilder {
    pub fn with_document_ingestion(self) -> Self {
        self.add_step(WorkflowStep::DocumentIngestion {
            source: DocumentSource::FileSystem,
            chunking: ChunkingStrategy::default(),
        })
    }
    
    pub fn with_embedding_generation(self, model: EmbeddingModel) -> Self {
        self.add_step(WorkflowStep::EmbeddingGeneration { model })
    }
    
    pub fn with_vector_storage(self, store: VectorStorage) -> Self {
        self.add_step(WorkflowStep::VectorStorage { store })
    }
    
    pub fn with_retrieval(self, strategy: RetrievalStrategy) -> Self {
        self.add_step(WorkflowStep::Retrieval { strategy })
    }
    
    pub fn build(self) -> RAGWorkflow {
        RAGWorkflow {
            steps: self.steps,
            pipeline: Arc::new(RAGPipeline::from_steps(self.steps)),
        }
    }
}
```

### 4.4 State Integration

Deep integration with StateManager for scope-aware vector storage:

```rust
// llmspell-rag/src/state_integration.rs
use llmspell_state_persistence::{StateManager, StateScope};
use llmspell_rag::prelude::*;

/// State-aware vector storage with automatic scoping
pub struct StateAwareVectorStorage {
    storage: Arc<dyn VectorStorage>,
    state_manager: Arc<StateManager>,
    scope_mapper: ScopeToNamespaceMapper,
}

impl StateAwareVectorStorage {
    /// Insert vectors with automatic state scope binding
    pub async fn insert_with_scope(
        &self,
        vectors: Vec<VectorEntry>,
        scope: StateScope,
    ) -> Result<Vec<String>> {
        // Map scope to isolated namespace
        let namespace = self.scope_mapper.get_namespace(&scope)?;
        
        // Track vector metadata in state
        let metadata = json!({
            "namespace": namespace,
            "vector_count": vectors.len(),
            "dimensions": vectors[0].embedding.len(),
            "timestamp": SystemTime::now(),
            "storage_bytes": vectors.iter().map(|v| v.embedding.len() * 4).sum::<usize>()
        });
        
        self.state_manager.set(
            scope.clone(),
            "rag:vector_metadata",
            metadata
        ).await?;
        
        // Store vectors with scope information
        let scoped_vectors: Vec<VectorEntry> = vectors.into_iter()
            .map(|mut v| {
                v.scope = scope.clone();
                v
            })
            .collect();
        
        self.storage.insert(scoped_vectors).await
    }
    
    /// Retrieve vectors within a specific scope
    pub async fn search_in_scope(
        &self,
        query: &str,
        scope: &StateScope,
        k: usize,
    ) -> Result<Vec<RetrievedChunk>> {
        // Build scoped query
        let vector_query = VectorQuery {
            text: query.to_string(),
            scope: Some(scope.clone()),
            k,
            ..Default::default()
        };
        
        // Track search in state for analytics
        self.state_manager.increment(
            scope.clone(),
            "rag:search_count"
        ).await?;
        
        self.storage.search_scoped(&vector_query, scope).await
    }
    
    /// Cleanup vectors when scope is deleted
    pub async fn cleanup_scope(&self, scope: &StateScope) -> Result<usize> {
        let deleted = self.storage.delete_scope(scope).await?;
        
        // Clear state metadata
        self.state_manager.delete(scope.clone(), "rag:vector_metadata").await?;
        self.state_manager.delete(scope.clone(), "rag:search_count").await?;
        
        Ok(deleted)
    }
}
```

### 4.5 Session Integration

Session-aware RAG with artifact storage:

```rust
// llmspell-rag/src/session_integration.rs
use llmspell_sessions::{SessionManager, SessionId, SessionArtifact, ArtifactType};
use llmspell_rag::prelude::*;

/// Session-aware RAG pipeline with artifact tracking
pub struct SessionAwareRAGPipeline {
    pipeline: Arc<RAGPipeline>,
    session_manager: Arc<SessionManager>,
    state_aware_storage: Arc<StateAwareVectorStorage>,
}

impl SessionAwareRAGPipeline {
    /// Create session-bound vector collection
    pub async fn create_session_collection(
        &self,
        session_id: SessionId,
        ttl_seconds: Option<u64>,
    ) -> Result<SessionVectorCollection> {
        // Validate session is active
        let session = self.session_manager.get_session(&session_id)?;
        
        // Create session scope
        let scope = StateScope::Session(session_id.to_string());
        
        // Initialize collection with TTL
        let collection = SessionVectorCollection {
            session_id,
            namespace: format!("session_{}", session_id),
            vector_count: 0,
            total_tokens: 0,
            embedding_provider: self.pipeline.current_provider().name,
            created_at: SystemTime::now(),
            expires_at: ttl_seconds.map(|s| SystemTime::now() + Duration::from_secs(s)),
        };
        
        // Store as session artifact
        self.session_manager.add_artifact(
            &session_id,
            SessionArtifact {
                id: ArtifactId::new(),
                artifact_type: ArtifactType::VectorCollection(collection.clone()),
                created_at: SystemTime::now(),
                metadata: json!({
                    "namespace": collection.namespace,
                    "ttl_seconds": ttl_seconds,
                }),
            }
        ).await?;
        
        Ok(collection)
    }
    
    /// Retrieve within session context
    pub async fn retrieve_in_session(
        &self,
        query: &str,
        session_id: SessionId,
        k: usize,
    ) -> Result<Vec<RetrievedChunk>> {
        // Validate session access
        let session = self.session_manager.get_session(&session_id)?;
        
        // Create session scope for search
        let scope = StateScope::Session(session_id.to_string());
        
        // Perform scoped search
        let results = self.state_aware_storage
            .search_in_scope(query, &scope, k)
            .await?;
        
        // Store query and results as session artifact
        let artifact = SessionArtifact {
            id: ArtifactId::new(),
            artifact_type: ArtifactType::VectorQuery {
                query: query.to_string(),
                results: results.clone(),
                timestamp: SystemTime::now(),
                provider: self.pipeline.current_provider().name,
            },
            created_at: SystemTime::now(),
            metadata: json!({
                "query": query,
                "result_count": results.len(),
                "session_id": session_id.to_string(),
            }),
        };
        
        self.session_manager.add_artifact(&session_id, artifact).await?;
        
        Ok(results)
    }
    
    /// Ingest documents into session-bound storage
    pub async fn ingest_in_session(
        &self,
        documents: Vec<Document>,
        session_id: SessionId,
    ) -> Result<IngestStats> {
        let scope = StateScope::Session(session_id.to_string());
        
        // Generate embeddings
        let vectors = self.pipeline.embed_documents(&documents).await?;
        
        // Store with session scope
        let ids = self.state_aware_storage
            .insert_with_scope(vectors, scope)
            .await?;
        
        // Update session collection metadata
        let stats = IngestStats {
            documents_processed: documents.len(),
            vectors_created: ids.len(),
            tokens_consumed: documents.iter().map(|d| d.token_count()).sum(),
        };
        
        // Store ingestion record as artifact
        self.session_manager.add_artifact(
            &session_id,
            SessionArtifact {
                id: ArtifactId::new(),
                artifact_type: ArtifactType::IngestionRecord(stats.clone()),
                created_at: SystemTime::now(),
                metadata: json!({ "document_count": documents.len() }),
            }
        ).await?;
        
        Ok(stats)
    }
}

/// Session-bound vector collection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionVectorCollection {
    pub session_id: SessionId,
    pub namespace: String,
    pub vector_count: usize,
    pub total_tokens: usize,
    pub embedding_provider: String,
    pub created_at: SystemTime,
    pub expires_at: Option<SystemTime>,
}
```

### 4.6 Security Integration

Security policies and access control for vector operations:

```rust
// llmspell-rag/src/security.rs
use llmspell_security::{SandboxContext, SandboxViolation};
use llmspell_sessions::security::SessionSecurityManager;

/// Vector access policy with RLS-inspired rules
#[derive(Debug, Clone)]
pub struct VectorAccessPolicy {
    pub tenant_id: String,
    pub allowed_operations: Vec<VectorOperation>,
    pub metadata_filters: HashMap<String, Value>,
    pub dimension_limit: Option<usize>,
    pub query_limit_per_minute: Option<usize>,
}

/// Security manager for vector operations
pub struct VectorSecurityManager {
    session_security: Arc<SessionSecurityManager>,
    access_policies: HashMap<String, VectorAccessPolicy>,
    sandbox_context: Arc<SandboxContext>,
}

impl VectorSecurityManager {
    /// Validate vector operation against security policies
    pub async fn validate_operation(
        &self,
        operation: &VectorOperation,
        scope: &StateScope,
        session_id: Option<SessionId>,
    ) -> Result<()> {
        // Check session-level access if session provided
        if let Some(sid) = session_id {
            if let StateScope::Session(target_session) = scope {
                self.session_security.validate_cross_session_access(
                    &sid,
                    &target_session.parse()?,
                    "vector_access"
                )?;
            }
        }
        
        // Get tenant from scope
        let tenant_id = match scope {
            StateScope::User(id) => id.clone(),
            StateScope::Session(sid) => {
                // Extract tenant from session
                sid.split('_').next().unwrap_or("default").to_string()
            }
            _ => "default".to_string(),
        };
        
        // Check tenant policy
        if let Some(policy) = self.access_policies.get(&tenant_id) {
            // Validate operation is allowed
            if !policy.allowed_operations.contains(operation) {
                return Err(SecurityError::OperationDenied {
                    operation: format!("{:?}", operation),
                    tenant: tenant_id,
                });
            }
            
            // Check rate limits
            if let Some(limit) = policy.query_limit_per_minute {
                self.check_rate_limit(&tenant_id, limit).await?;
            }
        }
        
        Ok(())
    }
    
    /// Apply Row-Level Security filters to query
    pub fn apply_rls_filters(
        &self,
        query: &mut VectorQuery,
        scope: &StateScope,
    ) -> Result<()> {
        // Add scope filter
        query.metadata_filters.insert(
            "scope".to_string(),
            json!(scope.to_string())
        );
        
        // Add tenant filter if applicable
        if let StateScope::User(tenant_id) = scope {
            if let Some(policy) = self.access_policies.get(tenant_id) {
                // Apply policy metadata filters
                for (key, value) in &policy.metadata_filters {
                    query.metadata_filters.insert(key.clone(), value.clone());
                }
            }
        }
        
        Ok(())
    }
    
    /// Sandbox vector operations to prevent resource abuse
    pub async fn sandbox_operation<F, R>(
        &self,
        operation: VectorOperation,
        scope: &StateScope,
        f: F,
    ) -> Result<R>
    where
        F: FnOnce() -> Result<R> + Send,
        R: Send,
    {
        // Set resource limits based on scope
        let limits = self.get_resource_limits(scope);
        
        self.sandbox_context.execute_with_limits(limits, f).await
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum VectorOperation {
    Search,
    Insert,
    Update,
    Delete,
    CreateNamespace,
    DeleteNamespace,
}
```

### 4.7 Multi-Tenant Architecture

Complete multi-tenant support with isolation strategies:

```rust
// llmspell-rag/src/multi_tenant.rs
use llmspell_state_traits::StateScope;

/// Tenant isolation strategies
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TenantIsolationStrategy {
    /// Separate HNSW index per tenant (strongest isolation)
    DatabasePerTenant,
    /// Logical namespace partitioning (recommended)
    NamespacePerTenant,
    /// Query-time metadata filtering (flexible but slower)
    MetadataFiltering,
    /// Hybrid approach combining namespace and filtering
    Hybrid,
}

/// Multi-tenant vector manager
pub struct MultiTenantVectorManager {
    storage: Arc<dyn VectorStorage>,
    isolation_strategy: TenantIsolationStrategy,
    usage_tracker: Arc<TenantUsageTracker>,
    access_policies: HashMap<String, VectorAccessPolicy>,
    security_manager: Arc<VectorSecurityManager>,
}

impl MultiTenantVectorManager {
    /// Create isolated namespace for new tenant
    pub async fn create_tenant_namespace(
        &self,
        tenant_id: &str,
        config: TenantConfig,
    ) -> Result<()> {
        match self.isolation_strategy {
            TenantIsolationStrategy::DatabasePerTenant => {
                // Create completely isolated HNSW index
                let storage = self.create_isolated_storage(&config)?;
                self.register_tenant_storage(tenant_id, storage).await?;
            }
            TenantIsolationStrategy::NamespacePerTenant => {
                // Create namespace within shared storage
                let namespace = format!("tenant_{}", tenant_id);
                self.storage.create_namespace(&namespace).await?;
            }
            TenantIsolationStrategy::MetadataFiltering => {
                // No physical isolation, just metadata
                self.register_tenant_metadata(tenant_id, config).await?;
            }
            TenantIsolationStrategy::Hybrid => {
                // Combine namespace and metadata
                let namespace = format!("tenant_{}", tenant_id);
                self.storage.create_namespace(&namespace).await?;
                self.register_tenant_metadata(tenant_id, config).await?;
            }
        }
        
        // Initialize usage tracking
        self.usage_tracker.init_tenant(tenant_id, config.limits).await?;
        
        // Set access policy
        self.access_policies.insert(
            tenant_id.to_string(),
            config.access_policy,
        );
        
        Ok(())
    }
    
    /// Perform tenant-scoped vector search
    pub async fn search_for_tenant(
        &self,
        tenant_id: &str,
        query: &str,
        k: usize,
    ) -> Result<Vec<RetrievedChunk>> {
        // Create tenant scope
        let scope = StateScope::User(tenant_id.to_string());
        
        // Validate access
        self.security_manager.validate_operation(
            &VectorOperation::Search,
            &scope,
            None,
        ).await?;
        
        // Track usage
        self.usage_tracker.track_search(tenant_id).await?;
        
        // Build scoped query
        let mut vector_query = VectorQuery {
            text: query.to_string(),
            scope: Some(scope.clone()),
            k,
            ..Default::default()
        };
        
        // Apply RLS filters
        self.security_manager.apply_rls_filters(&mut vector_query, &scope)?;
        
        // Route based on isolation strategy
        match self.isolation_strategy {
            TenantIsolationStrategy::DatabasePerTenant => {
                let tenant_storage = self.get_tenant_storage(tenant_id)?;
                tenant_storage.search(&vector_query).await
            }
            _ => {
                self.storage.search_scoped(&vector_query, &scope).await
            }
        }
    }
    
    /// Get usage statistics for tenant
    pub async fn get_tenant_usage(
        &self,
        tenant_id: &str,
    ) -> Result<TenantUsageStats> {
        self.usage_tracker.get_stats(tenant_id).await
    }
}

/// Tenant usage tracking
pub struct TenantUsageTracker {
    state_manager: Arc<StateManager>,
    cost_calculator: Arc<CostCalculator>,
}

impl TenantUsageTracker {
    pub async fn track_embedding_generation(
        &self,
        tenant_id: &str,
        provider: &str,
        token_count: usize,
        dimensions: usize,
    ) -> Result<()> {
        let scope = StateScope::User(tenant_id.to_string());
        
        // Update counters
        self.state_manager.increment(
            scope.clone(),
            "usage:embeddings_generated"
        ).await?;
        
        self.state_manager.increment_by(
            scope.clone(),
            "usage:tokens_consumed",
            token_count as i64
        ).await?;
        
        // Calculate and track costs
        let cost = self.cost_calculator.calculate_embedding_cost(
            provider,
            token_count,
            dimensions
        )?;
        
        self.state_manager.increment_by_float(
            scope,
            "usage:total_cost",
            cost
        ).await?;
        
        Ok(())
    }
    
    pub async fn check_limits(&self, tenant_id: &str) -> Result<()> {
        let scope = StateScope::User(tenant_id.to_string());
        
        // Get current usage
        let usage = self.state_manager.get(
            scope.clone(),
            "usage:total_cost"
        ).await?.unwrap_or(json!(0.0));
        
        // Get limits
        let limits = self.state_manager.get(
            scope,
            "config:spending_limit"
        ).await?.unwrap_or(json!(100.0));
        
        if usage.as_f64().unwrap_or(0.0) >= limits.as_f64().unwrap_or(100.0) {
            return Err(UsageError::LimitExceeded {
                tenant: tenant_id.to_string(),
                usage: usage.as_f64().unwrap_or(0.0),
                limit: limits.as_f64().unwrap_or(100.0),
            });
        }
        
        Ok(())
    }
}

/// Tenant configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TenantConfig {
    pub access_policy: VectorAccessPolicy,
    pub limits: TenantLimits,
    pub isolation_level: TenantIsolationStrategy,
    pub default_provider: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TenantLimits {
    pub max_vectors: usize,
    pub max_dimensions: usize,
    pub max_queries_per_minute: usize,
    pub max_storage_bytes: usize,
    pub monthly_spending_limit: f64,
}
```

---

## 5. Hook and Event Integration

### 5.1 Vector Operation Hooks

All vector operations emit events and support hooks:

```rust
// llmspell-rag/src/hooks/vector_hooks.rs
use llmspell_hooks::{Hook, HookContext, HookPoint, HookResult};
use llmspell_events::{EventBus, UniversalEvent};

#[derive(Debug, Clone)]
pub enum VectorHookPoint {
    BeforeEmbedding,
    AfterEmbedding,
    BeforeVectorInsert,
    AfterVectorInsert,
    BeforeVectorSearch,
    AfterVectorSearch,
    BeforeRerank,
    AfterRerank,
}

pub struct VectorEventHook {
    event_bus: Arc<EventBus>,
}

#[async_trait]
impl Hook for VectorEventHook {
    async fn execute(&self, context: &mut HookContext) -> Result<HookResult> {
        let event = UniversalEvent::new(
            &format!("vector.{:?}", context.point),
            json!({
                "operation": context.get::<String>("operation")?,
                "vector_count": context.get::<usize>("vector_count")?,
                "dimensions": context.get::<usize>("dimensions")?,
                "model": context.get:challenging"model")?,
                "duration_ms": context.get::<u64>("duration_ms")?,
            }),
            Language::Rust,
        );
        
        self.event_bus.publish(event).await?;
        Ok(HookResult::Continue)
    }
}
```

### 5.2 Performance Monitoring Hooks

```rust
// llmspell-rag/src/hooks/performance.rs
pub struct EmbeddingPerformanceHook {
    metrics: Arc<Mutex<EmbeddingMetrics>>,
}

#[derive(Default)]
pub struct EmbeddingMetrics {
    pub total_embeddings: u64,
    pub total_tokens: u64,
    pub avg_latency_ms: f64,
    pub p95_latency_ms: f64,
    pub p99_latency_ms: f64,
}

#[async_trait]
impl Hook for EmbeddingPerformanceHook {
    async fn execute(&self, context: &mut HookContext) -> Result<HookResult> {
        if context.point == VectorHookPoint::AfterEmbedding {
            let duration = context.get::<Duration>("duration")?;
            let token_count = context.get::<usize>("token_count")?;
            
            let mut metrics = self.metrics.lock().await;
            metrics.total_embeddings += 1;
            metrics.total_tokens += token_count as u64;
            metrics.update_latency(duration.as_millis() as f64);
        }
        
        Ok(HookResult::Continue)
    }
}
```

---

## 6. Configuration Schema

### 6.1 RAG Configuration

Extend `llmspell-config` with RAG settings:

```toml
# llmspell.toml
# IMPORTANT: RAG leverages existing provider configurations
[providers]
default_provider = "openai"

[providers.openai]
provider_type = "openai"
api_key_env = "OPENAI_API_KEY"
default_model = "gpt-4"
# Embedding-specific options in provider config
[providers.openai.options]
embedding_model = "text-embedding-3-small"
embedding_dimensions = 1536  # Can reduce to 512 for Matryoshka

[providers.anthropic]
provider_type = "anthropic"
api_key_env = "ANTHROPIC_API_KEY"
default_model = "claude-3-opus-20240229"
# Anthropic doesn't provide embeddings, use Voyage AI
[providers.anthropic.options]
embedding_provider = "voyage"  # Delegate to Voyage

[providers.voyage]
provider_type = "voyage"
api_key_env = "VOYAGE_API_KEY"
[providers.voyage.options]
embedding_model = "voyage-2"

[providers.google]
provider_type = "google"
api_key_env = "GOOGLE_API_KEY"
[providers.google.options]
embedding_model = "text-embedding-004"
embedding_dimensions = 768

[providers.local]
provider_type = "local"
[providers.local.options]
embedding_model = "BAAI/bge-m3"
embedding_dimensions = 1024
device = "cuda"  # or "cpu", "metal"

[rag]
# Vector storage backend
storage_backend = "embedded"  # "embedded", "memvdb", "qdrant"
# Use provider embeddings by default
embedding_provider = "default"  # Uses default_provider's embeddings
# Or override with specific provider
# embedding_provider = "local"  

# HNSW parameters
[rag.hnsw]
m = 32
ef_construction = 200
ef_search = 100
max_elements = 1_000_000

# Chunking configuration
[rag.chunking]
method = "sliding_window"  # "sliding_window", "semantic", "structural"
max_tokens = 512
overlap_tokens = 50

# Retrieval configuration
[rag.retrieval]
default_k = 10
rerank_enabled = true
rerank_model = "cross-encoder/ms-marco-MiniLM-L-12-v2"
hybrid_weights = { vector = 0.7, keyword = 0.2, metadata = 0.1 }

# Performance settings
[rag.performance]
embedding_cache_size = 10000
vector_cache_size = 100000
parallel_ingestion = true
ingestion_batch_size = 100

# Multi-tenancy configuration
[rag.multi_tenancy]
enabled = true
isolation_strategy = "namespace_per_tenant"  # "database_per_tenant", "metadata_filtering", "hybrid"
enable_cost_tracking = true
enable_usage_limits = true
default_tenant_limits = { max_vectors = 100000, max_queries_per_minute = 100, monthly_spending_limit = 100.0 }

# Security configuration
[rag.security]
enforce_scope_isolation = true
enable_rls_policies = true  # PostgreSQL-style row-level security
cross_tenant_access = "deny"  # "allow", "deny", "allow_with_permission"
enable_sandboxing = true

# Session integration
[rag.session_integration]
bind_vectors_to_sessions = true
session_vector_ttl = 86400  # 24 hours in seconds
store_queries_as_artifacts = true
enable_session_collections = true

# State integration
[rag.state_integration]
track_vector_metadata = true
scope_aware_storage = true
auto_cleanup_on_scope_delete = true
```

### 6.2 Configuration Rust Structure

```rust
// llmspell-config/src/rag.rs
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(default)]
pub struct RAGConfig {
    pub storage_backend: StorageBackend,
    pub hnsw: HNSWConfig,
    pub embedding: EmbeddingConfig,
    pub late_interaction: LateInteractionConfig,
    pub chunking: ChunkingConfig,
    pub retrieval: RetrievalConfig,
    pub performance: PerformanceConfig,
    pub multi_tenancy: MultiTenancyConfig,
    pub security: SecurityConfig,
    pub session_integration: SessionIntegrationConfig,
    pub state_integration: StateIntegrationConfig,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum StorageBackend {
    #[serde(rename = "embedded")]
    Embedded,
    #[serde(rename = "memvdb")]
    MemVDB,
    #[serde(rename = "qdrant")]
    Qdrant { url: String, api_key: Option<String> },
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(default)]
pub struct MultiTenancyConfig {
    pub enabled: bool,
    pub isolation_strategy: TenantIsolationStrategy,
    pub enable_cost_tracking: bool,
    pub enable_usage_limits: bool,
    pub default_tenant_limits: TenantLimits,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(default)]
pub struct SecurityConfig {
    pub enforce_scope_isolation: bool,
    pub enable_rls_policies: bool,
    pub cross_tenant_access: CrossTenantAccess,
    pub enable_sandboxing: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum CrossTenantAccess {
    #[serde(rename = "allow")]
    Allow,
    #[serde(rename = "deny")]
    Deny,
    #[serde(rename = "allow_with_permission")]
    AllowWithPermission,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(default)]
pub struct SessionIntegrationConfig {
    pub bind_vectors_to_sessions: bool,
    pub session_vector_ttl: u64,  // seconds
    pub store_queries_as_artifacts: bool,
    pub enable_session_collections: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(default)]
pub struct StateIntegrationConfig {
    pub track_vector_metadata: bool,
    pub scope_aware_storage: bool,
    pub auto_cleanup_on_scope_delete: bool,
}

impl Default for RAGConfig {
    fn default() -> Self {
        Self {
            storage_backend: StorageBackend::Embedded,
            hnsw: HNSWConfig::default(),
            embedding: EmbeddingConfig::default(),
            late_interaction: LateInteractionConfig::default(),
            chunking: ChunkingConfig::default(),
            retrieval: RetrievalConfig::default(),
            performance: PerformanceConfig::default(),
        }
    }
}
```

---

## 7. Bridge Layer Integration

The RAG system follows rs-llmspell's three-layer bridge architecture:

### 7.1 Native Rust Bridge Layer

The foundation is the native Rust implementation that provides core RAG functionality:

```rust
// llmspell-bridge/src/rag_bridge.rs
use llmspell_rag::prelude::*;
use llmspell_providers::ProviderManager;
use llmspell_core::Result;
use std::sync::Arc;

/// Native RAG bridge providing core functionality with multi-tenant support
pub struct RAGBridge {
    pipeline: Arc<RAGPipeline>,
    provider_manager: Arc<ProviderManager>,
    state_manager: Arc<StateManager>,
    session_manager: Option<Arc<SessionManager>>,
    multi_tenant_manager: Arc<MultiTenantVectorManager>,
}

impl RAGBridge {
    /// Create RAG bridge with full integration support
    pub async fn new(
        provider_manager: Arc<ProviderManager>,
        state_manager: Arc<StateManager>,
        session_manager: Option<Arc<SessionManager>>,
    ) -> Result<Self> {
        let pipeline = Arc::new(
            RAGPipeline::from_provider_manager(provider_manager.clone()).await?
        );
        
        let multi_tenant_manager = Arc::new(
            MultiTenantVectorManager::new(
                pipeline.storage.clone(),
                state_manager.clone(),
            )
        );
        
        Ok(Self {
            pipeline,
            provider_manager,
            state_manager,
            session_manager,
            multi_tenant_manager,
        })
    }
    
    /// Search for similar documents with scope isolation
    pub async fn search(
        &self,
        query: &str,
        k: usize,
        scope: Option<&StateScope>,
        provider: Option<&str>,
    ) -> Result<Vec<RetrievedChunk>> {
        // Apply scope if provided
        if let Some(s) = scope {
            // Use tenant-specific search
            if let StateScope::User(tenant_id) = s {
                return self.multi_tenant_manager
                    .search_for_tenant(tenant_id, query, k)
                    .await;
            }
            
            // Use state-aware search for other scopes
            let state_storage = StateAwareVectorStorage::new(
                self.pipeline.storage.clone(),
                self.state_manager.clone(),
            );
            return state_storage.search_in_scope(query, s, k).await;
        }
        
        // Default non-scoped search
        let mut pipeline = (*self.pipeline).clone();
        if let Some(p) = provider {
            pipeline.set_provider(p)?;
        }
        pipeline.retrieve(query, k).await
    }
    
    /// Ingest documents into vector storage with scope
    pub async fn ingest(
        &self,
        documents: Vec<Document>,
        use_cheapest: bool,
    ) -> Result<IngestStats> {
        let mut pipeline = (*self.pipeline).clone();
        
        if use_cheapest {
            // Pipeline will automatically select cheapest provider
        }
        
        pipeline.ingest(documents).await
    }
    
    /// Generate embeddings directly
    pub async fn embed(
        &self,
        texts: &[String],
        provider: Option<&str>,
    ) -> Result<Vec<Vec<f32>>> {
        let provider_name = provider.unwrap_or(&self.pipeline.current_provider);
        
        let provider = self.provider_manager
            .get_provider(provider_name)
            .and_then(|p| p.as_any().downcast_ref::<dyn EmbeddingProvider>())
            .ok_or_else(|| anyhow!("Provider {} not found or doesn't support embeddings", provider_name))?;
        
        provider.embed(texts).await
    }
    
    /// Switch embedding provider
    pub fn set_provider(&mut self, provider: &str) -> Result<()> {
        Arc::get_mut(&mut self.pipeline)
            .ok_or_else(|| anyhow!("Cannot modify shared pipeline"))?
            .set_provider(provider)
    }
    
    /// Get current embedding provider info
    pub fn current_provider_info(&self) -> ProviderInfo {
        ProviderInfo {
            name: self.pipeline.current_provider.clone(),
            dimensions: self.get_current_dimensions(),
            supports_dimension_reduction: self.supports_dimension_reduction(),
            cost_per_token: self.get_cost_per_token(),
        }
    }
}
```

### 7.2 Global Object Layer

The global object implements the `GlobalObject` trait to provide language-agnostic interface:

```rust
// llmspell-bridge/src/globals/rag_global.rs
use super::types::{GlobalContext, GlobalMetadata, GlobalObject};
use crate::rag_bridge::RAGBridge;
use llmspell_core::Result;
use std::sync::Arc;

/// RAG global object for script engines
pub struct RAGGlobal {
    bridge: Arc<RAGBridge>,
}

impl RAGGlobal {
    /// Create a new RAG global with provider integration
    pub async fn new(context: &GlobalContext) -> Result<Self> {
        // Get provider manager from context
        let provider_manager = context
            .get_bridge::<llmspell_providers::ProviderManager>("provider_manager")
            .ok_or_else(|| llmspell_core::LLMSpellError::Configuration {
                message: "Provider manager not found in context".to_string(),
                source: None,
            })?;
        
        let bridge = Arc::new(RAGBridge::new(provider_manager).await?);
        
        Ok(Self { bridge })
    }
    
    /// Get the RAG bridge
    pub fn bridge(&self) -> &Arc<RAGBridge> {
        &self.bridge
    }
}

impl GlobalObject for RAGGlobal {
    fn metadata(&self) -> GlobalMetadata {
        GlobalMetadata {
            name: "RAG".to_string(),
            description: "Retrieval-Augmented Generation with multi-provider embeddings".to_string(),
            dependencies: vec!["Provider".to_string()],  // Depends on provider system
            required: false,  // Optional feature
            version: "1.0.0".to_string(),
        }
    }
    
    #[cfg(feature = "lua")]
    fn inject_lua(&self, lua: &mlua::Lua, context: &GlobalContext) -> Result<()> {
        crate::lua::globals::rag::inject_rag_global(lua, context, self.bridge.clone())
            .map_err(|e| llmspell_core::LLMSpellError::Component {
                message: format!("Failed to inject RAG global: {}", e),
                source: None,
            })
    }
    
    #[cfg(feature = "javascript")]
    fn inject_javascript(
        &self,
        ctx: &mut boa_engine::Context,
        context: &GlobalContext,
    ) -> Result<()> {
        crate::javascript::globals::rag::inject_rag_global(ctx, context, self.bridge.clone())
            .map_err(|e| llmspell_core::LLMSpellError::Component {
                message: format!("Failed to inject RAG global for JavaScript: {}", e),
                source: None,
            })
    }
}
```

### 7.3 Lua Language Implementation

The Lua-specific implementation creates the actual script interface:

```rust
// llmspell-bridge/src/lua/globals/rag.rs
use crate::globals::GlobalContext;
use crate::lua::conversion::json_to_lua_value;
use crate::lua::sync_utils::block_on_async_lua;
use crate::rag_bridge::RAGBridge;
use mlua::{Lua, Table, Value};
use std::sync::Arc;

/// Inject RAG global into Lua environment
pub fn inject_rag_global(
    lua: &Lua,
    context: &GlobalContext,
    bridge: Arc<RAGBridge>,
) -> mlua::Result<()> {
    let rag_table = lua.create_table()?;
    
    // RAG.search(query, options) with scope support
    let bridge_clone = bridge.clone();
    let search_fn = lua.create_function(move |lua, (query, options): (String, Option<Table>)| {
        let bridge = bridge_clone.clone();
        
        // Parse options
        let k = options.as_ref()
            .and_then(|t| t.get::<_, Option<i64>>("k").ok())
            .flatten()
            .unwrap_or(10) as usize;
        
        let provider = options.as_ref()
            .and_then(|t| t.get::<_, Option<String>>("provider").ok())
            .flatten();
        
        // Parse scope (tenant, session, user, etc.)
        let scope = options.as_ref()
            .and_then(|t| t.get::<_, Option<Table>>("scope").ok())
            .flatten()
            .and_then(|scope_table| {
                let scope_type = scope_table.get::<_, String>("type").ok()?;
                let scope_id = scope_table.get::<_, Option<String>>("id").ok().flatten();
                
                Some(match scope_type.as_str() {
                    "global" => StateScope::Global,
                    "user" => StateScope::User(scope_id?),
                    "session" => StateScope::Session(scope_id?),
                    "agent" => StateScope::Agent(scope_id?),
                    _ => return None,
                })
            });
        
        // Execute async search with scope
        let result = block_on_async_lua(
            "rag_search",
            async move {
                bridge.search(&query, k, scope.as_ref(), provider.as_deref()).await
            }
        )?;
        
        // Convert results to Lua table
        let results_table = lua.create_table()?;
        for (i, chunk) in result.into_iter().enumerate() {
            let chunk_table = lua.create_table()?;
            chunk_table.set("id", chunk.id)?;
            chunk_table.set("text", chunk.text)?;
            chunk_table.set("score", chunk.score)?;
            chunk_table.set("metadata", json_to_lua_value(lua, &chunk.metadata)?)?;
            results_table.set(i + 1, chunk_table)?;
        }
        
        Ok(results_table)
    })?;
    rag_table.set("search", search_fn)?;
    
    // RAG.ingest(documents, options)
    let bridge_clone = bridge.clone();
    let ingest_fn = lua.create_function(move |lua, (documents, options): (Table, Option<Table>)| {
        let bridge = bridge_clone.clone();
        
        // Convert Lua documents to Rust
        let mut docs = Vec::new();
        for pair in documents.pairs::<i64, Table>() {
            let (_, doc_table) = pair?;
            let id = doc_table.get::<_, String>("id")?;
            let text = doc_table.get::<_, String>("text")?;
            let metadata = doc_table.get::<_, Option<Table>>("metadata")?
                .map(|t| lua_table_to_json(lua, t))
                .transpose()?
                .unwrap_or_default();
            
            docs.push(Document { id, text, metadata });
        }
        
        let use_cheapest = options.as_ref()
            .and_then(|t| t.get::<_, Option<bool>>("use_cheapest").ok())
            .flatten()
            .unwrap_or(false);
        
        // Execute async ingestion
        let result = block_on_async_lua(
            "rag_ingest",
            async move {
                bridge.ingest(docs, use_cheapest).await
            }
        )?;
        
        // Return stats
        let stats_table = lua.create_table()?;
        stats_table.set("vectors_stored", result.vectors_stored)?;
        stats_table.set("total_chunks", result.total_chunks)?;
        stats_table.set("embedding_cost", result.embedding_cost)?;
        
        Ok(stats_table)
    })?;
    rag_table.set("ingest", ingest_fn)?;
    
    // RAG.embed(texts, provider)
    let bridge_clone = bridge.clone();
    let embed_fn = lua.create_function(move |lua, (texts, provider): (Vec<String>, Option<String>)| {
        let bridge = bridge_clone.clone();
        
        let result = block_on_async_lua(
            "rag_embed",
            async move {
                bridge.embed(&texts, provider.as_deref()).await
            }
        )?;
        
        // Convert embeddings to Lua nested tables
        let embeddings_table = lua.create_table()?;
        for (i, embedding) in result.into_iter().enumerate() {
            let vec_table = lua.create_table()?;
            for (j, val) in embedding.into_iter().enumerate() {
                vec_table.set(j + 1, val)?;
            }
            embeddings_table.set(i + 1, vec_table)?;
        }
        
        Ok(embeddings_table)
    })?;
    rag_table.set("embed", embed_fn)?;
    
    // RAG.set_provider(provider)
    let bridge_clone = bridge.clone();
    let set_provider_fn = lua.create_function(move |_, provider: String| {
        let mut bridge = (*bridge_clone).clone();
        bridge.set_provider(&provider)
            .map_err(mlua::Error::external)?;
        Ok(())
    })?;
    rag_table.set("set_provider", set_provider_fn)?;
    
    // RAG.current_provider()
    let bridge_clone = bridge.clone();
    let current_provider_fn = lua.create_function(move |lua, ()| {
        let info = bridge_clone.current_provider_info();
        
        let info_table = lua.create_table()?;
        info_table.set("name", info.name)?;
        info_table.set("dimensions", info.dimensions)?;
        info_table.set("supports_dimension_reduction", info.supports_dimension_reduction)?;
        if let Some(cost) = info.cost_per_token {
            info_table.set("cost_per_token", cost)?;
        }
        
        Ok(info_table)
    })?;
    rag_table.set("current_provider", current_provider_fn)?;
    
    // Register as global
    lua.globals().set("RAG", rag_table)?;
    
    Ok(())
}
```

### 7.4 JavaScript Language Implementation

Similar implementation for JavaScript:

```rust
// llmspell-bridge/src/javascript/globals/rag.rs
use crate::globals::GlobalContext;
use crate::rag_bridge::RAGBridge;
use boa_engine::{Context, JsValue, JsResult, JsObject};
use std::sync::Arc;

/// Inject RAG global into JavaScript environment
pub fn inject_rag_global(
    ctx: &mut Context,
    _context: &GlobalContext,
    bridge: Arc<RAGBridge>,
) -> JsResult<()> {
    let rag_object = JsObject::default();
    
    // Implementation similar to Lua but using Boa engine APIs
    // RAG.search, RAG.ingest, RAG.embed, etc.
    
    ctx.register_global_property("RAG", rag_object, boa_engine::property::Attribute::all());
    
    Ok(())
}
```

---

## 8. Examples and Learning Path

Following rs-llmspell's pedagogy approach, we provide progressive examples from basics to production patterns:

### 8.1 Getting Started Examples

These examples follow the learning path in `examples/script-users/getting-started/`:

```lua
-- examples/script-users/getting-started/06-first-rag.lua
-- First RAG example - semantic search basics
-- Prerequisites: None (uses local embeddings)
-- Expected output: Search results with similarity scores

-- Basic semantic search with local embeddings
local documents = {
    {id = "1", text = "Rust is a systems programming language"},
    {id = "2", text = "JavaScript is used for web development"},
    {id = "3", text = "Python is popular for data science"}
}

-- Ingest documents using default (local) provider
local stats = RAG.ingest(documents)
print("Ingested " .. stats.vectors_stored .. " vectors")

-- Search for similar content
local results = RAG.search("What language is good for system programming?")
for i, result in ipairs(results) do
    print(i .. ". " .. result.text .. " (score: " .. result.score .. ")")
end
```

```lua
-- examples/script-users/getting-started/07-rag-with-providers.lua
-- RAG with different embedding providers
-- Prerequisites: OPENAI_API_KEY or other provider API keys
-- Expected output: Cost comparison between providers

-- Check available providers
local provider_info = RAG.current_provider()
print("Current provider: " .. provider_info.name)
print("Dimensions: " .. provider_info.dimensions)

-- Compare costs between providers
if os.getenv("OPENAI_API_KEY") then
    -- Use OpenAI for high-quality embeddings
    RAG.set_provider("openai")
    local texts = {"Sample text for embedding"}
    local embeddings = RAG.embed(texts)
    
    local info = RAG.current_provider()
    if info.cost_per_token then
        print("OpenAI cost: $" .. (info.cost_per_token * 10))  -- Estimate for 10 tokens
    end
end

-- Switch to local for free embeddings
RAG.set_provider("local")
print("Switched to local embeddings (free)")
```

### 8.2 Feature Examples

These go in `examples/script-users/features/`:

```lua
-- examples/script-users/features/rag-chunking.lua
-- Advanced document chunking strategies
-- Shows different chunking methods for optimal retrieval

-- Configure custom chunking
RAG.configure({
    chunking = {
        method = "semantic",  -- Use semantic boundaries
        max_tokens = 512,
        overlap_tokens = 50
    }
})

-- Load and process a large document
local large_doc = {
    id = "whitepaper",
    text = File.read("docs/technical/master-architecture-vision.md"),
    metadata = {type = "documentation", version = "1.0"}
}

-- Ingest with semantic chunking
local stats = RAG.ingest({large_doc})
print("Document chunked into " .. stats.total_chunks .. " pieces")

-- Query specific sections
local results = RAG.search("How does the bridge layer work?", {k = 3})
for _, chunk in ipairs(results) do
    print("Found in chunk: " .. chunk.metadata.chunk_index)
    print(chunk.text:sub(1, 200) .. "...")
end
```

```lua
-- examples/script-users/features/rag-hybrid-search.lua
-- Hybrid retrieval combining vector, keyword, and metadata
-- Shows advanced search strategies

-- Configure hybrid retrieval
RAG.configure({
    retrieval = {
        hybrid_weights = {
            vector = 0.7,
            keyword = 0.2,
            metadata = 0.1
        },
        rerank_enabled = true
    }
})

-- Ingest documents with rich metadata
local docs = {
    {id = "1", text = "OpenAI GPT-4 is a large language model", 
     metadata = {category = "ai", date = "2024", provider = "openai"}},
    {id = "2", text = "Anthropic Claude is an AI assistant",
     metadata = {category = "ai", date = "2024", provider = "anthropic"}},
    {id = "3", text = "Local BERT models can run on CPU",
     metadata = {category = "ml", date = "2023", provider = "local"}}
}

RAG.ingest(docs)

-- Hybrid search with metadata filtering
local results = RAG.search("language models", {
    k = 5,
    filters = {category = "ai", date = "2024"}
})

print("Found " .. #results .. " matching documents")
```

### 8.3 Cookbook Patterns

Production patterns in `examples/script-users/cookbook/`:

```lua
-- examples/script-users/cookbook/rag-cost-optimization.lua
-- Cost-aware embedding strategies for production
-- Shows how to minimize embedding costs

local CostOptimizedRAG = {}

function CostOptimizedRAG:new()
    local obj = {}
    setmetatable(obj, {__index = self})
    
    -- Track cumulative costs
    obj.total_cost = 0
    obj.vectors_processed = 0
    
    return obj
end

function CostOptimizedRAG:smart_ingest(documents)
    -- Use cheapest provider for bulk ingestion
    local original_provider = RAG.current_provider().name
    
    -- Check if we have local provider available
    local providers = Provider.list()
    local has_local = false
    for _, p in ipairs(providers) do
        if p.name == "local" then
            has_local = true
            break
        end
    end
    
    if has_local then
        print("Using local embeddings for bulk ingestion (free)")
        RAG.set_provider("local")
    elseif providers["openai"] then
        -- Use smaller model for cost savings
        Provider.configure("openai", {
            embedding_model = "text-embedding-3-small",
            embedding_dimensions = 512  -- Reduce dimensions for cost
        })
        RAG.set_provider("openai")
    end
    
    -- Ingest with cost tracking
    local stats = RAG.ingest(documents, {use_cheapest = true})
    self.total_cost = self.total_cost + (stats.embedding_cost or 0)
    self.vectors_processed = self.vectors_processed + stats.vectors_stored
    
    -- Restore original provider
    RAG.set_provider(original_provider)
    
    return stats
end

function CostOptimizedRAG:get_stats()
    return {
        total_cost = self.total_cost,
        vectors_processed = self.vectors_processed,
        avg_cost_per_vector = self.total_cost / math.max(1, self.vectors_processed)
    }
end

-- Usage
local optimizer = CostOptimizedRAG:new()
local docs = generate_test_documents(100)  -- 100 test documents
optimizer:smart_ingest(docs)

local stats = optimizer:get_stats()
print(string.format("Processed %d vectors for $%.6f (avg: $%.8f/vector)",
    stats.vectors_processed, stats.total_cost, stats.avg_cost_per_vector))
```

```lua
-- examples/script-users/cookbook/rag-with-agents.lua
-- Combining RAG with agents for enhanced context
-- Production pattern for agent memory

local RAGEnhancedAgent = {}

function RAGEnhancedAgent:new(agent_config)
    local obj = {
        agent = Agent.builder()
            :name(agent_config.name)
            :provider(agent_config.provider)
            :model(agent_config.model)
            :build(),
        conversation_history = {},
        max_history = 10
    }
    setmetatable(obj, {__index = self})
    return obj
end

function RAGEnhancedAgent:chat(user_input)
    -- Store user input in conversation history
    table.insert(self.conversation_history, {
        role = "user",
        content = user_input,
        timestamp = os.time()
    })
    
    -- Retrieve relevant context from RAG
    local context_results = RAG.search(user_input, {k = 3})
    
    -- Build enhanced prompt with RAG context
    local enhanced_prompt = self:build_contextual_prompt(user_input, context_results)
    
    -- Execute with agent
    local response = self.agent:complete({
        prompt = enhanced_prompt,
        max_tokens = 500
    })
    
    -- Store agent response
    table.insert(self.conversation_history, {
        role = "assistant",
        content = response.text,
        timestamp = os.time()
    })
    
    -- Asynchronously update RAG with new knowledge
    Hook.after("agent_response", function()
        RAG.ingest({{
            id = "conv_" .. os.time(),
            text = user_input .. "\n" .. response.text,
            metadata = {
                type = "conversation",
                agent = self.agent.name,
                timestamp = os.time()
            }
        }})
    end)
    
    -- Trim history if needed
    if #self.conversation_history > self.max_history * 2 then
        -- Keep only recent history
        local new_history = {}
        for i = #self.conversation_history - self.max_history + 1, #self.conversation_history do
            table.insert(new_history, self.conversation_history[i])
        end
        self.conversation_history = new_history
    end
    
    return response
end

function RAGEnhancedAgent:build_contextual_prompt(user_input, context_results)
    local prompt = "Context from knowledge base:\n"
    
    for _, result in ipairs(context_results) do
        if result.score > 0.7 then  -- Only include highly relevant context
            prompt = prompt .. "- " .. result.text .. "\n"
        end
    end
    
    prompt = prompt .. "\nConversation history:\n"
    
    -- Add recent conversation context
    local start_idx = math.max(1, #self.conversation_history - 4)
    for i = start_idx, #self.conversation_history do
        local entry = self.conversation_history[i]
        prompt = prompt .. entry.role .. ": " .. entry.content .. "\n"
    end
    
    prompt = prompt .. "\nUser: " .. user_input .. "\nAssistant:"
    
    return prompt
end

-- Usage example
local agent = RAGEnhancedAgent:new({
    name = "rag_assistant",
    provider = "openai",
    model = "gpt-4"
})

-- First interaction - no context yet
local response1 = agent:chat("What is HNSW?")
print("Agent: " .. response1.text)

-- Second interaction - uses previous context
local response2 = agent:chat("How does it compare to other indexing methods?")
print("Agent: " .. response2.text)
```

### 8.4 Rust Developer Examples

For Rust developers in `examples/rust-developers/rag-integration/`:

```rust
// examples/rust-developers/rag-integration/src/main.rs
// Embedding RAG capabilities in Rust applications

use llmspell_rag::prelude::*;
use llmspell_providers::{ProviderManager, ProviderConfig};
use llmspell_bridge::RAGBridge;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize provider manager
    let mut provider_manager = ProviderManager::new();
    
    // Configure OpenAI provider with embeddings
    let openai_config = ProviderConfig::new("openai", "gpt-4")
        .with_api_key(std::env::var("OPENAI_API_KEY")?)
        .with_custom("embedding_model", json!("text-embedding-3-small"))
        .with_custom("embedding_dimensions", json!(1536));
    
    provider_manager.register_provider("openai", openai_config)?;
    
    // Create RAG bridge
    let rag_bridge = RAGBridge::new(Arc::new(provider_manager)).await?;
    
    // Ingest documents
    let documents = vec![
        Document {
            id: "doc1".to_string(),
            text: "Rust provides memory safety without garbage collection".to_string(),
            metadata: Default::default(),
        },
        Document {
            id: "doc2".to_string(),
            text: "The ownership system prevents data races at compile time".to_string(),
            metadata: Default::default(),
        },
    ];
    
    let stats = rag_bridge.ingest(documents, false).await?;
    println!("Ingested {} vectors", stats.vectors_stored);
    
    // Perform semantic search
    let results = rag_bridge.search("How does Rust ensure memory safety?", 5, None).await?;
    
    for (i, chunk) in results.iter().enumerate() {
        println!("{}. {} (score: {})", i + 1, chunk.text, chunk.score);
    }
    
    Ok(())
}
```

### 8.5 Advanced Application Example

A complete application in `examples/script-users/applications/rag-assistant/`:

```lua
-- examples/script-users/applications/rag-assistant/main.lua
-- Production-ready RAG-powered assistant application
-- Demonstrates state persistence, multi-provider support, and monitoring

local RAGAssistant = require("modules.assistant")
local config = require("config")

-- Initialize application
local app = RAGAssistant:new(config)

-- Set up monitoring
Hook.register("before_embedding", function(ctx)
    Metrics.increment("embeddings.requests")
    Metrics.histogram("embeddings.batch_size", #ctx.data.texts)
end)

Hook.register("after_search", function(ctx)
    Metrics.histogram("search.latency_ms", ctx.duration_ms)
    Metrics.histogram("search.results_count", #ctx.data.results)
end)

-- Main interaction loop
function main()
    print("RAG Assistant initialized. Type 'help' for commands.")
    
    while true do
        io.write("> ")
        local input = io.read()
        
        if input == "exit" then
            break
        elseif input == "help" then
            app:show_help()
        elseif input:match("^/ingest ") then
            local path = input:match("^/ingest (.+)")
            app:ingest_document(path)
        elseif input:match("^/provider ") then
            local provider = input:match("^/provider (.+)")
            app:switch_provider(provider)
        elseif input:match("^/stats") then
            app:show_statistics()
        else
            -- Process as query
            local response = app:query(input)
            print("\n" .. response .. "\n")
        end
    end
    
    -- Save state before exit
    app:save_state()
    print("Goodbye!")
end

-- Run with error handling
local ok, err = pcall(main)
if not ok then
    print("Error: " .. tostring(err))
    os.exit(1)
end
```

### 8.4 Multi-Tenant Examples

Advanced multi-tenant patterns in `examples/script-users/cookbook/`:

```lua
-- examples/script-users/cookbook/rag-multi-tenant.lua
-- Multi-tenant RAG with isolation and cost tracking
-- Shows production multi-tenant patterns

-- Initialize multi-tenant RAG
local MultiTenantRAG = {}

function MultiTenantRAG:new()
    local obj = {
        tenants = {},
        usage = {}
    }
    setmetatable(obj, {__index = self})
    return obj
end

-- Create isolated tenant namespace
function MultiTenantRAG:create_tenant(tenant_id, limits)
    -- Create tenant scope
    local scope = {type = "user", id = tenant_id}
    
    -- Initialize tenant with limits
    RAG.create_tenant({
        tenant_id = tenant_id,
        scope = scope,
        limits = limits or {
            max_vectors = 10000,
            max_queries_per_minute = 60,
            monthly_spending_limit = 50.0
        }
    })
    
    self.tenants[tenant_id] = {
        scope = scope,
        created_at = os.time()
    }
    
    print("Created tenant: " .. tenant_id)
end

-- Tenant-scoped search
function MultiTenantRAG:search(tenant_id, query, k)
    if not self.tenants[tenant_id] then
        error("Unknown tenant: " .. tenant_id)
    end
    
    -- Search within tenant scope
    local results = RAG.search(query, {
        k = k or 10,
        scope = self.tenants[tenant_id].scope
    })
    
    -- Track usage
    self:track_usage(tenant_id, "search", 1)
    
    return results
end

-- Tenant-scoped ingestion
function MultiTenantRAG:ingest(tenant_id, documents)
    if not self.tenants[tenant_id] then
        error("Unknown tenant: " .. tenant_id)
    end
    
    -- Check limits before ingestion
    local usage = RAG.get_tenant_usage(tenant_id)
    if usage.vectors_stored + #documents > usage.limits.max_vectors then
        error("Tenant vector limit exceeded")
    end
    
    -- Ingest with tenant scope
    local stats = RAG.ingest(documents, {
        scope = self.tenants[tenant_id].scope
    })
    
    -- Track usage and costs
    self:track_usage(tenant_id, "ingest", stats.tokens_consumed)
    
    return stats
end

-- Track and report usage
function MultiTenantRAG:track_usage(tenant_id, operation, amount)
    if not self.usage[tenant_id] then
        self.usage[tenant_id] = {
            searches = 0,
            tokens = 0,
            cost = 0.0
        }
    end
    
    if operation == "search" then
        self.usage[tenant_id].searches = self.usage[tenant_id].searches + amount
    elseif operation == "ingest" then
        self.usage[tenant_id].tokens = self.usage[tenant_id].tokens + amount
        -- Calculate cost (example: $0.0001 per 1K tokens)
        self.usage[tenant_id].cost = self.usage[tenant_id].cost + (amount / 1000 * 0.0001)
    end
end

-- Get tenant usage report
function MultiTenantRAG:get_usage_report(tenant_id)
    local usage = self.usage[tenant_id] or {searches = 0, tokens = 0, cost = 0.0}
    local rag_usage = RAG.get_tenant_usage(tenant_id)
    
    return {
        tenant_id = tenant_id,
        searches_performed = usage.searches,
        tokens_consumed = usage.tokens,
        estimated_cost = usage.cost,
        vectors_stored = rag_usage.vectors_stored,
        storage_bytes = rag_usage.storage_bytes
    }
end

-- Example usage
local mt_rag = MultiTenantRAG:new()

-- Create tenants
mt_rag:create_tenant("acme_corp", {
    max_vectors = 50000,
    max_queries_per_minute = 100,
    monthly_spending_limit = 200.0
})

mt_rag:create_tenant("startup_inc", {
    max_vectors = 5000,
    max_queries_per_minute = 20,
    monthly_spending_limit = 10.0
})

-- Tenant-specific operations
mt_rag:ingest("acme_corp", {
    {id = "doc1", text = "Enterprise knowledge base content"},
    {id = "doc2", text = "Company policies and procedures"}
})

local results = mt_rag:search("acme_corp", "company policies", 5)
print("Found " .. #results .. " results for ACME Corp")

-- Get usage report
local report = mt_rag:get_usage_report("acme_corp")
print(string.format("ACME Corp usage: %d searches, %d tokens, $%.4f cost",
    report.searches_performed, report.tokens_consumed, report.estimated_cost))
```

```lua
-- examples/script-users/cookbook/rag-session-isolation.lua
-- Session-scoped RAG for ephemeral data
-- Shows how to isolate vectors per session

-- Create session-scoped RAG
local session_id = Session.current().id
local session_scope = {type = "session", id = session_id}

-- Ingest session-specific documents
RAG.ingest({
    {id = "chat1", text = "User asked about product features"},
    {id = "chat2", text = "Assistant explained pricing tiers"}
}, {scope = session_scope})

-- Search within session context only
local results = RAG.search("pricing information", {
    k = 3,
    scope = session_scope
})

-- Session vectors auto-expire after TTL
RAG.configure_session({
    session_id = session_id,
    vector_ttl = 3600  -- 1 hour
})

-- Clean up session vectors on logout
Session.on_end(function()
    RAG.cleanup_scope(session_scope)
    print("Session vectors cleaned up")
end)
```

---

## 9. Testing Strategy

### 9.1 Unit Tests

```rust
// llmspell-rag/tests/unit/embeddings_test.rs
#[tokio::test]
async fn test_bge_m3_embedding_generation() {
    let embedder = BGEM3Embedder::from_huggingface("BAAI/bge-m3")
        .await
        .unwrap();
    
    let texts = vec![
        "Hello, world!".to_string(),
        "Vector embeddings are useful.".to_string(),
    ];
    
    let embeddings = embedder.embed(&texts).await.unwrap();
    
    assert_eq!(embeddings.len(), 2);
    assert_eq!(embeddings[0].len(), 1024);  // BGE-M3 dimension
    
    // Verify embeddings are normalized
    for embedding in &embeddings {
        let norm: f32 = embedding.iter().map(|x| x * x).sum::<f32>().sqrt();
        assert!((norm - 1.0).abs() < 0.01);
    }
}

#[tokio::test]
async fn test_hnsw_index_performance() {
    let storage = EmbeddedHNSW::new(HNSWConfig {
        m: 32,
        ef_construction: 200,
        ef_search: 100,
        max_elements: 10000,
    }).unwrap();
    
    // Insert 10000 random vectors
    let vectors = generate_random_vectors(10000, 1024);
    let start = Instant::now();
    storage.insert(vectors).await.unwrap();
    let insert_time = start.elapsed();
    
    assert!(insert_time < Duration::from_secs(10));  // Should be fast
    
    // Search performance
    let query = generate_random_vector(1024);
    let start = Instant::now();
    let results = storage.search(&VectorQuery {
        vector: query,
        k: 10,
        filter: None,
    }).await.unwrap();
    let search_time = start.elapsed();
    
    assert!(search_time < Duration::from_millis(10));  // <10ms requirement
    assert_eq!(results.len(), 10);
}
```

### 8.2 Integration Tests

```rust
// llmspell-rag/tests/integration/rag_pipeline_test.rs
#[tokio::test]
async fn test_end_to_end_rag_pipeline() {
    // Setup pipeline
    let pipeline = RAGPipeline::builder()
        .with_embedder(Box::new(BGEM3Embedder::from_huggingface("BAAI/bge-m3").await?))
        .with_storage(Box::new(MemVDBStorage::new(HNSWConfig::default())?))
        .with_chunker(DocumentChunker::new(ChunkingStrategy::default()))
        .build()?;
    
    // Ingest documents
    let documents = vec![
        Document::from_text("doc1", "Rust is a systems programming language."),
        Document::from_text("doc2", "Vector databases enable semantic search."),
        Document::from_text("doc3", "HNSW is an efficient indexing algorithm."),
    ];
    
    let stats = pipeline.ingest(documents).await?;
    assert_eq!(stats.vectors_stored, 3);
    
    // Retrieve relevant documents
    let results = pipeline.retrieve("What is HNSW?", 2).await?;
    assert_eq!(results.len(), 2);
    assert!(results[0].text.contains("HNSW"));
}

#[tokio::test]
async fn test_agent_vector_integration() {
    let base_agent = MockAgent::new();
    let pipeline = create_test_pipeline().await?;
    
    let vector_agent = VectorEnhancedAgent::new(
        Box::new(base_agent),
        Arc::new(pipeline),
    );
    
    // First interaction - should store in memory
    let input1 = AgentInput::text("Tell me about Rust");
    let output1 = vector_agent.execute(input1, ExecutionContext::new()).await?;
    
    // Second interaction - should retrieve context
    let input2 = AgentInput::text("What programming language did we discuss?");
    let output2 = vector_agent.execute(input2, ExecutionContext::new()).await?;
    
    // Verify context was retrieved
    assert!(output2.metadata.contains_key("retrieved_context"));
}
```

### 8.3 Bridge Tests

```rust
// llmspell-bridge/tests/lua_rag_test.rs
#[tokio::test]
async fn test_lua_rag_search() {
    let runtime = create_test_runtime_with_rag().await?;
    
    let lua_code = r#"
        -- Ingest some documents
        local docs = {
            {id = "1", text = "Lua is a scripting language"},
            {id = "2", text = "Rust is a systems language"},
        }
        RAG.ingest(docs)
        
        -- Search for relevant documents
        local results = RAG.search("What is Lua?", {k = 1})
        assert(#results == 1)
        assert(results[1].id == "1")
        return results[1].score
    "#;
    
    let score: f32 = runtime.execute_lua(lua_code).await?;
    assert!(score > 0.5);  // Should have high relevance
}
```

### 9.2 Multi-Tenant Testing

Comprehensive tests for multi-tenant isolation and security:

```rust
// llmspell-rag/tests/integration/multi_tenant_test.rs
use llmspell_rag::multi_tenant::*;
use llmspell_state_traits::StateScope;

#[tokio::test]
async fn test_tenant_isolation() {
    let manager = MultiTenantVectorManager::new(config).await?;
    
    // Create two tenants
    manager.create_tenant_namespace("tenant_a", TenantConfig::default()).await?;
    manager.create_tenant_namespace("tenant_b", TenantConfig::default()).await?;
    
    // Ingest data for tenant A
    let docs_a = vec![
        Document::new("a1", "Confidential data for tenant A"),
        Document::new("a2", "Private information for tenant A"),
    ];
    manager.ingest_for_tenant("tenant_a", docs_a).await?;
    
    // Ingest data for tenant B
    let docs_b = vec![
        Document::new("b1", "Secret data for tenant B"),
        Document::new("b2", "Sensitive information for tenant B"),
    ];
    manager.ingest_for_tenant("tenant_b", docs_b).await?;
    
    // Search as tenant A - should only see A's data
    let results_a = manager.search_for_tenant("tenant_a", "confidential", 10).await?;
    assert!(results_a.iter().all(|r| r.id.starts_with("a")));
    assert_eq!(results_a.len(), 1);
    
    // Search as tenant B - should only see B's data
    let results_b = manager.search_for_tenant("tenant_b", "secret", 10).await?;
    assert!(results_b.iter().all(|r| r.id.starts_with("b")));
    assert_eq!(results_b.len(), 1);
    
    // Cross-tenant search should fail
    let cross_search = manager
        .search_with_wrong_tenant("tenant_a", StateScope::User("tenant_b".to_string()))
        .await;
    assert!(cross_search.is_err());
}

#[tokio::test]
async fn test_tenant_usage_limits() {
    let manager = MultiTenantVectorManager::new(config).await?;
    
    // Create tenant with strict limits
    let limited_config = TenantConfig {
        limits: TenantLimits {
            max_vectors: 10,
            max_queries_per_minute: 5,
            monthly_spending_limit: 1.0,
            ..Default::default()
        },
        ..Default::default()
    };
    
    manager.create_tenant_namespace("limited_tenant", limited_config).await?;
    
    // Try to exceed vector limit
    let many_docs: Vec<_> = (0..20).map(|i| {
        Document::new(&format!("doc{}", i), &format!("Document {}", i))
    }).collect();
    
    let result = manager.ingest_for_tenant("limited_tenant", many_docs).await;
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("limit exceeded"));
    
    // Test query rate limiting
    for i in 0..10 {
        let result = manager.search_for_tenant("limited_tenant", "test", 1).await;
        if i < 5 {
            assert!(result.is_ok());
        } else {
            assert!(result.is_err());
            assert!(result.unwrap_err().to_string().contains("rate limit"));
        }
    }
}

#[tokio::test]
async fn test_session_vector_expiration() {
    let session_manager = SessionManager::new(config).await?;
    let rag = SessionAwareRAGPipeline::new(pipeline, session_manager.clone()).await?;
    
    // Create session with short TTL
    let session_id = session_manager.create_session(Default::default()).await?;
    let collection = rag.create_session_collection(session_id, Some(1)).await?; // 1 second TTL
    
    // Ingest documents
    let docs = vec![Document::new("1", "Temporary session data")];
    rag.ingest_in_session(docs, session_id).await?;
    
    // Immediate search should work
    let results = rag.retrieve_in_session("session data", session_id, 10).await?;
    assert_eq!(results.len(), 1);
    
    // Wait for TTL to expire
    tokio::time::sleep(Duration::from_secs(2)).await;
    
    // Search should return empty after expiration
    let expired_results = rag.retrieve_in_session("session data", session_id, 10).await?;
    assert_eq!(expired_results.len(), 0);
}

#[tokio::test]
async fn test_rls_policy_enforcement() {
    let security_manager = VectorSecurityManager::new(config).await?;
    
    // Create policy with metadata filters
    let policy = VectorAccessPolicy {
        tenant_id: "secure_tenant".to_string(),
        allowed_operations: vec![VectorOperation::Search, VectorOperation::Insert],
        metadata_filters: hashmap! {
            "department".to_string() => json!("engineering"),
            "classification".to_string() => json!("internal")
        },
        dimension_limit: Some(1024),
        query_limit_per_minute: Some(100),
    };
    
    security_manager.add_policy(policy).await?;
    
    // Test query with RLS filters applied
    let mut query = VectorQuery {
        text: "search query".to_string(),
        scope: Some(StateScope::User("secure_tenant".to_string())),
        ..Default::default()
    };
    
    security_manager.apply_rls_filters(&mut query, &StateScope::User("secure_tenant".to_string()))?;
    
    // Verify filters were applied
    assert_eq!(query.metadata_filters.get("department"), Some(&json!("engineering")));
    assert_eq!(query.metadata_filters.get("classification"), Some(&json!("internal")));
    
    // Test denied operation
    let delete_result = security_manager.validate_operation(
        &VectorOperation::Delete,
        &StateScope::User("secure_tenant".to_string()),
        None
    ).await;
    assert!(delete_result.is_err());
}
```

---

## 10. Performance Targets

### 10.1 Benchmarks

```rust
// llmspell-rag/benches/vector_bench.rs
use criterion::{criterion_group, criterion_main, Criterion};

fn benchmark_embedding_generation(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();
    let embedder = rt.block_on(BGEM3Embedder::from_huggingface("BAAI/bge-m3")).unwrap();
    
    c.bench_function("bge_m3_embedding_single", |b| {
        b.to_async(&rt).iter(|| async {
            embedder.embed(&["Sample text".to_string()]).await
        });
    });
    
    c.bench_function("bge_m3_embedding_batch", |b| {
        let texts: Vec<String> = (0..32).map(|i| format!("Sample text {}", i)).collect();
        b.to_async(&rt).iter(|| async {
            embedder.embed(&texts).await
        });
    });
}

fn benchmark_vector_search(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();
    let storage = rt.block_on(setup_storage_with_vectors(100000)).unwrap();
    
    c.bench_function("hnsw_search_100k_vectors", |b| {
        let query = generate_random_vector(1024);
        b.to_async(&rt).iter(|| async {
            storage.search(&VectorQuery {
                vector: query.clone(),
                k: 10,
                filter: None,
            }).await
        });
    });
}

criterion_group!(benches, benchmark_embedding_generation, benchmark_vector_search);
criterion_main!(benches);
```

### 10.2 Performance Requirements

| Operation | Target | Maximum |
|-----------|--------|---------|
| Single embedding generation | <50ms | 100ms |
| Batch embedding (32 texts) | <500ms | 1s |
| Vector insertion (1000) | <100ms | 200ms |
| Vector search (1M vectors) | <10ms | 20ms |
| Document chunking (10KB) | <5ms | 10ms |
| RAG pipeline (end-to-end) | <200ms | 500ms |
| Memory overhead per vector | <2KB | 4KB |

### 10.3 Multi-Tenant Performance Targets

Specific performance requirements for multi-tenant operations:

| Operation | Target | Maximum | Notes |
|-----------|--------|---------|-------|
| Tenant namespace creation | <100ms | 500ms | One-time setup |
| Tenant-scoped search (10K vectors) | <5ms | 10ms | Namespace isolation benefit |
| Tenant-scoped search (100K vectors) | <8ms | 15ms | Still faster than global search |
| Cross-tenant isolation check | <1ms | 2ms | Security validation overhead |
| RLS filter application | <0.5ms | 1ms | Query-time filtering |
| Scope resolution | <0.1ms | 0.5ms | StateScope to namespace mapping |
| Usage tracking update | <2ms | 5ms | Async state updates |
| Session vector expiration | <10ms | 20ms | TTL cleanup per session |
| Tenant usage calculation | <5ms | 10ms | Aggregation across metrics |
| Cost calculation | <1ms | 2ms | Per-operation pricing |

### 10.4 Scalability Targets

Multi-tenant system scalability requirements:

- **Tenant Capacity**: Support 10,000+ tenants per instance
- **Vectors per Tenant**: 100K average, 1M maximum
- **Concurrent Tenants**: 1,000+ active tenants simultaneously
- **Isolation Overhead**: <5% performance penalty for namespace isolation
- **Memory per Tenant**: Base 10MB + 2KB per vector
- **Query Throughput**: 10,000 QPS across all tenants
- **Ingestion Rate**: 1,000 vectors/second per tenant
- **State Overhead**: <1MB per tenant for usage tracking
- **Session Vectors**: Support 10,000 concurrent sessions
- **Cleanup Performance**: Process 100K expired vectors/minute

### 10.5 Optimization Strategies

Performance optimizations for multi-tenant scenarios:

```rust
// Tenant-specific optimizations
impl MultiTenantVectorManager {
    /// Use exact search for small tenants (better accuracy)
    async fn optimize_search_strategy(&self, tenant_id: &str) -> SearchStrategy {
        let stats = self.get_tenant_stats(tenant_id).await?;
        
        if stats.vector_count < 1000 {
            SearchStrategy::ExactSearch  // Perfect recall for small datasets
        } else if stats.vector_count < 10000 {
            SearchStrategy::HNSWSmall { ef: 50 }  // Balanced
        } else {
            SearchStrategy::HNSWLarge { ef: 100 }  // Optimized for scale
        }
    }
    
    /// Cache embeddings per tenant for frequently accessed data
    async fn cache_strategy(&self, tenant_id: &str) -> CacheConfig {
        let usage = self.get_tenant_usage(tenant_id).await?;
        
        CacheConfig {
            // High-usage tenants get larger caches
            max_entries: match usage.queries_per_day {
                0..=100 => 100,
                101..=1000 => 500,
                _ => 2000,
            },
            ttl: Duration::from_secs(3600),
            strategy: if usage.is_premium {
                CacheStrategy::LRU
            } else {
                CacheStrategy::FIFO
            }
        }
    }
}
```

---

## 11. Migration and Rollout Plan

### 11.1 Phase 8 Week 1: Core Infrastructure

- [ ] Create `llmspell-rag` crate structure
- [ ] Implement vector storage traits
- [ ] Implement HNSW embedded storage using hnswlib-rs
- [ ] Implement BGE-M3 embedder using candle
- [ ] Basic unit tests for storage and embedding

### 11.2 Phase 8 Week 2: Integration and Bridge

- [ ] Implement RAG pipeline with chunking and retrieval
- [ ] Integrate ColBERT v2 for late interaction
- [ ] Add hook and event support for all operations
- [ ] Expose RAG API via Lua bridge
- [ ] Create semantic search and code search tools
- [ ] Integration tests and performance benchmarks
- [ ] Documentation and examples

### 11.3 CLI Integration (No Changes Required)

The `llmspell-cli` crate **does not require any modifications** for Phase 8 because:

1. **Configuration Delegation**: CLI already delegates all configuration loading to `llmspell-config`, which now includes RAG settings
2. **Runtime Creation**: CLI creates `ScriptRuntime` instances via the bridge layer, which will automatically expose RAG globals
3. **Validation**: The `validate` command already validates the entire config including new RAG sections
4. **Initialization**: The `init` command will automatically include RAG config sections in generated configs

### 11.4 Embedded Example Applications

While the CLI crate doesn't need code changes, we should **enhance embedded example applications** to showcase RAG capabilities:

#### Applications to Enhance:
1. **research-collector** - Add vector-based research memory and semantic search
2. **code-review-assistant** - Add code embedding and similarity search
3. **content-creator** - Add content library with vector search
4. **file-organizer** - Add semantic file similarity detection

#### New Example Applications to Create:
```lua
-- llmspell-cli/resources/applications/knowledge-base/main.lua
-- Knowledge Base Manager with RAG
-- Showcases: Multi-tenant RAG, session isolation, semantic search

local kb = KnowledgeBase:new()

-- Initialize tenant-specific knowledge base
kb:init_tenant("my_company", {
    max_documents = 10000,
    embedding_provider = "openai"
})

-- Ingest documents with automatic chunking
kb:ingest_documents({
    {path = "docs/*.md", chunking = "semantic"},
    {path = "code/*.rs", chunking = "structural"}
})

-- Semantic search with session context
local session = kb:create_session()
local results = session:search("How does authentication work?", {
    k = 5,
    filters = {category = "security"}
})

-- Q&A with RAG
local answer = session:ask("What are the security best practices?")
print(answer)
```

```lua
-- llmspell-cli/resources/applications/personal-assistant/main.lua
-- Personal Assistant with Memory
-- Showcases: Session-based memory, cost optimization, hybrid search

local assistant = PersonalAssistant:new()

-- Use session-scoped vectors for conversation memory
assistant:start_session({
    vector_ttl = 3600,  -- 1 hour memory
    provider = "local"  -- Use free local embeddings
})

-- Conversation with context accumulation
assistant:chat("I'm planning a trip to Japan")
assistant:chat("I prefer cultural experiences over tourist spots")

-- Assistant remembers context across conversation
local recommendation = assistant:chat("What cities should I visit?")
-- Response uses accumulated session context

-- Save important memories to long-term storage
assistant:save_memories({importance = "high"})
```

#### Example Configurations to Add:
```toml
# llmspell-cli/resources/applications/knowledge-base/config.toml
[rag]
enabled = true
storage_backend = "embedded"

[rag.multi_tenancy]
enabled = true
isolation_strategy = "namespace_per_tenant"

[rag.session_integration]
bind_vectors_to_sessions = true
session_vector_ttl = 3600
```

---

## 12. Provider Architecture Integration Summary

### 12.1 Leveraging Existing Infrastructure

This design properly leverages rs-llmspell's existing provider architecture:

1. **Extended `llmspell-providers` crate**: 
   - Added `EmbeddingProvider` trait extending `ProviderInstance`
   - Extended `RigProvider` to support both completions and embeddings
   - Prepared for future candle integration via `LocalEmbeddingModel` trait

2. **Unified Configuration**:
   - Embeddings configured within existing provider blocks
   - No parallel configuration system
   - Single `ProviderManager` for both LLMs and embeddings

3. **Technology Stack**:
   - **rig-core**: Handles API-based providers (OpenAI, Cohere)
   - **candle-core/transformers** (future): Local model execution
   - **tokenizers**: Shared tokenization infrastructure

### 12.2 Key Architectural Decisions

1. **No Parallel System**: RAG uses existing `ProviderManager` instead of creating `EmbeddingFactory`
2. **Dynamic Dimensions**: Storage layer adapts to provider-specific dimensions (256-4096)
3. **Cost-Aware Routing**: Provider selection considers embedding costs
4. **Provider Delegation**: Handles cases like Anthropic → Voyage AI transparently

### 12.3 Migration Path

```rust
// Before: Would have required separate embedding configuration
let embedder = EmbeddingFactory::create("openai", "text-embedding-3-small")?;

// After: Uses existing provider system
let provider = provider_manager.get_provider("openai")?;
let embeddings = provider.embed(&texts).await?;
```

### 12.4 Why Provider Integration Matters

The multi-provider embedding landscape fundamentally affects our design:

1. **No Universal Dimensions**: Cannot assume 1024 dims - must support 256-4096
2. **Cost vs Performance**: OpenAI text-embedding-3-small costs $0.00002/1K tokens vs free local
3. **Provider Gaps**: Anthropic doesn't offer embeddings, requiring fallback strategies
4. **Matryoshka Benefits**: OpenAI's dimension reduction allows 6x smaller vectors with minimal loss

### 12.5 Performance Impact

The dynamic dimension handling adds minimal overhead:
- **Storage**: ~5% overhead for dimension mapping
- **Retrieval**: <1ms to route to correct index
- **Memory**: Negligible (HashMap of indices)

The benefits far outweigh costs:
- **6x reduction** in storage with Matryoshka
- **10x cost savings** using local vs API embeddings when appropriate
- **Seamless provider switching** for A/B testing

---

## 13. Future Considerations (Phase 9 Preparation)

This vector storage foundation with multi-tenant support prepares for Phase 9's Adaptive Memory System by:

### 13.1 Foundation for Adaptive Memory

1. **Temporal Metadata**: All vectors store timestamps for bi-temporal modeling with tenant context
2. **Flexible Schema**: Metadata supports arbitrary fields for entity/relationship storage per tenant
3. **Event Integration**: Full event emission enables memory consolidation triggers with tenant isolation
4. **Hybrid Retrieval**: Combined vector/keyword/metadata search for graph traversal within tenant boundaries
5. **Scalable Architecture**: Storage traits allow swapping backends as tenant needs grow
6. **Hook System**: Memory management hooks can observe and optimize storage per tenant

### 13.2 Multi-Tenant Memory Architecture

The multi-tenant infrastructure established in Phase 8 will enable Phase 9's memory system to:

1. **Tenant-Specific Memory Graphs**: Each tenant maintains their own Adaptive Temporal Knowledge Graph
2. **Isolated Learning**: Agents learn from tenant-specific interactions without cross-contamination
3. **Personalized Context**: Memory consolidation happens within tenant boundaries
4. **Cost-Aware Memory Management**: Different memory retention policies based on tenant tier
5. **Session-to-Long-Term Migration**: Session vectors can be promoted to tenant's long-term memory
6. **Privacy-Preserving Learning**: Tenant data never influences other tenants' AI behavior

### 13.3 Migration Path from Phase 8 to Phase 9

```rust
// Phase 9 will extend Phase 8's multi-tenant foundation
pub struct TenantMemorySystem {
    // Phase 8 foundation
    vector_storage: Arc<MultiTenantVectorManager>,
    state_manager: Arc<StateManager>,
    session_manager: Arc<SessionManager>,
    
    // Phase 9 additions
    working_memory: Arc<WorkingMemoryLayer>,      // Per-tenant working memory
    episodic_memory: Arc<EpisodicMemoryLayer>,    // Session-based experiences
    semantic_memory: Arc<SemanticMemoryLayer>,    // Long-term knowledge graph
    memory_consolidator: Arc<MemoryConsolidator>, // Tenant-specific consolidation
}

impl TenantMemorySystem {
    /// Migrate session vectors to long-term memory
    pub async fn consolidate_session_memory(
        &self,
        tenant_id: &str,
        session_id: SessionId,
    ) -> Result<()> {
        // Get session vectors from Phase 8 infrastructure
        let session_scope = StateScope::Session(session_id.to_string());
        let session_vectors = self.vector_storage
            .get_vectors_for_scope(&session_scope)
            .await?;
        
        // Analyze importance and consolidate to tenant's long-term memory
        let tenant_scope = StateScope::User(tenant_id.to_string());
        let important_memories = self.memory_consolidator
            .extract_important_memories(&session_vectors)
            .await?;
        
        // Store in semantic memory graph
        self.semantic_memory
            .add_memories(tenant_scope, important_memories)
            .await?;
        
        Ok(())
    }
}
```

### 13.4 Implications for Phase 9 Design

The multi-tenant architecture from Phase 8 requires Phase 9 to consider:

1. **Memory Quota Management**: Enforce memory limits per tenant tier
2. **Cross-Tenant Knowledge Sharing**: Optional shared knowledge bases with access control
3. **Federated Learning**: Learn from aggregate patterns without exposing individual tenant data
4. **Memory Compression**: Tenant-specific compression strategies based on usage patterns
5. **Compliance and Data Residency**: Ensure memory storage complies with tenant's regulatory requirements

The RAG pipeline established here will serve as the "Episodic Memory" layer in Phase 9's Adaptive Temporal Knowledge Graph, while the multi-tenant infrastructure ensures complete isolation and personalization of each tenant's AI memory system.

---

## Appendix A: Crate Dependencies

```toml
# llmspell-rag/Cargo.toml
[package]
name = "llmspell-rag"
version = "0.8.0"

[dependencies]
# Core
llmspell-core = { path = "../llmspell-core" }
llmspell-utils = { path = "../llmspell-utils" }
llmspell-storage = { path = "../llmspell-storage" }
llmspell-hooks = { path = "../llmspell-hooks" }
llmspell-events = { path = "../llmspell-events" }
llmspell-providers = { path = "../llmspell-providers" }  # LEVERAGE EXISTING PROVIDERS

# Vector databases
memvdb = "0.1"
hnswlib-rs = "0.2"
qdrant-client = { version = "1.7", optional = true }

# NOTE: Embeddings handled via llmspell-providers which uses:
# - rig-core for API providers (OpenAI, Cohere, etc)
# - candle-core/transformers for local models (future)
# - tokenizers for text processing

# Utilities
tokio = { version = "1.35", features = ["full"] }
async-trait = "0.1"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
anyhow = "1.0"
tracing = "0.1"
chrono = "0.4"

[dev-dependencies]
criterion = { version = "0.5", features = ["async_tokio"] }
tempfile = "3.8"
```

---

## Appendix B: Example Usage

```lua
-- Lua script using RAG capabilities with provider integration
-- examples/rag_demo.lua

-- RAG automatically uses configured providers for embeddings
-- No separate embedding configuration needed!

-- Providers already configured in llmspell.toml:
-- [providers.openai]
-- provider_type = "openai"
-- api_key_env = "OPENAI_API_KEY"
-- [providers.openai.options]
-- embedding_model = "text-embedding-3-small"

-- RAG uses the provider system
RAG.configure({
    storage_backend = "embedded",
    -- Uses default provider's embeddings automatically
    embedding_provider = "default",  -- or "openai", "local", etc.
    retrieval = {
        k = 5,
        rerank_enabled = true
    }
})

-- Switch to local embeddings for bulk ingestion (cost savings)
RAG.set_provider("local")  -- Uses configured local provider

-- Ingest documents
local documents = {
    {id = "doc1", text = "LLMSpell is a framework for AI agents"},
    {id = "doc2", text = "Vector databases enable semantic search"},
    {id = "doc3", text = "HNSW provides efficient nearest neighbor search"}
}

local stats = RAG.ingest(documents)
print("Ingested " .. stats.vectors_stored .. " vectors")
print("Embedding cost: $" .. stats.embedding_cost)  -- Tracks costs!

-- Switch back to OpenAI for queries (better quality)
RAG.set_provider("openai")

-- Create a vector-enhanced agent
local agent = Agent.builder()
    :name("rag_agent")
    :type("llm")
    :with_vector_memory(true)
    :build()

-- Agent will automatically use vector memory for context
local response = agent:execute({
    prompt = "What is LLMSpell?",
    use_rag = true
})

print("Response: " .. response.text)
print("Retrieved context: " .. #response.metadata.retrieved_chunks .. " chunks")

-- Direct semantic search
local results = RAG.search("vector databases", {k = 2})
for i, result in ipairs(results) do
    print(i .. ": " .. result.text .. " (score: " .. result.score .. ")")
end
```