# Events Guide

## Introduction

The event system in rs-llmspell provides high-throughput asynchronous messaging for monitoring, analytics, and loose coupling between components. With support for pattern-based subscriptions and automatic backpressure management, events enable scalable reactive architectures.

## Core Concepts

### What is an Event?

An event is an asynchronous notification about something that happened in the system. Events:
- Are fire-and-forget (no return value)
- Support pattern-based subscriptions
- Handle 90,000+ events per second
- Work across languages (Lua, JavaScript, Rust)
- Can be persisted with storage backend
- Include automatic flow control

### Event Lifecycle

```
1. Publish → 2. Pattern Matching → 3. Queue Distribution → 4. Delivery
      ↓              ↓                      ↓                    ↓
  Validation    Route to Subs        Backpressure          Receive/Poll
```

## UniversalEvent Format

All events follow the UniversalEvent structure for cross-language compatibility:

```lua
{
    -- Event metadata
    id = "550e8400-e29b-41d4-a716-446655440000",
    event_type = "agent.execution.completed",
    timestamp = "2025-07-25T10:30:45.123Z",
    version = "1.0",
    
    -- Event source
    source = {
        component = "agent",
        instance_id = "research-agent-001",
        correlation_id = "req-12345"
    },
    
    -- Event payload
    data = {
        -- Event-specific data
        agent_name = "researcher",
        tokens_used = 150,
        duration_ms = 234,
        success = true
    },
    
    -- Optional metadata
    metadata = {
        environment = "production",
        region = "us-east-1"
    }
}
```

## Event Patterns

### Naming Conventions

Events use hierarchical dot notation:
- `component.action.status`
- `system.resource.event`
- `custom.domain.specific`

Examples:
- `agent.execution.started`
- `tool.filesystem.read`
- `workflow.stage.completed`
- `system.memory.high`

### Pattern Matching

Subscribe using glob patterns:

| Pattern | Matches | Example Events |
|---------|---------|----------------|
| `*` | Any single segment | `agent.*` matches `agent.started`, `agent.stopped` |
| `**` | Any number of segments | `agent.**` matches `agent.execution.started`, `agent.error` |
| `?` | Single character | `agent.?.started` matches `agent.1.started` |
| `[abc]` | Character set | `agent.[123].started` matches `agent.1.started` |
| `{a,b}` | Alternatives | `{agent,tool}.started` matches both |

## Publishing Events

### Basic Publishing

```lua
-- Simple event
Event.publish("custom.task.completed", {
    task_id = "task-123",
    duration_ms = 1500,
    status = "success"
})
```

### With Full Metadata

```lua
-- Detailed event with all fields
Event.publish({
    event_type = "payment.processed",
    data = {
        amount = 99.99,
        currency = "USD",
        method = "credit_card"
    },
    source = {
        component = "payment_service",
        instance_id = "payment-001"
    },
    metadata = {
        user_id = "user-456",
        session_id = "sess-789"
    }
})
```

### Batch Publishing

```lua
-- Publish multiple events efficiently
local events = {
    {event_type = "metric.cpu", data = {value = 45.2}},
    {event_type = "metric.memory", data = {value = 78.1}},
    {event_type = "metric.disk", data = {value = 62.5}}
}

for _, event in ipairs(events) do
    Event.publish(event.event_type, event.data)
end
```

## Subscribing to Events

### Basic Subscription

```lua
-- Subscribe to specific event
local sub = Event.subscribe("agent.execution.completed")

-- Receive events (blocking with timeout)
local event = Event.receive(sub, 5000)  -- 5 second timeout
if event then
    print("Agent completed:", event.data.agent_name)
end
```

### Pattern Subscriptions

```lua
-- Subscribe to all agent events
local agent_sub = Event.subscribe("agent.*")

-- Subscribe to all errors
local error_sub = Event.subscribe("**.error")

-- Subscribe to multiple patterns
local multi_sub = Event.subscribe({
    "agent.execution.*",
    "tool.*.completed",
    "workflow.stage.failed"
})
```

