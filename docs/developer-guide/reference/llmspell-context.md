# llmspell-context

## Purpose

Context engineering pipeline providing intelligent retrieval, reranking, and assembly of relevant context for LLM prompts. This Phase 13 crate implements query understanding, multi-strategy retrieval, and token-budget-aware context assembly to optimize LLM performance.

## Core Concepts

- **Query Understanding**: Intent classification, entity extraction, keyword detection
- **Retrieval Strategies**: Episodic (temporal), Semantic (graph), Hybrid, RAG
- **Reranking**: DeBERTa cross-encoder (Candle) and BM25 fallback for relevance scoring
- **Context Assembly**: Token budget management, temporal ordering, confidence scoring
- **Parallel Retrieval**: ~2x speedup using `tokio::join!` for hybrid strategies
- **Token-Aware**: Respects LLM context window limits with truncation
- **Metadata Enrichment**: Temporal spans, confidence scores, source attribution

## Primary Traits/Structs

### ContextPipeline

**Purpose**: End-to-end orchestration of context engineering from query to assembled context.

```rust
use llmspell_context::prelude::*;
use llmspell_memory::DefaultMemoryManager;

let memory = DefaultMemoryManager::new_in_memory().await?;

let pipeline = ContextPipeline::builder()
    .with_reranker(BM25Reranker::default())
    .with_assembler(ContextAssembler::default())
    .build()?;

let context = pipeline.process_query(
    "What is Rust ownership?",
    memory.episodic(),
    memory.semantic(),
    2000,  // max tokens
).await?;
```

### RetrievalStrategy Enum

**Purpose**: Select retrieval approach based on query type and requirements.

```rust
pub enum RetrievalStrategy {
    /// Recent conversation history (temporal, session-scoped)
    Episodic,

    /// Knowledge graph entities (conceptual, global)
    Semantic,

    /// Combine episodic + semantic (comprehensive)
    Hybrid,

    /// RAG vector search + episodic memory (documents + conversations)
    RAG,
}

// Strategy selection
let strategy = RetrievalStrategy::Hybrid;
```

### ContextChunk

**Purpose**: Represents a single retrieved chunk of context with scoring metadata.

```rust
pub struct ContextChunk {
    pub chunk: ChunkData,       // Actual content
    pub score: f32,             // Relevance score 0-1
    pub ranker: String,         // Reranking method ("bm25", "deberta")
}

pub struct ChunkData {
    pub role: String,           // "user", "assistant", "system"
    pub content: String,        // Chunk text
    pub source: String,         // Source identifier
    pub timestamp: DateTime<Utc>, // Temporal ordering
    pub metadata: HashMap<String, String>,
}
```

### AssembledContext

**Purpose**: Final structured context ready for LLM prompting.

```rust
pub struct AssembledContext {
    pub chunks: Vec<ContextChunk>,  // Ranked chunks
    pub total_confidence: f32,       // Average relevance
    pub temporal_span: (DateTime<Utc>, DateTime<Utc>), // Time range
    pub token_count: usize,          // Total tokens
    pub formatted: String,           // LLM-ready text
}

// Usage in prompts
let context = pipeline.assemble(...).await?;
let prompt = format!(
    "{}\n\nBased on the above context, {}",
    context.formatted,
    user_question
);
```

## Retrieval Strategies

### Episodic Retrieval

**Purpose**: Retrieve recent conversation history from episodic memory.

```rust
use llmspell_context::retrieval::EpisodicRetriever;

let retriever = EpisodicRetriever::new(memory.episodic());

let chunks = retriever.retrieve(
    "Rust ownership",
    Some("session-123"), // Session filter
    10,                  // Limit
).await?;

// Returns: Recent conversation exchanges about ownership
// Sorted by: Temporal recency (newest first)
// Scope: Session-specific
```

### Semantic Retrieval

**Purpose**: Retrieve entities and relationships from knowledge graph.

