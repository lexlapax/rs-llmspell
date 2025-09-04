//! Jupyter connection file format for LLMSpell kernel
//!
//! Implements standard Jupyter connection file specification with:
//! - 5 ZeroMQ channel ports (shell, iopub, stdin, control, heartbeat) 
//! - HMAC message authentication
//! - Support for TCP transport
//! - Dynamic port allocation and connection file generation

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::net::{TcpListener, SocketAddr};
use std::path::{Path, PathBuf};
use hex;

/// Jupyter connection file format
/// 
/// This follows the standard Jupyter connection file specification:
/// https://jupyter-client.readthedocs.io/en/stable/kernels.html#connection-files
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionInfo {
    /// Shell channel port (REQ-REP for execute requests)
    pub shell_port: u16,
    /// IOPub channel port (PUB for output publishing)  
    pub iopub_port: u16,
    /// Stdin channel port (REQ-REP for input requests)
    pub stdin_port: u16,
    /// Control channel port (REQ-REP for shutdown/interrupt/daemon)
    pub control_port: u16,
    /// Heartbeat channel port (REP for keepalive)
    pub hb_port: u16,
    /// IP address to bind to
    pub ip: String,
    /// HMAC key for message signing (hex-encoded)
    pub key: String,
    /// Transport protocol (always "tcp")
    pub transport: String,
    /// Message signature scheme (always "hmac-sha256")
    pub signature_scheme: String,
    /// Kernel name for identification
    pub kernel_name: String,
}

/// Configuration for connection file generation
#[derive(Debug, Clone)]
pub struct ConnectionConfig {
    /// IP address to bind to (default: "127.0.0.1")
    pub ip: String,
    /// Port range start for dynamic allocation (default: 9555)
    pub port_range_start: u16,
    /// Kernel name (default: "llmspell")
    pub kernel_name: String,
    /// Custom HMAC key (if None, generates random key)
    pub custom_key: Option<String>,
}

impl ConnectionInfo {
    /// Create new connection info with dynamic port allocation
    pub fn new(config: ConnectionConfig) -> Result<Self> {
        // Allocate 5 unique ports starting from port_range_start
        let ports = Self::allocate_ports(config.port_range_start, 5)?;
        
        // Generate HMAC key (32 random bytes, hex-encoded)
        let key = match config.custom_key {
            Some(key) => key,
            None => Self::generate_hmac_key(),
        };

        Ok(Self {
            shell_port: ports[0],
            iopub_port: ports[1], 
            stdin_port: ports[2],
            control_port: ports[3],
            hb_port: ports[4],
            ip: config.ip,
            key,
            transport: "tcp".to_string(),
            signature_scheme: "hmac-sha256".to_string(),
            kernel_name: config.kernel_name,
        })
    }

    /// Load connection info from JSON file
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let content = std::fs::read_to_string(path.as_ref())
            .with_context(|| format!("Failed to read connection file: {}", path.as_ref().display()))?;
        
        let connection_info: Self = serde_json::from_str(&content)
            .with_context(|| format!("Failed to parse connection file: {}", path.as_ref().display()))?;
        
