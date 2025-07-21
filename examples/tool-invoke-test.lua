-- Tool Invoke Test
print("=== Tool Invoke Test ===\n")

-- Get calculator tool
local calc = Tool.get("calculator")
if not calc then
    print("✗ Calculator tool not found")
    return
end

print("✓ Got calculator tool")

-- Inspect tool object
print("\nTool object structure:")
for k, v in pairs(calc) do
    print("  " .. k .. " = " .. type(v))
end

-- Try different invocation methods
print("\n=== Testing invocation methods ===")

-- Method 1: Direct invoke
if calc.invoke then
    print("\n1. Using calc.invoke():")
    local success, result = pcall(calc.invoke, calc, { input = "5 + 3" })
    if success then
        print("   Result: " .. tostring(result.output or result.text or result))
    else
        print("   Failed: " .. tostring(result))
    end
end

-- Method 2: execute
if calc.execute then
    print("\n2. Using calc.execute():")
    local success, result = pcall(calc.execute, calc, { input = "10 - 4" })
    if success then
        print("   Result: " .. tostring(result.output or result.text or result))
    else
        print("   Failed: " .. tostring(result))
    end
end

-- Method 3: run
if calc.run then
    print("\n3. Using calc.run():")
    local success, result = pcall(calc.run, calc, { input = "3 * 4" })
    if success then
        print("   Result: " .. tostring(result.output or result.text or result))
    else
        print("   Failed: " .. tostring(result))
    end
end

print("\n=== Test Complete ===")