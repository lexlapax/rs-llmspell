//! ABOUTME: Template customization API for dynamic template modification and extension
//! ABOUTME: Provides builders, mixins, and runtime customization capabilities for agent templates

use super::base::{AgentTemplate, TemplateInstantiationParams};
use super::schema::{
    CapabilityRequirement, ParameterConstraint, ParameterDefinition, ParameterType,
    ResourceRequirements, TemplateSchema, ToolDependency,
};
use anyhow::{anyhow, Result};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Template customization options
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TemplateCustomization {
    /// Additional parameters to add
    #[serde(default)]
    pub additional_parameters: Vec<ParameterDefinition>,
    /// Parameter overrides (update existing parameters)
    #[serde(default)]
    pub parameter_overrides: HashMap<String, ParameterDefinition>,
    /// Additional tool dependencies
    #[serde(default)]
    pub additional_tools: Vec<ToolDependency>,
    /// Tool dependency overrides
    #[serde(default)]
    pub tool_overrides: HashMap<String, ToolDependency>,
    /// Additional capabilities
    #[serde(default)]
    pub additional_capabilities: Vec<CapabilityRequirement>,
    /// Resource requirement overrides
    #[serde(default)]
    pub resource_overrides: Option<ResourceRequirements>,
    /// Configuration additions
    #[serde(default)]
    pub config_additions: HashMap<String, serde_json::Value>,
    /// Metadata overrides
    #[serde(default)]
    pub metadata_overrides: HashMap<String, String>,
}

/// Template customizer for applying customizations
pub struct TemplateCustomizer {
    base_template: Box<dyn AgentTemplate>,
    customization: TemplateCustomization,
}

impl TemplateCustomizer {
    /// Create new customizer for a template
    pub fn new(template: Box<dyn AgentTemplate>) -> Self {
        Self {
            base_template: template,
            customization: TemplateCustomization::default(),
        }
    }

    /// Add a parameter to the template
    pub fn add_parameter(mut self, param: ParameterDefinition) -> Self {
        self.customization.additional_parameters.push(param);
        self
    }

    /// Override an existing parameter
    pub fn override_parameter(mut self, name: &str, param: ParameterDefinition) -> Self {
        self.customization
            .parameter_overrides
            .insert(name.to_string(), param);
        self
    }

    /// Add a tool dependency
    pub fn add_tool_dependency(mut self, tool: ToolDependency) -> Self {
        self.customization.additional_tools.push(tool);
        self
    }

    /// Override a tool dependency
    pub fn override_tool(mut self, name: &str, tool: ToolDependency) -> Self {
        self.customization
            .tool_overrides
            .insert(name.to_string(), tool);
        self
    }

    /// Add a capability requirement
    pub fn add_capability(mut self, capability: CapabilityRequirement) -> Self {
        self.customization.additional_capabilities.push(capability);
        self
    }

    /// Override resource requirements
    pub fn override_resources(mut self, resources: ResourceRequirements) -> Self {
        self.customization.resource_overrides = Some(resources);
        self
    }

    /// Add configuration
    pub fn add_config(mut self, key: &str, value: serde_json::Value) -> Self {
        self.customization
            .config_additions
            .insert(key.to_string(), value);
        self
    }

    /// Override metadata
    pub fn override_metadata(mut self, key: &str, value: &str) -> Self {
        self.customization
            .metadata_overrides
            .insert(key.to_string(), value.to_string());
        self
    }

    /// Build the customized template
    pub fn build(self) -> Box<dyn AgentTemplate> {
        Box::new(CustomizedTemplate {
            base_template: self.base_template,
            customization: self.customization,
        })
    }
}

/// A customized template that wraps a base template with customizations
struct CustomizedTemplate {
    base_template: Box<dyn AgentTemplate>,
    customization: TemplateCustomization,
}

#[async_trait]
impl AgentTemplate for CustomizedTemplate {
    fn schema(&self) -> &TemplateSchema {
        // This is a bit of a hack - we return the base schema
        // In a real implementation, we'd create a modified schema
        self.base_template.schema()
    }

    async fn validate_parameters(&self, params: &TemplateInstantiationParams) -> Result<()> {
        // First validate base parameters
        self.base_template.validate_parameters(params).await?;

        // Then validate additional parameters
        for param in &self.customization.additional_parameters {
            if param.required && !params.parameters.contains_key(&param.name) {
                return Err(anyhow!("Missing required parameter: {}", param.name));
            }

            if let Some(value) = params.parameters.get(&param.name) {
                self.validate_parameter_value(param, value).await?;
            }
        }

        // Validate overridden parameters
        for (name, param) in &self.customization.parameter_overrides {
            if let Some(value) = params.parameters.get(name) {
                self.validate_parameter_value(param, value).await?;
            }
        }

        Ok(())
    }

