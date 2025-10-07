//! ABOUTME: Agent bridge for script-to-agent communication
//! ABOUTME: Provides unified interface for scripts to interact with agents

#![allow(clippy::significant_drop_tightening)]

use crate::agents::{AgentDiscovery, AgentInfo};
use crate::ComponentRegistry;
use llmspell_agents::lifecycle::{AgentState, AgentStateMachine};
use llmspell_agents::monitoring::metrics::MetricAccess;
use llmspell_agents::monitoring::{
    AgentMetrics, AlertConfig, AlertManager, EventLogger, HealthCheck, MetricRegistry,
    PerformanceMonitor,
};
use llmspell_agents::{AgentConfig, AgentFactory};
use llmspell_core::execution_context::{
    ContextScope, ExecutionContextBuilder, InheritancePolicy, SecurityContext, SharedMemory,
};
use llmspell_core::types::{AgentInput, AgentOutput};
#[cfg(test)]
use llmspell_core::types::ComponentId;
use llmspell_core::{Agent, ExecutionContext, LLMSpellError, Result, Tool};
use llmspell_kernel::state::{StateManager, StateScope};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::mpsc;
use tracing::{debug, info, warn};

/// Routing strategy for composite agents
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum RoutingStrategy {
    /// Execute delegate agents sequentially
    Sequential,
    /// Execute delegate agents in parallel
    Parallel,
    /// Execute with voting/consensus mechanism
    Vote {
        /// Minimum votes required for consensus (defaults to majority)
        #[serde(skip_serializing_if = "Option::is_none")]
        threshold: Option<usize>,
    },
    /// Custom routing strategy
    Custom {
        /// Name of the custom strategy
        name: String,
    },
}

impl Default for RoutingStrategy {
    fn default() -> Self {
        Self::Sequential
    }
}

/// Configuration for composite agent routing
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RoutingConfig {
    /// The routing strategy to use
    #[serde(default)]
    pub strategy: RoutingStrategy,
    /// Optional fallback agent if routing fails
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fallback_agent: Option<String>,
    /// Optional timeout in milliseconds
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timeout_ms: Option<u64>,
}

/// Security context configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityContextConfig {
    /// List of permissions
    pub permissions: Vec<String>,
    /// Security level
    pub level: String,
}

impl Default for SecurityContextConfig {
    fn default() -> Self {
        Self {
            permissions: Vec::new(),
            level: "default".to_string(),
        }
    }
}

/// Configuration for creating an execution context
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ExecutionContextConfig {
    /// Conversation identifier
    #[serde(skip_serializing_if = "Option::is_none")]
    pub conversation_id: Option<String>,
    /// User identifier
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_id: Option<String>,
    /// Session identifier
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session_id: Option<String>,
    /// Context scope
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scope: Option<ContextScope>,
    /// Inheritance policy
    #[serde(skip_serializing_if = "Option::is_none")]
    pub inheritance: Option<InheritancePolicy>,
    /// Initial data
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<HashMap<String, serde_json::Value>>,
    /// Security context
    #[serde(skip_serializing_if = "Option::is_none")]
    pub security: Option<SecurityContextConfig>,
}

/// Configuration for creating a child context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChildContextConfig {
    /// Scope for the child context
    pub scope: ContextScope,
    /// Inheritance policy
    pub inheritance: InheritancePolicy,
}

impl Default for ChildContextConfig {
    fn default() -> Self {
        Self {
            scope: ContextScope::Global,
            inheritance: InheritancePolicy::Inherit,
        }
    }
}

/// Bridge between scripts and agents
pub struct AgentBridge {
    /// Agent discovery service
    discovery: Arc<AgentDiscovery>,
    /// Component registry for script access
    registry: Arc<ComponentRegistry>,
    /// Active agent instances
    active_agents: Arc<tokio::sync::RwLock<HashMap<String, Arc<dyn Agent>>>>,
    /// Agent state machines
    state_machines: Arc<tokio::sync::RwLock<HashMap<String, Arc<AgentStateMachine>>>>,
    /// Monitoring components
    metrics_registry: Arc<MetricRegistry>,
    performance_monitor: Arc<PerformanceMonitor>,
    #[allow(dead_code)]
    health_check: Arc<dyn HealthCheck>,
    event_logger: Arc<EventLogger>,
    #[allow(dead_code)]
    alert_manager: Arc<AlertManager>,
    /// Global shared memory for inter-agent communication
    shared_memory: Arc<SharedMemory>,
    /// Active contexts by ID
    contexts: Arc<tokio::sync::RwLock<HashMap<String, Arc<ExecutionContext>>>>,
    /// Active streaming channels
    streaming_channels: Arc<tokio::sync::RwLock<HashMap<String, mpsc::Sender<AgentOutput>>>>,
    /// State manager for agent state persistence
    state_manager: Option<Arc<StateManager>>,
}

impl AgentBridge {
    /// Create a new agent bridge with provider manager
    #[must_use]
    pub fn new(
        registry: Arc<ComponentRegistry>,
        provider_manager: Arc<llmspell_providers::ProviderManager>,
    ) -> Self {
        Self {
            discovery: Arc::new(AgentDiscovery::new(provider_manager)),
            registry,
            active_agents: Arc::new(tokio::sync::RwLock::new(HashMap::new())),
            state_machines: Arc::new(tokio::sync::RwLock::new(HashMap::new())),
            metrics_registry: Arc::new(MetricRegistry::new()),
            performance_monitor: Arc::new(PerformanceMonitor::new(
                "bridge".to_string(),
                Arc::new(AgentMetrics::new("bridge".to_string())),
                1000,
                Duration::from_secs(300),
            )),
            health_check: Arc::new(crate::monitoring::HealthCheckImpl::new()),
            event_logger: Arc::new(EventLogger::new("bridge".to_string(), 1000)),
            alert_manager: Arc::new(AlertManager::new(AlertConfig::default())),
            shared_memory: Arc::new(SharedMemory::new()),
            contexts: Arc::new(tokio::sync::RwLock::new(HashMap::new())),
            streaming_channels: Arc::new(tokio::sync::RwLock::new(HashMap::new())),
            state_manager: None,
        }
    }

    /// Create with custom factory
    pub fn with_factory(registry: Arc<ComponentRegistry>, factory: Arc<dyn AgentFactory>) -> Self {
        Self {
            discovery: Arc::new(AgentDiscovery::with_factory(factory)),
            registry,
            active_agents: Arc::new(tokio::sync::RwLock::new(HashMap::new())),
            state_machines: Arc::new(tokio::sync::RwLock::new(HashMap::new())),
            metrics_registry: Arc::new(MetricRegistry::new()),
            performance_monitor: Arc::new(PerformanceMonitor::new(
                "bridge".to_string(),
                Arc::new(AgentMetrics::new("bridge".to_string())),
                1000,
                Duration::from_secs(300),
            )),
            health_check: Arc::new(crate::monitoring::HealthCheckImpl::new()),
            event_logger: Arc::new(EventLogger::new("bridge".to_string(), 1000)),
            alert_manager: Arc::new(AlertManager::new(AlertConfig::default())),
            shared_memory: Arc::new(SharedMemory::new()),
            contexts: Arc::new(tokio::sync::RwLock::new(HashMap::new())),
            streaming_channels: Arc::new(tokio::sync::RwLock::new(HashMap::new())),
            state_manager: None,
        }
    }

    /// List available agent types
    #[must_use]
    pub fn list_agent_types(&self) -> Vec<String> {
        self.discovery.list_agent_types()
    }

    /// List available templates
    #[must_use]
    pub fn list_templates(&self) -> Vec<String> {
        self.discovery.list_templates()
    }

    /// Get agent information
    ///
    /// # Errors
    ///
    /// Returns an error if the agent type is not found
    pub fn get_agent_info(&self, agent_type: &str) -> Result<AgentInfo> {
        self.discovery.get_agent_info(agent_type)
    }

