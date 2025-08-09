//! ABOUTME: Tool-specific hook integration following patterns from agent lifecycle
//! ABOUTME: Provides 8 hook points mapped to tool execution lifecycle with performance monitoring

use anyhow::Result;
use async_trait::async_trait;
use llmspell_core::{
    traits::tool::{SecurityLevel, Tool, ToolCategory},
    types::{AgentInput, AgentOutput},
    ComponentMetadata, ExecutionContext, LLMSpellError,
};
use llmspell_hooks::{
    CircuitBreaker, ComponentId, ComponentType, HookContext, HookExecutor, HookPoint, HookRegistry,
};
use llmspell_utils::resource_limits::{ResourceLimits, ResourceMetrics, ResourceTracker};
use serde_json::Value as JsonValue;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tracing::{debug, error, info, warn};

/// Hook execution features configuration
#[derive(Debug, Clone, Default)]
pub struct HookFeatures {
    /// Enable hook execution (can be disabled for performance)
    pub hooks_enabled: bool,
    /// Enable circuit breaker protection
    pub circuit_breaker_enabled: bool,
    /// Enable security-level validation for hooks
    pub security_validation_enabled: bool,
}

/// Audit logging configuration
#[derive(Debug, Clone, Default)]
pub struct AuditConfig {
    /// Enable comprehensive audit logging
    pub enabled: bool,
    /// Audit log sensitive parameters (be careful with secrets)
    pub log_parameters: bool,
}

/// Configuration for tool lifecycle hook integration
#[derive(Debug, Clone)]
pub struct ToolLifecycleConfig {
    /// Hook features configuration
    pub features: HookFeatures,
    /// Maximum time allowed for hook execution
    pub max_hook_execution_time: Duration,
    /// Resource limits for tool execution including hooks
    pub resource_limits: ResourceLimits,
    /// Circuit breaker configuration
    pub circuit_breaker_failure_threshold: u32,
    pub circuit_breaker_recovery_time: Duration,
    /// Maximum security level allowed for hook execution
    pub max_security_level: SecurityLevel,
    /// Audit logging configuration
    pub audit: AuditConfig,
}

impl Default for ToolLifecycleConfig {
    fn default() -> Self {
        Self {
            features: HookFeatures {
                hooks_enabled: true,
                circuit_breaker_enabled: true,
                security_validation_enabled: true,
            },
            max_hook_execution_time: Duration::from_millis(100), // 100ms max for hooks
            resource_limits: ResourceLimits::default(),
            circuit_breaker_failure_threshold: 5,
            circuit_breaker_recovery_time: Duration::from_secs(30),
            max_security_level: SecurityLevel::Privileged, // Allow all security levels by default
            audit: AuditConfig {
                enabled: true,
                log_parameters: false, // Don't log parameters by default to avoid leaking secrets
            },
        }
    }
}

/// Tool-specific hook context with execution metadata
#[derive(Debug, Clone)]
pub struct ToolHookContext {
    /// Base hook context
    pub base_context: HookContext,
    /// Tool metadata
    pub tool_metadata: ComponentMetadata,
    /// Tool category
    pub tool_category: ToolCategory,
    /// Security level
    pub security_level: SecurityLevel,
    /// Input parameters (pre-execution) or None (post-execution)
    pub input_parameters: Option<JsonValue>,
    /// Execution success flag (post-execution only)
    pub execution_success: Option<bool>,
    /// Resource usage metrics at this point
    pub resource_metrics: HashMap<String, JsonValue>,
    /// Tool execution phase
    pub execution_phase: ToolExecutionPhase,
}

/// Tool execution phases for hook context
#[derive(Debug, Clone)]
pub enum ToolExecutionPhase {
    PreExecution,
    PostExecution,
    ParameterValidation,
    SecurityCheck,
    ResourceAllocation,
    ResourceCleanup,
    ErrorHandling,
    Timeout,
}

impl ToolHookContext {
    /// Create a new tool hook context
    #[must_use]
    pub fn new(
        component_id: ComponentId,
        tool_metadata: ComponentMetadata,
        tool_category: ToolCategory,
        security_level: SecurityLevel,
        execution_phase: ToolExecutionPhase,
    ) -> Self {
        let hook_point = match execution_phase {
            ToolExecutionPhase::PreExecution => HookPoint::BeforeToolExecution,
            ToolExecutionPhase::PostExecution => HookPoint::AfterToolExecution,
            ToolExecutionPhase::ParameterValidation => {
                HookPoint::Custom("tool_parameter_validation".to_string())
            }
            ToolExecutionPhase::SecurityCheck => {
                HookPoint::Custom("tool_security_check".to_string())
            }
            ToolExecutionPhase::ResourceAllocation => {
                HookPoint::Custom("tool_resource_allocated".to_string())
            }
            ToolExecutionPhase::ResourceCleanup => {
                HookPoint::Custom("tool_resource_released".to_string())
            }
            ToolExecutionPhase::ErrorHandling => HookPoint::ToolError,
            ToolExecutionPhase::Timeout => HookPoint::Custom("tool_timeout".to_string()),
        };
        let base_context = HookContext::new(hook_point, component_id);

        Self {
            base_context,
            tool_metadata,
            tool_category,
            security_level,
            input_parameters: None,
            execution_success: None,
            resource_metrics: HashMap::new(),
            execution_phase,
        }
    }

