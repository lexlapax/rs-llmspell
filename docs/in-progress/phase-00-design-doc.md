# Phase 0: Foundation Infrastructure - Design Document

**Version**: 1.0  
**Date**: June 2025  
**Status**: Implementation Ready  
**Phase**: 0 (Foundation Infrastructure)  
**Timeline**: Weeks 1-2  
**Priority**: CRITICAL (MVP Prerequisite)

> **📋 Detailed Implementation Guide**: This document provides complete specifications for implementing Phase 0 foundation infrastructure for rs-llmspell.

---

## Phase Overview

### Goal
Establish core project infrastructure and build system that serves as the foundation for all subsequent phases.

### Core Principles
- **Zero Warnings Policy**: All code must compile without warnings
- **Documentation First**: Every component must be documented before implementation
- **Test-Driven Foundation**: Core traits tested before implementation
- **CI/CD Ready**: Pipeline validates every commit

### Success Criteria
- [ ] All crates compile without warnings
- [ ] Basic trait hierarchy compiles with full documentation
- [ ] CI runs successfully on Linux with comprehensive test suite
- [ ] Documentation builds without errors and generates complete API docs
- [ ] `cargo test` passes for all foundation tests with 100% coverage

---

## 1. Implementation Specifications

### 1.1 Cargo Workspace Structure

**Complete Workspace Layout:**

```toml
# /Cargo.toml
[workspace]
resolver = "2"
members = [
    "llmspell-cli",
    "llmspell-core", 
    "llmspell-agents",
    "llmspell-tools",
    "llmspell-workflows",
    "llmspell-bridge",
    "llmspell-providers",
    "llmspell-storage",
    "llmspell-config",
    "llmspell-security",
    "llmspell-hooks",
    "llmspell-testing",
]

[workspace.package]
version = "0.1.0"
edition = "2021"
license = "MIT OR Apache-2.0"
repository = "https://github.com/lexlapax/rs-llmspell"
authors = ["Rs-LLMSpell Contributors"]
categories = ["development-tools", "command-line-utilities"]
keywords = ["llm", "ai", "agents", "scripting", "automation"]

[workspace.dependencies]
# Core async and runtime
tokio = { version = "1.40", features = ["full"] }
tokio-stream = "0.1"
futures = "0.3"

# Error handling and result types  
anyhow = "1.0"
thiserror = "1.0"

# Serialization and data
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
toml = "0.8"

# Logging and observability
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter", "json"] }

# CLI and configuration
clap = { version = "4.4", features = ["derive", "env"] }

# Testing infrastructure
criterion = { version = "0.5", features = ["html_reports"] }
mockall = "0.12"
proptest = "1.4"

# Unique IDs and versioning
uuid = { version = "1.6", features = ["v4", "serde"] }
semver = { version = "1.0", features = ["serde"] }

[profile.dev]
debug = true
opt-level = 0

[profile.release]
debug = false
opt-level = 3
lto = true
codegen-units = 1
panic = "abort"

[profile.test]
debug = true
opt-level = 1
```

### 1.2 Core Trait Definitions

**BaseAgent Foundational Trait:**

