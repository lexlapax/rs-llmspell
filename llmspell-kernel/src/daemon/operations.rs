//! # Signal-Based Operations
//!
//! This module provides signal-triggered operations for the daemon,
//! including configuration reload (SIGUSR1) and state dump (SIGUSR2).

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::sync::RwLock;
use tracing::{debug, error, info, trace, warn};

// KernelConfig is defined in this module below
use crate::state::KernelState;

/// Configuration for signal operations
#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(clippy::struct_excessive_bools)]
pub struct SignalOperationsConfig {
    /// Path to configuration file for reload
    pub config_path: PathBuf,
    /// Path to state dump file
    pub state_dump_path: PathBuf,
    /// Whether to include metrics in state dump
    pub dump_metrics: bool,
    /// Whether to include full state in dump
    pub dump_full_state: bool,
    /// Maximum dump file size in MB
    pub max_dump_size_mb: u64,
    /// Whether config reload is enabled
    pub enable_config_reload: bool,
    /// Whether state dump is enabled
    pub enable_state_dump: bool,
}

impl Default for SignalOperationsConfig {
    fn default() -> Self {
        Self {
            config_path: PathBuf::from("~/.llmspell/kernel.toml"),
            state_dump_path: PathBuf::from("/tmp/llmspell_state_dump.json"),
            dump_metrics: true,
            dump_full_state: false,
            max_dump_size_mb: 100,
            enable_config_reload: true,
            enable_state_dump: true,
        }
    }
}

/// Statistics for signal operations
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SignalOperationsStats {
    /// Number of config reloads
    pub config_reloads: u64,
    /// Number of successful config reloads
    pub successful_reloads: u64,
    /// Number of state dumps
    pub state_dumps: u64,
    /// Number of successful state dumps
    pub successful_dumps: u64,
    /// Last config reload time
    pub last_reload_at: Option<u64>,
    /// Last state dump time
    pub last_dump_at: Option<u64>,
}

/// Signal operations handler
pub struct SignalOperationsHandler {
    /// Configuration
    config: Arc<RwLock<SignalOperationsConfig>>,
    /// Current kernel configuration
    kernel_config: Arc<RwLock<KernelConfig>>,
    /// Kernel state for dumps
    kernel_state: Option<Arc<KernelState>>,
    /// Statistics
    stats: Arc<RwLock<SignalOperationsStats>>,
    /// Track if reload is in progress
    reload_in_progress: Arc<RwLock<bool>>,
    /// Track if dump is in progress
    dump_in_progress: Arc<RwLock<bool>>,
}

impl SignalOperationsHandler {
    /// Create new signal operations handler
    pub fn new(config: SignalOperationsConfig) -> Self {
        Self {
            config: Arc::new(RwLock::new(config)),
            kernel_config: Arc::new(RwLock::new(KernelConfig::default())),
            kernel_state: None,
            stats: Arc::new(RwLock::new(SignalOperationsStats::default())),
            reload_in_progress: Arc::new(RwLock::new(false)),
            dump_in_progress: Arc::new(RwLock::new(false)),
        }
    }

    /// Set kernel state for dumps
    pub fn set_kernel_state(&mut self, state: Arc<KernelState>) {
        self.kernel_state = Some(state);
    }

    /// Handle config reload (SIGUSR1)
    ///
    /// # Errors
    ///
    /// Returns an error if config reload fails
    ///
    /// # Panics
    ///
    /// May panic if the global tracing subscriber cannot be set
    pub async fn handle_config_reload(&self) -> Result<()> {
        // Check if enabled
        let config = self.config.read().await;
        if !config.enable_config_reload {
            info!("Config reload disabled, ignoring SIGUSR1");
            return Ok(());
        }
        let config_path = config.config_path.clone();
        drop(config);

        // Check if already in progress
        {
            let mut in_progress = self.reload_in_progress.write().await;
            if *in_progress {
                warn!("Config reload already in progress, ignoring request");
                return Ok(());
            }
            *in_progress = true;
        }

        // Ensure we clear the flag on exit
        let _guard = ReloadGuard {
            flag: self.reload_in_progress.clone(),
        };

        info!("Starting configuration reload from SIGUSR1");

        // Update stats
        {
            let mut stats = self.stats.write().await;
            stats.config_reloads += 1;
        }

        // Expand home directory if needed
        let config_path = if config_path.starts_with("~") {
            let home = std::env::var("HOME").context("HOME not set")?;
            PathBuf::from(home).join(config_path.strip_prefix("~/").unwrap_or(&config_path))
        } else {
            config_path
        };

        // Read new configuration
        info!("Reading configuration from: {}", config_path.display());
        let config_str = std::fs::read_to_string(&config_path)
            .with_context(|| format!("Failed to read config from {}", config_path.display()))?;

        // Parse configuration
        let new_config: KernelConfig =
            toml::from_str(&config_str).context("Failed to parse configuration")?;

        // Store old config for comparison
        let old_config = self.kernel_config.read().await.clone();

        // Apply non-breaking changes
        let changes = self.apply_config_changes(old_config, new_config).await?;

        // Update stats
        {
            let mut stats = self.stats.write().await;
            stats.successful_reloads += 1;
            stats.last_reload_at = Some(
                SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_secs(),
            );
        }

        info!(
            "Configuration reload complete. {} changes applied",
            changes.len()
        );

        for change in changes {
            info!("Config change: {}", change);
        }

        Ok(())
    }

