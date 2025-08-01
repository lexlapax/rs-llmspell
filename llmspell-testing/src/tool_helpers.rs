//! ABOUTME: Test helpers for tool creation and testing
//! ABOUTME: Provides common utilities for tool tests including mock tools and test data

//! Tool testing helpers.
//!
//! This module provides common test utilities for testing tools
//! including mock tool creation, test input generation, and
//! tool execution helpers.
//!
//! # Examples
//!
//! ```rust,no_run
//! use llmspell_testing::tool_helpers::{
//!     create_test_tool,
//!     create_test_tool_input,
//!     create_mock_tool,
//! };
//! use llmspell_core::ComponentMetadata;
//!
//! # async fn test_example() {
//! // Create a test tool
//! let tool = create_test_tool(
//!     "test-tool",
//!     "Test Tool",
//!     vec![("param1", "string"), ("param2", "number")]
//! );
//!
//! // Create test input
//! let input = create_test_tool_input(vec![
//!     ("operation", "test"),
//!     ("param1", "value"),
//!     ("param2", "42"),
//! ]);
//!
//! // Execute tool
//! let output = tool.execute(input, Default::default()).await.unwrap();
//! # }
//! ```

use llmspell_core::{
    execution_context::ExecutionContext,
    traits::tool::{ParameterDef, ParameterType, Tool, ToolCategory, ToolSchema, SecurityLevel},
    types::{AgentInput, AgentOutput, ToolOutput},
    ComponentMetadata, LLMSpellError,
};
use serde_json::json;
use std::collections::HashMap;

/// A simple test tool implementation
pub struct TestTool {
    metadata: ComponentMetadata,
    schema: ToolSchema,
    handler: Option<Box<dyn Fn(&AgentInput) -> Result<ToolOutput, LLMSpellError> + Send + Sync>>,
}

impl TestTool {
    /// Create a new test tool
    pub fn new(name: &str, description: &str) -> Self {
        let metadata = ComponentMetadata::new(name.to_string(), description.to_string());
        let schema = ToolSchema::new(name.to_string(), description.to_string());
        
        Self {
            metadata,
            schema,
            handler: None,
        }
    }

    /// Add a parameter to the tool schema
    pub fn with_parameter(mut self, name: &str, param_type: &str, required: bool) -> Self {
        let param_type = match param_type {
            "string" => ParameterType::String,
            "number" => ParameterType::Number,
            "boolean" => ParameterType::Boolean,
            "array" => ParameterType::Array,
            "object" => ParameterType::Object,
            _ => ParameterType::String,  // Default to string for unknown types
        };

        self.schema = self.schema.with_parameter(ParameterDef {
            name: name.to_string(),
            param_type,
            description: format!("{} parameter", name),
            required,
            default: None,
        });
        
        self
    }

    /// Set a custom handler for the tool
    pub fn with_handler<F>(mut self, handler: F) -> Self
    where
        F: Fn(&AgentInput) -> Result<ToolOutput, LLMSpellError> + Send + Sync + 'static,
    {
        self.handler = Some(Box::new(handler));
        self
    }
}

#[async_trait::async_trait]
impl Tool for TestTool {
    fn category(&self) -> ToolCategory {
        ToolCategory::Utility
    }

    fn security_level(&self) -> SecurityLevel {
        SecurityLevel::Safe
    }

    fn schema(&self) -> ToolSchema {
        self.schema.clone()
    }
}

#[async_trait::async_trait]
impl llmspell_core::traits::base_agent::BaseAgent for TestTool {
    fn metadata(&self) -> &ComponentMetadata {
        &self.metadata
    }

    async fn execute(
        &self,
        input: AgentInput,
        _context: ExecutionContext,
    ) -> Result<AgentOutput, LLMSpellError> {
        if let Some(handler) = &self.handler {
            let output = handler(&input)?;
            let tool_call = llmspell_core::types::ToolCall {
                tool_id: self.metadata.id.to_string(),
                tool_name: self.metadata.name.clone(),
                parameters: input.parameters.clone(),
                result: Some(output),
            };
            Ok(AgentOutput::builder()
                .text(format!("Executed tool: {}", self.metadata.name))
                .add_tool_call(tool_call)
                .build())
        } else {
            // Default behavior: echo back the input as JSON
            let output = ToolOutput {
                success: true,
                data: serde_json::to_value(&input.parameters).unwrap_or(json!({})),
                error: None,
                execution_time_ms: Some(0),
            };
            let tool_call = llmspell_core::types::ToolCall {
                tool_id: self.metadata.id.to_string(),
                tool_name: self.metadata.name.clone(),
                parameters: input.parameters.clone(),
                result: Some(output),
            };
            Ok(AgentOutput::builder()
                .text(format!("Executed tool: {}", self.metadata.name))
                .add_tool_call(tool_call)
                .build())
        }
    }

    async fn validate_input(&self, _input: &AgentInput) -> Result<(), LLMSpellError> {
        // Test tool accepts all inputs
        Ok(())
    }

