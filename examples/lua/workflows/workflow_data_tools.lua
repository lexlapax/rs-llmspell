-- ABOUTME: Data processing tools integration with workflows
-- ABOUTME: Demonstrates JSON and CSV processing in workflow patterns

print("=== Data Processing Tools Workflow Examples ===\n")

-- Example 1: ETL Pipeline (Extract-Transform-Load)
print("--- Example 1: ETL Data Pipeline ---")

local etl_pipeline = Workflow.sequential({
    name = "etl_data_pipeline",
    description = "Extract, transform, and load data",
    error_strategy = "fail_fast",
    steps = {
        -- Extract: Read CSV data
        {
            name = "extract_csv",
            type = "tool",
            tool = "csv_analyzer",
            input = {
                input = [[name,age,department
John Doe,30,Engineering
Jane Smith,25,Marketing
Bob Johnson,35,Sales
Alice Brown,28,Engineering]],
                operation = "parse"
            }
        },
        -- Transform: Convert to JSON and enrich
        {
            name = "transform_to_json",
            type = "tool",
            tool = "json_processor",
            input = {
                input = "{{step:extract_csv}}",
                operation = "transform",
                transformation = {
                    add_timestamp = true,
                    add_id = true,
                    filter = {department = "Engineering"}
                }
            }
        },
        -- Validate transformed data
        {
            name = "validate_data",
            type = "tool",
            tool = "data_validation",
            input = {
                input = "{{step:transform_to_json}}",
                schema = {
                    type = "array",
                    items = {
                        type = "object",
                        required = ["name", "age", "department", "id"],
                        properties = {
                            name = {type = "string"},
                            age = {type = "number", minimum = 0},
                            department = {type = "string"},
                            id = {type = "string"}
                        }
                    }
                }
            }
        },
        -- Load: Save to file
        {
            name = "save_output",
            type = "tool",
            tool = "file_operations",
            input = {
                operation = "write",
                path = "./output/employees.json",
                content = "{{step:validate_data}}"
            }
        }
    }
})

-- Example 2: Parallel Data Analysis
print("\n--- Example 2: Parallel Data Analysis ---")

local parallel_analysis = Workflow.parallel({
    name = "parallel_data_analysis",
    description = "Analyze data from multiple sources concurrently",
    max_concurrency = 4,
    branches = {
        {
            name = "csv_analysis",
            steps = {
                {
                    name = "analyze_sales_csv",
                    type = "tool",
                    tool = "csv_analyzer",
                    input = {
                        input = [[date,product,quantity,price
2024-01-01,Widget A,100,10.00
2024-01-02,Widget B,150,15.00
2024-01-03,Widget A,200,10.00]],
                        operation = "analyze",
                        analysis = {
                            aggregations = ["sum", "avg", "count"],
                            group_by = "product"
                        }
                    }
                }
            }
        },
        {
            name = "json_processing",
            steps = {
                {
                    name = "process_config",
                    type = "tool",
                    tool = "json_processor",
                    input = {
                        input = '{"settings": {"theme": "dark", "language": "en"}, "features": ["auth", "api", "dashboard"]}',
                        operation = "query",
                        query = "$.features[*]"
                    }
                }
            }
        },
        {
            name = "data_transformation",
            steps = {
                {
                    name = "transform_user_data",
                    type = "tool",
                    tool = "json_processor",
                    input = {
                        input = '[{"id": 1, "name": "Alice"}, {"id": 2, "name": "Bob"}]',
                        operation = "map",
                        mapper = {
                            user_id = "id",
                            display_name = "name",
                            created_at = "{{timestamp}}"
                        }
                    }
                }
            }
        }
    }
})

-- Example 3: Conditional Data Processing
print("\n--- Example 3: Conditional Data Processing ---")

-- Set data format condition
State.set("data_format", "csv")

local conditional_processor = Workflow.conditional({
    name = "conditional_data_processor",
    description = "Process data based on format",
    branches = {
        {
            name = "csv_branch",
            condition = {
                type = "shared_data_equals",
                key = "data_format",
                expected = "csv"
            },
            steps = {
                {
                    name = "process_csv",
                    type = "tool",
                    tool = "csv_analyzer",
                    input = {
                        input = State.get("raw_data") or "col1,col2\nval1,val2",
                        operation = "parse",
                        options = {
                            delimiter = ",",
                            headers = true
                        }
                    }
                },
                {
                    name = "convert_csv_to_json",
                    type = "tool",
                    tool = "json_processor",
                    input = {
                        input = "{{step:process_csv}}",
                        operation = "format",
                        pretty = true
                    }
                }
            }
        },
        {
            name = "json_branch",
            condition = {
                type = "shared_data_equals",
                key = "data_format",
                expected = "json"
            },
            steps = {
                {
                    name = "process_json",
                    type = "tool",
                    tool = "json_processor",
                    input = {
                        input = State.get("raw_data") or '{"test": "data"}',
                        operation = "validate"
                    }
                },
                {
                    name = "transform_json",
                    type = "tool",
                    tool = "json_processor",
                    input = {
                        input = "{{step:process_json}}",
                        operation = "transform",
                        transformation = {
                            flatten = true,
                            remove_nulls = true
                        }
                    }
                }
            }
        }
    },
    default_branch = {
        name = "unknown_format",
        steps = {
            {
                name = "detect_format",
                type = "tool",
                tool = "data_validation",
                input = {
                    input = State.get("raw_data") or "",
                    operation = "detect_format"
                }
            }
        }
    }
})

