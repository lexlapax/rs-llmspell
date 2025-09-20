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
use llmspell_utils::resource_limits::{MemoryGuard, ResourceLimits, ResourceTracker};
use serde_json::json;
use std::fs;
use std::time::Instant;
use tracing::{debug, error, info, instrument, trace, warn};

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
    pub fn new(tool: T, limits: ResourceLimits) -> Self {
        info!(
            tool_category = ?tool.category(),
            tool_security = ?tool.security_level(),
            memory_limit_mb = limits.max_memory_bytes.map(|m| m / (1024 * 1024)),
            cpu_limit_ms = limits.max_cpu_time_ms,
            file_size_limit_mb = limits.max_file_size_bytes.map(|s| s / (1024 * 1024)),
            timeout_ms = limits.operation_timeout_ms,
            max_operations = limits.max_operations,
            "Creating ResourceLimitedTool wrapper"
        );
        Self {
            inner: tool,
            limits,
        }
    }

    /// Create with default limits
    pub fn with_defaults(tool: T) -> Self {
        debug!("Creating ResourceLimitedTool with default limits");
        Self::new(tool, ResourceLimits::default())
    }

    /// Create with strict limits
    pub fn with_strict_limits(tool: T) -> Self {
        debug!("Creating ResourceLimitedTool with strict limits");
        Self::new(tool, ResourceLimits::strict())
    }
}

#[async_trait]
impl<T: Tool + Send + Sync> BaseAgent for ResourceLimitedTool<T> {
    fn metadata(&self) -> &ComponentMetadata {
        self.inner.metadata()
    }

    #[instrument(skip(self))]
    async fn execute_impl(
        &self,
        input: AgentInput,
        context: ExecutionContext,
    ) -> LLMResult<AgentOutput> {
        let start = Instant::now();
        info!(
            input_size = input.text.len(),
            has_params = !input.parameters.is_empty(),
            memory_limit_mb = self.limits.max_memory_bytes.map(|m| m / (1024 * 1024)),
            cpu_limit_ms = self.limits.max_cpu_time_ms,
            timeout_ms = self.limits.operation_timeout_ms,
            "Executing ResourceLimitedTool wrapper"
        );

        let tracker = ResourceTracker::new(self.limits.clone());

        debug!(
            concurrent_limit = self.limits.max_concurrent_ops,
            "Tracking concurrent operation start"
        );

        // Track concurrent operation
        let _guard = tracker.track_concurrent_start()?;

        debug!(
            inner_tool = %self.inner.metadata().name,
            "Executing inner tool with resource tracking"
        );

        // Execute with timeout
        let execution_start = Instant::now();
        let result = tracker
            .with_timeout(self.inner.execute(input, context))
            .await;

        let execution_duration_ms = execution_start.elapsed().as_millis();

        match result {
            Ok(Ok(inner_result)) => {
                // Add resource metrics to output
                let metrics = tracker.get_metrics();
                trace!(
                    memory_bytes = metrics.memory_bytes,
                    cpu_time_ms = metrics.cpu_time_ms,
                    operations_count = metrics.operations_count,
                    "Collected resource usage metrics"
                );

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
                    inner_result.text,
                    serde_json::to_string_pretty(&metrics_json).unwrap()
                );

                let total_duration_ms = start.elapsed().as_millis();
                info!(
                    inner_tool = %self.inner.metadata().name,
                    execution_duration_ms,
                    total_duration_ms,
                    memory_used = metrics.memory_bytes,
                    cpu_used_ms = metrics.cpu_time_ms,
                    operations_count = metrics.operations_count,
                    success = true,
                    "ResourceLimitedTool execution completed successfully"
                );

                Ok(AgentOutput::text(output_text))
            }
            Ok(Err(e)) => {
                error!(
                    inner_tool = %self.inner.metadata().name,
                    execution_duration_ms,
                    error = %e,
                    "Inner tool execution failed"
                );
                Err(e)
            }
            Err(e) => {
                error!(
                    inner_tool = %self.inner.metadata().name,
                    execution_duration_ms,
                    error = %e,
                    "Resource-limited execution failed (timeout or resource limit)"
                );
                Err(e)
            }
        }
    }

    #[instrument(skip(self))]
    async fn validate_input(&self, input: &AgentInput) -> LLMResult<()> {
        self.inner.validate_input(input).await
    }

    #[instrument(skip(self))]
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
#[allow(clippy::cognitive_complexity)]
#[instrument(skip(tracker))]
pub async fn check_file_operation(
    tracker: &ResourceTracker,
    path: &std::path::Path,
    operation: &str,
) -> LLMResult<()> {
    let check_start = Instant::now();
    debug!(
        operation = %operation,
        file_path = %path.display(),
        "Starting file operation check"
    );

    tracker.track_operation()?;

    if operation == "read" || operation == "write" {
        if let Ok(metadata) = fs::metadata(path) {
            #[allow(clippy::cast_possible_truncation)]
            let size = metadata.len() as usize;

            debug!(
                operation = %operation,
                file_path = %path.display(),
                file_size_bytes = size,
                file_size_mb = size / (1024 * 1024),
                "Checking file size against limits"
            );

            match tracker.check_file_size(size) {
                Ok(()) => {
                    trace!(
                        operation = %operation,
                        file_path = %path.display(),
                        file_size_bytes = size,
                        "File size check passed"
                    );
                }
                Err(e) => {
                    error!(
                        operation = %operation,
                        file_path = %path.display(),
                        file_size_bytes = size,
                        error = %e,
                        "File size check failed"
                    );
                    return Err(e);
                }
            }
        } else {
            warn!(
                operation = %operation,
                file_path = %path.display(),
                "Could not read file metadata"
            );
        }
    }

    let check_duration_ms = check_start.elapsed().as_millis();
    debug!(
        operation = %operation,
        file_path = %path.display(),
        duration_ms = check_duration_ms,
        "File operation check completed successfully"
    );

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
    let processing_start = Instant::now();
    debug!(
        estimated_size_bytes = estimated_size,
        estimated_size_mb = estimated_size / (1024 * 1024),
        "Starting data processing with memory tracking"
    );

    let _guard = create_memory_guard(tracker, estimated_size)?;
    perform_resource_checks(tracker)?;

    trace!("All resource checks passed, executing operation");

    execute_tracked_operation(operation, estimated_size, processing_start)
}

