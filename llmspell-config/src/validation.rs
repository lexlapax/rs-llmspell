//! ABOUTME: Configuration validation logic for llmspell
//! ABOUTME: Validates configuration consistency and security requirements

use crate::{ConfigError, LLMSpellConfig};
use std::path::Path;
use tracing::{debug, warn};

/// Validate the entire configuration
pub fn validate_config(config: &LLMSpellConfig) -> Result<(), ConfigError> {
    debug!("Starting configuration validation");

    // Validate basic configuration
    validate_basic_config(config)?;

    // Validate engine configuration
    validate_engine_config(config)?;

    // Validate provider configuration
    validate_provider_config(config)?;

    // Validate tools configuration
    validate_tools_config(config)?;

    // Validate runtime configuration
    validate_runtime_config(config)?;

    // Validate events configuration
    validate_events_config(config)?;

    debug!("Configuration validation completed successfully");
    Ok(())
}

/// Validate basic configuration requirements
fn validate_basic_config(config: &LLMSpellConfig) -> Result<(), ConfigError> {
    // Validate default engine
    if config.default_engine.is_empty() {
        return Err(ConfigError::Validation {
            field: Some("default_engine".to_string()),
            message: "Default engine cannot be empty".to_string(),
        });
    }

    // Check if default engine is supported
    if !config.supports_engine(&config.default_engine) {
        return Err(ConfigError::Validation {
            field: Some("default_engine".to_string()),
            message: format!(
                "Default engine '{}' is not configured",
                config.default_engine
            ),
        });
    }

    Ok(())
}

/// Validate engine configuration
fn validate_engine_config(config: &LLMSpellConfig) -> Result<(), ConfigError> {
    // Validate Lua configuration
    if let Some(max_memory) = config.engines.lua.max_memory_bytes {
        if max_memory == 0 {
            return Err(ConfigError::Validation {
                field: Some("engines.lua.max_memory_bytes".to_string()),
                message: "Lua max memory cannot be zero".to_string(),
            });
        }

        if max_memory > 1_000_000_000 {
            // 1GB
            warn!("Lua max memory is very high: {} bytes", max_memory);
        }
    }

    if let Some(timeout) = config.engines.lua.timeout_ms {
        if timeout == 0 {
            return Err(ConfigError::Validation {
                field: Some("engines.lua.timeout_ms".to_string()),
                message: "Lua timeout cannot be zero".to_string(),
            });
        }

        if timeout > 300_000 {
            // 5 minutes
            warn!("Lua timeout is very high: {} ms", timeout);
        }
    }

    // Validate JavaScript configuration
    if let Some(max_heap) = config.engines.javascript.max_heap_size_bytes {
        if max_heap == 0 {
            return Err(ConfigError::Validation {
                field: Some("engines.javascript.max_heap_size_bytes".to_string()),
                message: "JavaScript max heap size cannot be zero".to_string(),
            });
        }

        if max_heap > 1_000_000_000 {
            // 1GB
            warn!("JavaScript max heap size is very high: {} bytes", max_heap);
        }
    }

    if let Some(timeout) = config.engines.javascript.timeout_ms {
        if timeout == 0 {
            return Err(ConfigError::Validation {
                field: Some("engines.javascript.timeout_ms".to_string()),
                message: "JavaScript timeout cannot be zero".to_string(),
            });
        }
    }

    Ok(())
}

