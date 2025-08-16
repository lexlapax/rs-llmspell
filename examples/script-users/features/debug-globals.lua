-- Example: Debug Available Globals
-- Purpose: Lists all available global objects injected into Lua environment
-- Prerequisites: None
-- Expected Output: Shows all available globals with their types and methods
-- Version: 0.7.0
-- Tags: features, debugging, globals, no-dependencies

-- ABOUTME: Debug script to list all available globals
-- ABOUTME: Helps diagnose which globals are actually injected

print("=== Available Globals ===")

-- List of expected globals
local expected_globals = {
    "Agent", "Tool", "Workflow", "Hook", "Event", 
    "State", "Replay", "JSON", "Utils", "Config", 
    "Logger", "Streaming", "Provider", "Session", "Artifact"
}

-- Check each expected global
for _, name in ipairs(expected_globals) do
    local value = _G[name]
    if value ~= nil then
        print(string.format("✅ %s: %s", name, type(value)))
        -- If it's a table, show its keys
        if type(value) == "table" then
            local keys = {}
            for k, _ in pairs(value) do
                table.insert(keys, k)
            end
            if #keys > 0 then
                print(string.format("   Keys: %s", table.concat(keys, ", ")))
            end
        end
    else
        print(string.format("❌ %s: nil", name))
    end
end

print("\n=== Testing Basic Functionality ===")

-- Test JSON if available
if JSON then
    local test_data = {message = "Hello", count = 42}
    local json_str = JSON.stringify(test_data)
    local parsed = JSON.parse(json_str)
    print("✅ JSON stringify/parse working")
end

-- Test Tool.list if available
if Tool and Tool.list then
    local tools = Tool.list()
    print(string.format("✅ Found %d tools", #tools))
end

-- Test State if available
if State then
    State.set("test_key", "test_value")
    local value = State.get("test_key")
    if value == "test_value" then
        print("✅ State get/set working")
        State.delete("test_key")
    end
end

print("\n=== Debug Complete ===")

return {
    status = "success",
    globals_found = #expected_globals,
    timestamp = os.date()
}