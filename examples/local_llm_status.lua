#!/usr/bin/env llmspell

--[[
Local LLM Status Example

Shows how to check the status of local LLM backends and list available models.

Usage:
    llmspell run examples/local_llm_status.lua

Requirements:
    - Ollama installed and running (optional)
    - OR Candle models downloaded (optional)
]]

print("=== Local LLM Status ===\n")

-- Check Ollama backend status
print("Ollama Backend:")
local ollama_status = LocalLLM.status("ollama")
print("  Health: " .. ollama_status.health)
print("  Available models: " .. ollama_status.available_models)

if ollama_status.version then
    print("  Version: " .. ollama_status.version)
end
print()

-- Check Candle backend status
print("Candle Backend:")
local candle_status = LocalLLM.status("candle")
print("  Health: " .. candle_status.health)
print("  Available models: " .. candle_status.available_models)

if candle_status.version then
    print("  Version: " .. candle_status.version)
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
