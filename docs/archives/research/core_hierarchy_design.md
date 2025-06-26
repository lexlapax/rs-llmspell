# Rs-LLMSpell Core Hierarchy Design

## Overview

Comprehensive design for the BaseAgent/Agent/Tool/Workflow hierarchy based on go-llms patterns, Google ADK architecture, and state management research.

## Table of Contents

1. [Design Principles](#design-principles)
2. [Core Hierarchy Overview](#core-hierarchy-overview)
3. [BaseAgent: Foundation Layer](#baseagent-foundation-layer)
4. [Agent: LLM Wrapper Layer](#agent-llm-wrapper-layer)
5. [Tool: Capability Extension Layer](#tool-capability-extension-layer)
6. [Workflow: Orchestration Layer](#workflow-orchestration-layer)
7. [ToolWrappedAgent: Composition Layer](#toolwrappedagent-composition-layer)
8. [State Integration](#state-integration)
9. [Hooks and Events Integration](#hooks-and-events-integration)
10. [Built-in Components Strategy](#built-in-components-strategy)
11. [Examples and Usage Patterns](#examples-and-usage-patterns)

## Design Principles

### 1. Layered Composition
Each layer builds upon the previous one, adding specific capabilities while maintaining the core interface contract.

### 2. State-First Architecture
All components are state-aware and support agent handoff through shared state management.

### 3. Hook-Enabled Observability
Every significant operation provides hook points for logging, metrics, debugging, and custom behavior injection.

### 4. Tool-Centric Capabilities
Tools are the primary mechanism for extending agent capabilities, and agents can themselves be wrapped as tools.

### 5. Type-Safe Bridge Compatibility
All components expose clean interfaces for the bridge layer to script engines.

## Core Hierarchy Overview

```
┌─────────────────────────────────────────────────────────────┐
│                    Script Layer (Lua/JS)                    │
├─────────────────────────────────────────────────────────────┤
│                    Bridge Layer                             │
├─────────────────────────────────────────────────────────────┤
│  ToolWrappedAgent  │        Workflow Types                  │
│  ┌─────────────┐   │  ┌──────────┬──────────┬─────────────┐ │
│  │    Agent    │◄──┤  │Sequential│ Parallel │ Conditional │ │
│  │             │   │  │          │          │    Loop     │ │
│  └─────────────┘   │  └──────────┴──────────┴─────────────┘ │
├─────────────────────────────────────────────────────────────┤
│                    Agent Layer                              │
│  ┌─────────────────────────────────────────────────────────┐ │
│  │               BaseAgent + Tools                         │ │
│  │  ┌─────────────┐  ┌─────┐ ┌─────┐ ┌─────┐ ┌─────────┐  │ │
│  │  │ BaseAgent   │──│Tool1│─│Tool2│─│...  │─│ToolN    │  │ │
│  │  │             │  └─────┘ └─────┘ └─────┘ └─────────┘  │ │
│  │  └─────────────┘                                       │ │
│  └─────────────────────────────────────────────────────────┘ │
├─────────────────────────────────────────────────────────────┤
│                 Foundation Layer                            │
│  ┌─────────────┐ ┌─────────────┐ ┌─────────────────────────┐ │
│  │ StateManager│ │ HookSystem  │ │    EventSystem          │ │
│  └─────────────┘ └─────────────┘ └─────────────────────────┘ │
└─────────────────────────────────────────────────────────────┘
```

## BaseAgent: Foundation Layer

The `BaseAgent` trait provides the fundamental capabilities that all agent types share: state management, tool handling, and hook integration.

### Core Interface

```rust
use std::collections::HashMap;
use std::sync::Arc;
use async_trait::async_trait;
use chrono::{DateTime, Utc};

/// Fundamental agent capabilities - tool handling, state management, hooks
#[async_trait]
pub trait BaseAgent: Send + Sync {
    type Error: std::error::Error + Send + Sync + 'static;
    
    // === Core Identity ===
    fn id(&self) -> &str;
    fn name(&self) -> &str;
    fn description(&self) -> &str;
    
    // === State Management ===
    async fn get_state(&self) -> Result<Arc<AgentState>, Self::Error>;
    async fn set_state(&mut self, state: AgentState) -> Result<(), Self::Error>;
    async fn update_state<F>(&mut self, updater: F) -> Result<(), Self::Error>
    where
        F: FnOnce(&mut AgentState) -> Result<(), Self::Error> + Send;
    
    // === Tool Management ===
    fn tools(&self) -> &[Arc<dyn Tool>];
    fn has_tool(&self, name: &str) -> bool;
    fn get_tool(&self, name: &str) -> Option<Arc<dyn Tool>>;
    async fn add_tool(&mut self, tool: Arc<dyn Tool>) -> Result<(), Self::Error>;
    async fn remove_tool(&mut self, name: &str) -> Result<bool, Self::Error>;
    
    // === Tool Execution ===
    async fn execute_tool(&self, tool_name: &str, input: ToolInput, context: ToolContext) 
        -> Result<ToolResult, Self::Error>;
    async fn execute_tool_chain(&self, chain: &[ToolCall]) 
        -> Result<Vec<ToolResult>, Self::Error>;
    
    // === Agent Handoff ===
    async fn can_accept_handoff(&self, state: &AgentState) -> bool;
    async fn prepare_handoff(&self, target_id: &str) -> Result<HandoffPayload, Self::Error>;
    async fn receive_handoff(&mut self, payload: HandoffPayload) -> Result<(), Self::Error>;
    
    // === Hook Integration ===
    async fn register_hook(&mut self, hook: Arc<dyn Hook>) -> Result<(), Self::Error>;
    async fn unregister_hook(&mut self, hook_id: &str) -> Result<bool, Self::Error>;
    fn hooks(&self) -> &[Arc<dyn Hook>];
    
    // === Execution Lifecycle ===
    async fn initialize(&mut self) -> Result<(), Self::Error>;
    async fn validate(&self) -> Result<(), Self::Error>;
    async fn cleanup(&mut self) -> Result<(), Self::Error>;
    
    // === Events ===
    fn event_emitter(&self) -> Arc<dyn EventEmitter>;
    
    // === Metadata ===
    fn metadata(&self) -> BaseAgentMetadata;
    fn capabilities(&self) -> AgentCapabilities;
}

/// Core state structure for all agents
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AgentState {
    pub state_id: String,
    pub agent_id: String,
    pub variables: HashMap<String, StateValue>,
    pub context: AgentContext,
    pub metadata: StateMetadata,
    pub version: u64,
}

/// Agent execution context
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AgentContext {
    pub conversation_id: Option<String>,
    pub session_id: Option<String>,
    pub parent_context: Option<String>,
    pub execution_trace: Vec<ExecutionStep>,
    pub permissions: HashSet<String>,
}

/// Single execution step in agent trace
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ExecutionStep {
    pub step_id: String,
    pub step_type: ExecutionStepType,
    pub timestamp: DateTime<Utc>,
    pub duration_ms: u64,
    pub success: bool,
    pub error: Option<String>,
    pub metadata: HashMap<String, StateValue>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum ExecutionStepType {
    ToolCall { tool_name: String },
    LlmCall { provider: String, model: String },
    StateUpdate { keys: Vec<String> },
    Handoff { target_agent: String },
    HookExecution { hook_type: String },
}

/// Agent handoff payload
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct HandoffPayload {
    pub from_agent: String,
    pub to_agent: String,
    pub state: AgentState,
    pub context: HandoffContext,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct HandoffContext {
    pub reason: String,
    pub expected_capabilities: Vec<String>,
    pub return_expected: bool,
    pub priority: HandoffPriority,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum HandoffPriority {
    Low,
    Normal,
    High,
    Critical,
}

/// Agent capabilities declaration
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AgentCapabilities {
    pub supported_input_types: Vec<String>,
    pub supported_output_types: Vec<String>,
    pub required_tools: Vec<String>,
    pub optional_tools: Vec<String>,
    pub max_conversation_length: Option<usize>,
    pub supports_streaming: bool,
    pub supports_handoff: bool,
    pub supports_parallel_execution: bool,
}

/// Base agent metadata
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct BaseAgentMetadata {
    pub agent_id: String,
    pub agent_type: String,
    pub version: String,
    pub created_at: DateTime<Utc>,
    pub last_active: DateTime<Utc>,
    pub execution_count: u64,
    pub success_rate: f64,
    pub average_execution_time_ms: f64,
}
```

### Standard BaseAgent Implementation

```rust
/// Standard implementation of BaseAgent
pub struct StandardBaseAgent {
    pub id: String,
    pub name: String,
    pub description: String,
    pub state: Arc<RwLock<AgentState>>,
    pub tools: Arc<RwLock<HashMap<String, Arc<dyn Tool>>>>,
    pub hooks: Arc<RwLock<Vec<Arc<dyn Hook>>>>,
    pub event_emitter: Arc<dyn EventEmitter>,
    pub state_manager: Arc<dyn StateManager>,
    pub metadata: Arc<RwLock<BaseAgentMetadata>>,
}

#[async_trait]
impl BaseAgent for StandardBaseAgent {
    type Error = AgentError;
    
    fn id(&self) -> &str { &self.id }
    fn name(&self) -> &str { &self.name }
    fn description(&self) -> &str { &self.description }
    
    async fn get_state(&self) -> Result<Arc<AgentState>, Self::Error> {
        let state = self.state.read().await;
        Ok(Arc::new(state.clone()))
    }
    
    async fn set_state(&mut self, new_state: AgentState) -> Result<(), Self::Error> {
        // Execute pre-state-update hooks
        self.execute_hooks(HookPoint::PreStateUpdate, &new_state).await?;
        
        {
            let mut state = self.state.write().await;
            *state = new_state;
        }
        
        // Persist state
        let state = self.state.read().await;
        self.state_manager.save_state(&state).await
            .map_err(|e| AgentError::StateError { message: e.to_string() })?;
        
        // Execute post-state-update hooks
        self.execute_hooks(HookPoint::PostStateUpdate, &state).await?;
        
        // Emit event
        self.event_emitter.emit(Event::StateUpdated {
            agent_id: self.id.clone(),
            state_id: state.state_id.clone(),
            timestamp: Utc::now(),
        }).await;
        
        Ok(())
    }
    
    async fn execute_tool(&self, tool_name: &str, input: ToolInput, context: ToolContext) 
        -> Result<ToolResult, Self::Error> {
        // Get tool
        let tools = self.tools.read().await;
        let tool = tools.get(tool_name)
            .ok_or_else(|| AgentError::ToolNotFound { name: tool_name.to_string() })?;
        
        // Execute pre-tool hooks
        self.execute_hooks(HookPoint::PreTool, &(tool_name, &input)).await?;
        
        // Execute tool
        let start = std::time::Instant::now();
        let result = tool.execute(input, context).await
            .map_err(|e| AgentError::ToolError { 
                tool_name: tool_name.to_string(), 
                source: Box::new(e) 
            })?;
        let duration = start.elapsed();
        
        // Update execution trace
        self.add_execution_step(ExecutionStep {
            step_id: format!("tool_{}_{}", tool_name, Utc::now().timestamp_nanos()),
            step_type: ExecutionStepType::ToolCall { tool_name: tool_name.to_string() },
            timestamp: Utc::now(),
            duration_ms: duration.as_millis() as u64,
            success: result.metadata.success,
            error: result.metadata.error_message.clone(),
            metadata: HashMap::new(),
        }).await?;
        
        // Execute post-tool hooks
        self.execute_hooks(HookPoint::PostTool, &(tool_name, &result)).await?;
        
        // Emit event
        self.event_emitter.emit(Event::ToolExecuted {
            agent_id: self.id.clone(),
            tool_name: tool_name.to_string(),
            duration_ms: duration.as_millis() as u64,
            success: result.metadata.success,
            timestamp: Utc::now(),
        }).await;
        
        Ok(result)
    }
    
    // ... additional implementation methods
}
```

## Agent: LLM Wrapper Layer

The `Agent` trait extends `BaseAgent` with LLM-specific capabilities: prompt management, model interaction, and conversation handling.

### Core Interface

```rust
/// LLM-specific agent capabilities
#[async_trait]
pub trait Agent: BaseAgent {
    type Provider: Provider;
    
    // === LLM Configuration ===
    fn provider(&self) -> &Self::Provider;
    fn model(&self) -> &str;
    fn system_prompt(&self) -> Option<&str>;
    fn temperature(&self) -> Option<f32>;
    fn max_tokens(&self) -> Option<u32>;
    
    // === LLM Interaction ===
    async fn run(&self, input: &str) -> Result<AgentResponse, Self::Error>;
    async fn run_with_context(&self, input: &str, context: &ConversationContext) 
        -> Result<AgentResponse, Self::Error>;
    async fn run_with_state(&self, input: &str, state: &mut AgentState) 
        -> Result<AgentResponse, Self::Error>;
    
    // === Streaming ===
    fn run_stream(&self, input: &str) 
        -> Pin<Box<dyn Stream<Item = Result<AgentChunk, Self::Error>> + Send>>;
    
    // === Conversation Management ===
    async fn continue_conversation(&self, message: &str) -> Result<AgentResponse, Self::Error>;
    async fn reset_conversation(&mut self) -> Result<(), Self::Error>;
    fn conversation_length(&self) -> usize;
    
    // === Configuration ===
    fn with_system_prompt(self, prompt: impl Into<String>) -> Self;
    fn with_temperature(self, temperature: f32) -> Self;
    fn with_max_tokens(self, max_tokens: u32) -> Self;
    fn with_tools(self, tools: Vec<Arc<dyn Tool>>) -> Self;
}

/// Agent response with rich metadata
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AgentResponse {
    pub content: Content,
    pub tool_calls: Vec<ToolCall>,
    pub usage: TokenUsage,
    pub metadata: AgentResponseMetadata,
    pub state_changes: Vec<StateChange>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AgentResponseMetadata {
    pub agent_id: String,
    pub model: String,
    pub provider: String,
    pub execution_time_ms: u64,
    pub retry_count: u32,
    pub finish_reason: FinishReason,
    pub hooks_executed: Vec<String>,
    pub events_emitted: Vec<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct StateChange {
    pub key: String,
    pub old_value: Option<StateValue>,
    pub new_value: StateValue,
    pub change_type: StateChangeType,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum StateChangeType {
    Created,
    Updated,
    Deleted,
    Merged,
}
```

### Standard Agent Implementation

```rust
/// Standard LLM agent implementation
pub struct StandardAgent {
    // BaseAgent composition
    pub base: StandardBaseAgent,
    
    // LLM-specific fields
    pub provider: Arc<dyn Provider>,
    pub model: String,
    pub system_prompt: Option<String>,
    pub temperature: Option<f32>,
    pub max_tokens: Option<u32>,
    pub conversation: Arc<RwLock<Vec<Message>>>,
    pub config: AgentConfig,
}

#[async_trait]
impl BaseAgent for StandardAgent {
    type Error = AgentError;
    
    // Delegate all BaseAgent methods to self.base
    fn id(&self) -> &str { self.base.id() }
    fn name(&self) -> &str { self.base.name() }
    // ... other delegations
}

#[async_trait]
impl Agent for StandardAgent {
    type Provider = dyn Provider;
    
    async fn run(&self, input: &str) -> Result<AgentResponse, Self::Error> {
        // Execute pre-LLM hooks
        self.execute_hooks(HookPoint::PreLlm, input).await?;
        
        // Build completion request
        let mut messages = self.conversation.read().await.clone();
        messages.push(Message {
            role: Role::User,
            content: Content::Text(input.to_string()),
            metadata: Some(MessageMetadata {
                timestamp: Utc::now(),
                id: Some(format!("msg_{}", Utc::now().timestamp_nanos())),
                tags: vec![],
            }),
        });
        
        if let Some(system_prompt) = &self.system_prompt {
            messages.insert(0, Message {
                role: Role::System,
                content: Content::Text(system_prompt.clone()),
                metadata: None,
            });
        }
        
        let request = CompletionRequest {
            messages,
            model: self.model.clone(),
            temperature: self.temperature,
            max_tokens: self.max_tokens,
            stop_sequences: None,
            tools: Some(self.get_tool_definitions()),
            stream: false,
        };
        
        // Execute LLM call
        let start = std::time::Instant::now();
        let response = self.provider.complete(request).await
            .map_err(|e| AgentError::LlmError(LlmError::Provider(Box::new(e))))?;
        let duration = start.elapsed();
        
        // Process tool calls if any
        let mut tool_results = Vec::new();
        for tool_call in &self.extract_tool_calls(&response.message) {
            let result = self.execute_tool(&tool_call.name, tool_call.input.clone(), 
                self.create_tool_context()).await?;
            tool_results.push(result);
        }
        
        // Update conversation
        {
            let mut conversation = self.conversation.write().await;
            conversation.push(response.message.clone());
        }
        
        // Create agent response
        let agent_response = AgentResponse {
            content: response.message.content,
            tool_calls: self.extract_tool_calls(&response.message),
            usage: response.usage,
            metadata: AgentResponseMetadata {
                agent_id: self.id().to_string(),
                model: response.model,
                provider: response.metadata.provider,
                execution_time_ms: duration.as_millis() as u64,
                retry_count: 0,
                finish_reason: response.finish_reason,
                hooks_executed: vec![], // Populated by hook system
                events_emitted: vec![], // Populated by event system
            },
            state_changes: vec![], // Populated if state was modified
        };
        
        // Execute post-LLM hooks
        self.execute_hooks(HookPoint::PostLlm, &agent_response).await?;
        
        // Emit completion event
        self.event_emitter().emit(Event::AgentCompleted {
            agent_id: self.id().to_string(),
            duration_ms: duration.as_millis() as u64,
            success: true,
            tool_calls_count: tool_results.len(),
            timestamp: Utc::now(),
        }).await;
        
        Ok(agent_response)
    }
    
    // ... additional Agent implementation methods
}
```

## Tool: Capability Extension Layer

Tools provide specific functionality that agents can invoke. The design supports both built-in tools and agent-wrapped tools.

### Core Interface

```rust
/// Tool execution capability
#[async_trait]
pub trait Tool: Send + Sync {
    type Error: std::error::Error + Send + Sync + 'static;
    
    // === Core Identity ===
    fn name(&self) -> &str;
    fn description(&self) -> &str;
    fn version(&self) -> &str;
    
    // === Tool Schema ===
    fn parameters_schema(&self) -> serde_json::Value;
    fn return_schema(&self) -> serde_json::Value;
    
    // === Execution ===
    async fn execute(&self, input: ToolInput, context: ToolContext) 
        -> Result<ToolResult, Self::Error>;
    
    // === Validation ===
    async fn validate_input(&self, input: &ToolInput) -> Result<(), Self::Error>;
    async fn health_check(&self) -> Result<(), Self::Error>;
    
    // === Metadata ===
    fn categories(&self) -> Vec<String>;
    fn requires_permission(&self) -> bool;
    fn is_builtin(&self) -> bool;
    fn dependencies(&self) -> Vec<String>;
    
    // === Configuration ===
    fn supports_streaming(&self) -> bool;
    fn timeout(&self) -> Duration;
    fn retry_policy(&self) -> RetryPolicy;
}

/// Universal tool input type
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ToolInput {
    pub parameters: HashMap<String, StateValue>,
    pub metadata: ToolInputMetadata,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ToolInputMetadata {
    pub source: ToolInputSource,
    pub validation_level: ValidationLevel,
    pub timeout_override: Option<Duration>,
    pub retry_override: Option<RetryPolicy>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum ToolInputSource {
    Agent { agent_id: String },
    Workflow { workflow_id: String, step: String },
    User { session_id: String },
    System,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum ValidationLevel {
    None,
    Basic,
    Strict,
    Custom(String),
}

/// Tool execution context
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ToolContext {
    pub execution_id: String,
    pub agent_id: String,
    pub session_id: Option<String>,
    pub conversation_id: Option<String>,
    pub permissions: HashSet<String>,
    pub environment: HashMap<String, String>,
    pub timeout: Duration,
    pub retry_policy: RetryPolicy,
}

/// Tool execution result
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ToolResult {
    pub output: StateValue,
    pub metadata: ToolResultMetadata,
    pub side_effects: Vec<SideEffect>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ToolResultMetadata {
    pub execution_time_ms: u64,
    pub success: bool,
    pub error_message: Option<String>,
    pub retry_count: u32,
    pub cache_hit: bool,
    pub resource_usage: ResourceUsage,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SideEffect {
    pub effect_type: SideEffectType,
    pub description: String,
    pub reversible: bool,
    pub metadata: HashMap<String, StateValue>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum SideEffectType {
    FileCreated { path: String },
    FileModified { path: String },
    FileDeleted { path: String },
    NetworkRequest { url: String, method: String },
    DatabaseChange { table: String, operation: String },
    StateModified { keys: Vec<String> },
    EventEmitted { event_type: String },
    Other { category: String },
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ResourceUsage {
    pub cpu_time_ms: u64,
    pub memory_bytes: u64,
    pub network_bytes_sent: u64,
    pub network_bytes_received: u64,
    pub file_operations: u32,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RetryPolicy {
    pub max_attempts: u32,
    pub base_delay_ms: u64,
    pub max_delay_ms: u64,
    pub backoff_factor: f64,
    pub retryable_errors: Vec<String>,
}
```

## ToolWrappedAgent: Composition Layer

Enables agents to be used as tools by other agents, creating powerful composition patterns.

### Core Interface

```rust
/// Agent wrapped as a tool for composition
pub struct ToolWrappedAgent {
    pub agent: Arc<dyn Agent>,
    pub tool_config: ToolConfig,
    pub wrapper_metadata: WrapperMetadata,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ToolConfig {
    pub name: String,
    pub description: String,
    pub parameters_schema: serde_json::Value,
    pub return_schema: serde_json::Value,
    pub timeout: Duration,
    pub retry_policy: RetryPolicy,
    pub categories: Vec<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct WrapperMetadata {
    pub wrapped_agent_id: String,
    pub wrapper_version: String,
    pub created_at: DateTime<Utc>,
    pub usage_count: u64,
    pub success_rate: f64,
}

#[async_trait]
impl Tool for ToolWrappedAgent {
    type Error = AgentError;
    
    fn name(&self) -> &str { &self.tool_config.name }
    fn description(&self) -> &str { &self.tool_config.description }
    fn version(&self) -> &str { &self.wrapper_metadata.wrapper_version }
    
    async fn execute(&self, input: ToolInput, context: ToolContext) 
        -> Result<ToolResult, Self::Error> {
        // Extract prompt from tool input
        let prompt = input.parameters.get("prompt")
            .and_then(|v| v.as_string())
            .ok_or_else(|| AgentError::InvalidInput { 
                field: "prompt".to_string(), 
                message: "prompt parameter required".to_string() 
            })?;
        
        // Create agent state from context
        let mut agent_state = AgentState {
            state_id: format!("tool_wrapped_{}", context.execution_id),
            agent_id: self.agent.id().to_string(),
            variables: HashMap::new(),
            context: AgentContext {
                conversation_id: context.conversation_id.clone(),
                session_id: context.session_id.clone(),
                parent_context: Some(context.agent_id.clone()),
                execution_trace: vec![],
                permissions: context.permissions.clone(),
            },
            metadata: StateMetadata {
                created_at: Utc::now(),
                updated_at: Utc::now(),
                owner_agent: self.agent.id().to_string(),
                version: 1,
                tags: vec!["tool_wrapped".to_string()],
            },
        };
        
        // Copy relevant context to state
        for (key, value) in &input.parameters {
            if key != "prompt" {
                agent_state.variables.insert(key.clone(), value.clone());
            }
        }
        
        let start = std::time::Instant::now();
        
        // Execute wrapped agent
        let response = self.agent.run_with_state(&prompt, &mut agent_state).await?;
        
        let duration = start.elapsed();
        
        // Convert agent response to tool result
        let tool_result = ToolResult {
            output: StateValue::Object({
                let mut obj = HashMap::new();
                obj.insert("content".to_string(), 
                          StateValue::String(response.content.to_string()));
                obj.insert("tool_calls".to_string(), 
                          StateValue::Array(response.tool_calls.into_iter()
                              .map(|tc| StateValue::String(format!("{:?}", tc)))
                              .collect()));
                obj.insert("usage".to_string(), 
                          StateValue::Object({
                              let mut usage = HashMap::new();
                              usage.insert("total_tokens".to_string(), 
                                         StateValue::Number(response.usage.total_tokens as f64));
                              usage
                          }));
                obj
            }),
            metadata: ToolResultMetadata {
                execution_time_ms: duration.as_millis() as u64,
                success: true,
                error_message: None,
                retry_count: 0,
                cache_hit: false,
                resource_usage: ResourceUsage {
                    cpu_time_ms: duration.as_millis() as u64,
                    memory_bytes: 0, // Would be measured in real implementation
                    network_bytes_sent: 0,
                    network_bytes_received: 0,
                    file_operations: 0,
                },
            },
            side_effects: vec![
                SideEffect {
                    effect_type: SideEffectType::StateModified { 
                        keys: agent_state.variables.keys().cloned().collect() 
                    },
                    description: "Agent execution modified state".to_string(),
                    reversible: false,
                    metadata: HashMap::new(),
                }
            ],
        };
        
        Ok(tool_result)
    }
    
    fn parameters_schema(&self) -> serde_json::Value {
        self.tool_config.parameters_schema.clone()
    }
    
    fn return_schema(&self) -> serde_json::Value {
        self.tool_config.return_schema.clone()
    }
    
    async fn validate_input(&self, input: &ToolInput) -> Result<(), Self::Error> {
        // Validate required prompt parameter
        if !input.parameters.contains_key("prompt") {
            return Err(AgentError::InvalidInput {
                field: "prompt".to_string(),
                message: "prompt parameter is required".to_string(),
            });
        }
        
        // Validate agent can handle the input
        // In real implementation, would use more sophisticated validation
        Ok(())
    }
    
    async fn health_check(&self) -> Result<(), Self::Error> {
        self.agent.validate().await
    }
    
    fn categories(&self) -> Vec<String> {
        self.tool_config.categories.clone()
    }
    
    fn requires_permission(&self) -> bool { true }
    fn is_builtin(&self) -> bool { false }
    fn dependencies(&self) -> Vec<String> { vec![self.agent.id().to_string()] }
    fn supports_streaming(&self) -> bool { false } // Could be enhanced
    fn timeout(&self) -> Duration { self.tool_config.timeout }
    fn retry_policy(&self) -> RetryPolicy { self.tool_config.retry_policy.clone() }
}

impl ToolWrappedAgent {
    pub fn new(agent: Arc<dyn Agent>, config: ToolConfig) -> Self {
        Self {
            agent,
            tool_config: config,
            wrapper_metadata: WrapperMetadata {
                wrapped_agent_id: agent.id().to_string(),
                wrapper_version: "1.0.0".to_string(),
                created_at: Utc::now(),
                usage_count: 0,
                success_rate: 1.0,
            },
        }
    }
    
    pub fn builder(agent: Arc<dyn Agent>) -> ToolWrapperBuilder {
        ToolWrapperBuilder::new(agent)
    }
}

/// Builder for ToolWrappedAgent
pub struct ToolWrapperBuilder {
    agent: Arc<dyn Agent>,
    name: Option<String>,
    description: Option<String>,
    parameters_schema: Option<serde_json::Value>,
    timeout: Duration,
    categories: Vec<String>,
}

impl ToolWrapperBuilder {
    pub fn new(agent: Arc<dyn Agent>) -> Self {
        Self {
            agent,
            name: None,
            description: None,
            parameters_schema: None,
            timeout: Duration::from_secs(30),
            categories: vec!["agent".to_string()],
        }
    }
    
    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }
    
    pub fn description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }
    
    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }
    
    pub fn categories(mut self, categories: Vec<String>) -> Self {
        self.categories = categories;
        self
    }
    
    pub fn build(self) -> ToolWrappedAgent {
        let name = self.name.unwrap_or_else(|| 
            format!("{}_tool", self.agent.name()));
        let description = self.description.unwrap_or_else(|| 
            format!("Tool wrapper for {} agent", self.agent.name()));
        
        let parameters_schema = self.parameters_schema.unwrap_or_else(|| {
            serde_json::json!({
                "type": "object",
                "properties": {
                    "prompt": {
                        "type": "string",
                        "description": "Input prompt for the wrapped agent"
                    }
                },
                "required": ["prompt"]
            })
        });
        
        let return_schema = serde_json::json!({
            "type": "object",
            "properties": {
                "content": {
                    "type": "string",
                    "description": "Agent response content"
                },
                "tool_calls": {
                    "type": "array",
                    "description": "Tools called by the agent"
                },
                "usage": {
                    "type": "object",
                    "description": "Token usage information"
                }
            }
        });
        
        ToolWrappedAgent::new(
            self.agent,
            ToolConfig {
                name,
                description,
                parameters_schema,
                return_schema,
                timeout: self.timeout,
                retry_policy: RetryPolicy {
                    max_attempts: 3,
                    base_delay_ms: 1000,
                    max_delay_ms: 10000,
                    backoff_factor: 2.0,
                    retryable_errors: vec![
                        "TemporaryFailure".to_string(),
                        "RateLimit".to_string(),
                    ],
                },
                categories: self.categories,
            },
        )
    }
}
```

## Workflow: Orchestration Layer

Workflows coordinate multiple agents and tools through structured execution patterns. Each workflow type provides specific orchestration semantics.

### Core Workflow Interface

```rust
/// Workflow orchestration capability
#[async_trait]
pub trait Workflow: Send + Sync {
    type Error: std::error::Error + Send + Sync + 'static;
    type Step: WorkflowStep;
    
    // === Core Identity ===
    fn id(&self) -> &str;
    fn name(&self) -> &str;
    fn workflow_type(&self) -> WorkflowType;
    
    // === Step Management ===
    fn steps(&self) -> &[Self::Step];
    fn add_step(&mut self, step: Self::Step) -> Result<(), Self::Error>;
    fn remove_step(&mut self, step_id: &str) -> Result<bool, Self::Error>;
    fn get_step(&self, step_id: &str) -> Option<&Self::Step>;
    
    // === Execution ===
    async fn execute(&self, context: WorkflowContext) -> Result<WorkflowResult, Self::Error>;
    async fn execute_with_state(&self, context: WorkflowContext, state: &mut AgentState) 
        -> Result<WorkflowResult, Self::Error>;
    
    // === Progress Monitoring ===
    fn execute_with_progress(&self, context: WorkflowContext) 
        -> Pin<Box<dyn Stream<Item = Result<WorkflowProgress, Self::Error>> + Send>>;
    
    // === Control ===
    async fn pause(&self) -> Result<(), Self::Error>;
    async fn resume(&self) -> Result<(), Self::Error>;
    async fn cancel(&self) -> Result<(), Self::Error>;
    
    // === Validation ===
    async fn validate(&self) -> Result<(), Self::Error>;
    fn dependencies(&self) -> Vec<String>;
    
    // === State Integration ===
    async fn checkpoint(&self) -> Result<WorkflowCheckpoint, Self::Error>;
    async fn restore_from_checkpoint(&mut self, checkpoint: WorkflowCheckpoint) 
        -> Result<(), Self::Error>;
    
    // === Metadata ===
    fn metadata(&self) -> WorkflowMetadata;
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum WorkflowType {
    Sequential,
    Parallel,
    Conditional,
    Loop,
    Dag, // Directed Acyclic Graph
    Custom(String),
}

/// Individual step in a workflow
#[async_trait]
pub trait WorkflowStep: Send + Sync {
    type Error: std::error::Error + Send + Sync + 'static;
    
    // === Identity ===
    fn id(&self) -> &str;
    fn name(&self) -> &str;
    fn step_type(&self) -> WorkflowStepType;
    
    // === Execution ===
    async fn execute(&self, context: &mut WorkflowContext) 
        -> Result<StepResult, Self::Error>;
    
    // === Dependencies ===
    fn dependencies(&self) -> Vec<String>;
    fn outputs(&self) -> Vec<String>;
    
    // === Configuration ===
    fn timeout(&self) -> Duration;
    fn retry_policy(&self) -> RetryPolicy;
    fn is_optional(&self) -> bool;
    fn skip_condition(&self) -> Option<String>; // Expression to evaluate
    
    // === Validation ===
    async fn validate(&self) -> Result<(), Self::Error>;
    async fn can_execute(&self, context: &WorkflowContext) -> bool;
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum WorkflowStepType {
    Agent { agent_id: String },
    Tool { tool_name: String },
    SubWorkflow { workflow_id: String },
    Condition { expression: String },
    Parallel { sub_steps: Vec<String> },
    Custom(String),
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct StepResult {
    pub step_id: String,
    pub success: bool,
    pub output: StateValue,
    pub execution_time_ms: u64,
    pub retry_count: u32,
    pub error: Option<String>,
    pub side_effects: Vec<SideEffect>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct WorkflowCheckpoint {
    pub workflow_id: String,
    pub completed_steps: Vec<String>,
    pub current_step: Option<String>,
    pub context: WorkflowContext,
    pub timestamp: DateTime<Utc>,
    pub metadata: HashMap<String, StateValue>,
}
```

### Sequential Workflow

```rust
/// Sequential workflow executes steps one after another
pub struct SequentialWorkflow {
    pub id: String,
    pub name: String,
    pub steps: Vec<Box<dyn WorkflowStep>>,
    pub continue_on_error: bool,
    pub checkpoint_frequency: CheckpointFrequency,
    pub metadata: WorkflowMetadata,
}

#[derive(Debug, Clone)]
pub enum CheckpointFrequency {
    Never,
    EveryStep,
    EveryNSteps(usize),
    OnError,
    Custom(Box<dyn Fn(&WorkflowContext) -> bool + Send + Sync>),
}

#[async_trait]
impl Workflow for SequentialWorkflow {
    type Error = WorkflowError;
    type Step = Box<dyn WorkflowStep>;
    
    async fn execute(&self, mut context: WorkflowContext) -> Result<WorkflowResult, Self::Error> {
        let start_time = Utc::now();
        let mut step_results = HashMap::new();
        let mut errors = Vec::new();
        
        for (index, step) in self.steps.iter().enumerate() {
            // Check skip condition
            if let Some(condition) = step.skip_condition() {
                if self.evaluate_condition(&condition, &context)? {
                    continue;
                }
            }
            
            // Check dependencies
            if !self.dependencies_satisfied(step, &step_results) {
                let error = format!("Dependencies not satisfied for step {}", step.name());
                errors.push(error.clone());
                if !self.continue_on_error {
                    return Err(WorkflowError::StepFailed {
                        step_name: step.name().to_string(),
                        source: Box::new(WorkflowError::DependencyError { error }),
                    });
                }
                continue;
            }
            
            // Execute step with retries
            let step_result = self.execute_step_with_retry(step.as_ref(), &mut context).await;
            
            match step_result {
                Ok(result) => {
                    step_results.insert(step.id().to_string(), result.output.clone());
                    
                    // Update context with step output
                    context.step_results.insert(step.id().to_string(), result.output.clone());
                    
                    // Checkpoint if needed
                    if self.should_checkpoint(&context, index) {
                        let checkpoint = self.create_checkpoint(&context, step.id()).await?;
                        context.metadata.insert("last_checkpoint".to_string(), 
                            StateValue::String(serde_json::to_string(&checkpoint)?));
                    }
                }
                Err(e) => {
                    let error = format!("Step {} failed: {}", step.name(), e);
                    errors.push(error.clone());
                    
                    if !self.continue_on_error {
                        return Err(WorkflowError::StepFailed {
                            step_name: step.name().to_string(),
                            source: Box::new(e),
                        });
                    }
                }
            }
        }
        
        let end_time = Utc::now();
        let execution_time = (end_time - start_time).num_milliseconds() as u64;
        
        Ok(WorkflowResult {
            success: errors.is_empty(),
            step_results,
            final_output: context.variables.get("final_output").cloned(),
            execution_time_ms: execution_time,
            metadata: WorkflowMetadata {
                workflow_id: self.id.clone(),
                execution_id: context.execution_id.clone(),
                started_at: start_time,
                completed_at: Some(end_time),
                step_count: self.steps.len(),
                errors,
            },
        })
    }
    
    // ... additional methods
}
```

### Parallel Workflow

```rust
/// Parallel workflow executes steps concurrently
pub struct ParallelWorkflow {
    pub id: String,
    pub name: String,
    pub step_groups: Vec<ParallelStepGroup>,
    pub max_concurrency: Option<usize>,
    pub fail_fast: bool,
    pub metadata: WorkflowMetadata,
}

#[derive(Debug, Clone)]
pub struct ParallelStepGroup {
    pub group_id: String,
    pub steps: Vec<Box<dyn WorkflowStep>>,
    pub merge_strategy: MergeStrategy,
    pub timeout: Option<Duration>,
}

#[derive(Debug, Clone)]
pub enum MergeStrategy {
    All,           // Wait for all steps to complete
    First,         // Return when first step completes
    Majority,      // Wait for majority to complete
    Custom(String), // Custom merge logic
}

#[async_trait]
impl Workflow for ParallelWorkflow {
    type Error = WorkflowError;
    type Step = Box<dyn WorkflowStep>;
    
    async fn execute(&self, context: WorkflowContext) -> Result<WorkflowResult, Self::Error> {
        let start_time = Utc::now();
        let mut all_results = HashMap::new();
        let mut all_errors = Vec::new();
        
        for group in &self.step_groups {
            let group_results = self.execute_step_group(group, &context).await?;
            
            // Merge group results based on strategy
            match &group.merge_strategy {
                MergeStrategy::All => {
                    // Include all results
                    for (step_id, result) in group_results {
                        all_results.insert(step_id, result);
                    }
                }
                MergeStrategy::First => {
                    // Include only the first result
                    if let Some((step_id, result)) = group_results.into_iter().next() {
                        all_results.insert(step_id, result);
                    }
                }
                MergeStrategy::Majority => {
                    // Include results from majority of successful steps
                    let success_count = group_results.len();
                    let total_steps = group.steps.len();
                    if success_count > total_steps / 2 {
                        for (step_id, result) in group_results {
                            all_results.insert(step_id, result);
                        }
                    }
                }
                MergeStrategy::Custom(logic) => {
                    // Apply custom merge logic
                    let merged = self.apply_custom_merge(logic, group_results)?;
                    for (step_id, result) in merged {
                        all_results.insert(step_id, result);
                    }
                }
            }
        }
        
        let end_time = Utc::now();
        let execution_time = (end_time - start_time).num_milliseconds() as u64;
        
        Ok(WorkflowResult {
            success: all_errors.is_empty(),
            step_results: all_results,
            final_output: None, // Parallel workflows typically don't have a single final output
            execution_time_ms: execution_time,
            metadata: WorkflowMetadata {
                workflow_id: self.id.clone(),
                execution_id: context.execution_id,
                started_at: start_time,
                completed_at: Some(end_time),
                step_count: self.step_groups.iter().map(|g| g.steps.len()).sum(),
                errors: all_errors,
            },
        })
    }
    
    async fn execute_step_group(&self, group: &ParallelStepGroup, base_context: &WorkflowContext) 
        -> Result<HashMap<String, StateValue>, WorkflowError> {
        use futures::future::join_all;
        
        // Create context clone for each step
        let step_futures: Vec<_> = group.steps.iter().map(|step| {
            let mut step_context = base_context.clone();
            step_context.execution_id = format!("{}_{}", base_context.execution_id, step.id());
            
            async move {
                let result = step.execute(&mut step_context).await;
                (step.id().to_string(), result)
            }
        }).collect();
        
        // Execute all steps concurrently
        let results = if let Some(timeout) = group.timeout {
            tokio::time::timeout(timeout, join_all(step_futures)).await
                .map_err(|_| WorkflowError::Timeout { 
                    operation: format!("step group {}", group.group_id) 
                })?
        } else {
            join_all(step_futures).await
        };
        
        // Process results
        let mut success_results = HashMap::new();
        let mut errors = Vec::new();
        
        for (step_id, result) in results {
            match result {
                Ok(step_result) => {
                    success_results.insert(step_id, step_result.output);
                }
                Err(e) => {
                    errors.push(format!("Step {} failed: {}", step_id, e));
                    if self.fail_fast {
                        return Err(WorkflowError::StepFailed {
                            step_name: step_id,
                            source: Box::new(e),
                        });
                    }
                }
            }
        }
        
        Ok(success_results)
    }
}
```

### Conditional Workflow

```rust
/// Conditional workflow executes different paths based on conditions
pub struct ConditionalWorkflow {
    pub id: String,
    pub name: String,
    pub conditions: Vec<ConditionalBranch>,
    pub default_branch: Option<Box<dyn Workflow>>,
    pub metadata: WorkflowMetadata,
}

#[derive(Debug)]
pub struct ConditionalBranch {
    pub condition: String, // Expression to evaluate
    pub workflow: Box<dyn Workflow>,
    pub priority: u32,
}

#[async_trait]
impl Workflow for ConditionalWorkflow {
    type Error = WorkflowError;
    type Step = Box<dyn WorkflowStep>;
    
    async fn execute(&self, context: WorkflowContext) -> Result<WorkflowResult, Self::Error> {
        // Sort branches by priority
        let mut sorted_branches = self.conditions.iter().collect::<Vec<_>>();
        sorted_branches.sort_by_key(|b| b.priority);
        
        // Evaluate conditions in priority order
        for branch in sorted_branches {
            if self.evaluate_condition(&branch.condition, &context)? {
                return branch.workflow.execute(context).await;
            }
        }
        
        // Execute default branch if no conditions matched
        if let Some(default) = &self.default_branch {
            default.execute(context).await
        } else {
            Err(WorkflowError::NoMatchingCondition {
                workflow_id: self.id.clone(),
            })
        }
    }
    
    fn evaluate_condition(&self, condition: &str, context: &WorkflowContext) 
        -> Result<bool, WorkflowError> {
        // Simple expression evaluator - in real implementation would use a proper parser
        // For now, support basic variable comparisons
        if condition.contains("==") {
            let parts: Vec<&str> = condition.split("==").collect();
            if parts.len() == 2 {
                let left = self.resolve_variable(parts[0].trim(), context)?;
                let right = self.resolve_variable(parts[1].trim(), context)?;
                return Ok(left == right);
            }
        }
        
        // Default to false for unrecognized conditions
        Ok(false)
    }
    
    fn resolve_variable(&self, expr: &str, context: &WorkflowContext) 
        -> Result<StateValue, WorkflowError> {
        // Remove quotes if present
        let expr = expr.trim_matches('"').trim_matches('\'');
        
        // Check if it's a context variable
        if expr.starts_with("$") {
            let var_name = &expr[1..];
            return context.variables.get(var_name)
                .cloned()
                .ok_or_else(|| WorkflowError::VariableNotFound {
                    variable: var_name.to_string(),
                });
        }
        
        // Try to parse as literal value
        if let Ok(num) = expr.parse::<f64>() {
            return Ok(StateValue::Number(num));
        }
        
        if expr == "true" {
            return Ok(StateValue::Boolean(true));
        }
        
        if expr == "false" {
            return Ok(StateValue::Boolean(false));
        }
        
        // Default to string
        Ok(StateValue::String(expr.to_string()))
    }
}
```

### Loop Workflow

```rust
/// Loop workflow repeats execution until a condition is met
pub struct LoopWorkflow {
    pub id: String,
    pub name: String,
    pub body: Box<dyn Workflow>,
    pub condition: LoopCondition,
    pub max_iterations: Option<u32>,
    pub break_on_error: bool,
    pub metadata: WorkflowMetadata,
}

#[derive(Debug, Clone)]
pub enum LoopCondition {
    While(String),           // Continue while expression is true
    Until(String),           // Continue until expression is true
    ForEach { 
        variable: String,    // Variable name for current item
        collection: String,  // Expression that resolves to an array
    },
    Count(u32),             // Fixed number of iterations
}

#[async_trait]
impl Workflow for LoopWorkflow {
    type Error = WorkflowError;
    type Step = Box<dyn WorkflowStep>;
    
    async fn execute(&self, mut context: WorkflowContext) -> Result<WorkflowResult, Self::Error> {
        let start_time = Utc::now();
        let mut iteration = 0;
        let mut all_results = HashMap::new();
        let mut errors = Vec::new();
        
        loop {
            iteration += 1;
            
            // Check max iterations
            if let Some(max) = self.max_iterations {
                if iteration > max {
                    break;
                }
            }
            
            // Check loop condition
            match &self.condition {
                LoopCondition::While(expr) => {
                    if !self.evaluate_condition(expr, &context)? {
                        break;
                    }
                }
                LoopCondition::Until(expr) => {
                    if self.evaluate_condition(expr, &context)? {
                        break;
                    }
                }
                LoopCondition::ForEach { variable, collection } => {
                    // Get collection from context
                    let collection_value = context.variables.get(collection)
                        .ok_or_else(|| WorkflowError::VariableNotFound {
                            variable: collection.clone(),
                        })?;
                    
                    if let StateValue::Array(items) = collection_value {
                        if iteration as usize > items.len() {
                            break;
                        }
                        // Set current item as variable
                        let current_item = items.get((iteration - 1) as usize).unwrap().clone();
                        context.variables.insert(variable.clone(), current_item);
                    } else {
                        return Err(WorkflowError::InvalidType {
                            expected: "Array".to_string(),
                            actual: format!("{:?}", collection_value),
                        });
                    }
                }
                LoopCondition::Count(count) => {
                    if iteration > *count {
                        break;
                    }
                }
            }
            
            // Set iteration counter
            context.variables.insert("iteration".to_string(), 
                StateValue::Number(iteration as f64));
            
            // Execute loop body
            let mut iteration_context = context.clone();
            iteration_context.execution_id = format!("{}_iter_{}", context.execution_id, iteration);
            
            match self.body.execute(iteration_context).await {
                Ok(result) => {
                    // Merge results with iteration prefix
                    for (key, value) in result.step_results {
                        all_results.insert(format!("iter_{}_{}", iteration, key), value);
                    }
                    
                    // Update context with final output if present
                    if let Some(output) = result.final_output {
                        context.variables.insert("last_output".to_string(), output);
                    }
                }
                Err(e) => {
                    let error = format!("Iteration {} failed: {}", iteration, e);
                    errors.push(error.clone());
                    
                    if self.break_on_error {
                        break;
                    }
                }
            }
        }
        
        let end_time = Utc::now();
        let execution_time = (end_time - start_time).num_milliseconds() as u64;
        
        Ok(WorkflowResult {
            success: errors.is_empty(),
            step_results: all_results,
            final_output: context.variables.get("last_output").cloned(),
            execution_time_ms: execution_time,
            metadata: WorkflowMetadata {
                workflow_id: self.id.clone(),
                execution_id: context.execution_id,
                started_at: start_time,
                completed_at: Some(end_time),
                step_count: iteration as usize,
                errors,
            },
        })
    }
}
```

## State Integration

All components in the hierarchy are state-aware and participate in the state management system.

### State Integration Points

```rust
/// State integration across the hierarchy
#[async_trait]
pub trait StateAware {
    async fn load_state(&mut self, state_id: &str) -> Result<(), StateError>;
    async fn save_state(&self) -> Result<String, StateError>;
    async fn get_state_snapshot(&self) -> Result<AgentState, StateError>;
    async fn restore_from_snapshot(&mut self, snapshot: AgentState) -> Result<(), StateError>;
}

/// Implement StateAware for all components
#[async_trait]
impl StateAware for StandardBaseAgent {
    async fn load_state(&mut self, state_id: &str) -> Result<(), StateError> {
        let state = self.state_manager.load_state(state_id).await?;
        self.set_state(state).await.map_err(|e| StateError::LoadError { 
            state_id: state_id.to_string(), 
            source: Box::new(e) 
        })
    }
    
    async fn save_state(&self) -> Result<String, StateError> {
        let state = self.get_state().await.map_err(|e| StateError::SaveError { 
            source: Box::new(e) 
        })?;
        self.state_manager.save_state(&state).await
    }
}

/// State-driven agent handoff
pub struct StateHandoffManager {
    pub agents: HashMap<String, Arc<dyn Agent>>,
    pub state_manager: Arc<dyn StateManager>,
    pub routing_rules: Vec<HandoffRule>,
}

#[derive(Debug, Clone)]
pub struct HandoffRule {
    pub condition: String,
    pub from_agent: Option<String>, // None means any agent
    pub to_agent: String,
    pub priority: u32,
}

impl StateHandoffManager {
    pub async fn execute_handoff(&self, current_agent_id: &str, state: &mut AgentState) 
        -> Result<String, HandoffError> {
        // Find applicable handoff rules
        let applicable_rules = self.routing_rules.iter()
            .filter(|rule| {
                rule.from_agent.as_ref().map_or(true, |id| id == current_agent_id)
            })
            .filter(|rule| self.evaluate_condition(&rule.condition, state).unwrap_or(false))
            .collect::<Vec<_>>();
        
        if applicable_rules.is_empty() {
            return Err(HandoffError::NoApplicableRules);
        }
        
        // Sort by priority and take the highest
        let selected_rule = applicable_rules.into_iter()
            .max_by_key(|rule| rule.priority)
            .unwrap();
        
        // Get target agent
        let target_agent = self.agents.get(&selected_rule.to_agent)
            .ok_or_else(|| HandoffError::AgentNotFound {
                agent_id: selected_rule.to_agent.clone(),
            })?;
        
        // Check if target can accept handoff
        if !target_agent.can_accept_handoff(state).await {
            return Err(HandoffError::HandoffRejected {
                agent_id: selected_rule.to_agent.clone(),
                reason: "Agent cannot handle current state".to_string(),
            });
        }
        
        // Create handoff payload
        let payload = HandoffPayload {
            from_agent: current_agent_id.to_string(),
            to_agent: selected_rule.to_agent.clone(),
            state: state.clone(),
            context: HandoffContext {
                reason: format!("Rule-based handoff: {}", selected_rule.condition),
                expected_capabilities: vec![],
                return_expected: false,
                priority: HandoffPriority::Normal,
            },
            timestamp: Utc::now(),
        };
        
        // Execute handoff
        let mut target_agent_mut = Arc::clone(target_agent);
        Arc::get_mut(&mut target_agent_mut)
            .ok_or(HandoffError::AgentLocked)?
            .receive_handoff(payload).await?;
        
        // Update state ownership
        state.metadata.owner_agent = selected_rule.to_agent.clone();
        state.metadata.updated_at = Utc::now();
        
        Ok(selected_rule.to_agent.clone())
    }
}
```

## Hooks and Events Integration

Every component provides hook points for observability and custom behavior injection.

### Hook System

```rust
/// Hook execution points throughout the system
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum HookPoint {
    // BaseAgent hooks
    PreStateUpdate,
    PostStateUpdate,
    PreToolExecution,
    PostToolExecution,
    PreHandoff,
    PostHandoff,
    
    // Agent hooks
    PreLlm,
    PostLlm,
    PreConversation,
    PostConversation,
    
    // Workflow hooks
    PreWorkflow,
    PostWorkflow,
    PreStep,
    PostStep,
    
    // Tool hooks
    PreTool,
    PostTool,
    
    // Custom hooks
    Custom(String),
}

/// Hook execution interface
#[async_trait]
pub trait Hook: Send + Sync {
    fn id(&self) -> &str;
    fn hook_points(&self) -> Vec<HookPoint>;
    
    async fn execute(&self, point: HookPoint, context: &HookContext) 
        -> Result<HookResult, HookError>;
    
    fn priority(&self) -> u32; // Higher numbers execute first
    fn is_enabled(&self) -> bool;
}

#[derive(Debug, Clone)]
pub struct HookContext {
    pub component_id: String,
    pub component_type: String,
    pub execution_id: String,
    pub data: HashMap<String, StateValue>,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Clone)]
pub struct HookResult {
    pub continue_execution: bool,
    pub modify_data: Option<HashMap<String, StateValue>>,
    pub side_effects: Vec<SideEffect>,
    pub metadata: HashMap<String, String>,
}

/// Built-in logging hook
pub struct LoggingHook {
    pub id: String,
    pub logger: Arc<dyn Logger>,
    pub log_level: LogLevel,
    pub enabled_points: HashSet<HookPoint>,
}

#[async_trait]
impl Hook for LoggingHook {
    fn id(&self) -> &str { &self.id }
    
    fn hook_points(&self) -> Vec<HookPoint> {
        self.enabled_points.iter().cloned().collect()
    }
    
    async fn execute(&self, point: HookPoint, context: &HookContext) 
        -> Result<HookResult, HookError> {
        if !self.enabled_points.contains(&point) {
            return Ok(HookResult {
                continue_execution: true,
                modify_data: None,
                side_effects: vec![],
                metadata: HashMap::new(),
            });
        }
        
        let log_message = format!(
            "[{}] {} {} - Execution: {}",
            point.name(),
            context.component_type,
            context.component_id,
            context.execution_id
        );
        
        self.logger.log(self.log_level, &log_message, &context.data).await?;
        
        Ok(HookResult {
            continue_execution: true,
            modify_data: None,
            side_effects: vec![SideEffect {
                effect_type: SideEffectType::Other { 
                    category: "logging".to_string() 
                },
                description: log_message,
                reversible: false,
                metadata: HashMap::new(),
            }],
            metadata: HashMap::new(),
        })
    }
    
    fn priority(&self) -> u32 { 100 } // High priority for logging
    fn is_enabled(&self) -> bool { true }
}

/// Built-in metrics hook
pub struct MetricsHook {
    pub id: String,
    pub metrics_collector: Arc<dyn MetricsCollector>,
    pub enabled_points: HashSet<HookPoint>,
}

#[async_trait]
impl Hook for MetricsHook {
    fn id(&self) -> &str { &self.id }
    
    fn hook_points(&self) -> Vec<HookPoint> {
        self.enabled_points.iter().cloned().collect()
    }
    
    async fn execute(&self, point: HookPoint, context: &HookContext) 
        -> Result<HookResult, HookError> {
        let metric_name = format!("{}_{}", context.component_type, point.name());
        
        match point {
            HookPoint::PreLlm | HookPoint::PreTool | HookPoint::PreWorkflow => {
                self.metrics_collector.increment_counter(&format!("{}_started", metric_name));
                self.metrics_collector.record_histogram(&format!("{}_rate", metric_name), 1.0);
            }
            HookPoint::PostLlm | HookPoint::PostTool | HookPoint::PostWorkflow => {
                self.metrics_collector.increment_counter(&format!("{}_completed", metric_name));
                
                // Record execution time if available
                if let Some(StateValue::Number(duration)) = context.data.get("execution_time_ms") {
                    self.metrics_collector.record_histogram(
                        &format!("{}_duration_ms", metric_name), 
                        *duration
                    );
                }
            }
            _ => {
                self.metrics_collector.increment_counter(&metric_name);
            }
        }
        
        Ok(HookResult {
            continue_execution: true,
            modify_data: None,
            side_effects: vec![],
            metadata: HashMap::new(),
        })
    }
    
    fn priority(&self) -> u32 { 50 } // Medium priority
    fn is_enabled(&self) -> bool { true }
}
```

### Event System

```rust
/// Event types emitted by components
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum Event {
    // Agent events
    AgentCreated { agent_id: String, timestamp: DateTime<Utc> },
    AgentStarted { agent_id: String, input: String, timestamp: DateTime<Utc> },
    AgentCompleted { 
        agent_id: String, 
        duration_ms: u64, 
        success: bool,
        tool_calls_count: usize,
        timestamp: DateTime<Utc> 
    },
    
    // State events
    StateCreated { state_id: String, agent_id: String, timestamp: DateTime<Utc> },
    StateUpdated { agent_id: String, state_id: String, timestamp: DateTime<Utc> },
    StateHandoff { 
        from_agent: String, 
        to_agent: String, 
        state_id: String, 
        timestamp: DateTime<Utc> 
    },
    
    // Tool events
    ToolExecuted { 
        agent_id: String, 
        tool_name: String, 
        duration_ms: u64, 
        success: bool, 
        timestamp: DateTime<Utc> 
    },
    
    // Workflow events
    WorkflowStarted { workflow_id: String, timestamp: DateTime<Utc> },
    WorkflowCompleted { 
        workflow_id: String, 
        duration_ms: u64, 
        success: bool, 
        step_count: usize,
        timestamp: DateTime<Utc> 
    },
    StepStarted { workflow_id: String, step_id: String, timestamp: DateTime<Utc> },
    StepCompleted { 
        workflow_id: String, 
        step_id: String, 
        success: bool, 
        timestamp: DateTime<Utc> 
    },
    
    // Custom events
    Custom { 
        event_type: String, 
        data: HashMap<String, StateValue>, 
        timestamp: DateTime<Utc> 
    },
}

/// Event emitter interface
#[async_trait]
pub trait EventEmitter: Send + Sync {
    async fn emit(&self, event: Event);
    fn subscribe(&self, filter: EventFilter) -> Pin<Box<dyn Stream<Item = Event> + Send>>;
    async fn get_recent_events(&self, count: usize) -> Vec<Event>;
}

#[derive(Debug, Clone)]
pub struct EventFilter {
    pub event_types: Option<Vec<String>>,
    pub agent_ids: Option<Vec<String>>,
    pub since: Option<DateTime<Utc>>,
    pub until: Option<DateTime<Utc>>,
}

/// Built-in event emitter implementation
pub struct StandardEventEmitter {
    pub subscribers: Arc<RwLock<Vec<EventSubscriber>>>,
    pub recent_events: Arc<RwLock<VecDeque<Event>>>,
    pub max_recent_events: usize,
}

struct EventSubscriber {
    pub id: String,
    pub filter: EventFilter,
    pub sender: tokio::sync::mpsc::UnboundedSender<Event>,
}

#[async_trait]
impl EventEmitter for StandardEventEmitter {
    async fn emit(&self, event: Event) {
        // Store in recent events
        {
            let mut recent = self.recent_events.write().await;
            recent.push_back(event.clone());
            
            // Trim to max size
            while recent.len() > self.max_recent_events {
                recent.pop_front();
            }
        }
        
        // Send to subscribers
        let subscribers = self.subscribers.read().await;
        for subscriber in subscribers.iter() {
            if self.event_matches_filter(&event, &subscriber.filter) {
                let _ = subscriber.sender.send(event.clone());
            }
        }
    }
    
    fn subscribe(&self, filter: EventFilter) -> Pin<Box<dyn Stream<Item = Event> + Send>> {
        let (sender, receiver) = tokio::sync::mpsc::unbounded_channel();
        let subscriber_id = format!("sub_{}", Utc::now().timestamp_nanos());
        
        let subscriber = EventSubscriber {
            id: subscriber_id,
            filter,
            sender,
        };
        
        // Add subscriber
        tokio::spawn({
            let subscribers = Arc::clone(&self.subscribers);
            async move {
                let mut subs = subscribers.write().await;
                subs.push(subscriber);
            }
        });
        
        Box::pin(tokio_stream::wrappers::UnboundedReceiverStream::new(receiver))
    }
    
    async fn get_recent_events(&self, count: usize) -> Vec<Event> {
        let recent = self.recent_events.read().await;
        recent.iter()
            .rev()
            .take(count)
            .cloned()
            .collect()
    }
}
```

## Built-in Components Strategy

Rs-llmspell comes with a comprehensive library of built-in tools and agents for common tasks.

### Built-in Tools

```rust
/// Registry for built-in tools
pub struct BuiltinToolRegistry {
    pub tools: HashMap<String, Arc<dyn Tool>>,
    pub categories: HashMap<String, Vec<String>>,
}

impl BuiltinToolRegistry {
    pub fn new() -> Self {
        let mut registry = Self {
            tools: HashMap::new(),
            categories: HashMap::new(),
        };
        
        registry.register_core_tools();
        registry.register_file_tools();
        registry.register_web_tools();
        registry.register_data_tools();
        registry.register_ai_tools();
        
        registry
    }
    
    fn register_core_tools(&mut self) {
        // Calculator tool
        self.register_tool(Arc::new(CalculatorTool::new()));
        
        // JSON processor
        self.register_tool(Arc::new(JsonTool::new()));
        
        // Template renderer
        self.register_tool(Arc::new(TemplateTool::new()));
        
        // UUID generator
        self.register_tool(Arc::new(UuidTool::new()));
        
        // DateTime tools
        self.register_tool(Arc::new(DateTimeTool::new()));
    }
    
    fn register_file_tools(&mut self) {
        // File operations
        self.register_tool(Arc::new(FileReadTool::new()));
        self.register_tool(Arc::new(FileWriteTool::new()));
        self.register_tool(Arc::new(FileListTool::new()));
        
        // Text processing
        self.register_tool(Arc::new(TextSearchTool::new()));
        self.register_tool(Arc::new(TextReplaceTool::new()));
        
        // CSV processing
        self.register_tool(Arc::new(CsvReaderTool::new()));
        self.register_tool(Arc::new(CsvWriterTool::new()));
    }
    
    fn register_web_tools(&mut self) {
        // HTTP client
        self.register_tool(Arc::new(HttpRequestTool::new()));
        
        // Web scraping
        self.register_tool(Arc::new(WebScrapeTool::new()));
        
        // URL processing
        self.register_tool(Arc::new(UrlParseTool::new()));
    }
    
    fn register_data_tools(&mut self) {
        // Database operations
        self.register_tool(Arc::new(SqlQueryTool::new()));
        
        // Data transformation
        self.register_tool(Arc::new(DataFilterTool::new()));
        self.register_tool(Arc::new(DataSortTool::new()));
        self.register_tool(Arc::new(DataGroupTool::new()));
        
        // Statistical analysis
        self.register_tool(Arc::new(StatsTool::new()));
    }
    
    fn register_ai_tools(&mut self) {
        // Text analysis
        self.register_tool(Arc::new(SentimentAnalysisTool::new()));
        self.register_tool(Arc::new(LanguageDetectionTool::new()));
        
        // Content generation
        self.register_tool(Arc::new(SummarizerTool::new()));
        self.register_tool(Arc::new(TranslatorTool::new()));
    }
}

/// Example built-in tool: Calculator
pub struct CalculatorTool {
    pub id: String,
    pub version: String,
}

impl CalculatorTool {
    pub fn new() -> Self {
        Self {
            id: "calculator".to_string(),
            version: "1.0.0".to_string(),
        }
    }
}

#[async_trait]
impl Tool for CalculatorTool {
    type Error = ToolError;
    
    fn name(&self) -> &str { "calculator" }
    fn description(&self) -> &str { "Performs mathematical calculations" }
    fn version(&self) -> &str { &self.version }
    
    fn parameters_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "expression": {
                    "type": "string",
                    "description": "Mathematical expression to evaluate"
                }
            },
            "required": ["expression"]
        })
    }
    
    fn return_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "result": {
                    "type": "number",
                    "description": "Calculation result"
                },
                "expression": {
                    "type": "string", 
                    "description": "Original expression"
                }
            }
        })
    }
    
    async fn execute(&self, input: ToolInput, _context: ToolContext) 
        -> Result<ToolResult, Self::Error> {
        let expression = input.parameters.get("expression")
            .and_then(|v| v.as_string())
            .ok_or_else(|| ToolError::InvalidInput {
                field: "expression".to_string(),
                message: "expression parameter required".to_string(),
            })?;
        
        let start = std::time::Instant::now();
        
        // Simple expression evaluator (in real implementation, use a proper math parser)
        let result = self.evaluate_expression(expression)
            .map_err(|e| ToolError::Execution { message: e })?;
        
        let duration = start.elapsed();
        
        let output = StateValue::Object({
            let mut obj = HashMap::new();
            obj.insert("result".to_string(), StateValue::Number(result));
            obj.insert("expression".to_string(), StateValue::String(expression.clone()));
            obj
        });
        
        Ok(ToolResult {
            output,
            metadata: ToolResultMetadata {
                execution_time_ms: duration.as_millis() as u64,
                success: true,
                error_message: None,
                retry_count: 0,
                cache_hit: false,
                resource_usage: ResourceUsage {
                    cpu_time_ms: duration.as_millis() as u64,
                    memory_bytes: 1024, // Minimal memory usage
                    network_bytes_sent: 0,
                    network_bytes_received: 0,
                    file_operations: 0,
                },
            },
            side_effects: vec![],
        })
    }
    
    fn evaluate_expression(&self, expr: &str) -> Result<f64, String> {
        // Simple arithmetic evaluator - in real implementation would use evalexpr or similar
        match expr.trim() {
            e if e.contains('+') => {
                let parts: Vec<&str> = e.split('+').collect();
                if parts.len() == 2 {
                    let a = parts[0].trim().parse::<f64>().map_err(|_| "Invalid number".to_string())?;
                    let b = parts[1].trim().parse::<f64>().map_err(|_| "Invalid number".to_string())?;
                    return Ok(a + b);
                }
            }
            e if e.contains('-') => {
                let parts: Vec<&str> = e.split('-').collect();
                if parts.len() == 2 {
                    let a = parts[0].trim().parse::<f64>().map_err(|_| "Invalid number".to_string())?;
                    let b = parts[1].trim().parse::<f64>().map_err(|_| "Invalid number".to_string())?;
                    return Ok(a - b);
                }
            }
            e if e.contains('*') => {
                let parts: Vec<&str> = e.split('*').collect();
                if parts.len() == 2 {
                    let a = parts[0].trim().parse::<f64>().map_err(|_| "Invalid number".to_string())?;
                    let b = parts[1].trim().parse::<f64>().map_err(|_| "Invalid number".to_string())?;
                    return Ok(a * b);
                }
            }
            e if e.contains('/') => {
                let parts: Vec<&str> = e.split('/').collect();
                if parts.len() == 2 {
                    let a = parts[0].trim().parse::<f64>().map_err(|_| "Invalid number".to_string())?;
                    let b = parts[1].trim().parse::<f64>().map_err(|_| "Invalid number".to_string())?;
                    if b == 0.0 {
                        return Err("Division by zero".to_string());
                    }
                    return Ok(a / b);
                }
            }
            e => {
                // Try to parse as number
                return e.parse::<f64>().map_err(|_| "Invalid expression".to_string());
            }
        }
        
        Err("Unsupported expression".to_string())
    }
    
    async fn validate_input(&self, input: &ToolInput) -> Result<(), Self::Error> {
        if !input.parameters.contains_key("expression") {
            return Err(ToolError::InvalidInput {
                field: "expression".to_string(),
                message: "expression parameter is required".to_string(),
            });
        }
        Ok(())
    }
    
    async fn health_check(&self) -> Result<(), Self::Error> {
        // Test basic calculation
        let test_input = ToolInput {
            parameters: {
                let mut params = HashMap::new();
                params.insert("expression".to_string(), StateValue::String("2 + 2".to_string()));
                params
            },
            metadata: ToolInputMetadata {
                source: ToolInputSource::System,
                validation_level: ValidationLevel::Basic,
                timeout_override: None,
                retry_override: None,
            },
        };
        
        let context = ToolContext {
            execution_id: "health_check".to_string(),
            agent_id: "system".to_string(),
            session_id: None,
            conversation_id: None,
            permissions: HashSet::new(),
            environment: HashMap::new(),
            timeout: Duration::from_secs(1),
            retry_policy: RetryPolicy {
                max_attempts: 1,
                base_delay_ms: 0,
                max_delay_ms: 0,
                backoff_factor: 1.0,
                retryable_errors: vec![],
            },
        };
        
        let result = self.execute(test_input, context).await?;
        
        // Verify result
        if let StateValue::Object(obj) = &result.output {
            if let Some(StateValue::Number(value)) = obj.get("result") {
                if (*value - 4.0).abs() < f64::EPSILON {
                    return Ok(());
                }
            }
        }
        
        Err(ToolError::Execution {
            message: "Health check failed: unexpected result".to_string(),
        })
    }
    
    fn categories(&self) -> Vec<String> {
        vec!["math".to_string(), "calculation".to_string(), "utility".to_string()]
    }
    
    fn requires_permission(&self) -> bool { false }
    fn is_builtin(&self) -> bool { true }
    fn dependencies(&self) -> Vec<String> { vec![] }
    fn supports_streaming(&self) -> bool { false }
    fn timeout(&self) -> Duration { Duration::from_secs(5) }
    
    fn retry_policy(&self) -> RetryPolicy {
        RetryPolicy {
            max_attempts: 3,
            base_delay_ms: 100,
            max_delay_ms: 1000,
            backoff_factor: 2.0,
            retryable_errors: vec!["TemporaryFailure".to_string()],
        }
    }
}
```

## Examples and Usage Patterns

### Basic Agent with Tools

```rust
use rs_llmspell::prelude::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create provider
    let provider = AnthropicProvider::new(AnthropicConfig {
        api_key: std::env::var("ANTHROPIC_API_KEY")?,
        base_url: None,
        timeout: Duration::from_secs(30),
    }).await?;
    
    // Create base agent
    let mut base_agent = StandardBaseAgent::new(
        "research_agent",
        "Research Assistant",
        "Helps with research tasks using web search and file operations"
    );
    
    // Add built-in tools
    let builtin_registry = BuiltinToolRegistry::new();
    base_agent.add_tool(builtin_registry.get_tool("web_search").unwrap()).await?;
    base_agent.add_tool(builtin_registry.get_tool("file_write").unwrap()).await?;
    base_agent.add_tool(builtin_registry.get_tool("calculator").unwrap()).await?;
    
    // Create LLM agent
    let agent = StandardAgent::new(base_agent, provider)
        .with_model("claude-3-opus")
        .with_system_prompt("You are a helpful research assistant with access to web search and file operations.")
        .with_temperature(0.7);
    
    // Add hooks
    let logging_hook = LoggingHook::new("logger", LogLevel::Info);
    agent.register_hook(Arc::new(logging_hook)).await?;
    
    let metrics_hook = MetricsHook::new("metrics");
    agent.register_hook(Arc::new(metrics_hook)).await?;
    
    // Execute research task
    let response = agent.run(
        "Research the latest developments in quantum computing and save a summary to quantum_research.md"
    ).await?;
    
    println!("Research completed: {}", response.content);
    println!("Tools used: {}", response.tool_calls.len());
    println!("Execution time: {}ms", response.metadata.execution_time_ms);
    
    Ok(())
}
```

### Agent-to-Agent Handoff

```rust
async fn agent_handoff_example() -> Result<(), Box<dyn std::error::Error>> {
    // Create multiple specialized agents
    let researcher = create_research_agent().await?;
    let writer = create_writing_agent().await?;
    let editor = create_editing_agent().await?;
    
    // Set up handoff manager
    let mut handoff_manager = StateHandoffManager::new();
    handoff_manager.register_agent("researcher", researcher.clone());
    handoff_manager.register_agent("writer", writer.clone());
    handoff_manager.register_agent("editor", editor.clone());
    
    // Define handoff rules
    handoff_manager.add_rule(HandoffRule {
        condition: "$task_type == 'research'".to_string(),
        from_agent: None,
        to_agent: "researcher".to_string(),
        priority: 100,
    });
    
    handoff_manager.add_rule(HandoffRule {
        condition: "$research_complete == true".to_string(),
        from_agent: Some("researcher".to_string()),
        to_agent: "writer".to_string(),
        priority: 90,
    });
    
    handoff_manager.add_rule(HandoffRule {
        condition: "$draft_complete == true".to_string(),
        from_agent: Some("writer".to_string()),
        to_agent: "editor".to_string(),
        priority: 80,
    });
    
    // Create initial state
    let mut state = AgentState {
        state_id: "content_creation_task".to_string(),
        agent_id: "system".to_string(),
        variables: {
            let mut vars = HashMap::new();
            vars.insert("task_type".to_string(), StateValue::String("research".to_string()));
            vars.insert("topic".to_string(), StateValue::String("AI safety".to_string()));
            vars.insert("target_length".to_string(), StateValue::Number(2000.0));
            vars
        },
        context: AgentContext {
            conversation_id: Some("content_task_001".to_string()),
            session_id: Some("session_123".to_string()),
            parent_context: None,
            execution_trace: vec![],
            permissions: ["web_search", "file_write", "file_read"].iter()
                .map(|s| s.to_string()).collect(),
        },
        metadata: StateMetadata {
            created_at: Utc::now(),
            updated_at: Utc::now(),
            owner_agent: "system".to_string(),
            version: 1,
            tags: vec!["content_creation".to_string()],
        },
    };
    
    // Execute handoff chain
    let mut current_agent = "system";
    
    loop {
        // Determine next agent
        let next_agent = handoff_manager.execute_handoff(current_agent, &mut state).await?;
        
        if next_agent == current_agent {
            break; // No more handoffs
        }
        
        // Get agent and process
        let agent = handoff_manager.get_agent(&next_agent)?;
        
        // Determine what the agent should do based on state
        let task = determine_task_for_agent(&next_agent, &state)?;
        
        // Execute agent task
        let response = agent.run_with_state(&task, &mut state).await?;
        
        println!("Agent {} completed task: {}", next_agent, response.content);
        
        // Update state based on agent completion
        update_state_after_agent(&next_agent, &mut state, &response).await?;
        
        current_agent = &next_agent;
    }
    
    println!("Content creation pipeline completed!");
    Ok(())
}

fn determine_task_for_agent(agent_id: &str, state: &AgentState) -> Result<String, Box<dyn std::error::Error>> {
    match agent_id {
        "researcher" => {
            let topic = state.variables.get("topic")
                .and_then(|v| v.as_string())
                .ok_or("Topic not found in state")?;
            Ok(format!("Research the topic: {}. Gather key information and save research notes.", topic))
        }
        "writer" => {
            Ok("Based on the research notes, write a comprehensive article on the topic.".to_string())
        }
        "editor" => {
            Ok("Review and edit the drafted article for clarity, accuracy, and flow.".to_string())
        }
        _ => Err(format!("Unknown agent: {}", agent_id).into())
    }
}

async fn update_state_after_agent(
    agent_id: &str, 
    state: &mut AgentState, 
    response: &AgentResponse
) -> Result<(), Box<dyn std::error::Error>> {
    match agent_id {
        "researcher" => {
            state.variables.insert("research_complete".to_string(), StateValue::Boolean(true));
            state.variables.insert("research_notes".to_string(), 
                StateValue::String(response.content.to_string()));
        }
        "writer" => {
            state.variables.insert("draft_complete".to_string(), StateValue::Boolean(true));
            state.variables.insert("article_draft".to_string(), 
                StateValue::String(response.content.to_string()));
        }
        "editor" => {
            state.variables.insert("editing_complete".to_string(), StateValue::Boolean(true));
            state.variables.insert("final_article".to_string(), 
                StateValue::String(response.content.to_string()));
        }
        _ => {}
    }
    
    state.metadata.updated_at = Utc::now();
    state.metadata.version += 1;
    
    Ok(())
}
```

### Sequential Workflow with Error Handling

```rust
async fn workflow_example() -> Result<(), Box<dyn std::error::Error>> {
    // Create agents
    let data_fetcher = create_data_fetcher_agent().await?;
    let analyzer = create_analysis_agent().await?;
    let reporter = create_reporting_agent().await?;
    
    // Create workflow steps
    let fetch_step = AgentStep::new("fetch_data", data_fetcher, 
        "Fetch sales data for Q4 2024");
    
    let analyze_step = AgentStep::new("analyze_data", analyzer, 
        "Analyze the sales data and identify trends: {{fetch_data.output}}");
    
    let report_step = AgentStep::new("generate_report", reporter, 
        "Generate a comprehensive report based on analysis: {{analyze_data.output}}");
    
    // Create sequential workflow
    let mut workflow = SequentialWorkflow::new(
        "sales_analysis_workflow",
        "Sales Analysis Pipeline"
    );
    
    workflow.add_step(Box::new(fetch_step))?;
    workflow.add_step(Box::new(analyze_step))?;
    workflow.add_step(Box::new(report_step))?;
    
    // Configure error handling
    workflow.continue_on_error = false;
    workflow.checkpoint_frequency = CheckpointFrequency::EveryStep;
    
    // Add hooks for monitoring
    let workflow_logger = LoggingHook::new("workflow_logger", LogLevel::Info);
    workflow.register_hook(Arc::new(workflow_logger)).await?;
    
    // Create execution context
    let context = WorkflowContext {
        execution_id: "sales_analysis_001".to_string(),
        variables: {
            let mut vars = HashMap::new();
            vars.insert("quarter".to_string(), StateValue::String("Q4".to_string()));
            vars.insert("year".to_string(), StateValue::Number(2024.0));
            vars
        },
        step_results: HashMap::new(),
        metadata: HashMap::new(),
    };
    
    // Execute workflow with progress monitoring
    let mut progress_stream = workflow.execute_with_progress(context);
    
    while let Some(progress) = progress_stream.next().await {
        match progress {
            Ok(progress) => {
                println!("Progress: {} ({}/{})", 
                    progress.current_step,
                    progress.completed_steps,
                    progress.total_steps
                );
            }
            Err(e) => {
                eprintln!("Workflow error: {}", e);
                break;
            }
        }
    }
    
    Ok(())
}
```

### Tool-Wrapped Agent Composition

```rust
async fn tool_wrapped_agent_example() -> Result<(), Box<dyn std::error::Error>> {
    // Create specialized agents
    let code_analyzer = create_code_analysis_agent().await?;
    let security_scanner = create_security_scanning_agent().await?;
    let documentation_generator = create_documentation_agent().await?;
    
    // Wrap agents as tools
    let analyzer_tool = ToolWrappedAgent::builder(code_analyzer)
        .name("code_analyzer")
        .description("Analyzes code quality and structure")
        .timeout(Duration::from_secs(120))
        .categories(vec!["analysis".to_string(), "code".to_string()])
        .build();
    
    let security_tool = ToolWrappedAgent::builder(security_scanner)
        .name("security_scanner") 
        .description("Scans code for security vulnerabilities")
        .timeout(Duration::from_secs(180))
        .categories(vec!["security".to_string(), "scanning".to_string()])
        .build();
    
    let docs_tool = ToolWrappedAgent::builder(documentation_generator)
        .name("docs_generator")
        .description("Generates comprehensive documentation")
        .timeout(Duration::from_secs(240))
        .categories(vec!["documentation".to_string(), "generation".to_string()])
        .build();
    
    // Create meta-agent that coordinates other agents
    let mut meta_agent = create_coordination_agent().await?;
    meta_agent.add_tool(Arc::new(analyzer_tool)).await?;
    meta_agent.add_tool(Arc::new(security_tool)).await?;
    meta_agent.add_tool(Arc::new(docs_tool)).await?;
    
    // Add built-in tools as well
    let builtin_registry = BuiltinToolRegistry::new();
    meta_agent.add_tool(builtin_registry.get_tool("file_read").unwrap()).await?;
    meta_agent.add_tool(builtin_registry.get_tool("file_write").unwrap()).await?;
    
    // Execute coordinated code review
    let response = meta_agent.run(
        "Perform a comprehensive code review of the project in ./src/. \
         Use the code analyzer to check quality, security scanner for vulnerabilities, \
         and docs generator to create documentation. Provide a summary report."
    ).await?;
    
    println!("Code review completed:");
    println!("{}", response.content);
    
    // Show tool usage
    for tool_call in &response.tool_calls {
        println!("Used tool: {} -> {}", tool_call.name, 
            tool_call.output.as_ref().map(|o| format!("{:?}", o)).unwrap_or("None".to_string()));
    }
    
    Ok(())
}

async fn create_coordination_agent() -> Result<Arc<dyn Agent>, Box<dyn std::error::Error>> {
    let provider = AnthropicProvider::new(AnthropicConfig {
        api_key: std::env::var("ANTHROPIC_API_KEY")?,
        base_url: None,
        timeout: Duration::from_secs(30),
    }).await?;
    
    let base_agent = StandardBaseAgent::new(
        "coordinator",
        "Code Review Coordinator", 
        "Coordinates multiple specialized agents for comprehensive code review"
    );
    
    let agent = StandardAgent::new(base_agent, provider)
        .with_model("claude-3-opus")
        .with_system_prompt(
            "You are a code review coordinator. You have access to specialized agents \
             for code analysis, security scanning, and documentation generation. \
             Use these tools systematically to provide comprehensive code reviews."
        )
        .with_temperature(0.3);
    
    Ok(Arc::new(agent))
}
```

## Conclusion

This comprehensive design provides a robust, state-driven architecture for rs-llmspell that:

1. **Enables Flexible Agent Composition**: BaseAgent → Agent → ToolWrappedAgent layers allow powerful composition patterns
2. **Supports State-Driven Handoff**: Agents can hand off to each other through shared state without rigid workflows
3. **Provides Rich Observability**: Comprehensive hooks and events system for monitoring and debugging
4. **Includes Built-in Components**: 30-40 ready-to-use tools and common agent patterns
5. **Integrates with Bridge Layer**: Clean interfaces for script engine integration
6. **Maintains Type Safety**: Consistent error handling and type conversions throughout

The design enables both simple single-agent use cases and complex multi-agent orchestrations while maintaining the bridge-first philosophy that exposes Rust LLM library functionality to scripts without reimplementing core features.