//! ABOUTME: Graceful shutdown mechanism for agents with resource cleanup and state preservation
//! ABOUTME: Provides coordinated shutdown across multiple agents with configurable timeouts and priorities

use super::events::{LifecycleEvent, LifecycleEventData, LifecycleEventSystem, LifecycleEventType};
use super::resources::ResourceManager;
use super::state_machine::{AgentState, AgentStateMachine};
use anyhow::{anyhow, Result};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{broadcast, Mutex, RwLock};
use tokio::time::timeout;
use tracing::{debug, error, info, warn};
use uuid::Uuid;

/// Shutdown priority levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, Default)]
pub enum ShutdownPriority {
    /// Critical agents that must shutdown first
    Critical = 0,
    /// High priority agents
    High = 1,
    /// Normal priority agents
    #[default]
    Normal = 2,
    /// Low priority agents
    Low = 3,
    /// Background agents that shutdown last
    Background = 4,
}

/// Shutdown request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShutdownRequest {
    /// Unique request ID
    pub id: String,
    /// Agent to shutdown
    pub agent_id: String,
    /// Shutdown priority
    pub priority: ShutdownPriority,
    /// Maximum time to wait for graceful shutdown
    pub timeout: Duration,
    /// Force shutdown if graceful fails
    pub force_if_timeout: bool,
    /// Reason for shutdown
    pub reason: Option<String>,
    /// Whether to preserve agent state
    pub preserve_state: bool,
    /// Metadata for the shutdown request
    pub metadata: HashMap<String, String>,
}

impl ShutdownRequest {
    #[must_use]
    pub fn new(agent_id: String) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            agent_id,
            priority: ShutdownPriority::default(),
            timeout: Duration::from_secs(30),
            force_if_timeout: true,
            reason: None,
            preserve_state: false,
            metadata: HashMap::new(),
        }
    }

    #[must_use]
    pub const fn with_priority(mut self, priority: ShutdownPriority) -> Self {
        self.priority = priority;
        self
    }

    #[must_use]
    pub const fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    #[must_use]
    pub fn with_reason(mut self, reason: String) -> Self {
        self.reason = Some(reason);
        self
    }

    #[must_use]
    pub const fn with_state_preservation(mut self, preserve: bool) -> Self {
        self.preserve_state = preserve;
        self
    }

    #[must_use]
    pub const fn force_shutdown(mut self, force: bool) -> Self {
        self.force_if_timeout = force;
        self
    }

    #[must_use]
    pub fn with_metadata(mut self, key: &str, value: &str) -> Self {
        self.metadata.insert(key.to_string(), value.to_string());
        self
    }
}

/// Shutdown result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShutdownResult {
    /// Request ID
    pub request_id: String,
    /// Agent ID
    pub agent_id: String,
    /// Whether shutdown was successful
    pub success: bool,
    /// Whether shutdown was forced
    pub forced: bool,
    /// Time taken for shutdown
    pub duration: Duration,
    /// Error message if shutdown failed
    pub error: Option<String>,
    /// Final agent state
    pub final_state: Option<AgentState>,
    /// Resources that were cleaned up
    pub resources_cleaned: u32,
}

/// Shutdown hook trait
#[async_trait]
pub trait ShutdownHook: Send + Sync {
    /// Called before shutdown begins
    async fn before_shutdown(&self, request: &ShutdownRequest) -> Result<()>;

    /// Called during shutdown to perform custom cleanup
    async fn on_shutdown(&self, request: &ShutdownRequest) -> Result<()>;

    /// Called after shutdown completes (success or failure)
    async fn after_shutdown(&self, result: &ShutdownResult) -> Result<()>;

    /// Get hook priority (lower numbers run first)
    fn priority(&self) -> u8 {
        50
    }
}

/// Shutdown coordinator manages graceful shutdown of agents
pub struct ShutdownCoordinator {
    /// Event system for notifications
    event_system: Arc<LifecycleEventSystem>,
    /// Resource manager for cleanup
    resource_manager: Arc<ResourceManager>,
    /// Registered shutdown hooks
    hooks: Arc<RwLock<Vec<Arc<dyn ShutdownHook>>>>,
    /// Active shutdown requests
    active_shutdowns: Arc<RwLock<HashMap<String, ShutdownRequest>>>,
    /// Shutdown history
    shutdown_history: Arc<Mutex<Vec<ShutdownResult>>>,
    /// Configuration
    config: ShutdownConfig,
    /// Emergency shutdown signal
    emergency_shutdown: broadcast::Sender<()>,
}

