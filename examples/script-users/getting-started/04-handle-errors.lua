-- ============================================================
-- LLMSPELL GETTING STARTED SHOWCASE
-- ============================================================
-- Phase: 13c.5.6 - Example Header Standardization
-- Category: getting-started
-- Profile: state (recommended)
-- Example ID: 04 - Error Handling v0.14.0
-- Complexity: BEGINNER
-- Real-World Use Case: Robust automation with graceful error recovery
--
-- Purpose: Learn essential error handling patterns for production scripts.
--          Demonstrates pcall for error catching, result validation, fallback
--          strategies, and user-friendly error reporting. Critical for reliability.
--
-- Architecture: Defensive programming with error boundaries
-- Crates Showcased: llmspell-tools, llmspell-agents, llmspell-state, llmspell-bridge
--
-- Key Features:
--   • Safe function wrapping with pcall
--   • Result validation patterns
--   • Graceful degradation strategies
--   • Informative error messages
--   • State API error handling with scopes
--
-- Prerequisites:
--   • LLMSpell installed and built
--   • Optional: OPENAI_API_KEY or ANTHROPIC_API_KEY for agent testing
--   • Optional: Use `-p state` for state persistence demo
--
-- HOW TO RUN:
-- # Basic (no state):
-- ./target/debug/llmspell -p minimal \
--   run examples/script-users/getting-started/04-handle-errors.lua
--
-- # With state enabled:
-- ./target/debug/llmspell -p state \
--   run examples/script-users/getting-started/04-handle-errors.lua
--
-- EXPECTED OUTPUT:
-- File operation errors handled gracefully
-- Agent creation errors caught and reported
-- State operations with proper scope handling
-- Best practices demonstrated
--
-- Runtime: ~5 seconds
-- ============================================================

print("=== LLMSpell: Handling Errors ===")
print("Example 04: BEGINNER - Production-ready error handling")
print("Showcasing: Defensive programming and graceful recovery\n")

print("1. Basic error handling with tool operations...")

-- Demonstrate error handling with file operations
local function safe_file_operation(operation, path, content)
    print("   Attempting " .. operation .. " on: " .. path)
    
    local params = {
        operation = operation,
        path = path
    }
    
    if content then
        params.input = content
    end
    
    local success, result = pcall(function()
        return Tool.execute("file-operations", params)
    end)
    
    if success and result and result.text then
        print("   ✅ Success: " .. operation .. " completed")
        print("   Result: " .. result.text)
        return result.text
    else
        local error_msg = success and (result and result.error or "Unknown error") or tostring(result)
        print("   ❌ Error: " .. error_msg)
        return nil
    end
end

-- Try to read a file that doesn't exist
print("\n📁 Testing file that doesn't exist:")
local missing_file = safe_file_operation("read", "/tmp/does_not_exist.txt")

-- Create a file and then read it
print("\n📁 Creating and reading a valid file:")
safe_file_operation("write", "/tmp/error_demo.txt", "This file demonstrates error handling")
safe_file_operation("read", "/tmp/error_demo.txt")

print()
print("2. Error handling with agent operations...")

local function safe_agent_creation(provider_name)
    print("   Attempting to create agent with provider: " .. provider_name)
    
    local success, agent_result = pcall(function()
        return Agent.builder()
            .provider(provider_name)
            .system_prompt("You are a helpful assistant.")
            .build()
    end)
    
    if success and agent_result and agent_result.success then
        print("   ✅ Agent created successfully")
        return agent_result.result
    else
        local error_msg = success and (agent_result and agent_result.error or "Unknown error") or tostring(agent_result)
        print("   ❌ Agent creation failed: " .. error_msg)
        return nil
    end
end

-- Try with an invalid provider
print("\n🤖 Testing invalid provider:")
local bad_agent = safe_agent_creation("invalid_provider")

-- Try with valid providers if available
print("\n🤖 Testing with available providers:")
local providers = Provider.list()
if #providers > 0 then
    local good_agent = safe_agent_creation(providers[1])
    
    if good_agent then
        print("   Attempting conversation...")
        local success, response = pcall(function()
            return good_agent:execute("Say hello!")
        end)
        
        if success and response and response.success then
            print("   ✅ Response: " .. string.sub(response.result.content, 1, 50) .. "...")
        else
            local error_msg = success and (response and response.error or "Unknown error") or tostring(response)
            print("   ❌ Conversation failed: " .. error_msg)
        end
    end
else
    print("   ⚠️  No providers available to test with")
end

print()
print("3. Error handling with state operations...")

local function safe_state_operation(operation, key, value)
    print("   Attempting state " .. operation .. " for key: " .. key)
    
    if not State then
        print("   ⚠️  State not available (run with state-enabled config)")
        return nil
    end
    
    local success, result
    if operation == "get" then
        success, result = pcall(function()
            return State.load("global", key)
        end)
    elseif operation == "set" then
        success, result = pcall(function()
            State.save("global", key, value)
            return true
        end)
    else
        print("   ❌ Unknown operation: " .. operation)
        return nil
    end
    
    if success then
        print("   ✅ State " .. operation .. " successful")
        return result
    else
        print("   ❌ State " .. operation .. " failed: " .. tostring(result))
        return nil
    end
end

print("\n💾 Testing state operations:")
safe_state_operation("set", "error_demo", "test_value")
safe_state_operation("get", "error_demo")
safe_state_operation("get", "nonexistent_key")

print()
print("4. Best practices for error handling...")

print("\n💡 Best practices demonstrated:")
print("   ✅ Always check .success before using .result")
print("   ✅ Provide helpful error messages to users")
print("   ✅ Use fallbacks when operations fail")
print("   ✅ Wrap operations in functions for reusability")
print("   ✅ Handle missing dependencies gracefully")

print("\n🔧 Example error handling pattern:")
print([[
   local function safe_operation()
       local result = SomeAPI.call()
       if result.success then
           return result.result
       else
           print("Error: " .. (result.error or "Unknown"))
           return nil
       end
   end
]])

print()
print("🎉 Congratulations! You've successfully learned:")
print("   - How to check for errors in tool operations")
print("   - How to handle agent creation failures")
print("   - How to gracefully handle missing dependencies")
print("   - Best practices for error handling patterns")
print()
print()
print("Next: Try 05-memory-rag-advanced.lua to learn about Memory & RAG systems!")