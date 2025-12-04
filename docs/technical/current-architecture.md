# Current Architecture (v0.14.0 - Phase 13c Complete)

**Status**: Experimental Platform with Production-Quality Foundations with Unified Storage Architecture, 21 Preset Profiles, Adaptive Memory, Context Engineering, Template System, Local LLM, and IDE Connectivity
**Last Updated**: December 2025
**Implementation**: Phases 0-13c Complete (15/15 major phases)
**Latest**: Phase 13c - Usability & Cohesion Refinement (Storage consolidation, 21 profiles, 56 examples, 5540 tests, 13 dependencies removed)
**Validation**: Cross-referenced with phase design documents and codebase

> **📋 Single Source of Truth**: This document reflects the ACTUAL implementation as evolved through 14 development phases, validated against phase design documents (phase-01 through phase-13c) and current codebase. **Phase 11 adds local LLM support via dual-backend implementation (Ollama + Candle) for cost-free, offline AI operations. Phase 11a consolidates bridge layer with 87% compile speedup, API standardization, and documentation completeness (security 40%→95%, env vars 0%→100%). Phase 11b enhances local LLM with cleanup (-120 LOC), unified profile system (10 builtins), T5 safetensors support (dual-architecture), and platform-aware Metal GPU detection. Phase 12 solves the "0-day retention problem" with 10 experimental platform with production-quality foundations AI workflow templates (6 base + 4 advanced patterns), enabling immediate user value post-installation with CLI-direct execution and comprehensive template library covering 9 categories. Phase 13 introduces adaptive memory with hot-swappable backends (InMemory/HNSW (vectorlite-rs) for episodic, SQLite/PostgreSQL for semantic graph), context engineering with parallel retrieval optimization (~2x speedup), and integrated bridge layer (MemoryBridge/ContextBridge) delivering >90% test coverage, 8.47x HNSW performance gains, and zero breaking changes to existing APIs. Phase 13c achieves unified storage consolidation (4 backends → 1 libsql-based SqliteBackend with vectorlite-rs), removes 13 dependencies (-60MB binary savings), adds 21 preset profiles (4-layer architecture: bases→backends→features→envs→presets), standardizes 56 Lua examples with consistent headers, and achieves 5540 tests with zero clippy warnings.**

## Related Documentation

This overview document is supported by detailed guides:
- **[Architecture Decisions](./architecture-decisions.md)**: All ADRs from Phase 0-13
- **[Performance Guide](./performance-guide.md)**: Performance targets, benchmarking, profiling, optimization (Phase 13b.20.3)
- **[RAG System Guide](./rag-system-guide.md)**: Complete RAG documentation including HNSW tuning
- **[RAG-Memory Integration](./rag-memory-integration.md)**: Phase 13 RAG integration with memory system
- **[Kernel Architecture](./kernel-architecture.md)**: Kernel design, protocol/transport layers, execution paths (Phase 13b.20.2)
- **[Template System Architecture](./template-system-architecture.md)**: Template trait system, 10 built-in templates, advanced patterns (Phase 12)
- **[CLI Command Architecture](./cli-command-architecture.md)**: Complete CLI reference including template+memory commands
- **[Phase 11a Design Document](../in-progress/phase-11a-design-doc.md)**: Bridge consolidation comprehensive documentation
- **[Phase 11b Design Document](../in-progress/phase-11b-design-doc.md)**: Local LLM cleanup & enhancement (2,645 lines, 8 sub-phases)
- **[Phase 12 Design Document](../in-progress/phase-12-design-doc.md)**: Template system design and actual implementation (2,900 lines)
- **[Phase 13 Design Document](../in-progress/phase-13-design-doc.md)**: Adaptive memory & context engineering implementation (updated with actual results)

---

## Table of Contents

