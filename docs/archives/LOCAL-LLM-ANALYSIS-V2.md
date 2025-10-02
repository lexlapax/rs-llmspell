# Phase 11 Design Doc - Comprehensive Gap Analysis

**Date**: 2025-10-01
**Context**: Ultrathink analysis of phase-11-design-doc.md integration with existing codebase
**Requested by**: User asking "it feels like it's missing a lot of details on connecting existing code to the new code proposed"

---

## Executive Summary

Phase 11 design doc proposes solid local LLM features but **lacks critical integration details** for 5 major touch points:
1. Config structure incompatibility (nested vs flat)
2. Provider routing logic missing
3. ProviderManager factory pattern not addressed
4. Model specifier parsing integration unclear
5. Bridge/CLI integration details absent

**Critical Finding**: rig-core ALREADY supports Ollama natively - design should leverage this rather than direct ollama-rs.

---

## 1. CONFIG STRUCTURE GAP - CRITICAL

### Current Implementation (llmspell-config/src/providers.rs:12-21)
```rust
pub struct ProviderManagerConfig {
    pub default_provider: Option<String>,
    pub providers: HashMap<String, ProviderConfig>,  // ← FLAT structure
}
```

### Design Doc Proposes (phase-11-design-doc.md:123-144)
```toml
[providers.local]                    # ← NESTED structure
enabled = true
default_backend = "ollama"

[providers.local.ollama]             # ← Second level nesting
url = "http://localhost:11434"
timeout_seconds = 120

[providers.local.candle]
model_directory = "${HOME}/.llmspell/models/candle"
device = "auto"
```

### The Problem
1. **Incompatible serialization**: Current `HashMap<String, ProviderConfig>` expects flat keys like `"openai"`, `"anthropic"`, not nested `local.ollama`
2. **Merge logic breaks**: `llmspell-config/src/lib.rs:246-334` iterates flat provider entries, not nested sections
3. **ProviderConfig struct mismatch**: Design shows backend-specific fields (`url`, `model_directory`) but current ProviderConfig only has generic fields

### Required Changes (MISSING FROM DESIGN DOC)

#### Option A: Flat Keys with Namespacing (RECOMMENDED)
```toml
[providers.local_ollama]
provider_type = "local"
backend = "ollama"
base_url = "http://localhost:11434"
timeout_seconds = 120
enabled = true

[providers.local_candle]
provider_type = "local"
backend = "candle"
model_directory = "${HOME}/.llmspell/models/candle"
device = "auto"
```

**Benefits**:
- Works with existing HashMap structure
- No changes to config merge logic
- Backend-specific fields go in `ProviderConfig.options: HashMap<String, serde_json::Value>` (already exists, providers.rs:138)

#### Option B: Add LocalProviderConfig Type
```rust
// In llmspell-config/src/providers.rs
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct LocalProviderConfig {
    pub enabled: bool,
    pub default_backend: String,
    pub ollama: OllamaConfig,
    pub candle: CandleConfig,
}

// Extend ProviderManagerConfig
pub struct ProviderManagerConfig {
    pub default_provider: Option<String>,
    pub providers: HashMap<String, ProviderConfig>,
    pub local: Option<LocalProviderConfig>,  // ← NEW
}
```

**Requires**: Modifications to config merge logic in lib.rs:246-334 to handle nested structure.

### Design Doc Update Required
**Section to add**: "3.2 Config Structure Integration"
- Show exact ProviderConfig field mapping
- Explain how nested config maps to flat HashMap OR propose ProviderManagerConfig extension
- Show modified config merge logic if nesting approach used
- Explain backend field storage (`options` HashMap vs dedicated structs)

---

## 2. PROVIDER ROUTING GAP - CRITICAL

### Current Implementation (llmspell-providers/src/abstraction.rs:427-431)
```rust
let implementation_name = match provider_name.as_str() {
    "openai" | "anthropic" | "cohere" | "groq" | "perplexity" | "together"
    | "gemini" | "mistral" | "replicate" | "fireworks" => "rig",
    other => other,
};
```

