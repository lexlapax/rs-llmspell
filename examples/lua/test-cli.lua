-- ABOUTME: Simple test script to verify CLI functionality
-- ABOUTME: Tests basic features without requiring LLM API calls

print("=== CLI Test Script ===\n")

-- Test 1: Tool is available
print("Test 1: Checking tool availability")
-- Use Tool.executeAsync as shown in tools-showcase.lua
local result = Tool.executeAsync("calculator", { input = "10 + 20" })
if result and result.output then
    print("✓ Tool.executeAsync() works")
    print("✓ Calculator result: " .. tostring(result.output))
else
    print("✗ Tool execution failed")
end

-- Test 2: JSON global
print("\nTest 2: JSON global")
local data = { name = "test", values = {1, 2, 3} }
local json_str = JSON.stringify(data)
print("✓ JSON.stringify works: " .. json_str)

local parsed = JSON.parse(json_str)
if parsed and parsed.name == "test" then
    print("✓ JSON.parse works")
else
    print("✗ JSON.parse failed")
end

-- Test 3: Workflow creation (without execution)
print("\nTest 3: Workflow API")
local workflow = Workflow.sequential({
    name = "test_workflow",
    steps = {
        {
            name = "step1",
            type = "tool",
            tool = "uuid_generator",
            input = { version = "v4" }
        }
    }
})

if workflow then
    print("✓ Workflow.sequential() creates workflow object")
    
    -- Check if it has execute method
    if type(workflow.execute) == "function" then
        print("✓ Workflow has execute method")
    else
        print("✗ Workflow missing execute method")
    end
else
    print("✗ Workflow.sequential() failed")
end

-- Test 4: Agent API (without actual LLM calls)
print("\nTest 4: Agent API")
-- Note: Agent creation might fail without proper provider config
local success, agent_or_err = pcall(function()
    return Agent.createAsync({
        name = "test_agent",
        model = "gpt-3.5-turbo",
        system_prompt = "Test agent"
    })
end)

if success and agent_or_err then
    print("✓ Agent.createAsync() works")
else
    print("✗ Agent.createAsync() failed (likely needs provider config)")
    print("  Error: " .. tostring(agent_or_err))
end

-- Test 5: Tool.list()
print("\nTest 5: Tool listing")
local tools = Tool.list()
if tools and #tools > 0 then
    print("✓ Tool.list() works - found " .. #tools .. " tools")
    print("  First 5 tools: " .. table.concat({tools[1], tools[2], tools[3], tools[4], tools[5]}, ", "))
else
    print("✗ Tool.list() failed or no tools found")
end

-- Test 6: Workflow types
print("\nTest 6: Workflow types")
if Workflow.discover_types then
    local types = Workflow.discover_types()
    if types and #types > 0 then
        print("✓ Workflow.discover_types() works")
        print("  Available types: " .. table.concat(types, ", "))
    else
        print("✗ No workflow types found")
    end
else
    print("✗ Workflow.discover_types() not available")
end

print("\n=== CLI Test Complete ===")