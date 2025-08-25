# Hooks and Events Overview

## Introduction

rs-llmspell provides two powerful extensibility mechanisms: **Hooks** and **Events**. While they may seem similar at first, they serve distinct purposes and are optimized for different use cases.

- **Hooks**: Synchronous interception points that can modify behavior
- **Events**: Asynchronous notifications for loose coupling and monitoring

## Quick Comparison

| Feature | Hooks | Events |
|---------|-------|--------|
| **Execution** | Synchronous | Asynchronous |
| **Can modify behavior** | Yes | No |
| **Performance impact** | Direct (<5% overhead) | Minimal |
| **Use case** | Intercept and modify | Monitor and react |
| **Return value** | HookResult (9 types) | None |
| **Cross-language** | Yes | Yes |
| **Ordering** | Priority-based | Pattern-based |
| **Backpressure** | CircuitBreaker | FlowController |

## When to Use Hooks

Use hooks when you need to:
- **Modify behavior**: Change input parameters or output results
- **Cancel operations**: Prevent execution based on conditions
- **Add security**: Validate inputs before processing
- **Implement caching**: Short-circuit with cached results
- **Add retry logic**: Automatically retry on failures
- **Track costs**: Monitor LLM token usage in real-time

### Hook Example
```lua
-- Register a hook to validate agent inputs
Hook.register("BeforeAgentExecution", function(context)
    local input = context.data.input
    
    -- Validate input length
    if #input.text > 1000 then
        return {
            action = "modify",
            modified_data = {
                input = {text = string.sub(input.text, 1, 1000)}
            }
        }
    end
    
    return "continue"
end, "high")
```

## When to Use Events

Use events when you need to:
- **Monitor activity**: Track what's happening without interfering
- **Loose coupling**: Connect components without direct dependencies
- **Audit logging**: Record actions for compliance
- **Analytics**: Collect metrics and statistics
- **Notifications**: Alert on specific conditions
- **Cross-language communication**: Share data between Lua, JavaScript, and Rust

### Event Example
```lua
-- Subscribe to agent completion events
local subscription = Event.subscribe("agent.execution.completed")

-- Publish an event
Event.publish("custom.analysis.complete", {
    agent = "analyzer",
    tokens_used = 150,
    duration_ms = 234
})

-- Receive events
local event = Event.receive(subscription, 1000) -- 1 second timeout
if event then
    print("Agent completed:", event.data.agent)
end
```

## Architecture Overview

```
┌─────────────────────────────────────────────────────────────┐
│                     User Scripts (Lua/JS)                   │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  Hooks System                    Events System              │
│  ┌─────────────┐                ┌─────────────┐           │
│  │ Hook Points │                │ Event Bus   │           │
│  │ (40+ types) │                │ (100K+ /sec)│           │
│  └──────┬──────┘                └──────┬──────┘           │
│         │                               │                   │
│  ┌──────▼──────┐                ┌──────▼──────┐           │
│  │ HookExecutor│                │FlowController│           │
│  │ +CircuitBrkr│                │ +Backpressure│           │
│  └──────┬──────┘                └──────┬──────┘           │
│         │                               │                   │
├─────────┴───────────────────────────────┴───────────────────┤
│              Core System (Agents, Tools, Workflows)         │
└─────────────────────────────────────────────────────────────┘
```

## Integration Points

Both hooks and events integrate at key points throughout the system:

### Agent Lifecycle
- **Hooks**: BeforeAgentInit, BeforeAgentExecution, AfterAgentExecution, AgentError
- **Events**: agent.created, agent.executed, agent.error, agent.shutdown

### Tool Execution
- **Hooks**: BeforeToolExecution, AfterToolExecution, ToolValidation, ToolError
- **Events**: tool.invoked, tool.completed, tool.failed

### Workflow Stages
- **Hooks**: BeforeWorkflowStage, AfterWorkflowStage, WorkflowCheckpoint
- **Events**: workflow.started, workflow.stage.*, workflow.completed

## Performance Characteristics

### Hooks
- **Overhead**: <5% guaranteed by CircuitBreaker
- **Slow hook protection**: Automatic disabling after 5 consecutive slow executions
- **Priority execution**: Highest → High → Normal → Low → Lowest
- **Batching**: Multiple hooks execute in order

### Events
- **Throughput**: 90,000+ events/second
- **Pattern matching**: Glob patterns with minimal overhead
- **Backpressure**: Four strategies (DropOldest, DropNewest, Block, Reject)
- **Persistence**: Optional with storage backend

## Cross-Language Support

Both systems work seamlessly across languages:

```lua
-- Lua: Register a hook
Hook.register("BeforeAgentExecution", myHook, "high")

-- Lua: Publish an event
Event.publish("my.custom.event", {data = "value"})
```

```javascript
// JavaScript (Phase 15): Same API
Hook.register("BeforeAgentExecution", myHook, "high");
Event.publish("my.custom.event", {data: "value"});
```

```rust
// Rust: Native implementation
hook_registry.register(
    HookPoint::BeforeAgentExecution,
    Arc::new(my_hook),
)?;
```

## Quick Start Examples

### 1. Simple Monitoring Hook
```lua
-- Count agent executions
local agent_count = 0

Hook.register("BeforeAgentExecution", function(context)
    agent_count = agent_count + 1
    print("Agent execution #" .. agent_count)
    return "continue"
end)
```

### 2. Event-Driven Workflow
```lua
-- Subscribe to multiple event patterns
local subs = {
    errors = Event.subscribe("*.error"),
    completions = Event.subscribe("*.completed")
}

-- Process events in a loop
while true do
    local error_event = Event.receive(subs.errors, 100)
    if error_event then
        print("Error detected:", error_event.event_type)
        -- Take corrective action
    end
    
    local completion = Event.receive(subs.completions, 100)
    if completion then
        print("Task completed:", completion.event_type)
    end
end
```

### 3. Performance Protection
```lua
-- This hook will be automatically disabled if it's too slow
Hook.register("BeforeToolExecution", function(context)
    -- Simulate slow operation
    os.execute("sleep 0.1")  -- 100ms delay
    return "continue"
end)

-- After 5 slow executions, CircuitBreaker will disable this hook
```

## Best Practices

### Hooks
1. **Keep hooks fast**: Aim for <10ms execution time
2. **Use appropriate priority**: Reserve "highest" for critical security checks
3. **Handle errors gracefully**: Don't let hook errors break the system
4. **Test with CircuitBreaker**: Verify your hooks work under protection

### Events
1. **Use patterns wisely**: More specific patterns = better performance
2. **Set appropriate timeouts**: Don't wait forever for events
3. **Clean up subscriptions**: Always unsubscribe when done
4. **Consider backpressure**: Choose the right overflow strategy

## Next Steps

- **[Hooks Guide](./hooks-guide.md)**: Deep dive into the hook system
- **[Events Guide](./events-guide.md)**: Master the event system
- **[Built-in Hooks Reference](./builtin-hooks-reference.md)**: Use production-ready hooks
- **[Hook Patterns](./hook-patterns.md)**: Common patterns and recipes
- **[Examples](./examples/hooks-events-cookbook.md)**: 23 working examples

## Summary

- **Hooks** are for modifying behavior synchronously with guaranteed performance
- **Events** are for monitoring and loose coupling with high throughput
- Both support cross-language integration
- Both have automatic performance protection
- Together they provide comprehensive extensibility for rs-llmspell