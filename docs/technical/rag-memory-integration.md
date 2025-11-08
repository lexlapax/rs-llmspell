# RAG+Memory Integration Architecture

**Status**: Implemented (Phase 13.10)
**Version**: 0.12.0
**Date**: January 2025

## Overview

The RAG+Memory integration combines vector-based document retrieval (RAG) with episodic conversation memory to provide context-aware AI assistants with both factual knowledge and conversational context.

## Architecture

### Component Diagram

```
┌─────────────────────────────────────────────────────────────┐
│                      ContextBridge                           │
│  (llmspell-bridge/src/context_bridge.rs)                    │
│                                                               │
│  ┌────────────────────────────────────────────────────────┐ │
│  │           assemble(query, "rag", budget, session)      │ │
│  └────────────────────────────────────────────────────────┘ │
└────────────────────┬────────────────────────────────────────┘
                     │
                     ▼
┌─────────────────────────────────────────────────────────────┐
│                  HybridRetriever                             │
│  (llmspell-context/src/retrieval/hybrid_rag_memory.rs)     │
│                                                               │
│  ┌──────────────────┐         ┌──────────────────────────┐  │
│  │  RAG Pipeline    │         │    MemoryManager         │  │
│  │  (Documents)     │         │    (Conversations)       │  │
│  └────────┬─────────┘         └─────────┬────────────────┘  │
│           │                             │                    │
│           │  retrieve()                 │  episodic()        │
│           │  k=rag_budget/100           │  search()          │
│           │                             │  k=mem_budget/100  │
│           ▼                             ▼                    │
│  ┌─────────────────┐         ┌──────────────────────────┐   │
│  │  RAG Results    │         │   Memory Results         │   │
│  │  score * 0.4    │         │   score * 0.6            │   │
│  └────────┬────────┘         └─────────┬────────────────┘   │
│           │                             │                    │
│           └──────────┬──────────────────┘                    │
│                      ▼                                        │
│           ┌──────────────────────┐                           │
│           │  Weighted Merge      │                           │
│           │  (40% RAG + 60% Mem) │                           │
│           └──────────┬───────────┘                           │
│                      │                                        │
│                      ▼                                        │
│           ┌──────────────────────┐                           │
│           │   BM25 Reranking     │                           │
│           │   (Unified Scoring)  │                           │
│           └──────────┬───────────┘                           │
│                      │                                        │
│                      ▼                                        │
│           ┌──────────────────────┐                           │
│           │  Context Assembly    │                           │
│           │  (Token Budget)      │                           │
│           └──────────┬───────────┘                           │
└──────────────────────┼─────────────────────────────────────┘
                       │
                       ▼
              ┌────────────────────┐
              │  Assembled Context │
              │  (Ranked Chunks)   │
              └────────────────────┘
```

### Data Flow

1. **Query Initiation**: User calls `Context.assemble("query", "rag", 2000, "session-123")`

2. **Token Budget Allocation**:
   - Total budget: 2000 tokens
   - RAG allocation: 2000 × 0.4 = 800 tokens
   - Memory allocation: 2000 × 0.6 = 1200 tokens
   - Configurable via `RetrievalWeights`

3. **Parallel Retrieval**:
   ```rust
   // RAG Pipeline
   let rag_k = (rag_budget / 100).max(1);  // ~8 results
   let rag_results = rag.retrieve(query, rag_k, scope).await?;

   // Memory Search
   let memory_k = (memory_budget / 100).max(1);  // ~12 results
   let memory_results = memory.episodic().search(query, memory_k).await?;
   ```

4. **Format Conversion**:
   - RAG: `RAGResult` → `RankedChunk` (via `rag_results_to_ranked_chunks`)
   - Memory: `EpisodicEntry` → `RankedChunk` (in `HybridRetriever`)

5. **Weighted Merge**:
   ```rust
   for chunk in &mut rag_chunks {
       chunk.score *= weights.rag_weight;  // *= 0.4
   }
   for chunk in &mut memory_chunks {
       chunk.score *= weights.memory_weight;  // *= 0.6
   }
   let merged = rag_chunks.extend(memory_chunks);
   ```

