-- Example: Agent API Comprehensive
-- Purpose: Comprehensive demonstration of the Agent API showing creation, discovery, and capabilities
-- Prerequisites: OpenAI API key (OPENAI_API_KEY) for agent creation
-- Expected Output: Agent registration, discovery, capabilities listing, and tool wrapping demo
-- Version: 0.7.0
-- Tags: features, agents, api-comprehensive, api-required

-- ABOUTME: Comprehensive Agent API demonstration showing all available methods
-- ABOUTME: Covers creation, discovery, composition, tool wrapping, and capabilities

print("=== Comprehensive Agent API Demo ===\n")

-- 1. Test Agent.register() - Register new agents
print("1. Testing Agent.register()...")
local agent_name = Agent.register({
    name = "demo-analyst",
    description = "Data analysis agent",
    agent_type = "llm",
    model = {
        provider = "openai",
        model_id = "gpt-3.5-turbo",
        temperature = 0.3,
        max_tokens = 200,
        settings = {}
    },
    allowed_tools = {"calculator", "data_validation"},
    custom_config = {
        system_prompt = "You are a data analyst. Provide concise insights."
    },
    resource_limits = {
        max_execution_time_secs = 60,
        max_memory_mb = 256,
        max_tool_calls = 10,
        max_recursion_depth = 5
    }
})
print("   ✓ Registered agent: " .. agent_name)

-- 2. Test Agent.get() - Retrieve existing agent
print("\n2. Testing Agent.get()...")
local retrieved_agent = Agent.get(agent_name)
if retrieved_agent then
    print("   ✓ Retrieved agent successfully")
    
    -- Test invoke
    local success, result = pcall(function()
        return retrieved_agent:invoke({
            text = "Analyze this data: Sales Q1: $100k, Q2: $120k, Q3: $95k, Q4: $140k"
        })
    end)
    
    if success and result and result.text then
        print("   Analysis: " .. result.text)
    else
        print("   ✗ Invoke failed: " .. tostring(result))
    end
else
    print("   ✗ Failed to retrieve agent")
end

-- 3. Test Agent.get_info() - Get agent type information (corrected method name)
print("\n3. Testing Agent.get_info()...")
local success3, llm_info = pcall(function()
    return Agent.get_info("llm")
end)

