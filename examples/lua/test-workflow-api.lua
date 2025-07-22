-- ABOUTME: Test the actual Workflow API to understand how to use it
-- ABOUTME: Debug test for fixing workflow examples

print("=== Testing Workflow API ===")
print()

-- Test 1: Create a simple workflow
print("Test 1: Creating sequential workflow")
local workflow = Workflow.sequential({
    name = "test_workflow",
    description = "Simple test workflow",
    steps = {
        {
            name = "step1",
            type = "tool",
            tool = "calculator",
            input = { operation = "evaluate", input = "2 + 2" }
        }
    }
})

print("Workflow created:", workflow)
print("Type:", type(workflow))
print()

-- Test 2: Check available methods
print("Test 2: Checking workflow methods")
if type(workflow) == "userdata" then
    print("Workflow is userdata (expected)")
    
    -- Try to call execute in different ways
    print("\nTrying different execution methods:")
    
    -- Method 1: Direct call (will fail with async)
    print("\n1. Direct call:")
    local ok1, err1 = pcall(function()
        return workflow:execute()
    end)
    print("  Result:", ok1)
    if not ok1 then
        print("  Error:", err1)
    end
    
    -- Method 2: In a coroutine
    print("\n2. In coroutine:")
    local co = coroutine.create(function()
        return workflow:execute()
    end)
    local ok2, result2 = coroutine.resume(co)
    print("  First resume:", ok2, result2)
    
    -- If it yielded, we need to resume again
    if ok2 and coroutine.status(co) ~= "dead" then
        ok2, result2 = coroutine.resume(co)
        print("  Second resume:", ok2, result2)
    end
    
    -- Method 3: Check other methods
    print("\n3. Other methods:")
    local methods = {"getInfo", "getState", "setState", "validate", "debug"}
    for _, method in ipairs(methods) do
        if workflow[method] then
            print("  - " .. method .. " exists")
            if method == "getInfo" then
                local ok, info = pcall(function() return workflow:getInfo() end)
                if ok then
                    print("    Info:", info.name, info.type)
                end
            end
        end
    end
end

-- Test 3: Try Workflow.register
print("\n\nTest 3: Testing Workflow.register()")
local workflow_id = Workflow.register("sequential", {
    name = "registered_workflow",
    steps = {
        {
            name = "calc",
            type = "tool", 
            tool = "calculator",
            input = { operation = "evaluate", input = "3 * 3" }
        }
    }
})
print("Registered workflow ID:", workflow_id)

-- Test 4: List workflows
print("\n\nTest 4: Listing workflows")
local co_list = coroutine.create(function()
    return Workflow.list()
end)
local ok_list, workflows = coroutine.resume(co_list)
if ok_list and coroutine.status(co_list) ~= "dead" then
    ok_list, workflows = coroutine.resume(co_list)
end

if ok_list and workflows then
    print("Found", #workflows, "workflows")
    for i, wf in ipairs(workflows) do
        print("  -", wf.id, "(" .. wf.type .. ")")
    end
end

print("\n=== Test Complete ===")