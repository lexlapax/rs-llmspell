//! ABOUTME: ToolCapable trait for components that can interact with tools
//! ABOUTME: Separate trait to maintain clean architecture and avoid trait cyclicity

use super::base_agent::BaseAgent;
use crate::execution_context::ExecutionContext;
use crate::types::AgentOutput;
use crate::{LLMSpellError, Result};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use std::collections::HashMap;

/// Tool query criteria for tool discovery.
///
/// Specifies the criteria for finding tools that match specific requirements.
/// This is used by components to discover suitable tools without direct
/// dependency on the tool registry.
///
/// # Examples
///
/// ```
/// use llmspell_core::traits::tool_capable::ToolQuery;
///
/// let query = ToolQuery::new()
///     .with_category("filesystem")
///     .with_capability("file_search")
///     .with_max_security_level("safe");
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct ToolQuery {
    /// Filter by tool categories
    pub categories: Vec<String>,
    /// Filter by required capabilities
    pub capabilities: Vec<String>,
    /// Maximum security level allowed
    pub max_security_level: Option<String>,
    /// Minimum security level required
    pub min_security_level: Option<String>,
    /// Text search in tool names/descriptions
    pub text_search: Option<String>,
    /// Custom query parameters
    pub custom_filters: HashMap<String, JsonValue>,
}

impl ToolQuery {
    /// Create a new empty tool query
    pub fn new() -> Self {
        Self::default()
    }

    /// Filter by tool category
    pub fn with_category(mut self, category: impl Into<String>) -> Self {
        self.categories.push(category.into());
        self
    }

    /// Filter by required capability
    pub fn with_capability(mut self, capability: impl Into<String>) -> Self {
        self.capabilities.push(capability.into());
        self
    }

    /// Set maximum security level
    pub fn with_max_security_level(mut self, level: impl Into<String>) -> Self {
        self.max_security_level = Some(level.into());
        self
    }

    /// Set minimum security level
    pub fn with_min_security_level(mut self, level: impl Into<String>) -> Self {
        self.min_security_level = Some(level.into());
        self
    }

    /// Add text search filter
    pub fn with_text_search(mut self, text: impl Into<String>) -> Self {
        self.text_search = Some(text.into());
        self
    }

    /// Add custom filter
    pub fn with_custom_filter(mut self, key: impl Into<String>, value: JsonValue) -> Self {
        self.custom_filters.insert(key.into(), value);
        self
    }
}

/// Information about a tool that can be invoked.
///
/// Contains metadata about a tool including its name, description, schema,
/// and requirements. This allows components to understand tool capabilities
/// before invocation.
///
/// # Examples
///
/// ```
/// use llmspell_core::traits::tool_capable::ToolInfo;
/// use serde_json::json;
///
/// let info = ToolInfo {
///     name: "file_search".to_string(),
///     description: "Search for files by pattern".to_string(),
///     category: "filesystem".to_string(),
///     security_level: "safe".to_string(),
///     schema: json!({
///         "type": "object",
///         "properties": {
///             "pattern": {"type": "string"},
///             "recursive": {"type": "boolean"}
///         }
///     }),
///     capabilities: vec!["file_operations".to_string()],
///     requirements: json!({}),
/// };
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ToolInfo {
    /// Tool name/identifier
    pub name: String,
    /// Tool description
    pub description: String,
    /// Tool category
    pub category: String,
    /// Security level required
    pub security_level: String,
    /// Parameter schema (JSON Schema format)
    pub schema: JsonValue,
    /// Tool capabilities
    pub capabilities: Vec<String>,
    /// Additional requirements
    pub requirements: JsonValue,
}

impl ToolInfo {
    /// Create a new ToolInfo
    pub fn new(
        name: impl Into<String>,
        description: impl Into<String>,
        category: impl Into<String>,
        security_level: impl Into<String>,
    ) -> Self {
        Self {
            name: name.into(),
            description: description.into(),
            category: category.into(),
            security_level: security_level.into(),
            schema: JsonValue::Object(serde_json::Map::new()),
            capabilities: Vec::new(),
            requirements: JsonValue::Object(serde_json::Map::new()),
        }
    }

    /// Set the parameter schema
    pub fn with_schema(mut self, schema: JsonValue) -> Self {
        self.schema = schema;
        self
    }

    /// Add a capability
    pub fn with_capability(mut self, capability: impl Into<String>) -> Self {
        self.capabilities.push(capability.into());
        self
    }

