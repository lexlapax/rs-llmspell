# llmspell-hooks

**Lifecycle hooks and execution interception**

**🔗 Navigation**: [← Rust API](README.md) | [Crate Docs](https://docs.rs/llmspell-hooks) | [Source](../../../../llmspell-hooks)

---

## Overview

`llmspell-hooks` provides lifecycle hooks for intercepting and modifying execution flow, enabling logging, monitoring, security checks, and custom behaviors.

**Key Features:**
- 🎣 Lifecycle interception points
- 📝 Pre/post execution hooks
- 🔄 Hook chains and priorities
- 🎯 Selective hook registration
- 📊 Hook metrics
- 🔐 Security hooks
- 💾 Hook replay
- ⚡ Async hook execution

## Hook Types

```rust
pub enum HookPoint {
    // Agent hooks
    BeforeAgentExecution,
    AfterAgentExecution,
    OnAgentError,
    
    // Tool hooks
    BeforeToolInvocation,
    AfterToolInvocation,
    OnToolError,
    
    // Workflow hooks
    BeforeWorkflowStart,
    AfterWorkflowComplete,
    BeforeStep,
    AfterStep,
    
    // State hooks
    BeforeStateRead,
    AfterStateWrite,
    
    // Session hooks
    OnSessionCreate,
    OnSessionEnd,
}
```

## Hook Registration

```rust
use llmspell_hooks::{HookManager, Hook, HookContext};

let hook_manager = HookManager::new();

// Register a simple hook
hook_manager.register(
    HookPoint::BeforeAgentExecution,
    Hook::new("logger", |context: HookContext| {
        println!("Agent executing: {:?}", context.data["agent_name"]);
        Ok(HookResult::Continue)
    }),
)?;

// Register with priority
hook_manager.register_with_priority(
    HookPoint::BeforeToolInvocation,
    Hook::new("security", security_check),
    Priority::High,
)?;
```

## Hook Context

```rust
pub struct HookContext {
    pub hook_point: HookPoint,
    pub data: HashMap<String, Value>,
    pub metadata: HookMetadata,
    pub state: Arc<RwLock<HashMap<String, Value>>>,
}

pub enum HookResult {
    Continue,                    // Continue execution
    Skip,                        // Skip the operation
    Modify(Value),              // Modify input/output
    Abort(String),              // Abort with error
    Redirect(String, Value),    // Redirect to different operation
}
```

## Common Hook Patterns

### Logging Hook
```rust
let logging_hook = Hook::new("logger", |context| {
    info!("Operation: {:?}, Data: {:?}", 
        context.hook_point, context.data);
    Ok(HookResult::Continue)
});
```

### Rate Limiting Hook
```rust
let rate_limit_hook = Hook::new("rate_limiter", |context| {
    let key = format!("{}:{}", 
        context.data["user_id"], 
        context.data["operation"]);
    
    if rate_limiter.check(&key)? {
        Ok(HookResult::Continue)
    } else {
        Ok(HookResult::Abort("Rate limit exceeded".to_string()))
    }
});
```

### Data Transformation Hook
```rust
let transform_hook = Hook::new("transformer", |context| {
    let mut data = context.data["input"].clone();
    data["timestamp"] = json!(SystemTime::now());
    Ok(HookResult::Modify(data))
});
```

## Hook Replay

```rust
use llmspell_hooks::HookReplay;

// Record hooks during execution
let recorder = HookRecorder::new();
hook_manager.set_recorder(Some(recorder));

// Execute operations...

// Replay hooks later
let replay = HookReplay::from_recording(recorder.get_recording());
replay.replay_all().await?;
```

## Related Documentation

- [llmspell-events](llmspell-events.md) - Event system integration
- [llmspell-sessions](llmspell-sessions.md) - Session lifecycle hooks