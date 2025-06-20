# Complete Error Handling Strategy

## Overview

This document provides the complete implementation specification for rs-llmspell's comprehensive error handling strategy. It defines unified error types, recovery patterns, cross-engine error translation, async error handling, and production-ready error management across all architectural components.

## Hierarchical Error Architecture

### Core Error Hierarchy

```rust
// Root error type for the entire rs-llmspell system
#[derive(Debug, thiserror::Error)]
pub enum LLMSpellError {
    // Core component errors
    #[error("Agent error: {0}")]
    Agent(#[from] AgentError),
    
    #[error("Tool error: {0}")]
    Tool(#[from] ToolError),
    
    #[error("Workflow error: {0}")]
    Workflow(#[from] WorkflowError),
    
    // Infrastructure errors
    #[error("Hook error: {0}")]
    Hook(#[from] HookError),
    
    #[error("Event error: {0}")]
    Event(#[from] EventError),
    
    // Bridge and scripting errors
    #[error("Script error: {0}")]
    Script(#[from] ScriptError),
    
    #[error("Bridge error: {0}")]
    Bridge(#[from] BridgeError),
    
    // Async and coordination errors
    #[error("Async execution error: {0}")]
    AsyncExecution(#[from] AsyncExecutionError),
    
    #[error("Cross-engine coordination error: {0}")]
    CrossEngineCoordination(#[from] CrossEngineCoordinationError),
    
    // System and infrastructure errors
    #[error("Configuration error: {0}")]
    Configuration(#[from] ConfigurationError),
    
    #[error("Runtime error: {0}")]
    Runtime(#[from] RuntimeError),
    
    #[error("Security error: {0}")]
    Security(#[from] SecurityError),
    
    #[error("Resource error: {0}")]
    Resource(#[from] ResourceError),
    
    // External dependency errors
    #[error("LLM provider error: {0}")]
    Provider(#[from] ProviderError),
    
    #[error("Storage error: {0}")]
    Storage(#[from] StorageError),
    
    #[error("Network error: {0}")]
    Network(#[from] NetworkError),
}

// Agent-specific error types
#[derive(Debug, thiserror::Error)]
pub enum AgentError {
    #[error("Agent not found: {agent_id}")]
    NotFound { agent_id: String },
    
    #[error("Agent initialization failed: {reason}")]
    InitializationFailed { reason: String, agent_type: String },
    
    #[error("Agent execution failed: {reason}")]
    ExecutionFailed { 
        reason: String, 
        agent_id: String,
        execution_context: Option<String>
    },
    
    #[error("Invalid agent configuration: {field} - {reason}")]
    InvalidConfiguration { field: String, reason: String },
    
    #[error("Agent timeout after {duration:?}")]
    Timeout { 
        duration: Duration, 
        agent_id: String,
        operation: String
    },
    
    #[error("Agent memory limit exceeded: {used}MB > {limit}MB")]
    MemoryLimitExceeded { 
        used: u64, 
        limit: u64,
        agent_id: String
    },
    
    #[error("Tool execution error in agent {agent_id}: {tool_name} - {error}")]
    ToolExecutionError { 
        agent_id: String,
        tool_name: String, 
        error: String,
        tool_parameters: Option<serde_json::Value>
    },
    
    #[error("Agent context overflow: {current_tokens} > {max_tokens}")]
    ContextOverflow {
        agent_id: String,
        current_tokens: u32,
        max_tokens: u32,
    },
    
    #[error("Agent state corruption: {description}")]
    StateCorruption {
        agent_id: String,
        description: String,
        recovery_possible: bool,
    },
}

// Tool-specific error types
#[derive(Debug, thiserror::Error)]
pub enum ToolError {
    #[error("Tool not found: {tool_name}")]
    NotFound { tool_name: String },
    
    #[error("Tool execution failed: {tool_name} - {reason}")]
    ExecutionFailed { 
        tool_name: String, 
        reason: String,
        parameters: Option<serde_json::Value>,
        execution_time: Duration,
    },
    
    #[error("Invalid tool parameters: {tool_name} - {validation_error}")]
    InvalidParameters { 
        tool_name: String, 
        validation_error: String,
        received_parameters: serde_json::Value,
        expected_schema: Option<serde_json::Value>,
    },
    
    #[error("Tool timeout: {tool_name} after {duration:?}")]
    Timeout { 
        tool_name: String, 
        duration: Duration,
        partial_result: Option<serde_json::Value>,
    },
    
    #[error("Tool permission denied: {tool_name} requires {permission:?}")]
    PermissionDenied { 
        tool_name: String, 
        permission: Permission,
        security_context: Option<String>,
    },
    
    #[error("Tool resource limit exceeded: {tool_name} - {resource_type}: {used} > {limit}")]
    ResourceLimitExceeded {
        tool_name: String,
        resource_type: String,
        used: u64,
        limit: u64,
    },
    
    #[error("Tool chain execution failed at step {step_index}: {step_name} - {error}")]
    ChainExecutionFailed {
        step_index: usize,
        step_name: String,
        error: String,
        completed_steps: Vec<String>,
    },
    
    #[error("Tool dependency error: {tool_name} depends on {dependency} - {error}")]
    DependencyError {
        tool_name: String,
        dependency: String,
        error: String,
    },
}

// Script execution error types
#[derive(Debug, thiserror::Error)]
pub enum ScriptError {
    #[error("Lua error in {context}: {error}")]
    Lua { 
        error: mlua::Error, 
        context: String,
        script_line: Option<u32>,
        stack_trace: Option<String>,
    },
    
    #[error("JavaScript error in {context}: {error}")]
    JavaScript { 
        error: String, 
        context: String,
        script_line: Option<u32>,
        stack_trace: Option<String>,
    },
    
    #[error("Script compilation failed: {language} - {error}")]
    CompilationFailed { 
        language: ScriptLanguage,
        error: String,
        source_code: Option<String>,
    },
    
    #[error("Script runtime error: {language} - {error}")]
    RuntimeError { 
        language: ScriptLanguage,
        error: String,
        execution_context: Option<String>,
    },
    
    #[error("Cross-engine communication error: {from_engine} -> {to_engine} - {error}")]
    CrossEngineCommunication { 
        from_engine: ScriptLanguage,
        to_engine: ScriptLanguage,
        error: String,
        data_type: Option<String>,
    },
    
    #[error("Script security violation: {violation_type} in {context}")]
    SecurityViolation {
        violation_type: String,
        context: String,
        severity: SecuritySeverity,
    },
    
    #[error("Script resource exhaustion: {resource_type} in {language}")]
    ResourceExhaustion {
        language: ScriptLanguage,
        resource_type: String,
        current_usage: u64,
        limit: u64,
    },
}

// Async execution error types
#[derive(Debug, thiserror::Error)]
pub enum AsyncExecutionError {
    #[error("Task not found: {task_id}")]
    TaskNotFound { task_id: TaskId },
    
    #[error("Task execution failed: {task_id} - {reason}")]
    TaskFailed { 
        task_id: TaskId,
        reason: String,
        task_type: String,
        execution_time: Duration,
    },
    
    #[error("Task timeout: {task_id} after {duration:?}")]
    TaskTimeout { 
        task_id: TaskId,
        duration: Duration,
        task_type: String,
    },
    
    #[error("Task cancelled: {task_id} - {reason}")]
    TaskCancelled { 
        task_id: TaskId,
        reason: String,
        cancellation_source: String,
    },
    
    #[error("Coordination failure: {coordination_id} - {error}")]
    CoordinationFailure {
        coordination_id: CoordinationId,
        error: String,
        successful_engines: Vec<ScriptEngine>,
        failed_engines: Vec<ScriptEngine>,
    },
    
    #[error("Resource allocation failed: {resource_type} - {reason}")]
    ResourceAllocationFailed {
        resource_type: String,
        reason: String,
        requested_amount: u64,
        available_amount: u64,
    },
    
    #[error("Async context corruption: {context_id} - {description}")]
    ContextCorruption {
        context_id: String,
        description: String,
        recovery_attempts: u32,
    },
}
```

