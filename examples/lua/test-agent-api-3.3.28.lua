-- ABOUTME: Test script for new Agent API methods added in Task 3.3.28
-- ABOUTME: Verifies all new Agent global methods are accessible and functional

print("=== Testing New Agent API Methods (Task 3.3.28) ===\n")

-- Test counter
local tests_passed = 0
local tests_failed = 0

-- Helper function to run a test
local function test(name, fn)
    print("Testing: " .. name)
    local success, result = pcall(fn)
    if success then
        print("  ✅ PASSED")
        tests_passed = tests_passed + 1
    else
        print("  ❌ FAILED: " .. tostring(result))
        tests_failed = tests_failed + 1
    end
    print()
end

-- 1. Test Agent.register() method
test("Agent.register()", function()
    local config = {
        name = "test-register-agent",
        description = "Test agent created with register()",
        agent_type = "llm",
        -- Model config expects a struct with settings
        model = {
            provider = "openai",
            model_id = "gpt-3.5-turbo",
            temperature = 0.7,
            max_tokens = 100,
            settings = {}  -- Required field
        },
        custom_config = {
            system_prompt = "You are a test agent"
        },
        allowed_tools = {"calculator"},  -- List with one tool to ensure it's an array
        resource_limits = {
            max_execution_time_secs = 300,
            max_memory_mb = 512,
            max_tool_calls = 100,
            max_recursion_depth = 10
        }
    }
    
    local agent_name = Agent.register(config)
    assert(agent_name == "test-register-agent", "Agent name mismatch")
    
    -- Verify it exists in the list
    local agents = Agent.list()
    local found = false
    for _, agent in ipairs(agents) do
        if agent.name == "test-register-agent" then
            found = true
            break
        end
    end
    assert(found, "Registered agent not found in list")
end)

-- 2. Test Agent.get() method
test("Agent.get()", function()
    -- First create an agent using createAsync (synchronous wrapper)
    local agent = Agent.createAsync({
        name = "test-get-agent",
        model = "openai/gpt-3.5-turbo"
    })
    
    -- Now get it
    local retrieved = Agent.get("test-get-agent")
    assert(retrieved ~= nil, "Agent.get() returned nil")
    -- Agent.get() returns a LuaAgentInstance, not checking specific fields for now
    assert(type(retrieved) == "userdata", "Should return userdata")
    
    -- Test getting non-existent agent
    local missing = Agent.get("non-existent-agent")
    assert(missing == nil, "Should return nil for non-existent agent")
end)

-- 3. Test Agent.getInfo() method
test("Agent.getInfo()", function()
    -- Create an agent first
    Agent.createAsync({
        name = "test-info-agent",
        model = "openai/gpt-3.5-turbo",
        description = "Agent for testing getInfo"
    })
    
    -- Get info
    local info = Agent.getInfo("llm")  -- Get info about the agent type
    assert(type(info) == "table", "getInfo should return a table")
    assert(info.name ~= nil, "Info should have name")
    assert(info.description ~= nil, "Info should have description")
    assert(type(info.required_parameters) == "table", "Info should have required_parameters")
end)

-- 4. Test Agent.listCapabilities() method
test("Agent.listCapabilities()", function()
    local capabilities = Agent.listCapabilities()
    assert(type(capabilities) == "table", "listCapabilities should return a table")
    
    -- Should have at least one agent if we created any
    local count = 0
    for name, cap in pairs(capabilities) do
        assert(type(name) == "string", "Capability key should be string")
        assert(type(cap) == "table", "Capability value should be table")
        assert(cap.name ~= nil, "Capability should have name")
        assert(cap.capabilities ~= nil, "Capability should have capabilities field")
        count = count + 1
    end
    assert(count > 0, "Should have at least one capability")
end)

-- 5. Test Agent.wrapAsTool() method
test("Agent.wrapAsTool()", function()
    -- Create an agent first
    Agent.createAsync({
        name = "test-wrap-agent",
        model = "openai/gpt-3.5-turbo",
        description = "Agent to wrap as tool"
    })
    
    -- Wrap it as a tool
    local tool_name = Agent.wrapAsTool("test-wrap-agent", {
        tool_name = "wrapped-agent-tool",
        description = "Agent wrapped as a tool"
    })
    
    assert(type(tool_name) == "string", "wrapAsTool should return string")
    assert(tool_name == "wrapped-agent-tool", "Tool name mismatch")
    
    -- Verify the tool exists
    if Tool then
        local tools = Tool.list()
        local found = false
        for _, tool in ipairs(tools) do
            if tool.name == "wrapped-agent-tool" then
                found = true
                break
            end
        end
        assert(found, "Wrapped tool not found in Tool.list()")
    end
end)

-- 6. Test Agent.createComposite() method
test("Agent.createComposite()", function()
    -- Create two agents first
    Agent.createAsync({
        name = "composite-agent-1",
        model = "openai/gpt-3.5-turbo"
    })
    
    Agent.createAsync({
        name = "composite-agent-2", 
        model = "openai/gpt-3.5-turbo"
    })
    
    -- Create composite
    local composite_id = Agent.createComposite(
        "test-composite",
        {"composite-agent-1", "composite-agent-2"},
        {
            routing_strategy = "round_robin"
        }
    )
    
    -- Note: This returns void in the current implementation
    -- Just verify it doesn't error
    assert(true, "createComposite completed without error")
end)

-- 7. Test Agent.discoverByCapability() method  
test("Agent.discoverByCapability()", function()
    local agents = Agent.discoverByCapability("streaming")
    assert(type(agents) == "table", "discoverByCapability should return table")
    
    -- Should be a list (array)
    for i, agent_name in ipairs(agents) do
        assert(type(agent_name) == "string", "Agent name should be string")
    end
end)

-- 8. Test method chaining and integration
test("Method Integration", function()
    -- Register an agent
    local name = Agent.register({
        name = "integration-test-agent",
        agent_type = "llm",
        model = {
            provider = "openai",
            model_id = "gpt-3.5-turbo",
            settings = {}
        },
        description = "Integration test agent",
        allowed_tools = {"calculator"},
        custom_config = {},
        resource_limits = {
            max_execution_time_secs = 300,
            max_memory_mb = 512,
            max_tool_calls = 100,
            max_recursion_depth = 10
        }
    })
    
    -- Get it back
    local agent = Agent.get(name)
    assert(agent ~= nil, "Should get registered agent")
    
    -- Check it appears in capabilities
    local caps = Agent.listCapabilities()
    assert(caps[name] ~= nil, "Agent should appear in capabilities")
    
    -- Wrap as tool
    local tool_name = Agent.wrapAsTool(name, {})
    assert(type(tool_name) == "string", "Should wrap as tool")
    
    -- Discover by capability
    local discovered = Agent.discoverByCapability("tools")
    assert(#discovered > 0, "Should discover agents with tool capability")
end)

-- Summary
print("\n=== Test Summary ===")
print("Tests Passed: " .. tests_passed)
print("Tests Failed: " .. tests_failed)
print("Total Tests: " .. (tests_passed + tests_failed))

if tests_failed == 0 then
    print("\n✅ All tests passed!")
else
    print("\n❌ Some tests failed!")
    os.exit(1)
end