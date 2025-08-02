// ABOUTME: System resource monitoring tool for tracking CPU, memory, and disk usage
// ABOUTME: Provides comprehensive system statistics and resource utilization monitoring

use async_trait::async_trait;
use llmspell_core::{
    traits::{
        base_agent::BaseAgent,
        tool::{ParameterDef, ParameterType, SecurityLevel, Tool, ToolCategory, ToolSchema},
    },
    types::{AgentInput, AgentOutput},
    ComponentMetadata, ExecutionContext, LLMSpellError, Result as LLMResult,
};
use llmspell_security::sandbox::SandboxContext;
use llmspell_utils::{
    extract_optional_string, extract_parameters,
    response::ResponseBuilder,
    system_info::{format_bytes, get_cpu_count, get_system_info},
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;
use tracing::{debug, info};

/// System resource statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemStats {
    /// CPU usage percentage (0-100)
    pub cpu_usage_percent: f64,
    /// CPU count (logical cores)
    pub cpu_count: usize,
    /// Total memory in bytes
    pub total_memory_bytes: u64,
    /// Available memory in bytes
    pub available_memory_bytes: u64,
    /// Used memory in bytes
    pub used_memory_bytes: u64,
    /// Memory usage percentage (0-100)
    pub memory_usage_percent: f64,
    /// Disk usage information by mount point
    pub disk_usage: HashMap<String, DiskStats>,
    /// System uptime in seconds
    pub uptime_seconds: Option<u64>,
    /// Load average (1, 5, 15 minutes) on Unix systems
    pub load_average: Option<[f64; 3]>,
    /// Number of running processes
    pub process_count: Option<u32>,
}

/// Disk usage statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiskStats {
    /// Total disk space in bytes
    pub total_bytes: u64,
    /// Available disk space in bytes
    pub available_bytes: u64,
    /// Used disk space in bytes
    pub used_bytes: u64,
    /// Disk usage percentage (0-100)
    pub usage_percent: f64,
    /// Filesystem type
    pub filesystem: Option<String>,
}

/// System monitor configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemMonitorConfig {
    /// Whether to collect CPU usage statistics
    pub collect_cpu_stats: bool,
    /// Whether to collect memory statistics
    pub collect_memory_stats: bool,
    /// Whether to collect disk usage statistics
    pub collect_disk_stats: bool,
    /// Whether to collect process information
    pub collect_process_stats: bool,
    /// Maximum number of disk mounts to report
    pub max_disk_mounts: usize,
    /// CPU sampling duration in milliseconds
    pub cpu_sample_duration_ms: u64,
    /// Whether to include detailed disk information
    pub include_disk_details: bool,
}

impl Default for SystemMonitorConfig {
    fn default() -> Self {
        Self {
            collect_cpu_stats: true,
            collect_memory_stats: true,
            collect_disk_stats: true,
            collect_process_stats: true,
            max_disk_mounts: 20,
            cpu_sample_duration_ms: 1000,
            include_disk_details: true,
        }
    }
}

/// System monitor tool for resource monitoring
#[derive(Clone)]
pub struct SystemMonitorTool {
    metadata: ComponentMetadata,
    config: SystemMonitorConfig,
    #[allow(dead_code)] // Reserved for future sandbox integration
    sandbox_context: Option<Arc<SandboxContext>>,
}

impl SystemMonitorTool {
    /// Create a new system monitor tool
    #[must_use]
    pub fn new(config: SystemMonitorConfig) -> Self {
        Self {
            metadata: ComponentMetadata::new(
                "system_monitor".to_string(),
                "System resource monitoring for CPU, memory, and disk usage tracking".to_string(),
            ),
            config,
            sandbox_context: None,
        }
    }

    /// Create a new system monitor tool with sandbox context
    #[must_use]
    pub fn with_sandbox(config: SystemMonitorConfig, sandbox_context: Arc<SandboxContext>) -> Self {
        Self {
            metadata: ComponentMetadata::new(
                "system_monitor".to_string(),
                "System resource monitoring for CPU, memory, and disk usage tracking".to_string(),
            ),
            config,
            sandbox_context: Some(sandbox_context),
        }
    }