### Error Context and Metadata

```rust
// Enhanced error context for debugging and recovery
#[derive(Debug, Clone)]
pub struct ErrorContext {
    pub error_id: ErrorId,
    pub timestamp: DateTime<Utc>,
    pub component: ComponentIdentifier,
    pub operation: String,
    pub user_context: Option<UserContext>,
    pub execution_trace: ExecutionTrace,
    pub environment: EnvironmentInfo,
    pub correlation_id: Option<String>,
    pub metadata: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone)]
pub struct ExecutionTrace {
    pub call_stack: Vec<CallFrame>,
    pub async_context: Option<AsyncContext>,
    pub hook_execution_path: Vec<HookExecution>,
    pub event_emission_chain: Vec<EventEmission>,
}

#[derive(Debug, Clone)]
pub struct CallFrame {
    pub function_name: String,
    pub file_location: Option<FileLocation>,
    pub parameters: Option<serde_json::Value>,
    pub local_variables: Option<serde_json::Value>,
    pub execution_time: Duration,
}

#[derive(Debug, Clone)]
pub struct AsyncContext {
    pub task_id: TaskId,
    pub coordination_id: Option<CoordinationId>,
    pub engine: ScriptEngine,
    pub async_operation_type: AsyncOperationType,
    pub resource_allocations: Vec<ResourceAllocation>,
}

// Error severity classification
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum ErrorSeverity {
    Trace,      // Debug information, not an actual error
    Info,       // Informational, expected behavior
    Warning,    // Potential issue, but operation can continue
    Error,      // Operation failed, but system can recover
    Critical,   // System component failed, requires attention
    Fatal,      // System failure, requires restart or intervention
}

// Component identification for error tracking
#[derive(Debug, Clone)]
pub enum ComponentIdentifier {
    Agent { agent_id: String, agent_type: String },
    Tool { tool_name: String, category: String },
    Workflow { workflow_id: String, workflow_type: String },
    Hook { hook_name: String, hook_point: HookPoint },
    Event { event_type: String, emission_id: String },
    Script { language: ScriptLanguage, script_id: String },
    Bridge { bridge_type: String },
    System { subsystem: String },
}
```

## Error Recovery Strategies

### Comprehensive Recovery Framework

