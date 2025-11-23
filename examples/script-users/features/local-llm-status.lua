-- Profile: -- ============================================================================ (recommended)
-- Example: Local LLM Status Check
-- Category: features

-- ============================================================
-- LLMSPELL FEATURES SHOWCASE  
-- ============================================================
-- Phase: 13c.5.6 - Example Header Standardization
-- Category: features
--   Check status and list available models for Ollama and Candle backends.
--   Demonstrates LocalLLM.status() and LocalLLM.list() APIs.
--
-- Prerequisites:
--   - LLMSpell installed and built
--   - Optional: Ollama running (ollama serve)
--   - Optional: Candle models downloaded
--
-- Profile: minimal
-- Runtime: ~1 second
-- Complexity: BEGINNER
--
-- Usage:
--   llmspell -p minimal run examples/script-users/features/local-llm-status.lua
--
-- Expected Output:
--   Status of Ollama and Candle backends with list of available models
--
-- Next Steps:
--   - Add models: ollama pull llama3.1:8b
--   - Add models: llmspell model pull tinyllama:Q4_K_M@candle
--   - See cookbook/local-llm-chat-patterns.lua for usage examples
--
-- ============================================================================

print("=== Local LLM Status ===\n")

-- Get status for all backends
local status = LocalLLM.status()

-- Check Ollama backend status
print("Ollama Backend:")
print("  Running: " .. tostring(status.ollama.running))
print("  Models: " .. status.ollama.models)
if status.ollama.error then
    print("  Error: " .. status.ollama.error)
end
if status.ollama.version then
    print("  Version: " .. status.ollama.version)
end
print()

-- Check Candle backend status
print("Candle Backend:")
print("  Ready: " .. tostring(status.candle.ready))
print("  Models: " .. status.candle.models)
if status.candle.error then
    print("  Error: " .. status.candle.error)
end
if status.candle.version then
    print("  Version: " .. status.candle.version)
end
print()

-- List all available models
print("=== Available Models ===\n")
local models = LocalLLM.list()

if #models == 0 then
    print("No models found.")
    print("\nTo get started:")
    print("  Ollama: ollama pull llama3.1:8b")
    print("  Candle: llmspell model pull tinyllama:Q4_K_M@candle")
else
    for _, model in ipairs(models) do
        print(model.id .. " (" .. model.backend .. ")")
        local size_mb = math.floor(model.size_bytes / 1024 / 1024)
        print("  Size: " .. size_mb .. " MB")

        if model.quantization then
            print("  Quantization: " .. model.quantization)
        end

        if model.modified_at then
            print("  Modified: " .. os.date("%Y-%m-%d %H:%M:%S", model.modified_at))
        end
        print()
    end

    print("Total: " .. #models .. " models")
end