    async fn handle_error(&self, error: LLMSpellError) -> Result<AgentOutput, LLMSpellError> {
        Ok(AgentOutput::text(format!("Tool error: {}", error)))
    }
}

/// Create a simple test tool
pub fn create_test_tool(
    name: &str,
    description: &str,
    parameters: Vec<(&str, &str)>,
) -> impl Tool {
    let mut tool = TestTool::new(name, description);
    
    for (param_name, param_type) in parameters {
        tool = tool.with_parameter(param_name, param_type, true);
    }
    
    tool
}

/// Create a test tool with custom handler
pub fn create_test_tool_with_handler<F>(
    name: &str,
    description: &str,
    parameters: Vec<(&str, &str)>,
    handler: F,
) -> impl Tool
where
    F: Fn(&AgentInput) -> Result<ToolOutput, LLMSpellError> + Send + Sync + 'static,
{
    let mut tool = TestTool::new(name, description);
    
    for (param_name, param_type) in parameters {
        tool = tool.with_parameter(param_name, param_type, true);
    }
    
    tool.with_handler(handler)
}

/// Create test tool input
pub fn create_test_tool_input(parameters: Vec<(&str, &str)>) -> AgentInput {
    let mut input = AgentInput::text("test tool execution");
    
    for (key, value) in parameters {
        // Try to parse as number or boolean first
        let json_value = if let Ok(n) = value.parse::<f64>() {
            json!(n)
        } else if let Ok(b) = value.parse::<bool>() {
            json!(b)
        } else {
            json!(value)
        };
        
        input = input.with_parameter(key, json_value);
    }
    
    input
}

/// Create test tool input with JSON values
pub fn create_test_tool_input_json(
    text: &str,
    parameters: HashMap<String, serde_json::Value>,
) -> AgentInput {
    let mut input = AgentInput::text(text);
    
    for (key, value) in parameters {
        input = input.with_parameter(&key, value);
    }
    
    input
}

/// Create a mock tool using mockall
pub fn create_mock_tool() -> crate::mocks::MockTool {
    use crate::mocks::MockTool;
    
    let mut mock = MockTool::new();
    
    // Set up default expectations
    mock.expect_metadata()
        .return_const(ComponentMetadata::new(
            "mock-tool".to_string(),
            "Mock tool for testing".to_string(),
        ));
    
    mock.expect_schema()
        .returning(|| ToolSchema::new(
            "mock-tool".to_string(),
            "Mock tool schema".to_string(),
        ));

    mock.expect_category()
        .returning(|| ToolCategory::Utility);

    mock.expect_security_level()
        .returning(|| SecurityLevel::Safe);
    
    mock
}

/// Common tool parameter sets for testing
pub mod common_params {
    /// File operation parameters
    pub fn file_params() -> Vec<(&'static str, &'static str)> {
        vec![
            ("operation", "string"),
            ("path", "string"),
            ("content", "string"),
            ("encoding", "string"),
        ]
    }
    
    /// Web request parameters
    pub fn web_params() -> Vec<(&'static str, &'static str)> {
        vec![
            ("operation", "string"),
            ("url", "string"),
            ("method", "string"),
            ("headers", "object"),
            ("body", "string"),
        ]
    }
    
    /// Process execution parameters
    pub fn process_params() -> Vec<(&'static str, &'static str)> {
        vec![
            ("operation", "string"),
            ("command", "string"),
            ("args", "array"),
            ("cwd", "string"),
            ("env", "object"),
        ]
    }
    
    /// Data processing parameters
    pub fn data_params() -> Vec<(&'static str, &'static str)> {
        vec![
            ("operation", "string"),
            ("input", "string"),
            ("format", "string"),
            ("options", "object"),
        ]
    }
}

/// Test data generators for tools
pub mod test_data {
    use super::*;
    
    /// Generate test file paths
    pub fn test_file_paths() -> Vec<String> {
        vec![
            "/tmp/test.txt".to_string(),
            "/tmp/data.json".to_string(),
            "/tmp/config.yaml".to_string(),
            "/tmp/script.py".to_string(),
        ]
    }
    
    /// Generate test URLs
    pub fn test_urls() -> Vec<String> {
        vec![
            "https://example.com".to_string(),
            "https://api.example.com/v1/data".to_string(),
            "http://localhost:8080".to_string(),
            "https://test.example.org/path/to/resource".to_string(),
        ]
    }
    
    /// Generate test JSON data
    pub fn test_json_data() -> Vec<serde_json::Value> {
        vec![
            json!({"key": "value"}),
            json!({"name": "test", "count": 42}),
            json!([1, 2, 3, 4, 5]),
            json!({"nested": {"data": {"value": true}}}),
        ]
    }
    
    /// Generate test commands
    pub fn test_commands() -> Vec<(&'static str, Vec<&'static str>)> {
        vec![
            ("echo", vec!["Hello, World!"]),
            ("ls", vec!["-la", "/tmp"]),
            ("python", vec!["-c", "print('test')"]),
            ("curl", vec!["-s", "https://example.com"]),
        ]
    }
}

/// Tool test assertions
pub mod assertions {
    use super::*;
    
