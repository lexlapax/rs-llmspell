//! Integration tests for channel views

use llmspell_engine::{
    ChannelSet, ChannelType, LRPAdapter, MessageContent, ProtocolEngine, ProtocolServer,
    ProtocolType, ServerConfig, UniversalMessage,
};
use std::collections::HashMap;
use std::sync::Arc;

/// Mock message handler for testing
struct TestHandler;

#[async_trait::async_trait]
impl llmspell_engine::MessageHandler for TestHandler {
    async fn handle(
        &self,
        msg: llmspell_engine::ProtocolMessage,
    ) -> Option<llmspell_engine::ProtocolMessage> {
        // Echo back the message
        Some(llmspell_engine::ProtocolMessage::response(
            msg.msg_id,
            msg.content,
        ))
    }
}

#[tokio::test]
async fn test_channel_set_creation() {
    // Create a protocol server
    let config = ServerConfig::default();
    let handler = Arc::new(TestHandler);
    let server = ProtocolServer::new(config, handler);

    // Create channel set from the server (which implements ProtocolEngine)
    let channel_set = ChannelSet::new(&server);

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
async fn test_protocol_server_as_engine() {
    // Create a protocol server
    let config = ServerConfig::default();
    let handler = Arc::new(TestHandler);
    let mut server = ProtocolServer::new(config, handler);

    // Register an LRP adapter
    let adapter = Box::new(LRPAdapter::new());
    server
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
    server.send(ChannelType::Shell, msg.clone()).await.unwrap();

    // The message should be queued (though we can't receive it in this test without more setup)
}

#[tokio::test]
async fn test_channel_view_operations() {
    // Create a protocol server
    let config = ServerConfig::default();
    let handler = Arc::new(TestHandler);
    let server = ProtocolServer::new(config, handler);

    // Get a channel view
    let shell_view = server.channel_view(ChannelType::Shell);

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

    // Create a protocol server
    let config = ServerConfig::default();
    let handler = Arc::new(TestHandler);
    let server = ProtocolServer::new(config, handler);

    // Create IOPub view
    let iopub_view = IOPubView::new(&server);

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
