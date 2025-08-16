-- Example: 04-save-state.lua
-- Author: LLMSpell Examples
-- Purpose: Introduction to state persistence - saving data between runs
-- Learning: State management and persistence across script executions

print("=== LLMSpell: Saving State ===")
print("This example shows how to save and load state between script runs!")
print()

-- Check if state persistence is available
if not State then
    print("❌ State persistence not available. Please run with a state-enabled configuration:")
    print("   llmspell run --config examples/script-users/configs/state-enabled.toml 04-save-state.lua")
    return
end

print("1. Checking existing state...")

-- Try to load existing counter from previous runs
local counter_result = State and State.get("example_counter")
local counter = 0

if counter_result and counter_result.success and counter_result.result then
    counter = tonumber(counter_result.result) or 0
    print("✅ Found existing counter: " .. counter)
else
    print("ℹ️  No existing counter found, starting fresh")
end

print()
print("2. Incrementing counter...")
counter = counter + 1
print("   New counter value: " .. counter)

-- Save the updated counter
local save_result = State and State.set("example_counter", tostring(counter))
if save_result and save_result.success then
    print("✅ Counter saved successfully!")
else
    print("❌ Error saving counter: " .. (save_result and save_result.error or "State not available"))
    return
end

print()
print("3. Saving run metadata...")

-- Save information about this run
local run_info = {
    timestamp = os.date("%Y-%m-%d %H:%M:%S"),
    run_number = counter,
    message = "Run #" .. counter .. " completed successfully"
}

-- Convert table to string for storage
local run_info_str = string.format("timestamp=%s,run_number=%d,message=%s", 
    run_info.timestamp, run_info.run_number, run_info.message)

local metadata_result = State and State.set("last_run_info", run_info_str)
if metadata_result and metadata_result.success then
    print("✅ Run metadata saved!")
    print("   Timestamp: " .. run_info.timestamp)
    print("   Run number: " .. run_info.run_number)
else
    print("❌ Error saving metadata: " .. (metadata_result and metadata_result.error or "State not available"))
end

print()
print("4. Demonstrating session-scoped state...")

-- Session state persists only during this session
if Session then
    local session_key = "session_demo_" .. os.time()
    local session_result = Session.set(session_key, "This data exists only during this session")
    
    if session_result.success then
        print("✅ Session data saved with key: " .. session_key)
        
        -- Retrieve it immediately
        local retrieve_result = Session.get(session_key)
        if retrieve_result.success then
            print("   Retrieved: " .. (retrieve_result.result or "No data"))
        end
    end
else
    print("ℹ️  Session state not available in this configuration")
end

print()
print("5. Listing all saved state keys...")
local list_result = State and State.list()
if list_result and list_result.success then
    print("📋 Current state keys:")
    for i, key in ipairs(list_result.result) do
        print("   " .. i .. ". " .. key)
    end
else
    print("❌ Error listing state: " .. (list_result and list_result.error or "State not available"))
end

print()
print("🎉 Congratulations! You've successfully:")
print("   - Loaded existing state from previous runs")
print("   - Modified and saved state data")
print("   - Stored metadata about script execution")
print("   - Listed all state keys")
print()
print("💡 Try running this script again to see the counter increment!")
print("   The counter will persist between runs because it's saved to state.")
print()
print("Next: Try 05-handle-errors.lua to learn about error handling!")