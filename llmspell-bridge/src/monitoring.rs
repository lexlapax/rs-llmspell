//! ABOUTME: Bridge monitoring implementations
//! ABOUTME: Provides concrete implementations of monitoring traits for the bridge

use async_trait::async_trait;
use llmspell_agents::monitoring::{
    ComponentHealth, HealthCheck, HealthCheckResult, HealthIndicator, HealthStatus,
};
use llmspell_core::{ComponentMetadata, Result};
use std::collections::HashMap;
use std::time::Duration;

/// Concrete implementation of `HealthCheck` trait
pub struct HealthCheckImpl {
    metadata: ComponentMetadata,
}

impl HealthCheckImpl {
    #[must_use]
    pub fn new() -> Self {
        Self {
            metadata: ComponentMetadata::new(
                "HealthCheckImpl".to_string(),
                "Bridge health check implementation".to_string(),
            ),
        }
    }
}

impl Default for HealthCheckImpl {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl HealthCheck for HealthCheckImpl {
    async fn check_health(&self) -> Result<Vec<HealthIndicator>> {
        // Mock health check implementation
        Ok(vec![HealthIndicator {
            name: "bridge_status".to_string(),
            status: HealthStatus::Healthy,
            message: Some("Bridge is functioning normally".to_string()),
            details: HashMap::new(),
            last_check: chrono::Utc::now(),
        }])
    }

    fn metadata(&self) -> &ComponentMetadata {
        &self.metadata
    }
}

/// Mock check agent health for bridge
pub async fn check_agent_health(_agent_id: &str) -> Result<HealthCheckResult> {
    let mut components = HashMap::new();
    components.insert(
        "agent_health".to_string(),
        ComponentHealth {
            metadata: ComponentMetadata::new(
                "agent_health".to_string(),
                "Agent health component".to_string(),
            ),
            status: HealthStatus::Healthy,
            indicators: vec![HealthIndicator {
                name: "agent_status".to_string(),
                status: HealthStatus::Healthy,
                message: Some("Agent is functioning normally".to_string()),
                details: HashMap::new(),
                last_check: chrono::Utc::now(),
            }],
            last_check: chrono::Utc::now(),
            check_duration: Duration::from_millis(5),
        },
    );

    Ok(HealthCheckResult {
        overall_status: HealthStatus::Healthy,
        components,
        total_duration: Duration::from_millis(10),
        timestamp: chrono::Utc::now(),
    })
}
