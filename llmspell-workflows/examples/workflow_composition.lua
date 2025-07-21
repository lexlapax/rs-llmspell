-- ABOUTME: Advanced workflow composition and nesting patterns
-- ABOUTME: Demonstrates combining workflows, reusable components, and complex orchestration

-- Workflow Composition: Building complex workflows from simpler ones
-- This example shows how to compose workflows for a complete ETL pipeline

-- Define reusable workflow components
local workflows = {}

-- Component 1: Data extraction workflow
workflows.extractor = Workflow.parallel({
    name = "data_extractor",
    description = "Extract data from multiple sources",
    
    branches = {
        {
            name = "extract_csv",
            steps = {
                {
                    name = "read_csv",
                    type = "tool",
                    tool = "file_operations",
                    input = {
                        operation = "read",
                        path = "/tmp/sales_data.csv"
                    },
                    on_error = function()
                        -- Create sample data if missing
                        Tools.get("file_operations"):execute({
                            operation = "write",
                            path = "/tmp/sales_data.csv",
                            content = "date,product,amount\n2024-01-01,Widget,100\n2024-01-02,Gadget,150"
                        })
                        return { retry = true }
                    end
                },
                {
                    name = "parse_csv",
                    type = "tool",
                    tool = "csv_analyzer",
                    input = {
                        input = "{{step:read_csv:output}}",
                        operation = "parse"
                    }
                }
            }
        },
        {
            name = "extract_json",
            steps = {
                {
                    name = "read_json",
                    type = "tool",
                    tool = "file_operations",
                    input = {
                        operation = "read",
                        path = "/tmp/config.json"
                    },
                    on_error = function()
                        Tools.get("file_operations"):execute({
                            operation = "write",
                            path = "/tmp/config.json",
                            content = '{"settings": {"threshold": 100, "region": "US"}}'
                        })
                        return { retry = true }
                    end
                },
                {
                    name = "parse_json",
                    type = "tool",
                    tool = "json_processor",
                    input = {
                        input = "{{step:read_json:output}}",
                        operation = "parse"
                    }
                }
            }
        },
        {
            name = "extract_api",
            steps = {
                {
                    name = "simulate_api",
                    type = "custom",
                    execute = function()
                        return {
                            success = true,
                            output = {
                                users = 500,
                                active_sessions = 45,
                                timestamp = os.time()
                            }
                        }
                    end
                }
            }
        }
    }
})

-- Component 2: Data transformation workflow
workflows.transformer = Workflow.sequential({
    name = "data_transformer",
    description = "Transform and enrich extracted data",
    
    steps = {
        -- Merge data from different sources
        {
            name = "merge_data",
            type = "custom",
            execute = function()
                local extraction_results = State.get("extraction_results") or {}
                local merged = {
                    csv_data = extraction_results.csv,
                    json_config = extraction_results.json,
                    api_metrics = extraction_results.api,
                    merge_timestamp = os.time()
                }
                return {
                    success = true,
                    output = merged
                }
            end
        },
        
        -- Validate merged data
        {
            name = "validate_data",
            type = "tool",
            tool = "data_validation",
            input = {
                input = "{{step:merge_data:output}}",
                schema = {
                    type = "object",
                    required = {"csv_data", "json_config", "api_metrics"}
                }
            }
        },
        
        -- Apply business rules
        {
            name = "apply_rules",
            type = "conditional",
            workflow = Workflow.conditional({
                name = "business_rules",
                branches = {
                    {
                        name = "high_volume",
                        condition = {
                            type = "custom",
                            evaluate = function()
                                local metrics = State.get("extraction_results").api or {}
                                return (metrics.users or 0) > 100
                            end
                        },
                        steps = {
                            {
                                name = "apply_high_volume_rules",
                                type = "tool",
                                tool = "template_engine",
                                input = {
                                    template = "High volume processing: {{users}} users",
                                    variables = {
                                        users = State.get("extraction_results").api.users
                                    }
                                }
                            }
                        }
                    },
                    {
                        name = "standard",
                        condition = { type = "always" },
                        steps = {
                            {
                                name = "apply_standard_rules",
                                type = "tool",
                                tool = "text_manipulator",
                                input = {
                                    input = "Standard processing applied",
                                    operation = "uppercase"
                                }
                            }
                        }
                    }
                }
            })
        },
        
        -- Enrich with calculated fields
        {
            name = "enrich_data",
            type = "tool",
            tool = "calculator",
            input = {
                input = "{{api_metrics.users}} * 0.15"  -- Calculate metric
            }
        }
    }
})

