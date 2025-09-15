//! Kernel command implementation - thin wrapper around kernel API
//!
//! This module provides CLI commands for kernel operations.
//! All kernel logic is in llmspell-kernel, this is just command handling.

use anyhow::Result;
use llmspell_config::LLMSpellConfig;
use llmspell_kernel::{connect_to_kernel, start_embedded_kernel, start_kernel_service};
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
            id: _,
            connection_file: _,
        } => {
            if daemon {
                // Service mode - start kernel that listens for connections
                info!("Starting kernel service on port {}", port);
                let service = start_kernel_service(port, runtime_config).await?;
                info!(
                    "Kernel service started. Connection file: {:?}",
                    service.connection_file()
                );
                service.run().await
            } else {
                // Embedded mode - run kernel in-process
                info!("Starting embedded kernel");
                let kernel = start_embedded_kernel(runtime_config).await?;
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
