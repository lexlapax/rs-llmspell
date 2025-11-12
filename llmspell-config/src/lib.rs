//! ABOUTME: Central configuration management system for llmspell
//! ABOUTME: Handles TOML parsing, validation, and environment variable overrides

use anyhow::{Context, Result};
use llmspell_core::error::LLMSpellError;
use serde::{Deserialize, Serialize};
use std::env as std_env;
use std::path::{Path, PathBuf};
use tokio::fs;
use tracing::debug;

// Re-export engine configurations from bridge
pub use crate::debug::DebugConfig;
pub use crate::engines::{EngineConfigs, JSConfig, LuaConfig};
pub use crate::env::{EnvCategory, EnvRegistry, EnvVarDef, EnvVarDefBuilder, IsolationMode};
pub use crate::memory::{ConsolidationConfig, DaemonConfig, MemoryConfig};
pub use crate::providers::{ProviderConfig, ProviderManagerConfig, ProviderManagerConfigBuilder};
pub use crate::rag::{
    ChunkingConfig, ChunkingStrategy, DistanceMetric, EmbeddingConfig, HNSWConfig, RAGCacheConfig,
    RAGConfig, RAGConfigBuilder, VectorBackend, VectorStorageConfig,
};
pub use crate::tools::{FileOperationsConfig, ToolsConfig};

pub mod debug;
pub mod engines;
pub mod env;
pub mod env_registry;
pub mod memory;
pub mod providers;
pub mod rag;
pub mod tools;
pub mod validation;

use crate::env_registry::register_standard_vars;

/// Configuration file discovery order
const CONFIG_SEARCH_PATHS: &[&str] = &[
    "llmspell.toml",
    ".llmspell.toml",
    "config/llmspell.toml",
    ".config/llmspell.toml",
];

/// Environment variable prefix (kept for documentation)
#[allow(dead_code)]
const ENV_PREFIX: &str = "LLMSPELL_";

/// Metadata describing a builtin configuration profile
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProfileMetadata {
    /// Profile name (e.g., "minimal", "rag-dev")
    pub name: &'static str,
    /// Category (e.g., "Core", "RAG", "Local LLM")
    pub category: &'static str,
    /// Short description
    pub description: &'static str,
    /// Common use cases
    pub use_cases: Vec<&'static str>,
    /// Key features
    pub features: Vec<&'static str>,
}

/// Central LLMSpell configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(default)]
pub struct LLMSpellConfig {
    /// Default script engine to use
    pub default_engine: String,
    /// Engine-specific configurations  
    pub engines: EngineConfigs,
    /// Provider configurations
    pub providers: ProviderManagerConfig,
    /// Global runtime settings
    pub runtime: GlobalRuntimeConfig,
    /// Tool-specific configurations
    pub tools: ToolsConfig,
    /// Hook system configuration
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hooks: Option<HookConfig>,
    /// Event system configuration
    pub events: EventsConfig,
    /// Debug system configuration
    pub debug: DebugConfig,
    /// RAG (Retrieval-Augmented Generation) configuration
    pub rag: RAGConfig,
}

impl Default for LLMSpellConfig {
    fn default() -> Self {
        Self {
            default_engine: "lua".to_string(),
            engines: EngineConfigs::default(),
            providers: ProviderManagerConfig::default(),
            runtime: GlobalRuntimeConfig::default(),
            tools: ToolsConfig::default(),
            hooks: None,
            events: EventsConfig::default(),
            debug: DebugConfig::default(),
            rag: RAGConfig::default(),
        }
    }
}

impl LLMSpellConfig {
    /// Create a new builder for `LLMSpellConfig`
    #[must_use]
    pub fn builder() -> LLMSpellConfigBuilder {
        LLMSpellConfigBuilder::new()
    }

    /// Load configuration from TOML file with validation
    pub async fn load_from_file(path: &Path) -> Result<Self, ConfigError> {
        let content = fs::read_to_string(path)
            .await
            .with_context(|| format!("Failed to read config file: {}", path.display()))?;

        Self::from_toml(&content)
    }

    /// Parse TOML content with environment variable overrides and validation
    pub fn from_toml(content: &str) -> Result<Self, ConfigError> {
        let mut config: LLMSpellConfig = toml::from_str(content)?;

        // Use registry for environment overrides
        config.apply_env_registry()?;
        config.validate()?;

        Ok(config)
    }

    /// Apply environment variable overrides using the centralized registry
    pub fn apply_env_registry(&mut self) -> Result<(), ConfigError> {
        // Create registry and load standard variables
        let registry = EnvRegistry::new();
        register_standard_vars(&registry).map_err(|e| ConfigError::Environment { message: e })?;

        // Load environment variables
        registry
            .load_from_env()
            .map_err(|e| ConfigError::Environment { message: e })?;

        // Build config from registry
        let env_config = registry
            .build_config()
            .map_err(|e| ConfigError::Environment { message: e })?;

        // Merge environment config into self
        self.merge_from_json_impl(&env_config)?;

        Ok(())
    }

    /// Merge values from JSON config (from registry) - exposed for testing
    pub fn merge_from_json(&mut self, json: &serde_json::Value) -> Result<(), ConfigError> {
        self.merge_from_json_impl(json)
    }