/// Validate provider configuration
fn validate_provider_config(config: &LLMSpellConfig) -> Result<(), ConfigError> {
    // Check if default provider exists
    if let Some(default_provider) = &config.providers.default_provider {
        if !config.providers.providers.contains_key(default_provider) {
            return Err(ConfigError::Validation {
                field: Some("providers.default_provider".to_string()),
                message: format!("Default provider '{}' is not configured", default_provider),
            });
        }
    }

    // Validate individual provider configurations
    for (name, provider_config) in &config.providers.providers {
        if provider_config.provider_type.is_empty() {
            return Err(ConfigError::Validation {
                field: Some(format!("providers.{}.provider_type", name)),
                message: "Provider type cannot be empty".to_string(),
            });
        }

        // Check credentials configuration
        if !provider_config.has_credentials() {
            warn!("Provider '{}' has no credentials configured", name);
        }

        // Validate timeout
        if let Some(timeout) = provider_config.timeout_seconds {
            if timeout == 0 {
                return Err(ConfigError::Validation {
                    field: Some(format!("providers.{}.timeout_seconds", name)),
                    message: "Provider timeout cannot be zero".to_string(),
                });
            }

            if timeout > 600 {
                // 10 minutes
                warn!(
                    "Provider '{}' timeout is very high: {} seconds",
                    name, timeout
                );
            }
        }

        // Validate max tokens
        if let Some(max_tokens) = provider_config.max_tokens {
            if max_tokens == 0 {
                return Err(ConfigError::Validation {
                    field: Some(format!("providers.providers.{}.max_tokens", name)),
                    message: "Provider max tokens cannot be zero".to_string(),
                });
            }
        }

        // Validate rate limiting
        if let Some(rate_limit) = &provider_config.rate_limit {
            if rate_limit.requests_per_minute == 0 {
                return Err(ConfigError::Validation {
                    field: Some(format!(
                        "providers.providers.{}.rate_limit.requests_per_minute",
                        name
                    )),
                    message: "Rate limit requests per minute cannot be zero".to_string(),
                });
            }
        }

        // Validate retry configuration
        if let Some(retry) = &provider_config.retry {
            if retry.max_retries > 10 {
                warn!(
                    "Provider '{}' has high retry count: {}",
                    name, retry.max_retries
                );
            }

            if retry.backoff_multiplier <= 0.0 {
                return Err(ConfigError::Validation {
                    field: Some(format!(
                        "providers.providers.{}.retry.backoff_multiplier",
                        name
                    )),
                    message: "Backoff multiplier must be positive".to_string(),
                });
            }
        }
    }

    Ok(())
}

/// Validate tools configuration
fn validate_tools_config(config: &LLMSpellConfig) -> Result<(), ConfigError> {
    // Validate file operations configuration
    let file_ops = &config.tools.file_operations;
    if file_ops.allowed_paths.is_empty() {
        return Err(ConfigError::Validation {
            field: Some("tools.file_operations.allowed_paths".to_string()),
            message: "File operations must have at least one allowed path".to_string(),
        });
    }

    // Validate allowed paths exist (for non-wildcard paths)
    for path in &file_ops.allowed_paths {
        if path != "*" && !Path::new(path).exists() {
            warn!("Allowed path does not exist: {}", path);
        }
    }

    if file_ops.max_file_size == 0 {
        return Err(ConfigError::Validation {
            field: Some("tools.file_operations.max_file_size".to_string()),
            message: "Max file size cannot be zero".to_string(),
        });
    }

    if file_ops.max_file_size > 1_000_000_000 {
        // 1GB
        warn!(
            "File operations max file size is very high: {} bytes",
            file_ops.max_file_size
        );
    }

    if let Some(max_depth) = file_ops.max_depth {
        if max_depth == 0 {
            return Err(ConfigError::Validation {
                field: Some("tools.file_operations.max_depth".to_string()),
                message: "Max depth cannot be zero".to_string(),
            });
        }

        if max_depth > 50 {
            warn!("File operations max depth is very high: {}", max_depth);
        }
    }

    // Validate web search configuration
    let web_search = &config.tools.web_search;
    if web_search.rate_limit_per_minute == 0 {
        return Err(ConfigError::Validation {
            field: Some("tools.web_search.rate_limit_per_minute".to_string()),
            message: "Web search rate limit cannot be zero".to_string(),
        });
    }

    if web_search.max_results == 0 {
        return Err(ConfigError::Validation {
            field: Some("tools.web_search.max_results".to_string()),
            message: "Web search max results cannot be zero".to_string(),
        });
    }

    if web_search.max_results > 100 {
        warn!(
            "Web search max results is very high: {}",
            web_search.max_results
        );
    }

    if web_search.timeout_seconds == 0 {
        return Err(ConfigError::Validation {
            field: Some("tools.web_search.timeout_seconds".to_string()),
            message: "Web search timeout cannot be zero".to_string(),
        });
    }

    // Validate HTTP request configuration
    let http_req = &config.tools.http_request;
    if http_req.max_request_size == 0 {
        return Err(ConfigError::Validation {
            field: Some("tools.http_request.max_request_size".to_string()),
            message: "HTTP request max size cannot be zero".to_string(),
        });
    }

    if http_req.timeout_seconds == 0 {
        return Err(ConfigError::Validation {
            field: Some("tools.http_request.timeout_seconds".to_string()),
            message: "HTTP request timeout cannot be zero".to_string(),
        });
    }

    Ok(())
}

