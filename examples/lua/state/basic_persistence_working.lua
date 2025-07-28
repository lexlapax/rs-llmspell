-- ABOUTME: Basic state persistence operations using the State global
-- ABOUTME: Demonstrates save, load, and delete functionality (without list_keys)

-- CONFIG: Use examples/configs/state-enabled.toml
-- WHY: This is a working version that avoids State.list_keys() which isn't available at runtime
-- HOW TO RUN: ./target/debug/llmspell -c examples/configs/state-enabled.toml run examples/lua/state/basic_persistence_working.lua
-- ALTERNATIVE: cargo run -- -c examples/configs/state-enabled.toml run examples/lua/state/basic_persistence_working.lua
-- NOTE: This example can be deleted as basic_persistence.lua has been fixed

print("🗄️  Basic State Persistence Example")
print("=====================================")

-- The State global provides persistent state management
-- Available operations: save, load, delete

-- 1. Basic Save Operation
print("\n1. Saving state data...")
local success, error = pcall(function()
    -- Save a simple string value
    State.save("global", "app_name", "My LLMSpell App")
    
    -- Save a number
    State.save("global", "version", 1.0)
    
    -- Save a boolean
    State.save("global", "is_configured", true)
    
    -- Save a complex table (will be serialized to JSON)
    State.save("global", "config", {
        theme = "dark",
        language = "en",
        features = {
            ai_assist = true,
            auto_save = false,
            debug_mode = false
        }
    })
    
    -- Save an array
    State.save("global", "recent_files", {
        "document1.txt",
        "script.lua",
        "config.json"
    })
end)

if success then
    print("✅ State saved successfully")
else
    print("❌ Error saving state: " .. tostring(error))
end

-- 2. Basic Load Operation
print("\n2. Loading state data...")
local app_name = State.load("global", "app_name")
local version = State.load("global", "version")
local is_configured = State.load("global", "is_configured")
local config = State.load("global", "config")
local recent_files = State.load("global", "recent_files")

print("   App Name: " .. tostring(app_name))
print("   Version: " .. tostring(version))
print("   Configured: " .. tostring(is_configured))
print("   Theme: " .. (config and config.theme or "not set"))
print("   Recent Files: " .. (recent_files and #recent_files or 0) .. " files")

-- 3. Loading non-existent keys returns nil
print("\n3. Loading non-existent key...")
local missing = State.load("global", "non_existent_key")
print("   Missing key value: " .. tostring(missing) .. " (should be nil)")

-- 4. Update existing values
print("\n4. Updating existing values...")
State.save("global", "version", 1.1)  -- Update version
local new_version = State.load("global", "version")
print("   Updated version to: " .. tostring(new_version))

-- 5. Working with complex data
print("\n5. Working with complex data structures...")

-- Save user preferences
State.save("global", "user_preferences", {
    display = {
        theme = "dark",
        font_size = 14,
        show_line_numbers = true
    },
    editor = {
        auto_indent = true,
        tab_size = 4,
        word_wrap = false
    },
    notifications = {
        enabled = true,
        sound = false,
        desktop = true
    }
})

-- Load and access nested data
local prefs = State.load("global", "user_preferences")
if prefs then
    print("   User theme: " .. (prefs.display and prefs.display.theme or "unknown"))
    print("   Tab size: " .. (prefs.editor and prefs.editor.tab_size or "unknown"))
    print("   Notifications enabled: " .. tostring(prefs.notifications and prefs.notifications.enabled))
end

-- 6. Delete operations
print("\n6. Deleting state entries...")

-- Delete specific keys
State.delete("global", "is_configured")
State.delete("global", "recent_files")

-- Verify deletion
local deleted1 = State.load("global", "is_configured")
local deleted2 = State.load("global", "recent_files")
print("   Deleted keys are now nil:")
print("   - is_configured: " .. tostring(deleted1))
print("   - recent_files: " .. tostring(deleted2))

-- 7. Error handling
print("\n7. Error handling examples...")

-- Try to save with invalid data (implementation dependent)
local ok, err = pcall(function()
    -- Some implementations might not support certain types
    State.save("global", "function_test", function() end)
end)
if not ok then
    print("   Function save error (expected): " .. tostring(err))
else
    print("   Functions can be saved (unexpected)")
end

-- 8. Working with nil values
print("\n8. Handling nil and special values...")

-- Saving nil effectively deletes the key in some implementations
State.save("global", "temp_key", "temporary value")
print("   Created temp_key: " .. tostring(State.load("global", "temp_key")))

-- Try saving nil
local ok, err = pcall(function()
    State.save("global", "temp_key", nil)
end)
if ok then
    local nil_value = State.load("global", "temp_key")
    print("   After saving nil: " .. tostring(nil_value))
else
    print("   Saving nil not allowed: " .. tostring(err))
end

-- 9. Performance tip: Batch operations
print("\n9. Performance tip: Batch operations...")
print("   When saving multiple related values, consider grouping them:")

-- Instead of multiple saves:
-- State.save("global", "user_name", "John")
-- State.save("global", "user_email", "john@example.com")
-- State.save("global", "user_role", "admin")

-- Group into a single object:
State.save("global", "user", {
    name = "John",
    email = "john@example.com",
    role = "admin"
})

local user = State.load("global", "user")
print("   User data saved as single object:")
print("   - Name: " .. (user and user.name or "unknown"))
print("   - Email: " .. (user and user.email or "unknown"))
print("   - Role: " .. (user and user.role or "unknown"))

-- 10. State persistence patterns
print("\n10. Common state persistence patterns...")

-- Pattern 1: Settings with defaults
local function load_settings()
    local settings = State.load("global", "app_settings") or {}
    -- Apply defaults for missing values
    settings.theme = settings.theme or "light"
    settings.language = settings.language or "en"
    settings.auto_save = settings.auto_save ~= false  -- Default true
    return settings
end

local settings = load_settings()
print("   Settings with defaults:")
print("   - Theme: " .. settings.theme)
print("   - Language: " .. settings.language)
print("   - Auto-save: " .. tostring(settings.auto_save))

-- Pattern 2: Versioned data
State.save("global", "data_v2", {
    version = 2,
    data = {
        -- Your data here
    }
})

-- Pattern 3: Timestamped entries
State.save("global", "last_save", {
    timestamp = os.time(),
    user = "current_user",
    action = "manual_save"
})

print("\n✅ Basic persistence example completed!")
print("\nKey takeaways:")
print("- Use State.save(scope, key, value) to store data")
print("- Use State.load(scope, key) to retrieve data")
print("- Use State.delete(scope, key) to remove data")
print("- Complex tables are automatically serialized")
print("- Missing keys return nil when loaded")
print("- Group related data for better performance")
print("- Always handle potential errors gracefully")