### Non-blocking Receive

```lua
-- Poll for events without blocking
local event = Event.receive(sub, 0)  -- No wait
if event then
    -- Process event
else
    -- No events available
end
```

### Batch Receiving

```lua
-- Receive multiple events at once
local events = Event.receive_batch(sub, {
    max_events = 10,
    timeout_ms = 1000
})

for _, event in ipairs(events) do
    print("Received:", event.event_type)
end
```

## Flow Control and Backpressure

The FlowController manages event throughput with four strategies:

### 1. DropOldest (Default)

```lua
-- Configure subscription with DropOldest
local sub = Event.subscribe("high.volume.*", {
    backpressure = "drop_oldest",
    max_queue_size = 10000
})
```

Oldest events are dropped when queue is full. Good for real-time monitoring.

### 2. DropNewest

```lua
-- Keep older events, drop new ones
local sub = Event.subscribe("audit.*", {
    backpressure = "drop_newest",
    max_queue_size = 50000
})
```

New events are rejected when queue is full. Good for audit trails.

### 3. Block

```lua
-- Block publishers when queue is full
local sub = Event.subscribe("critical.*", {
    backpressure = "block",
    max_queue_size = 1000,
    block_timeout_ms = 5000
})
```

Publishers wait when queue is full. Good for critical events.

### 4. Reject

```lua
-- Reject events immediately when full
local sub = Event.subscribe("optional.*", {
    backpressure = "reject",
    max_queue_size = 5000
})
```

Publishers get error when queue is full. Good for optional events.

## Event Persistence

With storage backend configured, events can be persisted:

```lua
-- Publish persistent event
Event.publish("important.transaction", {
    amount = 1000,
    account = "ACC-123"
}, {
    persistent = true,
    ttl_seconds = 86400  -- Keep for 24 hours
})

-- Query historical events
local historical = Event.query({
    event_type = "important.transaction",
    start_time = "2025-07-24T00:00:00Z",
    end_time = "2025-07-25T00:00:00Z",
    filters = {
        ["data.amount"] = {">=", 500}
    }
})
```

## Performance Characteristics

### Throughput

- **Publishing**: 100,000+ events/second
- **Pattern matching**: O(1) for exact, O(n) for wildcards
- **Delivery**: 90,000+ events/second per subscriber
- **Memory**: ~500 bytes per queued event

### Optimization Tips

1. **Use specific patterns**: More specific = better performance
2. **Batch operations**: Reduce overhead for high-volume scenarios
3. **Set appropriate queue sizes**: Balance memory vs. reliability
4. **Choose right backpressure**: Match strategy to use case

## Practical Examples

### 1. Metrics Collection

```lua
-- Metrics collector
local MetricsCollector = {
    init = function()
        local sub = Event.subscribe("metric.*")
        local metrics = {}
        
        -- Background collection
        while true do
            local event = Event.receive(sub, 1000)
            if event then
                local metric_type = event.event_type:match("metric%.(.+)")
                metrics[metric_type] = metrics[metric_type] or {}
                table.insert(metrics[metric_type], {
                    value = event.data.value,
                    timestamp = event.timestamp
                })
                
                -- Aggregate every 100 metrics
                if #metrics[metric_type] >= 100 then
                    local avg = 0
                    for _, m in ipairs(metrics[metric_type]) do
                        avg = avg + m.value
                    end
                    avg = avg / #metrics[metric_type]
                    
                    Event.publish("metric.aggregated", {
                        type = metric_type,
                        average = avg,
                        count = #metrics[metric_type]
                    })
                    
                    metrics[metric_type] = {}
                end
            end
        end
    end
}
```

### 2. Error Monitoring

