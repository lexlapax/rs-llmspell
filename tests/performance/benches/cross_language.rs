// ABOUTME: Performance test for cross-language bridge overhead
// ABOUTME: Validates performance of Lua↔Rust↔JavaScript event and hook coordination

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use llmspell_bridge::{Bridge, EventBridge, HookBridge, UniversalEventBridge};
use llmspell_core::{
    component::ComponentMetadata, execution::ExecutionContext, tracing::correlation::CorrelationId,
};
use llmspell_events::{Event, EventData, EventPattern, UniversalEvent};
use llmspell_hooks::{HookContext, HookPoint, HookResult, Priority};
use mlua::{Lua, Result as LuaResult};
use std::sync::Arc;
use std::time::Duration;
use tokio::runtime::Runtime;

/// Benchmark Lua to Rust hook execution
fn bench_lua_rust_hook_bridge(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    c.bench_function("lua_rust_hook_overhead", |b| {
        b.iter(|| {
            rt.block_on(async {
                let bridge = Arc::new(Bridge::new());
                let hook_bridge = bridge.hook_bridge();

                // Create Lua environment
                let lua = Lua::new();

                // Register Lua hook via bridge
                let lua_hook_code = r#"
                    function hook_handler(context)
                        -- Simple processing
                        local result = {
                            action = "continue",
                            modified_data = nil
                        }
                        return result
                    end
                    return hook_handler
                "#;

                let hook_fn: mlua::Function = lua.load(lua_hook_code).eval().unwrap();

                // Register the hook through bridge
                hook_bridge
                    .register_lua_hook(
                        HookPoint::BeforeAgentExecution,
                        hook_fn,
                        Priority::Normal,
                        Some("lua-hook".to_string()),
                    )
                    .await
                    .unwrap();

                // Execute hooks 1000 times
                for _ in 0..1000 {
                    let context =
                        HookContext::new(ComponentMetadata::agent("test").id, CorrelationId::new());

                    let result = hook_bridge
                        .execute_hooks(HookPoint::BeforeAgentExecution, context)
                        .await;

                    black_box(result);
                }
            });
        });
    });
}

/// Benchmark cross-language event propagation
fn bench_cross_language_event_propagation(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    c.bench_function("cross_language_event_propagation", |b| {
        b.iter(|| {
            rt.block_on(async {
                let bridge = Arc::new(Bridge::new());
                let event_bridge = bridge.event_bridge();

                // Create Lua environment
                let lua = Lua::new();

                // Set up Lua event subscriber
                let lua_subscriber_code = r#"
                    local events_received = 0
                    
                    function event_handler(event)
                        events_received = events_received + 1
                        -- Process event
                        local event_type = event.event_type or event.type
                        local data = event.data
                        return true
                    end
                    
                    function get_count()
                        return events_received
                    end
                    
                    return {
                        handler = event_handler,
                        get_count = get_count
                    }
                "#;

                let handlers: mlua::Table = lua.load(lua_subscriber_code).eval().unwrap();
                let handler: mlua::Function = handlers.get("handler").unwrap();

                // Subscribe via bridge
                let subscription = event_bridge
                    .subscribe_lua(EventPattern::new("cross_lang.*"), handler)
                    .await
                    .unwrap();

                // Publish events from Rust
                for i in 0..1000 {
                    let event = Event::new(
                        format!("cross_lang.test.{}", i % 10),
                        EventData::json(serde_json::json!({
                            "index": i,
                            "source": "rust",
                            "timestamp": chrono::Utc::now(),
                        })),
                    );

                    event_bridge.publish(event).await.unwrap();
                }

                // Small delay to ensure all events are processed
                tokio::time::sleep(Duration::from_millis(10)).await;

                // Get count from Lua
                let get_count: mlua::Function = handlers.get("get_count").unwrap();
                let count: i32 = get_count.call(()).unwrap();

                black_box(count);
            });
        });
    });
}

