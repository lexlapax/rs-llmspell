//! Concurrency and thread-safety tests for llmspell-core
//!
//! These tests verify that components work correctly under concurrent access

use async_trait::async_trait;
use llmspell_core::{
    traits::{
        agent::{Agent, AgentConfig, ConversationMessage},
        base_agent::BaseAgent,
    },
    types::{AgentInput, AgentOutput},
    ComponentId, ComponentMetadata, ExecutionContext, Result,
};
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc, Mutex,
};
use tokio::sync::RwLock;

/// Thread-safe agent implementation
struct ConcurrentAgent {
    metadata: ComponentMetadata,
    config: AgentConfig,
    conversation: Arc<RwLock<Vec<ConversationMessage>>>,
    execution_count: Arc<AtomicUsize>,
}

impl ConcurrentAgent {
    fn new(name: &str) -> Self {
        Self {
            metadata: ComponentMetadata::new(
                name.to_string(),
                format!("Concurrent agent: {}", name),
            ),
            config: AgentConfig::default(),
            conversation: Arc::new(RwLock::new(Vec::new())),
            execution_count: Arc::new(AtomicUsize::new(0)),
        }
    }

    fn get_execution_count(&self) -> usize {
        self.execution_count.load(Ordering::SeqCst)
    }
}

#[async_trait]
impl BaseAgent for ConcurrentAgent {
    fn metadata(&self) -> &ComponentMetadata {
        &self.metadata
    }

    async fn execute(&self, input: AgentInput, _context: ExecutionContext) -> Result<AgentOutput> {
        // Increment execution count atomically
        let count = self.execution_count.fetch_add(1, Ordering::SeqCst) + 1;

        // Add to conversation with write lock
        {
            let mut conv = self.conversation.write().await;
            conv.push(ConversationMessage::user(input.text.clone()));
            conv.push(ConversationMessage::assistant(format!(
                "Response #{}",
                count
            )));
        }

        // Simulate some work
        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;

        Ok(AgentOutput::text(format!("Execution #{}", count)))
    }

    async fn validate_input(&self, input: &AgentInput) -> Result<()> {
        if input.text.is_empty() {
            return Err(llmspell_core::LLMSpellError::Validation {
                message: "Empty prompt".to_string(),
                field: Some("prompt".to_string()),
            });
        }
        Ok(())
    }

    async fn handle_error(&self, error: llmspell_core::LLMSpellError) -> Result<AgentOutput> {
        Ok(AgentOutput::text(format!("Error: {}", error)))
    }
}

#[async_trait]
impl Agent for ConcurrentAgent {
    fn config(&self) -> &AgentConfig {
        &self.config
    }

    async fn get_conversation(&self) -> Result<Vec<ConversationMessage>> {
        let conv = self.conversation.read().await;
        Ok(conv.clone())
    }

    async fn add_message(&self, message: ConversationMessage) -> Result<()> {
        let mut conv = self.conversation.write().await;
        conv.push(message);
        Ok(())
    }

    async fn clear_conversation(&self) -> Result<()> {
        let mut conv = self.conversation.write().await;
        conv.clear();
        Ok(())
    }
}

#[cfg_attr(test_category = "integration")]
#[cfg_attr(test_category = "core")]
#[cfg_attr(test_category = "performance")]
#[tokio::test]
async fn test_concurrent_agent_execution() {
    let agent = Arc::new(ConcurrentAgent::new("concurrent-test"));
    let num_tasks = 100;

    // Spawn multiple concurrent tasks
    let mut handles = Vec::new();
    for i in 0..num_tasks {
        let agent_clone = Arc::clone(&agent);
        let handle = tokio::spawn(async move {
            let input = AgentInput::text(format!("Request {}", i));
            let context = ExecutionContext::with_conversation(format!("session-{}", i));
            agent_clone.execute(input, context).await
        });
        handles.push(handle);
    }

    // Wait for all tasks to complete
    let mut results = Vec::new();
    for handle in handles {
        let result = handle.await.unwrap();
        results.push(result);
    }

    // Verify all executions completed
    assert_eq!(results.len(), num_tasks);
    for result in &results {
        assert!(result.is_ok());
    }

    // Verify execution count
    assert_eq!(agent.get_execution_count(), num_tasks);

    // Verify conversation has correct number of messages
    let conv = agent.get_conversation().await.unwrap();
    assert_eq!(conv.len(), num_tasks * 2); // User + Assistant messages
}

