// ABOUTME: RetryHook implementation with exponential backoff for transient failure recovery
// ABOUTME: Provides configurable retry strategies with jitter support for distributed systems

use crate::context::HookContext;
use crate::result::HookResult;
use crate::traits::{Hook, MetricHook, ReplayableHook};
use crate::types::{HookMetadata, Language, Priority};
use anyhow::Result;
use async_trait::async_trait;
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use std::time::Duration;
use tracing::{debug, info, trace, warn};

/// Backoff strategy for retries
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BackoffStrategy {
    /// Fixed delay between retries
    Fixed(Duration),
    /// Linear backoff with base + (increment * attempt)
    Linear { base: Duration, increment: Duration },
    /// Exponential backoff with base * (multiplier ^ attempt)
    Exponential {
        base: Duration,
        multiplier: f64,
        max: Duration,
    },
    /// Fibonacci backoff
    Fibonacci { base: Duration, max: Duration },
}

impl Default for BackoffStrategy {
    fn default() -> Self {
        Self::Exponential {
            base: Duration::from_millis(100),
            multiplier: 2.0,
            max: Duration::from_secs(60),
        }
    }
}

/// Jitter strategy to prevent thundering herd
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub enum JitterStrategy {
    /// No jitter
    #[default]
    None,
    /// Full jitter (0 to delay)
    Full,
    /// Equal jitter (delay/2 to delay)
    Equal,
    /// Decorrelated jitter
    Decorrelated { previous: Option<Duration> },
}

/// Retry configuration
#[derive(Debug, Clone)]
pub struct RetryConfig {
    /// Maximum number of retry attempts
    pub max_attempts: u32,
    /// Backoff strategy
    pub backoff_strategy: BackoffStrategy,
    /// Jitter strategy
    pub jitter_strategy: JitterStrategy,
    /// Set of retryable error patterns
    pub retryable_errors: HashSet<String>,
    /// Set of non-retryable error patterns (takes precedence)
    pub non_retryable_errors: HashSet<String>,
    /// Whether to retry on timeout
    pub retry_on_timeout: bool,
    /// Whether to retry on rate limit
    pub retry_on_rate_limit: bool,
    /// Maximum total retry duration
    pub max_retry_duration: Option<Duration>,
    /// Hook points to apply retry logic
    pub applicable_points: HashSet<crate::types::HookPoint>,
}

impl Default for RetryConfig {
    fn default() -> Self {
        use crate::types::HookPoint;

        let mut retryable_errors = HashSet::new();
        retryable_errors.insert("connection_error".to_string());
        retryable_errors.insert("timeout".to_string());
        retryable_errors.insert("service_unavailable".to_string());
        retryable_errors.insert("internal_server_error".to_string());

        let mut applicable_points = HashSet::new();
        applicable_points.insert(HookPoint::ToolError);
        applicable_points.insert(HookPoint::AgentError);
        applicable_points.insert(HookPoint::AfterToolExecution);
        applicable_points.insert(HookPoint::AfterAgentExecution);

        Self {
            max_attempts: 3,
            backoff_strategy: BackoffStrategy::default(),
            jitter_strategy: JitterStrategy::default(),
            retryable_errors,
            non_retryable_errors: HashSet::new(),
            retry_on_timeout: true,
            retry_on_rate_limit: true,
            max_retry_duration: Some(Duration::from_secs(300)), // 5 minutes
            applicable_points,
        }
    }
}

/// Retry metrics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RetryMetrics {
    pub total_operations: u64,
    pub retry_attempts: u64,
    pub successful_retries: u64,
    pub failed_retries: u64,
    pub total_delay_ms: u64,
    pub max_attempts_reached: u64,
    pub max_duration_reached: u64,
    pub retry_reasons: HashMap<String, u64>,
}

impl RetryMetrics {
    pub fn retry_rate(&self) -> f64 {
        if self.total_operations == 0 {
            0.0
        } else {
            #[allow(clippy::cast_precision_loss)]
            let rate = self.retry_attempts as f64 / self.total_operations as f64;
            rate
        }
    }

    pub fn success_rate(&self) -> f64 {
        if self.retry_attempts == 0 {
            0.0
        } else {
            #[allow(clippy::cast_precision_loss)]
            let rate = self.successful_retries as f64 / self.retry_attempts as f64;
            rate
        }
    }
}

