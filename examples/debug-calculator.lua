-- Debug Calculator Tool
print("=== Debugging Calculator Tool ===\n")

local calc = Tool.get("calculator")
if not calc then
    print("✗ Calculator tool not found")
    return
end

print("✓ Calculator tool loaded")

-- Check schema
if calc.getSchema then
    local schema = calc:getSchema()
    print("\nTool Schema:")
    print(JSON.stringify(schema))
end

-- Try simple calculation with full result inspection
print("\n\nTesting: 2 + 2")
local result = calc:execute({ input = "2 + 2" })

print("\nFull result:")
print(JSON.stringify(result))

-- Check if it's an error
if result.error then
    print("\nError found: " .. tostring(result.error))
elseif result.result then
    print("\nSuccess! Result value: " .. tostring(result.result.result))
end