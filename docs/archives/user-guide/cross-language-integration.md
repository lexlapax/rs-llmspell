# Cross-Language Integration Guide

## Introduction

rs-llmspell's hook and event systems are designed from the ground up for seamless cross-language integration. This guide explains how hooks and events work across Lua, JavaScript (coming in Phase 15), and Rust, enabling you to build polyglot applications that leverage the strengths of each language.

## Architecture Overview

### Three-Layer Bridge Pattern

```
┌─────────────────────────────────────────────────────────────────┐
│                    Script Layer (User Code)                     │
│  ┌─────────────┐     ┌─────────────┐     ┌─────────────┐      │
│  │     Lua     │     │ JavaScript  │     │   Python    │      │
│  │   Scripts   │     │   Scripts   │     │   Scripts   │      │
│  └──────┬──────┘     └──────┬──────┘     └──────┬──────┘      │
│         │                    │                    │              │
├─────────┴────────────────────┴────────────────────┴─────────────┤
│                  Language Bindings (Sync Wrappers)              │
│  ┌─────────────┐     ┌─────────────┐     ┌─────────────┐      │
│  │ Lua Globals │     │  JS Globals │     │  Py Globals │      │
│  │  Hook/Event │     │  Hook/Event │     │  Hook/Event │      │
│  └──────┬──────┘     └──────┬──────┘     └──────┬──────┘      │
│         │                    │                    │              │
├─────────┴────────────────────┴────────────────────┴─────────────┤
│                     Bridge Layer (Async Core)                    │
│  ┌────────────────────────────────────────────────────────┐    │
│  │  HookBridge  │  EventBridge  │  Language Adapters      │    │
│  │              │               │  (HookAdapter trait)     │    │
│  └────────────────────────────────────────────────────────┘    │
│                              │                                   │
├──────────────────────────────┴───────────────────────────────────┤
│                        Rust Core Systems                         │
│  ┌────────────────┐  ┌────────────────┐  ┌────────────────┐   │
│  │  HookExecutor  │  │    EventBus    │  │  Storage       │   │
│  │  HookRegistry  │  │ FlowController │  │  Backend       │   │
│  └────────────────┘  └────────────────┘  └────────────────┘   │
└─────────────────────────────────────────────────────────────────┘
```

### Key Components

1. **Script Layer**: User code in Lua, JavaScript, or Python
2. **Language Bindings**: Synchronous wrappers that make async Rust code feel native
3. **Bridge Layer**: Thread-safe async bridges that manage cross-language state
4. **Core Systems**: High-performance Rust implementations

## Hook System Cross-Language Support

### Registering Hooks from Scripts

#### Lua
```lua
-- Register a hook from Lua
local handle = Hook.register("BeforeAgentExecution", function(context)
    -- Context is automatically converted from Rust HookContext
    print("Agent executing:", context.component_id.name)
    
    -- Modify input
    return {
        action = "modified",
        modified_data = {
            input = {
                text = context.data.input.text .. " [processed by Lua]"
            }
        }
    }
end, "high")

-- Later, unregister
Hook.unregister(handle)
```

#### JavaScript (Phase 15 Preview)
```javascript
// Register a hook from JavaScript
const handle = Hook.register("BeforeAgentExecution", async (context) => {
    // Context is automatically converted from Rust HookContext
    console.log("Agent executing:", context.componentId.name);
    
    // Modify input
    return {
        action: "modified",
        modifiedData: {
            input: {
                text: context.data.input.text + " [processed by JS]"
            }
        }
    };
}, "high");

// Later, unregister
Hook.unregister(handle);
```

### Hook Context Conversion

The bridge automatically converts between Rust types and script types:

```rust
// Rust HookContext
pub struct HookContext {
    pub hook_point: HookPoint,
    pub component_id: ComponentId,
    pub correlation_id: String,
    pub data: HashMap<String, Value>,
    pub metadata: HashMap<String, Value>,
    pub language: Language,
    pub state: HashMap<String, Value>,
}
```

```lua
-- Lua receives a table
context = {
    hook_point = "BeforeAgentExecution",
    component_id = {
        id = "uuid-here",
        name = "research-agent",
        component_type = "agent"
    },
    correlation_id = "req-12345",
    data = {
        input = {text = "User query"},
        -- other data
    },
    metadata = {
        user_id = "user-123",
        -- other metadata
    },
    language = "lua",
    state = {}
}
```

### Hook Result Types Across Languages

All 9 hook result types work across languages:

