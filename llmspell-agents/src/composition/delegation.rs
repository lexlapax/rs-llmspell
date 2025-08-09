//! ABOUTME: Delegation patterns for agent composition
//! ABOUTME: Enables agents to delegate tasks to other agents based on capabilities

#![allow(clippy::significant_drop_tightening)]

use super::traits::{
    Capability, CapabilityCategory, Composable, CompositionMetadata, CompositionType,
};
use async_trait::async_trait;
use llmspell_core::types::{AgentInput, AgentOutput};
use llmspell_core::{BaseAgent, ComponentMetadata, ExecutionContext, LLMSpellError, Result};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use tokio::sync::RwLock as TokioRwLock;

/// Strategy for selecting which agent to delegate to
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DelegationStrategy {
    /// First agent that matches capabilities
    FirstMatch,
    /// Agent with the best capability score
    BestMatch,
    /// Round-robin between matching agents
    RoundRobin,
    /// Random selection from matching agents
    Random,
    /// Load-balanced based on agent metrics
    LoadBalanced,
    /// Custom selection function
    Custom(String),
}

/// Delegation request containing task details
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DelegationRequest {
    /// Task identifier
    pub task_id: String,
    /// Required capabilities for the task
    pub required_capabilities: Vec<Capability>,
    /// Input data for the task
    pub input: Value,
    /// Priority level (higher is more important)
    pub priority: u8,
    /// Timeout for the task
    pub timeout: Option<std::time::Duration>,
    /// Additional metadata
    pub metadata: HashMap<String, Value>,
}

/// Result of a delegation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DelegationResult {
    /// Task identifier
    pub task_id: String,
    /// Agent that handled the task
    pub delegated_to: String,
    /// Result of the task
    pub result: Value,
    /// Time taken to complete
    pub duration: std::time::Duration,
    /// Whether the task was successful
    pub success: bool,
    /// Error message if failed
    pub error: Option<String>,
}

/// Metrics for delegation tracking
#[derive(Debug, Default)]
pub struct DelegationMetrics {
    /// Total delegations attempted
    total_delegations: u64,
    /// Successful delegations
    successful_delegations: u64,
    /// Failed delegations
    failed_delegations: u64,
    /// Delegations per agent
    agent_delegations: HashMap<String, u64>,
    /// Average delegation time
    avg_delegation_time: std::time::Duration,
}

/// A delegation-based composite agent
pub struct DelegatingAgent {
    /// Component metadata
    metadata: ComponentMetadata,
    /// Available agents for delegation
    agents: TokioRwLock<HashMap<String, Arc<dyn BaseAgent>>>,
    /// Agent capabilities index (key is capability name)
    capabilities_index: TokioRwLock<HashMap<String, Vec<String>>>,
    /// Delegation strategy
    strategy: RwLock<DelegationStrategy>,
    /// Round-robin index
    round_robin_index: RwLock<usize>,
    /// Delegation metrics
    metrics: RwLock<DelegationMetrics>,
    /// Configuration
    config: DelegationConfig,
}

/// Configuration for delegating agents
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DelegationConfig {
    /// Default timeout for delegations
    pub default_timeout: std::time::Duration,
    /// Whether to cache capability lookups
    pub cache_capabilities: bool,
    /// Maximum concurrent delegations
    pub max_concurrent: usize,
    /// Whether to retry failed delegations
    pub retry_on_failure: bool,
    /// Number of retry attempts
    pub max_retries: u32,
}

impl Default for DelegationConfig {
    fn default() -> Self {
        Self {
            default_timeout: std::time::Duration::from_secs(30),
            cache_capabilities: true,
            max_concurrent: 10,
            retry_on_failure: true,
            max_retries: 3,
        }
    }
}

impl DelegatingAgent {
    /// Create a new delegating agent
    pub fn new(name: impl Into<String>, config: DelegationConfig) -> Self {
        let name = name.into();
        let description = format!("Delegating agent: {name}");
        Self {
            metadata: ComponentMetadata::new(name, description),
            agents: TokioRwLock::new(HashMap::new()),
            capabilities_index: TokioRwLock::new(HashMap::new()),
            strategy: RwLock::new(DelegationStrategy::BestMatch),
            round_robin_index: RwLock::new(0),
            metrics: RwLock::new(DelegationMetrics::default()),
            config,
        }
    }

