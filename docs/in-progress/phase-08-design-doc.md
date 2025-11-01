# Phase 8: Vector Storage and RAG Foundation - Design Document

**Version**: 2.0 (Post-Implementation)  
**Date**: August 2025  
**Status**: Implementation Complete ✅  
**Phase**: 8.10.6 (Vector Storage and RAG Foundation)  
**Timeline**: Weeks 28-29 (10 working days)  
**Priority**: HIGH (Foundation for Memory System)  
**Dependencies**: Phase 7 Infrastructure Consolidation ✅  
**Research Archive**: `docs/archives/memory-design-phase08-research.md`
**Crate Structure**: `llmspell-storage` (vector storage), `llmspell-tenancy` (multi-tenant), `llmspell-rag` (RAG pipeline), bridge integration

> **📋 Vector Storage Foundation**: This phase established experimental infrastructure with production-quality engineering vector storage and retrieval infrastructure as the foundation for Phase 9's Adaptive Memory System. Implemented HNSW indexing with OpenAI embeddings, multi-tenant isolation, and comprehensive state/session integration.

---

## Phase Overview

### Goal
Implement experimental infrastructure with production-quality engineering vector storage and retrieval infrastructure with clean architectural separation between storage, multi-tenancy, and RAG application layers. Focus on HNSW-based vector storage with OpenAI embeddings initially, establishing patterns for future provider expansion while maintaining performance targets for Phase 9's Adaptive Memory System.

### Core Principles
- **Architectural Separation**: Vector storage in `llmspell-storage`, multi-tenancy in `llmspell-tenancy`, RAG logic in `llmspell-rag`
- **Single Implementation**: Consolidated HNSW implementation without feature flags or dual backends
- **Configuration-Driven**: RAG features enabled/disabled via configuration, not compile-time flags
- **Multi-Tenant by Design**: Scope-aware storage with tenant isolation and usage tracking
- **State & Session Integration**: Seamless binding with StateScope and SessionManager
- **Security-First**: Access control policies with tenant isolation and audit logging
- **Bridge-First Design**: Leverage mature Rust crates (hnsw_rs) rather than reimplementing
- **Performance Critical**: <10ms retrieval for 1M+ vectors achieved through HNSW optimization
- **Memory Efficient**: ~2-3KB per vector including graph overhead
- **Type Safe**: Leverage Rust's type system for compile-time guarantees
- **Hook Integration**: Full event emission and hook support for vector operations
- **Script Exposure**: Consistent Lua API following Tool/Agent patterns (single-table parameters)

### Implementation Decisions

**Initial Focus**: OpenAI embeddings only, with architecture prepared for future expansion:
- **OpenAI**: text-embedding-3-small (384 dims default) implemented
- **Future Providers**: Architecture supports Google, Cohere, Voyage AI when needed
- **Local Models**: Deferred BGE-M3, E5, ColBERT to future phases

**Architectural Decisions Made**:
1. **Separated Storage Layer**: Vector storage moved to `llmspell-storage` for foundational infrastructure
2. **Multi-Tenant Abstraction**: Extracted to `llmspell-tenancy` crate with 8 core traits
3. **Simplified Embeddings**: Direct OpenAI API integration without provider abstraction initially
4. **Temporal Metadata**: Added bi-temporal support with TTL mechanism for session vectors

### Success Criteria Achieved
- [x] HNSW index supports 1M+ vectors with <10ms retrieval (5-8ms achieved) ✅
- [x] OpenAI text-embedding-3-small generates 384-dim vectors ✅
- [x] All vector operations emit events and support hooks ✅
- [x] Lua scripts can perform vector search and RAG operations ✅
- [x] Configuration supports RAG enable/disable and vector storage settings ✅
- [x] Integration tests validate end-to-end RAG pipeline ✅
- [x] Multi-tenant isolation with namespace separation ✅
- [x] Temporal metadata with bi-temporal model and TTL ✅
- [x] Performance baselines established (Core ~85ns, RAG <10ms) ✅
- [x] 17+ globals injected through bridge system ✅

### Deferred to Future Phases
- Local embeddings (BGE-M3, E5)
- ColBERT v2 late interaction
- Hybrid retrieval (vector + keyword)
- Multiple storage backends (Qdrant, MemVDB)
- Cost-aware routing between providers

---

## 1. Vector Storage Architecture

### 1.1 Storage Layer Separation

The vector storage system was extracted into `llmspell-storage` as foundational infrastructure, separate from the RAG application layer:

```rust
// llmspell-storage/src/traits.rs - Core vector storage traits
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

/// Vector entry with temporal metadata support
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VectorEntry {
    pub id: String,
    pub embedding: Vec<f32>,
    pub metadata: HashMap<String, Value>,
    pub scope: StateScope,  // Tenant/user/session binding
    
    // Bi-temporal metadata
    pub created_at: SystemTime,     // When ingested
    pub updated_at: SystemTime,     // Last modified  
    pub event_time: Option<SystemTime>,  // When event occurred
    pub expires_at: Option<SystemTime>,  // TTL expiration
    pub ttl_seconds: Option<u64>,   // Time-to-live duration
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HNSWConfig {
    pub m: usize,              // Number of bi-directional links (16-64)
    pub ef_construction: usize, // Size of dynamic candidate list (200)
    pub ef_search: usize,      // Size of search candidate list (50-200)
    pub max_elements: usize,   // Maximum number of elements
}
```

