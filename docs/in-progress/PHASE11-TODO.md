# Phase 11: Local LLM Integration - Implementation Tasks

**Version**: 2.0 (Updated based on comprehensive design doc analysis)
**Date**: October 2025
**Status**: Implementation Ready
**Phase**: 11 (Local LLM Integration via Ollama + Candle)
**Timeline**: 20 working days
**Priority**: CRITICAL
**Dependencies**: Phase 10 ✅
**Arch-Document**: docs/technical/master-architecture-vision.md
**All-Phases-Document**: docs/in-progress/implementation-phases.md
**Current-Architecture-Document**: docs/technical/current-architecture.md
**Design-Document**: docs/in-progress/phase-11-design-doc.md
**Gap-Analysis**: docs/archives/LOCAL-LLM-ANALYSIS-V2.md
**Old-TODO**: docs/archives/PHASE11-TODO.invalid.md
**This-document**: working copy /TODO.md (pristine/immutable copy in docs/in-progress/PHASE11-DONE.md)

> **🎯 Implementation Focus**: This TODO comprehensively covers ALL integration points identified in the design doc, including: rig-based Ollama integration, kernel message protocol extensions, dual-mode CLI handlers, flat config structure, ModelSpecifier extensions, bridge layer integration, and Candle implementation.

---

## Success Criteria Summary

**Provider Layer:**
- [ ] llmspell-providers compiles with Ollama (rig-based) and Candle providers
- [ ] LocalProviderInstance trait extends ProviderInstance with model management
- [ ] ModelSpecifier parses backend syntax (`local/model:variant@backend`)
- [ ] Provider routing logic uses existing factory pattern

**Kernel Layer:**
- [ ] ModelRequest/ModelReply message types added to protocol
- [ ] handle_model_request() implements all model commands
- [ ] KernelHandle.send_model_request() and ClientHandle.send_model_request() added

**CLI Layer:**
- [ ] ModelCommands enum with all subcommands (list, pull, status, etc.)
- [ ] Dual-mode handlers (handle_model_embedded, handle_model_remote)
- [ ] Commands follow tool.rs pattern with ExecutionContext

**Config Layer:**
- [ ] Flat structure: `[providers.ollama]` and `[providers.candle]`
- [ ] Backend-specific options in `[providers.<name>.options]`
- [ ] No changes to existing ProviderConfig struct

**Bridge Layer:**
- [ ] LocalLLM global injected with status(), list(), pull() methods
- [ ] Agent.create() supports `model = "local/llama3.1:8b"` syntax
- [ ] Backend auto-detection works (prefers Ollama, falls back to Candle)

**Performance & Quality:**
- [ ] Ollama: <100ms first token latency
- [ ] Candle: <200ms first token latency
- [ ] Both: >20 tokens/sec for 7B models
- [ ] Memory <5GB for Q4_K_M models
- [ ] >90% test coverage for new code
- [ ] Zero clippy warnings

---

## SECTION 1: Provider Architecture Foundation

### Task 11.1.1: Extend ModelSpecifier with Backend Field

**File**: `llmspell-providers/src/model_specifier.rs`
**Priority**: CRITICAL
**Estimated**: 3 hours
**Dependencies**: None

**Context**: Current ModelSpecifier only has provider and model fields. Need to add backend field to parse `@ollama` or `@candle` suffix.

**Acceptance Criteria:**
- [ ] `backend: Option<String>` field added to ModelSpecifier struct
- [ ] Parse logic handles `local/model:variant@backend` syntax
- [ ] Parse logic handles `model:variant@backend` syntax (no provider prefix)
- [ ] Backend defaults to None for backward compatibility
- [ ] All existing tests still pass

**Implementation Steps:**
1. Read existing ModelSpecifier struct (llmspell-providers/src/model_specifier.rs)
2. Add `backend: Option<String>` field:
   ```rust
   pub struct ModelSpecifier {
       pub provider: Option<String>,
       pub model: String,
       pub backend: Option<String>,  // NEW
       pub base_url: Option<String>,
   }
   ```
3. Update `parse()` method to split on `@` first:
   ```rust
   pub fn parse(spec: &str) -> Result<Self> {
       // Split on @ to extract backend
       let (model_part, backend) = if let Some(idx) = spec.rfind('@') {
           (&spec[..idx], Some(spec[idx + 1..].to_string()))
       } else {
           (spec, None)
       };

       // Continue with existing provider/model parsing on model_part
       // ...
   }
   ```
4. Update all existing parse tests to verify backward compatibility
5. Add new tests for backend syntax:
   - `"local/llama3.1:8b"` → backend=None
   - `"local/llama3.1:8b@ollama"` → backend=Some("ollama")
   - `"llama3.1:8b@candle"` → backend=Some("candle"), provider=None

**Definition of Done:**
- [ ] Backend field compiles
- [ ] Parse logic handles all syntax variants
- [ ] Existing tests pass
- [ ] New tests cover backend parsing
- [ ] Zero clippy warnings
- [ ] rustdoc comments updated with examples

---

### Task 11.1.2: Update Provider Routing Logic

**File**: `llmspell-providers/src/abstraction.rs` (lines 427-431)
**Priority**: CRITICAL
**Estimated**: 2 hours
**Dependencies**: Task 11.1.1

**Context**: Current routing maps provider names to factory names (e.g., "openai" → "rig"). Need to add "local" provider with backend resolution logic.

**Acceptance Criteria:**
- [ ] "local" provider routes to backend-specific factory
- [ ] Backend resolution checks spec.backend first
- [ ] Falls back to config default_backend if spec.backend is None
- [ ] Final fallback to "ollama" if no configuration
- [ ] Existing cloud provider routing unchanged

**Implementation Steps:**
1. Read existing routing logic in abstraction.rs:427-431
2. Add "local" case to the match statement:
   ```rust
   let implementation_name = match provider_name.as_str() {
       "openai" | "anthropic" | "cohere" | "groq" | "perplexity"
       | "together" | "gemini" | "mistral" | "replicate" | "fireworks" => "rig",

       // NEW: Local provider routing with backend resolution
       "local" => {
           spec.backend.as_deref()
               .or_else(|| {
                   // Read default_backend from config.providers["ollama"].options
                   config.providers.get("ollama")
                       .and_then(|c| c.options.get("default_backend"))
                       .and_then(|v| v.as_str())
               })
               .unwrap_or("ollama")  // Final fallback
       }

       other => other,
   };
   ```
3. Add tracing:
   ```rust
   debug!("Provider routing: {} → {}", provider_name, implementation_name);
   if provider_name == "local" {
       trace!("Backend resolution: spec={:?}, config_default={:?}, final={}",
           spec.backend, /* config default */, implementation_name);
   }
   ```
4. Write unit tests for routing logic:
   - `"local"` with `spec.backend = Some("ollama")` → "ollama"
   - `"local"` with `spec.backend = None` and config default → config default
   - `"local"` with no spec or config → "ollama" fallback

**Definition of Done:**
- [ ] Routing logic compiles
- [ ] Backend resolution tested
- [ ] Tracing comprehensive
- [ ] Unit tests pass
- [ ] Zero clippy warnings

---

### Task 11.1.3: Create LocalProviderInstance Trait

**File**: `llmspell-providers/src/local/mod.rs` (NEW FILE)
**Priority**: CRITICAL
**Estimated**: 2 hours
**Dependencies**: None