```rust
// Error recovery strategy framework
#[derive(Debug, Clone)]
pub enum ErrorRecoveryStrategy {
    // Immediate failure without recovery
    FailFast {
        cleanup_required: bool,
    },
    
    // Retry with various backoff strategies
    RetryWithBackoff {
        max_retries: u32,
        initial_delay: Duration,
        max_delay: Duration,
        backoff_strategy: BackoffStrategy,
        retry_condition: RetryCondition,
    },
    
    // Fallback to alternative implementations
    Fallback {
        fallback_options: Vec<FallbackOption>,
        fallback_selection: FallbackSelectionStrategy,
    },
    
    // Continue with degraded functionality
    ContinueDegraded {
        degradation_mode: DegradationMode,
        recovery_monitor: bool,
    },
    
    // Circuit breaker pattern for failing services
    CircuitBreaker {
        failure_threshold: u32,
        timeout: Duration,
        recovery_timeout: Duration,
        health_check: HealthCheckStrategy,
    },
    
    // Graceful degradation with user notification
    GracefulDegradation {
        degradation_steps: Vec<DegradationStep>,
        user_notification: bool,
        automatic_recovery: bool,
    },
    
    // Complete restart of component or system
    RestartComponent {
        restart_scope: RestartScope,
        state_preservation: StatePreservationStrategy,
        max_restart_attempts: u32,
    },
}

#[derive(Debug, Clone)]
pub enum BackoffStrategy {
    Linear { increment: Duration },
    Exponential { multiplier: f64 },
    Fibonacci,
    Custom { function: String }, // Named function for custom backoff
}

#[derive(Debug, Clone)]
pub enum RetryCondition {
    Always,
    OnSpecificErrors(Vec<String>),
    OnTransientErrors,
    Custom { condition: String }, // Named condition function
}

#[derive(Debug, Clone)]
pub struct FallbackOption {
    pub option_type: FallbackType,
    pub priority: i32,
    pub availability_check: Option<String>,
    pub configuration: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone)]
pub enum FallbackType {
    AlternativeAgent { agent_id: String },
    AlternativeTool { tool_name: String },
    CachedResult { cache_key: String },
    DefaultResponse { response: serde_json::Value },
    UserPrompt { prompt_template: String },
    ExternalService { service_endpoint: String },
}

// Error recovery coordinator
pub struct ErrorRecoveryCoordinator {
    recovery_strategies: HashMap<String, ErrorRecoveryStrategy>,
    recovery_history: Arc<Mutex<RecoveryHistory>>,
    circuit_breakers: HashMap<String, CircuitBreaker>,
    fallback_registry: FallbackRegistry,
    health_monitor: HealthMonitor,
}

impl ErrorRecoveryCoordinator {
    pub async fn handle_error<T, F, Fut>(
        &self,
        error: LLMSpellError,
        context: ErrorContext,
        operation: F,
    ) -> Result<T, LLMSpellError>
    where
        F: Fn() -> Fut + Send + Clone + 'static,
        Fut: Future<Output = Result<T, LLMSpellError>> + Send + 'static,
        T: Send + 'static,
    {
        let error_signature = self.classify_error(&error, &context);
        let strategy = self.get_recovery_strategy(&error_signature, &context)?;
        
        // Record error occurrence
        self.record_error_occurrence(&error, &context).await;
        
        match strategy {
            ErrorRecoveryStrategy::FailFast { cleanup_required } => {
                if cleanup_required {
                    self.perform_cleanup(&context).await?;
                }
                Err(error)
            },
            
            ErrorRecoveryStrategy::RetryWithBackoff { 
                max_retries, 
                initial_delay, 
                max_delay, 
                backoff_strategy,
                retry_condition,
            } => {
                self.execute_retry_strategy(
                    operation,
                    max_retries,
                    initial_delay,
                    max_delay,
                    backoff_strategy,
                    retry_condition,
                    &context
                ).await
            },
            
            ErrorRecoveryStrategy::Fallback { 
                fallback_options, 
                fallback_selection 
            } => {
                self.execute_fallback_strategy(
                    fallback_options,
                    fallback_selection,
                    &context
                ).await
            },
            
            ErrorRecoveryStrategy::CircuitBreaker { 
                failure_threshold, 
                timeout, 
                recovery_timeout,
                health_check,
            } => {
                self.execute_circuit_breaker_strategy(
                    operation,
                    failure_threshold,
                    timeout,
                    recovery_timeout,
                    health_check,
                    &context
                ).await
            },
            
            ErrorRecoveryStrategy::ContinueDegraded { 
                degradation_mode, 
                recovery_monitor 
            } => {
                self.execute_degraded_operation(
                    operation,
                    degradation_mode,
                    recovery_monitor,
                    &context
                ).await
            },
            
            ErrorRecoveryStrategy::GracefulDegradation { 
                degradation_steps, 
                user_notification, 
                automatic_recovery 
            } => {
                self.execute_graceful_degradation(
                    operation,
                    degradation_steps,
                    user_notification,
                    automatic_recovery,
                    &context
                ).await
            },
            
            ErrorRecoveryStrategy::RestartComponent { 
                restart_scope, 
                state_preservation, 
                max_restart_attempts 
            } => {
                self.execute_component_restart(
                    operation,
                    restart_scope,
                    state_preservation,
                    max_restart_attempts,
                    &context
                ).await
            },
        }
    }
    
    async fn execute_retry_strategy<T, F, Fut>(
        &self,
        operation: F,
        max_retries: u32,
        initial_delay: Duration,
        max_delay: Duration,
        backoff_strategy: BackoffStrategy,
        retry_condition: RetryCondition,
        context: &ErrorContext,
    ) -> Result<T, LLMSpellError>
    where
        F: Fn() -> Fut + Send + Clone + 'static,
        Fut: Future<Output = Result<T, LLMSpellError>> + Send + 'static,
        T: Send + 'static,
    {
        let mut delay = initial_delay;
        let mut last_error = None;
        
        for attempt in 0..=max_retries {
            match operation().await {
                Ok(result) => {
                    if attempt > 0 {
                        self.record_successful_retry(attempt, context).await;
                    }
                    return Ok(result);
                },
                Err(error) => {
                    // Check if we should retry this error
                    if !self.should_retry(&error, &retry_condition) {
                        return Err(error);
                    }
                    
                    last_error = Some(error);
                    
                    if attempt < max_retries {
                        self.record_retry_attempt(attempt + 1, &delay, context).await;
                        
                        // Apply jitter to prevent thundering herd
                        let jitter = Duration::from_millis(
                            fastrand::u64(0..=(delay.as_millis() as u64 / 10))
                        );
                        tokio::time::sleep(delay + jitter).await;
                        
                        // Calculate next delay
                        delay = self.calculate_next_delay(delay, max_delay, &backoff_strategy);
                    }
                }
            }
        }
        
        // All retries exhausted
        self.record_retry_exhausted(max_retries, context).await;
        Err(last_error.unwrap())
    }
    
    async fn execute_fallback_strategy<T>(
        &self,
        fallback_options: Vec<FallbackOption>,
        selection_strategy: FallbackSelectionStrategy,
        context: &ErrorContext,
    ) -> Result<T, LLMSpellError>
    where
        T: Send + 'static,
    {
        let ordered_options = self.order_fallback_options(fallback_options, selection_strategy)?;
        
        for (index, option) in ordered_options.iter().enumerate() {
            // Check availability if specified
            if let Some(ref availability_check) = option.availability_check {
                if !self.check_availability(availability_check).await? {
                    continue;
                }
            }
            
            match self.execute_fallback_option(option, context).await {
                Ok(result) => {
                    self.record_successful_fallback(index, option, context).await;
                    return Ok(result);
                },
                Err(fallback_error) => {
                    self.record_failed_fallback(index, option, &fallback_error, context).await;
                    continue;
                }
            }
        }
        
        Err(LLMSpellError::Runtime(RuntimeError::AllFallbacksFailed {
            attempted_options: ordered_options.len(),
            context: context.clone(),
        }))
    }
}

// Circuit breaker implementation
pub struct CircuitBreaker {
    state: Arc<Mutex<CircuitBreakerState>>,
    config: CircuitBreakerConfig,
    metrics: CircuitBreakerMetrics,
}

#[derive(Debug, Clone)]
enum CircuitBreakerState {
    Closed { failure_count: u32 },
    Open { opened_at: Instant },
    HalfOpen { test_request_sent: bool },
}

#[derive(Debug, Clone)]
pub struct CircuitBreakerConfig {
    pub failure_threshold: u32,
    pub timeout: Duration,
    pub recovery_timeout: Duration,
    pub health_check_interval: Duration,
}

impl CircuitBreaker {
    pub async fn call<T, F, Fut>(&self, operation: F) -> Result<T, LLMSpellError>
    where
        F: FnOnce() -> Fut,
        Fut: Future<Output = Result<T, LLMSpellError>>,
    {
        // Check circuit breaker state
        let should_allow = {
            let mut state = self.state.lock().await;
            match *state {
                CircuitBreakerState::Closed { .. } => true,
                CircuitBreakerState::Open { opened_at } => {
                    if opened_at.elapsed() > self.config.recovery_timeout {
                        *state = CircuitBreakerState::HalfOpen { test_request_sent: false };
                        true
                    } else {
                        false
                    }
                },
                CircuitBreakerState::HalfOpen { test_request_sent } => {
                    if !test_request_sent {
                        // Allow one test request
                        if let CircuitBreakerState::HalfOpen { ref mut test_request_sent } = *state {
                            *test_request_sent = true;
                        }
                        true
                    } else {
                        false
                    }
                }
            }
        };
        
        if !should_allow {
            self.metrics.record_rejected_call().await;
            return Err(LLMSpellError::Runtime(RuntimeError::CircuitBreakerOpen {
                service: "unknown".to_string(), // Could be passed in
                opened_duration: self.get_opened_duration().await,
            }));
        }
        
        // Execute operation
        let start_time = Instant::now();
        let result = operation().await;
        let execution_time = start_time.elapsed();
        
        // Update circuit breaker state based on result
        {
            let mut state = self.state.lock().await;
            match result {
                Ok(_) => {
                    *state = CircuitBreakerState::Closed { failure_count: 0 };
                    self.metrics.record_successful_call(execution_time).await;
                },
                Err(_) => {
                    match *state {
                        CircuitBreakerState::Closed { failure_count } => {
                            let new_failure_count = failure_count + 1;
                            if new_failure_count >= self.config.failure_threshold {
                                *state = CircuitBreakerState::Open { 
                                    opened_at: Instant::now() 
                                };
                                self.metrics.record_circuit_opened().await;
                            } else {
                                *state = CircuitBreakerState::Closed { 
                                    failure_count: new_failure_count 
                                };
                            }
                        },
                        CircuitBreakerState::HalfOpen { .. } => {
                            *state = CircuitBreakerState::Open { 
                                opened_at: Instant::now() 
                            };
                            self.metrics.record_circuit_opened().await;
                        },
                        CircuitBreakerState::Open { .. } => {
                            // Already open, no state change needed
                        }
                    }
                    self.metrics.record_failed_call(execution_time).await;
                }
            }
        }
        
        result
    }
}
```

