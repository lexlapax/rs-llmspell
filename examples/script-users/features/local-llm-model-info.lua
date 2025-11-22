-- ============================================================================
-- Example: Local LLM Model Information
-- Category: features
-- Phase: 13c
-- ============================================================================
--
-- Description:
--   Get detailed model information and test inference with local models.
--   Demonstrates LocalLLM.info() API and agent creation.
--
-- Prerequisites:
--   - At least one local model:
--     - Ollama: ollama pull llama3.1:8b
--     - Candle: llmspell model pull tinyllama:Q4_K_M@candle
--
-- Profile: ollama
-- Runtime: ~10 seconds
-- Complexity: BEGINNER
--
-- Usage:
--   # Ollama models
--   llmspell -p ollama run examples/script-users/features/local-llm-model-info.lua llama3.1:8b@ollama
--
--   # Candle models
--   llmspell -p candle run examples/script-users/features/local-llm-model-info.lua tinyllama:Q4_K_M@candle
--
-- Expected Output:
--   Model details (size, architecture, quantization) followed by test inference
--
-- Arguments:
--   MODEL_SPEC - Model specification (e.g., "llama3.1:8b@ollama")
--
-- ============================================================================

-- Get model spec from command line argument or environment
local model_spec = arg[1] or os.getenv("MODEL_SPEC")

if not model_spec then
    print("Usage: llmspell run examples/local_llm_model_info.lua MODEL_SPEC")
    print()
    print("Examples:")
    print("  llmspell run examples/local_llm_model_info.lua llama3.1:8b@ollama")
    print("  llmspell run examples/local_llm_model_info.lua tinyllama:Q4_K_M@candle")
    print()
    print("Available models:")
    local models = LocalLLM.list()
    for _, model in ipairs(models) do
        print("  " .. model.id .. "@" .. model.backend)
    end
    os.exit(1)
end

print("=== Model Information ===\n")
print("Model: " .. model_spec)
print()

-- Get model information
local info = LocalLLM.info(model_spec)

-- Display information
print("Details:")
print("  ID: " .. info.id)
print("  Backend: " .. info.backend)
print("  Format: " .. info.format)

local size_mb = math.floor(info.size_bytes / 1024 / 1024)
local size_gb = info.size_bytes / 1024 / 1024 / 1024
if size_gb >= 1.0 then
    print(string.format("  Size: %.2f GB (%d bytes)", size_gb, info.size_bytes))
else
    print(string.format("  Size: %d MB (%d bytes)", size_mb, info.size_bytes))
end

if info.quantization then
    print("  Quantization: " .. info.quantization)
end

if info.parameter_count then
    print("  Parameters: " .. info.parameter_count)
end

if info.context_length then
    print("  Context Length: " .. info.context_length)
end

if info.architecture then
    print("  Architecture: " .. info.architecture)
end

if info.modified_at then
    print("  Modified: " .. os.date("%Y-%m-%d %H:%M:%S", info.modified_at))
end

if info.path then
    print("  Path: " .. info.path)
end

-- Test inference
print()
print("=== Test Inference ===")
print("Prompt: What is Rust?")
print()

local agent = Agent.create({ model = "local/" .. model_spec })
local response = agent:complete("What is Rust? Answer in one sentence.")

print("Response: " .. response)
