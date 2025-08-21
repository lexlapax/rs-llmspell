//! ABOUTME: File system monitoring tool for watching file changes
//! ABOUTME: Provides real-time file system event monitoring with pattern filtering

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::mpsc;
use std::sync::Arc;
use std::thread;
use std::time::Duration;

use anyhow::Result as AnyhowResult;
use async_trait::async_trait;
use llmspell_core::{
    traits::{
        base_agent::BaseAgent,
        tool::{
            ParameterDef, ParameterType, ResourceLimits, SecurityLevel, SecurityRequirements, Tool,
            ToolCategory, ToolSchema,
        },
    },
    types::{AgentInput, AgentOutput},
    ComponentMetadata, ExecutionContext, LLMSpellError, Result,
};
use llmspell_security::sandbox::{FileSandbox, SandboxContext};
use llmspell_utils::{
    extract_optional_bool, extract_optional_string, extract_optional_u64, extract_parameters,
    extract_required_array, extract_required_string,
    file_monitor::{debounce_events, FileEvent, FileEventType, WatchConfig},
    response::ResponseBuilder,
};
use notify::{RecommendedWatcher, RecursiveMode, Watcher};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use tracing::{error, info, warn};

/// Configuration for the `FileWatcherTool`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileWatcherConfig {
    /// Maximum number of events to buffer
    pub max_events: usize,
    /// Default timeout for watching operations in seconds
    pub default_timeout: u64,
    /// Default debounce duration in milliseconds
    pub default_debounce_ms: u64,
    /// Maximum number of paths to watch simultaneously
    pub max_paths: usize,
}

impl Default for FileWatcherConfig {
    fn default() -> Self {
        Self {
            max_events: 1000,
            default_timeout: 300, // 5 minutes
            default_debounce_ms: 100,
            max_paths: 100,
        }
    }
}

/// File system monitoring tool
pub struct FileWatcherTool {
    metadata: ComponentMetadata,
    config: FileWatcherConfig,
    sandbox: Arc<FileSandbox>,
}

impl FileWatcherTool {
    /// Create a new `FileWatcherTool`
    #[must_use]
    pub fn new(config: FileWatcherConfig, sandbox: Arc<FileSandbox>) -> Self {
        Self {
            metadata: ComponentMetadata::new(
                "file_watcher".to_string(),
                "Monitors file system changes and events".to_string(),
            ),
            config,
            sandbox,
        }
    }

    /// Start watching files and return events
    #[allow(clippy::unused_async)]
    async fn watch_files(&self, watch_config: WatchConfig) -> AnyhowResult<Vec<FileEvent>> {
        // Validate configuration
        watch_config.validate()?;

        // Validate paths with sandbox
        for path in &watch_config.paths {
            if self.sandbox.validate_path(path).is_err() {
                return Err(anyhow::anyhow!(
                    "Path not allowed by sandbox: {}",
                    path.display()
                ));
            }
        }

        if watch_config.paths.len() > self.config.max_paths {
            return Err(anyhow::anyhow!(
                "Too many paths to watch: {} (max: {})",
                watch_config.paths.len(),
                self.config.max_paths
            ));
        }

        let (tx, rx) = mpsc::channel();
        let mut events = Vec::new();

        // Create watcher
        let mut watcher = RecommendedWatcher::new(
            move |res| {
                if let Err(e) = tx.send(res) {
                    error!("Failed to send watch event: {}", e);
                }
            },
            notify::Config::default(),
        )?;

        // Start watching paths
        for path in &watch_config.paths {
            let mode = if watch_config.recursive {
                RecursiveMode::Recursive
            } else {
                RecursiveMode::NonRecursive
            };

            if let Err(e) = watcher.watch(path, mode) {
                warn!("Failed to watch path {}: {}", path.display(), e);
                continue;
            }

            info!("Started watching path: {}", path.display());
        }

        // Collect events
        let timeout = Duration::from_secs(
            watch_config
                .timeout_seconds
                .unwrap_or(self.config.default_timeout),
        );
        let start_time = std::time::Instant::now();

        while start_time.elapsed() < timeout && events.len() < self.config.max_events {
            match rx.recv_timeout(Duration::from_millis(100)) {
                Ok(Ok(event)) => {
                    if let Some(file_event) = self.convert_notify_event(event, &watch_config) {
                        events.push(file_event);
                    }
                }
                Ok(Err(e)) => {
                    warn!("Watch error: {}", e);
                }
                Err(mpsc::RecvTimeoutError::Timeout) => {
                    // Check if we should continue waiting
                    if !events.is_empty() {
                        // We have some events, check if we should wait for more
                        thread::sleep(Duration::from_millis(watch_config.debounce_ms));
                        break;
                    }
                }
                Err(mpsc::RecvTimeoutError::Disconnected) => {
                    break;
                }
            }
        }

        // Apply debouncing
        let debounced_events = debounce_events(events, watch_config.debounce_ms);

        info!("Collected {} file events", debounced_events.len());
        Ok(debounced_events)
    }

