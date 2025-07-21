print("=== Debugging Agent.createAsync Hanging ===\n")

-- Test what happens when provider isn't configured
print("1. Testing with unconfigured provider...")

-- Create a custom version with debug output
local function createAsyncDebug(config)
    local co = coroutine.create(function()
        return Agent.create(config)
    end)
    
    local resume_count = 0
    local max_resumes = 10  -- Prevent infinite loop
    
    -- Execute the coroutine
    local success, result = coroutine.resume(co)
    print("   Initial resume: success=" .. tostring(success) .. ", result=" .. tostring(result))
    print("   Coroutine status: " .. coroutine.status(co))
    
    -- Handle async operations that yield
    while success and coroutine.status(co) ~= "dead" and resume_count < max_resumes do
        resume_count = resume_count + 1
        print("   Resume #" .. resume_count .. ": passing result=" .. tostring(result))
        success, result = coroutine.resume(co, result)
        print("   After resume: success=" .. tostring(success) .. ", result=" .. tostring(result))
        print("   Coroutine status: " .. coroutine.status(co))
    end
    
    if resume_count >= max_resumes then
        error("Max resumes reached - coroutine appears to be stuck")
    end
    
    if not success then
        error(tostring(result))
    end
    
    return result
end

-- Test with an invalid provider
local status, err = pcall(function()
    return createAsyncDebug({
        model = "invalid_provider/test-model",
        system_prompt = "Test"
    })
end)

print("\n2. Test result:")
print("   Success: " .. tostring(status))
print("   Error/Result: " .. tostring(err))

print("\n=== Debug Complete ===")