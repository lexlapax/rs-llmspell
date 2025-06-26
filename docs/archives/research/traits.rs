// ABOUTME: Core trait definitions for rs-llmspell architecture
// ABOUTME: Demonstrates the trait-based design patterns for LLM orchestration

//! # Core Traits for Rs-LLMSpell
//! 
//! This file contains the fundamental trait definitions that form the backbone
//! of the rs-llmspell architecture. These traits demonstrate:
//! 
//! - Provider abstraction patterns
//! - Async-first design
//! - Type-safe error handling
//! - Composable agent patterns
//! - Tool system design
//! - Workflow orchestration

use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
use std::time::Duration;

use async_trait::async_trait;
use futures_util::Stream;
use serde::{Deserialize, Serialize};
use thiserror::Error;

// ================================================================================================
// CORE TYPES
// ================================================================================================

/// Unique identifier type for various entities
pub type Id = String;

/// Token usage statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenUsage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
}

/// Message roles in LLM conversations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Role {
    System,
    User,
    Assistant,
    Tool,
}

/// Content types for messages
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Content {
    Text(String),
    Multimodal(Vec<ContentPart>),
}

/// Parts of multimodal content
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ContentPart {
    Text(String),
    Image { 
        url: Option<String>, 
        data: Option<Vec<u8>>, 
        mime_type: String,
    },
}

/// Message in LLM conversation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub role: Role,
    pub content: Content,
    pub metadata: Option<MessageMetadata>,
}

/// Additional message metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageMetadata {
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub id: Option<Id>,
    pub tags: Vec<String>,
}

/// Completion request to LLM provider
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletionRequest {
    pub messages: Vec<Message>,
    pub model: String,
    pub temperature: Option<f32>,
    pub max_tokens: Option<u32>,
    pub stop_sequences: Option<Vec<String>>,
    pub tools: Option<Vec<ToolDefinition>>,
    pub stream: bool,
}

/// Response from LLM provider
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletionResponse {
    pub message: Message,
    pub usage: TokenUsage,
    pub model: String,
    pub finish_reason: FinishReason,
    pub metadata: ResponseMetadata,
}

/// Streaming chunk from LLM provider
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletionChunk {
    pub delta: ContentDelta,
    pub usage: Option<TokenUsage>,
    pub finish_reason: Option<FinishReason>,
}

/// Delta content for streaming
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ContentDelta {
    Text(String),
    ToolCall { name: String, input: serde_json::Value },
    Done,
}

/// Reason for completion finishing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FinishReason {
    Stop,
    Length,
    ToolCall,
    ContentFilter,
    Error,
}

/// Response metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseMetadata {
    pub request_id: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub processing_time_ms: u64,
    pub provider: String,
}

/// Tool definition for LLM
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolDefinition {
    pub name: String,
    pub description: String,
    pub parameters: serde_json::Value, // JSON Schema
}

/// Embedding request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbeddingRequest {
    pub input: Vec<String>,
    pub model: String,
}

/// Embedding response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbeddingResponse {
    pub embeddings: Vec<Vec<f32>>,
    pub usage: TokenUsage,
    pub model: String,
}

/// Model information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelInfo {
    pub id: String,
    pub name: String,
    pub description: String,
    pub max_tokens: Option<u32>,
    pub supports_streaming: bool,
    pub supports_tools: bool,
    pub supports_vision: bool,
}

// ================================================================================================
// ERROR TYPES
// ================================================================================================

