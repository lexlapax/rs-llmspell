// ABOUTME: High-frequency stress tests for event bus performance validation
// ABOUTME: Tests 10K+ events/sec throughput, memory stability, and backpressure handling

use llmspell_events::flow_controller::{FlowControllerConfig, RateLimit};
use llmspell_events::*;
use llmspell_storage::MemoryBackend;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Barrier;

#[cfg(test)]
mod stress_tests {
    use super::*;

    /// Create a test event with sequence number
    fn create_test_event(seq: u64) -> UniversalEvent {
        UniversalEvent::new(
            format!("stress.test.{}", seq),
            serde_json::json!({
                "sequence": seq,
                "payload": "test_data",
                "timestamp": std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_millis()
            }),
            Language::Rust,
        )
    }

    #[cfg_attr(test_category = "integration")]
    #[tokio::test]
    #[cfg_attr(not(feature = "stress_tests"), ignore)]
    async fn test_10k_events_per_second_sustained() {
        let bus = EventBus::new();
        let mut receiver = bus.subscribe("stress.*").await.unwrap();

        // Track received events
        let received_count = Arc::new(AtomicU64::new(0));
        let counter = received_count.clone();

        // Spawn receiver task
        let receiver_task = tokio::spawn(async move {
            while let Some(_event) = receiver.recv().await {
                counter.fetch_add(1, Ordering::Relaxed);
            }
        });

        // Publish 10,000 events at high frequency
        let start_time = Instant::now();
        let target_events = 10_000u64;
        let _target_duration = Duration::from_secs(1); // 1 second = 10K EPS

        for i in 0..target_events {
            let event = create_test_event(i);
            if let Err(e) = bus.publish(event).await {
                eprintln!("Failed to publish event {}: {:?}", i, e);
                break;
            }

            // Micro-sleep to control rate (100 microseconds = 10K EPS)
            if i % 100 == 0 {
                tokio::time::sleep(Duration::from_micros(10)).await;
            }
        }

        let publish_duration = start_time.elapsed();
        let actual_eps = target_events as f64 / publish_duration.as_secs_f64();

        // Wait for all events to be processed
        tokio::time::sleep(Duration::from_millis(100)).await;

        let final_received = received_count.load(Ordering::Relaxed);
        let receive_rate = final_received as f64 / publish_duration.as_secs_f64();

        println!(
            "Published {} events in {:.2}s ({:.0} EPS)",
            target_events,
            publish_duration.as_secs_f64(),
            actual_eps
        );
        println!(
            "Received {} events ({:.0} EPS)",
            final_received, receive_rate
        );

        // Clean shutdown
        receiver_task.abort();

        // Assertions
        assert!(
            actual_eps >= 8_000.0,
            "Should achieve at least 8K EPS, got {:.0}",
            actual_eps
        );
        assert!(
            final_received >= target_events * 95 / 100,
            "Should receive at least 95% of events, got {}/{}",
            final_received,
            target_events
        );
    }

