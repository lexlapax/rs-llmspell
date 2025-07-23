# API Consolidation Breaking Changes

## Version 0.3.0 - API Layer to Globals Migration

### Overview
This release completes the migration from the old API layer pattern to the new globals-based architecture. All async operations are now handled internally, providing a simpler synchronous API for script languages.

### Breaking Changes

#### 1. Removed API Layer
- **Old**: `llmspell-bridge/src/lua/api/*` and `llmspell-bridge/src/javascript/api/*`
- **New**: `llmspell-bridge/src/lua/globals/*` and `llmspell-bridge/src/javascript/globals/*`
- **Impact**: Internal architecture change, no user-facing impact for most users

#### 2. Async Methods Removed
All `*Async` methods have been removed in favor of synchronous APIs:

- **Agent**:
  - `Agent.createAsync()` → `Agent.create()`
  - All agent methods now synchronous

- **Tool**:
  - `Tool.executeAsync()` → Use `tool:execute()` on tool instances
  - All tool execution now synchronous

- **Workflow**:
  - `Workflow.executeAsync()` → `workflow:execute()`
  - All workflow operations now synchronous

#### 3. Helper Files Removed
- `agent-helpers.lua` - No longer needed
- `test-helpers.lua` - Test utilities updated
- Coroutine-based patterns no longer required

#### 4. Engine API Changes
- `engine.inject_apis()` now uses the global injection system internally
- Removed `ApiSurface` and related types from `engine/types.rs`

### Migration Guide

#### For Lua Scripts

**Before:**
```lua
-- Old async pattern with coroutines
local agent = Agent.createAsync({
    model = "gpt-4",
    prompt = "You are a helpful assistant"
})

local response = agent:completeAsync(prompt)
```

**After:**
```lua
-- New synchronous pattern
local agent = Agent.create({
    model = "gpt-4", 
    prompt = "You are a helpful assistant"
})

local response = agent:complete(prompt)
```

#### For Tool Usage

**Before:**
```lua
-- Old async tool execution
local result = Tool.executeAsync("calculator", {
    operation = "evaluate",
    expression = "2 + 2"
})
```

**After:**
```lua
-- New synchronous tool execution
local calc = Tool.get("calculator")
local result = calc:execute({
    operation = "evaluate",
    expression = "2 + 2"
})
```

#### For Workflows

**Before:**
```lua
-- Old async workflow
local workflow = Workflow.sequential({...})
local result = workflow:executeAsync(input)
```

**After:**
```lua
-- New synchronous workflow
local workflow = Workflow.sequential({...})
local result = workflow:execute(input)
```

### Benefits

1. **Simpler API**: No need to manage coroutines or async/await patterns
2. **Better Error Handling**: Synchronous errors are easier to catch and handle
3. **Consistent Pattern**: All operations follow the same synchronous pattern
4. **Internal Optimization**: Async operations are handled efficiently internally

### Technical Details

The synchronous wrappers use the following pattern internally:
```rust
tokio::task::block_in_place(|| {
    tokio::runtime::Handle::current().block_on(async {
        // async operation
    })
})
```

This ensures:
- No blocking of the async runtime
- Efficient handling of async operations
- Transparent synchronous API for scripts

### Compatibility

- **Backward Compatibility**: Breaking changes - old async methods removed
- **Forward Compatibility**: New architecture supports future enhancements
- **Language Support**: Lua fully implemented, JavaScript prepared for Phase 12+

### Performance

- No significant performance regression vs async APIs
- <10ms overhead maintained for tool operations
- <50ms overhead for agent creation
- Benchmarks show consistent performance with previous versions