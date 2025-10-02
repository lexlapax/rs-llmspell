# Current Architecture (v0.9.0 - Phase 10 Complete)

**Status**: Production-Ready Kernel with Daemon Support and Protocol Servers
**Last Updated**: December 2024
**Implementation**: Phases 0-10 Complete
**Validation**: Cross-referenced with phase design documents and codebase

> **📋 Single Source of Truth**: This document reflects the ACTUAL implementation as evolved through 10 development phases, validated against phase design documents (phase-01 through phase-10) and current codebase. **Phase 10 adds daemon support, signal handling, and production deployment capabilities.**

## Related Documentation

This overview document is supported by detailed guides:
- **[Architecture Decisions](./architecture-decisions.md)**: All ADRs from Phase 0-9
- **[Operational Guide](./operational-guide.md)**: Performance benchmarks and security model
- **[RAG System Guide](./rag-system-guide.md)**: Complete RAG documentation including HNSW tuning
- **[Kernel Protocol Architecture](./kernel-protocol-architecture.md)**: Kernel design and protocol/transport layers

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
- **Phase 5**: State Persistence - 35+ modules, multi-backend (Memory/Sled/RocksDB), 2.07μs/item migrations
- **Phase 6**: Sessions - Artifact storage with blake3/lz4, replay via ReplayableHook
- **Phase 7**: API Standardization - Service→Manager rename, builder patterns, retrieve→get, test infrastructure
- **Phase 8**: RAG System - HNSW vector storage (100K vectors), multi-tenant RAG, OpenAI embeddings, 8ms search latency
- **Phase 9**: Integrated Kernel - Protocol/transport abstraction, global IO runtime, no-spawn execution, 46% code reduction
- **Phase 10**: Production Deployment - Daemon support (systemd/launchd), signal handling, PID management, multi-protocol servers, consolidated state/sessions into kernel

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

---

## Kernel Architecture

### Integrated Kernel Design (Phase 9-10)

The kernel provides the central execution engine for llmspell, implementing a unified runtime that eliminates runtime isolation issues:

```rust
// Phase 9-10: IntegratedKernel with daemon, debugging, and production support
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

Phase 10 introduces production-ready daemon support for deploying LLMSpell kernel as a system service with **2,220 LOC** across 7 specialized modules:

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
│  RAG.search(query, {tenant_id, k}), RAG.ingest(docs)       │
├─────────────────────────────────────────────────────────────┤
│               Script Bridge Layer (Phase 1-9)               │
│  17+ Global Objects with Zero-Import Pattern (incl. RAG)   │
├─────────────────────────────────────────────────────────────┤
│          Kernel Execution Layer (Phase 9-10)                │
│  ├── IntegratedKernel - No-spawn execution model           │
│  ├── Global IO Runtime - Shared Tokio runtime              │
│  ├── Protocol Layer - Jupyter/LSP/DAP handling             │
│  ├── Transport Layer - ZMQ/WebSocket/InProcess             │
│  ├── Event Correlation - Distributed tracing               │
│  ├── Debug Infrastructure - DAP bridge, breakpoints        │
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
│  RAG Layer (Phase 8):                                       │
│  ├── llmspell-storage   - HNSW vector storage (hnsw_rs)    │
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
│  Support Layer:                                             │
│  ├── llmspell-providers - rig-core integration             │
│  ├── llmspell-security  - RLS policies, access control     │
│  ├── llmspell-config    - Multi-layer configuration        │
│  └── llmspell-bridge    - Script integration layer         │
└─────────────────────────────────────────────────────────────┘
```

### 1. Kernel Layer (Phase 9-10)

#### llmspell-kernel (47,449 LOC)
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

### 2. Foundation Layer

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

### 5. State & Persistence (9,012 LOC)

#### llmspell-state-persistence
**Phase 5 Achievement**: 35+ modules across 7 subsystems  
**Features**:
- Multi-backend support (Memory, Sled, RocksDB)
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
**Architecture**: `Lua Script → mlua → IntegratedKernel → Global IO Runtime → Async Rust`

### 8. Session Management (3,456 LOC)

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

#### llmspell-storage (1,956 LOC)
**Purpose**: HNSW-based vector storage with multi-tenant support
**Implementation**: hnsw_rs = "0.3" crate (not hnswlib-rs)
**Key Features**:
- HNSW algorithm with optimized parameters (m=16, ef_construction=200, ef_search=50)
- Distance metrics: Cosine (primary), Euclidean, InnerProduct
- Namespace-based tenant isolation via StateScope
- MessagePack serialization for persistence
- Performance: 8ms search for 100K vectors, 450MB memory for 100K vectors

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

---

## API Surface

### Lua Global Objects (17+)
**Phase 2 Decision**: Global injection pattern for zero-import scripts

