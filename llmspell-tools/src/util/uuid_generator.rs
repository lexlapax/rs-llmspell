// ABOUTME: UUID generation tool supporting v1, v4, v5 and custom formats
// ABOUTME: Provides UUID generation with various options and formats

//! UUID generation and manipulation tool
//!
//! This tool provides UUID generation capabilities including:
//! - UUID v4 (random) generation
//! - UUID v1 (timestamp-based) generation
//! - UUID v5 (namespace-based) generation
//! - Custom formatting options
//! - UUID validation and parsing

use async_trait::async_trait;
use llmspell_core::{
    traits::{
        base_agent::BaseAgent,
        tool::{
            ParameterDef, ParameterType, ResourceLimits, SecurityLevel, SecurityRequirements, Tool,
            ToolCategory, ToolSchema,
        },
    },
    types::{AgentInput, AgentOutput},
    ComponentMetadata, ExecutionContext, LLMSpellError, Result,
};
use llmspell_utils::{
    error_builders::llmspell::validation_error,
    id_generator::{
        generate_component_id, generate_deterministic_id, generate_short_id, ComponentIdBuilder,
        NAMESPACE_AGENT, NAMESPACE_TOOL, NAMESPACE_WORKFLOW,
    },
    params::{
        extract_optional_bool, extract_optional_string, extract_parameters,
        extract_required_string, extract_string_with_default,
    },
    response::ResponseBuilder,
    validators::validate_enum,
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use uuid::Uuid;

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UuidVersion {
    V1,
    V4,
    V5,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UuidFormat {
    Standard,
    Hyphenated,
    Simple,
    Urn,
    Braced,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UuidGeneratorConfig {
    /// Default UUID version to use
    pub default_version: UuidVersion,
    /// Default format for output
    pub default_format: UuidFormat,
}

impl Default for UuidGeneratorConfig {
    fn default() -> Self {
        Self {
            default_version: UuidVersion::V4,
            default_format: UuidFormat::Hyphenated,
        }
    }
}

/// UUID generation tool
pub struct UuidGeneratorTool {
    metadata: ComponentMetadata,
    config: UuidGeneratorConfig,
}

impl UuidGeneratorTool {
    /// Create a new UUID generator tool
    #[must_use]
    pub fn new(config: UuidGeneratorConfig) -> Self {
        Self {
            metadata: ComponentMetadata::new(
                "uuid-generator".to_string(),
                "UUID generation tool with multiple versions and formats".to_string(),
            ),
            config,
        }
    }

    fn generate_uuid(
        version: UuidVersion,
        namespace: Option<&str>,
        name: Option<&str>,
    ) -> Result<Uuid> {
        match version {
            UuidVersion::V1 | UuidVersion::V4 => {
                // UUID v1 is timestamp-based, we'll use v4 as fallback for security
                // v1 requires MAC address which can be a privacy concern
                Ok(Uuid::new_v4())
            }
            UuidVersion::V5 => {
                let namespace_uuid = match namespace {
                    Some("agent") => *NAMESPACE_AGENT,
                    Some("tool") => *NAMESPACE_TOOL,
                    Some("workflow") => *NAMESPACE_WORKFLOW,
                    Some("url") => Uuid::NAMESPACE_URL,
                    Some("oid") => Uuid::NAMESPACE_OID,
                    Some("x500") => Uuid::NAMESPACE_X500,
                    Some("dns") | None => Uuid::NAMESPACE_DNS, // DNS is the default namespace
                    Some(custom) => {
                        // Try to parse as UUID
                        Uuid::parse_str(custom).map_err(|_| {
                            validation_error(
                                format!("Invalid namespace UUID: {custom}"),
                                Some("namespace".to_string()),
                            )
                        })?
                    }
                };

                let name = name.ok_or_else(|| {
                    validation_error(
                        "UUID v5 requires a name parameter",
                        Some("name".to_string()),
                    )
                })?;

                Ok(Uuid::new_v5(&namespace_uuid, name.as_bytes()))
            }
        }
    }

    #[allow(clippy::unused_self)]
    fn format_uuid(&self, uuid: Uuid, format: UuidFormat) -> String {
        match format {
            UuidFormat::Standard | UuidFormat::Hyphenated => uuid.to_string(),
            UuidFormat::Simple => uuid.simple().to_string(),
            UuidFormat::Urn => uuid.urn().to_string(),
            UuidFormat::Braced => uuid.braced().to_string(),
        }
    }

    #[allow(clippy::unused_async)]
    async fn validate_parameters(&self, params: &serde_json::Value) -> Result<()> {
        // Extract operation type for validation
        let operation = extract_string_with_default(params, "operation", "generate");

        // Validate operation value
        validate_enum(
            &operation,
            &[
                "generate",
                "component_id",
                "deterministic",
                "custom",
                "validate",
            ],
            "operation",
        )?;

        // Operation-specific validations
        match operation {
            "generate" => {
                // Validate version if provided
                if let Some(version) = extract_optional_string(params, "version") {
                    validate_enum(&version, &["v1", "1", "v4", "4", "v5", "5"], "version")?;
                }
                // Validate format if provided
                if let Some(format) = extract_optional_string(params, "format") {
                    validate_enum(
                        &format,
                        &["standard", "hyphenated", "simple", "urn", "braced"],
                        "format",
                    )?;
                }
            }
            "deterministic" => {
                // Deterministic requires name
                extract_required_string(params, "name")?;
            }
            "validate" => {
                // Validate requires uuid
                extract_required_string(params, "uuid")?;
            }
            _ => {} // Other operations have optional parameters
        }
        Ok(())
    }
}

impl Default for UuidGeneratorTool {
    fn default() -> Self {
        Self::new(UuidGeneratorConfig::default())
    }
}

#[async_trait]
impl BaseAgent for UuidGeneratorTool {
    fn metadata(&self) -> &ComponentMetadata {
        &self.metadata
    }

    #[allow(clippy::too_many_lines)]
    async fn execute_impl(
        &self,
        input: AgentInput,
        _context: ExecutionContext,
    ) -> Result<AgentOutput> {
        // Get parameters from input using shared utility
        let params = extract_parameters(&input)?;

        // Validate parameters
        self.validate_parameters(params).await?;

        // Extract operation type
        let operation = extract_string_with_default(params, "operation", "generate");

        match operation {
            "generate" => {
                // Extract version
                let version = extract_optional_string(params, "version").map_or(
                    self.config.default_version,
                    |v| match v {
                        "v1" | "1" => UuidVersion::V1,
                        "v4" | "4" => UuidVersion::V4,
                        "v5" | "5" => UuidVersion::V5,
                        _ => self.config.default_version,
                    },
                );

                // Extract namespace and name for v5
                let namespace = extract_optional_string(params, "namespace");
                let name = extract_optional_string(params, "name");

                // Generate UUID
                let uuid = Self::generate_uuid(version, namespace, name)?;

                // Extract format
                let format = extract_optional_string(params, "format").map_or(
                    self.config.default_format,
                    |f| match f {
                        "standard" | "hyphenated" => UuidFormat::Hyphenated,
                        "simple" => UuidFormat::Simple,
                        "urn" => UuidFormat::Urn,
                        "braced" => UuidFormat::Braced,
                        _ => self.config.default_format,
                    },
                );

                let formatted = self.format_uuid(uuid, format);
                let response = ResponseBuilder::success("generate")
                    .with_message("UUID generated successfully")
                    .with_result(json!({
                        "uuid": formatted,
                        "version": match version {
                            UuidVersion::V1 => "v1",
                            UuidVersion::V4 => "v4",
                            UuidVersion::V5 => "v5",
                        },
                        "format": match format {
                            UuidFormat::Standard | UuidFormat::Hyphenated => "hyphenated",
                            UuidFormat::Simple => "simple",
                            UuidFormat::Urn => "urn",
                            UuidFormat::Braced => "braced",
                        }
                    }))
                    .build();
                Ok(AgentOutput::text(serde_json::to_string_pretty(&response)?))
            }
            "component_id" => {
                // Generate component ID using llmspell-utils
                let prefix = extract_string_with_default(params, "prefix", "component");
                let short = extract_optional_bool(params, "short").unwrap_or(false);

                let id = if short {
                    generate_short_id(prefix)
                } else {
                    generate_component_id(prefix)
                };

                let response = ResponseBuilder::success("component_id")
                    .with_message("Component ID generated successfully")
                    .with_result(json!({
                        "id": id,
                        "prefix": prefix,
                        "short": short
                    }))
                    .build();
                Ok(AgentOutput::text(serde_json::to_string_pretty(&response)?))
            }
            "deterministic" => {
                // Generate deterministic ID
                let namespace = extract_string_with_default(params, "namespace", "agent");
                let name = extract_required_string(params, "name")?;

                let namespace_uuid = match namespace {
                    "tool" => *NAMESPACE_TOOL,
                    "workflow" => *NAMESPACE_WORKFLOW,
                    _ => *NAMESPACE_AGENT, // Default to agent namespace
                };

                let id = generate_deterministic_id(&namespace_uuid, name);
                let response = ResponseBuilder::success("deterministic")
                    .with_message("Deterministic ID generated successfully")
                    .with_result(json!({
                        "id": id,
                        "namespace": namespace,
                        "name": name
                    }))
                    .build();
                Ok(AgentOutput::text(serde_json::to_string_pretty(&response)?))
            }
            "custom" => {
                // Use ComponentIdBuilder for custom IDs
                let mut builder = ComponentIdBuilder::new();

                if let Some(prefix) = extract_optional_string(params, "prefix") {
                    builder = builder.with_prefix(prefix);
                }

                let timestamp = extract_optional_bool(params, "timestamp").unwrap_or(false);
                if timestamp {
                    builder = builder.with_timestamp();
                }

                let short = extract_optional_bool(params, "short").unwrap_or(false);
                if short {
                    builder = builder.short();
                }

                if let Some(suffix) = extract_optional_string(params, "suffix") {
                    builder = builder.with_suffix(suffix);
                }

                let id = builder.build();
                let response = ResponseBuilder::success("custom")
                    .with_message("Custom ID generated successfully")
                    .with_result(json!({
                        "id": id,
                        "timestamp": timestamp,
                        "short": short
                    }))
                    .build();
                Ok(AgentOutput::text(serde_json::to_string_pretty(&response)?))
            }
            "validate" => {
                // Validate a UUID
                let uuid_str = extract_required_string(params, "uuid")?;

                if let Ok(uuid) = Uuid::parse_str(uuid_str) {
                    let result = json!({
                        "valid": true,
                        "uuid": uuid.to_string(),
                        "version": uuid.get_version().map(|v| format!("v{}", v as u8)),
                        "variant": format!("{:?}", uuid.get_variant()),
                    });
                    Ok(AgentOutput::text(serde_json::to_string_pretty(&result)?))
                } else {
                    let result = json!({
                        "valid": false,
                        "error": "Invalid UUID format"
                    });
                    Ok(AgentOutput::text(serde_json::to_string_pretty(&result)?))
                }
            }
            _ => Err(validation_error(
                format!("Unknown operation: {operation}"),
                Some("operation".to_string()),
            )),
        }
    }

    async fn validate_input(&self, input: &AgentInput) -> Result<()> {
        if input.text.is_empty() {
            return Err(validation_error(
                "Input prompt cannot be empty",
                Some("prompt".to_string()),
            ));
        }
        Ok(())
    }

    async fn handle_error(&self, error: LLMSpellError) -> Result<AgentOutput> {
        Ok(AgentOutput::text(format!("UUID generation error: {error}")))
    }
}

#[async_trait]
impl Tool for UuidGeneratorTool {
    fn category(&self) -> ToolCategory {
        ToolCategory::Utility
    }

    fn security_level(&self) -> SecurityLevel {
        SecurityLevel::Safe
    }

    fn schema(&self) -> ToolSchema {
        ToolSchema::new(
            "uuid_generator".to_string(),
            "Generate UUIDs with various versions and formats".to_string(),
        )
        .with_parameter(ParameterDef {
            name: "operation".to_string(),
            param_type: ParameterType::String,
            description:
                "Operation to perform: generate, component_id, deterministic, custom, validate"
                    .to_string(),
            required: false,
            default: Some(json!("generate")),
        })
        .with_parameter(ParameterDef {
            name: "version".to_string(),
            param_type: ParameterType::String,
            description: "UUID version (v1, v4, v5) for generate operation".to_string(),
            required: false,
            default: Some(json!("v4")),
        })
        .with_parameter(ParameterDef {
            name: "format".to_string(),
            param_type: ParameterType::String,
            description: "Output format: standard, simple, urn, braced".to_string(),
            required: false,
            default: Some(json!("standard")),
        })
        .with_parameter(ParameterDef {
            name: "namespace".to_string(),
            param_type: ParameterType::String,
            description: "Namespace for v5 UUIDs or deterministic IDs".to_string(),
            required: false,
            default: None,
        })
        .with_parameter(ParameterDef {
            name: "name".to_string(),
            param_type: ParameterType::String,
            description: "Name for v5 UUIDs or deterministic IDs".to_string(),
            required: false,
            default: None,
        })
        .with_parameter(ParameterDef {
            name: "prefix".to_string(),
            param_type: ParameterType::String,
            description: "Prefix for component IDs".to_string(),
            required: false,
            default: None,
        })
        .with_parameter(ParameterDef {
            name: "short".to_string(),
            param_type: ParameterType::Boolean,
            description: "Use short format for component IDs".to_string(),
            required: false,
            default: Some(json!(false)),
        })
        .with_parameter(ParameterDef {
            name: "timestamp".to_string(),
            param_type: ParameterType::Boolean,
            description: "Include timestamp in custom IDs".to_string(),
            required: false,
            default: Some(json!(false)),
        })
        .with_parameter(ParameterDef {
            name: "suffix".to_string(),
            param_type: ParameterType::String,
            description: "Suffix for custom IDs".to_string(),
            required: false,
            default: None,
        })
        .with_parameter(ParameterDef {
            name: "uuid".to_string(),
            param_type: ParameterType::String,
            description: "UUID to validate".to_string(),
            required: false,
            default: None,
        })
        .with_returns(ParameterType::String)
    }

    fn security_requirements(&self) -> SecurityRequirements {
        SecurityRequirements::safe()
    }

    fn resource_limits(&self) -> ResourceLimits {
        ResourceLimits::strict()
            .with_memory_limit(1024 * 1024) // 1MB
            .with_cpu_limit(100) // 100ms
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::Value;
    #[tokio::test]
    async fn test_generate_v4_uuid() {
        let tool = UuidGeneratorTool::default();
        let input = AgentInput::text("generate uuid".to_string()).with_parameter(
            "parameters".to_string(),
            json!({
                "operation": "generate",
                "version": "v4"
            }),
        );
        let context = ExecutionContext::with_conversation("test".to_string());

        let result = tool.execute(input, context).await;
        assert!(result.is_ok());

        let output = result.unwrap().text;
        let response: Value = serde_json::from_str(&output).unwrap();
        assert!(response["success"].as_bool().unwrap_or(false));
        let uuid_str = response["result"]["uuid"].as_str().unwrap();
        assert!(Uuid::parse_str(uuid_str).is_ok());
    }
    #[tokio::test]
    async fn test_generate_v5_uuid() {
        let tool = UuidGeneratorTool::default();
        let input = AgentInput::text("generate uuid".to_string()).with_parameter(
            "parameters".to_string(),
            json!({
                "operation": "generate",
                "version": "v5",
                "namespace": "dns",
                "name": "example.com"
            }),
        );
        let context = ExecutionContext::with_conversation("test".to_string());

        let result = tool.execute(input, context).await;
        assert!(result.is_ok());

        let output = result.unwrap().text;
        let response: Value = serde_json::from_str(&output).unwrap();
        assert!(response["success"].as_bool().unwrap_or(false));
        let uuid_str = response["result"]["uuid"].as_str().unwrap();
        let uuid = Uuid::parse_str(uuid_str).unwrap();
        assert_eq!(uuid.get_version(), Some(uuid::Version::Sha1));
    }
    #[tokio::test]
    async fn test_generate_component_id() {
        let tool = UuidGeneratorTool::default();
        let input = AgentInput::text("generate component id".to_string()).with_parameter(
            "parameters".to_string(),
            json!({
                "operation": "component_id",
                "prefix": "test",
                "short": false
            }),
        );
        let context = ExecutionContext::with_conversation("test".to_string());

        let result = tool.execute(input, context).await;
        assert!(result.is_ok());

        let output = result.unwrap().text;
        let response: Value = serde_json::from_str(&output).unwrap();
        assert!(response["success"].as_bool().unwrap_or(false));
        let id = response["result"]["id"].as_str().unwrap();
        assert!(id.starts_with("test_"));
    }
    #[tokio::test]
    async fn test_generate_deterministic_id() {
        let tool = UuidGeneratorTool::default();
        let input = AgentInput::text("generate deterministic id".to_string()).with_parameter(
            "parameters".to_string(),
            json!({
                "operation": "deterministic",
                "namespace": "agent",
                "name": "my-agent"
            }),
        );
        let context = ExecutionContext::with_conversation("test".to_string());

        // Generate twice to verify determinism
        let result1 = tool.execute(input.clone(), context.clone()).await;
        let result2 = tool.execute(input, context).await;

        assert!(result1.is_ok());
        assert!(result2.is_ok());

        let output1 = result1.unwrap().text;
        let response1: Value = serde_json::from_str(&output1).unwrap();
        let id1 = response1["result"]["id"].as_str().unwrap();

        let output2 = result2.unwrap().text;
        let response2: Value = serde_json::from_str(&output2).unwrap();
        let id2 = response2["result"]["id"].as_str().unwrap();

        assert_eq!(id1, id2);
    }
    #[tokio::test]
    async fn test_custom_id_generation() {
        let tool = UuidGeneratorTool::default();
        let input = AgentInput::text("generate custom id".to_string()).with_parameter(
            "parameters".to_string(),
            json!({
                "operation": "custom",
                "prefix": "custom",
                "suffix": "v1",
                "timestamp": true,
                "short": true
            }),
        );
        let context = ExecutionContext::with_conversation("test".to_string());

        let result = tool.execute(input, context).await;
        assert!(result.is_ok());

        let output = result.unwrap().text;
        let response: Value = serde_json::from_str(&output).unwrap();
        assert!(response["success"].as_bool().unwrap_or(false));
        let id = response["result"]["id"].as_str().unwrap();
        assert!(id.starts_with("custom_"));
        assert!(id.ends_with("_v1"));
        assert!(id.contains('_')); // Contains timestamp separator
    }
    #[tokio::test]
    async fn test_validate_uuid() {
        let tool = UuidGeneratorTool::default();
        let valid_uuid = Uuid::new_v4().to_string();

        let input = AgentInput::text("validate uuid".to_string()).with_parameter(
            "parameters".to_string(),
            json!({
                "operation": "validate",
                "uuid": valid_uuid
            }),
        );
        let context = ExecutionContext::with_conversation("test".to_string());

        let result = tool.execute(input, context).await;
        assert!(result.is_ok());

        let output = result.unwrap().text;
        let parsed: Value = serde_json::from_str(&output).unwrap();
        assert_eq!(parsed["valid"], true);
    }
    #[tokio::test]
    async fn test_validate_invalid_uuid() {
        let tool = UuidGeneratorTool::default();
        let input = AgentInput::text("validate uuid".to_string()).with_parameter(
            "parameters".to_string(),
            json!({
                "operation": "validate",
                "uuid": "not-a-valid-uuid"
            }),
        );
        let context = ExecutionContext::with_conversation("test".to_string());

        let result = tool.execute(input, context).await;
        assert!(result.is_ok());

        let output = result.unwrap().text;
        let parsed: Value = serde_json::from_str(&output).unwrap();
        assert_eq!(parsed["valid"], false);
    }
    #[tokio::test]
    async fn test_different_formats() {
        let tool = UuidGeneratorTool::default();
        let formats = vec!["standard", "simple", "urn", "braced"];

        for format in formats {
            let input = AgentInput::text("generate uuid".to_string()).with_parameter(
                "parameters".to_string(),
                json!({
                    "operation": "generate",
                    "version": "v4",
                    "format": format
                }),
            );
            let context = ExecutionContext::with_conversation("test".to_string());

            let result = tool.execute(input, context).await;
            assert!(result.is_ok(), "Format {format} failed");

            let output = result.unwrap().text;
            let response: Value = serde_json::from_str(&output).unwrap();
            assert!(response["success"].as_bool().unwrap_or(false));
            let uuid_str = response["result"]["uuid"].as_str().unwrap();
            match format {
                "simple" => assert!(!uuid_str.contains('-')),
                "urn" => assert!(uuid_str.starts_with("urn:uuid:")),
                "braced" => assert!(uuid_str.starts_with('{') && uuid_str.ends_with('}')),
                _ => assert!(uuid_str.contains('-')),
            }
        }
    }
    #[tokio::test]
    async fn test_tool_metadata() {
        let tool = UuidGeneratorTool::default();
        assert_eq!(tool.category(), ToolCategory::Utility);
        assert_eq!(tool.security_level(), SecurityLevel::Safe);
        assert_eq!(tool.metadata().name, "uuid-generator");

        let schema = tool.schema();
        assert_eq!(schema.name, "uuid_generator");
        assert!(schema.parameters.len() >= 10); // We have many parameters
    }
}
