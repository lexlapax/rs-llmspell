// ABOUTME: Performance test for event system throughput measurement
// ABOUTME: Validates the event system can handle 100K+ events per second

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use llmspell_events::{EventBus, UniversalEvent, Language};
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;
use tokio::runtime::Runtime;

/// Benchmark basic event publishing throughput
fn bench_event_publishing_throughput(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    // Pre-create events to avoid allocation overhead in benchmark
    let events: Vec<UniversalEvent> = (0..1000)
        .map(|i| {
            UniversalEvent::new(
                format!("test.event.{}", i % 10),
                serde_json::json!({
                    "index": i,
                    "data": format!("test-data-{}", i),
                }),
                Language::Rust,
            )
        })
        .collect();

    c.bench_function("event_publishing_1k", |b| {
        b.iter(|| {
            rt.block_on(async {
                let event_bus = EventBus::new();

                for event in &events {
                    let _ = event_bus.publish(event.clone()).await;
                }

                black_box(event_bus)
            })
        });
    });
}

/// Benchmark event publishing with subscribers
fn bench_event_with_subscribers(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    c.bench_function("event_with_10_subscribers", |b| {
        b.iter(|| {
            rt.block_on(async {
                let event_bus = EventBus::new();
                let counter = Arc::new(AtomicU64::new(0));

                // Register 10 subscribers
                for i in 0..10 {
                    let cnt = counter.clone();
                    let pattern = format!("test.event.{}", i);
                    let mut receiver = event_bus.subscribe(&pattern).await.unwrap();
                    
                    // Spawn task to handle events
                    tokio::spawn(async move {
                        while let Some(_event) = receiver.recv().await {
                            cnt.fetch_add(1, Ordering::Relaxed);
                        }
                    });
                }

                // Publish 1000 events
                for i in 0..1000 {
                    let event = UniversalEvent::new(
                        format!("test.event.{}", i % 10),
                        serde_json::json!({ "index": i }),
                        Language::Rust,
                    );
                    let _ = event_bus.publish(event).await;
                }

                // Small delay to ensure processing
                tokio::time::sleep(Duration::from_millis(1)).await;

                let total = counter.load(Ordering::Relaxed);
                black_box(total)
            })
        });
    });
}

/// Benchmark pattern matching performance
fn bench_pattern_matching(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    c.bench_function("event_pattern_matching", |b| {
        b.iter(|| {
            rt.block_on(async {
                let event_bus = EventBus::new();
                let counter = Arc::new(AtomicU64::new(0));

                // Subscribe with wildcard patterns
                let patterns = vec![
                    "test.*",
                    "*.event.*",
                    "test.event.*",
                    "*.specific",
                    "test.event.specific.*",
                ];

                for pattern in patterns {
                    let cnt = counter.clone();
                    let mut receiver = event_bus.subscribe(pattern).await.unwrap();
                    
                    // Spawn task to handle events
                    tokio::spawn(async move {
                        while let Some(_event) = receiver.recv().await {
                            cnt.fetch_add(1, Ordering::Relaxed);
                        }
                    });
                }

                // Publish events with various patterns
                for i in 0..100 {
                    let event_types = vec![
                        format!("test.event.{}", i),
                        format!("other.event.{}", i),
                        format!("test.specific"),
                        format!("test.event.specific.{}", i),
                    ];

                    for event_type in event_types {
                        let event = UniversalEvent::new(
                            event_type,
                            serde_json::json!({ "index": i }),
                            Language::Rust,
                        );
                        let _ = event_bus.publish(event).await;
                    }
                }

                // Small delay to ensure processing
                tokio::time::sleep(Duration::from_millis(1)).await;

                let total = counter.load(Ordering::Relaxed);
                black_box(total)
            })
        });
    });
}