/// Validate runtime configuration
fn validate_runtime_config(config: &LLMSpellConfig) -> Result<(), ConfigError> {
    let runtime = &config.runtime;

    // Validate concurrent scripts
    if runtime.max_concurrent_scripts == 0 {
        return Err(ConfigError::Validation {
            field: Some("runtime.max_concurrent_scripts".to_string()),
            message: "Max concurrent scripts cannot be zero".to_string(),
        });
    }

    if runtime.max_concurrent_scripts > 100 {
        warn!(
            "Max concurrent scripts is very high: {}",
            runtime.max_concurrent_scripts
        );
    }

    // Validate script timeout
    if runtime.script_timeout_seconds == 0 {
        return Err(ConfigError::Validation {
            field: Some("runtime.script_timeout_seconds".to_string()),
            message: "Script timeout cannot be zero".to_string(),
        });
    }

    if runtime.script_timeout_seconds > 3600 {
        // 1 hour
        warn!(
            "Script timeout is very high: {} seconds",
            runtime.script_timeout_seconds
        );
    }

    // Validate security settings
    let security = &runtime.security;

    if let Some(max_memory) = security.max_memory_bytes {
        if max_memory == 0 {
            return Err(ConfigError::Validation {
                field: Some("runtime.security.max_memory_bytes".to_string()),
                message: "Security max memory cannot be zero".to_string(),
            });
        }

        if max_memory > 2_000_000_000 {
            // 2GB
            warn!("Security max memory is very high: {} bytes", max_memory);
        }
    }

    if let Some(max_exec_time) = security.max_execution_time_ms {
        if max_exec_time == 0 {
            return Err(ConfigError::Validation {
                field: Some("runtime.security.max_execution_time_ms".to_string()),
                message: "Security max execution time cannot be zero".to_string(),
            });
        }
    }

    // Validate state persistence settings
    let state = &runtime.state_persistence;

    if let Some(max_state_size) = state.max_state_size_bytes {
        if max_state_size == 0 {
            return Err(ConfigError::Validation {
                field: Some("runtime.state_persistence.max_state_size_bytes".to_string()),
                message: "Max state size cannot be zero".to_string(),
            });
        }

        if max_state_size > 100_000_000 {
            // 100MB
            warn!(
                "Max state size per key is very high: {} bytes",
                max_state_size
            );
        }
    }

    // Validate backup configuration
    if let Some(backup) = &state.backup {
        if let Some(max_backups) = backup.max_backups {
            if max_backups == 0 {
                return Err(ConfigError::Validation {
                    field: Some("runtime.state_persistence.backup.max_backups".to_string()),
                    message: "Max backups cannot be zero".to_string(),
                });
            }
        }

        if backup.compression_level > 9 {
            return Err(ConfigError::Validation {
                field: Some("runtime.state_persistence.backup.compression_level".to_string()),
                message: "Compression level must be between 1 and 9".to_string(),
            });
        }

        if backup.compression_level == 0 {
            return Err(ConfigError::Validation {
                field: Some("runtime.state_persistence.backup.compression_level".to_string()),
                message: "Compression level cannot be zero".to_string(),
            });
        }
    }

    // Validate session settings
    let sessions = &runtime.sessions;

    if sessions.max_sessions == 0 {
        return Err(ConfigError::Validation {
            field: Some("runtime.sessions.max_sessions".to_string()),
            message: "Max sessions cannot be zero".to_string(),
        });
    }

    if sessions.max_artifacts_per_session == 0 {
        return Err(ConfigError::Validation {
            field: Some("runtime.sessions.max_artifacts_per_session".to_string()),
            message: "Max artifacts per session cannot be zero".to_string(),
        });
    }

    if sessions.session_timeout_seconds == 0 {
        return Err(ConfigError::Validation {
            field: Some("runtime.sessions.session_timeout_seconds".to_string()),
            message: "Session timeout cannot be zero".to_string(),
        });
    }

    Ok(())
}

