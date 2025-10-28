# Phase 13.10 Revised Tasks - RAG+Memory Integration

**Location**: llmspell-context (NOT llmspell-rag)
**Status**: ADR committed (c279bd05), ready to implement

---

## Task 13.10.1: Add llmspell-rag dependency + Create HybridRetriever

**Priority**: CRITICAL
**Estimated Time**: 4 hours
**Status**: READY TO START

**Goal**: Create `HybridRetriever` in llmspell-context that combines RAG vector search with episodic memory retrieval.

**Actions**:
1. Add `llmspell-rag` to `llmspell-context/Cargo.toml` dependencies
2. Create `llmspell-context/src/retrieval/hybrid_rag_memory.rs`
3. Implement `HybridRetriever` struct:
   - Fields: `memory_manager: Arc<dyn MemoryManager>`, `rag_pipeline: Option<Arc<RAGPipeline>>`
   - Method: `retrieve_hybrid(query, strategy, session_id)` → `Vec<RankedChunk>`
   - Combines RAG results + Memory BM25 results
   - Weighted merge (40% RAG, 60% Memory by default)
   - Unified BM25 reranking across both sources
4. Add comprehensive tracing (info!, debug!, warn!, error!)

**Acceptance Criteria**:
- [ ] llmspell-rag dependency added to llmspell-context
- [ ] HybridRetriever compiles with Option<RAGPipeline> (backward compatible)
- [ ] retrieve_hybrid() combines both sources successfully
- [ ] Configurable weight allocation (RAGWeight struct)
- [ ] Fallback: works with rag_pipeline = None (memory-only)
- [ ] Unit tests for hybrid merge logic
- [ ] Tracing verified (info!, debug!, warn!, error!)
- [ ] Zero clippy warnings
- [ ] Compiles: `cargo check -p llmspell-context`

**Files**:
- `llmspell-context/Cargo.toml` (MODIFY - add llmspell-rag dep)
- `llmspell-context/src/retrieval/hybrid_rag_memory.rs` (NEW - ~200 lines)
- `llmspell-context/src/retrieval/mod.rs` (MODIFY - export hybrid module)

---

## Task 13.10.2: Update ContextBridge with Optional RAG Integration

**Priority**: CRITICAL
**Estimated Time**: 4 hours
**Status**: BLOCKED by Task 13.10.1

**Goal**: Enhance `ContextBridge` to optionally use `HybridRetriever` when `RAGPipeline` is available.

**Actions**:
1. Update `ContextBridge` struct:
   - Add field: `rag_pipeline: Option<Arc<RAGPipeline>>`
   - Constructor: `ContextBridge::new(memory_manager)`
   - Builder: `with_rag_pipeline(rag: Arc<RAGPipeline>) -> Self`
2. Update `assemble()` method:
   - Detect if RAG pipeline is available
   - Use `HybridRetriever` when available
   - Fall back to memory-only BM25 retrieval when not
   - No API changes (backward compatible)
3. Add "rag" strategy option:
   - Episodic: memory BM25 only
   - Semantic: memory semantic only
   - Hybrid: memory hybrid (episodic + semantic)
   - **NEW** - Rag: RAG vector search + memory hybrid
4. Update tests in `llmspell-bridge/tests/context_global_test.rs`

**Acceptance Criteria**:
- [ ] ContextBridge has optional rag_pipeline field
- [ ] with_rag_pipeline() builder method works
- [ ] assemble() uses HybridRetriever when RAG available
- [ ] Backward compatible: existing code works without RAG
- [ ] "rag" strategy supported alongside episodic/semantic/hybrid
- [ ] Tests pass with and without RAG pipeline
- [ ] Zero clippy warnings
- [ ] Compiles: `cargo check -p llmspell-bridge`

**Files**:
- `llmspell-bridge/src/context_bridge.rs` (MODIFY - add rag_pipeline field + logic)
- `llmspell-bridge/tests/context_global_test.rs` (MODIFY - add RAG tests)

---

## Task 13.10.3: RAG Adapter + Token Budget Management

**Priority**: HIGH
**Estimated Time**: 4 hours
**Status**: BLOCKED by Task 13.10.1

**Goal**: Create adapter to convert RAGPipeline results to context retrieval format, implement token budget allocation.

**Actions**:
1. Create `llmspell-context/src/retrieval/rag_adapter.rs`:
   - Function: `adapt_rag_results(rag_results) -> Vec<RankedChunk>`
   - Convert RAGPipeline::RetrievalResult to context chunks
   - Preserve scores and metadata