This code maps provider types to factory names. All cloud providers route to `"rig"` factory.

### Design Doc Shows (phase-11-design-doc.md:251-259)
```lua
-- User syntax
local agent = Agent.create({
    model = "local/llama3.1:8b"  -- ← How does this route?
})

local ollama_agent = Agent.create({
    model = "local/phi3:3.8b@ollama"  -- ← What parses @backend?
})
```

### The Problem
1. **No parser shown**: Design doc doesn't show how `"local/llama3.1:8b"` gets parsed into provider routing
2. **Backend suffix handling**: Who parses `@ollama` vs `@candle`? ModelSpecifier? ProviderManager?
3. **Factory registration missing**: No code showing how "local" factory gets registered in ProviderRegistry
4. **Default backend resolution**: When `@backend` omitted, how does "default_backend = ollama" get applied?

### Current ModelSpecifier (llmspell-providers/src/model_specifier.rs)
Design doc doesn't reference this, but `create_agent_from_spec()` in abstraction.rs:388-486 uses it. Need to extend it:

```rust
// Current: handles "provider/model" and "model"
pub struct ModelSpecifier {
    pub provider: Option<String>,  // e.g., "openai"
    pub model: String,              // e.g., "gpt-4"
    pub base_url: Option<String>,
}

// NEEDED for Phase 11:
pub struct ModelSpecifier {
    pub provider: Option<String>,      // e.g., "local"
    pub model: String,                  // e.g., "llama3.1:8b"
    pub backend: Option<String>,        // e.g., "ollama" or "candle" ← NEW
    pub base_url: Option<String>,
}
```

### Required Changes (MISSING FROM DESIGN DOC)

#### 1. ModelSpecifier Extension
```rust
impl ModelSpecifier {
    pub fn parse(spec: &str) -> Result<Self> {
        // Parse formats:
        // "local/llama3.1:8b" -> provider="local", model="llama3.1:8b", backend=None
        // "local/llama3.1:8b@ollama" -> provider="local", model="llama3.1:8b", backend="ollama"
        // "llama3.1:8b@ollama" -> provider=None, model="llama3.1:8b", backend="ollama"
    }
}
```

#### 2. Provider Routing Update
```rust
// In abstraction.rs create_agent_from_spec(), add:
let implementation_name = match provider_name.as_str() {
    "openai" | "anthropic" | "cohere" | ... => "rig",
    "local" => {
        // Resolve backend from spec or config default_backend
        spec.backend
            .as_deref()
            .or_else(|| config.local.as_ref()?.default_backend.as_str())
            .unwrap_or("ollama")  // hardcoded fallback
    }
    other => other,
};
```

#### 3. Factory Registration
```rust
// Where does this happen? Kernel initialization?
provider_manager.register_provider("ollama", |config| {
    Ok(Box::new(OllamaProvider::new(config)?))
}).await;

provider_manager.register_provider("candle", |config| {
    Ok(Box::new(CandleProvider::new(config)?))
}).await;
```

### Design Doc Update Required
**Section to add**: "3.3 Provider Resolution Flow"
```
1. Lua: Agent.create({ model = "local/llama3.1:8b@ollama" })
   ↓
2. Bridge: Parse ModelSpecifier from string
   - provider = "local"
   - model = "llama3.1:8b"
   - backend = "ollama"
   ↓
3. ProviderManager.create_agent_from_spec():
   - Map provider="local" + backend="ollama" → factory_name="ollama"
   - If backend=None, read config.local.default_backend → "ollama"
   ↓
4. Registry lookup: factories.get("ollama") → OllamaProvider factory
   ↓
5. Factory creates OllamaProvider instance with config from providers.local_ollama
   ↓
6. Return initialized provider to Lua agent
```

---

## 3. RIG OLLAMA SUPPORT - ARCHITECTURAL DECISION NEEDED

