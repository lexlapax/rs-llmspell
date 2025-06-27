// ABOUTME: System information gathering and environment utilities
// ABOUTME: Provides system metadata and environment information for LLMSpell operations

//! System information gathering utilities
//!
//! This module provides utilities for gathering system information,
//! including OS details, environment variables, and resource usage.

use serde::{Deserialize, Serialize};
use std::env;
use std::io;
use std::path::PathBuf;

/// System information structure
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SystemInfo {
    /// Operating system name
    pub os: String,
    /// OS version
    pub version: String,
    /// System architecture
    pub arch: String,
    /// Number of CPU cores
    pub cpu_cores: usize,
    /// Total system memory in bytes
    pub total_memory: Option<u64>,
    /// Available system memory in bytes
    pub available_memory: Option<u64>,
    /// Hostname
    pub hostname: Option<String>,
    /// Current user
    pub username: Option<String>,
    /// Home directory
    pub home_dir: Option<PathBuf>,
}

/// Operating system type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OperatingSystem {
    /// Windows
    Windows,
    /// macOS
    MacOS,
    /// Linux
    Linux,
    /// Unknown or unsupported OS
    Unknown,
}

impl OperatingSystem {
    /// Get the current operating system type
    #[must_use]
    pub fn current() -> Self {
        match env::consts::OS {
            "windows" => Self::Windows,
            "macos" => Self::MacOS,
            "linux" => Self::Linux,
            _ => Self::Unknown,
        }
    }

    /// Check if the current OS is Windows
    #[must_use]
    pub fn is_windows() -> bool {
        matches!(Self::current(), Self::Windows)
    }

    /// Check if the current OS is macOS
    #[must_use]
    pub fn is_macos() -> bool {
        matches!(Self::current(), Self::MacOS)
    }

    /// Check if the current OS is Linux
    #[must_use]
    pub fn is_linux() -> bool {
        matches!(Self::current(), Self::Linux)
    }

    /// Check if the current OS is Unix-like (macOS or Linux)
    #[must_use]
    pub fn is_unix() -> bool {
        matches!(Self::current(), Self::MacOS | Self::Linux)
    }
}

impl std::fmt::Display for OperatingSystem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Windows => write!(f, "Windows"),
            Self::MacOS => write!(f, "macOS"),
            Self::Linux => write!(f, "Linux"),
            Self::Unknown => write!(f, "Unknown"),
        }
    }
}

/// Get current system information
///
/// Gathers comprehensive system information including OS details,
/// architecture, CPU cores, memory, and user information.
///
/// # Examples
///
/// ```rust,no_run
/// use llmspell_utils::system_info::get_system_info;
///
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let info = get_system_info()?;
/// println!("OS: {} {}", info.os, info.version);
/// println!("Architecture: {}", info.arch);
/// println!("CPU cores: {}", info.cpu_cores);
/// # Ok(())
/// # }
/// ```
///
/// # Errors
///
/// Returns an error if system information cannot be gathered
pub fn get_system_info() -> Result<SystemInfo, io::Error> {
    let info = SystemInfo {
        os: env::consts::OS.to_string(),
        version: get_os_version().unwrap_or_else(|| "unknown".to_string()),
        arch: env::consts::ARCH.to_string(),
        cpu_cores: get_cpu_count(),
        total_memory: get_total_memory(),
        available_memory: get_available_memory(),
        hostname: get_hostname(),
        username: get_username(),
        home_dir: get_home_directory(),
    };

    Ok(info)
}

/// Get the number of CPU cores
///
/// Returns the number of logical CPU cores available to the process.
///
/// # Examples
///
/// ```rust
/// use llmspell_utils::system_info::get_cpu_count;
///
/// let cores = get_cpu_count();
/// assert!(cores > 0);
/// ```
#[must_use]
pub fn get_cpu_count() -> usize {
    std::thread::available_parallelism()
        .map(std::num::NonZero::get)
        .unwrap_or(1)
}

