# Cross-Engine Compatibility Analysis

## Overview

This document analyzes compatibility challenges and solutions for running rs-llmspell across different scripting engines (Lua, JavaScript, and future languages). The goal is to ensure consistent behavior, performance, and developer experience regardless of the chosen scripting runtime.

## Hook Registration Across Engines

### Challenge: Engine-Specific Hook APIs

Different scripting engines have varying capabilities for function registration, callback handling, and metadata management.

### Lua Hook Registration

```lua
-- Lua hook registration patterns
local HookRegistry = {}

-- Function-based hook registration
Hooks.register("before_llm_call", function(context)
    -- Lua-specific context handling
    local input_text = context.input.message
    local agent_id = context.agent_id
    
    -- Lua-style logging
    print(string.format("[%s] Processing: %s", agent_id, input_text))
    
    -- Lua table manipulation
    context.metadata = context.metadata or {}
    context.metadata.processed_at = os.time()
    
    return {
        success = true,
        modifications = {
            input_validation_passed = true
        }
    }
end)

-- Table-based hook registration with metadata
Hooks.register("after_tool_call", {
    name = "tool_performance_monitor",
    priority = 100,
    execution_mode = "synchronous",
    
    handler = function(context)
        local duration = context.execution_time or 0
        
        -- Lua-specific performance tracking
        if not _G.tool_stats then
            _G.tool_stats = {}
        end
        
        local tool_name = context.tool_name
        if not _G.tool_stats[tool_name] then
            _G.tool_stats[tool_name] = {
                call_count = 0,
                total_time = 0,
                avg_time = 0
            }
        end
        
        local stats = _G.tool_stats[tool_name]
        stats.call_count = stats.call_count + 1
        stats.total_time = stats.total_time + duration
        stats.avg_time = stats.total_time / stats.call_count
        
        return { success = true }
    end
})

-- Lua-specific coroutine-based async hook
Hooks.register("before_workflow_start", {
    name = "async_workflow_validator",
    execution_mode = "asynchronous",
    
    handler = function(context)
        return coroutine.wrap(function()
            -- Async validation steps
            local validation_steps = {
                "check_permissions",
                "validate_inputs", 
                "verify_resources"
            }
            
            for _, step in ipairs(validation_steps) do
                print("Validation step:", step)
                -- Simulate async work
                coroutine.yield()
                
                local result = perform_validation_step(step, context)
                if not result.success then
                    return {
                        success = false,
                        error = "Validation failed: " .. step
                    }
                end
            end
            
            return { success = true }
        end)()
    end
})
```

### JavaScript Hook Registration

