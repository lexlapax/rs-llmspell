# Phase 11: Local LLM Integration - Design Document

**Version**: 1.1
**Date**: October 2025
**Status**: Implemented (with noted deviations)
**Phase**: 11.1-11.2 (Local LLM Support) ✅ COMPLETE
**Timeline**: Weeks 37-41 (20 working days - both Ollama and Candle)
**Priority**: CRITICAL (Enables offline and cost-free LLM operations)
**Dependencies**: Phase 10 Service Integration ✅
**Research Archive**: `/LOCAL-LLM-ANALYSIS.md`

> **📋 Local LLM Foundation**: This phase implements comprehensive local LLM support via both Ollama (external process) and Candle (embedded Rust). Provides unified model management, cost-free inference, and offline operation capabilities.

---

## Phase Overview

### Goal
Implement dual-path local LLM support: **Ollama** for ease of use and mature model ecosystem, **Candle** for embedded deployment and zero external dependencies. Provide unified model management CLI, consistent script API, and seamless provider switching.

### Core Principles
- **Dual Implementation**: Both Ollama and Candle support in single phase
- **Unified UX**: Same script API regardless of backend (Ollama vs Candle)
- **Model Agnostic**: Support LLaMA 3.1, Mistral, Phi-3, Gemma 2 via both paths
- **CLI-First**: Model download/install managed through `llmspell model` subcommand
- **Provider Abstraction**: Extend existing ProviderInstance trait for local providers
- **Configuration-Driven**: Switch providers via llmspell.toml or script
- **Performance Critical**: <100ms first-token latency for 7B models on modern hardware
- **Memory Efficient**: Quantized models (Q4_K_M) fit in <5GB RAM
- **Production Ready**: Automatic fallback, health checks, resource management

### Implementation Strategy
**Phase 11.1** (Days 1-10): Ollama Integration
- Fast path to local LLM (2-3 days implementation)
- Uses mature Ollama ecosystem
- External process dependency
- Rich model library via `ollama pull`

**Phase 11.2** (Days 11-20): Candle Integration
- Pure Rust embedded inference
- No external dependencies
- Manual model management
- Library mode ready

### Success Criteria - ACTUAL IMPLEMENTATION
- [x] Research completed on both Ollama and Candle approaches ✅
- [x] `llmspell model list` shows available local models (both Ollama and Candle) ✅
- [x] `llmspell model pull ollama/llama3.1:8b` downloads via Ollama ✅
- [x] `llmspell model pull candle/mistral:7b` downloads GGUF for Candle ✅
- [x] LocalLLM.list() works from Lua ✅ (alternative API)
- [x] LocalLLM.pull() works from Lua ✅ (alternative API)
- [x] Scripts use `model = "local/llama3.1:8b"` syntax (backend auto-detected) ✅
- [x] Ollama provider functional with streaming support ✅ (via rig)
- [x] Candle provider functional with GGUF loading ✅ (HuggingFace + tokenizer fallback)
- [x] Chat template support for TinyLlama-Chat and similar models ✅
- [x] Examples demonstrate both Ollama and Candle usage ✅ (4 Lua examples, 260 lines)
- [x] Comprehensive test coverage for both providers ✅ (10 integration tests: 5 Ollama + 5 Candle)
- [x] Documentation covers installation, configuration, and usage ✅ (320-line user guide)
- [x] CLI `llmspell model` commands ✅ **IMPLEMENTED** (468 lines, dual-mode: embedded + remote)

**Note**: Phase 11 delivered BOTH CLI commands AND LocalLLM Lua global for maximum flexibility. Users can manage models via CLI or from scripts.

---

## 0. Implementation Summary (Phase 11 Complete)

### What Was Delivered

**Core Providers:**
- ✅ Ollama integration via rig (llmspell-providers/src/local/ollama/)
- ✅ Candle GGUF inference (llmspell-providers/src/local/candle/)
- ✅ LocalProviderInstance trait with list, pull, info, status methods
- ✅ ModelSpecifier parsing with @ollama/@candle backend syntax
- ✅ Provider factory integration (register_local_providers)
- ✅ Chat template formatting for TinyLlama-Chat models
- ✅ HuggingFace model downloads with tokenizer fallback

**CLI Commands:**
- ✅ `llmspell model list` - List installed models (468 lines implementation)
- ✅ `llmspell model pull` - Download models via Ollama or Candle
- ✅ `llmspell model remove` - Delete local models
- ✅ `llmspell model info` - Show model details
- ✅ `llmspell model available` - List available models from libraries
- ✅ `llmspell model status` - Check backend health
- ✅ `llmspell model install-ollama` - Install Ollama binary
- ✅ Dual-mode handlers (embedded kernel + remote kernel)
- ✅ Kernel message protocol (model_request/model_reply)

**Script API (Lua):**
- ✅ LocalLLM global with 4 methods (list, pull, info, status)
- ✅ Agent.create() accepts `model = "local/llama3.1:8b@ollama"` syntax
- ✅ Backend auto-detection (prefers Ollama if available)

**Testing:**
- ✅ 10 integration tests (5 Ollama + 5 Candle with RUN_EXPENSIVE_TESTS guard)
- ✅ All tests passing with proper model directory handling
- ✅ Zero compiler/clippy warnings in Phase 11 code

**Documentation:**
- ✅ User guide: 320 lines covering both backends (docs/user-guide/local-llm.md)
- ✅ 4 Lua examples: status, chat, comparison, model info (260 lines total)
- ✅ 6 troubleshooting scenarios documented
- ✅ API documentation: cargo doc builds with zero warnings

**Performance:**
- ✅ Ollama: Functional via rig REST API integration
- ✅ Candle: GGUF loading with proper tensor handling
- ✅ Memory-efficient Q4_K_M quantization support

### Key Implementation Differences from Design

1. **CLI Commands (Section 4) - ✅ FULLY IMPLEMENTED:**
   - Design proposed: `llmspell model list`, `llmspell model pull`, etc.
   - Actually delivered: Complete CLI implementation (468 lines) + LocalLLM Lua global
   - **CORRECTION**: Initial documentation incorrectly stated this was deferred
   - Reality: Phase 11 delivered BOTH interfaces for maximum user flexibility
   - Implementation: llmspell-cli/src/commands/model.rs with dual-mode handlers

2. **Kernel Protocol Extension - ✅ FULLY IMPLEMENTED:**
   - Design proposed: ModelRequest/ModelReply message types
   - Actually delivered: Complete kernel protocol implementation
   - **CORRECTION**: Initial documentation incorrectly stated this was not needed
   - Reality: Kernel protocol properly extended for CLI model commands
   - Implementation: llmspell-kernel/src/execution/integrated.rs:2502 (handle_model_request)

3. **Candle Implementation:**
   - Design: Complex LogitsProcessor with temperature/top_p/top_k sampling
   - Actual: Simplified inference with chat template formatting focus
   - Added: TinyLlama-Chat template formatting (not in original design)
   - Added: Tokenizer fallback mechanism for HuggingFace models

4. **Test Structure:**
   - Design: Separate unit tests + integration tests
   - Actual: 10 comprehensive integration tests covering full stack
   - Guards: RUN_EXPENSIVE_TESTS for model downloads, OLLAMA_AVAILABLE for service checks

### Validated Architecture Decisions

1. ✅ **Flat Provider Config Structure**: HashMap<String, ProviderConfig> works perfectly
2. ✅ **Rig for Ollama**: Confirmed working, no need for direct ollama-rs
3. ✅ **Backend Resolution**: ModelSpecifier.backend field enables @ollama/@candle syntax
4. ✅ **Factory Pattern**: Existing ProviderManager handles local providers seamlessly
5. ✅ **LocalProviderInstance Trait**: Clean extension of ProviderInstance for model management
6. ✅ **Kernel Message Protocol**: model_request/model_reply properly integrated
7. ✅ **Dual-Mode CLI**: Embedded + remote kernel handlers follow tool.rs pattern perfectly

### Phase 11 Timeline

- **Planned**: 20 working days (10 Ollama + 10 Candle)
- **Actual**: Completed within planned timeline
- **Testing**: 11.8 comprehensive integration testing
- **Documentation**: 11.9 complete with user guide + examples

---

## 1. Architecture Overview

### 1.1 Integration with Existing Provider System

**Key Decision**: Use existing `ProviderManager` factory pattern - NO new ProviderCategory enum needed.

Local providers register as factories alongside cloud providers. The existing `ProviderManager` (llmspell-providers/src/abstraction.rs:257-567) handles both cloud and local providers uniformly via the factory pattern.

**Existing Architecture** (unchanged):
```rust
// llmspell-providers/src/abstraction.rs (EXISTING CODE)
pub struct ProviderManager {
    registry: Arc<RwLock<ProviderRegistry>>,
    instances: Arc<RwLock<ProviderInstanceMap>>,
    default_provider: Arc<RwLock<Option<String>>>,
}

impl ProviderManager {
    // Register factory by name
    pub async fn register_provider<F>(&self, name: impl Into<String>, factory: F)
    where F: Fn(ProviderConfig) -> Result<Box<dyn ProviderInstance>, LLMSpellError>
        + Send + Sync + 'static
    { /* ... */ }

    // Create from ModelSpecifier
    pub async fn create_agent_from_spec(
        &self,
        spec: ModelSpecifier,
        base_url_override: Option<&str>,
        api_key: Option<&str>,
    ) -> Result<Arc<Box<dyn ProviderInstance>>, LLMSpellError>
    { /* ... */ }
}
```

**What Changes**: Just register more factories during kernel initialization.

```rust
// llmspell-kernel initialization (NEW CODE - will be added)
pub async fn register_local_providers(
    provider_manager: &ProviderManager,
    config: &LLMSpellConfig,
) -> Result<()> {
    // Existing: cloud providers via rig
    provider_manager.register_provider("rig", create_rig_provider).await;

    // NEW: Local provider factories
    if config.providers.get("ollama").map_or(false, |c| c.enabled) {
        provider_manager.register_provider("ollama", |config| {
            // Uses rig's Ollama support (NOT direct ollama-rs)
            create_rig_provider(config)  // rig handles Ollama too!
        }).await;
    }

    if config.providers.get("candle").map_or(false, |c| c.enabled) {
        provider_manager.register_provider("candle", |config| {
            Ok(Box::new(CandleProvider::new(config)?))
        }).await;
    }

    Ok(())
}
```

### 1.2 Local Provider Trait Extension

**Extends** existing ProviderInstance trait with model management methods:

