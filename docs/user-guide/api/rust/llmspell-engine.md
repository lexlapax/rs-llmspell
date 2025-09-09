# llmspell-engine

**⚠️ DEPRECATED - Replaced by llmspell-kernel in v0.9.0**

**🔗 Navigation**: [← Rust API](README.md) | [Migration Guide](llmspell-kernel.md)

---

## ⚠️ Deprecation Notice

**This crate has been deprecated in v0.9.0 and replaced by [`llmspell-kernel`](llmspell-kernel.md).**

The `llmspell-engine` crate was the original protocol engine implementation that used:
- UnifiedProtocolEngine with adapter pattern
- Sidecar processes for kernel execution
- Custom protocol abstractions
- Complex multi-protocol support

## Why Deprecated?

Phase 9 analysis revealed fundamental architectural issues:
1. **Overengineering**: Multi-protocol abstractions were unnecessary complexity
2. **State Issues**: Sidecar processes couldn't maintain state properly
3. **Performance**: IPC overhead between processes was significant
4. **Debugging**: Complex to debug across process boundaries

## Migration to llmspell-kernel

The new `llmspell-kernel` crate provides:
- **Simpler Architecture**: `GenericKernel<T: Transport, P: Protocol>` trait-based design
- **EmbeddedKernel**: Runs in background thread, not separate process
- **Better Performance**: ~1ms overhead after first execution (connection reused)
- **State Persistence**: Maintains state across executions
- **Debug Support**: Integrated ExecutionManager and DAP bridge

### Migration Example

**Old (llmspell-engine):**
```rust
use llmspell_engine::{ProtocolEngine, EngineConfig};

// Complex setup with adapters
let engine = ProtocolEngine::new(config).await?;
engine.start_sidecar("lua").await?;

// Execute through sidecar
let result = engine.execute_script(code).await?;

// State lost between executions
```

**New (llmspell-kernel):**
```rust
use llmspell_kernel::{JupyterKernel, ConnectionInfo};

// Simple kernel spawn
let kernel = JupyterKernel::spawn_embedded().await?;

// Direct execution
let result = kernel.execute(code).await?;

// State persists automatically
```

## Key Differences

| Aspect | llmspell-engine | llmspell-kernel |
|--------|-----------------|-----------------|
| Architecture | UnifiedProtocolEngine with adapters | GenericKernel<T, P> traits |
| Process Model | Sidecar processes | EmbeddedKernel (thread) |
| Protocol Support | Multi-protocol abstractions | Clean trait separation |
| State Management | Per-execution | Session persistence |
| Debug Support | Limited | Full ExecutionManager + DAP |
| Performance | IPC overhead | ~1ms after first run |
| Complexity | High (adapters, sidecars) | Low (traits, embedded) |

## Removal Timeline

- **v0.9.0**: Deprecated, replaced by llmspell-kernel
- **v1.0.0**: Will be removed from codebase

## Migration Steps

1. **Update Dependencies:**
```toml
# Remove
llmspell-engine = "0.8"

# Add
llmspell-kernel = "0.9"
```

2. **Update Imports:**
```rust
// Remove
use llmspell_engine::{ProtocolEngine, EngineConfig};

// Add
use llmspell_kernel::{JupyterKernel, ConnectionInfo};
```

3. **Update Initialization:**
```rust
// Remove complex engine setup
// Add simple kernel spawn
let kernel = JupyterKernel::spawn_embedded().await?;
```

4. **Update Execution:**
```rust
// Same interface, better implementation
let result = kernel.execute(code).await?;
```

## Related Documentation

- [llmspell-kernel](llmspell-kernel.md) - New kernel architecture
- [Migration Guide](../../migration-0.9.0.md) - Full migration details
- [Kernel Architecture](../../../technical/kernel-protocol-architecture.md) - Design decisions

---

**For all new code, use [`llmspell-kernel`](llmspell-kernel.md) instead.**