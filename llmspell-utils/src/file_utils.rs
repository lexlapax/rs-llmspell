// ABOUTME: Cross-platform file system operations and path manipulation utilities
// ABOUTME: Provides safe abstractions for file operations used throughout LLMSpell

//! File system operations and path utilities
//!
//! This module provides cross-platform file operations, path manipulation,
//! and directory management utilities.
//!
//! # Examples
//!
//! ```rust,no_run
//! use llmspell_utils::file_utils;
//! use std::path::Path;
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! // Ensure a directory exists
//! file_utils::ensure_dir(Path::new("/tmp/myapp"))?;
//!
//! // Expand path with environment variables
//! let expanded = file_utils::expand_path("$HOME/.config/myapp")?;
//! println!("Expanded path: {}", expanded.display());
//!
//! // Safe file operations
//! file_utils::write_file_atomic(Path::new("/tmp/test.txt"), b"Hello, world!")?;
//! let content = file_utils::read_file(Path::new("/tmp/test.txt"))?;
//! # Ok(())
//! # }
//! ```

use anyhow::{Context, Result};
use path_clean::PathClean;
use std::env;
use std::fs::{self, File};
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};

/// Maximum file size for safe operations (100MB)
const MAX_FILE_SIZE: u64 = 100 * 1024 * 1024;

/// Ensure a directory exists, creating it if necessary
///
/// This function creates the directory and all parent directories if they don't exist.
/// It's safe to call even if the directory already exists.
///
/// # Examples
///
/// ```rust,no_run
/// use llmspell_utils::file_utils::ensure_dir;
/// use std::path::Path;
///
/// # fn main() -> std::io::Result<()> {
/// ensure_dir(Path::new("/tmp/myapp/data"))?;
/// # Ok(())
/// # }
/// ```
///
/// # Errors
///
/// Returns an error if:
/// - The path exists but is not a directory
/// - Directory creation fails due to permissions or other OS errors
pub fn ensure_dir(path: &Path) -> io::Result<()> {
    if path.exists() {
        if path.is_dir() {
            Ok(())
        } else {
            Err(io::Error::new(
                io::ErrorKind::AlreadyExists,
                format!("Path exists but is not a directory: {}", path.display()),
            ))
        }
    } else {
        fs::create_dir_all(path).map_err(|e| {
            io::Error::new(
                e.kind(),
                format!("Failed to create directory '{}': {}", path.display(), e),
            )
        })
    }
}

/// Expand path with environment variables and tilde expansion
///
/// Supports:
/// - Environment variable expansion: `$HOME`, `${HOME}`, `%HOME%` (Windows)
/// - Tilde expansion: `~` expands to home directory
/// - Relative path resolution to absolute paths
///
/// # Examples
///
/// ```rust,no_run
/// use llmspell_utils::file_utils::expand_path;
///
/// # fn main() -> Result<(), std::io::Error> {
/// let path = expand_path("~/Documents/config.json")?;
/// let path2 = expand_path("$HOME/.config/app")?;
/// let path3 = expand_path("${TMPDIR}/cache")?;
/// # Ok(())
/// # }
/// ```
///
/// # Errors
///
/// Returns an error if:
/// - Home directory cannot be determined
/// - Environment variable is not found
/// - Path contains invalid UTF-8
pub fn expand_path(path: &str) -> Result<PathBuf, io::Error> {
    let mut expanded = path.to_string();

    // Handle tilde expansion
    if expanded.starts_with("~/") || expanded == "~" {
        let home = env::var("HOME")
            .or_else(|_| env::var("USERPROFILE"))
            .map_err(|_| {
                io::Error::new(
                    io::ErrorKind::NotFound,
                    "Could not determine home directory",
                )
            })?;

        expanded = if expanded == "~" {
            home
        } else {
            format!("{}{}", home, &expanded[1..])
        };
    }

    // Handle environment variable expansion
    // Support both $VAR and ${VAR} syntax
    let mut result = String::new();
    let mut chars = expanded.chars();

    while let Some(ch) = chars.next() {
        if ch == '$' {
            let mut var_name = String::new();

            // Check if it's ${VAR} syntax
            let next_char = chars.clone().next();
            if next_char == Some('{') {
                chars.next(); // consume '{'
                              // Collect until we find '}'
                for c in chars.by_ref() {
                    if c == '}' {
                        break;
                    }
                    var_name.push(c);
                }
            } else {
                // $VAR syntax - collect alphanumeric and underscore chars
                for c in chars.clone() {
                    if c.is_alphanumeric() || c == '_' {
                        var_name.push(c);
                        chars.next(); // consume the character
                    } else {
                        break;
                    }
                }
            }

            if var_name.is_empty() {
                result.push('$');
            } else {
                match env::var(&var_name) {
                    Ok(value) => result.push_str(&value),
                    Err(_) => {
                        return Err(io::Error::new(
                            io::ErrorKind::NotFound,
                            format!("Environment variable '{var_name}' not found"),
                        ));
                    }
                }
            }
        } else if cfg!(windows) && ch == '%' {
            // Windows %VAR% syntax
            let mut var_name = String::new();
            for c in chars.by_ref() {
                if c == '%' {
                    break;
                }
                var_name.push(c);
            }

            if var_name.is_empty() {
                result.push('%');
            } else {
                match env::var(&var_name) {
                    Ok(value) => result.push_str(&value),
                    Err(_) => {
                        return Err(io::Error::new(
                            io::ErrorKind::NotFound,
                            format!("Environment variable '{var_name}' not found"),
                        ));
                    }
                }
            }
        } else {
            result.push(ch);
        }
    }

    // Convert to PathBuf and clean the path
    Ok(PathBuf::from(result).clean())
}

