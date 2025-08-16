-- Example: Agent Basics - Creating and Using Agents
-- Purpose: Introduction to agent creation, listing, and basic execution
-- Prerequisites: OpenAI API key (OPENAI_API_KEY) for best results
-- Expected Output: Agent creation, invocation, listing, and discovery demo
-- Version: 0.7.0
-- Tags: getting-started, agents, api-required

-- ABOUTME: Simple agent demonstration using only available Agent API methods
-- ABOUTME: Shows basic agent creation, listing, and execution

print("=== Simple Agent API Demo ===")
print()

-- Test 1: Create a basic agent
print("1. Creating a basic agent...")
local success1, agent1 = pcall(function()
    return Agent.builder()
        :name("simple-agent")
        :model("gpt-4o-mini")
        :system_prompt("You are a helpful assistant. Keep responses brief.")
        :build()
end)

if success1 and agent1 then
    print("   ✓ Agent created successfully")
    
    -- Test agent execution
    print("\n2. Testing agent execution...")
    local success2, response = pcall(function()
        return agent1:invoke({text = "What is 2 + 2?"})
    end)
    
    if success2 and response then
        if type(response) == "table" and response.text then
            print("   Agent response: " .. response.text)
        else
            print("   Agent response: " .. tostring(response))
        end
    else
        print("   ✗ Execution failed: " .. tostring(response))
    end
else
    print("   ✗ Failed to create agent: " .. tostring(agent1))
end

-- Test 2: Create multiple agents with different models
print("\n3. Creating agents with different models...")
local test_models = {
    {model = "gpt-4o-mini", name = "OpenAI GPT-4 Mini"},
    {model = "anthropic/claude-3-5-sonnet-20241022", name = "Claude 3.5 Sonnet"},
    {model = "gpt-3.5-turbo", name = "GPT 3.5 Turbo"}
}

local created_agents = 0
for i, test in ipairs(test_models) do
    local success, agent = pcall(function()
        return Agent.builder()
            :name("test-agent-" .. i)
            :model(test.model)
            :system_prompt("You are a test agent.")
            :build()
    end)
    
    if success and agent then
        print("   ✓ " .. test.name .. " - created")
        created_agents = created_agents + 1
    else
        print("   ✗ " .. test.name .. " - failed: " .. tostring(agent))
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

-- Test 4: Test discover method
print("\n5. Agent discovery...")
local success, agent_types = pcall(function()
    return Agent.discover()
end)

if success and type(agent_types) == "table" then
    print("   Available agent types:")
    for i, agent_type in ipairs(agent_types) do
        if type(agent_type) == "table" and agent_type.type then
            print("   - " .. agent_type.type)
        else
            print("   - " .. tostring(agent_type))
        end
    end
else
    print("   ✗ Failed to discover agents: " .. tostring(agent_types))
end

-- Test 5: Agent with custom parameters
print("\n6. Creating agent with custom parameters...")
local success, custom_agent = pcall(function()
    return Agent.builder()
        :name("custom-agent")
        :model("gpt-4o-mini")
        :system_prompt("You are a creative writer. Use vivid language.")
        :temperature(0.9)
        :max_tokens(150)
        :build()
end)

if success and custom_agent then
    print("   ✓ Custom agent created")
    
    -- Test with creative prompt
    local success2, response = pcall(function()
        return custom_agent:invoke({text = "Describe a sunset in one sentence."})
    end)
    
    if success2 and response then
        if type(response) == "table" and response.text then
            print("   Creative response: " .. response.text)
        else
            print("   Creative response: " .. tostring(response))
        end
    else
        print("   ✗ Execution failed: " .. tostring(response))
    end
else
    print("   ✗ Failed to create custom agent: " .. tostring(custom_agent))
end

print("\n=== Demo Complete ===")
print("\nNote: Some operations may fail due to:")
print("- Missing API keys")
print("- Network connectivity")
print("- Model availability")
print("\nThis demo uses the Agent API methods:")
print("- Agent.builder() - Create agents synchronously")
print("- Agent.list() - List agent instances")
print("- Agent.discover() - Discover agent types")
print("- Agent.register() - Register new agents")
print("- Agent.get() - Get existing agents")
print("- Agent.getInfo() - Get agent type information")
print("- Agent.listCapabilities() - List agent capabilities")
print("- Agent.wrapAsTool() - Wrap agents as tools")
print("- Agent.createComposite() - Create composite agents")
print("- Agent.discoverByCapability() - Find agents by capability")
print("- agent:invoke() - Execute agent tasks")