```rust
use llmspell_context::retrieval::SemanticRetriever;

let retriever = SemanticRetriever::new(memory.semantic());

let chunks = retriever.retrieve(
    "Rust ownership",
    None,  // No session filter (global knowledge)
    10,
).await?;

// Returns: Entities like "Ownership", "Borrowing", "Lifetimes"
// Sorted by: Graph centrality + name similarity
// Scope: Global knowledge base
```

### Hybrid Retrieval

**Purpose**: Combine episodic and semantic retrieval for comprehensive context.

```rust
use llmspell_context::retrieval::HybridRetriever;

let retriever = HybridRetriever::new(
    memory.episodic(),
    memory.semantic(),
);

let chunks = retriever.retrieve(
    "Rust ownership",
    Some("session-123"),
    20,  // Total limit (split between episodic and semantic)
).await?;

// Returns: Mix of conversation history + knowledge entities
// Parallel execution: ~2x speedup using tokio::join!
// Deduplication: Removes duplicate content
```

### RAG Retrieval

**Purpose**: Combine RAG pipeline (documents) with episodic memory (conversations).

```rust
use llmspell_context::retrieval::RAGRetriever;
use llmspell_rag::RAGPipeline;

let rag_pipeline = RAGPipeline::new(config).await?;

let retriever = RAGRetriever::new(
    Arc::new(rag_pipeline),
    memory.episodic(),
);

let chunks = retriever.retrieve(
    "Rust ownership",
    Some("session-123"),
    15,
).await?;

// Returns: Document chunks (40%) + conversation history (60%)
// Weighting: Configurable via weights parameter
// Fallback: Uses hybrid if RAG unavailable
```

## Reranking

### BM25Reranker (Default)

**Purpose**: Keyword-based reranking using BM25 algorithm.

```rust
use llmspell_context::reranking::BM25Reranker;

let reranker = BM25Reranker::default();

let ranked_chunks = reranker.rerank(
    "Rust ownership and borrowing",
    unranked_chunks,
).await?;

// Advantages: Fast, no dependencies, works offline
// Best for: Keyword-heavy queries
```

### DeBERTaReranker (Advanced)

**Purpose**: Neural reranking using DeBERTa cross-encoder model.

```rust
use llmspell_context::reranking::DeBERTaReranker;

let reranker = DeBERTaReranker::new().await?;

let ranked_chunks = reranker.rerank(
    "What are the benefits of Rust's ownership model?",
    unranked_chunks,
).await?;

// Advantages: Better semantic understanding, handles paraphrasing
// Requirements: Candle backend, model download (~400MB)
// Performance: ~50ms per chunk
```

## Context Assembly

### ContextAssembler

**Purpose**: Assemble ranked chunks into token-budget-aware context.

```rust
use llmspell_context::assembly::ContextAssembler;

let assembler = ContextAssembler::default();

let context = assembler.assemble(
    ranked_chunks,
    2000,  // max_tokens
    "temporal",  // ordering: "temporal", "score", "mixed"
).await?;

// Features:
// - Token counting (accurate approximation)
// - Truncation at token limit
// - Temporal ordering preservation
// - Confidence scoring
// - LLM-ready formatting
```

## Usage Patterns

### Basic Context Assembly

```rust
use llmspell_context::prelude::*;
use llmspell_memory::DefaultMemoryManager;

async fn basic_context_assembly() -> Result<()> {
    let memory = DefaultMemoryManager::new_in_memory().await?;

    // Add some conversation history
    memory.episodic().add(
        EpisodicEntry::new(
            "session-1".into(),
            "user".into(),
            "What is Rust?".into(),
        )
    ).await?;

    memory.episodic().add(
        EpisodicEntry::new(
            "session-1".into(),
            "assistant".into(),
            "Rust is a systems programming language...".into(),
        )
    ).await?;

    // Assemble context
    let retriever = EpisodicRetriever::new(memory.episodic());
    let chunks = retriever.retrieve("Rust", Some("session-1"), 10).await?;

    let reranker = BM25Reranker::default();
    let ranked = reranker.rerank("Rust programming", chunks).await?;

    let assembler = ContextAssembler::default();
    let context = assembler.assemble(ranked, 2000, "temporal").await?;

    println!("Context: {}", context.formatted);
    println!("Tokens: {}", context.token_count);
    println!("Confidence: {:.2}", context.total_confidence);

    Ok(())
}
```

