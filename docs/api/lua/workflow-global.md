# Workflow Global API Documentation

The `Workflow` global provides comprehensive Lua bindings for creating and managing workflows in llmspell. It supports four workflow patterns: Sequential, Conditional, Loop, and Parallel.

## Overview

The Workflow API enables script-based orchestration of agents and tools with full integration with Hook, Event, and State globals. All workflow operations are optimized for performance with <10ms overhead.

## Workflow Patterns

### Sequential Workflow

Executes steps one after another in order.

```lua
local workflow = Workflow.sequential({
    name = "data_pipeline",
    description = "Process data through multiple stages",
    error_strategy = "fail_fast", -- "fail_fast", "continue", or "retry"
    timeout_ms = 30000,
    steps = {
        {
            name = "load_data",
            type = "tool",
            tool = "file_reader",
            input = { path = "/data/input.csv" }
        },
        {
            name = "transform",
            type = "agent",
            agent = "data_transformer",
            input = "transform the data"
        },
        {
            name = "save_results",
            type = "tool", 
            tool = "file_writer",
            input = { path = "/data/output.json" }
        }
    }
})
```

### Conditional Workflow

Executes different branches based on conditions.

```lua
local workflow = Workflow.conditional({
    name = "smart_processor",
    description = "Process based on conditions",
    branches = {
        {
            name = "large_file_branch",
            condition = {
                type = "shared_data_equals",
                key = "file_size",
                expected = "large"
            },
            steps = {
                {
                    name = "stream_process",
                    type = "tool",
                    tool = "stream_processor",
                    input = {}
                }
            }
        },
        {
            name = "small_file_branch", 
            condition = {
                type = "shared_data_equals",
                key = "file_size",
                expected = "small"
            },
            steps = {
                {
                    name = "batch_process",
                    type = "tool",
                    tool = "batch_processor",
                    input = {}
                }
            }
        }
    },
    default_branch = {
        name = "fallback",
        steps = {
            {
                name = "generic_process",
                type = "tool",
                tool = "generic_processor",
                input = {}
            }
        }
    }
})
```

### Loop Workflow

Iterates over collections or conditions.

```lua
-- Range-based loop
local workflow = Workflow.loop({
    name = "batch_processor",
    iterator = "range",
    start = 1,
    ["end"] = 100,
    step = 10,
    body = {
        {
            name = "process_batch",
            type = "tool",
            tool = "batch_handler",
            input = {}
        }
    }
})

-- Collection-based loop
local workflow = Workflow.loop({
    name = "file_processor",
    iterator = "collection",
    items = {"file1.txt", "file2.txt", "file3.txt"},
    body = {
        {
            name = "process_file",
            type = "tool",
            tool = "file_processor",
            input = {}
        }
    }
})

-- While-condition loop
local workflow = Workflow.loop({
    name = "monitor_loop",
    iterator = "while",
    condition = "status_not_ready",
    max_iterations = 50,
    body = {
        {
            name = "check_status",
            type = "tool",
            tool = "status_checker",
            input = {}
        }
    }
})
```

### Parallel Workflow

Executes branches concurrently.

```lua
local workflow = Workflow.parallel({
    name = "parallel_analysis",
    description = "Analyze data in parallel",
    max_concurrency = 4,
    timeout_ms = 60000,
    error_strategy = "continue",
    branches = {
        {
            name = "sentiment_analysis",
            steps = {
                {
                    name = "analyze_sentiment",
                    type = "agent",
                    agent = "sentiment_analyzer",
                    input = "Analyze sentiment"
                }
            }
        },
        {
            name = "entity_extraction",
            steps = {
                {
                    name = "extract_entities",
                    type = "tool",
                    tool = "entity_extractor",
                    input = {}
                }
            }
        },
        {
            name = "summarization",
            steps = {
                {
                    name = "summarize",
                    type = "agent",
                    agent = "summarizer",
                    input = "Create summary"
                }
            }
        }
    }
})
```

## Workflow Instance Methods

### execute(input)

Executes the workflow with optional input data.