```rust
// llmspell-core/src/traits/base_agent.rs
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Unique identifier for any component in the system
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ComponentId(pub Uuid);

impl ComponentId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
    
    pub fn from_name(name: &str) -> Self {
        Self(Uuid::new_v5(&Uuid::NAMESPACE_DNS, name.as_bytes()))
    }
}

impl std::fmt::Display for ComponentId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Version information for components
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Version {
    pub major: u32,
    pub minor: u32,
    pub patch: u32,
}

impl Version {
    pub fn new(major: u32, minor: u32, patch: u32) -> Self {
        Self { major, minor, patch }
    }
}

impl Default for Version {
    fn default() -> Self {
        Self::new(0, 1, 0)
    }
}

impl std::fmt::Display for Version {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}.{}.{}", self.major, self.minor, self.patch)
    }
}

/// Metadata for any component
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentMetadata {
    pub tags: Vec<String>,
    pub capabilities: Vec<String>,
    pub dependencies: Vec<ComponentId>,
    pub provides: Vec<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
    pub custom: HashMap<String, serde_json::Value>,
}

impl Default for ComponentMetadata {
    fn default() -> Self {
        let now = chrono::Utc::now();
        Self {
            tags: Vec::new(),
            capabilities: Vec::new(),
            dependencies: Vec::new(),
            provides: Vec::new(),
            created_at: now,
            updated_at: now,
            custom: HashMap::new(),
        }
    }
}

/// Input provided to any component execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentInput {
    pub content: String,
    pub parameters: HashMap<String, serde_json::Value>,
    pub context: ExecutionContext,
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Output produced by any component execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentOutput {
    pub content: String,
    pub data: HashMap<String, serde_json::Value>,
    pub metadata: HashMap<String, serde_json::Value>,
    pub execution_info: ExecutionInfo,
}

/// Execution context for component operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionContext {
    pub session_id: String,
    pub trace_id: String,
    pub parent_span_id: Option<String>,
    pub security_context: SecurityContext,
    pub resource_limits: ResourceLimits,
    pub custom: HashMap<String, serde_json::Value>,
}

/// Security context for operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityContext {
    pub profile: String,
    pub permissions: Vec<String>,
    pub restricted_operations: Vec<String>,
    pub sandbox_enabled: bool,
}

impl Default for SecurityContext {
    fn default() -> Self {
        Self {
            profile: "medium".to_string(),
            permissions: vec!["read".to_string()],
            restricted_operations: Vec::new(),
            sandbox_enabled: true,
        }
    }
}

/// Resource limits for component execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceLimits {
    pub max_memory_mb: Option<u64>,
    pub max_execution_time_ms: Option<u64>,
    pub max_tool_calls: Option<u32>,
    pub max_iterations: Option<u32>,
}

impl Default for ResourceLimits {
    fn default() -> Self {
        Self {
            max_memory_mb: Some(100),
            max_execution_time_ms: Some(30000),
            max_tool_calls: Some(10),
            max_iterations: Some(5),
        }
    }
}

/// Information about component execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionInfo {
    pub duration_ms: u64,
    pub memory_used_mb: Option<f64>,
    pub tool_calls_made: u32,
    pub iterations_completed: u32,
    pub error: Option<String>,
}

impl Default for ExecutionInfo {
    fn default() -> Self {
        Self {
            duration_ms: 0,
            memory_used_mb: None,
            tool_calls_made: 0,
            iterations_completed: 0,
            error: None,
        }
    }
}

/// Forward declaration for Tool trait
pub use crate::traits::tool::Tool;

/// The foundational trait that all components implement
/// This provides common capabilities for agents, tools, and workflows
#[async_trait]
pub trait BaseAgent: Send + Sync + std::fmt::Debug {
    // Identity and Metadata
    fn id(&self) -> &ComponentId;
    fn name(&self) -> &str;
    fn description(&self) -> &str;
    fn version(&self) -> &Version;
    fn metadata(&self) -> &ComponentMetadata;
    
    // Core Execution Interface
    async fn execute(&mut self, input: AgentInput) -> crate::Result<AgentOutput>;
    async fn validate_input(&self, input: &AgentInput) -> crate::Result<ValidationResult>;
    async fn prepare_execution(&mut self, input: &AgentInput) -> crate::Result<()>;
    async fn cleanup_execution(&mut self) -> crate::Result<()>;
    
    // Tool Management
    fn tools(&self) -> &[Arc<dyn Tool>];
    async fn add_tool(&mut self, tool: Arc<dyn Tool>) -> crate::Result<()>;
    async fn remove_tool(&mut self, tool_id: &ComponentId) -> crate::Result<bool>;
    async fn find_tool(&self, name: &str) -> Option<Arc<dyn Tool>>;
    
    // State Management
    async fn get_state(&self) -> crate::Result<ComponentState>;
    async fn set_state(&mut self, state: ComponentState) -> crate::Result<()>;
    async fn reset_state(&mut self) -> crate::Result<()>;
    
    // Capability and Dependency Management
    fn capabilities(&self) -> &[String];
    fn dependencies(&self) -> &[ComponentId];
    fn provides(&self) -> &[String];
    fn can_handle(&self, input: &AgentInput) -> bool;
    
    // Lifecycle Management
    async fn initialize(&mut self) -> crate::Result<()>;
    async fn shutdown(&mut self) -> crate::Result<()>;
    fn is_healthy(&self) -> bool;
    
    // Configuration
    async fn configure(&mut self, config: ComponentConfig) -> crate::Result<()>;
    fn get_configuration(&self) -> ComponentConfig;
    
    // Clone support for composition patterns
    fn clone_box(&self) -> Box<dyn BaseAgent>;
}

/// Validation result for input validation
#[derive(Debug, Clone)]
pub struct ValidationResult {
    pub valid: bool,
    pub errors: Vec<ValidationError>,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct ValidationError {
    pub field: String,
    pub message: String,
    pub error_code: String,
}

/// Component state representation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentState {
    pub iteration_count: u32,
    pub last_execution: Option<chrono::DateTime<chrono::Utc>>,
    pub accumulated_data: HashMap<String, serde_json::Value>,
    pub internal_state: HashMap<String, serde_json::Value>,
}

impl Default for ComponentState {
    fn default() -> Self {
        Self {
            iteration_count: 0,
            last_execution: None,
            accumulated_data: HashMap::new(),
            internal_state: HashMap::new(),
        }
    }
}

/// Component configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentConfig {
    pub enabled: bool,
    pub timeout_ms: u64,
    pub retry_count: u32,
    pub circuit_breaker_enabled: bool,
    pub custom_settings: HashMap<String, serde_json::Value>,
}

impl Default for ComponentConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            timeout_ms: 30000,
            retry_count: 3,
            circuit_breaker_enabled: true,
            custom_settings: HashMap::new(),
        }
    }
}
```