## Cross-Engine Error Translation

### Unified Error Translation Framework

```rust
// Cross-engine error translation system
pub struct CrossEngineErrorTranslator {
    lua_translator: LuaErrorTranslator,
    js_translator: JavaScriptErrorTranslator,
    error_mappings: HashMap<String, ErrorMapping>,
    context_preservers: Vec<Box<dyn ContextPreserver>>,
}

impl CrossEngineErrorTranslator {
    pub fn translate_error(
        &self,
        error: LLMSpellError,
        target_engine: ScriptEngine,
        context: &ExecutionContext,
    ) -> Result<EngineSpecificError, TranslationError> {
        match target_engine {
            ScriptEngine::Lua => {
                self.lua_translator.translate_from_rust(error, context)
            },
            ScriptEngine::JavaScript => {
                self.js_translator.translate_from_rust(error, context)
            },
            ScriptEngine::Rust => {
                // No translation needed
                Ok(EngineSpecificError::Rust(error))
            }
        }
    }
    
    pub fn translate_from_engine(
        &self,
        engine_error: EngineSpecificError,
        source_engine: ScriptEngine,
        context: &ExecutionContext,
    ) -> Result<LLMSpellError, TranslationError> {
        match (engine_error, source_engine) {
            (EngineSpecificError::Lua(lua_error), ScriptEngine::Lua) => {
                self.lua_translator.translate_to_rust(lua_error, context)
            },
            (EngineSpecificError::JavaScript(js_error), ScriptEngine::JavaScript) => {
                self.js_translator.translate_to_rust(js_error, context)
            },
            (EngineSpecificError::Rust(rust_error), ScriptEngine::Rust) => {
                Ok(rust_error)
            },
            _ => Err(TranslationError::InvalidEngineErrorCombination)
        }
    }
}

// Lua error translation
pub struct LuaErrorTranslator {
    error_pattern_matchers: Vec<LuaErrorPattern>,
    stack_trace_parser: LuaStackTraceParser,
}

impl LuaErrorTranslator {
    pub fn translate_from_rust(
        &self,
        rust_error: LLMSpellError,
        context: &ExecutionContext,
    ) -> Result<EngineSpecificError, TranslationError> {
        let lua_error_info = LuaErrorInfo {
            error_type: self.map_rust_error_to_lua_type(&rust_error),
            message: self.format_error_message_for_lua(&rust_error, context),
            stack_trace: self.generate_lua_stack_trace(&rust_error, context),
            metadata: self.extract_lua_metadata(&rust_error, context),
        };
        
        Ok(EngineSpecificError::Lua(lua_error_info))
    }
    
    pub fn translate_to_rust(
        &self,
        lua_error: mlua::Error,
        context: &ExecutionContext,
    ) -> Result<LLMSpellError, TranslationError> {
        match lua_error {
            mlua::Error::RuntimeError(msg) => {
                // Parse runtime error message for specific error types
                if let Some(agent_error) = self.parse_agent_error(&msg) {
                    Ok(LLMSpellError::Agent(agent_error))
                } else if let Some(tool_error) = self.parse_tool_error(&msg) {
                    Ok(LLMSpellError::Tool(tool_error))
                } else {
                    Ok(LLMSpellError::Script(ScriptError::Lua {
                        error: lua_error,
                        context: context.operation.clone(),
                        script_line: self.extract_line_number(&msg),
                        stack_trace: self.extract_lua_stack_trace(&msg),
                    }))
                }
            },
            mlua::Error::SyntaxError { message, incomplete_input } => {
                Ok(LLMSpellError::Script(ScriptError::CompilationFailed {
                    language: ScriptLanguage::Lua,
                    error: message,
                    source_code: context.get_script_source(),
                }))
            },
            mlua::Error::MemoryError(msg) => {
                Ok(LLMSpellError::Resource(ResourceError::MemoryExhaustion {
                    context: "lua_runtime".to_string(),
                    requested: 0, // Not available from mlua
                    available: 0, // Not available from mlua
                    description: msg,
                }))
            },
            mlua::Error::SafetyError(msg) => {
                Ok(LLMSpellError::Security(SecurityError::ScriptSafetyViolation {
                    engine: ScriptLanguage::Lua,
                    violation: msg,
                    severity: SecuritySeverity::High,
                }))
            },
            _ => {
                Ok(LLMSpellError::Script(ScriptError::RuntimeError {
                    language: ScriptLanguage::Lua,
                    error: lua_error.to_string(),
                    execution_context: Some(context.operation.clone()),
                }))
            }
        }
    }
    
    fn parse_agent_error(&self, message: &str) -> Option<AgentError> {
        // Parse agent-specific error patterns
        for pattern in &self.error_pattern_matchers {
            if let Some(captures) = pattern.regex.captures(message) {
                return pattern.create_agent_error(&captures);
            }
        }
        None
    }
}

// JavaScript error translation
pub struct JavaScriptErrorTranslator {
    error_type_mappings: HashMap<String, JSErrorType>,
    stack_trace_parser: JSStackTraceParser,
}

impl JavaScriptErrorTranslator {
    pub fn translate_from_rust(
        &self,
        rust_error: LLMSpellError,
        context: &ExecutionContext,
    ) -> Result<EngineSpecificError, TranslationError> {
        let js_error_info = JavaScriptErrorInfo {
            name: self.map_rust_error_to_js_name(&rust_error),
            message: self.format_error_message_for_js(&rust_error, context),
            stack: self.generate_js_stack_trace(&rust_error, context),
            cause: self.extract_error_cause(&rust_error),
            custom_properties: self.extract_js_metadata(&rust_error, context),
        };
        
        Ok(EngineSpecificError::JavaScript(js_error_info))
    }
    
    pub fn translate_to_rust(
        &self,
        js_error: JavaScriptErrorInfo,
        context: &ExecutionContext,
    ) -> Result<LLMSpellError, TranslationError> {
        match js_error.name.as_str() {
            "AgentError" => {
                self.parse_js_agent_error(&js_error, context)
            },
            "ToolError" => {
                self.parse_js_tool_error(&js_error, context)
            },
            "TypeError" | "ReferenceError" | "SyntaxError" => {
                Ok(LLMSpellError::Script(ScriptError::JavaScript {
                    error: format!("{}: {}", js_error.name, js_error.message),
                    context: context.operation.clone(),
                    script_line: self.extract_line_from_stack(&js_error.stack),
                    stack_trace: Some(js_error.stack),
                }))
            },
            "SecurityError" => {
                Ok(LLMSpellError::Security(SecurityError::ScriptSafetyViolation {
                    engine: ScriptLanguage::JavaScript,
                    violation: js_error.message,
                    severity: self.determine_security_severity(&js_error),
                }))
            },
            _ => {
                Ok(LLMSpellError::Script(ScriptError::RuntimeError {
                    language: ScriptLanguage::JavaScript,
                    error: format!("{}: {}", js_error.name, js_error.message),
                    execution_context: Some(context.operation.clone()),
                }))
            }
        }
    }
}

// Error context preservation across engine boundaries
pub trait ContextPreserver: Send + Sync {
    fn preserve_context(
        &self,
        error: &LLMSpellError,
        source_context: &ExecutionContext,
    ) -> Result<PreservedContext, ContextPreservationError>;
    
    fn restore_context(
        &self,
        preserved: &PreservedContext,
        target_context: &mut ExecutionContext,
    ) -> Result<(), ContextPreservationError>;
}

pub struct StackTracePreserver;

impl ContextPreserver for StackTracePreserver {
    fn preserve_context(
        &self,
        error: &LLMSpellError,
        source_context: &ExecutionContext,
    ) -> Result<PreservedContext, ContextPreservationError> {
        let stack_trace = match error {
            LLMSpellError::Script(script_error) => {
                match script_error {
                    ScriptError::Lua { stack_trace, .. } => stack_trace.clone(),
                    ScriptError::JavaScript { stack_trace, .. } => stack_trace.clone(),
                    _ => None
                }
            },
            _ => None
        };
        
        Ok(PreservedContext {
            context_type: "stack_trace".to_string(),
            data: json!({
                "original_stack_trace": stack_trace,
                "execution_path": source_context.execution_trace.call_stack,
                "source_engine": source_context.component,
            }),
        })
    }
    
    fn restore_context(
        &self,
        preserved: &PreservedContext,
        target_context: &mut ExecutionContext,
    ) -> Result<(), ContextPreservationError> {
        if let Some(original_stack) = preserved.data.get("original_stack_trace") {
            target_context.metadata.insert(
                "preserved_stack_trace".to_string(),
                original_stack.clone()
            );
        }
        
        if let Some(execution_path) = preserved.data.get("execution_path") {
            target_context.metadata.insert(
                "original_execution_path".to_string(),
                execution_path.clone()
            );
        }
        
        Ok(())
    }
}
```

