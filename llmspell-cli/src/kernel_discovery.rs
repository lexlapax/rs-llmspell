//! Kernel discovery infrastructure for finding and managing running kernels
//!
//! This module provides functionality to discover running LLMSpell kernels by scanning
//! connection files, verifying process status, and cleaning up stale files.

use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};
use tracing::info;

/// Status of a running kernel
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum KernelStatus {
    /// Kernel is healthy and responsive
    Healthy,
    /// Kernel is processing requests
    Busy,
    /// Kernel is idle and ready
    Idle,
    /// Kernel is shutting down
    ShuttingDown,
    /// Status unknown
    Unknown,
}

/// Information about a discovered kernel
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KernelInfo {
    /// Unique kernel identifier
    pub id: String,
    /// Process ID of the kernel
    pub pid: u32,
    /// Base port for kernel communication
    pub port: u16,
    /// Path to the connection file
    pub connection_file: PathBuf,
    /// Optional path to PID file
    pub pid_file: Option<PathBuf>,
    /// Optional path to log file
    pub log_file: Option<PathBuf>,
    /// Current kernel status
    pub status: KernelStatus,
    /// Time when kernel was started
    pub start_time: Option<std::time::SystemTime>,
}

/// Connection information from Jupyter connection file
#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct ConnectionInfo {
    transport: String,
    ip: String,
    shell_port: u16,
    iopub_port: u16,
    stdin_port: u16,
    control_port: u16,
    hb_port: u16,
    key: String,
    signature_scheme: String,
    kernel_name: String,
    #[serde(default)]
    kernel_id: Option<String>,
    #[serde(default)]
    pid: Option<u32>,
}

/// Discovers all running kernels on the system
///
/// Scans standard kernel directories for connection files and returns
/// information about all running kernels. Automatically cleans up stale
/// connection files for processes that are no longer running.
pub fn discover_kernels() -> Result<Vec<KernelInfo>> {
    let kernel_dirs = vec![
        // Primary location
        dirs::home_dir().map(|h| h.join(".llmspell/kernels")),
        // Runtime directory
        dirs::runtime_dir().map(|r| r.join("llmspell/kernels")),
        // Fallback location
        Some(PathBuf::from("/tmp/llmspell/kernels")),
    ];

    let mut kernels = Vec::new();
    let mut seen_pids = HashSet::new();

    for dir_opt in kernel_dirs {
        if let Some(dir) = dir_opt {
            if dir.exists() {
                scan_directory(&dir, &mut kernels, &mut seen_pids)?;
            }
        }
    }

    Ok(kernels)
}

/// Scan a directory for kernel connection files
fn scan_directory(
    dir: &Path,
    kernels: &mut Vec<KernelInfo>,
    seen_pids: &mut HashSet<u32>,
) -> Result<()> {
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();

        // Only process JSON connection files
        if path.extension().map_or(false, |e| e == "json") {
            match parse_kernel_file(&path) {
                Ok(kernel) => {
                    // Check if we've already seen this PID (avoid duplicates)
                    if !seen_pids.contains(&kernel.pid) {
                        if is_process_alive(kernel.pid) {
                            seen_pids.insert(kernel.pid);
                            kernels.push(kernel);
                        } else {
                            // Clean up stale connection file
                            info!("Cleaning stale connection file: {}", path.display());
                            let _ = fs::remove_file(&path);
                        }
                    }
                }
                Err(e) => {
                    // Log but continue scanning
                    tracing::debug!(
                        "Failed to parse connection file {}: {}",
                        path.display(),
                        e
                    );
                }
            }
        }
    }
    Ok(())
}

