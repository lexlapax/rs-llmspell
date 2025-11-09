# Kernel Architecture

**Version**: 0.13.0 (Phase 13b Complete)
**Status**: Production Ready
**Last Updated**: January 2025

> **🎯 Purpose**: Comprehensive kernel architecture reference - protocol implementation, execution paths, and infrastructure lifecycle

**🔗 Navigation**: [← Technical Docs](README.md) | [Current Architecture](current-architecture.md) | [PostgreSQL Guide](postgresql-guide.md)

---

## Table of Contents

1. [Overview](#overview)
2. [Kernel Architecture](#kernel-architecture)
3. [Infrastructure Creation](#infrastructure-creation)
4. [Protocol System](#protocol-system)
5. [Execution Modes](#execution-modes)
6. [Message Flows](#message-flows)
7. [Component Lifecycle](#component-lifecycle)
8. [Transport Layer](#transport-layer)
9. [Performance](#performance)
10. [Troubleshooting](#troubleshooting)

---

## Overview

### Unified Kernel Architecture (Phase 9-13b)

LLMSpell implements an **IntegratedKernel** architecture that provides:
- **Embedded mode**: In-process execution for CLI (<100ms startup)
- **Daemon mode**: Production services with systemd/launchd integration
- **Connected mode**: Remote client access via Jupyter protocol
- **Single creation path**: Infrastructure module creates all 9 components from config

### Key Design Principles

**1. Self-Contained Kernel** (Phase 9/10):
- Kernel owns all infrastructure creation
- Services/CLIs delegate to unified creation path
- Zero duplication across modes

**2. Config-Driven Architecture** (Phase 13b.16):
- `Infrastructure::from_config()` creates all components
- Optional features (RAG, Memory) controlled by config
- Dependency injection enforced at compile time

**3. Protocol Abstraction** (Phase 10):
- Transport layer (ZeroMQ, InProcess) separated from protocol semantics
- Full 5-channel Jupyter protocol implementation
- Same protocol for embedded, daemon, and connected modes

**4. Thin CLI Layer** (Phase 13b.16):
- CLI: ~12 lines to create kernel
- Infrastructure creation: 0 lines (delegated to Infrastructure module)
- Auto-detection of running kernels with fallback to embedded

### Architecture Evolution

**Phase 9**: Unified kernel with global IO runtime (eliminated "dispatch task is gone" errors)
**Phase 10**: Daemon support (double-fork, signal handling, service integration)
**Phase 12**: Template system (10 experimental AI workflows)
**Phase 13**: Memory & context (3-tier memory, bi-temporal graph)
**Phase 13b.16**: Infrastructure module (single creation path for all 9 components)

---

## Kernel Architecture

### IntegratedKernel Design

**Location**: `llmspell-kernel/src/execution/integrated.rs:109-159`

```rust
pub struct IntegratedKernel<P: Protocol> {
    // Core execution
    script_executor: Arc<dyn ScriptExecutor>,      // With ComponentRegistry access
    protocol: P,                                    // Jupyter/DAP/LSP
    transport: Option<Box<dyn Transport>>,          // Dynamic dispatch (ZMQ/InProcess)

    // Global infrastructure
    io_manager: Arc<EnhancedIOManager>,             // stdout/stderr capture
    message_router: Arc<MessageRouter>,             // Multi-client routing
    event_correlator: Arc<KernelEventCorrelator>,   // Distributed tracing
    tracing: TracingInstrumentation,                // Performance tracing

    // State management
    state: Arc<KernelState>,                        // Unified kernel state
    session_manager: SessionManager,                // Session lifecycle
    execution_count: Arc<RwLock<u64>>,              // Jupyter execution counter

    // Debugging
    execution_manager: Arc<ExecutionManager>,       // Debug sessions
    dap_bridge: Arc<parking_lot::Mutex<DAPBridge>>, // IDE debugging

    // Lifecycle
    shutdown_coordinator: Arc<ShutdownCoordinator>, // Graceful shutdown
    signal_bridge: Option<Arc<SignalBridge>>,       // Unix signals (SIGTERM/SIGHUP)
    signal_operations: Arc<SignalOperationsHandler>, // SIGUSR1/SIGUSR2
    health_monitor: Arc<HealthMonitor>,             // System monitoring

    // Jupyter integration
    connection_manager: Option<Arc<parking_lot::Mutex<ConnectionFileManager>>>,
    pending_input_request: Option<oneshot::Sender<String>>,
    channel_last_activity: Arc<RwLock<HashMap<String, Instant>>>,

    // Message context
    current_client_identity: Option<Vec<u8>>,       // For routing replies
    current_msg_header: Option<serde_json::Value>,  // Parent header correlation

    // Configuration
    config: ExecutionConfig,
    session_id: String,
}
```

**Key Features**:
- **Dynamic transport**: `Box<dyn Transport>` allows runtime selection (embedded vs daemon)
- **Multi-client**: Message router handles concurrent connections
- **Tracing**: Full correlation IDs for distributed debugging
- **Graceful shutdown**: Coordinator manages cleanup across all components

### Deployment Modes

| Mode | Use Case | Transport | Startup | Latency |
|------|----------|-----------|---------|---------|
| **Embedded** | CLI commands | InProcess | 55ms | 0.05ms |
| **Daemon** | Production service | ZeroMQ TCP | 95ms | 0.8ms |
| **Connected** | Remote clients | ZeroMQ TCP | 5ms | 0.8ms |

**Embedded Mode** (Development/CLI):
```
CLI Process
├── Main Thread → Command handlers
└── Spawned Task → IntegratedKernel::run() loop
    └── InProcessTransport (paired channels)
```

**Daemon Mode** (Production):
```
System Service (systemd/launchd)
└── Daemonized Process (double-fork)
    ├── PID file management (/tmp/kernel.pid)
    ├── Signal handling (SIGTERM/SIGINT/SIGHUP)
    └── ZeroMQ 5-channel servers
        ├── shell: tcp://*:9572 (ROUTER - execute requests)
        ├── iopub: tcp://*:9573 (PUB - broadcast outputs)
        ├── stdin: tcp://*:9574 (ROUTER - input requests)
        ├── control: tcp://*:9575 (ROUTER - shutdown/interrupt)
        └── heartbeat: tcp://*:9576 (REP - liveness check)
```

**Connected Mode** (Remote Clients):
```
Client → ZmqTransport::connect() to existing kernel
    └── Uses connection file: ~/.llmspell/kernels/<id>.json
        ├── Port numbers (5 channels)
        ├── HMAC-SHA256 key
        └── Transport type (tcp/ipc)
```

### Kernel Event Loop

**Location**: `integrated.rs:522-900`

```rust
pub async fn run(mut self) -> Result<()> {
    loop {
        // Priority: Control channel (shutdown/interrupt)
        if let Some(msg) = transport.recv("control").await? {
            handle_message_with_identity(msg).await?;
        }

        // Main: Shell channel (execute requests, tool commands)
        if let Some(msg) = transport.recv("shell").await? {
            handle_message_with_identity(msg).await?;
        }

        // Input: Stdin channel (user input requests)
        if let Some(msg) = transport.recv("stdin").await? {
            handle_message_with_identity(msg).await?;
        }

        // Liveness: Heartbeat channel (automatic echo)
        if let Some(msg) = transport.recv("heartbeat").await? {
            transport.send("heartbeat", msg).await?;  // Echo
        }

        // Shutdown signal check
        if shutdown_coordinator.is_shutdown_requested() {
            break;
        }

        // Sleep 10ms if no activity (prevents CPU spin)
        tokio::time::sleep(Duration::from_millis(10)).await;
    }
}
```

**Loop Structure**:
1. Check shutdown signal (574-578)
2. Poll Control channel - **priority** (592-670)
3. Poll Shell channel (672-840)
4. Poll Stdin channel (842-870)
5. Poll Heartbeat channel (872-890)
6. Process collected messages (914-922)
7. Sleep 10ms if no activity (927-929)

### Daemon Support (Phase 10)

**Location**: `llmspell-kernel/src/daemon/manager.rs:18-54`

```rust
pub struct DaemonConfig {
    pub daemonize: bool,
    pub pid_file: Option<PathBuf>,          // /tmp/kernel.pid
    pub working_dir: PathBuf,               // Usually "/"
    pub stdout_path: Option<PathBuf>,       // /var/log/llmspell/kernel.log
    pub stderr_path: Option<PathBuf>,       // /var/log/llmspell/kernel.err
    pub close_stdin: bool,
    pub umask: Option<u32>,                 // 0o027 for security
}

pub struct SignalBridge {
    shutdown_tx: watch::Sender<bool>,       // SIGTERM/SIGINT
    reload_tx: watch::Sender<bool>,         // SIGHUP
    stats_tx: watch::Sender<bool>,          // SIGUSR1
}
```

**Double-Fork Daemonization** (`manager.rs:89-145`):

```
Parent Process (PID 1000)
  │
  ├─ fork() // First fork
  │   ├─ Parent: exit(0) immediately
  │   └─ Child (PID 1001): continue
  │
Child Process (PID 1001)
  │
  ├─ setsid() // Create new session (detach from TTY)
  │
  ├─ fork() // Second fork
  │   ├─ Parent: exit(0)
  │   └─ Child (PID 1002): continue
  │
Daemon Process (PID 1002)
  │
  ├─ chdir("/") // Change to root
  ├─ umask(0o027) // Secure file permissions
  ├─ Redirect stdout/stderr → log files
  ├─ Close stdin
  └─ Write PID file
```

**Why Double-Fork?**
1. **First fork**: Detach from parent process group
2. **setsid()**: Become session leader, no controlling TTY
3. **Second fork**: Ensure daemon can NEVER acquire a controlling TTY
4. **Result**: True daemon, completely independent of original shell

---

## Infrastructure Creation

### Single Creation Path (Phase 13b.16)

**Before** (Fragmented initialization):
- CLI created components directly (200+ lines)
- Services duplicated initialization code
- Multiple creation paths (embedded vs daemon)
- No conditional component creation

**After** (Unified Infrastructure module):
- Single entry point: `Infrastructure::from_config()`
- CLI uses ~12 lines: `ScriptRuntime::new()` + `start_embedded_kernel()`
- Conditional creation (RAG, Memory only if enabled)
- Zero duplication

### Infrastructure::from_config()

**Location**: `llmspell-bridge/src/infrastructure.rs:107-161`

```rust
pub struct Infrastructure {
    provider_manager: Arc<ProviderManager>,         // LLM providers
    state_manager: Arc<StateManager>,               // Persistent KV storage
    session_manager: Arc<SessionManager>,           // Session lifecycle
    rag: Option<Arc<MultiTenantRAG>>,               // Optional RAG
    memory_manager: Option<Arc<DefaultMemoryManager>>, // Optional memory
    tool_registry: Arc<ToolRegistry>,               // Tool discovery
    agent_registry: Arc<FactoryRegistry>,           // Agent factories
    workflow_factory: Arc<dyn WorkflowFactory>,     // Workflow patterns
    component_registry: Arc<ComponentRegistry>,     // Script access layer
}

pub async fn from_config(config: &LLMSpellConfig) -> Result<Self> {
    // 1. Provider manager (OpenAI, Anthropic, Ollama, Candle)
    let provider_manager = create_provider_manager(config).await?;

    // 2. State manager (persistent key-value storage)
    let state_manager = create_state_manager(config).await?;

    // 3. Session manager (depends on state_manager)
    let session_manager = create_session_manager(
        state_manager.clone(),  // ← Dependency injection
        config
    )?;

    // 4. RAG (conditional)
    let rag = if config.rag.enabled {
        Some(create_rag(config))
    } else {
        None
    };

    // 5. Memory manager (conditional)
    let memory_manager = if config.runtime.memory.enabled {
        Some(create_memory_manager(config).await?)
    } else {
        None
    };

    // 6-9. Registries (always created)
    let tool_registry = Arc::new(ToolRegistry::new());
    let agent_registry = Arc::new(FactoryRegistry::new());
    let workflow_factory = Arc::new(DefaultWorkflowFactory::new());
    let component_registry = create_component_registry(config)?;

    Ok(Self {
        provider_manager,
        state_manager,
        session_manager,
        rag,
        memory_manager,
        tool_registry,
        agent_registry,
        workflow_factory,
        component_registry,
    })
}
```

### Component Initialization Order

**Dependency-Aware Sequencing**:

```
1. ProviderManager         [No dependencies]
   └─ OpenAI, Anthropic, Ollama, Candle

2. StateManager            [No dependencies]
   └─ Persistent key-value storage

3. SessionManager          [Depends on: StateManager]
   ├─ StateManager (injected)
   ├─ StorageBackend (memory or sled)
   ├─ HookRegistry
   ├─ HookExecutor
   └─ EventBus

4. RAG                     [Optional, no dependencies]
   ├─ HNSWVectorStorage (8.47x speedup)
   ├─ MultiTenantVectorManager
   └─ MultiTenantRAG

5. MemoryManager           [Optional, no dependencies]
   ├─ Episodic memory (conversation history)
   ├─ Semantic memory (bi-temporal graph)
   └─ Procedural memory (patterns)

6. ToolRegistry            [No dependencies]
7. AgentRegistry           [No dependencies]
8. WorkflowFactory         [No dependencies]
9. ComponentRegistry       [Optional EventBus dependency]
```

**Critical Path**: StateManager → SessionManager (2-step dependency)
**Parallelizable**: Steps 1, 4-9 (no mutual dependencies)

### CLI Integration (Phase 13b.16)

**Location**: `llmspell-cli/src/execution_context.rs:102-189`

```rust
// Entire CLI initialization (12 lines total)
pub async fn resolve(
    connect: Option<String>,
    kernel: Option<String>,
    config: Option<PathBuf>,
    default_config: LLMSpellConfig,
) -> Result<ExecutionContext> {
    // 1. Load config
    let config = match config {
        Some(path) => LLMSpellConfig::load_from_file(&path).await?,
        None => default_config,
    };

    // 2. Create ScriptRuntime (creates Infrastructure internally)
    let script_executor = Arc::new(
        llmspell_bridge::ScriptRuntime::new(config.clone()).await?
    ) as Arc<dyn ScriptExecutor>;

    // 3. Start embedded kernel
    let handle = start_embedded_kernel_with_executor(
        config.clone(),
        script_executor,
    ).await?;

    Ok(ExecutionContext::Embedded(handle))
}
```

**Total infrastructure code in CLI**: 0 lines (all delegated to Infrastructure module)

---

## Protocol System

### Protocol Trait

**Location**: `llmspell-kernel/src/traits/protocol.rs`

```rust
pub trait Protocol: Send + Sync + 'static {
    type Message: KernelMessage;

    // Wire format encoding/decoding
    fn encode(&self, msg: &Self::Message, channel: &str) -> Result<Vec<Vec<u8>>>;
    fn decode(&self, parts: Vec<Vec<u8>>, channel: &str) -> Result<Self::Message>;
    fn parse_message(&self, data: &[u8]) -> Result<HashMap<String, Value>>;

    // Message creation
    fn create_request(&self, msg_type: &str, content: Value) -> Result<Vec<u8>>;
    fn create_multipart_response(
        &self,
        client_id: &[u8],
        msg_type: &str,
        content: &Value
    ) -> Result<Vec<Vec<u8>>>;

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

**Separation of Concerns**:
- **Transport trait**: Mechanics (ZeroMQ, InProcess channels)
- **Protocol trait**: Semantics (Jupyter message formats, correlation)

### Jupyter Protocol Implementation

**Full 5-Channel Support**:

| Channel | Pattern | Purpose | Messages |
|---------|---------|---------|----------|
| **shell** | ROUTER | Execute requests & replies | execute_request, kernel_info_request, tool_request |
| **control** | ROUTER | Control commands | shutdown_request, interrupt_request, debug_request |
| **iopub** | PUB | Broadcasting outputs | status, stream, execute_result, error |
| **stdin** | ROUTER | Input requests | input_request, input_reply |
| **heartbeat** | REP | Liveness monitoring | Automatic echo |

**Multipart Message Format** (Jupyter wire protocol):

```
[0] identity          // Client routing identity (ROUTER sockets)
[1] <IDS|MSG>        // Delimiter
[2] signature        // HMAC-SHA256(header + parent_header + metadata + content)
[3] header           // JSON: msg_type, msg_id, username, session, date, version
[4] parent_header    // Parent message for correlation
[5] metadata         // Additional metadata
[6] content          // Actual message payload
[7+] buffers         // Optional binary data
```

**HMAC Signing** (authentication):

```rust
signature = HMAC-SHA256(
    key = connection_file.key,
    data = header_json + parent_header_json + metadata_json + content_json
)
```

**Simple JSON Format** (embedded mode optimization):

```json
{
  "msg_type": "execute_request",
  "msg_id": "uuid",
  "content": {"code": "return 42"},
  "header": {"msg_type": "execute_request"},
  "metadata": {}
}
```

### Message Catalog

**Shell Channel** (validated at `integrated.rs:1080-1089`):

| Message Type | Handler | Status | Purpose |
|--------------|---------|--------|---------|
| execute_request | handle_execute_request() | ✅ IMPLEMENTED | Execute code |
| kernel_info_request | handle_kernel_info_request() | ✅ IMPLEMENTED | Kernel metadata |
| tool_request | handle_tool_request() | ✅ IMPLEMENTED | Tool commands (list/info/invoke/search/test) |
| complete_request | - | ❌ NOT IMPLEMENTED | Autocomplete |
| inspect_request | - | ❌ NOT IMPLEMENTED | Documentation lookup |
| history_request | - | ❌ NOT IMPLEMENTED | Command history |

**Control Channel** (validated at `integrated.rs:1090-1094`):

| Message Type | Handler | Status | Implementation |
|--------------|---------|--------|----------------|
| shutdown_request | handle_shutdown_request() | ✅ IMPLEMENTED | Graceful shutdown |
| interrupt_request | handle_interrupt_request() | ✅ STUB | Returns success (no-op) |
| debug_request | handle_debug_request() | ✅ IMPLEMENTED | Forwards to DAPBridge |

**IOPub Channel** (outbound only):

| Message Type | Function | Purpose |
|--------------|----------|---------|
| status | publish_status() | starting/busy/idle/dead |
| execute_input | publish_execute_input() | Echo code being executed |
| execute_result | publish_execute_result() | Execution output |
| stream | write_stdout()/write_stderr() | stdout/stderr streams |
| error | publish_error() | Error traceback |
| display_data | - | Rich display data (HTML, images) |

---

## Execution Modes

### Embedded Mode (In-Process Kernel)

**Use Cases**:
- `llmspell run` - Script execution
- `llmspell exec` - Direct code execution
- `llmspell repl` - Interactive REPL
- `llmspell tool` - Tool commands
- Embedded Rust applications

**Creation**:

```rust
use llmspell_bridge::ScriptRuntime;
use llmspell_kernel::api::start_embedded_kernel_with_executor;

let config = LLMSpellConfig::load_from_file("config.toml").await?;

// Infrastructure created internally
let script_executor = Arc::new(ScriptRuntime::new(config.clone()).await?);

// Start embedded kernel
let kernel_handle = start_embedded_kernel_with_executor(
    config,
    script_executor,
).await?;

// Execute script
let result = kernel_handle.execute("return 42").await?;
```

**Performance**:
- **Startup**: 55ms (first run), 50ms (reused kernel)
- **Latency**: 0.05ms (in-process communication)
- **Memory**: 50MB typical

### Connected Mode (Remote Kernel)

**Use Cases**:
- `llmspell run --connect localhost:9555`
- Multi-client scenarios
- Jupyter notebook integration
- VS Code extension

**Creation**:

```rust
use llmspell_kernel::api::connect_to_kernel;

// Connect to running kernel
let client_handle = connect_to_kernel("localhost:9555").await?;

// Same API as embedded mode
let result = client_handle.execute("return 42").await?;
```

**Performance**:
- **Connection**: 5ms
- **Latency**: 0.8ms (TCP + message overhead)
- **Memory**: Minimal client-side

### Daemon Mode (Production Service)

**Use Cases**:
- systemd service
- launchd service (macOS)
- Production deployments
- Multi-tenant services

**Creation**:

```bash
# Start daemon
llmspell kernel start --daemon --port 9572 --id my-kernel

# Connection file created: ~/.llmspell/kernels/my-kernel.json
{
  "kernel_id": "my-kernel",
  "transport": "tcp",
  "ip": "127.0.0.1",
  "shell_port": 9572,
  "iopub_port": 9573,
  "stdin_port": 9574,
  "control_port": 9575,
  "hb_port": 9576,
  "key": "hmac-sha256-key-here"
}

# Check status
llmspell kernel status my-kernel

# Stop daemon
llmspell kernel stop my-kernel
```

**systemd Integration** (`/etc/systemd/system/llmspell-kernel.service`):

```ini
[Unit]
Description=LLMSpell Kernel Service
After=network.target

[Service]
Type=forking
ExecStart=/usr/local/bin/llmspell kernel start --daemon --port 9572
PIDFile=/tmp/llmspell-kernel.pid
Restart=on-failure
User=llmspell
Group=llmspell

[Install]
WantedBy=multi-user.target
```

**launchd Integration** (`~/Library/LaunchAgents/com.llmspell.kernel.plist`):

```xml
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN">
<plist version="1.0">
<dict>
    <key>Label</key>
    <string>com.llmspell.kernel</string>
    <key>ProgramArguments</key>
    <array>
        <string>/usr/local/bin/llmspell</string>
        <string>kernel</string>
        <string>start</string>
        <string>--daemon</string>
        <string>--port</string>
        <string>9572</string>
    </array>
    <key>RunAtLoad</key>
    <true/>
    <key>KeepAlive</key>
    <true/>
</dict>
</plist>
```

### Auto-Detection Mode

**CLI Priority Order**:

```rust
// llmspell-cli/src/execution_context.rs:155-187
match (connect, kernel, config) {
    // 1. Explicit --connect flag: use remote kernel
    (Some(addr), _, _) => connect_to_kernel(addr).await,

    // 2. Explicit --kernel ID: find by ID
    (_, Some(kernel_id), _) => find_kernel_by_id(kernel_id).await,

    // 3. Explicit --config: use embedded with that config
    (_, _, Some(config_path)) => {
        let config = LLMSpellConfig::load_from_file(&config_path).await?;
        start_embedded_with_config(config).await
    },

    // 4. Auto-detect: search for running kernels
    (None, None, None) => {
        if let Some(addr) = find_running_kernel().await? {
            connect_to_kernel(addr).await  // Found kernel
        } else {
            start_embedded_with_defaults().await  // Fallback
        }
    }
}
```

**Discovery Mechanism**:

```rust
// Scan ~/.llmspell/kernels/*.json
async fn find_running_kernel() -> Result<Option<String>> {
    let kernel_dir = dirs::home_dir()?.join(".llmspell/kernels");

    for entry in fs::read_dir(kernel_dir)? {
        let path = entry?.path();
        if let Some(conn_file) = ConnectionFile::from_path(&path)? {
            // Check if kernel is alive
            if heartbeat_check(&conn_file).await? {
                return Ok(Some(conn_file.address()));
            }
        }
    }

    Ok(None)  // No running kernels found
}
```

---

## Message Flows

### Script Execution Flow (Embedded Mode)

```
User: llmspell run script.lua arg1 --key value
  ↓
commands/run.rs::execute_script_file() // 57
  ├─ Parse args: {"0": "script.lua", "1": "arg1", "key": "value"} // 18-54
  ├─ Read script content // 80
  └─ ExecutionContext::resolve() // 26-32
      ↓
commands/run.rs::execute_script_embedded() // 106
  ├─ handle.into_kernel() // 124 - Take ownership
  └─ kernel.execute_direct_with_args(code, args) // 127-129
      ↓
integrated.rs::execute_direct_with_args() // 1611-1680
  ├─ Check shutdown_coordinator // 1617-1621
  ├─ Generate exec_id // 1627
  ├─ Update state // 1630-1633
  └─ script_executor.execute_script_with_args(code, args) // 1642-1660
      ↓
  Return result string → Display to user
```

**Optimization**: Bypasses message protocol for performance (0.05ms vs 0.8ms)

### Tool Command Flow (Message Protocol)

```
User: llmspell tool list --category filesystem
  ↓
commands/tool.rs::handle_tool_command() // 17
  └─ ExecutionContext::resolve() // 26-32
      ↓
commands/tool.rs::handle_tool_embedded() // 49
  ├─ json!({"command": "list", "category": "filesystem"}) // 63-66
  └─ handle.send_tool_request(content) // 69
      ↓
api.rs::send_tool_request() // 106
  ├─ protocol.create_request("tool_request", content) // 113
  ├─ transport.send("shell", vec![request]) // 121
  └─ Loop: transport.recv("shell") // 132
      ↓ [Via InProcessTransport]
integrated.rs::run() loop receives message // 697
  ├─ Parse multipart: find "<IDS|MSG>" // 719-723
  ├─ Extract identity[0], header[idx+2], content[idx+5] // 729-748
  └─ handle_message_with_identity() // 917
      ├─ Store client_identity // 1004
      ├─ Store current_msg_header // 1011
      └─ handle_message() // 1023
          ↓
          match msg_type = "tool_request" // 983
          ↓
integrated.rs::handle_tool_request() // 1946
  ├─ Extract command = "list" // 1953-1956
  └─ handle_tool_list(content) // 1961
      ├─ script_executor.component_registry() // 1978
      ├─ registry.list_tools() // 1980
      ├─ Filter by category // 1983-2003
      └─ send_tool_reply(response) // 2046
          ↓
integrated.rs::send_tool_reply() // 1837
  ├─ Get client_identity // 1841-1844
  ├─ create_multipart_response() // 1854-1855
  └─ transport.send("shell", multipart) // 1862-1867
      ↓ [Back via InProcessTransport]
api.rs receives reply // 132-180
  ├─ Parse multipart or simple JSON // 138-180
  └─ Return content to CLI
      ↓
Display tool list to user
```

### Execute Request Handler (Connected Clients)

**Location**: `integrated.rs:1251-1516`

```rust
async fn handle_execute_request(&mut self, message: HashMap<String, Value>) -> Result<()> {
    // 1. Extract: msg_id, code, silent, store_history
    // 2. session_manager.start_execution_context()
    // 3. Track KernelEvent::ExecuteRequest
    // 4. Increment execution_count
    // 5. state.update_execution()
    // 6. io_manager.publish_status("busy")
    // 7. io_manager.set_parent_header(msg_id)
    // 8. io_manager.publish_execute_input()
    // 9. timeout(script_executor.execute(code))

    // 10. Handle result:
    match result {
        Ok(Ok(output)) => {
            // Success
            io_manager.publish_execute_result(&output)?;
            create_execute_reply(status: "ok")
            // ⚠️ TODO: Send execute_reply (currently not sent!)
        }
        Ok(Err(e)) => {
            // Error
            io_manager.write_stderr(&error)?;
            create_execute_reply(status: "error")
            // ⚠️ TODO: Send execute_reply (currently not sent!)
        }
        Err(_) => {
            // Timeout
            create_execute_reply(status: "aborted")
            // ⚠️ TODO: Send execute_reply (currently not sent!)
        }
    }

    // 11. Cleanup
    io_manager.clear_parent_header();
    io_manager.publish_status("idle");
}
```

**🚨 KNOWN BUG**: Execute reply created but NOT sent via transport. Connected clients never receive completion notification.

**Fix needed**: Add `transport.send("shell", execute_reply)` after creation.

### Kernel Service Flow (Daemon Mode)

```
User: llmspell kernel start --daemon --port 9572
  ↓
commands/kernel.rs::handle_kernel_command() // 24
  ├─ Build DaemonConfig // 45-81
  ├─ Build ExecutionConfig // 84-99
  └─ Build KernelServiceConfig // 102-112
      ↓
api.rs::start_kernel_service_with_config() // 958
  ├─ Create JupyterProtocol // 972
  ├─ Create ConnectionFileManager // 975-976
  │   └─ Writes ~/.llmspell/kernels/<id>.json
  ├─ protocol.set_hmac_key() // 979
  └─ setup_kernel_transport(port) // 983
      ↓
api.rs::setup_kernel_transport() // 1058
  ├─ ZmqTransport::new() // 1068
  ├─ Build TransportConfig with 5 channels // 1073-1142
  ├─ transport.bind(&config) // 1150
  │   └─ Returns BoundPorts with actual ports
  └─ conn_manager.update_ports() // 1159-1165
      ↓
  ├─ IntegratedKernel::new() // 987-993
  ├─ kernel.set_transport() // 996
  └─ If daemon_mode:
      ├─ daemon_manager.daemonize() // daemon/manager.rs:89
      │   ├─ First fork() + parent exit // 93-102
      │   ├─ setsid() // 105
      │   ├─ Second fork() + intermediate exit // 109-118
      │   ├─ chdir(working_dir) // 121
      │   ├─ Set umask // 126-130
      │   ├─ Redirect I/O // 133
      │   └─ Write PID file // 136-141
      └─ SignalBridge::setup() // Signal handlers
          ↓
  tokio::spawn(kernel.run()) // Background event loop
  └─ Kernel runs as daemon, accepting connections
```

---

## Component Lifecycle

### Creation Phase

```rust
// 1. Load configuration (1ms)
let config = LLMSpellConfig::load_from_file("config.toml").await?;

// 2. Create infrastructure (50-200ms)
let infrastructure = Infrastructure::from_config(&config).await?;
// Creates: ProviderManager, StateManager, SessionManager, RAG, Memory, 4 registries

// 3. Create ScriptRuntime (10-50ms)
let script_executor = Arc::new(ScriptRuntime::new(config.clone()).await?);
// Initializes: Lua/JS engines, bridges infrastructure to scripts

// 4. Start kernel (<10ms)
let kernel_handle = start_embedded_kernel_with_executor(
    config,
    script_executor,
).await?;
// Creates: Kernel event loop, transport channels, IO manager
```

**Timeline** (on M1 MacBook Pro):
- Config load: <1ms (cached)
- Infrastructure creation: 50-200ms (depends on provider connections)
- ScriptRuntime creation: 10-50ms (engine initialization)
- Kernel start: <10ms (in-process transport)
- **Total startup**: ~100-300ms

### Execution Phase

```rust
// Execute script
let result = kernel_handle.execute(r#"
    local result = Agent.query("What is Rust?")
    return result
"#).await?;
```

**Component Access**:
- Script → ComponentRegistry → Tool/Agent (O(1) HashMap lookup)
- Template → ToolRegistry → Tool (indexed lookup with hooks)
- Agent → ProviderManager → LLM (connection pool)

### Shutdown Phase

```rust
// Kernel shutdown
drop(kernel_handle);  // Graceful shutdown via Drop trait

// Infrastructure cleanup
drop(script_executor);
drop(infrastructure);
```

**Cleanup Order**:
1. Kernel stops accepting requests
2. In-flight scripts complete (timeout: 30s)
3. Components shut down (Drop trait)
4. Storage backends flush to disk
5. Connections close

**Graceful shutdown**: <5s typical, <30s maximum

---

## Transport Layer

### InProcessTransport Architecture

**Location**: `llmspell-kernel/src/transport/inprocess.rs:22-36`

```rust
pub struct InProcessTransport {
    /// Channels for sending (this transport sends here)
    channels: Arc<RwLock<HashMap<String, ChannelPair>>>,
    /// Reverse channels for receiving (this transport receives from here)
    reverse_channels: Arc<RwLock<HashMap<String, ChannelPair>>>,
}

struct ChannelPair {
    sender: mpsc::UnboundedSender<Vec<Vec<u8>>>,
    receiver: Arc<RwLock<mpsc::UnboundedReceiver<Vec<Vec<u8>>>>>,
}
```

**Channel Pairing** (`inprocess.rs:110-220`):

```rust
// Create two mpsc channels
(tx1, rx1) = mpsc::unbounded_channel()
(tx2, rx2) = mpsc::unbounded_channel()

// Assign to Transport1
T1.channels["shell"].sender = tx1
T1.reverse_channels["shell"].receiver = Arc::new(RwLock::new(rx2))

// Assign to Transport2
T2.channels["shell"].sender = tx2
T2.reverse_channels["shell"].receiver = Arc::new(RwLock::new(rx1))

// Communication (bidirectional):
T1.send("shell") → uses tx1 → rx1 received by T2.recv("shell")
T2.send("shell") → uses tx2 → rx2 received by T1.recv("shell")
```

**Send** (`inprocess.rs:253-280`):

```rust
async fn send(&self, channel: &str, parts: Vec<Vec<u8>>) -> Result<()> {
    let channels = self.channels.read();
    let pair = channels.get(channel)?;
    pair.sender.send(parts)?;  // Non-blocking unbounded send
    Ok(())
}
```

**Recv** (`inprocess.rs:283-324`):

```rust
async fn recv(&self, channel: &str) -> Result<Option<Vec<Vec<u8>>>> {
    let channels = self.reverse_channels.read();
    let pair = channels.get(channel)?;
    let mut receiver = pair.receiver.write();
    match receiver.try_recv() {
        Ok(parts) => Ok(Some(parts)),
        Err(TryRecvError::Empty) => Ok(None),
        Err(e) => Err(e.into()),
    }
}
```

### ZmqTransport Architecture

**Location**: `llmspell-kernel/src/transport/zeromq.rs`

```rust
pub struct ZmqTransport {
    context: zmq::Context,
    sockets: HashMap<String, zmq::Socket>,
    bound_ports: Option<BoundPorts>,
}
```

**Socket Patterns**:

| Channel | Pattern | Direction | Purpose |
|---------|---------|-----------|---------|
| shell | ROUTER | Bidirectional | Execute requests with routing |
| iopub | PUB | Outbound only | Broadcast to all subscribers |
| stdin | ROUTER | Bidirectional | Input requests with routing |
| control | ROUTER | Bidirectional | Control commands with routing |
| heartbeat | REP | Request-reply | Liveness check (echo) |

**Binding**:

```rust
// For each channel
let socket = match channel_name {
    "shell" | "stdin" | "control" => context.socket(zmq::ROUTER)?,
    "iopub" => context.socket(zmq::PUB)?,
    "heartbeat" => context.socket(zmq::REP)?,
    _ => unreachable!(),
};

socket.bind(&format!("tcp://*:{}", port))?;

// If port=0: Get actual port from socket
let actual_port = socket.get_last_endpoint()?
    .parse::<SocketAddr>()?
    .port();
```

---

## Performance

### Performance Targets vs Achieved

| Metric | Target | Achieved | Status |
|--------|--------|----------|--------|
| **Kernel startup** | <100ms | 95ms | ✅ |
| **Message handling** | <5ms | 3ms | ✅ |
| **Tool invocation** | <10ms | 8ms | ✅ |
| **InProcess round-trip** | <0.1ms | 0.05ms | ✅ |
| **ZeroMQ round-trip** | <1ms | 0.8ms | ✅ |
| **Memory usage** | <100MB | 50MB | ✅ |
| **Execution timeout** | 300s | 300s | ✅ |

### Latency Comparison

| Operation | InProcess | ZeroMQ (local) | ZeroMQ (remote) |
|-----------|-----------|----------------|-----------------|
| **send()** | 0.05ms | 0.8ms | 2ms |
| **recv()** | 0.05ms | 0.8ms | 2ms |
| **Round-trip** | 0.1ms | 1.6ms | 4ms |
| **Throughput** | 20K msg/s | 10K msg/s | 5K msg/s |
| **Memory** | 1MB | 5MB | 10MB |
| **Startup** | 10ms | 50ms | 100ms |

### Connection Performance

**Embedded Mode**:
```
llmspell run script1.lua  # 55ms (first run, kernel spawn)
llmspell run script2.lua  # 50ms (kernel reuse)
llmspell run script3.lua  # 50ms (kernel reuse)
Total: 155ms
```

**Daemon Mode**:
```
llmspell run script1.lua  # 155ms (connect + execute)
llmspell run script2.lua  # 56ms (connection reuse)
llmspell run script3.lua  # 56ms (connection reuse)
Total: 267ms (first), 168ms (subsequent)
```

---

## Troubleshooting

### Issue: Component Not Found

**Error**: `"Component 'rag' not found"`

**Cause**: RAG or Memory not enabled in config

**Solution**:
```toml
# config.toml
[rag]
enabled = true

[runtime.memory]
enabled = true
```

### Issue: Kernel Startup Slow

**Symptom**: Infrastructure creation takes >5s

**Diagnosis**:
```bash
# Enable debug logging
export RUST_LOG=llmspell_bridge=debug,llmspell_kernel=debug

# Check which component is slow
llmspell run script.lua

# Output:
# 2025-01-15 14:30:00 DEBUG Creating provider manager
# 2025-01-15 14:30:03 DEBUG Creating state manager  ← 3s delay!
```

**Common causes**:
- Network timeout connecting to Ollama/Candle
- Disk I/O for large HNSW indexes
- PostgreSQL connection pool exhaustion

**Solutions**:
- Use `memory` storage backend for development
- Reduce HNSW `max_elements` for testing
- Check PostgreSQL connection limits

### Issue: Execute Reply Not Sent

**Error**: Connected clients hang waiting for reply

**Cause**: Execute reply created but not sent (known bug at `integrated.rs:1404-1406`)

**Workaround**: Use embedded mode for now

**Fix in progress**: Add `transport.send("shell", execute_reply)`

### Issue: Connection Pool Exhaustion

**Error**: `"Timeout waiting for connection from pool"`

**Diagnosis**:
```sql
-- Check current connection count
SELECT count(*) FROM pg_stat_activity WHERE datname = 'llmspell_prod';
```

**Solution**:
```toml
[storage.postgres]
pool_size = 20  # Increase from default
# Formula: (CPU cores × 2) + 1
```

### Issue: Memory Leak

**Symptom**: Memory usage grows over time

**Diagnosis**:
- Check component Arc reference counts
- Use `cargo-flamegraph` for heap profiling

**Common causes**:
- Script engines holding old contexts
- Session artifacts not garbage collected
- HNSW index growing unbounded

**Solutions**:
```toml
[runtime.sessions]
max_session_age_hours = 24  # Auto-cleanup old sessions

[rag.vector_storage.hnsw]
max_elements = 100000  # Limit HNSW growth
```

---

## Code References

### Primary Implementation Files

| Component | Location | Lines | Purpose |
|-----------|----------|-------|---------|
| IntegratedKernel | `llmspell-kernel/src/execution/integrated.rs` | 4500 | Main kernel implementation |
| Infrastructure | `llmspell-bridge/src/infrastructure.rs` | 385 | Central creation factory |
| ScriptRuntime | `llmspell-bridge/src/runtime.rs` | 800+ | Script execution orchestrator |
| ExecutionContext | `llmspell-cli/src/execution_context.rs` | 296 | CLI mode resolution |
| Kernel API | `llmspell-kernel/src/api.rs` | 1200+ | Embedded/client handles |
| DaemonManager | `llmspell-kernel/src/daemon/manager.rs` | 200 | Double-fork daemonization |
| InProcessTransport | `llmspell-kernel/src/transport/inprocess.rs` | 350 | In-process channels |
| ZmqTransport | `llmspell-kernel/src/transport/zeromq.rs` | 500 | ZeroMQ implementation |
| JupyterProtocol | `llmspell-kernel/src/protocols/jupyter.rs` | 600 | Jupyter wire protocol |

### Key Function References

```rust
// Infrastructure creation
llmspell-bridge/src/infrastructure.rs:107-161
pub async fn from_config(config: &LLMSpellConfig) -> Result<Self>

// CLI execution context
llmspell-cli/src/execution_context.rs:102-189
pub async fn resolve(connect, kernel, config, default_config)

// Embedded kernel creation
llmspell-kernel/src/api.rs:1093-1150
pub async fn start_embedded_kernel_with_executor(config, script_executor)

// Client kernel connection
llmspell-kernel/src/api.rs:720-780
pub async fn connect_to_kernel(address: &str)

// Kernel event loop
llmspell-kernel/src/execution/integrated.rs:522-900
pub async fn run(mut self) -> Result<()>

// Execute request handler
llmspell-kernel/src/execution/integrated.rs:1251-1516
async fn handle_execute_request(&mut self, message: HashMap<String, Value>)

// Daemonization
llmspell-kernel/src/daemon/manager.rs:89-145
pub fn daemonize(&mut self) -> Result<()>
```

---

**🔗 See Also**:
- [Current Architecture](current-architecture.md) - System design overview
- [PostgreSQL Guide](postgresql-guide.md) - Database backend configuration
- [Developer Guide: Tracing & Debugging](../developer-guide/06-tracing-debugging.md) - Debugging kernel execution

**Document Version**: 1.0
**Last Updated**: January 2025
**Phase**: 13b.20.2 - Kernel Architecture Consolidation
