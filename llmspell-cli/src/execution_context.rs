//! # Execution Context Resolution for Dual-Mode Design
//!
//! This module implements the core ExecutionContext system that enables seamless switching
//! between embedded (in-process) and connected (remote) kernel execution modes. It provides
//! intelligent auto-detection, connection management, and configuration resolution.
//!
//! ## Architecture Overview
//!
//! The ExecutionContext enum represents two distinct execution modes:
//!
//! - **Embedded**: In-process kernel execution with direct configuration access
//! - **Connected**: Remote kernel execution via client handles
//!
//! ## Resolution Priority
//!
//! The context resolution follows this priority order:
//!
//! 1. `--connect <address>` - Explicit connection to remote kernel
//! 2. `--kernel <id>` - Connection by kernel ID lookup
//! 3. `--config <path>` - Embedded mode with specific configuration
//! 4. Auto-detection - Search for running kernels, fallback to embedded
//!
//! ## Auto-Detection Process
//!
//! When no explicit flags are provided, the system:
//! 1. Scans `~/.llmspell/kernels/` for active kernel connection files
//! 2. Tests connectivity to found kernels via TCP ping
//! 3. Connects to first responsive kernel found
//! 4. Falls back to embedded mode if no kernels are available
//!
//! ## Usage Examples
//!
//! ```rust,no_run
//! use llmspell_cli::execution_context::ExecutionContext;
//! use llmspell_config::LLMSpellConfig;
//!
//! # async fn example() -> anyhow::Result<()> {
//! // Auto-detection mode
//! let context = ExecutionContext::resolve(None, None, None, LLMSpellConfig::default()).await?;
//!
//! // Explicit connection
//! let context = ExecutionContext::resolve(
//!     Some("localhost:9572".to_string()),
//!     None,
//!     None,
//!     LLMSpellConfig::default()
//! ).await?;
//!
//! // Check execution mode
//! if context.is_embedded() {
//!     println!("Running in embedded mode");
//! } else {
//!     println!("Connected to: {}", context.connection_info().unwrap());
//! }
//! # Ok(())
//! # }
//! ```

use anyhow::Result;
use llmspell_config::LLMSpellConfig;
use llmspell_kernel::api::{connect_to_kernel, start_embedded_kernel, ClientHandle, KernelHandle};
use std::path::PathBuf;
use tracing::{debug, info};

/// Execution context for dual-mode design
pub enum ExecutionContext {
    /// Embedded kernel (default in-process mode)
    Embedded {
        handle: Box<KernelHandle>,
        config: Box<LLMSpellConfig>,
    },
    /// Connected to external kernel
    Connected {
        handle: ClientHandle,
        address: String,
    },
}

impl std::fmt::Debug for ExecutionContext {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ExecutionContext::Embedded { config, .. } => f
                .debug_struct("ExecutionContext::Embedded")
                .field("handle", &"<KernelHandle>")
                .field("config", config)
                .finish(),
            ExecutionContext::Connected { address, .. } => f
                .debug_struct("ExecutionContext::Connected")
                .field("handle", &"<ClientHandle>")
                .field("address", address)
                .finish(),
        }
    }
}

impl ExecutionContext {
    /// Resolve execution context based on CLI flags and auto-detection
    ///
    /// # Errors
    ///
    /// Returns an error if kernel connection fails or config is invalid
    pub async fn resolve(
        connect: Option<String>,
        kernel: Option<String>,
        config: Option<PathBuf>,
        default_config: LLMSpellConfig,
    ) -> Result<Self> {
        match (connect, kernel, config) {
            // --connect has highest priority
            (Some(addr), _, _) => {
                info!("Connecting to external kernel at {addr}");
                let handle = connect_to_kernel(&addr).await?;
                Ok(ExecutionContext::Connected {
                    handle,
                    address: addr,
                })
            }

            // --kernel mode (connect by ID)
            (_, Some(kernel_id), _) => {
                info!("Connecting to kernel by ID: {kernel_id}");
                // Try to find kernel connection info by ID
                let addr = find_kernel_by_id(&kernel_id).await?;
                let handle = connect_to_kernel(&addr).await?;
                Ok(ExecutionContext::Connected {
                    handle,
                    address: addr,
                })
            }

            // --config mode (use specific config)
            (_, _, Some(config_path)) => {
                info!("Using config file: {}", config_path.display());
                let config = LLMSpellConfig::load_from_file(&config_path).await?;
                let handle = start_embedded_kernel(config.clone()).await?;
                Ok(ExecutionContext::Embedded {
                    handle: Box::new(handle),
                    config: Box::new(config),
                })
            }

            // Auto-detection mode
            (None, None, None) => {
                debug!("Auto-detection mode: looking for running kernels");

                // Try to find a running kernel
                if let Some(addr) = find_running_kernel().await? {
                    info!("Found running kernel at {addr}, connecting...");
                    let handle = connect_to_kernel(&addr).await?;
                    Ok(ExecutionContext::Connected {
                        handle,
                        address: addr,
                    })
                } else {
                    info!("No running kernel found, starting embedded mode");
                    let handle = start_embedded_kernel(default_config.clone()).await?;
                    Ok(ExecutionContext::Embedded {
                        handle: Box::new(handle),
                        config: Box::new(default_config),
                    })
                }
            }
        }
    }

