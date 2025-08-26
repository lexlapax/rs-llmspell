//! ABOUTME: Core registry types and traits
//! ABOUTME: Defines the foundational types for agent registry

#![allow(clippy::significant_drop_tightening)]

use anyhow::Result;
use async_trait::async_trait;
use llmspell_core::traits::agent::Agent;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, sync::Arc};

/// Agent metadata for registry tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentMetadata {
    /// Unique agent ID
    pub id: String,

    /// Agent name
    pub name: String,

    /// Agent type
    pub agent_type: String,

    /// Description
    pub description: String,

    /// Categories/tags
    pub categories: Vec<String>,

    /// Custom metadata
    pub custom_metadata: HashMap<String, serde_json::Value>,

    /// Creation timestamp
    pub created_at: chrono::DateTime<chrono::Utc>,

    /// Last updated timestamp
    pub updated_at: chrono::DateTime<chrono::Utc>,

    /// Agent status
    pub status: AgentStatus,

    /// Performance metrics
    pub metrics: AgentMetrics,
}

/// Agent status in the registry
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum AgentStatus {
    /// Agent is active and ready
    Active,

    /// Agent is paused
    Paused,

    /// Agent is stopped
    Stopped,

    /// Agent has encountered an error
    Error(String),

    /// Agent is initializing
    Initializing,
}

/// Agent performance metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentMetrics {
    /// Total execution count
    pub execution_count: u64,

    /// Success rate (0.0 - 1.0)
    pub success_rate: f64,

    /// Average execution time in milliseconds
    pub avg_execution_time_ms: f64,

    /// Last execution time
    pub last_execution_time: Option<chrono::DateTime<chrono::Utc>>,

    /// Error count
    pub error_count: u64,

    /// Last error message
    pub last_error: Option<String>,
}

impl Default for AgentMetrics {
    fn default() -> Self {
        Self {
            execution_count: 0,
            success_rate: 1.0,
            avg_execution_time_ms: 0.0,
            last_execution_time: None,
            error_count: 0,
            last_error: None,
        }
    }
}

/// Query parameters for agent discovery
#[derive(Debug, Clone, Default)]
pub struct AgentQuery {
    /// Filter by name (substring match)
    pub name_filter: Option<String>,

    /// Filter by type
    pub type_filter: Option<String>,

    /// Filter by status
    pub status_filter: Option<AgentStatus>,

    /// Filter by categories
    pub category_filter: Vec<String>,

    /// Pagination offset
    pub offset: Option<usize>,

    /// Pagination limit
    pub limit: Option<usize>,
}

/// Agent registry trait
#[async_trait]
pub trait AgentRegistry: Send + Sync {
    /// Register a new agent
    async fn register_agent(
        &self,
        id: String,
        agent: Arc<dyn Agent>,
        metadata: AgentMetadata,
    ) -> Result<()>;

    /// Unregister an agent
    async fn unregister_agent(&self, id: &str) -> Result<()>;

    /// Get agent by ID
    async fn get_agent(&self, id: &str) -> Result<Option<Arc<dyn Agent>>>;

    /// Get agent metadata
    async fn get_metadata(&self, id: &str) -> Result<Option<AgentMetadata>>;

    /// Update agent metadata
    async fn update_metadata(&self, id: &str, metadata: AgentMetadata) -> Result<()>;

    /// Update agent status
    async fn update_status(&self, id: &str, status: AgentStatus) -> Result<()>;

    /// Query agents with filters
    async fn query_agents(&self, query: &AgentQuery) -> Result<Vec<AgentMetadata>>;

    /// List all agent IDs
    async fn list_agent_ids(&self) -> Result<Vec<String>>;

    /// Get total agent count
    async fn count_agents(&self) -> Result<usize>;

    /// Update agent metrics
    async fn update_metrics(&self, id: &str, metrics: AgentMetrics) -> Result<()>;

    /// Agent heartbeat
    async fn heartbeat(&self, id: &str) -> Result<()>;

    /// Check if agent exists
    async fn exists(&self, id: &str) -> Result<bool>;
}

/// In-memory agent registry implementation
pub struct InMemoryAgentRegistry {
    agents: Arc<tokio::sync::RwLock<HashMap<String, Arc<dyn Agent>>>>,
    metadata: Arc<tokio::sync::RwLock<HashMap<String, AgentMetadata>>>,
    heartbeats: Arc<tokio::sync::RwLock<HashMap<String, std::time::Instant>>>,
}

impl InMemoryAgentRegistry {
    /// Create new in-memory registry
    #[must_use]
    pub fn new() -> Self {
        Self {
            agents: Arc::new(tokio::sync::RwLock::new(HashMap::new())),
            metadata: Arc::new(tokio::sync::RwLock::new(HashMap::new())),
            heartbeats: Arc::new(tokio::sync::RwLock::new(HashMap::new())),
        }
    }
}

