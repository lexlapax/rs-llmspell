//! ABOUTME: Streaming types for agent responses and chunk-based communication
//! ABOUTME: Provides AgentStream, AgentChunk, and related types for streaming LLM interactions

use crate::error::LLMSpellError;
use chrono::{DateTime, Utc};
use futures::Stream;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;
use std::pin::Pin;

/// Type alias for a stream of agent chunks.
///
/// This stream is pinned and boxed to allow for dynamic dispatch across
/// different stream implementations. Each item in the stream is a Result
/// containing either an AgentChunk or an error.
///
/// # Examples
///
/// ```no_run
/// use llmspell_core::types::{AgentStream, AgentChunk, ChunkContent};
/// use futures::stream;
///
/// fn create_text_stream() -> AgentStream {
///     Box::pin(stream::iter(vec![
///         Ok(AgentChunk {
///             stream_id: "stream-1".to_string(),
///             chunk_index: 0,
///             content: ChunkContent::Text("Hello, ".to_string()),
///             metadata: Default::default(),
///             timestamp: chrono::Utc::now(),
///         }),
///         Ok(AgentChunk {
///             stream_id: "stream-1".to_string(),
///             chunk_index: 1,
///             content: ChunkContent::Text("world!".to_string()),
///             metadata: Default::default(),
///             timestamp: chrono::Utc::now(),
///         }),
///     ]))
/// }
/// ```
pub type AgentStream = Pin<Box<dyn Stream<Item = Result<AgentChunk, LLMSpellError>> + Send>>;

/// Represents a single chunk in a streaming agent response.
///
/// Each chunk contains a piece of content (text, tool call, etc.) along with
/// metadata about its position in the stream and various properties.
///
/// # Examples
///
/// ```
/// use llmspell_core::types::{AgentChunk, ChunkContent, ChunkMetadata};
///
/// let chunk = AgentChunk {
///     stream_id: "conversation-123".to_string(),
///     chunk_index: 0,
///     content: ChunkContent::Text("The answer is".to_string()),
///     metadata: ChunkMetadata {
///         is_final: false,
///         token_count: Some(3),
///         model: Some("gpt-4".to_string()),
///         reasoning_step: None,
///     },
///     timestamp: chrono::Utc::now(),
/// };
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AgentChunk {
    /// Unique identifier for this stream
    pub stream_id: String,

    /// Sequential index of this chunk in the stream
    pub chunk_index: usize,

    /// The actual content of this chunk
    pub content: ChunkContent,

    /// Metadata about this chunk
    pub metadata: ChunkMetadata,

    /// Timestamp when this chunk was created
    pub timestamp: DateTime<Utc>,
}

impl fmt::Display for AgentChunk {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "AgentChunk[{}/{}]: {}",
            self.stream_id, self.chunk_index, self.content
        )
    }
}

/// Content types that can appear in agent chunks.
///
/// Different variants represent different kinds of streaming content,
/// from simple text to complex tool calls and media.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", content = "data")]
pub enum ChunkContent {
    /// Plain text content
    Text(String),

    /// Partial tool call in progress
    ToolCallProgress {
        /// Tool call ID
        call_id: String,
        /// Tool name being called
        tool_name: String,
        /// Partial arguments being built
        partial_args: String,
    },

    /// Complete tool call ready for execution
    ToolCallComplete {
        /// Tool call ID
        call_id: String,
        /// Tool name to execute
        tool_name: String,
        /// Complete arguments as JSON string
        arguments: String,
    },

    /// Media content (images, audio, etc.)
    Media {
        /// MIME type of the media
        mime_type: String,
        /// Base64 encoded data or URL
        data: String,
        /// Optional caption or description
        caption: Option<String>,
    },

    /// Control messages for stream management
    Control(ControlMessage),
}

impl fmt::Display for ChunkContent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ChunkContent::Text(text) => write!(f, "Text: {}", text),
            ChunkContent::ToolCallProgress { tool_name, .. } => {
                write!(f, "ToolCallProgress: {}", tool_name)
            }
            ChunkContent::ToolCallComplete { tool_name, .. } => {
                write!(f, "ToolCallComplete: {}", tool_name)
            }
            ChunkContent::Media {
                mime_type, caption, ..
            } => {
                write!(
                    f,
                    "Media[{}]{}",
                    mime_type,
                    caption
                        .as_ref()
                        .map(|c| format!(": {}", c))
                        .unwrap_or_default()
                )
            }
            ChunkContent::Control(msg) => write!(f, "Control: {}", msg),
        }
    }
}

