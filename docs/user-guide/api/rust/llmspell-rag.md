# llmspell-rag

## Purpose

Retrieval-Augmented Generation system providing document ingestion, chunking, embedding generation, vector storage integration, and context retrieval for LLM augmentation. This Phase 8 crate enables applications to ground LLM responses in specific knowledge bases with multi-tenant support and session-specific collections.

## Core Concepts

- **Document Pipeline**: Ingestion -> Chunking -> Embedding -> Storage -> Retrieval
- **Chunking Strategies**: Fixed-size, sliding window, semantic, and recursive splitting
- **Embedding Providers**: Support for OpenAI, Cohere, HuggingFace, and local models
- **Multi-Tenant Isolation**: Tenant-specific document collections and retrieval
- **Session Collections**: Temporary collections for session-specific context
- **Hybrid Search**: Combine vector similarity with keyword and metadata filtering
- **Context Window Management**: Optimize retrieved context for LLM token limits
- **Incremental Indexing**: Add documents without rebuilding entire index

## Primary Traits/Structs

### RAGPipeline

**Purpose**: Main orchestrator for the complete RAG workflow from ingestion to retrieval.

```rust
use llmspell_rag::{
    RAGPipeline, RAGConfig, Document, ChunkingStrategy,
    EmbeddingProvider, RetrievalOptions
};

pub struct RAGPipeline {
    config: RAGConfig,
    chunker: Box<dyn DocumentChunker>,
    embedder: Arc<dyn EmbeddingProvider>,
    storage: Arc<dyn VectorStorage>,
    metadata_index: MetadataIndex,
}

impl RAGPipeline {
    pub async fn new(config: RAGConfig) -> Result<Self> {
        let chunker = ChunkingStrategy::from_config(&config.chunking)?;
        let embedder = EmbeddingProvider::from_config(&config.embedding).await?;
        let storage = VectorStorage::from_config(&config.storage).await?;
        
        Ok(Self {
            config,
            chunker: Box::new(chunker),
            embedder: Arc::new(embedder),
            storage: Arc::new(storage),
            metadata_index: MetadataIndex::new(),
        })
    }
    
    /// Ingest document into RAG system
    pub async fn ingest(
        &self,
        document: Document,
        tenant_id: Option<&str>,
    ) -> Result<IngestResult> {
        // Chunk document
        let chunks = self.chunker.chunk(&document).await?;
        
        // Generate embeddings
        let embeddings = self.embedder.embed_batch(
            &chunks.iter().map(|c| c.text.as_str()).collect::<Vec<_>>()
        ).await?;
        
        // Store in vector database
        let mut stored_ids = Vec::new();
        for (chunk, embedding) in chunks.iter().zip(embeddings) {
            let metadata = self.build_metadata(&document, chunk, tenant_id);
            
            let entry = VectorEntry {
                id: Uuid::new_v4().to_string(),
                vector: embedding,
                metadata,
                payload: Some(chunk.text.as_bytes().to_vec()),
            };
            
            let id = self.storage.store(&self.config.collection, entry).await?;
            stored_ids.push(id);
            
            // Update metadata index
            self.metadata_index.index(&entry).await?;
        }
        
        Ok(IngestResult {
            document_id: document.id,
            chunk_count: chunks.len(),
            stored_ids,
        })
    }
    
    /// Retrieve relevant context for query
    pub async fn retrieve(
        &self,
        query: &str,
        options: RetrievalOptions,
    ) -> Result<Vec<RetrievedContext>> {
        // Generate query embedding
        let query_embedding = self.embedder.embed(query).await?;
        
        // Search vector storage
        let search_options = VectorSearchOptions {
            limit: options.top_k * 2, // Over-fetch for re-ranking
            threshold: options.similarity_threshold,
            metadata_filter: options.build_filter(),
            include_vectors: false,
        };
        
        let results = self.storage.search(
            &self.config.collection,
            &query_embedding,
            search_options,
        ).await?;
        
        // Re-rank if configured
        let ranked_results = if let Some(reranker) = &self.config.reranker {
            reranker.rerank(query, results).await?
        } else {
            results
        };
        
        // Build context entries
        let mut contexts = Vec::new();
        for result in ranked_results.into_iter().take(options.top_k) {
            contexts.push(RetrievedContext {
                content: String::from_utf8_lossy(&result.payload.unwrap_or_default()).to_string(),
                score: result.score,
                metadata: result.metadata,
                source: result.metadata.get("source").and_then(|v| v.as_str()).map(String::from),
            });
        }
        
        Ok(contexts)
    }
}
```