```rust
// llmspell-providers/src/local/mod.rs (NEW FILE)

#[async_trait]
pub trait LocalProviderInstance: ProviderInstance {
    /// Check if backend is available/healthy
    async fn health_check(&self) -> Result<HealthStatus>;

    /// List locally available models
    async fn list_local_models(&self) -> Result<Vec<LocalModel>>;

    /// Pull/download a model
    async fn pull_model(&self, model_spec: &ModelSpec) -> Result<PullProgress>;

    /// Get model information
    async fn model_info(&self, model_id: &str) -> Result<ModelInfo>;

    /// Unload model from memory (if applicable)
    async fn unload_model(&self, model_id: &str) -> Result<()>;
}
```

### 1.3 Model Specification Syntax & Parsing

**Extends** existing `ModelSpecifier` (llmspell-providers/src/model_specifier.rs) with backend field:

```rust
// CURRENT ModelSpecifier (EXISTING CODE - llmspell-providers/src/model_specifier.rs)
pub struct ModelSpecifier {
    pub provider: Option<String>,  // e.g., "openai", "local"
    pub model: String,              // e.g., "gpt-4", "llama3.1:8b"
    pub base_url: Option<String>,
}

// EXTENDED ModelSpecifier (NEW CODE - adds backend field)
pub struct ModelSpecifier {
    pub provider: Option<String>,      // e.g., "local"
    pub model: String,                  // e.g., "llama3.1:8b"
    pub backend: Option<String>,        // e.g., "ollama" or "candle" ← NEW
    pub base_url: Option<String>,
}

impl ModelSpecifier {
    pub fn parse(spec: &str) -> Result<Self> {
        // Parse formats:
        // "local/llama3.1:8b" -> provider="local", model="llama3.1:8b", backend=None
        // "local/llama3.1:8b@ollama" -> provider="local", model="llama3.1:8b", backend=Some("ollama")
        // "llama3.1:8b@candle" -> provider=None, model="llama3.1:8b", backend=Some("candle")

        // Split on @ to extract backend
        let (model_part, backend) = if let Some(idx) = spec.rfind('@') {
            (&spec[..idx], Some(spec[idx + 1..].to_string()))
        } else {
            (spec, None)
        };

        // Existing logic to parse provider/model from model_part
        // ...

        Ok(Self { provider, model, backend, base_url: None })
    }
}
```

**Syntax Examples**:
```
Format: [<provider>/]<model>[:<variant>][@<backend>]

- local/llama3.1:8b           # Auto-detect backend (prefers Ollama)
- local/llama3.1:8b@ollama    # Force Ollama backend
- local/mistral:7b@candle     # Force Candle backend
- local/phi3:3.8b             # Auto-detect (Phi-3 mini)
- llama3.1:8b@ollama          # Ollama without "local/" prefix
```

**Backend Resolution** (in abstraction.rs:create_agent_from_spec):
```rust
// MODIFIED code in llmspell-providers/src/abstraction.rs (lines 427-431)
let implementation_name = match provider_name.as_str() {
    "openai" | "anthropic" | "cohere" | "groq" | "perplexity" | "together"
    | "gemini" | "mistral" | "replicate" | "fireworks" => "rig",

    // NEW: Local provider routing
    "local" => {
        // Resolve backend from spec or config default_backend
        spec.backend.as_deref()
            .or_else(|| {
                // Read from config.providers["ollama"].options["default_backend"]
                config.providers.get("ollama")
                    .and_then(|c| c.options.get("default_backend"))
                    .and_then(|v| v.as_str())
            })
            .unwrap_or("ollama")  // Final fallback
    }

    other => other,
};
```

### 1.4 Configuration Schema - Flat Structure

**Key Decision**: Use FLAT provider keys, not nested, to work with existing `HashMap<String, ProviderConfig>` (llmspell-config/src/providers.rs:20).

**Existing Config Structure** (unchanged):
```rust
// llmspell-config/src/providers.rs (EXISTING CODE - line 12-21)
pub struct ProviderManagerConfig {
    pub default_provider: Option<String>,
    pub providers: HashMap<String, ProviderConfig>,  // ← FLAT HashMap
}

// llmspell-config/src/providers.rs (EXISTING CODE - line 104-139)
pub struct ProviderConfig {
    pub name: String,
    pub provider_type: String,  // "ollama", "candle", etc.
    pub enabled: bool,
    pub base_url: Option<String>,
    pub api_key: Option<String>,
    pub default_model: Option<String>,
    pub max_tokens: Option<u32>,
    pub timeout_seconds: Option<u64>,
    pub options: HashMap<String, serde_json::Value>,  // ← Backend-specific fields
}
```

**Configuration File** (FLAT structure):
```toml
# llmspell.toml - Local LLM configuration

# Ollama provider (uses rig's Ollama support)
[providers.ollama]
provider_type = "ollama"
enabled = true
base_url = "http://localhost:11434"
timeout_seconds = 120

# Ollama-specific settings go in options
[providers.ollama.options]
auto_start = true
health_check_interval_seconds = 60
default_backend = "ollama"  # Used by backend resolution logic

# Candle provider (pure Rust embedded)
[providers.candle]
provider_type = "candle"
enabled = true
timeout_seconds = 300  # Longer for model loading

# Candle-specific settings in options (accessed via ProviderConfig.options)
[providers.candle.options]
model_directory = "${HOME}/.llmspell/models/candle"
device = "auto"  # "cpu", "cuda", "metal", or "auto"
max_concurrent = 1
default_quantization = "Q4_K_M"
cpu_threads = 0  # 0 = auto-detect
context_size = 4096
batch_size = 512
use_flash_attention = true

# Recommended models (documentation only, not parsed into ProviderConfig)
[providers.local_recommended_models]
phi3_mini = { model = "phi3:3.8b", backend = "ollama", size_gb = 2.4 }
mistral = { model = "mistral:7b", backend = "both", size_gb = 4.1 }
llama3_1 = { model = "llama3.1:8b", backend = "both", size_gb = 4.7 }
gemma2 = { model = "gemma2:9b", backend = "ollama", size_gb = 5.4 }
```

**How Backend-Specific Fields Are Accessed**:
```rust
// In OllamaProvider::new() or CandleProvider::new()
impl CandleProvider {
    pub fn new(config: ProviderConfig) -> Result<Self> {
        // Extract candle-specific fields from options HashMap
        let model_directory = config.options.get("model_directory")
            .and_then(|v| v.as_str())
            .map(|s| PathBuf::from(s))
            .unwrap_or_else(|| {
                dirs::home_dir().unwrap().join(".llmspell/models/candle")
            });

        let device = config.options.get("device")
            .and_then(|v| v.as_str())
            .unwrap_or("auto");

        let cpu_threads = config.options.get("cpu_threads")
            .and_then(|v| v.as_u64())
            .unwrap_or(0) as usize;

        // ... rest of initialization
    }
}
```

**Why Flat Structure?**:
1. Works with existing `HashMap<String, ProviderConfig>` without changes
2. No modifications needed to config merge logic (llmspell-config/src/lib.rs:246-334)
3. Backend-specific fields stored in `ProviderConfig.options` (already exists)
4. Consistent with existing cloud provider configuration pattern

---

## 2. Phase 11.1: Ollama Integration via Rig

**Key Decision**: Use rig's native Ollama support (rig-core v0.20.0+) instead of direct ollama-rs integration.

**Rationale**:
1. **Rig already supports Ollama** - among 20+ providers (confirmed via web research)
2. **4 hours vs 2 days** - Just add enum case to existing RigModel vs new provider implementation
3. **Consistency** - Same retry/timeout/streaming logic as cloud providers
4. **Maintenance** - Less code to maintain, leverages rig updates

### 2.1 Extend Existing RigProvider

**Modify** existing `llmspell-providers/src/rig.rs` to add Ollama support:

```rust
// llmspell-providers/src/rig.rs (EXISTING FILE - lines 17-22 MODIFIED)

// CURRENT enum (EXISTING CODE):
enum RigModel {
    OpenAI(providers::openai::CompletionModel),
    Anthropic(providers::anthropic::completion::CompletionModel),
    Cohere(providers::cohere::CompletionModel),
}

// EXTENDED enum (NEW CODE - add Ollama variant):
enum RigModel {
    OpenAI(providers::openai::CompletionModel),
    Anthropic(providers::anthropic::completion::CompletionModel),
    Cohere(providers::cohere::CompletionModel),
    Ollama(providers::ollama::CompletionModel),  // ← NEW
}

// MODIFIED create_rig_provider (lines 42-98) - add Ollama case:
pub fn create_rig_provider(config: ProviderConfig) -> Result<Box<dyn ProviderInstance>, LLMSpellError> {
    let model = match config.provider_type.as_str() {
        "openai" => {
            let api_key = config.api_key.as_ref().ok_or_else(...)?;
            let client = providers::openai::Client::new(api_key);
            let model = client.completion_model(&config.model);
            RigModel::OpenAI(model)
        }
        "anthropic" => { /* existing code */ }
        "cohere" => { /* existing code */ }

        // NEW: Ollama provider via rig
        "ollama" => {
            let base_url = config.base_url.as_deref()
                .or_else(|| config.options.get("base_url").and_then(|v| v.as_str()))
                .unwrap_or("http://localhost:11434");

            info!("Creating Ollama provider via rig: base_url={}", base_url);
            debug!("Ollama model: {}", config.model);

            let client = providers::ollama::Client::new(base_url);
            let model = client.completion_model(&config.model);
            RigModel::Ollama(model)
        }

        _ => {
            return Err(LLMSpellError::Configuration {
                message: format!("Unsupported provider type: {}", config.provider_type),
                source: None,
            });
        }
    };

    Ok(Box::new(RigProvider {
        model,
        config,
        capabilities: ProviderCapabilities::default(),
    }))
}

// ProviderInstance implementation already handles all RigModel variants via delegation
// No changes needed to complete/complete_streaming/etc.
```

**That's it!** ~20 lines of code to add Ollama support. Rig handles the rest (HTTP client, retry logic, streaming, error handling).

### 2.2 Model Management Layer (Uses ollama-rs)

**Hybrid Approach**: Rig for inference, direct ollama-rs for model management operations (pull, list, info).

