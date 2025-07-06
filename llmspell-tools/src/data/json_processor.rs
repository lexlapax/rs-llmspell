//! ABOUTME: JSON processing tool with jq-like syntax and schema validation
//! ABOUTME: Provides powerful JSON manipulation, transformation, and validation capabilities

use async_trait::async_trait;
use jsonschema::{Draft, JSONSchema};
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
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tokio::io::{AsyncBufReadExt, AsyncRead, BufReader};
use tracing::{debug, info};

/// JSON processing operation types
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum JsonOperation {
    /// Transform JSON using jq syntax (basic support)
    Transform,
    /// Validate JSON against a schema
    Validate,
    /// Format/pretty-print JSON
    Format,
    /// Minify JSON (remove whitespace)
    Minify,
    /// Extract specific fields
    Extract,
    /// Filter array elements
    Filter,
    /// Merge multiple JSON objects
    Merge,
}

impl std::fmt::Display for JsonOperation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            JsonOperation::Transform => write!(f, "transform"),
            JsonOperation::Validate => write!(f, "validate"),
            JsonOperation::Format => write!(f, "format"),
            JsonOperation::Minify => write!(f, "minify"),
            JsonOperation::Extract => write!(f, "extract"),
            JsonOperation::Filter => write!(f, "filter"),
            JsonOperation::Merge => write!(f, "merge"),
        }
    }
}

impl std::str::FromStr for JsonOperation {
    type Err = LLMSpellError;

    fn from_str(s: &str) -> Result<Self> {
        match s.to_lowercase().as_str() {
            "transform" => Ok(JsonOperation::Transform),
            "validate" => Ok(JsonOperation::Validate),
            "format" | "pretty" => Ok(JsonOperation::Format),
            "minify" | "compact" => Ok(JsonOperation::Minify),
            "extract" => Ok(JsonOperation::Extract),
            "filter" => Ok(JsonOperation::Filter),
            "merge" => Ok(JsonOperation::Merge),
            _ => Err(LLMSpellError::Validation {
                message: format!("Unknown JSON operation: {}", s),
                field: Some("operation".to_string()),
            }),
        }
    }
}

/// JSON processor configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonProcessorConfig {
    /// Maximum input size in bytes
    pub max_input_size: usize,
    /// Enable streaming for large files
    pub enable_streaming: bool,
    /// Pretty print indentation
    pub indent_size: usize,
    /// Schema validation draft
    pub schema_draft: String,
    /// Maximum nesting depth
    pub max_depth: usize,
}

impl Default for JsonProcessorConfig {
    fn default() -> Self {
        Self {
            max_input_size: 100 * 1024 * 1024, // 100MB
            enable_streaming: true,
            indent_size: 2,
            schema_draft: "draft-07".to_string(),
            max_depth: 100,
        }
    }
}

/// JSON processing result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessingResult {
    /// The processed JSON value
    pub value: Option<Value>,
    /// Processing statistics
    pub stats: ProcessingStats,
    /// Validation results if applicable
    pub validation: Option<ValidationResult>,
    /// Any warnings generated
    pub warnings: Vec<String>,
}

/// Processing statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessingStats {
    pub input_size: usize,
    pub output_size: usize,
    pub processing_time_ms: u64,
    pub objects_processed: usize,
    pub arrays_processed: usize,
}

/// Validation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    pub is_valid: bool,
    pub errors: Vec<ValidationError>,
}

/// Validation error details
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationError {
    pub path: String,
    pub message: String,
    pub keyword: String,
}

/// JSON processor tool implementation
pub struct JsonProcessorTool {
    metadata: ComponentMetadata,
    config: JsonProcessorConfig,
}

impl JsonProcessorTool {
    /// Create a new JSON processor tool
    pub fn new(config: JsonProcessorConfig) -> Self {
        Self {
            metadata: ComponentMetadata::new(
                "json-processor-tool".to_string(),
                "Process JSON with basic jq-like syntax, validation, and transformations"
                    .to_string(),
            ),
            config,
        }
    }

