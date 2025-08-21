//! ABOUTME: Tool trait for functional components with schema validation
//! ABOUTME: Extends `BaseAgent` with parameter validation and tool categorization

use super::base_agent::BaseAgent;
use crate::{
    types::{AgentInput, AgentStream},
    Result,
};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Tool category for organization and discovery.
///
/// Categorizes tools by their primary function to help with tool selection
/// and organization. Custom categories can be created for specialized tools.
///
/// # Examples
///
/// ```
/// use llmspell_core::traits::tool::ToolCategory;
///
/// let category = ToolCategory::Filesystem;
/// assert_eq!(category.to_string(), "filesystem");
///
/// // Custom category
/// let custom = ToolCategory::Custom("ai-tools".to_string());
/// assert_eq!(custom.to_string(), "ai-tools");
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum ToolCategory {
    Filesystem,
    Web,
    Api,
    Analysis,
    Data,
    System,
    Media,
    Utility,
    Custom(String),
}

impl std::fmt::Display for ToolCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ToolCategory::Filesystem => write!(f, "filesystem"),
            ToolCategory::Web => write!(f, "web"),
            ToolCategory::Api => write!(f, "api"),
            ToolCategory::Analysis => write!(f, "analysis"),
            ToolCategory::Data => write!(f, "data"),
            ToolCategory::System => write!(f, "system"),
            ToolCategory::Media => write!(f, "media"),
            ToolCategory::Utility => write!(f, "utility"),
            ToolCategory::Custom(name) => write!(f, "{}", name),
        }
    }
}

/// Security level for tools.
///
/// Defines the security requirements and permissions needed to execute a tool.
/// Higher levels include permissions of lower levels (Privileged > Restricted > Safe).
///
/// # Examples
///
/// ```
/// use llmspell_core::traits::tool::SecurityLevel;
///
/// let user_level = SecurityLevel::Restricted;
/// let tool_level = SecurityLevel::Safe;
///
/// // User with Restricted can run Safe tools
/// assert!(user_level.allows(&tool_level));
///
/// // But Safe user cannot run Restricted tools
/// assert!(!SecurityLevel::Safe.allows(&SecurityLevel::Restricted));
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum SecurityLevel {
    Safe,
    Restricted,
    Privileged,
}

impl SecurityLevel {
    /// Check if this security level allows execution at the given level
    #[must_use]
    pub fn allows(&self, required: &SecurityLevel) -> bool {
        self >= required
    }
}

/// Security requirements for tool execution
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SecurityRequirements {
    /// Required security level
    pub level: SecurityLevel,
    /// File system permissions (paths that can be accessed)
    pub file_permissions: Vec<String>,
    /// Network permissions (domains that can be accessed)
    pub network_permissions: Vec<String>,
    /// Environment variables that can be accessed
    pub env_permissions: Vec<String>,
    /// Custom security requirements
    pub custom_requirements: HashMap<String, serde_json::Value>,
}

impl Default for SecurityRequirements {
    fn default() -> Self {
        Self {
            level: SecurityLevel::Safe,
            file_permissions: Vec::new(),
            network_permissions: Vec::new(),
            env_permissions: Vec::new(),
            custom_requirements: HashMap::new(),
        }
    }
}

impl SecurityRequirements {
    /// Create safe security requirements (no file/network access)
    #[must_use]
    pub fn safe() -> Self {
        Self::default()
    }

    /// Create restricted security requirements with limited access
    #[must_use]
    pub fn restricted() -> Self {
        Self {
            level: SecurityLevel::Restricted,
            ..Default::default()
        }
    }

    /// Create privileged security requirements with full access
    #[must_use]
    pub fn privileged() -> Self {
        Self {
            level: SecurityLevel::Privileged,
            file_permissions: vec!["*".to_string()],
            network_permissions: vec!["*".to_string()],
            env_permissions: vec!["*".to_string()],
            custom_requirements: HashMap::new(),
        }
    }