```rust
// llmspell-providers/src/local/ollama_manager.rs (NEW FILE)

use ollama_rs::Ollama;  // Direct ollama-rs for model management only

/// Ollama model manager - handles pull/list/info operations
/// NOT used for inference (rig handles that)
pub struct OllamaModelManager {
    client: Ollama,
    base_url: String,
}

impl OllamaModelManager {
    pub fn new(base_url: &str) -> Self {
        Self {
            client: Ollama::new(base_url.to_string(), 11434),
            base_url: base_url.to_string(),
        }
    }

    /// List models installed locally
    pub async fn list_local_models(&self) -> Result<Vec<LocalModel>> {
        let models = self.client.list_local_models().await?;

        Ok(models.into_iter().map(|m| LocalModel {
            id: m.name,
            backend: LocalBackend::Ollama,
            size_bytes: m.size,
            modified_at: m.modified_at,
            format: ModelFormat::Ollama,
            quantization: None,  // Ollama handles internally
        }).collect())
    }

    /// Pull/download a model
    pub async fn pull_model(&self, model_id: &str) -> Result<PullProgress> {
        info!("Pulling Ollama model: {}", model_id);

        self.client.pull_model(model_id.to_string()).await?;

        Ok(PullProgress {
            model_id: model_id.to_string(),
            status: PullStatus::Complete,
            downloaded_bytes: 0,
            total_bytes: 0,
            percent_complete: 100.0,
        })
    }

    /// Check Ollama health
    pub async fn health_check(&self) -> Result<HealthStatus> {
        match self.client.list_local_models().await {
            Ok(models) => Ok(HealthStatus::Healthy {
                available_models: models.len(),
                version: Some("unknown".to_string()),
                uptime_seconds: None,
            }),
            Err(_) => Ok(HealthStatus::Unhealthy {
                reason: "Ollama not responding".to_string(),
                retry_after_seconds: Some(30),
            }),
        }
    }

    /// List available models in library (hardcoded for now)
    pub async fn list_library_models(&self) -> Result<Vec<LibraryModel>> {
        Ok(vec![
            LibraryModel {
                name: "llama3.1:8b".to_string(),
                description: "Meta LLaMA 3.1 8B".to_string(),
                size_gb: 4.7,
                parameter_count: "8B".to_string(),
                recommended: true,
            },
            LibraryModel {
                name: "mistral:7b".to_string(),
                description: "Mistral 7B v0.3".to_string(),
                size_gb: 4.1,
                parameter_count: "7B".to_string(),
                recommended: true,
            },
            LibraryModel {
                name: "phi3:3.8b".to_string(),
                description: "Microsoft Phi-3 Mini".to_string(),
                size_gb: 2.4,
                parameter_count: "3.8B".to_string(),
                recommended: true,
            },
            LibraryModel {
                name: "gemma2:9b".to_string(),
                description: "Google Gemma 2 9B".to_string(),
                size_gb: 5.4,
                parameter_count: "9B".to_string(),
                recommended: true,
            },
        ])
    }
}

#[derive(Debug, Clone)]
pub enum InstallationStatus {
    Installed,
    NotRunning,
    NotInstalled,
}

#[derive(Debug, Clone)]
pub struct LibraryModel {
    pub name: String,
    pub description: String,
    pub size_gb: f32,
    pub parameter_count: String,
    pub recommended: bool,
}
```

---

## 3. Phase 11.2: Candle Integration

### 3.1 Candle Provider Implementation

