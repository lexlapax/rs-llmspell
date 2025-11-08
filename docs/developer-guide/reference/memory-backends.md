# Memory Backends & Adaptive Memory

**Thematic guide to llmspell's memory system architecture**

🔗 **Quick Links**: [llmspell-memory](llmspell-memory.md) | [Crate Index](crate-index.md)

---

## Overview

llmspell's adaptive memory system (Phase 13) provides a 3-tier memory architecture with hot-swappable backends, enabling experimentation with different memory strategies while maintaining production-quality performance.

**Memory Tiers**:
1. **Episodic Memory**: Conversation history with vector search
2. **Semantic Memory**: Knowledge graph for facts and relationships
3. **Procedural Memory**: Learned patterns and skills

**Backend Options**:
- **InMemory**: Development and testing (fast, ephemeral)
- **HNSW**: Production episodic memory (8.47x speedup, persistent)
- **SurrealDB**: Semantic memory graph (bi-temporal, embedded)
- **ChromaDB/Qdrant**: External vector databases (optional)

---

## Memory Architecture

```
Application
    ↓
Memory Facade (MemorySystem)
    ↓
    ├─→ Episodic Memory
    │   ├─→ InMemoryBackend (dev)
    │   ├─→ HNSWBackend (production)
    │   └─→ ChromaDB/Qdrant (external)
    │
    ├─→ Semantic Memory
    │   ├─→ Knowledge Graph (SurrealDB)
    │   └─→ InMemory Graph (testing)
    │
    └─→ Procedural Memory
        └─→ Pattern Storage
```

---

## Memory System API

### Initialization

```rust
use llmspell_memory::{MemorySystem, MemoryConfig, MemoryBackend};

// Development setup (InMemory)
let memory = MemorySystem::new(
    MemoryConfig::default()
        .with_episodic_backend(MemoryBackend::InMemory)
        .with_semantic_backend(MemoryBackend::InMemory)
)?;

// Production setup (HNSW + SurrealDB)
let memory = MemorySystem::new(
    MemoryConfig::default()
        .with_episodic_backend(MemoryBackend::HNSW)
        .with_semantic_backend(MemoryBackend::SurrealDB)
        .with_persistence_path("/var/lib/llmspell/memory")
)?;
```

### Basic Operations

**Add Memories**:
```rust
// Add to episodic memory (conversation turn)
memory.add_episodic(
    session_id: "session_123",
    content: "User asked about PostgreSQL setup",
    metadata: json!({
        "role": "user",
        "timestamp": Utc::now(),
    })
).await?;

memory.add_episodic(
    session_id: "session_123",
    content: "Assistant provided setup instructions",
    metadata: json!({
        "role": "assistant",
        "timestamp": Utc::now(),
    })
).await?;

// Add to semantic memory (fact)
memory.add_semantic(
    "PostgreSQL is a relational database",
    relations: vec![
        ("PostgreSQL", "is_a", "database"),
        ("PostgreSQL", "supports", "ACID transactions"),
    ]
).await?;

// Add to procedural memory (pattern)
memory.add_procedural(
    pattern: "code_review",
    steps: vec![
        "Read code changes",
        "Check for bugs",
        "Suggest improvements",
    ]
).await?;
```

**Query Memories**:
```rust
// Query episodic (semantic search over conversation history)
let results = memory.query_episodic(
    query: "How do I set up PostgreSQL?",
    session_id: Some("session_123"),  // Optional: limit to session
    k: 5  // Top 5 results
).await?;

for result in results {
    println!("Memory: {}\nRelevance: {:.3}", result.content, result.score);
}

// Query semantic (traverse knowledge graph)
let results = memory.query_semantic(
    query: "What databases does llmspell support?",
    max_depth: 3  // Traverse up to 3 relationship hops
).await?;

// Query procedural (find matching patterns)
let pattern = memory.query_procedural("code_review").await?;
```

📚 **Full Details**: [llmspell-memory.md](llmspell-memory.md)

---

## Episodic Memory

**Purpose**: Store and retrieve conversation history with semantic search

### Backend Comparison

