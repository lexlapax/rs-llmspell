//! ABOUTME: Configuration bridge for runtime config access from scripts
//! ABOUTME: Provides secure, audited configuration access with granular permissions

use llmspell_config::{LLMSpellConfig, ProviderConfig, SecurityConfig, ToolsConfig};
use llmspell_core::error::LLMSpellError;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, RwLock};
use std::time::{SystemTime, UNIX_EPOCH};

/// Granular configuration permissions
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[allow(clippy::struct_excessive_bools)] // We need fine-grained permissions
pub struct ConfigPermissions {
    /// Can read configuration
    pub read: bool,
    /// Can modify provider settings
    pub modify_providers: bool,
    /// Can modify tool settings
    pub modify_tools: bool,
    /// Can modify runtime settings (non-security)
    pub modify_runtime: bool,
    /// Can modify security settings (DANGEROUS!)
    pub modify_security: bool,
    /// Can access sensitive data like API keys
    pub access_secrets: bool,
    /// Allowed provider names for modification
    pub allowed_providers: Option<HashSet<String>>,
    /// Allowed tool names for modification
    pub allowed_tools: Option<HashSet<String>>,
}

impl ConfigPermissions {
    /// Create read-only permissions
    #[must_use]
    pub const fn read_only() -> Self {
        Self {
            read: true,
            modify_providers: false,
            modify_tools: false,
            modify_runtime: false,
            modify_security: false,
            access_secrets: false,
            allowed_providers: None,
            allowed_tools: None,
        }
    }

    /// Create standard modify permissions (no security changes)
    #[must_use]
    pub const fn standard() -> Self {
        Self {
            read: true,
            modify_providers: true,
            modify_tools: true,
            modify_runtime: true,
            modify_security: false,
            access_secrets: false,
            allowed_providers: None,
            allowed_tools: None,
        }
    }

    /// Create full permissions (DANGEROUS!)
    #[must_use]
    pub const fn full() -> Self {
        Self {
            read: true,
            modify_providers: true,
            modify_tools: true,
            modify_runtime: true,
            modify_security: true,
            access_secrets: true,
            allowed_providers: None,
            allowed_tools: None,
        }
    }
}

/// Configuration change audit entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigAuditEntry {
    /// Timestamp of the change
    pub timestamp: u64,
    /// Script or component that made the change
    pub source: String,
    /// Type of change
    pub change_type: ConfigChangeType,
    /// Path to the changed value (e.g., "providers.openai.model")
    pub path: String,
    /// Previous value (if applicable)
    pub old_value: Option<serde_json::Value>,
    /// New value
    pub new_value: Option<serde_json::Value>,
    /// Whether the change was allowed
    pub allowed: bool,
    /// Reason if denied
    pub deny_reason: Option<String>,
}

/// Type of configuration change
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConfigChangeType {
    Read,
    Create,
    Update,
    Delete,
    SecretAccess,
}

/// Immutable configuration settings that cannot be changed at runtime
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImmutableSettings {
    /// Paths that are always immutable
    pub immutable_paths: HashSet<String>,
    /// Security settings that are locked at boot-time (cannot be changed at runtime)
    pub boot_locked_security: HashSet<String>,
    /// Whether to allow provider deletions
    pub lock_provider_deletion: bool,
    /// Maximum memory that can be set
    pub max_memory_limit: Option<usize>,
    /// Minimum timeout that can be set
    pub min_timeout_seconds: Option<u64>,
}

impl Default for ImmutableSettings {
    fn default() -> Self {
        let mut immutable_paths = HashSet::new();
        // Core configuration paths should be immutable by default
        immutable_paths.insert("runtime.max_concurrent_scripts".to_string());

        let mut boot_locked_security = HashSet::new();
        // Critical security settings that cannot be changed after boot
        boot_locked_security.insert("allow_process_spawn".to_string());
        boot_locked_security.insert("allow_network_access".to_string());
        boot_locked_security.insert("allow_file_access".to_string());

        Self {
            immutable_paths,
            boot_locked_security,
            lock_provider_deletion: true,
            max_memory_limit: Some(1024 * 1024 * 1024), // 1GB max
            min_timeout_seconds: Some(1),               // At least 1 second timeout
        }
    }
}