```rust
// llmspell-providers/src/local/candle.rs

use candle_core::{Device, Tensor};
use candle_transformers::models::quantized_llama as llama;
use tokenizers::Tokenizer;
use std::path::PathBuf;

/// Candle provider for embedded GGUF inference
pub struct CandleProvider {
    config: CandleConfig,
    device: Device,
    models: Arc<RwLock<HashMap<String, LoadedModel>>>,
    model_directory: PathBuf,
}

/// Loaded model in memory
struct LoadedModel {
    model: llama::ModelWeights,
    tokenizer: Tokenizer,
    config: llama::Config,
    generation_config: GenerationConfig,
}

/// Generation configuration
#[derive(Debug, Clone)]
struct GenerationConfig {
    temperature: f32,
    top_p: f32,
    top_k: usize,
    repeat_penalty: f32,
    repeat_last_n: usize,
}

impl Default for GenerationConfig {
    fn default() -> Self {
        Self {
            temperature: 0.8,
            top_p: 0.95,
            top_k: 50,
            repeat_penalty: 1.1,
            repeat_last_n: 64,
        }
    }
}

impl CandleProvider {
    /// Create new Candle provider
    pub fn new(config: CandleConfig) -> Result<Self> {
        info!("Initializing Candle provider: device={}, quantization={}",
            config.device, config.default_quantization);
        debug!("Candle config: max_concurrent={}, cpu_threads={}, model_dir={:?}",
            config.max_concurrent,
            config.cpu_threads,
            config.model_directory
        );

        // Detect best device
        let device = match config.device.as_str() {
            "cuda" => {
                info!("Using CUDA device for Candle inference");
                Device::cuda_if_available(0)?
            }
            "metal" => {
                info!("Using Metal device for Candle inference");
                Device::new_metal(0)?
            }
            "cpu" => {
                info!("Using CPU device for Candle inference (threads: {})", config.cpu_threads);
                Device::Cpu
            }
            "auto" => {
                if Device::cuda_if_available(0).is_ok() {
                    info!("Auto-detected CUDA device for Candle");
                    Device::cuda_if_available(0)?
                } else if Device::new_metal(0).is_ok() {
                    info!("Auto-detected Metal device for Candle");
                    Device::new_metal(0)?
                } else {
                    info!("Auto-detected CPU device for Candle (no GPU available)");
                    Device::Cpu
                }
            }
            _ => {
                warn!("Unknown device '{}', defaulting to CPU", config.device);
                Device::Cpu
            }
        };

        debug!("Candle provider using device: {:?}", device);

        let model_directory = config.model_directory.clone()
            .unwrap_or_else(|| {
                dirs::home_dir()
                    .unwrap()
                    .join(".llmspell")
                    .join("models")
                    .join("candle")
            });

        // Ensure model directory exists
        std::fs::create_dir_all(&model_directory)?;

        Ok(Self {
            config,
            device,
            models: Arc::new(RwLock::new(HashMap::new())),
            model_directory,
        })
    }

    /// Load a GGUF model
    async fn load_model(&self, model_id: &str) -> Result<()> {
        // Check if already loaded
        if self.models.read().await.contains_key(model_id) {
            debug!("Model {} already loaded, skipping", model_id);
            return Ok(());
        }

        info!("Loading GGUF model: {}", model_id);
        let start = std::time::Instant::now();

        // Find model files
        let model_path = self.model_directory.join(model_id);
        debug!("Searching for GGUF file in: {:?}", model_path);

        let gguf_path = self.find_gguf_file(&model_path)?;
        let tokenizer_path = self.find_tokenizer_file(&model_path)?;

        info!("Loading GGUF model from: {:?}", gguf_path);
        debug!("GGUF file size: {} bytes", std::fs::metadata(&gguf_path)?.len());
        info!("Loading tokenizer from: {:?}", tokenizer_path);

        // Load tokenizer
        trace!("Loading tokenizer from file");
        let tokenizer = Tokenizer::from_file(&tokenizer_path)
            .map_err(|e| {
                error!("Failed to load tokenizer from {:?}: {}", tokenizer_path, e);
                anyhow!("Failed to load tokenizer: {}", e)
            })?;
        debug!("Tokenizer loaded: vocab_size={}", tokenizer.get_vocab_size(true));

        // Load GGUF model
        trace!("Opening GGUF file for reading");
        let mut file = std::fs::File::open(&gguf_path)?;

        debug!("Parsing GGUF file with Candle");
        let model_content = candle_transformers::quantized_var_builder::VarBuilder::from_gguf(
            &mut file,
            &self.device
        ).map_err(|e| {
            error!("Failed to parse GGUF file: {}", e);
            e
        })?;

        // Create model config from GGUF metadata
        debug!("Extracting model configuration from GGUF metadata");
        let config = llama::Config::default(); // TODO: Extract from GGUF
        trace!("Model config: {:?}", config);

        // Load model weights
        info!("Loading model weights from GGUF");
        let model = llama::ModelWeights::from_gguf(model_content, &config)
            .map_err(|e| {
                error!("Failed to load model weights: {}", e);
                e
            })?;

        let model_size_mb = std::fs::metadata(&gguf_path)?.len() / (1024 * 1024);
        debug!("Model weights loaded: ~{}MB", model_size_mb);

        // Store loaded model
        self.models.write().await.insert(
            model_id.to_string(),
            LoadedModel {
                model,
                tokenizer,
                config,
                generation_config: GenerationConfig::default(),
            }
        );

        info!("Model {} loaded successfully in {:?} (~{}MB)",
            model_id, start.elapsed(), model_size_mb);
        Ok(())
    }

    /// Find GGUF file in model directory
    fn find_gguf_file(&self, model_path: &PathBuf) -> Result<PathBuf> {
        for entry in std::fs::read_dir(model_path)? {
            let entry = entry?;
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("gguf") {
                return Ok(path);
            }
        }
        Err(anyhow!("No GGUF file found in {:?}", model_path))
    }

    /// Find tokenizer file
    fn find_tokenizer_file(&self, model_path: &PathBuf) -> Result<PathBuf> {
        let tokenizer_json = model_path.join("tokenizer.json");
        if tokenizer_json.exists() {
            return Ok(tokenizer_json);
        }
        Err(anyhow!("No tokenizer.json found in {:?}", model_path))
    }

    /// Generate text with loaded model
    async fn generate_with_model(
        &self,
        model_id: &str,
        prompt: &str,
        params: &GenerationParams,
    ) -> Result<String> {
        debug!("Starting Candle generation: model={}, prompt_len={}, max_tokens={:?}",
            model_id, prompt.len(), params.max_tokens);
        trace!("Generation params: temp={}, top_p={}, top_k={}",
            params.temperature, params.top_p, params.top_k);

        let start = std::time::Instant::now();

        // Ensure model is loaded
        self.load_model(model_id).await?;

        let models = self.models.read().await;
        let loaded = models.get(model_id)
            .ok_or_else(|| {
                error!("Model {} not found in loaded models", model_id);
                anyhow!("Model {} not loaded", model_id)
            })?;

        // Tokenize prompt
        trace!("Tokenizing prompt");
        let encoding = loaded.tokenizer.encode(prompt, true)
            .map_err(|e| {
                error!("Tokenization failed for model {}: {}", model_id, e);
                anyhow!("Tokenization failed: {}", e)
            })?;
        let tokens = encoding.get_ids();
        debug!("Prompt tokenized: {} tokens", tokens.len());

        // Convert to tensor
        let input_tensor = Tensor::new(tokens, &self.device)?
            .unsqueeze(0)?; // Add batch dimension

        // Generate tokens
        let mut generated_tokens = Vec::new();
        let mut logits_processor = LogitsProcessor::new(
            params.temperature,
            params.top_p,
            params.top_k,
        );

        let max_tokens = params.max_tokens.unwrap_or(512);
        info!("Generating up to {} tokens with Candle", max_tokens);

        let mut token_count = 0;
        for i in 0..max_tokens {
            trace!("Generating token {}/{}", i + 1, max_tokens);

            // Forward pass
            let logits = loaded.model.forward(&input_tensor, 0)?;

            // Sample next token
            let next_token = logits_processor.sample(&logits)?;
            generated_tokens.push(next_token);
            token_count += 1;

            // Check for EOS
            if next_token == loaded.tokenizer.token_to_id("<|endoftext|>").unwrap_or(0) {
                debug!("EOS token generated at position {}", i + 1);
                break;
            }

            // Update input for next iteration
            // input_tensor = ... (append next_token)
        }

        let elapsed = start.elapsed();
        info!("Generated {} tokens in {:?} ({:.1} tokens/sec)",
            token_count,
            elapsed,
            token_count as f64 / elapsed.as_secs_f64()
        );

        // Decode tokens to text
        trace!("Decoding {} tokens to text", generated_tokens.len());
        let text = loaded.tokenizer.decode(&generated_tokens, true)
            .map_err(|e| {
                error!("Token decoding failed: {}", e);
                anyhow!("Decoding failed: {}", e)
            })?;

        debug!("Generated text length: {} chars", text.len());

        Ok(text)
    }

    /// Unload model from memory
    async fn unload_model(&self, model_id: &str) -> Result<()> {
        info!("Unloading Candle model: {}", model_id);

        if self.models.write().await.remove(model_id).is_some() {
            info!("Successfully unloaded model {}", model_id);
        } else {
            debug!("Model {} was not loaded, nothing to unload", model_id);
        }

        Ok(())
    }
}

#[async_trait]
impl ProviderInstance for CandleProvider {
    async fn complete(&self, input: AgentInput) -> Result<AgentOutput> {
        let model_id = input.parameters.get("model")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow!("No model specified for Candle provider"))?;

        // Extract generation parameters
        let params = GenerationParams {
            temperature: input.parameters.get("temperature")
                .and_then(|v| v.as_f64())
                .map(|f| f as f32)
                .unwrap_or(0.8),
            top_p: input.parameters.get("top_p")
                .and_then(|v| v.as_f64())
                .map(|f| f as f32)
                .unwrap_or(0.95),
            top_k: input.parameters.get("top_k")
                .and_then(|v| v.as_u64())
                .map(|u| u as usize)
                .unwrap_or(50),
            max_tokens: input.parameters.get("max_tokens")
                .and_then(|v| v.as_i64())
                .map(|i| i as usize),
        };

        // Generate
        let start = std::time::Instant::now();
        let text = self.generate_with_model(model_id, &input.text, &params).await?;
        let latency_ms = start.elapsed().as_millis() as u64;

        Ok(AgentOutput {
            text,
            media: vec![],
            tool_calls: vec![],
            metadata: OutputMetadata {
                model: Some(model_id.to_string()),
                provider: Some("candle".to_string()),
                finish_reason: Some("stop".to_string()),
                tokens: None, // TODO: Count tokens
                latency_ms: Some(latency_ms),
                ..Default::default()
            },
            transfer_to: None,
        })
    }

    async fn complete_streaming(&self, input: AgentInput) -> Result<AgentStream> {
        // TODO: Implement streaming for Candle
        // For Phase 11, can use non-streaming with chunking
        Err(LLMSpellError::NotImplemented("Candle streaming not yet implemented".into()))
    }

    fn supports_streaming(&self) -> bool {
        false // Phase 11.2: Not implemented yet
    }

    fn capabilities(&self) -> &ProviderCapabilities {
        &self.config.capabilities
    }
}

#[async_trait]
impl LocalProviderInstance for CandleProvider {
    async fn health_check(&self) -> Result<HealthStatus> {
        Ok(HealthStatus::Healthy {
            available_models: self.models.read().await.len(),
            version: Some(env!("CARGO_PKG_VERSION").to_string()),
            uptime_seconds: None,
        })
    }

    async fn list_local_models(&self) -> Result<Vec<LocalModel>> {
        let mut models = Vec::new();

        // Scan model directory
        for entry in std::fs::read_dir(&self.model_directory)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_dir() {
                let model_id = path.file_name()
                    .and_then(|n| n.to_str())
                    .ok_or_else(|| anyhow!("Invalid model directory name"))?
                    .to_string();

                // Check for GGUF file
                if self.find_gguf_file(&path).is_ok() {
                    let metadata = std::fs::metadata(&path)?;

                    models.push(LocalModel {
                        id: model_id,
                        backend: LocalBackend::Candle,
                        size_bytes: 0, // TODO: Calculate from GGUF
                        modified_at: metadata.modified()?,
                        format: ModelFormat::GGUF,
                        quantization: Some("Q4_K_M".to_string()), // TODO: Detect from GGUF
                    });
                }
            }
        }

        Ok(models)
    }

    async fn pull_model(&self, spec: &ModelSpec) -> Result<PullProgress> {
        // For Candle, "pull" means downloading GGUF from HuggingFace
        let model_name = &spec.model;
        let variant = spec.variant.as_ref().unwrap_or(&"7b".to_string());

        // Determine HuggingFace repo
        let hf_repo = match model_name.as_str() {
            "llama3.1" => format!("meta-llama/Meta-Llama-3.1-{}B-Instruct", variant),
            "mistral" => "mistralai/Mistral-7B-Instruct-v0.3".to_string(),
            "phi3" => "microsoft/Phi-3-mini-4k-instruct".to_string(),
            _ => return Err(anyhow!("Unknown model for Candle: {}", model_name)),
        };

        // Download GGUF and tokenizer from HuggingFace
        let model_dir = self.model_directory.join(format!("{}:{}", model_name, variant));
        std::fs::create_dir_all(&model_dir)?;

        info!("Downloading GGUF model from {}", hf_repo);

        // TODO: Implement HuggingFace download
        // Use hf-hub crate to download GGUF file and tokenizer

        Ok(PullProgress {
            model_id: format!("{}:{}", model_name, variant),
            status: PullStatus::Complete,
            downloaded_bytes: 0,
            total_bytes: 0,
            percent_complete: 100.0,
        })
    }

    async fn model_info(&self, model_id: &str) -> Result<ModelInfo> {
        let models = self.models.read().await;
        let loaded = models.get(model_id)
            .ok_or_else(|| anyhow!("Model {} not loaded", model_id))?;

        Ok(ModelInfo {
            id: model_id.to_string(),
            backend: LocalBackend::Candle,
            size_bytes: None,
            parameter_count: None,
            quantization: Some(self.config.default_quantization.clone()),
            context_size: Some(loaded.config.max_seq_len),
            architecture: Some("llama".to_string()), // TODO: Detect from model
        })
    }

    async fn unload_model(&self, model_id: &str) -> Result<()> {
        self.unload_model(model_id).await
    }
}

/// Candle configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CandleConfig {
    pub model_directory: Option<PathBuf>,
    pub device: String,
    pub max_concurrent: usize,
    pub default_quantization: String,
    pub cpu_threads: usize,
    pub capabilities: ProviderCapabilities,
}

impl Default for CandleConfig {
    fn default() -> Self {
        Self {
            model_directory: None,
            device: "auto".to_string(),
            max_concurrent: 1,
            default_quantization: "Q4_K_M".to_string(),
            cpu_threads: num_cpus::get(),
            capabilities: ProviderCapabilities {
                supports_streaming: false, // Phase 11.2: Not yet
                supports_multimodal: false,
                max_context_tokens: Some(4096),
                max_output_tokens: Some(2048),
                available_models: vec![],
                custom_features: HashMap::new(),
            },
        }
    }
}

/// Logits processor for sampling
struct LogitsProcessor {
    temperature: f32,
    top_p: f32,
    top_k: usize,
}

impl LogitsProcessor {
    fn new(temperature: f32, top_p: f32, top_k: usize) -> Self {
        Self { temperature, top_p, top_k }
    }

    fn sample(&mut self, logits: &Tensor) -> Result<u32> {
        // TODO: Implement proper sampling with temperature, top-p, top-k
        // For now, simple argmax
        let logits_vec: Vec<f32> = logits.to_vec1()?;
        let max_idx = logits_vec.iter()
            .enumerate()
            .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap())
            .map(|(idx, _)| idx)
            .unwrap();

        Ok(max_idx as u32)
    }
}

#[derive(Debug, Clone)]
struct GenerationParams {
    temperature: f32,
    top_p: f32,
    top_k: usize,
    max_tokens: Option<usize>,
}
```

### 3.2 Candle Model Management

```rust
// llmspell-providers/src/local/candle_management.rs

use hf_hub::{api::sync::Api, Repo, RepoType};

/// Candle model manager
pub struct CandleModelManager {
    provider: Arc<CandleProvider>,
    hf_api: Api,
}

impl CandleModelManager {
    pub fn new(provider: Arc<CandleProvider>) -> Result<Self> {
        let hf_api = Api::new()?;
        Ok(Self { provider, hf_api })
    }

    /// Download GGUF model from HuggingFace
    pub async fn download_gguf(
        &self,
        hf_repo: &str,
        quantization: &str,
    ) -> Result<PathBuf> {
        info!("Starting HuggingFace model download: repo={}, quantization={}",
            hf_repo, quantization);

        let start = std::time::Instant::now();
        let repo = self.hf_api.repo(Repo::new(hf_repo.to_string(), RepoType::Model));

        // Determine GGUF filename based on quantization
        let gguf_filename = format!("model-{}.gguf", quantization.to_lowercase());

        info!("Downloading GGUF file: {} from {}", gguf_filename, hf_repo);
        debug!("HuggingFace repo: {}", hf_repo);

        // Download GGUF file
        let gguf_path = repo.get(&gguf_filename)
            .map_err(|e| {
                error!("Failed to download GGUF file {} from {}: {}",
                    gguf_filename, hf_repo, e);
                e
            })?;

        let gguf_size_mb = std::fs::metadata(&gguf_path)?.len() / (1024 * 1024);
        info!("Downloaded GGUF file: {:?} (~{}MB)", gguf_path, gguf_size_mb);

        // Download tokenizer
        debug!("Downloading tokenizer.json");
        let tokenizer_path = repo.get("tokenizer.json")
            .map_err(|e| {
                error!("Failed to download tokenizer from {}: {}", hf_repo, e);
                e
            })?;
        info!("Downloaded tokenizer: {:?}", tokenizer_path);

        info!("Completed model download from {} in {:?} (~{}MB total)",
            hf_repo, start.elapsed(), gguf_size_mb);

        Ok(gguf_path)
    }

    /// List available GGUF models on HuggingFace
    pub async fn list_available_models(&self) -> Result<Vec<AvailableModel>> {
        Ok(vec![
            AvailableModel {
                name: "llama3.1:8b".to_string(),
                hf_repo: "meta-llama/Meta-Llama-3.1-8B-Instruct".to_string(),
                description: "Meta LLaMA 3.1 8B Instruct".to_string(),
                size_gb: 4.7,
                quantizations: vec!["Q4_K_M".to_string(), "Q5_K_M".to_string()],
                recommended: true,
            },
            AvailableModel {
                name: "mistral:7b".to_string(),
                hf_repo: "mistralai/Mistral-7B-Instruct-v0.3".to_string(),
                description: "Mistral 7B Instruct v0.3".to_string(),
                size_gb: 4.1,
                quantizations: vec!["Q4_K_M".to_string()],
                recommended: true,
            },
            AvailableModel {
                name: "phi3:3.8b".to_string(),
                hf_repo: "microsoft/Phi-3-mini-4k-instruct".to_string(),
                description: "Microsoft Phi-3 Mini (3.8B)".to_string(),
                size_gb: 2.4,
                quantizations: vec!["Q4_K_M".to_string()],
                recommended: true,
            },
        ])
    }
}

#[derive(Debug, Clone)]
pub struct AvailableModel {
    pub name: String,
    pub hf_repo: String,
    pub description: String,
    pub size_gb: f32,
    pub quantizations: Vec<String>,
    pub recommended: bool,
}
```