### Research Finding (Web Search 2025-10-01)
**rig-core NATIVELY supports Ollama** among 20+ providers:
> "Rig natively supports the following completion and embedding model provider integrations: Anthropic · Azure · Cohere · Deepseek · Galadriel · Gemini · Groq · Huggingface · Hyperbolic · Mira · Mistral · Moonshot · **Ollama** · Openai · OpenRouter · Perplexity · Together · Voyage AI · xAI"

### Current Implementation (llmspell-providers/src/rig.rs:17-22)
```rust
enum RigModel {
    OpenAI(providers::openai::CompletionModel),
    Anthropic(providers::anthropic::completion::CompletionModel),
    Cohere(providers::cohere::CompletionModel),
    // Ollama NOT included ← But rig supports it!
}
```

### Design Doc Assumption (phase-11-design-doc.md:267-280)
Design doc assumes direct `ollama-rs` crate usage:
```rust
pub struct OllamaProvider {
    client: ollama_rs::Ollama,  // ← Direct integration
    config: LocalProviderConfig,
}
```

### The Question: Rig or Direct?

#### Option A: Use Rig's Ollama Support (RECOMMENDED)
**Benefits**:
- Consistent with existing RigProvider pattern
- Rig handles retry, timeout, streaming already
- Less code to maintain
- Same API surface as other providers

**Changes needed**:
```rust
// In rig.rs, add to enum:
enum RigModel {
    OpenAI(providers::openai::CompletionModel),
    Anthropic(providers::anthropic::completion::CompletionModel),
    Cohere(providers::cohere::CompletionModel),
    Ollama(providers::ollama::CompletionModel),  // ← ADD THIS
}

// In create_rig_provider match:
"ollama" => {
    let base_url = config.endpoint.as_deref()
        .unwrap_or("http://localhost:11434");
    let client = providers::ollama::Client::new(base_url);
    let model = client.completion_model(&config.model);
    RigModel::Ollama(model)
}
```

**Effort**: ~4 hours (add case to existing enum, test)

#### Option B: Direct ollama-rs Integration (As Designed)
**Benefits**:
- More control over Ollama-specific features
- Can implement LocalProviderInstance trait with model management
- Direct access to Ollama API features rig might not expose

**Drawbacks**:
- Duplicates work rig already does
- Need separate retry/timeout logic
- Two provider patterns (rig for cloud, direct for local)

**Effort**: ~2 days (new provider implementation, testing)

### Design Doc Update Required
**Section to add**: "3.4 Ollama Integration Strategy"
- **Decision**: Use rig's Ollama support OR direct ollama-rs (with justification)
- If rig: Show RigModel enum extension in rig.rs
- If direct: Justify why bypassing rig (access to model management APIs?)
- **Hybrid approach**: Use rig for inference, direct ollama-rs for model management (pull, list, etc.)

---

## 4. PROVIDER MANAGER INTEGRATION GAP

### Current ProviderManager (abstraction.rs:257-567)

Key methods:
- `register_provider()` (line 275): Register factory by name
- `init_provider()` (line 292): Create provider from ProviderConfig
- `create_agent_from_spec()` (line 388): Create from ModelSpecifier
- `get_provider()` (line 317): Retrieve by hierarchical name

### Design Doc Shows (phase-11-design-doc.md:65-97)
```rust
pub enum ProviderCategory {
    Cloud(CloudProvider),
    Local(LocalProvider),
}

pub struct ProviderManager {
    // ... existing fields
    categories: HashMap<String, ProviderCategory>,  // ← NEW?
}
```

### The Problem
1. **Duplicate architecture?**: Design shows ProviderCategory enum, but existing ProviderManager uses factories
2. **Factory pattern ignored**: No mention of how OllamaProvider/CandleProvider register with ProviderRegistry
3. **Hierarchical naming**: Current system uses `"rig/openai/gpt-4"`, design uses `"local/llama3.1:8b"` - how do these coexist?

### Required Changes (MISSING FROM DESIGN DOC)

