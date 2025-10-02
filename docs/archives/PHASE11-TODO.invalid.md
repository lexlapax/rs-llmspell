# Phase 11: Local LLM Integration - TODO List

**Version**: 1.0
**Date**: October 2025
**Status**: Implementation Ready
**Phase**: 11 (Local LLM Integration)
**Timeline**: Weeks 37-41 (20 working days)
**Priority**: CRITICAL (Enables offline and cost-free LLM operations)
**Dependencies**: Phase 10 Service Integration ✅
**Arch-Document**: docs/technical/master-architecture-vision.md
**All-Phases-Document**: docs/in-progress/implementation-phases.md
**Design-Document**: docs/in-progress/phase-11-design-doc.md
**This-document**: working copy /TODO.md (pristine/immutable copy in docs/in-progress/PHASE11-DONE.md)

> **📋 Actionable Task List**: This document breaks down Phase 11 implementation into specific, measurable tasks for building production-ready local LLM support via both Ollama (external process) and Candle (embedded Rust) with unified model management.

---

## Overview

**Goal**: Implement dual-path local LLM support via Ollama and Candle with unified CLI, script API, and model management infrastructure.

**Success Criteria Summary:**
- [ ] `llmspell-providers` compiles with Ollama and Candle providers
- [ ] Zero clippy warnings across all new code
- [ ] `llmspell model` CLI commands functional
- [ ] LocalLLM global accessible from Lua scripts
- [ ] Ollama provider: <100ms first token latency
- [ ] Candle provider: <200ms first token latency
- [ ] Both providers: >20 tokens/sec for 7B models
- [ ] Memory usage <5GB RAM for Q4_K_M models
- [ ] 5 example applications working
- [ ] >90% test coverage for new code
- [ ] Documentation complete with migration guide

---

## Phase 11.1: Ollama Provider Implementation (Days 1-5)

### Task 11.1.1: Add Ollama Dependencies and Types
**Priority**: CRITICAL
**Estimated Time**: 2 hours
**Assignee**: Provider Team Lead
**Dependencies**: None

**Description**: Add ollama-rs crate dependency and create Ollama-specific types in llmspell-providers.

**Acceptance Criteria:**
- [ ] `ollama-rs = "0.3.2"` added to llmspell-providers/Cargo.toml
- [ ] `OllamaConfig` struct created with url, port, timeout, auto_start fields
- [ ] `OllamaModel` enum created for model tracking
- [ ] Types are Send + Sync + Clone where appropriate
- [ ] `cargo check -p llmspell-providers` passes
- [ ] Zero clippy warnings: `cargo clippy -p llmspell-providers`

**Implementation Steps:**
1. Add ollama-rs dependency to llmspell-providers/Cargo.toml
2. Create `llmspell-providers/src/ollama/mod.rs`
3. Create `llmspell-providers/src/ollama/types.rs`:
   ```rust
   use serde::{Deserialize, Serialize};

   #[derive(Debug, Clone, Serialize, Deserialize)]
   pub struct OllamaConfig {
       pub url: Option<String>,
       pub port: Option<u16>,
       pub timeout_seconds: Option<u64>,
       pub auto_start: bool,
   }

   #[derive(Debug, Clone)]
   pub struct OllamaModel {
       pub name: String,
       pub variant: String,
       pub size_bytes: u64,
   }
   ```
4. Update llmspell-providers/src/lib.rs to export ollama module
5. Run `cargo check` and `cargo clippy`

**Definition of Done:**
- [ ] All types compile without warnings
- [ ] Clippy passes with zero warnings
- [ ] Types have proper Debug, Clone traits
- [ ] Documentation comments complete

---

### Task 11.1.2: Implement OllamaProvider
**Priority**: CRITICAL
**Estimated Time**: 6 hours
**Assignee**: Provider Team
**Dependencies**: Task 11.1.1

**Description**: Implement OllamaProvider struct that implements ProviderInstance trait.

**Acceptance Criteria:**
- [ ] `OllamaProvider` struct implements `ProviderInstance` trait
- [ ] Health check method (`check_health`) works
- [ ] Model listing method (`list_local_models`) functional
- [ ] Completion method with streaming support
- [ ] Comprehensive tracing with info!, debug!, trace!, warn!, error! macros
- [ ] Unit tests pass with >90% coverage
- [ ] Zero clippy warnings

**Implementation Steps:**
1. Create `llmspell-providers/src/ollama/provider.rs`:
   ```rust
   use crate::abstraction::{ProviderInstance, ProviderCapabilities};
   use anyhow::Result;
   use tracing::{info, debug, trace, warn, error};

   pub struct OllamaProvider {
       client: ollama_rs::Ollama,
       config: OllamaConfig,
   }

   impl OllamaProvider {
       pub fn new(config: OllamaConfig) -> Result<Self> {
           let url = config.url.clone()
               .unwrap_or_else(|| "http://localhost".to_string());
           let port = config.port.unwrap_or(11434);

           info!("Initializing Ollama provider: {}:{}", url, port);
           debug!("Ollama config: timeout={}s, auto_start={}",
               config.timeout_seconds.unwrap_or(120),
               config.auto_start
           );

           let client = ollama_rs::Ollama::new(url, port);
           Ok(Self { client, config })
       }

       pub async fn check_health(&self) -> Result<HealthStatus> {
           trace!("Checking Ollama server health");
           // Implementation
       }
   }

   #[async_trait::async_trait]
   impl ProviderInstance for OllamaProvider {
       async fn complete(&self, input: AgentInput) -> Result<AgentOutput> {
           info!("Ollama completion request: prompt_len={}", input.prompt.len());
           // Implementation with full tracing
       }

       fn capabilities(&self) -> &ProviderCapabilities {
           // Return Ollama capabilities
       }
   }
   ```
