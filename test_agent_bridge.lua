-- Test script for agent bridge integration
print("Testing agent bridge creation...")

-- Create an agent using the bridge pattern
local agent_config = {
    name = "test-bridge-agent",
    description = "Test agent via bridge",
    agent_type = "llm",
    model = {
        provider = "openai",
        model_id = "gpt-4o-mini",
        temperature = 0.7,
        max_tokens = 100
    },
    allowed_tools = {},
    custom_config = {
        system_prompt = "You are a helpful assistant. Keep responses concise."
    },
    resource_limits = {
        max_execution_time_secs = 60,
        max_memory_mb = 512,
        max_tool_calls = 10,
        max_recursion_depth = 5
    }
}

-- Convert to JSON for bridge API
local json_config = JSON.encode(agent_config)
print("Agent config: " .. json_config)

print("Test completed - agent configuration created!")