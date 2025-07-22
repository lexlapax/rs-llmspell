-- ABOUTME: Simple agent demonstration using only available Agent API methods
-- ABOUTME: Shows basic agent creation, listing, and execution

-- Helper function to safely create an agent
local function safe_create_agent(config)
    local success, agent = pcall(function()
        return Agent.create(config)
    end)
    if success then
        return agent, nil
    else
        return nil, tostring(agent)
    end
end

print("=== Simple Agent API Demo ===")
print()

-- Test 1: Create a basic agent
print("1. Creating a basic agent...")
local agent1, err1 = safe_create_agent({
    model = "gpt-4o-mini",
    system_prompt = "You are a helpful assistant. Keep responses brief."
})

if agent1 then
    print("   ✓ Agent created successfully")
    
    -- Test agent execution
    print("\n2. Testing agent execution...")
    local success, response = pcall(function()
        return agent1:execute({text = "What is 2 + 2?"})
    end)
    
    if success then
        if type(response) == "table" and response.text then
            print("   Agent response: " .. response.text)
        else
            print("   Agent response: " .. tostring(response))
        end
    else
        print("   ✗ Execution failed: " .. tostring(response))
    end
else
    print("   ✗ Failed to create agent: " .. err1)
end

-- Test 2: Create multiple agents with different models
print("\n3. Creating agents with different models...")
local test_models = {
    {model = "gpt-4o-mini", name = "OpenAI GPT-4 Mini"},
    {model = "anthropic/claude-3-5-sonnet-20241022", name = "Claude 3.5 Sonnet"},
    {model = "gpt-3.5-turbo", name = "GPT 3.5 Turbo"}
}

local created_agents = 0
for _, test in ipairs(test_models) do
    local agent, err = safe_create_agent({
        model = test.model,
        system_prompt = "You are a test agent."
    })
    
    if agent then
        print("   ✓ " .. test.name .. " - created")
        created_agents = created_agents + 1
    else
        print("   ✗ " .. test.name .. " - failed: " .. err)
    end
end

print("\n   Created " .. created_agents .. " out of " .. #test_models .. " agents")

-- Test 3: List available agents
print("\n4. Listing available agents...")
local success, agents = pcall(function()
    return Agent.list()
end)

if success then
    print("   Available agents:")
    if type(agents) == "table" then
        for i, agent_info in ipairs(agents) do
            if type(agent_info) == "table" and agent_info.name then
                print("   - " .. agent_info.name)
            elseif type(agent_info) == "string" then
                print("   - " .. agent_info)
            else
                print("   - Agent " .. i)
            end
        end
    else
        print("   (No agents listed or unexpected format)")
    end
else
    print("   ✗ Failed to list agents: " .. tostring(agents))
end

-- Test 4: Skip discover (method doesn't exist yet)
print("\n5. Agent discovery...")
print("   (Agent.discover() not yet implemented)")

-- Test 5: Agent with custom parameters
print("\n6. Creating agent with custom parameters...")
local custom_agent, err = safe_create_agent({
    model = "gpt-4o-mini",
    system_prompt = "You are a creative writer. Use vivid language.",
    temperature = 0.9,
    max_tokens = 150
})

if custom_agent then
    print("   ✓ Custom agent created")
    
    -- Test with creative prompt
    local success, response = pcall(function()
        return custom_agent:execute({text = "Describe a sunset in one sentence."})
    end)
    
    if success then
        if type(response) == "table" and response.text then
            print("   Creative response: " .. response.text)
        else
            print("   Creative response: " .. tostring(response))
        end
    else
        print("   ✗ Execution failed: " .. tostring(response))
    end
else
    print("   ✗ Failed to create custom agent: " .. err)
end

print("\n=== Demo Complete ===")
print("\nNote: Some operations may fail due to:")
print("- Missing API keys")
print("- Network connectivity")
print("- Model availability")
print("\nThis demo uses only the currently available Agent API methods:")
print("- Agent.create()")
print("- Agent.list()")
print("- Agent.discover() [not yet implemented]")
print("- agent:execute()")