```javascript
// JavaScript hook registration patterns

// Function-based hook registration
Hooks.register('beforeLLMCall', (context) => {
    // JavaScript-specific context handling
    const { input, agentId } = context;
    
    // JavaScript-style logging
    console.log(`[${agentId}] Processing: ${input.message}`);
    
    // JavaScript object manipulation
    context.metadata = context.metadata || {};
    context.metadata.processedAt = Date.now();
    
    return {
        success: true,
        modifications: {
            inputValidationPassed: true
        }
    };
});

// Object-based hook registration with metadata
Hooks.register('afterToolCall', {
    name: 'toolPerformanceMonitor',
    priority: 100,
    executionMode: 'synchronous',
    
    handler: (context) => {
        const duration = context.executionTime || 0;
        
        // JavaScript-specific performance tracking
        if (!global.toolStats) {
            global.toolStats = new Map();
        }
        
        const toolName = context.toolName;
        if (!global.toolStats.has(toolName)) {
            global.toolStats.set(toolName, {
                callCount: 0,
                totalTime: 0,
                avgTime: 0
            });
        }
        
        const stats = global.toolStats.get(toolName);
        stats.callCount++;
        stats.totalTime += duration;
        stats.avgTime = stats.totalTime / stats.callCount;
        
        return { success: true };
    }
});

// JavaScript Promise-based async hook
Hooks.register('beforeWorkflowStart', {
    name: 'asyncWorkflowValidator',
    executionMode: 'asynchronous',
    
    handler: async (context) => {
        // Async validation steps
        const validationSteps = [
            'checkPermissions',
            'validateInputs',
            'verifyResources'
        ];
        
        for (const step of validationSteps) {
            console.log('Validation step:', step);
            
            const result = await performValidationStep(step, context);
            if (!result.success) {
                return {
                    success: false,
                    error: `Validation failed: ${step}`
                };
            }
        }
        
        return { success: true };
    }
});

// Class-based hook registration
class AdvancedPerformanceHook {
    constructor(config) {
        this.name = config.name;
        this.priority = config.priority || 50;
        this.metricsCollector = new MetricsCollector();
    }
    
    async handle(context) {
        const startTime = process.hrtime.bigint();
        
        try {
            // Hook-specific logic
            await this.processContext(context);
            
            const endTime = process.hrtime.bigint();
            const duration = Number(endTime - startTime) / 1000000; // Convert to ms
            
            this.metricsCollector.recordHookExecution(this.name, duration);
            
            return {
                success: true,
                metadata: {
                    hookExecutionTime: duration
                }
            };
        } catch (error) {
            return {
                success: false,
                error: error.message
            };
        }
    }
    
    async processContext(context) {
        // Override in subclasses
    }
}

// Register class-based hook
const performanceHook = new AdvancedPerformanceHook({
    name: 'advancedPerformanceMonitor',
    priority: 75
});

Hooks.register('afterAgentExecution', performanceHook);
```

### Cross-Engine Hook Compatibility Layer