/// Shutdown configuration
#[derive(Debug, Clone)]
pub struct ShutdownConfig {
    /// Default shutdown timeout
    pub default_timeout: Duration,
    /// Maximum concurrent shutdowns
    pub max_concurrent_shutdowns: usize,
    /// Enable detailed logging
    pub enable_logging: bool,
    /// Emergency shutdown timeout
    pub emergency_timeout: Duration,
    /// Maximum shutdown history size
    pub max_history_size: usize,
}

impl Default for ShutdownConfig {
    fn default() -> Self {
        Self {
            default_timeout: Duration::from_secs(30),
            max_concurrent_shutdowns: 10,
            enable_logging: true,
            emergency_timeout: Duration::from_secs(5),
            max_history_size: 1000,
        }
    }
}

impl ShutdownCoordinator {
    /// Create new shutdown coordinator
    #[must_use]
    pub fn new(
        event_system: Arc<LifecycleEventSystem>,
        resource_manager: Arc<ResourceManager>,
        config: ShutdownConfig,
    ) -> Self {
        let (emergency_shutdown, _) = broadcast::channel(1);

        Self {
            event_system,
            resource_manager,
            hooks: Arc::new(RwLock::new(Vec::new())),
            active_shutdowns: Arc::new(RwLock::new(HashMap::new())),
            shutdown_history: Arc::new(Mutex::new(Vec::new())),
            config,
            emergency_shutdown,
        }
    }

    /// Add shutdown hook
    pub async fn add_hook(&self, hook: Arc<dyn ShutdownHook>) {
        let mut hooks = self.hooks.write().await;
        hooks.push(hook);

        // Sort hooks by priority
        hooks.sort_by_key(|h| h.priority());

        if self.config.enable_logging {
            debug!(
                "Added shutdown hook with priority {}",
                hooks.last().unwrap().priority()
            );
        }
    }

