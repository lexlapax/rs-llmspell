//! ABOUTME: Core traits and interfaces for agent composition patterns
//! ABOUTME: Defines how agents can be composed into higher-level agents

use async_trait::async_trait;
use llmspell_core::{BaseAgent, ExecutionContext, LLMSpellError, Result, ToolCapable};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;

/// Trait for components that can be composed into higher-level structures
#[async_trait]
pub trait Composable: BaseAgent + Send + Sync {
    /// Get the composition metadata for this component
    fn composition_metadata(&self) -> CompositionMetadata;

    /// Check if this component can be composed with another
    fn can_compose_with(&self, other: &dyn Composable) -> bool;

    /// Get the exposed capabilities of this component
    fn exposed_capabilities(&self) -> Vec<Capability>;

    /// Get the required capabilities this component needs
    fn required_capabilities(&self) -> Vec<Capability>;
}

/// Trait for agents that compose other agents
#[async_trait]
pub trait CompositeAgent: BaseAgent + ToolCapable + Send + Sync {
    /// Add a child agent to this composite
    async fn add_component(&mut self, component: Arc<dyn BaseAgent>) -> Result<()>;

    /// Remove a child agent by ID
    async fn remove_component(&mut self, component_id: &str) -> Result<()>;

    /// Get all child components
    fn components(&self) -> Vec<Arc<dyn BaseAgent>>;

    /// Get a specific component by ID
    fn get_component(&self, component_id: &str) -> Option<Arc<dyn BaseAgent>>;

    /// Delegate execution to a specific component
    async fn delegate_to(
        &self,
        component_id: &str,
        input: Value,
        context: &ExecutionContext,
    ) -> Result<Value>;

    /// Execute all components in a specific pattern
    async fn execute_pattern(
        &self,
        pattern: ExecutionPattern,
        input: Value,
        context: &ExecutionContext,
    ) -> Result<Value>;
}

/// Trait for hierarchical agent composition
#[async_trait]
pub trait HierarchicalAgent: CompositeAgent {
    /// Get the parent agent if this is a child
    fn parent(&self) -> Option<Arc<dyn HierarchicalAgent>>;

    /// Get all child agents
    fn children(&self) -> Vec<Arc<dyn HierarchicalAgent>>;

    /// Add a child agent
    async fn add_child(&mut self, child: Arc<dyn HierarchicalAgent>) -> Result<()>;

    /// Remove a child agent
    async fn remove_child(&mut self, child_id: &str) -> Result<()>;

    /// Get the depth of this agent in the hierarchy
    fn depth(&self) -> usize;

    /// Propagate an event down the hierarchy
    async fn propagate_down(&self, event: HierarchyEvent) -> Result<()>;

    /// Propagate an event up the hierarchy
    async fn propagate_up(&self, event: HierarchyEvent) -> Result<()>;
}

/// Metadata about a composable component
#[derive(Debug, Clone)]
pub struct CompositionMetadata {
    /// Type of composition this component supports
    pub composition_type: CompositionType,
    /// Version of the composition interface
    pub version: String,
    /// Maximum number of components this can compose
    pub max_components: Option<usize>,
    /// Whether this component can be a parent
    pub can_be_parent: bool,
    /// Whether this component can be a child
    pub can_be_child: bool,
    /// Custom metadata
    pub custom: HashMap<String, Value>,
}

/// Types of composition patterns
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CompositionType {
    /// Hierarchical parent-child relationships
    Hierarchical,
    /// Peer-to-peer delegation
    Delegation,
    /// Pipeline composition
    Pipeline,
    /// Ensemble composition
    Ensemble,
    /// Custom composition type
    Custom(String),
}

/// Execution patterns for composite agents
#[derive(Debug, Clone)]
pub enum ExecutionPattern {
    /// Execute components sequentially
    Sequential,
    /// Execute components in parallel
    Parallel,
    /// Execute based on conditions
    Conditional(Vec<ExecutionCondition>),
    /// Execute in a round-robin fashion
    RoundRobin,
    /// Execute based on capability matching
    CapabilityBased(Vec<Capability>),
    /// Custom execution pattern
    Custom(String),
}

/// Condition for conditional execution
#[derive(Debug, Clone)]
pub struct ExecutionCondition {
    /// Component ID to check
    pub component_id: String,
    /// Field to evaluate
    pub field: String,
    /// Expected value
    pub expected: Value,
    /// Component to execute if true
    pub then_component: String,
    /// Component to execute if false (optional)
    pub else_component: Option<String>,
}

/// Represents a capability that an agent exposes or requires
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Capability {
    /// Name of the capability
    pub name: String,
    /// Category of the capability
    pub category: CapabilityCategory,
    /// Version requirement
    pub version: Option<String>,
    /// Additional metadata
    pub metadata: HashMap<String, String>,
}

/// Categories of capabilities
#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum CapabilityCategory {
    /// Tool-related capabilities
    Tool(String),
    /// LLM-related capabilities
    Model(String),
    /// Data processing capabilities
    DataProcessing,
    /// Orchestration capabilities
    Orchestration,
    /// Monitoring capabilities
    Monitoring,
    /// Custom capability
    Custom(String),
}

