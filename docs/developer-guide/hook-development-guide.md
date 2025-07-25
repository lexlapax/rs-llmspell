# Hook Development Guide

## Introduction

This guide explains how to create custom hooks in Rust for rs-llmspell. Custom hooks provide deeper integration and better performance than script-based hooks, making them ideal for production use cases.

## Hook Trait

All hooks must implement the `Hook` trait:

```rust
use async_trait::async_trait;
use llmspell_core::hooks::{Hook, HookContext, HookResult};

#[async_trait]
pub trait Hook: Send + Sync {
    /// Unique identifier for this hook
    fn id(&self) -> &str;
    
    /// Hook metadata
    fn metadata(&self) -> HookMetadata {
        HookMetadata::default()
    }
    
    /// Execute the hook
    async fn execute(&self, context: &mut HookContext) -> anyhow::Result<HookResult>;
    
    /// Optional: Called when hook is registered
    async fn on_register(&self) -> anyhow::Result<()> {
        Ok(())
    }
    
    /// Optional: Called when hook is unregistered
    async fn on_unregister(&self) -> anyhow::Result<()> {
        Ok(())
    }
}
```

## Basic Hook Example

### Simple Logging Hook

```rust
use async_trait::async_trait;
use llmspell_core::hooks::{Hook, HookContext, HookResult};
use tracing::{info, instrument};

pub struct LoggingHook {
    id: String,
    log_inputs: bool,
    log_outputs: bool,
}

impl LoggingHook {
    pub fn new(id: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            log_inputs: true,
            log_outputs: true,
        }
    }
}

#[async_trait]
impl Hook for LoggingHook {
    fn id(&self) -> &str {
        &self.id
    }
    
    #[instrument(skip(self, context))]
    async fn execute(&self, context: &mut HookContext) -> anyhow::Result<HookResult> {
        info!(
            hook_point = %context.hook_point,
            component = %context.component_id.name,
            "Hook executing"
        );
        
        if self.log_inputs {
            if let Some(input) = context.data.get("input") {
                info!(input = ?input, "Input data");
            }
        }
        
        if self.log_outputs {
            if let Some(output) = context.data.get("output") {
                info!(output = ?output, "Output data");
            }
        }
        
        Ok(HookResult::Continue)
    }
}
```

## Advanced Hook Examples

### 1. Security Validation Hook

```rust
use regex::Regex;
use once_cell::sync::Lazy;

static SQL_INJECTION_PATTERNS: Lazy<Vec<Regex>> = Lazy::new(|| {
    vec![
        Regex::new(r"(?i)\b(union|select|insert|update|delete|drop)\b.*\b(from|into|where)\b").unwrap(),
        Regex::new(r"(?i)['\"](\s)*(or|and)(\s)*['\"]?(\s)*=\s*['\"]?").unwrap(),
        Regex::new(r"(?i);\s*(drop|delete|truncate|alter|create|insert)").unwrap(),
    ]
});

pub struct SecurityValidationHook {
    id: String,
    strict_mode: bool,
    max_input_length: usize,
    blocked_patterns: Vec<Regex>,
}

impl SecurityValidationHook {
    pub fn new(config: SecurityConfig) -> Self {
        Self {
            id: "security-validator".to_string(),
            strict_mode: config.strict_mode,
            max_input_length: config.max_input_length,
            blocked_patterns: config.compile_patterns(),
        }
    }
    
    fn validate_input(&self, input: &str) -> Result<(), SecurityViolation> {
        // Length check
        if input.len() > self.max_input_length {
            return Err(SecurityViolation::InputTooLong {
                max: self.max_input_length,
                actual: input.len(),
            });
        }
        
        // SQL injection check
        for pattern in &*SQL_INJECTION_PATTERNS {
            if pattern.is_match(input) {
                return Err(SecurityViolation::SqlInjection);
            }
        }
        
        // Custom pattern check
        for pattern in &self.blocked_patterns {
            if pattern.is_match(input) {
                return Err(SecurityViolation::BlockedPattern {
                    pattern: pattern.as_str().to_string(),
                });
            }
        }
        
        Ok(())
    }
}

#[async_trait]
impl Hook for SecurityValidationHook {
    fn id(&self) -> &str {
        &self.id
    }
    
    async fn execute(&self, context: &mut HookContext) -> anyhow::Result<HookResult> {
        // Extract input text
        let input_text = context
            .data
            .get("input")
            .and_then(|v| v.get("text"))
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("No input text found"))?;
        
        // Validate
        match self.validate_input(input_text) {
            Ok(()) => Ok(HookResult::Continue),
            Err(violation) => {
                // Log security event
                tracing::warn!(
                    violation = ?violation,
                    component = %context.component_id.name,
                    "Security violation detected"
                );
                
                // Emit security event
                context.emit_event("security.violation", json!({
                    "violation_type": violation.type_name(),
                    "component": context.component_id.name,
                    "details": violation.details()
                }));
                
                if self.strict_mode {
                    Ok(HookResult::Cancel {
                        reason: format!("Security violation: {}", violation),
                    })
                } else {
                    // Sanitize input
                    let sanitized = self.sanitize_input(input_text);
                    Ok(HookResult::Modified {
                        data: json!({
                            "input": {
                                "text": sanitized
                            }
                        }),
                    })
                }
            }
        }
    }
}
```