    /// Internal implementation of merge_from_json
    fn merge_from_json_impl(&mut self, json: &serde_json::Value) -> Result<(), ConfigError> {
        // Merge top-level values
        if let Some(engine) = json.get("default_engine").and_then(|v| v.as_str()) {
            debug!("Overriding default engine from env: {}", engine);
            self.default_engine = engine.to_string();
        }

        // Merge runtime values
        if let Some(runtime) = json.get("runtime").and_then(|v| v.as_object()) {
            if let Some(max_scripts) = runtime
                .get("max_concurrent_scripts")
                .and_then(|v| v.as_u64())
            {
                debug!(
                    "Overriding max_concurrent_scripts from env: {}",
                    max_scripts
                );
                self.runtime.max_concurrent_scripts = max_scripts as usize;
            }

            if let Some(timeout) = runtime
                .get("script_timeout_seconds")
                .and_then(|v| v.as_u64())
            {
                debug!("Overriding script_timeout_seconds from env: {}", timeout);
                self.runtime.script_timeout_seconds = timeout;
            }

            // Merge security settings
            if let Some(security) = runtime.get("security").and_then(|v| v.as_object()) {
                if let Some(allow_file) =
                    security.get("allow_file_access").and_then(|v| v.as_bool())
                {
                    debug!("Overriding allow_file_access from env: {}", allow_file);
                    self.runtime.security.allow_file_access = allow_file;
                }

                if let Some(allow_network) = security
                    .get("allow_network_access")
                    .and_then(|v| v.as_bool())
                {
                    debug!(
                        "Overriding allow_network_access from env: {}",
                        allow_network
                    );
                    self.runtime.security.allow_network_access = allow_network;
                }

                if let Some(allow_spawn) = security
                    .get("allow_process_spawn")
                    .and_then(|v| v.as_bool())
                {
                    self.runtime.security.allow_process_spawn = allow_spawn;
                }

                if let Some(max_memory) = security.get("max_memory_bytes").and_then(|v| v.as_u64())
                {
                    self.runtime.security.max_memory_bytes = Some(max_memory as usize);
                }

                if let Some(max_time) = security
                    .get("max_execution_time_ms")
                    .and_then(|v| v.as_u64())
                {
                    self.runtime.security.max_execution_time_ms = Some(max_time);
                }
            }

            // Merge state persistence settings
            if let Some(state) = runtime.get("state_persistence").and_then(|v| v.as_object()) {
                if let Some(backend) = state.get("backend_type").and_then(|v| v.as_str()) {
                    self.runtime.state_persistence.backend_type = backend.to_string();
                }

                // Flattened structure - direct access
                if let Some(enabled) = state.get("enabled").and_then(|v| v.as_bool()) {
                    self.runtime.state_persistence.enabled = enabled;
                }
                if let Some(migration) = state.get("migration_enabled").and_then(|v| v.as_bool()) {
                    self.runtime.state_persistence.migration_enabled = migration;
                }
                if let Some(backup_on_migration) =
                    state.get("backup_on_migration").and_then(|v| v.as_bool())
                {
                    self.runtime.state_persistence.backup_on_migration = backup_on_migration;
                }
                if let Some(backup_enabled) = state.get("backup_enabled").and_then(|v| v.as_bool())
                {
                    self.runtime.state_persistence.backup_enabled = backup_enabled;
                }
            }

            // Merge session settings
            if let Some(sessions) = runtime.get("sessions").and_then(|v| v.as_object()) {
                if let Some(enabled) = sessions.get("enabled").and_then(|v| v.as_bool()) {
                    self.runtime.sessions.enabled = enabled;
                }
                if let Some(backend) = sessions.get("storage_backend").and_then(|v| v.as_str()) {
                    self.runtime.sessions.storage_backend = backend.to_string();
                }
            }

            // Merge memory settings
            if let Some(memory) = runtime.get("memory").and_then(|v| v.as_object()) {
                if let Some(enabled) = memory.get("enabled").and_then(|v| v.as_bool()) {
                    self.runtime.memory.enabled = enabled;
                }

                // Merge consolidation settings
                if let Some(consolidation) = memory.get("consolidation").and_then(|v| v.as_object())
                {
                    if let Some(provider) =
                        consolidation.get("provider_name").and_then(|v| v.as_str())
                    {
                        self.runtime.memory.consolidation.provider_name =
                            Some(provider.to_string());
                    }
                    if let Some(batch_size) =
                        consolidation.get("batch_size").and_then(|v| v.as_u64())
                    {
                        self.runtime.memory.consolidation.batch_size = batch_size as usize;
                    }
                    if let Some(max_concurrent) =
                        consolidation.get("max_concurrent").and_then(|v| v.as_u64())
                    {
                        self.runtime.memory.consolidation.max_concurrent = max_concurrent as usize;
                    }
                    if let Some(threshold) = consolidation
                        .get("active_session_threshold_secs")
                        .and_then(|v| v.as_u64())
                    {
                        self.runtime
                            .memory
                            .consolidation
                            .active_session_threshold_secs = threshold;
                    }
                }

                // Merge daemon settings
                if let Some(daemon) = memory.get("daemon").and_then(|v| v.as_object()) {
                    if let Some(enabled) = daemon.get("enabled").and_then(|v| v.as_bool()) {
                        self.runtime.memory.daemon.enabled = enabled;
                    }
                    if let Some(fast) = daemon.get("fast_interval_secs").and_then(|v| v.as_u64()) {
                        self.runtime.memory.daemon.fast_interval_secs = fast;
                    }
                    if let Some(normal) =
                        daemon.get("normal_interval_secs").and_then(|v| v.as_u64())
                    {
                        self.runtime.memory.daemon.normal_interval_secs = normal;
                    }
                    if let Some(slow) = daemon.get("slow_interval_secs").and_then(|v| v.as_u64()) {
                        self.runtime.memory.daemon.slow_interval_secs = slow;
                    }
                    if let Some(threshold_fast) =
                        daemon.get("queue_threshold_fast").and_then(|v| v.as_u64())
                    {
                        self.runtime.memory.daemon.queue_threshold_fast = threshold_fast as usize;
                    }
                    if let Some(threshold_slow) =
                        daemon.get("queue_threshold_slow").and_then(|v| v.as_u64())
                    {
                        self.runtime.memory.daemon.queue_threshold_slow = threshold_slow as usize;
                    }
                    if let Some(shutdown) = daemon
                        .get("shutdown_max_wait_secs")
                        .and_then(|v| v.as_u64())
                    {
                        self.runtime.memory.daemon.shutdown_max_wait_secs = shutdown;
                    }
                    if let Some(health) = daemon
                        .get("health_check_interval_secs")
                        .and_then(|v| v.as_u64())
                    {
                        self.runtime.memory.daemon.health_check_interval_secs = health;
                    }
                }
            }
        }

        // Merge provider configurations
        if let Some(providers) = json.get("providers").and_then(|v| v.as_object()) {
            // Handle flattened structure - provider configs are direct children
            for (name, config) in providers {
                // Skip default_provider field, only process provider configs
                if name == "default_provider" {
                    continue;
                }

                if let Some(provider_obj) = config.as_object() {
                    // Check if provider already exists in config
                    if let Some(existing_provider) = self.providers.providers.get_mut(name) {
                        // Provider exists - only update fields that are present in env config
                        if let Some(api_key) = provider_obj.get("api_key").and_then(|v| v.as_str())
                        {
                            existing_provider.api_key = Some(api_key.to_string());
                        }
                        if let Some(base_url) =
                            provider_obj.get("base_url").and_then(|v| v.as_str())
                        {
                            existing_provider.base_url = Some(base_url.to_string());
                        }
                        if let Some(model) =
                            provider_obj.get("default_model").and_then(|v| v.as_str())
                        {
                            existing_provider.default_model = Some(model.to_string());
                        }
                        if let Some(timeout) =
                            provider_obj.get("timeout_seconds").and_then(|v| v.as_u64())
                        {
                            existing_provider.timeout_seconds = Some(timeout);
                        }
                        if let Some(max_retries) =
                            provider_obj.get("max_retries").and_then(|v| v.as_u64())
                        {
                            existing_provider.max_retries = Some(max_retries as u32);
                        }
                        // Only update other fields if they're present
                        if let Some(provider_type) =
                            provider_obj.get("provider_type").and_then(|v| v.as_str())
                        {
                            existing_provider.provider_type = provider_type.to_string();
                        }
                        if let Some(enabled) = provider_obj.get("enabled").and_then(|v| v.as_bool())
                        {
                            existing_provider.enabled = enabled;
                        }
                        // Do NOT insert/replace - we already modified in place
                    } else {
                        // Provider doesn't exist in config - only create if it has minimum required fields
                        // Don't create incomplete providers from just API keys
                        if provider_obj.contains_key("default_model")
                            || provider_obj.contains_key("provider_type")
                        {
                            let mut provider_config = ProviderConfig {
                                name: name.clone(),
                                provider_type: provider_obj
                                    .get("provider_type")
                                    .and_then(|v| v.as_str())
                                    .unwrap_or(name)
                                    .to_string(),
                                enabled: true,
                                ..Default::default()
                            };

                            if let Some(api_key) =
                                provider_obj.get("api_key").and_then(|v| v.as_str())
                            {
                                provider_config.api_key = Some(api_key.to_string());
                            }
                            if let Some(base_url) =
                                provider_obj.get("base_url").and_then(|v| v.as_str())
                            {
                                provider_config.base_url = Some(base_url.to_string());
                            }
                            if let Some(model) =
                                provider_obj.get("default_model").and_then(|v| v.as_str())
                            {
                                provider_config.default_model = Some(model.to_string());
                            }

                            self.providers
                                .providers
                                .insert(name.clone(), provider_config);
                        }
                        // Otherwise skip - don't create incomplete provider from just api_key
                    }
                }
            }
        }

        // Merge tool configurations
        if let Some(tools) = json.get("tools").and_then(|v| v.as_object()) {
            if let Some(file_ops) = tools.get("file_operations").and_then(|v| v.as_object()) {
                if let Some(enabled) = file_ops.get("enabled").and_then(|v| v.as_bool()) {
                    self.tools.file_operations.enabled = enabled;
                }
                if let Some(max_size) = file_ops.get("max_file_size").and_then(|v| v.as_u64()) {
                    self.tools.file_operations.max_file_size = max_size as usize;
                }
                // Handle allowed_paths - can be either string (from env) or array (from JSON)
                if let Some(paths_value) = file_ops.get("allowed_paths") {
                    if let Some(paths_str) = paths_value.as_str() {
                        // From environment variable - comma-separated string
                        self.tools.file_operations.allowed_paths =
                            paths_str.split(',').map(|s| s.trim().to_string()).collect();
                    } else if let Some(paths_array) = paths_value.as_array() {
                        // From JSON - array of strings
                        self.tools.file_operations.allowed_paths = paths_array
                            .iter()
                            .filter_map(|v| v.as_str().map(String::from))
                            .collect();
                    }
                }
            }
            if let Some(network) = tools.get("network").and_then(|v| v.as_object()) {
                let net_config = self.tools.network.get_or_insert(Default::default());
                if let Some(timeout) = network.get("timeout_seconds").and_then(|v| v.as_u64()) {
                    net_config.timeout_seconds = timeout;
                }
            }
            if let Some(rate_limit) = tools.get("rate_limit_per_minute").and_then(|v| v.as_u64()) {
                self.tools.rate_limit_per_minute = Some(rate_limit as u32);
            }
        }

        // Merge hook configuration
        if let Some(hooks) = json.get("hooks").and_then(|v| v.as_object()) {
            let hook_config = self.hooks.get_or_insert(Default::default());
            if let Some(enabled) = hooks.get("enabled").and_then(|v| v.as_bool()) {
                hook_config.enabled = enabled;
            }
            if let Some(rate_limit) = hooks.get("rate_limit_per_minute").and_then(|v| v.as_u64()) {
                hook_config.rate_limit_per_minute = Some(rate_limit as u32);
            }
        }

        // Merge events configuration
        if let Some(events) = json.get("events").and_then(|v| v.as_object()) {
            if let Some(enabled) = events.get("enabled").and_then(|v| v.as_bool()) {
                debug!("Overriding events.enabled from env: {}", enabled);
                self.events.enabled = enabled;
            }

            if let Some(buffer_size) = events.get("buffer_size").and_then(|v| v.as_u64()) {
                debug!("Overriding events.buffer_size from env: {}", buffer_size);
                self.events.buffer_size = buffer_size as usize;
            }

            if let Some(emit_timing) = events.get("emit_timing_events").and_then(|v| v.as_bool()) {
                debug!(
                    "Overriding events.emit_timing_events from env: {}",
                    emit_timing
                );
                self.events.emit_timing_events = emit_timing;
            }

            if let Some(emit_state) = events.get("emit_state_events").and_then(|v| v.as_bool()) {
                debug!(
                    "Overriding events.emit_state_events from env: {}",
                    emit_state
                );
                self.events.emit_state_events = emit_state;
            }

            if let Some(emit_debug) = events.get("emit_debug_events").and_then(|v| v.as_bool()) {
                debug!(
                    "Overriding events.emit_debug_events from env: {}",
                    emit_debug
                );
                self.events.emit_debug_events = emit_debug;
            }

            if let Some(max_events) = events.get("max_events_per_second").and_then(|v| v.as_u64()) {
                debug!(
                    "Overriding events.max_events_per_second from env: {}",
                    max_events
                );
                self.events.max_events_per_second = Some(max_events as u32);
            }

            // Merge filtering configuration
            if let Some(filtering) = events.get("filtering").and_then(|v| v.as_object()) {
                if let Some(include_types_value) = filtering.get("include_types") {
                    if let Some(include_types_str) = include_types_value.as_str() {
                        // From environment variable - comma-separated string
                        self.events.filtering.include_types = include_types_str
                            .split(',')
                            .map(|s| s.trim().to_string())
                            .collect();
                    } else if let Some(include_types_array) = include_types_value.as_array() {
                        // From JSON - array of strings
                        self.events.filtering.include_types = include_types_array
                            .iter()
                            .filter_map(|v| v.as_str().map(String::from))
                            .collect();
                    }
                }

                if let Some(exclude_types_value) = filtering.get("exclude_types") {
                    if let Some(exclude_types_str) = exclude_types_value.as_str() {
                        // From environment variable - comma-separated string
                        self.events.filtering.exclude_types = exclude_types_str
                            .split(',')
                            .map(|s| s.trim().to_string())
                            .collect();
                    } else if let Some(exclude_types_array) = exclude_types_value.as_array() {
                        // From JSON - array of strings
                        self.events.filtering.exclude_types = exclude_types_array
                            .iter()
                            .filter_map(|v| v.as_str().map(String::from))
                            .collect();
                    }
                }

                if let Some(include_components_value) = filtering.get("include_components") {
                    if let Some(include_components_str) = include_components_value.as_str() {
                        // From environment variable - comma-separated string
                        self.events.filtering.include_components = include_components_str
                            .split(',')
                            .map(|s| s.trim().to_string())
                            .collect();
                    } else if let Some(include_components_array) =
                        include_components_value.as_array()
                    {
                        // From JSON - array of strings
                        self.events.filtering.include_components = include_components_array
                            .iter()
                            .filter_map(|v| v.as_str().map(String::from))
                            .collect();
                    }
                }

                if let Some(exclude_components_value) = filtering.get("exclude_components") {
                    if let Some(exclude_components_str) = exclude_components_value.as_str() {
                        // From environment variable - comma-separated string
                        self.events.filtering.exclude_components = exclude_components_str
                            .split(',')
                            .map(|s| s.trim().to_string())
                            .collect();
                    } else if let Some(exclude_components_array) =
                        exclude_components_value.as_array()
                    {
                        // From JSON - array of strings
                        self.events.filtering.exclude_components = exclude_components_array
                            .iter()
                            .filter_map(|v| v.as_str().map(String::from))
                            .collect();
                    }
                }
            }

            // Merge export configuration
            if let Some(export) = events.get("export").and_then(|v| v.as_object()) {
                if let Some(stdout) = export.get("stdout").and_then(|v| v.as_bool()) {
                    debug!("Overriding events.export.stdout from env: {}", stdout);
                    self.events.export.stdout = stdout;
                }

                if let Some(file) = export.get("file").and_then(|v| v.as_str()) {
                    debug!("Overriding events.export.file from env: {}", file);
                    self.events.export.file = Some(file.to_string());
                }

                if let Some(webhook) = export.get("webhook").and_then(|v| v.as_str()) {
                    debug!("Overriding events.export.webhook from env: {}", webhook);
                    self.events.export.webhook = Some(webhook.to_string());
                }

                if let Some(pretty_json) = export.get("pretty_json").and_then(|v| v.as_bool()) {
                    debug!(
                        "Overriding events.export.pretty_json from env: {}",
                        pretty_json
                    );
                    self.events.export.pretty_json = pretty_json;
                }
            }
        }

        // Merge RAG configuration
        if let Some(rag) = json.get("rag").and_then(|v| v.as_object()) {
            if let Some(enabled) = rag.get("enabled").and_then(|v| v.as_bool()) {
                debug!("Overriding rag.enabled from env: {}", enabled);
                self.rag.enabled = enabled;
            }

            if let Some(multi_tenant) = rag.get("multi_tenant").and_then(|v| v.as_bool()) {
                debug!("Overriding rag.multi_tenant from env: {}", multi_tenant);
                self.rag.multi_tenant = multi_tenant;
            }

            // Merge vector storage configuration
            if let Some(vector_storage) = rag.get("vector_storage").and_then(|v| v.as_object()) {
                if let Some(dimensions) = vector_storage.get("dimensions").and_then(|v| v.as_u64())
                {
                    debug!(
                        "Overriding rag.vector_storage.dimensions from env: {}",
                        dimensions
                    );
                    self.rag.vector_storage.dimensions = dimensions as usize;
                }

                if let Some(backend) = vector_storage.get("backend").and_then(|v| v.as_str()) {
                    debug!(
                        "Overriding rag.vector_storage.backend from env: {}",
                        backend
                    );
                    self.rag.vector_storage.backend = match backend {
                        "hnsw" => crate::rag::VectorBackend::HNSW,
                        _ => {
                            debug!("Unknown vector backend '{}', defaulting to HNSW", backend);
                            crate::rag::VectorBackend::HNSW
                        }
                    };
                }

                if let Some(persistence_path) = vector_storage
                    .get("persistence_path")
                    .and_then(|v| v.as_str())
                {
                    debug!(
                        "Overriding rag.vector_storage.persistence_path from env: {}",
                        persistence_path
                    );
                    self.rag.vector_storage.persistence_path =
                        Some(PathBuf::from(persistence_path));
                }

                if let Some(max_memory) =
                    vector_storage.get("max_memory_mb").and_then(|v| v.as_u64())
                {
                    debug!(
                        "Overriding rag.vector_storage.max_memory_mb from env: {}",
                        max_memory
                    );
                    self.rag.vector_storage.max_memory_mb = Some(max_memory as usize);
                }

                // Merge HNSW configuration
                if let Some(hnsw) = vector_storage.get("hnsw").and_then(|v| v.as_object()) {
                    if let Some(m) = hnsw.get("m").and_then(|v| v.as_u64()) {
                        debug!("Overriding rag.vector_storage.hnsw.m from env: {}", m);
                        self.rag.vector_storage.hnsw.m = m as usize;
                    }

                    if let Some(ef_construction) =
                        hnsw.get("ef_construction").and_then(|v| v.as_u64())
                    {
                        debug!(
                            "Overriding rag.vector_storage.hnsw.ef_construction from env: {}",
                            ef_construction
                        );
                        self.rag.vector_storage.hnsw.ef_construction = ef_construction as usize;
                    }

                    if let Some(ef_search) = hnsw.get("ef_search").and_then(|v| v.as_u64()) {
                        debug!(
                            "Overriding rag.vector_storage.hnsw.ef_search from env: {}",
                            ef_search
                        );
                        self.rag.vector_storage.hnsw.ef_search = ef_search as usize;
                    }

                    if let Some(max_elements) = hnsw.get("max_elements").and_then(|v| v.as_u64()) {
                        debug!(
                            "Overriding rag.vector_storage.hnsw.max_elements from env: {}",
                            max_elements
                        );
                        self.rag.vector_storage.hnsw.max_elements = max_elements as usize;
                    }

                    if let Some(seed) = hnsw.get("seed").and_then(|v| v.as_u64()) {
                        debug!("Overriding rag.vector_storage.hnsw.seed from env: {}", seed);
                        self.rag.vector_storage.hnsw.seed = Some(seed);
                    }

                    if let Some(metric) = hnsw.get("metric").and_then(|v| v.as_str()) {
                        debug!(
                            "Overriding rag.vector_storage.hnsw.metric from env: {}",
                            metric
                        );
                        self.rag.vector_storage.hnsw.metric = match metric {
                            "cosine" => crate::rag::DistanceMetric::Cosine,
                            "euclidean" => crate::rag::DistanceMetric::Euclidean,
                            "inner_product" => crate::rag::DistanceMetric::InnerProduct,
                            _ => {
                                debug!(
                                    "Unknown distance metric '{}', defaulting to Cosine",
                                    metric
                                );
                                crate::rag::DistanceMetric::Cosine
                            }
                        };
                    }

                    if let Some(allow_replace) =
                        hnsw.get("allow_replace_deleted").and_then(|v| v.as_bool())
                    {
                        debug!(
                            "Overriding rag.vector_storage.hnsw.allow_replace_deleted from env: {}",
                            allow_replace
                        );
                        self.rag.vector_storage.hnsw.allow_replace_deleted = allow_replace;
                    }

                    if let Some(num_threads) = hnsw.get("num_threads").and_then(|v| v.as_u64()) {
                        debug!(
                            "Overriding rag.vector_storage.hnsw.num_threads from env: {}",
                            num_threads
                        );
                        self.rag.vector_storage.hnsw.num_threads = Some(num_threads as usize);
                    }
                }
            }

            // Merge embedding configuration
            if let Some(embedding) = rag.get("embedding").and_then(|v| v.as_object()) {
                if let Some(provider) = embedding.get("default_provider").and_then(|v| v.as_str()) {
                    debug!(
                        "Overriding rag.embedding.default_provider from env: {}",
                        provider
                    );
                    self.rag.embedding.default_provider = provider.to_string();
                }

                if let Some(cache_enabled) =
                    embedding.get("cache_enabled").and_then(|v| v.as_bool())
                {
                    debug!(
                        "Overriding rag.embedding.cache_enabled from env: {}",
                        cache_enabled
                    );
                    self.rag.embedding.cache_enabled = cache_enabled;
                }

                if let Some(cache_size) = embedding.get("cache_size").and_then(|v| v.as_u64()) {
                    debug!(
                        "Overriding rag.embedding.cache_size from env: {}",
                        cache_size
                    );
                    self.rag.embedding.cache_size = cache_size as usize;
                }

                if let Some(cache_ttl) = embedding.get("cache_ttl_seconds").and_then(|v| v.as_u64())
                {
                    debug!(
                        "Overriding rag.embedding.cache_ttl_seconds from env: {}",
                        cache_ttl
                    );
                    self.rag.embedding.cache_ttl_seconds = cache_ttl;
                }

                if let Some(batch_size) = embedding.get("batch_size").and_then(|v| v.as_u64()) {
                    debug!(
                        "Overriding rag.embedding.batch_size from env: {}",
                        batch_size
                    );
                    self.rag.embedding.batch_size = batch_size as usize;
                }

                if let Some(timeout) = embedding.get("timeout_seconds").and_then(|v| v.as_u64()) {
                    debug!(
                        "Overriding rag.embedding.timeout_seconds from env: {}",
                        timeout
                    );
                    self.rag.embedding.timeout_seconds = timeout;
                }

                if let Some(max_retries) = embedding.get("max_retries").and_then(|v| v.as_u64()) {
                    debug!(
                        "Overriding rag.embedding.max_retries from env: {}",
                        max_retries
                    );
                    self.rag.embedding.max_retries = max_retries as u32;
                }
            }

            // Merge chunking configuration
            if let Some(chunking) = rag.get("chunking").and_then(|v| v.as_object()) {
                if let Some(strategy) = chunking.get("strategy").and_then(|v| v.as_str()) {
                    debug!("Overriding rag.chunking.strategy from env: {}", strategy);
                    self.rag.chunking.strategy = match strategy {
                        "sliding_window" => crate::rag::ChunkingStrategy::SlidingWindow,
                        "semantic" => crate::rag::ChunkingStrategy::Semantic,
                        "sentence" => crate::rag::ChunkingStrategy::Sentence,
                        _ => {
                            debug!(
                                "Unknown chunking strategy '{}', defaulting to SlidingWindow",
                                strategy
                            );
                            crate::rag::ChunkingStrategy::SlidingWindow
                        }
                    };
                }

                if let Some(chunk_size) = chunking.get("chunk_size").and_then(|v| v.as_u64()) {
                    debug!(
                        "Overriding rag.chunking.chunk_size from env: {}",
                        chunk_size
                    );
                    self.rag.chunking.chunk_size = chunk_size as usize;
                }

                if let Some(overlap) = chunking.get("overlap").and_then(|v| v.as_u64()) {
                    debug!("Overriding rag.chunking.overlap from env: {}", overlap);
                    self.rag.chunking.overlap = overlap as usize;
                }

                if let Some(max_chunk_size) =
                    chunking.get("max_chunk_size").and_then(|v| v.as_u64())
                {
                    debug!(
                        "Overriding rag.chunking.max_chunk_size from env: {}",
                        max_chunk_size
                    );
                    self.rag.chunking.max_chunk_size = max_chunk_size as usize;
                }

                if let Some(min_chunk_size) =
                    chunking.get("min_chunk_size").and_then(|v| v.as_u64())
                {
                    debug!(
                        "Overriding rag.chunking.min_chunk_size from env: {}",
                        min_chunk_size
                    );
                    self.rag.chunking.min_chunk_size = min_chunk_size as usize;
                }
            }

            // Merge cache configuration
            if let Some(cache) = rag.get("cache").and_then(|v| v.as_object()) {
                if let Some(search_cache_enabled) =
                    cache.get("search_cache_enabled").and_then(|v| v.as_bool())
                {
                    debug!(
                        "Overriding rag.cache.search_cache_enabled from env: {}",
                        search_cache_enabled
                    );
                    self.rag.cache.search_cache_enabled = search_cache_enabled;
                }

                if let Some(search_cache_size) =
                    cache.get("search_cache_size").and_then(|v| v.as_u64())
                {
                    debug!(
                        "Overriding rag.cache.search_cache_size from env: {}",
                        search_cache_size
                    );
                    self.rag.cache.search_cache_size = search_cache_size as usize;
                }

                if let Some(search_cache_ttl) = cache
                    .get("search_cache_ttl_seconds")
                    .and_then(|v| v.as_u64())
                {
                    debug!(
                        "Overriding rag.cache.search_cache_ttl_seconds from env: {}",
                        search_cache_ttl
                    );
                    self.rag.cache.search_cache_ttl_seconds = search_cache_ttl;
                }

                if let Some(document_cache_enabled) = cache
                    .get("document_cache_enabled")
                    .and_then(|v| v.as_bool())
                {
                    debug!(
                        "Overriding rag.cache.document_cache_enabled from env: {}",
                        document_cache_enabled
                    );
                    self.rag.cache.document_cache_enabled = document_cache_enabled;
                }

                if let Some(document_cache_size) =
                    cache.get("document_cache_size_mb").and_then(|v| v.as_u64())
                {
                    debug!(
                        "Overriding rag.cache.document_cache_size_mb from env: {}",
                        document_cache_size
                    );
                    self.rag.cache.document_cache_size_mb = document_cache_size as usize;
                }
            }
        }

        Ok(())
    }

