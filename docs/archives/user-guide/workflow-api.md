# Workflow Global API Documentation

🚧 **Phase 3.3 Development Status**: Core workflow patterns are implemented. Advanced features (hooks, events, complex composition) are in active development.

The `Workflow` global provides comprehensive Lua bindings for creating and managing workflows in llmspell. It supports four workflow patterns: Sequential, Conditional, Loop, and Parallel.

## Overview

The Workflow API enables script-based orchestration of agents and tools with full integration with Hook, Event, and State globals. All workflow operations are optimized for performance with <10ms overhead.

## Workflow Patterns

### Sequential Workflow ✅ **Fully Available**

Executes steps one after another in order.

**Examples:**
- [`workflow-sequential-basics.lua`](../../examples/script-users/workflows/workflow-sequential-basics.lua)
- [`03-first-workflow.lua`](../../examples/script-users/getting-started/03-first-workflow.lua)

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

### Conditional Workflow ✅ **Fully Available**

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

### Loop Workflow ✅ **Fully Available**

Iterates over collections or conditions.

**Examples:**
- [`workflow-loop.lua`](../../examples/script-users/workflows/workflow-loop.lua)

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

### Parallel Workflow ✅ **Fully Available**

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

## Hook & Event Integration 📋 **Phase 4 Features**

*Hook registration and event emission will be available in Phase 4. Use State global for current workflow coordination.*

```lua
-- ❌ NOT YET AVAILABLE - Phase 4 features
-- workflow:onBeforeExecute(function(context)
--     print("Starting workflow: " .. context.workflow_id)
-- end)
-- workflow:emit("processing_started", {...})

-- ✅ CURRENT WORKAROUND - Use State for workflow coordination
local workflow_id = Utils.uuid()
State.set("workflow_" .. workflow_id .. "_start_time", os.time())

-- Manual lifecycle tracking
local function track_workflow_start(workflow_name)
    local start_time = os.time()
    State.set("workflow_tracking", {
        name = workflow_name,
        start_time = start_time,
        status = "running"
    })
    Logger.info("Workflow started", {name = workflow_name, start_time = start_time})
end

local function track_workflow_complete(result)
    local tracking = State.get("workflow_tracking")
    if tracking then
        local duration = os.time() - tracking.start_time
        Logger.info("Workflow completed", {
            name = tracking.name,
            duration = duration,
            success = result.success
        })
        State.set("workflow_tracking", nil)
    end
end
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

-- ✅ CURRENT: Manual monitoring with State
etl_workflow:setState("start_time", os.time())
Logger.info("ETL workflow starting", {workflow_id = "etl_pipeline"})

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

## Advanced Tool Integration Patterns

### Tool Chaining in Workflows

Workflows excel at chaining tool outputs to inputs:

```lua
-- Multi-step data processing pipeline
local data_pipeline = Workflow.sequential({
    name = "data_processor",
    steps = {
        -- Step 1: Read and parse JSON
        {
            name = "read_data",
            tool = "FileOperationsTool",
            input = {
                operation = "read",
                path = "/data/input.json"
            }
        },
        -- Step 2: Transform using jq
        {
            name = "transform",
            tool = "JsonQueryTool", 
            input = {
                operation = "query",
                input = "$read_data.content", -- Reference previous output
                query = ".users[] | select(.active == true)"
            }
        },
        -- Step 3: Convert to CSV
        {
            name = "to_csv",
            tool = "CsvGeneratorTool",
            input = {
                operation = "create",
                data = "$transform.output", -- Chain from transform
                columns = ["id", "name", "email"]
            }
        },
        -- Step 4: Write result
        {
            name = "save",
            tool = "FileOperationsTool",
            input = {
                operation = "write",
                path = "/data/active_users.csv",
                content = "$to_csv.csv"
            }
        }
    }
})
```

### Batch Processing Pattern

Process multiple items efficiently:

```lua
-- Batch file processor
function createBatchProcessor(operation)
    return Workflow.parallel({
        name = "batch_processor",
        branches = {},
        max_concurrency = 5
    })