#[cfg_attr(test_category = "integration")]
#[cfg_attr(test_category = "core")]
#[cfg_attr(test_category = "performance")]
#[tokio::test]
async fn test_component_id_thread_safety() {
    let num_threads = 50;
    let names = Arc::new(Mutex::new(Vec::new()));

    // Generate unique names in parallel
    let mut handles = Vec::new();
    for i in 0..num_threads {
        let names_clone = Arc::clone(&names);
        let handle = std::thread::spawn(move || {
            let name = format!("component-{}", i);
            let id = ComponentId::from_name(&name);
            names_clone.lock().unwrap().push((name, id));
        });
        handles.push(handle);
    }

    // Wait for all threads
    for handle in handles {
        handle.join().unwrap();
    }

    // Verify deterministic ID generation
    let names_vec = names.lock().unwrap();
    for (name, id) in names_vec.iter() {
        let id2 = ComponentId::from_name(name);
        assert_eq!(*id, id2, "ComponentId generation should be deterministic");
    }
}

#[cfg_attr(test_category = "integration")]
#[cfg_attr(test_category = "core")]
#[cfg_attr(test_category = "performance")]
#[tokio::test]
async fn test_concurrent_conversation_modifications() {
    let agent = Arc::new(RwLock::new(ConcurrentAgent::new("conversation-test")));

    // Concurrent readers
    let mut read_handles = Vec::new();
    for i in 0..10 {
        let agent_clone = Arc::clone(&agent);
        let handle = tokio::spawn(async move {
            tokio::time::sleep(tokio::time::Duration::from_millis(i * 5)).await;
            let agent = agent_clone.read().await;
            agent.get_conversation().await
        });
        read_handles.push(handle);
    }

    // Concurrent writer
    let agent_clone = Arc::clone(&agent);
    let write_handle = tokio::spawn(async move {
        for i in 0..20 {
            tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
            let agent = agent_clone.read().await;
            agent
                .add_message(ConversationMessage::user(format!("Message {}", i)))
                .await
                .unwrap();
        }
    });

    // Wait for all operations
    for handle in read_handles {
        let _ = handle.await.unwrap();
    }
    write_handle.await.unwrap();

    // Verify final state
    let agent = agent.read().await;
    let conv = agent.get_conversation().await.unwrap();
    assert_eq!(conv.len(), 20);
}

#[cfg_attr(test_category = "integration")]
#[cfg_attr(test_category = "core")]
#[cfg_attr(test_category = "performance")]
#[tokio::test]
async fn test_metadata_immutability() {
    // ComponentMetadata should be safely shareable
    let metadata = Arc::new(ComponentMetadata::new(
        "shared-component".to_string(),
        "A shared component".to_string(),
    ));

    let mut handles = Vec::new();
    for _ in 0..10 {
        let metadata_clone = Arc::clone(&metadata);
        let handle = tokio::spawn(async move {
            // Multiple readers accessing metadata concurrently
            assert_eq!(metadata_clone.name, "shared-component");
            assert_eq!(metadata_clone.description, "A shared component");
            // Serialize to ensure thread safety
            let _ = serde_json::to_string(&*metadata_clone).unwrap();
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.await.unwrap();
    }
}

#[cfg_attr(test_category = "integration")]
#[cfg_attr(test_category = "core")]
#[cfg_attr(test_category = "performance")]
#[tokio::test]
async fn test_execution_context_concurrent_access() {
    let context = Arc::new({
        let mut context = ExecutionContext::with_conversation("shared-session".to_string());
        context.user_id = Some("user-123".to_string());
        context
            .with_data("KEY1".to_string(), serde_json::json!("value1"))
            .with_data("KEY2".to_string(), serde_json::json!("value2"))
    });

    let mut handles = Vec::new();
    for i in 0..20 {
        let context_clone = Arc::clone(&context);
        let handle = tokio::spawn(async move {
            // Concurrent reads
            assert_eq!(
                context_clone.conversation_id,
                Some("shared-session".to_string())
            );
            assert_eq!(context_clone.user_id, Some("user-123".to_string()));

            // Access data variables
            if i % 2 == 0 {
                assert_eq!(
                    context_clone.data.get("KEY1"),
                    Some(&serde_json::json!("value1"))
                );
            } else {
                assert_eq!(
                    context_clone.data.get("KEY2"),
                    Some(&serde_json::json!("value2"))
                );
            }
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.await.unwrap();
    }
}

#[cfg_attr(test_category = "integration")]
#[cfg_attr(test_category = "core")]
#[cfg_attr(test_category = "performance")]
#[test]
fn test_error_thread_safety() {
    use std::thread;

    // Errors should be Send + Sync
    let error = Arc::new(llmspell_core::LLMSpellError::Component {
        message: "Shared error".to_string(),
        source: None,
    });

    let mut handles = Vec::new();
    for _ in 0..10 {
        let error_clone = Arc::clone(&error);
        let handle = thread::spawn(move || {
            // Access error from multiple threads
            assert!(error_clone.to_string().contains("Shared error"));
            assert_eq!(
                error_clone.severity(),
                llmspell_core::error::ErrorSeverity::Error
            );
            assert_eq!(
                error_clone.category(),
                llmspell_core::error::ErrorCategory::Logic
            );
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }
}