    /// Add file permission
    pub fn with_file_access(mut self, path: impl Into<String>) -> Self {
        self.file_permissions.push(path.into());
        self
    }

    /// Add network permission
    pub fn with_network_access(mut self, domain: impl Into<String>) -> Self {
        self.network_permissions.push(domain.into());
        self
    }

    /// Add environment variable permission
    pub fn with_env_access(mut self, var: impl Into<String>) -> Self {
        self.env_permissions.push(var.into());
        self
    }
}

/// Resource limits for tool execution
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ResourceLimits {
    /// Maximum memory usage in bytes
    pub max_memory_bytes: Option<u64>,
    /// Maximum CPU time in milliseconds
    pub max_cpu_time_ms: Option<u64>,
    /// Maximum network bandwidth in bytes per second
    pub max_network_bps: Option<u64>,
    /// Maximum file operations per second
    pub max_file_ops_per_sec: Option<u32>,
    /// Custom resource limits
    pub custom_limits: HashMap<String, u64>,
}

impl Default for ResourceLimits {
    fn default() -> Self {
        Self {
            max_memory_bytes: Some(100 * 1024 * 1024), // 100MB default
            max_cpu_time_ms: Some(30_000),             // 30 seconds default
            max_network_bps: Some(10 * 1024 * 1024),   // 10MB/s default
            max_file_ops_per_sec: Some(100),           // 100 ops/sec default
            custom_limits: HashMap::new(),
        }
    }
}

impl ResourceLimits {
    /// Create unlimited resource limits (for privileged tools)
    #[must_use]
    pub fn unlimited() -> Self {
        Self {
            max_memory_bytes: None,
            max_cpu_time_ms: None,
            max_network_bps: None,
            max_file_ops_per_sec: None,
            custom_limits: HashMap::new(),
        }
    }

    /// Create strict resource limits (for safe tools)
    #[must_use]
    pub fn strict() -> Self {
        Self {
            max_memory_bytes: Some(10 * 1024 * 1024), // 10MB
            max_cpu_time_ms: Some(5_000),             // 5 seconds
            max_network_bps: Some(1024 * 1024),       // 1MB/s
            max_file_ops_per_sec: Some(10),           // 10 ops/sec
            custom_limits: HashMap::new(),
        }
    }

    /// Set memory limit
    #[must_use]
    pub fn with_memory_limit(mut self, bytes: u64) -> Self {
        self.max_memory_bytes = Some(bytes);
        self
    }

    /// Set CPU time limit
    #[must_use]
    pub fn with_cpu_limit(mut self, milliseconds: u64) -> Self {
        self.max_cpu_time_ms = Some(milliseconds);
        self
    }

    /// Set network bandwidth limit
    #[must_use]
    pub fn with_network_limit(mut self, bytes_per_second: u64) -> Self {
        self.max_network_bps = Some(bytes_per_second);
        self
    }
}

/// Parameter type information
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ParameterType {
    String,
    Number,
    Boolean,
    Array,
    Object,
    Null,
}

/// Tool parameter definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParameterDef {
    pub name: String,
    pub param_type: ParameterType,
    pub description: String,
    pub required: bool,
    pub default: Option<serde_json::Value>,
}

/// Tool schema for parameter validation.
///
/// Defines the structure and validation rules for tool parameters.
/// Can be converted to JSON Schema format for integration with LLMs.
///
/// # Examples
///
/// ```
/// use llmspell_core::traits::tool::{ToolSchema, ParameterDef, ParameterType};
/// use serde_json::json;
///
/// let schema = ToolSchema::new(
///     "search_files".to_string(),
///     "Search for files by pattern".to_string()
/// )
/// .with_parameter(ParameterDef {
///     name: "pattern".to_string(),
///     param_type: ParameterType::String,
///     description: "File pattern to search".to_string(),
///     required: true,
///     default: None,
/// })
/// .with_parameter(ParameterDef {
///     name: "recursive".to_string(),
///     param_type: ParameterType::Boolean,
///     description: "Search recursively".to_string(),
///     required: false,
///     default: Some(json!(true)),
/// })
/// .with_returns(ParameterType::Array);
///
/// // Get required parameters
/// assert_eq!(schema.required_parameters(), vec!["pattern"]);
///
/// // Convert to JSON Schema
/// let json_schema = schema.to_json_schema();
/// assert_eq!(json_schema["type"], "object");
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolSchema {
    pub name: String,
    pub description: String,
    pub parameters: Vec<ParameterDef>,
    pub returns: Option<ParameterType>,
}