```rust
// Rust compatibility layer for cross-engine hooks
pub struct CrossEngineHookRegistry {
    lua_hooks: HashMap<HookPoint, Vec<LuaHookWrapper>>,
    js_hooks: HashMap<HookPoint, Vec<JSHookWrapper>>,
    native_hooks: HashMap<HookPoint, Vec<Box<dyn Hook>>>,
    compatibility_layer: HookCompatibilityLayer,
}

pub struct HookCompatibilityLayer {
    context_translator: ContextTranslator,
    result_translator: ResultTranslator,
    error_handler: CrossEngineErrorHandler,
}

// Context translation between engines
pub struct ContextTranslator;

impl ContextTranslator {
    pub fn to_lua_context(&self, context: &HookContext) -> Result<mlua::Table> {
        let lua = mlua::Lua::new();
        let table = lua.create_table()?;
        
        // Convert Rust HookContext to Lua table
        table.set("hook_point", context.hook_point.to_string())?;
        table.set("agent_id", context.agent_id.clone())?;
        
        // Convert input JSON to Lua table
        let input_table = self.json_to_lua_table(&lua, &context.input)?;
        table.set("input", input_table)?;
        
        // Convert metadata HashMap to Lua table
        let metadata_table = lua.create_table()?;
        for (key, value) in &context.metadata {
            let lua_value = self.json_value_to_lua(&lua, value)?;
            metadata_table.set(key.clone(), lua_value)?;
        }
        table.set("metadata", metadata_table)?;
        
        Ok(table)
    }
    
    pub fn to_js_context(&self, context: &HookContext) -> Result<serde_json::Value> {
        // Convert Rust HookContext to JavaScript object
        Ok(json!({
            "hookPoint": context.hook_point.to_string(),
            "agentId": context.agent_id,
            "input": context.input,
            "metadata": context.metadata,
            "timestamp": context.timestamp.timestamp_millis(),
            "threadId": format!("{:?}", std::thread::current().id())
        }))
    }
    
    pub fn from_lua_result(&self, lua_result: mlua::Value) -> Result<HookResult> {
        match lua_result {
            mlua::Value::Table(table) => {
                let success = table.get::<_, bool>("success").unwrap_or(false);
                
                let modifications = if let Ok(mods_table) = table.get::<_, mlua::Table>("modifications") {
                    self.lua_table_to_hashmap(&mods_table)?
                } else {
                    HashMap::new()
                };
                
                let metadata = if let Ok(meta_table) = table.get::<_, mlua::Table>("metadata") {
                    self.lua_table_to_hashmap(&meta_table)?
                } else {
                    HashMap::new()
                };
                
                Ok(HookResult {
                    success,
                    modifications,
                    metadata,
                })
            },
            _ => Err(anyhow!("Invalid Lua hook result format"))
        }
    }
    
    pub fn from_js_result(&self, js_result: serde_json::Value) -> Result<HookResult> {
        Ok(HookResult {
            success: js_result.get("success")
                .and_then(|v| v.as_bool())
                .unwrap_or(false),
            modifications: js_result.get("modifications")
                .and_then(|v| v.as_object())
                .map(|obj| obj.iter().map(|(k, v)| (k.clone(), v.clone())).collect())
                .unwrap_or_default(),
            metadata: js_result.get("metadata")
                .and_then(|v| v.as_object())
                .map(|obj| obj.iter().map(|(k, v)| (k.clone(), v.clone())).collect())
                .unwrap_or_default(),
        })
    }
}

// Cross-engine hook execution
impl CrossEngineHookRegistry {
    pub async fn execute_hooks(&self, point: HookPoint, context: &mut HookContext) -> Result<CrossEngineHookResult> {
        let mut all_results = Vec::new();
        
        // Execute native Rust hooks first (highest priority)
        if let Some(native_hooks) = self.native_hooks.get(&point) {
            for hook in native_hooks {
                let result = hook.execute(context).await?;
                all_results.push(EngineHookResult::Native(result));
            }
        }
        
        // Execute Lua hooks
        if let Some(lua_hooks) = self.lua_hooks.get(&point) {
            for lua_hook in lua_hooks {
                let lua_context = self.compatibility_layer.context_translator.to_lua_context(context)?;
                let lua_result = lua_hook.execute(lua_context).await?;
                let translated_result = self.compatibility_layer.context_translator.from_lua_result(lua_result)?;
                all_results.push(EngineHookResult::Lua(translated_result));
            }
        }
        
        // Execute JavaScript hooks
        if let Some(js_hooks) = self.js_hooks.get(&point) {
            for js_hook in js_hooks {
                let js_context = self.compatibility_layer.context_translator.to_js_context(context)?;
                let js_result = js_hook.execute(js_context).await?;
                let translated_result = self.compatibility_layer.context_translator.from_js_result(js_result)?;
                all_results.push(EngineHookResult::JavaScript(translated_result));
            }
        }
        
        // Merge results from all engines
        self.merge_cross_engine_results(all_results, context)
    }
    
    fn merge_cross_engine_results(&self, results: Vec<EngineHookResult>, context: &mut HookContext) -> Result<CrossEngineHookResult> {
        let mut merged_modifications = HashMap::new();
        let mut merged_metadata = HashMap::new();
        let mut failed_hooks = Vec::new();
        let mut successful_hooks = Vec::new();
        
        for (index, result) in results.into_iter().enumerate() {
            match result {
                EngineHookResult::Native(hook_result) |
                EngineHookResult::Lua(hook_result) |
                EngineHookResult::JavaScript(hook_result) => {
                    if hook_result.success {
                        successful_hooks.push(index);
                        
                        // Merge modifications (later hooks override earlier ones)
                        for (key, value) in hook_result.modifications {
                            merged_modifications.insert(key, value);
                        }
                        
                        // Merge metadata
                        for (key, value) in hook_result.metadata {
                            merged_metadata.insert(format!("hook_{}_{}", index, key), value);
                        }
                    } else {
                        failed_hooks.push(HookFailure {
                            hook_id: format!("hook_{}", index),
                            error: "Hook execution failed".to_string(),
                            engine: match result {
                                EngineHookResult::Native(_) => "rust".to_string(),
                                EngineHookResult::Lua(_) => "lua".to_string(),
                                EngineHookResult::JavaScript(_) => "javascript".to_string(),
                            }
                        });
                    }
                }
            }
        }
        
        // Apply modifications to context
        for (key, value) in &merged_modifications {
            context.metadata.insert(key.clone(), value.clone());
        }
        
        Ok(CrossEngineHookResult {
            successful_hooks: successful_hooks.len(),
            failed_hooks,
            applied_modifications: merged_modifications,
            combined_metadata: merged_metadata,
        })
    }
}
```

