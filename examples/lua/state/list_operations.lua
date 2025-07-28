-- ABOUTME: Advanced state management patterns without list_keys functionality
-- ABOUTME: Shows alternative approaches for managing state entries

-- CONFIG: Use examples/configs/state-enabled.toml
-- WHY: Demonstrates state management patterns when State.list_keys() is not available
-- HOW TO RUN: ./target/debug/llmspell -c examples/configs/state-enabled.toml run examples/lua/state/list_operations.lua
-- ALTERNATIVE: cargo run -- -c examples/configs/state-enabled.toml run examples/lua/state/list_operations.lua
-- NOTE: This example shows how to work around the absence of State.list_keys() by maintaining key registries

print("📋 State Management Patterns Example")
print("=====================================")

-- Note: State.list_keys() is not available at runtime
-- This example shows alternative patterns for state management

-- 1. Setup test data across multiple scopes
print("\n1. Creating test data...")

-- Global settings
State.save("global", "app.name", "LLMSpell Pro")
State.save("global", "app.version", "2.1.0")
State.save("global", "app.license", "enterprise")
State.save("global", "feature.ai", true)
State.save("global", "feature.analytics", true)
State.save("global", "feature.export", false)

-- User preferences
State.save("global", "user.name", "Alice")
State.save("global", "user.theme", "dark")
State.save("global", "user.language", "en")
State.save("global", "user.notifications", true)

-- Agent states
State.save("agent:assistant", "model", "gpt-4")
State.save("agent:assistant", "temperature", 0.7)
State.save("agent:assistant", "max_tokens", 2000)
State.save("agent:assistant", "personality", "helpful")

State.save("agent:researcher", "model", "claude-2")
State.save("agent:researcher", "search_depth", 10)
State.save("agent:researcher", "sources", {"web", "docs", "papers"})

-- Workflow data
State.save("workflow:daily-report", "schedule", "0 9 * * *")
State.save("workflow:daily-report", "recipients", {"alice@example.com", "bob@example.com"})
State.save("workflow:daily-report", "last_run", os.time() - 86400)

print("✅ Created test data in multiple scopes")

-- 2. Alternative to list operations: Track keys manually
print("\n2. Manual key tracking pattern...")

-- Since list_keys isn't available, maintain your own registry
local key_registry = {
    app = {"app.name", "app.version", "app.license"},
    feature = {"feature.ai", "feature.analytics", "feature.export"},
    user = {"user.name", "user.theme", "user.language", "user.notifications"}
}

