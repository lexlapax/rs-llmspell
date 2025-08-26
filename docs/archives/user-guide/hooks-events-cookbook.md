# Hooks and Events Cookbook

## Introduction

This cookbook references the 23 working examples from the `/examples/lua/` directory, organized by use case. Each example is a complete, runnable script that demonstrates real-world applications of hooks and events in rs-llmspell.

## Hook Examples

### Basic Hook Usage

#### 1. Hook Registration and Lifecycle
**File**: `/examples/lua/hooks/hook-basic.lua`
```lua
-- Basic hook registration and unregistration
local handle = Hook.register("BeforeAgentExecution", function(context)
    print("Agent executing:", context.component_id.name)
    return "continue"
end)

-- Later, unregister
Hook.unregister(handle)
```
**Use Case**: Simple monitoring, debugging, basic interception

#### 2. Hook Priority System
**File**: `/examples/lua/hooks/hook-priorities.lua`
```lua
-- Demonstrates all 5 priority levels
Hook.register("BeforeToolExecution", securityCheck, "highest")
Hook.register("BeforeToolExecution", validation, "high")
Hook.register("BeforeToolExecution", monitoring, "normal")
Hook.register("BeforeToolExecution", logging, "low")
Hook.register("BeforeToolExecution", metrics, "lowest")
```
**Use Case**: Ensuring proper execution order for security, validation, monitoring

### Agent Integration

#### 3. Agent Lifecycle Hooks
**File**: `/examples/lua/hooks/hook-lifecycle.lua`
```lua
-- Monitor complete agent lifecycle
Hook.register("BeforeAgentInit", function(context)
    print("Agent initializing:", context.component_id.name)
    return "continue"
end)

Hook.register("AfterAgentExecution", function(context)
    print("Tokens used:", context.data.tokens_used)
    return "continue"
end)
```
**Use Case**: Resource allocation, cost tracking, performance monitoring

### Tool Integration

#### 4. Tool Execution Hooks
**File**: `/examples/lua/hooks/hook-tool-integration.lua`
```lua
-- Validate and monitor tool usage
Hook.register("BeforeToolExecution", function(context)
    local tool_name = context.data.tool_name
    
    -- Security check for sensitive tools
    if tool_name == "process_executor" then
        if not context.metadata.authorized then
            return {
                action = "cancel",
                reason = "Unauthorized process execution"
            }
        end
    end
    
    return "continue"
end, "highest")
```
**Use Case**: Tool security, parameter validation, usage auditing

### Workflow Integration

#### 5. Workflow Stage Hooks
**File**: `/examples/lua/hooks/hook-workflow-integration.lua`
```lua
-- Track workflow progress
local workflow_metrics = {}

Hook.register("BeforeWorkflowStage", function(context)
    workflow_metrics[context.workflow_id] = {
        current_stage = context.data.stage.name,
        start_time = os.time()
    }
    return "continue"
end)

Hook.register("AfterWorkflowStage", function(context)
    local metrics = workflow_metrics[context.workflow_id]
    if metrics then
        local duration = os.time() - metrics.start_time
        print(string.format("Stage %s took %d seconds", 
            context.data.stage.name, duration))
    end
    return "continue"
end)
```
**Use Case**: Workflow monitoring, performance tracking, progress reporting

### Data Modification

#### 6. Hook Result Types
**File**: `/examples/lua/hooks/hook-data-modification.lua`
```lua
-- Demonstrate all hook result types

-- Modify input data
Hook.register("BeforeAgentExecution", function(context)
    return {
        action = "modified",
        modified_data = {
            input = {
                text = context.data.input.text .. " [enhanced]"
            }
        }
    }
end)

-- Cancel execution
Hook.register("BeforeToolExecution", function(context)
    if context.data.tool_name == "dangerous_tool" then
        return {
            action = "cancel",
            reason = "Tool not allowed"
        }
    end
    return "continue"
end)

-- Retry on failure
Hook.register("ToolError", function(context)
    if context.data.error:match("timeout") then
        return {
            action = "retry",
            max_attempts = 3,
            backoff_ms = 1000
        }
    end
    return "continue"
end)
```
**Use Case**: Input sanitization, output filtering, error recovery

