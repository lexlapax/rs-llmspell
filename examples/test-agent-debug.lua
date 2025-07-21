-- Debug Agent global
print("Testing Agent global...")

-- Check if Agent exists
if Agent then
    print("✓ Agent global exists")
else
    print("✗ Agent global not found")
    return
end

-- Check what's in the Agent table
print("\nAgent table contents:")
for k, v in pairs(Agent) do
    print("  " .. k .. " = " .. type(v))
end

-- Try to call list
print("\nTrying Agent.list()...")
local success, result = pcall(Agent.list)
if success then
    print("✓ Agent.list() succeeded, returned " .. type(result))
    if type(result) == "table" then
        print("  Length: " .. #result)
    end
else
    print("✗ Agent.list() failed: " .. tostring(result))
end

-- Try to call discover if it exists
if Agent.discover then
    print("\nTrying Agent.discover()...")
    local success2, result2 = pcall(Agent.discover)
    if success2 then
        print("✓ Agent.discover() succeeded, returned " .. type(result2))
        if type(result2) == "table" then
            print("  Length: " .. #result2)
        end
    else
        print("✗ Agent.discover() failed: " .. tostring(result2))
    end
else
    print("\n✗ Agent.discover not found in Agent table")
end

print("\nDebug completed!")