//! ABOUTME: Resource-limited tool execution wrapper
//! ABOUTME: Provides `ResourceLimitedTool` trait and implementation for enforcing resource limits

use async_trait::async_trait;
use llmspell_core::{
    traits::{
        base_agent::BaseAgent,
        tool::{
            ResourceLimits as ToolResourceLimits, SecurityLevel, SecurityRequirements, Tool,
            ToolCategory, ToolSchema,
        },
    },
    types::{AgentInput, AgentOutput},
    ComponentMetadata, ExecutionContext, LLMSpellError, Result as LLMResult,
};
use llmspell_utils::resource_limits::{ResourceLimits, ResourceTracker};
use serde_json::json;

/// Trait for tools that support resource limiting
#[async_trait]
pub trait ResourceLimited: Tool {
    /// Get the resource limits for this tool
    fn resource_limits(&self) -> ResourceLimits {
        ResourceLimits::default()
    }

    /// Create a resource tracker for this tool
    fn create_tracker(&self) -> ResourceTracker {
        ResourceTracker::new(<Self as ResourceLimited>::resource_limits(self))
    }
}

/// Wrapper that adds resource limiting to any tool
pub struct ResourceLimitedTool<T: Tool> {
    inner: T,
    limits: ResourceLimits,
}

impl<T: Tool> ResourceLimitedTool<T> {
    /// Create a new resource-limited tool
    pub const fn new(tool: T, limits: ResourceLimits) -> Self {
        Self {
            inner: tool,
            limits,
        }
    }

    /// Create with default limits
    pub fn with_defaults(tool: T) -> Self {
        Self::new(tool, ResourceLimits::default())
    }

    /// Create with strict limits
    pub fn with_strict_limits(tool: T) -> Self {
        Self::new(tool, ResourceLimits::strict())
    }
}

#[async_trait]
impl<T: Tool + Send + Sync> BaseAgent for ResourceLimitedTool<T> {
    fn metadata(&self) -> &ComponentMetadata {
        self.inner.metadata()
    }

    async fn execute(
        &self,
        input: AgentInput,
        context: ExecutionContext,
    ) -> LLMResult<AgentOutput> {
        let tracker = ResourceTracker::new(self.limits.clone());

        // Track concurrent operation
        let _guard = tracker.track_concurrent_start()?;

        // Execute with timeout
        let result = tracker
            .with_timeout(self.inner.execute(input, context))
            .await??;

        // Add resource metrics to output
        let metrics = tracker.get_metrics();
        let metrics_json = json!({
            "resource_usage": {
                "memory_bytes": metrics.memory_bytes,
                "cpu_time_ms": metrics.cpu_time_ms,
                "operations_count": metrics.operations_count,
            }
        });

        let output_text = format!(
            "{}
\nResource usage: {}",
            result.text,
            serde_json::to_string_pretty(&metrics_json).unwrap()
        );
        Ok(AgentOutput::text(output_text))
    }

    async fn validate_input(&self, input: &AgentInput) -> LLMResult<()> {
        self.inner.validate_input(input).await
    }

    async fn handle_error(&self, error: LLMSpellError) -> LLMResult<AgentOutput> {
        self.inner.handle_error(error).await
    }
}

#[async_trait]
impl<T: Tool + Send + Sync> Tool for ResourceLimitedTool<T> {
    fn category(&self) -> ToolCategory {
        self.inner.category()
    }

    fn security_level(&self) -> SecurityLevel {
        self.inner.security_level()
    }

    fn schema(&self) -> ToolSchema {
        self.inner.schema()
    }

    fn security_requirements(&self) -> SecurityRequirements {
        self.inner.security_requirements()
    }

    fn resource_limits(&self) -> ToolResourceLimits {
        // Convert our ResourceLimits to Tool's ResourceLimits
        match self.limits.max_memory_bytes {
            Some(mem) if mem <= 10 * 1024 * 1024 => ToolResourceLimits::strict(),
            Some(_) => ToolResourceLimits::default(),
            None => ToolResourceLimits::unlimited(),
        }
    }
}

/// Extension trait to easily add resource limiting to tools
pub trait ResourceLimitExt: Tool + Sized {
    /// Wrap this tool with resource limits
    fn with_resource_limits(self, limits: ResourceLimits) -> ResourceLimitedTool<Self> {
        ResourceLimitedTool::new(self, limits)
    }

    /// Wrap with default resource limits
    fn with_default_limits(self) -> ResourceLimitedTool<Self> {
        ResourceLimitedTool::with_defaults(self)
    }

    /// Wrap with strict resource limits
    fn with_strict_limits(self) -> ResourceLimitedTool<Self> {
        ResourceLimitedTool::with_strict_limits(self)
    }
}

// Implement for all tools
impl<T: Tool> ResourceLimitExt for T {}