if success3 and llm_info then
    print("   LLM Agent Info:")
    print("   - Name: " .. (llm_info.name or "N/A"))
    print("   - Description: " .. (llm_info.description or "N/A"))
    if llm_info.required_parameters then
        print("   - Required parameters: " .. #llm_info.required_parameters)
    end
else
    print("   ✗ Failed to get agent info: " .. tostring(llm_info))
end

-- 4. Test Agent.list_capabilities() - List all capabilities (corrected method name)
print("\n4. Testing Agent.list_capabilities()...")
local success4, capabilities = pcall(function()
    return Agent.list_capabilities()
end)

if success4 and capabilities then
    print("   Available agent capabilities:")
    local count = 0
    for name, cap in pairs(capabilities) do
        count = count + 1
        print("   - " .. name .. ": " .. (cap.name or "Unknown"))
        if cap.capabilities and #cap.capabilities > 0 then
            print("     Capabilities: " .. table.concat(cap.capabilities, ", "))
        end
        if count >= 3 then
            print("   ... and more")
            break
        end
    end
else
    print("   ✗ Failed to list capabilities: " .. tostring(capabilities))
end

-- 5. Test Agent.wrap_as_tool() - Wrap agent as tool (corrected method name)
print("\n5. Testing Agent.wrap_as_tool()...")
-- Create an agent using Agent.builder() and wrap that
local writer_success, writer_agent = pcall(function()
    return Agent.builder()
        :name("creative-writer")
        :model("openai/gpt-3.5-turbo")
        :system_prompt("You are a creative writer. Write engaging short content.")
        :temperature(0.8)
        :max_tokens(100)
        :build()
end)

if writer_success and writer_agent then
    print("   ✓ Created writer agent")
    
    local wrap_success, tool_wrapper = pcall(function()
        return Agent.wrap_as_tool(writer_agent, {
            name = "creative_writer_tool",
            description = "A tool that generates creative content",
            parameters = {
                prompt = "The writing prompt to use"
            }
        })
    end)
    
    if wrap_success and tool_wrapper then
        print("   ✓ Successfully wrapped agent as tool")
    else
        print("   ✗ Failed to wrap agent as tool: " .. tostring(tool_wrapper))
    end
else
    print("   ✗ Failed to create writer agent: " .. tostring(writer_agent))
end

-- 6. Test Agent.create_composite() - Create composite agent
print("\n6. Testing Agent.create_composite()...")
local composite_success, composite_agent = pcall(function()
    return Agent.create_composite({
        name = "research-team",
        description = "A team of specialized research agents",
        agents = {
            {
                role = "researcher",
                agent_name = agent_name,
                weight = 0.6
            },
            {
                role = "writer", 
                agent_name = "creative-writer",
                weight = 0.4
            }
        },
        coordination_strategy = "sequential"
    })
end)

if composite_success and composite_agent then
    print("   ✓ Created composite agent successfully")
else
    print("   ✗ Failed to create composite agent: " .. tostring(composite_agent))
end

-- 7. Test Agent.discover() - Discover available agent types
print("\n7. Testing Agent.discover()...")
local discover_success, agent_types = pcall(function()
    return Agent.discover()
end)

if discover_success and agent_types then
    print("   Available agent types:")
    for i, agent_type in ipairs(agent_types) do
        if type(agent_type) == "table" and agent_type.type then
            print("   - " .. agent_type.type)
        else
            print("   - " .. tostring(agent_type))
        end
        if i >= 5 then
            print("   ... and more")
            break
        end
    end
else
    print("   ✗ Failed to discover agent types: " .. tostring(agent_types))
end

-- 8. Test Agent.discover_by_capability() - Find agents by capability
print("\n8. Testing Agent.discover_by_capability()...")
local capability_search_success, capability_agents = pcall(function()
    return Agent.discover_by_capability("text_generation")
end)

if capability_search_success and capability_agents then
    print("   Agents with text_generation capability:")
    for i, agent in ipairs(capability_agents) do
        print("   - " .. (agent.name or "Unknown"))
        if i >= 3 then
            print("   ... and more")
            break
        end
    end
else
    print("   ✗ Failed to discover by capability: " .. tostring(capability_agents))
end

-- 9. Test Agent.list() - List all agent instances
print("\n9. Testing Agent.list()...")
local list_success, all_agents = pcall(function()
    return Agent.list()
end)

if list_success and all_agents then
    print("   All agent instances:")
    for i, agent_info in ipairs(all_agents) do
        if type(agent_info) == "table" and agent_info.name then
            print("   - " .. agent_info.name)
        elseif type(agent_info) == "string" then
            print("   - " .. agent_info)
        else
            print("   - Agent " .. i)
        end
        if i >= 5 then
            print("   ... and more")
            break
        end
    end
else
    print("   ✗ Failed to list agents: " .. tostring(all_agents))
end

print("\n=== Agent API Demo Complete ===")
print("\nKey Demonstrated APIs:")
print("- Agent.register() - Register new agent types")
print("- Agent.get() - Retrieve existing agents")
print("- Agent.get_info() - Get agent type information")
print("- Agent.list_capabilities() - List agent capabilities")
print("- Agent.wrap_as_tool() - Wrap agents as tools")
print("- Agent.create_composite() - Create composite agents")
print("- Agent.discover() - Discover available agent types")
print("- Agent.discover_by_capability() - Find agents by capability")
print("- Agent.list() - List all agent instances")
print("- Agent.builder() - Create agents with builder pattern")

print("\nNote: Some operations may fail due to:")
print("- Missing API keys")
print("- Network connectivity")
print("- Model availability")
print("- API changes or limitations")