        Ok(connection_info)
    }

    /// Save connection info to JSON file  
    pub fn save_to_file<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let json = serde_json::to_string_pretty(self)
            .context("Failed to serialize connection info")?;
        
        std::fs::write(path.as_ref(), json)
            .with_context(|| format!("Failed to write connection file: {}", path.as_ref().display()))?;
        
        Ok(())
    }

    /// Generate temporary connection file path
    pub fn temp_connection_file(kernel_id: &str) -> PathBuf {
        let filename = format!("kernel-{}.json", kernel_id);
        std::env::temp_dir().join(filename)
    }

    /// Get all port numbers as a vector
    pub fn all_ports(&self) -> Vec<u16> {
        vec![self.shell_port, self.iopub_port, self.stdin_port, self.control_port, self.hb_port]
    }

    /// Get ZeroMQ bind address for a specific port
    pub fn bind_address(&self, port: u16) -> String {
        format!("{}://{}:{}", self.transport, self.ip, port)
    }

    /// Get shell channel address
    pub fn shell_address(&self) -> String {
        self.bind_address(self.shell_port)
    }

    /// Get iopub channel address  
    pub fn iopub_address(&self) -> String {
        self.bind_address(self.iopub_port)
    }

    /// Get stdin channel address
    pub fn stdin_address(&self) -> String {
        self.bind_address(self.stdin_port)
    }

    /// Get control channel address
    pub fn control_address(&self) -> String {
        self.bind_address(self.control_port)
    }

    /// Get heartbeat channel address
    pub fn heartbeat_address(&self) -> String {
        self.bind_address(self.hb_port)
    }

    /// Validate connection info
    pub fn validate(&self) -> Result<()> {
        // Check ports are unique
        let ports = self.all_ports();
        let unique_ports: HashSet<_> = ports.iter().collect();
        if unique_ports.len() != ports.len() {
            return Err(anyhow::anyhow!("Connection file has duplicate ports"));
        }

        // Check ports are in valid range
        for &port in &ports {
            if port == 0 {
                return Err(anyhow::anyhow!("Connection file contains zero port"));
            }
        }

        // Check transport is supported
        if self.transport != "tcp" {
            return Err(anyhow::anyhow!("Unsupported transport: {}", self.transport));
        }

        // Check signature scheme is supported
        if self.signature_scheme != "hmac-sha256" {
            return Err(anyhow::anyhow!("Unsupported signature scheme: {}", self.signature_scheme));
        }

        // Check HMAC key is valid hex
        hex::decode(&self.key)
            .with_context(|| format!("Invalid hex key: {}", self.key))?;

        Ok(())
    }

    /// Allocate N unique available ports starting from start_port
    fn allocate_ports(start_port: u16, count: usize) -> Result<Vec<u16>> {
        let mut ports = Vec::new();
        let mut current_port = start_port;

        while ports.len() < count {
            if Self::is_port_available(current_port)? {
                ports.push(current_port);
            }
            current_port += 1;
            
            if current_port == 0 {
                return Err(anyhow::anyhow!("Exhausted port range without finding {} available ports", count));
            }
        }

        Ok(ports)
    }

    /// Check if a port is available for binding
    fn is_port_available(port: u16) -> Result<bool> {
        let addr: SocketAddr = format!("127.0.0.1:{}", port).parse()
            .context("Invalid socket address")?;
        
        match TcpListener::bind(addr) {
            Ok(_) => Ok(true),
            Err(_) => Ok(false),
        }
    }

    /// Generate random HMAC key (32 bytes, hex-encoded)
    fn generate_hmac_key() -> String {
        let key_bytes: [u8; 32] = rand::random();
        hex::encode(key_bytes)
    }
}

impl ConnectionConfig {
    /// Create default connection configuration
    pub fn new() -> Self {
        Self {
            ip: "127.0.0.1".to_string(),
            port_range_start: 9555,
            kernel_name: "llmspell".to_string(),
            custom_key: None,
        }
    }

    /// Set custom IP address
    pub fn with_ip<S: Into<String>>(mut self, ip: S) -> Self {
        self.ip = ip.into();
        self
    }

    /// Set custom port range start
    pub fn with_port_range(mut self, start_port: u16) -> Self {
        self.port_range_start = start_port;
        self
    }

    /// Set custom kernel name
    pub fn with_kernel_name<S: Into<String>>(mut self, name: S) -> Self {
        self.kernel_name = name.into();
        self
    }

    /// Set custom HMAC key
    pub fn with_key<S: Into<String>>(mut self, key: S) -> Self {
        self.custom_key = Some(key.into());
        self
    }
}

impl Default for ConnectionInfo {
    fn default() -> Self {
        // Generate a basic connection info for testing
        // In production, always use ConnectionInfo::new() with proper config
        Self {
            shell_port: 9555,
            iopub_port: 9556,
            stdin_port: 9557,
            control_port: 9558,
            hb_port: 9559,
            ip: "127.0.0.1".to_string(),
            key: "0000000000000000000000000000000000000000000000000000000000000000".to_string(),
            transport: "tcp".to_string(),
            signature_scheme: "hmac-sha256".to_string(),
            kernel_name: "llmspell".to_string(),
        }
    }
}

impl Default for ConnectionConfig {
    fn default() -> Self {
        Self::new()
    }
}