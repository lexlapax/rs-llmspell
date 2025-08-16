-- Example: Hello World - Your First LLMSpell Script
-- Purpose: The simplest possible LLMSpell script to verify installation
-- Audience: Script Users (Beginners)
-- Prerequisites: LLMSpell installed
-- Expected Output: Welcome message and version information
-- Version: 0.7.0
-- Tags: getting-started, hello-world, beginner

-- This is the simplest LLMSpell script. It demonstrates:
-- 1. Basic Lua syntax
-- 2. Printing output
-- 3. Checking LLMSpell availability

print("Hello from LLMSpell! 🎉")
print("") -- Empty line for readability

-- Check if we're running in LLMSpell environment
if _VERSION then
    print("Lua version: " .. _VERSION)
end

-- Verify LLMSpell globals are available
local globals_available = {}
if Tool then table.insert(globals_available, "Tool") end
if Agent then table.insert(globals_available, "Agent") end
if Workflow then table.insert(globals_available, "Workflow") end
if State then table.insert(globals_available, "State") end
if Provider then table.insert(globals_available, "Provider") end

if #globals_available > 0 then
    print("LLMSpell globals available: " .. table.concat(globals_available, ", "))
else
    print("Warning: No LLMSpell globals detected. Are you running this with llmspell?")
end

print("")
print("✅ Your LLMSpell installation is working!")
print("")
print("Next steps:")
print("  1. Try '02-first-tool' to use your first tool")
print("  2. Run 'llmspell --help' to see all commands")
print("  3. Explore the examples directory for more")