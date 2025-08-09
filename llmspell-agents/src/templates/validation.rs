//! ABOUTME: Template validation utilities for comprehensive template and instantiation validation
//! ABOUTME: Provides validators, analyzers, and compatibility checkers for agent templates

use super::base::{AgentTemplate, TemplateInstantiationParams};
use super::schema::{
    ParameterConstraint, ParameterDefinition, ParameterType, ResourceRequirements, TemplateSchema,
};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

/// Validation result with detailed information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    /// Whether validation passed
    pub is_valid: bool,
    /// Validation errors
    pub errors: Vec<ValidationError>,
    /// Validation warnings
    pub warnings: Vec<ValidationWarning>,
    /// Validation metadata
    pub metadata: HashMap<String, String>,
}

impl ValidationResult {
    /// Create a successful validation result
    #[must_use]
    pub fn success() -> Self {
        Self {
            is_valid: true,
            errors: Vec::new(),
            warnings: Vec::new(),
            metadata: HashMap::new(),
        }
    }

    /// Add an error
    pub fn add_error(&mut self, error: ValidationError) {
        self.errors.push(error);
        self.is_valid = false;
    }

    /// Add a warning
    pub fn add_warning(&mut self, warning: ValidationWarning) {
        self.warnings.push(warning);
    }

    /// Add metadata
    pub fn add_metadata(&mut self, key: &str, value: &str) {
        self.metadata.insert(key.to_string(), value.to_string());
    }

    /// Check if there are any issues (errors or warnings)
    #[must_use]
    pub fn has_issues(&self) -> bool {
        !self.errors.is_empty() || !self.warnings.is_empty()
    }
}

/// Validation error types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ValidationError {
    /// Missing required field
    MissingRequired { field: String, context: String },
    /// Invalid value
    InvalidValue {
        field: String,
        value: String,
        expected: String,
    },
    /// Constraint violation
    ConstraintViolation {
        field: String,
        constraint: String,
        value: String,
    },
    /// Dependency not found
    DependencyNotFound { dependency: String, context: String },
    /// Circular dependency
    CircularDependency { items: Vec<String> },
    /// Resource limit exceeded
    ResourceLimitExceeded {
        resource: String,
        requested: u64,
        limit: u64,
    },
    /// Incompatible configuration
    IncompatibleConfig {
        field1: String,
        field2: String,
        reason: String,
    },
    /// Custom error
    Custom { message: String },
}

impl std::fmt::Display for ValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MissingRequired { field, context } => {
                write!(f, "Missing required field '{field}' in {context}")
            }
            Self::InvalidValue {
                field,
                value,
                expected,
            } => {
                write!(
                    f,
                    "Invalid value '{value}' for field '{field}', expected: {expected}"
                )
            }
            Self::ConstraintViolation {
                field,
                constraint,
                value,
            } => {
                write!(
                    f,
                    "Constraint violation for field '{field}': {constraint} (value: {value})"
                )
            }
            Self::DependencyNotFound {
                dependency,
                context,
            } => {
                write!(f, "Dependency '{dependency}' not found in {context}")
            }
            Self::CircularDependency { items } => {
                write!(f, "Circular dependency detected: {}", items.join(" -> "))
            }
            Self::ResourceLimitExceeded {
                resource,
                requested,
                limit,
            } => {
                write!(
                    f,
                    "Resource limit exceeded for {resource}: requested {requested}, limit {limit}"
                )
            }
            Self::IncompatibleConfig {
                field1,
                field2,
                reason,
            } => {
                write!(
                    f,
                    "Incompatible configuration between '{field1}' and '{field2}': {reason}"
                )
            }
            Self::Custom { message } => write!(f, "{message}"),
        }
    }
}

/// Validation warning types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ValidationWarning {
    /// Deprecated feature
    Deprecated {
        feature: String,
        alternative: Option<String>,
    },
    /// Suboptimal configuration
    SuboptimalConfig { field: String, suggestion: String },
    /// Missing optional feature
    MissingOptional { feature: String, impact: String },
    /// Performance concern
    PerformanceConcern { area: String, details: String },
    /// Security consideration
    SecurityConsideration {
        area: String,
        recommendation: String,
    },
    /// Custom warning
    Custom { message: String },
}

