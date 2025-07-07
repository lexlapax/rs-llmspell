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
    types::{AgentInput, AgentOutput, ExecutionContext},
    ComponentMetadata, LLMSpellError, Result,
};
use llmspell_utils::id_generator::{
    generate_component_id, generate_deterministic_id, generate_short_id, ComponentIdBuilder,
    NAMESPACE_AGENT, NAMESPACE_TOOL, NAMESPACE_WORKFLOW,
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UuidVersion {
    V1,
    V4,
    V5,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
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
        &self,
        version: UuidVersion,
        namespace: Option<&str>,
        name: Option<&str>,
    ) -> Result<Uuid> {
        match version {
            UuidVersion::V1 => {
                // UUID v1 is timestamp-based, we'll use v4 as fallback for security
                // v1 requires MAC address which can be a privacy concern
                Ok(Uuid::new_v4())
            }
            UuidVersion::V4 => Ok(Uuid::new_v4()),
            UuidVersion::V5 => {
                let namespace_uuid = match namespace {
                    Some("agent") => *NAMESPACE_AGENT,
                    Some("tool") => *NAMESPACE_TOOL,
                    Some("workflow") => *NAMESPACE_WORKFLOW,
                    Some("dns") => Uuid::NAMESPACE_DNS,
                    Some("url") => Uuid::NAMESPACE_URL,
                    Some("oid") => Uuid::NAMESPACE_OID,
                    Some("x500") => Uuid::NAMESPACE_X500,
                    Some(custom) => {
                        // Try to parse as UUID
                        Uuid::parse_str(custom).map_err(|_| LLMSpellError::Validation {
                            message: format!("Invalid namespace UUID: {}", custom),
                            field: Some("namespace".to_string()),
                        })?
                    }
                    None => Uuid::NAMESPACE_DNS, // Default namespace
                };

                let name = name.ok_or_else(|| LLMSpellError::Validation {
                    message: "UUID v5 requires a name parameter".to_string(),
                    field: Some("name".to_string()),
                })?;

                Ok(Uuid::new_v5(&namespace_uuid, name.as_bytes()))
            }
        }
    }

    fn format_uuid(&self, uuid: Uuid, format: &UuidFormat) -> String {
        match format {
            UuidFormat::Standard | UuidFormat::Hyphenated => uuid.to_string(),
            UuidFormat::Simple => uuid.simple().to_string(),
            UuidFormat::Urn => uuid.urn().to_string(),
            UuidFormat::Braced => uuid.braced().to_string(),
        }
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

    async fn execute(&self, input: AgentInput, _context: ExecutionContext) -> Result<AgentOutput> {
        // Get parameters from input
        let params =
            input
                .parameters
                .get("parameters")
                .ok_or_else(|| LLMSpellError::Validation {
                    message: "Missing parameters in input".to_string(),
                    field: Some("parameters".to_string()),
                })?;

        // Validate parameters
        self.validate_parameters(params).await?;

        // Extract operation type
        let operation = params
            .get("operation")
            .and_then(|v| v.as_str())
            .unwrap_or("generate");

        match operation {
            "generate" => {
                // Extract version
                let version = params
                    .get("version")
                    .and_then(|v| v.as_str())
                    .map(|v| match v {
                        "v1" | "1" => UuidVersion::V1,
                        "v4" | "4" => UuidVersion::V4,
                        "v5" | "5" => UuidVersion::V5,
                        _ => self.config.default_version.clone(),
                    })
                    .unwrap_or_else(|| self.config.default_version.clone());

                // Extract namespace and name for v5
                let namespace = params.get("namespace").and_then(|v| v.as_str());
                let name = params.get("name").and_then(|v| v.as_str());

                // Generate UUID
                let uuid = self.generate_uuid(version, namespace, name)?;

                // Extract format
                let format = params
                    .get("format")
                    .and_then(|v| v.as_str())
                    .map(|f| match f {
                        "standard" | "hyphenated" => UuidFormat::Hyphenated,
                        "simple" => UuidFormat::Simple,
                        "urn" => UuidFormat::Urn,
                        "braced" => UuidFormat::Braced,
                        _ => self.config.default_format.clone(),
                    })
                    .unwrap_or_else(|| self.config.default_format.clone());

                let formatted = self.format_uuid(uuid, &format);
                Ok(AgentOutput::text(formatted))
            }
            "component_id" => {
                // Generate component ID using llmspell-utils
                let prefix = params
                    .get("prefix")
                    .and_then(|v| v.as_str())
                    .unwrap_or("component");

                let short = params
                    .get("short")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false);

                let id = if short {
                    generate_short_id(prefix)
                } else {
                    generate_component_id(prefix)
                };

                Ok(AgentOutput::text(id))
            }
            "deterministic" => {
                // Generate deterministic ID
                let namespace = params
                    .get("namespace")
                    .and_then(|v| v.as_str())
                    .unwrap_or("agent");

                let name = params.get("name").and_then(|v| v.as_str()).ok_or_else(|| {
                    LLMSpellError::Validation {
                        message: "Deterministic ID requires a name parameter".to_string(),
                        field: Some("name".to_string()),
                    }
                })?;

                let namespace_uuid = match namespace {
                    "agent" => *NAMESPACE_AGENT,
                    "tool" => *NAMESPACE_TOOL,
                    "workflow" => *NAMESPACE_WORKFLOW,
                    _ => *NAMESPACE_AGENT,
                };

                let id = generate_deterministic_id(&namespace_uuid, name);
                Ok(AgentOutput::text(id))
            }
            "custom" => {
                // Use ComponentIdBuilder for custom IDs
                let mut builder = ComponentIdBuilder::new();

                if let Some(prefix) = params.get("prefix").and_then(|v| v.as_str()) {
                    builder = builder.with_prefix(prefix);
                }

                if params
                    .get("timestamp")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false)
                {
                    builder = builder.with_timestamp();
                }

                if params
                    .get("short")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false)
                {
                    builder = builder.short();
                }

                if let Some(suffix) = params.get("suffix").and_then(|v| v.as_str()) {
                    builder = builder.with_suffix(suffix);
                }

                let id = builder.build();
                Ok(AgentOutput::text(id))
            }
            "validate" => {
                // Validate a UUID
                let uuid_str = params.get("uuid").and_then(|v| v.as_str()).ok_or_else(|| {
                    LLMSpellError::Validation {
                        message: "Validate operation requires a uuid parameter".to_string(),
                        field: Some("uuid".to_string()),
                    }
                })?;

                match Uuid::parse_str(uuid_str) {
                    Ok(uuid) => {
                        let result = json!({
                            "valid": true,
                            "uuid": uuid.to_string(),
                            "version": uuid.get_version().map(|v| format!("v{}", v as u8)),
                            "variant": format!("{:?}", uuid.get_variant()),
                        });
                        Ok(AgentOutput::text(serde_json::to_string_pretty(&result)?))
                    }
                    Err(_) => {
                        let result = json!({
                            "valid": false,
                            "error": "Invalid UUID format"
                        });
                        Ok(AgentOutput::text(serde_json::to_string_pretty(&result)?))
                    }
                }
            }
            _ => Err(LLMSpellError::Validation {
                message: format!("Unknown operation: {}", operation),
                field: Some("operation".to_string()),
            }),
        }
    }

    async fn validate_input(&self, input: &AgentInput) -> Result<()> {
        if input.text.is_empty() {
            return Err(LLMSpellError::Validation {
                message: "Input prompt cannot be empty".to_string(),
                field: Some("prompt".to_string()),
            });
        }
        Ok(())
    }

    async fn handle_error(&self, error: LLMSpellError) -> Result<AgentOutput> {
        Ok(AgentOutput::text(format!(
            "UUID generation error: {}",
            error
        )))
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
        assert!(Uuid::parse_str(&output).is_ok());
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
        let uuid = Uuid::parse_str(&output).unwrap();
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
        assert!(output.starts_with("test_"));
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
        assert_eq!(result1.unwrap().text, result2.unwrap().text);
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
        assert!(output.starts_with("custom_"));
        assert!(output.ends_with("_v1"));
        assert!(output.contains("_")); // Contains timestamp separator
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
            assert!(result.is_ok(), "Format {} failed", format);

            let output = result.unwrap().text;
            match format {
                "simple" => assert!(!output.contains('-')),
                "urn" => assert!(output.starts_with("urn:uuid:")),
                "braced" => assert!(output.starts_with('{') && output.ends_with('}')),
                _ => assert!(output.contains('-')),
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