#### Option A: Keep Factory Pattern (RECOMMENDED - No Breaking Changes)
```rust
// During kernel initialization:
pub async fn init_providers(
    manager: &ProviderManager,
    config: &LLMSpellConfig,
) -> Result<()> {
    // Existing cloud providers
    manager.register_provider("rig", create_rig_provider).await;

    // NEW: Local providers
    if config.providers.local.enabled {
        manager.register_provider("ollama", |config| {
            Ok(Box::new(OllamaProvider::new(config)?))
        }).await;

        manager.register_provider("candle", |config| {
            Ok(Box::new(CandleProvider::new(config)?))
        }).await;
    }

    Ok(())
}
```

No changes to ProviderManager struct needed - just register more factories.

#### Option B: Add ProviderCategory Layer (Breaking Change)
Not recommended - adds complexity without clear benefit. Factory pattern handles routing well.

### Design Doc Update Required
**Section to add**: "3.5 ProviderManager Registration"
```rust
// Show WHERE and WHEN local provider factories are registered
// Likely in: llmspell-kernel initialization

impl Kernel {
    pub async fn initialize_providers(&mut self) -> Result<()> {
        // Show complete provider registration including local
    }
}
```

**Section to add**: "3.6 Instance Naming Convention"
- Cloud: `"rig/openai/gpt-4"` (existing)
- Local: `"local_ollama/llama3.1:8b"` or `"ollama/llama3.1:8b"`?
- How does `get_provider()` resolve these?

---

## 5. BRIDGE LAYER INTEGRATION GAP

### Current Bridge Pattern (llmspell-bridge)
Bridge injects globals into Lua/JS via `BridgeContext`. Design doc shows:

```lua
-- Phase 11 proposed API
local status = LocalLLM.status()
local models = LocalLLM.list()
LocalLLM.pull("ollama/phi3:3.8b")
```

### The Problem
1. **No Bridge module shown**: How does `LocalLLM` global get injected?
2. **Kernel message protocol missing**: Bridge calls kernel - what messages?
3. **Tool vs Global?**: Is LocalLLM a global API or should it be tool commands?

### Current Tool Architecture (llmspell-cli/src/commands/tool.rs)
Tool commands send messages to kernel via `tool_request`:
```rust
let request_content = json!({
    "command": "list",
    "category": category,
});
let response = handle.send_tool_request(request_content).await?;
```

### Required Changes (MISSING FROM DESIGN DOC)

#### Option A: LocalLLM as Bridge Global (Consistent with Agent API)
```rust
// In llmspell-bridge/src/lua/globals/mod.rs (add new module)
pub mod local_llm;

// In llmspell-bridge/src/lua/globals/local_llm.rs
pub fn inject_local_llm_global(lua: &Lua, kernel_handle: KernelHandle) -> Result<()> {
    let local_llm_table = lua.create_table()?;

    // LocalLLM.status()
    local_llm_table.set("status", lua.create_async_function(|lua, ()| async move {
        // Send kernel message: { type: "local_llm_status" }
        // Return: { ollama: "running", candle: "ready" }
    })?)?;

    // LocalLLM.list()
    local_llm_table.set("list", lua.create_async_function(|lua, backend: Option<String>| async move {
        // Send kernel message: { type: "local_llm_list", backend: "ollama" }
        // Return: Vec<LocalModel>
    })?)?;

    // LocalLLM.pull(spec)
    local_llm_table.set("pull", lua.create_async_function(|lua, spec: String| async move {
        // Send kernel message: { type: "local_llm_pull", spec: "ollama/phi3:3.8b" }
        // Stream progress updates
    })?)?;

    lua.globals().set("LocalLLM", local_llm_table)?;
    Ok(())
}
```

#### Option B: Extend Tool Command System (Consistent with CLI)
```bash
# CLI commands (design doc shows these)
llmspell model status
llmspell model list --backend ollama
llmspell model pull ollama/phi3:3.8b

# Could expose as tools in Lua:
Tools.local_model_status()
Tools.local_model_list({ backend = "ollama" })
Tools.local_model_pull("ollama/phi3:3.8b")
```

**Problem**: Tools are for external resources, LocalLLM is infrastructure. Global API more appropriate.

