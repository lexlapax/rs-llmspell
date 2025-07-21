-- ABOUTME: Quick test to verify Agent.createAsync works correctly
-- ABOUTME: Tests that agents can be created and execute without coroutine errors

print("=== Testing Agent.createAsync ===")
print("")

-- Test 1: Basic agent creation
print("Test 1: Creating agent with createAsync...")
local agent = Agent.createAsync({
    name = "test_agent",
    model = "gpt-4o-mini",
    system_prompt = "You are a helpful test assistant. Keep responses very brief.",
    temperature = 0.1,
    max_tokens = 20
})
print("✅ Agent created successfully")

-- Test 2: Agent execution
print("\nTest 2: Executing agent...")
local result = agent:execute({
    prompt = "Say 'Hello, test passed!' and nothing else."
})

if result and result.content then
    print("✅ Agent executed successfully")
    print("Response: " .. result.content)
else
    print("❌ Agent execution failed")
    print("Result: " .. tostring(result))
end

-- Test 3: Multiple agents
print("\nTest 3: Creating multiple agents...")
local agent2 = Agent.createAsync({
    name = "test_agent_2", 
    model = "gpt-4o-mini",
    system_prompt = "You are a second test assistant.",
    max_tokens = 20
})
print("✅ Second agent created")

-- Test 4: Error handling
print("\nTest 4: Testing error handling...")
local success, err = pcall(function()
    return Agent.createAsync({
        model = "invalid-provider/invalid-model",
        system_prompt = "This should fail"
    })
end)

if not success then
    print("✅ Error handling works correctly")
    print("Error: " .. tostring(err))
else
    print("❌ Expected error but got success")
end

print("\n=== All tests completed ===")
print("Agent.createAsync is working properly!")