    /// Shutdown single agent
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Agent is already shutting down
    /// - Shutdown process fails
    /// - Timeout occurs and force shutdown is disabled
    pub async fn shutdown_agent(
        &self,
        request: ShutdownRequest,
        state_machine: Arc<AgentStateMachine>,
    ) -> Result<ShutdownResult> {
        let start_time = Instant::now();
        let request_id = request.id.clone();
        let agent_id = request.agent_id.clone();

        if self.config.enable_logging {
            info!(
                "Starting shutdown for agent {} (priority: {:?}, timeout: {:?})",
                agent_id, request.priority, request.timeout
            );
        }

        // Check if already shutting down
        {
            let active = self.active_shutdowns.read().await;
            if active.contains_key(&agent_id) {
                return Err(anyhow!("Agent {} is already shutting down", agent_id));
            }
        }

        // Register active shutdown
        {
            let mut active = self.active_shutdowns.write().await;
            active.insert(agent_id.clone(), request.clone());
        }

        // Emit shutdown started event
        let event = LifecycleEvent::new(
            LifecycleEventType::TerminationStarted,
            agent_id.clone(),
            LifecycleEventData::Generic {
                message: format!("Shutdown initiated with priority {:?}", request.priority),
                details: request.metadata.clone(),
            },
            "shutdown_coordinator".to_string(),
        );

        if let Err(e) = self.event_system.emit(event).await {
            warn!("Failed to emit shutdown started event: {}", e);
        }

        // Perform shutdown with timeout
        let shutdown_result = if let Ok(result) = timeout(
            request.timeout,
            self.perform_shutdown(&request, state_machine.clone()),
        )
        .await
        {
            result
        } else {
            warn!("Shutdown timeout for agent {}", agent_id);

            if request.force_if_timeout {
                warn!("Forcing shutdown for agent {}", agent_id);
                self.force_shutdown(&request, state_machine.clone()).await
            } else {
                Err(anyhow!("Shutdown timeout and force disabled"))
            }
        };

        // Clean up active shutdown
        {
            let mut active = self.active_shutdowns.write().await;
            active.remove(&agent_id);
        }

        // Create result
        let final_state = state_machine.current_state().await;
        let result = match shutdown_result {
            Ok(resources_cleaned) => ShutdownResult {
                request_id: request_id.clone(),
                agent_id: agent_id.clone(),
                success: true,
                forced: false,
                duration: start_time.elapsed(),
                error: None,
                final_state: Some(final_state),
                resources_cleaned,
            },
            Err(e) => ShutdownResult {
                request_id: request_id.clone(),
                agent_id: agent_id.clone(),
                success: false,
                forced: false,
                duration: start_time.elapsed(),
                error: Some(e.to_string()),
                final_state: Some(final_state),
                resources_cleaned: 0,
            },
        };

        // Execute after shutdown hooks
        let hooks = self.hooks.read().await;
        for hook in hooks.iter() {
            if let Err(e) = hook.after_shutdown(&result).await {
                warn!("Shutdown hook failed in after_shutdown: {}", e);
            }
        }

        // Record result in history
        {
            let mut history = self.shutdown_history.lock().await;
            history.push(result.clone());

            // Trim history if needed
            if history.len() > self.config.max_history_size {
                history.remove(0);
            }
        }

        // Emit shutdown completed event
        let event_type = if result.success {
            LifecycleEventType::TerminationCompleted
        } else {
            LifecycleEventType::ErrorOccurred
        };

        let event = LifecycleEvent::new(
            event_type,
            agent_id.clone(),
            LifecycleEventData::Generic {
                message: if result.success {
                    "Shutdown completed successfully".to_string()
                } else {
                    format!(
                        "Shutdown failed: {}",
                        result
                            .error
                            .as_ref()
                            .unwrap_or(&"Unknown error".to_string())
                    )
                },
                details: HashMap::from([
                    (
                        "duration_ms".to_string(),
                        result.duration.as_millis().to_string(),
                    ),
                    (
                        "resources_cleaned".to_string(),
                        result.resources_cleaned.to_string(),
                    ),
                    ("forced".to_string(), result.forced.to_string()),
                ]),
            },
            "shutdown_coordinator".to_string(),
        );

        if let Err(e) = self.event_system.emit(event).await {
            warn!("Failed to emit shutdown completed event: {}", e);
        }

        if self.config.enable_logging {
            if result.success {
                info!(
                    "Successfully shut down agent {} in {:?}",
                    agent_id, result.duration
                );
            } else {
                error!(
                    "Failed to shut down agent {} after {:?}: {}",
                    agent_id,
                    result.duration,
                    result
                        .error
                        .as_ref()
                        .unwrap_or(&"Unknown error".to_string())
                );
            }
        }

        Ok(result)
    }

    /// Perform graceful shutdown
    async fn perform_shutdown(
        &self,
        request: &ShutdownRequest,
        state_machine: Arc<AgentStateMachine>,
    ) -> Result<u32> {
        let agent_id = &request.agent_id;

        // Execute before shutdown hooks
        let hooks = self.hooks.read().await;
        for hook in hooks.iter() {
            if let Err(e) = hook.before_shutdown(request).await {
                return Err(anyhow!("Before shutdown hook failed: {}", e));
            }
        }

        // Check if agent is already terminated
        let current_state = state_machine.current_state().await;
        if current_state == AgentState::Terminated {
            return Ok(0); // Already shut down
        }

        // Transition to terminating state
        if current_state != AgentState::Terminating {
            state_machine
                .transition_to_with_reason(
                    AgentState::Terminating,
                    request
                        .reason
                        .clone()
                        .or_else(|| Some("Graceful shutdown".to_string())),
                )
                .await?;
        }

        // Execute shutdown hooks
        for hook in hooks.iter() {
            if let Err(e) = hook.on_shutdown(request).await {
                warn!("Shutdown hook failed: {}", e);
                // Continue with other hooks
            }
        }

        // Clean up resources
        let initial_count = self.resource_manager.get_allocation_count().await;
        self.resource_manager.deallocate_all(agent_id).await?;
        let final_count = self.resource_manager.get_allocation_count().await;
        let resources_cleaned = u32::try_from(initial_count - final_count).unwrap_or(0);

        // Complete termination
        state_machine
            .transition_to_with_reason(
                AgentState::Terminated,
                Some("Shutdown completed".to_string()),
            )
            .await?;

        Ok(resources_cleaned)
    }

