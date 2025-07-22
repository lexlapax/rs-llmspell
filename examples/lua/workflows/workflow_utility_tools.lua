-- ABOUTME: Utility tools integration with workflows
-- ABOUTME: Demonstrates all 9 utility tools in workflow patterns

print("=== Utility Tools Workflow Examples ===\n")

-- Example 1: Data Processing Pipeline with Utilities
print("--- Example 1: Data Processing with Utilities ---")

local utility_pipeline = Workflow.sequential({
    name = "utility_data_pipeline",
    description = "Process data using various utility tools",
    steps = {
        -- Generate unique ID
        {
            name = "generate_id",
            type = "tool",
            tool = "uuid_generator",
            input = {version = "v4"}
        },
        -- Get timestamp
        {
            name = "get_timestamp",
            type = "tool",
            tool = "date_time_handler",
            input = {
                operation = "now",
                format = "iso"
            }
        },
        -- Create data structure
        {
            name = "create_template",
            type = "tool",
            tool = "template_engine",
            input = {
                template = [[{
    "id": "{{id}}",
    "created_at": "{{timestamp}}",
    "message": "{{message}}",
    "checksum": "{{checksum}}"
}]],
                data = {
                    id = "{{step:generate_id}}",
                    timestamp = "{{step:get_timestamp}}",
                    message = "Hello from workflow!",
                    checksum = "pending"
                }
            }
        },
        -- Calculate hash
        {
            name = "calculate_hash",
            type = "tool",
            tool = "hash_calculator",
            input = {
                input = "{{step:create_template}}",
                algorithm = "sha256"
            }
        },
        -- Encode result
        {
            name = "encode_data",
            type = "tool",
            tool = "base64_encoder",
            input = {
                input = "{{step:calculate_hash}}",
                operation = "encode"
            }
        }
    }
})

-- Example 2: Parallel Calculations
print("\n--- Example 2: Parallel Calculations ---")

local parallel_calc = Workflow.parallel({
    name = "parallel_calculations",
    description = "Perform multiple calculations concurrently",
    branches = {
        {
            name = "math_calculations",
            steps = {
                {
                    name = "basic_math",
                    type = "tool",
                    tool = "calculator",
                    input = {input = "sqrt(16) + pi * 2"}
                },
                {
                    name = "complex_math",
                    type = "tool",
                    tool = "calculator",
                    input = {input = "sin(45) * cos(30) + tan(60)"}
                }
            }
        },
        {
            name = "text_processing",
            steps = {
                {
                    name = "manipulate_text",
                    type = "tool",
                    tool = "text_manipulator",
                    input = {
                        input = "Hello World! This is a TEST.",
                        operation = "lowercase"
                    }
                },
                {
                    name = "reverse_text",
                    type = "tool",
                    tool = "text_manipulator",
                    input = {
                        input = "{{step:manipulate_text}}",
                        operation = "reverse"
                    }
                }
            }
        },
        {
            name = "data_generation",
            steps = {
                {
                    name = "generate_uuids",
                    type = "workflow",
                    workflow = Workflow.loop({
                        name = "uuid_loop",
                        iterator = "range",
                        start = 1,
                        ["end"] = 5,
                        body = {{
                            name = "gen_uuid",
                            type = "tool",
                            tool = "uuid_generator",
                            input = {version = "v4"}
                        }}
                    })
                }
            }
        }
    }
})

-- Example 3: Text Diff and Validation
print("\n--- Example 3: Text Diff and Validation ---")

local diff_validation = Workflow.sequential({
    name = "text_diff_validation",
    description = "Compare texts and validate changes",
    steps = {
        -- Original text
        {
            name = "original_text",
            type = "tool",
            tool = "text_manipulator",
            input = {
                input = "The quick brown fox jumps over the lazy dog.",
                operation = "trim"
            }
        },
        -- Modified text
        {
            name = "modified_text",
            type = "tool",
            tool = "text_manipulator",
            input = {
                input = "The quick brown fox leaps over the lazy cat.",
                operation = "trim"
            }
        },
        -- Calculate diff
        {
            name = "calculate_diff",
            type = "tool",
            tool = "diff_calculator",
            input = {
                text1 = "{{step:original_text}}",
                text2 = "{{step:modified_text}}",
                format = "unified"
            }
        },
        -- Validate the diff
        {
            name = "validate_diff",
            type = "tool",
            tool = "data_validation",
            input = {
                input = "{{step:calculate_diff}}",
                schema = {
                    type = "object",
                    properties = {
                        changes = {type = "array"},
                        additions = {type = "number"},
                        deletions = {type = "number"}
                    }
                }
            }
        }
    }
})

-- Example 4: Conditional Processing with Utilities
print("\n--- Example 4: Conditional Utility Processing ---")

State.set("encoding_type", "base64")

