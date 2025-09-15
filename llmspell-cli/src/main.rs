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

/// Set up tracing based on trace level
fn setup_tracing(trace_level: llmspell_cli::cli::TraceLevel) {
    let level: tracing::Level = trace_level.into();

    tracing_subscriber::fmt()
        .with_max_level(level)
        .with_target(false)
        .init();
}