### Design Doc Update Required
**Section to add**: "4.2 Bridge Layer Implementation"
```rust
// Show complete flow:
1. Lua script calls: LocalLLM.status()
2. Bridge creates kernel message: { type: "local_llm_status" }
3. Kernel routes to LocalModelManager component
4. LocalModelManager calls OllamaProvider.health_check() + CandleProvider.health_check()
5. Kernel sends response back through bridge
6. Bridge converts to Lua table and returns
```

**Section to add**: "4.3 Kernel Message Protocol Extensions"
```rust
// New message types needed:
pub enum LocalLLMRequest {
    Status,
    List { backend: Option<String> },
    Pull { spec: String },
    Info { model_id: String },
    Remove { model_id: String },
}

pub enum LocalLLMResponse {
    Status { ollama: ServiceStatus, candle: ServiceStatus },
    ModelList { models: Vec<LocalModel> },
    PullProgress { progress: f32, stage: String },
    // ...
}
```

---

## 6. CLI INTEGRATION GAP

### Current CLI Command Pattern (llmspell-cli/src/commands/mod.rs)
```rust
pub async fn execute_command(
    command: Commands,
    runtime_config: LLMSpellConfig,
    output_format: OutputFormat,
) -> Result<()> {
    match command {
        Commands::Tool { command, source } => {
            tool::handle_tool_command(command, source, runtime_config, output_format).await
        }
        // ... other commands
    }
}
```

### Design Doc Shows (phase-11-design-doc.md:281-297)
```bash
llmspell model status
llmspell model list [--backend ollama|candle]
llmspell model pull ollama/phi3:3.8b
llmspell model available [--recommended]
llmspell model install-ollama
```

### The Problem
1. **No CLI enum shown**: What's the clap command structure?
2. **Execution context**: Does model command use embedded kernel or connect to daemon?
3. **Config integration**: How does `llmspell model pull` read config for model_directory paths?

### Required Changes (MISSING FROM DESIGN DOC)

#### Add Model Command Enum
```rust
// In llmspell-cli/src/cli.rs
#[derive(Debug, Parser)]
pub enum Commands {
    // ... existing commands

    /// Local model management
    #[command(subcommand)]
    Model(ModelCommands),
}

#[derive(Debug, Subcommand)]
pub enum ModelCommands {
    /// Check local LLM backend status
    Status,

    /// List available local models
    List {
        /// Filter by backend
        #[arg(long)]
        backend: Option<String>,
    },

    /// Pull a model from registry
    Pull {
        /// Model specification (e.g., "ollama/phi3:3.8b")
        spec: String,

        /// Quantization level (candle only)
        #[arg(long)]
        quantization: Option<String>,
    },

    /// Show available models
    Available {
        /// Show only recommended models
        #[arg(long)]
        recommended: bool,
    },

    /// Install Ollama binary
    InstallOllama,
}
```

#### Add Model Command Handler
```rust
// In llmspell-cli/src/commands/model.rs (NEW FILE)
pub async fn handle_model_command(
    command: ModelCommands,
    runtime_config: LLMSpellConfig,
    output_format: OutputFormat,
) -> Result<()> {
    // Similar pattern to tool.rs:
    // 1. Resolve ExecutionContext (embedded or connected)
    // 2. Send model_request messages to kernel
    // 3. Format and display responses
}
```

### Design Doc Update Required
**Section to add**: "5.2 CLI Command Implementation"
- Show complete clap command structure
- Show handler implementation pattern (similar to tool.rs)
- Explain ExecutionContext usage (embedded for quick status checks)
- Show how progress streaming works for `llmspell model pull`

---

## 7. CANDLE INTEGRATION ARCHITECTURE GAP

### Design Doc Shows (phase-11-design-doc.md:347-355)
```rust
pub struct CandleProvider {
    model: CandleModel,
    tokenizer: Tokenizer,
    config: LocalProviderConfig,
}
```

