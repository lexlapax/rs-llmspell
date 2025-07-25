-- ABOUTME: Minimal test case to reproduce agent initialization bug
-- ABOUTME: Tests agent state synchronization between creation and invocation

print("=== Agent Initialization Bug Test ===")
print("Testing: Agent state synchronization issue")
print()

-- Test 1: Create agent and immediately invoke
print("Test 1: Create and immediately invoke")
local agent1 = Agent.create({
    name = "test-agent-1",
    model = "gpt-4o-mini",
    system_prompt = "You are a test agent."
})
print("   Agent created")

-- Try to invoke immediately
local success1, result1 = pcall(function()
    return agent1:invoke({text = "Hello"})
end)

if success1 then
    print("   ✅ Immediate invocation succeeded")
else
    print("   ❌ Immediate invocation failed: " .. tostring(result1))
end

print()

-- Test 2: Create agent, manually initialize, then invoke
print("Test 2: Create, initialize, then invoke")
local agent2 = Agent.create({
    name = "test-agent-2", 
    model = "gpt-4o-mini",
    system_prompt = "You are a test agent."
})
print("   Agent created")

-- Try manual initialization
if agent2.initialize then
    local init_success, init_err = pcall(function()
        agent2:initialize()
    end)
    if init_success then
        print("   ✅ Manual initialization succeeded")
    else
        print("   ❌ Manual initialization failed: " .. tostring(init_err))
    end
else
    print("   ⚠️  No initialize method available")
end

-- Small delay
os.execute("sleep 0.5")

-- Try to invoke after initialization
local success2, result2 = pcall(function()
    return agent2:invoke({text = "Hello"})
end)

if success2 then
    print("   ✅ Post-initialization invocation succeeded")
else
    print("   ❌ Post-initialization invocation failed: " .. tostring(result2))
end

print()

-- Test 3: Check agent state visibility
print("Test 3: Agent state visibility")
local agent3 = Agent.create({
    name = "test-agent-3",
    model = "gpt-4o-mini", 
    system_prompt = "You are a test agent."
})

-- Check if we can see agent state
if agent3.state then
    print("   Agent state: " .. tostring(agent3.state))
elseif agent3.getState then
    local state_success, state = pcall(function()
        return agent3:getState()
    end)
    if state_success then
        print("   Agent state: " .. tostring(state))
    else
        print("   ⚠️  Could not get state: " .. tostring(state))
    end
else
    print("   ⚠️  No state accessor available")
end

print()

-- Test 4: List agents to see if they're registered
print("Test 4: Agent registration check")
local agents = Agent.list()
print("   Total agents: " .. #agents)
for i, agent_info in ipairs(agents) do
    if type(agent_info) == "table" and agent_info.name then
        if agent_info.name:match("^test%-agent%-") then
            print("   - " .. agent_info.name .. " (state: " .. tostring(agent_info.state or "unknown") .. ")")
        end
    end
end

print()
print("=== Test Complete ===")
print()
print("Expected behavior:")
print("- Agents should be immediately invokable after creation")
print("- OR manual initialization should make them invokable")
print("- State should be visible and synchronized")
print()
print("Actual behavior: Agents remain in 'Uninitialized' state")