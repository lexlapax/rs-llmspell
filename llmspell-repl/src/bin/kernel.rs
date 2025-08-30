//! `LLMSpell` Kernel standalone executable
//!
//! This binary runs the `LLMSpell` kernel as a standalone process that can be
//! connected to by multiple clients via TCP channels.

use anyhow::Result;
use clap::Parser;
use llmspell_config::LLMSpellConfig;
use llmspell_repl::kernel::{KernelConfig, LLMSpellKernel};
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

/// `LLMSpell` Kernel - Multi-client REPL and debugging service
#[derive(Parser, Debug)]
#[command(name = "llmspell-kernel")]
#[command(version, about, long_about = None)]
struct Args {
    /// Kernel ID (auto-generated if not provided)
    #[arg(long)]
    kernel_id: Option<String>,

    /// IP address to bind to
    #[arg(long, default_value = "127.0.0.1")]
    ip: String,

    /// Starting port for channel allocation
    #[arg(long, default_value = "5555")]
    port: u16,

    /// Script engine to use (lua, javascript)
    #[arg(long, default_value = "lua")]
    engine: String,

    /// Maximum number of clients
    #[arg(long, default_value = "10")]
    max_clients: usize,

    /// Enable debug mode
    #[arg(long)]
    debug: bool,

    /// Enable authentication
    #[arg(long)]
    auth: bool,

    /// Configuration file path
    #[arg(long)]
    config: Option<String>,

    /// Verbosity level (can be used multiple times)
    #[arg(short, long, action = clap::ArgAction::Count)]
    verbose: u8,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    // Setup logging
    setup_logging(args.verbose);

    // Load configuration
    let runtime_config = if let Some(config_path) = args.config {
        // Load from file
        let config_str = tokio::fs::read_to_string(&config_path).await?;
        serde_json::from_str(&config_str)?
    } else {
        // Use default configuration
        LLMSpellConfig::default()
    };

    // Create kernel configuration
    let kernel_config = KernelConfig {
        kernel_id: args.kernel_id,
        ip: args.ip,
        port_range_start: args.port,
        debug_enabled: args.debug,
        max_clients: args.max_clients,
        engine: args.engine,
        runtime_config,
        auth_enabled: args.auth,
    };

    // Start and run the kernel
    tracing::info!("Starting LLMSpell kernel...");
    tracing::info!("Configuration: {:?}", kernel_config);

    let kernel = Box::pin(LLMSpellKernel::start(kernel_config)).await?;

    tracing::info!("Kernel {} started successfully", kernel.kernel_id);
    tracing::info!(
        "Connection file written to: {}",
        kernel.connection_info.connection_file_path().display()
    );
    tracing::info!(
        "Listening on {}:{}-{}",
        kernel.connection_info.ip,
        kernel.connection_info.shell_port,
        kernel.connection_info.hb_port
    );

    // Run the kernel event loop
    kernel.run().await?;

    tracing::info!("Kernel shutdown complete");
    Ok(())
}

fn setup_logging(verbosity: u8) {
    let filter_level = match verbosity {
        0 => "warn",
        1 => "info",
        2 => "debug",
        _ => "trace",
    };

    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(filter_level));

    tracing_subscriber::registry()
        .with(fmt::layer())
        .with(filter)
        .init();
}