    /// Set requirements
    pub fn with_requirements(mut self, requirements: JsonValue) -> Self {
        self.requirements = requirements;
        self
    }
}

/// Tool composition step for executing multiple tools in sequence.
///
/// Defines how tools should be composed together, including data flow
/// between tools and error handling strategies.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ToolCompositionStep {
    /// Tool name to invoke
    pub tool_name: String,
    /// Parameters for the tool (can reference outputs from previous steps)
    pub parameters: JsonValue,
    /// How to handle errors from this step
    pub error_strategy: ErrorStrategy,
    /// Whether to pass the entire context or just specific values
    pub context_mode: ContextMode,
}

/// Error handling strategy for tool composition steps.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ErrorStrategy {
    /// Stop the entire composition on error
    Fail,
    /// Continue with the next step, using a default value
    Continue,
    /// Retry the step up to N times
    Retry(u32),
    /// Skip this step and continue
    Skip,
}

/// Context passing mode for tool composition.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ContextMode {
    /// Pass the full execution context
    Full,
    /// Pass only output from the previous step
    Previous,
    /// Pass specific fields from context/previous outputs
    Selective(Vec<String>),
}

/// Tool composition definition.
///
/// Defines a sequence of tool invocations that should be executed as a workflow.
/// Supports data flow between tools, error handling, and various execution modes.
///
/// # Examples
///
/// ```
/// use llmspell_core::traits::tool_capable::{ToolComposition, ToolCompositionStep, ErrorStrategy, ContextMode};
/// use serde_json::json;
///
/// let composition = ToolComposition {
///     name: "file_analysis".to_string(),
///     description: "Search and analyze files".to_string(),
///     steps: vec![
///         ToolCompositionStep {
///             tool_name: "file_search".to_string(),
///             parameters: json!({"pattern": "*.txt"}),
///             error_strategy: ErrorStrategy::Fail,
///             context_mode: ContextMode::Full,
///         },
///         ToolCompositionStep {
///             tool_name: "text_analyzer".to_string(),
///             parameters: json!({"files": "${previous.output}"}),
///             error_strategy: ErrorStrategy::Continue,
///             context_mode: ContextMode::Previous,
///         },
///     ],
///     parallel: false,
/// };
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ToolComposition {
    /// Composition name
    pub name: String,
    /// Composition description
    pub description: String,
    /// Steps to execute
    pub steps: Vec<ToolCompositionStep>,
    /// Whether to execute steps in parallel (when possible)
    pub parallel: bool,
}

impl ToolComposition {
    /// Create a new tool composition
    pub fn new(name: impl Into<String>, description: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            description: description.into(),
            steps: Vec::new(),
            parallel: false,
        }
    }

    /// Add a composition step
    pub fn with_step(mut self, step: ToolCompositionStep) -> Self {
        self.steps.push(step);
        self
    }

    /// Enable parallel execution
    pub fn with_parallel(mut self, parallel: bool) -> Self {
        self.parallel = parallel;
        self
    }
}

impl Default for ToolComposition {
    fn default() -> Self {
        Self::new("unnamed_composition", "Tool composition")
    }
}

