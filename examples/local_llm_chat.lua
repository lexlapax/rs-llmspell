#!/usr/bin/env llmspell

-- ============================================================
-- LLMSPELL LOCAL LLM EXAMPLES
-- ============================================================
-- Example: Interactive Chat with Local LLM
-- Complexity Level: BEGINNER
-- Real-World Use Case: Build a local coding assistant chatbot
-- Category: Local LLM Integration
--
-- Purpose: Demonstrate interactive chat using local models (Ollama or Candle)
-- Architecture: Agent creation with local model + interactive REPL loop
-- Key Features:
--   • Interactive chat interface
--   • Persistent conversation context
--   • Works with any local model
--   • Configurable via environment variable
--
-- Prerequisites:
--   • LLMSpell installed and built
--   • Default: Ollama with llama3.1:8b model
--     - Install: ollama pull llama3.1:8b
--   • Alternative: Set MODEL env var to use different model
--     - Candle example: MODEL=local/tinyllama:Q4_K_M@candle
--
-- HOW TO RUN:
-- # Using default Ollama model (llama3.1:8b):
-- ./target/debug/llmspell -p ollama run examples/local_llm_chat.lua
--
-- # Using custom model via environment variable:
-- MODEL=local/tinyllama:Q4_K_M@candle \
--   ./target/debug/llmspell -p candle run examples/local_llm_chat.lua
--
-- EXPECTED OUTPUT:
-- Interactive chat prompt. Type messages and receive responses.
-- Type 'exit' to quit.
--
-- Environment Variables:
--   MODEL - Override default model (default: local/llama3.1:8b)
--
-- Next Steps:
-- - See local_llm_comparison.lua for backend performance comparison
-- - See cookbook/multi-agent-coordination.lua for advanced patterns
-- ============================================================

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
