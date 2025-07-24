//! ABOUTME: BaseAgent trait - foundation for all components
//! ABOUTME: Provides core functionality for agents, tools, and workflows

use crate::execution_context::ExecutionContext;
use crate::types::{AgentInput, AgentOutput, AgentStream, MediaType};
use crate::{ComponentMetadata, Result};
use async_trait::async_trait;

/// Base trait for all components in the LLMSpell system.
///
/// This is the foundational trait that all agents, tools, and workflows must implement.
/// It provides the core interface for component execution, validation, and error handling.
///
/// # Implementation Requirements
///
/// - Components must be `Send + Sync` for async execution
/// - All methods should handle errors gracefully
/// - Input validation should be thorough but not overly restrictive
/// - Error handling should provide meaningful recovery options
///
/// # Examples
///
/// ```
/// use llmspell_core::{
///     ComponentMetadata, Result, LLMSpellError, ExecutionContext,
///     types::{AgentInput, AgentOutput},
///     traits::base_agent::BaseAgent
/// };
/// use async_trait::async_trait;
///
/// struct MyAgent {
///     metadata: ComponentMetadata,
/// }
///
/// #[async_trait]
/// impl BaseAgent for MyAgent {
///     fn metadata(&self) -> &ComponentMetadata {
///         &self.metadata
///     }
///     
///     async fn execute(
///         &self,
///         input: AgentInput,
///         context: ExecutionContext,
///     ) -> Result<AgentOutput> {
///         // Validate input first
///         self.validate_input(&input).await?;
///         
///         // Process the input
///         let result = format!("Processed: {}", input.text);
///         
///         Ok(AgentOutput::text(result))
///     }
///     
///     async fn validate_input(&self, input: &AgentInput) -> Result<()> {
///         if input.text.is_empty() {
///             return Err(LLMSpellError::Validation {
///                 message: "Text cannot be empty".to_string(),
///                 field: Some("text".to_string()),
///             });
///         }
///         Ok(())
///     }
///     
///     async fn handle_error(&self, error: LLMSpellError) -> Result<AgentOutput> {
///         Ok(AgentOutput::text(format!("Error handled: {}", error)))
///     }
/// }
/// ```
#[async_trait]
pub trait BaseAgent: Send + Sync {
    /// Get component metadata.
    ///
    /// Returns a reference to the component's metadata including its ID,
    /// name, version, and description. This metadata is immutable and
    /// identifies the component throughout its lifecycle.
    fn metadata(&self) -> &ComponentMetadata;

    /// Execute the component with given input.
    ///
    /// This is the main execution method for all components. It processes
    /// the input according to the component's logic and returns the result.
    ///
    /// # Arguments
    ///
    /// * `input` - The input containing prompt and optional context data
    /// * `context` - Execution context with session info and environment
    ///
    /// # Returns
    ///
    /// Returns `Ok(AgentOutput)` on success, or an error if execution fails.
    /// The output contains the result content and optional metadata.
    async fn execute(&self, input: AgentInput, context: ExecutionContext) -> Result<AgentOutput>;

