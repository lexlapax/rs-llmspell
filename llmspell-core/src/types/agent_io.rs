//! ABOUTME: Agent input/output types with multimodal support
//! ABOUTME: Provides AgentInput, AgentOutput, and related types for agent communication

use super::{ComponentId, MediaContent, MediaType};
use crate::execution_context::ExecutionContext;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::fmt;

/// Represents a tool call made during agent execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCall {
    /// ID of the tool being called
    pub tool_id: String,
    /// Tool name for display
    pub tool_name: String,
    /// Input parameters for the tool
    pub parameters: HashMap<String, Value>,
    /// Result of the tool call (if completed)
    pub result: Option<ToolOutput>,
}

/// Output from a tool execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolOutput {
    /// Success status
    pub success: bool,
    /// Output data
    pub data: Value,
    /// Error message if failed
    pub error: Option<String>,
    /// Execution time in milliseconds
    pub execution_time_ms: Option<u64>,
}

/// Metadata about agent output
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct OutputMetadata {
    /// Model used for generation
    pub model: Option<String>,
    /// Number of tokens used
    pub token_count: Option<u32>,
    /// Execution time in milliseconds
    pub execution_time_ms: Option<u64>,
    /// Confidence score (0.0 to 1.0)
    pub confidence: Option<f32>,
    /// Additional metadata
    pub extra: HashMap<String, Value>,
}

/// Enhanced agent input with multimodal support
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentInput {
    /// Text prompt or instruction
    pub text: String,
    /// Optional media content
    pub media: Vec<MediaContent>,
    /// Context from previous interactions
    pub context: Option<ExecutionContext>,
    /// Parameters for execution
    pub parameters: HashMap<String, Value>,
    /// Preferred output modalities
    pub output_modalities: Vec<MediaType>,
}

impl AgentInput {
    /// Create a text-only input
    pub fn text(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            media: vec![],
            context: None,
            parameters: HashMap::new(),
            output_modalities: vec![MediaType::Text],
        }
    }

    /// Add media content to the input
    pub fn with_media(mut self, media: MediaContent) -> Self {
        self.media.push(media);
        self
    }

    /// Add multiple media items
    pub fn with_media_vec(mut self, media: Vec<MediaContent>) -> Self {
        self.media.extend(media);
        self
    }

    /// Set the execution context
    pub fn with_context(mut self, context: ExecutionContext) -> Self {
        self.context = Some(context);
        self
    }

    /// Add a parameter
    pub fn with_parameter(mut self, key: impl Into<String>, value: impl Into<Value>) -> Self {
        self.parameters.insert(key.into(), value.into());
        self
    }

    /// Set output modalities
    pub fn with_output_modalities(mut self, modalities: Vec<MediaType>) -> Self {
        self.output_modalities = modalities;
        self
    }

    /// Create a builder for more complex inputs
    pub fn builder() -> AgentInputBuilder {
        AgentInputBuilder::new()
    }

    /// Check if input has media content
    pub fn has_media(&self) -> bool {
        !self.media.is_empty()
    }

    /// Get media of a specific type
    pub fn get_media_by_type(&self, media_type: MediaType) -> Vec<&MediaContent> {
        self.media
            .iter()
            .filter(|m| m.media_type() == media_type)
            .collect()
    }
}

impl fmt::Display for AgentInput {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "AgentInput {{ text: \"{}\", media: {} items",
            if self.text.len() > 50 {
                format!("{}...", &self.text[..50])
            } else {
                self.text.clone()
            },
            self.media.len()
        )?;
        if !self.parameters.is_empty() {
            write!(f, ", parameters: {} items", self.parameters.len())?;
        }
        write!(f, " }}")
    }
}

/// Builder for AgentInput
pub struct AgentInputBuilder {
    text: String,
    media: Vec<MediaContent>,
    context: Option<ExecutionContext>,
    parameters: HashMap<String, Value>,
    output_modalities: Vec<MediaType>,
}

impl AgentInputBuilder {
    /// Create a new builder
    pub fn new() -> Self {
        Self {
            text: String::new(),
            media: vec![],
            context: None,
            parameters: HashMap::new(),
            output_modalities: vec![MediaType::Text],
        }
    }

    /// Set the text
    pub fn text(mut self, text: impl Into<String>) -> Self {
        self.text = text.into();
        self
    }

    /// Add media content
    pub fn add_media(mut self, media: MediaContent) -> Self {
        self.media.push(media);
        self
    }

    /// Set the context
    pub fn context(mut self, context: ExecutionContext) -> Self {
        self.context = Some(context);
        self
    }