**Agent Trait Definition:**

```rust
// llmspell-core/src/traits/agent.rs
use async_trait::async_trait;
use super::base_agent::{BaseAgent, AgentInput, AgentOutput};
use crate::providers::LLMProvider;

/// Specialized trait for LLM-powered agents
#[async_trait]
pub trait Agent: BaseAgent {
    // LLM Integration
    fn llm_provider(&self) -> &dyn LLMProvider;
    async fn set_llm_provider(&mut self, provider: Box<dyn LLMProvider>) -> crate::Result<()>;
    
    // Prompt Management
    fn system_prompt(&self) -> &str;
    async fn set_system_prompt(&mut self, prompt: String) -> crate::Result<()>;
    fn user_prompt_template(&self) -> &str;
    async fn set_user_prompt_template(&mut self, template: String) -> crate::Result<()>;
    
    // LLM-specific execution
    async fn generate_response(&mut self, prompt: &str) -> crate::Result<String>;
    async fn generate_with_tools(&mut self, prompt: &str) -> crate::Result<AgentOutput>;
    async fn chat(&mut self, message: &str) -> crate::Result<String>;
    
    // Conversation Management
    async fn start_conversation(&mut self) -> crate::Result<String>;
    async fn continue_conversation(&mut self, message: &str) -> crate::Result<String>;
    async fn end_conversation(&mut self) -> crate::Result<()>;
    fn conversation_history(&self) -> &[ConversationMessage];
    async fn clear_conversation(&mut self) -> crate::Result<()>;
    
    // Tool Integration for Agents
    async fn execute_tool_by_name(&mut self, name: &str, input: serde_json::Value) -> crate::Result<serde_json::Value>;
    fn available_tools_description(&self) -> String;
    
    // Agent-specific State
    async fn save_conversation_state(&self) -> crate::Result<ConversationState>;
    async fn restore_conversation_state(&mut self, state: ConversationState) -> crate::Result<()>;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversationMessage {
    pub role: MessageRole,
    pub content: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub metadata: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MessageRole {
    System,
    User,
    Assistant,
    Tool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversationState {
    pub messages: Vec<ConversationMessage>,
    pub context: HashMap<String, serde_json::Value>,
    pub system_prompt: String,
}
```

**Tool Trait Definition:**

```rust
// llmspell-core/src/traits/tool.rs
use async_trait::async_trait;
use super::base_agent::{BaseAgent, ComponentId};
use schemars::{JsonSchema, schema_for};

/// Tool-specific input with schema validation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolInput {
    pub parameters: serde_json::Value,
    pub context: ExecutionContext,
}

/// Tool-specific output with rich metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolOutput {
    pub result: serde_json::Value,
    pub success: bool,
    pub error_message: Option<String>,
    pub metadata: HashMap<String, serde_json::Value>,
    pub execution_time_ms: u64,
}

/// JSON Schema definition for tool parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolSchema {
    pub input_schema: serde_json::Value,
    pub output_schema: serde_json::Value,
    pub examples: Vec<ToolExample>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolExample {
    pub name: String,
    pub description: String,
    pub input: serde_json::Value,
    pub output: serde_json::Value,
}

/// Specialized trait for tool components
#[async_trait]
pub trait Tool: BaseAgent {
    // Tool Execution
    async fn execute_tool(&self, input: ToolInput) -> crate::Result<ToolOutput>;
    
    // Schema and Validation
    fn tool_schema(&self) -> &ToolSchema;
    async fn validate_parameters(&self, parameters: &serde_json::Value) -> crate::Result<ValidationResult>;
    
    // Tool Metadata
    fn tool_category(&self) -> ToolCategory;
    fn is_async(&self) -> bool;
    fn requires_confirmation(&self) -> bool;
    fn security_level(&self) -> SecurityLevel;
    
    // Tool Configuration
    fn default_timeout_ms(&self) -> u64;
    fn supports_streaming(&self) -> bool;
    fn is_deterministic(&self) -> bool;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ToolCategory {
    FileSystem,
    Network,
    Computation,
    DataProcessing,
    Communication,
    System,
    AI,
    Custom(String),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum SecurityLevel {
    Safe,        // No system access
    Restricted,  // Limited system access
    Elevated,    // Broad system access
    Dangerous,   // Full system access
}
```

