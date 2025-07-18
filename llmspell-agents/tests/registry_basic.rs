//! ABOUTME: Basic integration tests for the agent registry system
//! ABOUTME: Tests core registry operations without complex dependencies

use llmspell_agents::registry::{
    AgentMetadata, AgentMetrics, AgentQuery, AgentRegistry, AgentStatus, InMemoryAgentRegistry,
    PersistentAgentRegistry,
};
use llmspell_core::{
    traits::{
        agent::{Agent, AgentConfig as CoreAgentConfig, ConversationMessage},
        base_agent::BaseAgent,
    },
    types::{AgentInput, AgentOutput, ExecutionContext},
    ComponentMetadata, LLMSpellError,
};
use llmspell_storage::MemoryBackend;
use std::{collections::HashMap, sync::Arc};

/// Simple mock agent for testing
struct TestAgent {
    metadata: ComponentMetadata,
    config: CoreAgentConfig,
}

impl TestAgent {
    fn new(id: &str, name: &str) -> Self {
        Self {
            metadata: ComponentMetadata::new(id.to_string(), name.to_string()),
            config: CoreAgentConfig::default(),
        }
    }
}

#[async_trait::async_trait]
impl BaseAgent for TestAgent {
    fn metadata(&self) -> &ComponentMetadata {
        &self.metadata
    }

    async fn validate_input(&self, _input: &AgentInput) -> llmspell_core::Result<()> {
        Ok(())
    }

    async fn execute(
        &self,
        _input: AgentInput,
        _context: ExecutionContext,
    ) -> llmspell_core::Result<AgentOutput> {
        Ok(AgentOutput::text("Test response"))
    }

    async fn handle_error(&self, _error: LLMSpellError) -> llmspell_core::Result<AgentOutput> {
        Ok(AgentOutput::text("Error handled"))
    }
}

#[async_trait::async_trait]
impl Agent for TestAgent {
    fn config(&self) -> &CoreAgentConfig {
        &self.config
    }

    async fn get_conversation(&self) -> llmspell_core::Result<Vec<ConversationMessage>> {
        Ok(vec![])
    }

    async fn add_message(&mut self, _message: ConversationMessage) -> llmspell_core::Result<()> {
        Ok(())
    }

    async fn clear_conversation(&mut self) -> llmspell_core::Result<()> {
        Ok(())
    }
}

#[tokio::test]
async fn test_in_memory_registry_basic() {
    let registry = InMemoryAgentRegistry::new();

    // Create test agent
    let agent = Arc::new(TestAgent::new("test-1", "Test Agent 1"));

    let metadata = AgentMetadata {
        id: "test-1".to_string(),
        name: "Test Agent 1".to_string(),
        agent_type: "test".to_string(),
        description: "A test agent".to_string(),
        categories: vec!["test".to_string()],
        custom_metadata: HashMap::new(),
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
        status: AgentStatus::Active,
        metrics: AgentMetrics::default(),
    };

    // Test registration
    registry
        .register_agent("test-1".to_string(), agent.clone(), metadata.clone())
        .await
        .unwrap();

    // Test exists
    assert!(registry.exists("test-1").await.unwrap());

    // Test get metadata
    let retrieved = registry.get_metadata("test-1").await.unwrap();
    assert!(retrieved.is_some());
    assert_eq!(retrieved.unwrap().name, "Test Agent 1");

    // Test count
    assert_eq!(registry.count_agents().await.unwrap(), 1);

    // Test update status
    registry
        .update_status("test-1", AgentStatus::Paused)
        .await
        .unwrap();
    let updated = registry.get_metadata("test-1").await.unwrap().unwrap();
    assert_eq!(updated.status, AgentStatus::Paused);

    // Test unregister
    registry.unregister_agent("test-1").await.unwrap();
    assert!(!registry.exists("test-1").await.unwrap());
}

#[tokio::test]
async fn test_registry_query() {
    let registry = InMemoryAgentRegistry::new();

    // Register multiple agents
    for i in 1..=3 {
        let agent = Arc::new(TestAgent::new(
            &format!("agent-{}", i),
            &format!("Agent {}", i),
        ));

        let metadata = AgentMetadata {
            id: format!("agent-{}", i),
            name: format!("Agent {}", i),
            agent_type: if i % 2 == 0 { "even" } else { "odd" }.to_string(),
            description: format!("Test agent {}", i),
            categories: vec![format!("group-{}", if i <= 2 { "a" } else { "b" })],
            custom_metadata: HashMap::new(),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            status: if i % 2 == 0 {
                AgentStatus::Active
            } else {
                AgentStatus::Paused
            },
            metrics: AgentMetrics::default(),
        };

        registry
            .register_agent(format!("agent-{}", i), agent, metadata)
            .await
            .unwrap();
    }

    // Test query by type
    let query = AgentQuery {
        type_filter: Some("even".to_string()),
        ..Default::default()
    };
    let results = registry.query_agents(&query).await.unwrap();
    assert_eq!(results.len(), 1); // Only agent-2

    // Test query by status
    let query = AgentQuery {
        status_filter: Some(AgentStatus::Active),
        ..Default::default()
    };
    let results = registry.query_agents(&query).await.unwrap();
    assert_eq!(results.len(), 1); // Only agent-2

    // Test query by category
    let query = AgentQuery {
        category_filter: vec!["group-a".to_string()],
        ..Default::default()
    };
    let results = registry.query_agents(&query).await.unwrap();
    assert_eq!(results.len(), 2); // agents 1 and 2
}

