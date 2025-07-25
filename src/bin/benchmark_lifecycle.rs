//! ABOUTME: Command-line benchmark for agent lifecycle hook performance validation
//! ABOUTME: Provides detailed performance analysis to validate <1% overhead target

use llmspell_agents::lifecycle::{BenchmarkConfig, PerformanceBenchmark};
use anyhow::Result;
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "benchmark-lifecycle")]
#[command(about = "Performance benchmarks for agent lifecycle hook system")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Run performance validation benchmark
    Validate {
        /// Number of iterations to run
        #[arg(long, default_value = "10")]
        iterations: usize,
        
        /// Number of concurrent agents
        #[arg(long, default_value = "20")]
        agents: usize,
        
        /// State transitions per agent
        #[arg(long, default_value = "5")]
        transitions: usize,
        
        /// Hooks per hook point
        #[arg(long, default_value = "3")]
        hooks: usize,
    },
    /// Run quick performance check
    Quick,
    /// Run comprehensive performance suite
    Full,
}

#[tokio::main]
async fn main() -> Result<()> {
    println!("🎯 Agent Lifecycle Hook Performance Benchmark");
    println!("Target: <1% overhead for production hook integration\n");

    let cli = Cli::parse();

    match cli.command {
        Commands::Validate { iterations, agents, transitions, hooks } => {
            let config = BenchmarkConfig {
                iterations,
                concurrent_agents: agents,
                state_transitions_per_agent: transitions,
                hooks_per_point: hooks,
            };
            
            run_benchmark("Custom", config).await?;
        }
        
        Commands::Quick => {
            let config = BenchmarkConfig {
                iterations: 3,
                concurrent_agents: 5,
                state_transitions_per_agent: 3,
                hooks_per_point: 2,
            };
            
            run_benchmark("Quick", config).await?;
        }
        
        Commands::Full => {
            let configs = vec![
                ("Light", BenchmarkConfig {
                    iterations: 5,
                    concurrent_agents: 10,
                    state_transitions_per_agent: 3,
                    hooks_per_point: 2,
                }),
                ("Medium", BenchmarkConfig {
                    iterations: 10,
                    concurrent_agents: 20,
                    state_transitions_per_agent: 5,
                    hooks_per_point: 3,
                }),
                ("Heavy", BenchmarkConfig {
                    iterations: 15,
                    concurrent_agents: 50,
                    state_transitions_per_agent: 8,
                    hooks_per_point: 4,
                }),
            ];

            let mut all_passed = true;
            for (name, config) in configs {
                let passed = run_benchmark(name, config).await?;
                if !passed {
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
                println!("💡 Performance optimization needed");
            }
        }
    }

    Ok(())
}

async fn run_benchmark(name: &str, config: BenchmarkConfig) -> Result<bool> {
    println!("🔄 Running {} benchmark...", name);
    
    let benchmark = PerformanceBenchmark::new(config);
    let results = benchmark.run().await?;
    
    println!("{}", results.summary());
    
    let passed = results.meets_target();
    
    if passed {
        println!("✅ {} PASSED: {:.3}% overhead", name, results.overhead_percentage);
    } else {
        println!("❌ {} FAILED: {:.3}% overhead (target: <1%)", name, results.overhead_percentage);
    }
    
    Ok(passed)
}