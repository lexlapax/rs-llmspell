// ABOUTME: Environment variable access tool with security controls and filtering
// ABOUTME: Provides safe access to system environment variables with configurable permissions

use async_trait::async_trait;
use llmspell_core::{
    traits::{
        base_agent::BaseAgent,
        tool::{ParameterDef, ParameterType, SecurityLevel, Tool, ToolCategory, ToolSchema},
    },
    types::{AgentInput, AgentOutput},
    ComponentMetadata, ExecutionContext, LLMSpellError, Result as LLMResult,
};
use llmspell_security::sandbox::SandboxContext;
use llmspell_utils::{
    extract_parameters, extract_required_string,
    response::ResponseBuilder,
    system_info::{get_all_env_vars, get_env_var, set_env_var_if_allowed},
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::HashMap;
use std::sync::Arc;
use tracing::{debug, info, warn};

/// Environment reader tool configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvironmentReaderConfig {
    /// Maximum number of environment variables to return in list operations
    pub max_variables_returned: usize,
    /// Allowed environment variable patterns (glob patterns)
    pub allowed_patterns: Vec<String>,
    /// Explicitly blocked environment variable patterns
    pub blocked_patterns: Vec<String>,
    /// Whether to allow reading all environment variables
    pub allow_read_all: bool,
    /// Whether to allow setting environment variables (requires additional permissions)
    pub allow_set_variables: bool,
    /// Default environment variables that are always safe to read
    pub default_safe_vars: Vec<String>,
}

impl Default for EnvironmentReaderConfig {
    fn default() -> Self {
        Self {
            max_variables_returned: 100,
            allowed_patterns: vec![
                "PATH".to_string(),
                "HOME".to_string(),
                "USER".to_string(),
                "SHELL".to_string(),
                "TERM".to_string(),
                "LANG".to_string(),
                "LC_*".to_string(),
                "TZ".to_string(),
                "TMPDIR".to_string(),
                "TEMP".to_string(),
                "PWD".to_string(),
                "OLDPWD".to_string(),
            ],
            blocked_patterns: vec![
                "*PASSWORD*".to_string(),
                "*SECRET*".to_string(),
                "*KEY*".to_string(),
                "*TOKEN*".to_string(),
                "*CREDENTIAL*".to_string(),
                "*AUTH*".to_string(),
                "AWS_*".to_string(),
                "AZURE_*".to_string(),
                "GCP_*".to_string(),
                "*_PRIVATE_*".to_string(),
                "SSH_*".to_string(),
                "GITHUB_TOKEN".to_string(),
                "API_*".to_string(),
            ],
            allow_read_all: false,
            allow_set_variables: false,
            default_safe_vars: vec![
                "PATH".to_string(),
                "HOME".to_string(),
                "USER".to_string(),
                "SHELL".to_string(),
                "PWD".to_string(),
                "LANG".to_string(),
                "TERM".to_string(),
            ],
        }
    }
}

/// Environment reader tool for accessing system environment variables
#[derive(Clone)]
pub struct EnvironmentReaderTool {
    metadata: ComponentMetadata,
    config: EnvironmentReaderConfig,
    sandbox_context: Option<Arc<SandboxContext>>,
}

impl EnvironmentReaderTool {
    /// Create a new environment reader tool
    pub fn new(config: EnvironmentReaderConfig) -> Self {
        Self {
            metadata: ComponentMetadata::new(
                "environment_reader".to_string(),
                "System environment variable access with security controls".to_string(),
            ),
            config,
            sandbox_context: None,
        }
    }

    /// Create a new environment reader tool with sandbox context
    pub fn with_sandbox(
        config: EnvironmentReaderConfig,
        sandbox_context: Arc<SandboxContext>,
    ) -> Self {
        Self {
            metadata: ComponentMetadata::new(
                "environment_reader".to_string(),
                "System environment variable access with security controls".to_string(),
            ),
            config,
            sandbox_context: Some(sandbox_context),
        }
    }

