//! # Daemon Manager
//!
//! Implements the core daemon functionality using the double-fork technique
//! to properly detach from the controlling terminal.

use anyhow::{Context, Result};
use nix::unistd::{chdir, fork, setsid, ForkResult};
use serde::{Deserialize, Serialize};
use std::fs::{File, OpenOptions};
use std::os::unix::io::AsRawFd;
use std::path::PathBuf;
use std::process;
use tracing::{debug, info};

use super::pid::PidFile;

/// Configuration for the daemon process
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DaemonConfig {
    /// Whether to actually daemonize (fork to background)
    pub daemonize: bool,
    /// Path to the PID file
    pub pid_file: Option<PathBuf>,
    /// Working directory for the daemon
    pub working_dir: PathBuf,
    /// Path for stdout redirection
    pub stdout_path: Option<PathBuf>,
    /// Path for stderr redirection
    pub stderr_path: Option<PathBuf>,
    /// Whether to close stdin
    pub close_stdin: bool,
    /// File creation mask
    pub umask: Option<u32>,
}

impl Default for DaemonConfig {
    fn default() -> Self {
        Self {
            daemonize: true,
            pid_file: None,
            working_dir: PathBuf::from("/"),
            stdout_path: None,
            stderr_path: None,
            close_stdin: true,
            umask: Some(0o027),
        }
    }
}

/// Manages the daemonization process
pub struct DaemonManager {
    config: DaemonConfig,
    pid_file: Option<PidFile>,
}

impl DaemonManager {
    /// Creates a new daemon manager with the given configuration
    pub fn new(config: DaemonConfig) -> Self {
        Self {
            config,
            pid_file: None,
        }
    }