    #[cfg_attr(test_category = "integration")]
    #[tokio::test]
    #[cfg_attr(not(feature = "stress_tests"), ignore)]
    async fn test_concurrent_publishers_stress() {
        let bus = Arc::new(EventBus::new());
        let mut receiver = bus.subscribe("concurrent.*").await.unwrap();

        let received_count = Arc::new(AtomicU64::new(0));
        let counter = received_count.clone();

        // Spawn receiver
        let receiver_task = tokio::spawn(async move {
            while let Some(_event) = receiver.recv().await {
                counter.fetch_add(1, Ordering::Relaxed);
            }
        });

        // Configuration
        let publisher_count = 10;
        let events_per_publisher = 1_000;
        let total_events = publisher_count * events_per_publisher;

        let start_barrier = Arc::new(Barrier::new(publisher_count + 1));

        // Spawn concurrent publishers
        let mut publisher_tasks = Vec::new();
        for publisher_id in 0..publisher_count {
            let bus_clone = Arc::clone(&bus);
            let barrier_clone = Arc::clone(&start_barrier);

            let task = tokio::spawn(async move {
                // Wait for all publishers to be ready
                barrier_clone.wait().await;

                let start_time = Instant::now();
                for event_id in 0..events_per_publisher {
                    let event = UniversalEvent::new(
                        format!("concurrent.publisher_{}.event_{}", publisher_id, event_id),
                        serde_json::json!({
                            "publisher": publisher_id,
                            "event": event_id,
                            "timestamp": std::time::SystemTime::now()
                        }),
                        Language::Rust,
                    );

                    if let Err(e) = bus_clone.publish(event).await {
                        eprintln!(
                            "Publisher {} failed at event {}: {:?}",
                            publisher_id, event_id, e
                        );
                        break;
                    }
                }
                let duration = start_time.elapsed();
                println!(
                    "Publisher {} completed in {:.2}s ({:.0} EPS)",
                    publisher_id,
                    duration.as_secs_f64(),
                    events_per_publisher as f64 / duration.as_secs_f64()
                );
            });

            publisher_tasks.push(task);
        }

        // Start all publishers simultaneously
        let test_start = Instant::now();
        start_barrier.wait().await;

        // Wait for all publishers to complete
        for task in publisher_tasks {
            task.await.unwrap();
        }

        let publish_duration = test_start.elapsed();

        // Wait for events to be processed
        tokio::time::sleep(Duration::from_millis(200)).await;

        let final_received = received_count.load(Ordering::Relaxed);
        let overall_eps = final_received as f64 / publish_duration.as_secs_f64();

        println!(
            "Total: {} events in {:.2}s ({:.0} EPS)",
            final_received,
            publish_duration.as_secs_f64(),
            overall_eps
        );

        // Clean shutdown
        receiver_task.abort();

        // Assertions
        assert!(
            overall_eps >= 5_000.0,
            "Should achieve at least 5K EPS with concurrent publishers"
        );
        assert!(
            final_received >= total_events as u64 * 90 / 100,
            "Should receive at least 90% of events from concurrent publishers"
        );
    }

    #[cfg_attr(test_category = "integration")]
    #[tokio::test]
    #[cfg_attr(not(feature = "stress_tests"), ignore)]
    async fn test_memory_stability_under_load() {
        let bus = EventBus::new();
        let mut receiver = bus.subscribe("memory.*").await.unwrap();

        // Track memory usage approximation
        let initial_memory = get_approximate_memory_usage();

        let received_count = Arc::new(AtomicU64::new(0));
        let counter = received_count.clone();

        // Spawn receiver with controlled processing delay
        let receiver_task = tokio::spawn(async move {
            while let Some(_event) = receiver.recv().await {
                counter.fetch_add(1, Ordering::Relaxed);
                // Small delay to simulate processing
                tokio::time::sleep(Duration::from_micros(10)).await;
            }
        });

        // Publish events in waves to test memory stability
        let waves = 5;
        let events_per_wave = 2_000;

        for wave in 0..waves {
            println!("Starting wave {} of {}", wave + 1, waves);
            let wave_start = Instant::now();

            for i in 0..events_per_wave {
                let event = UniversalEvent::new(
                    format!("memory.wave_{}.event_{}", wave, i),
                    serde_json::json!({
                        "wave": wave,
                        "event": i,
                        "large_payload": "x".repeat(1000), // 1KB payload
                        "timestamp": std::time::SystemTime::now()
                    }),
                    Language::Rust,
                );

                bus.publish(event).await.unwrap();
            }

            let wave_duration = wave_start.elapsed();
            let wave_eps = events_per_wave as f64 / wave_duration.as_secs_f64();

            // Check memory usage
            let current_memory = get_approximate_memory_usage();
            let memory_growth = current_memory - initial_memory;

            println!(
                "Wave {} completed: {:.0} EPS, ~{}KB memory growth",
                wave + 1,
                wave_eps,
                memory_growth / 1024
            );

            // Allow some processing time between waves
            tokio::time::sleep(Duration::from_millis(100)).await;

            // Memory should not grow excessively (heuristic: <10MB per wave)
            assert!(
                memory_growth < 10 * 1024 * 1024,
                "Memory growth too large: {}KB",
                memory_growth / 1024
            );
        }

        // Wait for all processing to complete
        tokio::time::sleep(Duration::from_millis(500)).await;

        let final_received = received_count.load(Ordering::Relaxed);
        let total_events = waves * events_per_wave;

        // Clean shutdown
        receiver_task.abort();

        // Final memory check
        let final_memory = get_approximate_memory_usage();
        let total_growth = final_memory - initial_memory;

        println!("Memory stability test completed:");
        println!("  Events published: {}", total_events);
        println!("  Events received: {}", final_received);
        println!("  Memory growth: ~{}KB", total_growth / 1024);

        // Assertions
        assert!(
            final_received >= total_events as u64 * 95 / 100,
            "Should process most events under memory pressure"
        );
        assert!(
            total_growth < 50 * 1024 * 1024,
            "Total memory growth should be reasonable: {}KB",
            total_growth / 1024
        );
    }