end

-- Dynamic branch creation
local files = {"/data/file1.txt", "/data/file2.txt", "/data/file3.txt"}
local branches = {}

for i, filepath in ipairs(files) do
    table.insert(branches, {
        name = "process_" .. i,
        workflow = Workflow.sequential({
            name = "file_processor_" .. i,
            steps = {
                {
                    name = "read",
                    tool = "FileOperationsTool",
                    input = {operation = "read", path = filepath}
                },
                {
                    name = "hash", 
                    tool = "HashCalculatorTool",
                    input = {
                        operation = "hash",
                        data = "$read.content",
                        algorithm = "sha256"
                    }
                }
            }
        })
    })
end

-- Create and execute batch workflow
local batch_workflow = Workflow.parallel({
    name = "batch_hash",
    branches = branches,
    join_strategy = "all"
})

local results = batch_workflow:execute()
```

### Retry Pattern with Workflows

Implement robust retry logic:

```lua
-- Workflow with retry capability
local retry_workflow = Workflow.sequential({
    name = "api_with_retry",
    error_strategy = "continue", -- Don't stop on errors
    steps = {
        {
            name = "attempt_1",
            tool = "ApiTesterTool",
            input = {
                url = "https://api.example.com/data",
                method = "GET"
            }
        },
        {
            name = "check_retry",
            condition = function(context)
                local attempt1 = context.results.attempt_1
                return not attempt1 or not attempt1.success
            end,
            workflow = Workflow.sequential({
                name = "retry_block",
                steps = {
                    {
                        name = "wait",
                        tool = "TimerTool",
                        input = {duration_ms = 1000}
                    },
                    {
                        name = "attempt_2",
                        tool = "ApiTesterTool",
                        input = {
                            url = "https://api.example.com/data",
                            method = "GET"
                        }
                    }
                }
            })
        }
    }
})
```

### Caching Pattern

Cache expensive operations using State:

```lua
function createCachedWorkflow(cache_key, ttl_seconds)
    return Workflow.sequential({
        name = "cached_operation",
        steps = {
            -- Check cache
            {
                name = "check_cache",
                condition = function()
                    local cached = State.get(cache_key)
                    if cached then
                        local age = os.time() - cached.timestamp
                        return age < ttl_seconds
                    end
                    return false
                end,
                workflow = Workflow.sequential({
                    name = "use_cache",
                    steps = {
                        {
                            name = "return_cached",
                            tool = "IdentityTool", -- Pass through tool
                            input = function()
                                return State.get(cache_key).data
                            end
                        }
                    }
                })
            },
            -- Fetch if not cached
            {
                name = "fetch_data",
                condition = function()
                    local cached = State.get(cache_key)
                    if not cached then return true end
                    local age = os.time() - cached.timestamp
                    return age >= ttl_seconds
                end,
                workflow = Workflow.sequential({
                    name = "fetch_and_cache",
                    steps = {
                        {
                            name = "fetch",
                            tool = "ApiTesterTool",
                            input = {
                                url = "https://api.example.com/expensive",
                                method = "GET"
                            }
                        },
                        {
                            name = "cache_result",
                            tool = "IdentityTool",
                            input = function(context)
                                State.set(cache_key, {
                                    data = context.results.fetch,
                                    timestamp = os.time()
                                })
                                return context.results.fetch
                            end
                        }
                    }
                })
            }
        }
    })
