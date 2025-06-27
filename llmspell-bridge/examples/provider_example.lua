-- Example Lua script showing provider integration

-- Create an agent with default provider
local agent = Agent.create({
    system_prompt = "You are a helpful AI assistant",
    temperature = 0.7,
    max_tokens = 100
})

-- Execute a simple request
local result = agent:execute({
    text = "Hello, can you help me?"
})

print("Agent response: " .. result.text)

-- Create an agent with specific provider
local custom_agent = Agent.create({
    provider = "openai",
    model = "gpt-4",
    system_prompt = "You are an expert programmer",
    temperature = 0.5
})

-- Execute with the custom agent
local code_result = custom_agent:execute({
    text = "Write a hello world in Python"
})

print("Code response: " .. code_result.text)