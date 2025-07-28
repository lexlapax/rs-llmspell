-- ABOUTME: Advanced recovery scenarios demonstrating point-in-time recovery
-- ABOUTME: Shows incremental backups, selective restore, and recovery strategies

-- CONFIG: Use examples/configs/backup-enabled.toml
-- WHY: This example uses full backup API including incremental backups
-- HOW TO RUN: ./target/debug/llmspell -c examples/configs/backup-enabled.toml run examples/lua/backup/recovery_scenarios.lua
-- ALTERNATIVE: cargo run -- -c examples/configs/backup-enabled.toml run examples/lua/backup/recovery_scenarios.lua
-- TODO: This file needs to be updated to use State.save/load instead of State.set/get/list
-- NOTE: See recovery_scenarios_fixed.lua for a working version

print("🔄 rs-llmspell Advanced Recovery Scenarios")
print("==========================================")

-- Helper function to display state
function display_state(title)
    print("\n" .. title)
    local keys = State.list("global")
    if #keys == 0 then
        print("   [Empty state]")
    else
        for _, key in ipairs(keys) do
            local value = State.get("global", key)
            if type(value) == "table" then
                print("   - " .. key .. " = <table>")
            else
                print("   - " .. key .. " = " .. tostring(value))
            end
        end
    end
end

-- Scenario 1: Time-based recovery simulation
print("\n📅 Scenario 1: Time-Based Recovery")
print("==================================")

-- Day 1: Initial setup
print("\nDay 1: Initial application setup")
State.set("global", "app_version", "1.0.0")
State.set("global", "user_count", 100)
State.set("global", "features", {
    search = true,
    export = false,
    themes = false
})
display_state("Initial state (v1.0.0)")

-- Create baseline backup
print("\n  Creating baseline backup...")
local day1_backup = State.create_backup(false)
print("  ✅ Baseline backup ID: " .. day1_backup.backup_id)

-- Day 2: Feature update
print("\nDay 2: Feature release")
State.set("global", "app_version", "1.1.0")
State.set("global", "user_count", 150)
local features = State.get("global", "features")
features.export = true
State.set("global", "features", features)
State.set("global", "export_count", 0)
display_state("After feature release (v1.1.0)")

-- Create incremental backup
print("\n  Creating incremental backup...")
local day2_backup = State.create_backup(true)
print("  ✅ Incremental backup ID: " .. day2_backup.backup_id)

-- Day 3: Major update with issues
print("\nDay 3: Major update")
State.set("global", "app_version", "2.0.0")
State.set("global", "user_count", 200)
State.set("global", "features", {
    search = true,
    export = true,
    themes = true,
    ai_assist = true -- New feature
})
State.set("global", "export_count", 42)
State.set("global", "theme_prefs", {
    default = "dark",
    custom_count = 5
})
display_state("After major update (v2.0.0)")

-- Create another incremental backup
print("\n  Creating incremental backup...")
local day3_backup = State.create_backup(true)
print("  ✅ Incremental backup ID: " .. day3_backup.backup_id)

-- Simulate critical bug discovered
print("\n⚠️  Critical bug discovered in v2.0.0!")
print("   AI assist feature corrupting user data")

-- Show backup chain
print("\n📋 Available recovery points:")
local backups = State.list_backups()
for i, backup in ipairs(backups) do
    print(string.format("   %d. %s - %s backup (size: %d bytes)",
        i,
        backup.id,
        backup.is_incremental and "Incremental" or "Full",
        backup.size_bytes
    ))
    if backup.parent_id then
        print("      └─ Parent: " .. backup.parent_id)
    end
end

-- Restore to day 2 state
print("\n🔄 Restoring to Day 2 state (v1.1.0)...")
local restore_result = State.restore_backup(day2_backup.backup_id)
if restore_result.success then
    print("✅ Successfully restored to stable version")
    display_state("Restored state (back to v1.1.0)")
else
    print("❌ Restore failed: " .. restore_result.error)
end

-- Scenario 2: Selective component recovery
print("\n\n🎯 Scenario 2: Selective Component Recovery")
print("===========================================")

-- Set up multi-component state
print("\nSetting up multi-component application state...")
State.set("user", "profile", {
    name = "John Doe",
    email = "john@example.com",
    preferences = { theme = "light", notifications = true }
})
State.set("user", "session_count", 5)