    /// Set input parameters (for pre-execution hooks)
    #[must_use]
    pub fn with_input_parameters(mut self, parameters: JsonValue) -> Self {
        self.input_parameters = Some(parameters);
        self
    }

    /// Set execution success flag (for post-execution hooks)
    #[must_use]
    pub const fn with_execution_success(mut self, success: bool) -> Self {
        self.execution_success = Some(success);
        self
    }

    /// Add resource metrics
    #[must_use]
    pub fn with_resource_metrics(mut self, metrics: HashMap<String, JsonValue>) -> Self {
        self.resource_metrics = metrics;
        self
    }

    /// Add resource metrics from `ResourceTracker`
    #[must_use]
    pub fn with_resource_tracker_metrics(mut self, tracker: &ResourceTracker) -> Self {
        let metrics = tracker.get_metrics();
        self.resource_metrics = Self::convert_resource_metrics_to_json(&metrics);
        self
    }

    /// Convert `ResourceMetrics` to JSON `HashMap` for hook context
    fn convert_resource_metrics_to_json(metrics: &ResourceMetrics) -> HashMap<String, JsonValue> {
        let mut resource_metrics = HashMap::new();
        resource_metrics.insert(
            "memory_bytes".to_string(),
            JsonValue::from(metrics.memory_bytes),
        );
        resource_metrics.insert(
            "cpu_time_ms".to_string(),
            JsonValue::from(metrics.cpu_time_ms),
        );
        resource_metrics.insert(
            "operations_count".to_string(),
            JsonValue::from(metrics.operations_count),
        );
        resource_metrics.insert(
            "concurrent_ops".to_string(),
            JsonValue::from(metrics.concurrent_ops),
        );
        resource_metrics
    }

    /// Get hook point for this execution phase
    #[must_use]
    pub fn get_hook_point(&self) -> HookPoint {
        match self.execution_phase {
            ToolExecutionPhase::PreExecution => HookPoint::BeforeToolExecution,
            ToolExecutionPhase::PostExecution => HookPoint::AfterToolExecution,
            ToolExecutionPhase::ParameterValidation => {
                HookPoint::Custom("tool_parameter_validation".to_string())
            }
            ToolExecutionPhase::SecurityCheck => {
                HookPoint::Custom("tool_security_check".to_string())
            }
            ToolExecutionPhase::ResourceAllocation => {
                HookPoint::Custom("tool_resource_allocated".to_string())
            }
            ToolExecutionPhase::ResourceCleanup => {
                HookPoint::Custom("tool_resource_released".to_string())
            }
            ToolExecutionPhase::ErrorHandling => HookPoint::ToolError,
            ToolExecutionPhase::Timeout => HookPoint::Custom("tool_timeout".to_string()),
        }
    }
}

/// Enhanced tool executor with hook integration
#[derive(Clone)]
pub struct ToolExecutor {
    /// Hook executor for running hooks
    hook_executor: Option<Arc<HookExecutor>>,
    /// Hook registry for retrieving hooks
    hook_registry: Option<Arc<HookRegistry>>,
    /// Circuit breaker for performance protection
    circuit_breaker: Option<Arc<CircuitBreaker>>,
    /// Configuration
    config: ToolLifecycleConfig,
    /// Component ID for this tool executor
    #[allow(dead_code)]
    component_id: ComponentId,
}

impl ToolExecutor {
    /// Create a new tool executor with hook integration
    #[must_use]
    pub fn new(
        config: ToolLifecycleConfig,
        hook_executor: Option<Arc<HookExecutor>>,
        hook_registry: Option<Arc<HookRegistry>>,
    ) -> Self {
        let component_id = ComponentId::new(ComponentType::Tool, "tool_executor".to_string());

        let circuit_breaker = if config.features.circuit_breaker_enabled {
            hook_registry.as_ref().map(|_registry| {
                Arc::new(CircuitBreaker::new(format!(
                    "tool_executor_{}",
                    component_id.name
                )))
            })
        } else {
            None
        };

        Self {
            hook_executor,
            hook_registry,
            circuit_breaker,
            config,
            component_id,
        }
    }