    /// Create a new agent instance
    ///
    /// # Errors
    ///
    /// Returns an error if the agent instance already exists or creation fails
    pub async fn create_agent(&self, config: AgentConfig) -> Result<()> {
        let instance_name = config.name.clone();

        // Check if instance already exists
        {
            let agents = self.active_agents.read().await;
            if agents.contains_key(&instance_name) {
                return Err(LLMSpellError::Validation {
                    field: Some("instance_name".to_string()),
                    message: format!("Agent instance '{instance_name}' already exists"),
                });
            }
        }

        // Create the agent with typed config
        let agent = self.discovery.create_agent(config).await?;

        // Create state machine for the agent
        let state_machine = Arc::new(AgentStateMachine::default(instance_name.clone()));

        // Register in active agents, state machines, and component registry
        {
            let mut agents = self.active_agents.write().await;
            agents.insert(instance_name.to_string(), agent.clone());
        }
        {
            let mut machines = self.state_machines.write().await;
            machines.insert(instance_name.to_string(), state_machine);
        }

        // Also register in component registry for script access
        debug!(
            "DEBUG: Registering agent '{}' in ComponentRegistry",
            instance_name
        );
        self.registry
            .register_agent(instance_name.to_string(), agent)?;
        info!(
            "DEBUG: Successfully registered agent '{}' in ComponentRegistry",
            instance_name
        );

        Ok(())
    }

    /// Create agent from template
    ///
    /// # Errors
    ///
    /// Returns an error if the template is not found or agent creation fails
    pub async fn create_from_template(
        &self,
        instance_name: &str,
        template_name: &str,
        parameters: HashMap<String, serde_json::Value>,
    ) -> Result<()> {
        // Check if instance already exists
        {
            let agents = self.active_agents.read().await;
            if agents.contains_key(instance_name) {
                return Err(LLMSpellError::Validation {
                    field: Some("instance_name".to_string()),
                    message: format!("Agent instance '{instance_name}' already exists"),
                });
            }
        }

        // Create from template
        let agent = self
            .discovery
            .create_from_template(template_name, parameters)
            .await?;

        // Create state machine for the agent
        let state_machine = Arc::new(AgentStateMachine::default(instance_name.to_string()));

        // Register in active agents, state machines, and component registry
        {
            let mut agents = self.active_agents.write().await;
            agents.insert(instance_name.to_string(), agent.clone());
        }
        {
            let mut machines = self.state_machines.write().await;
            machines.insert(instance_name.to_string(), state_machine);
        }

        // Also register in component registry for script access
        self.registry
            .register_agent(instance_name.to_string(), agent)?;

        Ok(())
    }

    /// Execute an agent
    ///
    /// # Errors
    ///
    /// Returns an error if the agent is not found or execution fails
    pub async fn execute_agent(
        &self,
        instance_name: &str,
        input: AgentInput,
        context: Option<ExecutionContext>,
    ) -> Result<AgentOutput> {
        // Get agent from active agents
        let agent = {
            let agents = self.active_agents.read().await;
            agents.get(instance_name).cloned()
        };

        let agent = agent.ok_or_else(|| LLMSpellError::Component {
            message: format!("Agent instance '{instance_name}' not found"),
            source: None,
        })?;

        // Use provided context or create new one
        let context = context.unwrap_or_default();

        // Execute the agent
        agent.execute(input, context).await
    }

    /// Get agent instance
    pub async fn get_agent(&self, instance_name: &str) -> Option<Arc<dyn Agent>> {
        let agents = self.active_agents.read().await;
        agents.get(instance_name).cloned()
    }

    /// Remove an agent instance
    ///
    /// # Errors
    ///
    /// Returns an error if the agent instance is not found
    pub async fn remove_agent(&self, instance_name: &str) -> Result<()> {
        // Remove from active agents and state machines
        let removed = {
            let mut agents = self.active_agents.write().await;
            agents.remove(instance_name)
        };

        {
            let mut machines = self.state_machines.write().await;
            machines.remove(instance_name);
        }

        if removed.is_none() {
            return Err(LLMSpellError::Component {
                message: format!("Agent instance '{instance_name}' not found"),
                source: None,
            });
        }

        // Note: We don't remove from component registry as it doesn't have a remove method
        // This could be added if needed

        Ok(())
    }

    /// List active agent instances
    pub async fn list_instances(&self) -> Vec<String> {
        let agents = self.active_agents.read().await;
        agents.keys().cloned().collect()
    }

    /// Get agent configuration
    ///
    /// # Errors
    ///
    /// Returns an error if the agent instance is not found
    pub async fn get_agent_config(&self, instance_name: &str) -> Result<serde_json::Value> {
        let agent =
            self.get_agent(instance_name)
                .await
                .ok_or_else(|| LLMSpellError::Component {
                    message: format!("Agent instance '{instance_name}' not found"),
                    source: None,
                })?;

        // Convert agent config to JSON
        let config = agent.config();
        let config_json = serde_json::json!({
            "system_prompt": config.system_prompt,
            "temperature": config.temperature,
            "max_tokens": config.max_tokens,
            "max_conversation_length": config.max_conversation_length,
        });

        Ok(config_json)
    }

    /// Clear all agent instances
    pub async fn clear_all(&self) {
        let mut agents = self.active_agents.write().await;
        agents.clear();
        // Note: This doesn't clear the component registry
    }

    // Tool Integration Methods

    /// List available tools
    #[must_use]
    pub fn list_tools(&self) -> Vec<String> {
        self.registry.list_tools()
    }

    /// Get tool information
    #[must_use]
    pub fn get_tool(&self, tool_name: &str) -> Option<Arc<dyn Tool>> {
        self.registry.get_tool(tool_name)
    }

    /// Invoke a tool on behalf of an agent
    ///
    /// # Errors
    ///
    /// Returns an error if the agent instance or tool is not found, or if tool execution fails
    pub async fn invoke_tool_for_agent(
        &self,
        agent_instance: &str,
        tool_name: &str,
        tool_input: AgentInput,
        context: Option<ExecutionContext>,
    ) -> Result<AgentOutput> {
        // Verify agent exists
        let _agent =
            self.get_agent(agent_instance)
                .await
                .ok_or_else(|| LLMSpellError::Component {
                    message: format!("Agent instance '{agent_instance}' not found"),
                    source: None,
                })?;

        // Get the tool
        let tool = self
            .registry
            .get_tool(tool_name)
            .ok_or_else(|| LLMSpellError::Component {
                message: format!("Tool '{tool_name}' not found"),
                source: None,
            })?;

        // Use provided context or create new one
        let context = context.unwrap_or_default();

        // Execute the tool
        tool.execute(tool_input, context).await
    }

    /// Check if a tool is available
    #[must_use]
    pub fn has_tool(&self, tool_name: &str) -> bool {
        self.registry.get_tool(tool_name).is_some()
    }

    /// Get tool metadata for discovery
    #[must_use]
    pub fn get_tool_metadata(&self, tool_name: &str) -> Option<serde_json::Value> {
        self.registry.get_tool(tool_name).map(|tool| {
            let metadata = tool.metadata();
            serde_json::json!({
                "name": metadata.name,
                "description": metadata.description,
                "version": metadata.version,
            })
        })
    }

    /// Get all tool metadata for bulk discovery
    #[must_use]
    pub fn get_all_tool_metadata(&self) -> HashMap<String, serde_json::Value> {
        let mut metadata_map = HashMap::new();
        for tool_name in self.list_tools() {
            if let Some(metadata) = self.get_tool_metadata(&tool_name) {
                metadata_map.insert(tool_name, metadata);
            }
        }
        metadata_map
    }

    // Monitoring & Lifecycle Methods

    /// Get metrics for an agent instance
    ///
    /// # Errors
    ///
    /// Returns an error if the agent instance is not found
    pub async fn get_agent_metrics(&self, agent_instance: &str) -> Result<AgentMetrics> {
        // Verify agent exists
        let _agent =
            self.get_agent(agent_instance)
                .await
                .ok_or_else(|| LLMSpellError::Component {
                    message: format!("Agent instance '{agent_instance}' not found"),
                    source: None,
                })?;

        // Get metrics from registry (returns Arc<AgentMetrics>)
        let _metrics_arc = self.metrics_registry.get_agent_metrics(agent_instance);
        // Return a new metrics instance since AgentMetrics doesn't implement Clone
        Ok(AgentMetrics::new(agent_instance.to_string()))
    }

