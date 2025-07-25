# Hook Patterns and Recipes

## Introduction

This guide presents common hook patterns and recipes for building robust extensibility into your rs-llmspell applications. These patterns help you compose hooks for complex scenarios while maintaining performance and reliability.

## Basic Patterns

### 1. Simple Logging Hook

The most basic pattern - observe without modifying:

```lua
Hook.register("BeforeAgentExecution", function(context)
    print(string.format("[%s] Agent %s executing", 
        os.date("%Y-%m-%d %H:%M:%S"),
        context.component_id.name
    ))
    return "continue"
end, "lowest")
```

### 2. Conditional Hook

Execute only under certain conditions:

```lua
Hook.register("BeforeToolExecution", function(context)
    -- Only validate filesystem tools
    if not context.data.tool_name:match("^file") then
        return "continue"
    end
    
    -- Validate path parameter
    local path = context.data.parameters.path
    if path and path:match("\.\.") then
        return {
            action = "cancel",
            reason = "Path traversal detected"
        }
    end
    
    return "continue"
end, "high")
```

### 3. Data Enrichment Hook

Add metadata or enrich data:

```lua
Hook.register("BeforeAgentExecution", function(context)
    -- Add request metadata
    local enriched_input = {
        text = context.data.input.text,
        metadata = {
            timestamp = os.time(),
            user_id = context.metadata.user_id or "anonymous",
            session_id = context.correlation_id,
            environment = os.getenv("ENVIRONMENT") or "development"
        }
    }
    
    return {
        action = "modified",
        modified_data = {
            input = enriched_input
        }
    }
end, "normal")
```

## Composite Hook Patterns

### 4. Sequential Hooks with Early Exit

Run multiple checks in sequence, stopping on first failure:

```lua
local function createSequentialHook(checks)
    return function(context)
        for i, check in ipairs(checks) do
            local result = check.fn(context)
            if result ~= "continue" then
                print(string.format("Sequential hook stopped at check %d: %s", 
                    i, check.name))
                return result
            end
        end
        return "continue"
    end
end

-- Usage
local securityChecks = {
    {name = "auth", fn = checkAuthentication},
    {name = "rate_limit", fn = checkRateLimit},
    {name = "input_validation", fn = validateInput},
    {name = "permissions", fn = checkPermissions}
}

Hook.register("BeforeAgentExecution", 
    createSequentialHook(securityChecks), 
    "highest"
)
```

### 5. Parallel Hooks with Aggregation

Run multiple hooks and aggregate results:

```lua
local function createParallelHook(hooks)
    return function(context)
        local results = {}
        local modifications = {}
        
        -- Execute all hooks
        for _, hook in ipairs(hooks) do
            local result = hook(context)
            table.insert(results, result)
            
            -- Collect modifications
            if type(result) == "table" and result.action == "modified" then
                for k, v in pairs(result.modified_data) do
                    modifications[k] = v
                end
            end
        end
        
        -- Check for any cancellations
        for _, result in ipairs(results) do
            if type(result) == "table" and result.action == "cancel" then
                return result
            end
        end
        
        -- Apply all modifications
        if next(modifications) then
            return {
                action = "modified",
                modified_data = modifications
            }
        end
        
        return "continue"
    end
end
```

### 6. First-Match Pattern

Try multiple strategies until one succeeds:

```lua
local function createFirstMatchHook(strategies)
    return function(context)
        for _, strategy in ipairs(strategies) do
            local result = strategy.check(context)
            if result then
                print("Using strategy: " .. strategy.name)
                return strategy.handle(context, result)
            end
        end
        return "continue"
    end
end

-- Usage: Try multiple caching strategies
local cacheStrategies = {
    {
        name = "memory_cache",
        check = function(ctx) return MemoryCache.get(ctx.key) end,
        handle = function(ctx, cached) 
            return {action = "cache", result = cached}
        end
    },
    {
        name = "disk_cache",
        check = function(ctx) return DiskCache.get(ctx.key) end,
        handle = function(ctx, cached)
            -- Promote to memory cache
            MemoryCache.set(ctx.key, cached)
            return {action = "cache", result = cached}
        end
    },
    {
        name = "remote_cache",
        check = function(ctx) return RemoteCache.get(ctx.key) end,
        handle = function(ctx, cached)
            -- Promote to local caches
            DiskCache.set(ctx.key, cached)
            MemoryCache.set(ctx.key, cached)
            return {action = "cache", result = cached}
        end
    }
}
```

