use llmspell_state_persistence::{StateClass, StateManager, StateScope};
use serde_json::json;
use std::time::Instant;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Testing Phase 1: StateClass System & Fast Paths");
    println!("================================================");

    // Test data
    let test_data = json!({
        "conversation": ["Hello", "Hi there!"],
        "context": {"topic": "greeting"}
    });
    let scope = StateScope::Agent("benchmark:test-agent".to_string());

    // Baseline: Direct memory operations
    println!("\n1. Baseline (direct memory operations):");
    let start = Instant::now();
    for _ in 0..1000 {
        let _ = test_data.clone();
    }
    let baseline = start.elapsed();
    println!("   Time: {:?}", baseline);

    // Old path: Standard StateManager
    println!("\n2. Standard StateManager (old path):");
    let state_manager = StateManager::new().await?;
    let start = Instant::now();
    for i in 0..1000 {
        let key = format!("key-{}", i);
        state_manager.set(scope.clone(), &key, test_data.clone()).await?;
    }
    let standard_time = start.elapsed();
    println!("   Time: {:?}", standard_time);
    
    let standard_overhead = ((standard_time.as_nanos() as f64 / baseline.as_nanos() as f64) - 1.0) * 100.0;
    println!("   Overhead: {:.2}%", standard_overhead);

    // New path: Fast-path with Trusted class
    println!("\n3. Fast-path StateManager (new path):");
    let fast_manager = StateManager::new_benchmark().await?;
    let start = Instant::now();
    for i in 0..1000 {
        let key = format!("benchmark:key-{}", i);
        fast_manager.set_with_class(scope.clone(), &key, test_data.clone(), Some(StateClass::Trusted)).await?;
    }
    let fast_time = start.elapsed();
    println!("   Time: {:?}", fast_time);
    
    let fast_overhead = ((fast_time.as_nanos() as f64 / baseline.as_nanos() as f64) - 1.0) * 100.0;
    println!("   Overhead: {:.2}%", fast_overhead);

    // Results
    println!("\n4. Results Summary:");
    println!("   Target: <5% overhead");
    println!("   Standard path: {:.2}% {}", standard_overhead, if standard_overhead < 5.0 { "✅ PASS" } else { "❌ FAIL" });
    println!("   Fast path: {:.2}% {}", fast_overhead, if fast_overhead < 5.0 { "✅ PASS" } else { "❌ FAIL" });
    
    let improvement = ((standard_time.as_nanos() as f64 / fast_time.as_nanos() as f64) - 1.0) * 100.0;
    println!("   Improvement: {:.1}% faster", improvement);

    // Test state class inference
    println!("\n5. State Class Inference:");
    println!("   'benchmark:test' -> {:?}", StateClass::infer_from_key("benchmark:test"));
    println!("   'secret:token' -> {:?}", StateClass::infer_from_key("secret:token"));
    println!("   'temp:cache' -> {:?}", StateClass::infer_from_key("temp:cache"));
    println!("   'normal:data' -> {:?}", StateClass::infer_from_key("normal:data"));

    Ok(())
}