### 2. Caching Hook with Redis

```rust
use redis::aio::ConnectionManager;
use serde::{Serialize, Deserialize};
use sha2::{Sha256, Digest};

#[derive(Clone)]
pub struct RedisCachingHook {
    id: String,
    redis: ConnectionManager,
    default_ttl: Duration,
    key_prefix: String,
}

impl RedisCachingHook {
    pub async fn new(redis_url: &str) -> anyhow::Result<Self> {
        let client = redis::Client::open(redis_url)?;
        let redis = ConnectionManager::new(client).await?;
        
        Ok(Self {
            id: "redis-cache".to_string(),
            redis,
            default_ttl: Duration::from_secs(300),
            key_prefix: "llmspell:cache:".to_string(),
        })
    }
    
    fn generate_cache_key(&self, context: &HookContext) -> String {
        let mut hasher = Sha256::new();
        
        // Include component info
        hasher.update(context.component_id.name.as_bytes());
        hasher.update(context.component_id.component_type.as_bytes());
        
        // Include input data
        if let Some(input) = context.data.get("input") {
            hasher.update(serde_json::to_vec(input).unwrap_or_default());
        }
        
        let hash = hasher.finalize();
        format!("{}{:x}", self.key_prefix, hash)
    }
}

#[async_trait]
impl Hook for RedisCachingHook {
    fn id(&self) -> &str {
        &self.id
    }
    
    async fn execute(&self, context: &mut HookContext) -> anyhow::Result<HookResult> {
        let cache_key = self.generate_cache_key(context);
        
        // Check cache on Before hooks
        if context.hook_point.is_before() {
            let cached: Option<String> = redis::cmd("GET")
                .arg(&cache_key)
                .query_async(&mut self.redis.clone())
                .await?;
            
            if let Some(cached_json) = cached {
                let cached_value: Value = serde_json::from_str(&cached_json)?;
                
                info!(key = %cache_key, "Cache hit");
                context.metrics.cache_hits.inc();
                
                return Ok(HookResult::Cache {
                    ttl: self.default_ttl,
                    result: cached_value,
                });
            }
            
            context.metrics.cache_misses.inc();
            Ok(HookResult::Continue)
        }
        // Store in cache on After hooks
        else if context.hook_point.is_after() {
            if let Some(output) = context.data.get("output") {
                let json = serde_json::to_string(output)?;
                
                redis::cmd("SETEX")
                    .arg(&cache_key)
                    .arg(self.default_ttl.as_secs())
                    .arg(json)
                    .query_async::<_, ()>(&mut self.redis.clone())
                    .await?;
                
                info!(key = %cache_key, ttl = ?self.default_ttl, "Cached result");
            }
            
            Ok(HookResult::Continue)
        } else {
            Ok(HookResult::Continue)
        }
    }
}
```