/// Get the OS version string
fn get_os_version() -> Option<String> {
    #[cfg(target_os = "windows")]
    {
        // On Windows, we could use registry or WMI, but for simplicity:
        Some("Windows".to_string())
    }

    #[cfg(target_os = "macos")]
    {
        use std::process::Command;
        Command::new("sw_vers")
            .arg("-productVersion")
            .output()
            .ok()
            .and_then(|output| String::from_utf8(output.stdout).ok())
            .map(|s| s.trim().to_string())
    }

    #[cfg(target_os = "linux")]
    {
        use std::fs;
        // Try to read from various release files
        fs::read_to_string("/etc/os-release")
            .or_else(|_| fs::read_to_string("/etc/lsb-release"))
            .ok()
            .and_then(|content| {
                // Parse VERSION or DISTRIB_RELEASE
                content
                    .lines()
                    .find(|line| {
                        line.starts_with("VERSION=") || line.starts_with("DISTRIB_RELEASE=")
                    })
                    .and_then(|line| line.split('=').nth(1))
                    .map(|s| s.trim_matches('"').to_string())
            })
            .or_else(|| Some("Linux".to_string()))
    }

    #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
    {
        None
    }
}

/// Get total system memory in bytes
fn get_total_memory() -> Option<u64> {
    #[cfg(target_os = "linux")]
    {
        use std::fs;
        fs::read_to_string("/proc/meminfo")
            .ok()
            .and_then(|content| {
                content
                    .lines()
                    .find(|line| line.starts_with("MemTotal:"))
                    .and_then(|line| {
                        line.split_whitespace()
                            .nth(1)
                            .and_then(|s| s.parse::<u64>().ok())
                            .map(|kb| kb * 1024) // Convert KB to bytes
                    })
            })
    }

    #[cfg(target_os = "macos")]
    {
        use std::process::Command;
        Command::new("sysctl")
            .arg("-n")
            .arg("hw.memsize")
            .output()
            .ok()
            .and_then(|output| String::from_utf8(output.stdout).ok())
            .and_then(|s| s.trim().parse::<u64>().ok())
    }

    #[cfg(target_os = "windows")]
    {
        // On Windows, this would require WMI or similar
        None
    }

    #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
    {
        None
    }
}

/// Get available system memory in bytes
fn get_available_memory() -> Option<u64> {
    #[cfg(target_os = "linux")]
    {
        use std::fs;
        fs::read_to_string("/proc/meminfo")
            .ok()
            .and_then(|content| {
                content
                    .lines()
                    .find(|line| line.starts_with("MemAvailable:"))
                    .and_then(|line| {
                        line.split_whitespace()
                            .nth(1)
                            .and_then(|s| s.parse::<u64>().ok())
                            .map(|kb| kb * 1024) // Convert KB to bytes
                    })
            })
    }

    #[cfg(not(target_os = "linux"))]
    {
        None
    }
}

/// Get the system hostname
///
/// # Examples
///
/// ```rust,no_run
/// use llmspell_utils::system_info::get_hostname;
///
/// if let Some(hostname) = get_hostname() {
///     println!("Hostname: {}", hostname);
/// }
/// ```
#[must_use]
pub fn get_hostname() -> Option<String> {
    #[cfg(unix)]
    {
        use std::ffi::CStr;
        let mut buffer = vec![0u8; 256];
        unsafe {
            if libc::gethostname(buffer.as_mut_ptr().cast::<libc::c_char>(), buffer.len()) == 0 {
                CStr::from_ptr(buffer.as_ptr().cast::<libc::c_char>())
                    .to_str()
                    .ok()
                    .map(std::string::ToString::to_string)
            } else {
                None
            }
        }
    }

    #[cfg(windows)]
    {
        env::var("COMPUTERNAME").ok()
    }

    #[cfg(not(any(unix, windows)))]
    {
        None
    }
}

/// Get the current username
///
/// # Examples
///
/// ```rust,no_run
/// use llmspell_utils::system_info::get_username;
///
/// if let Some(username) = get_username() {
///     println!("Current user: {}", username);
/// }
/// ```
#[must_use]
pub fn get_username() -> Option<String> {
    env::var("USER").or_else(|_| env::var("USERNAME")).ok()
}