    /// Parse parameters from input
    fn parse_parameters(&self, params: &Value) -> Result<(JsonOperation, Value, Option<String>)> {
        let operation_str = params
            .get("operation")
            .and_then(|v| v.as_str())
            .unwrap_or("transform");
        let operation: JsonOperation = operation_str.parse()?;

        let input = params
            .get("input")
            .ok_or_else(|| LLMSpellError::Validation {
                message: "Missing 'input' parameter".to_string(),
                field: Some("input".to_string()),
            })?
            .clone();

        let query = params
            .get("query")
            .and_then(|v| v.as_str())
            .map(String::from);

        Ok((operation, input, query))
    }

    /// Transform JSON using basic jq-like syntax
    /// Note: This is a simplified implementation supporting basic operations
    async fn transform_json(&self, input: &Value, query: &str) -> Result<Value> {
        debug!("Transforming JSON with query: {}", query);

        // Basic jq-like operations support
        match query.trim() {
            "." => Ok(input.clone()),
            "[]" | ".[]" => {
                if let Value::Array(arr) = input {
                    Ok(Value::Array(arr.clone()))
                } else {
                    Err(LLMSpellError::Validation {
                        message: "Input must be an array for [] operation".to_string(),
                        field: Some("input".to_string()),
                    })
                }
            }
            query if query.starts_with('.') && !query.contains('[') => {
                // Simple field access like .field or .field.subfield
                let fields: Vec<&str> = query[1..].split('.').collect();
                let mut current = input;

                for field in fields {
                    if field.is_empty() {
                        continue;
                    }
                    current = current
                        .get(field)
                        .ok_or_else(|| LLMSpellError::Validation {
                            message: format!("Field '{}' not found", field),
                            field: Some("query".to_string()),
                        })?;
                }

                Ok(current.clone())
            }
            _ => {
                // For complex queries, return an error with helpful message
                Err(LLMSpellError::Validation {
                    message: format!(
                        "Complex jq syntax '{}' not yet supported. Supported operations: '.', '.field', '.field.subfield', '.[]'", 
                        query
                    ),
                    field: Some("query".to_string()),
                })
            }
        }
    }

    /// Validate JSON against a schema
    async fn validate_json(&self, input: &Value, schema: &Value) -> Result<ValidationResult> {
        debug!("Validating JSON against schema");

        let compiled = JSONSchema::options()
            .with_draft(Draft::Draft7)
            .compile(schema)
            .map_err(|e| LLMSpellError::Validation {
                message: format!("Invalid JSON schema: {}", e),
                field: Some("schema".to_string()),
            })?;

        let validation_result = compiled.validate(input);

        match validation_result {
            Ok(_) => Ok(ValidationResult {
                is_valid: true,
                errors: vec![],
            }),
            Err(errors) => {
                let error_list: Vec<ValidationError> = errors
                    .map(|error| ValidationError {
                        path: error.instance_path.to_string(),
                        message: error.to_string(),
                        keyword: format!("{:?}", error.kind)
                            .split('(')
                            .next()
                            .unwrap_or("Unknown")
                            .to_string(),
                    })
                    .collect();

                Ok(ValidationResult {
                    is_valid: false,
                    errors: error_list,
                })
            }
        }
    }

    /// Format JSON with pretty printing
    fn format_json(&self, input: &Value) -> Result<String> {
        serde_json::to_string_pretty(input).map_err(|e| LLMSpellError::Internal {
            message: format!("Failed to format JSON: {}", e),
            source: None,
        })
    }

    /// Minify JSON by removing whitespace
    fn minify_json(&self, input: &Value) -> Result<String> {
        serde_json::to_string(input).map_err(|e| LLMSpellError::Internal {
            message: format!("Failed to minify JSON: {}", e),
            source: None,
        })
    }

    /// Extract fields using basic path syntax
    async fn extract_fields(&self, input: &Value, query: &str) -> Result<Value> {
        // Use transform_json for extraction
        self.transform_json(input, query).await
    }

