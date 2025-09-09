# llmspell-kernel

**Jupyter-compatible execution kernel for LLMSpell** ⭐ **NEW Phase 9**

[← Back to Rust API](README.md) | [Crates.io](https://crates.io/crates/llmspell-kernel) | [API Docs](https://docs.rs/llmspell-kernel)

---

## Overview

The `llmspell-kernel` crate provides the core execution engine that manages script execution, client connections, and debug integration. This crate represents the foundation of Phase 9's kernel architecture, replacing the deprecated `llmspell-engine` with a clean, trait-based design that implements the industry-standard Jupyter Messaging Protocol.

**Key Features:**
- **Trait-based Architecture**: `GenericKernel<T: Transport, P: Protocol>` for extensibility
- **Jupyter Protocol**: Full implementation of Jupyter Messaging Protocol v5.3
- **EmbeddedKernel**: Runs in background thread, not standalone process
- **ZeroMQ Transport**: High-performance message passing with connection reuse
- **DAP Bridge**: 10 essential Debug Adapter Protocol commands for IDE integration
- **Session Persistence**: State maintained across executions
- **Multi-client Support**: Handle multiple concurrent client connections
- **Security Management**: Authentication and resource limits per client

## Architecture

### Trait-based Design

The kernel uses a three-layer trait architecture for clean separation:

```
Kernel Layer (GenericKernel)
    ↓
Protocol Layer (Protocol trait)
    ↓  
Transport Layer (Transport trait)
```

- **Transport**: Knows nothing about protocols, just moves bytes
- **Protocol**: Knows nothing about kernel, handles message encoding/decoding
- **Kernel**: Orchestrates execution using protocol and transport

### Core Components

```rust
// Generic kernel works with any Transport and Protocol
pub struct GenericKernel<T: Transport, P: Protocol> {
    pub kernel_id: String,
    transport: T,
    protocol: P,
    pub runtime: Arc<Mutex<ScriptRuntime>>,
    pub client_manager: Arc<ClientManager>,
    pub execution_state: Arc<RwLock<KernelState>>,
    pub state_manager: Option<Arc<StateManager>>,
    pub security_manager: Arc<SecurityManager>,
    pub session_mapper: Arc<SessionMapper>,
    // ... other fields
}

// Type alias for common Jupyter configuration
pub type JupyterKernel = GenericKernel<ZmqTransport, JupyterProtocol>;
```

## Core Traits

### Transport Trait

Abstracts message transport (ZeroMQ, TCP, IPC, etc.):

```rust
#[async_trait]
pub trait Transport: Send + Sync {
    /// Bind to addresses (server mode)
    async fn bind(&mut self, config: &TransportConfig) -> Result<()>;
    
    /// Connect to addresses (client mode)
    async fn connect(&mut self, config: &TransportConfig) -> Result<()>;
    
    /// Receive multipart message
    async fn recv(&self, channel: &str) -> Result<Option<Vec<Vec<u8>>>>;
    
    /// Send multipart message
    async fn send(&self, channel: &str, parts: Vec<Vec<u8>>) -> Result<()>;
    
    /// Handle heartbeat if required
    async fn heartbeat(&self) -> Result<bool>;
    
    /// Check if channel exists
    fn has_channel(&self, channel: &str) -> bool;
    
    /// Get available channels
    fn channels(&self) -> Vec<String>;
}
```

### Protocol Trait

Abstracts protocol handling (Jupyter, LSP, DAP, etc.):

```rust
#[async_trait]
pub trait Protocol: Send + Sync {
    type Message: KernelMessage;
    
    /// Protocol name
    fn name(&self) -> &str;
    
    /// Get transport configuration
    fn transport_config(&self) -> TransportConfig;
    
    /// Decode wire format to message
    fn decode(&self, parts: Vec<Vec<u8>>) -> Result<Self::Message>;
    
    /// Encode message to wire format
    fn encode(&self, msg: &Self::Message, channel: &str) -> Result<Vec<Vec<u8>>>;
    
    /// Create response message
    fn create_response(
        &self,
        msg_type: &str,
        content: serde_json::Value,
        parent: Option<&Self::Message>,
    ) -> Result<Self::Message>;
    
    /// Get execution flow for protocol
    fn execution_flow(&self, msg_type: &str) -> ExecutionFlow<Self::Message>;
    
    /// Get expected response flow
    fn response_flow(&self, msg_type: &str) -> ResponseFlow;
    
    /// Handle protocol-specific output
    async fn handle_output(&self, chunk: OutputChunk) -> Result<Vec<Self::Message>>;
}
```

### KernelMessage Trait

Abstracts message representation:

```rust
pub trait KernelMessage: Send + Sync + Clone {
    /// Get message type
    fn msg_type(&self) -> &str;
    
    /// Get message ID
    fn msg_id(&self) -> &str;
    
    /// Get session ID
    fn session_id(&self) -> &str;
    
    /// Get content as JSON
    fn content(&self) -> &serde_json::Value;
    
    /// Set parent message for reply chain
    fn set_parent(&mut self, parent: &Self);
    
    /// Create from components
    fn from_parts(
        msg_type: String,
        content: serde_json::Value,
        session: String,
    ) -> Self;
}
```

## Jupyter Implementation

### Message Types

The crate implements all standard Jupyter message types:

```rust
pub enum MessageContent {
    // Kernel lifecycle
    KernelInfoRequest {},
    KernelInfoReply { /* fields */ },
    ShutdownRequest { restart: bool },
    ShutdownReply { /* fields */ },
    
    // Code execution
    ExecuteRequest { code: String, silent: bool, /* ... */ },
    ExecuteReply { status: ExecutionStatus, /* ... */ },
    ExecuteResult { data: HashMap<String, Value>, /* ... */ },
    
    // Output streams
    Stream { name: StreamType, text: String },
    DisplayData { data: HashMap<String, Value>, /* ... */ },
    Error { ename: String, evalue: String, traceback: Vec<String> },
    
    // Introspection
    InspectRequest { code: String, cursor_pos: usize, /* ... */ },
    InspectReply { /* fields */ },
    CompleteRequest { code: String, cursor_pos: usize },
    CompleteReply { matches: Vec<String>, /* ... */ },
    
    // Input/output
    InputRequest { prompt: String, password: bool },
    InputReply { value: String },
    
    // Comm channels for session management
    CommOpen { comm_id: String, target_name: String, /* ... */ },
    CommMsg { comm_id: String, data: Value },
    CommClose { comm_id: String, data: Value },
    
    // LLMSpell extensions
    DapRequest { /* DAP bridge fields */ },
    DapResponse { /* DAP bridge fields */ },
}
```

### Channel Architecture

Jupyter uses 5 ZeroMQ channels:

| Channel | Pattern | Purpose |
|---------|---------|---------|
| shell | REQ-REP | Execute requests |
| iopub | PUB-SUB | Broadcast outputs |
| stdin | REQ-REP | Input requests |
| control | REQ-REP | Control commands |
| hb | REQ-REP | Heartbeat |

## Usage Examples

### Creating a Kernel

```rust
use llmspell_kernel::{JupyterKernel, ConnectionInfo};
use llmspell_config::LLMSpellConfig;
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<()> {
    // Create kernel with auto-generated connection info
    let config = Arc::new(LLMSpellConfig::default());
    let connection = ConnectionInfo::new_tcp("127.0.0.1")?;
    
    let kernel = JupyterKernel::from_connection_info(
        connection.clone(),
        config,
    ).await?;
    
    // Write connection file for clients
    connection.write_connection_file(
        "/tmp/kernel-connection.json"
    )?;
    
    // Start kernel event loop
    kernel.start().await?;
    
    Ok(())
}
```

### Connecting as Client

```rust
use llmspell_kernel::{JupyterClient, ConnectionInfo};

async fn connect_to_kernel() -> Result<JupyterClient> {
    // Read connection info
    let connection = ConnectionInfo::from_file(
        "/tmp/kernel-connection.json"
    )?;
    
    // Create client
    let client = JupyterClient::new(connection).await?;
    
    // Execute code
    let result = client.execute_code(
        "print('Hello from kernel!')",
        false, // not silent
    ).await?;
    
    println!("Output: {:?}", result.output);
    
    Ok(client)
}
```

### EmbeddedKernel Pattern

The EmbeddedKernel runs the kernel in a background thread within the same process:

```rust
use llmspell_kernel::{JupyterKernel, KernelDiscovery};

async fn embedded_kernel_example() -> Result<()> {
    // Check for existing kernel
    let discovery = KernelDiscovery::new();
    
    let kernel = if let Some(info) = discovery.find_running_kernel().await? {
        // Connect to existing
        JupyterKernel::connect(info).await?
    } else {
        // Spawn embedded kernel
        JupyterKernel::spawn_embedded().await?
    };
    
    // Kernel is now running in background thread
    // Connection overhead: ~1ms after first use
    
    Ok(())
}
```

### Custom Transport Implementation

```rust
use llmspell_kernel::traits::{Transport, TransportConfig};
use async_trait::async_trait;

pub struct TcpTransport {
    sockets: HashMap<String, TcpStream>,
}

#[async_trait]
impl Transport for TcpTransport {
    async fn bind(&mut self, config: &TransportConfig) -> Result<()> {
        // Bind TCP sockets
        for (channel, cfg) in &config.channels {
            let addr = format!("{}:{}", config.base_address, cfg.endpoint);
            let socket = TcpListener::bind(addr).await?;
            self.sockets.insert(channel.clone(), socket);
        }
        Ok(())
    }
    
    async fn recv(&self, channel: &str) -> Result<Option<Vec<Vec<u8>>>> {
        // Receive from TCP socket
        // ...
    }
    
    // ... other methods
}
```

### Custom Protocol Implementation

```rust
use llmspell_kernel::traits::{Protocol, KernelMessage};
use async_trait::async_trait;

pub struct LspProtocol {
    // LSP-specific state
}

#[async_trait]
impl Protocol for LspProtocol {
    type Message = LspMessage;
    
    fn name(&self) -> &str {
        "lsp"
    }
    
    fn decode(&self, parts: Vec<Vec<u8>>) -> Result<Self::Message> {
        // Decode LSP message from wire format
        let json = String::from_utf8(parts[0].clone())?;
        Ok(serde_json::from_str(&json)?)
    }
    
    fn encode(&self, msg: &Self::Message, _channel: &str) -> Result<Vec<Vec<u8>>> {
        // Encode to LSP wire format
        let json = serde_json::to_string(msg)?;
        Ok(vec![json.into_bytes()])
    }
    
    // ... other methods
}
```

## DAP Bridge Integration

The kernel includes a DAP bridge for IDE debugging:

```rust
use llmspell_kernel::dap_bridge::{DapBridge, DapCommand};

impl JupyterKernel {
    pub async fn handle_dap_request(&self, cmd: DapCommand) -> Result<DapResponse> {
        let bridge = DapBridge::new(self.runtime.clone());
        
        match cmd {
            DapCommand::SetBreakpoints { source, breakpoints } => {
                bridge.set_breakpoints(source, breakpoints).await
            }
            DapCommand::Continue { thread_id } => {
                bridge.continue_execution(thread_id).await
            }
            DapCommand::StepOver { thread_id } => {
                bridge.step_over(thread_id).await
            }
            DapCommand::Variables { reference } => {
                bridge.get_variables(reference).await
            }
            // ... other commands
        }
    }
}
```

### Supported DAP Commands

| Command | Purpose | Implementation |
|---------|---------|----------------|
| setBreakpoints | Set/clear breakpoints | ExecutionManager |
| continue | Resume execution | DebugCoordinator |
| next | Step over | ExecutionBridge |
| stepIn | Step into | ExecutionBridge |
| stepOut | Step out | ExecutionBridge |
| stackTrace | Get call stack | StackNavigator |
| scopes | Get variable scopes | VariableInspector |
| variables | Get variables | VariableInspector |
| evaluate | Evaluate expression | ConditionEvaluator |
| pause | Pause execution | DebugCoordinator |

## Session Persistence

The kernel maintains session state across executions:

```rust
use llmspell_kernel::session_persistence::{SessionMapper, SessionState};

impl SessionMapper {
    /// Map Jupyter session to LLMSpell session
    pub async fn get_or_create_session(&self, jupyter_session: &str) -> Result<SessionState> {
        if let Some(state) = self.cache.get(jupyter_session) {
            return Ok(state);
        }
        
        // Create new session with state manager
        let state = SessionState::new(jupyter_session);
        if let Some(ref mgr) = self.state_manager {
            state.restore_from(mgr).await?;
        }
        
        self.cache.insert(jupyter_session.to_string(), state);
        Ok(state)
    }
    
    /// Persist session state
    pub async fn save_session(&self, jupyter_session: &str) -> Result<()> {
        if let Some(state) = self.cache.get(jupyter_session) {
            if let Some(ref mgr) = self.state_manager {
                state.persist_to(mgr).await?;
            }
        }
        Ok(())
    }
}
```

## Security & Resource Management

### Authentication

```rust
pub struct SecurityManager {
    kernel_key: String,
    auth_enabled: bool,
    client_tokens: Arc<RwLock<HashMap<String, ClientAuth>>>,
}

impl SecurityManager {
    /// Validate client authentication
    pub async fn validate_client(&self, token: &str) -> Result<bool> {
        if !self.auth_enabled {
            return Ok(true);
        }
        
        let tokens = self.client_tokens.read().await;
        Ok(tokens.contains_key(token))
    }
    
    /// Register new client
    pub async fn register_client(&self, client_id: &str) -> Result<String> {
        let token = Uuid::new_v4().to_string();
        let auth = ClientAuth {
            client_id: client_id.to_string(),
            token: token.clone(),
            created_at: Utc::now(),
        };
        
        self.client_tokens.write().await.insert(token.clone(), auth);
        Ok(token)
    }
}
```

### Resource Limits

```rust
pub struct ClientResourceLimits {
    pub max_execution_time_ms: u64,      // Default: 30s
    pub max_memory_bytes: usize,         // Default: 100MB
    pub max_concurrent_executions: usize, // Default: 5
}

impl GenericKernel<T, P> {
    /// Apply resource limits to execution
    async fn execute_with_limits(&self, code: &str, client_id: &str) -> Result<ExecutionResult> {
        // Check concurrent execution limit
        let client = self.client_manager.get_client(client_id)?;
        if client.active_executions >= self.resource_limits.max_concurrent_executions {
            return Err(anyhow!("Concurrent execution limit exceeded"));
        }
        
        // Apply timeout
        let result = timeout(
            Duration::from_millis(self.resource_limits.max_execution_time_ms),
            self.runtime.lock().await.execute_script(code),
        ).await??;
        
        Ok(result)
    }
}
```

## Discovery & Auto-spawn

The kernel supports automatic discovery and spawning:

```rust
use llmspell_kernel::discovery::{KernelDiscovery, KernelInfo};

pub struct KernelDiscovery {
    runtime_dir: PathBuf,
}

impl KernelDiscovery {
    /// Find running kernels
    pub async fn find_running_kernel(&self) -> Result<Option<ConnectionInfo>> {
        // Scan runtime directory for connection files
        for entry in fs::read_dir(&self.runtime_dir)? {
            let path = entry?.path();
            if path.extension() == Some(OsStr::new("json")) {
                // Try to connect
                if let Ok(info) = ConnectionInfo::from_file(&path) {
                    if self.is_kernel_alive(&info).await {
                        return Ok(Some(info));
                    }
                }
            }
        }
        Ok(None)
    }
    
    /// Check if kernel is responsive
    async fn is_kernel_alive(&self, info: &ConnectionInfo) -> bool {
        // Send heartbeat and check response
        // ...
    }
    
    /// Auto-spawn kernel if needed
    pub async fn ensure_kernel(&self) -> Result<ConnectionInfo> {
        if let Some(info) = self.find_running_kernel().await? {
            Ok(info)
        } else {
            // Spawn new embedded kernel
            let kernel = JupyterKernel::spawn_embedded().await?;
            Ok(kernel.connection_info())
        }
    }
}
```

## Performance Metrics

Phase 9 achieved all performance targets:

| Metric | Target | Actual | Notes |
|--------|--------|--------|-------|
| Kernel startup | <100ms | 95ms | Including ScriptRuntime init |
| ZeroMQ round-trip | <1ms | 0.8ms | After connection established |
| Connection reuse | - | ✅ | ~1ms overhead after first run |
| Concurrent clients | 10+ | 50+ | Limited by resources |
| Message throughput | 1000/s | 5000/s | On localhost |

## Configuration

### Kernel Configuration

```toml
[runtime.kernel]
enabled = true
auth_enabled = false
max_clients = 50
heartbeat_interval_ms = 5000
execution_timeout_ms = 30000

[runtime.kernel.transport]
type = "tcp"
ip = "127.0.0.1"
shell_port = 50501
iopub_port = 50502
stdin_port = 50503
control_port = 50504
hb_port = 50505

[runtime.kernel.security]
signature_scheme = "hmac-sha256"
key = ""  # Auto-generated if empty
```

### Connection File Format

```json
{
  "shell_port": 50501,
  "iopub_port": 50502,
  "stdin_port": 50503,
  "control_port": 50504,
  "hb_port": 50505,
  "ip": "127.0.0.1",
  "key": "a7b3c5d9-4f2e-4a8b-9c6d-1e3f5g7h9i0j",
  "transport": "tcp",
  "signature_scheme": "hmac-sha256",
  "kernel_name": "llmspell"
}
```

## Testing

The crate includes comprehensive tests:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use llmspell_testing::prelude::*;
    
    #[tokio::test]
    async fn test_kernel_lifecycle() {
        let kernel = JupyterKernel::spawn_embedded().await.unwrap();
        
        // Test kernel info
        let info = kernel.kernel_info().await.unwrap();
        assert_eq!(info.implementation, "llmspell");
        
        // Test execution
        let result = kernel.execute("print('test')").await.unwrap();
        assert!(result.output.contains("test"));
        
        // Test shutdown
        kernel.shutdown(false).await.unwrap();
    }
    
    #[tokio::test]
    async fn test_concurrent_clients() {
        let kernel = JupyterKernel::spawn_embedded().await.unwrap();
        let info = kernel.connection_info();
        
        // Spawn multiple clients
        let mut clients = vec![];
        for i in 0..10 {
            let client = JupyterClient::new(info.clone()).await.unwrap();
            clients.push(client);
        }
        
        // Execute concurrently
        let mut handles = vec![];
        for (i, client) in clients.into_iter().enumerate() {
            let handle = tokio::spawn(async move {
                client.execute_code(&format!("print({})", i), false).await
            });
            handles.push(handle);
        }
        
        // Verify all completed
        for handle in handles {
            assert!(handle.await.unwrap().is_ok());
        }
    }
}
```

## Migration from llmspell-engine

The kernel replaces the deprecated `llmspell-engine` crate:

### Key Differences

| Aspect | llmspell-engine | llmspell-kernel |
|--------|-----------------|-----------------|
| Architecture | UnifiedProtocolEngine | GenericKernel<T, P> |
| Protocols | Adapter pattern | Trait-based |
| Process model | Sidecar processes | EmbeddedKernel |
| Transport | Custom protocol | ZeroMQ/Jupyter |
| State | Per-execution | Session persistence |

### Migration Steps

1. **Replace imports:**
```rust
// Old
use llmspell_engine::{ProtocolEngine, EngineConfig};