    /// Execute a tool with full hook integration
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Hook execution fails and cannot be recovered
    /// - Tool execution fails
    /// - Resource limits are exceeded
    /// - Circuit breaker trips due to repeated failures
    #[allow(clippy::too_many_lines)]
    pub async fn execute_tool_with_hooks(
        &self,
        tool: &dyn Tool,
        input: AgentInput,
        context: ExecutionContext,
    ) -> Result<AgentOutput, LLMSpellError> {
        let start_time = Instant::now();
        let resource_tracker = ResourceTracker::new(self.config.resource_limits.clone());

        // Track the operation
        resource_tracker.track_operation()?;

        // Create component ID for this specific tool
        let tool_component_id = ComponentId::new(ComponentType::Tool, tool.metadata().name.clone());

        // Security validation (before any execution)
        let security_level = tool.security_level();
        self.validate_tool_security(&security_level)?;

        // Phase 1: Parameter Validation Hook
        let mut tool_context = ToolHookContext::new(
            tool_component_id.clone(),
            tool.metadata().clone(),
            tool.category(),
            security_level.clone(),
            ToolExecutionPhase::ParameterValidation,
        )
        .with_input_parameters(serde_json::to_value(input.parameters.clone()).unwrap_or_default());

        // Create audit log entry for parameter validation phase
        let mut audit_entry = self.create_audit_log_entry(
            &tool.metadata().name,
            security_level.clone(),
            &ToolExecutionPhase::ParameterValidation,
            &input,
            &HashMap::new(),
        );
        self.log_audit_entry(&audit_entry);

        self.execute_hook_phase(&tool_context, None::<()>).await?;

        // Phase 2: Security Check Hook
        tool_context.execution_phase = ToolExecutionPhase::SecurityCheck;
        self.execute_hook_phase(&tool_context, None::<()>).await?;

        // Phase 3: Resource Allocation Hook
        tool_context.execution_phase = ToolExecutionPhase::ResourceAllocation;
        tool_context = tool_context.with_resource_tracker_metrics(&resource_tracker);
        self.execute_hook_phase(&tool_context, None::<()>).await?;

        // Phase 4: Pre-execution Hook
        tool_context.execution_phase = ToolExecutionPhase::PreExecution;
        self.execute_hook_phase(&tool_context, None::<()>).await?;
        let final_input = input; // For now, don't modify the input

        // Phase 5: Actual Tool Execution
        let execution_result = if let Some(ref _circuit_breaker) = self.circuit_breaker {
            // TODO: Integrate circuit breaker properly - for now just execute directly
            resource_tracker
                .with_timeout(async { tool.execute(final_input.clone(), context.clone()).await })
                .await
                .map_err(|timeout_error| LLMSpellError::Component {
                    message: format!("Tool execution timed out: {timeout_error}"),
                    source: Some(Box::new(timeout_error)),
                })
        } else {
            // Execute without circuit breaker
            resource_tracker
                .with_timeout(async { tool.execute(final_input.clone(), context.clone()).await })
                .await
                .map_err(|timeout_error| LLMSpellError::Component {
                    message: format!("Tool execution timed out: {timeout_error}"),
                    source: Some(Box::new(timeout_error)),
                })
        };

        // Phase 6: Handle result or error
        let final_result = match execution_result {
            Ok(result) => {
                match result {
                    Ok(output) => {
                        // Phase 6a: Post-execution Hook (success)
                        tool_context.execution_phase = ToolExecutionPhase::PostExecution;
                        tool_context = tool_context
                            .with_execution_success(true)
                            .with_resource_tracker_metrics(&resource_tracker);
                        self.execute_hook_phase(&tool_context, None::<()>).await?;

                        // Log successful completion
                        audit_entry.execution_phase = ToolExecutionPhase::PostExecution;
                        audit_entry.success = Some(true);
                        audit_entry
                            .resource_metrics
                            .clone_from(&tool_context.resource_metrics);
                        self.log_audit_entry(&audit_entry);

                        Ok(output)
                    }
                    Err(error) => {
                        // Phase 6b: Error Handling Hook
                        tool_context.execution_phase = ToolExecutionPhase::ErrorHandling;
                        tool_context = tool_context
                            .with_execution_success(false)
                            .with_resource_tracker_metrics(&resource_tracker);
                        self.execute_hook_phase(&tool_context, None::<()>).await?;

                        // Log error execution
                        audit_entry.execution_phase = ToolExecutionPhase::ErrorHandling;
                        audit_entry.success = Some(false);
                        audit_entry.error_message = Some(error.to_string());
                        audit_entry
                            .resource_metrics
                            .clone_from(&tool_context.resource_metrics);
                        self.log_audit_entry(&audit_entry);

                        // Try to handle the error through the tool's error handler
                        match tool.handle_error(error).await {
                            Ok(recovered_output) => {
                                info!("Tool error recovered successfully");
                                // Update audit log to reflect recovery
                                audit_entry.success = Some(true);
                                audit_entry.error_message = Some("Error recovered".to_string());
                                self.log_audit_entry(&audit_entry);
                                Ok(recovered_output)
                            }
                            Err(original_error) => Err(original_error), // Return original error if recovery failed
                        }
                    }
                }
            }
            Err(timeout_error) => {
                // Log timeout error
                audit_entry.execution_phase = ToolExecutionPhase::Timeout;
                audit_entry.success = Some(false);
                audit_entry.error_message = Some(timeout_error.to_string());
                audit_entry.resource_metrics = ToolHookContext::convert_resource_metrics_to_json(
                    &resource_tracker.get_metrics(),
                );
                self.log_audit_entry(&audit_entry);

                Err(timeout_error) // Timeout error from resource tracker
            }
        };

        // Phase 7: Resource Cleanup Hook
        tool_context.execution_phase = ToolExecutionPhase::ResourceCleanup;
        tool_context = tool_context.with_resource_tracker_metrics(&resource_tracker);
        self.execute_hook_phase(&tool_context, None::<()>).await?;

        // Final audit log entry for cleanup phase
        audit_entry.execution_phase = ToolExecutionPhase::ResourceCleanup;
        audit_entry
            .resource_metrics
            .clone_from(&tool_context.resource_metrics);
        self.log_audit_entry(&audit_entry);

        // Log execution metrics
        let execution_time = start_time.elapsed();
        debug!(
            "Tool '{}' executed in {:?} with hook integration",
            tool.metadata().name,
            execution_time
        );

        final_result
    }

