-- ABOUTME: Minimal test to debug Agent.createAsync timeout issue
-- ABOUTME: Tests the most basic agent creation possible

print("=== Minimal Agent Test ===")
print("")

-- Test with minimal configuration
print("Creating agent with minimal config...")
local agent = Agent.createAsync({
    model = "gpt-4o-mini",
    prompt = "Hello"
})

print("Agent created successfully!")
print("Type: " .. type(agent))

print("\n=== Test complete ===")