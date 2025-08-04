// ABOUTME: RateLimitHook implementation for API quota management and rate limiting
// ABOUTME: Provides configurable rate limiting with token bucket algorithm and per-key quotas

use crate::context::HookContext;
use crate::rate_limiter::{RateLimiter, TokenBucketConfig};
use crate::result::HookResult;
use crate::traits::{Hook, MetricHook, ReplayableHook};
use crate::types::{HookMetadata, Language, Priority};
use anyhow::Result;
use async_trait::async_trait;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

/// Rate limiting strategy
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub enum RateLimitStrategy {
    /// Global rate limit (all requests share same bucket)
    #[default]
    Global,
    /// Per-component rate limit
    PerComponent,
    /// Per-user rate limit (requires user_id in context)
    PerUser,
    /// Per-key rate limit (custom key generation)
    PerKey(String),
    /// Custom strategy
    Custom,
}

/// Rate limit configuration
#[derive(Debug, Clone)]
pub struct RateLimitConfig {
    /// Rate limiting strategy
    pub strategy: RateLimitStrategy,
    /// Token bucket configuration
    pub bucket_config: TokenBucketConfig,
    /// Whether to include burst capacity
    pub allow_burst: bool,
    /// Custom rate limits per key
    pub custom_limits: HashMap<String, TokenBucketConfig>,
    /// Default action when rate limited
    pub rate_limited_action: RateLimitAction,
    /// Whether to add rate limit headers
    pub add_headers: bool,
    /// Header prefix for rate limit headers
    pub header_prefix: String,
    /// Grace period for near-limit warnings
    pub warning_threshold: f64,
    /// Whether to emit events on rate limiting
    pub emit_events: bool,
}

/// Action to take when rate limited
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RateLimitAction {
    /// Cancel the operation
    Cancel,
    /// Delay the operation
    Delay(Duration),
    /// Redirect to alternative endpoint
    Redirect(String),
    /// Return cached result if available
    ReturnCached,
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            strategy: RateLimitStrategy::default(),
            bucket_config: TokenBucketConfig::default(),
            allow_burst: true,
            custom_limits: HashMap::new(),
            rate_limited_action: RateLimitAction::Cancel,
            add_headers: true,
            header_prefix: "X-RateLimit-".to_string(),
            warning_threshold: 0.8, // Warn at 80% capacity
            emit_events: true,
        }
    }
}

/// Rate limiting metrics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RateLimitMetrics {
    pub total_requests: u64,
    pub allowed_requests: u64,
    pub rate_limited_requests: u64,
    pub delayed_requests: u64,
    pub warning_issued: u64,
    pub tokens_consumed: u64,
    pub tokens_refilled: u64,
    pub burst_requests: u64,
    pub custom_key_hits: HashMap<String, u64>,
}

impl RateLimitMetrics {
    pub fn allowed_ratio(&self) -> f64 {
        if self.total_requests == 0 {
            1.0
        } else {
            self.allowed_requests as f64 / self.total_requests as f64
        }
    }

    pub fn rate_limited_ratio(&self) -> f64 {
        if self.total_requests == 0 {
            0.0
        } else {
            self.rate_limited_requests as f64 / self.total_requests as f64
        }
    }
}

/// Built-in rate limiting hook for API quota management
pub struct RateLimitHook {
    rate_limiter: Arc<RateLimiter>,
    config: RateLimitConfig,
    custom_limiters: Arc<std::sync::RwLock<HashMap<String, Arc<RateLimiter>>>>,
    metrics: Arc<std::sync::RwLock<RateLimitMetrics>>,
    metadata: HookMetadata,
}

impl RateLimitHook {
    /// Create a new rate limit hook with default configuration
    pub fn new() -> Self {
        let config = RateLimitConfig::default();
        Self {
            rate_limiter: Arc::new(RateLimiter::new(config.bucket_config.clone())),
            config,
            custom_limiters: Arc::new(std::sync::RwLock::new(HashMap::new())),
            metrics: Arc::new(std::sync::RwLock::new(RateLimitMetrics::default())),
            metadata: HookMetadata {
                name: "RateLimitHook".to_string(),
                description: Some("Built-in hook for API quota management".to_string()),
                priority: Priority::HIGHEST, // Run very early to prevent unnecessary work
                language: Language::Native,
                tags: vec![
                    "builtin".to_string(),
                    "rate-limit".to_string(),
                    "quota".to_string(),
                ],
                version: "1.0.0".to_string(),
            },
        }
    }

