# llmspell-kernel

## Purpose

The unified kernel crate providing integrated execution runtime, state management, session handling, debugging infrastructure, and daemon/service capabilities. This crate consolidates what were previously separate crates (llmspell-sessions, llmspell-state-persistence, llmspell-state-traits) and adds new Phase 10 daemon and protocol server features.

**Phase 12 Enhancements**: KernelHandle extended with template protocol support for interactive workflows and REPL integration.

## Core Concepts

- **Integrated Kernel**: Central execution runtime managing script execution with debugging support
- **Daemon Management**: System service deployment with signal handling and PID management
- **State Management**: Hierarchical state with multiple backends (memory, Sled, vector stores)
- **Session Management**: Session lifecycle with artifact storage and replay capabilities
- **Protocol Support**: Multi-protocol transport (Jupyter, DAP, LSP) with unified interface (Phase 12: template_request/template_reply)
- **Debug Infrastructure**: Complete debugging support with breakpoints, stepping, and variable inspection
- **Event Correlation**: Distributed tracing and event correlation across components
- **Global IO Runtime**: Shared Tokio runtime preventing "dispatch task is gone" errors
- **Template Integration**: Infrastructure access for production-ready AI workflow templates (Phase 12)

## Primary Structs/Modules

### IntegratedKernel

**Purpose**: Main kernel implementation providing script execution with integrated debugging, state management, and protocol support.

**When to use**: When you need a complete kernel instance for running LLMSpell scripts with full debugging and protocol capabilities.

**Key methods**:
- `new()` - Create kernel with protocol and configuration
- `start()` - Begin kernel operation
- `handle_message()` - Process protocol messages
- `shutdown()` - Graceful shutdown

```rust
use llmspell_kernel::{
    IntegratedKernel, ExecutionConfig,
    transport::jupyter::JupyterTransport,
};
use std::sync::Arc;

// Create kernel with Jupyter protocol
let transport = JupyterTransport::new(connection_info)?;
let config = ExecutionConfig {
    enable_debugging: true,
    max_execution_time: Duration::from_secs(300),
    memory_limit_mb: 512,
};
let script_executor = Arc::new(MyScriptExecutor::new());
let kernel = IntegratedKernel::new(
    transport,
    config,
    "session-123".to_string(),
    script_executor,
).await?;

// Start kernel
kernel.start().await?;
```

### DaemonManager

**Purpose**: Manages process daemonization using double-fork technique for system service deployment.

**When to use**: When deploying LLMSpell kernel as a system service (systemd/launchd).

**Key methods**:
- `new()` - Create manager with configuration
- `daemonize()` - Fork to background as daemon
- `create_pid_file()` - Create PID file for process management
- `cleanup()` - Clean shutdown with PID file removal

```rust
use llmspell_kernel::daemon::{DaemonManager, DaemonConfig};
use std::path::PathBuf;

let config = DaemonConfig {
    daemonize: true,
    pid_file: Some(PathBuf::from("/var/run/llmspell/kernel.pid")),
    working_dir: PathBuf::from("/var/lib/llmspell"),
    stdout_path: Some(PathBuf::from("/var/log/llmspell/stdout.log")),
    stderr_path: Some(PathBuf::from("/var/log/llmspell/stderr.log")),
    close_stdin: true,
    umask: Some(0o027),
};

let mut manager = DaemonManager::new(config);

// Daemonize the process
if config.daemonize {
    manager.daemonize()?;
}

// Your kernel runs here...

// Cleanup on shutdown
manager.cleanup()?;
```

### SignalBridge

**Purpose**: Bridges Unix signals to async events for graceful shutdown and configuration reload.

**When to use**: When you need to handle system signals in async context.

**Supported signals**:
- `SIGTERM` - Graceful shutdown
- `SIGINT` - Graceful shutdown (Ctrl+C)
- `SIGHUP` - Configuration reload
- `SIGUSR1` - Dump statistics
- `SIGUSR2` - Toggle debug logging

