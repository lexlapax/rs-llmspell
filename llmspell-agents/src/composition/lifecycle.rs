//! ABOUTME: Lifecycle management for composite agents
//! ABOUTME: Handles initialization, state transitions, and cleanup of composed agents

use super::traits::{CompositeAgent, HierarchicalAgent, HierarchyEvent};
use async_trait::async_trait;
use llmspell_core::{BaseAgent, LLMSpellError, Result};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Lifecycle state of a composite agent
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LifecycleState {
    /// Agent is being initialized
    Initializing,
    /// Agent is ready to accept work
    Ready,
    /// Agent is actively processing
    Active,
    /// Agent is paused
    Paused,
    /// Agent is shutting down
    ShuttingDown,
    /// Agent has terminated
    Terminated,
}

/// Lifecycle manager for composite agents
pub struct CompositeLifecycleManager {
    /// Current state
    state: RwLock<LifecycleState>,
    /// Managed components
    components: RwLock<HashMap<String, ComponentLifecycle>>,
    /// Event handlers
    event_handlers: RwLock<Vec<Box<dyn LifecycleEventHandler>>>,
    /// Configuration
    config: LifecycleConfig,
}

/// Lifecycle information for a single component
#[derive(Clone)]
pub struct ComponentLifecycle {
    /// Component ID
    pub id: String,
    /// Component reference
    pub component: Arc<dyn BaseAgent>,
    /// Component state
    pub state: LifecycleState,
    /// Initialization timestamp
    pub initialized_at: Option<chrono::DateTime<chrono::Utc>>,
    /// Last activity timestamp
    pub last_active: Option<chrono::DateTime<chrono::Utc>>,
    /// Error count
    pub error_count: u32,
}

impl std::fmt::Debug for ComponentLifecycle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ComponentLifecycle")
            .field("id", &self.id)
            .field("state", &self.state)
            .field("initialized_at", &self.initialized_at)
            .field("last_active", &self.last_active)
            .field("error_count", &self.error_count)
            .finish()
    }
}

/// Configuration for lifecycle management
#[derive(Debug, Clone)]
pub struct LifecycleConfig {
    /// Maximum initialization time
    pub init_timeout: std::time::Duration,
    /// Maximum shutdown time
    pub shutdown_timeout: std::time::Duration,
    /// Whether to cascade lifecycle events to children
    pub cascade_events: bool,
    /// Whether to wait for all components during state transitions
    pub wait_for_all: bool,
    /// Health check interval
    pub health_check_interval: Option<std::time::Duration>,
}

impl Default for LifecycleConfig {
    fn default() -> Self {
        Self {
            init_timeout: std::time::Duration::from_secs(30),
            shutdown_timeout: std::time::Duration::from_secs(30),
            cascade_events: true,
            wait_for_all: true,
            health_check_interval: Some(std::time::Duration::from_secs(60)),
        }
    }
}

/// Event handler for lifecycle events
#[async_trait]
pub trait LifecycleEventHandler: Send + Sync {
    /// Handle a lifecycle event
    async fn handle_event(&self, event: &LifecycleEvent) -> Result<()>;
}

/// Lifecycle events
#[derive(Debug, Clone)]
pub enum LifecycleEvent {
    /// State transition event
    StateTransition {
        component_id: String,
        from: LifecycleState,
        to: LifecycleState,
        timestamp: chrono::DateTime<chrono::Utc>,
    },
    /// Component added
    ComponentAdded {
        component_id: String,
        parent_id: Option<String>,
    },
    /// Component removed
    ComponentRemoved {
        component_id: String,
        reason: String,
    },
    /// Error occurred
    Error {
        component_id: String,
        error: String,
        severity: ErrorSeverity,
    },
    /// Health check result
    HealthCheck {
        component_id: String,
        healthy: bool,
        details: HashMap<String, String>,
    },
}

/// Error severity levels
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ErrorSeverity {
    /// Informational
    Info,
    /// Warning
    Warning,
    /// Error
    Error,
    /// Critical error
    Critical,
}

impl CompositeLifecycleManager {
    /// Create a new lifecycle manager
    #[must_use]
    pub fn new(config: LifecycleConfig) -> Self {
        Self {
            state: RwLock::new(LifecycleState::Initializing),
            components: RwLock::new(HashMap::new()),
            event_handlers: RwLock::new(Vec::new()),
            config,
        }
    }

