//! ABOUTME: Session resource limit policy leveraging `CostTrackingHook`
//! ABOUTME: Enforces memory, token, and operation limits for sessions

#![allow(clippy::cast_precision_loss)]

use anyhow::Result;
use async_trait::async_trait;
use llmspell_hooks::{
    builtin::cost_tracking::{CostTrackingHook, TokenUsage},
    traits::Hook,
    types::{HookMetadata, Priority},
    HookContext, HookResult,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// Resource configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceConfig {
    /// Maximum memory usage in bytes
    pub max_memory_bytes: Option<u64>,
    /// Maximum total tokens per session
    pub max_total_tokens: Option<u64>,
    /// Maximum tokens per operation
    pub max_tokens_per_operation: Option<u64>,
    /// Maximum number of operations
    pub max_operations: Option<u64>,
    /// Maximum cost in USD
    pub max_cost_usd: Option<f64>,
    /// Enable cost tracking
    pub enable_cost_tracking: bool,
    /// Cost alert thresholds (percentage of max)
    pub cost_alert_thresholds: Vec<f32>,
}

impl Default for ResourceConfig {
    fn default() -> Self {
        Self {
            max_memory_bytes: Some(1024 * 1024 * 1024), // 1GB
            max_total_tokens: Some(1_000_000),          // 1M tokens
            max_tokens_per_operation: Some(100_000),    // 100k per op
            max_operations: Some(10_000),               // 10k operations
            max_cost_usd: Some(100.0),                  // $100
            enable_cost_tracking: true,
            cost_alert_thresholds: vec![0.5, 0.75, 0.9], // 50%, 75%, 90%
        }
    }
}

/// Session resource limit policy
#[derive(Clone)]
pub struct SessionResourcePolicy {
    /// Resource configuration
    config: ResourceConfig,
    /// Hook metadata
    metadata: HookMetadata,
    /// Cost tracking hook (if enabled)
    cost_tracker: Option<Arc<CostTrackingHook>>,
}

impl SessionResourcePolicy {
    /// Create a new resource limit policy
    pub fn new(config: ResourceConfig) -> Self {
        let metadata = HookMetadata {
            name: "SessionResourcePolicy".to_string(),
            version: "1.0.0".to_string(),
            description: Some("Enforces memory, token, and cost limits for sessions".to_string()),
            priority: Priority(90),
            tags: vec![
                "policy".to_string(),
                "resource".to_string(),
                "limits".to_string(),
            ],
            language: llmspell_hooks::Language::Native,
        };

        let cost_tracker = if config.enable_cost_tracking {
            Some(Arc::new(CostTrackingHook::new()))
        } else {
            None
        };

        Self {
            config,
            metadata,
            cost_tracker,
        }
    }

    /// Check memory usage
    fn check_memory_usage(&self, context: &HookContext) -> ResourceStatus {
        if let Some(max_memory) = self.config.max_memory_bytes {
            let current_memory = context
                .data
                .get("memory_usage_bytes")
                .and_then(serde_json::Value::as_u64)
                .unwrap_or(0);

            if current_memory > max_memory {
                return ResourceStatus::Exceeded {
                    resource: ResourceType::Memory,
                    current: current_memory as f64,
                    limit: max_memory as f64,
                };
            }

            // Check if approaching limit (>90%)
            if current_memory as f64 > max_memory as f64 * 0.9 {
                return ResourceStatus::Warning {
                    resource: ResourceType::Memory,
                    usage_percent: (current_memory as f64 / max_memory as f64) * 100.0,
                };
            }
        }

        ResourceStatus::Ok
    }

    /// Check token usage
    fn check_token_usage(&self, context: &HookContext) -> ResourceStatus {
        // Check total tokens
        if let Some(max_tokens) = self.config.max_total_tokens {
            let total_tokens = context
                .data
                .get("total_tokens_used")
                .and_then(serde_json::Value::as_u64)
                .unwrap_or(0);

            if total_tokens > max_tokens {
                return ResourceStatus::Exceeded {
                    resource: ResourceType::TotalTokens,
                    current: total_tokens as f64,
                    limit: max_tokens as f64,
                };
            }
        }

        // Check tokens per operation
        if let Some(max_per_op) = self.config.max_tokens_per_operation {
            if let Some(token_usage) = context.data.get("token_usage") {
                if let Ok(usage) = serde_json::from_value::<TokenUsage>(token_usage.clone()) {
                    if usage.total_tokens > max_per_op {
                        return ResourceStatus::Exceeded {
                            resource: ResourceType::TokensPerOperation,
                            current: usage.total_tokens as f64,
                            limit: max_per_op as f64,
                        };
                    }
                }
            }
        }

        ResourceStatus::Ok
    }

