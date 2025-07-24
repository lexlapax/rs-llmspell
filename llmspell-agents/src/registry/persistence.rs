//! ABOUTME: Persistence integration for agent registry
//! ABOUTME: Uses llmspell-storage for backend-agnostic persistence

use super::{AgentMetadata, AgentQuery, AgentRegistry, AgentStatus};
use anyhow::Result;
use async_trait::async_trait;
use llmspell_core::traits::agent::Agent;
use llmspell_storage::{StorageBackend, StorageSerialize};
use std::{collections::HashMap, sync::Arc};
use tokio::sync::RwLock;

/// Key prefix for agent metadata
const AGENT_METADATA_PREFIX: &str = "agent:metadata:";

/// Key for registry snapshot
const REGISTRY_SNAPSHOT_KEY: &str = "registry:snapshot";

/// Persistent agent registry using llmspell-storage backend
pub struct PersistentAgentRegistry {
    /// Storage backend from llmspell-storage
    storage: Arc<dyn StorageBackend>,

    /// Runtime agents (not persisted)
    runtime_agents: Arc<RwLock<HashMap<String, Arc<dyn Agent>>>>,

    /// Metadata cache for performance
    metadata_cache: Arc<RwLock<HashMap<String, AgentMetadata>>>,
}

impl PersistentAgentRegistry {
    /// Create new persistent registry with given storage backend
    pub async fn new(storage: Arc<dyn StorageBackend>) -> Result<Self> {
        // Load existing metadata from storage
        let metadata = Self::load_all_metadata(&storage).await?;

        Ok(Self {
            storage,
            runtime_agents: Arc::new(RwLock::new(HashMap::new())),
            metadata_cache: Arc::new(RwLock::new(metadata)),
        })
    }

    /// Load all agent metadata from storage
    async fn load_all_metadata(
        storage: &Arc<dyn StorageBackend>,
    ) -> Result<HashMap<String, AgentMetadata>> {
        let mut metadata = HashMap::new();

        // List all agent metadata keys
        let keys = storage.list_keys(AGENT_METADATA_PREFIX).await?;

        for key in keys {
            if let Some(agent_id) = key.strip_prefix(AGENT_METADATA_PREFIX) {
                if let Some(data) = storage.get(&key).await? {
                    if let Ok(agent_metadata) = AgentMetadata::from_storage_bytes(&data) {
                        metadata.insert(agent_id.to_string(), agent_metadata);
                    }
                }
            }
        }

        Ok(metadata)
    }

    /// Get metadata key for agent
    fn metadata_key(agent_id: &str) -> String {
        format!("{}{}", AGENT_METADATA_PREFIX, agent_id)
    }

    /// Persist current state
    pub async fn persist(&self) -> Result<()> {
        let cache = self.metadata_cache.read().await;

        // Save individual metadata entries
        for (id, metadata) in cache.iter() {
            let key = Self::metadata_key(id);
            let data = metadata.to_storage_bytes()?;
            self.storage.set(&key, data).await?;
        }

        // Also save a snapshot for faster loading
        let snapshot_data = cache.to_storage_bytes()?;
        self.storage
            .set(REGISTRY_SNAPSHOT_KEY, snapshot_data)
            .await?;

        Ok(())
    }

    /// Load from snapshot (faster than loading individual entries)
    pub async fn load_from_snapshot(&mut self) -> Result<()> {
        if let Some(data) = self.storage.get(REGISTRY_SNAPSHOT_KEY).await? {
            if let Ok(metadata) = HashMap::<String, AgentMetadata>::from_storage_bytes(&data) {
                let mut cache = self.metadata_cache.write().await;
                *cache = metadata;
            }
        }
        Ok(())
    }
}

#[async_trait]
impl AgentRegistry for PersistentAgentRegistry {
    async fn register_agent(
        &self,
        id: String,
        agent: Arc<dyn Agent>,
        metadata: AgentMetadata,
    ) -> Result<()> {
        // Save to storage
        let key = Self::metadata_key(&id);
        let data = metadata.to_storage_bytes()?;
        self.storage.set(&key, data).await?;

        // Update runtime storage
        let mut agents = self.runtime_agents.write().await;
        let mut cache = self.metadata_cache.write().await;

        agents.insert(id.clone(), agent);
        cache.insert(id, metadata);

        Ok(())
    }

    async fn unregister_agent(&self, id: &str) -> Result<()> {
        // Remove from storage
        let key = Self::metadata_key(id);
        self.storage.delete(&key).await?;

        // Update runtime storage
        let mut agents = self.runtime_agents.write().await;
        let mut cache = self.metadata_cache.write().await;

        agents.remove(id);
        cache.remove(id);

        Ok(())
    }

    async fn get_agent(&self, id: &str) -> Result<Option<Arc<dyn Agent>>> {
        let agents = self.runtime_agents.read().await;
        Ok(agents.get(id).cloned())
    }

