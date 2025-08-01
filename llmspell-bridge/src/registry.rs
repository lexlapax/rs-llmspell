//! ABOUTME: Component registry for managing agents, tools, and workflows
//! ABOUTME: Central registry for all scriptable components accessible from engines

use llmspell_core::{Agent, LLMSpellError, Tool, Workflow};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

/// Central registry for all components accessible from scripts
pub struct ComponentRegistry {
    agents: Arc<RwLock<HashMap<String, Arc<dyn Agent>>>>,
    tools: Arc<RwLock<HashMap<String, Arc<dyn Tool>>>>,
    workflows: Arc<RwLock<HashMap<String, Arc<dyn Workflow>>>>,
}

impl ComponentRegistry {
    /// Create a new empty registry
    pub fn new() -> Self {
        Self {
            agents: Arc::new(RwLock::new(HashMap::new())),
            tools: Arc::new(RwLock::new(HashMap::new())),
            workflows: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Register an agent
    pub fn register_agent(&self, name: String, agent: Arc<dyn Agent>) -> Result<(), LLMSpellError> {
        let mut agents = self.agents.write().unwrap();
        if agents.contains_key(&name) {
            return Err(LLMSpellError::Validation {
                field: Some("agent_name".to_string()),
                message: format!("Agent '{}' already registered", name),
            });
        }
        agents.insert(name, agent);
        Ok(())
    }

    /// Get an agent by name
    pub fn get_agent(&self, name: &str) -> Option<Arc<dyn Agent>> {
        let agents = self.agents.read().unwrap();
        agents.get(name).cloned()
    }

    /// List all registered agents
    pub fn list_agents(&self) -> Vec<String> {
        let agents = self.agents.read().unwrap();
        agents.keys().cloned().collect()
    }

    /// Register a tool
    pub fn register_tool(&self, name: String, tool: Arc<dyn Tool>) -> Result<(), LLMSpellError> {
        let mut tools = self.tools.write().unwrap();
        if tools.contains_key(&name) {
            return Err(LLMSpellError::Validation {
                field: Some("tool_name".to_string()),
                message: format!("Tool '{}' already registered", name),
            });
        }
        tools.insert(name, tool);
        Ok(())
    }

    /// Get a tool by name
    pub fn get_tool(&self, name: &str) -> Option<Arc<dyn Tool>> {
        let tools = self.tools.read().unwrap();
        tools.get(name).cloned()
    }

    /// List all registered tools
    pub fn list_tools(&self) -> Vec<String> {
        let tools = self.tools.read().unwrap();
        tools.keys().cloned().collect()
    }

    /// Register a workflow
    pub fn register_workflow(
        &self,
        name: String,
        workflow: Arc<dyn Workflow>,
    ) -> Result<(), LLMSpellError> {
        let mut workflows = self.workflows.write().unwrap();
        if workflows.contains_key(&name) {
            return Err(LLMSpellError::Validation {
                field: Some("workflow_name".to_string()),
                message: format!("Workflow '{}' already registered", name),
            });
        }
        workflows.insert(name, workflow);
        Ok(())
    }

    /// Get a workflow by name
    pub fn get_workflow(&self, name: &str) -> Option<Arc<dyn Workflow>> {
        let workflows = self.workflows.read().unwrap();
        workflows.get(name).cloned()
    }

    /// List all registered workflows
    pub fn list_workflows(&self) -> Vec<String> {
        let workflows = self.workflows.read().unwrap();
        workflows.keys().cloned().collect()
    }
}

impl Default for ComponentRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;
    use llmspell_core::traits::agent::AgentConfig;
    use llmspell_core::types::{AgentInput, AgentOutput};
    use llmspell_core::BaseAgent;
    use llmspell_core::ExecutionContext;

    struct MockAgent {
        metadata: llmspell_core::ComponentMetadata,
        config: AgentConfig,
    }

    impl MockAgent {
        fn new() -> Self {
            Self {
                metadata: llmspell_core::ComponentMetadata::new(
                    "mock".to_string(),
                    "Mock agent".to_string(),
                ),
                config: AgentConfig::default(),
            }
        }
    }

    #[async_trait]
    impl BaseAgent for MockAgent {
        fn metadata(&self) -> &llmspell_core::ComponentMetadata {
            &self.metadata
        }

        async fn execute(
            &self,
            _input: AgentInput,
            _context: ExecutionContext,
        ) -> Result<AgentOutput, LLMSpellError> {
            Ok(AgentOutput::text("mock output"))
        }

        async fn validate_input(&self, _input: &AgentInput) -> Result<(), LLMSpellError> {
            Ok(())
        }

        async fn handle_error(&self, error: LLMSpellError) -> Result<AgentOutput, LLMSpellError> {
            Ok(AgentOutput::text(format!("Error: {}", error)))
        }
    }

    #[async_trait]
    impl Agent for MockAgent {
        fn config(&self) -> &AgentConfig {
            &self.config
        }

        async fn get_conversation(
            &self,
        ) -> Result<Vec<llmspell_core::traits::agent::ConversationMessage>, LLMSpellError> {
            Ok(vec![])
        }

        async fn add_message(
            &self,
            _message: llmspell_core::traits::agent::ConversationMessage,
        ) -> Result<(), LLMSpellError> {
            Ok(())
        }

        async fn clear_conversation(&self) -> Result<(), LLMSpellError> {
            Ok(())
        }
    }
    #[test]
    fn test_agent_registration() {
        let registry = ComponentRegistry::new();
        let agent = Arc::new(MockAgent::new());

        assert!(registry.register_agent("test".to_string(), agent).is_ok());
        assert!(registry.get_agent("test").is_some());
        assert_eq!(registry.list_agents(), vec!["test"]);

        // Duplicate registration should fail
        let agent2 = Arc::new(MockAgent::new());
        assert!(registry.register_agent("test".to_string(), agent2).is_err());
    }
}