    /// Daemonizes the current process using the double-fork technique
    ///
    /// This method:
    /// 1. Performs the first fork to create a child process
    /// 2. Creates a new session with `setsid()`
    /// 3. Performs the second fork to ensure we can't acquire a controlling terminal
    /// 4. Changes to the configured working directory
    /// 5. Sets the umask
    /// 6. Redirects I/O as configured
    /// 7. Creates a PID file if configured
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` in the daemon process, or an error if daemonization fails.
    /// The parent process exits with status 0.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Fork operation fails
    /// - Failed to create new session
    /// - Failed to change working directory
    /// - Failed to redirect I/O
    /// - Failed to create PID file
    pub fn daemonize(&mut self) -> Result<()> {
        info!("Starting daemonization process");

        // First fork
        match unsafe { fork() }.context("First fork failed")? {
            ForkResult::Parent { .. } => {
                // Parent process exits successfully
                debug!("Parent process exiting after first fork");
                process::exit(0);
            }
            ForkResult::Child => {
                debug!("First fork successful, continuing in child");
            }
        }

        // Create new session and become session leader
        setsid().context("Failed to create new session")?;
        debug!("Created new session");

        // Second fork - ensures we can't acquire a controlling terminal
        match unsafe { fork() }.context("Second fork failed")? {
            ForkResult::Parent { .. } => {
                // First child exits
                debug!("First child exiting after second fork");
                process::exit(0);
            }
            ForkResult::Child => {
                debug!("Second fork successful, now truly daemonized");
            }
        }

        // Change working directory
        chdir(&self.config.working_dir).context("Failed to change working directory")?;
        info!("Changed working directory to {:?}", self.config.working_dir);

        // Set umask if configured
        if let Some(mask) = self.config.umask {
            unsafe {
                libc::umask(mask as libc::mode_t);
            }
            debug!("Set umask to {:o}", mask);
        }

        // Redirect I/O
        self.redirect_io()?;

        // Create PID file if configured
        if let Some(ref pid_path) = self.config.pid_file {
            let mut pid_file = PidFile::new(pid_path.clone());
            pid_file.write()?;
            self.pid_file = Some(pid_file);
            info!("Created PID file at {:?}", pid_path);
        }

        info!("Daemonization complete");
        Ok(())
    }

    /// Redirects standard I/O streams
    fn redirect_io(&self) -> Result<()> {
        // Close stdin if configured
        if self.config.close_stdin {
            let dev_null = File::open("/dev/null").context("Failed to open /dev/null")?;
            let null_fd = dev_null.as_raw_fd();
            unsafe {
                libc::dup2(null_fd, libc::STDIN_FILENO);
            }
            debug!("Redirected stdin to /dev/null");
        }

        // Redirect stdout
        if let Some(ref stdout_path) = self.config.stdout_path {
            let stdout_file = OpenOptions::new()
                .create(true)
                .append(true)
                .open(stdout_path)
                .context("Failed to open stdout file")?;
            let stdout_fd = stdout_file.as_raw_fd();
            unsafe {
                libc::dup2(stdout_fd, libc::STDOUT_FILENO);
            }
            info!("Redirected stdout to {:?}", stdout_path);
        } else {
            // Redirect to /dev/null if no path specified
            let dev_null = File::open("/dev/null").context("Failed to open /dev/null")?;
            let null_fd = dev_null.as_raw_fd();
            unsafe {
                libc::dup2(null_fd, libc::STDOUT_FILENO);
            }
        }

        // Redirect stderr
        if let Some(ref stderr_path) = self.config.stderr_path {
            let stderr_file = OpenOptions::new()
                .create(true)
                .append(true)
                .open(stderr_path)
                .context("Failed to open stderr file")?;
            let stderr_fd = stderr_file.as_raw_fd();
            unsafe {
                libc::dup2(stderr_fd, libc::STDERR_FILENO);
            }
            info!("Redirected stderr to {:?}", stderr_path);
        } else {
            // Redirect to /dev/null if no path specified
            let dev_null = File::open("/dev/null").context("Failed to open /dev/null")?;
            let null_fd = dev_null.as_raw_fd();
            unsafe {
                libc::dup2(null_fd, libc::STDERR_FILENO);
            }
        }

        Ok(())
    }

    /// Checks if the daemon is already running by checking the PID file
    ///
    /// # Errors
    ///
    /// Returns an error if failed to check the PID file
    pub fn is_running(&self) -> Result<bool> {
        if let Some(ref pid_path) = self.config.pid_file {
            let pid_file = PidFile::new(pid_path.clone());
            return pid_file.is_running();
        }
        Ok(false)
    }

    /// Cleans up daemon resources (e.g., removes PID file)
    ///
    /// # Errors
    ///
    /// Returns an error if failed to remove the PID file
    pub fn cleanup(&mut self) -> Result<()> {
        if let Some(mut pid_file) = self.pid_file.take() {
            pid_file.remove()?;
            info!("Removed PID file");
        }
        Ok(())
    }
}

impl Drop for DaemonManager {
    fn drop(&mut self) {
        // Best-effort cleanup on drop
        let _ = self.cleanup();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_daemon_config_default() {
        let config = DaemonConfig::default();
        assert_eq!(config.working_dir, PathBuf::from("/"));
        assert!(config.close_stdin);
        assert_eq!(config.umask, Some(0o027));
    }

    #[test]
    fn test_daemon_manager_creation() {
        let config = DaemonConfig::default();
        let manager = DaemonManager::new(config);
        assert!(manager.pid_file.is_none());
    }

    #[test]
    fn test_is_running_without_pid_file() {
        let config = DaemonConfig {
            pid_file: None,
            ..Default::default()
        };
        let manager = DaemonManager::new(config);
        assert!(!manager.is_running().unwrap());
    }

    #[test]
    fn test_daemon_config_with_paths() {
        let temp_dir = tempdir().unwrap();
        let config = DaemonConfig {
            daemonize: true,
            pid_file: Some(temp_dir.path().join("test.pid")),
            working_dir: temp_dir.path().to_path_buf(),
            stdout_path: Some(temp_dir.path().join("stdout.log")),
            stderr_path: Some(temp_dir.path().join("stderr.log")),
            close_stdin: true,
            umask: Some(0o022),
        };

        assert!(config.pid_file.is_some());
        assert!(config.stdout_path.is_some());
        assert!(config.stderr_path.is_some());
    }

    #[test]
    fn test_io_redirection_configuration() {
        let temp_dir = tempdir().unwrap();
        let stdout_path = temp_dir.path().join("stdout.log");
        let stderr_path = temp_dir.path().join("stderr.log");

        let config = DaemonConfig {
            daemonize: false,
            pid_file: None,
            working_dir: temp_dir.path().to_path_buf(),
            stdout_path: Some(stdout_path.clone()),
            stderr_path: Some(stderr_path.clone()),
            close_stdin: true,
            umask: None,
        };

        let manager = DaemonManager::new(config);

        // Verify paths are set correctly
        assert_eq!(manager.config.stdout_path.as_ref().unwrap(), &stdout_path);
        assert_eq!(manager.config.stderr_path.as_ref().unwrap(), &stderr_path);
        assert!(manager.config.close_stdin);
    }

    #[test]
    fn test_io_redirection_to_dev_null() {
        let config = DaemonConfig {
            daemonize: true,
            pid_file: None,
            working_dir: PathBuf::from("/tmp"),
            stdout_path: None, // Should redirect to /dev/null
            stderr_path: None, // Should redirect to /dev/null
            close_stdin: true,
            umask: None,
        };

        let manager = DaemonManager::new(config);

        // Verify no paths set means /dev/null redirection
        assert!(manager.config.stdout_path.is_none());
        assert!(manager.config.stderr_path.is_none());
        assert!(manager.config.close_stdin);
    }
}