### The Problem
1. **Model loading lifecycle unclear**: When does GGUF load? On provider init or first inference?
2. **Memory management**: 4GB+ models - need lazy loading? Model unloading?
3. **GPU detection**: How does "device: auto" get resolved to CUDA/Metal/CPU?
4. **Quantization formats**: Q4_K_M vs Q5_K_M - where's the mapping?

### Missing Architecture Details

#### Model Loading Pipeline
```rust
// NEEDED: Show complete loading flow
impl CandleProvider {
    pub async fn new(config: ProviderConfig) -> Result<Self> {
        // 1. Resolve model_directory from config
        let model_path = resolve_model_path(&config)?;

        // 2. Check if GGUF file exists, if not -> error (don't auto-download in provider)
        if !model_path.exists() {
            return Err(Error::ModelNotFound {
                model: config.model,
                hint: "Run `llmspell model pull candle/model` first"
            });
        }

        // 3. Detect device (CUDA > Metal > CPU)
        let device = detect_device(&config)?;

        // 4. Load GGUF with mmap for memory efficiency
        let model = CandleModel::from_gguf_mmap(&model_path, device)?;

        // 5. Load tokenizer (separate file or embedded in GGUF?)
        let tokenizer = Tokenizer::from_file(&tokenizer_path)?;

        Ok(Self { model, tokenizer, config, device })
    }
}
```

#### Inference Loop
```rust
impl ProviderInstance for CandleProvider {
    async fn complete(&self, input: &AgentInput) -> Result<AgentOutput> {
        // 1. Tokenize input
        let tokens = self.tokenizer.encode(&input.content)?;

        // 2. Run inference (blocking - use spawn_blocking?)
        let output_tokens = tokio::task::spawn_blocking({
            let model = self.model.clone();
            move || model.generate(tokens, GenerateConfig::default())
        }).await??;

        // 3. Decode output
        let text = self.tokenizer.decode(&output_tokens)?;

        Ok(AgentOutput { content: text, ... })
    }
}
```

### Design Doc Update Required
**Section to add**: "11.2.3 Candle Provider Implementation Details"
- Model loading lifecycle and lazy loading strategy
- Device detection logic (CUDA > Metal > CPU fallback)
- Memory management (mmap usage, model unloading)
- Tokenizer handling (separate file vs embedded)
- Quantization format mapping (Q4_K_M, Q5_K_M, etc.)
- Async/blocking boundary (spawn_blocking for CPU-intensive inference)

**Section to add**: "11.2.4 HuggingFace Download Integration"
```rust
// How does `llmspell model pull candle/mistral:7b` work?
pub async fn download_gguf_model(
    model: &str,
    variant: &str,
    quantization: &str,
    target_dir: &Path,
) -> Result<PathBuf> {
    // 1. Construct HF model URL
    let hf_repo = format!("TheBloke/{}-GGUF", model);
    let filename = format!("{}.{}.gguf", variant, quantization);

    // 2. Download with progress streaming
    // 3. Verify checksum
    // 4. Place in target_dir
}
```

---

## SUMMARY: REQUIRED DESIGN DOC UPDATES

### High-Priority Additions (MUST HAVE)

1. **Section 3.2: Config Structure Integration** (Q1: "does it account for config structure?")
   - Show exact mapping to ProviderManagerConfig HashMap
   - Explain flat vs nested approach with code examples
   - Show modified merge logic if nested structure used

2. **Section 3.3: Provider Resolution Flow** (Q2 & Q4: "how does it change llmspell-provider?" and "is local/ollama through rig?")
   - Complete flow diagram from Lua to ProviderInstance
   - ModelSpecifier parsing extensions
   - Factory registration code

3. **Section 3.4: Rig vs Direct Ollama Decision** (Q4: "is local/ollama going through rig?")
   - **RECOMMENDATION**: Use rig's Ollama support (4 hours vs 2 days work)
   - Show RigModel enum extension
   - Justify if going direct ollama-rs instead

4. **Section 3.5: ProviderManager Registration** (Q3: "should llmspell-provider still be frontend?")
   - **YES**, llmspell-provider remains frontend
   - Show WHERE local factories register (kernel init)
   - Show factory function signatures for Ollama/Candle

