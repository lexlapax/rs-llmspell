// ABOUTME: CostTrackingHook implementation for AI/ML operation cost monitoring and budgeting
// ABOUTME: Provides multi-provider pricing models, cost aggregation, and budget alerting

use crate::context::HookContext;
use crate::result::HookResult;
use crate::traits::{Hook, MetricHook};
use crate::types::{ComponentId, HookMetadata, Language, Priority};
use anyhow::Result;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

/// Token usage information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenUsage {
    /// Input/prompt tokens
    pub input_tokens: u64,
    /// Output/completion tokens
    pub output_tokens: u64,
    /// Total tokens (input + output)
    pub total_tokens: u64,
    /// Model name/identifier
    pub model: String,
    /// Provider name (OpenAI, Anthropic, etc.)
    pub provider: String,
}

/// Cost breakdown
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CostBreakdown {
    /// Input token cost
    pub input_cost: f64,
    /// Output token cost
    pub output_cost: f64,
    /// Total cost
    pub total_cost: f64,
    /// Currency code (USD, EUR, etc.)
    pub currency: String,
    /// Timestamp of calculation
    pub timestamp: DateTime<Utc>,
}

/// Provider pricing information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderPricing {
    /// Provider name
    pub provider: String,
    /// Model pricing configurations
    pub models: HashMap<String, ModelPricing>,
    /// Default currency
    pub default_currency: String,
}

/// Model-specific pricing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelPricing {
    /// Cost per 1K input tokens
    pub input_cost_per_1k: f64,
    /// Cost per 1K output tokens
    pub output_cost_per_1k: f64,
    /// Minimum charge per request (if any)
    pub minimum_charge: Option<f64>,
    /// Maximum tokens allowed
    pub max_tokens: Option<u64>,
}

/// Budget alert configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BudgetAlert {
    /// Alert threshold amount
    pub threshold: f64,
    /// Alert level
    pub level: AlertLevel,
    /// Whether to block operations when exceeded
    pub block_on_exceed: bool,
    /// Custom message for alert
    pub message: Option<String>,
}

/// Alert severity levels
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum AlertLevel {
    Info,
    Warning,
    Critical,
}

/// Cost tracking configuration
#[derive(Debug, Clone)]
pub struct CostTrackingConfig {
    /// Provider pricing configurations
    pub providers: HashMap<String, ProviderPricing>,
    /// Budget alerts (sorted by threshold)
    pub budget_alerts: Vec<BudgetAlert>,
    /// Default currency for cost calculations
    pub default_currency: String,
    /// Whether to track costs per user
    pub track_per_user: bool,
    /// Whether to track costs per component
    pub track_per_component: bool,
    /// Cost aggregation window
    pub aggregation_window: Duration,
    /// Whether to emit cost events
    pub emit_events: bool,
}

impl Default for CostTrackingConfig {
    fn default() -> Self {
        let mut providers = HashMap::new();

        // OpenAI pricing (as of 2024)
        let mut openai_models = HashMap::new();
        openai_models.insert(
            "gpt-4".to_string(),
            ModelPricing {
                input_cost_per_1k: 0.03,
                output_cost_per_1k: 0.06,
                minimum_charge: None,
                max_tokens: Some(8192),
            },
        );
        openai_models.insert(
            "gpt-3.5-turbo".to_string(),
            ModelPricing {
                input_cost_per_1k: 0.0005,
                output_cost_per_1k: 0.0015,
                minimum_charge: None,
                max_tokens: Some(4096),
            },
        );

        providers.insert(
            "openai".to_string(),
            ProviderPricing {
                provider: "openai".to_string(),
                models: openai_models,
                default_currency: "USD".to_string(),
            },
        );

        // Anthropic pricing (as of 2024)
        let mut anthropic_models = HashMap::new();
        anthropic_models.insert(
            "claude-3-opus".to_string(),
            ModelPricing {
                input_cost_per_1k: 0.015,
                output_cost_per_1k: 0.075,
                minimum_charge: None,
                max_tokens: Some(200000),
            },
        );
        anthropic_models.insert(
            "claude-3-sonnet".to_string(),
            ModelPricing {
                input_cost_per_1k: 0.003,
                output_cost_per_1k: 0.015,
                minimum_charge: None,
                max_tokens: Some(200000),
            },
        );

        providers.insert(
            "anthropic".to_string(),
            ProviderPricing {
                provider: "anthropic".to_string(),
                models: anthropic_models,
                default_currency: "USD".to_string(),
            },
        );

        Self {
            providers,
            budget_alerts: vec![
                BudgetAlert {
                    threshold: 10.0,
                    level: AlertLevel::Info,
                    block_on_exceed: false,
                    message: Some("Cost tracking: $10 spent".to_string()),
                },
                BudgetAlert {
                    threshold: 50.0,
                    level: AlertLevel::Warning,
                    block_on_exceed: false,
                    message: Some("Warning: $50 budget threshold reached".to_string()),
                },
                BudgetAlert {
                    threshold: 100.0,
                    level: AlertLevel::Critical,
                    block_on_exceed: true,
                    message: Some("Critical: $100 budget limit exceeded".to_string()),
                },
            ],
            default_currency: "USD".to_string(),
            track_per_user: true,
            track_per_component: true,
            aggregation_window: Duration::from_secs(3600), // 1 hour
            emit_events: true,
        }
    }
}

