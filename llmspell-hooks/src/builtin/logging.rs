// ABOUTME: LoggingHook implementation for automatic hook execution logging
// ABOUTME: Provides configurable logging levels and structured logging for all hook points

use crate::context::HookContext;
use crate::result::HookResult;
use crate::traits::{Hook, MetricHook, ReplayableHook};
use crate::types::{HookMetadata, Language, Priority};
use anyhow::Result;
use async_trait::async_trait;
use chrono::Utc;
use log::{debug, error, info, trace, warn};
use serde::{Deserialize, Serialize};

// OPTIMIZATION: Pre-allocate common strings to reduce allocations
const BUILTIN_TAG: &str = "builtin";
const LOGGING_TAG: &str = "logging";
const HOOK_NAME: &str = "logging_hook";
const HOOK_DESCRIPTION: &str = "Built-in hook for logging hook execution events";
const HOOK_VERSION: &str = "1.0.0";
const EMPTY_JSON: &str = "{}";
const SERIALIZATION_ERROR: &str = "{serialization_error}";

/// Configuration for the logging hook
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    /// Minimum log level to capture
    pub level: LogLevel,
    /// Whether to include detailed context data
    pub include_context_data: bool,
    /// Whether to include metadata
    pub include_metadata: bool,
    /// Whether to log performance metrics
    pub log_performance: bool,
    /// Maximum data size to log (bytes)
    pub max_data_size: usize,
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            level: LogLevel::Info,
            include_context_data: true,
            include_metadata: true,
            log_performance: true,
            max_data_size: 1024, // 1KB default limit
        }
    }
}

/// Log levels for the logging hook
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum LogLevel {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
}

/// Built-in logging hook for comprehensive hook execution logging
pub struct LoggingHook {
    config: LoggingConfig,
    metadata: HookMetadata,
}

impl LoggingHook {
    /// Create a new logging hook with default configuration
    pub fn new() -> Self {
        Self {
            config: LoggingConfig::default(),
            metadata: HookMetadata {
                // OPTIMIZATION: Use pre-allocated constants to reduce allocations
                name: HOOK_NAME.to_owned(),
                description: Some(HOOK_DESCRIPTION.to_owned()),
                priority: Priority::LOW, // Run after other hooks
                language: Language::Native,
                tags: vec![BUILTIN_TAG.to_owned(), LOGGING_TAG.to_owned()],
                version: HOOK_VERSION.to_owned(),
            },
        }
    }

    /// Create a new logging hook with custom configuration
    pub fn with_config(config: LoggingConfig) -> Self {
        Self {
            config,
            metadata: HookMetadata {
                name: "LoggingHook".to_string(),
                description: Some("Built-in hook for logging hook execution".to_string()),
                priority: Priority::LOW,
                language: Language::Native,
                tags: vec!["builtin".to_string(), "logging".to_string()],
                version: "1.0.0".to_string(),
            },
        }
    }

    /// Set the log level
    pub fn with_level(mut self, level: LogLevel) -> Self {
        self.config.level = level;
        self
    }

    /// Enable or disable context data logging
    pub fn with_context_data(mut self, include: bool) -> Self {
        self.config.include_context_data = include;
        self
    }

    /// Enable or disable performance logging
    pub fn with_performance_logging(mut self, enable: bool) -> Self {
        self.config.log_performance = enable;
        self
    }

    /// Set maximum data size to log
    pub fn with_max_data_size(mut self, size: usize) -> Self {
        self.config.max_data_size = size;
        self
    }

    /// Truncate data if it exceeds the maximum size
    /// OPTIMIZATION: Use Cow to avoid unnecessary allocations
    fn truncate_data<'a>(&self, data: &'a str) -> std::borrow::Cow<'a, str> {
        if data.len() > self.config.max_data_size {
            std::borrow::Cow::Owned(format!(
                "{}... (truncated)",
                &data[..self.config.max_data_size]
            ))
        } else {
            std::borrow::Cow::Borrowed(data)
        }
    }

    /// Format context data as a log-friendly string
    /// OPTIMIZATION: Use constants and Cow to reduce allocations
    fn format_context_data(&self, context: &HookContext) -> String {
        if !self.config.include_context_data || context.data.is_empty() {
            return EMPTY_JSON.to_owned();
        }

        match serde_json::to_string(&context.data) {
            Ok(json) => self.truncate_data(&json).into_owned(),
            Err(_) => SERIALIZATION_ERROR.to_owned(),
        }
    }

    /// Format metadata as a log-friendly string
    /// OPTIMIZATION: Use constant for empty case
    fn format_metadata(&self, context: &HookContext) -> String {
        if !self.config.include_metadata || context.metadata.is_empty() {
            return EMPTY_JSON.to_owned();
        }

        format!("{:?}", context.metadata)
    }
}

