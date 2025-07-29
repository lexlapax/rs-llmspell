-- ABOUTME: Basic state persistence operations using the State global
-- ABOUTME: Demonstrates save, load, and delete functionality

-- CONFIG: Use examples/configs/state-enabled.toml
-- WHY: This example requires state persistence to be enabled with a backend
-- HOW TO RUN: ./target/debug/llmspell -c examples/configs/state-enabled.toml run examples/lua/state/basic_persistence.lua
-- ALTERNATIVE: cargo run -- -c examples/configs/state-enabled.toml run examples/lua/state/basic_persistence.lua

print("🗄️  Basic State Persistence Example")
print("=====================================")

-- The State global provides persistent state management
-- All operations require a scope and a key

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

-- 4. Verify saved keys exist
print("\n4. Verifying saved keys...")
local expected_keys = {"app_name", "version", "config", "user"}
local found = 0

for _, key in ipairs(expected_keys) do
    local value = State.load("global", key)
    if value ~= nil then
        found = found + 1
        print("   - " .. key .. " exists")
    end
end
print("   Found " .. found .. " of " .. #expected_keys .. " expected keys")

-- 5. Update existing values
print("\n5. Updating existing values...")
State.save("global", "version", 1.1)  -- Update version
local new_version = State.load("global", "version")
print("   Updated version to: " .. tostring(new_version))

-- 6. Delete operations
print("\n6. Deleting state entries...")

-- Check if keys exist before deletion
local is_configured_before = State.load("global", "is_configured")
local recent_files_before = State.load("global", "recent_files")
print("   Before deletion:")
print("   - is_configured: " .. tostring(is_configured_before ~= nil))
print("   - recent_files: " .. tostring(recent_files_before ~= nil))

-- Delete specific keys  
State.delete("global", "is_configured")
State.delete("global", "recent_files")

print("   After deletion:")
print("   - is_configured: " .. tostring(State.load("global", "is_configured")))
print("   - recent_files: " .. tostring(State.load("global", "recent_files")))

-- 7. Error handling
print("\n7. Error handling examples...")

-- Try to save with invalid scope (this might work, showing flexibility)
local ok, err = pcall(function()
    State.save("", "key", "value")  -- Empty scope
end)
if not ok then
    print("   Empty scope error: " .. tostring(err))
else
    print("   Empty scope allowed (implementation dependent)")
end

-- 8. Working with nil values
print("\n8. Handling nil and special values...")

-- Saving nil effectively deletes the key
State.save("global", "temp_key", "temporary value")
print("   Created temp_key: " .. tostring(State.load("global", "temp_key")))

-- Note: The actual behavior of saving nil depends on the implementation
-- Some implementations might delete the key, others might store null
State.save("global", "temp_key", nil)
local nil_value = State.load("global", "temp_key")
print("   After saving nil: " .. tostring(nil_value))

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

-- 10. Cleanup
print("\n10. Cleaning up example data...")
-- Define all keys we created
local cleanup_keys = {
    "app_name", "version", "config", "user",
    "temp_key", "is_configured", "recent_files"
}

local cleaned = 0
for _, key in ipairs(cleanup_keys) do
    if State.load("global", key) ~= nil then
        State.delete("global", key)
        cleaned = cleaned + 1
    end
end
print("   Cleaned up " .. cleaned .. " keys")

print("\n✅ Basic persistence example completed!")
print("\nKey takeaways:")
print("- Use State.save(scope, key, value) to store data")
print("- Use State.load(scope, key) to retrieve data")
print("- Use State.delete(scope, key) to remove data")
print("- Complex tables are automatically serialized to JSON")
print("- Missing keys return nil when loaded")
print("- Group related data for better performance")
print("- Track your keys manually for reliable state management")