//! ABOUTME: Agent-as-tool wrapper for enabling agents to be used as tools
//! ABOUTME: Provides seamless integration allowing any `BaseAgent` to be invoked as a `Tool`

use async_trait::async_trait;
use llmspell_core::{
    traits::{
        base_agent::BaseAgent,
        tool::{ParameterDef, ParameterType, SecurityLevel, Tool, ToolCategory, ToolSchema},
    },
    types::{AgentInput, AgentOutput},
    ComponentMetadata, ExecutionContext, LLMSpellError, Result,
};
use serde_json::{Map, Value as JsonValue};
use std::sync::Arc;

/// Wrapper that allows any `BaseAgent` to be used as a `Tool`.
///
/// This enables agent-as-tool composition patterns where agents can be
/// discovered, invoked, and composed like tools through the tool system.
///
/// # Examples
///
/// ```
/// use llmspell_agents::agent_wrapped_tool::AgentWrappedTool;
/// use llmspell_core::traits::tool::{Tool, ToolCategory, SecurityLevel};
/// use std::sync::Arc;
///
/// # async fn example() -> llmspell_core::Result<()> {
/// // Assuming we have an agent instance
/// // let agent: Arc<dyn BaseAgent> = ...;
///
/// // Wrap it as a tool
/// // let tool = AgentWrappedTool::new(agent, ToolCategory::Utility, SecurityLevel::Safe);
///
/// // Now it can be used as a tool in the tool ecosystem
/// // let schema = tool.schema();
/// // let result = tool.execute(input, context).await?;
/// # Ok(())
/// # }
/// ```
pub struct AgentWrappedTool {
    /// The wrapped agent instance
    agent: Arc<dyn BaseAgent>,
    /// Tool category for discovery
    category: ToolCategory,
    /// Security level for the wrapped agent
    security_level: SecurityLevel,
    /// Custom tool metadata override
    tool_metadata: Option<ToolMetadata>,
    /// Parameter mapping configuration
    parameter_config: ParameterMappingConfig,
}

/// Custom metadata for the wrapped tool
#[derive(Debug, Clone)]
pub struct ToolMetadata {
    /// Override tool name (defaults to agent name)
    pub name: Option<String>,
    /// Override tool description (defaults to agent description)
    pub description: Option<String>,
    /// Additional tool capabilities
    pub capabilities: Vec<String>,
    /// Tool-specific requirements
    pub requirements: JsonValue,
}

/// Configuration for parameter mapping between tool and agent interfaces
#[derive(Debug, Clone)]
pub struct ParameterMappingConfig {
    /// Whether to pass all tool parameters as a single "parameters" field to the agent
    pub bundle_parameters: bool,
    /// Custom parameter transformations
    pub parameter_transforms: std::collections::HashMap<String, ParameterTransform>,
    /// Whether to include tool metadata in agent context
    pub include_tool_context: bool,
    /// Custom input text template
    pub input_template: Option<String>,
}

/// Parameter transformation rule
#[derive(Debug, Clone)]
pub struct ParameterTransform {
    /// Target parameter name in agent input
    pub target_name: String,
    /// Value transformation type
    pub transform_type: TransformType,
    /// Whether this parameter is required
    pub required: bool,
    /// Default value if not provided
    pub default_value: Option<JsonValue>,
}

/// Types of parameter transformations
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TransformType {
    /// Pass through as-is
    Identity,
    /// Convert to string representation
    ToString,
    /// Extract specific field from object
    ExtractField(String),
    /// Apply custom JSON path
    JsonPath(String),
    /// Convert using custom function (name only for serialization)
    Custom(String),
}

impl Default for ParameterMappingConfig {
    fn default() -> Self {
        Self {
            bundle_parameters: true,
            parameter_transforms: std::collections::HashMap::new(),
            include_tool_context: true,
            input_template: None,
        }
    }
}

impl ParameterMappingConfig {
    /// Create new parameter mapping configuration
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Don't bundle parameters, pass them individually
    #[must_use]
    pub const fn with_unbundled_parameters(mut self) -> Self {
        self.bundle_parameters = false;
        self
    }

    /// Add a parameter transformation
    #[must_use]
    pub fn with_parameter_transform(
        mut self,
        source_name: impl Into<String>,
        transform: ParameterTransform,
    ) -> Self {
        self.parameter_transforms
            .insert(source_name.into(), transform);
        self
    }