fn create_memory_guard(
    tracker: &ResourceTracker,
    estimated_size: usize,
) -> LLMResult<MemoryGuard<'_>> {
    let guard_result = MemoryGuard::new(tracker, estimated_size);
    match guard_result {
        Ok(guard) => {
            trace!(
                allocated_bytes = estimated_size,
                "Memory guard created successfully"
            );
            Ok(guard)
        }
        Err(e) => {
            error!(
                estimated_size_bytes = estimated_size,
                error = %e,
                "Failed to create memory guard - memory limit exceeded"
            );
            Err(e)
        }
    }
}

fn perform_resource_checks(tracker: &ResourceTracker) -> LLMResult<()> {
    if let Err(e) = tracker.track_operation() {
        error!(
            error = %e,
            "Failed to track operation - operation limit exceeded"
        );
        return Err(e);
    }

    if let Err(e) = tracker.check_cpu_time() {
        error!(
            error = %e,
            "CPU time limit exceeded before operation"
        );
        return Err(e);
    }

    Ok(())
}

fn execute_tracked_operation<T, F>(
    operation: F,
    estimated_size: usize,
    processing_start: Instant,
) -> LLMResult<T>
where
    F: FnOnce() -> LLMResult<T>,
{
    let operation_start = Instant::now();
    let result = operation();
    let operation_duration_ms = operation_start.elapsed().as_millis();
    let total_processing_duration_ms = processing_start.elapsed().as_millis();

    match result {
        Ok(value) => {
            debug!(
                estimated_size_bytes = estimated_size,
                operation_duration_ms,
                total_duration_ms = total_processing_duration_ms,
                "Data processing completed successfully"
            );
            Ok(value)
        }
        Err(e) => {
            error!(
                estimated_size_bytes = estimated_size,
                operation_duration_ms,
                total_duration_ms = total_processing_duration_ms,
                error = %e,
                "Data processing operation failed"
            );
            Err(e)
        }
    }
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

        #[instrument(skip(self))]
        async fn execute_impl(
            &self,
            _input: AgentInput,
            _context: ExecutionContext,
        ) -> LLMResult<AgentOutput> {
            tokio::time::sleep(tokio::time::Duration::from_millis(self.delay_ms)).await;
            Ok(AgentOutput::text("Success"))
        }

        #[instrument(skip(self))]
        async fn validate_input(&self, _input: &AgentInput) -> LLMResult<()> {
            Ok(())
        }

        #[instrument(skip(self))]
        async fn handle_error(&self, error: LLMSpellError) -> LLMResult<AgentOutput> {
            Ok(AgentOutput::text(format!("Error: {error}")))
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
