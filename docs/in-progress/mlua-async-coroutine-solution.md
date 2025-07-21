# MLua Async/Coroutine Solution Design

**Version**: 1.0  
**Date**: 2025-07-21  
**Status**: Design & Implementation Plan  
**Issue**: Agent.createAsync timeout and coroutine errors  

## Executive Summary

The current Agent.createAsync implementation fails because it misunderstands how mlua's async functions work. This document provides a comprehensive solution based on deep research into mlua's async/coroutine architecture and Rust-Lua integration patterns.

## Problem Analysis

### Current Implementation Issues

1. **Misunderstood Future Handling**: The createAsync wrapper tries to repeatedly resume a coroutine containing an async function, but mlua async functions return Rust futures that need special handling
2. **Wrong Abstraction Level**: Attempting to handle async at the Lua level instead of properly integrating with Rust's async runtime
3. **Infinite Resume Loop**: The future is never properly polled, causing repeated yields that hit the max_resumes limit

### Root Cause

mlua's `create_async_function` creates functions that:
- Return Rust futures wrapped as Lua userdata
- Internally use `coroutine.yield()` to pause when futures aren't ready
- Require being called from within a coroutine context
- Need integration with a Rust async runtime (like Tokio) for proper polling

## Research Findings

### How mlua Async Works

From mlua documentation and GitHub issues:

1. **Async Function Internals**:
   ```rust
   // Simplified mlua async implementation
   while !future.is_ready() {
       coroutine.yield(POLL_PENDING);
   }
   return future.result();
   ```

2. **Required Usage Pattern**:
   ```lua
   -- Async functions MUST be called from coroutines
   local co = coroutine.wrap(async_function)
   local result = co()  -- First poll
   -- Result might be POLL_PENDING userdata
   ```

3. **Proper Polling**:
   - Need to detect mlua's internal POLL_PENDING value
   - Integrate with event loop for wake notifications
   - Resume coroutine when future is ready

## Solution Design

### Architecture Overview

```
┌─────────────────┐     ┌──────────────────┐     ┌─────────────────┐
│   Lua Script    │────▶│  Async Bridge    │────▶│  Rust Async     │
│                 │     │  (New Layer)     │     │  Runtime        │
│ Agent.create()  │     │                  │     │                 │
│ (sync-looking)  │     │ - Coroutine mgmt │     │ create_async_fn │
│                 │     │ - Future polling │     │ Tokio::spawn    │
│                 │     │ - Result caching │     │                 │
└─────────────────┘     └──────────────────┘     └─────────────────┘
```

### Implementation Strategy

#### Option 1: Synchronous Wrapper (Recommended for Phase 3.3)

Provide a synchronous interface to Lua that internally handles async:

```rust
// In agent.rs
let create_fn = lua.create_function(move |lua, args: Table| {
    let registry = registry_clone.clone();
    let providers = providers_clone.clone();
    
    // Use block_on to execute async code synchronously
    let runtime = tokio::runtime::Handle::current();
    let result = runtime.block_on(async {
        // ... existing async agent creation code ...
    });
    
    match result {
        Ok(agent) => Ok(LuaAgent::new(agent)),
        Err(e) => Err(mlua::Error::RuntimeError(e.to_string()))
    }
})?;
```

**Pros**:
- Simple to implement
- No coroutine complexity in Lua
- Works immediately
- Matches user expectations

**Cons**:
- Blocks Lua thread during async operations
- Not suitable for streaming/long-running operations

#### Option 2: Proper Async Integration (Future Enhancement)

Implement full async support with polling:

```rust
// Create async wrapper that exposes polling
let create_async_fn = lua.create_async_function(move |lua, args: Table| {
    // ... async creation ...
})?;

// Create poll helper
let poll_pending = lua.create_function(|_, ()| {
    Ok(mlua::Value::UserData(/* mlua's POLL_PENDING */))
})?;

// Lua-side polling wrapper
let create_with_polling = lua.load(r#"
    local poll_pending = ...
    return function(config)
        local co = coroutine.wrap(Agent.createRaw)
        local result = co(config)
        
        -- Poll until ready
        while result == poll_pending do
            coroutine.yield()  -- Let other work happen
            result = co()
        end
        
        return result
    end
"#).call(poll_pending)?;
```

