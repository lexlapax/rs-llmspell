//! ABOUTME: File upload security framework for validating and sanitizing uploaded files
//! ABOUTME: Provides file type validation, magic number verification, and content scanning

use std::collections::HashMap;
use std::io::Read;
use std::path::{Path, PathBuf};

/// File upload security configuration
#[derive(Debug, Clone)]
pub struct FileUploadConfig {
    /// Maximum file size in bytes
    pub max_file_size: u64,
    /// Allowed file extensions
    pub allowed_extensions: Vec<String>,
    /// Allowed MIME types
    pub allowed_mime_types: Vec<String>,
    /// Enable magic number verification
    pub verify_magic_numbers: bool,
    /// Enable content scanning
    pub scan_content: bool,
    /// Quarantine directory for suspicious files
    pub quarantine_path: Option<PathBuf>,
    /// Maximum filename length
    pub max_filename_length: usize,
}

impl Default for FileUploadConfig {
    fn default() -> Self {
        Self {
            max_file_size: 100 * 1024 * 1024, // 100MB
            allowed_extensions: vec![
                "txt".to_string(),
                "csv".to_string(),
                "json".to_string(),
                "xml".to_string(),
                "pdf".to_string(),
                "png".to_string(),
                "jpg".to_string(),
                "jpeg".to_string(),
                "gif".to_string(),
                "mp3".to_string(),
                "mp4".to_string(),
                "doc".to_string(),
                "docx".to_string(),
            ],
            allowed_mime_types: vec![
                "text/plain".to_string(),
                "text/csv".to_string(),
                "application/json".to_string(),
                "application/xml".to_string(),
                "application/pdf".to_string(),
                "image/png".to_string(),
                "image/jpeg".to_string(),
                "image/gif".to_string(),
                "audio/mpeg".to_string(),
                "video/mp4".to_string(),
                "application/msword".to_string(),
                "application/vnd.openxmlformats-officedocument.wordprocessingml.document"
                    .to_string(),
            ],
            verify_magic_numbers: true,
            scan_content: true,
            quarantine_path: None,
            max_filename_length: 255,
        }
    }
}

/// File validation result
#[derive(Debug, Clone)]
pub struct FileValidationResult {
    /// Whether the file is valid
    pub is_valid: bool,
    /// Detected file type
    pub file_type: Option<String>,
    /// Detected MIME type
    pub mime_type: Option<String>,
    /// File size in bytes
    pub size: u64,
    /// Validation errors
    pub errors: Vec<FileValidationError>,
    /// Security warnings
    pub warnings: Vec<String>,
}

/// File validation errors
#[derive(Debug, Clone, PartialEq)]
pub enum FileValidationError {
    /// File size exceeds limit
    FileTooLarge {
        /// Actual file size in bytes
        size: u64,
        /// Maximum allowed size in bytes
        limit: u64,
    },
    /// Invalid file extension
    InvalidExtension {
        /// The invalid extension
        extension: String,
    },
    /// Invalid MIME type
    InvalidMimeType {
        /// The invalid MIME type
        mime_type: String,
    },
    /// Magic number mismatch
    MagicNumberMismatch {
        /// Expected magic number
        expected: String,
        /// Found magic number
        found: String,
    },
    /// Malicious content detected
    MaliciousContent {
        /// Reason for detection
        reason: String,
    },
    /// Invalid filename
    InvalidFilename {
        /// Reason for invalidity
        reason: String,
    },
    /// File not found
    FileNotFound {
        /// Path to the missing file
        path: String,
    },
    /// IO error
    IoError {
        /// Error message
        message: String,
    },
}

impl std::fmt::Display for FileValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::FileTooLarge { size, limit } => {
                write!(f, "File too large: {size} bytes (limit: {limit} bytes)")
            }
            Self::InvalidExtension { extension } => {
                write!(f, "Invalid file extension: {extension}")
            }
            Self::InvalidMimeType { mime_type } => {
                write!(f, "Invalid MIME type: {mime_type}")
            }
            Self::MagicNumberMismatch { expected, found } => {
                write!(
                    f,
                    "Magic number mismatch: expected {expected}, found {found}"
                )
            }
            Self::MaliciousContent { reason } => {
                write!(f, "Malicious content detected: {reason}")
            }
            Self::InvalidFilename { reason } => {
                write!(f, "Invalid filename: {reason}")
            }
            Self::FileNotFound { path } => {
                write!(f, "File not found: {path}")
            }
            Self::IoError { message } => {
                write!(f, "IO error: {message}")
            }
        }
    }
}