    async fn get_metadata(&self, id: &str) -> Result<Option<AgentMetadata>> {
        let cache = self.metadata_cache.read().await;

        if let Some(metadata) = cache.get(id) {
            return Ok(Some(metadata.clone()));
        }

        // Try loading from storage if not in cache
        let key = Self::metadata_key(id);
        if let Some(data) = self.storage.get(&key).await? {
            if let Ok(metadata) = AgentMetadata::from_storage_bytes(&data) {
                return Ok(Some(metadata));
            }
        }

        Ok(None)
    }

    async fn update_metadata(&self, id: &str, metadata: AgentMetadata) -> Result<()> {
        // Save to storage
        let key = Self::metadata_key(id);
        let data = metadata.to_storage_bytes()?;
        self.storage.set(&key, data).await?;

        // Update cache
        let mut cache = self.metadata_cache.write().await;
        cache.insert(id.to_string(), metadata);

        Ok(())
    }

    async fn update_status(&self, id: &str, status: AgentStatus) -> Result<()> {
        let mut cache = self.metadata_cache.write().await;

        if let Some(metadata) = cache.get_mut(id) {
            metadata.status = status;
            metadata.updated_at = chrono::Utc::now();

            // Save to storage
            let key = Self::metadata_key(id);
            let data = metadata.to_storage_bytes()?;
            self.storage.set(&key, data).await?;
        } else {
            anyhow::bail!("Agent '{}' not found", id);
        }

        Ok(())
    }