/// Core error type for rs-llmspell
#[derive(Debug, Error)]
pub enum LlmError {
    #[error("Provider error: {0}")]
    Provider(#[from] Box<dyn std::error::Error + Send + Sync>),
    
    #[error("Configuration error: {message}")]
    Configuration { message: String },
    
    #[error("Validation error in {field}: {message}")]
    Validation { field: String, message: String },
    
    #[error("Network error: {0}")]
    Network(#[from] reqwest::Error),
    
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    
    #[error("Timeout after {duration:?}")]
    Timeout { duration: Duration },
    
    #[error("Rate limit exceeded: retry after {retry_after:?}")]
    RateLimit { retry_after: Option<Duration> },
    
    #[error("Authentication failed: {message}")]
    Authentication { message: String },
    
    #[error("Resource not found: {resource_type} {id}")]
    NotFound { resource_type: String, id: String },
    
    #[error("Operation cancelled")]
    Cancelled,
}

/// Agent-specific errors
#[derive(Debug, Error)]
pub enum AgentError {
    #[error("LLM error: {0}")]
    Llm(#[from] LlmError),
    
    #[error("Tool error: {tool_name}: {source}")]
    Tool {
        tool_name: String,
        #[source]
        source: Box<dyn std::error::Error + Send + Sync>,
    },
    
    #[error("Context error: {message}")]
    Context { message: String },
    
    #[error("Configuration error: {message}")]
    Configuration { message: String },
}

/// Tool-specific errors
#[derive(Debug, Error)]
pub enum ToolError {
    #[error("Execution failed: {message}")]
    Execution { message: String },
    
    #[error("Invalid input: {field}: {message}")]
    InvalidInput { field: String, message: String },
    
    #[error("Missing dependency: {dependency}")]
    MissingDependency { dependency: String },
    
    #[error("Permission denied: {operation}")]
    PermissionDenied { operation: String },
    
    #[error("Resource unavailable: {resource}")]
    ResourceUnavailable { resource: String },
}

/// Workflow-specific errors
#[derive(Debug, Error)]
pub enum WorkflowError {
    #[error("Step {step_name} failed: {source}")]
    StepFailed {
        step_name: String,
        #[source]
        source: Box<dyn std::error::Error + Send + Sync>,
    },
    
    #[error("Agent error: {0}")]
    Agent(#[from] AgentError),
    
    #[error("Dependency cycle detected in workflow")]
    CyclicDependency,
    
    #[error("Context error: {message}")]
    Context { message: String },
    
    #[error("Validation error: {message}")]
    Validation { message: String },
}

// ================================================================================================
// PROVIDER TRAIT
// ================================================================================================

/// Core trait for LLM providers
/// 
/// This trait abstracts over different LLM providers (OpenAI, Anthropic, etc.)
/// and allows switching between them without changing application code.
#[async_trait]
pub trait Provider: Send + Sync + Clone {
    /// Provider-specific error type
    type Error: std::error::Error + Send + Sync + 'static;
    
    /// Provider-specific configuration type
    type Config: Clone + Send + Sync;
    
    /// Create a new provider instance with configuration
    async fn new(config: Self::Config) -> Result<Self, Self::Error>;
    
    /// Complete a text generation request
    async fn complete(&self, request: CompletionRequest) -> Result<CompletionResponse, Self::Error>;
    
    /// Stream a text generation request
    fn complete_stream(&self, request: CompletionRequest) 
        -> Pin<Box<dyn Stream<Item = Result<CompletionChunk, Self::Error>> + Send>>;
    
    /// Generate embeddings for text
    async fn embed(&self, request: EmbeddingRequest) -> Result<EmbeddingResponse, Self::Error>;
    
    /// Get available models and their capabilities
    async fn models(&self) -> Result<Vec<ModelInfo>, Self::Error>;
    
    /// Validate configuration without making API calls
    fn validate_config(config: &Self::Config) -> Result<(), Self::Error>;
    
    /// Get provider name for debugging/logging
    fn name(&self) -> &'static str;
    
    /// Check if provider is healthy (optional health check)
    async fn health_check(&self) -> Result<(), Self::Error> {
        // Default implementation does nothing
        Ok(())
    }
}

// ================================================================================================
// AGENT TRAIT
// ================================================================================================

/// Configuration for agent behavior
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentConfig {
    pub model: String,
    pub temperature: Option<f32>,
    pub max_tokens: Option<u32>,
    pub timeout: Duration,
    pub max_retries: u32,
    pub enable_streaming: bool,
}

impl Default for AgentConfig {
    fn default() -> Self {
        Self {
            model: "gpt-3.5-turbo".to_string(),
            temperature: Some(0.7),
            max_tokens: Some(1000),
            timeout: Duration::from_secs(30),
            max_retries: 3,
            enable_streaming: false,
        }
    }
}

/// Response from agent execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentResponse {
    pub content: Content,
    pub usage: TokenUsage,
    pub tool_calls: Vec<ToolCall>,
    pub metadata: AgentMetadata,
}

/// Tool call made by agent
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCall {
    pub id: String,
    pub name: String,
    pub input: serde_json::Value,
    pub output: Option<serde_json::Value>,
    pub error: Option<String>,
}

/// Agent execution metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentMetadata {
    pub agent_id: String,
    pub execution_time_ms: u64,
    pub retry_count: u32,
    pub provider_used: String,
}

/// Streaming chunk from agent
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AgentChunk {
    Content(String),
    ToolCall(ToolCall),
    Done(AgentMetadata),
    Error(AgentError),
}

/// Context for conversation agents
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversationContext {
    pub messages: Vec<Message>,
    pub metadata: HashMap<String, serde_json::Value>,
}

/// High-level agent trait for LLM interactions
/// 
/// Agents encapsulate the logic for interacting with LLMs, including
/// system prompts, tool usage, and conversation management.
#[async_trait]
pub trait Agent: Send + Sync + Clone {
    /// Agent-specific error type
    type Error: std::error::Error + Send + Sync + 'static;
    
    /// Provider type this agent uses
    type Provider: Provider;
    
    /// Tool type this agent can use
    type Tool: Tool;
    
    /// Create a new agent with a provider
    fn new(provider: Self::Provider) -> Self;
    
    /// Configure the agent's system prompt
    fn with_system_prompt(self, prompt: impl Into<String>) -> Self;
    
    /// Add tools to the agent
    fn with_tools(self, tools: Vec<Self::Tool>) -> Self;
    
    /// Configure agent behavior
    fn with_config(self, config: AgentConfig) -> Self;
    
    /// Execute a single request
    async fn run(&self, input: &str) -> Result<AgentResponse, Self::Error>;
    
    /// Execute with conversation context
    async fn run_with_context(&self, input: &str, context: &ConversationContext) 
        -> Result<AgentResponse, Self::Error>;
    
    /// Stream responses in real-time
    fn run_stream(&self, input: &str) 
        -> Pin<Box<dyn Stream<Item = Result<AgentChunk, Self::Error>> + Send>>;
    
    /// Get available tools
    fn tools(&self) -> &[Self::Tool];
    
    /// Validate agent configuration
    fn validate(&self) -> Result<(), Self::Error>;
    
    /// Get agent metadata
    fn metadata(&self) -> AgentMetadata;
}

// ================================================================================================
// TOOL TRAIT
// ================================================================================================

/// Tool execution context
#[derive(Debug, Clone)]
pub struct ToolContext {
    pub agent_id: String,
    pub execution_id: String,
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Tool execution result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolResult<T> {
    pub output: T,
    pub metadata: ToolMetadata,
}

/// Tool execution metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolMetadata {
    pub execution_time_ms: u64,
    pub success: bool,
    pub error_message: Option<String>,
}

/// Tool trait for extending agent capabilities
/// 
/// Tools provide specific functionality that agents can invoke,
/// such as web search, file operations, calculations, etc.
#[async_trait]
pub trait Tool: Send + Sync + Clone {
    /// Tool-specific error type
    type Error: std::error::Error + Send + Sync + 'static;
    
    /// Strongly-typed input for this tool
    type Input: serde::de::DeserializeOwned + Send + Sync;
    
    /// Strongly-typed output from this tool
    type Output: serde::Serialize + Send + Sync;
    
    /// Tool name (must be unique within a toolset)
    fn name(&self) -> &str;
    
    /// Human-readable description
    fn description(&self) -> &str;
    
    /// JSON Schema for parameters
    fn parameters_schema(&self) -> serde_json::Value;
    
    /// Execute the tool with typed input
    async fn execute(&self, input: Self::Input, context: ToolContext) 
        -> Result<ToolResult<Self::Output>, Self::Error>;
    
    /// Validate input without executing
    fn validate_input(&self, input: &serde_json::Value) -> Result<(), Self::Error>;
    
    /// Check if tool is available (e.g., API keys present)
    async fn health_check(&self) -> Result<(), Self::Error>;
    
    /// Get tool categories/tags
    fn categories(&self) -> Vec<String> {
        vec![]
    }
    
    /// Whether tool requires special permissions
    fn requires_permission(&self) -> bool {
        false
    }
}

// ================================================================================================
// WORKFLOW TRAITS
// ================================================================================================

/// Context passed between workflow steps
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowContext {
    pub variables: HashMap<String, serde_json::Value>,
    pub step_results: HashMap<String, serde_json::Value>,
    pub metadata: HashMap<String, serde_json::Value>,
}

impl WorkflowContext {
    pub fn new() -> Self {
        Self {
            variables: HashMap::new(),
            step_results: HashMap::new(),
            metadata: HashMap::new(),
        }
    }
    
    pub fn set_variable(&mut self, key: impl Into<String>, value: impl Serialize) {
        if let Ok(json_value) = serde_json::to_value(value) {
            self.variables.insert(key.into(), json_value);
        }
    }
    
    pub fn get_variable<T>(&self, key: &str) -> Option<T> 
    where 
        T: serde::de::DeserializeOwned
    {
        self.variables.get(key)
            .and_then(|v| serde_json::from_value(v.clone()).ok())
    }
}

/// Result from workflow execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowResult {
    pub success: bool,
    pub step_results: HashMap<String, serde_json::Value>,
    pub final_output: Option<serde_json::Value>,
    pub execution_time_ms: u64,
    pub metadata: WorkflowMetadata,
}

/// Workflow execution metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowMetadata {
    pub workflow_id: String,
    pub execution_id: String,
    pub started_at: chrono::DateTime<chrono::Utc>,
    pub completed_at: Option<chrono::DateTime<chrono::Utc>>,
    pub step_count: usize,
    pub errors: Vec<String>,
}