---

## 4. Unified Model Management CLI ✅ **FULLY IMPLEMENTED**

> **Implementation Note**: This entire section WAS fully implemented in Phase 11.
>
> **Location**: llmspell-cli/src/commands/model.rs (468 lines)
> **Kernel Handler**: llmspell-kernel/src/execution/integrated.rs:2502
> **Protocol**: model_request/model_reply message types
>
> Additionally, LocalLLM Lua global (Section 5.1) provides script-accessible equivalent:
> - `LocalLLM.list()` - list local models
> - `LocalLLM.pull(model_spec)` - download models
> - `LocalLLM.info(model_id)` - get model information
> - `LocalLLM.status()` - check backend health
>
> **User Benefits**: Phase 11 delivered BOTH CLI and Lua API for maximum flexibility.

---

### 4.0 CLI Integration (Main Command Enum)

**Modify** existing `llmspell-cli/src/cli.rs` to add Model command:

```rust
// llmspell-cli/src/cli.rs (EXISTING FILE - add to Commands enum)

#[derive(Debug, Parser)]
pub enum Commands {
    // ... existing commands (Run, Exec, Repl, Debug, Kernel, etc.)

    /// Local model management (NEW)
    #[command(subcommand)]
    Model(ModelCommands),
}
```

**Modify** existing `llmspell-cli/src/commands/mod.rs` to handle Model:

```rust
// llmspell-cli/src/commands/mod.rs (EXISTING FILE - line 97+ in execute_command)

pub async fn execute_command(
    command: Commands,
    runtime_config: LLMSpellConfig,
    output_format: OutputFormat,
) -> Result<()> {
    match command {
        // ... existing commands

        // NEW: Model command
        Commands::Model(model_cmd) => {
            model::handle_model_command(model_cmd, runtime_config, output_format).await
        }
    }
}
```

### 4.1 CLI Structure (ModelCommands enum)

**Pattern**: Follows existing command pattern from tool.rs (dual-mode ExecutionContext support)

```rust
// llmspell-cli/src/commands/model.rs (NEW FILE)

use clap::{Args, Subcommand};
use crate::cli::OutputFormat;
use crate::execution_context::ExecutionContext;
use llmspell_config::LLMSpellConfig;

/// Model management commands
#[derive(Debug, Subcommand)]
pub enum ModelCommands {
    /// List locally installed models
    List {
        /// Filter by backend (ollama, candle, or all)
        #[arg(long, default_value = "all")]
        backend: String,

        /// Show detailed information
        #[arg(long, short)]
        verbose: bool,

        /// Output format override
        #[arg(long)]
        format: Option<OutputFormat>,
    },

    /// Pull/download a model
    Pull {
        /// Model specifier (e.g., "ollama/llama3.1:8b" or "candle/mistral:7b")
        model: String,

        /// Force re-download if already exists
        #[arg(long, short)]
        force: bool,

        /// Quantization level for Candle models
        #[arg(long, default_value = "Q4_K_M")]
        quantization: String,
    },

    /// Remove a local model
    Remove {
        /// Model identifier
        model: String,

        /// Skip confirmation
        #[arg(long, short)]
        yes: bool,
    },

    /// Show model information
    Info {
        /// Model identifier
        model: String,
    },

    /// List available models (library)
    Available {
        /// Backend to query (ollama or candle)
        #[arg(long)]
        backend: Option<String>,

        /// Filter by recommended models only
        #[arg(long)]
        recommended: bool,
    },

    /// Check local LLM installation status
    Status,

    /// Install Ollama (macOS/Linux only)
    InstallOllama,
}

/// Handle model management commands (MATCHES tool.rs PATTERN)
#[instrument(skip(runtime_config), fields(command_type))]
pub async fn handle_model_command(
    command: ModelCommands,
    runtime_config: LLMSpellConfig,
    output_format: OutputFormat,
) -> Result<()> {
    trace!("Handling model command");

    // Resolve execution context (embedded or connected kernel)
    let context = ExecutionContext::resolve(
        None, // No connect string for model commands (always local)
        None, // No port
        None, // No daemon config
        runtime_config.clone(),
    )
    .await?;

    match context {
        ExecutionContext::Embedded { handle, config } => {
            trace!("Using embedded context for model command");
            handle_model_embedded(command, handle, config, output_format).await
        }
        ExecutionContext::Connected { handle, address } => {
            trace!("Using connected context at address: {}", address);
            handle_model_remote(command, handle, address, output_format).await
        }
    }
}

### 4.2 Dual-Mode CLI Handlers (Embedded vs Remote)

**Pattern**: Matches tool.rs pattern - separate handlers for embedded and connected kernel modes

```rust
/// Handle model commands in embedded mode (kernel in same process)
async fn handle_model_embedded(
    command: ModelCommands,
    mut handle: Box<llmspell_kernel::api::KernelHandle>,
    _config: Box<LLMSpellConfig>,
    output_format: OutputFormat,
) -> Result<()> {
    trace!("Handle model embedded with command: {:?}", command);

    match command {
        ModelCommands::List { backend, verbose, format } => {
            info!("Listing models via kernel message protocol");

            // Create model_request message for list command
            let request_content = json!({
                "command": "list",
                "backend": backend,
                "verbose": verbose,
            });

            // Send request to kernel and wait for response
            let response = handle.send_model_request(request_content).await?;

            // Extract models from response
            let models = response
                .get("models")
                .and_then(|m| m.as_array())
                .map(|arr| {
                    arr.iter()
                        .filter_map(|v| v.as_object())
                        .collect::<Vec<_>>()
                })
                .unwrap_or_default();

            // Format output
            let formatter = OutputFormatter::new(format.unwrap_or(output_format));
            formatter.print_model_list(&models)?;
            Ok(())
        }

        ModelCommands::Pull { model, force, quantization } => {
            info!("Pulling model: {} via kernel", model);

            let request_content = json!({
                "command": "pull",
                "model": model,
                "force": force,
                "quantization": quantization,
            });

            let response = handle.send_model_request(request_content).await?;

            // Check for error in response
            if let Some(error) = response.get("error") {
                return Err(anyhow!("Model pull error: {}", error));
            }

            // Display progress
            if let Some(status) = response.get("status").and_then(|s| s.as_str()) {
                println!("✅ {}", status);
            }

            Ok(())
        }

        ModelCommands::Status => {
            info!("Checking local LLM status via kernel");

            let request_content = json!({
                "command": "status",
            });

            let response = handle.send_model_request(request_content).await?;

            // Format and display the response
            let formatter = OutputFormatter::new(output_format);
            formatter.print_json(&response)?;
            Ok(())
        }

        // Other commands follow same pattern...
        _ => {
            warn!("Command not yet implemented in embedded mode");
            Err(anyhow!("Command not yet implemented"))
        }
    }
}

/// Handle model commands in connected mode (remote kernel)
async fn handle_model_remote(
    command: ModelCommands,
    mut handle: llmspell_kernel::api::ClientHandle,
    address: String,
    output_format: OutputFormat,
) -> Result<()> {
    trace!("Handling model command in connected mode to {}", address);

    match command {
        ModelCommands::List { backend, verbose, format } => {
            info!("Listing models via remote kernel");

            let request_content = json!({
                "command": "list",
                "backend": backend,
                "verbose": verbose,
            });

            let response = handle.send_model_request(request_content).await?;

            let models = response
                .get("models")
                .and_then(|m| m.as_array())
                .map(|arr| {
                    arr.iter()
                        .filter_map(|v| v.as_object())
                        .collect::<Vec<_>>()
                })
                .unwrap_or_default();

            let formatter = OutputFormatter::new(format.unwrap_or(output_format));
            formatter.print_model_list(&models)?;
            Ok(())
        }

        ModelCommands::Pull { model, force, quantization } => {
            info!("Pulling model: {} via remote kernel", model);

            let request_content = json!({
                "command": "pull",
                "model": model,
                "force": force,
                "quantization": quantization,
            });

            let response = handle.send_model_request(request_content).await?;

            if let Some(error) = response.get("error") {
                return Err(anyhow!("Model pull error: {}", error));
            }

            if let Some(status) = response.get("status").and_then(|s| s.as_str()) {
                println!("✅ {}", status);
            }

            Ok(())
        }

        ModelCommands::Status => {
            info!("Checking local LLM status via remote kernel");

            let request_content = json!({
                "command": "status",
            });

            let response = handle.send_model_request(request_content).await?;

            let formatter = OutputFormatter::new(output_format);
            formatter.print_json(&response)?;
            Ok(())
        }

        _ => {
            warn!("Command not yet implemented in connected mode");
            Err(anyhow!("Command not yet implemented"))
        }
    }
}
```

### 4.3 Kernel Message Protocol Extension

**NEW**: Add `model_request` message type to kernel protocol (similar to `tool_request`)

```rust
// llmspell-kernel/src/protocol.rs (EXTEND existing message types)

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "msg_type")]
pub enum KernelMessage {
    // ... existing message types (execute_request, tool_request, etc.)

    /// Model management request (NEW)
    #[serde(rename = "model_request")]
    ModelRequest {
        content: serde_json::Value,
    },