impl ToolSchema {
    /// Create a new tool schema
    #[must_use]
    pub fn new(name: String, description: String) -> Self {
        Self {
            name,
            description,
            parameters: Vec::new(),
            returns: None,
        }
    }

    /// Add a parameter to the schema
    #[must_use]
    pub fn with_parameter(mut self, param: ParameterDef) -> Self {
        self.parameters.push(param);
        self
    }

    /// Set the return type
    #[must_use]
    pub fn with_returns(mut self, returns: ParameterType) -> Self {
        self.returns = Some(returns);
        self
    }

    /// Get required parameter names
    #[must_use]
    pub fn required_parameters(&self) -> Vec<String> {
        self.parameters
            .iter()
            .filter(|p| p.required)
            .map(|p| p.name.clone())
            .collect()
    }

    /// Convert to JSON schema format
    #[must_use]
    pub fn to_json_schema(&self) -> serde_json::Value {
        let mut properties = serde_json::Map::new();

        for param in &self.parameters {
            let mut param_schema = serde_json::Map::new();
            param_schema.insert(
                "type".to_string(),
                serde_json::Value::String(
                    match param.param_type {
                        ParameterType::String => "string",
                        ParameterType::Number => "number",
                        ParameterType::Boolean => "boolean",
                        ParameterType::Array => "array",
                        ParameterType::Object => "object",
                        ParameterType::Null => "null",
                    }
                    .to_string(),
                ),
            );
            param_schema.insert(
                "description".to_string(),
                serde_json::Value::String(param.description.clone()),
            );

            if let Some(default) = &param.default {
                param_schema.insert("default".to_string(), default.clone());
            }

            properties.insert(param.name.clone(), serde_json::Value::Object(param_schema));
        }

        serde_json::json!({
            "type": "object",
            "properties": properties,
            "required": self.required_parameters()
        })
    }
}

/// Tool trait for functional components.
///
/// Extends `BaseAgent` to create tools - specialized components that perform
/// specific functions with validated parameters. Tools have categories, security
/// levels, and schemas that define their interfaces.
///
/// # Implementation Requirements
///
/// - Must provide accurate parameter schema
/// - Should validate all parameters before execution
/// - Security level should reflect actual requirements
/// - Category should accurately describe tool function
///
/// # Examples
///
/// ```
/// use llmspell_core::{
///     ComponentMetadata, Result, LLMSpellError, ExecutionContext,
///     traits::{
///         base_agent::BaseAgent,
///         tool::{Tool, ToolCategory, SecurityLevel, ToolSchema, ParameterDef, ParameterType}
///     },
///     types::{AgentInput, AgentOutput}
/// };
/// use async_trait::async_trait;
/// use serde_json::json;
///
/// struct FileSearchTool {
///     metadata: ComponentMetadata,
/// }
///
/// #[async_trait]
/// impl Tool for FileSearchTool {
///     fn category(&self) -> ToolCategory {
///         ToolCategory::Filesystem
///     }
///     
///     fn security_level(&self) -> SecurityLevel {
///         SecurityLevel::Safe
///     }
///     
///     fn schema(&self) -> ToolSchema {
///         ToolSchema::new(
///             "file_search".to_string(),
///             "Search for files".to_string()
///         )
///         .with_parameter(ParameterDef {
///             name: "pattern".to_string(),
///             param_type: ParameterType::String,
///             description: "Search pattern".to_string(),
///             required: true,
///             default: None,
///         })
///         .with_returns(ParameterType::Array)
///     }
/// }
///
/// #[async_trait]
/// impl BaseAgent for FileSearchTool {
///     fn metadata(&self) -> &ComponentMetadata {
///         &self.metadata
///     }
///     
///     async fn execute(
///         &self,
///         input: AgentInput,
///         context: ExecutionContext,
///     ) -> Result<AgentOutput> {
///         // Get parameters from input context
///         let params = input.parameters.get("parameters")
///             .ok_or_else(|| LLMSpellError::Validation {
///                 message: "Missing parameters".to_string(),
///                 field: Some("parameters".to_string()),
///             })?;
///         
///         // Validate parameters
///         self.validate_parameters(params).await?;
///         
///         // Execute tool logic
///         let pattern = params["pattern"].as_str().unwrap();
///         let results = json!(["file1.txt", "file2.txt"]);
///         
///         Ok(AgentOutput::text(results.to_string()))
///     }
///     
///     async fn validate_input(&self, input: &AgentInput) -> Result<()> {
///         Ok(())
///     }
///     
///     async fn handle_error(&self, error: LLMSpellError) -> Result<AgentOutput> {
///         Ok(AgentOutput::text(format!("Tool error: {}", error)))
///     }
/// }
/// ```
#[async_trait]
pub trait Tool: BaseAgent {
    /// Get tool category
    fn category(&self) -> ToolCategory;