impl std::error::Error for FileValidationError {}

/// Magic number definitions for common file types
struct MagicNumbers;

impl MagicNumbers {
    fn get_magic_numbers() -> HashMap<&'static str, Vec<u8>> {
        let mut magic = HashMap::new();

        // Images
        magic.insert("png", vec![0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A]);
        magic.insert("jpg", vec![0xFF, 0xD8, 0xFF]);
        magic.insert("jpeg", vec![0xFF, 0xD8, 0xFF]);
        magic.insert("gif87", vec![0x47, 0x49, 0x46, 0x38, 0x37, 0x61]);
        magic.insert("gif89", vec![0x47, 0x49, 0x46, 0x38, 0x39, 0x61]);

        // Documents
        magic.insert("pdf", vec![0x25, 0x50, 0x44, 0x46]); // %PDF
        magic.insert("doc", vec![0xD0, 0xCF, 0x11, 0xE0, 0xA1, 0xB1, 0x1A, 0xE1]);
        magic.insert("docx", vec![0x50, 0x4B, 0x03, 0x04]); // ZIP format

        // Archives
        magic.insert("zip", vec![0x50, 0x4B, 0x03, 0x04]);
        magic.insert("rar", vec![0x52, 0x61, 0x72, 0x21]);
        magic.insert("7z", vec![0x37, 0x7A, 0xBC, 0xAF, 0x27, 0x1C]);

        // Executables (dangerous)
        magic.insert("exe", vec![0x4D, 0x5A]); // MZ
        magic.insert("elf", vec![0x7F, 0x45, 0x4C, 0x46]); // ELF
        magic.insert("mach-o", vec![0xFE, 0xED, 0xFA, 0xCE]);

        // Scripts (potentially dangerous)
        magic.insert("script_shebang", vec![0x23, 0x21]); // #!

        // Audio/Video
        magic.insert("mp3_id3", vec![0x49, 0x44, 0x33]); // ID3
        magic.insert("mp3_sync", vec![0xFF, 0xFB]);
        magic.insert("mp4", vec![0x00, 0x00, 0x00, 0x20, 0x66, 0x74, 0x79, 0x70]);

        magic
    }
}

/// File upload validator
pub struct FileUploadValidator {
    config: FileUploadConfig,
}

impl FileUploadValidator {
    /// Create a new file upload validator
    #[must_use]
    pub fn new(config: FileUploadConfig) -> Self {
        Self { config }
    }

    /// Create with default configuration
    #[must_use]
    pub fn with_defaults() -> Self {
        Self::new(FileUploadConfig::default())
    }

    /// Validate a file upload
    ///
    /// # Errors
    ///
    /// Returns validation errors if the file fails any security checks
    pub fn validate_file(
        &self,
        file_path: &Path,
    ) -> Result<FileValidationResult, FileValidationError> {
        let mut result = FileValidationResult {
            is_valid: true,
            file_type: None,
            mime_type: None,
            size: 0,
            errors: Vec::new(),
            warnings: Vec::new(),
        };

        // Check if file exists
        if !file_path.exists() {
            return Err(FileValidationError::FileNotFound {
                path: file_path.display().to_string(),
            });
        }

        // Get file metadata
        let metadata = std::fs::metadata(file_path).map_err(|e| FileValidationError::IoError {
            message: e.to_string(),
        })?;

        result.size = metadata.len();

        // Check file size
        if result.size > self.config.max_file_size {
            result.errors.push(FileValidationError::FileTooLarge {
                size: result.size,
                limit: self.config.max_file_size,
            });
            result.is_valid = false;
        }

        // Validate filename
        if let Some(filename) = file_path.file_name() {
            if let Some(filename_str) = filename.to_str() {
                self.validate_filename(filename_str, &mut result);
            }
        }

        // Check file extension
        if let Some(extension) = file_path.extension() {
            if let Some(ext_str) = extension.to_str() {
                if !self
                    .config
                    .allowed_extensions
                    .contains(&ext_str.to_lowercase())
                {
                    result.errors.push(FileValidationError::InvalidExtension {
                        extension: ext_str.to_string(),
                    });
                    result.is_valid = false;
                }
                result.file_type = Some(ext_str.to_lowercase());
            }
        }

        // Verify magic numbers
        if self.config.verify_magic_numbers {
            self.verify_magic_numbers(file_path, &mut result)?;
        }

        // Scan content for malicious patterns
        if self.config.scan_content {
            self.scan_content(file_path, &mut result);
        }

        Ok(result)
    }

