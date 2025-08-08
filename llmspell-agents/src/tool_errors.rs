//! ABOUTME: Comprehensive error handling for tool operations and agent-tool integration
//! ABOUTME: Provides structured error types, recovery strategies, and error context propagation

#![allow(clippy::significant_drop_tightening)]

use llmspell_core::{LLMSpellError, Result};
use serde_json::Value as JsonValue;
use std::collections::HashMap;
use std::fmt;
use std::time::Duration;

/// Comprehensive error type for tool integration operations
#[derive(Debug, Clone)]
pub enum ToolIntegrationError {
    /// Tool not found in registry
    ToolNotFound {
        tool_name: String,
        available_tools: Vec<String>,
    },
    /// Tool registration failed
    RegistrationFailed { tool_name: String, reason: String },
    /// Tool discovery failed
    DiscoveryFailed { query: String, reason: String },
    /// Tool invocation failed
    InvocationFailed {
        tool_name: String,
        parameters: JsonValue,
        error: String,
        retry_count: u32,
    },
    /// Parameter validation failed
    ParameterValidation {
        tool_name: String,
        parameter_name: String,
        expected_type: String,
        actual_value: JsonValue,
        validation_errors: Vec<String>,
    },
    /// Tool timeout occurred
    Timeout {
        tool_name: String,
        duration: Duration,
        max_allowed: Duration,
    },
    /// Tool composition failed
    CompositionFailed {
        composition_id: String,
        failed_step: String,
        step_errors: Vec<StepError>,
        partial_results: HashMap<String, JsonValue>,
    },
    /// Agent wrapping failed
    AgentWrappingFailed { agent_name: String, reason: String },
    /// Context propagation failed
    ContextPropagationFailed {
        context_id: String,
        propagation_type: String,
        reason: String,
    },
    /// Resource limit exceeded
    ResourceLimitExceeded {
        tool_name: String,
        resource_type: String,
        limit: u64,
        actual: u64,
    },
    /// Security constraint violation
    SecurityViolation {
        tool_name: String,
        security_level: String,
        violation_type: String,
        details: String,
    },
    /// Tool dependency resolution failed
    DependencyResolution {
        tool_name: String,
        missing_dependencies: Vec<String>,
        dependency_errors: Vec<String>,
    },
    /// Tool state corruption
    StateCorruption {
        tool_name: String,
        state_type: String,
        recovery_attempted: bool,
    },
}

/// Error that occurred during a composition step
#[derive(Debug, Clone)]
pub struct StepError {
    /// Step identifier
    pub step_id: String,
    /// Tool name that failed
    pub tool_name: String,
    /// Error message
    pub error: String,
    /// Whether the error is recoverable
    pub recoverable: bool,
    /// Suggested recovery actions
    pub recovery_suggestions: Vec<RecoveryAction>,
}

/// Suggested recovery actions for errors
#[derive(Debug, Clone)]
pub enum RecoveryAction {
    /// Retry with same parameters
    Retry { max_attempts: u32, delay: Duration },
    /// Retry with modified parameters
    RetryWithModifiedParams {
        parameter_modifications: HashMap<String, JsonValue>,
    },
    /// Skip this step and continue
    Skip,
    /// Use fallback tool
    UseFallback { fallback_tool: String },
    /// Use default value
    UseDefault { default_value: JsonValue },
    /// Request user intervention
    RequestUserIntervention {
        intervention_type: String,
        message: String,
    },
    /// Abort operation
    Abort,
}

/// Error recovery strategy
#[derive(Debug, Clone)]
pub enum ErrorRecoveryStrategy {
    /// Fail immediately on any error
    FailFast,
    /// Try to recover using suggested actions
    AttemptRecovery {
        max_recovery_attempts: u32,
        recovery_timeout: Duration,
    },
    /// Continue with best effort
    BestEffort,
    /// Collect all errors and report at end
    CollectErrors,
    /// Custom recovery strategy
    Custom(String),
}

/// Context for error handling and recovery
#[derive(Debug, Clone)]
pub struct ErrorContext {
    /// Operation being performed
    pub operation: String,
    /// Tool or component involved
    pub component: String,
    /// Execution context identifier
    pub context_id: String,
    /// Error recovery strategy
    pub recovery_strategy: ErrorRecoveryStrategy,
    /// Additional context data
    pub context_data: HashMap<String, JsonValue>,
    /// Error history for this context
    pub error_history: Vec<ErrorRecord>,
}

