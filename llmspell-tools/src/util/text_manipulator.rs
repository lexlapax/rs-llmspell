// ABOUTME: Text manipulation tool using llmspell-utils string functions
// ABOUTME: Provides string operations like uppercase, lowercase, reverse, trim, replace

//! Text manipulation and transformation tool
//!
//! This tool provides various string manipulation operations including:
//! - Case conversion (uppercase, lowercase)
//! - String reversal
//! - Trimming and normalization
//! - Pattern replacement
//! - Substring extraction
//! - Splitting and joining

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
use llmspell_utils::string_utils;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TextOperation {
    Uppercase,
    Lowercase,
    Reverse,
    Trim,
    Replace,
    Substring,
    Split,
    Join,
    #[serde(rename = "snake_case")]
    SnakeCase,
    #[serde(rename = "camel_case")]
    CamelCase,
    #[serde(rename = "pascal_case")]
    PascalCase,
    Sanitize,
    Truncate,
    Indent,
    Dedent,
    #[serde(rename = "normalize_whitespace")]
    NormalizeWhitespace,
    #[serde(rename = "word_wrap")]
    WordWrap,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextManipulatorConfig {
    /// Default truncation length
    pub default_truncate_length: usize,
    /// Default indentation spaces
    pub default_indent_spaces: usize,
    /// Default word wrap width
    pub default_wrap_width: usize,
}

impl Default for TextManipulatorConfig {
    fn default() -> Self {
        Self {
            default_truncate_length: 100,
            default_indent_spaces: 4,
            default_wrap_width: 80,
        }
    }
}

/// Text manipulation tool for string operations
pub struct TextManipulatorTool {
    metadata: ComponentMetadata,
    config: TextManipulatorConfig,
}

impl TextManipulatorTool {
    /// Create a new text manipulator tool
    pub fn new(config: TextManipulatorConfig) -> Self {
        Self {
            metadata: ComponentMetadata::new(
                "text-manipulator".to_string(),
                "Text manipulation and transformation tool".to_string(),
            ),
            config,
        }
    }

    fn perform_operation(
        &self,
        text: &str,
        operation: TextOperation,
        options: Option<Value>,
    ) -> Result<String> {
        match operation {
            TextOperation::Uppercase => Ok(string_utils::to_uppercase(text)),
            TextOperation::Lowercase => Ok(string_utils::to_lowercase(text)),
            TextOperation::Reverse => Ok(string_utils::reverse(text)),
            TextOperation::Trim => Ok(string_utils::trim(text)),
            TextOperation::Replace => {
                let opts = options.ok_or_else(|| LLMSpellError::Validation {
                    message: "Replace operation requires 'from' and 'to' options".to_string(),
                    field: Some("options".to_string()),
                })?;
                let from = opts.get("from").and_then(|v| v.as_str()).ok_or_else(|| {
                    LLMSpellError::Validation {
                        message: "Missing 'from' parameter".to_string(),
                        field: Some("from".to_string()),
                    }
                })?;
                let to = opts.get("to").and_then(|v| v.as_str()).ok_or_else(|| {
                    LLMSpellError::Validation {
                        message: "Missing 'to' parameter".to_string(),
                        field: Some("to".to_string()),
                    }
                })?;
                Ok(string_utils::replace_all(text, from, to))
            }
            TextOperation::Substring => {
                let opts = options.ok_or_else(|| LLMSpellError::Validation {
                    message: "Substring operation requires 'start' and 'end' options".to_string(),
                    field: Some("options".to_string()),
                })?;
                let start = opts.get("start").and_then(|v| v.as_u64()).ok_or_else(|| {
                    LLMSpellError::Validation {
                        message: "Missing or invalid 'start' parameter".to_string(),
                        field: Some("start".to_string()),
                    }
                })? as usize;
                let end = opts.get("end").and_then(|v| v.as_u64()).ok_or_else(|| {
                    LLMSpellError::Validation {
                        message: "Missing or invalid 'end' parameter".to_string(),
                        field: Some("end".to_string()),
                    }
                })? as usize;
                Ok(string_utils::substring(text, start, end))
            }
            TextOperation::Split => {
                let delimiter = options
                    .as_ref()
                    .and_then(|v| v.get("delimiter"))
                    .and_then(|v| v.as_str())
                    .unwrap_or(" ");
                let parts = string_utils::split_by(text, delimiter);
                Ok(
                    serde_json::to_string(&parts).map_err(|e| LLMSpellError::Component {
                        message: format!("Failed to serialize split result: {}", e),
                        source: Some(Box::new(e)),
                    })?,
                )
            }
            TextOperation::Join => {
                // For join, the text is expected to be a JSON array of strings
                let parts: Vec<String> =
                    serde_json::from_str(text).map_err(|_| LLMSpellError::Validation {
                        message: "Join operation requires text to be a JSON array of strings"
                            .to_string(),
                        field: Some("text".to_string()),
                    })?;
                let delimiter = options
                    .as_ref()
                    .and_then(|v| v.get("delimiter"))
                    .and_then(|v| v.as_str())
                    .unwrap_or(" ");
                let parts_refs: Vec<&str> = parts.iter().map(|s| s.as_str()).collect();
                Ok(string_utils::join_with(&parts_refs, delimiter))
            }
            TextOperation::SnakeCase => Ok(string_utils::to_snake_case(text)),
            TextOperation::CamelCase => Ok(string_utils::to_camel_case(text)),
            TextOperation::PascalCase => Ok(string_utils::to_pascal_case(text)),
            TextOperation::Sanitize => Ok(string_utils::sanitize(text)),
            TextOperation::Truncate => {
                let max_len = options
                    .as_ref()
                    .and_then(|v| v.get("max_length"))
                    .and_then(|v| v.as_u64())
                    .map(|v| v as usize)
                    .unwrap_or(self.config.default_truncate_length);
                Ok(string_utils::truncate(text, max_len))
            }
            TextOperation::Indent => {
                let spaces = options
                    .as_ref()
                    .and_then(|v| v.get("spaces"))
                    .and_then(|v| v.as_u64())
                    .map(|v| v as usize)
                    .unwrap_or(self.config.default_indent_spaces);
                Ok(string_utils::indent(text, spaces))
            }
            TextOperation::Dedent => Ok(string_utils::dedent(text)),
            TextOperation::NormalizeWhitespace => Ok(string_utils::normalize_whitespace(text)),
            TextOperation::WordWrap => {
                let width = options
                    .as_ref()
                    .and_then(|v| v.get("width"))
                    .and_then(|v| v.as_u64())
                    .map(|v| v as usize)
                    .unwrap_or(self.config.default_wrap_width);
                let lines = string_utils::word_wrap(text, width);
                Ok(lines.join("\n"))
            }
        }
    }
}

impl Default for TextManipulatorTool {
    fn default() -> Self {
        Self::new(TextManipulatorConfig::default())
    }
}

#[async_trait]
impl BaseAgent for TextManipulatorTool {
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

        // Extract parameters
        let text = params.get("text").and_then(|v| v.as_str()).ok_or_else(|| {
            LLMSpellError::Validation {
                message: "Missing 'text' parameter".to_string(),
                field: Some("text".to_string()),
            }
        })?;

        let operation_str = params
            .get("operation")
            .and_then(|v| v.as_str())
            .ok_or_else(|| LLMSpellError::Validation {
                message: "Missing 'operation' parameter".to_string(),
                field: Some("operation".to_string()),
            })?;

        let operation: TextOperation =
            serde_json::from_value(json!(operation_str)).map_err(|_| {
                LLMSpellError::Validation {
                    message: format!("Invalid operation: {}", operation_str),
                    field: Some("operation".to_string()),
                }
            })?;

        let options = params.get("options").cloned();

        // Perform the operation
        match self.perform_operation(text, operation, options) {
            Ok(result) => Ok(AgentOutput::text(result)),
            Err(e) => Err(e),
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
            "Text manipulation error: {}",
            error
        )))
    }
}

