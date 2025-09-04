# llmspell-kernel

Jupyter-compatible kernel service for rs-llmspell with unified configuration and shared state architecture.

## Overview

This crate implements the kernel service for rs-llmspell (Phase 9.8), providing a Jupyter Messaging Protocol compatible kernel that manages script execution, state persistence, and debug operations. It replaces the previous llmspell-engine architecture with a unified kernel-based approach.

## Current Status (Phase 9.8)

### ✅ Completed Tasks
- **Task 9.8.3**: Created llmspell-kernel crate structure
- **Task 9.8.4**: Migrated kernel code from llmspell-engine
- **Task 9.8.5**: Implemented core Jupyter Protocol support (partial)
- **Task 9.8.7**: Session persistence with Jupyter Protocol
- **Task 9.8.8**: Unified configuration & shared state architecture

### 🚧 In Progress
- **Task 9.8.1**: Refactor CLI to always use kernel connection
- **Task 9.8.2**: Kernel auto-start and discovery enhancement
- **Task 9.8.6**: Update CLI to use llmspell-kernel
- **Task 9.8.9**: Debug functionality completion
- **Task 9.8.10**: Complete removal of llmspell-engine
- **Task 9.8.11**: Clean migration to Jupyter architecture
- **Task 9.8.12**: Integration testing and validation

## Architecture

### Core Components

```
llmspell-kernel/
├── kernel.rs              # Main kernel implementation (GenericKernel)
├── protocol.rs            # Protocol abstraction layer
├── comm_handler.rs        # Comm channel message handling
├── session_persistence.rs # Session state management
├── connection.rs          # Connection file management
├── discovery.rs           # Kernel discovery system
├── security.rs            # Authentication & security
└── client.rs              # Client connection handling
```

### Key Features

#### 1. Unified Configuration (Task 9.8.8)
- Single `LLMSpellConfig` for entire system
- No duplicate `KernelConfig` - uses `LLMSpellConfig::runtime.kernel`
- Shared `StateManager` between kernel and `ScriptRuntime`
- Configuration-driven kernel behavior

#### 2. Jupyter Protocol Support
- **ZeroMQ Transport**: Multi-socket communication pattern
- **Message Types**: Execute, Inspect, Complete, History, Control
- **Comm Channels**: Custom bidirectional communication
- **Session Persistence**: State maintained across connections

#### 3. Shared State Architecture
```rust
// Single StateManager shared across components
let state_manager = StateFactory::create_from_config(&config).await?;

// Kernel uses shared StateManager
let kernel = GenericKernel::new(
    kernel_id,
    config.clone(),
    transport,
    protocol,
    state_manager.clone(), // Shared instance
).await?;

// ScriptRuntime uses same StateManager
let runtime = ScriptRuntime::new_with_engine_and_state_manager(
    &config.default_engine,
    (*config).clone(),
    state_manager.clone(), // Same instance
).await?;
```

#### 4. Session Management
- **Session Mapper**: Maps Jupyter sessions to internal state
- **Persistence**: Sessions survive kernel restarts
- **Multi-Client**: Support for multiple concurrent connections
- **State Sharing**: Sessions share state through unified StateManager

## Usage

### Starting the Kernel

```bash
# Start kernel with default configuration
llmspell-kernel

# Start with specific engine
llmspell-kernel --engine lua

# Start with custom port
llmspell-kernel --port 5678

# Enable authentication
llmspell-kernel --auth
```

### Connection File Format

The kernel generates a connection file compatible with Jupyter:

```json
{
  "transport": "tcp",
  "signature_scheme": "hmac-sha256",
  "key": "your-secret-key",
  "ip": "127.0.0.1",
  "control_port": 5678,
  "shell_port": 5679,
  "stdin_port": 5680,
  "iopub_port": 5681,
  "hb_port": 5682
}
```

### Programmatic Usage

