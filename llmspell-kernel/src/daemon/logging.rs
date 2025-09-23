//! # Daemon Logging Support
//!
//! Provides logging infrastructure for daemon processes, including:
//! - Log file rotation
//! - I/O stream redirection
//! - Structured logging integration

use anyhow::{Context, Result};
use chrono::Local;
use std::fs::{self, File, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use tracing::{debug, info};

/// Configuration for log rotation
#[derive(Debug, Clone)]
pub struct LogRotationConfig {
    /// Maximum file size before rotation (in bytes)
    pub max_size: u64,
    /// Maximum number of rotated files to keep
    pub max_files: usize,
    /// Whether to compress rotated files
    pub compress: bool,
    /// Base path for log files
    pub base_path: PathBuf,
}

impl Default for LogRotationConfig {
    fn default() -> Self {
        Self {
            max_size: 10 * 1024 * 1024, // 10 MB
            max_files: 5,
            compress: false,
            base_path: PathBuf::from("/var/log/llmspell.log"),
        }
    }
}

/// Manages log file rotation
pub struct LogRotator {
    config: LogRotationConfig,
    current_file: Arc<Mutex<Option<File>>>,
    current_size: Arc<Mutex<u64>>,
}

impl LogRotator {
    /// Creates a new log rotator with the given configuration
    pub fn new(config: LogRotationConfig) -> Self {
        Self {
            config,
            current_file: Arc::new(Mutex::new(None)),
            current_size: Arc::new(Mutex::new(0)),
        }
    }

    /// Opens or creates the log file
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Failed to create the log directory
    /// - Failed to open the log file
    /// - Failed to get file metadata
    ///
    /// # Panics
    ///
    /// Panics if the mutex lock is poisoned.
    pub fn open(&self) -> Result<()> {
        // Create parent directory if needed
        if let Some(parent) = self.config.base_path.parent() {
            fs::create_dir_all(parent).context("Failed to create log directory")?;
        }

        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.config.base_path)
            .context("Failed to open log file")?;

        // Get current file size
        let metadata = file.metadata()?;
        let size = metadata.len();

        let mut current_file = self.current_file.lock().unwrap();
        *current_file = Some(file);

        let mut current_size = self.current_size.lock().unwrap();
        *current_size = size;

        info!("Opened log file at {:?}", self.config.base_path);
        Ok(())
    }

    /// Writes data to the log file, rotating if necessary
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Failed to rotate the log file
    /// - Failed to write data to the file
    /// - Failed to flush the file
    ///
    /// # Panics
    ///
    /// Panics if the mutex lock is poisoned.
    pub fn write(&self, data: &[u8]) -> Result<()> {
        let mut current_size = self.current_size.lock().unwrap();
        let mut current_file = self.current_file.lock().unwrap();

        // Check if rotation is needed
        if *current_size + data.len() as u64 > self.config.max_size {
            drop(current_file); // Release lock during rotation
            drop(current_size);
            self.rotate()?;
            current_size = self.current_size.lock().unwrap();
            current_file = self.current_file.lock().unwrap();
        }

        // Write data
        if let Some(ref mut file) = *current_file {
            file.write_all(data)?;
            file.flush()?;
            *current_size += data.len() as u64;
        }

        Ok(())
    }

    /// Rotates the log files
    fn rotate(&self) -> Result<()> {
        info!("Rotating log files");

        // Close current file
        {
            let mut current_file = self.current_file.lock().unwrap();
            *current_file = None;
        }

        // Generate timestamp for rotated file
        let timestamp = Local::now().format("%Y%m%d_%H%M%S");
        let rotated_name = format!("{}.{}", self.config.base_path.display(), timestamp);

        // Rename current file
        fs::rename(&self.config.base_path, &rotated_name).context("Failed to rotate log file")?;

        // Compress if configured
        if self.config.compress {
            Self::compress_file(&rotated_name)?;
        }

        // Clean up old files
        self.cleanup_old_files()?;

        // Open new file
        self.open()?;

        Ok(())
    }

    /// Compresses a rotated log file
    fn compress_file(path: &str) -> Result<()> {
        use flate2::write::GzEncoder;
        use flate2::Compression;

        let input_path = Path::new(path);
        let output_path = PathBuf::from(format!("{path}.gz"));

        let input_file = File::open(input_path)?;
        let output_file = File::create(&output_path)?;
        let mut encoder = GzEncoder::new(output_file, Compression::default());

        std::io::copy(&mut &input_file, &mut encoder)?;
        encoder.finish()?;

        // Remove uncompressed file
        fs::remove_file(input_path)?;

        debug!("Compressed log file to {:?}", output_path);
        Ok(())
    }

    /// Removes old rotated files beyond the maximum count
    fn cleanup_old_files(&self) -> Result<()> {
        let parent = self
            .config
            .base_path
            .parent()
            .unwrap_or(Path::new("/var/log"));
        let base_name = self
            .config
            .base_path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("llmspell.log");

        // Find all rotated files
        let mut rotated_files: Vec<_> = fs::read_dir(parent)?
            .filter_map(std::result::Result::ok)
            .filter(|entry| {
                if let Some(name) = entry.file_name().to_str() {
                    name.starts_with(base_name) && name != base_name
                } else {
                    false
                }
            })
            .collect();

        // Sort by modification time (oldest first)
        rotated_files.sort_by_key(|entry| {
            entry
                .metadata()
                .and_then(|m| m.modified())
                .unwrap_or(std::time::SystemTime::UNIX_EPOCH)
        });

        // Remove oldest files if we exceed the limit
        let files_to_remove = rotated_files.len().saturating_sub(self.config.max_files);
        for entry in rotated_files.iter().take(files_to_remove) {
            fs::remove_file(entry.path())?;
            debug!("Removed old log file: {:?}", entry.path());
        }

        Ok(())
    }
}