    /// Convert notify event to our `FileEvent`
    fn convert_notify_event(
        &self,
        event: notify::Event,
        config: &WatchConfig,
    ) -> Option<FileEvent> {
        let event_type = match event.kind {
            notify::EventKind::Create(_) => FileEventType::Create,
            notify::EventKind::Modify(_) => FileEventType::Modify,
            notify::EventKind::Remove(_) => FileEventType::Delete,
            _ => FileEventType::Other,
        };

        // Handle paths
        let paths = event.paths;
        if paths.is_empty() {
            return None;
        }

        let path = paths[0].clone();

        // Check if path should be watched
        if !llmspell_utils::file_monitor::should_watch_path(&path, config) {
            return None;
        }

        // Check sandbox permissions
        if self.sandbox.validate_path(&path).is_err() {
            return None;
        }

        // Handle rename events
        if paths.len() > 1 {
            Some(FileEvent::new_rename(paths[0].clone(), paths[1].clone()))
        } else {
            Some(FileEvent::new(event_type, path))
        }
    }

    /// Validate parameters for file watching operations
    #[allow(clippy::unused_async)]
    async fn validate_parameters(&self, params: &Value) -> Result<()> {
        if !params.is_object() {
            return Err(LLMSpellError::Validation {
                message: "Parameters must be a JSON object".to_string(),
                field: Some("parameters".to_string()),
            });
        }

        let operation = params.get("operation").and_then(|v| v.as_str());
        if let Some(op) = operation {
            if !matches!(op, "watch" | "config") {
                return Err(LLMSpellError::Validation {
                    message: format!("Invalid operation: {op}"),
                    field: Some("operation".to_string()),
                });
            }
        }

        Ok(())
    }
}

impl Default for FileWatcherTool {
    fn default() -> Self {
        // Create a default sandbox context for testing
        let security_requirements = SecurityRequirements {
            level: SecurityLevel::Restricted,
            file_permissions: vec!["*".to_string()],
            network_permissions: vec![],
            env_permissions: vec![],
            custom_requirements: HashMap::new(),
        };
        let resource_limits = ResourceLimits::default();
        let context = SandboxContext::new(
            "default_file_watcher".to_string(),
            security_requirements,
            resource_limits,
        );
        let sandbox = Arc::new(FileSandbox::new(context).unwrap());
        Self::new(FileWatcherConfig::default(), sandbox)
    }
}

#[async_trait]
impl BaseAgent for FileWatcherTool {
    fn metadata(&self) -> &ComponentMetadata {
        &self.metadata
    }

    async fn execute_impl(
        &self,
        input: AgentInput,
        _context: ExecutionContext,
    ) -> Result<AgentOutput> {
        // Get parameters using shared utility
        let params = extract_parameters(&input)?;

        // Validate parameters
        self.validate_parameters(params).await?;

        let operation = extract_required_string(params, "operation")?;

        match operation {
            "watch" => {
                let paths_array = extract_required_array(params, "input")?;
                let paths: Vec<PathBuf> = paths_array
                    .iter()
                    .map(|v| {
                        v.as_str()
                            .ok_or_else(|| LLMSpellError::Validation {
                                message: "Path must be a string".to_string(),
                                field: Some("input".to_string()),
                            })
                            .map(PathBuf::from)
                    })
                    .collect::<Result<Vec<_>>>()?;

                let recursive = extract_optional_bool(params, "recursive").unwrap_or(true);
                let pattern = extract_optional_string(params, "pattern")
                    .map(std::string::ToString::to_string);
                let debounce_ms = extract_optional_u64(params, "debounce_ms")
                    .unwrap_or(self.config.default_debounce_ms);
                // Support both timeout_ms and timeout_seconds for flexibility
                let timeout_seconds = extract_optional_u64(params, "timeout_ms").map_or_else(
                    || {
                        extract_optional_u64(params, "timeout_seconds")
                            .unwrap_or(self.config.default_timeout)
                    },
                    |ms| ms.div_ceil(1000), // Round up to nearest second
                );
                let max_events = usize::try_from(
                    extract_optional_u64(params, "max_events")
                        .unwrap_or(self.config.max_events as u64),
                )
                .unwrap_or(usize::MAX);

                let mut watch_config = WatchConfig::new()
                    .recursive(recursive)
                    .debounce(Duration::from_millis(debounce_ms))
                    .timeout(Duration::from_secs(timeout_seconds))
                    .max_events(max_events);

                for path in paths {
                    watch_config = watch_config.add_path(path);
                }

                if let Some(p) = pattern {
                    watch_config = watch_config.pattern(p);
                }

                let events =
                    self.watch_files(watch_config)
                        .await
                        .map_err(|e| LLMSpellError::Tool {
                            message: format!("File watching failed: {e}"),
                            tool_name: Some("file_watcher".to_string()),
                            source: None,
                        })?;

                let response = ResponseBuilder::success("watch")
                    .with_message(format!("Captured {} file events", events.len()))
                    .with_result(json!({
                        "events": events,
                        "event_count": events.len()
                    }))
                    .build();

                Ok(AgentOutput::text(serde_json::to_string_pretty(&response)?))
            }
            "config" => {
                let response = ResponseBuilder::success("config")
                    .with_message("File watcher configuration")
                    .with_result(json!({
                        "max_events": self.config.max_events,
                        "default_timeout": self.config.default_timeout,
                        "default_debounce_ms": self.config.default_debounce_ms,
                        "max_paths": self.config.max_paths
                    }))
                    .build();

                Ok(AgentOutput::text(serde_json::to_string_pretty(&response)?))
            }
            _ => Err(LLMSpellError::Validation {
                message: format!("Unknown operation: {operation}"),
                field: Some("operation".to_string()),
            }),
        }
    }