```rust
use llmspell_kernel::{GenericKernel, JupyterTransport, JupyterProtocol};
use llmspell_config::LLMSpellConfig;
use std::sync::Arc;

// Create configuration
let config = Arc::new(LLMSpellConfig::builder()
    .default_engine("lua")
    .runtime(GlobalRuntimeConfig::builder()
        .kernel(KernelSettings {
            max_clients: 10,
            auth_enabled: true,
            heartbeat_interval_ms: 30000,
            legacy_tcp_port_offset: 1000,
            shutdown_timeout_seconds: 30,
        })
        .state_persistence(StatePersistenceConfig {
            enabled: true,
            backend_type: "sled".to_string(),
            ..Default::default()
        })
        .build())
    .build());

// Create transport and protocol
let transport = JupyterTransport::new(&connection_info).await?;
let protocol = JupyterProtocol::new();

// Create and run kernel
let kernel = GenericKernel::new(
    Uuid::new_v4().to_string(),
    config,
    transport,
    protocol,
).await?;

kernel.run().await?;
```

## Protocol Implementation

### Jupyter Message Handling

The kernel implements the Jupyter Messaging Protocol v5.3:

| Channel | Direction | Messages | Purpose |
|---------|-----------|----------|---------|
| Shell | REQ/REP | execute_request, inspect_request, complete_request | Main execution channel |
| IOPub | PUB | stream, display_data, execute_result, error | Output broadcasting |
| Stdin | REQ/REP | input_request, input_reply | User input handling |
| Control | REQ/REP | shutdown_request, interrupt_request | Kernel control |
| Heartbeat | REQ/REP | ping/pong | Connection monitoring |

### Comm Channels

Custom bidirectional communication for extensions:

```rust
// In kernel
comm_handler.register_handler("custom.channel", |msg| {
    // Handle custom messages
});

// From client
comm_open("custom.channel", {data: "value"});
comm_msg("custom.channel", {update: "new_value"});
comm_close("custom.channel");
```

## Security

### Authentication
- HMAC-SHA256 message signing
- Configurable authentication via `KernelSettings::auth_enabled`
- Secure key generation and storage
- Connection file permissions (0600)

### Multi-Client Isolation
- Session-scoped state isolation
- Client connection tracking
- Resource limits per client
- Secure comm channel registration

## State Persistence

### Shared StateManager Benefits
1. **No File Locks**: Single manager prevents conflicts
2. **Immediate Consistency**: All components see same state
3. **Memory Efficiency**: One backend instance
4. **Simplified Testing**: Single state source

### Session Persistence
```rust
// Sessions automatically persisted through StateManager
session_mapper.create_session(session_id).await?;
session_mapper.save_state(session_id, "key", value).await?;
let restored = session_mapper.load_state(session_id, "key").await?;
```

## Migration from llmspell-engine

### Phase 9.8 Migration Path

1. **Phase 9.8.1-9.8.5**: Create kernel crate, implement Jupyter protocol ✅
2. **Phase 9.8.6**: Update CLI to use kernel (in progress)
3. **Phase 9.8.7-9.8.8**: Session persistence & unified config ✅
4. **Phase 9.8.9**: Complete debug functionality
5. **Phase 9.8.10**: Remove llmspell-engine completely
6. **Phase 9.8.11-9.8.12**: Final migration and testing

### Breaking Changes
- `KernelConfig` removed - use `LLMSpellConfig::runtime.kernel`
- Direct TCP protocol replaced with Jupyter Messaging Protocol
- Engine-specific code moved to kernel implementation
- StateManager now shared between kernel and ScriptRuntime

## Testing

```bash
# Run kernel tests
cargo test -p llmspell-kernel

# Run integration tests
cargo test -p llmspell-kernel --test "*integration*"

# Test with Jupyter client
jupyter console --kernel llmspell --existing connection.json

# Test session persistence
cargo test -p llmspell-kernel test_session_persistence
```

## Performance

Target metrics (Phase 9.8):

| Operation | Target | Current |
|-----------|--------|---------|
| Kernel startup | <100ms | ~80ms |
| Message processing | <5ms | <3ms |
| Session creation | <10ms | <8ms |
| State persistence | <5ms | <5ms |
| Client connection | <50ms | ~40ms |

## Dependencies

- `llmspell-config` - Unified configuration management
- `llmspell-bridge` - ScriptRuntime integration
- `llmspell-state-persistence` - Shared state backend
- `llmspell-sessions` - Session management
- `zeromq` - Transport layer
- `serde_json` - Message serialization
- `tokio` - Async runtime
- `uuid` - Session identifiers

## Future Work (Phase 9.9+)

- Complete debug functionality integration
- Performance optimization for large sessions
- Extended Jupyter widget support
- Distributed kernel support
- GPU acceleration for ML workloads

## License

This project is licensed under Apache-2.0.