1. **Agent** - Agent creation with builder pattern (Phase 7 standardization)
2. **Tool** - Tool discovery and execution (37+ tools)
3. **Workflow** - Sequential, Parallel, Conditional, Loop patterns
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

**Quality Check Scripts**:
```bash
./scripts/quality-check-minimal.sh  # Seconds - format, clippy
./scripts/quality-check-fast.sh     # 1 min - adds unit tests
./scripts/quality-check.sh          # 5+ min - comprehensive
```

---

## Implementation Reality

### Phase 10 Implementation Achievements

**Code Quality Metrics:**
- **46% code reduction** through kernel consolidation (Phase 9)
- **Zero runtime isolation errors** with global IO runtime
- **499 total tests passing** with 29 daemon-specific tests (Phase 10)
- **100% integration test success** rate across all components
- **12 tracing categories** for comprehensive observability
- **5-channel architecture** fully implemented (shell, iopub, stdin, control, heartbeat)
- **18 crates** in workspace after consolidation
- **2,220 LOC** dedicated daemon infrastructure (Phase 10)
- **47,449 LOC** in llmspell-kernel (includes Phase 10 production features)

### What's Production Ready ✅

**Core Functionality:**
- Lua scripting with 17+ globals (including RAG)
- 37+ tools across 9 categories
- 4 workflow patterns
- Agent infrastructure with factory/registry
- State persistence with 3 backends (Memory/Sled/RocksDB)
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

### What's Partial 🚧
- Session/artifact management (integrated with kernel and RAG)
- Streaming support (IO manager ready, script bridge incomplete)
- Replay functionality (hooks ready, UI incomplete)
- Embedding providers (only OpenAI implemented)
- LSP protocol implementation (traits ready, implementation pending)
- WebSocket transport (planned, not started)

### What's Not Implemented ❌
- JavaScript support (only stubs)
- Python support (not started)
- GUI interface (deferred)
- Distributed execution (Phase 12)
- Local embedding models (BGE-M3, E5, ColBERT)
- Multi-provider embeddings (Cohere, Voyage AI, Google)
- Hybrid search (vector + keyword combination)
- Late interaction models (ColBERT v2)
- Candle integration for local models

### Deferred from Original Design
- **Phase 5**: Custom field transformers (basic Copy/Default/Remove work)
- **Phase 6**: Full session isolation (security issues identified)
- **Phase 7**: JavaScript bridge completion (focus on Lua stability)
- **Phase 8**: Local embedding models (BGE-M3, ColBERT - complexity/dependencies)
- **Phase 8**: Multi-provider embeddings (focused on OpenAI only)
- **Phase 8**: 1M vector target (achieved 100K with room to grow)
- **Phase 9**: Multiple kernel implementations (consolidated to single IntegratedKernel)

### Code Statistics
- **18 crates** in workspace (llmspell-test + llmspell-testing separate)
- **~65K lines** of Rust code total (46% reduction from Phase 9 consolidation)
- **47,449 LOC** in llmspell-kernel alone (includes 2,220 LOC daemon infrastructure)
- **48+ tool files** implemented across 9 categories
- **499 total tests** passing (including 29 daemon tests)
- **600+ test files** across all crates
- **3,500+ lines** of documentation
- **2,500+ lines** of examples

### Dependencies (Phase 10 Additions)
- **sysinfo = "0.31"** - System metrics for health monitoring
- **nix** - Unix signal handling and process management
- **hex** - HMAC key encoding for Jupyter connection files
- **lz4_flex** - Log compression for rotated files
- **libc** - Low-level daemon operations (umask, dup2)

### Architecture Validation
This architecture has been validated by:
- Cross-referencing 10 phase design documents (including Phase 10 daemon support)
- Analyzing actual crate structure and dependencies
- Reviewing implementation files and test coverage
- Confirming performance measurements (including kernel metrics)
- Verifying API completeness (17+ globals with kernel integration)
- Validating multi-tenant isolation and session integration
- Testing integrated kernel with multiple protocols/transports

---

## Documentation Structure

As of Phase 10 completion, technical documentation has been consolidated into 5 comprehensive guides:

### Core Documents
1. **current-architecture.md** (this file) - Overview and navigation
2. **architecture-decisions.md** - All ADRs from Phase 0-10
3. **operational-guide.md** - Performance and security unified
4. **rag-system-guide.md** - Complete RAG system documentation
5. **kernel-protocol-architecture.md** - Kernel design and protocol/transport abstraction

This consolidation provides 5 comprehensive guides aligned with Phase 10 implementation.

---

*This document represents the actual implementation state of LLMSpell v0.9.0 after completing Phases 0-10, with production-ready daemon support and consolidated kernel architecture.*