    async fn validate_input(&self, input: &AgentInput) -> Result<()> {
        if input.text.is_empty() {
            return Err(LLMSpellError::Validation {
                message: "Input prompt cannot be empty".to_string(),
                field: Some("prompt".to_string()),
            });
        }
        Ok(())
    }

    async fn handle_error(&self, error: LLMSpellError) -> Result<AgentOutput> {
        Ok(AgentOutput::text(format!("File watcher error: {error}")))
    }
}

#[async_trait]
impl Tool for FileWatcherTool {
    fn category(&self) -> ToolCategory {
        ToolCategory::Filesystem
    }

    fn security_level(&self) -> SecurityLevel {
        SecurityLevel::Restricted // File watching requires restricted security
    }

    fn schema(&self) -> ToolSchema {
        ToolSchema::new(
            "file_watcher".to_string(),
            "Monitors file system changes and events".to_string(),
        )
        .with_parameter(ParameterDef {
            name: "operation".to_string(),
            param_type: ParameterType::String,
            description: "Operation to perform: watch or config".to_string(),
            required: true,
            default: None,
        })
        .with_parameter(ParameterDef {
            name: "input".to_string(),
            param_type: ParameterType::Array,
            description: "Paths to watch for changes (required for watch operation)".to_string(),
            required: false,
            default: None,
        })
        .with_parameter(ParameterDef {
            name: "recursive".to_string(),
            param_type: ParameterType::Boolean,
            description: "Watch directories recursively".to_string(),
            required: false,
            default: Some(json!(true)),
        })
        .with_parameter(ParameterDef {
            name: "pattern".to_string(),
            param_type: ParameterType::String,
            description: "Glob pattern to filter watched files".to_string(),
            required: false,
            default: None,
        })
        .with_parameter(ParameterDef {
            name: "debounce_ms".to_string(),
            param_type: ParameterType::Number,
            description: "Debounce duration in milliseconds".to_string(),
            required: false,
            default: Some(json!(100)),
        })
        .with_parameter(ParameterDef {
            name: "timeout_seconds".to_string(),
            param_type: ParameterType::Number,
            description: "Timeout for watching in seconds".to_string(),
            required: false,
            default: Some(json!(300)),
        })
        .with_parameter(ParameterDef {
            name: "timeout_ms".to_string(),
            param_type: ParameterType::Number,
            description: "Timeout for watching in milliseconds (alternative to timeout_seconds)"
                .to_string(),
            required: false,
            default: None,
        })
        .with_parameter(ParameterDef {
            name: "max_events".to_string(),
            param_type: ParameterType::Number,
            description: "Maximum number of events to collect".to_string(),
            required: false,
            default: Some(json!(1000)),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use llmspell_core::traits::tool::{ResourceLimits, SecurityRequirements};
    use llmspell_security::sandbox::SandboxContext;
    use llmspell_testing::tool_helpers::create_test_tool_input;
    use tempfile::TempDir;

    fn create_test_file_watcher() -> (FileWatcherTool, TempDir) {
        let temp_dir = TempDir::new().unwrap();
        let config = FileWatcherConfig::default();

        let security_requirements = SecurityRequirements {
            level: SecurityLevel::Restricted,
            file_permissions: vec![temp_dir.path().to_string_lossy().to_string()],
            network_permissions: vec![],
            env_permissions: vec![],
            custom_requirements: HashMap::new(),
        };
        let resource_limits = ResourceLimits::default();
        let context = SandboxContext::new(
            "test_file_watcher".to_string(),
            security_requirements,
            resource_limits,
        );
        let sandbox = Arc::new(FileSandbox::new(context).unwrap());
        let tool = FileWatcherTool::new(config, sandbox);
        (tool, temp_dir)
    }
    #[tokio::test]
    async fn test_file_watcher_tool_metadata() {
        let (tool, _temp_dir) = create_test_file_watcher();
        let metadata = tool.metadata();
        assert_eq!(metadata.name, "file_watcher");
        assert_eq!(
            metadata.description,
            "Monitors file system changes and events"
        );
    }
    #[tokio::test]
    async fn test_file_watcher_tool_category() {
        let (tool, _temp_dir) = create_test_file_watcher();
        assert_eq!(tool.category(), ToolCategory::Filesystem);
    }
    #[tokio::test]
    async fn test_file_watcher_tool_security_level() {
        let (tool, _temp_dir) = create_test_file_watcher();
        assert_eq!(tool.security_level(), SecurityLevel::Restricted);
    }
    #[tokio::test]
    async fn test_schema() {
        let (tool, _temp_dir) = create_test_file_watcher();
        let schema = tool.schema();
        assert_eq!(schema.name, "file_watcher");
        assert_eq!(
            schema.description,
            "Monitors file system changes and events"
        );
        assert!(!schema.parameters.is_empty());
    }
    #[tokio::test]
    async fn test_config_operation() {
        let (tool, _temp_dir) = create_test_file_watcher();
        let input = create_test_tool_input(vec![("operation", "config")]);

        let result = tool
            .execute(input, ExecutionContext::default())
            .await
            .unwrap();

        let output: Value = serde_json::from_str(&result.text).unwrap();
        assert!(output["result"]["max_events"].is_number());
        assert!(output["result"]["default_timeout"].is_number());
        assert!(output["result"]["default_debounce_ms"].is_number());
        assert!(output["result"]["max_paths"].is_number());
    }
    #[tokio::test]
    async fn test_watch_operation_requires_paths() {
        let (tool, _temp_dir) = create_test_file_watcher();
        let input = create_test_tool_input(vec![("operation", "watch")]);

        let result = tool.execute(input, ExecutionContext::default()).await;
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Missing required array parameter 'input'"));
    }
    #[tokio::test]
    async fn test_watch_operation_with_nonexistent_path() {
        let (tool, _temp_dir) = create_test_file_watcher();
        let input = create_test_tool_input(vec![("operation", "watch"), ("timeout_seconds", "1")]);

        // Need to add the array parameter separately since create_test_tool_input handles simple values
        let mut input = input;
        input
            .parameters
            .get_mut("parameters")
            .and_then(|v| v.as_object_mut())
            .map(|obj| obj.insert("input".to_string(), json!(["/nonexistent/path"])));

        let result = tool.execute(input, ExecutionContext::default()).await;
        assert!(result.is_err());
    }
    #[tokio::test]
    async fn test_watch_operation_with_valid_path() {
        let (tool, temp_dir) = create_test_file_watcher();
        let input = create_test_tool_input(vec![
            ("operation", "watch"),
            ("timeout_seconds", "1"),
            ("max_events", "10"),
        ]);

        // Need to add the array parameter separately
        let mut input = input;
        input
            .parameters
            .get_mut("parameters")
            .and_then(|v| v.as_object_mut())
            .map(|obj| {
                obj.insert(
                    "input".to_string(),
                    json!([temp_dir.path().to_string_lossy()]),
                )
            });

        let result = tool
            .execute(input, ExecutionContext::default())
            .await
            .unwrap();

        let output: Value = serde_json::from_str(&result.text).unwrap();
        assert!(output["result"]["events"].is_array());
        assert!(output["result"]["event_count"].is_number());
    }
    #[tokio::test]
    async fn test_convert_notify_event() {
        let (tool, temp_dir) = create_test_file_watcher();
        let config = WatchConfig::new().add_path(temp_dir.path());

        let notify_event = notify::Event {
            kind: notify::EventKind::Create(notify::event::CreateKind::File),
            paths: vec![temp_dir.path().join("test.txt")],
            attrs: notify::event::EventAttributes::default(),
        };

        let file_event = tool.convert_notify_event(notify_event, &config);
        assert!(file_event.is_some());

        let event = file_event.unwrap();
        assert_eq!(event.event_type, FileEventType::Create);
        assert_eq!(event.path, temp_dir.path().join("test.txt"));
    }
}