**Context**: Need trait that extends ProviderInstance with model management methods (health_check, list_local_models, pull_model, etc.).

**Acceptance Criteria:**
- [ ] LocalProviderInstance trait extends ProviderInstance
- [ ] health_check() method defined
- [ ] list_local_models() method defined
- [ ] pull_model() method defined
- [ ] model_info() method defined
- [ ] unload_model() method defined
- [ ] All methods async with proper error types

**Implementation Steps:**
1. Create `llmspell-providers/src/local/mod.rs`
2. Define trait with comprehensive documentation:
   ```rust
   //! Local provider trait extensions for model management

   use async_trait::async_trait;
   use anyhow::Result;
   use crate::abstraction::ProviderInstance;

   /// Health status of a local provider backend
   #[derive(Debug, Clone)]
   pub enum HealthStatus {
       Healthy {
           available_models: usize,
           version: Option<String>,
       },
       Unhealthy {
           reason: String,
       },
       Unknown,
   }

   /// Local model metadata
   #[derive(Debug, Clone)]
   pub struct LocalModel {
       pub id: String,
       pub backend: String,
       pub size_bytes: u64,
       pub quantization: Option<String>,
       pub modified_at: Option<std::time::SystemTime>,
   }

   /// Model download progress
   #[derive(Debug, Clone)]
   pub struct PullProgress {
       pub model_id: String,
       pub status: DownloadStatus,
       pub percent_complete: f32,
       pub bytes_downloaded: u64,
       pub bytes_total: Option<u64>,
   }

   #[derive(Debug, Clone)]
   pub enum DownloadStatus {
       Starting,
       Downloading,
       Verifying,
       Complete,
       Failed(String),
   }

   /// Model specification for downloads
   #[derive(Debug, Clone)]
   pub struct ModelSpec {
       pub model: String,
       pub variant: Option<String>,
       pub backend: Option<String>,
   }

   impl ModelSpec {
       pub fn parse(spec: &str) -> Result<Self> {
           // Parse from model specifier string
           // e.g., "llama3.1:8b@ollama" → model="llama3.1", variant="8b", backend="ollama"
           todo!("Parse model specification")
       }
   }

   /// Trait for local LLM providers with model management
   #[async_trait]
   pub trait LocalProviderInstance: ProviderInstance {
       /// Check if backend is available and healthy
       async fn health_check(&self) -> Result<HealthStatus>;

       /// List locally available models
       async fn list_local_models(&self) -> Result<Vec<LocalModel>>;

       /// Pull/download a model
       async fn pull_model(&self, model_spec: &ModelSpec) -> Result<PullProgress>;

       /// Get detailed model information
       async fn model_info(&self, model_id: &str) -> Result<ModelInfo>;

       /// Unload model from memory (if applicable)
       async fn unload_model(&self, model_id: &str) -> Result<()>;
   }

   /// Detailed model information
   #[derive(Debug, Clone)]
   pub struct ModelInfo {
       pub id: String,
       pub backend: String,
       pub size_bytes: u64,
       pub parameter_count: Option<String>,
       pub quantization: Option<String>,
       pub format: String,  // "GGUF", "Safetensors", etc.
       pub loaded: bool,
   }
   ```
3. Add module to llmspell-providers/src/lib.rs:
   ```rust
   pub mod local;
   ```
4. Write rustdoc examples for each type

**Definition of Done:**
- [ ] Trait compiles
- [ ] All types well-documented
- [ ] Module exported from lib.rs
- [ ] Zero clippy warnings
- [ ] Rustdoc examples compile

---

### Task 11.1.4: Add Provider Configuration (No Struct Changes)

**Files**:
- `llmspell-config/src/providers.rs` (READ ONLY - verify no changes needed)
- Example configs for documentation

**Priority**: HIGH
**Estimated**: 1 hour
**Dependencies**: None

**Context**: Verify that existing ProviderConfig.options HashMap can handle backend-specific fields without struct changes.

**Acceptance Criteria:**
- [ ] Confirmed ProviderConfig struct needs NO changes
- [ ] Example TOML configs created for Ollama and Candle
- [ ] Backend-specific field extraction pattern documented
- [ ] Config merge logic confirmed to work with flat structure

**Implementation Steps:**
1. Read llmspell-config/src/providers.rs to confirm structure:
   ```rust
   // EXISTING CODE - verify no changes needed
   pub struct ProviderConfig {
       pub name: String,
       pub provider_type: String,
       pub enabled: bool,
       pub base_url: Option<String>,
       pub api_key: Option<String>,
       pub default_model: Option<String>,
       pub max_tokens: Option<u32>,
       pub timeout_seconds: Option<u64>,
       pub options: HashMap<String, serde_json::Value>,  // ← Used for backend-specific
   }
   ```
2. Create example config `examples/configs/local-llm-ollama.toml`:
   ```toml
   [providers.ollama]
   provider_type = "ollama"
   enabled = true
   base_url = "http://localhost:11434"
   timeout_seconds = 120

   [providers.ollama.options]
   auto_start = true
   health_check_interval_seconds = 60
   default_backend = "ollama"
   ```
3. Create example config `examples/configs/local-llm-candle.toml`:
   ```toml
   [providers.candle]
   provider_type = "candle"
   enabled = true
   timeout_seconds = 300

   [providers.candle.options]
   model_directory = "${HOME}/.llmspell/models/candle"
   device = "auto"
   max_concurrent = 1
   default_quantization = "Q4_K_M"
   cpu_threads = 0
   context_size = 4096
   batch_size = 512
   use_flash_attention = true
   ```
4. Document extraction pattern:
   ```rust
   // Example of reading backend-specific fields from options
   impl CandleProvider {
       pub fn new(config: ProviderConfig) -> Result<Self> {
           let device = config.options.get("device")
               .and_then(|v| v.as_str())
               .unwrap_or("auto");

           let model_directory = config.options.get("model_directory")
               .and_then(|v| v.as_str())
               .map(|s| PathBuf::from(s))
               .unwrap_or_else(|| {
                   dirs::home_dir().unwrap().join(".llmspell/models/candle")
               });

           // ...
       }
   }
   ```

**Definition of Done:**
- [ ] Confirmed no struct changes needed
- [ ] Example configs created
- [ ] Extraction pattern documented
- [ ] Config merge logic verified

---

## SECTION 2: Ollama Integration (via rig + ollama-rs hybrid)

### Task 11.2.1: Add Rig Ollama Variant to RigModel Enum

**File**: `llmspell-providers/src/rig.rs` (lines 17-22)
**Priority**: CRITICAL
**Estimated**: 2 hours
**Dependencies**: None

**Context**: rig-core v0.20.0 natively supports Ollama. We just need to add the Ollama variant to our RigModel enum.

**Acceptance Criteria:**
- [ ] Ollama variant added to RigModel enum
- [ ] Ollama case added to create_rig_provider match
- [ ] Base URL defaults to http://localhost:11434
- [ ] Compilation succeeds
- [ ] Zero clippy warnings

**Implementation Steps:**
1. Read existing RigModel enum (rig.rs:17-22):
   ```rust
   // EXISTING CODE
   enum RigModel {
       OpenAI(providers::openai::CompletionModel),
       Anthropic(providers::anthropic::completion::CompletionModel),
       Cohere(providers::cohere::CompletionModel),
       // Add Ollama here
   }
   ```
