-- ABOUTME: Test workflow example using aligned API pattern with executeAsync
-- ABOUTME: Demonstrates that workflow-helpers.lua is no longer needed

print("=== Testing Aligned Workflow API ===")

-- Test 1: Using the old Workflow.execute pattern (should still work)
print("\n1. Testing old execute pattern...")
local workflow1 = Workflow.sequential({
    name = "test_old_pattern",
    steps = {
        {
            name = "step1",
            type = "tool",
            tool = "calculator",
            input = {
                operation = "add",
                a = 5,
                b = 10
            }
        }
    }
})

-- This would normally fail without workflow-helpers.lua
local success1, result1 = pcall(function()
    -- Create coroutine manually (old way)
    local co = coroutine.create(function()
        return Workflow.execute(workflow1)
    end)
    local ok, res = coroutine.resume(co)
    while ok and coroutine.status(co) ~= "dead" do
        ok, res = coroutine.resume(co, res)
    end
    if not ok then error(res) end
    return res
end)

if success1 then
    print("   ✓ Old pattern works: " .. tostring(result1.outputs.step1))
else
    print("   ✗ Old pattern failed: " .. tostring(result1))
end

-- Test 2: Using the new executeAsync convenience wrapper
print("\n2. Testing new executeAsync pattern...")
local workflow2 = Workflow.sequential({
    name = "test_new_pattern",
    steps = {
        {
            name = "multiply",
            type = "tool", 
            tool = "calculator",
            input = {
                operation = "multiply",
                a = 7,
                b = 8
            }
        }
    }
})

-- This should work without any helpers!
local success2, result2 = pcall(function()
    return Workflow.executeAsync(workflow2)
end)

if success2 then
    print("   ✓ New executeAsync works: " .. tostring(result2.outputs.multiply))
else
    print("   ✗ New executeAsync failed: " .. tostring(result2))
end

-- Test 3: Performance comparison
print("\n3. Performance comparison...")
local start_time = os.clock()

-- Run 5 workflows with old pattern
for i = 1, 5 do
    local wf = Workflow.sequential({
        name = "perf_test_" .. i,
        steps = {{
            name = "calc",
            type = "tool",
            tool = "calculator", 
            input = { operation = "add", a = i, b = i }
        }}
    })
    Workflow.executeAsync(wf)
end

local elapsed = (os.clock() - start_time) * 1000
print("   Time for 5 workflows: " .. string.format("%.2f ms", elapsed))
print("   Average per workflow: " .. string.format("%.2f ms", elapsed / 5))

-- Test 4: Complex workflow
print("\n4. Testing complex workflow...")
local complex_workflow = Workflow.conditional({
    name = "complex_test",
    branches = {
        {
            condition = "always",
            steps = {
                {
                    name = "get_number",
                    type = "tool",
                    tool = "calculator",
                    input = { operation = "add", a = 10, b = 20 }
                },
                {
                    name = "process",
                    type = "tool",
                    tool = "json_processor",
                    input = {
                        data = { value = 30 },
                        query = ".value"
                    }
                }
            }
        }
    }
})

local success4, result4 = pcall(function()
    return Workflow.executeAsync(complex_workflow)
end)

if success4 then
    print("   ✓ Complex workflow works")
    if result4.outputs and result4.outputs.process then
        print("   Result: " .. tostring(result4.outputs.process))
    end
else
    print("   ✗ Complex workflow failed: " .. tostring(result4))
end

print("\n=== Alignment Test Complete ===")
print("\nSummary:")
print("- Workflows now use the same efficient async pattern as agents")
print("- No more need for workflow-helpers.lua")
print("- Workflow.executeAsync() provides convenient synchronous execution")
print("- Performance should be improved (no new runtime per call)")