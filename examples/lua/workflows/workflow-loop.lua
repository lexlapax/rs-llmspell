-- ABOUTME: Loop workflow example demonstrating iteration patterns
-- ABOUTME: Shows how to use Workflow.loop() for batch processing and iterations

-- Loop Workflow Example
-- Demonstrates various iteration patterns and batch processing

-- Note: All workflow and tool methods are now synchronous - no helpers needed

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
local range_result = range_loop:execute()

if range_result then
    print("Results:")
    print("- Completed iterations: " .. (range_result.data and range_result.data.completed_iterations or "N/A"))
    print("- Total iterations: " .. (range_result.data and range_result.data.total_iterations or "N/A"))
else
    print("Execution error: Unknown error")
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

-- Initialize inventory state file
local inventory_state_file = "/tmp/inventory_state.json"

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
            name = "read_current_total",
            type = "tool",
            tool = "file_operations",
            input = {
                operation = "read",
                path = inventory_state_file
            },
            on_error = function(err)
                -- Initialize if file doesn't exist
                return {
                    success = true,
                    output = '{"total": 0}'
                }
            end
        },
        {
            name = "parse_total",
            type = "tool",
            tool = "json_processor",
            input = {
                json = "{{step:read_current_total:output}}",
                query = "$.total"
            }
        },
        {
            name = "calculate_new_total",
            type = "tool",
            tool = "calculator",
            input = {
                input = "{{step:parse_total:output}} + {{step:calculate_value:output}}"
            }
        },
        {
            name = "save_new_total",
            type = "tool",
            tool = "json_processor",
            input = {
                json = '{"total": {{step:calculate_new_total:output}}}',
                query = "$"
            }
        },
        {
            name = "write_total_state",
            type = "tool",
            tool = "file_operations",
            input = {
                operation = "write",
                path = inventory_state_file,
                content = "{{step:save_new_total:output}}"
            }
        }
    },
    
    -- Initialize before loop
    on_start = function()
        -- Initialize state file
        local init_tool = Tool.getByName("file_operations")
        init_tool:execute({
            operation = "write",
            path = inventory_state_file,
            content = '{"total": 0}'
        })
        print("Starting inventory calculation...")
    end,
    
    -- Summarize after loop
    on_complete = function(result)
        -- Read final total
        local read_tool = Tool.getByName("file_operations")
        local final_state = read_tool:execute({
            operation = "read",
            path = inventory_state_file
        })
        
        if final_state.success then
            local json_tool = Tool.getByName("json_processor")
            local total_result = json_tool:execute({
                json = final_state.output,
                query = "$.total"
            })
            if total_result.success then
                print(string.format("\nTotal Inventory Value: $%.2f", tonumber(total_result.output)))
            end
        end
    end
})

print("Executing collection-based loop...")
local collection_result = collection_loop:execute()

if collection_result then
    print("Inventory processing completed:")
    print("- Items processed: " .. (collection_result.data and collection_result.data.completed_iterations or "N/A"))
else
    print("Execution error: Unknown error")
end

-- Example 3: While Condition Loop
print("\n\nExample 3: While Condition Loop")
print("-" .. string.rep("-", 31))

-- Initialize state files
local while_state_file = "/tmp/while_state.json"

local while_loop = Workflow.loop({
    name = "while_accumulator",
    description = "Accumulate values while condition is true",
    
    -- Continue while condition is met
    iterator = {
        while_condition = {
            type = "tool",
            tool = "json_processor",
            input = {
                json = "{{file:" .. while_state_file .. "}}",
                query = "$.sum < 100"
            }
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
            name = "read_state",
            type = "tool",
            tool = "file_operations",
            input = {
                operation = "read",
                path = while_state_file
            }
        },
        {
            name = "parse_sum",
            type = "tool",
            tool = "json_processor",
            input = {
                json = "{{step:read_state:output}}",
                query = "$.sum"
            }
        },
        {
            name = "parse_counter",
            type = "tool",
            tool = "json_processor",
            input = {
                json = "{{step:read_state:output}}",
                query = "$.counter"
            }
        },
        {
            name = "calculate_new_sum",
            type = "tool",
            tool = "calculator",
            input = {
                input = "{{step:parse_sum:output}} + {{step:generate_value:output}}"
            }
        },
        {
            name = "increment_counter",
            type = "tool",
            tool = "calculator",
            input = {
                input = "{{step:parse_counter:output}} + 1"
            }
        },
        {
            name = "create_update_message",
            type = "tool",
            tool = "template_engine",
            input = {
                template = "Added {{value}}, new sum: {{sum}}",
                variables = {
                    value = "{{step:generate_value:output}}",
                    sum = "{{step:calculate_new_sum:output}}"
                }
            }
        },
        {
            name = "update_state",
            type = "tool",
            tool = "json_processor",
            input = {
                json = '{"sum": {{step:calculate_new_sum:output}}, "counter": {{step:increment_counter:output}}}',
                query = "$"
            }
        },
        {
            name = "save_state",
            type = "tool",
            tool = "file_operations",
            input = {
                operation = "write",
                path = while_state_file,
                content = "{{step:update_state:output}}"
            }
        }
    },
    
    -- Break condition using json_processor
    break_condition = {
        type = "tool",
        tool = "json_processor",
        input = {
            json = "{{file:" .. while_state_file .. "}}",
            query = "$.sum > 150"
        }
    },
    
    -- Initialize before loop
    on_start = function()
        local init_tool = Tool.getByName("file_operations")
        init_tool:execute({
            operation = "write",
            path = while_state_file,
            content = '{"sum": 0, "counter": 0}'
        })
    end
})