2. Implement health check logic
3. Implement model listing via Ollama API
4. Implement completion with streaming
5. Add comprehensive tracing throughout
6. Write unit tests in `tests/ollama_provider_test.rs`
7. Run clippy and fix all warnings

**Definition of Done:**
- [ ] All ProviderInstance methods implemented
- [ ] Unit tests pass with >90% coverage
- [ ] Integration test with mock Ollama server passes
- [ ] Zero clippy warnings
- [ ] Tracing comprehensive (all levels used appropriately)

---

### Task 11.1.3: Implement Model Pull for Ollama
**Priority**: HIGH
**Estimated Time**: 4 hours
**Assignee**: Provider Team
**Dependencies**: Task 11.1.2

**Description**: Implement model download/pull functionality for Ollama with progress tracking.

**Acceptance Criteria:**
- [ ] `pull_model()` method downloads models via Ollama API
- [ ] Progress tracking with callbacks
- [ ] Error handling for network failures
- [ ] Model verification after download
- [ ] Comprehensive tracing
- [ ] Unit tests pass
- [ ] Zero clippy warnings

**Implementation Steps:**
1. Add pull_model method to OllamaProvider:
   ```rust
   pub async fn pull_model(
       &self,
       model_spec: &str,
       progress_callback: Option<Box<dyn Fn(u64, u64) + Send>>,
   ) -> Result<()> {
       info!("Pulling Ollama model: {}", model_spec);
       trace!("Initiating model download via Ollama API");

       // Implementation with progress tracking
       debug!("Model pull in progress: {} bytes downloaded", bytes_downloaded);

       info!("Model pull complete: {}", model_spec);
       Ok(())
   }
   ```
2. Implement progress tracking
3. Handle partial downloads and resume
4. Verify model after download
5. Write tests with mock progress
6. Add feature flag: `#[cfg(feature = "ollama")]`

**Definition of Done:**
- [ ] Model pull works end-to-end
- [ ] Progress tracking functional
- [ ] Error recovery tested
- [ ] Tests pass with >90% coverage
- [ ] Zero clippy warnings

---

### Task 11.1.4: Implement Ollama Auto-Start
**Priority**: MEDIUM
**Estimated Time**: 3 hours
**Assignee**: Provider Team
**Dependencies**: Task 11.1.2

**Description**: Add auto-start functionality to launch Ollama server if not running.

**Acceptance Criteria:**
- [ ] Detects if Ollama server is running
- [ ] Launches Ollama binary if not found
- [ ] Waits for server readiness with timeout
- [ ] Platform-specific binary paths (macOS, Linux, Windows)
- [ ] Comprehensive error handling
- [ ] Tests with mock server
- [ ] Zero clippy warnings

**Implementation Steps:**
1. Add server detection logic
2. Implement platform-specific binary path resolution
3. Launch server process with proper environment
4. Wait for health check with exponential backoff
5. Handle launch failures gracefully
6. Write unit tests with mock processes
7. Feature flag: `#[cfg(feature = "ollama-auto-start")]`

**Definition of Done:**
- [ ] Auto-start works on supported platforms
- [ ] Timeout handling correct
- [ ] Error messages actionable
- [ ] Tests comprehensive
- [ ] Zero clippy warnings

---

### Task 11.1.5: Integrate Ollama with ProviderManager
**Priority**: CRITICAL
**Estimated Time**: 4 hours
**Assignee**: Integration Team
**Dependencies**: Task 11.1.2

**Description**: Integrate OllamaProvider with existing ProviderManager and model specification system.

**Acceptance Criteria:**
- [ ] ModelSpecifier parses `local/llama3.1:8b` syntax
- [ ] ModelSpecifier parses `local/phi3:3.8b@ollama` syntax
- [ ] ProviderManager creates Ollama agents correctly
- [ ] Configuration loaded from llmspell.toml
- [ ] Integration tests pass
- [ ] Zero clippy warnings

**Implementation Steps:**
1. Update `llmspell-providers/src/model_specifier.rs` to handle "local" provider:
   ```rust
   pub fn parse(spec: &str) -> Result<Self> {
       // Handle "local/model:variant[@backend]" syntax
       if spec.starts_with("local/") {
           // Parse local model spec
           let (model_part, backend) = if let Some(idx) = spec.find('@') {
               (&spec[..idx], Some(&spec[idx+1..]))
           } else {
               (spec, None)
           };
           // Continue parsing...
       }
   }
   ```
2. Update ProviderManager to route "local" provider to LocalProviderFactory
3. Create LocalProviderFactory that dispatches to Ollama or Candle
4. Add configuration schema to LLMSpellConfig
5. Write integration tests
6. Run clippy and fix warnings

**Definition of Done:**
- [ ] ModelSpecifier handles all local syntaxes
- [ ] ProviderManager integration works
- [ ] Configuration schema documented
- [ ] Integration tests pass
- [ ] Zero clippy warnings

---

## Phase 11.2: Candle Provider Implementation (Days 6-10)

### Task 11.2.1: Add Candle Dependencies and Types
**Priority**: CRITICAL
**Estimated Time**: 3 hours
**Assignee**: Provider Team Lead
**Dependencies**: Task 11.1.5 complete

**Description**: Add candle-core and candle-transformers dependencies and create Candle-specific types.

**Acceptance Criteria:**
- [ ] `candle-core = "0.7"` added to Cargo.toml
- [ ] `candle-transformers = "0.7"` added
- [ ] `hf-hub = "0.3"` added for model downloads
- [ ] CandleConfig struct created
- [ ] Device detection (CPU/CUDA/Metal) implemented
- [ ] Zero clippy warnings

