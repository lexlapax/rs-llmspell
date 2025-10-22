# llmspell-memory

Adaptive memory system for LLMSpell with episodic, semantic, and procedural memory.

## Architecture

```
MemoryManager
├── EpisodicMemory (vector search via HNSW/ChromaDB/Qdrant)
├── SemanticMemory (knowledge graph via SurrealDB/Neo4j)
└── ProceduralMemory (pattern storage)
```

## Hot-Swappable Storage Backends

All memory types support multiple storage backends through trait abstraction:

- **HNSW** (default, from llmspell-kernel) - Zero external dependencies
- **ChromaDB** (optional external service) - Python-based vector database
- **Qdrant** (optional external service) - Rust-based vector database
- **InMemory** (testing/development) - Ephemeral storage

## Features

- **Episodic Memory**: Vector-indexed interaction history with semantic search
- **Semantic Memory**: Bi-temporal knowledge graph (event_time + ingestion_time)
- **Procedural Memory**: Learned patterns and state transitions
- **Consolidation**: Automatic conversion of episodic → semantic knowledge
- **Async/Await**: Full async API with tokio
- **Type-Safe**: Comprehensive error handling with thiserror

## Usage

```rust
use llmspell_memory::prelude::*;

// Create memory manager
let memory = DefaultMemoryManager::new_in_memory().await?;

// Add episodic memory
memory.episodic().add(EpisodicEntry {
    session_id: "session-1".into(),
    role: "user".into(),
    content: "What is Rust?".into(),
    timestamp: Utc::now(),
    metadata: json!({}),
}).await?;

// Search episodic memories
let results = memory.episodic().search("Rust", 5).await?;
```

## Development Status

- **Phase 13.1**: Memory Layer Foundation (IN PROGRESS)
  - ✅ Task 13.1.1: Crate structure created
  - ⏳ Task 13.1.2: Core traits (next)
  - ⏳ Task 13.1.3: HNSW episodic memory
  - ⏳ Task 13.1.4: In-memory fallback
  - ⏳ Task 13.1.5: Unit tests

## Performance Targets

- DMR (Diversity-Memory Ratio): >90%
- NDCG@10 (Retrieval Quality): >0.85
- P95 Context Assembly: <100ms
- Consolidation Throughput: >1000 entries/sec

## License

See workspace root LICENSE file.
