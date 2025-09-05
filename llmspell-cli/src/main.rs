//! ABOUTME: Main entry point for the llmspell command-line tool
//! ABOUTME: Handles argument parsing and dispatches to appropriate command handlers

use anyhow::Result;
use clap::Parser;
use llmspell_cli::{cli::Cli, commands::execute_command, config::load_runtime_config};
use llmspell_utils::debug::{global_debug_manager, DebugLevel};
use std::sync::Arc;

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

    // Initialize debug system based on CLI flags
    let debug_manager = global_debug_manager();

    // Apply CLI debug settings (highest priority)
    if cli.debug {
        debug_manager.set_enabled(true);
    }

    if let Some(ref level) = cli.debug_level {
        if let Some(debug_level) = parse_debug_level(level) {
            debug_manager.set_level(debug_level);
        }
    }

    if let Some(ref modules) = cli.debug_modules {
        apply_module_filters(&debug_manager, modules);
    }

    if cli.debug_perf {
        // Enable performance tracking will be handled by the config
    }

    // Validate engine selection
    cli.validate_engine()?;

    // Load runtime configuration
    let config_path = cli.config_path();
    let mut runtime_config = load_runtime_config(config_path.as_deref()).await?;

    // Apply CLI debug settings to config (for passing to script engines)
    apply_debug_cli_to_config(&cli, &mut runtime_config);

    // Execute the command if provided
    if let Some(command) = cli.command {
        execute_command(command, cli.engine, runtime_config, cli.output).await?;
    } else {
        eprintln!("No command provided. Use --help for usage information.");
        std::process::exit(1);
    }

    Ok(())
}

fn parse_debug_level(level: &str) -> Option<DebugLevel> {
    match level.to_lowercase().as_str() {
        "off" => Some(DebugLevel::Off),
        "error" => Some(DebugLevel::Error),
        "warn" => Some(DebugLevel::Warn),
        "info" => Some(DebugLevel::Info),
        "debug" => Some(DebugLevel::Debug),
        "trace" => Some(DebugLevel::Trace),
        _ => None,
    }
}

fn apply_module_filters(debug_manager: &Arc<llmspell_utils::debug::DebugManager>, modules: &str) {
    for module in modules.split(',') {
        let module = module.trim();
        if let Some(enabled_module) = module.strip_prefix('+') {
            debug_manager.add_module_filter(enabled_module, true);
        } else if let Some(disabled_module) = module.strip_prefix('-') {
            debug_manager.add_module_filter(disabled_module, false);
        } else {
            // Default to enabling if no prefix
            debug_manager.add_module_filter(module, true);
        }
    }
}

fn apply_debug_cli_to_config(cli: &Cli, config: &mut llmspell_config::LLMSpellConfig) {
    // Apply CLI debug settings to config (highest priority)
    if cli.debug {
        config.debug.enabled = true;
    }

    if let Some(ref level) = cli.debug_level {
        config.debug.level = level.clone();
    }

    if let Some(ref format) = cli.debug_format {
        config.debug.output.format = format.clone();
    }

    if let Some(ref modules) = cli.debug_modules {
        // Parse module filters
        for module in modules.split(',') {
            let module = module.trim();
            if let Some(enabled) = module.strip_prefix('+') {
                config
                    .debug
                    .module_filters
                    .enabled
                    .push(enabled.to_string());
            } else if let Some(disabled) = module.strip_prefix('-') {
                config
                    .debug
                    .module_filters
                    .disabled
                    .push(disabled.to_string());
            }
        }
    }

    if cli.debug_perf {
        config.debug.performance.enabled = true;
    }
}