6. **BM25 Reranking**: Unified relevance scoring across both sources

7. **Token Budget Enforcement**: Truncate to fit within original budget

8. **Assembly**: Format chunks into LLM-ready context

## Key Components

### HybridRetriever

**Location**: `llmspell-context/src/retrieval/hybrid_rag_memory.rs` (380 lines)

**Responsibility**: Orchestrate parallel retrieval from RAG + Memory sources

**Key Methods**:
- `retrieve_hybrid(query, session_id, token_budget)` - Main retrieval entry point
- `allocate_budget(token_budget)` - Split budget across sources
- `query_rag()` - Query RAG pipeline (returns empty if not configured)
- `query_memory()` - Query episodic memory
- `weighted_merge()` - Apply weights and merge results

**Features**:
- Optional RAG pipeline (`Option<Arc<dyn RAGRetriever>>`)
- Optional query pattern tracker (`Option<Arc<QueryPatternTracker>>`)
- Configurable retrieval weights (presets: balanced, rag_focused, memory_focused)

### RetrievalWeights

**Purpose**: Configure source weighting for hybrid merge

**Presets**:
- `balanced()`: 50% RAG + 50% Memory
- `rag_focused()`: 70% RAG + 30% Memory
- `memory_focused()` (default): 40% RAG + 60% Memory

**Validation**: Weights must sum to 1.0 ±0.01

### ContextBridge Enhancement

**Location**: `llmspell-bridge/src/context_bridge.rs`

**New Method**: `with_rag_pipeline(rag: Arc<dyn RAGRetriever>)` - Builder pattern

**Strategy Addition**: `RetrievalStrategy::Rag` - Calls `retrieve_rag_hybrid()`

**Fallback Behavior**: If RAG pipeline is `None`, falls back to "hybrid" strategy

### RAG Adapter

**Location**: `llmspell-context/src/retrieval/rag_adapter.rs` (202 lines)

**Purpose**: Convert `RAGResult` → `RankedChunk`

**Source Attribution**:
- Extracts `source` from RAG metadata (e.g., "rust-docs", "python-guide")
- Falls back to "rag" if no source in metadata
- Memory chunks always use "memory:session-id" format

### Query Pattern Tracker

**Location**: `llmspell-context/src/retrieval/query_pattern_tracker.rs` (270 lines)

**Purpose**: Track retrieval frequency for consolidation priority

**Integration**: Optional builder method `with_query_tracker()` on `HybridRetriever`

**API**: `get_consolidation_candidates(min_retrievals)` - Returns frequently-retrieved entry IDs

## Design Decisions

### 1. Integration in llmspell-context

**Decision**: Place RAG+Memory integration in `llmspell-context`, NOT `llmspell-rag`

**Rationale**:
- Avoids circular dependencies (bridge → rag → bridge)
- Natural fit: context layer already composes retrieval strategies
- `llmspell-rag` remains dependency-free
- Clean layering: rag + memory → context → bridge

### 2. Optional RAG Pipeline

**Decision**: RAG pipeline is `Option<Arc<dyn RAGRetriever>>`

**Benefits**:
- Backward compatible: ContextBridge works without RAG
- Graceful fallback: "rag" strategy falls back to "hybrid" if None
- No hard dependency: systems without RAG still function

### 3. Default Memory-Focused Weighting

**Decision**: Default weights favor memory (40% RAG + 60% Memory)

**Rationale**:
- Recent conversation context often more relevant than documents
- Configurable via presets for different use cases
- Can adjust per-query if needed (future enhancement)

### 4. Session-Aware Filtering

**Decision**: Pass session_id through all layers, filter memory results

**Implementation**:
- RAG: Session encoded in `StateScope::Custom("session:xyz")` (optional)
- Memory: Filtered by `entry.session_id == session` after retrieval
- HybridRetriever: Receives session_id parameter

### 5. Token Budget Allocation

**Decision**: Allocate budget proportionally to weights before retrieval

**Formula**:
```rust
let rag_budget = token_budget * weights.rag_weight;
let memory_budget = token_budget - rag_budget;
let rag_k = (rag_budget / 100).max(1);  // Estimate ~100 tokens/chunk
let memory_k = (memory_budget / 100).max(1);
```

