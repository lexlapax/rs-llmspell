// ABOUTME: Diff calculation tool for comparing texts, files, and JSON structures
// ABOUTME: Provides line-by-line text diff, JSON structural diff, and multiple output formats

//! Diff calculation tool
//!
//! This tool provides diff functionality for:
//! - Text line-by-line comparison
//! - JSON structural differences
//! - File comparison with encoding detection
//! - Multiple output formats (unified, context, inline)

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
    error_builders::llmspell::{tool_error, validation_error},
    params::{extract_optional_string, extract_parameters, extract_string_with_default},
    response::ResponseBuilder,
};
use serde_json::{json, Value};
use similar::{ChangeTag, DiffTag, TextDiff};
use std::fs;

/// Diff output format
#[derive(Debug, Clone)]
enum DiffFormat {
    /// Unified diff format (like git diff)
    Unified,
    /// Context diff format
    Context,
    /// Inline diff format
    Inline,
    /// Simple list of changes
    Simple,
}

impl DiffFormat {
    fn from_str(s: &str) -> Result<Self> {
        match s.to_lowercase().as_str() {
            "unified" => Ok(Self::Unified),
            "context" => Ok(Self::Context),
            "inline" => Ok(Self::Inline),
            "simple" => Ok(Self::Simple),
            _ => Err(validation_error(
                format!("Invalid diff format: {s}"),
                Some("format".to_string()),
            )),
        }
    }
}

/// Diff calculator tool
#[derive(Debug, Clone)]
pub struct DiffCalculatorTool {
    /// Tool metadata
    metadata: ComponentMetadata,
}

impl Default for DiffCalculatorTool {
    fn default() -> Self {
        Self {
            metadata: ComponentMetadata::new(
                "diff-calculator".to_string(),
                "Calculate differences between texts, files, or JSON structures".to_string(),
            ),
        }
    }
}

impl DiffCalculatorTool {
    /// Create a new diff calculator tool
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Calculate text diff
    #[allow(clippy::unused_self)]
    #[allow(clippy::too_many_lines)]
    fn calculate_text_diff(&self, old: &str, new: &str, format: &DiffFormat) -> String {
        use std::fmt::Write;

        let diff = TextDiff::from_lines(old, new);

        match format {
            DiffFormat::Unified => diff
                .unified_diff()
                .context_radius(3)
                .header("old", "new")
                .to_string(),
            DiffFormat::Context => {
                // Context format - show more surrounding lines
                let mut output = String::new();
                output.push_str("*** old\n--- new\n");
                output.push_str("***************\n");

                for group in diff.grouped_ops(3) {
                    for op in group {
                        let (tag, old_range, new_range) =
                            (op.tag(), op.old_range(), op.new_range());

                        match tag {
                            DiffTag::Equal => {
                                // Get the old slices and extract lines for the range
                                let old_slices = diff.old_slices();
                                for idx in old_range {
                                    if let Some(line) = old_slices.get(idx) {
                                        let _ = writeln!(output, "  {line}");
                                    }
                                }
                            }
                            DiffTag::Delete => {
                                // Get the old slices and extract lines for the range
                                let old_slices = diff.old_slices();
                                for idx in old_range {
                                    if let Some(line) = old_slices.get(idx) {
                                        let _ = writeln!(output, "- {line}");
                                    }
                                }
                            }
                            DiffTag::Insert => {
                                // Get the new slices and extract lines for the range
                                let new_slices = diff.new_slices();
                                for idx in new_range {
                                    if let Some(line) = new_slices.get(idx) {
                                        let _ = writeln!(output, "+ {line}");
                                    }
                                }
                            }
                            DiffTag::Replace => {
                                // For replace, show both delete and insert
                                let old_slices = diff.old_slices();
                                for idx in old_range {
                                    if let Some(line) = old_slices.get(idx) {
                                        let _ = writeln!(output, "- {line}");
                                    }
                                }
                                let new_slices = diff.new_slices();
                                for idx in new_range {
                                    if let Some(line) = new_slices.get(idx) {
                                        let _ = writeln!(output, "+ {line}");
                                    }
                                }
                            }
                        }
                    }
                    output.push_str("***************\n");
                }
                output
            }
            DiffFormat::Inline => {
                // For inline diff, we'll use iter_all_changes to show inline changes
                let mut output = String::new();
                for change in diff.iter_all_changes() {
                    let sign = match change.tag() {
                        ChangeTag::Delete => "-",
                        ChangeTag::Insert => "+",
                        ChangeTag::Equal => " ",
                    };
                    let _ = write!(output, "{sign}{change}");
                }
                output
            }
            DiffFormat::Simple => {
                let mut output = String::new();
                let mut changes = 0;

                for op in diff.ops() {
                    match op.tag() {
                        DiffTag::Delete => {
                            changes += 1;
                            let _ = writeln!(
                                output,
                                "Removed at line {}: {} line(s)",
                                op.old_range().start + 1,
                                op.old_range().len()
                            );
                        }
                        DiffTag::Insert => {
                            changes += 1;
                            let _ = writeln!(
                                output,
                                "Added at line {}: {} line(s)",
                                op.new_range().start + 1,
                                op.new_range().len()
                            );
                        }
                        DiffTag::Equal => {}
                        DiffTag::Replace => {
                            changes += 1;
                            let _ = writeln!(
                                output,
                                "Replaced at lines {}-{} with lines {}-{}",
                                op.old_range().start + 1,
                                op.old_range().end,
                                op.new_range().start + 1,
                                op.new_range().end
                            );
                        }
                    }
                }

                output.insert_str(0, &format!("Total changes: {changes}\n\n"));
                output
            }
        }
    }