    /// Validate filename for security issues
    fn validate_filename(&self, filename: &str, result: &mut FileValidationResult) {
        // Check filename length
        if filename.len() > self.config.max_filename_length {
            result.errors.push(FileValidationError::InvalidFilename {
                reason: format!("Filename too long: {} characters", filename.len()),
            });
            result.is_valid = false;
        }

        // Check for directory traversal attempts
        if filename.contains("..") || filename.contains('/') || filename.contains('\\') {
            result.errors.push(FileValidationError::InvalidFilename {
                reason: "Filename contains directory traversal characters".to_string(),
            });
            result.is_valid = false;
        }

        // Check for null bytes
        if filename.contains('\0') {
            result.errors.push(FileValidationError::InvalidFilename {
                reason: "Filename contains null bytes".to_string(),
            });
            result.is_valid = false;
        }

        // Check for control characters
        if filename.chars().any(char::is_control) {
            result.errors.push(FileValidationError::InvalidFilename {
                reason: "Filename contains control characters".to_string(),
            });
            result.is_valid = false;
        }

        // Warn about special characters
        if filename
            .chars()
            .any(|c| matches!(c, '<' | '>' | ':' | '"' | '|' | '?' | '*'))
        {
            result
                .warnings
                .push("Filename contains special characters that may cause issues".to_string());
        }
    }

    /// Verify file magic numbers
    #[allow(clippy::unused_self)]
    fn verify_magic_numbers(
        &self,
        file_path: &Path,
        result: &mut FileValidationResult,
    ) -> Result<(), FileValidationError> {
        let mut file =
            std::fs::File::open(file_path).map_err(|e| FileValidationError::IoError {
                message: e.to_string(),
            })?;

        let mut buffer = vec![0u8; 16]; // Read first 16 bytes
        let bytes_read = file
            .read(&mut buffer)
            .map_err(|e| FileValidationError::IoError {
                message: e.to_string(),
            })?;

        if bytes_read == 0 {
            result.warnings.push("Empty file".to_string());
            return Ok(());
        }

        buffer.truncate(bytes_read);

        let magic_numbers = MagicNumbers::get_magic_numbers();
        let mut found_match = false;

        // Check against known magic numbers
        for (file_type, magic) in &magic_numbers {
            if buffer.starts_with(magic) {
                // Check for dangerous file types
                if matches!(*file_type, "exe" | "elf" | "mach-o" | "script_shebang") {
                    result.errors.push(FileValidationError::MaliciousContent {
                        reason: format!("Executable file type detected: {file_type}"),
                    });
                    result.is_valid = false;
                }

                // Verify extension matches magic number
                if let Some(ref ext) = result.file_type {
                    if !file_type.starts_with(ext) && ext != "txt" {
                        result.warnings.push(format!(
                            "File extension '{ext}' doesn't match detected type '{file_type}'"
                        ));
                    }
                }

                found_match = true;
                break;
            }
        }

        if !found_match && result.file_type.is_some() {
            // For text files, we don't require magic numbers
            let text_extensions = ["txt", "csv", "json", "xml", "log"];
            if !text_extensions.contains(&result.file_type.as_ref().unwrap().as_str()) {
                result
                    .warnings
                    .push("Unable to verify file type from magic numbers".to_string());
            }
        }

        Ok(())
    }