#[async_trait]
impl Tool for TextManipulatorTool {
    fn category(&self) -> ToolCategory {
        ToolCategory::Utility
    }

    fn security_level(&self) -> SecurityLevel {
        SecurityLevel::Safe
    }

    fn schema(&self) -> ToolSchema {
        ToolSchema::new(
            "text_manipulator".to_string(),
            "Manipulate and transform text with various string operations".to_string(),
        )
        .with_parameter(ParameterDef {
            name: "text".to_string(),
            param_type: ParameterType::String,
            description: "The text to manipulate".to_string(),
            required: true,
            default: None,
        })
        .with_parameter(ParameterDef {
            name: "operation".to_string(),
            param_type: ParameterType::String,
            description: "The operation to perform on the text".to_string(),
            required: true,
            default: None,
        })
        .with_parameter(ParameterDef {
            name: "options".to_string(),
            param_type: ParameterType::Object,
            description: "Additional options for specific operations".to_string(),
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
            .with_memory_limit(10 * 1024 * 1024) // 10MB
            .with_cpu_limit(1000) // 1 second
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_missing_required_option() {
        let tool = TextManipulatorTool::default();
        let input = AgentInput::text("replace text".to_string()).with_parameter(
            "parameters".to_string(),
            json!({
                "text": "hello",
                "operation": "replace"
                // Missing required 'from' and 'to' options
            }),
        );
        let context = ExecutionContext::with_conversation("test".to_string());

        let result = tool.execute(input, context).await;
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(
            err.to_string().contains("Replace operation requires"),
            "Error was: {}",
            err
        );
    }

    #[tokio::test]
    async fn test_tool_metadata() {
        let tool = TextManipulatorTool::default();
        assert_eq!(tool.category(), ToolCategory::Utility);
        assert_eq!(tool.security_level(), SecurityLevel::Safe);
        assert_eq!(tool.metadata().name, "text-manipulator");

        let schema = tool.schema();
        assert_eq!(schema.name, "text_manipulator");
        assert_eq!(schema.parameters.len(), 3);
        assert_eq!(schema.required_parameters(), vec!["text", "operation"]);
    }

    #[tokio::test]
    async fn test_all_operations() {
        let tool = TextManipulatorTool::default();
        let context = ExecutionContext::with_conversation("test".to_string());

        // Test each operation
        let test_cases = vec![
            ("uppercase", "hello", None::<Value>, "HELLO"),
            ("lowercase", "HELLO", None::<Value>, "hello"),
            ("reverse", "hello", None::<Value>, "olleh"),
            ("trim", "  hello  ", None::<Value>, "hello"),
            ("snake_case", "HelloWorld", None::<Value>, "hello_world"),
            ("camel_case", "hello_world", None::<Value>, "helloWorld"),
            ("pascal_case", "hello_world", None::<Value>, "HelloWorld"),
            (
                "sanitize",
                "  Hello\x00World  ",
                None::<Value>,
                "HelloWorld",
            ),
            (
                "dedent",
                "    line1\n    line2",
                None::<Value>,
                "line1\nline2",
            ),
            (
                "normalize_whitespace",
                "hello    world",
                None::<Value>,
                "hello world",
            ),
        ];

        for (operation, input_text, options, expected) in test_cases {
            let mut params = json!({
                "text": input_text,
                "operation": operation,
            });

            if let Some(opts) = options {
                params
                    .as_object_mut()
                    .unwrap()
                    .insert("options".to_string(), opts);
            }

            let input = AgentInput::text(format!("test {}", operation))
                .with_parameter("parameters".to_string(), params);

            let result = tool.execute(input, context.clone()).await;
            match result {
                Ok(output) => {
                    assert_eq!(
                        output.text, expected,
                        "Operation {} produced unexpected result",
                        operation
                    );
                }
                Err(e) => {
                    panic!("Operation {} failed with error: {}", operation, e);
                }
            }
        }
    }
}
