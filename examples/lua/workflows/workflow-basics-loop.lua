-- ABOUTME: Basic loop workflow example using only tool steps
-- ABOUTME: Demonstrates iteration patterns without custom functions

-- Load workflow helpers for async execution
local helpers = dofile("examples/lua/workflows/workflow-helpers.lua")
-- Load tool helpers for async tool invocation
local tool_helpers = dofile("examples/lua/tools/tool-helpers.lua")

print("=== Basic Loop Workflow Example ===\n")

-- Example 1: Simple Range Loop
print("Example 1: Simple Range Loop")
print("-" .. string.rep("-", 28))

local range_loop = Workflow.loop({
    name = "range_processor",
    description = "Process numbers in a range",
    
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
            name = "square_number",
            type = "tool",
            tool = "calculator",
            input = {
                input = "{{loop:current_value}} * {{loop:current_value}}"
            }
        },
        {
            name = "format_result",
            type = "tool",
            tool = "template_engine",
            input = {
                template = "Number {{num}}: {{num}} squared = {{result}}",
                variables = {
                    num = "{{loop:current_value}}",
                    result = "{{step:square_number:output}}"
                }
            }
        },
        {
            name = "save_result",
            type = "tool",
            tool = "file_operations",
            input = {
                operation = "append",
                path = "/tmp/squares.txt",
                content = "{{step:format_result:output}}\n"
            }
        }
    },
    
    aggregation_strategy = "collect_all"
})

-- Clear output file
tool_helpers.invokeTool("file_operations", {
    operation = "write",
    path = "/tmp/squares.txt",
    content = "Square Numbers\n==============\n"
})

print("Executing range loop...")
local range_result, err = helpers.executeWorkflow(range_loop)

if range_result and range_result.success then
    print("✓ Range loop completed!")
    print("Iterations: " .. (range_result.data and range_result.data.completed_iterations or "N/A"))
    
    -- Read and display results
    local output = tool_helpers.invokeTool("file_operations", {
        operation = "read",
        path = "/tmp/squares.txt"
    })
    if output then
        print("\nOutput:\n" .. output.output)
    end
else
    print("✗ Range loop failed: " .. tostring(err))
end

-- Example 2: Collection Loop
print("\n\nExample 2: Collection Loop")
print("-" .. string.rep("-", 26))

-- Create a collection of items to process
local items = {
    { name = "Apple", price = 1.50, quantity = 10 },
    { name = "Banana", price = 0.75, quantity = 20 },
    { name = "Orange", price = 2.00, quantity = 15 },
    { name = "Grape", price = 3.50, quantity = 8 }
}

-- Items will be passed directly to the workflow

local collection_loop = Workflow.loop({
    name = "item_processor",
    description = "Calculate value for each item",
    
    -- Iterate over collection
    iterator = {
        collection = items
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
            name = "format_item",
            type = "tool",
            tool = "template_engine",
            input = {
                template = "{{name}}: ${{price}} × {{quantity}} = ${{total}}",
                variables = {
                    name = "{{loop:current_item.name}}",
                    price = "{{loop:current_item.price}}",
                    quantity = "{{loop:current_item.quantity}}",
                    total = "{{step:calculate_value:output}}"
                }
            }
        },
        {
            name = "append_to_report",
            type = "tool",
            tool = "file_operations",
            input = {
                operation = "append",
                path = "/tmp/inventory_values.txt",
                content = "{{step:format_item:output}}\n"
            }
        }
    }
})

-- Initialize report file
tool_helpers.invokeTool("file_operations", {
    operation = "write",
    path = "/tmp/inventory_values.txt",
    content = "Inventory Value Report\n=====================\n"
})

print("Processing collection items...")
local collection_result, err = helpers.executeWorkflow(collection_loop)

if collection_result and collection_result.success then
    print("✓ Collection processing completed!")
    print("Items processed: " .. (collection_result.data and collection_result.data.completed_iterations or "N/A"))
else
    print("✗ Collection processing failed: " .. tostring(err))
end

-- Example 3: Accumulator Pattern
print("\n\nExample 3: Accumulator Pattern")
print("-" .. string.rep("-", 30))

-- Create numbers to sum
local numbers = {10, 25, 15, 30, 20}

-- Numbers will be passed directly to the workflow

-- Initialize accumulator
tool_helpers.invokeTool("file_operations", {
    operation = "write",
    path = "/tmp/accumulator.txt",
    content = "0"
})

local accumulator_loop = Workflow.loop({
    name = "sum_calculator",
    description = "Calculate running sum",
    
    iterator = {
        collection = numbers
    },
    
    body = {
        -- Read current accumulator value
        {
            name = "read_sum",
            type = "tool",
            tool = "file_operations",
            input = {
                operation = "read",
                path = "/tmp/accumulator.txt"
            }
        },
        -- Add current number to sum
        {
            name = "add_to_sum",
            type = "tool",
            tool = "calculator",
            input = {
                input = "{{step:read_sum:output}} + {{loop:current_value}}"
            }
        },
        -- Save updated sum
        {
            name = "save_sum",
            type = "tool",
            tool = "file_operations",
            input = {
                operation = "write",
                path = "/tmp/accumulator.txt",
                content = "{{step:add_to_sum:output}}"
            }
        },
        -- Log progress
        {
            name = "log_progress",
            type = "tool",
            tool = "template_engine",
            input = {
                template = "Added {{num}}: Running total = {{total}}",
                variables = {
                    num = "{{loop:current_value}}",
                    total = "{{step:add_to_sum:output}}"
                }
            }
        }
    }
})