print("Executing while condition loop...")
local while_result = while_loop:execute()

if while_result then
    -- Read final state
    local read_tool = Tool.getByName("file_operations")
    local final_state = read_tool:execute({
        operation = "read",
        path = while_state_file
    })
    
    if final_state.success then
        local json_tool = Tool.getByName("json_processor")
        local sum_result = json_tool:execute({
            json = final_state.output,
            query = "$.sum"
        })
        local counter_result = json_tool:execute({
            json = final_state.output,
            query = "$.counter"
        })
        
        print("While loop completed:")
        print("- Final sum: " .. (sum_result.output or "N/A"))
        print("- Iterations: " .. (counter_result.output or "N/A"))
        print("- Break reason: " .. (while_result.data and while_result.data.break_reason or "condition met"))
    end
else
    print("Execution error: Unknown error")
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

-- Matrix results file
local matrix_results_file = "/tmp/matrix_results.json"

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
            name = "read_results",
            type = "tool",
            tool = "file_operations",
            input = {
                operation = "read",
                path = matrix_results_file
            },
            on_error = function(err)
                return {
                    success = true,
                    output = '{"rows": []}'
                }
            end
        },
        {
            name = "append_row",
            type = "tool",
            tool = "json_processor",
            input = {
                json = "{{step:read_results:output}}",
                query = "$.rows",
                operation = "append",
                value = "{{step:process_row:output}}"
            }
        },
        {
            name = "save_results",
            type = "tool",
            tool = "file_operations",
            input = {
                operation = "write",
                path = matrix_results_file,
                content = '{"rows": {{step:append_row:output}}}'
            }
        }
    },
    
    -- Initialize before loop
    on_start = function()
        local init_tool = Tool.getByName("file_operations")
        init_tool:execute({
            operation = "write",
            path = matrix_results_file,
            content = '{"rows": []}'
        })
    end
})

print("Executing nested loop processing...")
local nested_result = outer_loop:execute()

if nested_result then
    -- Read final results
    local read_tool = Tool.getByName("file_operations")
    local final_results = read_tool:execute({
        operation = "read",
        path = matrix_results_file
    })
    
    if final_results.success then
        local json_tool = Tool.getByName("json_processor")
        local rows_result = json_tool:execute({
            json = final_results.output,
            query = "$.rows | length"
        })
        
        print("Matrix processing completed:")
        print("- Rows processed: " .. (nested_result.data and nested_result.data.completed_iterations or "N/A"))
        print("- Total rows stored: " .. (rows_result.output or "N/A"))
    end
else
    print("Execution error: Unknown error")
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

