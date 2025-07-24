//! Direct performance test to understand hook overhead

use std::time::Instant;
use tokio;

// Basic operation without any hooks
async fn basic_operation() {
    // Simulate basic agent state transition work
    tokio::task::yield_now().await;
}

// Operation with minimal hook-like overhead
async fn operation_with_minimal_hooks() {
    // Simulate hook registry lookup
    let _hook_found = true;
    
    // Simulate minimal hook execution
    tokio::task::yield_now().await;
    
    // Actual operation
    tokio::task::yield_now().await;
}

#[tokio::main]
async fn main() {
    println!("🔬 Direct performance analysis");
    
    let iterations = 10000;
    
    // Baseline
    let start = Instant::now();
    for _ in 0..iterations {
        basic_operation().await;
    }
    let baseline = start.elapsed();
    
    // With minimal hooks
    let start = Instant::now();
    for _ in 0..iterations {
        operation_with_minimal_hooks().await;
    }
    let with_hooks = start.elapsed();
    
    let overhead = ((with_hooks.as_secs_f64() / baseline.as_secs_f64()) - 1.0) * 100.0;
    
    println!("Results for {} iterations:", iterations);
    println!("  Baseline: {:?}", baseline);
    println!("  With hooks: {:?}", with_hooks);
    println!("  Overhead: {:.3}%", overhead);
    
    if overhead < 1.0 {
        println!("✅ Target achieved!");
    } else {
        println!("❌ Target missed - need optimization");
    }
}