    /// Get basic system information
    #[allow(clippy::unused_async)]
    async fn get_basic_system_info(&self) -> LLMResult<SystemStats> {
        let system_info = get_system_info().map_err(|e| LLMSpellError::Tool {
            message: format!("Failed to get system information: {e}"),
            tool_name: Some("system_monitor".to_string()),
            source: None,
        })?;

        // Calculate memory usage
        let total_memory = system_info.total_memory.unwrap_or(0);
        let available_memory = system_info.available_memory.unwrap_or(0);
        let used_memory = total_memory.saturating_sub(available_memory);
        let memory_usage_percent = if total_memory > 0 {
            (used_memory as f64 / total_memory as f64) * 100.0
        } else {
            0.0
        };

        let stats = SystemStats {
            cpu_usage_percent: 0.0, // Will be updated by CPU monitoring if enabled
            cpu_count: system_info.cpu_cores,
            total_memory_bytes: total_memory,
            available_memory_bytes: available_memory,
            used_memory_bytes: used_memory,
            memory_usage_percent,
            disk_usage: HashMap::new(), // Will be updated by disk monitoring if enabled
            uptime_seconds: None,
            load_average: None,
            process_count: None,
        };

        debug!(
            "Basic system info - CPU cores: {}, Memory: {} / {} bytes",
            stats.cpu_count, stats.used_memory_bytes, stats.total_memory_bytes
        );

        Ok(stats)
    }

    /// Get CPU usage (simplified version without external dependencies)
    #[allow(clippy::unused_async)]
    async fn get_cpu_usage(&self) -> f64 {
        if !self.config.collect_cpu_stats {
            return 0.0;
        }

        // For a basic implementation, we'll simulate CPU usage measurement
        // In a real implementation, this would use platform-specific APIs
        // or a crate like `sysinfo` to get actual CPU usage

        // Simple CPU load approximation based on system load
        #[cfg(unix)]
        {
            if let Ok(load_avg) = self.get_load_average() {
                let cpu_count = get_cpu_count() as f64;
                // Convert load average to approximate CPU percentage
                let cpu_percent = (load_avg[0] / cpu_count * 100.0).min(100.0);
                debug!("Estimated CPU usage: {:.1}%", cpu_percent);
                return cpu_percent;
            }
        }

        // Fallback: return a reasonable default
        0.0
    }

