# State Persistence Guide

The State API in rs-llmspell provides a powerful and flexible way to persist data across script executions. This guide covers the complete State API, best practices, common patterns, and performance considerations.

## Table of Contents

1. [Overview](#overview)
2. [Core API Reference](#core-api-reference)
3. [Configuration](#configuration)
4. [Best Practices](#best-practices)
5. [Common Patterns](#common-patterns)
6. [Performance Considerations](#performance-considerations)
7. [Error Handling](#error-handling)
8. [Security Considerations](#security-considerations)

## Overview

The State API allows scripts to:
- Persist data across executions
- Share data between different scripts
- Implement backup and recovery strategies
- Handle schema migrations
- Maintain application state

State is accessed through the global `State` object available in all script environments (Lua, JavaScript, Python).

## Core API Reference

### Basic Operations

#### State.save(namespace, key, value)
Saves a value to persistent state.

```lua
-- Save simple values
State.save("global", "app_version", "1.0.0")
State.save("user", "preferences", {theme = "dark", language = "en"})

-- Save complex nested structures
State.save("app", "config", {
    database = {
        host = "localhost",
        port = 5432,
        pool_size = 10
    },
    features = {
        caching = true,
        analytics = false
    }
})
```

**Parameters:**
- `namespace` (string): Logical grouping for keys (e.g., "global", "user", "session")
- `key` (string): Unique identifier within the namespace
- `value` (any): Data to persist (strings, numbers, tables, booleans)

**Returns:** None

#### State.load(namespace, key)
Retrieves a value from persistent state.

```lua
-- Load saved values
local version = State.load("global", "app_version")
local prefs = State.load("user", "preferences")

-- Handle missing keys
local data = State.load("app", "non_existent") -- Returns nil
if data == nil then
    print("Key not found")
end
```

**Parameters:**
- `namespace` (string): The namespace to search in
- `key` (string): The key to retrieve

**Returns:** The stored value or `nil` if not found

#### State.delete(namespace, key)
Removes a key from persistent state.

```lua
-- Delete a single key
State.delete("user", "temp_data")

-- Clean up multiple keys
local temp_keys = {"cache1", "cache2", "cache3"}
for _, key in ipairs(temp_keys) do
    State.delete("temp", key)
end
```

**Parameters:**
- `namespace` (string): The namespace containing the key
- `key` (string): The key to delete

**Returns:** None

### Advanced Operations

#### State.list_keys(namespace)
Lists all keys in a namespace (when available).

```lua
-- Get all keys in a namespace
local keys = State.list_keys("global")
for _, key in ipairs(keys) do
    print("Key: " .. key)
end
```

**Note:** This function may not be available in all configurations. Always check availability before use.

#### State.migrate(target_version)
Performs automated schema migration (requires migration-enabled configuration).

```lua
-- Migrate to specific version
local result = State.migrate("2.0.0")
if result.success then
    print("Migrated from " .. result.from_version .. " to " .. result.to_version)
else
    print("Migration failed: " .. result.error)
end
```

#### State.schema_versions()
Returns available schema versions (requires migration-enabled configuration).

```lua
local versions = State.schema_versions()
for _, version in ipairs(versions) do
    print("Available version: " .. version)
end
```

#### State.create_backup(incremental)
Creates a backup of current state (requires backup-enabled configuration).

```lua
-- Create full backup
local backup = State.create_backup(false)
print("Backup ID: " .. backup.backup_id)

-- Create incremental backup
local incremental = State.create_backup(true)
print("Incremental backup size: " .. incremental.size_bytes)
```

#### State.restore_backup(backup_id)
Restores state from a backup (requires backup-enabled configuration).

```lua
local result = State.restore_backup("backup_12345")
if result.success then
    print("Restored " .. result.entries_restored .. " entries")
end
```

## Configuration

State persistence behavior is controlled through configuration files:

### Basic State Configuration
```toml
# examples/configs/state-enabled.toml
[agent]
default_provider = "openai"

[providers.openai]
api_key = "${OPENAI_API_KEY}"
model = "gpt-4"

[state]
enabled = true
backend = "disk"
path = "./state"
```

### Migration-Enabled Configuration
```toml
# examples/configs/migration-enabled.toml
[state]
enabled = true
backend = "disk"
path = "./state"
migration_enabled = true
schema_version = "1.0.0"
```

### Backup-Enabled Configuration
```toml
# examples/configs/backup-enabled.toml
[state]
enabled = true
backend = "disk"
path = "./state"
backup_enabled = true
backup_retention_days = 30
```

## Best Practices

### 1. Namespace Organization
Use consistent namespaces to organize your data:

```lua
-- Good namespace organization
State.save("config", "app_settings", {...})      -- Application configuration
State.save("user", "preferences", {...})         -- User-specific data
State.save("cache", "api_response", {...})       -- Temporary cached data
State.save("system", "metadata", {...})          -- System information

-- Avoid generic namespaces
State.save("data", "stuff", {...})              -- Too vague
State.save("global", "everything", {...})       -- Poor organization
```

### 2. Key Naming Conventions
Use descriptive, hierarchical key names:

```lua
-- Good key naming
State.save("user", "profile:display_name", "John")
State.save("user", "profile:email", "john@example.com")
State.save("cache", "api:users:1234", userData)

-- Avoid unclear keys
State.save("global", "n", "John")              -- What is 'n'?
State.save("data", "temp123", {...})           -- Unclear purpose
```

### 3. Data Structure Design
Design your data structures for easy updates:

```lua
-- Good: Modular structure
State.save("app", "features", {
    authentication = {enabled = true, provider = "oauth"},
    notifications = {enabled = false},
    analytics = {enabled = true, sample_rate = 0.1}
})

-- Avoid: Flat structure that's hard to update
State.save("app", "all_settings", {
    auth_enabled = true,
    auth_provider = "oauth",
    notif_enabled = false,
    analytics_enabled = true,
    analytics_rate = 0.1
})
```

### 4. Atomic Operations
Group related updates together:

```lua
-- Good: Atomic user update
local function updateUserProfile(userId, updates)
    local profile = State.load("user", "profile:" .. userId) or {}
    
    -- Apply all updates
    for key, value in pairs(updates) do
        profile[key] = value
    end
    profile.updated_at = os.time()
    
    -- Save once
    State.save("user", "profile:" .. userId, profile)
end

-- Avoid: Multiple separate saves
State.save("user", "name:" .. userId, "John")
State.save("user", "email:" .. userId, "john@example.com")
State.save("user", "updated:" .. userId, os.time())
```

## Common Patterns

### 1. Configuration Management
```lua
-- Load configuration with defaults
local function loadConfig()
    local config = State.load("app", "config")
    if not config then
        -- Initialize with defaults
        config = {
            theme = "light",
            language = "en",
            features = {
                autosave = true,
                notifications = true
            }
        }
        State.save("app", "config", config)
    end
    return config
end

-- Update specific configuration
local function updateConfig(path, value)
    local config = loadConfig()
    
    -- Navigate to nested path
    local current = config
    local keys = {}
    for key in string.gmatch(path, "[^.]+") do
        table.insert(keys, key)
    end
    
    -- Update the value
    for i = 1, #keys - 1 do
        current = current[keys[i]]
    end
    current[keys[#keys]] = value
    
    State.save("app", "config", config)
end

-- Usage
updateConfig("features.autosave", false)
updateConfig("theme", "dark")
```

### 2. Session Management
```lua
-- Create new session
local function createSession(userId)
    local sessionId = os.time() .. "_" .. math.random(10000)
    local session = {
        id = sessionId,
        user_id = userId,
        created_at = os.time(),
        last_active = os.time(),
        data = {}
    }
    
    State.save("session", sessionId, session)
    return sessionId
end

-- Update session activity
local function touchSession(sessionId)
    local session = State.load("session", sessionId)
    if session then
        session.last_active = os.time()
        State.save("session", sessionId, session)
        return true
    end
    return false
end

-- Clean up expired sessions
local function cleanupSessions(maxAge)
    local cutoff = os.time() - maxAge
    
    -- Note: This pattern works when list_keys is available
    if State.list_keys then
        local keys = State.list_keys("session")
        for _, key in ipairs(keys) do
            local session = State.load("session", key)
            if session and session.last_active < cutoff then
                State.delete("session", key)
            end
        end
    end
end
```

### 3. Cache with TTL
```lua
-- Save with TTL
local function cacheSet(key, value, ttl)
    local entry = {
        value = value,
        expires_at = os.time() + ttl
    }
    State.save("cache", key, entry)
end

-- Get with TTL check
local function cacheGet(key)
    local entry = State.load("cache", key)
    if entry then
        if entry.expires_at > os.time() then
            return entry.value
        else
            -- Expired, clean up
            State.delete("cache", key)
        end
    end
    return nil
end

-- Usage
cacheSet("api:user:123", userData, 3600) -- 1 hour TTL
local cached = cacheGet("api:user:123")
```

### 4. Counter and Statistics
```lua
-- Increment counter
local function incrementCounter(name)
    local stats = State.load("stats", "counters") or {}
    stats[name] = (stats[name] or 0) + 1
    State.save("stats", "counters", stats)
    return stats[name]
end

-- Track event with metadata
local function trackEvent(eventType, metadata)
    local events = State.load("stats", "events:" .. eventType) or {}
    table.insert(events, {
        timestamp = os.time(),
        data = metadata
    })
    
    -- Keep only last 100 events
    if #events > 100 then
        -- Remove oldest events
        local newEvents = {}
        for i = #events - 99, #events do
            table.insert(newEvents, events[i])
        end
        events = newEvents
    end
    
    State.save("stats", "events:" .. eventType, events)
end
```

### 5. Migration Patterns
```lua
-- Version-aware data loading
local function loadVersionedData(key)
    local version = State.load("system", "schema_version") or "1.0.0"
    local data = State.load("data", key)
    
    if not data then
        return nil
    end
    
    -- Apply migrations based on version
    if version < "2.0.0" then
        -- Migrate from v1 to v2 format
        if data.old_field then
            data.new_field = data.old_field
            data.old_field = nil
        end
    end
    
    return data
end

-- Safe schema upgrade
local function upgradeSchema(fromVersion, toVersion)
    -- Create backup first
    local backup_key = "schema_backup_" .. fromVersion
    local all_data = {}
    
    -- Backup current data (simplified - real implementation would be more robust)
    local namespaces = {"global", "user", "app"}
    for _, ns in ipairs(namespaces) do
        all_data[ns] = {}
        -- Would need list_keys here for complete backup
    end
    
    State.save("system", backup_key, all_data)
    
    -- Perform migration
    -- ... migration logic ...
    
    -- Update version
    State.save("system", "schema_version", toVersion)
end
```

## Performance Considerations

### 1. Batch Operations
Minimize the number of State operations:

```lua
-- Good: Single save operation
local userData = {
    profile = {name = "John", email = "john@example.com"},
    preferences = {theme = "dark", notifications = true},
    stats = {logins = 1, last_login = os.time()}
}
State.save("user", userId, userData)

-- Avoid: Multiple saves
State.save("user", userId .. ":name", "John")
State.save("user", userId .. ":email", "john@example.com")
State.save("user", userId .. ":theme", "dark")
-- ... many more saves
```

### 2. Data Size Management
Keep individual values reasonably sized:

```lua
-- Good: Paginated data storage
local function saveLargeDataset(dataset, chunkSize)
    chunkSize = chunkSize or 1000
    local totalChunks = math.ceil(#dataset / chunkSize)
    
    for i = 1, totalChunks do
        local startIdx = (i - 1) * chunkSize + 1
        local endIdx = math.min(i * chunkSize, #dataset)
        local chunk = {}
        
        for j = startIdx, endIdx do
            table.insert(chunk, dataset[j])
        end
        
        State.save("data", "chunk:" .. i, chunk)
    end
    
    State.save("data", "metadata", {
        total_chunks = totalChunks,
        total_items = #dataset,
        chunk_size = chunkSize
    })
end
```

### 3. Lazy Loading
Load data only when needed:

```lua
-- Good: Lazy loading pattern
local ConfigCache = {}

function ConfigCache:get(key)
    if not self.data then
        self.data = State.load("config", "app") or {}
    end
    return self.data[key]
end

function ConfigCache:set(key, value)
    if not self.data then
        self.data = State.load("config", "app") or {}
    end
    self.data[key] = value
    State.save("config", "app", self.data)
end

function ConfigCache:invalidate()
    self.data = nil
end
```

### 4. Cleanup Strategies
Implement regular cleanup of old data:

```lua
-- Time-based cleanup
local function cleanupOldData(namespace, maxAgeDays)
    local cutoff = os.time() - (maxAgeDays * 24 * 60 * 60)
    
    -- Pattern for time-prefixed keys
    if State.list_keys then
        local keys = State.list_keys(namespace)
        for _, key in ipairs(keys) do
            -- Extract timestamp from key if encoded
            local timestamp = string.match(key, "^(%d+)_")
            if timestamp and tonumber(timestamp) < cutoff then
                State.delete(namespace, key)
            end
        end
    end
end

-- Size-based cleanup
local function enforceQuota(namespace, maxItems)
    if State.list_keys then
        local keys = State.list_keys(namespace)
        if #keys > maxItems then
            -- Sort by key (assumes timestamp prefix)
            table.sort(keys)
            
            -- Delete oldest items
            local toDelete = #keys - maxItems
            for i = 1, toDelete do
                State.delete(namespace, keys[i])
            end
        end
    end
end
```

## Error Handling

### 1. Defensive Loading
Always handle missing or corrupted data:

```lua
-- Safe loading with validation
local function loadUserData(userId)
    local data = State.load("user", userId)
    
    -- Handle missing data
    if not data then
        return nil, "User not found"
    end
    
    -- Validate structure
    if type(data) ~= "table" then
        return nil, "Invalid user data format"
    end
    
    -- Ensure required fields
    local required = {"id", "email", "created_at"}
    for _, field in ipairs(required) do
        if data[field] == nil then
            return nil, "Missing required field: " .. field
        end
    end
    
    return data, nil
end

-- Usage
local user, err = loadUserData("user123")
if err then
    print("Error loading user: " .. err)
    -- Handle error appropriately
end
```

### 2. Transaction Patterns
Implement pseudo-transactions for critical operations:

```lua
-- Transactional update with rollback capability
local function updateUserAtomic(userId, updates)
    -- Load current state
    local original = State.load("user", userId)
    if not original then
        return false, "User not found"
    end
    
    -- Create backup
    State.save("temp", "backup:" .. userId, original)
    
    -- Apply updates
    local updated = {}
    for k, v in pairs(original) do
        updated[k] = v
    end
    for k, v in pairs(updates) do
        updated[k] = v
    end
    
    -- Validate updated state
    local valid, err = validateUserData(updated)
    if not valid then
        -- Rollback not needed, we haven't saved yet
        State.delete("temp", "backup:" .. userId)
        return false, err
    end
    
    -- Save updated state
    State.save("user", userId, updated)
    
    -- Clean up backup
    State.delete("temp", "backup:" .. userId)
    
    return true, nil
end
```

### 3. Recovery Patterns
Implement recovery mechanisms:

```lua
-- Self-healing configuration
local function getConfigWithRecovery()
    local config = State.load("app", "config")
    
    -- Detect corruption
    if config and type(config) ~= "table" then
        -- Try backup
        config = State.load("backup", "config")
        
        if config and type(config) == "table" then
            -- Restore from backup
            State.save("app", "config", config)
            print("Recovered configuration from backup")
        else
            -- Use defaults
            config = getDefaultConfig()
            State.save("app", "config", config)
            print("Reset configuration to defaults")
        end
    elseif not config then
        -- Initialize with defaults
        config = getDefaultConfig()
        State.save("app", "config", config)
    end
    
    return config
end
```

## Security Considerations

### 1. Sensitive Data Handling
Never store sensitive data in plain text:

```lua
-- Bad: Storing credentials directly
State.save("auth", "api_key", "sk-1234567890")

-- Better: Store references or encrypted data
State.save("auth", "api_key_ref", "vault://keys/api_key")

-- For truly sensitive data, use external secure storage
-- and only store references in State
```

### 2. Input Validation
Always validate data before storing:

```lua
local function saveUserInput(namespace, key, value)
    -- Validate namespace
    if not string.match(namespace, "^[a-zA-Z0-9_]+$") then
        return false, "Invalid namespace"
    end
    
    -- Validate key
    if string.len(key) > 255 then
        return false, "Key too long"
    end
    
    -- Validate value size
    local serialized = tostring(value)
    if string.len(serialized) > 1048576 then -- 1MB limit
        return false, "Value too large"
    end
    
    State.save(namespace, key, value)
    return true, nil
end
```

### 3. Access Control Patterns
Implement access control in your application logic:

```lua
-- Simple permission-based access
local function saveWithPermission(userId, namespace, key, value)
    -- Check user permissions
    local perms = State.load("auth", "permissions:" .. userId) or {}
    
    if not perms[namespace] or not perms[namespace].write then
        return false, "Access denied"
    end
    
    -- Add audit trail
    local audit = {
        user_id = userId,
        action = "write",
        namespace = namespace,
        key = key,
        timestamp = os.time()
    }
    State.save("audit", os.time() .. "_" .. userId, audit)
    
    -- Perform the save
    State.save(namespace, key, value)
    return true, nil
end
```

## Summary

The State API provides a powerful foundation for building stateful applications with rs-llmspell. By following these best practices and patterns, you can build robust, performant, and maintainable scripts that effectively manage persistent data.

Key takeaways:
- Use consistent namespace and key naming conventions
- Design data structures for easy updates and migrations
- Implement proper error handling and recovery mechanisms
- Consider performance implications of your state operations
- Always validate and sanitize data before storing
- Plan for schema evolution from the start
- Use configuration to enable advanced features like migrations and backups

For more examples, see the `examples/lua/state/` directory in the rs-llmspell repository.