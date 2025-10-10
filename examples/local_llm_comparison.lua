#!/usr/bin/env llmspell

--[[
Backend Comparison Example

Compares responses from Ollama and Candle backends.

Usage:
    llmspell run examples/local_llm_comparison.lua

Requirements:
    - Ollama with a model pulled
    - Candle with a model downloaded

Note: This example requires both backends to be available.
]]

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
