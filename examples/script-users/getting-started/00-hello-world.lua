-- Example: Hello World
-- Purpose: Simplest possible LLMSpell script demonstrating basic execution
-- Prerequisites: None
-- Expected Output: Prints greeting and returns success object
-- Version: 0.7.0
-- Tags: getting-started, basics, no-dependencies

-- Simple hello world script
print("Hello from LLMSpell!")

-- Return a value
return {
    message = "Script executed successfully",
    engine = "Lua",
    timestamp = os.date()
}