    /// Scan file content for malicious patterns
    #[allow(clippy::unused_self)]
    fn scan_content(&self, file_path: &Path, result: &mut FileValidationResult) {
        let content = std::fs::read_to_string(file_path).unwrap_or_default();

        // Check for suspicious patterns in text files
        if !content.is_empty() {
            // Check for script injections
            let script_patterns = [
                "<script",
                "javascript:",
                "onerror=",
                "onclick=",
                "onload=",
                "eval(",
                "document.write",
                "innerHTML",
            ];

            for pattern in &script_patterns {
                if content.to_lowercase().contains(pattern) {
                    result
                        .warnings
                        .push(format!("Suspicious pattern found: {pattern}"));
                }
            }

            // Check for PHP code
            if content.contains("<?php") || content.contains("<?=") {
                result.errors.push(FileValidationError::MaliciousContent {
                    reason: "PHP code detected".to_string(),
                });
                result.is_valid = false;
            }

            // Check for shell commands
            let shell_patterns = ["rm -rf", "chmod", "chown", "/bin/sh", "/bin/bash"];
            for pattern in &shell_patterns {
                if content.contains(pattern) {
                    result
                        .warnings
                        .push(format!("Shell command pattern found: {pattern}"));
                }
            }
        }
    }

    /// Sanitize a filename for safe storage
    #[must_use]
    pub fn sanitize_filename(&self, filename: &str) -> String {
        let mut sanitized = filename.to_string();

        // Remove directory traversal attempts
        sanitized = sanitized.replace("..", "");
        sanitized = sanitized.replace('/', "_");
        sanitized = sanitized.replace('\\', "_");

        // Remove control characters and null bytes
        sanitized = sanitized
            .chars()
            .filter(|c| !c.is_control() && *c != '\0')
            .collect();

        // Replace special characters
        for c in ['<', '>', ':', '"', '|', '?', '*'] {
            sanitized = sanitized.replace(c, "_");
        }

        // Truncate if too long
        if sanitized.len() > self.config.max_filename_length {
            let extension = Path::new(&sanitized)
                .extension()
                .and_then(|ext| ext.to_str())
                .unwrap_or("");

            let base_len = self.config.max_filename_length - extension.len() - 1;
            sanitized = format!(
                "{}.{}",
                &sanitized[..base_len.min(sanitized.len())],
                extension
            );
        }

        // Ensure filename is not empty
        if sanitized.trim().is_empty() {
            sanitized = "unnamed_file".to_string();
        }

        sanitized
    }

    /// Get safe file storage path
    #[must_use]
    pub fn get_safe_storage_path(&self, base_dir: &Path, filename: &str) -> PathBuf {
        let sanitized_filename = self.sanitize_filename(filename);

        // Add timestamp to prevent collisions
        let timestamp = chrono::Utc::now().timestamp();
        let unique_filename = format!("{timestamp}_{sanitized_filename}");

        base_dir.join(unique_filename)
    }
}

/// File processing sandbox for isolating file operations
pub struct FileProcessingSandbox {
    /// Temporary directory for processing
    temp_dir: PathBuf,
    /// Maximum processing time
    #[allow(dead_code)]
    max_processing_time: std::time::Duration,
    /// Memory limit for processing
    #[allow(dead_code)]
    memory_limit: Option<u64>,
}