/// Helper to track file operations with size limits
///
/// # Errors
///
/// Returns an error if:
/// - Operation tracking fails due to resource limits
/// - File size exceeds configured limits
#[allow(clippy::unused_async)]
pub async fn check_file_operation(
    tracker: &ResourceTracker,
    path: &std::path::Path,
    operation: &str,
) -> LLMResult<()> {
    use std::fs;

    tracker.track_operation()?;

    if operation == "read" || operation == "write" {
        if let Ok(metadata) = fs::metadata(path) {
            #[allow(clippy::cast_possible_truncation)]
            let size = metadata.len() as usize;
            tracker.check_file_size(size)?;
        }
    }

    Ok(())
}

/// Helper to track memory allocation for data processing
///
/// # Errors
///
/// Returns an error if:
/// - Memory allocation would exceed configured limits
/// - The operation function returns an error
pub fn track_data_processing<T, F>(
    tracker: &ResourceTracker,
    estimated_size: usize,
    operation: F,
) -> LLMResult<T>
where
    F: FnOnce() -> LLMResult<T>,
{
    use llmspell_utils::resource_limits::MemoryGuard;

    // Track the memory allocation
    let _guard = MemoryGuard::new(tracker, estimated_size)?;

    // Track the operation
    tracker.track_operation()?;
    tracker.check_cpu_time()?;

    // Execute the operation
    operation()
}

#[cfg(test)]
mod tests {
    use super::*;

    // Mock tool for testing
    struct MockTool {
        metadata: ComponentMetadata,
        delay_ms: u64,
    }

    impl MockTool {
        fn new(delay_ms: u64) -> Self {
            Self {
                metadata: ComponentMetadata::new(
                    "mock_tool".to_string(),
                    "Mock tool for testing".to_string(),
                ),
                delay_ms,
            }
        }
    }

    #[async_trait]
    impl BaseAgent for MockTool {
        fn metadata(&self) -> &ComponentMetadata {
            &self.metadata
        }

        async fn execute(
            &self,
            _input: AgentInput,
            _context: ExecutionContext,
        ) -> LLMResult<AgentOutput> {
            tokio::time::sleep(tokio::time::Duration::from_millis(self.delay_ms)).await;
            Ok(AgentOutput::text("Success"))
        }

        async fn validate_input(&self, _input: &AgentInput) -> LLMResult<()> {
            Ok(())
        }

        async fn handle_error(&self, error: LLMSpellError) -> LLMResult<AgentOutput> {
            Ok(AgentOutput::text(format!("Error: {}", error)))
        }
    }

    #[async_trait]
    impl Tool for MockTool {
        fn category(&self) -> ToolCategory {
            ToolCategory::Utility
        }

        fn security_level(&self) -> SecurityLevel {
            SecurityLevel::Safe
        }

        fn schema(&self) -> ToolSchema {
            ToolSchema::new("mock_tool".to_string(), "Mock tool for testing".to_string())
        }
    }
    #[tokio::test]
    async fn test_resource_limited_tool() {
        let mock_tool = MockTool::new(50);
        let limited_tool = mock_tool.with_default_limits();

        let input = AgentInput::text("test");
        let context = ExecutionContext::default();

        let result = limited_tool.execute(input, context).await.unwrap();
        assert!(result.text.contains("Success"));
        assert!(result.text.contains("Resource usage:"));
    }
    #[tokio::test]
    async fn test_timeout_enforcement() {
        let mock_tool = MockTool::new(200);
        let limits = ResourceLimits {
            operation_timeout_ms: Some(100),
            ..Default::default()
        };
        let limited_tool = mock_tool.with_resource_limits(limits);

        let input = AgentInput::text("test");
        let context = ExecutionContext::default();

        let result = limited_tool.execute(input, context).await;
        assert!(result.is_err());

        if let Err(e) = result {
            match e {
                LLMSpellError::ResourceLimit { resource, .. } => {
                    assert_eq!(resource, "timeout");
                }
                _ => panic!("Expected ResourceLimit error"),
            }
        }
    }
    #[test]
    fn test_file_size_checking() {
        let tracker = ResourceTracker::new(ResourceLimits {
            max_file_size_bytes: Some(1024),
            ..Default::default()
        });

        // Should succeed
        assert!(tracker.check_file_size(512).is_ok());

        // Should fail
        let result = tracker.check_file_size(2048);
        assert!(result.is_err());
    }
    #[test]
    fn test_memory_tracking() {
        let tracker = ResourceTracker::new(ResourceLimits {
            max_memory_bytes: Some(1000),
            ..Default::default()
        });

        let result = track_data_processing(&tracker, 500, || {
            Ok::<_, LLMSpellError>("processed".to_string())
        });
        assert!(result.is_ok());

        // Should fail - too much memory
        let result = track_data_processing(&tracker, 1500, || {
            Ok::<_, LLMSpellError>("processed".to_string())
        });
        assert!(result.is_err());
    }
}