/// Cost tracking metrics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CostTrackingMetrics {
    pub total_requests: u64,
    pub total_input_tokens: u64,
    pub total_output_tokens: u64,
    pub total_cost: f64,
    pub costs_by_provider: HashMap<String, f64>,
    pub costs_by_model: HashMap<String, f64>,
    pub costs_by_component: HashMap<String, f64>,
    pub costs_by_user: HashMap<String, f64>,
    pub alerts_triggered: HashMap<String, u64>,
    pub operations_blocked: u64,
}

impl CostTrackingMetrics {
    pub fn average_cost_per_request(&self) -> f64 {
        if self.total_requests == 0 {
            0.0
        } else {
            self.total_cost / self.total_requests as f64
        }
    }

    pub fn average_tokens_per_request(&self) -> f64 {
        if self.total_requests == 0 {
            0.0
        } else {
            (self.total_input_tokens + self.total_output_tokens) as f64 / self.total_requests as f64
        }
    }
}

/// Cost history entry
type CostHistoryEntry = (DateTime<Utc>, f64);

/// Cost aggregator for tracking costs over time
#[derive(Debug)]
struct CostAggregator {
    /// Costs by component
    component_costs: Arc<parking_lot::RwLock<HashMap<String, f64>>>,
    /// Costs by user
    user_costs: Arc<parking_lot::RwLock<HashMap<String, f64>>>,
    /// Total cost
    total_cost: Arc<parking_lot::RwLock<f64>>,
    /// Cost history for time-based aggregation
    cost_history: Arc<parking_lot::RwLock<Vec<CostHistoryEntry>>>,
}

impl CostAggregator {
    fn new() -> Self {
        Self {
            component_costs: Arc::new(parking_lot::RwLock::new(HashMap::new())),
            user_costs: Arc::new(parking_lot::RwLock::new(HashMap::new())),
            total_cost: Arc::new(parking_lot::RwLock::new(0.0)),
            cost_history: Arc::new(parking_lot::RwLock::new(Vec::new())),
        }
    }

    fn add_cost(&self, component_id: &ComponentId, user_id: Option<&str>, cost: f64) {
        // Update component costs
        {
            let mut component_costs = self.component_costs.write();
            let component_key = format!("{:?}:{}", component_id.component_type, component_id.name);
            *component_costs.entry(component_key).or_insert(0.0) += cost;
        }

        // Update user costs if tracking per user
        if let Some(user) = user_id {
            let mut user_costs = self.user_costs.write();
            *user_costs.entry(user.to_string()).or_insert(0.0) += cost;
        }

        // Update total cost
        {
            let mut total = self.total_cost.write();
            *total += cost;
        }

        // Add to history
        {
            let mut history = self.cost_history.write();
            history.push((Utc::now(), cost));
        }
    }

    fn get_total(&self) -> f64 {
        *self.total_cost.read()
    }

    fn get_component_total(&self, component_id: &ComponentId) -> f64 {
        let component_key = format!("{:?}:{}", component_id.component_type, component_id.name);
        self.component_costs
            .read()
            .get(&component_key)
            .copied()
            .unwrap_or(0.0)
    }

    fn get_user_total(&self, user_id: &str) -> f64 {
        self.user_costs.read().get(user_id).copied().unwrap_or(0.0)
    }

    fn get_window_total(&self, window: Duration) -> f64 {
        let cutoff = Utc::now() - chrono::Duration::from_std(window).unwrap();
        self.cost_history
            .read()
            .iter()
            .filter(|(timestamp, _)| *timestamp > cutoff)
            .map(|(_, cost)| cost)
            .sum()
    }

