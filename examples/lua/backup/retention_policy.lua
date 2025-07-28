-- ABOUTME: Backup retention policy demonstration
-- ABOUTME: Shows how to manage backup storage with retention policies

-- CONFIG: Use examples/configs/backup-enabled.toml
-- WHY: Retention policies require backup manager with cleanup_backups functionality
-- HOW TO RUN: ./target/debug/llmspell -c examples/configs/backup-enabled.toml run examples/lua/backup/retention_policy.lua
-- ALTERNATIVE: cargo run -- -c examples/configs/backup-enabled.toml run examples/lua/backup/retention_policy.lua
-- NOTE: This version uses correct State API (save/load/delete)

-- Backup Retention Policy Example
-- Demonstrates backup storage management and retention policies

print("=== Backup Retention Policy Demo ===\n")

-- Check if backup API is available
local has_backup_api = State.create_backup ~= nil
local has_storage_api = State.get_storage_usage ~= nil
local has_cleanup_api = State.cleanup_backups ~= nil

if not has_backup_api then
    print("⚠️  Backup API not available in this configuration")
    print("   Make sure to use backup-enabled.toml config")
    print("   This example will demonstrate patterns without actual backup calls\n")
end

-- First, create some test data and backups
print("1. Creating test data and backups...")

-- Add some state data for each backup
for i = 1, 5 do
    State.save("global", "backup_test_" .. i, {
        iteration = i,
        data = string.rep("test_data_", 100), -- Create some bulk
        timestamp = os.time()
    })
    
    -- Create a backup
    if has_backup_api then
        local result = State.create_backup(false) -- full backup
        if result and result.success then
            print(string.format("   Created backup #%d: %s", i, result.backup_id or "unknown"))
        else
            print(string.format("   ❌ Failed to create backup #%d: %s", i, result and result.error or "unknown"))
        end
    else
        print(string.format("   Simulated backup #%d", i))
    end
    
    -- Small delay between backups
    os.execute("sleep 0.1")
end

-- Check storage usage
print("\n2. Checking backup storage usage...")
if has_storage_api then
    local usage = State.get_storage_usage()
    if usage then
        print(string.format("   Total backups: %d", usage.total_backups or 0))
        print(string.format("   Total size: %d bytes", usage.total_size or 0))
        print(string.format("   Full backups: %d", usage.full_backups or 0))
        print(string.format("   Incremental backups: %d", usage.incremental_backups or 0))
    else
        print("   Storage usage API returned no data")
    end
else
    print("   Storage usage API not available")
end

-- List current backups
print("\n3. Current backup list:")
if has_backup_api and State.list_backups then
    local backups = State.list_backups()
    if backups and #backups > 0 then
        for i, backup in ipairs(backups) do
            print(string.format("   %d. %s - %s (%d bytes)", 
                i, 
                backup.id, 
                backup.is_incremental and "incremental" or "full",
                backup.size_bytes or 0
            ))
        end
    else
        print("   No backups found")
    end
else
    print("   Backup listing API not available")
end

-- Perform dry-run cleanup
print("\n4. Running cleanup simulation (dry run)...")
if has_cleanup_api then
    local dry_run_result = State.cleanup_backups(true) -- dry_run = true
    
    if dry_run_result and dry_run_result.success then
        print("   Cleanup simulation results:")
        print(string.format("   - Would delete: %d backups", dry_run_result.deleted_count or 0))
        print(string.format("   - Would retain: %d backups", dry_run_result.retained_count or 0))
        print(string.format("   - Would free: %d bytes", dry_run_result.space_freed or 0))
    else
        print(string.format("   ❌ Cleanup simulation failed: %s", dry_run_result and dry_run_result.error or "unknown"))
    end
else
    print("   Cleanup API not available")
end

-- Ask user before actual cleanup
print("\n5. Ready to perform actual cleanup?")
print("   This will delete old backups according to retention policies.")
print("   Press Enter to continue or Ctrl+C to cancel...")
io.read()

-- Perform actual cleanup
print("\n6. Running actual cleanup...")
if has_cleanup_api then
    local cleanup_result = State.cleanup_backups(false) -- dry_run = false
    
    if cleanup_result and cleanup_result.success then
        print("   ✅ Cleanup completed successfully!")
        print(string.format("   - Deleted: %d backups", cleanup_result.deleted_count or 0))
        print(string.format("   - Retained: %d backups", cleanup_result.retained_count or 0))
        print(string.format("   - Freed: %d bytes", cleanup_result.space_freed or 0))
    else
        print(string.format("   ❌ Cleanup failed: %s", cleanup_result and cleanup_result.error or "unknown"))
    end
else
    print("   Cleanup API not available")
end

-- Final storage check
print("\n7. Final storage usage:")
if has_storage_api then
    local final_usage = State.get_storage_usage()
    if final_usage then
        print(string.format("   Total backups: %d", final_usage.total_backups or 0))
        print(string.format("   Total size: %d bytes", final_usage.total_size or 0))
    end
else
    print("   Storage usage API not available")
end

-- List remaining backups
print("\n8. Remaining backups:")
if has_backup_api and State.list_backups then
    local final_backups = State.list_backups()
    if final_backups and #final_backups > 0 then
        for i, backup in ipairs(final_backups) do
            print(string.format("   %d. %s - %s", 
                i, 
                backup.id, 
                backup.created_at or "unknown time"
            ))
        end
    else
        print("   No backups remaining")
    end
else
    print("   Backup listing API not available")
end

print("\n=== Demo Complete ===")

-- Note: This example demonstrates the retention policy API.
-- In a real implementation, retention policies would be configured
-- in the backup configuration (max_backups, max_backup_age, etc.)
-- and applied automatically during backup creation or via scheduled cleanup.