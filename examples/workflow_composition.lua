-- ABOUTME: Workflow composition example showing how workflows can be combined
-- ABOUTME: Demonstrates building complex workflows from simpler ones

-- Example: ETL Pipeline with Nested Workflows
print("=== ETL Pipeline with Workflow Composition ===\n")

-- Step 1: Create extraction workflow
local extraction_workflow = Workflow.parallel({
    name = "data_extraction",
    description = "Extract data from multiple sources in parallel",
    max_concurrency = 5,
    branches = {
        {
            name = "database_extract",
            steps = {{
                name = "extract_from_db",
                type = "tool",
                tool = "database_reader",
                input = { 
                    connection = "postgres://localhost/mydb",
                    query = "SELECT * FROM users WHERE active = true"
                }
            }}
        },
        {
            name = "api_extract",
            steps = {{
                name = "extract_from_api",
                type = "tool",
                tool = "api_caller",
                input = {
                    url = "https://api.example.com/users",
                    method = "GET"
                }
            }}
        },
        {
            name = "file_extract",
            steps = {{
                name = "extract_from_file",
                type = "tool",
                tool = "csv_reader",
                input = { path = "/data/users.csv" }
            }}
        }
    }
})

-- Step 2: Create transformation workflow
local transform_workflow = Workflow.sequential({
    name = "data_transformation",
    description = "Transform and clean extracted data",
    steps = {
        {
            name = "merge_sources",
            type = "tool",
            tool = "data_merger",
            input = { 
                merge_key = "user_id",
                conflict_resolution = "newest"
            }
        },
        {
            name = "clean_data",
            type = "tool",
            tool = "data_cleaner",
            input = {
                remove_nulls = true,
                trim_strings = true,
                validate_emails = true
            }
        },
        {
            name = "enrich_data",
            type = "tool",
            tool = "data_enricher",
            input = {
                add_geolocation = true,
                add_timestamps = true
            }
        }
    }
})

-- Step 3: Create loading workflow with conditional logic
local load_workflow = Workflow.conditional({
    name = "data_loading",
    description = "Load data to appropriate destination based on size",
    branches = {
        {
            name = "small_dataset",
            condition = {
                type = "custom",
                expression = "data_size < 1000000",
                description = "Less than 1MB"
            },
            steps = {{
                name = "load_to_cache",
                type = "tool",
                tool = "cache_writer",
                input = { ttl = 3600 }
            }}
        },
        {
            name = "large_dataset",
            condition = {
                type = "custom",
                expression = "data_size >= 1000000",
                description = "1MB or larger"
            },
            steps = {{
                name = "load_to_database",
                type = "tool",
                tool = "batch_db_writer",
                input = { 
                    batch_size = 1000,
                    table = "users_processed"
                }
            }}
        }
    }
})

-- Step 4: Create master ETL workflow that composes the others
local etl_pipeline = Workflow.sequential({
    name = "master_etl_pipeline",
    description = "Complete ETL pipeline using composed workflows",
    error_strategy = "fail_fast",
    timeout_ms = 300000, -- 5 minutes
    steps = {
        {
            name = "initialize",
            type = "tool",
            tool = "pipeline_initializer",
            input = { 
                pipeline_id = "etl_" .. os.time(),
                log_level = "info"
            }
        },
        -- In a full implementation, these would be workflow steps
        -- For now, we simulate with tool steps
        {
            name = "extract_phase",
            type = "tool",
            tool = "workflow_executor",
            input = { 
                workflow_id = extraction_workflow:getInfo().id,
                phase = "extraction"
            }
        },
        {
            name = "transform_phase",
            type = "tool",
            tool = "workflow_executor",
            input = {
                workflow_id = transform_workflow:getInfo().id,
                phase = "transformation"
            }
        },
        {
            name = "load_phase",
            type = "tool",
            tool = "workflow_executor",
            input = {
                workflow_id = load_workflow:getInfo().id,
                phase = "loading"
            }
        },
        {
            name = "finalize",
            type = "tool",
            tool = "pipeline_finalizer",
            input = {
                generate_report = true,
                cleanup_temp_files = true
            }
        }
    }
})

-- Set up comprehensive monitoring
etl_pipeline:onBeforeExecute(function(context)
    print("Starting ETL pipeline...")
    etl_pipeline:setState("start_time", os.time())
    etl_pipeline:emit("etl_started", { pipeline_id = context.workflow_id })
end)

etl_pipeline:onAfterExecute(function(context)
    local duration = os.time() - (etl_pipeline:getState("start_time") or 0)
    print(string.format("ETL pipeline completed in %d seconds", duration))
    etl_pipeline:emit("etl_completed", { 
        pipeline_id = context.workflow_id,
        duration = duration
    })
end)

etl_pipeline:onError(function(error)
    print("ETL pipeline failed: " .. tostring(error))
    etl_pipeline:setState("status", "failed")
    etl_pipeline:setState("error", error)
    etl_pipeline:emit("etl_failed", {
        error = error,
        phase = etl_pipeline:getState("current_phase")
    })
end)

-- Example: Validation workflow that runs after ETL
local validation_loop = Workflow.loop({
    name = "data_validator",
    description = "Validate processed data in batches",
    iterator = "range",
    start = 0,
    ["end"] = 100,
    step = 10,
    body = {
        {
            name = "validate_batch",
            type = "tool",
            tool = "data_validator",
            input = {
                rules = {
                    "user_id_unique",
                    "email_format_valid",
                    "required_fields_present"
                }
            }
        }
    }
})

-- Create a monitoring workflow that runs periodically
local monitoring_workflow = Workflow.loop({
    name = "pipeline_monitor",
    description = "Monitor pipeline health",
    iterator = "while",
    condition = {
        type = "shared_data_equals",
        key = "monitoring_enabled",
        expected = true
    },
    max_iterations = 1000,
    body = {
        {
            name = "check_pipeline_health",
            type = "tool",
            tool = "health_checker",
            input = {
                check_memory = true,
                check_disk_space = true,
                check_processing_rate = true
            }
        },
        {
            name = "wait",
            type = "tool",
            tool = "timer",
            input = { seconds = 60 } -- Check every minute
        }
    }
})

-- Demonstrate workflow discovery and introspection
print("\n=== Registered Workflows ===")
local workflows = Workflow.list()
for _, wf in ipairs(workflows) do
    print(string.format("- %s (%s): %s", wf.id, wf.type, wf.description or ""))
end

print("\n=== Workflow Composition Example Complete ===")