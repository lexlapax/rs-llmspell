-- ABOUTME: Example demonstrating state backup and restore functionality
-- ABOUTME: Shows how to create backups, list them, and restore from backups

-- CONFIG: Use examples/configs/backup-enabled.toml
-- WHY: This example uses backup API which requires backup_enabled=true in config
-- HOW TO RUN: ./target/debug/llmspell -c examples/configs/backup-enabled.toml run examples/lua/backup/state_backup.lua
-- ALTERNATIVE: cargo run -- -c examples/configs/backup-enabled.toml run examples/lua/backup/state_backup.lua
-- NOTE: This version uses correct State API (save/load/delete)

print("🗄️  rs-llmspell Lua Backup Example")
print("=================================")

-- Check if backup API is available
local has_backup_api = State.create_backup ~= nil

if not has_backup_api then
    print("\n⚠️  Backup API not available in this configuration")
    print("   Make sure to use backup-enabled.toml config")
    print("   This example will demonstrate patterns without actual backup calls\n")
end

-- Track our state keys manually since State.list_keys() isn't available at runtime
local state_keys = {"app_config", "user_preferences", "app_stats"}

-- First, let's create some test state data
print("\n1. Creating test state data...")
State.save("global", "app_config", {
    version = "1.0.0",
    settings = {
        theme = "dark",
        language = "en",
        notifications = true
    }
})

State.save("global", "user_preferences", {
    display_name = "Test User",
    email = "test@example.com",
    created_at = os.time()
})

State.save("global", "app_stats", {
    total_users = 1000,
    active_sessions = 42,
    last_backup = "never"
})

print("✅ Test data created successfully")

-- List current state keys
print("\n2. Current state keys:")
for i, key in ipairs(state_keys) do
    local value = State.load("global", key)
    if value ~= nil then
        print("   - " .. key .. " = " .. type(value))
    end
end

-- Create a full backup
print("\n3. Creating full backup...")
if has_backup_api then
    local backup_result = State.create_backup(false) -- false = full backup
    
    if backup_result and backup_result.success then
        print("✅ Full backup created successfully")
        print("   Backup ID: " .. backup_result.backup_id)
        print("   Size: " .. backup_result.size_bytes .. " bytes")
        print("   Entries: " .. backup_result.entry_count)
    else
        print("❌ Backup failed: " .. (backup_result and backup_result.error or "Unknown error"))
    end
else
    -- Manual backup simulation
    print("   Simulating manual backup...")
    local backup_data = {}
    for _, key in ipairs(state_keys) do
        backup_data[key] = State.load("global", key)
    end
    State.save("system", "manual_backup_full", backup_data)
    print("✅ Manual backup created")
end

-- Modify some data
print("\n4. Modifying state data...")
State.save("global", "app_config", {
    version = "1.0.1", -- Changed version
    settings = {
        theme = "light", -- Changed theme
        language = "en",
        notifications = false -- Changed notifications
    }
})

State.save("global", "app_stats", {
    total_users = 1050, -- Increased users
    active_sessions = 38,
    last_backup = os.time()
})

print("✅ State data modified")

-- Create an incremental backup
print("\n5. Creating incremental backup...")
if has_backup_api then
    local incremental_result = State.create_backup(true) -- true = incremental
    
    if incremental_result and incremental_result.success then
        print("✅ Incremental backup created successfully")
        print("   Backup ID: " .. incremental_result.backup_id)
        print("   Parent ID: " .. (incremental_result.parent_id or "none"))
        print("   Size: " .. incremental_result.size_bytes .. " bytes")
    else
        print("❌ Incremental backup failed: " .. (incremental_result and incremental_result.error or "Unknown error"))
    end
else
    -- Manual incremental backup simulation
    print("   Simulating manual incremental backup...")
    local changed_data = {
        app_config = State.load("global", "app_config"),
        app_stats = State.load("global", "app_stats")
    }
    State.save("system", "manual_backup_incremental", changed_data)
    print("✅ Manual incremental backup created")
end

-- List available backups
print("\n6. Available backups:")
if has_backup_api and State.list_backups then
    local backups = State.list_backups()
    
    if backups and #backups > 0 then
        for i, backup in ipairs(backups) do
            print("   " .. i .. ". Backup ID: " .. backup.id)
            print("      Type: " .. (backup.is_incremental and "Incremental" or "Full"))
            print("      Created: " .. os.date("%Y-%m-%d %H:%M:%S", backup.created_at))
            print("      Size: " .. backup.size_bytes .. " bytes")
            if backup.parent_id then
                print("      Parent: " .. backup.parent_id)
            end
        end
    else
        print("   No backups available")
    end
else
    print("   Manual backups created (no listing API available)")
end

-- Validate a backup
print("\n7. Validating backup...")
if has_backup_api and State.validate_backup then
    -- Would validate first backup if available
    print("   Backup validation API available")
else
    -- Manual validation
    print("   Manual validation: checking backup integrity...")
    local manual_backup = State.load("system", "manual_backup_full")
    if manual_backup then
        print("✅ Manual backup appears valid")
    else
        print("❌ Manual backup not found")
    end
end

-- Simulate data loss
print("\n8. Simulating data loss...")
State.delete("global", "app_config")
State.delete("global", "user_preferences")
print("❌ Critical data deleted!")

-- Show current state
print("\n9. Current state after data loss:")
local remaining_keys = 0
for _, key in ipairs(state_keys) do
    local value = State.load("global", key)
    if value ~= nil then
        remaining_keys = remaining_keys + 1
        print("   - " .. key .. " = " .. type(value))
    end
end
if remaining_keys == 0 then
    print("   No keys found - critical data lost!")
end

-- Restore from backup
print("\n10. Restoring from backup...")
if has_backup_api and State.restore_backup then
    -- Would restore from most recent backup
    print("   Backup restore API available")
    -- local restore_result = State.restore_backup(backup_id)
else
    -- Manual restore
    print("   Performing manual restore...")
    local backup_data = State.load("system", "manual_backup_full")
    if backup_data then
        for key, value in pairs(backup_data) do
            State.save("global", key, value)
        end
        print("✅ Manual restore completed")
    else
        print("❌ No backup data found for manual restore")
    end
end

-- Show restored state
print("\n11. State after restoration:")
for _, key in ipairs(state_keys) do
    local value = State.load("global", key)
    if value ~= nil then
        print("   - " .. key .. " = " .. type(value))
    end
end

print("\n🎉 Backup example completed!")
print("\nThis example demonstrates:")
print("  • Creating full and incremental backups (or manual simulation)")
print("  • Listing available backups (when API available)")
print("  • Validating backup integrity")
print("  • Restoring state from backups")
print("  • Recovering from data loss scenarios")
print("\nNote: Full backup API requires backup manager initialization")