// New
use llmspell_kernel::{JupyterKernel, ConnectionInfo};
```

2. **Update initialization:**
```rust
// Old
let engine = ProtocolEngine::new(config).await?;
engine.start_sidecar("lua").await?;

// New
let kernel = JupyterKernel::spawn_embedded().await?;
```

3. **Update execution:**
```rust
// Old
let result = engine.execute_script(code).await?;

// New
let result = kernel.execute(code).await?;
```

## Future Extensions

The trait-based architecture enables future protocol support:

- **LSP**: Language Server Protocol for IDE integration
- **MCP**: Model Context Protocol for LLM tools
- **Custom**: Application-specific protocols

Example extension:
```rust
// Implement custom protocol
struct MyProtocol;
impl Protocol for MyProtocol { /* ... */ }

// Use with kernel
type MyKernel = GenericKernel<ZmqTransport, MyProtocol>;
```

## Related Documentation

- [llmspell-bridge](llmspell-bridge.md) - Script runtime that kernel manages
- [llmspell-debug](llmspell-debug.md) - Debug infrastructure integrated with kernel
- [llmspell-repl](llmspell-repl.md) - REPL that connects to kernel
- [llmspell-cli](llmspell-cli.md) - CLI commands for kernel management
- [Kernel Architecture](../../../technical/kernel-protocol-architecture.md) - Detailed design

---

**Version**: 0.9.0 | **Phase**: 9 | **Status**: Complete