### 3. Rate Limiting Hook

```rust
use std::sync::Arc;
use dashmap::DashMap;
use tokio::sync::Semaphore;

pub struct RateLimitingHook {
    id: String,
    limiters: Arc<DashMap<String, Arc<RateLimiter>>>,
    default_config: RateLimitConfig,
}

struct RateLimiter {
    semaphore: Arc<Semaphore>,
    window: Duration,
    last_reset: RwLock<Instant>,
}

#[derive(Clone)]
struct RateLimitConfig {
    requests_per_window: usize,
    window_duration: Duration,
    by_component: bool,
    by_user: bool,
}

impl RateLimitingHook {
    pub fn new(config: RateLimitConfig) -> Self {
        Self {
            id: "rate-limiter".to_string(),
            limiters: Arc::new(DashMap::new()),
            default_config: config,
        }
    }
    
    fn get_limiter_key(&self, context: &HookContext) -> String {
        let mut parts = Vec::new();
        
        if self.default_config.by_component {
            parts.push(context.component_id.name.clone());
        }
        
        if self.default_config.by_user {
            if let Some(user_id) = context.metadata.get("user_id") {
                parts.push(user_id.as_str().unwrap_or("unknown").to_string());
            }
        }
        
        if parts.is_empty() {
            "global".to_string()
        } else {
            parts.join(":")
        }
    }
    
    async fn get_or_create_limiter(&self, key: String) -> Arc<RateLimiter> {
        self.limiters
            .entry(key)
            .or_insert_with(|| {
                Arc::new(RateLimiter {
                    semaphore: Arc::new(Semaphore::new(
                        self.default_config.requests_per_window
                    )),
                    window: self.default_config.window_duration,
                    last_reset: RwLock::new(Instant::now()),
                })
            })
            .clone()
    }
}

#[async_trait]
impl Hook for RateLimitingHook {
    fn id(&self) -> &str {
        &self.id
    }
    
    async fn execute(&self, context: &mut HookContext) -> anyhow::Result<HookResult> {
        let key = self.get_limiter_key(context);
        let limiter = self.get_or_create_limiter(key.clone()).await;
        
        // Check if window needs reset
        {
            let mut last_reset = limiter.last_reset.write().await;
            if last_reset.elapsed() > limiter.window {
                *last_reset = Instant::now();
                // Reset semaphore by creating new one
                drop(limiter.semaphore);
                let new_semaphore = Arc::new(Semaphore::new(
                    self.default_config.requests_per_window
                ));
                // Note: In real implementation, use atomic swap
            }
        }
        
        // Try to acquire permit
        match limiter.semaphore.try_acquire() {
            Ok(_permit) => {
                // Permit is held until dropped
                context.state.insert("rate_limit_permit", "active");
                Ok(HookResult::Continue)
            }
            Err(_) => {
                warn!(
                    key = %key,
                    component = %context.component_id.name,
                    "Rate limit exceeded"
                );
                
                context.emit_event("rate_limit.exceeded", json!({
                    "limiter_key": key,
                    "component": context.component_id.name,
                }));
                
                Ok(HookResult::Cancel {
                    reason: "Rate limit exceeded".to_string(),
                })
            }
        }
    }
}
```

### 4. Cost Tracking Hook