/// Record of an error that occurred
#[derive(Debug, Clone)]
pub struct ErrorRecord {
    /// When the error occurred
    pub timestamp: chrono::DateTime<chrono::Utc>,
    /// The error that occurred
    pub error: ToolIntegrationError,
    /// Recovery action taken
    pub recovery_action: Option<RecoveryAction>,
    /// Whether recovery was successful
    pub recovery_successful: bool,
}

/// Error callback function type
pub type ErrorCallback = Box<dyn Fn(&ToolIntegrationError, &ErrorContext) + Send + Sync>;

/// Recovery callback function type  
pub type RecoveryCallback = Box<dyn Fn(&RecoveryAction, &ErrorContext) -> bool + Send + Sync>;

/// Error handler for tool integration operations
pub struct ToolErrorHandler {
    /// Default recovery strategy
    default_strategy: ErrorRecoveryStrategy,
    /// Tool-specific recovery strategies
    tool_strategies: HashMap<String, ErrorRecoveryStrategy>,
    /// Error callbacks for monitoring
    error_callbacks: Vec<ErrorCallback>,
    /// Recovery callbacks
    recovery_callbacks: Vec<RecoveryCallback>,
}

impl ToolIntegrationError {
    /// Get error severity level
    #[must_use]
    pub const fn severity(&self) -> ErrorSeverity {
        match self {
            Self::SecurityViolation { .. } | Self::StateCorruption { .. } => {
                ErrorSeverity::Critical
            }
            Self::ToolNotFound { .. }
            | Self::InvocationFailed { .. }
            | Self::CompositionFailed { .. }
            | Self::ResourceLimitExceeded { .. }
            | Self::DependencyResolution { .. } => ErrorSeverity::High,
            Self::RegistrationFailed { .. }
            | Self::ParameterValidation { .. }
            | Self::Timeout { .. }
            | Self::AgentWrappingFailed { .. } => ErrorSeverity::Medium,
            Self::DiscoveryFailed { .. } | Self::ContextPropagationFailed { .. } => {
                ErrorSeverity::Low
            }
        }
    }

    /// Check if error is recoverable
    #[must_use]
    pub fn is_recoverable(&self) -> bool {
        match self {
            Self::ToolNotFound { .. }
            | Self::RegistrationFailed { .. }
            | Self::AgentWrappingFailed { .. }
            | Self::SecurityViolation { .. } => false,
            Self::DiscoveryFailed { .. }
            | Self::InvocationFailed { .. }
            | Self::ParameterValidation { .. }
            | Self::Timeout { .. }
            | Self::CompositionFailed { .. }
            | Self::ContextPropagationFailed { .. }
            | Self::ResourceLimitExceeded { .. }
            | Self::DependencyResolution { .. } => true,
            Self::StateCorruption {
                recovery_attempted, ..
            } => !recovery_attempted,
        }
    }

    /// Get suggested recovery actions
    #[must_use]
    pub fn suggested_recovery_actions(&self) -> Vec<RecoveryAction> {
        match self {
            Self::ToolNotFound {
                available_tools, ..
            } => {
                if available_tools.is_empty() {
                    vec![RecoveryAction::Abort]
                } else {
                    vec![RecoveryAction::UseFallback {
                        fallback_tool: available_tools[0].clone(),
                    }]
                }
            }
            Self::InvocationFailed { retry_count, .. } => {
                if *retry_count < 3 {
                    vec![RecoveryAction::Retry {
                        max_attempts: 3,
                        #[allow(clippy::cast_lossless)]
                        delay: Duration::from_millis(1000 * (retry_count + 1) as u64),
                    }]
                } else {
                    vec![RecoveryAction::RequestUserIntervention {
                        intervention_type: "tool_invocation_failure".to_string(),
                        message: "Tool invocation has failed multiple times".to_string(),
                    }]
                }
            }
            Self::ParameterValidation { .. } => {
                vec![RecoveryAction::RequestUserIntervention {
                    intervention_type: "parameter_correction".to_string(),
                    message: "Parameter validation failed, manual correction needed".to_string(),
                }]
            }
            Self::Timeout { .. } => {
                vec![RecoveryAction::Retry {
                    max_attempts: 2,
                    delay: Duration::from_secs(1),
                }]
            }
            Self::CompositionFailed { .. } => {
                vec![
                    RecoveryAction::Skip,
                    RecoveryAction::UseDefault {
                        default_value: JsonValue::Null,
                    },
                ]
            }
            Self::ResourceLimitExceeded { .. } => {
                vec![RecoveryAction::RequestUserIntervention {
                    intervention_type: "resource_limit_adjustment".to_string(),
                    message: "Resource limits exceeded, consider increasing limits".to_string(),
                }]
            }
            Self::SecurityViolation { .. } => {
                vec![RecoveryAction::Abort]
            }
            Self::DependencyResolution {
                missing_dependencies,
                ..
            } => {
                vec![RecoveryAction::RequestUserIntervention {
                    intervention_type: "dependency_installation".to_string(),
                    message: format!("Missing dependencies: {}", missing_dependencies.join(", ")),
                }]
            }
            _ => vec![RecoveryAction::Abort],
        }
    }

