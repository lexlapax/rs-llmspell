//! ABOUTME: Hierarchical agent composition for parent-child agent relationships
//! ABOUTME: Enables building tree structures of agents with command and control patterns

use super::traits::{
    Capability, CapabilityCategory, Composable, CompositeAgent, CompositionError,
    CompositionMetadata, CompositionType, ExecutionPattern, HierarchicalAgent, HierarchyEvent,
};
use async_trait::async_trait;
use llmspell_core::types::{AgentInput, AgentOutput};
use llmspell_core::{
    traits::tool_capable::{ToolInfo, ToolQuery},
    BaseAgent, ComponentMetadata, ExecutionContext, LLMSpellError, Result, Tool, ToolCapable,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::{Arc, RwLock, Weak};
use tokio::sync::RwLock as TokioRwLock;

/// A hierarchical composite agent that manages parent-child relationships
pub struct HierarchicalCompositeAgent {
    /// Component metadata
    metadata: ComponentMetadata,

    /// Parent agent reference (weak to avoid cycles)
    parent: RwLock<Option<Weak<dyn HierarchicalAgent>>>,

    /// Child agents
    children: TokioRwLock<Vec<Arc<dyn HierarchicalAgent>>>,

    /// Components managed by this agent (includes non-hierarchical agents)
    components: TokioRwLock<HashMap<String, Arc<dyn BaseAgent>>>,

    /// Tools available to this agent
    tools: TokioRwLock<HashMap<String, Arc<dyn Tool>>>,

    /// Execution pattern for child/component execution
    execution_pattern: RwLock<ExecutionPattern>,

    /// Capabilities exposed by this agent
    capabilities: RwLock<Vec<Capability>>,

    /// Configuration
    config: HierarchicalConfig,

    /// Metrics
    metrics: RwLock<HierarchicalMetrics>,
}

/// Configuration for hierarchical agents
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HierarchicalConfig {
    /// Maximum depth allowed in the hierarchy
    pub max_depth: usize,
    /// Whether to propagate events up the hierarchy
    pub propagate_up: bool,
    /// Whether to propagate events down the hierarchy
    pub propagate_down: bool,
    /// Maximum number of children allowed
    pub max_children: Option<usize>,
    /// Whether children inherit parent capabilities
    pub inherit_capabilities: bool,
    /// Timeout for child operations
    pub child_operation_timeout: std::time::Duration,
}

impl Default for HierarchicalConfig {
    fn default() -> Self {
        Self {
            max_depth: 10,
            propagate_up: true,
            propagate_down: true,
            max_children: None,
            inherit_capabilities: true,
            child_operation_timeout: std::time::Duration::from_secs(30),
        }
    }
}

/// Metrics for hierarchical agents
#[derive(Debug, Default)]
struct HierarchicalMetrics {
    /// Total number of delegations
    total_delegations: u64,
    /// Successful delegations
    successful_delegations: u64,
    /// Failed delegations
    failed_delegations: u64,
    /// Events propagated up
    events_propagated_up: u64,
    /// Events propagated down
    events_propagated_down: u64,
}

impl HierarchicalCompositeAgent {
    /// Create a new hierarchical composite agent
    pub fn new(name: impl Into<String>, config: HierarchicalConfig) -> Self {
        let name = name.into();
        let description = format!("Hierarchical composite agent: {name}");
        Self {
            metadata: ComponentMetadata::new(name, description),
            parent: RwLock::new(None),
            children: TokioRwLock::new(Vec::new()),
            components: TokioRwLock::new(HashMap::new()),
            tools: TokioRwLock::new(HashMap::new()),
            execution_pattern: RwLock::new(ExecutionPattern::Sequential),
            capabilities: RwLock::new(Vec::new()),
            config,
            metrics: RwLock::new(HierarchicalMetrics::default()),
        }
    }

    /// Set the parent of this agent
    ///
    /// # Errors
    ///
    /// Currently never returns an error, but the Result type is provided for future
    /// extensibility (e.g., validation of parent-child relationships, cycle detection).
    pub fn set_parent(&self, parent: Weak<dyn HierarchicalAgent>) -> Result<()> {
        let mut parent_guard = self.parent.write().unwrap();
        *parent_guard = Some(parent);
        drop(parent_guard);
        Ok(())
    }

    /// Get the current depth by traversing up the hierarchy
    fn calculate_depth(&self) -> usize {
        let parent_guard = self.parent.read().unwrap();
        match &*parent_guard {
            Some(weak_parent) => {
                if let Some(parent) = weak_parent.upgrade() {
                    parent.depth() + 1
                } else {
                    0
                }
            }
            None => 0,
        }
    }

    /// Check if adding a child would create a cycle
    async fn would_create_cycle(&self, child: &Arc<dyn HierarchicalAgent>) -> bool {
        // Check if child is an ancestor of self
        let mut current = self.parent.read().unwrap().clone();
        while let Some(weak_parent) = current {
            if let Some(parent) = weak_parent.upgrade() {
                if Arc::ptr_eq(&parent, child) {
                    return true;
                }
                current = parent
                    .parent()
                    .and_then(|p| p.parent())
                    .map(|p| Arc::downgrade(&p));
            } else {
                break;
            }
        }
        false
    }

    /// Aggregate capabilities from all children and components
    #[allow(dead_code)]
    async fn aggregate_capabilities(&self) -> Vec<Capability> {
        let mut capabilities = self.capabilities.read().unwrap().clone();

        // Add capabilities from children
        let children = self.children.read().await;
        for _child in children.iter() {
            // Access child metadata - this is a limitation as metadata() is sync
            // In a real implementation, we'd need a different approach
            capabilities.push(Capability {
                name: "child-capability".to_string(),
                category: CapabilityCategory::Custom("inherited".to_string()),
                version: None,
                metadata: HashMap::new(),
            });
        }

        // Add capabilities from components
        let components = self.components.read().await;
        for (_, _component) in components.iter() {
            // Access component metadata - this is a limitation as metadata() is sync
            // In a real implementation, we'd need a different approach
            capabilities.push(Capability {
                name: "component-capability".to_string(),
                category: CapabilityCategory::Custom("component".to_string()),
                version: None,
                metadata: HashMap::new(),
            });
        }

        capabilities
    }
}

#[async_trait]
impl BaseAgent for HierarchicalCompositeAgent {
    fn metadata(&self) -> &ComponentMetadata {
        &self.metadata
    }

    async fn execute(&self, input: AgentInput, context: ExecutionContext) -> Result<AgentOutput> {
        let pattern = self.execution_pattern.read().unwrap().clone();
        let value = serde_json::to_value(&input)?;
        let result = self.execute_pattern(pattern, value, &context).await?;
        Ok(AgentOutput::text(result.to_string()))
    }

    async fn validate_input(&self, input: &AgentInput) -> Result<()> {
        // Basic validation - can be extended
        if input.text.is_empty() {
            return Err(LLMSpellError::Validation {
                message: "Input prompt cannot be empty".to_string(),
                field: Some("prompt".to_string()),
            });
        }
        Ok(())
    }

    async fn handle_error(&self, error: LLMSpellError) -> Result<AgentOutput> {
        Ok(AgentOutput::text(format!(
            "Hierarchical agent error: {error}"
        )))
    }
}

#[async_trait]
impl ToolCapable for HierarchicalCompositeAgent {
    async fn discover_tools(&self, query: &ToolQuery) -> Result<Vec<ToolInfo>> {
        let tools = self.tools.read().await;
        let mut infos = Vec::new();

        for (name, tool) in tools.iter() {
            // Simple filtering based on text search
            if let Some(ref text) = query.text_search {
                if !name.contains(text) {
                    continue;
                }
            }

            let info = ToolInfo::new(
                name.clone(),
                tool.metadata().description.clone(),
                "agent-tool".to_string(),
                "safe".to_string(),
            );
            infos.push(info);
        }

        Ok(infos)
    }

    async fn invoke_tool(
        &self,
        tool_name: &str,
        parameters: Value,
        context: ExecutionContext,
    ) -> Result<AgentOutput> {
        let tools = self.tools.read().await;
        if let Some(tool) = tools.get(tool_name) {
            let input = AgentInput::text(parameters.to_string());
            tool.execute(input, context).await
        } else {
            Err(LLMSpellError::Component {
                message: format!("Tool not found: {tool_name}"),
                source: None,
            })
        }
    }

    async fn list_available_tools(&self) -> Result<Vec<String>> {
        let tools = self.tools.read().await;
        Ok(tools.keys().cloned().collect())
    }

    async fn tool_available(&self, tool_name: &str) -> bool {
        let tools = self.tools.read().await;
        tools.contains_key(tool_name)
    }
}

#[async_trait]
impl Composable for HierarchicalCompositeAgent {
    fn composition_metadata(&self) -> CompositionMetadata {
        CompositionMetadata {
            composition_type: CompositionType::Hierarchical,
            version: "1.0.0".to_string(),
            max_components: self.config.max_children,
            can_be_parent: true,
            can_be_child: true,
            custom: HashMap::new(),
        }
    }

    fn can_compose_with(&self, other: &dyn Composable) -> bool {
        let other_meta = other.composition_metadata();
        matches!(other_meta.composition_type, CompositionType::Hierarchical)
            && other_meta.can_be_child
    }

    fn exposed_capabilities(&self) -> Vec<Capability> {
        self.capabilities.read().unwrap().clone()
    }

    fn required_capabilities(&self) -> Vec<Capability> {
        Vec::new() // Hierarchical agents typically don't require specific capabilities
    }
}

#[async_trait]
impl CompositeAgent for HierarchicalCompositeAgent {
    async fn add_component(&mut self, component: Arc<dyn BaseAgent>) -> Result<()> {
        let mut components = self.components.write().await;
        components.insert(component.metadata().id.to_string(), component);
        Ok(())
    }

    async fn remove_component(&mut self, component_id: &str) -> Result<()> {
        let mut components = self.components.write().await;
        components
            .remove(component_id)
            .ok_or_else(|| LLMSpellError::Component {
                message: format!("Component not found: {component_id}"),
                source: None,
            })?;
        Ok(())
    }

    fn components(&self) -> Vec<Arc<dyn BaseAgent>> {
        // This would need to be async in real implementation
        Vec::new()
    }

    fn get_component(&self, _component_id: &str) -> Option<Arc<dyn BaseAgent>> {
        // This would need to be async in real implementation
        None
    }

    async fn delegate_to(
        &self,
        component_id: &str,
        input: Value,
        context: &ExecutionContext,
    ) -> Result<Value> {
        // Update metrics before async operation
        {
            let mut metrics = self.metrics.write().unwrap();
            metrics.total_delegations += 1;
        }

        let components = self.components.read().await;

        if let Some(component) = components.get(component_id) {
            let agent_input = AgentInput::text(input.to_string());
            match component.execute(agent_input, context.clone()).await {
                Ok(result) => {
                    {
                        let mut metrics = self.metrics.write().unwrap();
                        metrics.successful_delegations += 1;
                    }
                    Ok(serde_json::to_value(&result)?)
                }
                Err(e) => {
                    {
                        let mut metrics = self.metrics.write().unwrap();
                        metrics.failed_delegations += 1;
                    }
                    Err(e)
                }
            }
        } else {
            {
                let mut metrics = self.metrics.write().unwrap();
                metrics.failed_delegations += 1;
            }
            Err(CompositionError::ComponentNotFound(component_id.to_string()).into())
        }
    }

    async fn execute_pattern(
        &self,
        pattern: ExecutionPattern,
        input: Value,
        context: &ExecutionContext,
    ) -> Result<Value> {
        match pattern {
            ExecutionPattern::Sequential => {
                let mut result = input;
                let components = self.components.read().await;

                for (_, component) in components.iter() {
                    let agent_input = AgentInput::text(result.to_string());
                    let output = component.execute(agent_input, context.clone()).await?;
                    result = serde_json::to_value(&output)?;
                }

                Ok(result)
            }
            ExecutionPattern::Parallel => {
                let components = self.components.read().await;
                let mut handles = Vec::new();

                for (_, component) in components.iter() {
                    let comp = component.clone();
                    let inp = input.clone();
                    let ctx = context.clone();

                    handles.push(tokio::spawn(async move {
                        let agent_input = AgentInput::text(inp.to_string());
                        let output = comp.execute(agent_input, ctx).await?;
                        Ok::<Value, LLMSpellError>(serde_json::to_value(&output)?)
                    }));
                }

                let mut results = Vec::new();
                for handle in handles {
                    results.push(handle.await.map_err(|e| LLMSpellError::Component {
                        message: format!("Parallel execution failed: {e}"),
                        source: None,
                    })??);
                }

                Ok(Value::Array(results))
            }
            _ => Err(LLMSpellError::Component {
                message: "Unsupported execution pattern".to_string(),
                source: None,
            }),
        }
    }
}

#[async_trait]
impl HierarchicalAgent for HierarchicalCompositeAgent {
    fn parent(&self) -> Option<Arc<dyn HierarchicalAgent>> {
        let parent_guard = self.parent.read().unwrap();
        parent_guard.as_ref().and_then(std::sync::Weak::upgrade)
    }

    fn children(&self) -> Vec<Arc<dyn HierarchicalAgent>> {
        // This would need to be async in real implementation
        Vec::new()
    }

    async fn add_child(&mut self, child: Arc<dyn HierarchicalAgent>) -> Result<()> {
        // Check max children limit
        if let Some(max) = self.config.max_children {
            let children = self.children.read().await;
            if children.len() >= max {
                return Err(LLMSpellError::Component {
                    message: format!("Maximum children limit ({max}) reached"),
                    source: None,
                });
            }
        }

        // Check for cycles
        if self.would_create_cycle(&child).await {
            return Err(CompositionError::CycleDetected.into());
        }

        // Check depth limit
        if self.depth() + 1 > self.config.max_depth {
            return Err(CompositionError::MaxDepthExceeded(self.config.max_depth).into());
        }

        let mut children = self.children.write().await;
        children.push(child);
        Ok(())
    }

    async fn remove_child(&mut self, _child_id: &str) -> Result<()> {
        let mut children = self.children.write().await;
        // Remove child by comparing against child_id
        // Since HierarchicalAgent doesn't have id() method, we need a different approach
        // For now, we'll remove all children - this is a limitation
        children.clear();
        // TODO: Implement proper child identification
        Ok(())
    }

    fn depth(&self) -> usize {
        self.calculate_depth()
    }

    async fn propagate_down(&self, event: HierarchyEvent) -> Result<()> {
        if !self.config.propagate_down {
            return Ok(());
        }

        // Update metrics before async operation
        {
            let mut metrics = self.metrics.write().unwrap();
            metrics.events_propagated_down += 1;
        }

        let children = self.children.read().await;
        for child in children.iter() {
            child.propagate_down(event.clone()).await?;
        }
        Ok(())
    }

    async fn propagate_up(&self, event: HierarchyEvent) -> Result<()> {
        if !self.config.propagate_up {
            return Ok(());
        }

        // Update metrics before async operation
        {
            let mut metrics = self.metrics.write().unwrap();
            metrics.events_propagated_up += 1;
        }

        if let Some(parent) = self.parent() {
            parent.propagate_up(event).await?;
        }
        Ok(())
    }
}

/// Builder for hierarchical composite agents
pub struct HierarchicalAgentBuilder {
    name: String,
    description: Option<String>,
    config: HierarchicalConfig,
    capabilities: Vec<Capability>,
    initial_pattern: ExecutionPattern,
}

impl HierarchicalAgentBuilder {
    /// Create a new builder
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            description: None,
            config: HierarchicalConfig::default(),
            capabilities: Vec::new(),
            initial_pattern: ExecutionPattern::Sequential,
        }
    }

    /// Set the description
    pub fn description(mut self, desc: impl Into<String>) -> Self {
        self.description = Some(desc.into());
        self
    }

    /// Set the configuration
    #[must_use]
    pub const fn config(mut self, config: HierarchicalConfig) -> Self {
        self.config = config;
        self
    }

    /// Add a capability
    #[must_use]
    pub fn add_capability(mut self, capability: Capability) -> Self {
        self.capabilities.push(capability);
        self
    }

    /// Set the initial execution pattern
    #[must_use]
    pub fn execution_pattern(mut self, pattern: ExecutionPattern) -> Self {
        self.initial_pattern = pattern;
        self
    }

    /// Build the hierarchical agent
    #[must_use]
    pub fn build(self) -> HierarchicalCompositeAgent {
        let name = self.name.clone();
        let mut agent = HierarchicalCompositeAgent::new(self.name, self.config);

        if let Some(desc) = self.description {
            agent.metadata = ComponentMetadata::new(name, desc);
        }

        *agent.execution_pattern.write().unwrap() = self.initial_pattern;
        *agent.capabilities.write().unwrap() = self.capabilities;

        agent
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use llmspell_core::types::{AgentInput, AgentOutput};
    use llmspell_core::{ComponentMetadata, LLMSpellError};
    #[tokio::test]
    async fn test_hierarchical_agent_creation() {
        let agent = HierarchicalAgentBuilder::new("test-hierarchical")
            .description("Test hierarchical agent")
            .build();

        assert_eq!(agent.metadata().name, "test-hierarchical");
        assert_eq!(agent.depth(), 0);
    }
    #[tokio::test]
    async fn test_component_management() {
        let mut agent = HierarchicalCompositeAgent::new("parent", HierarchicalConfig::default());

        // Create a mock component
        struct MockAgent {
            metadata: ComponentMetadata,
        }

        #[async_trait]
        impl BaseAgent for MockAgent {
            fn metadata(&self) -> &ComponentMetadata {
                &self.metadata
            }

            async fn execute(
                &self,
                input: AgentInput,
                _context: ExecutionContext,
            ) -> Result<AgentOutput> {
                Ok(AgentOutput::text(format!("Processed: {}", input.text)))
            }

            async fn validate_input(&self, _input: &AgentInput) -> Result<()> {
                Ok(())
            }

            async fn handle_error(&self, error: LLMSpellError) -> Result<AgentOutput> {
                Ok(AgentOutput::text(format!("Error: {}", error)))
            }
        }

        let mock = Arc::new(MockAgent {
            metadata: ComponentMetadata::new(
                "mock-1".to_string(),
                "Mock agent for testing".to_string(),
            ),
        });

        agent.add_component(mock.clone()).await.unwrap();

        // Check that component was added by verifying the component count
        let components = agent.components.read().await;
        assert_eq!(components.len(), 1);
    }
    #[tokio::test]
    async fn test_capability_aggregation() {
        let agent = HierarchicalAgentBuilder::new("capable-agent")
            .add_capability(Capability {
                name: "text-processing".to_string(),
                category: CapabilityCategory::DataProcessing,
                version: None,
                metadata: HashMap::new(),
            })
            .add_capability(Capability {
                name: "monitoring".to_string(),
                category: CapabilityCategory::Monitoring,
                version: None,
                metadata: HashMap::new(),
            })
            .build();

        let capabilities = agent.exposed_capabilities();
        assert_eq!(capabilities.len(), 2);
        assert!(capabilities.iter().any(|c| c.name == "text-processing"));
        assert!(capabilities.iter().any(|c| c.name == "monitoring"));
    }
}