    /// Apply configuration changes
    async fn apply_config_changes(
        &self,
        old_config: KernelConfig,
        new_config: KernelConfig,
    ) -> Result<Vec<String>> {
        let mut changes = Vec::new();

        // Check for changes in various settings
        if old_config.log_level != new_config.log_level {
            changes.push(format!(
                "Log level: {:?} -> {:?}",
                old_config.log_level, new_config.log_level
            ));
            // Apply log level change
            Self::update_log_level(&new_config.log_level);
        }

        if old_config.max_connections != new_config.max_connections {
            changes.push(format!(
                "Max connections: {} -> {}",
                old_config.max_connections, new_config.max_connections
            ));
        }

        if old_config.timeout_secs != new_config.timeout_secs {
            changes.push(format!(
                "Timeout: {}s -> {}s",
                old_config.timeout_secs, new_config.timeout_secs
            ));
        }

        // Update kernel config
        *self.kernel_config.write().await = new_config;

        Ok(changes)
    }

    /// Update log level dynamically
    fn update_log_level(level: &str) {
        match level.to_lowercase().as_str() {
            "trace" => {
                tracing::subscriber::set_global_default(
                    tracing_subscriber::FmtSubscriber::builder()
                        .with_max_level(tracing::Level::TRACE)
                        .finish(),
                )
                .ok();
                trace!("Log level changed to TRACE");
            }
            "debug" => {
                tracing::subscriber::set_global_default(
                    tracing_subscriber::FmtSubscriber::builder()
                        .with_max_level(tracing::Level::DEBUG)
                        .finish(),
                )
                .ok();
                debug!("Log level changed to DEBUG");
            }
            "info" => {
                tracing::subscriber::set_global_default(
                    tracing_subscriber::FmtSubscriber::builder()
                        .with_max_level(tracing::Level::INFO)
                        .finish(),
                )
                .ok();
                info!("Log level changed to INFO");
            }
            "warn" => {
                tracing::subscriber::set_global_default(
                    tracing_subscriber::FmtSubscriber::builder()
                        .with_max_level(tracing::Level::WARN)
                        .finish(),
                )
                .ok();
                warn!("Log level changed to WARN");
            }
            "error" => {
                tracing::subscriber::set_global_default(
                    tracing_subscriber::FmtSubscriber::builder()
                        .with_max_level(tracing::Level::ERROR)
                        .finish(),
                )
                .ok();
                error!("Log level changed to ERROR");
            }
            _ => {
                warn!("Unknown log level: {}, keeping current level", level);
            }
        }
    }

