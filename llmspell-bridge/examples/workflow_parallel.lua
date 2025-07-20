-- ABOUTME: Example of creating and executing a parallel workflow in Lua
-- ABOUTME: Demonstrates concurrent execution of multiple independent steps

-- Create a parallel workflow that performs multiple independent operations
local workflow = Workflow.parallel({
    name = "parallel_processing",
    description = "A workflow that executes multiple tasks concurrently",
    steps = {
        {
            name = "calculation_1",
            tool = "calculator",
            parameters = {
                operation = "add",
                a = 100,
                b = 200
            }
        },
        {
            name = "calculation_2",
            tool = "calculator",
            parameters = {
                operation = "multiply",
                a = 50,
                b = 3
            }
        },
        {
            name = "text_processing",
            tool = "text_manipulator",
            parameters = {
                operation = "uppercase",
                text = "hello parallel world"
            }
        },
        {
            name = "uuid_generation",
            tool = "uuid_generator",
            parameters = {
                format = "v4"
            }
        }
    },
    max_concurrency = 3,  -- Limit concurrent executions
    error_strategy = "continue"  -- Continue even if some steps fail
})

-- Execute the workflow
print("Executing parallel workflow...")
local start_time = os.clock()
local result = Workflow.execute(workflow)
local end_time = os.clock()

-- Check results
if result.success then
    print(string.format("Workflow completed successfully in %.2f seconds!", end_time - start_time))
    
    -- Display all outputs
    if result.outputs then
        print("\nParallel execution results:")
        for step_name, output in pairs(result.outputs) do
            print(string.format("  %s: %s", step_name, tostring(output)))
        end
    end
else
    print("Workflow completed with errors!")
    print("Overall error:", result.error or "Some steps failed")
    
    -- Check which steps succeeded/failed
    if result.step_results then
        print("\nStep results:")
        for step_name, step_result in pairs(result.step_results) do
            local status = step_result.success and "SUCCESS" or "FAILED"
            print(string.format("  %s: %s", step_name, status))
            if not step_result.success then
                print(string.format("    Error: %s", step_result.error))
            end
        end
    end
end

-- Performance comparison example
print("\n--- Performance Comparison ---")

-- Sequential version for comparison
local seq_workflow = Workflow.sequential({
    name = "sequential_comparison",
    steps = workflow.config.steps  -- Same steps
})

print("Running same tasks sequentially...")
start_time = os.clock()
local seq_result = Workflow.execute(seq_workflow)
end_time = os.clock()

print(string.format("Sequential execution time: %.2f seconds", end_time - start_time))
print("Parallel execution provides speedup for independent tasks!")