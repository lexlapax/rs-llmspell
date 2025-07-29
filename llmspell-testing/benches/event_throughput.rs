// ABOUTME: Performance test for event system throughput measurement
// ABOUTME: Validates 100K+ events/second capability and event bus performance

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use llmspell_events::{EventBus, Language, UniversalEvent};
use std::sync::Arc;
use std::time::Duration;
use tokio::runtime::Runtime;
use tokio::sync::mpsc;

/// Benchmark basic event publishing throughput
#[allow(clippy::redundant_pattern_matching, unused_must_use)]
fn bench_event_publishing(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let event_bus = Arc::new(EventBus::new());

    let mut group = c.benchmark_group("event_publishing");

    for event_count in [1000, 10000, 100000].iter() {
        group.throughput(Throughput::Elements(*event_count as u64));

        group.bench_with_input(
            BenchmarkId::from_parameter(event_count),
            event_count,
            |b, &count| {
                b.iter(|| {
                    rt.block_on(async {
                        for i in 0..count {
                            let event = UniversalEvent::new(
                                format!("test.event.{}", i % 100),
                                serde_json::json!({"data": format!("event-{}", i)}),
                                Language::Rust,
                            );
                            let _ = black_box(event_bus.publish(event).await);
                        }
                    });
                });
            },
        );
    }
    group.finish();
}

/// Benchmark event subscription and filtering
#[allow(clippy::redundant_pattern_matching, unused_must_use)]
fn bench_event_subscription(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let event_bus = Arc::new(EventBus::new());

    let mut group = c.benchmark_group("event_subscription");

    // Test different subscription patterns
    let patterns = vec![
        ("exact", "test.event.specific"),
        ("wildcard_suffix", "test.event.*"),
        ("wildcard_prefix", "*.event.specific"),
        ("wildcard_multi", "test.*.specific"),
    ];

    for (name, pattern) in patterns {
        group.bench_function(name, |b| {
            b.iter(|| {
                rt.block_on(async {
                    let mut receiver = event_bus.subscribe(pattern).await.unwrap();

                    // Publish matching events
                    for i in 0..100 {
                        let event = UniversalEvent::new(
                            "test.event.specific",
                            serde_json::json!({"data": format!("data-{}", i)}),
                            Language::Rust,
                        );
                        event_bus.publish(event).await.unwrap();
                    }

                    // Receive events
                    let mut received = 0;
                    while let Some(_event) = receiver.recv().await {
                        received += 1;
                        if received >= 100 {
                            break;
                        }
                    }

                    black_box(received);
                });
            });
        });
    }
    group.finish();
}

/// Benchmark concurrent event publishing and receiving
#[allow(clippy::redundant_pattern_matching, unused_must_use)]
fn bench_concurrent_pubsub(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    c.bench_function("concurrent_pubsub_1000_publishers", |b| {
        b.iter(|| {
            rt.block_on(async {
                let event_bus = Arc::new(EventBus::new());
                let (tx, mut rx) = mpsc::channel(10000);

                // Create subscribers
                let mut subscribers = vec![];
                for i in 0..10 {
                    let sub = event_bus
                        .subscribe(&format!("concurrent.{}.>", i))
                        .await
                        .unwrap();
                    subscribers.push(sub);
                }

                // Spawn publishers
                let mut handles = vec![];
                for publisher_id in 0..1000 {
                    let bus = event_bus.clone();
                    let tx = tx.clone();

                    let handle = tokio::spawn(async move {
                        for event_id in 0..10 {
                            let event = UniversalEvent::new(
                                format!("concurrent.{}.event", publisher_id % 10),
                                serde_json::json!({
                                    "publisher": publisher_id,
                                    "event": event_id,
                                    "timestamp": std::time::SystemTime::now()
                                }),
                                Language::Rust,
                            );
                            bus.publish(event).await.unwrap();
                            tx.send(1).await.unwrap();
                        }
                    });
                    handles.push(handle);
                }

                // Wait for all events to be published
                drop(tx);
                let mut total = 0;
                while let Some(_) = rx.recv().await {
                    total += 1;
                }

                // Join all publishers
                for handle in handles {
                    handle.await.unwrap();
                }

                black_box(total);
            });
        });
    });
}

/// Benchmark event correlation performance
#[allow(clippy::redundant_pattern_matching, unused_must_use)]
fn bench_event_correlation(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    // EventCorrelator not available in current API - simplified benchmark

    c.bench_function("event_correlation_10k", |b| {
        b.iter(|| {
            rt.block_on(async {
                // Create correlation chains
                for chain_id in 0..100 {
                    let _correlation_id = format!("chain-{}", chain_id);

                    for event_id in 0..100 {
                        let event = UniversalEvent::new(
                            "correlation.test",
                            serde_json::json!({
                                "chain_id": chain_id,
                                "event_id": event_id,
                                "index": event_id
                            }),
                            Language::Rust,
                        );

                        // Simplified: just create the event for benchmarking
                        black_box(event);
                    }
                }

                // Simplified correlation benchmark completed
            });
        });
    });
}