| Rust | Lua | JavaScript | Description |
|------|-----|------------|-------------|
| `HookResult::Continue` | `"continue"` | `"continue"` | Proceed normally |
| `HookResult::Modified` | `{action="modified", ...}` | `{action: "modified", ...}` | Modify data |
| `HookResult::Cancel` | `{action="cancel", ...}` | `{action: "cancel", ...}` | Stop execution |
| `HookResult::Redirect` | `{action="redirect", ...}` | `{action: "redirect", ...}` | Redirect to another component |
| `HookResult::Replace` | `{action="replace", ...}` | `{action: "replace", ...}` | Replace component |
| `HookResult::Retry` | `{action="retry", ...}` | `{action: "retry", ...}` | Retry with config |
| `HookResult::Fork` | `{action="fork", ...}` | `{action: "fork", ...}` | Fork execution |
| `HookResult::Cache` | `{action="cache", ...}` | `{action: "cache", ...}` | Use cached result |
| `HookResult::Skipped` | `{action="skipped", ...}` | `{action: "skipped", ...}` | Skip this hook |

## Event System Cross-Language Support

### Publishing Events

#### From Lua
```lua
-- Publish a simple event
Event.publish("user.action.completed", {
    action = "search",
    results = 10,
    duration_ms = 234
})

-- Publish with full options
Event.publish("system.alert", {
    level = "warning",
    message = "High memory usage"
}, {
    language = "lua",
    correlation_id = "req-12345",
    ttl_seconds = 3600
})
```

#### From JavaScript (Phase 15 Preview)
```javascript
// Publish a simple event
Event.publish("user.action.completed", {
    action: "search",
    results: 10,
    durationMs: 234
});

// Publish with full options
Event.publish("system.alert", {
    level: "warning",
    message: "High memory usage"
}, {
    language: "javascript",
    correlationId: "req-12345",
    ttlSeconds: 3600
});
```

### Subscribing to Events

#### Lua Subscription
```lua
-- Subscribe to a pattern
local subscription = Event.subscribe("user.*")

-- Receive events
while true do
    local event = Event.receive(subscription, 1000) -- 1 second timeout
    if event then
        print("Event received:", event.event_type)
        print("From language:", event.source.language)
        print("Data:", event.data)
        
        -- Events from any language appear here
        if event.source.language == "javascript" then
            print("Got event from JavaScript!")
        end
    end
end

-- Clean up
Event.unsubscribe(subscription)
```

#### JavaScript Subscription (Phase 15 Preview)
```javascript
// Subscribe to a pattern
const subscription = Event.subscribe("user.*");

// Receive events
while (true) {
    const event = await Event.receive(subscription, 1000); // 1 second timeout
    if (event) {
        console.log("Event received:", event.eventType);
        console.log("From language:", event.source.language);
        console.log("Data:", event.data);
        
        // Events from any language appear here
        if (event.source.language === "lua") {
            console.log("Got event from Lua!");
        }
    }
}

// Clean up
Event.unsubscribe(subscription);
```

### UniversalEvent Format

All events use the UniversalEvent format for cross-language compatibility:

```json
{
    "id": "550e8400-e29b-41d4-a716-446655440000",
    "event_type": "user.action.completed",
    "timestamp": "2025-07-25T10:30:45.123Z",
    "version": "1.0",
    "source": {
        "component": "script",
        "instance_id": "lua-script-001",
        "correlation_id": "req-12345",
        "language": "lua"
    },
    "data": {
        "action": "search",
        "results": 10,
        "duration_ms": 234
    },
    "metadata": {
        "user_id": "user-123",
        "session_id": "sess-456"
    }
}
```

## Language Adapter Pattern

### HookAdapter Trait

Each language implements the HookAdapter trait:

```rust
#[async_trait]
pub trait HookAdapter: Send + Sync {
    type Context;
    type Result;
    
    /// Language identifier
    fn language(&self) -> Language;
    
    /// Convert HookContext to language-specific format
    async fn adapt_context(
        &self,
        context: &HookContext
    ) -> Result<Self::Context, HookError>;
    
    /// Convert language result to HookResult
    async fn adapt_result(
        &self,
        result: Self::Result
    ) -> Result<HookResult, HookError>;
    
    /// Extract error information
    fn extract_error(
        &self,
        error: &Self::Result
    ) -> Option<String>;
}
```

### Lua Adapter Implementation

```rust
pub struct LuaHookAdapter;

#[async_trait]
impl HookAdapter for LuaHookAdapter {
    type Context = LuaTable<'static>;
    type Result = LuaValue<'static>;
    
    fn language(&self) -> Language {
        Language::Lua
    }
    
    async fn adapt_context(
        &self,
        context: &HookContext
    ) -> Result<Self::Context, HookError> {
        // Convert HookContext to Lua table
        let table = /* conversion logic */;
        Ok(table)
    }
    
    async fn adapt_result(
        &self,
        result: Self::Result
    ) -> Result<HookResult, HookError> {
        // Convert Lua value to HookResult
        match result {
            LuaValue::String(s) if s == "continue" => Ok(HookResult::Continue),
            LuaValue::Table(t) => /* parse table */,
            _ => Err(HookError::InvalidResult)
        }
    }
}
```

