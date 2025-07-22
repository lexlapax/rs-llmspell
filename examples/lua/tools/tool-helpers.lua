-- ABOUTME: Helper functions for tool examples
-- ABOUTME: Provides utilities for handling async tool invocation

-- Helper to invoke async tool methods synchronously
function invokeTool(name, input)
    local result = nil
    local error = nil
    
    -- Create coroutine for async execution
    local co = coroutine.create(function()
        return Tool.invoke(name, input)
    end)
    
    -- Resume until complete
    local ok, value = coroutine.resume(co)
    
    -- Handle async operations that yield
    while ok and coroutine.status(co) ~= "dead" do
        ok, value = coroutine.resume(co, value)
    end
    
    if ok then
        result = value
    else
        error = value
    end
    
    return result, error
end

-- Export helpers
return {
    invokeTool = invokeTool
}