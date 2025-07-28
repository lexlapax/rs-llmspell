-- ABOUTME: Parallel workflow example demonstrating concurrent execution
-- ABOUTME: Shows how to use Workflow.parallel() for fork-join patterns

-- CONFIG: Use examples/configs/state-enabled.toml (or minimal.toml if State not needed)
-- WHY: This example uses State.save for storing intermediate results
-- HOW TO RUN: ./target/debug/llmspell -c examples/configs/state-enabled.toml run examples/lua/workflows/workflow-parallel.lua
-- ALTERNATIVE: cargo run -- -c examples/configs/state-enabled.toml run examples/lua/workflows/workflow-parallel.lua

-- Parallel Workflow Example
-- Demonstrates concurrent execution and result aggregation

-- Note: All workflow and tool methods are now synchronous - no helpers needed

print("=== Parallel Workflow Example ===\n")

-- Example 1: Basic Parallel Execution
print("Example 1: Basic Parallel Execution")
print("-" .. string.rep("-", 35))

local basic_parallel = Workflow.parallel({
    name = "basic_parallel_tasks",
    description = "Execute multiple independent tasks concurrently",
    
    branches = {
        -- Branch 1: File processing
        {
            name = "file_processor",
            steps = {
                {
                    name = "create_report",
                    type = "tool",
                    tool = "template_engine",
                    input = {
                        template = "Report generated at {{timestamp}}",
                        variables = {
                            timestamp = os.date("%Y-%m-%d %H:%M:%S")
                        }
                    }
                },
                {
                    name = "save_report",
                    type = "tool",
                    tool = "file_operations",
                    input = {
                        operation = "write",
                        path = "/tmp/parallel_report.txt",
                        content = "{{step:create_report:output}}"
                    }
                }
            }
        },
        
        -- Branch 2: Data calculation
        {
            name = "calculator_branch",
            steps = {
                {
                    name = "complex_calc",
                    type = "tool",
                    tool = "calculator",
                    input = { input = "(100 + 200) * 3 / 2" }
                },
                {
                    name = "format_result",
                    type = "tool",
                    tool = "text_manipulator",
                    input = {
                        input = "Calculation result: {{step:complex_calc:output}}",
                        operation = "uppercase"
                    }
                }
            }
        },
        
        -- Branch 3: System check
        {
            name = "system_checker",
            steps = {
                {
                    name = "check_environment",
                    type = "tool",
                    tool = "environment_reader",
                    input = { 
                        variables = {"PATH", "HOME", "USER"} 
                    }
                },
                {
                    name = "generate_id",
                    type = "tool",
                    tool = "uuid_generator",
                    input = { version = "v4" }
                }
            }
        }
    },
    
    -- Maximum concurrent branches
    max_concurrency = 3
})

print("Executing basic parallel workflow...")
local start_time = os.clock()
local basic_result = basic_parallel:execute()
local elapsed = (os.clock() - start_time) * 1000

if basic_result then
    print("Results:")
    print("- Success: " .. tostring(basic_result.success))
    print("- Branches executed: " .. (basic_result.data and basic_result.data.successful_branches or "N/A"))
    print("- Execution time: " .. string.format("%.2f ms", elapsed))
    print("- Speedup vs sequential: ~3x (estimated)")
else
    print("Execution error: Unknown error")
end

-- Example 2: Fork-Join Pattern
print("\n\nExample 2: Fork-Join Pattern")
print("-" .. string.rep("-", 28))

-- Simulate data that needs parallel processing
local data_chunks = {
    { id = "chunk1", data = {10, 20, 30, 40, 50} },
    { id = "chunk2", data = {15, 25, 35, 45, 55} },
    { id = "chunk3", data = {12, 22, 32, 42, 52} },
    { id = "chunk4", data = {18, 28, 38, 48, 58} }
}

State.save("global", "data_chunks", data_chunks)
State.save("global", "chunk_results", {})

local fork_join = Workflow.parallel({
    name = "fork_join_processor",
    description = "Process data chunks in parallel then join results",
    
    -- Dynamically create branches for each chunk
    branches = (function()
        local branches = {}
        for i, chunk in ipairs(data_chunks) do
            table.insert(branches, {
                name = "process_" .. chunk.id,
                steps = {
                    {
                        name = "save_chunk_data",
                        type = "tool",
                        tool = "file_operations",
                        input = {
                            operation = "write",
                            path = "/tmp/chunk_" .. chunk.id .. ".json",
                            content = '[' .. table.concat(chunk.data, ',') .. ']'
                        }
                    },
                    {
                        name = "read_chunk_data",
                        type = "tool",
                        tool = "file_operations",
                        input = {
                            operation = "read",
                            path = "/tmp/chunk_" .. chunk.id .. ".json"
                        }
                    },
                    {
                        name = "sum_chunk",
                        type = "tool",
                        tool = "json_processor",
                        input = {
                            input = '{{step:read_chunk_data:output}}',
                            query = 'add'
                        }
                    },
                    {
                        name = "save_sum_result",
                        type = "tool",
                        tool = "file_operations",
                        input = {
                            operation = "write",
                            path = "/tmp/sum_" .. chunk.id .. ".json",
                            content = '{"chunk_id": "' .. chunk.id .. '", "sum": {{step:sum_chunk:output}}}'
                        }
                    },
                    {
                        name = "calculate_average",
                        type = "tool",
                        tool = "calculator",
                        input = {
                            input = "{{step:sum_chunk:output}} / " .. #chunk.data
                        }
                    }
                }
            })
        end
        return branches
    end)(),
    
    -- Maximum concurrent chunks to process
    max_concurrency = 4
})