    /// Calculate JSON diff
    #[allow(clippy::unused_self)]
    fn calculate_json_diff(&self, old_json: &Value, new_json: &Value) -> Result<Value> {
        let mut diff = json!({
            "added": {},
            "removed": {},
            "modified": {},
            "unchanged": []
        });

        compare_json_values(old_json, new_json, "", &mut diff)?;

        Ok(diff)
    }
}

/// Recursively compare JSON values
fn compare_json_values(old: &Value, new: &Value, path: &str, diff: &mut Value) -> Result<()> {
    match (old, new) {
        (Value::Object(old_map), Value::Object(new_map)) => {
            // Check for removed keys
            for (key, value) in old_map {
                let current_path = if path.is_empty() {
                    key.clone()
                } else {
                    format!("{path}.{key}")
                };

                if !new_map.contains_key(key) {
                    diff["removed"][&current_path] = value.clone();
                }
            }

            // Check for added and modified keys
            for (key, new_value) in new_map {
                let current_path = if path.is_empty() {
                    key.clone()
                } else {
                    format!("{path}.{key}")
                };

                if let Some(old_value) = old_map.get(key) {
                    if old_value == new_value {
                        diff["unchanged"]
                            .as_array_mut()
                            .ok_or_else(|| LLMSpellError::Internal {
                                message: "Failed to access unchanged array".to_string(),
                                source: None,
                            })?
                            .push(json!(current_path));
                    } else if old_value.is_object() && new_value.is_object() {
                        compare_json_values(old_value, new_value, &current_path, diff)?;
                    } else {
                        diff["modified"][&current_path] = json!({
                            "old": old_value,
                            "new": new_value
                        });
                    }
                } else {
                    diff["added"][&current_path] = new_value.clone();
                }
            }
        }
        (Value::Array(old_arr), Value::Array(new_arr)) => {
            let current_path = if path.is_empty() {
                "[]".to_string()
            } else {
                format!("{path}[]")
            };

            if old_arr.len() != new_arr.len() || old_arr != new_arr {
                diff["modified"][&current_path] = json!({
                    "old_length": old_arr.len(),
                    "new_length": new_arr.len(),
                    "old": old_arr,
                    "new": new_arr
                });
            } else {
                diff["unchanged"]
                    .as_array_mut()
                    .ok_or_else(|| LLMSpellError::Internal {
                        message: "Failed to access unchanged array".to_string(),
                        source: None,
                    })?
                    .push(json!(current_path));
            }
        }
        _ => {
            if old == new {
                diff["unchanged"]
                    .as_array_mut()
                    .ok_or_else(|| LLMSpellError::Internal {
                        message: "Failed to access unchanged array".to_string(),
                        source: None,
                    })?
                    .push(json!(path));
            } else {
                diff["modified"][path] = json!({
                    "old": old,
                    "new": new
                });
            }
        }
    }

    Ok(())
}