    /// Get system load average (Unix only)
    #[cfg(unix)]
    #[allow(clippy::unused_self)]
    fn get_load_average(&self) -> Result<[f64; 3], std::io::Error> {
        use std::fs;

        let loadavg_content = fs::read_to_string("/proc/loadavg")?;
        let parts: Vec<&str> = loadavg_content.split_whitespace().collect();

        if parts.len() >= 3 {
            let one_minute = parts[0].parse::<f64>().unwrap_or(0.0);
            let five_minutes = parts[1].parse::<f64>().unwrap_or(0.0);
            let fifteen_minutes = parts[2].parse::<f64>().unwrap_or(0.0);
            Ok([one_minute, five_minutes, fifteen_minutes])
        } else {
            Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Invalid loadavg format",
            ))
        }
    }

    /// Get disk usage statistics
    #[allow(clippy::unused_async)]
    async fn get_disk_usage(&self) -> HashMap<String, DiskStats> {
        if !self.config.collect_disk_stats {
            return HashMap::new();
        }

        let mut disk_stats = HashMap::new();

        // Cross-platform disk usage implementation
        #[cfg(unix)]
        {
            if let Ok(mounts) = self.get_unix_mounts() {
                let mut count = 0;
                for (mount_point, fs_type) in mounts {
                    if count >= self.config.max_disk_mounts {
                        break;
                    }

                    if let Ok(stats) = self.get_disk_stats_for_path(&mount_point) {
                        let disk_stat = DiskStats {
                            total_bytes: stats.total_bytes,
                            available_bytes: stats.available_bytes,
                            used_bytes: stats.used_bytes,
                            usage_percent: stats.usage_percent,
                            filesystem: if self.config.include_disk_details {
                                Some(fs_type)
                            } else {
                                None
                            },
                        };
                        disk_stats.insert(mount_point, disk_stat);
                        count += 1;
                    }
                }
            }
        }

        #[cfg(windows)]
        {
            // Windows implementation would use GetDiskFreeSpaceEx
            // For now, we'll provide a minimal implementation
            if let Ok(stats) = self.get_disk_stats_for_path("C:\\") {
                let disk_stat = DiskStats {
                    total_bytes: stats.total_bytes,
                    available_bytes: stats.available_bytes,
                    used_bytes: stats.used_bytes,
                    usage_percent: stats.usage_percent,
                    filesystem: if self.config.include_disk_details {
                        Some("NTFS".to_string())
                    } else {
                        None
                    },
                };
                disk_stats.insert("C:\\".to_string(), disk_stat);
            }
        }

        debug!("Collected disk statistics for {} mounts", disk_stats.len());
        disk_stats
    }

    /// Get Unix mount points
    #[cfg(unix)]
    #[allow(clippy::unused_self)]
    fn get_unix_mounts(&self) -> Result<Vec<(String, String)>, std::io::Error> {
        use std::fs;

        let mounts_content = fs::read_to_string("/proc/mounts")?;
        let mut mounts = Vec::new();

        for line in mounts_content.lines() {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 3 {
                let mount_point = parts[1].to_string();
                let fs_type = parts[2].to_string();

                // Skip virtual filesystems
                if !mount_point.starts_with("/proc")
                    && !mount_point.starts_with("/sys")
                    && !mount_point.starts_with("/dev/")
                    && !fs_type.starts_with("tmpfs")
                    && fs_type != "proc"
                    && fs_type != "sysfs"
                    && fs_type != "devpts"
                {
                    mounts.push((mount_point, fs_type));
                }
            }
        }

        Ok(mounts)
    }

    /// Get disk statistics for a specific path
    #[allow(clippy::unused_self)]
    fn get_disk_stats_for_path(&self, path: &str) -> Result<DiskStats, std::io::Error> {
        #[cfg(unix)]
        {
            use std::ffi::CString;
            use std::mem;

            let path_c = CString::new(path)?;
            // SAFETY: Creating a zeroed libc::statvfs struct is safe as all fields are scalar types
            #[allow(unsafe_code)]
            let mut statvfs: libc::statvfs = unsafe { mem::zeroed() };

            // SAFETY: path_c is a valid C string and statvfs is a valid mutable reference
            #[allow(unsafe_code)]
            let result = unsafe { libc::statvfs(path_c.as_ptr(), &mut statvfs) };

            if result == 0 {
                let block_size = statvfs.f_bsize;
                let total_blocks = u64::from(statvfs.f_blocks);
                let available_blocks = u64::from(statvfs.f_bavail);

                let total_bytes = total_blocks * block_size;
                let available_bytes = available_blocks * block_size;
                let used_bytes = total_bytes.saturating_sub(available_bytes);
                let usage_percent = if total_bytes > 0 {
                    (used_bytes as f64 / total_bytes as f64) * 100.0
                } else {
                    0.0
                };

                Ok(DiskStats {
                    total_bytes,
                    available_bytes,
                    used_bytes,
                    usage_percent,
                    filesystem: None, // Will be set by caller if needed
                })
            } else {
                Err(std::io::Error::last_os_error())
            }
        }

        #[cfg(windows)]
        {
            use std::ffi::OsStr;
            use std::os::windows::ffi::OsStrExt;
            use std::ptr;

            let path_wide: Vec<u16> = OsStr::new(path)
                .encode_wide()
                .chain(std::iter::once(0))
                .collect();

            let mut free_bytes = 0u64;
            let mut total_bytes = 0u64;

            let result = unsafe {
                winapi::um::fileapi::GetDiskFreeSpaceExW(
                    path_wide.as_ptr(),
                    &mut free_bytes,
                    &mut total_bytes,
                    ptr::null_mut(),
                )
            };

            if result != 0 {
                let used_bytes = total_bytes.saturating_sub(free_bytes);
                let usage_percent = if total_bytes > 0 {
                    (used_bytes as f64 / total_bytes as f64) * 100.0
                } else {
                    0.0
                };

                Ok(DiskStats {
                    total_bytes,
                    available_bytes: free_bytes,
                    used_bytes,
                    usage_percent,
                    filesystem: None,
                })
            } else {
                Err(std::io::Error::last_os_error())
            }
        }

        #[cfg(not(any(unix, windows)))]
        {
            // Fallback for unsupported platforms
            Err(std::io::Error::new(
                std::io::ErrorKind::Unsupported,
                "Disk statistics not supported on this platform",
            ))
        }
    }

    /// Get process count (simplified implementation)
    #[allow(clippy::unused_async)]
    async fn get_process_count(&self) -> Option<u32> {
        if !self.config.collect_process_stats {
            return None;
        }

        #[cfg(unix)]
        {
            if let Ok(entries) = std::fs::read_dir("/proc") {
                let count = entries
                    .filter_map(std::result::Result::ok)
                    .filter(|entry| {
                        entry
                            .file_name()
                            .to_string_lossy()
                            .chars()
                            .all(|c| c.is_ascii_digit())
                    })
                    .count() as u32;
                debug!("Process count: {}", count);
                return Some(count);
            }
        }

        #[cfg(windows)]
        {
            // Windows implementation would use process enumeration APIs
            // For now, return None
        }

        None
    }

    /// Get system uptime (simplified implementation)
    #[allow(clippy::unused_async)]
    async fn get_uptime(&self) -> Option<u64> {
        #[cfg(unix)]
        {
            if let Ok(uptime_content) = std::fs::read_to_string("/proc/uptime") {
                if let Some(uptime_str) = uptime_content.split_whitespace().next() {
                    if let Ok(uptime_seconds) = uptime_str.parse::<f64>() {
                        debug!("System uptime: {} seconds", uptime_seconds as u64);
                        return Some(uptime_seconds as u64);
                    }
                }
            }
        }

        None
    }

    /// Collect comprehensive system statistics
    async fn collect_system_stats(&self) -> LLMResult<SystemStats> {
        let start_time = Instant::now();

        // Start with basic system info
        let mut stats = self.get_basic_system_info().await?;

        // Collect CPU usage
        if self.config.collect_cpu_stats {
            stats.cpu_usage_percent = self.get_cpu_usage().await;
        }

        // Collect disk usage
        if self.config.collect_disk_stats {
            stats.disk_usage = self.get_disk_usage().await;
        }

        // Collect process count
        if self.config.collect_process_stats {
            stats.process_count = self.get_process_count().await;
        }

        // Get uptime
        stats.uptime_seconds = self.get_uptime().await;

        // Get load average (Unix only)
        #[cfg(unix)]
        {
            if let Ok(load_avg) = self.get_load_average() {
                stats.load_average = Some(load_avg);
            }
        }

        let collection_time = start_time.elapsed();
        info!(
            "System statistics collected in {}ms - CPU: {:.1}%, Memory: {:.1}%, Disks: {}",
            collection_time.as_millis(),
            stats.cpu_usage_percent,
            stats.memory_usage_percent,
            stats.disk_usage.len()
        );

        Ok(stats)
    }

    /// Validate monitoring parameters
    #[allow(clippy::unused_async)]
    async fn validate_monitoring_parameters(&self, params: &serde_json::Value) -> LLMResult<()> {
        // Validate operation if provided
        if let Some(operation) = extract_optional_string(params, "operation") {
            match operation {
                "stats" | "cpu" | "memory" | "disk" | "all" => {}
                _ => {
                    return Err(LLMSpellError::Validation {
                        message: format!(
                            "Invalid operation: {operation}. Supported operations: stats, cpu, memory, disk, all"
                        ),
                        field: Some("operation".to_string()),
                    });
                }
            }
        }

        Ok(())
    }
}