impl std::fmt::Display for ValidationWarning {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Deprecated {
                feature,
                alternative,
            } => {
                write!(f, "Deprecated feature '{feature}' used")?;
                if let Some(alt) = alternative {
                    write!(f, ", consider using '{alt}'")?;
                }
                Ok(())
            }
            Self::SuboptimalConfig { field, suggestion } => {
                write!(f, "Suboptimal configuration for '{field}': {suggestion}")
            }
            Self::MissingOptional { feature, impact } => {
                write!(f, "Missing optional feature '{feature}': {impact}")
            }
            Self::PerformanceConcern { area, details } => {
                write!(f, "Performance concern in {area}: {details}")
            }
            Self::SecurityConsideration {
                area,
                recommendation,
            } => {
                write!(f, "Security consideration for {area}: {recommendation}")
            }
            Self::Custom { message } => write!(f, "{message}"),
        }
    }
}

/// Template validator for comprehensive validation
pub struct TemplateValidator {
    /// Available tool registry
    available_tools: HashSet<String>,
    /// System resource limits
    system_limits: ResourceRequirements,
    /// Validation rules
    rules: Vec<Box<dyn ValidationRule>>,
}

impl Default for TemplateValidator {
    fn default() -> Self {
        Self::new()
    }
}

impl TemplateValidator {
    /// Create new validator
    #[must_use]
    pub fn new() -> Self {
        Self {
            available_tools: HashSet::new(),
            system_limits: ResourceRequirements {
                memory: Some(4 * 1024 * 1024 * 1024), // 4GB
                cpu: Some(100),                       // 100%
                disk: Some(100 * 1024 * 1024 * 1024), // 100GB
                network: Some(100 * 1024 * 1024),     // 100MB/s
                max_execution_time: Some(86400),      // 24 hours
            },
            rules: vec![
                Box::new(SchemaValidationRule),
                Box::new(ParameterValidationRule),
                Box::new(DependencyValidationRule),
                Box::new(ResourceValidationRule),
                Box::new(SecurityValidationRule),
            ],
        }
    }

    /// Register available tool
    pub fn register_tool(&mut self, tool_name: &str) {
        self.available_tools.insert(tool_name.to_string());
    }

    /// Register multiple tools
    pub fn register_tools(&mut self, tools: &[&str]) {
        for tool in tools {
            self.register_tool(tool);
        }
    }

    /// Set system resource limits
    pub const fn set_system_limits(&mut self, limits: ResourceRequirements) {
        self.system_limits = limits;
    }

    /// Add custom validation rule
    pub fn add_rule(&mut self, rule: Box<dyn ValidationRule>) {
        self.rules.push(rule);
    }

    /// Validate template schema
    #[must_use]
    pub fn validate_schema(&self, schema: &TemplateSchema) -> ValidationResult {
        let mut result = ValidationResult::success();

        // Run all validation rules
        for rule in &self.rules {
            rule.validate_schema(schema, &mut result, self);
        }

        // Add metadata
        result.add_metadata("schema_version", &schema.schema_version.to_string());
        result.add_metadata("template_id", &schema.metadata.id);
        result.add_metadata("validation_timestamp", &chrono::Utc::now().to_rfc3339());

        result
    }

    /// Validate template instantiation parameters
    #[must_use]
    pub fn validate_instantiation(
        &self,
        schema: &TemplateSchema,
        params: &TemplateInstantiationParams,
    ) -> ValidationResult {
        let mut result = ValidationResult::success();

        // Check required parameters
        for param in schema.required_parameters() {
            if !params.parameters.contains_key(&param.name) {
                result.add_error(ValidationError::MissingRequired {
                    field: param.name.clone(),
                    context: "instantiation parameters".to_string(),
                });
            }
        }

        // Validate parameter values
        for (name, value) in &params.parameters {
            if let Some(param_def) = schema.get_parameter(name) {
                Self::validate_parameter_value(param_def, value, &mut result);
            } else {
                result.add_warning(ValidationWarning::Custom {
                    message: format!("Unknown parameter '{name}' provided"),
                });
            }
        }

        // Check resource availability
        if let Some(_resource_manager) = &params.resource_manager {
            result.add_metadata("resource_manager", "provided");
        } else {
            result.add_warning(ValidationWarning::MissingOptional {
                feature: "resource_manager".to_string(),
                impact: "No resource tracking or limits will be enforced".to_string(),
            });
        }

        result
    }

