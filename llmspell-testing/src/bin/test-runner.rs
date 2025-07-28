// ABOUTME: CLI test runner for unified test discovery and execution
// ABOUTME: Provides simple interface to run different test categories

use clap::{Parser, Subcommand};
use colored::*;
use llmspell_testing::runner::{TestCategory, TestRunner, TestRunnerConfig};
use std::process;

#[derive(Parser)]
#[command(name = "llmspell-test")]
#[command(version = "1.0")]
#[command(about = "Unified test runner for rs-llmspell", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    /// Enable verbose output
    #[arg(short, long, global = true)]
    verbose: bool,

    /// Run tests in release mode
    #[arg(short, long, global = true)]
    release: bool,

    /// Number of parallel test threads (0 = number of CPUs)
    #[arg(short = 'j', long, global = true, default_value = "0")]
    jobs: usize,

    /// Don't capture test output
    #[arg(long, global = true)]
    nocapture: bool,
}

#[derive(Subcommand)]
enum Commands {
    /// List all available test categories
    List {
        /// Show detailed information about each category
        #[arg(short, long)]
        detailed: bool,
    },

    /// Run tests for specific categories
    Run {
        /// Test categories to run (unit, integration, agent, scenario, lua, performance, all)
        #[arg(value_name = "CATEGORY")]
        categories: Vec<String>,

        /// Filter tests by name pattern
        #[arg(short, long)]
        filter: Option<String>,

        /// Generate coverage report
        #[arg(long)]
        coverage: bool,

        /// Output format (pretty, json, junit)
        #[arg(long, default_value = "pretty")]
        format: String,
    },

    /// Show information about a specific test category
    Info {
        /// Test category to show info for
        category: String,
    },

    /// Run benchmarks
    Bench {
        /// Benchmark filter pattern
        #[arg(value_name = "PATTERN")]
        filter: Option<String>,

        /// Save benchmark results to file
        #[arg(long)]
        save: Option<String>,

        /// Compare with baseline
        #[arg(long)]
        baseline: Option<String>,
    },
}

fn main() {
    let cli = Cli::parse();

    // Configure test runner
    let config = TestRunnerConfig {
        verbose: cli.verbose,
        release: cli.release,
        jobs: cli.jobs,
        nocapture: cli.nocapture,
    };

    let runner = TestRunner::new(config);

    let result = match cli.command {
        Commands::List { detailed } => list_categories(detailed),
        Commands::Run {
            categories,
            filter,
            coverage,
            format,
        } => runner.run_tests(categories, filter, coverage, format),
        Commands::Info { category } => show_category_info(&category),
        Commands::Bench {
            filter,
            save,
            baseline,
        } => runner.run_benchmarks(filter, save, baseline),
    };

    if let Err(e) = result {
        eprintln!("{}: {}", "Error".red().bold(), e);
        process::exit(1);
    }
}

fn list_categories(detailed: bool) -> Result<(), Box<dyn std::error::Error>> {
    println!("{}", "Available Test Categories:".green().bold());
    println!();

    let categories = vec![
        (
            "unit",
            "Unit tests for individual components",
            "Fast, isolated tests that verify component behavior",
        ),
        (
            "integration",
            "Integration tests across components",
            "Tests that verify component interactions and data flow",
        ),
        (
            "agent",
            "Agent-specific functionality tests",
            "Tests for agent creation, lifecycle, and behavior",
        ),
        (
            "scenario",
            "End-to-end scenario tests",
            "Complex workflows simulating real-world usage",
        ),
        (
            "lua",
            "Lua scripting bridge tests",
            "Tests for Lua API and script execution",
        ),
        (
            "performance",
            "Performance benchmarks",
            "Criterion benchmarks measuring performance metrics",
        ),
        (
            "all",
            "Run all test categories",
            "Execute the complete test suite",
        ),
    ];

    for (name, short, long) in categories {
        if detailed {
            println!("  {} - {}", name.yellow().bold(), short);
            println!("    {}", long.dimmed());
            println!();
        } else {
            println!("  {} - {}", name.yellow(), short);
        }
    }

    if !detailed {
        println!();
        println!("Use 'llmspell-test list --detailed' for more information");
    }

    Ok(())
}

fn show_category_info(category: &str) -> Result<(), Box<dyn std::error::Error>> {
    let cat: TestCategory = category.parse()?;

    println!("{}: {}", "Category".green().bold(), cat.name().yellow());
    println!("{}: {}", "Description".green(), cat.description());
    println!();

    println!("{}:", "Test Files".green().bold());
    for file in cat.test_files() {
        println!("  - {}", file);
    }
    println!();

    println!("{}:", "Run Command".green().bold());
    println!("  llmspell-test run {}", category);
    println!();

    println!("{}:", "Cargo Command".green().bold());
    println!(
        "  cargo test -p llmspell-testing --features {}-tests",
        category
    );

    Ok(())
}
