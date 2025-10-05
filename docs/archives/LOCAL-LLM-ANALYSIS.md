# Local LLM Implementation Analysis

**Date**: 2025-10-01
**Context**: Phase 10 complete, Phase 11 starting. Need local LLM support decision.

## Initial Analysis: implementation-phases.md

### Finding 1: NO explicit local LLM phase planned
- Phase 1 mentions: "Simple LLM provider integration using `rig`"
- Phase 1 mentions: "Provider abstraction layer for future multimodal providers (mistral.rs, etc.)"
- Phase 2 mentions: Provider enhancement with ModelSpecifier for "provider/model" syntax
- **No dedicated phase for local LLM implementation**
- Only reference is "mistral.rs" as potential future provider (Phase 1:76)

### Finding 2: Current provider architecture
- Using `rig` crate for cloud LLM wrapping (Phase 1:74)
- Provider abstraction exists for multimodal support (Phase 1:76)
- ProviderConfig with provider_type field (Phase 3.3:252-254)
- Hierarchical naming: `rig/openai/gpt-4` pattern (Phase 3.3:253)

### Question: Why was local LLM deferred?
- Not in phases 0-21 explicit scope
- Mentioned as "future" in Phase 1 but never scheduled
- **Need to check PHASE*DONE.md files for actual implementation choices**

## Finding 3: Current llmspell-providers implementation (rig.rs)
- **RigProvider enum**: Wraps OpenAI, Anthropic, Cohere providers (rig.rs:18-22)
- **No local LLM support**: Only cloud providers via rig crate
- **Provider matching**: Hard-coded provider_type checks (rig.rs:42-97)
- **No Ollama integration**: Despite rig supporting it
- **Architecture**: Provider abstraction exists but only implements cloud

## Answer to Question 2: Why was local LLM never implemented?
1. **Phase 1**: Focused on cloud providers via rig for MVP speed
2. **Phase 2**: Focused on 26 self-contained tools, no provider work
3. **Phases 3-10**: Agent infrastructure, hooks, state, REPL, services - no provider expansion
4. **Result**: Local LLM deferred indefinitely, never scheduled

## Research Finding 4: Rust Candle (Hugging Face)
**What it is**: Pure Rust ML framework from Hugging Face for local inference
**Capabilities**:
- Pure Rust (zero Python deps) - 7.8k stars on GitHub
- CPU/GPU inference (CUDA, Metal, etc.)
- Supports: LLaMA 1/2/3, Falcon, GLM4, Gemma, Whisper, Stable Diffusion
- Projects built on it: candle-vllm (OpenAI API compatible), Crane (6x faster on Mac)
- Designed for serverless inference (lightweight binaries)
- Active development, strong ecosystem

**Integration approach**:
- Add `candle` crate dependency to llmspell-providers
- Implement new `CandleProvider` struct implementing `ProviderInstance` trait
- Load GGUF models from disk
- Handle model initialization and inference
- **Complexity**: MEDIUM-HIGH (need model loading, tokenization, generation loop)
- **Control**: MAXIMUM (direct inference in Rust process)

## Research Finding 5: Rust Orca (LLM Orchestration)
**What it is**: LLM Orchestration Framework, NOT an inference library
**Capabilities**:
- Orchestration layer (like langchain) built in Rust
- Aims for WebAssembly compilation
- Uses Candle under the hood for inference
- RAG CLI applications without internet
- **Status**: Early development, limited functionality

**Verdict**: NOT what user meant - likely confused with Candle. Orca is orchestration, Candle is inference.

## Research Finding 6: Rig-Ollama Integration
**What it is**: rig-core already supports Ollama as provider (v0.20.0)
**Capabilities**:
- Rig v0.20.0 supports Ollama among 19 providers (Anthropic, OpenAI, Ollama, etc.)
- Ollama runs as external server (Go-based, not Rust)
- Easy model management via CLI (`ollama pull llama2`)
- ollama-rs crate (v0.3.2) for Rust client
- OpenAI-compatible API
- **Complexity**: LOW (just add Ollama provider case to existing RigProvider enum)
- **External dependency**: Ollama binary must be installed and running

## Research Finding 7: Architecture Comparison
**Ollama (via rig)**:
- External process (Go binary)
- Pros: Easy setup, mature, model management CLI, wide model support
- Cons: External dependency, another process to manage, Go binary (not Rust-native)
- Integration effort: LOW (add to existing RigProvider enum)

**Candle (direct)**:
- In-process Rust library
- Pros: Pure Rust, no external deps, full control, embeddable
- Cons: Need to handle model loading/tokenization, larger binary, more complex
- Integration effort: MEDIUM-HIGH (new provider implementation)

---

# RECOMMENDATION: Two-Phase Approach (Pragmatic + Future-Proof)

## Phase 11.1: Rig-Ollama Integration (IMMEDIATE - 1-2 days)
**Recommendation**: Start with Ollama via rig for quick local LLM support

