//! ABOUTME: Session timeout policy implementation using Hook trait
//! ABOUTME: Enforces session duration limits and idle timeout detection

#![allow(clippy::unnecessary_wraps, clippy::single_match)]

use anyhow::Result;
use async_trait::async_trait;
use chrono::{DateTime, Duration as ChronoDuration, Utc};
use llmspell_hooks::{
    traits::Hook,
    types::{HookMetadata, Priority},
    HookContext, HookResult,
};
use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Timeout configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeoutConfig {
    /// Maximum session duration
    pub max_session_duration: Duration,
    /// Idle timeout duration
    pub idle_timeout: Duration,
    /// Grace period before hard timeout
    pub grace_period: Duration,
    /// Whether to send warnings before timeout
    pub enable_warnings: bool,
    /// Warning intervals (e.g., 5min, 1min before timeout)
    pub warning_intervals: Vec<Duration>,
}

impl Default for TimeoutConfig {
    fn default() -> Self {
        Self {
            max_session_duration: Duration::from_secs(3600), // 1 hour
            idle_timeout: Duration::from_secs(900),          // 15 minutes
            grace_period: Duration::from_secs(60),           // 1 minute
            enable_warnings: true,
            warning_intervals: vec![
                Duration::from_secs(300), // 5 minutes
                Duration::from_secs(60),  // 1 minute
            ],
        }
    }
}

/// Session timeout policy
#[derive(Debug, Clone)]
pub struct SessionTimeoutPolicy {
    /// Timeout configuration
    config: TimeoutConfig,
    /// Hook metadata
    metadata: HookMetadata,
}

impl SessionTimeoutPolicy {
    /// Create a new timeout policy
    pub fn new(config: TimeoutConfig) -> Self {
        let metadata = HookMetadata {
            name: "SessionTimeoutPolicy".to_string(),
            version: "1.0.0".to_string(),
            description: Some("Enforces session duration and idle timeout limits".to_string()),
            priority: Priority(100),
            tags: vec!["policy".to_string(), "timeout".to_string()],
            language: llmspell_hooks::Language::Native,
        };

        Self { config, metadata }
    }

    /// Check if session has exceeded max duration
    fn check_max_duration(&self, context: &HookContext) -> Result<TimeoutStatus> {
        // Get session start time from context
        let session_start = context
            .data
            .get("session_start_time")
            .and_then(|v| v.as_str())
            .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
            .map(|dt| dt.with_timezone(&Utc));

        if let Some(start_time) = session_start {
            let elapsed = Utc::now().signed_duration_since(start_time);
            let max_duration = ChronoDuration::from_std(self.config.max_session_duration)
                .unwrap_or(ChronoDuration::seconds(3600));

            if elapsed > max_duration {
                return Ok(TimeoutStatus::Exceeded {
                    duration: elapsed.to_std().unwrap_or(self.config.max_session_duration),
                    limit: self.config.max_session_duration,
                });
            }

            // Check if we're in warning zone
            for warning_interval in &self.config.warning_intervals {
                let warning_duration = ChronoDuration::from_std(*warning_interval)
                    .unwrap_or(ChronoDuration::seconds(300));
                let time_until_timeout = max_duration - elapsed;

                if time_until_timeout <= warning_duration
                    && time_until_timeout > ChronoDuration::zero()
                {
                    return Ok(TimeoutStatus::Warning {
                        time_remaining: time_until_timeout
                            .to_std()
                            .unwrap_or(Duration::from_secs(0)),
                    });
                }
            }
        }

        Ok(TimeoutStatus::Active)
    }

    /// Check idle timeout
    fn check_idle_timeout(&self, context: &HookContext) -> Result<TimeoutStatus> {
        // Get last activity time from context
        let last_activity = context
            .data
            .get("last_activity_time")
            .and_then(|v| v.as_str())
            .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
            .map(|dt| dt.with_timezone(&Utc));

        if let Some(last_time) = last_activity {
            let idle_duration = Utc::now().signed_duration_since(last_time);
            let idle_limit = ChronoDuration::from_std(self.config.idle_timeout)
                .unwrap_or(ChronoDuration::seconds(900));

            if idle_duration > idle_limit {
                return Ok(TimeoutStatus::Idle {
                    duration: idle_duration.to_std().unwrap_or(self.config.idle_timeout),
                });
            }
        }

        Ok(TimeoutStatus::Active)
    }

    /// Update last activity time
    fn update_activity_time(&self, context: &mut HookContext) {
        context.data.insert(
            "last_activity_time".to_string(),
            serde_json::json!(Utc::now().to_rfc3339()),
        );
    }
}