| Feature | InMemory | HNSW | ChromaDB | Qdrant |
|---------|----------|------|----------|--------|
| **Speed** | Fastest | Very Fast | Fast | Fast |
| **Persistence** | No | Yes | Yes | Yes |
| **External Service** | No | No | Yes | Yes |
| **Multi-Tenant** | Yes | Yes | Yes | Yes |
| **Cost** | Free | Free | Self-hosted/Cloud | Self-hosted/Cloud |

### HNSW Backend (Recommended for Production)

**Performance**:
- Add operation: <2ms (50x faster than target)
- Search (k=10): 2-10ms
- Memory overhead: Minimal
- 8.47x speedup over basic implementation

**Configuration**:
```rust
use llmspell_memory::backends::HNSWConfig;

let config = HNSWConfig {
    dimension: 1536,              // Embedding dimension
    max_elements: 1_000_000,      // Max memories
    ef_construction: 200,         // Build precision
    ef_search: 50,                // Search precision
    M: 16,                        // Connections per layer
    persistence_path: Some("/var/lib/llmspell/episodic".into()),
    ..Default::default()
};

let backend = MemoryBackend::HNSW(config);
```

**Usage**:
```rust
let memory = MemorySystem::new(
    MemoryConfig::default()
        .with_episodic_backend(MemoryBackend::HNSW)
)?;

// Add memories (automatically embedded and indexed)
memory.add_episodic("session_1", "First conversation turn", metadata).await?;
memory.add_episodic("session_1", "Second conversation turn", metadata).await?;

// Query (vector similarity search)
let results = memory.query_episodic(
    "What did we discuss earlier?",
    session_id: Some("session_1"),
    k: 3
).await?;
```

**Multi-Session Isolation**:
```rust
// Sessions are automatically isolated
memory.add_episodic("session_A", "Content for A", metadata).await?;
memory.add_episodic("session_B", "Content for B", metadata).await?;

// Only returns memories from session_A
let results = memory.query_episodic(
    "query",
    session_id: Some("session_A"),
    k: 5
).await?;
```

### ChromaDB Backend (External Vector DB)

**Use Case**: Shared vector database across multiple instances

**Setup**:
```rust
use llmspell_memory::backends::ChromaDBConfig;

let config = ChromaDBConfig {
    url: "http://localhost:8000".to_string(),
    collection: "llmspell_episodic",
    api_key: Some(env::var("CHROMA_API_KEY")?),
};

let backend = MemoryBackend::ChromaDB(config);
```

**Benefits**:
- Centralized memory store
- Scalable (distributed)
- Multi-instance access
- Built-in persistence

**Trade-offs**:
- Network latency
- External dependency
- Additional infrastructure

### Qdrant Backend

**Use Case**: High-performance vector search at scale

**Setup**:
```rust
use llmspell_memory::backends::QdrantConfig;

let config = QdrantConfig {
    url: "http://localhost:6333".to_string(),
    collection: "llmspell_memories",
    api_key: Some(env::var("QDRANT_API_KEY")?),
};

let backend = MemoryBackend::Qdrant(config);
```

**Benefits**:
- Production-grade performance
- Advanced filtering
- Horizontal scaling
- Cloud-native

---

## Semantic Memory

**Purpose**: Store facts, entities, and relationships in a knowledge graph

### SurrealDB Backend (Recommended)

**Features**:
- Bi-temporal tracking (event time + ingestion time)
- Graph queries and traversal
- Embedded database (no external service)
- SQL-like query language

**Configuration**:
```rust
use llmspell_memory::backends::SurrealDBConfig;

let config = SurrealDBConfig {
    path: "/var/lib/llmspell/semantic".into(),
    namespace: "llmspell",
    database: "semantic_memory",
};

let backend = MemoryBackend::SurrealDB(config);
```

**Usage**:
```rust
let memory = MemorySystem::new(
    MemoryConfig::default()
        .with_semantic_backend(MemoryBackend::SurrealDB)
)?;

// Add fact
memory.add_semantic(
    "llmspell supports PostgreSQL",
    relations: vec![
        ("llmspell", "supports", "PostgreSQL"),
        ("PostgreSQL", "is_a", "database"),
    ]
).await?;

// Query with graph traversal
let results = memory.query_semantic(
    "What databases does llmspell support?",
    max_depth: 2
).await?;

// Result: llmspell -> supports -> PostgreSQL -> is_a -> database
```