2. Add Ollama variant:
   ```rust
   use rig::providers;

   enum RigModel {
       OpenAI(providers::openai::CompletionModel),
       Anthropic(providers::anthropic::completion::CompletionModel),
       Cohere(providers::cohere::CompletionModel),
       Ollama(providers::ollama::CompletionModel),  // NEW
   }
   ```
3. Add Ollama case to create_rig_provider (approx line 42+):
   ```rust
   // In create_rig_provider function
   match config.provider_type.as_str() {
       "openai" => {
           // existing OpenAI code
       }
       "anthropic" => {
           // existing Anthropic code
       }
       "cohere" => {
           // existing Cohere code
       }
       "ollama" => {
           info!("Creating Ollama provider via rig");
           let base_url = config.base_url.as_deref()
               .unwrap_or("http://localhost:11434");
           debug!("Ollama base URL: {}", base_url);

           let client = providers::ollama::Client::new(base_url);
           let model = client.completion_model(&config.default_model
               .as_ref()
               .ok_or_else(|| anyhow!("Ollama requires default_model in config"))?);

           trace!("Ollama client created successfully");
           RigModel::Ollama(model)
       }
       _ => {
           return Err(anyhow!("Unknown provider type: {}", config.provider_type));
       }
   }
   ```
4. Update ProviderInstance impl to handle Ollama variant in complete():
   ```rust
   async fn complete(&self, input: AgentInput) -> Result<AgentOutput> {
       match &self.model {
           RigModel::OpenAI(model) => { /* existing */ }
           RigModel::Anthropic(model) => { /* existing */ }
           RigModel::Cohere(model) => { /* existing */ }
           RigModel::Ollama(model) => {
               info!("Ollama completion via rig");
               // Use same rig completion pattern as other providers
               model.complete(&input.prompt).await
           }
       }
   }
   ```
5. Verify rig-core dependency version supports Ollama (should be v0.20.0+)
6. Add unit test for Ollama variant creation

**Definition of Done:**
- [ ] Ollama variant compiles
- [ ] create_rig_provider handles "ollama" type
- [ ] complete() handles Ollama variant
- [ ] Unit test passes
- [ ] Zero clippy warnings
- [ ] Tracing comprehensive

---

### Task 11.2.2: Create OllamaModelManager for Model Operations

**File**: `llmspell-providers/src/local/ollama_manager.rs` (NEW FILE)
**Priority**: HIGH
**Estimated**: 4 hours
**Dependencies**: Task 11.1.3 (LocalProviderInstance trait)

**Context**: Rig handles inference, but we need ollama-rs for model management (list, pull, info, remove). Hybrid approach.

**Acceptance Criteria:**
- [ ] ollama-rs dependency added to Cargo.toml
- [ ] OllamaModelManager struct created
- [ ] health_check() implemented
- [ ] list_local_models() implemented
- [ ] pull_model() with progress tracking
- [ ] model_info() implemented
- [ ] All methods have comprehensive tracing
- [ ] Zero clippy warnings

**Implementation Steps:**
1. Add ollama-rs dependency to llmspell-providers/Cargo.toml:
   ```toml
   [dependencies]
   ollama-rs = "0.3.2"
   ```
2. Create `llmspell-providers/src/local/ollama_manager.rs`:
   ```rust
   //! Ollama model management using ollama-rs
   //!
   //! This module handles model operations (list, pull, info) via direct
   //! ollama-rs client, while inference goes through rig.

   use anyhow::{anyhow, Result};
   use ollama_rs::Ollama;
   use tracing::{info, debug, trace, warn, error};
   use super::{HealthStatus, LocalModel, PullProgress, ModelSpec, ModelInfo, DownloadStatus};

   /// Manager for Ollama model operations (not inference)
   pub struct OllamaModelManager {
       client: Ollama,
       base_url: String,
   }

   impl OllamaModelManager {
       pub fn new(base_url: impl Into<String>) -> Self {
           let base_url = base_url.into();
           info!("Initializing OllamaModelManager: {}", base_url);

           let client = Ollama::new(base_url.clone(), 11434);
           debug!("Ollama client created");

           Self { client, base_url }
       }

       pub async fn health_check(&self) -> Result<HealthStatus> {
           info!("Checking Ollama server health");
           trace!("Sending health check request to {}", self.base_url);

           match self.client.list_local_models().await {
               Ok(models) => {
                   let count = models.len();
                   debug!("Ollama healthy: {} models available", count);

                   // Try to get version if available
                   let version = None; // ollama-rs doesn't expose version yet

                   Ok(HealthStatus::Healthy {
                       available_models: count,
                       version,
                   })
               }
               Err(e) => {
                   warn!("Ollama health check failed: {}", e);
                   Ok(HealthStatus::Unhealthy {
                       reason: format!("Server not responding: {}", e),
                   })
               }
           }
       }

       pub async fn list_local_models(&self) -> Result<Vec<LocalModel>> {
           info!("Listing Ollama local models");
           trace!("Querying Ollama API for model list");

           let models = self.client.list_local_models().await
               .map_err(|e| {
                   error!("Failed to list Ollama models: {}", e);
                   anyhow!("Ollama list failed: {}", e)
               })?;

           debug!("Found {} Ollama models", models.len());

           let local_models = models.into_iter().map(|m| {
               trace!("Processing model: {}", m.name);
               LocalModel {
                   id: m.name.clone(),
                   backend: "ollama".to_string(),
                   size_bytes: m.size,
                   quantization: None, // Ollama doesn't expose this
                   modified_at: Some(m.modified_at),
               }
           }).collect();

           info!("Ollama model list complete");
           Ok(local_models)
       }

       pub async fn pull_model(&self, spec: &ModelSpec) -> Result<PullProgress> {
           let model_name = format!("{}:{}",
               spec.model,
               spec.variant.as_deref().unwrap_or("latest")
           );

           info!("Pulling Ollama model: {}", model_name);
           debug!("Model spec: {:?}", spec);

           // Start pull (ollama-rs provides progress streaming)
           trace!("Initiating Ollama pull request");

           self.client.pull_model(model_name.clone(), false).await
               .map_err(|e| {
                   error!("Ollama pull failed for {}: {}", model_name, e);
                   anyhow!("Pull failed: {}", e)
               })?;

           info!("Ollama model pull complete: {}", model_name);

           Ok(PullProgress {
               model_id: model_name,
               status: DownloadStatus::Complete,
               percent_complete: 100.0,
               bytes_downloaded: 0, // ollama-rs doesn't provide this
               bytes_total: None,
           })
       }

       pub async fn model_info(&self, model_id: &str) -> Result<ModelInfo> {
           info!("Getting Ollama model info: {}", model_id);
           trace!("Querying Ollama API for model details");

           let info = self.client.show_model_info(model_id.to_string()).await
               .map_err(|e| {
                   error!("Failed to get model info for {}: {}", model_id, e);
                   anyhow!("Model info failed: {}", e)
               })?;

           debug!("Model info retrieved for {}", model_id);

           Ok(ModelInfo {
               id: model_id.to_string(),
               backend: "ollama".to_string(),
               size_bytes: info.size,
               parameter_count: info.parameter_size.map(|s| s.to_string()),
               quantization: None,
               format: "Ollama".to_string(),
               loaded: false, // Ollama manages this internally
           })
       }
   }
   ```
3. Add to `llmspell-providers/src/local/mod.rs`:
   ```rust
   pub mod ollama_manager;
   pub use ollama_manager::OllamaModelManager;
   ```
