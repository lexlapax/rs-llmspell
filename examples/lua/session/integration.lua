-- ABOUTME: Example demonstrating session integration with other globals
-- ABOUTME: Shows how sessions work with State, Events, Hooks, Agents, and Tools

-- CONFIG: Requires runtime integration (see README.md for current status)
-- WHY: Sessions are most powerful when integrated with other system components
-- STATUS: Session/Artifact globals implemented but not yet integrated into CLI runtime
-- TODO: Runtime needs to initialize SessionManager - see llmspell-bridge/src/runtime.rs
-- NOTE: Some features (hooks, events) may require additional configuration

print("🔗 Session Integration Example")
print("==============================")

-- This example demonstrates:
-- 1. Session + State integration
-- 2. Session + Event integration
-- 3. Session + Hook integration (if available)
-- 4. Session + Agent integration
-- 5. Session + Tool integration
-- 6. Session + Workflow patterns

-- Check available globals
local has_events = Event ~= nil
local has_hooks = Hook ~= nil
local has_agents = Agent ~= nil
local has_workflows = Workflow ~= nil

print("\n📋 Available integrations:")
print("  State: ✓ (always available)")
print("  Events:", has_events and "✓" or "❌ (not configured)")
print("  Hooks:", has_hooks and "✓" or "❌ (not configured)")
print("  Agents:", has_agents and "✓" or "❌ (not configured)")
print("  Workflows:", has_workflows and "✓" or "❌ (not configured)")

-- Step 1: Session + State Integration
print("\n\n1. Session + State Integration")
print(string.rep("-", 40))

-- Create a session that uses state for configuration
local session_id = Session.create({
    name = "Integration Demo",
    description = "Demonstrating session integration patterns",
    tags = {"integration", "demo"},
    metadata = {
        integrations = {"state", "events", "hooks"}
    }
})
print("✅ Created integration session:", session_id)

-- Store session configuration in state
State.set("session:" .. session_id .. ":config", {
    max_artifacts = 100,
    compression_threshold = 5000,
    auto_save = true,
    retention_days = 30
})
print("💾 Stored session config in State")

-- Use state to track session metrics
State.set("session:" .. session_id .. ":metrics", {
    operations = 0,
    artifacts_stored = 0,
    events_emitted = 0
})

-- Helper to update metrics
local function update_metric(metric_name, increment)
    local metrics = State.get("session:" .. session_id .. ":metrics") or {}
    metrics[metric_name] = (metrics[metric_name] or 0) + (increment or 1)
    metrics.last_updated = os.time()
    State.set("session:" .. session_id .. ":metrics", metrics)
end

-- Step 2: Session + Event Integration
if has_events then
    print("\n2. Session + Event Integration")
    print(string.rep("-", 40))
    
    -- Subscribe to session-related events
    local session_events = Event.subscribe("session.*")
    print("📡 Subscribed to session events")
    
    -- Emit session started event
    Event.publish("session.started", {
        session_id = session_id,
        name = "Integration Demo",
        timestamp = os.time()
    })
    update_metric("events_emitted")
    
    -- Store artifact and emit event
    local artifact_id = Artifact.store(
        session_id,
        "tool_result",
        "integration_test.txt",
        "Testing event integration",
        {source = "integration_demo"}
    )
    
    Event.publish("session.artifact.stored", {
        session_id = session_id,
        artifact_id = artifact_id,
        artifact_type = "tool_result"
    })
    update_metric("artifacts_stored")
    update_metric("events_emitted")
    
    print("✅ Published session events")
    
    -- Check for events (non-blocking)
    local event = Event.receive(session_events, 100)  -- 100ms timeout
    if event then
        print("📨 Received event:", event.event_type)
    end
else
    print("\n2. Session + Event Integration")
    print(string.rep("-", 40))
    print("⚠️  Events not available in this configuration")
end

