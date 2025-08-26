-- Example: state-persistence.lua
-- Author: LLMSpell Examples
-- Purpose: Demonstrate state persistence capabilities with scoped namespaces
-- Config: Requires state-enabled configuration (see configs/state-enabled.toml)
-- Learning: How to persist data across script executions

print("=== LLMSpell: State Persistence ===")
print("This example demonstrates how to use the State API with scoped namespaces!")
print()

-- Check if State is available
if not State then
    print("❌ State API not available!")
    print("   Run with: ./llmspell -c configs/state-enabled.toml run features/state-persistence.lua")
    return
end

print("✅ State API is available!")
print()

-- 1. Basic save and load with global scope
print("1. Basic operations with 'global' scope...")
State.save("global", "app_version", "1.0.0")
State.save("global", "last_run", os.date())
print("   Saved app_version and last_run")

local version = State.load("global", "app_version")
local last_run = State.load("global", "last_run")
print("   Loaded: version=" .. tostring(version) .. ", last_run=" .. tostring(last_run))

-- 2. Using custom scope for user data
print("\n2. Using 'custom' scope for user preferences...")
State.save("custom", "theme", "dark")
State.save("custom", "language", "en")
State.save("custom", "font_size", "14")

local theme = State.load("custom", "theme")
print("   Current theme: " .. tostring(theme))

-- 3. Listing keys in a scope
print("\n3. Listing all keys in scopes...")
local global_keys = State.list_keys("global")
print("   Global scope keys: " .. #global_keys)
for i, key in ipairs(global_keys) do
    local value = State.load("global", key)
    print("     - " .. key .. " = " .. tostring(value))
end

local custom_keys = State.list_keys("custom")
print("   Custom scope keys: " .. #custom_keys)
for i, key in ipairs(custom_keys) do
    local value = State.load("custom", key)
    print("     - " .. key .. " = " .. tostring(value))
end

-- 4. Working with structured data (as strings)
print("\n4. Storing structured data...")
-- For complex data, store as formatted strings
local user_profile = "name:John Doe;email:john@example.com;theme:dark"
State.save("custom", "user_profile", user_profile)
print("   Saved user profile as string")

-- Load and parse
local loaded_profile = State.load("custom", "user_profile")
if loaded_profile then
    print("   Loaded profile: " .. tostring(loaded_profile))
end

-- 5. Workflow-specific state
print("\n5. Workflow-specific state helpers...")
-- These convenience methods use the workflow: prefix internally
local wf_data = State.workflow_get("data_pipeline", "step1")
print("   Workflow step1 data: " .. tostring(wf_data))

-- You can also use the workflow scope directly
State.save("workflow", "pipeline:status", "running")
local status = State.load("workflow", "pipeline:status")
print("   Pipeline status: " .. tostring(status))

-- 6. Agent-specific state
print("\n6. Agent-specific state helpers...")
State.agent_set("analyzer_001", "conversation_count", "5")
State.agent_set("analyzer_001", "last_topic", "data analysis")
local count = State.agent_get("analyzer_001", "conversation_count")
print("   Agent conversation count: " .. tostring(count))

-- 7. Tool-specific state
print("\n7. Tool-specific state helpers...")
State.tool_set("file_processor", "files_processed", "42")
State.tool_set("file_processor", "last_file", "/tmp/data.csv")
local processed = State.tool_get("file_processor", "files_processed")
print("   Tool files processed: " .. tostring(processed))

-- 8. Deleting specific keys
print("\n8. Cleaning up old data...")
State.delete("global", "temp_data")
State.delete("custom", "old_preference")
print("   Deleted temporary and old keys")

print("\n📊 State Summary:")
print("   Scopes: global, custom, workflow, agent, tool")
print("   Each scope provides namespace isolation")
print("   Data persists across script executions")
print("   Use appropriate scope for your data type")

print("\n🎉 State persistence example complete!")
print("   Run this script again to see data persistence!")
print("   Your saved values will still be there!")