impl DiffCalculatorTool {
    /// Process diff operation
    #[allow(clippy::unused_async)]
    #[allow(clippy::too_many_lines)]
    async fn process_operation(&self, params: &Value) -> Result<Value> {
        let diff_type = extract_string_with_default(params, "type", "text");

        match diff_type {
            "text" => {
                // Get text inputs
                let (old_text, new_text) = if let (Some(old_file), Some(new_file)) = (
                    extract_optional_string(params, "old_file"),
                    extract_optional_string(params, "new_file"),
                ) {
                    // Read from files
                    let old = fs::read_to_string(old_file).map_err(|e| {
                        tool_error(
                            format!("Failed to read old file: {e}"),
                            Some(self.metadata.name.clone()),
                        )
                    })?;
                    let new = fs::read_to_string(new_file).map_err(|e| {
                        tool_error(
                            format!("Failed to read new file: {e}"),
                            Some(self.metadata.name.clone()),
                        )
                    })?;
                    (old, new)
                } else if let (Some(old), Some(new)) = (
                    extract_optional_string(params, "old_text"),
                    extract_optional_string(params, "new_text"),
                ) {
                    (old.to_string(), new.to_string())
                } else {
                    return Err(validation_error(
                        "Either (old_text, new_text) or (old_file, new_file) must be provided",
                        Some("input".to_string()),
                    ));
                };

                // Get format
                let format_str = extract_string_with_default(params, "format", "unified");
                let format = DiffFormat::from_str(format_str)?;

                // Calculate diff
                let diff_output = self.calculate_text_diff(&old_text, &new_text, &format);

                let response = ResponseBuilder::success("text_diff")
                    .with_message("Text diff calculated successfully")
                    .with_result(json!({
                        "type": "text",
                        "format": format_str,
                        "diff": diff_output,
                        "stats": {
                            "old_lines": old_text.lines().count(),
                            "new_lines": new_text.lines().count()
                        }
                    }))
                    .build();
                Ok(response)
            }
            "json" => {
                // Get JSON inputs
                let (old_json, new_json) = if let (Some(old_file), Some(new_file)) = (
                    extract_optional_string(params, "old_file"),
                    extract_optional_string(params, "new_file"),
                ) {
                    // Read from files
                    let old_content = fs::read_to_string(old_file).map_err(|e| {
                        tool_error(
                            format!("Failed to read old file: {e}"),
                            Some(self.metadata.name.clone()),
                        )
                    })?;
                    let new_content = fs::read_to_string(new_file).map_err(|e| {
                        tool_error(
                            format!("Failed to read new file: {e}"),
                            Some(self.metadata.name.clone()),
                        )
                    })?;

                    let old_json: Value = serde_json::from_str(&old_content).map_err(|e| {
                        tool_error(
                            format!("Failed to parse old JSON: {e}"),
                            Some(self.metadata.name.clone()),
                        )
                    })?;
                    let new_json: Value = serde_json::from_str(&new_content).map_err(|e| {
                        tool_error(
                            format!("Failed to parse new JSON: {e}"),
                            Some(self.metadata.name.clone()),
                        )
                    })?;
                    (old_json, new_json)
                } else if let (Some(old), Some(new)) =
                    (params.get("old_json"), params.get("new_json"))
                {
                    (old.clone(), new.clone())
                } else {
                    return Err(validation_error(
                        "Either (old_json, new_json) or (old_file, new_file) must be provided",
                        Some("input".to_string()),
                    ));
                };

                // Calculate JSON diff
                let diff = self.calculate_json_diff(&old_json, &new_json)?;

                let response = ResponseBuilder::success("json_diff")
                    .with_message("JSON diff calculated successfully")
                    .with_result(json!({
                        "type": "json",
                        "diff": diff,
                        "summary": {
                            "added": diff["added"].as_object().map_or(0, serde_json::Map::len),
                            "removed": diff["removed"].as_object().map_or(0, serde_json::Map::len),
                            "modified": diff["modified"].as_object().map_or(0, serde_json::Map::len),
                            "unchanged": diff["unchanged"].as_array().map_or(0, std::vec::Vec::len)
                        }
                    }))
                    .build();
                Ok(response)
            }
            _ => Err(validation_error(
                format!("Invalid diff type: {diff_type}"),
                Some("type".to_string()),
            )),
        }
    }
}