-- Step 3: Session + Hook Integration
if has_hooks then
    print("\n3. Session + Hook Integration")
    print(string.rep("-", 40))
    
    -- Register a hook to capture tool executions within this session
    local handle = Hook.register("AfterToolExecution", function(context)
        -- Process hook context - store tool results as artifacts
        if context and context.data and context.data.result then
            local current_session = Session.getCurrent()
            if current_session then
                Artifact.store(
                    current_session,
                    "tool_result",
                    "hook_captured_" .. os.time() .. ".json",
                    JSON.stringify(context.data),
                    {
                        hook_point = "AfterToolExecution",
                        captured_at = os.date("!%Y-%m-%dT%H:%M:%SZ"),
                        correlation_id = context.correlation_id or "unknown"
                    }
                )
                print("📥 Hook captured tool result and stored as artifact")
            end
        end
        return "continue"
    end, "normal")
    
    print("🔗 Registered AfterToolExecution hook for session")
    
    -- Demonstrate hook triggering by executing a tool
    local calc = Tool.get("calculator")
    if calc then
        -- This will trigger our hook
        local result = calc:execute({
            operation = "evaluate",
            expression = "42 + 8"
        })
        print("🧮 Executed calculator (should trigger hook):", result)
        update_metric("operations")
    end
    
    -- Also store a direct artifact to show the difference
    Artifact.store(
        session_id,
        "system_generated", 
        "hook_demo.txt",
        "This demonstrates hook integration with session artifacts"
    )
    update_metric("artifacts_stored")
    
    -- Unregister the hook when done
    handle:unregister()
    print("✅ Hook integration demonstrated and cleaned up")
else
    print("\n3. Session + Hook Integration")
    print(string.rep("-", 40))
    print("⚠️  Hooks not available in this configuration")
end

-- Step 4: Session + Agent Integration
if has_agents then
    print("\n4. Session + Agent Integration")
    print(string.rep("-", 40))
    
    -- Create an agent that operates within a session
    local agent = Agent.create({
        name = "session_assistant",
        model = "anthropic/claude-3-5-sonnet-latest",
        system_prompt = "You are an assistant working within a session context. Be concise."
    })
    print("🤖 Created session-aware agent")
    
    -- Set session as context for agent
    Session.setCurrent(session_id)
    
    -- Agent interaction (stored as artifacts)
    local response = agent:invoke({text = "Summarize the session activities so far"})
    
    -- Store agent response as artifact
    Artifact.store(
        session_id,  -- Explicitly use session ID
        "agent_output",
        "agent_summary.txt",
        response and response.text or "No response",
        {
            agent = "session_assistant",
            prompt = "Summarize session activities"
        }
    )
    update_metric("artifacts_stored")
    update_metric("operations")
    
    print("✅ Agent response stored in session")
else
    print("\n4. Session + Agent Integration")
    print(string.rep("-", 40))
    print("⚠️  Agents not available in this configuration")
end

-- Step 5: Session + Tool Integration
print("\n5. Session + Tool Integration")
print(string.rep("-", 40))

-- Tools are always available
local calc = Tool.get("calculator")
if calc then
    -- Execute tool and store result in session
    local result = calc:execute({
        operation = "evaluate",
        expression = "42 * 10 + 7"
    })
    
    -- Store tool execution as artifact
    Artifact.store(
        session_id,
        "tool_result",
        "calculation_result.json",
        JSON.stringify({
            tool = "calculator",
            input = "42 * 10 + 7",
            result = result,
            timestamp = os.time()
        }),
        {
            mime_type = "application/json",
            tool = "calculator"
        }
    )
    update_metric("artifacts_stored")
    update_metric("operations")
    
    print("🧮 Calculator result stored:", result)
else
    print("⚠️  Calculator tool not found")
end

