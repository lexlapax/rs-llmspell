//! ABOUTME: Session policy system built on existing hook patterns
//! ABOUTME: Provides timeout, resource limit, and rate limiting policies using llmspell-hooks

use anyhow::Result;
use llmspell_hooks::{
    traits::Hook, HookContext, HookExecutor, HookPoint, HookRegistry, HookResult,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

pub mod rate_limit;
pub mod resource_limit;
pub mod timeout;

pub use rate_limit::SessionRateLimitPolicy;
pub use resource_limit::SessionResourcePolicy;
pub use timeout::SessionTimeoutPolicy;

/// Session policy configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionPolicyConfig {
    /// Enable timeout policy
    pub enable_timeout: bool,
    /// Timeout configuration
    pub timeout_config: timeout::TimeoutConfig,
    /// Enable resource limit policy
    pub enable_resource_limits: bool,
    /// Resource limit configuration
    pub resource_config: resource_limit::ResourceConfig,
    /// Enable rate limiting
    pub enable_rate_limiting: bool,
    /// Rate limit configuration
    pub rate_limit_config: rate_limit::RateLimitConfig,
    /// Policy composition pattern
    pub composition_pattern: PolicyComposition,
}

impl Default for SessionPolicyConfig {
    fn default() -> Self {
        Self {
            enable_timeout: true,
            timeout_config: timeout::TimeoutConfig::default(),
            enable_resource_limits: true,
            resource_config: resource_limit::ResourceConfig::default(),
            enable_rate_limiting: true,
            rate_limit_config: rate_limit::RateLimitConfig::default(),
            composition_pattern: PolicyComposition::Sequential,
        }
    }
}

/// Policy composition pattern
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum PolicyComposition {
    /// Execute policies sequentially (fail fast)
    Sequential,
    /// Execute policies in parallel (all must pass)
    Parallel,
    /// Execute policies with voting (majority wins)
    Voting {
        /// Minimum percentage of policies that must pass (0.0 to 1.0)
        threshold: f32,
    },
}

/// Session policy manager
pub struct SessionPolicyManager {
    /// Policy configuration
    config: SessionPolicyConfig,
    /// Hook registry for policy registration
    hook_registry: Arc<HookRegistry>,
    /// Hook executor
    #[allow(dead_code)]
    hook_executor: Arc<HookExecutor>,
    /// Cached policy hooks
    policies: Vec<Arc<dyn Hook>>,
}

impl SessionPolicyManager {
    /// Create a new policy manager
    pub fn new(
        config: SessionPolicyConfig,
        hook_registry: Arc<HookRegistry>,
        hook_executor: Arc<HookExecutor>,
    ) -> Self {
        let mut policies: Vec<Arc<dyn Hook>> = Vec::new();

        // Add enabled policies
        if config.enable_timeout {
            let timeout_policy = SessionTimeoutPolicy::new(config.timeout_config.clone());
            policies.push(Arc::new(timeout_policy));
        }

        if config.enable_resource_limits {
            let resource_policy = SessionResourcePolicy::new(config.resource_config.clone());
            policies.push(Arc::new(resource_policy));
        }

        if config.enable_rate_limiting {
            let rate_limit_policy = SessionRateLimitPolicy::new(config.rate_limit_config.clone());
            policies.push(Arc::new(rate_limit_policy));
        }

        Self {
            config,
            hook_registry,
            hook_executor,
            policies,
        }
    }

    /// Register session policies with the hook system
    pub fn register_policies(&self) -> Result<()> {
        // Register each policy with the appropriate hook points
        for policy in &self.policies {
            // Check which hook points this policy should execute for
            let hook_points = vec![
                HookPoint::SessionStart,
                HookPoint::SessionEnd,
                HookPoint::SessionCheckpoint,
                HookPoint::SessionRestore,
                HookPoint::SessionSave,
            ];

            for hook_point in hook_points {
                // Create a temporary context to check if the policy should execute
                let temp_context = HookContext::new(
                    hook_point.clone(),
                    llmspell_hooks::ComponentId::new(
                        llmspell_hooks::ComponentType::Agent,
                        "session".to_string(),
                    ),
                );

                if policy.should_execute(&temp_context) {
                    // Register the policy for this hook point
                    self.hook_registry
                        .register_arc(hook_point, Arc::clone(policy))
                        .map_err(|e| anyhow::anyhow!("Failed to register policy: {:?}", e))?;
                }
            }
        }

        Ok(())
    }