```rust
use std::sync::atomic::{AtomicU64, Ordering};

#[derive(Clone)]
pub struct CostTrackingHook {
    id: String,
    costs: Arc<DashMap<String, Arc<CostTracker>>>,
    model_rates: HashMap<String, ModelRate>,
    alert_threshold: f64,
    webhook_url: Option<String>,
}

struct CostTracker {
    total_input_tokens: AtomicU64,
    total_output_tokens: AtomicU64,
    total_cost_cents: AtomicU64,
}

struct ModelRate {
    input_per_1k: f64,  // Cost per 1K input tokens
    output_per_1k: f64, // Cost per 1K output tokens
}

impl CostTrackingHook {
    pub fn new(config: CostConfig) -> Self {
        let mut model_rates = HashMap::new();
        model_rates.insert("gpt-4".to_string(), ModelRate {
            input_per_1k: 0.03,
            output_per_1k: 0.06,
        });
        model_rates.insert("gpt-3.5-turbo".to_string(), ModelRate {
            input_per_1k: 0.001,
            output_per_1k: 0.002,
        });
        
        Self {
            id: "cost-tracker".to_string(),
            costs: Arc::new(DashMap::new()),
            model_rates,
            alert_threshold: config.alert_threshold,
            webhook_url: config.webhook_url,
        }
    }
    
    async fn send_alert(&self, message: String, details: Value) {
        if let Some(webhook_url) = &self.webhook_url {
            let client = reqwest::Client::new();
            let _ = client.post(webhook_url)
                .json(&json!({
                    "message": message,
                    "details": details,
                    "timestamp": Utc::now(),
                }))
                .send()
                .await;
        }
    }
}

#[async_trait]
impl Hook for CostTrackingHook {
    fn id(&self) -> &str {
        &self.id
    }
    
    async fn execute(&self, context: &mut HookContext) -> anyhow::Result<HookResult> {
        // Only track on AfterAgentExecution
        if context.hook_point != HookPoint::AfterAgentExecution {
            return Ok(HookResult::Continue);
        }
        
        // Extract token usage
        let input_tokens = context.data
            .get("tokens_used")
            .and_then(|v| v.get("input"))
            .and_then(|v| v.as_u64())
            .unwrap_or(0);
        
        let output_tokens = context.data
            .get("tokens_used")
            .and_then(|v| v.get("output"))
            .and_then(|v| v.as_u64())
            .unwrap_or(0);
        
        let model = context.data
            .get("model")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown");
        
        // Calculate cost
        if let Some(rate) = self.model_rates.get(model) {
            let input_cost = (input_tokens as f64 / 1000.0) * rate.input_per_1k;
            let output_cost = (output_tokens as f64 / 1000.0) * rate.output_per_1k;
            let total_cost = input_cost + output_cost;
            let cost_cents = (total_cost * 100.0) as u64;
            
            // Update tracker
            let tracker = self.costs
                .entry(context.component_id.name.clone())
                .or_insert_with(|| Arc::new(CostTracker {
                    total_input_tokens: AtomicU64::new(0),
                    total_output_tokens: AtomicU64::new(0),
                    total_cost_cents: AtomicU64::new(0),
                }))
                .clone();
            
            tracker.total_input_tokens.fetch_add(input_tokens, Ordering::Relaxed);
            tracker.total_output_tokens.fetch_add(output_tokens, Ordering::Relaxed);
            let new_total = tracker.total_cost_cents.fetch_add(cost_cents, Ordering::Relaxed) + cost_cents;
            
            // Check threshold
            let total_dollars = new_total as f64 / 100.0;
            if total_dollars > self.alert_threshold {
                self.send_alert(
                    format!("Cost threshold exceeded: ${:.2}", total_dollars),
                    json!({
                        "component": context.component_id.name,
                        "total_cost": total_dollars,
                        "threshold": self.alert_threshold,
                        "model": model,
                    })
                ).await;
            }
            
            // Add cost to context for other hooks
            context.data.insert("cost", json!({
                "session_cost": total_cost,
                "total_cost": total_dollars,
                "model": model,
            }));
        }
        
        Ok(HookResult::Continue)
    }
}
```

## Hook Registration

### Registering in Code

```rust
use llmspell_core::hooks::{HookRegistry, Priority};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let registry = HookRegistry::new();
    
    // Register custom hooks
    registry.register(
        HookPoint::BeforeAgentExecution,
        Arc::new(SecurityValidationHook::new(SecurityConfig::strict())),
        Priority::Highest,
    )?;
    
    registry.register(
        HookPoint::BeforeToolExecution,
        Arc::new(RateLimitingHook::new(RateLimitConfig {
            requests_per_window: 100,
            window_duration: Duration::from_secs(60),
            by_component: true,
            by_user: true,
        })),
        Priority::High,
    )?;
    
    registry.register_multiple(vec![
        (HookPoint::BeforeAgentExecution, Arc::new(cache_hook), Priority::Normal),
        (HookPoint::AfterAgentExecution, Arc::new(cache_hook), Priority::Normal),
    ])?;
    
    Ok(())
}
```