/// Normalize a path for cross-platform compatibility
///
/// This function:
/// - Converts backslashes to forward slashes on Unix
/// - Converts forward slashes to backslashes on Windows
/// - Removes redundant separators and dots
/// - Resolves `..` components where possible
///
/// # Examples
///
/// ```rust
/// use llmspell_utils::file_utils::normalize_path;
/// use std::path::Path;
///
/// let normalized = normalize_path(Path::new("/home/user/../user/./docs"));
/// assert_eq!(normalized.to_str().unwrap(), "/home/user/docs");
/// ```
#[must_use]
pub fn normalize_path(path: &Path) -> PathBuf {
    path.clean()
}

/// Safely read a file with size limits
///
/// Reads the entire contents of a file into memory, with a size limit
/// to prevent memory exhaustion.
///
/// # Examples
///
/// ```rust,no_run
/// use llmspell_utils::file_utils::read_file;
/// use std::path::Path;
///
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let content = read_file(Path::new("/etc/hosts"))?;
/// println!("File content: {}", String::from_utf8_lossy(&content));
/// # Ok(())
/// # }
/// ```
///
/// # Errors
///
/// Returns an error if:
/// - File does not exist
/// - File is larger than 100MB
/// - Read permissions are denied
/// - I/O error occurs
pub fn read_file(path: &Path) -> Result<Vec<u8>> {
    // Check file size first
    let metadata = fs::metadata(path)
        .with_context(|| format!("Failed to read metadata for file: {}", path.display()))?;

    if metadata.len() > MAX_FILE_SIZE {
        anyhow::bail!(
            "File '{}' is too large ({} bytes, max {} bytes)",
            path.display(),
            metadata.len(),
            MAX_FILE_SIZE
        );
    }

    let mut file =
        File::open(path).with_context(|| format!("Failed to open file: {}", path.display()))?;

    // Safe cast: we've already checked the file size is <= MAX_FILE_SIZE (100MB)
    #[allow(clippy::cast_possible_truncation)]
    let capacity = metadata.len().min(usize::MAX as u64) as usize;
    let mut contents = Vec::with_capacity(capacity);
    file.read_to_end(&mut contents)
        .with_context(|| format!("Failed to read file: {}", path.display()))?;

    Ok(contents)
}

/// Write data to a file with proper error handling
///
/// This is a simple write operation. For critical data, use `write_file_atomic`.
///
/// # Examples
///
/// ```rust,no_run
/// use llmspell_utils::file_utils::write_file;
/// use std::path::Path;
///
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// write_file(Path::new("/tmp/output.txt"), b"Hello, world!")?;
/// # Ok(())
/// # }
/// ```
///
/// # Errors
///
/// Returns an error if:
/// - Parent directory does not exist
/// - Write permissions are denied
/// - Disk is full
/// - I/O error occurs
pub fn write_file(path: &Path, data: &[u8]) -> Result<()> {
    // Ensure parent directory exists
    if let Some(parent) = path.parent() {
        ensure_dir(parent).with_context(|| {
            format!("Failed to create parent directory for: {}", path.display())
        })?;
    }

    let mut file =
        File::create(path).with_context(|| format!("Failed to create file: {}", path.display()))?;

    file.write_all(data)
        .with_context(|| format!("Failed to write to file: {}", path.display()))?;

    file.sync_all()
        .with_context(|| format!("Failed to sync file to disk: {}", path.display()))?;

    Ok(())
}

/// Atomically write data to a file
///
/// This function writes data to a temporary file and then atomically renames it
/// to the target path. This ensures that the file is either fully written or
/// not modified at all, preventing partial writes.
///
/// # Examples
///
/// ```rust,no_run
/// use llmspell_utils::file_utils::write_file_atomic;
/// use std::path::Path;
///
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// // This write is atomic - the file will either be fully written or unchanged
/// write_file_atomic(Path::new("/tmp/important.json"), b"{\"status\": \"ok\"}")?;
/// # Ok(())
/// # }
/// ```
///
/// # Errors
///
/// Returns an error if:
/// - Parent directory does not exist
/// - Write permissions are denied
/// - Disk is full
/// - I/O error occurs
pub fn write_file_atomic(path: &Path, data: &[u8]) -> Result<()> {
    // Ensure parent directory exists
    if let Some(parent) = path.parent() {
        ensure_dir(parent).with_context(|| {
            format!("Failed to create parent directory for: {}", path.display())
        })?;
    }

    // Create temporary file in the same directory
    let temp_path = {
        let mut temp = path.to_path_buf();
        let file_name = path
            .file_name()
            .ok_or_else(|| anyhow::anyhow!("Path has no file name: {}", path.display()))?;

        let file_name_str = file_name.to_string_lossy();
        let uuid_str = uuid::Uuid::new_v4().simple();
        let temp_name = format!(".{file_name_str}.tmp.{uuid_str}");

        temp.set_file_name(temp_name);
        temp
    };

    // Write to temporary file
    let result = (|| -> Result<()> {
        let mut file = File::create(&temp_path)
            .with_context(|| format!("Failed to create temporary file: {}", temp_path.display()))?;

        file.write_all(data).with_context(|| {
            format!("Failed to write to temporary file: {}", temp_path.display())
        })?;

        file.sync_all()
            .with_context(|| format!("Failed to sync temporary file: {}", temp_path.display()))?;

        Ok(())
    })();

    // If write failed, clean up temp file
    if let Err(e) = result {
        let _ = fs::remove_file(&temp_path);
        return Err(e);
    }

    // Atomic rename
    fs::rename(&temp_path, path).with_context(|| {
        format!(
            "Failed to rename {} to {}",
            temp_path.display(),
            path.display()
        )
    })?;

    Ok(())
}

