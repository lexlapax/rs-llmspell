-- Profile: minimal (recommended)
-- Run with: llmspell -p minimal run provider-info.lua
-- No LLM required

-- ============================================================
-- LLMSPELL FEATURES SHOWCASE  
-- ============================================================
-- Phase: 13c.5.6 - Example Header Standardization
-- Category: features
-- Feature ID: 05 - Provider Information v0.7.0
-- Complexity: BEGINNER
-- Real-World Use Case: Discovering available LLM providers and their capabilities
-- Feature Category: Providers
--
-- Purpose: Demonstrate Provider API for querying capabilities
-- Architecture: Provider registry with capability metadata
-- Key Capabilities:
--   • Provider.list() - Enumerate available providers
--   • Provider capabilities - Check features (streaming, multimodal)
--   • Model discovery - List available models per provider
--   • Configuration inspection - View provider settings
--
-- Prerequisites:
--   • LLMSpell installed and built
--   • Optional: API keys (OPENAI_API_KEY, ANTHROPIC_API_KEY) for full capability info
--
-- HOW TO RUN:
-- # Basic provider listing:
-- ./target/debug/llmspell run examples/script-users/features/provider-info.lua
--
-- # With full provider capabilities:
-- ./target/debug/llmspell -p providers run examples/script-users/features/provider-info.lua
--
-- EXPECTED OUTPUT:
-- Lists available providers with their capabilities and models
-- Execution time: <2 seconds
--
-- Time to Complete: 2 seconds
-- Next Steps: See agent-basics.lua for creating agents with providers
-- ============================================================

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