#[async_trait]
impl Hook for SessionTimeoutPolicy {
    async fn execute(&self, context: &mut HookContext) -> Result<HookResult> {
        // Check idle timeout BEFORE updating activity time
        match self.check_idle_timeout(context)? {
            TimeoutStatus::Idle { duration } => {
                // Check if we're in grace period
                let grace_exceeded = duration > self.config.idle_timeout + self.config.grace_period;

                if grace_exceeded {
                    context.data.insert(
                        "timeout_reason".to_string(),
                        serde_json::json!({
                            "type": "idle_timeout",
                            "idle_duration_seconds": duration.as_secs(),
                            "limit_seconds": self.config.idle_timeout.as_secs(),
                        }),
                    );

                    return Ok(HookResult::Cancel(
                        "Session idle timeout exceeded".to_string(),
                    ));
                }
                // In grace period
                context.data.insert(
                    "timeout_warning".to_string(),
                    serde_json::json!({
                        "type": "idle_grace_period",
                        "idle_duration_seconds": duration.as_secs(),
                    }),
                );
            }
            _ => {}
        }

        // Update activity time for any session operation
        self.update_activity_time(context);

        // Check max duration
        match self.check_max_duration(context)? {
            TimeoutStatus::Exceeded { duration, limit } => {
                context.data.insert(
                    "timeout_reason".to_string(),
                    serde_json::json!({
                        "type": "max_duration_exceeded",
                        "duration_seconds": duration.as_secs(),
                        "limit_seconds": limit.as_secs(),
                    }),
                );

                return Ok(HookResult::Cancel(
                    "Session exceeded maximum duration limit".to_string(),
                ));
            }
            TimeoutStatus::Warning { time_remaining } => {
                if self.config.enable_warnings {
                    context.data.insert(
                        "timeout_warning".to_string(),
                        serde_json::json!({
                            "type": "approaching_timeout",
                            "time_remaining_seconds": time_remaining.as_secs(),
                        }),
                    );
                }
            }
            TimeoutStatus::Active | TimeoutStatus::Idle { .. } => {}
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
        )
    }

    fn metadata(&self) -> HookMetadata {
        self.metadata.clone()
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

/// Timeout status
#[derive(Debug, Clone, PartialEq)]
enum TimeoutStatus {
    /// Session is active
    Active,
    /// Session is approaching timeout
    Warning { time_remaining: Duration },
    /// Session exceeded max duration
    Exceeded { duration: Duration, limit: Duration },
    /// Session is idle
    Idle { duration: Duration },
}

#[cfg(test)]
mod tests {
    use super::*;
    use llmspell_hooks::{types::ComponentId, HookPoint};

    #[cfg_attr(test_category = "unit")]
    #[tokio::test]
    async fn test_timeout_policy_creation() {
        let config = TimeoutConfig::default();
        let policy = SessionTimeoutPolicy::new(config);

        assert_eq!(policy.metadata.name, "SessionTimeoutPolicy");
        assert!(policy.should_execute(&HookContext::new(
            HookPoint::SessionStart,
            ComponentId::new(llmspell_hooks::ComponentType::Agent, "test".to_string())
        )));
    }

    #[cfg_attr(test_category = "unit")]
    #[tokio::test]
    async fn test_activity_time_update() {
        let config = TimeoutConfig::default();
        let policy = SessionTimeoutPolicy::new(config);

        let mut context = HookContext::new(
            HookPoint::SessionStart,
            ComponentId::new(llmspell_hooks::ComponentType::Agent, "test".to_string()),
        );

        let result = policy.execute(&mut context).await.unwrap();
        assert!(result.should_continue());
        assert!(context.data.contains_key("last_activity_time"));
    }

    #[cfg_attr(test_category = "unit")]
    #[tokio::test]
    async fn test_max_duration_check() {
        let config = TimeoutConfig {
            max_session_duration: Duration::from_secs(1), // 1 second for testing
            ..Default::default()
        };
        let policy = SessionTimeoutPolicy::new(config);

        let mut context = HookContext::new(
            HookPoint::SessionCheckpoint,
            ComponentId::new(llmspell_hooks::ComponentType::Agent, "test".to_string()),
        );

        // Set session start time to past
        let past_time = Utc::now() - ChronoDuration::seconds(2);
        context.data.insert(
            "session_start_time".to_string(),
            serde_json::json!(past_time.to_rfc3339()),
        );

        let result = policy.execute(&mut context).await.unwrap();
        assert!(!result.should_continue());
        assert!(context.data.contains_key("timeout_reason"));
    }

    #[cfg_attr(test_category = "unit")]
    #[tokio::test]
    async fn test_idle_timeout_check() {
        let config = TimeoutConfig {
            idle_timeout: Duration::from_secs(1), // 1 second for testing
            grace_period: Duration::from_secs(0), // No grace period
            ..Default::default()
        };
        let policy = SessionTimeoutPolicy::new(config);

        let mut context = HookContext::new(
            HookPoint::SessionCheckpoint,
            ComponentId::new(llmspell_hooks::ComponentType::Agent, "test".to_string()),
        );

        // Set last activity to past
        let past_time = Utc::now() - ChronoDuration::seconds(2);
        context.data.insert(
            "last_activity_time".to_string(),
            serde_json::json!(past_time.to_rfc3339()),
        );

        let result = policy.execute(&mut context).await.unwrap();
        assert!(!result.should_continue());
    }
}
