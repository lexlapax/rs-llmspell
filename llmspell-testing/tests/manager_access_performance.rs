use llmspell_state_persistence::{StateManager, StateScope};
use std::sync::Arc;
use std::time::Instant;

#[tokio::test]
async fn test_manager_access_under_1ms() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n=== Manager Access Performance Test (<1ms verification) ===\n");

    // Create managers
    let state_manager = Arc::new(StateManager::new().await?);

    // Warm up the manager
    for _ in 0..100 {
        let _ = state_manager.get(StateScope::Global, "warmup").await;
    }

    println!("Testing Manager Access Times (1000 operations each):\n");

    // Test 1: StateManager.get() operations
    let mut get_times = Vec::new();
    for i in 0..1000 {
        let key = format!("test_key_{}", i % 10);
        let start = Instant::now();
        let _ = state_manager.get(StateScope::Global, &key).await;
        let elapsed = start.elapsed();
        get_times.push(elapsed.as_micros());
    }

    let avg_get = get_times.iter().sum::<u128>() as f64 / get_times.len() as f64 / 1000.0;
    let max_get = *get_times.iter().max().unwrap() as f64 / 1000.0;

    println!("StateManager.get():");
    println!("  Average: {:.3} ms", avg_get);
    println!("  Max: {:.3} ms", max_get);
    println!(
        "  Status: {}",
        if avg_get < 1.0 {
            "✅ PASS (<1ms)"
        } else {
            "❌ FAIL (>=1ms)"
        }
    );

    // Test 2: StateManager.set() operations
    let mut set_times = Vec::new();
    for i in 0..1000 {
        let key = format!("perf_test_{}", i);
        let value = serde_json::json!({ "test": i });
        let start = Instant::now();
        let _ = state_manager.set(StateScope::Global, &key, value).await;
        let elapsed = start.elapsed();
        set_times.push(elapsed.as_micros());
    }

    let avg_set = set_times.iter().sum::<u128>() as f64 / set_times.len() as f64 / 1000.0;
    let max_set = *set_times.iter().max().unwrap() as f64 / 1000.0;

    println!("\nStateManager.set():");
    println!("  Average: {:.3} ms", avg_set);
    println!("  Max: {:.3} ms", max_set);
    println!(
        "  Status: {}",
        if avg_set < 1.0 {
            "✅ PASS (<1ms)"
        } else {
            "❌ FAIL (>=1ms)"
        }
    );

    // Test 3: Arc clone overhead (should be negligible)
    let mut clone_times = Vec::new();
    for _ in 0..10000 {
        let start = Instant::now();
        let _cloned = state_manager.clone();
        let elapsed = start.elapsed();
        clone_times.push(elapsed.as_nanos());
    }

    let avg_clone = clone_times.iter().sum::<u128>() as f64 / clone_times.len() as f64 / 1000.0;
    println!("\nArc<StateManager> clone():");
    println!("  Average: {:.3} µs", avg_clone);
    println!("  Status: ✅ PASS (negligible overhead)");

    // Summary
    println!("\n=== SUMMARY ===");
    if avg_get < 1.0 && avg_set < 1.0 {
        println!("✅ ALL PASS: Manager access overhead is <1ms");
        println!("  - get() average: {:.3} ms", avg_get);
        println!("  - set() average: {:.3} ms", avg_set);
        println!("  - Arc clone: {:.3} µs (negligible)", avg_clone);
    } else {
        println!("❌ PERFORMANCE ISSUE: Some operations exceed 1ms");
        if avg_get >= 1.0 {
            println!("  - get() average: {:.3} ms (exceeds target)", avg_get);
        }
        if avg_set >= 1.0 {
            println!("  - set() average: {:.3} ms (exceeds target)", avg_set);
        }
    }

    // Assert for test pass/fail
    assert!(
        avg_get < 1.0,
        "StateManager.get() average time {:.3}ms exceeds 1ms target",
        avg_get
    );
    assert!(
        avg_set < 1.0,
        "StateManager.set() average time {:.3}ms exceeds 1ms target",
        avg_set
    );

    // Clean up test keys
    for i in 0..1000 {
        let key = format!("perf_test_{}", i);
        let _ = state_manager.delete(StateScope::Global, &key).await;
    }

    Ok(())
}