    /// Add a parameter
    pub fn parameter(mut self, key: impl Into<String>, value: impl Into<Value>) -> Self {
        self.parameters.insert(key.into(), value.into());
        self
    }

    /// Set output modalities
    pub fn output_modalities(mut self, modalities: Vec<MediaType>) -> Self {
        self.output_modalities = modalities;
        self
    }

    /// Build the AgentInput
    pub fn build(self) -> AgentInput {
        AgentInput {
            text: self.text,
            media: self.media,
            context: self.context,
            parameters: self.parameters,
            output_modalities: self.output_modalities,
        }
    }
}

impl Default for AgentInputBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Enhanced agent output with multimodal support
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentOutput {
    /// Primary text response
    pub text: String,
    /// Generated or processed media
    pub media: Vec<MediaContent>,
    /// Tool calls made during execution
    pub tool_calls: Vec<ToolCall>,
    /// Metadata about the execution
    pub metadata: OutputMetadata,
    /// Next agent to transfer to (if any)
    pub transfer_to: Option<ComponentId>,
}

impl AgentOutput {
    /// Create a text-only output
    pub fn text(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            media: vec![],
            tool_calls: vec![],
            metadata: OutputMetadata::default(),
            transfer_to: None,
        }
    }

    /// Add media content to the output
    pub fn with_media(mut self, media: MediaContent) -> Self {
        self.media.push(media);
        self
    }

    /// Add a tool call
    pub fn with_tool_call(mut self, tool_call: ToolCall) -> Self {
        self.tool_calls.push(tool_call);
        self
    }

    /// Set metadata
    pub fn with_metadata(mut self, metadata: OutputMetadata) -> Self {
        self.metadata = metadata;
        self
    }

    /// Set transfer target
    pub fn with_transfer(mut self, agent_id: ComponentId) -> Self {
        self.transfer_to = Some(agent_id);
        self
    }

    /// Create a builder for more complex outputs
    pub fn builder() -> AgentOutputBuilder {
        AgentOutputBuilder::new()
    }

    /// Check if output has media content
    pub fn has_media(&self) -> bool {
        !self.media.is_empty()
    }

    /// Check if output has tool calls
    pub fn has_tool_calls(&self) -> bool {
        !self.tool_calls.is_empty()
    }

    /// Check if this is a transfer response
    pub fn is_transfer(&self) -> bool {
        self.transfer_to.is_some()
    }
}

impl fmt::Display for AgentOutput {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "AgentOutput {{ text: \"{}\", media: {} items",
            if self.text.len() > 50 {
                format!("{}...", &self.text[..50])
            } else {
                self.text.clone()
            },
            self.media.len()
        )?;
        if !self.tool_calls.is_empty() {
            write!(f, ", tool_calls: {} items", self.tool_calls.len())?;
        }
        if let Some(ref agent_id) = self.transfer_to {
            write!(f, ", transfer_to: {}", agent_id)?;
        }
        write!(f, " }}")
    }
}

/// Builder for AgentOutput
pub struct AgentOutputBuilder {
    text: String,
    media: Vec<MediaContent>,
    tool_calls: Vec<ToolCall>,
    metadata: OutputMetadata,
    transfer_to: Option<ComponentId>,
}

impl AgentOutputBuilder {
    /// Create a new builder
    pub fn new() -> Self {
        Self {
            text: String::new(),
            media: vec![],
            tool_calls: vec![],
            metadata: OutputMetadata::default(),
            transfer_to: None,
        }
    }

    /// Set the text
    pub fn text(mut self, text: impl Into<String>) -> Self {
        self.text = text.into();
        self
    }

    /// Add media content
    pub fn add_media(mut self, media: MediaContent) -> Self {
        self.media.push(media);
        self
    }

    /// Add a tool call
    pub fn add_tool_call(mut self, tool_call: ToolCall) -> Self {
        self.tool_calls.push(tool_call);
        self
    }

    /// Set metadata
    pub fn metadata(mut self, metadata: OutputMetadata) -> Self {
        self.metadata = metadata;
        self
    }

    /// Set transfer target
    pub fn transfer_to(mut self, agent_id: ComponentId) -> Self {
        self.transfer_to = Some(agent_id);
        self
    }

    /// Build the AgentOutput
    pub fn build(self) -> AgentOutput {
        AgentOutput {
            text: self.text,
            media: self.media,
            tool_calls: self.tool_calls,
            metadata: self.metadata,
            transfer_to: self.transfer_to,
        }
    }
}