```lua
local result = workflow:execute({
    data = "input data",
    config = { verbose = true }
})
```

### getInfo()

Gets workflow metadata.

```lua
local info = workflow:getInfo()
print("ID: " .. info.id)
print("Name: " .. info.name)
print("Type: " .. info.type)
```

### validate()

Validates workflow configuration.

```lua
local validation = workflow:validate()
if validation.valid then
    print("Workflow is valid")
else
    for _, error in ipairs(validation.errors) do
        print("Error: " .. error)
    end
end
```

### debug()

Gets debug information about the workflow.

```lua
local debug_info = workflow:debug()
print("Created at: " .. debug_info.runtime.created_at)
```

### getMetrics()

Gets execution metrics.

```lua
local metrics = workflow:getMetrics()
print("Total executions: " .. metrics.execution.total_executions)
print("Success rate: " .. metrics.execution.successful_executions / metrics.execution.total_executions)
print("Average duration: " .. metrics.execution.average_duration_ms .. "ms")
```

## State Integration

Workflows integrate with the State global for persistent data sharing.

```lua
-- Set workflow-specific state
workflow:setState("processing_stage", "validation")
workflow:setState("item_count", 42)

-- Get state
local stage = workflow:getState("processing_stage")
local count = workflow:getState("item_count")
```

## Hook Integration

Register hooks for workflow lifecycle events.

```lua
-- Before execution hook
workflow:onBeforeExecute(function(context)
    print("Starting workflow: " .. context.workflow_id)
    State.set("workflow_start_time", os.time())
end)

-- After execution hook
workflow:onAfterExecute(function(context)
    local duration = os.time() - State.get("workflow_start_time")
    print("Workflow completed in " .. duration .. " seconds")
    print("Result: " .. tostring(context.result))
end)

-- Error hook
workflow:onError(function(error)
    print("Error occurred: " .. tostring(error))
    workflow:emit("workflow_error", {
        error = tostring(error),
        timestamp = os.time()
    })
end)
```

## Event Integration

Emit events from workflows.

```lua
-- Emit custom events
workflow:emit("processing_started", {
    workflow_name = workflow:getInfo().name,
    timestamp = os.time()
})

-- Emit progress events
workflow:emit("progress_update", {
    stage = "transformation",
    percent_complete = 75
})
```

## Registry Methods

### Workflow.list()

List all registered workflows.

```lua
local workflows = Workflow.list()
for _, wf in ipairs(workflows) do
    print(wf.id .. ": " .. wf.type .. " - " .. wf.description)
end
```

### Workflow.get(id)

Get a specific workflow by ID.

```lua
local workflow = Workflow.get("workflow_123")
if workflow then
    local info = workflow:getInfo()
    print("Found: " .. info.name)
end
```

### Workflow.remove(id)

Remove a workflow from the registry.

```lua
Workflow.remove("workflow_123")
```

### Workflow.types()

Get supported workflow types.

```lua
local types = Workflow.types()
-- Returns: {"sequential", "conditional", "loop", "parallel"}
```

## Debugging Utilities

### Workflow.enableDebug(enabled)

Enable or disable debug mode globally.

```lua
Workflow.enableDebug(true)
-- Workflows will now output detailed debug information
```

### Workflow.isDebugEnabled()

Check if debug mode is enabled.

```lua
if Workflow.isDebugEnabled() then
    print("Debug mode is active")
end
```

### Workflow.setDefaultErrorHandler(handler)

Set a default error handler for all workflows.

```lua
Workflow.setDefaultErrorHandler(function(error, context)
    print("Error in workflow " .. context.workflow_id)
    print("Phase: " .. context.phase)
    print("Error: " .. tostring(error))
    
    -- Log to monitoring system
    Monitor.logError({
        workflow = context.workflow_id,
        error = tostring(error),
        timestamp = os.time()
    })
end)
```

## Conditions

Conditions control workflow branching and loops.

### Basic Conditions

- `"always"` - Always true
- `"never"` - Always false

### Complex Conditions