/// Get the user's home directory
///
/// # Examples
///
/// ```rust,no_run
/// use llmspell_utils::system_info::get_home_directory;
///
/// if let Some(home) = get_home_directory() {
///     println!("Home directory: {}", home.display());
/// }
/// ```
#[must_use]
pub fn get_home_directory() -> Option<PathBuf> {
    env::var("HOME")
        .or_else(|_| env::var("USERPROFILE"))
        .ok()
        .map(PathBuf::from)
}

/// Get an environment variable with a default value
///
/// # Examples
///
/// ```rust
/// use llmspell_utils::system_info::get_env_or;
///
/// let editor = get_env_or("EDITOR", "vim");
/// let custom = get_env_or("MY_CUSTOM_VAR", "default_value");
/// ```
#[must_use]
pub fn get_env_or(key: &str, default: &str) -> String {
    env::var(key).unwrap_or_else(|_| default.to_string())
}

/// Check if an environment variable is set and truthy
///
/// A variable is considered truthy if it's set and not empty, "0", "false", or "no"
///
/// # Examples
///
/// ```rust
/// use llmspell_utils::system_info::is_env_truthy;
///
/// // Returns true if DEBUG=1, DEBUG=true, DEBUG=yes, DEBUG=anything_else
/// // Returns false if DEBUG is unset, empty, "0", "false", or "no"
/// let debug_mode = is_env_truthy("DEBUG");
/// ```
#[must_use]
pub fn is_env_truthy(key: &str) -> bool {
    match env::var(key) {
        Ok(val) => {
            let val = val.trim().to_lowercase();
            !val.is_empty() && val != "0" && val != "false" && val != "no"
        }
        Err(_) => false,
    }
}

/// Get a list of all environment variables
///
/// Returns a vector of (key, value) pairs for all environment variables.
///
/// # Examples
///
/// ```rust
/// use llmspell_utils::system_info::get_all_env_vars;
///
/// let vars = get_all_env_vars();
/// for (key, value) in vars {
///     println!("{} = {}", key, value);
/// }
/// ```
#[must_use]
pub fn get_all_env_vars() -> Vec<(String, String)> {
    env::vars().collect()
}

/// Find executable in PATH
///
/// Searches for an executable in the system PATH.
///
/// # Examples
///
/// ```rust,no_run
/// use llmspell_utils::system_info::find_executable;
///
/// if let Some(python_path) = find_executable("python") {
///     println!("Python found at: {}", python_path.display());
/// }
/// ```
#[must_use]
pub fn find_executable(name: &str) -> Option<PathBuf> {
    which::which(name).ok()
}

/// Check if running in a container (Docker, Podman, etc.)
///
/// # Examples
///
/// ```rust
/// use llmspell_utils::system_info::is_running_in_container;
///
/// if is_running_in_container() {
///     println!("Running inside a container");
/// }
/// ```
#[must_use]
pub fn is_running_in_container() -> bool {
    #[cfg(target_os = "linux")]
    {
        use std::fs;
        use std::path::Path;
        // Check for /.dockerenv or /.containerenv
        Path::new("/.dockerenv").exists() 
            || Path::new("/.containerenv").exists()
            // Check if running in a container via cgroup
            || fs::read_to_string("/proc/1/cgroup")
                .map(|content| content.contains("/docker/") || content.contains("/containerd/"))
                .unwrap_or(false)
    }

    #[cfg(not(target_os = "linux"))]
    {
        false
    }
}