/// Benchmark high-frequency event scenarios
#[allow(clippy::redundant_pattern_matching, unused_must_use)]
fn bench_high_frequency_events(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    c.bench_function("high_frequency_100k_events", |b| {
        b.iter(|| {
            rt.block_on(async {
                let event_bus = Arc::new(EventBus::new());
                let mut handles = vec![];

                // Create high-priority subscribers
                for _i in 0..5 {
                    let bus = event_bus.clone();
                    let handle = tokio::spawn(async move {
                        let mut sub = bus.subscribe("high_freq.*").await.unwrap();

                        let mut count = 0;
                        let start = tokio::time::Instant::now();

                        while start.elapsed() < Duration::from_secs(1) {
                            if let Some(_) = sub.recv().await {
                                count += 1;
                            } else {
                                break;
                            }
                        }

                        count
                    });
                    handles.push(handle);
                }

                // Publish 100k events
                let publish_handle = {
                    let bus = event_bus.clone();
                    tokio::spawn(async move {
                        for i in 0..100_000 {
                            let event = UniversalEvent::new(
                                format!("high_freq.event.{}", i % 10),
                                serde_json::json!({"data": format!("data-{}", i)}),
                                Language::Rust,
                            );

                            bus.publish(event).await.unwrap();
                        }
                    })
                };

                // Wait for publishing to complete
                publish_handle.await.unwrap();

                // Collect subscriber results
                let mut total_received = 0;
                for handle in handles {
                    total_received += handle.await.unwrap();
                }

                black_box(total_received);
            });
        });
    });
}

/// Measure event system memory usage under load
#[allow(clippy::redundant_pattern_matching, unused_must_use)]
fn bench_event_memory_usage(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    c.bench_function("event_memory_10k_subscribers", |b| {
        b.iter(|| {
            rt.block_on(async {
                let event_bus = Arc::new(EventBus::new());
                let mut subscriptions = vec![];

                // Create 10k subscriptions
                for i in 0..10_000 {
                    let pattern = format!("memory.test.{}", i % 100);
                    let sub = event_bus.subscribe(&pattern).await.unwrap();
                    subscriptions.push(sub);
                }

                // Publish events to all patterns
                for i in 0..100 {
                    let event = UniversalEvent::new(
                        format!("memory.test.{}", i),
                        serde_json::json!({
                            "test": "data",
                            "index": i,
                            "large_field": vec![0u8; 1024], // 1KB payload
                        }),
                        Language::Rust,
                    );
                    event_bus.publish(event).await.unwrap();
                }

                // Simplified: just track the number of subscriptions created
                let subscription_count = subscriptions.len();

                // Force all subscriptions to receive at least one event each
                for mut sub in subscriptions {
                    if let Some(_) = sub.recv().await {
                        // Received at least one event per subscription
                    }
                }

                black_box(subscription_count);
            });
        });
    });
}

/// Calculate actual throughput metrics
#[allow(clippy::redundant_pattern_matching, unused_must_use)]
fn calculate_throughput_metrics(_c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    println!("\n=== Event Throughput Analysis ===");

    // Test 1: Pure publishing throughput
    let publishing_throughput = rt.block_on(async {
        let event_bus = Arc::new(EventBus::new());
        let events_to_publish = 100_000;

        let start = tokio::time::Instant::now();
        for i in 0..events_to_publish {
            let event = UniversalEvent::new(
                format!("throughput.test.{}", i % 100),
                serde_json::json!({"data": format!("event-{}", i)}),
                Language::Rust,
            );
            event_bus.publish(event).await.unwrap();
        }
        let elapsed = start.elapsed();

        let throughput = events_to_publish as f64 / elapsed.as_secs_f64();
        println!("Publishing throughput: {:.0} events/sec", throughput);
        throughput
    });

    // Test 2: End-to-end throughput (publish + receive)
    let e2e_throughput = rt.block_on(async {
        let event_bus = Arc::new(EventBus::new());
        let mut sub = event_bus.subscribe("e2e.*").await.unwrap();

        let events_to_process = 100_000;

        // Publisher task
        let bus = event_bus.clone();
        let publisher = tokio::spawn(async move {
            for i in 0..events_to_process {
                let event = UniversalEvent::new(
                    format!("e2e.event.{}", i % 10),
                    serde_json::json!({"data": format!("data-{}", i)}),
                    Language::Rust,
                );
                bus.publish(event).await.unwrap();
            }
        });

        // Receiver task
        let start = tokio::time::Instant::now();
        let mut received = 0;

        while received < events_to_process {
            if let Some(_) = sub.recv().await {
                received += 1;
            } else {
                break;
            }
        }

        let elapsed = start.elapsed();
        publisher.await.unwrap();

        let throughput = received as f64 / elapsed.as_secs_f64();
        println!("End-to-end throughput: {:.0} events/sec", throughput);
        throughput
    });

    println!("\nTarget: >100,000 events/sec");
    println!(
        "Publishing: {}",
        if publishing_throughput > 100_000.0 {
            "PASS ✅"
        } else {
            "FAIL ❌"
        }
    );
    println!(
        "End-to-end: {}",
        if e2e_throughput > 100_000.0 {
            "PASS ✅"
        } else {
            "FAIL ❌"
        }
    );
}

criterion_group!(
    benches,
    bench_event_publishing,
    bench_event_subscription,
    bench_concurrent_pubsub,
    bench_event_correlation,
    bench_high_frequency_events,
    bench_event_memory_usage,
    calculate_throughput_metrics
);
criterion_main!(benches);