    /// Apply environment variable overrides (DEPRECATED - use apply_env_registry)
    #[deprecated(note = "Use apply_env_registry() for centralized environment handling")]
    pub fn apply_env_overrides(&mut self) -> Result<(), ConfigError> {
        self.apply_env_registry()
    }

    /// Validate configuration
    pub fn validate(&self) -> Result<(), ConfigError> {
        validation::validate_config(self)
    }

    /// Get engine-specific configuration
    pub fn get_engine_config(&self, engine_name: &str) -> Result<serde_json::Value, ConfigError> {
        match engine_name {
            "lua" => Ok(serde_json::to_value(&self.engines.lua)?),
            "javascript" | "js" => Ok(serde_json::to_value(&self.engines.javascript)?),
            custom => {
                self.engines
                    .custom
                    .get(custom)
                    .cloned()
                    .ok_or_else(|| ConfigError::Validation {
                        field: Some("engine".to_string()),
                        message: format!("Engine configuration not found for '{custom}'"),
                    })
            }
        }
    }

    /// Check if an engine is configured
    #[must_use]
    pub fn supports_engine(&self, engine_name: &str) -> bool {
        match engine_name {
            "lua" | "javascript" | "js" => true,
            custom => self.engines.custom.contains_key(custom),
        }
    }

