-- Example: 02-first-agent.lua
-- Author: LLMSpell Examples  
-- Purpose: First introduction to creating and using an agent
-- Learning: Basic agent creation and conversation

print("=== LLMSpell: Your First Agent ===")
print("This example shows how to create and talk to an AI agent!")
print()

-- First, let's check what providers are available
print("1. Checking available providers...")
local providers = Provider.list()
print("   Available providers: " .. table.concat(providers, ", "))

if #providers == 0 then
    print("❌ No providers configured. Please check your configuration.")
    print("   See examples/script-users/configs/ for configuration examples.")
    return
end

print()
print("2. Creating your first agent...")

-- Create a simple agent using the first available provider
local agent_result = Agent.builder()
    .provider(providers[1])
    .system_prompt("You are a helpful assistant who gives brief, friendly responses.")
    .build()

if not agent_result.success then
    print("❌ Error creating agent: " .. (agent_result.error or "Unknown error"))
    return
end

local agent = agent_result.result
print("✅ Agent created successfully with provider: " .. providers[1])

print()
print("3. Having a conversation...")

-- Send a simple message
local response = agent:invoke("Hello! Can you tell me what 2 + 2 equals?")

if response.success then
    print("🤖 Agent response:")
    print("   " .. response.result.content)
else
    print("❌ Error getting response: " .. (response.error or "Unknown error"))
    return
end

print()
print("4. Getting agent information...")
local info = agent:get_info()
if info.success then
    print("📊 Agent info:")
    print("   Provider: " .. (info.result.provider or "Unknown"))
    print("   Model: " .. (info.result.model or "Unknown"))
else
    print("❌ Error getting agent info: " .. (info.error or "Unknown error"))
end

print()
print("🎉 Congratulations! You've successfully:")
print("   - Listed available providers")
print("   - Created your first agent")
print("   - Had a conversation with the agent")
print("   - Retrieved agent information")
print()
print("Next: Try 03-first-workflow.lua to learn about workflows!")