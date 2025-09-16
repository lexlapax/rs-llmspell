//! ABOUTME: Session rate limiting policy leveraging `RateLimitHook`
//! ABOUTME: Enforces API call rates and operation frequency limits

#![allow(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::cast_precision_loss
)]

use anyhow::Result;
use async_trait::async_trait;
use llmspell_hooks::{
    builtin::RateLimitHook,
    traits::Hook,
    types::{HookMetadata, Priority},
    HookContext, HookResult,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

/// Rate limit configuration for sessions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitConfig {
    /// Global rate limit (requests per minute)
    pub global_rpm: u32,
    /// Per-session rate limit
    pub per_session_rpm: u32,
    /// Per-operation type limits
    pub operation_limits: HashMap<String, u32>,
    /// Burst capacity multiplier
    pub burst_multiplier: f32,
    /// Whether to allow burst traffic
    pub allow_burst: bool,
    /// Rate limit window duration
    pub window_duration: Duration,
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        let mut operation_limits = HashMap::new();
        operation_limits.insert("llm_call".to_string(), 60); // 60 LLM calls per minute
        operation_limits.insert("tool_execution".to_string(), 120); // 120 tool calls per minute
        operation_limits.insert("state_update".to_string(), 300); // 300 state updates per minute

        Self {
            global_rpm: 1000,     // 1000 requests per minute globally
            per_session_rpm: 100, // 100 requests per minute per session
            operation_limits,
            burst_multiplier: 1.5, // Allow 50% burst
            allow_burst: true,
            window_duration: Duration::from_secs(60), // 1 minute window
        }
    }
}

/// Session rate limit policy
pub struct SessionRateLimitPolicy {
    /// Rate limit configuration
    config: RateLimitConfig,
    /// Hook metadata
    metadata: HookMetadata,
    /// Global rate limiter
    global_limiter: Arc<RateLimitHook>,
    /// Per-session rate limiter
    session_limiter: Arc<RateLimitHook>,
    /// Operation-specific limiters
    operation_limiters: HashMap<String, Arc<RateLimitHook>>,
}

impl Clone for SessionRateLimitPolicy {
    fn clone(&self) -> Self {
        Self {
            config: self.config.clone(),
            metadata: self.metadata.clone(),
            global_limiter: Arc::clone(&self.global_limiter),
            session_limiter: Arc::clone(&self.session_limiter),
            operation_limiters: self.operation_limiters.clone(),
        }
    }
}

impl SessionRateLimitPolicy {
    /// Create a new rate limit policy
    pub fn new(config: RateLimitConfig) -> Self {
        let metadata = HookMetadata {
            name: "SessionRateLimitPolicy".to_string(),
            version: "1.0.0".to_string(),
            description: Some("Enforces rate limits for session operations".to_string()),
            priority: Priority(80),
            tags: vec!["policy".to_string(), "rate-limit".to_string()],
            language: llmspell_hooks::Language::Native,
        };

        // Create global rate limiter
        let global_limiter = Arc::new(
            RateLimitHook::new()
                .with_rate_per_second((config.global_rpm as f64) / 60.0)
                .with_burst(((config.global_rpm as f64) * config.burst_multiplier as f64) as usize),
        );

        // Create per-session rate limiter
        let session_limiter = Arc::new(
            RateLimitHook::new()
                .with_rate_per_second((config.per_session_rpm as f64) / 60.0)
                .with_burst(
                    ((config.per_session_rpm as f64) * config.burst_multiplier as f64) as usize,
                ),
        );

        // Create operation-specific limiters
        let mut operation_limiters = HashMap::new();
        for (op_type, limit) in &config.operation_limits {
            let op_limiter = Arc::new(
                RateLimitHook::new()
                    .with_rate_per_second((*limit as f64) / 60.0)
                    .with_burst(((*limit as f64) * config.burst_multiplier as f64) as usize),
            );
            operation_limiters.insert(op_type.clone(), op_limiter);
        }

        Self {
            config,
            metadata,
            global_limiter,
            session_limiter,
            operation_limiters,
        }
    }

    /// Get the operation type from context
    fn get_operation_type(context: &HookContext) -> Option<String> {
        context
            .data
            .get("operation_type")
            .and_then(|v| v.as_str())
            .map(std::string::ToString::to_string)
    }

    /// Check rate limits for the operation
    async fn check_rate_limits(&self, context: &mut HookContext) -> Result<RateLimitStatus> {
        // Check global rate limit
        let global_result = self.global_limiter.execute(context).await?;
        if !global_result.should_continue() {
            return Ok(RateLimitStatus::Exceeded {
                limit_type: "global".to_string(),
                retry_after: Self::extract_retry_after(context),
            });
        }

        // Check per-session rate limit
        let session_result = self.session_limiter.execute(context).await?;
        if !session_result.should_continue() {
            return Ok(RateLimitStatus::Exceeded {
                limit_type: "session".to_string(),
                retry_after: Self::extract_retry_after(context),
            });
        }

        // Check operation-specific rate limit
        if let Some(op_type) = Self::get_operation_type(context) {
            if let Some(op_limiter) = self.operation_limiters.get(&op_type) {
                let op_result = op_limiter.execute(context).await?;
                if !op_result.should_continue() {
                    return Ok(RateLimitStatus::Exceeded {
                        limit_type: format!("operation:{op_type}"),
                        retry_after: Self::extract_retry_after(context),
                    });
                }
            }
        }

        // Check if we're approaching limits
        if let Some(remaining) = Self::extract_remaining_tokens(context) {
            if let Some(limit) = Self::extract_limit(context) {
                let usage_percent = ((limit - remaining) as f64 / limit as f64) * 100.0;
                if usage_percent > 80.0 {
                    return Ok(RateLimitStatus::Warning { usage_percent });
                }
            }
        }

        Ok(RateLimitStatus::Ok)
    }

