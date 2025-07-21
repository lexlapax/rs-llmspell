print("=== Testing Base URL Override Scenario ===\n")

-- Test base URL override parsing works
local success, err = pcall(function()
    return Agent.create({
        model = "openai/gpt-3.5-turbo",
        base_url = "http://localhost:8080/v1",
        prompt = "You are a test assistant"
    })
end)

print("pcall result:")
print("  success: " .. tostring(success))
print("  error: " .. tostring(err))

-- Should fail with provider error (openai not configured)
assert(not success, "Should fail with unconfigured provider")
assert(err, "Should have error message")

-- The error should mention provider, not syntax issue
local error_str = tostring(err)
print("\nChecking error message...")
print("  Error string: " .. error_str)
print("  Contains 'provider': " .. tostring(error_str:find("provider") ~= nil))
print("  Contains 'Unknown': " .. tostring(error_str:find("Unknown") ~= nil))

local has_provider_error = error_str:find("provider") or error_str:find("Unknown")
assert(has_provider_error, "Error should be about provider configuration: " .. error_str)

print("\n=== Test Complete - All assertions passed ===")