    /// Create a new rate limit hook with custom configuration
    pub fn with_config(config: RateLimitConfig) -> Self {
        Self {
            rate_limiter: Arc::new(RateLimiter::new(config.bucket_config.clone())),
            config,
            custom_limiters: Arc::new(std::sync::RwLock::new(HashMap::new())),
            metrics: Arc::new(std::sync::RwLock::new(RateLimitMetrics::default())),
            metadata: HookMetadata {
                name: "RateLimitHook".to_string(),
                description: Some("Built-in hook for API quota management".to_string()),
                priority: Priority::HIGHEST,
                language: Language::Native,
                tags: vec![
                    "builtin".to_string(),
                    "rate-limit".to_string(),
                    "quota".to_string(),
                ],
                version: "1.0.0".to_string(),
            },
        }
    }

    /// Configure rate limiting strategy
    pub fn with_strategy(mut self, strategy: RateLimitStrategy) -> Self {
        self.config.strategy = strategy;
        self
    }

    /// Set token bucket configuration
    pub fn with_bucket_config(mut self, bucket_config: TokenBucketConfig) -> Self {
        self.config.bucket_config = bucket_config.clone();
        self.rate_limiter = Arc::new(RateLimiter::new(bucket_config));
        self
    }

    /// Set rate per second
    pub fn with_rate_per_second(mut self, rate: f64) -> Self {
        self.config.bucket_config.capacity = rate as usize;
        self.config.bucket_config.refill_rate = rate;
        self.config.bucket_config.refill_interval = Duration::from_secs(1);
        self.rate_limiter = Arc::new(RateLimiter::new(self.config.bucket_config.clone()));
        self
    }

    /// Set burst capacity
    pub fn with_burst(mut self, burst_size: usize) -> Self {
        self.config.bucket_config.burst_capacity = burst_size;
        self.config.allow_burst = true;
        self.rate_limiter = Arc::new(RateLimiter::new(self.config.bucket_config.clone()));
        self
    }

    /// Add custom rate limit for specific key
    pub fn with_custom_limit(mut self, key: String, config: TokenBucketConfig) -> Self {
        self.config.custom_limits.insert(key, config);
        self
    }

    /// Set action when rate limited
    pub fn with_rate_limited_action(mut self, action: RateLimitAction) -> Self {
        self.config.rate_limited_action = action;
        self
    }

    /// Get rate limiting metrics
    pub fn metrics(&self) -> RateLimitMetrics {
        self.metrics.read().unwrap().clone()
    }

    /// Reset metrics
    pub fn reset_metrics(&self) {
        let mut metrics = self.metrics.write().unwrap();
        *metrics = RateLimitMetrics::default();
    }

    /// Generate rate limit key based on strategy
    fn generate_key(&self, context: &HookContext) -> String {
        match &self.config.strategy {
            RateLimitStrategy::Global => "global".to_string(),
            RateLimitStrategy::PerComponent => {
                format!(
                    "{:?}:{}",
                    context.component_id.component_type, context.component_id.name
                )
            }
            RateLimitStrategy::PerUser => context
                .get_metadata("user_id")
                .unwrap_or("anonymous")
                .to_string(),
            RateLimitStrategy::PerKey(key_pattern) => {
                // Simple key pattern replacement
                let mut key = key_pattern.clone();
                if key.contains("{component}") {
                    key = key.replace("{component}", &context.component_id.name);
                }
                if key.contains("{hook_point}") {
                    key = key.replace("{hook_point}", &format!("{:?}", context.point));
                }
                if key.contains("{user}") {
                    let user = context.get_metadata("user_id").unwrap_or("anonymous");
                    key = key.replace("{user}", user);
                }
                key
            }
            RateLimitStrategy::Custom => {
                // For custom strategy, use component + hook point
                format!(
                    "{:?}:{}:{:?}",
                    context.component_id.component_type, context.component_id.name, context.point
                )
            }
        }
    }