    async fn query_agents(&self, query: &AgentQuery) -> Result<Vec<AgentMetadata>> {
        let cache = self.metadata_cache.read().await;
        let mut results = Vec::new();

        for metadata in cache.values() {
            // Apply filters
            if let Some(name_filter) = &query.name_filter {
                if !metadata.name.contains(name_filter) {
                    continue;
                }
            }

            if let Some(type_filter) = &query.type_filter {
                if &metadata.agent_type != type_filter {
                    continue;
                }
            }

            if let Some(status_filter) = &query.status_filter {
                if &metadata.status != status_filter {
                    continue;
                }
            }

            if !query.category_filter.is_empty() {
                let has_category = query
                    .category_filter
                    .iter()
                    .any(|cat| metadata.categories.contains(cat));
                if !has_category {
                    continue;
                }
            }

            results.push(metadata.clone());
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
        let cache = self.metadata_cache.read().await;
        Ok(cache.keys().cloned().collect())
    }

    async fn count_agents(&self) -> Result<usize> {
        let cache = self.metadata_cache.read().await;
        Ok(cache.len())
    }

    async fn update_metrics(&self, id: &str, metrics: super::AgentMetrics) -> Result<()> {
        let mut cache = self.metadata_cache.write().await;

        if let Some(metadata) = cache.get_mut(id) {
            metadata.metrics = metrics;
            metadata.updated_at = chrono::Utc::now();

            // Save to storage
            let key = Self::metadata_key(id);
            let data = metadata.to_storage_bytes()?;
            self.storage.set(&key, data).await?;
        } else {
            anyhow::bail!("Agent '{}' not found", id);
        }

        Ok(())
    }

    async fn heartbeat(&self, id: &str) -> Result<()> {
        let cache = self.metadata_cache.read().await;

        if cache.contains_key(id) {
            Ok(())
        } else {
            anyhow::bail!("Agent '{}' not found", id);
        }
    }

    async fn exists(&self, id: &str) -> Result<bool> {
        let cache = self.metadata_cache.read().await;

        if cache.contains_key(id) {
            return Ok(true);
        }

        // Check storage
        let key = Self::metadata_key(id);
        self.storage.exists(&key).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use llmspell_storage::MemoryBackend;

    #[tokio::test]
    async fn test_persistent_registry_basic_operations() {
        let storage = Arc::new(MemoryBackend::new());
        let registry = PersistentAgentRegistry::new(storage).await.unwrap();

        // Create test metadata
        let metadata = AgentMetadata {
            id: "test-agent".to_string(),
            name: "Test Agent".to_string(),
            agent_type: "test".to_string(),
            description: "Test agent".to_string(),
            categories: vec!["test".to_string()],
            custom_metadata: HashMap::new(),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            status: AgentStatus::Active,
            metrics: crate::registry::AgentMetrics::default(),
        };

        // Create a mock agent
        struct MockAgent;

        #[async_trait]
        impl llmspell_core::traits::base_agent::BaseAgent for MockAgent {
            fn metadata(&self) -> &llmspell_core::ComponentMetadata {
                panic!("Not implemented for test")
            }

            async fn validate_input(
                &self,
                _input: &llmspell_core::types::AgentInput,
            ) -> llmspell_core::Result<()> {
                Ok(())
            }

            async fn execute(
                &self,
                _input: llmspell_core::types::AgentInput,
                _context: llmspell_core::ExecutionContext,
            ) -> llmspell_core::Result<llmspell_core::types::AgentOutput> {
                panic!("Not implemented for test")
            }

            async fn handle_error(
                &self,
                _error: llmspell_core::LLMSpellError,
            ) -> llmspell_core::Result<llmspell_core::types::AgentOutput> {
                panic!("Not implemented for test")
            }
        }

        #[async_trait]
        impl llmspell_core::traits::agent::Agent for MockAgent {
            fn config(&self) -> &llmspell_core::traits::agent::AgentConfig {
                panic!("Not implemented for test")
            }

            async fn get_conversation(
                &self,
            ) -> llmspell_core::Result<Vec<llmspell_core::traits::agent::ConversationMessage>>
            {
                panic!("Not implemented for test")
            }

            async fn add_message(
                &mut self,
                _message: llmspell_core::traits::agent::ConversationMessage,
            ) -> llmspell_core::Result<()> {
                panic!("Not implemented for test")
            }

            async fn clear_conversation(&mut self) -> llmspell_core::Result<()> {
                panic!("Not implemented for test")
            }
        }

        let agent = Arc::new(MockAgent);

        // Test registration
        registry
            .register_agent("test-agent".to_string(), agent, metadata.clone())
            .await
            .unwrap();

        // Test get metadata
        let retrieved = registry.get_metadata("test-agent").await.unwrap();
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().name, "Test Agent");

        // Test exists
        assert!(registry.exists("test-agent").await.unwrap());

        // Test query
        let query = AgentQuery {
            name_filter: Some("Test".to_string()),
            ..Default::default()
        };
        let results = registry.query_agents(&query).await.unwrap();
        assert_eq!(results.len(), 1);

        // Test update status
        registry
            .update_status("test-agent", AgentStatus::Paused)
            .await
            .unwrap();
        let updated = registry.get_metadata("test-agent").await.unwrap().unwrap();
        assert_eq!(updated.status, AgentStatus::Paused);

        // Test unregister
        registry.unregister_agent("test-agent").await.unwrap();
        assert!(!registry.exists("test-agent").await.unwrap());
    }

    #[tokio::test]
    async fn test_persistent_registry_persistence() {
        let storage = Arc::new(MemoryBackend::new());

        // Create and populate registry
        {
            let registry = PersistentAgentRegistry::new(storage.clone()).await.unwrap();

            let metadata = AgentMetadata {
                id: "persistent-agent".to_string(),
                name: "Persistent Agent".to_string(),
                agent_type: "test".to_string(),
                description: "Test persistence".to_string(),
                categories: vec![],
                custom_metadata: HashMap::new(),
                created_at: chrono::Utc::now(),
                updated_at: chrono::Utc::now(),
                status: AgentStatus::Active,
                metrics: crate::registry::AgentMetrics::default(),
            };

            struct MockAgent;
            #[async_trait]
            impl llmspell_core::traits::base_agent::BaseAgent for MockAgent {
                fn metadata(&self) -> &llmspell_core::ComponentMetadata {
                    panic!("Not implemented for test")
                }

                async fn validate_input(
                    &self,
                    _input: &llmspell_core::types::AgentInput,
                ) -> llmspell_core::Result<()> {
                    Ok(())
                }

                async fn execute(
                    &self,
                    _input: llmspell_core::types::AgentInput,
                    _context: llmspell_core::ExecutionContext,
                ) -> llmspell_core::Result<llmspell_core::types::AgentOutput> {
                    panic!("Not implemented for test")
                }

                async fn handle_error(
                    &self,
                    _error: llmspell_core::LLMSpellError,
                ) -> llmspell_core::Result<llmspell_core::types::AgentOutput> {
                    panic!("Not implemented for test")
                }
            }

            #[async_trait]
            impl llmspell_core::traits::agent::Agent for MockAgent {
                fn config(&self) -> &llmspell_core::traits::agent::AgentConfig {
                    panic!("Not implemented for test")
                }

                async fn get_conversation(
                    &self,
                ) -> llmspell_core::Result<Vec<llmspell_core::traits::agent::ConversationMessage>>
                {
                    panic!("Not implemented for test")
                }

                async fn add_message(
                    &mut self,
                    _message: llmspell_core::traits::agent::ConversationMessage,
                ) -> llmspell_core::Result<()> {
                    panic!("Not implemented for test")
                }

                async fn clear_conversation(&mut self) -> llmspell_core::Result<()> {
                    panic!("Not implemented for test")
                }
            }

            registry
                .register_agent(
                    "persistent-agent".to_string(),
                    Arc::new(MockAgent),
                    metadata,
                )
                .await
                .unwrap();

            // Persist state
            registry.persist().await.unwrap();
        }

        // Create new registry with same storage
        {
            let registry = PersistentAgentRegistry::new(storage).await.unwrap();

            // Verify data persisted
            assert!(registry.exists("persistent-agent").await.unwrap());
            let metadata = registry
                .get_metadata("persistent-agent")
                .await
                .unwrap()
                .unwrap();
            assert_eq!(metadata.name, "Persistent Agent");
        }
    }
}
