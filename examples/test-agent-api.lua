-- Test Agent API functions
print("Testing Agent API...")

-- List all instances
print("\n1. Agent.list() - List active agent instances:")
local instances = Agent.list()
print("   Found " .. #instances .. " active instances")

-- List all available agent instances (might be different)
print("\n2. Agent.listInstances() - List all instances:")
local all_instances = Agent.listInstances()
print("   Found " .. #all_instances .. " instances")

-- List templates
print("\n3. Agent.listTemplates() - List agent templates:")
local success, templates = pcall(Agent.listTemplates)
if success then
    print("   Found " .. #templates .. " templates")
    for i, template in ipairs(templates) do
        print("     " .. i .. ". " .. tostring(template))
    end
else
    print("   Failed: " .. tostring(templates))
end

-- List capabilities 
print("\n4. Agent.listCapabilities() - List agent capabilities:")
local success2, caps = pcall(Agent.listCapabilities)
if success2 then
    print("   Found " .. #caps .. " capabilities")
else
    print("   Failed: " .. tostring(caps))
end

-- Try to create a simple agent with just a model name
print("\n5. Creating a simple agent:")
local success3, agent = pcall(Agent.create, {
    model = "basic"
})
if success3 then
    print("   ✓ Agent created successfully")
else
    print("   ✗ Failed: " .. tostring(agent))
end

print("\nAPI test completed!")