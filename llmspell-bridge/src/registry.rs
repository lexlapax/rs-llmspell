//! ABOUTME: Component registry for managing agents, tools, workflows, and templates
//! ABOUTME: Central registry for all scriptable components accessible from engines

use crate::event_bus_adapter::EventBusAdapter;
use async_trait::async_trait;
use llmspell_core::traits::event::EventConfig;
use llmspell_core::{
    Agent, BaseAgent, ComponentLookup, ExecutionContext, LLMSpellError, Tool, Workflow,
};
use llmspell_events::EventBus;
use llmspell_templates::registry::TemplateRegistry;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

/// Central registry for all components accessible from scripts
pub struct ComponentRegistry {
    agents: Arc<RwLock<HashMap<String, Arc<dyn Agent>>>>,
    tools: Arc<RwLock<HashMap<String, Arc<dyn Tool>>>>,
    workflows: Arc<RwLock<HashMap<String, Arc<dyn Workflow>>>>,
    template_registry: Option<Arc<TemplateRegistry>>,
    event_bus: Option<Arc<EventBus>>,
    event_config: EventConfig,
}

impl ComponentRegistry {
    /// Create a new empty registry
    #[must_use]
    pub fn new() -> Self {
        Self {
            agents: Arc::new(RwLock::new(HashMap::new())),
            tools: Arc::new(RwLock::new(HashMap::new())),
            workflows: Arc::new(RwLock::new(HashMap::new())),
            template_registry: None,
            event_bus: None,
            event_config: EventConfig::default(),
        }
    }

    /// Create a new registry with `EventBus` support
    #[must_use]
    pub fn with_event_bus(event_bus: Arc<EventBus>, config: EventConfig) -> Self {
        Self {
            agents: Arc::new(RwLock::new(HashMap::new())),
            tools: Arc::new(RwLock::new(HashMap::new())),
            workflows: Arc::new(RwLock::new(HashMap::new())),
            template_registry: None,
            event_bus: Some(event_bus),
            event_config: config,
        }
    }

    /// Create a new registry with templates initialized
    ///
    /// # Errors
    ///
    /// Returns error if built-in template registration fails
    pub fn with_templates() -> Result<Self, LLMSpellError> {
        let template_registry =
            TemplateRegistry::with_builtin_templates().map_err(|e| LLMSpellError::Component {
                message: format!("Failed to initialize built-in templates: {e}"),
                source: None,
            })?;

        Ok(Self {
            agents: Arc::new(RwLock::new(HashMap::new())),
            tools: Arc::new(RwLock::new(HashMap::new())),
            workflows: Arc::new(RwLock::new(HashMap::new())),
            template_registry: Some(Arc::new(template_registry)),
            event_bus: None,
            event_config: EventConfig::default(),
        })
    }

    /// Create a new registry with both `EventBus` and templates initialized
    ///
    /// # Errors
    ///
    /// Returns error if built-in template registration fails
    pub fn with_event_bus_and_templates(
        event_bus: Arc<EventBus>,
        config: EventConfig,
    ) -> Result<Self, LLMSpellError> {
        let template_registry =
            TemplateRegistry::with_builtin_templates().map_err(|e| LLMSpellError::Component {
                message: format!("Failed to initialize built-in templates: {e}"),
                source: None,
            })?;

        Ok(Self {
            agents: Arc::new(RwLock::new(HashMap::new())),
            tools: Arc::new(RwLock::new(HashMap::new())),
            workflows: Arc::new(RwLock::new(HashMap::new())),
            template_registry: Some(Arc::new(template_registry)),
            event_bus: Some(event_bus),
            event_config: config,
        })
    }

    /// Get the `EventBus` if available
    #[must_use]
    pub fn event_bus(&self) -> Option<Arc<EventBus>> {
        self.event_bus.clone()
    }

    /// Get the `TemplateRegistry` if available
    #[must_use]
    pub fn template_registry(&self) -> Option<Arc<TemplateRegistry>> {
        self.template_registry.clone()
    }

    /// Create an `ExecutionContext` with registry services (state, events, etc.)
    #[must_use]
    pub fn create_execution_context(&self, base_context: ExecutionContext) -> ExecutionContext {
        let mut ctx = base_context;

        // Add events if available and enabled
        if let Some(ref event_bus) = self.event_bus {
            if self.event_config.enabled {
                let adapter =
                    EventBusAdapter::with_config(event_bus.clone(), self.event_config.clone());
                ctx.events = Some(Arc::new(adapter));
            }
        }

        // Note: State would be added here too if ComponentRegistry had state_manager
        // For now, state is managed separately

        ctx
    }

