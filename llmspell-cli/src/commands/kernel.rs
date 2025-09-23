//! Kernel command implementation - thin wrapper around kernel API
//!
//! This module provides CLI commands for kernel operations.
//! All kernel logic is in llmspell-kernel, this is just command handling.

use anyhow::{anyhow, Result};
use colored::Colorize;
use llmspell_config::LLMSpellConfig;
use llmspell_kernel::{
    connect_to_kernel, start_embedded_kernel_with_executor, start_kernel_service_with_config,
    daemon::DaemonConfig, execution::ExecutionConfig, monitoring::HealthThresholds,
};
use nix::sys::signal::{self, Signal};
use nix::unistd::Pid;
use std::fs;
use std::path::PathBuf;
use std::time::{Duration, Instant};
use tabled::{builder::Builder, settings::Style};
use tracing::{info, warn};

use crate::kernel_discovery::{self, KernelInfo, KernelMetrics, KernelStatus};

/// Handle kernel management commands
pub async fn handle_kernel_command(
    command: crate::cli::KernelCommands,
    runtime_config: LLMSpellConfig,
    output_format: crate::cli::OutputFormat,
) -> Result<()> {
    use crate::cli::KernelCommands;

    match command {
        KernelCommands::Start {
            port,
            daemon,
            id,
            connection_file,
            log_file,
            pid_file,
            idle_timeout,
            max_clients,
            log_rotate_size,
            log_rotate_count,
        } => {
            // Build daemon configuration if daemon mode requested
            let daemon_config = if daemon {
                // Set default paths if not provided
                let default_pid_path = || {
                    dirs::runtime_dir()
                        .unwrap_or_else(|| PathBuf::from("/tmp"))
                        .join(format!("llmspell-kernel-{}.pid",
                            id.as_ref().unwrap_or(&format!("port-{}", port))))
                };

                let default_log_path = || {
                    dirs::state_dir()
                        .unwrap_or_else(|| dirs::home_dir()
                            .unwrap_or_else(|| PathBuf::from("/tmp"))
                            .join(".llmspell"))
                        .join("logs")
                        .join(format!("kernel-{}.log",
                            id.as_ref().unwrap_or(&format!("port-{}", port))))
                };

                Some(DaemonConfig {
                    daemonize: true,
                    pid_file: Some(pid_file.unwrap_or_else(default_pid_path)),
                    working_dir: PathBuf::from("/"),
                    stdout_path: log_file.clone().or_else(|| Some(default_log_path())),
                    stderr_path: log_file.or_else(|| Some(default_log_path())),
                    close_stdin: true,
                    umask: Some(0o027),
                })
            } else {
                None
            };

            // Build execution config with daemon settings
            let exec_config = ExecutionConfig {
                runtime_config: serde_json::to_value(&runtime_config)
                    .ok()
                    .and_then(|v| v.as_object().cloned())
                    .unwrap_or_default()
                    .into_iter()
                    .collect(),
                io_config: llmspell_kernel::execution::IOConfig::default(),
                max_history: 1000,
                execution_timeout_secs: idle_timeout,
                monitor_agents: true,
                track_performance: true,
                daemon_mode: daemon,
                daemon_config: daemon_config.clone(),
                health_thresholds: Some(HealthThresholds::default()),
            };

            if daemon {
                // Service mode - start kernel that listens for connections
                info!("Starting kernel service on port {} in daemon mode", port);

                // Create log directory if needed
                if let Some(ref config) = daemon_config {
                    if let Some(ref log_path) = config.stdout_path {
                        if let Some(parent) = log_path.parent() {
                            std::fs::create_dir_all(parent).ok();
                        }
                    }
                }

                // Create real ScriptExecutor from llmspell-bridge
                let script_executor =
                    llmspell_bridge::create_script_executor(runtime_config.clone()).await?;

                let service = start_kernel_service_with_config(
                    port,
                    exec_config,
                    id,
                    connection_file,
                    max_clients,
                    log_rotate_size,
                    log_rotate_count,
                    script_executor,
                ).await?;

                info!(
                    "Kernel service started. Connection file: {:?}",
                    service.connection_file()
                );
                service.run().await
            } else {
                // Embedded mode - run kernel in-process
                info!("Starting embedded kernel");

                // Create real ScriptExecutor from llmspell-bridge
                let script_executor =
                    llmspell_bridge::create_script_executor(runtime_config.clone()).await?;

                // Create kernel with real executor
                let kernel =
                    start_embedded_kernel_with_executor(runtime_config, script_executor).await?;
                info!("Kernel {} started", kernel.kernel_id());
                kernel.run().await
            }
        }

        KernelCommands::Connect { address } => {
            // Client mode - connect to existing kernel
            match address {
                Some(addr) => {
                    info!("Connecting to kernel at: {}", addr);
                    let mut client = connect_to_kernel(&addr).await?;
                    client.run_repl()
                }
                None => {
                    info!("No address provided, using default connection");
                    anyhow::bail!(
                        "No kernel address provided and auto-discovery not yet implemented"
                    )
                }
            }
        }

        KernelCommands::Stop {
            id,
            pid_file,
            all,
            force,
            timeout,
            no_cleanup,
        } => {
            // Validate arguments
            if id.is_some() && pid_file.is_some() {
                anyhow::bail!("Cannot specify both --id and --pid-file");
            }
            if all && (id.is_some() || pid_file.is_some()) {
                anyhow::bail!("Cannot use --all with --id or --pid-file");
            }
            if !all && id.is_none() && pid_file.is_none() {
                anyhow::bail!("Must specify --id, --pid-file, or --all");
            }

            // Find kernels to stop
            let kernels_to_stop = if all {
                info!("Discovering all running kernels...");
                kernel_discovery::discover_kernels()?
            } else if let Some(kernel_id) = id {
                info!("Finding kernel with ID: {}", kernel_id);
                vec![kernel_discovery::find_kernel_by_id(&kernel_id)?]
            } else if let Some(pid_file_path) = pid_file {
                info!("Reading PID from file: {}", pid_file_path.display());

                // Read PID from file
                let pid_str = fs::read_to_string(&pid_file_path)
                    .map_err(|e| anyhow!("Failed to read PID file: {}", e))?;
                let pid: u32 = pid_str.trim().parse()
                    .map_err(|e| anyhow!("Invalid PID in file: {}", e))?;

                // Create minimal KernelInfo
                vec![KernelInfo {
                    id: format!("pid-{}", pid),
                    pid,
                    port: 0, // Unknown
                    connection_file: PathBuf::new(),
                    pid_file: Some(pid_file_path),
                    log_file: None,
                    status: kernel_discovery::KernelStatus::Unknown,
                    start_time: None,
                }]
            } else {
                vec![]
            };

            if kernels_to_stop.is_empty() {
                println!("No kernels found to stop");
                return Ok(());
            }

            // Stop each kernel
            let mut stopped_count = 0;
            let mut failed_count = 0;

            for kernel in kernels_to_stop {
                println!("Stopping kernel {} (PID: {})", kernel.id, kernel.pid);

                match stop_kernel_process(&kernel, force, timeout, no_cleanup).await {
                    Ok(()) => {
                        println!("✓ Kernel {} stopped successfully", kernel.id);
                        stopped_count += 1;
                    }
                    Err(e) => {
                        eprintln!("✗ Failed to stop kernel {}: {}", kernel.id, e);
                        failed_count += 1;
                    }
                }
            }

            // Report results
            if failed_count == 0 {
                println!("\n✓ Successfully stopped {} kernel(s)", stopped_count);
                Ok(())
            } else {
                anyhow::bail!(
                    "Failed to stop {} kernel(s), succeeded with {} kernel(s)",
                    failed_count,
                    stopped_count
                )
            }
        }

        KernelCommands::Status { id, format, quiet, watch, interval } => {
            // Parse output format from string
            let output_format = match format.as_str() {
                "json" => crate::cli::OutputFormat::Json,
                "yaml" => crate::cli::OutputFormat::Yaml,
                "table" | "pretty" => crate::cli::OutputFormat::Pretty,
                _ => crate::cli::OutputFormat::Text,
            };

            // Check for detailed mode based on format string
            let detailed = format.contains("detailed");

            // Handle watch mode
            if watch {
                loop {
                    // Clear screen for watch mode
                    print!("\x1B[2J\x1B[1;1H");

                    display_kernel_status(id.as_deref(), quiet, detailed, &output_format)?;

                    tokio::time::sleep(Duration::from_secs(interval)).await;
                }
            } else {
                display_kernel_status(id.as_deref(), quiet, detailed, &output_format)
            }
        }
    }
}