#[async_trait]
impl BaseAgent for DiffCalculatorTool {
    fn metadata(&self) -> &ComponentMetadata {
        &self.metadata
    }

    async fn execute_impl(
        &self,
        input: AgentInput,
        _context: ExecutionContext,
    ) -> Result<AgentOutput> {
        // Get parameters using shared utility
        let params = extract_parameters(&input)?;

        // Process the operation
        let result = self.process_operation(params).await?;

        // Return the result as JSON formatted text
        Ok(AgentOutput::text(
            serde_json::to_string_pretty(&result).unwrap(),
        ))
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
        Ok(AgentOutput::text(format!(
            "Diff calculation error: {error}"
        )))
    }
}

#[async_trait]
impl Tool for DiffCalculatorTool {
    fn category(&self) -> ToolCategory {
        ToolCategory::Utility
    }

    fn security_level(&self) -> SecurityLevel {
        SecurityLevel::Safe
    }

    fn schema(&self) -> ToolSchema {
        ToolSchema::new(
            "diff_calculator".to_string(),
            "Calculate differences between texts, files, or JSON structures".to_string(),
        )
        .with_parameter(ParameterDef {
            name: "type".to_string(),
            param_type: ParameterType::String,
            description: "Type of diff: 'text' or 'json' (default: 'text')".to_string(),
            required: false,
            default: Some(json!("text")),
        })
        .with_parameter(ParameterDef {
            name: "old_text".to_string(),
            param_type: ParameterType::String,
            description: "Old text content (for text diff)".to_string(),
            required: false,
            default: None,
        })
        .with_parameter(ParameterDef {
            name: "new_text".to_string(),
            param_type: ParameterType::String,
            description: "New text content (for text diff)".to_string(),
            required: false,
            default: None,
        })
        .with_parameter(ParameterDef {
            name: "old_json".to_string(),
            param_type: ParameterType::Object,
            description: "Old JSON object (for JSON diff)".to_string(),
            required: false,
            default: None,
        })
        .with_parameter(ParameterDef {
            name: "new_json".to_string(),
            param_type: ParameterType::Object,
            description: "New JSON object (for JSON diff)".to_string(),
            required: false,
            default: None,
        })
        .with_parameter(ParameterDef {
            name: "old_file".to_string(),
            param_type: ParameterType::String,
            description: "Path to old file".to_string(),
            required: false,
            default: None,
        })
        .with_parameter(ParameterDef {
            name: "new_file".to_string(),
            param_type: ParameterType::String,
            description: "Path to new file".to_string(),
            required: false,
            default: None,
        })
        .with_parameter(ParameterDef {
            name: "format".to_string(),
            param_type: ParameterType::String,
            description: "Output format for text diff: 'unified', 'context', 'inline', 'simple' (default: 'unified')".to_string(),
            required: false,
            default: Some(json!("unified")),
        })
        .with_returns(ParameterType::Object)
    }

    fn security_requirements(&self) -> SecurityRequirements {
        SecurityRequirements::safe()
    }