2. Implement token budget allocation in `HybridRetriever`:
   - Split budget across RAG + Memory sources
   - Example: 2000 tokens → 800 RAG (40%) + 1200 Memory (60%)
   - Respect individual source limits
3. Add `RetrievalWeights` config:
   - Default: 40% RAG, 60% Memory
   - Presets: balanced (50/50), rag_focused (80/20), memory_focused (20/80)
   - Validation: weights sum to 1.0
4. Integration test combining real RAG + Memory retrieval

**Acceptance Criteria**:
- [ ] RAG adapter converts RetrievalResult to RankedChunk format
- [ ] Token budget splits correctly across sources
- [ ] RetrievalWeights struct with validation
- [ ] Preset weight configurations work
- [ ] Integration test: RAG + Memory hybrid search succeeds
- [ ] Token limits respected for each source
- [ ] Zero clippy warnings
- [ ] All tests pass: `cargo test -p llmspell-context`

**Files**:
- `llmspell-context/src/retrieval/rag_adapter.rs` (NEW - ~100 lines)
- `llmspell-context/src/retrieval/hybrid_rag_memory.rs` (MODIFY - add budget allocation)
- `llmspell-context/tests/hybrid_retrieval_test.rs` (NEW - integration test)

---

## Task 13.10.4: End-to-End Integration Test + Examples

**Priority**: HIGH
**Estimated Time**: 4 hours
**Status**: BLOCKED by Tasks 13.10.1-3

**Goal**: Create comprehensive end-to-end test and Lua example demonstrating RAG + Memory hybrid retrieval.

**Actions**:
1. Create integration test in `llmspell-bridge/tests/rag_memory_integration_test.rs`:
   - Setup: In-memory RAG + in-memory Memory
   - Ingest documents into RAG
   - Add conversation to episodic memory
   - Query with hybrid retrieval (rag strategy)
   - Verify: Results include both RAG documents + memory conversations
   - Verify: Correct weighting applied
2. Create Lua example `examples/script-users/cookbook/rag-memory-hybrid.lua`:
   - Demonstrate RAG ingestion
   - Add conversation to memory
   - Use Context.assemble() with "rag" strategy
   - Show hybrid results
3. Update `docs/user-guide/api/lua/README.md`:
   - Document "rag" strategy for Context.assemble()
   - Add hybrid retrieval example
   - Explain when to use RAG vs memory-only strategies

**Acceptance Criteria**:
- [ ] Integration test passes: RAG + Memory hybrid retrieval works end-to-end
- [ ] Lua example runs successfully via `llmspell run`
- [ ] Example demonstrates both RAG and Memory results in output
- [ ] API documentation updated with "rag" strategy
- [ ] Tracing verified in integration test
- [ ] Zero clippy warnings
- [ ] All Phase 13.10 tests pass: `cargo test -p llmspell-context -p llmspell-bridge`

**Files**:
- `llmspell-bridge/tests/rag_memory_integration_test.rs` (NEW - ~150 lines)
- `examples/script-users/cookbook/rag-memory-hybrid.lua` (NEW - ~80 lines)
- `docs/user-guide/api/lua/README.md` (MODIFY - add "rag" strategy docs)

---

## Phase 13.10 Completion Checklist

### Dependencies
- [x] Phase 13.8 complete (Memory + Context globals)
- [x] Phase 13.9 complete (Documentation + validation)
- [x] Architectural Decision committed (c279bd05)

### Implementation
- [ ] Task 13.10.1: Hybrid RAG+Memory Retriever complete
- [ ] Task 13.10.2: ContextBridge Enhancement complete
- [ ] Task 13.10.3: RAG Adapter + Budget Management complete
- [ ] Task 13.10.4: Integration Tests + Examples complete

### Quality Gates
- [ ] Zero clippy warnings: `cargo clippy -p llmspell-context -p llmspell-bridge --all-targets --all-features`
- [ ] All tests pass: `cargo test -p llmspell-context -p llmspell-bridge`
- [ ] Lua example runs: `./target/release/llmspell run examples/script-users/cookbook/rag-memory-hybrid.lua`
- [ ] Tracing verified: `RUST_LOG=debug` shows proper instrumentation
- [ ] Documentation complete: API docs + user guide updated

### Performance Targets
- [ ] Hybrid retrieval overhead: <50ms additional latency vs memory-only
- [ ] Token budget allocation: accurate within ±5%
- [ ] Backward compatibility: context works without RAG (no regressions)

---

**Next Steps After Phase 13.10**:
- Phase 13.11: Template Integration (memory-aware workflows)
- Phase 13.12: CLI + UX Integration (memory CLI commands)
- Phase 13.13: Performance Optimization (benchmarking, caching)