-- Display tracked keys
print("   Tracked key groups:")
for group, keys in pairs(key_registry) do
    print("   " .. group .. ": " .. #keys .. " keys")
    for i, key in ipairs(keys) do
        local value = State.load("global", key)
        print(string.format("     %d. %s = %s", i, key, tostring(value)))
    end
end

-- 3. Alternative filtering: Use predefined key lists
print("\n3. Working with known key patterns...")

-- Without list_keys, define your key patterns upfront
local app_settings = {
    {key = "app.name", description = "Application name"},
    {key = "app.version", description = "Version number"},
    {key = "app.license", description = "License type"}
}

local features = {
    {key = "feature.ai", name = "AI Assistant"},
    {key = "feature.analytics", name = "Analytics"},
    {key = "feature.export", name = "Export"}
}

local user_prefs = {
    {key = "user.name", name = "Username"},
    {key = "user.theme", name = "Theme"},
    {key = "user.language", name = "Language"},
    {key = "user.notifications", name = "Notifications"}
}

-- Display app settings
print("   App settings:")
for _, setting in ipairs(app_settings) do
    local value = State.load("global", setting.key)
    print("   - " .. setting.key .. " = " .. tostring(value))
end

-- Display features
print("\n   Features:")
for _, feature in ipairs(features) do
    local value = State.load("global", feature.key)
    local status = value and "enabled" or "disabled"
    print("   - " .. feature.name .. ": " .. status)
end

-- Display user preferences
print("\n   User preferences:")
for _, pref in ipairs(user_prefs) do
    local value = State.load("global", pref.key)
    print("   - " .. pref.name .. ": " .. tostring(value))
end

-- 4. Cross-scope operations with known configurations
print("\n4. Managing multiple scopes...")

-- Define scope configurations upfront
local scope_configs = {
    ["agent:assistant"] = {
        {key = "model", default = "gpt-3.5-turbo"},
        {key = "temperature", default = 0.7},
        {key = "max_tokens", default = 1000},
        {key = "personality", default = "neutral"}
    },
    ["agent:researcher"] = {
        {key = "model", default = "claude-2"},
        {key = "search_depth", default = 5},
        {key = "sources", default = {"web", "docs"}}
    },
    ["workflow:daily-report"] = {
        {key = "schedule", default = "0 9 * * *"},
        {key = "recipients", default = {}},
        {key = "last_run", default = 0}
    }
}

-- Display configurations for each scope
print("   Agent and workflow configurations:")
for scope, configs in pairs(scope_configs) do
    print("   " .. scope .. ":")
    for _, config in ipairs(configs) do
        local value = State.load(scope, config.key)
        if value == nil then
            value = config.default
        end
        local value_str = type(value) == "table" and table.concat(value, ", ") or tostring(value)
        print("     - " .. config.key .. ": " .. value_str)
    end
end

-- 5. State validation and statistics
print("\n5. State validation and statistics...")

-- Function to validate and analyze known keys
local function validate_state_keys(scope, expected_keys)
    local stats = {
        total = #expected_keys,
        present = 0,
        missing = {},
        by_type = {}
    }
    
    for _, key_info in ipairs(expected_keys) do
        local key = type(key_info) == "table" and key_info.key or key_info
        local value = State.load(scope, key)
        
        if value ~= nil then
            stats.present = stats.present + 1
            local value_type = type(value)
            stats.by_type[value_type] = (stats.by_type[value_type] or 0) + 1
        else
            table.insert(stats.missing, key)
        end
    end
    
    return stats
end

-- Combine all known keys for global scope
local all_global_keys = {}
for _, key in ipairs(key_registry.app) do table.insert(all_global_keys, key) end
for _, key in ipairs(key_registry.feature) do table.insert(all_global_keys, key) end
for _, key in ipairs(key_registry.user) do table.insert(all_global_keys, key) end

local global_stats = validate_state_keys("global", all_global_keys)
print("   Global scope validation:")
print("   - Expected keys: " .. global_stats.total)
print("   - Present keys: " .. global_stats.present)
print("   - Missing keys: " .. #global_stats.missing)
if #global_stats.missing > 0 then
    print("   - Missing: " .. table.concat(global_stats.missing, ", "))
end
print("   - By type:")
for vtype, count in pairs(global_stats.by_type) do
    print("     * " .. vtype .. ": " .. count)
end

-- 6. Batch operations with known keys
print("\n6. Batch operations...")

-- Function to export known keys from a scope
local function export_known_keys(scope, key_list)
    local export_data = {
        scope = scope,
        timestamp = os.time(),
        entries = {}
    }
    
    for _, key_info in ipairs(key_list) do
        local key = type(key_info) == "table" and key_info.key or key_info
        local value = State.load(scope, key)
        if value ~= nil then
            export_data.entries[key] = value
        end
    end
    
    return export_data
end

-- Function to import scope data
local function import_scope(export_data)
    local scope = export_data.scope
    local imported = 0
    
    for key, value in pairs(export_data.entries) do
        State.save(scope, key, value)
        imported = imported + 1
    end
    
    return imported
end

-- Export and reimport example
local export = export_known_keys("agent:assistant", scope_configs["agent:assistant"])
local entry_count = 0
for _ in pairs(export.entries) do entry_count = entry_count + 1 end
print("   Exported " .. entry_count .. " entries from agent:assistant")

-- 7. Key maintenance with managed registries
print("\n7. Key maintenance operations...")

-- Function to clean up deprecated keys
local function cleanup_deprecated_keys(scope, deprecated_keys)
    local cleaned = 0
    
    for _, key in ipairs(deprecated_keys) do
        local value = State.load(scope, key)
        if value ~= nil then
            State.delete(scope, key)
            cleaned = cleaned + 1
            print("   - Removed deprecated key: " .. key)
        end
    end
    
    return cleaned
end

-- Example deprecated keys
local deprecated = {
    "old_setting",
    "legacy_config",
    "temp_data"
}

local cleaned = cleanup_deprecated_keys("global", deprecated)
if cleaned > 0 then
    print("   Cleaned up " .. cleaned .. " deprecated keys")
else
    print("   No deprecated keys found")
end

-- Function to migrate keys to new names
local function migrate_keys(scope, migration_map)
    local migrated = 0
    
    for old_key, new_key in pairs(migration_map) do
        local value = State.load(scope, old_key)
        if value ~= nil then
            State.save(scope, new_key, value)
            State.delete(scope, old_key)
            migrated = migrated + 1
            print("   - Migrated: " .. old_key .. " → " .. new_key)
        end
    end
    
    return migrated
end

-- Example: Rename feature. to features.
local migration_map = {}
for _, key in ipairs(key_registry.feature) do
    local new_key = key:gsub("^feature%.", "features.")
    migration_map[key] = new_key
end

local migrated_count = migrate_keys("global", migration_map)
print("   Migrated " .. migrated_count .. " keys from 'feature.' to 'features.'")

-- 8. Performance best practices
print("\n8. Performance tips for state management...")

print("   - Maintain key registries to avoid discovery overhead")
print("   - Group related data into single state entries")
print("   - Use batch operations to minimize state access")
print("   - Cache frequently accessed values locally")

-- Example: Efficient state access patterns
local state_cache = {}

-- Function to get cached state value
local function get_cached_value(scope, key)
    local cache_key = scope .. ":" .. key
    if state_cache[cache_key] == nil then
        state_cache[cache_key] = State.load(scope, key)
    end
    return state_cache[cache_key]
end

-- Function to save and update cache
local function save_with_cache(scope, key, value)
    State.save(scope, key, value)
    local cache_key = scope .. ":" .. key
    state_cache[cache_key] = value
end

-- Example usage
print("   Using cached access:")
local theme = get_cached_value("global", "user.theme")
print("   - Cached theme: " .. tostring(theme))

-- Update with cache
save_with_cache("global", "user.theme", "light")
print("   - Updated theme in state and cache")

print("\n✅ State management patterns example completed!")
print("\nKey patterns demonstrated:")
print("- Maintain key registries instead of discovery")
print("- Use predefined configurations for scopes")
print("- Validate state against expected keys")
print("- Perform batch operations with known keys")
print("- Implement key migration and cleanup")
print("- Cache values for better performance")
print("\nNote: These patterns work without State.list_keys()")