/// Retry state for tracking attempts
#[derive(Debug, Clone)]
struct RetryState {
    attempts: u32,
    total_delay: Duration,
    start_time: std::time::Instant,
    last_delay: Option<Duration>,
}

/// Built-in retry hook with exponential backoff
#[derive(Debug)]
pub struct RetryHook {
    config: RetryConfig,
    attempt_tracker: Arc<parking_lot::RwLock<HashMap<String, RetryState>>>,
    metrics: Arc<std::sync::RwLock<RetryMetrics>>,
    metadata: HookMetadata,
}

impl RetryHook {
    /// Create a new retry hook with default configuration
    pub fn new() -> Self {
        Self::with_config(RetryConfig::default())
    }

    /// Create a new retry hook with custom configuration
    pub fn with_config(config: RetryConfig) -> Self {
        Self {
            config,
            attempt_tracker: Arc::new(parking_lot::RwLock::new(HashMap::new())),
            metrics: Arc::new(std::sync::RwLock::new(RetryMetrics::default())),
            metadata: HookMetadata {
                name: "RetryHook".to_string(),
                description: Some("Built-in hook for automatic retry with backoff".to_string()),
                priority: Priority::HIGH, // Run early to catch errors
                language: Language::Native,
                tags: vec![
                    "builtin".to_string(),
                    "retry".to_string(),
                    "resilience".to_string(),
                ],
                version: "1.0.0".to_string(),
            },
        }
    }

    /// Set maximum attempts
    pub fn with_max_attempts(mut self, max_attempts: u32) -> Self {
        self.config.max_attempts = max_attempts;
        self
    }

    /// Set backoff strategy
    pub fn with_backoff_strategy(mut self, strategy: BackoffStrategy) -> Self {
        self.config.backoff_strategy = strategy;
        self
    }

    /// Set jitter strategy
    pub fn with_jitter_strategy(mut self, strategy: JitterStrategy) -> Self {
        self.config.jitter_strategy = strategy;
        self
    }

    /// Add retryable error pattern
    pub fn with_retryable_error(mut self, pattern: String) -> Self {
        self.config.retryable_errors.insert(pattern);
        self
    }

    /// Add non-retryable error pattern
    pub fn with_non_retryable_error(mut self, pattern: String) -> Self {
        self.config.non_retryable_errors.insert(pattern);
        self
    }

    /// Get retry metrics
    pub fn metrics(&self) -> RetryMetrics {
        self.metrics.read().unwrap().clone()
    }

    /// Reset metrics
    pub fn reset_metrics(&self) {
        let mut metrics = self.metrics.write().unwrap();
        *metrics = RetryMetrics::default();
    }

    /// Check if error is retryable
    fn is_retryable_error(&self, error: &str) -> bool {
        // Check non-retryable first (takes precedence)
        for pattern in &self.config.non_retryable_errors {
            if error.contains(pattern) {
                return false;
            }
        }

        // Check retryable patterns
        for pattern in &self.config.retryable_errors {
            if error.contains(pattern) {
                return true;
            }
        }

        // Check special cases
        if self.config.retry_on_timeout && error.contains("timeout") {
            return true;
        }

        if self.config.retry_on_rate_limit && error.contains("rate_limit") {
            return true;
        }

        false
    }

