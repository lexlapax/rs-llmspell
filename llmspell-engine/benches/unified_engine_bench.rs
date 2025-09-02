//! Benchmarks for `UnifiedProtocolEngine` architecture

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use llmspell_engine::engine::{
    ChannelType, MessageContent, MessageRouter, ProtocolType, RoutingStrategy, UniversalMessage,
};
use llmspell_engine::protocol::message::{MessageType, ProtocolMessage};
use llmspell_engine::protocol::LRPRequest;
use std::collections::HashMap;
use std::time::Duration;
use tokio::runtime::Runtime;

/// Benchmark `MessageRouter` routing strategies
fn bench_message_router_strategies(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let mut group = c.benchmark_group("message_router");

    // Setup router with handlers
    let router = MessageRouter::new();
    rt.block_on(async {
        for i in 1..=10 {
            router
                .register_handler(ProtocolType::LRP, ChannelType::Shell, format!("handler{i}"))
                .await
                .unwrap();
        }
    });

    let msg = UniversalMessage {
        id: "bench-msg".to_string(),
        protocol: ProtocolType::LRP,
        channel: ChannelType::Shell,
        content: MessageContent::Request {
            method: "test".to_string(),
            params: serde_json::json!({}),
        },
        metadata: HashMap::new(),
    };

    // Benchmark Direct routing
    group.bench_function("direct_routing", |b| {
        b.to_async(&rt).iter(|| async {
            let result = router.route(&msg).await.unwrap();
            black_box(result);
        });
    });

    // Benchmark RoundRobin routing
    let mut rr_router = create_router_with_handlers(&rt);
    rr_router.set_strategy(ChannelType::Shell, RoutingStrategy::RoundRobin);
    group.bench_function("round_robin_routing", |b| {
        b.to_async(&rt).iter(|| async {
            let result = rr_router.route(&msg).await.unwrap();
            black_box(result);
        });
    });

    // Benchmark LoadBalanced routing
    let mut lb_router = create_router_with_handlers(&rt);
    lb_router.set_strategy(ChannelType::Shell, RoutingStrategy::LoadBalanced);
    group.bench_function("load_balanced_routing", |b| {
        b.to_async(&rt).iter(|| async {
            let result = lb_router.route(&msg).await.unwrap();
            black_box(result);
        });
    });

    // Benchmark Broadcast routing (with IOPub channel)
    let broadcast_msg = UniversalMessage {
        id: "bench-msg".to_string(),
        protocol: ProtocolType::LRP,
        channel: ChannelType::IOPub,
        content: MessageContent::Notification {
            event: "test".to_string(),
            data: serde_json::json!({}),
        },
        metadata: HashMap::new(),
    };

    rt.block_on(async {
        for i in 1..=10 {
            router
                .register_handler(
                    ProtocolType::LRP,
                    ChannelType::IOPub,
                    format!("iopub_handler{i}"),
                )
                .await
                .unwrap();
        }
    });

    group.bench_function("broadcast_routing", |b| {
        b.to_async(&rt).iter(|| async {
            let result = router.route(&broadcast_msg).await.unwrap();
            black_box(result);
        });
    });

    group.finish();
}

/// Benchmark protocol message serialization/deserialization
fn bench_protocol_message_serde(c: &mut Criterion) {
    let mut group = c.benchmark_group("protocol_message_serde");

    // Create sample messages of different sizes
    let small_msg = ProtocolMessage::request("msg-1", LRPRequest::KernelInfoRequest);

    let medium_msg = ProtocolMessage::request(
        "msg-2",
        LRPRequest::ExecuteRequest {
            code: "x = 42\ny = x * 2\nprint(y)".to_string(),
            silent: false,
            store_history: true,
            user_expressions: None,
            allow_stdin: false,
            stop_on_error: true,
        },
    );

    let large_code = "x = 42\n".repeat(1000);
    let large_msg = ProtocolMessage::request(
        "msg-3",
        LRPRequest::ExecuteRequest {
            code: large_code,
            silent: false,
            store_history: true,
            user_expressions: None,
            allow_stdin: false,
            stop_on_error: true,
        },
    );

    // Benchmark serialization
    for (name, msg) in [
        ("small", &small_msg),
        ("medium", &medium_msg),
        ("large", &large_msg),
    ] {
        group.throughput(Throughput::Elements(1));
        group.bench_with_input(BenchmarkId::new("serialize", name), msg, |b, msg| {
            b.iter(|| {
                let json = serde_json::to_string(msg).unwrap();
                black_box(json);
            });
        });
    }

    // Benchmark deserialization
    let small_json = serde_json::to_string(&small_msg).unwrap();
    let medium_json = serde_json::to_string(&medium_msg).unwrap();
    let large_json = serde_json::to_string(&large_msg).unwrap();

    for (name, json) in [
        ("small", &small_json),
        ("medium", &medium_json),
        ("large", &large_json),
    ] {
        group.throughput(Throughput::Bytes(json.len() as u64));
        group.bench_with_input(BenchmarkId::new("deserialize", name), json, |b, json| {
            b.iter(|| {
                let msg: ProtocolMessage = serde_json::from_str(json).unwrap();
                black_box(msg);
            });
        });
    }

    group.finish();
}

