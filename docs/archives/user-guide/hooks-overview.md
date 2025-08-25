# Workflow Hooks System (Phase 4)

**Version**: 1.0  
**Date**: July 2025  
**Status**: Design Prepared (Implementation in Phase 4)  

> **📋 Note**: This document describes the workflow hooks system that will be available in Phase 4. Currently, you can use State and Event globals for similar functionality.

## Overview

The workflow hooks system will provide a powerful mechanism for monitoring, controlling, and extending workflow execution. Hooks will be available at key lifecycle points, enabling scripts to react to workflow events, modify behavior, and collect metrics.

### Key Features

- **20+ Hook Points**: Comprehensive coverage of workflow lifecycle
- **Script Accessible**: Register hooks directly from Lua/JavaScript
- **Performance Optimized**: <2ms overhead per hook execution
- **Type Safe**: Structured context data for each hook point
- **Async Support**: Hooks can perform async operations

## Hook Points

### Workflow Lifecycle Hooks

#### 1. **before_start**
Triggered before workflow begins execution.

```lua
Hook.register("workflow:before_start", function(context)
    -- context.workflow_id: string
    -- context.workflow_name: string
    -- context.workflow_type: string
    -- context.state: table
    return {
        continue_execution = true,
        state_updates = {initial_time = os.time()}
    }
end)
```

#### 2. **after_complete**
Triggered after workflow completes successfully.

```lua
Hook.register("workflow:after_complete", function(context)
    -- context.workflow_id: string
    -- context.total_duration_ms: number
    -- context.steps_completed: number
    -- context.final_output: any
    Logger.info("Workflow completed", {
        id = context.workflow_id,
        duration = context.total_duration_ms
    })
end)
```

#### 3. **on_error**
Triggered when workflow encounters an unrecoverable error.

```lua
Hook.register("workflow:on_error", function(context)
    -- context.error: string
    -- context.error_type: string
    -- context.failed_step: string (optional)
    Event.emit("workflow_failed", {
        workflow_id = context.workflow_id,
        error = context.error
    })
end)
```

### Step Lifecycle Hooks

#### 4. **before_step**
Triggered before each step executes.

```lua
Hook.register("workflow:before_step", function(context)
    -- context.step.name: string
    -- context.step.index: number
    -- context.step.type: string ("tool", "agent", "workflow")
    -- context.step.input: any
    
    -- Check cache
    local cache_key = "step:" .. context.step.name
    local cached = State.get(cache_key)
    if cached then
        return {
            continue_execution = false,
            state_updates = {
                [context.step.name .. "_output"] = cached
            }
        }
    end
    
    return {continue_execution = true}
end)
```

#### 5. **after_step**
Triggered after each step completes.

```lua
Hook.register("workflow:after_step", function(context)
    -- context.step.output: any
    -- context.step.duration_ms: number
    
    -- Cache result
    State.set("step:" .. context.step.name, context.step.output)
    
    -- Collect metrics
    local metrics = State.get("metrics") or {}
    table.insert(metrics, {
        step = context.step.name,
        duration = context.step.duration_ms
    })
    State.set("metrics", metrics)
end)
```

#### 6. **on_step_error**
Triggered when a step fails.

```lua
Hook.register("workflow:on_step_error", function(context)
    -- context.step.error: string
    -- context.step.retry_count: number
    
    if context.step.retry_count < 3 then
        Logger.warn("Step failed, will retry", {
            step = context.step.name,
            attempt = context.step.retry_count + 1
        })
        return {
            continue_execution = true,
            retry_step = true
        }
    end
    
    return {continue_execution = false}
end)
```

### Workflow-Specific Hooks

#### 7. **parallel:branch_start**
For parallel workflows, triggered when a branch starts.

```lua
Hook.register("parallel:branch_start", function(context)
    -- context.branch_name: string
    -- context.branch_index: number
    -- context.total_branches: number
end)
```

#### 8. **conditional:condition_evaluated**
For conditional workflows, triggered after condition evaluation.

```lua
Hook.register("conditional:condition_evaluated", function(context)
    -- context.condition_result: boolean
    -- context.selected_branch: string
    -- context.evaluation_time_ms: number
end)
```

#### 9. **loop:iteration_complete**
For loop workflows, triggered after each iteration.

```lua
Hook.register("loop:iteration_complete", function(context)
    -- context.iteration: number
    -- context.continue_loop: boolean
    -- context.iteration_output: any
end)
```

## Future Enhancements

### Phase 5 Integration
- Persistent hook state
- Cross-workflow hook sharing
- Hook versioning

### Phase 6 Integration
- Session-aware hooks
- Hook replay for debugging
- Visual hook monitoring

## Examples

Complete examples will be available in Phase 4:
- `workflow_monitoring.lua` - Performance monitoring setup
- `error_recovery.lua` - Advanced error handling
- `conditional_execution.lua` - Dynamic workflow control
- `hook_composition.lua` - Combining multiple hook patterns

## Current Workarounds

Until Phase 4, you can achieve similar functionality using:

### Using State for Monitoring
```lua
-- Track workflow execution
local function with_monitoring(workflow_fn)
    return function(...)
        State.set("workflow_start", os.time())
        local result = workflow_fn(...)
        local duration = os.time() - State.get("workflow_start")
        Logger.info("Workflow completed", {duration = duration})
        return result
    end
end
```

### Using Event for Notifications
```lua
-- Emit events for workflow milestones
Event.emit("workflow:started", {name = "my_workflow"})
-- ... workflow execution ...
Event.emit("workflow:completed", {name = "my_workflow", duration = 100})
```