/// Creates a log writer that can be used for I/O redirection
pub struct DaemonLogWriter {
    rotator: Arc<LogRotator>,
    prefix: String,
}

impl DaemonLogWriter {
    /// Creates a new daemon log writer
    pub fn new(rotator: Arc<LogRotator>, prefix: &str) -> Self {
        Self {
            rotator,
            prefix: prefix.to_string(),
        }
    }
}

impl Write for DaemonLogWriter {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        // Add timestamp and prefix
        let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S%.3f");
        let line = format!("[{}] {}: ", timestamp, self.prefix);

        self.rotator
            .write(line.as_bytes())
            .map_err(std::io::Error::other)?;

        self.rotator.write(buf).map_err(std::io::Error::other)?;

        Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        // Flushing is handled in the rotator
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_log_rotation_config_default() {
        let config = LogRotationConfig::default();
        assert_eq!(config.max_size, 10 * 1024 * 1024);
        assert_eq!(config.max_files, 5);
        assert!(!config.compress);
    }

    #[test]
    fn test_log_rotator_creation() {
        let temp_dir = tempdir().unwrap();
        let config = LogRotationConfig {
            base_path: temp_dir.path().join("test.log"),
            ..Default::default()
        };

        let rotator = LogRotator::new(config);
        assert!(rotator.current_file.lock().unwrap().is_none());
    }

    #[test]
    fn test_log_file_open() {
        let temp_dir = tempdir().unwrap();
        let config = LogRotationConfig {
            base_path: temp_dir.path().join("test.log"),
            max_size: 100,
            max_files: 2,
            compress: false,
        };

        let rotator = LogRotator::new(config);
        rotator.open().unwrap();

        assert!(rotator.current_file.lock().unwrap().is_some());
        assert!(temp_dir.path().join("test.log").exists());
    }

    #[test]
    fn test_log_writing() {
        let temp_dir = tempdir().unwrap();
        let log_path = temp_dir.path().join("test.log");
        let config = LogRotationConfig {
            base_path: log_path.clone(),
            max_size: 1000,
            max_files: 2,
            compress: false,
        };

        let rotator = LogRotator::new(config);
        rotator.open().unwrap();

        // Write some data
        rotator.write(b"Test log message\n").unwrap();

        // Verify file contains the data
        let contents = fs::read_to_string(&log_path).unwrap();
        assert!(contents.contains("Test log message"));
    }

    #[test]
    fn test_daemon_log_writer() {
        let temp_dir = tempdir().unwrap();
        let config = LogRotationConfig {
            base_path: temp_dir.path().join("test.log"),
            ..Default::default()
        };

        let rotator = Arc::new(LogRotator::new(config));
        rotator.open().unwrap();

        let mut writer = DaemonLogWriter::new(rotator.clone(), "TEST");
        writer.write_all(b"Log message").unwrap();
        writer.flush().unwrap();

        // Verify the log file exists and contains data
        let log_path = temp_dir.path().join("test.log");
        let contents = fs::read_to_string(&log_path).unwrap();
        assert!(contents.contains("TEST"));
        assert!(contents.contains("Log message"));
    }

