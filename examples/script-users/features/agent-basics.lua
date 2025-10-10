-- ============================================================
-- LLMSPELL FEATURES SHOWCASE  
-- ============================================================
-- Feature ID: 01 - Agent Basics v0.7.0
-- Complexity Level: INTERMEDIATE
-- Real-World Use Case: Building conversational AI assistants
-- Feature Category: Agents
--
-- Purpose: Core agent functionality - creation, invocation, discovery
-- Architecture: Synchronous agent API with builder pattern
-- Key Capabilities:
--   • Agent.builder() - Fluent agent creation
--   • agent:execute() - Synchronous execution
--   • Agent.list() - Discovery and enumeration
--   • Agent.register() - Custom agent registration
--   • Multiple provider support (OpenAI, Anthropic)
--
-- Prerequisites:
--   • API keys: OPENAI_API_KEY or ANTHROPIC_API_KEY environment variable
--   • Network connectivity for API calls
--
-- HOW TO RUN:
-- ./target/debug/llmspell -p providers run examples/script-users/features/agent-basics.lua
--
-- EXPECTED OUTPUT:
-- Creates 3 agents, demonstrates invocation, shows discovery methods
-- Execution time: <8 seconds with API calls
--
-- Time to Complete: 8 seconds
-- Next Steps: See advanced-patterns/multi-agent-orchestration.lua
-- ============================================================

print("=== Agent Basics - Core Functionality ===\n")

-- 1. BASIC AGENT CREATION
print("1. Creating a conversational agent...")
local success, agent = pcall(function()
    return Agent.builder()
        :name("assistant")
        :model("openai/gpt-3.5-turbo")  -- Format: provider/model
        :system_prompt("You are a helpful AI assistant. Be concise and friendly.")
        :temperature(0.7)
        :max_tokens(150)
        :build()
end)

if success and agent then
    print("   ✓ Agent created successfully")
    
    -- Execute the agent
    local response = agent:execute({
        text = "What are the three primary colors?"
    })
    
    if response and response.text then
        print("   Response: " .. response.text)
    end
else
    print("   ✗ Failed (check API keys): " .. tostring(agent))
end

-- 2. SPECIALIZED AGENT WITH LOWER TEMPERATURE
print("\n2. Creating a code expert agent...")
local code_agent = Agent.builder()
    :name("code-expert")
    :model("openai/gpt-3.5-turbo")
    :system_prompt("You are a programming expert. Provide concise code examples.")
    :temperature(0.3)  -- Lower temperature for focused responses
    :max_tokens(200)
    :build()

if code_agent then
    local result = code_agent:execute({
        text = "Write a one-line Python list comprehension to filter even numbers"
    })
    if result and result.text then
        print("   Code response received (length: " .. string.len(result.text) .. " chars)")
    end
end

-- 3. AGENT REGISTRATION (Advanced API)
print("\n3. Registering a custom agent...")
local success, agent_name = pcall(function()
    return Agent.register({
        name = "data-analyst",
        description = "Data analysis specialist",
        type = "llm",
        model = {
            provider = "openai",
            model_id = "gpt-3.5-turbo",
            temperature = 0.3,
            max_tokens = 200,
            settings = {}  -- Required field
        },
        custom_config = {
            system_prompt = "You are a data analyst. Provide concise insights."
        },
        resource_limits = {
            max_execution_time_secs = 60,
            max_memory_mb = 256
        }
    })
end)
if success then
    print("   ✓ Registered: " .. tostring(agent_name))
else
    print("   ✗ Registration failed (expected - requires full config)")
end

-- 4. AGENT DISCOVERY
print("\n4. Agent discovery methods...")

-- List all agents
local agents = Agent.list()
print("   Found " .. #agents .. " agents in registry")

-- Get specific agent info
local success, info = pcall(function()
    return Agent.get_info("llm")
end)
if success and info then
    print("   LLM agent type available")
end

-- 5. PROVIDER FLEXIBILITY
print("\n5. Multi-provider support...")
local providers = {
    {provider = "openai", model = "gpt-3.5-turbo"},
    {provider = "anthropic", model = "claude-3-5-haiku-latest"}
}

for _, config in ipairs(providers) do
    local success = pcall(function()
        return Agent.builder()
            :name(config.provider .. "-test")
            :model(config.provider .. "/" .. config.model)
            :build()
    end)
    print("   " .. config.provider .. ": " .. (success and "✓ available" or "✗ not configured"))
end

-- 6. STREAMING RESPONSES (mention only)
print("\n6. Advanced features:")
print("   • Streaming responses - See invokeStream() method")
print("   • Agent composition - See advanced-patterns/")
print("   • Tool integration - See agent:execute() with tools in builder")

print("\n=== Agent Basics Complete ===")
print("Next: Explore tool-basics.lua for tool integration")