/// Progress information during workflow execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowProgress {
    pub current_step: String,
    pub completed_steps: usize,
    pub total_steps: usize,
    pub percentage: f32,
    pub estimated_remaining_ms: Option<u64>,
}

/// Individual step in a workflow
#[async_trait]
pub trait WorkflowStep: Send + Sync {
    /// Step-specific error type
    type Error: std::error::Error + Send + Sync + 'static;
    
    /// Execute this step with the given context
    async fn execute(&self, context: &mut WorkflowContext) 
        -> Result<serde_json::Value, Self::Error>;
    
    /// Get step name for identification
    fn name(&self) -> &str;
    
    /// Get step description
    fn description(&self) -> &str;
    
    /// Validate step configuration
    fn validate(&self) -> Result<(), Self::Error>;
    
    /// Get step dependencies (for DAG workflows)
    fn dependencies(&self) -> Vec<String> {
        vec![]
    }
    
    /// Whether this step can be retried on failure
    fn is_retryable(&self) -> bool {
        true
    }
    
    /// Maximum retry attempts
    fn max_retries(&self) -> u32 {
        3
    }
}

/// Workflow orchestration trait
/// 
/// Workflows coordinate multiple agents and tools to accomplish
/// complex tasks through structured execution patterns.
#[async_trait]
pub trait Workflow: Send + Sync {
    /// Workflow-specific error type
    type Error: std::error::Error + Send + Sync + 'static;
    
