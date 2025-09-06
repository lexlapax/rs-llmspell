//! Connection file discovery for kernel-client connections
//!
//! Provides mechanisms for discovering running kernels through connection files.

use crate::connection::ConnectionInfo;
use anyhow::Result;
use glob::glob;
use std::path::PathBuf;

/// Kernel discovery service
pub struct KernelDiscovery {
    /// Directory to search for connection files
    connection_dir: PathBuf,
}

impl KernelDiscovery {
    /// Create a new kernel discovery service
    #[must_use]
    pub fn new() -> Self {
        Self {
            connection_dir: ConnectionInfo::connection_dir(),
        }
    }

    /// Create with custom connection directory
    #[must_use]
    pub const fn with_dir(dir: PathBuf) -> Self {
        Self {
            connection_dir: dir,
        }
    }

    /// Find all kernel connection files
    ///
    /// # Errors
    ///
    /// Returns an error if glob pattern matching fails
    pub fn find_connection_files(&self) -> Result<Vec<PathBuf>> {
        // Ensure directory exists
        if !self.connection_dir.exists() {
            return Ok(Vec::new());
        }

        let pattern = self
            .connection_dir
            .join("llmspell-kernel-*.json")
            .to_string_lossy()
            .to_string();

        let mut files = Vec::new();
        for path in glob(&pattern)?.flatten() {
            files.push(path);
        }

        tracing::debug!("Found {} connection files", files.len());
        Ok(files)
    }

    /// Discover all running kernels
    ///
    /// # Errors
    ///
    /// Returns an error if discovery fails
    pub async fn discover_kernels(&self) -> Result<Vec<ConnectionInfo>> {
        let files = self.find_connection_files()?;
        let mut kernels = Vec::new();

        for file in files {
            match ConnectionInfo::read_connection_file(&file).await {
                Ok(info) => {
                    tracing::debug!("Found kernel: {}", info.kernel_id);
                    kernels.push(info);
                }
                Err(e) => {
                    tracing::warn!("Failed to read connection file {}: {}", file.display(), e);
                    // Optionally clean up invalid file
                    let _ = tokio::fs::remove_file(&file).await;
                }
            }
        }

        Ok(kernels)
    }

    /// Find a specific kernel by ID
    ///
    /// # Errors
    ///
    /// Returns an error if discovery fails
    pub async fn find_kernel(&self, kernel_id: &str) -> Result<Option<ConnectionInfo>> {
        let kernels = self.discover_kernels().await?;
        Ok(kernels.into_iter().find(|k| k.kernel_id == kernel_id))
    }

    /// Check if a kernel is reachable using proper ZMQ heartbeat ping
    ///
    /// # Errors
    ///
    /// Returns an error if connection check fails
    pub async fn is_kernel_alive(info: &ConnectionInfo) -> Result<bool> {
        // Use async block to preserve async signature for API compatibility
        let info_clone = info.clone();
        tokio::task::spawn_blocking(move || Self::check_heartbeat(&info_clone))
            .await
            .map_err(|e| anyhow::anyhow!("Failed to check heartbeat: {}", e))?
    }

    /// Internal synchronous heartbeat check implementation
    fn check_heartbeat(info: &ConnectionInfo) -> Result<bool> {
        let addr = format!("tcp://{}:{}", info.ip, info.hb_port);
        let socket = Self::create_heartbeat_socket()?;

        if !Self::connect_socket(&socket, &addr) {
            return Ok(false);
        }

        if !Self::send_ping(&socket) {
            return Ok(false);
        }

        Ok(Self::check_ping_response(&socket, &addr))
    }

    /// Create and configure ZMQ socket for heartbeat
    fn create_heartbeat_socket() -> Result<zmq::Socket> {
        let context = zmq::Context::new();
        let socket = context.socket(zmq::REQ)?;
        socket.set_rcvtimeo(2000)?; // 2 seconds
        socket.set_sndtimeo(2000)?; // 2 seconds
        Ok(socket)
    }

    /// Connect socket to address
    fn connect_socket(socket: &zmq::Socket, addr: &str) -> bool {
        if let Err(e) = socket.connect(addr) {
            tracing::debug!("Failed to connect heartbeat socket to {}: {}", addr, e);
            false
        } else {
            true
        }
    }

    /// Send ping message
    fn send_ping(socket: &zmq::Socket) -> bool {
        let ping_data = b"ping";
        if let Err(e) = socket.send(&ping_data[..], 0) {
            tracing::debug!("Failed to send heartbeat ping: {}", e);
            false
        } else {
            true
        }
    }

    /// Check for ping response
    fn check_ping_response(socket: &zmq::Socket, addr: &str) -> bool {
        let ping_data = b"ping";
        match socket.recv_bytes(0) {
            Ok(response) if response == ping_data[..] => {
                tracing::trace!("Heartbeat successful for kernel at {}", addr);
                true
            }
            Ok(_) => {
                tracing::warn!("Unexpected heartbeat response from {}", addr);
                false
            }
            Err(e) => {
                tracing::debug!("No heartbeat response from {}: {}", addr, e);
                false
            }
        }
    }

    /// Clean up stale connection files
    ///
    /// # Errors
    ///
    /// Returns an error if cleanup fails
    pub async fn cleanup_stale_connections(&self) -> Result<Vec<String>> {
        let kernels = self.discover_kernels().await?;
        let mut removed = Vec::new();

        for info in kernels {
            if !Self::is_kernel_alive(&info).await? {
                tracing::info!("Removing stale connection for kernel {}", info.kernel_id);
                info.remove_connection_file().await?;
                removed.push(info.kernel_id);
            }
        }

        Ok(removed)
    }
}

impl Default for KernelDiscovery {
    fn default() -> Self {
        Self::new()
    }
}

/// Auto-discovery with fallback to starting new kernel
pub struct AutoDiscovery {
    discovery: KernelDiscovery,
}

impl AutoDiscovery {
    /// Create a new auto-discovery service
    #[must_use]
    pub fn new() -> Self {
        Self {
            discovery: KernelDiscovery::new(),
        }
    }

    /// Discover or start a kernel
    ///
    /// # Errors
    ///
    /// Returns an error if discovery or startup fails
    pub async fn discover_or_start(&self) -> Result<ConnectionInfo> {
        // First, try to find existing kernels
        let kernels = self.discovery.discover_kernels().await?;

        // Try to connect to existing kernels
        for info in kernels {
            if KernelDiscovery::is_kernel_alive(&info).await? {
                tracing::info!("Connecting to existing kernel: {}", info.kernel_id);
                return Ok(info);
            }
        }

        // No alive kernels found, start a new one
        tracing::info!("No existing kernels found, starting new kernel");
        Ok(Self::start_new_kernel())
    }

    /// Start a new kernel (placeholder - actual implementation will be in kernel module)
    fn start_new_kernel() -> ConnectionInfo {
        // This will be implemented to actually start a kernel process
        // For now, return a dummy connection info
        let kernel_id = uuid::Uuid::new_v4().to_string();

        // In real implementation:
        // 1. Start kernel process
        // 2. Wait for it to write connection file
        // 3. Read and return connection info

        ConnectionInfo::new(kernel_id, "127.0.0.1".to_string(), 9555)
    }
}

impl Default for AutoDiscovery {
    fn default() -> Self {
        Self::new()
    }
}
