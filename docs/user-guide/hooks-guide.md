# Hooks Guide

## Introduction

Hooks are synchronous interception points throughout rs-llmspell that allow you to observe, modify, or control the behavior of agents, tools, and workflows. With over 40 hook points and automatic performance protection, hooks provide production-ready extensibility.

## Core Concepts

### What is a Hook?

A hook is a function that executes at specific points in the system lifecycle. Hooks can:
- Inspect data flowing through the system
- Modify inputs or outputs
- Cancel operations
- Redirect execution flow
- Cache results
- Add retry logic
- Track metrics and costs

### Hook Lifecycle

```
1. Registration → 2. Execution → 3. Result Processing → 4. Cleanup
      ↓                ↓                    ↓                ↓
  Priority Sort    CircuitBreaker      Handle Result    Unregister
```

## All Hook Points (40+)

### Agent Lifecycle Hooks (7)

| Hook Point | Description | Common Use Cases |
|------------|-------------|------------------|
| `BeforeAgentInit` | Before agent initialization | Resource allocation, configuration validation |
| `AfterAgentInit` | After agent initialized | Logging, metric collection |
| `BeforeAgentExecution` | Before agent processes input | Input validation, rate limiting, caching |
| `AfterAgentExecution` | After agent produces output | Output filtering, cost tracking |
| `AgentError` | When agent encounters error | Error recovery, alerting |
| `BeforeAgentShutdown` | Before agent cleanup | Save state, flush caches |
| `AfterAgentShutdown` | After agent cleaned up | Resource verification |

### Tool Execution Hooks (6)

| Hook Point | Description | Common Use Cases |
|------------|-------------|------------------|
| `BeforeToolDiscovery` | Before tool lookup | Custom tool injection |
| `AfterToolDiscovery` | After tool found | Tool verification |
| `BeforeToolExecution` | Before tool runs | Parameter validation, security checks |
| `AfterToolExecution` | After tool completes | Result transformation, logging |
| `ToolValidation` | During input validation | Custom validation rules |
| `ToolError` | When tool fails | Error handling, retry logic |

### Workflow Hooks (8)

| Hook Point | Description | Common Use Cases |
|------------|-------------|------------------|
| `BeforeWorkflowStart` | Before workflow begins | Setup, resource checks |
| `WorkflowStageTransition` | Between workflow stages | Progress tracking |
| `BeforeWorkflowStage` | Before each stage | Stage validation |
| `AfterWorkflowStage` | After stage completes | Checkpoint creation |
| `WorkflowCheckpoint` | At checkpoint creation | State persistence |
| `WorkflowRollback` | During rollback | Cleanup actions |
| `AfterWorkflowComplete` | After workflow ends | Final reporting |
| `WorkflowError` | On workflow error | Error recovery |

### State Management Hooks (6)

| Hook Point | Description | Common Use Cases |
|------------|-------------|------------------|
| `BeforeStateRead` | Before reading state | Access control |
| `AfterStateRead` | After reading state | Audit logging |
| `BeforeStateWrite` | Before writing state | Validation, conflict detection |
| `AfterStateWrite` | After writing state | Replication, notifications |
| `StateConflict` | On concurrent modification | Conflict resolution |
| `StateMigration` | During version migration | Data transformation |

### System Hooks (5)

| Hook Point | Description | Common Use Cases |
|------------|-------------|------------------|
| `SystemStartup` | At system initialization | Global setup |
| `SystemShutdown` | Before system exit | Cleanup, persistence |
| `ConfigurationChange` | When config updates | Hot reloading |
| `ResourceLimitExceeded` | When limits hit | Throttling, alerting |
| `SecurityViolation` | On security issues | Audit, blocking |

### Additional Hooks (8+)

| Category | Hook Points | Use Cases |
|----------|-------------|-----------|
| **Session** | `SessionStart`, `SessionEnd`, `SessionCheckpoint`, `SessionRestore` | User session management |
| **Events** | `BeforeEventEmit`, `AfterEventEmit`, `EventHandlerError` | Event system integration |
| **Performance** | `PerformanceThresholdExceeded`, `MemoryUsageHigh`, `CpuUsageHigh` | Performance monitoring |
| **Custom** | `Custom(String)` | Application-specific hooks |