    /// Step type used in this workflow
    type Step: WorkflowStep;
    
    /// Add a step to the workflow
    fn add_step(self, step: Self::Step) -> Self;
    
    /// Execute the entire workflow
    async fn run(&self, context: WorkflowContext) -> Result<WorkflowResult, Self::Error>;
    
    /// Execute with progress reporting
    fn run_with_progress(&self, context: WorkflowContext) 
        -> Pin<Box<dyn Stream<Item = Result<WorkflowProgress, Self::Error>> + Send>>;
    
    /// Validate workflow configuration
    fn validate(&self) -> Result<(), Self::Error>;
    
    /// Get workflow metadata
    fn metadata(&self) -> WorkflowMetadata;
    
    /// Cancel workflow execution
    async fn cancel(&self) -> Result<(), Self::Error> {
        // Default implementation does nothing
        Ok(())
    }
    
    /// Pause workflow execution (if supported)
    async fn pause(&self) -> Result<(), Self::Error> {
        Err(self.unsupported_operation("pause"))
    }
    
    /// Resume paused workflow (if supported)
    async fn resume(&self) -> Result<(), Self::Error> {
        Err(self.unsupported_operation("resume"))
    }
    
    /// Helper for unsupported operations
    fn unsupported_operation(&self, operation: &str) -> Self::Error {
        // This is a placeholder - actual implementation would depend on the concrete error type
        panic!("Operation '{}' not supported by this workflow type", operation)
    }
}

// ================================================================================================
// CONFIGURATION TRAITS
// ================================================================================================

/// Trait for types that can be configured
pub trait Configurable {
    type Config: Clone + Send + Sync;
    type Error: std::error::Error + Send + Sync + 'static;
    
    /// Apply configuration
    fn configure(&mut self, config: Self::Config) -> Result<(), Self::Error>;
    
    /// Get current configuration
    fn config(&self) -> &Self::Config;
    
    /// Validate configuration
    fn validate_config(config: &Self::Config) -> Result<(), Self::Error>;
}