**Entity Extraction**:
```rust
// Automatically extract entities from text
let entities = memory.extract_entities(
    "llmspell is written in Rust and supports PostgreSQL"
).await?;

// Entities: [llmspell, Rust, PostgreSQL]
// Relationships: [(llmspell, written_in, Rust), (llmspell, supports, PostgreSQL)]
```

**Temporal Queries**:
```rust
// Query knowledge as of a specific time
let past_state = memory.query_semantic_at_time(
    "What did we know about PostgreSQL?",
    time: DateTime::parse_from_rfc3339("2024-01-01T00:00:00Z")?
).await?;

// Query changes over time
let changes = memory.query_semantic_changes(
    entity: "llmspell",
    start: DateTime::parse_from_rfc3339("2024-01-01T00:00:00Z")?,
    end: DateTime::parse_from_rfc3339("2024-02-01T00:00:00Z")?
).await?;
```

---

## Procedural Memory

**Purpose**: Store learned patterns, skills, and procedures

### Pattern Storage

**Adding Patterns**:
```rust
memory.add_procedural(
    name: "debug_error",
    pattern: ProceduralPattern {
        trigger: "error occurred",
        steps: vec![
            "Read error message",
            "Check logs",
            "Identify root cause",
            "Apply fix",
            "Verify resolution",
        ],
        success_rate: 0.85,
        usage_count: 142,
    }
).await?;
```

**Querying Patterns**:
```rust
// Find pattern by name
let pattern = memory.get_procedural("debug_error").await?;

// Find patterns by trigger
let patterns = memory.find_procedural_by_trigger("error").await?;

// Find most successful patterns
let top_patterns = memory.get_top_procedural_patterns(limit: 10).await?;
```

**Pattern Learning**:
```rust
// Record pattern execution
memory.record_procedural_execution(
    pattern: "debug_error",
    success: true,
    duration: Duration::from_secs(120),
).await?;

// Patterns auto-update success rates
let pattern = memory.get_procedural("debug_error").await?;
println!("Success rate: {:.1}%", pattern.success_rate * 100.0);
```

---

## Memory Consolidation

**Purpose**: LLM-driven knowledge extraction from episodic to semantic memory

### Consolidation Process

```rust
use llmspell_memory::consolidation::{ConsolidationEngine, ConsolidationConfig};

let engine = ConsolidationEngine::new(
    memory.clone(),
    llm_provider,
    ConsolidationConfig {
        min_memories: 10,              // Consolidate after 10 episodic memories
        consolidation_interval: Duration::from_hours(1),
        confidence_threshold: 0.7,     // Only extract high-confidence facts
    }
)?;

// Run consolidation
let results = engine.consolidate_session("session_123").await?;

println!("Extracted {} facts", results.facts_extracted);
println!("Created {} entities", results.entities_created);
println!("Created {} relationships", results.relationships_created);
```

**Manual Consolidation**:
```rust
// Consolidate specific conversation
memory.consolidate_conversation(
    session_id: "session_123",
    start_turn: 10,
    end_turn: 20
).await?;

// Consolidates turns 10-20 into semantic facts
```

**Example**:
```
Episodic Input:
- User: "How do I install PostgreSQL?"
- Assistant: "Use apt install postgresql on Ubuntu"
- User: "What's the default port?"
- Assistant: "PostgreSQL uses port 5432 by default"

Semantic Output (extracted facts):
- (PostgreSQL, install_command, "apt install postgresql")
- (PostgreSQL, platform, "Ubuntu")
- (PostgreSQL, default_port, "5432")
```

---

## Multi-Tenant Memory

### Tenant Isolation

**Pattern**: Prefix collections with tenant ID

```rust
async fn get_tenant_memory(
    base_memory: &MemorySystem,
    tenant_id: &str,
) -> MemorySystem {
    MemorySystem::new(
        MemoryConfig::default()
            .with_collection_prefix(format!("tenant_{}", tenant_id))
            .with_episodic_backend(base_memory.episodic_backend())
            .with_semantic_backend(base_memory.semantic_backend())
    )
}

// Usage
let tenant_a_memory = get_tenant_memory(&memory, "tenant_a").await;
let tenant_b_memory = get_tenant_memory(&memory, "tenant_b").await;

// Completely isolated
tenant_a_memory.add_episodic("session_1", "A's data", metadata).await?;
tenant_b_memory.add_episodic("session_1", "B's data", metadata).await?;
```

