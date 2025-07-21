print("=== Testing Agent.createAsync ===\n")

-- Test creating an LLM agent with createAsync
print("1. Creating LLM agent with createAsync...")
local status, agent = pcall(function()
    return Agent.createAsync({
        name = "test-llm",
        model = "openai/gpt-3.5-turbo",
        system_prompt = "You are a helpful assistant.",
        temperature = 0.7,
        max_tokens = 100
    })
end)

if status then
    print("   ✓ Agent created successfully")
    print("   Type: " .. type(agent))
else
    print("   ✗ Agent creation failed: " .. tostring(agent))
end

print("\n=== Test Complete ===")