    /// Calculate backoff delay with jitter
    fn calculate_delay(&self, state: &RetryState) -> Duration {
        let base_delay = match &self.config.backoff_strategy {
            BackoffStrategy::Fixed(duration) => *duration,
            BackoffStrategy::Linear { base, increment } => *base + (*increment * state.attempts),
            BackoffStrategy::Exponential {
                base,
                multiplier,
                max,
            } => {
                #[allow(clippy::cast_possible_truncation, clippy::cast_precision_loss)]
                let base_f64 = base.as_millis() as f64;
                #[allow(clippy::cast_lossless)]
                let attempts_f64 = f64::from(state.attempts);
                let delay = base_f64 * multiplier.powf(attempts_f64);
                #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
                let delay_ms = delay.min(max.as_millis() as f64) as u64;
                Duration::from_millis(delay_ms)
            }
            BackoffStrategy::Fibonacci { base, max } => {
                let fib = self.fibonacci(state.attempts);
                #[allow(clippy::cast_possible_truncation)]
                let base_ms = base.as_millis() as u64;
                let delay = base_ms * fib;
                #[allow(clippy::cast_possible_truncation)]
                let max_ms = max.as_millis() as u64;
                Duration::from_millis(delay.min(max_ms))
            }
        };

        // Apply jitter
        match &self.config.jitter_strategy {
            JitterStrategy::None => base_delay,
            JitterStrategy::Full => {
                let mut rng = rand::thread_rng();
                #[allow(clippy::cast_possible_truncation)]
                let max_ms = base_delay.as_millis() as u64;
                Duration::from_millis(rng.gen_range(0..=max_ms))
            }
            JitterStrategy::Equal => {
                let mut rng = rand::thread_rng();
                #[allow(clippy::cast_possible_truncation)]
                let base_ms = base_delay.as_millis() as u64;
                let half = base_ms / 2;
                Duration::from_millis(rng.gen_range(half..=base_ms))
            }
            JitterStrategy::Decorrelated { .. } => {
                let mut rng = rand::thread_rng();
                let previous = state.last_delay.unwrap_or(base_delay);
                #[allow(clippy::cast_possible_truncation)]
                let min = base_delay.as_millis() as u64;
                #[allow(clippy::cast_possible_truncation)]
                let max = (previous.as_millis() as u64 * 3).min(60000); // Cap at 60s
                Duration::from_millis(rng.gen_range(min..=max))
            }
        }
    }

    /// Calculate Fibonacci number
    fn fibonacci(&self, n: u32) -> u64 {
        match n {
            0 => 1,
            1 => 1,
            _ => {
                let mut a = 1u64;
                let mut b = 1u64;
                for _ in 2..=n {
                    let temp = a + b;
                    a = b;
                    b = temp;
                }
                b
            }
        }
    }

    /// Generate retry key for tracking
    fn generate_retry_key(&self, context: &HookContext) -> String {
        format!(
            "{:?}:{:?}:{}",
            context.point, context.component_id.component_type, context.component_id.name
        )
    }
}

impl Default for RetryHook {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Hook for RetryHook {
    async fn execute(&self, context: &mut HookContext) -> Result<HookResult> {
        // Check if this hook point is applicable
        if !self.config.applicable_points.contains(&context.point) {
            return Ok(HookResult::Continue);
        }

        // Update metrics
        {
            let mut metrics = self.metrics.write().unwrap();
            metrics.total_operations += 1;
        }

        // Check if there's an error to retry
        let error = match context.get_metadata("error") {
            Some(error_str) => error_str.to_string(),
            None => {
                // Check if this is a result with error
                if let Some(result) = context.data.get("result") {
                    if let Some(error) = result.get("error").and_then(|e| e.as_str()) {
                        error.to_string()
                    } else {
                        return Ok(HookResult::Continue);
                    }
                } else {
                    return Ok(HookResult::Continue);
                }
            }
        };

        // Check if error is retryable
        if !self.is_retryable_error(&error) {
            debug!("RetryHook: Error '{}' is not retryable", error);
            return Ok(HookResult::Continue);
        }

        let retry_key = self.generate_retry_key(context);

        // Get or create retry state
        let mut tracker = self.attempt_tracker.write();
        let state = tracker
            .entry(retry_key.clone())
            .or_insert_with(|| RetryState {
                attempts: 0,
                total_delay: Duration::ZERO,
                start_time: std::time::Instant::now(),
                last_delay: None,
            });

        state.attempts += 1;

        // Update metrics
        {
            let mut metrics = self.metrics.write().unwrap();
            metrics.retry_attempts += 1;
            metrics
                .retry_reasons
                .entry(error.clone())
                .and_modify(|e| *e += 1)
                .or_insert(1);
        }

        // Check max attempts
        if state.attempts >= self.config.max_attempts {
            warn!(
                "RetryHook: Max attempts ({}) reached for {}",
                self.config.max_attempts, retry_key
            );

            {
                let mut metrics = self.metrics.write().unwrap();
                metrics.max_attempts_reached += 1;
                metrics.failed_retries += 1;
            }

            tracker.remove(&retry_key);
            return Ok(HookResult::Continue);
        }

        // Check max duration
        if let Some(max_duration) = self.config.max_retry_duration {
            if state.start_time.elapsed() > max_duration {
                warn!("RetryHook: Max retry duration exceeded for {}", retry_key);

                {
                    let mut metrics = self.metrics.write().unwrap();
                    metrics.max_duration_reached += 1;
                    metrics.failed_retries += 1;
                }

                tracker.remove(&retry_key);
                return Ok(HookResult::Continue);
            }
        }

        // Calculate delay
        let delay = self.calculate_delay(state);
        state.total_delay += delay;
        state.last_delay = Some(delay);

        // Update metrics
        {
            let mut metrics = self.metrics.write().unwrap();
            #[allow(clippy::cast_possible_truncation)]
            let delay_ms = delay.as_millis() as u64;
            metrics.total_delay_ms += delay_ms;
        }

        // Add retry metadata
        context.insert_metadata("retry_attempt".to_string(), state.attempts.to_string());
        context.insert_metadata("retry_delay_ms".to_string(), delay.as_millis().to_string());
        context.insert_metadata("retry_reason".to_string(), error);

        info!(
            "RetryHook: Retrying {} (attempt {}/{}) with {:?} delay",
            retry_key, state.attempts, self.config.max_attempts, delay
        );

        // Return retry result
        Ok(HookResult::Retry {
            delay,
            max_attempts: self.config.max_attempts - state.attempts,
        })
    }