/// Configuration bridge with security and auditing
#[derive(Clone)]
pub struct ConfigBridge {
    /// The base configuration
    base_config: Arc<RwLock<LLMSpellConfig>>,
    /// Script-specific configuration overrides
    script_overrides: Arc<RwLock<HashMap<String, LLMSpellConfig>>>,
    /// Permissions for this bridge
    permissions: ConfigPermissions,
    /// Immutable settings
    immutable: ImmutableSettings,
    /// Audit trail of all config accesses and changes
    audit_trail: Arc<RwLock<Vec<ConfigAuditEntry>>>,
    /// Configuration snapshots for rollback
    snapshots: Arc<RwLock<Vec<(u64, LLMSpellConfig)>>>,
    /// Script context (for sandboxing)
    script_context: Option<String>,
    /// Maximum audit entries to keep
    max_audit_entries: usize,
    /// Maximum snapshots to keep
    max_snapshots: usize,
}

impl ConfigBridge {
    /// Create a new config bridge with specified permissions
    #[must_use]
    pub fn new(config: LLMSpellConfig, permissions: ConfigPermissions) -> Self {
        Self {
            base_config: Arc::new(RwLock::new(config)),
            script_overrides: Arc::new(RwLock::new(HashMap::new())),
            permissions,
            immutable: ImmutableSettings::default(),
            audit_trail: Arc::new(RwLock::new(Vec::new())),
            snapshots: Arc::new(RwLock::new(Vec::new())),
            script_context: None,
            max_audit_entries: 1000,
            max_snapshots: 10,
        }
    }

    /// Create a bridge for a specific script context
    #[must_use]
    pub fn for_script(
        config: LLMSpellConfig,
        script_id: String,
        permissions: ConfigPermissions,
    ) -> Self {
        let mut bridge = Self::new(config, permissions);
        bridge.script_context = Some(script_id);
        bridge
    }

    /// Add an audit entry
    fn audit(
        &self,
        change_type: ConfigChangeType,
        path: String,
        old_value: Option<serde_json::Value>,
        new_value: Option<serde_json::Value>,
        allowed: bool,
        deny_reason: Option<String>,
    ) {
        let entry = ConfigAuditEntry {
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            source: self
                .script_context
                .clone()
                .unwrap_or_else(|| "system".to_string()),
            change_type,
            path,
            old_value,
            new_value,
            allowed,
            deny_reason,
        };

        if let Ok(mut trail) = self.audit_trail.write() {
            trail.push(entry);
            // Trim to max entries
            if trail.len() > self.max_audit_entries {
                let drain_count = trail.len() - self.max_audit_entries;
                trail.drain(0..drain_count);
            }
        }
    }

    /// Check if a path is immutable
    fn is_immutable(&self, path: &str) -> bool {
        self.immutable.immutable_paths.contains(path)
    }

    /// Check if a security setting is boot-locked (cannot be changed at runtime)
    fn is_security_setting_locked(&self, setting_name: &str) -> bool {
        self.immutable.boot_locked_security.contains(setting_name)
    }

    /// Redact sensitive information from a value
    fn redact_secrets(&self, mut value: serde_json::Value) -> serde_json::Value {
        if !self.permissions.access_secrets {
            if let serde_json::Value::Object(ref mut map) = value {
                // Redact API keys and other secrets
                for (key, val) in map.iter_mut() {
                    if key.contains("api_key") || key.contains("secret") || key.contains("password")
                    {
                        *val = serde_json::Value::String("<REDACTED>".to_string());
                    }
                }
            }
        }
        value
    }

