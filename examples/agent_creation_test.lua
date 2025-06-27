-- Example Lua script demonstrating the new model specification syntax
-- ABOUTME: Test script for Phase 2.1.3 - ModelSpecifier integration

print("Testing new agent creation syntax...")

-- Test 1: New syntax with provider/model
print("Test 1: Creating agent with 'openai/gpt-4' syntax...")
local agent1 = Agent.create({
    model = "openai/gpt-4",
    system_prompt = "You are a helpful assistant.",
    temperature = 0.7,
    max_tokens = 100
})

-- Test 2: New syntax with base URL override
print("Test 2: Creating agent with base URL override...")
local agent2 = Agent.create({
    model = "anthropic/claude-3",
    base_url = "https://api.custom.com/v1",
    system_prompt = "You are a coding assistant.",
    temperature = 0.5
})

-- Test 3: Legacy syntax compatibility
print("Test 3: Testing legacy syntax compatibility...")
local agent3 = Agent.create({
    provider = "openai",
    model_name = "gpt-3.5-turbo",
    system_prompt = "You are a creative writer.",
    temperature = 0.9
})

-- Test 4: Model only (should use default provider)
print("Test 4: Testing model-only syntax...")
local agent4 = Agent.create({
    model = "gpt-4",
    system_prompt = "You are a data analyst."
})

print("All agent creation tests completed!")
print("Note: These tests verify API compatibility, actual LLM calls require valid API keys.")