## Hook Results (9 Types)

Hooks return a `HookResult` that determines what happens next:

| Result | Description | Example Use Case |
|--------|-------------|------------------|
| `Continue` | Proceed normally | Simple logging |
| `Modified { data }` | Use modified data | Input sanitization |
| `Cancel { reason }` | Stop execution | Security blocking |
| `Redirect { target }` | Go elsewhere | A/B testing |
| `Replace { component }` | Use different component | Feature flags |
| `Retry { config }` | Try again | Transient errors |
| `Fork { branches }` | Multiple paths | Parallel testing |
| `Cache { ttl }` | Use cached result | Performance optimization |
| `Skipped { reason }` | Skip this hook | Conditional logic |

### Result Examples

```lua
-- Continue: Most common result
return "continue"

-- Modified: Change the input
return {
    action = "modified",
    modified_data = {
        input = {text = sanitized_text}
    }
}

-- Cancel: Stop execution
return {
    action = "cancel",
    reason = "Input contains prohibited content"
}

-- Retry: Try again with backoff
return {
    action = "retry",
    max_attempts = 3,
    backoff_ms = 1000
}
```

## Priority System

Hooks execute in priority order:

| Priority | Value | Use Case |
|----------|-------|----------|
| `highest` | -1000 | Critical security checks |
| `high` | -100 | Important validations |
| `normal` | 0 | Standard processing |
| `low` | 100 | Optional enhancements |
| `lowest` | 1000 | Logging, metrics |

```lua
-- Register with high priority
Hook.register("BeforeAgentExecution", myHook, "high")

-- Or use string priorities
Hook.register("BeforeToolExecution", myHook, "highest")
```

## CircuitBreaker Protection

Every hook is automatically protected by a CircuitBreaker that ensures system performance:

### How it Works

1. **Monitoring**: Each hook execution is timed
2. **Threshold**: If execution exceeds 100ms (configurable)
3. **Counting**: After 5 consecutive slow executions
4. **Opening**: CircuitBreaker opens, disabling the hook
5. **Recovery**: After 30 seconds, it tries again

### Performance Guarantees

- **<5% overhead**: Total hook system overhead
- **Automatic protection**: Slow hooks can't break the system
- **Graceful degradation**: System continues without slow hooks
- **Self-healing**: Hooks re-enable when performance improves

### Monitoring CircuitBreaker Status

```lua
-- This hook will be automatically disabled if too slow
Hook.register("BeforeToolExecution", function(context)
    -- Check if we're being monitored
    if context.performance_warning then
        print("Warning: Hook is slow, may be disabled soon")
    end
    
    -- Your hook logic here
    return "continue"
end)
```

## Hook Registration

### Basic Registration

```lua
-- Simple function hook
local function myHook(context)
    print("Hook called for:", context.component_id.name)
    return "continue"
end

local handle = Hook.register("BeforeAgentExecution", myHook)
```

### With Priority

```lua
-- High priority security check
local handle = Hook.register("BeforeToolExecution", function(context)
    if not validate_security(context) then
        return {
            action = "cancel",
            reason = "Security validation failed"
        }
    end
    return "continue"
end, "high")
```

### With Metadata

```lua
-- Named hook with tags
local handle = Hook.register({
    point = "BeforeAgentExecution",
    handler = myHook,
    priority = "normal",
    name = "my-custom-hook",
    tags = {"monitoring", "production"}
})
```

## Hook Context

Every hook receives a context object with:

```lua
context = {
    -- Component information
    component_id = {
        id = "uuid",
        name = "agent-name",
        component_type = "agent"
    },
    
    -- Correlation for tracing
    correlation_id = "request-uuid",
    
    -- Hook-specific data
    data = {
        -- Varies by hook point
        input = {...},
        output = {...},
        error = {...}
    },
    
    -- Performance monitoring
    performance_warning = false,
    
    -- Cross-language support
    language = "lua"
}
```

## Practical Examples

### 1. Input Validation Hook