local conditional_encoding = Workflow.conditional({
    name = "conditional_encoding",
    description = "Encode data based on requirements",
    branches = {
        {
            name = "base64_branch",
            condition = {
                type = "shared_data_equals",
                key = "encoding_type",
                expected = "base64"
            },
            steps = {
                {
                    name = "encode_base64",
                    type = "tool",
                    tool = "base64_encoder",
                    input = {
                        input = "Sensitive data to encode",
                        operation = "encode"
                    }
                }
            }
        },
        {
            name = "hash_branch",
            condition = {
                type = "shared_data_equals",
                key = "encoding_type",
                expected = "hash"
            },
            steps = {
                {
                    name = "hash_data",
                    type = "tool",
                    tool = "hash_calculator",
                    input = {
                        input = "Sensitive data to hash",
                        algorithm = "sha512"
                    }
                }
            }
        }
    },
    default_branch = {
        name = "default_encoding",
        steps = {
            {
                name = "simple_encode",
                type = "tool",
                tool = "text_manipulator",
                input = {
                    input = "Data to process",
                    operation = "base64"
                }
            }
        }
    }
})

-- Example 5: Template-based Report Generation
print("\n--- Example 5: Template Report Generation ---")

local report_generator = Workflow.sequential({
    name = "template_report_generator",
    description = "Generate reports using templates",
    steps = {
        -- Gather data
        {
            name = "collect_metrics",
            type = "tool",
            tool = "calculator",
            input = {input = "100 * 0.85"} -- 85% completion
        },
        {
            name = "get_date",
            type = "tool",
            tool = "date_time_handler",
            input = {
                operation = "now",
                format = "YYYY-MM-DD"
            }
        },
        {
            name = "generate_report_id",
            type = "tool",
            tool = "uuid_generator",
            input = {version = "v4"}
        },
        -- Create report
        {
            name = "generate_report",
            type = "tool",
            tool = "template_engine",
            input = {
                template = [[
# Project Status Report
**Report ID**: {{report_id}}
**Date**: {{date}}

## Summary
Project completion: {{completion}}%

## Details
- Tasks completed: {{tasks_done}}
- Tasks remaining: {{tasks_remaining}}
- Estimated completion: {{estimated_date}}

## Hash Verification
Report hash: {{hash}}
                ]],
                data = {
                    report_id = "{{step:generate_report_id}}",
                    date = "{{step:get_date}}",
                    completion = "{{step:collect_metrics}}",
                    tasks_done = 85,
                    tasks_remaining = 15,
                    estimated_date = "2024-02-15",
                    hash = "pending"
                }
            }
        },
        -- Calculate report hash
        {
            name = "hash_report",
            type = "tool",
            tool = "hash_calculator",
            input = {
                input = "{{step:generate_report}}",
                algorithm = "md5"
            }
        }
    }
})

-- Example 6: Data Validation Pipeline
print("\n--- Example 6: Data Validation Pipeline ---")

local validation_pipeline = Workflow.sequential({
    name = "data_validation_pipeline",
    description = "Validate and process various data formats",
    error_strategy = "continue",
    steps = {
        -- Validate JSON
        {
            name = "validate_json",
            type = "tool",
            tool = "data_validation",
            input = {
                input = '{"name": "Test User", "age": 25, "email": "test@example.com"}',
                schema = {
                    type = "object",
                    required = ["name", "email"],
                    properties = {
                        name = {type = "string", minLength = 1},
                        age = {type = "number", minimum = 0, maximum = 150},
                        email = {type = "string", pattern = "^[^@]+@[^@]+\\.[^@]+$"}
                    }
                }
            }
        },
        -- Validate array data
        {
            name = "validate_array",
            type = "tool",
            tool = "data_validation",
            input = {
                input = '[1, 2, 3, 4, 5]',
                schema = {
                    type = "array",
                    items = {type = "number"},
                    minItems = 1,
                    maxItems = 10
                }
            }
        },
        -- Complex validation
        {
            name = "validate_complex",
            type = "tool",
            tool = "data_validation",
            input = {
                input = '{"users": [{"id": "123", "active": true}, {"id": "456", "active": false}]}',
                schema = {
                    type = "object",
                    properties = {
                        users = {
                            type = "array",
                            items = {
                                type = "object",
                                required = ["id", "active"],
                                properties = {
                                    id = {type = "string"},
                                    active = {type = "boolean"}
                                }
                            }
                        }
                    }
                }
            }
        }
    }
})

-- Example 7: Time-based Processing Loop
print("\n--- Example 7: Time-based Processing ---")

local time_processor = Workflow.loop({
    name = "time_based_processor",
    description = "Process data at specific time intervals",
    iterator = "range",
    start = 1,
    ["end"] = 5,
    body = {
        {
            name = "get_current_time",
            type = "tool",
            tool = "date_time_handler",
            input = {
                operation = "now",
                format = "unix"
            }
        },
        {
            name = "calculate_elapsed",
            type = "tool",
            tool = "calculator",
            input = {
                input = "{{step:get_current_time}} - {{loop:start_time}}"
            }
        },
        {
            name = "format_duration",
            type = "tool",
            tool = "date_time_handler",
            input = {
                operation = "format_duration",
                seconds = "{{step:calculate_elapsed}}"
            }
        }
    }
})

-- Add monitoring hooks
report_generator:onBeforeExecute(function(context)
    print("[Hook] Starting report generation")
    State.set("report_start_time", os.time())
end)

report_generator:onAfterExecute(function(context)
    print("[Hook] Report generated successfully")
    local duration = os.time() - State.get("report_start_time")
    
    -- Emit completion event
    report_generator:emit("report_generated", {
        workflow_id = context.workflow_id,
        duration = duration,
        report_id = context.result.report_id
    })
end)

print("\n=== Utility Tools Examples Complete ===")