### End-to-End Pipeline

```rust
use llmspell_context::prelude::*;

async fn e2e_pipeline() -> Result<()> {
    let memory = DefaultMemoryManager::new_in_memory().await?;

    let pipeline = ContextPipeline::builder()
        .with_reranker(BM25Reranker::default())
        .with_assembler(ContextAssembler::default())
        .build()?;

    let context = pipeline.process_query(
        "How does Rust prevent data races?",
        memory.episodic(),
        memory.semantic(),
        2000,
    ).await?;

    // Use in LLM prompt
    let prompt = format!(
        "Context:\n{}\n\nQuestion: How does Rust prevent data races?",
        context.formatted
    );

    Ok(())
}
```

### Hybrid Retrieval with Parallel Execution

```rust
use llmspell_context::retrieval::HybridRetriever;

async fn parallel_hybrid_retrieval() -> Result<()> {
    let memory = DefaultMemoryManager::new_in_memory().await?;

    let retriever = HybridRetriever::new(
        memory.episodic(),
        memory.semantic(),
    );

    // Parallel execution: episodic and semantic in parallel
    let start = std::time::Instant::now();

    let chunks = retriever.retrieve(
        "Rust ownership model",
        Some("session-1"),
        20,
    ).await?;

    let duration = start.elapsed();
    println!("Retrieved {} chunks in {:?} (parallel)", chunks.len(), duration);

    // Typically ~2x faster than sequential retrieval
    Ok(())
}
```

### RAG + Memory Integration

```rust
use llmspell_context::retrieval::RAGRetriever;
use llmspell_rag::RAGPipeline;

async fn rag_memory_integration() -> Result<()> {
    let memory = DefaultMemoryManager::new_in_memory().await?;

    // Setup RAG pipeline
    let rag_pipeline = RAGPipeline::new(RAGConfig::default()).await?;

    // Ingest documents
    rag_pipeline.ingest(document, None).await?;

    // Create RAG+Memory retriever
    let retriever = RAGRetriever::new(
        Arc::new(rag_pipeline),
        memory.episodic(),
    );

    // Retrieve: 40% RAG documents + 60% conversation history
    let chunks = retriever.retrieve(
        "Rust ownership",
        Some("session-1"),
        15,
    ).await?;

    // Chunks contain both document excerpts and conversation exchanges
    Ok(())
}
```

### Token Budget Management

```rust
use llmspell_context::assembly::ContextAssembler;

async fn token_budget_management() -> Result<()> {
    let assembler = ContextAssembler::default();

    // Small budget (fits in prompt)
    let small_context = assembler.assemble(
        chunks.clone(),
        500,  // 500 tokens
        "score",
    ).await?;

    assert!(small_context.token_count <= 500);
    println!("Small context: {} chunks, {} tokens",
        small_context.chunks.len(),
        small_context.token_count
    );

    // Large budget (comprehensive)
    let large_context = assembler.assemble(
        chunks.clone(),
        8000,  // 8K tokens
        "temporal",
    ).await?;

    println!("Large context: {} chunks, {} tokens",
        large_context.chunks.len(),
        large_context.token_count
    );

    Ok(())
}
```

## Integration with Other Crates

### With llmspell-memory

Context retrieval depends on memory subsystems:

```rust
use llmspell_context::ContextBridge;
use llmspell_memory::DefaultMemoryManager;

let memory = DefaultMemoryManager::new_in_memory().await?;
let context_bridge = ContextBridge::new(memory.clone());

// Exposes context.assemble() to Lua scripts
```

### With llmspell-bridge (Lua API)

Context exposed via `Context` global:

```lua
-- Assemble context from memory
local context = Context.assemble(
    "Rust ownership",
    "episodic",
    2000,
    "session-123"
)

print("Retrieved " .. #context.chunks .. " chunks")
print("Confidence: " .. context.total_confidence)
print("Tokens: " .. context.token_count)

-- Use formatted context
local prompt = context.formatted .. "\n\nUser: " .. user_query
```

