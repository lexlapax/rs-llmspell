//! ABOUTME: JSON processing tool with full jq support using jaq
//! ABOUTME: Provides comprehensive JSON manipulation with real jq syntax support

use async_trait::async_trait;
use jaq_interpret::{FilterT, Val};
use jsonschema::{Draft, JSONSchema};
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
use llmspell_utils::{extract_parameters, response::ResponseBuilder};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::time::Instant;
use tokio::io::{AsyncBufReadExt, AsyncRead, BufReader};
use tracing::{debug, error, info, instrument, trace};

/// JSON processing operation types
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum JsonOperation {
    /// Transform JSON using full jq syntax
    Query,
    /// Validate JSON against a schema
    Validate,
    /// Stream process JSON lines
    Stream,
}

impl std::fmt::Display for JsonOperation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Query => write!(f, "query"),
            Self::Validate => write!(f, "validate"),
            Self::Stream => write!(f, "stream"),
        }
    }
}

impl std::str::FromStr for JsonOperation {
    type Err = LLMSpellError;

    fn from_str(s: &str) -> Result<Self> {
        match s.to_lowercase().as_str() {
            "query" | "transform" | "jq" => Ok(Self::Query),
            "validate" => Ok(Self::Validate),
            "stream" => Ok(Self::Stream),
            _ => Err(LLMSpellError::Validation {
                message: format!("Unknown JSON operation: {s}"),
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
    /// Maximum execution time in milliseconds
    pub max_execution_time_ms: u64,
}

impl Default for JsonProcessorConfig {
    fn default() -> Self {
        Self {
            max_input_size: 100 * 1024 * 1024, // 100MB
            enable_streaming: true,
            max_execution_time_ms: 30000, // 30 seconds
        }
    }
}

/// JSON processing tool with full jq support
pub struct JsonProcessorTool {
    metadata: ComponentMetadata,
    config: JsonProcessorConfig,
}

impl JsonProcessorTool {
    #[must_use]
    pub fn new(config: JsonProcessorConfig) -> Self {
        info!(
            tool_name = "json-processor",
            supported_operations = 3, // query, validate, stream
            jq_engine = "jaq",
            max_input_size_mb = config.max_input_size / (1024 * 1024),
            enable_streaming = config.enable_streaming,
            max_execution_time_seconds = config.max_execution_time_ms / 1000,
            security_level = "Safe",
            category = "Data",
            phase = "Phase 3 (comprehensive instrumentation)",
            "Creating JsonProcessorTool with configuration"
        );
        Self {
            metadata: ComponentMetadata::new(
                "json-processor".to_string(),
                "Process JSON with full jq syntax support using jaq engine".to_string(),
            ),
            config,
        }
    }

    /// Validate JQ query for security issues
    #[allow(clippy::unused_self)]
    fn validate_jq_query(&self, query: &str) -> Result<()> {
        // List of dangerous JQ functions and patterns
        let dangerous_patterns = [
            "env",        // Access environment variables
            "__inputs",   // Access inputs beyond the provided data
            "input",      // Read additional inputs
            "inputs",     // Read all inputs
            "debug",      // Debug output that might leak info
            "modulemeta", // Module metadata access
            "builtins",   // List all builtins
            "$__loc__",   // Location metadata
            "$ENV",       // Environment access
            "include",    // Include external files
            "import",     // Import modules
        ];

        let query_lower = query.to_lowercase();

        for pattern in &dangerous_patterns {
            if query_lower.contains(pattern) {
                return Err(LLMSpellError::Validation {
                    message: format!(
                        "Security: JQ query contains potentially dangerous function: {pattern}"
                    ),
                    field: Some("query".to_string()),
                });
            }
        }

        // Check for suspicious patterns that might indicate attempts to access system info
        if query_lower.contains("/etc/")
            || query_lower.contains("/proc/")
            || query_lower.contains("passwd")
            || query_lower.contains("shadow")
        {
            return Err(LLMSpellError::Validation {
                message: "Security: JQ query contains suspicious system path references"
                    .to_string(),
                field: Some("query".to_string()),
            });
        }

        Ok(())
    }

    /// Execute a jq query on JSON data
    fn execute_jq_query(&self, input: &Value, query: &str) -> Result<Vec<Value>> {
        debug!("Executing jq query: {}", query);

        // Validate query for security
        self.validate_jq_query(query)?;

        // Create parse context with core and std library
        let mut parse_ctx = jaq_interpret::ParseCtx::new(Vec::new());
        parse_ctx.insert_natives(jaq_core::core());
        parse_ctx.insert_defs(jaq_std::std());

        // Parse the filter
        let (filter, errs) = jaq_parse::parse(query, jaq_parse::main());
        if !errs.is_empty() {
            let error_msg = errs
                .into_iter()
                .map(|e| e.to_string())
                .collect::<Vec<_>>()
                .join("; ");
            return Err(LLMSpellError::Validation {
                message: format!("Invalid jq syntax: {error_msg}"),
                field: Some("query".to_string()),
            });
        }

        let filter = filter.ok_or_else(|| LLMSpellError::Validation {
            message: "Failed to parse jq filter".to_string(),
            field: Some("query".to_string()),
        })?;

        // Compile the filter
        let filter = parse_ctx.compile(filter);

        if !parse_ctx.errs.is_empty() {
            // jaq_interpret errors don't implement Display or Debug, so we'll use a generic message
            return Err(LLMSpellError::Validation {
                message: "Failed to compile jq filter: compilation errors occurred".to_string(),
                field: Some("query".to_string()),
            });
        }

        // Convert serde_json::Value to jaq Val
        let jaq_input = Val::from(input.clone());

        // Create empty inputs iterator
        let inputs = jaq_interpret::RcIter::new(core::iter::empty());

        // Run the filter
        let ctx = jaq_interpret::Ctx::new([], &inputs);
        let outputs = filter.run((ctx, jaq_input));

        // Convert results back to serde_json::Value
        let mut results = Vec::new();
        for output in outputs {
            match output {
                Ok(val) => {
                    // Convert Val to serde_json::Value
                    let json_val: Value = val.into();
                    results.push(json_val);
                }
                Err(e) => {
                    return Err(LLMSpellError::Tool {
                        message: format!("jq execution error: {e}"),
                        tool_name: Some("json_processor".to_string()),
                        source: None,
                    })
                }
            }
        }

        Ok(results)
    }

    /// Validate JSON against a schema
    #[allow(clippy::unused_async)]
    #[instrument(skip(self))]
    async fn validate_json(&self, input: &Value, schema: &Value) -> Result<ValidationResult> {
        debug!("Validating JSON against schema");

        let compiled = JSONSchema::options()
            .with_draft(Draft::Draft7)
            .compile(schema)
            .map_err(|e| LLMSpellError::Validation {
                message: format!("Invalid JSON schema: {e}"),
                field: Some("schema".to_string()),
            })?;

        let validation_result = compiled.validate(input);

        match validation_result {
            Ok(()) => Ok(ValidationResult {
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

    /// Process JSON lines with streaming
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The JQ query is invalid or contains security risks
    /// - JSON parsing fails for any line
    /// - I/O errors occur while reading from the stream
    #[instrument(skip(self, reader))]
    pub async fn process_json_stream<R: AsyncRead + Unpin>(
        &self,
        reader: R,
        query: &str,
    ) -> Result<Vec<Value>> {
        // Validate query for security before processing
        self.validate_jq_query(query)?;

        let mut buffer = BufReader::new(reader);
        let mut line = String::new();
        let mut results = Vec::new();

        while buffer.read_line(&mut line).await? > 0 {
            let trimmed = line.trim();
            if trimmed.is_empty() {
                line.clear();
                continue;
            }

            // Parse JSON line
            let value: Value =
                serde_json::from_str(trimmed).map_err(|e| LLMSpellError::Validation {
                    message: format!("Invalid JSON: {e}"),
                    field: Some("input".to_string()),
                })?;

            // Apply jq query
            let query_results = self.execute_jq_query(&value, query)?;
            results.extend(query_results);

            line.clear();
        }

        Ok(results)
    }

    /// Parse parameters from input
    ///
    /// # Errors
    ///
    /// Returns an error if the operation string cannot be parsed
    fn parse_parameters(params: &Value) -> Result<(JsonOperation, Option<Value>, Option<String>)> {
        let operation_str = params
            .get("operation")
            .and_then(|v| v.as_str())
            .unwrap_or("query");
        let operation: JsonOperation = operation_str.parse()?;

        // Handle input - could be a JSON string or already parsed
        let input = params.get("input").map(|v| {
            v.as_str().map_or_else(
                || v.clone(),
                |s| {
                    // Try to parse string as JSON
                    serde_json::from_str(s).unwrap_or_else(|_| v.clone())
                },
            )
        });
        let query = params
            .get("query")
            .and_then(|v| v.as_str())
            .map(String::from);

        Ok((operation, input, query))
    }
}

impl Default for JsonProcessorTool {
    fn default() -> Self {
        info!(
            tool_name = "json-processor",
            category = "Tool",
            phase = "Phase 3 (comprehensive instrumentation)",
            "Creating JsonProcessorTool"
        );

        Self::new(JsonProcessorConfig::default())
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct ValidationResult {
    is_valid: bool,
    errors: Vec<ValidationError>,
}

#[derive(Debug, Serialize, Deserialize)]
struct ValidationError {
    path: String,
    message: String,
    keyword: String,
}

#[async_trait]
impl BaseAgent for JsonProcessorTool {
    fn metadata(&self) -> &ComponentMetadata {
        &self.metadata
    }

    #[allow(clippy::too_many_lines)]
    #[instrument(skip(_context, input, self), fields(tool = %self.metadata().name))]
    async fn execute_impl(
        &self,
        input: AgentInput,
        _context: ExecutionContext,
    ) -> Result<AgentOutput> {
        let _execute_start = Instant::now();
        info!(
            tool_name = %self.metadata().name,
            input_text_length = input.text.len(),
            has_parameters = !input.parameters.is_empty(),
            "Starting JsonProcessorTool execution"
        );

        // Get parameters using shared utility
        let params = match extract_parameters(&input) {
            Ok(params) => {
                debug!(
                    param_count = params.as_object().map_or(0, serde_json::Map::len),
                    "Successfully extracted parameters"
                );
                params
            }
            Err(e) => {
                error!(
                    error = %e,
                    input_text_length = input.text.len(),
                    "Failed to extract parameters"
                );
                return Err(e);
            }
        };

        let parameter_parsing_start = Instant::now();
        let (operation, input_json, query) = match Self::parse_parameters(params) {
            Ok(parsed) => {
                let parameter_parsing_duration_ms = parameter_parsing_start.elapsed().as_millis();
                debug!(
                    operation = %parsed.0,
                    has_input_json = parsed.1.is_some(),
                    has_query = parsed.2.is_some(),
                    parameter_parsing_duration_ms,
                    "Successfully parsed operation parameters"
                );
                parsed
            }
            Err(e) => {
                let parameter_parsing_duration_ms = parameter_parsing_start.elapsed().as_millis();
                error!(
                    error = %e,
                    parameter_parsing_duration_ms,
                    "Failed to parse operation parameters"
                );
                return Err(e);
            }
        };

        info!(
            operation = %operation,
            has_input_json = input_json.is_some(),
            has_query = query.is_some(),
            input_json_type = input_json.as_ref().map(|v| match v {
                serde_json::Value::Null => "null",
                serde_json::Value::Bool(_) => "boolean",
                serde_json::Value::Number(_) => "number",
                serde_json::Value::String(_) => "string",
                serde_json::Value::Array(_) => "array",
                serde_json::Value::Object(_) => "object",
            }),
            input_json_size_estimate = input_json.as_ref()
                .and_then(|v| serde_json::to_string(v).ok())
                .map_or(0, |s| s.len()),
            query_length = query.as_ref().map(std::string::String::len),
            "Executing JSON operation"
        );

        trace!(
            input_json_preview = ?input_json.as_ref().map(|v|
                serde_json::to_string(v).unwrap_or_default()
                    .chars().take(200).collect::<String>()
            ),
            query_preview = ?query.as_ref().map(|q| q.chars().take(100).collect::<String>()),
            "Operation details"
        );

        let _operation_execution_start = Instant::now();
        let result = match operation {
            JsonOperation::Query => {
                debug!("Starting JSON query operation");

                let input_val = if let Some(val) = input_json {
                    debug!(
                        input_type = match &val {
                            serde_json::Value::Null => "null",
                            serde_json::Value::Bool(_) => "boolean",
                            serde_json::Value::Number(_) => "number",
                            serde_json::Value::String(_) => "string",
                            serde_json::Value::Array(arr) => {
                                if arr.len() > 100 {
                                    "large_array"
                                } else {
                                    "array"
                                }
                            }
                            serde_json::Value::Object(obj) => {
                                if obj.len() > 50 {
                                    "large_object"
                                } else {
                                    "object"
                                }
                            }
                        },
                        "Input value validated for query operation"
                    );
                    val
                } else {
                    error!("Query operation missing required 'input' parameter");
                    return Err(LLMSpellError::Validation {
                        message: "Query operation requires 'input' parameter".to_string(),
                        field: Some("input".to_string()),
                    });
                };

                let query_str = if let Some(q) = query {
                    debug!(
                        query_length = q.len(),
                        query_preview = %q.chars().take(50).collect::<String>(),
                        "Query string validated for query operation"
                    );
                    q
                } else {
                    error!("Query operation missing required 'query' parameter");
                    return Err(LLMSpellError::Validation {
                        message: "Query operation requires 'query' parameter".to_string(),
                        field: Some("query".to_string()),
                    });
                };

                let jq_execution_start = Instant::now();
                let results = match self.execute_jq_query(&input_val, &query_str) {
                    Ok(results) => {
                        let jq_execution_duration_ms = jq_execution_start.elapsed().as_millis();
                        debug!(
                            result_count = results.len(),
                            jq_execution_duration_ms, "JQ query executed successfully"
                        );
                        results
                    }
                    Err(e) => {
                        let jq_execution_duration_ms = jq_execution_start.elapsed().as_millis();
                        error!(
                            query = %query_str,
                            jq_execution_duration_ms,
                            error = %e,
                            "JQ query execution failed"
                        );
                        return Err(e);
                    }
                };

                // If single result, return it directly; otherwise return array
                let final_result = if results.len() == 1 {
                    trace!("Returning single query result");
                    results.into_iter().next().unwrap()
                } else {
                    trace!(
                        result_count = results.len(),
                        "Returning array of query results"
                    );
                    Value::Array(results)
                };

                debug!(
                    operation = "query",
                    input_size_estimate =
                        serde_json::to_string(&input_val).unwrap_or_default().len(),
                    query_length = query_str.len(),
                    result_size_estimate = serde_json::to_string(&final_result)
                        .unwrap_or_default()
                        .len(),
                    "Query operation completed successfully"
                );

                final_result
            }
            JsonOperation::Validate => {
                let input_val = input_json.ok_or_else(|| LLMSpellError::Validation {
                    message: "Validate operation requires 'input' parameter".to_string(),
                    field: Some("input".to_string()),
                })?;
                let schema = params
                    .get("schema")
                    .ok_or_else(|| LLMSpellError::Validation {
                        message: "Validate operation requires 'schema' parameter".to_string(),
                        field: Some("schema".to_string()),
                    })?;

                let validation = self.validate_json(&input_val, schema).await?;
                serde_json::to_value(validation)?
            }
            JsonOperation::Stream => {
                // For stream operation, input should contain the JSON lines content
                let stream_content = if let Some(input_val) = &input_json {
                    // If input is a string, use it directly
                    if let Some(content_str) = input_val.as_str() {
                        content_str.to_string()
                    } else {
                        // Otherwise convert to string
                        serde_json::to_string(input_val)?
                    }
                } else {
                    return Err(LLMSpellError::Validation {
                        message:
                            "Stream operation requires 'input' parameter with JSON lines content"
                                .to_string(),
                        field: Some("input".to_string()),
                    });
                };
                let query_str = query.unwrap_or_else(|| ".".to_string());

                // Process JSON lines
                let reader = stream_content.as_bytes();
                let results = self.process_json_stream(reader, &query_str).await?;

                // Return as array
                Value::Array(results)
            }
        };

        // Use ResponseBuilder for metadata, but return actual result as text
        let message = match operation {
            JsonOperation::Query => "JSON query executed successfully",
            JsonOperation::Validate => "JSON validation completed",
            JsonOperation::Stream => "JSON stream processing completed",
        };

        let response = ResponseBuilder::success(operation.to_string())
            .with_message(message.to_string())
            .with_result(result.clone())
            .build();

        let mut metadata = llmspell_core::types::OutputMetadata::default();
        metadata.extra.insert(
            "operation".to_string(),
            Value::String(operation.to_string()),
        );
        metadata.extra.insert("response".to_string(), response);

        // For data processing tools, return the actual result as text
        let output_text = serde_json::to_string_pretty(&result)?;
        Ok(AgentOutput::text(output_text).with_metadata(metadata))
    }

    #[instrument(skip(self))]
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

    #[instrument(skip(self))]
    async fn handle_error(&self, error: LLMSpellError) -> Result<AgentOutput> {
        Ok(AgentOutput::text(format!("JSON processing error: {error}")))
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
        ToolSchema {
            name: "json_processor".to_string(),
            description: "Process JSON data using full jq syntax with the jaq engine".to_string(),
            parameters: vec![
                ParameterDef {
                    name: "operation".to_string(),
                    description: "Operation to perform: query, validate, stream".to_string(),
                    param_type: ParameterType::String,
                    required: false,
                    default: Some(serde_json::json!("query")),
                },
                ParameterDef {
                    name: "input".to_string(),
                    description:
                        "Input JSON data (for stream operation, provide JSON lines as string)"
                            .to_string(),
                    param_type: ParameterType::Object,
                    required: false,
                    default: None,
                },
                ParameterDef {
                    name: "query".to_string(),
                    description: "jq query expression".to_string(),
                    param_type: ParameterType::String,
                    required: false,
                    default: Some(serde_json::json!(".")),
                },
                ParameterDef {
                    name: "schema".to_string(),
                    description: "JSON Schema for validation".to_string(),
                    param_type: ParameterType::Object,
                    required: false,
                    default: None,
                },
            ],
            returns: Some(ParameterType::Object),
        }
    }

    fn security_requirements(&self) -> SecurityRequirements {
        SecurityRequirements::safe()
    }

    fn resource_limits(&self) -> ResourceLimits {
        ResourceLimits::default()
            .with_memory_limit(self.config.max_input_size as u64)
            .with_cpu_limit(self.config.max_execution_time_ms)
    }
}

impl JsonProcessorTool {
    /// Check if this tool supports hook integration
    #[must_use]
    pub const fn supports_hooks(&self) -> bool {
        true // All tools that implement Tool automatically support hooks
    }

    /// Get hook integration metadata for this tool
    #[must_use]
    pub fn hook_metadata(&self) -> serde_json::Value {
        use serde_json::json;
        json!({
            "tool_name": self.metadata().name,
            "hook_points_supported": [
                "parameter_validation",
                "security_check",
                "resource_allocation",
                "pre_execution",
                "post_execution",
                "error_handling",
                "resource_cleanup",
                "timeout"
            ],
            "security_level": self.security_level(),
            "resource_limits": {
                "memory_mb": self.config.max_input_size / (1024 * 1024),
                "cpu_time_ms": self.config.max_execution_time_ms,
                "max_input_size_bytes": self.config.max_input_size,
                "data_processing_critical": true
            },
            "hook_integration_benefits": [
                "JSON query validation and sanitization",
                "Input size limits enforcement for DoS protection",
                "Memory usage monitoring for large JSON processing",
                "jq query complexity analysis and timeouts",
                "Schema validation performance tracking",
                "Error handling for malformed JSON and invalid queries"
            ],
            "security_considerations": [
                "Safe security level for JSON processing",
                "Input size limits to prevent memory exhaustion",
                "Query execution time limits to prevent infinite loops",
                "JSON schema validation for data integrity"
            ],
            "supported_operations": [
                "query (full jq syntax support)",
                "validate (JSON schema validation)",
                "format (pretty print and minify)",
                "extract (field extraction)"
            ]
        })
    }

    /// Demonstrate hook-aware execution for JSON processing
    /// This method showcases how the JSON processor tool works with the hook system
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Tool execution fails
    /// - Hook execution fails
    /// - JSON parsing or processing fails
    #[instrument(skip(self, tool_executor))]
    pub async fn demonstrate_hook_integration(
        &self,
        tool_executor: &crate::lifecycle::ToolExecutor,
        operation: &str,
        json_data: &str,
        query_or_schema: Option<&str>,
    ) -> Result<AgentOutput> {
        use crate::lifecycle::HookableToolExecution;

        let mut params = serde_json::json!({
            "operation": operation,
            "input": json_data,
            "hook_integration": true  // Flag to indicate this is a hook demo
        });

        if let Some(query_or_schema) = query_or_schema {
            match operation {
                "query" => params["query"] = serde_json::json!(query_or_schema),
                "validate" => params["schema"] = serde_json::json!(query_or_schema),
                _ => {}
            }
        }

        let input = AgentInput::text("JSON processing hook demonstration")
            .with_parameter("parameters", params);
        let context = ExecutionContext::default();

        // Execute with hooks using the HookableToolExecution trait
        self.execute_with_hooks(input, context, tool_executor).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_operation_parsing() {
        assert_eq!(
            "query".parse::<JsonOperation>().unwrap(),
            JsonOperation::Query
        );
        assert_eq!(
            "validate".parse::<JsonOperation>().unwrap(),
            JsonOperation::Validate
        );
        assert_eq!(
            "stream".parse::<JsonOperation>().unwrap(),
            JsonOperation::Stream
        );
        assert!("invalid".parse::<JsonOperation>().is_err());
    }
    #[tokio::test]
    async fn test_json_processor_creation() {
        let config = JsonProcessorConfig::default();
        let tool = JsonProcessorTool::new(config);

        // Just check that an ID exists
        assert_eq!(tool.metadata().name, "json-processor");
    }
    #[tokio::test]
    async fn test_jq_query() {
        let tool = JsonProcessorTool::default();

        let input_json = serde_json::json!({
            "users": [
                {"name": "Alice", "age": 30},
                {"name": "Bob", "age": 25},
                {"name": "Charlie", "age": 35}
            ]
        });

        // Test basic query
        let results = tool.execute_jq_query(&input_json, ".users[].name").unwrap();
        assert_eq!(results.len(), 3);
        assert!(results.contains(&serde_json::json!("Alice")));
        assert!(results.contains(&serde_json::json!("Bob")));
        assert!(results.contains(&serde_json::json!("Charlie")));

        // Test filter
        let results = tool
            .execute_jq_query(&input_json, ".users | map(select(.age > 26))")
            .unwrap();
        assert_eq!(results.len(), 1);
        let filtered = &results[0];
        assert!(filtered.is_array());
        assert_eq!(filtered.as_array().unwrap().len(), 2);

        // Test complex query
        let results = tool
            .execute_jq_query(&input_json, ".users | map(.age) | add / length")
            .unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0], serde_json::json!(30.0)); // Average age
    }
    #[test]
    fn test_hook_integration_metadata() {
        let tool = JsonProcessorTool::default();

        // Test that the tool supports hooks
        assert!(tool.supports_hooks());

        // Test hook metadata
        let metadata = tool.hook_metadata();
        assert_eq!(metadata["tool_name"], "json-processor");
        assert!(metadata["hook_points_supported"].is_array());
        assert_eq!(
            metadata["hook_points_supported"].as_array().unwrap().len(),
            8
        );
        assert!(metadata["hook_integration_benefits"].is_array());
        assert!(metadata["security_considerations"].is_array());
        assert_eq!(metadata["security_level"], "Safe");
        assert!(metadata["supported_operations"].is_array());
    }
    #[tokio::test]
    async fn test_json_processor_hook_integration() {
        use crate::lifecycle::{ToolExecutor, ToolLifecycleConfig};
        let tool = JsonProcessorTool::default();

        let config = ToolLifecycleConfig::default();
        let tool_executor = ToolExecutor::new(config, None, None);

        // Demonstrate hook integration with a simple JSON query
        let test_json = r#"{"name": "test", "value": 42}"#;
        let result = tool
            .demonstrate_hook_integration(&tool_executor, "query", test_json, Some(".name"))
            .await;

        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.text.contains("success") || output.text.contains("test"));
    }
    #[tokio::test]
    async fn test_hookable_tool_execution_trait_json() {
        use crate::lifecycle::{HookableToolExecution, ToolExecutor, ToolLifecycleConfig};
        let tool = JsonProcessorTool::default();

        // Verify the tool implements HookableToolExecution
        let config = ToolLifecycleConfig::default();
        let tool_executor = ToolExecutor::new(config, None, None);

        let input = AgentInput::text("Hook trait test").with_parameter(
            "parameters",
            serde_json::json!({
                "operation": "query",
                "input": r#"{"test": "data"}"#,
                "query": ".test"
            }),
        );
        let context = ExecutionContext::default();

        // This should compile and execute without errors
        let result = tool
            .execute_with_hooks(input, context, &tool_executor)
            .await;
        assert!(result.is_ok());
    }
}
