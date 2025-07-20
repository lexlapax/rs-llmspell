# Workflow Bridge Guide

The Workflow Bridge provides a comprehensive interface for scripts to interact with workflow systems in rs-llmspell. It enables script-based workflow creation, execution, and management with full support for multi-agent coordination patterns.

## Table of Contents

1. [Overview](#overview)
2. [Architecture](#architecture)
3. [Workflow Types](#workflow-types)
4. [Script Integration](#script-integration)
5. [Multi-Agent Coordination](#multi-agent-coordination)
6. [Performance Optimization](#performance-optimization)
7. [API Reference](#api-reference)
8. [Examples](#examples)

## Overview

The Workflow Bridge serves as the primary interface between scripting languages (Lua, JavaScript) and the rs-llmspell workflow system. It provides:

- **Workflow Discovery**: Find available workflow types and their capabilities
- **Workflow Management**: Create, execute, and manage workflow instances
- **Multi-Agent Coordination**: Orchestrate multiple agents through workflow patterns
- **Performance Optimization**: Sub-10ms operation overhead with caching and validation
- **Script Integration**: Consistent API across Lua and JavaScript environments

## Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                         Script Layer (Lua/JS)                    │
├─────────────────────────────────────────────────────────────────┤
│                        Workflow Bridge API                       │
│  ┌──────────────┐  ┌──────────────┐  ┌────────────────────┐  │
│  │  Discovery   │  │   Factory    │  │    Execution       │  │
│  │  Service     │  │   Service    │  │    Engine          │  │
│  └──────────────┘  └──────────────┘  └────────────────────┘  │
├─────────────────────────────────────────────────────────────────┤
│                     Workflow Core Components                     │
│  ┌──────────────┐  ┌──────────────┐  ┌────────────────────┐  │
│  │ Sequential   │  │ Conditional  │  │    Parallel        │  │
│  │ Workflow     │  │ Workflow     │  │    Workflow        │  │
│  └──────────────┘  └──────────────┘  └────────────────────┘  │
└─────────────────────────────────────────────────────────────────┘
```

### Key Components

1. **WorkflowBridge**: Main entry point for script interaction
2. **WorkflowDiscovery**: Service for discovering available workflow types
3. **WorkflowFactory**: Creates workflow instances from parameters
4. **WorkflowExecutor**: Executes workflows with input data
5. **Performance Layer**: Caching, validation, and optimization

## Workflow Types

### Sequential Workflow
Execute steps in order, with each step's output feeding into the next.

```lua
local workflow = Workflow.sequential({
    name = "data_pipeline",
    steps = {
        {name = "fetch", tool = "web_fetch"},
        {name = "process", agent = "processor"},
        {name = "store", tool = "file_writer"}
    }
})
```

### Conditional Workflow
Branch execution based on conditions.

```lua
local workflow = Workflow.conditional({
    name = "smart_router",
    condition = {
        type = "expression",
        expression = "$input.priority > 5"
    },
    then_branch = {agent = "urgent_handler"},
    else_branch = {agent = "normal_handler"}
})
```

### Loop Workflow
Iterate over collections or conditions.

```lua
local workflow = Workflow.loop({
    name = "batch_processor",
    iterator = {
        type = "collection",
        items = "$input.items"
    },
    body = {
        steps = [{agent = "item_processor"}]
    }
})
```

### Parallel Workflow
Execute multiple branches concurrently.

```lua
local workflow = Workflow.parallel({
    name = "multi_analysis",
    branches = {
        {name = "sentiment", agent = "sentiment_analyzer"},
        {name = "facts", agent = "fact_checker"},
        {name = "style", agent = "style_analyzer"}
    },
    max_concurrency = 3
})
```

## Script Integration

### Lua API

The Lua API provides a data-oriented interface for workflow management:

```lua
-- List available workflow types
local types = Workflow.types()

-- Get workflow information
local info = Workflow.info("sequential")

-- Create and execute workflow
local result = Workflow.execute({
    type = "sequential",
    name = "my_workflow",
    steps = [...]
}, {
    input_data = "test"
})

-- Access execution history
local history = Workflow.history()
```

### Bridge Architecture

The bridge uses several optimization techniques:

1. **Parameter Validation**: Fast validation before workflow creation
2. **Execution Caching**: Cache results for identical workflows
3. **Type Discovery Caching**: Cache workflow type information
4. **Performance Metrics**: Track and optimize operation times

## Multi-Agent Coordination

The workflow bridge supports sophisticated multi-agent patterns:

### Pipeline Pattern
Sequential agent collaboration where each agent processes and enriches data:

```lua
local pipeline = create_pipeline_workflow(
    "research_pipeline",
    {"research_agent", "analysis_agent", "summary_agent"},
    {topic = "AI in healthcare"}
)
```

### Fork-Join Pattern
Parallel agent execution with result aggregation:

```lua
local parallel = create_fork_join_workflow(
    "document_analysis",
    {
        {"sentiment_agent", "analyze_sentiment", {text = doc}},
        {"fact_checker", "verify_facts", {claims = claims}},
        {"style_agent", "analyze_style", {document = doc}}
    },
    "coordinator_agent"
)
```

### Consensus Pattern
Multiple agents evaluate options and reach consensus:

```lua
local consensus = create_consensus_workflow(
    "investment_decision",
    {"financial_expert", "market_expert", "tech_expert"},
    0.7, -- 70% consensus threshold
    investment_options
)
```

## Performance Optimization

The workflow bridge is optimized to maintain <10ms overhead:

### Optimization Strategies

1. **Parameter Validation Cache**
   - Pre-compiled validators for common workflow types
   - Skip full parsing for known-good parameters

2. **Execution Result Cache**
   - LRU cache for recent workflow executions
   - 60-second TTL for cached results
   - Configurable cache size (default: 100 entries)

3. **Type Discovery Cache**
   - Static cache for workflow type information
   - Eliminates repeated discovery operations

4. **Performance Metrics**
   - Real-time tracking of operation durations
   - P99 latency monitoring
   - Automatic performance boundary checks

### Performance Monitoring

```lua
-- Get performance metrics
local metrics = Workflow.metrics()
print("Average operation: " .. metrics.average_duration_ms .. "ms")
print("P99 latency: " .. metrics.p99_duration_ms .. "ms")
print("Within bounds: " .. tostring(metrics.is_within_10ms_target))
```

## API Reference

### WorkflowBridge Methods

#### `list_workflow_types() -> Vec<String>`
Returns a list of available workflow types.

#### `get_workflow_info(workflow_type: &str) -> Option<WorkflowInfo>`
Get detailed information about a specific workflow type.

#### `create_workflow(workflow_type: &str, params: Value) -> Result<String>`
Create a new workflow instance. Returns workflow ID.

#### `execute_workflow(workflow_id: &str, input: Value) -> Result<Value>`
Execute a workflow with the given input.

#### `execute_workflow_oneshot(workflow_type: &str, params: Value, input: Value) -> Result<Value>`
Create, execute, and clean up a workflow in one operation.

#### `get_execution_history() -> Vec<WorkflowExecutionRecord>`
Get the execution history for all workflows.

#### `get_bridge_metrics() -> Value`
Get comprehensive bridge performance metrics.

#### `get_performance_metrics() -> Value`
Get performance-specific metrics.

### Multi-Agent Workflow Functions

#### `create_pipeline_workflow(name: String, agents: Vec<String>, input: Value) -> Result<Box<dyn Workflow>>`
Create a sequential pipeline of agents.

#### `create_fork_join_workflow(name: String, tasks: Vec<(String, String, Value)>, coordinator: Option<String>) -> Result<Box<dyn Workflow>>`
Create a parallel execution pattern with optional coordination.

#### `create_consensus_workflow(name: String, evaluators: Vec<String>, threshold: f64, options: Value) -> Result<Box<dyn Workflow>>`
Create a consensus evaluation workflow.

## Examples

### Basic Sequential Workflow

```lua
-- Create a data processing pipeline
local workflow = Workflow.sequential({
    name = "data_processor",
    steps = {
        {
            name = "fetch_data",
            tool = "http_client",
            parameters = {url = "https://api.example.com/data"}
        },
        {
            name = "transform",
            agent = "data_transformer",
            parameters = {format = "json"}
        },
        {
            name = "save",
            tool = "file_writer",
            parameters = {path = "/tmp/output.json"}
        }
    },
    error_strategy = "continue"
})

local result = Workflow.execute(workflow, {})
```

### Multi-Agent Research Workflow

```lua
-- Create a research workflow with multiple specialized agents
local research_flow = Workflow.sequential({
    name = "comprehensive_research",
    steps = {
        {
            name = "gather_sources",
            type = "parallel",
            branches = {
                {agent = "web_researcher", task = "search_academic"},
                {agent = "news_researcher", task = "search_news"},
                {agent = "social_researcher", task = "search_social"}
            }
        },
        {
            name = "analyze_data",
            agent = "research_analyst",
            parameters = {
                sources = "$gather_sources.outputs",
                analysis_depth = "comprehensive"
            }
        },
        {
            name = "generate_report",
            agent = "report_generator",
            parameters = {
                analysis = "$analyze_data.output",
                format = "executive_summary"
            }
        }
    }
})
```

### Conditional Processing with Loops

```lua
-- Process items with conditional handling
local processor = Workflow.sequential({
    name = "smart_processor",
    steps = {
        {
            name = "categorize",
            agent = "categorizer",
            parameters = {items = "$input.items"}
        },
        {
            name = "process_by_category",
            type = "loop",
            iterator = {
                type = "collection",
                items = "$categorize.output.categories"
            },
            body = {
                type = "conditional",
                condition = {
                    expression = "$item.priority == 'high'"
                },
                then_branch = {
                    agent = "priority_processor",
                    parameters = {item = "$item"}
                },
                else_branch = {
                    agent = "standard_processor",
                    parameters = {item = "$item"}
                }
            }
        }
    }
})
```

### Performance Monitoring Example

```lua
-- Monitor workflow performance
local workflow_id = Workflow.create("sequential", {
    name = "perf_test",
    steps = [{tool = "mock_tool"}]
})

-- Execute multiple times
for i = 1, 100 do
    Workflow.execute(workflow_id, {test = i})
end

-- Check performance
local perf = Workflow.performance()
print(string.format(
    "Operations: Avg %.2fms, P99 %.0fms, Within target: %s",
    perf.average_duration_ms,
    perf.p99_duration_ms,
    perf.is_within_10ms_target and "YES" or "NO"
))
```

## Best Practices

1. **Use Workflow.execute() for one-shot operations** to avoid manual cleanup
2. **Cache workflow definitions** when executing the same workflow multiple times
3. **Monitor performance metrics** to ensure operations stay within bounds
4. **Use appropriate error strategies** (fail_fast, continue, retry) based on your use case
5. **Leverage multi-agent patterns** for complex orchestration scenarios
6. **Validate parameters** before creating workflows to catch errors early

## Troubleshooting

### Common Issues

1. **Workflow creation fails**
   - Check parameter validation using `Workflow.info(type)`
   - Ensure all required parameters are provided
   - Verify agent/tool names exist in the registry

2. **Performance degradation**
   - Check cache hit rates in metrics
   - Verify workflow complexity isn't excessive
   - Monitor P99 latencies for outliers

3. **Multi-agent coordination issues**
   - Ensure agents are registered and available
   - Check agent compatibility with workflow patterns
   - Verify parameter passing between agents

## Future Enhancements

- JavaScript API implementation (Task 3.3.16)
- Advanced workflow composition patterns
- Distributed workflow execution
- Workflow versioning and migration
- Visual workflow designer integration