**Workflow Trait Definition:**

```rust
// llmspell-core/src/traits/workflow.rs
use async_trait::async_trait;
use super::base_agent::{BaseAgent, AgentInput, AgentOutput, ComponentId};

/// Step in a workflow execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowStep {
    pub id: String,
    pub name: String,
    pub component_id: ComponentId,
    pub input_mapping: HashMap<String, String>,
    pub output_mapping: HashMap<String, String>,
    pub condition: Option<String>,
    pub retry_policy: RetryPolicy,
    pub timeout_ms: Option<u64>,
}

/// Retry policy for workflow steps
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryPolicy {
    pub max_attempts: u32,
    pub backoff_strategy: BackoffStrategy,
    pub retry_on_errors: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BackoffStrategy {
    Fixed { delay_ms: u64 },
    Exponential { initial_delay_ms: u64, multiplier: f64, max_delay_ms: u64 },
    Linear { initial_delay_ms: u64, increment_ms: u64 },
}

/// Result of a workflow step execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StepResult {
    pub step_id: String,
    pub success: bool,
    pub output: Option<AgentOutput>,
    pub error: Option<String>,
    pub execution_time_ms: u64,
    pub retry_count: u32,
}

/// Complete workflow execution result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowResult {
    pub success: bool,
    pub step_results: Vec<StepResult>,
    pub total_execution_time_ms: u64,
    pub final_output: Option<AgentOutput>,
    pub error: Option<String>,
}

/// Specialized trait for workflow orchestration components
#[async_trait]
pub trait Workflow: BaseAgent {
    // Workflow Definition
    fn steps(&self) -> &[WorkflowStep];
    async fn add_step(&mut self, step: WorkflowStep) -> crate::Result<()>;
    async fn remove_step(&mut self, step_id: &str) -> crate::Result<bool>;
    async fn update_step(&mut self, step_id: &str, step: WorkflowStep) -> crate::Result<()>;
    
    // Workflow Execution
    async fn execute_workflow(&mut self, input: AgentInput) -> crate::Result<WorkflowResult>;
    async fn execute_step(&mut self, step_id: &str, input: AgentInput) -> crate::Result<StepResult>;
    
    // State Management
    async fn pause_execution(&mut self) -> crate::Result<()>;
    async fn resume_execution(&mut self) -> crate::Result<()>;
    async fn cancel_execution(&mut self) -> crate::Result<()>;
    fn execution_status(&self) -> WorkflowStatus;
    
    // Workflow Analysis
    fn validate_workflow(&self) -> crate::Result<ValidationResult>;
    fn get_execution_plan(&self) -> ExecutionPlan;
    async fn simulate_execution(&self, input: AgentInput) -> crate::Result<SimulationResult>;
    
    // Component Dependencies
    fn required_components(&self) -> Vec<ComponentId>;
    async fn resolve_component(&self, id: &ComponentId) -> crate::Result<Arc<dyn BaseAgent>>;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WorkflowStatus {
    NotStarted,
    Running { current_step: String },
    Paused { current_step: String },
    Completed,
    Failed { error: String },
    Cancelled,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionPlan {
    pub total_steps: usize,
    pub estimated_duration_ms: u64,
    pub parallel_branches: Vec<Vec<String>>,
    pub critical_path: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimulationResult {
    pub would_succeed: bool,
    pub estimated_duration_ms: u64,
    pub potential_errors: Vec<String>,
    pub resource_requirements: ResourceRequirements,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceRequirements {
    pub max_memory_mb: u64,
    pub max_concurrent_components: u32,
    pub external_dependencies: Vec<String>,
}
```

### 1.3 Error Handling System

**Comprehensive Error Types:**