impl Default for InMemoryAgentRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl AgentRegistry for InMemoryAgentRegistry {
    async fn register_agent(
        &self,
        id: String,
        agent: Arc<dyn Agent>,
        metadata: AgentMetadata,
    ) -> Result<()> {
        let mut agents = self.agents.write().await;
        let mut metadata_store = self.metadata.write().await;

        agents.insert(id.clone(), agent);
        metadata_store.insert(id, metadata);

        Ok(())
    }

    async fn unregister_agent(&self, id: &str) -> Result<()> {
        let mut agents = self.agents.write().await;
        let mut metadata_store = self.metadata.write().await;
        let mut heartbeats = self.heartbeats.write().await;

        agents.remove(id);
        metadata_store.remove(id);
        heartbeats.remove(id);

        Ok(())
    }

    async fn get_agent(&self, id: &str) -> Result<Option<Arc<dyn Agent>>> {
        let agents = self.agents.read().await;
        Ok(agents.get(id).cloned())
    }

    async fn get_metadata(&self, id: &str) -> Result<Option<AgentMetadata>> {
        let metadata = self.metadata.read().await;
        Ok(metadata.get(id).cloned())
    }

    async fn update_metadata(&self, id: &str, metadata: AgentMetadata) -> Result<()> {
        let mut metadata_store = self.metadata.write().await;
        metadata_store.insert(id.to_string(), metadata);
        Ok(())
    }

    async fn update_status(&self, id: &str, status: AgentStatus) -> Result<()> {
        let mut metadata = self.metadata.write().await;

        if let Some(meta) = metadata.get_mut(id) {
            meta.status = status;
            meta.updated_at = chrono::Utc::now();
        } else {
            anyhow::bail!("Agent '{}' not found", id);
        }

        Ok(())
    }

    async fn query_agents(&self, query: &AgentQuery) -> Result<Vec<AgentMetadata>> {
        let metadata = self.metadata.read().await;
        let mut results = Vec::new();

        for meta in metadata.values() {
            // Apply filters
            if let Some(name_filter) = &query.name_filter {
                if !meta.name.contains(name_filter) {
                    continue;
                }
            }

            if let Some(type_filter) = &query.type_filter {
                if &meta.agent_type != type_filter {
                    continue;
                }
            }

            if let Some(status_filter) = &query.status_filter {
                if &meta.status != status_filter {
                    continue;
                }
            }

            if !query.category_filter.is_empty() {
                let has_category = query
                    .category_filter
                    .iter()
                    .any(|cat| meta.categories.contains(cat));
                if !has_category {
                    continue;
                }
            }

            results.push(meta.clone());
        }

        // Apply pagination
        if let Some(offset) = query.offset {
            results = results.into_iter().skip(offset).collect();
        }

        if let Some(limit) = query.limit {
            results.truncate(limit);
        }

        Ok(results)
    }

    async fn list_agent_ids(&self) -> Result<Vec<String>> {
        let metadata = self.metadata.read().await;
        Ok(metadata.keys().cloned().collect())
    }

    async fn count_agents(&self) -> Result<usize> {
        let metadata = self.metadata.read().await;
        Ok(metadata.len())
    }

    async fn update_metrics(&self, id: &str, metrics: AgentMetrics) -> Result<()> {
        let mut metadata = self.metadata.write().await;

        if let Some(meta) = metadata.get_mut(id) {
            meta.metrics = metrics;
            meta.updated_at = chrono::Utc::now();
        } else {
            anyhow::bail!("Agent '{}' not found", id);
        }

        Ok(())
    }

    async fn heartbeat(&self, id: &str) -> Result<()> {
        let mut heartbeats = self.heartbeats.write().await;

        if self.metadata.read().await.contains_key(id) {
            heartbeats.insert(id.to_string(), std::time::Instant::now());
            Ok(())
        } else {
            anyhow::bail!("Agent '{}' not found", id);
        }
    }

    async fn exists(&self, id: &str) -> Result<bool> {
        let metadata = self.metadata.read().await;
        Ok(metadata.contains_key(id))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_agent_status_equality() {
        assert_eq!(AgentStatus::Active, AgentStatus::Active);
        assert_ne!(AgentStatus::Active, AgentStatus::Paused);
    }
    #[test]
    #[allow(clippy::float_cmp)] // Test assertion on float values
    fn test_agent_metrics_default() {
        let metrics = AgentMetrics::default();
        assert_eq!(metrics.execution_count, 0);
        assert_eq!(metrics.success_rate, 1.0);
    }
}