### DocumentChunker Trait

**Purpose**: Interface for different document chunking strategies.

```rust
#[async_trait]
pub trait DocumentChunker: Send + Sync {
    /// Chunk document into smaller pieces
    async fn chunk(&self, document: &Document) -> Result<Vec<Chunk>>;
    
    /// Estimate token count for text
    fn estimate_tokens(&self, text: &str) -> usize {
        text.len() / 4 // Rough estimate
    }
}

pub enum ChunkingStrategy {
    /// Fixed-size chunks with optional overlap
    FixedSize {
        size: usize,
        overlap: usize,
    },
    
    /// Sliding window with configurable stride
    SlidingWindow {
        window_size: usize,
        stride: usize,
    },
    
    /// Semantic chunking based on sentence boundaries
    Semantic {
        min_size: usize,
        max_size: usize,
        sentence_detector: Box<dyn SentenceDetector>,
    },
    
    /// Recursive splitting for structured documents
    Recursive {
        separators: Vec<String>,
        chunk_size: usize,
        chunk_overlap: usize,
    },
}

impl ChunkingStrategy {
    pub fn chunk(&self, document: &Document) -> Vec<Chunk> {
        match self {
            Self::FixedSize { size, overlap } => {
                let mut chunks = Vec::new();
                let text = &document.content;
                let mut start = 0;
                
                while start < text.len() {
                    let end = (start + size).min(text.len());
                    chunks.push(Chunk {
                        text: text[start..end].to_string(),
                        start_offset: start,
                        end_offset: end,
                        metadata: HashMap::new(),
                    });
                    
                    start += size - overlap;
                }
                
                chunks
            }
            // Other strategies...
        }
    }
}
```

### EmbeddingProvider Trait

**Purpose**: Abstraction for different embedding model providers.

```rust
#[async_trait]
pub trait EmbeddingProvider: Send + Sync {
    /// Generate embedding for single text
    async fn embed(&self, text: &str) -> Result<Vec<f32>>;
    
    /// Generate embeddings for batch of texts
    async fn embed_batch(&self, texts: &[&str]) -> Result<Vec<Vec<f32>>>;
    
    /// Get embedding dimension
    fn dimension(&self) -> usize;
    
    /// Get model name
    fn model_name(&self) -> &str;
}

/// OpenAI embeddings provider
pub struct OpenAIEmbeddings {
    client: OpenAIClient,
    model: String,
    dimension: usize,
}

impl OpenAIEmbeddings {
    pub async fn new(api_key: String, model: Option<String>) -> Result<Self> {
        let model = model.unwrap_or_else(|| "text-embedding-3-small".to_string());
        let dimension = match model.as_str() {
            "text-embedding-3-small" => 1536,
            "text-embedding-3-large" => 3072,
            "text-embedding-ada-002" => 1536,
            _ => return Err(RAGError::UnsupportedModel(model)),
        };
        
        Ok(Self {
            client: OpenAIClient::new(api_key),
            model,
            dimension,
        })
    }
}

#[async_trait]
impl EmbeddingProvider for OpenAIEmbeddings {
    async fn embed(&self, text: &str) -> Result<Vec<f32>> {
        let response = self.client.embeddings()
            .model(&self.model)
            .input(text)
            .build()
            .send()
            .await?;
        
        Ok(response.data[0].embedding.clone())
    }
    
    async fn embed_batch(&self, texts: &[&str]) -> Result<Vec<Vec<f32>>> {
        // OpenAI supports batch embedding
        let response = self.client.embeddings()
            .model(&self.model)
            .input(texts.to_vec())
            .build()
            .send()
            .await?;
        
        Ok(response.data.into_iter()
            .map(|d| d.embedding)
            .collect())
    }
    
    fn dimension(&self) -> usize {
        self.dimension
    }
    
    fn model_name(&self) -> &str {
        &self.model
    }
}
```

### MultiTenantRAG

**Purpose**: RAG system with tenant isolation for SaaS applications.