    /// Get resource metrics from all tool executions
    #[must_use]
    pub const fn get_execution_metrics(&self) -> ExecutionMetrics {
        ExecutionMetrics {
            total_executions: 0,     // TODO: Track this across executions
            hook_overhead_ms: 0,     // TODO: Track hook execution time
            resource_limits_hit: 0,  // TODO: Track resource limit violations
            average_memory_usage: 0, // TODO: Track across executions
            average_cpu_time: 0,     // TODO: Track across executions
        }
    }

    /// Execute hooks for a specific phase
    async fn execute_hook_phase<T: Clone>(
        &self,
        tool_context: &ToolHookContext,
        input_data: Option<T>,
    ) -> Result<Option<T>, LLMSpellError> {
        if !self.config.features.hooks_enabled {
            return Ok(input_data);
        }

        let (Some(ref hook_executor), Some(ref hook_registry)) =
            (&self.hook_executor, &self.hook_registry)
        else {
            return Ok(input_data);
        };

        let hook_point = tool_context.get_hook_point();

        // Track hook execution time for performance monitoring
        let hook_start = Instant::now();

        // Log resource metrics at this hook point
        if let Some(memory_bytes) = tool_context.resource_metrics.get("memory_bytes") {
            if let Some(cpu_time_ms) = tool_context.resource_metrics.get("cpu_time_ms") {
                debug!(
                    "Hook phase {:?} - Resource usage: {}MB memory, {}ms CPU",
                    tool_context.execution_phase,
                    memory_bytes.as_u64().unwrap_or(0) / (1024 * 1024),
                    cpu_time_ms.as_u64().unwrap_or(0)
                );
            }
        }

        // Get hooks from registry for this hook point
        let hooks = hook_registry.get_hooks(&hook_point);

        if !hooks.is_empty() {
            // Convert tool context to hook context for execution
            let mut hook_context = tool_context.base_context.clone();

            // Add tool-specific metadata
            hook_context.metadata.insert(
                "tool_name".to_string(),
                tool_context.tool_metadata.name.clone(),
            );
            hook_context.metadata.insert(
                "tool_category".to_string(),
                format!("{:?}", tool_context.tool_category),
            );
            hook_context.metadata.insert(
                "security_level".to_string(),
                format!("{:?}", tool_context.security_level),
            );
            hook_context.metadata.insert(
                "execution_phase".to_string(),
                format!("{:?}", tool_context.execution_phase),
            );

            // Add resource metrics to hook context
            for (key, value) in &tool_context.resource_metrics {
                hook_context
                    .data
                    .insert(format!("resource_{key}"), value.clone());
            }

            // Execute hooks
            let results = hook_executor.execute_hooks(&hooks, &mut hook_context).await;

            match results {
                Ok(hook_results) => {
                    // Check results for any that should block execution
                    for result in hook_results {
                        if let llmspell_hooks::HookResult::Cancel(reason) = result {
                            return Err(LLMSpellError::Tool {
                                message: format!(
                                    "Hook cancelled tool execution for phase {:?}: {}",
                                    tool_context.execution_phase, reason
                                ),
                                tool_name: Some(tool_context.tool_metadata.name.clone()),
                                source: None,
                            });
                        }
                    }
                }
                Err(e) => {
                    warn!(
                        "Hook execution failed for phase {:?}: {}",
                        tool_context.execution_phase, e
                    );
                    // Continue execution - hooks should not break tool functionality
                }
            }
        }

        let hook_duration = hook_start.elapsed();

        debug!(
            "Hook phase {:?} completed in {:?} with {} resource metrics",
            tool_context.execution_phase,
            hook_duration,
            tool_context.resource_metrics.len()
        );

        // Check if hook execution is taking too long
        if hook_duration > self.config.max_hook_execution_time {
            tracing::warn!(
                "Hook phase {:?} took {:?}, exceeding max time of {:?}",
                tool_context.execution_phase,
                hook_duration,
                self.config.max_hook_execution_time
            );
        }

        // Return input data unchanged for now
        Ok(input_data)
    }