    /// Register an agent for delegation
    ///
    /// # Errors
    ///
    /// Returns an error if agent registration fails
    pub async fn register_agent(&self, agent: Arc<dyn BaseAgent>) -> Result<()> {
        let agent_id = agent.metadata().id.to_string();

        // Store the agent
        let mut agents = self.agents.write().await;
        agents.insert(agent_id.clone(), agent.clone());
        drop(agents);

        // Index capabilities if caching is enabled
        if self.config.cache_capabilities {
            self.index_agent_capabilities(&agent_id, agent).await?;
        }

        Ok(())
    }

    /// Unregister an agent
    ///
    /// # Errors
    ///
    /// Returns an error if agent unregistration fails
    pub async fn unregister_agent(&self, agent_id: &str) -> Result<()> {
        let mut agents = self.agents.write().await;
        agents.remove(agent_id);
        drop(agents);

        // Remove from capabilities index
        let mut index = self.capabilities_index.write().await;
        for agents in index.values_mut() {
            agents.retain(|id| id != agent_id);
        }
        drop(index);

        Ok(())
    }

    /// Index an agent's capabilities
    async fn index_agent_capabilities(
        &self,
        agent_id: &str,
        _agent: Arc<dyn BaseAgent>,
    ) -> Result<()> {
        let mut index = self.capabilities_index.write().await;

        // In a real implementation, we would query the agent's capabilities
        // For now, we'll add some default capabilities
        let capabilities = vec![Capability {
            name: "general-processing".to_string(),
            category: CapabilityCategory::DataProcessing,
            version: None,
            metadata: HashMap::new(),
        }];

        for capability in capabilities {
            index
                .entry(capability.name)
                .or_default()
                .push(agent_id.to_string());
        }
        drop(index);

        Ok(())
    }

    /// Find agents matching required capabilities
    async fn find_matching_agents(&self, capabilities: &[Capability]) -> Vec<String> {
        if self.config.cache_capabilities {
            let index = self.capabilities_index.read().await;
            let mut matching_agents = Vec::new();

            for capability in capabilities {
                if let Some(agents) = index.get(&capability.name) {
                    for agent in agents {
                        if !matching_agents.contains(agent) {
                            matching_agents.push(agent.clone());
                        }
                    }
                }
            }

            matching_agents
        } else {
            // Without caching, return all agents
            let agents = self.agents.read().await;
            agents.keys().cloned().collect()
        }
    }

    /// Select an agent based on the configured strategy
    fn select_agent(
        &self,
        matching_agents: Vec<String>,
        _request: &DelegationRequest,
    ) -> Option<String> {
        if matching_agents.is_empty() {
            return None;
        }

        let strategy = self.strategy.read().unwrap().clone();
        match strategy {
            DelegationStrategy::FirstMatch => matching_agents.into_iter().next(),

            DelegationStrategy::BestMatch => {
                // In a real implementation, we would score agents
                // For now, just return the first
                matching_agents.into_iter().next()
            }

            DelegationStrategy::RoundRobin => {
                let selected = {
                    let mut index = self.round_robin_index.write().unwrap();
                    let selected = matching_agents[*index % matching_agents.len()].clone();
                    *index += 1;
                    selected
                };
                Some(selected)
            }

            DelegationStrategy::Random => {
                use rand::Rng;
                let mut rng = rand::thread_rng();
                let index = rng.gen_range(0..matching_agents.len());
                Some(matching_agents[index].clone())
            }

            DelegationStrategy::LoadBalanced => {
                // Select agent with fewest delegations
                let metrics = self.metrics.read().unwrap();
                matching_agents
                    .into_iter()
                    .min_by_key(|agent| metrics.agent_delegations.get(agent).copied().unwrap_or(0))
            }

            DelegationStrategy::Custom(_) => {
                // Custom strategy would be implemented by extending this
                matching_agents.into_iter().next()
            }
        }
    }

