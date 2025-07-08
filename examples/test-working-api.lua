-- test-working-api.lua
-- Test the working API with proper syntax

print("🔍 Working API Test")
print("===================")

-- Test Tool API first
print("\n1. Testing Tool API:")
local tools = Tool.list()
print("Available tools:", #tools)
for i, tool in ipairs(tools) do
    print(string.format("  %d. %s", i, tool))
end

-- Test getting specific tool
if #tools > 0 then
    local first_tool = tools[1]
    print(string.format("\n2. Getting tool details for: %s", first_tool))
    
    local tool_details = Tool.get(first_tool)
    print("Tool details type:", type(tool_details))
    if tool_details then
        print("Tool details:")
        for k, v in pairs(tool_details) do
            print(string.format("  %s: %s", k, type(v)))
        end
    end
end

-- Test Agent creation with proper table syntax
print("\n3. Testing Agent Creation with Table:")
local agent_success, agent_or_error = pcall(function()
    return Agent.create({
        model = "test-model",
        temperature = 0.7
    })
end)

if agent_success then
    print("✅ Agent creation successful")
    local agent = agent_or_error
    print("Agent type:", type(agent))
    
    if agent then
        print("Agent methods:")
        for k, v in pairs(agent) do
            print(string.format("  agent.%s: %s", k, type(v)))
        end
    end
else
    print("❌ Agent creation failed:", agent_or_error)
end

-- Test using a tool directly
print("\n4. Testing Tool Usage:")
if #tools > 0 then
    local calculator_available = false
    for _, tool in ipairs(tools) do
        if tool == "calculator" then
            calculator_available = true
            break
        end
    end
    
    if calculator_available then
        print("✅ Calculator tool is available")
        -- Try to use the calculator tool
        local calc_success, calc_result = pcall(function()
            local tool_instance = Tool.get("calculator")
            if tool_instance and tool_instance.execute then
                return tool_instance.execute({
                    expression = "2 + 3 * 4"
                })
            end
            return "No execute method"
        end)
        
        if calc_success then
            print("Calculator result:", calc_result)
        else
            print("Calculator failed:", calc_result)
        end
    else
        print("❌ Calculator tool not available")
    end
end

print("\n✅ Working API test complete!")

return {
    available_tools = tools,
    agent_creation_works = agent_success,
    status = "working_api_test_complete"
}