## Async Error Handling

### Async Error Propagation

```rust
// Async error handling with proper propagation
pub struct AsyncErrorHandler {
    error_channels: HashMap<TaskId, ErrorChannel>,
    error_aggregator: ErrorAggregator,
    cancellation_manager: CancellationManager,
    timeout_manager: TimeoutManager,
}

impl AsyncErrorHandler {
    pub async fn handle_async_error(
        &self,
        task_id: TaskId,
        error: LLMSpellError,
        async_context: AsyncContext,
    ) -> Result<AsyncErrorResolution, AsyncErrorHandlingError> {
        // Determine error handling strategy based on async context
        let strategy = self.determine_async_strategy(&error, &async_context)?;
        
        match strategy {
            AsyncErrorStrategy::PropagateImmediately => {
                self.propagate_error_immediately(task_id, error, async_context).await
            },
            AsyncErrorStrategy::CollectAndBatch => {
                self.collect_error_for_batching(task_id, error, async_context).await
            },
            AsyncErrorStrategy::CancelDependentTasks => {
                self.cancel_dependent_tasks(task_id, error, async_context).await
            },
            AsyncErrorStrategy::IsolateAndContinue => {
                self.isolate_failed_task(task_id, error, async_context).await
            },
            AsyncErrorStrategy::RetryWithBackoff => {
                self.schedule_async_retry(task_id, error, async_context).await
            },
        }
    }
    
    async fn cancel_dependent_tasks(
        &self,
        failed_task_id: TaskId,
        error: LLMSpellError,
        async_context: AsyncContext,
    ) -> Result<AsyncErrorResolution, AsyncErrorHandlingError> {
        // Find all tasks that depend on the failed task
        let dependent_tasks = self.find_dependent_tasks(&failed_task_id).await?;
        
        let mut cancellation_results = Vec::new();
        
        for dependent_task in dependent_tasks {
            let cancellation_result = self.cancellation_manager.cancel_task(
                dependent_task.task_id,
                CancellationReason::DependencyFailed {
                    failed_dependency: failed_task_id,
                    original_error: error.clone(),
                }
            ).await;
            
            cancellation_results.push(TaskCancellationResult {
                task_id: dependent_task.task_id,
                result: cancellation_result,
                cancellation_time: Instant::now(),
            });
        }
        
        Ok(AsyncErrorResolution::DependentTasksCancelled {
            original_error: error,
            cancelled_tasks: cancellation_results,
            propagation_chain: self.build_propagation_chain(&failed_task_id).await?,
        })
    }
    
    async fn handle_cross_engine_async_error(
        &self,
        coordination_id: CoordinationId,
        engine_errors: Vec<(ScriptEngine, LLMSpellError)>,
        coordination_context: CoordinationContext,
    ) -> Result<CrossEngineErrorResolution, AsyncErrorHandlingError> {
        let error_summary = self.analyze_cross_engine_errors(&engine_errors).await?;
        
        match error_summary.error_pattern {
            CrossEngineErrorPattern::AllEnginesFailed => {
                // All engines failed - propagate the most severe error
                let most_severe = error_summary.most_severe_error;
                Ok(CrossEngineErrorResolution::AllFailed {
                    representative_error: most_severe,
                    engine_errors,
                    coordination_id,
                })
            },
            CrossEngineErrorPattern::PartialFailure => {
                // Some engines succeeded - decide whether to continue or fail
                let continuation_decision = self.evaluate_partial_failure_continuation(
                    &engine_errors,
                    &coordination_context
                ).await?;
                
                match continuation_decision {
                    ContinuationDecision::ContinueWithSuccessful => {
                        Ok(CrossEngineErrorResolution::PartialSuccess {
                            successful_engines: error_summary.successful_engines,
                            failed_engines: engine_errors,
                            coordination_id,
                        })
                    },
                    ContinuationDecision::FailEntireOperation => {
                        Ok(CrossEngineErrorResolution::OperationFailed {
                            reason: "Partial failure exceeded tolerance threshold".to_string(),
                            engine_errors,
                            coordination_id,
                        })
                    }
                }
            },
            CrossEngineErrorPattern::CascadingFailure => {
                // One failure caused others - identify root cause
                let root_cause = self.identify_cascade_root_cause(&engine_errors).await?;
                Ok(CrossEngineErrorResolution::CascadingFailure {
                    root_cause,
                    cascade_chain: engine_errors,
                    coordination_id,
                })
            },
        }
    }
}

// Async task cancellation with proper cleanup
pub struct CancellationManager {
    active_cancellations: Arc<Mutex<HashMap<TaskId, CancellationHandle>>>,
    cleanup_registry: CleanupRegistry,
    cancellation_listeners: Vec<Box<dyn CancellationListener>>,
}

impl CancellationManager {
    pub async fn cancel_task(
        &self,
        task_id: TaskId,
        reason: CancellationReason,
    ) -> Result<CancellationResult, CancellationError> {
        let cancellation_handle = CancellationHandle::new(task_id, reason.clone());
        
        // Register cancellation
        {
            let mut active = self.active_cancellations.lock().await;
            active.insert(task_id, cancellation_handle.clone());
        }
        
        // Notify listeners
        for listener in &self.cancellation_listeners {
            listener.on_cancellation_started(&cancellation_handle).await;
        }
        
        // Perform actual cancellation
        let cancellation_result = match self.find_task_executor(&task_id).await? {
            TaskExecutor::Rust(executor) => {
                executor.cancel_task(task_id).await
            },
            TaskExecutor::Lua(executor) => {
                executor.cancel_coroutine(task_id).await
            },
            TaskExecutor::JavaScript(executor) => {
                executor.cancel_promise(task_id).await
            },
        };
        
        // Perform cleanup
        let cleanup_result = self.cleanup_registry.cleanup_task_resources(
            &task_id,
            &reason
        ).await;
        
        // Remove from active cancellations
        {
            let mut active = self.active_cancellations.lock().await;
            active.remove(&task_id);
        }
        
        // Notify listeners of completion
        for listener in &self.cancellation_listeners {
            listener.on_cancellation_completed(&cancellation_handle, &cancellation_result).await;
        }
        
        Ok(CancellationResult {
            task_id,
            reason,
            cancellation_result,
            cleanup_result,
            cancellation_time: Instant::now(),
        })
    }
}

// Timeout management for async operations
pub struct TimeoutManager {
    active_timeouts: Arc<Mutex<HashMap<TaskId, TimeoutHandle>>>,
    default_timeouts: HashMap<TaskType, Duration>,
    timeout_strategies: HashMap<String, TimeoutStrategy>,
}

impl TimeoutManager {
    pub async fn start_timeout(
        &self,
        task_id: TaskId,
        timeout_duration: Duration,
        timeout_strategy: TimeoutStrategy,
    ) -> Result<TimeoutHandle, TimeoutError> {
        let timeout_handle = TimeoutHandle::new(task_id, timeout_duration);
        
        // Start timeout timer
        let timeout_task = {
            let task_id = task_id;
            let timeout_duration = timeout_duration;
            let timeout_strategy = timeout_strategy.clone();
            let handle = timeout_handle.clone();
            
            tokio::spawn(async move {
                tokio::time::sleep(timeout_duration).await;
                
                // Check if timeout is still active
                if handle.is_active().await {
                    handle.trigger_timeout(timeout_strategy).await
                }
            })
        };
        
        timeout_handle.set_timeout_task(timeout_task).await;
        
        // Register timeout
        {
            let mut active = self.active_timeouts.lock().await;
            active.insert(task_id, timeout_handle.clone());
        }
        
        Ok(timeout_handle)
    }
    
    pub async fn clear_timeout(&self, task_id: &TaskId) -> Result<(), TimeoutError> {
        let timeout_handle = {
            let mut active = self.active_timeouts.lock().await;
            active.remove(task_id)
        };
        
        if let Some(handle) = timeout_handle {
            handle.cancel().await?;
        }
        
        Ok(())
    }
}
```

