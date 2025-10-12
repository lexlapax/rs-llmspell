#!/usr/bin/env llmspell

-- ============================================================
-- LLMSPELL LOCAL LLM EXAMPLES
-- ============================================================
-- Example: Local LLM Status
-- Complexity Level: BEGINNER
-- Real-World Use Case: Check status of local LLM backends
-- Category: Local LLM Integration
--
-- Purpose: Demonstrate LocalLLM.status() and LocalLLM.list() APIs
--          for checking backend availability and listing models
-- Architecture: Direct LocalLLM API usage without agent creation
-- Key Features:
--   • Check Ollama backend status
--   • Check Candle backend status
--   • List all available local models
--   • Display model metadata (size, quantization, modified date)
--
-- Prerequisites:
--   • LLMSpell installed and built
--   • Optional: Ollama installed and running
--   • Optional: Candle models downloaded via llmspell model pull
--
-- HOW TO RUN:
-- ./target/debug/llmspell -p minimal run examples/local_llm_status.lua
--
-- EXPECTED OUTPUT:
-- Status of Ollama and Candle backends with list of available models
-- Execution time: <1 second
--
-- Next Steps:
-- - To add Ollama models: ollama pull llama3.1:8b
-- - To add Candle models: llmspell model pull tinyllama:Q4_K_M@candle
-- - See local_llm_chat.lua for interactive chat with local models
-- ============================================================

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
