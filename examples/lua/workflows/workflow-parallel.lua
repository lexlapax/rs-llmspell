-- ABOUTME: Parallel workflow example demonstrating concurrent execution
-- ABOUTME: Shows how to use Workflow.parallel() for fork-join patterns

-- Parallel Workflow Example
-- Demonstrates concurrent execution and result aggregation

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

print("Results:")
print("- Success: " .. tostring(basic_result.success))
print("- Branches executed: " .. basic_result.data.successful_branches)
print("- Execution time: " .. string.format("%.2f ms", elapsed))
print("- Speedup vs sequential: ~3x (estimated)")

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

State.set("data_chunks", data_chunks)
State.set("chunk_results", {})

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
                        name = "sum_chunk",
                        type = "custom",
                        execute = function()
                            local sum = 0
                            for _, val in ipairs(chunk.data) do
                                sum = sum + val
                            end
                            return {
                                success = true,
                                output = { chunk_id = chunk.id, sum = sum }
                            }
                        end
                    },
                    {
                        name = "calculate_average",
                        type = "tool",
                        tool = "calculator",
                        input = {
                            input = string.format("%d / %d", 
                                "{{step:sum_chunk:output.sum}}", 
                                #chunk.data)
                        }
                    }
                }
            })
        end
        return branches
    end)(),
    
    -- Join results after parallel execution
    on_complete = function(result)
        -- Aggregate results from all branches
        local total_sum = 0
        local total_count = 0
        
        for _, branch in ipairs(result.branches) do
            if branch.success then
                -- Extract sum from first step
                local sum_step = branch.steps[1]
                if sum_step and sum_step.output then
                    total_sum = total_sum + sum_step.output.sum
                    total_count = total_count + #data_chunks[1].data
                end
            end
        end
        
        -- Store aggregated results (would use State in Phase 5)
        fork_join_result_data = {
            total_sum = total_sum,
            total_count = total_count,
            average = total_sum / total_count
        }
    end
})

print("Executing fork-join pattern...")
local fork_join_result = fork_join:execute()

print("Fork-Join Results:")
print("- Chunks processed: " .. #data_chunks)
if fork_join_result_data then
    print("- Total sum: " .. fork_join_result_data.total_sum)
    print("- Overall average: " .. string.format("%.2f", fork_join_result_data.average))
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
                    name = "enhance_data",
                    type = "custom",
                    execute = function()
                        -- Simulate 50% failure rate
                        if math.random() > 0.5 then
                            return { success = true, output = "Enhanced successfully" }
                        else
                            error("Enhancement failed (simulated)")
                        end
                    end
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

print("Results:")
print("- Overall success: " .. tostring(dep_result.success))
print("- Successful branches: " .. dep_result.data.successful_branches .. "/" .. 
      #dependency_parallel.branches)
if dep_result.data.failed_branches > 0 then
    print("- Failed branches: " .. dep_result.data.failed_branches)
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
                name = "process_task",
                type = "custom",
                execute = function()
                    -- Simulate varying processing time
                    local processing_time = math.random(100, 300) / 1000
                    os.execute("sleep " .. processing_time)
                    
                    return {
                        success = true,
                        output = string.format("Task %d completed in %.0fms", 
                                             i, processing_time * 1000)
                    }
                end
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
    
    -- Track progress
    on_branch_complete = function(branch_name, result)
        print("Completed: " .. branch_name)
    end
})

print("Executing rate-limited parallel workflow (10 tasks, max 3 concurrent)...")
local rate_start = os.clock()
local rate_result = rate_limited:execute()
local rate_elapsed = (os.clock() - rate_start) * 1000

print("\nRate-limited execution completed:")
print(string.format("- Total time: %.2f ms", rate_elapsed))
print("- All tasks completed: " .. tostring(rate_result.success))

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

State.set("word_counts", {})

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
                        name = "count_words",
                        type = "custom",
                        execute = function()
                            local words = {}
                            for word in doc:gmatch("%w+") do
                                word = word:lower()
                                words[word] = (words[word] or 0) + 1
                            end
                            return {
                                success = true,
                                output = words
                            }
                        end
                    }
                }
            })
        end
        return branches
    end)(),
    
    -- Reduce phase: combine results
    post_steps = {
        {
            name = "reduce_counts",
            type = "custom",
            execute = function(context)
                local global_counts = {}
                
                -- Combine word counts from all branches
                for _, branch_result in ipairs(context.branch_results) do
                    if branch_result.success then
                        local word_counts = branch_result.steps[1].output
                        for word, count in pairs(word_counts) do
                            global_counts[word] = (global_counts[word] or 0) + count
                        end
                    end
                end
                
                -- Find top words
                local sorted_words = {}
                for word, count in pairs(global_counts) do
                    table.insert(sorted_words, {word = word, count = count})
                end
                table.sort(sorted_words, function(a, b) return a.count > b.count end)
                
                -- Store word count results (would use State in Phase 5)
                word_count_results = {
                    total_words = #sorted_words,
                    top_words = {sorted_words[1], sorted_words[2], sorted_words[3]}
                }
                
                return {
                    success = true,
                    output = "Reduce completed"
                }
            end
        }
    }
})

print("Executing map-reduce word count...")
local mapreduce_result = map_reduce:execute()

print("Map-Reduce Results:")
print("- Documents processed: " .. #documents)
if word_count_results then
    print("- Unique words: " .. word_count_results.total_words)
end
print("- Top words:")
for i, word_data in ipairs(word_results.top_words or {}) do
    print(string.format("  %d. '%s' - %d occurrences", 
                        i, word_data.word, word_data.count))
end

-- Performance comparison
print("\n\n=== Parallel Performance Analysis ===")

-- Sequential baseline
local seq_start = os.clock()
for i = 1, 3 do
    Tools.get("calculator"):execute({ input = "100 * 2" })
    Tools.get("uuid_generator"):execute({ version = "v4" })
    Tools.get("text_manipulator"):execute({ 
        input = "test", 
        operation = "uppercase" 
    })
end
local seq_time = (os.clock() - seq_start) * 1000

-- Parallel execution
local par_workflow = Workflow.parallel({
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