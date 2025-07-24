-- ABOUTME: Basic parallel workflow examples demonstrating concurrent execution
-- ABOUTME: Shows fork-join patterns, concurrent tool execution, and result aggregation

-- Basic parallel workflow - Multiple independent tasks
local parallel_tasks = Workflow.parallel({
    name = "concurrent_processor",
    description = "Execute multiple independent tasks concurrently",
    
    branches = {
        -- Branch 1: File processing
        {
            name = "file_processor",
            steps = {
                {
                    name = "create_file",
                    type = "tool",
                    tool = "file_operations",
                    input = {
                        operation = "write",
                        path = "/tmp/parallel_test_1.txt",
                        content = "Processing in parallel branch 1"
                    }
                },
                {
                    name = "hash_file",
                    type = "tool",
                    tool = "hash_calculator",
                    input = {
                        input = "{{step:create_file:output.content}}",
                        algorithm = "sha256"
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
                    input = { input = "(100 * 25) + (30 / 6) - 15" }
                },
                {
                    name = "format_result",
                    type = "tool",
                    tool = "template_engine",
                    input = {
                        template = "Calculation result: {{result}}",
                        variables = {
                            result = "{{step:complex_calc:output}}"
                        }
                    }
                }
            }
        },
        
        -- Branch 3: UUID generation
        {
            name = "id_generator",
            steps = {
                {
                    name = "gen_uuid_v4",
                    type = "tool",
                    tool = "uuid_generator",
                    input = { version = "v4" }
                },
                {
                    name = "gen_uuid_v5",
                    type = "tool",
                    tool = "uuid_generator",
                    input = { 
                        version = "v5",
                        namespace = "6ba7b810-9dad-11d1-80b4-00c04fd430c8",
                        name = "parallel_test"
                    }
                }
            }
        }
    },
    
    -- Execute all branches concurrently
    max_concurrency = 3,
    error_strategy = "continue"  -- Continue other branches if one fails
})

print("Executing basic parallel workflow...")
local parallel_result = parallel_tasks:execute()
print("Successful branches: " .. parallel_result.data.successful_branches)
print("Total duration: " .. parallel_result.duration_ms .. "ms")

-- Parallel data fetching workflow
-- Simulate fetching data from multiple sources concurrently
local data_fetcher = Workflow.parallel({
    name = "multi_source_fetcher",
    description = "Fetch data from multiple sources in parallel",
    
    branches = {
        -- Database query simulation
        {
            name = "database_query",
            required = true,  -- This branch must succeed
            steps = {
                {
                    name = "query_users",
                    type = "custom",
                    execute = function()
                        -- Simulate database query
                        return {
                            success = true,
                            output = {
                                users = 1250,
                                active = 980,
                                new_today = 15
                            }
                        }
                    end
                },
                {
                    name = "format_db_result",
                    type = "tool",
                    tool = "json_processor",
                    input = {
                        input = "{{step:query_users:output}}",
                        operation = "stringify"
                    }
                }
            }
        },
        
        -- API call simulation
        {
            name = "api_fetch",
            required = true,
            steps = {
                {
                    name = "fetch_weather",
                    type = "custom",
                    execute = function()
                        -- Simulate API call
                        return {
                            success = true,
                            output = {
                                temperature = 72,
                                condition = "Sunny",
                                humidity = 45
                            }
                        }
                    end
                },
                {
                    name = "process_weather",
                    type = "tool",
                    tool = "template_engine",
                    input = {
                        template = "Current: {{temp}}°F, {{condition}}",
                        variables = {
                            temp = "{{step:fetch_weather:output.temperature}}",
                            condition = "{{step:fetch_weather:output.condition}}"
                        }
                    }
                }
            }
        },
        
        -- File system check
        {
            name = "filesystem_check",
            required = false,  -- Optional branch
            steps = {
                {
                    name = "check_disk",
                    type = "tool",
                    tool = "system_monitor",
                    input = { metrics = {"disk"} }
                },
                {
                    name = "check_files",
                    type = "tool",
                    tool = "file_operations",
                    input = {
                        operation = "list",
                        path = "/tmp"
                    }
                }
            }
        }
    },
    
    -- Aggregate results from all branches
    on_complete = function(results)
        local aggregated = {
            database = results.branches.database_query,
            api = results.branches.api_fetch,
            filesystem = results.branches.filesystem_check,
            total_time = results.duration_ms
        }
        State.set("aggregated_data", aggregated)
    end,
    
    fail_fast = true  -- Stop all branches if a required branch fails
})

print("\n\nExecuting parallel data fetching...")
local fetch_result = data_fetcher:execute()
print("Data fetching completed: " .. (fetch_result.success and "Success" or "Failed"))

-- Map-Reduce style parallel workflow
-- Process data in parallel then reduce results
local map_reduce = Workflow.parallel({
    name = "map_reduce_processor",
    description = "Parallel map operations followed by reduction",
    
    branches = {
        -- Map operations on different data chunks
        {
            name = "chunk_1",
            steps = {
                {
                    name = "process_chunk",
                    type = "tool",
                    tool = "calculator",
                    input = { input = "sum([10, 20, 30, 40])" }  -- Sum: 100
                }
            }
        },
        {
            name = "chunk_2",
            steps = {
                {
                    name = "process_chunk",
                    type = "tool",
                    tool = "calculator",
                    input = { input = "sum([15, 25, 35, 45])" }  -- Sum: 120
                }
            }
        },
        {
            name = "chunk_3",
            steps = {
                {
                    name = "process_chunk",
                    type = "tool",
                    tool = "calculator",
                    input = { input = "sum([5, 10, 15, 20])" }  -- Sum: 50
                }
            }
        }
    },
    
    -- Reduce step after all parallel branches complete
    post_steps = {
        {
            name = "reduce_results",
            type = "custom",
            execute = function(context)
                local total = 0
                -- Sum all chunk results
                for _, branch in ipairs(context.branch_results) do
                    if branch.success then
                        local chunk_sum = tonumber(branch.steps[1].output) or 0
                        total = total + chunk_sum
                    end
                end
                
                return {
                    success = true,
                    output = "Total sum across all chunks: " .. total
                }
            end
        }
    }
})

print("\n\nExecuting map-reduce workflow...")
local map_reduce_result = map_reduce:execute()
if map_reduce_result.success then
    print("Map-reduce completed successfully")
end

-- Parallel validation workflow
-- Validate data using multiple validators concurrently
local validators = Workflow.parallel({
    name = "multi_validator",
    description = "Validate data using multiple validation rules in parallel",
    
    -- Data to validate
    setup = function()
        State.set("data_to_validate", {
            email = "user@example.com",
            age = 25,
            username = "john_doe_123",
            password = "SecurePass123!",
            phone = "+1-555-123-4567"
        })
    end,
    
    branches = {
        -- Email validation
        {
            name = "email_validator",
            steps = {
                {
                    name = "validate_email",
                    type = "tool",
                    tool = "data_validation",
                    input = {
                        input = State.get("data_to_validate").email,
                        schema = {
                            type = "string",
                            format = "email"
                        }
                    }
                }
            }
        },
        
        -- Age validation
        {
            name = "age_validator",
            steps = {
                {
                    name = "validate_age",
                    type = "tool",
                    tool = "data_validation",
                    input = {
                        input = State.get("data_to_validate").age,
                        schema = {
                            type = "number",
                            minimum = 18,
                            maximum = 120
                        }
                    }
                }
            }
        },
        
        -- Username validation
        {
            name = "username_validator",
            steps = {
                {
                    name = "validate_username",
                    type = "tool",
                    tool = "text_manipulator",
                    input = {
                        input = State.get("data_to_validate").username,
                        operation = "validate",
                        pattern = "^[a-zA-Z0-9_]{3,20}$"
                    }
                }
            }
        },
        
        -- Password strength check
        {
            name = "password_validator",
            steps = {
                {
                    name = "check_password",
                    type = "custom",
                    execute = function()
                        local password = State.get("data_to_validate").password
                        local strength = 0
                        
                        if #password >= 8 then strength = strength + 1 end
                        if password:match("[A-Z]") then strength = strength + 1 end
                        if password:match("[a-z]") then strength = strength + 1 end
                        if password:match("[0-9]") then strength = strength + 1 end
                        if password:match("[!@#$%%^&*]") then strength = strength + 1 end
                        
                        return {
                            success = strength >= 4,
                            output = "Password strength: " .. strength .. "/5"
                        }
                    end
                }
            }
        }
    },
    
    -- All validators must pass
    success_criteria = "all",
    
    -- Generate validation report
    on_complete = function(results)
        local report = "Validation Report:\n"
        for _, branch in ipairs(results.branches) do
            local status = branch.success and "PASS" or "FAIL"
            report = report .. "- " .. branch.name .. ": " .. status .. "\n"
        end
        State.set("validation_report", report)
    end
})

print("\n\nExecuting parallel validation workflow...")
local validation_result = validators:execute()
print("Validation result: " .. (validation_result.success and "All passed" or "Some failed"))
print(State.get("validation_report"))