    fn cleanup_old_history(&self, retention: Duration) {
        let cutoff = Utc::now() - chrono::Duration::from_std(retention).unwrap();
        let mut history = self.cost_history.write();
        history.retain(|(timestamp, _)| *timestamp > cutoff);
    }
}

/// Built-in cost tracking hook for AI/ML operations
pub struct CostTrackingHook {
    config: CostTrackingConfig,
    aggregator: Arc<CostAggregator>,
    metrics: Arc<std::sync::RwLock<CostTrackingMetrics>>,
    metadata: HookMetadata,
}

impl CostTrackingHook {
    /// Create a new cost tracking hook with default configuration
    pub fn new() -> Self {
        Self::with_config(CostTrackingConfig::default())
    }

    /// Create a new cost tracking hook with custom configuration
    pub fn with_config(mut config: CostTrackingConfig) -> Self {
        // Sort budget alerts by threshold
        config
            .budget_alerts
            .sort_by(|a, b| a.threshold.partial_cmp(&b.threshold).unwrap());

        Self {
            config,
            aggregator: Arc::new(CostAggregator::new()),
            metrics: Arc::new(std::sync::RwLock::new(CostTrackingMetrics::default())),
            metadata: HookMetadata {
                name: "CostTrackingHook".to_string(),
                description: Some("Built-in hook for AI/ML operation cost monitoring".to_string()),
                priority: Priority::LOW, // Run after operations complete
                language: Language::Native,
                tags: vec![
                    "builtin".to_string(),
                    "cost-tracking".to_string(),
                    "monitoring".to_string(),
                ],
                version: "1.0.0".to_string(),
            },
        }
    }

    /// Add custom provider pricing
    pub fn with_provider_pricing(mut self, provider: String, pricing: ProviderPricing) -> Self {
        self.config.providers.insert(provider, pricing);
        self
    }

    /// Add budget alert
    pub fn with_budget_alert(mut self, alert: BudgetAlert) -> Self {
        self.config.budget_alerts.push(alert);
        // Keep alerts sorted by threshold
        self.config
            .budget_alerts
            .sort_by(|a, b| a.threshold.partial_cmp(&b.threshold).unwrap());
        self
    }

    /// Get cost tracking metrics
    pub fn metrics(&self) -> CostTrackingMetrics {
        self.metrics.read().unwrap().clone()
    }

    /// Reset metrics
    pub fn reset_metrics(&self) {
        let mut metrics = self.metrics.write().unwrap();
        *metrics = CostTrackingMetrics::default();
    }

    /// Get total cost
    pub fn total_cost(&self) -> f64 {
        self.aggregator.get_total()
    }

    /// Get component cost
    pub fn component_cost(&self, component_id: &ComponentId) -> f64 {
        self.aggregator.get_component_total(component_id)
    }

    /// Get user cost
    pub fn user_cost(&self, user_id: &str) -> f64 {
        self.aggregator.get_user_total(user_id)
    }

    /// Calculate cost from token usage
    fn calculate_cost(&self, usage: &TokenUsage) -> Result<CostBreakdown> {
        let provider = self
            .config
            .providers
            .get(&usage.provider.to_lowercase())
            .ok_or_else(|| anyhow::anyhow!("Unknown provider: {}", usage.provider))?;

        let model_pricing = provider
            .models
            .get(&usage.model.to_lowercase())
            .ok_or_else(|| {
                anyhow::anyhow!(
                    "Unknown model: {} for provider {}",
                    usage.model,
                    usage.provider
                )
            })?;

        let input_cost = (usage.input_tokens as f64 / 1000.0) * model_pricing.input_cost_per_1k;
        let output_cost = (usage.output_tokens as f64 / 1000.0) * model_pricing.output_cost_per_1k;
        let mut total_cost = input_cost + output_cost;

        // Apply minimum charge if applicable
        if let Some(min_charge) = model_pricing.minimum_charge {
            total_cost = total_cost.max(min_charge);
        }

        Ok(CostBreakdown {
            input_cost,
            output_cost,
            total_cost,
            currency: provider.default_currency.clone(),
            timestamp: Utc::now(),
        })
    }

