-- ABOUTME: Debug script to list all available globals
-- ABOUTME: Helps diagnose which globals are actually injected

print("=== Available Globals ===")

-- List of expected globals
local expected_globals = {
    "Agent", "Tool", "Workflow", "Hook", "Event", 
    "State", "Replay", "JSON", "Utils", "Config", 
    "Logger", "Streaming"
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

print("\n=== All Non-Standard Globals ===")
-- List all globals that aren't standard Lua globals
local standard_globals = {
    "_G", "_VERSION", "assert", "collectgarbage", "dofile", "error",
    "getmetatable", "ipairs", "load", "loadfile", "next", "pairs",
    "pcall", "print", "rawequal", "rawget", "rawlen", "rawset",
    "require", "select", "setmetatable", "tonumber", "tostring",
    "type", "xpcall", "coroutine", "debug", "io", "math", "os",
    "package", "string", "table", "utf8"
}

local standard_set = {}
for _, name in ipairs(standard_globals) do
    standard_set[name] = true
end

local count = 0
for name, value in pairs(_G) do
    if not standard_set[name] then
        print(string.format("  %s: %s", name, type(value)))
        count = count + 1
    end
end

print(string.format("\nTotal non-standard globals: %d", count))