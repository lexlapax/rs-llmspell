-- ABOUTME: Loop workflow example demonstrating iteration patterns
-- ABOUTME: Shows how to use Workflow.loop() for batch processing and iterations

-- Loop Workflow Example
-- Demonstrates various iteration patterns and batch processing

-- Load workflow helpers for async execution
local helpers = dofile("examples/lua/workflows/workflow-helpers.lua")

print("=== Loop Workflow Example ===\n")

-- Example 1: Range-Based Loop
print("Example 1: Range-Based Loop")
print("-" .. string.rep("-", 27))

local range_loop = Workflow.loop({
    name = "range_processor",
    description = "Process items in a numeric range",
    
    -- Define range iterator
    iterator = {
        range = {
            start = 1,
            ["end"] = 5,
            step = 1
        }
    },
    
    -- Steps to execute for each iteration
    body = {
        {
            name = "process_number",
            type = "tool",
            tool = "calculator",
            input = {
                input = "{{loop:current_value}} * {{loop:current_value}}"  -- Square the number
            }
        },
        {
            name = "format_result",
            type = "tool",
            tool = "template_engine", 
            input = {
                template = "Iteration {{index}}: {{value}} squared = {{result}}",
                variables = {
                    index = "{{loop:current_index}}",
                    value = "{{loop:current_value}}",
                    result = "{{step:process_number:output}}"
                }
            }
        }
    },
    
    -- Collect all results
    aggregation_strategy = "collect_all"
})

print("Executing range-based loop...")
local range_result, err = helpers.executeWorkflow(range_loop)

if range_result then
    print("Results:")
    print("- Completed iterations: " .. (range_result.data and range_result.data.completed_iterations or "N/A"))
    print("- Total iterations: " .. (range_result.data and range_result.data.total_iterations or "N/A"))
else
    print("Execution error: " .. tostring(err))
end

-- Example 2: Collection-Based Loop
print("\n\nExample 2: Collection-Based Loop")
print("-" .. string.rep("-", 32))

-- Create a collection to process
local products = {
    { name = "Widget A", price = 19.99, quantity = 100 },
    { name = "Gadget B", price = 49.99, quantity = 50 },
    { name = "Tool C", price = 99.99, quantity = 25 },
    { name = "Device D", price = 149.99, quantity = 10 }
}

-- Inventory tracking variables
local inventory_total = 0

local collection_loop = Workflow.loop({
    name = "inventory_processor",
    description = "Calculate inventory values",
    
    -- Iterate over collection
    iterator = {
        collection = products
    },
    
    body = {
        {
            name = "calculate_value",
            type = "tool",
            tool = "calculator",
            input = {
                input = "{{loop:current_item.price}} * {{loop:current_item.quantity}}"
            }
        },
        {
            name = "create_record",
            type = "tool",
            tool = "template_engine",
            input = {
                template = [[
Product: {{name}}
Price: ${{price}}
Quantity: {{quantity}}
Total Value: ${{value}}
]],
                variables = {
                    name = "{{loop:current_item.name}}",
                    price = "{{loop:current_item.price}}",
                    quantity = "{{loop:current_item.quantity}}",
                    value = "{{step:calculate_value:output}}"
                }
            }
        },
        {
            name = "accumulate_total",
            type = "custom",
            execute = function(context)
                local item_value = tonumber(context.steps.calculate_value.output) or 0
                inventory_total = inventory_total + item_value
                
                return {
                    success = true,
                    output = "Running total: $" .. string.format("%.2f", inventory_total)
                }
            end
        }
    },
    
    -- Initialize before loop
    on_start = function()
        inventory_total = 0
        print("Starting inventory calculation...")
    end,
    
    -- Summarize after loop
    on_complete = function(result)
        print(string.format("\nTotal Inventory Value: $%.2f", inventory_total))
    end
})

print("Executing collection-based loop...")
local collection_result, err = helpers.executeWorkflow(collection_loop)

if collection_result then
    print("Inventory processing completed:")
    print("- Items processed: " .. (collection_result.data and collection_result.data.completed_iterations or "N/A"))
else
    print("Execution error: " .. tostring(err))
end

-- Example 3: While Condition Loop
print("\n\nExample 3: While Condition Loop")
print("-" .. string.rep("-", 31))

-- Initialize counter variables
local counter = 0
local sum = 0

local while_loop = Workflow.loop({
    name = "while_accumulator",
    description = "Accumulate values while condition is true",
    
    -- Continue while condition is met
    iterator = {
        while_condition = {
            type = "custom",
            evaluate = function()
                return sum < 100
            end
        }
    },
    
    -- Safety limit
    max_iterations = 20,
    
    body = {
        {
            name = "generate_value",
            type = "tool",
            tool = "calculator",
            input = {
                input = "10 + {{random:1:20}}"  -- Random value between 11-30
            }
        },
        {
            name = "update_sum",
            type = "custom",
            execute = function(context)
                local new_value = tonumber(context.steps.generate_value.output) or 0
                sum = sum + new_value
                counter = counter + 1
                
                return {
                    success = true,
                    output = string.format(
                        "Added %d, new sum: %d",
                        new_value,
                        sum
                    )
                }
            end
        }
    },
    
    -- Break condition
    break_condition = {
        type = "custom",
        evaluate = function()
            return sum > 150  -- Stop if sum exceeds 150
        end
    }
})

print("Executing while condition loop...")
local while_result, err = helpers.executeWorkflow(while_loop)