    /// Validate security level for tool execution
    fn validate_tool_security(&self, security_level: &SecurityLevel) -> Result<(), LLMSpellError> {
        if !self.config.features.security_validation_enabled {
            return Ok(());
        }

        if !self.config.max_security_level.allows(security_level) {
            error!(
                "Security validation failed: Tool requires {:?} but maximum allowed is {:?}",
                security_level, self.config.max_security_level
            );
            return Err(LLMSpellError::Security {
                message: format!(
                    "Tool security level {:?} exceeds maximum allowed level {:?}",
                    security_level, self.config.max_security_level
                ),
                violation_type: Some("security_level_exceeded".to_string()),
            });
        }

        debug!(
            "Security validation passed: Tool level {:?} allowed by maximum {:?}",
            security_level, self.config.max_security_level
        );
        Ok(())
    }

    /// Create audit log entry for tool execution
    fn create_audit_log_entry(
        &self,
        tool_name: &str,
        security_level: SecurityLevel,
        execution_phase: &ToolExecutionPhase,
        input: &AgentInput,
        resource_metrics: &HashMap<String, JsonValue>,
    ) -> AuditLogEntry {
        let parameters = if self.config.audit.log_parameters {
            Some(input.parameters.clone())
        } else {
            Some({
                let mut safe_params = HashMap::new();
                // Only log non-sensitive parameter keys, not values
                for key in input.parameters.keys() {
                    safe_params.insert(key.clone(), JsonValue::String("[REDACTED]".to_string()));
                }
                safe_params
            })
        };

        AuditLogEntry {
            timestamp: Instant::now(),
            tool_name: tool_name.to_string(),
            security_level,
            execution_phase: execution_phase.clone(),
            input_text: input.text.clone(),
            parameters,
            resource_metrics: resource_metrics.clone(),
            success: None, // Will be set later
            error_message: None,
        }
    }

    /// Log audit entry
    fn log_audit_entry(&self, entry: &AuditLogEntry) {
        if !self.config.audit.enabled {
            return;
        }

        let success_status = entry.success.map_or("PENDING".to_string(), |s| {
            if s {
                "SUCCESS".to_string()
            } else {
                "FAILURE".to_string()
            }
        });

        info!(
            "AUDIT: tool={} security_level={:?} phase={:?} status={} memory_mb={} cpu_ms={}",
            entry.tool_name,
            entry.security_level,
            entry.execution_phase,
            success_status,
            entry
                .resource_metrics
                .get("memory_bytes")
                .and_then(serde_json::Value::as_u64)
                .unwrap_or(0)
                / (1024 * 1024),
            entry
                .resource_metrics
                .get("cpu_time_ms")
                .and_then(serde_json::Value::as_u64)
                .unwrap_or(0)
        );

        if let Some(ref error) = entry.error_message {
            warn!("AUDIT: tool={} error=\"{}\"", entry.tool_name, error);
        }
    }
}

/// Execution metrics for tool runs with hooks
#[derive(Debug, Clone, Default)]
pub struct ExecutionMetrics {
    /// Total number of tool executions
    pub total_executions: u64,
    /// Total hook execution overhead in milliseconds
    pub hook_overhead_ms: u64,
    /// Number of times resource limits were hit
    pub resource_limits_hit: u64,
    /// Average memory usage in bytes
    pub average_memory_usage: usize,
    /// Average CPU time in milliseconds
    pub average_cpu_time: u64,
}

/// Audit log entry for tool execution
#[derive(Debug, Clone)]
pub struct AuditLogEntry {
    /// Timestamp of the entry
    pub timestamp: Instant,
    /// Tool name
    pub tool_name: String,
    /// Security level of the tool
    pub security_level: SecurityLevel,
    /// Execution phase when this entry was created
    pub execution_phase: ToolExecutionPhase,
    /// Input text
    pub input_text: String,
    /// Input parameters (may be redacted)
    pub parameters: Option<HashMap<String, JsonValue>>,
    /// Resource metrics at time of entry
    pub resource_metrics: HashMap<String, JsonValue>,
    /// Success status (None for pending, Some(bool) for completed)
    pub success: Option<bool>,
    /// Error message if any
    pub error_message: Option<String>,
}