### Plugin System

```rust
// Define plugin trait
pub trait HookPlugin: Send + Sync {
    fn name(&self) -> &str;
    fn version(&self) -> &str;
    fn hooks(&self) -> Vec<(HookPoint, Arc<dyn Hook>, Priority)>;
}

// Example plugin
pub struct SecurityPlugin;

impl HookPlugin for SecurityPlugin {
    fn name(&self) -> &str {
        "security-plugin"
    }
    
    fn version(&self) -> &str {
        "1.0.0"
    }
    
    fn hooks(&self) -> Vec<(HookPoint, Arc<dyn Hook>, Priority)> {
        vec![
            (
                HookPoint::BeforeAgentExecution,
                Arc::new(SecurityValidationHook::new(SecurityConfig::strict())),
                Priority::Highest,
            ),
            (
                HookPoint::BeforeToolExecution,
                Arc::new(InputSanitizerHook::new()),
                Priority::High,
            ),
        ]
    }
}

// Load plugins
let plugin = SecurityPlugin;
for (point, hook, priority) in plugin.hooks() {
    registry.register(point, hook, priority)?;
}
```

## Testing Hooks

### Unit Testing

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use llmspell_core::testing::{MockContext, assert_hook_result};
    
    #[tokio::test]
    async fn test_security_hook_blocks_sql_injection() {
        let hook = SecurityValidationHook::new(SecurityConfig::strict());
        let mut context = MockContext::new()
            .with_input("SELECT * FROM users WHERE id = '1' OR '1'='1'")
            .build();
        
        let result = hook.execute(&mut context).await.unwrap();
        
        assert_hook_result!(
            result,
            HookResult::Cancel { reason } => {
                assert!(reason.contains("Security violation"));
            }
        );
    }
    
    #[tokio::test]
    async fn test_cache_hook_returns_cached_value() {
        let cache_hook = MockCacheHook::new();
        cache_hook.set("test_key", json!({"cached": true}));
        
        let mut context = MockContext::new()
            .with_component("test-agent", "agent")
            .with_input("test input")
            .build();
        
        let result = cache_hook.execute(&mut context).await.unwrap();
        
        match result {
            HookResult::Cache { result, .. } => {
                assert_eq!(result["cached"], true);
            }
            _ => panic!("Expected cache result"),
        }
    }
}
```

### Integration Testing

```rust
#[tokio::test]
async fn test_hook_integration() {
    // Create test environment
    let registry = HookRegistry::new();
    let events = EventBus::new();
    
    // Register hooks
    registry.register(
        HookPoint::BeforeAgentExecution,
        Arc::new(LoggingHook::new("test-logger")),
        Priority::Low,
    )?;
    
    // Create agent with hooks
    let agent = Agent::builder()
        .name("test-agent")
        .hooks(registry)
        .events(events)
        .build()?;
    
    // Execute and verify hooks were called
    let result = agent.execute(AgentInput::new("test")).await?;
    
    // Verify through events or metrics
    assert!(events.query("hook.executed").await.len() > 0);
}
```

## Performance Considerations

### 1. Avoid Blocking Operations

```rust
// ❌ Bad: Blocking I/O in hook
async fn execute(&self, context: &mut HookContext) -> anyhow::Result<HookResult> {
    let data = std::fs::read_to_string("config.json")?; // Blocks thread
    Ok(HookResult::Continue)
}

// ✅ Good: Use async I/O
async fn execute(&self, context: &mut HookContext) -> anyhow::Result<HookResult> {
    let data = tokio::fs::read_to_string("config.json").await?;
    Ok(HookResult::Continue)
}
```

### 2. Use Efficient Data Structures

```rust
// ✅ Use Arc for shared immutable data
pub struct OptimizedHook {
    config: Arc<Config>,  // Shared across clones
    cache: Arc<DashMap<String, Value>>,  // Concurrent HashMap
}