4. Write unit tests with mock Ollama server

**Definition of Done:**
- [ ] OllamaModelManager compiles
- [ ] All methods implemented
- [ ] Tracing comprehensive
- [ ] Unit tests pass
- [ ] Zero clippy warnings

---

### Task 11.2.3: Implement OllamaProvider with LocalProviderInstance

**File**: `llmspell-providers/src/local/ollama_provider.rs` (NEW FILE)
**Priority**: CRITICAL
**Estimated**: 3 hours
**Dependencies**: Task 11.2.1, Task 11.2.2

**Context**: Wrapper that combines rig for inference + OllamaModelManager for model ops.

**Acceptance Criteria:**
- [ ] OllamaProvider implements LocalProviderInstance
- [ ] Inference delegated to rig
- [ ] Model management delegated to OllamaModelManager
- [ ] health_check() functional
- [ ] list_local_models() functional
- [ ] All trait methods implemented
- [ ] Zero clippy warnings

**Implementation Steps:**
1. Create `llmspell-providers/src/local/ollama_provider.rs`:
   ```rust
   //! Ollama provider combining rig inference + ollama-rs management

   use std::sync::Arc;
   use async_trait::async_trait;
   use anyhow::Result;
   use tracing::{info, debug};

   use crate::abstraction::{ProviderInstance, AgentInput, AgentOutput};
   use super::{LocalProviderInstance, OllamaModelManager, HealthStatus, LocalModel,
               PullProgress, ModelSpec, ModelInfo};

   /// Ollama provider using rig for inference, ollama-rs for management
   pub struct OllamaProvider {
       rig_provider: Arc<Box<dyn ProviderInstance>>,  // Rig handles inference
       manager: OllamaModelManager,                    // ollama-rs handles models
   }

   impl OllamaProvider {
       pub fn new(
           rig_provider: Box<dyn ProviderInstance>,
           base_url: impl Into<String>,
       ) -> Self {
           info!("Creating OllamaProvider with rig + ollama-rs hybrid");
           let manager = OllamaModelManager::new(base_url);
           debug!("OllamaProvider initialized");

           Self {
               rig_provider: Arc::new(rig_provider),
               manager,
           }
       }
   }

   #[async_trait]
   impl ProviderInstance for OllamaProvider {
       async fn complete(&self, input: AgentInput) -> Result<AgentOutput> {
           info!("OllamaProvider delegating completion to rig");
           // Delegate to rig provider
           self.rig_provider.complete(input).await
       }

       fn capabilities(&self) -> &ProviderCapabilities {
           self.rig_provider.capabilities()
       }
   }

   #[async_trait]
   impl LocalProviderInstance for OllamaProvider {
       async fn health_check(&self) -> Result<HealthStatus> {
           debug!("OllamaProvider health check");
           self.manager.health_check().await
       }

       async fn list_local_models(&self) -> Result<Vec<LocalModel>> {
           debug!("OllamaProvider listing models");
           self.manager.list_local_models().await
       }

       async fn pull_model(&self, spec: &ModelSpec) -> Result<PullProgress> {
           info!("OllamaProvider pulling model: {:?}", spec);
           self.manager.pull_model(spec).await
       }

       async fn model_info(&self, model_id: &str) -> Result<ModelInfo> {
           debug!("OllamaProvider getting model info: {}", model_id);
           self.manager.model_info(model_id).await
       }

       async fn unload_model(&self, _model_id: &str) -> Result<()> {
           // Ollama manages model loading internally, nothing to do
           Ok(())
       }
   }
   ```
2. Update provider factory registration to create OllamaProvider
3. Write integration tests

**Definition of Done:**
- [ ] OllamaProvider compiles
- [ ] Both traits implemented
- [ ] Delegation works correctly
- [ ] Tests pass
- [ ] Zero clippy warnings

---

## SECTION 3: Kernel Protocol Extension

### Task 11.3.1: Add ModelRequest/ModelReply Message Types

**File**: `llmspell-kernel/src/protocol.rs`
**Priority**: CRITICAL
**Estimated**: 2 hours
**Dependencies**: None

**Context**: Need new message types for model operations, similar to existing tool_request pattern.

**Acceptance Criteria:**
- [ ] ModelRequest message type added to KernelMessage enum
- [ ] ModelReply message type added to KernelMessage enum
- [ ] Serialization/deserialization works
- [ ] Message types follow existing pattern
- [ ] Zero clippy warnings