```rust
pub struct MultiTenantRAG {
    pipelines: Arc<RwLock<HashMap<String, Arc<RAGPipeline>>>>,
    default_config: RAGConfig,
    storage: Arc<dyn VectorStorage>,
}

impl MultiTenantRAG {
    /// Get or create tenant-specific pipeline
    pub async fn get_pipeline(&self, tenant_id: &str) -> Arc<RAGPipeline> {
        let pipelines = self.pipelines.read().await;
        
        if let Some(pipeline) = pipelines.get(tenant_id) {
            return pipeline.clone();
        }
        
        drop(pipelines);
        
        // Create new pipeline for tenant
        let mut config = self.default_config.clone();
        config.collection = format!("tenant_{}", tenant_id);
        
        let pipeline = Arc::new(RAGPipeline::new(config).await.unwrap());
        
        let mut pipelines = self.pipelines.write().await;
        pipelines.insert(tenant_id.to_string(), pipeline.clone());
        
        pipeline
    }
    
    /// Ingest document for specific tenant
    pub async fn ingest(
        &self,
        tenant_id: &str,
        document: Document,
    ) -> Result<IngestResult> {
        let pipeline = self.get_pipeline(tenant_id).await;
        pipeline.ingest(document, Some(tenant_id)).await
    }
    
    /// Search within tenant's documents
    pub async fn search(
        &self,
        tenant_id: &str,
        query: &str,
        options: RetrievalOptions,
    ) -> Result<Vec<RetrievedContext>> {
        let pipeline = self.get_pipeline(tenant_id).await;
        
        // Ensure tenant filter is applied
        let mut options = options;
        options.metadata_filters.insert(
            "tenant_id".to_string(),
            json!(tenant_id),
        );
        
        pipeline.retrieve(query, options).await
    }
}
```

## Usage Patterns

### Basic RAG Pipeline

**When to use**: Standard document Q&A applications.

**Benefits**: Simple setup with sensible defaults.

**Example**:
```rust
use llmspell_rag::{RAGPipeline, RAGConfig, Document};

async fn setup_basic_rag() -> Result<RAGPipeline> {
    let config = RAGConfig::default()
        .with_collection("knowledge_base")
        .with_embedding_model("text-embedding-3-small")
        .with_chunking_strategy(ChunkingStrategy::FixedSize {
            size: 500,
            overlap: 50,
        });
    
    RAGPipeline::new(config).await
}

async fn ingest_documents(pipeline: &RAGPipeline) -> Result<()> {
    let documents = vec![
        Document {
            id: "doc1".to_string(),
            content: "LLMSpell is a framework for building LLM applications...".to_string(),
            metadata: hashmap! {
                "source" => json!("documentation"),
                "category" => json!("framework"),
            },
        },
        // More documents...
    ];
    
    for doc in documents {
        pipeline.ingest(doc, None).await?;
    }
    
    Ok(())
}

async fn answer_question(
    pipeline: &RAGPipeline,
    question: &str,
) -> Result<String> {
    // Retrieve relevant context
    let contexts = pipeline.retrieve(question, RetrievalOptions {
        top_k: 5,
        similarity_threshold: Some(0.7),
        ..Default::default()
    }).await?;
    
    // Build prompt with context
    let context_text = contexts.iter()
        .map(|c| &c.content)
        .collect::<Vec<_>>()
        .join("\n\n");
    
    let prompt = format!(
        "Based on the following context, answer the question.\n\n\
         Context:\n{}\n\n\
         Question: {}\n\n\
         Answer:",
        context_text, question
    );
    
    // Call LLM with augmented prompt
    // ... LLM call here ...
    
    Ok("answer".to_string())
}
```

### Session-Specific RAG

**When to use**: Chat applications where context is session-specific.

**Benefits**: Isolated context per conversation, automatic cleanup.