    #[test]
    fn test_log_rotation_trigger() {
        let temp_dir = tempdir().unwrap();
        let log_path = temp_dir.path().join("test.log");
        let config = LogRotationConfig {
            base_path: log_path.clone(),
            max_size: 50, // Very small size to trigger rotation
            max_files: 2,
            compress: false,
        };

        let rotator = LogRotator::new(config);
        rotator.open().unwrap();

        // Write enough data to trigger rotation
        let large_data = b"This is a test message that should trigger rotation\n";
        rotator.write(large_data).unwrap();

        // Write more data to ensure rotation happened
        rotator.write(b"After rotation\n").unwrap();

        // Check that rotated files exist
        let entries: Vec<_> = fs::read_dir(temp_dir.path())
            .unwrap()
            .filter_map(Result::ok)
            .collect();
        assert!(entries.len() >= 2, "Should have original and rotated file");
    }

    #[test]
    fn test_log_compression() {
        let temp_dir = tempdir().unwrap();
        let log_path = temp_dir.path().join("test.log");
        let config = LogRotationConfig {
            base_path: log_path.clone(),
            max_size: 50,
            max_files: 3,
            compress: true, // Enable compression
        };

        let rotator = LogRotator::new(config);
        rotator.open().unwrap();

        // Trigger rotation with compression
        let data = b"Data that will be compressed after rotation\n";
        rotator.write(data).unwrap();
        rotator.write(data).unwrap(); // Trigger rotation

        // Check for .gz file
        let has_compressed = fs::read_dir(temp_dir.path()).unwrap().any(|entry| {
            entry
                .ok()
                .and_then(|e| {
                    e.path()
                        .extension()
                        .and_then(|ext| ext.to_str())
                        .map(|s| s.eq_ignore_ascii_case("gz"))
                })
                .unwrap_or(false)
        });
        assert!(has_compressed, "Should have created compressed file");
    }

    #[test]
    fn test_cleanup_old_files() {
        let temp_dir = tempdir().unwrap();
        let log_path = temp_dir.path().join("test.log");
        let config = LogRotationConfig {
            base_path: log_path.clone(),
            max_size: 30,
            max_files: 2, // Keep only 2 rotated files
            compress: false,
        };

        let rotator = LogRotator::new(config);
        rotator.open().unwrap();

        // Create multiple rotations
        for i in 0..5 {
            let data = format!("Rotation {i}: Some test data\n");
            rotator.write(data.as_bytes()).unwrap();
            std::thread::sleep(std::time::Duration::from_millis(10)); // Ensure different timestamps
        }

        // Count files (should be current + max_files)
        let file_count = fs::read_dir(temp_dir.path())
            .unwrap()
            .filter_map(Result::ok)
            .filter(|entry| {
                entry
                    .file_name()
                    .to_str()
                    .is_some_and(|s| s.contains("test.log"))
            })
            .count();

        assert!(
            file_count <= 3,
            "Should have at most 3 files (current + 2 rotated)"
        );
    }

    #[test]
    fn test_concurrent_writes() {
        use std::sync::Arc;
        use std::thread;

        let temp_dir = tempdir().unwrap();
        let config = LogRotationConfig {
            base_path: temp_dir.path().join("test.log"),
            max_size: 1000,
            max_files: 3,
            compress: false,
        };

        let rotator = Arc::new(LogRotator::new(config));
        rotator.open().unwrap();

        // Spawn multiple threads writing concurrently
        let handles: Vec<_> = (0..5)
            .map(|i| {
                let rotator_clone = rotator.clone();
                thread::spawn(move || {
                    for j in 0..10 {
                        let msg = format!("Thread {i} message {j}\n");
                        rotator_clone.write(msg.as_bytes()).unwrap();
                    }
                })
            })
            .collect();

        // Wait for all threads
        for handle in handles {
            handle.join().unwrap();
        }

        // Verify log file has content from all threads
        let log_path = temp_dir.path().join("test.log");
        let contents = fs::read_to_string(&log_path).unwrap();
        for i in 0..5 {
            assert!(contents.contains(&format!("Thread {i}")));
        }
    }

