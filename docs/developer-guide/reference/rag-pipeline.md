# RAG Pipeline & Context Engineering

**Thematic guide to Retrieval-Augmented Generation architecture**

🔗 **Quick Links**: [llmspell-rag](llmspell-rag.md) | [llmspell-context](llmspell-context.md) | [llmspell-graph](llmspell-graph.md) | [Crate Index](crate-index.md)

---

## Overview

llmspell's RAG (Retrieval-Augmented Generation) pipeline provides a complete system for augmenting LLM responses with retrieved knowledge. Phase 13 adds sophisticated context engineering with multi-strategy retrieval, reranking, and bi-temporal knowledge graphs.

**Core Components**:
- **llmspell-rag**: Document ingestion, chunking, embedding, retrieval (Phase 8)
- **llmspell-context**: Context assembly, query understanding, reranking (Phase 13)
- **llmspell-graph**: Bi-temporal knowledge graph for semantic memory (Phase 13)

---

## Architecture Overview

```
Document Input
    ↓
[llmspell-rag]
    ├─→ Document Loading (PDF, Markdown, HTML)
    ├─→ Chunking (semantic, fixed-size, recursive)
    ├─→ Embedding (OpenAI, local models)
    └─→ Vector Storage (HNSW)
    ↓
User Query
    ↓
[llmspell-context]
    ├─→ Query Understanding (intent classification)
    ├─→ Multi-Strategy Retrieval (episodic, semantic, hybrid)
    ├─→ Reranking (DeBERTa cross-encoder)
    └─→ Context Assembly (token-budget aware)
    ↓
[llmspell-graph]
    ├─→ Entity Extraction
    ├─→ Relationship Mapping
    └─→ Temporal Queries
    ↓
Augmented Prompt → LLM → Response
```

---

## RAG Pipeline (llmspell-rag)

**Purpose**: Document ingestion and retrieval for knowledge augmentation

### Document Ingestion

**Supported Formats**:
- Markdown (.md)
- PDF (.pdf)
- HTML (.html)
- Plain text (.txt)
- JSON (.json)

**Ingestion Pipeline**:
```rust
use llmspell_rag::{RAGPipeline, Document, ChunkingStrategy};

let rag = RAGPipeline::builder()
    .with_collection("documentation")
    .with_embedding_model("text-embedding-3-small")
    .build()?;

// Load documents
let documents = vec![
    Document::from_file("docs/getting-started.md")?,
    Document::from_url("https://example.com/api-docs").await?,
    Document::from_text("Manual entry", "content..."),
];

// Ingest with chunking
for doc in documents {
    rag.ingest(doc, ChunkingStrategy::Semantic).await?;
}
```

### Chunking Strategies

**1. Semantic Chunking** (default, recommended):
```rust
ChunkingStrategy::Semantic {
    max_chunk_size: 512,  // tokens
    overlap: 50,          // tokens
    boundary_detection: true,  // Respect paragraph boundaries
}
```

**Benefits**:
- Preserves semantic coherence
- Respects document structure
- Better retrieval quality

**2. Fixed-Size Chunking**:
```rust
ChunkingStrategy::FixedSize {
    chunk_size: 256,
    overlap: 32,
}
```

**3. Recursive Chunking**:
```rust
ChunkingStrategy::Recursive {
    max_chunk_size: 1024,
    separators: vec!["\n\n", "\n", ". ", " "],
}
```

**4. Custom Chunking**:
```rust
impl ChunkingStrategy for CustomChunker {
    async fn chunk(&self, document: &Document) -> Result<Vec<Chunk>> {
        // Custom logic
    }
}
```

### Embedding Models

**OpenAI Embeddings**:
```rust
use llmspell_rag::embeddings::OpenAIEmbedder;

let embedder = OpenAIEmbedder::new("text-embedding-3-small")?;
// Dimensions: 1536, Cost: $0.02/1M tokens
```

**Local Embeddings** (Candle):
```rust
use llmspell_rag::embeddings::CandleEmbedder;

let embedder = CandleEmbedder::new("all-MiniLM-L6-v2")?;
// Dimensions: 384, Cost: Free (CPU/GPU)
```

### Retrieval

**Basic Retrieval**:
```rust
let results = rag.search(
    "How do I configure the system?",
    k: 5,  // Top 5 results
    None   // No metadata filter
).await?;

for result in results {
    println!("Chunk: {}\nScore: {}", result.content, result.score);
}
```