    /// Add rate limit headers to context
    fn add_rate_limit_headers(
        &self,
        context: &mut HookContext,
        key: &str,
        tokens_remaining: f64,
        rate_limiter: &RateLimiter,
    ) {
        if !self.config.add_headers {
            return;
        }

        let bucket_state = rate_limiter.get_bucket_state(key);

        // Add standard rate limit headers
        context.insert_metadata(
            format!("{}Limit", self.config.header_prefix),
            bucket_state.capacity.to_string(),
        );
        context.insert_metadata(
            format!("{}Remaining", self.config.header_prefix),
            tokens_remaining.floor().to_string(),
        );
        context.insert_metadata(
            format!("{}Reset", self.config.header_prefix),
            bucket_state.next_refill.to_rfc3339(),
        );

        // Add burst capacity if enabled
        if self.config.allow_burst {
            context.insert_metadata(
                format!("{}Burst", self.config.header_prefix),
                self.config.bucket_config.burst_capacity.to_string(),
            );
        }

        // Add retry-after header if rate limited
        if tokens_remaining <= 0.0 {
            let retry_after = bucket_state
                .next_refill
                .signed_duration_since(Utc::now())
                .num_seconds()
                .max(1);
            context.insert_metadata(
                format!("{}Retry-After", self.config.header_prefix),
                retry_after.to_string(),
            );
        }
    }

    /// Check if we should issue a warning
    fn check_warning_threshold(&self, tokens_remaining: f64, capacity: f64) -> bool {
        let usage_ratio = 1.0 - (tokens_remaining / capacity);
        usage_ratio >= self.config.warning_threshold
    }
}

impl Default for RateLimitHook {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Hook for RateLimitHook {
    async fn execute(&self, context: &mut HookContext) -> Result<HookResult> {
        let key = self.generate_key(context);

        // Update metrics without holding lock across await
        {
            let mut metrics = self.metrics.write().unwrap();
            metrics.total_requests += 1;
        }

        // Check if we have a custom limit for this key
        let rate_limiter = if let Some(custom_config) = self.config.custom_limits.get(&key) {
            // Update metrics
            {
                let mut metrics = self.metrics.write().unwrap();
                metrics
                    .custom_key_hits
                    .entry(key.clone())
                    .and_modify(|e| *e += 1)
                    .or_insert(1);
            }

            // Get or create custom limiter
            let mut custom_limiters = self.custom_limiters.write().unwrap();
            custom_limiters
                .entry(key.clone())
                .or_insert_with(|| Arc::new(RateLimiter::new(custom_config.clone())))
                .clone()
        } else {
            self.rate_limiter.clone()
        };

        // Try to acquire tokens
        let tokens_requested = 1.0; // Could be configurable per operation
        let (allowed, tokens_remaining) = rate_limiter.try_acquire(&key, tokens_requested);

        if allowed {
            // Update metrics
            {
                let mut metrics = self.metrics.write().unwrap();
                metrics.allowed_requests += 1;
                metrics.tokens_consumed += tokens_requested as u64;
            }

            // Check if we're using burst capacity
            let bucket_state = rate_limiter.get_bucket_state(&key);
            if tokens_remaining < bucket_state.capacity as f64 - bucket_state.burst_capacity as f64
            {
                let mut metrics = self.metrics.write().unwrap();
                metrics.burst_requests += 1;
            }

            // Check warning threshold
            let total_capacity = (bucket_state.capacity + bucket_state.burst_capacity) as f64;
            if self.check_warning_threshold(tokens_remaining, total_capacity) {
                let mut metrics = self.metrics.write().unwrap();
                metrics.warning_issued += 1;
                context.insert_metadata(
                    format!("{}Warning", self.config.header_prefix),
                    "Rate limit threshold approaching".to_string(),
                );
            }

            // Add headers
            self.add_rate_limit_headers(context, &key, tokens_remaining, &rate_limiter);

            // Log if debug level
            log::debug!(
                "RateLimitHook: Allowed request for key '{}', {} tokens remaining",
                key,
                tokens_remaining
            );

            Ok(HookResult::Continue)
        } else {
            // Update metrics for rate limited request
            {
                let mut metrics = self.metrics.write().unwrap();
                metrics.rate_limited_requests += 1;
            }

            // Add headers even when rate limited
            self.add_rate_limit_headers(context, &key, tokens_remaining, &rate_limiter);

            // Log rate limiting
            log::warn!(
                "RateLimitHook: Rate limited request for key '{}', action: {:?}",
                key,
                self.config.rate_limited_action
            );

            // Take configured action
            match &self.config.rate_limited_action {
                RateLimitAction::Cancel => {
                    Ok(HookResult::Cancel("Rate limit exceeded".to_string()))
                }
                RateLimitAction::Delay(duration) => {
                    // Update delayed requests metric
                    {
                        let mut metrics = self.metrics.write().unwrap();
                        metrics.delayed_requests += 1;
                    }

                    // Add delay metadata
                    context.insert_metadata(
                        "rate_limit_delay_ms".to_string(),
                        duration.as_millis().to_string(),
                    );

                    // Sleep for the configured duration
                    tokio::time::sleep(*duration).await;

                    // Try again after delay
                    let (allowed_after_delay, new_tokens) =
                        rate_limiter.try_acquire(&key, tokens_requested);
                    if allowed_after_delay {
                        {
                            let mut metrics = self.metrics.write().unwrap();
                            metrics.allowed_requests += 1;
                        }
                        self.add_rate_limit_headers(context, &key, new_tokens, &rate_limiter);
                        Ok(HookResult::Continue)
                    } else {
                        Ok(HookResult::Cancel(
                            "Rate limit exceeded after delay".to_string(),
                        ))
                    }
                }
                RateLimitAction::Redirect(endpoint) => Ok(HookResult::Redirect(endpoint.clone())),
                RateLimitAction::ReturnCached => {
                    // Signal that cached result should be used
                    context.insert_metadata("use_cached_result".to_string(), "true".to_string());
                    Ok(HookResult::Modified(serde_json::json!({
                        "rate_limited": true,
                        "action": "return_cached"
                    })))
                }
            }
        }
    }

