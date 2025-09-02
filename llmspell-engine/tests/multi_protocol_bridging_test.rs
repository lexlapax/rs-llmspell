//! Tests for multi-protocol bridging scenarios

use llmspell_engine::engine::{
    ChannelType, MessageContent, MessageRouter, ProtocolType, UniversalMessage,
};
use std::collections::HashMap;

// Allow similar names for protocol prefixes in multi-protocol bridging tests
// LRP/LDP are standard protocol identifiers where parallel naming aids semantic clarity

#[tokio::test]
#[allow(clippy::similar_names)] // LRP/LDP protocol prefixes are semantically meaningful
async fn test_lrp_to_ldp_message_conversion() {
    // Test converting LRP messages to LDP format via UniversalMessage
    let lrp_msg = UniversalMessage {
        id: "msg-1".to_string(),
        protocol: ProtocolType::LRP,
        channel: ChannelType::Shell,
        content: MessageContent::Request {
            method: "kernel_info_request".to_string(),
            params: serde_json::json!({}),
        },
        metadata: HashMap::new(),
    };

    // Convert to LDP format by changing protocol type
    let ldp_msg = UniversalMessage {
        id: lrp_msg.id.clone(),
        protocol: ProtocolType::LDP,
        channel: lrp_msg.channel,
        content: lrp_msg.content.clone(),
        metadata: lrp_msg.metadata.clone(),
    };

    assert_eq!(lrp_msg.id, ldp_msg.id);
    assert_eq!(lrp_msg.channel, ldp_msg.channel);
    assert_eq!(lrp_msg.content, ldp_msg.content);
    assert_ne!(lrp_msg.protocol, ldp_msg.protocol);
    assert_eq!(ldp_msg.protocol, ProtocolType::LDP);
}

#[tokio::test]
#[allow(clippy::similar_names)] // LRP/LDP protocol prefixes are semantically meaningful
async fn test_cross_protocol_routing() {
    let router = MessageRouter::new();

    // Register LRP handler for shell
    router
        .register_handler(
            ProtocolType::LRP,
            ChannelType::Shell,
            "lrp_handler".to_string(),
        )
        .await
        .unwrap();

    // Register LDP handler for shell
    router
        .register_handler(
            ProtocolType::LDP,
            ChannelType::Shell,
            "ldp_handler".to_string(),
        )
        .await
        .unwrap();

    // Route LRP message
    let lrp_msg = UniversalMessage {
        id: "lrp-msg".to_string(),
        protocol: ProtocolType::LRP,
        channel: ChannelType::Shell,
        content: MessageContent::Request {
            method: "execute".to_string(),
            params: serde_json::json!({"code": "print('hello')"}),
        },
        metadata: HashMap::new(),
    };

    let lrp_handlers = router.route(&lrp_msg).await.unwrap();
    assert_eq!(lrp_handlers, vec!["lrp_handler"]);

    // Route LDP message
    let ldp_msg = UniversalMessage {
        id: "ldp-msg".to_string(),
        protocol: ProtocolType::LDP,
        channel: ChannelType::Shell,
        content: MessageContent::Request {
            method: "debug_info".to_string(),
            params: serde_json::json!({"thread_id": 42}),
        },
        metadata: HashMap::new(),
    };

    let ldp_handlers = router.route(&ldp_msg).await.unwrap();
    assert_eq!(ldp_handlers, vec!["ldp_handler"]);
}