    async fn instantiate(
        &self,
        _params: TemplateInstantiationParams,
    ) -> Result<super::base::TemplateInstantiationResult> {
        // Delegate to base template with potential modifications
        self.base_template.instantiate(_params).await
    }

    fn clone_template(&self) -> Box<dyn AgentTemplate> {
        Box::new(CustomizedTemplate {
            base_template: self.base_template.clone_template(),
            customization: self.customization.clone(),
        })
    }
}

/// Template mixin for adding common functionality
pub trait TemplateMixin {
    /// Add logging capabilities
    fn with_logging(self) -> Self;

    /// Add metrics collection
    fn with_metrics(self) -> Self;

    /// Add retry logic
    fn with_retry(self, max_retries: u32) -> Self;

    /// Add caching
    fn with_caching(self) -> Self;

    /// Add rate limiting
    fn with_rate_limiting(self, requests_per_minute: u32) -> Self;
}

impl TemplateMixin for TemplateCustomizer {
    fn with_logging(self) -> Self {
        self.add_tool_dependency(ToolDependency {
            name: "logger".to_string(),
            version: Some("1.0.0".to_string()),
            required: true,
            alternatives: vec!["structured_logger".to_string()],
            config: HashMap::from([
                ("level".to_string(), "info".into()),
                ("format".to_string(), "json".into()),
            ]),
        })
        .add_config("enable_logging", true.into())
        .add_config("log_level", "info".into())
    }

    fn with_metrics(self) -> Self {
        self.add_tool_dependency(ToolDependency {
            name: "metrics_collector".to_string(),
            version: Some("1.0.0".to_string()),
            required: true,
            alternatives: vec!["telemetry_agent".to_string()],
            config: HashMap::new(),
        })
        .add_capability(CapabilityRequirement {
            name: "metrics_collection".to_string(),
            min_level: 5,
            critical: false,
            usage_description: "Collect and report performance metrics".to_string(),
        })
        .add_config("enable_metrics", true.into())
        .add_config("metrics_interval_seconds", 60.into())
    }

    fn with_retry(self, max_retries: u32) -> Self {
        self.add_config("enable_retry", true.into())
            .add_config("max_retries", max_retries.into())
            .add_config("retry_backoff_ms", 1000.into())
            .add_config("retry_max_backoff_ms", 60000.into())
    }

    fn with_caching(self) -> Self {
        self.add_tool_dependency(ToolDependency {
            name: "cache_manager".to_string(),
            version: Some("1.0.0".to_string()),
            required: false,
            alternatives: vec!["memory_cache".to_string(), "redis_cache".to_string()],
            config: HashMap::from([
                ("cache_size_mb".to_string(), 100.into()),
                ("ttl_seconds".to_string(), 3600.into()),
            ]),
        })
        .add_config("enable_caching", true.into())
        .add_config("cache_ttl_seconds", 3600.into())
    }

    fn with_rate_limiting(self, requests_per_minute: u32) -> Self {
        self.add_config("enable_rate_limiting", true.into())
            .add_config("rate_limit_per_minute", requests_per_minute.into())
            .add_config(
                "rate_limit_burst_size",
                (requests_per_minute / 10).max(1).into(),
            )
    }
}

/// Template builder for creating templates programmatically
pub struct TemplateBuilder {
    schema: TemplateSchema,
}

impl TemplateBuilder {
    /// Create new template builder
    pub fn new(metadata: super::schema::TemplateMetadata) -> Self {
        Self {
            schema: TemplateSchema::new(metadata),
        }
    }

    /// Add parameter
    pub fn parameter(mut self, param: ParameterDefinition) -> Self {
        self.schema = self.schema.with_parameter(param);
        self
    }

    /// Add required string parameter
    pub fn string_param(self, name: &str, description: &str) -> Self {
        self.parameter(ParameterDefinition {
            name: name.to_string(),
            description: description.to_string(),
            param_type: ParameterType::String,
            required: true,
            default_value: None,
            constraints: vec![],
            examples: vec![],
        })
    }

    /// Add optional integer parameter with default
    pub fn int_param(self, name: &str, description: &str, default: i64) -> Self {
        self.parameter(ParameterDefinition {
            name: name.to_string(),
            description: description.to_string(),
            param_type: ParameterType::Integer,
            required: false,
            default_value: Some(default.into()),
            constraints: vec![],
            examples: vec![],
        })
    }

