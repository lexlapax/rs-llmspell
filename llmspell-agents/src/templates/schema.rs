//! ABOUTME: Template definition schema for agent templates
//! ABOUTME: Provides structured definition format for agent templates with validation and customization support

use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;

/// Template schema version for compatibility tracking
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum SchemaVersion {
    /// Initial template schema version
    #[default]
    V1,
}

impl fmt::Display for SchemaVersion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::V1 => write!(f, "v1"),
        }
    }
}

/// Template category for organizing templates
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TemplateCategory {
    /// Agents that primarily execute tools
    ToolExecution,
    /// Agents that orchestrate other agents or workflows
    Orchestration,
    /// Agents that monitor systems or other agents
    Monitoring,
    /// Agents that analyze data or provide insights
    Analytics,
    /// Agents that handle communication tasks
    Communication,
    /// Agents that provide utility functions
    Utility,
    /// Custom category defined by user
    Custom(String),
}

impl TemplateCategory {
    #[must_use]
    pub fn name(&self) -> String {
        match self {
            Self::ToolExecution => "tool_execution".to_string(),
            Self::Orchestration => "orchestration".to_string(),
            Self::Monitoring => "monitoring".to_string(),
            Self::Analytics => "analytics".to_string(),
            Self::Communication => "communication".to_string(),
            Self::Utility => "utility".to_string(),
            Self::Custom(name) => name.clone(),
        }
    }
}

/// Template complexity level
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ComplexityLevel {
    /// Simple templates with minimal configuration
    Basic,
    /// Intermediate templates with moderate customization
    Intermediate,
    /// Advanced templates with extensive configuration options
    Advanced,
    /// Expert-level templates requiring deep understanding
    Expert,
}

impl ComplexityLevel {
    #[must_use]
    pub const fn description(&self) -> &'static str {
        match self {
            Self::Basic => "Simple template with minimal configuration required",
            Self::Intermediate => "Moderate complexity with some customization options",
            Self::Advanced => "Advanced template with extensive configuration",
            Self::Expert => "Expert-level template requiring deep understanding",
        }
    }
}

/// Parameter definition for template customization
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ParameterDefinition {
    /// Parameter name
    pub name: String,
    /// Human-readable description
    pub description: String,
    /// Parameter type information
    pub param_type: ParameterType,
    /// Whether this parameter is required
    pub required: bool,
    /// Default value (if any)
    pub default_value: Option<serde_json::Value>,
    /// Validation constraints
    pub constraints: Vec<ParameterConstraint>,
    /// Examples of valid values
    pub examples: Vec<serde_json::Value>,
}

/// Supported parameter types
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ParameterType {
    /// String parameter
    String,
    /// Integer parameter
    Integer,
    /// Floating point parameter
    Float,
    /// Boolean parameter
    Boolean,
    /// Array of values
    Array(Box<ParameterType>),
    /// Object with defined schema
    Object(HashMap<String, ParameterDefinition>),
    /// Enum with defined values
    Enum(Vec<String>),
    /// Tool reference
    ToolReference,
    /// Agent reference
    AgentReference,
    /// Custom type
    Custom(String),
}

/// Parameter validation constraints
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ParameterConstraint {
    /// Minimum value for numeric types
    MinValue(f64),
    /// Maximum value for numeric types
    MaxValue(f64),
    /// Minimum length for strings/arrays
    MinLength(usize),
    /// Maximum length for strings/arrays
    MaxLength(usize),
    /// Regular expression pattern for strings
    Pattern(String),
    /// Custom validation rule
    Custom(String),
}

/// Tool dependency specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolDependency {
    /// Tool name or identifier
    pub name: String,
    /// Tool version requirement (optional)
    pub version: Option<String>,
    /// Whether this tool is required or optional
    pub required: bool,
    /// Alternative tools that can satisfy this dependency
    pub alternatives: Vec<String>,
    /// Configuration for this tool
    pub config: HashMap<String, serde_json::Value>,
}

/// Agent capability requirement
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapabilityRequirement {
    /// Capability name
    pub name: String,
    /// Minimum capability level required
    pub min_level: u8,
    /// Whether this capability is critical
    pub critical: bool,
    /// Description of how this capability is used
    pub usage_description: String,
}

/// Resource requirements for the template
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceRequirements {
    /// Memory requirements in bytes
    pub memory: Option<u64>,
    /// CPU requirements as percentage (0-100)
    pub cpu: Option<u8>,
    /// Disk space requirements in bytes
    pub disk: Option<u64>,
    /// Network bandwidth requirements in bytes/sec
    pub network: Option<u64>,
    /// Maximum execution time in seconds
    pub max_execution_time: Option<u64>,
}

impl Default for ResourceRequirements {
    fn default() -> Self {
        Self {
            memory: Some(512 * 1024 * 1024), // 512MB default
            cpu: Some(25),                   // 25% CPU default
            disk: Some(100 * 1024 * 1024),   // 100MB default
            network: Some(10 * 1024 * 1024), // 10MB/s default
            max_execution_time: Some(300),   // 5 minutes default
        }
    }
}

