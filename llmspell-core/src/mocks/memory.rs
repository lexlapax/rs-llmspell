//! Mock implementation of memory integration traits

use crate::error::LLMSpellError;
use crate::traits::memory::{
    ConsolidationResult, ContextQuery, InteractionLog, MemoryIntegration, MemoryItem, MemoryStats,
    MemoryType,
};
use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Mock memory integration for testing
#[derive(Debug, Default)]
pub struct MockMemoryIntegration {
    memories: Arc<RwLock<Vec<MemoryItem>>>,
    interactions: Arc<RwLock<Vec<InteractionLog>>>,
}

impl MockMemoryIntegration {
    /// Create a new mock memory integration
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a test memory item
    pub async fn add_test_memory(&self, content: String, memory_type: MemoryType) {
        let mut memories = self.memories.write().await;
        memories.push(MemoryItem {
            id: uuid::Uuid::new_v4().to_string(),
            content,
            memory_type,
            relevance: 0.8,
            created_at: chrono::Utc::now(),
            last_accessed: chrono::Utc::now(),
            metadata: HashMap::new(),
        });
    }

    /// Get the count of stored interactions
    pub async fn interaction_count(&self) -> usize {
        self.interactions.read().await.len()
    }
}

#[async_trait]
impl MemoryIntegration for MockMemoryIntegration {
    async fn store_interaction(&self, interaction: InteractionLog) -> Result<(), LLMSpellError> {
        let mut interactions = self.interactions.write().await;
        interactions.push(interaction);
        Ok(())
    }

    async fn query_context(&self, query: ContextQuery) -> Result<Vec<MemoryItem>, LLMSpellError> {
        let memories = self.memories.read().await;

        // Simple mock implementation: filter by memory type and return up to max_results
        let filtered: Vec<MemoryItem> = memories
            .iter()
            .filter(|m| query.memory_types.contains(&m.memory_type))
            .filter(|m| m.relevance >= query.min_relevance)
            .take(query.max_results)
            .cloned()
            .collect();

        Ok(filtered)
    }

    async fn consolidate_memories(&self) -> Result<ConsolidationResult, LLMSpellError> {
        // Mock consolidation: just return some fake stats
        Ok(ConsolidationResult {
            consolidated_count: 5,
            deleted_count: 2,
            new_connections: 3,
            duration: std::time::Duration::from_secs(1),
        })
    }

    async fn clear_working_memory(&self, _session_id: &str) -> Result<(), LLMSpellError> {
        let mut memories = self.memories.write().await;
        memories.retain(|m| m.memory_type != MemoryType::Working);
        Ok(())
    }

    async fn get_memory_stats(&self) -> Result<MemoryStats, LLMSpellError> {
        let memories = self.memories.read().await;
        let mut by_type = HashMap::new();

        for memory in memories.iter() {
            let type_name = format!("{:?}", memory.memory_type);
            *by_type.entry(type_name).or_insert(0) += 1;
        }

        Ok(MemoryStats {
            total_memories: memories.len(),
            by_type,
            storage_size: memories.len() * 100, // Mock size calculation
            last_consolidation: Some(chrono::Utc::now()),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_store_and_query_interaction() {
        let memory = MockMemoryIntegration::new();

        // Store an interaction
        let interaction = InteractionLog {
            id: "test-1".to_string(),
            timestamp: chrono::Utc::now(),
            user_input: "Hello".to_string(),
            agent_response: "Hi there!".to_string(),
            metadata: HashMap::new(),
            session_id: Some("session-1".to_string()),
        };

        assert!(memory.store_interaction(interaction).await.is_ok());
        assert_eq!(memory.interaction_count().await, 1);
    }

    #[tokio::test]
    async fn test_memory_query_with_filters() {
        let memory = MockMemoryIntegration::new();

        // Add test memories
        memory
            .add_test_memory("Working memory 1".to_string(), MemoryType::Working)
            .await;
        memory
            .add_test_memory("Episodic memory 1".to_string(), MemoryType::Episodic)
            .await;
        memory
            .add_test_memory("Semantic memory 1".to_string(), MemoryType::Semantic)
            .await;

        // Query only working memory
        let query = ContextQuery {
            query: "test".to_string(),
            max_results: 10,
            memory_types: vec![MemoryType::Working],
            min_relevance: 0.5,
            time_range: None,
        };

        let results = memory.query_context(query).await.unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].memory_type, MemoryType::Working);
    }

    #[tokio::test]
    async fn test_memory_consolidation() {
        let memory = MockMemoryIntegration::new();

        let result = memory.consolidate_memories().await.unwrap();
        assert_eq!(result.consolidated_count, 5);
        assert_eq!(result.deleted_count, 2);
        assert_eq!(result.new_connections, 3);
    }

    #[tokio::test]
    async fn test_clear_working_memory() {
        let memory = MockMemoryIntegration::new();

        // Add memories of different types
        memory
            .add_test_memory("Working 1".to_string(), MemoryType::Working)
            .await;
        memory
            .add_test_memory("Working 2".to_string(), MemoryType::Working)
            .await;
        memory
            .add_test_memory("Episodic 1".to_string(), MemoryType::Episodic)
            .await;

        let stats = memory.get_memory_stats().await.unwrap();
        assert_eq!(stats.total_memories, 3);

        // Clear working memory
        memory.clear_working_memory("session-1").await.unwrap();

        let stats = memory.get_memory_stats().await.unwrap();
        assert_eq!(stats.total_memories, 1); // Only episodic remains
    }
}