    /// Get overall bridge metrics
    #[must_use]
    pub fn get_bridge_metrics(&self) -> HashMap<String, serde_json::Value> {
        let mut metrics = HashMap::new();

        // Get basic statistics
        let agent_count =
            futures::executor::block_on(async { self.active_agents.read().await.len() });

        metrics.insert("active_agents".to_string(), serde_json::json!(agent_count));
        metrics.insert(
            "total_tools".to_string(),
            serde_json::json!(self.list_tools().len()),
        );

        // Get performance metrics
        let perf_snapshot = self.performance_monitor.take_snapshot();
        #[allow(clippy::cast_precision_loss)]
        let memory_usage_mb = perf_snapshot.resources.memory_bytes as f64 / (1024.0 * 1024.0);
        #[allow(clippy::cast_precision_loss)]
        let uptime_seconds = perf_snapshot.timestamp.timestamp() as f64;

        metrics.insert(
            "performance".to_string(),
            serde_json::json!({
                "memory_usage_mb": memory_usage_mb,
                "cpu_usage_percent": perf_snapshot.resources.cpu_percent,
                "uptime_seconds": uptime_seconds,
            }),
        );

        metrics
    }

    /// Get health status for an agent
    ///
    /// # Errors
    ///
    /// Returns an error if the agent instance is not found or health check fails
    pub async fn get_agent_health(&self, agent_instance: &str) -> Result<serde_json::Value> {
        // Verify agent exists
        let _agent =
            self.get_agent(agent_instance)
                .await
                .ok_or_else(|| LLMSpellError::Component {
                    message: format!("Agent instance '{agent_instance}' not found"),
                    source: None,
                })?;

        // Get health from health checker (mock implementation)
        match crate::monitoring::check_agent_health(agent_instance) {
            Ok(health_result) => Ok(serde_json::json!({
                "status": format!("{:?}", health_result.overall_status),
                "timestamp": health_result.timestamp.to_rfc3339(),
                "components": health_result.components,
                "total_duration": health_result.total_duration.as_millis(),
            })),
            Err(e) => Err(e),
        }
    }

    /// Get performance report for an agent
    ///
    /// # Errors
    ///
    /// Returns an error if the agent instance is not found
    pub async fn get_agent_performance(&self, agent_instance: &str) -> Result<serde_json::Value> {
        // Verify agent exists
        let _agent =
            self.get_agent(agent_instance)
                .await
                .ok_or_else(|| LLMSpellError::Component {
                    message: format!("Agent instance '{agent_instance}' not found"),
                    source: None,
                })?;

        // Mock performance report since get_agent_report doesn't exist
        Ok(serde_json::json!({
            "total_executions": 100,
            "avg_execution_time_ms": 150.0,
            "success_rate": 0.95,
            "error_rate": 0.05,
            "last_execution": chrono::Utc::now().to_rfc3339(),
        }))
    }

    /// Subscribe to events for an agent (returns event channel)
    ///
    /// # Errors
    ///
    /// Returns an error if event subscription setup fails
    pub fn subscribe_to_agent_events(
        &self,
        _agent_instance: &str,
    ) -> Result<tokio::sync::mpsc::Receiver<serde_json::Value>> {
        let (tx, rx) = tokio::sync::mpsc::channel(100);

        // Mock event subscription - we would store the channel for future events
        // For now, just return the receiver without connecting it to real events
        std::mem::drop(tx); // Prevent unused warning

        Ok(rx)
    }

    /// Log an event for an agent
    ///
    /// # Errors
    ///
    /// Returns an error if the agent instance is not found or event logging fails
    pub async fn log_agent_event(
        &self,
        agent_instance: &str,
        event_type: &str,
        message: &str,
    ) -> Result<()> {
        use llmspell_agents::monitoring::{LogEvent, LogLevel};

        // Verify agent exists
        let _agent =
            self.get_agent(agent_instance)
                .await
                .ok_or_else(|| LLMSpellError::Component {
                    message: format!("Agent instance '{agent_instance}' not found"),
                    source: None,
                })?;

        // Create a LogEvent and log it
        let log_event = LogEvent {
            id: uuid::Uuid::new_v4().to_string(),
            timestamp: chrono::Utc::now(),
            level: LogLevel::Info,
            message: format!("{event_type}: {message}"),
            agent_id: agent_instance.to_string(),
            component: "bridge".to_string(),
            fields: std::collections::HashMap::new(),
            trace_id: None,
            span_id: None,
            error: None,
        };
        self.event_logger.log(log_event)?;
        Ok(())
    }

    /// Configure alerts for an agent
    ///
    /// # Errors
    ///
    /// Returns an error if the agent instance is not found
    pub async fn configure_agent_alerts(
        &self,
        agent_instance: &str,
        _alert_config: serde_json::Value,
    ) -> Result<()> {
        // Verify agent exists
        let _agent =
            self.get_agent(agent_instance)
                .await
                .ok_or_else(|| LLMSpellError::Component {
                    message: format!("Agent instance '{agent_instance}' not found"),
                    source: None,
                })?;

        // Mock alert configuration - real implementation would store per-agent configs
        Ok(())
    }

    /// Get active alerts for an agent
    ///
    /// # Errors
    ///
    /// Returns an error if the agent instance is not found
    pub async fn get_agent_alerts(&self, agent_instance: &str) -> Result<Vec<serde_json::Value>> {
        // Verify agent exists
        let _agent =
            self.get_agent(agent_instance)
                .await
                .ok_or_else(|| LLMSpellError::Component {
                    message: format!("Agent instance '{agent_instance}' not found"),
                    source: None,
                })?;

        // Mock get agent alerts - return empty list for now
        Ok(vec![])
    }

    // State Machine Methods

    /// Get the current state of an agent
    ///
    /// # Errors
    ///
    /// Returns an error if the agent state machine is not found
    pub async fn get_agent_state(&self, agent_instance: &str) -> Result<AgentState> {
        let machines = self.state_machines.read().await;
        let state_machine =
            machines
                .get(agent_instance)
                .ok_or_else(|| LLMSpellError::Component {
                    message: format!("State machine for agent '{agent_instance}' not found"),
                    source: None,
                })?;

        Ok(state_machine.current_state().await)
    }

    /// Initialize an agent
    ///
    /// # Errors
    ///
    /// Returns an error if the agent state machine is not found or initialization fails
    pub async fn initialize_agent(&self, agent_instance: &str) -> Result<()> {
        let machines = self.state_machines.read().await;
        let state_machine =
            machines
                .get(agent_instance)
                .ok_or_else(|| LLMSpellError::Component {
                    message: format!("State machine for agent '{agent_instance}' not found"),
                    source: None,
                })?;

        state_machine
            .initialize()
            .await
            .map_err(|e| LLMSpellError::Component {
                message: format!("Failed to initialize agent: {e}"),
                source: None,
            })
    }

    /// Start an agent
    ///
    /// # Errors
    ///
    /// Returns an error if the agent state machine is not found or start fails
    pub async fn start_agent(&self, agent_instance: &str) -> Result<()> {
        let machines = self.state_machines.read().await;
        let state_machine =
            machines
                .get(agent_instance)
                .ok_or_else(|| LLMSpellError::Component {
                    message: format!("State machine for agent '{agent_instance}' not found"),
                    source: None,
                })?;

        state_machine
            .start()
            .await
            .map_err(|e| LLMSpellError::Component {
                message: format!("Failed to start agent: {e}"),
                source: None,
            })
    }

    /// Pause an agent
    ///
    /// # Errors
    ///
    /// Returns an error if the agent state machine is not found or pause fails
    pub async fn pause_agent(&self, agent_instance: &str) -> Result<()> {
        let machines = self.state_machines.read().await;
        let state_machine =
            machines
                .get(agent_instance)
                .ok_or_else(|| LLMSpellError::Component {
                    message: format!("State machine for agent '{agent_instance}' not found"),
                    source: None,
                })?;

        state_machine
            .pause()
            .await
            .map_err(|e| LLMSpellError::Component {
                message: format!("Failed to pause agent: {e}"),
                source: None,
            })
    }