-- Step 6: Session + Workflow Integration
if has_workflows then
    print("\n6. Session + Workflow Integration")
    print(string.rep("-", 40))
    
    -- NOTE: Workflows require tool references, not inline functions
    -- Here's a simple workflow using existing tools
    local workflow = Workflow.sequential({
        name = "session_workflow",
        description = "Simple workflow that stores results in session",
        steps = {
            {
                name = "generate_id",
                tool = "uuid_generator",
                input = {}
            },
            {
                name = "format_result",
                tool = "template_engine",
                input = {
                    template = "Workflow {{id}} executed in session {{session}}",
                    variables = {
                        id = "{{step:generate_id:output}}",
                        session = session_id
                    }
                }
            }
        }
    })
    
    local result = workflow:execute()
    if result and result.success then
        -- Store workflow result as artifact
        Artifact.store(
            session_id,
            "system_generated",
            "workflow_result.txt",
            result.data and result.data.final_output or "Workflow completed",
            {
                workflow = "session_workflow",
                execution_time = os.time()
            }
        )
        update_metric("artifacts_stored")
        print("✅ Workflow executed and result stored in session")
    else
        print("⚠️  Workflow execution failed")
    end
else
    print("\n6. Session + Workflow Integration")
    print(string.rep("-", 40))
    print("⚠️  Workflows not available in this configuration")
end

-- Step 7: Advanced Integration Patterns
print("\n7. Advanced Integration Patterns")
print(string.rep("-", 40))

-- Pattern 1: Session-scoped state namespace
print("\n📌 Pattern 1: Session-scoped state")
local session_state_key = "session:" .. session_id .. ":data"
State.set(session_state_key, {
    user_preferences = {theme = "dark", language = "en"},
    workspace = {current_file = "main.lua", open_tabs = 3}
})
print("  ✓ Created session-scoped state namespace")

-- Pattern 2: Session artifact indexing
print("\n📌 Pattern 2: Artifact indexing")
local artifacts = Artifact.list(session_id)
local artifact_index = {}
for _, artifact in ipairs(artifacts) do
    local artifact_type = artifact.artifact_type
    artifact_index[artifact_type] = artifact_index[artifact_type] or {}
    table.insert(artifact_index[artifact_type], {
        name = artifact.name,
        id = artifact.id
    })
end
State.set("session:" .. session_id .. ":index", artifact_index)
print("  ✓ Created artifact type index in state")

-- Pattern 3: Session metrics summary
print("\n📌 Pattern 3: Metrics summary")
local metrics = State.get("session:" .. session_id .. ":metrics") or {}
print("  📊 Session metrics:")
for metric, value in pairs(metrics) do
    if metric ~= "last_updated" then
        print(string.format("     %s: %d", metric, value))
    end
end

-- Cleanup
print("\n8. Cleanup")
print(string.rep("-", 40))

-- Clean up state keys
State.delete("session:" .. session_id .. ":config")
State.delete("session:" .. session_id .. ":metrics")
State.delete("session:" .. session_id .. ":data")
State.delete("session:" .. session_id .. ":index")
print("🧹 Cleaned up session state")

-- Complete session
Session.complete(session_id)
print("✅ Session completed")

-- Summary
print("\n\n🎉 Integration Example Completed!")
print("=================================")
print("\nDemonstrated integrations:")
print("  ✓ State: Configuration, metrics, and data storage")
if has_events then
    print("  ✓ Events: Session lifecycle and artifact events")
end
if has_hooks then
    print("  ✓ Hooks: Intercepting session operations")
end
if has_agents then
    print("  ✓ Agents: Context-aware agent interactions")
end
print("  ✓ Tools: Storing tool results as artifacts")
if has_workflows then
    print("  ✓ Workflows: Session-aware workflow execution")
end
print("\nIntegration patterns:")
print("  • Session-scoped state namespaces")
print("  • Artifact indexing for fast lookup")
print("  • Metrics tracking and aggregation")
print("  • Event-driven session monitoring")
print("  • Context propagation across components")
print("\nBest practices:")
print("  • Use consistent key patterns for state")
print("  • Emit events for important operations")
print("  • Store all outputs as artifacts")
print("  • Clean up state on session completion")
print("  • Use current session context when possible")