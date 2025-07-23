# Agent and Workflow API Reference

**Version**: Phase 3.3 API Reference  
**Status**: ✅ **CURRENT** - Complete API documentation  
**Last Updated**: July 2025

> **📖 API REFERENCE**: Comprehensive API documentation for Agents and Workflows in rs-llmspell. Includes all methods, parameters, and usage patterns.

**🔗 Navigation**: [← User Guide](README.md) | [Documentation Hub](../README.md) | [Tutorial](tutorial-agents-workflows.md) | [Examples](../../examples/)

---

## Table of Contents
- [Agent API](#agent-api)
- [Workflow API](#workflow-api)
- [Integration Patterns](#integration-patterns)
- [Error Handling](#error-handling)
- [Performance Tips](#performance-tips)

## Agent API

### Agent.create(config)

Creates a new agent instance.

```lua
local agent = Agent.create({
    name = "my_agent",                    -- Required: Agent name
    description = "Agent description",     -- Optional: Description
    model = "provider/model",             -- Required: Model in provider/model format
    system_prompt = "You are...",         -- Required: System prompt
    temperature = 0.7,                    -- Optional: Temperature (0.0-1.0)
    max_tokens = 2000,                    -- Optional: Max response tokens
    timeout = 30000                       -- Optional: Timeout in ms
})
```

**Available Models**:
- OpenAI: `openai/gpt-4`, `openai/gpt-3.5-turbo`, `openai/gpt-4o-mini`
- Anthropic: `anthropic/claude-3-opus`, `anthropic/claude-3-sonnet`, `anthropic/claude-3-haiku`
- Local models: Check your configuration

### Agent.register(name, agent)

Registers an agent globally for reuse.

```lua
Agent.register("analyzer", agent)
```

### Agent.get(name)

Retrieves a registered agent.

```lua
local agent = Agent.get("analyzer")
```

### agent:execute(params)

Executes the agent with given parameters.

```lua
local result = agent:execute({
    prompt = "Analyze this data: ...",    -- Required: The prompt
    variables = {                         -- Optional: Template variables
        key = "value"
    },
    max_tokens = 1000,                    -- Optional: Override max tokens
    temperature = 0.5                     -- Optional: Override temperature
})

-- Result structure:
-- {
--     success = true/false,
--     output = "Response text",
--     usage = { prompt_tokens = N, completion_tokens = N, total_tokens = N },
--     error = "Error message if failed"
-- }
```

## Workflow API

### Workflow.sequential(config)

Creates a sequential workflow that executes steps in order.

```lua
local workflow = Workflow.sequential({
    name = "my_workflow",                 -- Required: Workflow name
    description = "Description",          -- Optional: Description
    
    steps = {                            -- Required: Array of steps
        {
            name = "step1",              -- Required: Step name
            type = "tool",               -- Required: tool/agent/custom
            tool = "calculator",         -- For tool type
            agent = agent_instance,      -- For agent type
            input = {                    -- Input parameters
                -- Parameters with template support
            },
            execute = function(context)   -- For custom type
                -- Custom logic
                return { success = true, output = "result" }
            end
        }
    },
    
    error_strategy = "fail_fast",        -- Optional: fail_fast/continue/retry
    timeout_ms = 30000,                  -- Optional: Workflow timeout
    
    -- Hooks
    on_start = function() end,           -- Optional: Start hook
    on_complete = function(success) end, -- Optional: Complete hook
    on_step_complete = function(name, result) end  -- Optional
})
```

### Workflow.conditional(config)

Creates a conditional workflow with branch-based execution.

```lua
local workflow = Workflow.conditional({
    name = "conditional_workflow",
    
    branches = {                         -- Required: Array of branches
        {
            name = "branch1",
            condition = {                -- Required: Branch condition
                type = "shared_data_equals",
                key = "status",
                value = "active"
            },
            steps = { ... }              -- Same as sequential steps
        }
    },
    
    default_branch = { ... },            -- Optional: Default branch
    execute_all_matching = false,        -- Optional: Execute all matches
    execute_default_on_no_match = true   -- Optional: Use default
})
```

#### Condition Types

- `always` - Always true
- `never` - Always false
- `shared_data_equals` - Compare state value
- `shared_data_greater_than` - Numeric comparison
- `shared_data_less_than` - Numeric comparison
- `step_output_contains` - Check step output
- `and` - Logical AND of conditions
- `or` - Logical OR of conditions
- `not` - Logical NOT of condition
- `custom` - Custom evaluation function

```lua
-- Complex condition example
condition = {
    type = "and",
    conditions = [
        {
            type = "shared_data_greater_than",
            key = "score",
            value = 80
        },
        {
            type = "step_output_contains",
            step = "validation",
            substring = "approved"
        }
    ]
}
```

### Workflow.loop(config)

Creates a loop workflow for iteration.

```lua
local workflow = Workflow.loop({
    name = "loop_workflow",
    
    iterator = {                         -- Required: Iterator config
        -- Range iterator
        range = {
            start = 1,
            ["end"] = 10,
            step = 1
        },
        
        -- OR Collection iterator
        collection = {item1, item2, ...},
        
        -- OR While condition
        while_condition = {
            type = "shared_data_less_than",
            key = "counter",
            value = 100
        }
    },
    
    body = { ... },                      -- Required: Steps to repeat
    
    max_iterations = 1000,               -- Optional: Safety limit
    break_condition = { ... },           -- Optional: Early exit
    aggregation_strategy = "collect_all", -- Optional: Result handling
    
    -- Hooks
    on_iteration_complete = function(index, result) end
})
```

### Workflow.parallel(config)

Creates a parallel workflow for concurrent execution.

```lua
local workflow = Workflow.parallel({
    name = "parallel_workflow",
    
    branches = {                         -- Required: Parallel branches
        {
            name = "branch1",
            required = false,            -- Optional: Must succeed
            steps = { ... }
        }
    },
    
    max_concurrency = 10,                -- Optional: Concurrency limit
    fail_fast = false,                   -- Optional: Stop on failure
    
    post_steps = { ... },                -- Optional: After parallel
    
    -- Hooks
    on_branch_complete = function(name, result) end
})
```

### Step Reference Syntax

Reference outputs from previous steps:

```lua
{
    input = {
        -- Reference step output
        value = "{{step:step_name:output}}",
        
        -- Reference specific field
        field = "{{step:step_name:output.field}}",
        
        -- Reference state
        state_val = "{{state:key_name}}",
        
        -- Reference loop context
        item = "{{loop:current_item}}",
        index = "{{loop:current_index}}",
        
        -- Reference branch output (parallel)
        branch = "{{branch:branch_name:step_name:output}}"
    }
}
```

## Integration Patterns

### Agent as Workflow Step

```lua
local workflow = Workflow.sequential({
    steps = {
        {
            name = "analyze",
            type = "agent",
            agent = analyzer_agent,
            input = {
                prompt = "Analyze: {{step:previous:output}}"
            }
        }
    }
})
```

### Tool in Workflow

```lua
local workflow = Workflow.sequential({
    steps = {
        {
            name = "calculate",
            type = "tool",
            tool = "calculator",
            input = {
                operation = "evaluate",
                input = "{{state:expression}}"
            }
        }
    }
})
```

### Workflow Composition

```lua
local sub_workflow = Workflow.sequential({
    name = "sub_process",
    steps = { ... }
})

local main_workflow = Workflow.sequential({
    steps = {
        {
            name = "sub_workflow",
            type = "workflow",
            workflow = sub_workflow,
            input = { ... }
        }
    }
})
```

### Dynamic Workflow Creation

```lua
function create_workflow(params)
    local steps = {}
    
    for _, task in ipairs(params.tasks) do
        table.insert(steps, {
            name = task.name,
            type = "tool",
            tool = task.tool,
            input = task.params
        })
    end
    
    return Workflow.sequential({
        name = "dynamic_workflow",
        steps = steps
    })
end
```

## Error Handling

### Error Strategies

1. **fail_fast** (default) - Stop on first error
2. **continue** - Continue despite errors
3. **retry** - Retry failed steps

```lua
-- Simple strategy
error_strategy = "continue"

-- Retry configuration
error_strategy = {
    type = "retry",
    max_attempts = 3,
    backoff_ms = 1000,
    backoff_multiplier = 2
}
```

### Step-Level Error Handling

```lua
{
    name = "risky_step",
    type = "tool",
    tool = "some_tool",
    input = { ... },
    
    on_error = function(error)
        -- Log error
        Logger.error("Step failed", {error = error})
        
        -- Return fallback
        return {
            success = false,
            output = "default_value"
        }
    end,
    
    retry = {
        max_attempts = 3,
        backoff_ms = 500
    }
}
```

### Global Error Handling

```lua
local workflow = Workflow.sequential({
    steps = { ... },
    
    error_handler = function(step_name, error)
        -- Log the error
        Logger.error("Workflow error", {
            step = step_name,
            error = error
        })
        
        -- Decide action
        if error:match("timeout") then
            return "retry"
        else
            return "continue"
        end
    end
})
```

## Performance Tips

### 1. Use Parallel Workflows

```lua
-- Instead of sequential
local slow_workflow = Workflow.sequential({
    steps = {
        {name = "task1", ...},
        {name = "task2", ...},
        {name = "task3", ...}
    }
})

-- Use parallel when possible
local fast_workflow = Workflow.parallel({
    branches = {
        {name = "task1", steps = [{...}]},
        {name = "task2", steps = [{...}]},
        {name = "task3", steps = [{...}]}
    }
})
```

### 2. Batch Processing in Loops

```lua
-- Process in batches
local batch_workflow = Workflow.loop({
    iterator = { collection = large_dataset },
    batch_size = 10,  -- Process 10 items at once
    body = {
        {
            type = "custom",
            execute = function(context)
                -- context.batch contains 10 items
                return process_batch(context.batch)
            end
        }
    }
})
```

### 3. Minimize State Access

```lua
-- Cache state values
local cached_config = State.get("config")

local workflow = Workflow.loop({
    body = {
        {
            execute = function()
                -- Use cached_config instead of State.get("config")
                return process_with_config(cached_config)
            end
        }
    }
})
```

### 4. Use Appropriate Aggregation

```lua
-- For large loops
aggregation_strategy = "summary"  -- Less memory than "collect_all"

-- Custom aggregation
aggregation_strategy = {
    type = "custom",
    aggregate = function(results)
        return {
            total = #results,
            successful = count_successful(results)
        }
    end
}
```

## Complete Example

```lua
-- Create specialized agents
local analyzer = Agent.create({
    name = "data_analyzer",
    model = "openai/gpt-4",
    system_prompt = "You are a data analysis expert."
})

-- Register for reuse
Agent.register("analyzer", analyzer)

-- Create integrated workflow
local workflow = Workflow.sequential({
    name = "analysis_pipeline",
    
    steps = {
        -- Load data
        {
            name = "load_data",
            type = "tool",
            tool = "file_operations",
            input = { 
                operation = "read", 
                path = "/data.json" 
            }
        },
        
        -- Analyze with AI
        {
            name = "analyze",
            type = "agent",
            agent = Agent.get("analyzer"),
            input = {
                prompt = "Analyze: {{step:load_data:output}}"
            }
        },
        
        -- Process results in parallel
        {
            name = "process_results",
            type = "workflow",
            workflow = Workflow.parallel({
                branches = {
                    {
                        name = "save_analysis",
                        steps = [{
                            type = "tool",
                            tool = "file_operations",
                            input = {
                                operation = "write",
                                path = "/analysis.txt",
                                content = "{{step:analyze:output}}"
                            }
                        }]
                    },
                    {
                        name = "notify",
                        steps = [{
                            type = "custom",
                            execute = function()
                                Logger.info("Analysis complete!")
                                return { success = true }
                            end
                        }]
                    }
                }
            })
        }
    ],
    
    error_strategy = "retry",
    
    on_complete = function(success)
        Logger.info("Pipeline completed", {success = success})
    end
})

-- Execute
local result = workflow:execute()
```

---

**See Also**:
- [Tutorial: Agents & Workflows](tutorial-agents-workflows.md) - Step-by-step guide
- [Agent API Guide](agent-api.md) - Detailed agent documentation
- [Workflow API Guide](workflow-api.md) - Workflow patterns
- [Tool Reference](tool-reference.md) - Available tools
- [Examples](../../examples/) - Working code examples