#[tokio::test]
async fn test_protocol_adapter_pattern() {
    // Test the concept of protocol adapters converting between formats

    // Simulate an adapter that converts between protocols
    struct ProtocolAdapter;

    impl ProtocolAdapter {
        fn adapt_lrp_to_ldp(lrp_msg: &UniversalMessage) -> UniversalMessage {
            // Convert LRP-style request to LDP-style request
            let new_content = match &lrp_msg.content {
                MessageContent::Request { method, params } => MessageContent::Request {
                    method: format!("debug_{method}"),
                    params: params.clone(),
                },
                other => other.clone(),
            };

            UniversalMessage {
                id: format!("adapted_{}", lrp_msg.id),
                protocol: ProtocolType::LDP,
                channel: lrp_msg.channel,
                content: new_content,
                metadata: {
                    let mut meta = lrp_msg.metadata.clone();
                    meta.insert("adapted_from".to_string(), serde_json::json!("LRP"));
                    meta
                },
            }
        }
    }

    let original_lrp = UniversalMessage {
        id: "original".to_string(),
        protocol: ProtocolType::LRP,
        channel: ChannelType::Shell,
        content: MessageContent::Request {
            method: "execute".to_string(),
            params: serde_json::json!({"code": "x = 42"}),
        },
        metadata: HashMap::new(),
    };

    let adapted_ldp = ProtocolAdapter::adapt_lrp_to_ldp(&original_lrp);

    // Verify adaptation
    assert_eq!(adapted_ldp.id, "adapted_original");
    assert_eq!(adapted_ldp.protocol, ProtocolType::LDP);
    assert_eq!(adapted_ldp.channel, ChannelType::Shell);

    if let MessageContent::Request { method, .. } = &adapted_ldp.content {
        assert_eq!(method, "debug_execute");
    } else {
        panic!("Expected Request content");
    }

    assert_eq!(
        adapted_ldp.metadata.get("adapted_from").unwrap(),
        &serde_json::json!("LRP")
    );
}

#[tokio::test]
async fn test_broadcast_across_protocols() {
    let router = MessageRouter::new();

    // Register handlers for different protocols on IOPub
    router
        .register_handler(
            ProtocolType::LRP,
            ChannelType::IOPub,
            "lrp_iopub".to_string(),
        )
        .await
        .unwrap();

    router
        .register_handler(
            ProtocolType::LDP,
            ChannelType::IOPub,
            "ldp_iopub".to_string(),
        )
        .await
        .unwrap();

    // Send a notification that should be broadcasted
    let notification = UniversalMessage {
        id: "broadcast-msg".to_string(),
        protocol: ProtocolType::LRP,
        channel: ChannelType::IOPub,
        content: MessageContent::Notification {
            event: "execution_state".to_string(),
            data: serde_json::json!({"state": "idle"}),
        },
        metadata: HashMap::new(),
    };

    // This should only go to LRP handlers since protocol is LRP
    let handlers = router.route(&notification).await.unwrap();
    assert_eq!(handlers, vec!["lrp_iopub"]);

    // For true cross-protocol broadcast, we'd need a different routing strategy
    // or a meta-protocol that handles cross-protocol communication
}

#[tokio::test]
async fn test_message_content_variants() {
    // Test all message content variants can be created and converted

    let request = MessageContent::Request {
        method: "test_method".to_string(),
        params: serde_json::json!({"param": "value"}),
    };

    let response = MessageContent::Response {
        result: Some(serde_json::json!({"success": true})),
        error: None,
    };

    let error_response = MessageContent::Response {
        result: None,
        error: Some(serde_json::json!({"code": -1, "message": "Test error"})),
    };

    let notification = MessageContent::Notification {
        event: "test_event".to_string(),
        data: serde_json::json!({"data": "value"}),
    };

    // Create UniversalMessage with each content type
    let messages = vec![
        (request, "request"),
        (response, "response"),
        (error_response, "error_response"),
        (notification, "notification"),
    ];

    for (content, name) in messages {
        let msg = UniversalMessage {
            id: format!("test_{name}"),
            protocol: ProtocolType::LRP,
            channel: ChannelType::Shell,
            content: content.clone(),
            metadata: HashMap::new(),
        };

        // Verify message can be created and content matches
        assert_eq!(msg.id, format!("test_{name}"));
        assert_eq!(msg.content, content);

        // Verify serialization/deserialization works
        let json = serde_json::to_string(&msg).unwrap();
        let deserialized: UniversalMessage = serde_json::from_str(&json).unwrap();
        assert_eq!(msg.id, deserialized.id);
        assert_eq!(msg.protocol, deserialized.protocol);
        assert_eq!(msg.content, deserialized.content);
    }
}