```rust
use llmspell_kernel::daemon::SignalBridge;
use tokio::select;

let signal_bridge = SignalBridge::new();

// Install signal handlers
signal_bridge.install_handlers()?;

// In your async runtime
loop {
    select! {
        _ = signal_bridge.wait_for_shutdown() => {
            info!("Received shutdown signal");
            break;
        }
        _ = signal_bridge.wait_for_reload() => {
            info!("Received reload signal");
            reload_configuration().await?;
        }
        // ... other async operations
    }
}
```

### KernelState

**Purpose**: Unified state management with hierarchical scoping and multiple storage backends.

**When to use**: When you need persistent state across script executions with proper isolation.

**Key features**:
- Hierarchical scopes (global, session, agent, tool)
- Multiple backends (memory, Sled DB, vector stores)
- Atomic operations with optimistic concurrency
- TTL support for ephemeral data

```rust
use llmspell_kernel::state::{
    KernelState, StateScope, StorageBackend,
    MemoryBackend, SledBackend,
};
use serde_json::json;

// Create state manager with Sled backend
let backend = Box::new(SledBackend::new("/var/lib/llmspell/state")?);
let state = KernelState::new(backend);

// Write to different scopes
state.set(
    StateScope::Global,
    "config",
    json!({"theme": "dark"}),
).await?;

state.set(
    StateScope::Session("session-123".to_string()),
    "user_prefs",
    json!({"language": "en"}),
).await?;

// Read with fallback through scopes
let value = state.get_with_fallback(
    StateScope::Agent("agent-1".to_string()),
    "setting",
    vec![
        StateScope::Session("session-123".to_string()),
        StateScope::Global,
    ],
).await?;

// Atomic compare-and-swap
let updated = state.compare_and_swap(
    StateScope::Global,
    "counter",
    Some(json!(5)),  // expected
    json!(6),         // new value
).await?;
```

### SessionManager

**Purpose**: Manages session lifecycle with artifact storage and replay capabilities.

**When to use**: When you need to track user sessions with associated artifacts and state.

**Key features**:
- Session creation with metadata
- Artifact storage (code, output, media)
- Session replay for testing
- Security context per session

```rust
use llmspell_kernel::sessions::{
    SessionManager, SessionManagerConfig,
    CreateSessionOptions, ArtifactType,
};
use std::sync::Arc;

let config = SessionManagerConfig::builder()
    .max_sessions(100)
    .session_timeout(Duration::from_hours(24))
    .artifact_storage_path("/var/lib/llmspell/artifacts")
    .build()?;

let manager = Arc::new(SessionManager::new(config).await?);

// Create session
let options = CreateSessionOptions::builder()
    .timeout(Duration::from_hours(1))
    .metadata("user_id", "user-123")
    .metadata("project", "my-project")
    .build();

let session = manager.create_session(options).await?;

// Store artifact
let artifact_id = session.store_artifact(
    ArtifactType::Code,
    b"print('Hello, World!')",
    Some("hello.py".to_string()),
).await?;

// Retrieve artifact
let artifact = session.get_artifact(&artifact_id).await?;
```

### KernelHandle ⭐ **Phase 12**

**Purpose**: Handle for embedded kernel providing programmatic access to kernel operations including templates.

**When to use**: When templates need REPL/interactive kernel access or for embedded kernel execution.

**Key features**:
- Template protocol support (template_request/template_reply)
- Tool and model management protocol
- Code execution via Jupyter protocol
- In-process transport for embedded kernels

**Phase 12 Enhancement**: Added `send_template_request()` for interactive workflows (Subtask 12.9.5).