**Filtered Retrieval**:
```rust
use llmspell_storage::MetadataFilter;

let filter = MetadataFilter::builder()
    .eq("category", "configuration")
    .gt("version", "1.0.0")
    .build();

let results = rag.search_with_filter("query", 5, Some(filter)).await?;
```

**Hybrid Search** (keyword + semantic):
```rust
let results = rag.hybrid_search(
    "configuration options",
    k: 10,
    keyword_weight: 0.3,    // BM25 weight
    semantic_weight: 0.7     // Vector similarity weight
).await?;
```

### Multi-Tenant RAG

**Tenant Isolation**:
```rust
async fn ingest_tenant_document(
    rag: &RAGPipeline,
    tenant_id: &str,
    document: Document,
) -> Result<()> {
    let collection = format!("tenant_{}:docs", tenant_id);

    // Ensure collection exists
    rag.create_collection(&collection).await?;

    // Add tenant ID to metadata
    let mut doc = document;
    doc.metadata.insert("tenant_id", tenant_id);

    rag.ingest_to_collection(&collection, doc).await?;
    Ok(())
}
```

📚 **Full Details**: [llmspell-rag.md](llmspell-rag.md)

---

## Context Engineering (llmspell-context)

**Purpose**: Intelligent context assembly with query understanding and reranking

### Context Assembly Pipeline

**1. Query Understanding**:
```rust
use llmspell_context::{ContextAssembler, QueryIntent};

let assembler = ContextAssembler::new(config)?;

// Classify query intent
let intent = assembler.understand_query("What is RAG?").await?;

match intent {
    QueryIntent::Factual => {
        // Use semantic search
    }
    QueryIntent::Procedural => {
        // Use episodic memory (how-to steps)
    }
    QueryIntent::Contextual => {
        // Use hybrid approach
    }
}
```

**2. Multi-Strategy Retrieval**:
```rust
// Episodic retrieval (conversation history)
let episodic_results = assembler.retrieve_episodic(query, k: 3).await?;

// Semantic retrieval (knowledge graph)
let semantic_results = assembler.retrieve_semantic(query, k: 3).await?;

// RAG retrieval (document chunks)
let rag_results = assembler.retrieve_rag(query, k: 5).await?;

// Parallel retrieval (~2x speedup)
let all_results = assembler.retrieve_parallel(query, strategies: vec![
    RetrievalStrategy::Episodic,
    RetrievalStrategy::Semantic,
    RetrievalStrategy::RAG,
]).await?;
```

**3. Reranking**:
```rust
use llmspell_context::reranking::CrossEncoderReranker;

// DeBERTa cross-encoder for relevance scoring
let reranker = CrossEncoderReranker::new("cross-encoder/ms-marco-MiniLM-L-6-v2")?;

let reranked = reranker.rerank(query, candidates, top_k: 5).await?;

// Reranked results ordered by true relevance
for (i, result) in reranked.iter().enumerate() {
    println!("{}. {} (score: {:.3})", i+1, result.content, result.score);
}
```

**4. Token-Budget-Aware Assembly**:
```rust
let context = assembler.assemble_context(
    query: "How to set up PostgreSQL?",
    token_budget: 2000,       // Max tokens for context
    strategies: vec![
        RetrievalStrategy::Episodic,
        RetrievalStrategy::RAG,
    ],
    rerank: true
).await?;

// Context guaranteed to fit in token budget
assert!(context.token_count() <= 2000);
```

### Retrieval Strategies

**Episodic Strategy**:
- Retrieves from conversation history
- Uses recency and relevance
- Best for: Contextual queries, follow-ups

**Semantic Strategy**:
- Queries knowledge graph
- Follows entity relationships
- Best for: Factual queries, entity-centric questions

**Hybrid Strategy**:
- Combines episodic + semantic + RAG
- Parallel retrieval for speed
- Reranks combined results
- Best for: Complex queries requiring multiple sources

**RAG Strategy**:
- Traditional vector search
- Document chunk retrieval
- Best for: Documentation queries, how-to questions

### Context Optimization

**Confidence Scoring**:
```rust
let results = assembler.retrieve_with_confidence(query).await?;

for result in results {
    if result.confidence > 0.8 {
        println!("High confidence: {}", result.content);
    } else {
        println!("Low confidence (score: {:.2}): {}", result.confidence, result.content);
    }
}
```

