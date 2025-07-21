print("=== Testing Async Yield Count ===\n")

-- Create a version that counts yields
local function createAsyncWithCount(config)
    local co = coroutine.create(function()
        return Agent.create(config)
    end)
    
    local resume_count = 0
    local max_resumes = 10
    local yields = {}
    
    -- Execute the coroutine
    local success, result = coroutine.resume(co)
    table.insert(yields, {success = success, result = tostring(result), status = coroutine.status(co)})
    
    -- Handle async operations that yield
    while success and coroutine.status(co) ~= "dead" and resume_count < max_resumes do
        resume_count = resume_count + 1
        success, result = coroutine.resume(co, result)
        table.insert(yields, {
            resume = resume_count,
            success = success, 
            result = tostring(result), 
            status = coroutine.status(co)
        })
    end
    
    return {
        resume_count = resume_count,
        final_success = success,
        final_result = result,
        yields = yields
    }
end

-- Test with no provider
print("1. Testing with no provider configured...")
local result = createAsyncWithCount({
    model = "openai/gpt-4",
    system_prompt = "Test"
})

print("   Resume count: " .. result.resume_count)
print("   Final success: " .. tostring(result.final_success))
print("   Yields:")
for i, y in ipairs(result.yields) do
    print(string.format("     %d: success=%s, status=%s, result=%.50s...", 
        i, tostring(y.success), y.status, y.result))
end

print("\n=== Test Complete ===")