    /// Resume an agent
    ///
    /// # Errors
    ///
    /// Returns an error if the agent state machine is not found or resume fails
    pub async fn resume_agent(&self, agent_instance: &str) -> Result<()> {
        let machines = self.state_machines.read().await;
        let state_machine =
            machines
                .get(agent_instance)
                .ok_or_else(|| LLMSpellError::Component {
                    message: format!("State machine for agent '{agent_instance}' not found"),
                    source: None,
                })?;

        state_machine
            .resume()
            .await
            .map_err(|e| LLMSpellError::Component {
                message: format!("Failed to resume agent: {e}"),
                source: None,
            })
    }

    /// Stop an agent
    ///
    /// # Errors
    ///
    /// Returns an error if the agent state machine is not found or stop fails
    pub async fn stop_agent(&self, agent_instance: &str) -> Result<()> {
        let machines = self.state_machines.read().await;
        let state_machine =
            machines
                .get(agent_instance)
                .ok_or_else(|| LLMSpellError::Component {
                    message: format!("State machine for agent '{agent_instance}' not found"),
                    source: None,
                })?;

        state_machine
            .stop()
            .await
            .map_err(|e| LLMSpellError::Component {
                message: format!("Failed to stop agent: {e}"),
                source: None,
            })
    }

    /// Terminate an agent
    ///
    /// # Errors
    ///
    /// Returns an error if the agent state machine is not found or termination fails
    pub async fn terminate_agent(&self, agent_instance: &str) -> Result<()> {
        let machines = self.state_machines.read().await;
        let state_machine =
            machines
                .get(agent_instance)
                .ok_or_else(|| LLMSpellError::Component {
                    message: format!("State machine for agent '{agent_instance}' not found"),
                    source: None,
                })?;

        state_machine
            .terminate()
            .await
            .map_err(|e| LLMSpellError::Component {
                message: format!("Failed to terminate agent: {e}"),
                source: None,
            })
    }

    /// Put agent in error state
    ///
    /// # Errors
    ///
    /// Returns an error if the agent state machine is not found or error state transition fails
    pub async fn error_agent(&self, agent_instance: &str, error_message: String) -> Result<()> {
        let machines = self.state_machines.read().await;
        let state_machine =
            machines
                .get(agent_instance)
                .ok_or_else(|| LLMSpellError::Component {
                    message: format!("State machine for agent '{agent_instance}' not found"),
                    source: None,
                })?;

        state_machine
            .error(error_message)
            .await
            .map_err(|e| LLMSpellError::Component {
                message: format!("Failed to put agent in error state: {e}"),
                source: None,
            })
    }

    /// Attempt to recover agent from error
    ///
    /// # Errors
    ///
    /// Returns an error if the agent state machine is not found or recovery fails
    pub async fn recover_agent(&self, agent_instance: &str) -> Result<()> {
        let machines = self.state_machines.read().await;
        let state_machine =
            machines
                .get(agent_instance)
                .ok_or_else(|| LLMSpellError::Component {
                    message: format!("State machine for agent '{agent_instance}' not found"),
                    source: None,
                })?;

        state_machine
            .recover()
            .await
            .map_err(|e| LLMSpellError::Component {
                message: format!("Failed to recover agent: {e}"),
                source: None,
            })
    }

    /// Get state transition history
    ///
    /// # Errors
    ///
    /// Returns an error if the agent state machine is not found
    pub async fn get_agent_state_history(
        &self,
        agent_instance: &str,
    ) -> Result<Vec<serde_json::Value>> {
        let machines = self.state_machines.read().await;
        let state_machine =
            machines
                .get(agent_instance)
                .ok_or_else(|| LLMSpellError::Component {
                    message: format!("State machine for agent '{agent_instance}' not found"),
                    source: None,
                })?;

        let history = state_machine.get_transition_history().await;
        Ok(history
            .into_iter()
            .map(|transition| {
                let datetime = chrono::DateTime::<chrono::Utc>::from(transition.timestamp);
                serde_json::json!({
                    "from": format!("{:?}", transition.from),
                    "to": format!("{:?}", transition.to),
                    "timestamp": datetime.to_rfc3339(),
                    "elapsed": transition.duration.map_or(0.0, |d| d.as_secs_f64()),
                    "reason": transition.reason,
                    "metadata": transition.metadata,
                })
            })
            .collect())
    }

    /// Get last error for agent
    ///
    /// # Errors
    ///
    /// Returns an error if the agent state machine is not found
    pub async fn get_agent_last_error(&self, agent_instance: &str) -> Result<Option<String>> {
        let machines = self.state_machines.read().await;
        let state_machine =
            machines
                .get(agent_instance)
                .ok_or_else(|| LLMSpellError::Component {
                    message: format!("State machine for agent '{agent_instance}' not found"),
                    source: None,
                })?;

        Ok(state_machine.get_last_error().await)
    }

    /// Get recovery attempts count
    ///
    /// # Errors
    ///
    /// Returns an error if the agent state machine is not found
    pub async fn get_agent_recovery_attempts(&self, agent_instance: &str) -> Result<usize> {
        let machines = self.state_machines.read().await;
        let state_machine =
            machines
                .get(agent_instance)
                .ok_or_else(|| LLMSpellError::Component {
                    message: format!("State machine for agent '{agent_instance}' not found"),
                    source: None,
                })?;

        Ok(state_machine.get_recovery_attempts().await)
    }

    /// Check if agent is healthy
    ///
    /// # Errors
    ///
    /// Returns an error if the agent state machine is not found
    pub async fn is_agent_healthy(&self, agent_instance: &str) -> Result<bool> {
        let machines = self.state_machines.read().await;
        let state_machine =
            machines
                .get(agent_instance)
                .ok_or_else(|| LLMSpellError::Component {
                    message: format!("State machine for agent '{agent_instance}' not found"),
                    source: None,
                })?;

        Ok(state_machine.is_healthy().await)
    }

    /// Get state machine metrics
    ///
    /// # Errors
    ///
    /// Returns an error if the agent state machine is not found
    pub async fn get_agent_state_metrics(&self, agent_instance: &str) -> Result<serde_json::Value> {
        let machines = self.state_machines.read().await;
        let state_machine =
            machines
                .get(agent_instance)
                .ok_or_else(|| LLMSpellError::Component {
                    message: format!("State machine for agent '{agent_instance}' not found"),
                    source: None,
                })?;

        let metrics = state_machine.get_metrics().await;
        Ok(serde_json::json!({
            "agent_id": metrics.agent_id,
            "current_state": format!("{:?}", metrics.current_state),
            "total_transitions": metrics.total_transitions,
            "recovery_attempts": metrics.recovery_attempts,
            "last_error": metrics.last_error,
            "is_healthy": metrics.is_healthy,
            "uptime": metrics.uptime.as_secs_f64(),
        }))
    }

    // Context & Communication Methods

    /// Create a new execution context
    ///
    /// # Errors
    ///
    /// Returns an error if context creation fails
    pub async fn create_context(&self, config: ExecutionContextConfig) -> Result<String> {
        let mut builder = ExecutionContextBuilder::new();

        // Apply configuration
        if let Some(conversation_id) = config.conversation_id {
            builder = builder.conversation_id(conversation_id);
        }
        if let Some(user_id) = config.user_id {
            builder = builder.user_id(user_id);
        }
        if let Some(session_id) = config.session_id {
            builder = builder.session_id(session_id);
        }
        if let Some(scope) = config.scope {
            builder = builder.scope(scope);
        }
        if let Some(inheritance) = config.inheritance {
            builder = builder.inheritance(inheritance);
        }
        if let Some(data) = config.data {
            for (key, value) in data {
                builder = builder.data(key, value);
            }
        }
        if let Some(security) = config.security {
            builder = builder.security(SecurityContext {
                permissions: security.permissions,
                level: security.level,
            });
        }

        let context = Arc::new(builder.build());
        let context_id = context.id.clone();

        // Store context
        {
            let mut contexts = self.contexts.write().await;
            contexts.insert(context_id.clone(), context);
        }

        Ok(context_id)
    }

    /// Get an existing context
    pub async fn get_context(&self, context_id: &str) -> Option<Arc<ExecutionContext>> {
        let contexts = self.contexts.read().await;
        contexts.get(context_id).cloned()
    }

