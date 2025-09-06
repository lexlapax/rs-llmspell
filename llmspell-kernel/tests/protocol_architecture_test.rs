//! Comprehensive tests for the protocol trait architecture

use anyhow::Result;
use llmspell_kernel::traits::output::OutputCapture;
use llmspell_kernel::traits::protocol::{
    ChannelTopology, ExecutionFlow, ExecutionResult, KernelStatus, OutputChunk, Protocol,
    ResponseCollector, ResponseFlow, StreamData,
};
use llmspell_kernel::traits::protocol::{ExecutionError, StreamType};
use llmspell_kernel::traits::KernelMessage;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use uuid::Uuid;

// Mock message type for testing
#[derive(Debug, Clone, Serialize, Deserialize)]
struct MockMessage {
    msg_type: String,
    content: Value,
    session: String,
    msg_id: String,
}

impl KernelMessage for MockMessage {
    fn msg_type(&self) -> &str {
        &self.msg_type
    }

    fn msg_id(&self) -> &str {
        &self.msg_id
    }

    fn session_id(&self) -> &str {
        &self.session
    }

    fn parent_id(&self) -> Option<&str> {
        None
    }

    fn content(&self) -> Value {
        self.content.clone()
    }

    fn metadata(&self) -> Value {
        serde_json::json!({})
    }

    fn set_parent(&mut self, _parent_id: String, _parent_type: String) {
        // Mock implementation
    }

    fn new(msg_type: String, content: Value) -> Self {
        Self {
            msg_type,
            content,
            session: "test-session".to_string(),
            msg_id: uuid::Uuid::new_v4().to_string(),
        }
    }

    fn set_parent_from_json(&mut self, _parent: Value) {
        // Mock implementation
    }
}

// Mock output context for testing
#[derive(Default)]
struct MockOutputContext {
    chunks: Vec<OutputChunk>,
}

// Mock protocol implementation
struct MockProtocol {
    pub status_messages_created: std::sync::Arc<std::sync::Mutex<Vec<KernelStatus>>>,
    pub output_flushed: std::sync::Arc<std::sync::Mutex<bool>>,
}

impl MockProtocol {
    fn new() -> Self {
        Self {
            status_messages_created: std::sync::Arc::new(std::sync::Mutex::new(Vec::new())),
            output_flushed: std::sync::Arc::new(std::sync::Mutex::new(false)),
        }
    }
}

impl Protocol for MockProtocol {
    type Message = MockMessage;
    type OutputContext = MockOutputContext;

    fn encode(&self, _msg: &Self::Message, _channel: &str) -> Result<Vec<Vec<u8>>> {
        Ok(vec![vec![1, 2, 3]])
    }

    fn decode(&self, _parts: Vec<Vec<u8>>, _channel: &str) -> Result<Self::Message> {
        Ok(MockMessage {
            msg_type: "test".to_string(),
            content: serde_json::json!({}),
            session: "test-session".to_string(),
            msg_id: "test-id".to_string(),
        })
    }

    fn transport_config(&self) -> llmspell_kernel::traits::TransportConfig {
        let mut channels = HashMap::new();
        channels.insert(
            "shell".to_string(),
            llmspell_kernel::traits::transport::ChannelConfig {
                pattern: "router".to_string(),
                endpoint: "5555".to_string(),
            },
        );
        llmspell_kernel::traits::TransportConfig {
            transport_type: "tcp".to_string(),
            base_address: "127.0.0.1".to_string(),
            channels,
        }
    }

