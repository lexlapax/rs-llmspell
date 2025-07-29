-- ABOUTME: Example demonstrating agent state persistence
-- ABOUTME: Shows how to save and load agent state using the State API

-- CONFIG: Use examples/configs/state-enabled.toml
-- WHY: This example requires state persistence to be enabled with a backend
-- HOW TO RUN: ./target/debug/llmspell -c examples/configs/state-enabled.toml run examples/lua/state/agent_state_persistence.lua
-- ALTERNATIVE: cargo run -- -c examples/configs/state-enabled.toml run examples/lua/state/agent_state_persistence.lua

print("🤖 Agent State Persistence Example")
print("===================================")

-- This example demonstrates:
-- 1. Creating an agent with conversation
-- 2. Saving agent state (metadata, conversation, config)
-- 3. Loading/checking agent state
-- 4. Deleting agent state
-- 5. Registry management for saved agents

-- Create a simple test agent
local agent = Agent.create({
    name = "state-test-agent",
    model = "mock/test",  -- Use mock provider for testing
    description = "Agent for testing state persistence",
    system_prompt = "You are a helpful test agent",
    temperature = 0.7,
    max_conversation_length = 50
})

print("Created agent: " .. agent.getAgentState())

-- Step 1: Build conversation history
print("\n1. Building Conversation History")
print(string.rep("-", 40))

local response1 = agent:invoke({
    text = "Hello! My name is Alice. I'm interested in learning about AI."
})
print("Agent response 1: " .. response1.text)

local response2 = agent:invoke({
    text = "What is my name and what am I interested in?"
})
print("Agent response 2: " .. response2.text)

-- Step 2: Save agent state
print("\n2. Saving Agent State")
print(string.rep("-", 40))

print("Saving agent state (metadata, conversation, config)...")
agent:saveState()
print("✅ State saved successfully")

-- Verify what was saved by checking the registry
local saved_agents = State.get(State.scope.Global, "saved_agents_registry")
if saved_agents then
    print("📋 Saved agents registry: " .. json.encode(saved_agents))
else
    print("📋 No saved agents registry found")
end

-- Step 3: Check state existence
print("\n3. Checking State Existence")
print(string.rep("-", 40))

local state_exists = agent:loadState()
print("State exists: " .. tostring(state_exists))

if state_exists then
    print("✅ Agent state is persisted and can be found")
    print("⚠️  Note: Due to Arc<dyn Agent> limitation, actual conversation")
    print("    history cannot be automatically restored to running instance.")
    print("    This would require agent recreation or architectural changes.")
else
    print("❌ No state found for this agent")
end

-- Step 4: Clean up
print("\n4. Cleanup - Delete Saved State")
print(string.rep("-", 40))

print("Deleting agent state...")
agent:deleteState()
print("✅ State deleted successfully")

-- Verify deletion
local state_exists_after = agent:loadState()
print("State exists after delete: " .. tostring(state_exists_after))

-- Verify registry cleanup
local saved_agents_after = State.get(State.scope.Global, "saved_agents_registry")
if saved_agents_after then
    print("📋 Registry after cleanup: " .. json.encode(saved_agents_after))
else
    print("📋 Registry cleaned up")
end

-- Step 5: Cleanup
print("\n5. Agent Cleanup")
print(string.rep("-", 40))

agent:destroy()
print("✅ Agent destroyed")

print("\n🎉 Agent state persistence example completed!")
print("Key features demonstrated:")
print("  • Automatic state save on pause/stop lifecycle events")
print("  • Manual state save/load/delete via Lua API")
print("  • Registry management for tracking saved agents")
print("  • Complete state including metadata, conversation, and config")