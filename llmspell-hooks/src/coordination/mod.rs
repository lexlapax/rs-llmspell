//! ABOUTME: Cross-component hook coordination for complex execution chains
//! ABOUTME: Provides dependency management, event correlation, and performance isolation

//! # Cross-Component Hook Coordination
//!
//! This module provides coordination between hooks executed by different components
//! (agents, tools, workflows) in complex execution chains. It ensures proper
//! ordering, context propagation, and performance isolation across component boundaries.
//!
//! ## Features
//!
//! - **Dependency Management**: Hook execution ordering based on dependencies
//! - **Event Correlation**: Tracking events across component boundaries
//! - **Context Propagation**: Maintaining hook context through execution chains
//! - **Performance Isolation**: Preventing component hook interference
//!
//! ## Example
//!
//! ```rust,no_run
//! use llmspell_hooks::coordination::{CrossComponentCoordinator, ExecutionChain};
//! use llmspell_hooks::{HookContext, ComponentId, ComponentType, HookPoint};
//!
//! # async fn example() -> anyhow::Result<()> {
//! let mut coordinator = CrossComponentCoordinator::new();
//!
//! // Create execution chain: Agent -> Tool -> Workflow
//! let agent_id = ComponentId::new(ComponentType::Agent, "gpt-4".to_string());
//! let tool_id = ComponentId::new(ComponentType::Tool, "calculator".to_string());
//! let workflow_id = ComponentId::new(ComponentType::Workflow, "analysis".to_string());
//!
//! let chain = ExecutionChain::new()
//!     .add_component(agent_id.clone())
//!     .add_component(tool_id.clone())
//!     .add_component(workflow_id.clone());
//!
//! coordinator.register_chain("agent-tool-workflow", chain).await?;
//! # Ok(())
//! # }
//! ```

pub mod dependency_graph;
pub mod event_correlation;

use crate::{ComponentId, HookContext, HookResult, PerformanceMetrics};
use anyhow::Result;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{Mutex, RwLock};
use tracing::{debug, info};
use uuid::Uuid;

pub use dependency_graph::{DependencyGraph, DependencyNode, ExecutionOrder};
pub use event_correlation::{CorrelationId, EventCorrelator, EventTrace};

/// Cross-component hook coordinator for managing complex execution chains
#[derive(Debug)]
pub struct CrossComponentCoordinator {
    /// Active execution chains by name
    chains: RwLock<HashMap<String, ExecutionChain>>,
    /// Event correlator for tracking cross-component events
    correlator: Arc<EventCorrelator>,
    /// Dependency graph for hook execution ordering
    #[allow(dead_code)]
    dependency_graph: Arc<Mutex<DependencyGraph>>,
    /// Performance metrics by component
    performance_metrics: RwLock<HashMap<ComponentId, PerformanceMetrics>>,
    /// Configuration settings
    config: CoordinatorConfig,
}

/// Configuration for the cross-component coordinator
#[derive(Debug, Clone)]
pub struct CoordinatorConfig {
    /// Maximum execution time per component in a chain
    pub max_component_execution_time: Duration,
    /// Maximum total chain execution time
    pub max_chain_execution_time: Duration,
    /// Enable performance isolation between components
    pub enable_performance_isolation: bool,
    /// Enable event correlation tracking
    pub enable_event_correlation: bool,
    /// Maximum number of active chains
    pub max_active_chains: usize,
}

impl Default for CoordinatorConfig {
    fn default() -> Self {
        Self {
            max_component_execution_time: Duration::from_secs(30),
            max_chain_execution_time: Duration::from_secs(300), // 5 minutes
            enable_performance_isolation: true,
            enable_event_correlation: true,
            max_active_chains: 100,
        }
    }
}

/// Represents an execution chain of components
#[derive(Debug, Clone)]
pub struct ExecutionChain {
    /// Unique identifier for this chain
    pub id: Uuid,
    /// Name of the execution chain
    pub name: String,
    /// Components in execution order
    pub components: Vec<ComponentId>,
    /// Chain-level metadata
    pub metadata: HashMap<String, String>,
    /// Creation timestamp
    pub created_at: Instant,
    /// Current execution state
    pub state: ChainState,
}

/// State of an execution chain
#[derive(Debug, Clone, PartialEq)]
pub enum ChainState {
    /// Chain is registered but not yet executing
    Pending,
    /// Chain is currently executing
    Executing {
        /// Current component index
        current_component: usize,
        /// Execution start time
        started_at: Instant,
        /// Correlation ID for tracking
        correlation_id: CorrelationId,
    },
    /// Chain completed successfully
    Completed {
        /// Total execution duration
        duration: Duration,
        /// Correlation ID for tracking
        correlation_id: CorrelationId,
    },
    /// Chain failed during execution
    Failed {
        /// Component where failure occurred
        failed_at_component: usize,
        /// Error message
        error: String,
        /// Partial execution duration
        duration: Duration,
        /// Correlation ID for tracking
        correlation_id: CorrelationId,
    },
}