/// Benchmark UniversalEvent serialization/deserialization overhead
fn bench_universal_event_serialization(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    c.bench_function("universal_event_serialization", |b| {
        b.iter(|| {
            rt.block_on(async {
                let bridge = UniversalEventBridge::new();

                // Create complex events
                let mut events = vec![];
                for i in 0..100 {
                    let event = UniversalEvent {
                        id: format!("event-{}", i),
                        event_type: format!("benchmark.type.{}", i % 5),
                        source: "rust-benchmark".to_string(),
                        timestamp: chrono::Utc::now(),
                        correlation_id: Some(format!("correlation-{}", i / 10)),
                        causation_id: if i > 0 {
                            Some(format!("event-{}", i - 1))
                        } else {
                            None
                        },
                        data: serde_json::json!({
                            "index": i,
                            "nested": {
                                "field1": "value1",
                                "field2": i * 2,
                                "array": vec![1, 2, 3, 4, 5],
                            },
                            "large_string": "x".repeat(1000),
                        }),
                        metadata: std::collections::HashMap::from([
                            ("language".to_string(), "rust".to_string()),
                            ("version".to_string(), "1.0".to_string()),
                        ]),
                    };
                    events.push(event);
                }

                // Serialize to cross-language format
                for event in &events {
                    let serialized = bridge.serialize_event(event).unwrap();
                    black_box(serialized);
                }

                // Deserialize back
                for event in &events {
                    let serialized = bridge.serialize_event(event).unwrap();
                    let deserialized: UniversalEvent =
                        bridge.deserialize_event(&serialized).unwrap();
                    black_box(deserialized);
                }
            });
        });
    });
}

/// Benchmark JavaScript bridge overhead (simulated)
fn bench_javascript_bridge_simulation(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    c.bench_function("javascript_bridge_simulation", |b| {
        b.iter(|| {
            rt.block_on(async {
                // Simulate JavaScript bridge with JSON serialization
                let bridge = Arc::new(Bridge::new());
                let event_bridge = bridge.event_bridge();

                // Simulate JS event handler
                let js_handler = Arc::new(tokio::sync::Mutex::new(Vec::new()));
                let handler = js_handler.clone();

                // Subscribe with handler that simulates JS processing
                let subscription = event_bridge
                    .subscribe_with_handler(
                        EventPattern::new("js.bridge.*"),
                        Box::new(move |event: Event| {
                            let handler = handler.clone();
                            Box::pin(async move {
                                // Simulate JSON serialization (JS boundary)
                                let json = serde_json::to_string(&event).unwrap();

                                // Simulate JS processing
                                let processed = json.len() > 0;

                                // Store result
                                let mut results = handler.lock().await;
                                results.push(processed);

                                Ok(())
                            })
                        }),
                    )
                    .await
                    .unwrap();

                // Send events through bridge
                for i in 0..1000 {
                    let event = Event::new(
                        format!("js.bridge.event.{}", i),
                        EventData::json(serde_json::json!({
                            "id": i,
                            "data": format!("test-data-{}", i),
                        })),
                    );

                    event_bridge.publish(event).await.unwrap();
                }

                // Wait for processing
                tokio::time::sleep(Duration::from_millis(50)).await;

                let results = js_handler.lock().await;
                black_box(results.len());
            });
        });
    });
}

/// Benchmark multi-language coordination scenario
fn bench_multi_language_coordination(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    c.bench_function("multi_language_coordination", |b| {
        b.iter(|| {
            rt.block_on(async {
                let bridge = Arc::new(Bridge::new());
                let hook_bridge = bridge.hook_bridge();
                let event_bridge = bridge.event_bridge();

                // Set up Lua environment
                let lua = Lua::new();

                // Lua hook that publishes events
                let lua_hook_code = r#"
                    local event_bridge = ...
                    
                    function hook_handler(context)
                        -- Publish event from hook
                        event_bridge:publish({
                            type = "lua.hook.triggered",
                            data = {
                                hook_point = context.hook_point,
                                component_id = context.component_id,
                                timestamp = os.time()
                            }
                        })
                        
                        return { action = "continue" }
                    end
                    
                    return hook_handler
                "#;

                // Pass event bridge to Lua
                lua.globals()
                    .set("event_bridge", event_bridge.clone())
                    .unwrap();
                let hook_fn: mlua::Function = lua.load(lua_hook_code).eval().unwrap();

                // Register Lua hook
                hook_bridge
                    .register_lua_hook(
                        HookPoint::BeforeToolExecution,
                        hook_fn,
                        Priority::High,
                        Some("coordination-hook".to_string()),
                    )
                    .await
                    .unwrap();

                // Rust event subscriber counting Lua events
                let counter = Arc::new(tokio::sync::atomic::AtomicU64::new(0));
                let cnt = counter.clone();

                event_bridge
                    .subscribe_with_handler(
                        EventPattern::new("lua.hook.triggered"),
                        Box::new(move |_event| {
                            cnt.fetch_add(1, tokio::sync::atomic::Ordering::Relaxed);
                            Box::pin(async move { Ok(()) })
                        }),
                    )
                    .await
                    .unwrap();

                // Trigger hooks which will publish events
                for _ in 0..100 {
                    let context = HookContext::new(
                        ComponentMetadata::tool("test-tool").id,
                        CorrelationId::new(),
                    );

                    hook_bridge
                        .execute_hooks(HookPoint::BeforeToolExecution, context)
                        .await
                        .unwrap();
                }

                // Small delay for event processing
                tokio::time::sleep(Duration::from_millis(10)).await;

                let total = counter.load(tokio::sync::atomic::Ordering::Relaxed);
                black_box(total);
            });
        });
    });
}