    /// Check budget alerts
    fn check_budget_alerts(
        &self,
        total_cost: f64,
        context: &mut HookContext,
    ) -> Option<HookResult> {
        for alert in &self.config.budget_alerts {
            if total_cost >= alert.threshold {
                // Update metrics
                {
                    let mut metrics = self.metrics.write().unwrap();
                    let alert_key = format!("{:?}:{}", alert.level, alert.threshold);
                    *metrics.alerts_triggered.entry(alert_key).or_insert(0) += 1;
                }

                // Add alert metadata
                context
                    .insert_metadata("cost_alert_level".to_string(), format!("{:?}", alert.level));
                context.insert_metadata(
                    "cost_alert_threshold".to_string(),
                    alert.threshold.to_string(),
                );
                context.insert_metadata("cost_alert_total".to_string(), total_cost.to_string());

                if let Some(message) = &alert.message {
                    context.insert_metadata("cost_alert_message".to_string(), message.clone());
                }

                // Log alert
                match alert.level {
                    AlertLevel::Info => log::info!(
                        "Cost alert: ${:.2} (threshold: ${:.2})",
                        total_cost,
                        alert.threshold
                    ),
                    AlertLevel::Warning => log::warn!(
                        "Cost warning: ${:.2} (threshold: ${:.2})",
                        total_cost,
                        alert.threshold
                    ),
                    AlertLevel::Critical => log::error!(
                        "Cost critical: ${:.2} (threshold: ${:.2})",
                        total_cost,
                        alert.threshold
                    ),
                }

                // Block if configured
                if alert.block_on_exceed {
                    let mut metrics = self.metrics.write().unwrap();
                    metrics.operations_blocked += 1;

                    return Some(HookResult::Cancel(format!(
                        "Operation blocked: Cost limit exceeded (${:.2} >= ${:.2})",
                        total_cost, alert.threshold
                    )));
                }
            }
        }
        None
    }
}

impl Default for CostTrackingHook {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Hook for CostTrackingHook {
    async fn execute(&self, context: &mut HookContext) -> Result<HookResult> {
        // Only track costs after agent execution
        if context.point != crate::types::HookPoint::AfterAgentExecution {
            return Ok(HookResult::Continue);
        }

        // Extract token usage from context
        let usage = if let Some(usage_value) = context.data.get("token_usage") {
            match serde_json::from_value::<TokenUsage>(usage_value.clone()) {
                Ok(usage) => usage,
                Err(e) => {
                    log::debug!("CostTrackingHook: Failed to parse token usage: {}", e);
                    return Ok(HookResult::Continue);
                }
            }
        } else {
            // No token usage data
            return Ok(HookResult::Continue);
        };

        // Calculate cost
        let cost_breakdown = match self.calculate_cost(&usage) {
            Ok(breakdown) => breakdown,
            Err(e) => {
                log::warn!("CostTrackingHook: Failed to calculate cost: {}", e);
                return Ok(HookResult::Continue);
            }
        };

        // Extract user ID if tracking per user
        let user_id = if self.config.track_per_user {
            context.get_metadata("user_id")
        } else {
            None
        };

        // Add cost to aggregator
        self.aggregator
            .add_cost(&context.component_id, user_id, cost_breakdown.total_cost);

        // Update metrics
        {
            let mut metrics = self.metrics.write().unwrap();
            metrics.total_requests += 1;
            metrics.total_input_tokens += usage.input_tokens;
            metrics.total_output_tokens += usage.output_tokens;
            metrics.total_cost += cost_breakdown.total_cost;

            // Update provider costs
            *metrics
                .costs_by_provider
                .entry(usage.provider.clone())
                .or_insert(0.0) += cost_breakdown.total_cost;

            // Update model costs
            *metrics
                .costs_by_model
                .entry(format!("{}/{}", usage.provider, usage.model))
                .or_insert(0.0) += cost_breakdown.total_cost;

            // Update component costs
            if self.config.track_per_component {
                let component_key = format!(
                    "{:?}:{}",
                    context.component_id.component_type, context.component_id.name
                );
                *metrics
                    .costs_by_component
                    .entry(component_key)
                    .or_insert(0.0) += cost_breakdown.total_cost;
            }

            // Update user costs
            if let Some(user) = user_id {
                *metrics.costs_by_user.entry(user.to_string()).or_insert(0.0) +=
                    cost_breakdown.total_cost;
            }
        }

        // Add cost information to context
        context.insert_metadata(
            "cost_input".to_string(),
            format!("{:.6}", cost_breakdown.input_cost),
        );
        context.insert_metadata(
            "cost_output".to_string(),
            format!("{:.6}", cost_breakdown.output_cost),
        );
        context.insert_metadata(
            "cost_total".to_string(),
            format!("{:.6}", cost_breakdown.total_cost),
        );
        context.insert_metadata("cost_currency".to_string(), cost_breakdown.currency);

        // Check budget alerts
        let window_total = self
            .aggregator
            .get_window_total(self.config.aggregation_window);
        if let Some(result) = self.check_budget_alerts(window_total, context) {
            return Ok(result);
        }

        // Clean up old history periodically
        self.aggregator
            .cleanup_old_history(Duration::from_secs(86400)); // 24 hours

        log::debug!(
            "CostTrackingHook: Tracked cost ${:.6} for {} ({} tokens)",
            cost_breakdown.total_cost,
            usage.model,
            usage.total_tokens
        );

        Ok(HookResult::Continue)
    }