    /// Add optional boolean parameter with default
    pub fn bool_param(self, name: &str, description: &str, default: bool) -> Self {
        self.parameter(ParameterDefinition {
            name: name.to_string(),
            description: description.to_string(),
            param_type: ParameterType::Boolean,
            required: false,
            default_value: Some(default.into()),
            constraints: vec![],
            examples: vec![],
        })
    }

    /// Add enum parameter
    pub fn enum_param(
        self,
        name: &str,
        description: &str,
        values: Vec<String>,
        default: Option<String>,
    ) -> Self {
        self.parameter(ParameterDefinition {
            name: name.to_string(),
            description: description.to_string(),
            param_type: ParameterType::Enum(values),
            required: default.is_none(),
            default_value: default.map(|v| v.into()),
            constraints: vec![],
            examples: vec![],
        })
    }

    /// Add tool dependency
    pub fn tool(mut self, name: &str, required: bool) -> Self {
        self.schema = self.schema.with_tool_dependency(ToolDependency {
            name: name.to_string(),
            version: None,
            required,
            alternatives: vec![],
            config: HashMap::new(),
        });
        self
    }

    /// Add capability
    pub fn capability(mut self, name: &str, min_level: u8, critical: bool) -> Self {
        self.schema = self
            .schema
            .with_capability_requirement(CapabilityRequirement {
                name: name.to_string(),
                min_level,
                critical,
                usage_description: format!("Capability for {}", name),
            });
        self
    }

    /// Set resource requirements
    pub fn resources(mut self, memory_mb: u64, cpu_percent: u8) -> Self {
        self.schema = self
            .schema
            .with_resource_requirements(ResourceRequirements {
                memory: Some(memory_mb * 1024 * 1024),
                cpu: Some(cpu_percent),
                disk: None,
                network: None,
                max_execution_time: None,
            });
        self
    }

    /// Add configuration
    pub fn config(mut self, key: &str, value: serde_json::Value) -> Self {
        self.schema = self.schema.with_config(key, value);
        self
    }

    /// Build into a basic template
    pub fn build(self) -> Result<BasicTemplate> {
        self.schema.validate()?;
        Ok(BasicTemplate {
            schema: self.schema,
        })
    }
}

/// Basic template implementation for programmatically created templates
pub struct BasicTemplate {
    schema: TemplateSchema,
}

#[async_trait]
impl AgentTemplate for BasicTemplate {
    fn schema(&self) -> &TemplateSchema {
        &self.schema
    }

    async fn instantiate(
        &self,
        _params: TemplateInstantiationParams,
    ) -> Result<super::base::TemplateInstantiationResult> {
        Err(anyhow!(
            "BasicTemplate cannot be instantiated directly - extend with custom implementation"
        ))
    }

    fn clone_template(&self) -> Box<dyn AgentTemplate> {
        Box::new(BasicTemplate {
            schema: self.schema.clone(),
        })
    }
}

/// Create a custom parameter with validation
pub fn create_validated_parameter(
    name: &str,
    description: &str,
    param_type: ParameterType,
    constraints: Vec<ParameterConstraint>,
) -> ParameterDefinition {
    ParameterDefinition {
        name: name.to_string(),
        description: description.to_string(),
        param_type,
        required: true,
        default_value: None,
        constraints,
        examples: vec![],
    }
}

/// Create a range-constrained numeric parameter
pub fn create_range_parameter(
    name: &str,
    description: &str,
    min: f64,
    max: f64,
    default: Option<f64>,
) -> ParameterDefinition {
    ParameterDefinition {
        name: name.to_string(),
        description: description.to_string(),
        param_type: ParameterType::Float,
        required: default.is_none(),
        default_value: default.map(|v| v.into()),
        constraints: vec![
            ParameterConstraint::MinValue(min),
            ParameterConstraint::MaxValue(max),
        ],
        examples: vec![min.into(), ((min + max) / 2.0).into(), max.into()],
    }
}