impl Default for AgentOutputBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_agent_input_text_constructor() {
        let input = AgentInput::text("Hello, world!");
        assert_eq!(input.text, "Hello, world!");
        assert!(input.media.is_empty());
        assert!(input.context.is_none());
        assert!(input.parameters.is_empty());
        assert_eq!(input.output_modalities, vec![MediaType::Text]);
    }

    #[test]
    fn test_agent_input_builder() {
        let context = ExecutionContext::new()
            .with_data("key".to_string(), Value::String("value".to_string()));

        let input = AgentInput::builder()
            .text("Test prompt")
            .context(context)
            .parameter("temperature", 0.7)
            .parameter("max_tokens", 100)
            .output_modalities(vec![MediaType::Text, MediaType::Image])
            .build();

        assert_eq!(input.text, "Test prompt");
        assert!(input.context.is_some());
        assert_eq!(input.parameters.len(), 2);
        assert_eq!(input.parameters["temperature"], 0.7);
        assert_eq!(input.parameters["max_tokens"], 100);
        assert_eq!(input.output_modalities.len(), 2);
    }

    #[test]
    fn test_agent_input_with_media() {
        let media = MediaContent::Text("Additional context".to_string());
        let input = AgentInput::text("Main prompt").with_media(media);

        assert_eq!(input.text, "Main prompt");
        assert_eq!(input.media.len(), 1);
        assert!(input.has_media());
    }

    #[test]
    fn test_agent_output_text_constructor() {
        let output = AgentOutput::text("Response text");
        assert_eq!(output.text, "Response text");
        assert!(output.media.is_empty());
        assert!(output.tool_calls.is_empty());
        assert!(output.transfer_to.is_none());
    }

    #[test]
    fn test_agent_output_builder() {
        let tool_call = ToolCall {
            tool_id: "tool-123".to_string(),
            tool_name: "Calculator".to_string(),
            parameters: HashMap::new(),
            result: None,
        };

        let metadata = OutputMetadata {
            model: Some("gpt-4".to_string()),
            token_count: Some(150),
            ..Default::default()
        };

        let output = AgentOutput::builder()
            .text("Calculated result")
            .add_tool_call(tool_call)
            .metadata(metadata)
            .build();

        assert_eq!(output.text, "Calculated result");
        assert_eq!(output.tool_calls.len(), 1);
        assert_eq!(output.tool_calls[0].tool_name, "Calculator");
        assert_eq!(output.metadata.model, Some("gpt-4".to_string()));
        assert_eq!(output.metadata.token_count, Some(150));
    }

    #[test]
    fn test_agent_output_with_transfer() {
        let agent_id = ComponentId::from_name("next-agent");
        let output = AgentOutput::text("Transferring to specialist").with_transfer(agent_id);

        assert!(output.is_transfer());
        assert_eq!(output.transfer_to, Some(agent_id));
    }

    #[test]
    fn test_execution_context() {
        let context = ExecutionContext::with_conversation("conv-123".to_string())
            .with_data("user_name".to_string(), Value::String("Alice".to_string()))
            .with_data(
                "session_type".to_string(),
                Value::String("chat".to_string()),
            );

        assert_eq!(context.conversation_id, Some("conv-123".to_string()));
        assert_eq!(context.data.len(), 2);
        assert_eq!(context.data["user_name"], "Alice");
    }

    #[test]
    fn test_tool_output() {
        let output = ToolOutput {
            success: true,
            data: serde_json::json!({"result": 42}),
            error: None,
            execution_time_ms: Some(25),
        };

        assert!(output.success);
        assert_eq!(output.data["result"], 42);
        assert!(output.error.is_none());
        assert_eq!(output.execution_time_ms, Some(25));
    }

    #[test]
    fn test_serialization() {
        let input = AgentInput::text("Test").with_parameter("key", "value");
        let json = serde_json::to_string(&input).unwrap();
        let deserialized: AgentInput = serde_json::from_str(&json).unwrap();
        assert_eq!(input.text, deserialized.text);
        assert_eq!(input.parameters, deserialized.parameters);

        let output = AgentOutput::text("Response");
        let json = serde_json::to_string(&output).unwrap();
        let deserialized: AgentOutput = serde_json::from_str(&json).unwrap();
        assert_eq!(output.text, deserialized.text);
    }

    #[test]
    fn test_display_formatting() {
        let input = AgentInput::text("This is a very long prompt that should be truncated in the display output for readability");
        let display = format!("{}", input);
        assert!(display.contains("..."));
        assert!(display.contains("AgentInput"));

        let output = AgentOutput::text("This is a very long response that should be truncated in the display output for readability");
        let display = format!("{}", output);
        assert!(display.contains("..."));
        assert!(display.contains("AgentOutput"));
    }
}