### With llmspell-rag

RAG retrieval strategy integrates with RAG pipeline:

```rust
use llmspell_context::retrieval::RAGRetriever;
use llmspell_rag::RAGPipeline;

let rag = RAGPipeline::new(config).await?;
let retriever = RAGRetriever::new(Arc::new(rag), memory.episodic());
```

## Performance Characteristics

| Operation | Time | Notes |
|-----------|------|-------|
| Episodic retrieval (10 chunks) | ~5ms | HNSW vector search |
| Semantic retrieval (10 entities) | ~10ms | Graph traversal |
| Hybrid retrieval (20 chunks) | ~8ms | Parallel execution (~2x speedup) |
| BM25 reranking (20 chunks) | ~2ms | Keyword scoring |
| DeBERTa reranking (20 chunks) | ~1s | Neural cross-encoder |
| Context assembly (2K tokens) | ~1ms | Token counting + formatting |

**Total Pipeline (episodic + BM25 + assembly)**: ~8ms for typical query

## Error Handling

```rust
use llmspell_context::{ContextError, Result};

async fn handle_context_errors() -> Result<()> {
    let retriever = EpisodicRetriever::new(memory.episodic());

    match retriever.retrieve("query", Some("session"), 10).await {
        Ok(chunks) => println!("Retrieved {} chunks", chunks.len()),
        Err(ContextError::MemoryError(e)) => eprintln!("Memory error: {e}"),
        Err(ContextError::TokenBudgetExceeded { requested, limit }) => {
            eprintln!("Token budget exceeded: {requested} > {limit}");
        },
        Err(ContextError::InvalidStrategy(s)) => eprintln!("Invalid strategy: {s}"),
        Err(e) => eprintln!("Other error: {e}"),
    }

    Ok(())
}
```

## Testing

```rust
use llmspell_context::prelude::*;

#[tokio::test]
async fn test_context_assembly() {
    let memory = DefaultMemoryManager::new_in_memory().await.unwrap();

    // Add test data
    memory.episodic().add(
        EpisodicEntry::new("test".into(), "user".into(), "Test message".into())
    ).await.unwrap();

    // Retrieve
    let retriever = EpisodicRetriever::new(memory.episodic());
    let chunks = retriever.retrieve("Test", Some("test"), 10).await.unwrap();

    assert!(!chunks.is_empty());

    // Rerank
    let reranker = BM25Reranker::default();
    let ranked = reranker.rerank("Test", chunks).await.unwrap();

    // Assemble
    let assembler = ContextAssembler::default();
    let context = assembler.assemble(ranked, 1000, "score").await.unwrap();

    assert!(context.token_count <= 1000);
    assert!(!context.formatted.is_empty());
}
```

## Configuration

### Strategy Selection Guidelines

| Query Type | Recommended Strategy | Reason |
|------------|---------------------|--------|
| "What did we discuss about X?" | Episodic | Temporal, session-scoped |
| "What is X?" | Semantic | Conceptual knowledge |
| "Explain X based on our conversation and general knowledge" | Hybrid | Comprehensive |
| "Find information about X in documents" | RAG | Document search + memory |

### Token Budget Recommendations

- **Short prompts (API calls)**: 500-1000 tokens
- **Medium prompts (chat)**: 2000-4000 tokens
- **Long prompts (research)**: 8000-16000 tokens
- **Maximum safe budget**: 32000 tokens (depends on LLM)

## Related Documentation

- [llmspell-memory](llmspell-memory.md) - Memory system providing episodic and semantic backends
- [llmspell-graph](llmspell-graph.md) - Knowledge graph for semantic retrieval
- [llmspell-rag](llmspell-rag.md) - RAG pipeline integration
- [Context Lua API](../lua/README.md#context) - Script-level context access
- [Phase 13 Design](../../../../docs/in-progress/phase-13-design-doc.md) - Architecture

---

**Phase 13 Integration** | Intelligent context engineering for LLM prompts | [RAG Memory Integration Guide](../../rag-memory-integration.md)