```rust
// llmspell-core/src/error.rs
use thiserror::Error;

/// Main error type for the entire rs-llmspell system
#[derive(Error, Debug)]
pub enum LLMSpellError {
    // Component Errors
    #[error("Component error: {message}")]
    Component { message: String, component_id: Option<String> },
    
    #[error("Agent execution error: {message}")]
    AgentExecution { message: String, agent_id: String },
    
    #[error("Tool execution error: {message}")]
    ToolExecution { message: String, tool_name: String },
    
    #[error("Workflow execution error: {message}")]
    WorkflowExecution { message: String, workflow_id: String, step_id: Option<String> },
    
    // Configuration Errors
    #[error("Configuration error: {message}")]
    Configuration { message: String, config_path: Option<String> },
    
    #[error("Invalid configuration value: {field} = {value}")]
    InvalidConfiguration { field: String, value: String },
    
    // Provider Errors
    #[error("LLM provider error: {message}")]
    LLMProvider { message: String, provider: String },
    
    #[error("Provider not found: {provider}")]
    ProviderNotFound { provider: String },
    
    // Script Engine Errors
    #[error("Script execution error: {message}")]
    ScriptExecution { message: String, engine: String, line: Option<u32> },
    
    #[error("Script compilation error: {message}")]
    ScriptCompilation { message: String, engine: String },
    
    // State Management Errors
    #[error("State error: {message}")]
    State { message: String },
    
    #[error("State serialization error: {message}")]
    StateSerialization { message: String },
    
    // Security Errors
    #[error("Security violation: {message}")]
    Security { message: String, violation_type: String },
    
    #[error("Permission denied: {operation}")]
    PermissionDenied { operation: String },
    
    // Resource Errors
    #[error("Resource limit exceeded: {resource} limit: {limit}")]
    ResourceLimitExceeded { resource: String, limit: String },
    
    #[error("Timeout error: operation timed out after {timeout_ms}ms")]
    Timeout { timeout_ms: u64 },
    
    // Validation Errors
    #[error("Validation error: {field}: {message}")]
    Validation { field: String, message: String },
    
    #[error("Schema validation error: {message}")]
    SchemaValidation { message: String, schema_path: Option<String> },
    
    // I/O and External Errors
    #[error("I/O error: {message}")]
    Io { 
        message: String,
        #[source]
        source: Option<std::io::Error>
    },
    
    #[error("Network error: {message}")]
    Network { message: String, url: Option<String> },
    
    #[error("Database error: {message}")]
    Database { message: String },
    
    // System Errors
    #[error("Internal error: {message}")]
    Internal { message: String },
    
    #[error("Not implemented: {feature}")]
    NotImplemented { feature: String },
    
    #[error("Dependency not found: {dependency}")]
    DependencyNotFound { dependency: String },
}

impl LLMSpellError {
    pub fn component(message: impl Into<String>) -> Self {
        Self::Component { 
            message: message.into(), 
            component_id: None 
        }
    }
    
    pub fn component_with_id(message: impl Into<String>, component_id: impl Into<String>) -> Self {
        Self::Component { 
            message: message.into(), 
            component_id: Some(component_id.into()) 
        }
    }
    
    pub fn agent_execution(message: impl Into<String>, agent_id: impl Into<String>) -> Self {
        Self::AgentExecution { 
            message: message.into(), 
            agent_id: agent_id.into() 
        }
    }
    
    pub fn tool_execution(message: impl Into<String>, tool_name: impl Into<String>) -> Self {
        Self::ToolExecution { 
            message: message.into(), 
            tool_name: tool_name.into() 
        }
    }
    
    pub fn timeout(timeout_ms: u64) -> Self {
        Self::Timeout { timeout_ms }
    }
    
    pub fn security(message: impl Into<String>, violation_type: impl Into<String>) -> Self {
        Self::Security { 
            message: message.into(), 
            violation_type: violation_type.into() 
        }
    }
    
    pub fn validation(field: impl Into<String>, message: impl Into<String>) -> Self {
        Self::Validation { 
            field: field.into(), 
            message: message.into() 
        }
    }
    
    pub fn internal(message: impl Into<String>) -> Self {
        Self::Internal { message: message.into() }
    }
    
    /// Get error category for metrics and logging
    pub fn category(&self) -> &'static str {
        match self {
            Self::Component { .. } => "component",
            Self::AgentExecution { .. } => "agent_execution",
            Self::ToolExecution { .. } => "tool_execution", 
            Self::WorkflowExecution { .. } => "workflow_execution",
            Self::Configuration { .. } | Self::InvalidConfiguration { .. } => "configuration",
            Self::LLMProvider { .. } | Self::ProviderNotFound { .. } => "llm_provider",
            Self::ScriptExecution { .. } | Self::ScriptCompilation { .. } => "script_engine",
            Self::State { .. } | Self::StateSerialization { .. } => "state_management",
            Self::Security { .. } | Self::PermissionDenied { .. } => "security",
            Self::ResourceLimitExceeded { .. } | Self::Timeout { .. } => "resources",
            Self::Validation { .. } | Self::SchemaValidation { .. } => "validation",
            Self::Io { .. } | Self::Network { .. } | Self::Database { .. } => "external",
            Self::Internal { .. } | Self::NotImplemented { .. } | Self::DependencyNotFound { .. } => "system",
        }
    }
    
    /// Check if error is retryable
    pub fn is_retryable(&self) -> bool {
        matches!(self, 
            Self::Network { .. } | 
            Self::Database { .. } |
            Self::LLMProvider { .. } |
            Self::Timeout { .. }
        )
    }
    
    /// Get error severity level
    pub fn severity(&self) -> ErrorSeverity {
        match self {
            Self::Internal { .. } | Self::Security { .. } => ErrorSeverity::Critical,
            Self::Component { .. } | Self::AgentExecution { .. } => ErrorSeverity::High,
            Self::ToolExecution { .. } | Self::Configuration { .. } => ErrorSeverity::Medium,
            Self::Validation { .. } | Self::ScriptExecution { .. } => ErrorSeverity::Low,
            _ => ErrorSeverity::Medium,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorSeverity {
    Low,
    Medium, 
    High,
    Critical,
}

/// Result type used throughout the codebase
pub type Result<T> = std::result::Result<T, LLMSpellError>;

/// Convenience macros for common error patterns
#[macro_export]
macro_rules! component_error {
    ($msg:expr) => {
        $crate::error::LLMSpellError::component($msg)
    };
    ($msg:expr, $id:expr) => {
        $crate::error::LLMSpellError::component_with_id($msg, $id)
    };
}

#[macro_export]
macro_rules! tool_error {
    ($msg:expr, $tool:expr) => {
        $crate::error::LLMSpellError::tool_execution($msg, $tool)
    };
}

#[macro_export]
macro_rules! validation_error {
    ($field:expr, $msg:expr) => {
        $crate::error::LLMSpellError::validation($field, $msg)
    };
}
```