    /// Initialize a composite agent
    pub async fn initialize_composite(&self, agent: &dyn CompositeAgent) -> Result<()> {
        // Set state to initializing
        *self.state.write().await = LifecycleState::Initializing;

        // Initialize all components
        let components = agent.components();

        // If no components, just transition to ready
        if components.is_empty() {
            *self.state.write().await = LifecycleState::Ready;
            return Ok(());
        }

        let mut component_lifecycles = self.components.write().await;

        for component in components {
            let component_id = component.metadata().id.to_string();

            // Create lifecycle entry
            let lifecycle = ComponentLifecycle {
                id: component_id.clone(),
                component: component.clone(),
                state: LifecycleState::Initializing,
                initialized_at: None,
                last_active: None,
                error_count: 0,
            };

            component_lifecycles.insert(component_id.clone(), lifecycle);

            // Emit event
            self.emit_event(LifecycleEvent::ComponentAdded {
                component_id,
                parent_id: Some(agent.metadata().id.to_string()),
            })
            .await?;
        }

        // Initialize components with timeout
        let init_timeout = self.config.init_timeout;
        match tokio::time::timeout(init_timeout, self.initialize_all_components()).await {
            Ok(Ok(())) => {
                *self.state.write().await = LifecycleState::Ready;
                Ok(())
            }
            Ok(Err(e)) => Err(e),
            Err(_) => Err(LLMSpellError::Component {
                message: "Initialization timeout".to_string(),
                source: None,
            }),
        }
    }

    /// Initialize all components
    async fn initialize_all_components(&self) -> Result<()> {
        let mut components = self.components.write().await;

        for (id, lifecycle) in components.iter_mut() {
            lifecycle.state = LifecycleState::Ready;
            lifecycle.initialized_at = Some(chrono::Utc::now());

            self.emit_event(LifecycleEvent::StateTransition {
                component_id: id.clone(),
                from: LifecycleState::Initializing,
                to: LifecycleState::Ready,
                timestamp: chrono::Utc::now(),
            })
            .await?;
        }

        Ok(())
    }

    /// Activate the composite agent
    pub async fn activate(&self) -> Result<()> {
        self.transition_state(LifecycleState::Active).await
    }

    /// Pause the composite agent
    pub async fn pause(&self) -> Result<()> {
        self.transition_state(LifecycleState::Paused).await
    }

    /// Resume from pause
    pub async fn resume(&self) -> Result<()> {
        let current = self.state.read().await.clone();
        if current != LifecycleState::Paused {
            return Err(LLMSpellError::Component {
                message: "Can only resume from paused state".to_string(),
                source: None,
            });
        }
        self.transition_state(LifecycleState::Active).await
    }

    /// Shutdown the composite agent
    pub async fn shutdown(&self) -> Result<()> {
        *self.state.write().await = LifecycleState::ShuttingDown;

        // Shutdown all components
        let shutdown_timeout = self.config.shutdown_timeout;
        match tokio::time::timeout(shutdown_timeout, self.shutdown_all_components()).await {
            Ok(Ok(())) => {
                *self.state.write().await = LifecycleState::Terminated;
                Ok(())
            }
            Ok(Err(e)) => Err(e),
            Err(_) => {
                // Force terminate on timeout
                *self.state.write().await = LifecycleState::Terminated;
                Err(LLMSpellError::Component {
                    message: "Shutdown timeout - forced termination".to_string(),
                    source: None,
                })
            }
        }
    }

    /// Shutdown all components
    async fn shutdown_all_components(&self) -> Result<()> {
        let mut components = self.components.write().await;

        for (id, lifecycle) in components.iter_mut() {
            lifecycle.state = LifecycleState::Terminated;

            self.emit_event(LifecycleEvent::StateTransition {
                component_id: id.clone(),
                from: LifecycleState::Active,
                to: LifecycleState::Terminated,
                timestamp: chrono::Utc::now(),
            })
            .await?;
        }

        Ok(())
    }

