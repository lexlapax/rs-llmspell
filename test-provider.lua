-- Test provider configuration
print("Testing provider configuration...")

-- Try to list available providers
if Provider then
    local providers = Provider.list()
    print("Available providers:")
    for _, p in ipairs(providers) do
        print("  - " .. p)
    end
else
    print("Provider API not available")
end

-- Try to create a simple agent
if Agent then
    print("\nTrying to create an agent...")
    local success, result = pcall(function()
        return Agent.create({
            name = "test_agent",
            provider = "openai",
            model = "gpt-4",
            system_prompt = "You are a test agent"
        })
    end)
    
    if success then
        print("Agent created successfully!")
    else
        print("Agent creation failed: " .. tostring(result))
    end
else
    print("Agent API not available")
end

print("\nTest complete.")