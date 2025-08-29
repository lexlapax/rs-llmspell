# llmspell-rag

Retrieval-Augmented Generation (RAG) framework for Rs-LLMSpell with production-grade document processing, multi-tenant isolation, and high-performance vector retrieval.

## Features

### Document Processing Pipeline
- **Smart Chunking**: Multiple strategies (semantic, fixed-size, recursive) with configurable overlap
- **Tokenizer Integration**: Accurate token counting and boundary detection for optimal chunk sizes
- **Metadata Extraction**: Automatic extraction of document structure, headings, and contextual information
- **Batch Processing**: High-throughput document ingestion with parallel processing

### Embedding Management
- **Multi-Provider Support**: OpenAI, Anthropic, local models with automatic fallback
- **Intelligent Caching**: Persistent embedding cache with LRU eviction and deduplication
- **Dimension Routing**: Automatic model selection based on embedding dimensions
- **Factory Pattern**: Pluggable embedding providers with standardized interface

### RAG Pipeline
- **Builder Pattern**: Fluent configuration of ingestion and retrieval workflows
- **Hybrid Retrieval**: Combine vector similarity with keyword search and reranking
- **Context Assembly**: Intelligent context window management with relevance scoring
- **Quality Metrics**: Built-in evaluation of retrieval quality and relevance

### Enterprise Integration
- **Multi-Tenant Isolation**: Complete data separation with tenant-scoped operations
- **Session Integration**: Long-running RAG sessions with conversation context
- **State Management**: Persistent RAG state with atomic updates and rollback
- **Audit Logging**: Complete traceability of document access and retrieval operations

## Usage

### Basic RAG Pipeline
```rust
use llmspell_rag::{
    RAGPipeline, ChunkingStrategy, EmbeddingProvider, RetrievalConfig
};
use llmspell_state_traits::StateScope;

// Create RAG pipeline with production configuration
let pipeline = RAGPipeline::builder()
    .with_chunking(ChunkingStrategy::Semantic {
        chunk_size: 512,
        overlap: 50,
        min_chunk_size: 100,
    })
    .with_embedding_provider(EmbeddingProvider::OpenAI {
        model: "text-embedding-ada-002".to_string(),
        api_key: api_key.clone(),
    })
    .with_vector_storage(vector_storage)
    .with_retrieval_config(RetrievalConfig {
        top_k: 10,
        similarity_threshold: 0.8,
        rerank: true,
    })
    .build().await?;

// Ingest documents with tenant isolation
let documents = vec![
    Document::new("user-guide.md", content)
        .with_metadata("document_type", "documentation")
        .with_scope(StateScope::Custom("tenant:company-123".to_string())),
];

let ingestion_result = pipeline.ingest_documents(documents).await?;
println!("Ingested {} chunks", ingestion_result.chunks_created);
```

### Advanced Multi-Tenant Retrieval
```rust
use llmspell_rag::{
    MultiTenantRAG, TenantConfig, RetrievalQuery, ContextAssembly
};

// Create multi-tenant RAG system
let mt_rag = MultiTenantRAG::new(vector_storage, embedding_factory).await?;

// Configure tenant-specific settings
mt_rag.configure_tenant("company-123", TenantConfig {
    max_chunks_per_query: 5,
    similarity_threshold: 0.85,
    enable_reranking: true,
    context_window_size: 8192,
}).await?;

// Perform tenant-scoped retrieval
let query = RetrievalQuery::new("How do I configure authentication?")
    .with_tenant("company-123")
    .with_filters(vec![("document_type", "documentation")])
    .with_context_assembly(ContextAssembly::Relevance);

let results = mt_rag.retrieve(&query).await?;

// Assemble context with metadata
let context = results.assemble_context(8192)?;  // Token limit
println!("Retrieved {} chunks, context: {} tokens", 
    results.chunks.len(), context.token_count);
```

### Session-Integrated RAG
```rust
use llmspell_rag::{SessionRAG, ConversationMemory};
use llmspell_sessions::Session;

// Create session-aware RAG
let session = Session::create("research-session").await?;
let session_rag = SessionRAG::new(pipeline, session.id()).await?;

// Maintain conversation context
session_rag.set_conversation_memory(ConversationMemory {
    max_turns: 10,
    context_decay: 0.9,
    relevance_boost: 1.2,
}).await?;

// Query with conversation context
let response = session_rag.query_with_context(
    "What were the performance metrics we discussed?",
    &conversation_history
).await?;

// Automatic context updates
session_rag.update_conversation_context(
    &user_message,
    &assistant_response
).await?;
```

