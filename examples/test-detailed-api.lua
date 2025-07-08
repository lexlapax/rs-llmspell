-- test-detailed-api.lua
-- Test detailed API methods available

print("🔍 Detailed API Testing")
print("=======================")

print("\n1. Agent API Methods:")
if Agent then
    for k, v in pairs(Agent) do
        print(string.format("  Agent.%s: %s", k, type(v)))
    end
else
    print("  Agent not available")
end

print("\n2. Tool API Methods:")
if Tool then
    for k, v in pairs(Tool) do
        print(string.format("  Tool.%s: %s", k, type(v)))
    end
else
    print("  Tool not available")
end

print("\n3. Workflow API Methods:")
if Workflow then
    for k, v in pairs(Workflow) do
        print(string.format("  Workflow.%s: %s", k, type(v)))
    end
else
    print("  Workflow not available")
end

print("\n4. Streaming API Methods:")
if Streaming then
    for k, v in pairs(Streaming) do
        print(string.format("  Streaming.%s: %s", k, type(v)))
    end
else
    print("  Streaming not available")
end

-- Test creating an agent with a simple configuration
print("\n5. Testing Agent Creation:")
local success, agent_or_error = pcall(function()
    return Agent.create("test-model")
end)

if success then
    print("✅ Agent creation successful")
    local agent = agent_or_error
    print("Agent type:", type(agent))
    
    -- Test agent methods
    if agent then
        print("Agent methods:")
        for k, v in pairs(agent) do
            print(string.format("  agent.%s: %s", k, type(v)))
        end
    end
else
    print("❌ Agent creation failed:", agent_or_error)
end

-- Test Tool API
print("\n6. Testing Tool API:")
local tool_success, tool_result = pcall(function()
    return Tool.list()
end)

if tool_success then
    print("✅ Tool.list() successful")
    print("Available tools:", #tool_result)
    for i, tool in ipairs(tool_result) do
        print(string.format("  %d. %s", i, tool.name or tool))
    end
else
    print("❌ Tool.list() failed:", tool_result)
end

local function get_keys(t)
    local keys = {}
    for k, _ in pairs(t or {}) do
        table.insert(keys, k)
    end
    return keys
end

return {
    agent_methods = Agent and table.concat(get_keys(Agent), ", ") or "none",
    tool_methods = Tool and table.concat(get_keys(Tool), ", ") or "none",
    status = "detailed_api_test_complete"
}