-- Test basic agent creation without LLM providers
-- ABOUTME: Simple test to verify agent API works

print("Testing basic agent creation...")

-- Test Agent.list() function
print("\nListing available agents:")
local agents = Agent.list()
print("Found " .. #agents .. " agents")

-- Test Agent.discover() function
print("\nDiscovering agent types:")
local types = Agent.discover()
print("Available agent types: " .. #types)
for i, t in ipairs(types) do
    print("  " .. i .. ". Type: " .. t.type)
end

print("\nTest completed!")