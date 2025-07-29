# llmspell-hooks

Comprehensive hook and event system for rs-llmspell with 40+ hook points, CircuitBreaker protection, and cross-language support.

## Overview

This crate provides the complete hook system implementation for rs-llmspell (Phase 4), enabling:

- **40+ Hook Points**: Complete coverage across agents, tools, and workflows
- **Performance Protection**: CircuitBreaker ensures <1% overhead (achieved <2%)
- **Cross-Language Support**: Hooks work seamlessly between Lua, JavaScript, and Rust
- **9 Hook Result Types**: Fine-grained control over execution flow
- **Built-in Hooks**: 18+ production-ready hooks for common patterns
- **Replay Support**: ReplayableHook trait for Phase 5 state persistence

## Features

### Hook Points

The system provides comprehensive hook coverage:

#### Agent Hooks (6 states)
- `agent:before_creation`, `agent:after_creation`
- `agent:before_execution`, `agent:after_execution`
- `agent:before_stop`, `agent:after_stop`

#### Tool Hooks (34 tools × 2)
- `tool:{name}:before_execution`
- `tool:{name}:after_execution`

#### Workflow Hooks (4 patterns × multiple points)
- Sequential: before/after each step
- Conditional: branch evaluation points
- Loop: iteration boundaries
- Parallel: fork/join points

#### State Hooks (Phase 5)
- `state:before_save`, `state:after_save`
- `state:before_load`, `state:after_load`
- `state:before_delete`, `state:after_delete`
- `state:before_migrate`, `state:after_migrate`

### Hook Result Types

```rust
pub enum HookResult {
    Continue,           // Normal execution
    Modified(Value),    // Modify input/output
    Cancel(String),     // Cancel with reason
    Redirect(String),   // Redirect to different operation
    Replace(Value),     // Replace entire result
    Retry(RetryConfig), // Retry with backoff
    Fork(Vec<Value>),   // Fork into parallel paths
    Cache(Duration),    // Cache result for duration
    Skipped,           // Skip this hook
}
```

### Performance Protection

The CircuitBreaker automatically disables slow hooks:
- Tracks execution time per hook
- Disables hooks exceeding threshold (default 50ms)
- Automatic recovery after cooldown period
- Maintains <2% total overhead guarantee

### Built-in Hooks

1. **LoggingHook**: Configurable logging with levels
2. **MetricsHook**: Comprehensive metrics collection
3. **DebuggingHook**: Enhanced debugging with traces
4. **SecurityHook**: Audit logging and validation
5. **CachingHook**: Automatic result caching
6. **RateLimitHook**: Token bucket rate limiting
7. **RetryHook**: Exponential backoff retry logic
8. **CostTrackingHook**: AI/ML cost monitoring
9. **PerformanceHook**: Execution time tracking
10. **ValidationHook**: Input/output validation
11. **TransformHook**: Data transformation
12. **NotificationHook**: Event notifications
13. **AuditHook**: Compliance logging
14. **FilterHook**: Request filtering
15. **EnrichmentHook**: Data enrichment
16. **CompressionHook**: Payload compression
17. **EncryptionHook**: Data encryption
18. **ThrottlingHook**: Request throttling

## Usage

### Basic Hook Registration

```rust
use llmspell_hooks::{HookManager, Hook, HookContext, HookResult};

// Create a custom hook
struct MyHook;

#[async_trait]
impl Hook for MyHook {
    async fn execute(&self, context: &mut HookContext) -> Result<HookResult> {
        println!("Hook executing for: {}", context.hook_point);
        Ok(HookResult::Continue)
    }
}

// Register the hook
let manager = HookManager::new();
manager.register("agent:before_execution", Box::new(MyHook), Priority::Normal)?;
```

### Using Built-in Hooks

```rust
use llmspell_hooks::builtin::{LoggingHook, MetricsHook, CachingHook};

// Configure logging hook
let logging = LoggingHook::new()
    .with_level(LogLevel::Debug)
    .with_filter(|ctx| ctx.component == "agent");

// Configure caching hook
let caching = CachingHook::new()
    .with_ttl(Duration::from_secs(300))
    .with_max_size(1000);

manager.register("tool:*:after_execution", Box::new(logging), Priority::High)?;
manager.register("agent:after_execution", Box::new(caching), Priority::Normal)?;
```

### Cross-Language Hooks (Lua)

```lua
-- Register a Lua hook
Hook.register("agent:before_execution", function(context)
    Logger.info("Agent starting", {agent_id = context.agent_id})
    return {continue_execution = true}
end)

-- List registered hooks
local hooks = Hook.list({
    language = "lua",
    hook_point = "agent:*"
})
```

### Composite Hook Patterns

```rust
use llmspell_hooks::composite::{SequentialHooks, ParallelHooks, VotingHooks};

// Sequential execution
let sequential = SequentialHooks::new(vec![
    Box::new(ValidationHook::new()),
    Box::new(LoggingHook::new()),
    Box::new(MetricsHook::new()),
]);

// Parallel execution
let parallel = ParallelHooks::new(vec![
    Box::new(NotificationHook::new()),
    Box::new(AuditHook::new()),
]);

// Voting pattern (majority wins)
let voting = VotingHooks::new(vec![
    Box::new(SecurityHook::new()),
    Box::new(FilterHook::new()),
    Box::new(ValidationHook::new()),
]);
```

### ReplayableHook for State Persistence

```rust
use llmspell_hooks::{ReplayableHook, HookReplay};

struct StatefulHook {
    counter: AtomicUsize,
}

impl ReplayableHook for StatefulHook {
    fn capture_state(&self) -> Result<Value> {
        Ok(json!({
            "counter": self.counter.load(Ordering::Relaxed)
        }))
    }
    
    fn replay(&mut self, state: Value) -> Result<()> {
        if let Some(counter) = state["counter"].as_u64() {
            self.counter.store(counter as usize, Ordering::Relaxed);
        }
        Ok(())
    }
}
```

## Performance

Achieved performance metrics (v0.5.0):

| Metric | Target | Actual |
|--------|--------|--------|
| Hook Registration | <0.1ms | ~0.46ms |
| Hook Execution Overhead | <5% | <2% |
| CircuitBreaker Response | <5ms | <2ms |
| Memory per Hook | <1KB | ~800B |
| Cross-Language Bridge | <10ms | <5ms |

## Architecture

The crate is organized into:

- **Core** (`manager.rs`, `registry.rs`) - Hook management and registration
- **Execution** (`executor.rs`, `context.rs`) - Hook execution engine
- **Protection** (`circuit_breaker.rs`) - Performance protection
- **Built-in** (`builtin/`) - Production-ready hooks
- **Composite** (`composite/`) - Hook composition patterns
- **Bridge** (`bridge/`) - Cross-language support
- **Replay** (`replay.rs`) - State persistence support

## Testing

```bash
# Run all tests
cargo test -p llmspell-hooks

# Run with performance monitoring
RUST_LOG=llmspell_hooks=debug cargo test

# Benchmark hook performance
cargo bench -p llmspell-hooks
```

## Dependencies

- `llmspell-core` - Core traits and types
- `llmspell-events` - Event system integration
- `llmspell-utils` - Shared utilities
- `tokio` - Async runtime
- `dashmap` - Concurrent collections
- `parking_lot` - Synchronization primitives

## License

This project is dual-licensed under MIT OR Apache-2.0.