impl Default for LoggingHook {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Hook for LoggingHook {
    async fn execute(&self, context: &mut HookContext) -> Result<HookResult> {
        let start_time = std::time::Instant::now();

        // Format log message components
        let context_data = self.format_context_data(context);
        let metadata = self.format_metadata(context);

        let base_message = format!(
            "Hook execution: point={:?}, component={:?}:{}, language={:?}, correlation_id={}",
            context.point,
            context.component_id.component_type,
            context.component_id.name,
            context.language,
            context.correlation_id
        );

        let detailed_message = if self.config.include_context_data || self.config.include_metadata {
            format!(
                "{}, data={}, metadata={}",
                base_message, context_data, metadata
            )
        } else {
            base_message
        };

        // Log based on configured level
        match self.config.level {
            LogLevel::Trace => trace!("{}", detailed_message),
            LogLevel::Debug => debug!("{}", detailed_message),
            LogLevel::Info => info!("{}", detailed_message),
            LogLevel::Warn => warn!("{}", detailed_message),
            LogLevel::Error => error!("{}", detailed_message),
        }

        // Log performance if enabled
        if self.config.log_performance {
            let duration = start_time.elapsed();
            debug!(
                "LoggingHook performance: {:?} for hook point {:?}",
                duration, context.point
            );
        }

        // Add logging metadata to context
        context.insert_metadata("logged_at".to_string(), Utc::now().to_rfc3339());
        context.insert_metadata(
            "logging_hook_version".to_string(),
            self.metadata.version.clone(),
        );

        Ok(HookResult::Continue)
    }

    fn metadata(&self) -> HookMetadata {
        self.metadata.clone()
    }

