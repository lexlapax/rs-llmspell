# Workflow Unified Architecture

## Overview

This document describes the unified workflow architecture implemented in Phase 7.3.12.8, which consolidates workflow management, execution, and state handling into a cohesive system.

## Architecture Components

### 1. Core Workflow System (`llmspell-workflows`)

The workflow system provides four fundamental workflow types:

- **Sequential**: Execute steps one after another in order
- **Parallel**: Execute multiple branches concurrently  
- **Conditional**: Execute different branches based on conditions
- **Loop**: Execute steps repeatedly based on iteration criteria

Each workflow type implements the `Workflow` trait from `llmspell-core` and supports:
- State persistence via `StateManager`
- Event emission through the event bus
- Nested workflow composition
- Error handling and retry policies

### 2. Bridge Layer (`llmspell-bridge`)

The bridge layer provides script language integration through `WorkflowBridge`:

```rust
pub struct WorkflowBridge {
    registry: Arc<ComponentRegistry>,
    state_manager: Option<Arc<StateManager>>,
    event_bus: Option<Arc<EventBus>>,
}
```

Key responsibilities:
- Workflow registration and discovery
- Lua/JavaScript API exposure
- State context propagation
- Event integration

### 3. State Management Integration

Workflows integrate with the state persistence layer through:

- **Unified State Context**: Single `StateManager` instance shared across all workflows
- **Scoped State Access**: Each workflow step gets its own state scope
- **State Propagation**: Parent workflow state is accessible to nested workflows
- **Persistence**: Automatic state persistence based on configuration

## ID Scheme and Naming Conventions

### Workflow ID Prefixing

The system uses intelligent ID prefixing to disambiguate between different component types:

#### Automatic Prefixing Rules

1. **Workflow IDs**: Automatically prefixed with `workflow_` if not already present
   ```lua
   -- Both create workflow with ID "workflow_data_processor"
   local w1 = Workflow.sequential({id = "data_processor"})
   local w2 = Workflow.sequential({id = "workflow_data_processor"})
   ```

2. **Agent IDs**: No automatic prefixing (agents manage their own IDs)
   ```lua
   local agent = Agent.builder():id("analyzer"):build()
   -- ID remains "analyzer"
   ```

3. **Tool IDs**: No automatic prefixing (tools are registered with exact names)
   ```lua
   Tool.execute("file-reader", params)
   -- Looks for tool with exact ID "file-reader"
   ```

#### ID Resolution in Nested Workflows

When workflows reference other components:

```lua
local workflow = Workflow.sequential({
    id = "parent",
    steps = {
        {
            id = "step1",
            type = "workflow",
            workflow_id = "child"  -- Resolves to "workflow_child"
        },
        {
            id = "step2", 
            type = "agent",
            agent_id = "processor"  -- Resolves to exact "processor"
        }
    }
})
```

### Component Registry Lookup

The `ComponentRegistry` maintains separate namespaces:

```rust
pub struct ComponentRegistry {
    workflows: HashMap<ComponentId, Arc<dyn Workflow>>,
    agents: HashMap<ComponentId, Arc<dyn Agent>>,
    tools: HashMap<String, Arc<dyn Tool>>,
}
```

Lookup behavior:
- **Workflows**: Try with prefix first, then without
- **Agents**: Exact match only
- **Tools**: Exact string match (not ComponentId)

## Lua API

### Workflow Creation

```lua
-- Builder pattern
local workflow = Workflow.builder()
    :id("my-workflow")
    :type("sequential")
    :add_step({
        id = "step1",
        action = function(input, context)
            return {success = true, result = "done"}
        end
    })
    :build()

-- Direct creation methods
local seq = Workflow.sequential({
    id = "seq-workflow",
    steps = {...}
})

local cond = Workflow.conditional({
    id = "cond-workflow", 
    condition = function(input) return input.value > 10 end,
    if_branch = {...},
    else_branch = {...}
})
```

### Workflow Discovery

```lua
-- List all registered workflows
local workflows = Workflow.list()
for i, w in ipairs(workflows) do
    print(w.id, w.type, w.description)
end

-- Get specific workflow
local workflow = Workflow.get("workflow_data_processor")
```

### Table-Based Conditions

Conditional workflows support table-based condition definitions:

```lua
local workflow = Workflow.conditional({
    id = "router",
    condition = {
        type = "expression",
        expression = "input.priority == 'high'",
        fallback = false
    },
    if_branch = {...},
    else_branch = {...}
})

-- Or with function
local workflow = Workflow.conditional({
    id = "router",
    condition = {
        type = "function",
        evaluator = function(input, context)
            return context.state:read("threshold") > 100
        end
    },
    if_branch = {...},
    else_branch = {...}
})
```

## Nested Workflow Support

### Architecture

Nested workflows are supported through the `WorkflowStep` type:

```rust
pub enum WorkflowStep {
    Action(StepAction),
    Agent(AgentStep),
    Workflow(NestedWorkflowStep),
    Tool(ToolStep),
}
```