/// Benchmark `UniversalMessage` conversion overhead
fn bench_universal_message_conversion(c: &mut Criterion) {
    let mut group = c.benchmark_group("universal_message");

    let lrp_msg = ProtocolMessage::request("msg-1", LRPRequest::KernelInfoRequest);

    // Benchmark conversion from ProtocolMessage to UniversalMessage
    group.bench_function("from_protocol_message", |b| {
        b.iter(|| {
            // This would need the actual conversion implementation
            // For now, we'll create a UniversalMessage directly
            let universal = UniversalMessage {
                id: lrp_msg.msg_id.clone(),
                protocol: ProtocolType::LRP,
                channel: channel_type_from_str(&lrp_msg.channel),
                content: MessageContent::Request {
                    method: "kernel_info_request".to_string(),
                    params: serde_json::json!({}),
                },
                metadata: HashMap::new(),
            };
            black_box(universal);
        });
    });

    // Benchmark conversion from UniversalMessage to ProtocolMessage
    let universal_msg = UniversalMessage {
        id: "msg-1".to_string(),
        protocol: ProtocolType::LRP,
        channel: ChannelType::Shell,
        content: MessageContent::Request {
            method: "kernel_info_request".to_string(),
            params: serde_json::json!({}),
        },
        metadata: HashMap::new(),
    };

    group.bench_function("to_protocol_message", |b| {
        b.iter(|| {
            // This would need the actual conversion implementation
            // For now, we'll create a ProtocolMessage directly
            let protocol = ProtocolMessage {
                msg_id: universal_msg.id.clone(),
                msg_type: MessageType::Request,
                channel: "shell".to_string(),
                content: serde_json::json!({"msg_type": "KernelInfoRequest"}),
            };
            black_box(protocol);
        });
    });

    group.finish();
}

/// Benchmark channel operations
fn bench_channel_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("channel_operations");
    group.measurement_time(Duration::from_secs(10));

    // Simulate channel send/recv with mpsc
    let rt = Runtime::new().unwrap();

    group.bench_function("mpsc_send_recv", |b| {
        b.to_async(&rt).iter(|| async {
            let (tx, mut rx) = tokio::sync::mpsc::channel(100);

            // Send a message
            tx.send("test_message").await.unwrap();

            // Receive the message
            let msg = rx.recv().await.unwrap();
            black_box(msg);
        });
    });

    // Benchmark RwLock overhead (used in MessageRouter)
    let data = std::sync::Arc::new(tokio::sync::RwLock::new(vec![1, 2, 3, 4, 5]));

    group.bench_function("rwlock_read", |b| {
        let data = data.clone();
        b.to_async(&rt).iter(|| async {
            let sum: i32 = data.read().await.iter().sum();
            black_box(sum);
        });
    });

    group.bench_function("rwlock_write", |b| {
        let data = data.clone();
        b.to_async(&rt).iter(|| async {
            let mut guard = data.write().await;
            guard.push(6);
            guard.pop();
            black_box(&*guard);
        });
    });

    group.finish();
}

// Helper for ChannelType conversion
fn channel_type_from_str(s: &str) -> ChannelType {
    match s {
        "iopub" => ChannelType::IOPub,
        "stdin" => ChannelType::Stdin,
        "control" => ChannelType::Control,
        "heartbeat" => ChannelType::Heartbeat,
        _ => ChannelType::Shell,
    }
}

// Create new routers for benchmarks instead of cloning
fn create_router_with_handlers(rt: &Runtime) -> MessageRouter {
    let router = MessageRouter::new();
    rt.block_on(async {
        for i in 1..=10 {
            router
                .register_handler(ProtocolType::LRP, ChannelType::Shell, format!("handler{i}"))
                .await
                .unwrap();
        }
    });
    router
}

criterion_group!(
    benches,
    bench_message_router_strategies,
    bench_protocol_message_serde,
    bench_universal_message_conversion,
    bench_channel_operations
);
criterion_main!(benches);