    /// Validate parameter value against definition
    fn validate_parameter_value(
        param_def: &ParameterDefinition,
        value: &serde_json::Value,
        result: &mut ValidationResult,
    ) {
        // Type validation
        match &param_def.param_type {
            ParameterType::String => {
                if !value.is_string() {
                    result.add_error(ValidationError::InvalidValue {
                        field: param_def.name.clone(),
                        value: value.to_string(),
                        expected: "string".to_string(),
                    });
                }
            }
            ParameterType::Integer => {
                if !value.is_i64() && !value.is_u64() {
                    result.add_error(ValidationError::InvalidValue {
                        field: param_def.name.clone(),
                        value: value.to_string(),
                        expected: "integer".to_string(),
                    });
                }
            }
            ParameterType::Float => {
                if !value.is_f64() && !value.is_i64() && !value.is_u64() {
                    result.add_error(ValidationError::InvalidValue {
                        field: param_def.name.clone(),
                        value: value.to_string(),
                        expected: "number".to_string(),
                    });
                }
            }
            ParameterType::Boolean => {
                if !value.is_boolean() {
                    result.add_error(ValidationError::InvalidValue {
                        field: param_def.name.clone(),
                        value: value.to_string(),
                        expected: "boolean".to_string(),
                    });
                }
            }
            ParameterType::Array(_elem_type) => {
                if let Some(array) = value.as_array() {
                    // Could validate array elements here
                    if array.is_empty() && param_def.required {
                        result.add_warning(ValidationWarning::SuboptimalConfig {
                            field: param_def.name.clone(),
                            suggestion: "Empty array provided for required parameter".to_string(),
                        });
                    }
                } else {
                    result.add_error(ValidationError::InvalidValue {
                        field: param_def.name.clone(),
                        value: value.to_string(),
                        expected: "array".to_string(),
                    });
                }
            }
            ParameterType::Object(_) => {
                if !value.is_object() {
                    result.add_error(ValidationError::InvalidValue {
                        field: param_def.name.clone(),
                        value: value.to_string(),
                        expected: "object".to_string(),
                    });
                }
            }
            ParameterType::Enum(allowed) => {
                if let Some(str_val) = value.as_str() {
                    if !allowed.contains(&str_val.to_string()) {
                        result.add_error(ValidationError::InvalidValue {
                            field: param_def.name.clone(),
                            value: str_val.to_string(),
                            expected: format!("one of: {allowed:?}"),
                        });
                    }
                } else {
                    result.add_error(ValidationError::InvalidValue {
                        field: param_def.name.clone(),
                        value: value.to_string(),
                        expected: "string enum value".to_string(),
                    });
                }
            }
            _ => {}
        }

        // Constraint validation
        for constraint in &param_def.constraints {
            Self::validate_constraint(&param_def.name, constraint, value, result);
        }
    }