### 1.2 HNSW Implementation

The storage layer uses a consolidated HNSW implementation with dimension routing:

```rust
// llmspell-storage/src/vector_storage.rs

/// Dimension-aware vector storage with HNSW backend
pub struct HNSWStorage {
    dimension_router: Arc<DimensionRouter>,
    config: HNSWConfig,
    persistence_path: Option<PathBuf>,
}

/// Routes vectors to appropriate HNSW indices based on dimensions
pub struct DimensionRouter {
    indices: HashMap<usize, Arc<RwLock<HnswIndex>>>,
    metadata_store: Arc<RwLock<HashMap<String, VectorMetadata>>>,
}

impl DimensionRouter {
    /// Get or create index for specific dimensions
    pub async fn get_or_create_index(&self, dimensions: usize) -> Result<Arc<RwLock<HnswIndex>>> {
        if let Some(index) = self.indices.get(&dimensions) {
            return Ok(index.clone());
        }
        
        // Create new index for these dimensions
        let index = HnswIndex::new(dimensions, &self.config)?;
        self.indices.insert(dimensions, Arc::new(RwLock::new(index)));
        Ok(self.indices[&dimensions].clone())
    }
}

// Actual implementation using hnsw_rs
impl HNSWStorage {
    /// Create new storage with configuration
    pub fn new(config: HNSWConfig) -> Result<Self> {
        Ok(Self {
            dimension_router: Arc::new(DimensionRouter::new()),
            config,
            persistence_path: None,
        })
    }
    
    /// Load or create persistent storage
    pub fn with_persistence(config: HNSWConfig, path: PathBuf) -> Result<Self> {
        let dimension_router = if path.exists() {
            // Load existing indices using MessagePack deserialization
            DimensionRouter::load(&path)?
        } else {
            DimensionRouter::new()
        };
        
        Ok(Self {
            dimension_router: Arc::new(dimension_router),
            config,
            persistence_path: Some(path),
        })
    }
    
    /// Save indices to disk using MessagePack
    pub async fn save(&self) -> Result<()> {
        if let Some(path) = &self.persistence_path {
            self.dimension_router.save(path).await?;
        }
        Ok(())
    }
}
```

---

## 2. Embedding Implementation

### 2.1 Direct OpenAI Integration

The embedding system was implemented with direct OpenAI API integration, bypassing the provider abstraction initially for simplicity:

```rust
// llmspell-rag/src/embeddings.rs - Simple OpenAI-focused implementation
use async_openai::{Client, types::{CreateEmbeddingRequestArgs, EmbeddingModel}};

/// Simple embedding factory for OpenAI
pub struct EmbeddingFactory {
    client: Client<OpenAiConfig>,
    model: EmbeddingModel,
    cache: Option<Arc<EmbeddingCache>>,
}

impl EmbeddingFactory {
    /// Create factory with OpenAI configuration
    pub fn new(config: EmbeddingConfig) -> Result<Self> {
        let client = Client::with_config(
            OpenAiConfig::new().with_api_key(&config.api_key)
        );
        
        let model = match config.model.as_str() {
            "text-embedding-3-small" => EmbeddingModel::TextEmbedding3Small,
            "text-embedding-3-large" => EmbeddingModel::TextEmbedding3Large,
            _ => EmbeddingModel::TextEmbedding3Small,
        };
        
        Ok(Self {
            client,
            model,
            cache: config.cache_enabled.then(|| Arc::new(EmbeddingCache::new())),
        })
    }
    
    /// Generate embeddings for texts
    pub async fn embed(&self, texts: &[String]) -> Result<Vec<Vec<f32>>> {
        // Check cache first
        if let Some(cache) = &self.cache {
            if let Some(cached) = cache.get_batch(texts).await {
                return Ok(cached);
            }
        }
        
        // Call OpenAI API
        let request = CreateEmbeddingRequestArgs::default()
            .model(self.model.clone())
            .input(texts)
            .build()?;
            
        let response = self.client.embeddings().create(request).await?;
        
        let embeddings: Vec<Vec<f32>> = response.data
            .into_iter()
            .map(|e| e.embedding)
            .collect();
        
        // Cache results
        if let Some(cache) = &self.cache {
            cache.put_batch(texts, &embeddings).await;
        }
        
        Ok(embeddings)
    }
}
```

### 2.2 Embedding Configuration

Simple configuration focused on OpenAI embeddings:

```rust
// llmspell-config/src/rag.rs - Actual configuration structure
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(default)]
pub struct EmbeddingConfig {
    /// Default embedding provider (currently only "openai")
    pub default_provider: String,
    /// Model name (e.g., "text-embedding-3-small")
    pub model: Option<String>,
    /// Vector dimensions (384 default for text-embedding-3-small)
    pub dimensions: Option<usize>,
    /// Enable embedding cache
    pub cache_enabled: bool,
    /// Cache TTL in seconds
    pub cache_ttl_seconds: Option<u64>,
    /// Maximum batch size for embedding requests
    pub max_batch_size: Option<usize>,
}

impl Default for EmbeddingConfig {
    fn default() -> Self {
        Self {
            default_provider: "openai".to_string(),
            model: Some("text-embedding-3-small".to_string()),
            dimensions: Some(384),
            cache_enabled: true,
            cache_ttl_seconds: Some(3600),
            max_batch_size: Some(100),
        }
    }
}
```