/// Template metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateMetadata {
    /// Template unique identifier
    pub id: String,
    /// Template display name
    pub name: String,
    /// Template version
    pub version: String,
    /// Template description
    pub description: String,
    /// Template author/organization
    pub author: String,
    /// Template license
    pub license: String,
    /// Repository or source URL
    pub repository: Option<String>,
    /// Documentation URL
    pub documentation: Option<String>,
    /// Template keywords for discovery
    pub keywords: Vec<String>,
    /// Template category
    pub category: TemplateCategory,
    /// Complexity level
    pub complexity: ComplexityLevel,
}

/// Complete template definition schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateSchema {
    /// Schema version for compatibility
    pub schema_version: SchemaVersion,
    /// Template metadata
    pub metadata: TemplateMetadata,
    /// Template parameters
    pub parameters: Vec<ParameterDefinition>,
    /// Tool dependencies
    pub tool_dependencies: Vec<ToolDependency>,
    /// Required capabilities
    pub capability_requirements: Vec<CapabilityRequirement>,
    /// Resource requirements
    pub resource_requirements: ResourceRequirements,
    /// Template-specific configuration
    pub template_config: HashMap<String, serde_json::Value>,
    /// Validation rules
    pub validation_rules: Vec<String>,
}

impl TemplateSchema {
    /// Create a new template schema
    #[must_use]
    pub fn new(metadata: TemplateMetadata) -> Self {
        Self {
            schema_version: SchemaVersion::default(),
            metadata,
            parameters: Vec::new(),
            tool_dependencies: Vec::new(),
            capability_requirements: Vec::new(),
            resource_requirements: ResourceRequirements::default(),
            template_config: HashMap::new(),
            validation_rules: Vec::new(),
        }
    }

    /// Add a parameter to the template
    #[must_use]
    pub fn with_parameter(mut self, parameter: ParameterDefinition) -> Self {
        self.parameters.push(parameter);
        self
    }

    /// Add a tool dependency
    #[must_use]
    pub fn with_tool_dependency(mut self, dependency: ToolDependency) -> Self {
        self.tool_dependencies.push(dependency);
        self
    }

    /// Add a capability requirement
    #[must_use]
    pub fn with_capability_requirement(mut self, requirement: CapabilityRequirement) -> Self {
        self.capability_requirements.push(requirement);
        self
    }

    /// Set resource requirements
    #[must_use]
    pub const fn with_resource_requirements(mut self, requirements: ResourceRequirements) -> Self {
        self.resource_requirements = requirements;
        self
    }

    /// Add template configuration
    #[must_use]
    pub fn with_config(mut self, key: &str, value: serde_json::Value) -> Self {
        self.template_config.insert(key.to_string(), value);
        self
    }

    /// Add validation rule
    #[must_use]
    pub fn with_validation_rule(mut self, rule: String) -> Self {
        self.validation_rules.push(rule);
        self
    }

    /// Validate the template schema
    pub fn validate(&self) -> Result<()> {
        // Validate metadata
        if self.metadata.id.is_empty() {
            return Err(anyhow!("Template ID cannot be empty"));
        }
        if self.metadata.name.is_empty() {
            return Err(anyhow!("Template name cannot be empty"));
        }
        if self.metadata.version.is_empty() {
            return Err(anyhow!("Template version cannot be empty"));
        }

        // Validate parameters
        let mut param_names = std::collections::HashSet::new();
        for param in &self.parameters {
            if param.name.is_empty() {
                return Err(anyhow!("Parameter name cannot be empty"));
            }
            if !param_names.insert(&param.name) {
                return Err(anyhow!("Duplicate parameter name: {}", param.name));
            }

            // Validate required parameters have no default value conflicts
            if param.required && param.default_value.is_some() {
                return Err(anyhow!(
                    "Required parameter '{}' should not have default value",
                    param.name
                ));
            }
        }

        // Validate tool dependencies
        let mut tool_names = std::collections::HashSet::new();
        for tool in &self.tool_dependencies {
            if tool.name.is_empty() {
                return Err(anyhow!("Tool dependency name cannot be empty"));
            }
            if !tool_names.insert(&tool.name) {
                return Err(anyhow!("Duplicate tool dependency: {}", tool.name));
            }
        }

        // Validate resource requirements
        if let Some(cpu) = self.resource_requirements.cpu {
            if cpu > 100 {
                return Err(anyhow!("CPU requirement cannot exceed 100%"));
            }
        }

        Ok(())
    }

    /// Get parameter by name
    #[must_use]
    pub fn get_parameter(&self, name: &str) -> Option<&ParameterDefinition> {
        self.parameters.iter().find(|p| p.name == name)
    }

    /// Get tool dependency by name
    #[must_use]
    pub fn get_tool_dependency(&self, name: &str) -> Option<&ToolDependency> {
        self.tool_dependencies.iter().find(|t| t.name == name)
    }