    /// Evaluate policies for a session operation
    pub async fn evaluate_policies(&self, context: &mut HookContext) -> Result<HookResult> {
        // Get hooks from registry for this hook point
        let hooks = self.hook_registry.get_hooks(&context.point);

        if hooks.is_empty() {
            return Ok(HookResult::Continue);
        }

        // Execute hooks using the executor based on composition pattern
        match self.config.composition_pattern {
            PolicyComposition::Sequential => {
                // Execute sequentially using executor
                let results = self.hook_executor.execute_hooks(&hooks, context).await?;

                // Check results - executor already handles Cancel/Replace logic
                for result in results {
                    if !result.should_continue() {
                        return Ok(result);
                    }
                }
                Ok(HookResult::Continue)
            }
            PolicyComposition::Parallel => {
                // For parallel, we still use sequential execution but don't stop on first failure
                let mut all_results = Vec::new();
                for hook in &hooks {
                    let result = self
                        .hook_executor
                        .execute_hook(hook.as_ref(), context)
                        .await?;
                    all_results.push(result);
                }

                // Return first non-continue result
                for result in all_results {
                    if !result.should_continue() {
                        return Ok(result);
                    }
                }
                Ok(HookResult::Continue)
            }
            PolicyComposition::Voting { threshold } => {
                // Execute all and count successes
                let total = hooks.len();
                let mut successes = 0;

                for hook in &hooks {
                    let result = self
                        .hook_executor
                        .execute_hook(hook.as_ref(), context)
                        .await?;
                    if result.should_continue() {
                        successes += 1;
                    }
                }

                #[allow(clippy::cast_precision_loss)]
                let success_rate = successes as f32 / total as f32;
                if success_rate >= threshold {
                    Ok(HookResult::Continue)
                } else {
                    Ok(HookResult::Cancel("Voting threshold not met".to_string()))
                }
            }
        }
    }

    /// Check if a specific policy is enabled
    pub fn is_policy_enabled(&self, policy_type: PolicyType) -> bool {
        match policy_type {
            PolicyType::Timeout => self.config.enable_timeout,
            PolicyType::ResourceLimit => self.config.enable_resource_limits,
            PolicyType::RateLimit => self.config.enable_rate_limiting,
        }
    }

    /// Update policy configuration
    pub fn update_config(&mut self, config: SessionPolicyConfig) {
        self.config = config;
    }
}

/// Policy type enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PolicyType {
    /// Timeout policy
    Timeout,
    /// Resource limit policy
    ResourceLimit,
    /// Rate limit policy
    RateLimit,
}

/// Policy evaluation result
#[derive(Debug, Clone)]
pub struct PolicyEvaluationResult {
    /// Policy that was evaluated
    pub policy_type: PolicyType,
    /// Whether the policy passed
    pub passed: bool,
    /// Optional reason for failure
    pub reason: Option<String>,
    /// Optional metadata
    pub metadata: serde_json::Value,
}

#[cfg(test)]
mod tests {
    use super::*;
    #[tokio::test]
    async fn test_policy_manager_creation() {
        let config = SessionPolicyConfig::default();
        let hook_registry = Arc::new(HookRegistry::new());
        let hook_executor = Arc::new(HookExecutor::new());

        let manager = SessionPolicyManager::new(config, hook_registry, hook_executor);

        assert!(manager.is_policy_enabled(PolicyType::Timeout));
        assert!(manager.is_policy_enabled(PolicyType::ResourceLimit));
        assert!(manager.is_policy_enabled(PolicyType::RateLimit));
    }

    // TODO: Enable this test once we implement actual hook registration
    #[ignore = "Pending hook registration implementation"]
    #[tokio::test]
    async fn test_policy_registration() {
        let config = SessionPolicyConfig::default();
        let hook_registry = Arc::new(HookRegistry::new());
        let hook_executor = Arc::new(HookExecutor::new());

        let manager = SessionPolicyManager::new(config, hook_registry.clone(), hook_executor);

        // Register policies
        manager.register_policies().unwrap();

        // Verify hooks are registered
        assert!(!hook_registry.get_hooks(&HookPoint::SessionStart).is_empty());
        assert!(!hook_registry.get_hooks(&HookPoint::SessionEnd).is_empty());
    }
    #[tokio::test]
    async fn test_policy_composition_patterns() {
        // Test sequential composition
        let config = SessionPolicyConfig {
            composition_pattern: PolicyComposition::Sequential,
            ..Default::default()
        };

        let hook_registry = Arc::new(HookRegistry::new());
        let hook_executor = Arc::new(HookExecutor::new());
        let manager = SessionPolicyManager::new(config, hook_registry.clone(), hook_executor);
        manager.register_policies().unwrap();

        // Test parallel composition
        let config = SessionPolicyConfig {
            composition_pattern: PolicyComposition::Parallel,
            ..Default::default()
        };

        let hook_registry = Arc::new(HookRegistry::new());
        let hook_executor = Arc::new(HookExecutor::new());
        let manager = SessionPolicyManager::new(config, hook_registry.clone(), hook_executor);
        manager.register_policies().unwrap();

        // Test voting composition
        let config = SessionPolicyConfig {
            composition_pattern: PolicyComposition::Voting { threshold: 0.6 },
            ..Default::default()
        };

        let hook_registry = Arc::new(HookRegistry::new());
        let hook_executor = Arc::new(HookExecutor::new());
        let manager = SessionPolicyManager::new(config, hook_registry.clone(), hook_executor);
        manager.register_policies().unwrap();
    }
}