    /// Discover configuration file in standard locations
    pub async fn discover_config_file() -> Result<Option<PathBuf>, ConfigError> {
        // Check current directory first
        for path in CONFIG_SEARCH_PATHS {
            let path = PathBuf::from(path);
            if path.exists() {
                return Ok(Some(path));
            }
        }

        // Check home directory
        if let Ok(home_dir) = std_env::var("HOME").or_else(|_| std_env::var("USERPROFILE")) {
            let home_path = PathBuf::from(home_dir);

            for filename in &[".llmspell.toml", ".config/llmspell.toml"] {
                let path = home_path.join(filename);
                if path.exists() {
                    return Ok(Some(path));
                }
            }
        }

        // Check system config directories (Linux/macOS style)
        if let Ok(xdg_config) = std_env::var("XDG_CONFIG_HOME") {
            let path = PathBuf::from(xdg_config)
                .join("llmspell")
                .join("config.toml");
            if path.exists() {
                return Ok(Some(path));
            }
        }

        Ok(None)
    }

    /// Load configuration with optional builtin profile
    ///
    /// Loads configuration using the following precedence rules:
    /// 1. `profile` - If provided, loads the named builtin profile
    /// 2. `explicit_path` - If provided and profile is None, loads config from file path
    /// 3. Discovery - If neither provided, searches standard locations
    /// 4. Default - If nothing found, uses default configuration
    ///
    /// Environment variables override all configuration sources.
    ///
    /// # Arguments
    ///
    /// * `explicit_path` - Optional path to custom config file
    /// * `profile` - Optional builtin profile name
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// // Load builtin profile
    /// let config = LLMSpellConfig::load_with_profile(None, Some("development")).await?;
    ///
    /// // Load custom config file
    /// let config = LLMSpellConfig::load_with_profile(Some(Path::new("my.toml")), None).await?;
    ///
    /// // Use discovery
    /// let config = LLMSpellConfig::load_with_profile(None, None).await?;
    /// ```
    pub async fn load_with_profile(
        explicit_path: Option<&Path>,
        profile: Option<&str>,
    ) -> Result<Self, ConfigError> {
        let mut config = if let Some(prof) = profile {
            debug!("Loading builtin profile: {}", prof);
            Self::load_builtin_profile(prof)?
        } else if let Some(path) = explicit_path {
            debug!("Loading config from file: {}", path.display());
            if path.exists() {
                Self::load_from_file(path).await?
            } else {
                return Err(ConfigError::NotFound {
                    path: path.to_string_lossy().to_string(),
                    message: "Explicitly specified config file not found".to_string(),
                });
            }
        } else {
            debug!("Using config discovery");
            if let Some(discovered) = Self::discover_config_file().await? {
                debug!("Discovered config file: {}", discovered.display());
                Self::load_from_file(&discovered).await?
            } else {
                debug!("No config file found, using defaults");
                Self::default()
            }
        };

        // Environment variables ALWAYS override
        config.apply_env_registry()?;
        config.validate()?;

        Ok(config)
    }

    /// Load a builtin configuration profile
    ///
    /// Builtin profiles are complete TOML configurations embedded at compile time.
    ///
    /// # Available Profiles
    ///
    /// ## Core Profiles
    /// - `minimal` - Bare minimum settings for basic operation
    /// - `development` - Development-friendly settings with debug logging
    ///
    /// ## Local LLM Profiles
    /// - `ollama` - Ollama backend configuration
    /// - `candle` - Candle embedded inference configuration
    ///
    /// ## RAG Profiles
    /// - `rag-dev` - Development RAG settings (small dimensions, fast iteration)
    /// - `rag-prod` - Production RAG settings (reliability, monitoring, security)
    /// - `rag-perf` - Performance-optimized RAG settings (high memory, many cores)
    ///
    /// # Errors
    ///
    /// Returns `ConfigError::NotFound` if the profile name is not recognized.
    fn load_builtin_profile(name: &str) -> Result<Self, ConfigError> {
        let toml_content = match name {
            // Core profiles
            "minimal" => include_str!("../builtins/minimal.toml"),
            "development" => include_str!("../builtins/development.toml"),
            "default" => include_str!("../builtins/default.toml"),

            // Common workflow profiles
            "providers" => include_str!("../builtins/providers.toml"),
            "state" => include_str!("../builtins/state.toml"),
            "sessions" => include_str!("../builtins/sessions.toml"),

            // Local LLM profiles
            "ollama" => include_str!("../builtins/ollama.toml"),
            "candle" => include_str!("../builtins/candle.toml"),

            // Memory system profile
            "memory" => include_str!("../builtins/memory.toml"),

            // RAG profiles
            "rag-dev" => include_str!("../builtins/rag-development.toml"),
            "rag-prod" => include_str!("../builtins/rag-production.toml"),
            "rag-perf" => include_str!("../builtins/rag-performance.toml"),

            _ => {
                return Err(ConfigError::NotFound {
                    path: format!("builtin:{}", name),
                    message: format!(
                        "Unknown builtin profile '{}'.\n\
                         Available profiles:\n\
                         Core: minimal, development, default\n\
                         Common: providers, state, sessions\n\
                         Local LLM: ollama, candle\n\
                         Memory: memory\n\
                         RAG: rag-dev, rag-prod, rag-perf",
                        name
                    ),
                });
            }
        };

        Self::from_toml(toml_content)
    }