**Implementation Steps:**
1. Add dependencies to llmspell-providers/Cargo.toml:
   ```toml
   [dependencies]
   candle-core = "0.7"
   candle-transformers = "0.7"
   hf-hub = "0.3"
   tokenizers = "0.20"
   ```
2. Create `llmspell-providers/src/candle/mod.rs`
3. Create `llmspell-providers/src/candle/types.rs`:
   ```rust
   #[derive(Debug, Clone)]
   pub struct CandleConfig {
       pub model_directory: PathBuf,
       pub device: String, // "auto", "cpu", "cuda", "metal"
       pub default_quantization: String, // "Q4_K_M", "Q5_K_M", etc.
       pub max_concurrent: usize,
       pub cpu_threads: Option<usize>,
   }
   ```
4. Implement device detection logic
5. Run cargo check and clippy

**Definition of Done:**
- [ ] Dependencies compile
- [ ] Types well-documented
- [ ] Device detection works
- [ ] Zero clippy warnings

---

### Task 11.2.2: Implement GGUF Model Loading
**Priority**: CRITICAL
**Estimated Time**: 8 hours
**Assignee**: Candle Team Lead
**Dependencies**: Task 11.2.1

**Description**: Implement GGUF model loading from HuggingFace with proper error handling.

**Acceptance Criteria:**
- [ ] Loads GGUF files from local directory
- [ ] Downloads from HuggingFace if not present
- [ ] Validates model format and quantization
- [ ] Loads tokenizer correctly
- [ ] Comprehensive tracing
- [ ] Unit tests with fixture models
- [ ] Zero clippy warnings

**Implementation Steps:**
1. Create `llmspell-providers/src/candle/gguf_loader.rs`:
   ```rust
   use candle_core::{Device, Tensor};
   use tracing::{info, debug, trace, error};

   pub struct GGUFLoader {
       model_dir: PathBuf,
       device: Device,
   }

   impl GGUFLoader {
       pub async fn load_model(&self, model_id: &str) -> Result<LoadedModel> {
           info!("Loading GGUF model: {}", model_id);
           let start = std::time::Instant::now();

           trace!("Loading tokenizer from file");
           let tokenizer = self.load_tokenizer(model_id)
               .map_err(|e| {
                   error!("Failed to load tokenizer: {}", e);
                   anyhow!("Tokenizer load error: {}", e)
               })?;
           debug!("Tokenizer loaded: vocab_size={}", tokenizer.get_vocab_size(true));

           trace!("Loading GGUF model file");
           let model = self.load_gguf_file(model_id)?;
           debug!("Model loaded: layers={}, params={}", model.layers, model.params);

           info!("Model loading complete in {:?}", start.elapsed());
           Ok(LoadedModel { model, tokenizer })
       }
   }
   ```
2. Implement GGUF file parsing
3. Implement HuggingFace download with hf-hub
4. Implement tokenizer loading
5. Add caching for loaded models
6. Write tests with small test models
7. Feature flag: `#[cfg(feature = "candle")]`

**Definition of Done:**
- [ ] GGUF loading works for supported models
- [ ] HuggingFace integration functional
- [ ] Tests pass with fixture models
- [ ] Comprehensive tracing
- [ ] Zero clippy warnings

---

### Task 11.2.3: Implement CandleProvider with Inference
**Priority**: CRITICAL
**Estimated Time**: 10 hours
**Assignee**: Candle Team
**Dependencies**: Task 11.2.2

**Description**: Implement CandleProvider with text generation inference loop.

**Acceptance Criteria:**
- [ ] CandleProvider implements ProviderInstance trait
- [ ] Text generation with sampling strategies
- [ ] Streaming output support
- [ ] Temperature and top_p parameters
- [ ] Model unloading and memory management
- [ ] >20 tokens/sec for 7B models
- [ ] Tests with small models pass
- [ ] Zero clippy warnings

**Implementation Steps:**
1. Create `llmspell-providers/src/candle/provider.rs`:
   ```rust
   pub struct CandleProvider {
       loader: GGUFLoader,
       loaded_models: Arc<RwLock<HashMap<String, LoadedModel>>>,
       config: CandleConfig,
   }

   #[async_trait::async_trait]
   impl ProviderInstance for CandleProvider {
       async fn complete(&self, input: AgentInput) -> Result<AgentOutput> {
           info!("Candle completion request: prompt_len={}", input.prompt.len());
           debug!("Generation params: temp={}, top_p={}",
               input.temperature, input.top_p);

           trace!("Tokenizing input");
           let tokens = self.tokenize(&input.prompt)?;
           debug!("Input tokenized: {} tokens", tokens.len());

           trace!("Starting generation loop");
           let mut generated_tokens = Vec::new();
           for step in 0..input.max_tokens.unwrap_or(512) {
               trace!("Generation step {}", step);
               let logits = self.forward(&tokens)?;
               let next_token = self.sample(logits, input.temperature)?;
               generated_tokens.push(next_token);

               if next_token == self.eos_token {
                   debug!("EOS token generated at step {}", step);
                   break;
               }
           }

           info!("Generation complete: {} tokens", generated_tokens.len());
           Ok(self.detokenize(generated_tokens)?)
       }
   }
   ```
2. Implement text generation loop
3. Implement sampling strategies (temperature, top_p, top_k)
4. Implement streaming support
5. Add generation benchmarks
6. Write inference tests
7. Optimize performance (<200ms first token)

**Definition of Done:**
- [ ] Inference generates coherent text
- [ ] Performance meets targets
- [ ] Streaming works correctly
- [ ] Tests comprehensive
- [ ] Zero clippy warnings

---

### Task 11.2.4: Implement Candle Model Download
**Priority**: HIGH
**Estimated Time**: 5 hours
**Assignee**: Candle Team
**Dependencies**: Task 11.2.2