/// Context for cross-component hook execution
#[derive(Debug, Clone)]
pub struct CrossComponentContext {
    /// Base hook context
    pub hook_context: HookContext,
    /// Execution chain information
    pub chain_id: Uuid,
    /// Current component in the chain
    pub current_component: ComponentId,
    /// Position in the execution chain
    pub chain_position: usize,
    /// Total components in chain
    pub chain_length: usize,
    /// Correlation ID for event tracking
    pub correlation_id: CorrelationId,
    /// Propagated data from previous components
    pub propagated_data: HashMap<String, serde_json::Value>,
    /// Performance metrics from previous components
    pub previous_metrics: Vec<PerformanceMetrics>,
}

impl Default for CrossComponentCoordinator {
    fn default() -> Self {
        Self::with_config(CoordinatorConfig::default())
    }
}

impl CrossComponentCoordinator {
    /// Creates a new cross-component coordinator
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates a new coordinator with custom configuration
    pub fn with_config(config: CoordinatorConfig) -> Self {
        Self {
            chains: RwLock::new(HashMap::new()),
            correlator: Arc::new(EventCorrelator::new()),
            dependency_graph: Arc::new(Mutex::new(DependencyGraph::new())),
            performance_metrics: RwLock::new(HashMap::new()),
            config,
        }
    }

    /// Registers a new execution chain
    pub async fn register_chain(
        &self,
        name: impl Into<String>,
        mut chain: ExecutionChain,
    ) -> Result<Uuid> {
        let name = name.into();
        chain.name = name.clone();

        let mut chains = self.chains.write().await;

        if chains.len() >= self.config.max_active_chains {
            return Err(anyhow::anyhow!(
                "Maximum number of active chains ({}) exceeded",
                self.config.max_active_chains
            ));
        }

        let chain_id = chain.id;
        chains.insert(name.clone(), chain);

        info!(
            chain_id = %chain_id,
            chain_name = %name,
            "Registered cross-component execution chain"
        );

        Ok(chain_id)
    }

    /// Starts execution of a registered chain
    pub async fn start_chain_execution(
        &self,
        chain_name: &str,
        _initial_context: HookContext,
    ) -> Result<CorrelationId> {
        let mut chains = self.chains.write().await;
        let chain = chains
            .get_mut(chain_name)
            .ok_or_else(|| anyhow::anyhow!("Chain '{}' not found", chain_name))?;

        if chain.state != ChainState::Pending {
            return Err(anyhow::anyhow!(
                "Chain '{}' is not in pending state: {:?}",
                chain_name,
                chain.state
            ));
        }

        let correlation_id = self.correlator.create_correlation().await;
        let now = Instant::now();

        chain.state = ChainState::Executing {
            current_component: 0,
            started_at: now,
            correlation_id: correlation_id.clone(),
        };

        info!(
            chain_name = %chain_name,
            correlation_id = %correlation_id,
            components = ?chain.components,
            "Started cross-component chain execution"
        );

        // Initialize event correlation for this chain
        self.correlator
            .start_chain_trace(correlation_id.clone(), chain.components.clone())
            .await?;

        Ok(correlation_id)
    }

    /// Executes hooks for the next component in a chain
    pub async fn execute_next_component(
        &self,
        correlation_id: &CorrelationId,
        hook_context: HookContext,
    ) -> Result<(HookResult, Option<CrossComponentContext>)> {
        // Implementation for executing the next component in the chain
        // This will be implemented in subsequent subtasks

        debug!(
            correlation_id = %correlation_id,
            component = ?hook_context.component_id,
            "Executing next component in cross-component chain"
        );

        // For now, return a basic response
        Ok((HookResult::Continue, None))
    }

    /// Gets the current state of a chain
    pub async fn get_chain_state(&self, chain_name: &str) -> Option<ChainState> {
        let chains = self.chains.read().await;
        chains.get(chain_name).map(|chain| chain.state.clone())
    }

    /// Gets performance metrics for a component
    pub async fn get_component_metrics(
        &self,
        component_id: &ComponentId,
    ) -> Option<PerformanceMetrics> {
        let metrics = self.performance_metrics.read().await;
        metrics.get(component_id).cloned()
    }

    /// Removes a completed or failed chain
    pub async fn cleanup_chain(&self, chain_name: &str) -> Result<()> {
        let mut chains = self.chains.write().await;
        if let Some(chain) = chains.remove(chain_name) {
            info!(
                chain_id = %chain.id,
                chain_name = %chain_name,
                final_state = ?chain.state,
                "Cleaned up execution chain"
            );
        }
        Ok(())
    }
}

