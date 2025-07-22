-- Test script for LLM agent integration
print("Testing LLM agent creation...")

-- Create an LLM agent
local agent = Agent.createAsync({
    name = "test-llm-agent",
    description = "Test LLM agent",
    model = "gpt-4o-mini",
    temperature = 0.7,
    max_tokens = 100,
    system_prompt = "You are a helpful assistant. Keep responses concise."
})

print("Agent created successfully!")

-- Test agent invocation
local response = agent:invoke({
    input = "Hello! What is 2+2?"
})

print("Agent response:")
print(response.output)

-- Clean up
agent:destroy()
print("Test completed!")