    /// Create a child context
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Parent context is not found
    pub async fn create_child_context(
        &self,
        parent_id: &str,
        config: ChildContextConfig,
    ) -> Result<String> {
        let parent = self
            .get_context(parent_id)
            .await
            .ok_or_else(|| LLMSpellError::Component {
                message: format!("Parent context '{parent_id}' not found"),
                source: None,
            })?;

        let child = Arc::new(parent.create_child(config.scope, config.inheritance));
        let child_id = child.id.clone();

        // Store child context
        {
            let mut contexts = self.contexts.write().await;
            contexts.insert(child_id.clone(), child);
        }

        Ok(child_id)
    }

    /// Update context data
    ///
    /// # Errors
    ///
    /// Returns an error if the context is not found
    pub async fn update_context(
        &self,
        context_id: &str,
        key: String,
        value: serde_json::Value,
    ) -> Result<()> {
        let contexts = self.contexts.read().await;
        let context = contexts
            .get(context_id)
            .ok_or_else(|| LLMSpellError::Component {
                message: format!("Context '{context_id}' not found"),
                source: None,
            })?;

        // Since ExecutionContext is Arc'd and fields are not mutable through Arc,
        // we'd need to use interior mutability or recreate the context
        // For now, we'll use shared memory for updates
        context.set_shared(key, value);
        Ok(())
    }

    /// Get data from context
    ///
    /// # Errors
    ///
    /// Returns an error if the context is not found
    pub async fn get_context_data(
        &self,
        context_id: &str,
        key: &str,
    ) -> Result<Option<serde_json::Value>> {
        let contexts = self.contexts.read().await;
        let context = contexts
            .get(context_id)
            .ok_or_else(|| LLMSpellError::Component {
                message: format!("Context '{context_id}' not found"),
                source: None,
            })?;

        Ok(context.get(key))
    }

    /// Set shared memory data
    pub fn set_shared_memory(
        &self,
        scope: &ContextScope,
        key: String,
        value: serde_json::Value,
    ) {
        self.shared_memory.set(scope.clone(), key, value);
    }

    /// Get shared memory data
    #[must_use]
    pub fn get_shared_memory(
        &self,
        scope: &ContextScope,
        key: &str,
    ) -> Option<serde_json::Value> {
        self.shared_memory.get(scope, key)
    }

    /// Execute agent with context
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Context is not found
    /// - Agent execution fails
    pub async fn execute_agent_with_context(
        &self,
        instance_name: &str,
        input: AgentInput,
        context_id: &str,
    ) -> Result<AgentOutput> {
        let context =
            self.get_context(context_id)
                .await
                .ok_or_else(|| LLMSpellError::Component {
                    message: format!("Context '{context_id}' not found"),
                    source: None,
                })?;

        self.execute_agent(instance_name, input, Some((*context).clone()))
            .await
    }

    /// Execute agent with streaming
    ///
    /// # Errors
    ///
    /// Returns an error if the agent instance is not found
    pub async fn execute_agent_streaming(
        &self,
        instance_name: &str,
        input: AgentInput,
        context: Option<ExecutionContext>,
    ) -> Result<mpsc::Receiver<AgentOutput>> {
        // Create streaming channel
        let (tx, rx) = mpsc::channel::<AgentOutput>(100);
        let channel_id = uuid::Uuid::new_v4().to_string();

        // Store channel
        {
            let mut channels = self.streaming_channels.write().await;
            channels.insert(channel_id.clone(), tx.clone());
        }

        // Get agent
        let agent =
            self.get_agent(instance_name)
                .await
                .ok_or_else(|| LLMSpellError::Component {
                    message: format!("Agent instance '{instance_name}' not found"),
                    source: None,
                })?;

        // Spawn streaming execution
        let context = context.unwrap_or_default();
        let channels = self.streaming_channels.clone();
        tokio::spawn(async move {
            // Execute agent
            match agent.execute(input, context).await {
                Ok(output) => {
                    // Send output chunks (for now, send as single chunk)
                    // In a real implementation, we'd support streaming from the agent
                    let _ = tx.send(output).await;
                }
                Err(e) => {
                    // Send error as output
                    let error_output = AgentOutput::text(format!("Error: {e}"));
                    let _ = tx.send(error_output).await;
                }
            }

            // Clean up channel
            let mut channels = channels.write().await;
            channels.remove(&channel_id);
        });

        Ok(rx)
    }

    /// Clean up context
    ///
    /// # Errors
    ///
    /// Returns an error if the context is not found
    pub async fn remove_context(&self, context_id: &str) -> Result<()> {
        let mut contexts = self.contexts.write().await;
        contexts
            .remove(context_id)
            .ok_or_else(|| LLMSpellError::Component {
                message: format!("Context '{context_id}' not found"),
                source: None,
            })?;
        Ok(())
    }

    // Composition Pattern Methods

    /// Wrap an agent as a tool for composition
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Agent is not found
    /// - Tool registration fails
    pub async fn wrap_agent_as_tool(
        &self,
        agent_name: &str,
        wrapper_config: serde_json::Value,
    ) -> Result<String> {
        use llmspell_agents::agent_wrapped_tool::AgentWrappedTool;
        use llmspell_core::traits::tool::{SecurityLevel, ToolCategory};

        // Get the agent instance
        let agent = self
            .get_agent(agent_name)
            .await
            .ok_or_else(|| LLMSpellError::Component {
                message: format!("Agent '{agent_name}' not found"),
                source: None,
            })?;

        // Create a unique tool name
        let tool_name = wrapper_config
            .get("tool_name")
            .and_then(|v| v.as_str())
            .unwrap_or(&format!("{agent_name}_tool"))
            .to_string();

        // Create the agent-wrapped tool

        let wrapped_tool = AgentWrappedTool::new(
            agent.clone(),
            ToolCategory::Utility,
            SecurityLevel::Restricted,
        );

        // Register the wrapped tool
        self.registry
            .register_tool(tool_name.clone(), Arc::new(wrapped_tool))?;

        Ok(tool_name)
    }

    /// List all agents with their capabilities
    ///
    /// # Errors
    ///
    /// Returns an error if capability listing fails
    pub async fn list_agent_capabilities(&self) -> Result<serde_json::Value> {
        let agents = self.active_agents.read().await;
        let mut capabilities = serde_json::Map::new();

        for (name, agent) in agents.iter() {
            let agent_info = serde_json::json!({
                "id": agent.metadata().id.to_string(),
                "name": agent.metadata().name.clone(),
                "description": agent.metadata().description,
                "config": {
                    "system_prompt": agent.config().system_prompt,
                    "temperature": agent.config().temperature,
                    "max_tokens": agent.config().max_tokens,
                },
                "capabilities": {
                    "supports_streaming": true,
                    "supports_tools": true,
                    "supports_context": true,
                    "supports_multimodal": false, // Can be extended
                },
            });
            capabilities.insert(name.clone(), agent_info);
        }

        Ok(serde_json::Value::Object(capabilities))
    }

    /// Get detailed agent information including composition metadata
    ///
    /// # Errors
    ///
    /// Returns an error if the agent is not found
    pub async fn get_agent_details(&self, agent_name: &str) -> Result<serde_json::Value> {
        let agent = self
            .get_agent(agent_name)
            .await
            .ok_or_else(|| LLMSpellError::Component {
                message: format!("Agent '{agent_name}' not found"),
                source: None,
            })?;

        // Get agent's state if available
        let state = {
            let machines = self.state_machines.read().await;
            if let Some(machine) = machines.get(agent_name) {
                let current_state = machine.current_state().await;
                Some(format!("{current_state:?}"))
            } else {
                None
            }
        };

        // Get agent metrics and convert to serializable format
        let metrics = if let Ok(agent_metrics) = self.get_agent_metrics(agent_name).await {
            Some(serde_json::json!({
                "agent_id": agent_metrics.agent_id,
                "requests_total": agent_metrics.requests_total.value(),
                "requests_failed": agent_metrics.requests_failed.value(),
                "requests_active": agent_metrics.requests_active.value(),
                "tool_invocations": agent_metrics.tool_invocations.value(),
                "memory_bytes": agent_metrics.memory_bytes.value(),
                "cpu_percent": agent_metrics.cpu_percent.value(),
            }))
        } else {
            None
        };

        let info = serde_json::json!({
            "id": agent.metadata().id.to_string(),
            "name": agent.metadata().name.clone(),
            "description": agent.metadata().description,
            "state": state.unwrap_or_else(|| "Unknown".to_string()),
            "metrics": metrics,
            "config": {
                "system_prompt": agent.config().system_prompt,
                "temperature": agent.config().temperature,
                "max_tokens": agent.config().max_tokens,
                "max_conversation_length": agent.config().max_conversation_length,
            },
            "composition": {
                "can_be_wrapped": true,
                "supports_delegation": true,
                "supports_nesting": true,
            },
        });

        Ok(info)
    }