    /// Assert tool output contains expected fields
    pub fn assert_tool_output_contains(
        output: &AgentOutput,
        expected_fields: Vec<&str>,
    ) -> Result<(), String> {
        // Check if the output contains tool calls
        if output.tool_calls.is_empty() {
            return Err("No tool calls found in output".to_string());
        }
        
        // Check the first tool call's result
        if let Some(result) = &output.tool_calls[0].result {
            let data = &result.data;
            for field in expected_fields {
                if data.get(field).is_none() {
                    return Err(format!("Tool output missing field: {}", field));
                }
            }
            Ok(())
        } else {
            Err("Tool call has no result".to_string())
        }
    }
    
    /// Assert tool output has success status
    pub fn assert_tool_success(output: &AgentOutput) -> Result<(), String> {
        if output.tool_calls.is_empty() {
            return Err("No tool calls found in output".to_string());
        }
        
        // Check the first tool call's result
        if let Some(result) = &output.tool_calls[0].result {
            if result.success {
                Ok(())
            } else {
                Err(format!("Tool output indicates failure: {:?}", result.error))
            }
        } else {
            Err("Tool call has no result".to_string())
        }
    }
    
    /// Assert tool output matches expected structure
    pub fn assert_tool_output_structure(
        output: &AgentOutput,
        expected_type: &str,
    ) -> Result<(), String> {
        if output.tool_calls.is_empty() {
            return Err("No tool calls found in output".to_string());
        }
        
        // Check the first tool call's result
        if let Some(result) = &output.tool_calls[0].result {
            let data = &result.data;
            if let Some(output_type) = data.get("type").and_then(|v| v.as_str()) {
                if output_type == expected_type {
                    Ok(())
                } else {
                    Err(format!(
                        "Expected output type '{}', got '{}'",
                        expected_type, output_type
                    ))
                }
            } else {
                // Check operation field as fallback
                if let Some(operation) = data.get("operation").and_then(|v| v.as_str()) {
                    if operation == expected_type {
                        Ok(())
                    } else {
                        Err(format!(
                            "Expected operation '{}', got '{}'",
                            expected_type, operation
                        ))
                    }
                } else {
                    Err("Tool output missing type/operation field".to_string())
                }
            }
        } else {
            Err("Tool call has no result".to_string())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use llmspell_core::traits::base_agent::BaseAgent;

    #[tokio::test]
    async fn test_create_test_tool() {
        let tool = create_test_tool(
            "test-tool",
            "A test tool",
            vec![("input", "string"), ("count", "number")],
        );
        
        assert_eq!(tool.metadata().name, "test-tool");
        assert_eq!(tool.schema().name, "test-tool");
        assert_eq!(tool.schema().parameters.len(), 2);
    }

    #[tokio::test]
    async fn test_create_test_tool_input() {
        let input = create_test_tool_input(vec![
            ("operation", "process"),
            ("count", "42"),
            ("enabled", "true"),
        ]);
        
        assert_eq!(input.text, "test tool execution");
        assert_eq!(input.parameters["operation"], json!("process"));
        assert_eq!(input.parameters["count"], json!(42.0));
        assert_eq!(input.parameters["enabled"], json!(true));
    }

    #[tokio::test]
    async fn test_tool_with_handler() {
        let tool = create_test_tool_with_handler(
            "echo-tool",
            "Echoes input",
            vec![("message", "string")],
            |input| {
                Ok(ToolOutput::new(serde_json::json!({
                    "echoed": input.parameters.get("message").cloned().unwrap_or(json!(""))
                })))
            },
        );
        
        let input = create_test_tool_input(vec![("message", "Hello")]);
        let output = tool.execute(input, ExecutionContext::default()).await.unwrap();
        
        // Check that we got a tool output with the echoed message
        assert!(!output.tool_calls.is_empty());
        if let Some(result) = &output.tool_calls[0].result {
            assert_eq!(result.data["echoed"], json!("Hello"));
        } else {
            panic!("Expected tool call result");
        }
    }

    #[test]
    fn test_common_params() {
        let file_params = common_params::file_params();
        assert!(file_params.iter().any(|(name, _)| *name == "path"));
        
        let web_params = common_params::web_params();
        assert!(web_params.iter().any(|(name, _)| *name == "url"));
    }

    #[test]
    fn test_assertions() {
        let tool_output = ToolOutput {
            success: true,
            data: json!({
                "result": "data",
                "type": "processed"
            }),
            error: None,
            execution_time_ms: Some(10),
        };
        let tool_call = llmspell_core::types::ToolCall {
            tool_id: "test-tool".to_string(),
            tool_name: "Test Tool".to_string(),
            parameters: HashMap::new(),
            result: Some(tool_output),
        };
        let output = AgentOutput::builder()
            .text("Tool executed")
            .add_tool_call(tool_call)
            .build();
        
        assert!(assertions::assert_tool_success(&output).is_ok());
        assert!(assertions::assert_tool_output_contains(&output, vec!["result"]).is_ok());
        assert!(assertions::assert_tool_output_structure(&output, "processed").is_ok());
    }
}