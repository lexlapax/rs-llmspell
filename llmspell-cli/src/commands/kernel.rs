//! Kernel command implementation - thin wrapper around kernel API
//!
//! This module provides CLI commands for kernel operations.
//! All kernel logic is in llmspell-kernel, this is just command handling.

use anyhow::Result;
use llmspell_config::LLMSpellConfig;
use llmspell_kernel::{
    connect_to_kernel, start_embedded_kernel_with_executor, start_kernel_service_with_config,
    daemon::DaemonConfig, execution::ExecutionConfig, monitoring::HealthThresholds,
};
use std::path::PathBuf;
use tracing::info;

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

        KernelCommands::Stop { id } => {
            // Stop a running kernel
            match id {
                Some(kernel_id) => {
                    info!("Stopping kernel: {}", kernel_id);
                    // This would send shutdown signal to the kernel
                    // For now, this is a placeholder
                    anyhow::bail!("Kernel stop not yet implemented")
                }
                None => {
                    info!("Stopping all kernels");
                    anyhow::bail!("Stopping all kernels not yet implemented")
                }
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