    /// Create a snapshot of current configuration
    ///
    /// # Errors
    ///
    /// Returns an error if snapshot write fails
    ///
    /// # Panics
    ///
    /// Panics if system time is before `UNIX_EPOCH` (extremely unlikely)
    pub fn snapshot(&self) -> Result<(), LLMSpellError> {
        let config = self.get_effective_config()?;
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let mut snapshots = self
            .snapshots
            .write()
            .map_err(|e| LLMSpellError::Configuration {
                message: format!("Failed to write snapshots: {e}"),
                source: None,
            })?;

        snapshots.push((timestamp, config));

        // Trim to max snapshots
        if snapshots.len() > self.max_snapshots {
            let drain_count = snapshots.len() - self.max_snapshots;
            snapshots.drain(0..drain_count);
        }
        drop(snapshots);

        Ok(())
    }

    /// Restore from a snapshot
    ///
    /// # Errors
    ///
    /// Returns an error if user lacks `modify_runtime` permission or snapshot not found
    pub fn restore_snapshot(&self, timestamp: u64) -> Result<(), LLMSpellError> {
        if !self.permissions.modify_runtime {
            return Err(LLMSpellError::Configuration {
                message: "Insufficient permissions to restore snapshot".to_string(),
                source: None,
            });
        }

        let snapshots = self
            .snapshots
            .read()
            .map_err(|e| LLMSpellError::Configuration {
                message: format!("Failed to read snapshots: {e}"),
                source: None,
            })?;

        let snapshot = snapshots
            .iter()
            .find(|(ts, _)| *ts == timestamp)
            .ok_or_else(|| LLMSpellError::Configuration {
                message: format!("Snapshot not found: {timestamp}"),
                source: None,
            })?
            .clone();
        drop(snapshots);

        let mut config = self
            .base_config
            .write()
            .map_err(|e| LLMSpellError::Configuration {
                message: format!("Failed to write config: {e}"),
                source: None,
            })?;

        *config = snapshot.1;
        drop(config);

        self.audit(
            ConfigChangeType::Update,
            "<snapshot_restore>".to_string(),
            None,
            Some(serde_json::json!({ "timestamp": timestamp })),
            true,
            None,
        );

        Ok(())
    }

    /// Get effective configuration (base + script overrides)
    fn get_effective_config(&self) -> Result<LLMSpellConfig, LLMSpellError> {
        let base = self
            .base_config
            .read()
            .map_err(|e| LLMSpellError::Configuration {
                message: format!("Failed to read config: {e}"),
                source: None,
            })?;

        // If there's a script context, merge with overrides
        if let Some(ref script_id) = self.script_context {
            let overrides =
                self.script_overrides
                    .read()
                    .map_err(|e| LLMSpellError::Configuration {
                        message: format!("Failed to read overrides: {e}"),
                        source: None,
                    })?;

            if let Some(script_config) = overrides.get(script_id) {
                // TODO: Implement proper config merging
                return Ok(script_config.clone());
            }
        }

        Ok(base.clone())
    }

    /// Get the current configuration (read-only, with secret redaction)
    ///
    /// # Errors
    ///
    /// Returns an error if user lacks read permission
    pub fn get(&self) -> Result<serde_json::Value, LLMSpellError> {
        if !self.permissions.read {
            self.audit(
                ConfigChangeType::Read,
                "<full_config>".to_string(),
                None,
                None,
                false,
                Some("No read permission".to_string()),
            );
            return Err(LLMSpellError::Configuration {
                message: "No read permission for configuration".to_string(),
                source: None,
            });
        }

        let config = self.get_effective_config()?;
        let json = serde_json::to_value(&config).map_err(|e| LLMSpellError::Configuration {
            message: format!("Failed to serialize config: {e}"),
            source: None,
        })?;

        let redacted = self.redact_secrets(json);

        self.audit(
            ConfigChangeType::Read,
            "<full_config>".to_string(),
            None,
            Some(redacted.clone()),
            true,
            None,
        );

        Ok(redacted)
    }