### 7. Voting/Consensus Pattern

Multiple hooks vote on the outcome:

```lua
local function createVotingHook(voters, threshold)
    return function(context)
        local votes = {
            continue = 0,
            cancel = 0,
            modify = 0
        }
        local modifications = {}
        local cancel_reasons = {}
        
        -- Collect votes
        for _, voter in ipairs(voters) do
            local result = voter.vote(context)
            
            if result == "continue" then
                votes.continue = votes.continue + 1
            elseif type(result) == "table" then
                if result.action == "cancel" then
                    votes.cancel = votes.cancel + 1
                    table.insert(cancel_reasons, result.reason)
                elseif result.action == "modified" then
                    votes.modify = votes.modify + 1
                    for k, v in pairs(result.modified_data) do
                        modifications[k] = v
                    end
                end
            end
        end
        
        -- Determine outcome
        local total_votes = #voters
        
        if votes.cancel / total_votes >= threshold then
            return {
                action = "cancel",
                reason = "Consensus: " .. table.concat(cancel_reasons, "; ")
            }
        elseif votes.modify / total_votes >= threshold then
            return {
                action = "modified",
                modified_data = modifications
            }
        else
            return "continue"
        end
    end
end
```

## Cross-Component Coordination

### 8. Agent-Tool Coordination

Coordinate between agent and tool hooks:

```lua
-- Store agent context for tools
local agent_contexts = {}

Hook.register("BeforeAgentExecution", function(context)
    -- Store agent context
    agent_contexts[context.correlation_id] = {
        agent_name = context.component_id.name,
        start_time = os.time(),
        input_tokens = countTokens(context.data.input.text)
    }
    return "continue"
end, "high")

Hook.register("BeforeToolExecution", function(context)
    -- Access agent context
    local agent_ctx = agent_contexts[context.correlation_id]
    if agent_ctx then
        -- Apply agent-specific tool policies
        if agent_ctx.agent_name == "research-agent" and 
           context.data.tool_name == "web_search" then
            -- Limit search results for research agent
            return {
                action = "modified",
                modified_data = {
                    parameters = {
                        max_results = 5,
                        timeout = 10
                    }
                }
            }
        end
    end
    return "continue"
end, "normal")

Hook.register("AfterAgentExecution", function(context)
    -- Cleanup
    agent_contexts[context.correlation_id] = nil
    return "continue"
end, "lowest")
```

### 9. Workflow Stage Coordination

Track and coordinate workflow stages:

```lua
local workflow_state = {}

Hook.register("BeforeWorkflowStart", function(context)
    workflow_state[context.workflow_id] = {
        stages = {},
        start_time = os.time(),
        total_cost = 0
    }
    return "continue"
end)

Hook.register("BeforeWorkflowStage", function(context)
    local state = workflow_state[context.workflow_id]
    if state then
        -- Check dependencies
        local deps = context.data.stage.dependencies or {}
        for _, dep in ipairs(deps) do
            if not state.stages[dep] or 
               not state.stages[dep].completed then
                return {
                    action = "cancel",
                    reason = "Dependency not met: " .. dep
                }
            end
        end
        
        -- Initialize stage tracking
        state.stages[context.data.stage.name] = {
            start_time = os.time(),
            completed = false
        }
    end
    return "continue"
end)

Hook.register("AfterWorkflowStage", function(context)
    local state = workflow_state[context.workflow_id]
    if state and state.stages[context.data.stage.name] then
        state.stages[context.data.stage.name].completed = true
        state.stages[context.data.stage.name].duration = 
            os.time() - state.stages[context.data.stage.name].start_time
    end
    return "continue"
end)
```