```rust
use llmspell_kernel::api::{KernelHandle, start_embedded_kernel_with_infrastructure};
use llmspell_config::LLMSpellConfig;
use serde_json::json;

// Start embedded kernel with full infrastructure
let config = LLMSpellConfig::load_from_file("llmspell.toml")?;
let script_executor = create_script_executor()?;
let session_manager = create_session_manager().await?;

let mut kernel_handle = start_embedded_kernel_with_infrastructure(
    config,
    script_executor,
    session_manager,
).await?;

// Execute template via kernel (Phase 12)
let template_params = json!({
    "template_name": "research-assistant",
    "params": {
        "topic": "Rust async programming",
        "max_sources": 10
    }
});

let result = kernel_handle.send_template_request(template_params).await?;
println!("Template result: {}", result);

// Execute code
let code_result = kernel_handle.execute("print('Hello from kernel')").await?;

// Send tool request
let tool_request = json!({
    "tool_name": "calculator",
    "parameters": {"operation": "add", "a": 5, "b": 3}
});
let tool_result = kernel_handle.send_tool_request(tool_request).await?;
```

**Integration with llmspell-templates**:

Templates use KernelHandle via `ExecutionContext.kernel_handle` for interactive sessions:

```rust
use llmspell_templates::{Template, ExecutionContext, TemplateParams};

// In template implementation
async fn execute(
    &self,
    params: TemplateParams,
    context: ExecutionContext,
) -> Result<TemplateOutput> {
    // Access kernel for REPL operations
    if let Some(kernel) = context.kernel_handle() {
        let result = kernel.send_template_request(request).await?;
        // Process result...
    }
    Ok(output)
}
```

### ExecutionManager

**Purpose**: Manages script execution with debugging support including breakpoints and stepping.

**When to use**: When implementing debugging features in script executors.

**Key features**:
- Breakpoint management
- Step debugging (in/over/out)
- Variable inspection
- Call stack management

```rust
use llmspell_kernel::debug::{
    ExecutionManager, Breakpoint, StepMode,
};
use std::sync::Arc;

let manager = Arc::new(ExecutionManager::new("session-123".to_string()));

// Set breakpoints
manager.set_breakpoints(vec![
    Breakpoint {
        id: 1,
        file: "script.lua".to_string(),
        line: 10,
        condition: None,
        hit_count: 0,
    },
]).await?;

// Check if should pause at location
if manager.should_pause("script.lua", 10).await? {
    // Pause execution
    manager.pause_at("script.lua", 10).await?;

    // Wait for debugger command
    match manager.wait_for_resume().await? {
        StepMode::Continue => {
            // Continue execution
        }
        StepMode::StepIn => {
            // Step into function
        }
        StepMode::StepOver => {
            // Step over line
        }
        StepMode::StepOut => {
            // Step out of function
        }
    }
}
```

### DAPBridge

**Purpose**: Debug Adapter Protocol implementation for IDE integration.

**When to use**: When integrating with VS Code or other DAP-compatible debuggers.

**Key features**:
- Full DAP protocol support
- Source mapping
- Variable evaluation
- Conditional breakpoints

```rust
use llmspell_kernel::debug::{DAPBridge, DapCapabilities};
use std::sync::Arc;

let execution_manager = Arc::new(ExecutionManager::new("session-123".to_string()));
let bridge = DAPBridge::new("session-123".to_string(), execution_manager);

// Initialize DAP session
let capabilities = bridge.initialize(client_capabilities).await?;

// Handle DAP requests
match request {
    DapRequest::SetBreakpoints { source, breakpoints } => {
        let result = bridge.set_breakpoints(source, breakpoints).await?;
        send_response(result);
    }
    DapRequest::StackTrace { thread_id } => {
        let frames = bridge.get_stack_trace(thread_id).await?;
        send_response(frames);
    }
    DapRequest::Variables { reference } => {
        let vars = bridge.get_variables(reference).await?;
        send_response(vars);
    }
    // ... other DAP commands
}
```

### EventCorrelator

**Purpose**: Correlates events across distributed components for tracing and debugging.

**When to use**: When you need to track execution flow across multiple components.

**Key features**:
- Correlation ID generation
- Parent-child relationships
- Distributed tracing support
- Event aggregation