    /// Delegate a request to an appropriate agent
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - No suitable agent is found
    /// - Agent execution fails
    /// - Strategy processing fails
    ///
    /// # Panics
    ///
    /// Panics if a RwLock is poisoned
    pub async fn delegate(&self, request: DelegationRequest) -> Result<DelegationResult> {
        let start_time = std::time::Instant::now();

        // Find matching agents
        let matching_agents = self
            .find_matching_agents(&request.required_capabilities)
            .await;

        // Select an agent
        let selected_agent = self
            .select_agent(matching_agents, &request)
            .ok_or_else(|| LLMSpellError::Component {
                message: "No matching agent found for delegation".to_string(),
                source: None,
            })?;

        // Update metrics
        {
            let mut metrics = self.metrics.write().unwrap();
            metrics.total_delegations += 1;
            *metrics
                .agent_delegations
                .entry(selected_agent.clone())
                .or_insert(0) += 1;
        }

        // Get the agent and delegate
        let agents = self.agents.read().await;
        let agent = agents
            .get(&selected_agent)
            .ok_or_else(|| LLMSpellError::Component {
                message: format!("Agent not found: {selected_agent}"),
                source: None,
            })?;

        // Execute with timeout
        let timeout = request.timeout.unwrap_or(self.config.default_timeout);
        let input = AgentInput::text(request.input.to_string());
        let context = ExecutionContext::new();

        let result = tokio::time::timeout(timeout, agent.execute(input, context))
            .await
            .map_err(|_| LLMSpellError::Component {
                message: "Delegation timeout".to_string(),
                source: None,
            })?;

        let duration = start_time.elapsed();

        // Update metrics and create result
        match result {
            Ok(output) => {
                {
                    let mut metrics = self.metrics.write().unwrap();
                    metrics.successful_delegations += 1;
                    metrics.avg_delegation_time = std::time::Duration::from_secs(
                        (metrics.avg_delegation_time.as_secs()
                            * (metrics.successful_delegations - 1)
                            + duration.as_secs())
                            / metrics.successful_delegations,
                    );
                }

                Ok(DelegationResult {
                    task_id: request.task_id,
                    delegated_to: selected_agent,
                    result: serde_json::to_value(&output)?,
                    duration,
                    success: true,
                    error: None,
                })
            }
            Err(e) => {
                {
                    let mut metrics = self.metrics.write().unwrap();
                    metrics.failed_delegations += 1;
                }

                if self.config.retry_on_failure && request.priority > 5 {
                    // Could implement retry logic here
                }

                Ok(DelegationResult {
                    task_id: request.task_id,
                    delegated_to: selected_agent,
                    result: Value::Null,
                    duration,
                    success: false,
                    error: Some(e.to_string()),
                })
            }
        }
    }

    /// Set the delegation strategy
    ///
    /// # Panics
    ///
    /// Panics if the RwLock is poisoned
    pub fn set_strategy(&self, strategy: DelegationStrategy) {
        *self.strategy.write().unwrap() = strategy;
    }

    /// Get delegation metrics
    ///
    /// # Panics
    ///
    /// Panics if the RwLock is poisoned
    pub fn metrics(&self) -> DelegationMetrics {
        self.metrics.read().unwrap().clone()
    }
}

#[async_trait]
impl BaseAgent for DelegatingAgent {
    fn metadata(&self) -> &ComponentMetadata {
        &self.metadata
    }

    async fn execute(&self, input: AgentInput, _context: ExecutionContext) -> Result<AgentOutput> {
        // Parse the input as a delegation request
        let request: DelegationRequest = serde_json::from_str(&input.text)?;

        // Delegate the request
        let result = self.delegate(request).await?;

        // Return the result
        Ok(AgentOutput::text(serde_json::to_string(&result)?))
    }

    async fn validate_input(&self, input: &AgentInput) -> Result<()> {
        // Validate that input can be parsed as DelegationRequest
        let _: DelegationRequest = serde_json::from_str(&input.text)?;
        Ok(())
    }

    async fn handle_error(&self, error: LLMSpellError) -> Result<AgentOutput> {
        Ok(AgentOutput::text(format!("Delegation error: {error}")))
    }
}

#[async_trait]
impl Composable for DelegatingAgent {
    fn composition_metadata(&self) -> CompositionMetadata {
        CompositionMetadata {
            composition_type: CompositionType::Delegation,
            version: "1.0.0".to_string(),
            max_components: None,
            can_be_parent: true,
            can_be_child: true,
            custom: HashMap::new(),
        }
    }

    fn can_compose_with(&self, other: &dyn Composable) -> bool {
        // Can compose with any other composable agent
        let other_meta = other.composition_metadata();
        other_meta.can_be_child
    }

    fn exposed_capabilities(&self) -> Vec<Capability> {
        vec![
            Capability {
                name: "delegation".to_string(),
                category: CapabilityCategory::Orchestration,
                version: None,
                metadata: HashMap::new(),
            },
            Capability {
                name: "load-balancing".to_string(),
                category: CapabilityCategory::Orchestration,
                version: None,
                metadata: HashMap::new(),
            },
        ]
    }

    fn required_capabilities(&self) -> Vec<Capability> {
        Vec::new() // No specific requirements
    }
}