    /// Handle state dump (SIGUSR2)
    ///
    /// # Errors
    ///
    /// Returns an error if state dump fails
    ///
    /// # Panics
    ///
    /// May panic if file operations fail unexpectedly
    pub async fn handle_state_dump(&self) -> Result<()> {
        // Check if enabled
        let config = self.config.read().await;
        if !config.enable_state_dump {
            info!("State dump disabled, ignoring SIGUSR2");
            return Ok(());
        }
        let state_dump_path = config.state_dump_path.clone();
        let dump_metrics = config.dump_metrics;
        let dump_full_state = config.dump_full_state;
        let max_size_bytes = config.max_dump_size_mb * 1024 * 1024;
        drop(config);

        // Check if already in progress
        {
            let mut in_progress = self.dump_in_progress.write().await;
            if *in_progress {
                warn!("State dump already in progress, ignoring request");
                return Ok(());
            }
            *in_progress = true;
        }

        // Ensure we clear the flag on exit
        let _guard = DumpGuard {
            flag: self.dump_in_progress.clone(),
        };

        info!("Starting state dump from SIGUSR2");

        // Update stats
        {
            let mut stats = self.stats.write().await;
            stats.state_dumps += 1;
        }

        // Collect state information
        let state_info = self
            .collect_state_info(dump_metrics, dump_full_state)
            .await?;

        // Serialize state
        let json =
            serde_json::to_string_pretty(&state_info).context("Failed to serialize state")?;

        // Check size limit
        if json.len() as u64 > max_size_bytes {
            warn!(
                "State dump size ({} bytes) exceeds limit ({} bytes), truncating",
                json.len(),
                max_size_bytes
            );
            // Truncate to limit (this is a simple approach, could be smarter)
            let truncated = &json[..max_size_bytes as usize];
            std::fs::write(&state_dump_path, truncated).with_context(|| {
                format!(
                    "Failed to write state dump to {}",
                    state_dump_path.display()
                )
            })?;
        } else {
            std::fs::write(&state_dump_path, json).with_context(|| {
                format!(
                    "Failed to write state dump to {}",
                    state_dump_path.display()
                )
            })?;
        }

        // Update stats
        {
            let mut stats = self.stats.write().await;
            stats.successful_dumps += 1;
            stats.last_dump_at = Some(
                SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_secs(),
            );
        }

        info!("State dump complete: {}", state_dump_path.display());

        // Also log key metrics to standard log
        if dump_metrics {
            Self::log_key_metrics(&state_info);
        }

        Ok(())
    }

    /// Collect state information for dump
    async fn collect_state_info(
        &self,
        include_metrics: bool,
        include_full_state: bool,
    ) -> Result<serde_json::Value> {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let mut state_info = serde_json::json!({
            "timestamp": timestamp,
            "timestamp_iso": chrono::Utc::now().to_rfc3339(),
            "kernel": {
                "version": env!("CARGO_PKG_VERSION"),
                "name": env!("CARGO_PKG_NAME"),
            }
        });

        // Add configuration
        let config = self.kernel_config.read().await;
        state_info["config"] = serde_json::json!({
            "log_level": config.log_level,
            "max_connections": config.max_connections,
            "timeout_secs": config.timeout_secs,
        });
        drop(config);

        // Add metrics if requested
        if include_metrics {
            let stats = self.stats.read().await;
            state_info["metrics"] = serde_json::json!({
                "config_reloads": stats.config_reloads,
                "successful_reloads": stats.successful_reloads,
                "state_dumps": stats.state_dumps,
                "successful_dumps": stats.successful_dumps,
                "last_reload_at": stats.last_reload_at,
                "last_dump_at": stats.last_dump_at,
            });
        }

        // Add full kernel state if requested and available
        if include_full_state {
            if let Some(ref kernel_state) = self.kernel_state {
                // Get basic state info - we can't serialize the full state directly
                state_info["kernel_state"] = serde_json::json!({
                    "session_id": kernel_state.session_id(),
                    "execution_count": kernel_state.execution_count(),
                    // Add more state fields as needed
                });
            }
        }

        // Add system info
        state_info["system"] = serde_json::json!({
            "uptime_secs": Self::get_uptime_secs(),
            "memory_usage_mb": Self::get_memory_usage_mb(),
            "cpu_usage_percent": Self::get_cpu_usage_percent(),
        });

        Ok(state_info)
    }

    /// Log key metrics to standard log
    fn log_key_metrics(state_info: &serde_json::Value) {
        info!("=== State Dump Metrics ===");

        if let Some(metrics) = state_info.get("metrics") {
            info!("Config reloads: {}", metrics["config_reloads"]);
            info!("State dumps: {}", metrics["state_dumps"]);
        }

        if let Some(system) = state_info.get("system") {
            info!("Uptime: {} seconds", system["uptime_secs"]);
            info!("Memory usage: {} MB", system["memory_usage_mb"]);
            info!("CPU usage: {}%", system["cpu_usage_percent"]);
        }

        if let Some(kernel_state) = state_info.get("kernel_state") {
            info!("Session ID: {}", kernel_state["session_id"]);
            info!("Execution count: {}", kernel_state["execution_count"]);
        }

        info!("=========================");
    }

    /// Get process uptime in seconds
    fn get_uptime_secs() -> u64 {
        // This is a simplified implementation
        // In production, track actual start time
        0
    }

    /// Get memory usage in MB
    fn get_memory_usage_mb() -> u64 {
        // Simplified - in production use actual memory tracking
        0
    }