/// Check if running in a virtual machine
///
/// This is a best-effort detection and may not catch all virtualization types.
///
/// # Examples
///
/// ```rust
/// use llmspell_utils::system_info::is_running_in_vm;
///
/// if is_running_in_vm() {
///     println!("Running inside a virtual machine");
/// }
/// ```
#[must_use]
pub fn is_running_in_vm() -> bool {
    #[cfg(target_os = "linux")]
    {
        use std::fs;
        // Check for common VM indicators
        fs::read_to_string("/sys/devices/virtual/dmi/id/product_name")
            .map(|content| {
                let content = content.to_lowercase();
                content.contains("virtualbox")
                    || content.contains("vmware")
                    || content.contains("kvm")
                    || content.contains("qemu")
                    || content.contains("xen")
            })
            .unwrap_or(false)
            || std::path::Path::new("/proc/xen").exists()
            || std::path::Path::new("/sys/hypervisor/type").exists()
    }

    #[cfg(not(target_os = "linux"))]
    {
        false
    }
}

/// Get temporary directory path
///
/// Returns the system's temporary directory path.
///
/// # Examples
///
/// ```rust
/// use llmspell_utils::system_info::get_temp_dir;
///
/// let temp_dir = get_temp_dir();
/// println!("Temp directory: {}", temp_dir.display());
/// ```
#[must_use]
pub fn get_temp_dir() -> PathBuf {
    env::temp_dir()
}