/// Create a pattern-validated string parameter
pub fn create_pattern_parameter(
    name: &str,
    description: &str,
    pattern: &str,
    examples: Vec<String>,
) -> ParameterDefinition {
    ParameterDefinition {
        name: name.to_string(),
        description: description.to_string(),
        param_type: ParameterType::String,
        required: true,
        default_value: None,
        constraints: vec![ParameterConstraint::Pattern(pattern.to_string())],
        examples: examples.into_iter().map(|e| e.into()).collect(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::templates::schema::{ComplexityLevel, TemplateCategory, TemplateMetadata};
    use crate::templates::tool_agent::ToolAgentTemplate;

    #[tokio::test]
    async fn test_template_customizer() {
        let base_template = Box::new(ToolAgentTemplate::new());

        let customizer = TemplateCustomizer::new(base_template)
            .add_parameter(ParameterDefinition {
                name: "custom_param".to_string(),
                description: "A custom parameter".to_string(),
                param_type: ParameterType::String,
                required: false,
                default_value: Some("default".into()),
                constraints: vec![],
                examples: vec![],
            })
            .add_tool_dependency(ToolDependency {
                name: "custom_tool".to_string(),
                version: None,
                required: false,
                alternatives: vec![],
                config: HashMap::new(),
            })
            .add_config("custom_config", "value".into());

        let customized = customizer.build();

        // Verify the template still works
        let params = TemplateInstantiationParams::new("test-agent".to_string())
            .with_parameter("agent_name", "Test Agent".into())
            .with_parameter("custom_param", "custom_value".into());

        let result = customized.validate_parameters(&params).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_template_mixins() {
        let base_template = Box::new(ToolAgentTemplate::new());

        let customizer = TemplateCustomizer::new(base_template)
            .with_logging()
            .with_metrics()
            .with_retry(3)
            .with_caching()
            .with_rate_limiting(100);

        assert_eq!(customizer.customization.additional_tools.len(), 3); // logging, metrics, caching
        assert_eq!(
            customizer
                .customization
                .config_additions
                .get("enable_logging"),
            Some(&true.into())
        );
        assert_eq!(
            customizer.customization.config_additions.get("max_retries"),
            Some(&3.into())
        );
        assert_eq!(
            customizer
                .customization
                .config_additions
                .get("rate_limit_per_minute"),
            Some(&100.into())
        );
    }

    #[tokio::test]
    async fn test_template_builder() {
        let metadata = TemplateMetadata {
            id: "custom_template".to_string(),
            name: "Custom Template".to_string(),
            version: "1.0.0".to_string(),
            description: "A custom template".to_string(),
            author: "Test".to_string(),
            license: "MIT".to_string(),
            repository: None,
            documentation: None,
            keywords: vec![],
            category: TemplateCategory::Utility,
            complexity: ComplexityLevel::Basic,
        };

        let template = TemplateBuilder::new(metadata)
            .string_param("name", "Agent name")
            .int_param("max_tasks", "Maximum concurrent tasks", 5)
            .bool_param("enable_logging", "Enable logging", true)
            .enum_param(
                "mode",
                "Operation mode",
                vec!["fast".to_string(), "normal".to_string(), "slow".to_string()],
                Some("normal".to_string()),
            )
            .tool("calculator", true)
            .tool("file_reader", false)
            .capability("task_execution", 7, true)
            .resources(256, 30)
            .config("version", "1.0".into())
            .build()
            .unwrap();

        assert_eq!(template.schema().parameters.len(), 4);
        assert_eq!(template.schema().tool_dependencies.len(), 2);
        assert_eq!(template.schema().capability_requirements.len(), 1);
    }

    #[test]
    fn test_parameter_helpers() {
        let range_param =
            create_range_parameter("temperature", "LLM temperature", 0.0, 2.0, Some(0.7));
        assert_eq!(range_param.name, "temperature");
        assert_eq!(range_param.constraints.len(), 2);
        assert_eq!(range_param.default_value, Some(0.7.into()));

        let pattern_param = create_pattern_parameter(
            "email",
            "Email address",
            r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$",
            vec!["user@example.com".to_string(), "admin@test.org".to_string()],
        );
        assert_eq!(pattern_param.name, "email");
        assert_eq!(pattern_param.constraints.len(), 1);
        assert_eq!(pattern_param.examples.len(), 2);
    }

    #[tokio::test]
    async fn test_customization_overrides() {
        let base_template = Box::new(ToolAgentTemplate::new());

        let customizer = TemplateCustomizer::new(base_template)
            .override_parameter(
                "max_tools",
                ParameterDefinition {
                    name: "max_tools".to_string(),
                    description: "Maximum tools (customized)".to_string(),
                    param_type: ParameterType::Integer,
                    required: true, // Changed from optional to required
                    default_value: None,
                    constraints: vec![ParameterConstraint::MaxValue(10.0)],
                    examples: vec![5.into(), 10.into()],
                },
            )
            .override_resources(ResourceRequirements {
                memory: Some(64 * 1024 * 1024), // 64MB instead of default
                cpu: Some(10),                  // 10% instead of default
                disk: None,
                network: None,
                max_execution_time: None,
            });

        let customized = customizer.build();

        // Test that override validation works
        let params = TemplateInstantiationParams::new("test-agent".to_string())
            .with_parameter("agent_name", "Test Agent".into())
            .with_parameter("max_tools", 20.into()); // Should fail - exceeds max of 10

        let result = customized.validate_parameters(&params).await;
        assert!(result.is_err());
    }
}
