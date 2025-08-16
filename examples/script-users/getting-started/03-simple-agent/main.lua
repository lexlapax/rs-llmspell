-- Example: Your First AI Agent
-- Purpose: Learn how to create and interact with an AI agent
-- Audience: Script Users (Beginners)
-- Prerequisites: Completed 02-first-tool, API key set (OPENAI_API_KEY or ANTHROPIC_API_KEY)
-- Expected Output: Agent conversation demonstration
-- Version: 0.7.0
-- Tags: getting-started, agents, ai, conversation, beginner

print("=== Your First AI Agent ===")
print("")

-- Agents are AI assistants that can understand and respond to your requests.
-- They can also use tools to perform actions.

-- First, check if we have providers configured
print("1. Checking available AI providers...")
local providers = Provider.list()
if #providers == 0 then
    print("   ⚠️  No providers configured.")
    print("   To use agents, set an API key in your environment:")
    print("   export OPENAI_API_KEY='your-key-here'")
    print("   or")
    print("   export ANTHROPIC_API_KEY='your-key-here'")
    print("")
    print("   For this demo, we'll create a mock agent.")
    print("")
end

-- Step 2: Create an agent using the builder pattern
print("2. Creating your AI assistant...")

local agent
local success, err = pcall(function()
    agent = Agent.builder()
        :name("my_first_assistant")
        :model("openai/gpt-3.5-turbo")  -- or "anthropic/claude-3-haiku-20240307"
        :system_prompt("You are a helpful and friendly AI assistant. Keep responses concise.")
        :temperature(0.7)  -- 0.0 = focused, 1.0 = creative
        :max_tokens(150)   -- Maximum response length
        :build()
end)

if success and agent then
    print("   ✅ Agent created successfully!")
else
    print("   ❌ Failed to create agent: " .. tostring(err))
    print("   Make sure you have an API key configured.")
    return
end

print("")

-- Step 3: Have a conversation
print("3. Starting conversation...")
print("")

-- Example 1: Simple question
print("You: What's the capital of France?")
local response = agent:invoke({
    text = "What's the capital of France?"
})

if response and response.text then
    print("Assistant: " .. response.text)
else
    print("Assistant: [No response]")
end

print("")

-- Example 2: Math question
print("You: What's 42 multiplied by 17?")
response = agent:invoke({
    text = "What's 42 multiplied by 17?"
})

if response and response.text then
    print("Assistant: " .. response.text)
else
    print("Assistant: [No response]")
end

print("")

-- Example 3: Creative request
print("You: Write a haiku about programming")
response = agent:invoke({
    text = "Write a haiku about programming"
})

if response and response.text then
    print("Assistant: " .. response.text)
else
    print("Assistant: [No response]")
end

print("")

-- Step 4: Create a specialized agent
print("4. Creating a specialized coding assistant...")

local code_agent = Agent.builder()
    :name("code_helper")
    :model("openai/gpt-3.5-turbo")
    :system_prompt([[
You are a programming assistant specializing in Lua.
Provide clear, concise code examples.
Always explain your code.
    ]])
    :temperature(0.3)  -- Lower temperature for more focused responses
    :max_tokens(200)
    :build()

print("   ✅ Code assistant created!")
print("")

print("You: How do I reverse a string in Lua?")
response = code_agent:invoke({
    text = "How do I reverse a string in Lua?"
})

if response and response.text then
    print("Code Assistant: " .. response.text)
else
    print("Code Assistant: [No response]")
end

print("")
print("🎉 Congratulations! You've successfully:")
print("   - Created AI agents using Agent.builder()")
print("   - Configured agents with prompts and parameters")
print("   - Had conversations with your agents")
print("   - Created specialized agents for specific tasks")
print("")
print("💡 Key Concepts:")
print("   - Agents are created with Agent.builder() pattern")
print("   - system_prompt defines the agent's personality/role")
print("   - temperature controls creativity (0=focused, 1=creative)")
print("   - max_tokens limits response length")
print("")
print("Next: Continue to '04-basic-workflow' to chain multiple operations!")