print("Calculating sum with accumulator...")
local acc_result, err = helpers.executeWorkflow(accumulator_loop)

if acc_result and acc_result.success then
    print("✓ Accumulator loop completed!")
    
    -- Read final sum
    local final_sum = tool_helpers.invokeTool("file_operations", {
        operation = "read",
        path = "/tmp/accumulator.txt"
    })
    if final_sum then
        print("Final sum: " .. final_sum.output)
        
        -- Expected sum: 10 + 25 + 15 + 30 + 20 = 100
        print("Expected sum: 100")
    end
else
    print("✗ Accumulator loop failed: " .. tostring(err))
end

-- Example 4: Filtered Processing
print("\n\nExample 4: Filtered Processing")
print("-" .. string.rep("-", 30))

-- Create data with mixed types
local mixed_data = {
    { type = "product", name = "Widget", value = 100 },
    { type = "service", name = "Support", value = 50 },
    { type = "product", name = "Gadget", value = 150 },
    { type = "service", name = "Training", value = 200 },
    { type = "product", name = "Tool", value = 75 }
}

-- Process only products
local filter_loop = Workflow.loop({
    name = "product_processor",
    description = "Process only product items",
    
    iterator = {
        collection = mixed_data
    },
    
    body = {
        -- Check if current item is a product
        {
            name = "check_type",
            type = "tool",
            tool = "json_processor",
            input = {
                operation = "query",
                input = '{"type": "{{loop:current_item.type}}"}',
                query = '.type == "product"'
            }
        },
        -- Process only if it's a product (using conditional within loop)
        {
            name = "process_product",
            type = "tool",
            tool = "template_engine",
            input = {
                template = "Product: {{name}} (Value: ${{value}})",
                variables = {
                    name = "{{loop:current_item.name}}",
                    value = "{{loop:current_item.value}}"
                }
            },
            skip_condition = {
                tool = "json_processor",
                input = {
                    operation = "query",
                    input = "{{step:check_type:output}}",
                    query = '. == false'
                }
            }
        },
        {
            name = "save_product",
            type = "tool",
            tool = "file_operations",
            input = {
                operation = "append",
                path = "/tmp/products_only.txt",
                content = "{{step:process_product:output}}\n"
            },
            skip_condition = {
                tool = "json_processor",
                input = {
                    operation = "query",
                    input = "{{step:check_type:output}}",
                    query = '. == false'
                }
            }
        }
    }
})

-- Initialize output file
tool_helpers.invokeTool("file_operations", {
    operation = "write",
    path = "/tmp/products_only.txt",
    content = "Products List\n============\n"
})

print("Processing filtered items...")
local filter_result, err = helpers.executeWorkflow(filter_loop)

if filter_result and filter_result.success then
    print("✓ Filtered processing completed!")
    
    -- Show results
    local products = tool_helpers.invokeTool("file_operations", {
        operation = "read",
        path = "/tmp/products_only.txt"
    })
    if products then
        print("\n" .. products.output)
    end
else
    print("✗ Filtered processing failed: " .. tostring(err))
end

-- Example 5: Batch Processing
print("\n\nExample 5: Batch Processing")
print("-" .. string.rep("-", 27))

-- Create a larger dataset
local large_dataset = {}
for i = 1, 20 do
    table.insert(large_dataset, {
        id = i,
        value = math.random(10, 100)
    })
end

-- Process in batches of 5
local batch_size = 5
local batch_loop = Workflow.loop({
    name = "batch_processor",
    description = "Process data in batches",
    
    iterator = {
        range = {
            start = 1,
            ["end"] = #large_dataset,
            step = batch_size
        }
    },
    
    body = {
        -- Create batch slice
        {
            name = "create_batch",
            type = "tool",
            tool = "json_processor",
            input = {
                operation = "query",
                input = '[]',  -- Placeholder, will be replaced dynamically
                query = string.format('.[({{loop:current_value}}-1):({{loop:current_value}}+%d-1)]', batch_size)
            }
        },
        -- Process batch
        {
            name = "batch_average",
            type = "tool",
            tool = "json_processor",
            input = {
                operation = "query",
                input = "{{step:create_batch:output}}",
                query = 'map(.value) | add / length'
            }
        },
        -- Format result
        {
            name = "batch_report",
            type = "tool",
            tool = "template_engine",
            input = {
                template = "Batch starting at {{start}}: Average value = {{avg}}",
                variables = {
                    start = "{{loop:current_value}}",
                    avg = "{{step:batch_average:output}}"
                }
            }
        }
    }
})

print("Processing in batches...")
local batch_result, err = helpers.executeWorkflow(batch_loop)

if batch_result and batch_result.success then
    print("✓ Batch processing completed!")
    print("Batches processed: " .. (batch_result.data and batch_result.data.completed_iterations or "N/A"))
else
    print("✗ Batch processing failed: " .. tostring(err))
end

-- Summary
print("\n\n=== Basic Loop Workflow Summary ===")
print("Key concepts demonstrated:")
print("1. Range-based loops for numeric iteration")
print("2. Collection-based loops for arrays/objects")
print("3. Accumulator pattern using file storage")
print("4. Filtered processing with skip conditions")
print("5. Batch processing for large datasets")
print("\nAll operations use standard tools - no custom functions!")

print("\n=== Example Complete ===")