/// Stop a kernel process with graceful shutdown and cleanup
async fn stop_kernel_process(
    kernel: &KernelInfo,
    force: bool,
    timeout: u64,
    no_cleanup: bool,
) -> Result<()> {
    // Check if process is alive
    if !kernel_discovery::is_process_alive(kernel.pid) {
        info!("Kernel {} (PID {}) is not running", kernel.id, kernel.pid);

        // Still clean up files if requested
        if !no_cleanup {
            cleanup_kernel_files(kernel)?;
        }

        return Ok(());
    }

    // If force mode, skip graceful shutdown
    if force {
        warn!("Force killing kernel {} (PID {})", kernel.id, kernel.pid);
        signal::kill(Pid::from_raw(kernel.pid as i32), Signal::SIGKILL)
            .map_err(|e| anyhow!("Failed to send SIGKILL: {}", e))?;
    } else {
        // Send SIGTERM for graceful shutdown
        info!("Sending SIGTERM to kernel {} (PID {})", kernel.id, kernel.pid);
        signal::kill(Pid::from_raw(kernel.pid as i32), Signal::SIGTERM)
            .map_err(|e| anyhow!("Failed to send SIGTERM: {}", e))?;

        // Wait for graceful shutdown
        let deadline = Instant::now() + Duration::from_secs(timeout);
        let mut last_check = Instant::now();

        while Instant::now() < deadline {
            if !kernel_discovery::is_process_alive(kernel.pid) {
                info!("Kernel {} stopped gracefully", kernel.id);
                break;
            }

            // Print progress every 5 seconds
            if Instant::now() - last_check > Duration::from_secs(5) {
                let remaining = (deadline - Instant::now()).as_secs();
                println!("  Waiting for graceful shutdown... {} seconds remaining", remaining);
                last_check = Instant::now();
            }

            tokio::time::sleep(Duration::from_millis(100)).await;
        }

        // Force kill if still alive
        if kernel_discovery::is_process_alive(kernel.pid) {
            warn!("Kernel {} didn't shutdown gracefully within {} seconds, sending SIGKILL",
                  kernel.id, timeout);
            signal::kill(Pid::from_raw(kernel.pid as i32), Signal::SIGKILL)
                .map_err(|e| anyhow!("Failed to send SIGKILL: {}", e))?;

            // Wait briefly for SIGKILL to take effect
            tokio::time::sleep(Duration::from_millis(500)).await;
        }
    }

    // Verify process is dead
    if kernel_discovery::is_process_alive(kernel.pid) {
        return Err(anyhow!("Failed to stop kernel {} (PID {})", kernel.id, kernel.pid));
    }

    // Clean up files unless --no-cleanup was specified
    if !no_cleanup {
        cleanup_kernel_files(kernel)?;
    }

    Ok(())
}

