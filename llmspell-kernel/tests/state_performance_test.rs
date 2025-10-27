use llmspell_kernel::state::{StateManager, StateScope};
use serde_json::json;
use std::time::Instant;

#[tokio::test]
async fn test_state_performance_targets() {
    // Create state manager with memory backend
    let state_manager = StateManager::new(None).await.unwrap();

    // Test write performance
    let write_start = Instant::now();
    for i in 0..100 {
        state_manager
            .set(
                StateScope::Global,
                &format!("test_key_{}", i),
                json!({"value": i, "data": "test data for performance testing"}),
            )
            .await
            .unwrap();
    }
    let write_duration = write_start.elapsed();
    let avg_write_ms = write_duration.as_micros() as f64 / 100.0 / 1000.0; // Convert to ms

    // Test read performance
    let read_start = Instant::now();
    for i in 0..100 {
        state_manager
            .get(StateScope::Global, &format!("test_key_{}", i))
            .await
            .unwrap();
    }
    let read_duration = read_start.elapsed();
    let avg_read_ms = read_duration.as_micros() as f64 / 100.0 / 1000.0; // Convert to ms

    println!("Performance Test Results:");
    println!("  Average write time: {:.3}ms", avg_write_ms);
    println!("  Average read time: {:.3}ms", avg_read_ms);
    println!("  Target: <5ms write, <1ms read");

    // Assert performance targets are met
    assert!(
        avg_write_ms < 5.0,
        "Write performance target not met: {:.3}ms > 5ms",
        avg_write_ms
    );
    assert!(
        avg_read_ms < 1.0,
        "Read performance target not met: {:.3}ms > 1ms",
        avg_read_ms
    );

    println!("✅ PASS: Performance targets met!");
}