/// Parse a kernel connection file and extract kernel information
fn parse_kernel_file(path: &Path) -> Result<KernelInfo> {
    let content = fs::read_to_string(path)?;
    let conn_info: ConnectionInfo = serde_json::from_str(&content)?;

    // Extract kernel ID from filename if not in JSON
    let kernel_id = conn_info.kernel_id.unwrap_or_else(|| {
        path.file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown")
            .to_string()
            .replace("kernel-", "")
    });

    // Try to find associated PID file
    let pid_file = find_pid_file(&kernel_id);

    // Get PID from file or connection info
    let pid = if let Some(ref pf) = pid_file {
        read_pid_from_file(pf).ok()
    } else {
        conn_info.pid
    }
    .ok_or_else(|| anyhow!("No PID found for kernel {}", kernel_id))?;

    // Try to find log file
    let log_file = find_log_file(&kernel_id);

    // Get file creation time as start time
    let start_time = fs::metadata(path)
        .ok()
        .and_then(|m| m.created().ok());

    Ok(KernelInfo {
        id: kernel_id,
        pid,
        port: conn_info.shell_port,
        connection_file: path.to_path_buf(),
        pid_file,
        log_file,
        status: KernelStatus::Unknown,
        start_time,
    })
}

/// Check if a process is alive using kill(pid, 0)
pub fn is_process_alive(pid: u32) -> bool {
    // kill(pid, 0) checks if process exists without sending signal
    // Returns 0 if process exists, -1 with errno=ESRCH if not
    unsafe { libc::kill(pid as i32, 0) == 0 }
}

/// Read PID from a file
fn read_pid_from_file(path: &Path) -> Result<u32> {
    let content = fs::read_to_string(path)?;
    content
        .trim()
        .parse()
        .map_err(|e| anyhow!("Invalid PID in file {}: {}", path.display(), e))
}

/// Find PID file for a kernel ID
fn find_pid_file(kernel_id: &str) -> Option<PathBuf> {
    let candidates = vec![
        dirs::runtime_dir().map(|r| r.join(format!("llmspell-{}.pid", kernel_id))),
        dirs::home_dir().map(|h| h.join(format!(".llmspell/{}.pid", kernel_id))),
        Some(PathBuf::from(format!("/tmp/llmspell-{}.pid", kernel_id))),
        dirs::home_dir().map(|h| h.join(format!(".llmspell/kernel-{}.pid", kernel_id))),
    ];

    candidates
        .into_iter()
        .flatten()
        .find(|p| p.exists())
}

/// Find log file for a kernel ID
fn find_log_file(kernel_id: &str) -> Option<PathBuf> {
    let candidates = vec![
        dirs::state_dir().map(|s| s.join(format!("llmspell/{}.log", kernel_id))),
        dirs::home_dir().map(|h| h.join(format!(".llmspell/logs/{}.log", kernel_id))),
        Some(PathBuf::from(format!("/tmp/llmspell-{}.log", kernel_id))),
        dirs::home_dir().map(|h| h.join(format!(".llmspell/kernel-{}.log", kernel_id))),
    ];

    candidates
        .into_iter()
        .flatten()
        .find(|p| p.exists())
}

/// Find a kernel by its ID
pub fn find_kernel_by_id(id: &str) -> Result<KernelInfo> {
    discover_kernels()?
        .into_iter()
        .find(|k| k.id == id)
        .ok_or_else(|| anyhow!("Kernel '{}' not found", id))
}

/// Find a kernel by port
pub fn find_kernel_by_port(port: u16) -> Result<KernelInfo> {
    discover_kernels()?
        .into_iter()
        .find(|k| k.port == port)
        .ok_or_else(|| anyhow!("No kernel found on port {}", port))
}

/// Get all healthy kernels
pub fn get_healthy_kernels() -> Result<Vec<KernelInfo>> {
    Ok(discover_kernels()?
        .into_iter()
        .filter(|k| matches!(k.status, KernelStatus::Healthy | KernelStatus::Idle))
        .collect())
}