5. **Section 4.2: Bridge Layer Implementation** (missing Lua API details)
   - Complete LocalLLM global injection code
   - Kernel message protocol extensions
   - Async function signatures for status/list/pull

### Medium-Priority Additions (SHOULD HAVE)

6. **Section 5.2: CLI Command Implementation**
   - Complete clap command structure
   - Handler pattern (similar to tool.rs)
   - ExecutionContext usage

7. **Section 11.2.3: Candle Implementation Details**
   - Model loading lifecycle
   - Device detection logic
   - Memory management strategy
   - Async/blocking boundaries

---

## ANSWERS TO SPECIFIC QUESTIONS

### Q1: "Does the design doc account for the config structure for providers in llmspell-config?"

**NO** - Critical mismatch:
- Design shows nested `[providers.local.ollama]` structure
- Current code uses flat `HashMap<String, ProviderConfig>`
- Design doesn't explain how backend-specific fields (url, model_directory) map to ProviderConfig
- Config merge logic in lib.rs:246-334 not addressed

**Required**: Section 3.2 showing config integration approach.

### Q2: "How does adding the local models change the llmspell-provider?"

**MINIMAL CHANGES** if done right:
- Add Ollama/Candle factory registrations (no struct changes)
- Extend ModelSpecifier to parse `@backend` suffix
- Update provider routing logic in abstraction.rs:427-431
- **llmspell-provider remains the frontend** - just more factories registered

**NOT mentioned in design doc**: Factory registration code, ModelSpecifier extensions.

### Q3: "Should llmspell-provider still be the frontend for the kernel and bridge?"

**YES** - No reason to change architecture:
- ProviderManager factory pattern handles local providers fine
- Bridge continues calling ProviderManager.create_agent_from_spec()
- Kernel continues using ProviderManager.get_provider()
- **Design doc correctly keeps this, but doesn't explicitly state it**

**Required**: Section 3.5 explicitly confirming this with code examples.

### Q4: "Is local/ollama going through rig? or directly?"

**SHOULD go through rig** (design doc assumes direct):
- rig-core v0.20.0 supports Ollama natively (confirmed via web research)
- Using rig: Add RigModel::Ollama case (~4 hours work)
- Using direct ollama-rs: New provider implementation (~2 days work)
- **Rig approach is 10x faster and maintains consistency**

**Design doc shows direct ollama-rs approach** - should justify this decision or switch to rig.

### Q5: "What's missing for holistic coverage?"

**7 major gaps identified above:**
1. Config structure mapping (critical)
2. Provider routing logic (critical)
3. Rig vs direct decision (architectural)
4. Factory registration code (critical)
5. Bridge layer protocol (critical)
6. CLI command structure (important)
7. Candle implementation details (important)

**Most critical**: Sections 3.2-3.5 showing config integration, provider resolution, and factory registration.

---

## RECOMMENDATION: TWO-PHASE UPDATE

### Phase 1: Critical Integration Details (1-2 days documentation)
1. Add Section 3.2: Config Structure Integration
2. Add Section 3.3: Provider Resolution Flow
3. Add Section 3.4: Rig Ollama Decision (**RECOMMEND using rig**)
4. Add Section 3.5: ProviderManager Registration
5. Add Section 4.2: Bridge Layer Implementation

**This makes the design doc implementable**.

### Phase 2: Implementation Details (ongoing during dev)
6. Add Section 5.2: CLI Command Implementation
7. Add Section 11.2.3: Candle Implementation Details

**This provides complete reference during implementation**.

---

## FINAL VERDICT

**Design doc has excellent feature design but lacks critical integration details**. The 5 gaps from Phase 1 make it impossible to implement without making major architectural decisions during coding. Adding Sections 3.2-3.5 and 4.2 will make the design doc truly "holistic" with "full coverage of existing code touch points" as requested.

**Key insight**: rig already supports Ollama - using it reduces Phase 11.1 from 10 days to ~3 days (RigModel enum extension vs full provider implementation).