## Error Handling Patterns

### 10. Retry with Exponential Backoff

```lua
local retry_state = {}

Hook.register("ToolError", function(context)
    local tool_id = context.component_id.id
    local error_msg = context.data.error
    
    -- Initialize retry state
    retry_state[tool_id] = retry_state[tool_id] or {
        attempts = 0,
        last_attempt = 0
    }
    
    local state = retry_state[tool_id]
    
    -- Check if error is retryable
    local retryable_patterns = {
        "timeout",
        "rate_limit",
        "temporary_failure",
        "connection_refused"
    }
    
    local is_retryable = false
    for _, pattern in ipairs(retryable_patterns) do
        if error_msg:match(pattern) then
            is_retryable = true
            break
        end
    end
    
    if is_retryable and state.attempts < 3 then
        state.attempts = state.attempts + 1
        local backoff = math.pow(2, state.attempts) * 1000 -- Exponential
        local jitter = math.random(0, 500) -- Add jitter
        
        return {
            action = "retry",
            max_attempts = 1,
            backoff_ms = backoff + jitter
        }
    else
        -- Cleanup retry state
        retry_state[tool_id] = nil
        return "continue"
    end
end, "high")
```

### 11. Circuit Breaker Pattern

```lua
local circuit_breakers = {}

local function getCircuitBreaker(component_id)
    if not circuit_breakers[component_id] then
        circuit_breakers[component_id] = {
            failure_count = 0,
            last_failure_time = 0,
            state = "closed", -- closed, open, half-open
            success_count = 0
        }
    end
    return circuit_breakers[component_id]
end

Hook.register("BeforeAgentExecution", function(context)
    local cb = getCircuitBreaker(context.component_id.id)
    
    if cb.state == "open" then
        -- Check if we should try half-open
        if os.time() - cb.last_failure_time > 30 then
            cb.state = "half-open"
            cb.success_count = 0
        else
            return {
                action = "cancel",
                reason = "Circuit breaker is open"
            }
        end
    end
    
    return "continue"
end, "highest")

Hook.register("AgentError", function(context)
    local cb = getCircuitBreaker(context.component_id.id)
    
    cb.failure_count = cb.failure_count + 1
    cb.last_failure_time = os.time()
    
    if cb.failure_count >= 5 then
        cb.state = "open"
        Event.publish("circuit_breaker.opened", {
            component_id = context.component_id.id,
            failure_count = cb.failure_count
        })
    end
    
    return "continue"
end)

Hook.register("AfterAgentExecution", function(context)
    local cb = getCircuitBreaker(context.component_id.id)
    
    if cb.state == "half-open" then
        cb.success_count = cb.success_count + 1
        if cb.success_count >= 2 then
            cb.state = "closed"
            cb.failure_count = 0
            Event.publish("circuit_breaker.closed", {
                component_id = context.component_id.id
            })
        end
    elseif cb.state == "closed" then
        -- Reset failure count on success
        cb.failure_count = 0
    end
    
    return "continue"
end)
```

## Performance Monitoring Patterns

### 12. Performance Tracking with Alerts

