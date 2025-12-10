//! ABOUTME: Main entry point for llmspell CLI - Phase 9.4.4 Complete Restructure
//! ABOUTME: Professional CLI with dual-mode design and comprehensive tracing

use anyhow::Result;
use clap::Parser;
use llmspell_cli::{cli::Cli, commands::execute_command, config::load_runtime_config};

fn main() -> Result<()> {
    // Check for -V flag before full parsing (simple version output)
    let args: Vec<String> = std::env::args().collect();
    if args.len() >= 2 && args[1] == "-V" {
        llmspell_cli::commands::version::show_version_simple();
        return Ok(());
    }

    let cli = Cli::parse();

    // Initialize tracing based on --trace flag
    setup_tracing(cli.trace);

    // Check for daemon mode in both Kernel and Web commands
    // Check for daemon mode in both Kernel and Web commands
    let is_daemon = matches!(
        &cli.command,
        llmspell_cli::cli::Commands::Kernel {
            command: llmspell_cli::cli::KernelCommands::Start { daemon: true, .. },
        } | llmspell_cli::cli::Commands::Web {
            command: llmspell_cli::cli::WebCommands::Start { daemon: true, .. },
        }
    );

    if is_daemon {
        // Handle daemon mode specially - fork BEFORE creating tokio runtime
        return handle_daemon_mode(cli);
    }

    // For all other cases, use normal async runtime
    let runtime = tokio::runtime::Runtime::new()?;
    runtime.block_on(async {
        // Load runtime configuration
        let config_path = cli.config_path();
        let profile = cli.profile.as_deref();
        let runtime_config = load_runtime_config(config_path.as_deref(), profile).await?;

        // Execute the command with new architecture
        execute_command(cli.command, runtime_config, cli.output, config_path).await
    })
}