/// Check if a path is absolute
///
/// This function correctly handles platform differences:
/// - On Unix: paths starting with `/`
/// - On Windows: paths with drive letters (C:\) or UNC paths (\\server\share)
///
/// # Examples
///
/// ```rust
/// use llmspell_utils::file_utils::is_absolute_path;
/// use std::path::Path;
///
/// assert!(is_absolute_path(Path::new("/home/user")));
/// assert!(!is_absolute_path(Path::new("relative/path")));
///
/// #[cfg(windows)]
/// {
///     assert!(is_absolute_path(Path::new("C:\\Windows")));
///     assert!(is_absolute_path(Path::new("\\\\server\\share")));
/// }
/// ```
#[must_use]
pub fn is_absolute_path(path: &Path) -> bool {
    path.is_absolute()
}

/// Join paths safely, handling platform differences
///
/// This function safely joins path components, handling:
/// - Empty components
/// - Absolute paths in components (which reset the path)
/// - Platform-specific separators
///
/// # Examples
///
/// ```rust
/// use llmspell_utils::file_utils::join_paths;
/// use std::path::Path;
///
/// let joined = join_paths(&[Path::new("/home"), Path::new("user"), Path::new("docs")]);
/// assert_eq!(joined.to_str().unwrap(), "/home/user/docs");
/// ```
#[must_use]
pub fn join_paths(paths: &[&Path]) -> PathBuf {
    let mut result = PathBuf::new();

    for path in paths {
        if path.is_absolute() {
            result = path.to_path_buf();
        } else {
            result.push(path);
        }
    }

    result.clean()
}

/// Get the parent directory of a path
///
/// Returns None if the path has no parent (e.g., "/" or "C:\")
///
/// # Examples
///
/// ```rust
/// use llmspell_utils::file_utils::parent_dir;
/// use std::path::Path;
///
/// assert_eq!(
///     parent_dir(Path::new("/home/user/file.txt")),
///     Some(Path::new("/home/user").to_path_buf())
/// );
/// ```
#[must_use]
pub fn parent_dir(path: &Path) -> Option<PathBuf> {
    path.parent().map(std::path::Path::to_path_buf)
}

/// Copy a file with proper error handling
///
/// # Examples
///
/// ```rust,no_run
/// use llmspell_utils::file_utils::copy_file;
/// use std::path::Path;
///
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// copy_file(Path::new("/tmp/source.txt"), Path::new("/tmp/dest.txt"))?;
/// # Ok(())
/// # }
/// ```
///
/// # Errors
///
/// Returns an error if:
/// - Source file does not exist
/// - Destination directory does not exist
/// - Insufficient permissions
/// - I/O error occurs
pub fn copy_file(from: &Path, to: &Path) -> Result<u64> {
    // Ensure destination directory exists
    if let Some(parent) = to.parent() {
        ensure_dir(parent).with_context(|| {
            format!(
                "Failed to create destination directory for: {}",
                to.display()
            )
        })?;
    }

    fs::copy(from, to)
        .with_context(|| format!("Failed to copy from {} to {}", from.display(), to.display()))
}

/// Remove a file if it exists
///
/// This function does not return an error if the file doesn't exist.
///
/// # Examples
///
/// ```rust,no_run
/// use llmspell_utils::file_utils::remove_file_if_exists;
/// use std::path::Path;
///
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// remove_file_if_exists(Path::new("/tmp/old-file.txt"))?;
/// # Ok(())
/// # }
/// ```
///
/// # Errors
///
/// Returns an error if:
/// - The path exists but is a directory
/// - Insufficient permissions
/// - I/O error occurs (other than `NotFound`)
pub fn remove_file_if_exists(path: &Path) -> Result<()> {
    match fs::remove_file(path) {
        Ok(()) => Ok(()),
        Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(()),
        Err(e) => Err(e).with_context(|| format!("Failed to remove file: {}", path.display())),
    }
}

/// Remove a directory and all its contents if it exists
///
/// This function does not return an error if the directory doesn't exist.
///
/// # Examples
///
/// ```rust,no_run
/// use llmspell_utils::file_utils::remove_dir_all_if_exists;
/// use std::path::Path;
///
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// remove_dir_all_if_exists(Path::new("/tmp/old-dir"))?;
/// # Ok(())
/// # }
/// ```
///
/// # Errors
///
/// Returns an error if:
/// - The path exists but is a file
/// - Insufficient permissions
/// - I/O error occurs (other than `NotFound`)
pub fn remove_dir_all_if_exists(path: &Path) -> Result<()> {
    match fs::remove_dir_all(path) {
        Ok(()) => Ok(()),
        Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(()),
        Err(e) => Err(e).with_context(|| format!("Failed to remove directory: {}", path.display())),
    }
}

/// Append data to a file
///
/// This function appends data to an existing file or creates it if it doesn't exist.
/// The parent directory is created if necessary.
///
/// # Examples
///
/// ```rust,no_run
/// use llmspell_utils::file_utils::append_file;
/// use std::path::Path;
///
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// append_file(Path::new("/tmp/log.txt"), b"\nNew log entry")?;
/// # Ok(())
/// # }
/// ```
///
/// # Errors
///
/// Returns an error if:
/// - Parent directory cannot be created
/// - Write permissions are denied
/// - I/O error occurs
pub fn append_file(path: &Path, data: &[u8]) -> Result<()> {
    // Ensure parent directory exists
    if let Some(parent) = path.parent() {
        ensure_dir(parent).with_context(|| {
            format!("Failed to create parent directory for: {}", path.display())
        })?;
    }

    let mut file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .with_context(|| format!("Failed to open file for appending: {}", path.display()))?;

    file.write_all(data)
        .with_context(|| format!("Failed to append to file: {}", path.display()))?;

    file.sync_all()
        .with_context(|| format!("Failed to sync file to disk: {}", path.display()))?;

    Ok(())
}

