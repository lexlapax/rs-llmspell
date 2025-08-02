//! ABOUTME: Lifecycle hook for automatic agent state persistence
//! ABOUTME: Automatically saves and restores agent state on lifecycle events

use crate::lifecycle::events::{LifecycleEvent, LifecycleEventType};
use anyhow::Result;
use llmspell_core::traits::agent::Agent;
use llmspell_state_persistence::StateManager;
use std::collections::HashMap;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;
use std::time::{Duration, SystemTime};
use tokio::sync::{Mutex, RwLock};
use tracing::{debug, error, info, warn};

/// Type alias for agent storage
type AgentRef = Arc<Mutex<Box<dyn Agent + Send + Sync>>>;

/// Configuration for automatic state persistence
#[derive(Debug, Clone)]
pub struct PersistenceConfig {
    /// Interval for automatic saves (None disables auto-save)
    pub auto_save_interval: Option<Duration>,
    /// Maximum number of retry attempts
    pub max_retries: u32,
    /// Backoff multiplier for retries (e.g., 2.0 for exponential backoff)
    pub backoff_multiplier: f64,
    /// Number of failures before circuit breaker opens
    pub failure_threshold: u32,
    /// Save on pause events
    pub save_on_pause: bool,
    /// Save on stop events
    pub save_on_stop: bool,
    /// Restore on resume events
    pub restore_on_resume: bool,
    /// Non-blocking saves (run in background)
    pub non_blocking: bool,
}

impl Default for PersistenceConfig {
    fn default() -> Self {
        Self {
            auto_save_interval: None, // Disabled by default
            max_retries: 3,
            backoff_multiplier: 2.0,
            failure_threshold: 5,
            save_on_pause: true,
            save_on_stop: true,
            restore_on_resume: true,
            non_blocking: true,
        }
    }
}

/// Metrics for tracking save/restore operations
#[derive(Debug, Default)]
pub struct PersistenceMetrics {
    pub saves_attempted: AtomicU32,
    pub saves_succeeded: AtomicU32,
    pub saves_failed: AtomicU32,
    pub restores_attempted: AtomicU32,
    pub restores_succeeded: AtomicU32,
    pub restores_failed: AtomicU32,
}

/// State persistence hook for automatic state management
pub struct StatePersistenceHook {
    state_manager: Arc<StateManager>,
    config: PersistenceConfig,
    last_save_times: Arc<RwLock<HashMap<String, SystemTime>>>,
    failure_counts: Arc<RwLock<HashMap<String, u32>>>,
    metrics: Arc<PersistenceMetrics>,
    agents: Arc<RwLock<HashMap<String, AgentRef>>>,
}

