-- ABOUTME: Agent-scoped state persistence example
-- ABOUTME: Shows how agents can maintain isolated persistent state

-- CONFIG: Use examples/configs/state-enabled.toml
-- WHY: Demonstrates agent-scoped state isolation which requires state persistence
-- HOW TO RUN: ./target/debug/llmspell -c examples/configs/state-enabled.toml run examples/lua/state/agent_state.lua
-- ALTERNATIVE: cargo run -- -c examples/configs/state-enabled.toml run examples/lua/state/agent_state.lua

print("🤖 Agent State Persistence Example")
print("=====================================")

-- Agent-scoped state uses the format "agent:agent-name"
-- This ensures each agent has its own isolated state namespace

-- 1. Create state for different agents
print("\n1. Creating state for multiple agents...")

-- Agent 1: Research Assistant
State.save("agent:research-assistant", "role", "Research and Information Gathering")
State.save("agent:research-assistant", "status", "active")
State.save("agent:research-assistant", "config", {
    max_sources = 10,
    preferred_domains = {"wikipedia.org", "arxiv.org", "pubmed.gov"},
    language = "en",
    summarize = true
})
State.save("agent:research-assistant", "statistics", {
    queries_handled = 0,
    documents_analyzed = 0,
    summaries_generated = 0
})

-- Agent 2: Code Assistant
State.save("agent:code-assistant", "role", "Code Generation and Review")
State.save("agent:code-assistant", "status", "idle")
State.save("agent:code-assistant", "config", {
    languages = {"lua", "python", "javascript", "rust"},
    style_guide = "google",
    auto_format = true,
    explain_code = true
})
State.save("agent:code-assistant", "recent_tasks", {})

-- Agent 3: Task Coordinator
State.save("agent:task-coordinator", "role", "Multi-Agent Task Coordination")
State.save("agent:task-coordinator", "status", "active")
State.save("agent:task-coordinator", "active_workflows", {})
State.save("agent:task-coordinator", "agent_registry", {
    "research-assistant",
    "code-assistant"
})

print("✅ Created state for 3 agents")

-- 2. Demonstrate scope isolation
print("\n2. Demonstrating scope isolation...")

-- Define expected keys for each agent
local agent_keys = {
    ["research-assistant"] = {"role", "status", "config", "statistics", "last_query"},
    ["code-assistant"] = {"role", "status", "config", "recent_tasks"},
    ["task-coordinator"] = {"role", "status", "active_workflows", "agent_registry"}
}

print("   Research Assistant state:")
for _, key in ipairs(agent_keys["research-assistant"]) do
    local value = State.load("agent:research-assistant", key)
    if value ~= nil then
        print("   - " .. key)
    end
end

print("\n   Code Assistant state:")
for _, key in ipairs(agent_keys["code-assistant"]) do
    local value = State.load("agent:code-assistant", key)
    if value ~= nil then
        print("   - " .. key)
    end
end

print("\n   Note: Each agent has its own isolated state!")

-- 3. Simulate agent activity with state updates
print("\n3. Simulating agent activity...")

-- Research Assistant performs a task
local stats = State.load("agent:research-assistant", "statistics")
stats.queries_handled = stats.queries_handled + 1
stats.documents_analyzed = stats.documents_analyzed + 5
stats.summaries_generated = stats.summaries_generated + 1
State.save("agent:research-assistant", "statistics", stats)
State.save("agent:research-assistant", "last_query", {
    query = "LLM optimization techniques",
    timestamp = os.time(),
    sources_found = 5
})

print("   Research Assistant handled a query")

-- Code Assistant performs a task
State.save("agent:code-assistant", "status", "busy")
local recent = State.load("agent:code-assistant", "recent_tasks") or {}
table.insert(recent, {
    type = "code_review",
    language = "lua",
    timestamp = os.time(),
    lines_reviewed = 150
})
-- Keep only last 5 tasks
if #recent > 5 then
    table.remove(recent, 1)
end
State.save("agent:code-assistant", "recent_tasks", recent)

print("   Code Assistant completed a code review")

-- Task Coordinator updates workflow
local workflows = State.load("agent:task-coordinator", "active_workflows") or {}
workflows["workflow-001"] = {
    name = "Research and Implement",
    agents = {"research-assistant", "code-assistant"},
    status = "in_progress",
    created = os.time()
}
State.save("agent:task-coordinator", "active_workflows", workflows)

print("   Task Coordinator started a new workflow")

-- 4. Agent state queries
print("\n4. Querying agent states...")

-- Check all agent statuses
local agents = {"research-assistant", "code-assistant", "task-coordinator"}
for _, agent in ipairs(agents) do
    local scope = "agent:" .. agent
    local status = State.load(scope, "status")
    local role = State.load(scope, "role")
    print(string.format("   %s:", agent))
    print(string.format("     Role: %s", role))
    print(string.format("     Status: %s", status))
