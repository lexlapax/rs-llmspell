-- Example: Using Agent.createAsync for coroutine-safe agent creation
-- This example demonstrates how to create and use agents in Lua scripts

-- Load agent helpers
local helpers = dofile("agent-helpers.lua")

print("=== Agent Async Example ===\n")

-- 1. Create an agent using Agent.createAsync
print("1. Creating a conversational agent...")
local agent, err = helpers.createAgent({
    name = "assistant",
    model = "openai/gpt-3.5-turbo",  -- Format: provider/model
    system_prompt = "You are a helpful AI assistant. Be concise and friendly.",
    temperature = 0.7,
    max_tokens = 150
})
if agent then
    print("   ✓ Agent created: " .. type(agent))
else
    print("   ✗ Failed to create agent: " .. tostring(err))
    return
end

-- 2. Execute a simple query
print("\n2. Asking a simple question...")
local response, err = helpers.invokeAgent(agent, {
    text = "What are the three primary colors?"
})
if response then
    print("   Response: " .. (response.text or response.output or "No response"))
else
    print("   ✗ Failed to invoke agent: " .. tostring(err))
end

-- 3. Create a specialized agent
print("\n3. Creating a code expert agent...")
local codeAgent, err = helpers.createAgent({
    name = "code-expert",
    model = "openai/gpt-3.5-turbo",
    system_prompt = "You are a programming expert. Provide code examples when appropriate. Be concise.",
    temperature = 0.3,  -- Lower temperature for more focused responses
    max_tokens = 200
})

if codeAgent then
    -- 4. Ask a programming question
    print("\n4. Asking a programming question...")
    local codeResponse, err = helpers.invokeAgent(codeAgent, {
        text = "Write a Python function to reverse a string"
    })
    if codeResponse then
        print("   Response:\n" .. (codeResponse.text or codeResponse.output or "No response"))
    else
        print("   ✗ Failed to invoke code agent: " .. tostring(err))
    end
else
    print("   ✗ Failed to create code agent: " .. tostring(err))
end

-- 5. Working with different providers (if configured)
print("\n5. Creating agents with different providers...")

-- Example with Anthropic (requires ANTHROPIC_API_KEY)
local providers = {
    {provider = "anthropic", model = "claude-3-sonnet-20240229"},
    {provider = "openai", model = "gpt-4"},
    {provider = "cohere", model = "command-r"}
}

for _, config in ipairs(providers) do
    local agent, err = helpers.createAgent({
        name = config.provider .. "-agent",
        model = config.provider .. "/" .. config.model,
        system_prompt = "You are a helpful assistant.",
        temperature = 0.7
    })
    
    if agent then
        print("   ✓ Created " .. config.provider .. " agent")
    else
        print("   ✗ Failed to create " .. config.provider .. " agent: " .. tostring(err))
    end
end

-- 6. List all created agents
print("\n6. Listing all agents...")
local agents = Agent.list()
print("   Found " .. #agents .. " agents:")
for i, agentInfo in ipairs(agents) do
    print("   - " .. agentInfo.name)
end

print("\n=== Example Complete ===")

-- Important notes:
-- 1. Always use Agent.createAsync() instead of Agent.createAsync() to avoid coroutine errors
-- 2. Wrap async method calls (like execute) in the asyncCall helper
-- 3. Set appropriate API keys in environment variables:
--    - OPENAI_API_KEY for OpenAI
--    - ANTHROPIC_API_KEY for Anthropic
--    - COHERE_API_KEY for Cohere