    fn metadata(&self) -> HookMetadata {
        self.metadata.clone()
    }

    fn should_execute(&self, _context: &HookContext) -> bool {
        true
    }
}

#[async_trait]
impl MetricHook for CostTrackingHook {
    async fn record_pre_execution(&self, context: &HookContext) -> Result<()> {
        log::trace!(
            "CostTrackingHook: Pre-execution for hook point {:?}",
            context.point
        );
        Ok(())
    }

    async fn record_post_execution(
        &self,
        _context: &HookContext,
        _result: &HookResult,
        _duration: Duration,
    ) -> Result<()> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{ComponentId, ComponentType, HookPoint};

    fn create_test_context_with_usage(
        provider: &str,
        model: &str,
        input: u64,
        output: u64,
    ) -> HookContext {
        let component_id = ComponentId::new(ComponentType::Agent, "test-agent".to_string());
        let mut context = HookContext::new(HookPoint::AfterAgentExecution, component_id);

        let usage = TokenUsage {
            input_tokens: input,
            output_tokens: output,
            total_tokens: input + output,
            model: model.to_string(),
            provider: provider.to_string(),
        };

        context.data.insert(
            "token_usage".to_string(),
            serde_json::to_value(usage).unwrap(),
        );
        context
    }

    #[tokio::test]
    async fn test_cost_tracking_hook_basic() {
        let hook = CostTrackingHook::new();
        let mut context = create_test_context_with_usage("openai", "gpt-3.5-turbo", 1000, 500);

        let result = hook.execute(&mut context).await.unwrap();
        assert!(matches!(result, HookResult::Continue));

        // Check cost metadata
        assert!(context.get_metadata("cost_total").is_some());
        assert!(context.get_metadata("cost_input").is_some());
        assert!(context.get_metadata("cost_output").is_some());
        assert_eq!(context.get_metadata("cost_currency").unwrap(), "USD");
    }

    #[tokio::test]
    async fn test_cost_calculation() {
        let hook = CostTrackingHook::new();

        // Test GPT-3.5-turbo pricing
        let mut context = create_test_context_with_usage("openai", "gpt-3.5-turbo", 1000, 500);
        let _ = hook.execute(&mut context).await.unwrap();

        let total_cost = context
            .get_metadata("cost_total")
            .unwrap()
            .parse::<f64>()
            .unwrap();
        // 1000 input tokens * $0.0005/1K + 500 output tokens * $0.0015/1K
        let expected = 0.0005 + 0.00075;
        assert!((total_cost - expected).abs() < 0.000001);
    }

    #[tokio::test]
    async fn test_budget_alerts() {
        let hook = CostTrackingHook::new()
            .with_budget_alert(BudgetAlert {
                threshold: 0.001,
                level: AlertLevel::Warning,
                block_on_exceed: false,
                message: Some("Test warning".to_string()),
            })
            .with_budget_alert(BudgetAlert {
                threshold: 0.01,
                level: AlertLevel::Critical,
                block_on_exceed: true,
                message: Some("Test block".to_string()),
            });

        // First request should trigger warning but not block
        let mut context = create_test_context_with_usage("openai", "gpt-3.5-turbo", 2000, 1000);
        let result = hook.execute(&mut context).await.unwrap();
        assert!(matches!(result, HookResult::Continue));
        assert_eq!(context.get_metadata("cost_alert_level").unwrap(), "Warning");

        // Accumulate more cost to trigger blocking
        for _ in 0..10 {
            let mut context = create_test_context_with_usage("openai", "gpt-3.5-turbo", 2000, 1000);
            let result = hook.execute(&mut context).await.unwrap();
            if matches!(result, HookResult::Cancel(_)) {
                assert_eq!(
                    context.get_metadata("cost_alert_level").unwrap(),
                    "Critical"
                );
                break;
            }
        }
    }

