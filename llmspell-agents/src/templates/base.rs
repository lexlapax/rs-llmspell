//! ABOUTME: Base template trait and factory for creating agents from template definitions
//! ABOUTME: Provides standardized template interface with validation, customization, and instantiation capabilities

use super::schema::{ParameterDefinition, TemplateSchema};
use crate::lifecycle::events::LifecycleEventSystem;
use crate::lifecycle::resources::ResourceManager;
use anyhow::{anyhow, Result};
use async_trait::async_trait;
use llmspell_core::BaseAgent;
use std::collections::HashMap;
use std::sync::Arc;

/// Template instantiation parameters
#[derive(Clone)]
pub struct TemplateInstantiationParams {
    /// Agent ID for the new instance
    pub agent_id: String,
    /// Parameter values for template customization
    pub parameters: HashMap<String, serde_json::Value>,
    /// Resource manager for the agent
    #[allow(dead_code)]
    pub resource_manager: Option<Arc<ResourceManager>>,
    /// Event system for lifecycle management
    #[allow(dead_code)]
    pub event_system: Option<Arc<LifecycleEventSystem>>,
    /// Custom configuration overrides
    pub config_overrides: HashMap<String, serde_json::Value>,
    /// Environment variables for the agent
    pub environment: HashMap<String, String>,
}

impl std::fmt::Debug for TemplateInstantiationParams {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TemplateInstantiationParams")
            .field("agent_id", &self.agent_id)
            .field("parameters", &self.parameters)
            .field("resource_manager", &"<Arc<ResourceManager>>")
            .field("event_system", &"<Arc<LifecycleEventSystem>>")
            .field("config_overrides", &self.config_overrides)
            .field("environment", &self.environment)
            .finish()
    }
}

impl TemplateInstantiationParams {
    /// Create new instantiation parameters
    pub fn new(agent_id: String) -> Self {
        Self {
            agent_id,
            parameters: HashMap::new(),
            resource_manager: None,
            event_system: None,
            config_overrides: HashMap::new(),
            environment: HashMap::new(),
        }
    }

    /// Add parameter value
    pub fn with_parameter(mut self, name: &str, value: serde_json::Value) -> Self {
        self.parameters.insert(name.to_string(), value);
        self
    }

    /// Set resource manager
    pub fn with_resource_manager(mut self, resource_manager: Arc<ResourceManager>) -> Self {
        self.resource_manager = Some(resource_manager);
        self
    }

    /// Set event system
    pub fn with_event_system(mut self, event_system: Arc<LifecycleEventSystem>) -> Self {
        self.event_system = Some(event_system);
        self
    }

    /// Add config override
    pub fn with_config_override(mut self, key: &str, value: serde_json::Value) -> Self {
        self.config_overrides.insert(key.to_string(), value);
        self
    }

    /// Add environment variable
    pub fn with_environment(mut self, key: &str, value: &str) -> Self {
        self.environment.insert(key.to_string(), value.to_string());
        self
    }
}

/// Template instantiation result
pub struct TemplateInstantiationResult {
    /// Created agent instance
    pub agent: Box<dyn BaseAgent>,
    /// Template schema used
    pub template_schema: TemplateSchema,
    /// Applied parameter values
    pub applied_parameters: HashMap<String, serde_json::Value>,
    /// Applied configuration
    pub applied_config: HashMap<String, serde_json::Value>,
}

/// Base trait for all agent templates
#[async_trait]
pub trait AgentTemplate: Send + Sync {
    /// Get template schema
    fn schema(&self) -> &TemplateSchema;

    /// Validate instantiation parameters
    async fn validate_parameters(&self, params: &TemplateInstantiationParams) -> Result<()> {
        // Default validation implementation
        let schema = self.schema();

        // Check required parameters
        for param in schema.required_parameters() {
            if !params.parameters.contains_key(&param.name) {
                return Err(anyhow!("Missing required parameter: {}", param.name));
            }
        }

        // Validate parameter types and constraints
        for (name, value) in &params.parameters {
            if let Some(param_def) = schema.get_parameter(name) {
                self.validate_parameter_value(param_def, value).await?;
            }
        }

        Ok(())
    }