    /// Get the default engine
    ///
    /// # Errors
    ///
    /// Returns an error if user lacks read permission
    pub fn get_default_engine(&self) -> Result<String, LLMSpellError> {
        if !self.permissions.read {
            return Err(LLMSpellError::Configuration {
                message: "No read permission".to_string(),
                source: None,
            });
        }

        let config = self.get_effective_config()?;

        self.audit(
            ConfigChangeType::Read,
            "default_engine".to_string(),
            None,
            Some(serde_json::json!(config.default_engine)),
            true,
            None,
        );

        Ok(config.default_engine)
    }

    /// Get provider configuration by name (with secret redaction)
    ///
    /// # Errors
    ///
    /// Returns an error if user lacks read permission
    pub fn get_provider(&self, name: &str) -> Result<Option<serde_json::Value>, LLMSpellError> {
        if !self.permissions.read {
            return Err(LLMSpellError::Configuration {
                message: "No read permission".to_string(),
                source: None,
            });
        }

        let config = self.get_effective_config()?;
        let provider = config.providers.configs.get(name).cloned();

        let result = provider.map(|p| {
            let json = serde_json::to_value(&p).unwrap_or(serde_json::Value::Null);
            self.redact_secrets(json)
        });

        self.audit(
            ConfigChangeType::Read,
            format!("providers.{name}"),
            None,
            result.clone(),
            true,
            None,
        );

        Ok(result)
    }

    /// List all configured providers
    ///
    /// # Errors
    ///
    /// Returns an error if user lacks read permission
    pub fn list_providers(&self) -> Result<Vec<String>, LLMSpellError> {
        if !self.permissions.read {
            return Err(LLMSpellError::Configuration {
                message: "No read permission".to_string(),
                source: None,
            });
        }

        let config = self.get_effective_config()?;
        let providers: Vec<String> = config.providers.configs.keys().cloned().collect();

        self.audit(
            ConfigChangeType::Read,
            "providers.<list>".to_string(),
            None,
            Some(serde_json::json!(providers)),
            true,
            None,
        );

        Ok(providers)
    }

    /// Get security configuration (requires special permission)
    ///
    /// # Errors
    ///
    /// Returns an error if user lacks read permission
    pub fn get_security(&self) -> Result<SecurityConfig, LLMSpellError> {
        if !self.permissions.read {
            return Err(LLMSpellError::Configuration {
                message: "No read permission".to_string(),
                source: None,
            });
        }

        let config = self.get_effective_config()?;

        self.audit(
            ConfigChangeType::Read,
            "runtime.security".to_string(),
            None,
            Some(serde_json::to_value(&config.runtime.security).unwrap_or(serde_json::Value::Null)),
            true,
            None,
        );

        Ok(config.runtime.security)
    }

    /// Get tools configuration
    ///
    /// # Errors
    ///
    /// Returns an error if user lacks read permission
    pub fn get_tools_config(&self) -> Result<ToolsConfig, LLMSpellError> {
        if !self.permissions.read {
            return Err(LLMSpellError::Configuration {
                message: "No read permission".to_string(),
                source: None,
            });
        }

        let config = self.get_effective_config()?;

        self.audit(
            ConfigChangeType::Read,
            "tools".to_string(),
            None,
            Some(serde_json::to_value(&config.tools).unwrap_or(serde_json::Value::Null)),
            true,
            None,
        );

        Ok(config.tools)
    }

    /// Check if file access is allowed
    ///
    /// # Errors
    ///
    /// Returns an error if user lacks read permission
    pub fn is_file_access_allowed(&self) -> Result<bool, LLMSpellError> {
        if !self.permissions.read {
            return Err(LLMSpellError::Configuration {
                message: "No read permission".to_string(),
                source: None,
            });
        }

        let config = self.get_effective_config()?;
        Ok(config.runtime.security.allow_file_access)
    }