```lua
Hook.register("BeforeAgentExecution", function(context)
    local input = context.data.input
    
    -- Validate input length
    if #input.text > 10000 then
        return {
            action = "cancel",
            reason = "Input too long (max 10000 characters)"
        }
    end
    
    -- Check for prohibited content
    if input.text:match("password") then
        return {
            action = "modified",
            modified_data = {
                input = {
                    text = input.text:gsub("password", "[REDACTED]")
                }
            }
        }
    end
    
    return "continue"
end, "high")
```

### 2. Caching Hook

```lua
local cache = {}

Hook.register("BeforeToolExecution", function(context)
    local tool_name = context.data.tool_name
    local params = context.data.parameters
    local cache_key = tool_name .. ":" .. serialize(params)
    
    -- Check cache
    local cached = cache[cache_key]
    if cached and os.time() - cached.time < 300 then  -- 5 minute TTL
        return {
            action = "cache",
            ttl = 300,
            result = cached.result
        }
    end
    
    return "continue"
end, "normal")

-- Cache results after execution
Hook.register("AfterToolExecution", function(context)
    local tool_name = context.data.tool_name
    local params = context.data.parameters
    local result = context.data.result
    local cache_key = tool_name .. ":" .. serialize(params)
    
    cache[cache_key] = {
        time = os.time(),
        result = result
    }
    
    return "continue"
end, "normal")
```

### 3. Cost Tracking Hook

```lua
local total_cost = 0
local cost_by_agent = {}

Hook.register("AfterAgentExecution", function(context)
    local agent_name = context.component_id.name
    local tokens = context.data.tokens_used or 0
    local model = context.data.model or "unknown"
    
    -- Calculate cost (example rates)
    local cost_per_1k = {
        ["gpt-4"] = 0.03,
        ["gpt-3.5-turbo"] = 0.002,
        ["claude-3-opus"] = 0.015
    }
    
    local rate = cost_per_1k[model] or 0.01
    local cost = (tokens / 1000) * rate
    
    -- Track costs
    total_cost = total_cost + cost
    cost_by_agent[agent_name] = (cost_by_agent[agent_name] or 0) + cost
    
    -- Alert if over budget
    if total_cost > 10.0 then
        Event.publish("cost.budget.exceeded", {
            total_cost = total_cost,
            threshold = 10.0
        })
    end
    
    return "continue"
end, "low")
```

### 4. Retry Hook for Transient Errors

```lua
Hook.register("ToolError", function(context)
    local error = context.data.error
    local attempt = context.data.attempt or 1
    
    -- Check if error is retryable
    local retryable_errors = {
        "timeout",
        "rate_limit",
        "temporary_failure"
    }
    
    for _, pattern in ipairs(retryable_errors) do
        if error.message:match(pattern) then
            return {
                action = "retry",
                max_attempts = 3,
                backoff_ms = 1000 * attempt,  -- Exponential backoff
                jitter = true
            }
        end
    end
    
    return "continue"
end, "high")
```

## Hook Management

### Listing Hooks

```lua
-- List all hooks
local all_hooks = Hook.list()
for _, hook in ipairs(all_hooks) do
    print(hook.name, hook.point, hook.priority)
end

-- Filter by hook point
local agent_hooks = Hook.list({
    point = "BeforeAgentExecution"
})

-- Filter by priority
local high_priority = Hook.list({
    priority = "high"
})

-- Multiple filters
local specific = Hook.list({
    point = "BeforeToolExecution",
    priority = "highest",
    tag = "security"
})
```

### Unregistering Hooks

```lua
-- Method 1: Using handle
local handle = Hook.register("BeforeAgentExecution", myHook)
handle:unregister()

-- Method 2: Using Hook.unregister
Hook.unregister(handle)

-- Unregister by criteria
Hook.unregister({
    name = "my-custom-hook"
})
```

## Performance Best Practices

### 1. Keep Hooks Fast

```lua
-- ❌ Bad: Slow synchronous operation
Hook.register("BeforeAgentExecution", function(context)
    local result = fetch_from_slow_api()  -- Blocks execution
    return "continue"
end)

-- ✅ Good: Quick check or async operation
Hook.register("BeforeAgentExecution", function(context)
    -- Quick validation
    if not context.data.input.api_key then
        return {action = "cancel", reason = "Missing API key"}
    end
    
    -- Queue async work instead of blocking
    Event.publish("validation.needed", {
        component = context.component_id.name
    })
    
    return "continue"
end)
```