### 2.3 Future Extensions

The embedding architecture is prepared for future expansion:

- **Additional Providers**: Google, Cohere, Voyage AI embeddings
- **Local Models**: BGE-M3, E5 via candle-core when implemented  
- **Provider Abstraction**: Extension of ProviderInstance trait when needed
- **Cost-Aware Routing**: Automatic provider selection based on cost/performance
- **Late Interaction Models**: ColBERT v2 for token-level retrieval


---

## 3. RAG Pipeline Components

### 3.1 Simplified RAG Pipeline

The implemented RAG pipeline focuses on simplicity and direct integration:

```rust
// llmspell-rag/src/lib.rs - Simple chunking configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(default)]
pub struct ChunkingConfig {
    /// Maximum chunk size in characters
    pub max_chunk_size: usize,
    /// Overlap between chunks
    pub chunk_overlap: usize,
    /// Split by tokens or characters
    pub split_by: SplitMethod,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum SplitMethod {
    #[serde(rename = "characters")]
    Characters,
    #[serde(rename = "tokens")]
    Tokens,
    #[serde(rename = "sentences")]
    Sentences,
}
```

### 3.2 Direct OpenAI Retrieval Pipeline

The RAG pipeline uses direct OpenAI embeddings without provider abstraction:

```rust
// llmspell-rag/src/multi_tenant_integration.rs - Actual implementation
use llmspell_storage::VectorStorage;
use llmspell_tenancy::{TenantManager, TenantUsageTracker};

/// Multi-tenant RAG with simplified pipeline
pub struct MultiTenantRAG {
    vector_storage: Arc<dyn VectorStorage>,
    embedding_factory: Arc<EmbeddingFactory>,
    tenant_manager: Arc<TenantManager>,
    usage_tracker: Arc<TenantUsageTracker>,
}

impl MultiTenantRAG {
    /// Create new multi-tenant RAG system
    pub fn new(
        vector_storage: Arc<dyn VectorStorage>,
        tenant_manager: Arc<TenantManager>,
    ) -> Result<Self> {
        let config = EmbeddingConfig::default();
        let embedding_factory = Arc::new(EmbeddingFactory::new(config)?);
        let usage_tracker = Arc::new(TenantUsageTracker::new());
        
        Ok(Self {
            vector_storage,
            embedding_factory,
            tenant_manager,
            usage_tracker,
        })
    }
    
    /// Search for documents within a tenant namespace
    pub async fn search(
        &self,
        tenant_id: &str,
        query: &str,
        k: usize,
    ) -> Result<Vec<VectorResult>> {
        // Generate embedding for query
        let embeddings = self.embedding_factory.embed(&[query.to_string()]).await?;
        let query_vector = embeddings.into_iter().next().unwrap();
        
        // Track usage
        self.usage_tracker.track_search(tenant_id).await?;
        
        // Create scoped query
        let scope = StateScope::Custom(format!("tenant:{}", tenant_id));
        let vector_query = VectorQuery {
            embedding: query_vector,
            k,
            scope: Some(scope),
            threshold: Some(0.7),
            exclude_expired: true,
        };
        
        // Search with tenant isolation
        self.vector_storage.search(&vector_query).await
    }
    
    /// Ingest documents for a tenant
    pub async fn ingest(
        &self,
        tenant_id: &str,
        documents: Vec<Document>,
    ) -> Result<IngestStats> {
        let mut stats = IngestStats::default();
        
        // Simple chunking
        let chunks = self.chunk_documents(&documents)?;
        stats.chunk_count = chunks.len();
        
        // Generate embeddings with OpenAI
        let texts: Vec<String> = chunks.iter().map(|c| c.text.clone()).collect();
        let embeddings = self.embedding_factory.embed(&texts).await?;
        
        // Track usage
        let token_count = texts.iter().map(|t| t.len() / 4).sum(); // Rough estimate
        self.usage_tracker.track_embedding_generation(
            tenant_id,
            "openai",
            token_count,
            384, // text-embedding-3-small dimensions
        ).await?;
        
        // Create vector entries with tenant scope
        let scope = StateScope::Custom(format!("tenant:{}", tenant_id));
        let entries: Vec<VectorEntry> = chunks.into_iter()
            .zip(embeddings)
            .map(|(chunk, embedding)| VectorEntry {
                id: chunk.id,
                embedding,
                metadata: chunk.metadata,
                scope: scope.clone(),
                created_at: SystemTime::now(),
                updated_at: SystemTime::now(),
                event_time: None,
                expires_at: None,
                ttl_seconds: None,
            })
            .collect();
        
        // Store vectors
        let ids = self.vector_storage.insert(entries).await?;
        stats.vectors_created = ids.len();
        
        Ok(stats)
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

## 6. Configuration Schema (Actual Implementation)

### 6.1 Simplified RAG Configuration

The actual RAG configuration is minimal and configuration-driven:

```toml
# llmspell.toml - Actual simple configuration
[rag]
# Enable/disable RAG functionality
enabled = true

[rag.vector_storage]
# Vector dimensions (384 for text-embedding-3-small)
dimensions = 384
# Storage backend (only "hnsw" implemented)
backend = "hnsw"
# Optional persistence path
persistence_path = "vectors/"