    /// Validate a single constraint
    fn validate_constraint(
        param_name: &str,
        constraint: &ParameterConstraint,
        value: &serde_json::Value,
        result: &mut ValidationResult,
    ) {
        match constraint {
            ParameterConstraint::MinValue(min) => {
                if let Some(num) = value.as_f64() {
                    if num < *min {
                        result.add_error(ValidationError::ConstraintViolation {
                            field: param_name.to_string(),
                            constraint: format!("minimum value {min}"),
                            value: num.to_string(),
                        });
                    }
                }
            }
            ParameterConstraint::MaxValue(max) => {
                if let Some(num) = value.as_f64() {
                    if num > *max {
                        result.add_error(ValidationError::ConstraintViolation {
                            field: param_name.to_string(),
                            constraint: format!("maximum value {max}"),
                            value: num.to_string(),
                        });
                    }
                }
            }
            ParameterConstraint::MinLength(min_len) => {
                let length = value
                    .as_str()
                    .map(str::len)
                    .or_else(|| value.as_array().map(Vec::len))
                    .unwrap_or(0);
                if length < *min_len {
                    result.add_error(ValidationError::ConstraintViolation {
                        field: param_name.to_string(),
                        constraint: format!("minimum length {min_len}"),
                        value: length.to_string(),
                    });
                }
            }
            ParameterConstraint::MaxLength(max_len) => {
                let length = value
                    .as_str()
                    .map(str::len)
                    .or_else(|| value.as_array().map(Vec::len))
                    .unwrap_or(0);
                if length > *max_len {
                    result.add_error(ValidationError::ConstraintViolation {
                        field: param_name.to_string(),
                        constraint: format!("maximum length {max_len}"),
                        value: length.to_string(),
                    });
                }
            }
            ParameterConstraint::Pattern(pattern) => {
                if let Some(s) = value.as_str() {
                    if let Ok(regex) = regex::Regex::new(pattern) {
                        if !regex.is_match(s) {
                            result.add_error(ValidationError::ConstraintViolation {
                                field: param_name.to_string(),
                                constraint: format!("pattern {pattern}"),
                                value: s.to_string(),
                            });
                        }
                    } else {
                        result.add_error(ValidationError::Custom {
                            message: format!("Invalid regex pattern for parameter '{param_name}'"),
                        });
                    }
                }
            }
            ParameterConstraint::Custom(rule) => {
                // Custom constraints would need specific handling
                result.add_metadata(&format!("custom_constraint_{param_name}"), rule);
            }
        }
    }
}

/// Validation rule trait
pub trait ValidationRule: Send + Sync {
    /// Validate schema
    fn validate_schema(
        &self,
        schema: &TemplateSchema,
        result: &mut ValidationResult,
        validator: &TemplateValidator,
    );
}

/// Schema structure validation
struct SchemaValidationRule;

impl ValidationRule for SchemaValidationRule {
    fn validate_schema(
        &self,
        schema: &TemplateSchema,
        result: &mut ValidationResult,
        _validator: &TemplateValidator,
    ) {
        // Check metadata
        if schema.metadata.id.is_empty() {
            result.add_error(ValidationError::MissingRequired {
                field: "metadata.id".to_string(),
                context: "template schema".to_string(),
            });
        }
        if schema.metadata.name.is_empty() {
            result.add_error(ValidationError::MissingRequired {
                field: "metadata.name".to_string(),
                context: "template schema".to_string(),
            });
        }
        if schema.metadata.version.is_empty() {
            result.add_error(ValidationError::MissingRequired {
                field: "metadata.version".to_string(),
                context: "template schema".to_string(),
            });
        }

        // Check for duplicate parameter names
        let mut param_names = HashSet::new();
        for param in &schema.parameters {
            if !param_names.insert(&param.name) {
                result.add_error(ValidationError::Custom {
                    message: format!("Duplicate parameter name: {}", param.name),
                });
            }
        }

        // Check for duplicate tool dependencies
        let mut tool_names = HashSet::new();
        for tool in &schema.tool_dependencies {
            if !tool_names.insert(&tool.name) {
                result.add_error(ValidationError::Custom {
                    message: format!("Duplicate tool dependency: {}", tool.name),
                });
            }
        }
    }
}

/// Parameter validation
struct ParameterValidationRule;

impl ValidationRule for ParameterValidationRule {
    fn validate_schema(
        &self,
        schema: &TemplateSchema,
        result: &mut ValidationResult,
        _validator: &TemplateValidator,
    ) {
        for param in &schema.parameters {
            // Check that required parameters don't have defaults
            if param.required && param.default_value.is_some() {
                result.add_warning(ValidationWarning::SuboptimalConfig {
                    field: param.name.clone(),
                    suggestion: "Required parameters should not have default values".to_string(),
                });
            }

            // Validate examples match the parameter type
            for _example in &param.examples {
                // Basic type checking could be done here
            }

            // Check constraint consistency
            let mut min_val = None;
            let mut max_val = None;
            for constraint in &param.constraints {
                match constraint {
                    ParameterConstraint::MinValue(min) => min_val = Some(*min),
                    ParameterConstraint::MaxValue(max) => max_val = Some(*max),
                    _ => {}
                }
            }
            if let (Some(min), Some(max)) = (min_val, max_val) {
                if min > max {
                    result.add_error(ValidationError::IncompatibleConfig {
                        field1: format!("{}.min_value", param.name),
                        field2: format!("{}.max_value", param.name),
                        reason: "Minimum value exceeds maximum value".to_string(),
                    });
                }
            }
        }
    }
}