    /// Check if network access is allowed
    ///
    /// # Errors
    ///
    /// Returns an error if user lacks read permission
    pub fn is_network_access_allowed(&self) -> Result<bool, LLMSpellError> {
        if !self.permissions.read {
            return Err(LLMSpellError::Configuration {
                message: "No read permission".to_string(),
                source: None,
            });
        }

        let config = self.get_effective_config()?;
        Ok(config.runtime.security.allow_network_access)
    }

    /// Set a provider configuration (requires `modify_providers` permission)
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - User lacks `modify_providers` permission
    /// - Provider is not in allowed list
    /// - Provider path is immutable
    /// - Configuration write fails
    pub fn set_provider(&self, name: &str, provider: &ProviderConfig) -> Result<(), LLMSpellError> {
        if !self.permissions.modify_providers {
            self.audit(
                ConfigChangeType::Update,
                format!("providers.{name}"),
                None,
                Some(serde_json::to_value(provider).unwrap_or(serde_json::Value::Null)),
                false,
                Some("No modify_providers permission".to_string()),
            );
            return Err(LLMSpellError::Configuration {
                message: "Cannot modify providers without permission".to_string(),
                source: None,
            });
        }

        // Check if provider is in allowed list
        if let Some(ref allowed) = self.permissions.allowed_providers {
            if !allowed.contains(name) {
                self.audit(
                    ConfigChangeType::Update,
                    format!("providers.{name}"),
                    None,
                    None,
                    false,
                    Some(format!("Provider '{name}' not in allowed list")),
                );
                return Err(LLMSpellError::Configuration {
                    message: format!("Not allowed to modify provider: {name}"),
                    source: None,
                });
            }
        }

        let path = format!("providers.{name}");
        if self.is_immutable(&path) {
            return Err(LLMSpellError::Configuration {
                message: format!("Provider {name} is immutable"),
                source: None,
            });
        }

        let mut config = self
            .base_config
            .write()
            .map_err(|e| LLMSpellError::Configuration {
                message: format!("Failed to write configuration: {e}"),
                source: None,
            })?;

        let old_value = config
            .providers
            .configs
            .get(name)
            .map(|p| serde_json::to_value(p).unwrap_or(serde_json::Value::Null));

        config
            .providers
            .configs
            .insert(name.to_string(), provider.clone());
        drop(config);

        self.audit(
            ConfigChangeType::Update,
            path,
            old_value,
            Some(serde_json::to_value(provider).unwrap_or(serde_json::Value::Null)),
            true,
            None,
        );

        Ok(())
    }

    /// Update file operation allowed paths (requires `modify_tools` permission)
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - User lacks `modify_tools` permission
    /// - Path is immutable
    /// - Configuration write fails
    pub fn add_allowed_path(&self, path: &str) -> Result<(), LLMSpellError> {
        if !self.permissions.modify_tools {
            self.audit(
                ConfigChangeType::Update,
                "tools.file_operations.allowed_paths".to_string(),
                None,
                Some(serde_json::json!(path)),
                false,
                Some("No modify_tools permission".to_string()),
            );
            return Err(LLMSpellError::Configuration {
                message: "Cannot modify tools without permission".to_string(),
                source: None,
            });
        }

        if self.is_immutable("tools.file_operations.allowed_paths") {
            return Err(LLMSpellError::Configuration {
                message: "File operation paths are immutable".to_string(),
                source: None,
            });
        }

        let mut config = self
            .base_config
            .write()
            .map_err(|e| LLMSpellError::Configuration {
                message: format!("Failed to write configuration: {e}"),
                source: None,
            })?;

        let old_value = Some(
            serde_json::to_value(&config.tools.file_operations.allowed_paths)
                .unwrap_or(serde_json::Value::Null),
        );

        // Add the new path
        config
            .tools
            .file_operations
            .allowed_paths
            .push(path.to_string());

        let new_value = Some(
            serde_json::to_value(&config.tools.file_operations.allowed_paths)
                .unwrap_or(serde_json::Value::Null),
        );
        drop(config);

        self.audit(
            ConfigChangeType::Update,
            "tools.file_operations.allowed_paths".to_string(),
            old_value,
            new_value,
            true,
            None,
        );

        Ok(())
    }