    /// Get security level
    fn security_level(&self) -> SecurityLevel;

    /// Get parameter schema
    fn schema(&self) -> ToolSchema;

    /// Get security requirements for this tool
    fn security_requirements(&self) -> SecurityRequirements {
        SecurityRequirements {
            level: self.security_level(),
            ..Default::default()
        }
    }

    /// Get resource limits for this tool
    fn resource_limits(&self) -> ResourceLimits {
        match self.security_level() {
            SecurityLevel::Safe => ResourceLimits::strict(),
            SecurityLevel::Restricted => ResourceLimits::default(),
            SecurityLevel::Privileged => ResourceLimits::unlimited(),
        }
    }

    /// Execute tool with streaming output
    async fn stream_execute(
        &self,
        input: AgentInput,
        context: crate::execution_context::ExecutionContext,
    ) -> Result<AgentStream> {
        // Default implementation: execute normally and convert to a single chunk stream
        let output = self.execute(input, context).await?;

        // Convert AgentOutput to AgentChunk
        let chunk = crate::types::AgentChunk {
            stream_id: format!("tool-{}", uuid::Uuid::new_v4()),
            chunk_index: 0,
            content: crate::types::ChunkContent::Text(output.text),
            metadata: crate::types::ChunkMetadata {
                is_final: true,
                token_count: None,
                model: None,
                reasoning_step: None,
            },
            timestamp: chrono::Utc::now(),
        };

        // Create a simple stream with just the final chunk
        use futures::stream;
        let stream = stream::once(async move { Ok(chunk) });
        Ok(Box::pin(stream))
    }

