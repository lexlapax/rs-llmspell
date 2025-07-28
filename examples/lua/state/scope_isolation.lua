-- ABOUTME: Demonstrates state scope isolation and namespace management
-- ABOUTME: Shows how different scopes keep state separate and organized

-- CONFIG: Use examples/configs/state-enabled.toml
-- WHY: Shows all four scope types (global, system, agent:*, workflow:*) which need state persistence
-- HOW TO RUN: ./target/debug/llmspell -c examples/configs/state-enabled.toml run examples/lua/state/scope_isolation.lua
-- ALTERNATIVE: cargo run -- -c examples/configs/state-enabled.toml run examples/lua/state/scope_isolation.lua

print("🔒 State Scope Isolation Example")
print("=====================================")

-- LLMSpell supports four main scope types:
-- 1. "global" - Application-wide state
-- 2. "system" - System configuration and metadata
-- 3. "agent:name" - Agent-specific state
-- 4. "workflow:name" - Workflow-specific state

-- 1. Set up data in different scopes
print("\n1. Creating data in different scopes...")

-- Global scope - application-wide settings
State.save("global", "app_name", "LLMSpell Application")
State.save("global", "version", "1.0.0")
State.save("global", "user_count", 42)

-- System scope - system configuration
State.save("system", "install_date", os.time())
State.save("system", "license_key", "DEMO-1234-5678")
State.save("system", "features_enabled", {
    advanced_ai = true,
    multi_agent = true,
    state_persistence = true
})

-- Agent scopes - individual agent data
State.save("agent:analyzer", "type", "data-analyzer")
State.save("agent:analyzer", "processed_items", 1000)
State.save("agent:analyzer", "accuracy", 0.95)

State.save("agent:generator", "type", "content-generator")
State.save("agent:generator", "generated_items", 500)
State.save("agent:generator", "quality_score", 0.88)

-- Workflow scopes - workflow-specific data
State.save("workflow:data-pipeline", "status", "running")
State.save("workflow:data-pipeline", "stages", {
    "input", "validation", "processing", "output"
})
State.save("workflow:data-pipeline", "current_stage", 2)

State.save("workflow:report-generation", "status", "idle")
State.save("workflow:report-generation", "last_run", os.time() - 3600)
State.save("workflow:report-generation", "output_format", "pdf")

print("✅ Created data in 6 different scopes")

-- 2. Demonstrate scope isolation
print("\n2. Showing data by scope (demonstrating isolation)...")

-- Define expected keys for each scope
local scope_keys = {
    global = {"app_name", "version", "user_count", "status", "active_agents", "active_workflows"},
    system = {"install_date", "license_key", "features_enabled", "status", "component_health"},
    ["agent:analyzer"] = {"type", "processed_items", "accuracy", "status"},
    ["agent:generator"] = {"type", "generated_items", "quality_score"},
    ["workflow:data-pipeline"] = {"status", "stages", "current_stage"},
    ["workflow:report-generation"] = {"status", "last_run", "output_format"}
}

local function show_scope_data(scope, expected_keys)
    print("\n   Scope: '" .. scope .. "'")
    local found = 0
    for _, key in ipairs(expected_keys) do
        local value = State.load(scope, key)
        if value ~= nil then
            found = found + 1
            local value_str = type(value) == "table" and "table" or tostring(value)
            print("     - " .. key .. " = " .. value_str)
        end
    end
    print("     (" .. found .. " entries)")
end

-- Show all scopes
for scope, keys in pairs(scope_keys) do
    show_scope_data(scope, keys)
end

-- 3. Demonstrate same key in different scopes
print("\n3. Same key in different scopes...")

-- Save "status" in multiple scopes
State.save("global", "status", "healthy")
State.save("system", "status", "operational")
State.save("agent:analyzer", "status", "active")
State.save("workflow:data-pipeline", "status", "running")

-- Load and display
print("   Key 'status' in different scopes:")
print("   - global scope: " .. tostring(State.load("global", "status")))
print("   - system scope: " .. tostring(State.load("system", "status")))
print("   - agent:analyzer scope: " .. tostring(State.load("agent:analyzer", "status")))
print("   - workflow:data-pipeline scope: " .. tostring(State.load("workflow:data-pipeline", "status")))
print("   ✅ Same key maintains different values in each scope!")

-- 4. Cross-scope operations (intentional sharing)
print("\n4. Cross-scope data sharing patterns...")