/// Metadata associated with each chunk.
///
/// Provides additional context about the chunk such as whether it's
/// the final chunk, token counts, model information, etc.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct ChunkMetadata {
    /// Whether this is the final chunk in the stream
    pub is_final: bool,

    /// Number of tokens in this chunk (if available)
    pub token_count: Option<usize>,

    /// Model that generated this chunk
    pub model: Option<String>,

    /// If this chunk is part of a reasoning step
    pub reasoning_step: Option<ReasoningStep>,
}

impl fmt::Display for ChunkMetadata {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut parts = vec![];

        if self.is_final {
            parts.push("final".to_string());
        }

        if let Some(count) = self.token_count {
            parts.push(format!("{} tokens", count));
        }

        if let Some(model) = &self.model {
            parts.push(format!("model={}", model));
        }

        if let Some(step) = &self.reasoning_step {
            parts.push(format!("step={}", step.step_type));
        }

        write!(f, "Metadata[{}]", parts.join(", "))
    }
}

/// Information about reasoning steps in the agent's thought process.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ReasoningStep {
    /// Type of reasoning step
    pub step_type: String,

    /// Step number in sequence
    pub step_number: usize,

    /// Additional properties for this step
    pub properties: HashMap<String, String>,
}

/// Control messages for stream management.
///
/// These messages control the flow of the stream and provide
/// status updates about the streaming process.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ControlMessage {
    /// Stream has started
    StreamStart {
        /// Expected total chunks (if known)
        expected_chunks: Option<usize>,
        /// Stream configuration
        config: HashMap<String, String>,
    },

    /// Stream has ended normally
    StreamEnd {
        /// Total chunks sent
        total_chunks: usize,
        /// Total tokens used
        total_tokens: Option<usize>,
        /// Duration in milliseconds
        duration_ms: u64,
    },

    /// Stream was cancelled
    StreamCancelled {
        /// Reason for cancellation
        reason: String,
    },

    /// Heartbeat to keep connection alive
    Heartbeat,

    /// Rate limit information
    RateLimit {
        /// Requests remaining
        remaining: usize,
        /// Reset time
        reset_at: DateTime<Utc>,
    },

    /// Custom control message
    Custom {
        /// Message type
        message_type: String,
        /// Message payload
        payload: HashMap<String, String>,
    },
}