/// Dependency validation
struct DependencyValidationRule;

impl ValidationRule for DependencyValidationRule {
    fn validate_schema(
        &self,
        schema: &TemplateSchema,
        result: &mut ValidationResult,
        validator: &TemplateValidator,
    ) {
        for tool in &schema.tool_dependencies {
            // Check if required tools are available
            if tool.required && !validator.available_tools.contains(&tool.name) {
                let mut found_alternative = false;
                for alt in &tool.alternatives {
                    if validator.available_tools.contains(alt) {
                        found_alternative = true;
                        break;
                    }
                }
                if !found_alternative {
                    result.add_warning(ValidationWarning::MissingOptional {
                        feature: format!("tool '{}'", tool.name),
                        impact: "Template may not function without this tool".to_string(),
                    });
                }
            }
        }
    }
}

/// Resource validation
struct ResourceValidationRule;

impl ValidationRule for ResourceValidationRule {
    fn validate_schema(
        &self,
        schema: &TemplateSchema,
        result: &mut ValidationResult,
        validator: &TemplateValidator,
    ) {
        let reqs = &schema.resource_requirements;

        // Check against system limits
        if let Some(memory) = reqs.memory {
            if let Some(limit) = validator.system_limits.memory {
                if memory > limit {
                    result.add_error(ValidationError::ResourceLimitExceeded {
                        resource: "memory".to_string(),
                        requested: memory,
                        limit,
                    });
                }
            }
        }

        if let Some(cpu) = reqs.cpu {
            if cpu > 100 {
                result.add_error(ValidationError::InvalidValue {
                    field: "resource_requirements.cpu".to_string(),
                    value: cpu.to_string(),
                    expected: "0-100".to_string(),
                });
            }
        }

        // Performance warnings
        if let Some(memory) = reqs.memory {
            if memory > 1024 * 1024 * 1024 {
                // 1GB
                result.add_warning(ValidationWarning::PerformanceConcern {
                    area: "memory".to_string(),
                    details: "High memory requirement may limit deployment options".to_string(),
                });
            }
        }
    }
}

/// Security validation
struct SecurityValidationRule;

impl ValidationRule for SecurityValidationRule {
    fn validate_schema(
        &self,
        schema: &TemplateSchema,
        result: &mut ValidationResult,
        _validator: &TemplateValidator,
    ) {
        // Check for security-sensitive tools
        let sensitive_tools = [
            "file_writer",
            "network_access",
            "system_command",
            "database_connector",
        ];
        for tool in &schema.tool_dependencies {
            if sensitive_tools.contains(&tool.name.as_str()) {
                result.add_warning(ValidationWarning::SecurityConsideration {
                    area: format!("tool '{}'", tool.name),
                    recommendation: "Ensure proper access controls and sandboxing".to_string(),
                });
            }
        }

        // Check for overly permissive resource requirements
        if let Some(network) = schema.resource_requirements.network {
            if network > 50 * 1024 * 1024 {
                // 50MB/s
                result.add_warning(ValidationWarning::SecurityConsideration {
                    area: "network bandwidth".to_string(),
                    recommendation: "High network usage may indicate data exfiltration risk"
                        .to_string(),
                });
            }
        }
    }
}