    /// Filter array elements using basic conditions
    async fn filter_array(&self, input: &Value, query: &str) -> Result<Value> {
        // Ensure input is an array
        let arr = input.as_array().ok_or_else(|| LLMSpellError::Validation {
            message: "Input must be an array for filter operation".to_string(),
            field: Some("input".to_string()),
        })?;

        // Basic filter operations
        if query.contains("==") {
            let parts: Vec<&str> = query.split("==").collect();
            if parts.len() != 2 {
                return Err(LLMSpellError::Validation {
                    message: "Invalid filter syntax".to_string(),
                    field: Some("query".to_string()),
                });
            }

            let field = parts[0].trim().trim_start_matches('.');
            let value_str = parts[1].trim().trim_matches('"');

            let filtered: Vec<Value> = arr
                .iter()
                .filter(|item| {
                    if let Some(field_value) = item.get(field) {
                        if let Some(str_value) = field_value.as_str() {
                            str_value == value_str
                        } else {
                            field_value.to_string().trim_matches('"') == value_str
                        }
                    } else {
                        false
                    }
                })
                .cloned()
                .collect();

            Ok(Value::Array(filtered))
        } else {
            Err(LLMSpellError::Validation {
                message: format!(
                    "Filter syntax '{}' not supported. Use '.field == \"value\"'",
                    query
                ),
                field: Some("query".to_string()),
            })
        }
    }

    /// Merge multiple JSON objects
    fn merge_json(&self, inputs: Vec<Value>) -> Result<Value> {
        if inputs.is_empty() {
            return Ok(Value::Null);
        }

        let mut result = serde_json::Map::new();

        for input in inputs {
            if let Value::Object(obj) = input {
                for (k, v) in obj {
                    result.insert(k, v);
                }
            } else {
                return Err(LLMSpellError::Validation {
                    message: "All inputs must be objects for merge operation".to_string(),
                    field: Some("input".to_string()),
                });
            }
        }

        Ok(Value::Object(result))
    }

    /// Process streaming JSON input
    #[allow(dead_code)]
    async fn process_streaming<R: AsyncRead + Unpin>(
        &self,
        reader: R,
        operation: JsonOperation,
        query: Option<String>,
    ) -> Result<ProcessingResult> {
        let mut buffer = BufReader::new(reader);
        let mut line = String::new();
        let mut results = Vec::new();
        let mut stats = ProcessingStats {
            input_size: 0,
            output_size: 0,
            processing_time_ms: 0,
            objects_processed: 0,
            arrays_processed: 0,
        };

        let start = std::time::Instant::now();

        while buffer.read_line(&mut line).await? > 0 {
            stats.input_size += line.len();

            // Parse JSON line
            let value: Value =
                serde_json::from_str(&line).map_err(|e| LLMSpellError::Validation {
                    message: format!("Invalid JSON: {}", e),
                    field: Some("input".to_string()),
                })?;

            // Process based on operation
            let processed = match operation {
                JsonOperation::Transform => {
                    if let Some(ref q) = query {
                        self.transform_json(&value, q).await?
                    } else {
                        value
                    }
                }
                JsonOperation::Filter => {
                    if let Some(ref q) = query {
                        self.filter_array(&value, q).await?
                    } else {
                        value
                    }
                }
                _ => value,
            };

            // Update stats
            match &processed {
                Value::Object(_) => stats.objects_processed += 1,
                Value::Array(_) => stats.arrays_processed += 1,
                _ => {}
            }

            results.push(processed);
            line.clear();
        }

        stats.processing_time_ms = start.elapsed().as_millis() as u64;

        // Combine results
        let final_value = if results.len() == 1 {
            results.into_iter().next()
        } else {
            Some(Value::Array(results))
        };

        if let Some(ref val) = final_value {
            stats.output_size = serde_json::to_string(val)?.len();
        }

        Ok(ProcessingResult {
            value: final_value,
            stats,
            validation: None,
            warnings: vec![],
        })
    }
}

impl Default for JsonProcessorTool {
    fn default() -> Self {
        Self::new(JsonProcessorConfig::default())
    }
}

