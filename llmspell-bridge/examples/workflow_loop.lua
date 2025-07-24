-- ABOUTME: Example of creating and executing a loop workflow in Lua
-- ABOUTME: Demonstrates iterative execution with dynamic conditions

-- Example 1: Simple counter loop
local counter_workflow = Workflow.loop({
    name = "counter_loop",
    description = "A simple loop that counts up to 5",
    
    max_iterations = 10,  -- Safety limit
    condition = "iteration < 5",  -- Loop while this is true
    
    body = {
        name = "count_step",
        tool = "calculator",
        parameters = {
            operation = "add",
            a = "$iteration",  -- Special variable available in loops
            b = 1
        }
    }
})

print("=== Counter Loop Example ===")
local result = Workflow.execute(counter_workflow)

if result.success then
    print("Loop completed successfully!")
    print(string.format("Total iterations: %d", result.iterations or 0))
    
    if result.iteration_outputs then
        print("\nIteration outputs:")
        for i, output in ipairs(result.iteration_outputs) do
            print(string.format("  Iteration %d: %s", i - 1, tostring(output)))
        end
    end
else
    print("Loop failed:", result.error)
end

-- Example 2: Accumulator pattern
print("\n=== Accumulator Loop Example ===")

local accumulator_workflow = Workflow.loop({
    name = "sum_accumulator",
    description = "Sum numbers from 1 to N",
    
    max_iterations = 100,
    condition = "iteration <= 10",  -- Sum 1 to 10
    
    -- Initialize accumulator
    initial_state = {
        sum = 0
    },
    
    body = {
        name = "add_to_sum",
        tool = "calculator",
        parameters = {
            operation = "add",
            a = "$state.sum",  -- Access loop state
            b = "$iteration"
        }
    },
    
    -- Update state after each iteration
    state_update = "state.sum = body_output"
})

result = Workflow.execute(accumulator_workflow)
if result.success then
    print(string.format("Sum of 1 to 10: %s", tostring(result.final_state and result.final_state.sum)))
end

-- Example 3: Dynamic termination condition
print("\n=== Dynamic Termination Example ===")

local search_workflow = Workflow.loop({
    name = "search_loop",
    description = "Search for a specific value",
    
    max_iterations = 20,
    
    -- Continue until we find what we're looking for
    condition = "not state.found",
    
    initial_state = {
        found = false,
        target = 42,
        current = 0
    },
    
    body = {
        name = "generate_and_check",
        tool = "calculator",
        parameters = {
            operation = "add",
            a = "$state.current",
            b = 7  -- Increment by 7 each time
        }
    },
    
    -- Check if we found our target
    state_update = [[
        state.current = body_output
        state.found = (body_output >= state.target)
    ]]
})

result = Workflow.execute(search_workflow)
if result.success then
    local final_state = result.final_state or {}
    if final_state.found then
        print(string.format("Found target! Value: %s after %d iterations", 
            tostring(final_state.current), result.iterations))
    else
        print("Target not found within iteration limit")
    end
end

-- Example 4: Processing a list with early termination
print("\n=== List Processing Loop Example ===")

-- Simulate processing items until we find an error
local process_items_workflow = Workflow.loop({
    name = "item_processor",
    description = "Process items until error or completion",
    
    max_iterations = 50,
    
    initial_state = {
        items = {10, 20, 30, 0, 40, 50},  -- 0 will cause an error
        index = 1,
        results = {},
        error_found = false
    },
    
    -- Continue while we have items and no error
    condition = "state.index <= #state.items and not state.error_found",
    
    body = {
        name = "process_item",
        tool = "calculator",
        parameters = {
            operation = "divide",
            a = 100,
            b = "$state.items[state.index]"  -- This will fail on 0
        }
    },
    
    -- Handle result or error
    state_update = [[
        if body_success then
            table.insert(state.results, body_output)
            state.index = state.index + 1
        else
            state.error_found = true
            state.error_item = state.items[state.index]
            state.error_message = body_error
        end
    ]],
    
    -- Continue on error to allow our state update to handle it
    error_strategy = "continue"
})

result = Workflow.execute(process_items_workflow)
if result.success or result.partial_success then
    local final_state = result.final_state or {}
    print(string.format("Processed %d items", #(final_state.results or {})))
    
    if final_state.error_found then
        print(string.format("Error found at item value %s: %s", 
            tostring(final_state.error_item), 
            final_state.error_message or "unknown error"))
    end
    
    if final_state.results and #final_state.results > 0 then
        print("\nProcessed results:")
        for i, res in ipairs(final_state.results) do
            print(string.format("  Item %d: %s", i, tostring(res)))
        end
    end
end

-- Show all workflow types
print("\n=== Available Workflow Types ===")
local types = Workflow.discover_types()
for _, wf_type in ipairs(types) do
    print(string.format("- %s: %s", wf_type.type, wf_type.description))
    if wf_type.features and #wf_type.features > 0 then
        print("  Features:", table.concat(wf_type.features, ", "))
    end
end