```rust
use llmspell_kernel::events::{
    KernelEventCorrelator, KernelEvent,
    EventBroadcaster,
};
use std::sync::Arc;

let correlator = Arc::new(KernelEventCorrelator::new());
let broadcaster = Arc::new(EventBroadcaster::new());

// Create correlated event
let correlation_id = correlator.generate_id();
let event = KernelEvent::ExecutionStarted {
    correlation_id: correlation_id.clone(),
    parent_id: None,
    session_id: "session-123".to_string(),
    timestamp: Utc::now(),
};

// Broadcast event
broadcaster.broadcast(event).await;

// Create child event
let child_event = KernelEvent::ToolInvoked {
    correlation_id: correlator.generate_child_id(&correlation_id),
    parent_id: Some(correlation_id),
    tool_name: "calculator".to_string(),
    timestamp: Utc::now(),
};

broadcaster.broadcast(child_event).await;
```

### Transport Traits

**Purpose**: Unified interface for different protocol transports (Jupyter, DAP, LSP).

**When to use**: When implementing new protocol support or customizing transport behavior.

**Key traits**:
- `Protocol` - Protocol-specific message handling
- `Transport` - Low-level transport operations
- `ChannelConfig` - Channel configuration

```rust
use llmspell_kernel::traits::{Protocol, Transport};
use async_trait::async_trait;

#[async_trait]
impl<T: Transport> Protocol<T> for MyProtocol {
    async fn initialize(&mut self) -> Result<()> {
        // Protocol initialization
        Ok(())
    }

    async fn handle_message(&mut self, msg: T::Message) -> Result<T::Response> {
        // Handle protocol-specific message
        Ok(response)
    }

    async fn shutdown(&mut self) -> Result<()> {
        // Cleanup
        Ok(())
    }
}
```

## Advanced Features

### Circuit Breaker

**Purpose**: Prevents cascading failures by monitoring error rates and temporarily disabling operations.

**When to use**: When integrating with external services that may fail.

```rust
use llmspell_kernel::state::circuit_breaker::{
    CircuitBreaker, CircuitState,
};

let breaker = CircuitBreaker::new(
    5,                              // failure threshold
    Duration::from_secs(60),        // reset timeout
    Duration::from_secs(300),       // monitoring window
);

// Check if circuit allows operation
if breaker.can_proceed() {
    match external_service_call().await {
        Ok(result) => {
            breaker.record_success();
            // Process result
        }
        Err(e) => {
            breaker.record_failure();
            // Handle error
        }
    }
} else {
    // Circuit is open, skip operation
    return Err("Service temporarily unavailable");
}
```

### Performance Optimization

**Purpose**: Fast-path state operations for ephemeral data and trusted sources.

```rust
use llmspell_kernel::state::performance::{
    FastPathManager, FastPathConfig,
};

let config = FastPathConfig {
    use_messagepack: true,           // Binary serialization
    enable_compression: true,        // Compress large values
    compression_threshold: 1024,     // Compress over 1KB
    enable_ephemeral_cache: true,    // In-memory cache
    ephemeral_cache_limit: 10_000,   // Max cache entries
};

let fast_path = FastPathManager::new(config);

// Store ephemeral data (no persistence)
fast_path.store_ephemeral(
    &StateScope::Session("session-123".to_string()),
    "temp_data",
    json!({"status": "processing"}),
)?;

// Fast serialization for trusted data
let serialized = fast_path.serialize_trusted(&value)?;
```

## Usage Patterns

### Starting a Kernel Service

