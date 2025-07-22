-- ABOUTME: Test workflow executeAsync functionality
-- ABOUTME: Verifies that workflows can be executed without helper files

print("=== Testing Workflow.executeAsync ===")

-- Test: Using executeAsync with a workflow instance
print("\n1. Creating and executing workflow with executeAsync...")

-- Create a workflow instance
local workflow = Workflow.sequential({
    name = "test_async",
    steps = {
        {
            name = "add_numbers",
            type = "tool",
            tool = "calculator",
            input = {
                operation = "add",
                a = 15,
                b = 25
            }
        }
    }
})

-- Execute using the async wrapper
local success, result = pcall(function()
    return Workflow.executeAsync(workflow)
end)

if success then
    print("   ✓ Workflow executed successfully")
    if result and result.outputs and result.outputs.add_numbers then
        print("   Result: 15 + 25 = " .. tostring(result.outputs.add_numbers))
    else
        print("   Raw result: " .. tostring(result))
        if type(result) == "table" then
            for k, v in pairs(result) do
                print("     " .. tostring(k) .. " = " .. tostring(v))
            end
        end
    end
else
    print("   ✗ Workflow execution failed: " .. tostring(result))
end

-- Test 2: Direct execute (for comparison)
print("\n2. Testing direct execute method...")
local workflow2 = Workflow.sequential({
    name = "test_direct",
    steps = {
        {
            name = "multiply",
            type = "tool",
            tool = "calculator",
            input = {
                operation = "multiply",
                a = 6,
                b = 7
            }
        }
    }
})

-- Try direct execute with manual coroutine
local success2, result2 = pcall(function()
    local co = coroutine.create(function()
        return workflow2:execute()
    end)
    local ok, res = coroutine.resume(co)
    while ok and coroutine.status(co) ~= "dead" do
        ok, res = coroutine.resume(co, res)
    end
    if not ok then error(res) end
    return res
end)

if success2 then
    print("   ✓ Direct execute works")
    if result2 and result2.outputs and result2.outputs.multiply then
        print("   Result: 6 × 7 = " .. tostring(result2.outputs.multiply))
    else
        print("   Raw result: " .. tostring(result2))
        if type(result2) == "table" then
            for k, v in pairs(result2) do
                print("     " .. tostring(k) .. " = " .. tostring(v))
                if k == "outputs" and type(v) == "table" then
                    print("     Outputs:")
                    for ok, ov in pairs(v) do
                        print("       " .. tostring(ok) .. " = " .. tostring(ov))
                    end
                end
            end
        end
    end
else
    print("   ✗ Direct execute failed: " .. tostring(result2))
end

print("\n=== Test Complete ===")
print("\nThe executeAsync wrapper eliminates the need for workflow-helpers.lua!")