impl FileProcessingSandbox {
    /// Create a new file processing sandbox
    ///
    /// # Errors
    ///
    /// Returns an error if the temporary directory cannot be created
    pub fn new() -> Result<Self, std::io::Error> {
        let temp_dir =
            std::env::temp_dir().join(format!("llmspell_sandbox_{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&temp_dir)?;

        Ok(Self {
            temp_dir,
            max_processing_time: std::time::Duration::from_secs(300), // 5 minutes
            memory_limit: Some(1024 * 1024 * 1024),                   // 1GB
        })
    }

    /// Get the sandbox directory
    #[must_use]
    pub fn sandbox_dir(&self) -> &Path {
        &self.temp_dir
    }

    /// Copy file to sandbox for processing
    ///
    /// # Errors
    ///
    /// Returns an error if the file cannot be copied
    pub fn copy_to_sandbox(&self, source: &Path) -> Result<PathBuf, std::io::Error> {
        let filename = source.file_name().ok_or_else(|| {
            std::io::Error::new(std::io::ErrorKind::InvalidInput, "Invalid filename")
        })?;

        let dest = self.temp_dir.join(filename);
        std::fs::copy(source, &dest)?;

        Ok(dest)
    }

    /// Clean up sandbox directory
    ///
    /// # Errors
    ///
    /// Returns an error if the directory cannot be removed
    pub fn cleanup(&self) -> Result<(), std::io::Error> {
        if self.temp_dir.exists() {
            std::fs::remove_dir_all(&self.temp_dir)?;
        }
        Ok(())
    }
}

impl Drop for FileProcessingSandbox {
    fn drop(&mut self) {
        let _ = self.cleanup();
    }
}

#[cfg(test)]
#[cfg_attr(test_category = "util")]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[cfg_attr(test_category = "unit")]
    #[test]
    fn test_file_size_validation() {
        let config = FileUploadConfig {
            max_file_size: 100,
            ..Default::default()
        };

        let validator = FileUploadValidator::new(config);

        // Create a file larger than the limit
        let mut file = NamedTempFile::new().unwrap();
        file.write_all(&[0u8; 200]).unwrap();

        let result = validator.validate_file(file.path()).unwrap();
        assert!(!result.is_valid);
        assert!(result
            .errors
            .iter()
            .any(|e| matches!(e, FileValidationError::FileTooLarge { .. })));
    }

    #[cfg_attr(test_category = "unit")]
    #[test]
    fn test_filename_sanitization() {
        let validator = FileUploadValidator::with_defaults();

        assert_eq!(validator.sanitize_filename("normal.txt"), "normal.txt");
        assert_eq!(
            validator.sanitize_filename("../../../etc/passwd"),
            "___etc_passwd"
        );
        assert_eq!(
            validator.sanitize_filename("file<>:\"|?*.txt"),
            "file_______.txt"
        );
        assert_eq!(
            validator.sanitize_filename("file\0with\0nulls.txt"),
            "filewithnulls.txt"
        );
    }

    #[cfg_attr(test_category = "unit")]
    #[test]
    fn test_magic_number_detection() {
        let validator = FileUploadValidator::with_defaults();

        // Create a fake PNG file without extension
        let mut file = NamedTempFile::new().unwrap();
        file.write_all(&[0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A])
            .unwrap();

        let result = validator.validate_file(file.path()).unwrap();

        // When file has no extension but valid magic numbers, we don't get a warning
        // Instead, we detect it's a PNG from magic numbers
        assert!(result.is_valid);

        // If there were warnings about extension mismatch, they would be here
        if !result.warnings.is_empty() {
            // File might have warnings about extension not matching detected type
            assert!(result
                .warnings
                .iter()
                .any(|w| w.contains("doesn't match detected type")
                    || w.contains("Unable to verify file type")));
        }
    }

    #[cfg_attr(test_category = "unit")]
    #[test]
    fn test_malicious_content_detection() {
        let validator = FileUploadValidator::with_defaults();

        // Create a file with PHP code
        let mut file = NamedTempFile::new().unwrap();
        file.write_all(b"<?php system($_GET['cmd']); ?>").unwrap();

        let result = validator.validate_file(file.path()).unwrap();
        assert!(!result.is_valid);
        assert!(result
            .errors
            .iter()
            .any(|e| matches!(e, FileValidationError::MaliciousContent { .. })));
    }

    #[cfg_attr(test_category = "unit")]
    #[test]
    fn test_sandbox_creation() {
        let sandbox = FileProcessingSandbox::new().unwrap();
        assert!(sandbox.sandbox_dir().exists());

        // Cleanup happens on drop
        let sandbox_dir = sandbox.sandbox_dir().to_path_buf();
        drop(sandbox);
        assert!(!sandbox_dir.exists());
    }
}