    /// Transition to a new state
    async fn transition_state(&self, new_state: LifecycleState) -> Result<()> {
        let current = self.state.read().await.clone();

        // Validate transition
        if !self.is_valid_transition(&current, &new_state) {
            return Err(LLMSpellError::Component {
                message: format!("Invalid state transition: {current:?} -> {new_state:?}"),
                source: None,
            });
        }

        // Update state
        *self.state.write().await = new_state.clone();

        // Update component states if cascading
        if self.config.cascade_events {
            let mut components = self.components.write().await;
            for (id, lifecycle) in components.iter_mut() {
                lifecycle.state = new_state.clone();

                self.emit_event(LifecycleEvent::StateTransition {
                    component_id: id.clone(),
                    from: current.clone(),
                    to: new_state.clone(),
                    timestamp: chrono::Utc::now(),
                })
                .await?;
            }
        }

        Ok(())
    }

    /// Check if a state transition is valid
    const fn is_valid_transition(&self, from: &LifecycleState, to: &LifecycleState) -> bool {
        matches!(
            (from, to),
            (LifecycleState::Initializing, LifecycleState::Ready)
                | (
                    LifecycleState::Ready | LifecycleState::Paused,
                    LifecycleState::Active | LifecycleState::ShuttingDown
                )
                | (
                    LifecycleState::Active,
                    LifecycleState::Paused | LifecycleState::ShuttingDown
                )
                | (LifecycleState::ShuttingDown, LifecycleState::Terminated)
        )
    }

    /// Add a component dynamically
    pub async fn add_component(&self, component: Arc<dyn BaseAgent>) -> Result<()> {
        let component_id = component.metadata().id.to_string();

        let lifecycle = ComponentLifecycle {
            id: component_id.clone(),
            component: component.clone(),
            state: LifecycleState::Ready,
            initialized_at: Some(chrono::Utc::now()),
            last_active: None,
            error_count: 0,
        };

        self.components
            .write()
            .await
            .insert(component_id.clone(), lifecycle);

        self.emit_event(LifecycleEvent::ComponentAdded {
            component_id,
            parent_id: None,
        })
        .await
    }

    /// Remove a component
    pub async fn remove_component(&self, component_id: &str, reason: &str) -> Result<()> {
        self.components.write().await.remove(component_id);

        self.emit_event(LifecycleEvent::ComponentRemoved {
            component_id: component_id.to_string(),
            reason: reason.to_string(),
        })
        .await
    }

    /// Register an event handler
    pub async fn register_handler(&self, handler: Box<dyn LifecycleEventHandler>) {
        self.event_handlers.write().await.push(handler);
    }

    /// Emit a lifecycle event
    async fn emit_event(&self, event: LifecycleEvent) -> Result<()> {
        let handlers = self.event_handlers.read().await;
        for handler in handlers.iter() {
            handler.handle_event(&event).await?;
        }
        Ok(())
    }

    /// Get current state
    pub async fn state(&self) -> LifecycleState {
        self.state.read().await.clone()
    }

    /// Get component states
    pub async fn component_states(&self) -> HashMap<String, LifecycleState> {
        self.components
            .read()
            .await
            .iter()
            .map(|(id, lifecycle)| (id.clone(), lifecycle.state.clone()))
            .collect()
    }

    /// Perform health check
    pub async fn health_check(&self) -> Result<HealthCheckResult> {
        let state = self.state.read().await.clone();
        let components = self.components.read().await;

        let mut component_health = HashMap::new();
        let mut all_healthy = true;

        for (id, lifecycle) in components.iter() {
            let healthy = matches!(
                lifecycle.state,
                LifecycleState::Ready | LifecycleState::Active | LifecycleState::Paused
            );

            if !healthy {
                all_healthy = false;
            }

            component_health.insert(
                id.clone(),
                ComponentHealth {
                    healthy,
                    state: lifecycle.state.clone(),
                    error_count: lifecycle.error_count,
                    last_active: lifecycle.last_active,
                },
            );

            self.emit_event(LifecycleEvent::HealthCheck {
                component_id: id.clone(),
                healthy,
                details: HashMap::new(),
            })
            .await?;
        }

        Ok(HealthCheckResult {
            overall_health: all_healthy,
            manager_state: state,
            component_health,
            timestamp: chrono::Utc::now(),
        })
    }

    /// Update component activity
    pub async fn update_activity(&self, component_id: &str) -> Result<()> {
        if let Some(lifecycle) = self.components.write().await.get_mut(component_id) {
            lifecycle.last_active = Some(chrono::Utc::now());
        }
        Ok(())
    }