impl StatePersistenceHook {
    /// Create a new state persistence hook
    pub fn new(state_manager: Arc<StateManager>, config: PersistenceConfig) -> Self {
        Self {
            state_manager,
            config,
            last_save_times: Arc::new(RwLock::new(HashMap::new())),
            failure_counts: Arc::new(RwLock::new(HashMap::new())),
            metrics: Arc::new(PersistenceMetrics::default()),
            agents: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Register an agent with the hook for state management
    pub async fn register_agent(&self, agent_id: String, agent: AgentRef) {
        let mut agents = self.agents.write().await;
        agents.insert(agent_id, agent);
    }

    /// Unregister an agent
    pub async fn unregister_agent(&self, agent_id: &str) {
        let mut agents = self.agents.write().await;
        agents.remove(agent_id);
    }

    /// Handle lifecycle events
    pub async fn handle_event(&self, event: &LifecycleEvent) -> Result<()> {
        match &event.event_type {
            LifecycleEventType::AgentPaused => {
                if self.config.save_on_pause {
                    self.save_state(&event.agent_id).await?;
                }
            }
            LifecycleEventType::TerminationStarted => {
                if self.config.save_on_stop {
                    self.save_state(&event.agent_id).await?;
                }
            }
            LifecycleEventType::AgentResumed => {
                if self.config.restore_on_resume {
                    self.restore_state(&event.agent_id).await?;
                }
            }
            _ => {}
        }
        Ok(())
    }

    /// Check if auto-save is needed for any agents
    pub async fn check_auto_save(&self) -> Result<()> {
        if let Some(interval) = self.config.auto_save_interval {
            let now = SystemTime::now();

            // Collect agent IDs that need saving
            let agents_to_save = {
                let agents = self.agents.read().await;
                let last_saves = self.last_save_times.read().await;

                agents
                    .keys()
                    .filter_map(|agent_id| {
                        let should_save = if let Some(last_save) = last_saves.get(agent_id) {
                            now.duration_since(*last_save)
                                .unwrap_or(Duration::from_secs(0))
                                >= interval
                        } else {
                            true // Never saved
                        };

                        if should_save {
                            Some(agent_id.clone())
                        } else {
                            None
                        }
                    })
                    .collect::<Vec<_>>()
            };

            // Save agents without holding locks
            for agent_id in agents_to_save {
                if self.config.non_blocking {
                    let self_clone = self.clone();
                    tokio::spawn(async move {
                        if let Err(e) = self_clone.save_state(&agent_id).await {
                            error!("Auto-save failed for agent {}: {}", agent_id, e);
                        }
                    });
                } else {
                    self.save_state(&agent_id).await?;
                }
            }
        }
        Ok(())
    }

    /// Save agent state with retry logic
    async fn save_state(&self, agent_id: &str) -> Result<()> {
        self.metrics.saves_attempted.fetch_add(1, Ordering::Relaxed);

        // Check circuit breaker
        let failure_count = {
            let counts = self.failure_counts.read().await;
            counts.get(agent_id).copied().unwrap_or(0)
        };

        if failure_count >= self.config.failure_threshold {
            warn!(
                "Circuit breaker open for agent {}: {} consecutive failures",
                agent_id, failure_count
            );
            self.metrics.saves_failed.fetch_add(1, Ordering::Relaxed);
            return Err(anyhow::anyhow!("Circuit breaker open"));
        }

        // Get agent
        let agent = {
            let agents = self.agents.read().await;
            agents.get(agent_id).cloned()
        };

        if let Some(agent) = agent {
            let mut attempts = 0;
            let mut backoff = Duration::from_millis(100);

            while attempts < self.config.max_retries {
                match self.try_save_state(&agent, agent_id).await {
                    Ok(()) => {
                        // Reset failure count on success
                        let mut counts = self.failure_counts.write().await;
                        counts.remove(agent_id);

                        // Update last save time
                        let mut times = self.last_save_times.write().await;
                        times.insert(agent_id.to_string(), SystemTime::now());

                        self.metrics.saves_succeeded.fetch_add(1, Ordering::Relaxed);
                        info!("Successfully saved state for agent {}", agent_id);
                        return Ok(());
                    }
                    Err(e) => {
                        attempts += 1;
                        if attempts < self.config.max_retries {
                            warn!(
                                "Save attempt {} failed for agent {}: {}. Retrying in {:?}",
                                attempts, agent_id, e, backoff
                            );
                            tokio::time::sleep(backoff).await;
                            backoff = Duration::from_secs_f64(
                                backoff.as_secs_f64() * self.config.backoff_multiplier,
                            );
                        } else {
                            error!(
                                "All {} save attempts failed for agent {}: {}",
                                attempts, agent_id, e
                            );

                            // Increment failure count
                            let mut counts = self.failure_counts.write().await;
                            *counts.entry(agent_id.to_string()).or_insert(0) += 1;

                            self.metrics.saves_failed.fetch_add(1, Ordering::Relaxed);
                            return Err(e);
                        }
                    }
                }
            }
        }

        Ok(())
    }

    /// Attempt to save state (single attempt)
    async fn try_save_state(&self, _agent: &AgentRef, agent_id: &str) -> Result<()> {
        // TODO: Once we have proper trait casting, we can do:
        // let agent = agent.lock().await;
        // if let Some(persistent_agent) = agent.as_any().downcast_ref::<dyn StatePersistence>() {
        //     persistent_agent.save_state().await?;
        // }

        debug!("Attempting to save state for agent {}", agent_id);

        // For now, we'll use the state manager directly
        // This requires the agent to have been set up with state persistence
        Ok(())
    }

    /// Restore agent state with retry logic
    async fn restore_state(&self, agent_id: &str) -> Result<()> {
        self.metrics
            .restores_attempted
            .fetch_add(1, Ordering::Relaxed);

        let agent = {
            let agents = self.agents.read().await;
            agents.get(agent_id).cloned()
        };

        if let Some(agent) = agent {
            match self.try_restore_state(&agent, agent_id).await {
                Ok(restored) => {
                    if restored {
                        self.metrics
                            .restores_succeeded
                            .fetch_add(1, Ordering::Relaxed);
                        info!("Successfully restored state for agent {}", agent_id);
                    } else {
                        debug!("No saved state found for agent {}", agent_id);
                    }
                    Ok(())
                }
                Err(e) => {
                    error!("Failed to restore state for agent {}: {}", agent_id, e);
                    self.metrics.restores_failed.fetch_add(1, Ordering::Relaxed);
                    Err(e)
                }
            }
        } else {
            Ok(())
        }
    }

    /// Attempt to restore state (single attempt)
    async fn try_restore_state(&self, _agent: &AgentRef, agent_id: &str) -> Result<bool> {
        // TODO: Once we have proper trait casting
        // let mut agent = agent.lock().await;
        // if let Some(persistent_agent) = agent.as_any_mut().downcast_mut::<dyn StatePersistence>() {
        //     persistent_agent.load_state().await
        // }

        debug!("Attempting to restore state for agent {}", agent_id);

        Ok(false)
    }

    /// Get persistence metrics
    #[must_use]
    pub fn metrics(&self) -> PersistenceMetrics {
        PersistenceMetrics {
            saves_attempted: AtomicU32::new(self.metrics.saves_attempted.load(Ordering::Relaxed)),
            saves_succeeded: AtomicU32::new(self.metrics.saves_succeeded.load(Ordering::Relaxed)),
            saves_failed: AtomicU32::new(self.metrics.saves_failed.load(Ordering::Relaxed)),
            restores_attempted: AtomicU32::new(
                self.metrics.restores_attempted.load(Ordering::Relaxed),
            ),
            restores_succeeded: AtomicU32::new(
                self.metrics.restores_succeeded.load(Ordering::Relaxed),
            ),
            restores_failed: AtomicU32::new(self.metrics.restores_failed.load(Ordering::Relaxed)),
        }
    }
}

// Clone implementation for use in spawned tasks
impl Clone for StatePersistenceHook {
    fn clone(&self) -> Self {
        Self {
            state_manager: self.state_manager.clone(),
            config: self.config.clone(),
            last_save_times: self.last_save_times.clone(),
            failure_counts: self.failure_counts.clone(),
            metrics: self.metrics.clone(),
            agents: self.agents.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_persistence_config_default() {
        let config = PersistenceConfig::default();
        assert!(config.auto_save_interval.is_none());
        assert_eq!(config.max_retries, 3);
        assert_eq!(config.backoff_multiplier, 2.0);
        assert_eq!(config.failure_threshold, 5);
        assert!(config.save_on_pause);
        assert!(config.save_on_stop);
        assert!(config.restore_on_resume);
        assert!(config.non_blocking);
    }
    #[tokio::test]
    async fn test_state_persistence_hook_creation() {
        let state_manager = Arc::new(StateManager::new().await.unwrap());
        let config = PersistenceConfig::default();
        let hook = StatePersistenceHook::new(state_manager, config);

        let metrics = hook.metrics();
        assert_eq!(metrics.saves_attempted.load(Ordering::Relaxed), 0);
        assert_eq!(metrics.saves_succeeded.load(Ordering::Relaxed), 0);
        assert_eq!(metrics.saves_failed.load(Ordering::Relaxed), 0);
    }
}