/// Clean up kernel-related files (connection file, PID file)
fn cleanup_kernel_files(kernel: &KernelInfo) -> Result<()> {
    // Clean up connection file
    if !kernel.connection_file.as_os_str().is_empty() && kernel.connection_file.exists() {
        info!("Removing connection file: {}", kernel.connection_file.display());
        fs::remove_file(&kernel.connection_file)
            .map_err(|e| anyhow!("Failed to remove connection file: {}", e))?;
    }

    // Clean up PID file
    if let Some(ref pid_file) = kernel.pid_file {
        if pid_file.exists() {
            info!("Removing PID file: {}", pid_file.display());
            fs::remove_file(pid_file)
                .map_err(|e| anyhow!("Failed to remove PID file: {}", e))?;
        }
    }

    // Note: We intentionally do NOT delete log files as they may be useful for debugging
    if let Some(ref log_file) = kernel.log_file {
        info!("Preserving log file for debugging: {}", log_file.display());
    }

    Ok(())
}

/// Display kernel status information
fn display_kernel_status(
    kernel_id: Option<&str>,
    quiet: bool,
    detailed: bool,
    output_format: &crate::cli::OutputFormat,
) -> Result<()> {
    use crate::cli::OutputFormat;

    // Discover kernels
    let kernels = if let Some(id) = kernel_id {
        // Find specific kernel
        vec![kernel_discovery::find_kernel_by_id(id)?]
    } else {
        // Find all kernels
        kernel_discovery::discover_kernels()?
    };

    if kernels.is_empty() {
        if !quiet {
            println!("No kernels currently running");
        }
        return Ok(());
    }

    // Collect metrics for each kernel
    let mut kernel_data = Vec::new();
    for kernel in &kernels {
        let metrics = kernel_discovery::get_kernel_metrics(kernel).ok();
        kernel_data.push((kernel, metrics));
    }

    match output_format {
        OutputFormat::Pretty => {
            if detailed {
                display_detailed_table(&kernel_data)?;
            } else {
                display_summary_table(&kernel_data)?;
            }
        }
        OutputFormat::Json => {
            display_json(&kernel_data)?;
        }
        OutputFormat::Yaml => {
            // Convert to YAML format
            println!("{}", serde_yaml::to_string(&kernel_data.iter()
                .map(|(k, m)| serde_json::json!({
                    "kernel": k,
                    "metrics": m,
                }))
                .collect::<Vec<_>>())?);
        }
        OutputFormat::Text => {
            display_simple(&kernel_data, quiet)?;
        }
    }

    Ok(())
}