    /// Don't include tool context in agent execution
    #[must_use]
    pub const fn without_tool_context(mut self) -> Self {
        self.include_tool_context = false;
        self
    }

    /// Set custom input template for agent
    #[must_use]
    pub fn with_input_template(mut self, template: impl Into<String>) -> Self {
        self.input_template = Some(template.into());
        self
    }
}

impl ParameterTransform {
    /// Create identity transform
    pub fn identity(target_name: impl Into<String>) -> Self {
        Self {
            target_name: target_name.into(),
            transform_type: TransformType::Identity,
            required: false,
            default_value: None,
        }
    }

    /// Create string conversion transform
    pub fn to_string(target_name: impl Into<String>) -> Self {
        Self {
            target_name: target_name.into(),
            transform_type: TransformType::ToString,
            required: false,
            default_value: None,
        }
    }

    /// Create field extraction transform
    pub fn extract_field(target_name: impl Into<String>, field_name: impl Into<String>) -> Self {
        Self {
            target_name: target_name.into(),
            transform_type: TransformType::ExtractField(field_name.into()),
            required: false,
            default_value: None,
        }
    }

    /// Mark transform as required
    #[must_use]
    pub const fn required(mut self) -> Self {
        self.required = true;
        self
    }

    /// Set default value
    #[must_use]
    pub fn with_default(mut self, default: JsonValue) -> Self {
        self.default_value = Some(default);
        self
    }
}

impl AgentWrappedTool {
    /// Create a new agent-wrapped tool with default configuration
    pub fn new(
        agent: Arc<dyn BaseAgent>,
        category: ToolCategory,
        security_level: SecurityLevel,
    ) -> Self {
        Self {
            agent,
            category,
            security_level,
            tool_metadata: None,
            parameter_config: ParameterMappingConfig::default(),
        }
    }

    /// Create with custom tool metadata
    pub fn with_metadata(
        agent: Arc<dyn BaseAgent>,
        category: ToolCategory,
        security_level: SecurityLevel,
        metadata: ToolMetadata,
    ) -> Self {
        Self {
            agent,
            category,
            security_level,
            tool_metadata: Some(metadata),
            parameter_config: ParameterMappingConfig::default(),
        }
    }

    /// Create with custom parameter mapping
    pub fn with_parameter_config(
        agent: Arc<dyn BaseAgent>,
        category: ToolCategory,
        security_level: SecurityLevel,
        parameter_config: ParameterMappingConfig,
    ) -> Self {
        Self {
            agent,
            category,
            security_level,
            tool_metadata: None,
            parameter_config,
        }
    }

    /// Create with both custom metadata and parameter configuration
    pub fn with_full_config(
        agent: Arc<dyn BaseAgent>,
        category: ToolCategory,
        security_level: SecurityLevel,
        metadata: ToolMetadata,
        parameter_config: ParameterMappingConfig,
    ) -> Self {
        Self {
            agent,
            category,
            security_level,
            tool_metadata: Some(metadata),
            parameter_config,
        }
    }

    /// Get the wrapped agent
    #[must_use]
    pub fn agent(&self) -> &Arc<dyn BaseAgent> {
        &self.agent
    }

    /// Get tool metadata override
    #[must_use]
    pub const fn tool_metadata(&self) -> Option<&ToolMetadata> {
        self.tool_metadata.as_ref()
    }

    /// Get parameter mapping configuration
    #[must_use]
    pub const fn parameter_config(&self) -> &ParameterMappingConfig {
        &self.parameter_config
    }