#[async_trait]
impl BaseAgent for SystemMonitorTool {
    fn metadata(&self) -> &ComponentMetadata {
        &self.metadata
    }

    async fn execute(
        &self,
        input: AgentInput,
        _context: ExecutionContext,
    ) -> LLMResult<AgentOutput> {
        // Get parameters using shared utility
        let params = extract_parameters(&input)?;

        self.validate_monitoring_parameters(params).await?;

        // Extract operation (default to "all")
        let operation = extract_optional_string(params, "operation").unwrap_or("all");

        // Collect system statistics
        let stats = self.collect_system_stats().await?;

        // Format response based on operation
        let response = match operation {
            "cpu" => {
                let message = format!(
                    "CPU usage: {:.1}% ({} cores)",
                    stats.cpu_usage_percent, stats.cpu_count
                );
                ResponseBuilder::success("cpu")
                    .with_message(message)
                    .with_result(json!({
                        "cpu_usage_percent": stats.cpu_usage_percent,
                        "cpu_count": stats.cpu_count,
                        "load_average": stats.load_average
                    }))
                    .build()
            }
            "memory" => {
                let message = format!(
                    "Memory usage: {:.1}% ({} / {} used)",
                    stats.memory_usage_percent,
                    format_bytes(stats.used_memory_bytes),
                    format_bytes(stats.total_memory_bytes)
                );
                ResponseBuilder::success("memory")
                    .with_message(message)
                    .with_result(json!({
                        "total_memory_bytes": stats.total_memory_bytes,
                        "used_memory_bytes": stats.used_memory_bytes,
                        "available_memory_bytes": stats.available_memory_bytes,
                        "memory_usage_percent": stats.memory_usage_percent,
                        "total_memory_formatted": format_bytes(stats.total_memory_bytes),
                        "used_memory_formatted": format_bytes(stats.used_memory_bytes),
                        "available_memory_formatted": format_bytes(stats.available_memory_bytes)
                    }))
                    .build()
            }
            "disk" => {
                let message = format!(
                    "Disk usage for {} mount points collected",
                    stats.disk_usage.len()
                );
                ResponseBuilder::success("disk")
                    .with_message(message)
                    .with_result(json!({
                        "disk_usage": stats.disk_usage
                    }))
                    .build()
            }
            "stats" | "all" => {
                let message = format!(
                    "System stats: CPU {:.1}%, Memory {:.1}% ({}/{}), {} disks, {} processes",
                    stats.cpu_usage_percent,
                    stats.memory_usage_percent,
                    format_bytes(stats.used_memory_bytes),
                    format_bytes(stats.total_memory_bytes),
                    stats.disk_usage.len(),
                    stats.process_count.unwrap_or(0)
                );
                ResponseBuilder::success("all")
                    .with_message(message)
                    .with_result(json!({
                        "cpu_usage_percent": stats.cpu_usage_percent,
                        "cpu_count": stats.cpu_count,
                        "total_memory_bytes": stats.total_memory_bytes,
                        "used_memory_bytes": stats.used_memory_bytes,
                        "available_memory_bytes": stats.available_memory_bytes,
                        "memory_usage_percent": stats.memory_usage_percent,
                        "disk_usage": stats.disk_usage,
                        "uptime_seconds": stats.uptime_seconds,
                        "load_average": stats.load_average,
                        "process_count": stats.process_count,
                        "formatted": {
                            "total_memory": format_bytes(stats.total_memory_bytes),
                            "used_memory": format_bytes(stats.used_memory_bytes),
                            "available_memory": format_bytes(stats.available_memory_bytes)
                        }
                    }))
                    .build()
            }
            _ => unreachable!(), // Already validated above
        };

        Ok(AgentOutput::text(serde_json::to_string_pretty(&response)?))
    }

