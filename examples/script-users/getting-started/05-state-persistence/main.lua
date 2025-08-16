-- Example: State Persistence - Saving Data Between Runs
-- Purpose: Learn how to save and retrieve state across script executions
-- Audience: Script Users (Beginners)
-- Prerequisites: Completed 04-basic-workflow
-- Expected Output: State persistence demonstration
-- Version: 0.7.0
-- Tags: getting-started, state, persistence, storage, beginner

print("=== State Persistence ===")
print("")

-- State allows you to save data that persists between script runs.
-- This is useful for counters, user preferences, conversation history, etc.

-- Note: State may require configuration. If not available, we'll use files.

print("1. Checking if State is available...")

local state_available = false
local success, err = pcall(function()
    -- Try to get a simple value
    local test = State.get("test_key")
    state_available = true
end)

if not state_available then
    print("   ⚠️  State API not available. Using file-based persistence.")
    print("   To enable State, configure state persistence in your config.")
    print("")
else
    print("   ✅ State API is available!")
    print("")
end

-- Helper functions for file-based state (fallback)
local function save_to_file(key, value)
    local path = "/tmp/llmspell_state_" .. key .. ".txt"
    Tool.invoke("file_operations", {
        operation = "write",
        path = path,
        input = tostring(value)
    })
end

local function load_from_file(key, default)
    local path = "/tmp/llmspell_state_" .. key .. ".txt"
    local result = Tool.invoke("file_operations", {
        operation = "read",
        path = path
    })
    if result and result.text then
        return result.text
    end
    return default
end

-- Wrapper functions that use State API or files
local function save_state(key, value)
    if state_available then
        State.set(key, value)
    else
        save_to_file(key, value)
    end
end

local function load_state(key, default)
    if state_available then
        local value = State.get(key)
        return value or default
    else
        return load_from_file(key, default)
    end
end

-- Example 1: Visit counter
print("2. Visit Counter Example")
print("")

local visit_count = tonumber(load_state("visit_count", "0")) or 0
visit_count = visit_count + 1
save_state("visit_count", tostring(visit_count))

print("   This script has been run " .. visit_count .. " time(s)")
print("   (Try running this script multiple times!)")
print("")

-- Example 2: User preferences
print("3. User Preferences Example")
print("")

local last_run = load_state("last_run_time", "never")
local current_time = os.date("%Y-%m-%d %H:%M:%S")
save_state("last_run_time", current_time)

print("   Last run: " .. last_run)
print("   Current run: " .. current_time)
print("")

-- Example 3: Simple todo list
print("4. Simple Todo List")
print("")

-- Load existing todos
local todos_str = load_state("todos", "[]")
local todos = {}

-- Simple parsing (in real code, use json_processor tool)
if todos_str ~= "[]" then
    for item in todos_str:gmatch("([^,]+)") do
        table.insert(todos, item:match("^%s*(.-)%s*$"))  -- trim whitespace
    end
end

-- Add a new todo with timestamp
table.insert(todos, "Task added at " .. os.date("%H:%M:%S"))

-- Keep only last 3 todos
while #todos > 3 do
    table.remove(todos, 1)
end

-- Save todos
local todos_to_save = table.concat(todos, ", ")
save_state("todos", todos_to_save)

print("   Your last 3 todos:")
for i, todo in ipairs(todos) do
    print("   " .. i .. ". " .. todo)
end
print("")

-- Example 4: Session data
print("5. Session Information")
print("")

-- Create a session ID if it doesn't exist
local session_id = load_state("session_id", nil)
if not session_id then
    local id_result = Tool.invoke("uuid_generator", {
        operation = "generate",
        version = "v4"
    })
    session_id = id_result and id_result.text or "unknown"
    save_state("session_id", session_id)
    print("   New session created: " .. session_id)
else
    print("   Continuing session: " .. session_id)
end

print("")

-- Clean up option (commented out - uncomment to reset)
-- print("6. Resetting state (uncomment to enable)...")
-- save_state("visit_count", "0")
-- save_state("last_run_time", "never")
-- save_state("todos", "[]")
-- save_state("session_id", nil)
-- print("   State reset!")

print("")
print("🎉 Congratulations! You've successfully:")
print("   - Learned about state persistence")
print("   - Implemented a visit counter")
print("   - Saved user preferences")
print("   - Created a simple todo list")
print("   - Managed session data")
print("")
print("💡 Key Concepts:")
print("   - State persists data between script runs")
print("   - Use State.set(key, value) to save data")
print("   - Use State.get(key) to retrieve data")
print("   - File-based storage works as a fallback")
print("")
print("Next: Continue to '06-error-handling' to learn robust error handling!")