    /// Extract retry-after duration from context
    fn extract_retry_after(context: &HookContext) -> Option<Duration> {
        context
            .data
            .get("retry_after_seconds")
            .and_then(serde_json::Value::as_u64)
            .map(Duration::from_secs)
    }

    /// Extract remaining tokens from context
    fn extract_remaining_tokens(context: &HookContext) -> Option<u32> {
        context
            .data
            .get("rate_limit_remaining")
            .and_then(serde_json::Value::as_u64)
            .map(|v| v as u32)
    }

    /// Extract rate limit from context
    fn extract_limit(context: &HookContext) -> Option<u32> {
        context
            .data
            .get("rate_limit_limit")
            .and_then(serde_json::Value::as_u64)
            .map(|v| v as u32)
    }
}

#[async_trait]
impl Hook for SessionRateLimitPolicy {
    async fn execute(&self, context: &mut HookContext) -> Result<HookResult> {
        // Check rate limits
        match self.check_rate_limits(context).await? {
            RateLimitStatus::Exceeded {
                limit_type,
                retry_after,
            } => {
                let retry_seconds = retry_after.map_or(60, |d| d.as_secs());

                context.data.insert(
                    "rate_limit_exceeded".to_string(),
                    serde_json::json!({
                        "limit_type": limit_type,
                        "retry_after_seconds": retry_seconds,
                    }),
                );

                return Ok(HookResult::Cancel(format!(
                    "Rate limit exceeded ({limit_type}). Retry after {retry_seconds} seconds"
                )));
            }
            RateLimitStatus::Warning { usage_percent } => {
                context.data.insert(
                    "rate_limit_warning".to_string(),
                    serde_json::json!({
                        "usage_percent": usage_percent,
                    }),
                );
            }
            RateLimitStatus::Ok => {}
        }

        Ok(HookResult::Continue)
    }

    fn should_execute(&self, context: &HookContext) -> bool {
        // Execute for all session operations
        matches!(
            context.point,
            llmspell_hooks::HookPoint::SessionStart
                | llmspell_hooks::HookPoint::SessionEnd
                | llmspell_hooks::HookPoint::SessionCheckpoint
                | llmspell_hooks::HookPoint::SessionRestore
                | llmspell_hooks::HookPoint::SessionSave
        )
    }

    fn metadata(&self) -> HookMetadata {
        self.metadata.clone()
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

/// Rate limit status
#[derive(Debug, Clone, PartialEq)]
enum RateLimitStatus {
    /// Rate limit not exceeded
    Ok,
    /// Approaching rate limit
    Warning { usage_percent: f64 },
    /// Rate limit exceeded
    Exceeded {
        limit_type: String,
        retry_after: Option<Duration>,
    },
}

#[cfg(test)]
mod tests {
    use super::*;
    use llmspell_hooks::{
        types::{ComponentId, ComponentType},
        HookPoint,
    };
    #[tokio::test]
    async fn test_rate_limit_policy_creation() {
        let config = RateLimitConfig::default();
        let policy = SessionRateLimitPolicy::new(config);

        assert_eq!(policy.metadata.name, "SessionRateLimitPolicy");
        assert!(!policy.operation_limiters.is_empty());
    }
    #[tokio::test]
    async fn test_operation_type_extraction() {
        let config = RateLimitConfig::default();
        let _policy = SessionRateLimitPolicy::new(config);

        let mut context = HookContext::new(
            HookPoint::SessionCheckpoint,
            ComponentId::new(ComponentType::Agent, "test".to_string()),
        );

        context
            .data
            .insert("operation_type".to_string(), serde_json::json!("llm_call"));

        let op_type = SessionRateLimitPolicy::get_operation_type(&context);
        assert_eq!(op_type, Some("llm_call".to_string()));
    }
    #[tokio::test]
    async fn test_rate_limit_with_session_id() {
        let config = RateLimitConfig {
            per_session_rpm: 1, // Very low for testing
            ..Default::default()
        };
        let policy = SessionRateLimitPolicy::new(config);

        let mut context = HookContext::new(
            HookPoint::SessionCheckpoint,
            ComponentId::new(ComponentType::Agent, "test-session".to_string()),
        );

        // Add session ID to context
        context
            .data
            .insert("session_id".to_string(), serde_json::json!("test-session"));

        // First call should succeed
        let result = policy.execute(&mut context).await.unwrap();
        assert!(result.should_continue());

        // Rapid subsequent calls might be rate limited
        // (Note: Actual rate limiting depends on the underlying RateLimitHook implementation)
    }
    #[tokio::test]
    async fn test_warning_detection() {
        let config = RateLimitConfig::default();
        let policy = SessionRateLimitPolicy::new(config);

        let mut context = HookContext::new(
            HookPoint::SessionCheckpoint,
            ComponentId::new(ComponentType::Agent, "test".to_string()),
        );

        // Simulate high usage
        context
            .data
            .insert("rate_limit_remaining".to_string(), serde_json::json!(10u64));
        context
            .data
            .insert("rate_limit_limit".to_string(), serde_json::json!(100u64));

        let result = policy.execute(&mut context).await.unwrap();
        assert!(result.should_continue());
        assert!(context.data.contains_key("rate_limit_warning"));
    }
}
