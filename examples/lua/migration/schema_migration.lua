-- ABOUTME: Example demonstrating state schema migration from Lua scripts
-- ABOUTME: Shows how scripts can trigger migrations and monitor progress

print("🚀 rs-llmspell Lua Migration Example")
print("====================================")

-- First, let's check the current migration status
print("\n1. Checking current migration status...")
local status = State.migration_status()

if status.migration_available then
    print("✅ Migration system available")
    print("   Current version: " .. status.current_version)
    
    if status.latest_version then
        print("   Latest version:  " .. status.latest_version)
        print("   Is latest:       " .. tostring(status.is_latest))
    end
    
    print("   Total schemas:   " .. status.total_schemas)
    
    -- List migration targets
    if status.migration_targets and #status.migration_targets > 0 then
        print("   Migration targets available:")
        for i, version in ipairs(status.migration_targets) do
            print("     - " .. version)
        end
    else
        print("   No migration targets available")
    end
else
    print("❌ Migration system not available")
    if status.error then
        print("   Error: " .. status.error)
    end
    return
end

-- List all available schema versions
print("\n2. Available schema versions:")
local versions = State.schema_versions()
for i, version in ipairs(versions) do
    print("   " .. i .. ". " .. version)
end

-- Example: Try to migrate to a specific version (if available)
print("\n3. Migration example...")

-- Get available migration targets
local targets = State.migration_status().migration_targets or {}

if #targets > 0 then
    local target_version = targets[1] -- Use first available target
    print("   Attempting migration to: " .. target_version)
    
    -- Trigger migration
    local result = State.migrate(target_version)
    
    if result.success then
        print("✅ Migration successful!")
        print("   Status:         " .. result.status)
        print("   From version:   " .. result.from_version)
        print("   To version:     " .. result.to_version)
        print("   Items migrated: " .. result.items_migrated)
        print("   Duration:       " .. result.duration_ms .. "ms")
        
        -- Show warnings if any
        if result.warnings and #result.warnings > 0 then
            print("   Warnings:")
            for i, warning in ipairs(result.warnings) do
                print("     ⚠️  " .. warning)
            end
        end
        
        -- Show errors if any
        if result.errors and #result.errors > 0 then
            print("   Errors:")
            for i, error in ipairs(result.errors) do
                print("     ❌ " .. error)
            end
        end
    else
        print("❌ Migration failed!")
        print("   Error: " .. (result.error or "Unknown error"))
        if result.from_version then
            print("   From: " .. result.from_version)
        end
        if result.target_version then
            print("   To:   " .. result.target_version)
        end
    end
else
    print("   No migration targets available - trying to migrate to same version")
    local current_version = State.migration_status().current_version
    local result = State.migrate(current_version)
    
    if result.success and result.status == "already_current" then
        print("✅ Already at current version: " .. result.current_version)
    else
        print("❌ Unexpected result: " .. (result.error or "Unknown"))
    end
end

-- Demonstrate state operations with migration context
print("\n4. State operations with migration...")

-- Store some test data
State.save("global", "test_migration_key", {
    message = "Hello from migrated state!",
    timestamp = os.time(),
    version = State.migration_status().current_version
})

-- Load and verify the data
local test_data = State.load("global", "test_migration_key")
if test_data then
    print("✅ Test data stored successfully:")
    print("   Message:   " .. test_data.message)
    print("   Timestamp: " .. test_data.timestamp)
    print("   Version:   " .. test_data.version)
else
    print("❌ Failed to load test data")
end

-- Final status check
print("\n5. Final migration status check...")
local final_status = State.migration_status()
print("   Current version: " .. final_status.current_version)
print("   Migration available: " .. tostring(final_status.migration_available))

print("\n🎉 Migration example completed!")
print("\nThis example demonstrates:")
print("  • Checking migration system availability")
print("  • Getting current schema version and migration status") 
print("  • Listing available schema versions and migration targets")
print("  • Triggering schema migrations from Lua")
print("  • Handling migration results and error conditions")
print("  • Using state operations in migration context")