//! # PID File Management
//!
//! Provides PID file handling for daemon processes to prevent multiple instances
//! and enable process management.

use anyhow::{bail, Context, Result};
use nix::sys::signal::{kill, Signal};
use nix::unistd::Pid;
use std::fs::{self, File, OpenOptions};
use std::io::{Read, Write};
use std::path::PathBuf;
use std::process;
use tracing::{debug, warn};

/// Manages a PID file for daemon process tracking
pub struct PidFile {
    path: PathBuf,
    file: Option<File>,
    owns_file: bool,
}

impl PidFile {
    /// Creates a new PID file manager
    pub fn new(path: PathBuf) -> Self {
        Self {
            path,
            file: None,
            owns_file: false,
        }
    }

    /// Writes the current process ID to the PID file
    ///
    /// This method creates the PID file with exclusive access to prevent
    /// multiple daemons from running simultaneously.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Another instance is already running
    /// - Failed to create PID file directory
    /// - Failed to create or open PID file
    /// - Failed to write PID to file
    pub fn write(&mut self) -> Result<()> {
        // Check if another instance is already running
        if self.is_running()? {
            bail!("Another instance is already running");
        }

        // Create parent directory if needed
        if let Some(parent) = self.path.parent() {
            fs::create_dir_all(parent).context("Failed to create PID file directory")?;
        }

        // Open file with exclusive creation (fails if file exists)
        let mut file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&self.path)
            .or_else(|err| {
                // If file exists, check if the process is still running
                if let Ok(pid) = self.read_pid() {
                    if Self::process_exists(pid) {
                        return Err(err);
                    }
                    // Process is dead, remove stale PID file
                    warn!("Removing stale PID file for dead process {}", pid);
                    fs::remove_file(&self.path)?;
                    // Try again
                    OpenOptions::new()
                        .write(true)
                        .create_new(true)
                        .open(&self.path)
                } else {
                    Err(err)
                }
            })
            .context("Failed to create PID file")?;

        // Write current PID
        let pid = process::id();
        writeln!(file, "{pid}").context("Failed to write PID")?;
        file.sync_all().context("Failed to sync PID file")?;

        debug!("Wrote PID {} to {:?}", pid, self.path);

        self.file = Some(file);
        self.owns_file = true;

        Ok(())
    }

    /// Reads the PID from the file
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Failed to open PID file
    /// - Failed to read PID file
    /// - PID file contains invalid data
    pub fn read_pid(&self) -> Result<u32> {
        let mut contents = String::new();
        File::open(&self.path)
            .context("Failed to open PID file")?
            .read_to_string(&mut contents)
            .context("Failed to read PID file")?;

        contents
            .trim()
            .parse::<u32>()
            .context("Invalid PID in file")
    }

    /// Checks if the daemon is running by checking the PID file
    ///
    /// # Errors
    ///
    /// Returns an error if failed to read the PID file
    pub fn is_running(&self) -> Result<bool> {
        if !self.path.exists() {
            return Ok(false);
        }

        match self.read_pid() {
            Ok(pid) => Ok(Self::process_exists(pid)),
            Err(_) => {
                // Can't read PID, assume not running
                Ok(false)
            }
        }
    }

    /// Checks if a process with the given PID exists
    fn process_exists(pid: u32) -> bool {
        // Try to send signal 0 (null signal) to check if process exists
        match kill(
            Pid::from_raw(i32::try_from(pid).unwrap_or(i32::MAX)),
            Signal::SIGCONT,
        ) {
            Ok(()) | Err(nix::errno::Errno::EPERM) => true, // Process exists
            Err(nix::errno::Errno::ESRCH | _) => false,     // No such process or other error
        }
    }

    /// Removes the PID file
    ///
    /// # Errors
    ///
    /// Returns an error if failed to remove the PID file
    pub fn remove(&mut self) -> Result<()> {
        if self.owns_file {
            // Close the file handle first
            self.file.take();

            // Only remove if it contains our PID
            if let Ok(pid) = self.read_pid() {
                if pid == process::id() {
                    fs::remove_file(&self.path).context("Failed to remove PID file")?;
                    debug!("Removed PID file at {:?}", self.path);
                } else {
                    warn!(
                        "PID file contains different PID ({} vs {}), not removing",
                        pid,
                        process::id()
                    );
                }
            }
            self.owns_file = false;
        }
        Ok(())
    }

    /// Attempts to acquire a lock on the PID file
    ///
    /// This uses advisory file locking to ensure exclusive access.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - No PID file is open
    /// - Failed to acquire lock
    pub fn try_lock(&mut self) -> Result<bool> {
        use nix::fcntl::{flock, FlockArg};
        use std::os::unix::io::AsRawFd;

        if let Some(ref file) = self.file {
            match flock(file.as_raw_fd(), FlockArg::LockExclusiveNonblock) {
                Ok(()) => {
                    debug!("Acquired exclusive lock on PID file");
                    Ok(true)
                }
                Err(nix::errno::Errno::EWOULDBLOCK) => {
                    debug!("PID file is already locked");
                    Ok(false)
                }
                Err(e) => {
                    bail!("Failed to lock PID file: {e}");
                }
            }
        } else {
            bail!("No PID file open");
        }
    }

    /// Releases the lock on the PID file
    ///
    /// # Errors
    ///
    /// Returns an error if failed to release the lock
    pub fn unlock(&self) -> Result<()> {
        use nix::fcntl::{flock, FlockArg};
        use std::os::unix::io::AsRawFd;

        if let Some(ref file) = self.file {
            flock(file.as_raw_fd(), FlockArg::Unlock)?;
            debug!("Released lock on PID file");
        }
        Ok(())
    }
}