    /// Validate individual parameter value
    async fn validate_parameter_value(
        &self,
        param_def: &ParameterDefinition,
        value: &serde_json::Value,
    ) -> Result<()> {
        use super::schema::{ParameterConstraint, ParameterType};

        // Check parameter type
        match &param_def.param_type {
            ParameterType::String => {
                if !value.is_string() {
                    return Err(anyhow!("Parameter '{}' must be a string", param_def.name));
                }
            }
            ParameterType::Integer => {
                if !value.is_i64() && !value.is_u64() {
                    return Err(anyhow!("Parameter '{}' must be an integer", param_def.name));
                }
            }
            ParameterType::Float => {
                if !value.is_f64() && !value.is_i64() && !value.is_u64() {
                    return Err(anyhow!("Parameter '{}' must be a number", param_def.name));
                }
            }
            ParameterType::Boolean => {
                if !value.is_boolean() {
                    return Err(anyhow!("Parameter '{}' must be a boolean", param_def.name));
                }
            }
            ParameterType::Array(_) => {
                if !value.is_array() {
                    return Err(anyhow!("Parameter '{}' must be an array", param_def.name));
                }
            }
            ParameterType::Object(_) => {
                if !value.is_object() {
                    return Err(anyhow!("Parameter '{}' must be an object", param_def.name));
                }
            }
            ParameterType::Enum(allowed_values) => {
                if let Some(str_value) = value.as_str() {
                    if !allowed_values.contains(&str_value.to_string()) {
                        return Err(anyhow!(
                            "Parameter '{}' must be one of: {:?}",
                            param_def.name,
                            allowed_values
                        ));
                    }
                } else {
                    return Err(anyhow!(
                        "Parameter '{}' must be a string enum value",
                        param_def.name
                    ));
                }
            }
            _ => {
                // Custom validation for other types can be implemented by subclasses
            }
        }

        // Check constraints
        for constraint in &param_def.constraints {
            match constraint {
                ParameterConstraint::MinValue(min) => {
                    if let Some(num) = value.as_f64() {
                        if num < *min {
                            return Err(anyhow!(
                                "Parameter '{}' must be >= {}",
                                param_def.name,
                                min
                            ));
                        }
                    }
                }
                ParameterConstraint::MaxValue(max) => {
                    if let Some(num) = value.as_f64() {
                        if num > *max {
                            return Err(anyhow!(
                                "Parameter '{}' must be <= {}",
                                param_def.name,
                                max
                            ));
                        }
                    }
                }
                ParameterConstraint::MinLength(min_len) => {
                    let length = if let Some(s) = value.as_str() {
                        s.len()
                    } else if let Some(arr) = value.as_array() {
                        arr.len()
                    } else {
                        0
                    };
                    if length < *min_len {
                        return Err(anyhow!(
                            "Parameter '{}' must have minimum length {}",
                            param_def.name,
                            min_len
                        ));
                    }
                }
                ParameterConstraint::MaxLength(max_len) => {
                    let length = if let Some(s) = value.as_str() {
                        s.len()
                    } else if let Some(arr) = value.as_array() {
                        arr.len()
                    } else {
                        0
                    };
                    if length > *max_len {
                        return Err(anyhow!(
                            "Parameter '{}' must have maximum length {}",
                            param_def.name,
                            max_len
                        ));
                    }
                }
                ParameterConstraint::Pattern(pattern) => {
                    if let Some(s) = value.as_str() {
                        let regex = regex::Regex::new(pattern).map_err(|e| {
                            anyhow!(
                                "Invalid regex pattern for parameter '{}': {}",
                                param_def.name,
                                e
                            )
                        })?;
                        if !regex.is_match(s) {
                            return Err(anyhow!(
                                "Parameter '{}' does not match pattern: {}",
                                param_def.name,
                                pattern
                            ));
                        }
                    }
                }
                ParameterConstraint::Custom(rule) => {
                    // Custom constraint validation can be implemented by subclasses
                    self.validate_custom_constraint(&param_def.name, rule, value)
                        .await?;
                }
            }
        }

        Ok(())
    }

    /// Validate custom constraints (override in implementations)
    async fn validate_custom_constraint(
        &self,
        _parameter_name: &str,
        _rule: &str,
        _value: &serde_json::Value,
    ) -> Result<()> {
        Ok(())
    }

    /// Apply default values to parameters
    async fn apply_defaults(&self, params: &mut TemplateInstantiationParams) -> Result<()> {
        let schema = self.schema();

        for param in &schema.parameters {
            if !params.parameters.contains_key(&param.name) {
                if let Some(default) = &param.default_value {
                    params
                        .parameters
                        .insert(param.name.clone(), default.clone());
                }
            }
        }

        Ok(())
    }

    /// Create agent instance from template
    async fn instantiate(
        &self,
        mut params: TemplateInstantiationParams,
    ) -> Result<TemplateInstantiationResult>;

    /// Get template category
    fn category(&self) -> &super::schema::TemplateCategory {
        &self.schema().metadata.category
    }

    /// Get template complexity
    fn complexity(&self) -> &super::schema::ComplexityLevel {
        &self.schema().metadata.complexity
    }

