//! Integration tests for channel views

use llmspell_engine::{
    ChannelSet, ChannelType, LRPAdapter, MessageContent, ProtocolEngine, ProtocolType,
    UnifiedProtocolEngine, UniversalMessage,
};
use std::collections::HashMap;
use std::sync::Arc;

/// Test implementation of MessageProcessor for testing
#[derive(Debug)]
struct TestMessageProcessor;

#[async_trait::async_trait]
impl llmspell_engine::processor::MessageProcessor for TestMessageProcessor {
    async fn process_lrp(
        &self,
        request: llmspell_engine::protocol::lrp::LRPRequest,
    ) -> Result<
        llmspell_engine::protocol::lrp::LRPResponse,
        llmspell_engine::processor::ProcessorError,
    > {
        use llmspell_engine::processor::ProcessorError;
        use llmspell_engine::protocol::lrp::{LRPRequest, LRPResponse, LanguageInfo};

        // Return minimal valid responses for testing
        match request {
            LRPRequest::KernelInfoRequest => Ok(LRPResponse::KernelInfoReply {
                protocol_version: "5.3".to_string(),
                implementation: "test".to_string(),
                implementation_version: "0.0.0".to_string(),
                language_info: LanguageInfo {
                    name: "test".to_string(),
                    version: "0.0.0".to_string(),
                    mimetype: "text/plain".to_string(),
                    file_extension: ".test".to_string(),
                    pygments_lexer: None,
                    codemirror_mode: None,
                    nbconvert_exporter: None,
                },
                banner: "Test kernel".to_string(),
                debugger: false,
                help_links: vec![],
            }),
            _ => Err(ProcessorError::NotImplemented(
                "Test processor only handles kernel info".to_string(),
            )),
        }
    }

    async fn process_ldp(
        &self,
        _request: llmspell_engine::protocol::ldp::LDPRequest,
    ) -> Result<
        llmspell_engine::protocol::ldp::LDPResponse,
        llmspell_engine::processor::ProcessorError,
    > {
        use llmspell_engine::protocol::ldp::LDPResponse;

        // Return a simple response for testing
        Ok(LDPResponse::VariablesResponse { variables: vec![] })
    }
}

#[tokio::test]
async fn test_channel_set_creation() {
    // Create a unified protocol engine with mock transport
    let transport = Box::new(llmspell_engine::transport::mock::MockTransport::new());
    let processor = Arc::new(TestMessageProcessor);
    let engine = UnifiedProtocolEngine::with_processor(transport, processor);

    // Create channel set from the engine (which implements ProtocolEngine)
    let channel_set = ChannelSet::new(&engine);

    // Verify all channels are accessible
    let _shell = &channel_set.shell;
    let _iopub = &channel_set.iopub;
    let _stdin = &channel_set.stdin;
    let _control = &channel_set.control;
    let _heartbeat = &channel_set.heartbeat;

    // Get port information
    let ports = channel_set.get_ports();
    assert_eq!(ports.shell_port, 9555);
    assert_eq!(ports.iopub_port, 9556);
}

#[tokio::test]
async fn test_protocol_engine_with_adapter() {
    // Create a unified protocol engine
    let transport = Box::new(llmspell_engine::transport::mock::MockTransport::new());
    let processor = Arc::new(TestMessageProcessor);
    let mut engine = UnifiedProtocolEngine::with_processor(transport, processor);

    // Register an LRP adapter
    let adapter = Box::new(LRPAdapter::new());
    engine
        .register_adapter(ProtocolType::LRP, adapter)
        .await
        .unwrap();

    // Create a test message
    let msg = UniversalMessage {
        id: "test_msg".to_string(),
        protocol: ProtocolType::LRP,
        channel: ChannelType::Shell,
        content: MessageContent::Request {
            method: "test".to_string(),
            params: serde_json::json!({"key": "value"}),
        },
        metadata: HashMap::default(),
    };

    // Send message through the engine
    engine.send(ChannelType::Shell, msg.clone()).await.unwrap();

    // The message should be queued (though we can't receive it in this test without more setup)
}

#[tokio::test]
async fn test_channel_view_operations() {
    // Create a unified protocol engine
    let transport = Box::new(llmspell_engine::transport::mock::MockTransport::new());
    let processor = Arc::new(TestMessageProcessor);
    let engine = UnifiedProtocolEngine::with_processor(transport, processor);

    // Get a channel view
    let shell_view = engine.channel_view(ChannelType::Shell);

    // Create a test message
    let msg = UniversalMessage {
        id: "test_view_msg".to_string(),
        protocol: ProtocolType::LRP,
        channel: ChannelType::Shell,
        content: MessageContent::Request {
            method: "echo".to_string(),
            params: serde_json::json!({"text": "Hello, World!"}),
        },
        metadata: HashMap::default(),
    };

    // Send through the view
    shell_view.send(msg).await.unwrap();
}

#[tokio::test]
async fn test_iopub_broadcast() {
    use llmspell_engine::IOPubView;

    // Create a unified protocol engine
    let transport = Box::new(llmspell_engine::transport::mock::MockTransport::new());
    let processor = Arc::new(TestMessageProcessor);
    let engine = UnifiedProtocolEngine::with_processor(transport, processor);

    // Create IOPub view
    let iopub_view = IOPubView::new(&engine);

    // Subscribe to messages
    let mut subscriber = iopub_view.subscribe();

    // Publish a message
    iopub_view
        .publish(
            "status".to_string(),
            serde_json::json!({"execution_state": "idle"}),
        )
        .await
        .unwrap();

    // Should receive the broadcast
    if let Ok(msg) = subscriber.try_recv() {
        match msg {
            llmspell_engine::ChannelMessage::Broadcast { msg_type, .. } => {
                assert_eq!(msg_type, "status");
            }
            _ => panic!("Expected broadcast message"),
        }
    }
}

#[test]
fn test_iopub_message_compatibility() {
    use llmspell_engine::IOPubMessage;

    // Test that IOPubMessage enum works
    let msg = IOPubMessage::Status {
        execution_state: "busy".to_string(),
    };

    // Convert to channel message
    let channel_msg = msg.to_channel_message();

    match channel_msg {
        llmspell_engine::ChannelMessage::Broadcast { msg_type, .. } => {
            assert_eq!(msg_type, "status");
        }
        _ => panic!("Expected broadcast message"),
    }
}
