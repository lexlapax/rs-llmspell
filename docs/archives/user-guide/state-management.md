# State Management Guide

## Overview

✅ **Fully Available** (In-Memory Only): Thread-safe storage for scripts and workflows. 📋 **Phase 5 Enhancement**: Persistent storage backend planned.

The State global provides thread-safe in-memory storage for your scripts and workflows. Use it to share data between different parts of your script, track workflow progress, or maintain temporary application state.

## Current Implementation

### Basic Usage

```lua
-- Store a value
State.set("key", "value")

-- Retrieve a value
local value = State.get("key")

-- Delete a value
State.delete("key")

-- List all keys
local keys = State.list()
```

### Working with Complex Data

The State global automatically handles JSON conversion:

```lua
-- Store a table
State.set("user_config", {
    name = "John",
    retries = 3,
    settings = {
        timeout = 30,
        verbose = true
    }
})

-- Retrieve and use
local config = State.get("user_config")
print("Retries: " .. config.retries)
```

### Workflow Integration

Use State to share data between workflow steps:

```lua
-- In one step
State.set("processed_data", {
    records = 1000,
    errors = 5,
    timestamp = os.time()
})

-- In another step
local data = State.get("processed_data")
if data and data.errors > 0 then
    -- Handle errors
end
```

## Best Practices

### State Design

1. **Minimize State**: Store only essential data
2. **Clear Naming**: Use descriptive keys
3. **Scope Keys**: Use prefixes for organization
4. **Clean Up**: Remove unused state

Example of good key naming:
```lua
-- Use prefixes for organization
State.set("workflow:data_pipeline:status", "running")
State.set("workflow:data_pipeline:progress", 0.5)
State.set("cache:api_response:user_123", userData)
```

### Concurrency

State is thread-safe and handles concurrent access:

```lua
-- Safe for parallel workflows
State.set("branch_1_complete", true)
State.set("branch_2_complete", true)

-- Check if all branches are done
local branch1 = State.get("branch_1_complete")
local branch2 = State.get("branch_2_complete")
if branch1 and branch2 then
    print("All branches complete")
end
```

### Error Handling

Always handle missing keys gracefully:

```lua
-- Safe pattern
local value = State.get("might_not_exist")
if value then
    -- Use value
else
    -- Handle missing key
    State.set("might_not_exist", "default_value")
end

-- With defaults
local retries = State.get("retry_count") or 0
```

## Common Patterns

### Counter Pattern
```lua
local function increment_counter(key)
    local count = State.get(key) or 0
    State.set(key, count + 1)
    return count + 1
end

-- Usage
local attempts = increment_counter("api_attempts")
```

### Cache Pattern
```lua
local function get_with_cache(key, fetch_function)
    local cached = State.get("cache:" .. key)
    if cached then
        return cached
    end
    
    local value = fetch_function()
    State.set("cache:" .. key, value)
    return value
end

-- Usage
local data = get_with_cache("user_123", function()
    return fetch_user_from_api(123)
end)
```

### Progress Tracking
```lua
local function track_progress(workflow_id, current, total)
    State.set("progress:" .. workflow_id, {
        current = current,
        total = total,
        percent = (current / total) * 100,
        timestamp = os.time()
    })
end

-- Usage
track_progress("import_job", 50, 100)
```

## Future Enhancements

### Phase 4 Integration
- Hook system integration for state change events
- Event emission on state updates
- State change triggers

### Phase 5 Features
- Persistent storage (data survives restarts)
- State versioning and history
- Distributed state sync
- Backup and restore

## Limitations

Current limitations (will be addressed in future phases):
- State is in-memory only (lost on restart)
- No built-in expiration/TTL
- No size limits enforced
- No state change notifications

## Migration Path

When persistent storage arrives in Phase 5:
- Same API will continue to work
- Add opt-in persistence configuration
- Automatic migration tools
- Gradual rollout with feature flags

Your existing State usage will continue to work without changes!