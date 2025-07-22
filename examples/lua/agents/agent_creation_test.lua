-- Example Lua script demonstrating the new model specification syntax
-- ABOUTME: Test script for Phase 2.1.3 - ModelSpecifier integration

print("Testing new agent creation syntax...")

-- Test 1: New syntax with provider/model
print("Test 1: Creating agent with 'openai/gpt-4' syntax...")
local success1, agent1 = pcall(function()
    return Agent.create({
        model = "openai/gpt-4",
        system_prompt = "You are a helpful assistant.",
        temperature = 0.7,
        max_tokens = 100
    })
end)
if success1 then
    print("  ✓ Agent 1 created successfully")
else
    print("  ✗ Agent 1 failed: " .. tostring(agent1))
end

-- Test 2: New syntax with base URL override
print("Test 2: Creating agent with base URL override...")
local success2, agent2 = pcall(function()
    return Agent.create({
        model = "anthropic/claude-3",
        base_url = "https://api.custom.com/v1",
        system_prompt = "You are a coding assistant.",
        temperature = 0.5
    })
end)
if success2 then
    print("  ✓ Agent 2 created successfully")
else
    print("  ✗ Agent 2 failed: " .. tostring(agent2))
end

-- Test 3: Legacy syntax compatibility
print("Test 3: Testing legacy syntax compatibility...")
local success3, agent3 = pcall(function()
    return Agent.create({
        provider = "openai",
        model_name = "gpt-3.5-turbo",
        system_prompt = "You are a creative writer.",
        temperature = 0.9
    })
end)
if success3 then
    print("  ✓ Agent 3 created successfully")
else
    print("  ✗ Agent 3 failed: " .. tostring(agent3))
end

-- Test 4: Model only (should use default provider)
print("Test 4: Testing model-only syntax...")
local success4, agent4 = pcall(function()
    return Agent.create({
        model = "gpt-4",
        system_prompt = "You are a data analyst."
    })
end)
if success4 then
    print("  ✓ Agent 4 created successfully")
else
    print("  ✗ Agent 4 failed: " .. tostring(agent4))
end

print("All agent creation tests completed!")
print("Note: These tests verify API compatibility, actual LLM calls require valid API keys.")