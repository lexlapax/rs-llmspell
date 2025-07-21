-- ABOUTME: Basic conditional workflow examples with tool integration
-- ABOUTME: Demonstrates condition-based branching and decision logic

-- Basic conditional workflow with simple conditions
local time_based_workflow = Workflow.conditional({
    name = "time_based_processor",
    description = "Execute different branches based on time of day",
    
    branches = {
        -- Morning branch (6 AM - 12 PM)
        {
            name = "morning_tasks",
            condition = {
                type = "custom",
                evaluate = function()
                    local hour = tonumber(os.date("%H"))
                    return hour >= 6 and hour < 12
                end
            },
            steps = {
                {
                    name = "morning_greeting",
                    type = "tool",
                    tool = "text_manipulator",
                    input = {
                        input = "Good morning! Starting daily tasks...",
                        operation = "uppercase"
                    }
                },
                {
                    name = "generate_morning_report",
                    type = "tool",
                    tool = "template_engine",
                    input = {
                        template = "Morning Report - {{date}}\nTasks for today: {{tasks}}",
                        variables = {
                            date = os.date("%Y-%m-%d"),
                            tasks = "Review emails, Team standup, Code review"
                        }
                    }
                }
            }
        },
        
        -- Afternoon branch (12 PM - 6 PM)
        {
            name = "afternoon_tasks",
            condition = {
                type = "custom",
                evaluate = function()
                    local hour = tonumber(os.date("%H"))
                    return hour >= 12 and hour < 18
                end
            },
            steps = {
                {
                    name = "afternoon_greeting",
                    type = "tool",
                    tool = "text_manipulator",
                    input = {
                        input = "Good afternoon! Processing afternoon tasks...",
                        operation = "capitalize"
                    }
                },
                {
                    name = "calculate_progress",
                    type = "tool",
                    tool = "calculator",
                    input = { input = "8 * 60 * 0.625" }  -- Minutes worked so far
                }
            }
        },
        
        -- Evening/Night branch (default)
        {
            name = "evening_tasks",
            condition = { type = "always" },  -- Default branch
            steps = {
                {
                    name = "evening_greeting",
                    type = "tool",
                    tool = "text_manipulator",
                    input = {
                        input = "Good evening! Wrapping up daily tasks...",
                        operation = "lowercase"
                    }
                },
                {
                    name = "generate_summary",
                    type = "tool",
                    tool = "uuid_generator",
                    input = { version = "v4" }
                }
            }
        }
    },
    
    execute_default_on_no_match = true,
    error_strategy = "continue"
})

print("Executing time-based workflow...")
local time_result = time_based_workflow:execute()
print("Executed branch: " .. (time_result.data.executed_branches > 0 and "found" or "none"))

-- File type processing workflow
-- Different processing based on file extension
local file_processor_workflow = Workflow.conditional({
    name = "file_type_processor",
    description = "Process files differently based on their type",
    
    branches = {
        -- JSON processing branch
        {
            name = "json_processor",
            condition = {
                type = "step_output_contains",
                step_name = "detect_type",
                substring = "json"
            },
            steps = {
                {
                    name = "parse_json",
                    type = "tool",
                    tool = "json_processor",
                    input = {
                        input = '{"name": "test", "value": 42}',
                        operation = "parse"
                    }
                },
                {
                    name = "validate_json",
                    type = "tool",
                    tool = "data_validation",
                    input = {
                        input = "{{step:parse_json:output}}",
                        schema = {
                            type = "object",
                            required = {"name", "value"}
                        }
                    }
                }
            }
        },
        
        -- CSV processing branch
        {
            name = "csv_processor",
            condition = {
                type = "step_output_contains",
                step_name = "detect_type",
                substring = "csv"
            },
            steps = {
                {
                    name = "analyze_csv",
                    type = "tool",
                    tool = "csv_analyzer",
                    input = {
                        input = "name,value\ntest,42\n",
                        operation = "analyze"
                    }
                },
                {
                    name = "convert_to_json",
                    type = "tool",
                    tool = "json_processor",
                    input = {
                        input = "{{step:analyze_csv:output}}",
                        operation = "stringify"
                    }
                }
            }
        },
        
        -- Text processing branch (default)
        {
            name = "text_processor",
            condition = { type = "always" },
            steps = {
                {
                    name = "process_text",
                    type = "tool",
                    tool = "text_manipulator",
                    input = {
                        input = "Processing as plain text",
                        operation = "uppercase"
                    }
                }
            }
        }
    },
    
    -- Pre-processing step to detect file type
    pre_steps = {
        {
            name = "detect_type",
            type = "custom",
            execute = function()
                -- Simulate file type detection
                local file_type = "json"  -- Would normally detect from file
                return {
                    success = true,
                    output = file_type
                }
            end
        }
    }
})

-- Data validation workflow with multiple conditions
local validation_workflow = Workflow.conditional({
    name = "data_validator",
    description = "Validate and process data based on multiple conditions",
    
    branches = {
        -- Valid data branch
        {
            name = "valid_data_processing",
            condition = {
                type = "and",
                conditions = {
                    {
                        type = "shared_data_equals",
                        key = "data_valid",
                        value = true
                    },
                    {
                        type = "shared_data_greater_than",
                        key = "record_count",
                        value = 0
                    }
                }
            },
            steps = {
                {
                    name = "process_valid_data",
                    type = "tool",
                    tool = "calculator",
                    input = { input = "100 * 1.1" }  -- Apply 10% bonus for valid data
                },
                {
                    name = "save_results",
                    type = "tool",
                    tool = "file_operations",
                    input = {
                        operation = "write",
                        path = "/tmp/valid_results.txt",
                        content = "Valid data processed successfully"
                    }
                }
            }
        },
        
        -- Invalid data branch
        {
            name = "invalid_data_handling",
            condition = {
                type = "or",
                conditions = {
                    {
                        type = "shared_data_equals",
                        key = "data_valid",
                        value = false
                    },
                    {
                        type = "shared_data_less_than",
                        key = "record_count", 
                        value = 1
                    }
                }
            },
            steps = {
                {
                    name = "log_error",
                    type = "tool",
                    tool = "text_manipulator",
                    input = {
                        input = "ERROR: Invalid data detected",
                        operation = "uppercase"
                    }
                },
                {
                    name = "generate_error_id",
                    type = "tool",
                    tool = "uuid_generator",
                    input = { version = "v4" }
                }
            }
        },
        
        -- Partial data branch
        {
            name = "partial_data_recovery", 
            condition = {
                type = "and",
                conditions = {
                    {
                        type = "not",
                        condition = {
                            type = "shared_data_equals",
                            key = "data_complete",
                            value = true
                        }
                    },
                    {
                        type = "shared_data_greater_than",
                        key = "record_count",
                        value = 5
                    }
                }
            },
            steps = {
                {
                    name = "attempt_recovery",
                    type = "tool",
                    tool = "template_engine",
                    input = {
                        template = "Attempting to recover {{count}} partial records",
                        variables = {
                            count = State.get("record_count") or 0
                        }
                    }
                }
            }
        }
    },
    
    execute_all_matching = true,  -- Execute all branches that match
    error_strategy = "continue"
})

-- Set up test data
State.set("data_valid", true)
State.set("record_count", 10)
State.set("data_complete", false)

print("\nExecuting validation workflow...")
local val_result = validation_workflow:execute()
print("Branches executed: " .. val_result.data.executed_branches)
print("Success: " .. tostring(val_result.success))