/// Analyze template for best practices
pub fn analyze_template(template: &dyn AgentTemplate) -> HashMap<String, String> {
    let mut analysis = HashMap::new();
    let schema = template.schema();

    // Complexity analysis
    let complexity_score = calculate_complexity_score(schema);
    analysis.insert("complexity_score".to_string(), complexity_score.to_string());
    analysis.insert(
        "complexity_rating".to_string(),
        match complexity_score {
            0..=20 => "Simple",
            21..=50 => "Moderate",
            51..=80 => "Complex",
            _ => "Very Complex",
        }
        .to_string(),
    );

    // Tool dependency analysis
    let required_tools = template.required_tools().len();
    let optional_tools = template.optional_tools().len();
    analysis.insert("required_tools".to_string(), required_tools.to_string());
    analysis.insert("optional_tools".to_string(), optional_tools.to_string());
    analysis.insert(
        "total_dependencies".to_string(),
        (required_tools + optional_tools).to_string(),
    );

    // Resource usage analysis
    if let Some(memory) = schema.resource_requirements.memory {
        analysis.insert("memory_mb".to_string(), (memory / 1024 / 1024).to_string());
    }
    if let Some(cpu) = schema.resource_requirements.cpu {
        analysis.insert("cpu_percent".to_string(), cpu.to_string());
    }

    // Parameter analysis
    let required_params = schema.required_parameters().len();
    let optional_params = schema.parameters.len() - required_params;
    analysis.insert(
        "required_parameters".to_string(),
        required_params.to_string(),
    );
    analysis.insert(
        "optional_parameters".to_string(),
        optional_params.to_string(),
    );

    analysis
}