### 2. Use Appropriate Priority

```lua
-- ✅ Correct priority usage
Hook.register("BeforeToolExecution", securityCheck, "highest")  -- Security first
Hook.register("BeforeToolExecution", validation, "high")        -- Then validate
Hook.register("BeforeToolExecution", logging, "lowest")         -- Log last
```

### 3. Handle Errors Gracefully

```lua
Hook.register("BeforeAgentExecution", function(context)
    -- Wrap in pcall to prevent hook errors from breaking system
    local ok, result = pcall(function()
        -- Your hook logic
        return process_hook(context)
    end)
    
    if not ok then
        -- Log error but continue
        print("Hook error:", result)
        return "continue"
    end
    
    return result
end)
```

### 4. Monitor Performance

```lua
-- Self-monitoring hook
Hook.register("BeforeAgentExecution", function(context)
    local start = os.clock()
    
    -- Your logic here
    
    local duration = (os.clock() - start) * 1000
    if duration > 50 then  -- Warning at 50ms
        print(string.format("Hook slow: %.2fms", duration))
    end
    
    return "continue"
end)
```

## Advanced Patterns

### Composite Hooks

See [Hook Patterns](./hook-patterns.md) for details on:
- Sequential hooks with early termination
- Parallel hooks with result aggregation
- First-match patterns
- Voting mechanisms

### Cross-Component Coordination

Hooks can coordinate across components:

```lua
-- Coordinate between agent and tool hooks
local agent_context = {}

Hook.register("BeforeAgentExecution", function(context)
    -- Store context for tools
    agent_context[context.correlation_id] = {
        agent = context.component_id.name,
        start_time = os.time()
    }
    return "continue"
end)

Hook.register("BeforeToolExecution", function(context)
    -- Access agent context
    local agent_info = agent_context[context.correlation_id]
    if agent_info then
        print(string.format("Tool %s called by agent %s",
            context.data.tool_name,
            agent_info.agent
        ))
    end
    return "continue"
end)
```

## Troubleshooting

### Hook Not Executing

1. **Check registration**: Ensure hook is properly registered
2. **Verify hook point**: Confirm the hook point exists and is triggered
3. **Check CircuitBreaker**: Hook may be disabled for poor performance
4. **Priority conflicts**: Higher priority hooks may be canceling execution

### Performance Issues

1. **Monitor execution time**: Use os.clock() to measure
2. **Check CircuitBreaker logs**: Look for disabled hooks
3. **Reduce work in hooks**: Move heavy operations to events
4. **Use caching**: Cache expensive computations

### Debugging Hooks

```lua
-- Debug wrapper
local function debug_hook(name, handler)
    return function(context)
        print(string.format("[HOOK] %s called with:", name))
        print("  Component:", context.component_id.name)
        print("  Correlation:", context.correlation_id)
        
        local start = os.clock()
        local result = handler(context)
        local duration = (os.clock() - start) * 1000
        
        print(string.format("[HOOK] %s completed in %.2fms", name, duration))
        return result
    end
end

-- Use debug wrapper
Hook.register("BeforeAgentExecution", 
    debug_hook("agent-validator", myValidationFunction)
)
```

## Next Steps

- **[Events Guide](./events-guide.md)**: Learn about the asynchronous event system
- **[Built-in Hooks Reference](./builtin-hooks-reference.md)**: Use production-ready hooks
- **[Hook Patterns](./hook-patterns.md)**: Advanced hook composition patterns
- **[Hook Development Guide](../developer-guide/hook-development-guide.md)**: Create custom hooks in Rust
- **[Examples](./examples/hooks-events-cookbook.md)**: 23 working examples

## Summary

- Hooks provide synchronous interception with 40+ integration points
- 9 different result types enable complex control flow
- Priority system ensures proper execution order
- CircuitBreaker guarantees <5% performance overhead
- Cross-language support enables universal extensibility
- Built-in hooks provide production-ready functionality