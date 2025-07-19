-- ABOUTME: Example demonstrating agent composition patterns in LLMSpell
-- ABOUTME: Shows how to wrap agents as tools, create composite agents, and discover agents by capability

local llmspell = require("llmspell")

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

-- Create a research agent
local research_agent = Agent.create({
    name = "research_agent",
    system_prompt = "You are a research assistant. Focus on finding and summarizing information.",
    temperature = 0.3
})
print("Created research agent")

-- Create an analysis agent
local analysis_agent = Agent.create({
    name = "analysis_agent", 
    system_prompt = "You are a data analyst. Focus on analyzing patterns and providing insights.",
    temperature = 0.5
})
print("Created analysis agent")

-- Create a writer agent
local writer_agent = Agent.create({
    name = "writer_agent",
    system_prompt = "You are a creative writer. Focus on producing well-written content.",
    temperature = 0.8
})
print("Created writer agent")

-- List all agent capabilities
print_result("Agent Capabilities", Agent.listCapabilities())

-- Get detailed info about an agent
print_result("Research Agent Info", Agent.getInfo("research_agent"))

-- Wrap an agent as a tool
print("\n=== Agent-as-Tool Composition ===")
local research_tool_name = Agent.wrapAsTool("research_agent", {
    tool_name = "research_tool",
    description = "Research agent wrapped as a tool for other agents to use"
})
print("Wrapped research agent as tool: " .. research_tool_name)

-- Verify the tool is available
local tools = Tool.list()
print("\nAvailable tools after wrapping:")
for _, tool in ipairs(tools) do
    if string.find(tool, "research") then
        print("  - " .. tool)
    end
end

-- Create a composite agent that delegates to multiple agents
print("\n=== Creating Composite Agent ===")
Agent.createComposite("composite_analyst", 
    {"research_agent", "analysis_agent"}, 
    {
        routing_strategy = "capability_based",
        description = "Composite agent that coordinates research and analysis"
    }
)
print("Created composite agent: composite_analyst")

-- List all agent instances including composite
local instances = Agent.listInstances()
print("\nAll agent instances:")
for _, instance in ipairs(instances) do
    print("  - " .. instance)
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

-- Get composition hierarchy
print_result("Composite Agent Hierarchy", Agent.getHierarchy("composite_analyst"))

-- Use the composite agent
print("\n=== Using Composite Agent ===")
local result = composite_analyst:execute({
    text = "Research the latest trends in AI and analyze their potential impact on business"
})
print("\nComposite agent result:")
print(result.text)

-- Use an agent that leverages the wrapped tool
print("\n=== Using Agent with Wrapped Tool ===")
local enhanced_writer = Agent.create({
    name = "enhanced_writer",
    system_prompt = "You are a writer who can use research tools to create well-informed content.",
    temperature = 0.7
})

-- The enhanced writer can now discover and use the research tool
local available_tools = enhanced_writer:listTools()
print("\nTools available to enhanced writer:")
for _, tool in ipairs(available_tools) do
    if string.find(tool, "research") then
        print("  - " .. tool .. " (agent-wrapped tool)")
    end
end

-- Execute with tool usage
local informed_result = enhanced_writer:execute({
    text = "Write an article about quantum computing, using research tools to gather accurate information"
})
print("\nEnhanced writer result:")
print(informed_result.text)

-- Demonstrate nested composition
print("\n=== Nested Composition ===")

-- Create a meta-coordinator that uses the composite agent
Agent.createComposite("meta_coordinator",
    {"composite_analyst", "enhanced_writer"},
    {
        routing_strategy = "task_based",
        description = "Meta-coordinator that orchestrates complex multi-step tasks"
    }
)

local meta_info = Agent.getInfo("meta_coordinator")
print_result("Meta-Coordinator Info", meta_info)

-- Clean up
print("\n=== Cleanup ===")
print("Example completed - agents remain available for further use")