    /// Convert to `LLMSpellError`
    #[must_use]
    pub fn into_llmspell_error(self) -> LLMSpellError {
        match self {
            Self::ToolNotFound { tool_name, .. } => LLMSpellError::Component {
                message: format!("Tool not found: {tool_name}"),
                source: None,
            },
            Self::ParameterValidation {
                parameter_name,
                expected_type,
                ..
            } => LLMSpellError::Validation {
                message: format!(
                    "Parameter '{parameter_name}' validation failed, expected {expected_type}"
                ),
                field: Some(parameter_name),
            },
            Self::Timeout {
                tool_name,
                duration,
                ..
            } => LLMSpellError::Timeout {
                message: format!("Tool execution: {tool_name} timed out"),
                duration_ms: Some(duration.as_millis() as u64),
            },
            Self::SecurityViolation {
                tool_name,
                violation_type,
                details,
                ..
            } => LLMSpellError::Security {
                message: format!("Security violation in {tool_name}: {violation_type} - {details}"),
                violation_type: Some(violation_type),
            },
            _ => LLMSpellError::Component {
                message: self.to_string(),
                source: None,
            },
        }
    }
}

/// Error severity levels
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum ErrorSeverity {
    Low,
    Medium,
    High,
    Critical,
}

impl fmt::Display for ToolIntegrationError {
    #[allow(clippy::too_many_lines)]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ToolNotFound {
                tool_name,
                available_tools,
            } => {
                write!(
                    f,
                    "Tool '{}' not found. Available tools: {}",
                    tool_name,
                    available_tools.join(", ")
                )
            }
            Self::RegistrationFailed { tool_name, reason } => {
                write!(f, "Failed to register tool '{tool_name}': {reason}")
            }
            Self::DiscoveryFailed { query, reason } => {
                write!(f, "Tool discovery failed for query '{query}': {reason}")
            }
            Self::InvocationFailed {
                tool_name,
                error,
                retry_count,
                ..
            } => {
                write!(
                    f,
                    "Tool '{}' invocation failed (attempt {}): {}",
                    tool_name,
                    retry_count + 1,
                    error
                )
            }
            Self::ParameterValidation {
                tool_name,
                parameter_name,
                expected_type,
                ..
            } => {
                write!(
                    f,
                    "Parameter validation failed for tool '{tool_name}': parameter '{parameter_name}' should be {expected_type}"
                )
            }
            Self::Timeout {
                tool_name,
                duration,
                max_allowed,
            } => {
                write!(
                    f,
                    "Tool '{tool_name}' execution timed out after {duration:?} (max allowed: {max_allowed:?})"
                )
            }
            Self::CompositionFailed {
                composition_id,
                failed_step,
                ..
            } => {
                write!(
                    f,
                    "Composition '{composition_id}' failed at step '{failed_step}'"
                )
            }
            Self::AgentWrappingFailed { agent_name, reason } => {
                write!(f, "Failed to wrap agent '{agent_name}' as tool: {reason}")
            }
            Self::ContextPropagationFailed {
                context_id,
                propagation_type,
                reason,
            } => {
                write!(
                    f,
                    "Context propagation failed for '{context_id}' (type: {propagation_type}): {reason}"
                )
            }
            Self::ResourceLimitExceeded {
                tool_name,
                resource_type,
                limit,
                actual,
            } => {
                write!(
                    f,
                    "Tool '{tool_name}' exceeded {resource_type} limit: {actual} > {limit}"
                )
            }
            Self::SecurityViolation {
                tool_name,
                violation_type,
                details,
                ..
            } => {
                write!(
                    f,
                    "Security violation in tool '{tool_name}': {violation_type} - {details}"
                )
            }
            Self::DependencyResolution {
                tool_name,
                missing_dependencies,
                ..
            } => {
                write!(
                    f,
                    "Tool '{}' has unresolved dependencies: {}",
                    tool_name,
                    missing_dependencies.join(", ")
                )
            }
            Self::StateCorruption {
                tool_name,
                state_type,
                recovery_attempted,
            } => {
                write!(
                    f,
                    "Tool '{tool_name}' state corruption detected in {state_type} (recovery attempted: {recovery_attempted})"
                )
            }
        }
    }
}

