-- ABOUTME: Basic parallel workflow example using only tool steps
-- ABOUTME: Demonstrates concurrent execution without custom functions

-- Note: All workflow and tool methods are now synchronous - no helpers needed

print("=== Basic Parallel Workflow Example ===\n")

-- Example 1: Simple Parallel Tool Execution
print("Example 1: Simple Parallel Tool Execution")
print("-" .. string.rep("-", 41))

local simple_parallel = Workflow.parallel({
    name = "parallel_tools",
    description = "Execute multiple tools concurrently",
    
    branches = {
        -- Branch 1: Generate IDs
        {
            name = "id_generation",
            steps = {
                {
                    name = "uuid1",
                    tool = "uuid_generator",
                    input = { version = "v4" }
                },
                {
                    name = "uuid2",
                    tool = "uuid_generator",
                    input = { version = "v4" }
                }
            }
        },
        -- Branch 2: Time operations
        {
            name = "time_operations",
            steps = {
                {
                    name = "current_time",
                    tool = "date_time_handler",
                    input = {
                        operation = "now",
                        format = "ISO8601"
                    }
                },
                {
                    name = "unix_timestamp",
                    tool = "date_time_handler",
                    input = {
                        operation = "now",
                        format = "unix"
                    }
                }
            }
        },
        -- Branch 3: Calculations
        {
            name = "calculations",
            steps = {
                {
                    name = "calc1",
                    tool = "calculator",
                    input = { input = "100 * 2.5 + 50" }
                },
                {
                    name = "calc2",
                    tool = "calculator",
                    input = { input = "sqrt(144) + pow(2, 3)" }
                }
            }
        }
    },
    
    max_concurrency = 3
})

print("Executing parallel tools...")
local start_time = os.clock()
local result = simple_parallel:execute()
local elapsed = (os.clock() - start_time) * 1000

if result and result.success then
    print("✓ Parallel execution completed in " .. string.format("%.2f ms", elapsed))
    print("Branches executed: " .. (result.data and result.data.successful_branches or "N/A"))
else
    print("✗ Parallel execution failed: " .. tostring(result and result.error or "Unknown error"))
end

-- Example 2: Parallel Data Processing
print("\n\nExample 2: Parallel Data Processing")
print("-" .. string.rep("-", 35))

-- Create multiple data files to process
local datasets = {
    sales = {
        { product = "Widget", amount = 100 },
        { product = "Gadget", amount = 200 },
        { product = "Tool", amount = 150 }
    },
    inventory = {
        { item = "Widget", stock = 50 },
        { item = "Gadget", stock = 30 },
        { item = "Tool", stock = 75 }
    },
    customers = {
        { name = "Alice", purchases = 5 },
        { name = "Bob", purchases = 3 },
        { name = "Charlie", purchases = 8 }
    }
}

-- Save datasets
for name, data in pairs(datasets) do
    local json = Tool.invoke("json_processor", {
        operation = "format",
        input = data,
        pretty = true
    })
    if json and json.output then
        Tool.invoke("file_operations", {
            operation = "write",
            path = "/tmp/" .. name .. "_data.json",
            content = json.output
        })
    end
end

local data_parallel = Workflow.parallel({
    name = "parallel_data_processing",
    description = "Process multiple datasets concurrently",
    
    branches = {
        -- Process sales data
        {
            name = "sales_analysis",
            steps = {
                {
                    name = "read_sales",
                    tool = "file_operations",
                    input = {
                        operation = "read",
                        path = "/tmp/sales_data.json"
                    }
                },
                {
                    name = "total_sales",
                    tool = "json_processor",
                    input = {
                        operation = "query",
                        input = "{{step:read_sales:output}}",
                        query = 'map(.amount) | add'
                    }
                },
                {
                    name = "sales_report",
                    tool = "template_engine",
                    input = {
                        template = "Total Sales: ${{total}}",
                        variables = {
                            total = "{{step:total_sales:output}}"
                        }
                    }
                }
            }
        },
        -- Process inventory data
        {
            name = "inventory_analysis",
            steps = {
                {
                    name = "read_inventory",
                    tool = "file_operations",
                    input = {
                        operation = "read",
                        path = "/tmp/inventory_data.json"
                    }
                },
                {
                    name = "total_stock",
                    tool = "json_processor",
                    input = {
                        operation = "query",
                        input = "{{step:read_inventory:output}}",
                        query = 'map(.stock) | add'
                    }
                },
                {
                    name = "inventory_report",
                    tool = "template_engine",
                    input = {
                        template = "Total Stock: {{total}} units",
                        variables = {
                            total = "{{step:total_stock:output}}"
                        }
                    }
                }
            }
        },
        -- Process customer data
        {
            name = "customer_analysis",
            steps = {
                {
                    name = "read_customers",
                    tool = "file_operations",
                    input = {
                        operation = "read",
                        path = "/tmp/customers_data.json"
                    }
                },
                {
                    name = "avg_purchases",
                    tool = "json_processor",
                    input = {
                        operation = "query",
                        input = "{{step:read_customers:output}}",
                        query = 'map(.purchases) | add / length'
                    }
                },
                {
                    name = "customer_report",
                    tool = "template_engine",
                    input = {
                        template = "Average Purchases: {{avg}}",
                        variables = {
                            avg = "{{step:avg_purchases:output}}"
                        }
                    }
                }
            }
        }
    }
})

print("Processing multiple datasets in parallel...")
local data_result = data_parallel:execute()

if data_result and data_result.success then
    print("✓ All datasets processed successfully!")
    print("Successful branches: " .. (data_result.data and data_result.data.successful_branches or "0"))