    /// Record component error
    pub async fn record_error(
        &self,
        component_id: &str,
        error: &str,
        severity: ErrorSeverity,
    ) -> Result<()> {
        if let Some(lifecycle) = self.components.write().await.get_mut(component_id) {
            lifecycle.error_count += 1;
        }

        self.emit_event(LifecycleEvent::Error {
            component_id: component_id.to_string(),
            error: error.to_string(),
            severity,
        })
        .await
    }
}

/// Health check result
#[derive(Debug)]
pub struct HealthCheckResult {
    /// Overall health status
    pub overall_health: bool,
    /// Manager state
    pub manager_state: LifecycleState,
    /// Component health statuses
    pub component_health: HashMap<String, ComponentHealth>,
    /// Timestamp of check
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// Component health information
#[derive(Debug)]
pub struct ComponentHealth {
    /// Whether component is healthy
    pub healthy: bool,
    /// Component state
    pub state: LifecycleState,
    /// Error count
    pub error_count: u32,
    /// Last activity
    pub last_active: Option<chrono::DateTime<chrono::Utc>>,
}

/// Hierarchical lifecycle manager for hierarchical agents
pub struct HierarchicalLifecycleManager {
    /// Base lifecycle manager
    base: CompositeLifecycleManager,
    /// Parent-child relationships
    hierarchy: RwLock<HashMap<String, Vec<String>>>,
}

impl HierarchicalLifecycleManager {
    /// Create a new hierarchical lifecycle manager
    #[must_use]
    pub fn new(config: LifecycleConfig) -> Self {
        Self {
            base: CompositeLifecycleManager::new(config),
            hierarchy: RwLock::new(HashMap::new()),
        }
    }

    /// Initialize a hierarchical agent
    pub async fn initialize_hierarchical(&self, agent: &dyn HierarchicalAgent) -> Result<()> {
        // Initialize base
        self.base.initialize_composite(agent).await?;

        // Build hierarchy
        let mut hierarchy = self.hierarchy.write().await;
        let agent_id = agent.metadata().id.to_string();

        let children: Vec<String> = agent
            .children()
            .into_iter()
            .map(|child| child.metadata().id.to_string())
            .collect();

        hierarchy.insert(agent_id, children);

        Ok(())
    }

    /// Cascade an event through the hierarchy
    pub async fn cascade_event(
        &self,
        from_id: &str,
        event: HierarchyEvent,
        direction: CascadeDirection,
    ) -> Result<()> {
        let hierarchy = self.hierarchy.read().await;

        match direction {
            CascadeDirection::Down => {
                if let Some(children) = hierarchy.get(from_id) {
                    for child_id in children {
                        // Process event for child
                        self.process_hierarchy_event(child_id, &event).await?;
                        // Recursively cascade down
                        Box::pin(self.cascade_event(child_id, event.clone(), direction)).await?;
                    }
                }
            }
            CascadeDirection::Up => {
                // Find parent
                for (parent_id, children) in hierarchy.iter() {
                    if children.contains(&from_id.to_string()) {
                        // Process event for parent
                        self.process_hierarchy_event(parent_id, &event).await?;
                        // Recursively cascade up
                        Box::pin(self.cascade_event(parent_id, event.clone(), direction)).await?;
                        break;
                    }
                }
            }
        }

        Ok(())
    }

    /// Process a hierarchy event for a component
    async fn process_hierarchy_event(
        &self,
        component_id: &str,
        event: &HierarchyEvent,
    ) -> Result<()> {
        match event {
            HierarchyEvent::StateChange {
                old_state,
                new_state,
            } => {
                self.base
                    .emit_event(LifecycleEvent::StateTransition {
                        component_id: component_id.to_string(),
                        from: self.parse_lifecycle_state(old_state),
                        to: self.parse_lifecycle_state(new_state),
                        timestamp: chrono::Utc::now(),
                    })
                    .await?;
            }
            HierarchyEvent::Error(error) => {
                self.base
                    .record_error(component_id, error, ErrorSeverity::Error)
                    .await?;
            }
            _ => {
                // Other events handled elsewhere
            }
        }
        Ok(())
    }