    /// Get CPU usage percentage
    fn get_cpu_usage_percent() -> f64 {
        // Simplified - in production use actual CPU tracking
        0.0
    }

    /// Get current statistics
    pub async fn get_stats(&self) -> SignalOperationsStats {
        self.stats.read().await.clone()
    }

    /// Check if config reload is in progress
    pub async fn is_reload_in_progress(&self) -> bool {
        *self.reload_in_progress.read().await
    }

    /// Check if state dump is in progress
    pub async fn is_dump_in_progress(&self) -> bool {
        *self.dump_in_progress.read().await
    }
}

/// Guard to ensure reload flag is cleared
struct ReloadGuard {
    flag: Arc<RwLock<bool>>,
}

impl Drop for ReloadGuard {
    fn drop(&mut self) {
        // Use blocking write since we're in Drop
        if let Ok(mut flag) = self.flag.try_write() {
            *flag = false;
        }
    }
}

/// Guard to ensure dump flag is cleared
struct DumpGuard {
    flag: Arc<RwLock<bool>>,
}

impl Drop for DumpGuard {
    fn drop(&mut self) {
        // Use blocking write since we're in Drop
        if let Ok(mut flag) = self.flag.try_write() {
            *flag = false;
        }
    }
}

/// Kernel configuration for reload
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KernelConfig {
    /// Log level
    pub log_level: String,
    /// Maximum connections
    pub max_connections: usize,
    /// Timeout in seconds
    pub timeout_secs: u64,
    // Add more configuration fields as needed
}

impl Default for KernelConfig {
    fn default() -> Self {
        Self {
            log_level: "info".to_string(),
            max_connections: 100,
            timeout_secs: 300,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_signal_operations_handler_creation() {
        let config = SignalOperationsConfig::default();
        let handler = SignalOperationsHandler::new(config);

        assert!(!handler.is_reload_in_progress().await);
        assert!(!handler.is_dump_in_progress().await);
    }

    #[tokio::test]
    async fn test_config_reload_disabled() {
        let config = SignalOperationsConfig {
            enable_config_reload: false,
            ..SignalOperationsConfig::default()
        };

        let handler = SignalOperationsHandler::new(config);
        let result = handler.handle_config_reload().await;

        assert!(result.is_ok());

        let stats = handler.get_stats().await;
        assert_eq!(stats.config_reloads, 0);
    }

    #[tokio::test]
    async fn test_state_dump_disabled() {
        let config = SignalOperationsConfig {
            enable_state_dump: false,
            ..SignalOperationsConfig::default()
        };

        let handler = SignalOperationsHandler::new(config);
        let result = handler.handle_state_dump().await;

        assert!(result.is_ok());

        let stats = handler.get_stats().await;
        assert_eq!(stats.state_dumps, 0);
    }

    #[tokio::test]
    async fn test_state_dump_to_file() {
        let temp_dir = TempDir::new().unwrap();
        let dump_path = temp_dir.path().join("state_dump.json");

        let config = SignalOperationsConfig {
            state_dump_path: dump_path.clone(),
            dump_metrics: true,
            dump_full_state: false,
            ..SignalOperationsConfig::default()
        };

        let handler = SignalOperationsHandler::new(config);
        let result = handler.handle_state_dump().await;

        assert!(result.is_ok());
        assert!(dump_path.exists());

        // Read and verify dump content
        let dump_content = std::fs::read_to_string(&dump_path).unwrap();
        let dump_json: serde_json::Value = serde_json::from_str(&dump_content).unwrap();

        assert!(dump_json["timestamp"].is_number());
        assert!(dump_json["kernel"].is_object());
        assert!(dump_json["metrics"].is_object());

        let stats = handler.get_stats().await;
        assert_eq!(stats.state_dumps, 1);
        assert_eq!(stats.successful_dumps, 1);
        assert!(stats.last_dump_at.is_some());
    }

    #[tokio::test]
    async fn test_config_changes_detection() {
        let handler = SignalOperationsHandler::new(SignalOperationsConfig::default());

        let old_config = KernelConfig {
            log_level: "info".to_string(),
            max_connections: 100,
            timeout_secs: 300,
        };

        let new_config = KernelConfig {
            log_level: "debug".to_string(),
            max_connections: 200,
            timeout_secs: 300,
        };

        let changes = handler
            .apply_config_changes(old_config, new_config)
            .await
            .unwrap();

        assert_eq!(changes.len(), 2);
        assert!(changes[0].contains("Log level"));
        assert!(changes[1].contains("Max connections"));
    }
}