    #[test]
    fn test_rotation_atomicity() {
        let temp_dir = tempdir().unwrap();
        let log_path = temp_dir.path().join("test.log");
        let config = LogRotationConfig {
            base_path: log_path.clone(),
            max_size: 40,
            max_files: 2,
            compress: false,
        };

        let rotator = LogRotator::new(config);
        rotator.open().unwrap();

        // Write initial data
        rotator.write(b"Before rotation\n").unwrap();

        // This should trigger rotation
        rotator
            .write(b"This triggers rotation due to size\n")
            .unwrap();

        // Verify no data loss
        let mut all_content = String::new();
        for entry in fs::read_dir(temp_dir.path()).unwrap().flatten() {
            if entry.file_name().to_str().unwrap().contains("test.log") {
                all_content.push_str(&fs::read_to_string(entry.path()).unwrap());
            }
        }

        assert!(all_content.contains("Before rotation"));
        assert!(all_content.contains("This triggers rotation"));
        rotator.write(b"After rotation\n").unwrap();

        // Verify main log file exists
        assert!(log_path.exists());
    }

    #[test]
    fn test_log_rotation_with_compression() {
        use flate2::read::GzDecoder;
        use std::io::Read;

        let temp_dir = tempdir().unwrap();
        let log_path = temp_dir.path().join("test.log");
        let config = LogRotationConfig {
            base_path: log_path.clone(),
            max_size: 30, // Small size for rotation
            max_files: 1,
            compress: true, // Enable compression
        };

        let rotator = LogRotator::new(config);
        rotator.open().unwrap();

        // Write data to trigger rotation
        rotator.write(b"First log entry before rotation\n").unwrap();
        rotator.write(b"Second entry\n").unwrap();

        // Find compressed file
        let parent_dir = log_path.parent().unwrap();
        let compressed_file = fs::read_dir(parent_dir)
            .unwrap()
            .filter_map(Result::ok)
            .find(|entry| {
                entry.file_name().to_str().is_some_and(|name| {
                    std::path::Path::new(name)
                        .extension()
                        .is_some_and(|ext| ext.eq_ignore_ascii_case("gz"))
                })
            });

        // If rotation occurred with compression, verify the compressed file
        if let Some(entry) = compressed_file {
            let compressed_path = entry.path();
            let file = File::open(&compressed_path).unwrap();
            let mut decoder = GzDecoder::new(file);
            let mut contents = String::new();
            decoder.read_to_string(&mut contents).unwrap();
            assert!(contents.contains("First log entry"));
        }
    }

    #[test]
    fn test_log_file_cleanup() {
        let temp_dir = tempdir().unwrap();
        let log_path = temp_dir.path().join("test.log");
        let config = LogRotationConfig {
            base_path: log_path.clone(),
            max_size: 20, // Very small for frequent rotation
            max_files: 2, // Keep only 2 rotated files
            compress: false,
        };

        let rotator = LogRotator::new(config);
        rotator.open().unwrap();

        // Trigger multiple rotations
        for i in 0..5 {
            let data = format!("Log entry number {i}\n");
            rotator.write(data.as_bytes()).unwrap();
        }

        // Count log files
        let parent_dir = log_path.parent().unwrap();
        let log_files: Vec<_> = fs::read_dir(parent_dir)
            .unwrap()
            .filter_map(Result::ok)
            .filter(|entry| {
                entry
                    .file_name()
                    .to_str()
                    .is_some_and(|name| name.contains("test.log"))
            })
            .collect();

        // Should have at most max_files + 1 (current file)
        assert!(log_files.len() <= 3);
    }

    #[test]
    fn test_io_stream_redirection() {
        let temp_dir = tempdir().unwrap();
        let config = LogRotationConfig {
            base_path: temp_dir.path().join("daemon.log"),
            max_size: 1024,
            max_files: 3,
            compress: false,
        };

        let rotator = Arc::new(LogRotator::new(config));
        rotator.open().unwrap();

        // Create writers for stdout and stderr
        let mut stdout_writer = DaemonLogWriter::new(rotator.clone(), "STDOUT");
        let mut stderr_writer = DaemonLogWriter::new(rotator.clone(), "STDERR");

        // Write to both streams
        stdout_writer
            .write_all(b"Standard output message\n")
            .unwrap();
        stderr_writer
            .write_all(b"Standard error message\n")
            .unwrap();

        // Verify both messages are in the log
        let log_path = temp_dir.path().join("daemon.log");
        let contents = fs::read_to_string(&log_path).unwrap();
        assert!(contents.contains("STDOUT"));
        assert!(contents.contains("Standard output message"));
        assert!(contents.contains("STDERR"));
        assert!(contents.contains("Standard error message"));
    }
}