**Rationale**:
1. **Minimal effort**: Add one provider case to existing RigProvider enum (llmspell-providers/src/rig.rs:42-97)
2. **Proven mature**: Ollama is production-ready with excellent model ecosystem
3. **Existing architecture fit**: rig v0.20.0 already supports Ollama - just wire it up
4. **User productivity**: Enables local LLM immediately without waiting for complex Candle integration
5. **Phase 11 needs**: Adaptive Memory System (Phase 11) needs LLM for knowledge extraction NOW

**Implementation**:
```rust
// In llmspell-providers/src/rig.rs, add to match statement at line 42:
"ollama" => {
    let base_url = config.endpoint.as_deref()
        .unwrap_or("http://localhost:11434");
    let client = providers::ollama::Client::new(base_url);
    let model = client.completion_model(&config.model);
    RigModel::Ollama(model)
}
```

**Estimated effort**: 4-6 hours (add provider case, test with local Ollama, update docs)

**User experience**:
```lua
-- User just needs: ollama pull llama2
local agent = Agent.create({
  model = "ollama/llama2",  -- Works immediately!
  temperature = 0.7
})
```

## Phase 11.2 or Later: Candle Integration (FUTURE - Embedded scenarios)
**Recommendation**: Add Candle for truly embedded, dependency-free scenarios

**When to implement**:
- Phase 15 (Library Mode) - embedding llmspell in other Rust apps
- Phase 16 (Cross-Platform) - serverless deployments, edge devices
- When users need: No external processes, WASM compilation, maximum control

**Rationale for later**:
1. **Complexity**: Candle requires model loading, tokenization, generation loop (1-2 weeks)
2. **Not blocking**: Ollama covers 90% of local LLM use cases now
3. **Better timing**: Library Mode (Phase 15) is natural fit for embedded inference
4. **Resource optimization**: Candle shines in constrained environments (Phase 15/16 focus)

**Implementation sketch**:
```rust
// New file: llmspell-providers/src/candle.rs
pub struct CandleProvider {
    model: CandleModel,
    tokenizer: Tokenizer,
    config: ProviderConfig,
}

impl ProviderInstance for CandleProvider {
    async fn complete(&self, input: AgentInput) -> Result<AgentOutput> {
        // Load GGUF, tokenize, run inference loop
    }
}
```

**Estimated effort**: 1-2 weeks (model loading, tokenization, generation, streaming)

---

# ANSWERS TO YOUR QUESTIONS

## Q1: Where's the plan for local LLM implementation?
**Answer**: NOWHERE. Never explicitly planned in phases 0-21.
- Phase 1 mentioned "mistral.rs" as future option but never scheduled
- Deferred indefinitely due to MVP focus on cloud providers

## Q2: What happened during phases? Why no local?
**Answer**: Speed-to-MVP prioritization
- Phase 1: Cloud via rig (fast MVP delivery)
- Phase 2: 26 tools (no provider work)
- Phases 3-10: Infrastructure (agents, hooks, state, REPL, services)
- Result: Local LLM fell through the cracks - should be added NOW for Phase 11

## Q3: What approach should we use?
**Answer**: BOTH, but staged:
1. **NOW (Phase 11.1)**: Ollama via rig - quick wins, enables Phase 11 Memory System
2. **LATER (Phase 15+)**: Candle direct - embedded scenarios, library mode

**Why both?**
- Ollama: External process, easy setup, mature - covers 90% of users immediately
- Candle: Embedded in-process, complex setup - needed for library mode, WASM, edge

**Project fit**:
- Ollama aligns with Phase 11 (Memory needs LLM extraction NOW)
- Candle aligns with Phase 15 (Library Mode needs embedded inference)

---

# IMPLEMENTATION PRIORITY

**IMMEDIATE (This Week)**:
- Add Ollama to RigProvider enum (4-6 hours)
- Test with `ollama pull llama2`
- Update docs with local LLM examples
- Unblock Phase 11 Memory System work

**FUTURE (Phase 15-16)**:
- Design CandleProvider for embedded scenarios
- Implement model loading and tokenization
- Add to Phase 15 Library Mode design doc
- Target: serverless, WASM, edge deployments

---

# TECHNICAL JUSTIFICATION

**Why Ollama first?**
1. **Rig v0.20.0 already supports it** - we're late to the party
2. **1-2 days vs 1-2 weeks** - 10x faster to production
3. **Phase 11 blocked** - Memory System needs LLM for entity extraction
4. **Mature ecosystem** - Ollama has wide adoption, model zoo
5. **User expectation** - most users already use Ollama locally

**Why Candle eventually?**
1. **Phase 15 Library Mode** - embedding llmspell needs in-process inference
2. **Phase 16 WASM target** - Candle compiles to WASM, Ollama doesn't
3. **Zero dependencies** - serverless/edge scenarios can't run external processes
4. **Rust-native control** - full control over inference for optimization