### Error Handling

#### 7. Error Recovery Hooks
**File**: `/examples/lua/hooks/hook-error-handling.lua`
```lua
-- Graceful error handling with fallbacks
Hook.register("AgentError", function(context)
    local error_msg = context.data.error
    
    -- Rate limit handling
    if error_msg:match("rate_limit") then
        print("Rate limit hit, waiting...")
        return {
            action = "retry",
            max_attempts = 5,
            backoff_ms = 30000  -- 30 seconds
        }
    end
    
    -- Fallback to different model
    if error_msg:match("model_overloaded") then
        return {
            action = "redirect",
            target = "backup-agent"
        }
    end
    
    -- Log and continue
    Logger.error("Agent error", {error = error_msg})
    return "continue"
end, "high")
```
**Use Case**: Automatic retry, fallback strategies, error logging

### Cross-Language Integration

#### 8. Cross-Language Hook Coordination
**File**: `/examples/lua/hooks/hook-cross-language.lua`
```lua
-- Coordinate hooks across languages
Hook.register("BeforeAgentExecution", function(context)
    -- Check language-specific settings
    if context.language == "javascript" then
        -- Apply JS-specific modifications
        return {
            action = "modified",
            modified_data = {
                metadata = {
                    js_specific = true
                }
            }
        }
    end
    return "continue"
end)
```
**Use Case**: Language-specific behavior, cross-language coordination

### Advanced Patterns

#### 9. Hook Filtering and Management
**File**: `/examples/lua/hooks/hook-filtering-listing.lua`
```lua
-- List and manage hooks dynamically
local function list_security_hooks()
    local hooks = Hook.list({
        priority = "highest",
        tag = "security"
    })
    
    for _, hook in ipairs(hooks) do
        print(string.format("Security hook: %s at %s", 
            hook.name, hook.hook_point))
    end
end

-- Conditional hook registration
if Config.get("enable_audit") then
    Hook.register("AfterToolExecution", auditHook, "low")
end
```
**Use Case**: Dynamic hook management, conditional features, debugging

#### 10. Complex Hook Patterns
**File**: `/examples/lua/hooks/hook-advanced-patterns.lua`
```lua
-- Circuit breaker pattern
local circuit_breaker = {
    failures = 0,
    threshold = 5,
    is_open = false
}

Hook.register("BeforeAgentExecution", function(context)
    if circuit_breaker.is_open then
        return {
            action = "cancel",
            reason = "Circuit breaker open"
        }
    end
    return "continue"
end, "highest")

Hook.register("AgentError", function(context)
    circuit_breaker.failures = circuit_breaker.failures + 1
    if circuit_breaker.failures >= circuit_breaker.threshold then
        circuit_breaker.is_open = true
        Event.publish("circuit_breaker.opened", {
            component = context.component_id.name
        })
    end
    return "continue"
end)
```
**Use Case**: Circuit breakers, complex state machines, advanced patterns

## Event Examples

### Basic Event Usage

#### 11. Event Publishing and Subscription
**File**: `/examples/lua/events/event-basic.lua`
```lua
-- Basic publish/subscribe
local sub = Event.subscribe("user.action")

-- Publish event
Event.publish("user.action", {
    action = "login",
    user_id = "user123",
    timestamp = os.time()
})

-- Receive event
local event = Event.receive(sub, 1000)
if event then
    print("User action:", event.data.action)
end

Event.unsubscribe(sub)
```
**Use Case**: Basic event-driven communication, activity tracking

#### 12. Pattern-Based Subscriptions
**File**: `/examples/lua/events/event-patterns.lua`
```lua
-- Subscribe to patterns
local error_sub = Event.subscribe("*.error")
local user_sub = Event.subscribe("user.*")
local all_sub = Event.subscribe("**")  -- Everything

-- Different event types
Event.publish("agent.error", {message = "Failed"})
Event.publish("tool.error", {message = "Timeout"})
Event.publish("user.login", {user_id = "123"})
Event.publish("user.logout", {user_id = "123"})

-- Pattern matching in action
local error_event = Event.receive(error_sub, 100)
-- Receives both agent.error and tool.error
```
**Use Case**: Flexible event routing, monitoring specific patterns