    /// Set security settings (requires `modify_security` permission - DANGEROUS!)
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - User lacks `modify_security` permission
    /// - Specific security settings are boot-locked
    /// - Memory limit exceeds maximum allowed
    /// - Configuration write fails
    pub fn set_security(&self, security: &SecurityConfig) -> Result<(), LLMSpellError> {
        if !self.permissions.modify_security {
            self.audit(
                ConfigChangeType::Update,
                "runtime.security".to_string(),
                None,
                Some(serde_json::to_value(security).unwrap_or(serde_json::Value::Null)),
                false,
                Some("No modify_security permission".to_string()),
            );
            return Err(LLMSpellError::Configuration {
                message: "Cannot modify security settings without permission".to_string(),
                source: None,
            });
        }

        // Get current security config to check what's changing
        let current_config = self.get_effective_config()?;
        let current_security = &current_config.runtime.security;

        // Check for changes to boot-locked security settings
        if current_security.allow_process_spawn != security.allow_process_spawn
            && self.is_security_setting_locked("allow_process_spawn")
        {
            return Err(LLMSpellError::Configuration {
                message: "Process spawn permission is boot-locked and cannot be changed"
                    .to_string(),
                source: None,
            });
        }

        if current_security.allow_network_access != security.allow_network_access
            && self.is_security_setting_locked("allow_network_access")
        {
            return Err(LLMSpellError::Configuration {
                message: "Network access permission is boot-locked and cannot be changed"
                    .to_string(),
                source: None,
            });
        }

        if current_security.allow_file_access != security.allow_file_access
            && self.is_security_setting_locked("allow_file_access")
        {
            return Err(LLMSpellError::Configuration {
                message: "File access permission is boot-locked and cannot be changed".to_string(),
                source: None,
            });
        }

        // Validate security settings against immutable limits
        if let Some(max_mem) = self.immutable.max_memory_limit {
            if let Some(mem) = security.max_memory_bytes {
                if mem > max_mem {
                    return Err(LLMSpellError::Configuration {
                        message: format!("Memory limit {mem} exceeds maximum {max_mem}"),
                        source: None,
                    });
                }
            }
        }

        let mut config = self
            .base_config
            .write()
            .map_err(|e| LLMSpellError::Configuration {
                message: format!("Failed to write configuration: {e}"),
                source: None,
            })?;

        let old_value =
            Some(serde_json::to_value(&config.runtime.security).unwrap_or(serde_json::Value::Null));
        config.runtime.security = security.clone();
        drop(config);

        self.audit(
            ConfigChangeType::Update,
            "runtime.security".to_string(),
            old_value,
            Some(serde_json::to_value(security).unwrap_or(serde_json::Value::Null)),
            true,
            None,
        );

        Ok(())
    }

    /// Get current permissions
    #[must_use]
    pub const fn permissions(&self) -> &ConfigPermissions {
        &self.permissions
    }

    /// Export configuration as JSON (for script access)
    ///
    /// # Errors
    ///
    /// Returns an error if serialization fails
    pub fn to_json(&self) -> Result<serde_json::Value, LLMSpellError> {
        let config = self.get()?;
        serde_json::to_value(&config).map_err(|e| LLMSpellError::Configuration {
            message: format!("Failed to serialize configuration: {e}"),
            source: None,
        })
    }