#### Option 3: Event Loop Integration (Advanced)

For environments with event loops (like Neovim):

```lua
-- Integration with vim.loop or similar
local function create_agent_async(config, callback)
    local co = coroutine.wrap(Agent.createRaw)
    local function poll()
        local result = co(config)
        if result == poll_pending then
            vim.defer_fn(poll, 0)  -- Schedule next poll
        else
            callback(result)
        end
    end
    poll()
end
```

## Implementation Plan

### Phase 1: Immediate Fix (1 day)

1. **Replace async function with sync wrapper**:
   ```rust
   // Change from create_async_function to create_function
   // Use tokio::runtime::Handle::block_on internally
   ```

2. **Remove createAsync Lua wrapper**:
   - Delete the coroutine-based createAsync implementation
   - Make Agent.create the primary API

3. **Update examples**:
   - Remove Agent.createAsync usage
   - Use Agent.create directly

### Phase 2: Test & Validate (1 day)

1. **Test all agent examples**
2. **Verify provider integration works**
3. **Check error handling**
4. **Performance validation**

### Phase 3: Documentation (0.5 days)

1. **Update API documentation**
2. **Add notes about sync behavior**
3. **Document future async roadmap**

### Phase 4: Future Async Support (Post-MVP)

1. **Design proper async API**:
   - Agent.createAsync with callback
   - Agent.createPromise for Promise-based API
   - Streaming support

2. **Implement polling infrastructure**
3. **Add event loop integrations**

## Code Changes

### 1. Agent API (agent.rs)

```rust
// Before
let create_fn = lua.create_async_function(move |_lua, args: Table| {
    // async code
})?;

// After  
let create_fn = lua.create_function(move |lua, args: Table| {
    let runtime = tokio::runtime::Handle::current();
    runtime.block_on(async {
        // existing async code
    })
})?;
```

### 2. Remove createAsync Helper

```rust
// Delete lines 768-814 in agent.rs (the createAsync implementation)
```

### 3. Update Agent Execute Method

```rust
// Similar change for agent:execute method
impl UserData for LuaAgent {
    fn add_methods<'lua, M: UserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_method("execute", |_, this, input: Table| {
            let runtime = tokio::runtime::Handle::current();
            runtime.block_on(async {
                // async execute code
            })
        });
    }
}
```

## Migration Guide

### For Examples

```lua
-- Before
local agent = Agent.createAsync({
    model = "gpt-4o-mini",
    prompt = "Hello"
})

-- After
local agent = Agent.create({
    model = "gpt-4o-mini", 
    prompt = "Hello"
})
```

### For Tests

Update all tests to use Agent.create directly without worrying about coroutine context.

## Future Enhancements

### 1. True Async Support (Post-MVP)

When proper async is needed:
- Implement callback-based API
- Add Promise/Future patterns
- Support streaming responses

### 2. Event Loop Integrations

- Neovim: vim.loop integration
- Node.js: libuv integration  
- Custom: User-provided event loops

### 3. Advanced Patterns

- Async iterators for streaming
- Concurrent agent operations
- Progress callbacks

## Risk Assessment

### Low Risk
- Synchronous wrapper is battle-tested pattern
- Similar to how many Lua libraries handle async
- Maintains API compatibility

### Medium Risk  
- Performance impact for long operations
- Need careful timeout handling

### Mitigation
- Add configurable timeouts
- Document blocking behavior
- Plan async roadmap

## Success Criteria

1. **All agent examples run without errors**
2. **No more "attempt to yield from outside coroutine"**
3. **No more timeout errors**
4. **API remains simple and intuitive**
5. **Performance acceptable for typical use cases**

## Conclusion

The synchronous wrapper approach provides an immediate, robust solution that matches user expectations while avoiding coroutine complexity. This unblocks MVP completion while leaving room for proper async support in future phases.

The key insight is that forcing async patterns onto Lua users creates more problems than it solves. A clean, synchronous API that "just works" is preferable for most use cases, with opt-in async for advanced scenarios.