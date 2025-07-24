-- Example: Using Agent.create for synchronous agent creation
-- This example demonstrates how to create and use agents in Lua scripts

print("=== Agent Synchronous Example ===\n")

-- 1. Create an agent using Agent.create
print("1. Creating a conversational agent...")
local success, agent = pcall(function()
    return Agent.create({
        name = "assistant",
        model = "openai/gpt-3.5-turbo",  -- Format: provider/model
        system_prompt = "You are a helpful AI assistant. Be concise and friendly.",
        temperature = 0.7,
        max_tokens = 150
    })
end)
if success and agent then
    print("   ✓ Agent created: " .. type(agent))
else
    print("   ✗ Failed to create agent: " .. tostring(agent))
    return
end

-- 2. Execute a simple query
print("\n2. Asking a simple question...")
local success2, response = pcall(function()
    return agent:invoke({
        text = "What are the three primary colors?"
    })
end)
if success2 and response then
    print("   Response: " .. (response.text or response.output or "No response"))
else
    print("   ✗ Failed to invoke agent: " .. tostring(response))
end

-- 3. Create a specialized agent
print("\n3. Creating a code expert agent...")
local success3, codeAgent = pcall(function()
    return Agent.create({
        name = "code-expert",
        model = "openai/gpt-3.5-turbo",
        system_prompt = "You are a programming expert. Provide code examples when appropriate. Be concise.",
        temperature = 0.3,  -- Lower temperature for more focused responses
        max_tokens = 200
    })
end)

if success3 and codeAgent then
    -- 4. Ask a programming question
    print("\n4. Asking a programming question...")
    local success4, codeResponse = pcall(function()
        return codeAgent:invoke({
            text = "Write a Python function to reverse a string"
        })
    end)
    if success4 and codeResponse then
        print("   Response:\n" .. (codeResponse.text or codeResponse.output or "No response"))
    else
        print("   ✗ Failed to invoke code agent: " .. tostring(codeResponse))
    end
else
    print("   ✗ Failed to create code agent: " .. tostring(codeAgent))
end

-- 5. Working with different providers (if configured)
print("\n5. Creating agents with different providers...")

-- Example with Anthropic (requires ANTHROPIC_API_KEY)
local providers = {
    {provider = "anthropic", model = "claude-3-5-haiku-latest"},
    {provider = "openai", model = "gpt-4"},
}

for i, config in ipairs(providers) do
    local success, agent = pcall(function()
        return Agent.create({
            name = config.provider .. "-agent-" .. i,
            model = config.provider .. "/" .. config.model,
            system_prompt = "You are a helpful assistant.",
            temperature = 0.7
        })
    end)
    
    if success and agent then
        print("   ✓ Created " .. config.provider .. " agent")
    else
        print("   ✗ Failed to create " .. config.provider .. " agent: " .. tostring(agent))
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
-- 1. Agent.create() is now synchronous - no coroutines needed
-- 2. All agent methods like invoke() are synchronous
-- 3. Set appropriate API keys in environment variables:
--    - OPENAI_API_KEY for OpenAI
--    - ANTHROPIC_API_KEY for Anthropic
--    - COHERE_API_KEY for Cohere