    /// Check operation count
    fn check_operation_count(&self, context: &HookContext) -> ResourceStatus {
        if let Some(max_ops) = self.config.max_operations {
            let operation_count = context
                .data
                .get("operation_count")
                .and_then(serde_json::Value::as_u64)
                .unwrap_or(0);

            if operation_count > max_ops {
                return ResourceStatus::Exceeded {
                    resource: ResourceType::Operations,
                    current: operation_count as f64,
                    limit: max_ops as f64,
                };
            }
        }

        ResourceStatus::Ok
    }

    /// Check cost limits
    async fn check_cost_limits(&self, context: &mut HookContext) -> Result<ResourceStatus> {
        if let (Some(cost_tracker), Some(max_cost)) = (&self.cost_tracker, self.config.max_cost_usd)
        {
            // Execute cost tracking hook to update costs
            cost_tracker.execute(context).await?;

            // Get total cost from context
            let total_cost = context
                .data
                .get("total_cost_usd")
                .and_then(serde_json::Value::as_f64)
                .unwrap_or(0.0);

            if total_cost > max_cost {
                return Ok(ResourceStatus::Exceeded {
                    resource: ResourceType::Cost,
                    current: total_cost,
                    limit: max_cost,
                });
            }

            // Check alert thresholds
            for threshold in &self.config.cost_alert_thresholds {
                let threshold_value = max_cost * f64::from(*threshold);
                if total_cost > threshold_value && total_cost <= threshold_value * 1.1 {
                    return Ok(ResourceStatus::Warning {
                        resource: ResourceType::Cost,
                        usage_percent: (total_cost / max_cost) * 100.0,
                    });
                }
            }
        }

        Ok(ResourceStatus::Ok)
    }

    /// Update resource usage metrics
    fn update_usage(context: &mut HookContext) {
        // Increment operation count
        let op_count = context
            .data
            .get("operation_count")
            .and_then(serde_json::Value::as_u64)
            .unwrap_or(0);
        context.data.insert(
            "operation_count".to_string(),
            serde_json::json!(op_count + 1),
        );

        // Update total tokens if token usage is present
        if let Some(token_usage) = context.data.get("token_usage") {
            if let Ok(usage) = serde_json::from_value::<TokenUsage>(token_usage.clone()) {
                let total_tokens = context
                    .data
                    .get("total_tokens_used")
                    .and_then(serde_json::Value::as_u64)
                    .unwrap_or(0);
                context.data.insert(
                    "total_tokens_used".to_string(),
                    serde_json::json!(total_tokens + usage.total_tokens),
                );
            }
        }
    }
}

#[async_trait]
impl Hook for SessionResourcePolicy {
    #[allow(clippy::too_many_lines)]
    async fn execute(&self, context: &mut HookContext) -> Result<HookResult> {
        // Update usage metrics
        Self::update_usage(context);

        // Check memory usage
        if let ResourceStatus::Exceeded {
            resource,
            current,
            limit,
        } = self.check_memory_usage(context)
        {
            context.data.insert(
                "resource_limit_exceeded".to_string(),
                serde_json::json!({
                    "resource": resource.to_string(),
                    "current": current,
                    "limit": limit,
                }),
            );
            return Ok(HookResult::Cancel(format!(
                "Memory limit exceeded: {:.2} MB / {:.2} MB",
                current / 1_048_576.0,
                limit / 1_048_576.0
            )));
        }

        // Check token usage
        match self.check_token_usage(context) {
            ResourceStatus::Exceeded {
                resource,
                current,
                limit,
            } => {
                context.data.insert(
                    "resource_limit_exceeded".to_string(),
                    serde_json::json!({
                        "resource": resource.to_string(),
                        "current": current,
                        "limit": limit,
                    }),
                );
                return Ok(HookResult::Cancel(format!(
                    "{resource} limit exceeded: {current} / {limit}"
                )));
            }
            ResourceStatus::Warning {
                resource,
                usage_percent,
            } => {
                context.data.insert(
                    "resource_warning".to_string(),
                    serde_json::json!({
                        "resource": resource.to_string(),
                        "usage_percent": usage_percent,
                    }),
                );
            }
            ResourceStatus::Ok => {}
        }

        // Check operation count
        if let ResourceStatus::Exceeded {
            resource,
            current,
            limit,
        } = self.check_operation_count(context)
        {
            context.data.insert(
                "resource_limit_exceeded".to_string(),
                serde_json::json!({
                    "resource": resource.to_string(),
                    "current": current,
                    "limit": limit,
                }),
            );
            return Ok(HookResult::Cancel(format!(
                "Operation limit exceeded: {current} / {limit}"
            )));
        }

        // Check cost limits
        match self.check_cost_limits(context).await? {
            ResourceStatus::Exceeded {
                resource,
                current,
                limit,
            } => {
                context.data.insert(
                    "resource_limit_exceeded".to_string(),
                    serde_json::json!({
                        "resource": resource.to_string(),
                        "current": current,
                        "limit": limit,
                    }),
                );
                return Ok(HookResult::Cancel(format!(
                    "Cost limit exceeded: ${current:.2} / ${limit:.2}"
                )));
            }
            ResourceStatus::Warning {
                resource,
                usage_percent,
            } => {
                context.data.insert(
                    "cost_warning".to_string(),
                    serde_json::json!({
                        "resource": resource.to_string(),
                        "usage_percent": usage_percent,
                    }),
                );
            }
            ResourceStatus::Ok => {}
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

/// Resource type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ResourceType {
    Memory,
    TotalTokens,
    TokensPerOperation,
    Operations,
    Cost,
}

impl std::fmt::Display for ResourceType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ResourceType::Memory => write!(f, "Memory"),
            ResourceType::TotalTokens => write!(f, "Total Tokens"),
            ResourceType::TokensPerOperation => write!(f, "Tokens per Operation"),
            ResourceType::Operations => write!(f, "Operations"),
            ResourceType::Cost => write!(f, "Cost"),
        }
    }
}