/// Move or rename a file
///
/// This function moves a file from one location to another, creating parent
/// directories as needed. It's equivalent to rename on the same filesystem.
///
/// # Examples
///
/// ```rust,no_run
/// use llmspell_utils::file_utils::move_file;
/// use std::path::Path;
///
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// move_file(Path::new("/tmp/old.txt"), Path::new("/tmp/new.txt"))?;
/// # Ok(())
/// # }
/// ```
///
/// # Errors
///
/// Returns an error if:
/// - Source file does not exist
/// - Destination directory cannot be created
/// - Insufficient permissions
/// - Files are on different filesystems and copy fails
pub fn move_file(from: &Path, to: &Path) -> Result<()> {
    // Ensure destination directory exists
    if let Some(parent) = to.parent() {
        ensure_dir(parent).with_context(|| {
            format!(
                "Failed to create destination directory for: {}",
                to.display()
            )
        })?;
    }

    // Try to rename first (most efficient)
    match fs::rename(from, to) {
        Ok(()) => Ok(()),
        Err(e) if e.raw_os_error() == Some(18) => {
            // Cross-device link error (EXDEV on Unix) - need to copy and delete
            copy_file(from, to)?;
            remove_file_if_exists(from)?;
            Ok(())
        }
        Err(e) => Err(e).with_context(|| {
            format!(
                "Failed to move file from {} to {}",
                from.display(),
                to.display()
            )
        }),
    }
}

/// Get file or directory metadata
///
/// Returns detailed metadata about a file or directory including size,
/// timestamps, and permissions.
///
/// # Examples
///
/// ```rust,no_run
/// use llmspell_utils::file_utils::get_metadata;
/// use std::path::Path;
///
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let metadata = get_metadata(Path::new("/tmp/file.txt"))?;
/// println!("File size: {} bytes", metadata.size);
/// println!("Is directory: {}", metadata.is_dir);
/// # Ok(())
/// # }
/// ```
///
/// # Errors
///
/// Returns an error if:
/// - Path does not exist
/// - Insufficient permissions to read metadata
/// - I/O error occurs
#[derive(Debug, Clone, PartialEq, Eq)]
#[allow(clippy::struct_excessive_bools)] // These are all essential metadata fields
pub struct FileMetadata {
    /// File or directory size in bytes
    pub size: u64,
    /// Whether this is a directory
    pub is_dir: bool,
    /// Whether this is a regular file
    pub is_file: bool,
    /// Whether this is a symbolic link
    pub is_symlink: bool,
    /// Whether the file is read-only
    pub readonly: bool,
    /// Creation time (platform-dependent, may not be available)
    pub created: Option<std::time::SystemTime>,
    /// Last modification time
    pub modified: Option<std::time::SystemTime>,
    /// Last access time (may not be reliable on all systems)
    pub accessed: Option<std::time::SystemTime>,
}

/// Get file or directory metadata
///
/// Returns detailed metadata about a file or directory.
///
/// # Examples
///
/// ```rust,no_run
/// use llmspell_utils::file_utils::get_metadata;
/// use std::path::Path;
///
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let metadata = get_metadata(Path::new("/tmp/file.txt"))?;
/// println!("File size: {} bytes", metadata.size);
/// # Ok(())
/// # }
/// ```
///
/// # Errors
///
/// Returns an error if:
/// - Path does not exist
/// - Insufficient permissions
/// - I/O error occurs
pub fn get_metadata(path: &Path) -> Result<FileMetadata> {
    let metadata = fs::metadata(path)
        .with_context(|| format!("Failed to get metadata for: {}", path.display()))?;

    Ok(FileMetadata {
        size: metadata.len(),
        is_dir: metadata.is_dir(),
        is_file: metadata.is_file(),
        is_symlink: metadata.is_symlink(),
        readonly: metadata.permissions().readonly(),
        created: metadata.created().ok(),
        modified: metadata.modified().ok(),
        accessed: metadata.accessed().ok(),
    })
}

/// Check if a file or directory exists
///
/// # Examples
///
/// ```rust
/// use llmspell_utils::file_utils::file_exists;
/// use std::path::Path;
///
/// assert!(!file_exists(Path::new("/tmp/nonexistent.txt")));
/// ```
#[must_use]
pub fn file_exists(path: &Path) -> bool {
    path.exists()
}

/// List directory contents with metadata
///
/// Returns a vector of directory entries with basic metadata.
/// Entries are sorted alphabetically by name.
///
/// # Examples
///
/// ```rust,no_run
/// use llmspell_utils::file_utils::list_dir;
/// use std::path::Path;
///
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let entries = list_dir(Path::new("/tmp"))?;
/// for entry in entries {
///     println!("{}: {} bytes", entry.name, entry.size);
/// }
/// # Ok(())
/// # }
/// ```
///
/// # Errors
///
/// Returns an error if:
/// - Path is not a directory
/// - Insufficient permissions
/// - I/O error occurs
#[derive(Debug, Clone)]
pub struct DirEntry {
    /// File or directory name (not full path)
    pub name: String,
    /// Full path to the entry
    pub path: PathBuf,
    /// File size in bytes
    pub size: u64,
    /// Whether this is a directory
    pub is_dir: bool,
    /// Whether this is a regular file
    pub is_file: bool,
    /// Whether this is a symbolic link
    pub is_symlink: bool,
    /// Last modification time
    pub modified: Option<std::time::SystemTime>,
}