-- Error tracking file
local error_state_file = "/tmp/error_state.json"

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
                -- Update error count in state file
                local read_tool = Tool.getByName("file_operations")
                local state_result = read_tool:execute({
                    operation = "read",
                    path = error_state_file
                })
                
                if state_result.success then
                    local json_tool = Tool.getByName("json_processor")
                    local error_count_result = json_tool:execute({
                        json = state_result.output,
                        query = "$.error_count"
                    })
                    
                    local calc_tool = Tool.getByName("calculator")
                    local new_error_count = calc_tool:execute({
                        input = (error_count_result.output or "0") .. " + 1"
                    })
                    
                    if new_error_count.success then
                        local update_result = json_tool:execute({
                            json = state_result.output,
                            query = "$",
                            operation = "set",
                            path = "$.error_count",
                            value = new_error_count.output
                        })
                        
                        read_tool:execute({
                            operation = "write",
                            path = error_state_file,
                            content = update_result.output
                        })
                    end
                end
                
                return {
                    success = false,
                    output = "ERROR"
                }
            end
        },
        {
            name = "read_state",
            type = "tool",
            tool = "file_operations",
            input = {
                operation = "read",
                path = error_state_file
            }
        },
        {
            name = "update_count",
            type = "tool",
            tool = "json_processor",
            input = {
                json = "{{step:read_state:output}}",
                query = "$",
                operation = "set",
                path = "$.{{step:parse_value:success ? 'success_count' : 'error_count'}}",
                value = "{{step:parse_value:success ? '{{step:read_state:output | json_query:$.success_count}} + 1' : '{{step:read_state:output | json_query:$.error_count}}'}}"
            }
        },
        {
            name = "save_state",
            type = "tool",
            tool = "file_operations",
            input = {
                operation = "write",
                path = error_state_file,
                content = "{{step:update_count:output}}"
            }
        }
    },
    
    -- Initialize before loop
    on_start = function()
        local init_tool = Tool.getByName("file_operations")
        init_tool:execute({
            operation = "write",
            path = error_state_file,
            content = '{"success_count": 0, "error_count": 0}'
        })
    end,
    
    -- Continue processing despite errors
    error_strategy = "continue"
})

print("Executing error handling loop...")
local error_result = error_handling_loop:execute()

if error_result then
    -- Read final counts
    local read_tool = Tool.getByName("file_operations")
    local final_state = read_tool:execute({
        operation = "read",
        path = error_state_file
    })
    
    if final_state.success then
        local json_tool = Tool.getByName("json_processor")
        local success_count = json_tool:execute({
            json = final_state.output,
            query = "$.success_count"
        })
        local error_count = json_tool:execute({
            json = final_state.output,
            query = "$.error_count"
        })
        
        print("\nError handling results:")
        print("- Total items: " .. #data_with_errors)
        print("- Successful: " .. (success_count.output or "0"))
        print("- Errors: " .. (error_count.output or "0"))
    end
else
    print("Execution error: Unknown error")
end

-- Example 6: Performance-Optimized Loop
print("\n\nExample 6: Performance-Optimized Loop")
print("-" .. string.rep("-", 37))

-- Large dataset simulation
local large_dataset = {}
for i = 1, 100 do
    table.insert(large_dataset, i)
end

-- Batch results file
local batch_results_file = "/tmp/batch_results.json"

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
            name = "batch_to_json",
            type = "tool",
            tool = "json_processor",
            input = {
                json = "{{loop:current_item | to_json}}",
                query = "$"
            }
        },
        {
            name = "calculate_batch_sum",
            type = "tool",
            tool = "json_processor",
            input = {
                json = "{{step:batch_to_json:output}}",
                query = "$[*] | add"
            }
        },
        {
            name = "read_results",
            type = "tool",
            tool = "file_operations",
            input = {
                operation = "read",
                path = batch_results_file
            },
            on_error = function(err)
                return {
                    success = true,
                    output = '{"batches": []}'
                }
            end
        },
        {
            name = "append_batch_sum",
            type = "tool",
            tool = "json_processor",
            input = {
                json = "{{step:read_results:output}}",
                query = "$.batches",
                operation = "append",
                value = "{{step:calculate_batch_sum:output}}"
            }
        },
        {
            name = "save_results",
            type = "tool",
            tool = "file_operations",
            input = {
                operation = "write",
                path = batch_results_file,
                content = '{"batches": {{step:append_batch_sum:output}}}'
            }
        }
    },
    
    -- Initialize before loop
    on_start = function()
        local init_tool = Tool.getByName("file_operations")
        init_tool:execute({
            operation = "write",
            path = batch_results_file,
            content = '{"batches": []}'
        })
    end,
    
    -- Only collect summary
    aggregation_strategy = "summary"
})

local start_time = os.clock()
local perf_result = optimized_loop:execute()
local elapsed = (os.clock() - start_time) * 1000

if perf_result then
    print("Performance results:")
    print(string.format("- Processed %d items in %.2f ms", 
                        #large_dataset, elapsed))
    print(string.format("- Throughput: %.0f items/second", 
                        (#large_dataset / elapsed) * 1000))
else
    print("Execution error: Unknown error")
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