#[tokio::test]
async fn test_persistent_registry() {
    let storage = Arc::new(MemoryBackend::new());
    let registry = PersistentAgentRegistry::new(storage.clone()).await.unwrap();

    // Register an agent
    let agent = Arc::new(TestAgent::new("persist-1", "Persistent Agent"));

    let metadata = AgentMetadata {
        id: "persist-1".to_string(),
        name: "Persistent Agent".to_string(),
        agent_type: "persistent".to_string(),
        description: "A persistent test agent".to_string(),
        categories: vec!["persistent".to_string()],
        custom_metadata: HashMap::new(),
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
        status: AgentStatus::Active,
        metrics: AgentMetrics::default(),
    };

    registry
        .register_agent("persist-1".to_string(), agent, metadata)
        .await
        .unwrap();

    // Persist the state
    registry.persist().await.unwrap();

    // Create a new registry with the same storage
    let new_registry = PersistentAgentRegistry::new(storage).await.unwrap();

    // Verify data persisted
    assert!(new_registry.exists("persist-1").await.unwrap());
    let retrieved = new_registry
        .get_metadata("persist-1")
        .await
        .unwrap()
        .unwrap();
    assert_eq!(retrieved.name, "Persistent Agent");
}

#[tokio::test]
async fn test_metrics_update() {
    let registry = InMemoryAgentRegistry::new();

    let agent = Arc::new(TestAgent::new("metrics-1", "Metrics Agent"));

    let metadata = AgentMetadata {
        id: "metrics-1".to_string(),
        name: "Metrics Agent".to_string(),
        agent_type: "metrics".to_string(),
        description: "Testing metrics".to_string(),
        categories: vec![],
        custom_metadata: HashMap::new(),
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
        status: AgentStatus::Active,
        metrics: AgentMetrics::default(),
    };

    registry
        .register_agent("metrics-1".to_string(), agent, metadata)
        .await
        .unwrap();

    // Update metrics
    let new_metrics = AgentMetrics {
        execution_count: 100,
        success_rate: 0.95,
        avg_execution_time_ms: 50.0,
        last_execution_time: Some(chrono::Utc::now()),
        error_count: 5,
        last_error: None,
    };

    registry
        .update_metrics("metrics-1", new_metrics.clone())
        .await
        .unwrap();

    // Verify metrics updated
    let updated = registry.get_metadata("metrics-1").await.unwrap().unwrap();
    assert_eq!(updated.metrics.execution_count, 100);
    assert_eq!(updated.metrics.success_rate, 0.95);
    assert_eq!(updated.metrics.avg_execution_time_ms, 50.0);
}

#[tokio::test]
async fn test_category_management() {
    use llmspell_agents::registry::categories::{
        AgentTag, CategoryBuilder, CategoryManager, TagType,
    };

    let mut manager = CategoryManager::new();

    // Test standard categories are loaded
    assert!(manager.get_category("tool-agents").is_some());
    assert!(manager.get_category("llm-agents").is_some());

    // Test adding custom category
    let custom_category = CategoryBuilder::new("custom-tools".to_string())
        .name("Custom Tools".to_string())
        .description("Custom tool agents".to_string())
        .parent("tool-agents".to_string())
        .build();

    manager.add_category(custom_category).unwrap();

    // Test hierarchy
    let hierarchy = manager.get_hierarchy("custom-tools");
    assert_eq!(hierarchy, vec!["tool-agents", "custom-tools"]);

    // Test agent categorization
    manager
        .assign_to_category("agent-1", "custom-tools")
        .unwrap();
    manager.assign_to_category("agent-1", "llm-agents").unwrap();

    let agent1_cats = manager.get_agent_categories("agent-1");
    assert_eq!(agent1_cats.len(), 2);

    // Test tagging
    let version_tag = AgentTag {
        name: "version".to_string(),
        value: Some("1.0.0".to_string()),
        tag_type: TagType::Version,
    };

    manager.add_tag("agent-1", version_tag).unwrap();

    let agent1_tags = manager.get_agent_tags("agent-1");
    assert_eq!(agent1_tags.len(), 1);
}