    fn name(&self) -> &'static str {
        "mock"
    }

    fn version(&self) -> &'static str {
        "1.0"
    }

    fn requires_reply(&self, msg: &Self::Message) -> bool {
        msg.msg_type.ends_with("_request")
    }

    fn create_reply(
        &self,
        request: &Self::Message,
        content: serde_json::Value,
    ) -> Result<Self::Message> {
        let reply_type = request.msg_type.replace("_request", "_reply");
        Ok(MockMessage {
            msg_type: reply_type,
            content,
            session: request.session.clone(),
            msg_id: Uuid::new_v4().to_string(),
        })
    }

    fn reply_channel(&self, _msg: &Self::Message) -> &'static str {
        "shell"
    }

    fn create_broadcast(
        &self,
        msg_type: &str,
        content: serde_json::Value,
        _parent_msg: Option<&Self::Message>,
        _kernel_id: &str,
    ) -> Result<Self::Message> {
        Ok(MockMessage {
            msg_type: msg_type.to_string(),
            content,
            session: "test".to_string(),
            msg_id: Uuid::new_v4().to_string(),
        })
    }

    fn create_execution_flow(&self, _request: &Self::Message) -> ExecutionFlow<Self::Message> {
        ExecutionFlow {
            pre_execution: vec![
                (
                    "iopub".to_string(),
                    MockMessage {
                        msg_type: "status".to_string(),
                        content: serde_json::json!({"execution_state": "busy"}),
                        session: "test".to_string(),
                        msg_id: "pre-1".to_string(),
                    },
                ),
                (
                    "iopub".to_string(),
                    MockMessage {
                        msg_type: "execute_input".to_string(),
                        content: serde_json::json!({"code": "test"}),
                        session: "test".to_string(),
                        msg_id: "pre-2".to_string(),
                    },
                ),
            ],
            capture_output: true,
            post_execution: vec![(
                "iopub".to_string(),
                MockMessage {
                    msg_type: "status".to_string(),
                    content: serde_json::json!({"execution_state": "idle"}),
                    session: "test".to_string(),
                    msg_id: "post-1".to_string(),
                },
            )],
        }
    }

    fn create_status_message(&self, status: KernelStatus) -> Result<Self::Message> {
        self.status_messages_created
            .lock()
            .unwrap()
            .push(status.clone());
        let state = match status {
            KernelStatus::Idle => "idle",
            KernelStatus::Busy => "busy",
            KernelStatus::Starting => "starting",
        };
        Ok(MockMessage {
            msg_type: "status".to_string(),
            content: serde_json::json!({"execution_state": state}),
            session: "test".to_string(),
            msg_id: format!("status-{state}"),
        })
    }

    fn create_execute_input_message(&self, code: &str, _count: u32) -> Result<Self::Message> {
        Ok(MockMessage {
            msg_type: "execute_input".to_string(),
            content: serde_json::json!({"code": code}),
            session: "test".to_string(),
            msg_id: "execute_input".to_string(),
        })
    }

    fn create_stream_message(&self, stream: StreamData) -> Result<Self::Message> {
        let stream_name = match stream.stream_type {
            StreamType::Stdout => "stdout",
            StreamType::Stderr => "stderr",
        };
        Ok(MockMessage {
            msg_type: "stream".to_string(),
            content: serde_json::json!({
                "name": stream_name,
                "text": stream.text
            }),
            session: "test".to_string(),
            msg_id: "stream".to_string(),
        })
    }

    fn create_execute_result(&self, result: ExecutionResult) -> Result<Self::Message> {
        Ok(MockMessage {
            msg_type: "execute_result".to_string(),
            content: serde_json::json!({
                "execution_count": result.execution_count,
                "data": result.result_value
            }),
            session: "test".to_string(),
            msg_id: "result".to_string(),
        })
    }

    fn create_error_message(&self, error: ExecutionError) -> Result<Self::Message> {
        Ok(MockMessage {
            msg_type: "error".to_string(),
            content: serde_json::json!({
                "ename": error.name,
                "evalue": error.message,
                "traceback": error.traceback
            }),
            session: "test".to_string(),
            msg_id: "error".to_string(),
        })
    }

    fn create_output_context(&self) -> Self::OutputContext {
        MockOutputContext::default()
    }

    fn handle_output(&self, ctx: &mut Self::OutputContext, output: OutputChunk) {
        ctx.chunks.push(output);
    }

    fn flush_output(&self, ctx: Self::OutputContext) -> Vec<(String, Self::Message)> {
        *self.output_flushed.lock().unwrap() = true;
        let mut messages = Vec::new();

        for chunk in ctx.chunks {
            let msg = match chunk {
                OutputChunk::Stdout(text) => MockMessage {
                    msg_type: "stream".to_string(),
                    content: serde_json::json!({"name": "stdout", "text": text}),
                    session: "test".to_string(),
                    msg_id: "stdout".to_string(),
                },
                OutputChunk::Stderr(text) => MockMessage {
                    msg_type: "stream".to_string(),
                    content: serde_json::json!({"name": "stderr", "text": text}),
                    session: "test".to_string(),
                    msg_id: "stderr".to_string(),
                },
                OutputChunk::Result(data) => MockMessage {
                    msg_type: "execute_result".to_string(),
                    content: serde_json::json!({"data": data}),
                    session: "test".to_string(),
                    msg_id: "result".to_string(),
                },
                OutputChunk::Error(err) => MockMessage {
                    msg_type: "error".to_string(),
                    content: serde_json::json!({"error": err}),
                    session: "test".to_string(),
                    msg_id: "error".to_string(),
                },
            };
            messages.push(("iopub".to_string(), msg));
        }

        messages
    }

    fn channel_topology(&self) -> ChannelTopology {
        use llmspell_kernel::traits::protocol::ChannelPattern;
        let mut channels = HashMap::new();
        channels.insert("shell".to_string(), ChannelPattern::RequestReply);
        channels.insert("control".to_string(), ChannelPattern::RequestReply);
        channels.insert("iopub".to_string(), ChannelPattern::PubSub);
        channels.insert("stdin".to_string(), ChannelPattern::RequestReply);

        ChannelTopology {
            channels,
            shell_channel: "shell".to_string(),
            broadcast_channel: Some("iopub".to_string()),
        }
    }

    fn expected_response_flow(&self, msg_type: &str) -> ResponseFlow {
        use llmspell_kernel::traits::protocol::ExpectedMessage;
        match msg_type {
            "execute_request" => ResponseFlow {
                expected_messages: vec![
                    ExpectedMessage {
                        channel: "shell".to_string(),
                        message_type: "execute_reply".to_string(),
                        required: true,
                    },
                    ExpectedMessage {
                        channel: "iopub".to_string(),
                        message_type: "status".to_string(),
                        required: true,
                    },
                ],
                timeout_ms: 30000,
            },
            _ => ResponseFlow {
                expected_messages: vec![ExpectedMessage {
                    channel: "shell".to_string(),
                    message_type: format!("{}_reply", msg_type.trim_end_matches("_request")),
                    required: true,
                }],
                timeout_ms: 30000,
            },
        }
    }
}

