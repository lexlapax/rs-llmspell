//! # Kernel-Specific Hook Types
//!
//! Specialized hook implementations for kernel execution flow including
//! `PreExecute`, `PostExecute`, `PreDebug`, and `StateChange` hooks with rich context.

use anyhow::Result;
use async_trait::async_trait;
use llmspell_hooks::{Hook, HookContext, HookMetadata, HookPoint, HookResult, Language, Priority};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

/// Execution context for kernel hooks
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionContext {
    /// Script code being executed
    pub code: String,
    /// Execution session ID
    pub session_id: String,
    /// Message ID for correlation
    pub message_id: String,
    /// Whether execution is silent (no output)
    pub silent: bool,
    /// User expressions to evaluate
    pub user_expressions: HashMap<String, String>,
    /// Expected execution time estimate
    pub estimated_duration: Option<Duration>,
    /// Execution priority
    pub priority: ExecutionPriority,
    /// Additional execution parameters
    pub parameters: Value,
}

/// Debug context for debug-related hooks
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DebugContext {
    /// Debug session ID
    pub session_id: String,
    /// Script being debugged
    pub script_path: Option<String>,
    /// Current breakpoint information
    pub breakpoint: Option<BreakpointInfo>,
    /// Current execution line
    pub current_line: Option<u32>,
    /// Local variables
    pub variables: HashMap<String, Value>,
    /// Call stack depth
    pub stack_depth: u32,
    /// Debug command being executed
    pub command: DebugCommand,
    /// Debug session state
    pub debug_state: DebugSessionState,
}

/// State change context for state management hooks
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateContext {
    /// Type of state being changed
    pub state_type: StateType,
    /// Previous state value
    pub previous_state: Option<Value>,
    /// New state value
    pub new_state: Value,
    /// Reason for state change
    pub change_reason: String,
    /// Session ID associated with change
    pub session_id: Option<String>,
    /// Whether change should be persisted
    pub persist: bool,
    /// State change metadata
    pub metadata: HashMap<String, String>,
}

/// Execution priority levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExecutionPriority {
    /// Low priority execution
    Low,
    /// Normal priority execution
    Normal,
    /// High priority execution
    High,
    /// Critical priority execution
    Critical,
}

/// Breakpoint information for debug context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BreakpointInfo {
    /// Unique breakpoint identifier
    pub id: u32,
    /// Line number where breakpoint is set
    pub line: u32,
    /// Optional condition for conditional breakpoints
    pub condition: Option<String>,
    /// Number of times this breakpoint has been hit
    pub hit_count: u32,
    /// Whether the breakpoint is currently enabled
    pub enabled: bool,
}

/// Debug commands
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DebugCommand {
    /// Continue execution
    Continue,
    /// Step into function calls
    StepIn,
    /// Step out of current function
    StepOut,
    /// Step over current line
    StepOver,
    /// Pause execution
    Pause,
    /// Set a new breakpoint
    SetBreakpoint {
        /// Line number for the breakpoint
        line: u32,
        /// Optional condition for the breakpoint
        condition: Option<String>,
    },
    /// Remove an existing breakpoint
    RemoveBreakpoint {
        /// Breakpoint ID to remove
        id: u32,
    },
    /// Evaluate an expression in current context
    Evaluate {
        /// Expression to evaluate
        expression: String,
    },
    /// Get current variables in scope
    GetVariables,
    /// Get current call stack trace
    GetStackTrace,
}

/// Debug session states
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DebugSessionState {
    /// Debug session is starting
    Starting,
    /// Debug session is running
    Running,
    /// Debug session is paused
    Paused,
    /// Debug session is stepping through code
    Stepping,
    /// Debug session has terminated
    Terminated,
}

/// State types for state change hooks
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StateType {
    /// Execution state
    Execution,
    /// Debug state
    Debug,
    /// Session state
    Session,
    /// Kernel state
    Kernel,
    /// Memory state
    Memory,
    /// Storage state
    Storage,
}

/// Kernel-specific hook enum
#[derive(Debug)]
pub enum KernelHook {
    /// Pre-execution hook
    PreExecute(PreExecuteHook),
    /// Post-execution hook
    PostExecute(PostExecuteHook),
    /// Pre-debug hook
    PreDebug(PreDebugHook),
    /// State change hook
    StateChange(StateChangeHook),
}