    #[tokio::test]
    async fn test_cost_aggregation() {
        let hook = CostTrackingHook::new();

        // Multiple requests
        for i in 0..5 {
            let mut context = create_test_context_with_usage("openai", "gpt-3.5-turbo", 1000, 500);
            if i % 2 == 0 {
                context.insert_metadata("user_id".to_string(), "user1".to_string());
            } else {
                context.insert_metadata("user_id".to_string(), "user2".to_string());
            }
            let _ = hook.execute(&mut context).await.unwrap();
        }

        let metrics = hook.metrics();
        assert_eq!(metrics.total_requests, 5);
        assert_eq!(metrics.total_input_tokens, 5000);
        assert_eq!(metrics.total_output_tokens, 2500);
        assert!(metrics.total_cost > 0.0);
        assert_eq!(metrics.costs_by_user.len(), 2);
    }

    #[tokio::test]
    async fn test_unknown_provider() {
        let hook = CostTrackingHook::new();
        let mut context =
            create_test_context_with_usage("unknown_provider", "some-model", 1000, 500);

        let result = hook.execute(&mut context).await.unwrap();
        assert!(matches!(result, HookResult::Continue));

        // Should not have cost metadata since provider is unknown
        assert!(context.get_metadata("cost_total").is_none());
    }

    #[tokio::test]
    async fn test_custom_provider_pricing() {
        let custom_pricing = ProviderPricing {
            provider: "custom".to_string(),
            models: {
                let mut models = HashMap::new();
                models.insert(
                    "custom-model".to_string(),
                    ModelPricing {
                        input_cost_per_1k: 0.01,
                        output_cost_per_1k: 0.02,
                        minimum_charge: Some(0.001),
                        max_tokens: Some(10000),
                    },
                );
                models
            },
            default_currency: "EUR".to_string(),
        };

        let hook =
            CostTrackingHook::new().with_provider_pricing("custom".to_string(), custom_pricing);

        let mut context = create_test_context_with_usage("custom", "custom-model", 100, 50);
        let _ = hook.execute(&mut context).await.unwrap();

        let total_cost = context
            .get_metadata("cost_total")
            .unwrap()
            .parse::<f64>()
            .unwrap();
        // Should use minimum charge since calculated cost would be 0.001 + 0.001 = 0.002
        assert_eq!(total_cost, 0.002);
        assert_eq!(context.get_metadata("cost_currency").unwrap(), "EUR");
    }

    #[test]
    fn test_hook_metadata() {
        let hook = CostTrackingHook::new();
        let metadata = hook.metadata();

        assert_eq!(metadata.name, "CostTrackingHook");
        assert!(metadata.description.is_some());
        assert_eq!(metadata.priority, Priority::LOW);
        assert_eq!(metadata.language, Language::Native);
        assert!(metadata.tags.contains(&"builtin".to_string()));
        assert!(metadata.tags.contains(&"cost-tracking".to_string()));
    }

    #[test]
    fn test_cost_metrics_calculations() {
        let mut metrics = CostTrackingMetrics::default();
        metrics.total_requests = 100;
        metrics.total_cost = 50.0;
        metrics.total_input_tokens = 50000;
        metrics.total_output_tokens = 25000;

        assert_eq!(metrics.average_cost_per_request(), 0.5);
        assert_eq!(metrics.average_tokens_per_request(), 750.0);
    }

    #[test]
    fn test_alert_sorting() {
        let mut config = CostTrackingConfig::default();
        config.budget_alerts.clear();

        // Add alerts out of order
        config.budget_alerts.push(BudgetAlert {
            threshold: 50.0,
            level: AlertLevel::Warning,
            block_on_exceed: false,
            message: None,
        });
        config.budget_alerts.push(BudgetAlert {
            threshold: 10.0,
            level: AlertLevel::Info,
            block_on_exceed: false,
            message: None,
        });
        config.budget_alerts.push(BudgetAlert {
            threshold: 100.0,
            level: AlertLevel::Critical,
            block_on_exceed: true,
            message: None,
        });

        let hook = CostTrackingHook::with_config(config);

        // Alerts should be sorted by threshold
        assert_eq!(hook.config.budget_alerts[0].threshold, 10.0);
        assert_eq!(hook.config.budget_alerts[1].threshold, 50.0);
        assert_eq!(hook.config.budget_alerts[2].threshold, 100.0);
    }
}