    /// Register an agent
    ///
    /// # Errors
    ///
    /// Returns an error if an agent with the same name is already registered
    ///
    /// # Panics
    ///
    /// Panics if the agents lock is poisoned
    pub fn register_agent(&self, name: String, agent: Arc<dyn Agent>) -> Result<(), LLMSpellError> {
        let mut agents = self.agents.write().unwrap();
        if agents.contains_key(&name) {
            return Err(LLMSpellError::Validation {
                field: Some("agent_name".to_string()),
                message: format!("Agent '{name}' already registered"),
            });
        }
        agents.insert(name, agent);
        drop(agents); // Explicitly drop the lock
        Ok(())
    }

    /// Get an agent by name
    ///
    /// # Panics
    ///
    /// Panics if the agents lock is poisoned
    #[must_use]
    pub fn get_agent(&self, name: &str) -> Option<Arc<dyn Agent>> {
        let agents = self.agents.read().unwrap();
        agents.get(name).cloned()
    }

    /// List all registered agents
    ///
    /// # Panics
    ///
    /// Panics if the agents lock is poisoned
    #[must_use]
    pub fn list_agents(&self) -> Vec<String> {
        let agents = self.agents.read().unwrap();
        agents.keys().cloned().collect()
    }

    /// Register a tool
    ///
    /// # Errors
    ///
    /// Returns an error if a tool with the same name is already registered
    ///
    /// # Panics
    ///
    /// Panics if the tools lock is poisoned
    pub fn register_tool(&self, name: String, tool: Arc<dyn Tool>) -> Result<(), LLMSpellError> {
        let mut tools = self.tools.write().unwrap();
        if tools.contains_key(&name) {
            return Err(LLMSpellError::Validation {
                field: Some("tool_name".to_string()),
                message: format!("Tool '{name}' already registered"),
            });
        }
        tools.insert(name, tool);
        drop(tools); // Explicitly drop the lock
        Ok(())
    }

    /// Get a tool by name
    ///
    /// # Panics
    ///
    /// Panics if the tools lock is poisoned
    #[must_use]
    pub fn get_tool(&self, name: &str) -> Option<Arc<dyn Tool>> {
        let tools = self.tools.read().unwrap();
        tools.get(name).cloned()
    }

    /// List all registered tools
    ///
    /// # Panics
    ///
    /// Panics if the tools lock is poisoned
    #[must_use]
    pub fn list_tools(&self) -> Vec<String> {
        let tools = self.tools.read().unwrap();
        tools.keys().cloned().collect()
    }

    /// Register a workflow
    ///
    /// # Errors
    ///
    /// Returns an error if a workflow with the same name is already registered
    ///
    /// # Panics
    ///
    /// Panics if the workflows lock is poisoned
    pub fn register_workflow(
        &self,
        name: String,
        workflow: Arc<dyn Workflow>,
    ) -> Result<(), LLMSpellError> {
        let mut workflows = self.workflows.write().unwrap();
        if workflows.contains_key(&name) {
            return Err(LLMSpellError::Validation {
                field: Some("workflow_name".to_string()),
                message: format!("Workflow '{name}' already registered"),
            });
        }
        workflows.insert(name, workflow);
        drop(workflows); // Explicitly drop the lock
        Ok(())
    }

    /// Get a workflow by name
    ///
    /// # Panics
    ///
    /// Panics if the workflows lock is poisoned
    #[must_use]
    pub fn get_workflow(&self, name: &str) -> Option<Arc<dyn Workflow>> {
        let workflows = self.workflows.read().unwrap();
        workflows.get(name).cloned()
    }

    /// List all registered workflows
    ///
    /// # Panics
    ///
    /// Panics if the workflows lock is poisoned
    #[must_use]
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

/// Implementation of `ComponentLookup` trait for `ComponentRegistry`
/// This allows the registry to be used from llmspell-workflows without circular deps
#[async_trait]
impl ComponentLookup for ComponentRegistry {
    async fn get_agent(&self, name: &str) -> Option<Arc<dyn Agent>> {
        self.get_agent(name)
    }

    async fn get_tool(&self, name: &str) -> Option<Arc<dyn Tool>> {
        self.get_tool(name)
    }

    async fn get_workflow(&self, name: &str) -> Option<Arc<dyn Workflow>> {
        self.get_workflow(name)
    }

    async fn get_component(&self, component_type: &str, name: &str) -> Option<Arc<dyn BaseAgent>> {
        match component_type {
            "agent" => self.get_agent(name).map(|a| a as Arc<dyn BaseAgent>),
            "tool" => self.get_tool(name).map(|t| t as Arc<dyn BaseAgent>),
            "workflow" => self.get_workflow(name).map(|w| w as Arc<dyn BaseAgent>),
            _ => None,
        }
    }

    async fn list_agents(&self) -> Vec<String> {
        self.list_agents()
    }

    async fn list_tools(&self) -> Vec<String> {
        self.list_tools()
    }

    async fn list_workflows(&self) -> Vec<String> {
        self.list_workflows()
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

        async fn execute_impl(
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
            Ok(AgentOutput::text(format!("Error: {error}")))
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
