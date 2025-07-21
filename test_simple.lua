-- Simple test script
print("Testing basic Lua functionality...")

-- Test that globals are available
print("Agent global available:", Agent ~= nil)
print("Tool global available:", Tool ~= nil)

-- List available tools
print("\nAvailable tools:")
local tools = Tool.list()
if tools then
    print("  Found " .. #tools .. " tools")
    for i, tool in ipairs(tools) do
        if type(tool) == "table" and tool.name then
            print("  " .. i .. ". " .. tool.name)
        elseif type(tool) == "string" then
            print("  " .. i .. ". " .. tool)
        else
            print("  " .. i .. ". (tool type: " .. type(tool) .. ")")
        end
    end
else
    print("  No tools found")
end

print("\nTest completed!")