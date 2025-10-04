#!/usr/bin/env llmspell

--[[
Local LLM Chat Example

Simple chat interface using local LLM (Ollama or Candle).

Usage:
    llmspell run examples/local_llm_chat.lua

Requirements:
    - Ollama with llama3.1:8b OR
    - Candle with tinyllama:Q4_K_M

Environment:
    MODEL - Override model (default: local/llama3.1:8b)
]]

-- Configure model (default to Ollama llama3.1)
local model = os.getenv("MODEL") or "local/llama3.1:8b"

print("=== Local LLM Chat ===")
print("Model: " .. model)
print("Type 'exit' to quit\n")

-- Create agent
local agent = Agent.create({
    model = model,
    system_prompt = "You are a helpful coding assistant. Keep responses concise."
})

-- Chat loop
while true do
    io.write("You: ")
    io.flush()
    local user_input = io.read()

    if not user_input or user_input == "exit" then
        print("\nGoodbye!")
        break
    end

    if user_input:match("^%s*$") then
        -- Skip empty input
        goto continue
    end

    io.write("Assistant: ")
    io.flush()

    local response = agent:complete(user_input)
    print(response)
    print()

    ::continue::
end
