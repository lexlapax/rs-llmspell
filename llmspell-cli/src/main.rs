//! ABOUTME: Main entry point for llmspell CLI - Phase 9.4.4 Complete Restructure
//! ABOUTME: Professional CLI with dual-mode design and comprehensive tracing

use anyhow::Result;
use clap::Parser;
use llmspell_cli::{cli::Cli, commands::execute_command, config::load_runtime_config};

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    // Initialize tracing based on --trace flag
    setup_tracing(cli.trace);

    // Load runtime configuration
    let config_path = cli.config_path();
    let runtime_config = load_runtime_config(config_path.as_deref()).await?;

    // Execute the command with new architecture
    execute_command(cli.command, runtime_config, cli.output).await?;

    Ok(())
}

/// Set up tracing based on RUST_LOG environment variable or --trace flag
/// Priority: RUST_LOG > --trace flag > default (warn)
///
/// Best Practice: Tracing output goes to stderr to keep stdout clean for program output
/// This allows: `llmspell exec "code" > output.txt 2> debug.log`
fn setup_tracing(trace_level: llmspell_cli::cli::TraceLevel) {
    use std::io;
    use tracing_subscriber::EnvFilter;

    // Check if RUST_LOG is set
    if std::env::var("RUST_LOG").is_ok() {
        // Use RUST_LOG environment variable with EnvFilter
        tracing_subscriber::fmt()
            .with_env_filter(EnvFilter::from_default_env())
            .with_writer(io::stderr) // Explicitly use stderr for tracing
            .with_target(false)
            .init();
    } else {
        // Use --trace flag
        let level: tracing::Level = trace_level.into();
        tracing_subscriber::fmt()
            .with_max_level(level)
            .with_writer(io::stderr) // Explicitly use stderr for tracing
            .with_target(false)
            .init();
    }
}