#[async_trait]
impl BaseAgent for JsonProcessorTool {
    fn metadata(&self) -> &ComponentMetadata {
        &self.metadata
    }

    async fn execute(&self, input: AgentInput, _context: ExecutionContext) -> Result<AgentOutput> {
        let start = std::time::Instant::now();

        let params =
            input
                .parameters
                .get("parameters")
                .ok_or_else(|| LLMSpellError::Validation {
                    message: "Missing parameters".to_string(),
                    field: Some("parameters".to_string()),
                })?;

        // Parse parameters
        let (operation, input_json, query) = self.parse_parameters(params)?;

        info!("Executing JSON {} operation", operation);

        // Process based on operation
        let result = match operation {
            JsonOperation::Transform => {
                let query = query.ok_or_else(|| LLMSpellError::Validation {
                    message: "Transform operation requires 'query' parameter".to_string(),
                    field: Some("query".to_string()),
                })?;
                let value = self.transform_json(&input_json, &query).await?;
                ProcessingResult {
                    value: Some(value),
                    stats: ProcessingStats {
                        input_size: serde_json::to_string(&input_json)?.len(),
                        output_size: 0, // Will be updated
                        processing_time_ms: start.elapsed().as_millis() as u64,
                        objects_processed: if input_json.is_object() { 1 } else { 0 },
                        arrays_processed: if input_json.is_array() { 1 } else { 0 },
                    },
                    validation: None,
                    warnings: vec![],
                }
            }
            JsonOperation::Validate => {
                let schema = params
                    .get("schema")
                    .ok_or_else(|| LLMSpellError::Validation {
                        message: "Validate operation requires 'schema' parameter".to_string(),
                        field: Some("schema".to_string()),
                    })?;
                let validation = self.validate_json(&input_json, schema).await?;
                ProcessingResult {
                    value: Some(input_json.clone()),
                    stats: ProcessingStats {
                        input_size: serde_json::to_string(&input_json)?.len(),
                        output_size: serde_json::to_string(&input_json)?.len(),
                        processing_time_ms: start.elapsed().as_millis() as u64,
                        objects_processed: if input_json.is_object() { 1 } else { 0 },
                        arrays_processed: if input_json.is_array() { 1 } else { 0 },
                    },
                    validation: Some(validation),
                    warnings: vec![],
                }
            }
            JsonOperation::Format => {
                let formatted = self.format_json(&input_json)?;
                let input_size = serde_json::to_string(&input_json)?.len();
                let is_object = input_json.is_object();
                let is_array = input_json.is_array();
                ProcessingResult {
                    value: Some(input_json),
                    stats: ProcessingStats {
                        input_size,
                        output_size: formatted.len(),
                        processing_time_ms: start.elapsed().as_millis() as u64,
                        objects_processed: if is_object { 1 } else { 0 },
                        arrays_processed: if is_array { 1 } else { 0 },
                    },
                    validation: None,
                    warnings: vec![],
                }
            }
            JsonOperation::Minify => {
                let minified = self.minify_json(&input_json)?;
                let input_size = serde_json::to_string(&input_json)?.len();
                let is_object = input_json.is_object();
                let is_array = input_json.is_array();
                ProcessingResult {
                    value: Some(input_json),
                    stats: ProcessingStats {
                        input_size,
                        output_size: minified.len(),
                        processing_time_ms: start.elapsed().as_millis() as u64,
                        objects_processed: if is_object { 1 } else { 0 },
                        arrays_processed: if is_array { 1 } else { 0 },
                    },
                    validation: None,
                    warnings: vec![],
                }
            }
            JsonOperation::Extract => {
                let query = query.ok_or_else(|| LLMSpellError::Validation {
                    message: "Extract operation requires 'query' parameter".to_string(),
                    field: Some("query".to_string()),
                })?;
                let value = self.extract_fields(&input_json, &query).await?;
                ProcessingResult {
                    value: Some(value),
                    stats: ProcessingStats {
                        input_size: serde_json::to_string(&input_json)?.len(),
                        output_size: 0, // Will be updated
                        processing_time_ms: start.elapsed().as_millis() as u64,
                        objects_processed: if input_json.is_object() { 1 } else { 0 },
                        arrays_processed: if input_json.is_array() { 1 } else { 0 },
                    },
                    validation: None,
                    warnings: vec![],
                }
            }
            JsonOperation::Filter => {
                let query = query.ok_or_else(|| LLMSpellError::Validation {
                    message: "Filter operation requires 'query' parameter".to_string(),
                    field: Some("query".to_string()),
                })?;
                let value = self.filter_array(&input_json, &query).await?;
                ProcessingResult {
                    value: Some(value),
                    stats: ProcessingStats {
                        input_size: serde_json::to_string(&input_json)?.len(),
                        output_size: 0, // Will be updated
                        processing_time_ms: start.elapsed().as_millis() as u64,
                        objects_processed: 0,
                        arrays_processed: 1,
                    },
                    validation: None,
                    warnings: vec![],
                }
            }
            JsonOperation::Merge => {
                let inputs = if let Value::Array(arr) = input_json {
                    arr
                } else {
                    vec![input_json]
                };
                let value = self.merge_json(inputs)?;
                ProcessingResult {
                    value: Some(value),
                    stats: ProcessingStats {
                        input_size: 0,  // Complex to calculate
                        output_size: 0, // Will be updated
                        processing_time_ms: start.elapsed().as_millis() as u64,
                        objects_processed: 1,
                        arrays_processed: 0,
                    },
                    validation: None,
                    warnings: vec![],
                }
            }
        };

        // Update output size
        let mut final_result = result;
        if let Some(ref val) = final_result.value {
            final_result.stats.output_size = serde_json::to_string(val)?.len();
        }

        // Format output
        let output_text = match operation {
            JsonOperation::Format => {
                self.format_json(final_result.value.as_ref().unwrap_or(&Value::Null))?
            }
            JsonOperation::Minify => {
                self.minify_json(final_result.value.as_ref().unwrap_or(&Value::Null))?
            }
            _ => serde_json::to_string_pretty(final_result.value.as_ref().unwrap_or(&Value::Null))?,
        };

        // Create metadata
        let mut metadata = llmspell_core::types::OutputMetadata::default();
        metadata
            .extra
            .insert("result".to_string(), serde_json::to_value(&final_result)?);

        Ok(AgentOutput::text(output_text).with_metadata(metadata))
    }

