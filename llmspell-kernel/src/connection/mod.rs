//! Jupyter connection file management
//!
//! Handles creation and management of connection files that allow
//! Jupyter clients to discover and connect to running kernels.

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use tracing::{debug, info, warn};

/// Jupyter kernel connection information
///
/// This structure matches the Jupyter connection file format
/// and contains all information needed for clients to connect.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionInfo {
    /// Transport type (always "tcp" for `ZeroMQ`)
    pub transport: String,
    /// IP address to connect to
    pub ip: String,
    /// Shell channel port (REQ-REP)
    pub shell_port: u16,
    /// `IOPub` channel port (SUB)
    pub iopub_port: u16,
    /// Stdin channel port (REQ-REP)
    pub stdin_port: u16,
    /// Control channel port (REQ-REP)
    pub control_port: u16,
    /// Heartbeat channel port (REQ-REP)
    pub hb_port: u16,
    /// HMAC key for message signing
    pub key: String,
    /// Signature scheme (usually "hmac-sha256")
    pub signature_scheme: String,
    /// Kernel name for identification
    pub kernel_name: String,
}

impl ConnectionInfo {
    /// Create new connection info with default settings
    pub fn new(base_port: u16) -> Self {
        Self {
            transport: "tcp".to_string(),
            ip: "127.0.0.1".to_string(),
            shell_port: base_port,
            iopub_port: base_port + 1,
            stdin_port: base_port + 2,
            control_port: base_port + 3,
            hb_port: base_port + 4,
            key: Self::generate_key(),
            signature_scheme: "hmac-sha256".to_string(),
            kernel_name: "llmspell".to_string(),
        }
    }

    /// Generate a random HMAC key
    fn generate_key() -> String {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        let key: Vec<u8> = (0..32).map(|_| rng.gen()).collect();
        hex::encode(key)
    }
}

/// Manages kernel connection files
pub struct ConnectionFileManager {
    /// Path to the connection file
    file_path: Option<PathBuf>,
    /// Connection info
    info: ConnectionInfo,
    /// Kernel ID for file naming
    kernel_id: String,
}

impl ConnectionFileManager {
    /// Create a new connection file manager
    pub fn new(kernel_id: String, base_port: u16) -> Self {
        Self {
            file_path: None,
            info: ConnectionInfo::new(base_port),
            kernel_id,
        }
    }

    /// Write connection file to disk
    ///
    /// Creates a JSON file containing connection information for Jupyter clients.
    /// The file is written to `~/.llmspell/kernels/kernel-{id}.json` by default.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Failed to create directory
    /// - Failed to write file
    /// - Failed to serialize connection info
    pub fn write(&mut self) -> Result<PathBuf> {
        let connection_dir = Self::get_connection_dir();

        // Ensure directory exists
        fs::create_dir_all(&connection_dir).with_context(|| {
            format!(
                "Failed to create connection directory: {}",
                connection_dir.display()
            )
        })?;

        // Generate filename
        let file_name = format!("kernel-{}.json", self.kernel_id);
        let file_path = connection_dir.join(file_name);

        // Serialize connection info
        let json = serde_json::to_string_pretty(&self.info)
            .context("Failed to serialize connection info")?;

        // Write to file
        fs::write(&file_path, json)
            .with_context(|| format!("Failed to write connection file: {}", file_path.display()))?;

        info!("Created connection file at {}", file_path.display());
        self.file_path = Some(file_path.clone());

        Ok(file_path)
    }

    /// Get the connection directory
    fn get_connection_dir() -> PathBuf {
        // Try standard locations in order
        if let Some(home) = dirs::home_dir() {
            let llmspell_dir = home.join(".llmspell").join("kernels");
            return llmspell_dir;
        }

        // Fallback to runtime dir
        if let Some(runtime) = dirs::runtime_dir() {
            let llmspell_dir = runtime.join("llmspell").join("kernels");
            return llmspell_dir;
        }

        // Last resort: /tmp
        PathBuf::from("/tmp/llmspell/kernels")
    }

    /// Remove connection file
    ///
    /// # Errors
    ///
    /// Returns an error if the file exists but cannot be removed
    pub fn remove(&mut self) -> Result<()> {
        if let Some(ref path) = self.file_path {
            if path.exists() {
                fs::remove_file(path).with_context(|| {
                    format!("Failed to remove connection file: {}", path.display())
                })?;
                info!("Removed connection file: {}", path.display());
            } else {
                debug!("Connection file already removed: {}", path.display());
            }
            self.file_path = None;
        }
        Ok(())
    }

    /// Get the connection info
    pub fn info(&self) -> &ConnectionInfo {
        &self.info
    }

    /// Get the file path if written
    pub fn file_path(&self) -> Option<&Path> {
        self.file_path.as_deref()
    }

    /// Update connection info with actual bound ports
    ///
    /// Called after transport binding to update with real port numbers
    pub fn update_ports(&mut self, shell: u16, iopub: u16, stdin: u16, control: u16, hb: u16) {
        self.info.shell_port = shell;
        self.info.iopub_port = iopub;
        self.info.stdin_port = stdin;
        self.info.control_port = control;
        self.info.hb_port = hb;
    }

    /// Set the HMAC key
    pub fn set_key(&mut self, key: String) {
        self.info.key = key;
    }
}

impl Drop for ConnectionFileManager {
    fn drop(&mut self) {
        // Best-effort cleanup on drop
        if let Err(e) = self.remove() {
            warn!("Failed to remove connection file on drop: {}", e);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_connection_info_creation() {
        let info = ConnectionInfo::new(5555);
        assert_eq!(info.transport, "tcp");
        assert_eq!(info.ip, "127.0.0.1");
        assert_eq!(info.shell_port, 5555);
        assert_eq!(info.iopub_port, 5556);
        assert_eq!(info.stdin_port, 5557);
        assert_eq!(info.control_port, 5558);
        assert_eq!(info.hb_port, 5559);
        assert!(!info.key.is_empty());
        assert_eq!(info.signature_scheme, "hmac-sha256");
        assert_eq!(info.kernel_name, "llmspell");
    }

    #[test]
    fn test_connection_file_write_and_remove() {
        let temp_dir = tempdir().unwrap();
        std::env::set_var("HOME", temp_dir.path());

        let mut manager = ConnectionFileManager::new("test-kernel".to_string(), 6000);

        // Write connection file
        let file_path = manager.write().unwrap();
        assert!(file_path.exists());

        // Verify content
        let content = fs::read_to_string(&file_path).unwrap();
        let parsed: ConnectionInfo = serde_json::from_str(&content).unwrap();
        assert_eq!(parsed.shell_port, 6000);

        // Remove file
        manager.remove().unwrap();
        assert!(!file_path.exists());
    }

    #[test]
    fn test_update_ports() {
        let mut manager = ConnectionFileManager::new("test-kernel".to_string(), 7000);
        manager.update_ports(7001, 7002, 7003, 7004, 7005);

        assert_eq!(manager.info().shell_port, 7001);
        assert_eq!(manager.info().iopub_port, 7002);
        assert_eq!(manager.info().stdin_port, 7003);
        assert_eq!(manager.info().control_port, 7004);
        assert_eq!(manager.info().hb_port, 7005);
    }

    #[test]
    fn test_serialization() {
        let info = ConnectionInfo::new(8000);
        let json = serde_json::to_string(&info).unwrap();
        let parsed: ConnectionInfo = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed.shell_port, info.shell_port);
        assert_eq!(parsed.key, info.key);
        assert_eq!(parsed.kernel_name, info.kernel_name);
    }
}