/// Trait for types that can be observed/monitored
pub trait Observable {
    type Event: Clone + Send + Sync;
    
    /// Subscribe to events from this object
    fn subscribe(&self) -> Pin<Box<dyn Stream<Item = Self::Event> + Send>>;
    
    /// Get current metrics
    fn metrics(&self) -> HashMap<String, f64> {
        HashMap::new()
    }
}

// ================================================================================================
// UTILITY TRAITS
// ================================================================================================

/// Trait for types that can be retried with backoff
#[async_trait]
pub trait Retryable {
    type Output;
    type Error: std::error::Error + Send + Sync + 'static;
    
    /// Execute operation with retry logic
    async fn execute_with_retry(&self, max_attempts: u32, base_delay: Duration) 
        -> Result<Self::Output, Self::Error>;
    
    /// Check if error is retryable
    fn is_retryable_error(&self, error: &Self::Error) -> bool;
}

/// Trait for types that support timeouts
#[async_trait]
pub trait Timeout {
    type Output;
    type Error: std::error::Error + Send + Sync + 'static;
    
    /// Execute with timeout
    async fn execute_with_timeout(&self, timeout: Duration) 
        -> Result<Self::Output, Self::Error>;
}

/// Trait for types that can be validated
pub trait Validatable {
    type Error: std::error::Error + Send + Sync + 'static;
    
    /// Validate the object
    fn validate(&self) -> Result<(), Self::Error>;
}

// ================================================================================================
// EXAMPLE IMPLEMENTATIONS
// ================================================================================================

/// Example agent step for workflows
pub struct AgentStep<A: Agent> {
    pub name: String,
    pub agent: A,
    pub input_template: String,
}

#[async_trait]
impl<A: Agent> WorkflowStep for AgentStep<A> {
    type Error = A::Error;
    
    async fn execute(&self, context: &mut WorkflowContext) 
        -> Result<serde_json::Value, Self::Error> {
        // Template the input with context variables
        let input = self.render_template(&self.input_template, context);
        
        // Execute agent
        let response = self.agent.run(&input).await?;
        
        // Store result in context
        let result_value = serde_json::to_value(&response)
            .map_err(|e| panic!("Serialization error: {}", e))?; // In real impl, handle properly
            
        context.step_results.insert(self.name.clone(), result_value.clone());
        
        Ok(result_value)
    }
    
    fn name(&self) -> &str {
        &self.name
    }
    
    fn description(&self) -> &str {
        "Execute agent with templated input"
    }
    
    fn validate(&self) -> Result<(), Self::Error> {
        self.agent.validate()
    }
}

impl<A: Agent> AgentStep<A> {
    fn render_template(&self, template: &str, context: &WorkflowContext) -> String {
        // Simple template rendering - in real implementation would be more sophisticated
        let mut result = template.to_string();
        
        for (key, value) in &context.variables {
            if let Ok(string_value) = serde_json::from_value::<String>(value.clone()) {
                result = result.replace(&format!("{{{{{}}}}}", key), &string_value);
            }
        }
        
        result
    }
}

/// Example tool step for workflows
pub struct ToolStep<T: Tool> {
    pub name: String,
    pub tool: T,
    pub input: T::Input,
}

#[async_trait]
impl<T: Tool> WorkflowStep for ToolStep<T> {
    type Error = T::Error;
    
    async fn execute(&self, context: &mut WorkflowContext) 
        -> Result<serde_json::Value, Self::Error> {
        let tool_context = ToolContext {
            agent_id: "workflow".to_string(),
            execution_id: "step".to_string(),
            metadata: HashMap::new(),
        };
        
        let result = self.tool.execute(self.input.clone(), tool_context).await?;
        
        let result_value = serde_json::to_value(&result.output)
            .map_err(|e| panic!("Serialization error: {}", e))?; // In real impl, handle properly
            
        context.step_results.insert(self.name.clone(), result_value.clone());
        
        Ok(result_value)
    }
    
    fn name(&self) -> &str {
        &self.name
    }
    
    fn description(&self) -> &str {
        "Execute tool with provided input"
    }
    
    fn validate(&self) -> Result<(), Self::Error> {
        // Validate tool input
        let input_json = serde_json::to_value(&self.input)
            .map_err(|e| panic!("Serialization error: {}", e))?; // In real impl, handle properly
        self.tool.validate_input(&input_json)
    }
}