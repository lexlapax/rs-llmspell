//! Service infrastructure traits for Phase 12: Daemon and Service Mode
//!
//! Provides traits and types for implementing long-running daemon mode with scheduler,
//! API endpoints, and service integration (systemd/launchd).

use crate::error::LLMSpellError;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt::Debug;

/// Service mode configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceConfig {
    /// Port to listen on
    pub port: u16,
    /// Bind address
    pub bind_address: String,
    /// Enable scheduling
    pub enable_scheduler: bool,
    /// API authentication token
    pub api_token: Option<String>,
    /// Maximum concurrent requests
    pub max_concurrent_requests: usize,
}

impl Default for ServiceConfig {
    fn default() -> Self {
        Self {
            port: 9555,
            bind_address: "127.0.0.1".to_string(),
            enable_scheduler: true,
            api_token: None,
            max_concurrent_requests: 100,
        }
    }
}

/// Scheduled task definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScheduledTask {
    /// Unique task identifier
    pub id: String,
    /// Task name
    pub name: String,
    /// Cron expression or interval
    pub schedule: Schedule,
    /// Script to execute
    pub script: String,
    /// Task metadata
    pub metadata: HashMap<String, serde_json::Value>,
    /// Whether the task is enabled
    pub enabled: bool,
}

/// Schedule type for tasks
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Schedule {
    /// Cron expression (e.g., "0 0 * * *")
    Cron(String),
    /// Fixed interval in seconds
    Interval(u64),
    /// One-time execution at specific time
    Once(chrono::DateTime<chrono::Utc>),
}

/// Service status information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceStatus {
    /// Whether service is running
    pub running: bool,
    /// Service uptime
    pub uptime: std::time::Duration,
    /// Number of active connections
    pub active_connections: usize,
    /// Scheduled tasks count
    pub scheduled_tasks: usize,
    /// Memory usage in bytes
    pub memory_usage: usize,
}

/// Service infrastructure trait for Phase 12 Daemon Mode
///
/// This trait defines the interface for implementing daemon mode with scheduler support.
/// Implementations should integrate with Phase 4's FlowController and CircuitBreaker
/// for stability, preventing memory exhaustion and runaway operations in long-running services.
#[async_trait]
pub trait ServiceInfrastructure: Send + Sync + Debug {
    /// Start the service with given configuration
    async fn start_service(&self, config: ServiceConfig) -> Result<(), LLMSpellError>;

    /// Stop the service gracefully
    async fn stop_service(&self) -> Result<(), LLMSpellError>;

    /// Schedule a task
    async fn schedule_task(&self, task: ScheduledTask) -> Result<(), LLMSpellError>;

    /// Cancel a scheduled task
    async fn cancel_task(&self, task_id: &str) -> Result<(), LLMSpellError>;

    /// Get service status
    async fn get_status(&self) -> Result<ServiceStatus, LLMSpellError>;

    /// List all scheduled tasks
    async fn list_tasks(&self) -> Result<Vec<ScheduledTask>, LLMSpellError>;

    /// Execute a task immediately (bypass schedule)
    async fn execute_task_now(&self, task_id: &str) -> Result<(), LLMSpellError>;
}
