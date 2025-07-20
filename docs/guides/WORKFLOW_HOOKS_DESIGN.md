# Workflow Hooks System Design (Phase 4)

**Version**: 1.0  
**Date**: July 2025  
**Status**: Design Prepared (Implementation in Phase 4)  

> **📋 Design Document**: This document outlines the workflow hooks system that will be implemented in Phase 4, allowing comprehensive lifecycle management and monitoring of workflows.

---

## Overview

The workflow hooks system will provide a powerful mechanism for monitoring, controlling, and extending workflow execution. Hooks will be available at key lifecycle points, enabling scripts to react to workflow events, modify behavior, and collect metrics.

### Key Features

- **20+ Hook Points**: Comprehensive coverage of workflow lifecycle
- **Script Accessible**: Register hooks directly from Lua/JavaScript
- **Performance Optimized**: <2ms overhead per hook execution
- **Type Safe**: Structured context data for each hook point
- **Async Support**: Hooks can perform async operations

---

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

---

## Hook Context Structure

Each hook receives a context object with relevant information:

```lua
context = {
    -- Common fields
    workflow_id = "uuid",
    workflow_name = "my_workflow",
    workflow_type = "sequential",
    hook_point = "before_step",
    timestamp = "2025-07-20T10:00:00Z",
    
    -- Workflow state
    state = {
        -- Current workflow state
    },
    
    -- Step information (if applicable)
    step = {
        name = "process_data",
        index = 2,
        type = "tool",
        input = {...},
        output = {...},  -- Only in after_step
        duration_ms = 150,  -- Only in after_step
        error = "...",  -- Only in on_step_error
    },
    
    -- Additional metadata
    metadata = {
        -- Custom metadata
    }
}
```

---

## Hook Return Values

Hooks can control workflow execution by returning specific values:

```lua
return {
    -- Continue or halt execution
    continue_execution = true,  -- or false to stop
    
    -- Update workflow state
    state_updates = {
        key = "value"
    },
    
    -- Retry the current step (for error hooks)
    retry_step = true,
    
    -- Skip to a specific step
    skip_to_step = "step_name",
    
    -- Log a message
    message = "Hook executed successfully"
}
```

---

## Implementation Patterns

### Pattern 1: Performance Monitoring

```lua
-- Global performance monitor
local PerformanceMonitor = {
    init = function()
        Hook.register("workflow:before_start", function(ctx)
            State.set("perf:" .. ctx.workflow_id, {
                start_time = os.time(),
                steps = {}
            })
            return {continue_execution = true}
        end)
        
        Hook.register("workflow:after_step", function(ctx)
            local perf = State.get("perf:" .. ctx.workflow_id)
            table.insert(perf.steps, {
                name = ctx.step.name,
                duration = ctx.step.duration_ms
            })
            State.set("perf:" .. ctx.workflow_id, perf)
            return {continue_execution = true}
        end)
        
        Hook.register("workflow:after_complete", function(ctx)
            local perf = State.get("perf:" .. ctx.workflow_id)
            perf.total_duration = os.time() - perf.start_time
            
            -- Emit performance event
            Event.emit("workflow:performance", perf)
            
            -- Clean up
            State.delete("perf:" .. ctx.workflow_id)
        end)
    end
}
```

### Pattern 2: Error Recovery

```lua
-- Intelligent error recovery
local ErrorRecovery = {
    init = function()
        Hook.register("workflow:on_step_error", function(ctx)
            local error_type = ctx.step.error:match("(%w+)Error")
            
            if error_type == "Network" then
                -- Wait and retry for network errors
                os.execute("sleep 2")
                return {
                    continue_execution = true,
                    retry_step = true,
                    message = "Retrying after network error"
                }
            elseif error_type == "RateLimit" then
                -- Skip to fallback step
                return {
                    continue_execution = true,
                    skip_to_step = ctx.step.name .. "_fallback"
                }
            else
                -- Unrecoverable error
                Event.emit("critical_error", {
                    workflow = ctx.workflow_id,
                    step = ctx.step.name,
                    error = ctx.step.error
                })
                return {continue_execution = false}
            end
        end)
    end
}
```

### Pattern 3: Conditional Execution

```lua
-- Feature flag based execution
local FeatureFlags = {
    init = function()
        Hook.register("workflow:before_step", function(ctx)
            local flags = Config.get("feature_flags") or {}
            
            -- Skip experimental steps if flag is off
            if ctx.step.metadata and ctx.step.metadata.experimental then
                if not flags.enable_experimental then
                    return {
                        continue_execution = true,
                        skip_to_step = ctx.step.next_step,
                        message = "Skipping experimental step"
                    }
                end
            end
            
            return {continue_execution = true}
        end)
    end
}
```

---

## Performance Considerations

### Hook Execution Order

1. Hooks are executed in registration order
2. Each hook must complete before the next runs
3. Async hooks are awaited

### Optimization Strategies

1. **Cache Hook Results**: For expensive computations
2. **Batch Updates**: Aggregate state updates
3. **Async Operations**: Use Event.emit for non-critical operations
4. **Early Exit**: Return quickly for non-matching conditions

```lua
-- Optimized hook example
Hook.register("workflow:after_step", function(ctx)
    -- Early exit for non-monitored steps
    if not ctx.step.metadata or not ctx.step.metadata.monitor then
        return {continue_execution = true}
    end
    
    -- Batch metrics update
    local batch_key = "metrics_batch:" .. ctx.workflow_id
    local batch = State.get(batch_key) or {}
    table.insert(batch, {
        step = ctx.step.name,
        duration = ctx.step.duration_ms
    })
    
    -- Flush every 10 steps
    if #batch >= 10 then
        Event.emit("metrics:batch", batch)
        State.delete(batch_key)
    else
        State.set(batch_key, batch)
    end
    
    return {continue_execution = true}
end)
```

---

## Migration Guide

### Preparing for Phase 4

1. **Structure Your Workflows**
   ```lua
   -- Add metadata to workflows
   local workflow = Workflow.sequential({
       name = "my_workflow",
       metadata = {
           version = "1.0",
           author = "team",
           monitor = true
       },
       steps = [...]
   })
   ```

2. **Use State for Temporary Hooks**
   ```lua
   -- Current workaround
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

3. **Plan Hook Points**
   ```lua
   -- Document where you'll need hooks
   local workflow_config = {
       name = "data_pipeline",
       hooks_needed = {
           "before_start",  -- Initialize resources
           "after_step",    -- Collect metrics
           "on_error",      -- Error recovery
           "after_complete" -- Cleanup
       }
   }
   ```

---

## Security Considerations

1. **Hook Sandboxing**: Hooks run in restricted environment
2. **Resource Limits**: CPU and memory limits enforced
3. **Access Control**: Hooks can only access allowed globals
4. **Timeout Protection**: Hooks timeout after 5 seconds

---

## Future Enhancements

### Phase 5 Integration
- Persistent hook state
- Cross-workflow hook sharing
- Hook versioning

### Phase 6 Integration
- Session-aware hooks
- Hook replay for debugging
- Visual hook monitoring

---

## Examples

Complete examples will be available in Phase 4:
- `workflow_monitoring.lua` - Performance monitoring setup
- `error_recovery.lua` - Advanced error handling
- `conditional_execution.lua` - Dynamic workflow control
- `hook_composition.lua` - Combining multiple hook patterns