/// Validate events configuration
fn validate_events_config(config: &LLMSpellConfig) -> Result<(), ConfigError> {
    let events = &config.events;

    // Validate buffer size
    if events.buffer_size == 0 {
        return Err(ConfigError::Validation {
            field: Some("events.buffer_size".to_string()),
            message: "Events buffer size cannot be zero".to_string(),
        });
    }

    if events.buffer_size > 1_000_000 {
        warn!(
            "Events buffer size is very high: {} - this may consume significant memory",
            events.buffer_size
        );
    }

    // Validate rate limiting
    if let Some(max_events_per_second) = events.max_events_per_second {
        if max_events_per_second == 0 {
            return Err(ConfigError::Validation {
                field: Some("events.max_events_per_second".to_string()),
                message: "Events max events per second cannot be zero".to_string(),
            });
        }

        if max_events_per_second > 100_000 {
            warn!(
                "Events max events per second is very high: {} - this may impact performance",
                max_events_per_second
            );
        }
    }

    // Validate filtering configuration
    let filtering = &events.filtering;

    // Check for conflicting include/exclude patterns
    for include_pattern in &filtering.include_types {
        for exclude_pattern in &filtering.exclude_types {
            if include_pattern == exclude_pattern {
                return Err(ConfigError::Validation {
                    field: Some("events.filtering".to_string()),
                    message: format!(
                        "Event type pattern '{}' is both included and excluded",
                        include_pattern
                    ),
                });
            }
        }
    }

    for include_pattern in &filtering.include_components {
        for exclude_pattern in &filtering.exclude_components {
            if include_pattern == exclude_pattern {
                return Err(ConfigError::Validation {
                    field: Some("events.filtering".to_string()),
                    message: format!(
                        "Component pattern '{}' is both included and excluded",
                        include_pattern
                    ),
                });
            }
        }
    }

    // Warn if all event types are disabled
    if !events.emit_timing_events && !events.emit_state_events && !events.emit_debug_events {
        warn!("All event types are disabled - event system will emit no events");
    }

    // Validate export configuration
    let export = &events.export;

    // Check if any export method is configured when events are enabled
    if events.enabled && !export.stdout && export.file.is_none() && export.webhook.is_none() {
        warn!("Events are enabled but no export method is configured - events will be generated but not output");
    }

    // Validate file path if specified
    if let Some(file_path) = &export.file {
        if file_path.is_empty() {
            return Err(ConfigError::Validation {
                field: Some("events.export.file".to_string()),
                message: "Event export file path cannot be empty".to_string(),
            });
        }

        // Check if parent directory exists
        if let Some(parent) = std::path::Path::new(file_path).parent() {
            if !parent.exists() {
                warn!(
                    "Event export file parent directory does not exist: {}",
                    parent.display()
                );
            }
        }
    }

    // Validate webhook URL if specified
    if let Some(webhook_url) = &export.webhook {
        if webhook_url.is_empty() {
            return Err(ConfigError::Validation {
                field: Some("events.export.webhook".to_string()),
                message: "Event export webhook URL cannot be empty".to_string(),
            });
        }

        // Basic URL validation
        if !webhook_url.starts_with("http://") && !webhook_url.starts_with("https://") {
            return Err(ConfigError::Validation {
                field: Some("events.export.webhook".to_string()),
                message: "Event export webhook URL must start with http:// or https://".to_string(),
            });
        }

        // Warn about http (non-secure) webhooks
        if webhook_url.starts_with("http://")
            && !webhook_url.contains("localhost")
            && !webhook_url.contains("127.0.0.1")
        {
            warn!(
                "Event export webhook uses insecure HTTP: {} - consider using HTTPS",
                webhook_url
            );
        }
    }

    Ok(())
}

