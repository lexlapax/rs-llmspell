-- ABOUTME: Test script to check which State API functions are available
-- ABOUTME: Useful for debugging and understanding the runtime State API

-- CONFIG: Use examples/configs/state-enabled.toml (or any config with state persistence)
-- WHY: This test script helps verify which State functions are available at runtime
-- HOW TO RUN: ./target/debug/llmspell -c examples/configs/state-enabled.toml run examples/lua/state/test_state_api.lua
-- ALTERNATIVE: cargo run -- -c examples/configs/state-enabled.toml run examples/lua/state/test_state_api.lua
-- NOTE: This shows that State.list_keys() is defined but not available at runtime

-- Test what State API functions are available

print("Testing State API availability...")
print("=====================================")

-- Check if State global exists
if State then
    print("✅ State global exists")
else
    print("❌ State global not found")
    return
end

-- Check available functions
local functions = {
    "save", "load", "delete", "list_keys", 
    "migrate", "schema_versions",
    "create_backup", "restore_backup"
}

print("\nAvailable State functions:")
for _, func in ipairs(functions) do
    if State[func] then
        print("✅ State." .. func .. "()")
    else
        print("❌ State." .. func .. "() - NOT AVAILABLE")
    end
end

-- Test basic operations
print("\nTesting basic operations...")

-- Test save
local ok, err = pcall(function()
    State.save("global", "test_key", "test_value")
end)
if ok then
    print("✅ State.save() works")
else
    print("❌ State.save() failed: " .. tostring(err))
end

-- Test load
ok, err = pcall(function()
    local value = State.load("global", "test_key")
    print("✅ State.load() works - got: " .. tostring(value))
end)
if not ok then
    print("❌ State.load() failed: " .. tostring(err))
end

-- Test list_keys
ok, err = pcall(function()
    local keys = State.list_keys("global")
    print("✅ State.list_keys() works - found " .. #keys .. " keys")
end)
if not ok then
    print("❌ State.list_keys() failed: " .. tostring(err))
end

-- Test delete
ok, err = pcall(function()
    State.delete("global", "test_key")
    print("✅ State.delete() works")
end)
if not ok then
    print("❌ State.delete() failed: " .. tostring(err))
end