// Clone implementation for DelegationMetrics
impl Clone for DelegationMetrics {
    fn clone(&self) -> Self {
        Self {
            total_delegations: self.total_delegations,
            successful_delegations: self.successful_delegations,
            failed_delegations: self.failed_delegations,
            agent_delegations: self.agent_delegations.clone(),
            avg_delegation_time: self.avg_delegation_time,
        }
    }
}

/// Builder for delegating agents
pub struct DelegatingAgentBuilder {
    name: String,
    config: DelegationConfig,
    strategy: DelegationStrategy,
    initial_agents: Vec<Arc<dyn BaseAgent>>,
}

impl DelegatingAgentBuilder {
    /// Create a new builder
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            config: DelegationConfig::default(),
            strategy: DelegationStrategy::BestMatch,
            initial_agents: Vec::new(),
        }
    }

    /// Set the configuration
    #[must_use]
    pub const fn config(mut self, config: DelegationConfig) -> Self {
        self.config = config;
        self
    }

    /// Set the delegation strategy
    #[must_use]
    pub fn strategy(mut self, strategy: DelegationStrategy) -> Self {
        self.strategy = strategy;
        self
    }

    /// Add an initial agent
    #[must_use]
    pub fn add_agent(mut self, agent: Arc<dyn BaseAgent>) -> Self {
        self.initial_agents.push(agent);
        self
    }

    /// Build the delegating agent
    ///
    /// # Errors
    ///
    /// Returns an error if agent building fails
    pub async fn build(self) -> Result<DelegatingAgent> {
        let agent = DelegatingAgent::new(self.name, self.config);
        agent.set_strategy(self.strategy);

        // Register initial agents
        for initial_agent in self.initial_agents {
            agent.register_agent(initial_agent).await?;
        }

        Ok(agent)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[tokio::test]
    async fn test_delegating_agent_creation() {
        let agent = DelegatingAgent::new("test-delegator", DelegationConfig::default());
        assert_eq!(agent.metadata().name, "test-delegator");
    }
    #[tokio::test]
    async fn test_agent_registration() {
        let delegator = DelegatingAgent::new("delegator", DelegationConfig::default());

        // Create a mock agent
        struct MockAgent {
            metadata: ComponentMetadata,
        }

        #[async_trait]
        impl BaseAgent for MockAgent {
            fn metadata(&self) -> &ComponentMetadata {
                &self.metadata
            }

            async fn execute(
                &self,
                _input: AgentInput,
                _context: ExecutionContext,
            ) -> Result<AgentOutput> {
                Ok(AgentOutput::text("Mock response"))
            }

            async fn validate_input(&self, _input: &AgentInput) -> Result<()> {
                Ok(())
            }

            async fn handle_error(&self, error: LLMSpellError) -> Result<AgentOutput> {
                Ok(AgentOutput::text(format!("Error: {}", error)))
            }
        }

        let mock = Arc::new(MockAgent {
            metadata: ComponentMetadata::new(
                "mock-agent".to_string(),
                "Mock agent for testing".to_string(),
            ),
        });

        delegator.register_agent(mock).await.unwrap();

        let agents = delegator.agents.read().await;
        assert_eq!(agents.len(), 1);
    }
    #[test]
    fn test_delegation_strategy() {
        let strategies = vec![
            DelegationStrategy::FirstMatch,
            DelegationStrategy::BestMatch,
            DelegationStrategy::RoundRobin,
            DelegationStrategy::Random,
            DelegationStrategy::LoadBalanced,
            DelegationStrategy::Custom("test".to_string()),
        ];

        for strategy in strategies {
            match strategy {
                DelegationStrategy::FirstMatch => {}
                DelegationStrategy::BestMatch => {}
                DelegationStrategy::RoundRobin => {}
                DelegationStrategy::Random => {}
                DelegationStrategy::LoadBalanced => {}
                DelegationStrategy::Custom(s) => assert_eq!(s, "test"),
            }
        }
    }
    #[test]
    fn test_delegation_request() {
        let request = DelegationRequest {
            task_id: "task-123".to_string(),
            required_capabilities: vec![Capability {
                name: "text-processing".to_string(),
                category: CapabilityCategory::DataProcessing,
                version: None,
                metadata: HashMap::new(),
            }],
            input: Value::String("Process this text".to_string()),
            priority: 8,
            timeout: Some(std::time::Duration::from_secs(10)),
            metadata: HashMap::new(),
        };

        assert_eq!(request.task_id, "task-123");
        assert_eq!(request.priority, 8);
        assert_eq!(request.required_capabilities.len(), 1);
    }
}
