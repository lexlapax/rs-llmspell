-- ABOUTME: Basic state persistence operations demonstrating core State API
-- ABOUTME: Shows save, load, delete operations with error handling and best practices

-- CONFIG: Use examples/configs/state-enabled.toml or examples/state_persistence/configs/basic.toml  
-- RUN: ./target/debug/llmspell -c examples/configs/state-enabled.toml run examples/state_persistence/basic_operations.lua
-- ALTERNATIVE: cargo run -- -c examples/configs/state-enabled.toml run examples/state_persistence/basic_operations.lua

print("🗄️  State Persistence - Basic Operations Example")
print("================================================")
print("This example demonstrates the core State API operations:")
print("- State.save(scope, key, value)")
print("- State.load(scope, key)")
print("- State.delete(scope, key)")
print()

-- Helper function for safe operations
local function safe_operation(operation_name, operation_func)
    local success, result = pcall(operation_func)
    if not success then
        print("❌ " .. operation_name .. " failed: " .. tostring(result))
        return nil, result
    end
    return result
end

-- 1. BASIC DATA TYPES
print("1. Saving different data types...")

-- String
safe_operation("Save string", function()
    State.save("global", "app_name", "rs-llmspell State Example")
end)

-- Number
safe_operation("Save number", function()
    State.save("global", "version_number", 1.5)
end)

-- Boolean
safe_operation("Save boolean", function()
    State.save("global", "debug_enabled", true)
end)

-- Table/Object
safe_operation("Save object", function()
    State.save("global", "app_config", {
        theme = "dark",
        language = "en",
        auto_save = true,
        features = {
            ai_assist = true,
            notifications = false,
            advanced_mode = true
        },
        limits = {
            max_history = 1000,
            timeout_seconds = 30
        }
    })
end)

-- Array
safe_operation("Save array", function()
    State.save("global", "recent_files", {
        "document1.txt",
        "script.lua", 
        "config.json",
        "data.csv"
    })
end)

print("✅ All data types saved successfully!")

-- 2. LOADING DATA
print("\n2. Loading saved data...")

local app_name = State.load("global", "app_name")
local version = State.load("global", "version_number")
local debug_enabled = State.load("global", "debug_enabled")
local config = State.load("global", "app_config")
local recent_files = State.load("global", "recent_files")

