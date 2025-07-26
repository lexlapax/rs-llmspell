-- ABOUTME: Example demonstrating state backup and restore functionality
-- ABOUTME: Shows how to create backups, list them, and restore from backups

print("🗄️  rs-llmspell Lua Backup Example")
print("=================================")

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
local keys = State.list_keys("global")
for i, key in ipairs(keys) do
    local value = State.load("global", key)
    print("   - " .. key .. " = " .. tostring(value))
end

-- Create a full backup
print("\n3. Creating full backup...")
local backup_result = State.create_backup(false) -- false = full backup

if backup_result.success then
    print("✅ Full backup created successfully")
    print("   Backup ID: " .. backup_result.backup_id)
    print("   Size: " .. backup_result.size_bytes .. " bytes")
    print("   Entries: " .. backup_result.entry_count)
else
    print("❌ Backup failed: " .. (backup_result.error or "Unknown error"))
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
local incremental_result = State.create_backup(true) -- true = incremental

if incremental_result.success then
    print("✅ Incremental backup created successfully")
    print("   Backup ID: " .. incremental_result.backup_id)
    print("   Parent ID: " .. incremental_result.parent_id)
    print("   Size: " .. incremental_result.size_bytes .. " bytes")
else
    print("❌ Incremental backup failed: " .. (incremental_result.error or "Unknown error"))
end

-- List available backups
print("\n6. Available backups:")
local backups = State.list_backups()

if #backups > 0 then
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

-- Validate a backup
print("\n7. Validating backup...")
if #backups > 0 then
    local backup_to_validate = backups[1].id
    local validation_result = State.validate_backup(backup_to_validate)
    
    if validation_result.is_valid then
        print("✅ Backup " .. backup_to_validate .. " is valid")
        print("   Checksum: " .. (validation_result.checksum_valid and "Valid" or "Invalid"))
        print("   Integrity: " .. (validation_result.integrity_valid and "Valid" or "Invalid"))
    else
        print("❌ Backup " .. backup_to_validate .. " is invalid")
        if validation_result.errors and #validation_result.errors > 0 then
            print("   Errors:")
            for _, err in ipairs(validation_result.errors) do
                print("     - " .. err)
            end
        end
    end
else
    print("   No backups to validate")
end

-- Simulate data loss
print("\n8. Simulating data loss...")
State.delete("global", "app_config")
State.delete("global", "user_preferences")
print("❌ Critical data deleted!")

-- Show current state
print("\n9. Current state after data loss:")
keys = State.list_keys("global")
if #keys == 0 then
    print("   No keys found - all data lost!")
else
    for i, key in ipairs(keys) do
        local value = State.load("global", key)
        print("   - " .. key .. " = " .. tostring(value))
    end
end

-- Restore from backup
print("\n10. Restoring from backup...")
if #backups > 0 then
    local backup_to_restore = backups[1].id -- Restore from most recent
    local restore_result = State.restore_backup(backup_to_restore)
    
    if restore_result.success then
        print("✅ Successfully restored from backup " .. backup_to_restore)
        
        -- Show restored state
        print("\n11. State after restoration:")
        keys = State.list_keys("global")
        for i, key in ipairs(keys) do
            local value = State.load("global", key)
            print("   - " .. key .. " = " .. tostring(value))
        end
    else
        print("❌ Restore failed: " .. (restore_result.error or "Unknown error"))
    end
else
    print("   No backups available to restore from")
end

print("\n🎉 Backup example completed!")
print("\nThis example demonstrates:")
print("  • Creating full and incremental backups")
print("  • Listing available backups")
print("  • Validating backup integrity")
print("  • Restoring state from backups")
print("  • Recovering from data loss scenarios")

-- Note about current implementation
print("\n⚠️  Note: Backup functionality is currently stubbed.")
print("   Full implementation will be available when BackupManager is integrated.")