/// Find and clean up all stale kernel files
pub fn cleanup_stale_kernels() -> Result<usize> {
    let kernel_dirs = vec![
        dirs::home_dir().map(|h| h.join(".llmspell/kernels")),
        dirs::runtime_dir().map(|r| r.join("llmspell/kernels")),
        Some(PathBuf::from("/tmp/llmspell/kernels")),
    ];

    let mut cleaned = 0;

    for dir_opt in kernel_dirs {
        if let Some(dir) = dir_opt {
            if dir.exists() {
                cleaned += cleanup_directory(&dir)?;
            }
        }
    }

    Ok(cleaned)
}

/// Clean up stale files in a directory
fn cleanup_directory(dir: &Path) -> Result<usize> {
    let mut cleaned = 0;

    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.extension().map_or(false, |e| e == "json") {
            // Try to parse the connection file
            if let Ok(conn_info) = parse_kernel_file(&path) {
                if !is_process_alive(conn_info.pid) {
                    info!("Removing stale connection file: {}", path.display());
                    fs::remove_file(&path)?;
                    cleaned += 1;

                    // Also try to clean up associated PID and log files
                    if let Some(pid_file) = conn_info.pid_file {
                        let _ = fs::remove_file(pid_file);
                    }
                    if let Some(log_file) = conn_info.log_file {
                        // Don't delete logs - they might be useful for debugging
                        tracing::debug!("Keeping log file for debugging: {}", log_file.display());
                    }
                }
            }
        }
    }

    Ok(cleaned)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::PathBuf;
    use tempfile::TempDir;

    #[test]
    fn test_process_alive_check() {
        // Current process should be alive
        let current_pid = std::process::id();
        assert!(is_process_alive(current_pid));

        // Very high PID unlikely to exist
        assert!(!is_process_alive(999999999));
    }

    #[test]
    fn test_kernel_status_serialization() {
        let status = KernelStatus::Healthy;
        let json = serde_json::to_string(&status).unwrap();
        let deserialized: KernelStatus = serde_json::from_str(&json).unwrap();
        assert_eq!(status, deserialized);
    }

    #[test]
    fn test_kernel_info_serialization() {
        let info = KernelInfo {
            id: "test-kernel".to_string(),
            pid: 12345,
            port: 9555,
            connection_file: PathBuf::from("/tmp/kernel.json"),
            pid_file: Some(PathBuf::from("/tmp/kernel.pid")),
            log_file: None,
            status: KernelStatus::Idle,
            start_time: Some(std::time::SystemTime::now()),
        };

        let json = serde_json::to_string(&info).unwrap();
        let deserialized: KernelInfo = serde_json::from_str(&json).unwrap();
        assert_eq!(info.id, deserialized.id);
        assert_eq!(info.pid, deserialized.pid);
        assert_eq!(info.port, deserialized.port);
    }

    #[test]
    fn test_connection_file_parsing() {
        let temp_dir = TempDir::new().unwrap();
        let conn_file = temp_dir.path().join("kernel-test123.json");

        let conn_data = r#"{
            "transport": "tcp",
            "ip": "127.0.0.1",
            "shell_port": 9555,
            "iopub_port": 9556,
            "stdin_port": 9557,
            "control_port": 9558,
            "hb_port": 9559,
            "key": "test-key",
            "signature_scheme": "hmac-sha256",
            "kernel_name": "llmspell",
            "pid": 12345
        }"#;

        fs::write(&conn_file, conn_data).unwrap();

        let kernel = parse_kernel_file(&conn_file).unwrap();
        assert_eq!(kernel.id, "test123");
        assert_eq!(kernel.pid, 12345);
        assert_eq!(kernel.port, 9555);
    }

    #[test]
    fn test_kernel_discovery_with_no_kernels() {
        // Should not panic even if no kernels found
        let kernels = discover_kernels();
        assert!(kernels.is_ok());
    }

    #[test]
    fn test_cleanup_stale_kernels() {
        // Should not panic even if no stale kernels
        let result = cleanup_stale_kernels();
        assert!(result.is_ok());
    }
}