State.set("system", "config", {
    api_endpoint = "https://api.example.com",
    timeout = 30,
    retry_count = 3
})
State.set("system", "maintenance_mode", false)

State.set("cache", "recent_searches", {"rust", "lua", "backup"})
State.set("cache", "last_updated", os.time())

display_state("Full application state across scopes")

-- Create full backup
print("\n  Creating full system backup...")
local full_backup = State.create_backup(false)
print("  ✅ Full backup ID: " .. full_backup.backup_id)

-- Simulate partial data corruption
print("\n💥 Simulating cache corruption and user data loss...")
State.delete("cache", "recent_searches")
State.set("cache", "last_updated", -1) -- Invalid timestamp
State.delete("user", "profile")

display_state("State after corruption")

-- Note: In a real implementation, we would restore only specific scopes
print("\n🔄 Restoring from backup...")
local partial_restore = State.restore_backup(full_backup.backup_id)
if partial_restore.success then
    print("✅ Full restoration completed")
    display_state("Restored state")
end

-- Scenario 3: Backup chain validation
print("\n\n🔗 Scenario 3: Backup Chain Validation")
print("======================================")

-- Create a chain of backups
print("\nCreating backup chain...")
State.set("global", "chain_test", "version_1")
local chain_backup1 = State.create_backup(false)
print("  1️⃣ Full backup: " .. chain_backup1.backup_id)

State.set("global", "chain_test", "version_2")
local chain_backup2 = State.create_backup(true)
print("  2️⃣ Incremental: " .. chain_backup2.backup_id)

State.set("global", "chain_test", "version_3")
local chain_backup3 = State.create_backup(true)
print("  3️⃣ Incremental: " .. chain_backup3.backup_id)

-- Validate backup chain integrity
print("\n🔍 Validating backup chain...")
for _, backup_id in ipairs({chain_backup1.backup_id, chain_backup2.backup_id, chain_backup3.backup_id}) do
    local validation = State.validate_backup(backup_id)
    if validation.is_valid then
        print("  ✅ " .. backup_id .. " - Valid")
    else
        print("  ❌ " .. backup_id .. " - Invalid: " .. table.concat(validation.errors, ", "))
    end
end

-- Scenario 4: Recovery with progress monitoring
print("\n\n📊 Scenario 4: Recovery with Progress Monitoring")
print("===============================================")

-- Create large state for progress demonstration
print("\nCreating large state dataset...")
for i = 1, 20 do
    State.set("bulk", "data_" .. i, {
        id = i,
        value = "test_data_" .. i,
        timestamp = os.time(),
        metadata = { size = i * 100, important = i % 5 == 0 }
    })
end
print("  Created 20 bulk data entries")

-- Create backup of large dataset
print("\n  Creating backup of large dataset...")
local large_backup = State.create_backup(false)
print("  ✅ Large backup ID: " .. large_backup.backup_id)
print("     Size: " .. large_backup.size_bytes .. " bytes")
print("     Entries: " .. large_backup.entry_count)

-- Clear the data
print("\n  Clearing bulk data...")
local bulk_keys = State.list_keys("bulk")
for _, key in ipairs(bulk_keys) do
    State.delete("bulk", key)
end

-- Restore with simulated progress
print("\n🔄 Restoring large dataset...")
print("  [Progress tracking would show here in actual implementation]")
local large_restore = State.restore_backup(large_backup.backup_id)
if large_restore.success then
    print("✅ Large dataset restored successfully")
    local restored_keys = State.list_keys("bulk")
    print("   Restored " .. #restored_keys .. " entries")
end

-- Summary
print("\n\n🎉 Recovery Scenarios Completed!")
print("================================")
print("\nDemonstrated scenarios:")
print("  ✓ Time-based recovery (rollback to previous version)")
print("  ✓ Multi-component state management")
print("  ✓ Backup chain creation and validation")
print("  ✓ Large dataset backup and recovery")
print("\nKey takeaways:")
print("  • Incremental backups save storage space")
print("  • Backup chains enable point-in-time recovery")
print("  • Validation ensures backup integrity")
print("  • Recovery can handle complex state structures")