impl fmt::Display for ControlMessage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ControlMessage::StreamStart {
                expected_chunks, ..
            } => {
                write!(
                    f,
                    "StreamStart{}",
                    expected_chunks
                        .map(|n| format!(" (expecting {} chunks)", n))
                        .unwrap_or_default()
                )
            }
            ControlMessage::StreamEnd {
                total_chunks,
                duration_ms,
                ..
            } => {
                write!(
                    f,
                    "StreamEnd ({} chunks in {}ms)",
                    total_chunks, duration_ms
                )
            }
            ControlMessage::StreamCancelled { reason } => {
                write!(f, "StreamCancelled: {}", reason)
            }
            ControlMessage::Heartbeat => write!(f, "Heartbeat"),
            ControlMessage::RateLimit { remaining, .. } => {
                write!(f, "RateLimit ({} remaining)", remaining)
            }
            ControlMessage::Custom { message_type, .. } => {
                write!(f, "Custom[{}]", message_type)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg_attr(test_category = "unit")]
    #[test]
    fn test_agent_chunk_serialization() {
        let chunk = AgentChunk {
            stream_id: "test-stream".to_string(),
            chunk_index: 0,
            content: ChunkContent::Text("Hello".to_string()),
            metadata: ChunkMetadata::default(),
            timestamp: Utc::now(),
        };

        let json = serde_json::to_string(&chunk).unwrap();
        let deserialized: AgentChunk = serde_json::from_str(&json).unwrap();

        assert_eq!(chunk.stream_id, deserialized.stream_id);
        assert_eq!(chunk.chunk_index, deserialized.chunk_index);
        assert_eq!(chunk.content, deserialized.content);
        assert_eq!(chunk.metadata, deserialized.metadata);
    }

    #[cfg_attr(test_category = "unit")]
    #[test]
    fn test_chunk_content_variants_serialization() {
        let test_cases = vec![
            ChunkContent::Text("Hello world".to_string()),
            ChunkContent::ToolCallProgress {
                call_id: "call-123".to_string(),
                tool_name: "search".to_string(),
                partial_args: r#"{"query": "rust"#.to_string(),
            },
            ChunkContent::ToolCallComplete {
                call_id: "call-456".to_string(),
                tool_name: "calculator".to_string(),
                arguments: r#"{"expression": "2 + 2"}"#.to_string(),
            },
            ChunkContent::Media {
                mime_type: "image/png".to_string(),
                data: "base64data".to_string(),
                caption: Some("A test image".to_string()),
            },
            ChunkContent::Control(ControlMessage::Heartbeat),
        ];

        for content in test_cases {
            let json = serde_json::to_string(&content).unwrap();
            let deserialized: ChunkContent = serde_json::from_str(&json).unwrap();
            assert_eq!(content, deserialized);
        }
    }

    #[cfg_attr(test_category = "unit")]
    #[test]
    fn test_chunk_metadata_serialization() {
        let metadata = ChunkMetadata {
            is_final: true,
            token_count: Some(42),
            model: Some("gpt-4".to_string()),
            reasoning_step: Some(ReasoningStep {
                step_type: "analysis".to_string(),
                step_number: 1,
                properties: HashMap::from([("depth".to_string(), "3".to_string())]),
            }),
        };

        let json = serde_json::to_string(&metadata).unwrap();
        let deserialized: ChunkMetadata = serde_json::from_str(&json).unwrap();

        assert_eq!(metadata, deserialized);
    }

    #[cfg_attr(test_category = "unit")]
    #[test]
    fn test_control_message_variants_serialization() {
        let test_cases = vec![
            ControlMessage::StreamStart {
                expected_chunks: Some(10),
                config: HashMap::from([("mode".to_string(), "fast".to_string())]),
            },
            ControlMessage::StreamEnd {
                total_chunks: 5,
                total_tokens: Some(100),
                duration_ms: 1500,
            },
            ControlMessage::StreamCancelled {
                reason: "User interrupted".to_string(),
            },
            ControlMessage::Heartbeat,
            ControlMessage::RateLimit {
                remaining: 50,
                reset_at: Utc::now(),
            },
            ControlMessage::Custom {
                message_type: "debug".to_string(),
                payload: HashMap::from([("level".to_string(), "info".to_string())]),
            },
        ];

        for message in test_cases {
            let json = serde_json::to_string(&message).unwrap();
            let deserialized: ControlMessage = serde_json::from_str(&json).unwrap();
            assert_eq!(message, deserialized);
        }
    }

    #[cfg_attr(test_category = "unit")]
    #[test]
    fn test_display_implementations() {
        let chunk = AgentChunk {
            stream_id: "stream-1".to_string(),
            chunk_index: 5,
            content: ChunkContent::Text("Hello".to_string()),
            metadata: ChunkMetadata::default(),
            timestamp: Utc::now(),
        };

        assert_eq!(format!("{}", chunk), "AgentChunk[stream-1/5]: Text: Hello");

        let metadata = ChunkMetadata {
            is_final: true,
            token_count: Some(10),
            model: Some("gpt-4".to_string()),
            reasoning_step: None,
        };

        assert_eq!(
            format!("{}", metadata),
            "Metadata[final, 10 tokens, model=gpt-4]"
        );

        let control = ControlMessage::StreamEnd {
            total_chunks: 20,
            total_tokens: Some(500),
            duration_ms: 2000,
        };

        assert_eq!(format!("{}", control), "StreamEnd (20 chunks in 2000ms)");
    }

    #[cfg_attr(test_category = "unit")]
    #[test]
    fn test_default_chunk_metadata() {
        let metadata = ChunkMetadata::default();

        assert!(!metadata.is_final);
        assert!(metadata.token_count.is_none());
        assert!(metadata.model.is_none());
        assert!(metadata.reasoning_step.is_none());
    }
}