-- Example 4: Data Processing Loop
print("\n--- Example 4: Data Processing Loop ---")

-- Sample data batches
State.set("data_batches", {
    '{"batch": 1, "records": 100}',
    '{"batch": 2, "records": 150}',
    '{"batch": 3, "records": 200}'
})

local batch_processor = Workflow.loop({
    name = "batch_data_processor",
    description = "Process data in batches",
    iterator = "collection",
    items = State.get("data_batches"),
    body = {
        {
            name = "parse_batch",
            type = "tool",
            tool = "json_processor",
            input = {
                input = "{{loop:current_item}}",
                operation = "parse"
            }
        },
        {
            name = "validate_batch",
            type = "tool",
            tool = "data_validation",
            input = {
                input = "{{step:parse_batch}}",
                schema = {
                    type = "object",
                    required = ["batch", "records"],
                    properties = {
                        batch = {type = "number"},
                        records = {type = "number", minimum = 0}
                    }
                }
            }
        },
        {
            name = "aggregate_stats",
            type = "tool",
            tool = "json_processor",
            input = {
                input = "{{step:validate_batch}}",
                operation = "query",
                query = "$.records",
                aggregate = "sum"
            }
        }
    }
})

-- Example 5: Complex Data Transformation Pipeline
print("\n--- Example 5: Complex Data Transformation ---")

local complex_transform = Workflow.sequential({
    name = "complex_data_transformation",
    description = "Multi-stage data transformation with validation",
    steps = {
        -- Stage 1: Load and parse CSV
        {
            name = "load_csv_data",
            type = "tool",
            tool = "csv_analyzer",
            input = {
                input = [[id,name,email,age,salary
1,John Doe,john@example.com,30,50000
2,Jane Smith,jane@example.com,25,45000
3,Bob Wilson,bob@example.com,35,60000
4,Alice Brown,alice@example.com,28,52000]],
                operation = "parse"
            }
        },
        -- Stage 2: Convert to JSON for processing
        {
            name = "csv_to_json",
            type = "tool",
            tool = "json_processor",
            input = {
                input = "{{step:load_csv_data}}",
                operation = "format"
            }
        },
        -- Stage 3: Enrich data
        {
            name = "enrich_data",
            type = "tool",
            tool = "json_processor",
            input = {
                input = "{{step:csv_to_json}}",
                operation = "transform",
                transformation = {
                    add_fields = {
                        processed_at = "{{timestamp}}",
                        department = "{{derive_from:salary}}",
                        seniority = "{{calculate_from:age}}"
                    },
                    calculations = {
                        annual_bonus = "salary * 0.1",
                        retirement_years = "65 - age"
                    }
                }
            }
        },
        -- Stage 4: Filter and sort
        {
            name = "filter_sort",
            type = "tool",
            tool = "json_processor",
            input = {
                input = "{{step:enrich_data}}",
                operation = "query",
                query = "$[?(@.age > 25)]",
                sort = {field = "salary", order = "desc"}
            }
        },
        -- Stage 5: Generate report
        {
            name = "generate_report",
            type = "tool",
            tool = "csv_analyzer",
            input = {
                input = "{{step:filter_sort}}",
                operation = "generate_report",
                report_type = "summary",
                metrics = ["count", "avg_salary", "age_distribution"]
            }
        }
    }
})

-- Example 6: Real-time Data Stream Processing
print("\n--- Example 6: Stream Data Processing ---")

local stream_processor = Workflow.loop({
    name = "stream_data_processor",
    description = "Process streaming data with aggregation",
    iterator = "while",
    condition = "stream_active",
    max_iterations = 100,
    body = {
        -- Read from stream
        {
            name = "read_stream",
            type = "tool",
            tool = "json_processor",
            input = {
                input = "{{stream:next}}",
                operation = "parse"
            }
        },
        -- Validate stream data
        {
            name = "validate_stream",
            type = "tool",
            tool = "data_validation",
            input = {
                input = "{{step:read_stream}}",
                schema = {
                    type = "object",
                    required = ["timestamp", "value"],
                    properties = {
                        timestamp = {type = "number"},
                        value = {type = "number"}
                    }
                }
            }
        },
        -- Aggregate in window
        {
            name = "aggregate_window",
            type = "tool",
            tool = "json_processor",
            input = {
                input = "{{step:validate_stream}}",
                operation = "aggregate",
                window = {
                    type = "sliding",
                    size = 10,
                    aggregations = ["min", "max", "avg", "std"]
                }
            }
        }
    },
    break_conditions = {
        {
            type = "shared_data_equals",
            key = "stream_complete",
            expected = true
        }
    }
})

-- Add hooks for monitoring
complex_transform:onBeforeExecute(function(context)
    print("[Hook] Starting data transformation pipeline")
    complex_transform:setState("records_processed", 0)
end)

complex_transform:onAfterExecute(function(context)
    local records = complex_transform:getState("records_processed")
    print("[Hook] Transformation complete. Processed " .. records .. " records")
    
    -- Emit metrics
    complex_transform:emit("data_pipeline_complete", {
        workflow_id = context.workflow_id,
        records_processed = records,
        duration = os.time() - complex_transform:getState("start_time")
    })
end)

-- Error handling for data validation
complex_transform:onError(function(error)
    print("[Hook] Data processing error: " .. tostring(error))
    
    if error:find("validation") then
        print("[Hook] Validation error - check data schema")
        State.set("validation_errors", (State.get("validation_errors") or 0) + 1)
    end
end)

print("\n=== Data Processing Tools Examples Complete ===")