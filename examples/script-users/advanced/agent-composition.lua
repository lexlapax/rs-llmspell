-- Example: Agent Composition Patterns
-- Purpose: Demonstrates agent composition patterns, wrapping agents as tools, and discovering agents by capability
-- Prerequisites: API key set in environment (OPENAI_API_KEY)
-- Expected Output: Agent composition demonstrations including composite agents and agent-wrapped tools
-- Version: 0.7.0
-- Tags: agent, composition, advanced

-- ABOUTME: Example demonstrating agent composition patterns in LLMSpell
-- ABOUTME: Shows how to wrap agents as tools, create composite agents, and discover agents by capability

-- Helper function to print results
local function print_result(name, result)
    print("\n=== " .. name .. " ===")
    if type(result) == "table" then
        for k, v in pairs(result) do
            if type(v) == "table" then
                print(k .. ": <table>")
            else
                print(k .. ": " .. tostring(v))
            end
        end
    else
        print(tostring(result))
    end
end

-- Create multiple agents with different capabilities
print("\n=== Creating Agents ===")

-- Create a research agent using builder pattern
local research_agent = Agent.builder()
    :name("research_agent_comp_1")
    :description("Research assistant for finding and summarizing information")
    :agent_type("llm")
    :model({
        provider = "openai",
        model_id = "gpt-3.5-turbo",
        temperature = 0.3,
        max_tokens = 200,
        settings = {}
    })
    :custom_config({
        system_prompt = "You are a research assistant. Focus on finding and summarizing information."
    })
    :allowed_tools({"web_search", "text_manipulator"})
    :resource_limits({
        max_execution_time_secs = 60,
        max_memory_mb = 256,
        max_tool_calls = 10,
        max_recursion_depth = 5
    })
    :build()
local research_agent_name = research_agent.name
print("Registered research agent: " .. research_agent_name)

-- Create an analysis agent using builder pattern
local analysis_agent = Agent.builder()
    :name("analysis_agent_comp_2")
    :description("Data analyst for patterns and insights")
    :agent_type("llm")
    :model({
        provider = "openai",
        model_id = "gpt-3.5-turbo",
        temperature = 0.5,
        max_tokens = 300,
        settings = {}
    })
    :custom_config({
        system_prompt = "You are a data analyst. Focus on analyzing patterns and providing insights."
    })
    :allowed_tools({"calculator", "data_validation", "json_processor"})
    :resource_limits({
        max_execution_time_secs = 90,
        max_memory_mb = 512,
        max_tool_calls = 20,
        max_recursion_depth = 5
    })
    :build()
local analysis_agent_name = analysis_agent.name
print("Registered analysis agent: " .. analysis_agent_name)

-- Create a writer agent using builder pattern
local writer_agent = Agent.builder()
    :name("writer_agent_comp_3")
    :description("Creative writer for content generation")
    :agent_type("llm")
    :model({
        provider = "openai",
        model_id = "gpt-3.5-turbo",
        temperature = 0.8,
        max_tokens = 500,
        settings = {}
    })
    :custom_config({
        system_prompt = "You are a creative writer. Focus on producing well-written content."
    })
    :allowed_tools({"template_engine", "text_manipulator"})
    :resource_limits({
        max_execution_time_secs = 120,
        max_memory_mb = 256,
        max_tool_calls = 5,
        max_recursion_depth = 3
    })
    :build()
local writer_agent_name = writer_agent.name
print("Registered writer agent: " .. writer_agent_name)

-- List all agent capabilities
print_result("Agent Capabilities", Agent.list_capabilities())

-- Get detailed info about agent type
print_result("LLM Agent Type Info", Agent.get_info("llm"))

-- Wrap an agent as a tool
print("\n=== Agent-as-Tool Composition ===")
local research_tool_name = Agent.wrapAsTool(research_agent_name, {
    tool_name = "research_tool",
    description = "Research agent wrapped as a tool for other agents to use"
})
print("Wrapped research agent as tool: " .. research_tool_name)