    /// List available builtin profiles
    ///
    /// Returns a vector of all builtin profile names that can be used with
    /// `load_with_profile()` or the `--profile` CLI flag.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use llmspell_config::LLMSpellConfig;
    ///
    /// let profiles = LLMSpellConfig::list_builtin_profiles();
    /// assert_eq!(profiles.len(), 12);
    /// assert!(profiles.contains(&"default"));
    /// assert!(profiles.contains(&"development"));
    /// assert!(profiles.contains(&"memory"));
    /// assert!(profiles.contains(&"ollama"));
    /// assert!(profiles.contains(&"rag-prod"));
    /// ```
    #[must_use]
    pub fn list_builtin_profiles() -> Vec<&'static str> {
        vec![
            "minimal",
            "development",
            "default",
            "providers",
            "state",
            "sessions",
            "ollama",
            "candle",
            "memory",
            "rag-dev",
            "rag-prod",
            "rag-perf",
        ]
    }

    /// Get metadata for a specific builtin profile
    ///
    /// Returns structured information about a profile including category,
    /// description, use cases, and key features.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use llmspell_config::LLMSpellConfig;
    ///
    /// let metadata = LLMSpellConfig::get_profile_metadata("providers").unwrap();
    /// assert_eq!(metadata.category, "Common Workflows");
    /// assert_eq!(metadata.name, "providers");
    /// ```
    #[must_use]
    pub fn get_profile_metadata(name: &str) -> Option<ProfileMetadata> {
        match name {
            "minimal" => Some(ProfileMetadata {
                name: "minimal",
                category: "Core",
                description: "Tools only, no LLM providers",
                use_cases: vec![
                    "Testing tools",
                    "Learning workflow patterns",
                    "Scripts without LLM access",
                ],
                features: vec!["Lua stdlib: Basic", "No providers", "No RAG", "No sessions"],
            }),
            "development" => Some(ProfileMetadata {
                name: "development",
                category: "Core",
                description: "Dev settings with debug logging",
                use_cases: vec!["Development", "Debugging", "Learning LLM integration"],
                features: vec![
                    "Lua stdlib: All",
                    "OpenAI + Anthropic",
                    "Debug logging",
                    "Small resource limits",
                ],
            }),
            "default" => Some(ProfileMetadata {
                name: "default",
                category: "Core",
                description: "Simple local LLM setup using Ollama",
                use_cases: vec![
                    "General purpose scripting",
                    "Template execution",
                    "Agent development",
                ],
                features: vec![
                    "Lua stdlib: All",
                    "Ollama provider (llama3.2:3b)",
                    "4096 max tokens",
                    "Sensible defaults",
                ],
            }),
            "providers" => Some(ProfileMetadata {
                name: "providers",
                category: "Common Workflows",
                description: "OpenAI + Anthropic setup",
                use_cases: vec!["Agent examples", "LLM scripts", "Production agents"],
                features: vec![
                    "OpenAI gpt-3.5-turbo",
                    "Anthropic claude-3-haiku",
                    "Cost-efficient models",
                    "No RAG",
                ],
            }),
            "state" => Some(ProfileMetadata {
                name: "state",
                category: "Common Workflows",
                description: "State persistence with memory backend",
                use_cases: vec![
                    "State management examples",
                    "Scripts requiring state",
                    "Learning persistence",
                ],
                features: vec![
                    "Memory backend",
                    "10MB max state",
                    "No providers",
                    "Migration/backup disabled",
                ],
            }),
            "sessions" => Some(ProfileMetadata {
                name: "sessions",
                category: "Common Workflows",
                description: "Sessions + state + hooks + events",
                use_cases: vec![
                    "Conversational apps",
                    "Session management",
                    "Event-driven workflows",
                ],
                features: vec![
                    "Session tracking",
                    "Artifact storage",
                    "Hooks enabled",
                    "Events enabled",
                ],
            }),
            "ollama" => Some(ProfileMetadata {
                name: "ollama",
                category: "Local LLM",
                description: "Ollama backend configuration",
                use_cases: vec!["Local LLM with Ollama", "Offline inference", "GGUF models"],
                features: vec![
                    "Ollama provider",
                    "Local inference",
                    "No API keys needed",
                    "Full stdlib",
                ],
            }),
            "candle" => Some(ProfileMetadata {
                name: "candle",
                category: "Local LLM",
                description: "Candle embedded inference",
                use_cases: vec![
                    "Local LLM with Candle",
                    "CPU/GPU inference",
                    "Rust-native models",
                ],
                features: vec![
                    "Candle provider",
                    "Rust inference",
                    "No API keys needed",
                    "Full stdlib",
                ],
            }),
            "memory" => Some(ProfileMetadata {
                name: "memory",
                category: "Memory System",
                description: "Adaptive memory with LLM consolidation and temporal knowledge graph",
                use_cases: vec![
                    "Long-running agents",
                    "Knowledge accumulation",
                    "RAG with episodic memory",
                ],
                features: vec![
                    "Episodic memory storage",
                    "LLM-driven consolidation",
                    "Bi-temporal knowledge graph",
                    "Context-aware retrieval",
                    "Adaptive daemon scheduling",
                ],
            }),
            "rag-dev" => Some(ProfileMetadata {
                name: "rag-dev",
                category: "RAG",
                description: "Development RAG (small dims, fast)",
                use_cases: vec![
                    "Learning RAG",
                    "Prototyping knowledge bases",
                    "Fast iteration",
                ],
                features: vec![
                    "384-dim vectors",
                    "HNSW index",
                    "OpenAI embeddings",
                    "Small memory footprint",
                ],
            }),
            "rag-prod" => Some(ProfileMetadata {
                name: "rag-prod",
                category: "RAG",
                description: "Production RAG (reliability, monitoring)",
                use_cases: vec![
                    "Production RAG deployment",
                    "Large knowledge bases",
                    "SaaS platforms",
                ],
                features: vec![
                    "1536-dim vectors",
                    "Caching enabled",
                    "Monitoring ready",
                    "Production settings",
                ],
            }),
            "rag-perf" => Some(ProfileMetadata {
                name: "rag-perf",
                category: "RAG",
                description: "Performance RAG (high memory, cores)",
                use_cases: vec![
                    "High-performance RAG",
                    "Large-scale search",
                    "Multi-core systems",
                ],
                features: vec![
                    "Optimized HNSW",
                    "Large caches",
                    "Multi-threaded",
                    "High memory limits",
                ],
            }),
            _ => None,
        }
    }

    /// List metadata for all builtin profiles
    ///
    /// Returns structured information about all available profiles,
    /// organized by category.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use llmspell_config::LLMSpellConfig;
    ///
    /// let all_metadata = LLMSpellConfig::list_profile_metadata();
    /// assert_eq!(all_metadata.len(), 12);
    /// ```
    #[must_use]
    pub fn list_profile_metadata() -> Vec<ProfileMetadata> {
        Self::list_builtin_profiles()
            .iter()
            .filter_map(|name| Self::get_profile_metadata(name))
            .collect()
    }

    /// Load configuration with automatic discovery
    ///
    /// This method delegates to `load_with_profile` with no profile specified.
    /// Kept for backward compatibility.
    ///
    /// # Deprecation Note
    ///
    /// Consider using `load_with_profile(explicit_path, None)` directly for new code.
    pub async fn load_with_discovery(explicit_path: Option<&Path>) -> Result<Self, ConfigError> {
        Self::load_with_profile(explicit_path, None).await
    }
}

/// Builder for `LLMSpellConfig`
#[derive(Debug, Clone)]
pub struct LLMSpellConfigBuilder {
    config: LLMSpellConfig,
}

impl LLMSpellConfigBuilder {
    /// Create a new builder with default configuration
    #[must_use]
    pub fn new() -> Self {
        Self {
            config: LLMSpellConfig::default(),
        }
    }

    /// Set the default script engine
    #[must_use]
    pub fn default_engine(mut self, engine: impl Into<String>) -> Self {
        self.config.default_engine = engine.into();
        self
    }

    /// Set the engine configurations
    #[must_use]
    pub fn engines(mut self, engines: EngineConfigs) -> Self {
        self.config.engines = engines;
        self
    }

    /// Set the provider configuration
    #[must_use]
    pub fn providers(mut self, providers: ProviderManagerConfig) -> Self {
        self.config.providers = providers;
        self
    }

    /// Set the global runtime configuration
    #[must_use]
    pub fn runtime(mut self, runtime: GlobalRuntimeConfig) -> Self {
        self.config.runtime = runtime;
        self
    }

    /// Set the tools configuration
    #[must_use]
    pub fn tools(mut self, tools: ToolsConfig) -> Self {
        self.config.tools = tools;
        self
    }

    /// Set the events configuration
    #[must_use]
    pub fn events(mut self, events: EventsConfig) -> Self {
        self.config.events = events;
        self
    }

    /// Set the debug configuration
    #[must_use]
    pub fn debug(mut self, debug: DebugConfig) -> Self {
        self.config.debug = debug;
        self
    }

    /// Set the RAG configuration
    #[must_use]
    pub fn rag(mut self, rag: RAGConfig) -> Self {
        self.config.rag = rag;
        self
    }

    /// Build the configuration
    #[must_use]
    pub fn build(self) -> LLMSpellConfig {
        self.config
    }
}

