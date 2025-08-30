//! Connection file discovery for kernel-client connections
//!
//! Provides mechanisms for discovering running kernels through connection files.

use std::path::PathBuf;
use glob::glob;
use anyhow::Result;
use crate::connection::ConnectionInfo;

/// Kernel discovery service
pub struct KernelDiscovery {
    /// Directory to search for connection files
    connection_dir: PathBuf,
}

impl KernelDiscovery {
    /// Create a new kernel discovery service
    pub fn new() -> Self {
        Self {
            connection_dir: ConnectionInfo::connection_dir(),
        }
    }
    
    /// Create with custom connection directory
    pub fn with_dir(dir: PathBuf) -> Self {
        Self {
            connection_dir: dir,
        }
    }
    
    /// Find all kernel connection files
    pub async fn find_connection_files(&self) -> Result<Vec<PathBuf>> {
        // Ensure directory exists
        if !self.connection_dir.exists() {
            return Ok(Vec::new());
        }
        
        let pattern = self.connection_dir
            .join("llmspell-kernel-*.json")
            .to_string_lossy()
            .to_string();
        
        let mut files = Vec::new();
        for entry in glob(&pattern)? {
            if let Ok(path) = entry {
                files.push(path);
            }
        }
        
        tracing::debug!("Found {} connection files", files.len());
        Ok(files)
    }
    
    /// Discover all running kernels
    pub async fn discover_kernels(&self) -> Result<Vec<ConnectionInfo>> {
        let files = self.find_connection_files().await?;
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
    pub async fn find_kernel(&self, kernel_id: &str) -> Result<Option<ConnectionInfo>> {
        let kernels = self.discover_kernels().await?;
        Ok(kernels.into_iter().find(|k| k.kernel_id == kernel_id))
    }
    
    /// Check if a kernel is reachable
    pub async fn is_kernel_alive(info: &ConnectionInfo) -> Result<bool> {
        use tokio::net::TcpStream;
        use tokio::time::{timeout, Duration};
        
        // Try to connect to heartbeat channel
        let addr = format!("{}:{}", info.ip, info.hb_port);
        
        match timeout(Duration::from_secs(2), TcpStream::connect(&addr)).await {
            Ok(Ok(_stream)) => {
                // Connection successful, kernel is alive
                Ok(true)
            }
            _ => {
                // Connection failed or timed out
                Ok(false)
            }
        }
    }
    
    /// Clean up stale connection files
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
    pub fn new() -> Self {
        Self {
            discovery: KernelDiscovery::new(),
        }
    }
    
    /// Discover or start a kernel
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
        self.start_new_kernel().await
    }
    
    /// Start a new kernel (placeholder - actual implementation will be in kernel module)
    async fn start_new_kernel(&self) -> Result<ConnectionInfo> {
        // This will be implemented to actually start a kernel process
        // For now, return a dummy connection info
        let kernel_id = uuid::Uuid::new_v4().to_string();
        let info = ConnectionInfo::new(kernel_id, "127.0.0.1".to_string(), 5555);
        
        // In real implementation:
        // 1. Start kernel process
        // 2. Wait for it to write connection file
        // 3. Read and return connection info
        
        Ok(info)
    }
}

impl Default for AutoDiscovery {
    fn default() -> Self {
        Self::new()
    }
}