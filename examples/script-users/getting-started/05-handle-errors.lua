-- Example: 05-handle-errors.lua
-- Author: LLMSpell Examples
-- Purpose: Introduction to proper error handling in LLMSpell scripts
-- Learning: How to handle errors gracefully and provide good user feedback

print("=== LLMSpell: Handling Errors ===")
print("This example shows how to handle errors gracefully in your scripts!")
print()

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
        return Tool.invoke("file_operations", params)
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
            return good_agent:invoke("Say hello!")
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
    
    local result
    if operation == "get" then
        result = State and State.get(key)
    elseif operation == "set" then
        result = State and State.set(key, value)
    else
        print("   ❌ Unknown operation: " .. operation)
        return nil
    end
    
    if not State then
        print("   ⚠️  State not available (run with state-enabled config)")
        return nil
    end
    
    if result and result.success then
        print("   ✅ State " .. operation .. " successful")
        return result.result
    else
        print("   ❌ State " .. operation .. " failed: " .. (result and result.error or "Unknown error"))
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
print("🚀 You've completed the getting-started series!")
print("   Next: Explore examples/script-users/features/ for advanced topics!")