//! ABOUTME: Main entry point for the llmspell command-line tool
//! ABOUTME: Handles argument parsing and dispatches to appropriate command handlers

use anyhow::Result;
use clap::Parser;
use llmspell_cli::{cli::Cli, commands::execute_command, config::load_runtime_config};

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    // Initialize tracing based on trace level
    if let Some(trace_level) = cli.trace {
        let level = if trace_level == llmspell_cli::cli::TraceLevel::Off {
            // For "off" level, set to ERROR to minimize output
            tracing::Level::ERROR
        } else {
            trace_level.into()
        };

        tracing_subscriber::fmt()
            .with_max_level(level)
            .with_target(false)
            .init();
    } else {
        // Default to INFO level if no trace flag specified
        tracing_subscriber::fmt()
            .with_max_level(tracing::Level::INFO)
            .with_target(false)
            .init();
    }

    // Validate engine selection
    cli.validate_engine()?;

    // Load runtime configuration
    let config_path = cli.config_path();
    let runtime_config = load_runtime_config(config_path.as_deref()).await?;

    // Execute the command if provided
    if let Some(command) = cli.command {
        execute_command(command, cli.engine, runtime_config, cli.output).await?;
    } else {
        eprintln!("No command provided. Use --help for usage information.");
        std::process::exit(1);
    }

    Ok(())
}