**Description**: Implement model download from HuggingFace Hub with progress tracking.

**Acceptance Criteria:**
- [ ] Downloads GGUF models from HuggingFace
- [ ] Progress tracking with callbacks
- [ ] Partial download resume support
- [ ] Model verification after download
- [ ] Comprehensive error handling
- [ ] Tests with mock downloads
- [ ] Zero clippy warnings

**Implementation Steps:**
1. Create `llmspell-providers/src/candle/downloader.rs`:
   ```rust
   pub async fn download_model(
       repo_id: &str,
       filename: &str,
       progress_callback: Option<Box<dyn Fn(u64, u64) + Send>>,
   ) -> Result<PathBuf> {
       info!("Downloading model from HuggingFace: {}/{}", repo_id, filename);

       let api = hf_hub::api::sync::Api::new()?;
       let repo = api.model(repo_id.to_string());

       debug!("Fetching model file: {}", filename);
       let path = repo.get(filename)?;

       info!("Model downloaded successfully to: {}", path.display());
       Ok(path)
   }
   ```
2. Implement progress tracking via callbacks
3. Handle network errors with retry
4. Verify model integrity
5. Write tests with mock HF Hub
6. Feature flag integration

**Definition of Done:**
- [ ] Downloads work reliably
- [ ] Progress tracking accurate
- [ ] Error handling comprehensive
- [ ] Tests pass
- [ ] Zero clippy warnings

---

### Task 11.2.5: Integrate Candle with ProviderManager
**Priority**: CRITICAL
**Estimated Time**: 4 hours
**Assignee**: Integration Team
**Dependencies**: Task 11.2.3

**Description**: Integrate CandleProvider with ProviderManager for backend selection.

**Acceptance Criteria:**
- [ ] ModelSpecifier parses `local/mistral:7b@candle` syntax
- [ ] LocalProviderFactory dispatches to Candle when requested
- [ ] Auto-detection prefers Ollama, falls back to Candle
- [ ] Configuration from llmspell.toml works
- [ ] Integration tests pass
- [ ] Zero clippy warnings

**Implementation Steps:**
1. Update LocalProviderFactory in `llmspell-providers/src/local_factory.rs`:
   ```rust
   pub async fn create_provider(
       spec: &ModelSpecifier,
       config: &LLMSpellConfig,
   ) -> Result<Box<dyn ProviderInstance>> {
       match spec.backend.as_deref() {
           Some("ollama") => {
               info!("Creating Ollama provider (explicit)");
               create_ollama_provider(spec, config).await
           }
           Some("candle") => {
               info!("Creating Candle provider (explicit)");
               create_candle_provider(spec, config).await
           }
           None => {
               info!("Auto-detecting local provider backend");
               // Try Ollama first, fallback to Candle
               if ollama_available().await {
                   debug!("Ollama detected, using Ollama provider");
                   create_ollama_provider(spec, config).await
               } else {
                   debug!("Ollama not available, using Candle provider");
                   create_candle_provider(spec, config).await
               }
           }
           Some(other) => {
               error!("Unknown backend: {}", other);
               Err(anyhow!("Unknown backend: {}", other))
           }
       }
   }
   ```
2. Implement backend auto-detection
3. Add configuration schema
4. Write integration tests
5. Run clippy

**Definition of Done:**
- [ ] Backend selection works
- [ ] Auto-detection functional
- [ ] Integration tests pass
- [ ] Zero clippy warnings

---

## Phase 11.3: CLI Implementation (Days 11-13)

### Task 11.3.1: Create Model Command Structure
**Priority**: CRITICAL
**Estimated Time**: 3 hours
**Assignee**: CLI Team Lead
**Dependencies**: Task 11.2.5 complete

**Description**: Add `llmspell model` subcommand structure to CLI.

**Acceptance Criteria:**
- [ ] ModelCommands enum with all subcommands
- [ ] Clap integration with proper help text
- [ ] Subcommand routing in main command handler
- [ ] Compiles without warnings
- [ ] Zero clippy warnings

**Implementation Steps:**
1. Update `llmspell-cli/src/cli.rs` to add ModelCommands:
   ```rust
   #[derive(Subcommand)]
   pub enum Commands {
       // ... existing commands

       /// Manage local LLM models (Ollama and Candle)
       Model {
           #[command(subcommand)]
           command: ModelCommands,
       },
   }

   #[derive(Subcommand)]
   pub enum ModelCommands {
       /// Check status of local LLM backends
       Status,

       /// List installed local models
       List {
           /// Filter by backend (ollama or candle)
           #[arg(long)]
           backend: Option<String>,

           /// Show verbose output
           #[arg(short, long)]
           verbose: bool,
       },

       /// Download a model
       Pull {
           /// Model specification (e.g., "ollama/llama3.1:8b" or "candle/mistral:7b")
           model: String,

           /// Force re-download even if model exists
           #[arg(short, long)]
           force: bool,

           /// Quantization level for Candle models
           #[arg(long, default_value = "Q4_K_M")]
           quantization: String,
       },

       /// Remove a model
       Remove {
           /// Model name to remove
           model: String,

           /// Skip confirmation prompt
           #[arg(short = 'y', long)]
           yes: bool,
       },

       /// Show model information
       Info {
           /// Model name
           model: String,
       },

       /// List available models
       Available {
           /// Filter by backend
           #[arg(long)]
           backend: Option<String>,

           /// Show only recommended models
           #[arg(long)]
           recommended: bool,
       },

       /// Install Ollama binary (macOS and Linux only)
       InstallOllama,
   }
   ```
2. Add command routing in commands/mod.rs
3. Write help text documentation
4. Test CLI parsing with clap

**Definition of Done:**
- [ ] All subcommands parse correctly
- [ ] Help text comprehensive
- [ ] Compiles without warnings
- [ ] Zero clippy warnings