/// Calculate complexity score for a template
fn calculate_complexity_score(schema: &TemplateSchema) -> u32 {
    let mut score = 0;

    // Base complexity from metadata
    score += match &schema.metadata.complexity {
        super::schema::ComplexityLevel::Basic => 10,
        super::schema::ComplexityLevel::Intermediate => 30,
        super::schema::ComplexityLevel::Advanced => 50,
        super::schema::ComplexityLevel::Expert => 70,
    };

    // Add points for parameters
    #[allow(clippy::cast_possible_truncation)]
    let param_count = schema.parameters.len() as u32;
    score += param_count * 2;
    #[allow(clippy::cast_possible_truncation)]
    let required_param_count = schema.required_parameters().len() as u32;
    score += required_param_count * 3;

    // Add points for dependencies
    #[allow(clippy::cast_possible_truncation)]
    let dep_count = schema.tool_dependencies.len() as u32;
    score += dep_count * 3;
    #[allow(clippy::cast_possible_truncation)]
    let required_dep_count = schema
        .tool_dependencies
        .iter()
        .filter(|t| t.required)
        .count() as u32;
    score += required_dep_count * 2;

    // Add points for capabilities
    #[allow(clippy::cast_possible_truncation)]
    let capability_count = schema.capability_requirements.len() as u32;
    score += capability_count * 4;

    // Add points for resource requirements
    if let Some(memory) = schema.resource_requirements.memory {
        #[allow(clippy::cast_possible_truncation)]
        let memory_points = (memory / (256 * 1024 * 1024)) as u32; // Points per 256MB
        score += memory_points;
    }

    score
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::templates::schema::{ComplexityLevel, TemplateCategory, TemplateMetadata};

    fn create_test_schema() -> TemplateSchema {
        let metadata = TemplateMetadata {
            id: "test_template".to_string(),
            name: "Test Template".to_string(),
            version: "1.0.0".to_string(),
            description: "Test template".to_string(),
            author: "Test".to_string(),
            license: "MIT".to_string(),
            repository: None,
            documentation: None,
            keywords: vec![],
            category: TemplateCategory::Utility,
            complexity: ComplexityLevel::Basic,
        };
        TemplateSchema::new(metadata)
    }
    #[test]
    fn test_validation_result() {
        let mut result = ValidationResult::success();
        assert!(result.is_valid);
        assert!(!result.has_issues());

        result.add_warning(ValidationWarning::Custom {
            message: "Test warning".to_string(),
        });
        assert!(result.is_valid); // Warnings don't invalidate
        assert!(result.has_issues());

        result.add_error(ValidationError::Custom {
            message: "Test error".to_string(),
        });
        assert!(!result.is_valid); // Errors invalidate
    }
    #[test]
    fn test_schema_validation() {
        let validator = TemplateValidator::new();

        // Test valid schema
        let schema = create_test_schema();
        let result = validator.validate_schema(&schema);
        assert!(result.is_valid);

        // Test invalid schema (empty ID)
        let mut invalid_schema = create_test_schema();
        invalid_schema.metadata.id = String::new();
        let result = validator.validate_schema(&invalid_schema);
        assert!(!result.is_valid);
        assert!(result.errors.iter().any(|e| matches!(e, ValidationError::MissingRequired { field, .. } if field == "metadata.id")));
    }
    #[test]
    fn test_parameter_validation() {
        let validator = TemplateValidator::new();
        let mut schema = create_test_schema();

        // Add parameter with conflicting constraints
        schema = schema.with_parameter(ParameterDefinition {
            name: "test_param".to_string(),
            description: "Test".to_string(),
            param_type: ParameterType::Integer,
            required: true,
            default_value: None,
            constraints: vec![
                ParameterConstraint::MinValue(100.0),
                ParameterConstraint::MaxValue(50.0), // Max < Min
            ],
            examples: vec![],
        });

        let result = validator.validate_schema(&schema);
        assert!(result
            .errors
            .iter()
            .any(|e| matches!(e, ValidationError::IncompatibleConfig { .. })));
    }
    #[test]
    fn test_resource_validation() {
        let mut validator = TemplateValidator::new();
        validator.set_system_limits(ResourceRequirements {
            memory: Some(1024 * 1024 * 1024), // 1GB limit
            cpu: Some(100),
            disk: None,
            network: None,
            max_execution_time: None,
        });

        let mut schema = create_test_schema();
        schema = schema.with_resource_requirements(ResourceRequirements {
            memory: Some(2 * 1024 * 1024 * 1024), // 2GB - exceeds limit
            cpu: Some(50),
            disk: None,
            network: None,
            max_execution_time: None,
        });

        let result = validator.validate_schema(&schema);
        assert!(result.errors.iter().any(|e| matches!(e, ValidationError::ResourceLimitExceeded { resource, .. } if resource == "memory")));
    }
    #[test]
    fn test_instantiation_validation() {
        let validator = TemplateValidator::new();
        let mut schema = create_test_schema();

        // Add required parameter
        schema = schema.with_parameter(ParameterDefinition {
            name: "required_param".to_string(),
            description: "Required".to_string(),
            param_type: ParameterType::String,
            required: true,
            default_value: None,
            constraints: vec![],
            examples: vec![],
        });

        // Test missing required parameter
        let params = TemplateInstantiationParams::new("test-agent".to_string());
        let result = validator.validate_instantiation(&schema, &params);
        assert!(!result.is_valid);
        assert!(result.errors.iter().any(|e| matches!(e, ValidationError::MissingRequired { field, .. } if field == "required_param")));

        // Test valid instantiation
        let params = TemplateInstantiationParams::new("test-agent".to_string())
            .with_parameter("required_param", "value".into());
        let result = validator.validate_instantiation(&schema, &params);
        assert!(result.is_valid);
    }
    #[test]
    fn test_constraint_validation() {
        let _validator = TemplateValidator::new();
        let mut result = ValidationResult::success();

        // Test MinValue constraint
        TemplateValidator::validate_constraint(
            "test",
            &ParameterConstraint::MinValue(10.0),
            &5.into(),
            &mut result,
        );
        assert!(!result.is_valid);

        // Test Pattern constraint
        let mut result = ValidationResult::success();
        TemplateValidator::validate_constraint(
            "email",
            &ParameterConstraint::Pattern(r"^[^@]+@[^@]+\.[^@]+$".to_string()),
            &"invalid-email".into(),
            &mut result,
        );
        assert!(!result.is_valid);
    }
    #[test]
    fn test_complexity_analysis() {
        let mut schema = create_test_schema();

        // Add complexity
        for i in 0..5 {
            schema = schema.with_parameter(ParameterDefinition {
                name: format!("param_{i}"),
                description: "Test".to_string(),
                param_type: ParameterType::String,
                required: i < 3,
                default_value: None,
                constraints: vec![],
                examples: vec![],
            });
        }

        let score = calculate_complexity_score(&schema);
        assert!(score > 10); // Basic complexity + parameters
    }
}
