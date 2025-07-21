-- Working Tools Test
-- Tests tool discovery and usage

print("=== Tool Discovery Test ===\n")

-- Check if Tool global exists
if not Tool then
    print("✗ Tool global not found")
    return
end

print("✓ Tool global exists")

-- List all tools
local tools = Tool.list()
print("\nFound " .. #tools .. " tools")

-- Inspect tool structure
if #tools > 0 then
    print("\nInspecting first tool structure:")
    local first_tool = tools[1]
    for k, v in pairs(first_tool) do
        print("  " .. k .. " = " .. type(v))
    end
end

-- Try to get a specific tool
print("\n=== Testing Tool.get() ===")
local calc_tool = Tool.get("calculator")
if calc_tool then
    print("✓ Got calculator tool")
    
    -- Try to use it
    print("\nTesting calculator tool:")
    local success, result = pcall(function()
        return calc_tool({
            input = "2 + 2"
        })
    end)
    
    if success then
        print("✓ Calculator result: " .. tostring(result.output or result.text or result))
    else
        print("✗ Calculator failed: " .. tostring(result))
    end
else
    print("✗ Calculator tool not found")
end

-- Test Tool.exists
print("\n=== Testing Tool.exists() ===")
print("calculator exists: " .. tostring(Tool.exists("calculator")))
print("nonexistent exists: " .. tostring(Tool.exists("nonexistent")))

-- Tool.categories() is not available in current API
print("\n=== Tool Summary ===")
print("Tool.categories() is not available in current API")
print("Total tools available: " .. #Tool.list())

print("\n=== Test Complete ===")