**Implementation Steps:**
1. Read existing protocol.rs to understand message pattern
2. Add new message types to KernelMessage enum:
   ```rust
   // llmspell-kernel/src/protocol.rs (EXTEND existing enum)

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
3. Add unit tests for serialization:
   ```rust
   #[test]
   fn test_model_request_serialization() {
       let msg = KernelMessage::ModelRequest {
           content: json!({"command": "list", "backend": "ollama"}),
       };
       let serialized = serde_json::to_string(&msg).unwrap();
       assert!(serialized.contains("model_request"));
   }
   ```

**Definition of Done:**
- [ ] Message types added
- [ ] Serialization tests pass
- [ ] Pattern matches existing messages
- [ ] Zero clippy warnings

---

### Task 11.3.2: Implement handle_model_request in Kernel

**File**: `llmspell-kernel/src/handlers/mod.rs` (NEW HANDLER)
**Priority**: CRITICAL
**Estimated**: 6 hours
**Dependencies**: Task 11.3.1, Task 11.2.3

**Context**: Kernel handler that processes model commands (list, pull, status) using ProviderManager.

**Acceptance Criteria:**
- [ ] handle_model_request() function implemented
- [ ] "list" command queries both Ollama and Candle
- [ ] "pull" command downloads models
- [ ] "status" command checks backend health
- [ ] "info" command returns model details
- [ ] All commands have comprehensive tracing
- [ ] Error handling robust
- [ ] Zero clippy warnings

**Implementation Steps:**
1. Add to llmspell-kernel/src/handlers/mod.rs:
   ```rust
   //! Model request handler - similar to tool_request handler

   use anyhow::{anyhow, Result};
   use serde_json::{json, Value};
   use tracing::{info, debug, trace, warn, error};
   use crate::component_registry::ComponentRegistry;
   use llmspell_providers::local::{ModelSpec, LocalProviderInstance};

   /// Handle model management requests from CLI
   pub async fn handle_model_request(
       content: Value,
       registry: &ComponentRegistry,
   ) -> Result<Value> {
       let command = content.get("command")
           .and_then(|c| c.as_str())
           .ok_or_else(|| {
               error!("Model request missing 'command' field");
               anyhow!("Missing command field")
           })?;

       info!("Handling model request: command={}", command);
       trace!("Request content: {:?}", content);

       let provider_manager = registry.get_provider_manager();

       match command {
           "list" => {
               info!("Processing model list request");
               let backend = content.get("backend")
                   .and_then(|b| b.as_str())
                   .unwrap_or("all");

               debug!("Backend filter: {}", backend);
               let mut models = Vec::new();

               // Query Ollama
               if backend == "all" || backend == "ollama" {
                   trace!("Querying Ollama provider");
                   if let Ok(Some(ollama)) = provider_manager.get_local_provider("ollama").await {
                       match ollama.list_local_models().await {
                           Ok(ollama_models) => {
                               debug!("Found {} Ollama models", ollama_models.len());
                               models.extend(ollama_models);
                           }
                           Err(e) => {
                               warn!("Failed to list Ollama models: {}", e);
                           }
                       }
                   } else {
                       debug!("Ollama provider not available");
                   }
               }

               // Query Candle
               if backend == "all" || backend == "candle" {
                   trace!("Querying Candle provider");
                   if let Ok(Some(candle)) = provider_manager.get_local_provider("candle").await {
                       match candle.list_local_models().await {
                           Ok(candle_models) => {
                               debug!("Found {} Candle models", candle_models.len());
                               models.extend(candle_models);
                           }
                           Err(e) => {
                               warn!("Failed to list Candle models: {}", e);
                           }
                       }
                   } else {
                       debug!("Candle provider not available");
                   }
               }

               info!("Model list complete: {} total models", models.len());
               Ok(json!({
                   "models": models,
                   "count": models.len()
               }))
           }

           "pull" => {
               info!("Processing model pull request");
               let model_spec = content.get("model")
                   .and_then(|m| m.as_str())
                   .ok_or_else(|| {
                       error!("Pull request missing 'model' field");
                       anyhow!("Missing model field")
                   })?;

               debug!("Parsing model spec: {}", model_spec);
               let spec = ModelSpec::parse(model_spec)?;
               let backend = spec.backend.as_deref()
                   .ok_or_else(|| anyhow!("Backend must be specified for pull"))?;

               trace!("Backend: {}, Model: {}", backend, spec.model);

               let provider = provider_manager.get_local_provider(backend).await?
                   .ok_or_else(|| {
                       error!("Provider {} not available", backend);
                       anyhow!("Provider {} not available", backend)
                   })?;

               info!("Pulling model via {} backend", backend);
               let progress = provider.pull_model(&spec).await?;

               info!("Model pull complete: {}", progress.model_id);
               Ok(json!({
                   "status": format!("Model {} downloaded successfully", progress.model_id),
                   "model_id": progress.model_id,
                   "percent_complete": progress.percent_complete
               }))
           }

           "status" => {
               info!("Processing status request");
               let mut status = json!({});

               // Check Ollama
               trace!("Checking Ollama status");
               if let Ok(Some(ollama)) = provider_manager.get_local_provider("ollama").await {
                   match ollama.health_check().await {
                       Ok(health) => {
                           debug!("Ollama health: {:?}", health);
                           status["ollama"] = json!(health);
                       }
                       Err(e) => {
                           warn!("Ollama health check failed: {}", e);
                       }
                   }
               } else {
                   debug!("Ollama provider not configured");
               }

               // Check Candle
               trace!("Checking Candle status");
               if let Ok(Some(candle)) = provider_manager.get_local_provider("candle").await {
                   match candle.health_check().await {
                       Ok(health) => {
                           debug!("Candle health: {:?}", health);
                           status["candle"] = json!(health);
                       }
                       Err(e) => {
                           warn!("Candle health check failed: {}", e);
                       }
                   }
               } else {
                   debug!("Candle provider not configured");
               }

               info!("Status check complete");
               Ok(status)
           }

           "info" => {
               info!("Processing model info request");
               let model_id = content.get("model")
                   .and_then(|m| m.as_str())
                   .ok_or_else(|| anyhow!("Missing model field"))?;

               // Determine backend from model_id or explicit param
               let backend = content.get("backend")
                   .and_then(|b| b.as_str())
                   .ok_or_else(|| anyhow!("Backend must be specified for info"))?;

               debug!("Getting info for {} from {} backend", model_id, backend);

               let provider = provider_manager.get_local_provider(backend).await?
                   .ok_or_else(|| anyhow!("Provider {} not available", backend))?;

               let info = provider.model_info(model_id).await?;

               info!("Model info retrieved for {}", model_id);
               Ok(json!(info))
           }

           _ => {
               error!("Unknown model command: {}", command);
               Err(anyhow!("Unknown model command: {}", command))
           }
       }
   }
   ```
2. Wire up handler in kernel message dispatch loop
3. Write unit tests with mock ComponentRegistry

**Definition of Done:**
- [ ] All commands implemented
- [ ] Tracing comprehensive
- [ ] Error handling robust
- [ ] Unit tests pass
- [ ] Zero clippy warnings

---

### Task 11.3.3: Add send_model_request to KernelHandle and ClientHandle

**File**: `llmspell-kernel/src/api.rs`
**Priority**: CRITICAL
**Estimated**: 2 hours
**Dependencies**: Task 11.3.1

**Context**: CLI needs to send model_request messages to kernel (both embedded and remote modes).

**Acceptance Criteria:**
- [ ] KernelHandle.send_model_request() added
- [ ] ClientHandle.send_model_request() added
- [ ] Both methods async
- [ ] Return serde_json::Value
- [ ] Pattern matches existing send_tool_request()
- [ ] Zero clippy warnings

**Implementation Steps:**
1. Read existing send_tool_request() in api.rs for pattern
2. Add to KernelHandle:
   ```rust
   impl KernelHandle {
       // Existing methods...

       /// Send model management request to kernel
       pub async fn send_model_request(
           &mut self,
           content: serde_json::Value,
       ) -> Result<serde_json::Value> {
           info!("Sending model request to kernel");
           debug!("Request content: {:?}", content);

           // Create model_request message
           let msg = KernelMessage::ModelRequest { content };

           // Send to kernel and await reply
           let reply = self.send_message_and_await_reply(msg).await?;

           // Extract content from ModelReply
           match reply {
               KernelMessage::ModelReply { content, status } => {
                   debug!("Model reply received: status={:?}", status);
                   Ok(content)
               }
               _ => {
                   error!("Unexpected reply type for model request");
                   Err(anyhow!("Unexpected reply type"))
               }
           }
       }
   }
   ```
3. Add to ClientHandle with same signature
4. Write unit tests

**Definition of Done:**
- [ ] Both methods added
- [ ] Implementation follows pattern
- [ ] Tests pass
- [ ] Zero clippy warnings

---

## SECTION 4: CLI Implementation (Dual-Mode)

### Task 11.4.1: Create ModelCommands Enum

**File**: `llmspell-cli/src/cli.rs`
**Priority**: CRITICAL
**Estimated**: 2 hours
**Dependencies**: None

**Context**: Add Model subcommand to Commands enum with all model management subcommands.

**Acceptance Criteria:**
- [ ] ModelCommands enum created with all subcommands
- [ ] Model variant added to Commands enum
- [ ] Clap derives correct
- [ ] Help text comprehensive
- [ ] Compiles without warnings
- [ ] Zero clippy warnings

**Implementation Steps:**
1. Add to llmspell-cli/src/cli.rs:
   ```rust
   #[derive(Debug, Subcommand)]
   pub enum Commands {
       // ... existing commands (Run, Exec, Repl, Debug, etc.)

       /// Manage local LLM models (Ollama and Candle) (NEW)
       #[command(subcommand)]
       Model(ModelCommands),
   }

   /// Model management commands
   #[derive(Debug, Subcommand)]
   pub enum ModelCommands {
       /// List installed local models
       List {
           /// Filter by backend (ollama, candle, or all)
           #[arg(long, default_value = "all")]
           backend: String,

           /// Show verbose output with sizes and dates
           #[arg(long, short)]
           verbose: bool,

           /// Output format override
           #[arg(long)]
           format: Option<OutputFormat>,
       },

       /// Download a model
       Pull {
           /// Model specification (e.g., "ollama/llama3.1:8b" or "candle/mistral:7b")
           model: String,

           /// Force re-download even if exists
           #[arg(long, short)]
           force: bool,

           /// Quantization level for Candle models
           #[arg(long, default_value = "Q4_K_M")]
           quantization: String,
       },

       /// Remove a model
       Remove {
           /// Model identifier
           model: String,

           /// Skip confirmation prompt
           #[arg(long, short = 'y')]
           yes: bool,
       },

       /// Show model information
       Info {
           /// Model identifier
           model: String,
       },

       /// List available models from library
       Available {
           /// Backend to query (ollama or candle)
           #[arg(long)]
           backend: Option<String>,

           /// Show only recommended models
           #[arg(long)]
           recommended: bool,
       },

       /// Check local LLM installation status
       Status,

       /// Install Ollama binary (macOS and Linux only)
       InstallOllama,
   }
   ```
2. Update help text with examples
3. Test CLI parsing with clap

**Definition of Done:**
- [ ] Enum compiles
- [ ] Help text shows correctly
- [ ] Clap parsing works
- [ ] Zero clippy warnings

---

### Task 11.4.2: Add Model Command to execute_command

**File**: `llmspell-cli/src/commands/mod.rs` (around line 223)
**Priority**: CRITICAL
**Estimated**: 1 hour
**Dependencies**: Task 11.4.1

**Context**: Wire Model command into main command dispatcher.

**Acceptance Criteria:**
- [ ] Commands::Model case added to execute_command
- [ ] Calls model::handle_model_command
- [ ] Pattern matches existing commands
- [ ] Compiles without warnings
- [ ] Zero clippy warnings

**Implementation Steps:**
1. Add to execute_command match in commands/mod.rs:
   ```rust
   // llmspell-cli/src/commands/mod.rs

   pub async fn execute_command(
       command: Commands,
       runtime_config: LLMSpellConfig,
       output_format: OutputFormat,
   ) -> Result<()> {
       match command {
           // ... existing commands

           Commands::Tool { command, source } => {
               tool::handle_tool_command(command, source, runtime_config, output_format).await
           }

           // NEW: Model command
           Commands::Model(model_cmd) => {
               model::handle_model_command(model_cmd, runtime_config, output_format).await
           }

           Commands::Version(version_cmd) => version::execute(version_cmd, output_format).await,
       }
   }
   ```
2. Ensure model module declared in commands/mod.rs

**Definition of Done:**
- [ ] Case added
- [ ] Compiles
- [ ] Pattern correct
- [ ] Zero clippy warnings

---

### Task 11.4.3: Implement Dual-Mode Model Command Handlers

**File**: `llmspell-cli/src/commands/model.rs` (NEW FILE)
**Priority**: CRITICAL
**Estimated**: 8 hours
**Dependencies**: Task 11.3.3, Task 11.4.1

**Context**: Implement handle_model_command following tool.rs dual-mode pattern (embedded vs remote kernel).

**Acceptance Criteria:**
- [ ] handle_model_command() dispatches to embedded or remote
- [ ] handle_model_embedded() sends requests to kernel via KernelHandle
- [ ] handle_model_remote() sends requests to kernel via ClientHandle
- [ ] All ModelCommands variants handled
- [ ] Pattern matches tool.rs exactly
- [ ] Comprehensive tracing
- [ ] Zero clippy warnings

**Implementation Steps:**
1. Create llmspell-cli/src/commands/model.rs following tool.rs:
   ```rust
   //! Model command implementation - sends model requests to kernel
   //!
   //! Follows tool.rs pattern: dual-mode with embedded and remote handlers

   use anyhow::{anyhow, Result};
   use serde_json::json;
   use tracing::{info, instrument, trace, warn};

   use crate::cli::{ModelCommands, OutputFormat};
   use crate::execution_context::ExecutionContext;
   use crate::output::OutputFormatter;
   use llmspell_config::LLMSpellConfig;

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
           None, // No connect string for model commands
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

               trace!("Sending model_request to kernel");
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

               debug!("Received {} models from kernel", models.len());

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

               // Check for error
               if let Some(error) = response.get("error") {
                   return Err(anyhow!("Model pull error: {}", error));
               }

               // Display status
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

               // Format and display
               let formatter = OutputFormatter::new(output_format);
               formatter.print_json(&response)?;
               Ok(())
           }

           ModelCommands::Info { model } => {
               info!("Getting model info: {} via kernel", model);

               // Need to determine backend - could parse from model or require explicit
               let request_content = json!({
                   "command": "info",
                   "model": model,
               });

               let response = handle.send_model_request(request_content).await?;

               if let Some(error) = response.get("error") {
                   return Err(anyhow!("Model info error: {}", error));
               }

               let formatter = OutputFormatter::new(output_format);
               formatter.print_json(&response)?;
               Ok(())
           }

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
2. Add module declaration in commands/mod.rs:
   ```rust
   pub mod model;
   ```
