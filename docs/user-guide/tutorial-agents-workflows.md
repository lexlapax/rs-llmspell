# Tutorial: Agents and Workflows

**Version**: Phase 3.3 Tutorial  
**Status**: ✅ **CURRENT** - Working with implemented features  
**Last Updated**: July 2025

> **📚 COMPREHENSIVE TUTORIAL**: Learn how to use Agents and Workflows in rs-llmspell through practical examples and patterns. From basic concepts to advanced techniques.

**🔗 Navigation**: [← User Guide](README.md) | [Documentation Hub](../README.md) | [Examples](../../examples/) | [API Reference](api-reference-agents-workflows.md)

---

## Table of Contents

1. [Getting Started](#getting-started)
2. [Working with Agents](#working-with-agents)
3. [Building Workflows](#building-workflows)
4. [Combining Agents and Workflows](#combining-agents-and-workflows)
5. [Advanced Patterns](#advanced-patterns)
6. [Enhancing with Hooks and Events](#enhancing-with-hooks-and-events)
7. [Best Practices](#best-practices)
8. [Troubleshooting](#troubleshooting)

## Getting Started

### Prerequisites

- rs-llmspell installed and configured
- Basic Lua knowledge
- API keys configured in `llmspell.toml`

### Running Examples

```bash
# Run a Lua script
llmspell run examples/lua/agents/agent-orchestrator.lua

# With specific configuration
llmspell run --config my-config.toml my-script.lua
```

## Working with Agents

### What are Agents?

Agents are AI-powered components that can understand context, make decisions, and generate responses. They're perfect for tasks requiring intelligence and reasoning.

### Your First Agent

```lua
-- Create a simple agent
local assistant = Agent.create({
    name = "helpful_assistant",
    model = "openai/gpt-3.5-turbo",  -- Note: provider/model format
    system_prompt = "You are a helpful assistant. Be concise and friendly.",
    temperature = 0.7
})

-- Use the agent
local response = assistant:execute({
    prompt = "What's the capital of France?"
})

print(response.output)  -- "The capital of France is Paris."
```

### Agent Patterns

#### 1. Specialized Agents

Create agents with specific expertise:

```lua
-- Data Analyst Agent
local analyst = Agent.create({
    name = "data_analyst",
    model = "openai/gpt-4",
    system_prompt = [[
You are a data analysis expert. When given data:
1. Identify patterns and trends
2. Calculate key statistics
3. Provide actionable insights
Always be precise and data-driven.
]],
    temperature = 0.3  -- Lower for more consistent analysis
})

-- Creative Writer Agent
local writer = Agent.create({
    name = "creative_writer",
    model = "anthropic/claude-3-haiku",
    system_prompt = "You are a creative writer. Write engaging, imaginative content.",
    temperature = 0.9  -- Higher for more creativity
})
```

#### 2. Agent with Tools

Agents can orchestrate tools:

```lua
local orchestrator = Agent.create({
    name = "tool_orchestrator",
    model = "openai/gpt-4",
    system_prompt = [[
You coordinate tools to accomplish tasks. Available tools:
- calculator: For mathematical calculations
- file_operations: For reading/writing files
- json_processor: For JSON manipulation
Break down tasks and use tools efficiently.
]]
})

-- Execute with tool coordination
local result = orchestrator:execute({
    prompt = [[
Calculate the sum of 125.50, 89.99, and 234.00, 
then save the result to a file called total.txt
]]
})
```

#### 3. Chain of Thought Agent

For complex reasoning:

```lua
local reasoner = Agent.create({
    name = "reasoner",
    model = "openai/gpt-4",
    system_prompt = [[
You solve problems step by step. Always:
1. Break down the problem
2. Show your reasoning
3. Verify your answer
Think out loud through each step.
]],
    temperature = 0.2
})
```

## Building Workflows

### What are Workflows?

Workflows orchestrate multiple steps, tools, and agents into automated processes. They provide structure and flow control.

### Sequential Workflows

Execute steps one after another:

```lua
local etl_workflow = Workflow.sequential({
    name = "etl_pipeline",
    
    steps = {
        -- Extract
        {
            name = "extract_data",
            type = "tool",
            tool = "file_operations",
            input = {
                operation = "read",
                path = "/tmp/raw_data.csv"
            }
        },
        
        -- Transform
        {
            name = "parse_csv",
            type = "tool",
            tool = "csv_analyzer",
            input = {
                input = "{{step:extract_data:output}}",
                operation = "parse"
            }
        },
        
        -- Load
        {
            name = "save_json",
            type = "tool",
            tool = "json_processor",
            input = {
                input = "{{step:parse_csv:output}}",
                operation = "stringify",
                pretty = true
            }
        }
    }
})

-- Execute the workflow
local result = etl_workflow:execute()
```

### Conditional Workflows

Make decisions based on conditions:

```lua
local router_workflow = Workflow.conditional({
    name = "request_router",
    
    branches = {
        -- High priority branch
        {
            name = "high_priority",
            condition = {
                type = "shared_data_equals",
                key = "priority",
                value = "high"
            },
            steps = {
                {
                    name = "immediate_process",
                    type = "custom",
                    execute = function()
                        print("Processing immediately!")
                        return { success = true }
                    end
                }
            }
        },
        
        -- Default branch
        {
            name = "normal_priority",
            condition = { type = "always" },
            steps = {
                {
                    name = "queue_process",
                    type = "custom",
                    execute = function()
                        print("Added to queue")
                        return { success = true }
                    end
                }
            }
        }
    }
})

-- Set condition and execute
State.set("priority", "high")
router_workflow:execute()
```

### Loop Workflows

Process collections or iterate:

```lua
-- Process each file in a directory
local files = {"data1.txt", "data2.txt", "data3.txt"}

local file_processor = Workflow.loop({
    name = "file_batch_processor",
    
    iterator = {
        collection = files
    },
    
    body = {
        {
            name = "process_file",
            type = "tool",
            tool = "file_operations",
            input = {
                operation = "read",
                path = "/tmp/{{loop:current_item}}"
            }
        },
        {
            name = "analyze",
            type = "custom",
            execute = function(context)
                local content = context.steps.process_file.output
                -- Process content
                return { 
                    success = true, 
                    output = "Processed: " .. context.current_item 
                }
            end
        }
    }
})

file_processor:execute()
```

### Parallel Workflows

Execute multiple branches concurrently:

```lua
local parallel_analysis = Workflow.parallel({
    name = "multi_analysis",
    
    branches = {
        {
            name = "statistical_analysis",
            steps = {
                {
                    name = "calculate_stats",
                    type = "tool",
                    tool = "calculator",
                    input = { 
                        operation = "evaluate",
                        input = "mean([10, 20, 30, 40, 50])" 
                    }
                }
            }
        },
        {
            name = "data_validation",
            steps = {
                {
                    name = "validate",
                    type = "tool",
                    tool = "data_validation",
                    input = {
                        operation = "validate",
                        input = {score = 85},
                        schema = {
                            type = "object",
                            properties = {
                                score = {type = "number", minimum = 0, maximum = 100}
                            }
                        }
                    }
                }
            }
        }
    },
    
    max_concurrency = 2
})

local result = parallel_analysis:execute()
print("Completed " .. result.data.successful_branches .. " analyses")
```

## Combining Agents and Workflows

### Agent-Driven Workflows

Use agents to make intelligent decisions within workflows:

```lua
-- Create analysis agent
local analyzer = Agent.create({
    name = "business_analyst",
    model = "openai/gpt-4",
    system_prompt = "Analyze business metrics and provide insights."
})

-- Workflow using the agent
local analysis_workflow = Workflow.sequential({
    name = "quarterly_analysis",
    
    steps = {
        -- Load data
        {
            name = "load_metrics",
            type = "tool",
            tool = "file_operations",
            input = {
                operation = "read",
                path = "/tmp/q4_metrics.json"
            }
        },
        
        -- AI Analysis
        {
            name = "analyze_metrics",
            type = "agent",
            agent = analyzer,
            input = {
                prompt = [[
Analyze these Q4 metrics:
{{step:load_metrics:output}}

Provide:
1. Key performance indicators
2. Trends compared to previous quarters
3. Recommendations for Q1
]]
            }
        },
        
        -- Save report
        {
            name = "save_analysis",
            type = "tool",
            tool = "file_operations",
            input = {
                operation = "write",
                path = "/tmp/q4_analysis.md",
                content = "{{step:analyze_metrics:output}}"
            }
        }
    }
})

analysis_workflow:execute()
```

### Multi-Agent Coordination

Coordinate multiple agents for complex tasks:

```lua
-- Create specialized agents
local researcher = Agent.create({
    name = "researcher",
    model = "openai/gpt-4",
    system_prompt = "Research topics thoroughly and provide comprehensive information."
})

local writer = Agent.create({
    name = "writer",
    model = "anthropic/claude-3-haiku",
    system_prompt = "Write clear, engaging content based on research."
})

local editor = Agent.create({
    name = "editor",
    model = "openai/gpt-4",
    system_prompt = "Edit content for clarity, correctness, and style."
})

-- Content creation workflow
local content_workflow = Workflow.sequential({
    name = "content_pipeline",
    
    steps = {
        {
            name = "research_topic",
            type = "agent",
            agent = researcher,
            input = {
                prompt = "Research the topic: Benefits of workflow automation"
            }
        },
        {
            name = "write_draft",
            type = "agent",
            agent = writer,
            input = {
                prompt = "Write an article based on: {{step:research_topic:output}}"
            }
        },
        {
            name = "edit_content",
            type = "agent",
            agent = editor,
            input = {
                prompt = "Edit and improve: {{step:write_draft:output}}"
            }
        }
    }
})
```

## Advanced Patterns

### 1. Dynamic Workflow Generation

Create workflows based on runtime conditions:

```lua
function create_processing_workflow(data_type)
    local steps = {
        {
            name = "load_data",
            type = "tool",
            tool = "file_operations",
            input = { 
                operation = "read", 
                path = "/tmp/data." .. data_type 
            }
        }
    }
    
    -- Add type-specific steps
    if data_type == "json" then
        table.insert(steps, {
            name = "parse_json",
            type = "tool",
            tool = "json_processor",
            input = { 
                operation = "parse",
                input = "{{step:load_data:output}}"
            }
        })
    elseif data_type == "csv" then
        table.insert(steps, {
            name = "parse_csv",
            type = "tool",
            tool = "csv_analyzer",
            input = { 
                operation = "parse",
                input = "{{step:load_data:output}}"
            }
        })
    end
    
    return Workflow.sequential({
        name = "dynamic_processor",
        steps = steps
    })
end

-- Use it
local json_workflow = create_processing_workflow("json")
json_workflow:execute()
```

### 2. Error Recovery Patterns

Implement robust error handling:

```lua
local resilient_workflow = Workflow.sequential({
    name = "resilient_processor",
    
    steps = {
        {
            name = "risky_operation",
            type = "custom",
            execute = function()
                if math.random() > 0.5 then
                    error("Random failure!")
                end
                return { success = true, output = "Success!" }
            end,
            
            -- Retry configuration
            retry = {
                max_attempts = 3,
                backoff_ms = 1000
            },
            
            -- Fallback on failure
            on_error = function(err)
                print("Operation failed, using fallback")
                return { success = true, output = "Fallback value" }
            end
        }
    },
    
    error_strategy = "continue"
})
```

### 3. State Management Patterns

Use state for complex workflows:

```lua
-- Initialize state
State.set("process_stats", {
    total_processed = 0,
    errors = 0,
    start_time = os.time()
})

local stateful_workflow = Workflow.loop({
    name = "stateful_processor",
    
    iterator = {
        range = { start = 1, ["end"] = 10, step = 1 }
    },
    
    body = {
        {
            name = "process_item",
            type = "custom",
            execute = function(context)
                -- Update state
                local stats = State.get("process_stats")
                stats.total_processed = stats.total_processed + 1
                State.set("process_stats", stats)
                
                return { 
                    success = true, 
                    output = "Processed item " .. context.current_value 
                }
            end
        }
    },
    
    on_complete = function()
        local stats = State.get("process_stats")
        stats.duration = os.time() - stats.start_time
        print(string.format(
            "Processed %d items in %d seconds",
            stats.total_processed,
            stats.duration
        ))
    end
})
```

### 4. Event-Driven Patterns (Phase 4 Preview)

📋 **Note**: The Event API is a Phase 4 feature and not yet available. Here's how it will work:

```lua
-- FUTURE FEATURE (Phase 4) - Not yet available
-- Event.on("data_received", function(data)
--     print("New data received: " .. data.source)
-- end)

-- CURRENT WORKAROUND - Use State polling
local function check_for_events()
    local events = State.get("pending_events") or {}
    for _, event in ipairs(events) do
        if event.type == "data_received" then
            -- Process event
            print("Processing data from: " .. event.source)
        end
    end
    State.set("pending_events", {})  -- Clear processed events
end

-- Simulate event emission using State
local function emit_event(event_type, data)
    local events = State.get("pending_events") or {}
    table.insert(events, {
        type = event_type,
        data = data,
        timestamp = os.time()
    })
    State.set("pending_events", events)
end

-- Use it
emit_event("data_received", { source = "api", data = "..." })
check_for_events()
```

## Enhancing with Hooks and Events

### Hook Integration for Agents

Hooks allow you to intercept and modify agent behavior at key points in their lifecycle:

```lua
-- Monitor agent execution
Hook.register("BeforeAgentExecution", function(context)
    print(string.format("[%s] Agent %s starting execution", 
        os.date("%H:%M:%S"), 
        context.component_id.name
    ))
    
    -- Add request ID for tracking
    return {
        action = "modified",
        modified_data = {
            metadata = {
                request_id = generateUUID(),
                start_time = os.time()
            }
        }
    }
end, "high")

-- Track costs
Hook.register("AfterAgentExecution", function(context)
    local tokens = context.data.tokens_used or {}
    local cost = calculate_cost(tokens.input or 0, tokens.output or 0)
    
    -- Emit cost event
    Event.publish("agent.cost", {
        agent = context.component_id.name,
        tokens = tokens,
        cost = cost
    })
    
    return "continue"
end, "low")

-- Handle agent errors gracefully
Hook.register("AgentError", function(context)
    local error_msg = context.data.error
    
    -- Retry on rate limits
    if error_msg:match("rate_limit") then
        return {
            action = "retry",
            max_attempts = 3,
            backoff_ms = 2000
        }
    end
    
    -- Log other errors
    Event.publish("agent.error", {
        agent = context.component_id.name,
        error = error_msg
    })
    
    return "continue"
end, "high")
```

### Event-Driven Workflows

Use events to coordinate complex workflows:

```lua
-- Create event-driven workflow coordinator
local WorkflowCoordinator = {
    init = function()
        -- Subscribe to workflow events
        local sub = Event.subscribe("workflow.*")
        
        Task.spawn(function()
            while true do
                local event = Event.receive(sub, 1000)
                if event then
                    if event.event_type == "workflow.stage.completed" then
                        -- Trigger next stage based on results
                        local next_stage = determine_next_stage(event.data)
                        if next_stage then
                            Event.publish("workflow.trigger", {
                                workflow = next_stage,
                                input = event.data.output
                            })
                        end
                    elseif event.event_type == "workflow.error" then
                        -- Handle workflow errors
                        handle_workflow_error(event.data)
                    end
                end
            end
        end)
    end
}

-- Usage
WorkflowCoordinator.init()

-- Create workflow that publishes events
local monitored_workflow = Workflow.sequential({
    name = "data_pipeline",
    
    -- Hook integration
    hooks = {
        before_stage = function(stage_name)
            Event.publish("workflow.stage.started", {
                workflow = "data_pipeline",
                stage = stage_name
            })
        end,
        
        after_stage = function(stage_name, result)
            Event.publish("workflow.stage.completed", {
                workflow = "data_pipeline",
                stage = stage_name,
                success = result.success,
                output = result.data
            })
        end
    },
    
    steps = {
        -- Your workflow steps
    }
})
```

### Cross-Component Coordination

Combine hooks and events for sophisticated patterns:

```lua
-- Rate limiting across all agents
local rate_limiter = {
    requests = {},
    limit = 100,  -- 100 requests per minute
    window = 60   -- 60 seconds
}

Hook.register("BeforeAgentExecution", function(context)
    local now = os.time()
    local window_start = now - rate_limiter.window
    
    -- Clean old requests
    local new_requests = {}
    for _, timestamp in ipairs(rate_limiter.requests) do
        if timestamp > window_start then
            table.insert(new_requests, timestamp)
        end
    end
    rate_limiter.requests = new_requests
    
    -- Check limit
    if #rate_limiter.requests >= rate_limiter.limit then
        Event.publish("rate_limit.exceeded", {
            component = "agents",
            current = #rate_limiter.requests,
            limit = rate_limiter.limit
        })
        
        return {
            action = "cancel",
            reason = "Rate limit exceeded"
        }
    end
    
    -- Track request
    table.insert(rate_limiter.requests, now)
    
    return "continue"
end, "highest")

-- Monitor rate limit events
local monitor_sub = Event.subscribe("rate_limit.exceeded")
Task.spawn(function()
    while true do
        local event = Event.receive(monitor_sub, 1000)
        if event then
            -- Could trigger alerts, scaling, etc.
            print("Rate limit hit:", event.data.component)
        end
    end
end)
```

### Performance Monitoring

Track performance across your entire system:

```lua
-- Global performance monitor
Hook.register("AfterAgentExecution", function(context)
    if context.data.duration_ms then
        Event.publish("performance.metric", {
            component_type = "agent",
            component_name = context.component_id.name,
            metric = "execution_time",
            value = context.data.duration_ms,
            tags = {
                model = context.data.model,
                success = context.data.success
            }
        })
    end
    return "continue"
end, "lowest")

Hook.register("AfterToolExecution", function(context)
    if context.data.duration_ms then
        Event.publish("performance.metric", {
            component_type = "tool",
            component_name = context.data.tool_name,
            metric = "execution_time",
            value = context.data.duration_ms
        })
    end
    return "continue"
end, "lowest")

-- Aggregate performance metrics
local metrics_sub = Event.subscribe("performance.metric")
local metrics_aggregator = {}

Task.spawn(function()
    while true do
        local event = Event.receive(metrics_sub, 100)
        if event then
            local key = event.data.component_type .. ":" .. event.data.component_name
            
            if not metrics_aggregator[key] then
                metrics_aggregator[key] = {
                    count = 0,
                    total_time = 0,
                    min_time = math.huge,
                    max_time = 0
                }
            end
            
            local metrics = metrics_aggregator[key]
            metrics.count = metrics.count + 1
            metrics.total_time = metrics.total_time + event.data.value
            metrics.min_time = math.min(metrics.min_time, event.data.value)
            metrics.max_time = math.max(metrics.max_time, event.data.value)
            
            -- Report every 100 executions
            if metrics.count % 100 == 0 then
                print(string.format(
                    "%s - Avg: %.2fms, Min: %.2fms, Max: %.2fms (n=%d)",
                    key,
                    metrics.total_time / metrics.count,
                    metrics.min_time,
                    metrics.max_time,
                    metrics.count
                ))
            end
        end
    end
end)
```

## Best Practices

### 1. Agent Design

- **Single Responsibility**: Each agent should have one clear purpose
- **Clear Prompts**: System prompts should be specific and actionable
- **Temperature Settings**: Lower for consistency, higher for creativity
- **Model Selection**: Use appropriate models for the task complexity

### 2. Workflow Design

- **Modular Steps**: Keep steps focused and reusable
- **Error Handling**: Always plan for failures
- **State Management**: Minimize shared state to avoid complexity
- **Performance**: Use parallel workflows when possible

### 3. Testing

```lua
-- Test individual components
local function test_agent()
    local agent = Agent.create({
        name = "test_agent",
        model = "openai/gpt-3.5-turbo",
        system_prompt = "You are a test assistant."
    })
    
    local result = agent:execute({
        prompt = "Say 'test passed'"
    })
    
    assert(result.success, "Agent execution failed")
    assert(result.output:find("test passed"), "Unexpected response")
    print("✓ Agent test passed")
end

-- Test workflows
local function test_workflow()
    local workflow = Workflow.sequential({
        name = "test_workflow",
        steps = {
            {
                name = "test_step",
                type = "custom",
                execute = function()
                    return { success = true, output = "OK" }
                end
            }
        }
    })
    
    local result = workflow:execute()
    assert(result.success, "Workflow failed")
    print("✓ Workflow test passed")
end

test_agent()
test_workflow()
```

## Troubleshooting

### Common Issues

#### 1. Agent Not Responding

```lua
-- Add timeout and error handling
local agent = Agent.create({
    name = "my_agent",
    model = "openai/gpt-4",
    system_prompt = "...",
    timeout = 30000  -- 30 second timeout
})

local success, result = pcall(function()
    return agent:execute({
        prompt = "..."
    })
end)

if not success then
    print("Agent error: " .. tostring(result))
    -- Fallback logic
end
```

#### 2. Workflow Step Failures

```lua
-- Debug workflow execution
local workflow = Workflow.sequential({
    name = "debug_workflow",
    
    steps = { ... },
    
    on_step_complete = function(step_name, result)
        print(string.format(
            "Step '%s': %s",
            step_name,
            result.success and "✓" or "✗"
        ))
        if not result.success then
            print("  Error: " .. (result.error or "Unknown"))
        end
    end
})
```

#### 3. State Conflicts

```lua
-- Use namespaced state keys
State.set("workflow_x.counter", 0)
State.set("workflow_y.counter", 0)

-- Or use local variables when possible
local counter = 0
```

### Performance Tips

1. **Batch Operations**: Process multiple items together
2. **Parallel Execution**: Use parallel workflows for independent tasks
3. **Caching**: Cache agent responses when appropriate
4. **Resource Limits**: Set appropriate timeouts and limits

```lua
-- Example: Optimized batch processing
local optimized_workflow = Workflow.parallel({
    name = "batch_processor",
    max_concurrency = 5,  -- Limit concurrent operations
    
    branches = create_branches_for_items(items),
    
    -- Aggregate results efficiently
    post_steps = {
        {
            name = "aggregate",
            type = "custom",
            execute = function(context)
                -- Process all results at once
                return { success = true }
            end
        }
    }
})
```

## Next Steps

1. Explore the [example files](../../examples/lua/) for working code
2. Read the [API Reference](api-reference-agents-workflows.md) for detailed documentation
3. Check the [Tool Reference](tool-reference.md) for available tools
4. Join the community for support and sharing

Happy automating with rs-llmspell! 🚀

---

**See Also**:
- [Agent API Guide](agent-api.md) - Detailed agent documentation
- [Workflow API Guide](workflow-api.md) - Workflow patterns and features
- [Tool Reference](tool-reference.md) - All 34 available tools
- [Examples Directory](../../examples/) - Working examples