---

### Task 11.3.2: Implement Model Status Command
**Priority**: HIGH
**Estimated Time**: 4 hours
**Assignee**: CLI Team
**Dependencies**: Task 11.3.1

**Description**: Implement `llmspell model status` to check backend availability.

**Acceptance Criteria:**
- [ ] Checks Ollama server status
- [ ] Checks Candle availability
- [ ] Shows installed model counts
- [ ] Pretty formatted output
- [ ] JSON output option
- [ ] Zero clippy warnings

**Implementation Steps:**
1. Create `llmspell-cli/src/commands/model.rs`:
   ```rust
   pub async fn handle_model_command(
       command: ModelCommands,
       runtime_config: LLMSpellConfig,
       output_format: OutputFormat,
   ) -> Result<()> {
       match command {
           ModelCommands::Status => {
               info!("Checking local LLM backend status");

               // Check Ollama
               let ollama_status = check_ollama_status().await?;
               debug!("Ollama status: available={}, models={}",
                   ollama_status.available, ollama_status.model_count);

               // Check Candle
               let candle_status = check_candle_status().await?;
               debug!("Candle status: available={}, models={}",
                   candle_status.available, candle_status.model_count);

               // Format output
               let formatter = OutputFormatter::new(output_format);
               formatter.print_backend_status(&ollama_status, &candle_status)?;

               info!("Status check complete");
               Ok(())
           }
           // ... other commands
       }
   }
   ```
2. Implement Ollama health check
3. Implement Candle availability check
4. Create formatted output
5. Add JSON output support
6. Write tests

**Definition of Done:**
- [ ] Status check works
- [ ] Output formatted properly
- [ ] Tests pass
- [ ] Zero clippy warnings

---

### Task 11.3.3: Implement Model List Command
**Priority**: HIGH
**Estimated Time**: 4 hours
**Assignee**: CLI Team
**Dependencies**: Task 11.3.1

**Description**: Implement `llmspell model list` to show installed models.

**Acceptance Criteria:**
- [ ] Lists Ollama models
- [ ] Lists Candle models
- [ ] Shows model sizes and variants
- [ ] Backend filtering works
- [ ] Verbose mode shows details
- [ ] Zero clippy warnings

**Implementation Steps:**
1. Implement list command in model.rs
2. Query Ollama API for model list
3. Scan Candle model directory
4. Format output as table
5. Add verbose mode with metadata
6. Write tests with mock data

**Definition of Done:**
- [ ] List command functional
- [ ] Filtering works
- [ ] Output well-formatted
- [ ] Tests pass
- [ ] Zero clippy warnings

---

### Task 11.3.4: Implement Model Pull Command
**Priority**: CRITICAL
**Estimated Time**: 6 hours
**Assignee**: CLI Team
**Dependencies**: Task 11.3.1

**Description**: Implement `llmspell model pull` to download models.

**Acceptance Criteria:**
- [ ] Downloads Ollama models via `ollama pull`
- [ ] Downloads Candle models from HuggingFace
- [ ] Progress bar shows download status
- [ ] Force re-download works
- [ ] Error handling comprehensive
- [ ] Zero clippy warnings

**Implementation Steps:**
1. Implement pull command
2. Add progress bar with indicatif crate
3. Route to appropriate backend
4. Handle download errors
5. Verify model after download
6. Write tests with mock downloads

**Definition of Done:**
- [ ] Pull command works for both backends
- [ ] Progress bar functional
- [ ] Error handling robust
- [ ] Tests pass
- [ ] Zero clippy warnings

---

### Task 11.3.5: Implement Remaining Model Commands
**Priority**: MEDIUM
**Estimated Time**: 5 hours
**Assignee**: CLI Team
**Dependencies**: Task 11.3.4

**Description**: Implement remaining commands: remove, info, available, install-ollama.

**Acceptance Criteria:**
- [ ] Remove command deletes models
- [ ] Info command shows model details
- [ ] Available command lists recommended models
- [ ] Install-ollama downloads and installs Ollama
- [ ] All commands have tests
- [ ] Zero clippy warnings

**Implementation Steps:**
1. Implement remove command with confirmation
2. Implement info command with metadata
3. Implement available with recommended list
4. Implement install-ollama for macOS/Linux
5. Write comprehensive tests
6. Run clippy

**Definition of Done:**
- [ ] All commands functional
- [ ] Tests comprehensive
- [ ] Error handling robust
- [ ] Zero clippy warnings

---

## Phase 11.4: Bridge Layer Integration (Days 14-16)

### Task 11.4.1: Create LocalLLM Global Object
**Priority**: CRITICAL
**Estimated Time**: 4 hours
**Assignee**: Bridge Team Lead
**Dependencies**: Task 11.3.5 complete

**Description**: Create LocalLLM global object for script access to local models.

**Acceptance Criteria:**
- [ ] LocalLLMGlobal struct created
- [ ] GlobalObject trait implemented
- [ ] Registered in global registry
- [ ] Compiles without warnings
- [ ] Zero clippy warnings

**Implementation Steps:**
1. Create `llmspell-bridge/src/lua/globals/local_llm.rs`:
   ```rust
   use crate::globals::GlobalContext;
   use mlua::{Lua, Table};
   use tracing::{info, debug};

   pub fn inject_local_llm_global(
       lua: &Lua,
       _context: &GlobalContext,
   ) -> mlua::Result<()> {
       info!("Injecting LocalLLM global object");

       let local_llm_table = lua.create_table()?;

       // Register methods
       register_status_method(lua, &local_llm_table)?;
       register_list_method(lua, &local_llm_table)?;
       register_pull_method(lua, &local_llm_table)?;

       lua.globals().set("LocalLLM", local_llm_table)?;
       debug!("LocalLLM global registered successfully");
       Ok(())
   }
   ```
