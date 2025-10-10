-- Test a single tool to understand output format

-- Load test helpers
local TestHelpers = dofile("../../examples/test-helpers.lua")

-- Use the helper function
local result = TestHelpers.execute_tool("uuid-generator", {
    operation = "generate"
})

print("Result from TestHelpers.execute_tool:")
print("  success:", result.success)
print("  error:", result.error)
print("  output type:", type(result.output))

-- Check if output needs parsing
if result.output and type(result.output) == "string" then
    print("\nOutput is a string, content:")
    print(result.output)
    
    -- Try to parse it as JSON
    local success, parsed = pcall(function()
        -- Lua doesn't have built-in JSON, but the output shows it's already parsed
        -- in the working examples. Let's check what's different
        return "JSON parsing would happen here"
    end)
end

-- Check if result has nested structure
if result.result then
    print("\nFound result.result:")
    for k, v in pairs(result.result) do
        print("  " .. k .. ":", v)
    end
end

-- Try direct tool access
print("\n\nDirect tool access:")
local tool = Tool.get("uuid-generator")
if tool then
    local direct_result = tool.execute({operation = "generate"})
    print("Direct result type:", type(direct_result))
    if type(direct_result) == "table" then
        for k, v in pairs(direct_result) do
            print("  " .. k .. ":", type(v), tostring(v):sub(1, 50))
        end
    end
end