    /// Force shutdown (emergency)
    async fn force_shutdown(
        &self,
        request: &ShutdownRequest,
        state_machine: Arc<AgentStateMachine>,
    ) -> Result<u32> {
        warn!("Force shutting down agent {}", request.agent_id);

        // Force state transition
        let _ = state_machine
            .transition_to_with_reason(AgentState::Terminated, Some("Force shutdown".to_string()))
            .await;

        // Force resource cleanup
        let initial_count = self.resource_manager.get_allocation_count().await;
        let _ = self
            .resource_manager
            .deallocate_all(&request.agent_id)
            .await;
        let final_count = self.resource_manager.get_allocation_count().await;

        Ok(u32::try_from(initial_count - final_count).unwrap_or(0))
    }

    /// Shutdown multiple agents by priority
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - State machine lookup fails
    /// - Any agent shutdown fails
    pub async fn shutdown_agents_by_priority(
        &self,
        requests: Vec<ShutdownRequest>,
        state_machines: HashMap<String, Arc<AgentStateMachine>>,
    ) -> Result<Vec<ShutdownResult>> {
        let mut requests = requests;
        // Sort by priority (Critical first, Background last)
        requests.sort_by_key(|r| r.priority);

        let mut results = Vec::new();
        let mut current_priority = None;
        let mut current_batch = Vec::new();

        // Group requests by priority
        for request in requests {
            if current_priority != Some(request.priority) {
                // Process previous batch
                if !current_batch.is_empty() {
                    let batch_results = self.shutdown_batch(current_batch, &state_machines).await;
                    results.extend(batch_results);
                    current_batch = Vec::new();
                }
                current_priority = Some(request.priority);
            }
            current_batch.push(request);
        }

        // Process final batch
        if !current_batch.is_empty() {
            let batch_results = self.shutdown_batch(current_batch, &state_machines).await;
            results.extend(batch_results);
        }

        Ok(results)
    }

    /// Shutdown a batch of agents concurrently
    async fn shutdown_batch(
        &self,
        requests: Vec<ShutdownRequest>,
        state_machines: &HashMap<String, Arc<AgentStateMachine>>,
    ) -> Vec<ShutdownResult> {
        let tasks: Vec<_> = requests
            .into_iter()
            .map(|request| {
                let agent_id = request.agent_id.clone();
                let state_machine = state_machines.get(&agent_id).cloned();
                let coordinator = self;

                async move {
                    if let Some(state_machine) = state_machine {
                        coordinator.shutdown_agent(request, state_machine).await
                    } else {
                        Err(anyhow!("State machine not found for agent {}", agent_id))
                    }
                }
            })
            .collect();

        // Wait for all tasks to complete
        let results = futures::future::join_all(tasks).await;

        results
            .into_iter()
            .map(|r| {
                r.unwrap_or_else(|e| ShutdownResult {
                    request_id: "unknown".to_string(),
                    agent_id: "unknown".to_string(),
                    success: false,
                    forced: false,
                    duration: Duration::from_secs(0),
                    error: Some(e.to_string()),
                    final_state: None,
                    resources_cleaned: 0,
                })
            })
            .collect()
    }

    /// Emergency shutdown all agents
    ///
    /// # Errors
    ///
    /// Returns an error if emergency broadcast fails
    pub async fn emergency_shutdown(&self) -> Result<()> {
        warn!("Emergency shutdown initiated");

        // Send emergency shutdown signal
        let _ = self.emergency_shutdown.send(());

        // Force immediate shutdown for all active agents
        let active_agents = {
            let active = self.active_shutdowns.read().await;
            active.keys().cloned().collect::<Vec<_>>()
        };

        for agent_id in active_agents {
            warn!("Emergency shutdown for agent {}", agent_id);
            let _ = self.resource_manager.deallocate_all(&agent_id).await;
        }

        info!("Emergency shutdown completed");
        Ok(())
    }

    /// Get emergency shutdown receiver
    #[must_use]
    pub fn subscribe_emergency_shutdown(&self) -> broadcast::Receiver<()> {
        self.emergency_shutdown.subscribe()
    }

    /// Get shutdown history
    pub async fn get_shutdown_history(&self) -> Vec<ShutdownResult> {
        let history = self.shutdown_history.lock().await;
        history.clone()
    }