    /// Get the kernel handle for execution
    pub fn kernel_handle_type(&self) -> &'static str {
        match self {
            ExecutionContext::Embedded { .. } => "KernelHandle",
            ExecutionContext::Connected { .. } => "ClientHandle",
        }
    }

    /// Check if this is an embedded context
    pub fn is_embedded(&self) -> bool {
        matches!(self, ExecutionContext::Embedded { .. })
    }

    /// Check if this is a connected context
    pub fn is_connected(&self) -> bool {
        matches!(self, ExecutionContext::Connected { .. })
    }

    /// Get connection info if connected
    pub fn connection_info(&self) -> Option<&str> {
        match self {
            ExecutionContext::Connected { address, .. } => Some(address),
            ExecutionContext::Embedded { .. } => None,
        }
    }
}

/// Find running kernel by auto-detection
async fn find_running_kernel() -> Result<Option<String>> {
    // Check for kernels in default locations
    let home = dirs::home_dir().ok_or_else(|| anyhow::anyhow!("Cannot find home directory"))?;
    let kernels_dir = home.join(".llmspell").join("kernels");

    if !kernels_dir.exists() {
        debug!(
            "Kernels directory does not exist: {}",
            kernels_dir.display()
        );
        return Ok(None);
    }

    // Look for active kernel connection files
    for entry in std::fs::read_dir(&kernels_dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.extension().and_then(|s| s.to_str()) == Some("json") {
            if let Ok(content) = std::fs::read_to_string(&path) {
                if let Ok(connection_info) = serde_json::from_str::<KernelConnectionInfo>(&content)
                {
                    // Try to ping the kernel to see if it's alive
                    if ping_kernel(&connection_info.address).await {
                        debug!("Found active kernel at {}", connection_info.address);
                        return Ok(Some(connection_info.address));
                    }
                }
            }
        }
    }

    Ok(None)
}

/// Find kernel by ID
async fn find_kernel_by_id(kernel_id: &str) -> Result<String> {
    let home = dirs::home_dir().ok_or_else(|| anyhow::anyhow!("Cannot find home directory"))?;
    let kernel_file = home
        .join(".llmspell")
        .join("kernels")
        .join(format!("{kernel_id}.json"));

    if !kernel_file.exists() {
        anyhow::bail!("Kernel '{kernel_id}' not found");
    }

    let content = std::fs::read_to_string(&kernel_file)?;
    let connection_info: KernelConnectionInfo = serde_json::from_str(&content)?;

    // Verify kernel is still alive
    if !ping_kernel(&connection_info.address).await {
        anyhow::bail!("Kernel '{kernel_id}' is not responding");
    }

    Ok(connection_info.address)
}

/// Ping kernel to check if it's alive
async fn ping_kernel(address: &str) -> bool {
    // Simple TCP connection test
    if let Some((host, port)) = address.split_once(':') {
        if let Ok(port) = port.parse::<u16>() {
            return tokio::net::TcpStream::connect((host, port)).await.is_ok();
        }
    }
    false
}

/// Kernel connection information
#[derive(Debug, serde::Deserialize, serde::Serialize)]
struct KernelConnectionInfo {
    id: String,
    address: String,
    created_at: String,
    protocol: String,
}
