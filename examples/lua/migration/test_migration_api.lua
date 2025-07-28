-- ABOUTME: Test script to verify migration API availability
-- ABOUTME: Shows that migration APIs are properly exposed when configured

-- CONFIG: Use examples/configs/migration-enabled.toml
-- WHY: This tests if migration APIs are available (requires migration_enabled=true)
-- HOW TO RUN: ./target/debug/llmspell -c examples/configs/migration-enabled.toml run examples/lua/migration/test_migration_api.lua
-- ALTERNATIVE: cargo run -- -c examples/configs/migration-enabled.toml run examples/lua/migration/test_migration_api.lua
-- NOTE: With state-enabled.toml, migration APIs will not be available

print("🔍 Testing Migration API Availability")
print("=====================================")

-- 1. Check State global exists
print("\n1. Checking State global...")
if not State then
    print("❌ State global not found!")
    return
end
print("✅ State global exists")

-- 2. Check migration methods
print("\n2. Checking migration methods...")
local migration_methods = {
    "migrate",
    "migration_status", 
    "schema_versions"
}

local all_found = true
for _, method in ipairs(migration_methods) do
    if State[method] then
        print("✅ State." .. method .. "() - Found")
    else
        print("❌ State." .. method .. "() - NOT FOUND")
        all_found = false
    end
end

if not all_found then
    print("\n❌ Migration API not properly initialized!")
    return
end

print("\n✅ All migration APIs are available!")

-- 3. Test migration_status
print("\n3. Testing State.migration_status()...")
local status = State.migration_status and State.migration_status() or nil
print("   Returned type: " .. type(status))

if type(status) == "table" then
    print("   Status contents:")
    for k, v in pairs(status) do
        print("     - " .. k .. " = " .. tostring(v))
    end
end

-- 4. Test schema_versions
print("\n4. Testing State.schema_versions()...")
local versions = State.schema_versions and State.schema_versions() or {}
print("   Returned type: " .. type(versions))
print("   Number of schemas: " .. #versions)

if #versions == 0 then
    print("   (No schemas registered yet - this is expected)")
end

-- 5. Explanation
print("\n📝 Summary:")
print("   The migration API is properly initialized and available.")
print("   The 'No current schema found' error is expected because")
print("   no schemas have been registered yet. To use migrations,")
print("   you would need to:")
print("   1. Register schema definitions")
print("   2. Load existing state data")
print("   3. Then perform migrations between schema versions")