/// List directory contents with metadata
///
/// Returns a vector of directory entries sorted alphabetically.
///
/// # Examples
///
/// ```rust,no_run
/// use llmspell_utils::file_utils::list_dir;
/// use std::path::Path;
///
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let entries = list_dir(Path::new("/tmp"))?;
/// for entry in entries {
///     println!("{}: {} bytes", entry.name, entry.size);
/// }
/// # Ok(())
/// # }
/// ```
///
/// # Errors
///
/// Returns an error if:
/// - Path is not a directory
/// - Insufficient permissions
/// - I/O error occurs
pub fn list_dir(path: &Path) -> Result<Vec<DirEntry>> {
    let mut entries = Vec::new();

    let dir = fs::read_dir(path)
        .with_context(|| format!("Failed to read directory: {}", path.display()))?;

    for entry in dir {
        let entry = entry
            .with_context(|| format!("Failed to read directory entry in: {}", path.display()))?;

        let file_name = entry.file_name();
        let file_name_str = file_name.to_string_lossy().to_string();
        let entry_path = entry.path();

        // Get metadata - if it fails, create minimal entry
        let (size, is_dir, is_file, is_symlink, modified) = match entry.metadata() {
            Ok(metadata) => (
                metadata.len(),
                metadata.is_dir(),
                metadata.is_file(),
                metadata.is_symlink(),
                metadata.modified().ok(),
            ),
            Err(_) => (0, false, false, false, None),
        };

        entries.push(DirEntry {
            name: file_name_str,
            path: entry_path,
            size,
            is_dir,
            is_file,
            is_symlink,
            modified,
        });
    }

    // Sort entries by name
    entries.sort_by(|a, b| a.name.cmp(&b.name));

    Ok(entries)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;
    use std::fs;

    #[test]
    fn test_normalize_path() {
        // Test basic normalization
        let path = normalize_path(Path::new("/home/user/../user/./docs"));
        assert_eq!(path, Path::new("/home/user/docs"));

        // Test with multiple dots
        let path = normalize_path(Path::new("./foo/./bar/../baz"));
        assert_eq!(path, Path::new("foo/baz"));

        // Test with trailing slash
        let path = normalize_path(Path::new("/home/user/"));
        assert_eq!(path, Path::new("/home/user"));
    }

    #[test]
    fn test_is_absolute_path() {
        assert!(is_absolute_path(Path::new("/home/user")));
        assert!(!is_absolute_path(Path::new("relative/path")));
        assert!(!is_absolute_path(Path::new("./relative")));
        assert!(!is_absolute_path(Path::new("../parent")));
    }

    #[test]
    fn test_join_paths() {
        // Basic join
        let joined = join_paths(&[Path::new("/home"), Path::new("user"), Path::new("docs")]);
        assert_eq!(joined, Path::new("/home/user/docs"));

        // Join with absolute path in middle (should reset)
        let joined = join_paths(&[Path::new("/home"), Path::new("/usr"), Path::new("bin")]);
        assert_eq!(joined, Path::new("/usr/bin"));

        // Join with empty components
        let joined = join_paths(&[Path::new("/home"), Path::new(""), Path::new("user")]);
        assert_eq!(joined, Path::new("/home/user"));
    }

    #[test]
    fn test_parent_dir() {
        assert_eq!(
            parent_dir(Path::new("/home/user/file.txt")),
            Some(Path::new("/home/user").to_path_buf())
        );

        assert_eq!(
            parent_dir(Path::new("/home")),
            Some(Path::new("/").to_path_buf())
        );

        // Root has no parent
        assert_eq!(parent_dir(Path::new("/")), None);
    }

    #[test]
    fn test_expand_path_tilde() {
        // Set HOME for consistent testing
        let original_home = env::var("HOME").ok();
        env::set_var("HOME", "/test/home");

        assert_eq!(expand_path("~").unwrap(), Path::new("/test/home"));

        assert_eq!(
            expand_path("~/Documents").unwrap(),
            Path::new("/test/home/Documents")
        );

        // Restore original HOME
        if let Some(home) = original_home {
            env::set_var("HOME", home);
        } else {
            env::remove_var("HOME");
        }
    }

    #[test]
    fn test_expand_path_env_vars() {
        // Set test environment variable
        env::set_var("TEST_VAR", "/test/path");

        // Test $VAR syntax
        assert_eq!(
            expand_path("$TEST_VAR/file.txt").unwrap(),
            Path::new("/test/path/file.txt")
        );

        // Test ${VAR} syntax
        assert_eq!(
            expand_path("${TEST_VAR}/file.txt").unwrap(),
            Path::new("/test/path/file.txt")
        );

        // Test missing variable
        assert!(expand_path("$NONEXISTENT_VAR/file.txt").is_err());

        // Cleanup
        env::remove_var("TEST_VAR");
    }

    #[test]
    fn test_ensure_dir() {
        let temp_dir = std::env::temp_dir();
        let test_dir = temp_dir.join(format!("llmspell_test_{}", uuid::Uuid::new_v4()));

        // Test creating new directory
        assert!(ensure_dir(&test_dir).is_ok());
        assert!(test_dir.exists());
        assert!(test_dir.is_dir());

        // Test calling on existing directory (should be no-op)
        assert!(ensure_dir(&test_dir).is_ok());

        // Test with nested directories
        let nested = test_dir.join("a/b/c");
        assert!(ensure_dir(&nested).is_ok());
        assert!(nested.exists());

        // Cleanup
        let _ = fs::remove_dir_all(&test_dir);
    }

    #[test]
    fn test_ensure_dir_file_exists() {
        let temp_dir = std::env::temp_dir();
        let test_file = temp_dir.join(format!("llmspell_test_file_{}", uuid::Uuid::new_v4()));

        // Create a file
        fs::write(&test_file, "test").unwrap();

        // Try to ensure_dir on a file (should fail)
        let result = ensure_dir(&test_file);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().kind(), io::ErrorKind::AlreadyExists);

        // Cleanup
        let _ = fs::remove_file(&test_file);
    }

    #[test]
    fn test_read_write_file() {
        let temp_dir = std::env::temp_dir();
        let test_file = temp_dir.join(format!("llmspell_test_rw_{}", uuid::Uuid::new_v4()));

        let data = b"Hello, world!";

        // Test write
        assert!(write_file(&test_file, data).is_ok());
        assert!(test_file.exists());

        // Test read
        let read_data = read_file(&test_file).unwrap();
        assert_eq!(read_data, data);

        // Cleanup
        let _ = fs::remove_file(&test_file);
    }

    #[test]
    fn test_write_file_creates_parent_dirs() {
        let temp_dir = std::env::temp_dir();
        let test_file = temp_dir.join(format!(
            "llmspell_test_{}/nested/file.txt",
            uuid::Uuid::new_v4()
        ));

        let data = b"Test data";

        // Parent directories don't exist yet
        assert!(!test_file.parent().unwrap().exists());

        // Write should create parent directories
        assert!(write_file(&test_file, data).is_ok());
        assert!(test_file.exists());

        // Verify content
        let read_data = read_file(&test_file).unwrap();
        assert_eq!(read_data, data);

        // Cleanup
        let _ = fs::remove_dir_all(test_file.parent().unwrap().parent().unwrap());
    }

    #[test]
    fn test_atomic_write() {
        let temp_dir = std::env::temp_dir();
        let test_file = temp_dir.join(format!("llmspell_test_atomic_{}", uuid::Uuid::new_v4()));

        let data1 = b"Initial data";
        let data2 = b"Updated data";

        // Initial write
        assert!(write_file_atomic(&test_file, data1).is_ok());
        assert_eq!(read_file(&test_file).unwrap(), data1);

        // Atomic update
        assert!(write_file_atomic(&test_file, data2).is_ok());
        assert_eq!(read_file(&test_file).unwrap(), data2);

        // Verify no temp files remain
        let parent = test_file.parent().unwrap();
        for entry in fs::read_dir(parent).unwrap() {
            let entry = entry.unwrap();
            let name = entry.file_name();
            let name_str = name.to_string_lossy();
            assert!(!name_str.contains(".tmp."), "Found temp file: {name_str}");
        }

        // Cleanup
        let _ = fs::remove_file(&test_file);
    }

    #[test]
    fn test_copy_file() {
        let temp_dir = std::env::temp_dir();
        let source = temp_dir.join(format!("llmspell_test_src_{}", uuid::Uuid::new_v4()));
        let dest = temp_dir.join(format!("llmspell_test_dst_{}", uuid::Uuid::new_v4()));

        let data = b"Test data for copy";

        // Create source file
        write_file(&source, data).unwrap();

        // Copy file
        let bytes_copied = copy_file(&source, &dest).unwrap();
        assert_eq!(bytes_copied, data.len() as u64);

        // Verify destination
        assert!(dest.exists());
        assert_eq!(read_file(&dest).unwrap(), data);

        // Cleanup
        let _ = fs::remove_file(&source);
        let _ = fs::remove_file(&dest);
    }

    #[test]
    fn test_remove_file_if_exists() {
        let temp_dir = std::env::temp_dir();
        let test_file = temp_dir.join(format!("llmspell_test_rm_{}", uuid::Uuid::new_v4()));

        // Remove non-existent file (should succeed)
        assert!(remove_file_if_exists(&test_file).is_ok());

        // Create and remove file
        write_file(&test_file, b"test").unwrap();
        assert!(test_file.exists());
        assert!(remove_file_if_exists(&test_file).is_ok());
        assert!(!test_file.exists());

        // Remove again (should still succeed)
        assert!(remove_file_if_exists(&test_file).is_ok());
    }

    #[test]
    fn test_remove_dir_all_if_exists() {
        let temp_dir = std::env::temp_dir();
        let test_dir = temp_dir.join(format!("llmspell_test_rmdir_{}", uuid::Uuid::new_v4()));

        // Remove non-existent directory (should succeed)
        assert!(remove_dir_all_if_exists(&test_dir).is_ok());

        // Create directory with contents
        let nested = test_dir.join("nested");
        ensure_dir(&nested).unwrap();
        write_file(&nested.join("file.txt"), b"test").unwrap();

        assert!(test_dir.exists());
        assert!(remove_dir_all_if_exists(&test_dir).is_ok());
        assert!(!test_dir.exists());

        // Remove again (should still succeed)
        assert!(remove_dir_all_if_exists(&test_dir).is_ok());
    }

    #[test]
    fn test_append_file() {
        let temp_dir = std::env::temp_dir();
        let test_file = temp_dir.join(format!("llmspell_test_append_{}", uuid::Uuid::new_v4()));

        // Append to non-existent file (should create it)
        assert!(append_file(&test_file, b"Hello").is_ok());
        assert_eq!(read_file(&test_file).unwrap(), b"Hello");

        // Append to existing file
        assert!(append_file(&test_file, b" World").is_ok());
        assert_eq!(read_file(&test_file).unwrap(), b"Hello World");

        // Append with newline
        assert!(append_file(&test_file, b"\nNew line").is_ok());
        assert_eq!(read_file(&test_file).unwrap(), b"Hello World\nNew line");

        // Cleanup
        let _ = fs::remove_file(&test_file);
    }

    #[test]
    fn test_append_file_creates_parent_dirs() {
        let temp_dir = std::env::temp_dir();
        let test_file = temp_dir.join(format!(
            "llmspell_test_{}/nested/append.txt",
            uuid::Uuid::new_v4()
        ));

        // Parent directories don't exist yet
        assert!(!test_file.parent().unwrap().exists());

        // Append should create parent directories
        assert!(append_file(&test_file, b"Test").is_ok());
        assert!(test_file.exists());
        assert_eq!(read_file(&test_file).unwrap(), b"Test");

        // Cleanup
        let _ = fs::remove_dir_all(test_file.parent().unwrap().parent().unwrap());
    }

    #[test]
    fn test_move_file() {
        let temp_dir = std::env::temp_dir();
        let source = temp_dir.join(format!("llmspell_test_src_{}", uuid::Uuid::new_v4()));
        let dest = temp_dir.join(format!("llmspell_test_dst_{}", uuid::Uuid::new_v4()));

        let data = b"Test data for move";

        // Create source file
        write_file(&source, data).unwrap();
        assert!(source.exists());

        // Move file
        assert!(move_file(&source, &dest).is_ok());

        // Verify move
        assert!(!source.exists(), "Source file should not exist after move");
        assert!(dest.exists(), "Destination file should exist after move");
        assert_eq!(read_file(&dest).unwrap(), data);

        // Cleanup
        let _ = fs::remove_file(&dest);
    }

    #[test]
    fn test_move_file_creates_parent_dirs() {
        let temp_dir = std::env::temp_dir();
        let source = temp_dir.join(format!("llmspell_test_src_{}", uuid::Uuid::new_v4()));
        let dest = temp_dir.join(format!(
            "llmspell_test_{}/nested/moved.txt",
            uuid::Uuid::new_v4()
        ));

        // Create source file
        write_file(&source, b"test").unwrap();

        // Parent directories don't exist yet
        assert!(!dest.parent().unwrap().exists());

        // Move should create parent directories
        assert!(move_file(&source, &dest).is_ok());
        assert!(!source.exists());
        assert!(dest.exists());

        // Cleanup
        let _ = fs::remove_dir_all(dest.parent().unwrap().parent().unwrap());
    }

    #[test]
    fn test_get_metadata() {
        let temp_dir = std::env::temp_dir();
        let test_file = temp_dir.join(format!("llmspell_test_meta_{}", uuid::Uuid::new_v4()));
        let test_dir = temp_dir.join(format!("llmspell_test_metadir_{}", uuid::Uuid::new_v4()));

        // Test file metadata
        let data = b"Test content";
        write_file(&test_file, data).unwrap();

        let metadata = get_metadata(&test_file).unwrap();
        assert_eq!(metadata.size, data.len() as u64);
        assert!(metadata.is_file);
        assert!(!metadata.is_dir);
        assert!(!metadata.is_symlink);
        assert!(metadata.modified.is_some());

        // Test directory metadata
        ensure_dir(&test_dir).unwrap();
        let dir_metadata = get_metadata(&test_dir).unwrap();
        assert!(dir_metadata.is_dir);
        assert!(!dir_metadata.is_file);

        // Cleanup
        let _ = fs::remove_file(&test_file);
        let _ = fs::remove_dir(&test_dir);
    }

    #[test]
    fn test_file_exists() {
        let temp_dir = std::env::temp_dir();
        let test_file = temp_dir.join(format!("llmspell_test_exists_{}", uuid::Uuid::new_v4()));

        // File doesn't exist
        assert!(!file_exists(&test_file));

        // Create file
        write_file(&test_file, b"test").unwrap();
        assert!(file_exists(&test_file));

        // Remove file
        fs::remove_file(&test_file).unwrap();
        assert!(!file_exists(&test_file));
    }

    #[test]
    fn test_list_dir() {
        let temp_dir = std::env::temp_dir();
        let test_dir = temp_dir.join(format!("llmspell_test_list_{}", uuid::Uuid::new_v4()));

        // Create test directory structure
        ensure_dir(&test_dir).unwrap();
        write_file(&test_dir.join("file1.txt"), b"content1").unwrap();
        write_file(&test_dir.join("file2.txt"), b"content22").unwrap();
        ensure_dir(&test_dir.join("subdir")).unwrap();
        write_file(&test_dir.join("subdir/nested.txt"), b"nested").unwrap();

        // List directory
        let entries = list_dir(&test_dir).unwrap();

        // Verify entries (should be sorted alphabetically)
        assert_eq!(entries.len(), 3);

        assert_eq!(entries[0].name, "file1.txt");
        assert!(entries[0].is_file);
        assert!(!entries[0].is_dir);
        assert_eq!(entries[0].size, 8); // "content1"

        assert_eq!(entries[1].name, "file2.txt");
        assert!(entries[1].is_file);
        assert_eq!(entries[1].size, 9); // "content22"

        assert_eq!(entries[2].name, "subdir");
        assert!(entries[2].is_dir);
        assert!(!entries[2].is_file);

        // All entries should have modification times
        for entry in &entries {
            assert!(entry.modified.is_some());
        }

        // Cleanup
        let _ = fs::remove_dir_all(&test_dir);
    }

    #[test]
    fn test_list_empty_dir() {
        let temp_dir = std::env::temp_dir();
        let test_dir = temp_dir.join(format!("llmspell_test_empty_{}", uuid::Uuid::new_v4()));

        ensure_dir(&test_dir).unwrap();
        let entries = list_dir(&test_dir).unwrap();
        assert!(entries.is_empty());

        // Cleanup
        let _ = fs::remove_dir(&test_dir);
    }

    #[test]
    fn test_list_dir_error() {
        let temp_dir = std::env::temp_dir();
        let test_file = temp_dir.join(format!("llmspell_test_notdir_{}", uuid::Uuid::new_v4()));

        // Create a file, not a directory
        write_file(&test_file, b"test").unwrap();

        // Trying to list a file should fail
        let result = list_dir(&test_file);
        assert!(result.is_err());

        // Cleanup
        let _ = fs::remove_file(&test_file);
    }

    #[cfg(unix)]
    #[test]
    fn test_unix_specific_paths() {
        // Test Unix-specific path handling
        assert!(is_absolute_path(Path::new("/usr/bin")));
        assert!(is_absolute_path(Path::new("/home/user/.config")));
        assert!(!is_absolute_path(Path::new("usr/bin")));
    }

    #[cfg(windows)]
    #[test]
    fn test_windows_specific_paths() {
        // Test Windows-specific path handling
        assert!(is_absolute_path(Path::new("C:\\Windows")));
        assert!(is_absolute_path(Path::new("\\\\server\\share")));
        assert!(!is_absolute_path(Path::new("Windows\\System32")));

        // Test Windows environment variable syntax
        env::set_var("WINTEST", "C:\\Test");
        assert_eq!(
            expand_path("%WINTEST%\\file.txt").unwrap(),
            Path::new("C:\\Test\\file.txt")
        );
        env::remove_var("WINTEST");
    }
}

