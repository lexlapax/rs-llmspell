-- provider-info.lua
-- Demonstrates provider API usage (Phase 1 - listing providers)

print("=== LLMSpell Provider Information ===\n")

-- Get list of available providers
local providers = Provider.list()

print(string.format("Found %d provider(s)\n", #providers))

-- Display detailed information about each provider
for i, provider in ipairs(providers) do
    print(string.format("Provider #%d: %s", i, provider.name))
    
    -- Check capabilities if available
    if provider.capabilities then
        local caps = provider.capabilities
        print("  Capabilities:")
        print("    - Streaming: " .. tostring(caps.supports_streaming))
        print("    - Multimodal: " .. tostring(caps.supports_multimodal))
        
        -- Show available models if present
        if caps.available_models and #caps.available_models > 0 then
            print("    - Available models:")
            for _, model in ipairs(caps.available_models) do
                print("      * " .. model)
            end
        end
        
        -- Show context window if available
        if caps.max_context_tokens then
            print("    - Max context tokens: " .. caps.max_context_tokens)
        end
    end
    
    print() -- Empty line between providers
end

-- Return summary information
return {
    provider_count = #providers,
    providers = providers,
    timestamp = os.date("%Y-%m-%d %H:%M:%S")
}