    /// Check if template supports a specific capability
    fn supports_capability(&self, capability: &str) -> bool {
        self.schema()
            .capability_requirements
            .iter()
            .any(|req| req.name == capability)
    }

    /// Get required tools for this template
    fn required_tools(&self) -> Vec<String> {
        self.schema()
            .tool_dependencies
            .iter()
            .filter(|dep| dep.required)
            .map(|dep| dep.name.clone())
            .collect()
    }

    /// Get optional tools for this template
    fn optional_tools(&self) -> Vec<String> {
        self.schema()
            .tool_dependencies
            .iter()
            .filter(|dep| !dep.required)
            .map(|dep| dep.name.clone())
            .collect()
    }

    /// Clone template (for factory pattern)
    fn clone_template(&self) -> Box<dyn AgentTemplate>;
}

/// Template factory for managing and instantiating templates
#[derive(Default)]
pub struct TemplateFactory {
    /// Registered templates by ID
    templates: HashMap<String, Box<dyn AgentTemplate>>,
    /// Templates by category
    templates_by_category: HashMap<super::schema::TemplateCategory, Vec<String>>,
}

impl TemplateFactory {
    /// Create new template factory
    pub fn new() -> Self {
        Self::default()
    }

    /// Register a template
    pub fn register_template(&mut self, template: Box<dyn AgentTemplate>) -> Result<()> {
        let template_id = template.schema().metadata.id.clone();
        let category = template.category().clone();

        // Check for duplicate IDs
        if self.templates.contains_key(&template_id) {
            return Err(anyhow!(
                "Template with ID '{}' already registered",
                template_id
            ));
        }

        // Add to category index
        self.templates_by_category
            .entry(category)
            .or_default()
            .push(template_id.clone());

        // Register template
        self.templates.insert(template_id, template);

        Ok(())
    }

    /// Get template by ID
    pub fn get_template(&self, template_id: &str) -> Option<&dyn AgentTemplate> {
        self.templates.get(template_id).map(|t| t.as_ref())
    }

    /// Get all templates in category
    pub fn get_templates_by_category(
        &self,
        category: &super::schema::TemplateCategory,
    ) -> Vec<&dyn AgentTemplate> {
        if let Some(template_ids) = self.templates_by_category.get(category) {
            template_ids
                .iter()
                .filter_map(|id| self.get_template(id))
                .collect()
        } else {
            Vec::new()
        }
    }

    /// Get all template IDs
    pub fn list_templates(&self) -> Vec<String> {
        self.templates.keys().cloned().collect()
    }

    /// Get template schemas
    pub fn get_template_schemas(&self) -> Vec<&TemplateSchema> {
        self.templates.values().map(|t| t.schema()).collect()
    }

    /// Find templates by keyword
    pub fn find_templates(&self, keyword: &str) -> Vec<&dyn AgentTemplate> {
        let keyword_lower = keyword.to_lowercase();
        self.templates
            .values()
            .filter(|template| {
                let schema = template.schema();
                schema.metadata.name.to_lowercase().contains(&keyword_lower)
                    || schema
                        .metadata
                        .description
                        .to_lowercase()
                        .contains(&keyword_lower)
                    || schema
                        .metadata
                        .keywords
                        .iter()
                        .any(|k| k.to_lowercase().contains(&keyword_lower))
            })
            .map(|t| t.as_ref())
            .collect()
    }

    /// Instantiate template by ID
    pub async fn instantiate_template(
        &self,
        template_id: &str,
        params: TemplateInstantiationParams,
    ) -> Result<TemplateInstantiationResult> {
        let template = self
            .get_template(template_id)
            .ok_or_else(|| anyhow!("Template not found: {}", template_id))?;

        template.instantiate(params).await
    }

    /// Validate template parameters without instantiation
    pub async fn validate_template_parameters(
        &self,
        template_id: &str,
        params: &TemplateInstantiationParams,
    ) -> Result<()> {
        let template = self
            .get_template(template_id)
            .ok_or_else(|| anyhow!("Template not found: {}", template_id))?;

        template.validate_parameters(params).await
    }

    /// Get template count
    pub fn template_count(&self) -> usize {
        self.templates.len()
    }

    /// Check if template exists
    pub fn has_template(&self, template_id: &str) -> bool {
        self.templates.contains_key(template_id)
    }