/// Tool-capable trait for components that can interact with tools.
///
/// This trait extends `BaseAgent` to provide tool integration capabilities.
/// Components can implement this trait to gain access to tool discovery,
/// invocation, and composition features while keeping the base trait clean.
///
/// # Design Philosophy
///
/// - **Separation of Concerns**: Tool integration is a specialized capability
/// - **Optional Feature**: Components opt-in by implementing this trait
/// - **Clean Architecture**: Avoids polluting BaseAgent with specialized methods
/// - **No Cyclicity**: Tool: BaseAgent, ToolCapable: BaseAgent (no cycles)
///
/// # Examples
///
/// ```
/// use llmspell_core::{
///     ComponentMetadata, Result, ExecutionContext,
///     traits::{
///         base_agent::BaseAgent,
///         tool_capable::{ToolCapable, ToolQuery, ToolInfo, ToolComposition}
///     },
///     types::{AgentInput, AgentOutput}
/// };
/// use async_trait::async_trait;
/// use serde_json::json;
///
/// struct ToolAwareAgent {
///     metadata: ComponentMetadata,
///     // tool manager would be injected here
/// }
///
/// #[async_trait]
/// impl BaseAgent for ToolAwareAgent {
///     fn metadata(&self) -> &ComponentMetadata {
///         &self.metadata
///     }
///     
///     async fn execute(&self, input: AgentInput, context: ExecutionContext) -> Result<AgentOutput> {
///         // Can use tool capabilities here
///         let tools = self.list_available_tools().await?;
///         if tools.contains(&"file_search".to_string()) {
///             let result = self.invoke_tool(
///                 "file_search",
///                 json!({"pattern": "*.txt"}),
///                 context
///             ).await?;
///             return Ok(result);
///         }
///         Ok(AgentOutput::text("No tools available".to_string()))
///     }
///     
///     async fn validate_input(&self, _input: &AgentInput) -> Result<()> {
///         Ok(())
///     }
///     
///     async fn handle_error(&self, error: llmspell_core::LLMSpellError) -> Result<AgentOutput> {
///         Ok(AgentOutput::text(format!("Error: {}", error)))
///     }
/// }
///
/// #[async_trait]
/// impl ToolCapable for ToolAwareAgent {
///     async fn discover_tools(&self, query: &ToolQuery) -> Result<Vec<ToolInfo>> {
///         // Implementation would delegate to ToolManager
///         Ok(Vec::new())
///     }
///     
///     async fn invoke_tool(&self, tool_name: &str, parameters: serde_json::Value, context: ExecutionContext) -> Result<AgentOutput> {
///         // Implementation would delegate to ToolManager
///         Ok(AgentOutput::text(format!("Invoked {}", tool_name)))
///     }
///     
///     async fn list_available_tools(&self) -> Result<Vec<String>> {
///         Ok(vec!["file_search".to_string(), "text_processor".to_string()])
///     }
///     
///     async fn tool_available(&self, tool_name: &str) -> bool {
///         tool_name == "file_search" || tool_name == "text_processor"
///     }
///     
///     async fn get_tool_info(&self, tool_name: &str) -> Result<Option<ToolInfo>> {
///         if self.tool_available(tool_name).await {
///             Ok(Some(ToolInfo::new(tool_name, "Tool description", "utility", "safe")))
///         } else {
///             Ok(None)
///         }
///     }
///     
///     async fn compose_tools(&self, composition: &ToolComposition, context: ExecutionContext) -> Result<AgentOutput> {
///         // Implementation would execute composition steps
///         Ok(AgentOutput::text(format!("Executed composition: {}", composition.name)))
///     }
/// }
/// ```
#[async_trait]
pub trait ToolCapable: BaseAgent {
    /// Discover available tools based on query criteria.
    ///
    /// Returns information about tools that match the given criteria. This allows
    /// components to find suitable tools for their needs without having direct
    /// dependencies on the tool registry.
    ///
    /// # Arguments
    ///
    /// * `query` - Search criteria for tool discovery
    ///
    /// # Returns
    ///
    /// Returns a vector of tool information structures, or an error if discovery fails.
    ///
    /// # Default Implementation
    ///
    /// Returns an empty vector by default, indicating no tool discovery capability.
    async fn discover_tools(&self, _query: &ToolQuery) -> Result<Vec<ToolInfo>> {
        Ok(Vec::new())
    }

    /// Invoke a tool by name with given parameters.
    ///
    /// Provides a standardized way for components to invoke tools. The implementation
    /// should handle tool discovery, parameter validation, execution, and result conversion.
    ///
    /// # Arguments
    ///
    /// * `tool_name` - Name or identifier of the tool to invoke
    /// * `parameters` - Parameters to pass to the tool (JSON object)
    /// * `context` - Execution context to pass to the tool
    ///
    /// # Returns
    ///
    /// Returns the tool's output wrapped in an `AgentOutput`, or an error if invocation fails.
    ///
    /// # Default Implementation
    ///
    /// Returns a NotImplemented error by default.
    async fn invoke_tool(
        &self,
        _tool_name: &str,
        _parameters: JsonValue,
        _context: ExecutionContext,
    ) -> Result<AgentOutput> {
        Err(LLMSpellError::Component {
            message: "Tool invocation not supported by this component".to_string(),
            source: None,
        })
    }

    /// List all available tools that this component can access.
    ///
    /// Returns the names/identifiers of all tools that can be invoked by this component.
    /// This is useful for introspection and building tool-aware interfaces.
    ///
    /// # Returns
    ///
    /// Returns a vector of tool names, or an error if listing fails.
    ///
    /// # Default Implementation
    ///
    /// Returns an empty vector by default.
    async fn list_available_tools(&self) -> Result<Vec<String>> {
        Ok(Vec::new())
    }