**Why not Candle now?**
1. **Complexity** - need model loading, tokenization, generation (1-2 weeks)
2. **Phase 11 timing** - Memory System development starting, can't wait 2 weeks
3. **User friction** - Ollama's `ollama pull` UX beats "download 4GB GGUF file"
4. **80/20 rule** - Ollama covers 90% of use cases with 5% of implementation effort

**Project philosophy alignment**:
- "Less code is better" → Ollama adds ~20 lines to existing enum
- "No shortcuts" → Both will exist, just staged appropriately
- "Future-based thinking" → Candle planned for Phase 15 (Library Mode)
- "Attack complexity" → Start simple (Ollama), add complexity when needed (Candle in Phase 15)

---

**FINAL VERDICT**: Implement Ollama NOW (Phase 11.1), plan Candle for Phase 15 (Library Mode).
This unblocks Phase 11 immediately while positioning for embedded scenarios in Phase 15-16.

---

# UPDATE: Phase 11 Design Complete

**Decision**: Implement BOTH Ollama and Candle in Phase 11 (20 days)
- Phase 11.1 (Days 1-10): Ollama integration
- Phase 11.2 (Days 11-20): Candle integration

**Design Document**: `docs/in-progress/phase-11-design-doc.md`

## Key Design Features

### Unified Model Management
- Single CLI: `llmspell model` for both backends
- Consistent syntax: `local/<model>:<variant>[@backend]`
- Auto-detection: Prefers Ollama, falls back to Candle
- Examples:
  - `local/llama3.1:8b` (auto-detect)
  - `local/phi3:3.8b@ollama` (force Ollama)
  - `local/mistral:7b@candle` (force Candle)

### Recommended Models
1. **Phi-3 Mini (3.8B)** - Smallest, fastest (~2.4GB)
2. **Mistral 7B** - Best quality/size ratio (~4.1GB)
3. **LLaMA 3.1 8B** - Most capable (~4.7GB)
4. **Gemma 2 9B** - Google's efficient model (~5.4GB)

### Implementation Details
**Ollama**: Uses ollama-rs crate (v0.3.2+)
- External process at localhost:11434
- Rich model library via `ollama pull`
- Streaming support built-in
- Auto-start capability

**Candle**: Uses candle-core + candle-transformers
- Pure Rust embedded inference
- GGUF format from HuggingFace
- GPU acceleration (CUDA/Metal)
- Q4_K_M quantization default

### CLI Commands
```bash
# Check status
llmspell model status

# List local models
llmspell model list [--backend ollama|candle]

# Download models
llmspell model pull ollama/phi3:3.8b
llmspell model pull candle/mistral:7b --quantization Q4_K_M

# View available models
llmspell model available [--recommended]

# Install Ollama
llmspell model install-ollama
```

### Script API (Lua)
```lua
-- Auto-detect backend
local agent = Agent.create({
    model = "local/llama3.1:8b"
})

-- Force specific backend
local ollama_agent = Agent.create({
    model = "local/phi3:3.8b@ollama"
})

local candle_agent = Agent.create({
    model = "local/mistral:7b@candle"
})

-- Check status
local status = LocalLLM.status()

-- List models
local models = LocalLLM.list()

-- Download from script
LocalLLM.pull("ollama/phi3:3.8b")
```

### Performance Targets
- **First token latency**: <100ms (Ollama), <200ms (Candle)
- **Throughput**: >20 tokens/second for 7B models
- **Memory**: <5GB RAM for Q4_K_M quantized models
- **Model load**: <5 seconds
- **Cold start**: <10 seconds total

### Example Applications
Created 5 example applications in `examples/script-users/applications/local-chat/`:
1. `local_chat.lua` - Simple chat with auto-detection
2. `ollama_chat.lua` - Ollama-specific example
3. `candle_inference.lua` - Candle-specific with benchmarking
4. `backend_comparison.lua` - Compare Ollama vs Candle performance
5. `llmspell.toml` - Configuration template

## Implementation Priority

**Immediate (Phase 11.1 - Days 1-10)**:
1. Ollama provider implementation
2. Model management CLI basics
3. Bridge layer integration
4. Basic examples and tests
5. Documentation

**Follow-up (Phase 11.2 - Days 11-20)**:
1. Candle provider implementation
2. GGUF loading pipeline
3. HuggingFace model download
4. Performance benchmarks
5. Comprehensive testing

## Technical Justification

**Why both in Phase 11?**
1. **Complete local solution**: Users get choice of ease (Ollama) vs embedded (Candle)
2. **Library mode prep**: Candle implementation prepares for Phase 15
3. **Performance comparison**: Real benchmarks inform future decisions
4. **User flexibility**: Different use cases favor different backends
5. **Provider abstraction validation**: Tests architecture with diverse implementations

**Architecture benefits**:
- Unified `LocalProviderInstance` trait
- Backend-agnostic script API
- Consistent model specification
- Shared model management tooling
- Future-proof for Phase 15+ scenarios

Total effort: 20 days (acceptable for comprehensive local LLM foundation)