    /// Transform tool parameters to agent input according to configuration
    fn transform_parameters(&self, tool_parameters: &Map<String, JsonValue>) -> Result<AgentInput> {
        let agent_metadata = self.agent.metadata();

        // Start with base input text
        let input_text = self.parameter_config.input_template.as_ref().map_or_else(
            || format!("Tool invocation of agent: {}", agent_metadata.name),
            |template| template.clone()
        );

        let mut agent_input = AgentInput::text(input_text);

        if self.parameter_config.bundle_parameters {
            // Bundle all parameters as a single "parameters" field
            let parameters_value = JsonValue::Object(tool_parameters.clone());
            agent_input = agent_input.with_parameter("parameters".to_string(), parameters_value);
        } else {
            // Pass parameters individually with potential transformations
            for (param_name, param_value) in tool_parameters {
                let (target_name, transformed_value) = if let Some(transform) =
                    self.parameter_config.parameter_transforms.get(param_name)
                {
                    let transformed = Self::apply_transform(param_value, transform)?;
                    (transform.target_name.clone(), transformed)
                } else {
                    (param_name.clone(), param_value.clone())
                };

                agent_input = agent_input.with_parameter(target_name, transformed_value);
            }

            // Apply default values for missing required transforms
            for (source_name, transform) in &self.parameter_config.parameter_transforms {
                if transform.required && !tool_parameters.contains_key(source_name) {
                    if let Some(default_value) = &transform.default_value {
                        agent_input = agent_input
                            .with_parameter(transform.target_name.clone(), default_value.clone());
                    } else {
                        return Err(LLMSpellError::Validation {
                            message: format!("Required parameter '{source_name}' not provided"),
                            field: Some(source_name.clone()),
                        });
                    }
                }
            }
        }

        // Add tool context if enabled
        if self.parameter_config.include_tool_context {
            let tool_context = JsonValue::Object({
                let mut context = Map::new();
                context.insert("tool_name".to_string(), JsonValue::String(self.tool_name()));
                context.insert(
                    "tool_category".to_string(),
                    JsonValue::String(self.category.to_string()),
                );
                context.insert(
                    "security_level".to_string(),
                    JsonValue::String(
                        match self.security_level {
                            SecurityLevel::Safe => "safe",
                            SecurityLevel::Restricted => "restricted",
                            SecurityLevel::Privileged => "privileged",
                        }
                        .to_string(),
                    ),
                );
                context
            });
            agent_input = agent_input.with_parameter("tool_context".to_string(), tool_context);
        }

        Ok(agent_input)
    }

    /// Apply a single parameter transformation
    fn apply_transform(value: &JsonValue, transform: &ParameterTransform) -> Result<JsonValue> {
        match &transform.transform_type {
            TransformType::Identity => Ok(value.clone()),
            TransformType::ToString => {
                let string_value = match value {
                    JsonValue::String(s) => s.clone(),
                    JsonValue::Number(n) => n.to_string(),
                    JsonValue::Bool(b) => b.to_string(),
                    JsonValue::Null => "null".to_string(),
                    _ => serde_json::to_string(value).map_err(|e| LLMSpellError::Component {
                        message: format!("Failed to convert value to string: {e}"),
                        source: Some(Box::new(e)),
                    })?,
                };
                Ok(JsonValue::String(string_value))
            }
            TransformType::ExtractField(field_name) => {
                if let JsonValue::Object(obj) = value {
                    Ok(obj.get(field_name).cloned().unwrap_or(JsonValue::Null))
                } else {
                    Err(LLMSpellError::Validation {
                        message: format!(
                            "Cannot extract field '{field_name}' from non-object value"
                        ),
                        field: Some(field_name.clone()),
                    })
                }
            }
            TransformType::JsonPath(path) => {
                // Simple JSON path implementation for basic cases
                if let Some(field_path) = path.strip_prefix("$.") {
                    let mut current = value;
                    for field in field_path.split('.') {
                        if let JsonValue::Object(obj) = current {
                            current = obj.get(field).unwrap_or(&JsonValue::Null);
                        } else {
                            return Ok(JsonValue::Null);
                        }
                    }
                    Ok(current.clone())
                } else {
                    Err(LLMSpellError::Validation {
                        message: format!("Unsupported JSON path: {path}"),
                        field: Some("json_path".to_string()),
                    })
                }
            }
            TransformType::Custom(function_name) => {
                // For now, just log that custom transforms are not implemented
                tracing::warn!(
                    "Custom transform '{}' not implemented, using identity",
                    function_name
                );
                Ok(value.clone())
            }
        }
    }

    /// Get the effective tool name
    fn tool_name(&self) -> String {
        if let Some(metadata) = &self.tool_metadata {
            if let Some(name) = &metadata.name {
                return name.clone();
            }
        }
        format!("agent-{}", self.agent.metadata().name)
    }

