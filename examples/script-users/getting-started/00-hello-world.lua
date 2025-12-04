-- ============================================================
-- LLMSPELL GETTING STARTED SHOWCASE
-- ============================================================
-- Phase: 13c.5.6 - Example Header Standardization
-- Category: getting-started
-- Profile: minimal (recommended)
-- Example ID: 00 - Hello World v0.14.0
-- Complexity: BEGINNER
-- Real-World Use Case: Installation verification and basic script execution
--
-- Purpose: Verify LLMSpell installation and demonstrate simplest script structure.
--          This is your first step into LLMSpell scripting - confirms the runtime
--          is working and shows basic Lua execution environment.
--
-- Architecture: Direct Lua execution with no dependencies
-- Crates Showcased: llmspell-bridge (Lua runtime)
--
-- Key Features:
--   • Basic script execution
--   • Return value demonstration
--   • Environment information display
--
-- Prerequisites:
--   • LLMSpell installed and built
--   • No API keys required
--   • No configuration files required
--
-- HOW TO RUN:
-- ./target/debug/llmspell -p minimal \
--   run examples/script-users/getting-started/00-hello-world.lua
--
-- EXPECTED OUTPUT:
-- Hello from LLMSpell!
-- LLMSpell version info
-- Lua version: Lua 5.4
-- Available globals: Agent, Tool, Workflow, State, Provider, Config, Debug, JSON, Args
-- Script executed successfully
--
-- Runtime: ~2 seconds
-- ============================================================

print("Hello from LLMSpell!")
print("LLMSpell version: 0.7.0")
print("Lua version: " .. _VERSION)

-- Display available globals
local globals = {}
for name, _ in pairs(_G) do
    if name:match("^[A-Z]") and type(_G[name]) == "table" then
        table.insert(globals, name)
    end
end
table.sort(globals)
print("Available globals: " .. table.concat(globals, ", "))

-- Return success
print("Script executed successfully at " .. os.date())
return {
    success = true,
    message = "Hello World completed",
    timestamp = os.date()
}