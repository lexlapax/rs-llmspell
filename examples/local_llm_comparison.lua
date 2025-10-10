#!/usr/bin/env llmspell

-- ============================================================
-- LLMSPELL LOCAL LLM EXAMPLES
-- ============================================================
-- Example: Backend Performance Comparison
-- Complexity Level: INTERMEDIATE
-- Real-World Use Case: Compare Ollama vs Candle performance and quality
-- Category: Local LLM Integration
--
-- Purpose: Side-by-side comparison of Ollama and Candle backends
-- Architecture: Dual agents with different backends + performance timing
-- Key Features:
--   • Create agents for both Ollama and Candle
--   • Compare response quality across backends
--   • Measure inference latency for each backend
--   • Automatic model selection from available models
--
-- Prerequisites:
--   • LLMSpell installed and built
--   • BOTH backends required:
--     - Ollama installed and running with at least one model
--       (Install: ollama pull llama3.1:8b)
--     - Candle with at least one model downloaded
--       (Install: llmspell model pull tinyllama:Q4_K_M@candle)
--
-- HOW TO RUN:
-- # Requires both Ollama and Candle backends configured:
-- ./target/debug/llmspell -p development \
--   run examples/local_llm_comparison.lua
--
-- # Alternative: Use custom config with both backends:
-- ./target/debug/llmspell -c path/to/dual-backend-config.toml \
--   run examples/local_llm_comparison.lua
--
-- EXPECTED OUTPUT:
-- Three test prompts with responses from both backends and timing data
-- Execution time: 30-60 seconds (depends on model sizes)
--
-- Note:
-- This example requires both backends to be available. The script will
-- automatically select the first available model from each backend.
--
-- Next Steps:
-- - See local_llm_chat.lua for single-backend interactive chat
-- - See cookbook/performance-monitoring.lua for advanced metrics
-- ============================================================

-- Check if both backends are available
local status = LocalLLM.status()

if not status.ollama.running or status.ollama.models == 0 then
    print("Error: Ollama backend not available or no models")
    if status.ollama.error then
        print("Ollama error: " .. status.ollama.error)
    end
    print("Run: ollama pull llama3.1:8b")
    os.exit(1)
end

if not status.candle.ready or status.candle.models == 0 then
    print("Error: Candle backend not ready or no models")
    if status.candle.error then
        print("Candle error: " .. status.candle.error)
    end
    print("Run: llmspell model pull tinyllama:Q4_K_M@candle")
    os.exit(1)
end

-- Get first model from each backend
local ollama_models = LocalLLM.list("ollama")
local candle_models = LocalLLM.list("candle")

local ollama_model = "local/" .. ollama_models[1].id .. "@ollama"
local candle_model = "local/" .. candle_models[1].id .. "@candle"

print("=== Backend Comparison ===\n")
print("Ollama model: " .. ollama_model)
print("Candle model: " .. candle_model)
print()

-- Create agents for both backends
local ollama_agent = Agent.create({ model = ollama_model })
local candle_agent = Agent.create({ model = candle_model })

-- Test prompts
local prompts = {
    "Explain async/await in JavaScript in one sentence.",
    "What is the difference between a stack and a heap?",
    "Write a haiku about programming."
}

for i, prompt in ipairs(prompts) do
    print("=== Prompt " .. i .. " ===")
    print("Q: " .. prompt)
    print()

    -- Ollama response
    print("[Ollama]")
    local ollama_start = os.clock()
    local ollama_response = ollama_agent:complete(prompt)
    local ollama_time = os.clock() - ollama_start
    print(ollama_response)
    print(string.format("(%.2fs)\n", ollama_time))

    -- Candle response
    print("[Candle]")
    local candle_start = os.clock()
    local candle_response = candle_agent:complete(prompt)
    local candle_time = os.clock() - candle_start
    print(candle_response)
    print(string.format("(%.2fs)\n", candle_time))

    print(string.rep("-", 60))
    print()
end