```lua
local performance_metrics = {}

local function trackPerformance(component_type)
    return function(context)
        local key = component_type .. ":" .. context.component_id.name
        
        if not performance_metrics[key] then
            performance_metrics[key] = {
                count = 0,
                total_time = 0,
                min_time = math.huge,
                max_time = 0,
                errors = 0
            }
        end
        
        local metrics = performance_metrics[key]
        
        -- Store start time
        context.state._perf_start = os.clock()
        
        return "continue"
    end
end

local function recordPerformance(component_type)
    return function(context)
        local key = component_type .. ":" .. context.component_id.name
        local metrics = performance_metrics[key]
        
        if metrics and context.state._perf_start then
            local duration = (os.clock() - context.state._perf_start) * 1000
            
            metrics.count = metrics.count + 1
            metrics.total_time = metrics.total_time + duration
            metrics.min_time = math.min(metrics.min_time, duration)
            metrics.max_time = math.max(metrics.max_time, duration)
            
            -- Alert on slow operations
            if duration > 1000 then
                Event.publish("performance.slow_operation", {
                    component_type = component_type,
                    component_name = context.component_id.name,
                    duration_ms = duration
                })
            end
            
            -- Periodic reporting
            if metrics.count % 100 == 0 then
                Event.publish("performance.report", {
                    component = key,
                    average_ms = metrics.total_time / metrics.count,
                    min_ms = metrics.min_time,
                    max_ms = metrics.max_time,
                    count = metrics.count,
                    error_rate = metrics.errors / metrics.count
                })
            end
        end
        
        return "continue"
    end
end

-- Register for agents
Hook.register("BeforeAgentExecution", trackPerformance("agent"), "low")
Hook.register("AfterAgentExecution", recordPerformance("agent"), "low")

-- Register for tools
Hook.register("BeforeToolExecution", trackPerformance("tool"), "low")
Hook.register("AfterToolExecution", recordPerformance("tool"), "low")
```

### 13. Resource Usage Monitoring

```lua
local resource_limits = {
    max_memory_mb = 512,
    max_concurrent = 10,
    max_queue_size = 1000
}

local current_usage = {
    concurrent_ops = 0,
    queue_size = 0
}

Hook.register("BeforeAgentExecution", function(context)
    -- Check concurrent operations
    if current_usage.concurrent_ops >= resource_limits.max_concurrent then
        return {
            action = "cancel",
            reason = "Maximum concurrent operations reached"
        }
    end
    
    current_usage.concurrent_ops = current_usage.concurrent_ops + 1
    
    -- Track in context for cleanup
    context.state._resource_tracked = true
    
    return "continue"
end, "high")

Hook.register("AfterAgentExecution", function(context)
    if context.state._resource_tracked then
        current_usage.concurrent_ops = current_usage.concurrent_ops - 1
    end
    return "continue"
end, "lowest")

Hook.register("AgentError", function(context)
    -- Ensure cleanup on error
    if context.state._resource_tracked then
        current_usage.concurrent_ops = current_usage.concurrent_ops - 1
        context.state._resource_tracked = nil
    end
    return "continue"
end, "lowest")
```

## Integration with Events

### 14. Hook-Event Coordination

Hooks that publish events for monitoring:

```lua
-- Publish events from hooks
Hook.register("BeforeAgentExecution", function(context)
    Event.publish("agent.execution.started", {
        agent_name = context.component_id.name,
        correlation_id = context.correlation_id,
        input_length = #context.data.input.text
    })
    return "continue"
end, "low")

-- Subscribe to events and update hook behavior
local high_load = false
local load_monitor = Event.subscribe("system.load.high")

-- Background task to monitor load
Task.spawn(function()
    while true do
        local event = Event.receive(load_monitor, 1000)
        if event then
            high_load = true
            -- Reduce load for 60 seconds
            Task.sleep(60)
            high_load = false
        end
    end
end)

-- Hook that adjusts behavior based on load
Hook.register("BeforeAgentExecution", function(context)
    if high_load then
        -- Use faster model during high load
        return {
            action = "modified",
            modified_data = {
                model = "gpt-3.5-turbo" -- Faster model
            }
        }
    end
    return "continue"
end, "normal")
```

### 15. Distributed Tracing Pattern

```lua
-- Trace context propagation
Hook.register("BeforeAgentExecution", function(context)
    local trace_id = context.metadata.trace_id or generateTraceId()
    local span_id = generateSpanId()
    
    -- Store trace context
    context.state._trace = {
        trace_id = trace_id,
        span_id = span_id,
        parent_span_id = context.metadata.parent_span_id
    }
    
    -- Publish span start event
    Event.publish("trace.span.start", {
        trace_id = trace_id,
        span_id = span_id,
        parent_span_id = context.metadata.parent_span_id,
        operation = "agent.execution",
        component = context.component_id.name,
        timestamp = os.time()
    })
    
    return "continue"
end, "high")

Hook.register("AfterAgentExecution", function(context)
    if context.state._trace then
        Event.publish("trace.span.end", {
            trace_id = context.state._trace.trace_id,
            span_id = context.state._trace.span_id,
            duration_ms = context.data.duration_ms,
            status = "success",
            timestamp = os.time()
        })
    end
    return "continue"
end, "low")
```