**Example**:
```rust
use llmspell_rag::{SessionRAG, SessionConfig};

pub struct ChatWithRAG {
    session_rag: Arc<SessionRAG>,
    llm_client: Arc<dyn LLMClient>,
}

impl ChatWithRAG {
    pub async fn create_session(&self, session_id: &str) -> Result<()> {
        self.session_rag.create_session(session_id, SessionConfig {
            ttl: Duration::from_secs(3600), // 1 hour
            max_documents: 100,
            auto_cleanup: true,
        }).await
    }
    
    pub async fn add_to_context(
        &self,
        session_id: &str,
        content: &str,
        source: &str,
    ) -> Result<()> {
        let document = Document {
            id: Uuid::new_v4().to_string(),
            content: content.to_string(),
            metadata: hashmap! {
                "source" => json!(source),
                "timestamp" => json!(SystemTime::now()),
            },
        };
        
        self.session_rag.ingest_to_session(session_id, document).await
    }
    
    pub async fn chat(
        &self,
        session_id: &str,
        message: &str,
    ) -> Result<String> {
        // Retrieve session-specific context
        let contexts = self.session_rag.search_session(
            session_id,
            message,
            RetrievalOptions::default(),
        ).await?;
        
        // Generate response with context
        let response = self.llm_client.complete_with_context(
            message,
            contexts,
        ).await?;
        
        // Add response to session context for future queries
        self.add_to_context(session_id, &response, "assistant").await?;
        
        Ok(response)
    }
}
```

## Integration Examples

### With LLM Agents

```rust
use llmspell_rag::RAGPipeline;
use llmspell_agents::{Agent, Tool};

pub struct RAGTool {
    pipeline: Arc<RAGPipeline>,
}

#[async_trait]
impl Tool for RAGTool {
    fn name(&self) -> &str {
        "knowledge_search"
    }
    
    async fn execute(&self, params: Value) -> Result<Value> {
        let query = params["query"].as_str()
            .ok_or("Missing query parameter")?;
        
        let contexts = self.pipeline.retrieve(query, RetrievalOptions {
            top_k: params["top_k"].as_u64().unwrap_or(5) as usize,
            ..Default::default()
        }).await?;
        
        Ok(json!({
            "contexts": contexts.iter().map(|c| {
                json!({
                    "content": c.content,
                    "score": c.score,
                    "source": c.source,
                })
            }).collect::<Vec<_>>(),
        }))
    }
}

// Use in agent
let agent = Agent::builder()
    .name("rag_agent")
    .tool(RAGTool { pipeline })
    .system_prompt("Use the knowledge_search tool to find relevant information before answering.")
    .build();
```

## Configuration

```toml
[rag]
default_collection = "knowledge_base"

# Chunking configuration
[rag.chunking]
strategy = "fixed_size"  # or "sliding_window", "semantic", "recursive"
chunk_size = 500
chunk_overlap = 50

# Embedding configuration
[rag.embedding]
provider = "openai"  # or "cohere", "huggingface", "local"
model = "text-embedding-3-small"
batch_size = 100
cache_embeddings = true

# Storage configuration
[rag.storage]
backend = "hnsw"
collection_prefix = "rag_"

# Retrieval configuration
[rag.retrieval]
default_top_k = 5
default_threshold = 0.7
enable_reranking = false
max_context_length = 4000

# Multi-tenant configuration
[rag.multi_tenant]
enabled = true
isolation_level = "strict"  # or "shared"
per_tenant_collections = true
```

## Performance Considerations

- **Batch Embedding**: Process documents in batches to reduce API calls
- **Embedding Cache**: Cache embeddings for frequently accessed documents
- **Chunk Size**: Balance between context granularity and retrieval accuracy
- **Index Optimization**: Periodically optimize HNSW index for better performance
- **Async Processing**: Use async ingestion for large document sets
- **Metadata Filtering**: Pre-filter with metadata before vector search
- **Incremental Updates**: Add new documents without full reindexing

## Security Considerations

- **Tenant Isolation**: Always enforce tenant filters in multi-tenant setups
- **Input Sanitization**: Sanitize document content before ingestion
- **API Key Security**: Store embedding provider API keys securely
- **Access Control**: Implement document-level access control
- **PII Detection**: Scan for and redact PII before storage
- **Rate Limiting**: Limit ingestion and retrieval rates per tenant
- **Audit Logging**: Log all document access for compliance

## Migration Guide

### New in v0.8.0 (Phase 8)

Features:
- Multi-tenant RAG with strict isolation
- Session-specific collections
- HNSW vector storage integration
- Multiple embedding providers
- Hybrid search capabilities
- Incremental indexing

Migration steps:
1. Update RAGConfig to specify collection names
2. Add tenant_id to all ingest operations
3. Configure embedding provider credentials
4. Set up vector storage backend
5. Update retrieval calls to use RetrievalOptions