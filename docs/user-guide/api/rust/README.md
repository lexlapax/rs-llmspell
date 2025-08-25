# LLMSpell Rust API Reference

**Version**: 0.6.0  
**Status**: Production Ready  
**Purpose**: Complete API reference for extending LLMSpell with Rust

> **🦀 COMPREHENSIVE REFERENCE**: This document provides complete API documentation for all public Rust traits, structs, and patterns in LLMSpell. Designed for both human developers and LLM-based coding assistants developing Rust extensions.

## Table of Contents

1. [Core Traits](#core-traits) - Fundamental trait definitions
2. [Agent API](#agent-api) - Agent trait and implementations
3. [Tool API](#tool-api) - Tool trait and registry
4. [Workflow API](#workflow-api) - Workflow traits and execution
5. [State Management](#state-management) - State persistence traits
6. [Hook System](#hook-system) - Hook traits and lifecycle
7. [Event System](#event-system) - Event emission and handling
8. [Provider API](#provider-api) - LLM provider traits
9. [Error Types](#error-types) - Error handling
10. [Builder Patterns](#builder-patterns) - Builder APIs
11. [Component Registry](#component-registry) - Component registration
12. [Bridge APIs](#bridge-apis) - Script language bridges

---

## Core Traits

Core traits define the fundamental abstractions in LLMSpell.

### BaseComponent Trait

Base trait for all LLMSpell components.

```rust
use llmspell_core::traits::BaseComponent;
use async_trait::async_trait;

#[async_trait]
pub trait BaseComponent: Send + Sync {
    /// Unique identifier for the component
    fn id(&self) -> &str;
    
    /// Component type identifier
    fn component_type(&self) -> &str;
    
    /// Human-readable description
    fn description(&self) -> Option<&str> {
        None
    }
    
    /// Component metadata
    fn metadata(&self) -> Option<serde_json::Value> {
        None
    }
}
```

**Implementation Example:**
```rust
struct MyComponent {
    id: String,
}

#[async_trait]
impl BaseComponent for MyComponent {
    fn id(&self) -> &str {
        &self.id
    }
    
    fn component_type(&self) -> &str {
        "custom"
    }
}
```

### Executable Trait

Trait for executable components.

```rust
use llmspell_core::traits::Executable;

#[async_trait]
pub trait Executable: BaseComponent {
    /// Input type for execution
    type Input: Send + Sync;
    
    /// Output type for execution
    type Output: Send + Sync;
    
    /// Executes the component
    async fn execute(
        &self,
        input: Self::Input,
        context: &ExecutionContext,
    ) -> Result<Self::Output, ComponentError>;
}
```

---

## Agent API

Agent-related types and traits for LLM interaction.

### Agent Trait

Core trait for agent implementations.

```rust
use llmspell_agents::traits::Agent;
use llmspell_core::ExecutionContext;

#[async_trait]
pub trait Agent: BaseComponent + Send + Sync {
    /// Executes the agent with given input
    async fn execute(
        &self,
        input: AgentInput,
        context: &ExecutionContext,
    ) -> Result<AgentOutput, AgentError>;
    
    /// Gets agent configuration
    fn config(&self) -> &AgentConfig;
    
    /// Lists available tools
    fn tools(&self) -> Vec<String> {
        vec![]
    }
}
```

### AgentInput Struct

Input structure for agent execution.

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentInput {
    /// The prompt to process
    pub prompt: String,
    
    /// Optional conversation history
    pub messages: Option<Vec<Message>>,
    
    /// Temperature override
    pub temperature: Option<f32>,
    
    /// Max tokens override
    pub max_tokens: Option<u32>,
    
    /// Additional context
    pub context: Option<serde_json::Value>,
}

impl AgentInput {
    pub fn new(prompt: impl Into<String>) -> Self {
        Self {
            prompt: prompt.into(),
            messages: None,
            temperature: None,
            max_tokens: None,
            context: None,
        }
    }
}
```

### AgentOutput Struct

Output structure from agent execution.

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentOutput {
    /// Response content
    pub content: String,
    
    /// Token usage statistics
    pub usage: Option<TokenUsage>,
    
    /// Response metadata
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenUsage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
}
```

### AgentBuilder

Builder for creating agents.

```rust
use llmspell_agents::AgentBuilder;

let agent = AgentBuilder::new("assistant")
    .agent_type(AgentType::LLM)
    .model("gpt-4")
    .provider("openai")
    .system_prompt("You are a helpful assistant")
    .temperature(0.7)
    .max_tokens(1000)
    .build()
    .await?;
```

### Agent Registration

Register custom agents with the system.

```rust
use llmspell_agents::registry::AgentRegistry;

let registry = AgentRegistry::global();
registry.register(agent).await?;

// Retrieve agent
let agent = registry.get("assistant").await?;

// List all agents
let agents = registry.list().await?;
```

---

## Tool API

Tool system for extending functionality.

### Tool Trait

Core trait for tool implementations.

```rust
use llmspell_tools::traits::Tool;

#[async_trait]
pub trait Tool: BaseComponent + Send + Sync {
    /// Tool input type
    type Input: DeserializeOwned + Send;
    
    /// Tool output type
    type Output: Serialize + Send;
    
    /// Executes the tool
    async fn execute(
        &self,
        input: Self::Input,
    ) -> Result<Self::Output, ToolError>;
    
    /// Tool parameter schema
    fn schema(&self) -> serde_json::Value;
}
```

### Creating Custom Tools

```rust
use llmspell_tools::{Tool, ToolError};

pub struct FileTool;

#[derive(Deserialize)]
pub struct FileInput {
    pub path: String,
    pub operation: String,
}

#[derive(Serialize)]
pub struct FileOutput {
    pub success: bool,
    pub content: Option<String>,
}

#[async_trait]
impl Tool for FileTool {
    type Input = FileInput;
    type Output = FileOutput;
    
    async fn execute(
        &self,
        input: Self::Input,
    ) -> Result<Self::Output, ToolError> {
        // Implementation
        Ok(FileOutput {
            success: true,
            content: Some("File content".to_string()),
        })
    }
    
    fn schema(&self) -> serde_json::Value {
        json!({
            "type": "object",
            "properties": {
                "path": {"type": "string"},
                "operation": {"type": "string"}
            },
            "required": ["path", "operation"]
        })
    }
}
```

### Tool Registry

```rust
use llmspell_tools::registry::ToolRegistry;

let registry = ToolRegistry::global();

// Register tool
registry.register("file-tool", Box::new(FileTool)).await?;

// Invoke tool
let result = registry.invoke(
    "file-tool",
    json!({"path": "/tmp/test.txt", "operation": "read"})
).await?;
```

---

## Workflow API

Workflow orchestration system.

### Workflow Trait

Core trait for workflows.

```rust
use llmspell_workflows::traits::Workflow;

#[async_trait]
pub trait Workflow: BaseComponent + Send + Sync {
    /// Executes the workflow
    async fn execute(
        &self,
        context: WorkflowContext,
    ) -> Result<WorkflowResult, WorkflowError>;
    
    /// Workflow type
    fn workflow_type(&self) -> WorkflowType;
    
    /// Validates workflow configuration
    fn validate(&self) -> Result<(), WorkflowError> {
        Ok(())
    }
}
```

### WorkflowBuilder

Builder for creating workflows.

```rust
use llmspell_workflows::WorkflowBuilder;

let workflow = WorkflowBuilder::sequential("data-pipeline")
    .add_step(Step::agent("analyze", "analyzer"))
    .add_step(Step::tool("process", "processor"))
    .add_step(Step::agent("summarize", "summarizer"))
    .build()?;

let result = workflow.execute(context).await?;
```

### Workflow Types

```rust
pub enum WorkflowType {
    Sequential,
    Parallel,
    Conditional,
    Loop,
    Nested,
}
```

### Step Definition

```rust
use llmspell_workflows::Step;

pub struct Step {
    pub name: String,
    pub step_type: StepType,
    pub timeout: Option<Duration>,
    pub retry_policy: Option<RetryPolicy>,
}

pub enum StepType {
    Agent { id: String },
    Tool { name: String },
    Function { handler: Box<dyn StepHandler> },
    Workflow { id: String },
}
```

---

## State Management

State persistence and management.

### StateManager Trait

```rust
use llmspell_state_persistence::traits::StateManager;

#[async_trait]
pub trait StateManager: Send + Sync {
    /// Saves state value
    async fn save(
        &self,
        key: &str,
        value: &[u8],
    ) -> Result<(), StateError>;
    
    /// Loads state value
    async fn load(
        &self,
        key: &str,
    ) -> Result<Option<Vec<u8>>, StateError>;
    
    /// Checks if key exists
    async fn exists(
        &self,
        key: &str,
    ) -> Result<bool, StateError>;
    
    /// Deletes state value
    async fn delete(
        &self,
        key: &str,
    ) -> Result<(), StateError>;
    
    /// Lists all keys
    async fn list_keys(&self) -> Result<Vec<String>, StateError>;
}
```

### Using StateManager

```rust
use llmspell_state_persistence::StateManagerBuilder;

let state_manager = StateManagerBuilder::new()
    .with_backend(Backend::Sqlite)
    .with_path("/tmp/state.db")
    .build()
    .await?;

// Save state
state_manager.save("user:123", b"preferences").await?;

// Load state
if let Some(data) = state_manager.load("user:123").await? {
    let preferences = String::from_utf8(data)?;
}
```

---

## Hook System

Lifecycle hooks and event interception.

### Hook Trait

```rust
use llmspell_hooks::traits::Hook;

#[async_trait]
pub trait Hook: Send + Sync {
    /// Hook name
    fn name(&self) -> &str;
    
    /// Hook priority (lower executes first)
    fn priority(&self) -> i32 {
        100
    }
    
    /// Executes the hook
    async fn execute(
        &self,
        event: HookEvent,
        data: HookData,
    ) -> Result<HookResult, HookError>;
}
```

### HookEvent Enum

```rust
pub enum HookEvent {
    BeforeToolExecution,
    AfterToolExecution,
    BeforeAgentExecution,
    AfterAgentExecution,
    BeforeWorkflowStep,
    AfterWorkflowStep,
    SessionStart,
    SessionEnd,
    Custom(String),
}
```

### Registering Hooks

```rust
use llmspell_hooks::HookRegistry;

struct LoggingHook;

#[async_trait]
impl Hook for LoggingHook {
    fn name(&self) -> &str {
        "logging"
    }
    
    async fn execute(
        &self,
        event: HookEvent,
        data: HookData,
    ) -> Result<HookResult, HookError> {
        println!("Hook: {:?}", event);
        Ok(HookResult::Continue)
    }
}

let registry = HookRegistry::global();
registry.register(HookEvent::BeforeAgentExecution, Box::new(LoggingHook))?;
```

---

## Event System

Event emission and subscription system.

### EventEmitter Trait

```rust
use llmspell_events::traits::EventEmitter;

#[async_trait]
pub trait EventEmitter: Send + Sync {
    /// Emits an event
    async fn emit(
        &self,
        event: Event,
    ) -> Result<(), EventError>;
}
```

### Event Struct

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Event {
    pub id: Uuid,
    pub event_type: String,
    pub timestamp: SystemTime,
    pub data: serde_json::Value,
    pub metadata: EventMetadata,
}

impl Event {
    pub fn new(event_type: impl Into<String>, data: serde_json::Value) -> Self {
        Self {
            id: Uuid::new_v4(),
            event_type: event_type.into(),
            timestamp: SystemTime::now(),
            data,
            metadata: EventMetadata::default(),
        }
    }
}
```

### Event Subscription

```rust
use llmspell_events::EventBus;

let bus = EventBus::global();

// Subscribe to events
let subscription = bus.subscribe("user_action", |event| {
    println!("User action: {:?}", event.data);
})?;

// Emit event
bus.emit(Event::new("user_action", json!({
    "action": "click",
    "target": "button"
})))?;

// Unsubscribe
bus.unsubscribe(subscription)?;
```

---

## Provider API

LLM provider integration.

### Provider Trait

```rust
use llmspell_providers::traits::Provider;

#[async_trait]
pub trait Provider: Send + Sync {
    /// Provider name
    fn name(&self) -> &str;
    
    /// Lists available models
    async fn list_models(&self) -> Result<Vec<Model>, ProviderError>;
    
    /// Creates a completion
    async fn complete(
        &self,
        request: CompletionRequest,
    ) -> Result<CompletionResponse, ProviderError>;
    
    /// Streams a completion
    async fn stream(
        &self,
        request: CompletionRequest,
    ) -> Result<Box<dyn Stream<Item = Result<CompletionChunk, ProviderError>>>, ProviderError>;
}
```

### CompletionRequest

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletionRequest {
    pub model: String,
    pub messages: Vec<Message>,
    pub temperature: Option<f32>,
    pub max_tokens: Option<u32>,
    pub top_p: Option<f32>,
    pub stop: Option<Vec<String>>,
    pub stream: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub role: MessageRole,
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MessageRole {
    System,
    User,
    Assistant,
}
```

---

## Error Types

Common error types used throughout LLMSpell.

### ComponentError

```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ComponentError {
    #[error("Component not found: {0}")]
    NotFound(String),
    
    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),
    
    #[error("Execution failed: {0}")]
    ExecutionFailed(String),
    
    #[error("Timeout after {0:?}")]
    Timeout(Duration),
    
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}
```

### Result Type Aliases

```rust
pub type AgentResult<T> = Result<T, AgentError>;
pub type ToolResult<T> = Result<T, ToolError>;
pub type WorkflowResult<T> = Result<T, WorkflowError>;
pub type StateResult<T> = Result<T, StateError>;
```

---

## Builder Patterns

Common builder patterns in LLMSpell.

### Generic Builder Pattern

```rust
pub struct Builder<T> {
    inner: T,
}

impl<T: Default> Builder<T> {
    pub fn new() -> Self {
        Self {
            inner: T::default(),
        }
    }
    
    pub fn build(self) -> Result<T, BuilderError> {
        self.validate()?;
        Ok(self.inner)
    }
    
    fn validate(&self) -> Result<(), BuilderError> {
        // Validation logic
        Ok(())
    }
}
```

### Fluent Interface

```rust
impl AgentBuilder {
    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }
    
    pub fn model(mut self, model: impl Into<String>) -> Self {
        self.model = Some(model.into());
        self
    }
}
```

---

## Component Registry

Central registry for components.

### ComponentRegistry

```rust
use llmspell_core::registry::ComponentRegistry;

pub struct ComponentRegistry {
    agents: HashMap<String, Arc<dyn Agent>>,
    tools: HashMap<String, Arc<dyn Tool>>,
    workflows: HashMap<String, Arc<dyn Workflow>>,
}

impl ComponentRegistry {
    /// Gets the global registry instance
    pub fn global() -> &'static Self;
    
    /// Registers a component
    pub async fn register<T: BaseComponent>(
        &self,
        component: T,
    ) -> Result<(), RegistryError>;
    
    /// Gets a component by ID
    pub async fn get<T: BaseComponent>(
        &self,
        id: &str,
    ) -> Result<Arc<T>, RegistryError>;
    
    /// Lists all components of a type
    pub async fn list<T: BaseComponent>(&self) -> Vec<ComponentInfo>;
}
```

### Using the Registry

```rust
use llmspell_core::registry::ComponentRegistry;

let registry = ComponentRegistry::global();

// Register components
registry.register(my_agent).await?;
registry.register(my_tool).await?;

// Retrieve components
let agent: Arc<dyn Agent> = registry.get("my_agent").await?;

// List components
let agents = registry.list::<dyn Agent>().await;
```

---

## Bridge APIs

APIs for script language integration.

### Bridge Trait

```rust
use llmspell_bridge::traits::Bridge;

#[async_trait]
pub trait Bridge: Send + Sync {
    /// Bridge name
    fn name(&self) -> &str;
    
    /// Initializes the bridge
    async fn initialize(&mut self) -> Result<(), BridgeError>;
    
    /// Executes a script
    async fn execute(
        &self,
        script: &str,
        context: BridgeContext,
    ) -> Result<BridgeResult, BridgeError>;
}
```

### Creating Custom Bridges

```rust
use llmspell_bridge::{Bridge, BridgeContext, BridgeResult};

struct CustomBridge;

#[async_trait]
impl Bridge for CustomBridge {
    fn name(&self) -> &str {
        "custom"
    }
    
    async fn initialize(&mut self) -> Result<(), BridgeError> {
        // Initialize runtime
        Ok(())
    }
    
    async fn execute(
        &self,
        script: &str,
        context: BridgeContext,
    ) -> Result<BridgeResult, BridgeError> {
        // Execute script
        Ok(BridgeResult::default())
    }
}
```

---

## Async Runtime

LLMSpell uses Tokio for async operations.

### Runtime Management

```rust
use tokio::runtime::Runtime;

// Create runtime
let runtime = Runtime::new()?;

// Execute async code
let result = runtime.block_on(async {
    agent.execute(input, &context).await
})?;

// Using handle
let handle = runtime.handle();
handle.spawn(async {
    // Background task
});
```

---

## Testing Utilities

Testing helpers for LLMSpell extensions.

### Test Fixtures

```rust
use llmspell_testing::{TestFixture, MockAgent, MockTool};

#[tokio::test]
async fn test_agent_execution() {
    let fixture = TestFixture::new();
    
    let agent = MockAgent::builder()
        .with_response("Test response")
        .build();
    
    let result = agent.execute(
        AgentInput::new("Test prompt"),
        &fixture.context(),
    ).await?;
    
    assert_eq!(result.content, "Test response");
}
```

---

## Macros

Useful macros for LLMSpell development.

### Component Registration Macro

```rust
use llmspell_macros::register_component;

#[register_component]
struct MyAgent;

// Expands to:
// impl BaseComponent for MyAgent { ... }
// impl Agent for MyAgent { ... }
```

### Error Conversion Macro

```rust
use llmspell_macros::impl_error_from;

#[impl_error_from(std::io::Error, serde_json::Error)]
pub struct CustomError(String);
```

---

## Performance Considerations

### Optimization Guidelines

1. **Use Arc for shared components** - Avoid cloning large structures
2. **Stream large responses** - Use streaming for long outputs
3. **Cache provider connections** - Reuse HTTP clients
4. **Batch operations** - Group multiple operations when possible
5. **Use async correctly** - Don't block the runtime

### Benchmarking

```rust
use criterion::{criterion_group, criterion_main, Criterion};

fn benchmark_agent(c: &mut Criterion) {
    let runtime = tokio::runtime::Runtime::new().unwrap();
    
    c.bench_function("agent_execute", |b| {
        b.to_async(&runtime).iter(|| async {
            agent.execute(input, &context).await
        });
    });
}

criterion_group!(benches, benchmark_agent);
criterion_main!(benches);
```

---

## Security Considerations

### Input Validation

```rust
use validator::Validate;

#[derive(Validate)]
struct SecureInput {
    #[validate(length(min = 1, max = 1000))]
    prompt: String,
    
    #[validate(range(min = 0.0, max = 2.0))]
    temperature: f32,
}
```

### Sandboxing

```rust
use llmspell_utils::security::Sandbox;

let sandbox = Sandbox::new()
    .with_memory_limit(100_000_000) // 100MB
    .with_time_limit(Duration::from_secs(30))
    .with_allowed_paths(vec!["/tmp"])
    .build()?;

sandbox.execute(|| {
    // Sandboxed code
})?;
```

---

## Examples

Complete working examples:

```rust
// Complete agent implementation
use llmspell_agents::{Agent, AgentBuilder, AgentInput, AgentOutput};
use llmspell_core::{BaseComponent, ExecutionContext};
use async_trait::async_trait;

pub struct CustomAgent {
    id: String,
    config: AgentConfig,
}

#[async_trait]
impl BaseComponent for CustomAgent {
    fn id(&self) -> &str {
        &self.id
    }
    
    fn component_type(&self) -> &str {
        "agent"
    }
}

#[async_trait]
impl Agent for CustomAgent {
    async fn execute(
        &self,
        input: AgentInput,
        context: &ExecutionContext,
    ) -> Result<AgentOutput, AgentError> {
        // Your implementation
        Ok(AgentOutput {
            content: format!("Processed: {}", input.prompt),
            usage: None,
            metadata: None,
        })
    }
    
    fn config(&self) -> &AgentConfig {
        &self.config
    }
}

// Usage
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let agent = CustomAgent {
        id: "custom".to_string(),
        config: AgentConfig::default(),
    };
    
    let registry = ComponentRegistry::global();
    registry.register(agent).await?;
    
    let agent = registry.get::<dyn Agent>("custom").await?;
    let result = agent.execute(
        AgentInput::new("Hello"),
        &ExecutionContext::default(),
    ).await?;
    
    println!("Response: {}", result.content);
    Ok(())
}
```

---

## See Also

- [Lua API Reference](../lua/README.md) - Lua scripting API
- [Developer Guide](../../../developer-guide/README.md) - Development guide
- [Examples](../../../../examples/EXAMPLE-INDEX.md) - Working examples
- [Architecture](../../../technical/master-architecture-vision.md) - System architecture