    fn metadata(&self) -> HookMetadata {
        self.metadata.clone()
    }

    fn should_execute(&self, _context: &HookContext) -> bool {
        true
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

#[async_trait]
impl MetricHook for RetryHook {
    async fn record_pre_execution(&self, context: &HookContext) -> Result<()> {
        trace!(
            "RetryHook: Pre-execution for hook point {:?}",
            context.point
        );
        Ok(())
    }

    async fn record_post_execution(
        &self,
        context: &HookContext,
        result: &HookResult,
        _duration: Duration,
    ) -> Result<()> {
        // If the operation succeeded after retry, update metrics
        if matches!(result, HookResult::Continue) {
            if let Some(attempt_str) = context.get_metadata("retry_attempt") {
                if let Ok(attempts) = attempt_str.parse::<u32>() {
                    if attempts > 0 {
                        let mut metrics = self.metrics.write().unwrap();
                        metrics.successful_retries += 1;

                        // Clean up retry state
                        let retry_key = self.generate_retry_key(context);
                        self.attempt_tracker.write().remove(&retry_key);
                    }
                }
            }
        }
        Ok(())
    }
}

#[async_trait]
impl ReplayableHook for RetryHook {
    fn is_replayable(&self) -> bool {
        true
    }

    fn serialize_context(&self, ctx: &HookContext) -> Result<Vec<u8>> {
        // Create a serializable version of the context with retry config
        let mut context_data = ctx.data.clone();

        // Add retry configuration for replay
        context_data.insert(
            "_retry_config".to_string(),
            serde_json::json!({
                "max_attempts": self.config.max_attempts,
                "backoff_strategy": match &self.config.backoff_strategy {
                    BackoffStrategy::Fixed(d) => {
                        #[allow(clippy::cast_possible_truncation)]
                        let delay_ms = d.as_millis() as u64;
                        serde_json::json!({
                            "type": "fixed",
                            "delay_ms": delay_ms
                        })
                    },
                    BackoffStrategy::Linear { base, increment } => {
                        #[allow(clippy::cast_possible_truncation)]
                        let base_ms = base.as_millis() as u64;
                        #[allow(clippy::cast_possible_truncation)]
                        let increment_ms = increment.as_millis() as u64;
                        serde_json::json!({
                            "type": "linear",
                            "base_ms": base_ms,
                            "increment_ms": increment_ms
                        })
                    },
                    BackoffStrategy::Exponential { base, multiplier, max } => {
                        #[allow(clippy::cast_possible_truncation)]
                        let base_ms = base.as_millis() as u64;
                        #[allow(clippy::cast_possible_truncation)]
                        let max_ms = max.as_millis() as u64;
                        serde_json::json!({
                            "type": "exponential",
                            "base_ms": base_ms,
                            "multiplier": multiplier,
                            "max_ms": max_ms
                        })
                    },
                    BackoffStrategy::Fibonacci { base, max } => {
                        #[allow(clippy::cast_possible_truncation)]
                        let base_ms = base.as_millis() as u64;
                        #[allow(clippy::cast_possible_truncation)]
                        let max_ms = max.as_millis() as u64;
                        serde_json::json!({
                            "type": "fibonacci",
                            "base_ms": base_ms,
                            "max_ms": max_ms
                        })
                    },
                },
                "jitter_strategy": match &self.config.jitter_strategy {
                    JitterStrategy::None => "none",
                    JitterStrategy::Full => "full",
                    JitterStrategy::Equal => "equal",
                    JitterStrategy::Decorrelated { .. } => "decorrelated",
                },
                "retry_on_timeout": self.config.retry_on_timeout,
                "retry_on_rate_limit": self.config.retry_on_rate_limit,
                "retryable_errors_count": self.config.retryable_errors.len(),
                "non_retryable_errors_count": self.config.non_retryable_errors.len(),
            }),
        );

        // Add retry state if present
        let retry_states = self.attempt_tracker.read();
        if let Some(state) = retry_states.get(&ctx.correlation_id.to_string()) {
            #[allow(clippy::cast_possible_truncation)]
            let last_delay_ms = state.last_delay.map(|d| d.as_millis() as u64);
            #[allow(clippy::cast_possible_truncation)]
            let total_delay_ms = state.total_delay.as_millis() as u64;
            context_data.insert(
                "_retry_state".to_string(),
                serde_json::json!({
                    "attempts": state.attempts,
                    "last_delay_ms": last_delay_ms,
                    "total_delay_ms": total_delay_ms,
                }),
            );
        }

        // Add metrics snapshot
        let metrics = self.metrics.read().unwrap();
        context_data.insert(
            "_retry_metrics".to_string(),
            serde_json::to_value(&*metrics)?,
        );

        let mut replay_context = ctx.clone();
        replay_context.data = context_data;

        Ok(serde_json::to_vec(&replay_context)?)
    }

    fn deserialize_context(&self, data: &[u8]) -> Result<HookContext> {
        let mut context: HookContext = serde_json::from_slice(data)?;

        // Remove the retry-specific data from context
        context.data.remove("_retry_config");
        context.data.remove("_retry_state");
        context.data.remove("_retry_metrics");

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

    fn create_test_context_with_error(error: &str) -> HookContext {
        let component_id = ComponentId::new(ComponentType::Tool, "test".to_string());
        let mut context = HookContext::new(HookPoint::ToolError, component_id);
        context.insert_metadata("error".to_string(), error.to_string());
        context
    }
    #[tokio::test]
    async fn test_retry_hook_basic() {
        let hook = RetryHook::new();
        let mut context = create_test_context_with_error("connection_error");

        let result = hook.execute(&mut context).await.unwrap();
        assert!(matches!(result, HookResult::Retry { .. }));

        // Check metadata
        assert_eq!(context.get_metadata("retry_attempt").unwrap(), "1");
        assert!(context.get_metadata("retry_delay_ms").is_some());
    }
    #[tokio::test]
    async fn test_retry_hook_non_retryable() {
        let hook = RetryHook::new();
        let mut context = create_test_context_with_error("validation_error");

        let result = hook.execute(&mut context).await.unwrap();
        assert!(matches!(result, HookResult::Continue));
    }
    #[tokio::test]
    async fn test_retry_hook_max_attempts() {
        let hook = RetryHook::new().with_max_attempts(2);

        let mut context = create_test_context_with_error("timeout");

        // First retry
        let result = hook.execute(&mut context).await.unwrap();
        assert!(matches!(result, HookResult::Retry { .. }));

        // Second retry (max reached)
        let result = hook.execute(&mut context).await.unwrap();
        assert!(matches!(result, HookResult::Continue));

        let metrics = hook.metrics();
        assert_eq!(metrics.max_attempts_reached, 1);
    }
    #[tokio::test]
    async fn test_backoff_strategies() {
        // Test fixed backoff
        let hook = RetryHook::new()
            .with_backoff_strategy(BackoffStrategy::Fixed(Duration::from_millis(100)));

        let mut context = create_test_context_with_error("timeout");
        let result = hook.execute(&mut context).await.unwrap();

        if let HookResult::Retry { delay, .. } = result {
            assert_eq!(delay, Duration::from_millis(100));
        } else {
            panic!("Expected Retry result");
        }

        // Test exponential backoff
        let hook = RetryHook::new().with_backoff_strategy(BackoffStrategy::Exponential {
            base: Duration::from_millis(100),
            multiplier: 2.0,
            max: Duration::from_secs(1),
        });

        let mut context = create_test_context_with_error("timeout");
        let _ = hook.execute(&mut context).await.unwrap();
        let result = hook.execute(&mut context).await.unwrap();

        if let HookResult::Retry { delay, .. } = result {
            // Second attempt should have ~400ms delay (100 * 2^2)
            assert!(delay >= Duration::from_millis(350));
            assert!(delay <= Duration::from_millis(450));
        }
    }
    #[tokio::test]
    async fn test_jitter_strategies() {
        // Test full jitter
        let hook = RetryHook::new()
            .with_backoff_strategy(BackoffStrategy::Fixed(Duration::from_millis(1000)))
            .with_jitter_strategy(JitterStrategy::Full);

        let mut delays = Vec::new();
        for _ in 0..5 {
            let mut context = create_test_context_with_error("timeout");
            if let HookResult::Retry { delay, .. } = hook.execute(&mut context).await.unwrap() {
                delays.push(delay.as_millis());
            }
        }

        // With full jitter, delays should vary
        let unique_delays: std::collections::HashSet<_> = delays.iter().collect();
        assert!(unique_delays.len() > 1);
        assert!(delays.iter().all(|&d| d <= 1000));
    }
    #[tokio::test]
    async fn test_retryable_patterns() {
        let hook = RetryHook::new()
            .with_retryable_error("custom_error".to_string())
            .with_non_retryable_error("fatal_error".to_string());

        // Should retry custom error
        let mut context = create_test_context_with_error("custom_error occurred");
        let result = hook.execute(&mut context).await.unwrap();
        assert!(matches!(result, HookResult::Retry { .. }));

        // Should not retry fatal error
        let mut context = create_test_context_with_error("fatal_error occurred");
        let result = hook.execute(&mut context).await.unwrap();
        assert!(matches!(result, HookResult::Continue));

        // Non-retryable takes precedence
        let mut context = create_test_context_with_error("custom_error with fatal_error");
        let result = hook.execute(&mut context).await.unwrap();
        assert!(matches!(result, HookResult::Continue));
    }
    #[tokio::test]
    async fn test_retry_metrics() {
        let hook = RetryHook::new().with_max_attempts(3);

        // Successful retry
        let mut context = create_test_context_with_error("timeout");
        let _ = hook.execute(&mut context).await.unwrap();

        // Simulate success by recording post execution
        hook.record_post_execution(&context, &HookResult::Continue, Duration::from_millis(100))
            .await
            .unwrap();

        let metrics = hook.metrics();
        assert_eq!(metrics.total_operations, 1);
        assert_eq!(metrics.retry_attempts, 1);
        assert_eq!(metrics.successful_retries, 1);
        assert_eq!(metrics.retry_rate(), 1.0);
        assert_eq!(metrics.success_rate(), 1.0);
    }
    #[test]
    fn test_fibonacci_calculation() {
        let hook = RetryHook::new();
        assert_eq!(hook.fibonacci(0), 1);
        assert_eq!(hook.fibonacci(1), 1);
        assert_eq!(hook.fibonacci(2), 2);
        assert_eq!(hook.fibonacci(3), 3);
        assert_eq!(hook.fibonacci(4), 5);
        assert_eq!(hook.fibonacci(5), 8);
    }
    #[test]
    fn test_hook_metadata() {
        let hook = RetryHook::new();
        let metadata = hook.metadata();

        assert_eq!(metadata.name, "RetryHook");
        assert!(metadata.description.is_some());
        assert_eq!(metadata.priority, Priority::HIGH);
        assert_eq!(metadata.language, Language::Native);
        assert!(metadata.tags.contains(&"builtin".to_string()));
        assert!(metadata.tags.contains(&"retry".to_string()));
    }
    #[test]
    fn test_retry_config_defaults() {
        let config = RetryConfig::default();
        assert_eq!(config.max_attempts, 3);
        assert!(config.retry_on_timeout);
        assert!(config.retry_on_rate_limit);
        assert!(config.retryable_errors.contains("timeout"));
        assert!(config.retryable_errors.contains("connection_error"));
    }
}
