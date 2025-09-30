# Kernel and Protocol Architecture

**Version**: v0.9.0
**Status**: Production-Ready with Daemon Support
**Last Updated**: December 2024
**Phase**: 10 (Integrated Kernel with Daemon and Service Support)

## Executive Summary

This document describes the kernel and protocol architecture implemented in LLMSpell v0.9.0. The system uses an **IntegratedKernel** architecture (Phase 9-10) that can run either embedded in the CLI process or as a standalone daemon service. The architecture provides clean separation between transport mechanics (ZeroMQ, InProcess, WebSocket) and protocol semantics (Jupyter, DAP, LSP) through a trait-based design with full daemon support for production deployment.

**Key Decisions**:
- Phase 9: Unified kernel architecture with global IO runtime eliminating "dispatch task is gone" errors
- Phase 10: Added daemon mode with double-fork, signal handling, and systemd/launchd service integration
- Consolidated state/sessions/debug into unified llmspell-kernel crate
- Full 5-channel Jupyter protocol implementation with bidirectional communication

---

## Table of Contents

1. [Kernel Architecture](#1-kernel-architecture)
2. [Protocol Trait System](#2-protocol-trait-system)
3. [Communication Flow](#3-communication-flow)
4. [Jupyter Protocol Implementation](#4-jupyter-protocol-implementation)
5. [Tool Protocol](#5-tool-protocol)
6. [Message Handling](#6-message-handling)
7. [Performance Characteristics](#7-performance-characteristics)
8. [Architecture Decision Records](#8-architecture-decision-records)
9. [Future Protocol Extensions](#9-future-protocol-extensions)
10. [Implementation Guide](#10-implementation-guide)

---

## 1. Kernel Architecture

### 1.1 IntegratedKernel Design (Phase 9-10)

The kernel provides a unified execution runtime that can run embedded or as a daemon:

```rust
// llmspell-kernel/src/execution/integrated.rs
pub struct IntegratedKernel<P: Protocol> {
    /// Script executor with debug support and ComponentRegistry access
    script_executor: Arc<dyn ScriptExecutor>,
    /// Protocol handler (Jupyter/DAP/LSP)
    protocol: P,
    /// Transport layer using dynamic dispatch
    transport: Option<Box<dyn Transport>>,
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
    /// Current client identity for message routing
    current_client_identity: Option<Vec<u8>>,
    /// Current message header (becomes parent_header in replies)
    current_msg_header: Option<serde_json::Value>,
}
```

**Note**: The transport uses dynamic dispatch (`Box<dyn Transport>`) rather than generics, allowing runtime transport selection and switching.

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
   └── Kernel Thread → IntegratedKernel with InProcessTransport
       └── Bidirectional channels via setup_paired_channel()

2. Daemon Mode (Production)
   System Service (systemd/launchd)
   └── IntegratedKernel (forked daemon)
       ├── PID file management
       ├── Signal handling
       └── ZeroMQ 5-channel servers

3. Connection Modes
   - Embedded: InProcessTransport with paired channels
   - Local: Unix sockets or localhost TCP
   - Remote: TCP with authentication
   - Multi-client: MessageRouter handles concurrent clients
```

### 1.4 Kernel Lifecycle

1. **Startup**: Auto-spawn, embedded, or daemon start with service manager
2. **Connection Discovery**: Checks connection files in ~/.llmspell/kernels/
3. **Channel Setup**: All 5 Jupyter channels initialized
4. **Heartbeat Verification**: Dedicated heartbeat channel for liveness
5. **Signal Handling**: SIGTERM/SIGINT for graceful shutdown, SIGHUP for reload
6. **PID Management**: Prevents concurrent instances, enables service control
7. **Idle Timeout**: Configurable shutdown after inactivity
8. **Cleanup**: PID file removal, resource cleanup, state persistence

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
    fn parse_message(&self, data: &[u8]) -> Result<HashMap<String, Value>>;

    // Message creation
    fn create_request(&self, msg_type: &str, content: Value) -> Result<Vec<u8>>;
    fn create_multipart_response(&self, client_id: &[u8], msg_type: &str, content: &Value)
        -> Result<Vec<Vec<u8>>>;

    // Protocol semantics
    fn create_execution_flow(&self, request: &Self::Message) -> ExecutionFlow<Self::Message>;
    fn create_status_message(&self, status: KernelStatus) -> Result<Self::Message>;
    fn create_stream_message(&self, stream: StreamData) -> Result<Self::Message>;
    fn create_execute_result(&self, result: ExecutionResult) -> Result<Self::Message>;
    fn create_error_message(&self, error: ExecutionError) -> Result<Self::Message>;

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
    async fn bind(&mut self, config: &TransportConfig) -> Result<Option<BoundPorts>>;

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

    /// Clone as boxed trait object for dynamic dispatch
    fn box_clone(&self) -> Box<dyn Transport>;
}
```

### 2.3 Transport Implementations

**Available Transports**:

```rust
// llmspell-kernel/src/transport/
pub fn create_transport(transport_type: &str) -> Result<Box<dyn Transport>> {
    match transport_type {
        "inprocess" | "embedded" => {
            // Must use create_pair() for bidirectional communication
            let (kernel_transport, client_transport) = InProcessTransport::create_pair();
            Ok(Box::new(kernel_transport))
        }
        "zeromq" | "zmq" => {
            #[cfg(feature = "zeromq")]
            Ok(Box::new(ZmqTransport::new()?))
        }
        "websocket" | "ws" => {
            #[cfg(feature = "websocket")]
            Ok(Box::new(WebSocketTransport::new()?))
        }
        _ => Err(anyhow::anyhow!("Unknown transport type"))
    }
}
```

### 2.4 InProcessTransport Channel Pairing

For bidirectional communication in embedded mode, transports must be properly paired:

```rust
// Creating a connected pair for embedded mode
let (mut kernel_transport, mut client_transport) = InProcessTransport::create_pair();

// Setup bidirectional channels
for channel_name in ["shell", "control", "iopub", "stdin", "heartbeat"] {
    InProcessTransport::setup_paired_channel(
        &mut kernel_transport,
        &mut client_transport,
        channel_name
    );
}

// Critical: Two separate InProcessTransport::new() instances won't communicate!
// Must use create_pair() and setup_paired_channel() for proper connection
```

**Channel Pairing Architecture**:
```
Transport1 (Kernel):
  channels["shell"] → sends to → Transport2.reverse_channels["shell"]
  reverse_channels["shell"] → receives from → Transport2.channels["shell"]

Transport2 (Client):
  channels["shell"] → sends to → Transport1.reverse_channels["shell"]
  reverse_channels["shell"] → receives from → Transport1.channels["shell"]
```

---

## 3. Communication Flow

### 3.1 Message Flow Architecture

```
CLI/Client                    Transport                    Kernel
    |                            |                           |
    |------ tool_request ------->|                           |
    |        (shell)             |------ forward ----------->|
    |                            |                           |
    |                            |                     handle_message()
    |                            |                           |
    |                            |<----- tool_reply --------|
    |<------ forward ------------|        (shell)            |
    |                            |                           |
```

### 3.2 Client Identity Routing

For proper message routing, especially with ROUTER sockets:

```rust
// Embedded mode uses fixed identity
let client_identity = b"inprocess_client".to_vec();

// ZeroMQ mode extracts from multipart message
let client_identity = if delimiter_idx.is_some() {
    message_parts.first().unwrap().clone()  // Part 0 is identity
} else {
    b"unknown_client".to_vec()
};
```

---

## 4. Jupyter Protocol Implementation

### 4.1 Full 5-Channel Implementation

All Jupyter channels are fully implemented:

```
Channel     Pattern    Purpose                        Status
-------     -------    -------                        ------
shell       ROUTER     Execute requests & replies     ✅ Implemented
control     ROUTER     Control commands               ✅ Implemented
iopub       PUB        Broadcasting outputs           ✅ Implemented
stdin       ROUTER     Input requests                 ✅ Implemented
heartbeat   REP        Liveness monitoring            ✅ Implemented
```

### 4.2 Message Types by Channel

**Shell Channel**:
- execute_request, execute_reply
- complete_request, complete_reply
- inspect_request, inspect_reply
- kernel_info_request, kernel_info_reply
- comm_info_request, comm_info_reply
- history_request, history_reply
- **tool_request, tool_reply** (LLMSpell extension)

**Control Channel**:
- interrupt_request, interrupt_reply
- shutdown_request, shutdown_reply
- debug_request, debug_reply

**IOPub Channel**:
- status (busy/idle)
- execute_input
- execute_result
- stream (stdout/stderr)
- display_data
- error
- debug_event

**Stdin Channel**:
- input_request
- input_reply

### 4.3 Message Format

The kernel handles two message formats:

#### Multipart Jupyter Wire Protocol
```
[0] identity          // Client routing identity
[1] <IDS|MSG>        // Delimiter
[2] signature        // HMAC signature (if auth enabled)
[3] header          // JSON header with msg_type, msg_id, etc.
[4] parent_header   // Parent message header for correlation
[5] metadata        // Additional metadata
[6] content         // Actual message content
[7+] buffers        // Optional binary buffers
```

#### Simple JSON Format (Embedded Mode)
```json
{
  "msg_type": "tool_request",
  "msg_id": "uuid",
  "content": {...},
  "header": {...},
  "metadata": {}
}
```

---

## 5. Tool Protocol

### 5.1 Tool Request/Reply Protocol

The kernel implements tool command handling via special message types:

```rust
// Tool request from CLI to kernel
{
    "msg_type": "tool_request",
    "content": {
        "command": "list" | "info" | "invoke" | "search" | "test",
        "name": "tool_name",     // For info/invoke/test
        "params": {...},         // For invoke
        "category": "...",       // For list/search filtering
        "query": ["..."],        // For search
    }
}

// Tool reply from kernel to CLI
{
    "msg_type": "tool_reply",
    "content": {
        "status": "ok" | "error",
        "tools": [...],          // For list
        "result": {...},         // For invoke
        "info": {...},          // For info
        "matches": [...],        // For search
        "error": "..."          // On error
    }
}
```

### 5.2 Tool Execution Architecture

```
CLI → tool_request → Kernel.handle_message() → handle_tool_request()
                          ↓
              script_executor.component_registry()
                          ↓
                  ComponentRegistry.get_tool()
                          ↓
                    Tool.execute()
                          ↓
              send_tool_reply() → tool_reply → CLI
```

**Key Points**:
- All 40+ tools execute in kernel process, not CLI
- Tools accessed via `script_executor.component_registry()`
- CLI is a thin client sending requests and displaying results
- ComponentRegistry only exists in kernel context

---

## 6. Message Handling

### 6.1 Message Type Recognition

The kernel validates message types based on channel:

```rust
// Shell channel accepts:
if msg_type == "execute_request"
    || msg_type == "complete_request"
    || msg_type == "inspect_request"
    || msg_type == "kernel_info_request"
    || msg_type == "comm_info_request"
    || msg_type == "history_request"
    || msg_type == "tool_request"  // LLMSpell extension

// Control channel accepts:
if msg_type == "interrupt_request"
    || msg_type == "shutdown_request"
    || msg_type == "debug_request"
```

### 6.2 Message Structure Handling

The kernel handles nested message structures:

```rust
// Check for msg_type in header (Jupyter format) or top level (simplified)
let msg_type = parsed_msg.get("header")
    .and_then(|h| h.get("msg_type"))
    .and_then(|v| v.as_str())
    .or_else(|| parsed_msg.get("msg_type").and_then(|v| v.as_str()));

// Flatten header fields to top level for handle_message
if let Some(header_obj) = header.as_object() {
    for (key, value) in header_obj {
        flattened_message.insert(key.clone(), value.clone());
    }
}
```

### 6.3 Client Message Parsing

Clients must handle both multipart and simple formats:

```rust
let reply_msg = if let Some(idx) = delimiter_idx {
    // Parse multipart message (header at idx+2, content at idx+5)
    if reply_parts.len() > idx + 5 {
        let header = serde_json::from_slice(&reply_parts[idx + 2])?;
        let content = serde_json::from_slice(&reply_parts[idx + 5])?;

        let mut msg = HashMap::new();
        msg.insert("header".to_string(), header);
        msg.insert("content".to_string(), content);
        msg
    }
} else {
    // Try parsing as simple JSON for embedded mode
    protocol.parse_message(first_part)?
};
```

---

## 7. Performance Characteristics

### 7.1 Connection Performance

```
# Embedded Mode with InProcessTransport
llmspell run script1.lua  # 55ms (kernel in-process)
llmspell run script2.lua  # 50ms (reuse runtime)
llmspell run script3.lua  # 50ms (reuse runtime)
Total: 155ms

# Daemon Mode with ZeroMQ
llmspell run script1.lua  # 155ms (connect + execute)
llmspell run script2.lua  # 56ms (reuse connection)
llmspell run script3.lua  # 56ms (reuse connection)
Total: 267ms (first run), 168ms (subsequent batch)
```

### 7.2 Performance Metrics

| Metric | Target | Achieved | Notes |
|--------|--------|----------|-------|
| Kernel startup | <100ms | 95ms | First-time spawn |
| InProcess round-trip | <0.1ms | 0.05ms | Direct channel communication |
| ZeroMQ round-trip | <1ms | 0.8ms | Localhost communication |
| Connection reuse | Enabled | ✅ | Faster subsequent runs |
| Memory usage | <100MB | 50MB | Kernel + runtime |
| Tool invocation | <10ms | 8ms | ComponentRegistry lookup |

---

## 8. Architecture Decision Records

### ADR-001: Dynamic Transport Dispatch

**Context**: Need flexibility to switch transports at runtime.

**Decision**: Use `Box<dyn Transport>` instead of generic parameters.

**Benefits**:
- Runtime transport selection
- Easier testing with mock transports
- Simpler API surface
- Support for transport switching without recompilation

### ADR-002: Full Channel Implementation

**Context**: Originally planned single-channel, but full Jupyter compatibility needed.

**Decision**: Implement all 5 Jupyter channels.

**Rationale**:
- Complete Jupyter notebook compatibility
- Proper separation of concerns (control vs execution)
- IOPub broadcasting for multiple clients
- Heartbeat for reliable liveness detection
- stdin for interactive input support

### ADR-003: Embedded Mode with InProcessTransport

**Context**: Need fast local execution without network overhead.

**Decision**: Keep InProcessTransport with proper channel pairing.

**Benefits**:
- Zero network overhead for local scripts
- Faster startup times (55ms vs 155ms)
- Simpler debugging without network layer
- Same protocol semantics as network mode

### ADR-004: Tool Execution in Kernel

**Context**: Tools need access to kernel state and ComponentRegistry.

**Decision**: All tools execute in kernel process, CLI sends requests.

**Rationale**:
- ComponentRegistry only exists in kernel
- Tools need ExecutionContext from kernel
- Enables multi-client tool sharing
- Consistent with Jupyter execution model

---

## 9. Future Protocol Extensions

### 9.1 Supported Protocols

The architecture supports multiple protocols:

```rust
// Implemented protocols (Phase 9-10)
pub type JupyterKernel = IntegratedKernel<JupyterProtocol>;
// DAP is integrated via DAPBridge in IntegratedKernel

// Future protocol implementations
pub type LSPKernel = IntegratedKernel<LSPProtocol>;
pub type MCPKernel = IntegratedKernel<MCPProtocol>;
```

### 9.2 Protocol Capabilities

| Protocol | Purpose | Transport | Status |
|----------|---------|-----------|--------|
| Jupyter | Notebook execution | ZeroMQ/InProcess | ✅ Implemented |
| DAP | Debug adapter | Via Jupyter control | ✅ Implemented |
| Tool | Tool execution | Via Jupyter shell | ✅ Implemented |
| LSP | Language server | TCP/Pipe | 🔮 Future (Phase 11) |
| MCP | Model context | WebSocket | 🔮 Future (Phase 12) |
| HTTP/REST | API access | TCP/HTTP | 🔮 Future |
| gRPC | High-performance RPC | HTTP/2 | 🔮 Future |

---

## 10. Implementation Guide

### 10.1 Creating a Kernel Connection

```rust
// For embedded mode
let (kernel_transport, client_transport) = InProcessTransport::create_pair();
// Setup channels
for channel in &["shell", "control", "iopub", "stdin", "heartbeat"] {
    InProcessTransport::setup_paired_channel(
        &mut kernel_transport,
        &mut client_transport,
        channel
    );
}

// For daemon mode
let mut client = ZmqKernelClient::new(Arc::new(config)).await?;
client.connect_or_start().await?;
```

### 10.2 Kernel Discovery

```rust
async fn find_or_spawn_kernel(&mut self) -> Result<ConnectionInfo> {
    // 1. Check ~/.llmspell/kernels/ for running kernels
    let kernel_dir = dirs::home_dir()
        .unwrap()
        .join(".llmspell/kernels");

    // 2. If found, verify it's alive via heartbeat channel
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

### 10.3 External Kernel Mode

The architecture supports standalone kernel mode:

```bash
# Start kernel daemon
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

**Phase 9-10 Achievements**:
1. **IntegratedKernel** with dynamic transport dispatch
2. **Full 5-channel Jupyter protocol** implementation
3. **Bidirectional InProcessTransport** with channel pairing
4. **Tool protocol** for ComponentRegistry access
5. **DAP debugging** via Jupyter control channel
6. **Flexible message handling** for multipart and simple formats
7. **Daemon mode** with signal handling and service integration
8. **Global IO runtime** eliminating runtime isolation issues

The architecture supports both embedded execution (fast local scripts) and daemon mode (production deployment) while maintaining full Jupyter protocol compatibility and extensibility for future protocols.

---

*This document reflects the actual implementation as of Phase 10, with accurate details about transport architecture, channel implementation, and message handling.*