    fn metadata(&self) -> HookMetadata {
        self.metadata.clone()
    }

    fn should_execute(&self, _context: &HookContext) -> bool {
        // Always execute rate limiting
        true
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

#[async_trait]
impl MetricHook for RateLimitHook {
    async fn record_pre_execution(&self, context: &HookContext) -> Result<()> {
        log::trace!(
            "RateLimitHook: Pre-execution for hook point {:?}",
            context.point
        );
        Ok(())
    }

    async fn record_post_execution(
        &self,
        context: &HookContext,
        _result: &HookResult,
        _duration: Duration,
    ) -> Result<()> {
        // Record token refill if needed
        let key = self.generate_key(context);
        let bucket_state = self.rate_limiter.get_bucket_state(&key);

        if bucket_state.last_refill_amount > 0.0 {
            let mut metrics = self.metrics.write().unwrap();
            metrics.tokens_refilled += bucket_state.last_refill_amount as u64;
        }

        Ok(())
    }
}

#[async_trait]
impl ReplayableHook for RateLimitHook {
    fn is_replayable(&self) -> bool {
        true
    }

    fn serialize_context(&self, ctx: &HookContext) -> Result<Vec<u8>> {
        // Create a serializable version of the context with rate limit config
        let mut context_data = ctx.data.clone();

        // Add rate limit configuration for replay context
        context_data.insert(
            "_rate_limit_config".to_string(),
            serde_json::json!({
                "strategy": serde_json::to_value(&self.config.strategy)?,
                "rate_limited_action": serde_json::to_value(&self.config.rate_limited_action)?,
                "warning_threshold": self.config.warning_threshold,
                "header_prefix": self.config.header_prefix,
                "allow_burst": self.config.allow_burst,
                "bucket_capacity": self.config.bucket_config.capacity,
                "refill_rate": self.config.bucket_config.refill_rate,
                "burst_capacity": self.config.bucket_config.burst_capacity,
            }),
        );

        // Add current metrics snapshot for debugging
        let metrics = self.metrics.read().unwrap();
        context_data.insert(
            "_rate_limit_metrics".to_string(),
            serde_json::to_value(&*metrics)?,
        );

        // Note: We don't serialize the actual rate limiter state as it's runtime-specific
        // and replays should start with fresh rate limits

        let mut replay_context = ctx.clone();
        replay_context.data = context_data;

        Ok(serde_json::to_vec(&replay_context)?)
    }

    fn deserialize_context(&self, data: &[u8]) -> Result<HookContext> {
        let mut context: HookContext = serde_json::from_slice(data)?;

        // Remove the rate limit specific data from context
        context.data.remove("_rate_limit_config");
        context.data.remove("_rate_limit_metrics");

        Ok(context)
    }

    fn replay_id(&self) -> String {
        format!("{}:{}", self.metadata.name, self.metadata.version)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{ComponentId, ComponentType, HookPoint};
    use llmspell_testing::hook_helpers::create_test_hook_context_with_component;
    use std::time::Duration;

    fn create_test_context() -> HookContext {
        create_test_hook_context_with_component(
            HookPoint::BeforeToolExecution,
            ComponentType::System,
            "test",
        )
    }
    #[tokio::test]
    async fn test_rate_limit_hook_basic() {
        let hook = RateLimitHook::new()
            .with_rate_per_second(10.0)
            .with_burst(20);

        let mut context = create_test_context();

        // First request should be allowed
        let result = hook.execute(&mut context).await.unwrap();
        assert!(matches!(result, HookResult::Continue));

        // Check headers were added
        assert!(context.get_metadata("X-RateLimit-Limit").is_some());
        assert!(context.get_metadata("X-RateLimit-Remaining").is_some());
        assert!(context.get_metadata("X-RateLimit-Reset").is_some());
    }
    #[tokio::test]
    async fn test_rate_limit_exceeded() {
        let hook = RateLimitHook::new().with_rate_per_second(1.0).with_burst(2);

        let mut context = create_test_context();

        // First three requests should be allowed (capacity=1 + burst=2)
        for _ in 0..3 {
            let result = hook.execute(&mut context).await.unwrap();
            assert!(matches!(result, HookResult::Continue));
        }

        // Fourth request should be rate limited
        let result = hook.execute(&mut context).await.unwrap();
        assert!(matches!(result, HookResult::Cancel(_)));

        // Check retry-after header
        assert!(context.get_metadata("X-RateLimit-Retry-After").is_some());
    }
    #[tokio::test]
    async fn test_rate_limit_strategies() {
        // Test per-component strategy
        let hook = RateLimitHook::new()
            .with_strategy(RateLimitStrategy::PerComponent)
            .with_rate_per_second(5.0);

        let mut context1 = create_test_context();
        let mut context2 = HookContext::new(
            HookPoint::BeforeToolExecution,
            ComponentId::new(ComponentType::Tool, "different".to_string()),
        );

        // Both contexts should be allowed (different components)
        let result1 = hook.execute(&mut context1).await.unwrap();
        let result2 = hook.execute(&mut context2).await.unwrap();

        assert!(matches!(result1, HookResult::Continue));
        assert!(matches!(result2, HookResult::Continue));
    }
    #[tokio::test]
    async fn test_rate_limit_delay_action() {
        // Use a very fast refill rate for testing
        let config = TokenBucketConfig {
            capacity: 1,
            refill_rate: 10.0,                           // 10 tokens per interval
            refill_interval: Duration::from_millis(100), // Fast refill
            burst_capacity: 0,
        };

        let hook = RateLimitHook::new()
            .with_bucket_config(config)
            .with_rate_limited_action(RateLimitAction::Delay(Duration::from_millis(150))); // Delay longer than refill

        let mut context = create_test_context();

        // First request allowed
        let result = hook.execute(&mut context).await.unwrap();
        assert!(matches!(result, HookResult::Continue));

        // Second request should be delayed but then allowed after refill
        let start = std::time::Instant::now();
        let result = hook.execute(&mut context).await.unwrap();
        let elapsed = start.elapsed();

        // Should have delayed for at least 150ms
        assert!(elapsed >= Duration::from_millis(140)); // Allow some tolerance
        assert!(matches!(result, HookResult::Continue));
    }
    #[tokio::test]
    async fn test_rate_limit_custom_limits() {
        let hook = RateLimitHook::new()
            .with_strategy(RateLimitStrategy::PerKey("api-{component}".to_string()))
            .with_rate_per_second(5.0)
            .with_custom_limit(
                "api-special".to_string(),
                TokenBucketConfig {
                    capacity: 100,
                    refill_rate: 50.0,
                    refill_interval: Duration::from_secs(1),
                    burst_capacity: 0,
                },
            );

        let mut context = HookContext::new(
            HookPoint::BeforeToolExecution,
            ComponentId::new(ComponentType::Tool, "special".to_string()),
        );

        // Should use custom limit for "api-special"
        let result = hook.execute(&mut context).await.unwrap();
        assert!(matches!(result, HookResult::Continue));

        // Check that custom limit was used
        let limit = context.get_metadata("X-RateLimit-Limit").unwrap();
        assert_eq!(limit, "100");
    }
    #[tokio::test]
    async fn test_rate_limit_warning_threshold() {
        let hook = RateLimitHook::new()
            .with_rate_per_second(1.0)
            .with_burst(10);

        let mut context = create_test_context();

        // Use up tokens until we hit warning threshold (80%)
        for _ in 0..8 {
            let _ = hook.execute(&mut context).await.unwrap();
            context = create_test_context(); // Fresh context to clear headers
        }

        // Next request should trigger warning
        let result = hook.execute(&mut context).await.unwrap();
        assert!(matches!(result, HookResult::Continue));
        assert!(context.get_metadata("X-RateLimit-Warning").is_some());
    }
    #[tokio::test]
    async fn test_rate_limit_metrics() {
        let hook = RateLimitHook::new().with_rate_per_second(1.0).with_burst(1); // Total capacity = 2

        let mut context = create_test_context();

        // Make several requests
        for _ in 0..3 {
            let _ = hook.execute(&mut context).await;
        }

        let metrics = hook.metrics();
        assert_eq!(metrics.total_requests, 3);
        assert_eq!(metrics.allowed_requests, 2);
        assert_eq!(metrics.rate_limited_requests, 1);
        assert_eq!(metrics.allowed_ratio(), 2.0 / 3.0);
    }
    #[test]
    fn test_hook_metadata() {
        let hook = RateLimitHook::new();
        let metadata = hook.metadata();

        assert_eq!(metadata.name, "RateLimitHook");
        assert!(metadata.description.is_some());
        assert_eq!(metadata.priority, Priority::HIGHEST);
        assert_eq!(metadata.language, Language::Native);
        assert!(metadata.tags.contains(&"builtin".to_string()));
        assert!(metadata.tags.contains(&"rate-limit".to_string()));
    }
    #[test]
    fn test_rate_limit_config_defaults() {
        let config = RateLimitConfig::default();
        assert!(matches!(config.strategy, RateLimitStrategy::Global));
        assert!(config.allow_burst);
        assert!(matches!(
            config.rate_limited_action,
            RateLimitAction::Cancel
        ));
        assert!(config.add_headers);
        assert_eq!(config.header_prefix, "X-RateLimit-");
        assert_eq!(config.warning_threshold, 0.8);
    }
    #[tokio::test]
    async fn test_replayable_hook_implementation() {
        let hook = RateLimitHook::new()
            .with_rate_per_second(10.0)
            .with_burst(5)
            .with_strategy(RateLimitStrategy::PerComponent);
        let component_id = ComponentId::new(ComponentType::Agent, "test-agent".to_string());
        let mut context = HookContext::new(HookPoint::BeforeAgentExecution, component_id);

        // Add test data
        context.insert_data("test_key".to_string(), serde_json::json!("test_value"));
        context.insert_metadata("user_id".to_string(), "test_user".to_string());

        // Execute to create some state
        hook.execute(&mut context).await.unwrap();

        // Test serialization
        let serialized = hook.serialize_context(&context).unwrap();
        assert!(!serialized.is_empty());

        // Test deserialization
        let deserialized = hook.deserialize_context(&serialized).unwrap();
        assert_eq!(deserialized.point, context.point);
        assert_eq!(deserialized.component_id, context.component_id);
        assert_eq!(
            deserialized.data.get("test_key"),
            context.data.get("test_key")
        );

        // Ensure rate limit specific data was removed
        assert!(deserialized.data.get("_rate_limit_config").is_none());
        assert!(deserialized.data.get("_rate_limit_metrics").is_none());

        // Test replay ID
        assert_eq!(hook.replay_id(), "RateLimitHook:1.0.0");
        assert!(hook.is_replayable());
    }
}