## Serialization and Data Flow

### Type Mapping

| Rust Type | Lua Type | JavaScript Type | Notes |
|-----------|----------|-----------------|-------|
| `String` | `string` | `string` | UTF-8 encoded |
| `i64` | `number` | `number` | 53-bit precision in JS |
| `f64` | `number` | `number` | IEEE 754 |
| `bool` | `boolean` | `boolean` | - |
| `HashMap` | `table` | `object` | Key-value pairs |
| `Vec` | `table` | `array` | Sequential values |
| `Option<T>` | `T or nil` | `T or null` | Optional values |
| `DateTime` | `string` | `string` | ISO 8601 format |
| `Uuid` | `string` | `string` | Standard UUID format |

### Complex Data Example

```lua
-- Lua: Create complex data structure
local complexData = {
    user = {
        id = "user-123",
        name = "Alice",
        tags = {"premium", "active"}
    },
    metrics = {
        cpu = 45.2,
        memory = 78.1,
        timestamps = {1627849200, 1627849260, 1627849320}
    },
    nested = {
        level1 = {
            level2 = {
                level3 = "deep value"
            }
        }
    }
}

-- Publish event with complex data
Event.publish("data.processed", complexData)
```

```rust
// Rust: Receives as serde_json::Value
let event = event_bus.receive().await?;
let user_id = event.data["user"]["id"].as_str().unwrap();
let cpu_metric = event.data["metrics"]["cpu"].as_f64().unwrap();
```

## Performance Considerations

### Cross-Language Overhead

| Operation | Overhead | Notes |
|-----------|----------|-------|
| Hook registration | <1ms | One-time cost |
| Hook execution | <2ms | Per hook call |
| Event publish | <0.5ms | Async operation |
| Event receive | <0.5ms | When event available |
| Type conversion | <0.1ms | Simple types |
| Complex conversion | <1ms | Nested structures |

### Optimization Strategies

1. **Batch Operations**
   ```lua
   -- Instead of multiple publishes
   for i = 1, 100 do
       Event.publish("metric", {value = i})
   end
   
   -- Batch into single event
   local metrics = {}
   for i = 1, 100 do
       table.insert(metrics, i)
   end
   Event.publish("metrics.batch", {values = metrics})
   ```

2. **Use Native Types**
   ```lua
   -- Avoid complex serialization
   -- ❌ Bad: Custom userdata
   local customObject = createCustomObject()
   Event.publish("data", customObject) -- Requires serialization
   
   -- ✅ Good: Native tables
   local data = {
       id = customObject:getId(),
       name = customObject:getName()
   }
   Event.publish("data", data) -- Direct conversion
   ```

3. **Minimize Hook Chains**
   ```lua
   -- ❌ Bad: Many small hooks
   Hook.register("BeforeAgentExecution", validateInput, "high")
   Hook.register("BeforeAgentExecution", checkPermissions, "high")
   Hook.register("BeforeAgentExecution", logAccess, "high")
   
   -- ✅ Good: Combined hook
   Hook.register("BeforeAgentExecution", function(context)
       validateInput(context)
       checkPermissions(context)
       logAccess(context)
       return "continue"
   end, "high")
   ```

## Security Considerations

### Sandboxing

Each language runs in a sandboxed environment:

1. **Lua Sandbox**:
   - Restricted globals
   - No file system access (use File tool)
   - No network access (use HTTP tool)
   - No process spawning
   - Memory limits enforced

2. **JavaScript Sandbox** (Phase 15):
   - V8 isolates
   - No Node.js APIs
   - Restricted globals
   - CPU and memory limits

### Data Validation

All cross-language data is validated:

```rust
// Automatic validation in bridge
pub async fn publish_event(
    &self,
    event_type: String,
    data: Value,
) -> Result<(), EventError> {
    // Validate event type
    if !event_type.matches(VALID_EVENT_PATTERN) {
        return Err(EventError::InvalidEventType);
    }
    
    // Validate data size
    if serde_json::to_vec(&data)?.len() > MAX_EVENT_SIZE {
        return Err(EventError::DataTooLarge);
    }
    
    // Sanitize data
    let sanitized = self.sanitize_data(data)?;
    
    // Proceed with publishing
    self.event_bus.publish(event_type, sanitized).await
}
```

## Common Patterns

### 1. Language-Specific Hook Routing

