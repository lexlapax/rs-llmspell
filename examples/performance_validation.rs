//! ABOUTME: Production performance validation example for agent lifecycle hooks
//! ABOUTME: Demonstrates <1% overhead achievement with realistic production workloads

use llmspell_agents::lifecycle::{BenchmarkConfig, PerformanceBenchmark};
use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    println!("🎯 Agent Lifecycle Hook Performance Validation");
    println!("Target: <1% overhead for production hook integration\n");

    // Test with different workload intensities
    let configs = vec![
        // Light workload
        ("Light workload", BenchmarkConfig {
            iterations: 3,
            concurrent_agents: 5,
            state_transitions_per_agent: 3,
            hooks_per_point: 2,
        }),
        // Medium workload 
        ("Medium workload", BenchmarkConfig {
            iterations: 5,
            concurrent_agents: 10,
            state_transitions_per_agent: 5,
            hooks_per_point: 3,
        }),
        // Heavy workload
        ("Heavy workload", BenchmarkConfig {
            iterations: 8,
            concurrent_agents: 20,
            state_transitions_per_agent: 8,
            hooks_per_point: 4,
        }),
    ];

    let mut all_passed = true;

    for (name, config) in configs {
        println!("🔄 Running {} benchmark...", name);
        
        let benchmark = PerformanceBenchmark::new(config);
        let results = benchmark.run().await?;
        
        println!("{}", results.summary());
        
        if results.meets_target() {
            println!("✅ {} PASSED: {:.3}% overhead", name, results.overhead_percentage);
        } else {
            println!("❌ {} FAILED: {:.3}% overhead (target: <1%)", name, results.overhead_percentage);
            all_passed = false;
        }
        println!();
    }

    if all_passed {
        println!("🎉 ALL BENCHMARKS PASSED!");
        println!("✅ Hook system meets <1% performance overhead target");
        println!("✅ Ready for production deployment");
    } else {
        println!("⚠️  Some benchmarks exceeded 1% overhead target");
        println!("💡 Consider optimizing hook implementations or reducing hook count");
    }

    Ok(())
}