### 1.4 Logging Infrastructure

**Structured Logging Setup:**

```rust
// llmspell-core/src/logging.rs
use tracing::{info, warn, error, debug, trace};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter, Registry};
use std::sync::Once;

static INIT: Once = Once::new();

/// Initialize the logging system for rs-llmspell
pub fn init_logging(level: Option<&str>) -> crate::Result<()> {
    INIT.call_once(|| {
        let env_filter = EnvFilter::try_from_default_env()
            .unwrap_or_else(|_| {
                EnvFilter::new(level.unwrap_or("info"))
            });

        let fmt_layer = tracing_subscriber::fmt::layer()
            .with_target(true)
            .with_thread_ids(true)
            .with_file(true)
            .with_line_number(true)
            .json();

        Registry::default()
            .with(env_filter)
            .with(fmt_layer)
            .init();
    });

    info!("Rs-LLMSpell logging initialized");
    Ok(())
}

/// Structured logging macros for consistent component logging
#[macro_export]
macro_rules! log_component_event {
    ($level:ident, $component_id:expr, $event:expr, $($key:expr => $value:expr),*) => {
        tracing::$level!(
            component_id = %$component_id,
            event = $event,
            $($key = ?$value,)*
        );
    };
}

#[macro_export]
macro_rules! log_execution_start {
    ($component_id:expr, $operation:expr) => {
        log_component_event!(info, $component_id, "execution_start", 
            operation => $operation,
            timestamp => chrono::Utc::now()
        );
    };
}

#[macro_export]
macro_rules! log_execution_end {
    ($component_id:expr, $operation:expr, $duration_ms:expr, $success:expr) => {
        log_component_event!(info, $component_id, "execution_end",
            operation => $operation,
            duration_ms => $duration_ms,
            success => $success,
            timestamp => chrono::Utc::now()
        );
    };
}

#[macro_export]
macro_rules! log_error {
    ($component_id:expr, $error:expr) => {
        log_component_event!(error, $component_id, "error",
            error => %$error,
            error_category => $error.category(),
            severity => ?$error.severity(),
            retryable => $error.is_retryable()
        );
    };
}
```

---

## 2. Step-by-Step Implementation Guidance

### 2.1 Implementation Order

**Phase 0.1: Workspace Setup (Day 1)**
1. Create root `Cargo.toml` with workspace configuration
2. Create all crate directories with basic `Cargo.toml` files
3. Set up basic `lib.rs` files with module structure
4. Verify `cargo check` passes for entire workspace

**Phase 0.2: Core Traits (Days 1-2)**
1. Implement `ComponentId`, `Version`, and metadata types
2. Define `BaseAgent` trait with full method signatures
3. Define `Agent` trait extending `BaseAgent`
4. Define `Tool` trait with schema support
5. Define `Workflow` trait with step management
6. Verify all traits compile without errors

**Phase 0.3: Error Handling (Day 2)**
1. Implement comprehensive `LLMSpellError` enum
2. Add error convenience macros
3. Define `Result<T>` type alias
4. Add error categorization and severity methods
5. Write unit tests for error handling

**Phase 0.4: Logging Infrastructure (Day 3)**
1. Set up `tracing` with structured JSON output
2. Implement logging initialization function
3. Create logging macros for consistent component logging
4. Add environment variable configuration
5. Test logging output format