/// Display kernels in a summary table
fn display_summary_table(kernel_data: &[(&KernelInfo, Option<KernelMetrics>)]) -> Result<()> {
    let mut builder = Builder::default();

    // Add header
    builder.push_record(["ID", "PID", "Port", "Status", "CPU%", "Memory", "Uptime"]);

    for (kernel, metrics) in kernel_data {
        let status_str = format_status(&kernel.status);

        if let Some(m) = metrics {
            builder.push_record([
                &kernel.id,
                &kernel.pid.to_string(),
                &kernel.port.to_string(),
                &status_str,
                &format!("{:.1}", m.cpu_percent),
                &format_memory(m.memory_bytes),
                &format_duration(&m.uptime),
            ]);
        } else {
            builder.push_record([
                &kernel.id,
                &kernel.pid.to_string(),
                &kernel.port.to_string(),
                &status_str,
                "N/A",
                "N/A",
                "N/A",
            ]);
        }
    }

    let table = builder.build().with(Style::rounded()).to_string();
    println!("{}", table);

    Ok(())
}

/// Display kernels in a detailed table
fn display_detailed_table(kernel_data: &[(&KernelInfo, Option<KernelMetrics>)]) -> Result<()> {
    for (kernel, metrics) in kernel_data {
        println!("\n{}", "═".repeat(60));
        println!("Kernel: {}", kernel.id.bold());
        println!("{}", "─".repeat(60));

        let mut builder = Builder::default();
        builder.push_record(["Property", "Value"]);

        builder.push_record(["Process ID", &kernel.pid.to_string()]);
        builder.push_record(["Port", &kernel.port.to_string()]);
        builder.push_record(["Status", &format_status(&kernel.status)]);
        builder.push_record(["Connection File", &kernel.connection_file.display().to_string()]);

        if let Some(pid_file) = &kernel.pid_file {
            builder.push_record(["PID File", &pid_file.display().to_string()]);
        }

        if let Some(log_file) = &kernel.log_file {
            builder.push_record(["Log File", &log_file.display().to_string()]);
        }

        if let Some(m) = metrics {
            builder.push_record(["", ""]);
            builder.push_record(["CPU Usage", &format!("{:.1}%", m.cpu_percent)]);
            builder.push_record(["Memory Usage", &format!("{} ({:.1}%)", format_memory(m.memory_bytes), m.memory_percent)]);
            builder.push_record(["Open Files", &m.open_files.to_string()]);
            builder.push_record(["Active Connections", &m.active_connections.to_string()]);
            builder.push_record(["Uptime", &format_duration(&m.uptime)]);

            if let Some(last_activity) = m.last_activity {
                let elapsed = std::time::SystemTime::now()
                    .duration_since(last_activity)
                    .unwrap_or(Duration::ZERO);
                builder.push_record(["Last Activity", &format!("{} ago", format_duration(&elapsed))]);
            }
        }

        let table = builder.build().with(Style::rounded()).to_string();
        println!("{}", table);
    }

    Ok(())
}

