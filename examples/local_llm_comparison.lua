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
local ollama_status = LocalLLM.status("ollama")
local candle_status = LocalLLM.status("candle")

if ollama_status.health ~= "healthy" then
    print("Error: Ollama backend not available")
    print("Run: ollama pull llama3.1:8b")
    os.exit(1)
end

if candle_status.available_models == 0 then
    print("Error: No Candle models available")
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