**Note**: Final assembly may use fewer tokens due to BM25 filtering

## Performance Characteristics

### Latency

- **RAG retrieval**: 10-50ms (HNSW vector search)
- **Memory search**: 5-20ms (in-memory vector search)
- **Parallel execution**: max(RAG, Memory) + merge overhead
- **BM25 reranking**: <5ms for 20 chunks
- **Total P95**: <100ms for 2000 token budget

### Scalability

- **RAG corpus**: Scales to millions of documents (HNSW)
- **Memory**: Scales to thousands of entries per session
- **Consolidation**: QueryPatternTracker O(1) record, O(n log n) candidates

### Memory Usage

- **HybridRetriever**: Minimal (no cached state)
- **QueryPatternTracker**: O(unique entries tracked)
- **Merged results**: O(rag_k + memory_k) chunks in memory

## Testing

### Unit Tests

- `llmspell-context/src/retrieval/hybrid_rag_memory.rs`: 10 tests
- `llmspell-context/src/retrieval/query_pattern_tracker.rs`: 7 tests
- **Total**: 17 unit tests

### Integration Tests

- `llmspell-context/tests/query_pattern_integration_test.rs`: 8 tests
- `llmspell-bridge/tests/rag_memory_e2e_test.rs`: 5 E2E tests
- **Total**: 13 integration tests

### E2E Tests

1. `test_rag_memory_hybrid_retrieval` - Basic hybrid retrieval
2. `test_rag_memory_with_query_tracker` - Query pattern tracking
3. `test_rag_memory_session_isolation` - Session filtering
4. `test_rag_memory_without_rag_pipeline` - Graceful fallback
5. `test_rag_memory_token_budget_allocation` - Budget allocation

## Usage Examples

### Rust API

```rust
use llmspell_bridge::ContextBridge;
use llmspell_rag::pipeline::RAGRetriever;
use std::sync::Arc;

// Create context bridge with RAG pipeline
let context = ContextBridge::new(memory_manager.clone())
    .with_rag_pipeline(rag_pipeline.clone());

// Hybrid retrieval
let result = context
    .assemble("Rust ownership", "rag", 2000, Some("session-123"))
    .await?;

println!("Retrieved {} chunks", result["chunks"].as_array().unwrap().len());
```

### Lua API

```lua
-- Hybrid RAG+Memory retrieval
local result = Context.assemble(
    "Explain Rust ownership",
    "rag",
    2000,
    "session-123"
)

-- Analyze sources
for i, chunk in ipairs(result.chunks) do
    local source = chunk.chunk.source
    if source:match("^memory:") then
        print(string.format("[%d] Memory: %s", i, chunk.chunk.content:sub(1, 80)))
    else
        print(string.format("[%d] RAG: %s", i, chunk.chunk.content:sub(1, 80)))
    end
end
```

## Future Enhancements

### Near-Term (Phase 13.11)

1. Template integration: Add `memory_enabled` and `rag_enabled` parameters
2. Per-query weight configuration: Allow runtime weight adjustment
3. Caching: Cache frequent queries for <1ms response

### Long-Term (Phase 14+)

1. Multi-modal RAG: Support images, audio in document corpus
2. Semantic memory integration: Add third source (episodic + semantic + RAG)
3. Adaptive weighting: Learn optimal weights from user feedback
4. Distributed retrieval: Federated search across multiple RAG pipelines

## References

- **Phase 13.10 Design**: `docs/in-progress/phase-13-design-doc.md`
- **Memory Architecture**: `docs/technical/memory-architecture.md`
- **Context Engineering**: `docs/technical/context-engineering.md`
- **RAG Pipeline**: `llmspell-rag/src/pipeline/`
- **API Documentation**: [Lua API Reference](../user-guide/appendix/lua-api-reference.md)

## Change History

- **2025-01-28**: Initial implementation (Phase 13.10.3, 13.10.4, 13.10.5)
- **2025-01-28**: E2E tests and Lua examples added
- **2025-01-28**: Architecture documentation created