2. Register in globals/mod.rs
3. Write basic injection tests
4. Run clippy

**Definition of Done:**
- [ ] Global object injects correctly
- [ ] Registration works
- [ ] Tests pass
- [ ] Zero clippy warnings

---

### Task 11.4.2: Implement LocalLLM.status() Method
**Priority**: HIGH
**Estimated Time**: 3 hours
**Assignee**: Bridge Team
**Dependencies**: Task 11.4.1

**Description**: Implement LocalLLM.status() to check backend status from scripts.

**Acceptance Criteria:**
- [ ] Returns table with backend status
- [ ] Async execution via block_on_async
- [ ] Error handling proper
- [ ] Comprehensive tracing
- [ ] Tests pass
- [ ] Zero clippy warnings

**Implementation Steps:**
1. Implement status method:
   ```rust
   fn register_status_method(lua: &Lua, table: &Table) -> mlua::Result<()> {
       let status_fn = lua.create_function(|lua, ()| {
           info!("LocalLLM.status() called from script");

           let status = block_on_async("local_llm_status", async {
               debug!("Checking Ollama status");
               let ollama = check_ollama_status().await?;

               debug!("Checking Candle status");
               let candle = check_candle_status().await?;

               Ok((ollama, candle))
           })?;

           let result = lua.create_table()?;
           result.set("ollama_available", status.0.available)?;
           result.set("ollama_models", status.0.model_count)?;
           result.set("candle_available", status.1.available)?;
           result.set("candle_models", status.1.model_count)?;

           debug!("Status check complete: ollama={}, candle={}",
               status.0.available, status.1.available);
           Ok(result)
       })?;

       table.set("status", status_fn)?;
       Ok(())
   }
   ```
2. Implement backend status checks
3. Write Lua integration test
4. Run clippy

**Definition of Done:**
- [ ] Method works from Lua
- [ ] Status accurate
- [ ] Tests pass
- [ ] Zero clippy warnings

---

### Task 11.4.3: Implement LocalLLM.list() Method
**Priority**: HIGH
**Estimated Time**: 3 hours
**Assignee**: Bridge Team
**Dependencies**: Task 11.4.1

**Description**: Implement LocalLLM.list() to list models from scripts.

**Acceptance Criteria:**
- [ ] Returns array of model tables
- [ ] Optional backend filter
- [ ] Model metadata included
- [ ] Tests pass
- [ ] Zero clippy warnings

**Implementation Steps:**
1. Implement list method with optional backend parameter
2. Query both backends
3. Format as Lua tables
4. Write integration test
5. Run clippy

**Definition of Done:**
- [ ] List method functional
- [ ] Filtering works
- [ ] Tests pass
- [ ] Zero clippy warnings

---

### Task 11.4.4: Implement LocalLLM.pull() Method
**Priority**: HIGH
**Estimated Time**: 4 hours
**Assignee**: Bridge Team
**Dependencies**: Task 11.4.1

**Description**: Implement LocalLLM.pull() to download models from scripts.

**Acceptance Criteria:**
- [ ] Downloads models via backend
- [ ] Progress callback support
- [ ] Error handling comprehensive
- [ ] Tests with mock downloads
- [ ] Zero clippy warnings

**Implementation Steps:**
1. Implement pull method with progress callbacks
2. Route to appropriate backend
3. Handle async progress updates
4. Write integration tests
5. Run clippy

**Definition of Done:**
- [ ] Pull method works
- [ ] Progress tracking functional
- [ ] Tests pass
- [ ] Zero clippy warnings

---

### Task 11.4.5: Update Agent.create() for Local Models
**Priority**: CRITICAL
**Estimated Time**: 4 hours
**Assignee**: Bridge Team
**Dependencies**: Task 11.4.4

**Description**: Update Agent.create() to support local model syntax.

**Acceptance Criteria:**
- [ ] `Agent.create({model = "local/llama3.1:8b"})` works
- [ ] Backend auto-detection functional
- [ ] Explicit backend selection works
- [ ] Backward compatibility maintained
- [ ] Tests comprehensive
- [ ] Zero clippy warnings

**Implementation Steps:**
1. Update agent creation in `llmspell-bridge/src/lua/globals/agent.rs`
2. Parse local model specifications
3. Route to LocalProviderFactory
4. Handle configuration merging
5. Write integration tests
6. Run clippy

**Definition of Done:**
- [ ] Local model syntax works
- [ ] All modes tested
- [ ] Backward compatible
- [ ] Zero clippy warnings

---

## Phase 11.5: Example Applications (Days 17-18)

### Task 11.5.1: Create local_chat.lua Example
**Priority**: HIGH
**Estimated Time**: 3 hours
**Assignee**: Examples Team
**Dependencies**: Task 11.4.5

**Description**: Create basic local chat example with auto-detection.

**Acceptance Criteria:**
- [ ] Example uses `local/llama3.1:8b` syntax
- [ ] Auto-detects backend
- [ ] Interactive chat loop
- [ ] Well-commented code
- [ ] Includes llmspell.toml
- [ ] Example runs successfully

**Implementation Steps:**
1. Create `examples/script-users/applications/local-chat/local_chat.lua`:
   ```lua
   -- Simple local LLM chat with auto-detection
   -- This example demonstrates using local models with automatic
   -- backend selection (prefers Ollama, falls back to Candle)

   local Agent = require("agent")
   local LocalLLM = require("local_llm")

   -- Check backend status
   print("Checking local LLM backends...")
   local status = LocalLLM.status()
   print(string.format("Ollama: %s (%d models)",
       status.ollama_available and "Available" or "Not available",
       status.ollama_models))
   print(string.format("Candle: %s (%d models)",
       status.candle_available and "Available" or "Not available",
       status.candle_models))

   -- Create agent with local model (auto-detects backend)
   local agent = Agent.create({
       model = "local/llama3.1:8b",
       temperature = 0.7,
       max_tokens = 512
   })

   -- Interactive chat loop
   while true do
       io.write("\nYou: ")
       local user_input = io.read()
       if user_input == "exit" then break end

       local response = agent:complete(user_input)
       print("Assistant: " .. response)
   end
   ```