/// Validate security configuration for risky settings
pub fn validate_security_requirements(config: &LLMSpellConfig) -> Result<(), ConfigError> {
    debug!("Validating security requirements");

    // Check for potentially unsafe configurations
    if config.runtime.security.allow_process_spawn {
        warn!("Process spawning is enabled - this may be a security risk");
    }

    if config.runtime.security.allow_file_access {
        // Check if file access is overly permissive
        if config
            .tools
            .file_operations
            .allowed_paths
            .contains(&"*".to_string())
        {
            warn!("File access allows all paths (*) - this may be a security risk");
        }

        // Check for sensitive paths
        for path in &config.tools.file_operations.allowed_paths {
            if path.starts_with("/etc") || path.starts_with("/root") || path.starts_with("/sys") {
                warn!(
                    "File access includes sensitive path: {} - this may be a security risk",
                    path
                );
            }
        }
    }

    // Check for overly permissive network access
    if config
        .tools
        .web_search
        .allowed_domains
        .contains(&"*".to_string())
        && config.tools.web_search.blocked_domains.is_empty()
    {
        warn!("Web search allows all domains with no blocked list - consider adding restrictions");
    }

    if config
        .tools
        .http_request
        .allowed_hosts
        .contains(&"*".to_string())
    {
        // Check if localhost/internal IPs are properly blocked
        let has_localhost_blocks = config.tools.http_request.blocked_hosts.iter().any(|host| {
            host.contains("localhost") || host.contains("127.0.0.1") || host.contains("0.0.0.0")
        });

        if !has_localhost_blocks {
            warn!("HTTP requests allow all hosts but don't block localhost - this may be a security risk");
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::providers::ProviderConfig;

    #[test]
    fn test_validate_basic_config_empty_engine() {
        let config = LLMSpellConfig {
            default_engine: String::new(),
            ..Default::default()
        };

        let result = validate_basic_config(&config);
        assert!(result.is_err());

        if let Err(ConfigError::Validation { field, message }) = result {
            assert_eq!(field, Some("default_engine".to_string()));
            assert!(message.contains("cannot be empty"));
        }
    }

    #[test]
    fn test_validate_basic_config_unsupported_engine() {
        let config = LLMSpellConfig {
            default_engine: "unsupported".to_string(),
            ..Default::default()
        };

        let result = validate_basic_config(&config);
        assert!(result.is_err());

        if let Err(ConfigError::Validation { field, message }) = result {
            assert_eq!(field, Some("default_engine".to_string()));
            assert!(message.contains("not configured"));
        }
    }

    #[test]
    fn test_validate_engine_config_zero_memory() {
        let mut config = LLMSpellConfig::default();
        config.engines.lua.max_memory_bytes = Some(0);

        let result = validate_engine_config(&config);
        assert!(result.is_err());

        if let Err(ConfigError::Validation { field, .. }) = result {
            assert_eq!(field, Some("engines.lua.max_memory_bytes".to_string()));
        }
    }

    #[test]
    fn test_validate_provider_config_missing_default() {
        let mut config = LLMSpellConfig::default();
        config.providers.default_provider = Some("nonexistent".to_string());

        let result = validate_provider_config(&config);
        assert!(result.is_err());

        if let Err(ConfigError::Validation { field, message }) = result {
            assert_eq!(field, Some("providers.default_provider".to_string()));
            assert!(message.contains("not configured"));
        }
    }

    #[test]
    fn test_validate_tools_config_empty_paths() {
        let mut config = LLMSpellConfig::default();
        config.tools.file_operations.allowed_paths.clear();

        let result = validate_tools_config(&config);
        assert!(result.is_err());

        if let Err(ConfigError::Validation { field, .. }) = result {
            assert_eq!(
                field,
                Some("tools.file_operations.allowed_paths".to_string())
            );
        }
    }

    #[test]
    fn test_validate_runtime_config_zero_values() {
        let config = LLMSpellConfig {
            runtime: crate::GlobalRuntimeConfig {
                max_concurrent_scripts: 0,
                ..Default::default()
            },
            ..Default::default()
        };

        let result = validate_runtime_config(&config);
        assert!(result.is_err());

        if let Err(ConfigError::Validation { field, .. }) = result {
            assert_eq!(field, Some("runtime.max_concurrent_scripts".to_string()));
        }
    }

    #[test]
    fn test_validate_security_requirements_warnings() {
        let config = LLMSpellConfig {
            runtime: crate::GlobalRuntimeConfig {
                security: crate::SecurityConfig {
                    allow_process_spawn: true,
                    ..Default::default()
                },
                ..Default::default()
            },
            tools: crate::ToolsConfig {
                file_operations: crate::FileOperationsConfig {
                    allowed_paths: vec!["*".to_string()],
                    ..Default::default()
                },
                ..Default::default()
            },
            ..Default::default()
        };

        // This should pass validation but generate warnings
        let result = validate_security_requirements(&config);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_config_success() {
        let config = LLMSpellConfig::default();
        let result = validate_config(&config);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_provider_config_credentials() {
        let provider_config = ProviderConfig::builder().provider_type("openai").build();

        let mut config = LLMSpellConfig::default();
        config
            .providers
            .providers
            .insert("openai".to_string(), provider_config);

        // Should pass validation but may generate warnings
        let result = validate_provider_config(&config);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_backup_config_compression_level() {
        let config = LLMSpellConfig {
            runtime: crate::GlobalRuntimeConfig {
                state_persistence: crate::StatePersistenceConfig {
                    backup: Some(crate::BackupConfig {
                        backup_dir: Some("./backups".to_string()),
                        compression_enabled: true,
                        compression_type: "zstd".to_string(),
                        compression_level: 10, // Invalid - must be 1-9
                        incremental_enabled: true,
                        max_backups: Some(10),
                        max_backup_age: Some(2_592_000),
                    }),
                    ..Default::default()
                },
                ..Default::default()
            },
            ..Default::default()
        };

        let result = validate_runtime_config(&config);
        assert!(result.is_err());

        if let Err(ConfigError::Validation { field, message }) = result {
            assert!(field.unwrap().contains("compression_level"));
            assert!(message.contains("between 1 and 9"));
        }
    }

    #[test]
    fn test_validate_events_config_zero_buffer_size() {
        let config = LLMSpellConfig {
            events: crate::EventsConfig {
                buffer_size: 0,
                ..Default::default()
            },
            ..Default::default()
        };

        let result = validate_events_config(&config);
        assert!(result.is_err());

        if let Err(ConfigError::Validation { field, message }) = result {
            assert_eq!(field, Some("events.buffer_size".to_string()));
            assert!(message.contains("cannot be zero"));
        }
    }

    #[test]
    fn test_validate_events_config_zero_rate_limit() {
        let config = LLMSpellConfig {
            events: crate::EventsConfig {
                max_events_per_second: Some(0),
                ..Default::default()
            },
            ..Default::default()
        };

        let result = validate_events_config(&config);
        assert!(result.is_err());

        if let Err(ConfigError::Validation { field, message }) = result {
            assert_eq!(field, Some("events.max_events_per_second".to_string()));
            assert!(message.contains("cannot be zero"));
        }
    }

    #[test]
    fn test_validate_events_config_conflicting_filters() {
        let config = LLMSpellConfig {
            events: crate::EventsConfig {
                filtering: crate::EventFilterConfig {
                    include_types: vec!["workflow.*".to_string()],
                    exclude_types: vec!["workflow.*".to_string()],
                    ..Default::default()
                },
                ..Default::default()
            },
            ..Default::default()
        };

        let result = validate_events_config(&config);
        assert!(result.is_err());

        if let Err(ConfigError::Validation { field, message }) = result {
            assert_eq!(field, Some("events.filtering".to_string()));
            assert!(message.contains("both included and excluded"));
        }
    }

    #[test]
    fn test_validate_events_config_conflicting_component_filters() {
        let config = LLMSpellConfig {
            events: crate::EventsConfig {
                filtering: crate::EventFilterConfig {
                    include_components: vec!["agent-*".to_string()],
                    exclude_components: vec!["agent-*".to_string()],
                    ..Default::default()
                },
                ..Default::default()
            },
            ..Default::default()
        };

        let result = validate_events_config(&config);
        assert!(result.is_err());

        if let Err(ConfigError::Validation { field, message }) = result {
            assert_eq!(field, Some("events.filtering".to_string()));
            assert!(message.contains("both included and excluded"));
        }
    }

    #[test]
    fn test_validate_events_config_empty_file_path() {
        let config = LLMSpellConfig {
            events: crate::EventsConfig {
                export: crate::EventExportConfig {
                    file: Some(String::new()),
                    ..Default::default()
                },
                ..Default::default()
            },
            ..Default::default()
        };

        let result = validate_events_config(&config);
        assert!(result.is_err());

        if let Err(ConfigError::Validation { field, message }) = result {
            assert_eq!(field, Some("events.export.file".to_string()));
            assert!(message.contains("cannot be empty"));
        }
    }

    #[test]
    fn test_validate_events_config_empty_webhook_url() {
        let config = LLMSpellConfig {
            events: crate::EventsConfig {
                export: crate::EventExportConfig {
                    webhook: Some(String::new()),
                    ..Default::default()
                },
                ..Default::default()
            },
            ..Default::default()
        };

        let result = validate_events_config(&config);
        assert!(result.is_err());

        if let Err(ConfigError::Validation { field, message }) = result {
            assert_eq!(field, Some("events.export.webhook".to_string()));
            assert!(message.contains("cannot be empty"));
        }
    }

    #[test]
    fn test_validate_events_config_invalid_webhook_url() {
        let config = LLMSpellConfig {
            events: crate::EventsConfig {
                export: crate::EventExportConfig {
                    webhook: Some("invalid-url".to_string()),
                    ..Default::default()
                },
                ..Default::default()
            },
            ..Default::default()
        };

        let result = validate_events_config(&config);
        assert!(result.is_err());

        if let Err(ConfigError::Validation { field, message }) = result {
            assert_eq!(field, Some("events.export.webhook".to_string()));
            assert!(message.contains("must start with http:// or https://"));
        }
    }

    #[test]
    fn test_validate_events_config_valid_webhook_urls() {
        let valid_urls = vec![
            "https://example.com/webhook",
            "http://localhost:8080/events",
            "http://127.0.0.1:3000/webhook",
        ];

        for url in valid_urls {
            let config = LLMSpellConfig {
                events: crate::EventsConfig {
                    export: crate::EventExportConfig {
                        webhook: Some(url.to_string()),
                        ..Default::default()
                    },
                    ..Default::default()
                },
                ..Default::default()
            };

            let result = validate_events_config(&config);
            assert!(result.is_ok(), "URL {} should be valid", url);
        }
    }

    #[test]
    fn test_validate_events_config_success() {
        let config = LLMSpellConfig {
            events: crate::EventsConfig {
                enabled: true,
                buffer_size: 10000,
                max_events_per_second: Some(1000),
                export: crate::EventExportConfig {
                    stdout: true,
                    ..Default::default()
                },
                ..Default::default()
            },
            ..Default::default()
        };

        let result = validate_events_config(&config);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_full_config_with_events() {
        let config = LLMSpellConfig {
            events: crate::EventsConfig {
                enabled: true,
                buffer_size: 10000,
                export: crate::EventExportConfig {
                    stdout: true,
                    ..Default::default()
                },
                ..Default::default()
            },
            ..Default::default()
        };

        let result = validate_config(&config);
        assert!(result.is_ok());
    }
}