    /// Model management reply (NEW)
    #[serde(rename = "model_reply")]
    ModelReply {
        content: serde_json::Value,
        status: ExecutionStatus,
    },
}
```

**Handler in Kernel** (llmspell-kernel/src/handlers/mod.rs):

```rust
// NEW: Model request handler
async fn handle_model_request(
    content: serde_json::Value,
    component_registry: &ComponentRegistry,
) -> Result<serde_json::Value> {
    let command = content.get("command")
        .and_then(|c| c.as_str())
        .ok_or_else(|| anyhow!("Missing command field"))?;

    let provider_manager = component_registry.get_provider_manager();

    match command {
        "list" => {
            let backend = content.get("backend")
                .and_then(|b| b.as_str())
                .unwrap_or("all");

            let mut models = Vec::new();

            // Query Ollama
            if backend == "all" || backend == "ollama" {
                if let Ok(Some(ollama)) = provider_manager.get_local_provider("ollama").await {
                    if let Ok(ollama_models) = ollama.list_local_models().await {
                        models.extend(ollama_models);
                    }
                }
            }

            // Query Candle
            if backend == "all" || backend == "candle" {
                if let Ok(Some(candle)) = provider_manager.get_local_provider("candle").await {
                    if let Ok(candle_models) = candle.list_local_models().await {
                        models.extend(candle_models);
                    }
                }
            }

            Ok(json!({
                "models": models,
                "count": models.len()
            }))
        }

        "pull" => {
            let model_spec = content.get("model")
                .and_then(|m| m.as_str())
                .ok_or_else(|| anyhow!("Missing model field"))?;

            let spec = ModelSpec::parse(model_spec)?;
            let backend = spec.backend.as_deref().unwrap_or("ollama");

            let provider = provider_manager.get_local_provider(backend).await?
                .ok_or_else(|| anyhow!("Provider {} not available", backend))?;

            let progress = provider.pull_model(&spec).await?;

            Ok(json!({
                "status": format!("Model {} downloaded successfully", progress.model_id),
                "model_id": progress.model_id,
                "percent_complete": progress.percent_complete
            }))
        }

        "status" => {
            let mut status = json!({});

            // Check Ollama
            if let Ok(Some(ollama)) = provider_manager.get_local_provider("ollama").await {
                if let Ok(health) = ollama.health_check().await {
                    status["ollama"] = json!(health);
                }
            }

            // Check Candle
            if let Ok(Some(candle)) = provider_manager.get_local_provider("candle").await {
                if let Ok(health) = candle.health_check().await {
                    status["candle"] = json!(health);
                }
            }

            Ok(status)
        }

        _ => Err(anyhow!("Unknown model command: {}", command))
    }
}
```

### 4.4 Integration Summary

**Changes Required**:

1. **llmspell-cli/src/cli.rs**: Add `Model(ModelCommands)` variant to Commands enum
2. **llmspell-cli/src/commands/mod.rs**: Add `Commands::Model` case in execute_command
3. **llmspell-cli/src/commands/model.rs**: NEW FILE - implement dual-mode handlers as shown above
4. **llmspell-kernel/src/protocol.rs**: Add ModelRequest/ModelReply message types
5. **llmspell-kernel/src/handlers/mod.rs**: Add handle_model_request function
6. **llmspell-kernel/src/api.rs**: Add send_model_request() to KernelHandle and ClientHandle

**Architecture Decision**: Use kernel message protocol (like tool.rs) rather than direct ProviderManager access to support both embedded and connected kernel modes consistently.

---

## 5. Bridge Layer Integration

### 5.1 Lua Global for Local LLMs

```lua
-- Lua API for local LLMs (same interface regardless of backend)

-- Check local LLM status
local status = LocalLLM.status()
print("Ollama running: " .. tostring(status.ollama.running))
print("Candle ready: " .. tostring(status.candle.ready))

-- List local models
local models = LocalLLM.list()
for _, model in ipairs(models) do
    print(model.name .. " (" .. model.backend .. ")")
end

-- Create agent with local model (backend auto-detected)
local agent = Agent.create({
    model = "local/llama3.1:8b",  -- Prefers Ollama if available
    temperature = 0.7
})

-- Force specific backend
local ollama_agent = Agent.create({
    model = "local/phi3:3.8b@ollama"
})

local candle_agent = Agent.create({
    model = "local/mistral:7b@candle"
})

-- Download model from script
LocalLLM.pull("ollama/llama3.1:8b")  -- Via Ollama
LocalLLM.pull("candle/mistral:7b", {quantization = "Q4_K_M"})  -- Via Candle
```

### 5.2 Bridge Implementation

```rust
// llmspell-bridge/src/lua/globals/local_llm.rs

pub fn inject_local_llm_global(
    lua: &Lua,
    context: &GlobalContext,
    provider_manager: Arc<ProviderManager>,
) -> mlua::Result<()> {
    let local_llm = lua.create_table()?;

    // LocalLLM.status()
    let pm_clone = provider_manager.clone();
    let status_fn = lua.create_async_function(move |lua, ()| {
        let pm = pm_clone.clone();
        async move {
            let ollama_status = pm.get_local_provider("ollama").await
                .ok()
                .and_then(|p| p)
                .map(|p| async_rt::block_on(p.health_check()).ok())
                .flatten();

            let candle_status = pm.get_local_provider("candle").await
                .ok()
                .and_then(|p| p)
                .map(|p| async_rt::block_on(p.health_check()).ok())
                .flatten();

            let status_table = lua.create_table()?;

            // Ollama status
            let ollama_table = lua.create_table()?;
            ollama_table.set("running", ollama_status.is_some())?;
            if let Some(HealthStatus::Healthy { available_models, .. }) = ollama_status {
                ollama_table.set("models", available_models)?;
            }
            status_table.set("ollama", ollama_table)?;

            // Candle status
            let candle_table = lua.create_table()?;
            candle_table.set("ready", candle_status.is_some())?;
            if let Some(HealthStatus::Healthy { available_models, .. }) = candle_status {
                candle_table.set("models", available_models)?;
            }
            status_table.set("candle", candle_table)?;

            Ok(status_table)
        }
    })?;
    local_llm.set("status", status_fn)?;

    // LocalLLM.list()
    let pm_clone = provider_manager.clone();
    let list_fn = lua.create_async_function(move |lua, ()| {
        let pm = pm_clone.clone();
        async move {
            let mut all_models = Vec::new();

            // Get Ollama models
            if let Ok(Some(ollama)) = pm.get_local_provider("ollama").await {
                if let Ok(models) = ollama.list_local_models().await {
                    all_models.extend(models);
                }
            }

            // Get Candle models
            if let Ok(Some(candle)) = pm.get_local_provider("candle").await {
                if let Ok(models) = candle.list_local_models().await {
                    all_models.extend(models);
                }
            }

            // Convert to Lua table
            let models_table = lua.create_table()?;
            for (i, model) in all_models.iter().enumerate() {
                let model_table = lua.create_table()?;
                model_table.set("name", model.id.clone())?;
                model_table.set("backend", format!("{:?}", model.backend))?;
                model_table.set("size_gb", model.size_bytes as f64 / 1_000_000_000.0)?;
                models_table.set(i + 1, model_table)?;
            }

            Ok(models_table)
        }
    })?;
    local_llm.set("list", list_fn)?;

    // LocalLLM.pull(model_spec, options)
    let pm_clone = provider_manager.clone();
    let pull_fn = lua.create_async_function(move |lua, (model_spec, options): (String, Option<Table>)| {
        let pm = pm_clone.clone();
        async move {
            let spec = ModelSpec::parse(&model_spec)
                .map_err(|e| mlua::Error::RuntimeError(format!("Invalid model spec: {}", e)))?;

            let backend = spec.backend.as_ref()
                .ok_or_else(|| mlua::Error::RuntimeError("Backend not specified".to_string()))?;

            let provider = pm.get_local_provider(backend).await
                .map_err(|e| mlua::Error::RuntimeError(format!("Failed to get provider: {}", e)))?
                .ok_or_else(|| mlua::Error::RuntimeError(format!("Provider {} not available", backend)))?;

            let progress = provider.pull_model(&spec).await
                .map_err(|e| mlua::Error::RuntimeError(format!("Pull failed: {}", e)))?;

            let result_table = lua.create_table()?;
            result_table.set("model_id", progress.model_id)?;
            result_table.set("status", format!("{:?}", progress.status))?;
            result_table.set("percent_complete", progress.percent_complete)?;

            Ok(result_table)
        }
    })?;
    local_llm.set("pull", pull_fn)?;

    // Register global
    lua.globals().set("LocalLLM", local_llm)?;

    Ok(())
}
```

---

## 6. Example Applications

### 6.1 Simple Local Chat Example

```lua
-- examples/script-users/applications/local-chat/local_chat.lua
-- Simple chat application using local LLM
-- Works with both Ollama and Candle backends

-- Check what's available
local status = LocalLLM.status()

if not (status.ollama.running or status.candle.ready) then
    print("❌ No local LLM available")
    print("Install Ollama: llmspell model install-ollama")
    return
end

-- List available models
print("📦 Available models:")
local models = LocalLLM.list()
for _, model in ipairs(models) do
    print("  • " .. model.name .. " (" .. model.backend .. ")")
end

-- Use first available model
if #models == 0 then
    print("No models found. Download one:")
    print("  llmspell model pull ollama/phi3:3.8b")
    return
end

local model_name = models[1].name
print("\n💬 Using model: " .. model_name)

-- Create chat agent with local model
local agent = Agent.create({
    name = "local_assistant",
    model = "local/" .. model_name,
    system_prompt = [[You are a helpful AI assistant running locally.
Be concise and friendly.]],
    temperature = 0.7
})

-- Chat loop
print("\n🤖 Local Chat (type 'exit' to quit)")
print("─────────────────────────────────────")

while true do
    io.write("\nYou: ")
    local input = io.read()

    if input == "exit" then
        break
    end

    io.write("Assistant: ")

    -- Stream response
    local stream = agent:execute_streaming({text = input})
    for chunk in stream do
        if chunk.content.type == "text" then
            io.write(chunk.content.text)
            io.flush()
        end
    end

    io.write("\n")
end

print("\n👋 Goodbye!")
```

### 6.2 Ollama-Specific Example

```lua
-- examples/script-users/applications/local-chat/ollama_chat.lua
-- Chat example explicitly using Ollama backend

local agent = Agent.create({
    name = "ollama_assistant",
    model = "local/phi3:3.8b@ollama",  -- Force Ollama
    temperature = 0.7
})

local result = agent:execute({
    text = "Explain what Ollama is in one sentence."
})

print(result.text)
print("\nModel: " .. result.metadata.model)
print("Backend: " .. result.metadata.provider)
print("Tokens: " .. (result.metadata.tokens.total or 0))
print("Latency: " .. (result.metadata.latency_ms or 0) .. "ms")
```

### 6.3 Candle-Specific Example

```lua
-- examples/script-users/applications/local-chat/candle_inference.lua
-- Direct Candle inference example

-- Ensure Candle model is downloaded
if not candle_model_exists("mistral:7b") then
    print("Downloading Mistral 7B GGUF...")
    LocalLLM.pull("candle/mistral:7b", {
        quantization = "Q4_K_M"
    })
end