// Mock OutputCapture implementation
struct MockOutputCapture {
    stdout: Vec<String>,
    stderr: Vec<String>,
    results: Vec<Value>,
    errors: Vec<ExecutionError>,
}

impl MockOutputCapture {
    const fn new() -> Self {
        Self {
            stdout: Vec::new(),
            stderr: Vec::new(),
            results: Vec::new(),
            errors: Vec::new(),
        }
    }
}

impl OutputCapture for MockOutputCapture {
    fn capture_stdout(&mut self, text: &str) {
        self.stdout.push(text.to_string());
    }

    fn capture_stderr(&mut self, text: &str) {
        self.stderr.push(text.to_string());
    }

    fn capture_result(&mut self, value: Value) {
        self.results.push(value);
    }

    fn capture_error(&mut self, error: ExecutionError) {
        self.errors.push(error);
    }

    fn flush(&mut self) {
        // No-op for test
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_protocol_creates_execution_flow() {
        let protocol = MockProtocol::new();
        let request = MockMessage {
            msg_type: "execute_request".to_string(),
            content: serde_json::json!({"code": "print('hello')"}),
            session: "test-session".to_string(),
            msg_id: "request-1".to_string(),
        };

        let flow = protocol.create_execution_flow(&request);

        // Verify pre-execution messages
        assert_eq!(flow.pre_execution.len(), 2);
        assert_eq!(flow.pre_execution[0].0, "iopub");
        assert_eq!(flow.pre_execution[0].1.msg_type, "status");
        assert_eq!(flow.pre_execution[1].1.msg_type, "execute_input");

        // Verify output capture is enabled
        assert!(flow.capture_output);

        // Verify post-execution messages
        assert_eq!(flow.post_execution.len(), 1);
        assert_eq!(flow.post_execution[0].0, "iopub");
        assert_eq!(flow.post_execution[0].1.msg_type, "status");
    }

    #[test]
    fn test_protocol_creates_status_messages() {
        let protocol = MockProtocol::new();

        let busy_msg = protocol.create_status_message(KernelStatus::Busy).unwrap();
        assert_eq!(busy_msg.msg_type, "status");
        assert_eq!(
            busy_msg.content["execution_state"],
            serde_json::json!("busy")
        );

        let idle_msg = protocol.create_status_message(KernelStatus::Idle).unwrap();
        assert_eq!(
            idle_msg.content["execution_state"],
            serde_json::json!("idle")
        );

        // Verify tracking
        let statuses = protocol.status_messages_created.lock().unwrap();
        assert_eq!(statuses.len(), 2);
        drop(statuses);
    }

    #[test]
    fn test_protocol_handles_output_chunks() {
        let protocol = MockProtocol::new();
        let mut ctx = protocol.create_output_context();

        // Add various output chunks
        protocol.handle_output(&mut ctx, OutputChunk::Stdout("Hello ".to_string()));
        protocol.handle_output(&mut ctx, OutputChunk::Stdout("World\n".to_string()));
        protocol.handle_output(&mut ctx, OutputChunk::Stderr("Error!\n".to_string()));
        protocol.handle_output(
            &mut ctx,
            OutputChunk::Result(serde_json::json!({"text/plain": "42"})),
        );

        // Verify chunks were collected
        assert_eq!(ctx.chunks.len(), 4);

        // Flush and get messages
        let messages = protocol.flush_output(ctx);
        assert_eq!(messages.len(), 4);
        assert!(messages.iter().all(|(ch, _)| ch == "iopub"));

        // Verify flush was called
        assert!(*protocol.output_flushed.lock().unwrap());
    }

    #[test]
    fn test_protocol_channel_topology() {
        let protocol = MockProtocol::new();
        let topology = protocol.channel_topology();

        assert!(topology.channels.contains_key("shell"));
        assert!(topology.channels.contains_key("control"));
        assert!(topology.channels.contains_key("iopub"));
        assert_eq!(topology.shell_channel, "shell");
        assert_eq!(topology.broadcast_channel, Some("iopub".to_string()));
    }

    #[test]
    fn test_protocol_response_flow() {
        let protocol = MockProtocol::new();

        // Execute request flow
        let exec_flow = protocol.expected_response_flow("execute_request");
        assert_eq!(exec_flow.expected_messages.len(), 2);
        assert_eq!(exec_flow.expected_messages[0].channel, "shell");
        assert_eq!(exec_flow.expected_messages[0].message_type, "execute_reply");
        assert_eq!(exec_flow.expected_messages[1].channel, "iopub");
        assert_eq!(exec_flow.timeout_ms, 30000);

        // Other request flow
        let other_flow = protocol.expected_response_flow("kernel_info_request");
        assert_eq!(other_flow.expected_messages.len(), 1);
        assert_eq!(other_flow.expected_messages[0].channel, "shell");
        assert_eq!(
            other_flow.expected_messages[0].message_type,
            "kernel_info_reply"
        );
    }

    #[test]
    fn test_output_capture_trait() {
        let mut capture = MockOutputCapture::new();

        // Test stdout capture
        capture.capture_stdout("Hello ");
        capture.capture_stdout("World");
        assert_eq!(capture.stdout.len(), 2);
        assert_eq!(capture.stdout[0], "Hello ");

        // Test stderr capture
        capture.capture_stderr("Error message");
        assert_eq!(capture.stderr.len(), 1);

        // Test result capture
        capture.capture_result(serde_json::json!({"value": 42}));
        assert_eq!(capture.results.len(), 1);

        // Test error capture
        capture.capture_error(ExecutionError {
            name: "RuntimeError".to_string(),
            message: "Something failed".to_string(),
            traceback: vec!["line 1".to_string()],
        });
        assert_eq!(capture.errors.len(), 1);
        assert_eq!(capture.errors[0].name, "RuntimeError");
    }

    #[test]
    fn test_response_collector() {
        let mut collector = ResponseCollector::default();

        // Add shell reply
        let _ = collector.add_message(
            "shell",
            serde_json::json!({
                "msg_type": "execute_reply",
                "content": {"status": "ok"}
            }),
        );

        // Add IOPub messages
        let _ = collector.add_message(
            "iopub",
            serde_json::json!({
                "msg_type": "status",
                "content": {"execution_state": "busy"}
            }),
        );

        let _ = collector.add_message(
            "iopub",
            serde_json::json!({
                "msg_type": "stream",
                "content": {"name": "stdout", "text": "output"}
            }),
        );

        // Check message counts
        assert_eq!(collector.shell_messages.len(), 1);
        assert_eq!(collector.iopub_messages.len(), 2);

        // Mark as idle
        let _ = collector.add_message(
            "iopub",
            serde_json::json!({
                "msg_type": "status",
                "content": {"execution_state": "idle"}
            }),
        );

        assert!(collector.received_idle);
    }

    #[test]
    fn test_stream_message_creation() {
        let protocol = MockProtocol::new();

        let stream = StreamData {
            stream_type: StreamType::Stdout,
            text: "Hello, world!\n".to_string(),
        };

        let msg = protocol.create_stream_message(stream).unwrap();
        assert_eq!(msg.msg_type, "stream");
        assert_eq!(msg.content["name"], "stdout");
        assert_eq!(msg.content["text"], "Hello, world!\n");
    }

    #[test]
    fn test_error_message_creation() {
        let protocol = MockProtocol::new();

        let error = ExecutionError {
            name: "ValueError".to_string(),
            message: "Invalid value".to_string(),
            traceback: vec!["File test.py, line 1".to_string()],
        };

        let msg = protocol.create_error_message(error).unwrap();
        assert_eq!(msg.msg_type, "error");
        assert_eq!(msg.content["ename"], "ValueError");
        assert_eq!(msg.content["evalue"], "Invalid value");
    }

    #[test]
    fn test_execute_result_creation() {
        let protocol = MockProtocol::new();

        let result = ExecutionResult {
            execution_count: 1,
            result_value: Some(serde_json::json!({
                "text/plain": "42",
                "text/html": "<b>42</b>"
            })),
            output: vec![],
            errors: vec![],
        };

        let msg = protocol.create_execute_result(result).unwrap();
        assert_eq!(msg.msg_type, "execute_result");
        assert_eq!(msg.content["execution_count"], 1);
        let data = msg.content["data"].as_object().unwrap();
        assert_eq!(data["text/plain"], "42");
    }
}
