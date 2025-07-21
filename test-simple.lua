-- Simple test
print("=== Simple Provider Test ===")

-- Check if API is available
print("Agent API available:", Agent ~= nil)
print("Tool API available:", Tool ~= nil)

-- Try to get available providers
if Provider and Provider.list then
    print("\nAvailable providers:")
    local providers = Provider.list()
    for _, p in ipairs(providers) do
        print("  - " .. p)
    end
end

print("\nTest complete")