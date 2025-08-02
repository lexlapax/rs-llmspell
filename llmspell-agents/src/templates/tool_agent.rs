//! ABOUTME: Tool Agent template for creating agents that primarily execute tools
//! ABOUTME: Provides standardized template for tool-execution focused agents with configurable tool sets

use super::base::{AgentTemplate, TemplateInstantiationParams, TemplateInstantiationResult};
use super::schema::{
    CapabilityRequirement, ComplexityLevel, ParameterDefinition, ParameterType,
    ResourceRequirements, TemplateCategory, TemplateMetadata, TemplateSchema, ToolDependency,
};
use anyhow::{anyhow, Result};
use async_trait::async_trait;
use llmspell_core::{
    types::{AgentInput, AgentOutput, ComponentId, ComponentMetadata, OutputMetadata, Version},
    BaseAgent, ExecutionContext, LLMSpellError,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Tool Agent configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolAgentConfig {
    /// Maximum number of tools to load
    pub max_tools: usize,
    /// Enable tool result caching
    pub enable_caching: bool,
    /// Tool execution timeout in seconds
    pub tool_timeout: u64,
    /// Maximum concurrent tool executions
    pub max_concurrent_tools: usize,
    /// Enable tool error recovery
    pub enable_error_recovery: bool,
    /// Tool discovery patterns
    pub tool_discovery_patterns: Vec<String>,
    /// Custom tool configurations
    pub tool_configs: HashMap<String, serde_json::Value>,
}

impl Default for ToolAgentConfig {
    fn default() -> Self {
        Self {
            max_tools: 50,
            enable_caching: true,
            tool_timeout: 300, // 5 minutes
            max_concurrent_tools: 5,
            enable_error_recovery: true,
            tool_discovery_patterns: vec!["*.rs".to_string(), "*.lua".to_string()],
            tool_configs: HashMap::new(),
        }
    }
}

/// Tool Agent template implementation
pub struct ToolAgentTemplate {
    schema: TemplateSchema,
    config: ToolAgentConfig,
}

impl ToolAgentTemplate {
    /// Create new Tool Agent template
    #[must_use]
    pub fn new() -> Self {
        let metadata = TemplateMetadata {
            id: "tool_agent".to_string(),
            name: "Tool Agent".to_string(),
            version: "1.0.0".to_string(),
            description: "Agent template for tool execution and automation tasks".to_string(),
            author: "rs-llmspell".to_string(),
            license: "MIT".to_string(),
            repository: Some("https://github.com/lexlapax/rs-llmspell".to_string()),
            documentation: Some("https://docs.rs/llmspell-agents".to_string()),
            keywords: vec![
                "tool".to_string(),
                "execution".to_string(),
                "automation".to_string(),
                "agent".to_string(),
            ],
            category: TemplateCategory::ToolExecution,
            complexity: ComplexityLevel::Intermediate,
        };

        let mut schema = TemplateSchema::new(metadata);

        // Add parameters
        schema = schema
            .with_parameter(ParameterDefinition {
                name: "agent_name".to_string(),
                description: "Human-readable name for the agent".to_string(),
                param_type: ParameterType::String,
                required: true,
                default_value: None,
                constraints: vec![],
                examples: vec!["File Processor Agent".into(), "Data Analysis Agent".into()],
            })
            .with_parameter(ParameterDefinition {
                name: "max_tools".to_string(),
                description: "Maximum number of tools this agent can load".to_string(),
                param_type: ParameterType::Integer,
                required: false,
                default_value: Some(50.into()),
                constraints: vec![],
                examples: vec![10.into(), 50.into(), 100.into()],
            })
            .with_parameter(ParameterDefinition {
                name: "tool_timeout".to_string(),
                description: "Tool execution timeout in seconds".to_string(),
                param_type: ParameterType::Integer,
                required: false,
                default_value: Some(300.into()),
                constraints: vec![],
                examples: vec![60.into(), 300.into(), 600.into()],
            })
            .with_parameter(ParameterDefinition {
                name: "enable_caching".to_string(),
                description: "Enable tool result caching for performance".to_string(),
                param_type: ParameterType::Boolean,
                required: false,
                default_value: Some(true.into()),
                constraints: vec![],
                examples: vec![true.into(), false.into()],
            })
            .with_parameter(ParameterDefinition {
                name: "max_concurrent_tools".to_string(),
                description: "Maximum number of tools that can run concurrently".to_string(),
                param_type: ParameterType::Integer,
                required: false,
                default_value: Some(5.into()),
                constraints: vec![],
                examples: vec![1.into(), 5.into(), 10.into()],
            })
            .with_parameter(ParameterDefinition {
                name: "tool_discovery_patterns".to_string(),
                description: "File patterns for discovering available tools".to_string(),
                param_type: ParameterType::Array(Box::new(ParameterType::String)),
                required: false,
                default_value: Some(vec!["*.rs", "*.lua"].into()),
                constraints: vec![],
                examples: vec![vec!["*.rs", "*.js"].into(), vec!["*.py", "*.lua"].into()],
            })
            .with_parameter(ParameterDefinition {
                name: "specialized_tools".to_string(),
                description: "Specific tools this agent specializes in".to_string(),
                param_type: ParameterType::Array(Box::new(ParameterType::String)),
                required: false,
                default_value: None,
                constraints: vec![],
                examples: vec![
                    vec!["file_reader", "text_processor"].into(),
                    vec!["web_scraper", "data_analyzer"].into(),
                ],
            });

        // Add common tool dependencies
        schema = schema
            .with_tool_dependency(ToolDependency {
                name: "calculator".to_string(),
                version: Some("1.0.0".to_string()),
                required: false,
                alternatives: vec!["math_tool".to_string()],
                config: HashMap::new(),
            })
            .with_tool_dependency(ToolDependency {
                name: "file_reader".to_string(),
                version: Some("1.0.0".to_string()),
                required: false,
                alternatives: vec!["text_reader".to_string()],
                config: HashMap::new(),
            })
            .with_tool_dependency(ToolDependency {
                name: "text_processor".to_string(),
                version: Some("1.0.0".to_string()),
                required: false,
                alternatives: vec!["string_utils".to_string()],
                config: HashMap::new(),
            });

        // Add capability requirements
        schema = schema
            .with_capability_requirement(CapabilityRequirement {
                name: "tool_execution".to_string(),
                min_level: 7,
                critical: true,
                usage_description: "Core capability for executing tools and handling results"
                    .to_string(),
            })
            .with_capability_requirement(CapabilityRequirement {
                name: "error_handling".to_string(),
                min_level: 6,
                critical: true,
                usage_description: "Handle tool execution errors and implement recovery strategies"
                    .to_string(),
            })
            .with_capability_requirement(CapabilityRequirement {
                name: "resource_management".to_string(),
                min_level: 5,
                critical: false,
                usage_description: "Manage tool resources and prevent resource exhaustion"
                    .to_string(),
            });

        // Set resource requirements
        schema = schema.with_resource_requirements(ResourceRequirements {
            memory: Some(256 * 1024 * 1024), // 256MB
            cpu: Some(30),                   // 30% CPU
            disk: Some(50 * 1024 * 1024),    // 50MB
            network: Some(5 * 1024 * 1024),  // 5MB/s
            max_execution_time: Some(600),   // 10 minutes
        });

        // Add configuration
        schema = schema
            .with_config("agent_type", "tool_agent".into())
            .with_config("supports_streaming", true.into())
            .with_config("supports_batch_processing", true.into());

        Self {
            schema,
            config: ToolAgentConfig::default(),
        }
    }

    /// Create Tool Agent template with custom configuration
    #[must_use]
    pub fn with_config(mut self, config: ToolAgentConfig) -> Self {
        self.config = config;
        self
    }

    /// Create specialized tool agent template
    #[must_use]
    pub fn specialized(tools: Vec<String>) -> Self {
        let mut template = Self::new();

        // Create unique ID based on tools
        let tools_str = tools.join("_").to_lowercase().replace('-', "_");
        template.schema.metadata.id = format!("tool_agent_specialized_{tools_str}");

        // Update schema for specialized agent
        template.schema.metadata.name = format!("Specialized Tool Agent ({})", tools.join(", "));
        template.schema.metadata.description =
            format!("Specialized agent for tools: {}", tools.join(", "));

        // Clear existing tool dependencies and add only specialized tools as required
        template.schema.tool_dependencies.clear();

        // Add tool dependencies for specialized tools
        for tool in &tools {
            template.schema = template.schema.with_tool_dependency(ToolDependency {
                name: tool.clone(),
                version: None,
                required: true,
                alternatives: vec![],
                config: HashMap::new(),
            });
        }

        // Set default specialized tools parameter
        if let Some(param) = template
            .schema
            .parameters
            .iter_mut()
            .find(|p| p.name == "specialized_tools")
        {
            param.default_value = Some(tools.into());
        }

        template
    }

    /// Create lightweight tool agent template
    #[must_use]
    pub fn lightweight() -> Self {
        let mut template = Self::new();

        // Update configuration for lightweight operation
        template.config.max_tools = 10;
        template.config.max_concurrent_tools = 2;
        template.config.enable_caching = false;

        // Update resource requirements
        template.schema = template
            .schema
            .with_resource_requirements(ResourceRequirements {
                memory: Some(64 * 1024 * 1024), // 64MB
                cpu: Some(15),                  // 15% CPU
                disk: Some(10 * 1024 * 1024),   // 10MB
                network: Some(1024 * 1024),     // 1MB/s
                max_execution_time: Some(300),  // 5 minutes
            });

        // Update metadata
        template.schema.metadata.id = "tool_agent_lightweight".to_string();
        template.schema.metadata.name = "Lightweight Tool Agent".to_string();
        template.schema.metadata.description =
            "Lightweight agent for basic tool execution tasks".to_string();
        template.schema.metadata.complexity = ComplexityLevel::Basic;

        template
    }

    /// Create batch processing tool agent template
    #[must_use]
    pub fn batch_processor() -> Self {
        let mut template = Self::new();

        // Update configuration for batch processing
        template.config.max_tools = 25;
        template.config.max_concurrent_tools = 10;
        template.config.enable_caching = true;
        template.config.tool_timeout = 1800; // 30 minutes

        // Update resource requirements
        template.schema = template
            .schema
            .with_resource_requirements(ResourceRequirements {
                memory: Some(512 * 1024 * 1024), // 512MB
                cpu: Some(50),                   // 50% CPU
                disk: Some(200 * 1024 * 1024),   // 200MB
                network: Some(10 * 1024 * 1024), // 10MB/s
                max_execution_time: Some(3600),  // 1 hour
            });

        // Update metadata
        template.schema.metadata.id = "tool_agent_batch_processor".to_string();
        template.schema.metadata.name = "Batch Processing Tool Agent".to_string();
        template.schema.metadata.description =
            "Agent optimized for batch processing and high-throughput tool execution".to_string();
        template.schema.metadata.complexity = ComplexityLevel::Advanced;

        // Add batch-specific configuration
        template.schema = template
            .schema
            .with_config("batch_mode", true.into())
            .with_config("batch_size", 100.into())
            .with_config("parallel_processing", true.into());

        template
    }

    /// Apply parameters to config
    fn apply_parameters_to_config(
        &self,
        config: &mut ToolAgentConfig,
        params: &HashMap<String, serde_json::Value>,
    ) -> Result<()> {
        if let Some(max_tools) = params.get("max_tools") {
            if let Some(value) = max_tools.as_u64() {
                config.max_tools = value as usize;
            }
        }

        if let Some(tool_timeout) = params.get("tool_timeout") {
            if let Some(value) = tool_timeout.as_u64() {
                config.tool_timeout = value;
            }
        }

        if let Some(enable_caching) = params.get("enable_caching") {
            if let Some(value) = enable_caching.as_bool() {
                config.enable_caching = value;
            }
        }

        if let Some(max_concurrent) = params.get("max_concurrent_tools") {
            if let Some(value) = max_concurrent.as_u64() {
                config.max_concurrent_tools = value as usize;
            }
        }

        if let Some(patterns) = params.get("tool_discovery_patterns") {
            if let Some(array) = patterns.as_array() {
                config.tool_discovery_patterns = array
                    .iter()
                    .filter_map(|v| v.as_str().map(std::string::ToString::to_string))
                    .collect();
            }
        }

        Ok(())
    }
}

impl Default for ToolAgentTemplate {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl AgentTemplate for ToolAgentTemplate {
    fn schema(&self) -> &TemplateSchema {
        &self.schema
    }

    async fn instantiate(
        &self,
        mut params: TemplateInstantiationParams,
    ) -> Result<TemplateInstantiationResult> {
        // Apply defaults
        self.apply_defaults(&mut params).await?;

        // Validate parameters
        self.validate_parameters(&params).await?;

        // Create agent-specific configuration
        let mut agent_config = self.config.clone();
        self.apply_parameters_to_config(&mut agent_config, &params.parameters)?;

        // Build final configuration
        let mut final_config = HashMap::new();
        final_config.insert("agent_type".to_string(), "tool_agent".into());
        final_config.insert(
            "max_tools".to_string(),
            (agent_config.max_tools as u64).into(),
        );
        final_config.insert("tool_timeout".to_string(), agent_config.tool_timeout.into());
        final_config.insert(
            "enable_caching".to_string(),
            agent_config.enable_caching.into(),
        );
        final_config.insert(
            "max_concurrent_tools".to_string(),
            (agent_config.max_concurrent_tools as u64).into(),
        );
        final_config.insert(
            "enable_error_recovery".to_string(),
            agent_config.enable_error_recovery.into(),
        );
        final_config.insert(
            "tool_discovery_patterns".to_string(),
            agent_config
                .tool_discovery_patterns
                .iter()
                .map(std::string::String::as_str)
                .collect::<Vec<_>>()
                .into(),
        );

        // Add specialized tools if specified
        if let Some(specialized_tools) = params.parameters.get("specialized_tools") {
            final_config.insert("specialized_tools".to_string(), specialized_tools.clone());
        }

        // Apply config overrides
        for (key, value) in params.config_overrides {
            final_config.insert(key, value);
        }

        // Get agent name
        let agent_name = params
            .parameters
            .get("agent_name")
            .and_then(|v| v.as_str())
            .unwrap_or(&params.agent_id)
            .to_string();

        // For now, return a mock result since we can't create actual BaseAgent instances
        // In a real implementation, this would create the actual tool agent
        let mock_agent =
            MockToolAgent::new(params.agent_id.clone(), agent_name, final_config.clone());

        Ok(TemplateInstantiationResult {
            agent: Box::new(mock_agent),
            template_schema: self.schema.clone(),
            applied_parameters: params.parameters,
            applied_config: final_config,
        })
    }

    async fn validate_custom_constraint(
        &self,
        parameter_name: &str,
        rule: &str,
        value: &serde_json::Value,
    ) -> Result<()> {
        match rule {
            "positive_integer" => {
                if let Some(num) = value.as_u64() {
                    if num == 0 {
                        return Err(anyhow!(
                            "Parameter '{}' must be a positive integer",
                            parameter_name
                        ));
                    }
                } else {
                    return Err(anyhow!(
                        "Parameter '{}' must be a positive integer",
                        parameter_name
                    ));
                }
            }
            "valid_file_pattern" => {
                if let Some(pattern) = value.as_str() {
                    if !pattern.contains('*') && !pattern.contains('?') {
                        return Err(anyhow!(
                            "Parameter '{}' must be a valid file pattern (contain * or ?)",
                            parameter_name
                        ));
                    }
                }
            }
            _ => {
                return Err(anyhow!(
                    "Unknown custom constraint '{}' for parameter '{}'",
                    rule,
                    parameter_name
                ));
            }
        }
        Ok(())
    }

    fn clone_template(&self) -> Box<dyn AgentTemplate> {
        Box::new(Self {
            schema: self.schema.clone(),
            config: self.config.clone(),
        })
    }
}

/// Mock tool agent for testing (replace with actual implementation)
#[allow(dead_code)]
struct MockToolAgent {
    id: String,
    name: String,
    config: HashMap<String, serde_json::Value>,
    metadata: ComponentMetadata,
}

impl MockToolAgent {
    fn new(id: String, name: String, config: HashMap<String, serde_json::Value>) -> Self {
        let metadata = ComponentMetadata {
            id: ComponentId::from_name(&id),
            name: name.clone(),
            version: Version::new(1, 0, 0),
            description: "Mock tool agent for template testing".to_string(),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };
        Self {
            id,
            name,
            config,
            metadata,
        }
    }
}

#[async_trait]
impl BaseAgent for MockToolAgent {
    fn metadata(&self) -> &ComponentMetadata {
        &self.metadata
    }

    async fn execute(
        &self,
        _input: AgentInput,
        _context: ExecutionContext,
    ) -> Result<AgentOutput, LLMSpellError> {
        Ok(AgentOutput {
            text: "Mock tool agent execution result".to_string(),
            media: vec![],
            tool_calls: vec![],
            metadata: OutputMetadata::default(),
            transfer_to: None,
        })
    }

    async fn validate_input(&self, _input: &AgentInput) -> Result<(), LLMSpellError> {
        Ok(())
    }

    async fn handle_error(&self, error: LLMSpellError) -> Result<AgentOutput, LLMSpellError> {
        Ok(AgentOutput {
            text: format!("Error handled by mock agent: {error}"),
            media: vec![],
            tool_calls: vec![],
            metadata: OutputMetadata::default(),
            transfer_to: None,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[tokio::test]
    async fn test_tool_agent_template_creation() {
        let template = ToolAgentTemplate::new();

        assert_eq!(template.schema().metadata.id, "tool_agent");
        assert_eq!(template.category(), &TemplateCategory::ToolExecution);
        assert_eq!(template.complexity(), &ComplexityLevel::Intermediate);

        let required_params = template.schema().required_parameters();
        assert_eq!(required_params.len(), 1);
        assert_eq!(required_params[0].name, "agent_name");
    }
    #[tokio::test]
    async fn test_specialized_tool_agent() {
        let tools = vec!["calculator".to_string(), "file_reader".to_string()];
        let template = ToolAgentTemplate::specialized(tools.clone());

        assert!(template.schema().metadata.name.contains("calculator"));
        assert!(template.schema().metadata.name.contains("file_reader"));

        // Check that specialized tools are required dependencies
        let calculator_dep = template.schema().get_tool_dependency("calculator");
        assert!(calculator_dep.is_some());
        assert!(calculator_dep.unwrap().required);
    }
    #[tokio::test]
    async fn test_lightweight_tool_agent() {
        let template = ToolAgentTemplate::lightweight();

        assert_eq!(template.config.max_tools, 10);
        assert_eq!(template.config.max_concurrent_tools, 2);
        assert!(!template.config.enable_caching);
        assert_eq!(template.complexity(), &ComplexityLevel::Basic);
    }
    #[tokio::test]
    async fn test_batch_processor_tool_agent() {
        let template = ToolAgentTemplate::batch_processor();

        assert_eq!(template.config.max_concurrent_tools, 10);
        assert_eq!(template.config.tool_timeout, 1800);
        assert_eq!(template.complexity(), &ComplexityLevel::Advanced);

        // Check batch-specific configuration
        let batch_mode = template.schema().template_config.get("batch_mode");
        assert_eq!(batch_mode, Some(&true.into()));
    }
    #[tokio::test]
    async fn test_parameter_validation() {
        let template = ToolAgentTemplate::new();

        // Test missing required parameter
        let params = TemplateInstantiationParams::new("test-agent".to_string());
        let result = template.validate_parameters(&params).await;
        assert!(result.is_err());

        // Test valid parameters
        let params = TemplateInstantiationParams::new("test-agent".to_string())
            .with_parameter("agent_name", "Test Tool Agent".into())
            .with_parameter("max_tools", 25.into())
            .with_parameter("enable_caching", false.into());

        let result = template.validate_parameters(&params).await;
        assert!(result.is_ok());
    }
    #[tokio::test]
    async fn test_template_instantiation() {
        let template = ToolAgentTemplate::new();

        let params = TemplateInstantiationParams::new("test-agent".to_string())
            .with_parameter("agent_name", "Test Tool Agent".into())
            .with_parameter("max_tools", 15.into());

        let result = template.instantiate(params).await;
        assert!(result.is_ok()); // Mock implementation succeeds

        let result = result.unwrap();
        // Check that our parameters were applied
        assert_eq!(
            result.applied_parameters.get("agent_name"),
            Some(&"Test Tool Agent".into())
        );
        assert_eq!(result.applied_parameters.get("max_tools"), Some(&15.into()));
        assert_eq!(result.applied_config.get("max_tools"), Some(&15.into()));
    }
    #[tokio::test]
    async fn test_tool_requirements() {
        let template = ToolAgentTemplate::new();

        let optional_tools = template.optional_tools();
        assert!(optional_tools.contains(&"calculator".to_string()));
        assert!(optional_tools.contains(&"file_reader".to_string()));

        let required_tools = template.required_tools();
        assert!(required_tools.is_empty()); // Base template has no required tools
    }
    #[tokio::test]
    async fn test_capability_support() {
        let template = ToolAgentTemplate::new();

        assert!(template.supports_capability("tool_execution"));
        assert!(template.supports_capability("error_handling"));
        assert!(template.supports_capability("resource_management"));
        assert!(!template.supports_capability("nonexistent_capability"));
    }
}