```lua
-- Step output condition
condition = {
    type = "step_output_equals",
    step = "validation_step",
    expected = "success"
}

-- Shared data condition
condition = {
    type = "shared_data_equals",
    key = "processing_mode",
    expected = "batch"
}

-- AND condition
condition = {
    type = "and",
    conditions = {
        {type = "always"},
        {type = "shared_data_equals", key = "ready", expected = true}
    }
}

-- OR condition
condition = {
    type = "or",
    conditions = {
        {type = "step_output_equals", step = "check1", expected = "pass"},
        {type = "step_output_equals", step = "check2", expected = "pass"}
    }
}

-- NOT condition
condition = {
    type = "not",
    condition = {type = "shared_data_equals", key = "disabled", expected = true}
}

-- Custom condition
condition = {
    type = "custom",
    expression = "data.temperature > 100 and data.pressure < 50",
    description = "High temperature, low pressure condition"
}
```

## Error Strategies

- `"fail_fast"` - Stop on first error (default)
- `"continue"` - Continue execution, collect all errors
- `"retry"` - Retry failed steps with exponential backoff

## Performance

All workflow operations are optimized for performance:
- Workflow creation: <20µs
- Workflow execution overhead: <10ms
- Registry operations: <1ms

## Complete Example

```lua
-- Enable debugging
Workflow.enableDebug(true)

-- Set default error handler
Workflow.setDefaultErrorHandler(function(error, context)
    print("[ERROR] " .. context.workflow_id .. ": " .. tostring(error))
end)

-- Create a complex ETL workflow
local etl_workflow = Workflow.sequential({
    name = "etl_pipeline",
    description = "Extract, Transform, Load data pipeline",
    error_strategy = "retry",
    timeout_ms = 300000,
    steps = {
        -- Extract phase
        {
            name = "extract_data",
            type = "tool",
            tool = "data_extractor",
            input = {
                source = "database",
                query = "SELECT * FROM transactions"
            }
        },
        -- Transform phase (conditional)
        {
            name = "transform_router",
            type = "workflow",
            workflow = Workflow.conditional({
                name = "transform_conditional",
                branches = {
                    {
                        name = "large_dataset",
                        condition = {
                            type = "shared_data_equals",
                            key = "record_count",
                            expected = "large"
                        },
                        steps = {
                            {
                                name = "parallel_transform",
                                type = "workflow",
                                workflow = Workflow.parallel({
                                    name = "parallel_transforms",
                                    max_concurrency = 4,
                                    branches = {
                                        {name = "clean_data", steps = {{name = "clean", type = "tool", tool = "data_cleaner", input = {}}}},
                                        {name = "enrich_data", steps = {{name = "enrich", type = "tool", tool = "data_enricher", input = {}}}},
                                        {name = "validate_data", steps = {{name = "validate", type = "tool", tool = "data_validator", input = {}}}}
                                    }
                                })
                            }
                        }
                    }
                },
                default_branch = {
                    name = "standard_transform",
                    steps = {
                        {name = "simple_transform", type = "tool", tool = "transformer", input = {}}
                    }
                }
            })
        },
        -- Load phase
        {
            name = "load_data",
            type = "tool",
            tool = "data_loader",
            input = {
                destination = "warehouse",
                table = "processed_transactions"
            }
        }
    }
})

-- Set up monitoring
etl_workflow:onBeforeExecute(function(context)
    etl_workflow:setState("start_time", os.time())
    etl_workflow:emit("etl_started", {workflow_id = context.workflow_id})
end)

etl_workflow:onAfterExecute(function(context)
    local duration = os.time() - etl_workflow:getState("start_time")
    etl_workflow:emit("etl_completed", {
        workflow_id = context.workflow_id,
        duration = duration,
        success = true
    })
end)

-- Execute the workflow
local result = etl_workflow:execute({
    batch_id = "2024-01-20-001",
    mode = "production"
})

-- Check results
if result.success then
    print("ETL completed successfully")
    local metrics = etl_workflow:getMetrics()
    print("Execution time: " .. metrics.execution.average_duration_ms .. "ms")
else
    print("ETL failed: " .. result.error)
end
```