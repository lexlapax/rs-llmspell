//! ABOUTME: Tests for streaming output functionality
//! ABOUTME: Validates progress indicators and Ctrl+C handling

use chrono::Utc;
use futures::stream;
use llmspell_bridge::engine::{ScriptMetadata, ScriptStream};
use llmspell_cli::{output::print_stream, OutputFormat};
use llmspell_core::types::{AgentChunk, ChunkContent, ChunkMetadata, ControlMessage};

#[tokio::test]
async fn test_streaming_text_output() {
    // Create a mock stream
    let chunks = vec![
        Ok(AgentChunk {
            stream_id: "test-stream".to_string(),
            chunk_index: 0,
            content: ChunkContent::Control(ControlMessage::StreamStart {
                expected_chunks: Some(3),
                config: Default::default(),
            }),
            metadata: ChunkMetadata::default(),
            timestamp: Utc::now(),
        }),
        Ok(AgentChunk {
            stream_id: "test-stream".to_string(),
            chunk_index: 1,
            content: ChunkContent::Text("Hello, ".to_string()),
            metadata: ChunkMetadata::default(),
            timestamp: Utc::now(),
        }),
        Ok(AgentChunk {
            stream_id: "test-stream".to_string(),
            chunk_index: 2,
            content: ChunkContent::Text("streaming world!".to_string()),
            metadata: ChunkMetadata {
                is_final: true,
                ..Default::default()
            },
            timestamp: Utc::now(),
        }),
    ];

    let stream = Box::pin(stream::iter(chunks));
    let mut script_stream = ScriptStream {
        stream,
        metadata: ScriptMetadata {
            engine: "test".to_string(),
            execution_time_ms: 0,
            memory_usage_bytes: None,
            warnings: vec![],
        },
    };

    // Test text output
    let result = print_stream(&mut script_stream, OutputFormat::Text).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_streaming_json_output() {
    // Create a mock stream with tool calls
    let chunks = vec![
        Ok(AgentChunk {
            stream_id: "test-stream".to_string(),
            chunk_index: 0,
            content: ChunkContent::ToolCallProgress {
                call_id: "call-1".to_string(),
                tool_name: "calculator".to_string(),
                partial_args: r#"{"expression": "2 +"#.to_string(),
            },
            metadata: ChunkMetadata::default(),
            timestamp: Utc::now(),
        }),
        Ok(AgentChunk {
            stream_id: "test-stream".to_string(),
            chunk_index: 1,
            content: ChunkContent::ToolCallComplete {
                call_id: "call-1".to_string(),
                tool_name: "calculator".to_string(),
                arguments: r#"{"expression": "2 + 2"}"#.to_string(),
            },
            metadata: ChunkMetadata::default(),
            timestamp: Utc::now(),
        }),
    ];

    let stream = Box::pin(stream::iter(chunks));
    let mut script_stream = ScriptStream {
        stream,
        metadata: ScriptMetadata {
            engine: "test".to_string(),
            execution_time_ms: 0,
            memory_usage_bytes: None,
            warnings: vec![],
        },
    };

    // Test JSON output
    let result = print_stream(&mut script_stream, OutputFormat::Json).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_streaming_with_media() {
    // Create a mock stream with media content
    let chunks = vec![
        Ok(AgentChunk {
            stream_id: "test-stream".to_string(),
            chunk_index: 0,
            content: ChunkContent::Text("Here's an image: ".to_string()),
            metadata: ChunkMetadata::default(),
            timestamp: Utc::now(),
        }),
        Ok(AgentChunk {
            stream_id: "test-stream".to_string(),
            chunk_index: 1,
            content: ChunkContent::Media {
                mime_type: "image/png".to_string(),
                data: "base64_encoded_image_data".to_string(),
                caption: Some("A beautiful sunset".to_string()),
            },
            metadata: ChunkMetadata::default(),
            timestamp: Utc::now(),
        }),
    ];

    let stream = Box::pin(stream::iter(chunks));
    let mut script_stream = ScriptStream {
        stream,
        metadata: ScriptMetadata {
            engine: "test".to_string(),
            execution_time_ms: 0,
            memory_usage_bytes: None,
            warnings: vec![],
        },
    };

    // Test pretty output with media
    let result = print_stream(&mut script_stream, OutputFormat::Pretty).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_streaming_control_messages() {
    // Create a mock stream with various control messages
    let chunks = vec![
        Ok(AgentChunk {
            stream_id: "test-stream".to_string(),
            chunk_index: 0,
            content: ChunkContent::Control(ControlMessage::StreamStart {
                expected_chunks: Some(5),
                config: Default::default(),
            }),
            metadata: ChunkMetadata::default(),
            timestamp: Utc::now(),
        }),
        Ok(AgentChunk {
            stream_id: "test-stream".to_string(),
            chunk_index: 1,
            content: ChunkContent::Control(ControlMessage::Heartbeat),
            metadata: ChunkMetadata::default(),
            timestamp: Utc::now(),
        }),
        Ok(AgentChunk {
            stream_id: "test-stream".to_string(),
            chunk_index: 2,
            content: ChunkContent::Text("Processing...".to_string()),
            metadata: ChunkMetadata::default(),
            timestamp: Utc::now(),
        }),
        Ok(AgentChunk {
            stream_id: "test-stream".to_string(),
            chunk_index: 3,
            content: ChunkContent::Control(ControlMessage::RateLimit {
                remaining: 100,
                reset_at: Utc::now() + chrono::Duration::minutes(5),
            }),
            metadata: ChunkMetadata::default(),
            timestamp: Utc::now(),
        }),
        Ok(AgentChunk {
            stream_id: "test-stream".to_string(),
            chunk_index: 4,
            content: ChunkContent::Control(ControlMessage::StreamEnd {
                total_chunks: 5,
                total_tokens: Some(150),
                duration_ms: 1234,
            }),
            metadata: ChunkMetadata::default(),
            timestamp: Utc::now(),
        }),
    ];

    let stream = Box::pin(stream::iter(chunks));
    let mut script_stream = ScriptStream {
        stream,
        metadata: ScriptMetadata {
            engine: "test".to_string(),
            execution_time_ms: 0,
            memory_usage_bytes: None,
            warnings: vec![],
        },
    };

    // Test pretty output with control messages
    let result = print_stream(&mut script_stream, OutputFormat::Pretty).await;
    assert!(result.is_ok());
}

// This test would be for manual testing of Ctrl+C handling
// It's commented out because it requires manual intervention
/*
#[tokio::test]
#[ignore = "Manual test - requires Ctrl+C interaction"]
async fn test_streaming_interruption() {
    // Create a slow stream that can be interrupted
    let chunks = futures::stream::unfold(0, |state| async move {
        if state < 100 {
            sleep(Duration::from_millis(500)).await;
            Some((
                Ok(AgentChunk {
                    stream_id: "slow-stream".to_string(),
                    chunk_index: state,
                    content: ChunkContent::Text(format!("Chunk {} ", state)),
                    metadata: ChunkMetadata::default(),
                    timestamp: Utc::now(),
                }),
                state + 1,
            ))
        } else {
            None
        }
    });

    let stream = Box::pin(chunks);
    let mut script_stream = ScriptStream {
        stream,
        metadata: Default::default(),
    };

    println!("Press Ctrl+C to interrupt the stream...");
    let result = print_stream(&mut script_stream, OutputFormat::Pretty).await;
    assert!(result.is_ok());
}
*/