```rust
use llmspell_kernel::{
    daemon::{DaemonManager, DaemonConfig, SignalBridge},
    IntegratedKernel, ExecutionConfig,
    transport::jupyter::JupyterTransport,
};

#[tokio::main]
async fn main() -> Result<()> {
    // Setup daemon if configured
    let daemon_config = DaemonConfig::from_env()?;
    let mut daemon_manager = DaemonManager::new(daemon_config.clone());

    if daemon_config.daemonize {
        daemon_manager.daemonize()?;
    }

    // Setup signal handling
    let signal_bridge = SignalBridge::new();
    signal_bridge.install_handlers()?;

    // Create and start kernel
    let transport = JupyterTransport::from_connection_file("kernel.json")?;
    let kernel = IntegratedKernel::new(
        transport,
        ExecutionConfig::default(),
        "main".to_string(),
        create_script_executor()?,
    ).await?;

    // Run until shutdown signal
    tokio::select! {
        result = kernel.start() => {
            result?;
        }
        _ = signal_bridge.wait_for_shutdown() => {
            info!("Shutting down kernel");
            kernel.shutdown().await?;
        }
    }

    daemon_manager.cleanup()?;
    Ok(())
}
```

### Implementing Debug Support

```rust
use llmspell_kernel::debug::{DebugContext, ExecutionManager};
use llmspell_bridge::ScriptExecutor;

struct DebugAwareExecutor {
    debug_context: Option<Arc<dyn DebugContext>>,
}

#[async_trait]
impl ScriptExecutor for DebugAwareExecutor {
    fn set_debug_context(&self, context: Option<Arc<dyn DebugContext>>) {
        self.debug_context.write().unwrap() = context;
    }

    async fn execute_script(&self, script: &str) -> Result<ScriptOutput> {
        // Parse script and execute line by line
        for (line_num, line_code) in script.lines().enumerate() {
            // Check for breakpoints
            if let Some(ctx) = &self.debug_context {
                if ctx.should_pause("script", line_num as u32).await? {
                    ctx.pause_at("script", line_num as u32).await?;

                    // Wait for debugger command
                    let mode = ctx.wait_for_resume().await?;
                    // Handle step mode...
                }
            }

            // Execute line
            execute_line(line_code)?;
        }

        Ok(output)
    }
}
```

## Configuration

### Kernel Configuration

```toml
[kernel]
enable_debugging = true
max_execution_time_secs = 300
memory_limit_mb = 512
idle_timeout_secs = 3600

[daemon]
daemonize = true
pid_file = "/var/run/llmspell/kernel.pid"
working_dir = "/var/lib/llmspell"
log_file = "/var/log/llmspell/kernel.log"

[state]
backend = "sled"  # or "memory", "redis"
path = "/var/lib/llmspell/state"
cache_size_mb = 100

[session]
max_sessions = 100
session_timeout_hours = 24
artifact_storage_path = "/var/lib/llmspell/artifacts"
```

## Error Handling

```rust
use llmspell_kernel::{KernelError, Result};

// Kernel-specific errors
match operation().await {
    Err(KernelError::Daemon { message, source }) => {
        // Daemon operation failed
    }
    Err(KernelError::Signal { signal, errno }) => {
        // Signal handling error
    }
    Err(KernelError::Protocol { protocol, message }) => {
        // Protocol error
    }
    Err(KernelError::State { scope, key, operation }) => {
        // State operation failed
    }
    Err(KernelError::Session { session_id, operation }) => {
        // Session operation failed
    }
    Err(KernelError::Debug { message }) => {
        // Debug operation failed
    }
    Ok(result) => {
        // Success
    }
}
```

## Performance Considerations

- **Global IO Runtime**: Single shared Tokio runtime prevents context switching overhead
- **State Caching**: LRU cache for frequently accessed state reduces backend queries
- **Ephemeral Cache**: In-memory storage for temporary data avoids persistence overhead
- **MessagePack**: Binary serialization faster than JSON for large payloads
- **Connection Pooling**: Reuse protocol connections for better throughput
- **Circuit Breaker**: Prevents cascading failures from external service issues

## Security Notes

- **Daemon Privileges**: Drop privileges after binding to ports
- **PID File Security**: Restrict PID file permissions (0644)
- **Signal Validation**: Only handle expected signals
- **State Isolation**: Enforce scope boundaries in multi-tenant scenarios
- **Session Security**: Validate session tokens and enforce timeouts
- **Debug Access**: Require authentication for debug operations