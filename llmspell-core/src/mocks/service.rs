//! Mock implementation of service infrastructure traits

use crate::error::LLMSpellError;
#[cfg(test)]
use crate::traits::service::Schedule;
use crate::traits::service::{ScheduledTask, ServiceConfig, ServiceInfrastructure, ServiceStatus};
use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;

/// Mock service infrastructure for testing
#[derive(Debug)]
pub struct MockServiceInfrastructure {
    running: Arc<RwLock<bool>>,
    tasks: Arc<RwLock<HashMap<String, ScheduledTask>>>,
    start_time: Arc<RwLock<Option<Instant>>>,
    connections: Arc<RwLock<usize>>,
}

impl Default for MockServiceInfrastructure {
    fn default() -> Self {
        Self {
            running: Arc::new(RwLock::new(false)),
            tasks: Arc::new(RwLock::new(HashMap::new())),
            start_time: Arc::new(RwLock::new(None)),
            connections: Arc::new(RwLock::new(0)),
        }
    }
}

impl MockServiceInfrastructure {
    /// Create a new mock service infrastructure
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Simulate adding a connection
    pub async fn add_connection(&self) {
        let mut connections = self.connections.write().await;
        *connections += 1;
    }

    /// Simulate removing a connection
    pub async fn remove_connection(&self) {
        let mut connections = self.connections.write().await;
        if *connections > 0 {
            *connections -= 1;
        }
    }
}

#[async_trait]
impl ServiceInfrastructure for MockServiceInfrastructure {
    async fn start_service(&self, _config: ServiceConfig) -> Result<(), LLMSpellError> {
        let mut running = self.running.write().await;
        if *running {
            return Err(LLMSpellError::Validation {
                message: "Service already running".to_string(),
                field: Some("service_state".to_string()),
            });
        }
        *running = true;

        let mut start_time = self.start_time.write().await;
        *start_time = Some(Instant::now());

        Ok(())
    }

    async fn stop_service(&self) -> Result<(), LLMSpellError> {
        let mut running = self.running.write().await;
        if !*running {
            return Err(LLMSpellError::Validation {
                message: "Service not running".to_string(),
                field: Some("service_state".to_string()),
            });
        }
        *running = false;

        let mut start_time = self.start_time.write().await;
        *start_time = None;

        Ok(())
    }

    async fn schedule_task(&self, task: ScheduledTask) -> Result<(), LLMSpellError> {
        let mut tasks = self.tasks.write().await;
        if tasks.contains_key(&task.id) {
            return Err(LLMSpellError::Validation {
                message: format!("Task {} already exists", task.id),
                field: Some("task_id".to_string()),
            });
        }
        tasks.insert(task.id.clone(), task);
        Ok(())
    }

    async fn cancel_task(&self, task_id: &str) -> Result<(), LLMSpellError> {
        let mut tasks = self.tasks.write().await;
        if tasks.remove(task_id).is_none() {
            return Err(LLMSpellError::Resource {
                message: format!("Task {} not found", task_id),
                resource_type: Some("task".to_string()),
                source: None,
            });
        }
        Ok(())
    }

    async fn get_status(&self) -> Result<ServiceStatus, LLMSpellError> {
        let running = *self.running.read().await;
        let start_time = *self.start_time.read().await;
        let tasks = self.tasks.read().await;
        let connections = *self.connections.read().await;

        let uptime = if let Some(start) = start_time {
            start.elapsed()
        } else {
            Duration::from_secs(0)
        };

        Ok(ServiceStatus {
            running,
            uptime,
            active_connections: connections,
            scheduled_tasks: tasks.len(),
            memory_usage: 1024 * 1024 * 50, // Mock 50MB
        })
    }

    async fn list_tasks(&self) -> Result<Vec<ScheduledTask>, LLMSpellError> {
        let tasks = self.tasks.read().await;
        Ok(tasks.values().cloned().collect())
    }

    async fn execute_task_now(&self, task_id: &str) -> Result<(), LLMSpellError> {
        let tasks = self.tasks.read().await;
        if !tasks.contains_key(task_id) {
            return Err(LLMSpellError::Resource {
                message: format!("Task {} not found", task_id),
                resource_type: Some("task".to_string()),
                source: None,
            });
        }
        // Mock execution - just return success
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_service_lifecycle() {
        let service = MockServiceInfrastructure::new();

        // Start service
        let config = ServiceConfig::default();
        assert!(service.start_service(config.clone()).await.is_ok());

        // Check status
        let status = service.get_status().await.unwrap();
        assert!(status.running);

        // Can't start again
        assert!(service.start_service(config).await.is_err());

        // Stop service
        assert!(service.stop_service().await.is_ok());

        // Check status
        let status = service.get_status().await.unwrap();
        assert!(!status.running);

        // Can't stop again
        assert!(service.stop_service().await.is_err());
    }

    #[tokio::test]
    async fn test_task_management() {
        let service = MockServiceInfrastructure::new();

        let task = ScheduledTask {
            id: "test-task".to_string(),
            name: "Test Task".to_string(),
            schedule: Schedule::Interval(60),
            script: "print('hello')".to_string(),
            metadata: HashMap::new(),
            enabled: true,
        };

        // Schedule task
        assert!(service.schedule_task(task.clone()).await.is_ok());

        // Can't schedule same task again
        assert!(service.schedule_task(task).await.is_err());

        // List tasks
        let tasks = service.list_tasks().await.unwrap();
        assert_eq!(tasks.len(), 1);

        // Execute task
        assert!(service.execute_task_now("test-task").await.is_ok());

        // Execute non-existent task
        assert!(service.execute_task_now("non-existent").await.is_err());

        // Cancel task
        assert!(service.cancel_task("test-task").await.is_ok());

        // Can't cancel again
        assert!(service.cancel_task("test-task").await.is_err());

        // List tasks
        let tasks = service.list_tasks().await.unwrap();
        assert_eq!(tasks.len(), 0);
    }

    #[tokio::test]
    async fn test_connections() {
        let service = MockServiceInfrastructure::new();

        // Start service
        let config = ServiceConfig::default();
        service.start_service(config).await.unwrap();

        // Add connections
        service.add_connection().await;
        service.add_connection().await;

        let status = service.get_status().await.unwrap();
        assert_eq!(status.active_connections, 2);

        // Remove connection
        service.remove_connection().await;

        let status = service.get_status().await.unwrap();
        assert_eq!(status.active_connections, 1);
    }
}
