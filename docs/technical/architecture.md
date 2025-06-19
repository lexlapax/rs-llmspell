# Rs-LLMSpell Architecture

## Table of Contents

1. [Introduction](#introduction)
2. [Why Rs-LLMSpell Exists](#why-rs-llmspell-exists)
3. [Core Philosophy](#core-philosophy)
4. [Architecture Overview](#architecture-overview)
5. [Bridge-First Design](#bridge-first-design)
6. [Script Engine System](#script-engine-system)
7. [Type Safety & Error Handling](#type-safety--error-handling)
8. [Component Architecture](#component-architecture)
9. [Testing Architecture](#testing-architecture)
10. [Security Model](#security-model)
11. [Examples](#examples)
12. [Future Considerations](#future-considerations)

---

**Quick Links:**
- [TODO.md](../../TODO.md) - Current implementation tasks
- [TODO-DONE.md](../../TODO-DONE.md) - Completed work tracking

## Introduction

Rs-LLMSpell is a **scriptable interface for LLM interactions** that bridges multiple scripting languages (Lua, JavaScript, and potentially others) to powerful Rust-based LLM libraries. It enables developers to write "spells" - scripts that orchestrate AI agents, workflows, and tools without needing to compile Rust code, while benefiting from Rust's performance and safety in the underlying implementation.

### What is a Spell?

A spell is a script written in your language of choice (Lua, JavaScript, etc.) that controls LLMs and their associated tools through rs-llmspell's bridge layer. Think of it as magical incantations that bring AI capabilities to life through simple, expressive scripting.

```lua
-- Example spell: Research Assistant (Lua)
local researcher = llm.agent({
    provider = "anthropic",
    model = "claude-3-opus",
    tools = {"web_search", "file_write"},
    system = "You are a helpful research assistant"
})

local topic = "quantum computing breakthroughs 2025"
local research = researcher:run("Research " .. topic .. " and summarize findings")
tools.file_write("research_summary.md", research)
```

```javascript
// Example spell: Research Assistant (JavaScript)
const researcher = await llm.agent({
    provider: "anthropic",
    model: "claude-3-opus", 
    tools: ["web_search", "file_write"],
    system: "You are a helpful research assistant"
});

const topic = "quantum computing breakthroughs 2025";
const research = await researcher.run(`Research ${topic} and summarize findings`);
await tools.file_write("research_summary.md", research);
```

## Why Rs-LLMSpell Exists

### The Problem

1. **Compilation Barrier**: Working with LLMs in Rust requires compilation for every change
2. **Rapid Prototyping**: AI workflows need constant iteration and experimentation  
3. **Language Preference**: Different teams prefer different scripting languages
4. **Hot Reloading**: Production systems need to update AI behaviors without downtime
5. **Complexity**: Direct Rust LLM library usage requires deep Rust knowledge

### The Solution

Rs-LLMSpell provides a **bridge-first architecture** that:
- Exposes Rust LLM library functionality through simple scripting APIs
- Supports multiple scripting languages with the same interface
- Enables hot-reloading of AI behaviors
- Maintains type safety at bridge boundaries
- Provides security through sandboxed script execution
- Leverages existing Rust LLM libraries like Rig as the foundation

### Key Benefits

🚀 **Rapid Development**: Test AI workflows instantly without compilation  
🔄 **Hot Reloading**: Update spells in production without restarts  
🌐 **Multi-Language**: Choose Lua, JavaScript, or other languages based on your needs  
🔒 **Secure**: Sandboxed script execution with resource limits  
⚡ **Rust Performance**: Underlying Rust implementations provide native speed  
📚 **Reusable**: Build libraries of spells for common tasks  

## Core Philosophy

### 1. Bridge, Don't Build

**Fundamental Rule: If it's not in existing Rust LLM libraries, we don't implement it in rs-llmspell.**

We exclusively provide bridging interfaces to expose existing Rust LLM library functionality to scripts. We never reimplement features that exist in libraries like Rig. This strict adherence ensures:
- Automatic compatibility with upstream library updates
- Minimal maintenance burden
- Consistent behavior with direct Rust library usage
- No feature drift or divergent implementations

**What This Means in Practice:**
- ✅ Create script bindings for Rust LLM library types and functions
- ✅ Build type converters between script and Rust types
- ✅ Implement script engine infrastructure
- ❌ Never implement LLM logic, agents, tools, or workflows ourselves
- ❌ Never add features that should belong in underlying Rust libraries
- ❌ Never maintain custom versions of Rust LLM functionality

### 2. Engine-Agnostic Design

All features work identically across Lua, JavaScript, and future script engines. Scripts are portable between engines with minimal changes.

### 3. Script-Friendly APIs

We hide Rust LLM library complexity behind intuitive scripting interfaces. Complex Rust patterns become simple script calls.

### 4. Security First

All script execution happens in sandboxed environments with configurable resource limits and permission controls.

### 5. Upstream-First Development

When new features or improvements are needed for core LLM, agent, tools, or workflow functionality, we contribute them upstream to Rust LLM libraries rather than implementing them locally. This ensures:

- **Community Benefit**: Improvements benefit the entire Rust LLM ecosystem
- **Maintenance Reduction**: No custom forks or divergent implementations to maintain
- **Quality Assurance**: Changes go through established library review and testing processes
- **Long-term Compatibility**: Ensures rs-llmspell stays aligned with library evolution

### 6. Testing-First Architecture

Testing infrastructure is treated as a first-class architectural concern:

- **Reusable Test Components**: Shared fixtures, mocks, and helpers across all crates
- **Cross-Engine Validation**: Ensure identical behavior across Lua, JavaScript, and future engines
- **Bridge Contract Testing**: Standardized test suites that all bridge implementations must pass
- **Integration Testing**: End-to-end validation of script → engine → bridge → library interactions

## Architecture Overview

```
┌─────────────────────────┐
│   User Scripts          │  ← Your spells (Lua/JS/others)
├─────────────────────────┤
│   Script Engines        │  ← Multi-language execution
├─────────────────────────┤  
│   Bridge Layer          │  ← Type-safe Rust library access
├─────────────────────────┤
│   Rust LLM Libraries    │  ← Rig, other Rust LLM crates
└─────────────────────────┘
```

**Key Principle**: We bridge to Rust LLM library functionality rather than reimplementing it.

Rs-LLMSpell consists of several key crates:

```
rs-llmspell/
├── rs-llmspell-core/           # Core script engine interfaces and types
│   ├── src/
│   │   ├── lib.rs
│   │   ├── engine/             # Script engine interfaces
│   │   │   ├── mod.rs
│   │   │   ├── interface.rs    # ScriptEngine trait
│   │   │   ├── registry.rs     # Engine registry and discovery
│   │   │   ├── types.rs        # Script value type system
│   │   │   └── converter.rs    # Type conversion utilities
│   │   ├── bridge/             # Bridge interface definitions
│   │   │   ├── mod.rs
│   │   │   ├── interface.rs    # Bridge trait
│   │   │   ├── manager.rs      # Bridge lifecycle management
│   │   │   └── registry.rs     # Bridge registry
│   │   ├── types/              # Common types for bridges
│   │   │   ├── mod.rs
│   │   │   ├── script_value.rs # Universal script value type
│   │   │   ├── error.rs        # Error types
│   │   │   └── metadata.rs     # Metadata types
│   │   └── security/           # Security and sandboxing
│   │       ├── mod.rs
│   │       ├── sandbox.rs      # Script sandbox
│   │       └── limits.rs       # Resource limits
│   └── Cargo.toml
├── rs-llmspell-engines/        # Script engine implementations
│   ├── src/
│   │   ├── lib.rs
│   │   ├── lua/                # Lua engine via mlua
│   │   │   ├── mod.rs
│   │   │   ├── engine.rs       # Lua script engine
│   │   │   ├── converter.rs    # Lua type conversion
│   │   │   └── sandbox.rs      # Lua sandbox implementation
│   │   ├── javascript/         # JavaScript engine via boa or v8
│   │   │   ├── mod.rs
│   │   │   ├── engine.rs       # JS script engine
│   │   │   ├── converter.rs    # JS type conversion
│   │   │   └── sandbox.rs      # JS sandbox implementation
│   │   └── common/             # Common engine utilities
│   │       ├── mod.rs
│   │       ├── timeout.rs      # Execution timeouts
│   │       └── memory.rs       # Memory management
│   └── Cargo.toml
├── rs-llmspell-bridges/        # Bridges to Rust LLM libraries
│   ├── src/
│   │   ├── lib.rs
│   │   ├── rig/                # Rig library bridge
│   │   │   ├── mod.rs
│   │   │   ├── llm.rs          # LLM provider bridge
│   │   │   ├── agent.rs        # Agent bridge
│   │   │   ├── tools.rs        # Tools bridge
│   │   │   └── workflows.rs    # Workflow bridge
│   │   ├── util/               # Utility bridges
│   │   │   ├── mod.rs
│   │   │   ├── fs.rs           # File system operations
│   │   │   ├── http.rs         # HTTP requests
│   │   │   └── json.rs         # JSON operations
│   │   └── builtin/            # Built-in script functions
│   │       ├── mod.rs
│   │       ├── console.rs      # Console/logging functions
│   │       └── math.rs         # Math functions
│   └── Cargo.toml
├── rs-llmspell-test-common/    # Shared testing infrastructure
│   ├── src/
│   │   ├── lib.rs
│   │   ├── fixtures/           # Test fixtures and data
│   │   │   ├── mod.rs
│   │   │   ├── scripts.rs      # Cross-engine test scripts
│   │   │   ├── responses.rs    # Mock LLM responses
│   │   │   └── configs.rs      # Test configurations
│   │   ├── mocks/             # Mock implementations
│   │   │   ├── mod.rs
│   │   │   ├── engine.rs       # Mock script engine
│   │   │   ├── bridge.rs       # Mock bridge
│   │   │   └── llm_client.rs   # Mock LLM clients
│   │   ├── traits/            # Test traits and contracts
│   │   │   ├── mod.rs
│   │   │   ├── engine_tests.rs # ScriptEngine test suite
│   │   │   ├── bridge_tests.rs # Bridge test suite
│   │   │   └── integration.rs  # Integration test suite
│   │   ├── helpers/           # Test utilities
│   │   │   ├── mod.rs
│   │   │   ├── environment.rs  # Test environment setup
│   │   │   ├── assertions.rs   # Custom assertions
│   │   │   └── benchmarks.rs   # Performance testing
│   │   └── builders/          # Test data builders
│   │       ├── mod.rs
│   │       ├── script_builder.rs
│   │       └── response_builder.rs
│   └── Cargo.toml
└── rs-llmspell/                # Main binary and convenience crate
    ├── src/
    │   ├── main.rs             # CLI tool for running spells
    │   ├── lib.rs              # Re-exports for library usage
    │   ├── cli/                # Command-line interface
    │   │   ├── mod.rs
    │   │   ├── run.rs          # Run spell command
    │   │   ├── validate.rs     # Validate spell command
    │   │   └── info.rs         # Info/help commands
    │   └── config/             # Configuration management
    │       ├── mod.rs
    │       └── spell_config.rs # Spell execution config
    └── Cargo.toml
```

### Data Flow

```
User Script (Lua/JS)
    ↓ [Script API Calls]
Script Engine (rs-llmspell-engines)
    ↓ [Type Conversion]
Bridge Layer (rs-llmspell-bridges)
    ↓ [Rust Function Calls]
Rust LLM Libraries (Rig, etc.)
    ↓ [Provider APIs]
LLM Providers (OpenAI, Anthropic, etc.)
```

## Bridge-First Design

### What We Build vs What We Adapt

#### We Build (Our Value-Add)
1. **Script Engine System** - Multi-language execution environments
2. **Type Conversion Layer** - Seamless script ↔ Rust type conversions  
3. **Bridge Interfaces** - Thin wrappers exposing Rust LLM libraries to scripts
4. **Language Bindings** - Idiomatic APIs for each scripting language
5. **Script Infrastructure** - Sandboxing, resource limits, hot reload

**Important**: We ONLY create implementations, structs, and interfaces that are necessary to support scripting and bridging. No business logic, no LLM functionality, no duplicating Rust LLM library features.

#### We Bridge (From Rust LLM Libraries)
Everything else comes from Rust LLM libraries:
1. **LLM Providers** - All provider interfaces and implementations (via Rig, etc.)
2. **Agents** - Complete agent system with tools and orchestration (via Rig, etc.)
3. **Tools** - All built-in and custom tool functionality (via library ecosystem)
4. **Workflows** - All workflow types and execution logic (via libraries)
5. **Vector Operations** - Embeddings, vector search, RAG (via Rig ecosystem)
6. **Infrastructure** - HTTP clients, JSON parsing, auth (via Rust ecosystem)

#### We Never Implement
These belong in Rust LLM libraries, not rs-llmspell:
1. **New LLM Features** - Submit upstream to library authors first
2. **Custom Agents** - Use existing Rust LLM library agent systems
3. **New Tools** - Contribute to Rust LLM library tool ecosystems
4. **Workflow Types** - Extend existing library workflow engines
5. **Any Core Logic** - All intelligence lives in upstream libraries

### Bridge Implementation Strategy

Each bridge follows these principles:

1. **Thin Wrappers**: Minimal code between scripts and Rust LLM libraries
2. **Type Safety**: Handle all conversions at bridge boundaries
3. **Error Mapping**: Convert Rust errors to script-friendly formats
4. **No Business Logic**: Bridges only translate, never implement features
5. **Direct Delegation**: All functionality comes from Rust LLM libraries

Example bridge structure:

```rust
// Bridge to Rig LLM functionality - ONLY wrapping, no implementation
pub struct RigLlmBridge {
    client: rig::Client,  // From Rig library
}

impl Bridge for RigLlmBridge {
    fn methods(&self) -> Vec<MethodInfo> {
        // Define script-accessible methods that map to Rig
        vec![
            MethodInfo::new("complete", "Generate text completion"),
            MethodInfo::new("embed", "Generate embeddings"),
            // etc - all delegating to Rig
        ]
    }
}

// Example method - just type conversion and delegation
impl RigLlmBridge {
    async fn complete(&self, args: Vec<ScriptValue>) -> Result<ScriptValue, BridgeError> {
        // 1. Convert script types to Rust types
        let request = self.convert_completion_request(args)?;
        
        // 2. Call Rig directly
        let response = self.client.complete(request).await?;
        
        // 3. Convert result back to script types
        Ok(self.convert_to_script_value(response)?)
    }
}

// NEVER implement features like this:
// ❌ impl RigLlmBridge { fn custom_llm_logic(...) { ... } }
// ❌ impl RigLlmBridge { fn my_own_agent_feature(...) { ... } }
```

## Script Engine System

### Core Interfaces

#### ScriptEngine Trait

The foundation of script execution:

```rust
#[async_trait]
pub trait ScriptEngine: Send + Sync {
    /// Engine-specific error type
    type Error: std::error::Error + Send + Sync + 'static;
    
    /// Engine-specific configuration type
    type Config: Clone + Send + Sync;
    
    /// Initialize the engine with configuration
    async fn initialize(config: Self::Config) -> Result<Self, Self::Error>
    where
        Self: Sized;
    
    /// Execute a script string with parameters
    async fn execute(
        &self, 
        script: &str, 
        params: HashMap<String, ScriptValue>
    ) -> Result<ScriptValue, Self::Error>;
    
    /// Execute a script file with parameters
    async fn execute_file(
        &self, 
        path: &Path, 
        params: HashMap<String, ScriptValue>
    ) -> Result<ScriptValue, Self::Error>;
    
    /// Register a bridge with the engine
    fn register_bridge(&mut self, name: &str, bridge: Box<dyn Bridge>) -> Result<(), Self::Error>;
    
    /// Unregister a bridge
    fn unregister_bridge(&mut self, name: &str) -> Result<(), Self::Error>;
    
    /// Set memory limit for script execution
    fn set_memory_limit(&mut self, bytes: usize) -> Result<(), Self::Error>;
    
    /// Set timeout for script execution
    fn set_timeout(&mut self, timeout: Duration) -> Result<(), Self::Error>;
    
    /// Get engine name/type
    fn name(&self) -> &'static str;
    
    /// Validate script syntax without executing
    fn validate_syntax(&self, script: &str) -> Result<(), Self::Error>;
}
```

#### Agent Trait

High-level agent behavior:

```rust
#[async_trait]
pub trait Agent: Send + Sync + Clone {
    type Error: std::error::Error + Send + Sync + 'static;
    type Provider: Provider;
    type Tool: Tool;
    
    /// Create a new agent with a provider
    fn new(provider: Self::Provider) -> Self;
    
    /// Configure the agent's behavior
    fn with_system_prompt(self, prompt: impl Into<String>) -> Self;
    fn with_tools(self, tools: Vec<Self::Tool>) -> Self;
    fn with_config(self, config: AgentConfig) -> Self;
    
    /// Execute a single request
    async fn run(&self, input: &str) -> Result<AgentResponse, Self::Error>;
    
    /// Execute with conversation context
    async fn run_with_context(&self, input: &str, context: &ConversationContext) 
        -> Result<AgentResponse, Self::Error>;
    
    /// Stream responses in real-time
    fn run_stream(&self, input: &str) 
        -> impl Stream<Item = Result<AgentChunk, Self::Error>> + Send;
    
    /// Get available tools
    fn tools(&self) -> &[Self::Tool];
    
    /// Validate agent configuration
    fn validate(&self) -> Result<(), Self::Error>;
}
```

#### Tool Trait

Tool interface for extending agent capabilities:

```rust
#[async_trait]
pub trait Tool: Send + Sync + Clone {
    type Error: std::error::Error + Send + Sync + 'static;
    type Input: serde::de::DeserializeOwned + Send + Sync;
    type Output: serde::Serialize + Send + Sync;
    
    /// Tool metadata
    fn name(&self) -> &str;
    fn description(&self) -> &str;
    fn parameters_schema(&self) -> serde_json::Value;
    
    /// Execute the tool
    async fn execute(&self, input: Self::Input) -> Result<Self::Output, Self::Error>;
    
    /// Validate input without executing
    fn validate_input(&self, input: &serde_json::Value) -> Result<(), Self::Error>;
    
    /// Check if tool is available (e.g., API keys present)
    async fn health_check(&self) -> Result<(), Self::Error>;
}
```

#### Workflow Trait

Orchestration of multiple agents and tools:

```rust
#[async_trait]
pub trait Workflow: Send + Sync {
    type Error: std::error::Error + Send + Sync + 'static;
    type Step: WorkflowStep;
    type Context: WorkflowContext;
    
    /// Add a step to the workflow
    fn add_step(self, step: Self::Step) -> Self;
    
    /// Execute the entire workflow
    async fn run(&self, context: Self::Context) -> Result<WorkflowResult, Self::Error>;
    
    /// Execute with progress reporting
    async fn run_with_progress(&self, context: Self::Context) 
        -> impl Stream<Item = Result<WorkflowProgress, Self::Error>> + Send;
    
    /// Validate workflow configuration
    fn validate(&self) -> Result<(), Self::Error>;
    
    /// Get workflow metadata
    fn metadata(&self) -> WorkflowMetadata;
}
```

### Type System Design

#### Core Types

```rust
// Message types for LLM communication
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub role: Role,
    pub content: Content,
    pub metadata: Option<MessageMetadata>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Role {
    System,
    User,
    Assistant,
    Tool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Content {
    Text(String),
    Multimodal(Vec<ContentPart>),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ContentPart {
    Text(String),
    Image { 
        url: Option<String>, 
        data: Option<Vec<u8>>, 
        mime_type: String 
    },
}

// Request/Response types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletionRequest {
    pub messages: Vec<Message>,
    pub model: String,
    pub temperature: Option<f32>,
    pub max_tokens: Option<u32>,
    pub stop_sequences: Option<Vec<String>>,
    pub tools: Option<Vec<ToolDefinition>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletionResponse {
    pub message: Message,
    pub usage: TokenUsage,
    pub model: String,
    pub finish_reason: FinishReason,
    pub metadata: ResponseMetadata,
}

// Error types with rich context
#[derive(Debug, thiserror::Error)]
pub enum LlmError {
    #[error("Provider error: {0}")]
    Provider(#[from] Box<dyn std::error::Error + Send + Sync>),
    
    #[error("Configuration error: {message}")]
    Configuration { message: String },
    
    #[error("Validation error: {field}: {message}")]
    Validation { field: String, message: String },
    
    #[error("Network error: {0}")]
    Network(#[from] reqwest::Error),
    
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    
    #[error("Timeout after {duration:?}")]
    Timeout { duration: std::time::Duration },
    
    #[error("Rate limit exceeded: retry after {retry_after:?}")]
    RateLimit { retry_after: Option<std::time::Duration> },
}
```

## Async System Design

### Async Patterns

#### 1. Provider Operations

All provider operations are async and cancellable:

```rust
impl Provider for OpenAiProvider {
    async fn complete(&self, request: CompletionRequest) -> Result<CompletionResponse, Self::Error> {
        // Built-in timeout and cancellation
        let response = tokio::time::timeout(
            self.config.timeout,
            self.client.complete(request)
        ).await??;
        
        Ok(response)
    }
}
```

#### 2. Concurrent Agent Execution

Agents can be executed concurrently with proper error handling:

```rust
pub async fn run_agents_concurrently(
    agents: Vec<Agent>,
    inputs: Vec<String>,
) -> Result<Vec<AgentResponse>, LlmError> {
    let tasks: Vec<_> = agents
        .into_iter()
        .zip(inputs)
        .map(|(agent, input)| {
            tokio::spawn(async move {
                agent.run(&input).await
            })
        })
        .collect();
    
    let results = futures::future::try_join_all(tasks).await?;
    results.into_iter().collect()
}
```

#### 3. Streaming Responses

Real-time streaming with backpressure:

```rust
#[async_trait]
impl Agent for StreamingAgent {
    fn run_stream(&self, input: &str) 
        -> impl Stream<Item = Result<AgentChunk, Self::Error>> + Send {
        async_stream::try_stream! {
            let mut stream = self.provider.complete_stream(request).await?;
            
            while let Some(chunk) = stream.next().await {
                let chunk = chunk?;
                yield AgentChunk::from(chunk);
            }
        }
    }
}
```

#### 4. Workflow Orchestration

Complex async orchestration patterns:

```rust
impl Workflow for SequentialWorkflow {
    async fn run(&self, mut context: WorkflowContext) -> Result<WorkflowResult, Self::Error> {
        let mut results = Vec::new();
        
        for step in &self.steps {
            // Each step can access previous results
            let step_input = self.prepare_step_input(&context, &results)?;
            
            // Execute step with timeout and error handling
            let step_result = tokio::time::timeout(
                step.timeout(),
                step.execute(step_input)
            ).await??;
            
            // Update context for next step
            context.update_with_result(&step_result)?;
            results.push(step_result);
        }
        
        Ok(WorkflowResult { results, context })
    }
}
```

### Error Handling Patterns

#### 1. Hierarchical Error Types

```rust
#[derive(Debug, thiserror::Error)]
pub enum WorkflowError {
    #[error("Step {step_name} failed: {source}")]
    StepFailed {
        step_name: String,
        #[source]
        source: Box<dyn std::error::Error + Send + Sync>,
    },
    
    #[error("Agent error: {0}")]
    Agent(#[from] AgentError),
    
    #[error("Provider error: {0}")]
    Provider(#[from] LlmError),
    
    #[error("Configuration error: {0}")]
    Configuration(String),
}
```

#### 2. Retry Policies

```rust
pub struct RetryPolicy {
    pub max_attempts: u32,
    pub base_delay: Duration,
    pub max_delay: Duration,
    pub backoff: BackoffStrategy,
    pub retryable_errors: fn(&dyn std::error::Error) -> bool,
}

impl RetryPolicy {
    pub async fn execute<F, T, E>(&self, operation: F) -> Result<T, E>
    where
        F: Fn() -> Pin<Box<dyn Future<Output = Result<T, E>> + Send>>,
        E: std::error::Error,
    {
        let mut attempt = 0;
        let mut delay = self.base_delay;
        
        loop {
            attempt += 1;
            
            match operation().await {
                Ok(result) => return Ok(result),
                Err(error) if attempt >= self.max_attempts => return Err(error),
                Err(error) if !(self.retryable_errors)(&error) => return Err(error),
                Err(_) => {
                    tokio::time::sleep(delay).await;
                    delay = std::cmp::min(delay * 2, self.max_delay);
                }
            }
        }
    }
}
```

## Type Safety & Error Handling

### Compile-Time Guarantees

#### 1. Provider Configuration

```rust
// Configuration is validated at compile time
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAiConfig {
    pub api_key: SecretString,  // Type-safe secret handling
    pub base_url: Option<url::Url>,  // Validated URL type
    pub timeout: Duration,
    pub max_retries: u32,
}

impl OpenAiConfig {
    pub fn new(api_key: impl Into<SecretString>) -> Self {
        Self {
            api_key: api_key.into(),
            base_url: None,
            timeout: Duration::from_secs(30),
            max_retries: 3,
        }
    }
    
    // Builder pattern with compile-time validation
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }
}
```

#### 2. Tool Parameter Validation

```rust
// Tools use strong typing for parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebSearchTool {
    client: reqwest::Client,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WebSearchInput {
    pub query: String,
    #[serde(default = "default_max_results")]
    pub max_results: u32,
    pub filter: Option<SearchFilter>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WebSearchOutput {
    pub results: Vec<SearchResult>,
    pub query_metadata: QueryMetadata,
}

#[async_trait]
impl Tool for WebSearchTool {
    type Input = WebSearchInput;
    type Output = WebSearchOutput;
    type Error = WebSearchError;
    
    async fn execute(&self, input: Self::Input) -> Result<Self::Output, Self::Error> {
        // Input is already validated at compile time
        self.search_web(input).await
    }
}
```

#### 3. Workflow Type Safety

```rust
// Workflows enforce type safety between steps
pub struct TypedWorkflow<Input, Output> {
    steps: Vec<Box<dyn WorkflowStep<Input, Output>>>,
    _phantom: PhantomData<(Input, Output)>,
}

impl<Input, Output> TypedWorkflow<Input, Output> 
where 
    Input: Clone + Send + Sync + 'static,
    Output: Send + Sync + 'static,
{
    pub fn new() -> Self {
        Self {
            steps: Vec::new(),
            _phantom: PhantomData,
        }
    }
    
    // Type-safe step addition
    pub fn add_step<S>(mut self, step: S) -> Self 
    where 
        S: WorkflowStep<Input, Output> + 'static
    {
        self.steps.push(Box::new(step));
        self
    }
}
```

### Runtime Safety

#### 1. Resource Management

```rust
// Automatic cleanup with RAII
pub struct ResourceGuard<T> {
    resource: Option<T>,
    cleanup: Box<dyn FnOnce(T) + Send>,
}

impl<T> ResourceGuard<T> {
    pub fn new(resource: T, cleanup: impl FnOnce(T) + Send + 'static) -> Self {
        Self {
            resource: Some(resource),
            cleanup: Box::new(cleanup),
        }
    }
}

impl<T> Drop for ResourceGuard<T> {
    fn drop(&mut self) {
        if let Some(resource) = self.resource.take() {
            (self.cleanup)(resource);
        }
    }
}
```

#### 2. Memory Limits

```rust
// Memory-bounded operations
pub struct BoundedString {
    inner: String,
    max_size: usize,
}

impl BoundedString {
    pub fn new(max_size: usize) -> Self {
        Self {
            inner: String::new(),
            max_size,
        }
    }
    
    pub fn push_str(&mut self, s: &str) -> Result<(), CapacityError> {
        if self.inner.len() + s.len() > self.max_size {
            return Err(CapacityError::ExceedsLimit);
        }
        self.inner.push_str(s);
        Ok(())
    }
}
```

## Component Architecture

### Agent System

```rust
// Basic agent implementation
pub struct BasicAgent<P: Provider> {
    provider: P,
    system_prompt: Option<String>,
    config: AgentConfig,
}

impl<P: Provider> BasicAgent<P> {
    pub fn new(provider: P) -> Self {
        Self {
            provider,
            system_prompt: None,
            config: AgentConfig::default(),
        }
    }
}

#[async_trait]
impl<P: Provider> Agent for BasicAgent<P> {
    type Provider = P;
    type Error = AgentError;
    
    async fn run(&self, input: &str) -> Result<AgentResponse, Self::Error> {
        let mut messages = Vec::new();
        
        if let Some(system) = &self.system_prompt {
            messages.push(Message::system(system));
        }
        
        messages.push(Message::user(input));
        
        let request = CompletionRequest {
            messages,
            model: self.config.model.clone(),
            temperature: self.config.temperature,
            max_tokens: self.config.max_tokens,
            stop_sequences: None,
            tools: None,
        };
        
        let response = self.provider.complete(request).await?;
        
        Ok(AgentResponse {
            content: response.message.content,
            usage: response.usage,
            metadata: response.metadata.into(),
        })
    }
}
```

### Tool System

```rust
// Tool registry with type safety
pub struct ToolRegistry {
    tools: HashMap<String, Box<dyn ErasedTool>>,
}

// Type-erased tool trait for storage
trait ErasedTool: Send + Sync {
    fn name(&self) -> &str;
    fn description(&self) -> &str;
    fn parameters_schema(&self) -> serde_json::Value;
    
    fn execute_json(&self, input: serde_json::Value) 
        -> Pin<Box<dyn Future<Output = Result<serde_json::Value, Box<dyn std::error::Error + Send + Sync>>> + Send>>;
}

impl<T: Tool + 'static> ErasedTool for T {
    fn name(&self) -> &str {
        Tool::name(self)
    }
    
    fn description(&self) -> &str {
        Tool::description(self)
    }
    
    fn parameters_schema(&self) -> serde_json::Value {
        Tool::parameters_schema(self)
    }
    
    fn execute_json(&self, input: serde_json::Value) 
        -> Pin<Box<dyn Future<Output = Result<serde_json::Value, Box<dyn std::error::Error + Send + Sync>>> + Send>> {
        Box::pin(async move {
            let typed_input: T::Input = serde_json::from_value(input)?;
            let output = self.execute(typed_input).await?;
            let json_output = serde_json::to_value(output)?;
            Ok(json_output)
        })
    }
}
```

## Testing Architecture

### Core Testing Philosophy

**Fundamental Principle: Testing infrastructure must be as reusable and composable as the production architecture.**

In a multi-layered system like rs-llmspell, testing components should be shared across:
- Multiple script engines (Lua, JavaScript, future engines)
- Multiple bridges (Rig, direct providers, custom bridges)
- Integration scenarios (end-to-end workflows)
- Performance and security validation

### Testing Infrastructure Components

#### 1. Test Traits and Contracts

```rust
// Core testing traits that define behavioral contracts
pub trait ScriptEngineTestSuite {
    type Engine: ScriptEngine;
    
    /// Standard test suite that any script engine must pass
    fn run_standard_tests(&self, engine: &Self::Engine) -> TestResult;
    
    /// Language-specific tests (syntax, features)
    fn run_language_tests(&self, engine: &Self::Engine) -> TestResult;
    
    /// Performance benchmarks
    fn run_performance_tests(&self, engine: &Self::Engine) -> BenchmarkResult;
    
    /// Security and sandboxing tests
    fn run_security_tests(&self, engine: &Self::Engine) -> SecurityTestResult;
}

pub trait BridgeTestSuite {
    type Bridge: Bridge;
    
    /// Standard bridge behavior tests
    fn run_standard_tests(&self, bridge: &Self::Bridge) -> TestResult;
    
    /// Type conversion validation
    fn run_type_safety_tests(&self, bridge: &Self::Bridge) -> TestResult;
    
    /// Error handling verification
    fn run_error_handling_tests(&self, bridge: &Self::Bridge) -> TestResult;
    
    /// Method signature validation
    fn run_method_tests(&self, bridge: &Self::Bridge) -> TestResult;
}

pub trait IntegrationTestSuite {
    /// End-to-end workflow tests
    fn run_e2e_tests(&self) -> TestResult;
    
    /// Cross-engine compatibility tests
    fn run_cross_engine_tests(&self) -> TestResult;
    
    /// Resource limit and timeout tests
    fn run_resource_tests(&self) -> TestResult;
}
```

#### 2. Shared Test Fixtures

```rust
// Standard test fixtures used across all components
pub struct TestFixtures {
    pub scripts: ScriptFixtures,
    pub mock_responses: MockResponseFixtures,
    pub test_data: TestDataFixtures,
    pub configs: ConfigFixtures,
}

pub struct ScriptFixtures {
    // Standard test scripts that work across engines
    pub basic_hello_world: ScriptSet,
    pub type_conversion_tests: ScriptSet,
    pub error_handling_tests: ScriptSet,
    pub async_operation_tests: ScriptSet,
    pub bridge_interaction_tests: ScriptSet,
    pub performance_benchmarks: ScriptSet,
    pub security_violation_tests: ScriptSet,
}

pub struct ScriptSet {
    pub lua: &'static str,
    pub javascript: &'static str,
    pub description: &'static str,
    pub expected_output: ExpectedOutput,
}

// Example fixture
impl ScriptFixtures {
    pub fn basic_llm_interaction() -> ScriptSet {
        ScriptSet {
            lua: r#"
                local response = llm.complete({
                    model = "test-model",
                    messages = {{role = "user", content = "Hello"}}
                })
                return response.content
            "#,
            javascript: r#"
                const response = await llm.complete({
                    model: "test-model",
                    messages: [{role: "user", content: "Hello"}]
                });
                return response.content;
            "#,
            description: "Basic LLM completion request",
            expected_output: ExpectedOutput::String("Hello from test model".into()),
        }
    }
}
```

#### 3. Mock Implementations

```rust
// Reusable mock implementations for testing

/// Mock script engine for testing bridges
pub struct MockScriptEngine {
    pub responses: HashMap<String, ScriptValue>,
    pub execution_log: Vec<ExecutionEvent>,
    pub should_fail: bool,
}

impl ScriptEngine for MockScriptEngine {
    // Implementation that records calls and returns predefined responses
}

/// Mock bridge for testing script engines
pub struct MockBridge {
    pub method_responses: HashMap<String, ScriptValue>,
    pub call_log: Vec<BridgeCall>,
    pub should_fail_method: Option<String>,
}

impl Bridge for MockBridge {
    // Implementation that logs calls and returns test data
}

/// Mock LLM library for testing bridges
pub struct MockRigClient {
    pub completion_responses: VecDeque<CompletionResponse>,
    pub embedding_responses: VecDeque<EmbeddingResponse>,
    pub call_log: Vec<ApiCall>,
    pub latency_ms: Option<u64>,
}

// Mock implementations are configured through builders
impl MockRigClient {
    pub fn new() -> MockRigClientBuilder {
        MockRigClientBuilder::default()
    }
}

pub struct MockRigClientBuilder {
    responses: Vec<CompletionResponse>,
    latency: Option<u64>,
    should_fail: bool,
}

impl MockRigClientBuilder {
    pub fn with_completion(mut self, response: CompletionResponse) -> Self {
        self.responses.push(response);
        self
    }
    
    pub fn with_latency(mut self, ms: u64) -> Self {
        self.latency = Some(ms);
        self
    }
    
    pub fn should_fail(mut self) -> Self {
        self.should_fail = true;
        self
    }
    
    pub fn build(self) -> MockRigClient {
        // Build the mock with configured behavior
    }
}
```

#### 4. Test Helpers and Utilities

```rust
// Reusable test utilities across all components

pub struct TestEnvironment {
    pub temp_dir: TempDir,
    pub config: TestConfig,
    pub cleanup_hooks: Vec<Box<dyn FnOnce()>>,
}

impl TestEnvironment {
    pub fn new() -> Self {
        // Set up isolated test environment
    }
    
    pub fn create_script_file(&self, name: &str, content: &str) -> PathBuf {
        // Create temporary script file
    }
    
    pub fn create_test_engine(&self, engine_type: EngineType) -> Box<dyn ScriptEngine> {
        // Create configured test engine
    }
    
    pub fn create_mock_bridge(&self, bridge_name: &str) -> Box<dyn Bridge> {
        // Create configured mock bridge
    }
}

pub struct TestHelpers;

impl TestHelpers {
    /// Assert that script produces expected output across all engines
    pub async fn assert_script_output_cross_engine(
        script_set: &ScriptSet,
        engines: &[Box<dyn ScriptEngine>],
    ) -> TestResult {
        for engine in engines {
            let script = match engine.name() {
                "lua" => script_set.lua,
                "javascript" => script_set.javascript,
                _ => panic!("Unsupported engine type"),
            };
            
            let result = engine.execute(script, HashMap::new()).await?;
            assert_eq!(result, script_set.expected_output.to_script_value());
        }
        Ok(())
    }
    
    /// Validate bridge type conversion round-trip
    pub fn assert_type_conversion_round_trip<T>(
        bridge: &dyn Bridge,
        original: T,
    ) -> TestResult 
    where 
        T: Serialize + DeserializeOwned + PartialEq + Clone
    {
        let script_value = bridge.to_script_value(&original)?;
        let converted_back: T = bridge.from_script_value(&script_value)?;
        assert_eq!(original, converted_back);
        Ok(())
    }
    
    /// Run performance benchmark across engines
    pub async fn benchmark_script_execution(
        script_set: &ScriptSet,
        engines: &[Box<dyn ScriptEngine>],
        iterations: usize,
    ) -> BenchmarkResult {
        // Run performance comparison across engines
    }
}
```

### Component-Specific Testing Strategies

#### Script Engine Testing

```rust
// Standard test suite for any script engine implementation
pub struct StandardScriptEngineTests;

impl StandardScriptEngineTests {
    pub async fn run_all<E: ScriptEngine>(engine: &E) -> TestResult {
        // Core functionality tests
        Self::test_basic_execution(engine).await?;
        Self::test_type_conversions(engine).await?;
        Self::test_bridge_interactions(engine).await?;
        Self::test_error_handling(engine).await?;
        Self::test_async_operations(engine).await?;
        
        // Security tests
        Self::test_memory_limits(engine).await?;
        Self::test_timeout_enforcement(engine).await?;
        Self::test_sandbox_isolation(engine).await?;
        
        // Performance tests
        Self::test_execution_performance(engine).await?;
        Self::test_memory_usage(engine).await?;
        
        Ok(())
    }
    
    async fn test_basic_execution<E: ScriptEngine>(engine: &E) -> TestResult {
        let fixtures = TestFixtures::load();
        
        for script_set in &fixtures.scripts.basic_tests {
            let script = script_set.get_for_engine(engine.name());
            let result = engine.execute(script, HashMap::new()).await?;
            
            assert_eq!(result, script_set.expected_output.to_script_value());
        }
        
        Ok(())
    }
    
    async fn test_bridge_interactions<E: ScriptEngine>(engine: &E) -> TestResult {
        // Test with mock bridges
        let mock_bridge = MockBridge::new()
            .with_method("test_method", ScriptValue::String("test_response".into()))
            .build();
            
        engine.register_bridge("test", Box::new(mock_bridge))?;
        
        let script = script_set.get_for_engine(engine.name());
        let result = engine.execute(script, HashMap::new()).await?;
        
        // Verify bridge was called correctly
        assert!(mock_bridge.was_method_called("test_method"));
        
        Ok(())
    }
}
```

#### Bridge Testing

```rust
// Standard test suite for any bridge implementation
pub struct StandardBridgeTests;

impl StandardBridgeTests {
    pub async fn run_all<B: Bridge>(bridge: &B) -> TestResult {
        // Core bridge functionality
        Self::test_method_execution(bridge).await?;
        Self::test_type_conversions(bridge).await?;
        Self::test_error_handling(bridge).await?;
        
        // Integration with mock engines
        Self::test_engine_integration(bridge).await?;
        
        // Performance
        Self::test_method_performance(bridge).await?;
        
        Ok(())
    }
    
    async fn test_method_execution<B: Bridge>(bridge: &B) -> TestResult {
        for method in bridge.methods() {
            let test_args = TestFixtures::args_for_method(&method.name);
            let result = bridge.execute_method(&method.name, test_args).await?;
            
            // Validate result type matches method signature
            Self::validate_result_type(&method, &result)?;
        }
        
        Ok(())
    }
    
    async fn test_engine_integration<B: Bridge>(bridge: &B) -> TestResult {
        let mock_engine = MockScriptEngine::new();
        
        // Register bridge with mock engine
        mock_engine.register_bridge("test", Box::new(bridge.clone()))?;
        
        // Execute test scripts that use the bridge
        let test_scripts = TestFixtures::load().scripts.bridge_interaction_tests;
        
        for script_set in test_scripts {
            let result = mock_engine.execute(
                script_set.get_for_engine("mock"), 
                HashMap::new()
            ).await?;
            
            assert_eq!(result, script_set.expected_output.to_script_value());
        }
        
        Ok(())
    }
}
```

#### Integration Testing

```rust
// End-to-end integration test framework
pub struct IntegrationTestFramework {
    engines: Vec<Box<dyn ScriptEngine>>,
    bridges: Vec<Box<dyn Bridge>>,
    mock_clients: HashMap<String, Box<dyn MockClient>>,
}

impl IntegrationTestFramework {
    pub fn new() -> Self {
        Self {
            engines: vec![
                Box::new(LuaScriptEngine::new_for_testing()),
                Box::new(JavaScriptScriptEngine::new_for_testing()),
            ],
            bridges: vec![
                Box::new(MockRigBridge::new()),
                Box::new(MockUtilBridge::new()),
            ],
            mock_clients: HashMap::new(),
        }
    }
    
    pub async fn run_cross_engine_compatibility_tests(&self) -> TestResult {
        let test_scenarios = TestFixtures::load().integration_scenarios;
        
        for scenario in test_scenarios {
            for engine in &self.engines {
                // Register all bridges
                for bridge in &self.bridges {
                    engine.register_bridge(&bridge.name(), bridge.clone())?;
                }
                
                // Execute scenario script
                let script = scenario.get_for_engine(engine.name());
                let result = engine.execute(script, scenario.params.clone()).await?;
                
                // Validate result
                assert_eq!(result, scenario.expected_output.to_script_value());
                
                // Validate side effects (API calls, file operations, etc.)
                scenario.validate_side_effects(&self.mock_clients)?;
            }
        }
        
        Ok(())
    }
    
    pub async fn run_performance_comparison(&self) -> BenchmarkResult {
        let benchmark_scripts = TestFixtures::load().performance_benchmarks;
        let mut results = BenchmarkResult::new();
        
        for script_set in benchmark_scripts {
            for engine in &self.engines {
                let start = Instant::now();
                
                for _ in 0..1000 {
                    engine.execute(
                        script_set.get_for_engine(engine.name()),
                        HashMap::new()
                    ).await?;
                }
                
                let duration = start.elapsed();
                results.add_measurement(engine.name(), &script_set.name, duration);
            }
        }
        
        Ok(results)
    }
}
```

### Test Organization Structure

```
rs-llmspell/
├── rs-llmspell-test-common/    # Shared testing infrastructure
│   ├── src/
│   │   ├── lib.rs
│   │   ├── fixtures/           # Test fixtures and data
│   │   │   ├── mod.rs
│   │   │   ├── scripts.rs      # Cross-engine test scripts
│   │   │   ├── responses.rs    # Mock LLM responses
│   │   │   └── configs.rs      # Test configurations
│   │   ├── mocks/             # Mock implementations
│   │   │   ├── mod.rs
│   │   │   ├── engine.rs       # Mock script engine
│   │   │   ├── bridge.rs       # Mock bridge
│   │   │   └── llm_client.rs   # Mock LLM clients
│   │   ├── traits/            # Test traits and contracts
│   │   │   ├── mod.rs
│   │   │   ├── engine_tests.rs # ScriptEngine test suite
│   │   │   ├── bridge_tests.rs # Bridge test suite
│   │   │   └── integration.rs  # Integration test suite
│   │   ├── helpers/           # Test utilities
│   │   │   ├── mod.rs
│   │   │   ├── environment.rs  # Test environment setup
│   │   │   ├── assertions.rs   # Custom assertions
│   │   │   └── benchmarks.rs   # Performance testing
│   │   └── builders/          # Test data builders
│   │       ├── mod.rs
│   │       ├── script_builder.rs
│   │       └── response_builder.rs
│   └── Cargo.toml
├── rs-llmspell-engines/
│   └── tests/                 # Uses rs-llmspell-test-common
│       ├── lua_engine_tests.rs
│       ├── js_engine_tests.rs
│       └── cross_engine_tests.rs
├── rs-llmspell-bridges/
│   └── tests/                 # Uses rs-llmspell-test-common
│       ├── rig_bridge_tests.rs
│       ├── util_bridge_tests.rs
│       └── bridge_integration_tests.rs
└── rs-llmspell/
    └── tests/                 # Integration tests
        ├── e2e_tests.rs
        ├── performance_tests.rs
        └── security_tests.rs
```

### Test Data Management

```rust
// Centralized test data that's versioned and reusable

pub struct TestDataManager {
    fixtures: TestFixtures,
    version: semver::Version,
}

impl TestDataManager {
    pub fn load_fixtures(version: &str) -> Result<TestFixtures, TestError> {
        // Load versioned test fixtures
        // This allows testing against different fixture versions
        // to ensure backward compatibility
    }
    
    pub fn generate_cross_engine_scripts() -> Vec<ScriptSet> {
        // Generate equivalent scripts for all supported engines
        // Ensures feature parity testing
    }
    
    pub fn create_mock_responses() -> MockResponseSet {
        // Create realistic mock responses for different scenarios
        // Success cases, error cases, edge cases, performance cases
    }
}

// Test fixtures are generated programmatically to ensure consistency
#[derive(Builder)]
pub struct ScriptTestCase {
    pub name: String,
    pub description: String,
    pub lua_script: String,
    pub javascript_script: String,
    pub input_params: HashMap<String, ScriptValue>,
    pub expected_output: ScriptValue,
    pub expected_side_effects: Vec<SideEffect>,
    pub performance_threshold: Option<Duration>,
}
```

This testing architecture ensures that:

1. **All components are thoroughly tested** with standardized test suites
2. **Tests are reusable** across different implementations (engines, bridges)
3. **Integration scenarios** are validated across all engine combinations
4. **Performance** is consistently measured and compared
5. **Regression testing** is automated through shared test fixtures
6. **Mock implementations** provide consistent testing environments

## Security Model

### 1. Memory Safety

Rust's ownership system provides automatic memory safety:
- No buffer overflows
- No use-after-free
- No data races in safe code
- Automatic cleanup of resources

### 2. Type Safety

Strong typing prevents many classes of errors:
- API parameter validation at compile time
- Impossible to mix incompatible types
- Exhaustive pattern matching
- No null pointer dereferences

### 3. Secure Defaults

```rust
// Secure configuration by default
#[derive(Debug, Clone)]
pub struct SecurityConfig {
    pub max_request_size: usize,          // 1MB default
    pub max_response_size: usize,         // 10MB default
    pub request_timeout: Duration,        // 30s default
    pub max_concurrent_requests: usize,   // 10 default
    pub allowed_domains: Option<Vec<String>>, // None = all allowed
    pub require_tls: bool,                // true default
}

impl Default for SecurityConfig {
    fn default() -> Self {
        Self {
            max_request_size: 1024 * 1024,      // 1MB
            max_response_size: 10 * 1024 * 1024, // 10MB
            request_timeout: Duration::from_secs(30),
            max_concurrent_requests: 10,
            allowed_domains: None,
            require_tls: true,
        }
    }
}
```

### 4. Secret Management

```rust
use secrecy::{Secret, SecretString};

// Secrets are never accidentally logged or printed
#[derive(Debug, Clone)]
pub struct ProviderConfig {
    pub api_key: SecretString,  // Automatically protected
    pub endpoint: String,
}

impl ProviderConfig {
    pub fn new(api_key: impl Into<SecretString>) -> Self {
        Self {
            api_key: api_key.into(),
            endpoint: "https://api.provider.com".to_string(),
        }
    }
}
```

## Examples

### Basic LLM Interaction

**Lua:**
```lua
-- basic-llm.lua
local response = llm.complete({
    model = "gpt-4",
    messages = {
        {role = "user", content = "Explain quantum computing in simple terms"}
    }
})

console.log("Response: " .. response.content)
console.log("Tokens used: " .. response.usage.total_tokens)
```

**JavaScript:**
```javascript
// basic-llm.js
const response = await llm.complete({
    model: "gpt-4",
    messages: [
        {role: "user", content: "Explain quantum computing in simple terms"}
    ]
});

console.log(`Response: ${response.content}`);
console.log(`Tokens used: ${response.usage.total_tokens}`);
```

**Running the spell:**
```bash
rs-llmspell run basic-llm.lua
rs-llmspell run basic-llm.js
```

### Agent with Tools

**Lua:**
```lua
-- research-agent.lua
local researcher = llm.agent({
    model = "claude-3-opus",
    tools = {"web_search", "file_write"},
    system = "You are a helpful research assistant"
})

local topic = params.topic or "quantum computing breakthroughs 2025"
local research = researcher:run("Research " .. topic .. " and summarize key findings")

-- Save results
tools.file_write("research/" .. topic:gsub(" ", "_") .. ".md", research)
console.log("Research completed and saved!")

return {
    topic = topic,
    summary = research:sub(1, 200) .. "...",
    status = "completed"
}
```

**JavaScript:**
```javascript
// research-agent.js
const researcher = await llm.agent({
    model: "claude-3-opus",
    tools: ["web_search", "file_write"],
    system: "You are a helpful research assistant"
});

const topic = params.topic || "quantum computing breakthroughs 2025";
const research = await researcher.run(`Research ${topic} and summarize key findings`);

// Save results
await tools.file_write(`research/${topic.replace(/\s+/g, "_")}.md`, research);
console.log("Research completed and saved!");

return {
    topic: topic,
    summary: research.substring(0, 200) + "...",
    status: "completed"
};
```

**Running with parameters:**
```bash
rs-llmspell run research-agent.lua --param topic="AI safety research"
rs-llmspell run research-agent.js --param topic="AI safety research"
```

### Sequential Workflow

**Lua:**
```lua
-- research-workflow.lua
local workflow = workflow.sequential({
    name = "comprehensive_research"
})

-- Step 1: Initial research
workflow:add_step({
    name = "research",
    agent = llm.agent({
        model = "gpt-4",
        tools = {"web_search"},
        system = "You are an expert researcher"
    }),
    prompt = "Research {{topic}} and gather comprehensive information"
})

-- Step 2: Summarize findings
workflow:add_step({
    name = "summarize", 
    agent = llm.agent({
        model = "claude-3-opus",
        system = "You are an expert at creating clear, concise summaries"
    }),
    prompt = "Summarize these research findings: {{research.output}}"
})

-- Step 3: Create final report
workflow:add_step({
    name = "report",
    agent = llm.agent({
        model = "gpt-4",
        tools = {"file_write"},
        system = "You are a technical writer"
    }),
    prompt = "Create a structured report from this summary: {{summarize.output}}"
})

-- Execute workflow
local result = workflow:run({
    topic = params.topic or "quantum computing applications"
})

console.log("Workflow completed:")
for step_name, step_result in pairs(result.steps) do
    console.log("- " .. step_name .. ": " .. step_result.status)
end

return result
```

**JavaScript:**
```javascript
// research-workflow.js  
const workflow = workflow.sequential({
    name: "comprehensive_research"
});

// Step 1: Initial research
workflow.addStep({
    name: "research",
    agent: await llm.agent({
        model: "gpt-4", 
        tools: ["web_search"],
        system: "You are an expert researcher"
    }),
    prompt: "Research {{topic}} and gather comprehensive information"
});

// Step 2: Summarize findings
workflow.addStep({
    name: "summarize",
    agent: await llm.agent({
        model: "claude-3-opus",
        system: "You are an expert at creating clear, concise summaries"  
    }),
    prompt: "Summarize these research findings: {{research.output}}"
});

// Step 3: Create final report
workflow.addStep({
    name: "report",
    agent: await llm.agent({
        model: "gpt-4",
        tools: ["file_write"],
        system: "You are a technical writer"
    }),
    prompt: "Create a structured report from this summary: {{summarize.output}}"
});

// Execute workflow
const result = await workflow.run({
    topic: params.topic || "quantum computing applications"
});

console.log("Workflow completed:");
for (const [stepName, stepResult] of Object.entries(result.steps)) {
    console.log(`- ${stepName}: ${stepResult.status}`);
}

return result;
```

### Error Handling and Retries

**Lua:**
```lua
-- robust-agent.lua
local function robust_llm_call(prompt, max_retries)
    max_retries = max_retries or 3
    local attempt = 0
    
    while attempt < max_retries do
        attempt = attempt + 1
        
        local success, result = pcall(function()
            return llm.complete({
                model = "gpt-4",
                messages = {{role = "user", content = prompt}}
            })
        end)
        
        if success then
            return result
        else
            console.log("Attempt " .. attempt .. " failed: " .. tostring(result))
            
            if attempt < max_retries then
                -- Exponential backoff
                util.sleep(math.pow(2, attempt) * 1000)
            end
        end
    end
    
    error("Failed after " .. max_retries .. " attempts")
end

-- Usage
local result = robust_llm_call("Explain machine learning", 3)
console.log("Success: " .. result.content)
```

**JavaScript:**
```javascript
// robust-agent.js
async function robustLlmCall(prompt, maxRetries = 3) {
    let attempt = 0;
    
    while (attempt < maxRetries) {
        attempt++;
        
        try {
            const result = await llm.complete({
                model: "gpt-4",
                messages: [{role: "user", content: prompt}]
            });
            return result;
        } catch (error) {
            console.log(`Attempt ${attempt} failed: ${error.message}`);
            
            if (attempt < maxRetries) {
                // Exponential backoff
                await util.sleep(Math.pow(2, attempt) * 1000);
            }
        }
    }
    
    throw new Error(`Failed after ${maxRetries} attempts`);
}

// Usage
const result = await robustLlmCall("Explain machine learning", 3);
console.log(`Success: ${result.content}`);
```

### Configuration and Environment

**Lua:**
```lua
-- config-example.lua
-- Configuration is passed through environment or params
local config = {
    model = env.LLM_MODEL or "gpt-3.5-turbo",
    temperature = tonumber(env.LLM_TEMPERATURE) or 0.7,
    max_tokens = tonumber(env.LLM_MAX_TOKENS) or 1000
}

local agent = llm.agent({
    model = config.model,
    temperature = config.temperature,
    max_tokens = config.max_tokens,
    system = "You are configured with: " .. json.encode(config)
})

local response = agent:run(params.query or "Hello!")
return {
    config = config,
    response = response.content
}
```

**Running with environment:**
```bash
export LLM_MODEL="claude-3-opus"
export LLM_TEMPERATURE="0.3"
rs-llmspell run config-example.lua --param query="What is your configuration?"
```

## Future Considerations

### Planned Enhancements

1. **Vector Store Integration**
   - Bridge abstractions for vector databases
   - RAG (Retrieval-Augmented Generation) script patterns
   - Similarity search and semantic caching through bridges

2. **Advanced Script Engine Features**
   - Python script engine (via PyO3)
   - WebAssembly script engine
   - Custom domain-specific script languages

3. **Enhanced Bridge Ecosystem**
   - Database operation bridges (SQL, NoSQL)
   - Cloud service bridges (AWS, GCP, Azure)
   - Communication bridges (email, Slack, Discord)

4. **Observability and Monitoring**
   - Script execution tracing and metrics
   - Bridge performance monitoring
   - Cross-engine compatibility dashboards

5. **Testing Infrastructure Evolution**
   - Automated test generation from script examples
   - Property-based testing for cross-engine compatibility
   - Performance regression testing framework
   - Chaos engineering for bridge reliability

6. **Advanced Workflow Patterns**
   - DAG (Directed Acyclic Graph) workflows
   - Conditional and loop workflows  
   - Parallel workflow execution with aggregation
   - Real-time reactive workflows

### Extension Points

The architecture supports extensions through:

1. **Custom Script Engines** - Implement the `ScriptEngine` trait for new languages
2. **Custom Bridges** - Implement the `Bridge` trait for new library integrations  
3. **Custom Security Policies** - Extend sandboxing and resource limit frameworks
4. **Custom Test Suites** - Add domain-specific testing contracts and fixtures

### Integration with Rust Ecosystem

Rs-LLMSpell bridges to the rich Rust ecosystem:

- **LLM Libraries**: Rig, Candle, Burn for AI/ML functionality
- **Script Engines**: mlua, boa, quickjs for language support
- **Web Frameworks**: Axum, Warp, Actix-web for server integration
- **Async Runtime**: Tokio for high-performance execution
- **Testing**: rstest, proptest for comprehensive validation
- **Serialization**: Serde for data interchange
- **Observability**: Tracing, Metrics for monitoring

## Conclusion

Rs-LLMSpell's bridge-first, script-engine architecture provides a powerful foundation for building scriptable LLM applications. By leveraging existing Rust libraries like Rig as foundation layers and focusing on clean bridge abstractions, we create a system that is:

- **Scriptable**: Write AI workflows in Lua, JavaScript, or other languages without compilation
- **Performant**: Rust-powered bridges provide native performance with scripting flexibility
- **Secure**: Sandboxed script execution with configurable resource limits
- **Testable**: Comprehensive testing infrastructure ensures reliability across engines and bridges
- **Extensible**: Easy to add new script engines, bridges, and integrations
- **Maintainable**: Clean separation between scripting layer and Rust implementation

The architecture strikes a balance between rapid development through scripting and production-grade performance through Rust, making it easy to build both simple AI automations and complex multi-agent systems with the same foundational platform.