    /// Validate input before execution.
    ///
    /// Called before `execute()` to validate the input parameters.
    /// Implementations should check for required fields, validate formats,
    /// and ensure the input meets the component's requirements.
    ///
    /// # Arguments
    ///
    /// * `input` - The input to validate
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` if validation passes, or a `Validation` error
    /// with details about what failed.
    async fn validate_input(&self, input: &AgentInput) -> Result<()>;

    /// Handle execution errors.
    ///
    /// Provides a way for components to handle errors gracefully and
    /// potentially recover or provide fallback responses. This method
    /// is called when an error occurs during execution.
    ///
    /// # Arguments
    ///
    /// * `error` - The error that occurred
    ///
    /// # Returns
    ///
    /// Returns an `AgentOutput` with error information or a fallback response,
    /// or propagates the error if it cannot be handled.
    async fn handle_error(&self, error: crate::LLMSpellError) -> Result<AgentOutput>;

    /// Execute the component with streaming output.
    ///
    /// This method provides streaming execution capabilities, allowing components
    /// to emit partial results as they become available. This is especially useful
    /// for LLM interactions where text can be generated incrementally.
    ///
    /// # Arguments
    ///
    /// * `input` - The input containing prompt and optional context data
    /// * `context` - Execution context with session info and environment
    ///
    /// # Returns
    ///
    /// Returns a stream of `AgentChunk` items containing partial results,
    /// or an error if streaming is not supported by this component.
    ///
    /// # Default Implementation
    ///
    /// The default implementation returns a NotImplemented error, indicating
    /// that the component does not support streaming execution.
    async fn stream_execute(
        &self,
        _input: AgentInput,
        _context: ExecutionContext,
    ) -> Result<AgentStream> {
        Err(crate::LLMSpellError::Component {
            message: "Streaming execution not supported by this component".to_string(),
            source: None,
        })
    }

    /// Check if this component supports streaming execution.
    ///
    /// Returns `true` if the component implements streaming via `stream_execute()`,
    /// `false` otherwise. Components that support streaming should override this
    /// method to return `true`.
    ///
    /// # Default Implementation
    ///
    /// Returns `false` by default, indicating no streaming support.
    fn supports_streaming(&self) -> bool {
        false
    }

    /// Check if this component supports multimodal content.
    ///
    /// Returns `true` if the component can process and generate content
    /// beyond plain text (images, audio, video, binary), `false` otherwise.
    ///
    /// # Default Implementation
    ///
    /// Returns `false` by default, indicating text-only support.
    fn supports_multimodal(&self) -> bool {
        false
    }

    /// Get the media types supported by this component.
    ///
    /// Returns a vector of `MediaType` values indicating which types of
    /// content this component can process in its input and/or generate
    /// in its output.
    ///
    /// # Default Implementation
    ///
    /// Returns only `MediaType::Text` by default.
    fn supported_media_types(&self) -> Vec<MediaType> {
        vec![MediaType::Text]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{AgentInput, AgentOutput};
    use crate::ExecutionContext;

    // Mock implementation for testing
    struct MockAgent {
        metadata: ComponentMetadata,
    }

    impl MockAgent {
        fn new() -> Self {
            Self {
                metadata: ComponentMetadata::new(
                    "mock-agent".to_string(),
                    "A mock agent for testing".to_string(),
                ),
            }
        }
    }

    #[async_trait]
    impl BaseAgent for MockAgent {
        fn metadata(&self) -> &ComponentMetadata {
            &self.metadata
        }

        async fn execute(
            &self,
            input: AgentInput,
            _context: ExecutionContext,
        ) -> Result<AgentOutput> {
            Ok(AgentOutput::text(format!("Processed: {}", input.text)))
        }

        async fn validate_input(&self, input: &AgentInput) -> Result<()> {
            if input.text.is_empty() {
                return Err(crate::LLMSpellError::Validation {
                    message: "Text cannot be empty".to_string(),
                    field: Some("text".to_string()),
                });
            }
            Ok(())
        }

        async fn handle_error(&self, error: crate::LLMSpellError) -> Result<AgentOutput> {
            Ok(AgentOutput::text(format!("Error handled: {}", error)))
        }
    }

    #[tokio::test]
    async fn test_base_agent_implementation() {
        let agent = MockAgent::new();

        // Test metadata access
        let metadata = agent.metadata();
        assert_eq!(metadata.name, "mock-agent");
        assert_eq!(metadata.description, "A mock agent for testing");

        // Test successful execution
        let input = AgentInput::text("test prompt");
        let context = ExecutionContext::new();
        let result = agent.execute(input, context).await.unwrap();
        assert_eq!(result.text, "Processed: test prompt");
    }

    #[tokio::test]
    async fn test_base_agent_validation() {
        let agent = MockAgent::new();

        // Test valid input
        let valid_input = AgentInput::text("valid prompt");
        assert!(agent.validate_input(&valid_input).await.is_ok());

        // Test invalid input
        let invalid_input = AgentInput::text("");
        let validation_result = agent.validate_input(&invalid_input).await;
        assert!(validation_result.is_err());

        if let Err(crate::LLMSpellError::Validation { message, .. }) = validation_result {
            assert_eq!(message, "Text cannot be empty");
        } else {
            panic!("Expected validation error");
        }
    }

    #[tokio::test]
    async fn test_base_agent_error_handling() {
        let agent = MockAgent::new();

        let error = crate::LLMSpellError::Component {
            message: "Test error".to_string(),
            source: None,
        };

        let result = agent.handle_error(error).await.unwrap();
        assert!(result.text.contains("Error handled"));
        assert!(result.text.contains("Test error"));
    }

    #[tokio::test]
    async fn test_base_agent_streaming_default() {
        let agent = MockAgent::new();

        // Test that streaming is not supported by default
        assert!(!agent.supports_streaming());

        // Test that multimodal is not supported by default
        assert!(!agent.supports_multimodal());

        // Test that only text is supported by default
        let supported_types = agent.supported_media_types();
        assert_eq!(supported_types.len(), 1);
        assert_eq!(supported_types[0], MediaType::Text);

        // Test that stream_execute returns NotImplemented error
        let input = AgentInput::text("test stream");
        let context = ExecutionContext::new();
        let stream_result = agent.stream_execute(input, context).await;
        assert!(stream_result.is_err());

        if let Err(crate::LLMSpellError::Component { message, .. }) = stream_result {
            assert!(message.contains("Streaming execution not supported"));
        } else {
            panic!("Expected Component error");
        }
    }
}