-- Pattern 1: Global registry of active components
State.save("global", "active_agents", {
    "analyzer",
    "generator"
})
State.save("global", "active_workflows", {
    "data-pipeline",
    "report-generation"
})

-- Pattern 2: System monitoring data
State.save("system", "component_health", {
    ["agent:analyzer"] = "healthy",
    ["agent:generator"] = "healthy", 
    ["workflow:data-pipeline"] = "running",
    ["workflow:report-generation"] = "idle"
})

print("   Created global registry and system monitoring data")
print("   This allows coordinated access while maintaining isolation")

-- 5. Scope naming conventions and patterns
print("\n5. Scope naming patterns...")

-- Dynamic scope creation
local function create_timestamped_workflow()
    local workflow_id = "workflow:backup-" .. os.time()
    State.save(workflow_id, "type", "backup")
    State.save(workflow_id, "created", os.time())
    State.save(workflow_id, "status", "pending")
    return workflow_id
end

local backup_workflow = create_timestamped_workflow()
print("   Created dynamic workflow: " .. backup_workflow)

-- Hierarchical naming pattern
State.save("agent:coordinator.scheduler", "type", "sub-component")
State.save("agent:coordinator.executor", "type", "sub-component")
print("   Created hierarchical agent components (using dot notation)")

-- 6. Scope security and access patterns
print("\n6. Scope access patterns...")

-- Function to check if scope has data
local function scope_has_data(scope, check_keys)
    for _, key in ipairs(check_keys) do
        if State.load(scope, key) ~= nil then
            return true
        end
    end
    return false
end

-- Function to copy data between scopes
local function copy_scope_data(from_scope, to_scope, keys_to_copy)
    local copied = 0
    for _, key in ipairs(keys_to_copy) do
        local value = State.load(from_scope, key)
        if value ~= nil then
            State.save(to_scope, key, value)
            copied = copied + 1
        end
    end
    return copied
end

-- Example: Archive an agent's state
local analyzer_keys = scope_keys["agent:analyzer"] or {}
if scope_has_data("agent:analyzer", analyzer_keys) then
    local copied = copy_scope_data("agent:analyzer", "agent:analyzer-archive", analyzer_keys)
    print("   Archived analyzer agent: " .. copied .. " entries copied")
end

-- 7. Cleanup demonstrations
print("\n7. Scope cleanup patterns...")

-- Clean up a specific scope
local function cleanup_scope(scope, keys_to_clean)
    local cleaned = 0
    for _, key in ipairs(keys_to_clean) do
        local value = State.load(scope, key)
        if value ~= nil then
            State.delete(scope, key)
            cleaned = cleaned + 1
        end
    end
    return cleaned
end

-- Clean up dynamic workflow
local workflow_keys = {"type", "created", "status"}
local cleaned = cleanup_scope(backup_workflow, workflow_keys)
print("   Cleaned up " .. backup_workflow .. ": " .. cleaned .. " entries removed")

-- 8. Best practices summary
print("\n8. Scope usage best practices...")

print("\n   Global scope ('global'):")
print("   - Application-wide settings")
print("   - Shared registries and indexes")
print("   - Cross-component communication")

print("\n   System scope ('system'):")
print("   - Installation and licensing")
print("   - System-level configuration")
print("   - Infrastructure metadata")

print("\n   Agent scopes ('agent:name'):")
print("   - Individual agent state")
print("   - Agent-specific configuration")
print("   - Performance metrics")

print("\n   Workflow scopes ('workflow:name'):")
print("   - Workflow execution state")
print("   - Progress tracking")
print("   - Workflow-specific data")

-- 9. Final scope summary
print("\n9. Final scope summary...")

-- Update scope_keys with additional scopes
scope_keys["agent:analyzer-archive"] = analyzer_keys
scope_keys["agent:coordinator.scheduler"] = {"type"}
scope_keys["agent:coordinator.executor"] = {"type"}

-- Count entries in each scope
for scope, keys in pairs(scope_keys) do
    local count = 0
    for _, key in ipairs(keys) do
        if State.load(scope, key) ~= nil then
            count = count + 1
        end
    end
    if count > 0 then
        print(string.format("   %-30s: %d entries", scope, count))
    end
end

print("\n✅ Scope isolation example completed!")
print("\nKey takeaways:")
print("- Scopes provide complete isolation between components")
print("- Same keys can exist in different scopes without conflict") 
print("- Use consistent naming patterns for organization")
print("- Global scope for shared data, specific scopes for isolation")
print("- Clean up scopes when components are removed")