2. Create accompanying llmspell.toml
3. Test example end-to-end
4. Add README.md with instructions

**Definition of Done:**
- [ ] Example runs successfully
- [ ] Code well-documented
- [ ] README helpful
- [ ] llmspell.toml complete

---

### Task 11.5.2: Create Backend-Specific Examples
**Priority**: MEDIUM
**Estimated Time**: 4 hours
**Assignee**: Examples Team
**Dependencies**: Task 11.5.1

**Description**: Create ollama_chat.lua and candle_inference.lua examples.

**Acceptance Criteria:**
- [ ] ollama_chat.lua forces Ollama backend
- [ ] candle_inference.lua forces Candle backend
- [ ] Both include performance benchmarks
- [ ] Error handling demonstrated
- [ ] Examples run successfully

**Implementation Steps:**
1. Create ollama_chat.lua with `@ollama` syntax
2. Create candle_inference.lua with `@candle` syntax
3. Add benchmarking code
4. Test both examples
5. Update README

**Definition of Done:**
- [ ] Both examples work
- [ ] Benchmarking accurate
- [ ] Documentation complete

---

### Task 11.5.3: Create backend_comparison.lua Example
**Priority**: MEDIUM
**Estimated Time**: 3 hours
**Assignee**: Examples Team
**Dependencies**: Task 11.5.2

**Description**: Create example comparing Ollama vs Candle performance.

**Acceptance Criteria:**
- [ ] Runs same prompt on both backends
- [ ] Measures first token latency
- [ ] Measures throughput
- [ ] Formats comparison table
- [ ] Example runs successfully

**Implementation Steps:**
1. Create backend_comparison.lua
2. Implement benchmark functions
3. Add result formatting
4. Test with both backends
5. Document findings

**Definition of Done:**
- [ ] Comparison works
- [ ] Benchmarks accurate
- [ ] Output informative

---

## Phase 11.6: Testing and Validation (Day 19)

### Task 11.6.1: Unit Test Suite
**Priority**: CRITICAL
**Estimated Time**: 6 hours
**Assignee**: QA Team

**Description**: Comprehensive unit tests for all components.

**Acceptance Criteria:**
- [ ] >90% code coverage for new code
- [ ] All edge cases covered
- [ ] Feature-based test organization
- [ ] Mocks for external services
- [ ] All tests pass in CI
- [ ] Zero clippy warnings

**Implementation Steps:**
1. Ollama provider tests
2. Candle provider tests
3. ModelSpecifier tests
4. CLI command tests
5. Bridge integration tests
6. Feature flags: `#[cfg(test)]` and `#[cfg(feature = "ollama")]`

**Definition of Done:**
- [ ] Coverage >90%
- [ ] All tests pass
- [ ] CI integrated
- [ ] Zero clippy warnings

---

### Task 11.6.2: Performance Benchmarks
**Priority**: HIGH
**Estimated Time**: 4 hours
**Assignee**: Performance Team

**Description**: Benchmark all performance-critical operations.

**Acceptance Criteria:**
- [ ] Ollama: <100ms first token latency
- [ ] Candle: <200ms first token latency
- [ ] Both: >20 tokens/sec for 7B models
- [ ] Memory <5GB for Q4_K_M models
- [ ] Benchmarks documented

**Implementation Steps:**
1. Create `benches/ollama_bench.rs`
2. Create `benches/candle_bench.rs`
3. Run benchmarks on target hardware
4. Document results
5. Set up regression detection

**Definition of Done:**
- [ ] All targets met
- [ ] Benchmarks reproducible
- [ ] Results documented

---

### Task 11.6.3: Integration Tests
**Priority**: HIGH
**Estimated Time**: 4 hours
**Assignee**: QA Team

**Description**: End-to-end integration tests.

**Acceptance Criteria:**
- [ ] Full workflow tested (pull → list → create agent → complete)
- [ ] Backend switching tested
- [ ] Error scenarios covered
- [ ] Example applications validated
- [ ] All tests pass

**Implementation Steps:**
1. Write end-to-end test scenarios
2. Test with both backends
3. Test error conditions
4. Validate all examples
5. Document test scenarios

**Definition of Done:**
- [ ] All scenarios tested
- [ ] Tests reliable
- [ ] Documentation complete

---

## Phase 11.7: Documentation and Handoff (Day 20)

### Task 11.7.1: API Documentation
**Priority**: HIGH
**Estimated Time**: 4 hours
**Assignee**: Documentation Team

**Description**: Complete API documentation for all public interfaces.

**Acceptance Criteria:**
- [ ] All public items documented
- [ ] Examples in doc comments
- [ ] Migration guide created
- [ ] Architecture diagram included
- [ ] README updated

**Implementation Steps:**
1. Document all provider APIs
2. Document CLI commands
3. Document script APIs
4. Create migration guide
5. Generate docs with `cargo doc`

**Definition of Done:**
- [ ] Docs coverage >95%
- [ ] Examples compile
- [ ] Migration guide helpful

---

### Task 11.7.2: User Guide and Examples
**Priority**: HIGH
**Estimated Time**: 4 hours
**Assignee**: Documentation Team

**Description**: Create comprehensive user guide.

**Acceptance Criteria:**
- [ ] Getting started guide
- [ ] Model installation guide
- [ ] Configuration guide
- [ ] Troubleshooting section
- [ ] Example walkthroughs