impl KernelHook {
    /// Get the hook point for this kernel hook
    pub fn hook_point(&self) -> HookPoint {
        match self {
            KernelHook::PreExecute(_) => HookPoint::BeforeAgentExecution,
            KernelHook::PostExecute(_) => HookPoint::AfterAgentExecution,
            KernelHook::PreDebug(_) => HookPoint::BeforeToolExecution,
            KernelHook::StateChange(_) => HookPoint::SystemStartup,
        }
    }

    /// Get the hook name
    pub fn name(&self) -> &str {
        match self {
            KernelHook::PreExecute(hook) => &hook.metadata.name,
            KernelHook::PostExecute(hook) => &hook.metadata.name,
            KernelHook::PreDebug(hook) => &hook.metadata.name,
            KernelHook::StateChange(hook) => &hook.metadata.name,
        }
    }
}

#[async_trait]
impl Hook for KernelHook {
    async fn execute(&self, context: &mut HookContext) -> Result<HookResult> {
        match self {
            KernelHook::PreExecute(hook) => hook.execute(context).await,
            KernelHook::PostExecute(hook) => hook.execute(context).await,
            KernelHook::PreDebug(hook) => hook.execute(context).await,
            KernelHook::StateChange(hook) => hook.execute(context).await,
        }
    }

    fn metadata(&self) -> HookMetadata {
        match self {
            KernelHook::PreExecute(hook) => hook.metadata(),
            KernelHook::PostExecute(hook) => hook.metadata(),
            KernelHook::PreDebug(hook) => hook.metadata(),
            KernelHook::StateChange(hook) => hook.metadata(),
        }
    }