/// Calculate cross-language overhead metrics
fn calculate_cross_language_overhead(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    println!("\n=== Cross-Language Bridge Overhead Analysis ===");

    rt.block_on(async {
        // Test 1: Native Rust hook execution baseline
        let native_baseline = {
            let start = tokio::time::Instant::now();
            let hook_system = llmspell_hooks::HookSystem::new();

            hook_system
                .register_hook(
                    HookPoint::BeforeAgentExecution,
                    Box::new(|_ctx| Box::pin(async move { HookResult::Continue })),
                    Priority::Normal,
                    None,
                )
                .await
                .unwrap();

            for _ in 0..10000 {
                let context =
                    HookContext::new(ComponentMetadata::agent("test").id, CorrelationId::new());
                hook_system
                    .execute_hooks(HookPoint::BeforeAgentExecution, context)
                    .await
                    .unwrap();
            }

            start.elapsed()
        };

        // Test 2: Lua bridge overhead
        let lua_bridge_time = {
            let start = tokio::time::Instant::now();
            let bridge = Bridge::new();
            let hook_bridge = bridge.hook_bridge();
            let lua = Lua::new();

            let hook_fn: mlua::Function = lua
                .load(r#"function(ctx) return {action="continue"} end"#)
                .eval()
                .unwrap();

            hook_bridge
                .register_lua_hook(
                    HookPoint::BeforeAgentExecution,
                    hook_fn,
                    Priority::Normal,
                    None,
                )
                .await
                .unwrap();

            for _ in 0..10000 {
                let context =
                    HookContext::new(ComponentMetadata::agent("test").id, CorrelationId::new());
                hook_bridge
                    .execute_hooks(HookPoint::BeforeAgentExecution, context)
                    .await
                    .unwrap();
            }

            start.elapsed()
        };

        let lua_overhead_ms =
            (lua_bridge_time.as_secs_f64() - native_baseline.as_secs_f64()) * 1000.0;
        let lua_overhead_percent =
            (lua_overhead_ms / (native_baseline.as_secs_f64() * 1000.0)) * 100.0;

        println!("Native Rust baseline: {:?}", native_baseline);
        println!("Lua bridge time: {:?}", lua_bridge_time);
        println!(
            "Lua bridge overhead: {:.2}ms ({:.2}%)",
            lua_overhead_ms, lua_overhead_percent
        );

        // Test 3: Event serialization overhead
        let serialization_overhead = {
            let bridge = UniversalEventBridge::new();
            let event = UniversalEvent {
                id: "test-1".to_string(),
                event_type: "benchmark.test".to_string(),
                source: "rust".to_string(),
                timestamp: chrono::Utc::now(),
                correlation_id: Some("corr-1".to_string()),
                causation_id: None,
                data: serde_json::json!({"test": "data", "nested": {"field": "value"}}),
                metadata: Default::default(),
            };

            let start = tokio::time::Instant::now();
            for _ in 0..10000 {
                let serialized = bridge.serialize_event(&event).unwrap();
                let _deserialized: UniversalEvent = bridge.deserialize_event(&serialized).unwrap();
            }
            start.elapsed()
        };

        println!(
            "\nEvent serialization (10k events): {:?}",
            serialization_overhead
        );
        println!(
            "Per-event overhead: {:.3}μs",
            serialization_overhead.as_micros() as f64 / 10000.0
        );

        println!("\nTarget: <10% overhead for cross-language operations");
        println!(
            "Status: {}",
            if lua_overhead_percent < 10.0 {
                "PASS ✅"
            } else {
                "FAIL ❌"
            }
        );
    });
}

criterion_group!(
    benches,
    bench_lua_rust_hook_bridge,
    bench_cross_language_event_propagation,
    bench_universal_event_serialization,
    bench_javascript_bridge_simulation,
    bench_multi_language_coordination,
    calculate_cross_language_overhead
);
criterion_main!(benches);
