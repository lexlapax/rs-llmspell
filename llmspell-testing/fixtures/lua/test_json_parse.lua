-- Check if JSON parsing is available

print("Checking for JSON support...")

-- Check if json module exists
local json_ok, json = pcall(require, "json")
if json_ok then
    print("  ✅ json module available")
else
    print("  ❌ json module not available:", json)
end

-- Check if cjson exists
local cjson_ok, cjson = pcall(require, "cjson")
if cjson_ok then
    print("  ✅ cjson module available")
else
    print("  ❌ cjson module not available")
end

-- Check if dkjson exists
local dkjson_ok, dkjson = pcall(require, "dkjson")
if dkjson_ok then
    print("  ✅ dkjson module available")
else
    print("  ❌ dkjson module not available")
end

-- Check global JSON
if JSON then
    print("  ✅ Global JSON available")
else
    print("  ❌ Global JSON not available")
end

-- Test if output is already parsed by checking a real tool
print("\nTesting real tool output:")
local tool = Tool.get("uuid_generator")
if tool then
    local result = tool.execute({operation = "generate"})
    print("  Result type:", type(result))
    print("  Output type:", type(result.output))
    
    -- The working examples access result.result, let's see if that exists
    print("  Has result.result?", result.result ~= nil)
    
    -- Maybe the bridge auto-parses?
    if result.output and type(result.output) == "string" then
        -- Try to access as if it was parsed
        print("\nChecking if examples work differently...")
        
        -- Run through test helper
        local TestHelpers = dofile("examples/test-helpers.lua")
        local helper_result = TestHelpers.execute_tool("uuid_generator", {operation = "generate"})
        
        print("  Helper result type:", type(helper_result))
        print("  Helper has result?", helper_result.result ~= nil)
        print("  Helper has output?", helper_result.output ~= nil)
        
        -- Print full structure
        print("\nFull helper result structure:")
        for k, v in pairs(helper_result) do
            print("    " .. k .. ":", type(v))
            if type(v) == "table" then
                for k2, v2 in pairs(v) do
                    print("      " .. k2 .. ":", type(v2), tostring(v2):sub(1, 30))
                end
            end
        end
    end
end