#[cfg(test)]
mod property_tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn test_normalize_path_idempotent(path in prop::collection::vec("[a-zA-Z0-9./\\-_]+", 1..10)) {
            let path_str = path.join("/");
            let path = Path::new(&path_str);
            let normalized = normalize_path(path);
            let normalized_again = normalize_path(&normalized);
            assert_eq!(normalized, normalized_again);
        }

        #[test]
        fn test_join_paths_associative(
            a in "[a-zA-Z0-9]+",
            b in "[a-zA-Z0-9]+",
            c in "[a-zA-Z0-9]+"
        ) {
            let path_a = Path::new(&a);
            let path_b = Path::new(&b);
            let path_c = Path::new(&c);

            let result1 = join_paths(&[&join_paths(&[path_a, path_b]), path_c]);
            let result2 = join_paths(&[path_a, &join_paths(&[path_b, path_c])]);
            assert_eq!(result1, result2);
        }

        #[test]
        fn test_write_read_roundtrip(data: Vec<u8>) {
            let temp_dir = std::env::temp_dir();
            let test_file = temp_dir.join(format!("llmspell_prop_{}", uuid::Uuid::new_v4()));

            // Skip if data is too large
            if data.len() > 1_000_000 {
                return Ok(());
            }

            write_file(&test_file, &data).unwrap();
            let read_data = read_file(&test_file).unwrap();
            assert_eq!(data, read_data);

            // Cleanup
            let _ = fs::remove_file(&test_file);
        }
    }
}