    /// Get active shutdowns
    pub async fn get_active_shutdowns(&self) -> HashMap<String, ShutdownRequest> {
        let active = self.active_shutdowns.read().await;
        active.clone()
    }

    /// Check if agent is shutting down
    pub async fn is_shutting_down(&self, agent_id: &str) -> bool {
        let active = self.active_shutdowns.read().await;
        active.contains_key(agent_id)
    }
}

/// Default resource cleanup hook
pub struct ResourceCleanupHook {
    resource_manager: Arc<ResourceManager>,
}

impl ResourceCleanupHook {
    #[must_use]
    pub const fn new(resource_manager: Arc<ResourceManager>) -> Self {
        Self { resource_manager }
    }
}

#[async_trait]
impl ShutdownHook for ResourceCleanupHook {
    async fn before_shutdown(&self, _request: &ShutdownRequest) -> Result<()> {
        Ok(())
    }

    async fn on_shutdown(&self, request: &ShutdownRequest) -> Result<()> {
        debug!("Cleaning up resources for agent {}", request.agent_id);
        self.resource_manager
            .deallocate_all(&request.agent_id)
            .await?;
        Ok(())
    }

    async fn after_shutdown(&self, result: &ShutdownResult) -> Result<()> {
        if result.success {
            debug!("Resource cleanup completed for agent {}", result.agent_id);
        } else {
            warn!(
                "Resource cleanup may be incomplete for agent {}",
                result.agent_id
            );
        }
        Ok(())
    }

    fn priority(&self) -> u8 {
        10 // High priority for resource cleanup
    }
}

/// Logging shutdown hook
pub struct LoggingShutdownHook;

#[async_trait]
impl ShutdownHook for LoggingShutdownHook {
    async fn before_shutdown(&self, request: &ShutdownRequest) -> Result<()> {
        info!(
            "Beginning shutdown for agent {} (priority: {:?})",
            request.agent_id, request.priority
        );
        Ok(())
    }

    async fn on_shutdown(&self, request: &ShutdownRequest) -> Result<()> {
        debug!("Processing shutdown for agent {}", request.agent_id);
        Ok(())
    }

    async fn after_shutdown(&self, result: &ShutdownResult) -> Result<()> {
        if result.success {
            info!(
                "Agent {} shutdown completed in {:?}",
                result.agent_id, result.duration
            );
        } else {
            error!(
                "Agent {} shutdown failed after {:?}: {}",
                result.agent_id,
                result.duration,
                result
                    .error
                    .as_ref()
                    .unwrap_or(&"Unknown error".to_string())
            );
        }
        Ok(())
    }