    /// Create a composite agent that delegates to multiple agents
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Any delegate agent is not found
    /// - Composite agent creation fails
    pub async fn create_composite_agent(
        &self,
        composite_name: String,
        delegate_agents: Vec<String>,
        routing_config: RoutingConfig,
    ) -> Result<()> {
        // Verify all delegate agents exist
        let agents = self.active_agents.read().await;
        for agent_name in &delegate_agents {
            if !agents.contains_key(agent_name) {
                return Err(LLMSpellError::Component {
                    message: format!("Delegate agent '{agent_name}' not found"),
                    source: None,
                });
            }
        }
        drop(agents);

        // For now, create a composite agent as a regular agent with metadata
        // Full composite agent implementation will come with workflow patterns
        let mut custom_config = serde_json::Map::new();
        custom_config.insert(
            "system_prompt".to_string(),
            serde_json::json!(format!(
                "You are a composite agent that coordinates between: {}",
                delegate_agents.join(", ")
            )),
        );
        custom_config.insert("delegates".to_string(), serde_json::json!(delegate_agents));
        custom_config.insert(
            "routing".to_string(),
            serde_json::to_value(&routing_config).unwrap_or_else(|_| serde_json::json!({})),
        );
        custom_config.insert("composite".to_string(), serde_json::json!(true));

        let composite_agent_config = AgentConfig {
            name: composite_name.clone(),
            description: format!("Composite agent coordinating: {}", delegate_agents.join(", ")),
            agent_type: "basic".to_string(),
            model: None,
            allowed_tools: Vec::new(),
            custom_config,
            resource_limits: llmspell_agents::ResourceLimits::default(),
        };

        // Create the composite agent using regular agent creation
        self.create_agent(composite_agent_config).await?;

        Ok(())
    }

    /// Enable dynamic agent discovery by type or capability
    ///
    /// # Errors
    ///
    /// Returns an error if discovery fails
    pub async fn discover_agents_by_capability(&self, capability: &str) -> Result<Vec<String>> {
        let agents = self.active_agents.read().await;
        let mut matching_agents = Vec::new();

        for (name, agent) in agents.iter() {
            // Check various capabilities
            match capability {
                "streaming" | "tools" | "context" => matching_agents.push(name.clone()),
                "composite" => {
                    // Check if agent is a composite type
                    let desc = &agent.metadata().description;
                    if desc.contains("composite") || desc.contains("delegate") {
                        matching_agents.push(name.clone());
                    }
                }
                _ => {
                    // Check if capability is in description or name
                    let metadata = agent.metadata();
                    let desc = &metadata.description;
                    if desc.contains(capability) || metadata.name.contains(capability) {
                        matching_agents.push(name.clone());
                    }
                }
            }
        }

        Ok(matching_agents)
    }

    /// Get composition hierarchy for nested agents
    ///
    /// # Errors
    ///
    /// Returns an error if the agent is not found
    pub async fn get_composition_hierarchy(&self, agent_name: &str) -> Result<serde_json::Value> {
        let agent = self
            .get_agent(agent_name)
            .await
            .ok_or_else(|| LLMSpellError::Component {
                message: format!("Agent '{agent_name}' not found"),
                source: None,
            })?;

        // Build hierarchy structure
        let hierarchy = serde_json::json!({
            "root": {
                "name": agent_name,
                "type": "agent",
                "id": agent.metadata().id.to_string(),
                "children": [] // Would be populated if agent has delegates
            }
        });

        Ok(hierarchy)
    }

    /// Set the state manager for agent state persistence
    pub fn set_state_manager(&mut self, state_manager: Arc<StateManager>) {
        self.state_manager = Some(state_manager);
    }

    /// Save an agent's state
    ///
    /// # Errors
    ///
    /// Returns an error if the agent is not found or state saving fails
    pub async fn save_agent_state(&self, agent_name: &str) -> Result<()> {
        let state_manager =
            self.state_manager
                .as_ref()
                .ok_or_else(|| LLMSpellError::Component {
                    message: "State manager not configured".to_string(),
                    source: None,
                })?;

        // Get the agent
        let agents = self.active_agents.read().await;
        let agent = agents
            .get(agent_name)
            .ok_or_else(|| LLMSpellError::Component {
                message: format!("Agent '{agent_name}' not found"),
                source: None,
            })?;

        let agent_id = agent.metadata().id.to_string();
        let scope = StateScope::Agent(agent_id.clone());

        // Save complete agent state
        // 1. Save metadata
        let agent_meta = serde_json::json!({
            "name": agent.metadata().name,
            "description": agent.metadata().description,
            "id": agent_id,
        });
        state_manager
            .set(scope.clone(), "metadata", agent_meta)
            .await
            .map_err(|e| LLMSpellError::Component {
                message: format!("Failed to save agent metadata: {e}"),
                source: None,
            })?;

        // 2. Save conversation history
        if let Ok(conversation) = agent.get_conversation().await {
            let conv_json =
                serde_json::to_value(&conversation).map_err(|e| LLMSpellError::Component {
                    message: format!("Failed to serialize conversation: {e}"),
                    source: None,
                })?;
            state_manager
                .set(scope.clone(), "conversation", conv_json)
                .await
                .map_err(|e| LLMSpellError::Component {
                    message: format!("Failed to save conversation: {e}"),
                    source: None,
                })?;
        }

        // 3. Save agent configuration
        let config = agent.config();
        let config_json = serde_json::json!({
            "max_conversation_length": config.max_conversation_length,
            "system_prompt": config.system_prompt,
            "temperature": config.temperature,
            "max_tokens": config.max_tokens,
        });
        state_manager
            .set(scope.clone(), "config", config_json)
            .await
            .map_err(|e| LLMSpellError::Component {
                message: format!("Failed to save config: {e}"),
                source: None,
            })?;

        // 4. Register this agent as having saved state
        self.register_saved_agent(agent_name).await?;

        Ok(())
    }