    fn should_execute(&self, _context: &HookContext) -> bool {
        // Always execute logging hook unless explicitly disabled
        true
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

#[async_trait]
impl MetricHook for LoggingHook {
    async fn record_pre_execution(&self, context: &HookContext) -> Result<()> {
        if self.config.log_performance {
            trace!("LoggingHook starting for hook point {:?}", context.point);
        }
        Ok(())
    }

    async fn record_post_execution(
        &self,
        context: &HookContext,
        result: &HookResult,
        duration: std::time::Duration,
    ) -> Result<()> {
        if self.config.log_performance {
            debug!(
                "LoggingHook completed for hook point {:?}: result={}, duration={:?}",
                context.point,
                result.description(),
                duration
            );
        }
        Ok(())
    }
}

#[async_trait]
impl ReplayableHook for LoggingHook {
    fn is_replayable(&self) -> bool {
        true
    }

    fn serialize_context(&self, ctx: &HookContext) -> Result<Vec<u8>> {
        // Create a serializable version of the context with logging config
        let mut context_data = ctx.data.clone();

        // Add logging configuration to the context for replay
        context_data.insert(
            "_logging_config".to_string(),
            serde_json::to_value(&self.config)?,
        );

        // Create a modified context with the config data
        let mut replay_context = ctx.clone();
        replay_context.data = context_data;

        Ok(serde_json::to_vec(&replay_context)?)
    }

    fn deserialize_context(&self, data: &[u8]) -> Result<HookContext> {
        let mut context: HookContext = serde_json::from_slice(data)?;

        // Remove the logging config from the context data
        // (it was only needed for serialization)
        context.data.remove("_logging_config");

        Ok(context)
    }

    fn replay_id(&self) -> String {
        format!("{}:{}", self.metadata.name, self.metadata.version)
    }
}

#[cfg(test)]
#[cfg_attr(test_category = "hook")]
mod tests {
    use super::*;
    use crate::types::{ComponentId, ComponentType, HookPoint};
    use serde_json::json;

    #[cfg_attr(test_category = "unit")]
    #[tokio::test]
    async fn test_logging_hook_basic() {
        let hook = LoggingHook::new();
        let component_id = ComponentId::new(ComponentType::System, "test".to_string());
        let mut context = HookContext::new(HookPoint::SystemStartup, component_id);

        let result = hook.execute(&mut context).await.unwrap();
        assert!(matches!(result, HookResult::Continue));

        // Check that metadata was added
        assert!(context.get_metadata("logged_at").is_some());
        assert!(context.get_metadata("logging_hook_version").is_some());
    }

    #[cfg_attr(test_category = "unit")]
    #[tokio::test]
    async fn test_logging_hook_with_data() {
        let hook = LoggingHook::new().with_context_data(true);
        let component_id = ComponentId::new(ComponentType::Agent, "test-agent".to_string());
        let mut context = HookContext::new(HookPoint::BeforeAgentExecution, component_id);

        // Add some test data
        context.insert_data("test_key".to_string(), json!("test_value"));
        context.insert_metadata("test_meta".to_string(), "meta_value".to_string());

        let result = hook.execute(&mut context).await.unwrap();
        assert!(matches!(result, HookResult::Continue));
    }

    #[cfg_attr(test_category = "unit")]
    #[tokio::test]
    async fn test_logging_hook_different_levels() {
        for level in [
            LogLevel::Trace,
            LogLevel::Debug,
            LogLevel::Info,
            LogLevel::Warn,
            LogLevel::Error,
        ] {
            let hook = LoggingHook::new().with_level(level);
            let component_id = ComponentId::new(ComponentType::Tool, "test-tool".to_string());
            let mut context = HookContext::new(HookPoint::BeforeToolExecution, component_id);

            let result = hook.execute(&mut context).await.unwrap();
            assert!(matches!(result, HookResult::Continue));
        }
    }

    #[cfg_attr(test_category = "unit")]
    #[test]
    fn test_data_truncation() {
        let hook = LoggingHook::new().with_max_data_size(10);
        let long_data = "a".repeat(20);
        let truncated = hook.truncate_data(&long_data);

        assert!(truncated.len() > 10); // Includes "... (truncated)"
        assert!(truncated.contains("... (truncated)"));
    }

    #[cfg_attr(test_category = "unit")]
    #[tokio::test]
    async fn test_performance_logging() {
        let hook = LoggingHook::new().with_performance_logging(true);
        let component_id = ComponentId::new(ComponentType::Workflow, "test-workflow".to_string());
        let mut context = HookContext::new(HookPoint::BeforeWorkflowStart, component_id);

        // Test MetricHook implementation
        hook.record_pre_execution(&context).await.unwrap();

        let result = hook.execute(&mut context).await.unwrap();
        assert!(matches!(result, HookResult::Continue));

        hook.record_post_execution(&context, &result, std::time::Duration::from_millis(10))
            .await
            .unwrap();
    }

    #[cfg_attr(test_category = "unit")]
    #[test]
    fn test_logging_config_defaults() {
        let config = LoggingConfig::default();
        assert_eq!(config.level, LogLevel::Info);
        assert!(config.include_context_data);
        assert!(config.include_metadata);
        assert!(config.log_performance);
        assert_eq!(config.max_data_size, 1024);
    }

    #[cfg_attr(test_category = "unit")]
    #[test]
    fn test_hook_metadata() {
        let hook = LoggingHook::new();
        let metadata = hook.metadata();

        assert_eq!(metadata.name, HOOK_NAME);
        assert!(metadata.description.is_some());
        assert_eq!(metadata.priority, Priority::LOW);
        assert_eq!(metadata.language, Language::Native);
        assert!(metadata.tags.contains(&"builtin".to_string()));
        assert!(metadata.tags.contains(&"logging".to_string()));
    }

    #[cfg_attr(test_category = "unit")]
    #[test]
    fn test_should_execute() {
        let hook = LoggingHook::new();
        let component_id = ComponentId::new(ComponentType::System, "test".to_string());
        let context = HookContext::new(HookPoint::SystemStartup, component_id);

        assert!(hook.should_execute(&context));
    }

    #[cfg_attr(test_category = "unit")]
    #[tokio::test]
    async fn test_replayable_hook_implementation() {
        let hook = LoggingHook::new()
            .with_level(LogLevel::Debug)
            .with_max_data_size(512);
        let component_id = ComponentId::new(ComponentType::Agent, "test-agent".to_string());
        let mut context = HookContext::new(HookPoint::BeforeAgentExecution, component_id);

        // Add test data
        context.insert_data("test_key".to_string(), json!("test_value"));
        context.insert_metadata("test_meta".to_string(), "meta_value".to_string());

        // Test serialization
        let serialized = hook.serialize_context(&context).unwrap();
        assert!(!serialized.is_empty());

        // Test deserialization
        let deserialized = hook.deserialize_context(&serialized).unwrap();
        assert_eq!(deserialized.point, context.point);
        assert_eq!(deserialized.component_id, context.component_id);
        assert_eq!(
            deserialized.data.get("test_key"),
            context.data.get("test_key")
        );
        assert_eq!(
            deserialized.get_metadata("test_meta"),
            context.get_metadata("test_meta")
        );

        // Ensure _logging_config was removed
        assert!(deserialized.data.get("_logging_config").is_none());

        // Test replay ID
        assert_eq!(hook.replay_id(), "logging_hook:1.0.0");
        assert!(hook.is_replayable());
    }
}