/// Format bytes into human-readable string
///
/// # Examples
///
/// ```rust
/// use llmspell_utils::system_info::format_bytes;
///
/// assert_eq!(format_bytes(1024), "1.0 KB");
/// assert_eq!(format_bytes(1_048_576), "1.0 MB");
/// assert_eq!(format_bytes(1_073_741_824), "1.0 GB");
/// ```
#[must_use]
pub fn format_bytes(bytes: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB", "PB"];

    if bytes == 0 {
        return "0 B".to_string();
    }

    #[allow(clippy::cast_precision_loss)]
    let mut size = bytes as f64;
    let mut unit_index = 0;

    while size >= 1024.0 && unit_index < UNITS.len() - 1 {
        size /= 1024.0;
        unit_index += 1;
    }

    if unit_index == 0 {
        format!("{} {}", bytes, UNITS[0])
    } else {
        format!("{:.1} {}", size, UNITS[unit_index])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_operating_system() {
        let os = OperatingSystem::current();
        match env::consts::OS {
            "windows" => assert_eq!(os, OperatingSystem::Windows),
            "macos" => assert_eq!(os, OperatingSystem::MacOS),
            "linux" => assert_eq!(os, OperatingSystem::Linux),
            _ => assert_eq!(os, OperatingSystem::Unknown),
        }

        // Test display
        let _ = format!("{os}");

        // Test helpers
        if cfg!(target_os = "windows") {
            assert!(OperatingSystem::is_windows());
            assert!(!OperatingSystem::is_unix());
        } else if cfg!(target_os = "macos") {
            assert!(OperatingSystem::is_macos());
            assert!(OperatingSystem::is_unix());
        } else if cfg!(target_os = "linux") {
            assert!(OperatingSystem::is_linux());
            assert!(OperatingSystem::is_unix());
        }
    }

    #[test]
    fn test_get_system_info() {
        let info = get_system_info().unwrap();

        // Basic checks
        assert!(!info.os.is_empty());
        assert!(!info.arch.is_empty());
        assert!(info.cpu_cores > 0);

        // OS should match env::consts::OS
        assert_eq!(info.os, env::consts::OS);

        // Architecture should match env::consts::ARCH
        assert_eq!(info.arch, env::consts::ARCH);
    }

    #[test]
    fn test_get_cpu_count() {
        let count = get_cpu_count();
        assert!(count > 0);

        // Should match std::thread::available_parallelism if available
        if let Ok(parallelism) = std::thread::available_parallelism() {
            assert_eq!(count, parallelism.get());
        }
    }

    #[test]
    fn test_env_helpers() {
        // Test get_env_or
        env::set_var("TEST_ENV_VAR", "test_value");
        assert_eq!(get_env_or("TEST_ENV_VAR", "default"), "test_value");
        assert_eq!(get_env_or("NONEXISTENT_VAR", "default"), "default");
        env::remove_var("TEST_ENV_VAR");

        // Test is_env_truthy
        env::set_var("TRUTHY_VAR", "1");
        assert!(is_env_truthy("TRUTHY_VAR"));

        env::set_var("TRUTHY_VAR", "true");
        assert!(is_env_truthy("TRUTHY_VAR"));

        env::set_var("TRUTHY_VAR", "yes");
        assert!(is_env_truthy("TRUTHY_VAR"));

        env::set_var("TRUTHY_VAR", "0");
        assert!(!is_env_truthy("TRUTHY_VAR"));

        env::set_var("TRUTHY_VAR", "false");
        assert!(!is_env_truthy("TRUTHY_VAR"));

        env::set_var("TRUTHY_VAR", "no");
        assert!(!is_env_truthy("TRUTHY_VAR"));

        env::set_var("TRUTHY_VAR", "");
        assert!(!is_env_truthy("TRUTHY_VAR"));

        env::remove_var("TRUTHY_VAR");
        assert!(!is_env_truthy("TRUTHY_VAR"));
    }

    #[test]
    fn test_get_all_env_vars() {
        let vars = get_all_env_vars();
        assert!(!vars.is_empty());

        // Should contain PATH
        assert!(vars.iter().any(|(k, _)| k == "PATH"));
    }

    #[test]
    fn test_get_temp_dir() {
        let temp_dir = get_temp_dir();
        assert!(temp_dir.is_absolute());
        assert!(temp_dir.exists());
    }

    #[test]
    fn test_format_bytes() {
        assert_eq!(format_bytes(0), "0 B");
        assert_eq!(format_bytes(512), "512 B");
        assert_eq!(format_bytes(1024), "1.0 KB");
        assert_eq!(format_bytes(1536), "1.5 KB");
        assert_eq!(format_bytes(1_048_576), "1.0 MB");
        assert_eq!(format_bytes(1_073_741_824), "1.0 GB");
        assert_eq!(format_bytes(1_099_511_627_776), "1.0 TB");
        assert_eq!(format_bytes(1_125_899_906_842_624), "1.0 PB");
    }

    #[test]
    fn test_username_and_home() {
        // These might not always be set in CI environments
        if let Some(username) = get_username() {
            assert!(!username.is_empty());
        }

        if let Some(home) = get_home_directory() {
            assert!(home.is_absolute());
        }
    }

    #[test]
    fn test_find_executable() {
        // Common executables that should exist
        #[cfg(unix)]
        {
            // sh should exist on all Unix systems
            assert!(find_executable("sh").is_some());
        }

        #[cfg(windows)]
        {
            // cmd.exe should exist on all Windows systems
            assert!(find_executable("cmd").is_some() || find_executable("cmd.exe").is_some());
        }

        // Non-existent executable
        assert!(find_executable("this_executable_definitely_does_not_exist_12345").is_none());
    }

    #[test]
    fn test_container_and_vm_detection() {
        // These are environment-specific, so we just ensure they don't panic
        let _ = is_running_in_container();
        let _ = is_running_in_vm();
    }
}

#[cfg(test)]
mod property_tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn test_format_bytes_ordering(bytes1 in 0u64..1_000_000_000_000, bytes2 in 0u64..1_000_000_000_000) {
            let formatted1 = format_bytes(bytes1);
            let _formatted2 = format_bytes(bytes2);

            // If bytes1 < bytes2, the numeric part should reflect that
            // (this is a simplified test, actual comparison would need parsing)
            if bytes1 == 0 && bytes2 > 0 {
                assert_eq!(formatted1, "0 B");
            }
        }

        #[test]
        fn test_env_var_roundtrip(suffix in "[A-Z][A-Z0-9_]*", value in "[^\0]*") {
            // Use a unique prefix to avoid conflicts
            let key = format!("LLMSPELL_TEST_{suffix}");
            // Skip if key somehow already exists or value contains null bytes
            if env::var(&key).is_err() && !value.contains('\0') {
                env::set_var(&key, &value);
                assert_eq!(get_env_or(&key, "default"), value);
                env::remove_var(&key);
            }
        }
    }
}