print("Executing fork-join pattern...")
local fork_join_result = fork_join:execute()

if fork_join_result then
    print("Fork-Join Results:")
    print("- Chunks processed: " .. #data_chunks)
    print("- Success: " .. tostring(fork_join_result.success))
    print("- Parallel processing completed")
else
    print("Execution error: Unknown error")
end

-- Example 3: Parallel with Dependencies
print("\n\nExample 3: Parallel with Required Branches")
print("-" .. string.rep("-", 42))

local dependency_parallel = Workflow.parallel({
    name = "critical_operations",
    description = "Some branches are required for success",
    
    branches = {
        -- Critical branch (required)
        {
            name = "critical_validation",
            required = true,  -- Must succeed for workflow to succeed
            steps = {
                {
                    name = "validate_critical",
                    type = "tool",
                    tool = "data_validation",
                    input = {
                        input = { status = "active", count = 10 },
                        schema = {
                            type = "object",
                            required = {"status", "count"},
                            properties = {
                                status = { type = "string" },
                                count = { type = "number", minimum = 0 }
                            }
                        }
                    }
                }
            }
        },
        
        -- Optional branch
        {
            name = "optional_enhancement",
            required = false,  -- Can fail without affecting overall success
            steps = {
                {
                    name = "attempt_read_optional_file",
                    type = "tool",
                    tool = "file_operations",
                    input = {
                        operation = "read",
                        path = "/tmp/optional_config_" .. os.time() .. ".json"  -- File that might not exist
                    }
                }
            }
        },
        
        -- Another critical branch
        {
            name = "critical_save",
            required = true,
            steps = {
                {
                    name = "save_data",
                    type = "tool",
                    tool = "file_operations",
                    input = {
                        operation = "write",
                        path = "/tmp/critical_data.json",
                        content = '{"status": "processed", "timestamp": "' .. 
                                 os.date("%Y-%m-%d %H:%M:%S") .. '"}'
                    }
                }
            }
        }
    },
    
    -- Stop all branches if a required branch fails
    fail_fast = true
})

print("Executing parallel workflow with dependencies...")
local dep_result = dependency_parallel:execute()

if dep_result then
    print("Results:")
    print("- Overall success: " .. tostring(dep_result.success))
    print("- Successful branches: " .. (dep_result.data and dep_result.data.successful_branches or "N/A") .. "/3")
    if dep_result.data and dep_result.data.failed_branches and dep_result.data.failed_branches > 0 then
        print("- Failed branches: " .. dep_result.data.failed_branches)
    end
else
    print("Execution error: Unknown error")
end

-- Example 4: Resource-Limited Parallel Execution
print("\n\nExample 4: Resource-Limited Parallel Execution")
print("-" .. string.rep("-", 46))

-- Simulate many tasks that need rate limiting
local many_tasks = {}
for i = 1, 10 do
    table.insert(many_tasks, {
        name = "task_" .. i,
        steps = {
            {
                name = "create_task_data",
                type = "tool",
                tool = "uuid_generator",
                input = { version = "v4" }
            },
            {
                name = "process_task",
                type = "tool",
                tool = "file_operations",
                input = {
                    operation = "write",
                    path = "/tmp/task_" .. i .. "_{{step:create_task_data:output}}.txt",
                    content = "Task " .. i .. " processed at " .. os.date("%Y-%m-%d %H:%M:%S")
                }
            },
            {
                name = "log_completion",
                type = "tool",
                tool = "template_engine",
                input = {
                    template = "Task {{task_id}} completed with UUID {{uuid}}",
                    variables = {
                        task_id = tostring(i),
                        uuid = "{{step:create_task_data:output}}"
                    }
                }
            }
        }
    })
end

local rate_limited = Workflow.parallel({
    name = "rate_limited_processor",
    description = "Process many tasks with concurrency limit",
    
    branches = many_tasks,
    
    -- Limit concurrent execution
    max_concurrency = 3,  -- Only 3 tasks run at once
    
    -- Maximum time to wait
    timeout = 10000  -- 10 seconds
})

print("Executing rate-limited parallel workflow (10 tasks, max 3 concurrent)...")
local rate_start = os.clock()
local rate_result = rate_limited:execute()
local rate_elapsed = (os.clock() - rate_start) * 1000

if rate_result then
    print("\nRate-limited execution completed:")
    print(string.format("- Total time: %.2f ms", rate_elapsed))
    print("- All tasks completed: " .. tostring(rate_result.success))
else
    print("Execution error: Unknown error")
end

-- Example 5: Map-Reduce Pattern
print("\n\nExample 5: Map-Reduce Pattern")
print("-" .. string.rep("-", 29))

-- Data to process in map-reduce style
local documents = {
    "The quick brown fox jumps over the lazy dog",
    "A journey of a thousand miles begins with a single step",
    "To be or not to be that is the question",
    "All that glitters is not gold"
}

State.save("global", "word_counts", {})

local map_reduce = Workflow.parallel({
    name = "word_count_mapreduce",
    description = "Count words across documents in parallel",
    
    -- Map phase: process each document in parallel
    branches = (function()
        local branches = {}
        for i, doc in ipairs(documents) do
            table.insert(branches, {
                name = "map_doc_" .. i,
                steps = {
                    {
                        name = "save_document",
                        type = "tool",
                        tool = "file_operations",
                        input = {
                            operation = "write",
                            path = "/tmp/doc_" .. i .. ".txt",
                            content = doc
                        }
                    },
                    {
                        name = "normalize_text",
                        type = "tool",
                        tool = "text_manipulator",
                        input = {
                            input = doc,
                            operation = "lowercase"
                        }
                    },
                    {
                        name = "process_words",
                        type = "tool",
                        tool = "text_manipulator",
                        input = {
                            input = "{{step:normalize_text:output}}",
                            operation = "replace",
                            pattern = "[^a-z0-9 ]+",
                            replacement = " "
                        }
                    },
                    {
                        name = "save_word_counts",
                        type = "tool",
                        tool = "file_operations",
                        input = {
                            operation = "write",
                            path = "/tmp/doc_words_" .. i .. ".txt",
                            content = "{{step:process_words:output}}"
                        }
                    }
                }
            })
        end
        return branches
    end)(),
    
    -- Reduce phase: combine results
    post_steps = {
        {
            name = "create_summary",
            type = "tool",
            tool = "template_engine",
            input = {
                template = "Processed {{doc_count}} documents in parallel. Results saved to individual files.",
                variables = {
                    doc_count = tostring(#documents)
                }
            }
        },
        {
            name = "save_summary",
            type = "tool",
            tool = "file_operations",
            input = {
                operation = "write",
                path = "/tmp/mapreduce_summary.txt",
                content = "{{step:create_summary:output}}"
            }
        }
    }
})

print("Executing map-reduce word count...")
local mapreduce_result = map_reduce:execute()

if mapreduce_result then
    print("Map-Reduce Results:")
    print("- Documents processed: " .. #documents)
    print("- Success: " .. tostring(mapreduce_result.success))
    print("- Results saved to individual word count files")
    
    -- Map-reduce completed successfully
else
    print("Execution error: Unknown error")
end

-- Performance comparison
print("\n\n=== Parallel Performance Analysis ===")

-- Sequential baseline
local seq_start = os.clock()
for i = 1, 3 do
    Tool.invoke("calculator", { input = "100 * 2" })
    Tool.invoke("uuid_generator", { version = "v4" })
    Tool.invoke("text_manipulator", { 
        input = "test", 
        operation = "uppercase" 
    })
end
local seq_time = (os.clock() - seq_start) * 1000

-- Parallel execution
local par_workflow = Workflow.parallel({
    name = "performance_test",
    branches = {
        { name = "b1", steps = {{ name = "s1", type = "tool", tool = "calculator", input = { input = "100 * 2" }}}},
        { name = "b2", steps = {{ name = "s2", type = "tool", tool = "uuid_generator", input = { version = "v4" }}}},
        { name = "b3", steps = {{ name = "s3", type = "tool", tool = "text_manipulator", input = { input = "test", operation = "uppercase" }}}}
    }
})

local par_start = os.clock()
par_workflow:execute()
local par_time = (os.clock() - par_start) * 1000

print(string.format("Sequential time: %.2f ms", seq_time))
print(string.format("Parallel time: %.2f ms", par_time))
print(string.format("Speedup: %.1fx", seq_time / par_time))

-- Summary
print("\n=== Parallel Workflow Summary ===")
print("Examples demonstrated:")
print("1. Basic parallel execution")
print("2. Fork-join pattern") 
print("3. Required/optional branches")
print("4. Rate-limited execution")
print("5. Map-reduce pattern")
print("\nKey features:")
print("- Concurrent branch execution")
print("- Max concurrency control")
print("- Required branch support")
print("- Post-processing steps")
print("- Performance optimization")

print("\n=== Parallel Workflow Example Complete ===")