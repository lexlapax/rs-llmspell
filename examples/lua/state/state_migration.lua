-- ABOUTME: State migration example showing schema evolution
-- ABOUTME: Demonstrates how to handle state structure changes over time

-- CONFIG: Use examples/configs/migration-enabled.toml (or state-enabled.toml for manual patterns)
-- WHY: Migration API (State.migrate) requires migration_enabled=true in config
-- HOW TO RUN: ./target/debug/llmspell -c examples/configs/migration-enabled.toml run examples/lua/state/state_migration.lua
-- ALTERNATIVE: cargo run -- -c examples/configs/migration-enabled.toml run examples/lua/state/state_migration.lua
-- NOTE: Without migration-enabled.toml, only manual migration patterns will be demonstrated

print("🔄 State Migration Example")
print("=====================================")

-- IMPORTANT: Migration features require StateGlobal to be initialized with
-- migration support. If these functions are not available, the example
-- will demonstrate the patterns you would use.

-- Check if migration API is available
local has_migration = State.migrate ~= nil

if not has_migration then
    print("\n⚠️  Migration API not available in this configuration")
    print("   The State global needs to be initialized with migration support")
    print("   This example will demonstrate the migration patterns\n")
end

-- 1. Simulating schema versions with state
print("\n1. Setting up versioned state...")

-- Simulate v1.0.0 schema
State.save("global", "schema_version", "1.0.0")
State.save("global", "user_data", {
    name = "John Doe",
    email = "john@example.com",
    created = os.time()
})
State.save("global", "app_config", {
    theme = "light",
    notifications = true
})

print("   Created state with v1.0.0 schema")

-- 2. Check current schema if migration is available
if has_migration then
    print("\n2. Checking migration status...")
    
    -- Get available schema versions
    local versions = State.schema_versions()
    if versions then
        print("   Available schema versions:")
        for i, version in ipairs(versions) do
            print("   - " .. version)
        end
    end
else
    print("\n2. Manual migration pattern (when API not available)...")
end

-- 3. Demonstrate migration patterns
print("\n3. Migration patterns...")

-- Pattern 1: Field addition
local function migrate_add_field()
    print("\n   Pattern 1: Adding new fields")
    local user = State.load("global", "user_data") or {}
    
    -- Add new fields with defaults
    if not user.preferences then
        user.preferences = {
            language = "en",
            timezone = "UTC"
        }
    end
    
    State.save("global", "user_data", user)
    print("   ✅ Added preferences to user_data")
end

-- Pattern 2: Field renaming
local function migrate_rename_field()
    print("\n   Pattern 2: Renaming fields")
    local config = State.load("global", "app_config") or {}
    
    -- Rename 'theme' to 'ui_theme'
    if config.theme and not config.ui_theme then
        config.ui_theme = config.theme
        config.theme = nil  -- Remove old field
    end
    
    State.save("global", "app_config", config)
    print("   ✅ Renamed theme to ui_theme")
end

-- Pattern 3: Data structure change
local function migrate_structure_change()
    print("\n   Pattern 3: Changing data structure")
    local user = State.load("global", "user_data") or {}
    
    -- Convert flat structure to nested
    if user.name and not user.profile then
        user.profile = {
            display_name = user.name,
            email = user.email,
            created = user.created
        }
        -- Remove old fields
        user.name = nil
        user.email = nil
        user.created = nil
    end
    
    State.save("global", "user_data", user)
    print("   ✅ Converted flat user data to nested structure")
end

-- Pattern 4: Data type conversion
local function migrate_type_conversion()
    print("\n   Pattern 4: Converting data types")
    local config = State.load("global", "app_config") or {}
    
    -- Convert boolean to string enum
    if type(config.notifications) == "boolean" then
        config.notification_level = config.notifications and "all" or "none"
        config.notifications = nil
    end
    
    State.save("global", "app_config", config)
    print("   ✅ Converted boolean to enum type")
end

-- Execute manual migrations
migrate_add_field()
migrate_rename_field()
migrate_structure_change()
migrate_type_conversion()

-- Update schema version after manual migration
State.save("global", "schema_version", "2.0.0")
print("\n   Manual migration completed to v2.0.0")

