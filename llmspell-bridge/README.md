# llmspell-bridge

Script engine integration bridge providing seamless Lua and JavaScript access to rs-llmspell functionality with zero-configuration global APIs.

## Overview

This crate provides the bridge layer between script languages and rs-llmspell's Rust implementation, enabling:

- **Zero-Configuration Access**: Pre-injected global objects (Agent, Tool, Workflow, State, Hook, Event)
- **Synchronous API**: Transparent async-to-sync conversion for script compatibility
- **Cross-Language Support**: Consistent API across Lua and JavaScript
- **Performance Optimized**: <5ms injection overhead, <10ms execution overhead
- **State Management**: Full Phase 5 persistent state access from scripts
- **Hook & Event Integration**: Complete Phase 4 hook/event system access

## Features

### Global Objects

All scripts automatically have access to these globals:

```lua
-- Agent operations
local agent = Agent.create({
    model = "openai/gpt-4",
    system_prompt = "You are helpful"
})
local response = agent:execute({prompt = "Hello"})

-- Tool usage
local tool = Tool.get("web_search")
local results = tool:execute({query = "rs-llmspell"})

-- Workflow orchestration
local workflow = Workflow.sequential({
    name = "research_flow",
    steps = {
        {tool = "web_search", input = {query = "$input"}},
        {agent = agent, prompt = "Summarize: $step1.output"}
    }
})

-- State persistence (Phase 5)
State.save("agent:gpt-4", "history", messages)
local data = State.load("global", "config")
State.migrate({from_version = 1, to_version = 2})

-- Hook registration (Phase 4)
Hook.register("agent:before_execution", function(ctx)
    Logger.info("Starting", {id = ctx.agent_id})
    return {continue_execution = true}
end)

-- Event handling (Phase 4)
Event.subscribe("*.error", function(event)
    Alert.send("Error", event.payload)
end)
```

### Synchronous Wrappers

The bridge transparently converts Rust's async operations to synchronous for scripts:

```rust
// Rust async operation
pub async fn execute(&self, prompt: &str) -> Result<String>

// Becomes synchronous in Lua
local result = agent:execute({prompt = "Hello"})
```

### State Management API (Phase 5)

```lua
-- Save state with automatic persistence
State.save(scope, key, value)

-- Load state with optional default
local value = State.load(scope, key) or default_value

-- Delete state
State.delete(scope, key)

-- List keys in scope
local keys = State.list_keys(scope)

-- Perform migration
State.migrate({
    from_version = 1,
    to_version = 2,
    transformations = {
        {field = "old_field", transform = "copy", to = "new_field"},
        {field = "version", transform = "default", value = 2}
    }
})

-- Backup operations
local backup_id = State.backup({description = "Pre-update"})
State.restore(backup_id)
```

### Hook System Access (Phase 4)

```lua
-- Register hooks with priority
Hook.register("tool:*:before_execution", handler, "high")

-- Unregister hooks
Hook.unregister(hook_id)

-- List hooks with filtering
local hooks = Hook.list({
    hook_point = "agent:*",
    language = "lua",
    priority = "high"
})

-- Pause/resume hooks
Hook.pause(hook_id)
Hook.resume(hook_id)
```

### Event System Access (Phase 4)

```lua
-- Subscribe to events with patterns
local sub_id = Event.subscribe("workflow.*.completed", handler)

-- Emit custom events
Event.emit({
    event_type = "custom:milestone",
    payload = {progress = 0.75}
})

-- Unsubscribe
Event.unsubscribe(sub_id)

-- Get event statistics
local stats = Event.stats()
```

## Architecture

The bridge consists of several layers:

### 1. Language Bindings (`src/lua/`, `src/javascript/`)
- mlua for Lua 5.4 integration
- boa/quickjs for JavaScript support
- Language-specific type conversions

### 2. Global Injection (`src/lua/globals/`)
- `agent.rs` - Agent global with 23+ methods
- `tool.rs` - Tool discovery and execution
- `workflow.rs` - Workflow patterns
- `state.rs` - State persistence operations
- `hook.rs` - Hook registration and management
- `event.rs` - Event pub/sub system
- `logger.rs` - Structured logging
- `json.rs` - JSON utilities

### 3. Synchronous Utilities (`src/lua/sync_utils.rs`)
- `block_on_async()` - Efficient async-to-sync conversion
- Coroutine detection for proper yielding
- Minimal overhead execution

### 4. Type Conversion (`src/lua/conversions.rs`)
- Bidirectional Lua ↔ Rust conversions
- StateValue serialization
- Error propagation

## Performance

Achieved performance metrics (v0.5.0):

| Operation | Target | Actual |
|-----------|--------|--------|
| Global Injection | <5ms | 2-4ms |
| Method Call Overhead | <10ms | <5ms |
| State Operation | <5ms | <5ms |
| Hook Registration | <1ms | <0.5ms |
| Event Emission | <1ms | <0.8ms |
| Memory per Context | <5MB | 1.8MB |

## Usage Examples

### Complete Script Example

```lua
-- Initialize agent with state restoration
local agent_id = "research-bot"
local agent = Agent.create({
    name = agent_id,
    model = "anthropic/claude-3",
    system_prompt = "You are a research assistant"
})

-- Restore previous conversation
local history = State.load("agent:" .. agent_id, "history") or {}
if #history > 0 then
    Logger.info("Restored conversation", {messages = #history})
end

-- Set up monitoring
Hook.register("agent:after_execution", function(ctx)
    -- Update metrics
    local metrics = State.load("global", "metrics") or {}
    metrics.total_calls = (metrics.total_calls or 0) + 1
    State.save("global", "metrics", metrics)
    
    -- Emit event
    Event.emit({
        event_type = "agent:executed",
        payload = {
            agent_id = agent_id,
            duration_ms = ctx.duration_ms
        }
    })
end)

-- Main workflow
local workflow = Workflow.sequential({
    name = "research_workflow",
    steps = {
        {
            name = "search",
            tool = "web_search",
            input = {query = "$input"}
        },
        {
            name = "analyze",
            agent = agent,
            prompt = "Analyze these results: $search.output"
        },
        {
            name = "save",
            handler = function(ctx)
                -- Save results
                State.save("workflow:research", "last_result", ctx.analyze)
                return {success = true}
            end
        }
    }
})

-- Execute with error handling
local ok, result = pcall(function()
    return workflow:execute({input = "quantum computing advances"})
end)

if not ok then
    Logger.error("Workflow failed", {error = result})
    -- Restore from backup if available
    local backups = State.list_backups()
    if #backups > 0 then
        State.restore(backups[1].id)
    end
end
```

## Testing

```bash
# Run bridge tests
cargo test -p llmspell-bridge

# Test Lua integration
cargo test lua_integration

# Test JavaScript integration  
cargo test js_integration

# Benchmark performance
cargo bench -p llmspell-bridge
```

## Dependencies

- `llmspell-core` - Core traits and types
- `llmspell-agents` - Agent functionality
- `llmspell-tools` - Tool execution
- `llmspell-workflows` - Workflow patterns
- `llmspell-state-persistence` - State management
- `llmspell-hooks` - Hook system
- `llmspell-events` - Event system
- `llmspell-security` - Sandboxing
- `mlua` - Lua 5.4 bindings
- `boa_engine` - JavaScript engine

## License

This project is dual-licensed under MIT OR Apache-2.0.