**Temporal Ordering**:
```rust
// Order results by relevance and recency
let context = assembler.assemble_context_with_options(
    query,
    ContextOptions {
        temporal_decay: true,     // Decay old results
        decay_factor: 0.95,       // 5% decay per day
        max_age: Duration::days(30),  // Ignore results older than 30 days
        ..Default::default()
    }
).await?;
```

📚 **Full Details**: [llmspell-context.md](llmspell-context.md)

---

## Bi-Temporal Knowledge Graph (llmspell-graph)

**Purpose**: Track entities, relationships, and temporal evolution of knowledge

### Bi-Temporal Model

**Two Time Dimensions**:
1. **Event Time**: When the fact was true in the real world
2. **Ingestion Time**: When we learned about the fact

**Benefits**:
- Correct historical errors without losing history
- Time-travel queries ("What did we know on Jan 1?")
- Track knowledge evolution
- Support for late-arriving facts

### Entity and Relationship Storage

**Creating Entities**:
```rust
use llmspell_graph::{KnowledgeGraph, Entity, Relationship};

let graph = KnowledgeGraph::new(config)?;

// Create entity
let entity = Entity {
    id: "llmspell",
    type_: "software",
    properties: json!({
        "name": "llmspell",
        "description": "AI experimentation platform",
        "version": "0.13.0"
    }),
    event_time: Utc::now(),  // When this info is true
};

graph.add_entity(entity).await?;
```

**Creating Relationships**:
```rust
let relationship = Relationship {
    from: "llmspell",
    to: "rust",
    type_: "written_in",
    properties: json!({
        "since": "2024-01-01"
    }),
    event_time: Utc::now(),
};

graph.add_relationship(relationship).await?;
```

### Querying the Graph

**Entity Queries**:
```rust
// Find entity by ID
let entity = graph.get_entity("llmspell").await?;

// Find entities by type
let software_entities = graph.find_entities_by_type("software").await?;

// Find entities by property
let entities = graph.find_entities_where(
    "version",
    Operator::GreaterThan,
    "0.12.0"
).await?;
```

**Relationship Queries**:
```rust
// Find relationships from entity
let outgoing = graph.get_relationships_from("llmspell").await?;

// Find relationships to entity
let incoming = graph.get_relationships_to("llmspell").await?;

// Find relationships by type
let dependencies = graph.find_relationships_by_type("depends_on").await?;
```

**Graph Traversal**:
```rust
// Find all dependencies (recursive)
let dependencies = graph.traverse(
    "llmspell",
    TraversalType::Outgoing("depends_on"),
    max_depth: 5
).await?;

// Find shortest path
let path = graph.shortest_path(
    from: "llmspell",
    to: "tokio",
    relationship_type: Some("depends_on")
).await?;
```

### Temporal Queries

**Point-in-Time Query**:
```rust
// "What did the graph look like on 2024-01-01?"
let snapshot = graph.query_at_time(
    DateTime::parse_from_rfc3339("2024-01-01T00:00:00Z")?
).await?;

// Query entities as of that time
let entities = snapshot.get_entity("llmspell").await?;
```

**Time Range Query**:
```rust
// "What changed between these dates?"
let changes = graph.query_time_range(
    start: DateTime::parse_from_rfc3339("2024-01-01T00:00:00Z")?,
    end: DateTime::parse_from_rfc3339("2024-02-01T00:00:00Z")?
).await?;

for change in changes {
    match change.type_ {
        ChangeType::EntityAdded(entity) => println!("Added: {:?}", entity),
        ChangeType::EntityModified(before, after) => println!("Modified: {:?} -> {:?}", before, after),
        ChangeType::EntityDeleted(entity) => println!("Deleted: {:?}", entity),
        _ => {}
    }
}
```

**Correction Handling**:
```rust
// Original (incorrect) fact
graph.add_entity(Entity {
    id: "project_x",
    properties: json!({ "launch_date": "2024-03-01" }),
    event_time: DateTime::parse_from_rfc3339("2024-03-01T00:00:00Z")?,
}).await?;

// Correction (actual launch was earlier)
graph.correct_entity(
    id: "project_x",
    properties: json!({ "launch_date": "2024-02-15" }),
    event_time: DateTime::parse_from_rfc3339("2024-02-15T00:00:00Z")?,
    correction_time: Utc::now(),  // When we learned about the correction
).await?;

// Both versions preserved in history
let history = graph.get_entity_history("project_x").await?;
assert_eq!(history.len(), 2);
```