## Event Handling Differences

### Engine-Specific Event Patterns

Different scripting engines have varying event handling capabilities and patterns.

### Lua Event Handling

```lua
-- Lua event handling patterns
local EventEmitter = {}
EventEmitter.__index = EventEmitter

function EventEmitter.new()
    return setmetatable({
        listeners = {},
        wildcard_listeners = {}
    }, EventEmitter)
end

-- Traditional callback-based events
function EventEmitter:on(event_type, callback)
    if not self.listeners[event_type] then
        self.listeners[event_type] = {}
    end
    table.insert(self.listeners[event_type], callback)
end

-- Wildcard event handling
function EventEmitter:on_any(callback)
    table.insert(self.wildcard_listeners, callback)
end

-- Coroutine-based event handling
function EventEmitter:on_async(event_type, async_handler)
    self:on(event_type, function(event_data)
        -- Wrap async handler in coroutine
        local co = coroutine.create(async_handler)
        
        local function resume_coroutine()
            local success, result = coroutine.resume(co, event_data)
            if not success then
                print("Error in async event handler:", result)
            elseif coroutine.status(co) ~= "dead" then
                -- Handler yielded, schedule continuation
                coroutine.yield() -- Yield to allow other handlers
                resume_coroutine()
            end
        end
        
        resume_coroutine()
    end)
end

-- Event emission with Lua-specific features
function EventEmitter:emit(event_type, event_data)
    local event = {
        type = event_type,
        data = event_data,
        timestamp = os.time(),
        source = "lua_emitter"
    }
    
    -- Call specific listeners
    if self.listeners[event_type] then
        for _, callback in ipairs(self.listeners[event_type]) do
            local success, result = pcall(callback, event)
            if not success then
                print("Event handler error:", result)
            end
        end
    end
    
    -- Call wildcard listeners
    for _, callback in ipairs(self.wildcard_listeners) do
        local success, result = pcall(callback, event)
        if not success then
            print("Wildcard event handler error:", result)
        end
    end
end

-- Usage example
local emitter = EventEmitter.new()

-- Standard event handler
emitter:on("user_login", function(event)
    print("User logged in:", event.data.user_id)
    update_user_stats(event.data.user_id)
end)

-- Async event handler with coroutines
emitter:on_async("file_upload", function(event)
    print("Starting file processing...")
    
    -- Simulate async operations with yields
    coroutine.yield() -- Allow other events to process
    
    local file_path = event.data.file_path
    process_file(file_path)
    
    coroutine.yield() -- Another yield point
    
    validate_file(file_path)
    
    print("File processing completed")
end)

-- Wildcard handler for debugging
emitter:on_any(function(event)
    print("Event received:", event.type, event.timestamp)
end)
```

### JavaScript Event Handling

