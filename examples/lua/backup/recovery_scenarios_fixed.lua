-- ABOUTME: Advanced recovery scenarios demonstrating backup patterns
-- ABOUTME: Shows backup creation, validation, and recovery strategies

-- CONFIG: Use examples/configs/backup-enabled.toml
-- WHY: Backup API (create_backup, restore_backup) requires backup_enabled=true and backup manager
-- HOW TO RUN: ./target/debug/llmspell -c examples/configs/backup-enabled.toml run examples/lua/backup/recovery_scenarios_fixed.lua
-- ALTERNATIVE: cargo run -- -c examples/configs/backup-enabled.toml run examples/lua/backup/recovery_scenarios_fixed.lua
-- NOTE: Without backup-enabled config, this example demonstrates manual backup patterns

print("🔄 rs-llmspell Advanced Recovery Scenarios")
print("==========================================")

-- Note: This example demonstrates backup patterns but actual backup API
-- (create_backup, restore_backup, validate_backup) requires backup manager initialization

-- Helper function to display state
local function display_state(title, scope, keys)
    print("\n" .. title)
    if #keys == 0 then
        print("   [Empty state]")
    else
        for _, key in ipairs(keys) do
            local value = State.load(scope, key)
            if value ~= nil then
                if type(value) == "table" then
                    print("   - " .. key .. " = <table>")
                else
                    print("   - " .. key .. " = " .. tostring(value))
                end
            end
        end
    end
end

-- Check if backup API is available
local has_backup_api = State.create_backup ~= nil

if not has_backup_api then
    print("\n⚠️  Backup API not available in this configuration")
    print("   The State global needs backup manager initialization")
    print("   This example will demonstrate the patterns you would use\n")
end

-- Scenario 1: Time-based recovery simulation
print("\n📅 Scenario 1: Time-Based Recovery Pattern")
print("==========================================")

-- Define tracked keys
local app_keys = {"app_version", "user_count", "features", "export_count", "theme_prefs"}

-- Day 1: Initial setup
print("\nDay 1: Initial application setup")
State.save("global", "app_version", "1.0.0")
State.save("global", "user_count", 100)
State.save("global", "features", {
    search = true,
    export = false,
    themes = false
})
display_state("Initial state (v1.0.0)", "global", app_keys)

-- Simulate baseline backup
print("\n  Creating baseline backup...")
if has_backup_api then
    local day1_backup = State.create_backup(false)
    print("  ✅ Baseline backup ID: " .. (day1_backup.backup_id or "simulated"))
else
    -- Manual backup simulation
    local backup_data = {}
    for _, key in ipairs(app_keys) do
        backup_data[key] = State.load("global", key)
    end
    State.save("system", "backup:day1", backup_data)
    print("  ✅ Simulated baseline backup saved")
end

-- Day 2: Feature update
print("\nDay 2: Feature release")
State.save("global", "app_version", "1.1.0")
State.save("global", "user_count", 150)
local features = State.load("global", "features") or {}
features.export = true
State.save("global", "features", features)
State.save("global", "export_count", 0)
display_state("After feature release (v1.1.0)", "global", app_keys)

-- Simulate incremental backup
print("\n  Creating incremental backup...")
if has_backup_api then
    local day2_backup = State.create_backup(true)
    print("  ✅ Incremental backup ID: " .. (day2_backup.backup_id or "simulated"))
else
    -- Manual incremental backup simulation
    local backup_data = {}
    for _, key in ipairs(app_keys) do
        backup_data[key] = State.load("global", key)
    end
    State.save("system", "backup:day2", backup_data)
    print("  ✅ Simulated incremental backup saved")
end

-- Day 3: Major update with issues
print("\nDay 3: Major update")
State.save("global", "app_version", "2.0.0")
State.save("global", "user_count", 200)
State.save("global", "features", {
    search = true,
    export = true,
    themes = true,
    ai_assist = true -- New feature
})
State.save("global", "export_count", 42)
State.save("global", "theme_prefs", {
    default = "dark",
    custom_count = 5
})
display_state("After major update (v2.0.0)", "global", app_keys)

-- Simulate critical bug discovered
print("\n⚠️  Critical bug discovered in v2.0.0!")
print("   AI assist feature corrupting user data")

-- Show backup options
print("\n📋 Available recovery points:")
if has_backup_api then
    local backups = State.list_backups()
    if backups then
        for i, backup in ipairs(backups) do
            print(string.format("   %d. %s - %s backup",
                i,
                backup.id,
                backup.incremental and "Incremental" or "Full"
            ))
        end
    end
else
    print("   1. backup:day1 - Full backup (v1.0.0)")
    print("   2. backup:day2 - Incremental backup (v1.1.0)")
    print("   3. [Current state] - v2.0.0 with issues")
end

-- Restore to day 2 state
print("\n🔄 Restoring to Day 2 state (v1.1.0)...")
if has_backup_api then
    local restore_result = State.restore_backup("day2_backup_id")
    if restore_result and restore_result.success then
        print("✅ Successfully restored to stable version")
    else
        print("❌ Restore failed: " .. (restore_result and restore_result.error or "unknown"))
    end