    async fn validate_input(&self, input: &AgentInput) -> LLMResult<()> {
        if input.text.is_empty() {
            return Err(LLMSpellError::Validation {
                message: "Input prompt cannot be empty".to_string(),
                field: Some("prompt".to_string()),
            });
        }
        Ok(())
    }

    async fn handle_error(&self, error: LLMSpellError) -> LLMResult<AgentOutput> {
        Ok(AgentOutput::text(format!("System monitor error: {error}")))
    }
}

#[async_trait]
impl Tool for SystemMonitorTool {
    fn category(&self) -> ToolCategory {
        ToolCategory::System
    }

    fn security_level(&self) -> SecurityLevel {
        SecurityLevel::Safe // System monitoring is generally safe
    }

    fn schema(&self) -> ToolSchema {
        ToolSchema::new(
            "system_monitor".to_string(),
            "Monitor system resources including CPU, memory, and disk usage".to_string(),
        )
        .with_parameter(ParameterDef {
            name: "operation".to_string(),
            param_type: ParameterType::String,
            description: "Type of monitoring: all, stats, cpu, memory, disk".to_string(),
            required: false,
            default: Some(json!("all")),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use llmspell_testing::tool_helpers::{create_test_tool, create_test_tool_input};

    fn create_test_system_monitor() -> SystemMonitorTool {
        let config = SystemMonitorConfig::default();
        SystemMonitorTool::new(config)
    }

    fn create_test_tool_with_custom_config() -> SystemMonitorTool {
        let config = SystemMonitorConfig {
            max_disk_mounts: 5,
            cpu_sample_duration_ms: 500,
            include_disk_details: false,
            ..Default::default()
        };
        SystemMonitorTool::new(config)
    }
    #[tokio::test]
    async fn test_collect_all_stats() {
        let tool = create_test_system_monitor();

        let input = create_test_tool_input(vec![("operation", "all")]);

        let result = tool
            .execute(input, ExecutionContext::default())
            .await
            .unwrap();
        assert!(result.text.contains("System stats"));
        assert!(result.text.contains("CPU"));
        assert!(result.text.contains("Memory"));
    }
    #[tokio::test]
    async fn test_collect_cpu_stats() {
        let tool = create_test_system_monitor();

        let input = create_test_tool_input(vec![("operation", "cpu")]);

        let result = tool
            .execute(input, ExecutionContext::default())
            .await
            .unwrap();
        assert!(result.text.contains("CPU usage"));
        assert!(result.text.contains("cores"));
    }
    #[tokio::test]
    async fn test_collect_memory_stats() {
        let tool = create_test_system_monitor();

        let input = create_test_tool_input(vec![("operation", "memory")]);

        let result = tool
            .execute(input, ExecutionContext::default())
            .await
            .unwrap();
        assert!(result.text.contains("Memory usage"));
        assert!(result.text.contains("%"));
    }
    #[tokio::test]
    async fn test_collect_disk_stats() {
        let tool = create_test_system_monitor();

        let input = create_test_tool_input(vec![("operation", "disk")]);

        let result = tool
            .execute(input, ExecutionContext::default())
            .await
            .unwrap();
        assert!(result.text.contains("Disk usage"));
        assert!(result.text.contains("mount"));
    }
    #[tokio::test]
    async fn test_invalid_operation() {
        let tool = create_test_system_monitor();

        let input = create_test_tool_input(vec![("operation", "invalid")]);

        let result = tool.execute(input, ExecutionContext::default()).await;
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Invalid operation"));
    }
    #[tokio::test]
    async fn test_default_operation() {
        let tool = create_test_system_monitor();

        // No operation parameter should default to "all"
        let input = create_test_input("Get default statistics", json!({}));

        let result = tool
            .execute(input, ExecutionContext::default())
            .await
            .unwrap();
        assert!(result.text.contains("System stats"));
    }
    #[tokio::test]
    async fn test_basic_system_info() {
        let tool = create_test_system_monitor();

        let stats = tool.get_basic_system_info().await.unwrap();
        assert!(stats.cpu_count > 0);
        assert!(stats.memory_usage_percent >= 0.0);
        assert!(stats.memory_usage_percent <= 100.0);
    }
    #[tokio::test]
    async fn test_cpu_usage_measurement() {
        let tool = create_test_system_monitor();

        let cpu_usage = tool.get_cpu_usage().await;
        assert!(cpu_usage >= 0.0);
        assert!(cpu_usage <= 100.0);
    }
    #[tokio::test]
    async fn test_disk_usage_collection() {
        let tool = create_test_system_monitor();

        let disk_usage = tool.get_disk_usage().await;
        // Disk usage collection might return empty on some test environments
        // so we just verify it doesn't crash
        for (mount_point, stats) in &disk_usage {
            assert!(!mount_point.is_empty());
            assert!(stats.usage_percent >= 0.0);
            assert!(stats.usage_percent <= 100.0);
            assert!(stats.total_bytes >= stats.used_bytes);
        }
    }
    #[tokio::test]
    async fn test_process_count() {
        let tool = create_test_system_monitor();

        let process_count = tool.get_process_count().await;
        // Process count might be None on some platforms
        if let Some(count) = process_count {
            assert!(count > 0);
        }
    }
    #[tokio::test]
    async fn test_tool_metadata() {
        let tool = create_test_system_monitor();

        let metadata = tool.metadata();
        assert_eq!(metadata.name, "system_monitor");
        assert!(
            metadata.description.contains("System resource monitoring")
                || metadata.description.contains("system resource")
        );

        let schema = tool.schema();
        assert_eq!(schema.name, "system_monitor");
        assert_eq!(tool.category(), ToolCategory::System);
        assert_eq!(tool.security_level(), SecurityLevel::Safe);

        // Check optional parameters
        let required_params = schema.required_parameters();
        assert_eq!(required_params.len(), 0); // All parameters are optional
    }
    #[tokio::test]
    async fn test_custom_config() {
        let tool = create_test_tool_with_custom_config();

        // Test that custom configuration is applied
        assert_eq!(tool.config.max_disk_mounts, 5);
        assert_eq!(tool.config.cpu_sample_duration_ms, 500);
        assert!(!tool.config.include_disk_details);
    }
    #[tokio::test]
    async fn test_selective_collection() {
        let config = SystemMonitorConfig {
            collect_cpu_stats: false,
            collect_disk_stats: false,
            collect_process_stats: false,
            ..Default::default()
        };
        let tool = SystemMonitorTool::new(config);

        let stats = tool.collect_system_stats().await.unwrap();

        // CPU usage should be 0 when collection is disabled
        assert_eq!(stats.cpu_usage_percent, 0.0);

        // Disk usage should be empty when collection is disabled
        assert!(stats.disk_usage.is_empty());

        // Process count should be None when collection is disabled
        assert!(stats.process_count.is_none());

        // Memory stats should still be collected
        // total_memory_bytes is u64, so it's always >= 0
    }

    #[cfg(unix)]
    #[tokio::test]
    async fn test_load_average() {
        let tool = create_test_system_monitor();

        // Load average might not be available in all test environments
        if let Ok(load_avg) = tool.get_load_average() {
            assert!(load_avg[0] >= 0.0);
            assert!(load_avg[1] >= 0.0);
            assert!(load_avg[2] >= 0.0);
        }
    }
    #[tokio::test]
    async fn test_uptime() {
        let tool = create_test_system_monitor();

        let uptime = tool.get_uptime().await;
        // Uptime might not be available in all test environments
        if let Some(uptime_secs) = uptime {
            assert!(uptime_secs > 0);
        }
    }
}
