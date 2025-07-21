-- Test agent creation
print("=== Testing Agent Creation ===")

-- Try different provider specifications
local tests = {
    {
        name = "With provider field",
        config = {
            name = "test1",
            provider = "openai",
            model = "gpt-4"
        }
    },
    {
        name = "With provider_model field",
        config = {
            name = "test2",
            provider_model = "openai/gpt-4"
        }
    },
    {
        name = "With model specifier",
        config = {
            name = "test3",
            model = "openai/gpt-4"
        }
    }
}

for _, test in ipairs(tests) do
    print("\n" .. test.name .. ":")
    local success, result = pcall(function()
        return Agent.create(test.config)
    end)
    
    if success then
        print("  SUCCESS - Agent created")
        if result and result.id then
            print("  Agent ID: " .. result.id)
        end
    else
        print("  FAILED: " .. tostring(result))
    end
end

print("\nTest complete")