#[cfg(all(test, not(debug_assertions)))]
mod benchmarks {
    use super::*;
    use criterion::{black_box, Criterion};

    pub fn bench_normalize_path(c: &mut Criterion) {
        c.bench_function("normalize_path", |b| {
            b.iter(|| normalize_path(black_box(Path::new("/home/user/../user/./docs/./file.txt"))));
        });
    }

    pub fn bench_expand_path(c: &mut Criterion) {
        env::set_var("BENCH_VAR", "/test/path");

        c.bench_function("expand_path", |b| {
            b.iter(|| expand_path(black_box("$BENCH_VAR/subdir/file.txt")).unwrap());
        });

        env::remove_var("BENCH_VAR");
    }

    pub fn bench_write_file(c: &mut Criterion) {
        let temp_dir = std::env::temp_dir();
        let test_file = temp_dir.join("llmspell_bench_write");
        let data = vec![b'x'; 1024]; // 1KB of data

        c.bench_function("write_file_1kb", |b| {
            b.iter(|| write_file(black_box(&test_file), black_box(&data)).unwrap());
        });

        // Cleanup
        let _ = fs::remove_file(&test_file);
    }

    pub fn bench_write_file_atomic(c: &mut Criterion) {
        let temp_dir = std::env::temp_dir();
        let test_file = temp_dir.join("llmspell_bench_atomic");
        let data = vec![b'x'; 1024]; // 1KB of data

        c.bench_function("write_file_atomic_1kb", |b| {
            b.iter(|| write_file_atomic(black_box(&test_file), black_box(&data)).unwrap());
        });

        // Cleanup
        let _ = fs::remove_file(&test_file);
    }

    pub fn bench_read_file(c: &mut Criterion) {
        let temp_dir = std::env::temp_dir();
        let test_file = temp_dir.join("llmspell_bench_read");
        let data = vec![b'x'; 1024]; // 1KB of data

        // Create file
        write_file(&test_file, &data).unwrap();

        c.bench_function("read_file_1kb", |b| {
            b.iter(|| read_file(black_box(&test_file)).unwrap());
        });

        // Cleanup
        let _ = fs::remove_file(&test_file);
    }
}
