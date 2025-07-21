-- Simple Tool Test
print("=== Simple Tool Test ===\n")

-- List tools (they're just names)
local tools = Tool.list()
print("Found " .. #tools .. " tools:")
for i = 1, math.min(5, #tools) do
    print("  " .. i .. ". " .. tools[i])
end
if #tools > 5 then
    print("  ... and " .. (#tools - 5) .. " more")
end

-- Get and test calculator
print("\n=== Calculator Test ===")
local calc = Tool.get("calculator")
if calc then
    print("✓ Got calculator tool")
    
    -- Get tool info
    if calc.name then
        print("  Name: " .. calc.name)
    end
    if calc.schema then
        print("  Has schema: yes")
    end
    
    -- Try a calculation
    print("\nCalculating 2 + 2 * 3:")
    local result = calc:execute({ operation = "evaluate", input = "2 + 2 * 3" })
    if result then
        print("Result: " .. tostring(result.output or result.text or "no output"))
    else
        print("Result: calculation failed")
    end
else
    print("✗ Calculator not found")
end

print("\n=== Test Complete ===")