    /// Check if a specific tool is available for invocation.
    ///
    /// Allows components to check tool availability before attempting invocation.
    /// This can be more efficient than catching invocation errors.
    ///
    /// # Arguments
    ///
    /// * `tool_name` - Name or identifier of the tool to check
    ///
    /// # Returns
    ///
    /// Returns `true` if the tool is available, `false` otherwise.
    ///
    /// # Default Implementation
    ///
    /// Returns `false` by default.
    async fn tool_available(&self, _tool_name: &str) -> bool {
        false
    }

    /// Get information about a specific tool.
    ///
    /// Returns detailed information about a tool including its schema, capabilities,
    /// and requirements. This enables components to understand tool interfaces
    /// before invocation.
    ///
    /// # Arguments
    ///
    /// * `tool_name` - Name or identifier of the tool
    ///
    /// # Returns
    ///
    /// Returns tool information if found, or `None` if the tool is not available.
    ///
    /// # Default Implementation
    ///
    /// Returns `None` by default.
    async fn get_tool_info(&self, _tool_name: &str) -> Result<Option<ToolInfo>> {
        Ok(None)
    }

    /// Compose multiple tools into a workflow.
    ///
    /// Executes a sequence of tool invocations where the output of one tool
    /// can be used as input to subsequent tools. This enables complex operations
    /// through tool composition.
    ///
    /// # Arguments
    ///
    /// * `composition` - Description of how tools should be composed
    /// * `context` - Execution context for the composition
    ///
    /// # Returns
    ///
    /// Returns the final output after all tools have been executed, or an error
    /// if any step in the composition fails.
    ///
    /// # Default Implementation
    ///
    /// Returns a NotImplemented error by default.
    async fn compose_tools(
        &self,
        _composition: &ToolComposition,
        _context: ExecutionContext,
    ) -> Result<AgentOutput> {
        Err(LLMSpellError::Component {
            message: "Tool composition not supported by this component".to_string(),
            source: None,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{AgentInput, AgentOutput};
    use crate::ComponentMetadata;
    use crate::ExecutionContext;
    use serde_json::json;

    // Mock implementation for testing
    struct MockToolCapableAgent {
        metadata: ComponentMetadata,
    }

    impl MockToolCapableAgent {
        fn new() -> Self {
            Self {
                metadata: ComponentMetadata::new(
                    "mock-tool-capable".to_string(),
                    "A mock tool-capable agent for testing".to_string(),
                ),
            }
        }
    }

    #[async_trait]
    impl BaseAgent for MockToolCapableAgent {
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

        async fn validate_input(&self, input: &AgentInput) -> Result<()> {
            if input.text.is_empty() {
                return Err(LLMSpellError::Validation {
                    message: "Text cannot be empty".to_string(),
                    field: Some("text".to_string()),
                });
            }
            Ok(())
        }

        async fn handle_error(&self, error: LLMSpellError) -> Result<AgentOutput> {
            Ok(AgentOutput::text(format!("Error handled: {}", error)))
        }
    }

    #[async_trait]
    impl ToolCapable for MockToolCapableAgent {
        async fn discover_tools(&self, query: &ToolQuery) -> Result<Vec<ToolInfo>> {
            let mut tools = Vec::new();

            if query.categories.is_empty() || query.categories.contains(&"utility".to_string()) {
                tools.push(ToolInfo::new("mock_tool", "Mock tool", "utility", "safe"));
            }

            Ok(tools)
        }

        async fn invoke_tool(
            &self,
            tool_name: &str,
            parameters: JsonValue,
            _context: ExecutionContext,
        ) -> Result<AgentOutput> {
            Ok(AgentOutput::text(format!(
                "Invoked {} with params: {}",
                tool_name, parameters
            )))
        }

        async fn list_available_tools(&self) -> Result<Vec<String>> {
            Ok(vec!["mock_tool".to_string(), "test_tool".to_string()])
        }

        async fn tool_available(&self, tool_name: &str) -> bool {
            tool_name == "mock_tool" || tool_name == "test_tool"
        }

        async fn get_tool_info(&self, tool_name: &str) -> Result<Option<ToolInfo>> {
            if self.tool_available(tool_name).await {
                Ok(Some(ToolInfo::new(
                    tool_name,
                    "Test tool description",
                    "utility",
                    "safe",
                )))
            } else {
                Ok(None)
            }
        }

        async fn compose_tools(
            &self,
            composition: &ToolComposition,
            _context: ExecutionContext,
        ) -> Result<AgentOutput> {
            let steps = composition.steps.len();
            Ok(AgentOutput::text(format!(
                "Executed composition '{}' with {} steps",
                composition.name, steps
            )))
        }
    }
    #[test]
    fn test_tool_query_builder() {
        let query = ToolQuery::new()
            .with_category("filesystem")
            .with_capability("file_search")
            .with_max_security_level("safe")
            .with_text_search("search");

        assert_eq!(query.categories, vec!["filesystem"]);
        assert_eq!(query.capabilities, vec!["file_search"]);
        assert_eq!(query.max_security_level, Some("safe".to_string()));
        assert_eq!(query.text_search, Some("search".to_string()));
    }
    #[test]
    fn test_tool_info_builder() {
        let info = ToolInfo::new("test_tool", "Test description", "utility", "safe")
            .with_capability("testing")
            .with_schema(json!({"type": "object"}))
            .with_requirements(json!({"mem": "10MB"}));

        assert_eq!(info.name, "test_tool");
        assert_eq!(info.description, "Test description");
        assert_eq!(info.category, "utility");
        assert_eq!(info.security_level, "safe");
        assert_eq!(info.capabilities, vec!["testing"]);
        assert_eq!(info.schema, json!({"type": "object"}));
        assert_eq!(info.requirements, json!({"mem": "10MB"}));
    }
    #[test]
    fn test_tool_composition_builder() {
        let composition = ToolComposition::new("test_workflow", "Test composition")
            .with_step(ToolCompositionStep {
                tool_name: "tool1".to_string(),
                parameters: json!({"input": "data"}),
                error_strategy: ErrorStrategy::Fail,
                context_mode: ContextMode::Full,
            })
            .with_parallel(true);

        assert_eq!(composition.name, "test_workflow");
        assert_eq!(composition.description, "Test composition");
        assert_eq!(composition.steps.len(), 1);
        assert!(composition.parallel);
        assert_eq!(composition.steps[0].tool_name, "tool1");
    }
    #[tokio::test]
    async fn test_tool_capable_discovery() {
        let agent = MockToolCapableAgent::new();

        // Test tool discovery
        let query = ToolQuery::new().with_category("utility");
        let tools = agent.discover_tools(&query).await.unwrap();
        assert_eq!(tools.len(), 1);
        assert_eq!(tools[0].name, "mock_tool");

        // Test empty query
        let empty_query = ToolQuery::new().with_category("nonexistent");
        let no_tools = agent.discover_tools(&empty_query).await.unwrap();
        assert_eq!(no_tools.len(), 0);
    }
    #[tokio::test]
    async fn test_tool_capable_invocation() {
        let agent = MockToolCapableAgent::new();

        // Test tool invocation
        let params = json!({"test": "data"});
        let context = ExecutionContext::new();
        let result = agent
            .invoke_tool("mock_tool", params, context)
            .await
            .unwrap();
        assert!(result.text.contains("Invoked mock_tool"));
        assert!(result.text.contains("test"));

        // Test tool availability
        assert!(agent.tool_available("mock_tool").await);
        assert!(!agent.tool_available("nonexistent").await);

        // Test tool info
        let info = agent.get_tool_info("mock_tool").await.unwrap();
        assert!(info.is_some());
        assert_eq!(info.unwrap().name, "mock_tool");

        let no_info = agent.get_tool_info("nonexistent").await.unwrap();
        assert!(no_info.is_none());
    }
    #[tokio::test]
    async fn test_tool_capable_composition() {
        let agent = MockToolCapableAgent::new();

        let composition = ToolComposition::new("test_flow", "Test composition")
            .with_step(ToolCompositionStep {
                tool_name: "tool1".to_string(),
                parameters: json!({}),
                error_strategy: ErrorStrategy::Fail,
                context_mode: ContextMode::Full,
            })
            .with_step(ToolCompositionStep {
                tool_name: "tool2".to_string(),
                parameters: json!({}),
                error_strategy: ErrorStrategy::Continue,
                context_mode: ContextMode::Previous,
            });

        let context = ExecutionContext::new();
        let result = agent.compose_tools(&composition, context).await.unwrap();
        assert!(result.text.contains("Executed composition 'test_flow'"));
        assert!(result.text.contains("2 steps"));
    }
    #[tokio::test]
    async fn test_tool_capable_listing() {
        let agent = MockToolCapableAgent::new();

        let tools = agent.list_available_tools().await.unwrap();
        assert_eq!(tools.len(), 2);
        assert!(tools.contains(&"mock_tool".to_string()));
        assert!(tools.contains(&"test_tool".to_string()));
    }
}