impl Default for LLMSpellConfigBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Global runtime configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(default)]
pub struct GlobalRuntimeConfig {
    /// Maximum concurrent scripts
    pub max_concurrent_scripts: usize,
    /// Script execution timeout in seconds
    pub script_timeout_seconds: u64,
    /// Enable streaming by default
    pub enable_streaming: bool,
    /// Security settings
    pub security: SecurityConfig,
    /// State persistence settings
    pub state_persistence: StatePersistenceConfig,
    /// Session management settings
    pub sessions: SessionConfig,
    /// Memory system configuration
    pub memory: MemoryConfig,
}

impl Default for GlobalRuntimeConfig {
    fn default() -> Self {
        Self {
            max_concurrent_scripts: 10,
            script_timeout_seconds: 300,
            enable_streaming: true,
            security: SecurityConfig::default(),
            state_persistence: StatePersistenceConfig::default(),
            sessions: SessionConfig::default(),
            memory: MemoryConfig::default(),
        }
    }
}

impl GlobalRuntimeConfig {
    /// Create a new builder for `GlobalRuntimeConfig`
    #[must_use]
    pub fn builder() -> GlobalRuntimeConfigBuilder {
        GlobalRuntimeConfigBuilder::new()
    }
}

/// Builder for `GlobalRuntimeConfig`
#[derive(Debug, Clone)]
pub struct GlobalRuntimeConfigBuilder {
    config: GlobalRuntimeConfig,
}

impl GlobalRuntimeConfigBuilder {
    /// Create a new builder with default configuration
    #[must_use]
    pub fn new() -> Self {
        Self {
            config: GlobalRuntimeConfig::default(),
        }
    }

    /// Set the maximum concurrent scripts
    #[must_use]
    pub const fn max_concurrent_scripts(mut self, max: usize) -> Self {
        self.config.max_concurrent_scripts = max;
        self
    }

    /// Set the script execution timeout in seconds
    #[must_use]
    pub const fn script_timeout_seconds(mut self, timeout: u64) -> Self {
        self.config.script_timeout_seconds = timeout;
        self
    }

    /// Enable or disable streaming
    #[must_use]
    pub const fn enable_streaming(mut self, enable: bool) -> Self {
        self.config.enable_streaming = enable;
        self
    }

    /// Set the security configuration
    #[must_use]
    pub const fn security(mut self, security: SecurityConfig) -> Self {
        self.config.security = security;
        self
    }

    /// Set the state persistence configuration
    #[must_use]
    pub fn state_persistence(mut self, persistence: StatePersistenceConfig) -> Self {
        self.config.state_persistence = persistence;
        self
    }

    /// Set the session configuration
    #[must_use]
    pub fn sessions(mut self, sessions: SessionConfig) -> Self {
        self.config.sessions = sessions;
        self
    }

    /// Set the memory configuration
    #[must_use]
    pub fn memory(mut self, memory: MemoryConfig) -> Self {
        self.config.memory = memory;
        self
    }

    /// Build the configuration
    #[must_use]
    pub fn build(self) -> GlobalRuntimeConfig {
        self.config
    }
}

impl Default for GlobalRuntimeConfigBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Security configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(default)]
pub struct SecurityConfig {
    /// Allow file system access
    pub allow_file_access: bool,
    /// Allow network access
    pub allow_network_access: bool,
    /// Allow process spawning
    pub allow_process_spawn: bool,
    /// Maximum memory usage in bytes
    pub max_memory_bytes: Option<usize>,
    /// Maximum execution time in milliseconds
    pub max_execution_time_ms: Option<u64>,
}

impl Default for SecurityConfig {
    fn default() -> Self {
        Self {
            allow_file_access: false,
            allow_network_access: true,
            allow_process_spawn: false,
            max_memory_bytes: Some(50_000_000),   // 50MB
            max_execution_time_ms: Some(300_000), // 5 minutes
        }
    }
}

/// State persistence configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(default)]
pub struct StatePersistenceConfig {
    /// Enable state persistence (flattened from flags.core.enabled)
    pub enabled: bool,
    /// Enable migration functionality (flattened from flags.core.migration_enabled)
    pub migration_enabled: bool,
    /// Automatic backup on migration (flattened from flags.backup.backup_on_migration)
    pub backup_on_migration: bool,
    /// Enable backup functionality (flattened from flags.backup.backup_enabled)
    pub backup_enabled: bool,
    /// Backend type for storage (memory, file, redis, etc.)
    pub backend_type: String,
    /// Directory for schema definitions
    pub schema_directory: Option<String>,
    /// Maximum state size per key in bytes
    pub max_state_size_bytes: Option<usize>,
    /// Backup configuration
    pub backup: Option<BackupConfig>,
}

/// Backup configuration for state persistence
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct BackupConfig {
    /// Directory for backup storage
    pub backup_dir: Option<String>,
    /// Enable compression for backups
    pub compression_enabled: bool,
    /// Compression type to use
    pub compression_type: String,
    /// Compression level (1-9)
    pub compression_level: u8,
    /// Enable incremental backups
    pub incremental_enabled: bool,
    /// Maximum number of backups to keep
    pub max_backups: Option<usize>,
    /// Maximum age of backups in seconds
    pub max_backup_age: Option<u64>,
}

impl Default for StatePersistenceConfig {
    fn default() -> Self {
        Self {
            enabled: true, // In-memory state by default
            migration_enabled: false,
            backup_on_migration: true,
            backup_enabled: false,
            backend_type: "memory".to_string(),
            schema_directory: None,
            max_state_size_bytes: Some(10_000_000), // 10MB per key
            backup: None,
        }
    }
}

impl Default for BackupConfig {
    fn default() -> Self {
        Self {
            backup_dir: Some("./backups".to_string()),
            compression_enabled: true,
            compression_type: "zstd".to_string(),
            compression_level: 3,
            incremental_enabled: true,
            max_backups: Some(10),
            max_backup_age: Some(2_592_000), // 30 days
        }
    }
}

/// Hook system configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(default)]
pub struct HookConfig {
    /// Enable hook system
    pub enabled: bool,
    /// Rate limiting for hooks (executions per minute)
    pub rate_limit_per_minute: Option<u32>,
    /// Hook timeout in milliseconds
    pub timeout_ms: Option<u64>,
    /// Circuit breaker threshold
    pub circuit_breaker_threshold: Option<f64>,
}

impl Default for HookConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            rate_limit_per_minute: Some(100),
            timeout_ms: Some(5000),
            circuit_breaker_threshold: Some(0.01), // 1% overhead threshold
        }
    }
}

/// Session management configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(default)]
pub struct SessionConfig {
    /// Enable session management
    pub enabled: bool,
    /// Maximum number of concurrent sessions
    pub max_sessions: usize,
    /// Maximum artifacts per session
    pub max_artifacts_per_session: usize,
    /// Artifact compression threshold in bytes
    pub artifact_compression_threshold: usize,
    /// Session timeout in seconds
    pub session_timeout_seconds: u64,
    /// Storage backend type (memory, sqlite, postgres)
    pub storage_backend: String,
}

impl Default for SessionConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            max_sessions: 100,
            max_artifacts_per_session: 1000,
            artifact_compression_threshold: 10240, // 10KB
            session_timeout_seconds: 3600,         // 1 hour
            storage_backend: "memory".to_string(),
        }
    }
}

/// Event system configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(default)]
pub struct EventsConfig {
    /// Enable event system globally
    pub enabled: bool,
    /// EventBus buffer size for queuing events
    pub buffer_size: usize,
    /// Enable timing/performance events
    pub emit_timing_events: bool,
    /// Enable state change events
    pub emit_state_events: bool,
    /// Enable debug-level events
    pub emit_debug_events: bool,
    /// Maximum events per second (rate limiting)
    pub max_events_per_second: Option<u32>,
    /// Event filtering configuration
    pub filtering: EventFilterConfig,
    /// Event export configuration
    pub export: EventExportConfig,
}

impl Default for EventsConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            buffer_size: 10000,
            emit_timing_events: true,
            emit_state_events: false,
            emit_debug_events: false,
            max_events_per_second: None,
            filtering: EventFilterConfig::default(),
            export: EventExportConfig::default(),
        }
    }
}

/// Event filtering configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(default)]
pub struct EventFilterConfig {
    /// Event types to include (glob patterns)
    pub include_types: Vec<String>,
    /// Event types to exclude (glob patterns)
    pub exclude_types: Vec<String>,
    /// Component IDs to include (glob patterns)
    pub include_components: Vec<String>,
    /// Component IDs to exclude (glob patterns)
    pub exclude_components: Vec<String>,
}

impl Default for EventFilterConfig {
    fn default() -> Self {
        Self {
            include_types: vec!["*".to_string()], // Include all by default
            exclude_types: Vec::new(),
            include_components: vec!["*".to_string()],
            exclude_components: Vec::new(),
        }
    }
}

/// Event export configuration
#[derive(Debug, Clone, Deserialize, Serialize, Default)]
#[serde(default)]
pub struct EventExportConfig {
    /// Export events to stdout (for debugging)
    pub stdout: bool,
    /// Export events to file
    pub file: Option<String>,
    /// Export events to webhook
    pub webhook: Option<String>,
    /// Pretty-print JSON output
    pub pretty_json: bool,
}