```lua
-- Centralized error handler
local ErrorMonitor = {
    init = function()
        local sub = Event.subscribe("**.error")
        local error_counts = {}
        
        while true do
            local events = Event.receive_batch(sub, {
                max_events = 50,
                timeout_ms = 5000
            })
            
            for _, event in ipairs(events) do
                local component = event.source.component or "unknown"
                error_counts[component] = (error_counts[component] or 0) + 1
                
                -- Alert on threshold
                if error_counts[component] > 10 then
                    Event.publish("alert.error_threshold", {
                        component = component,
                        count = error_counts[component],
                        recent_error = event.data.error
                    })
                    error_counts[component] = 0  -- Reset
                end
            end
        end
    end
}
```

### 3. Workflow Orchestration

```lua
-- Event-driven workflow
local EventWorkflow = {
    run = function()
        -- Subscribe to workflow events
        local subs = {
            data_ready = Event.subscribe("data.import.completed"),
            processing = Event.subscribe("processing.*.completed"),
            errors = Event.subscribe("workflow.*.error")
        }
        
        -- Wait for data
        local data_event = Event.receive(subs.data_ready, 60000)
        if not data_event then
            error("Timeout waiting for data")
        end
        
        -- Trigger processing
        Event.publish("processing.started", {
            data_id = data_event.data.import_id,
            stages = {"validate", "transform", "analyze"}
        })
        
        -- Monitor progress
        local completed_stages = {}
        while #completed_stages < 3 do
            local event = Event.receive(subs.processing, 30000)
            if event then
                table.insert(completed_stages, event.data.stage)
                print("Completed stage:", event.data.stage)
            else
                -- Check for errors
                local error_event = Event.receive(subs.errors, 0)
                if error_event then
                    error("Workflow failed: " .. error_event.data.error)
                end
            end
        end
        
        Event.publish("workflow.completed", {
            duration_ms = os.clock() * 1000,
            stages = completed_stages
        })
    end
}
```

### 4. Cross-Language Communication

```lua
-- Lua publishes events for JavaScript consumers
local CrossLangBridge = {
    notifyJS = function(action, data)
        Event.publish("bridge.lua_to_js", {
            action = action,
            data = data,
            language = "lua",
            timestamp = os.time()
        })
    end,
    
    listenToJS = function()
        local sub = Event.subscribe("bridge.js_to_lua")
        while true do
            local event = Event.receive(sub, 1000)
            if event then
                print("Received from JS:", event.data.action)
                -- Process JavaScript request
            end
        end
    end
}
```

## Event Management

### Listing Subscriptions

```lua
-- Get all active subscriptions
local subs = Event.list_subscriptions()
for _, sub in ipairs(subs) do
    print(sub.id, sub.pattern, sub.queue_size)
end

-- Get subscription stats
local stats = Event.subscription_stats(sub)
print("Events received:", stats.total_received)
print("Events dropped:", stats.dropped)
print("Current queue:", stats.queue_size)
```

### Unsubscribing

```lua
-- Method 1: Using subscription handle
local sub = Event.subscribe("temp.*")
-- ... use subscription ...
sub:unsubscribe()

-- Method 2: Using Event.unsubscribe
Event.unsubscribe(sub)

-- Unsubscribe all matching patterns
Event.unsubscribe_pattern("temp.*")
```

### Event Filtering

```lua
-- Subscribe with filters
local sub = Event.subscribe("transaction.*", {
    filter = function(event)
        return event.data.amount > 100
    end
})

-- Only high-value transactions will be received
```

## Best Practices

### 1. Event Naming

```lua
-- ✅ Good: Hierarchical, descriptive
Event.publish("payment.processing.started", data)
Event.publish("user.profile.updated", data)
Event.publish("system.resource.memory.high", data)

-- ❌ Bad: Flat, unclear
Event.publish("payment_start", data)
Event.publish("update", data)
Event.publish("memory", data)
```

### 2. Subscription Lifecycle