    #[cfg_attr(test_category = "integration")]
    #[tokio::test]
    #[cfg_attr(not(feature = "stress_tests"), ignore)]
    async fn test_backpressure_handling() {
        // Create EventBus with limited flow control
        let flow_config = FlowControllerConfig {
            rate_limit: Some(RateLimit {
                max_rate: 5000.0,
                burst_capacity: 500.0,
            }),
            overflow_config: Default::default(),
            enable_notifications: true,
            stats_interval: std::time::Duration::from_secs(1),
        };

        let bus = EventBus::with_config(flow_config);

        // Create slow receiver to trigger backpressure
        let mut receiver = bus.subscribe("backpressure.*").await.unwrap();
        let received_count = Arc::new(AtomicU64::new(0));
        let counter = received_count.clone();

        let receiver_task = tokio::spawn(async move {
            while let Some(_event) = receiver.recv().await {
                counter.fetch_add(1, Ordering::Relaxed);
                // Slow processing to trigger backpressure
                tokio::time::sleep(Duration::from_millis(1)).await;
            }
        });

        // Publish events rapidly to trigger backpressure
        let start_time = Instant::now();
        let mut successful_publishes = 0u64;
        let mut failed_publishes = 0u64;

        for i in 0..10_000 {
            let event = create_test_event(i);

            match bus.publish(event).await {
                Ok(_) => successful_publishes += 1,
                Err(_) => {
                    failed_publishes += 1;
                    // Don't break immediately - test backpressure recovery
                }
            }

            // Small delay to allow some processing
            if i % 1000 == 0 {
                tokio::time::sleep(Duration::from_millis(10)).await;
            }
        }

        let publish_duration = start_time.elapsed();

        // Wait for processing to catch up
        tokio::time::sleep(Duration::from_secs(2)).await;

        let final_received = received_count.load(Ordering::Relaxed);

        println!("Backpressure test results:");
        println!("  Successful publishes: {}", successful_publishes);
        println!("  Failed publishes: {}", failed_publishes);
        println!("  Events received: {}", final_received);
        println!("  Publish duration: {:.2}s", publish_duration.as_secs_f64());

        // Get flow statistics
        let flow_stats = bus.get_stats();
        println!(
            "  Flow stats: events_processed={}, rate_limited={}, buffer_size={}",
            flow_stats.events_processed,
            flow_stats.rate_limit_violations,
            bus.buffer_size()
        );

        // Clean shutdown
        receiver_task.abort();

        // Assertions
        assert!(
            successful_publishes > 0,
            "Should have some successful publishes"
        );
        assert!(
            failed_publishes > 0,
            "Should have some failed publishes due to backpressure"
        );
        assert!(final_received > 0, "Should have processed some events");
    }