3. Write unit tests with mock handles

**Definition of Done:**
- [ ] All handlers implemented
- [ ] Dual-mode pattern followed
- [ ] Tracing comprehensive
- [ ] Tests pass
- [ ] Zero clippy warnings

---

## SECTION 5: Bridge Layer Integration

### Task 11.5.1: Create LocalLLM Global Object Injection

**File**: `llmspell-bridge/src/lua/globals/local_llm.rs` (NEW FILE)
**Priority**: HIGH
**Estimated**: 3 hours
**Dependencies**: Task 11.2.3 (OllamaProvider)

**Context**: Inject LocalLLM global into Lua for script access to local models.

**Acceptance Criteria:**
- [ ] inject_local_llm_global() function created
- [ ] LocalLLM table injected into Lua globals
- [ ] ProviderManager passed via context
- [ ] Compilation succeeds
- [ ] Zero clippy warnings

**Implementation Steps:**
1. Create llmspell-bridge/src/lua/globals/local_llm.rs:
   ```rust
   //! LocalLLM global object for Lua scripts

   use mlua::{Lua, Table, Result as LuaResult};
   use std::sync::Arc;
   use tracing::{info, debug};

   use crate::globals::GlobalContext;
   use llmspell_providers::abstraction::ProviderManager;

   /// Inject LocalLLM global object into Lua
   pub fn inject_local_llm_global(
       lua: &Lua,
       context: &GlobalContext,
       provider_manager: Arc<ProviderManager>,
   ) -> LuaResult<()> {
       info!("Injecting LocalLLM global object");

       let local_llm_table = lua.create_table()?;

       // Register methods (implemented in next tasks)
       register_status_method(lua, &local_llm_table, provider_manager.clone())?;
       register_list_method(lua, &local_llm_table, provider_manager.clone())?;
       register_pull_method(lua, &local_llm_table, provider_manager.clone())?;

       lua.globals().set("LocalLLM", local_llm_table)?;
       debug!("LocalLLM global registered successfully");
       Ok(())
   }

   // Method implementations in next task
   fn register_status_method(
       lua: &Lua,
       table: &Table,
       provider_manager: Arc<ProviderManager>,
   ) -> LuaResult<()> {
       // Implemented in Task 5.2
       Ok(())
   }

   fn register_list_method(
       lua: &Lua,
       table: &Table,
       provider_manager: Arc<ProviderManager>,
   ) -> LuaResult<()> {
       // Implemented in Task 5.3
       Ok(())
   }

   fn register_pull_method(
       lua: &Lua,
       table: &Table,
       provider_manager: Arc<ProviderManager>,
   ) -> LuaResult<()> {
       // Implemented in Task 5.4
       Ok(())
   }
   ```