### Cross-Language Events

#### 13. Cross-Language Event Communication
**File**: `/examples/lua/events/event-cross-language.lua`
```lua
-- Publish event with language metadata
Event.publish("data.processed", {
    processor = "lua-script",
    records = 1000
}, {
    language = "lua",
    correlation_id = "batch-123"
})

-- Subscribe and filter by language
local sub = Event.subscribe("data.processed")
while true do
    local event = Event.receive(sub, 1000)
    if event then
        print(string.format("Data processed by %s (%s)",
            event.data.processor,
            event.source.language or "unknown"
        ))
    end
end
```
**Use Case**: Multi-language coordination, distributed processing

### Complex Data Structures

#### 14. Nested Event Data
**File**: `/examples/lua/events/event-data-structures.lua`
```lua
-- Complex nested data
Event.publish("analysis.complete", {
    results = {
        summary = {
            total_items = 1000,
            processed = 950,
            errors = 50
        },
        details = {
            by_category = {
                documents = 500,
                images = 300,
                videos = 150
            },
            processing_times = {10.5, 23.1, 15.7, 8.9}
        }
    },
    metadata = {
        version = "1.0",
        processor_id = "proc-001"
    }
})
```
**Use Case**: Rich data events, analysis results, complex reports

### Subscription Management

#### 15. Subscription Lifecycle
**File**: `/examples/lua/events/event-subscription-management.lua`
```lua
-- Multiple subscriptions with cleanup
local subscriptions = {}

-- Create subscriptions
function setup_monitoring()
    subscriptions.errors = Event.subscribe("*.error")
    subscriptions.performance = Event.subscribe("performance.*")
    subscriptions.security = Event.subscribe("security.*")
end

-- Process events
function process_events()
    for name, sub in pairs(subscriptions) do
        local event = Event.receive(sub, 0)  -- Non-blocking
        if event then
            handle_event(name, event)
        end
    end
end

-- Cleanup
function cleanup()
    for _, sub in pairs(subscriptions) do
        Event.unsubscribe(sub)
    end
    subscriptions = {}
end
```
**Use Case**: Resource management, proper cleanup, monitoring systems

### Performance Scenarios

#### 16. High-Throughput Events
**File**: `/examples/lua/events/event-performance.lua`
```lua
-- High-performance event processing
local processed = 0
local sub = Event.subscribe("metrics.*")

-- Batch publisher
Task.spawn(function()
    for i = 1, 10000 do
        Event.publish("metrics.cpu", {
            value = math.random() * 100,
            timestamp = os.time()
        })
        if i % 100 == 0 then
            Utils.sleep(0.001)  -- Prevent overwhelming
        end
    end
end)

-- Batch receiver
Task.spawn(function()
    while processed < 10000 do
        local events = Event.receive_batch(sub, {
            max_events = 100,
            timeout_ms = 100
        })
        processed = processed + #events
        -- Process batch
    end
    print("Processed", processed, "events")
end)
```
**Use Case**: High-volume data processing, metrics collection, streaming

### Error Handling

#### 17. Event Timeouts and Errors
**File**: `/examples/lua/events/event-timeout-handling.lua`
```lua
-- Timeout handling patterns
local sub = Event.subscribe("task.result")

-- With timeout fallback
local function wait_for_result(timeout_ms)
    local event = Event.receive(sub, timeout_ms)
    if event then
        return event.data.result
    else
        -- Timeout occurred
        Logger.warn("Task timeout", {timeout = timeout_ms})
        return {success = false, error = "timeout"}
    end
end

-- Retry with exponential backoff
local function wait_with_retry(max_attempts)
    for attempt = 1, max_attempts do
        local timeout = 1000 * math.pow(2, attempt - 1)
        local result = wait_for_result(timeout)
        if result.success then
            return result
        end
    end
    error("Failed after " .. max_attempts .. " attempts")
end
```
**Use Case**: Reliable event handling, timeout management, retry logic

### Monitoring and Statistics