end

-- 5. Agent lifecycle management
print("\n5. Agent lifecycle management...")

-- Pause an agent (save current state and mark as paused)
local function pause_agent(agent_name)
    local scope = "agent:" .. agent_name
    State.save(scope, "status", "paused")
    State.save(scope, "paused_at", os.time())
    print("   Paused agent: " .. agent_name)
end

-- Resume an agent
local function resume_agent(agent_name)
    local scope = "agent:" .. agent_name
    State.save(scope, "status", "active")
    State.delete(scope, "paused_at")
    print("   Resumed agent: " .. agent_name)
end

-- Stop an agent (save final state)
local function stop_agent(agent_name)
    local scope = "agent:" .. agent_name
    State.save(scope, "status", "stopped")
    State.save(scope, "stopped_at", os.time())
    
    -- Could also archive the state or clean up
    -- For this example, we'll keep the state for inspection
    print("   Stopped agent: " .. agent_name)
end

pause_agent("code-assistant")
resume_agent("code-assistant")
stop_agent("research-assistant")

-- 6. Agent state backup pattern
print("\n6. Agent state backup pattern...")

-- Create a snapshot of an agent's complete state
local function snapshot_agent(agent_name)
    local scope = "agent:" .. agent_name
    local keys = agent_keys[agent_name] or {}
    local snapshot = {
        agent_name = agent_name,
        timestamp = os.time(),
        state = {}
    }
    
    for _, key in ipairs(keys) do
        local value = State.load(scope, key)
        if value ~= nil then
            snapshot.state[key] = value
        end
    end
    
    -- Save snapshot with timestamp
    State.save("global", "snapshot:" .. agent_name .. ":" .. os.time(), snapshot)
    return snapshot
end

-- Utility function
local function table_length(t)
    local count = 0
    for _ in pairs(t) do count = count + 1 end
    return count
end

local snapshot = snapshot_agent("code-assistant")
print("   Created snapshot for code-assistant")
print("   Snapshot contains " .. table_length(snapshot.state) .. " state entries")

-- 7. Cross-agent communication via shared state
print("\n7. Cross-agent communication pattern...")

-- Agents can communicate through a shared message queue
State.save("global", "agent_messages", {})

-- Send a message from one agent to another
local function send_agent_message(from_agent, to_agent, message)
    local messages = State.load("global", "agent_messages") or {}
    table.insert(messages, {
        from = from_agent,
        to = to_agent,
        message = message,
        timestamp = os.time(),
        read = false
    })
    State.save("global", "agent_messages", messages)
end

-- Read messages for an agent
local function read_agent_messages(agent_name)
    local messages = State.load("global", "agent_messages") or {}
    local agent_messages = {}
    
    for i, msg in ipairs(messages) do
        if msg.to == agent_name and not msg.read then
            table.insert(agent_messages, msg)
            messages[i].read = true
        end
    end
    
    State.save("global", "agent_messages", messages)
    return agent_messages
end

send_agent_message("task-coordinator", "code-assistant", "Please review the research findings")
send_agent_message("code-assistant", "task-coordinator", "Review complete, ready for implementation")

local coordinator_messages = read_agent_messages("task-coordinator")
print("   Task Coordinator has " .. #coordinator_messages .. " new message(s)")

-- 8. Cleanup specific agent state
print("\n8. Cleaning up agent state...")

local function cleanup_agent(agent_name)
    local scope = "agent:" .. agent_name
    local keys = agent_keys[agent_name] or {}
    local cleaned = 0
    for _, key in ipairs(keys) do
        local value = State.load(scope, key)
        if value ~= nil then
            State.delete(scope, key)
            cleaned = cleaned + 1
        end
    end
    print("   Cleaned up " .. cleaned .. " entries for: " .. agent_name)
end

-- Clean up stopped agent
cleanup_agent("research-assistant")

-- Verify cleanup
print("\n9. Final state summary:")
for _, agent in ipairs(agents) do
    local scope = "agent:" .. agent
    local keys = agent_keys[agent] or {}
    local count = 0
    for _, key in ipairs(keys) do
        if State.load(scope, key) ~= nil then
            count = count + 1
        end
    end
    print("   " .. agent .. ": " .. count .. " state entries")
end

print("\n✅ Agent state persistence example completed!")
print("\nKey patterns demonstrated:")
print("- Use 'agent:name' scope for agent-specific state")
print("- Each agent has isolated state namespace")
print("- State persists across agent lifecycle")
print("- Agents can communicate through shared global state")
print("- Snapshot pattern for state backup")
print("- Proper cleanup when agents are removed")