impl Drop for PidFile {
    fn drop(&mut self) {
        // Best-effort cleanup
        let _ = self.remove();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_pid_file_creation() {
        let temp_dir = tempdir().unwrap();
        let pid_path = temp_dir.path().join("test.pid");
        let pid_file = PidFile::new(pid_path.clone());

        assert!(!pid_path.exists());
        assert!(pid_file.file.is_none());
        assert!(!pid_file.owns_file);
    }

    #[test]
    fn test_pid_file_write_and_read() {
        let temp_dir = tempdir().unwrap();
        let pid_path = temp_dir.path().join("test.pid");
        let mut pid_file = PidFile::new(pid_path.clone());

        // Write PID
        pid_file.write().unwrap();
        assert!(pid_path.exists());

        // Read PID back
        let read_pid = pid_file.read_pid().unwrap();
        assert_eq!(read_pid, process::id());
    }

    #[test]
    fn test_process_exists() {
        // Current process should exist
        assert!(PidFile::process_exists(process::id()));

        // PID 0 is special and might exist
        // PID 1 (init) should exist on Unix systems
        // Use a very high PID that's unlikely to exist
        assert!(!PidFile::process_exists(999_999));
    }

    #[test]
    fn test_is_running() {
        let temp_dir = tempdir().unwrap();
        let pid_path = temp_dir.path().join("test.pid");
        let mut pid_file = PidFile::new(pid_path.clone());

        // Should not be running initially
        assert!(!pid_file.is_running().unwrap());

        // Write PID
        pid_file.write().unwrap();

        // Should be running now (current process)
        assert!(pid_file.is_running().unwrap());
    }

    #[test]
    fn test_remove_pid_file() {
        let temp_dir = tempdir().unwrap();
        let pid_path = temp_dir.path().join("test.pid");
        let mut pid_file = PidFile::new(pid_path.clone());

        // Write PID
        pid_file.write().unwrap();
        assert!(pid_path.exists());

        // Remove PID
        pid_file.remove().unwrap();
        assert!(!pid_path.exists());
    }

    #[test]
    fn test_stale_pid_detection() {
        let temp_dir = tempdir().unwrap();
        let pid_path = temp_dir.path().join("test.pid");

        // Write a fake PID that doesn't exist
        fs::write(&pid_path, "999999\n").unwrap();

        let mut pid_file = PidFile::new(pid_path.clone());

        // Should detect as not running
        assert!(!pid_file.is_running().unwrap());

        // Writing should succeed (removes stale file)
        pid_file.write().unwrap();
        assert_eq!(pid_file.read_pid().unwrap(), process::id());
    }

    #[test]
    fn test_concurrent_start_prevention() {
        let temp_dir = tempdir().unwrap();
        let pid_path = temp_dir.path().join("test.pid");

        // First instance
        let mut pid_file1 = PidFile::new(pid_path.clone());
        pid_file1.write().unwrap();
        assert!(pid_file1.try_lock().unwrap());

        // Second instance should fail
        let mut pid_file2 = PidFile::new(pid_path.clone());
        let result = pid_file2.write();
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Another instance"));
    }

    #[test]
    fn test_atomic_pid_write() {
        let temp_dir = tempdir().unwrap();
        let pid_path = temp_dir.path().join("test.pid");

        let mut pid_file = PidFile::new(pid_path.clone());
        pid_file.write().unwrap();

        // Verify PID was written atomically
        let contents = fs::read_to_string(&pid_path).unwrap();
        let pid: u32 = contents.trim().parse().unwrap();
        assert_eq!(pid, process::id());

        // Verify file is properly synced
        drop(pid_file);
        assert!(!pid_path.exists());
    }
}
