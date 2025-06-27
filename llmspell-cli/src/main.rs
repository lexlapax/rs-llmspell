//! ABOUTME: Main entry point for the llmspell command-line tool
//! ABOUTME: Handles argument parsing and dispatches to appropriate command handlers

use anyhow::Result;
use clap::Parser;
use llmspell_cli::{
    cli::Cli,
    commands::execute_command,
    config::load_runtime_config,
};

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging based on verbosity
    let cli = Cli::parse();
    
    let log_level = if cli.verbose {
        tracing::Level::DEBUG
    } else {
        tracing::Level::INFO
    };
    
    tracing_subscriber::fmt()
        .with_max_level(log_level)
        .with_target(false)
        .init();

    // Validate engine selection
    cli.validate_engine()?;

    // Load runtime configuration
    let config_path = cli.config_path();
    let runtime_config = load_runtime_config(config_path.as_deref()).await?;

    // Execute the command
    execute_command(
        cli.command,
        cli.engine,
        runtime_config,
        cli.output,
    ).await?;

    Ok(())
}