#### 18. Event System Monitoring
**File**: `/examples/lua/events/event-statistics.lua`
```lua
-- Monitor event system health
local stats = {
    events_published = 0,
    events_received = 0,
    subscriptions = {}
}

-- Monitoring wrapper
local original_publish = Event.publish
Event.publish = function(event_type, data, options)
    stats.events_published = stats.events_published + 1
    return original_publish(event_type, data, options)
end

-- Subscription tracking
local function monitored_subscribe(pattern)
    local sub = Event.subscribe(pattern)
    stats.subscriptions[pattern] = Event.subscription_stats(sub)
    return sub
end

-- Periodic reporting
Task.spawn(function()
    while true do
        Utils.sleep(60)  -- Every minute
        print("Event stats:", JSON.encode(stats))
    end
end)
```
**Use Case**: System monitoring, performance tracking, debugging

### Workflow Coordination

#### 19. Event-Driven Workflows
**File**: `/examples/lua/events/event-workflow-coordination.lua`
```lua
-- Coordinate workflow stages via events
local workflow_events = Event.subscribe("workflow.*")

-- Workflow orchestrator
local function orchestrate()
    while true do
        local event = Event.receive(workflow_events, 5000)
        if event then
            if event.event_type == "workflow.stage.completed" then
                -- Determine next stage
                local next_stage = determine_next_stage(
                    event.data.workflow_id,
                    event.data.stage,
                    event.data.result
                )
                
                if next_stage then
                    Event.publish("workflow.stage.start", {
                        workflow_id = event.data.workflow_id,
                        stage = next_stage
                    })
                else
                    Event.publish("workflow.completed", {
                        workflow_id = event.data.workflow_id
                    })
                end
            end
        end
    end
end
```
**Use Case**: Dynamic workflows, stage orchestration, process automation

### Hook-Event Integration

#### 20. Events Triggered by Hooks
**File**: `/examples/lua/events/event-hook-integration.lua`
```lua
-- Hooks that publish events
Hook.register("AfterAgentExecution", function(context)
    -- Publish performance event
    Event.publish("agent.performance", {
        agent_name = context.component_id.name,
        duration_ms = context.data.duration_ms,
        tokens_used = context.data.tokens_used,
        success = context.data.success
    })
    return "continue"
end, "low")

-- Event subscribers that affect hooks
local threshold_sub = Event.subscribe("config.threshold.updated")
Task.spawn(function()
    while true do
        local event = Event.receive(threshold_sub, 1000)
        if event then
            -- Update hook behavior based on event
            update_hook_thresholds(event.data.new_thresholds)
        end
    end
end)
```
**Use Case**: Hook-event coordination, dynamic configuration, monitoring

## Integration Examples

### Complete System Integration

#### 21. Hook-Event Coordination
**File**: `/examples/lua/integration/hook-event-coordination.lua`
```lua
-- Complete example showing hooks publishing events
-- and events triggering hook changes

-- Cost tracking via hooks and events
local cost_threshold = 10.0

Hook.register("AfterAgentExecution", function(context)
    local cost = calculate_cost(context.data.tokens_used)
    
    Event.publish("cost.tracked", {
        component = context.component_id.name,
        cost = cost,
        timestamp = os.time()
    })
    
    if cost > cost_threshold then
        Event.publish("cost.threshold.exceeded", {
            component = context.component_id.name,
            cost = cost,
            threshold = cost_threshold
        })
    end
    
    return "continue"
end)

-- React to threshold events
local threshold_sub = Event.subscribe("cost.threshold.exceeded")
Task.spawn(function()
    while true do
        local event = Event.receive(threshold_sub, 1000)
        if event then
            -- Could disable expensive operations
            print("Cost alert!", event.data.component, event.data.cost)
        end
    end
end)
```
**Use Case**: Bidirectional hook-event integration, reactive systems

### Real-World Applications