```lua
-- Route to language-specific handlers
Hook.register("BeforeAgentExecution", function(context)
    local handler_map = {
        lua = handleLuaAgent,
        javascript = handleJSAgent,
        rust = handleRustAgent
    }
    
    local handler = handler_map[context.language] or handleDefault
    return handler(context)
end)
```

### 2. Cross-Language RPC via Events

```lua
-- Lua: Request service from JavaScript
local request_id = generateUUID()
Event.publish("rpc.request", {
    id = request_id,
    method = "processData",
    params = {data = "some data"},
    reply_to = "rpc.response." .. request_id
})

-- Subscribe for response
local sub = Event.subscribe("rpc.response." .. request_id)
local response = Event.receive(sub, 5000) -- 5 second timeout
if response then
    print("Result:", response.data.result)
end
Event.unsubscribe(sub)
```

### 3. Language Migration Pattern

```lua
-- Gradually migrate from Lua to JavaScript
Hook.register("BeforeToolExecution", function(context)
    -- Check feature flag
    if Config.get("use_js_validation") then
        -- Publish event for JS handler
        Event.publish("validation.needed", {
            tool = context.data.tool_name,
            params = context.data.parameters
        })
        
        -- Wait for JS response
        local sub = Event.subscribe("validation.result")
        local result = Event.receive(sub, 100)
        Event.unsubscribe(sub)
        
        if result and not result.data.valid then
            return {action = "cancel", reason = result.data.reason}
        end
    else
        -- Use Lua validation
        if not validateTool(context) then
            return {action = "cancel", reason = "Validation failed"}
        end
    end
    
    return "continue"
end)
```

## Debugging Cross-Language Issues

### Enable Debug Logging

```lua
-- Lua: Enable bridge debug logging
Config.set("bridge.debug", true)
Config.set("bridge.log_conversions", true)

-- Log all cross-language events
local debug_sub = Event.subscribe("*")
Task.spawn(function()
    while true do
        local event = Event.receive(debug_sub, 100)
        if event then
            print(string.format("[DEBUG] Event: %s from %s",
                event.event_type,
                event.source.language or "unknown"
            ))
        end
    end
end)
```

### Common Issues and Solutions

1. **Type Conversion Errors**
   ```lua
   -- Problem: Userdata can't be serialized
   local file = File.open("data.txt")
   Event.publish("file.opened", {file = file}) -- Error!
   
   -- Solution: Extract serializable data
   Event.publish("file.opened", {
       path = "data.txt",
       size = file:size(),
       mode = "read"
   })
   ```

2. **Missing Hook Results**
   ```lua
   -- Problem: Forgot to return
   Hook.register("BeforeAgentExecution", function(context)
       validateInput(context)
       -- No return!
   end)
   
   -- Solution: Always return a result
   Hook.register("BeforeAgentExecution", function(context)
       validateInput(context)
       return "continue" -- Explicit return
   end)
   ```

3. **Event Pattern Mismatches**
   ```lua
   -- Problem: Pattern doesn't match
   Event.subscribe("user.*.completed") -- Won't match "user.completed"
   
   -- Solution: Use correct patterns
   Event.subscribe("user.**") -- Matches all user events
   -- or
   Event.subscribe("user.completed") -- Exact match
   ```

## Future Enhancements (Phase 15+)

### JavaScript Integration
- Full Hook and Event API
- Promise-based async patterns
- TypeScript type definitions
- React/Vue integration helpers

### Python Integration
- Async/await support
- Type hints via stub files
- Popular framework integration
- Data science library compatibility

### WebAssembly Support
- Run any WASM language
- Standardized component model
- Near-native performance
- Browser compatibility

## Best Practices

1. **Use Events for Loose Coupling**: Don't create tight dependencies between language components
2. **Keep Hooks Fast**: Cross-language calls add overhead
3. **Validate Early**: Check data before crossing language boundaries
4. **Handle Errors Gracefully**: Language bridges can fail
5. **Monitor Performance**: Track cross-language overhead
6. **Document Language Requirements**: Be clear about which languages are needed
7. **Test Cross-Language Scenarios**: Don't assume compatibility

## Examples

See the following examples for practical cross-language patterns:
- `/examples/lua/hooks/hook-cross-language.lua`
- `/examples/lua/events/event-cross-language.lua`
- `/examples/lua/integration/hook-event-coordination.lua`

## Summary

Cross-language integration in rs-llmspell:
- **Seamless**: Automatic type conversion and marshalling
- **Performant**: <2ms overhead for most operations
- **Secure**: Sandboxed execution environments
- **Flexible**: Support for multiple languages and patterns
- **Future-proof**: Designed for easy language additions
