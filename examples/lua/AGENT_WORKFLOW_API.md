# Agent and Workflow API Reference

This document provides a comprehensive API reference for using Agents and Workflows in rs-llmspell Lua scripts.

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
    model = "gpt-4",                      -- Required: Model to use
    system_prompt = "You are...",         -- Required: System prompt
    temperature = 0.7,                    -- Optional: Temperature (0.0-1.0)
    max_tokens = 2000,                    -- Optional: Max response tokens
    timeout = 30000                       -- Optional: Timeout in ms
})
```

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
    on_error = function(err)              -- Optional: Error handler
        -- Handle error
    end
})

-- Result structure:
-- {
--     success = true/false,
--     content = "Response text",
--     usage = { tokens, etc },
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

### Workflow Composition

```lua
local workflow = Workflow.sequential({
    steps = {
        {
            name = "sub_workflow",
            type = "workflow",
            workflow = another_workflow,
            input = { ... }
        }
    }
})
```

### Dynamic Workflow Creation

```lua
function create_workflow(params)
    return Workflow.conditional({
        branches = params.conditions.map(function(cond)
            return {
                name = cond.name,
                condition = parse_condition(cond),
                steps = create_steps(cond.actions)
            }
        end)
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
        print("Error: " .. tostring(error))
        
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

## Performance Tips

### 1. Use Parallel Workflows

```lua
-- Instead of sequential
for _, item in ipairs(items) do
    process(item)  -- Slow
end

-- Use parallel
Workflow.parallel({
    branches = items.map(function(item)
        return { steps = { process_step(item) } }
    end)
})
```

### 2. Batch Processing in Loops

```lua
Workflow.loop({
    iterator = { collection = large_dataset },
    batch_size = 10,  -- Process 10 items at once
    body = { ... }
})
```

### 3. Minimize State Access

```lua
-- Cache state values
local cached_value = State.get("key")

Workflow.loop({
    body = {
        {
            execute = function()
                -- Use cached_value instead of State.get()
            end
        }
    }
})
```

### 4. Use Appropriate Aggregation

```lua
-- For large loops, use summary instead of collect_all
aggregation_strategy = "summary"  -- Less memory usage
```

## Complete Example

```lua
-- Create specialized agents
local analyzer = Agent.create({
    name = "data_analyzer",
    model = "gpt-4",
    system_prompt = "You are a data analysis expert."
})

-- Create integrated workflow
local workflow = Workflow.sequential({
    name = "analysis_pipeline",
    
    steps = {
        -- Load data
        {
            name = "load_data",
            type = "tool",
            tool = "file_operations",
            input = { operation = "read", path = "/data.json" }
        },
        
        -- Analyze with AI
        {
            name = "analyze",
            type = "agent",
            agent = analyzer,
            input = {
                prompt = "Analyze: {{step:load_data:output}}"
            }
        },
        
        -- Process results in parallel
        {
            name = "process_results",
            type = "parallel",
            workflow = Workflow.parallel({
                branches = {
                    {
                        name = "save_analysis",
                        steps = {{
                            type = "tool",
                            tool = "file_operations",
                            input = {
                                operation = "write",
                                path = "/analysis.txt",
                                content = "{{step:analyze:output}}"
                            }
                        }}
                    },
                    {
                        name = "notify",
                        steps = {{
                            type = "custom",
                            execute = function()
                                print("Analysis complete!")
                                return { success = true }
                            end
                        }}
                    }
                }
            })
        }
    ],
    
    error_strategy = "retry",
    
    on_complete = function(success)
        print("Pipeline " .. (success and "succeeded" or "failed"))
    end
})

-- Execute
local result = workflow:execute()
```