```lua
-- ✅ Good: Clean up subscriptions
local function processEvents()
    local sub = Event.subscribe("task.*")
    
    local ok, err = pcall(function()
        -- Process events
        while running do
            local event = Event.receive(sub, 1000)
            -- Handle event
        end
    end)
    
    -- Always clean up
    sub:unsubscribe()
    
    if not ok then error(err) end
end

-- ❌ Bad: Leaking subscriptions
local function processEvents()
    local sub = Event.subscribe("task.*")
    -- Process events
    -- Never unsubscribe - memory leak!
end
```

### 3. Error Handling

```lua
-- Robust event processing
local function safeEventHandler(event)
    local ok, result = pcall(function()
        -- Your event handling logic
        return processEvent(event)
    end)
    
    if not ok then
        -- Don't let handler errors break the event loop
        Logger.error("Event handler failed", {
            error = result,
            event_type = event.event_type
        })
        
        -- Optionally publish error event
        Event.publish("handler.error", {
            original_event = event.event_type,
            error = tostring(result)
        })
    end
    
    return ok, result
end
```

### 4. Performance Optimization

```lua
-- ✅ Efficient: Batch processing
local events = Event.receive_batch(sub, {
    max_events = 100,
    timeout_ms = 1000
})
for _, event in ipairs(events) do
    -- Process in batch
end

-- ❌ Inefficient: Individual receives in tight loop
for i = 1, 100 do
    local event = Event.receive(sub, 10)
    if event then
        -- Process one by one
    end
end
```

## Troubleshooting

### Events Not Received

1. **Check pattern match**: Verify pattern matches event type
2. **Check backpressure**: Events may be dropped
3. **Check subscription**: Ensure subscription is active
4. **Check timing**: Event may be published before subscription

### High Memory Usage

1. **Reduce queue sizes**: Lower max_queue_size
2. **Process events faster**: Reduce processing time
3. **Use appropriate backpressure**: Drop events if needed
4. **Unsubscribe unused**: Clean up inactive subscriptions

### Performance Issues

1. **Use specific patterns**: Avoid broad wildcards
2. **Batch operations**: Process multiple events together
3. **Async processing**: Don't block event loop
4. **Monitor queue sizes**: Check subscription stats

## Advanced Topics

### Event Replay

```lua
-- Record events for replay
local recorded_events = {}
local recording_sub = Event.subscribe("**")

local function startRecording(duration)
    local start_time = os.time()
    while os.time() - start_time < duration do
        local event = Event.receive(recording_sub, 100)
        if event then
            table.insert(recorded_events, event)
        end
    end
end

-- Replay recorded events
local function replay()
    for _, event in ipairs(recorded_events) do
        Event.publish(event.event_type, event.data)
        -- Optional: maintain timing
        os.execute("sleep 0.1")
    end
end
```

### Event Transformation

```lua
-- Transform events between formats
local Transformer = {
    init = function()
        local sub = Event.subscribe("legacy.*")
        
        while true do
            local event = Event.receive(sub, 1000)
            if event then
                -- Transform to new format
                local new_event = {
                    event_type = "modern." .. event.event_type:sub(8),
                    data = transform_data(event.data),
                    metadata = {
                        transformed_from = event.event_type,
                        transform_time = os.time()
                    }
                }
                
                Event.publish(new_event.event_type, new_event.data)
            end
        end
    end
}
```

## Next Steps

- **[Hooks Guide](./hooks-guide.md)**: Learn about synchronous interception
- **[Built-in Events Reference](./builtin-events-reference.md)**: Standard event types
- **[Event Patterns](./event-patterns.md)**: Common patterns and recipes
- **[Event Development Guide](../developer-guide/event-development-guide.md)**: Create custom events
- **[Examples](./examples/hooks-events-cookbook.md)**: 23 working examples

## Summary

- Events provide asynchronous, high-throughput messaging
- Pattern-based subscriptions enable flexible routing
- FlowController manages backpressure automatically
- Cross-language support enables universal communication
- 90,000+ events/second throughput for scalable systems
- Storage backend enables event persistence and replay