### Integration with Memory System

**Semantic Memory Backend**:
```rust
use llmspell_memory::{SemanticMemory, MemoryBackend};
use llmspell_graph::KnowledgeGraph;

let graph = Arc::new(KnowledgeGraph::new(config)?);

let semantic_memory = SemanticMemory::builder()
    .with_backend(MemoryBackend::Graph(graph))
    .build()?;

// Memories stored as graph entities
semantic_memory.add("llmspell is written in Rust").await?;
semantic_memory.add("Rust guarantees memory safety").await?;

// Query uses graph traversal
let results = semantic_memory.query("What language is llmspell written in?").await?;
// Traverses: llmspell -> written_in -> Rust -> guarantees -> memory safety
```

📚 **Full Details**: [llmspell-graph.md](llmspell-graph.md)

---

## Complete RAG Example

### End-to-End Implementation

```rust
use llmspell_rag::RAGPipeline;
use llmspell_context::ContextAssembler;
use llmspell_graph::KnowledgeGraph;
use llmspell_providers::OpenAIProvider;

#[tokio::main]
async fn main() -> Result<()> {
    // 1. Initialize components
    let rag = RAGPipeline::builder()
        .with_collection("docs")
        .with_embedding_model("text-embedding-3-small")
        .build()?;

    let graph = Arc::new(KnowledgeGraph::new(config)?);

    let context_assembler = ContextAssembler::builder()
        .with_rag_pipeline(rag.clone())
        .with_knowledge_graph(graph.clone())
        .build()?;

    let llm = OpenAIProvider::new("gpt-4", api_key)?;

    // 2. Ingest documents
    let documents = vec![
        Document::from_file("docs/getting-started.md")?,
        Document::from_file("docs/api-reference.md")?,
    ];

    for doc in documents {
        rag.ingest(doc, ChunkingStrategy::Semantic).await?;
    }

    // 3. Process user query
    let query = "How do I set up PostgreSQL storage?";

    let context = context_assembler.assemble_context(
        query,
        token_budget: 2000,
        strategies: vec![
            RetrievalStrategy::RAG,
            RetrievalStrategy::Semantic,
        ],
        rerank: true
    ).await?;

    // 4. Generate augmented prompt
    let prompt = format!(
        "Context:\n{}\n\nQuestion: {}\n\nAnswer:",
        context.formatted_text(),
        query
    );

    // 5. Get LLM response
    let response = llm.generate(&prompt, config).await?;

    println!("Response: {}", response);

    // 6. Update knowledge graph with new facts
    graph.extract_and_add_entities(&response).await?;

    Ok(())
}
```

---

## Performance Characteristics

| Component | Operation | Typical Time |
|-----------|-----------|--------------|
| RAG Ingestion | 1 document (1000 words) | 100-200ms |
| Vector Search | Top 10 results | 2-10ms |
| Reranking | 10 candidates | 50-100ms |
| Context Assembly | Full pipeline | 100-300ms |
| Graph Query | Single entity | 1-5ms |
| Graph Traversal | Depth 3 | 10-30ms |

**Optimization Tips**:
1. Use parallel retrieval for multi-strategy queries
2. Cache embeddings for frequently queried documents
3. Batch document ingestion
4. Use appropriate reranking models (smaller = faster)
5. Set reasonable token budgets to limit context size

---

## Related Documentation

- **Detailed API Docs**:
  - [llmspell-rag.md](llmspell-rag.md) - RAG pipeline
  - [llmspell-context.md](llmspell-context.md) - Context engineering
  - [llmspell-graph.md](llmspell-graph.md) - Knowledge graph

- **Other Guides**:
  - [storage-backends.md](storage-backends.md) - Vector storage
  - [memory-backends.md](memory-backends.md) - Memory system
  - [core-traits.md](core-traits.md) - Foundation

- **User Guides**:
  - [../../user-guide/concepts.md](../../user-guide/concepts.md) - RAG concepts
  - [../../user-guide/storage/postgresql-setup.md](../../user-guide/storage/postgresql-setup.md) - Setup

---

**Version**: 0.13.0 | **Phase**: 13b.18.1 | **Last Updated**: 2025-11-08
