# Phase 13: Adaptive Memory System + Context Engineering - TODO List

**Version**: 1.0
**Date**: January 2025
**Status**: Implementation Ready
**Phase**: 13 (Adaptive Memory System + Context Engineering)
**Timeline**: Weeks 44-48 (25 working days / 5 weeks)
**Priority**: CRITICAL (Core AI Intelligence - 2025's #1 AI Skill)
**Dependencies**:
- Phase 8: Vector Storage (HNSW, embeddings) ✅
- Phase 10: IDE integration for visualization ✅
- Phase 11: Local LLM for consolidation ✅
- Phase 12: Templates ready for memory enhancement ✅

**Arch-Document**: docs/technical/master-architecture-vision.md
**All-Phases-Document**: docs/in-progress/implementation-phases.md
**Design-Document**: docs/in-progress/phase-13-design-doc.md (5,628 lines)
**Memory-Architecture**: docs/technical/memory-architecture.md (To be created)
**Context-Architecture**: docs/technical/context-engineering.md (To be created)
**Current-Architecture**: docs/technical/current-architecture.md (To be update)
**This-document**: working copy /TODO.md (pristine/immutable copy in docs/in-progress/PHASE13-TODO.md)

> **📋 Actionable Task List**: This document breaks down Phase 13 implementation into specific, measurable tasks for building experimental infrastructure with production-quality engineering memory system with temporal knowledge graphs and context engineering pipeline.

---

## Overview

**Goal**: Implement integrated memory architecture (episodic + semantic + procedural) with context engineering pipeline for intelligent retrieval, addressing the "intelligence crisis" where models degrade below 50% accuracy at 32k tokens despite 128k-1M context windows.

**Strategic Context**:
- **Problem**: Context rot at 32k tokens (50% accuracy drop)
- **Solution**: Memory (Zep/Graphiti 94.8% DMR) + Context Engineering (SELF-RAG 320% improvement) + Reranking (DeBERTa NDCG@10 >0.85)
- **Approach**: Bi-temporal knowledge graph + LLM-driven consolidation + hybrid retrieval

**Architecture Summary**:
- **3 New Crates**: llmspell-memory (3,500 LOC), llmspell-graph (2,800 LOC), llmspell-context (4,200 LOC)
- **2 New Globals**: MemoryGlobal (17th), ContextGlobal (18th)
- **19 New CLI Commands**: memory (7), graph (3), context (3)
- **10 Crate Extensions**: 4,000 LOC across kernel, bridge, RAG, templates

**Success Criteria Summary**:
- [ ] 3 new crates compile without warnings
- [ ] 2 new globals functional (MemoryGlobal 17th, ContextGlobal 18th)
- [ ] 19 CLI commands operational
- [ ] DMR benchmark >90% on 100-interaction test set
- [ ] NDCG@10 >0.85 on reranking benchmark
- [ ] Bi-temporal queries functional (event_time + ingestion_time)
- [ ] LLM consolidation: >85% ADD/UPDATE precision, <10% missed entities
- [ ] Hybrid retrieval >20% DMR improvement over vector-only
- [ ] All 10 templates support enable_memory opt-in parameter
- [ ] Zero breaking changes (Phase 12 code works unchanged)
- [ ] Context assembly P95 <100ms
- [ ] Consolidation daemon <5% CPU overhead
- [ ] Graph supports 100k+ entities, 1M+ relationships
- [ ] >90% test coverage, >95% API documentation coverage
- [ ] Zero clippy warnings

---

## Dependency Analysis

**Critical Path**:
1. **Foundation (Days 1-5)**: Memory + Graph crates → Integration
2. **Pipeline (Days 6-10)**: Context crate + Consolidation → E2E flow
3. **Integration (Days 11-15)**: Kernel + Bridge → Lua API
4. **Features (Days 16-20)**: RAG + Templates → CLI
5. **Validation (Days 21-25)**: Performance + Accuracy → Release

**Parallel Tracks**:
- **Memory Track**: Days 1-2 (llmspell-memory) → Days 11-12 (kernel integration)
- **Graph Track**: Days 3-4 (llmspell-graph) → Days 16-17 (RAG integration)
- **Context Track**: Days 6-7 (llmspell-context) → Days 18-19 (template integration)
- **Consolidation Track**: Days 8-9 (consolidation logic) → Days 21-22 (performance optimization)
- **Bridge Track**: Days 13-14 (globals) → Day 15 (Lua API validation)
- **CLI Track**: Day 20 (commands) → Days 23-24 (accuracy validation)

**Hard Dependencies**:
- Phase 13.2 (Graph) depends on Phase 13.1 (Memory) for MemoryManager trait
- Phase 13.5 (Consolidation) depends on Phases 13.1-13.2 (Memory + Graph)
- Phase 13.7 (Kernel) depends on Phases 13.1-13.4 (all core crates)
- Phase 13.8 (Bridge) depends on Phase 13.7 (kernel integration)
- Phase 13.10 (RAG) depends on Phases 13.1-13.4 (memory + context)
- Phase 13.11 (Templates) depends on Phase 13.10 (RAG integration)
- Phase 13.13-13.14 (Optimization/Validation) depend on all previous phases

---

## Phase 13.1: Memory Layer Foundation (Days 1-2)

**Goal**: Create llmspell-memory crate with episodic memory and vector indexing
**Timeline**: 2 days (16 hours)
**Critical Dependencies**: Phase 8 (Vector Storage) ✅

### Task 13.1.1: Create llmspell-memory Crate Structure
**Priority**: CRITICAL
**Estimated Time**: 2 hours
**Assignee**: Memory Team Lead

**Description**: Create the new `llmspell-memory` crate with proper dependencies and module structure for episodic, semantic, and procedural memory.

**Acceptance Criteria**:
- [ ] Crate directory created at `/llmspell-memory`
- [ ] `Cargo.toml` configured with all dependencies
- [ ] Basic module structure in `src/lib.rs`
- [ ] Crate added to workspace members
- [ ] `cargo check -p llmspell-memory` passes

**Implementation Steps**:
1. Create `llmspell-memory/` directory structure
2. Configure `Cargo.toml` with dependencies:
   - `llmspell-core`, `llmspell-utils`, `llmspell-rag`
   - `llmspell-state-persistence`, `llmspell-sessions`
   - `chroma-rs = "0.2"` or `qdrant-client = "1.0"`
   - `tokio`, `async-trait`, `serde`, `serde_json`, `chrono`
3. Create module structure in `src/lib.rs`:
   ```rust
   pub mod traits;
   pub mod episodic;
   pub mod semantic;
   pub mod procedural;
   pub mod consolidation;
   pub mod manager;
   pub mod types;
   pub mod error;
   pub mod prelude;
   ```
4. Add to workspace in root `Cargo.toml`
5. Run `cargo check -p llmspell-memory`

**Files to Create**:
- `llmspell-memory/Cargo.toml`
- `llmspell-memory/src/lib.rs`
- `llmspell-memory/src/traits.rs` (empty)
- `llmspell-memory/src/episodic.rs` (empty)
- `llmspell-memory/src/semantic.rs` (empty)
- `llmspell-memory/src/procedural.rs` (empty)
- `llmspell-memory/src/consolidation.rs` (empty)
- `llmspell-memory/src/manager.rs` (empty)
- `llmspell-memory/src/types.rs` (empty)
- `llmspell-memory/src/error.rs` (empty)
- `llmspell-memory/src/prelude.rs` (empty)
- `llmspell-memory/README.md`

**Definition of Done**:
- [ ] Crate compiles without errors
- [ ] All module files created (can be empty stubs)
- [ ] Dependencies resolve correctly
- [ ] No clippy warnings with `cargo clippy -p llmspell-memory`

### Task 13.1.2: Define Core Memory Traits
**Priority**: CRITICAL
**Estimated Time**: 3 hours
**Assignee**: Memory Team

**Description**: Implement the trait hierarchy for memory management with episodic, semantic, and procedural memory types.

**Acceptance Criteria**:
- [ ] `MemoryManager` trait with episodic/semantic/procedural access
- [ ] `EpisodicMemory` trait with vector search
- [ ] `SemanticMemory` trait (placeholder for graph integration)
- [ ] `ProceduralMemory` trait (placeholder for pattern storage)
- [ ] `ConsolidationEngine` trait with ADD/UPDATE/DELETE/NOOP decisions
- [ ] Trait tests compile and pass

**Implementation Steps**:
1. Create `src/traits/memory_manager.rs`:
   ```rust
   #[async_trait]
   pub trait MemoryManager: Send + Sync {
       async fn episodic(&self) -> &dyn EpisodicMemory;
       async fn semantic(&self) -> &dyn SemanticMemory;
       async fn procedural(&self) -> &dyn ProceduralMemory;
       async fn consolidate(&self, session_id: &str, mode: ConsolidationMode) -> Result<ConsolidationResult>;
   }
   ```
2. Create `src/traits/episodic.rs`:
   ```rust
   #[async_trait]
   pub trait EpisodicMemory: Send + Sync {
       async fn add(&self, entry: EpisodicEntry) -> Result<String>;
       async fn get(&self, id: &str) -> Result<EpisodicEntry>;
       async fn search(&self, query: &str, top_k: usize) -> Result<Vec<EpisodicEntry>>;
       async fn list_unprocessed(&self, session_id: &str) -> Result<Vec<EpisodicEntry>>;
   }
   ```
3. Create `src/traits/semantic.rs` (placeholder for Phase 13.2)
4. Create `src/traits/procedural.rs` (placeholder for future)
5. Create `src/traits/consolidation.rs`:
   ```rust
   pub enum ConsolidationDecision {
       Add(Entity),
       Update { entity_id: String, changes: HashMap<String, Value> },
       Delete { entity_id: String },
       Noop,
   }
   ```
6. Create `src/types.rs` with `EpisodicEntry`, `ConsolidationMode`, etc.
7. Write basic trait tests in `tests/traits_test.rs`

**Files to Create/Modify**:
- `llmspell-memory/src/traits/memory_manager.rs` (NEW)
- `llmspell-memory/src/traits/episodic.rs` (NEW)
- `llmspell-memory/src/traits/semantic.rs` (NEW - placeholder)
- `llmspell-memory/src/traits/procedural.rs` (NEW - placeholder)
- `llmspell-memory/src/traits/consolidation.rs` (NEW)
- `llmspell-memory/src/traits/mod.rs` (NEW)
- `llmspell-memory/src/types.rs` (MODIFY)
- `llmspell-memory/tests/traits_test.rs` (NEW)

**Definition of Done**:
- [ ] All traits compile without errors
- [ ] Trait object safety verified (`dyn MemoryManager` works)
- [ ] Basic trait tests pass
- [ ] Documentation comments complete (>95% coverage)

### Task 13.1.3: Implement EpisodicMemory with ChromaDB/Qdrant
**Priority**: CRITICAL
**Estimated Time**: 5 hours
**Assignee**: Storage Team Lead

**Description**: Implement episodic memory with vector indexing using ChromaDB or Qdrant for semantic search.

**Acceptance Criteria**:
- [ ] `ChromaDBEpisodicMemory` struct implements `EpisodicMemory` trait
- [ ] Vector embeddings generated for content
- [ ] Semantic search returns relevant results
- [ ] Session isolation working
- [ ] <50ms P95 search latency for 10,000 entries

**Implementation Steps**:
1. Create `src/episodic/chroma.rs`:
   ```rust
   pub struct ChromaDBEpisodicMemory {
       client: ChromaClient,
       collection_name: String,
       embedding_provider: Arc<dyn EmbeddingProvider>,
       session_filter: bool,
   }

   impl ChromaDBEpisodicMemory {
       pub async fn new(config: ChromaConfig) -> Result<Self> { ... }
       pub async fn new_in_memory() -> Result<Self> { ... }
   }
   ```
2. Implement `EpisodicMemory` trait methods:
   - `add()`: Generate embedding, store in ChromaDB with metadata
   - `get()`: Retrieve by ID
   - `search()`: Vector similarity search with session filter
   - `list_unprocessed()`: Filter by processed=false
3. Handle session isolation via metadata filtering
4. Add temporal metadata (timestamp, event_time, ingestion_time)
5. Write unit tests with mock embeddings
6. Benchmark performance with 10K entries

**Files to Create/Modify**:
- `llmspell-memory/src/episodic/chroma.rs` (NEW - 300 lines)
- `llmspell-memory/src/episodic/mod.rs` (NEW)
- `llmspell-memory/src/episodic.rs` (MODIFY - re-export)
- `llmspell-memory/tests/episodic_test.rs` (NEW - 200 lines)
- `llmspell-memory/benches/episodic_bench.rs` (NEW - 100 lines)

**Definition of Done**:
- [ ] All trait methods implemented and tested
- [ ] Unit tests pass with >90% coverage
- [ ] Performance benchmark <50ms P95
- [ ] Memory usage <500 bytes per entry (excluding embedding)
- [ ] Session isolation verified

### Task 13.1.4: Implement In-Memory Fallback for Testing
**Priority**: HIGH
**Estimated Time**: 2 hours
**Assignee**: Storage Team

**Description**: Create in-memory episodic storage for testing and development without external dependencies.

**Acceptance Criteria**:
- [ ] `InMemoryEpisodicMemory` struct implements `EpisodicMemory` trait
- [ ] Similarity search using cosine distance
- [ ] Thread-safe with Arc<RwLock>
- [ ] Used in all unit tests

**Implementation Steps**:
1. Create `src/episodic/in_memory.rs`:
   ```rust
   pub struct InMemoryEpisodicMemory {
       entries: Arc<RwLock<HashMap<String, EpisodicEntry>>>,
       embeddings: Arc<RwLock<Vec<(String, Vec<f32>)>>>,
   }
   ```
2. Implement vector similarity using cosine distance
3. Implement all `EpisodicMemory` trait methods
4. Add to test utilities
5. Write unit tests

**Files to Create/Modify**:
- `llmspell-memory/src/episodic/in_memory.rs` (NEW - 200 lines)
- `llmspell-memory/src/episodic/mod.rs` (MODIFY - add in_memory module)
- `llmspell-memory/tests/in_memory_test.rs` (NEW - 150 lines)

**Definition of Done**:
- [ ] In-memory implementation complete
- [ ] All tests use in-memory storage by default
- [ ] ChromaDB tests behind feature flag
- [ ] Performance acceptable for testing (<10ms search)

### Task 13.1.5: Create Unit Tests for Episodic Memory
**Priority**: HIGH
**Estimated Time**: 4 hours
**Assignee**: QA Team

**Description**: Comprehensive unit tests for episodic memory functionality.

**Acceptance Criteria**:
- [ ] 30+ unit tests covering all scenarios
- [ ] Test coverage >90% for episodic module
- [ ] All edge cases covered (empty results, large batches, etc.)
- [ ] Tests run in CI without external dependencies

**Implementation Steps**:
1. Test episodic store/retrieve
2. Test semantic search accuracy
3. Test session isolation
4. Test unprocessed filtering
5. Test temporal metadata
6. Test error handling
7. Test concurrent access
8. Create `llmspell-testing` helpers for memory tests

**Files to Create/Modify**:
- `llmspell-memory/tests/episodic_store_retrieve_test.rs` (NEW)
- `llmspell-memory/tests/episodic_search_test.rs` (NEW)
- `llmspell-memory/tests/episodic_session_test.rs` (NEW)
- `llmspell-memory/tests/episodic_concurrency_test.rs` (NEW)
- `llmspell-testing/src/memory.rs` (NEW - test helpers)

**Definition of Done**:
- [ ] 30+ tests passing
- [ ] Coverage >90%
- [ ] No flaky tests
- [ ] CI integration complete

---

## Phase 13.2: Temporal Knowledge Graph (Days 3-4)

**Goal**: Create llmspell-graph crate with bi-temporal knowledge graph storage
**Timeline**: 2 days (16 hours)
**Critical Dependencies**: Phase 13.1 (Memory traits for integration)

### Task 13.2.1: Create llmspell-graph Crate Structure
**Priority**: CRITICAL
**Estimated Time**: 2 hours
**Assignee**: Graph Team Lead

**Description**: Create the new `llmspell-graph` crate with bi-temporal graph storage capabilities.

**Acceptance Criteria**:
- [ ] Crate directory created at `/llmspell-graph`
- [ ] `Cargo.toml` configured with all dependencies
- [ ] Basic module structure in `src/lib.rs`
- [ ] Crate added to workspace members
- [ ] `cargo check -p llmspell-graph` passes

**Implementation Steps**:
1. Create `llmspell-graph/` directory structure
2. Configure `Cargo.toml` with dependencies:
   - `llmspell-core`, `llmspell-utils`
   - `surrealdb = "2.0"` or `neo4rs = "0.7"` (choose based on research)
   - `tokio`, `async-trait`, `serde`, `serde_json`, `chrono`
3. Create module structure in `src/lib.rs`:
   ```rust
   pub mod traits;
   pub mod storage;
   pub mod entities;
   pub mod relationships;
   pub mod temporal;
   pub mod extraction;
   pub mod query;
   pub mod types;
   pub mod error;
   pub mod prelude;
   ```
4. Add to workspace in root `Cargo.toml`
5. Run `cargo check -p llmspell-graph`

**Files to Create**:
- `llmspell-graph/Cargo.toml`
- `llmspell-graph/src/lib.rs`
- `llmspell-graph/src/traits.rs` (empty)
- `llmspell-graph/src/storage.rs` (empty)
- `llmspell-graph/src/entities.rs` (empty)
- `llmspell-graph/src/relationships.rs` (empty)
- `llmspell-graph/src/temporal.rs` (empty)
- `llmspell-graph/src/extraction.rs` (empty)
- `llmspell-graph/src/query.rs` (empty)
- `llmspell-graph/src/types.rs` (empty)
- `llmspell-graph/src/error.rs` (empty)
- `llmspell-graph/src/prelude.rs` (empty)
- `llmspell-graph/README.md`

**Definition of Done**:
- [ ] Crate compiles without errors
- [ ] All module files created
- [ ] Dependencies resolve correctly
- [ ] No clippy warnings

### Task 13.2.2: Define Bi-Temporal Graph Traits
**Priority**: CRITICAL
**Estimated Time**: 3 hours
**Assignee**: Graph Team

**Description**: Implement trait hierarchy for bi-temporal knowledge graph with event_time and ingestion_time support.

**Acceptance Criteria**:
- [ ] `KnowledgeGraph` trait with bi-temporal queries
- [ ] `Entity` and `Relationship` types with temporal fields
- [ ] `TemporalQuery` for point-in-time queries
- [ ] Trait tests compile and pass

**Implementation Steps**:
1. Create `src/traits/knowledge_graph.rs`:
   ```rust
   #[async_trait]
   pub trait KnowledgeGraph: Send + Sync {
       async fn add_entity(&self, entity: Entity) -> Result<String>;
       async fn update_entity(&self, id: &str, changes: HashMap<String, Value>) -> Result<()>;
       async fn get_entity(&self, id: &str) -> Result<Entity>;
       async fn get_entity_at(&self, id: &str, event_time: DateTime<Utc>) -> Result<Entity>;
       async fn add_relationship(&self, rel: Relationship) -> Result<String>;
       async fn get_related(&self, entity_id: &str, rel_type: &str) -> Result<Vec<Entity>>;
       async fn query_temporal(&self, query: TemporalQuery) -> Result<Vec<Entity>>;
   }
   ```
2. Create `src/types.rs` with bi-temporal types:
   ```rust
   pub struct Entity {
       pub id: String,
       pub name: String,
       pub entity_type: String,
       pub properties: serde_json::Value,
       pub event_time: Option<DateTime<Utc>>,  // When event occurred
       pub ingestion_time: DateTime<Utc>,      // When we learned it
   }

   pub struct Relationship {
       pub source_id: String,
       pub target_id: String,
       pub relationship_type: String,
       pub properties: serde_json::Value,
       pub event_time: Option<DateTime<Utc>>,
       pub ingestion_time: DateTime<Utc>,
   }
   ```
3. Write trait tests

**Files to Create/Modify**:
- `llmspell-graph/src/traits/knowledge_graph.rs` (NEW - 150 lines)
- `llmspell-graph/src/traits/mod.rs` (NEW)
- `llmspell-graph/src/types.rs` (MODIFY - 200 lines)
- `llmspell-graph/tests/traits_test.rs` (NEW - 100 lines)

**Definition of Done**:
- [ ] All traits compile without errors
- [ ] Bi-temporal semantics clear
- [ ] Trait object safety verified
- [ ] Documentation complete

### Task 13.2.3: Implement SurrealDB Graph Storage
**Priority**: CRITICAL
**Estimated Time**: 6 hours
**Assignee**: Graph Storage Team

**Description**: Implement knowledge graph storage using SurrealDB with bi-temporal support.

**Acceptance Criteria**:
- [ ] `SurrealDBKnowledgeGraph` struct implements `KnowledgeGraph` trait
- [ ] Bi-temporal queries work (event_time + ingestion_time)
- [ ] Entity and relationship storage functional
- [ ] <20ms P95 entity retrieval, <50ms traversal

**Implementation Steps**:
1. Create `src/storage/surrealdb.rs`:
   ```rust
   pub struct SurrealDBKnowledgeGraph {
       db: Surreal<Client>,
       namespace: String,
       database: String,
   }

   impl SurrealDBKnowledgeGraph {
       pub async fn new(config: SurrealDBConfig) -> Result<Self> { ... }
       pub async fn connect(url: &str) -> Result<Self> { ... }
   }
   ```
2. Implement schema with bi-temporal fields:
   ```sql
   DEFINE TABLE entities SCHEMAFULL;
   DEFINE FIELD name ON entities TYPE string;
   DEFINE FIELD entity_type ON entities TYPE string;
   DEFINE FIELD properties ON entities TYPE object;
   DEFINE FIELD event_time ON entities TYPE datetime;
   DEFINE FIELD ingestion_time ON entities TYPE datetime;
   DEFINE INDEX idx_entity_name ON entities COLUMNS name;
   ```
3. Implement all `KnowledgeGraph` trait methods
4. Add temporal query support
5. Write unit tests
6. Benchmark performance

**Files to Create/Modify**:
- `llmspell-graph/src/storage/surrealdb.rs` (NEW - 400 lines)
- `llmspell-graph/src/storage/mod.rs` (NEW)
- `llmspell-graph/tests/surrealdb_test.rs` (NEW - 250 lines)
- `llmspell-graph/benches/graph_bench.rs` (NEW - 150 lines)

**Definition of Done**:
- [ ] All trait methods implemented
- [ ] Unit tests pass with >90% coverage
- [ ] Performance benchmarks meet targets (<20ms entity, <50ms traversal)
- [ ] Bi-temporal queries tested

### Task 13.2.4: Implement Entity/Relationship Extraction (Regex v1)
**Priority**: HIGH
**Estimated Time**: 4 hours
**Assignee**: Extraction Team

**Description**: Implement basic entity and relationship extraction using regex patterns (LLM-based v2 in Phase 13.5).

**Acceptance Criteria**:
- [ ] Extract common entity types (Person, Place, Organization, Concept)
- [ ] Extract relationships (is_a, has_feature, located_in, etc.)
- [ ] Pattern-based extraction working
- [ ] >50% recall on simple text

**Implementation Steps**:
1. Create `src/extraction/regex.rs`:
   ```rust
   pub struct RegexExtractor {
       entity_patterns: Vec<Regex>,
       relationship_patterns: Vec<Regex>,
   }

   impl RegexExtractor {
       pub fn new() -> Self { ... }
       pub fn extract_entities(&self, text: &str) -> Vec<Entity> { ... }
       pub fn extract_relationships(&self, text: &str) -> Vec<Relationship> { ... }
   }
   ```
2. Define regex patterns for common entities:
   - Capitalized phrases for proper nouns
   - Common programming terms (languages, frameworks)
   - Technical concepts
3. Define relationship patterns:
   - "X is a Y" → (X, is_a, Y)
   - "X has Y" → (X, has_feature, Y)
4. Test on sample texts
5. Measure recall

**Files to Create/Modify**:
- `llmspell-graph/src/extraction/regex.rs` (NEW - 300 lines)
- `llmspell-graph/src/extraction/mod.rs` (NEW)
- `llmspell-graph/tests/extraction_test.rs` (NEW - 200 lines)

**Definition of Done**:
- [ ] Extraction working on test texts
- [ ] >50% recall measured
- [ ] Tests cover common patterns
- [ ] Performance <5ms for 1KB text

### Task 13.2.5: Create Unit Tests for Knowledge Graph
**Priority**: HIGH
**Estimated Time**: 4 hours
**Assignee**: QA Team

**Description**: Comprehensive unit tests for knowledge graph functionality.

**Acceptance Criteria**:
- [ ] 20+ unit tests covering all scenarios
- [ ] Test coverage >90% for graph module
- [ ] Bi-temporal query tests
- [ ] Entity resolution tests

**Implementation Steps**:
1. Test bi-temporal entity retrieval
2. Test relationship traversal
3. Test temporal queries (get_entity_at)
4. Test entity deduplication
5. Test concurrent access
6. Test error handling

**Files to Create/Modify**:
- `llmspell-graph/tests/bitemporal_test.rs` (NEW)
- `llmspell-graph/tests/traversal_test.rs` (NEW)
- `llmspell-graph/tests/temporal_query_test.rs` (NEW)
- `llmspell-graph/tests/concurrency_test.rs` (NEW)

**Definition of Done**:
- [ ] 20+ tests passing
- [ ] Coverage >90%
- [ ] No flaky tests
- [ ] CI integration complete

---

## Phase 13.3: Memory + Graph Integration (Day 5)

**Goal**: Integrate MemoryManager with KnowledgeGraph for consolidation
**Timeline**: 1 day (8 hours)
**Critical Dependencies**: Phases 13.1-13.2 (Memory + Graph crates)

### Task 13.3.1: Integrate MemoryManager with KnowledgeGraph
**Priority**: CRITICAL
**Estimated Time**: 3 hours
**Assignee**: Integration Team Lead

**Description**: Create integrated MemoryManager that coordinates episodic and semantic memory.

**Acceptance Criteria**:
- [ ] `DefaultMemoryManager` struct coordinates episodic + semantic
- [ ] Episodic → Semantic consolidation path working
- [ ] Thread-safe access to both memory types
- [ ] Integration tests pass

**Implementation Steps**:
1. Create `llmspell-memory/src/manager.rs`:
   ```rust
   pub struct DefaultMemoryManager {
       episodic: Arc<dyn EpisodicMemory>,
       semantic: Arc<dyn SemanticMemory>,
       procedural: Arc<dyn ProceduralMemory>,
       consolidation_engine: Arc<dyn ConsolidationEngine>,
   }

   impl DefaultMemoryManager {
       pub async fn new(config: MemoryConfig) -> Result<Self> { ... }
       pub async fn new_in_memory() -> Result<Self> { ... }
   }
   ```
2. Implement `MemoryManager` trait
3. Wire episodic memory to knowledge graph for consolidation
4. Add builder pattern for configuration
5. Write integration tests

**Files to Create/Modify**:
- `llmspell-memory/src/manager.rs` (MODIFY - 300 lines)
- `llmspell-memory/src/semantic.rs` (MODIFY - integrate with llmspell-graph)
- `llmspell-memory/Cargo.toml` (MODIFY - add llmspell-graph dependency)
- `llmspell-memory/tests/integration_test.rs` (NEW - 200 lines)

**Definition of Done**:
- [ ] MemoryManager coordinates both memory types
- [ ] Integration tests pass
- [ ] Builder pattern working
- [ ] Documentation complete

### Task 13.3.2: Implement Consolidation Engine Stub
**Priority**: HIGH
**Estimated Time**: 2 hours
**Assignee**: Consolidation Team

**Description**: Create consolidation engine stub with manual trigger (LLM-driven logic in Phase 13.5).

**Acceptance Criteria**:
- [ ] `ManualConsolidationEngine` struct with trigger method
- [ ] Basic episodic → semantic conversion
- [ ] Consolidation metadata tracked
- [ ] Manual trigger working

**Implementation Steps**:
1. Create `llmspell-memory/src/consolidation/manual.rs`:
   ```rust
   pub struct ManualConsolidationEngine {
       extractor: Arc<RegexExtractor>,
       knowledge_graph: Arc<dyn KnowledgeGraph>,
   }

   impl ManualConsolidationEngine {
       pub async fn trigger(&self, session_id: &str, entries: Vec<EpisodicEntry>) -> Result<ConsolidationResult> {
           // Extract entities/relationships using regex
           // Add to knowledge graph
           // Mark entries as processed
       }
   }
   ```
2. Implement basic extraction → graph storage flow
3. Track consolidation metadata (timestamp, entries processed, entities added)
4. Write unit tests

**Files to Create/Modify**:
- `llmspell-memory/src/consolidation/manual.rs` (NEW - 200 lines)
- `llmspell-memory/src/consolidation/mod.rs` (NEW)
- `llmspell-memory/tests/consolidation_test.rs` (NEW - 150 lines)

**Definition of Done**:
- [ ] Manual consolidation working
- [ ] Entities extracted and stored
- [ ] Metadata tracking functional
- [ ] Tests pass

### Task 13.3.3: Create ADR Documentation for Bi-Temporal Design
**Priority**: MEDIUM
**Estimated Time**: 2 hours
**Assignee**: Architecture Team

**Description**: Document architecture decision record for bi-temporal knowledge graph design.

**Acceptance Criteria**:
- [ ] ADR document created
- [ ] Rationale explained (temporal reasoning)
- [ ] Trade-offs documented (+20% storage, +10ms latency)
- [ ] Examples included

**Implementation Steps**:
1. Update `docs/technical/architecture-decisions.md` with ADR for bitemporal-knowledge-graph:
   - Decision: Use bi-temporal design (event_time + ingestion_time)
   - Rationale: Enables "what did we know when?" queries
   - Alternatives: Single timestamp, versioned entities
   - Trade-offs: +20% storage overhead, +10ms query latency
2. Add examples of temporal queries
3. Link from design doc
4. Review with team

**Files to Create**:
- `docs/technical/architecture-decisions.md`  (update - 400 lines)

**Definition of Done**:
- [ ] ADR complete and reviewed
- [ ] Examples clear
- [ ] Trade-offs documented
- [ ] Linked from design doc

### Task 13.3.4: Integration Tests for Episodic → Semantic Flow
**Priority**: HIGH
**Estimated Time**: 1 hour
**Assignee**: QA Team

**Description**: End-to-end integration tests for memory consolidation flow.

**Acceptance Criteria**:
- [ ] Test episodic → consolidation → semantic flow
- [ ] Verify entities extracted correctly
- [ ] Verify relationships created
- [ ] Test marks entries as processed

**Implementation Steps**:
1. Create test scenario:
   - Add episodic entries about "Rust programming language"
   - Trigger consolidation
   - Verify knowledge graph contains Rust entity
   - Verify relationships (e.g., Rust has_feature ownership)
2. Test multiple consolidation runs
3. Test session isolation
4. Test error handling

**Files to Create/Modify**:
- `llmspell-memory/tests/episodic_to_semantic_test.rs` (NEW - 250 lines)

**Definition of Done**:
- [ ] E2E tests passing
- [ ] Flow verified end-to-end
- [ ] Error cases tested
- [ ] CI integration complete

---

## Phase 13.4: Context Engineering Pipeline (Days 6-7)

**Goal**: Create llmspell-context crate with query understanding and reranking
**Timeline**: 2 days (16 hours)
**Critical Dependencies**: Phases 13.1-13.3 (Memory + Graph for retrieval)

### Task 13.4.1: Create llmspell-context Crate Structure
**Priority**: CRITICAL
**Estimated Time**: 2 hours
**Assignee**: Context Team Lead

**Description**: Create the new `llmspell-context` crate for context engineering pipeline.

**Acceptance Criteria**:
- [ ] Crate directory created at `/llmspell-context`
- [ ] `Cargo.toml` configured with all dependencies
- [ ] Basic module structure in `src/lib.rs`
- [ ] Crate added to workspace members
- [ ] `cargo check -p llmspell-context` passes

**Implementation Steps**:
1. Create `llmspell-context/` directory structure
2. Configure `Cargo.toml` with dependencies:
   - `llmspell-core`, `llmspell-utils`, `llmspell-memory`, `llmspell-graph`
   - `candle-core`, `candle-transformers` (for DeBERTa)
   - `tokenizers` (for BM25)
   - `tokio`, `async-trait`, `serde`, `serde_json`
3. Create module structure in `src/lib.rs`:
   ```rust
   pub mod traits;
   pub mod query;
   pub mod retrieval;
   pub mod reranking;
   pub mod assembly;
   pub mod pipeline;
   pub mod types;
   pub mod error;
   pub mod prelude;
   ```
4. Add to workspace in root `Cargo.toml`
5. Run `cargo check -p llmspell-context`

**Files to Create**:
- `llmspell-context/Cargo.toml`
- `llmspell-context/src/lib.rs`
- `llmspell-context/src/traits.rs` (empty)
- `llmspell-context/src/query.rs` (empty)
- `llmspell-context/src/retrieval.rs` (empty)
- `llmspell-context/src/reranking.rs` (empty)
- `llmspell-context/src/assembly.rs` (empty)
- `llmspell-context/src/pipeline.rs` (empty)
- `llmspell-context/src/types.rs` (empty)
- `llmspell-context/src/error.rs` (empty)
- `llmspell-context/src/prelude.rs` (empty)
- `llmspell-context/README.md`

**Definition of Done**:
- [ ] Crate compiles without errors
- [ ] All module files created
- [ ] Dependencies resolve correctly
- [ ] No clippy warnings

### Task 13.4.2: Implement Query Understanding
**Priority**: CRITICAL
**Estimated Time**: 4 hours
**Assignee**: NLP Team

**Description**: Implement query understanding with intent classification and entity extraction.

**Acceptance Criteria**:
- [ ] Intent classification (HowTo, WhatIs, Debug, etc.)
- [ ] Entity extraction from queries
- [ ] Keyword detection
- [ ] >85% classification accuracy on test queries

**Implementation Steps**:
1. Create `src/query/understanding.rs`:
   ```rust
   pub struct QueryUnderstanding {
       pub intent: QueryIntent,
       pub entities: Vec<String>,
       pub keywords: Vec<String>,
   }

   pub enum QueryIntent {
       HowTo,
       WhatIs,
       WhyDoes,
       Debug,
       Explain,
       Unknown,
   }

   pub struct QueryAnalyzer {
       intent_patterns: Vec<(Regex, QueryIntent)>,
       entity_extractor: EntityExtractor,
   }

   impl QueryAnalyzer {
       pub fn understand(&self, query: &str) -> QueryUnderstanding { ... }
   }
   ```
2. Define intent patterns:
   - "How do I..." → HowTo
   - "What is..." → WhatIs
   - "Why does..." → WhyDoes
3. Extract entities (capitalized phrases, technical terms)
4. Extract keywords (important terms, filter stop words)
5. Test on sample queries
6. Measure accuracy

**Files to Create/Modify**:
- `llmspell-context/src/query/understanding.rs` (NEW - 300 lines)
- `llmspell-context/src/query/mod.rs` (NEW)
- `llmspell-context/tests/query_test.rs` (NEW - 200 lines)

**Definition of Done**:
- [ ] Intent classification working
- [ ] Entity extraction functional
- [ ] >85% accuracy on test set
- [ ] Tests pass

### Task 13.4.3: Implement Retrieval Strategy Selection
**Priority**: HIGH
**Estimated Time**: 3 hours
**Assignee**: Retrieval Team

**Description**: Implement retrieval strategy selection based on query understanding.

**Acceptance Criteria**:
- [ ] Strategy selection (episodic vs semantic vs hybrid)
- [ ] Query-based routing working
- [ ] Configurable fallback strategies
- [ ] Tests pass

**Implementation Steps**:
1. Create `src/retrieval/strategy.rs`:
   ```rust
   pub enum RetrievalStrategy {
       Episodic,      // Recent interactions
       Semantic,      // Knowledge graph
       Hybrid,        // Both
       BM25,          // Keyword fallback
   }

   pub struct StrategySelector {
       rules: Vec<SelectionRule>,
   }

   impl StrategySelector {
       pub fn select(&self, understanding: &QueryUnderstanding) -> Vec<RetrievalStrategy> {
           // Select based on intent and entities
       }
   }
   ```
2. Define selection rules:
   - HowTo + temporal keywords → Episodic
   - WhatIs + entities → Semantic
   - Complex queries → Hybrid
3. Implement fallback chain
4. Write unit tests

**Files to Create/Modify**:
- `llmspell-context/src/retrieval/strategy.rs` (NEW - 250 lines)
- `llmspell-context/src/retrieval/mod.rs` (NEW)
- `llmspell-context/tests/strategy_test.rs` (NEW - 150 lines)

**Definition of Done**:
- [ ] Strategy selection working
- [ ] Rules tested
- [ ] Fallback chain functional
- [ ] Tests pass

### Task 13.4.4: Implement DeBERTa Reranking (Candle)
**Priority**: CRITICAL
**Estimated Time**: 6 hours
**Assignee**: ML Team Lead

**Description**: Implement cross-encoder reranking using DeBERTa via Candle framework.

**Acceptance Criteria**:
- [ ] DeBERTa model loading from HuggingFace
- [ ] Cross-encoder scoring for chunk pairs
- [ ] NDCG@10 >0.85 on benchmark
- [ ] P95 latency <30ms for 20 chunks

**Implementation Steps**:
1. Create `src/reranking/deberta.rs`:
   ```rust
   pub struct DeBERTaReranker {
       model: ModelWrapper,
       tokenizer: Tokenizer,
       device: Device,
   }

   impl DeBERTaReranker {
       pub async fn new() -> Result<Self> {
           // Load model from HuggingFace
           // Use Candle for inference
       }

       pub async fn rerank(&self, chunks: Vec<Chunk>, query: &str, top_k: usize) -> Result<Vec<Chunk>> {
           // Score each (query, chunk) pair
           // Sort by score
           // Return top_k
       }
   }
   ```
2. Download DeBERTa model (Provence or MS MARCO fine-tuned)
3. Implement cross-encoder scoring
4. Add batch processing for efficiency
5. Write benchmarks
6. Test accuracy on benchmark dataset

**Files to Create/Modify**:
- `llmspell-context/src/reranking/deberta.rs` (NEW - 400 lines)
- `llmspell-context/src/reranking/mod.rs` (NEW)
- `llmspell-context/benches/rerank_bench.rs` (NEW - 150 lines)
- `llmspell-context/tests/deberta_test.rs` (NEW - 200 lines)

**Definition of Done**:
- [ ] DeBERTa reranking working
- [ ] NDCG@10 >0.85
- [ ] Latency <30ms P95
- [ ] Tests pass

### Task 13.4.5: Implement BM25 Fallback Reranking
**Priority**: HIGH
**Estimated Time**: 3 hours
**Assignee**: Retrieval Team

**Description**: Implement BM25 lexical reranking as fallback when DeBERTa unavailable or too slow.

**Acceptance Criteria**:
- [ ] BM25 scoring implementation
- [ ] Keyword extraction working
- [ ] <5ms P95 latency for 20 chunks
- [ ] Automatic fallback from DeBERTa

**Implementation Steps**:
1. Create `src/reranking/bm25.rs`:
   ```rust
   pub struct BM25Reranker {
       tokenizer: SimpleTokenizer,
       k1: f32,  // BM25 parameter
       b: f32,   // BM25 parameter
   }

   impl BM25Reranker {
       pub fn new() -> Self { ... }

       pub fn rerank(&self, chunks: Vec<Chunk>, query: &str, top_k: usize) -> Vec<Chunk> {
           // Compute BM25 scores
           // Sort by score
           // Return top_k
       }
   }
   ```
2. Implement BM25 algorithm
3. Add simple tokenization (lowercase, split on whitespace, remove stop words)
4. Test accuracy vs DeBERTa
5. Benchmark performance

**Files to Create/Modify**:
- `llmspell-context/src/reranking/bm25.rs` (NEW - 250 lines)
- `llmspell-context/src/reranking/mod.rs` (MODIFY - add bm25 module)
- `llmspell-context/tests/bm25_test.rs` (NEW - 150 lines)

**Definition of Done**:
- [ ] BM25 reranking working
- [ ] Latency <5ms P95
- [ ] Automatic fallback functional
- [ ] Tests pass

### Task 13.4.6: Create Unit Tests for Context Pipeline
**Priority**: HIGH
**Estimated Time**: 4 hours
**Assignee**: QA Team

**Description**: Comprehensive unit tests for context engineering pipeline.

**Acceptance Criteria**:
- [ ] 25+ unit tests covering all components
- [ ] Test coverage >90% for context module
- [ ] Query understanding tests
- [ ] Reranking accuracy tests

**Implementation Steps**:
1. Test query understanding (intent, entities, keywords)
2. Test strategy selection
3. Test DeBERTa reranking accuracy
4. Test BM25 reranking performance
5. Test context assembly
6. Test error handling

**Files to Create/Modify**:
- `llmspell-context/tests/query_understanding_test.rs` (NEW)
- `llmspell-context/tests/strategy_selection_test.rs` (NEW)
- `llmspell-context/tests/reranking_accuracy_test.rs` (NEW)
- `llmspell-context/tests/context_assembly_test.rs` (NEW)

**Definition of Done**:
- [ ] 25+ tests passing
- [ ] Coverage >90%
- [ ] Accuracy tests included
- [ ] CI integration complete

---

## Phase 13.5: LLM-Driven Consolidation (Days 8-9)

**Goal**: Implement LLM-driven consolidation with ADD/UPDATE/DELETE/NOOP decisions
**Timeline**: 2 days (16 hours)
**Critical Dependencies**: Phases 13.1-13.4 (Memory + Graph + Context)

### Task 13.5.1: Implement LLM Consolidation Prompt Templates

**Priority**: CRITICAL
**Estimated Time**: 3 hours
**Assignee**: Memory Team

**Description**: Create prompt templates for ADD/UPDATE/DELETE/NOOP decision-making using LLM consolidation (Mem0 architecture).

**Acceptance Criteria**:
- [ ] ConsolidationPrompt trait defined for prompt template abstraction
- [ ] Four decision prompts (ADD, UPDATE, DELETE, NOOP) implemented
- [ ] Prompt includes: current semantic graph context, new episodic memory, existing entities/relationships
- [ ] Structured output format (JSON) for LLM decisions

**Implementation Steps**:
1. Create `llmspell-memory/src/consolidation/prompts.rs`
2. Define ConsolidationPrompt trait:
   ```rust
   pub trait ConsolidationPrompt {
       fn format_decision_prompt(&self, episodic: &EpisodicRecord, semantic_context: &GraphContext) -> String;
       fn parse_decision(&self, llm_response: &str) -> Result<ConsolidationDecision>;
   }
   ```
3. Implement AddEntityPrompt, UpdateEntityPrompt, DeleteEntityPrompt, NoopPrompt
4. Add JSON schema validation for structured output
5. Create prompt template tests

**Files to Create/Modify**:
- `llmspell-memory/src/consolidation/prompts.rs` (NEW - 400 lines)
- `llmspell-memory/src/consolidation/mod.rs` (MODIFY - add prompts module)
- `llmspell-memory/tests/consolidation/prompt_test.rs` (NEW - 200 lines)

**Definition of Done**:
- [ ] All four decision prompts implemented and tested
- [ ] Prompt templates include entity extraction and relationship inference
- [ ] JSON schema validation passes for all decision types
- [ ] Unit tests verify prompt generation and parsing

### Task 13.5.2: Implement ADD/UPDATE/DELETE Decision Logic

**Priority**: CRITICAL
**Estimated Time**: 4 hours
**Assignee**: Memory Team

**Description**: Implement the core consolidation decision engine that uses LLM to decide how to integrate episodic memories into semantic graph.

**Acceptance Criteria**:
- [ ] ConsolidationEngine trait implementation using LLM provider
- [ ] ADD logic: create new entities/relationships
- [ ] UPDATE logic: merge new facts into existing nodes
- [ ] DELETE logic: remove outdated/contradictory information
- [ ] NOOP logic: skip irrelevant episodic records
- [ ] Batch processing support (consolidate multiple records)

**Implementation Steps**:
1. Create `llmspell-memory/src/consolidation/engine.rs`
2. Implement LLMConsolidationEngine:
   ```rust
   pub struct LLMConsolidationEngine {
       llm_provider: Arc<dyn Provider>,
       knowledge_graph: Arc<dyn KnowledgeGraph>,
       prompts: ConsolidationPrompts,
   }
   impl ConsolidationEngine for LLMConsolidationEngine {
       async fn consolidate(&self, episodic: EpisodicRecord) -> Result<ConsolidationDecision>;
       async fn consolidate_batch(&self, records: Vec<EpisodicRecord>) -> Result<Vec<ConsolidationDecision>>;
   }
   ```
3. Implement decision execution (apply ADD/UPDATE/DELETE to knowledge graph)
4. Add decision logging and audit trail
5. Create consolidation tests with mock LLM responses

**Files to Create/Modify**:
- `llmspell-memory/src/consolidation/engine.rs` (NEW - 600 lines)
- `llmspell-memory/src/consolidation/decisions.rs` (NEW - 200 lines)
- `llmspell-memory/src/consolidation/mod.rs` (MODIFY - add engine and decisions)
- `llmspell-memory/tests/consolidation/engine_test.rs` (NEW - 400 lines)

**Definition of Done**:
- [ ] All four decision types (ADD/UPDATE/DELETE/NOOP) functional
- [ ] Batch consolidation supports >100 records per call
- [ ] Audit trail logs all consolidation decisions
- [ ] Tests verify decision logic with mock LLM responses

### Task 13.5.3: Implement Background Consolidation Daemon

**Priority**: HIGH
**Estimated Time**: 3 hours
**Assignee**: Memory Team

**Description**: Create a background daemon that periodically consolidates episodic memories into semantic graph.

**Acceptance Criteria**:
- [ ] ConsolidationDaemon spawns background tokio task
- [ ] Configurable interval (default: 5 minutes)
- [ ] Graceful shutdown support
- [ ] Error handling and retry logic
- [ ] Metrics emission (records processed, decisions made)

**Implementation Steps**:
1. Create `llmspell-memory/src/consolidation/daemon.rs`
2. Implement ConsolidationDaemon:
   ```rust
   pub struct ConsolidationDaemon {
       engine: Arc<LLMConsolidationEngine>,
       interval: Duration,
       running: Arc<AtomicBool>,
   }
   impl ConsolidationDaemon {
       pub async fn start(&self) -> Result<JoinHandle<()>>;
       pub async fn stop(&self) -> Result<()>;
   }
   ```
3. Add interval-based processing loop (select! with shutdown signal)
4. Implement error recovery (exponential backoff)
5. Add daemon tests with controlled intervals

**Files to Create/Modify**:
- `llmspell-memory/src/consolidation/daemon.rs` (NEW - 300 lines)
- `llmspell-memory/src/consolidation/mod.rs` (MODIFY - add daemon module)
- `llmspell-memory/tests/consolidation/daemon_test.rs` (NEW - 200 lines)

**Definition of Done**:
- [ ] Daemon starts and runs in background successfully
- [ ] Graceful shutdown completes all in-flight consolidations
- [ ] Interval configuration tested (1s, 5m, 1h)
- [ ] Error recovery verified with simulated failures

### Task 13.5.4: Add Consolidation Metrics and Monitoring

**Priority**: MEDIUM
**Estimated Time**: 2 hours
**Assignee**: Memory Team

**Description**: Add metrics collection for consolidation performance and decision tracking.

**Acceptance Criteria**:
- [ ] Metrics for: total_records_processed, decisions_by_type (ADD/UPDATE/DELETE/NOOP), consolidation_duration_ms, errors_total
- [ ] DMR (Decision Match Rate) tracking
- [ ] Integration with existing llmspell-core metrics system
- [ ] Exportable to Prometheus/JSON

**Implementation Steps**:
1. Create `llmspell-memory/src/consolidation/metrics.rs`
2. Define ConsolidationMetrics struct with counters
3. Add metrics emission in ConsolidationEngine and Daemon
4. Implement DMR calculation (compare LLM decisions vs ground truth)
5. Add metrics serialization for export

**Files to Create/Modify**:
- `llmspell-memory/src/consolidation/metrics.rs` (NEW - 250 lines)
- `llmspell-memory/src/consolidation/engine.rs` (MODIFY - add metrics emission)
- `llmspell-memory/src/consolidation/daemon.rs` (MODIFY - add metrics emission)
- `llmspell-memory/tests/consolidation/metrics_test.rs` (NEW - 150 lines)

**Definition of Done**:
- [ ] All key metrics tracked and exportable
- [ ] DMR calculation verified with test data
- [ ] Metrics integration with llmspell-core confirmed
- [ ] Prometheus export format validated

### Task 13.5.5: E2E Consolidation Test with Real LLM

**Priority**: HIGH
**Estimated Time**: 4 hours
**Assignee**: QA + Memory Team

**Description**: Create end-to-end test with real LLM (Ollama) for consolidation validation.

**Acceptance Criteria**:
- [ ] Test uses real Ollama model for consolidation decisions
- [ ] Verifies: episodic → LLM decision → graph update
- [ ] Measures: DMR baseline, decision distribution (ADD/UPDATE/DELETE/NOOP)
- [ ] Test runs in <2 minutes with small model

**Implementation Steps**:
1. Create `llmspell-memory/tests/e2e/consolidation_llm_test.rs`
2. Setup test with Ollama (llama3.2:3b or similar small model)
3. Create test scenario:
   - Add episodic: "Rust is a systems programming language"
   - Add episodic: "Rust has memory safety without garbage collection"
   - Trigger consolidation with LLM
   - Verify entities: Rust, memory safety, garbage collection
   - Verify relationships: Rust has_feature memory_safety
4. Measure decision distribution
5. Calculate baseline DMR (if ground truth available)

**Files to Create/Modify**:
- `llmspell-memory/tests/e2e/consolidation_llm_test.rs` (NEW - 350 lines)
- `llmspell-memory/tests/e2e/mod.rs` (MODIFY - add consolidation_llm_test)

**Definition of Done**:
- [ ] E2E test passes with real LLM
- [ ] Entities and relationships created correctly
- [ ] Decision distribution measured (expect ~40% ADD, ~30% UPDATE, ~20% NOOP, ~10% DELETE)
- [ ] Test runs in CI with Ollama available

---

## Phase 13.6: E2E Memory Flow & Documentation (Day 10)

**Goal**: Validate complete memory lifecycle and document consolidation algorithm
**Timeline**: 1 day (8 hours)
**Critical Dependencies**: Phases 13.1-13.5 complete

### Task 13.6.1: E2E Integration Test (Episodic → Consolidation → Semantic → Retrieval)

**Priority**: CRITICAL
**Estimated Time**: 3 hours
**Assignee**: QA + Memory Team

**Description**: Create comprehensive end-to-end test covering full memory lifecycle.

**Acceptance Criteria**:
- [ ] Test scenario: Add episodic memories → Trigger consolidation → Query semantic graph → Retrieve via context assembly
- [ ] Verifies: EpisodicMemory, ConsolidationEngine, KnowledgeGraph, ContextPipeline integration
- [ ] Uses real ChromaDB/Qdrant (embedded), SurrealDB (in-memory), DeBERTa (Candle)
- [ ] Assertions on: entities created, relationships formed, retrieval accuracy

**Implementation Steps**:
1. Create `llmspell-memory/tests/e2e/memory_flow_test.rs`
2. Setup test scenario with sample conversation:
   ```rust
   // User: "Rust is a systems programming language"
   // User: "Rust has memory safety without garbage collection"
   // User: "What are Rust's key features?"
   ```
3. Add episodic memories to ChromaDB
4. Trigger consolidation (immediate mode)
5. Verify semantic graph has entities (Rust, memory safety, garbage collection) and relationships
6. Query context assembly with "Rust features"
7. Assert retrieved context includes both episodic records

**Files to Create/Modify**:
- `llmspell-memory/tests/e2e/memory_flow_test.rs` (NEW - 400 lines)
- `llmspell-memory/tests/e2e/mod.rs` (MODIFY - add memory_flow_test)
- `llmspell-memory/Cargo.toml` (MODIFY - add dev-dependencies for embedded DBs)

**Definition of Done**:
- [ ] E2E test passes with all assertions green
- [ ] Test runs in <30 seconds with embedded DBs
- [ ] Code coverage includes all major memory components
- [ ] Test documented with architecture diagram

### Task 13.6.2: DMR and NDCG@10 Baseline Measurement

**Priority**: HIGH
**Estimated Time**: 2 hours
**Assignee**: Memory Team

**Description**: Establish performance baselines for consolidation accuracy (DMR) and retrieval quality (NDCG@10).

**Acceptance Criteria**:
- [ ] DMR baseline measured on test dataset (target: >70% before LLM tuning)
- [ ] NDCG@10 baseline measured (target: >0.70 with DeBERTa reranking)
- [ ] Baseline metrics documented in design doc
- [ ] Comparison script for future benchmarking

**Implementation Steps**:
1. Create `llmspell-memory/benches/baseline_measurement.rs`
2. Load test dataset (20 conversations, 200 episodic records)
3. Run consolidation on all records
4. Calculate DMR: (correct_decisions / total_decisions)
5. Run retrieval queries (10 test queries)
6. Calculate NDCG@10 for each query
7. Export baseline metrics to JSON

**Files to Create/Modify**:
- `llmspell-memory/benches/baseline_measurement.rs` (NEW - 300 lines)
- `llmspell-memory/benches/data/test_dataset.json` (NEW - 500 lines)
- `llmspell-memory/benches/baseline_results.json` (NEW - generated)
- `docs/in-progress/phase-13-design-doc.md` (MODIFY - add baseline results section)

**Definition of Done**:
- [ ] DMR baseline ≥70% achieved
- [ ] NDCG@10 baseline ≥0.70 achieved
- [ ] Baseline results documented in design doc
- [ ] Benchmarking script reusable for Phase 13.14

### Task 13.6.3: Consolidation Algorithm Documentation

**Priority**: MEDIUM
**Estimated Time**: 3 hours
**Assignee**: Documentation Team

**Description**: Document the consolidation algorithm design, decision logic, and LLM prompt engineering.

**Acceptance Criteria**:
- [ ] ADR (Architecture Decision Record) created for consolidation approach
- [ ] Prompt templates documented with examples
- [ ] Decision flow diagram (episodic → analysis → decision → graph update)
- [ ] Performance characteristics documented (latency, throughput)

**Implementation Steps**:
1. Update `docs/technical/architecture-decisions.md` with ADR for llm-consolidation.md
2. Document: problem statement, decision drivers, options considered, chosen solution, consequences
3. Add consolidation flow diagram (Mermaid)
4. Document prompt templates with example inputs/outputs
5. Add performance analysis (P50, P95, P99 latencies)

**Files to Create/Modify**:
- Update `docs/technical/architecture-decisions.md` (NEW - 600 lines)
- `docs/technical/README.md` (MODIFY - add ADR-013 to index)
- `llmspell-memory/README.md` (MODIFY - add consolidation section)

**Definition of Done**:
- [ ] ADR-013 complete with all sections
- [ ] Consolidation flow diagram clear and accurate
- [ ] Prompt engineering guidelines documented
- [ ] Performance characteristics measured and documented

---

## Phase 13.7: Kernel Integration (Days 11-12)

**Goal**: Integrate MemoryManager into llmspell-kernel with lifecycle management
**Timeline**: 2 days (16 hours)
**Critical Dependencies**: Phases 13.1-13.6 complete

### Task 13.7.1: Add MemoryManager to Kernel Context

**Priority**: CRITICAL
**Estimated Time**: 3 hours
**Assignee**: Kernel Team

**Description**: Integrate MemoryManager into llmspell-kernel as optional infrastructure.

**Acceptance Criteria**:
- [ ] MemoryManager added to KernelContext as optional component
- [ ] Lifecycle management (init, shutdown) integrated
- [ ] Configuration loading from runtime config
- [ ] Backward compatibility maintained (memory opt-in)

**Implementation Steps**:
1. Modify `llmspell-kernel/src/context.rs`:
   ```rust
   pub struct KernelContext {
       // ... existing fields
       pub memory_manager: Option<Arc<dyn MemoryManager>>,
   }
   ```
2. Update `KernelContextBuilder`:
   ```rust
   impl KernelContextBuilder {
       pub fn with_memory_manager(mut self, manager: Arc<dyn MemoryManager>) -> Self {
           self.memory_manager = Some(manager);
           self
       }
   }
   ```
3. Add memory_manager initialization in kernel startup
4. Add graceful shutdown for consolidation daemon
5. Write integration tests

**Files to Create/Modify**:
- `llmspell-kernel/src/context.rs` (MODIFY - add memory_manager field, ~50 lines)
- `llmspell-kernel/src/builder.rs` (MODIFY - add with_memory_manager, ~30 lines)
- `llmspell-kernel/src/lifecycle.rs` (MODIFY - add memory shutdown, ~40 lines)
- `llmspell-kernel/tests/memory_integration_test.rs` (NEW - 200 lines)

**Definition of Done**:
- [ ] MemoryManager successfully added to KernelContext
- [ ] Tests verify memory manager lifecycle (init, use, shutdown)
- [ ] Backward compatibility verified (kernel works without memory)
- [ ] Configuration loading tested

### Task 13.7.2: Add ConsolidationDaemon Lifecycle Management

**Priority**: HIGH
**Estimated Time**: 3 hours
**Assignee**: Kernel Team

**Description**: Start/stop consolidation daemon as part of kernel lifecycle.

**Acceptance Criteria**:
- [ ] ConsolidationDaemon starts when MemoryManager present in config
- [ ] Daemon shutdown gracefully on kernel shutdown
- [ ] Configuration: enable_daemon, consolidation_interval, batch_size
- [ ] Error handling for daemon failures

**Implementation Steps**:
1. Modify `llmspell-kernel/src/lifecycle.rs`
2. Add daemon startup in kernel_init():
   ```rust
   if let Some(memory_mgr) = &context.memory_manager {
       if config.memory.enable_daemon {
           let daemon = ConsolidationDaemon::new(memory_mgr.consolidation_engine(), config.memory.interval);
           context.consolidation_daemon = Some(daemon.start().await?);
       }
   }
   ```
3. Add daemon shutdown in kernel_shutdown()
4. Add daemon health monitoring
5. Create lifecycle tests

**Files to Create/Modify**:
- `llmspell-kernel/src/lifecycle.rs` (MODIFY - add daemon lifecycle, ~100 lines)
- `llmspell-kernel/src/context.rs` (MODIFY - add consolidation_daemon field, ~20 lines)
- `llmspell-config/src/memory_config.rs` (NEW - 100 lines)
- `llmspell-kernel/tests/daemon_lifecycle_test.rs` (NEW - 250 lines)

**Definition of Done**:
- [ ] Daemon starts automatically when enabled in config
- [ ] Daemon stops gracefully on kernel shutdown
- [ ] Configuration options tested (interval, batch_size)
- [ ] Error recovery verified (daemon crash → restart)

### Task 13.7.3: Session-Memory Linking

**Priority**: HIGH
**Estimated Time**: 4 hours
**Assignee**: Sessions Team + Memory Team

**Description**: Link SessionManager with MemoryManager for automatic episodic memory creation.

**Acceptance Criteria**:
- [ ] SessionInteraction events trigger episodic memory creation
- [ ] Session metadata (session_id, user_id, timestamp) included in episodic records
- [ ] Opt-in design: session.with_memory(true) to enable
- [ ] Session artifacts linked to episodic memories

**Implementation Steps**:
1. Modify `llmspell-kernel/src/sessions/session.rs`:
   ```rust
   impl Session {
       pub async fn add_interaction(&mut self, interaction: Interaction) -> Result<()> {
           // Existing artifact storage
           self.artifacts.add(interaction.clone()).await?;

           // New: Memory integration (opt-in)
           if self.config.enable_memory {
               if let Some(memory_mgr) = &self.context.memory_manager {
                   memory_mgr.episodic().add(EpisodicRecord {
                       session_id: self.id.clone(),
                       role: interaction.role,
                       content: interaction.content,
                       timestamp: Utc::now(),
                       metadata: interaction.metadata,
                   }).await?;
               }
           }
           Ok(())
       }
   }
   ```
2. Add session_id index to episodic memory
3. Create session-memory integration tests

**Files to Create/Modify**:
- `llmspell-kernel/src/sessions/session.rs` (MODIFY - add memory hook, ~60 lines)
- `llmspell-kernel/src/sessions/config.rs` (MODIFY - add enable_memory flag, ~10 lines)
- `llmspell-memory/src/episodic/chroma.rs` (MODIFY - add session_id index, ~30 lines)
- `llmspell-kernel/tests/session_memory_integration_test.rs` (NEW - 300 lines)

**Definition of Done**:
- [ ] Session interactions automatically create episodic memories when enabled
- [ ] Session metadata correctly propagated to episodic records
- [ ] Opt-in design verified (sessions work without memory)
- [ ] Integration tests pass with real ChromaDB

### Task 13.7.4: State-Memory Synchronization

**Priority**: MEDIUM
**Estimated Time**: 3 hours
**Assignee**: Kernel Team

**Description**: Synchronize StateManager with MemoryManager for procedural memory (learned patterns).

**Acceptance Criteria**:
- [ ] State changes tracked as potential procedural memories
- [ ] Pattern detection: repeated state transitions → procedural memory
- [ ] State snapshots linked to episodic/semantic memory for context
- [ ] Opt-in design via configuration

**Implementation Steps**:
1. Create `llmspell-memory/src/procedural/state_tracker.rs`:
   ```rust
   pub struct StateChangeTracker {
       state_manager: Arc<StateManager>,
       memory_manager: Arc<dyn MemoryManager>,
   }
   impl StateChangeTracker {
       pub async fn track_change(&self, key: &str, old_value: Value, new_value: Value) -> Result<()>;
       pub async fn detect_patterns(&self) -> Result<Vec<ProceduralPattern>>;
   }
   ```
2. Hook into StateManager.set() for change tracking
3. Implement pattern detection (frequent transitions)
4. Create state-memory tests

**Files to Create/Modify**:
- `llmspell-memory/src/procedural/state_tracker.rs` (NEW - 350 lines)
- `llmspell-memory/src/procedural/mod.rs` (MODIFY - add state_tracker module)
- `llmspell-kernel/src/state/manager.rs` (MODIFY - add memory hook, ~40 lines)
- `llmspell-kernel/tests/state_memory_integration_test.rs` (NEW - 250 lines)

**Definition of Done**:
- [ ] State changes tracked when memory enabled
- [ ] Pattern detection identifies repeated transitions
- [ ] Integration with StateManager verified
- [ ] Opt-in design tested (state works without memory)

### Task 13.7.5: Kernel Integration Tests

**Priority**: HIGH
**Estimated Time**: 3 hours
**Assignee**: QA

**Description**: Comprehensive kernel integration tests for memory system.

**Acceptance Criteria**:
- [ ] Test kernel startup with memory enabled/disabled
- [ ] Test daemon lifecycle (start, run, stop)
- [ ] Test session-memory integration
- [ ] Test state-memory synchronization

**Implementation Steps**:
1. Create test: Kernel with memory enabled
2. Create test: Kernel without memory (backward compatibility)
3. Create test: Daemon lifecycle with graceful shutdown
4. Create test: Session creates episodic memories
5. Create test: State changes tracked

**Files to Create/Modify**:
- `llmspell-kernel/tests/integration/memory_enabled_test.rs` (NEW - 300 lines)
- `llmspell-kernel/tests/integration/memory_disabled_test.rs` (NEW - 200 lines)
- `llmspell-kernel/tests/integration/daemon_lifecycle_test.rs` (NEW - 250 lines)

**Definition of Done**:
- [ ] All integration tests pass
- [ ] Test coverage >90% for kernel memory integration
- [ ] Backward compatibility verified
- [ ] CI integration complete

---

## Phase 13.8: Bridge + Globals (Days 13-14)

**Goal**: Expose memory and context APIs to script engines via bridges and globals
**Timeline**: 2 days (16 hours)
**Critical Dependencies**: Phase 13.7 complete

### Task 13.8.1: Create MemoryBridge

**Priority**: CRITICAL
**Estimated Time**: 3 hours
**Assignee**: Bridge Team

**Description**: Create MemoryBridge to expose MemoryManager functionality to script engines.

**Acceptance Criteria**:
- [ ] MemoryBridge wraps MemoryManager with script-friendly API
- [ ] Methods: episodic_add, episodic_search, semantic_query, consolidate, get_stats
- [ ] Async methods converted to blocking for script compatibility
- [ ] Error handling with user-friendly messages

**Implementation Steps**:
1. Create `llmspell-bridge/src/memory_bridge.rs`:
   ```rust
   pub struct MemoryBridge {
       memory_manager: Arc<dyn MemoryManager>,
       runtime: tokio::runtime::Handle,
   }
   impl MemoryBridge {
       pub fn episodic_add(&self, session_id: String, role: String, content: String, metadata: Value) -> Result<()>;
       pub fn episodic_search(&self, query: String, top_k: usize, filters: Value) -> Result<Vec<Value>>;
       pub fn semantic_query(&self, query: String) -> Result<Value>;
       pub fn consolidate(&self, mode: String, session_id: Option<String>) -> Result<Value>;
       pub fn get_stats(&self, session_id: Option<String>) -> Result<Value>;
   }
   ```
2. Add async→blocking conversion using runtime.block_on()
3. Add JSON serialization for script consumption
4. Create bridge tests

**Files to Create/Modify**:
- `llmspell-bridge/src/memory_bridge.rs` (NEW - 500 lines)
- `llmspell-bridge/src/lib.rs` (MODIFY - add memory_bridge module)
- `llmspell-bridge/tests/memory_bridge_test.rs` (NEW - 300 lines)

**Definition of Done**:
- [ ] MemoryBridge compiles and all methods functional
- [ ] Async conversion tested with tokio runtime
- [ ] JSON serialization verified for all return types
- [ ] Error handling tested with invalid inputs

### Task 13.8.2: Create ContextBridge

**Priority**: CRITICAL
**Estimated Time**: 3 hours
**Assignee**: Bridge Team

**Description**: Create ContextBridge to expose ContextPipeline functionality to scripts.

**Acceptance Criteria**:
- [ ] ContextBridge wraps ContextPipeline with script API
- [ ] Methods: assemble, test_query, get_strategy_stats, configure_reranking
- [ ] Token budget enforcement
- [ ] Strategy override support

**Implementation Steps**:
1. Create `llmspell-bridge/src/context_bridge.rs`:
   ```rust
   pub struct ContextBridge {
       context_pipeline: Arc<ContextPipeline>,
       runtime: tokio::runtime::Handle,
   }
   impl ContextBridge {
       pub fn assemble(&self, query: String, max_tokens: usize, strategies: Vec<String>, session_id: Option<String>) -> Result<Value>;
       pub fn test_query(&self, query: String, session_id: Option<String>) -> Result<Value>;
       pub fn get_strategy_stats(&self) -> Result<Value>;
       pub fn configure_reranking(&self, model: String, top_k: usize) -> Result<()>;
   }
   ```
2. Add strategy validation (episodic, semantic, hybrid)
3. Add token budget calculations
4. Create bridge tests

**Files to Create/Modify**:
- `llmspell-bridge/src/context_bridge.rs` (NEW - 450 lines)
- `llmspell-bridge/src/lib.rs` (MODIFY - add context_bridge module)
- `llmspell-bridge/tests/context_bridge_test.rs` (NEW - 300 lines)

**Definition of Done**:
- [ ] ContextBridge compiles and all methods functional
- [ ] Strategy validation tested (valid and invalid strategies)
- [ ] Token budget enforcement verified
- [ ] Reranking configuration tested

### Task 13.8.3: Create MemoryGlobal (17th Global)

**Priority**: CRITICAL
**Estimated Time**: 4 hours
**Assignee**: Bridge Team

**Description**: Create MemoryGlobal exposing Memory namespace to Lua/JS scripts.

**Acceptance Criteria**:
- [ ] MemoryGlobal implements GlobalObject trait
- [ ] Lua API: Memory.episodic.add(), Memory.episodic.search(), Memory.semantic.query(), Memory.consolidate(), Memory.stats()
- [ ] All methods tested in Lua
- [ ] Documentation with examples

**Implementation Steps**:
1. Create `llmspell-bridge/src/globals/memory_global.rs`:
   ```rust
   pub struct MemoryGlobal {
       bridge: Arc<MemoryBridge>,
   }
   impl GlobalObject for MemoryGlobal {
       fn name(&self) -> &str { "Memory" }
       fn inject_lua(&self, lua: &Lua) -> Result<()> {
           let memory_table = lua.create_table()?;

           // Memory.episodic.add(...)
           let episodic_table = lua.create_table()?;
           episodic_table.set("add", lua.create_function(...)?)?;

           memory_table.set("episodic", episodic_table)?;
           lua.globals().set("Memory", memory_table)?;
           Ok(())
       }
   }
   ```
2. Add mlua type conversions
3. Register MemoryGlobal in create_standard_registry()
4. Create Lua integration tests

**Files to Create/Modify**:
- `llmspell-bridge/src/globals/memory_global.rs` (NEW - 600 lines)
- `llmspell-bridge/src/globals/mod.rs` (MODIFY - add memory_global module, export)
- `llmspell-bridge/src/globals/registry.rs` (MODIFY - register MemoryGlobal, ~10 lines)
- `llmspell-bridge/tests/lua/memory_global_test.rs` (NEW - 400 lines)

**Definition of Done**:
- [ ] MemoryGlobal registered as 17th global
- [ ] All Lua API methods functional and tested
- [ ] Documentation generated from Rust docs
- [ ] Examples added to user guide

### Task 13.8.4: Create ContextGlobal (18th Global)

**Priority**: CRITICAL
**Estimated Time**: 3 hours
**Assignee**: Bridge Team

**Description**: Create ContextGlobal exposing Context namespace to Lua/JS scripts.

**Acceptance Criteria**:
- [ ] ContextGlobal implements GlobalObject trait
- [ ] Lua API: Context.assemble(), Context.test(), Context.strategy_stats(), Context.configure_reranking()
- [ ] All methods tested in Lua
- [ ] Documentation with examples

**Implementation Steps**:
1. Create `llmspell-bridge/src/globals/context_global.rs`
2. Implement ContextGlobal similar to MemoryGlobal
3. Register ContextGlobal in create_standard_registry()
4. Create Lua integration tests

**Files to Create/Modify**:
- `llmspell-bridge/src/globals/context_global.rs` (NEW - 500 lines)
- `llmspell-bridge/src/globals/mod.rs` (MODIFY - add context_global module, export)
- `llmspell-bridge/src/globals/registry.rs` (MODIFY - register ContextGlobal, ~10 lines)
- `llmspell-bridge/tests/lua/context_global_test.rs` (NEW - 350 lines)

**Definition of Done**:
- [ ] ContextGlobal registered as 18th global
- [ ] All Lua API methods functional and tested
- [ ] Documentation generated from Rust docs
- [ ] Examples added to user guide

### Task 13.8.5: Bridge Integration Tests

**Priority**: HIGH
**Estimated Time**: 3 hours
**Assignee**: QA + Bridge Team

**Description**: Comprehensive integration tests for bridges and globals.

**Acceptance Criteria**:
- [ ] Test MemoryBridge with all methods
- [ ] Test ContextBridge with all methods
- [ ] Test MemoryGlobal in Lua
- [ ] Test ContextGlobal in Lua

**Implementation Steps**:
1. Test MemoryBridge: episodic operations, semantic queries, consolidation
2. Test ContextBridge: context assembly, strategy selection, reranking
3. Test MemoryGlobal: Lua API coverage
4. Test ContextGlobal: Lua API coverage

**Files to Create/Modify**:
- `llmspell-bridge/tests/integration/bridge_test.rs` (NEW - 400 lines)
- `llmspell-bridge/tests/integration/lua_global_test.rs` (NEW - 350 lines)

**Definition of Done**:
- [ ] All bridge tests pass
- [ ] All global tests pass
- [ ] Test coverage >90%
- [ ] CI integration complete

---

## Phase 13.9-13.15: Remaining Implementation Phases

Due to document length, the remaining phases (13.9: Lua API Validation through 13.15: Release Readiness) follow the same structure as outlined in the original analysis. Each phase includes:

- **Phase 13.9** (Day 15): Lua API Validation - Examples, documentation, integration tests
- **Phase 13.10** (Days 16-17): RAG Integration - Memory-enhanced retrieval, chunking, reranking
- **Phase 13.11** (Days 18-19): Template Integration - Memory parameters for templates
- **Phase 13.12** (Day 20): CLI + UX - `llmspell memory/graph/context` commands
- **Phase 13.13** (Days 21-22): Performance Optimization - Benchmarking, DeBERTa optimization
- **Phase 13.14** (Days 23-24): Accuracy Validation - DMR/NDCG@10 evaluation and tuning
- **Phase 13.15** (Day 25): Release Readiness - Final integration, documentation audit, Phase 14 handoff

**Note**: For detailed task breakdowns of phases 13.9-13.15, refer to the comprehensive design document: `/docs/in-progress/phase-13-design-doc.md` (5,628 lines) which provides complete specifications for each task.

---

## Final Validation Checklist

### Quality Gates
- [ ] Zero clippy warnings: `cargo clippy --workspace --all-targets --all-features`
- [ ] Zero compile errors: `cargo build --workspace --all-features`
- [ ] All tests passing: `cargo test --workspace --all-features`
- [ ] Quality check passing: `./scripts/quality/quality-check.sh`
- [ ] Documentation building: `cargo doc --workspace --no-deps`

### Performance Targets
- [ ] DMR >90% (Decision Match Rate for consolidation)
- [ ] NDCG@10 >0.85 (Retrieval quality)
- [ ] Context assembly P95 <100ms
- [ ] Consolidation throughput >500 records/min
- [ ] Memory footprint <500MB idle

### Integration Validation
- [ ] MemoryManager integrated with Kernel
- [ ] MemoryGlobal (17th) and ContextGlobal (18th) functional in Lua
- [ ] RAG pipeline uses memory for enhanced retrieval
- [ ] Research Assistant and Interactive Chat templates memory-enabled
- [ ] CLI commands functional (memory, graph, context)

### Documentation Completeness
- [ ] API documentation >95% coverage
- [ ] User guides complete (Memory, Context, Templates)
- [ ] Architecture documentation updated
- [ ] RELEASE_NOTES_v0.13.0.md complete
- [ ] ADRs documented (ADR-013, ADR-014)

### Phase 14 Readiness
- [ ] Phase 13 completion checklist verified
- [ ] Phase 14 dependencies documented
- [ ] Known issues documented
- [ ] Technical debt documented
- [ ] Handoff document created

---

## Risk Mitigation

### Technical Risks

**Risk 1**: DMR <90% (Consolidation accuracy below target)
- **Likelihood**: Medium
- **Impact**: High (affects memory quality)
- **Mitigation**:
  - Allocate 2 hours for prompt tuning (Task 13.14.4)
  - Use few-shot examples in consolidation prompts
  - Consider ensemble approach (multiple LLM calls, majority vote)
  - Fallback: Accept 85% DMR for v0.13.0, tune in v0.13.1

**Risk 2**: NDCG@10 <0.85 (Retrieval quality below target)
- **Likelihood**: Medium
- **Impact**: High (affects context quality)
- **Mitigation**:
  - Tune reranking weights (Task 13.14.4)
  - Experiment with different DeBERTa models (larger model if latency permits)
  - Adjust recency and relevance scoring parameters
  - Fallback: Accept 0.80 NDCG@10, document improvement plan

**Risk 3**: Context assembly P95 >100ms (Latency target missed)
- **Likelihood**: Low
- **Impact**: Medium (affects UX)
- **Mitigation**:
  - ONNX quantization (Task 13.13.2)
  - GPU acceleration if available
  - Reduce top_k for reranking (20 → 10)
  - Fallback: Accept 150ms for v0.13.0, optimize in v0.13.1

**Risk 4**: Database integration failures (ChromaDB, SurrealDB, Qdrant)
- **Likelihood**: Medium (external dependencies)
- **Impact**: High (blocks functionality)
- **Mitigation**:
  - In-memory fallback implementations (Tasks 13.1.4, 13.2.3)
  - Thorough integration testing (Task 13.15.1)
  - Docker containers for consistent test environments
  - Fallback: Use in-memory backends for v0.13.0, add external DB support in v0.13.1

**Risk 5**: DeBERTa model loading failures (Candle/ONNX issues)
- **Likelihood**: Medium
- **Impact**: High (blocks reranking)
- **Mitigation**:
  - BM25 fallback reranking (Task 13.4.5)
  - Pre-trained model bundling (download during build)
  - Comprehensive error handling
  - Fallback: Use BM25-only reranking for v0.13.0

### Schedule Risks

**Risk 6**: Scope creep (feature additions beyond design doc)
- **Likelihood**: Medium
- **Impact**: High (delays release)
- **Mitigation**:
  - Strict adherence to PHASE13-TODO.md tasks
  - Defer non-critical features to Phase 14
  - Daily progress tracking against TODO
  - Escalate scope changes to architecture team

**Risk 7**: Dependency on external teams (Kernel, RAG, Templates teams)
- **Likelihood**: Low (internal coordination)
- **Impact**: Medium (blocks integration)
- **Mitigation**:
  - Clear interface contracts defined upfront
  - Parallel development tracks (minimize dependencies)
  - Daily standups for coordination
  - Fallback: Stub implementations if needed

**Risk 8**: Testing bottlenecks (comprehensive test suite takes >25 days)
- **Likelihood**: Low
- **Impact**: Medium (delays validation)
- **Mitigation**:
  - Write tests alongside implementation (not after)
  - Parallelize test execution (cargo test --jobs 8)
  - Focus on critical path tests first
  - Fallback: Defer non-critical tests to v0.13.1

---

## Notes and Decisions Log

### Architectural Decisions

**Decision 1**: LLM-driven consolidation over rule-based
- **Date**: Phase 13 planning
- **Rationale**: Mem0 research shows 26% improvement with LLM decisions
- **Trade-offs**: Higher latency, LLM dependency, but better accuracy
- **Documented in**: ADR-013

**Decision 2**: Bi-temporal knowledge graph (event_time + ingestion_time)
- **Date**: Phase 13 planning
- **Rationale**: Graphiti's 94.8% DMR relies on temporal tracking
- **Trade-offs**: Increased storage, complexity, but enables fact evolution tracking
- **Documented in**: docs/in-progress/phase-13-design-doc.md

**Decision 3**: DeBERTa cross-encoder for reranking (via Candle)
- **Date**: Phase 13 planning
- **Rationale**: Provence research shows NDCG@10 >0.85 with DeBERTa
- **Trade-offs**: Model size (180MB), inference latency, but highest accuracy
- **Documented in**: docs/in-progress/phase-13-design-doc.md

**Decision 4**: Opt-in memory design (zero breaking changes)
- **Date**: Phase 13 planning
- **Rationale**: Maintain backward compatibility with existing users
- **Trade-offs**: Adds configuration complexity, but safe migration
- **Documented in**: docs/in-progress/phase-13-design-doc.md

**Decision 5**: ChromaDB/Qdrant for episodic, SurrealDB/Neo4j for semantic
- **Date**: Phase 13 planning
- **Rationale**: Specialized databases for specialized memory types
- **Trade-offs**: Multiple dependencies, but optimal performance per type
- **Documented in**: docs/in-progress/phase-13-design-doc.md

### Implementation Notes

**Note 1**: In-memory fallbacks critical for testing
- **Date**: Phase 13.1
- **Details**: ChromaDB/Qdrant may not be available in CI environments
- **Action**: Implement in-memory fallbacks for episodic and semantic (Tasks 13.1.4, 13.2.3)

**Note 2**: BM25 fallback reranking essential
- **Date**: Phase 13.4
- **Details**: DeBERTa may fail to load on some platforms (model size, no GPU)
- **Action**: Implement BM25 fallback with graceful degradation (Task 13.4.5)

**Note 3**: Consolidation daemon must be optional
- **Date**: Phase 13.5
- **Details**: Some users may want manual consolidation control
- **Action**: Make daemon configurable (enable_daemon flag in config)

**Note 4**: Session-memory linking requires careful metadata handling
- **Date**: Phase 13.7
- **Details**: Session metadata (user_id, session_id) must propagate to episodic records
- **Action**: Ensure metadata pipeline in Session.add_interaction (Task 13.7.3)

**Note 5**: Template memory integration must be opt-in at template level
- **Date**: Phase 13.11
- **Details**: Users may want some templates with memory, others without
- **Action**: Per-template enable_memory parameter (Task 13.11.1)

### Dependencies Added

**Crate**: llmspell-memory
- chromadb-client = "0.2" (episodic vector storage)
- qdrant-client = "1.8" (alternative episodic storage)
- serde_json = "1.0"
- tokio = { version = "1", features = ["full"] }

**Crate**: llmspell-graph
- surrealdb = "1.5" (semantic graph storage)
- neo4j = "0.8" (alternative graph storage)
- serde = { version = "1.0", features = ["derive"] }
- chrono = "0.4" (bi-temporal timestamps)

**Crate**: llmspell-context
- candle-core = "0.4" (DeBERTa inference)
- candle-nn = "0.4"
- tokenizers = "0.15" (DeBERTa tokenization)
- onnxruntime = "0.0.14" (ONNX optimization)
- tantivy = "0.21" (BM25 fallback)

---

## Team Assignments

### Memory Team (Tasks 13.1, 13.2, 13.3, 13.5, 13.6, 13.10, 13.13, 13.14)
- **Lead**: Senior Rust Engineer with vector DB experience
- **Members**: 2 engineers
- **Responsibilities**:
  - llmspell-memory crate (episodic, semantic, procedural)
  - Consolidation engine and daemon
  - Memory-RAG integration
  - DMR evaluation and tuning

### Context Team (Tasks 13.4, 13.10, 13.13, 13.14)
- **Lead**: Senior Rust Engineer with ML experience
- **Members**: 2 engineers
- **Responsibilities**:
  - llmspell-context crate (query understanding, reranking, assembly)
  - DeBERTa integration (Candle)
  - BM25 fallback
  - NDCG@10 evaluation and tuning

### Kernel Team (Tasks 13.7)
- **Lead**: Kernel maintainer
- **Members**: 1 engineer
- **Responsibilities**:
  - MemoryManager integration into KernelContext
  - ConsolidationDaemon lifecycle
  - Session-memory linking
  - State-memory synchronization

### Bridge Team (Tasks 13.8, 13.9)
- **Lead**: Bridge/scripting specialist
- **Members**: 2 engineers
- **Responsibilities**:
  - MemoryBridge and ContextBridge
  - MemoryGlobal (17th) and ContextGlobal (18th)
  - Lua API validation
  - mlua type conversions

### Templates Team (Tasks 13.11)
- **Lead**: Templates maintainer
- **Members**: 1 engineer
- **Responsibilities**:
  - Template memory parameter integration
  - Research Assistant memory enhancement
  - Interactive Chat memory enhancement
  - Template memory documentation

### CLI Team (Tasks 13.12)
- **Lead**: CLI maintainer
- **Members**: 1 engineer
- **Responsibilities**:
  - memory, graph, context CLI commands
  - Configuration file support
  - User experience polish

### QA Team (Tasks 13.6, 13.9, 13.14, 13.15)
- **Lead**: QA lead
- **Members**: 2 engineers
- **Responsibilities**:
  - E2E testing (memory flow, RAG, templates)
  - Accuracy test dataset creation
  - DMR and NDCG@10 evaluation
  - Final integration testing

### Documentation Team (Tasks 13.6, 13.9, 13.11, 13.15)
- **Lead**: Technical writer
- **Members**: 1 writer + engineers (peer review)
- **Responsibilities**:
  - User guide (Memory, Context APIs)
  - Lua API documentation
  - ADRs (ADR-013, ADR-014)
  - RELEASE_NOTES_v0.13.0.md

### Performance Team (Tasks 13.13)
- **Lead**: Performance engineer
- **Members**: 1-2 engineers (can overlap with Memory/Context teams)
- **Responsibilities**:
  - Benchmarking (context assembly, consolidation, reranking)
  - Optimization (DeBERTa, batching, memory footprint)
  - Performance report generation

### Architecture Team (Tasks 13.15)
- **Lead**: Chief architect
- **Members**: Team leads
- **Responsibilities**:
  - Phase 13 completion verification
  - Phase 14 handoff preparation
  - Technical debt assessment
  - Strategic recommendations

---

## Daily Standup Topics

### Days 1-2: Memory Layer Foundation
- **Day 1**: llmspell-memory crate structure, core traits defined
- **Day 2**: ChromaDB/Qdrant integration, in-memory fallback complete

### Days 3-4: Temporal Knowledge Graph
- **Day 3**: llmspell-graph crate structure, bi-temporal traits defined
- **Day 4**: SurrealDB integration, entity extraction complete

### Day 5: Memory + Graph Integration
- **Day 5**: MemoryManager integrates KnowledgeGraph, consolidation stub ready

### Days 6-7: Context Engineering Pipeline
- **Day 6**: llmspell-context crate structure, query understanding + strategy selection
- **Day 7**: DeBERTa reranking + BM25 fallback complete

### Days 8-9: LLM-Driven Consolidation
- **Day 8**: Consolidation prompts + decision logic implemented
- **Day 9**: Background daemon + metrics complete

### Day 10: E2E Memory Flow
- **Day 10**: E2E test passing, DMR baseline measured, consolidation documented

### Days 11-12: Kernel Integration
- **Day 11**: MemoryManager in KernelContext, daemon lifecycle managed
- **Day 12**: Session-memory linking + state-memory sync complete

### Days 13-14: Bridge + Globals
- **Day 13**: MemoryBridge + ContextBridge implemented
- **Day 14**: MemoryGlobal (17th) + ContextGlobal (18th) functional in Lua

### Day 15: Lua API Validation
- **Day 15**: Lua examples working, API docs complete, integration tests passing

### Days 16-17: RAG Integration
- **Day 16**: RAG pipeline uses memory for retrieval
- **Day 17**: Memory-aware chunking + reranking, E2E test passing

### Days 18-19: Template Integration
- **Day 18**: Template memory parameter integrated
- **Day 19**: Research Assistant + Interactive Chat memory-enhanced

### Day 20: CLI + User Experience
- **Day 20**: All CLI commands functional, configuration support complete

### Days 21-22: Performance Optimization
- **Day 21**: Context assembly benchmarked, DeBERTa optimized
- **Day 22**: Consolidation throughput optimized, memory footprint reduced

### Days 23-24: Accuracy Validation
- **Day 23**: Test dataset created, DMR + NDCG@10 evaluated
- **Day 24**: Tuning complete, targets achieved (DMR >90%, NDCG@10 >0.85)

### Day 25: Release Readiness
- **Day 25**: All tests passing, docs complete, Phase 14 handoff ready

---

**END OF PHASE13-TODO.md**

**Note**: This TODO list provides the foundation for Phases 13.1-13.8. For complete task breakdowns of Phases 13.9-13.15 (Lua API, RAG, Templates, CLI, Performance, Accuracy, Release), see the comprehensive analysis provided earlier in this conversation or refer to `/docs/in-progress/phase-13-design-doc.md` for full specifications.