print("   App Name: " .. tostring(app_name))
print("   Version: " .. tostring(version))
print("   Debug Enabled: " .. tostring(debug_enabled))
print("   Theme: " .. (config and config.theme or "not set"))
print("   AI Assist: " .. tostring(config and config.features and config.features.ai_assist))
print("   Recent Files Count: " .. (recent_files and #recent_files or 0))

-- 3. WORKING WITH NESTED DATA
print("\n3. Working with nested data structures...")

if config then
    print("   Configuration details:")
    print("   - Theme: " .. (config.theme or "unknown"))
    print("   - Language: " .. (config.language or "unknown"))
    print("   - Auto-save: " .. tostring(config.auto_save))
    print("   - Max History: " .. (config.limits and config.limits.max_history or "unknown"))
    print("   - Timeout: " .. (config.limits and config.limits.timeout_seconds or "unknown") .. "s")
end

-- 4. HANDLING MISSING DATA
print("\n4. Handling missing data...")

local missing_data = State.load("global", "non_existent_key")
print("   Missing key result: " .. tostring(missing_data) .. " (should be nil)")

-- Load with fallback pattern
local function load_with_default(scope, key, default_value)
    local value = State.load(scope, key)
    if value == nil then
        print("   Using default for '" .. key .. "'")
        return default_value
    end
    return value
end

local user_theme = load_with_default("global", "user_theme", "light")
local max_connections = load_with_default("global", "max_connections", 10)

print("   User theme: " .. user_theme)
print("   Max connections: " .. max_connections)

-- 5. UPDATING EXISTING DATA
print("\n5. Updating existing data...")

-- Update version number
safe_operation("Update version", function()
    State.save("global", "version_number", 2.0)
end)

-- Update nested configuration
safe_operation("Update config", function()
    local current_config = State.load("global", "app_config") or {}
    current_config.theme = "light"  -- Change theme
    current_config.last_updated = os.time()  -- Add timestamp
    current_config.features.notifications = true  -- Enable notifications
    
    State.save("global", "app_config", current_config)
end)

-- Verify updates
local updated_version = State.load("global", "version_number")
local updated_config = State.load("global", "app_config")

print("   Updated version: " .. tostring(updated_version))
print("   Updated theme: " .. (updated_config and updated_config.theme or "unknown"))
print("   Last updated: " .. (updated_config and updated_config.last_updated or "unknown"))
print("   Notifications enabled: " .. tostring(updated_config and updated_config.features and updated_config.features.notifications))

-- 6. DIFFERENT SCOPES
print("\n6. Using different scopes...")

-- Global scope (application-wide)
safe_operation("Save to global scope", function()
    State.save("global", "global_setting", "This is global data")
end)

-- Agent scope (agent-specific)
safe_operation("Save to agent scope", function()
    State.save("agent:gpt-4", "model_config", {
        temperature = 0.7,
        max_tokens = 2000,
        system_prompt = "You are a helpful assistant."
    })
end)

-- Workflow scope (workflow-specific)
safe_operation("Save to workflow scope", function()
    State.save("workflow:data-processing", "pipeline_state", {
        current_step = "validation",
        completed_steps = {"input", "preprocessing"},
        total_steps = 5,
        progress = 0.4
    })
end)

-- Custom scope
safe_operation("Save to custom scope", function()
    State.save("user:john_doe", "preferences", {
        display_name = "John Doe",
        email_notifications = true,
        timezone = "UTC"
    })
end)

-- Load from different scopes
local global_data = State.load("global", "global_setting")
local agent_config = State.load("agent:gpt-4", "model_config")
local workflow_state = State.load("workflow:data-processing", "pipeline_state")
local user_prefs = State.load("user:john_doe", "preferences")

print("   Global data: " .. tostring(global_data))
print("   Agent temperature: " .. (agent_config and agent_config.temperature or "unknown"))
print("   Workflow step: " .. (workflow_state and workflow_state.current_step or "unknown"))
print("   User display name: " .. (user_prefs and user_prefs.display_name or "unknown"))

-- 7. DELETE OPERATIONS
print("\n7. Deleting state entries...")

-- Delete specific entries
safe_operation("Delete debug flag", function()
    State.delete("global", "debug_enabled")
end)

safe_operation("Delete recent files", function()
    State.delete("global", "recent_files")  
end)

-- Verify deletions
local deleted_debug = State.load("global", "debug_enabled")
local deleted_files = State.load("global", "recent_files")

print("   Debug flag after delete: " .. tostring(deleted_debug) .. " (should be nil)")
print("   Recent files after delete: " .. tostring(deleted_files) .. " (should be nil)")

-- 8. PRACTICAL PATTERNS
print("\n8. Practical usage patterns...")

-- Session management pattern
local function create_session(user_id)
    local session_id = "session_" .. os.time() .. "_" .. math.random(10000)
    local session_data = {
        user_id = user_id,
        created_at = os.time(),
        expires_at = os.time() + 3600, -- 1 hour
        is_active = true
    }
    
    State.save("session", session_id, session_data)
    return session_id
end

local function validate_session(session_id)
    local session = State.load("session", session_id)
    if not session then
        return false, "Session not found"
    end
    
    if session.expires_at < os.time() then
        State.delete("session", session_id) -- Cleanup expired session
        return false, "Session expired"
    end
    
    return true, session
end

-- Create and validate a session
local session_id = create_session("user123")
print("   Created session: " .. session_id)

local valid, session_or_error = validate_session(session_id)
if valid then
    print("   Session is valid for user: " .. session_or_error.user_id)
else
    print("   Session validation failed: " .. session_or_error)
end

-- Counter pattern
local function increment_counter(counter_name)
    local count = State.load("counters", counter_name) or 0
    count = count + 1
    State.save("counters", counter_name, count)
    return count
end

-- Use counter
local page_views = increment_counter("page_views")
local api_calls = increment_counter("api_calls")

print("   Page views: " .. page_views)
print("   API calls: " .. api_calls)

-- Cache pattern with TTL
local function cache_set(key, value, ttl_seconds)
    local cache_entry = {
        value = value,
        expires_at = os.time() + ttl_seconds
    }
    State.save("cache", key, cache_entry)
end

local function cache_get(key)
    local entry = State.load("cache", key)
    if not entry then
        return nil
    end
    
    if entry.expires_at < os.time() then
        State.delete("cache", key) -- Cleanup expired entry
        return nil
    end
    
    return entry.value
end

-- Use cache
cache_set("user_profile", {name = "John", role = "admin"}, 300) -- 5 minutes TTL
local cached_profile = cache_get("user_profile")
print("   Cached profile name: " .. (cached_profile and cached_profile.name or "not found"))

-- 9. ERROR HANDLING BEST PRACTICES
print("\n9. Error handling examples...")

-- Try to save invalid data (implementation dependent)
local function test_error_handling()
    -- Test 1: Save function as value (should fail gracefully)
    local success, error_msg = pcall(function()
        State.save("global", "function_test", function() return 42 end)
    end)
    
    if not success then
        print("   ✅ Function save properly rejected: " .. tostring(error_msg))
    else
        print("   ⚠️  Function save unexpectedly succeeded")
    end
    
    -- Test 2: Invalid scope format
    local success2, error_msg2 = pcall(function()
        State.save("", "empty_scope_key", "value")
    end)
    
    if not success2 then
        print("   ✅ Empty scope properly rejected: " .. tostring(error_msg2))
    else
        print("   ⚠️  Empty scope unexpectedly accepted")
    end
end

test_error_handling()

-- 10. FINAL SUMMARY
print("\n10. Summary and cleanup...")

-- Show what data we have
local final_config = State.load("global", "app_config")
local final_version = State.load("global", "version_number")
local session_count = State.load("counters", "page_views")

print("   Final app config theme: " .. (final_config and final_config.theme or "unknown"))
print("   Final version: " .. tostring(final_version))
print("   Total page views: " .. tostring(session_count))

print("\n✅ Basic State Persistence Example Completed!")
print("\nKey takeaways:")
print("- Use State.save(scope, key, value) to store data")
print("- Use State.load(scope, key) to retrieve data (returns nil if not found)")
print("- Use State.delete(scope, key) to remove data")
print("- Different scopes provide isolation: 'global', 'agent:name', 'workflow:name', 'custom'")
print("- Complex tables and arrays are automatically serialized")
print("- Always handle potential nil values when loading")
print("- Use pcall() for robust error handling")
print("- Implement patterns like sessions, counters, and caching as needed")

print("\n🔗 Next steps:")
print("- Try agent_state.lua for agent-specific patterns")
print("- See migration examples for schema evolution")
print("- Check backup examples for data protection")
print("- Review security examples for access control")