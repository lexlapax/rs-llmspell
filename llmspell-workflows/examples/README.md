# Workflow Examples

This directory contains comprehensive examples demonstrating all workflow patterns and integration features in rs-llmspell.

## Directory Structure

```
examples/
├── sequential/          # Sequential workflow patterns
│   ├── basic_sequential_tools.lua    # Tool integration examples
│   ├── sequential_with_state.lua     # State management patterns
│   └── sequential_with_agents.lua    # Agent integration (Phase 3.3 preview)
├── conditional/         # Conditional workflow patterns
│   ├── basic_conditional.lua         # Branch-based execution
│   └── conditional_with_agents.lua   # AI-powered decision making
├── loop/               # Loop workflow patterns
│   ├── basic_loop.lua               # Range, collection, and while loops
│   └── loop_with_agents.lua         # Agent-based iteration
├── parallel/           # Parallel workflow patterns
│   ├── basic_parallel.lua           # Fork-join and concurrent execution
│   └── parallel_with_agents.lua     # Multi-agent collaboration
├── workflow_composition.lua          # Advanced composition patterns
├── error_handling.lua               # Error handling and debugging
├── performance_benchmarks.lua       # Performance testing
└── cross_workflow_coordination.lua  # Workflow orchestration patterns
```

## Quick Start

### Basic Sequential Workflow
```lua
local workflow = Workflow.sequential({
    name = "my_workflow",
    steps = {
        {
            name = "step1",
            type = "tool",
            tool = "calculator",
            input = { input = "10 + 20" }
        },
        {
            name = "step2",
            type = "tool",
            tool = "text_manipulator",
            input = {
                input = "Result: {{step:step1:output}}",
                operation = "uppercase"
            }
        }
    }
})

local result = workflow:execute()
```

### Conditional Workflow
```lua
local workflow = Workflow.conditional({
    name = "decision_workflow",
    branches = {
        {
            name = "high_value",
            condition = {
                type = "shared_data_greater_than",
                key = "value",
                value = 100
            },
            steps = [...]
        },
        {
            name = "default",
            condition = { type = "always" },
            steps = [...]
        }
    }
})
```

### Loop Workflow
```lua
local workflow = Workflow.loop({
    name = "batch_processor",
    iterator = {
        collection = {"item1", "item2", "item3"}
    },
    body = {
        {
            name = "process",
            type = "tool",
            tool = "text_manipulator",
            input = {
                input = "{{loop:current_item}}",
                operation = "uppercase"
            }
        }
    }
})
```

### Parallel Workflow
```lua
local workflow = Workflow.parallel({
    name = "concurrent_tasks",
    branches = {
        { name = "task1", steps = [...] },
        { name = "task2", steps = [...] },
        { name = "task3", steps = [...] }
    },
    max_concurrency = 3
})
```

## Integration Features

### Tool Integration
All 33+ tools from Phases 3.0-3.2 are fully integrated:
- File system tools (8)
- Data processing tools (4)
- Utility tools (9)
- System integration tools (4)
- API/Web tools (8)

### State Management
```lua
-- Set state
State.set("key", value)

-- Get state
local value = State.get("key")

-- Use in workflows
{
    name = "step",
    type = "tool",
    tool = "template_engine",
    input = {
        template = "Value: {{state:key}}",
        variables = {}
    }
}
```

### Agent Integration (Preview)
```lua
local agent = Agent.create({
    name = "analyzer",
    model = "gpt-4",
    system_prompt = "You are a data analyst"
})

{
    name = "analyze",
    type = "agent",
    agent = agent,
    input = {
        prompt = "Analyze this data: {{data}}"
    }
}
```

### Hook/Event Integration (Preview)
```lua
-- Register hooks
Hook.register("workflow.step.complete", function(data)
    print("Step completed: " .. data.step_name)
end)

-- Emit events
Event.emit("custom_event", { data = "value" })
```

## Performance Characteristics

Based on benchmarks:
- **Sequential**: ~0.5ms overhead per step
- **Parallel**: ~2ms setup + concurrent execution time
- **Conditional**: ~1ms per condition evaluation
- **Loop**: ~0.1ms per iteration

Maximum throughput: >1000 ops/sec for simple workflows

## Error Handling Strategies

1. **Fail Fast**: Stop on first error (default)
2. **Continue**: Continue despite errors
3. **Retry**: Retry with exponential backoff

```lua
error_strategy = "fail_fast"  -- or "continue" or retry config
```

## Advanced Patterns

### Workflow Composition
```lua
local etl_pipeline = Workflow.sequential({
    steps = {
        { name = "extract", type = "workflow", workflow = extractor },
        { name = "transform", type = "workflow", workflow = transformer },
        { name = "load", type = "workflow", workflow = loader }
    }
})
```

### Cross-Workflow Coordination
- Producer-Consumer pattern
- Pipeline orchestration
- Event-driven coordination
- Saga pattern for distributed transactions

## Running Examples

```bash
# Run individual examples
llmspell run examples/sequential/basic_sequential_tools.lua
llmspell run examples/parallel/basic_parallel.lua

# Run performance benchmarks
llmspell run examples/performance_benchmarks.lua

# Run all examples
./run_all_examples.sh
```

## Best Practices

1. **Keep workflows focused**: Single responsibility principle
2. **Use appropriate patterns**: Sequential for dependencies, parallel for independent tasks
3. **Handle errors gracefully**: Choose appropriate error strategy
4. **Monitor performance**: Use benchmarks to optimize
5. **Leverage composition**: Build complex workflows from simple ones
6. **Use state wisely**: Minimize shared state access in loops

## Further Reading

- [Workflow API Documentation](../../../docs/api/lua/workflow-global.md)
- [Tool Reference](../../../docs/api/tools/)
- [Architecture Guide](../../../docs/technical/rs-llmspell-final-architecture.md)