else
    -- Manual restore simulation
    local backup_data = State.load("system", "backup:day2")
    if backup_data then
        for key, value in pairs(backup_data) do
            State.save("global", key, value)
        end
        print("✅ Successfully restored to stable version")
    end
end
display_state("Restored state (back to v1.1.0)", "global", app_keys)

-- Scenario 2: Multi-scope backup pattern
print("\n\n🎯 Scenario 2: Multi-Scope Backup Pattern")
print("=========================================")

-- Define scope configurations
local scope_configs = {
    user = {"profile", "session_count"},
    system = {"config", "maintenance_mode"},
    cache = {"recent_searches", "last_updated"}
}

-- Set up multi-component state
print("\nSetting up multi-component application state...")
State.save("user", "profile", {
    name = "John Doe",
    email = "john@example.com",
    preferences = { theme = "light", notifications = true }
})
State.save("user", "session_count", 5)

State.save("system", "config", {
    api_endpoint = "https://api.example.com",
    timeout = 30,
    retry_count = 3
})
State.save("system", "maintenance_mode", false)

State.save("cache", "recent_searches", {"rust", "lua", "backup"})
State.save("cache", "last_updated", os.time())

-- Display all scopes
for scope, keys in pairs(scope_configs) do
    display_state("Scope: " .. scope, scope, keys)
end

-- Create multi-scope backup
print("\n  Creating full system backup...")
local backup_manifest = {
    timestamp = os.time(),
    scopes = {}
}

for scope, keys in pairs(scope_configs) do
    backup_manifest.scopes[scope] = {}
    for _, key in ipairs(keys) do
        backup_manifest.scopes[scope][key] = State.load(scope, key)
    end
end

State.save("system", "backup:full_system", backup_manifest)
print("  ✅ Full system backup created")

-- Simulate partial data corruption
print("\n💥 Simulating cache corruption and user data loss...")
State.delete("cache", "recent_searches")
State.save("cache", "last_updated", -1) -- Invalid timestamp
State.delete("user", "profile")

print("\nState after corruption:")
display_state("User scope", "user", scope_configs.user)
display_state("Cache scope", "cache", scope_configs.cache)

-- Restore from backup
print("\n🔄 Restoring from backup...")
local saved_backup = State.load("system", "backup:full_system")
if saved_backup and saved_backup.scopes then
    for scope, data in pairs(saved_backup.scopes) do
        for key, value in pairs(data) do
            State.save(scope, key, value)
        end
    end
    print("✅ Full restoration completed")
end

-- Verify restoration
print("\nRestored state:")
display_state("User scope", "user", scope_configs.user)
display_state("Cache scope", "cache", scope_configs.cache)

-- Scenario 3: Backup metadata and validation
print("\n\n🔗 Scenario 3: Backup Metadata Pattern")
print("======================================")

-- Create backup with metadata
print("\nCreating backup with metadata...")
local backup_with_meta = {
    id = "backup_" .. os.time(),
    created = os.time(),
    type = "manual",
    description = "Pre-deployment backup",
    state = {},
    checksum = 0
}

-- Backup current state with checksum
local checksum = 0
for _, key in ipairs(app_keys) do
    local value = State.load("global", key)
    if value ~= nil then
        backup_with_meta.state[key] = value
        -- Simple checksum calculation (in real implementation, use proper hash)
        checksum = checksum + string.len(tostring(value))
    end
end
backup_with_meta.checksum = checksum

State.save("system", "backup:with_metadata", backup_with_meta)
print("  ✅ Backup created with metadata")
print("     ID: " .. backup_with_meta.id)
print("     Checksum: " .. backup_with_meta.checksum)

-- Validate backup integrity
print("\n🔍 Validating backup integrity...")
local stored_backup = State.load("system", "backup:with_metadata")
if stored_backup then
    local validation_checksum = 0
    for key, value in pairs(stored_backup.state) do
        validation_checksum = validation_checksum + string.len(tostring(value))
    end
    
    if validation_checksum == stored_backup.checksum then
        print("  ✅ Backup validation passed")
    else
        print("  ❌ Backup validation failed!")
        print("     Expected: " .. stored_backup.checksum)
        print("     Actual: " .. validation_checksum)
    end
end

-- Summary
print("\n\n🎉 Recovery Scenarios Completed!")
print("================================")
print("\nDemonstrated patterns:")
print("  ✓ Time-based backup and recovery")
print("  ✓ Multi-scope state management")
print("  ✓ Backup metadata and validation")
print("  ✓ Manual restore procedures")
print("\nKey takeaways:")
print("  • Track your state keys for reliable backup")
print("  • Use metadata for backup management")
print("  • Implement validation for data integrity")
print("  • Plan recovery strategies for different scenarios")
if not has_backup_api then
    print("\nNote: Enable backup manager in StateGlobal for full backup API")
end