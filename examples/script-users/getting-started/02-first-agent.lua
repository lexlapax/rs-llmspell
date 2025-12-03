-- ============================================================
-- LLMSPELL GETTING STARTED SHOWCASE
-- ============================================================
-- Phase: 13c.5.6 - Example Header Standardization
-- Category: getting-started
-- Profile: providers (recommended)
-- Example ID: 02 - First Agent v0.14.0
-- Complexity: BEGINNER
-- Real-World Use Case: Creating AI assistants for automation and support
--
-- Purpose: Learn how to create and interact with LLM agents. Demonstrates
--          agent builder pattern, configuration options, and basic prompting.
--          This is your gateway to AI-powered automation in LLMSpell.
--
-- Architecture: Agent builder pattern with provider abstraction
-- Crates Showcased: llmspell-agents, llmspell-bridge
--
-- Key Features:
--   • Agent builder pattern
--   • Provider detection and selection
--   • System prompt configuration
--   • Synchronous agent invocation
--   • Response handling
--
-- Prerequisites:
--   • LLMSpell installed and built
--   • Environment: OPENAI_API_KEY or ANTHROPIC_API_KEY
--   • Network connectivity for API calls
--
-- HOW TO RUN:
-- ./target/debug/llmspell -p providers \
--   run examples/script-users/getting-started/02-first-agent.lua
--
-- EXPECTED OUTPUT:
-- Available providers: openai, anthropic (or configured providers)
-- Agent created successfully
-- Agent response: "2 + 2 equals 4"
-- Agent info: provider and model details
--
-- Runtime: ~10 seconds
-- ============================================================

print("=== LLMSpell: Your First Agent ===")
print("Example 02: BEGINNER - Creating and using AI agents")
print("Showcasing: Agent builder pattern and basic interaction\n")

-- First, let's check what providers are available
print("1. Checking available providers...")
local providers = Provider.list()
-- Provider.list() returns array of {name, enabled, capabilities} tables
local provider_names = {}
for i, p in ipairs(providers) do
    provider_names[i] = p.name
end
print("   Available providers: " .. table.concat(provider_names, ", "))

if #providers == 0 then
    print("❌ No providers configured. Please check your configuration.")
    print("   See examples/script-users/configs/ for configuration examples.")
    return
end

print()
print("2. Creating your first agent...")

-- Create a simple agent using the first available provider
-- Note: Use colon notation (:method) for builder pattern, not dot notation (.method)
-- build() returns the agent directly or throws an error, so use pcall for safety
-- Provider names are formatted as "rig/provider/model" - extract provider and model
local provider_full = provider_names[1]
local parts = {}
for part in string.gmatch(provider_full, "[^/]+") do
    table.insert(parts, part)
end
-- parts[1] = "rig", parts[2] = "openai"/"anthropic"/etc, parts[3] = model name
local provider_name = parts[2] or "openai"
local model_name = parts[3] or "gpt-4"
print("   Using provider: " .. provider_name .. ", model: " .. model_name)

local success, agent = pcall(function()
    return Agent.builder()
        :provider(provider_name)
        :model(model_name)
        :system_prompt("You are a helpful assistant who gives brief, friendly responses.")
        :build()
end)

if not success then
    print("❌ Error creating agent: " .. tostring(agent))
    return
end

print("✅ Agent created successfully with provider: " .. provider_name)

print()
print("3. Having a conversation...")

-- Send a simple message
-- execute() expects a table with input parameters
local exec_success, response = pcall(function()
    return agent:execute({ text = "Hello! Can you tell me what 2 + 2 equals?" })
end)

if not exec_success then
    print("❌ Error getting response: " .. tostring(response))
    return
end

-- execute() returns {text = "...", metadata = {...}} for plain text responses
print("🤖 Agent response:")
print("   " .. (response.text or "No response text"))

print()
print("4. Getting agent information...")
-- Note: get_info is on Agent global, not on instance. Use get_config for instance info.
local config = agent:get_config()
print("📊 Agent configuration:")
print("   Provider: " .. provider_name)
print("   Model: " .. model_name)
-- Agent metrics available via get_metrics()
local metrics = agent:get_metrics()
if metrics then
    print("   Requests: " .. (metrics.requests_total or 0))
end

print()
print("🎉 Congratulations! You've successfully:")
print("   - Listed available providers")
print("   - Created your first agent")
print("   - Had a conversation with the agent")
print("   - Retrieved agent information")
print()
print("Next: Try 03-first-workflow.lua to learn about workflows!")