/// Configuration errors
#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("Configuration file not found: {path} - {message}")]
    NotFound { path: String, message: String },

    #[error("Configuration validation failed in field '{field:?}': {message}")]
    Validation {
        field: Option<String>,
        message: String,
    },

    #[error("Environment variable parsing error: {message}")]
    Environment { message: String },

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("TOML parsing error: {0}")]
    Toml(#[from] toml::de::Error),

    #[error("JSON serialization error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("General error: {0}")]
    Other(#[from] anyhow::Error),
}

impl From<ConfigError> for LLMSpellError {
    fn from(err: ConfigError) -> Self {
        match err {
            ConfigError::Validation { field, message } => {
                LLMSpellError::Validation { field, message }
            }
            ConfigError::NotFound { path, message } => LLMSpellError::Configuration {
                message: format!("Configuration file not found: {} - {}", path, message),
                source: None,
            },
            _ => LLMSpellError::Configuration {
                message: err.to_string(),
                source: Some(Box::new(err)),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_llmspell_config_default() {
        let config = LLMSpellConfig::default();
        assert_eq!(config.default_engine, "lua");
        assert!(config.supports_engine("lua"));
        assert!(config.supports_engine("javascript"));
        assert!(!config.supports_engine("python"));
    }

    #[test]
    fn test_security_config_defaults() {
        let config = SecurityConfig::default();
        assert!(!config.allow_file_access);
        assert!(config.allow_network_access);
        assert!(!config.allow_process_spawn);
        assert_eq!(config.max_memory_bytes, Some(50_000_000));
        assert_eq!(config.max_execution_time_ms, Some(300_000));
    }

    #[test]
    fn test_config_builder() {
        let config = LLMSpellConfig::builder()
            .default_engine("javascript")
            .build();

        assert_eq!(config.default_engine, "javascript");
    }

    #[test]
    fn test_minimal_toml_config() {
        let toml_str = r#"default_engine = "lua""#;
        let result = LLMSpellConfig::from_toml(toml_str);

        // Should parse successfully
        assert!(
            result.is_ok(),
            "Failed to parse minimal config: {:?}",
            result
        );

        let config = result.unwrap();
        assert_eq!(config.default_engine, "lua");
    }

    #[test]
    fn test_empty_toml_config() {
        let toml_str = "";
        let result = LLMSpellConfig::from_toml(toml_str);

        // Should use defaults
        assert!(result.is_ok(), "Failed to parse empty config: {:?}", result);

        let config = result.unwrap();
        assert_eq!(config.default_engine, "lua");
    }

    #[test]
    fn test_rag_config_environment_merge() {
        use serde_json::json;

        let mut config = LLMSpellConfig::default();

        // Verify initial defaults
        assert!(!config.rag.enabled);
        assert_eq!(config.rag.vector_storage.dimensions, 384);
        assert_eq!(config.rag.embedding.default_provider, "openai");
        assert!(!config.rag.multi_tenant);

        // Simulate environment variable override
        let env_json = json!({
            "rag": {
                "enabled": true,
                "multi_tenant": true,
                "vector_storage": {
                    "dimensions": 768,
                    "backend": "hnsw",
                    "persistence_path": "/tmp/vectors",
                    "max_memory_mb": 1000,
                    "hnsw": {
                        "m": 32,
                        "ef_construction": 400,
                        "ef_search": 100,
                        "max_elements": 500000,
                        "seed": 42,
                        "metric": "cosine",
                        "allow_replace_deleted": false,
                        "num_threads": 4
                    }
                },
                "embedding": {
                    "default_provider": "local",
                    "cache_enabled": false,
                    "cache_size": 5000,
                    "cache_ttl_seconds": 7200,
                    "batch_size": 64,
                    "timeout_seconds": 60,
                    "max_retries": 5
                },
                "chunking": {
                    "strategy": "semantic",
                    "chunk_size": 1024,
                    "overlap": 128,
                    "max_chunk_size": 4096,
                    "min_chunk_size": 200
                },
                "cache": {
                    "search_cache_enabled": false,
                    "search_cache_size": 2000,
                    "search_cache_ttl_seconds": 900,
                    "document_cache_enabled": false,
                    "document_cache_size_mb": 200
                }
            }
        });

        // Apply environment overrides
        config.merge_from_json(&env_json).unwrap();

        // Verify overrides were applied
        assert!(config.rag.enabled);
        assert!(config.rag.multi_tenant);

        // Vector storage
        assert_eq!(config.rag.vector_storage.dimensions, 768);
        assert!(matches!(
            config.rag.vector_storage.backend,
            crate::rag::VectorBackend::HNSW
        ));
        assert_eq!(
            config.rag.vector_storage.persistence_path,
            Some(PathBuf::from("/tmp/vectors"))
        );
        assert_eq!(config.rag.vector_storage.max_memory_mb, Some(1000));

        // HNSW
        assert_eq!(config.rag.vector_storage.hnsw.m, 32);
        assert_eq!(config.rag.vector_storage.hnsw.ef_construction, 400);
        assert_eq!(config.rag.vector_storage.hnsw.ef_search, 100);
        assert_eq!(config.rag.vector_storage.hnsw.max_elements, 500000);
        assert_eq!(config.rag.vector_storage.hnsw.seed, Some(42));
        assert!(matches!(
            config.rag.vector_storage.hnsw.metric,
            crate::rag::DistanceMetric::Cosine
        ));
        assert!(!config.rag.vector_storage.hnsw.allow_replace_deleted);
        assert_eq!(config.rag.vector_storage.hnsw.num_threads, Some(4));

        // Embedding
        assert_eq!(config.rag.embedding.default_provider, "local");
        assert!(!config.rag.embedding.cache_enabled);
        assert_eq!(config.rag.embedding.cache_size, 5000);
        assert_eq!(config.rag.embedding.cache_ttl_seconds, 7200);
        assert_eq!(config.rag.embedding.batch_size, 64);
        assert_eq!(config.rag.embedding.timeout_seconds, 60);
        assert_eq!(config.rag.embedding.max_retries, 5);

        // Chunking
        assert!(matches!(
            config.rag.chunking.strategy,
            crate::rag::ChunkingStrategy::Semantic
        ));
        assert_eq!(config.rag.chunking.chunk_size, 1024);
        assert_eq!(config.rag.chunking.overlap, 128);
        assert_eq!(config.rag.chunking.max_chunk_size, 4096);
        assert_eq!(config.rag.chunking.min_chunk_size, 200);

        // Cache
        assert!(!config.rag.cache.search_cache_enabled);
        assert_eq!(config.rag.cache.search_cache_size, 2000);
        assert_eq!(config.rag.cache.search_cache_ttl_seconds, 900);
        assert!(!config.rag.cache.document_cache_enabled);
        assert_eq!(config.rag.cache.document_cache_size_mb, 200);
    }

    #[test]
    fn test_rag_config_unknown_enum_fallbacks() {
        use serde_json::json;

        let mut config = LLMSpellConfig::default();

        let env_json = json!({
            "rag": {
                "vector_storage": {
                    "backend": "unknown_backend",
                    "hnsw": {
                        "metric": "unknown_metric"
                    }
                },
                "chunking": {
                    "strategy": "unknown_strategy"
                }
            }
        });

        // Should not fail and use fallback values
        config.merge_from_json(&env_json).unwrap();

        // Verify fallbacks were used
        assert!(matches!(
            config.rag.vector_storage.backend,
            crate::rag::VectorBackend::HNSW
        ));
        assert!(matches!(
            config.rag.vector_storage.hnsw.metric,
            crate::rag::DistanceMetric::Cosine
        ));
        assert!(matches!(
            config.rag.chunking.strategy,
            crate::rag::ChunkingStrategy::SlidingWindow
        ));
    }

    #[test]
    fn test_provider_toml_deserialization() {
        let toml_str = r#"
default_engine = "lua"

[providers]

[providers.candle]
provider_type = "candle"
enabled = true
timeout_seconds = 300
    "#;

        let config: LLMSpellConfig = toml::from_str(toml_str).unwrap();

        println!("Providers count: {}", config.providers.providers.len());
        println!("Providers: {:#?}", config.providers.providers);

        assert!(
            !config.providers.providers.is_empty(),
            "Should have at least one provider"
        );
        assert!(
            config.providers.providers.contains_key("candle"),
            "Should have candle provider"
        );

        let candle = config.providers.providers.get("candle").unwrap();
        assert_eq!(candle.provider_type, "candle");
        assert!(candle.enabled);
        assert_eq!(candle.timeout_seconds, Some(300));
    }

    // Profile system tests
    #[test]
    fn test_list_builtin_profiles() {
        let profiles = LLMSpellConfig::list_builtin_profiles();
        assert_eq!(profiles.len(), 12);
        assert!(profiles.contains(&"minimal"));
        assert!(profiles.contains(&"development"));
        assert!(profiles.contains(&"default"));
        assert!(profiles.contains(&"providers"));
        assert!(profiles.contains(&"state"));
        assert!(profiles.contains(&"sessions"));
        assert!(profiles.contains(&"ollama"));
        assert!(profiles.contains(&"candle"));
        assert!(profiles.contains(&"memory"));
        assert!(profiles.contains(&"rag-dev"));
        assert!(profiles.contains(&"rag-prod"));
        assert!(profiles.contains(&"rag-perf"));
    }

    #[test]
    fn test_load_builtin_profile_minimal() {
        let config = LLMSpellConfig::load_builtin_profile("minimal").unwrap();

        // Minimal profile should have basic settings
        assert_eq!(config.default_engine, "lua");
        assert!(matches!(
            config.engines.lua.stdlib,
            crate::engines::StdlibLevel::Basic
        ));

        // No providers configured
        assert!(config.providers.providers.is_empty());

        // RAG disabled by default
        assert!(!config.rag.enabled);
    }

    #[test]
    fn test_load_builtin_profile_development() {
        let config = LLMSpellConfig::load_builtin_profile("development").unwrap();

        // Development profile should have full stdlib
        assert_eq!(config.default_engine, "lua");
        assert!(matches!(
            config.engines.lua.stdlib,
            crate::engines::StdlibLevel::All
        ));

        // Should have OpenAI and Anthropic providers configured
        assert!(config.providers.providers.contains_key("openai"));
        assert!(config.providers.providers.contains_key("anthropic"));

        let openai = config.providers.providers.get("openai").unwrap();
        assert_eq!(openai.provider_type, "openai");
        assert_eq!(openai.default_model, Some("gpt-4".to_string()));
        assert_eq!(openai.api_key_env, Some("OPENAI_API_KEY".to_string()));

        let anthropic = config.providers.providers.get("anthropic").unwrap();
        assert_eq!(anthropic.provider_type, "anthropic");
        assert_eq!(
            anthropic.default_model,
            Some("claude-3-5-sonnet-20241022".to_string())
        );
        assert_eq!(anthropic.api_key_env, Some("ANTHROPIC_API_KEY".to_string()));
    }

    #[test]
    fn test_load_builtin_profile_rag_dev() {
        let config = LLMSpellConfig::load_builtin_profile("rag-dev").unwrap();

        // RAG should be enabled
        assert!(config.rag.enabled);
        assert!(!config.rag.multi_tenant);

        // Vector storage settings for development
        assert_eq!(config.rag.vector_storage.dimensions, 384);
        assert!(matches!(
            config.rag.vector_storage.backend,
            crate::rag::VectorBackend::HNSW
        ));
        assert_eq!(config.rag.vector_storage.max_memory_mb, Some(512));

        // HNSW settings optimized for development
        assert_eq!(config.rag.vector_storage.hnsw.m, 8);
        assert_eq!(config.rag.vector_storage.hnsw.ef_construction, 50);
        assert_eq!(config.rag.vector_storage.hnsw.ef_search, 25);
        assert_eq!(config.rag.vector_storage.hnsw.max_elements, 10000);
        assert_eq!(config.rag.vector_storage.hnsw.num_threads, Some(2));

        // Embedding settings
        assert_eq!(config.rag.embedding.default_provider, "openai");
        assert!(!config.rag.embedding.cache_enabled);
        assert_eq!(config.rag.embedding.batch_size, 4);

        // Chunking settings
        assert!(matches!(
            config.rag.chunking.strategy,
            crate::rag::ChunkingStrategy::SlidingWindow
        ));
        assert_eq!(config.rag.chunking.chunk_size, 256);
        assert_eq!(config.rag.chunking.overlap, 32);

        // Cache settings
        assert!(!config.rag.cache.search_cache_enabled);
        assert!(!config.rag.cache.document_cache_enabled);
    }

    #[test]
    fn test_load_builtin_profile_providers() {
        let config = LLMSpellConfig::load_builtin_profile("providers").unwrap();

        // Should have both providers configured
        assert!(config.providers.providers.contains_key("openai"));
        assert!(config.providers.providers.contains_key("anthropic"));

        // Verify OpenAI provider settings
        let openai = config.providers.providers.get("openai").unwrap();
        assert_eq!(openai.provider_type, "openai");
        assert_eq!(openai.default_model, Some("gpt-3.5-turbo".to_string()));
        assert_eq!(openai.api_key_env, Some("OPENAI_API_KEY".to_string()));

        // Verify Anthropic provider settings
        let anthropic = config.providers.providers.get("anthropic").unwrap();
        assert_eq!(anthropic.provider_type, "anthropic");
        assert_eq!(
            anthropic.default_model,
            Some("claude-3-haiku-20240307".to_string())
        );
        assert_eq!(anthropic.api_key_env, Some("ANTHROPIC_API_KEY".to_string()));

        // Verify default provider
        assert_eq!(
            config.providers.default_provider,
            Some("openai".to_string())
        );

        // Verify RAG/sessions disabled (state is enabled by default)
        assert!(!config.rag.enabled);
        assert!(!config.runtime.sessions.enabled);

        // State persistence uses default (enabled=true with memory backend)
        assert!(config.runtime.state_persistence.enabled);
        assert_eq!(config.runtime.state_persistence.backend_type, "memory");
    }

    #[test]
    fn test_load_builtin_profile_state() {
        let config = LLMSpellConfig::load_builtin_profile("state").unwrap();

        // Verify state persistence enabled
        assert!(config.runtime.state_persistence.enabled);
        assert_eq!(config.runtime.state_persistence.backend_type, "memory");
        assert_eq!(
            config.runtime.state_persistence.max_state_size_bytes,
            Some(10_000_000)
        );

        // Verify migration and backup disabled
        assert!(!config.runtime.state_persistence.migration_enabled);
        assert!(!config.runtime.state_persistence.backup_enabled);

        // Verify no providers configured
        assert!(config.providers.providers.is_empty());

        // Verify sessions and RAG disabled
        assert!(!config.runtime.sessions.enabled);
        assert!(!config.rag.enabled);
    }

    #[test]
    fn test_load_builtin_profile_sessions() {
        let config = LLMSpellConfig::load_builtin_profile("sessions").unwrap();

        // Verify all 4 features enabled
        assert!(config.runtime.state_persistence.enabled);
        assert!(config.runtime.sessions.enabled);
        assert_eq!(
            config.runtime.state_persistence.backend_type,
            "memory".to_string()
        );

        // Verify session limits
        assert_eq!(config.runtime.sessions.max_sessions, 100);
        assert_eq!(config.runtime.sessions.max_artifacts_per_session, 1000);
        assert_eq!(config.runtime.sessions.session_timeout_seconds, 3600);
        assert_eq!(
            config.runtime.sessions.storage_backend,
            "memory".to_string()
        );

        // Verify events enabled
        assert!(config.events.enabled);
        assert_eq!(config.events.buffer_size, 1000);

        // Verify hooks enabled (if hook config exists)
        // Note: hooks may be optional in some configs

        // Verify no providers by default
        assert!(config.providers.providers.is_empty());

        // Verify RAG disabled
        assert!(!config.rag.enabled);
    }

    #[test]
    fn test_load_builtin_profile_default() {
        let config = LLMSpellConfig::load_builtin_profile("default").unwrap();

        // Verify default provider exists and is configured
        assert_eq!(
            config.providers.default_provider,
            Some("default".to_string())
        );
        assert!(config.providers.providers.contains_key("default"));

        let default_provider = config.providers.providers.get("default").unwrap();
        assert_eq!(default_provider.provider_type, "ollama");
        assert_eq!(
            default_provider.default_model,
            Some("llama3.2:3b".to_string())
        );
        assert_eq!(default_provider.temperature, Some(0.7));
        assert_eq!(default_provider.max_tokens, Some(4096));
        assert_eq!(default_provider.timeout_seconds, Some(30));
        assert_eq!(default_provider.max_retries, Some(3));

        // Verify Lua stdlib is All
        assert!(matches!(
            config.engines.lua.stdlib,
            crate::engines::StdlibLevel::All
        ));

        // Verify memory disabled by default
        assert!(!config.runtime.memory.enabled);
    }

    #[test]
    fn test_load_builtin_profile_memory() {
        let config = LLMSpellConfig::load_builtin_profile("memory").unwrap();

        // Verify two providers exist
        assert_eq!(
            config.providers.default_provider,
            Some("default".to_string())
        );
        assert!(config.providers.providers.contains_key("default"));
        assert!(config.providers.providers.contains_key("consolidation-llm"));

        // Verify default provider config
        let default_provider = config.providers.providers.get("default").unwrap();
        assert_eq!(default_provider.provider_type, "ollama");
        assert_eq!(
            default_provider.default_model,
            Some("llama3.2:3b".to_string())
        );
        assert_eq!(default_provider.temperature, Some(0.7));
        assert_eq!(default_provider.max_tokens, Some(4096));

        // Verify consolidation provider config (low temperature for deterministic consolidation)
        let consolidation_provider = config.providers.providers.get("consolidation-llm").unwrap();
        assert_eq!(consolidation_provider.provider_type, "ollama");
        assert_eq!(
            consolidation_provider.default_model,
            Some("llama3.2:3b".to_string())
        );
        assert_eq!(consolidation_provider.temperature, Some(0.0));
        assert_eq!(consolidation_provider.max_tokens, Some(2000));

        // Verify memory system enabled
        assert!(config.runtime.memory.enabled);

        // Verify consolidation config
        assert_eq!(
            config.runtime.memory.consolidation.provider_name,
            Some("consolidation-llm".to_string())
        );
        assert_eq!(config.runtime.memory.consolidation.batch_size, 10);
        assert_eq!(config.runtime.memory.consolidation.max_concurrent, 3);
        assert_eq!(
            config
                .runtime
                .memory
                .consolidation
                .active_session_threshold_secs,
            300
        );

        // Verify daemon config
        assert!(config.runtime.memory.daemon.enabled);
        assert_eq!(config.runtime.memory.daemon.fast_interval_secs, 30);
        assert_eq!(config.runtime.memory.daemon.normal_interval_secs, 300);
        assert_eq!(config.runtime.memory.daemon.slow_interval_secs, 600);
        assert_eq!(config.runtime.memory.daemon.queue_threshold_fast, 5);
        assert_eq!(config.runtime.memory.daemon.queue_threshold_slow, 20);
        assert_eq!(config.runtime.memory.daemon.shutdown_max_wait_secs, 30);
        assert_eq!(config.runtime.memory.daemon.health_check_interval_secs, 60);
    }

    #[test]
    fn test_load_builtin_profile_unknown() {
        let result = LLMSpellConfig::load_builtin_profile("nonexistent");

        assert!(result.is_err());
        match result {
            Err(ConfigError::NotFound { path, message }) => {
                assert_eq!(path, "builtin:nonexistent");
                assert!(message.contains("Unknown builtin profile"));
                assert!(message.contains("Available profiles"));
                assert!(message.contains("minimal"));
                assert!(message.contains("ollama"));
                assert!(message.contains("rag-prod"));
            }
            _ => panic!("Expected NotFound error"),
        }
    }

    #[tokio::test]
    async fn test_load_with_profile_precedence() {
        // Test that profile takes precedence over discovery
        let config = LLMSpellConfig::load_with_profile(None, Some("minimal"))
            .await
            .unwrap();

        // Should have minimal profile settings
        assert!(matches!(
            config.engines.lua.stdlib,
            crate::engines::StdlibLevel::Basic
        ));

        // Test that discovery works when no profile specified
        let config2 = LLMSpellConfig::load_with_profile(None, None).await.unwrap();

        // Should use defaults if no config file found
        assert_eq!(config2.default_engine, "lua");
    }
}