    /// Check if template has required tool
    #[must_use]
    pub fn requires_tool(&self, tool_name: &str) -> bool {
        self.tool_dependencies
            .iter()
            .any(|t| t.name == tool_name && t.required)
    }

    /// Get all required parameters
    #[must_use]
    pub fn required_parameters(&self) -> Vec<&ParameterDefinition> {
        self.parameters.iter().filter(|p| p.required).collect()
    }

    /// Get all optional parameters with defaults
    #[must_use]
    pub fn optional_parameters_with_defaults(&self) -> Vec<&ParameterDefinition> {
        self.parameters
            .iter()
            .filter(|p| !p.required && p.default_value.is_some())
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_template_schema_creation() {
        let metadata = TemplateMetadata {
            id: "test_template".to_string(),
            name: "Test Template".to_string(),
            version: "1.0.0".to_string(),
            description: "A test template".to_string(),
            author: "Test Author".to_string(),
            license: "MIT".to_string(),
            repository: None,
            documentation: None,
            keywords: vec!["test".to_string()],
            category: TemplateCategory::Utility,
            complexity: ComplexityLevel::Basic,
        };

        let schema = TemplateSchema::new(metadata.clone());
        assert_eq!(schema.metadata.id, "test_template");
        assert_eq!(schema.schema_version, SchemaVersion::V1);
        assert!(schema.parameters.is_empty());
    }
    #[test]
    fn test_parameter_definition() {
        let param = ParameterDefinition {
            name: "test_param".to_string(),
            description: "A test parameter".to_string(),
            param_type: ParameterType::String,
            required: true,
            default_value: None,
            constraints: vec![ParameterConstraint::MinLength(1)],
            examples: vec!["example".into()],
        };

        assert_eq!(param.name, "test_param");
        assert!(param.required);
        assert_eq!(param.constraints.len(), 1);
    }
    #[test]
    fn test_schema_validation() {
        let metadata = TemplateMetadata {
            id: "valid_template".to_string(),
            name: "Valid Template".to_string(),
            version: "1.0.0".to_string(),
            description: "A valid template".to_string(),
            author: "Test Author".to_string(),
            license: "MIT".to_string(),
            repository: None,
            documentation: None,
            keywords: vec!["test".to_string()],
            category: TemplateCategory::Utility,
            complexity: ComplexityLevel::Basic,
        };

        let schema = TemplateSchema::new(metadata);
        assert!(schema.validate().is_ok());
    }
    #[test]
    fn test_schema_validation_empty_id() {
        let metadata = TemplateMetadata {
            id: "".to_string(),
            name: "Test Template".to_string(),
            version: "1.0.0".to_string(),
            description: "A test template".to_string(),
            author: "Test Author".to_string(),
            license: "MIT".to_string(),
            repository: None,
            documentation: None,
            keywords: vec!["test".to_string()],
            category: TemplateCategory::Utility,
            complexity: ComplexityLevel::Basic,
        };

        let schema = TemplateSchema::new(metadata);
        assert!(schema.validate().is_err());
    }
    #[test]
    fn test_template_category_names() {
        assert_eq!(TemplateCategory::ToolExecution.name(), "tool_execution");
        assert_eq!(TemplateCategory::Orchestration.name(), "orchestration");
        assert_eq!(TemplateCategory::Custom("test".to_string()).name(), "test");
    }
    #[test]
    fn test_complexity_descriptions() {
        assert!(!ComplexityLevel::Basic.description().is_empty());
        assert!(!ComplexityLevel::Expert.description().is_empty());
    }
    #[test]
    fn test_schema_builder_pattern() {
        let metadata = TemplateMetadata {
            id: "builder_test".to_string(),
            name: "Builder Test".to_string(),
            version: "1.0.0".to_string(),
            description: "Test builder pattern".to_string(),
            author: "Test Author".to_string(),
            license: "MIT".to_string(),
            repository: None,
            documentation: None,
            keywords: vec!["test".to_string()],
            category: TemplateCategory::Utility,
            complexity: ComplexityLevel::Basic,
        };

        let param = ParameterDefinition {
            name: "test_param".to_string(),
            description: "A test parameter".to_string(),
            param_type: ParameterType::String,
            required: true,
            default_value: None,
            constraints: vec![],
            examples: vec![],
        };

        let tool_dep = ToolDependency {
            name: "test_tool".to_string(),
            version: Some("1.0.0".to_string()),
            required: true,
            alternatives: vec![],
            config: HashMap::new(),
        };

        let schema = TemplateSchema::new(metadata)
            .with_parameter(param)
            .with_tool_dependency(tool_dep)
            .with_config("test_key", "test_value".into());

        assert_eq!(schema.parameters.len(), 1);
        assert_eq!(schema.tool_dependencies.len(), 1);
        assert!(schema.template_config.contains_key("test_key"));
    }
}
