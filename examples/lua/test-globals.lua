-- ABOUTME: Test which globals are available in the Lua environment
-- ABOUTME: Simple script to identify available global objects

print("=== Testing Global Objects ===\n")

-- List of expected globals
local globals_to_test = {
    "Tool", "State", "Agent", "Workflow", 
    "Hook", "Event", "JSON", "Logger", 
    "Config", "Utils", "Security"
}

print("Testing individual globals:")
for _, global_name in ipairs(globals_to_test) do
    if _G[global_name] then
        print("✓ " .. global_name .. " is available")
        
        -- Check if it's a table and list its keys
        if type(_G[global_name]) == "table" then
            local keys = {}
            for k, _ in pairs(_G[global_name]) do
                table.insert(keys, k)
            end
            if #keys > 0 then
                table.sort(keys)
                print("  Methods: " .. table.concat(keys, ", "))
            end
        end
    else
        print("✗ " .. global_name .. " is NOT available")
    end
end

-- Also print all globals (excluding standard Lua ones)
print("\nAll available globals:")
local lua_standard = {
    "_G", "_VERSION", "assert", "collectgarbage", "dofile", "error", 
    "getmetatable", "ipairs", "load", "loadfile", "next", "pairs", 
    "pcall", "print", "rawequal", "rawget", "rawlen", "rawset", 
    "require", "select", "setmetatable", "tonumber", "tostring", 
    "type", "xpcall", "coroutine", "debug", "io", "math", "os", 
    "package", "string", "table", "utf8"
}

local standard_set = {}
for _, v in ipairs(lua_standard) do
    standard_set[v] = true
end

local custom_globals = {}
for k, _ in pairs(_G) do
    if not standard_set[k] then
        table.insert(custom_globals, k)
    end
end

if #custom_globals > 0 then
    table.sort(custom_globals)
    print("Custom globals: " .. table.concat(custom_globals, ", "))
else
    print("No custom globals found!")
end

print("\n=== Test Complete ===")