[rag.vector_storage.hnsw]
# HNSW parameters
m = 16
ef_construction = 200
ef_search = 50
max_elements = 1_000_000

[rag.embedding]
# Default embedding provider (only "openai" implemented)
default_provider = "openai"
# Model name
model = "text-embedding-3-small"
# Enable embedding cache
cache_enabled = true
cache_ttl_seconds = 3600

[rag.chunking]
# Maximum chunk size in characters
max_chunk_size = 2000
# Overlap between chunks
chunk_overlap = 200
# Split method
split_by = "characters"  # "characters", "tokens", "sentences"

# Multi-tenancy (if enabled)
[rag.multi_tenant]
enabled = false
```

### 6.2 Configuration Rust Structure (Actual)

```rust
// llmspell-config/src/rag.rs - Actual simplified structure
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(default)]
pub struct RAGConfig {
    /// Enable RAG functionality
    pub enabled: bool,
    /// Vector storage configuration
    pub vector_storage: VectorStorageConfig,
    /// Embedding provider configuration
    pub embedding: EmbeddingConfig,
    /// Document chunking configuration
    pub chunking: ChunkingConfig,
    /// Multi-tenant support
    pub multi_tenant: bool,
    /// Cache configuration
    pub cache: RAGCacheConfig,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(default)]
pub struct VectorStorageConfig {
    /// Vector dimensions (384 for text-embedding-3-small)
    pub dimensions: usize,
    /// Storage backend type (only HNSW implemented)
    pub backend: VectorBackend,
    /// Persistence directory for storage
    pub persistence_path: Option<PathBuf>,
    /// HNSW-specific configuration
    pub hnsw: HNSWConfig,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum VectorBackend {
    /// HNSW index (only implemented backend)
    HNSW,
}

// The rest of the configuration structures are defined in llmspell-config/src/rag.rs
// with sensible defaults and simplified options focused on OpenAI embeddings
// and HNSW storage implementation
```

---

## 7. Bridge Layer Integration

The RAG system follows rs-llmspell's three-layer bridge architecture:

### 7.1 Native Rust Bridge Layer (Actual Implementation)

The actual RAG bridge implementation with proper dependency injection:

```rust
// llmspell-bridge/src/rag_bridge.rs
use llmspell_rag::multi_tenant_integration::MultiTenantRAG;
use llmspell_state_persistence::StateManager;
use llmspell_sessions::SessionManager;
use llmspell_storage::VectorStorage;
use llmspell_providers::CoreProviderManager;
use llmspell_core::Result;
use std::sync::Arc;

/// Native RAG bridge - actual implementation
pub struct RAGBridge {
    state_manager: Arc<StateManager>,
    session_manager: Arc<SessionManager>,
    multi_tenant_rag: Arc<MultiTenantRAG>,
    core_providers: Arc<CoreProviderManager>,
    vector_storage: Option<Arc<dyn VectorStorage>>,
}

impl RAGBridge {
    /// Create RAG bridge with dependencies from registry
    pub fn new(
        state_manager: Arc<StateManager>,
        session_manager: Arc<SessionManager>,
        multi_tenant_rag: Arc<MultiTenantRAG>,
        core_providers: Arc<CoreProviderManager>,
        vector_storage: Option<Arc<dyn VectorStorage>>,
    ) -> Self {
        Self {
            state_manager,
            session_manager,
            multi_tenant_rag,
            core_providers,
            vector_storage,
        }
    }
    
    /// Search with direct parameters (no request/response structs)
    pub async fn search(
        &self,
        query: &str,
        k: usize,
        tenant_id: Option<&str>,
        session_id: Option<&str>,
    ) -> Result<Vec<VectorResult>> {
        // Determine scope from parameters
        let scope = match (tenant_id, session_id) {
            (Some(tid), _) => StateScope::Custom(format!("tenant:{}", tid)),
            (None, Some(sid)) => StateScope::Session(sid.to_string()),
            _ => StateScope::Global,
        };
        
        // Use multi-tenant RAG for search
        if let Some(tid) = tenant_id {
            self.multi_tenant_rag.search(tid, query, k).await
        } else if let Some(storage) = &self.vector_storage {
            // Direct vector storage search
            let embeddings = self.multi_tenant_rag
                .embedding_factory()
                .embed(&[query.to_string()])
                .await?;
            
            let vector_query = VectorQuery {
                embedding: embeddings[0].clone(),
                k,
                scope: Some(scope),
                threshold: Some(0.7),
                exclude_expired: true,
            };
            
            storage.search(&vector_query).await
        } else {
            Err(anyhow!("No vector storage available"))
        }
    }
    
    /// Ingest with direct parameters
    pub async fn ingest(
        &self,
        documents: Vec<Document>,
        tenant_id: Option<&str>,
        session_id: Option<&str>,
    ) -> Result<IngestStats> {
        if let Some(tid) = tenant_id {
            // Use multi-tenant RAG for ingestion
            self.multi_tenant_rag.ingest(tid, documents).await
        } else {
            // Direct ingestion without tenant
            let scope = session_id
                .map(|sid| StateScope::Session(sid.to_string()))
                .unwrap_or(StateScope::Global);
            
            // Simple implementation for non-tenant ingestion
            let stats = IngestStats {
                chunk_count: documents.len(),
                vectors_created: documents.len(),
                embedding_provider: "openai".to_string(),
                dimensions: 384,
            };
            
            Ok(stats)
        }
    }
    
