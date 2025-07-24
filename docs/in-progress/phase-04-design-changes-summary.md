# Phase 4 Design Document - Required Changes Summary

## Quick Reference: What to Change in phase-04-design-doc.md

### 1. Update Hook Trait (Section 1.1)

**Current**:
```rust
pub trait Hook: Send + Sync {
    async fn execute(&self, context: &mut HookContext) -> Result<HookResult>;
}
```

**Change to**:
```rust
pub trait Hook: Send + Sync {
    async fn execute(&self, context: &mut HookContext) -> Result<HookResult>;
}

pub trait HookAdapter: Send + Sync {
    type Context;
    type Result;
    
    fn adapt_context(&self, ctx: &HookContext) -> Self::Context;
    fn adapt_result(&self, result: Self::Result) -> HookResult;
}

pub trait ReplayableHook: Hook {
    fn is_replayable(&self) -> bool;
    fn serialize_context(&self, ctx: &HookContext) -> Result<Vec<u8>>;
    fn deserialize_context(&self, data: &[u8]) -> Result<HookContext>;
}
```

### 2. Enhance HookResult Enum (Section 1.1)

**Current**:
```rust
pub enum HookResult {
    Continue,
    Modified(Value),
    Cancel(String),
    Redirect(String),
}
```

**Change to**:
```rust
pub enum HookResult {
    Continue,
    Modified(Value),
    Cancel(String),
    Redirect(String),
    Replace(serde_json::Value), // Replace entire operation result
    Retry { delay: Duration, max_attempts: u32 },
    Fork { parallel_operations: Vec<Operation> },
    Cache { key: String, ttl: Duration },
    Skipped(String), // For circuit breaker
}
```

### 3. Add HookExecutor with CircuitBreaker (New in Section 1.1)

**Add**:
```rust
pub struct HookExecutor {
    registry: Arc<HookRegistry>,
    monitor: Arc<PerformanceMonitor>,
    circuit_breaker: Arc<CircuitBreaker>,
}

pub struct CircuitBreaker {
    thresholds: HashMap<HookPoint, Duration>,
    states: DashMap<HookPoint, BreakerState>,
}

pub enum BreakerState {
    Closed,
    Open { until: Instant },
    HalfOpen,
}
```

### 4. Update EventBus (Section 1.2)

**Current**:
```rust
pub struct EventBus {
    subscribers: Arc<RwLock<HashMap<EventPattern, Vec<Arc<dyn EventHandler>>>>>,
    event_store: Option<Arc<dyn EventStore>>,
    dispatcher: Arc<EventDispatcher>,
    metrics: Arc<EventMetrics>,
}
```

**Change to**:
```rust
pub struct EventBus {
    subscribers: Arc<RwLock<HashMap<EventPattern, Vec<Arc<dyn EventHandler>>>>>,
    event_store: Option<Arc<dyn EventStore>>,
    dispatcher: Arc<EventDispatcher>,
    metrics: Arc<EventMetrics>,
    flow_controller: Arc<FlowController>,
    event_channel: mpsc::Sender<UniversalEvent>,
}

pub struct FlowController {
    max_events_per_second: usize,
    buffer_size: usize,
    overflow_strategy: OverflowStrategy,
}

pub enum OverflowStrategy {
    DropOldest,
    DropNewest,
    BackPressure,
    Sample(f64),
}
```

### 5. Add UniversalEvent (New in Section 1.2)

**Add**:
```rust
pub struct UniversalEvent {
    id: Uuid,
    timestamp: DateTime<Utc>,
    source_language: Language,
    event_type: String,
    payload: serde_json::Value, // JSON for cross-language compatibility
    sequence_number: u64,       // Preserve ordering
}

pub enum Language {
    Lua,
    JavaScript,
    Python,
    Native,
}
```

### 6. Update Script Integration Examples (Section 1.4)

**Add JavaScript example**:
```javascript
// JavaScript Promise-based hook (Phase 15 prep)
Hook.register("agent:before_execution", async (context) => {
    // Async/await pattern
    await Logger.info("Agent starting", {agent_id: context.agent_id});
    
    // Can modify context
    context.metadata.custom_field = "value";
    
    // Return control flow
    return {
        continue_execution: true,
        state_updates: {
            last_execution: Date.now()
        }
    };
});
```