end
```

### Error Aggregation Pattern

Collect errors from multiple operations:

```lua
-- Workflow that aggregates errors
local validation_workflow = Workflow.parallel({
    name = "multi_validation",
    branches = {
        {
            name = "validate_format",
            workflow = Workflow.sequential({
                name = "format_check",
                error_strategy = "continue",
                steps = {
                    {
                        name = "check_json",
                        tool = "JsonValidatorTool",
                        input = {content = "$$.input_data"}
                    }
                }
            })
        },
        {
            name = "validate_schema",
            workflow = Workflow.sequential({
                name = "schema_check",
                error_strategy = "continue",
                steps = {
                    {
                        name = "check_schema",
                        tool = "SchemaValidatorTool",
                        input = {
                            data = "$$.input_data",
                            schema = "$$.schema"
                        }
                    }
                }
            })
        },
        {
            name = "validate_business",
            workflow = Workflow.sequential({
                name = "business_check",
                error_strategy = "continue",
                steps = {
                    {
                        name = "check_rules",
                        tool = "BusinessRulesTool",
                        input = {data = "$$.input_data"}
                    }
                }
            })
        }
    },
    join_strategy = "all"
})

-- Collect all validation results
validation_workflow:onAfterExecute(function(context)
    local errors = {}
    for branch, result in pairs(context.results) do
        if result.error then
            table.insert(errors, {
                validator = branch,
                error = result.error
            })
        end
    end
    
    if #errors > 0 then
        validation_workflow:setState("validation_errors", errors)
    end
end)
```

### Tool Response Parsing

When working with tools in workflows, responses are automatically parsed:

```lua
-- Tools return structured data that workflows handle
local result = workflow:execute({
    -- Workflow automatically handles:
    -- 1. Tool exceptions (caught and returned as errors)
    -- 2. JSON parsing of tool outputs
    -- 3. Success/failure status checking
    -- 4. Result extraction from response structure
})

-- The workflow result contains:
-- result.success - boolean indicating overall success
-- result.error - error message if failed
-- result.output - final output from last step
-- result.step_results - individual step results
```

## Best Practices

1. **Use Reference Variables**: Leverage `$stepName` syntax for clean data flow
2. **Handle Errors Gracefully**: Use error_strategy and conditional steps
3. **Optimize Parallelism**: Use parallel workflows for independent operations
4. **Cache Expensive Operations**: Combine workflows with State for caching
5. **Monitor Performance**: Use hooks and metrics for observability
6. **Validate Early**: Add validation steps at the beginning of workflows
7. **Document Complex Flows**: Use descriptive names and comments
8. **Test Each Step**: Workflows can be tested step by step

## Current Limitations & Workarounds

### 🚧 **Phase 3.3 Development Limitations**

**Missing Features**:
- **Lifecycle Hooks**: `onBeforeExecute()`, `onAfterExecute()` → Use manual State tracking
- **Event System**: `workflow:emit()` → Use Logger for notifications  
- **Advanced Error Recovery**: Limited retry and circuit breaker patterns
- **Dynamic Workflow Modification**: Workflows are immutable after creation

**Current Workarounds**:
```lua
-- Instead of hooks, use manual tracking
State.set("workflow_start", os.time())
local result = workflow:execute(input)
local duration = os.time() - State.get("workflow_start")
Logger.info("Workflow completed", {duration = duration, success = result.success})

-- Instead of events, use Logger and State
Logger.info("Workflow progress", {stage = "validation", percent = 25})
State.set("workflow_progress", {stage = "validation", percent = 25})
```

### 🔮 **Coming in Phase 4**

These limitations will be resolved with the hook and event system:
- Full lifecycle hooks with 20+ trigger points
- Event bus for workflow coordination
- Advanced error recovery strategies  
- Dynamic workflow reconfiguration

## Troubleshooting

### Common Issues

1. **Step Reference Not Found**
   - Ensure step names match exactly
   - Check that referenced step executes before current step

2. **Tool Execution Failures**
   - Verify tool parameters match schema
   - Check tool permissions and limits
   - Use error_strategy: "continue" for debugging

3. **Workflow Hangs**
   - Check for infinite loops in conditional workflows
   - Verify max_iterations is set for loops
   - Use manual State tracking instead of missing hooks

4. **Performance Issues**
   - Use parallel workflows for independent operations
   - Cache results with State global
   - Check workflow metrics with getMetrics()

### Debug Mode

Enable debug mode for detailed execution logs:

```lua
Workflow.enableDebug(true)
-- Now all workflows will log detailed execution information
```