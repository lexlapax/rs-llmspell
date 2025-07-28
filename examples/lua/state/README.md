# State API Examples

This directory contains examples demonstrating the State persistence API in rs-llmspell. The State API allows scripts to persist data across executions, share state between scripts, and implement advanced patterns like migrations and backups.

## Quick Reference

### Basic State Operations

```lua
-- Save data
State.save("namespace", "key", value)

-- Load data
local value = State.load("namespace", "key")

-- Delete data
State.delete("namespace", "key")

-- List keys (when available)
local keys = State.list_keys("namespace")
```

### Available Examples

| Example | Description | Config Required |
|---------|-------------|-----------------|
| `test_state_api.lua` | Tests which State API functions are available | `state-enabled.toml` |
| `basic_persistence.lua` | Core State API: save, load, delete | `state-enabled.toml` |
| `scope_isolation.lua` | How scopes provide isolation | `state-enabled.toml` |
| `agent_state.lua` | Agent-specific state management patterns | `state-enabled.toml` |
| `list_operations.lua` | Working without State.list_keys() | `state-enabled.toml` |
| `state_migration.lua` | Schema evolution and migration patterns | `migration-enabled.toml` |

## Running Examples

All examples require specific configuration files to enable State features:

```bash
# Basic state operations
./target/debug/llmspell -c examples/configs/state-enabled.toml run examples/lua/state/test_state_api.lua

# Migration examples (requires migration features)
./target/debug/llmspell -c examples/configs/migration-enabled.toml run examples/lua/state/state_migration.lua

# Or using cargo run
cargo run -- -c examples/configs/state-enabled.toml run examples/lua/state/agent_state.lua
```

## Configuration Files

### Basic State Configuration (`state-enabled.toml`)
Enables basic State operations: save, load, delete.

```toml
[state]
enabled = true
backend = "disk"
path = "./state"
```

### Migration Configuration (`migration-enabled.toml`)
Adds migration capabilities: migrate, schema_versions.

```toml
[state]
enabled = true
backend = "disk"
path = "./state"
migration_enabled = true
schema_version = "1.0.0"
```

### Backup Configuration (`backup-enabled.toml`)
Adds backup features: create_backup, restore_backup, list_backups.

```toml
[state]
enabled = true
backend = "disk"
path = "./state"
backup_enabled = true
backup_retention_days = 30
```

## Common Patterns

### 1. Configuration Management
```lua
-- Save application config
State.save("app", "config", {
    theme = "dark",
    language = "en",
    features = {
        autosave = true,
        notifications = true
    }
})

-- Load with defaults
local config = State.load("app", "config") or {
    theme = "light",
    language = "en"
}
```

### 2. User Preferences
```lua
-- Save user-specific data
State.save("user", "user123:preferences", {
    display_name = "John Doe",
    email_notifications = true,
    timezone = "UTC"
})

-- Load user data
local prefs = State.load("user", "user123:preferences")
```

### 3. Session Management
```lua
-- Create session
local session_id = os.time() .. "_" .. math.random(10000)
State.save("session", session_id, {
    user_id = "user123",
    created_at = os.time(),
    expires_at = os.time() + 3600
})

-- Check session validity
local session = State.load("session", session_id)
if session and session.expires_at > os.time() then
    -- Session is valid
else
    -- Session expired
    State.delete("session", session_id)
end
```

### 4. Caching with TTL
```lua
-- Save with expiration
local function cache_set(key, value, ttl)
    State.save("cache", key, {
        value = value,
        expires_at = os.time() + ttl
    })
end

-- Get with expiration check
local function cache_get(key)
    local entry = State.load("cache", key)
    if entry and entry.expires_at > os.time() then
        return entry.value
    end
    State.delete("cache", key)
    return nil
end
```

### 5. Counter/Statistics
```lua
-- Increment counter
local function increment(counter_name)
    local count = State.load("stats", counter_name) or 0
    count = count + 1
    State.save("stats", counter_name, count)
    return count
end

-- Track events
local function track_event(event_type)
    local events = State.load("stats", "events") or {}
    table.insert(events, {
        type = event_type,
        timestamp = os.time()
    })
    State.save("stats", "events", events)
end
```

## Best Practices

### Namespace Organization
- `global` - Application-wide settings
- `user` - User-specific data
- `session` - Temporary session data
- `cache` - Cached values with TTL
- `system` - System metadata
- `backup` - Backup data

### Key Naming
- Use descriptive, hierarchical keys: `user:123:preferences`
- Include type prefixes: `cache:api:response:users`
- Add timestamps for time-series: `log:2024:07:28:events`

### Error Handling
```lua
-- Always check for nil values
local data = State.load("namespace", "key")
if data == nil then
    -- Handle missing data
    data = get_default_value()
end

-- Validate loaded data
if type(data) ~= "table" then
    -- Handle corrupted data
    data = {}
end
```

### Performance Tips
1. Batch related updates into single save operations
2. Use lazy loading for large datasets
3. Implement cleanup for old/expired data
4. Keep individual values under 1MB
5. Use pagination for large collections

## Important Notes

1. **State.list_keys() is not available at runtime** - Examples show how to work around this by maintaining explicit key registries

2. **API Methods**:
   - ✅ Use: `State.save()`, `State.load()`, `State.delete()`
   - ❌ Don't use: `State.set()`, `State.get()`, `State.list()`

3. **Scopes** provide complete isolation:
   - `"global"` - Application-wide state
   - `"system"` - System configuration
   - `"agent:name"` - Agent-specific state
   - `"workflow:name"` - Workflow-specific state

4. **Migration and Backup APIs** require specific configuration:
   - Migration: Set `migration_enabled = true` in config
   - Backup: Set `backup_enabled = true` in config

## Troubleshooting

### State Not Persisting
- Check that state is enabled in your config file
- Verify the state path exists and is writable
- Ensure you're using the correct namespace/key

### API Functions Not Available
- Some functions require specific config settings
- Migration API needs `migration_enabled = true`
- Backup API needs `backup_enabled = true`
- Check with `test_state_api.lua` to see available functions

### Data Corruption
- Implement validation when loading data
- Keep backups of critical state
- Use atomic update patterns
- Add version fields for migration

## See Also

- [State Persistence Guide](/docs/user-guide/state-persistence-guide.md) - Comprehensive documentation
- [Backup Examples](/examples/lua/backup/) - Backup and recovery patterns
- [Migration Examples](/examples/lua/migration/) - Schema migration patterns
- [Configuration Examples](/examples/configs/) - Various configuration files