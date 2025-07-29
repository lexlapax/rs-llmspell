-- ABOUTME: Basic test to verify Replay global is available
-- ABOUTME: Checks that the Replay API is properly injected

-- Test 1: Check if Replay global exists
print("=== Testing Replay Global ===")
print("Replay global exists:", Replay ~= nil)

if Replay == nil then
    error("Replay global is not available!")
end

-- Test 2: Check if modes are available
print("\n=== Testing Replay Modes ===")
print("Replay.modes exists:", Replay.modes ~= nil)

if Replay.modes then
    print("Exact mode:", Replay.modes.exact ~= nil)
    print("Modified mode:", Replay.modes.modified ~= nil)
    print("Simulate mode:", Replay.modes.simulate ~= nil)
    print("Debug mode:", Replay.modes.debug ~= nil)
end

-- Test 3: Check if functions are available
print("\n=== Testing Replay Functions ===")
print("create_config function:", type(Replay.create_config))
print("create_modification function:", type(Replay.create_modification))
print("create_comparator function:", type(Replay.create_comparator))

-- Test 4: Try creating a simple config
print("\n=== Testing Config Creation ===")
local ok, result = pcall(function()
    local config = Replay.create_config()
    return config
end)
print("Config creation successful:", ok)
if not ok then
    print("Error:", result)
end

print("\nBasic Replay test completed!")