/// Benchmark high-frequency event scenario
fn bench_high_frequency_events(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    c.bench_function("high_frequency_100k_events", |b| {
        b.iter(|| {
            rt.block_on(async {
                let event_bus = EventBus::new();
                let start = tokio::time::Instant::now();

                // Publish 100K events as fast as possible
                for i in 0..100_000 {
                    let event = UniversalEvent::new(
                        format!("high.freq.{}", i % 100),
                        serde_json::json!({ "seq": i }),
                        Language::Rust,
                    );
                    let _ = event_bus.publish(event).await;
                }

                let duration = start.elapsed();
                let events_per_sec = 100_000.0 / duration.as_secs_f64();

                black_box(events_per_sec)
            })
        });
    });
}

/// Calculate and verify event throughput
fn verify_event_throughput(_c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    println!("\n=== Event Throughput Analysis ===");

    rt.block_on(async {
        use tokio::time::Instant;

        let event_bus = EventBus::new();

        // Test 1: Basic publishing throughput
        let start = Instant::now();
        for i in 0..100_000 {
            let event = UniversalEvent::new(
                format!("perf.test.{}", i % 1000),
                serde_json::json!({ "index": i }),
                Language::Rust,
            );
            let _ = event_bus.publish(event).await;
        }
        let duration = start.elapsed();
        let basic_throughput = 100_000.0 / duration.as_secs_f64();

        println!("Basic publishing: {:.0} events/sec", basic_throughput);

        // Test 2: With subscribers
        let event_bus_sub = EventBus::new();
        let counter = Arc::new(AtomicU64::new(0));

        // Add 10 subscribers
        for i in 0..10 {
            let cnt = counter.clone();
            let pattern = format!("perf.sub.{}", i);
            let mut receiver = event_bus_sub.subscribe(&pattern).await.unwrap();
            
            // Spawn task to handle events
            tokio::spawn(async move {
                while let Some(_event) = receiver.recv().await {
                    cnt.fetch_add(1, Ordering::Relaxed);
                }
            });
        }

        let start = Instant::now();
        for i in 0..10_000 {
            let event = UniversalEvent::new(
                format!("perf.sub.{}", i % 10),
                serde_json::json!({ "index": i }),
                Language::Rust,
            );
            let _ = event_bus_sub.publish(event).await;
        }

        // Wait for processing
        tokio::time::sleep(Duration::from_millis(10)).await;

        let duration = start.elapsed();
        let sub_throughput = 10_000.0 / duration.as_secs_f64();
        let processed = counter.load(Ordering::Relaxed);

        println!(
            "With 10 subscribers: {:.0} events/sec ({} processed)",
            sub_throughput, processed
        );

        // Test 3: Pattern matching overhead
        let event_bus_pattern = EventBus::new();
        let pattern_counter = Arc::new(AtomicU64::new(0));

        // Complex pattern subscriptions
        let patterns = vec!["perf.*", "*.pattern.*", "perf.pattern.*"];
        for pattern in patterns {
            let cnt = pattern_counter.clone();
            let mut receiver = event_bus_pattern.subscribe(pattern).await.unwrap();
            
            // Spawn task to handle events
            tokio::spawn(async move {
                while let Some(_event) = receiver.recv().await {
                    cnt.fetch_add(1, Ordering::Relaxed);
                }
            });
        }

        let start = Instant::now();
        for i in 0..10_000 {
            let event = UniversalEvent::new(
                "perf.pattern.test",
                serde_json::json!({ "index": i }),
                Language::Rust,
            );
            let _ = event_bus_pattern.publish(event).await;
        }

        tokio::time::sleep(Duration::from_millis(10)).await;

        let duration = start.elapsed();
        let pattern_throughput = 10_000.0 / duration.as_secs_f64();
        let pattern_processed = pattern_counter.load(Ordering::Relaxed);

        println!(
            "With pattern matching: {:.0} events/sec ({} matches)",
            pattern_throughput, pattern_processed
        );

        println!("\nTarget: >100K events/sec");
        println!(
            "Status: {}",
            if basic_throughput > 100_000.0 {
                "PASS ✅"
            } else {
                "FAIL ❌"
            }
        );
    });
}

criterion_group!(
    benches,
    bench_event_publishing_throughput,
    bench_event_with_subscribers,
    bench_pattern_matching,
    bench_high_frequency_events,
    verify_event_throughput
);
criterion_main!(benches);
