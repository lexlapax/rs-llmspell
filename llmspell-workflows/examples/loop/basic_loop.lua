-- ABOUTME: Basic loop workflow examples demonstrating iteration patterns
-- ABOUTME: Shows range, collection, and condition-based loops with tool integration

-- Range-based loop workflow
-- Process a fixed number of items
local range_workflow = Workflow.loop({
    name = "batch_processor",
    description = "Process items in batches using range iteration",
    
    -- Define the iteration range
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
            name = "generate_id",
            type = "tool",
            tool = "uuid_generator",
            input = { version = "v4" }
        },
        {
            name = "process_item",
            type = "tool",
            tool = "template_engine",
            input = {
                template = "Processing item {{index}} with ID: {{id}}",
                variables = {
                    index = "{{loop:current_value}}",
                    id = "{{step:generate_id:output}}"
                }
            }
        },
        {
            name = "calculate_progress",
            type = "tool",
            tool = "calculator",
            input = {
                input = "{{loop:current_value}} / {{loop:total_iterations}} * 100"
            }
        }
    },
    
    -- Aggregate results
    aggregation_strategy = "collect_all",
    error_strategy = "continue"
})

print("Executing range-based loop workflow...")
local range_result = range_workflow:execute()
print("Processed " .. range_result.data.completed_iterations .. " items")

-- Collection-based loop workflow
-- Process each item in a collection
local files_to_process = {
    "report_january.csv",
    "report_february.csv", 
    "report_march.csv",
    "report_april.csv"
}

State.set("files_collection", files_to_process)

local collection_workflow = Workflow.loop({
    name = "file_processor",
    description = "Process multiple files in sequence",
    
    -- Iterate over collection
    iterator = {
        collection = State.get("files_collection")
    },
    
    body = {
        {
            name = "check_file",
            type = "custom",
            execute = function(context)
                local filename = context.current_item
                print("Checking file: " .. filename)
                return {
                    success = true,
                    output = filename
                }
            end
        },
        {
            name = "extract_month",
            type = "tool",
            tool = "text_manipulator",
            input = {
                input = "{{loop:current_item}}",
                operation = "extract",
                pattern = "report_(\\w+)\\.csv"
            }
        },
        {
            name = "process_data",
            type = "tool",
            tool = "template_engine",
            input = {
                template = "Processing {{month}} data from {{file}}",
                variables = {
                    month = "{{step:extract_month:output}}",
                    file = "{{loop:current_item}}"
                }
            }
        },
        {
            name = "generate_hash",
            type = "tool",
            tool = "hash_calculator",
            input = {
                input = "{{loop:current_item}}",
                algorithm = "md5"
            }
        }
    },
    
    -- Store results in state
    on_iteration_complete = function(index, results)
        local processed = State.get("processed_files") or {}
        table.insert(processed, {
            file = results.current_item,
            hash = results.steps.generate_hash.output,
            index = index
        })
        State.set("processed_files", processed)
    end
})

print("\nExecuting collection-based loop workflow...")
local collection_result = collection_workflow:execute()
print("Files processed: " .. collection_result.data.completed_iterations)

-- While condition loop workflow
-- Continue until condition is false
local while_workflow = Workflow.loop({
    name = "accumulator",
    description = "Accumulate values until threshold reached",
    
    -- Continue while condition is true
    iterator = {
        while_condition = {
            type = "shared_data_less_than",
            key = "total_sum",
            value = 1000
        }
    },
    
    body = {
        {
            name = "generate_value",
            type = "tool",
            tool = "calculator",
            input = {
                -- Generate random value between 50-150
                input = "50 + {{random}} * 100"
            }
        },
        {
            name = "add_to_total",
            type = "custom",
            execute = function(context)
                local current_sum = State.get("total_sum") or 0
                local new_value = tonumber(context.steps.generate_value.output) or 0
                local new_sum = current_sum + new_value
                
                State.set("total_sum", new_sum)
                State.set("last_value", new_value)
                
                return {
                    success = true,
                    output = "Added " .. new_value .. ", total: " .. new_sum
                }
            end
        }
    },
    
    -- Safety limit
    max_iterations = 50,
    error_strategy = "fail_fast"
})

-- Initialize state
State.set("total_sum", 0)

print("\nExecuting while condition loop workflow...")
local while_result = while_workflow:execute()
print("Final sum: " .. State.get("total_sum"))
print("Iterations: " .. while_result.data.completed_iterations)

-- Advanced loop: Data transformation pipeline
local transform_workflow = Workflow.loop({
    name = "data_transformer",
    description = "Transform and validate data records",
    
    iterator = {
        collection = {
            { name = "Alice", age = 30, department = "Engineering" },
            { name = "Bob", age = 25, department = "Marketing" },
            { name = "Charlie", age = 35, department = "Sales" },
            { name = "Diana", age = 28, department = "Engineering" }
        }
    },
    
    body = {
        -- Validate record
        {
            name = "validate_record",
            type = "tool",
            tool = "data_validation",
            input = {
                input = "{{loop:current_item}}",
                schema = {
                    type = "object",
                    required = {"name", "age", "department"},
                    properties = {
                        name = { type = "string" },
                        age = { type = "number", minimum = 18 },
                        department = { type = "string" }
                    }
                }
            }
        },
        
        -- Generate employee ID
        {
            name = "generate_id",
            type = "tool",
            tool = "template_engine",
            input = {
                template = "EMP-{{dept_code}}-{{timestamp}}",
                variables = {
                    dept_code = "{{loop:current_item.department:substring:0:3:uppercase}}",
                    timestamp = "{{timestamp:format:YYYYMMDD}}"
                }
            }
        },
        
        -- Calculate benefits
        {
            name = "calculate_benefits",
            type = "tool",
            tool = "calculator",
            input = {
                input = "{{loop:current_item.age}} > 30 ? 5000 : 3000"
            }
        },
        
        -- Create enriched record
        {
            name = "enrich_record",
            type = "tool",
            tool = "json_processor",
            input = {
                operation = "merge",
                input = "{{loop:current_item}}",
                merge_data = {
                    employee_id = "{{step:generate_id:output}}",
                    benefits = "{{step:calculate_benefits:output}}",
                    processed_at = "{{timestamp}}"
                }
            }
        }
    },
    
    -- Collect all enriched records
    aggregation_strategy = "collect_outputs",
    
    -- Continue on validation errors
    error_strategy = "continue",
    
    -- Break condition - stop if we find a specific department
    break_condition = {
        type = "step_output_contains",
        step_name = "enrich_record",
        substring = "Finance"
    }
})

print("\nExecuting data transformation loop workflow...")
local transform_result = transform_workflow:execute()
print("Records processed: " .. transform_result.data.completed_iterations)
print("Aggregated results available: " .. 
    (#transform_result.data.aggregated_results > 0 and "Yes" or "No"))