    /// Unregister template
    pub fn unregister_template(&mut self, template_id: &str) -> Result<()> {
        if let Some(template) = self.templates.remove(template_id) {
            let category = template.category().clone();

            // Remove from category index
            if let Some(template_ids) = self.templates_by_category.get_mut(&category) {
                template_ids.retain(|id| id != template_id);
                if template_ids.is_empty() {
                    self.templates_by_category.remove(&category);
                }
            }

            Ok(())
        } else {
            Err(anyhow!("Template not found: {}", template_id))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::templates::schema::{
        ComplexityLevel, ParameterConstraint, ParameterType, TemplateCategory, TemplateMetadata,
    };

    // Mock template for testing
    struct MockTemplate {
        schema: TemplateSchema,
    }

    impl MockTemplate {
        fn new() -> Self {
            let metadata = TemplateMetadata {
                id: "mock_template".to_string(),
                name: "Mock Template".to_string(),
                version: "1.0.0".to_string(),
                description: "A mock template for testing".to_string(),
                author: "Test Author".to_string(),
                license: "MIT".to_string(),
                repository: None,
                documentation: None,
                keywords: vec!["test".to_string(), "mock".to_string()],
                category: TemplateCategory::Utility,
                complexity: ComplexityLevel::Basic,
            };

            let mut schema = TemplateSchema::new(metadata);

            // Add a required string parameter
            let param = ParameterDefinition {
                name: "test_param".to_string(),
                description: "A test parameter".to_string(),
                param_type: ParameterType::String,
                required: true,
                default_value: None,
                constraints: vec![ParameterConstraint::MinLength(1)],
                examples: vec!["example".into()],
            };

            schema = schema.with_parameter(param);

            Self { schema }
        }
    }

    #[async_trait]
    impl AgentTemplate for MockTemplate {
        fn schema(&self) -> &TemplateSchema {
            &self.schema
        }

        async fn instantiate(
            &self,
            _params: TemplateInstantiationParams,
        ) -> Result<TemplateInstantiationResult> {
            // Mock implementation - would create actual agent in real template
            Err(anyhow!("Mock template cannot create real agents"))
        }

        fn clone_template(&self) -> Box<dyn AgentTemplate> {
            Box::new(MockTemplate {
                schema: self.schema.clone(),
            })
        }
    }

    #[tokio::test]
    async fn test_template_factory_registration() {
        let mut factory = TemplateFactory::new();
        let template = Box::new(MockTemplate::new());

        assert_eq!(factory.template_count(), 0);

        factory.register_template(template).unwrap();

        assert_eq!(factory.template_count(), 1);
        assert!(factory.has_template("mock_template"));
        assert!(!factory.has_template("nonexistent"));
    }

    #[tokio::test]
    async fn test_template_factory_categories() {
        let mut factory = TemplateFactory::new();
        let template = Box::new(MockTemplate::new());

        factory.register_template(template).unwrap();

        let utility_templates = factory.get_templates_by_category(&TemplateCategory::Utility);
        assert_eq!(utility_templates.len(), 1);

        let monitoring_templates = factory.get_templates_by_category(&TemplateCategory::Monitoring);
        assert_eq!(monitoring_templates.len(), 0);
    }

    #[tokio::test]
    async fn test_template_factory_search() {
        let mut factory = TemplateFactory::new();
        let template = Box::new(MockTemplate::new());

        factory.register_template(template).unwrap();

        let found = factory.find_templates("mock");
        assert_eq!(found.len(), 1);

        let found = factory.find_templates("test");
        assert_eq!(found.len(), 1);

        let found = factory.find_templates("nonexistent");
        assert_eq!(found.len(), 0);
    }

    #[tokio::test]
    async fn test_parameter_validation() {
        let template = MockTemplate::new();

        // Test missing required parameter
        let params = TemplateInstantiationParams::new("test-agent".to_string());
        let result = template.validate_parameters(&params).await;
        assert!(result.is_err());

        // Test valid parameter
        let params = TemplateInstantiationParams::new("test-agent".to_string())
            .with_parameter("test_param", "valid_value".into());
        let result = template.validate_parameters(&params).await;
        assert!(result.is_ok());

        // Test invalid parameter type
        let params = TemplateInstantiationParams::new("test-agent".to_string())
            .with_parameter("test_param", 123.into());
        let result = template.validate_parameters(&params).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_template_metadata() {
        let template = MockTemplate::new();

        assert_eq!(template.category(), &TemplateCategory::Utility);
        assert_eq!(template.complexity(), &ComplexityLevel::Basic);
        assert_eq!(template.required_tools().len(), 0);
        assert_eq!(template.optional_tools().len(), 0);
    }

    #[tokio::test]
    async fn test_template_unregistration() {
        let mut factory = TemplateFactory::new();
        let template = Box::new(MockTemplate::new());

        factory.register_template(template).unwrap();
        assert_eq!(factory.template_count(), 1);

        factory.unregister_template("mock_template").unwrap();
        assert_eq!(factory.template_count(), 0);

        // Test unregistering non-existent template
        let result = factory.unregister_template("nonexistent");
        assert!(result.is_err());
    }
}