// ✅ Use atomics for counters
pub struct MetricsHook {
    request_count: Arc<AtomicU64>,
    error_count: Arc<AtomicU64>,
}
```

### 3. Minimize Allocations

```rust
// ✅ Reuse buffers
pub struct EfficientHook {
    buffer_pool: Arc<ArrayQueue<Vec<u8>>>,
}

impl EfficientHook {
    async fn process(&self, data: &[u8]) -> Vec<u8> {
        let mut buffer = self.buffer_pool
            .pop()
            .unwrap_or_else(|| Vec::with_capacity(1024));
        
        buffer.clear();
        buffer.extend_from_slice(data);
        
        // Process buffer...
        
        // Return to pool
        if buffer.capacity() <= 4096 {
            let _ = self.buffer_pool.push(buffer);
        }
        
        result
    }
}
```

## Deployment

### As a Crate

```toml
# Cargo.toml
[package]
name = "my-llmspell-hooks"
version = "0.1.0"

[dependencies]
llmspell-core = "0.1"
async-trait = "0.1"
tokio = { version = "1", features = ["full"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"

[lib]
name = "my_hooks"
crate-type = ["cdylib", "rlib"]
```

### Dynamic Loading

```rust
use libloading::{Library, Symbol};

pub fn load_hook_plugin(path: &Path) -> anyhow::Result<Arc<dyn Hook>> {
    unsafe {
        let lib = Library::new(path)?;
        let constructor: Symbol<fn() -> Arc<dyn Hook>> = 
            lib.get(b"create_hook")?;
        Ok(constructor())
    }
}

// In plugin
#[no_mangle]
pub extern "C" fn create_hook() -> Arc<dyn Hook> {
    Arc::new(MyCustomHook::new())
}
```

## Best Practices

1. **Keep Hooks Fast**: Target <10ms execution time
2. **Handle Errors Gracefully**: Don't panic in hooks
3. **Use Structured Logging**: Include context in logs
4. **Implement Metrics**: Track hook performance
5. **Test Thoroughly**: Unit and integration tests
6. **Document Behavior**: Clear documentation for users
7. **Version Compatibility**: Handle version mismatches
8. **Resource Cleanup**: Implement proper cleanup in `on_unregister`

## Common Patterns

### Decorator Pattern

```rust
pub struct TimingDecorator<H: Hook> {
    inner: H,
    metrics: Arc<Metrics>,
}

impl<H: Hook> Hook for TimingDecorator<H> {
    async fn execute(&self, context: &mut HookContext) -> anyhow::Result<HookResult> {
        let start = Instant::now();
        let result = self.inner.execute(context).await;
        let duration = start.elapsed();
        
        self.metrics.record_duration(self.inner.id(), duration);
        
        if duration > Duration::from_millis(100) {
            warn!(hook_id = %self.inner.id(), ?duration, "Slow hook execution");
        }
        
        result
    }
}
```

### Chain of Responsibility

```rust
pub struct HookChain {
    hooks: Vec<Arc<dyn Hook>>,
}

impl Hook for HookChain {
    async fn execute(&self, context: &mut HookContext) -> anyhow::Result<HookResult> {
        for hook in &self.hooks {
            match hook.execute(context).await? {
                HookResult::Continue => continue,
                result => return Ok(result),
            }
        }
        Ok(HookResult::Continue)
    }
}
```

## Next Steps

- **[Hook Architecture](../technical/hook-event-architecture.md)**: Deep dive into internals
- **[Built-in Hooks](../user-guide/builtin-hooks-reference.md)**: Available hooks
- **[Hook Patterns](../user-guide/hook-patterns.md)**: Common patterns
- **[Examples](https://github.com/anthropics/llmspell/tree/main/examples/hooks)**: Working examples

## Summary

- Implement the `Hook` trait for custom hooks
- Use async/await for non-blocking operations
- Leverage built-in types like `HookResult` and `HookContext`
- Test thoroughly with unit and integration tests
- Monitor performance with CircuitBreaker protection
- Deploy as crates or dynamic libraries