/// Display kernels as JSON
fn display_json(kernel_data: &[(&KernelInfo, Option<KernelMetrics>)]) -> Result<()> {
    #[derive(serde::Serialize)]
    struct KernelStatusOutput {
        kernel: KernelInfo,
        metrics: Option<KernelMetrics>,
    }

    let output: Vec<KernelStatusOutput> = kernel_data
        .iter()
        .map(|(k, m)| KernelStatusOutput {
            kernel: (*k).clone(),
            metrics: m.clone(),
        })
        .collect();

    println!("{}", serde_json::to_string_pretty(&output)?);

    Ok(())
}

/// Display kernels in simple format
fn display_simple(kernel_data: &[(&KernelInfo, Option<KernelMetrics>)], quiet: bool) -> Result<()> {
    for (kernel, metrics) in kernel_data {
        if quiet {
            println!("{}", kernel.id);
        } else {
            print!("{}: PID={} Port={} Status={}",
                kernel.id, kernel.pid, kernel.port, format_status(&kernel.status));

            if let Some(m) = metrics {
                print!(" CPU={:.1}% Memory={} Uptime={}",
                    m.cpu_percent,
                    format_memory(m.memory_bytes),
                    format_duration(&m.uptime));
            }

            println!();
        }
    }

    Ok(())
}

/// Format kernel status with color
fn format_status(status: &KernelStatus) -> String {
    match status {
        KernelStatus::Healthy => "Healthy".green().to_string(),
        KernelStatus::Busy => "Busy".yellow().to_string(),
        KernelStatus::Idle => "Idle".blue().to_string(),
        KernelStatus::ShuttingDown => "Shutting Down".red().to_string(),
        KernelStatus::Unknown => "Unknown".dimmed().to_string(),
    }
}

/// Format memory size in human-readable format
fn format_memory(bytes: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
    let mut size = bytes as f64;
    let mut unit_idx = 0;

    while size >= 1024.0 && unit_idx < UNITS.len() - 1 {
        size /= 1024.0;
        unit_idx += 1;
    }

    if unit_idx == 0 {
        format!("{} {}", size as u64, UNITS[unit_idx])
    } else {
        format!("{:.1} {}", size, UNITS[unit_idx])
    }
}

/// Format duration in human-readable format
fn format_duration(duration: &Duration) -> String {
    let secs = duration.as_secs();

    if secs < 60 {
        format!("{}s", secs)
    } else if secs < 3600 {
        format!("{}m {}s", secs / 60, secs % 60)
    } else if secs < 86400 {
        format!("{}h {}m", secs / 3600, (secs % 3600) / 60)
    } else {
        format!("{}d {}h", secs / 86400, (secs % 86400) / 3600)
    }
}
