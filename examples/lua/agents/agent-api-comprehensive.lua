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

-- 3. Test Agent.getInfo() - Get agent type information
print("\n3. Testing Agent.getInfo()...")
local llm_info = Agent.getInfo("llm")
if llm_info then
    print("   LLM Agent Info:")
    print("   - Name: " .. (llm_info.name or "N/A"))
    print("   - Description: " .. (llm_info.description or "N/A"))
    if llm_info.required_parameters then
        print("   - Required parameters: " .. #llm_info.required_parameters)
    end
else
    print("   ✗ Failed to get agent info")
end

-- 4. Test Agent.listCapabilities() - List all capabilities
print("\n4. Testing Agent.listCapabilities()...")
local capabilities = Agent.listCapabilities()
if capabilities then
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
    print("   ✗ Failed to list capabilities")
end

-- 5. Test Agent.wrapAsTool() - Wrap agent as tool
print("\n5. Testing Agent.wrapAsTool()...")
-- Skip Agent.register for now - it has a different format requirement
-- Instead, create an agent using Agent.create and wrap that
local writer_success, writer_agent = pcall(function()
    return Agent.create({
        name = "creative-writer",
        model = "openai/gpt-3.5-turbo",
        system_prompt = "You are a creative writer. Write engaging short content.",
        temperature = 0.8,
        max_tokens = 100
    })
end)

if not writer_success then
    print("   ✗ Failed to create writer agent: " .. tostring(writer_agent))
    print("   Skipping wrapAsTool test")
else
    local writer_agent_name = "creative-writer"
    
    local tool_name = Agent.wrapAsTool(writer_agent_name, {
        tool_name = "creative_writer_tool",
        description = "Tool for creative writing tasks"
    })
    print("   ✓ Wrapped agent as tool: " .. tool_name)
    
    -- Check if tool exists
    if Tool and Tool.exists then
        local exists = Tool.exists(tool_name)
        print("   Tool registered: " .. tostring(exists))
    end
end

-- 6. Test Agent.discoverByCapability() - Find agents by capability
print("\n6. Testing Agent.discoverByCapability()...")
local streaming_agents = Agent.discoverByCapability("streaming")
print("   Agents with streaming capability:")
if #streaming_agents > 0 then
    for i, agent in ipairs(streaming_agents) do
        print("   - " .. agent)
    end
else
    print("   (None found or capability not available)")
end

-- 7. Test Agent.createComposite() - Create composite agent
print("\n7. Testing Agent.createComposite()...")
-- First create another agent using Agent.create
local summarizer_success, summarizer = pcall(function()
    return Agent.create({
        name = "summarizer",
        model = "openai/gpt-3.5-turbo",
        system_prompt = "You are a summarizer. Create brief summaries.",
        temperature = 0.2,
        max_tokens = 100
    })
end)

if not summarizer_success then
    print("   ✗ Failed to create summarizer: " .. tostring(summarizer))
    print("   Skipping createComposite test")
else
    local summarizer_name = "summarizer"
    
    -- Create composite agent
    Agent.createComposite(
        "analysis-and-summary",
        {agent_name, summarizer_name},
        {
            routing_strategy = "sequential",
            description = "Analyzes data then summarizes findings"
        }
    )
    print("   ✓ Created composite agent: analysis-and-summary")
end

-- 8. Test Agent.list() - List all agents
print("\n8. Testing Agent.list()...")
local all_agents = Agent.list()
print("   Current agents (" .. #all_agents .. " total):")
for i, agent_info in ipairs(all_agents) do
    if i <= 5 then
        print("   - " .. (agent_info.name or "Agent " .. i))
    end
end
if #all_agents > 5 then
    print("   ... and " .. (#all_agents - 5) .. " more")
end

-- 9. Test Agent.discover() - Discover agent types
print("\n9. Testing Agent.discover()...")
local agent_types = Agent.discover()
print("   Available agent types:")
for i, type_info in ipairs(agent_types) do
    print("   - " .. (type_info.type or tostring(type_info)))
end

-- 10. Test invokeStream method
print("\n10. Testing agent:invokeStream()...")
local streaming_agent = Agent.get(agent_name)
if streaming_agent then
    print("   Starting streaming response...")
    local chunks = {}
    
    -- Direct streaming invocation (synchronous)
    local success, result = pcall(function()
        return streaming_agent:invokeStream(
            { text = "Count from 1 to 5 slowly" },
            function(chunk)
                table.insert(chunks, chunk)
                if chunk.text then
                    io.write("   Chunk: " .. chunk.text)
                    io.flush()
                end
            end
        )
    end)
    
    if success and result then
        print("\n   Received " .. #chunks .. " chunks")
    else
        print("\n   ✗ Streaming failed: " .. tostring(result))
    end
else
    print("   ✗ Failed to get agent for streaming")
end

print("\n=== Demo Complete ===")
print("\nKey takeaways:")
print("- Use Agent.register() to create agents with full configuration")
print("- Use Agent.get() to retrieve existing agents")  
print("- Agents can be wrapped as tools for composition")
print("- Composite agents enable multi-agent workflows")
print("- All agent methods support async operations")