-- 4. Using the migration API (if available)
if has_migration then
    print("\n4. Using State.migrate() API...")
    
    -- Attempt to migrate to a specific version
    local target_version = "2.1.0"
    print("   Attempting migration to " .. target_version)
    
    local result = State.migrate(target_version)
    
    if result.success then
        print("   ✅ Migration successful!")
        print("   - From version: " .. result.from_version)
        print("   - To version: " .. result.to_version)
        print("   - Items migrated: " .. result.items_migrated)
        print("   - Duration: " .. result.duration_ms .. "ms")
        
        if result.warnings and #result.warnings > 0 then
            print("   ⚠️  Warnings:")
            for _, warning in ipairs(result.warnings) do
                print("     - " .. warning)
            end
        end
    else
        print("   ❌ Migration failed: " .. (result.error or "Unknown error"))
    end
else
    print("\n4. Migration verification...")
end

-- 5. Safe migration patterns
print("\n5. Safe migration patterns...")

-- Pattern: Version check before operations
local function safe_load_user()
    local version = State.load("global", "schema_version") or "1.0.0"
    local user = State.load("global", "user_data") or {}
    
    -- Handle different versions
    if version == "1.0.0" then
        -- Old flat structure
        return {
            display_name = user.name,
            email = user.email
        }
    elseif version >= "2.0.0" then
        -- New nested structure
        return {
            display_name = user.profile and user.profile.display_name,
            email = user.profile and user.profile.email
        }
    end
end

local user_info = safe_load_user()
print("   Loaded user safely across versions:")
print("   - Name: " .. (user_info.display_name or "unknown"))
print("   - Email: " .. (user_info.email or "unknown"))

-- 6. Migration rollback pattern
print("\n6. Migration rollback pattern...")

-- Save backup before migration
local function backup_before_migration()
    local backup_data = {}
    -- Define keys to backup
    local backup_keys = {
        "schema_version",
        "user_data",
        "app_config"
    }
    
    for _, key in ipairs(backup_keys) do
        local value = State.load("global", key)
        if value ~= nil then
            backup_data[key] = value
        end
    end
    
    State.save("system", "migration_backup", backup_data)
    State.save("system", "backup_version", State.load("global", "schema_version"))
    
    local count = 0
    for _ in pairs(backup_data) do count = count + 1 end
    print("   Created migration backup with " .. count .. " entries")
end

-- Restore from backup
local function rollback_migration()
    local backup = State.load("system", "migration_backup")
    local backup_version = State.load("system", "backup_version")
    
    if backup then
        -- Clear current state (using same key list)
        local clear_keys = {
            "schema_version",
            "user_data",
            "app_config"
        }
        for _, key in ipairs(clear_keys) do
            State.delete("global", key)
        end
        
        -- Restore backup
        for key, value in pairs(backup) do
            State.save("global", key, value)
        end
        
        print("   Rolled back to version " .. (backup_version or "unknown"))
    else
        print("   No backup available for rollback")
    end
end

-- Utility function (needed before use)
local function print_table(t, indent)
    indent = indent or ""
    for k, v in pairs(t) do
        if type(v) == "table" then
            print(indent .. k .. ":")
            print_table(v, indent .. "  ")
        else
            print(indent .. k .. " = " .. tostring(v))
        end
    end
end

-- Demonstrate backup/rollback
backup_before_migration()
-- Could perform migration here
-- rollback_migration()  -- Uncomment to test rollback

-- 7. Display final migrated state
print("\n7. Final migrated state...")
print("   Schema version: " .. (State.load("global", "schema_version") or "unknown"))

local final_user = State.load("global", "user_data")
if final_user then
    print("   User data structure:")
    print_table(final_user, "   ")
end

local final_config = State.load("global", "app_config")
if final_config then
    print("   App config structure:")
    print_table(final_config, "   ")
end

-- Remove duplicate function definition

print("\n✅ State migration example completed!")
print("\nKey migration patterns:")
print("- Always version your state schema")
print("- Add new fields with sensible defaults")
print("- Handle field renames carefully")
print("- Support loading data from multiple versions")
print("- Create backups before major migrations")
print("- Test migrations thoroughly before production")
if has_migration then
    print("- Use State.migrate() when available for automated migrations")
else
    print("- Enable migration support in StateGlobal for automated migrations")
end