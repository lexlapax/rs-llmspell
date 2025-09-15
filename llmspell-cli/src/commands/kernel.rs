//! Kernel command implementation - thin wrapper around kernel API
//!
//! This module provides CLI commands for kernel operations.
//! All kernel logic is in llmspell-kernel, this is just command handling.

use anyhow::Result;
use llmspell_config::LLMSpellConfig;
use llmspell_kernel::{connect_to_kernel, start_embedded_kernel, start_kernel_service};
use tracing::info;

/// Execute kernel command based on CLI arguments
pub async fn execute_kernel_command(
    subcommand: crate::cli::KernelSubcommand,
    config: LLMSpellConfig,
) -> Result<()> {
    use crate::cli::KernelSubcommand;

    match subcommand {
        KernelSubcommand::Start {
            port,
            daemon,
            id: _,
        } => {
            if daemon {
                // Service mode - start kernel that listens for connections
                let port = port.unwrap_or(9999);
                info!("Starting kernel service on port {}", port);
                let service = start_kernel_service(port, config).await?;
                info!(
                    "Kernel service started. Connection file: {:?}",
                    service.connection_file()
                );
                service.run().await
            } else {
                // Embedded mode - run kernel in-process
                info!("Starting embedded kernel");
                let kernel = start_embedded_kernel(config).await?;
                info!("Kernel {} started", kernel.kernel_id());
                kernel.run().await
            }
        }

        KernelSubcommand::Connect { address } => {
            // Client mode - connect to existing kernel
            info!("Connecting to kernel at: {}", address);
            let mut client = connect_to_kernel(&address).await?;
            client.run_repl()
        }

        KernelSubcommand::Stop { id } => {
            // Stop a running kernel
            info!("Stopping kernel: {}", id);
            // This would send shutdown signal to the kernel
            // For now, this is a placeholder
            anyhow::bail!("Kernel stop not yet implemented")
        }

        KernelSubcommand::Status { id } => {
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

/// Helper function for the main command handler to call
pub async fn handle_kernel_subcommand(
    subcommand: crate::cli::KernelSubcommand,
    config: LLMSpellConfig,
) -> Result<()> {
    execute_kernel_command(subcommand, config).await
}