    fn priority(&self) -> u8 {
        100 // Low priority for logging
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lifecycle::events::EventSystemConfig;
    use tokio::time::sleep;
    #[tokio::test]
    async fn test_shutdown_coordinator_basic() {
        let event_system = Arc::new(LifecycleEventSystem::new(EventSystemConfig::default()));
        let resource_manager = Arc::new(ResourceManager::new(
            crate::lifecycle::resources::ResourceLimits::default(),
            event_system.clone(),
        ));

        let coordinator = ShutdownCoordinator::new(
            event_system,
            resource_manager.clone(),
            ShutdownConfig::default(),
        );

        coordinator.add_hook(Arc::new(LoggingShutdownHook)).await;
        coordinator
            .add_hook(Arc::new(ResourceCleanupHook::new(resource_manager)))
            .await;

        let request = ShutdownRequest::new("test-agent".to_string())
            .with_priority(ShutdownPriority::Normal)
            .with_reason("Test shutdown".to_string());

        let state_machine = Arc::new(AgentStateMachine::default("test-agent".to_string()));
        state_machine.initialize().await.unwrap();

        let result = coordinator
            .shutdown_agent(request, state_machine.clone())
            .await
            .unwrap();

        assert!(result.success);
        assert_eq!(result.agent_id, "test-agent");
        assert_eq!(state_machine.current_state().await, AgentState::Terminated);
    }
    #[tokio::test]
    async fn test_shutdown_priorities() {
        let event_system = Arc::new(LifecycleEventSystem::new(EventSystemConfig::default()));
        let resource_manager = Arc::new(ResourceManager::new(
            crate::lifecycle::resources::ResourceLimits::default(),
            event_system.clone(),
        ));

        let coordinator =
            ShutdownCoordinator::new(event_system, resource_manager, ShutdownConfig::default());

        let requests = vec![
            ShutdownRequest::new("background-agent".to_string())
                .with_priority(ShutdownPriority::Background),
            ShutdownRequest::new("critical-agent".to_string())
                .with_priority(ShutdownPriority::Critical),
            ShutdownRequest::new("normal-agent".to_string())
                .with_priority(ShutdownPriority::Normal),
        ];

        let mut state_machines = HashMap::new();
        for request in &requests {
            let state_machine = Arc::new(AgentStateMachine::default(request.agent_id.clone()));
            state_machine.initialize().await.unwrap();
            state_machines.insert(request.agent_id.clone(), state_machine);
        }

        let results = coordinator
            .shutdown_agents_by_priority(requests, state_machines)
            .await
            .unwrap();

        assert_eq!(results.len(), 3);

        // Critical should be first, background should be last
        assert_eq!(results[0].agent_id, "critical-agent");
        assert_eq!(results[2].agent_id, "background-agent");
    }
    #[tokio::test]
    async fn test_shutdown_timeout() {
        let event_system = Arc::new(LifecycleEventSystem::new(EventSystemConfig::default()));
        let resource_manager = Arc::new(ResourceManager::new(
            crate::lifecycle::resources::ResourceLimits::default(),
            event_system.clone(),
        ));

        let coordinator =
            ShutdownCoordinator::new(event_system, resource_manager, ShutdownConfig::default());

        // Add a hook that takes too long
        struct SlowShutdownHook;

        #[async_trait]
        impl ShutdownHook for SlowShutdownHook {
            async fn before_shutdown(&self, _request: &ShutdownRequest) -> Result<()> {
                Ok(())
            }

            async fn on_shutdown(&self, _request: &ShutdownRequest) -> Result<()> {
                sleep(Duration::from_millis(100)).await; // Takes longer than timeout
                Ok(())
            }

            async fn after_shutdown(&self, _result: &ShutdownResult) -> Result<()> {
                Ok(())
            }
        }

        coordinator.add_hook(Arc::new(SlowShutdownHook)).await;

        let request = ShutdownRequest::new("test-agent".to_string())
            .with_timeout(Duration::from_millis(50)) // Short timeout
            .force_shutdown(true);

        let state_machine = Arc::new(AgentStateMachine::default("test-agent".to_string()));
        state_machine.initialize().await.unwrap();

        let result = coordinator
            .shutdown_agent(request, state_machine.clone())
            .await
            .unwrap();

        // Should succeed due to force shutdown
        assert!(result.success);
        assert_eq!(state_machine.current_state().await, AgentState::Terminated);
    }
    #[tokio::test]
    async fn test_emergency_shutdown() {
        let event_system = Arc::new(LifecycleEventSystem::new(EventSystemConfig::default()));
        let resource_manager = Arc::new(ResourceManager::new(
            crate::lifecycle::resources::ResourceLimits::default(),
            event_system.clone(),
        ));

        let coordinator =
            ShutdownCoordinator::new(event_system, resource_manager, ShutdownConfig::default());

        let mut emergency_receiver = coordinator.subscribe_emergency_shutdown();

        // Start emergency shutdown
        let emergency_task = tokio::spawn(async move { emergency_receiver.recv().await });

        coordinator.emergency_shutdown().await.unwrap();

        // Should receive emergency signal
        assert!(emergency_task.await.unwrap().is_ok());
    }
    #[tokio::test]
    async fn test_shutdown_request_builder() {
        let request = ShutdownRequest::new("test-agent".to_string())
            .with_priority(ShutdownPriority::High)
            .with_timeout(Duration::from_secs(60))
            .with_reason("Test reason".to_string())
            .with_state_preservation(true)
            .force_shutdown(false)
            .with_metadata("test", "value");

        assert_eq!(request.agent_id, "test-agent");
        assert_eq!(request.priority, ShutdownPriority::High);
        assert_eq!(request.timeout, Duration::from_secs(60));
        assert_eq!(request.reason, Some("Test reason".to_string()));
        assert!(request.preserve_state);
        assert!(!request.force_if_timeout);
        assert_eq!(request.metadata.get("test"), Some(&"value".to_string()));
    }
}