-- Verify the tool is available
local tools = Tool.list()
print("\nAvailable tools after wrapping:")
for _, tool in ipairs(tools) do
    if tool.name and string.find(tool.name, "research") then
        print("  - " .. tool.name)
    end
end

-- Create a composite agent that delegates to multiple agents
print("\n=== Creating Composite Agent ===")
Agent.createComposite("composite_analyst", 
    {research_agent_name, analysis_agent_name}, 
    {
        routing_strategy = "capability_based",
        description = "Composite agent that coordinates research and analysis"
    }
)
print("Created composite agent: composite_analyst")

-- List all agent instances including composite
local instances = Agent.list()
print("\nAll agent instances:")
for _, instance in ipairs(instances) do
    print("  - " .. (instance.name or tostring(instance)))
end

-- Discover agents by capability
print("\n=== Agent Discovery by Capability ===")

local streaming_agents = Agent.discoverByCapability("streaming")
print("\nAgents with streaming capability:")
for _, agent in ipairs(streaming_agents) do
    print("  - " .. agent)
end

local composite_agents = Agent.discoverByCapability("composite")
print("\nComposite agents:")
for _, agent in ipairs(composite_agents) do
    print("  - " .. agent)
end

-- Get composite agent info
print("\n=== Composite Agent Info ===")
local composite_agent = Agent.get("composite_analyst")
if composite_agent then
    print("Composite agent retrieved successfully")
else
    print("Note: Agent.getHierarchy() not implemented")
end

-- Use the composite agent
print("\n=== Using Composite Agent ===")
local composite = Agent.get("composite_analyst")
if composite then
    local success, result = pcall(function()
        return composite:invoke({
            text = "Research the latest trends in AI and analyze their potential impact on business"
        })
    end)
    
    if success and result and result.text then
        print("\nComposite agent result:")
        print(result.text)
    else
        print("\nComposite agent execution failed: " .. tostring(result))
    end
else
    print("Failed to get composite agent")
end

-- Create an agent that leverages the wrapped tool
print("\n=== Using Agent with Wrapped Tool ===")
local enhanced_writer = Agent.builder()
    :name("enhanced_writer_comp_4")
    :description("Writer that uses research tools")
    :agent_type("llm")
    :model({
        provider = "openai",
        model_id = "gpt-3.5-turbo",
        temperature = 0.7,
        max_tokens = 400,
        settings = {}
    })
    :custom_config({
        system_prompt = "You are a writer who can use research tools to create well-informed content."
    })
    :allowed_tools({"research_tool", "template_engine", "text_manipulator"})
    :resource_limits({
        max_execution_time_secs = 90,
        max_memory_mb = 256,
        max_tool_calls = 15,
        max_recursion_depth = 5
    })
    :build()
local enhanced_writer_name = enhanced_writer.name
print("Created enhanced writer: " .. enhanced_writer_name)

-- List tools to verify research_tool is available
local tools = Tool.list()
print("\nChecking for agent-wrapped research tool:")
for _, tool in ipairs(tools) do
    if tool.name == "research_tool" then
        print("  ✓ Found: " .. tool.name .. " - " .. tool.description)
    end
end

-- Demonstrate nested composition
print("\n=== Nested Composition ===")

-- Create a meta-coordinator that uses the composite agent
Agent.createComposite("meta_coordinator",
    {"composite_analyst", enhanced_writer_name},
    {
        routing_strategy = "task_based",
        description = "Meta-coordinator that orchestrates complex multi-step tasks"
    }
)
print("Created meta-coordinator with nested composition")

-- List all agents to show hierarchy
local all_agents = Agent.list()
print("\nFinal agent hierarchy:")
for _, agent in ipairs(all_agents) do
    print("  - " .. (agent.name or "unnamed"))
end

-- Clean up
print("\n=== Cleanup ===")
print("Example completed - agents remain available for further use")