    /// Check if an environment variable is allowed to be read
    fn is_var_allowed(&self, var_name: &str) -> bool {
        // Check sandbox permissions first if available
        if let Some(sandbox) = &self.sandbox_context {
            if !sandbox.security_requirements.env_permissions.is_empty() {
                let allowed = sandbox
                    .security_requirements
                    .env_permissions
                    .iter()
                    .any(|pattern| self.matches_pattern(var_name, pattern));
                if allowed {
                    debug!(
                        "Environment variable '{}' allowed by sandbox permissions",
                        var_name
                    );
                    return true;
                } else {
                    debug!(
                        "Environment variable '{}' not allowed by sandbox permissions",
                        var_name
                    );
                    return false;
                }
            }
        }

        // Check blocked patterns first (takes precedence)
        for pattern in &self.config.blocked_patterns {
            if self.matches_pattern(var_name, pattern) {
                debug!(
                    "Environment variable '{}' blocked by pattern '{}'",
                    var_name, pattern
                );
                return false;
            }
        }

        // Check if read all is allowed
        if self.config.allow_read_all {
            return true;
        }

        // Check default safe variables
        if self
            .config
            .default_safe_vars
            .contains(&var_name.to_string())
        {
            return true;
        }

        // Check allowed patterns
        for pattern in &self.config.allowed_patterns {
            if self.matches_pattern(var_name, pattern) {
                debug!(
                    "Environment variable '{}' allowed by pattern '{}'",
                    var_name, pattern
                );
                return true;
            }
        }

        debug!(
            "Environment variable '{}' not allowed by any pattern",
            var_name
        );
        false
    }

    /// Simple glob pattern matching for environment variable names
    fn matches_pattern(&self, var_name: &str, pattern: &str) -> bool {
        if pattern == "*" {
            return true;
        }

        if pattern.contains('*') {
            if pattern.starts_with('*') && pattern.ends_with('*') {
                let middle = &pattern[1..pattern.len() - 1];
                return var_name.contains(middle);
            } else if let Some(suffix) = pattern.strip_prefix('*') {
                return var_name.ends_with(suffix);
            } else if let Some(prefix) = pattern.strip_suffix('*') {
                return var_name.starts_with(prefix);
            }
        }

        var_name == pattern
    }

    /// Get a single environment variable
    async fn get_single_var(&self, var_name: &str) -> LLMResult<Option<String>> {
        if !self.is_var_allowed(var_name) {
            return Err(LLMSpellError::Security {
                message: format!(
                    "Access to environment variable '{}' is not permitted",
                    var_name
                ),
                violation_type: Some("env_access_denied".to_string()),
            });
        }

        match get_env_var(var_name) {
            Some(value) => {
                info!("Retrieved environment variable: {}", var_name);
                Ok(Some(value))
            }
            None => {
                debug!("Environment variable '{}' not found", var_name);
                Ok(None)
            }
        }
    }

    /// Get all allowed environment variables
    async fn get_all_vars(&self) -> LLMResult<HashMap<String, String>> {
        let all_vars = get_all_env_vars();
        let mut allowed_vars = HashMap::new();
        let mut count = 0;

        for (key, value) in all_vars {
            if count >= self.config.max_variables_returned {
                warn!(
                    "Reached maximum variables limit ({}), stopping enumeration",
                    self.config.max_variables_returned
                );
                break;
            }

            if self.is_var_allowed(&key) {
                allowed_vars.insert(key, value);
                count += 1;
            }
        }

        info!(
            "Retrieved {} allowed environment variables",
            allowed_vars.len()
        );
        Ok(allowed_vars)
    }

    /// Get environment variables matching a pattern
    async fn get_vars_by_pattern(&self, pattern: &str) -> LLMResult<HashMap<String, String>> {
        let all_vars = get_all_env_vars();
        let mut matching_vars = HashMap::new();
        let mut count = 0;

        for (key, value) in all_vars {
            if count >= self.config.max_variables_returned {
                warn!(
                    "Reached maximum variables limit ({}), stopping pattern search",
                    self.config.max_variables_returned
                );
                break;
            }

            if self.matches_pattern(&key, pattern) && self.is_var_allowed(&key) {
                matching_vars.insert(key, value);
                count += 1;
            }
        }

        info!(
            "Found {} environment variables matching pattern '{}'",
            matching_vars.len(),
            pattern
        );
        Ok(matching_vars)
    }

    /// Set an environment variable (if allowed)
    async fn set_var(&self, var_name: &str, value: &str) -> LLMResult<()> {
        if !self.config.allow_set_variables {
            return Err(LLMSpellError::Security {
                message: "Setting environment variables is not permitted by configuration"
                    .to_string(),
                violation_type: Some("env_set_disabled".to_string()),
            });
        }

        if !self.is_var_allowed(var_name) {
            return Err(LLMSpellError::Security {
                message: format!(
                    "Setting environment variable '{}' is not permitted",
                    var_name
                ),
                violation_type: Some("env_set_denied".to_string()),
            });
        }

        match set_env_var_if_allowed(var_name, value) {
            Ok(()) => {
                info!("Set environment variable: {} = {}", var_name, value);
                Ok(())
            }
            Err(e) => Err(LLMSpellError::Tool {
                message: format!("Failed to set environment variable: {}", e),
                tool_name: Some("environment_reader".to_string()),
                source: None,
            }),
        }
    }
}