    fn resource_limits(&self) -> ResourceLimits {
        ResourceLimits::strict()
            .with_memory_limit(100 * 1024 * 1024) // 100MB
            .with_cpu_limit(10000) // 10 seconds
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    #[tokio::test]
    async fn test_text_diff_unified() {
        let tool = DiffCalculatorTool::new();
        let old_text = "line 1\nline 2\nline 3";
        let new_text = "line 1\nline 2 modified\nline 3\nline 4";

        let input = AgentInput::text("diff texts").with_parameter(
            "parameters",
            json!({
                "type": "text",
                "old_text": old_text,
                "new_text": new_text,
                "format": "unified"
            }),
        );

        let result = tool
            .execute(input, ExecutionContext::default())
            .await
            .unwrap();
        let output: Value = serde_json::from_str(&result.text).unwrap();

        assert!(output["success"].as_bool().unwrap_or(false));
        assert_eq!(output["result"]["type"], "text");
        assert_eq!(output["result"]["format"], "unified");
        assert!(output["result"]["diff"].as_str().unwrap().contains("@@"));
    }
    #[tokio::test]
    async fn test_text_diff_simple() {
        let tool = DiffCalculatorTool::new();
        let old_text = "line 1\nline 2\nline 3";
        let new_text = "line 1\nline 2 modified\nline 3";

        let input = AgentInput::text("diff texts").with_parameter(
            "parameters",
            json!({
                "type": "text",
                "old_text": old_text,
                "new_text": new_text,
                "format": "simple"
            }),
        );

        let result = tool
            .execute(input, ExecutionContext::default())
            .await
            .unwrap();
        let output: Value = serde_json::from_str(&result.text).unwrap();

        assert!(output["success"].as_bool().unwrap_or(false));
        assert_eq!(output["result"]["type"], "text");
        assert_eq!(output["result"]["format"], "simple");
        assert!(output["result"]["diff"]
            .as_str()
            .unwrap()
            .contains("Total changes:"));
    }
    #[tokio::test]
    async fn test_json_diff() {
        let tool = DiffCalculatorTool::new();
        let old_json = json!({
            "name": "John",
            "age": 30,
            "city": "New York",
            "hobbies": ["reading", "gaming"]
        });
        let new_json = json!({
            "name": "John",
            "age": 31,
            "city": "San Francisco",
            "hobbies": ["reading", "gaming", "hiking"],
            "job": "Engineer"
        });

        let input = AgentInput::text("diff json").with_parameter(
            "parameters",
            json!({
                "type": "json",
                "old_json": old_json,
                "new_json": new_json
            }),
        );

        let result = tool
            .execute(input, ExecutionContext::default())
            .await
            .unwrap();
        let output: Value = serde_json::from_str(&result.text).unwrap();

        assert!(output["success"].as_bool().unwrap_or(false));
        assert_eq!(output["result"]["type"], "json");
        assert!(output["result"]["diff"]["added"]["job"].is_string());
        assert!(output["result"]["diff"]["modified"]["age"].is_object());
        assert!(output["result"]["diff"]["modified"]["city"].is_object());
        assert_eq!(output["result"]["summary"]["added"], 1);
        assert_eq!(output["result"]["summary"]["modified"], 3); // age, city, hobbies[]
    }
    #[tokio::test]
    async fn test_missing_input() {
        let tool = DiffCalculatorTool::new();

        let input = AgentInput::text("diff without input").with_parameter(
            "parameters",
            json!({
                "type": "text",
                "format": "unified"
            }),
        );

        let result = tool.execute(input, ExecutionContext::default()).await;
        assert!(result.is_err());
    }
    #[tokio::test]
    async fn test_invalid_format() {
        let tool = DiffCalculatorTool::new();

        let input = AgentInput::text("diff with invalid format").with_parameter(
            "parameters",
            json!({
                "type": "text",
                "old_text": "test",
                "new_text": "test2",
                "format": "invalid"
            }),
        );

        let result = tool.execute(input, ExecutionContext::default()).await;
        assert!(result.is_err());
    }
    #[tokio::test]
    async fn test_tool_metadata() {
        let tool = DiffCalculatorTool::new();

        assert_eq!(tool.metadata().name, "diff-calculator");
        assert!(tool.metadata().description.contains("differences"));
        assert_eq!(tool.category(), ToolCategory::Utility);
        assert_eq!(tool.security_level(), SecurityLevel::Safe);
    }
}