### Execution Flow

1. **Parent Workflow** creates execution context with state/events
2. **Nested Workflow Step** detected during execution
3. **Child Workflow** retrieved from ComponentRegistry
4. **Context Propagation**: Parent's state manager and event bus passed to child
5. **Result Aggregation**: Child's output merged into parent's state

### Example

```lua
local parent = Workflow.sequential({
    id = "parent",
    steps = {
        {
            id = "prepare",
            action = function(input, context)
                context.state:write("status", "preparing")
                return {success = true}
            end
        },
        {
            id = "process",
            type = "workflow",
            workflow = {
                type = "parallel",
                branches = {
                    {id = "b1", action = processA},
                    {id = "b2", action = processB}
                }
            }
        },
        {
            id = "complete",
            action = function(input, context)
                local status = context.state:read("status")
                return {success = true, result = status}
            end
        }
    }
})
```

## State Persistence Architecture

### State Hierarchy

```
StateManager (root)
├── Workflow State (workflow_id)
│   ├── Step State (step_id)
│   │   └── Nested Workflow State
│   └── Shared Data (accessible to all steps)
└── Global State (cross-workflow)
```

### State Access Patterns

1. **Step-Local State**: Each step has isolated state
   ```lua
   context.state:write("step_data", value)  -- Scoped to current step
   ```

2. **Workflow-Shared State**: Accessible across all steps
   ```lua
   context.shared:write("workflow_data", value)  -- Shared within workflow
   ```

3. **Parent State Access**: Nested workflows can access parent state
   ```lua
   context.parent_state:read("parent_data")  -- Read from parent workflow
   ```

## Event Integration

### Event Flow

1. **Workflow Start**: Emits `workflow.started` event
2. **Step Execution**: Emits `workflow.step.started` and `workflow.step.completed`
3. **Workflow Complete**: Emits `workflow.completed` with results
4. **Error Handling**: Emits `workflow.failed` with error details

### Event Correlation

All events within a workflow execution share a correlation ID:

```json
{
  "event_type": "workflow.step.completed",
  "component_id": "workflow_data_processor",
  "correlation_id": "550e8400-e29b-41d4-a716-446655440000",
  "data": {
    "step_id": "transform",
    "duration_ms": 125,
    "result": {...}
  }
}
```

## Error Handling

### Retry Policies

Workflows support configurable retry policies:

```lua
local workflow = Workflow.sequential({
    id = "resilient",
    retry_policy = {
        max_attempts = 3,
        backoff = "exponential",
        initial_delay_ms = 100
    },
    steps = {...}
})
```

### Error Propagation

- **Step Failure**: Can be caught and handled within workflow
- **Workflow Failure**: Propagates to parent or caller
- **Nested Failure**: Bubbles up through workflow hierarchy

## Performance Considerations

### Optimization Strategies

1. **Lazy Loading**: Workflows loaded from registry only when needed
2. **State Caching**: Frequently accessed state cached in memory
3. **Parallel Execution**: True concurrent execution for parallel workflows
4. **Event Batching**: Events batched for efficient transmission

### Resource Management

- **Memory**: Workflows released after execution unless registered
- **State**: Automatic cleanup of temporary state after workflow completion
- **Events**: Event buffer limits prevent memory exhaustion

## Testing Support

### Test Utilities

The `llmspell-testing` crate provides workflow testing utilities:

```rust
use llmspell_testing::workflow_test_utils::*;

#[tokio::test]
async fn test_nested_workflow() {
    let harness = WorkflowTestHarness::new()
        .with_state()
        .with_events();
    
    let workflow = create_test_workflow();
    let result = harness.execute(workflow, test_input()).await;
    
    assert!(result.is_success());
    assert_eq!(harness.events_emitted(), 5);
}
```

### Debug Support

Enable workflow debugging:

```lua
Workflow.enableDebug(true)
local workflow = Workflow.sequential({...})
-- Detailed execution logging enabled
```

## Migration from Previous Architecture

### Key Changes from Pre-7.3.12.8

1. **Unified State**: Single StateManager instead of per-workflow
2. **ID Prefixing**: Automatic `workflow_` prefix handling
3. **Simplified API**: Consistent builder pattern across all types
4. **Event Integration**: Built-in event emission without manual setup
5. **Registry-Based**: All workflows registered in ComponentRegistry

### Breaking Changes

- Workflow IDs now auto-prefixed (may affect lookups)
- State access API changed from `workflow.state` to `context.state`
- Event emission now automatic (remove manual event calls)

## Future Enhancements

### Phase 8 Considerations

- Workflow versioning and migration
- Distributed workflow execution
- Workflow templates and inheritance
- Visual workflow designer integration

### Phase 9-16 Roadmap

- Cross-language workflow definitions
- Workflow marketplace integration  
- AI-powered workflow optimization
- Real-time workflow modification