**Phase 0.5: Documentation (Days 3-4)**
1. Add comprehensive `rustdoc` comments to all public APIs
2. Create crate-level documentation with examples
3. Set up `cargo doc` generation
4. Add README files for each crate
5. Verify documentation builds without warnings

**Phase 0.6: Testing Framework (Days 4-5)**
1. Set up `criterion` for benchmarking
2. Configure `mockall` for trait mocking
3. Add `proptest` for property-based testing
4. Create test utilities and helpers
5. Write foundation tests for all traits

**Phase 0.7: CI/CD Pipeline (Days 5-6)**
1. Create GitHub Actions workflow
2. Configure testing on Linux (Ubuntu latest)
3. Add clippy linting with deny warnings
4. Add cargo formatting checks
5. Set up documentation generation and deployment

### 2.2 Code Patterns and Best Practices

**Consistent Module Structure:**
```rust
// Each crate follows this pattern:
// src/
//   lib.rs              - Public API exports
//   error.rs            - Crate-specific errors  
//   types.rs            - Data types and structs
//   traits/             - Trait definitions
//     mod.rs            - Trait exports
//     base_agent.rs     - BaseAgent trait
//     agent.rs          - Agent trait
//     tool.rs           - Tool trait
//     workflow.rs       - Workflow trait
//   implementations/    - Concrete implementations
//   testing/            - Test utilities
//   examples/           - Usage examples
```

**Documentation Standards:**
```rust
/// A comprehensive summary of what this does
/// 
/// This should explain the purpose, behavior, and any important
/// implementation details. Include examples when helpful.
/// 
/// # Arguments
/// 
/// * `param1` - Description of first parameter
/// * `param2` - Description of second parameter
/// 
/// # Returns
/// 
/// Description of return value and any error conditions
/// 
/// # Examples
/// 
/// ```rust
/// use llmspell_core::ComponentId;
/// 
/// let id = ComponentId::new();
/// assert!(!id.to_string().is_empty());
/// ```
/// 
/// # Errors
/// 
/// This function returns an error when:
/// * Condition 1 occurs
/// * Condition 2 is violated
pub async fn example_function(param1: &str, param2: u32) -> crate::Result<String> {
    // Implementation
}
```

**Error Handling Patterns:**
```rust
// Prefer explicit error types over generic errors
match some_operation() {
    Ok(result) => process_result(result),
    Err(e) => match e {
        SpecificError::ConfigNotFound => handle_missing_config(),
        SpecificError::InvalidValue { field } => {
            return Err(validation_error!(field, "Invalid configuration value"));
        }
        _ => return Err(e.into()),
    }
}

// Use the ? operator for error propagation
let result = some_fallible_operation()
    .map_err(|e| component_error!("Operation failed", component_id))?;
```

**Testing Patterns:**
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use mockall::predicate::*;
    
    #[tokio::test]
    async fn test_component_execution_success() {
        // Arrange
        let mut component = create_test_component();
        let input = create_test_input();
        
        // Act
        let result = component.execute(input).await;
        
        // Assert
        assert!(result.is_ok());
        let output = result.unwrap();
        assert_eq!(output.content, "expected output");
    }
    
    #[tokio::test] 
    async fn test_component_execution_error() {
        // Test error conditions
    }
    
    // Property-based testing example
    use proptest::prelude::*;
    
    proptest! {
        #[test]
        fn test_component_id_roundtrip(name in "\\PC*") {
            let id1 = ComponentId::from_name(&name);
            let id2 = ComponentId::from_name(&name);
            prop_assert_eq!(id1, id2);
        }
    }
}
```

### 2.3 Common Pitfalls and Avoidance

**Pitfall 1: Async Trait Object Issues**
```rust
// Problem: Async traits don't work well with trait objects
// Solution: Use async_trait and careful boxing

#[async_trait]
pub trait MyTrait {
    async fn async_method(&self) -> Result<String>;
}

// When using as trait object:
let boxed: Box<dyn MyTrait + Send + Sync> = Box::new(implementation);
```

**Pitfall 2: Complex Error Chains**
```rust
// Problem: Losing error context through conversions
// Solution: Use thiserror with source preservation

#[derive(Error, Debug)]
#[error("High-level operation failed")]
pub struct HighLevelError {
    #[source]
    source: LowerLevelError,
}
```

**Pitfall 3: State Management Races**
```rust
// Problem: Concurrent access to component state
// Solution: Use Arc<RwLock<T>> for shared state

pub struct ComponentImpl {
    state: Arc<RwLock<ComponentState>>,
    // other fields
}

impl ComponentImpl {
    async fn get_state(&self) -> Result<ComponentState> {
        let state = self.state.read().await;
        Ok(state.clone())
    }
}
```