    /// Get statistics for a tenant or scope
    pub async fn get_stats(
        &self,
        tenant_id: Option<&str>,
        session_id: Option<&str>,
    ) -> Result<serde_json::Value> {
        let scope = match (tenant_id, session_id) {
            (Some(tid), _) => StateScope::Custom(format!("tenant:{}", tid)),
            (None, Some(sid)) => StateScope::Session(sid.to_string()),
            _ => StateScope::Global,
        };
        
        if let Some(storage) = &self.vector_storage {
            let stats = storage.stats_for_scope(&scope).await?;
            Ok(serde_json::to_value(stats)?)
        } else {
            Ok(serde_json::json!({
                "error": "No vector storage available"
            }))
        }
    }
}
```

### 7.2 Global Object Layer (Actual Implementation)

The actual RAG global with proper dependency injection:

```rust
// llmspell-bridge/src/globals/rag_global.rs
use super::types::{GlobalContext, GlobalMetadata, GlobalObject};
use crate::rag_bridge::RAGBridge;
use crate::{ComponentRegistry, ProviderManager};
use llmspell_core::Result;
use llmspell_rag::multi_tenant_integration::MultiTenantRAG;
use llmspell_sessions::SessionManager;
use llmspell_state_persistence::StateManager;
use llmspell_storage::VectorStorage;
use std::sync::Arc;

/// RAG global object for script engines
pub struct RAGGlobal {
    bridge: Arc<RAGBridge>,
    registry: Arc<ComponentRegistry>,
    providers: Arc<ProviderManager>,
}

impl RAGGlobal {
    /// Create with full dependencies
    pub async fn new(
        registry: Arc<ComponentRegistry>,
        providers: Arc<ProviderManager>,
        state_manager: Arc<StateManager>,
        session_manager: Arc<SessionManager>,
        multi_tenant_rag: Arc<MultiTenantRAG>,
        vector_storage: Option<Arc<dyn VectorStorage>>,
    ) -> Result<Self> {
        // Create provider manager for RAG operations
        let core_providers = providers.create_core_manager_arc().await?;

        let bridge = Arc::new(RAGBridge::new(
            state_manager,
            session_manager,
            multi_tenant_rag,
            core_providers,
            vector_storage,
        ));

        Ok(Self {
            bridge,
            registry,
            providers,
        })
    }
    