    async fn validate_input(&self, input: &AgentInput) -> Result<()> {
        if input.parameters.is_empty() {
            return Err(LLMSpellError::Validation {
                message: "No parameters provided".to_string(),
                field: Some("parameters".to_string()),
            });
        }

        // Check size limit
        if let Some(params) = input.parameters.get("parameters") {
            let size = serde_json::to_string(params)?.len();
            if size > self.config.max_input_size {
                return Err(LLMSpellError::Validation {
                    message: format!(
                        "Input size {} bytes exceeds maximum {} bytes",
                        size, self.config.max_input_size
                    ),
                    field: Some("input".to_string()),
                });
            }
        }

        Ok(())
    }

    async fn handle_error(&self, error: LLMSpellError) -> Result<AgentOutput> {
        Ok(AgentOutput::text(format!(
            "JSON processing error: {}",
            error
        )))
    }
}

#[async_trait]
impl Tool for JsonProcessorTool {
    fn category(&self) -> ToolCategory {
        ToolCategory::Data
    }

    fn security_level(&self) -> SecurityLevel {
        SecurityLevel::Safe
    }

    fn schema(&self) -> ToolSchema {
        ToolSchema::new(
            "json_processor".to_string(),
            "Process JSON with basic jq-like syntax, validation, and transformations".to_string(),
        )
        .with_parameter(ParameterDef {
            name: "operation".to_string(),
            param_type: ParameterType::String,
            description: "Operation: transform, validate, format, minify, extract, filter, merge"
                .to_string(),
            required: false,
            default: Some(serde_json::json!("transform")),
        })
        .with_parameter(ParameterDef {
            name: "input".to_string(),
            param_type: ParameterType::Object,
            description: "The JSON input to process".to_string(),
            required: true,
            default: None,
        })
        .with_parameter(ParameterDef {
            name: "query".to_string(),
            param_type: ParameterType::String,
            description: "Query for transform/extract/filter operations (basic syntax: '.field', '.field.subfield', '.[]')".to_string(),
            required: false,
            default: None,
        })
        .with_parameter(ParameterDef {
            name: "schema".to_string(),
            param_type: ParameterType::Object,
            description: "JSON schema for validation".to_string(),
            required: false,
            default: None,
        })
        .with_returns(ParameterType::Object)
    }