if while_result then
    print("While loop completed:")
    print("- Final sum: " .. sum)
    print("- Iterations: " .. counter)
    print("- Break reason: " .. (while_result.data and while_result.data.break_reason or "condition met"))
else
    print("Execution error: " .. tostring(err))
end

-- Example 4: Nested Loops
print("\n\nExample 4: Nested Loop Processing")
print("-" .. string.rep("-", 33))

-- Matrix data for nested processing
local matrix = {
    { 1, 2, 3 },
    { 4, 5, 6 },
    { 7, 8, 9 }
}

-- Matrix processing variables
local matrix_results = {}

local outer_loop = Workflow.loop({
    name = "matrix_row_processor",
    description = "Process each row of the matrix",
    
    iterator = {
        collection = matrix
    },
    
    body = {
        {
            name = "process_row",
            type = "loop",
            workflow = Workflow.loop({
                name = "row_element_processor",
                iterator = {
                    collection = "{{loop:current_item}}"  -- Current row
                },
                body = {
                    {
                        name = "double_value",
                        type = "tool",
                        tool = "calculator",
                        input = {
                            input = "{{loop:current_item}} * 2"
                        }
                    }
                },
                aggregation_strategy = "collect_outputs"
            })
        },
        {
            name = "store_row_result",
            type = "custom",
            execute = function(context)
                table.insert(matrix_results, context.steps.process_row.output)
                
                return {
                    success = true,
                    output = "Row processed"
                }
            end
        }
    }
})

print("Executing nested loop processing...")
local nested_result, err = helpers.executeWorkflow(outer_loop)

if nested_result then
    print("Matrix processing completed:")
    print("- Rows processed: " .. (nested_result.data and nested_result.data.completed_iterations or "N/A"))
    print("- Total elements processed: " .. (#matrix_results * #matrix_results[1]))
else
    print("Execution error: " .. tostring(err))
end

-- Example 5: Loop with Error Handling
print("\n\nExample 5: Loop with Error Handling")
print("-" .. string.rep("-", 35))

-- Data with some problematic entries
local data_with_errors = {
    { id = 1, value = "10" },
    { id = 2, value = "20" },
    { id = 3, value = "invalid" },  -- This will cause an error
    { id = 4, value = "30" },
    { id = 5, value = "not_a_number" },  -- Another error
    { id = 6, value = "40" }
}

-- Error tracking variables
local error_count = 0
local success_count = 0

local error_handling_loop = Workflow.loop({
    name = "robust_processor",
    description = "Process data with error recovery",
    
    iterator = {
        collection = data_with_errors
    },
    
    body = {
        {
            name = "parse_value",
            type = "tool",
            tool = "calculator",
            input = {
                input = "{{loop:current_item.value}} + 5"
            },
            on_error = function(err)
                print("Error processing item " .. 
                      context.current_index .. 
                      ": " .. tostring(err))
                error_count = error_count + 1
                return {
                    success = false,
                    output = "ERROR"
                }
            end
        },
        {
            name = "record_result",
            type = "custom",
            execute = function(context)
                if context.steps.parse_value.success then
                    success_count = success_count + 1
                    return {
                        success = true,
                        output = "Processed: " .. context.steps.parse_value.output
                    }
                else
                    return {
                        success = true,  -- Continue loop despite error
                        output = "Skipped due to error"
                    }
                end
            end
        }
    },
    
    -- Continue processing despite errors
    error_strategy = "continue"
})

print("Executing error handling loop...")
local error_result, err = helpers.executeWorkflow(error_handling_loop)

if error_result then
    print("\nError handling results:")
    print("- Total items: " .. #data_with_errors)
    print("- Successful: " .. success_count)
    print("- Errors: " .. error_count)
else
    print("Execution error: " .. tostring(err))
end

-- Example 6: Performance-Optimized Loop
print("\n\nExample 6: Performance-Optimized Loop")
print("-" .. string.rep("-", 37))

-- Large dataset simulation
local large_dataset = {}
for i = 1, 100 do
    table.insert(large_dataset, i)
end

local optimized_loop = Workflow.loop({
    name = "performance_loop",
    description = "Optimized batch processing",
    
    iterator = {
        collection = large_dataset
    },
    
    -- Process in batches
    batch_size = 10,
    
    body = {
        {
            name = "batch_sum",
            type = "custom",
            execute = function(context)
                -- In batch mode, current_item is an array
                local batch = context.current_item
                local sum = 0
                for _, val in ipairs(batch) do
                    sum = sum + val
                end
                return {
                    success = true,
                    output = sum
                }
            end
        }
    },
    
    -- Only collect summary
    aggregation_strategy = "summary"
})

local start_time = os.clock()
local perf_result, err = helpers.executeWorkflow(optimized_loop)
local elapsed = (os.clock() - start_time) * 1000

if perf_result then
    print("Performance results:")
    print(string.format("- Processed %d items in %.2f ms", 
                        #large_dataset, elapsed))
    print(string.format("- Throughput: %.0f items/second", 
                        (#large_dataset / elapsed) * 1000))
else
    print("Execution error: " .. tostring(err))
end

-- Summary
print("\n\n=== Loop Workflow Summary ===")
print("Examples demonstrated:")
print("1. Range-based iteration")
print("2. Collection processing")
print("3. While condition loops")
print("4. Nested loop patterns")
print("5. Error handling in loops")
print("6. Performance optimization")
print("\nKey features:")
print("- Multiple iterator types")
print("- Break conditions")
print("- Aggregation strategies")
print("- Batch processing")
print("- State management")

print("\n=== Loop Workflow Example Complete ===")