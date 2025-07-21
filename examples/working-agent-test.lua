-- Working Agent Test
-- Tests agent creation using discovered templates

print("=== Agent Template Discovery Test ===\n")

-- 1. List available templates
print("Available agent templates:")
local templates = Agent.listTemplates()
for i, template in ipairs(templates) do
    print("  " .. i .. ". " .. template)
end

-- 2. Try to create from a template
print("\nCreating agent from 'basic' template...")
local success, agent = pcall(Agent.createFromTemplate, "basic")
if success then
    print("✓ Successfully created agent from template")
    
    -- Get agent info
    local info = agent:getInfo()
    print("\nAgent info:")
    print("  Name: " .. (info.name or "unnamed"))
    print("  Type: " .. (info.type or "unknown"))
    
    -- Try to invoke the agent
    print("\nInvoking agent with test input...")
    local invoke_success, response = pcall(function()
        return agent:invoke({
            input = "Hello from Lua!"
        })
    end)
    
    if invoke_success then
        print("✓ Agent invoked successfully")
        print("  Response: " .. (response.text or response.output or "no text"))
    else
        print("✗ Agent invocation failed: " .. tostring(response))
    end
else
    print("✗ Failed to create from template: " .. tostring(agent))
end

-- 3. Test Tool listing
print("\n=== Tool Discovery Test ===\n")
if Tool then
    print("Tool global exists")
    local tools = Tool.list()
    print("Found " .. #tools .. " tools:")
    -- Show first 5 tools
    for i = 1, math.min(5, #tools) do
        local tool = tools[i]
        print("  " .. i .. ". " .. tool.name .. " - " .. tool.description)
    end
    if #tools > 5 then
        print("  ... and " .. (#tools - 5) .. " more tools")
    end
else
    print("Tool global not found")
end

print("\n=== Test Complete ===")