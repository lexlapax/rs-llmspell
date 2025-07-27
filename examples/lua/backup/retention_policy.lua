#!/usr/bin/env llmspell

-- Backup Retention Policy Example
-- Demonstrates backup storage management and retention policies

print("=== Backup Retention Policy Demo ===\n")

-- First, create some test data and backups
print("1. Creating test data and backups...")

-- Add some state data for each backup
for i = 1, 5 do
    State.set(State.Scope.Global, "backup_test_" .. i, {
        iteration = i,
        data = string.rep("test_data_", 100), -- Create some bulk
        timestamp = os.time()
    })
    
    -- Create a backup
    local result = State.create_backup(false) -- full backup
    if result.success then
        print(string.format("   Created backup #%d: %s", i, result.backup_id or "unknown"))
    else
        print(string.format("   ❌ Failed to create backup #%d: %s", i, result.error))
    end
    
    -- Small delay between backups
    os.execute("sleep 0.1")
end

-- Check storage usage
print("\n2. Checking backup storage usage...")
local usage = State.get_storage_usage()
print(string.format("   Total backups: %d", usage.total_backups))
print(string.format("   Total size: %d bytes", usage.total_size))
print(string.format("   Full backups: %d", usage.full_backups))
print(string.format("   Incremental backups: %d", usage.incremental_backups))

-- List current backups
print("\n3. Current backup list:")
local backups = State.list_backups()
if #backups > 0 then
    for i, backup in ipairs(backups) do
        print(string.format("   %d. %s - %s (%d bytes)", 
            i, 
            backup.id, 
            backup.is_incremental and "incremental" or "full",
            backup.size_bytes
        ))
    end
else
    print("   No backups found (backup functionality may not be implemented)")
end

-- Perform dry-run cleanup
print("\n4. Running cleanup simulation (dry run)...")
local dry_run_result = State.cleanup_backups(true) -- dry_run = true

if dry_run_result.success then
    print("   Cleanup simulation results:")
    print(string.format("   - Would delete: %d backups", dry_run_result.deleted_count))
    print(string.format("   - Would retain: %d backups", dry_run_result.retained_count))
    print(string.format("   - Would free: %d bytes", dry_run_result.space_freed))
else
    print(string.format("   ❌ Cleanup simulation failed: %s", dry_run_result.error))
end

-- Ask user before actual cleanup
print("\n5. Ready to perform actual cleanup?")
print("   This will delete old backups according to retention policies.")
print("   Press Enter to continue or Ctrl+C to cancel...")
io.read()

-- Perform actual cleanup
print("\n6. Running actual cleanup...")
local cleanup_result = State.cleanup_backups(false) -- dry_run = false

if cleanup_result.success then
    print("   ✅ Cleanup completed successfully!")
    print(string.format("   - Deleted: %d backups", cleanup_result.deleted_count))
    print(string.format("   - Retained: %d backups", cleanup_result.retained_count))
    print(string.format("   - Freed: %d bytes", cleanup_result.space_freed))
else
    print(string.format("   ❌ Cleanup failed: %s", cleanup_result.error))
end

-- Final storage check
print("\n7. Final storage usage:")
local final_usage = State.get_storage_usage()
print(string.format("   Total backups: %d", final_usage.total_backups))
print(string.format("   Total size: %d bytes", final_usage.total_size))

-- List remaining backups
print("\n8. Remaining backups:")
local final_backups = State.list_backups()
if #final_backups > 0 then
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

print("\n=== Demo Complete ===")

-- Note: This example demonstrates the retention policy API.
-- In a real implementation, retention policies would be configured
-- in the backup configuration (max_backups, max_backup_age, etc.)
-- and applied automatically during backup creation or via scheduled cleanup.