#[async_trait]
impl BaseAgent for EnvironmentReaderTool {
    fn metadata(&self) -> &ComponentMetadata {
        &self.metadata
    }

    async fn execute(
        &self,
        input: AgentInput,
        _context: ExecutionContext,
    ) -> LLMResult<AgentOutput> {
        // Get parameters using shared utility
        let params = extract_parameters(&input)?;
        let operation = extract_required_string(params, "operation")?;

        let result = match operation {
            "get" => {
                let var_name = extract_required_string(params, "variable_name")?;

                match self.get_single_var(var_name).await? {
                    Some(value) => {
                        let response = ResponseBuilder::success("get")
                            .with_message(format!(
                                "Environment variable '{}' = '{}'",
                                var_name, value
                            ))
                            .with_result(json!({
                                "variable_name": var_name,
                                "value": value,
                                "found": true
                            }))
                            .build();
                        AgentOutput::text(serde_json::to_string_pretty(&response)?)
                    }
                    None => {
                        let response = ResponseBuilder::success("get")
                            .with_message(format!("Environment variable '{}' not found", var_name))
                            .with_result(json!({
                                "variable_name": var_name,
                                "value": null,
                                "found": false
                            }))
                            .build();
                        AgentOutput::text(serde_json::to_string_pretty(&response)?)
                    }
                }
            }
            "list" => {
                let vars = self.get_all_vars().await?;
                let response = ResponseBuilder::success("list")
                    .with_message(format!(
                        "Found {} allowed environment variables",
                        vars.len()
                    ))
                    .with_result(json!({
                        "variables": vars,
                        "count": vars.len()
                    }))
                    .build();
                AgentOutput::text(serde_json::to_string_pretty(&response)?)
            }
            "pattern" => {
                let pattern = extract_required_string(params, "pattern")?;

                let vars = self.get_vars_by_pattern(pattern).await?;
                let response = ResponseBuilder::success("pattern")
                    .with_message(format!(
                        "Found {} environment variables matching pattern '{}'",
                        vars.len(),
                        pattern
                    ))
                    .with_result(json!({
                        "pattern": pattern,
                        "variables": vars,
                        "count": vars.len()
                    }))
                    .build();
                AgentOutput::text(serde_json::to_string_pretty(&response)?)
            }
            "set" => {
                let var_name = extract_required_string(params, "variable_name")?;
                let value = extract_required_string(params, "value")?;

                self.set_var(var_name, value).await?;
                let response = ResponseBuilder::success("set")
                    .with_message(format!(
                        "Set environment variable '{}' = '{}'",
                        var_name, value
                    ))
                    .with_result(json!({
                        "variable_name": var_name,
                        "value": value,
                        "success": true
                    }))
                    .build();
                AgentOutput::text(serde_json::to_string_pretty(&response)?)
            }
            _ => {
                return Err(LLMSpellError::Validation {
                    message: format!(
                        "Unknown operation: '{}'. Supported operations: get, list, pattern, set",
                        operation
                    ),
                    field: Some("operation".to_string()),
                });
            }
        };

        Ok(result)
    }

    async fn validate_input(&self, input: &AgentInput) -> LLMResult<()> {
        if input.text.is_empty() {
            return Err(LLMSpellError::Validation {
                message: "Input prompt cannot be empty".to_string(),
                field: Some("prompt".to_string()),
            });
        }
        Ok(())
    }

    async fn handle_error(&self, error: LLMSpellError) -> LLMResult<AgentOutput> {
        Ok(AgentOutput::text(format!(
            "Environment reader error: {}",
            error
        )))
    }
}

#[async_trait]
impl Tool for EnvironmentReaderTool {
    fn category(&self) -> ToolCategory {
        ToolCategory::System
    }

    fn security_level(&self) -> SecurityLevel {
        SecurityLevel::Restricted // Environment access requires restricted security
    }