-- Component 3: Data loading workflow
workflows.loader = Workflow.sequential({
    name = "data_loader",
    description = "Load transformed data to destinations",
    
    steps = {
        -- Format for output
        {
            name = "format_output",
            type = "tool",
            tool = "json_processor",
            input = {
                input = State.get("transformed_data"),
                operation = "stringify",
                pretty = true
            }
        },
        
        -- Save to file
        {
            name = "save_to_file",
            type = "tool",
            tool = "file_operations",
            input = {
                operation = "write",
                path = "/tmp/etl_output_{{timestamp}}.json",
                content = "{{step:format_output:output}}"
            }
        },
        
        -- Generate report
        {
            name = "generate_report",
            type = "tool",
            tool = "template_engine",
            input = {
                template = [[
# ETL Pipeline Report
Generated: {{timestamp}}

## Extraction Summary
- CSV Records: {{csv_count}}
- API Users: {{api_users}}
- Config Loaded: {{config_status}}

## Transformation
- Rules Applied: {{rules}}
- Validation: {{validation}}

## Loading
- Output File: {{output_file}}
- Status: Complete
]],
                variables = {
                    timestamp = os.date("%Y-%m-%d %H:%M:%S"),
                    csv_count = "{{extraction.csv.row_count}}",
                    api_users = "{{extraction.api.users}}",
                    config_status = "Success",
                    rules = State.get("applied_rules") or "Standard",
                    validation = "Passed",
                    output_file = "{{step:save_to_file:output.path}}"
                }
            }
        }
    }
})

-- Master ETL Pipeline Workflow (Composition)
local etl_pipeline = Workflow.sequential({
    name = "master_etl_pipeline",
    description = "Complete ETL pipeline composed of extraction, transformation, and loading",
    
    steps = {
        -- Step 1: Execute extraction workflow
        {
            name = "extract",
            type = "workflow",
            workflow = workflows.extractor,
            on_complete = function(result)
                -- Store extraction results
                State.set("extraction_results", {
                    csv = result.branches.extract_csv,
                    json = result.branches.extract_json,
                    api = result.branches.extract_api
                })
            end
        },
        
        -- Step 2: Execute transformation workflow
        {
            name = "transform",
            type = "workflow",
            workflow = workflows.transformer,
            on_complete = function(result)
                State.set("transformed_data", result.data)
            end
        },
        
        -- Step 3: Execute loading workflow
        {
            name = "load",
            type = "workflow",
            workflow = workflows.loader
        }
    },
    
    error_strategy = "fail_fast"
})

-- Execute the composed pipeline
print("Starting composed ETL pipeline...")
local pipeline_result = etl_pipeline:execute()
print("ETL pipeline completed: " .. (pipeline_result.success and "Success" or "Failed"))

-- Advanced Composition: Nested Parallel-Sequential Workflows
local advanced_workflow = Workflow.parallel({
    name = "advanced_composition",
    description = "Complex workflow with nested parallel and sequential patterns",
    
    branches = {
        -- Branch 1: Sequential data processing
        {
            name = "data_branch",
            workflow = Workflow.sequential({
                name = "data_processor",
                steps = {
                    {
                        name = "fetch_data",
                        type = "custom",
                        execute = function()
                            return { success = true, output = "Data fetched" }
                        end
                    },
                    {
                        name = "process_data",
                        type = "parallel",
                        workflow = Workflow.parallel({
                            branches = {
                                {
                                    name = "clean_data",
                                    steps = {{
                                        name = "clean",
                                        type = "tool",
                                        tool = "text_manipulator",
                                        input = { input = "raw data", operation = "trim" }
                                    }}
                                },
                                {
                                    name = "validate_data",
                                    steps = {{
                                        name = "validate",
                                        type = "tool",
                                        tool = "data_validation",
                                        input = { 
                                            input = "data",
                                            schema = { type = "string" }
                                        }
                                    }}
                                }
                            }
                        })
                    }
                }
            })
        },
        
        -- Branch 2: Conditional monitoring
        {
            name = "monitor_branch",
            workflow = Workflow.conditional({
                name = "monitor",
                branches = {
                    {
                        name = "alert",
                        condition = {
                            type = "shared_data_greater_than",
                            key = "error_count",
                            value = 0
                        },
                        steps = {{
                            name = "send_alert",
                            type = "tool",
                            tool = "template_engine",
                            input = {
                                template = "ALERT: {{count}} errors detected",
                                variables = { count = State.get("error_count") or 0 }
                            }
                        }}
                    }
                }
            })
        }
    }
})

-- Recursive Workflow Pattern
-- A workflow that can call itself for recursive processing
local recursive_processor = Workflow.conditional({
    name = "recursive_processor",
    description = "Process nested data structures recursively",
    
    branches = {
        -- Base case: simple value
        {
            name = "base_case",
            condition = {
                type = "custom",
                evaluate = function()
                    local data = State.get("current_data")
                    return type(data) ~= "table"
                end
            },
            steps = {{
                name = "process_value",
                type = "tool",
                tool = "text_manipulator",
                input = {
                    input = State.get("current_data"),
                    operation = "uppercase"
                }
            }}
        },
        
        -- Recursive case: nested structure
        {
            name = "recursive_case",
            condition = { type = "always" },
            steps = {
                {
                    name = "process_nested",
                    type = "loop",
                    workflow = Workflow.loop({
                        iterator = {
                            collection = State.get("current_data")
                        },
                        body = {
                            {
                                name = "recurse",
                                type = "workflow",
                                workflow = recursive_processor,
                                setup = function(item)
                                    State.set("current_data", item)
                                end
                            }
                        }
                    })
                }
            }
        }
    }
})

print("\n\nWorkflow composition examples completed!")