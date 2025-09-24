//! ABOUTME: Main entry point for llmspell CLI - Phase 9.4.4 Complete Restructure
//! ABOUTME: Professional CLI with dual-mode design and comprehensive tracing

use anyhow::Result;
use clap::Parser;
use llmspell_cli::{cli::Cli, commands::execute_command, config::load_runtime_config};

fn main() -> Result<()> {
    let cli = Cli::parse();

    // Initialize tracing based on --trace flag
    setup_tracing(cli.trace);

    // Check if this is a kernel start command with daemon mode
    // We need to handle daemon mode BEFORE creating any tokio runtime
    if let llmspell_cli::cli::Commands::Kernel {
        command: llmspell_cli::cli::KernelCommands::Start { daemon: true, .. },
    } = cli.command
    {
        // Handle daemon mode specially - fork BEFORE creating tokio runtime
        return handle_daemon_mode(cli);
    }

    // For all other cases, use normal async runtime
    let runtime = tokio::runtime::Runtime::new()?;
    runtime.block_on(async {
        // Load runtime configuration
        let config_path = cli.config_path();
        let runtime_config = load_runtime_config(config_path.as_deref()).await?;

        // Execute the command with new architecture
        execute_command(cli.command, runtime_config, cli.output).await
    })
}

/// Handle daemon mode by forking BEFORE creating tokio runtime
fn handle_daemon_mode(cli: Cli) -> Result<()> {
    use llmspell_kernel::daemon::{DaemonConfig, DaemonManager};
    use std::path::PathBuf;

    // Extract daemon-specific parameters
    let (port, id, _connection_file, log_file, pid_file) =
        if let llmspell_cli::cli::Commands::Kernel {
            command:
                llmspell_cli::cli::KernelCommands::Start {
                    port,
                    id,
                    connection_file,
                    log_file,
                    pid_file,
                    ..
                },
        } = &cli.command
        {
            (
                *port,
                id.clone(),
                connection_file.clone(),
                log_file.clone(),
                pid_file.clone(),
            )
        } else {
            unreachable!("Already checked this is a daemon kernel start command");
        };

    // Set up daemon configuration
    let default_pid_path = || {
        PathBuf::from("/tmp").join(format!(
            "llmspell-kernel-{}.pid",
            id.as_deref().unwrap_or(&format!("port-{}", port))
        ))
    };

    let default_log_path = || {
        let base = log_file
            .clone()
            .unwrap_or_else(|| {
                dirs::home_dir()
                    .unwrap_or_else(|| PathBuf::from("/tmp"))
                    .join(".llmspell")
            })
            .join("logs")
            .join(format!(
                "kernel-{}.log",
                id.as_deref().unwrap_or(&format!("port-{}", port))
            ));

        // Ensure parent directory exists
        if let Some(parent) = base.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        base
    };

    let daemon_config = DaemonConfig {
        daemonize: true,
        pid_file: Some(pid_file.unwrap_or_else(default_pid_path)),
        working_dir: PathBuf::from("/tmp"), // Use /tmp as working directory for writeable access
        stdout_path: Some(default_log_path()),
        stderr_path: Some(default_log_path()),
        close_stdin: true,
        umask: Some(0o027),
    };

    // Create DaemonManager and daemonize BEFORE creating tokio runtime
    let mut daemon_manager = DaemonManager::new(daemon_config.clone());

    // Check if already running
    if daemon_manager.is_running()? {
        return Err(anyhow::anyhow!(
            "Kernel daemon already running with PID file {:?}",
            daemon_config.pid_file
        ));
    }

    // Daemonize the process (this will fork)
    daemon_manager.daemonize()?;

    // Now we're in the daemon child process with no tokio runtime
    // Create a fresh runtime and run the kernel
    let runtime = tokio::runtime::Runtime::new()?;
    runtime.block_on(async {
        // Load runtime configuration
        let config_path = cli.config_path();
        let runtime_config = load_runtime_config(config_path.as_deref()).await?;

        // Execute the command (now in daemon mode with fresh runtime)
        execute_command(cli.command, runtime_config, cli.output).await
    })
}

/// Set up tracing based on RUST_LOG environment variable or --trace flag
/// Priority: RUST_LOG > --trace flag > default (warn)
///
/// Best Practice: Tracing output goes to stderr to keep stdout clean for program output
/// This allows: `llmspell exec "code" > output.txt 2> debug.log`
fn setup_tracing(trace_level: llmspell_cli::cli::TraceLevel) {
    use std::io;
    use tracing::Level;
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
        let level: Level = trace_level.into();
        tracing_subscriber::fmt()
            .with_max_level(level)
            .with_writer(io::stderr) // Explicitly use stderr for tracing
            .with_target(false)
            .init();
    }
}
