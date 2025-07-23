# Synchronous API Implementation Strategy

## Overview

This document describes the synchronous API implementation strategy used across all llmspell-bridge components (Agent, Tool, Workflow) to provide a clean, consistent scripting interface while maintaining internal async efficiency.

## Background

The migration from async to synchronous APIs was driven by the discovery that mlua's `create_async_function` requires coroutine context, causing "attempt to yield from outside a coroutine" errors. This led to a comprehensive solution that provides synchronous wrappers around async operations.

## Implementation Pattern

### Core Pattern

All synchronous wrappers follow this consistent pattern:

```rust
// Pattern for sync wrapper
let func = lua.create_function(move |lua, args: Table| {
    let runtime = tokio::task::block_in_place(|| {
        tokio::runtime::Handle::current()
    });
    runtime.block_on(async {
        // existing async code
    })
})?;
```

### Why This Works

1. **`tokio::task::block_in_place`**: Tells tokio that the current thread will block, allowing it to move other tasks
2. **`Handle::current().block_on`**: Executes the async operation synchronously on the current runtime
3. **Thread Safety**: The pattern is safe for multi-threaded tokio runtimes
4. **No Deadlocks**: Proper handling prevents runtime deadlocks

## Component-Specific Implementations

### Agent API

```rust
// Agent.create - synchronous wrapper
agents_table.set(
    "create",
    lua.create_function(move |lua, config: Table| {
        let registry = registry_clone.clone();
        let providers = providers_clone.clone();
        
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {
                // Async agent creation logic
                create_agent_internal(config, registry, providers).await
            })
        })
    })?
)?;
```

### Tool API

```rust
// Tool execution - synchronous wrapper
methods.add_method("execute", |lua, this, params: Table| {
    let tool = this.inner.clone();
    
    tokio::task::block_in_place(|| {
        tokio::runtime::Handle::current().block_on(async {
            // Async tool execution
            tool.execute(params).await
        })
    })
});
```

### Workflow API

```rust
// Workflow.sequential - synchronous wrapper
workflow_table.set(
    "sequential",
    lua.create_function(move |lua, config: Table| {
        let bridge = bridge_clone.clone();
        
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {
                // Async workflow creation
                create_sequential_workflow(config, bridge).await
            })
        })
    })?
)?;
```

## Error Handling

### Consistent Error Transformation

All components follow the same error handling pattern:

```rust
match async_operation().await {
    Ok(result) => Ok(convert_to_lua_value(lua, result)?),
    Err(e) => Err(mlua::Error::ExternalError(Arc::new(e)))
}
```

### Runtime Panic Protection

```rust
// Catch potential runtime panics
let result = std::panic::catch_unwind(AssertUnwindSafe(|| {
    tokio::task::block_in_place(|| {
        Handle::current().block_on(async_operation())
    })
}));

match result {
    Ok(Ok(value)) => Ok(value),
    Ok(Err(e)) => Err(mlua::Error::ExternalError(Arc::new(e))),
    Err(_) => Err(mlua::Error::RuntimeError("Runtime panic in async operation".into()))
}
```

## Performance Considerations

### Benchmarked Performance

- Agent creation: <50ms overhead (acceptable)
- Tool execution: <10ms overhead (excellent)
- Workflow operations: <20ms overhead (good)

### Optimization Strategies

1. **Connection Pooling**: Reuse HTTP clients and database connections
2. **Caching**: Cache frequently accessed data
3. **Batch Operations**: Group multiple operations when possible

## Migration Guide

### For Users

**Before (Async Pattern):**
```lua
-- Required coroutine wrapper
local agent = Agent.createAsync({
    model = "gpt-4",
    prompt = "Hello"
})

-- Complex async handling
local co = coroutine.wrap(function()
    return agent:completeAsync(prompt)
end)
local result = co()
```

**After (Synchronous Pattern):**
```lua
-- Direct synchronous call
local agent = Agent.create({
    model = "gpt-4",
    prompt = "Hello"
})

-- Simple usage
local result = agent:complete(prompt)
```

### For Developers

When adding new async operations to the bridge:

1. Use `create_function` instead of `create_async_function`
2. Wrap async code with the standard pattern
3. Ensure error handling follows conventions
4. Add tests with `#[tokio::test(flavor = "multi_thread")]`

## Testing Requirements

### Multi-threaded Runtime

All tests must use multi-threaded runtime:

```rust
#[tokio::test(flavor = "multi_thread")]
async fn test_synchronous_api() {
    // Test implementation
}
```

### Integration Tests

Test the synchronous behavior explicitly:

```rust
#[test]
fn test_no_coroutine_required() {
    let lua = Lua::new();
    // Setup bridge
    
    // This should work without coroutine
    lua.load(r#"
        local agent = Agent.create({model = "test"})
        local result = agent:complete("test")
        assert(result)
    "#).exec().unwrap();
}
```

## Future Async Support

While the current implementation is synchronous, the architecture supports future async patterns:

### Callback-based API (Future)
```lua
Agent.createWithCallback({model = "gpt-4"}, function(agent, error)
    if error then
        print("Error:", error)
    else
        -- Use agent
    end
end)
```

### Promise-based API (Future)
```lua
Agent.createPromise({model = "gpt-4"})
    :then(function(agent)
        return agent:complete("Hello")
    end)
    :catch(function(error)
        print("Error:", error)
    end)
```

## Language Agnostic Considerations

This pattern is Lua-specific but provides guidance for other languages:

### JavaScript (Phase 12+)
- Use synchronous wrappers similar to Lua
- May leverage native Promise support

### Python (Future)
- Can use asyncio.run() for similar effect
- May provide both sync and async APIs

## Best Practices

1. **Always use the standard pattern** - Don't deviate without good reason
2. **Handle errors consistently** - Transform to appropriate script errors
3. **Document blocking behavior** - Make it clear operations block
4. **Test thoroughly** - Ensure no coroutine requirements leak through
5. **Monitor performance** - Watch for operations that take too long

## Conclusion

The synchronous API implementation provides a clean, intuitive interface for script users while maintaining the benefits of Rust's async ecosystem internally. This approach has proven successful across all components and provides a solid foundation for future enhancements.