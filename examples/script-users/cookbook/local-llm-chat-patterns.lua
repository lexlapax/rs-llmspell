-- ============================================================
-- LLMSPELL COOKBOOK SHOWCASE
-- ============================================================
-- Example: Local LLM Chat Patterns
-- Phase: 13c.5.6 - Example Header Standardization
-- Category: cookbook
--   Comprehensive local LLM chat patterns including interactive chat and
--   backend comparison. Demonstrates Ollama and Candle usage patterns.
--
-- Prerequisites:
--   - Ollama running with models: ollama pull llama3.1:8b
--   - Optional: Candle models for comparison section
--
-- Profile: ollama
-- Runtime: ~5 minutes (interactive)
-- Complexity: INTERMEDIATE
--
-- Usage:
--   llmspell -p ollama run examples/script-users/cookbook/local-llm-chat-patterns.lua
--
-- Expected Output:
--   Interactive chat interface followed by optional backend comparison
--
-- ============================================================================

-- Section 1: Interactive Chat with Local LLM
-- ============================================================================
print("=== Section 1: Interactive Chat ===\n")

local function run_interactive_chat()
    -- Configure model (default to Ollama llama3.1)
    local model = os.getenv("MODEL") or "local/llama3.1:8b"

    print("Model: " .. model)
    print("Type 'exit' to quit, 'compare' to run backend comparison\n")

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

        if user_input == "compare" then
            print("\nSwitching to backend comparison...\n")
            return "compare"
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

    return "exit"
end

-- Section 2: Backend Performance Comparison
-- ============================================================================
local function run_backend_comparison()
    print("=== Section 2: Backend Comparison ===\n")

    -- Check if both backends are available
    local status = LocalLLM.status()

    if not status.ollama.running or status.ollama.models == 0 then
        print("⚠ Ollama backend not available or no models")
        if status.ollama.error then
            print("  Error: " .. status.ollama.error)
        end
        print("  Setup: ollama pull llama3.1:8b")
        print("\nSkipping backend comparison.\n")
        return
    end

    if not status.candle.ready or status.candle.models == 0 then
        print("⚠ Candle backend not ready or no models")
        if status.candle.error then
            print("  Error: " .. status.candle.error)
        end
        print("  Setup: llmspell model pull tinyllama:Q4_K_M@candle")
        print("\nSkipping backend comparison.\n")
        return
    end

    -- Get first model from each backend
    local ollama_models = LocalLLM.list("ollama")
    local candle_models = LocalLLM.list("candle")

    local ollama_model = "local/" .. ollama_models[1].id .. "@ollama"
    local candle_model = "local/" .. candle_models[1].id .. "@candle"

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

    print("Backend comparison complete!")
end

-- Main Program
-- ============================================================================
local result = run_interactive_chat()

if result == "compare" then
    run_backend_comparison()
end

print("\n=== Example Complete ===")
print("Next steps:")
print("  - Explore features/local-llm-status.lua for model management")
print("  - Explore features/local-llm-model-info.lua for model details")
print("  - See cookbook/ for more advanced patterns")
