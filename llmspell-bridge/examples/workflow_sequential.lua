-- ABOUTME: Example of creating and executing a sequential workflow in Lua
-- ABOUTME: Demonstrates step-by-step execution with data flow between steps

-- Create a sequential workflow that performs multiple calculations
local workflow = Workflow.sequential({
    name = "calculation_pipeline",
    description = "A sequential workflow that performs a series of calculations",
    steps = {
        {
            name = "add_numbers",
            tool = "calculator",
            parameters = {
                operation = "add",
                a = 10,
                b = 20
            },
            error_strategy = "fail_fast"
        },
        {
            name = "multiply_result",
            tool = "calculator",
            parameters = {
                operation = "multiply",
                a = "$add_numbers_output",  -- Reference output from previous step
                b = 2
            }
        },
        {
            name = "final_calculation",
            tool = "calculator",
            parameters = {
                operation = "subtract",
                a = "$multiply_result_output",
                b = 5
            }
        }
    },
    error_strategy = "fail_fast"  -- Stop on first error
})

-- Execute the workflow
print("Executing sequential workflow...")
local result = Workflow.execute(workflow)

-- Check results
if result.success then
    print("Workflow completed successfully!")
    print("Final output:", result.outputs.final_calculation)
    
    -- Access individual step outputs
    if result.outputs then
        print("\nStep outputs:")
        print("  add_numbers:", result.outputs.add_numbers)
        print("  multiply_result:", result.outputs.multiply_result)
        print("  final_calculation:", result.outputs.final_calculation)
    end
else
    print("Workflow failed!")
    print("Error:", result.error)
    
    -- Check which step failed
    if result.failed_step then
        print("Failed at step:", result.failed_step)
    end
end

-- List active workflows
local active_workflows = Workflow.list()
print("\nActive workflows:", #active_workflows)
for i, w in ipairs(active_workflows) do
    print(string.format("  %d. ID: %s, Type: %s", i, w.id, w.type))
end