2. Update bridge initialization to call inject_local_llm_global
3. Write basic injection test

**Definition of Done:**
- [ ] Global injection works
- [ ] ProviderManager accessible
- [ ] Test passes
- [ ] Zero clippy warnings

---

### Task 11.5.2: Implement LocalLLM.status() Method

**File**: `llmspell-bridge/src/lua/globals/local_llm.rs`
**Priority**: HIGH
**Estimated**: 2 hours
**Dependencies**: Task 11.5.1

**Context**: Implement status() method to check backend availability from Lua.

**Acceptance Criteria:**
- [ ] status() returns table with backend status
- [ ] Checks both Ollama and Candle
- [ ] Async execution via tokio
- [ ] Error handling proper
- [ ] Tracing comprehensive
- [ ] Zero clippy warnings

**Implementation Steps:**
1. Implement register_status_method:
   ```rust
   fn register_status_method(
       lua: &Lua,
       table: &Table,
       provider_manager: Arc<ProviderManager>,
   ) -> LuaResult<()> {
       let status_fn = lua.create_async_function(move |lua, ()| {
           let pm = provider_manager.clone();
           async move {
               info!("LocalLLM.status() called from script");

               // Check Ollama
               debug!("Checking Ollama status");
               let ollama_status = match pm.get_local_provider("ollama").await {
                   Ok(Some(provider)) => {
                       match provider.health_check().await {
                           Ok(health) => Some(health),
                           Err(e) => {
                               warn!("Ollama health check failed: {}", e);
                               None
                           }
                       }
                   }
                   _ => None,
               };

               // Check Candle
               debug!("Checking Candle status");
               let candle_status = match pm.get_local_provider("candle").await {
                   Ok(Some(provider)) => {
                       match provider.health_check().await {
                           Ok(health) => Some(health),
                           Err(e) => {
                               warn!("Candle health check failed: {}", e);
                               None
                           }
                       }
                   }
                   _ => None,
               };

               // Create result table
               let result = lua.create_table()?;

               // Ollama status
               let ollama_table = lua.create_table()?;
               ollama_table.set("running", ollama_status.is_some())?;
               if let Some(HealthStatus::Healthy { available_models, .. }) = ollama_status {
                   ollama_table.set("models", available_models)?;
               }
               result.set("ollama", ollama_table)?;

               // Candle status
               let candle_table = lua.create_table()?;
               candle_table.set("ready", candle_status.is_some())?;
               if let Some(HealthStatus::Healthy { available_models, .. }) = candle_status {
                   candle_table.set("models", available_models)?;
               }
               result.set("candle", candle_table)?;

               debug!("Status check complete");
               Ok(result)
           }
       })?;

       table.set("status", status_fn)?;
       Ok(())
   }
   ```
2. Write Lua test:
   ```lua
   local status = LocalLLM.status()
   assert(status.ollama ~= nil)
   assert(status.candle ~= nil)
   ```

**Definition of Done:**
- [ ] Method works from Lua
- [ ] Returns correct status
- [ ] Tests pass
- [ ] Zero clippy warnings

---

### Task 11.5.3: Implement LocalLLM.list() Method

**File**: `llmspell-bridge/src/lua/globals/local_llm.rs`
**Priority**: HIGH
**Estimated**: 2 hours
**Dependencies**: Task 11.5.1

**Context**: Implement list() to get local models from Lua.

**Acceptance Criteria:**
- [ ] list() returns array of model tables
- [ ] Queries both backends
- [ ] Model metadata included
- [ ] Tests pass
- [ ] Zero clippy warnings

**Implementation Steps:**
1. Implement register_list_method similar to status
2. Convert Vec<LocalModel> to Lua tables
3. Write Lua test

**Definition of Done:**
- [ ] Method functional
- [ ] Returns model data
- [ ] Tests pass
- [ ] Zero clippy warnings

---

### Task 11.5.4: Update Agent.create() for Local Models

**File**: `llmspell-bridge/src/lua/globals/agent.rs`
**Priority**: CRITICAL
**Estimated**: 3 hours
**Dependencies**: Task 11.1.1 (ModelSpecifier extension)

**Context**: Update Agent.create() to parse and handle `model = "local/llama3.1:8b"` syntax.

**Acceptance Criteria:**
- [ ] Parses "local/model:variant" syntax
- [ ] Parses "local/model:variant@backend" syntax
- [ ] Routes to LocalProviderFactory via ProviderManager
- [ ] Backend auto-detection works
- [ ] Backward compatibility maintained
- [ ] Tests comprehensive
- [ ] Zero clippy warnings

**Implementation Steps:**
1. Read existing Agent.create() implementation in agent.rs
2. Update model parsing to use extended ModelSpecifier
3. No changes needed if ProviderManager routing is correct (Task 1.2)
4. Write tests with local model syntax:
   ```lua
   -- Test auto-detection
   local agent1 = Agent.create({model = "local/llama3.1:8b"})

   -- Test explicit Ollama
   local agent2 = Agent.create({model = "local/phi3:3.8b@ollama"})

   -- Test explicit Candle
   local agent3 = Agent.create({model = "local/mistral:7b@candle"})
   ```

**Definition of Done:**
- [ ] Local syntax works
- [ ] All modes tested
- [ ] Backward compatible
- [ ] Zero clippy warnings

---

## SECTION 6: Candle Implementation

### Task 11.6.1: Add Candle Dependencies

**File**: `llmspell-providers/Cargo.toml`
**Priority**: CRITICAL
**Estimated**: 1 hour
**Dependencies**: None

**Context**: Add candle-core, candle-transformers, and related dependencies.

**Acceptance Criteria:**
- [ ] candle-core added
- [ ] candle-transformers added
- [ ] hf-hub added for downloads
- [ ] tokenizers added
- [ ] Dependencies compile
- [ ] Zero clippy warnings

**Implementation Steps:**
1. Add to Cargo.toml:
   ```toml
   [dependencies]
   candle-core = "0.7"
   candle-transformers = "0.7"
   hf-hub = "0.3"
   tokenizers = "0.20"
   ```
2. Run cargo check

**Definition of Done:**
- [ ] Dependencies compile
- [ ] Versions compatible
- [ ] Zero warnings

---

### Task 11.6.2: Implement GGUF Model Loading

**File**: `llmspell-providers/src/local/candle/gguf_loader.rs` (NEW FILE)
**Priority**: CRITICAL
**Estimated**: 8 hours
**Dependencies**: Task 11.6.1

**Context**: Core GGUF loading from local files and HuggingFace.

**Acceptance Criteria:**
- [ ] Loads GGUF files from disk
- [ ] Downloads from HuggingFace if not present
- [ ] Validates model format
- [ ] Loads tokenizer
- [ ] Device detection (CPU/CUDA/Metal)
- [ ] Comprehensive tracing
- [ ] Unit tests with fixture models
- [ ] Zero clippy warnings

**Implementation Steps:**
1. Create directory structure: `llmspell-providers/src/local/candle/`
2. Implement GGUFLoader (see design doc Section 3.1 for detailed code)
3. Implement device detection
4. Implement model loading lifecycle
5. Add tracing at all levels
6. Write tests with small test models