/// Resource status
#[derive(Debug, Clone, PartialEq)]
enum ResourceStatus {
    /// Resource usage is within limits
    Ok,
    /// Resource usage is approaching limit
    Warning {
        resource: ResourceType,
        usage_percent: f64,
    },
    /// Resource limit exceeded
    Exceeded {
        resource: ResourceType,
        current: f64,
        limit: f64,
    },
}

#[cfg(test)]
mod tests {
    use super::*;
    use llmspell_hooks::{types::ComponentId, HookPoint};
    #[tokio::test]
    async fn test_resource_policy_creation() {
        let config = ResourceConfig::default();
        let policy = SessionResourcePolicy::new(config);

        assert_eq!(policy.metadata.name, "SessionResourcePolicy");
        assert!(policy.cost_tracker.is_some());
    }
    #[tokio::test]
    async fn test_memory_limit_check() {
        let config = ResourceConfig {
            max_memory_bytes: Some(1000),
            ..Default::default()
        };
        let policy = SessionResourcePolicy::new(config);

        let mut context = HookContext::new(
            HookPoint::SessionCheckpoint,
            ComponentId::new(llmspell_hooks::ComponentType::Agent, "test".to_string()),
        );

        // Set memory usage above limit
        context
            .data
            .insert("memory_usage_bytes".to_string(), serde_json::json!(2000u64));

        let result = policy.execute(&mut context).await.unwrap();
        assert!(!result.should_continue());
        assert!(context.data.contains_key("resource_limit_exceeded"));
    }
    #[tokio::test]
    async fn test_token_limit_check() {
        let config = ResourceConfig {
            max_total_tokens: Some(1000),
            ..Default::default()
        };
        let policy = SessionResourcePolicy::new(config);

        let mut context = HookContext::new(
            HookPoint::SessionCheckpoint,
            ComponentId::new(llmspell_hooks::ComponentType::Agent, "test".to_string()),
        );

        // Set token usage above limit
        context
            .data
            .insert("total_tokens_used".to_string(), serde_json::json!(2000u64));

        let result = policy.execute(&mut context).await.unwrap();
        assert!(!result.should_continue());
    }
    #[tokio::test]
    async fn test_operation_count_update() {
        let config = ResourceConfig::default();
        let policy = SessionResourcePolicy::new(config);

        let mut context = HookContext::new(
            HookPoint::SessionCheckpoint,
            ComponentId::new(llmspell_hooks::ComponentType::Agent, "test".to_string()),
        );

        // Execute multiple times
        for _ in 0..3 {
            let result = policy.execute(&mut context).await.unwrap();
            assert!(result.should_continue());
        }

        // Check operation count was incremented
        let op_count = context
            .data
            .get("operation_count")
            .and_then(serde_json::Value::as_u64)
            .unwrap_or(0);
        assert_eq!(op_count, 3);
    }
}
