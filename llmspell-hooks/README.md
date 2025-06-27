# llmspell-hooks

Hook and event system for Rs-LLMSpell.

## Features
- 20+ hook points throughout execution lifecycle
- Async event bus for real-time coordination
- Built-in hooks for logging, metrics, and debugging

## Usage
```rust
use llmspell_hooks::{HookManager, HookPoint, EventBus};

let hooks = HookManager::new()
    .register(HookPoint::BeforeExecution, logging_hook)
    .register(HookPoint::AfterCompletion, metrics_hook);

let event_bus = EventBus::new();
event_bus.subscribe("agent.started", handler).await;
```

## Dependencies
- `llmspell-core` - Hook trait definitions
- External: `tokio-stream`, `crossbeam`
- `llmspell-config` - Hook configuration