## Best Practices

### Hook Composition Guidelines

1. **Keep Individual Hooks Simple**: Each hook should have a single responsibility
2. **Use Appropriate Priorities**: Security highest, monitoring lowest
3. **Handle Errors Gracefully**: Always use pcall for external calls
4. **Clean Up Resources**: Use finally patterns or cleanup hooks
5. **Avoid State Pollution**: Use context.state for hook-specific data
6. **Document Side Effects**: Clear comments on what hooks modify
7. **Test Compositions**: Verify hook interactions

### Performance Considerations

1. **Minimize Hook Overhead**: Target <5ms per hook
2. **Use Events for Heavy Work**: Don't block in hooks
3. **Cache Computed Values**: Avoid redundant calculations
4. **Batch Operations**: Aggregate data before processing
5. **Monitor Hook Performance**: Use CircuitBreaker protection

### Security Patterns

1. **Defense in Depth**: Layer security hooks
2. **Fail Secure**: Default to denying access
3. **Audit Everything**: Log security decisions
4. **Validate Early**: Check inputs at highest priority
5. **Sanitize Outputs**: Clean data before returning

## Real-World Examples

### Multi-Tenant SaaS Application

```lua
-- Tenant isolation and rate limiting
local tenant_limits = {
    free = {requests_per_minute = 10, max_tokens = 1000},
    pro = {requests_per_minute = 60, max_tokens = 10000},
    enterprise = {requests_per_minute = 600, max_tokens = 100000}
}

local tenant_usage = {}

Hook.register("BeforeAgentExecution", function(context)
    local tenant_id = context.metadata.tenant_id
    local tenant_tier = context.metadata.tenant_tier or "free"
    
    if not tenant_id then
        return {action = "cancel", reason = "No tenant ID"}
    end
    
    -- Initialize tenant tracking
    tenant_usage[tenant_id] = tenant_usage[tenant_id] or {
        requests = {},
        tokens_used = 0
    }
    
    local usage = tenant_usage[tenant_id]
    local limits = tenant_limits[tenant_tier]
    
    -- Rate limiting
    local now = os.time()
    local recent_requests = 0
    for i = #usage.requests, 1, -1 do
        if now - usage.requests[i] < 60 then
            recent_requests = recent_requests + 1
        else
            table.remove(usage.requests, i)
        end
    end
    
    if recent_requests >= limits.requests_per_minute then
        return {
            action = "cancel",
            reason = "Rate limit exceeded"
        }
    end
    
    table.insert(usage.requests, now)
    
    -- Token limits
    local input_tokens = countTokens(context.data.input.text)
    if usage.tokens_used + input_tokens > limits.max_tokens then
        return {
            action = "cancel",
            reason = "Token limit exceeded"
        }
    end
    
    return "continue"
end, "highest")
```

## Next Steps

- **[Cross-Language Integration](./cross-language-integration.md)**: Hook/event cross-language patterns
- **[Built-in Hooks Reference](./builtin-hooks-reference.md)**: Production-ready hooks
- **[Hook Development Guide](../developer-guide/hook-development-guide.md)**: Create custom hooks
- **[Examples Cookbook](./examples/hooks-events-cookbook.md)**: Complete working examples

## Summary

These patterns provide building blocks for creating sophisticated hook-based extensions:
- **Composite patterns** for complex logic
- **Coordination patterns** for multi-component systems  
- **Error handling patterns** for resilience
- **Performance patterns** for monitoring
- **Integration patterns** for hook-event coordination

Combine these patterns to build production-ready extensibility into your rs-llmspell applications.