### 2.4 Testing Strategies

**Unit Testing:**
- Test each trait method in isolation
- Mock dependencies using `mockall`
- Test error conditions thoroughly
- Verify state changes correctly

**Integration Testing:**
- Test trait implementations together
- Verify error propagation through layers
- Test configuration loading and validation

**Property-Based Testing:**
- Use `proptest` for ID generation functions
- Test serialization/deserialization roundtrips
- Verify invariants hold across operations

**Performance Testing:**
- Use `criterion` for micro-benchmarks
- Measure trait method execution times
- Profile memory usage patterns

---

## 3. Integration and Transition Planning

### 3.1 Integration with Phase 1

**Preparation for Script Runtime:**
- All traits must be `Send + Sync` for async runtime
- Error types must be serializable for script bridge
- Component IDs must be string-convertible for script access
- State types must support JSON serialization

**Bridge Compatibility:**
- All public methods must be script-callable
- Parameter types must convert to/from JSON
- Async methods must work with cooperative scheduling
- Error messages must be script-friendly

### 3.2 Breaking Changes Documentation

**Allowed Changes in Phase 0:**
- Trait method signature refinements
- Error type additions and modifications
- New metadata fields
- Additional validation rules

**Migration Guides:**
- Document any trait signature changes
- Provide upgrade paths for consumers
- Version compatibility matrix
- Deprecation notices for removed functionality

### 3.3 Handoff to Phase 1 Team

**Deliverables:**
- Complete foundation crate compilation
- 100% documented public APIs
- Full test coverage of core traits
- CI/CD pipeline validating all changes
- Performance baseline measurements

**Knowledge Transfer:**
- Architectural decision rationale document
- Common patterns and idioms guide
- Testing approach and coverage requirements
- Performance characteristics and bottlenecks

---

## 4. Performance Targets and Constraints

### 4.1 Compilation Performance

**Targets:**
- Clean build: < 60 seconds
- Incremental build: < 10 seconds  
- Documentation generation: < 30 seconds
- Test suite execution: < 120 seconds

**Constraints:**
- Zero compilation warnings
- All clippy lints pass at deny level
- Documentation coverage > 95%
- Test coverage > 90%

### 4.2 Runtime Performance

**Memory Usage:**
- Basic trait objects: < 1KB each
- Error types: < 256 bytes per error
- Component metadata: < 2KB per component
- State serialization: < 10KB per component

**Execution Performance:**
- Trait method dispatch: < 1μs overhead
- Error creation and propagation: < 100μs
- State serialization/deserialization: < 10ms
- Component validation: < 1ms

### 4.3 Measurement and Monitoring

**Benchmarking:**
```rust
// Benchmark trait execution overhead
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn benchmark_component_execution(c: &mut Criterion) {
    let component = create_test_component();
    let input = create_test_input();
    
    c.bench_function("component_execute", |b| {
        b.iter(|| {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async {
                component.execute(black_box(input.clone())).await
            })
        })
    });
}

criterion_group!(benches, benchmark_component_execution);
criterion_main!(benches);
```

---

## 5. Security Considerations

### 5.1 Threat Model for Phase 0

**Threats:**
- Malicious trait implementations
- Resource exhaustion through component creation
- Information disclosure through error messages
- Code injection through configuration

**Mitigations:**
- Trait bounds require `Send + Sync` for safe concurrency
- Resource limits on component operations
- Sanitized error messages without sensitive data
- Configuration validation and sanitization

### 5.2 Security Requirements

**Component Security:**
- All components must declare security level
- Dangerous operations require explicit permission
- Sandbox-safe trait implementations by default
- Audit logging for security-sensitive operations

**Error Security:**
- No sensitive data in error messages
- Error categorization for security analysis
- Rate limiting on error generation
- Secure error propagation chains

---

## Success Metrics

### Phase 0 Completion Criteria

**Technical Metrics:**
- [ ] 100% compilation success rate
- [ ] 0 compiler warnings across all crates
- [ ] >95% documentation coverage  
- [ ] >90% test coverage
- [ ] <60s clean build time
- [ ] All CI/CD checks passing

**Quality Metrics:**
- [ ] All trait methods fully documented with examples
- [ ] Comprehensive error handling for all failure modes
- [ ] Property-based tests for core functionality
- [ ] Performance benchmarks established
- [ ] Security review completed

**Readiness Metrics:**
- [ ] Phase 1 team can begin immediately after handoff
- [ ] All architectural decisions documented
- [ ] Clear integration points defined
- [ ] Migration strategy documented
- [ ] Performance baselines established

This foundation will enable rapid development of Phase 1 while maintaining high quality and consistency standards throughout the project.