/// Handle daemon mode by forking BEFORE creating tokio runtime
fn handle_daemon_mode(cli: Cli) -> Result<()> {
    use llmspell_kernel::daemon::{DaemonConfig, DaemonManager};
    use std::path::PathBuf;

    // Extract daemon-specific parameters
    let (port, id, log_file, pid_file, host) = match &cli.command {
        llmspell_cli::cli::Commands::Kernel {
            command:
                llmspell_cli::cli::KernelCommands::Start {
                    port,
                    id,
                    log_file,
                    pid_file,
                    ..
                },
        } => (
            *port,
            id.clone(),
            log_file.clone(),
            pid_file.clone(),
            "127.0.0.1".to_string(),
        ),

        llmspell_cli::cli::Commands::Web {
            command:
                llmspell_cli::cli::WebCommands::Start {
                    port,
                    log_file,
                    pid_file,
                    host,
                    ..
                },
        } => (
            *port,
            Some("web".to_string()),
            log_file.clone(),
            pid_file.clone(),
            host.clone(),
        ),

        _ => unreachable!("Already checked this is a daemon start command"),
    };

    // Set up daemon configuration
    let default_pid_path = || {
        PathBuf::from("/tmp").join(format!(
            "llmspell-kernel-{}.pid",
            id.as_deref().unwrap_or(&format!("port-{}", port))
        ))
    };

    let default_log_path = || {
        // If log_file is provided and ends with .log, use it directly
        // Otherwise treat it as a directory base
        let base = if let Some(ref log_path) = log_file {
            if log_path.extension().and_then(|s| s.to_str()) == Some("log") {
                // Full log file path provided
                log_path.clone()
            } else {
                // Directory path provided, append kernel log filename
                log_path.join(format!(
                    "kernel-{}.log",
                    id.as_deref().unwrap_or(&format!("port-{}", port))
                ))
            }
        } else {
            // No log file specified, use default location
            dirs::home_dir()
                .unwrap_or_else(|| PathBuf::from("/tmp"))
                .join(".llmspell")
                .join("logs")
                .join(format!(
                    "kernel-{}.log",
                    id.as_deref().unwrap_or(&format!("port-{}", port))
                ))
        };

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

    // Pre-flight check: Load config and print API keys if in production mode
    // We do this BEFORE daemonizing so stdout is still attached to the terminal
    {
        // Use a temporary runtime for config loading
        if let Ok(rt) = tokio::runtime::Runtime::new() {
            let config_path = cli.config_path();
            let profile = cli.profile.as_deref();
            if let Ok(_config) = rt.block_on(load_runtime_config(config_path.as_deref(), profile)) {
                // Get Web Service API keys (what the server actually uses)
                // Note: Currently web command uses default config, so we mirror that here
                let web_config = llmspell_web::config::WebConfig::default();
                let api_keys = web_config.api_keys;

                if !api_keys.is_empty() {
                    println!("\n┌──────────────────────────────────────────────────┐");
                    println!("│                🔐 Access Control                 │");
                    println!("│        (Process will run in background)          │");
                    println!("├──────────────────────────────────────────────────┤");
                    let url = format!("http://{}:{}", host, port);
                    println!("│ Access URL: {:<36} │", url); // aligned for box width
                    println!("│                                                  │");
                    println!("│ Use one of the following API keys to log in:     │");
                    for key in &api_keys {
                        println!("│ • {:<46} │", key);
                    }
                    println!("└──────────────────────────────────────────────────┘\n");
                }
            }
        }
    }

    // Create DaemonManager and daemonize BEFORE creating tokio runtime
    let mut daemon_manager = DaemonManager::new(daemon_config.clone());

    // Check if already running
    if daemon_manager.is_running()? {
        return Err(anyhow::anyhow!(
            "Process already running with PID file {:?}",
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
        let profile = cli.profile.as_deref();
        let runtime_config = load_runtime_config(config_path.as_deref(), profile).await?;

        // Execute the command (now in daemon mode with fresh runtime)
        execute_command(cli.command, runtime_config, cli.output, config_path).await
    })
}

/// Set up tracing based on RUST_LOG environment variable or --trace flag
/// Priority: RUST_LOG > --trace flag > default (warn)
///
/// Best Practice: Tracing output goes to stderr to keep stdout clean for program output
/// This allows: `llmspell exec "code" > output.txt 2> debug.log`
fn setup_tracing(trace_level: llmspell_cli::cli::TraceLevel) {
    use std::io;
    use tracing_subscriber::{fmt, EnvFilter};

    // Define noisy crates to suppress (keep them at WARN even if level is INFO/DEBUG)
    // Detailed debug of these crates should be done via explicit RUST_LOG env var
    const NOISY_CRATES: &[&str] = &[
        "rig_core", "hyper", "h2", "reqwest", "rustls", "want", "mio",
    ];

    // Check if RUST_LOG is set
    if let Ok(env_filter) = std::env::var("RUST_LOG") {
        // Use RUST_LOG environment variable with EnvFilter
        fmt()
            .with_env_filter(EnvFilter::new(env_filter))
            .with_writer(io::stderr) // Explicitly use stderr for tracing
            .with_target(false)
            .init();
    } else {
        // Construct filter based on trace level
        // Map enum to string because EnvFilter parses strings nicely for directives
        let level_str = match trace_level {
            llmspell_cli::cli::TraceLevel::Off => "error",
            llmspell_cli::cli::TraceLevel::Error => "error",
            llmspell_cli::cli::TraceLevel::Warn => "warn",
            llmspell_cli::cli::TraceLevel::Info => "info",
            llmspell_cli::cli::TraceLevel::Debug => "debug",
            llmspell_cli::cli::TraceLevel::Trace => "trace",
        };

        // Start with the base level
        let mut filter_str = level_str.to_string();

        // If the user requested INFO or higher (DEBUG/TRACE), suppress noisy dependencies to WARN
        // This ensures application logs are visible but raw network/library noise is hidden
        // User can still override this by setting RUST_LOG explicitly
        if matches!(
            trace_level,
            llmspell_cli::cli::TraceLevel::Info
                | llmspell_cli::cli::TraceLevel::Debug
                | llmspell_cli::cli::TraceLevel::Trace
        ) {
            for krate in NOISY_CRATES {
                filter_str.push_str(&format!(",{}=warn", krate));
            }
        }

        fmt()
            .with_env_filter(EnvFilter::new(filter_str))
            .with_writer(io::stderr) // Explicitly use stderr for tracing
            .with_target(false)
            .init();
    }
}