### Resource Quotas

```rust
pub struct MemoryQuota {
    max_episodic_memories: usize,
    max_semantic_entities: usize,
    max_procedural_patterns: usize,
}

async fn enforce_quota(
    memory: &MemorySystem,
    tenant_id: &str,
    quota: &MemoryQuota,
) -> Result<()> {
    let stats = memory.get_stats().await?;

    if stats.episodic_count >= quota.max_episodic_memories {
        // Prune oldest memories
        memory.prune_episodic(keep_recent: quota.max_episodic_memories).await?;
    }

    Ok(())
}
```

---

## Performance Optimization

### Memory Pooling

```rust
use std::sync::Arc;
use dashmap::DashMap;

pub struct MemoryPool {
    memories: Arc<DashMap<String, Arc<MemorySystem>>>,
}

impl MemoryPool {
    pub async fn get_or_create(&self, session_id: &str) -> Arc<MemorySystem> {
        if let Some(memory) = self.memories.get(session_id) {
            return memory.clone();
        }

        let memory = Arc::new(MemorySystem::new(config)?);
        self.memories.insert(session_id.to_string(), memory.clone());
        memory
    }
}
```

### Batch Operations

```rust
// Instead of individual adds
for memory in memories {
    system.add_episodic(session_id, memory.content, memory.metadata).await?;
}

// Use batch add
system.batch_add_episodic(
    session_id,
    memories.into_iter().map(|m| (m.content, m.metadata)).collect()
).await?;
```

### Caching

```rust
use lru::LruCache;

pub struct CachedMemory {
    inner: MemorySystem,
    query_cache: Mutex<LruCache<String, Vec<MemoryResult>>>,
}

impl CachedMemory {
    pub async fn query_episodic_cached(
        &self,
        query: &str,
        session_id: Option<&str>,
        k: usize,
    ) -> Result<Vec<MemoryResult>> {
        let cache_key = format!("{}:{}:{}", query, session_id.unwrap_or(""), k);

        // Check cache
        {
            let mut cache = self.query_cache.lock().await;
            if let Some(results) = cache.get(&cache_key) {
                return Ok(results.clone());
            }
        }

        // Query memory
        let results = self.inner.query_episodic(query, session_id, k).await?;

        // Update cache
        {
            let mut cache = self.query_cache.lock().await;
            cache.put(cache_key, results.clone());
        }

        Ok(results)
    }
}
```

---

## Testing Patterns

### Mock Memory System

```rust
use llmspell_testing::mocks::MockMemorySystem;

let mock_memory = MockMemorySystem::new()
    .with_episodic_results(vec![
        MemoryResult {
            content: "Previous conversation about PostgreSQL",
            score: 0.95,
            metadata: json!({}),
        }
    ])
    .with_semantic_results(vec![
        ("PostgreSQL", "is_a", "database"),
    ]);

// Use in tests
let results = mock_memory.query_episodic("PostgreSQL", None, 5).await?;
assert_eq!(results.len(), 1);
```

### Integration Testing

```rust
#[tokio::test]
async fn test_memory_integration() {
    let memory = MemorySystem::new(
        MemoryConfig::default()
            .with_episodic_backend(MemoryBackend::InMemory)
            .with_semantic_backend(MemoryBackend::InMemory)
    )?;

    // Add memories
    memory.add_episodic("session_1", "Test memory", json!({})).await?;

    // Query
    let results = memory.query_episodic("Test", Some("session_1"), 5).await?;
    assert!(!results.is_empty());

    // Cleanup automatic (InMemory backend)
}
```

---

## Related Documentation

- **Detailed API**: [llmspell-memory.md](llmspell-memory.md)
- **Other Guides**:
  - [rag-pipeline.md](rag-pipeline.md) - RAG integration
  - [storage-backends.md](storage-backends.md) - Vector storage
  - [core-traits.md](core-traits.md) - Foundation traits
- **User Guides**:
  - [../../user-guide/concepts.md](../../user-guide/concepts.md) - Memory concepts
  - [../../user-guide/appendix/lua-api-reference.md](../../user-guide/appendix/lua-api-reference.md) - Lua memory API

---

**Version**: 0.13.0 | **Phase**: 13b.18.1 | **Last Updated**: 2025-11-08