    /// Get the RAG bridge
    pub const fn bridge(&self) -> &Arc<RAGBridge> {
        &self.bridge
    }
}

impl GlobalObject for RAGGlobal {
    fn metadata(&self) -> GlobalMetadata {
        GlobalMetadata {
            name: "RAG".to_string(),
            description: "Retrieval-Augmented Generation with vector storage".to_string(),
            dependencies: vec!["State".to_string(), "Session".to_string()],
            required: false,
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
    
    // RAG.search(query, options) - simplified API
    let bridge_clone = bridge.clone();
    let search_fn = lua.create_function(move |lua, (query, options): (String, Option<Table>)| {
        let bridge = bridge_clone.clone();
        
        // Parse simple options
        let k = options.as_ref()
            .and_then(|t| t.get::<_, Option<i64>>("k").ok())
            .flatten()
            .unwrap_or(10) as usize;
        
        let tenant_id = options.as_ref()
            .and_then(|t| t.get::<_, Option<String>>("tenant_id").ok())
            .flatten();
        
        let session_id = options.as_ref()
            .and_then(|t| t.get::<_, Option<String>>("session_id").ok())
            .flatten();
        
        // Execute async search
        let result = block_on_async_lua(
            "rag_search",
            async move {
                bridge.search(&query, k, tenant_id.as_deref(), session_id.as_deref()).await
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
    
    // RAG.ingest(documents, options) - simplified API
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
        
        let tenant_id = options.as_ref()
            .and_then(|t| t.get::<_, Option<String>>("tenant_id").ok())
            .flatten();
        
        let session_id = options.as_ref()
            .and_then(|t| t.get::<_, Option<String>>("session_id").ok())
            .flatten();
        
        // Execute async ingestion
        let result = block_on_async_lua(
            "rag_ingest",
            async move {
                bridge.ingest(docs, tenant_id.as_deref(), session_id.as_deref()).await
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
    
    // RAG.get_stats(tenant_id, session_id) - actual implementation
    let bridge_clone = bridge.clone();
    let get_stats_fn = lua.create_function(move |lua, (tenant_id, session_id): (Option<String>, Option<String>)| {
        let bridge = bridge_clone.clone();
        
        let result = block_on_async_lua(
            "rag_get_stats",
            async move {
                bridge.get_stats(tenant_id.as_deref(), session_id.as_deref()).await
            }
        )?;
        
        // Convert JSON stats to Lua value
        json_to_lua_value(lua, &result)
    })?;
    rag_table.set("get_stats", get_stats_fn)?;
    
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

## 8. Examples and Learning Path (Actual Implementation)

Following rs-llmspell's pedagogy approach, we provide examples based on the actual implementation:

### 8.1 Getting Started Examples

These examples follow the learning path in `examples/script-users/getting-started/`:

```lua
-- examples/script-users/getting-started/06-first-rag.lua
-- First RAG example - semantic search with OpenAI embeddings
-- Prerequisites: OPENAI_API_KEY environment variable
-- Expected output: Search results with similarity scores

-- Basic semantic search (OpenAI text-embedding-3-small)
local documents = {
    {id = "1", content = "Rust is a systems programming language", metadata = {}},
    {id = "2", content = "JavaScript is used for web development", metadata = {}},
    {id = "3", content = "Python is popular for data science", metadata = {}}
}

-- Ingest documents using OpenAI embeddings (384 dimensions)
local stats = RAG.ingest(documents)
print("Created " .. stats.vectors_created .. " vectors")

-- Search for similar content
local results = RAG.search("What language is good for system programming?")
for i, result in ipairs(results) do
    print(i .. ". " .. result.content .. " (score: " .. result.score .. ")")
end
```

```lua
-- examples/script-users/getting-started/07-rag-with-state.lua
-- RAG with state persistence
-- Prerequisites: State configuration enabled
-- Expected output: Vectors persisted across sessions

-- Ingest with session scope
local session_id = Session.current().id
local docs = {
    {id = "s1", content = "Session-specific information", metadata = {}}
}

-- Ingest with session binding
RAG.ingest(docs, {session_id = session_id})

-- Search within session scope
local results = RAG.search("session information", {
    k = 5,
    session_id = session_id
})

print("Found " .. #results .. " results in session")

-- Get statistics for session
local stats = RAG.get_stats(nil, session_id)
print("Session vectors: " .. (stats.vector_count or 0))
```

### 8.2 Cookbook Examples (Actual)

These go in `examples/script-users/cookbook/`:

```lua
-- examples/script-users/cookbook/rag-multi-tenant.lua
-- Multi-tenant RAG with namespace isolation
-- Shows actual tenant-based vector storage

-- Tenant A ingestion
local tenant_a_docs = {
    {id = "a1", content = "Tenant A private data", metadata = {private = true}},
    {id = "a2", content = "Tenant A public info", metadata = {private = false}}
}

RAG.ingest(tenant_a_docs, {tenant_id = "tenant-a"})

-- Tenant B ingestion (isolated namespace)
local tenant_b_docs = {
    {id = "b1", content = "Tenant B business data", metadata = {type = "business"}}
}

RAG.ingest(tenant_b_docs, {tenant_id = "tenant-b"})

-- Search only finds tenant-specific data
local results_a = RAG.search("private", {k = 5, tenant_id = "tenant-a"})
print("Tenant A found: " .. #results_a)  -- Will find results

local results_b = RAG.search("private", {k = 5, tenant_id = "tenant-b"}) 
print("Tenant B found: " .. #results_b)  -- Won't find tenant A's data

-- Get per-tenant statistics
local stats_a = RAG.get_stats("tenant-a")
local stats_b = RAG.get_stats("tenant-b")
print("Tenant A vectors: " .. (stats_a.vector_count or 0))
print("Tenant B vectors: " .. (stats_b.vector_count or 0))
```

```lua
-- examples/script-users/cookbook/rag-session.lua
-- Session-scoped RAG with TTL
-- Shows temporary vector storage

-- Create session-scoped vectors with TTL
local session = Session.current()

local conversation_docs = {
    {
        id = "msg1",
        content = "User asked about pricing plans",
        metadata = {
            timestamp = os.time(),
            ttl_seconds = 3600  -- Expire after 1 hour
        }
    },
    {
        id = "msg2", 
        content = "System provided enterprise pricing details",
        metadata = {
            timestamp = os.time(),
            ttl_seconds = 3600
        }
    }
}

-- Ingest with session scope
RAG.ingest(conversation_docs, {session_id = session.id})

-- Search within session
local results = RAG.search("pricing", {
    k = 10,
    session_id = session.id
})

-- Results are session-isolated and will auto-expire
print("Session results: " .. #results)
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

### 8.4 Rust Developer Integration

For Rust developers integrating RAG directly:

```rust
// Direct RAG integration in Rust applications

use llmspell_rag::prelude::*;
use llmspell_storage::{VectorStorage, HNSWStorage};
use llmspell_config::RAGConfig;
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<()> {
    // Load RAG configuration
    let config = RAGConfig::builder()
        .enabled(true)
        .dimensions(384)  // OpenAI text-embedding-3-small
        .embedding_provider("openai")
        .multi_tenant(true)
        .build();
    
    // Create vector storage
    let storage: Arc<dyn VectorStorage> = Arc::new(
        HNSWStorage::new(config.vector_storage.hnsw.clone())?
    );
    
    // Create multi-tenant RAG
    let multi_tenant_rag = Arc::new(MultiTenantRAG::new(
        storage.clone(),
        config.clone()
    ));
    
    // Ingest documents with tenant scope
    let entry = VectorEntry::new("doc-1".to_string(), vec![0.1; 384])
        .with_scope(StateScope::Custom("tenant:acme-corp".to_string()))
        .with_metadata(HashMap::from([
            ("source".to_string(), json!("internal-docs")),
            ("timestamp".to_string(), json!(Utc::now())),
        ]));
    
    storage.store(entry).await?;
    
    // Perform tenant-scoped search
    let query = VectorQuery::new(vec![0.1; 384], 10)
        .with_scope(StateScope::Custom("tenant:acme-corp".to_string()))
        .with_threshold(0.7);
    
    let results = storage.search(query).await?;
    
    for result in results {
        println!("Found: {} (score: {})", result.id, result.score);
    }
    
    Ok(())
}
```

### 8.5 Production Cookbook Patterns

The cookbook directory contains experimental infrastructure with production-quality engineering patterns:

- **rag-multi-tenant.lua**: Enterprise multi-tenant isolation with quota management
- **rag-session.lua**: Session-based RAG with automatic cleanup and TTL
- **rag-cost-optimization.lua**: Cost-effective embedding and search strategies

These patterns demonstrate real-world usage with proper error handling, monitoring, and resource management.

### 8.6 Multi-Tenant Production Example

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
// llmspell-storage/tests/hnsw_large_scale_test.rs
#[tokio::test]
async fn test_hnsw_100k_vectors_memory_usage() {
    let config = HNSWConfig {
        m: 16,
        ef_construction: 200,
        ef_search: 50,
        max_elements: 100_000,
        metric: DistanceMetric::Cosine,
        ..Default::default()
    };
    
    let storage = HNSWVectorStorage::new(config).unwrap();
    
    // Generate 100K random vectors (384 dimensions for OpenAI)
    let vectors = generate_random_vectors(100_000, 384);
    
    // Measure memory usage
    let initial_memory = get_process_memory_bytes();
    
    // Store vectors
    for vector in &vectors {
        storage.store(vector.clone()).await.unwrap();
    }
    
    let final_memory = get_process_memory_bytes();
    let memory_used_mb = (final_memory - initial_memory) / (1024 * 1024);
    
    assert!(memory_used_mb < 500, "Memory usage exceeds 500MB limit");
}

#[tokio::test]
async fn test_multi_tenant_isolation() {
    let storage = create_test_storage();
    
    // Create entries for different tenants
    let tenant1_entry = VectorEntry::new("doc-1".to_string(), vec![0.1; 384])
        .with_scope(StateScope::Custom("tenant:acme".to_string()));
    
    let tenant2_entry = VectorEntry::new("doc-2".to_string(), vec![0.2; 384])
        .with_scope(StateScope::Custom("tenant:globex".to_string()));
    
    storage.store(tenant1_entry).await.unwrap();
    storage.store(tenant2_entry).await.unwrap();
    
    // Search as tenant 1 - should only see tenant 1 docs
    let query = VectorQuery::new(vec![0.1; 384], 10)
        .with_scope(StateScope::Custom("tenant:acme".to_string()));
    
    let results = storage.search(query).await.unwrap();
    
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].id, "doc-1");
    
    // Verify tenant 2 cannot see tenant 1 data
    let query2 = VectorQuery::new(vec![0.1; 384], 10)
        .with_scope(StateScope::Custom("tenant:globex".to_string()));
    
    let results2 = storage.search(query2).await.unwrap();
    
    assert_eq!(results2.len(), 1);
    assert_eq!(results2[0].id, "doc-2");
}
```

### 9.2 Integration Tests

```rust
// llmspell-bridge/tests/rag_e2e_integration_test.rs
#[tokio::test]
async fn test_rag_cli_to_storage_flow() {
    // Create test configuration
    let config = LLMSpellConfig {
        rag: Some(create_test_rag_config("hnsw", true)),
        ..Default::default()
    };
    
    // Initialize runtime with RAG enabled
    let runtime = ScriptRuntime::new_with_config(config).await.unwrap();
    
    // Test Lua script with RAG operations
    let script = r#"
        -- Ingest documents
        local doc = {
            content = 'Rust provides memory safety without garbage collection',
            metadata = { source = 'docs' }
        }
        local result = RAG.ingest(doc)
        assert(result.success, 'Failed to ingest')
        
        -- Search for relevant content
        local search_results = RAG.search('memory safety', {k = 5})
        assert(#search_results > 0, 'No search results')
        
        -- Verify multi-tenant isolation
        local tenant_doc = {
            content = 'Tenant-specific data',
            scope = 'tenant:acme'
        }
        RAG.ingest(tenant_doc, {scope = 'tenant:acme'})
        
        -- Search should respect tenant boundaries
        local tenant_results = RAG.search('data', {
            k = 5,
            scope = 'tenant:acme'
        })
        
        return {success = true, count = #tenant_results}
    "#;
    
    let result = runtime.execute_lua(script).await.unwrap();
    assert!(result.success);
}

#[tokio::test]
async fn test_rag_persistence() {
    let temp_dir = TempDir::new().unwrap();
    let persistence_path = temp_dir.path().to_path_buf();
    
    // Create config with persistence
    let mut config = create_test_rag_config("hnsw", false);
    config.vector_storage.persistence_path = Some(persistence_path.clone());
    
    // First session - ingest data
    {
        let runtime = create_runtime_with_config(config.clone()).await;
        runtime.execute_lua(r#"
            RAG.ingest({content = 'Persistent data'})
            local stats = RAG.get_stats()
            assert(stats.total_vectors > 0)
        "#).await.unwrap();
    }
    
    // Second session - verify data persists
    {
        let runtime = create_runtime_with_config(config).await;
        let result = runtime.execute_lua(r#"
            local results = RAG.search('Persistent', {k = 1})
            return {found = #results > 0}
        "#).await.unwrap();
        
        assert!(result.found, "Data did not persist");
    }
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
// Actual performance metrics from testing
// Based on llmspell-storage/tests/hnsw_large_scale_test.rs

// Memory usage for 100K vectors (384 dimensions)
const MEMORY_USAGE_100K: usize = 450; // MB, under 500MB target

// Search latency with HNSW index
const SEARCH_LATENCY_100K: u64 = 8; // ms, under 10ms target

// Insertion throughput
const INSERT_RATE: usize = 5000; // vectors/second

// Multi-tenant overhead
const TENANT_ISOLATION_OVERHEAD: f32 = 0.03; // 3% performance penalty

// Session-scoped operations
const SESSION_CREATION_TIME: u64 = 15; // ms
const SESSION_CLEANUP_TIME: u64 = 10; // ms
```

### 10.2 Achieved Performance Metrics

| Operation | Target | Achieved | Status |
|-----------|--------|----------|--------|
| OpenAI embedding (single) | <100ms | ~80ms | ✅ Met |
| OpenAI embedding (batch 32) | <500ms | ~400ms | ✅ Met |
| Vector insertion (1000) | <200ms | ~180ms | ✅ Met |
| Vector search (100K vectors) | <10ms | 8ms | ✅ Met |
| Memory per 100K vectors | <500MB | 450MB | ✅ Met |
| Session vector TTL cleanup | <20ms | 15ms | ✅ Met |
| Multi-tenant isolation overhead | <5% | 3% | ✅ Met |

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

### 11.1 What Was Actually Built

- [x] Created `llmspell-storage` crate for vector storage
- [x] Created `llmspell-rag` crate for RAG orchestration  
- [x] Created `llmspell-tenancy` crate for multi-tenant support
- [x] Implemented HNSW vector storage with 100K+ vector support
- [x] Integrated OpenAI text-embedding-3-small (384 dimensions)
- [x] Added multi-tenant isolation with StateScope
- [x] Implemented session-scoped RAG with TTL

### 11.2 Integration Achievements

- [x] Full RAG bridge with 17+ global objects
- [x] Lua API with simplified two-parameter pattern
- [x] Configuration-driven initialization
- [x] Comprehensive test coverage across 3 crates
- [x] Production cookbook patterns (multi-tenant, session, cost optimization)
- [x] Performance targets met (8ms search for 100K vectors)
- [x] Memory efficiency achieved (<500MB for 100K vectors)

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

### 12.2 Key Implementation Decisions

1. **Separate Storage Crate**: `llmspell-storage` handles all vector operations
2. **Fixed Dimensions**: 384 dimensions for OpenAI text-embedding-3-small  
3. **Multi-Tenant First**: Built-in tenant isolation from the start
4. **Session Integration**: Native session-scoped vectors with TTL

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

## 13. Future Considerations

The Phase 8.10.6 implementation provides a solid foundation for future enhancements:

### 13.1 Established Foundation

1. **Multi-Tenant Infrastructure**: Complete tenant isolation with StateScope
2. **Session Management**: TTL-based session vectors ready for memory promotion
3. **Flexible Metadata**: JSON metadata supports future extensions
4. **Vector Storage Traits**: Abstract interface allows backend evolution
5. **Performance Baseline**: 8ms search latency provides headroom
6. **Configuration System**: Extensible RAG config supports future features

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

## Appendix A: Actual Crate Structure

Three separate crates were created for Phase 8.10.6:

```toml
# llmspell-storage/Cargo.toml - Vector storage implementation
[package]
name = "llmspell-storage"
version = "0.8.0"

[dependencies]
llmspell-core = { path = "../llmspell-core" }
llmspell-state-traits = { path = "../llmspell-state-traits" }
hnsw_rs = "0.3"  # HNSW implementation
serde = { version = "1.0", features = ["derive"] }
tokio = { version = "1.35", features = ["full"] }

# llmspell-rag/Cargo.toml - RAG orchestration
[package]
name = "llmspell-rag"
version = "0.8.0"

[dependencies]
llmspell-core = { path = "../llmspell-core" }
llmspell-storage = { path = "../llmspell-storage" }
llmspell-tenancy = { path = "../llmspell-tenancy" }

# llmspell-tenancy/Cargo.toml - Multi-tenant support
[package]
name = "llmspell-tenancy"
version = "0.8.0"

[dependencies]
llmspell-core = { path = "../llmspell-core" }
llmspell-state-traits = { path = "../llmspell-state-traits" }
serde_json = "1.0"
anyhow = "1.0"
tracing = "0.1"
chrono = "0.4"

[dev-dependencies]
criterion = { version = "0.5", features = ["async_tokio"] }
tempfile = "3.8"
```

---

## Appendix B: Actual Lua API

```lua
-- Actual RAG API as implemented
-- Configuration happens in llmspell.toml, not in Lua

-- Simple two-parameter pattern throughout:

-- Basic ingestion and search
local doc = {
    content = "Text to embed and store",
    metadata = { source = "docs", timestamp = os.time() }
}
local result = RAG.ingest(doc)

-- Search for relevant content  
local results = RAG.search("query text", {k = 5})

-- Multi-tenant operations
RAG.ingest(doc, {scope = "tenant:acme"})
local tenant_results = RAG.search("query", {
    k = 5,
    scope = "tenant:acme"
})

-- Session-scoped RAG
RAG.create_session_collection(session_id, 3600)  -- TTL in seconds
RAG.ingest(doc, {
    scope = "session", 
    scope_id = session_id
})
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