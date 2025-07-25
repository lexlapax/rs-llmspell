// ABOUTME: Performance test for event system throughput measurement
// ABOUTME: Validates 100K+ events/second capability and event bus performance

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use llmspell_events::{
    bus::EventBus, correlation::EventCorrelator, Event, EventData, EventFilter, EventPattern,
    EventPriority, EventSubscription, UniversalEvent,
};
use std::sync::Arc;
use std::time::Duration;
use tokio::runtime::Runtime;
use tokio::sync::mpsc;

/// Benchmark basic event publishing throughput
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
                            let event = Event::new(
                                format!("test.event.{}", i % 100),
                                EventData::simple(format!("event-{}", i)),
                            );
                            black_box(event_bus.publish(event).await);
                        }
                    });
                });
            },
        );
    }
    group.finish();
}

/// Benchmark event subscription and filtering
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
                    let subscription = event_bus
                        .subscribe(EventPattern::new(pattern))
                        .await
                        .unwrap();

                    // Publish matching events
                    for i in 0..100 {
                        let event = Event::new(
                            "test.event.specific",
                            EventData::simple(format!("data-{}", i)),
                        );
                        event_bus.publish(event).await.unwrap();
                    }

                    // Receive events
                    let mut received = 0;
                    while let Ok(Some(_)) = subscription.try_receive().await {
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
                        .subscribe(EventPattern::new(&format!("concurrent.{}.>", i)))
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
                            let event = Event::new(
                                format!("concurrent.{}.event", publisher_id % 10),
                                EventData::json(serde_json::json!({
                                    "publisher": publisher_id,
                                    "event": event_id,
                                    "timestamp": std::time::SystemTime::now()
                                })),
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
fn bench_event_correlation(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let correlator = Arc::new(EventCorrelator::new());

    c.bench_function("event_correlation_10k", |b| {
        b.iter(|| {
            rt.block_on(async {
                // Create correlation chains
                for chain_id in 0..100 {
                    let correlation_id = format!("chain-{}", chain_id);

                    for event_id in 0..100 {
                        let event = UniversalEvent {
                            id: format!("event-{}-{}", chain_id, event_id),
                            event_type: "correlation.test".to_string(),
                            source: "benchmark".to_string(),
                            timestamp: chrono::Utc::now(),
                            correlation_id: Some(correlation_id.clone()),
                            causation_id: if event_id > 0 {
                                Some(format!("event-{}-{}", chain_id, event_id - 1))
                            } else {
                                None
                            },
                            data: serde_json::json!({"index": event_id}),
                            metadata: Default::default(),
                        };

                        correlator.track_event(event).await;
                    }
                }

                // Query correlation chains
                for chain_id in 0..100 {
                    let chain = correlator
                        .get_correlation_chain(&format!("chain-{}", chain_id))
                        .await;
                    black_box(chain);
                }
            });
        });
    });
}

/// Benchmark high-frequency event scenarios
fn bench_high_frequency_events(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    c.bench_function("high_frequency_100k_events", |b| {
        b.iter(|| {
            rt.block_on(async {
                let event_bus = Arc::new(EventBus::new());
                let mut handles = vec![];

                // Create high-priority subscribers
                for i in 0..5 {
                    let bus = event_bus.clone();
                    let handle = tokio::spawn(async move {
                        let sub = bus
                            .subscribe_with_filter(
                                EventPattern::new("high_freq.*"),
                                EventFilter::builder().priority(EventPriority::High).build(),
                            )
                            .await
                            .unwrap();

                        let mut count = 0;
                        let start = tokio::time::Instant::now();

                        while start.elapsed() < Duration::from_secs(1) {
                            if let Ok(Some(_)) = sub.try_receive().await {
                                count += 1;
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
                            let event = Event::builder()
                                .event_type(format!("high_freq.event.{}", i % 10))
                                .data(EventData::simple(format!("data-{}", i)))
                                .priority(if i % 100 == 0 {
                                    EventPriority::High
                                } else {
                                    EventPriority::Normal
                                })
                                .build();

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
                    let sub = event_bus
                        .subscribe(EventPattern::new(&pattern))
                        .await
                        .unwrap();
                    subscriptions.push(sub);
                }

                // Publish events to all patterns
                for i in 0..100 {
                    let event = Event::new(
                        format!("memory.test.{}", i),
                        EventData::json(serde_json::json!({
                            "test": "data",
                            "index": i,
                            "large_field": vec![0u8; 1024], // 1KB payload
                        })),
                    );
                    event_bus.publish(event).await.unwrap();
                }

                // Force all subscriptions to receive
                for sub in &subscriptions {
                    while let Ok(Some(_)) = sub.try_receive().await {
                        // Drain events
                    }
                }

                black_box(subscriptions.len());
            });
        });
    });
}

/// Calculate actual throughput metrics
fn calculate_throughput_metrics(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    println!("\n=== Event Throughput Analysis ===");

    // Test 1: Pure publishing throughput
    let publishing_throughput = rt.block_on(async {
        let event_bus = Arc::new(EventBus::new());
        let events_to_publish = 100_000;

        let start = tokio::time::Instant::now();
        for i in 0..events_to_publish {
            let event = Event::new(
                format!("throughput.test.{}", i % 100),
                EventData::simple(format!("event-{}", i)),
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
        let sub = event_bus
            .subscribe(EventPattern::new("e2e.*"))
            .await
            .unwrap();

        let events_to_process = 100_000;

        // Publisher task
        let bus = event_bus.clone();
        let publisher = tokio::spawn(async move {
            for i in 0..events_to_process {
                let event = Event::new(
                    format!("e2e.event.{}", i % 10),
                    EventData::simple(format!("data-{}", i)),
                );
                bus.publish(event).await.unwrap();
            }
        });

        // Receiver task
        let start = tokio::time::Instant::now();
        let mut received = 0;

        while received < events_to_process {
            if let Ok(Some(_)) = sub.receive_timeout(Duration::from_millis(100)).await {
                received += 1;
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