    fn schema(&self) -> ToolSchema {
        ToolSchema::new(
            "environment_reader".to_string(),
            "Access system environment variables with security controls".to_string(),
        )
        .with_parameter(ParameterDef {
            name: "operation".to_string(),
            param_type: ParameterType::String,
            description: "Operation to perform: get, list, pattern, set".to_string(),
            required: true,
            default: None,
        })
        .with_parameter(ParameterDef {
            name: "variable_name".to_string(),
            param_type: ParameterType::String,
            description: "Name of environment variable (required for get and set operations)"
                .to_string(),
            required: false,
            default: None,
        })
        .with_parameter(ParameterDef {
            name: "value".to_string(),
            param_type: ParameterType::String,
            description: "Value to set (required for set operation)".to_string(),
            required: false,
            default: None,
        })
        .with_parameter(ParameterDef {
            name: "pattern".to_string(),
            param_type: ParameterType::String,
            description: "Glob pattern to match variable names (required for pattern operation)"
                .to_string(),
            required: false,
            default: None,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use llmspell_core::traits::tool::{ResourceLimits, SecurityRequirements};
    use llmspell_testing::tool_helpers::{create_test_tool, create_test_tool_input};
    use std::collections::HashMap;

    fn create_test_environment_reader() -> EnvironmentReaderTool {
        let config = EnvironmentReaderConfig::default();
        EnvironmentReaderTool::new(config)
    }

    fn create_test_tool_with_sandbox() -> EnvironmentReaderTool {
        let security_requirements = SecurityRequirements {
            level: SecurityLevel::Restricted,
            file_permissions: vec![],
            network_permissions: vec![],
            env_permissions: vec!["TEST_*".to_string(), "PATH".to_string()],
            custom_requirements: HashMap::new(),
        };
        let resource_limits = ResourceLimits::default();
        let sandbox_context = Arc::new(SandboxContext::new(
            "test_env_reader".to_string(),
            security_requirements,
            resource_limits,
        ));

        let config = EnvironmentReaderConfig::default();
        EnvironmentReaderTool::with_sandbox(config, sandbox_context)
    }
    #[tokio::test]
    async fn test_get_existing_variable() {
        let tool = create_test_environment_reader();

        // Test getting PATH variable (should be allowed by default)
        let input = create_test_tool_input(vec![("operation", "get"), ("variable_name", "PATH")]);

        let result = tool
            .execute(input, ExecutionContext::default())
            .await
            .unwrap();
        assert!(result.text.contains("PATH"));
    }
    #[tokio::test]
    async fn test_get_nonexistent_variable() {
        let mut config = EnvironmentReaderConfig::default();
        // Allow reading variables starting with NONEXISTENT for this test
        config.allowed_patterns.push("NONEXISTENT*".to_string());
        let tool = EnvironmentReaderTool::new(config);

        let input = create_test_tool_input(vec![
            ("operation", "get"),
            ("variable_name", "NONEXISTENT_VAR_12345"),
        ]);

        let result = tool
            .execute(input, ExecutionContext::default())
            .await
            .unwrap();
        assert!(result.text.contains("not found"));
    }
    #[tokio::test]
    async fn test_get_blocked_variable() {
        let tool = create_test_environment_reader();

        let input = create_test_tool_input(vec![
            ("operation", "get"),
            ("variable_name", "SECRET_PASSWORD"),
        ]);

        let result = tool.execute(input, ExecutionContext::default()).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("not permitted"));
    }
    #[tokio::test]
    async fn test_list_variables() {
        let tool = create_test_environment_reader();

        let input = create_test_tool_input(vec![("operation", "list")]);

        let result = tool
            .execute(input, ExecutionContext::default())
            .await
            .unwrap();
        assert!(result.text.contains("Found"));
        assert!(result.text.contains("environment variables"));
    }
    #[tokio::test]
    async fn test_pattern_matching() {
        let tool = create_test_environment_reader();

        let input = create_test_tool_input(vec![("operation", "pattern"), ("pattern", "PATH*")]);

        let result = tool
            .execute(input, ExecutionContext::default())
            .await
            .unwrap();
        assert!(result.text.contains("Found"));
    }
    #[tokio::test]
    async fn test_set_variable_disabled() {
        let tool = create_test_environment_reader();

        let input = create_test_tool_input(vec![
            ("operation", "set"),
            ("variable_name", "TEST_VAR"),
            ("value", "test_value"),
        ]);

        let result = tool.execute(input, ExecutionContext::default()).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("not permitted"));
    }
    #[tokio::test]
    async fn test_set_variable_enabled() {
        let allowed_patterns = vec!["TEST_*".to_string()];
        let config = EnvironmentReaderConfig {
            allow_set_variables: true,
            allowed_patterns,
            ..Default::default()
        };
        let tool = EnvironmentReaderTool::new(config);

        let input = create_test_tool_input(vec![
            ("operation", "set"),
            ("variable_name", "TEST_VAR_12345"),
            ("value", "test_value"),
        ]);

        let result = tool
            .execute(input, ExecutionContext::default())
            .await
            .unwrap();
        assert!(result.text.contains("Set environment variable"));
    }
    #[tokio::test]
    async fn test_invalid_operation() {
        let tool = create_test_environment_reader();

        let input = create_test_tool_input(vec![("operation", "invalid")]);

        let result = tool.execute(input, ExecutionContext::default()).await;
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Unknown operation"));
    }
    #[tokio::test]
    async fn test_missing_parameters() {
        let tool = create_test_environment_reader();

        // Missing operation
        let input1 = create_test_input("Missing operation", json!({}));
        let result1 = tool.execute(input1, ExecutionContext::default()).await;
        assert!(result1.is_err());

        // Missing variable_name for get operation
        let input2 = create_test_input(
            "Missing variable name",
            json!({
                "operation": "get"
            }),
        );
        let result2 = tool.execute(input2, ExecutionContext::default()).await;
        assert!(result2.is_err());

        // Missing pattern for pattern operation
        let input3 = create_test_input(
            "Missing pattern",
            json!({
                "operation": "pattern"
            }),
        );
        let result3 = tool.execute(input3, ExecutionContext::default()).await;
        assert!(result3.is_err());
    }
    #[tokio::test]
    async fn test_sandbox_permissions() {
        let tool = create_test_tool_with_sandbox();

        // Should allow TEST_* variables due to sandbox permissions
        let input1 = create_test_input(
            "Get test variable",
            json!({
                "operation": "get",
                "variable_name": "TEST_ALLOWED"
            }),
        );
        let result1 = tool.execute(input1, ExecutionContext::default()).await;
        // Should succeed (even if variable doesn't exist)
        assert!(
            result1.is_ok(),
            "TEST_* variable should be allowed by sandbox"
        );

        // Should allow PATH due to sandbox permissions
        let input2 = create_test_input(
            "Get PATH",
            json!({
                "operation": "get",
                "variable_name": "PATH"
            }),
        );
        let result2 = tool.execute(input2, ExecutionContext::default()).await;
        assert!(result2.is_ok(), "PATH should be allowed by sandbox");

        // Should deny HOME even though it's in default safe vars (sandbox overrides)
        let input3 = create_test_input(
            "Get HOME",
            json!({
                "operation": "get",
                "variable_name": "HOME"
            }),
        );
        let result3 = tool.execute(input3, ExecutionContext::default()).await;
        assert!(
            result3.is_err(),
            "HOME should be denied when not in sandbox permissions"
        );
    }
    #[tokio::test]
    async fn test_pattern_matching_logic() {
        let tool = create_test_environment_reader();

        // Test exact match
        assert!(tool.matches_pattern("PATH", "PATH"));
        assert!(!tool.matches_pattern("PATH", "HOME"));

        // Test wildcard patterns
        assert!(tool.matches_pattern("PATH_EXTRA", "PATH*"));
        assert!(tool.matches_pattern("MY_PATH", "*PATH"));
        assert!(tool.matches_pattern("MY_PATH_EXTRA", "*PATH*"));
        assert!(tool.matches_pattern("ANYTHING", "*"));

        // Test non-matches
        assert!(!tool.matches_pattern("HOME", "PATH*"));
        assert!(!tool.matches_pattern("HOME", "*PATH"));
    }
    #[tokio::test]
    async fn test_blocked_patterns_precedence() {
        let tool = create_test_environment_reader();

        // SECRET_KEY should be blocked even though it might match allowed patterns
        assert!(!tool.is_var_allowed("SECRET_KEY"));
        assert!(!tool.is_var_allowed("MY_PASSWORD"));
        assert!(!tool.is_var_allowed("API_TOKEN"));
        assert!(!tool.is_var_allowed("AWS_ACCESS_KEY"));

        // Safe variables should be allowed
        assert!(tool.is_var_allowed("PATH"));
        assert!(tool.is_var_allowed("HOME"));
        assert!(tool.is_var_allowed("USER"));
    }
    #[tokio::test]
    async fn test_tool_metadata() {
        let tool = create_test_environment_reader();

        let metadata = tool.metadata();
        assert_eq!(metadata.name, "environment_reader");
        assert!(metadata.description.contains("environment variable"));

        let schema = tool.schema();
        assert_eq!(schema.name, "environment_reader");
        assert_eq!(tool.category(), ToolCategory::System);
        assert_eq!(tool.security_level(), SecurityLevel::Restricted);

        // Check required parameters
        let required_params = schema.required_parameters();
        assert!(required_params.contains(&"operation".to_string()));
        assert_eq!(required_params.len(), 1);
    }
}