    fn should_execute(&self, context: &HookContext) -> bool {
        match self {
            KernelHook::PreExecute(hook) => hook.should_execute(context),
            KernelHook::PostExecute(hook) => hook.should_execute(context),
            KernelHook::PreDebug(hook) => hook.should_execute(context),
            KernelHook::StateChange(hook) => hook.should_execute(context),
        }
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

/// Pre-execution hook for code execution
pub struct PreExecuteHook {
    handler: Arc<dyn PreExecuteHandler>,
    metadata: HookMetadata,
}

/// Post-execution hook for code execution
pub struct PostExecuteHook {
    handler: Arc<dyn PostExecuteHandler>,
    metadata: HookMetadata,
}

/// Pre-debug hook for debug operations
pub struct PreDebugHook {
    handler: Arc<dyn PreDebugHandler>,
    metadata: HookMetadata,
}

/// State change hook for state management
pub struct StateChangeHook {
    handler: Arc<dyn StateChangeHandler>,
    metadata: HookMetadata,
}

/// Handler trait for pre-execution hooks
#[async_trait]
pub trait PreExecuteHandler: Send + Sync {
    /// Handle pre-execution hook with execution context
    async fn handle_pre_execute(
        &self,
        execution_context: &ExecutionContext,
        hook_context: &mut HookContext,
    ) -> Result<HookResult>;
}

/// Handler trait for post-execution hooks
#[async_trait]
pub trait PostExecuteHandler: Send + Sync {
    /// Handle post-execution hook with execution context and results
    async fn handle_post_execute(
        &self,
        execution_context: &ExecutionContext,
        execution_result: &Value,
        duration: Duration,
        hook_context: &mut HookContext,
    ) -> Result<HookResult>;
}

/// Handler trait for pre-debug hooks
#[async_trait]
pub trait PreDebugHandler: Send + Sync {
    /// Handle pre-debug hook with debug context
    async fn handle_pre_debug(
        &self,
        debug_context: &DebugContext,
        hook_context: &mut HookContext,
    ) -> Result<HookResult>;
}

/// Handler trait for state change hooks
#[async_trait]
pub trait StateChangeHandler: Send + Sync {
    /// Handle state change hook with state context
    async fn handle_state_change(
        &self,
        state_context: &StateContext,
        hook_context: &mut HookContext,
    ) -> Result<HookResult>;
}

impl PreExecuteHook {
    /// Create a new pre-execute hook
    pub fn new(name: &str, handler: Arc<dyn PreExecuteHandler>) -> Self {
        Self {
            handler,
            metadata: HookMetadata {
                name: name.to_string(),
                description: Some("Pre-execution hook for code execution".to_string()),
                priority: Priority::HIGH,
                language: Language::Native,
                tags: vec![
                    "kernel".to_string(),
                    "execution".to_string(),
                    "pre".to_string(),
                ],
                version: "1.0.0".to_string(),
            },
        }
    }

    /// Create hook with custom metadata
    #[must_use]
    pub fn with_metadata(mut self, metadata: HookMetadata) -> Self {
        self.metadata = metadata;
        self
    }
}

#[async_trait]
impl Hook for PreExecuteHook {
    async fn execute(&self, context: &mut HookContext) -> Result<HookResult> {
        tracing::debug!("PreExecuteHook '{}': Executing", self.metadata.name);

        // Extract execution context from hook context
        let execution_context = if let Some(exec_data) = context.data.get("execution_context") {
            serde_json::from_value(exec_data.clone())?
        } else {
            // Create default execution context from available data
            ExecutionContext {
                code: context.get_metadata("code").unwrap_or("").to_string(),
                session_id: context.get_metadata("session_id").unwrap_or("").to_string(),
                message_id: context.get_metadata("message_id").unwrap_or("").to_string(),
                silent: context.get_metadata("silent").is_some_and(|s| s == "true"),
                user_expressions: HashMap::new(),
                estimated_duration: None,
                priority: ExecutionPriority::Normal,
                parameters: Value::Null,
            }
        };

        self.handler
            .handle_pre_execute(&execution_context, context)
            .await
    }

    fn metadata(&self) -> HookMetadata {
        self.metadata.clone()
    }

    fn should_execute(&self, _context: &HookContext) -> bool {
        true
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl PostExecuteHook {
    /// Create a new post-execute hook
    pub fn new(name: &str, handler: Arc<dyn PostExecuteHandler>) -> Self {
        Self {
            handler,
            metadata: HookMetadata {
                name: name.to_string(),
                description: Some("Post-execution hook for code execution".to_string()),
                priority: Priority::NORMAL,
                language: Language::Native,
                tags: vec![
                    "kernel".to_string(),
                    "execution".to_string(),
                    "post".to_string(),
                ],
                version: "1.0.0".to_string(),
            },
        }
    }

    /// Create hook with custom metadata
    #[must_use]
    pub fn with_metadata(mut self, metadata: HookMetadata) -> Self {
        self.metadata = metadata;
        self
    }
}

#[async_trait]
impl Hook for PostExecuteHook {
    async fn execute(&self, context: &mut HookContext) -> Result<HookResult> {
        tracing::debug!("PostExecuteHook '{}': Executing", self.metadata.name);

        // Extract execution context and result
        let execution_context = if let Some(exec_data) = context.data.get("execution_context") {
            serde_json::from_value(exec_data.clone())?
        } else {
            ExecutionContext {
                code: context.get_metadata("code").unwrap_or("").to_string(),
                session_id: context.get_metadata("session_id").unwrap_or("").to_string(),
                message_id: context.get_metadata("message_id").unwrap_or("").to_string(),
                silent: false,
                user_expressions: HashMap::new(),
                estimated_duration: None,
                priority: ExecutionPriority::Normal,
                parameters: Value::Null,
            }
        };

        let execution_result = context
            .data
            .get("execution_result")
            .cloned()
            .unwrap_or(Value::Null);

        let duration = context
            .get_metadata("execution_duration")
            .and_then(|d| d.parse::<u64>().ok())
            .map_or(Duration::ZERO, Duration::from_millis);

        self.handler
            .handle_post_execute(&execution_context, &execution_result, duration, context)
            .await
    }

    fn metadata(&self) -> HookMetadata {
        self.metadata.clone()
    }

    fn should_execute(&self, _context: &HookContext) -> bool {
        true
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl PreDebugHook {
    /// Create a new pre-debug hook
    pub fn new(name: &str, handler: Arc<dyn PreDebugHandler>) -> Self {
        Self {
            handler,
            metadata: HookMetadata {
                name: name.to_string(),
                description: Some("Pre-debug hook for debug operations".to_string()),
                priority: Priority::HIGH,
                language: Language::Native,
                tags: vec!["kernel".to_string(), "debug".to_string(), "pre".to_string()],
                version: "1.0.0".to_string(),
            },
        }
    }

    /// Create hook with custom metadata
    #[must_use]
    pub fn with_metadata(mut self, metadata: HookMetadata) -> Self {
        self.metadata = metadata;
        self
    }
}

#[async_trait]
impl Hook for PreDebugHook {
    async fn execute(&self, context: &mut HookContext) -> Result<HookResult> {
        tracing::debug!("PreDebugHook '{}': Executing", self.metadata.name);

        // Extract debug context
        let debug_context = if let Some(debug_data) = context.data.get("debug_context") {
            serde_json::from_value(debug_data.clone())?
        } else {
            DebugContext {
                session_id: context
                    .get_metadata("debug_session_id")
                    .unwrap_or("")
                    .to_string(),
                script_path: context.get_metadata("script_path").map(String::from),
                breakpoint: None,
                current_line: context
                    .get_metadata("current_line")
                    .and_then(|l| l.parse().ok()),
                variables: HashMap::new(),
                stack_depth: 0,
                command: DebugCommand::Continue,
                debug_state: DebugSessionState::Running,
            }
        };

        self.handler.handle_pre_debug(&debug_context, context).await
    }

    fn metadata(&self) -> HookMetadata {
        self.metadata.clone()
    }

    fn should_execute(&self, _context: &HookContext) -> bool {
        true
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl StateChangeHook {
    /// Create a new state change hook
    pub fn new(name: &str, handler: Arc<dyn StateChangeHandler>) -> Self {
        Self {
            handler,
            metadata: HookMetadata {
                name: name.to_string(),
                description: Some("State change hook for state management".to_string()),
                priority: Priority::NORMAL,
                language: Language::Native,
                tags: vec![
                    "kernel".to_string(),
                    "state".to_string(),
                    "change".to_string(),
                ],
                version: "1.0.0".to_string(),
            },
        }
    }

    /// Create hook with custom metadata
    #[must_use]
    pub fn with_metadata(mut self, metadata: HookMetadata) -> Self {
        self.metadata = metadata;
        self
    }
}

#[async_trait]
impl Hook for StateChangeHook {
    async fn execute(&self, context: &mut HookContext) -> Result<HookResult> {
        tracing::debug!("StateChangeHook '{}': Executing", self.metadata.name);

        // Extract state context
        let state_context = if let Some(state_data) = context.data.get("state_context") {
            serde_json::from_value(state_data.clone())?
        } else {
            StateContext {
                state_type: StateType::Kernel,
                previous_state: context.data.get("previous_state").cloned(),
                new_state: context
                    .data
                    .get("new_state")
                    .cloned()
                    .unwrap_or(Value::Null),
                change_reason: context
                    .get_metadata("change_reason")
                    .unwrap_or("unknown")
                    .to_string(),
                session_id: context.get_metadata("session_id").map(String::from),
                persist: true,
                metadata: HashMap::new(),
            }
        };

        self.handler
            .handle_state_change(&state_context, context)
            .await
    }

    fn metadata(&self) -> HookMetadata {
        self.metadata.clone()
    }

    fn should_execute(&self, _context: &HookContext) -> bool {
        true
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

/// Kernel hook manager for registering and managing kernel-specific hooks
pub struct KernelHookManager {
    pre_execute: Vec<PreExecuteHook>,
    post_execute: Vec<PostExecuteHook>,
    pre_debug: Vec<PreDebugHook>,
    state_change: Vec<StateChangeHook>,
}

impl KernelHookManager {
    /// Create a new kernel hook manager
    pub fn new() -> Self {
        Self {
            pre_execute: Vec::new(),
            post_execute: Vec::new(),
            pre_debug: Vec::new(),
            state_change: Vec::new(),
        }
    }

    /// Register a pre-execute hook
    pub fn register_pre_execute(&mut self, hook: PreExecuteHook) {
        self.pre_execute.push(hook);
    }

    /// Register a post-execute hook
    pub fn register_post_execute(&mut self, hook: PostExecuteHook) {
        self.post_execute.push(hook);
    }

    /// Register a pre-debug hook
    pub fn register_pre_debug(&mut self, hook: PreDebugHook) {
        self.pre_debug.push(hook);
    }

    /// Register a state change hook
    pub fn register_state_change(&mut self, hook: StateChangeHook) {
        self.state_change.push(hook);
    }

    /// Get all registered hooks as kernel hooks
    pub fn get_all_hooks(&self) -> Vec<KernelHook> {
        let mut hooks = Vec::new();

        for hook in &self.pre_execute {
            hooks.push(KernelHook::PreExecute(hook.clone()));
        }

        for hook in &self.post_execute {
            hooks.push(KernelHook::PostExecute(hook.clone()));
        }

        for hook in &self.pre_debug {
            hooks.push(KernelHook::PreDebug(hook.clone()));
        }

        for hook in &self.state_change {
            hooks.push(KernelHook::StateChange(hook.clone()));
        }

        hooks
    }

    /// Get hook count by type
    pub fn hook_counts(&self) -> (usize, usize, usize, usize) {
        (
            self.pre_execute.len(),
            self.post_execute.len(),
            self.pre_debug.len(),
            self.state_change.len(),
        )
    }
}

impl Default for KernelHookManager {
    fn default() -> Self {
        Self::new()
    }
}

// Manual Clone implementations for hooks that need to be cloned
impl Clone for PreExecuteHook {
    fn clone(&self) -> Self {
        Self {
            handler: self.handler.clone(),
            metadata: self.metadata.clone(),
        }
    }
}

impl Clone for PostExecuteHook {
    fn clone(&self) -> Self {
        Self {
            handler: self.handler.clone(),
            metadata: self.metadata.clone(),
        }
    }
}

impl Clone for PreDebugHook {
    fn clone(&self) -> Self {
        Self {
            handler: self.handler.clone(),
            metadata: self.metadata.clone(),
        }
    }
}

impl Clone for StateChangeHook {
    fn clone(&self) -> Self {
        Self {
            handler: self.handler.clone(),
            metadata: self.metadata.clone(),
        }
    }
}

// Manual Debug implementations for hooks containing trait objects
impl std::fmt::Debug for PreExecuteHook {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PreExecuteHook")
            .field("metadata", &self.metadata)
            .field("handler", &"<trait object>")
            .finish()
    }
}

impl std::fmt::Debug for PostExecuteHook {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PostExecuteHook")
            .field("metadata", &self.metadata)
            .field("handler", &"<trait object>")
            .finish()
    }
}

impl std::fmt::Debug for PreDebugHook {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PreDebugHook")
            .field("metadata", &self.metadata)
            .field("handler", &"<trait object>")
            .finish()
    }
}

impl std::fmt::Debug for StateChangeHook {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("StateChangeHook")
            .field("metadata", &self.metadata)
            .field("handler", &"<trait object>")
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use llmspell_hooks::{ComponentId, ComponentType, HookPoint};

    struct TestPreExecuteHandler;

    #[async_trait]
    impl PreExecuteHandler for TestPreExecuteHandler {
        async fn handle_pre_execute(
            &self,
            _execution_context: &ExecutionContext,
            _hook_context: &mut HookContext,
        ) -> Result<HookResult> {
            Ok(HookResult::Continue)
        }
    }

    #[tokio::test]
    async fn test_pre_execute_hook() {
        let handler = Arc::new(TestPreExecuteHandler);
        let hook = PreExecuteHook::new("test_pre_execute", handler);

        let component_id = ComponentId::new(ComponentType::System, "test".to_string());
        let mut context = HookContext::new(HookPoint::BeforeAgentExecution, component_id);
        context.insert_metadata("code".to_string(), "print('test')".to_string());

        let result = hook.execute(&mut context).await.unwrap();
        assert!(matches!(result, HookResult::Continue));
    }

    #[test]
    fn test_kernel_hook_manager() {
        let mut manager = KernelHookManager::new();

        let handler = Arc::new(TestPreExecuteHandler);
        let hook = PreExecuteHook::new("test", handler);
        manager.register_pre_execute(hook);

        let (pre, post, debug, state) = manager.hook_counts();
        assert_eq!(pre, 1);
        assert_eq!(post, 0);
        assert_eq!(debug, 0);
        assert_eq!(state, 0);

        let all_hooks = manager.get_all_hooks();
        assert_eq!(all_hooks.len(), 1);
    }

    #[test]
    fn test_execution_context_serialization() {
        let context = ExecutionContext {
            code: "test code".to_string(),
            session_id: "session1".to_string(),
            message_id: "msg1".to_string(),
            silent: false,
            user_expressions: HashMap::new(),
            estimated_duration: Some(Duration::from_secs(1)),
            priority: ExecutionPriority::High,
            parameters: Value::Null,
        };

        let serialized = serde_json::to_string(&context).unwrap();
        let deserialized: ExecutionContext = serde_json::from_str(&serialized).unwrap();

        assert_eq!(context.code, deserialized.code);
        assert_eq!(context.session_id, deserialized.session_id);
        assert_eq!(context.priority, deserialized.priority);
    }
}