### Hybrid Retrieval with Reranking
```rust
use llmspell_rag::{
    HybridRetriever, VectorRetrieval, KeywordRetrieval, CrossEncoder
};

// Combine multiple retrieval strategies
let hybrid = HybridRetriever::builder()
    .with_vector_retrieval(VectorRetrieval {
        weight: 0.7,
        top_k: 20,
    })
    .with_keyword_retrieval(KeywordRetrieval {
        weight: 0.3,
        algorithm: "BM25",
        top_k: 20,
    })
    .with_reranker(CrossEncoder {
        model: "cross-encoder/ms-marco-MiniLM-L-6-v2",
        top_k: 10,
    })
    .build().await?;

let results = hybrid.retrieve(query, &retrieval_context).await?;
```

### Custom Embedding Provider
```rust
use llmspell_rag::{EmbeddingProvider, EmbeddingRequest, EmbeddingResponse};

#[async_trait]
impl EmbeddingProvider for CustomProvider {
    async fn embed_texts(&self, texts: &[String]) -> Result<Vec<Vec<f32>>, RAGError> {
        // Custom embedding logic
        let embeddings = self.model.encode(texts).await?;
        Ok(embeddings)
    }
    
    fn dimensions(&self) -> usize {
        self.model.dimensions()
    }
    
    fn model_name(&self) -> &str {
        "custom-embedder-v1"
    }
}

// Register custom provider
let factory = EmbeddingFactory::new();
factory.register_provider("custom", Box::new(CustomProvider::new())).await?;
```

## Performance Characteristics

### Document Ingestion
- **Chunking**: 1-10MB/sec depending on strategy and document complexity
- **Embedding Generation**: 100-1000 texts/sec (provider dependent)
- **Vector Storage**: <10ms insert time for batches of 100+ vectors
- **Memory Usage**: ~2-5MB per 1000 document chunks

### Retrieval Performance
- **Vector Search**: <10ms for similarity search across 1M+ vectors
- **Hybrid Retrieval**: <50ms including reranking for 10K+ document corpus
- **Context Assembly**: <5ms for typical 8K token context windows
- **Multi-Tenant Overhead**: <5% per tenant for query operations

### Scalability
- **Document Corpus**: Tested up to 1M documents (10GB+ text content)
- **Concurrent Queries**: 100+ concurrent retrieval operations
- **Tenant Isolation**: Zero cross-tenant data leakage with <5% performance overhead
- **Memory Efficiency**: Configurable caching with LRU eviction

## Architecture

```
llmspell-rag
├── chunking/
│   ├── strategies.rs       # Semantic, fixed-size, recursive chunking
│   ├── tokenizer.rs        # Token counting and boundary detection
│   └── mod.rs              # Chunking strategy interface
├── embeddings/
│   ├── provider.rs         # EmbeddingProvider trait
│   ├── openai.rs           # OpenAI embedding provider
│   ├── local.rs            # Local model provider (e.g., SentenceTransformers)
│   ├── cache.rs            # Persistent embedding cache with deduplication
│   ├── factory.rs          # Provider factory with automatic selection
│   └── dimensions.rs       # Multi-dimensional embedding routing
├── pipeline/
│   ├── rag_pipeline.rs     # Main RAG pipeline implementation
│   ├── builder.rs          # Pipeline builder with fluent configuration
│   ├── ingestion.rs        # Document ingestion workflow
│   ├── retrieval_flow.rs   # Query processing and retrieval flow
│   └── config.rs           # Pipeline configuration and validation
├── traits/
│   └── hybrid.rs           # Hybrid retrieval trait definitions
├── retrieval.rs            # Core retrieval algorithms and ranking
├── multi_tenant_integration.rs  # Multi-tenant RAG operations
├── session_integration.rs       # Session-aware RAG functionality
└── state_integration.rs         # State management integration
```

## Dependencies
- `llmspell-core` - Core traits and error types
- `llmspell-storage` - Vector storage and HNSW integration
- `llmspell-state-traits` - State management and tenant scoping
- `llmspell-security` - Access control and tenant isolation
- `llmspell-sessions` - Session management integration
- `tokenizers` - Tokenization and text processing
- `reqwest` - HTTP client for embedding providers
- `serde` - Serialization for configuration and caching
- `tracing` - Structured logging and performance monitoring

## Configuration Examples

See `examples/script-users/configs/` for production-ready configurations:
- `rag-basic.toml` - Simple RAG setup
- `rag-multi-tenant.toml` - Multi-tenant isolation
- `rag-performance.toml` - High-performance configuration
- `rag-production.toml` - Production deployment settings