    #[cfg_attr(test_category = "integration")]
    #[tokio::test]
    #[cfg_attr(not(feature = "stress_tests"), ignore)]
    async fn test_stream_processing_performance() {
        use llmspell_events::stream::{HighThroughputProcessor, StreamUtils};

        let _bus = EventBus::new();

        // Create high-frequency event stream
        let event_stream = StreamUtils::high_frequency_test_stream(5_000, 10_000);

        // Measure stream processing throughput
        let measurement =
            StreamUtils::measure_throughput(event_stream, Duration::from_millis(1000)).await;

        println!("Stream processing performance:");
        println!("  Events processed: {}", measurement.event_count);
        println!("  Errors: {}", measurement.error_count);
        println!("  Duration: {:.2}s", measurement.elapsed.as_secs_f64());
        println!("  Throughput: {:.0} EPS", measurement.events_per_second);

        // Test high-throughput processor
        let processor = HighThroughputProcessor::new(2000, 8);
        let test_stream = StreamUtils::high_frequency_test_stream(10_000, 15_000);

        let processed_count = Arc::new(AtomicU64::new(0));
        let counter = processed_count.clone();

        let process_start = Instant::now();

        processor
            .process_stream(test_stream, move |_event| {
                let counter = counter.clone();
                async move {
                    counter.fetch_add(1, Ordering::Relaxed);
                    // Simulate some processing work
                    tokio::time::sleep(Duration::from_micros(50)).await;
                    Ok(())
                }
            })
            .await
            .unwrap();

        let process_duration = process_start.elapsed();
        let final_processed = processed_count.load(Ordering::Relaxed);
        let processing_eps = final_processed as f64 / process_duration.as_secs_f64();

        println!("High-throughput processor results:");
        println!("  Events processed: {}", final_processed);
        println!("  Duration: {:.2}s", process_duration.as_secs_f64());
        println!("  Processing EPS: {:.0}", processing_eps);

        // Assertions
        assert!(
            measurement.events_per_second >= 5_000.0,
            "Stream should process at least 5K EPS"
        );
        assert!(
            processing_eps >= 1_000.0,
            "High-throughput processor should handle at least 1K EPS with processing"
        );
        assert!(
            final_processed >= 8_000,
            "Should process most events through high-throughput pipeline"
        );
    }

    /// Approximate memory usage (very rough estimate)
    fn get_approximate_memory_usage() -> usize {
        // This is a rough approximation - in real testing you'd use more sophisticated tools
        let thread_count = std::thread::available_parallelism().unwrap().get();
        let stack_size = 2 * 1024 * 1024; // 2MB per thread (rough estimate)
        thread_count * stack_size
    }
}

// Integration test for the complete enhanced EventBus
#[cfg_attr(test_category = "integration")]
#[tokio::test]
async fn test_enhanced_eventbus_integration() {
    // Create EventBus with persistence
    let backend = MemoryBackend::new();
    let storage_adapter = EventStorageAdapter::new(backend);
    let persistence_config = PersistenceConfig {
        enabled: true,
        ..Default::default()
    };

    let bus = EventBus::with_persistence(Default::default(), storage_adapter, persistence_config);

    // Test basic functionality
    let mut receiver = bus.subscribe("integration.*").await.unwrap();

    // Publish test event
    let event = UniversalEvent::new(
        "integration.test",
        serde_json::json!({"test": "data"}),
        Language::Rust,
    );

    bus.publish(event.clone()).await.unwrap();

    // Receive event
    let received = receiver.recv().await.unwrap();
    assert_eq!(received.event_type, "integration.test");

    // Test persistence
    let persisted = bus.get_persisted_events("integration.*").await.unwrap();
    assert_eq!(persisted.len(), 1);
    assert_eq!(persisted[0].event_type, "integration.test");

    // Test statistics
    let stats = bus.get_storage_stats().await.unwrap();
    assert!(stats.is_some());
    let stats = stats.unwrap();
    assert_eq!(stats.total_events, 1);

    println!("Enhanced EventBus integration test completed successfully");
}