#### 22. System Monitoring Dashboard
**File**: `/examples/lua/integration/real-world-monitoring.lua`
```lua
-- Complete monitoring system using hooks and events

-- Performance tracking
Hook.register("AfterAgentExecution", performanceHook, "lowest")
Hook.register("AfterToolExecution", performanceHook, "lowest")

-- Error tracking
Hook.register("AgentError", errorHook, "high")
Hook.register("ToolError", errorHook, "high")

-- Event aggregation
local metrics_sub = Event.subscribe("metrics.*")
local error_sub = Event.subscribe("*.error")

-- Dashboard update loop
Task.spawn(function()
    while true do
        update_dashboard()
        Utils.sleep(1)  -- Update every second
    end
end)
```
**Use Case**: Real-time monitoring, dashboards, observability

#### 23. Data Pipeline Coordination
**File**: `/examples/lua/integration/real-world-pipeline.lua`
```lua
-- Data pipeline with hooks and events

-- Pipeline stages
local pipeline = {
    stages = {"ingest", "validate", "transform", "store"},
    current = 1
}

-- Stage execution hooks
Hook.register("BeforeWorkflowStage", function(context)
    Event.publish("pipeline.stage.starting", {
        stage = pipeline.stages[pipeline.current],
        input_size = context.data.input_size
    })
    return "continue"
end)

-- Stage completion events
local stage_sub = Event.subscribe("pipeline.stage.*")
Task.spawn(function()
    while true do
        local event = Event.receive(stage_sub, 5000)
        if event and event.event_type == "pipeline.stage.completed" then
            pipeline.current = pipeline.current + 1
            if pipeline.current <= #pipeline.stages then
                trigger_next_stage()
            else
                Event.publish("pipeline.completed", {
                    total_time = calculate_total_time()
                })
            end
        end
    end
end)
```
**Use Case**: ETL pipelines, data processing, workflow automation

#### 24. Distributed Error Recovery
**File**: `/examples/lua/integration/real-world-error-recovery.lua`
```lua
-- Distributed error recovery with circuit breakers

-- Component health tracking
local component_health = {}

-- Error detection hook
Hook.register("AgentError", function(context)
    local component = context.component_id.name
    component_health[component] = component_health[component] or {
        failures = 0,
        last_failure = 0
    }
    
    local health = component_health[component]
    health.failures = health.failures + 1
    health.last_failure = os.time()
    
    -- Publish health event
    Event.publish("component.health.degraded", {
        component = component,
        failures = health.failures,
        error = context.data.error
    })
    
    return "continue"
end)

-- Recovery orchestration
local health_sub = Event.subscribe("component.health.*")
Task.spawn(function()
    while true do
        local event = Event.receive(health_sub, 1000)
        if event then
            orchestrate_recovery(event)
        end
    end
end)
```
**Use Case**: Fault tolerance, distributed recovery, self-healing systems

## Running the Examples

### Individual Examples
```bash
# Run a specific example
llmspell run examples/lua/hooks/hook-basic.lua

# Run with configuration
llmspell run --config production.toml examples/lua/events/event-performance.lua
```

### Batch Execution
```bash
# Run all hook examples
./examples/lua/hooks/run-hook-examples.sh

# Run all event examples
./examples/lua/events/run-event-examples.sh

# Run integration examples
./examples/lua/integration/run-integration-examples.sh
```

## Best Practices from Examples

1. **Always Clean Up**: Unsubscribe from events, unregister hooks when done
2. **Use Appropriate Priorities**: Security highest, monitoring lowest
3. **Handle Errors Gracefully**: Use pcall and proper error events
4. **Batch When Possible**: For high-throughput scenarios
5. **Monitor Performance**: Track metrics and set thresholds
6. **Document Integration Points**: Clear comments on hook/event interactions
7. **Test Edge Cases**: Timeouts, errors, and resource limits

## Next Steps

- Experiment with individual examples to understand concepts
- Combine patterns for your specific use cases
- Review the architecture documentation for deeper understanding
- Contribute your own examples to help the community

## Summary

These 23 examples demonstrate:
- **10 Hook Examples**: From basic to advanced patterns
- **10 Event Examples**: Publishing, subscribing, and patterns
- **3 Integration Examples**: Real-world applications
- **Complete Solutions**: Each example is production-ready
- **Progressive Learning**: Start simple, build complexity
- **Best Practices**: Proper resource management and error handling