impl std::error::Error for ToolIntegrationError {}

impl ErrorContext {
    /// Create new error context
    pub fn new(operation: impl Into<String>, component: impl Into<String>) -> Self {
        Self {
            operation: operation.into(),
            component: component.into(),
            context_id: uuid::Uuid::new_v4().to_string(),
            recovery_strategy: ErrorRecoveryStrategy::FailFast,
            context_data: HashMap::new(),
            error_history: Vec::new(),
        }
    }

    /// Set recovery strategy
    #[must_use]
    pub fn with_recovery_strategy(mut self, strategy: ErrorRecoveryStrategy) -> Self {
        self.recovery_strategy = strategy;
        self
    }

    /// Add context data
    #[must_use]
    pub fn with_data(mut self, key: impl Into<String>, value: JsonValue) -> Self {
        self.context_data.insert(key.into(), value);
        self
    }

    /// Record an error in the context
    pub fn record_error(
        &mut self,
        error: ToolIntegrationError,
        recovery_action: Option<RecoveryAction>,
        recovery_successful: bool,
    ) {
        self.error_history.push(ErrorRecord {
            timestamp: chrono::Utc::now(),
            error,
            recovery_action,
            recovery_successful,
        });
    }

    /// Get error count for this context
    #[must_use]
    pub fn error_count(&self) -> usize {
        self.error_history.len()
    }

    /// Get successful recovery count
    #[must_use]
    pub fn successful_recovery_count(&self) -> usize {
        self.error_history
            .iter()
            .filter(|record| record.recovery_successful)
            .count()
    }
}

impl ToolErrorHandler {
    /// Create new error handler
    #[must_use]
    pub fn new() -> Self {
        Self {
            default_strategy: ErrorRecoveryStrategy::FailFast,
            tool_strategies: HashMap::new(),
            error_callbacks: Vec::new(),
            recovery_callbacks: Vec::new(),
        }
    }

    /// Set default recovery strategy
    #[must_use]
    pub fn with_default_strategy(mut self, strategy: ErrorRecoveryStrategy) -> Self {
        self.default_strategy = strategy;
        self
    }

    /// Set tool-specific recovery strategy
    #[must_use]
    pub fn with_tool_strategy(
        mut self,
        tool_name: impl Into<String>,
        strategy: ErrorRecoveryStrategy,
    ) -> Self {
        self.tool_strategies.insert(tool_name.into(), strategy);
        self
    }

    /// Handle a tool integration error
    pub async fn handle_error(
        &self,
        error: ToolIntegrationError,
        context: &mut ErrorContext,
    ) -> Result<Option<JsonValue>> {
        // Record the error
        context.record_error(error.clone(), None, false);

        // Notify error callbacks
        for callback in &self.error_callbacks {
            callback(&error, context);
        }

        // Get recovery strategy for this component
        let strategy = self
            .tool_strategies
            .get(&context.component)
            .unwrap_or(&self.default_strategy);

        match strategy {
            ErrorRecoveryStrategy::FailFast => Err(error.into_llmspell_error()),
            ErrorRecoveryStrategy::AttemptRecovery {
                max_recovery_attempts,
                recovery_timeout,
            } => {
                self.attempt_recovery(error, context, *max_recovery_attempts, *recovery_timeout)
                    .await
            }
            ErrorRecoveryStrategy::BestEffort => {
                tracing::warn!("Error occurred but continuing with best effort: {}", error);
                Ok(Some(JsonValue::Null))
            }
            ErrorRecoveryStrategy::CollectErrors => {
                tracing::warn!("Error collected for later reporting: {}", error);
                Ok(None)
            }
            ErrorRecoveryStrategy::Custom(_strategy_name) => {
                // Custom strategies would be implemented here
                tracing::warn!("Custom recovery strategy not implemented, failing fast");
                Err(error.into_llmspell_error())
            }
        }
    }

    /// Attempt recovery from an error
    async fn attempt_recovery(
        &self,
        error: ToolIntegrationError,
        context: &mut ErrorContext,
        max_attempts: u32,
        timeout: Duration,
    ) -> Result<Option<JsonValue>> {
        let recovery_actions = error.suggested_recovery_actions();

        for (attempt, action) in recovery_actions.iter().enumerate() {
            if attempt >= max_attempts as usize {
                break;
            }

            // Check if recovery callback approves this action
            let approved = self
                .recovery_callbacks
                .iter()
                .all(|callback| callback(action, context));

            if !approved {
                continue;
            }

            tracing::info!("Attempting recovery action: {:?}", action);

            match self.execute_recovery_action(action, context, timeout).await {
                Ok(result) => {
                    context.record_error(error.clone(), Some(action.clone()), true);
                    tracing::info!("Recovery successful");
                    return Ok(Some(result));
                }
                Err(recovery_error) => {
                    context.record_error(error.clone(), Some(action.clone()), false);
                    tracing::warn!("Recovery action failed: {}", recovery_error);
                }
            }
        }

        // All recovery attempts failed
        Err(error.into_llmspell_error())
    }