/// Event types for hierarchical agents
#[derive(Debug, Clone)]
pub enum HierarchyEvent {
    /// Configuration change event
    ConfigurationChange(HashMap<String, Value>),
    /// State change event
    StateChange {
        old_state: String,
        new_state: String,
    },
    /// Error event
    Error(String),
    /// Custom event
    Custom { event_type: String, data: Value },
}

/// Builder for creating composite agents
pub struct CompositeAgentBuilder {
    #[allow(dead_code)]
    name: String,
    description: String,
    components: Vec<Arc<dyn BaseAgent>>,
    execution_pattern: ExecutionPattern,
    metadata: HashMap<String, Value>,
}

impl CompositeAgentBuilder {
    /// Create a new composite agent builder
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            description: String::new(),
            components: Vec::new(),
            execution_pattern: ExecutionPattern::Sequential,
            metadata: HashMap::new(),
        }
    }

    /// Set the description
    pub fn description(mut self, desc: impl Into<String>) -> Self {
        self.description = desc.into();
        self
    }

    /// Add a component
    pub fn add_component(mut self, component: Arc<dyn BaseAgent>) -> Self {
        self.components.push(component);
        self
    }

    /// Set the execution pattern
    #[must_use]
    pub fn execution_pattern(mut self, pattern: ExecutionPattern) -> Self {
        self.execution_pattern = pattern;
        self
    }

    /// Add metadata
    pub fn metadata(mut self, key: impl Into<String>, value: Value) -> Self {
        self.metadata.insert(key.into(), value);
        self
    }

    /// Build the composite agent (to be implemented by concrete types)
    /// 
    /// # Errors
    /// 
    /// Returns an error if:
    /// - The build method is called on the base builder (must be implemented by concrete types)
    /// - Component validation fails
    /// - Agent construction fails due to configuration issues
    pub fn build<T: CompositeAgent>(self) -> Result<T> {
        // This will be implemented by concrete composite agent types
        Err(LLMSpellError::Component {
            message: "Build method must be implemented by concrete types".to_string(),
            source: None,
        })
    }
}

/// Error types specific to composition
#[derive(Debug, thiserror::Error)]
pub enum CompositionError {
    /// Component not found
    #[error("Component not found: {0}")]
    ComponentNotFound(String),

    /// Invalid composition
    #[error("Invalid composition: {0}")]
    InvalidComposition(String),

    /// Capability mismatch
    #[error("Capability mismatch: required {required:?}, provided {provided:?}")]
    CapabilityMismatch {
        required: Vec<Capability>,
        provided: Vec<Capability>,
    },

    /// Cycle detected in hierarchy
    #[error("Cycle detected in hierarchy")]
    CycleDetected,

    /// Maximum depth exceeded
    #[error("Maximum hierarchy depth exceeded: {0}")]
    MaxDepthExceeded(usize),

    /// Delegation failed
    #[error("Delegation failed to component {0}: {1}")]
    DelegationFailed(String, String),
}

impl From<CompositionError> for LLMSpellError {
    fn from(err: CompositionError) -> Self {
        Self::Component {
            message: err.to_string(),
            source: Some(Box::new(err)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_capability_creation() {
        let cap = Capability {
            name: "text-processing".to_string(),
            category: CapabilityCategory::DataProcessing,
            version: Some("1.0.0".to_string()),
            metadata: HashMap::new(),
        };

        assert_eq!(cap.name, "text-processing");
        assert_eq!(cap.category, CapabilityCategory::DataProcessing);
    }
    #[test]
    fn test_composition_metadata() {
        let mut metadata = CompositionMetadata {
            composition_type: CompositionType::Hierarchical,
            version: "1.0.0".to_string(),
            max_components: Some(10),
            can_be_parent: true,
            can_be_child: false,
            custom: HashMap::new(),
        };

        metadata
            .custom
            .insert("test".to_string(), Value::Bool(true));
        assert_eq!(metadata.composition_type, CompositionType::Hierarchical);
        assert_eq!(metadata.max_components, Some(10));
    }
    #[test]
    fn test_execution_pattern() {
        let condition = ExecutionCondition {
            component_id: "comp1".to_string(),
            field: "status".to_string(),
            expected: Value::String("ready".to_string()),
            then_component: "comp2".to_string(),
            else_component: Some("comp3".to_string()),
        };

        let pattern = ExecutionPattern::Conditional(vec![condition]);
        match pattern {
            ExecutionPattern::Conditional(conditions) => {
                assert_eq!(conditions.len(), 1);
                assert_eq!(conditions[0].component_id, "comp1");
            }
            _ => panic!("Expected conditional pattern"),
        }
    }
    #[test]
    fn test_composite_agent_builder() {
        let builder = CompositeAgentBuilder::new("test-composite")
            .description("A test composite agent")
            .execution_pattern(ExecutionPattern::Parallel)
            .metadata("version", Value::String("1.0.0".to_string()));

        assert_eq!(builder.name, "test-composite");
        assert_eq!(builder.description, "A test composite agent");
        assert!(matches!(
            builder.execution_pattern,
            ExecutionPattern::Parallel
        ));
    }
}