    /// Validate tool parameters
    async fn validate_parameters(&self, params: &serde_json::Value) -> Result<()> {
        // Basic validation implementation
        let schema = self.schema();

        // Check that params is an object
        let params_map = params
            .as_object()
            .ok_or_else(|| crate::LLMSpellError::Validation {
                message: "Parameters must be an object".to_string(),
                field: Some("parameters".to_string()),
            })?;

        // Check required parameters
        for required in schema.required_parameters() {
            if !params_map.contains_key(&required) {
                return Err(crate::LLMSpellError::Validation {
                    message: format!("Missing required parameter: {required}"),
                    field: Some(required.clone()),
                });
            }
        }

        // Check parameter types
        for param_def in &schema.parameters {
            if let Some(value) = params_map.get(&param_def.name) {
                let valid_type = match param_def.param_type {
                    ParameterType::String => value.is_string(),
                    ParameterType::Number => value.is_number(),
                    ParameterType::Boolean => value.is_boolean(),
                    ParameterType::Array => value.is_array(),
                    ParameterType::Object => value.is_object(),
                    ParameterType::Null => value.is_null(),
                };

                if !valid_type {
                    return Err(crate::LLMSpellError::Validation {
                        message: format!(
                            "Invalid type for parameter '{}': expected {:?}",
                            param_def.name, param_def.param_type
                        ),
                        field: Some(param_def.name.clone()),
                    });
                }
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{AgentInput, AgentOutput};
    use crate::ComponentMetadata;
    use crate::ExecutionContext;
    #[test]
    fn test_tool_category_display() {
        assert_eq!(ToolCategory::Filesystem.to_string(), "filesystem");
        assert_eq!(ToolCategory::Web.to_string(), "web");
        assert_eq!(ToolCategory::Custom("ai".to_string()).to_string(), "ai");
    }
    #[test]
    fn test_security_level_ordering() {
        assert!(SecurityLevel::Safe < SecurityLevel::Restricted);
        assert!(SecurityLevel::Restricted < SecurityLevel::Privileged);
        assert!(SecurityLevel::Privileged.allows(&SecurityLevel::Safe));
        assert!(!SecurityLevel::Safe.allows(&SecurityLevel::Privileged));
    }
    #[test]
    fn test_parameter_def_creation() {
        let param = ParameterDef {
            name: "input".to_string(),
            param_type: ParameterType::String,
            description: "Input text".to_string(),
            required: true,
            default: None,
        };

        assert_eq!(param.name, "input");
        assert!(param.required);
        assert_eq!(param.param_type, ParameterType::String);
    }
    #[test]
    fn test_tool_schema_builder() {
        let schema = ToolSchema::new("test_tool".to_string(), "A test tool".to_string())
            .with_parameter(ParameterDef {
                name: "text".to_string(),
                param_type: ParameterType::String,
                description: "Input text".to_string(),
                required: true,
                default: None,
            })
            .with_parameter(ParameterDef {
                name: "count".to_string(),
                param_type: ParameterType::Number,
                description: "Number of items".to_string(),
                required: false,
                default: Some(serde_json::json!(1)),
            })
            .with_returns(ParameterType::String);

        assert_eq!(schema.name, "test_tool");
        assert_eq!(schema.parameters.len(), 2);
        assert_eq!(schema.returns, Some(ParameterType::String));
        assert_eq!(schema.required_parameters(), vec!["text"]);
    }
    #[test]
    fn test_tool_schema_json_conversion() {
        let schema =
            ToolSchema::new("test".to_string(), "Test".to_string()).with_parameter(ParameterDef {
                name: "input".to_string(),
                param_type: ParameterType::String,
                description: "Input parameter".to_string(),
                required: true,
                default: None,
            });

        let json_schema = schema.to_json_schema();
        assert_eq!(json_schema["type"], "object");
        assert_eq!(json_schema["required"], serde_json::json!(["input"]));
        assert_eq!(json_schema["properties"]["input"]["type"], "string");
    }

    // Mock tool implementation for testing
    struct MockTool {
        metadata: ComponentMetadata,
        category: ToolCategory,
        security_level: SecurityLevel,
        schema: ToolSchema,
    }

    impl MockTool {
        fn new() -> Self {
            let schema = ToolSchema::new(
                "mock_tool".to_string(),
                "A mock tool for testing".to_string(),
            )
            .with_parameter(ParameterDef {
                name: "text".to_string(),
                param_type: ParameterType::String,
                description: "Text to process".to_string(),
                required: true,
                default: None,
            })
            .with_parameter(ParameterDef {
                name: "uppercase".to_string(),
                param_type: ParameterType::Boolean,
                description: "Convert to uppercase".to_string(),
                required: false,
                default: Some(serde_json::json!(false)),
            })
            .with_returns(ParameterType::String);

            Self {
                metadata: ComponentMetadata::new(
                    "mock-tool".to_string(),
                    "A mock tool for testing".to_string(),
                ),
                category: ToolCategory::Utility,
                security_level: SecurityLevel::Safe,
                schema,
            }
        }
    }

    #[async_trait]
    impl BaseAgent for MockTool {
        fn metadata(&self) -> &ComponentMetadata {
            &self.metadata
        }

        async fn execute_impl(
            &self,
            input: AgentInput,
            _context: ExecutionContext,
        ) -> Result<AgentOutput> {
            // Parse parameters from input context
            let params = input.parameters.get("parameters").ok_or_else(|| {
                crate::LLMSpellError::Validation {
                    message: "Missing parameters in input".to_string(),
                    field: Some("parameters".to_string()),
                }
            })?;

            // Validate parameters
            self.validate_parameters(params).await?;

            // Execute tool logic
            let text = params["text"].as_str().unwrap();
            let uppercase = params
                .get("uppercase")
                .and_then(|v| v.as_bool())
                .unwrap_or(false);

            let result = if uppercase {
                text.to_uppercase()
            } else {
                text.to_string()
            };

            Ok(AgentOutput::text(result))
        }

        async fn validate_input(&self, input: &AgentInput) -> Result<()> {
            if input.text.is_empty() {
                return Err(crate::LLMSpellError::Validation {
                    message: "Input prompt cannot be empty".to_string(),
                    field: Some("prompt".to_string()),
                });
            }
            Ok(())
        }

        async fn handle_error(&self, error: crate::LLMSpellError) -> Result<AgentOutput> {
            Ok(AgentOutput::text(format!("Tool error: {}", error)))
        }
    }

    #[async_trait]
    impl Tool for MockTool {
        fn category(&self) -> ToolCategory {
            self.category.clone()
        }

        fn security_level(&self) -> SecurityLevel {
            self.security_level.clone()
        }

        fn schema(&self) -> ToolSchema {
            self.schema.clone()
        }
    }
    #[tokio::test]
    async fn test_tool_parameter_validation() {
        let tool = MockTool::new();

        // Valid parameters
        let valid_params = serde_json::json!({
            "text": "hello",
            "uppercase": true
        });
        assert!(tool.validate_parameters(&valid_params).await.is_ok());

        // Missing required parameter
        let missing_params = serde_json::json!({
            "uppercase": true
        });
        let result = tool.validate_parameters(&missing_params).await;
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Missing required parameter"));

        // Wrong parameter type
        let wrong_type = serde_json::json!({
            "text": 123,
            "uppercase": true
        });
        let result = tool.validate_parameters(&wrong_type).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Invalid type"));

        // Non-object parameters
        let non_object = serde_json::json!("not an object");
        let result = tool.validate_parameters(&non_object).await;
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("must be an object"));
    }
    #[tokio::test]
    async fn test_tool_execution() {
        let tool = MockTool::new();

        // Test with uppercase = false
        let input = AgentInput::text("process text".to_string()).with_parameter(
            "parameters".to_string(),
            serde_json::json!({
                "text": "hello world",
                "uppercase": false
            }),
        );
        let context = ExecutionContext::with_conversation("session".to_string());

        let result = tool.execute(input, context).await.unwrap();
        assert_eq!(result.text, "hello world");

        // Test with uppercase = true
        let input = AgentInput::text("process text".to_string()).with_parameter(
            "parameters".to_string(),
            serde_json::json!({
                "text": "hello world",
                "uppercase": true
            }),
        );
        let context = ExecutionContext::with_conversation("session".to_string());

        let result = tool.execute(input, context).await.unwrap();
        assert_eq!(result.text, "HELLO WORLD");
    }
    #[test]
    fn test_tool_metadata() {
        let tool = MockTool::new();

        assert_eq!(tool.category(), ToolCategory::Utility);
        assert_eq!(tool.security_level(), SecurityLevel::Safe);
        assert_eq!(tool.schema().name, "mock_tool");
        assert_eq!(tool.metadata().name, "mock-tool");
    }
}
