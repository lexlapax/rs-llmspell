-- ABOUTME: Simple workflow example without workflow-helpers.lua
-- ABOUTME: Demonstrates using Workflow.executeAsync() directly

print("=== Simple Workflow Without Helpers ===\n")

-- Create a simple sequential workflow
local workflow = Workflow.sequential({
    name = "simple_math_pipeline",
    description = "Performs simple math operations",
    steps = {
        {
            name = "add_numbers",
            type = "tool",
            tool = "calculator",
            input = {
                operation = "add",
                a = 10,
                b = 20
            }
        },
        {
            name = "multiply_result",
            type = "tool", 
            tool = "calculator",
            input = {
                operation = "multiply",
                a = 30,  -- We know the previous result is 30
                b = 2
            }
        }
    }
})

-- Execute the workflow using executeAsync (no helpers needed!)
print("Executing workflow...")
local result = Workflow.executeAsync(workflow)

-- Check results
if result and result.success then
    print("✓ Workflow completed successfully!")
    print("  Workflow type: " .. tostring(result.workflow_type))
    print("  Duration: " .. tostring(result.duration_ms) .. "ms")
    
    -- Access step results
    if result.data and result.data.step_results then
        print("\nStep Results:")
        for step_name, step_result in pairs(result.data.step_results) do
            print("  " .. step_name .. ": " .. tostring(step_result))
        end
    end
else
    print("✗ Workflow failed!")
    if result and result.error then
        print("  Error: " .. tostring(result.error.message or result.error))
    end
end

print("\n=== Parallel Workflow Example ===\n")

-- Create a parallel workflow
local parallel_workflow = Workflow.parallel({
    name = "parallel_calculations",
    description = "Run multiple calculations in parallel",
    branches = {
        {
            name = "branch1",
            steps = {
                {
                    name = "calc1",
                    type = "tool",
                    tool = "calculator",
                    input = { operation = "add", a = 5, b = 10 }
                }
            }
        },
        {
            name = "branch2", 
            steps = {
                {
                    name = "calc2",
                    type = "tool",
                    tool = "calculator",
                    input = { operation = "multiply", a = 3, b = 7 }
                }
            }
        }
    }
})

-- Execute parallel workflow
print("Executing parallel workflow...")
local parallel_result = Workflow.executeAsync(parallel_workflow)

if parallel_result and parallel_result.success then
    print("✓ Parallel workflow completed!")
    print("  Duration: " .. tostring(parallel_result.duration_ms) .. "ms")
    
    -- Show branch results
    if parallel_result.data and parallel_result.data.branch_results then
        print("\nBranch Results:")
        for branch_name, branch_data in pairs(parallel_result.data.branch_results) do
            print("  " .. branch_name .. ": " .. tostring(branch_data.status))
        end
    end
else
    print("✗ Parallel workflow failed!")
end

print("\n=== Summary ===")
print("• No workflow-helpers.lua needed!")
print("• Workflow.executeAsync() provides synchronous execution")
print("• Works with all workflow types: sequential, parallel, conditional, loop")
print("• Same efficient async pattern as Agent API")