//! LLMSpell Kernel Binary - Lightweight wrapper
//!
//! This binary is a thin wrapper that starts the LLMSpell kernel.
//! The actual kernel implementation is in llmspell_kernel::kernel module.

use anyhow::Result;
use clap::Parser;
use llmspell_config::LLMSpellConfig;
use llmspell_kernel::{KernelConfig, LLMSpellKernel};
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

/// LLMSpell Kernel - Jupyter-compatible execution kernel
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
    #[arg(long, default_value = "9555")]
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
    let runtime_config = if let Some(config_path) = &args.config {
        let config_str = tokio::fs::read_to_string(config_path).await?;
        serde_json::from_str(&config_str)?
    } else {
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

    // Start and run the kernel (heavy lifting is in kernel.rs)
    let kernel = LLMSpellKernel::start(kernel_config).await?;
    kernel.run().await?;

    Ok(())
}

fn setup_logging(verbosity: u8) {
    let filter_level = match verbosity {
        0 => "warn", 
        1 => "info",
        2 => "debug",
        _ => "trace",
    };

    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new(filter_level));

    tracing_subscriber::registry()
        .with(fmt::layer())
        .with(filter)
        .init();
}