    fn security_requirements(&self) -> SecurityRequirements {
        SecurityRequirements::safe()
    }

    fn resource_limits(&self) -> ResourceLimits {
        ResourceLimits::default()
            .with_memory_limit(self.config.max_input_size as u64)
            .with_cpu_limit(30000) // 30 seconds for complex queries
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_operation_parsing() {
        assert_eq!(
            "transform".parse::<JsonOperation>().unwrap(),
            JsonOperation::Transform
        );
        assert_eq!(
            "validate".parse::<JsonOperation>().unwrap(),
            JsonOperation::Validate
        );
        assert_eq!(
            "format".parse::<JsonOperation>().unwrap(),
            JsonOperation::Format
        );
        assert_eq!(
            "pretty".parse::<JsonOperation>().unwrap(),
            JsonOperation::Format
        );
        assert_eq!(
            "minify".parse::<JsonOperation>().unwrap(),
            JsonOperation::Minify
        );
        assert!("invalid".parse::<JsonOperation>().is_err());
    }

    #[tokio::test]
    async fn test_json_processor_creation() {
        let config = JsonProcessorConfig::default();
        let tool = JsonProcessorTool::new(config);

        assert_eq!(tool.category(), ToolCategory::Data);
        assert_eq!(tool.security_level(), SecurityLevel::Safe);

        let schema = tool.schema();
        assert_eq!(schema.name, "json_processor");
        assert_eq!(schema.required_parameters(), vec!["input"]);
    }

    #[tokio::test]
    async fn test_format_operation() {
        let tool = JsonProcessorTool::default();

        let input_json = serde_json::json!({
            "name": "test",
            "value": 42,
            "nested": {"key": "value"}
        });

        let input = AgentInput::text("format json").with_parameter(
            "parameters".to_string(),
            serde_json::json!({
                "operation": "format",
                "input": input_json
            }),
        );

        let output = tool
            .execute(input, ExecutionContext::default())
            .await
            .unwrap();

        assert!(output.text.contains("{\n"));
        assert!(output.text.contains("  \"name\": \"test\""));
    }

    #[tokio::test]
    async fn test_minify_operation() {
        let tool = JsonProcessorTool::default();

        let input_json = serde_json::json!({
            "name": "test",
            "value": 42
        });

        let input = AgentInput::text("minify json").with_parameter(
            "parameters".to_string(),
            serde_json::json!({
                "operation": "minify",
                "input": input_json
            }),
        );

        let output = tool
            .execute(input, ExecutionContext::default())
            .await
            .unwrap();

        assert!(!output.text.contains('\n'));
        assert!(output.text.contains("{\"name\":\"test\",\"value\":42}"));
    }

    #[tokio::test]
    async fn test_transform_operation() {
        let tool = JsonProcessorTool::default();

        let input_json = serde_json::json!({
            "users": [
                {"name": "Alice", "age": 30},
                {"name": "Bob", "age": 25}
            ],
            "count": 2
        });

        // Test simple field access
        let input = AgentInput::text("transform json").with_parameter(
            "parameters".to_string(),
            serde_json::json!({
                "operation": "transform",
                "input": input_json.clone(),
                "query": ".count"
            }),
        );

        let output = tool
            .execute(input, ExecutionContext::default())
            .await
            .unwrap();

        assert_eq!(output.text.trim(), "2");

        // Test nested field access
        let input = AgentInput::text("transform json").with_parameter(
            "parameters".to_string(),
            serde_json::json!({
                "operation": "transform",
                "input": input_json,
                "query": ".users"
            }),
        );

        let output = tool
            .execute(input, ExecutionContext::default())
            .await
            .unwrap();

        let result: Value = serde_json::from_str(&output.text).unwrap();
        assert!(result.is_array());
        assert_eq!(result.as_array().unwrap().len(), 2);
    }

    #[tokio::test]
    async fn test_validate_operation() {
        let tool = JsonProcessorTool::default();

        let input_json = serde_json::json!({
            "name": "test",
            "age": 25
        });

        let schema = serde_json::json!({
            "type": "object",
            "properties": {
                "name": {"type": "string"},
                "age": {"type": "integer", "minimum": 0}
            },
            "required": ["name", "age"]
        });

        let input = AgentInput::text("validate json").with_parameter(
            "parameters".to_string(),
            serde_json::json!({
                "operation": "validate",
                "input": input_json,
                "schema": schema
            }),
        );

        let output = tool
            .execute(input, ExecutionContext::default())
            .await
            .unwrap();

        let metadata = &output.metadata;
        let result = metadata.extra.get("result").unwrap();
        let validation = result.get("validation").unwrap();
        assert_eq!(validation.get("is_valid").unwrap(), true);
    }

    #[tokio::test]
    async fn test_filter_operation() {
        let tool = JsonProcessorTool::default();

        let input_json = serde_json::json!([
            {"name": "Alice", "age": 30},
            {"name": "Bob", "age": 25},
            {"name": "Charlie", "age": 35}
        ]);

        let input = AgentInput::text("filter json").with_parameter(
            "parameters".to_string(),
            serde_json::json!({
                "operation": "filter",
                "input": input_json,
                "query": ".name == \"Alice\""
            }),
        );

        let output = tool
            .execute(input, ExecutionContext::default())
            .await
            .unwrap();

        let result: Value = serde_json::from_str(&output.text).unwrap();
        assert!(result.is_array());
        let arr = result.as_array().unwrap();
        assert_eq!(arr.len(), 1);
        assert_eq!(arr[0]["name"], "Alice");
    }

    #[tokio::test]
    async fn test_merge_operation() {
        let tool = JsonProcessorTool::default();

        let input_json = serde_json::json!([
            {"a": 1, "b": 2},
            {"b": 3, "c": 4},
            {"d": 5}
        ]);

        let input = AgentInput::text("merge json").with_parameter(
            "parameters".to_string(),
            serde_json::json!({
                "operation": "merge",
                "input": input_json
            }),
        );

        let output = tool
            .execute(input, ExecutionContext::default())
            .await
            .unwrap();

        let result: Value = serde_json::from_str(&output.text).unwrap();
        assert_eq!(
            result,
            serde_json::json!({
                "a": 1,
                "b": 3,  // Second object overwrites
                "c": 4,
                "d": 5
            })
        );
    }

    #[tokio::test]
    async fn test_invalid_query() {
        let tool = JsonProcessorTool::default();

        let input = AgentInput::text("transform json").with_parameter(
            "parameters".to_string(),
            serde_json::json!({
                "operation": "transform",
                "input": {"test": "value"},
                "query": "complex | jq | syntax"
            }),
        );

        let result = tool.execute(input, ExecutionContext::default()).await;
        assert!(result.is_err());

        if let Err(e) = result {
            match e {
                LLMSpellError::Validation { message, .. } => {
                    assert!(message.contains("not yet supported"));
                }
                _ => panic!("Expected validation error"),
            }
        }
    }

    #[tokio::test]
    async fn test_size_limit() {
        let config = JsonProcessorConfig {
            max_input_size: 100, // Very small limit
            ..Default::default()
        };
        let tool = JsonProcessorTool::new(config);

        let large_input = serde_json::json!({
            "data": "x".repeat(200)
        });

        let input = AgentInput::text("process json").with_parameter(
            "parameters".to_string(),
            serde_json::json!({
                "operation": "format",
                "input": large_input
            }),
        );

        let result = tool.validate_input(&input).await;
        assert!(result.is_err());
    }
}
