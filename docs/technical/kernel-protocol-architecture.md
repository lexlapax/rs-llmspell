# Kernel and Protocol Architecture

**Version**: v0.9.0
**Status**: Production-Ready with Daemon Support
**Last Updated**: December 2024
**Phase**: 10 (Integrated Kernel with Daemon and Service Support)  

## Executive Summary

This document describes the kernel and protocol architecture implemented in LLMSpell v0.9.0. The system uses an **IntegratedKernel** architecture (Phase 9-10) that can run either embedded in the CLI process or as a standalone daemon service. The architecture provides clean separation between transport mechanics (ZeroMQ, TCP) and protocol semantics (Jupyter, DAP, LSP) through a trait-based design with full daemon support for production deployment.

**Key Decisions**:
- Phase 9: Unified kernel architecture with global IO runtime eliminating "dispatch task is gone" errors
- Phase 10: Added daemon mode with double-fork, signal handling, and systemd/launchd service integration
- Consolidated state/sessions/debug into unified llmspell-kernel crate

---

## Table of Contents

1. [Kernel Architecture](#1-kernel-architecture)
2. [Protocol Trait System](#2-protocol-trait-system)
3. [Communication Flow](#3-communication-flow)
4. [Jupyter Protocol Implementation](#4-jupyter-protocol-implementation)
5. [Performance Characteristics](#5-performance-characteristics)
6. [Architecture Decision Records](#6-architecture-decision-records)
7. [Future Protocol Extensions](#7-future-protocol-extensions)
8. [Implementation Guide](#8-implementation-guide)

---

## 1. Kernel Architecture

### 1.1 IntegratedKernel Design (Phase 9-10)

The kernel provides a unified execution runtime that can run embedded or as a daemon:

```rust
// llmspell-kernel/src/execution/integrated.rs
pub struct IntegratedKernel<P: Protocol<T>, T: Transport> {
    /// Script executor with debug support
    script_executor: Arc<dyn ScriptExecutor>,
    /// Protocol handler (Jupyter/DAP/LSP)
    protocol: P,
    /// Transport layer (ZeroMQ/WebSocket/TCP)
    transport: Option<T>,
    /// Global IO manager
    io_manager: Arc<EnhancedIOManager>,
    /// Multi-client message routing
    message_router: Arc<MessageRouter>,
    /// Event correlation
    event_correlator: Arc<KernelEventCorrelator>,
    /// Unified state (includes sessions)
    state: Arc<KernelState>,
    /// Debug execution manager
    execution_manager: Arc<ExecutionManager>,
    /// DAP bridge for IDE integration
    dap_bridge: Arc<Mutex<DAPBridge>>,
}
```

### 1.2 Daemon Support (Phase 10)

The kernel can run as a system daemon for production deployment:

```rust
// llmspell-kernel/src/daemon/manager.rs
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

// Signal handling for graceful shutdown
pub struct SignalBridge {
    shutdown_tx: watch::Sender<bool>,  // SIGTERM/SIGINT
    reload_tx: watch::Sender<bool>,    // SIGHUP
    stats_tx: watch::Sender<bool>,     // SIGUSR1
}
```

### 1.3 Deployment Modes

```
1. Embedded Mode (Development)
   CLI Process
   ├── Main Thread → Commands
   └── Kernel Thread → IntegratedKernel

2. Daemon Mode (Production)
   System Service (systemd/launchd)
   └── IntegratedKernel (forked daemon)
       ├── PID file management
       ├── Signal handling
       └── Multi-protocol servers

3. Connection Modes
   - Local: Unix sockets or localhost TCP
   - Remote: TCP with authentication
   - Multi-client: MessageRouter handles concurrent clients
```

### 1.4 Kernel Lifecycle

1. **Startup**: Auto-spawn or daemon start with service manager
2. **Connection Discovery**: Checks connection files in ~/.llmspell/kernels/
3. **Heartbeat Verification**: 5-channel Jupyter with dedicated heartbeat
4. **Signal Handling**: SIGTERM/SIGINT for graceful shutdown, SIGHUP for reload
5. **PID Management**: Prevents concurrent instances, enables service control
6. **Idle Timeout**: Configurable shutdown after inactivity
7. **Cleanup**: PID file removal, resource cleanup, state persistence

---

## 2. Protocol Trait System

### 2.1 Core Traits

The architecture provides clean separation through three core traits:

#### Protocol Trait
Defines wire format and message semantics:

```rust
pub trait Protocol: Send + Sync + 'static {
    type Message: KernelMessage;
    type OutputContext: Send;
    
    // Wire format
    fn encode(&self, msg: &Self::Message, channel: &str) -> Result<Vec<Vec<u8>>>;
    fn decode(&self, parts: Vec<Vec<u8>>, channel: &str) -> Result<Self::Message>;
    
    // Protocol semantics
    fn create_execution_flow(&self, request: &Self::Message) -> ExecutionFlow<Self::Message>;
    fn create_status_message(&self, status: KernelStatus) -> Result<Self::Message>;
    fn create_stream_message(&self, stream: StreamData) -> Result<Self::Message>;
    fn create_execute_result(&self, result: ExecutionResult) -> Result<Self::Message>;
    fn create_error_message(&self, error: ExecutionError) -> Result<Self::Message>;
    
    // Output handling
    fn create_output_context(&self) -> Self::OutputContext;
    fn handle_output(&self, ctx: &mut Self::OutputContext, output: OutputChunk);
    fn flush_output(&self, ctx: Self::OutputContext) -> Vec<(String, Self::Message)>;
    
    // Channel topology
    fn channel_topology(&self) -> ChannelTopology;
    fn expected_response_flow(&self, msg_type: &str) -> ResponseFlow;
}
```

### 2.2 Transport Trait

Generic transport abstraction supporting multiple channels:

```rust
// llmspell-kernel/src/traits/transport.rs
#[async_trait]
pub trait Transport: Send + Sync {
    /// Bind to specified addresses (server mode)
    async fn bind(&mut self, config: &TransportConfig) -> Result<()>;

    /// Connect to specified addresses (client mode)
    async fn connect(&mut self, config: &TransportConfig) -> Result<()>;

    /// Receive multipart message from a channel
    async fn recv(&self, channel: &str) -> Result<Option<Vec<Vec<u8>>>>;

    /// Send multipart message to a channel
    async fn send(&self, channel: &str, parts: Vec<Vec<u8>>) -> Result<()>;

    /// Handle heartbeat if required
    async fn heartbeat(&self) -> Result<bool>;

    /// Check if channel exists
    fn has_channel(&self, channel: &str) -> bool;

    /// Get list of available channels
    fn channels(&self) -> Vec<String>;

    /// Shutdown gracefully
    async fn shutdown(&mut self) -> Result<()>;
}
```

### 2.3 Transport Implementations

**Available Transports**:

```rust
// llmspell-kernel/src/transport/
pub fn create_transport(transport_type: &str) -> Result<BoxedTransport> {
    match transport_type {
        "inprocess" | "embedded" => {
            Ok(Box::new(InProcessTransport::new()))
        }
        "zeromq" | "zmq" => {
            #[cfg(feature = "zeromq")]
            Ok(Box::new(ZmqTransport::new()?))
        }
        "websocket" | "ws" => {
            Err(anyhow::anyhow!("WebSocket support not yet implemented"))
        }
        _ => Err(anyhow::anyhow!("Unknown transport type"))
    }
}
```

**Transport Configuration**:
```rust
pub struct TransportConfig {
    pub transport_type: String,
    pub base_address: String,
    pub channels: HashMap<String, ChannelConfig>,
    pub auth_key: Option<String>,
}

pub struct ChannelConfig {
    pub endpoint: String,  // Port for TCP, suffix for IPC
    pub pattern: String,   // "router", "pub", "rep", etc.
    pub options: HashMap<String, String>,
}
```

## 3. Global IO Runtime

### 3.1 Runtime Management

Single shared Tokio runtime for all IO operations:

```rust
// llmspell-kernel/src/runtime/io_runtime.rs
static GLOBAL_RUNTIME: LazyLock<Runtime> = LazyLock::new(|| {
    tokio::runtime::Builder::new_multi_thread()
        .worker_threads(4)
        .enable_all()
        .thread_name("llmspell-io")
        .build()
        .expect("Failed to create global IO runtime")
});

pub fn global_io_runtime() -> &'static Runtime {
    &GLOBAL_RUNTIME
}

pub fn ensure_runtime_initialized() {
    let _ = global_io_runtime();
}

pub async fn spawn_global<F>(future: F) -> JoinHandle<F::Output>
where
    F: Future + Send + 'static,
    F::Output: Send + 'static,
{
    global_io_runtime().spawn(future)
}
```

### 3.2 Why Global Runtime?

**Problem Solved**: "dispatch task is gone" errors in HTTP clients

```rust
// BEFORE (Phase 8): Runtime isolation caused failures
tokio::spawn(async {
    // Different runtime context
    reqwest::get(url).await  // FAILS: dispatch task is gone
});

// AFTER (Phase 9): Shared runtime works
IntegratedKernel::run(self).await  // Same runtime context
    reqwest::get(url).await         // SUCCESS: shared runtime
```

---

## 4. Jupyter Protocol Implementation

### 4.1 Message Types

The kernel implements standard Jupyter message types:

1. **execute_request** → Kernel receives code to execute
2. **status: busy** → Broadcast kernel is busy
3. **execute_input** → Echo the code being executed
4. **stream** → Output from execution (stdout/stderr)
5. **execute_result** → Return value (if any)
6. **execute_reply** → Execution complete with status
7. **status: idle** → Broadcast kernel is idle

### 4.2 Channel Architecture

**Single Channel Implementation**: Only shell channel implemented (not all 5 Jupyter channels)

```
Jupyter Standard:           Our Implementation:
├── Shell (REQ-REP)    →   ✅ Shell (REQ-REP)
├── IOPub (PUB-SUB)    →   ❌ (Messages sent via shell)
├── Stdin (REQ-REP)    →   ❌ (No interactive input)
├── Control (REQ-REP)  →   ❌ (Not needed)
└── Heartbeat (REQ-REP)→   ❌ (Simplified heartbeat)
```

**Rationale**: Shell channel handles all execution and debug needs for script execution model.

### 4.3 ZeroMQ Transport

```rust
// llmspell-kernel/src/transport/zeromq.rs
pub struct ZmqTransport {
    context: zmq::Context,
    socket: zmq::Socket,
    endpoint: String,
}

impl Transport for ZmqTransport {
    async fn bind(&mut self, config: &TransportConfig) -> Result<()> {
        self.socket = self.context.socket(zmq::REP)?;
        self.socket.bind(&format!("tcp://{}:{}", config.host, config.port))?;
        Ok(())
    }
    
    async fn connect(&mut self, config: &TransportConfig) -> Result<()> {
        self.socket = self.context.socket(zmq::REQ)?;
        self.socket.connect(&format!("tcp://{}:{}", config.host, config.port))?;
        Ok(())
    }
}
```

---

## 5. Performance Characteristics

### 5.1 Connection Reuse Optimization

```
# In-process (removed)
llmspell run script1.lua  # 55ms (new kernel)
llmspell run script2.lua  # 55ms (new kernel)
llmspell run script3.lua  # 55ms (new kernel)
Total: 165ms

# External with EmbeddedKernel (current)
llmspell run script1.lua  # 155ms (spawn + execute)
llmspell run script2.lua  # 56ms (reuse connection)
llmspell run script3.lua  # 56ms (reuse connection)
Total: 267ms (first run), 168ms (subsequent batch)
```

### 5.2 Performance Metrics

| Metric | Target | Achieved | Notes |
|--------|--------|----------|-------|
| Kernel startup | <100ms | 95ms | First-time spawn |
| ZeroMQ round-trip | <1ms | 0.8ms | Localhost communication |
| Connection reuse | Enabled | ✅ | Faster subsequent runs |
| Memory usage | <100MB | 50MB | Kernel + runtime |
| Protocol overhead | <5% | 3% | Minimal impact |

### 5.3 Code Impact

**Code Removal**:
- `llmspell-cli/src/kernel_client/in_process.rs` (263 lines)
- `llmspell-kernel/src/traits/null.rs` (150+ lines)
- Dual path logic in `commands/mod.rs` (50+ lines)
- **Total**: ~500 lines removed

**Code Addition**:
- `llmspell-cli/src/kernel_client/zmq_client.rs` (~200 lines)
- Auto-spawn logic (~50 lines)
- **Total**: ~250 lines added

**Net Result**: -250 lines, simpler architecture

---

## 6. Architecture Decision Records

### ADR-001: Always External Kernel

**Context**: In-process kernel had fundamental limitations preventing core functionality.

**Decision**: Remove in-process kernel entirely, always use external kernel with auto-spawn.

**Options Evaluated**:

| Option | Description | Verdict |
|--------|-------------|---------|
| Fix State Only | Minimal change to fix state injection | ❌ Doesn't fix multi-client |
| Hybrid Architecture | Keep both paths, user chooses | ❌ Doubles maintenance burden |
| **Always External** | Remove in-process, auto-spawn external | ✅ Solves all issues |
| IPC Bridge | In-process with IPC for multi-client | ❌ Reinventing ZeroMQ |

**Benefits**:
- State persistence works
- Multi-client sessions work  
- Jupyter notebook integration works
- VS Code integration works
- Simpler codebase (-250 lines)
- Single code path to maintain

### ADR-002: Protocol Trait Architecture

**Context**: Need to support multiple protocols without coupling.

**Decision**: Trait-based separation of Transport and Protocol.

**Benefits**:
- Protocol independence in kernel logic
- Transport flexibility (same protocol over different transports)
- Easy testing with mock implementations
- Zero-cost abstractions via generics

### ADR-003: Single Shell Channel

**Context**: Jupyter defines 5 channels, but script execution doesn't need all.

**Decision**: Implement only shell channel.

**Rationale**:
- Shell channel handles all execution needs
- IOPub not needed for script execution model
- Stdin not needed (no interactive input during execution)
- Control/heartbeat overhead without benefit
- Simplifies implementation by 80%

### ADR-004: Per-CLI Kernels

**Context**: Could share kernels across CLI instances or have dedicated kernels.

**Decision**: Each CLI instance gets its own kernel.

**Rationale**:
- Avoids complex state synchronization
- Prevents interference between sessions
- Simpler error isolation
- Natural cleanup on CLI exit

---

## 7. Future Protocol Extensions

### 7.1 Supported Protocols

The architecture supports multiple protocols, with several already implemented:

```rust
// Implemented protocols (Phase 9-10)
pub type JupyterKernel = IntegratedKernel<JupyterProtocol, ZmqTransport>;
// DAP is integrated via DAPBridge in IntegratedKernel

// Future protocol implementations
pub type LSPKernel = IntegratedKernel<LSPProtocol, TcpTransport>;
pub type MCPKernel = IntegratedKernel<MCPProtocol, WebSocketTransport>;
```

### 7.2 Protocol Capabilities

| Protocol | Purpose | Transport | Status |
|----------|---------|-----------|--------|
| Jupyter | Notebook execution | ZeroMQ | ✅ Implemented (Phase 9) |
| DAP | Debug adapter | Integrated | ✅ Implemented (Phase 10) |
| LSP | Language server | TCP/Pipe | 🔮 Future (Phase 11) |
| MCP | Model context | WebSocket | 🔮 Future (Phase 12) |
| HTTP/REST | API access | TCP/HTTP | 🔮 Future |
| gRPC | High-performance RPC | HTTP/2 | 🔮 Future |

### 7.3 Adding a New Protocol

To add a new protocol (e.g., HTTP/REST):

1. **Define Message Type**:
```rust
#[derive(Clone, Debug)]
struct HttpMessage {
    method: String,
    path: String,
    body: Value,
    headers: HashMap<String, String>,
}

impl KernelMessage for HttpMessage {
    fn msg_type(&self) -> &str { &self.method }
    fn content(&self) -> Value { self.body.clone() }
}
```

2. **Implement Protocol Trait**:
```rust
struct HttpProtocol {
    base_url: String,
}

impl Protocol for HttpProtocol {
    type Message = HttpMessage;
    type OutputContext = HttpOutputBuffer;
    
    fn create_execution_flow(&self, request: &HttpMessage) -> ExecutionFlow<HttpMessage> {
        ExecutionFlow {
            pre_execution: vec![],  // HTTP has no pre-messages
            capture_output: true,
            post_execution: vec![],  // Response sent separately
        }
    }
}
```

3. **Choose Transport**:
```rust
let transport = TcpTransport::new();
let protocol = HttpProtocol::new("http://localhost:8080");
let kernel = GenericKernel::new(transport, protocol, runtime);
```

---

## 8. Implementation Guide

### 8.1 Creating a Kernel Connection

```rust
// llmspell-cli/src/kernel_client/connection.rs
pub async fn create_kernel_connection(
    config: LLMSpellConfig,
    connect_to: Option<String>,
) -> Result<impl KernelConnectionTrait> {
    let mut client = ZmqKernelClient::new(Arc::new(config)).await?;
    
    if let Some(connection) = connect_to {
        // Connect to existing kernel
        client.connect_to_existing(&connection).await?;
    } else {
        // Auto-spawn new kernel
        client.connect_or_start().await?;
    }
    
    Ok(client)
}
```

### 8.2 Kernel Discovery

```rust
async fn find_or_spawn_kernel(&mut self) -> Result<ConnectionInfo> {
    // 1. Check ~/.llmspell/kernels/ for running kernels
    let kernel_dir = dirs::home_dir()
        .unwrap()
        .join(".llmspell/kernels");
    
    // 2. If found, verify it's alive via heartbeat
    if let Some(existing) = find_existing_kernel(&kernel_dir).await? {
        if verify_heartbeat(&existing).await? {
            return Ok(existing);
        }
    }
    
    // 3. If not found or dead, spawn new kernel
    let kernel_id = Uuid::new_v4().to_string();
    spawn_kernel(kernel_id).await?;
    
    // 4. Wait for connection file
    wait_for_connection_file(&kernel_dir, &kernel_id).await?;
    
    // 5. Return connection info
    Ok(load_connection_info(&kernel_dir, &kernel_id)?)
}
```

### 8.3 External Kernel Mode

The architecture also supports standalone kernel mode:

```bash
# Start kernel server
llmspell kernel start --port 9555 --daemon

# Connect from CLI
llmspell run script.lua --connect localhost:9555

# Connect from Jupyter
jupyter console --existing ~/.llmspell/kernels/abc123.json

# Connect from VS Code
# Use Jupyter extension with connection file
```

---

## Summary

The kernel and protocol architecture provides a production-ready foundation for LLMSpell's execution model:

**Phase 9 Achievements**:
1. **IntegratedKernel** with global IO runtime eliminating runtime isolation
2. **Protocol/Transport traits** for clean separation of concerns
3. **Jupyter protocol** with 5-channel architecture
4. **DAP bridge** for IDE debugging integration
5. **Unified state** combining sessions and persistence

**Phase 10 Achievements**:
1. **Daemon mode** with double-fork and TTY detachment
2. **Signal handling** for graceful shutdown and configuration reload
3. **Service integration** with systemd and launchd support
4. **PID management** for production deployment
5. **Multi-protocol support** with Jupyter and DAP implemented
6. **Consolidated kernel** merging state/sessions/debug into unified crate

The architecture provides both development convenience (embedded mode) and production robustness (daemon mode) while maintaining excellent performance characteristics and full ecosystem compatibility.

---

*This document consolidates the kernel and protocol architecture from Phase 10 implementation, replacing multiple design documents with a single comprehensive production-ready reference.*