    /// Export specific section as JSON
    ///
    /// # Errors
    ///
    /// Returns an error if section is unknown or serialization fails
    pub fn section_to_json(&self, section: &str) -> Result<serde_json::Value, LLMSpellError> {
        let config = self.get_effective_config()?;
        let value = match section {
            "providers" => serde_json::to_value(&config.providers),
            "engines" => serde_json::to_value(&config.engines),
            "runtime" => serde_json::to_value(&config.runtime),
            "tools" => serde_json::to_value(&config.tools),
            "security" => serde_json::to_value(&config.runtime.security),
            _ => {
                return Err(LLMSpellError::Configuration {
                    message: format!("Unknown configuration section: {section}"),
                    source: None,
                })
            }
        };

        value.map_err(|e| LLMSpellError::Configuration {
            message: format!("Failed to serialize section {section}: {e}"),
            source: None,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use llmspell_config::providers::ProviderConfig;

    #[test]
    fn test_config_bridge_read_only() {
        let config = LLMSpellConfig::default();
        let bridge = ConfigBridge::new(config, ConfigPermissions::read_only());

        // Reading should work
        assert!(bridge.get().is_ok());
        assert!(bridge.get_default_engine().is_ok());
        assert!(bridge.list_providers().is_ok());

        // Writing should fail
        let provider = ProviderConfig {
            name: "test".to_string(),
            provider_type: "test".to_string(),
            enabled: true,
            api_key_env: None,
            api_key: None,
            base_url: None,
            default_model: None,
            max_tokens: None,
            timeout_seconds: Some(60),
            max_retries: None,
            rate_limit: None,
            retry: None,
            options: HashMap::default(),
        };
        assert!(bridge.set_provider("test", &provider).is_err());
        assert!(bridge.add_allowed_path("/test").is_err());
    }

    #[test]
    fn test_config_bridge_modify() {
        let config = LLMSpellConfig::default();
        let bridge = ConfigBridge::new(config, ConfigPermissions::standard());

        // Should be able to modify providers
        let provider = ProviderConfig {
            name: "test".to_string(),
            provider_type: "test".to_string(),
            enabled: true,
            api_key_env: None,
            api_key: None,
            base_url: None,
            default_model: None,
            max_tokens: None,
            timeout_seconds: Some(60),
            max_retries: None,
            rate_limit: None,
            retry: None,
            options: HashMap::default(),
        };
        assert!(bridge.set_provider("test", &provider).is_ok());

        // Should not be able to modify security
        let security = SecurityConfig::default();
        assert!(bridge.set_security(&security).is_err());
    }

    #[test]
    fn test_config_bridge_full() {
        let config = LLMSpellConfig::default();
        let mut bridge = ConfigBridge::new(config, ConfigPermissions::full());

        // Unlock specific security settings for this test
        bridge.immutable.boot_locked_security.clear();

        // Should be able to modify security settings (when not boot-locked)
        let security = SecurityConfig::default();
        match bridge.set_security(&security) {
            Ok(()) => {}
            Err(e) => panic!("set_security failed: {e}"),
        }
    }

    #[test]
    fn test_config_bridge_boot_locked_security() {
        let config = LLMSpellConfig::default();
        let bridge = ConfigBridge::new(config, ConfigPermissions::full());

        // Try to modify boot-locked security settings - should fail
        let mut security = SecurityConfig::default();
        security.allow_process_spawn = !security.allow_process_spawn; // Try to flip this setting

        match bridge.set_security(&security) {
            Ok(()) => panic!("Expected boot-locked security to prevent changes"),
            Err(e) => assert!(e.to_string().contains("boot-locked")),
        }
    }

    #[test]
    fn test_config_export_json() {
        let config = LLMSpellConfig::default();
        let bridge = ConfigBridge::new(config, ConfigPermissions::read_only());

        // Should be able to export to JSON
        let json = bridge.to_json().unwrap();
        assert!(json.is_object());

        // Should be able to export sections
        let providers_json = bridge.section_to_json("providers").unwrap();
        assert!(providers_json.is_object());
    }
}