    /// Get the effective tool description
    fn tool_description(&self) -> String {
        if let Some(metadata) = &self.tool_metadata {
            if let Some(description) = &metadata.description {
                return description.clone();
            }
        }
        format!(
            "Agent '{}' wrapped as tool: {}",
            self.agent.metadata().name,
            self.agent.metadata().description
        )
    }
}

#[async_trait]
impl BaseAgent for AgentWrappedTool {
    fn metadata(&self) -> &ComponentMetadata {
        // We'll create a cached metadata on first access
        // For now, delegate to the wrapped agent
        self.agent.metadata()
    }

    async fn execute(&self, input: AgentInput, context: ExecutionContext) -> Result<AgentOutput> {
        // Check if this is a tool invocation (has "parameters" field) or direct agent execution
        if input.parameters.contains_key("parameters") {
            // This is a tool invocation - transform parameters
            let tool_parameters = input
                .parameters
                .get("parameters")
                .and_then(|v| v.as_object())
                .cloned()
                .unwrap_or_else(|| {
                    // If no "parameters" field, use all input parameters
                    input
                        .parameters
                        .iter()
                        .map(|(k, v)| (k.clone(), v.clone()))
                        .collect()
                });

            // Transform parameters according to configuration
            let agent_input = self.transform_parameters(&tool_parameters)?;

            // Execute the wrapped agent
            self.agent.execute(agent_input, context).await
        } else {
            // Direct agent execution - pass through
            self.agent.execute(input, context).await
        }
    }

    async fn validate_input(&self, input: &AgentInput) -> Result<()> {
        self.agent.validate_input(input).await
    }

    async fn handle_error(&self, error: LLMSpellError) -> Result<AgentOutput> {
        self.agent.handle_error(error).await
    }
}

#[async_trait]
impl Tool for AgentWrappedTool {
    fn category(&self) -> ToolCategory {
        self.category.clone()
    }

    fn security_level(&self) -> SecurityLevel {
        self.security_level.clone()
    }

