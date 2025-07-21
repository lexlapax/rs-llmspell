-- Simulate the exact test scenario from provider_enhancement_test.rs
print("=== Testing Provider Enhancement Scenario ===\n")

-- Test 1: Direct createAsync (like test_agent_create_with_provider_model_syntax)
print("1. Direct Agent.createAsync calls...")
local agent1 = Agent.createAsync({
    model = "openai/gpt-4",
    prompt = "You are a test assistant"
})
print("   Created agent1: " .. type(agent1))

local agent2 = Agent.createAsync({
    model = "anthropic/claude-3-opus",
    prompt = "You are another test assistant"
})
print("   Created agent2: " .. type(agent2))

print("\n2. Testing with pcall (like test_base_url_override)...")
local success, err = pcall(function()
    return Agent.createAsync({
        model = "openai/gpt-3.5-turbo",
        base_url = "http://localhost:8080/v1",
        prompt = "You are a test assistant"
    })
end)

print("   Success: " .. tostring(success))
print("   Error: " .. tostring(err))

-- The test expects this to fail
assert(not success, "Should fail with unconfigured provider")
assert(err, "Should have error message")

-- Check error message
local error_str = tostring(err)
assert(
    error_str:find("provider") or error_str:find("Unknown"),
    "Error should be about provider configuration: " .. error_str
)

print("\n=== Test Complete ===")