/// Trait for tools that support enhanced hook execution
#[async_trait]
pub trait HookableToolExecution: Tool {
    /// Execute with hook integration
    async fn execute_with_hooks(
        &self,
        input: AgentInput,
        context: ExecutionContext,
        tool_executor: &ToolExecutor,
    ) -> Result<AgentOutput, LLMSpellError>;
}

// Blanket implementation for all tools
#[async_trait]
impl<T: Tool> HookableToolExecution for T {
    async fn execute_with_hooks(
        &self,
        input: AgentInput,
        context: ExecutionContext,
        tool_executor: &ToolExecutor,
    ) -> Result<AgentOutput, LLMSpellError> {
        tool_executor
            .execute_tool_with_hooks(self, input, context)
            .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use llmspell_core::traits::base_agent::BaseAgent;
    use llmspell_core::traits::tool::{ParameterDef, ParameterType, ToolSchema};

    // Mock tool for testing
    struct MockTool {
        metadata: ComponentMetadata,
    }

    impl MockTool {
        fn new() -> Self {
            Self {
                metadata: ComponentMetadata::new(
                    "mock_tool".to_string(),
                    "A mock tool for testing".to_string(),
                ),
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
            input: AgentInput,
            _context: ExecutionContext,
        ) -> Result<AgentOutput, LLMSpellError> {
            Ok(AgentOutput::text(format!("Processed: {}", input.text)))
        }

        async fn validate_input(&self, _input: &AgentInput) -> Result<(), LLMSpellError> {
            Ok(())
        }

        async fn handle_error(&self, error: LLMSpellError) -> Result<AgentOutput, LLMSpellError> {
            Ok(AgentOutput::text(format!("Error handled: {}", error)))
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
            ToolSchema::new("mock_tool".to_string(), "Mock tool".to_string())
                .with_parameter(ParameterDef {
                    name: "input".to_string(),
                    param_type: ParameterType::String,
                    description: "Input parameter".to_string(),
                    required: true,
                    default: None,
                })
                .with_returns(ParameterType::String)
        }
    }
    #[tokio::test]
    async fn test_tool_executor_creation() {
        let config = ToolLifecycleConfig::default();
        let executor = ToolExecutor::new(config, None, None);

        // Just verify it was created successfully
        assert!(!executor.component_id.name.is_empty());
    }
    #[tokio::test]
    async fn test_tool_execution_without_hooks() {
        let config = ToolLifecycleConfig {
            features: HookFeatures {
                hooks_enabled: false,
                ..Default::default()
            },
            ..Default::default()
        };
        let executor = ToolExecutor::new(config, None, None);
        let tool = MockTool::new();

        let input = AgentInput::text("test input");
        let context = ExecutionContext::default();

        let result = executor
            .execute_tool_with_hooks(&tool, input, context)
            .await;
        assert!(result.is_ok());

        let output = result.unwrap();
        assert!(output.text.contains("Processed: test input"));
    }
    #[tokio::test]
    async fn test_tool_hook_context_creation() {
        let component_id = ComponentId::new(ComponentType::Tool, "test_tool".to_string());
        let metadata = ComponentMetadata::new("test".to_string(), "Test tool".to_string());

        let context = ToolHookContext::new(
            component_id,
            metadata,
            ToolCategory::Utility,
            SecurityLevel::Safe,
            ToolExecutionPhase::PreExecution,
        );

        assert_eq!(context.tool_category, ToolCategory::Utility);
        assert_eq!(context.security_level, SecurityLevel::Safe);
        assert_eq!(context.get_hook_point(), HookPoint::BeforeToolExecution);
    }
    #[tokio::test]
    async fn test_hook_point_mapping() {
        let component_id = ComponentId::new(ComponentType::Tool, "test_tool".to_string());
        let metadata = ComponentMetadata::new("test".to_string(), "Test tool".to_string());

        // Test all phases
        let phases_and_points = vec![
            (
                ToolExecutionPhase::PreExecution,
                HookPoint::BeforeToolExecution,
            ),
            (
                ToolExecutionPhase::PostExecution,
                HookPoint::AfterToolExecution,
            ),
            (
                ToolExecutionPhase::ParameterValidation,
                HookPoint::Custom("tool_parameter_validation".to_string()),
            ),
            (
                ToolExecutionPhase::SecurityCheck,
                HookPoint::Custom("tool_security_check".to_string()),
            ),
            (
                ToolExecutionPhase::ResourceAllocation,
                HookPoint::Custom("tool_resource_allocated".to_string()),
            ),
            (
                ToolExecutionPhase::ResourceCleanup,
                HookPoint::Custom("tool_resource_released".to_string()),
            ),
            (ToolExecutionPhase::ErrorHandling, HookPoint::ToolError),
            (
                ToolExecutionPhase::Timeout,
                HookPoint::Custom("tool_timeout".to_string()),
            ),
        ];

        for (phase, expected_point) in phases_and_points {
            let context = ToolHookContext::new(
                component_id.clone(),
                metadata.clone(),
                ToolCategory::Utility,
                SecurityLevel::Safe,
                phase,
            );

            assert_eq!(context.get_hook_point(), expected_point);
        }
    }
    #[tokio::test]
    async fn test_resource_metrics_integration() {
        use llmspell_utils::resource_limits::{ResourceLimits, ResourceTracker};

        let component_id = ComponentId::new(ComponentType::Tool, "test_tool".to_string());
        let metadata = ComponentMetadata::new("test".to_string(), "Test tool".to_string());

        // Create resource tracker with some usage
        let limits = ResourceLimits::default();
        let tracker = ResourceTracker::new(limits);

        // Simulate some resource usage
        tracker.track_operation().unwrap();
        tracker.track_memory(1024).unwrap();

        // Create hook context with resource metrics
        let context = ToolHookContext::new(
            component_id,
            metadata,
            ToolCategory::Utility,
            SecurityLevel::Safe,
            ToolExecutionPhase::ResourceAllocation,
        )
        .with_resource_tracker_metrics(&tracker);

        // Verify resource metrics are populated
        assert!(context.resource_metrics.contains_key("memory_bytes"));
        assert!(context.resource_metrics.contains_key("cpu_time_ms"));
        assert!(context.resource_metrics.contains_key("operations_count"));
        assert!(context.resource_metrics.contains_key("concurrent_ops"));

        // Verify values are reasonable
        assert!(context.resource_metrics["memory_bytes"].as_u64().unwrap() >= 1024);
        assert!(
            context.resource_metrics["operations_count"]
                .as_u64()
                .unwrap()
                >= 1
        );
    }
    #[tokio::test]
    async fn test_execution_metrics_structure() {
        let metrics = ExecutionMetrics::default();

        assert_eq!(metrics.total_executions, 0);
        assert_eq!(metrics.hook_overhead_ms, 0);
        assert_eq!(metrics.resource_limits_hit, 0);
        assert_eq!(metrics.average_memory_usage, 0);
        assert_eq!(metrics.average_cpu_time, 0);
    }
    #[tokio::test]
    async fn test_tool_executor_with_resource_tracking() {
        let config = ToolLifecycleConfig::default();
        let executor = ToolExecutor::new(config, None, None);
        let tool = MockTool::new();

        let input = AgentInput::text("test input with resource tracking");
        let context = ExecutionContext::default();

        // Execute tool - this should track resource usage through the lifecycle
        let result = executor
            .execute_tool_with_hooks(&tool, input, context)
            .await;

        assert!(result.is_ok());

        // Get execution metrics (currently returns defaults)
        let metrics = executor.get_execution_metrics();
        assert_eq!(metrics.total_executions, 0); // TODO: This will be tracked in future
    }
    #[tokio::test]
    async fn test_security_validation_pass() {
        let config = ToolLifecycleConfig {
            max_security_level: SecurityLevel::Privileged,
            ..Default::default()
        };
        let executor = ToolExecutor::new(config, None, None);
        let tool = MockTool::new(); // Safe security level

        let input = AgentInput::text("test security validation");
        let context = ExecutionContext::default();

        let result = executor
            .execute_tool_with_hooks(&tool, input, context)
            .await;
        assert!(
            result.is_ok(),
            "Safe tool should pass Privileged security validation"
        );
    }
    #[tokio::test]
    async fn test_security_validation_fail() {
        // Create a restricted security config
        let config = ToolLifecycleConfig {
            max_security_level: SecurityLevel::Safe,
            ..Default::default()
        };
        let executor = ToolExecutor::new(config, None, None);

        // Create a mock tool with higher security level
        struct PrivilegedMockTool {
            metadata: ComponentMetadata,
        }

        impl PrivilegedMockTool {
            fn new() -> Self {
                Self {
                    metadata: ComponentMetadata::new(
                        "privileged_tool".to_string(),
                        "A privileged mock tool".to_string(),
                    ),
                }
            }
        }

        #[async_trait]
        impl BaseAgent for PrivilegedMockTool {
            fn metadata(&self) -> &ComponentMetadata {
                &self.metadata
            }

            async fn execute(
                &self,
                input: AgentInput,
                _context: ExecutionContext,
            ) -> Result<AgentOutput, LLMSpellError> {
                Ok(AgentOutput::text(format!("Processed: {}", input.text)))
            }

            async fn validate_input(&self, _input: &AgentInput) -> Result<(), LLMSpellError> {
                Ok(())
            }

            async fn handle_error(
                &self,
                error: LLMSpellError,
            ) -> Result<AgentOutput, LLMSpellError> {
                Ok(AgentOutput::text(format!("Error handled: {}", error)))
            }
        }

        #[async_trait]
        impl Tool for PrivilegedMockTool {
            fn category(&self) -> ToolCategory {
                ToolCategory::System
            }

            fn security_level(&self) -> SecurityLevel {
                SecurityLevel::Privileged // Higher than allowed Safe level
            }

            fn schema(&self) -> ToolSchema {
                ToolSchema::new("privileged_tool".to_string(), "Privileged tool".to_string())
                    .with_parameter(ParameterDef {
                        name: "input".to_string(),
                        param_type: ParameterType::String,
                        description: "Input parameter".to_string(),
                        required: true,
                        default: None,
                    })
                    .with_returns(ParameterType::String)
            }
        }

        let tool = PrivilegedMockTool::new();
        let input = AgentInput::text("test security validation failure");
        let context = ExecutionContext::default();

        let result = executor
            .execute_tool_with_hooks(&tool, input, context)
            .await;
        assert!(
            result.is_err(),
            "Privileged tool should fail Safe security validation"
        );

        if let Err(e) = result {
            assert!(e.to_string().contains("security level"));
        }
    }
    #[tokio::test]
    async fn test_audit_logging_enabled() {
        let config = ToolLifecycleConfig {
            audit: AuditConfig {
                enabled: true,
                log_parameters: false, // Don't log parameters for security
            },
            ..Default::default()
        };
        let executor = ToolExecutor::new(config, None, None);
        let tool = MockTool::new();

        let input = AgentInput::text("test audit logging");
        let context = ExecutionContext::default();

        // This should generate audit log entries
        let result = executor
            .execute_tool_with_hooks(&tool, input, context)
            .await;
        assert!(result.is_ok());

        // We can't easily test the audit log output in a unit test,
        // but we can verify the execution completed without errors
    }
    #[tokio::test]
    async fn test_audit_logging_disabled() {
        let config = ToolLifecycleConfig {
            audit: AuditConfig {
                enabled: false,
                ..Default::default()
            },
            ..Default::default()
        };
        let executor = ToolExecutor::new(config, None, None);
        let tool = MockTool::new();

        let input = AgentInput::text("test audit logging disabled");
        let context = ExecutionContext::default();

        // Should execute normally without audit logging
        let result = executor
            .execute_tool_with_hooks(&tool, input, context)
            .await;
        assert!(result.is_ok());
    }
    #[tokio::test]
    async fn test_security_validation_disabled() {
        let config = ToolLifecycleConfig {
            features: HookFeatures {
                security_validation_enabled: false,
                ..Default::default()
            },
            max_security_level: SecurityLevel::Safe, // Restrictive, but disabled
            ..Default::default()
        };
        let executor = ToolExecutor::new(config, None, None);

        // Use the privileged tool from the previous test
        struct PrivilegedMockTool {
            metadata: ComponentMetadata,
        }

        impl PrivilegedMockTool {
            fn new() -> Self {
                Self {
                    metadata: ComponentMetadata::new(
                        "privileged_tool".to_string(),
                        "A privileged mock tool".to_string(),
                    ),
                }
            }
        }

        #[async_trait]
        impl BaseAgent for PrivilegedMockTool {
            fn metadata(&self) -> &ComponentMetadata {
                &self.metadata
            }

            async fn execute(
                &self,
                input: AgentInput,
                _context: ExecutionContext,
            ) -> Result<AgentOutput, LLMSpellError> {
                Ok(AgentOutput::text(format!("Processed: {}", input.text)))
            }

            async fn validate_input(&self, _input: &AgentInput) -> Result<(), LLMSpellError> {
                Ok(())
            }

            async fn handle_error(
                &self,
                error: LLMSpellError,
            ) -> Result<AgentOutput, LLMSpellError> {
                Ok(AgentOutput::text(format!("Error handled: {}", error)))
            }
        }

        #[async_trait]
        impl Tool for PrivilegedMockTool {
            fn category(&self) -> ToolCategory {
                ToolCategory::System
            }

            fn security_level(&self) -> SecurityLevel {
                SecurityLevel::Privileged
            }

            fn schema(&self) -> ToolSchema {
                ToolSchema::new("privileged_tool".to_string(), "Privileged tool".to_string())
                    .with_parameter(ParameterDef {
                        name: "input".to_string(),
                        param_type: ParameterType::String,
                        description: "Input parameter".to_string(),
                        required: true,
                        default: None,
                    })
                    .with_returns(ParameterType::String)
            }
        }

        let tool = PrivilegedMockTool::new();
        let input = AgentInput::text("test security validation disabled");
        let context = ExecutionContext::default();

        // Should succeed because security validation is disabled
        let result = executor
            .execute_tool_with_hooks(&tool, input, context)
            .await;
        assert!(
            result.is_ok(),
            "Should succeed when security validation is disabled"
        );
    }
}