```javascript
// JavaScript event handling patterns

class AdvancedEventEmitter extends EventEmitter {
    constructor() {
        super();
        this.wildcardListeners = [];
        this.asyncHandlers = new Map();
        this.eventHistory = [];
        this.maxHistorySize = 1000;
    }
    
    // Enhanced event emission with metadata
    emit(eventType, eventData) {
        const event = {
            type: eventType,
            data: eventData,
            timestamp: Date.now(),
            source: 'js_emitter',
            id: this.generateEventId()
        };
        
        // Store in history
        this.eventHistory.push(event);
        if (this.eventHistory.length > this.maxHistorySize) {
            this.eventHistory.shift();
        }
        
        // Emit to specific listeners
        super.emit(eventType, event);
        
        // Emit to wildcard listeners
        this.wildcardListeners.forEach(callback => {
            try {
                callback(event);
            } catch (error) {
                console.error('Wildcard event handler error:', error);
            }
        });
        
        return event.id;
    }
    
    // Async event handling with Promises
    onAsync(eventType, asyncHandler) {
        const wrappedHandler = async (event) => {
            try {
                await asyncHandler(event);
            } catch (error) {
                console.error(`Async event handler error for ${eventType}:`, error);
                this.emit('handlerError', { eventType, error, event });
            }
        };
        
        this.on(eventType, wrappedHandler);
        
        // Track async handlers
        if (!this.asyncHandlers.has(eventType)) {
            this.asyncHandlers.set(eventType, []);
        }
        this.asyncHandlers.get(eventType).push(wrappedHandler);
    }
    
    // Observable-style event handling
    observe(eventType) {
        return new Promise((resolve) => {
            this.once(eventType, resolve);
        });
    }
    
    // Stream-style event handling
    stream(eventType) {
        return new ReadableStream({
            start: (controller) => {
                const handler = (event) => {
                    controller.enqueue(event);
                };
                
                this.on(eventType, handler);
                
                // Cleanup when stream is cancelled
                return () => {
                    this.off(eventType, handler);
                };
            }
        });
    }
    
    // Wildcard event handling
    onAny(callback) {
        this.wildcardListeners.push(callback);
    }
    
    offAny(callback) {
        const index = this.wildcardListeners.indexOf(callback);
        if (index !== -1) {
            this.wildcardListeners.splice(index, 1);
        }
    }
    
    // Event replay functionality
    replayEvents(filter, callback) {
        const filteredEvents = this.eventHistory.filter(filter);
        filteredEvents.forEach(callback);
    }
    
    generateEventId() {
        return `${Date.now()}_${Math.random().toString(36).substr(2, 9)}`;
    }
}

// Usage examples
const emitter = new AdvancedEventEmitter();

// Standard event handler
emitter.on('userLogin', (event) => {
    console.log('User logged in:', event.data.userId);
    updateUserStats(event.data.userId);
});

// Async event handler
emitter.onAsync('fileUpload', async (event) => {
    console.log('Starting file processing...');
    
    const filePath = event.data.filePath;
    
    try {
        await processFile(filePath);
        await validateFile(filePath);
        console.log('File processing completed');
        
        emitter.emit('fileProcessed', {
            filePath,
            success: true
        });
    } catch (error) {
        console.error('File processing failed:', error);
        emitter.emit('fileProcessed', {
            filePath,
            success: false,
            error: error.message
        });
    }
});

// Promise-based event waiting
async function waitForUserAction() {
    const userEvent = await emitter.observe('userAction');
    console.log('User action received:', userEvent.data);
    return userEvent.data;
}

// Stream-based event processing
const userEventStream = emitter.stream('userEvent');
const reader = userEventStream.getReader();

async function processUserEventStream() {
    while (true) {
        const { done, value } = await reader.read();
        if (done) break;
        
        console.log('Processing user event:', value);
        // Process event
    }
}

// Wildcard handler for analytics
emitter.onAny((event) => {
    analytics.track(event.type, {
        timestamp: event.timestamp,
        source: event.source,
        data: event.data
    });
});
```

### Cross-Engine Event Compatibility

