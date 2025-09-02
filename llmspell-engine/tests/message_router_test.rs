//! Tests for `MessageRouter` routing strategies

use llmspell_engine::engine::{
    ChannelType, MessageContent, MessageRouter, ProtocolType, RoutingStrategy, UniversalMessage,
};
use std::collections::{HashMap, HashSet};

#[tokio::test]
async fn test_round_robin_routing() {
    let mut router = MessageRouter::new();

    // Set Shell channel to use RoundRobin for testing
    router.set_strategy(ChannelType::Shell, RoutingStrategy::RoundRobin);

    // Register 3 handlers for shell channel
    router
        .register_handler(
            ProtocolType::LRP,
            ChannelType::Shell,
            "handler1".to_string(),
        )
        .await
        .unwrap();

    router
        .register_handler(
            ProtocolType::LRP,
            ChannelType::Shell,
            "handler2".to_string(),
        )
        .await
        .unwrap();

    router
        .register_handler(
            ProtocolType::LRP,
            ChannelType::Shell,
            "handler3".to_string(),
        )
        .await
        .unwrap();

    // Create test messages
    let msg = UniversalMessage {
        id: "test-msg".to_string(),
        protocol: ProtocolType::LRP,
        channel: ChannelType::Shell,
        content: MessageContent::Request {
            method: "test".to_string(),
            params: serde_json::json!({"test": "data"}),
        },
        metadata: HashMap::new(),
    };

    // Route 6 messages and verify round-robin distribution
    let mut handler_counts = std::collections::HashMap::new();

    for _ in 0..6 {
        let handlers = router.route(&msg).await.unwrap();
        assert_eq!(handlers.len(), 1);

        let handler = &handlers[0];
        *handler_counts.entry(handler.clone()).or_insert(0) += 1;
    }

    // Each handler should have been selected exactly twice
    assert_eq!(handler_counts.len(), 3);
    for count in handler_counts.values() {
        assert_eq!(*count, 2);
    }
}

#[tokio::test]
async fn test_load_balanced_routing() {
    let mut router = MessageRouter::new();

    // Set Shell channel to use LoadBalanced for testing
    router.set_strategy(ChannelType::Shell, RoutingStrategy::LoadBalanced);

    // Register 2 handlers
    router
        .register_handler(
            ProtocolType::LRP,
            ChannelType::Shell,
            "handler1".to_string(),
        )
        .await
        .unwrap();

    router
        .register_handler(
            ProtocolType::LRP,
            ChannelType::Shell,
            "handler2".to_string(),
        )
        .await
        .unwrap();

    let msg = UniversalMessage {
        id: "test-msg".to_string(),
        protocol: ProtocolType::LRP,
        channel: ChannelType::Shell,
        content: MessageContent::Request {
            method: "test".to_string(),
            params: serde_json::json!({"test": "data"}),
        },
        metadata: HashMap::new(),
    };

    // Route a message (should go to handler with lowest load)
    let handlers = router.route(&msg).await.unwrap();
    let first_handler = handlers[0].clone();

    // Route another message (should go to the other handler since first has load)
    let handlers = router.route(&msg).await.unwrap();
    let second_handler = handlers[0].clone();

    // Handlers should be different
    assert_ne!(first_handler, second_handler);

    // Decrement load on first handler
    router.decrement_handler_load(&first_handler).await;

    // Next message should go to first handler again (now has lower load)
    let handlers = router.route(&msg).await.unwrap();
    assert_eq!(handlers[0], first_handler);
}

#[tokio::test]
async fn test_broadcast_routing() {
    let router = MessageRouter::new();

    // Register 3 handlers for IOPub (which uses Broadcast)
    let handler_ids: Vec<String> = (1..=3).map(|i| format!("handler{i}")).collect();

    for handler_id in &handler_ids {
        router
            .register_handler(ProtocolType::LRP, ChannelType::IOPub, handler_id.clone())
            .await
            .unwrap();
    }

    let msg = UniversalMessage {
        id: "test-msg".to_string(),
        protocol: ProtocolType::LRP,
        channel: ChannelType::IOPub,
        content: MessageContent::Notification {
            event: "test".to_string(),
            data: serde_json::json!({"test": "broadcast"}),
        },
        metadata: HashMap::new(),
    };

    // Route a message - should go to all handlers
    let handlers = router.route(&msg).await.unwrap();
    assert_eq!(handlers.len(), 3);

    // Verify all handlers are included
    let handler_set: HashSet<_> = handlers.into_iter().collect();
    let expected_set: HashSet<_> = handler_ids.into_iter().collect();
    assert_eq!(handler_set, expected_set);
}

#[tokio::test]
async fn test_direct_routing() {
    let router = MessageRouter::new();

    // Register 2 handlers for Shell (which uses Direct by default)
    router
        .register_handler(
            ProtocolType::LRP,
            ChannelType::Shell,
            "handler1".to_string(),
        )
        .await
        .unwrap();

    router
        .register_handler(
            ProtocolType::LRP,
            ChannelType::Shell,
            "handler2".to_string(),
        )
        .await
        .unwrap();

    let msg = UniversalMessage {
        id: "test-msg".to_string(),
        protocol: ProtocolType::LRP,
        channel: ChannelType::Shell,
        content: MessageContent::Request {
            method: "test".to_string(),
            params: serde_json::json!({"test": "direct"}),
        },
        metadata: HashMap::new(),
    };

    // Route multiple messages - should always go to first handler
    for _ in 0..5 {
        let handlers = router.route(&msg).await.unwrap();
        assert_eq!(handlers.len(), 1);
        assert_eq!(handlers[0], "handler1");
    }
}