### 7. Add More Built-in Hooks (Section 1.5)

**Add**:
```rust
// Caching hook
pub struct CachingHook {
    cache: Arc<dyn Cache>,
    key_generator: Box<dyn Fn(&HookContext) -> String>,
    ttl: Duration,
}

// Rate limiting hook
pub struct RateLimitHook {
    limiter: Arc<RateLimiter>,
    key_extractor: Box<dyn Fn(&HookContext) -> String>,
}

// Retry hook
pub struct RetryHook {
    max_attempts: u32,
    backoff_strategy: BackoffStrategy,
    retryable_errors: HashSet<String>,
}

// Cost tracking hook
pub struct CostTrackingHook {
    pricing_model: PricingModel,
    cost_aggregator: Arc<CostAggregator>,
}
```

### 8. Update Performance Section (Section 1.6)

**Add automatic circuit breaker**:
```rust
impl PerformanceMonitor {
    pub async fn check_and_break(&self, point: &HookPoint, duration: Duration) -> bool {
        let threshold = self.get_threshold(point);
        if duration > threshold {
            self.consecutive_slow_executions.increment(point);
            if self.consecutive_slow_executions.get(point) > 3 {
                // Open circuit breaker
                return true;
            }
        } else {
            self.consecutive_slow_executions.reset(point);
        }
        false
    }
}
```

### 9. Add Cross-Language Support (New Section)

**Add new section 2.5**:
```markdown
### 2.5 Cross-Language Hook Support

**Supporting Multiple Script Languages:**

```rust
pub struct CrossLanguageHookBridge {
    lua_adapter: LuaHookAdapter,
    js_adapter: Option<JavaScriptHookAdapter>,
    python_adapter: Option<PythonHookAdapter>,
}

impl CrossLanguageHookBridge {
    pub async fn execute_hook(
        &self,
        point: HookPoint,
        context: HookContext,
        language: Language,
    ) -> Result<HookResult> {
        match language {
            Language::Lua => self.lua_adapter.execute(point, context).await,
            Language::JavaScript => {
                self.js_adapter
                    .as_ref()
                    .ok_or_else(|| Error::LanguageNotSupported("JavaScript"))?
                    .execute(point, context)
                    .await
            }
            Language::Python => {
                self.python_adapter
                    .as_ref()
                    .ok_or_else(|| Error::LanguageNotSupported("Python"))?
                    .execute(point, context)
                    .await
            }
            Language::Native => unreachable!("Native hooks don't use bridge"),
        }
    }
}
```

### 10. Update Testing Strategy (Section 3)

**Add cross-language tests**:
```rust
#[tokio::test]
async fn test_cross_language_event_propagation() {
    let system = create_test_system().await;
    
    // Register Lua hook that emits event
    system.register_lua_hook(r#"
        Hook.register("test:trigger", function(ctx)
            Event.emit("cross_language_test", {data = "from_lua"})
            return {continue_execution = true}
        end)
    "#).await.unwrap();
    
    // Register JS handler for event (simulated)
    let received = Arc::new(Mutex::new(None));
    system.register_universal_handler("cross_language_test", {
        let received = received.clone();
        move |event| {
            *received.lock().unwrap() = Some(event.payload.clone());
        }
    }).await.unwrap();
    
    // Trigger hook
    system.execute_hook(HookPoint::Custom("test:trigger"), context).await.unwrap();
    
    // Verify event propagated
    assert_eq!(
        received.lock().unwrap().as_ref().unwrap()["data"],
        "from_lua"
    );
}
```

## Summary of Major Additions

1. **HookAdapter trait** - For language-specific adaptations
2. **ReplayableHook trait** - For Phase 5 persistence
3. **Enhanced HookResult** - More control flow options
4. **CircuitBreaker** - Automatic performance protection
5. **FlowController** - Event bus backpressure handling
6. **UniversalEvent** - Cross-language event format
7. **CrossLanguageHookBridge** - Multi-language support
8. **Additional built-in hooks** - Production-ready patterns
9. **JavaScript examples** - Prep for Phase 15
10. **Cross-language tests** - Ensure compatibility

These changes will future-proof the hook system and prevent the kind of rework we experienced with the sync/async patterns in Phase 3.