**Implementation Steps:**
1. Write getting started guide
2. Document model installation
3. Explain configuration options
4. Create troubleshooting FAQ
5. Walk through examples

**Definition of Done:**
- [ ] Guide comprehensive
- [ ] All examples documented
- [ ] Troubleshooting helpful

---

### Task 11.7.3: Phase 12 Handoff Package
**Priority**: CRITICAL
**Estimated Time**: 3 hours
**Assignee**: Team Lead

**Description**: Create Phase 11 handoff package for Phase 12 team.

**Acceptance Criteria:**
- [ ] Architecture documented
- [ ] API reference complete
- [ ] Integration points identified
- [ ] Known issues listed
- [ ] Performance baselines documented

**Implementation Steps:**
1. Create `PHASE11_HANDOFF_PACKAGE.md`
2. Document architecture decisions
3. List integration points for Phase 12
4. Document known limitations
5. Include performance data

**Definition of Done:**
- [ ] Package complete
- [ ] Reviewed by team
- [ ] Ready for Phase 12

---

## Final Validation Checklist

### Quality Gates
- [ ] All crates compile without warnings
- [ ] Clippy passes: `cargo clippy --workspace --all-features --all-targets`
- [ ] Format compliance: `cargo fmt --all --check`
- [ ] Tests pass: `cargo test --workspace --all-features`
- [ ] Documentation builds: `cargo doc --workspace --all-features --no-deps`
- [ ] Examples run successfully
- [ ] Benchmarks meet targets

### Performance Validation
- [ ] Ollama: <100ms first token latency
- [ ] Candle: <200ms first token latency
- [ ] Both: >20 tokens/sec for 7B models
- [ ] Memory: <5GB RAM for Q4_K_M models
- [ ] Model loading: <5 seconds
- [ ] Cold start: <10 seconds total

### Feature Validation
- [ ] `llmspell model status` works
- [ ] `llmspell model list` works
- [ ] `llmspell model pull ollama/llama3.1:8b` works
- [ ] `llmspell model pull candle/mistral:7b` works
- [ ] LocalLLM.status() works from Lua
- [ ] LocalLLM.list() works from Lua
- [ ] LocalLLM.pull() works from Lua
- [ ] Agent.create({model = "local/llama3.1:8b"}) works
- [ ] Backend auto-detection works
- [ ] Explicit backend selection works

### Documentation Validation
- [ ] API docs coverage >95%
- [ ] User guide complete
- [ ] Examples documented
- [ ] Migration guide ready
- [ ] README updated

### Phase 12 Readiness
- [ ] Local LLM integration complete
- [ ] Performance baselines captured
- [ ] Integration points documented
- [ ] Handoff package delivered

---

## Risk Mitigation

### Technical Risks
1. **Ollama not installed**: Mitigated by auto-detection and install-ollama command
2. **Candle compilation issues**: Feature flags allow building without Candle
3. **Model download failures**: Retry logic and resume support
4. **Performance not meeting targets**: Optimization in Phase 12 if needed

### Schedule Risks
1. **Candle complexity**: Focus on Ollama first, Candle in parallel
2. **Testing infrastructure**: Reuse existing llmspell-testing patterns
3. **Documentation scope**: Start docs early, iterate

---

## Notes and Decisions Log

### Architectural Decisions
- **Decision**: Extend llmspell-providers vs new crate
  - **Rationale**: Consistent provider abstraction, no circular dependencies
  - **Impact**: Cleaner integration with existing code

- **Decision**: Auto-detection prefers Ollama
  - **Rationale**: Ollama is more mature and easier to use
  - **Impact**: Better default experience

- **Decision**: Feature flags for Ollama and Candle
  - **Rationale**: Optional dependencies, faster builds
  - **Impact**: Users can choose which backend to compile

### Implementation Notes
- Using ollama-rs v0.3.2 for Ollama client
- Using candle-core v0.7 for Candle inference
- GGUF Q4_K_M quantization as default
- Progress tracking via callbacks for both backends

### Dependencies Added
- `ollama-rs = "0.3.2"` - Ollama client
- `candle-core = "0.7"` - Candle inference
- `candle-transformers = "0.7"` - Transformers support
- `hf-hub = "0.3"` - HuggingFace downloads
- `tokenizers = "0.20"` - Tokenization

---

## Team Assignments

**Provider Team Lead**: Overall provider architecture, Ollama and Candle coordination
**Candle Team**: GGUF loading, inference, model management
**CLI Team**: Command implementation, user experience
**Bridge Team**: Lua integration, script APIs
**Integration Team**: ProviderManager integration, backend selection
**QA Team**: Testing, benchmarks, validation
**Documentation Team**: API docs, user guides, examples
**Performance Team**: Benchmarking, optimization

---

## Daily Standup Topics

**Day 1**: Ollama dependencies and types
**Day 2**: OllamaProvider implementation
**Day 3**: Ollama model pull and auto-start
**Day 4**: Ollama integration complete
**Day 5**: Ollama testing and validation
**Day 6**: Candle dependencies and GGUF loading
**Day 7**: CandleProvider implementation
**Day 8**: Candle inference and model download
**Day 9**: Candle integration complete
**Day 10**: Candle testing and validation
**Day 11**: CLI command structure
**Day 12**: CLI model commands
**Day 13**: CLI testing
**Day 14**: LocalLLM global object
**Day 15**: LocalLLM methods
**Day 16**: Agent.create() integration
**Day 17**: Example applications (local_chat)
**Day 18**: Example applications (backend comparison)
**Day 19**: Testing and benchmarks
**Day 20**: Documentation and handoff

---

**END OF PHASE 11 TODO DOCUMENT**
