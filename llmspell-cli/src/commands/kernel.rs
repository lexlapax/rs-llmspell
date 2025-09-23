//! Kernel command implementation - thin wrapper around kernel API
//!
//! This module provides CLI commands for kernel operations.
//! All kernel logic is in llmspell-kernel, this is just command handling.

use anyhow::{anyhow, Result};
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
use tracing::{info, warn};

use crate::kernel_discovery::{self, KernelInfo};

/// Handle kernel management commands
pub async fn handle_kernel_command(
    command: crate::cli::KernelCommands,
    runtime_config: LLMSpellConfig,
    _output_format: crate::cli::OutputFormat,
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

        KernelCommands::Status { id } => {
            // Get kernel status
            if let Some(id) = id {
                info!("Getting status for kernel: {}", id);
                println!("Kernel {} status: unknown", id);
            } else {
                info!("Getting status for all kernels");
                println!("No kernels currently running");
            }
            Ok(())
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