    fn schema(&self) -> ToolSchema {
        // Create a basic schema that accepts flexible parameters
        let mut schema = ToolSchema::new(self.tool_name(), self.tool_description());

        // Add parameters based on configuration
        if self.parameter_config.bundle_parameters {
            // Single parameters object
            schema = schema.with_parameter(ParameterDef {
                name: "parameters".to_string(),
                param_type: ParameterType::Object,
                description: "Parameters to pass to the wrapped agent".to_string(),
                required: false,
                default: Some(JsonValue::Object(Map::new())),
            });
        } else {
            // Individual parameters based on transforms
            for (source_name, transform) in &self.parameter_config.parameter_transforms {
                schema = schema.with_parameter(ParameterDef {
                    name: source_name.clone(),
                    param_type: ParameterType::Object, // Accept any type for flexibility
                    description: format!("Parameter '{source_name}' for agent"),
                    required: transform.required,
                    default: transform.default_value.clone(),
                });
            }

            // Add a generic input parameter if no transforms defined
            if self.parameter_config.parameter_transforms.is_empty() {
                schema = schema.with_parameter(ParameterDef {
                    name: "input".to_string(),
                    param_type: ParameterType::Object,
                    description: "Generic input for the wrapped agent".to_string(),
                    required: false,
                    default: Some(JsonValue::Object(Map::new())),
                });
            }
        }

        schema
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use llmspell_core::ComponentMetadata;
    use serde_json::json;

    // Mock agent for testing
    struct MockAgent {
        metadata: ComponentMetadata,
    }

    impl MockAgent {
        fn new(name: &str, description: &str) -> Self {
            Self {
                metadata: ComponentMetadata::new(name.to_string(), description.to_string()),
            }
        }
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
            let text = input.text;
            let params = input
                .parameters
                .get("parameters")
                .map(|v| format!(" with params: {}", v))
                .unwrap_or_default();

            Ok(AgentOutput::text(format!(
                "MockAgent executed: {}{}",
                text, params
            )))
        }

        async fn validate_input(&self, _input: &AgentInput) -> Result<()> {
            Ok(())
        }

        async fn handle_error(&self, error: LLMSpellError) -> Result<AgentOutput> {
            Ok(AgentOutput::text(format!("Error handled: {}", error)))
        }
    }
    #[tokio::test]
    async fn test_agent_wrapped_tool_creation() {
        let agent = Arc::new(MockAgent::new("test-agent", "A test agent"));
        let wrapped = AgentWrappedTool::new(agent, ToolCategory::Utility, SecurityLevel::Safe);

        assert_eq!(wrapped.category(), ToolCategory::Utility);
        assert_eq!(wrapped.security_level(), SecurityLevel::Safe);
        assert_eq!(wrapped.agent().metadata().name, "test-agent");
    }
    #[tokio::test]
    async fn test_tool_schema_generation() {
        let agent = Arc::new(MockAgent::new("test-agent", "A test agent"));
        let wrapped = AgentWrappedTool::new(agent, ToolCategory::Utility, SecurityLevel::Safe);

        let schema = wrapped.schema();
        assert_eq!(schema.name, "agent-test-agent");
        assert!(schema.description.contains("test agent"));
        assert!(!schema.parameters.is_empty());
    }
    #[tokio::test]
    async fn test_parameter_bundling() {
        let agent = Arc::new(MockAgent::new("test-agent", "A test agent"));
        let wrapped = AgentWrappedTool::new(agent, ToolCategory::Utility, SecurityLevel::Safe);

        let input = AgentInput::text("test input").with_parameter(
            "parameters".to_string(),
            json!({
                "key1": "value1",
                "key2": 42
            }),
        );

        let context = ExecutionContext::new();
        let result = wrapped.execute(input, context).await.unwrap();

        assert!(result.text.contains("MockAgent executed"));
        assert!(result.text.contains("with params"));
    }
    #[tokio::test]
    async fn test_parameter_transforms() {
        let agent = Arc::new(MockAgent::new("test-agent", "A test agent"));

        let config = ParameterMappingConfig::new()
            .with_unbundled_parameters()
            .with_parameter_transform(
                "input_text",
                ParameterTransform::identity("text").required(),
            )
            .with_parameter_transform("number_val", ParameterTransform::to_string("number_str"));

        let wrapped = AgentWrappedTool::with_parameter_config(
            agent,
            ToolCategory::Utility,
            SecurityLevel::Safe,
            config,
        );

        let input = AgentInput::text("test input")
            .with_parameter("input_text".to_string(), json!("hello"))
            .with_parameter("number_val".to_string(), json!(123));

        let context = ExecutionContext::new();
        let result = wrapped.execute(input, context).await.unwrap();

        assert!(result.text.contains("MockAgent executed"));
    }
    #[tokio::test]
    async fn test_custom_metadata() {
        let agent = Arc::new(MockAgent::new("test-agent", "A test agent"));

        let metadata = ToolMetadata {
            name: Some("custom-tool-name".to_string()),
            description: Some("Custom tool description".to_string()),
            capabilities: vec!["capability1".to_string(), "capability2".to_string()],
            requirements: json!({"min_memory": "1GB"}),
        };

        let wrapped = AgentWrappedTool::with_metadata(
            agent,
            ToolCategory::Analysis,
            SecurityLevel::Restricted,
            metadata,
        );

        let schema = wrapped.schema();
        assert_eq!(schema.name, "custom-tool-name");
        assert_eq!(schema.description, "Custom tool description");
    }
    #[tokio::test]
    async fn test_parameter_transform_types() {
        let agent = Arc::new(MockAgent::new("test-agent", "A test agent"));
        let _wrapped = AgentWrappedTool::new(agent, ToolCategory::Utility, SecurityLevel::Safe);

        // Test ToString transform
        let transform = ParameterTransform::to_string("target");
        let result = AgentWrappedTool::apply_transform(&json!(123), &transform).unwrap();
        assert_eq!(result, json!("123"));

        // Test ExtractField transform
        let transform = ParameterTransform::extract_field("target", "field1");
        let result = AgentWrappedTool::apply_transform(
            &json!({"field1": "value1", "field2": "value2"}),
            &transform,
        )
        .unwrap();
        assert_eq!(result, json!("value1"));

        // Test Identity transform
        let transform = ParameterTransform::identity("target");
        let result =
            AgentWrappedTool::apply_transform(&json!({"complex": "object"}), &transform).unwrap();
        assert_eq!(result, json!({"complex": "object"}));
    }
}