else
    print("✗ Data processing failed: " .. tostring(data_result and data_result.error or "Unknown error"))
end

-- Example 3: Parallel File Operations
print("\n\nExample 3: Parallel File Operations")
print("-" .. string.rep("-", 35))

local file_parallel = Workflow.parallel({
    name = "parallel_file_ops",
    description = "Perform multiple file operations concurrently",
    
    branches = {
        -- Create multiple report files
        {
            name = "daily_report",
            steps = {
                {
                    name = "generate_daily",
                    tool = "template_engine",
                    input = {
                        template = "Daily Report - {{date}}\n=================\nStatus: All systems operational",
                        variables = {
                            date = os.date("%Y-%m-%d")
                        }
                    }
                },
                {
                    name = "save_daily",
                    tool = "file_operations",
                    input = {
                        operation = "write",
                        path = "/tmp/daily_report.txt",
                        content = "{{step:generate_daily:output}}"
                    }
                }
            }
        },
        {
            name = "weekly_summary",
            steps = {
                {
                    name = "generate_weekly",
                    tool = "template_engine",
                    input = {
                        template = "Weekly Summary\n==============\nWeek of {{date}}\nTasks completed: 42",
                        variables = {
                            date = os.date("%Y-%m-%d")
                        }
                    }
                },
                {
                    name = "save_weekly",
                    tool = "file_operations",
                    input = {
                        operation = "write",
                        path = "/tmp/weekly_summary.txt",
                        content = "{{step:generate_weekly:output}}"
                    }
                }
            }
        },
        {
            name = "metrics_log",
            steps = {
                {
                    name = "calculate_metrics",
                    tool = "calculator",
                    input = { input = "95.5 + 98.2 + 97.1 + 99.0" }  -- Simulated metrics
                },
                {
                    name = "format_metrics",
                    tool = "template_engine",
                    input = {
                        template = "System Metrics\n=============\nTotal Score: {{score}}\nAverage: {{avg}}",
                        variables = {
                            score = "{{step:calculate_metrics:output}}",
                            avg = "{{step:calculate_metrics:output}} / 4"
                        }
                    }
                },
                {
                    name = "save_metrics",
                    tool = "file_operations",
                    input = {
                        operation = "write",
                        path = "/tmp/metrics_log.txt",
                        content = "{{step:format_metrics:output}}"
                    }
                }
            }
        }
    }
})

print("Creating multiple reports in parallel...")
local file_result = file_parallel:execute()

if file_result and file_result.success then
    print("✓ All reports created successfully!")
    local report_files = {"/tmp/daily_report.txt", "/tmp/weekly_summary.txt", "/tmp/metrics_log.txt"}
    print("Files created:")
    for _, file in ipairs(report_files) do
        print("  - " .. file)
    end
else
    print("✗ Report creation failed: " .. tostring(file_result and file_result.error or "Unknown error"))
end

-- Example 4: Map-Reduce Pattern
print("\n\nExample 4: Map-Reduce Pattern")
print("-" .. string.rep("-", 29))

-- Create data chunks
local chunks = {
    { 10, 20, 30 },
    { 15, 25, 35 },
    { 5, 15, 25 },
    { 20, 30, 40 }
}

-- Save chunks
for i, chunk in ipairs(chunks) do
    local json = Tool.invoke("json_processor", {
        operation = "format",
        input = chunk
    })
    if json then
        Tool.invoke("file_operations", {
            operation = "write",
            path = "/tmp/chunk_" .. i .. ".json",
            content = json.output
        })
    end
end

-- Create parallel branches for each chunk
local map_branches = {}
for i = 1, #chunks do
    table.insert(map_branches, {
        name = "process_chunk_" .. i,
        steps = {
            {
                name = "read_chunk",
                tool = "file_operations",
                input = {
                    operation = "read",
                    path = "/tmp/chunk_" .. i .. ".json"
                }
            },
            {
                name = "sum_chunk",
                tool = "json_processor",
                input = {
                    operation = "query",
                    input = "{{step:read_chunk:output}}",
                    query = 'add'  -- Sum all numbers in array
                }
            },
            {
                name = "save_result",
                tool = "file_operations",
                input = {
                    operation = "write",
                    path = "/tmp/sum_" .. i .. ".txt",
                    content = "{{step:sum_chunk:output}}"
                }
            }
        }
    })
end

local map_reduce = Workflow.parallel({
    name = "map_reduce_sum",
    description = "Process chunks in parallel then combine",
    branches = map_branches
})

print("Executing map phase (parallel processing)...")
local map_result = map_reduce:execute()

if map_result and map_result.success then
    print("✓ Map phase completed!")
    
    -- Reduce phase - combine results
    print("\nExecuting reduce phase...")
    local total = 0
    for i = 1, #chunks do
        local result = Tool.invoke("file_operations", {
            operation = "read",
            path = "/tmp/sum_" .. i .. ".txt"
        })
        if result and result.output then
            total = total + tonumber(result.output)
        end
    end
    
    print("✓ Reduce phase completed!")
    print("Final sum: " .. total)
else
    print("✗ Map phase failed: " .. tostring(map_result and map_result.error or "Unknown error"))
end

-- Summary
print("\n\n=== Basic Parallel Workflow Summary ===")
print("Key concepts demonstrated:")
print("1. Branches execute concurrently for speed")
print("2. Each branch can have multiple sequential steps")
print("3. Max concurrency can be controlled")
print("4. Map-reduce patterns using parallel + sequential")
print("\nAll operations use standard tools - no custom functions!")

print("\n=== Example Complete ===")