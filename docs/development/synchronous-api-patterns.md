# Synchronous API Patterns for rs-llmspell

This guide explains the synchronous wrapper patterns used in rs-llmspell to provide a synchronous Lua/JavaScript API over asynchronous Rust operations.

## Overview

While rs-llmspell's core operations (LLM calls, tool execution, workflows) are inherently asynchronous in Rust, the Lua and JavaScript scripting APIs present a synchronous interface. This is achieved through careful use of synchronous wrappers that properly handle the async-to-sync bridge.

## Core Pattern: The sync_utils Module

All synchronous wrapping is centralized in the `llmspell-bridge/src/lua/sync_utils.rs` module, which provides two main functions:

### 1. `block_on_async<F, T, E>`

For general async operations that return `Result<T, E>`:

```rust
pub fn block_on_async<F, T, E>(
    operation_name: &str,
    future: F,
    timeout: Option<Duration>,
) -> LuaResult<T>
where
    F: Future<Output = Result<T, E>>,
    E: std::error::Error + Send + Sync + 'static,
```

Key features:
- Panic safety with `catch_unwind`
- Optional timeout support
- Proper error transformation from `E` to `mlua::Error`
- Requires multi-threaded tokio runtime

### 2. `block_on_async_lua<'lua, F>`

For operations that already return `LuaResult<LuaValue>`:

```rust
pub fn block_on_async_lua<'lua, F>(
    operation_name: &str,
    future: F,
    timeout: Option<Duration>,
) -> LuaResult<LuaValue<'lua>>
where
    F: Future<Output = LuaResult<LuaValue<'lua>>>,
```

## Usage Examples

### Agent Creation

```rust
// In lua/globals/agent.rs
let result = block_on_async::<_, AgentInstance, LLMSpellError>(
    "agent_create",
    async move {
        bridge.create_agent(model_spec, config_json).await
    },
    None,
)?;
```

### Tool Execution

```rust
// In lua/globals/tool.rs
let result = block_on_async_lua(
    &format!("tool_execute_{}", tool_name),
    async move {
        // Tool execution logic
    },
    timeout,
)?;
```

### Workflow Operations

```rust
// In lua/globals/workflow.rs
let result = block_on_async::<_, WorkflowInstance, LLMSpellError>(
    "workflow_create_sequential",
    async move {
        // Convert Lua errors to LLMSpellError::Script
        let name = config.get("name").map_err(|e| LLMSpellError::Script {
            message: format!("Failed to get workflow name: {}", e),
            language: Some("lua".to_string()),
            line: None,
            source: None,
        })?;
        
        // Create workflow
        bridge.create_workflow("sequential", params).await
    },
    None,
)?;
```

## Error Handling Patterns

### Converting Lua Errors

When mixing Lua operations (like `table.get()`) with async Rust operations, convert `mlua::Error` to `LLMSpellError::Script`:

```rust
let value = lua_table.get("key").map_err(|e| LLMSpellError::Script {
    message: format!("Failed to get key: {}", e),
    language: Some("lua".to_string()),
    line: None,
    source: None,
})?;
```

### Handling Non-Result Types

For async operations that don't return Result, wrap them:

```rust
let workflows = block_on_async::<_, Vec<(String, WorkflowInfo)>, LLMSpellError>(
    "workflow_list",
    async move { 
        Ok::<Vec<(String, WorkflowInfo)>, LLMSpellError>(
            bridge.list_workflows().await
        )
    },
    None,
)?;
```

## Consistent Error Messages

All sync wrappers use consistent error messaging:

- **No runtime**: `"No async runtime available for {operation}: {error}"`
- **Timeout**: `"{operation} timed out after {duration:?}"`
- **Panic**: `"Runtime panic in {operation}: operation failed unexpectedly"`

## Testing Synchronous Behavior

Tests should verify:

1. **Immediate return**: Operations block and return values, not promises
2. **Error propagation**: Errors are thrown synchronously
3. **No async artifacts**: Results don't have `.then()` or callback methods

Example test:

```lua
-- Should be able to use result immediately
local agent = Agent.create({name = "test", model = "gpt-4"})
assert(agent.id, "Agent should have ID immediately")

-- Errors should be thrown synchronously
local success, err = pcall(function()
    return Agent.create({}) -- Missing required fields
end)
assert(not success, "Should throw error synchronously")
```

## Best Practices

1. **Always use sync_utils**: Don't create ad-hoc `block_on` patterns
2. **Handle mixed error types**: Convert all errors to a common type before async blocks
3. **Provide operation names**: Use descriptive names for debugging and logging
4. **Consider timeouts**: Add timeout support for long-running operations
5. **Test on multi-threaded runtime**: Use `#[tokio::test(flavor = "multi_thread")]`

## Common Pitfalls

1. **Type inference issues**: Explicitly specify generic parameters when needed:
   ```rust
   block_on_async::<_, MyType, LLMSpellError>(...)
   ```

2. **Mixed error types**: Convert all errors in async block to avoid inference issues

3. **Moved values**: Clone Arc-wrapped values before moving into async blocks:
   ```rust
   let bridge_clone = bridge.clone();
   async move { bridge_clone.do_something().await }
   ```

## Migration Guide

When updating code to use the shared sync utilities:

1. Identify all uses of `tokio::task::block_in_place` or `catch_unwind`
2. Replace with appropriate `block_on_async` variant
3. Ensure error types are properly converted
4. Update tests to use multi-threaded runtime
5. Verify consistent error messages

## Future Considerations

As rs-llmspell evolves, consider:

- Adding metrics/tracing to sync operations
- Implementing configurable default timeouts
- Supporting cancellation tokens
- Providing async variants of the scripting APIs