1. [Architecture Evolution](#architecture-evolution)
2. [Kernel Architecture](#kernel-architecture)
3. [Core Components](#core-components)
4. [Performance Characteristics](#performance-characteristics)
5. [API Surface](#api-surface)
6. [Testing Infrastructure](#testing-infrastructure)
7. [Implementation Reality](#implementation-reality)

---

## Architecture Evolution

### Phase Progression

- **Phase 0**: Foundation (June 2025) - Core traits (BaseAgent), basic structure
- **Phase 1**: Execution Runtime - llmspell-utils crate, Lua runtime, streaming/multimodal types
- **Phase 2**: Tools Library - 26 self-contained tools, provider/model syntax, DRY principles
- **Phase 3**: Infrastructure - Tool standardization (33→37 tools), agent factory, workflow patterns
- **Phase 4**: Hook System - Event-driven hooks, 40+ points, cross-language support, circuit breakers
- **Phase 5**: State Persistence - 35+ modules, multi-backend (Memory/SQLite/PostgreSQL), 2.07μs/item migrations
- **Phase 6**: Sessions - Artifact storage with blake3/lz4, replay via ReplayableHook
- **Phase 7**: API Standardization - Service→Manager rename, builder patterns, retrieve→get, test infrastructure
- **Phase 8**: RAG System - HNSW vector storage (100K vectors), multi-tenant RAG, OpenAI embeddings, 8ms search latency
- **Phase 9**: Integrated Kernel - Protocol/transport abstraction, global IO runtime, no-spawn execution, 46% code reduction
- **Phase 10**: Production Deployment - Daemon support (systemd/launchd), signal handling, PID management, multi-protocol servers, consolidated state/sessions into kernel
- **Phase 11**: Local LLM Integration - Dual-backend (Ollama via rig + Candle embedded), LocalProviderInstance trait, model CLI commands, 2.5K LOC provider implementation, 40 tok/s inference
- **Phase 11a**: Bridge Consolidation & Documentation Completeness - Feature gates (87% compile speedup: 38s→5s bridge-only), workflow introspection (agent output collection), Tool.execute API standardization (40+ tools), Custom steps removal (876 LOC cleanup), security docs (40%→95% coverage), environment variables (0%→100%, 41+ security vars), Config global bug fix (critical), 1,866 LOC documentation added
- **Phase 11b**: Local LLM Cleanup & Enhancement ✅ COMPLETE - LocalLLM registration fix (14→15 globals), binary removal (-675 LOC, enforced single-binary), unified profile system (10 builtin TOML profiles replacing CLI hack), config consolidation (40+ Lua files updated), model discovery UX (URLs in help), auto-load profile (improved errors), Metal GPU detection (platform-aware device selection), T5 safetensors support (dual-architecture: LLaMA GGUF + T5 safetensors, ModelArchitecture enum, Metal blocked by Candle v0.9), net -120 LOC (+755 new, -875 deleted), 72 tests passing, 0 warnings
- **Phase 12**: Experimental Platform with Production-Quality Foundations AI Agent Templates ✅ COMPLETE (Oct 5-24, 2025) - Template trait system (Template/TemplateRegistry/ExecutionContext/TemplateParams/TemplateOutput), 10 production templates (research-assistant, interactive-chat, data-analysis, code-generator, document-processor, workflow-orchestrator, code-review, content-generation, file-classification, knowledge-management), 2,651 LOC template code, 5 CLI commands (list/info/exec/search/schema), Template global (16th global, 6 methods), 149 tests (122 unit + 27 integration), 3,655 lines documentation, 4 advanced patterns (multi-aspect analysis, quality-driven iteration, scan-classify-act, RAG CRUD), performance targets exceeded 20-50x, zero warnings
- **Phase 13**: Adaptive Memory & Context Engineering ✅ COMPLETE (Jan 2025) - Hot-swappable memory backends (InMemory/HNSW (vectorlite-rs) for episodic, SQLite/PostgreSQL for semantic graph graph), context assembly strategies (episodic/semantic/hybrid/rag/combined), 3 new crates (llmspell-memory 3,500+ LOC, llmspell-graph 2,200+ LOC, llmspell-context simplified), MemoryBridge + ContextBridge integration, 149 total tests (68 memory + 34 graph + 6 E2E + existing), 8.47x HNSW speedup at 10K entries, <2ms episodic add overhead (50x better than target), parallel retrieval optimization (~2x speedup), NoopConsolidationEngine default (regex-based extraction), graph storage (migrated to SQLite/PostgreSQL in Phase 13c), Phase 13.15 accuracy validation deferred (baseline metrics established: DMR 248µs, NDCG 0.87 mock), zero breaking changes to Phase 1-12 APIs
- **Phase 13b**: ScriptRuntime Refactor & PostgreSQL Infrastructure ✅ COMPLETE (Jan 2025) - Infrastructure module (unified component creation from config), ScriptRuntime refactor (single creation path, 200+ LOC → 12 LOC CLI), PostgreSQL 18 + VectorChord integration (15 migrations, 15+ tables, 2,434 DDL lines), 3-tier storage architecture (Memory/SQLite/PostgreSQL backends), 10 storage components (vector embeddings 4 dimensions, episodic/semantic/procedural memory, agent state, workflow states, sessions, artifacts, event log, hook history), RLS multi-tenancy (<5% overhead, 4.9% measured), HNSW vector indexes (8.47x speedup), bi-temporal graph (valid time + transaction time), content-addressed artifacts (blake3 deduplication), monthly event partitioning (12.5x query speedup), connection pooling (formula: CPU × 2 + 1), hot-swappable storage backends (per-component selection), 4,154 lines PostgreSQL documentation, 1,746 lines technical guides
- **Phase 13c**: Usability & Cohesion Refinement ✅ COMPLETE (Dec 2025) - Unified storage consolidation (4 backends → 1 libsql-based SqliteBackend), vectorlite-rs (1,098 LOC pure Rust HNSW replacing hnsw_rs), 13 dependencies removed (-60MB binary size: lazy_static, once_cell, surrealdb, sled, hnsw_rs, quickjs_runtime, blake3, serde_yaml, mysql support), 21 preset profiles (4-layer architecture: 3 bases + 7 backends + 4 features + 1 env → 21 presets), 56 standardized Lua examples with consistent headers, 6 progressive getting-started examples (<30 min onboarding), examples-validation.sh automated testing, 5540 tests passing (100% coverage), 44MB release binary, bidirectional SQLite ↔ PostgreSQL data portability, single-file backup (storage.db), zero clippy warnings maintained

### Key Architectural Decisions (Evolved Through Phases)

- **Phase 1**: BaseAgent trait as universal foundation (ADR-001)
- **Phase 1**: Async-first with sync bridge pattern for scripts (ADR-003/004)
- **Phase 2**: Global injection over require() for zero-import scripts (ADR-005)
- **Phase 3**: Clean break strategy for pre-1.0 improvements
- **Phase 4**: Unified event-driven hook system (<5% overhead) (ADR-009)
- **Phase 5**: Multi-backend state with 4-level scope hierarchy (ADR-007/008)
- **Phase 6**: Content-addressed artifacts with blake3 (10x faster than SHA256)
- **Phase 7**: Universal builder pattern and API standardization (ADR-011/012)
- **Phase 8**: HNSW-based RAG with namespace multi-tenancy (3% isolation overhead) (ADR-013/014)
- **Phase 8**: Separate storage crate for vector operations (ADR-015)
- **Phase 8**: Multi-tenant first design with StateScope integration (ADR-016)
- **Phase 8**: Simplified two-parameter Lua API pattern (ADR-017)
- **Phase 8**: Configuration-driven RAG without compile flags (ADR-018)
- **Phase 9**: Global IO runtime for preventing "dispatch task is gone" (ADR-019)
- **Phase 9**: Protocol/Transport trait abstraction (ADR-020)
- **Phase 9**: No-spawn execution model for kernel (ADR-021)
- **Phase 10**: Daemon process management with double-fork (ADR-022)
- **Phase 10**: Signal bridge for async signal handling (ADR-023)
- **Phase 10**: Unified kernel consolidating state/sessions (ADR-024)
- **Phase 11**: Dual-backend local LLM (Ollama + Candle) over single-path (ADR-025)
- **Phase 11**: LocalProviderInstance trait extending ProviderInstance (ADR-026)
- **Phase 11**: Backend specifier syntax (@ollama/@candle) (ADR-027)
- **Phase 11**: Kernel model protocol for CLI integration (ADR-028)
- **Phase 11a**: Feature gate architecture for optional language runtimes (ADR-042)
- **Phase 11a**: Workflow agent output collection for debugging (ADR-043)
- **Phase 11a**: Tool.execute API standardization across all tools
- **Phase 11a**: Config global fix (empty stub → full implementation)
- **Phase 11b**: LocalLLM global registration via context.providers Arc field (always available fix)
- **Phase 11b**: Single-binary architecture enforcement (removed llmspell-test binary, -675 LOC)
- **Phase 11b**: Unified profile system (10 builtin TOML files replacing 100+ lines CLI mutations)
- **Phase 11b**: Dual-architecture model support (ModelArchitecture enum: LLaMA GGUF + T5 Safetensors)
- **Phase 11b**: Platform-aware device selection (macOS Metal → fallback CPU, Linux/Windows CUDA → fallback CPU)
- **Phase 11b**: Backend specifier precedence (--profile > -c > discovery > default)
- **Phase 11b**: Profile auto-loading with improved error messages (missing profile → clearer guidance)
- **Phase 12**: Template trait system over monolithic orchestration (composable workflow patterns)
- **Phase 12**: Dual-layer registry (ComponentRegistry for scripts + ToolRegistry for templates)
- **Phase 12**: ExecutionContext dependency injection (testability and flexibility)
- **Phase 12**: Parameter schema validation before execution (fail-fast design)
- **Phase 12**: Template global (16th global) following established GlobalObject pattern
- **Phase 12**: CLI-first template design (direct execution without scripting required)
- **Phase 12**: Advanced template patterns (multi-aspect, quality-driven, scan-classify-act, RAG CRUD)
- **Phase 13**: Hot-swappable episodic backends (InMemory for testing, HNSW for production) via MemoryConfig
- **Phase 13**: Semantic memory graph with bi-temporal modeling (valid time + transaction time)
- **Phase 13**: Context assembly strategies (episodic/semantic/hybrid/rag) for token budget optimization
- **Phase 13**: MemoryBridge + ContextBridge integration with kernel message protocol
- **Phase 13b**: Infrastructure module for unified component creation (single path: Infrastructure::from_config())
- **Phase 13b**: ScriptRuntime self-contained creation (CLI delegation, 200+ LOC → 12 LOC)
- **Phase 13b**: 3-tier storage architecture (StorageBackend trait → Memory/SQLite/PostgreSQL implementations)
- **Phase 13b**: PostgreSQL 18 + VectorChord for production storage (HNSW + bi-temporal graph + RLS)
- **Phase 13b**: Per-component backend selection (hot-swap without code changes)
- **Phase 13b**: Content-addressed artifacts with blake3 (50-90% deduplication)
- **Phase 13b**: Monthly event log partitioning (12.5x query speedup via partition pruning)
- **Phase 13b**: Connection pool formula standardization ((CPU × 2) + 1 for optimal concurrency)
- **Phase 13**: SQLite/PostgreSQL graph storage for semantic memory (llmspell-graph crate)
- **Phase 13**: NoopConsolidationEngine default with ManualConsolidationEngine option (simplified vs LLM-driven)
- **Phase 13**: RegexExtractor for entity/relationship extraction (pragmatic vs LLM complexity)
- **Phase 13**: Parallel retrieval optimization using tokio::join! (~2x speedup for hybrid strategy)
- **Phase 13**: MemoryBridge + ContextBridge separate globals (clear separation of concerns)
- **Phase 13**: Opt-in memory/context (zero breaking changes to existing template APIs)
- **Phase 13**: Deferred accuracy validation (Phase 13.15) in favor of experimental platform with production-quality foundations foundation
- **Phase 13c**: Unified libsql storage over 4 fragmented backends (SqliteBackend consolidates HNSW files, SurrealDB, Sled, filesystem)
- **Phase 13c**: vectorlite-rs pure Rust HNSW over hnsw_rs (embedded SQLite extension, MessagePack persistence, zero external dependencies)
- **Phase 13c**: Standard library replacements for init crates (std::sync::LazyLock for lazy_static, std::sync::OnceLock for once_cell)
- **Phase 13c**: 4-layer profile architecture (bases → backends → features → envs → presets) for zero-config deployment
- **Phase 13c**: Pre-1.0 breaking changes acceptable (removed SurrealDB, Sled, hnsw_rs without backward compatibility)
- **Phase 13c**: Single-file storage.db backup over 4 separate backup procedures
- **Phase 13c**: Bidirectional SQLite ↔ PostgreSQL data portability for dev→prod transition
- **Phase 13c**: Example standardization with consistent headers and profile recommendations

---

## Kernel Architecture

### Integrated Kernel Design (Phase 9-11)

The kernel provides the central execution engine for llmspell, implementing a unified runtime that eliminates runtime isolation issues. Phase 11 extends with model management protocol:

```rust
// Phase 9-11: IntegratedKernel with daemon, debugging, production, and local LLM support
pub struct IntegratedKernel<P: Protocol> {
    // Core execution
    script_executor: Arc<dyn ScriptExecutor>,
    protocol: P,                              // Protocol handler (Jupyter/LSP/DAP)
    transport: Option<Box<dyn Transport>>,    // Transport layer (ZMQ/WebSocket/InProcess)

    // I/O and messaging
    io_manager: Arc<EnhancedIOManager>,       // Global IO management
    message_router: Arc<MessageRouter>,       // Multi-client support
    event_correlator: Arc<KernelEventCorrelator>,

    // State and sessions
    state: Arc<KernelState>,                  // Unified state management
    session_manager: SessionManager,          // Session lifecycle

    // Debugging infrastructure (Phase 9)
    execution_manager: Arc<ExecutionManager>, // Debug support
    dap_bridge: Arc<Mutex<DAPBridge>>,       // IDE integration

    // Production features (Phase 10)
    shutdown_coordinator: Arc<ShutdownCoordinator>,     // 6-phase graceful shutdown
    signal_bridge: Option<Arc<SignalBridge>>,          // Unix signal handling
    signal_operations: Arc<SignalOperationsHandler>,   // SIGUSR1/SIGUSR2 handlers
    connection_manager: Option<Arc<Mutex<ConnectionFileManager>>>, // Jupyter discovery
    health_monitor: Arc<HealthMonitor>,                // sysinfo-based monitoring

    // Provider infrastructure (Phase 11)
    provider_manager: Option<Arc<ProviderManager>>,    // Local + cloud provider management

    // Runtime state
    pending_input_request: Option<oneshot::Sender<String>>,
    channel_last_activity: Arc<RwLock<HashMap<String, Instant>>>,
    current_client_identity: Option<Vec<u8>>,
    current_msg_header: Option<Value>,
}
```

### Protocol/Transport Abstraction

**Clean Separation of Concerns:**
- **Protocol Layer**: Handles message semantics (Jupyter, LSP, DAP)
- **Transport Layer**: Handles message transport (ZeroMQ, WebSocket, TCP)

```rust
// Protocol knows about message format
pub trait Protocol: Send + Sync {
    fn parse_message(&self, data: &[u8]) -> Result<HashMap<String, Value>>;
    fn create_response(&self, msg_type: &str, content: Value) -> Result<Vec<u8>>;
    fn create_request(&self, msg_type: &str, content: Value) -> Result<Vec<u8>>;
}

// Transport knows about channels and delivery
pub trait Transport: Send + Sync {
    async fn bind(&mut self, config: &TransportConfig) -> Result<()>;
    async fn connect(&mut self, config: &TransportConfig) -> Result<()>;
    async fn recv(&self, channel: &str) -> Result<Option<Vec<Vec<u8>>>>;
    async fn send(&self, channel: &str, parts: Vec<Vec<u8>>) -> Result<()>;
    async fn heartbeat(&self) -> Result<bool>;
}
```

### Global IO Runtime

**Key Innovation**: Single shared Tokio runtime for all IO operations
- Prevents "dispatch task is gone" errors
- Ensures HTTP clients work in long-running operations
- Zero runtime context mismatches

```rust
// Global runtime management
pub fn global_io_runtime() -> &'static Runtime;
pub fn ensure_runtime_initialized();
pub async fn spawn_global<F>(future: F) -> JoinHandle<F::Output>;
pub fn block_on_global<F>(future: F) -> F::Output;
```

### Execution Model

**No-Spawn Architecture**: IntegratedKernel runs directly without `tokio::spawn`
- Script execution happens in kernel context
- No runtime isolation between components
- Direct message flow from transport to script and back

## Daemon and Service Support (Phase 10)

### Daemon Architecture

Phase 10 introduces experimental platform with production-quality foundations daemon support for deploying LLMSpell kernel as a system service with **2,220 LOC** across 7 specialized modules:

```rust
// Daemon management with double-fork technique (manager.rs - 229 LOC)
pub struct DaemonManager {
    config: DaemonConfig,
    pid_file: Option<PidFile>,
}

pub struct DaemonConfig {
    pub daemonize: bool,
    pub pid_file: Option<PathBuf>,
    pub working_dir: PathBuf,
    pub stdout_path: Option<PathBuf>,
    pub stderr_path: Option<PathBuf>,
    pub close_stdin: bool,
    pub umask: Option<u32>,  // 0o027 for security
}
```

**Double-Fork Process**:
1. First fork creates child, parent exits
2. `setsid()` creates new session (child becomes session leader)
3. Second fork prevents acquiring controlling terminal
4. Change working directory, set umask, redirect I/O
5. Write PID file with exclusive lock

### Signal Handling

**Signal Bridge for Async Runtime** (signals.rs - 593 LOC):
```rust
pub struct SignalBridge {
    handler: SignalHandler,
    shutdown_requested: Arc<AtomicBool>,
    message_sender: Option<mpsc::Sender<KernelMessage>>,
}

// Signal-to-Message Mapping:
// SIGTERM → KernelMessage::ShutdownRequest { restart: false }
// SIGINT  → KernelMessage::InterruptRequest
// SIGHUP  → KernelMessage::ConfigReload
// SIGUSR1 → KernelMessage::ConfigReload
// SIGUSR2 → KernelMessage::StateDump
```

**SignalOperationsHandler** (operations.rs - 704 LOC):
- **SIGUSR1**: Dynamic configuration reload from TOML with change detection
- **SIGUSR2**: State dump to JSON with system metrics (via sysinfo)
- **Prevents concurrent operations**: Atomic flags prevent overlapping reloads/dumps
- **System metrics collection**: Uptime, memory usage, CPU percentage

### Graceful Shutdown

**ShutdownCoordinator** (shutdown.rs - 586 LOC) implements 6-phase graceful shutdown:

```rust
pub enum ShutdownPhase {
    Running,                  // Normal operation
    Initiated,               // Shutdown requested, reject new requests
    WaitingForOperations,    // Wait for active operations to complete
    SavingState,            // Persist kernel state to disk
    NotifyingClients,       // Send shutdown notifications
    Cleanup,                // Release resources
    Complete,               // Shutdown finished
}

pub struct ShutdownCoordinator {
    config: ShutdownConfig,
    phase: Arc<RwLock<ShutdownPhase>>,
    active_operations: Arc<AtomicU64>,
    shutdown_tx: broadcast::Sender<ShutdownPhase>,
    // ... metrics and state
}
```

**OperationGuard RAII Pattern**:
```rust
// Automatic operation tracking
pub struct OperationGuard {
    coordinator: Arc<ShutdownCoordinator>,
}

impl Drop for OperationGuard {
    fn drop(&mut self) {
        self.coordinator.end_operation();
    }
}

// Usage: let _guard = OperationGuard::new(coordinator);
// Operation is automatically tracked and counted
```

**Shutdown Features**:
- Configurable grace period (default 5s) and operation timeout (default 10s)
- State preservation to `~/.llmspell/kernel_state.json`
- Client notification via MessageRouter broadcast
- Statistics tracking (operations completed/cancelled, clients notified)
- Forced shutdown on timeout with partial state save

### Service Deployment

**systemd Support (Linux)**:
```ini
[Service]
Type=forking
ExecStart=/usr/local/bin/llmspell kernel start --daemon --port 9555
PIDFile=/var/run/llmspell/kernel.pid
Restart=on-failure
PrivateTmp=yes
NoNewPrivileges=yes
```

**launchd Support (macOS)**:
```xml
<key>ProgramArguments</key>
<array>
    <string>/usr/local/bin/llmspell</string>
    <string>kernel</string>
    <string>start</string>
    <string>--daemon</string>
</array>
<key>RunAtLoad</key><true/>
<key>KeepAlive</key><true/>
```

### Health Monitoring

**HealthMonitor** (monitoring/mod.rs - 384 LOC) provides comprehensive system tracking:

```rust
pub struct HealthMonitor {
    system: Arc<RwLock<System>>,      // sysinfo 0.31 integration
    thresholds: HealthThresholds,
    pid: Pid,
    start_time: Instant,
}

pub enum HealthStatus {
    Healthy,    // All systems normal
    Degraded,   // Issues detected but functional
    Unhealthy,  // Critical issues, may not be functional
}
```

**Monitored Metrics**:
- **System**: CPU usage, memory (process & system), uptime, load average (Unix)
- **Connections**: Active client count, registered clients, client IDs
- **Performance**: Read/write latency, error rates, circuit breaker trips
- **Thresholds**: Configurable limits with warning/critical levels

**Health Check Performance**: <50ms for full report generation

### Connection Management

**ConnectionFileManager** (connection/mod.rs - 171 LOC) enables Jupyter client discovery:

```rust
pub struct ConnectionInfo {
    pub transport: String,        // "tcp"
    pub ip: String,              // "127.0.0.1"
    pub shell_port: u16,         // Base port
    pub iopub_port: u16,         // Base + 1
    pub stdin_port: u16,         // Base + 2
    pub control_port: u16,       // Base + 3
    pub hb_port: u16,           // Base + 4
    pub key: String,            // HMAC key (hex-encoded, 32-byte random)
    pub signature_scheme: String, // "hmac-sha256"
    pub kernel_name: String,     // "llmspell"
}
```

**Features**:
- Automatic HMAC key generation using `hex` crate
- Connection file written to `~/.llmspell/kernels/kernel-{id}.json`
- Automatic cleanup on shutdown (via Drop trait)
- Port update support for dynamically bound transports

### Log Rotation

**LogRotator** (logging.rs - 644 LOC) provides production logging:

**Features**:
- Size-based rotation (default 10MB) and file count limits (default 5 files)
- Optional lz4 compression for rotated files
- Timestamp-based file naming: `llmspell.log.20250101_143022`
- Automatic cleanup of old files beyond retention limit
- Thread-safe concurrent writes with mutex protection
- Atomic file operations to prevent data loss

### Production Features

- **PID File Management**: Prevents concurrent daemon instances with exclusive locks
- **TTY Detachment**: Complete terminal separation via double-fork
- **I/O Redirection**: Configurable stdout/stderr paths or /dev/null
- **Signal Safety**: Async-signal-safe handlers with atomic flags
- **Health Monitoring**: Real-time system metrics via sysinfo
- **Connection Discovery**: Jupyter-compatible connection files
- **Graceful Shutdown**: 6-phase shutdown with state preservation
- **Log Rotation**: Automatic rotation with compression support

---

## Core Components

```
┌─────────────────────────────────────────────────────────────┐
│                     User Scripts (Lua)                      │
│  RAG.search(), LocalLLM.list(), Agent.create({model="local/llama3.1:8b@ollama"}) │
├─────────────────────────────────────────────────────────────┤
│               Script Bridge Layer (Phase 1-12)              │
│  19 Global Objects with Zero-Import Pattern (incl. Template) │
├─────────────────────────────────────────────────────────────┤
│          Kernel Execution Layer (Phase 9-11)                │
│  ├── IntegratedKernel - No-spawn execution model           │
│  ├── Global IO Runtime - Shared Tokio runtime              │
│  ├── Protocol Layer - Jupyter/LSP/DAP + model_request      │
│  ├── Transport Layer - ZMQ/WebSocket/InProcess             │
│  ├── Event Correlation - Distributed tracing               │
│  ├── Debug Infrastructure - DAP bridge, breakpoints        │
│  ├── Provider Management - ProviderManager integration     │
│  └── Production Layer (Phase 10):                          │
│      ├── Daemon Manager - Double-fork daemonization        │
│      ├── Signal Bridge - Unix signal → kernel messages     │
│      ├── Signal Operations - SIGUSR1/SIGUSR2 handlers      │
│      ├── Shutdown Coordinator - 6-phase graceful shutdown  │
│      ├── Health Monitor - sysinfo-based metrics            │
│      ├── Connection Manager - Jupyter discovery            │
│      └── Log Rotator - Compression and retention           │
├─────────────────────────────────────────────────────────────┤
│                  Rust Core Architecture                     │
│                                                              │
│  Kernel Layer (Phase 9):                                    │
│  └── llmspell-kernel    - Integrated execution engine      │
│                                                              │
│  Foundation Layer (Phase 0-1):                              │
│  ├── llmspell-core      - BaseAgent trait, core types      │
│  └── llmspell-utils     - Shared utilities (Phase 1 DRY)   │
│                                                              │
│  Component Layer (Phase 2-3):                               │
│  ├── llmspell-tools     - 37+ tools (evolved from 26)      │
│  ├── llmspell-agents    - Factory, registry, templates     │
│  └── llmspell-workflows - 4 patterns (Seq/Par/Cond/Loop)   │
│                                                              │
│  RAG Layer (Phase 8, 13c):                                  │
│  ├── llmspell-storage   - Unified libsql storage (vectorlite-rs HNSW) │
│  ├── vectorlite-rs      - Pure Rust HNSW SQLite extension (1,098 LOC) │
│  ├── llmspell-rag       - RAG orchestration, integration   │
│  └── llmspell-tenancy   - Multi-tenant isolation, metrics  │
│                                                              │
│  Infrastructure Layer (Phase 4-7):                          │
│  ├── llmspell-hooks     - 40+ points, circuit breakers     │
│  ├── llmspell-events    - 90K+ events/sec throughput       │
│  ├── llmspell-state-persistence - 35+ modules, 3 backends  │
│  ├── llmspell-sessions  - Artifacts with blake3/lz4        │
│  └── llmspell-testing   - Feature-based test categories    │
│                                                              │
│  Provider Layer (Phase 11):                                 │
│  ├── llmspell-providers - Dual-path LLM integration        │
│  │   ├── Cloud Providers - rig-core (OpenAI, Anthropic, etc.) │
│  │   ├── Ollama Provider - rig + ollama-rs hybrid          │
│  │   └── Candle Provider - Embedded GGUF inference         │
│  │                                                           │
│  Template Layer (Phase 12):                                │
│  └── llmspell-templates - Production-ready workflow templates │
│      ├── Template Trait - Interface for all templates       │
│      ├── TemplateRegistry - Discovery, search, execution    │
│      ├── ExecutionContext - Infrastructure dependency injection │
│      └── Built-in Templates - 10 production templates       │
│          ├── Base (6): research-assistant, interactive-chat, │
│          │             data-analysis, code-generator,        │
│          │             document-processor, workflow-orchestrator │
│          └── Advanced (4): code-review, content-generation, │
│                           file-classification, knowledge-management │
│                                                              │
│  Support Layer:                                             │
│  ├── llmspell-security  - RLS policies, access control     │
│  ├── llmspell-config    - Multi-layer configuration        │
│  └── llmspell-bridge    - Script integration layer         │
└─────────────────────────────────────────────────────────────┘
```

### 1. Provider Layer (Phase 11) - Local + Cloud LLM Integration

#### llmspell-providers (Enhanced with 2.5K LOC local implementation)
**Purpose**: Unified provider abstraction for cloud and local LLMs
**Phase 11 Achievement**: Dual-backend local LLM support (Ollama + Candle)
**Key Innovation**: LocalProviderInstance trait extends ProviderInstance with model management

**Architecture - Three Provider Paths:**

1. **Cloud Providers** (via rig-core):
   - OpenAI, Anthropic, Cohere, Groq, Perplexity, Together, Gemini, Mistral
   - Unified through `RigModel` enum and `RigProvider`
   - Consistent retry/timeout/streaming logic

2. **Ollama Provider** (hybrid rig + ollama-rs):
   ```rust
   // Uses rig for inference, ollama-rs for model management
   pub struct OllamaProvider {
       rig_provider: Arc<Box<dyn ProviderInstance>>,  // Inference
       manager: OllamaModelManager,                    // Model ops
   }
   ```
   - Inference: rig's native Ollama support (CompletionModel)
   - Management: ollama-rs for list/pull/info operations
   - Zero-cost local inference (no API keys required)

3. **Candle Provider** (embedded GGUF + Safetensors):
   ```rust
   pub struct CandleProvider {
       config: CandleConfig,
       device: Device,                              // CPU/CUDA/Metal
       models: Arc<RwLock<HashMap<String, LoadedModel>>>,
       model_directory: PathBuf,
   }
   ```
   - Pure Rust embedded inference via candle-core
   - **Phase 11b.8**: Dual-architecture support (LLaMA GGUF + T5 Safetensors)
   - GGUF model loading from HuggingFace (LLaMA-family)
   - Safetensors model loading for T5 encoder-decoder models
   - Q4_K_M quantization support (~4GB for 7B models)
   - Chat template formatting (TinyLlama-Chat validated)
   - Tokenizer fallback mechanism for model compatibility

**LocalProviderInstance Trait** (src/local/mod.rs:233-320):
```rust
#[async_trait]
pub trait LocalProviderInstance: ProviderInstance {
    async fn health_check(&self) -> Result<HealthStatus>;
    async fn list_local_models(&self) -> Result<Vec<LocalModel>>;
    async fn pull_model(&self, spec: &ModelSpec) -> Result<PullProgress>;
    async fn model_info(&self, model_id: &str) -> Result<ModelInfo>;
    async fn unload_model(&self, model_id: &str) -> Result<()>;
}
```

**ModelSpecifier Backend Extension** (src/model_specifier.rs:10-18):
```rust
pub struct ModelSpecifier {
    pub provider: Option<String>,      // "local", "openai", etc.
    pub model: String,                  // "llama3.1:8b"
    pub backend: Option<String>,        // "ollama" or "candle"
    pub base_url: Option<String>,
}

// Syntax: local/llama3.1:8b@ollama or mistral:7b@candle
```

**Model Management Modules** (src/local/):
- `mod.rs` (386 LOC) - Trait definitions and core types
- `ollama_manager.rs` (161 LOC) - Ollama model operations via ollama-rs
- `ollama_provider.rs` (93 LOC) - Hybrid provider implementation
- `candle/` (~2,017 LOC across 9 modules - Phase 11b.8 adds +160 LOC):
  - `mod.rs` - Provider initialization and GGUF loading
  - `inference.rs` - Text generation with sampling
  - `download.rs` - HuggingFace model downloads via hf-hub
  - `tokenizer.rs` - Tokenizer loading with fallback strategies
  - `chat_template.rs` - Chat formatting for instruct models
  - `device.rs` - Device selection (CPU/CUDA/Metal)
  - `config.rs` - Provider configuration
  - **`model_type.rs` (NEW - 160 LOC)** - Phase 11b.8: ModelArchitecture enum
  - **`model_wrapper.rs` (refactored)** - Phase 11b.8: Enum-based dual architecture

**Phase 11b.8 - T5 Safetensors Support** (4h 52min, Metal BLOCKED):
```rust
// NEW: ModelArchitecture enum for type-safe dispatch (model_type.rs)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModelArchitecture {
    /// LLaMA-family models (quantized GGUF format)
    /// Normalization: RMS-norm (Metal support: BLOCKED by Candle v0.9)
    LLaMA,

    /// T5 encoder-decoder models (safetensors format)
    /// Normalization: LayerNorm (Metal support: BLOCKED by softmax-last-dim missing)
    T5,
}

// Refactored: ModelWrapper now enum supporting both architectures
pub enum ModelWrapper {
    LLaMA {
        model: Box<quantized_llama::ModelWeights>,
        tokenizer: Box<TokenizerLoader>,
        metadata: GGUFMetadata,
        device: Device,
    },
    T5 {
        model: Box<t5::T5ForConditionalGeneration>,
        tokenizer: Box<Tokenizer>,
        config: t5::Config,
        device: Device,
    },
}
```

**Architecture Detection Logic**:
- **GGUF file present** → LLaMA (TinyLlama, Mistral, Phi, Gemma, Qwen)
- **Safetensors + config.json** → T5 (T5, FLAN-T5, UL2, MADLAD400)
- Auto-detection via `ModelArchitecture::detect(path)`

**Platform-Aware Device Selection** (Phase 11b.7 - 45 min):
- **macOS**: Try Metal first → fallback CPU (both LLaMA & T5 blocked by Candle v0.9)
- **Linux/Windows**: Try CUDA first → fallback CPU
- **Graceful fallback**: Models load on CPU when GPU unavailable
- **Clear errors**: "Metal not supported for this model architecture" guidance

**Metal GPU Status** (Phase 11b.8):
- **LLaMA**: BLOCKED by missing RMS-norm kernel in Candle v0.9
- **T5**: BLOCKED by missing softmax-last-dim kernel in Candle v0.9
- **Workaround**: All models fall back to CPU (functional, ~40 tok/s)
- **Future**: Candle v0.10+ may add missing Metal kernels

**Impact**:
- **+755 LOC new code** (model_type.rs, T5 loading, generation logic)
- **2 unit tests passing** (architecture detection, Metal support flags)
- **Net effect**: Dual-architecture foundation for future model families

**Kernel Integration** (Phase 11.3):
- Model protocol handlers in IntegratedKernel (integrated.rs:2502-2880)
- `handle_model_list()` - Query across backends (2527-2603)
- `handle_model_pull()` - Download with progress (2606-2705)
- `handle_model_status()` - Health checks (2708-2807)
- `handle_model_info()` - Model details (2810-2880)

**Provider Factory Registration**:
```rust
// Kernel initialization registers all providers
provider_manager.register_provider("rig", create_rig_provider);
provider_manager.register_provider("ollama", create_ollama_provider);
provider_manager.register_provider("candle", create_candle_provider);
```

**Performance Characteristics**:
- **Ollama**: Functional via REST API, performance varies by model
- **Candle**: 40 tok/s (7B Q4_K_M on modern CPU), <200ms first-token latency
- **Memory**: <5GB for Q4_K_M 7B models, ~450MB per loaded model in Candle
- **Startup**: Ollama instant (external daemon), Candle ~2s model load

### 2. Kernel Layer (Phase 9-11)

#### llmspell-kernel (47,449 LOC + model protocol)
**Purpose**: Central execution engine with integrated runtime and production deployment
**Phase 9 Achievement**: 46% code reduction through consolidation
**Phase 10 Achievement**: Production-ready daemon with health monitoring
**Key Components**:

**Core Modules**:
- `execution/` (3,105 LOC) - IntegratedKernel implementation with daemon support
- `runtime/` - Global IO runtime management
- `transport/` - ZeroMQ, InProcess, Jupyter transports
- `protocols/` - Jupyter wire protocol 5.3
- `traits/` - Protocol and Transport abstractions
- `io/` - Enhanced IO manager with streaming
- `events/` - Event correlation with distributed tracing
- `debug/` - DAP bridge and execution management
- `state/` - Unified kernel state with circuit breaker
- `sessions/` - Integrated session management
- `hooks/` - Kernel-level hook execution
- `api/` - External API surface

**Production Modules (Phase 10 - 2,220 LOC)**:
- `daemon/manager.rs` (229 LOC) - Double-fork daemonization, TTY detachment
- `daemon/signals.rs` (593 LOC) - Signal bridge, kernel message conversion
- `daemon/shutdown.rs` (586 LOC) - 6-phase graceful shutdown, OperationGuard
- `daemon/logging.rs` (644 LOC) - Log rotation, compression, retention
- `daemon/pid.rs` (355 LOC) - PID file management, process locking
- `daemon/operations.rs` (704 LOC) - SIGUSR1/SIGUSR2 handlers, metrics
- `monitoring/mod.rs` (384 LOC) - Health monitoring via sysinfo
- `connection/mod.rs` (171 LOC) - Jupyter connection file management

**Model Protocol (Phase 11)**:
- Model request/reply message types (model_request, model_reply)
- 4 command handlers: list, pull, status, info (integrated.rs:2527-2880)
- ProviderManager integration for local + cloud provider access
- Multi-backend queries (aggregate results from Ollama + Candle)

**Tracing Categories (12 core categories)**:
```rust
pub enum OperationCategory {
    KernelStartup, KernelShutdown,
    MessageReceive, MessageSend,
    ExecuteRequest, ExecuteResponse,
    StateRead, StateWrite, StateMigration,
    SessionCreate, SessionSave, SessionRestore,
    DebugBreakpoint, DebugStep, DebugInspect,
    EventEmit, EventHandle, Custom(String),
}
```

### 3. CLI Layer (Phase 11 - Model Commands)

#### llmspell-cli (Enhanced with 467 LOC model commands)
**Purpose**: Command-line interface for all operations including local model management
**Phase 11 Achievement**: Dual-mode model command handlers (embedded + remote kernel)

**Model Commands** (cli.rs:661-786 + commands/model.rs:467 LOC):
```rust
#[derive(Debug, Subcommand)]
pub enum ModelCommands {
    List,           // List installed models (filter by backend)
    Pull,           // Download models via Ollama or Candle
    Remove,         // Delete local models
    Info,           // Show model details
    Available,      // List available models from libraries
    Status,         // Check backend health
    InstallOllama,  // Install Ollama binary (macOS/Linux)
}
```

**Dual-Mode Architecture** (follows tool.rs pattern):
- **Embedded Mode**: Direct kernel access in same process
- **Connected Mode**: Remote kernel via ClientHandle
- Model requests sent via kernel message protocol
- Consistent behavior across both modes

**Command Examples**:
```bash
llmspell model list --backend ollama
llmspell model pull llama3.1:8b@ollama
llmspell model status
llmspell model info phi3:3.8b
```

**Integration Points**:
- ExecutionContext resolution (embedded vs connected)
- Kernel model protocol (model_request/model_reply messages)
- OutputFormatter for consistent display
- Error handling with user-friendly messages

### 4. Foundation Layer

#### llmspell-core (1,234 LOC)
**Purpose**: Core traits and types defining the entire system  
**Phase 1 Innovation**: BaseAgent as universal foundation  
**Key Components**:
- `BaseAgent` trait - Universal foundation for all components (execute/validate/error handling)
- `Agent`, `Tool`, `Workflow` traits extending BaseAgent
- `ComponentMetadata` - ID, name, version, description for all components
- `ExecutionContext` - State, events, correlation tracking
- `AgentInput/Output` - Multimodal support (text, media, tool calls)
- `AgentStream` - Streaming execution support
- Error hierarchy with 15+ error variants

#### llmspell-utils (2,567 LOC)
**Purpose**: Shared utilities (Phase 1 addition for DRY principle)  
**Key Components**:
- Resource tracking with memory/CPU/time limits
- Path utilities with canonicalization and sandboxing
- Async helpers including retry logic and timeout management
- String manipulation and formatting utilities
- JSON/YAML/TOML serialization helpers
- UUID generation with prefixes
- System info and environment detection

### 2. Tool Library (11,456 LOC)

#### llmspell-tools
**37+ Production Tools** (evolved from 26 in Phase 2 to 37+ in Phase 3)
**Phase 3 Standardization**: Unified parameter naming (input/path/operation)
**Phase 11a Standardization**: Tool.execute() API consistency across all 40+ tools

**API Evolution:**
- **Phase 2-10**: Multiple invocation methods (execute, call, invoke) caused confusion
- **Phase 11a**: Unified to `Tool.execute(name, params)` across all tools
- **Impact**: Zero ambiguity, single source of truth, consistent examples/docs

**Lua API (Consistent):**
```lua
-- All tools use the same pattern
Tool.execute("file-operations", {operation = "read", path = "data.txt"})
Tool.execute("http-request", {url = "https://api.example.com"})
Tool.execute("calculator", {expression = "2+2"})
```

**Categories & Tools**:
- **Utilities (10)**: calculator, datetime-handler, uuid-generator, hash-calculator, base64-encoder, diff-calculator, text-manipulator, template-engine, data-validator, regex-matcher
- **File System (5)**: file-operations, file-search, file-converter, file-watcher, archive-handler  
- **Data Processing (3)**: json-processor, csv-analyzer, xml-processor
- **Web (8)**: web-scraper, api-tester, webhook-caller, url-analyzer, sitemap-crawler, webpage-monitor, http-request, web-search
- **Media (3)**: image-processor, audio-processor, video-processor
- **System (4)**: process-executor, environment-manager, system-monitor, service-checker
- **Communication (2)**: email-sender, database-connector
- **Document (1)**: pdf-processor
- **State (1)**: tool-state

### 3. Agent Infrastructure (8,234 LOC)

**Phase 3.3 Evolution**: Factory pattern, registry, and templates

#### llmspell-agents
- Agent factory with builder pattern
- Component registry for discovery
- Agent templates for common patterns
- Multi-agent coordination support
- Conversation management
- Provider integration via rig-core

### 4. Workflow System (5,123 LOC)

#### llmspell-workflows
**4 Workflow Types** (Phase 3 achievement):
- **Sequential**: Steps execute in order
- **Parallel**: Steps execute concurrently
- **Conditional**: Branching based on conditions
- **Loop**: Iterative execution with state

**Phase 11a Enhancement - Workflow Introspection (ADR-043):**
```rust
pub struct WorkflowResult {
    pub outputs: HashMap<String, serde_json::Value>,
    pub metadata: HashMap<String, String>,
    pub agent_outputs: Vec<(String, serde_json::Value)>,  // Phase 11a: debugging
}
```

**Agent Output Collection** (Phase 11a):
- Collects agent outputs during workflow execution for debugging multi-step workflows
- Zero overhead for tool steps, <1ms overhead per agent step
- Enables workflow introspection without custom logging
- Foundation for Phase 14 (Agent-to-Agent) result passing

**Lua API Example:**
```lua
local result = workflow:execute({})

-- Debug agent outputs
for i, output in ipairs(result.agent_outputs) do
    print("Agent " .. output[1] .. " output:", output[2])
end
```

**StepType Simplification** (Phase 11a):
- Removed unused `StepType::Custom` variant (876 LOC cleanup)
- Simplified to `Tool | Agent` only for clearer abstractions
- Easier to reason about, reduced maintenance burden

### 5. State & Persistence (9,012 LOC)

#### llmspell-state-persistence
**Phase 5 Achievement**: 35+ modules across 7 subsystems  
**Features**:
- Multi-backend support (Memory, SQLite, PostgreSQL)
- Schema migrations at 2.07μs per item (483K items/sec)
- Atomic backup/restore with retention policies
- 4-level scope hierarchy (Global, Session, Workflow, Component)
- Compression (lz4) and encryption support
- Circular reference detection
- Sensitive data protection for API keys

### 6. Hook & Event System (4,567 LOC)

#### llmspell-hooks
**Phase 4 Innovation**: Event-driven hook system with <5% overhead  
**Hook Points**: 40+ defined points across 6 agent states, 34 tools, 4 workflows  
**Features**:
- Pre/post execution hooks with automatic circuit breakers
- State change hooks with correlation tracking
- Cross-language support (Lua, JS, Python adapters)
- ReplayableHook trait for persistence integration
- Built-in hooks: logging, metrics, caching, rate limiting
- HookResult variants: Continue, Modified, Cancel, Redirect, Replace, Retry, Fork, Cache

#### llmspell-events
**Phase 4 Achievement**: 90K+ events/second throughput  
**Features**:
- Async event bus with tokio channels
- Event correlation via UUID tracking
- Backpressure handling for high-frequency events
- Event filtering and routing by type/component
- Integration with hook system for unified event-driven architecture

### 7. Bridge Layer (12,345 LOC)

#### llmspell-bridge
**Phase 1-9 Evolution**: Synchronous wrapper over async Rust with kernel integration
**Phase 11a Enhancement**: Feature-gated language runtimes for compile performance
**Architecture**: `Lua Script → mlua → IntegratedKernel → Global IO Runtime → Async Rust`

**Feature Gate Architecture (Phase 11a - ADR-042):**
```toml
[features]
default = ["lua", "javascript"]
lua = ["mlua", "mlua-vendored"]
javascript = ["boa_engine"]
```

**Compile Performance Impact:**
- **Bridge-only build** (no default features): 38s → 5s (87% faster)
- **Bridge + Lua only**: 38s → 12s (68% faster)
- **Full workspace build**: ~3min (unchanged)
- **Pattern extends to**: Future language runtimes (Python, Ruby) and MCP backends

**Usage:**
```bash
# Fast bridge-only compilation for bridge development
cargo build -p llmspell-bridge --no-default-features  # 5s

# Lua-only compilation
cargo build -p llmspell-bridge --features lua  # 12s

# Full compilation (default)
cargo build -p llmspell-bridge  # 38s
```

**Developer Experience**: 87% faster iteration for bridge layer development, zero runtime performance impact.

**Phase 11b Enhancement - LocalLLM Global Registration Fix:**
```rust
// FIXED (mod.rs:244-247): Changed from get_bridge() check to context.providers Arc
builder.register(Arc::new(local_llm_global::LocalLLMGlobal::new(
    context.providers.create_core_manager_arc().await?,
)));
```
- **Issue**: LocalLLM global was 14/15 globals (missing from injection)
- **Root Cause**: get_bridge() returned None even when providers available
- **Fix**: Use context.providers Arc field directly (always available)
- **Impact**: 15/15 globals injected correctly (Phase 11b.1, 45 min)

### 8. Configuration Layer (Phase 11b - Unified Profiles)

#### llmspell-config (Enhanced with builtin profiles)
**Purpose**: Multi-layer configuration with builtin profile system
**Phase 11b Achievement**: Unified profile system (10 builtin TOML files)
**Key Innovation**: Replace 100+ lines of CLI hacks with declarative profiles

**Builtin Profile System** (llmspell-config/builtins/):
- **10 Profiles**: minimal, development, ollama, candle, rag-development, rag-production, rag-performance, providers, state, sessions
- **Format**: TOML files with complete configuration sections
- **Location**: Embedded in binary via `include_str!()` macros
- **Precedence**: `--profile` > `-c` > discovery > default

**Profile Examples**:
```toml
# builtins/ollama.toml - Ollama-focused configuration
[providers.local]
enabled = true
default_backend = "ollama"
ollama_host = "http://localhost:11434"

# builtins/candle.toml - Candle-focused configuration
[providers.local]
enabled = true
default_backend = "candle"

[providers.local.candle]
model_directory = "~/.llmspell/models/candle"
device = "auto"  # Phase 11b.7: Platform-aware (macOS→Metal, Linux→CUDA, fallback CPU)
```

**Phase 11b.3 Impact** (2h 30min):
- **Removed**: 100+ lines hardcoded CLI profile mutations
- **Added**: 10 builtin TOML profiles with complete config sections
- **Benefit**: Declarative, maintainable, extensible profile system

**Phase 11b.4 - Config Consolidation** (95% complete):
- **40+ Lua example files updated** to use modern config patterns
- **Documentation pending**: Config examples in user guide need updates
- **Quality**: All examples tested and validated

**Phase 11b.6 - Auto-Load Profile** (45 min):
- **Improved error messages** when profile missing or invalid
- **Better guidance** for profile selection and troubleshooting

### 9. Session Management (3,456 LOC)

#### llmspell-sessions
**Phase 6 Implementation**: Complete session and artifact system  
**Features**:
- Session lifecycle with auto-save intervals
- Content-addressed artifact storage (blake3 hashing)
- Automatic compression for artifacts >10KB (lz4_flex)
- Session replay via ReplayableHook integration
- Full context preservation across restarts
- Performance: 24.5μs creation, 15.3μs save

### 9. RAG System (Phase 8) (~6,337 LOC total)

#### llmspell-rag (2,847 LOC)
**Purpose**: RAG orchestration with OpenAI embeddings integration
**Phase 8 Achievement**: Complete RAG system with 8ms search on 100K vectors  
**Key Components**:
- `multi_tenant_integration.rs` - Tenant isolation via StateScope
- `state_integration.rs` - StateScope-aware vector operations
- `session_integration.rs` - Session-scoped RAG with TTL support
- `embeddings/` - OpenAI text-embedding-3-small (384 dimensions only)
- `traits/` - Hybrid retrieval traits for future expansion
- `chunking/` - Document chunking strategies (sliding window implemented)

#### llmspell-storage (10,000+ LOC - Phase 13c expansion)
**Purpose**: Unified storage backend with 10 storage components via libsql
**Implementation**: libsql v0.9 + vectorlite-rs (Phase 13c consolidated)
**Key Features**:
- **10 Storage Components**: V3 (vectors), V4 (graph), V5 (procedural), V6 (agent), V7 (KV), V8 (workflow), V9 (sessions), V10 (artifacts), V11 (events), V13 (hooks)
- **vectorlite-rs HNSW**: Pure Rust implementation (1,098 LOC), embedded SQLite extension, MessagePack persistence
- HNSW algorithm with optimized parameters (m=16, ef_construction=200, ef_search=50)
- Distance metrics: Cosine (primary), Euclidean, InnerProduct
- **Bi-temporal graph storage**: SqliteGraphStorage replaces SurrealDB, recursive CTE traversal (1-10 hops)
- **State storage**: SqliteStateStorage replaces Sled (agent, KV, workflow states)
- Namespace-based tenant isolation via StateScope
- Performance: <8ms vector search, <50ms graph traversal, <10ms state operations
- **Single-file backup**: storage.db replaces 4 separate backup procedures
- **PostgreSQL parity**: Bidirectional SQLite ↔ PostgreSQL data portability

**API Surface**:
```rust
#[async_trait]
pub trait VectorStorage: Send + Sync {
    async fn store(&self, entry: VectorEntry) -> Result<String>;
    async fn search(&self, query: VectorQuery) -> Result<Vec<VectorResult>>;
    async fn delete(&self, id: &str) -> Result<bool>;
    async fn get_stats(&self) -> Result<StorageStats>;
    async fn clear(&self) -> Result<()>;
    async fn persist(&self) -> Result<()>;
}
```

#### llmspell-tenancy (1,534 LOC) 
**Purpose**: Multi-tenant vector management and cost tracking  
**Key Features**:
- Tenant isolation via `StateScope::Custom("tenant:id")` pattern
- Usage metrics (embeddings, searches, storage bytes, costs)
- Resource limits and quota enforcement
- Per-tenant vector configuration and constraints

**Multi-Tenant Architecture**:
```rust
pub struct TenantUsageMetrics {
    pub embeddings_generated: u64,
    pub embedding_tokens: u64,
    pub searches_performed: u64,
    pub documents_indexed: u64,
    pub storage_bytes: u64,
    pub embedding_cost_cents: u64,
}
```

### 10. Security Framework (2,847 LOC)

#### llmspell-security
**Enhanced Security Model** (Phase 3 + 8):
```rust
pub enum SecurityLevel {
    Safe,       // No file/network access
    Restricted, // Limited, validated access
    Privileged, // Full system access
}

// Phase 8: Row-level Security for RAG operations
pub enum AccessDecision {
    Allow,
    Deny(String),
    AllowWithFilters(Vec<SecurityFilter>),  // Multi-tenant filtering
}
```

**Phase 8 RAG Security Features**:
- Compile-time safe tenant isolation via StateScope types
- No cross-tenant data leakage by design (namespace separation)
- Session vectors with automatic TTL-based expiration
- Access control policies enforced at vector storage layer
- Audit logging for all multi-tenant operations

**Sandboxing Features** (Phase 3 + 8):
- Lua stdlib restrictions (no os.execute, io.popen)
- Path traversal prevention
- Resource limit enforcement
- Network domain whitelisting
- IntegratedSandbox for RAG operations (file/network/resource controls)

### 11. Debug Infrastructure (1,890 LOC)

#### llmspell-utils/debug & llmspell-bridge
**Comprehensive Debug System** (Phase 7):

**Architecture Layers**:
```
Script Layer (Lua/JS) → Debug Global API
     ↓
Bridge Layer → DebugBridge (thread-safe wrapper)
     ↓  
Core Layer → DebugManager (global singleton)
```

**Core Components**:
- **DebugManager**: Global singleton via LazyLock with atomic operations
- **Performance Profiler**: Statistical analysis with percentiles
- **Module Filtering**: Hierarchical, wildcard, and regex patterns
- **Stack Trace Collection**: Lua-specific frame capture
- **Object Dumping**: Circular reference detection

**Key Features**:
- Zero-cost when disabled (atomic bool check)
- Thread-safe with interior mutability
- <10ms operation overhead
- Circular buffer for captured entries
- Pluggable output handlers

---

## Performance Characteristics

### Measured Performance (Validated in Phases 5-8)

| Operation | Target | Actual | Phase Achieved |
|-----------|--------|--------|----------------|
| Tool Initialization | <10ms | <10ms | Phase 2 ✅ |
| Agent Creation | <50ms | <50ms | Phase 3 ✅ |
| Hook Overhead | <5% | <2% | Phase 4 ✅ |
| State Write | <5ms | <5ms | Phase 5 ✅ |
| State Read | <1ms | <1ms | Phase 5 ✅ |
| State Migration | - | 2.07μs/item | Phase 5 ✅ |
| Event Throughput | 50K+/sec | 90K+/sec | Phase 4 ✅ |
| Session Creation | - | 24.5μs | Phase 6 ✅ |
| Session Save | - | 15.3μs | Phase 6 ✅ |
| Memory Baseline | <50MB | 12-15MB | Phase 1 ✅ |
| Global Injection | <5ms | 2-4ms | Phase 2 ✅ |
| Vector Search (100K) | <10ms | 8ms | Phase 8 ✅ |
| Vector Insertion (1K) | <200ms | 180ms | Phase 8 ✅ |
| Memory/100K vectors | <500MB | 450MB | Phase 8 ✅ |
| Embedding (single) | <100ms | ~80ms | Phase 8 ✅ |
| Embedding (batch 32) | <500ms | ~400ms | Phase 8 ✅ |
| Tenant Isolation | <5% | 3% | Phase 8 ✅ |
| Session Vector TTL | <20ms | 15ms | Phase 8 ✅ |
| Kernel Startup | <200ms | <100ms | Phase 9 ✅ |
| Message Processing | <10ms | <5ms | Phase 9 ✅ |
| Protocol Parsing | <5ms | <1ms | Phase 9 ✅ |
| Transport Send/Recv | <5ms | <1ms | Phase 9 ✅ |
| Event Correlation | <1ms | <100μs | Phase 9 ✅ |
| Debug Stepping | <20ms | <10ms | Phase 9 ✅ |
| Daemon Startup | <3s | 1.8s | Phase 10 ✅ |
| Signal Response | <100ms | 85ms | Phase 10 ✅ |
| Health Check | <100ms | <50ms | Phase 10 ✅ |
| Connection File Write | <20ms | <10ms | Phase 10 ✅ |
| Tool CLI (list) | <50ms | ~15ms | Phase 10 ✅ |
| Log Rotation | <100ms | ~50ms | Phase 10 ✅ |
| Ollama Model List | <100ms | ~50ms | Phase 11 ✅ |
| Candle Model Load | <3s | ~2s | Phase 11 ✅ |
| Candle Inference (7B) | <200ms | ~150ms | Phase 11 ✅ |
| Candle Throughput | >30 tok/s | 40 tok/s | Phase 11 ✅ |
| Model Download (GGUF) | - | ~4GB/5min | Phase 11 ✅ |
| HuggingFace Tokenizer | <500ms | ~300ms | Phase 11 ✅ |
| Bridge compile (no features) | - | 5s | Phase 11a ✅ |
| Bridge compile (lua only) | - | 12s | Phase 11a ✅ |
| Bridge compile (full) | 38s | 38s | Phase 11a baseline |
| Workflow output collection | - | <1ms | Phase 11a ✅ |
| Tool.execute standardization | - | 100% | Phase 11a ✅ |
| LocalLLM global registration | - | 15/15 | Phase 11b ✅ |
| Profile system overhead | - | 0ms | Phase 11b ✅ |
| Model architecture detection | <50ms | <10ms | Phase 11b ✅ |
| T5 model loading | <5s | ~3s | Phase 11b ✅ |
| Platform device selection | <100ms | <50ms | Phase 11b ✅ |

---

## API Surface

### Lua Global Objects (19)
**Phase 2 Decision**: Global injection pattern for zero-import scripts
**Phase 11a Consistency**: Tool.execute() standardization
**Phase 12 Addition**: Template global for workflow execution

1. **Agent** - Agent creation with builder pattern (Phase 7 standardization)
2. **Tool** - Tool discovery and execution via Tool.execute() (37+ tools, Phase 11a standardization)
3. **Workflow** - Sequential, Parallel, Conditional, Loop patterns (Phase 11a: agent output collection)
4. **State** - Persistence with save/load/migrate (Phase 5)
5. **Session** - Lifecycle with artifacts (Phase 6)
6. **Hook** - Registration for 40+ hook points (Phase 4)
7. **Event** - Emission with correlation tracking
8. **Config** - Multi-layer configuration (Phase 7)
9. **Provider** - LLM providers with provider/model syntax
10. **Debug** - Utilities with configurable verbosity
11. **JSON** - Manipulation with jq-like queries
12. **Args** - CLI argument parsing
13. **Streaming** - Coroutine-based streaming (Phase 1)
14. **Artifact** - Storage with compression (Phase 6)
15. **Replay** - Session replay via hooks (Phase 6)
16. **RAG** - Vector storage and retrieval with multi-tenant support (Phase 8)
17. **Metrics** - Performance metrics collection and monitoring
18. **LocalLLM** - Local model management (list, pull, info, status) (Phase 11)
19. **Template** - Production-ready workflow templates (list, info, execute, search, schema) (Phase 12)

### RAG API (Phase 8)
**Simplified Two-Parameter Pattern**:
```lua
-- Basic operations
RAG.ingest(doc, {options})              -- Ingest document with optional scope
RAG.search(query, {k = 5, scope = id})  -- Search with k results and scope

-- Multi-tenant operations  
RAG.ingest(doc, {scope = "tenant:acme"})
RAG.search(query, {k = 10, scope = "tenant:acme"})

-- Session-scoped operations
RAG.create_session_collection(session_id, ttl_seconds)
RAG.ingest(doc, {scope = "session", scope_id = session_id})

-- Get statistics
RAG.get_stats(namespace, scope)
```

### LocalLLM API (Phase 11)
**Simplified Local Model Management**:
```lua
-- Check backend status
local status = LocalLLM.status()  -- {ollama: {running: true, models: 17}, candle: {ready: true}}

-- List installed models
local models = LocalLLM.list()    -- [{id, backend, size_bytes, quantization}, ...]

-- Download model
local progress = LocalLLM.pull("llama3.1:8b@ollama")  -- {model_id, status, percent_complete}

-- Get model info
local info = LocalLLM.info("phi3:3.8b")  -- {id, backend, size, format, loaded, ...}

-- Create agent with local model
local agent = Agent.create({
    model = "local/llama3.1:8b@ollama",  -- Backend auto-detection or explicit
    temperature = 0.7
})

-- Backend selection
local ollama = Agent.create({model = "local/phi3:3.8b@ollama"})   -- Force Ollama
local candle = Agent.create({model = "local/mistral:7b@candle"})  -- Force Candle
local auto = Agent.create({model = "local/llama3.1:8b"})          -- Auto (prefers Ollama)
```

### Template API (Phase 12)
**Experimental Platform with Production-Quality Foundations Workflow Templates**:
```lua
-- List available templates
local templates = Template.list()  -- [{id, name, description, category, version, tags}, ...]
local research = Template.list("Research")  -- Filter by category

-- Get template info
local info = Template.info("research-assistant")
-- {metadata, config_schema: {parameters: [{name, type, required, default, constraints}]}}

local info_with_schema = Template.info("research-assistant", true)  -- Include full schema

-- Execute template
local result = Template.execute("research-assistant", {
    topic = "Rust async programming",
    max_sources = 5,
    model = "ollama/llama3.2:3b"
})
-- Returns: {result: {text|structured|file}, artifacts: [], metadata: {...}, metrics: {duration_ms, tokens_used}}

-- Advanced templates
local code_review = Template.execute("code-review", {
    code_path = "src/main.rs",
    aspects = {"security", "quality", "performance"},
    model = "ollama/llama3.2:3b"
})

local content = Template.execute("content-generation", {
    topic = "Introduction to Rust",
    content_type = "blog_post",
    quality_threshold = 8,
    max_iterations = 5,
    model = "ollama/llama3.2:3b"
})

-- Search templates
local matches = Template.search("RAG")  -- [{id, name, description, category}, ...]

-- Get parameter schema
local schema = Template.schema("knowledge-management")
-- {version, parameters: [{name, description, param_type, required, default, constraints}]}
```

### Core Rust Traits

```rust
// Phase 1: BaseAgent as foundation
#[async_trait]
pub trait BaseAgent: Send + Sync {
    fn metadata(&self) -> &ComponentMetadata;
    async fn execute(&self, input: AgentInput, context: ExecutionContext) -> Result<AgentOutput>;
    async fn validate_input(&self, input: &AgentInput) -> Result<()>;
    async fn handle_error(&self, error: LLMSpellError) -> Result<AgentOutput>;
    
    // Optional streaming (Phase 1)
    async fn stream_execute(&self, input: AgentInput, context: ExecutionContext) -> Result<AgentStream>;
    fn supports_streaming(&self) -> bool { false }
    fn supports_multimodal(&self) -> bool { false }
}

// Phase 2-3: Specialized traits
pub trait Agent: BaseAgent { /* LLM-specific */ }
pub trait Tool: BaseAgent { /* Tool-specific */ }
pub trait Workflow: BaseAgent { /* Workflow-specific */ }
```

---

## Testing Infrastructure

### Test Categories (Phase 7 Reorganization)
**llmspell-testing crate**: Centralized test infrastructure
**Phase 11b.2**: Single-binary architecture enforcement (-675 LOC, 15 min)

**Feature-Based Categories**:
- `unit-tests` - Component unit tests
- `integration-tests` - Cross-component tests
- `external-tests` - Network-dependent tests
- `agent-tests` - Agent-specific scenarios
- `tool-tests` - Tool validation tests
- `workflow-tests` - Workflow pattern tests
- `benchmark-tests` - Performance measurements
- `stress-tests` - Load and stability tests
- `security-tests` - Security validation

**Test Suites**:
- `fast-tests` - Unit + integration (<1 minute)
- `comprehensive-tests` - All except external
- `all-tests` - Complete test suite

**Phase 11b.2 - Binary Removal** (15 min, -675 LOC):
```
REMOVED: llmspell-testing/src/bin/ (204 LOC)
REMOVED: llmspell-testing/src/runner/ (471 LOC)
```
- **Before**: llmspell binary + llmspell-test binary (dual-binary architecture)
- **After**: Only llmspell binary remains (single-binary architecture)
- **Rationale**: llmspell-test binary was unused, added maintenance burden
- **Impact**: Enforced single-binary philosophy, cleaner architecture
- **Testing**: All test features work through cargo test --features

**Quality Check Scripts**:
```bash
./scripts/quality-check-minimal.sh  # Seconds - format, clippy
./scripts/quality-check-fast.sh     # 1 min - adds unit tests
./scripts/quality-check.sh          # 5+ min - comprehensive
```

---

## Implementation Reality

### Phase 11 Implementation Achievements

**Code Quality Metrics:**
- **46% code reduction** through kernel consolidation (Phase 9)
- **Zero runtime isolation errors** with global IO runtime
- **5540 total tests passing** with comprehensive coverage across all crates (Phase 13c)
- **100% integration test success** rate across all components including local LLM
- **12 tracing categories** for comprehensive observability
- **5-channel architecture** fully implemented (shell, iopub, stdin, control, heartbeat)
- **21 crates** in workspace after Phase 13c consolidation (includes vectorlite-rs)
- **2,220 LOC** dedicated daemon infrastructure (Phase 10)
- **2,497 LOC** dedicated local provider infrastructure (Phase 11)
- **47,449 LOC** in llmspell-kernel (includes Phase 10 production + Phase 11 model protocol)
- **10,000+ LOC** in llmspell-storage (Phase 13c expansion with 10 storage components)
- **1,098 LOC** in vectorlite-rs (pure Rust HNSW SQLite extension)
- **~75K LOC** total Rust code (Phase 13c adds storage consolidation + vectorlite-rs)

**Phase 11 Local LLM Metrics:**
- **Dual-backend architecture**: Ollama (external) + Candle (embedded)
- **Performance**: 40 tok/s Candle inference, <200ms first-token latency
- **Memory efficiency**: <5GB for Q4_K_M 7B models
- **Model support**: LLaMA 3.1, Mistral, Phi-3, TinyLlama validated
- **Documentation**: 580 LOC (320-line guide + 260-line examples)
- **Zero warnings**: Full clippy compliance maintained

### Phase 11a Consolidation Metrics

**Code Quality Improvements:**
- **87% compile speedup** for bridge-only builds (38s → 5s)
- **876 lines removed** through Custom steps cleanup
- **1,866 lines added** for documentation (security, env vars, design doc)
- **100% API consistency** (Tool.execute across 40+ tools)
- **Zero compiler warnings** maintained across workspace
- **All quality gates passing** (format, clippy, compile, test, doc)

**Documentation Completeness:**
- **Security coverage**: 40% → 95% (+371 lines security-and-permissions.md)
- **Environment variables**: 0% → 100% (+405 lines across 3 guides)
- **TOML schema accuracy**: 30% → 95% (fixed fake [security.sandboxing] sections)
- **Deployment patterns**: 6 documented (GitHub Actions, GitLab CI, Docker, Docker Compose, systemd, CLI)

**Quality Metrics Achieved:**

| Metric | Before | After | Improvement |
|--------|--------|-------|-------------|
| Bridge compile time | 38s | 5s | 87% faster |
| Security docs coverage | 40% | 95% | +55% |
| Env vars documentation | 0% | 100% | +100% |
| API consistency (tools) | 60% | 100% | +40% |
| TOML schema accuracy | 30% | 95% | +65% |
| Code removed | 0 | 876 lines | Simplification |
| Documentation lines | baseline | +1,866 lines | Comprehensive |

**Critical Bug Fixes:**
- **Config global**: Fixed empty stub → full ConfigBridgeGlobal implementation
- **TOML schema**: Removed fake [security.sandboxing], added correct [tools.*] sections

**Architectural Decisions:**
- **ADR-042**: Feature gate architecture for optional language runtimes
- **ADR-043**: Workflow agent output collection for debugging

**Foundation for Future Phases:**
- **Phase 12 (Memory)**: Fast iteration via compile speedup, workflow debugging, security docs
- **Phase 13 (MCP)**: Feature gates extend to MCP backends, Tool.execute for MCP tools
- **Phase 14 (A2A)**: Workflow introspection for result passing, security isolation
- **Phase 15 (Dynamic Workflows)**: Simplified StepType enum easier to generate

### Phase 13c Consolidation Metrics

**Storage Consolidation:**
- **Unified backend**: 4 backends → 1 libsql-based SqliteBackend
- **vectorlite-rs**: 1,098 LOC pure Rust HNSW (replacing hnsw_rs external dependency)
- **10 storage components**: V3-V13 (vectors, graph, procedural, agent, KV, workflow, sessions, artifacts, events, hooks)
- **Graph storage**: SqliteGraphStorage replaces SurrealDB (bi-temporal, recursive CTE traversal)
- **State storage**: SqliteStateStorage replaces Sled (agent, KV, workflow states)
- **Single-file backup**: storage.db replaces 4 separate backup procedures
- **Data portability**: Bidirectional SQLite ↔ PostgreSQL export/import

**Dependency Reduction (13 removed):**
- **Initialization crates**: lazy_static → std::sync::LazyLock, once_cell → std::sync::OnceLock (Rust 1.80+)
- **Storage backends removed**: surrealdb, sled, hnsw_rs, rocksdb (transitively)
- **Unused dependencies**: quickjs_runtime, blake3, serde_yaml, mysql support
- **Tokio ecosystem**: tokio-stream removed from 4 crates, tokio-util removed from 2 crates
- **Binary size reduction**: ~60MB savings from dependency removal
- **Compile time improvement**: 10-25% faster builds

**Profile System (21 presets):**
- **4-layer architecture**: bases (3) → backends (7) → features (4) → envs (1) → presets (21)
- **Zero-config deployment**: Pick a preset and start scripting
- **Decision matrix**: Clear profile selection guide for different use cases
- **Example integration**: All 56 examples include profile recommendations

**Example Standardization:**
- **56 Lua examples**: All with consistent headers and documentation
- **6 getting-started**: Progressive <30 minute onboarding path
- **examples-validation.sh**: Automated testing ensures all examples work
- **Profile recommendations**: Every example specifies its optimal profile

**Quality Metrics:**
- **5540 tests**: 100% passing across all crates
- **Zero clippy warnings**: Strict linting maintained
- **44MB release binary**: Optimized with LTO
- **Zero breaking changes**: All existing scripts and configurations work unchanged

### Phase 11b Enhancement Metrics (7/8 complete, 95%)

**Code Quality Improvements:**
- **Net -120 LOC** (+755 new, -875 deleted) - continued code reduction trend
- **72 tests passing** with 0 warnings - maintained quality standards
- **Single-binary architecture** enforced (-675 LOC binary removal)
- **15/15 globals registered** (LocalLLM fix from 14/15)
- **10 builtin profiles** replacing 100+ lines CLI mutations
- **Dual-architecture support** (LLaMA GGUF + T5 Safetensors)

**Sub-Phase Breakdown** (8 tasks, 7 complete, 1 partial):
- **11b.1**: LocalLLM Registration Fix (45 min) ✅
- **11b.2**: Binary Removal (-675 LOC, 15 min) ✅
- **11b.3**: Unified Profile System (2h 30min) ✅
- **11b.4**: Config Consolidation (95% complete) 🚧
- **11b.5**: Model Discovery UX (20 min) ✅
- **11b.6**: Auto-Load Profile (45 min) ✅
- **11b.7**: Metal GPU Detection (45 min) ✅
- **11b.8**: T5 Safetensors Support (4h 52min) ✅

**Technical Achievements:**

| Component | Before | After | Impact |
|-----------|--------|-------|--------|
| Global registration | 14/15 | 15/15 | LocalLLM now injected |
| Binary count | 2 | 1 | -675 LOC, cleaner architecture |
| Profile system | CLI hacks | 10 TOML files | Declarative, maintainable |
| Model architectures | LLaMA only | LLaMA + T5 | Dual-architecture foundation |
| Device selection | Manual | Platform-aware | macOS→Metal, Linux→CUDA, fallback CPU |
| Metal support | Hoped | Blocked | Clear status (Candle v0.9 limitations) |
| Lua examples | Outdated | Updated | 40+ files modernized |

**Time Investment**: 9h 27min (actual) across 8 sub-phases
**Documentation**: Phase 11b design doc (2,645 lines) created in previous session

**Known Limitations:**
- **Metal GPU**: Both LLaMA (RMS-norm) and T5 (softmax-last-dim) blocked by Candle v0.9
- **Config consolidation**: User guide examples need updates (5% remaining)
- **Platform-specific testing**: Metal GPU fallback tested on macOS only

**Foundation for Phase 12:**
- **Dual-architecture pattern** ready for additional model families (BERT, GPT-2, etc.)
- **Unified profile system** supports complex memory configurations
- **Platform-aware device selection** enables GPU optimization experiments

### What's Production Ready ✅

**Core Functionality:**
- Lua scripting with 17+ globals (including RAG)
- 37+ tools across 9 categories
- 4 workflow patterns
- Agent infrastructure with factory/registry
- State persistence with 3 backends (Memory/SQLite/PostgreSQL)
- Hook system with 40+ points, <2% overhead
- Event system with 90K+ events/sec throughput
- Security sandboxing with tenant isolation

**RAG System:**
- HNSW vector storage supporting 100K+ vectors
- OpenAI embeddings (text-embedding-3-small, 384 dims)
- Multi-tenant RAG with StateScope isolation (3% overhead)
- Session-scoped RAG with TTL support
- Simplified two-parameter Lua API

**Kernel Infrastructure:**
- Integrated kernel with protocol/transport abstraction
- Global IO runtime preventing dispatch errors
- Multi-protocol support (Jupyter primary, DAP integrated)
- 5-channel Jupyter architecture (shell, iopub, stdin, control, heartbeat)
- Comprehensive distributed tracing (12 categories)
- Debug infrastructure with DAP bridge, breakpoint support

**Production Deployment (Phase 10):**
- **Daemon Infrastructure** (2,220 LOC):
  - Double-fork daemonization with TTY detachment
  - PID file management with exclusive locking
  - systemd/launchd service file templates
- **Signal Handling**:
  - SIGTERM/SIGINT → graceful shutdown
  - SIGHUP/SIGUSR1 → dynamic config reload
  - SIGUSR2 → state dump with metrics
  - Signal-to-kernel-message bridge (async-safe)
- **Graceful Shutdown**:
  - 6-phase shutdown (Initiated → WaitingForOperations → SavingState → NotifyingClients → Cleanup → Complete)
  - OperationGuard RAII pattern for automatic tracking
  - Configurable timeouts and grace periods
- **Health Monitoring**:
  - sysinfo integration for CPU/memory/uptime metrics
  - Three-tier status (Healthy/Degraded/Unhealthy)
  - Configurable thresholds with warning/critical levels
  - <50ms health check performance
- **Connection Management**:
  - Jupyter-compatible connection file generation
  - HMAC key generation for message signing
  - Automatic cleanup on shutdown
- **Log Management**:
  - Size-based log rotation with compression (lz4)
  - Configurable retention policies
  - Thread-safe concurrent writes
- **Tool CLI Commands**:
  - Direct tool invocation without scripts (list, info, invoke, search, test)
  - Kernel message protocol integration
  - Dual-mode support (embedded/connected)

**Local LLM Integration (Phase 11):**
- **Dual-Backend Support** (2.5K LOC):
  - Ollama provider via rig + ollama-rs hybrid (inference + management)
  - Candle provider with embedded GGUF inference (pure Rust)
  - LocalProviderInstance trait for model operations
- **Model Management**:
  - CLI commands: list, pull, remove, info, available, status (467 LOC)
  - Kernel protocol: model_request/model_reply messages
  - 4 handlers: list, pull, status, info (integrated.rs:2527-2880)
- **Script API**:
  - LocalLLM global (18th Lua global)
  - Agent.create() with local/model@backend syntax
  - Backend auto-detection (Ollama preferred)
- **ModelSpecifier Extension**:
  - Backend field for @ollama/@candle suffix
  - Parse: local/llama3.1:8b@ollama
- **Testing & Validation**:
  - 10 integration tests (5 Ollama + 5 Candle)
  - 100% test pass rate
  - Zero compiler/clippy warnings
- **Documentation**:
  - 320-line user guide (docs/user-guide/local-llm.md)
  - 4 experimental platform with production-quality foundations Lua examples (260 lines)
  - 6 troubleshooting scenarios
- **Performance**:
  - Candle: 40 tok/s, <200ms first-token latency
  - Memory: <5GB for Q4_K_M 7B models
  - Model loading: ~2s for Candle GGUF

**Phase 11a Consolidation (October 2025):**
- **Developer Experience**:
  - Fast bridge compilation (5s for bridge-only builds, 87% speedup)
  - Zero-ambiguity API (Tool.execute across all tools)
  - Comprehensive documentation (security 95%, env vars 100%)
- **Workflow Debugging**:
  - Agent output collection for multi-step workflow introspection
  - <1ms collection overhead per agent step
  - Foundation for advanced debugging and A2A communication
- **Documentation Completeness**:
  - Security & permissions guide (371 lines, 3 levels, 4 scenarios, 5 troubleshooting)
  - Environment variables (41+ security vars, 6 deployment patterns)
  - Correct TOML schema examples (fixed fake sections)
- **Code Health**:
  - Simplified workflow abstractions (Custom steps removed, 876 LOC cleanup)
  - Config global bug fixed (critical)
  - Zero compiler warnings maintained

### What's Partial 🚧
- Session/artifact management (integrated with kernel and RAG)
- Streaming support (IO manager ready, script bridge incomplete, Candle not implemented)
- Replay functionality (hooks ready, UI incomplete)
- Embedding providers (only OpenAI implemented)
- LSP protocol implementation (traits ready, implementation pending)
- WebSocket transport (planned, not started)
- Candle streaming inference (non-streaming complete)

### What's Not Implemented ❌
- JavaScript support (only stubs)
- Python support (not started)
- GUI interface (deferred)
- Distributed execution (Phase 12)
- Local embedding models (BGE-M3, E5, ColBERT - Phase 12)
- Multi-provider embeddings (Cohere, Voyage AI, Google - Phase 12)
- Hybrid search (vector + keyword combination - Phase 12)
- Late interaction models (ColBERT v2 - Phase 12)
- Advanced Candle features:
  - Streaming inference (basic non-streaming complete)
  - GPU optimization beyond basic CUDA/Metal
  - Multi-model concurrent loading
  - Custom sampling strategies beyond temperature/top-p/top-k

### Deferred from Original Design
- **Phase 5**: Custom field transformers (basic Copy/Default/Remove work)
- **Phase 6**: Full session isolation (security issues identified)
- **Phase 7**: JavaScript bridge completion (focus on Lua stability)
- **Phase 8**: Local embedding models (BGE-M3, ColBERT - complexity/dependencies)
- **Phase 8**: Multi-provider embeddings (focused on OpenAI only)
- **Phase 8**: 1M vector target (achieved 100K with room to grow)
- **Phase 9**: Multiple kernel implementations (consolidated to single IntegratedKernel)

### Code Statistics
- **21 crates** in workspace (Phase 13c adds vectorlite-rs)
- **~75K lines** of Rust code total (Phase 13c adds storage consolidation + vectorlite-rs)
- **47,449 LOC** in llmspell-kernel (includes 2,220 LOC daemon + model protocol)
- **10,000+ LOC** in llmspell-storage (Phase 13c: 10 unified storage components)
- **1,098 LOC** in vectorlite-rs (pure Rust HNSW SQLite extension)
- **llmspell-providers**: Enhanced with 2,657 LOC local implementation (+160 Phase 11b.8)
  - `local/` directory: 386 (mod) + 161 (ollama_manager) + 93 (ollama_provider) + ~2,017 (candle, +160 LOC Phase 11b.8)
  - New files: `model_type.rs` (160 LOC), `model_wrapper.rs` (refactored to enum)
- **llmspell-cli**: Enhanced with 467 LOC model commands
- **llmspell-config**: Enhanced with 21 preset profiles (Phase 13c: 4-layer architecture)
- **llmspell-testing**: Reduced by 675 LOC (binary removal Phase 11b.2)
- **48+ tool files** implemented across 9 categories
- **5540 total tests** passing (Phase 13c: comprehensive coverage, 0 warnings)
- **600+ test files** across all crates
- **56 Lua examples** (Phase 13c: standardized with consistent headers)
- **4,080+ lines** of documentation (adds 580 LOC Phase 11 docs)
- **6 getting-started examples** (Phase 13c: progressive <30 min onboarding)

### Dependencies (Phase 10-11-13c Additions/Removals)

**Phase 10 (Daemon/Production):**
- **sysinfo = "0.31"** - System metrics for health monitoring
- **nix** - Unix signal handling and process management
- **hex** - HMAC key encoding for Jupyter connection files
- **lz4_flex** - Log compression for rotated files
- **libc** - Low-level daemon operations (umask, dup2)

**Phase 11 (Local LLM):**
- **ollama-rs = "0.3.2"** - Ollama model management (list, pull, info)
- **candle-core** (workspace) - Core tensor operations and device abstraction
- **candle-transformers** (workspace) - Pre-built transformer models and GGUF loading
- **hf-hub** (workspace) - HuggingFace model downloads
- **tokenizers** (workspace) - Fast tokenization with fallback support
- **rig-core = "0.25"** (upgraded) - Ollama inference support added

**Phase 13c (Storage Consolidation - 13 dependencies REMOVED):**
- **REMOVED lazy_static** → std::sync::LazyLock (Rust 1.80+ standard library)
- **REMOVED once_cell** → std::sync::OnceLock (Rust 1.80+ standard library)
- **REMOVED surrealdb** - Replaced by SqliteGraphStorage (llmspell-storage)
- **REMOVED sled** - Replaced by SqliteStateStorage (llmspell-storage)
- **REMOVED hnsw_rs** - Replaced by vectorlite-rs (pure Rust, workspace crate)
- **REMOVED rocksdb** (transitively via surrealdb)
- **REMOVED quickjs_runtime** - Unused (no code references)
- **REMOVED blake3** - Test-only, replaced by sha2 (already in deps)
- **REMOVED serde_yaml** - Unused (no code references)
- **REMOVED MySQL support** - sqlx-mysql removed (mock-only, never functional)
- **REMOVED tokio-stream** from 4 crates (kept in llmspell-events only)
- **REMOVED tokio-util** from 2 crates (no longer needed)
- **ADDED libsql = "0.9"** - Unified SQLite storage with encryption support
- **ADDED vectorlite-rs** - Workspace crate (1,098 LOC pure Rust HNSW)
- **ADDED zerocopy = "0.8"** - Zero-copy Vec<f32> marshaling for vectors

### Architecture Validation
This architecture has been validated by:
- Cross-referencing 15 phase design documents (Phase 0-13c, including 3,765-line Phase 13c design doc)
- Analyzing actual crate structure and dependencies (21 crates)
- Reviewing implementation files and test coverage (5540 tests passing)
- Confirming performance measurements (including local LLM inference and compile speedup)
- Verifying API completeness (19 globals including LocalLLM, Template, Memory, Context)
- Validating multi-tenant isolation and session integration
- Testing integrated kernel with multiple protocols/transports
- Validating dual-backend local LLM (Ollama + Candle)
- Confirming ModelSpecifier backend parsing and routing
- Testing CLI model commands in embedded + connected modes
- Validating Phase 11a feature gates (87% compile speedup)
- Verifying workflow introspection (agent output collection)
- Confirming Tool.execute standardization across 40+ tools
- Validating security documentation completeness (95% coverage)
- Verifying environment variables documentation (100% coverage, 41+ vars)
- Validating Phase 11b LocalLLM global registration (15/15 globals)
- Confirming single-binary architecture enforcement (-675 LOC)
- Testing unified profile system (21 preset profiles)
- Validating dual-architecture model support (LLaMA GGUF + T5 Safetensors)
- Confirming platform-aware device selection (macOS Metal, Linux CUDA, fallback CPU)
- Testing Metal GPU fallback behavior (both architectures blocked by Candle v0.9)
- **Phase 13c**: Validating unified libsql storage backend (SqliteBackend with 10 components)
- **Phase 13c**: Confirming vectorlite-rs HNSW performance (<8ms search, 8.47x speedup)
- **Phase 13c**: Testing SqliteGraphStorage bi-temporal traversal (recursive CTEs, 1-10 hops)
- **Phase 13c**: Verifying 13 dependencies removed (~60MB binary savings)
- **Phase 13c**: Validating 21 preset profiles (4-layer architecture)
- **Phase 13c**: Testing 56 standardized Lua examples (examples-validation.sh)
- **Phase 13c**: Confirming SQLite ↔ PostgreSQL data portability

---

## Documentation Structure

As of Phase 13c completion (v0.14.0), technical documentation has been consolidated into 5 comprehensive guides plus Phase 11-13c design documentation:

### Core Documents
1. **current-architecture.md** (this file) - Overview and navigation (updated for Phase 13c)
2. **architecture-decisions.md** - All ADRs from Phase 0-13c
3. **performance-guide.md** - Performance targets, benchmarking, profiling, optimization (Phase 13b.20.3)
4. **rag-system-guide.md** - Complete RAG system documentation
5. **kernel-architecture.md** - Kernel design, protocol/transport abstraction, execution paths (Phase 13b.20.2)

### Phase 11 User Documentation
6. **docs/user-guide/local-llm.md** (320 lines) - Local LLM integration guide
   - Ollama setup and configuration
   - Candle embedded inference
   - Model management CLI commands
   - Script API (LocalLLM global)
   - 4 experimental platform with production-quality foundations Lua examples (260 lines)
   - Troubleshooting guide (6 scenarios)

### Phase 11a Design Documentation
7. **docs/in-progress/phase-11a-design-doc.md** (1,685 lines) - Comprehensive consolidation documentation
   - 12 sections covering all 8 sub-phases
   - Component sections: Feature Gates, Workflow Output, API Naming, Custom Steps, Security Docs, Env Vars
   - Quality metrics, testing validation, architectural impact
   - Lessons learned & future enablement (Phase 12-15)
8. **ADR-042**: Feature gate architecture for optional language runtimes
9. **ADR-043**: Workflow agent output collection for debugging

### Phase 11a User Documentation Updates
10. **docs/user-guide/security-and-permissions.md** (+256 lines) - Environment variable override section
    - CI/CD patterns (GitHub Actions, GitLab CI)
    - Container patterns (Docker, Docker Compose)
    - Service patterns (systemd)
    - 41+ security environment variables documented
11. **docs/user-guide/configuration.md** (+143 lines) - Security & Permissions Variables section
    - Complete security env var reference
    - Environment variable to config path mapping
    - Common deployment patterns
12. **docs/user-guide/getting-started.md** (+6 lines) - Optional security env vars for development

### Phase 11b Design Documentation
13. **docs/in-progress/phase-11b-design-doc.md** (2,645 lines) - Local LLM cleanup & enhancement
    - 8 sub-phases: LocalLLM fix, binary removal, unified profiles, config consolidation, model UX, auto-load, Metal detection, T5 support
    - Executive summary with code quality metrics (-120 net LOC, 72 tests, 0 warnings)
    - Architecture diagrams for ModelArchitecture enum and device selection flow
    - Component deep-dives: Profile system, T5 architecture, platform-aware GPU selection
    - Known limitations: Metal GPU blocked by Candle v0.9 (both LLaMA & T5)
    - Lessons learned & future roadmap (Candle v0.10+, additional model families)

This consolidation provides comprehensive technical and user-facing documentation aligned with Phase 13c implementation.

### Phase 13c Design Documentation
14. **docs/in-progress/phase-13c-design-doc.md** (3,765 lines) - Usability & Cohesion Refinement
    - Storage consolidation architecture (4 backends → 1 libsql)
    - vectorlite-rs pure Rust HNSW implementation
    - 21 preset profiles (4-layer architecture)
    - 56 standardized Lua examples
    - Dependency reduction strategy (13 removed)
    - SQLite ↔ PostgreSQL data portability

---

*This document represents the actual implementation state of LLMSpell v0.14.0 after completing Phases 0-13c, with unified storage consolidation (SqliteBackend with 10 components, vectorlite-rs pure Rust HNSW), experimental platform with production-quality foundations daemon support, consolidated kernel architecture, dual-backend local LLM integration (Ollama + Candle) for cost-free offline AI operations, Phase 11a bridge consolidation (87% compile speedup, 95% docs coverage, API standardization, workflow introspection), Phase 11b local LLM enhancements (LocalLLM fix, single-binary architecture, unified profile system, dual-architecture model support with LLaMA GGUF + T5 Safetensors, platform-aware device selection), and Phase 13c usability refinements (13 dependencies removed saving ~60MB, 21 preset profiles, 56 standardized examples, 5540 tests passing, 44MB release binary, SQLite ↔ PostgreSQL data portability).*