```rust
// Cross-engine event system
pub struct CrossEngineEventBus {
    lua_emitters: HashMap<String, LuaEventEmitter>,
    js_emitters: HashMap<String, JSEventEmitter>,
    native_emitter: Arc<NativeEventBus>,
    event_translator: EventTranslator,
    bridge_channels: HashMap<String, BridgeChannel>,
}

pub struct EventTranslator;

impl EventTranslator {
    pub fn to_lua_event(&self, event: &Event) -> Result<mlua::Table> {
        let lua = mlua::Lua::new();
        let table = lua.create_table()?;
        
        table.set("type", event.event_type.clone())?;
        table.set("timestamp", event.timestamp.timestamp_millis())?;
        table.set("source", event.source.clone())?;
        
        // Convert JSON data to Lua table
        let data_table = self.json_to_lua_table(&lua, &event.data)?;
        table.set("data", data_table)?;
        
        Ok(table)
    }
    
    pub fn to_js_event(&self, event: &Event) -> Result<serde_json::Value> {
        Ok(json!({
            "type": event.event_type,
            "data": event.data,
            "timestamp": event.timestamp.timestamp_millis(),
            "source": event.source,
            "id": event.sequence.to_string()
        }))
    }
    
    pub fn from_lua_event(&self, lua_table: mlua::Table) -> Result<Event> {
        let event_type = lua_table.get::<_, String>("type")?;
        let timestamp_ms = lua_table.get::<_, i64>("timestamp")?;
        let source = lua_table.get::<_, String>("source").unwrap_or_else(|_| "lua".to_string());
        
        let data_table = lua_table.get::<_, mlua::Table>("data")?;
        let data = self.lua_table_to_json(&data_table)?;
        
        Ok(Event {
            event_type,
            data,
            timestamp: DateTime::from_timestamp_millis(timestamp_ms).unwrap_or_else(Utc::now),
            sequence: 0, // Will be assigned by event bus
            source,
        })
    }
}

impl CrossEngineEventBus {
    pub async fn emit_cross_engine(&self, event: Event) -> Result<CrossEngineEmissionResult> {
        let mut results = HashMap::new();
        
        // Emit to native event bus
        let native_result = self.native_emitter.emit_event(event.clone()).await?;
        results.insert("native".to_string(), native_result);
        
        // Emit to Lua emitters
        for (name, lua_emitter) in &self.lua_emitters {
            let lua_event = self.event_translator.to_lua_event(&event)?;
            let result = lua_emitter.emit(lua_event).await?;
            results.insert(format!("lua_{}", name), result);
        }
        
        // Emit to JavaScript emitters
        for (name, js_emitter) in &self.js_emitters {
            let js_event = self.event_translator.to_js_event(&event)?;
            let result = js_emitter.emit(js_event).await?;
            results.insert(format!("js_{}", name), result);
        }
        
        Ok(CrossEngineEmissionResult {
            original_event: event,
            engine_results: results,
            total_subscribers: self.count_total_subscribers(),
        })
    }
    
    pub async fn subscribe_cross_engine(&mut self, event_type: &str, subscriber: CrossEngineSubscriber) -> Result<()> {
        match subscriber {
            CrossEngineSubscriber::Lua(lua_handler) => {
                // Register Lua subscriber
                let emitter_name = format!("cross_engine_{}", Uuid::new_v4());
                let mut lua_emitter = LuaEventEmitter::new();
                lua_emitter.on(event_type, lua_handler)?;
                self.lua_emitters.insert(emitter_name, lua_emitter);
            },
            CrossEngineSubscriber::JavaScript(js_handler) => {
                // Register JavaScript subscriber
                let emitter_name = format!("cross_engine_{}", Uuid::new_v4());
                let mut js_emitter = JSEventEmitter::new();
                js_emitter.on(event_type, js_handler)?;
                self.js_emitters.insert(emitter_name, js_emitter);
            },
            CrossEngineSubscriber::Native(native_handler) => {
                // Register native subscriber
                self.native_emitter.subscribe(event_type, native_handler).await?;
            }
        }
        
        Ok(())
    }
}
```

This comprehensive cross-engine compatibility analysis ensures consistent behavior across all supported scripting runtimes while leveraging each engine's unique strengths.