    /// Parse lifecycle state from string
    fn parse_lifecycle_state(&self, state: &str) -> LifecycleState {
        match state {
            "initializing" => LifecycleState::Initializing,
            "ready" => LifecycleState::Ready,
            "active" => LifecycleState::Active,
            "paused" => LifecycleState::Paused,
            "shutting_down" => LifecycleState::ShuttingDown,
            "terminated" => LifecycleState::Terminated,
            _ => LifecycleState::Ready,
        }
    }
}

/// Direction for cascading events
#[derive(Debug, Clone, Copy)]
pub enum CascadeDirection {
    /// Cascade down to children
    Down,
    /// Cascade up to parents
    Up,
}

#[cfg(test)]
mod tests {
    use super::*;
    use llmspell_core::{
        types::{AgentInput, AgentOutput},
        ComponentMetadata, ExecutionContext,
    };

    struct MockCompositeAgent {
        metadata: ComponentMetadata,
        components: Vec<Arc<dyn BaseAgent>>,
    }

    #[async_trait]
    impl BaseAgent for MockCompositeAgent {
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

    #[async_trait]
    impl CompositeAgent for MockCompositeAgent {
        async fn add_component(&mut self, component: Arc<dyn BaseAgent>) -> Result<()> {
            self.components.push(component);
            Ok(())
        }

        async fn remove_component(&mut self, _component_id: &str) -> Result<()> {
            Ok(())
        }

        fn components(&self) -> Vec<Arc<dyn BaseAgent>> {
            self.components.clone()
        }

        fn get_component(&self, _component_id: &str) -> Option<Arc<dyn BaseAgent>> {
            None
        }

        async fn delegate_to(
            &self,
            _component_id: &str,
            _input: serde_json::Value,
            _context: &ExecutionContext,
        ) -> Result<serde_json::Value> {
            Ok(serde_json::json!({}))
        }

        async fn execute_pattern(
            &self,
            _pattern: super::super::traits::ExecutionPattern,
            _input: serde_json::Value,
            _context: &ExecutionContext,
        ) -> Result<serde_json::Value> {
            Ok(serde_json::json!({}))
        }
    }

    #[async_trait]
    impl llmspell_core::ToolCapable for MockCompositeAgent {
        async fn discover_tools(
            &self,
            _query: &llmspell_core::traits::tool_capable::ToolQuery,
        ) -> Result<Vec<llmspell_core::traits::tool_capable::ToolInfo>> {
            Ok(Vec::new())
        }

        async fn invoke_tool(
            &self,
            _tool_name: &str,
            _parameters: serde_json::Value,
            _context: ExecutionContext,
        ) -> Result<AgentOutput> {
            Ok(AgentOutput::text("Mock tool invocation"))
        }
    }
    #[tokio::test]
    async fn test_lifecycle_state_transitions() {
        let manager = CompositeLifecycleManager::new(LifecycleConfig::default());

        // Initial state should be Initializing
        assert_eq!(manager.state().await, LifecycleState::Initializing);

        // Mock agent
        let agent = MockCompositeAgent {
            metadata: ComponentMetadata::new("test-agent".to_string(), "Test".to_string()),
            components: vec![],
        };

        // Initialize
        manager.initialize_composite(&agent).await.unwrap();
        assert_eq!(manager.state().await, LifecycleState::Ready);

        // Activate
        manager.activate().await.unwrap();
        assert_eq!(manager.state().await, LifecycleState::Active);

        // Pause
        manager.pause().await.unwrap();
        assert_eq!(manager.state().await, LifecycleState::Paused);

        // Resume
        manager.resume().await.unwrap();
        assert_eq!(manager.state().await, LifecycleState::Active);

        // Shutdown
        manager.shutdown().await.unwrap();
        assert_eq!(manager.state().await, LifecycleState::Terminated);
    }
    #[tokio::test]
    async fn test_invalid_state_transitions() {
        let manager = CompositeLifecycleManager::new(LifecycleConfig::default());

        // Try to activate before ready
        assert!(manager.activate().await.is_err());

        // Try to resume when not paused
        assert!(manager.resume().await.is_err());
    }
    #[tokio::test]
    async fn test_health_check() {
        let manager = CompositeLifecycleManager::new(LifecycleConfig::default());

        let agent = MockCompositeAgent {
            metadata: ComponentMetadata::new("test-agent".to_string(), "Test".to_string()),
            components: vec![],
        };

        manager.initialize_composite(&agent).await.unwrap();

        let health = manager.health_check().await.unwrap();
        assert!(health.overall_health);
        assert_eq!(health.manager_state, LifecycleState::Ready);
    }
}