## Production Error Monitoring

### Comprehensive Error Tracking

```rust
// Production error monitoring and tracking
pub struct ErrorMonitoringSystem {
    error_collector: ErrorCollector,
    metric_reporter: MetricReporter,
    alert_manager: AlertManager,
    error_analyzer: ErrorAnalyzer,
    dashboard_updater: DashboardUpdater,
}

impl ErrorMonitoringSystem {
    pub async fn record_error(
        &self,
        error: &LLMSpellError,
        context: &ErrorContext,
        recovery_result: Option<&RecoveryResult>,
    ) -> Result<(), MonitoringError> {
        // Collect error data
        let error_record = ErrorRecord {
            error_id: context.error_id,
            timestamp: context.timestamp,
            error_type: self.classify_error_type(error),
            severity: self.determine_error_severity(error, context),
            component: context.component.clone(),
            user_impact: self.assess_user_impact(error, context),
            recovery_applied: recovery_result.is_some(),
            error_details: self.extract_error_details(error),
            context_snapshot: self.create_context_snapshot(context),
        };
        
        // Store error record
        self.error_collector.collect_error(error_record.clone()).await?;
        
        // Update metrics
        self.metric_reporter.update_error_metrics(&error_record).await?;
        
        // Check alert conditions
        if self.should_trigger_alert(&error_record).await? {
            self.alert_manager.send_alert(
                self.create_alert_from_error(&error_record).await?
            ).await?;
        }
        
        // Update dashboard
        self.dashboard_updater.update_error_dashboard(&error_record).await?;
        
        Ok(())
    }
    
    pub async fn analyze_error_trends(
        &self,
        time_window: Duration,
        analysis_config: AnalysisConfig,
    ) -> Result<ErrorTrendAnalysis, MonitoringError> {
        let error_records = self.error_collector.get_errors_in_window(time_window).await?;
        
        let analysis = ErrorTrendAnalysis {
            time_window,
            total_errors: error_records.len(),
            error_rate: self.calculate_error_rate(&error_records, time_window),
            severity_distribution: self.analyze_severity_distribution(&error_records),
            component_breakdown: self.analyze_component_breakdown(&error_records),
            error_type_trends: self.analyze_error_type_trends(&error_records),
            recovery_success_rate: self.calculate_recovery_success_rate(&error_records),
            user_impact_assessment: self.assess_aggregate_user_impact(&error_records),
            recommendations: self.generate_recommendations(&error_records, &analysis_config),
        };
        
        Ok(analysis)
    }
    
    async fn should_trigger_alert(&self, error_record: &ErrorRecord) -> Result<bool, MonitoringError> {
        // Check immediate alert conditions
        if error_record.severity >= ErrorSeverity::Critical {
            return Ok(true);
        }
        
        // Check rate-based alerts
        let recent_errors = self.error_collector.get_recent_errors_for_component(
            &error_record.component,
            Duration::from_minutes(5)
        ).await?;
        
        if recent_errors.len() > 10 {
            return Ok(true);
        }
        
        // Check pattern-based alerts
        let error_pattern = self.error_analyzer.detect_error_pattern(
            &error_record.error_type,
            Duration::from_minutes(15)
        ).await?;
        
        if let Some(pattern) = error_pattern {
            if pattern.is_concerning() {
                return Ok(true);
            }
        }
        
        Ok(false)
    }
}

// Error metrics and reporting
pub struct MetricReporter {
    metrics_client: Box<dyn MetricsClient>,
    gauge_cache: HashMap<String, f64>,
    counter_cache: HashMap<String, u64>,
}

impl MetricReporter {
    pub async fn update_error_metrics(&self, error_record: &ErrorRecord) -> Result<(), MetricsError> {
        // Update error counters
        self.metrics_client.increment_counter(
            "llmspell_errors_total",
            &[
                ("component", &error_record.component.to_string()),
                ("error_type", &error_record.error_type),
                ("severity", &error_record.severity.to_string()),
            ]
        ).await?;
        
        // Update error rate gauge
        let current_rate = self.calculate_current_error_rate().await?;
        self.metrics_client.set_gauge(
            "llmspell_error_rate",
            current_rate,
            &[]
        ).await?;
        
        // Update recovery metrics
        if error_record.recovery_applied {
            self.metrics_client.increment_counter(
                "llmspell_error_recoveries_total",
                &[("component", &error_record.component.to_string())]
            ).await?;
        }
        
        // Update user impact metrics
        self.metrics_client.set_gauge(
            "llmspell_user_impact_score",
            error_record.user_impact.impact_score,
            &[("severity", &error_record.severity.to_string())]
        ).await?;
        
        Ok(())
    }
}

// Alert management
pub struct AlertManager {
    alert_channels: Vec<Box<dyn AlertChannel>>,
    alert_rules: Vec<AlertRule>,
    rate_limiter: AlertRateLimiter,
}

impl AlertManager {
    pub async fn send_alert(&self, alert: Alert) -> Result<(), AlertError> {
        // Check rate limiting
        if !self.rate_limiter.should_send_alert(&alert).await? {
            return Ok(()); // Silently drop rate-limited alerts
        }
        
        // Send to all configured channels
        let mut send_results = Vec::new();
        
        for channel in &self.alert_channels {
            let result = channel.send_alert(&alert).await;
            send_results.push(result);
        }
        
        // Record alert metrics
        self.record_alert_metrics(&alert, &send_results).await?;
        
        // Check if any channel succeeded
        let any_success = send_results.iter().any(|r| r.is_ok());
        if !any_success {
            return Err(AlertError::AllChannelsFailed {
                alert_id: alert.id,
                channel_errors: send_results.into_iter()
                    .filter_map(|r| r.err())
                    .collect(),
            });
        }
        
        Ok(())
    }
}

// Error pattern detection
pub struct ErrorAnalyzer {
    pattern_detectors: Vec<Box<dyn ErrorPatternDetector>>,
    ml_model: Option<ErrorPredictionModel>,
}

impl ErrorAnalyzer {
    pub async fn detect_error_pattern(
        &self,
        error_type: &str,
        time_window: Duration,
    ) -> Result<Option<ErrorPattern>, AnalysisError> {
        for detector in &self.pattern_detectors {
            if let Some(pattern) = detector.detect_pattern(error_type, time_window).await? {
                return Ok(Some(pattern));
            }
        }
        
        Ok(None)
    }
    
    pub async fn predict_error_likelihood(
        &self,
        context: &PredictionContext,
    ) -> Result<ErrorPrediction, AnalysisError> {
        if let Some(ref model) = self.ml_model {
            model.predict_error_likelihood(context).await
        } else {
            // Fallback to rule-based prediction
            self.rule_based_error_prediction(context).await
        }
    }
}
```

This complete error handling strategy provides production-ready error management with comprehensive recovery, cross-engine translation, async error handling, and monitoring capabilities that ensure robust operation across all rs-llmspell components.