**Definition of Done:**
- [ ] GGUF loading works
- [ ] HuggingFace integration functional
- [ ] Tests pass
- [ ] Tracing comprehensive
- [ ] Zero clippy warnings

---

### Task 11.6.3: Implement CandleProvider with Inference Loop

**File**: `llmspell-providers/src/local/candle/provider.rs` (NEW FILE)
**Priority**: CRITICAL
**Estimated**: 10 hours
**Dependencies**: Task 11.6.2

**Context**: CandleProvider with text generation inference.

**Acceptance Criteria:**
- [ ] Implements ProviderInstance trait
- [ ] Implements LocalProviderInstance trait
- [ ] Text generation with sampling
- [ ] Streaming support
- [ ] Temperature, top_p, top_k parameters
- [ ] >20 tokens/sec for 7B models
- [ ] Memory <5GB for Q4_K_M
- [ ] Tests pass
- [ ] Zero clippy warnings

**Implementation Steps:**
1. Implement CandleProvider struct (see design doc Section 3.1)
2. Implement generation loop with sampling
3. Implement model caching
4. Benchmark performance
5. Optimize if needed
6. Write comprehensive tests

**Definition of Done:**
- [ ] Inference works
- [ ] Performance targets met
- [ ] Tests pass
- [ ] Zero clippy warnings

---

## SECTION 7: Testing & Validation

### Task 11.7.1: Unit Test Suite

**Priority**: CRITICAL
**Estimated**: 8 hours

**Acceptance Criteria:**
- [ ] >90% coverage for new code
- [ ] All providers tested
- [ ] ModelSpecifier tests comprehensive
- [ ] Kernel protocol tests
- [ ] CLI handler tests
- [ ] Bridge layer tests
- [ ] All tests pass in CI
- [ ] Zero clippy warnings

**Implementation Steps:**
1. Write provider unit tests
2. Write protocol unit tests
3. Write CLI unit tests
4. Write bridge unit tests
5. Ensure >90% coverage
6. Run in CI

**Definition of Done:**
- [ ] Coverage >90%
- [ ] All tests pass
- [ ] CI integrated
- [ ] Zero warnings

---

### Task 11.7.2: Integration Tests

**Priority**: HIGH
**Estimated**: 6 hours

**Acceptance Criteria:**
- [ ] End-to-end workflows tested
- [ ] Backend switching tested
- [ ] Error scenarios covered
- [ ] Example applications validated
- [ ] All tests pass

**Implementation Steps:**
1. Write end-to-end test scenarios
2. Test both backends
3. Test error conditions
4. Validate all examples
5. Document test scenarios

**Definition of Done:**
- [ ] All scenarios tested
- [ ] Tests reliable
- [ ] Documentation complete

---

### Task 11.7.3: Performance Benchmarks

**Priority**: HIGH
**Estimated**: 4 hours

**Acceptance Criteria:**
- [ ] Ollama: <100ms first token
- [ ] Candle: <200ms first token
- [ ] Both: >20 tokens/sec
- [ ] Memory <5GB
- [ ] Benchmarks reproducible

**Implementation Steps:**
1. Create benches/ollama_bench.rs
2. Create benches/candle_bench.rs
3. Run on target hardware
4. Document results
5. Set up regression detection

**Definition of Done:**
- [ ] Targets met
- [ ] Benchmarks reproducible
- [ ] Results documented

---

## SECTION 8: Documentation

### Task 11.8.1: API Documentation

**Priority**: HIGH
**Estimated**: 4 hours

**Acceptance Criteria:**
- [ ] All public items documented
- [ ] Examples in doc comments
- [ ] Migration guide created
- [ ] README updated
- [ ] Docs build without warnings

**Implementation Steps:**
1. Document all provider APIs
2. Document CLI commands
3. Document script APIs
4. Create migration guide
5. Generate docs with cargo doc

**Definition of Done:**
- [ ] Docs coverage >95%
- [ ] Examples compile
- [ ] Migration guide helpful

---

### Task 11.8.2: User Guide

**Priority**: HIGH
**Estimated**: 4 hours

**Acceptance Criteria:**
- [ ] Getting started guide
- [ ] Model installation guide
- [ ] Configuration guide
- [ ] Troubleshooting section
- [ ] Example walkthroughs

**Implementation Steps:**
1. Write getting started
2. Document model installation
3. Explain configuration
4. Create troubleshooting FAQ
5. Walk through examples

**Definition of Done:**
- [ ] Guide comprehensive
- [ ] Examples documented
- [ ] Troubleshooting helpful

---

### Task 11.8.3: Example Applications

**Priority**: MEDIUM
**Estimated**: 5 hours

**Acceptance Criteria:**
- [ ] local_chat.lua example
- [ ] ollama_chat.lua example
- [ ] candle_inference.lua example
- [ ] backend_comparison.lua example
- [ ] All examples run successfully

**Implementation Steps:**
1. Create local_chat.lua
2. Create backend-specific examples
3. Create comparison example
4. Test all examples
5. Document in README

**Definition of Done:**
- [ ] All examples work
- [ ] Well-documented
- [ ] README complete

---

## Final Validation Checklist

### Quality Gates
- [ ] All crates compile: `cargo build --workspace --all-features`
- [ ] Clippy passes: `cargo clippy --workspace --all-features --all-targets`
- [ ] Format compliance: `cargo fmt --all --check`
- [ ] Tests pass: `cargo test --workspace --all-features`
- [ ] Docs build: `cargo doc --workspace --all-features --no-deps`
- [ ] Examples run successfully

### Performance Targets
- [ ] Ollama: <100ms first token
- [ ] Candle: <200ms first token
- [ ] Both: >20 tokens/sec for 7B models
- [ ] Memory: <5GB for Q4_K_M models

### Feature Completeness
- [ ] `llmspell model status` works
- [ ] `llmspell model list` works
- [ ] `llmspell model pull ollama/llama3.1:8b` works
- [ ] `llmspell model pull candle/mistral:7b` works
- [ ] LocalLLM.status() works from Lua
- [ ] LocalLLM.list() works from Lua
- [ ] Agent.create({model = "local/llama3.1:8b"}) works
- [ ] Backend auto-detection works

### Architecture Validation
- [ ] Uses rig for Ollama inference (not direct ollama-rs)
- [ ] Hybrid approach (rig + ollama-rs) implemented
- [ ] Kernel message protocol extended (ModelRequest/ModelReply)
- [ ] Dual-mode CLI handlers follow tool.rs pattern
- [ ] Flat config structure using existing HashMap
- [ ] ModelSpecifier extended with backend field
- [ ] LocalProviderInstance trait implemented
- [ ] Provider routing uses existing factory pattern

---

## Risk Mitigation

### Technical Risks
1. **Rig Ollama support incomplete**: Verified via web research - v0.20.0 supports Ollama
2. **Candle GGUF complexity**: Use candle-transformers, comprehensive error handling
3. **Performance targets**: Early benchmarking, GPU acceleration, quantization
4. **Model compatibility**: Test with recommended models (Phi-3, LLaMA 3.1, Mistral)

### Integration Risks
1. **Kernel protocol breaking changes**: Follow exact pattern from tool_request
2. **CLI pattern deviation**: Use tool.rs as template, verify with tests
3. **Config merge issues**: Verified flat structure works with existing code

---

**END OF COMPREHENSIVE PHASE 11 TODO**

This TODO comprehensively covers all integration points from the updated design doc with specific file paths, line numbers, code examples, and validation criteria.