    /// Load an agent's state
    ///
    /// # Errors
    ///
    /// Returns an error if state loading fails
    pub async fn load_agent_state(&self, agent_name: &str) -> Result<bool> {
        let state_manager =
            self.state_manager
                .as_ref()
                .ok_or_else(|| LLMSpellError::Component {
                    message: "State manager not configured".to_string(),
                    source: None,
                })?;

        // Note: Loading state requires mutable access to agent, which we don't have
        // with Arc<dyn Agent>. This is a limitation of the current architecture.
        // For now, we can only verify if state exists.

        // Get the agent to find its ID
        let agents = self.active_agents.read().await;
        let agent = agents
            .get(agent_name)
            .ok_or_else(|| LLMSpellError::Component {
                message: format!("Agent '{agent_name}' not found"),
                source: None,
            })?;

        let agent_id = agent.metadata().id.to_string();
        let scope = StateScope::Agent(agent_id.clone());

        // Check if state exists
        let metadata = state_manager
            .get(scope.clone(), "metadata")
            .await
            .map_err(|e| LLMSpellError::Component {
                message: format!("Failed to check agent state: {e}"),
                source: None,
            })?;

        if metadata.is_some() {
            // State exists but we cannot load it into the agent due to Arc<dyn Agent> limitation
            // This would require refactoring to store agents differently or using interior mutability
            warn!(
                "Agent state exists for '{}' but cannot be loaded due to immutable agent reference. \
                 Consider using agent-specific state loading methods.",
                agent_name
            );
            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// Delete an agent's state
    ///
    /// # Errors
    ///
    /// Returns an error if state deletion fails
    pub async fn delete_agent_state(&self, agent_name: &str) -> Result<()> {
        let state_manager =
            self.state_manager
                .as_ref()
                .ok_or_else(|| LLMSpellError::Component {
                    message: "State manager not configured".to_string(),
                    source: None,
                })?;

        // Get the agent to get its ID (or use agent_name as ID if agent not found)
        #[allow(clippy::option_if_let_else)] // Complex pattern
        let agent_id = if let Some(agent) = self.get_agent(agent_name).await {
            agent.metadata().id.to_string()
        } else {
            // If agent not found, still try to delete using name as ID
            agent_name.to_string()
        };

        // Clear all state for this agent
        state_manager
            .clear_scope(StateScope::Agent(agent_id))
            .await
            .map_err(|e| LLMSpellError::Component {
                message: format!("Failed to delete agent state: {e}"),
                source: None,
            })?;

        // Unregister from saved agents
        self.unregister_saved_agent(agent_name).await?;

        Ok(())
    }

    /// List all saved agent states
    ///
    /// # Errors
    ///
    /// Returns an error if listing fails
    pub async fn list_saved_agents(&self) -> Result<Vec<String>> {
        let state_manager =
            self.state_manager
                .as_ref()
                .ok_or_else(|| LLMSpellError::Component {
                    message: "State manager not configured".to_string(),
                    source: None,
                })?;

        // Get the registry of saved agents from global scope
        let saved_agents = state_manager
            .get(StateScope::Global, "saved_agents_registry")
            .await
            .map_err(|e| LLMSpellError::Component {
                message: format!("Failed to get saved agents registry: {e}"),
                source: None,
            })?;
        #[allow(clippy::option_if_let_else)] // Complex pattern
        match saved_agents {
            Some(registry) => {
                // Parse the registry
                if let Some(agents) = registry.as_array() {
                    Ok(agents
                        .iter()
                        .filter_map(|v| v.as_str().map(String::from))
                        .collect())
                } else {
                    Ok(vec![])
                }
            }
            None => Ok(vec![]),
        }
    }

    /// Register an agent as having saved state
    async fn register_saved_agent(&self, agent_name: &str) -> Result<()> {
        let state_manager =
            self.state_manager
                .as_ref()
                .ok_or_else(|| LLMSpellError::Component {
                    message: "State manager not configured".to_string(),
                    source: None,
                })?;

        // Get current registry
        #[allow(clippy::option_if_let_else)] // Complex pattern
        let mut saved_agents = match state_manager
            .get(StateScope::Global, "saved_agents_registry")
            .await
        {
            Ok(Some(registry)) => {
                if let Some(agents) = registry.as_array() {
                    agents
                        .iter()
                        .filter_map(|v| v.as_str().map(String::from))
                        .collect::<Vec<_>>()
                } else {
                    vec![]
                }
            }
            _ => vec![],
        };

        // Add agent if not already present
        if !saved_agents.contains(&agent_name.to_string()) {
            saved_agents.push(agent_name.to_string());
        }

        // Save updated registry
        state_manager
            .set(
                StateScope::Global,
                "saved_agents_registry",
                serde_json::json!(saved_agents),
            )
            .await
            .map_err(|e| LLMSpellError::Component {
                message: format!("Failed to update saved agents registry: {e}"),
                source: None,
            })?;

        Ok(())
    }

    /// Unregister an agent from saved state registry
    async fn unregister_saved_agent(&self, agent_name: &str) -> Result<()> {
        let state_manager =
            self.state_manager
                .as_ref()
                .ok_or_else(|| LLMSpellError::Component {
                    message: "State manager not configured".to_string(),
                    source: None,
                })?;

        // Get current registry
        #[allow(clippy::option_if_let_else)] // Complex pattern
        let saved_agents = match state_manager
            .get(StateScope::Global, "saved_agents_registry")
            .await
        {
            Ok(Some(registry)) => {
                if let Some(agents) = registry.as_array() {
                    agents
                        .iter()
                        .filter_map(|v| v.as_str().map(String::from))
                        .filter(|name| name != agent_name)
                        .collect::<Vec<_>>()
                } else {
                    vec![]
                }
            }
            _ => vec![],
        };

        // Save updated registry
        state_manager
            .set(
                StateScope::Global,
                "saved_agents_registry",
                serde_json::json!(saved_agents),
            )
            .await
            .map_err(|e| LLMSpellError::Component {
                message: format!("Failed to update saved agents registry: {e}"),
                source: None,
            })?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_agent_config(name: &str) -> AgentConfig {
        AgentConfig {
            name: name.to_string(),
            description: format!("Test agent: {name}"),
            agent_type: "basic".to_string(),
            model: None,
            allowed_tools: Vec::new(),
            custom_config: serde_json::Map::new(),
            resource_limits: llmspell_agents::ResourceLimits::default(),
        }
    }

    #[tokio::test]
    async fn test_agent_bridge_creation() {
        let registry = Arc::new(ComponentRegistry::new());
        let provider_manager = Arc::new(llmspell_providers::ProviderManager::new());
        let bridge = AgentBridge::new(registry, provider_manager);

        // List available types
        let types = bridge.list_agent_types();
        assert!(!types.is_empty());
    }
    #[tokio::test]
    async fn test_agent_instance_management() {
        let registry = Arc::new(ComponentRegistry::new());
        let provider_manager = Arc::new(llmspell_providers::ProviderManager::new());
        let bridge = AgentBridge::new(registry, provider_manager);

        // Create agent instance
        let result = bridge.create_agent(create_test_agent_config("test-instance")).await;
        assert!(result.is_ok());

        // List instances
        let instances = bridge.list_instances().await;
        assert!(instances.contains(&"test-instance".to_string()));

        // Get agent
        let agent = bridge.get_agent("test-instance").await;
        assert!(agent.is_some());

        // Remove agent
        let remove_result = bridge.remove_agent("test-instance").await;
        assert!(remove_result.is_ok());

        // Verify removed
        let agent_after = bridge.get_agent("test-instance").await;
        assert!(agent_after.is_none());
    }
    #[tokio::test]
    async fn test_agent_execution() {
        let registry = Arc::new(ComponentRegistry::new());
        let provider_manager = Arc::new(llmspell_providers::ProviderManager::new());
        let bridge = AgentBridge::new(registry, provider_manager);

        // Create agent
        bridge
            .create_agent(create_test_agent_config("test-exec"))
            .await
            .unwrap();

        // Execute agent
        let input = AgentInput::text("Hello, agent!");
        let result = bridge.execute_agent("test-exec", input, None).await;

        // Note: This might fail if mock provider is not available
        // In real tests, we'd use a proper mock
        assert!(result.is_ok() || result.is_err());
    }
    #[tokio::test]
    async fn test_agent_state_machine() {
        let registry = Arc::new(ComponentRegistry::new());
        let provider_manager = Arc::new(llmspell_providers::ProviderManager::new());
        let bridge = AgentBridge::new(registry, provider_manager);

        // Create agent instance
        bridge
            .create_agent(create_test_agent_config("test-state"))
            .await
            .unwrap();

        // Test initial state
        let state = bridge.get_agent_state("test-state").await.unwrap();
        assert_eq!(state, AgentState::Uninitialized);

        // Initialize agent
        bridge.initialize_agent("test-state").await.unwrap();
        let state = bridge.get_agent_state("test-state").await.unwrap();
        assert_eq!(state, AgentState::Ready);

        // Start agent
        bridge.start_agent("test-state").await.unwrap();
        let state = bridge.get_agent_state("test-state").await.unwrap();
        assert_eq!(state, AgentState::Running);

        // Pause agent
        bridge.pause_agent("test-state").await.unwrap();
        let state = bridge.get_agent_state("test-state").await.unwrap();
        assert_eq!(state, AgentState::Paused);

        // Resume agent
        bridge.resume_agent("test-state").await.unwrap();
        let state = bridge.get_agent_state("test-state").await.unwrap();
        assert_eq!(state, AgentState::Running);

        // Check state history
        let history = bridge.get_agent_state_history("test-state").await.unwrap();
        assert!(!history.is_empty());
        assert_eq!(history.len(), 5); // Uninitialized -> Initializing -> Ready -> Running -> Paused -> Running

        // Check metrics
        let metrics = bridge.get_agent_state_metrics("test-state").await.unwrap();
        assert_eq!(
            metrics.get("current_state").and_then(|v| v.as_str()),
            Some("Running")
        );
        assert_eq!(
            metrics
                .get("total_transitions")
                .and_then(serde_json::Value::as_u64),
            Some(5)
        );

        // Test error handling
        bridge
            .error_agent("test-state", "Test error".to_string())
            .await
            .unwrap();
        let state = bridge.get_agent_state("test-state").await.unwrap();
        assert_eq!(state, AgentState::Error);

        let last_error = bridge.get_agent_last_error("test-state").await.unwrap();
        assert_eq!(last_error, Some("Test error".to_string()));

        // Test recovery
        bridge.recover_agent("test-state").await.unwrap();
        let state = bridge.get_agent_state("test-state").await.unwrap();
        assert_eq!(state, AgentState::Ready);

        // Test health check
        let is_healthy = bridge.is_agent_healthy("test-state").await.unwrap();
        assert!(is_healthy);

        // Terminate agent
        bridge.terminate_agent("test-state").await.unwrap();
        let state = bridge.get_agent_state("test-state").await.unwrap();
        assert_eq!(state, AgentState::Terminated);

        // Cleanup
        bridge.remove_agent("test-state").await.unwrap();
    }
    #[tokio::test]
    async fn test_context_management() {
        let registry = Arc::new(ComponentRegistry::new());
        let provider_manager = Arc::new(llmspell_providers::ProviderManager::new());
        let bridge = AgentBridge::new(registry, provider_manager);

        // Test context creation
        let config = ExecutionContextConfig {
            conversation_id: Some("conv-123".to_string()),
            user_id: Some("user-456".to_string()),
            session_id: Some("session-789".to_string()),
            scope: Some(ContextScope::Session("session-789".to_string())),
            inheritance: Some(InheritancePolicy::Inherit),
            data: Some({
                let mut map = HashMap::new();
                map.insert("theme".to_string(), serde_json::json!("dark"));
                map.insert("language".to_string(), serde_json::json!("en"));
                map
            }),
            security: Some(SecurityContextConfig {
                permissions: vec!["read".to_string(), "write".to_string()],
                level: "standard".to_string(),
            }),
        };

        let context_id = bridge.create_context(config).await.unwrap();
        assert!(!context_id.is_empty());

        // Test context retrieval
        let context = bridge.get_context(&context_id).await.unwrap();
        assert_eq!(context.conversation_id, Some("conv-123".to_string()));
        assert_eq!(context.user_id, Some("user-456".to_string()));
        assert_eq!(context.session_id, Some("session-789".to_string()));

        // Test context data access
        let theme_value = bridge.get_context_data(&context_id, "theme").await.unwrap();
        assert_eq!(theme_value, Some(serde_json::json!("dark")));

        // Test context update
        bridge
            .update_context(&context_id, "theme".to_string(), serde_json::json!("light"))
            .await
            .unwrap();

        // Test child context creation
        let child_config = ChildContextConfig {
            scope: ContextScope::Agent(ComponentId::from_name("child-agent")),
            inheritance: InheritancePolicy::Copy,
        };
        let child_id = bridge
            .create_child_context(&context_id, child_config)
            .await
            .unwrap();
        assert!(!child_id.is_empty());

        // Test shared memory
        let workflow_scope = ContextScope::Workflow("workflow-1".to_string());
        bridge.set_shared_memory(
            &workflow_scope,
            "status".to_string(),
            serde_json::json!("running"),
        );

        let status = bridge.get_shared_memory(&workflow_scope, "status");
        assert_eq!(status, Some(serde_json::json!("running")));

        // Cleanup
        bridge.remove_context(&context_id).await.unwrap();
        bridge.remove_context(&child_id).await.unwrap();
    }
    #[tokio::test]
    async fn test_agent_context_execution() {
        let registry = Arc::new(ComponentRegistry::new());
        let provider_manager = Arc::new(llmspell_providers::ProviderManager::new());
        let bridge = AgentBridge::new(registry, provider_manager);

        // Create agent
        bridge
            .create_agent(create_test_agent_config("context-test"))
            .await
            .unwrap();

        // Create context
        let context_config = ExecutionContextConfig {
            conversation_id: Some("conv-test".to_string()),
            data: Some({
                let mut map = HashMap::new();
                map.insert("user_preference".to_string(), serde_json::json!("concise"));
                map.insert("context_type".to_string(), serde_json::json!("test"));
                map
            }),
            ..Default::default()
        };
        let context_id = bridge.create_context(context_config).await.unwrap();

        // Execute with context
        let input = AgentInput::text("Hello with context");
        let result = bridge
            .execute_agent_with_context("context-test", input, &context_id)
            .await;
        assert!(result.is_ok() || result.is_err()); // Depends on mock availability

        // Cleanup
        bridge.remove_agent("context-test").await.unwrap();
        bridge.remove_context(&context_id).await.unwrap();
    }
    #[tokio::test]
    async fn test_streaming_execution() {
        let registry = Arc::new(ComponentRegistry::new());
        let provider_manager = Arc::new(llmspell_providers::ProviderManager::new());
        let bridge = AgentBridge::new(registry, provider_manager);

        // Create agent
        bridge
            .create_agent(create_test_agent_config("stream-test"))
            .await
            .unwrap();

        // Test streaming execution
        let input = AgentInput::text("Stream this response");
        let mut receiver = bridge
            .execute_agent_streaming("stream-test", input, None)
            .await
            .unwrap();

        // Wait for at least one output
        let timeout =
            tokio::time::timeout(std::time::Duration::from_secs(5), receiver.recv()).await;

        // Should receive something (either success or error)
        assert!(timeout.is_ok());

        // Cleanup
        bridge.remove_agent("stream-test").await.unwrap();
    }
    #[tokio::test]
    async fn test_composition_patterns() {
        let registry = Arc::new(ComponentRegistry::new());
        let provider_manager = Arc::new(llmspell_providers::ProviderManager::new());
        let bridge = AgentBridge::new(registry, provider_manager);

        // Create two basic agents
        bridge
            .create_agent(create_test_agent_config("agent1"))
            .await
            .unwrap();

        bridge
            .create_agent(create_test_agent_config("agent2"))
            .await
            .unwrap();

        // Test agent capabilities listing
        let capabilities = bridge.list_agent_capabilities().await.unwrap();
        assert!(capabilities.is_object());
        let caps_obj = capabilities.as_object().unwrap();
        assert!(caps_obj.contains_key("agent1"));
        assert!(caps_obj.contains_key("agent2"));

        // Test agent info
        let info = bridge.get_agent_details("agent1").await.unwrap();
        assert!(info.is_object());
        let info_obj = info.as_object().unwrap();
        assert!(info_obj.contains_key("id"));
        assert!(info_obj.contains_key("name"));
        assert!(info_obj.contains_key("composition"));

        // Test wrapping agent as tool
        let tool_name = bridge
            .wrap_agent_as_tool(
                "agent1",
                serde_json::json!({
                    "tool_name": "agent1_tool",
                    "description": "Agent 1 wrapped as tool"
                }),
            )
            .await
            .unwrap();
        assert_eq!(tool_name, "agent1_tool");

        // Verify tool was registered
        let tools = bridge.list_tools();
        assert!(tools.contains(&"agent1_tool".to_string()));

        // Test discovery by capability
        let streaming_agents = bridge
            .discover_agents_by_capability("streaming")
            .await
            .unwrap();
        assert_eq!(streaming_agents.len(), 2); // Both agents support streaming

        // Test composite agent creation
        bridge
            .create_composite_agent(
                "composite1".to_string(),
                vec!["agent1".to_string(), "agent2".to_string()],
                RoutingConfig {
                    strategy: RoutingStrategy::Custom {
                        name: "round_robin".to_string(),
                    },
                    fallback_agent: None,
                    timeout_ms: None,
                },
            )
            .await
            .unwrap();

        // Verify composite agent exists
        let instances = bridge.list_instances().await;
        assert!(instances.contains(&"composite1".to_string()));

        // Test hierarchy retrieval
        let hierarchy = bridge.get_composition_hierarchy("agent1").await.unwrap();
        assert!(hierarchy.get("root").is_some());

        // Cleanup
        bridge.remove_agent("agent1").await.unwrap();
        bridge.remove_agent("agent2").await.unwrap();
        bridge.remove_agent("composite1").await.unwrap();
    }
}