-- Create agent with Candle backend
local agent = Agent.create({
    name = "candle_assistant",
    model = "local/mistral:7b@candle",  -- Force Candle
    temperature = 0.8,
    max_tokens = 512
})

-- Benchmark inference
local start = os.clock()
local result = agent:execute({
    text = "Write a haiku about Rust programming."
})
local elapsed = os.clock() - start

print(result.text)
print("\n⚡ Performance:")
print("  Time: " .. string.format("%.2f", elapsed) .. "s")
print("  Tokens: " .. (result.metadata.tokens.completion or 0))
print("  Speed: " .. string.format("%.1f",
    (result.metadata.tokens.completion or 0) / elapsed) .. " tokens/sec")
```

### 6.4 Comparison Example

```lua
-- examples/script-users/applications/local-chat/backend_comparison.lua
-- Compare Ollama vs Candle performance

local prompt = "Explain quantum computing in simple terms."

print("🔬 Backend Comparison\n")
print("Prompt: " .. prompt)
print("─────────────────────────────────────")

-- Test Ollama
print("\n🦙 Ollama (llama3.1:8b):")
local ollama_agent = Agent.create({
    model = "local/llama3.1:8b@ollama",
    temperature = 0.7
})

local start = os.clock()
local ollama_result = ollama_agent:execute({text = prompt})
local ollama_time = os.clock() - start

print(ollama_result.text:sub(1, 200) .. "...")
print("\nMetrics:")
print("  Time: " .. string.format("%.2f", ollama_time) .. "s")
print("  Tokens: " .. (ollama_result.metadata.tokens.total or 0))
print("  Speed: " .. string.format("%.1f",
    (ollama_result.metadata.tokens.completion or 0) / ollama_time) .. " tok/s")

-- Test Candle
print("\n🕯️  Candle (mistral:7b):")
local candle_agent = Agent.create({
    model = "local/mistral:7b@candle",
    temperature = 0.7
})

start = os.clock()
local candle_result = candle_agent:execute({text = prompt})
local candle_time = os.clock() - start

print(candle_result.text:sub(1, 200) .. "...")
print("\nMetrics:")
print("  Time: " .. string.format("%.2f", candle_time) .. "s")
print("  Tokens: " .. (candle_result.metadata.tokens.total or 0))
print("  Speed: " .. string.format("%.1f",
    (candle_result.metadata.tokens.completion or 0) / candle_time) .. " tok/s")

-- Summary
print("\n📊 Summary:")
print("  Faster: " .. (ollama_time < candle_time and "Ollama" or "Candle"))
print("  Difference: " .. string.format("%.1f",
    math.abs(ollama_time - candle_time)) .. "s")
```

### 6.5 Configuration Files

```toml
# examples/script-users/applications/local-chat/llmspell.toml
# Local chat application configuration

[providers.local]
enabled = true
default_backend = "ollama"
auto_pull_missing = true
fallback_to_cloud = false

[providers.local.ollama]
url = "http://localhost:11434"
timeout_seconds = 120
auto_start = true

[providers.local.candle]
model_directory = "${HOME}/.llmspell/models/candle"
device = "auto"
default_quantization = "Q4_K_M"
cpu_threads = 0

[providers.local.candle.performance]
context_size = 4096
batch_size = 512
use_flash_attention = true
```

---

## 7. Testing Strategy

### 7.1 Unit Tests

```rust
// llmspell-providers/src/local/tests/ollama_tests.rs

#[cfg(test)]
mod ollama_tests {
    use super::*;

    #[tokio::test]
    async fn test_ollama_provider_creation() {
        let config = OllamaConfig::default();
        let provider = OllamaProvider::new(config);
        assert!(provider.is_ok());
    }

    #[tokio::test]
    #[ignore] // Requires Ollama running
    async fn test_ollama_health_check() {
        let config = OllamaConfig::default();
        let provider = OllamaProvider::new(config).unwrap();

        let health = provider.health_check().await;
        assert!(health.is_ok());
    }

    #[tokio::test]
    #[ignore] // Requires Ollama with models
    async fn test_ollama_list_models() {
        let config = OllamaConfig::default();
        let provider = OllamaProvider::new(config).unwrap();

        let models = provider.list_local_models().await;
        assert!(models.is_ok());
    }

    #[tokio::test]
    #[ignore] // Requires Ollama with llama3.1
    async fn test_ollama_completion() {
        let config = OllamaConfig::default();
        let provider = OllamaProvider::new(config).unwrap();

        let input = AgentInput {
            text: "Say hello in one word".to_string(),
            media: vec![],
            context: None,
            parameters: json!({
                "model": "llama3.1:8b",
                "temperature": 0.7
            }).as_object().unwrap().clone(),
            output_modalities: vec![],
        };

        let output = provider.complete(input).await;
        assert!(output.is_ok());

        let output = output.unwrap();
        assert!(!output.text.is_empty());
        assert_eq!(output.metadata.provider, Some("ollama".to_string()));
    }
}
```

```rust
// llmspell-providers/src/local/tests/candle_tests.rs

#[cfg(test)]
mod candle_tests {
    use super::*;

    #[tokio::test]
    async fn test_candle_provider_creation() {
        let config = CandleConfig::default();
        let provider = CandleProvider::new(config);
        assert!(provider.is_ok());
    }

    #[tokio::test]
    async fn test_candle_health_check() {
        let config = CandleConfig::default();
        let provider = CandleProvider::new(config).unwrap();

        let health = provider.health_check().await;
        assert!(health.is_ok());
    }

    #[tokio::test]
    #[ignore] // Requires downloaded models
    async fn test_candle_list_models() {
        let config = CandleConfig::default();
        let provider = CandleProvider::new(config).unwrap();

        let models = provider.list_local_models().await;
        assert!(models.is_ok());
    }

    #[tokio::test]
    #[ignore] // Requires mistral:7b model
    async fn test_candle_load_model() {
        let config = CandleConfig::default();
        let provider = CandleProvider::new(config).unwrap();

        let result = provider.load_model("mistral:7b").await;
        assert!(result.is_ok());
    }
}
```

### 7.2 Integration Tests

```rust
// tests/integration/local_llm_test.rs

#[tokio::test]
#[ignore] // Expensive test
async fn test_local_llm_full_workflow() {
    // This test validates the full workflow:
    // 1. Check status
    // 2. List models
    // 3. Pull model if needed
    // 4. Execute completion

    let config = RuntimeConfig::load().await.unwrap();
    let provider_manager = ProviderManager::new(config.providers).await.unwrap();

    // Get Ollama provider
    let ollama = provider_manager.get_local_provider("ollama").await
        .unwrap()
        .expect("Ollama provider not configured");

    // Check health
    let health = ollama.health_check().await.unwrap();
    assert!(matches!(health, HealthStatus::Healthy { .. }));

    // List models
    let models = ollama.list_local_models().await.unwrap();

    // Pull phi3 if not available
    if !models.iter().any(|m| m.id.contains("phi3")) {
        let spec = ModelSpec {
            category: "local".to_string(),
            model: "phi3".to_string(),
            variant: Some("3.8b".to_string()),
            backend: Some("ollama".to_string()),
        };

        let _progress = ollama.pull_model(&spec).await.unwrap();
    }

    // Execute completion
    let input = AgentInput {
        text: "What is 2+2?".to_string(),
        parameters: json!({"model": "phi3:3.8b"}).as_object().unwrap().clone(),
        ..Default::default()
    };

    let output = ollama.complete(input).await.unwrap();
    assert!(!output.text.is_empty());
    assert!(output.text.contains("4"));
}

#[tokio::test]
#[ignore] // Expensive test
async fn test_backend_switching() {
    // Test that agents can switch between Ollama and Candle

    let config = RuntimeConfig::load().await.unwrap();
    let runtime = ScriptRuntime::new(config).await.unwrap();

    let script = r#"
        -- Test Ollama
        local ollama_agent = Agent.create({
            model = "local/phi3:3.8b@ollama"
        })

        local result1 = ollama_agent:execute({
            text = "Say 'ollama' in one word"
        })

        assert(result1.metadata.provider == "ollama")

        -- Test Candle
        local candle_agent = Agent.create({
            model = "local/mistral:7b@candle"
        })

        local result2 = candle_agent:execute({
            text = "Say 'candle' in one word"
        })

        assert(result2.metadata.provider == "candle")

        return "success"
    "#;

    let output = runtime.execute_script(script).await.unwrap();
    assert_eq!(output.as_string().unwrap(), "success");
}
```

### 7.3 Performance Benchmarks

```rust
// benches/local_llm_benchmarks.rs

use criterion::{criterion_group, criterion_main, Criterion, BenchmarkId};

fn benchmark_ollama_completion(c: &mut Criterion) {
    let mut group = c.benchmark_group("ollama_completion");

    let runtime = tokio::runtime::Runtime::new().unwrap();
    let config = runtime.block_on(async {
        OllamaConfig::default()
    });

    let provider = OllamaProvider::new(config).unwrap();

    let inputs = vec![
        "What is 2+2?",
        "Explain Rust in one sentence.",
        "Write a short poem about coding.",
    ];

    for input in inputs.iter() {
        group.bench_with_input(
            BenchmarkId::new("ollama", input),
            input,
            |b, prompt| {
                b.to_async(&runtime).iter(|| async {
                    let input = AgentInput {
                        text: prompt.to_string(),
                        parameters: json!({"model": "phi3:3.8b"})
                            .as_object().unwrap().clone(),
                        ..Default::default()
                    };

                    provider.complete(input).await.unwrap()
                });
            },
        );
    }

    group.finish();
}

fn benchmark_candle_completion(c: &mut Criterion) {
    let mut group = c.benchmark_group("candle_completion");

    // Similar structure for Candle benchmarks

    group.finish();
}

