//! Connection management for kernel-client communication
//!
//! Handles connection information, discovery files, and client connections.

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use tokio::fs;

/// Connection information for discovering and connecting to a kernel
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionInfo {
    /// Unique kernel identifier
    pub kernel_id: String,
    /// Transport protocol (e.g., "tcp")
    pub transport: String,
    /// IP address
    pub ip: String,
    /// Shell channel port
    pub shell_port: u16,
    /// `IOPub` channel port
    pub iopub_port: u16,
    /// Stdin channel port
    pub stdin_port: u16,
    /// Control channel port
    pub control_port: u16,
    /// Heartbeat channel port
    pub hb_port: u16,
    /// Authentication key
    pub key: String,
    /// Signature scheme (e.g., "hmac-sha256")
    pub signature_scheme: String,
}

impl ConnectionInfo {
    /// Create a new connection info
    #[must_use]
    pub fn new(kernel_id: String, ip: String, base_port: u16) -> Self {
        Self {
            kernel_id,
            transport: "tcp".to_string(),
            ip,
            shell_port: base_port,
            iopub_port: base_port + 1,
            stdin_port: base_port + 2,
            control_port: base_port + 3,
            hb_port: base_port + 4,
            key: uuid::Uuid::new_v4().to_string(),
            signature_scheme: "hmac-sha256".to_string(),
        }
    }

    /// Generate the connection file path
    #[must_use]
    pub fn connection_file_path(&self) -> PathBuf {
        let dir = Self::connection_dir();
        dir.join(format!("llmspell-kernel-{}.json", self.kernel_id))
    }

    /// Get the standard connection directory
    #[must_use]
    pub fn connection_dir() -> PathBuf {
        dirs::home_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join(".llmspell")
            .join("kernels")
    }

    /// Write connection file to disk
    ///
    /// # Errors
    ///
    /// Returns an error if file writing fails
    pub async fn write_connection_file(&self) -> Result<PathBuf> {
        let path = self.connection_file_path();

        // Ensure directory exists
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).await?;
        }

        // Write JSON file
        let json = serde_json::to_string_pretty(self)?;
        fs::write(&path, json).await?;

        tracing::info!("Written connection file: {}", path.display());
        Ok(path)
    }

    /// Read connection file from disk
    ///
    /// # Errors
    ///
    /// Returns an error if file reading or parsing fails
    pub async fn read_connection_file(path: &Path) -> Result<Self> {
        let json = fs::read_to_string(path).await?;
        let info = serde_json::from_str(&json)?;
        Ok(info)
    }

    /// Remove connection file from disk
    ///
    /// # Errors
    ///
    /// Returns an error if file removal fails
    pub async fn remove_connection_file(&self) -> Result<()> {
        let path = self.connection_file_path();
        if path.exists() {
            fs::remove_file(&path).await?;
            tracing::info!("Removed connection file: {}", path.display());
        }
        Ok(())
    }
}

/// Connection state for a kernel client
#[derive(Debug, Clone)]
pub struct KernelConnection {
    /// Connection information
    pub info: ConnectionInfo,
    /// Whether the connection is authenticated
    pub authenticated: bool,
    /// Connection timestamp
    pub connected_at: chrono::DateTime<chrono::Utc>,
}

impl KernelConnection {
    /// Create a new kernel connection
    #[must_use]
    pub fn new(info: ConnectionInfo) -> Self {
        Self {
            info,
            authenticated: false,
            connected_at: chrono::Utc::now(),
        }
    }

    /// Authenticate the connection
    pub fn authenticate(&mut self, key: &str) -> bool {
        if self.info.key == key {
            self.authenticated = true;
            true
        } else {
            false
        }
    }
}