    /// Execute a specific recovery action
    async fn execute_recovery_action(
        &self,
        action: &RecoveryAction,
        _context: &ErrorContext,
        _timeout: Duration,
    ) -> Result<JsonValue> {
        match action {
            RecoveryAction::Retry { delay, .. } => {
                tokio::time::sleep(*delay).await;
                // Return indication that retry should be attempted
                Ok(JsonValue::String("retry".to_string()))
            }
            RecoveryAction::Skip => Ok(JsonValue::String("skipped".to_string())),
            RecoveryAction::UseDefault { default_value } => Ok(default_value.clone()),
            RecoveryAction::UseFallback { fallback_tool } => {
                Ok(JsonValue::String(format!("fallback:{fallback_tool}")))
            }
            RecoveryAction::RetryWithModifiedParams {
                parameter_modifications,
            } => Ok(JsonValue::Object(
                parameter_modifications
                    .iter()
                    .map(|(k, v)| (k.clone(), v.clone()))
                    .collect(),
            )),
            RecoveryAction::RequestUserIntervention { message, .. } => {
                tracing::warn!("User intervention requested: {}", message);
                Err(LLMSpellError::Component {
                    message: format!("User intervention required: {message}"),
                    source: None,
                })
            }
            RecoveryAction::Abort => Err(LLMSpellError::Component {
                message: "Recovery action: abort".to_string(),
                source: None,
            }),
        }
    }
}

impl Default for ToolErrorHandler {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_error_severity() {
        let error = ToolIntegrationError::SecurityViolation {
            tool_name: "test_tool".to_string(),
            security_level: "safe".to_string(),
            violation_type: "unauthorized_access".to_string(),
            details: "Test violation".to_string(),
        };

        assert_eq!(error.severity(), ErrorSeverity::Critical);
        assert!(!error.is_recoverable());
    }
    #[test]
    fn test_recovery_actions() {
        let error = ToolIntegrationError::InvocationFailed {
            tool_name: "test_tool".to_string(),
            parameters: JsonValue::Null,
            error: "Test error".to_string(),
            retry_count: 1,
        };

        let actions = error.suggested_recovery_actions();
        assert!(!actions.is_empty());

        if let RecoveryAction::Retry { max_attempts, .. } = &actions[0] {
            assert_eq!(*max_attempts, 3);
        } else {
            panic!("Expected retry action");
        }
    }
    #[test]
    fn test_error_context() {
        let mut context = ErrorContext::new("test_operation", "test_component")
            .with_recovery_strategy(ErrorRecoveryStrategy::AttemptRecovery {
                max_recovery_attempts: 3,
                recovery_timeout: Duration::from_secs(10),
            })
            .with_data("test_key", JsonValue::String("test_value".to_string()));

        assert_eq!(context.error_count(), 0);

        let error = ToolIntegrationError::Timeout {
            tool_name: "test_tool".to_string(),
            duration: Duration::from_secs(5),
            max_allowed: Duration::from_secs(3),
        };

        context.record_error(error, None, false);
        assert_eq!(context.error_count(), 1);
        assert_eq!(context.successful_recovery_count(), 0);
    }
    #[test]
    fn test_error_display() {
        let error = ToolIntegrationError::ToolNotFound {
            tool_name: "missing_tool".to_string(),
            available_tools: vec!["tool1".to_string(), "tool2".to_string()],
        };

        let display_string = error.to_string();
        assert!(display_string.contains("missing_tool"));
        assert!(display_string.contains("tool1"));
        assert!(display_string.contains("tool2"));
    }
    #[tokio::test]
    async fn test_error_handler() {
        let handler =
            ToolErrorHandler::new().with_default_strategy(ErrorRecoveryStrategy::BestEffort);

        let mut context = ErrorContext::new("test_operation", "test_component");

        let error = ToolIntegrationError::DiscoveryFailed {
            query: "test query".to_string(),
            reason: "test reason".to_string(),
        };

        let result = handler.handle_error(error, &mut context).await.unwrap();
        assert!(result.is_some());
        assert_eq!(context.error_count(), 1);
    }
}