criterion_group!(
    benches,
    benchmark_ollama_completion,
    benchmark_candle_completion
);
criterion_main!(benches);
```

---

## 8. Documentation Requirements

### 8.1 User Documentation

Create comprehensive user guide at `docs/user-guide/local-llm/README.md`:

**Topics to cover:**
1. Introduction to Local LLMs
2. Installing Ollama (macOS, Linux, Windows)
3. Installing and configuring Candle
4. Model management via CLI
5. Configuration options
6. Performance tuning
7. Troubleshooting common issues
8. Model recommendations
9. Cost-free operation benefits
10. Offline usage scenarios

### 8.2 API Documentation

Document all new APIs in rustdoc:
- `OllamaProvider`
- `CandleProvider`
- `LocalProviderInstance` trait
- `ModelSpec` parsing
- CLI commands

### 8.3 Example Documentation

Document all examples in `examples/script-users/applications/local-chat/README.md`:
- Setup instructions
- Usage examples
- Expected output
- Performance expectations

---

## 9. Success Validation ✅ COMPLETE

### 9.1 Acceptance Tests - ACTUAL RESULTS

Phase 11.1 (Ollama) - ✅ COMPLETE:
- ✅ Ollama provider functional via rig integration
- ✅ `llmspell model list --backend ollama` shows Ollama models
- ✅ `llmspell model pull ollama/phi3:3.8b` downloads model
- ✅ LocalLLM.list() shows Ollama models from Lua scripts (alternative API)
- ✅ LocalLLM.pull("ollama/phi3:3.8b") downloads model from scripts
- ✅ Scripts use Ollama models successfully (examples demonstrate)
- ✅ Streaming works with Ollama (via rig)
- ✅ 5 Ollama integration tests passing
- ✅ CLI commands fully implemented (468 lines, dual-mode)

Phase 11.2 (Candle) - ✅ COMPLETE:
- ✅ Candle provider functional with GGUF loading
- ✅ `llmspell model pull candle/tinyllama:Q4_K_M` downloads from HF
- ✅ GGUF models load correctly with HuggingFace downloads
- ✅ Tokenizer fallback mechanism working
- ✅ Chat template formatting for TinyLlama-Chat models
- ✅ LocalLLM.pull("candle/tinyllama:Q4_K_M") downloads from HF (script API)
- ✅ Scripts use Candle models successfully (examples demonstrate)
- ✅ 5 Candle integration tests passing with proper model directory handling
- ✅ Memory-efficient Q4_K_M quantization support
- ✅ CLI commands fully implemented (same 468-line implementation handles both backends)

### 9.2 Performance Validation - ACTUAL IMPLEMENTATION

Validation Status:
- ✅ Ollama: Functional via rig REST API (performance depends on local Ollama installation)
- ✅ Candle: GGUF loading with proper tensor handling
- ✅ Memory: Q4_K_M quantization support implemented
- ℹ️ Performance benchmarks not run in Phase 11 (functional validation prioritized)

Target metrics for 7B models (from design, not validated in Phase 11):
- **First token latency**: <100ms (Ollama), <200ms (Candle)
- **Throughput**: >20 tokens/second
- **Memory**: <5GB RAM (Q4_K_M quantization)
- **Model load time**: <5 seconds
- **Cold start**: <10 seconds (including model load)

**Note**: Performance validation can be added in future phase if needed. Phase 11 focused on functional correctness and integration.

---

## 10. Phase Deliverables ✅ ACTUAL IMPLEMENTATION

### 10.1 Code Deliverables - DELIVERED

**Phase 11.1 (Ollama) - ✅ COMPLETE:**
1. ✅ OllamaProvider implementation via rig (llmspell-providers/src/local/ollama/)
2. ✅ LocalProviderInstance trait with list, pull, info, status methods
3. ✅ CLI commands fully implemented (llmspell-cli/src/commands/model.rs - 468 lines)
4. ✅ Bridge integration for Lua (LocalLLM global)
5. ✅ Example applications using Ollama (2 of 4 examples)
6. ✅ 5 Ollama integration tests with OLLAMA_AVAILABLE guard

**Phase 11.2 (Candle) - ✅ COMPLETE:**
1. ✅ CandleProvider implementation (llmspell-providers/src/local/candle/)
2. ✅ GGUF loading with HuggingFace tokenizer fallback
3. ✅ Chat template formatting for TinyLlama-Chat models
4. ✅ CLI commands fully implemented (same dual-mode handler for both backends)
5. ✅ GGUF loading and inference pipeline with proper tensor handling
6. ✅ Example applications using Candle (2 of 4 examples)
7. ✅ 5 Candle integration tests with RUN_EXPENSIVE_TESTS guard
8. [-] Performance benchmarks deferred (functional validation prioritized)

**Shared - ✅ COMPLETE:**
1. ✅ Unified model management CLI (7 subcommands: list, pull, remove, info, available, status, install-ollama)
2. ✅ Kernel message protocol extension (model_request/model_reply in llmspell-kernel/src/execution/integrated.rs:2502)
3. ✅ Configuration schema extensions (flat structure using HashMap)
4. ✅ LocalProviderInstance trait
5. ✅ ModelSpecifier parser with @backend syntax
6. ✅ Bridge layer integration (LocalLLM Lua global)
7. ✅ Documentation suite (320-line user guide + 4 examples)

### 10.2 Documentation Deliverables - DELIVERED

1. ✅ API documentation (rustdoc) for all new types - zero warnings
2. ✅ User guide for local LLM setup and usage (docs/user-guide/local-llm.md - 320 lines)
3. ✅ Example documentation (4 Lua examples: status, chat, comparison, model info - 260 lines)
4. [-] Migration guide from cloud to local (covered in user guide)
5. [-] Performance tuning guide (deferred, basic guidance in user guide)
6. ✅ Troubleshooting guide (6 common scenarios in user guide)
7. ✅ Model recommendation guide (supported models section in user guide)

### 10.3 Quality Metrics - ACTUAL RESULTS

- ✅ **Zero compiler warnings** - verified
- ✅ **Zero clippy warnings** in Phase 11 code (llmspell-providers Phase 11 packages clean)
- ✅ **All integration tests passing** - 10/10 tests (5 Ollama + 5 Candle)
- [-] **Performance benchmarks** - deferred, functional validation prioritized
- ✅ **Documentation builds without warnings** - cargo doc clean
- ✅ **Examples documented** - 4 production-ready Lua examples with usage instructions

---

## 11. Risk Mitigation

### 11.1 Technical Risks

**Risk**: Ollama not installed/running on user system
- **Mitigation**: Auto-detection, helpful error messages, `install-ollama` command
- **Fallback**: Instructions for manual installation

**Risk**: Candle GGUF loading complexity
- **Mitigation**: Use well-tested `candle-transformers` crate
- **Fallback**: Comprehensive error handling, model validation

**Risk**: Performance doesn't meet targets
- **Mitigation**: Early benchmarking, GPU acceleration, model quantization
- **Fallback**: Document minimum requirements, provide tuning guide

**Risk**: Model compatibility issues
- **Mitigation**: Test with recommended models (Phi-3, LLaMA 3.1, Mistral, Gemma 2)
- **Fallback**: Clear model support matrix, version tracking

### 11.2 User Experience Risks

**Risk**: Confusing model specification syntax
- **Mitigation**: Clear documentation, helpful error messages, auto-detection
- **Fallback**: CLI validation with suggestions

**Risk**: Large model downloads
- **Mitigation**: Progress indicators, size warnings, resume support
- **Fallback**: Recommend starting with smaller models (Phi-3 3.8B)

**Risk**: Insufficient disk space
- **Mitigation**: Pre-flight checks, clear space requirements
- **Fallback**: Helpful error messages with cleanup commands

---

## Appendix A: Recommended Models

### Small Models (<3GB)

**Phi-3 Mini (3.8B)** - Recommended for beginners
- Size: ~2.4GB (Q4_K_M)
- Speed: Fast on CPU
- Quality: GPT-3.5 level
- Backend: Ollama (primary), Candle

### Medium Models (3-5GB)

**Mistral 7B v0.3** - Best overall 7B model
- Size: ~4.1GB (Q4_K_M)
- Speed: Good on CPU, excellent on GPU
- Quality: Excellent instruction following
- Backend: Both (recommended for Candle testing)

**LLaMA 3.1 8B** - Most capable open model
- Size: ~4.7GB (Q4_K_M)
- Speed: Moderate on CPU, fast on GPU
- Quality: Best reasoning and instruction following
- Backend: Both (recommended for production)

### Large Models (5-7GB)

**Gemma 2 9B** - Google's efficient model
- Size: ~5.4GB (Q4_K_M)
- Speed: Good with optimizations
- Quality: Strong performance on benchmarks
- Backend: Ollama (primary)

---

## Appendix B: Performance Baselines

Based on research and expected performance:

### Ollama Performance (M1 Mac, 16GB RAM)

**Phi-3 3.8B:**
- First token: 50-80ms
- Throughput: 30-40 tokens/sec
- Memory: 3GB

**LLaMA 3.1 8B:**
- First token: 80-120ms
- Throughput: 20-30 tokens/sec
- Memory: 6GB

### Candle Performance (Same hardware)

**Mistral 7B (Q4_K_M):**
- Load time: 3-5 seconds
- First token: 150-250ms
- Throughput: 15-25 tokens/sec
- Memory: 5GB

**CPU vs GPU:**
- GPU (Metal): 3-5x faster
- GPU (CUDA): 4-6x faster
- CPU: Baseline (usable for smaller models)

---

## 12. Phase 11 Completion Summary

**Status**: ✅ PHASE 11 COMPLETE (v0.11.0)

**Delivered:**
- ✅ Dual-backend local LLM support (Ollama via rig + Candle embedded GGUF)
- ✅ CLI model commands (7 subcommands: list, pull, remove, info, available, status, install-ollama)
- ✅ Kernel message protocol extension (model_request/model_reply)
- ✅ LocalLLM Lua global with list, pull, info, status methods
- ✅ ModelSpecifier with @ollama/@candle backend syntax
- ✅ 10 integration tests passing (5 Ollama + 5 Candle)
- ✅ Comprehensive documentation (320-line user guide + 4 examples)
- ✅ Zero compiler/clippy warnings
- ✅ Chat template formatting for TinyLlama-Chat models
- ✅ HuggingFace downloads with tokenizer fallback

**Deferred (Non-Blocking):**
- [-] Performance benchmarks (functional validation prioritized, can be added later)

**Key Achievements:**
1. **Dual Interface**: Both CLI commands AND Lua API for maximum user flexibility
2. **Validated Architecture**: Flat config structure, rig integration, factory pattern, kernel protocol all work perfectly
3. **Test Coverage**: 10 comprehensive integration tests covering full stack
4. **Documentation**: Complete user guide with troubleshooting + 4 production examples
5. **Zero Technical Debt**: All warnings resolved, clean codebase
6. **Privacy-First**: Offline-capable local model inference

**Timeline:**
- **Planned**: 20 working days
- **Actual**: Completed within planned timeline

**Impact:**
- Users can now run LLMs locally without API costs
- Privacy-first workflows with offline capability
- Support for both Ollama (ease of use) and Candle (embedded)
- Dual interface (CLI + Lua API) for shell scripts and interactive workflows
- Full kernel protocol integration for remote kernel model management

---

**End of Phase 11 Design Document**

**Total Time**: 20 working days (as planned)
- Phase 11.1 (Ollama): ~10 days
- Phase 11.2 (Candle): ~10 days
- Phase 11.8 (Testing): comprehensive integration tests
- Phase 11.9 (Documentation): user guide + examples

**Next Phase**: Phase 12 - Adaptive Memory System with local embeddings