impl ExecutionChain {
    /// Creates a new execution chain
    pub fn new() -> Self {
        Self {
            id: Uuid::new_v4(),
            name: String::new(),
            components: Vec::new(),
            metadata: HashMap::new(),
            created_at: Instant::now(),
            state: ChainState::Pending,
        }
    }

    /// Adds a component to the execution chain
    pub fn add_component(mut self, component_id: ComponentId) -> Self {
        self.components.push(component_id);
        self
    }

    /// Sets the chain name
    pub fn with_name(mut self, name: impl Into<String>) -> Self {
        self.name = name.into();
        self
    }

    /// Adds metadata to the chain
    pub fn with_metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }

    /// Gets the next component in the chain
    pub fn get_next_component(&self, current_index: usize) -> Option<&ComponentId> {
        self.components.get(current_index + 1)
    }

    /// Checks if this is the last component in the chain
    pub fn is_last_component(&self, current_index: usize) -> bool {
        current_index >= self.components.len().saturating_sub(1)
    }
}

impl Default for ExecutionChain {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
#[cfg_attr(test_category = "hook")]
mod tests {
    use super::*;
    use crate::{ComponentType, HookPoint};

    #[cfg_attr(test_category = "unit")]
    #[tokio::test]
    async fn test_coordinator_creation() {
        let coordinator = CrossComponentCoordinator::new();

        // Coordinator should be created successfully
        assert_eq!(coordinator.chains.read().await.len(), 0);
    }

    #[cfg_attr(test_category = "unit")]
    #[tokio::test]
    async fn test_chain_registration() {
        let coordinator = CrossComponentCoordinator::new();

        let agent_id = ComponentId::new(ComponentType::Agent, "test-agent".to_string());
        let tool_id = ComponentId::new(ComponentType::Tool, "test-tool".to_string());

        let chain = ExecutionChain::new()
            .with_name("test-chain")
            .add_component(agent_id)
            .add_component(tool_id);

        let chain_id = coordinator
            .register_chain("test-chain", chain)
            .await
            .expect("Should register chain successfully");

        assert!(!chain_id.is_nil());
        assert_eq!(coordinator.chains.read().await.len(), 1);
    }

    #[cfg_attr(test_category = "unit")]
    #[tokio::test]
    async fn test_chain_execution_start() {
        let coordinator = CrossComponentCoordinator::new();

        let agent_id = ComponentId::new(ComponentType::Agent, "test-agent".to_string());
        let chain = ExecutionChain::new()
            .with_name("test-chain")
            .add_component(agent_id.clone());

        coordinator
            .register_chain("test-chain", chain)
            .await
            .expect("Should register chain");

        let context = HookContext::new(HookPoint::BeforeAgentInit, agent_id);
        let correlation_id = coordinator
            .start_chain_execution("test-chain", context)
            .await
            .expect("Should start chain execution");

        assert!(!correlation_id.to_string().is_empty());

        let state = coordinator
            .get_chain_state("test-chain")
            .await
            .expect("Should get chain state");

        matches!(state, ChainState::Executing { .. });
    }

    #[cfg_attr(test_category = "unit")]
    #[test]
    fn test_execution_chain_builder() {
        let agent_id = ComponentId::new(ComponentType::Agent, "agent".to_string());
        let tool_id = ComponentId::new(ComponentType::Tool, "tool".to_string());

        let chain = ExecutionChain::new()
            .with_name("test-chain")
            .add_component(agent_id.clone())
            .add_component(tool_id.clone())
            .with_metadata("purpose", "testing");

        assert_eq!(chain.name, "test-chain");
        assert_eq!(chain.components.len(), 2);
        assert_eq!(chain.components[0], agent_id);
        assert_eq!(chain.components[1], tool_id);
        assert_eq!(chain.metadata.get("purpose"), Some(&"testing".to_string()));
        assert_eq!(chain.state, ChainState::Pending);
    }

    #[cfg_attr(test_category = "unit")]
    #[test]
    fn test_chain_navigation() {
        let agent_id = ComponentId::new(ComponentType::Agent, "agent".to_string());
        let tool_id = ComponentId::new(ComponentType::Tool, "tool".to_string());
        let workflow_id = ComponentId::new(ComponentType::Workflow, "workflow".to_string());

        let chain = ExecutionChain::new()
            .add_component(agent_id)
            .add_component(tool_id.clone())
            .add_component(workflow_id.clone());

        assert_eq!(chain.get_next_component(0), Some(&tool_id));
        assert_eq!(chain.get_next_component(1), Some(&workflow_id));
        assert_eq!(chain.get_next_component(2), None);

        assert!(!chain.is_last_component(0));
        assert!(!chain.is_last_component(1));
        assert!(chain.is_last_component(2));
    }
}
