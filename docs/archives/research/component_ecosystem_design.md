# Complete Component Ecosystem Design

## Overview

This document synthesizes all research into a complete, cohesive design for rs-llmspell's component ecosystem, including trait hierarchies, built-in components, hook/event systems, and composition patterns.

## 1. Full Trait Hierarchy

### 1.1 Core Trait Definitions

```rust
// ===== Base Foundation =====

/// Core trait for any component that can handle tools
#[async_trait]
pub trait BaseAgent: Send + Sync {
    /// Unique identifier for this agent
    fn id(&self) -> &str;
    
    /// Human-readable name
    fn name(&self) -> &str;
    
    /// Get available tools
    fn tools(&self) -> &[Box<dyn Tool>];
    
    /// Execute with tool handling
    async fn execute(&mut self, input: AgentInput) -> Result<AgentOutput>;
    
    /// Get current state
    fn state(&self) -> &AgentState;
    
    /// Handle state transition
    async fn transition(&mut self, event: StateEvent) -> Result<()>;
}

/// State management for agents
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentState {
    pub status: AgentStatus,
    pub context: HashMap<String, Value>,
    pub history: Vec<StateTransition>,
    pub metadata: AgentMetadata,
}

/// Tool definition - special functions LLMs can call
#[async_trait]
pub trait Tool: Send + Sync {
    /// Tool name (must be unique within agent)
    fn name(&self) -> &str;
    
    /// Tool description for LLM
    fn description(&self) -> &str;
    
    /// JSON schema for parameters
    fn parameters_schema(&self) -> Value;
    
    /// Execute the tool
    async fn execute(&self, params: Value) -> Result<ToolOutput>;
    
    /// Validate parameters before execution
    fn validate_params(&self, params: &Value) -> Result<()>;
}

// ===== Agent Hierarchy =====

/// LLM-powered agent that extends BaseAgent
#[async_trait]
pub trait Agent: BaseAgent {
    /// Get the LLM provider
    fn llm_provider(&self) -> &dyn LLMProvider;
    
    /// Get system prompt
    fn system_prompt(&self) -> &str;
    
    /// Process a conversation turn
    async fn chat(&mut self, message: &str) -> Result<String>;
    
    /// Stream response
    async fn stream_chat(&mut self, message: &str) -> Result<ResponseStream>;
}

/// Workflow types - deterministic agent patterns
#[async_trait]
pub trait Workflow: BaseAgent {
    /// Workflow type identifier
    fn workflow_type(&self) -> WorkflowType;
    
    /// Get workflow steps
    fn steps(&self) -> &[WorkflowStep];
    
    /// Execute workflow
    async fn run(&mut self, input: WorkflowInput) -> Result<WorkflowOutput>;
    
    /// Pause/resume support
    async fn pause(&mut self) -> Result<WorkflowCheckpoint>;
    async fn resume(&mut self, checkpoint: WorkflowCheckpoint) -> Result<()>;
}

// ===== Provider Abstractions =====

/// LLM provider abstraction (wraps rig)
#[async_trait]
pub trait LLMProvider: Send + Sync {
    /// Provider name
    fn name(&self) -> &str;
    
    /// Complete a prompt
    async fn complete(&self, request: CompletionRequest) -> Result<CompletionResponse>;
    
    /// Stream completion
    async fn stream(&self, request: CompletionRequest) -> Result<ResponseStream>;
    
    /// Count tokens
    fn count_tokens(&self, text: &str) -> Result<usize>;
    
    /// Get model info
    fn model_info(&self) -> &ModelInfo;
}

/// Embedding provider
#[async_trait]
pub trait EmbeddingProvider: Send + Sync {
    /// Generate embeddings
    async fn embed(&self, texts: &[String]) -> Result<Vec<Embedding>>;
    
    /// Embedding dimension
    fn dimension(&self) -> usize;
}
```

### 1.2 Composition Traits

```rust
/// Composable - can be composed into larger structures
pub trait Composable {
    /// Get child components
    fn children(&self) -> &[Box<dyn BaseAgent>];
    
    /// Add child component
    fn add_child(&mut self, child: Box<dyn BaseAgent>) -> Result<()>;
    
    /// Remove child by ID
    fn remove_child(&mut self, id: &str) -> Result<()>;
}

/// Observable - emits events
pub trait Observable {
    /// Subscribe to events
    fn subscribe(&mut self, handler: Box<dyn EventHandler>) -> SubscriptionId;
    
    /// Unsubscribe
    fn unsubscribe(&mut self, id: SubscriptionId);
    
    /// Emit event
    fn emit(&self, event: Event);
}

/// Hookable - supports lifecycle hooks
pub trait Hookable {
    /// Register hook
    fn register_hook(&mut self, point: HookPoint, hook: Box<dyn Hook>);
    
    /// Get hooks for point
    fn hooks(&self, point: HookPoint) -> &[Box<dyn Hook>];
    
    /// Execute hooks
    async fn execute_hooks(&self, point: HookPoint, context: &mut HookContext) -> Result<()>;
}

/// Scriptable - can be controlled from scripts
pub trait Scriptable {
    /// Export to script value
    fn to_script_value(&self) -> ScriptValue;
    
    /// Import from script value
    fn from_script_value(value: ScriptValue) -> Result<Self> where Self: Sized;
    
    /// Register script methods
    fn register_methods(&self, registry: &mut MethodRegistry);
}
```

### 1.3 Trait Relationships

```
┌─────────────────────────────────────────────────────────┐
│                    Scriptable                           │
│  ┌─────────────────────────────────────────────────┐   │
│  │                 Observable                       │   │
│  │  ┌───────────────────────────────────────────┐  │   │
│  │  │              Hookable                     │  │   │
│  │  │  ┌─────────────────────────────────────┐  │  │   │
│  │  │  │         BaseAgent                   │  │  │   │
│  │  │  │  ┌─────────────┐ ┌───────────────┐  │  │  │   │
│  │  │  │  │   Agent     │ │   Workflow    │  │  │  │   │
│  │  │  │  └─────────────┘ └───────────────┘  │  │  │   │
│  │  │  └─────────────────────────────────────┘  │  │   │
│  │  └───────────────────────────────────────────┘  │   │
│  └─────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────┘

                    ┌──────────┐
                    │   Tool   │ (can wrap Agent as Tool)
                    └──────────┘
```

## 2. Built-in Component Library

### 2.1 Built-in Tools (40+ tools organized by category)

```rust
pub mod tools {
    pub mod system {
        pub struct FileSystemTool;      // Read/write files
        pub struct ShellCommandTool;    // Execute commands
        pub struct EnvironmentTool;     // Environment variables
        pub struct ProcessTool;         // Process management
    }
    
    pub mod data {
        pub struct JsonTool;            // JSON manipulation
        pub struct XmlTool;             // XML parsing
        pub struct CsvTool;             // CSV operations
        pub struct SqliteTool;          // SQLite queries
        pub struct RegexTool;           // Regex operations
    }
    
    pub mod web {
        pub struct HttpRequestTool;     // HTTP client
        pub struct WebScraperTool;      // Web scraping
        pub struct RssFeedTool;         // RSS parsing
        pub struct WebSocketTool;       // WebSocket client
    }
    
    pub mod ai {
        pub struct EmbeddingTool;       // Generate embeddings
        pub struct VectorSearchTool;    // Similarity search
        pub struct ImageGenerationTool; // Generate images
        pub struct AudioTranscribeTool; // Speech to text
        pub struct CodeInterpreterTool; // Execute code
    }
    
    pub mod communication {
        pub struct EmailTool;           // Send emails
        pub struct SlackTool;           // Slack integration
        pub struct DiscordTool;         // Discord bot
        pub struct TwilioTool;          // SMS/calls
    }
    
    pub mod math {
        pub struct CalculatorTool;      // Basic math
        pub struct StatisticsTool;      // Statistical ops
        pub struct LinearAlgebraTool;   // Matrix operations
        pub struct SymbolicMathTool;    // Symbolic computation
    }
    
    pub mod time {
        pub struct DateTimeTool;        // Date/time operations
        pub struct TimerTool;           // Set timers
        pub struct SchedulerTool;       // Cron-like scheduling
        pub struct TimeZoneTool;        // Timezone conversion
    }
    
    pub mod crypto {
        pub struct HashTool;            // Hashing functions
        pub struct EncryptionTool;      // Encrypt/decrypt
        pub struct SignatureTool;       // Digital signatures
        pub struct RandomTool;          // Secure random
    }
    
    pub mod integration {
        pub struct GitHubTool;          // GitHub API
        pub struct JiraTool;            // Jira integration
        pub struct AwsTool;             // AWS services
        pub struct KubernetesTool;      // K8s operations
    }
}
```

### 2.2 Built-in Agent Templates

```rust
pub mod agents {
    /// Conversational agent with memory
    pub struct ChatAgent {
        llm: Box<dyn LLMProvider>,
        memory: ConversationMemory,
        tools: Vec<Box<dyn Tool>>,
        config: ChatConfig,
    }
    
    /// Research agent that can search and synthesize
    pub struct ResearchAgent {
        llm: Box<dyn LLMProvider>,
        search_tools: Vec<Box<dyn Tool>>,
        synthesis_prompt: String,
    }
    
    /// Code generation and review agent
    pub struct CodeAgent {
        llm: Box<dyn LLMProvider>,
        language_parsers: HashMap<String, Parser>,
        linters: Vec<Box<dyn Linter>>,
    }
    
    /// Data analysis agent
    pub struct DataAnalyst {
        llm: Box<dyn LLMProvider>,
        data_tools: Vec<Box<dyn Tool>>,
        visualization: Box<dyn Visualizer>,
    }
    
    /// Task planning and execution agent
    pub struct PlannerAgent {
        llm: Box<dyn LLMProvider>,
        task_decomposer: TaskDecomposer,
        executor: TaskExecutor,
    }
    
    /// Multi-agent coordinator
    pub struct OrchestratorAgent {
        agents: HashMap<String, Box<dyn Agent>>,
        routing_strategy: RoutingStrategy,
        coordination_prompt: String,
    }
}
```

### 2.3 Built-in Workflows

```rust
pub mod workflows {
    /// Sequential workflow - steps in order
    pub struct SequentialWorkflow {
        steps: Vec<WorkflowStep>,
        state: WorkflowState,
    }
    
    /// Parallel workflow - concurrent execution
    pub struct ParallelWorkflow {
        branches: Vec<WorkflowBranch>,
        join_strategy: JoinStrategy,
    }
    
    /// Conditional workflow - branching logic
    pub struct ConditionalWorkflow {
        conditions: Vec<Condition>,
        branches: HashMap<String, WorkflowBranch>,
        default_branch: Option<WorkflowBranch>,
    }
    
    /// Loop workflow - iteration
    pub struct LoopWorkflow {
        condition: LoopCondition,
        body: Box<dyn Workflow>,
        max_iterations: Option<usize>,
    }
    
    /// Map-reduce workflow
    pub struct MapReduceWorkflow {
        mapper: Box<dyn BaseAgent>,
        reducer: Box<dyn BaseAgent>,
        partitioner: Partitioner,
    }
    
    /// Pipeline workflow - data transformation
    pub struct PipelineWorkflow {
        stages: Vec<PipelineStage>,
        error_handler: ErrorHandler,
    }
}
```

## 3. Hook and Event System

### 3.1 Hook System Design

```rust
/// Hook points in agent lifecycle
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum HookPoint {
    // Agent lifecycle
    BeforeInit,
    AfterInit,
    BeforeExecute,
    AfterExecute,
    BeforeShutdown,
    AfterShutdown,
    
    // LLM interactions
    BeforeLLMCall,
    AfterLLMCall,
    OnLLMError,
    OnTokenLimit,
    
    // Tool execution
    BeforeToolCall,
    AfterToolCall,
    OnToolError,
    OnToolValidation,
    
    // State management
    BeforeStateChange,
    AfterStateChange,
    OnStateError,
    
    // Workflow specific
    BeforeWorkflowStep,
    AfterWorkflowStep,
    OnWorkflowBranch,
    OnWorkflowComplete,
}

/// Hook trait
#[async_trait]
pub trait Hook: Send + Sync {
    /// Hook name
    fn name(&self) -> &str;
    
    /// Execute hook
    async fn execute(&self, context: &mut HookContext) -> Result<HookResult>;
    
    /// Priority (lower executes first)
    fn priority(&self) -> i32 {
        0
    }
}

/// Hook context passed to hooks
pub struct HookContext {
    pub point: HookPoint,
    pub agent_id: String,
    pub data: HashMap<String, Value>,
    pub state: AgentState,
    pub span: Span,
}

/// Built-in hooks
pub mod hooks {
    pub struct LoggingHook {
        level: Level,
        format: LogFormat,
    }
    
    pub struct MetricsHook {
        collector: MetricsCollector,
        labels: HashMap<String, String>,
    }
    
    pub struct TracingHook {
        tracer: Tracer,
        sample_rate: f64,
    }
    
    pub struct RateLimitHook {
        limiter: RateLimiter,
        key_fn: Box<dyn Fn(&HookContext) -> String>,
    }
    
    pub struct CacheHook {
        cache: Box<dyn Cache>,
        ttl: Duration,
    }
    
    pub struct ValidationHook {
        validators: Vec<Box<dyn Validator>>,
    }
}
```

### 3.2 Event System Design

```rust
/// Event types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Event {
    // Agent events
    AgentStarted { agent_id: String, timestamp: DateTime<Utc> },
    AgentCompleted { agent_id: String, result: AgentResult },
    AgentFailed { agent_id: String, error: String },
    
    // Tool events
    ToolCalled { tool_name: String, params: Value },
    ToolCompleted { tool_name: String, output: ToolOutput },
    ToolFailed { tool_name: String, error: String },
    
    // State events
    StateChanged { from: AgentStatus, to: AgentStatus },
    ContextUpdated { key: String, value: Value },
    
    // Workflow events
    WorkflowStarted { workflow_id: String },
    WorkflowStepCompleted { step_id: String },
    WorkflowBranchTaken { branch_id: String },
    
    // Custom events
    Custom { name: String, data: Value },
}

/// Event handler trait
#[async_trait]
pub trait EventHandler: Send + Sync {
    /// Handle event
    async fn handle(&self, event: &Event) -> Result<()>;
    
    /// Event filter
    fn filter(&self) -> Option<EventFilter> {
        None
    }
}

/// Event bus implementation
pub struct EventBus {
    /// Async broadcast channel for in-process events
    async_tx: broadcast::Sender<Event>,
    
    /// Crossbeam channel for cross-thread events
    sync_tx: crossbeam::channel::Sender<Event>,
    
    /// Handlers
    handlers: Arc<RwLock<Vec<Box<dyn EventHandler>>>>,
    
    /// Event processors
    processors: Vec<Box<dyn EventProcessor>>,
}

impl EventBus {
    /// Emit event
    pub fn emit(&self, event: Event) {
        // Send to both channels
        let _ = self.async_tx.send(event.clone());
        let _ = self.sync_tx.send(event);
    }
    
    /// Subscribe to events
    pub async fn subscribe(&self, handler: Box<dyn EventHandler>) {
        self.handlers.write().await.push(handler);
    }
    
    /// Process events with processors
    pub async fn process(&self) {
        // Run processors in parallel
        let tasks: Vec<_> = self.processors.iter()
            .map(|p| p.process(self.async_tx.subscribe()))
            .collect();
        
        futures::future::join_all(tasks).await;
    }
}
```

## 4. Composition and Orchestration Patterns

### 4.1 Agent Composition Patterns

```rust
/// Tool-wrapped agent pattern
pub struct AgentAsTool<A: Agent> {
    agent: A,
    tool_name: String,
    tool_description: String,
}

impl<A: Agent> Tool for AgentAsTool<A> {
    fn name(&self) -> &str {
        &self.tool_name
    }
    
    async fn execute(&self, params: Value) -> Result<ToolOutput> {
        let message = params.get("message")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow!("Missing message parameter"))?;
            
        let response = self.agent.chat(message).await?;
        
        Ok(ToolOutput {
            content: response,
            metadata: Default::default(),
        })
    }
}

/// Composite agent pattern
pub struct CompositeAgent {
    agents: Vec<Box<dyn Agent>>,
    routing_strategy: Box<dyn RoutingStrategy>,
    aggregation_strategy: Box<dyn AggregationStrategy>,
}

/// Pipeline agent pattern
pub struct PipelineAgent {
    stages: Vec<Box<dyn Agent>>,
    transformers: Vec<Box<dyn Transformer>>,
}

/// Hierarchical agent pattern
pub struct HierarchicalAgent {
    supervisor: Box<dyn Agent>,
    workers: HashMap<String, Box<dyn Agent>>,
    delegation_strategy: Box<dyn DelegationStrategy>,
}
```

### 4.2 Orchestration Patterns

```rust
/// Agent pool for load balancing
pub struct AgentPool<A: Agent + Clone> {
    agents: Vec<A>,
    strategy: LoadBalancingStrategy,
    health_checker: HealthChecker,
}

/// Agent mesh for peer-to-peer coordination
pub struct AgentMesh {
    nodes: HashMap<String, MeshNode>,
    discovery: ServiceDiscovery,
    gossip: GossipProtocol,
}

/// Saga pattern for distributed transactions
pub struct AgentSaga {
    steps: Vec<SagaStep>,
    compensations: Vec<CompensationStep>,
    coordinator: SagaCoordinator,
}

/// Event-driven orchestration
pub struct EventDrivenOrchestrator {
    event_bus: EventBus,
    rules: Vec<OrchestrationRule>,
    state_machine: StateMachine,
}
```

### 4.3 State Handoff Patterns

```rust
/// State transfer between agents
pub struct StateHandoff {
    from_agent: String,
    to_agent: String,
    state_filter: StateFilter,
    transform: Option<StateTransform>,
}

/// Shared state repository
pub struct SharedStateRepository {
    store: Box<dyn StateStore>,
    subscriptions: HashMap<String, Vec<String>>,
    access_control: AccessControl,
}

/// State synchronization
pub struct StateSynchronizer {
    agents: Vec<String>,
    sync_strategy: SyncStrategy,
    conflict_resolver: ConflictResolver,
}
```

## 5. Script Integration Layer

### 4.1 Script API Design

```rust
/// Script runtime abstraction
pub trait ScriptRuntime: Send + Sync {
    /// Execute script
    async fn execute(&self, script: &str) -> Result<ScriptValue>;
    
    /// Register global object
    fn register_global(&mut self, name: &str, value: ScriptValue) -> Result<()>;
    
    /// Create agent from script
    async fn create_agent(&self, definition: &str) -> Result<Box<dyn Agent>>;
}

/// Lua-specific bindings
pub mod lua {
    use mlua::prelude::*;
    
    impl LuaUserData for ScriptableAgent {
        fn add_methods<'lua, M: LuaUserDataMethods<'lua, Self>>(methods: &mut M) {
            methods.add_async_method("chat", |_, this, message: String| async move {
                this.chat(&message).await.to_lua_err()
            });
            
            methods.add_method("add_tool", |_, this, tool: LuaTable| {
                let tool = Tool::from_lua_table(tool)?;
                this.add_tool(Box::new(tool)).to_lua_err()
            });
            
            methods.add_method("on", |_, this, (event, handler): (String, LuaFunction)| {
                this.register_event_handler(&event, handler).to_lua_err()
            });
        }
    }
}

/// JavaScript-specific bindings
pub mod js {
    use rquickjs::prelude::*;
    
    impl<'js> IntoJs<'js> for ScriptableAgent {
        fn into_js(self, ctx: Ctx<'js>) -> Result<Value<'js>> {
            let obj = Object::new(ctx)?;
            
            obj.set("chat", Func::from(|message: String| async move {
                self.chat(&message).await
            }))?;
            
            obj.set("addTool", Func::from(|tool: Object| {
                let tool = Tool::from_js_object(tool)?;
                self.add_tool(Box::new(tool))
            }))?;
            
            Ok(obj.into())
        }
    }
}
```

### 4.2 Cross-Language Async Patterns

```rust
/// Promise/Future wrapper for scripts
pub struct ScriptPromise {
    future: Pin<Box<dyn Future<Output = Result<ScriptValue>> + Send>>,
    runtime: Arc<dyn ScriptRuntime>,
}

/// Async coordinator for script execution
pub struct AsyncCoordinator {
    /// Lua coroutine scheduler
    lua_scheduler: LuaCoroutineScheduler,
    
    /// JavaScript promise resolver
    js_resolver: JsPromiseResolver,
    
    /// Unified task queue
    task_queue: TaskQueue,
}

/// Cooperative scheduling
impl AsyncCoordinator {
    pub async fn run(&mut self) {
        loop {
            tokio::select! {
                Some(lua_task) = self.lua_scheduler.next() => {
                    self.handle_lua_task(lua_task).await;
                }
                Some(js_task) = self.js_resolver.next() => {
                    self.handle_js_task(js_task).await;
                }
                _ = tokio::time::sleep(Duration::from_millis(10)) => {
                    self.yield_to_scripts().await;
                }
            }
        }
    }
}
```

## 6. Example Usage Patterns

### 6.1 Basic Agent Creation

```rust
// Rust
let agent = ChatAgent::builder()
    .llm_provider(rig_provider)
    .system_prompt("You are a helpful assistant")
    .add_tool(CalculatorTool::new())
    .add_tool(WebSearchTool::new())
    .with_hook(LoggingHook::new())
    .build();
```

```lua
-- Lua
local agent = Agent.new({
    provider = "openai",
    model = "gpt-4",
    system_prompt = "You are a helpful assistant",
    tools = {
        Calculator.new(),
        WebSearch.new()
    },
    hooks = {
        logging = { level = "info" },
        metrics = { prefix = "agent" }
    }
})

-- Add event handler
agent:on("tool_called", function(event)
    print("Tool called: " .. event.tool_name)
end)
```

```javascript
// JavaScript
const agent = new Agent({
    provider: "anthropic",
    model: "claude-3",
    systemPrompt: "You are a helpful assistant",
    tools: [
        new Calculator(),
        new WebSearch()
    ]
});

// Async chat
const response = await agent.chat("What is 2+2?");

// Streaming
for await (const chunk of agent.streamChat("Tell me a story")) {
    console.log(chunk);
}
```

### 6.2 Workflow Example

```lua
-- Define a research workflow
local workflow = Workflow.sequential({
    steps = {
        -- Step 1: Search for information
        {
            agent = ResearchAgent.new(),
            input = "{{query}}",
            output = "search_results"
        },
        -- Step 2: Analyze results
        {
            agent = DataAnalyst.new(),
            input = "{{search_results}}",
            output = "analysis"
        },
        -- Step 3: Generate report
        {
            agent = ReportWriter.new(),
            input = {
                analysis = "{{analysis}}",
                format = "markdown"
            },
            output = "report"
        }
    }
})

-- Execute workflow
local result = workflow:run({ query = "climate change impacts" })
```

### 6.3 Multi-Agent Orchestration

```javascript
// Create an orchestrator
const orchestrator = new OrchestratorAgent({
    agents: {
        researcher: new ResearchAgent(),
        coder: new CodeAgent(),
        reviewer: new ReviewAgent()
    },
    routing: {
        strategy: "skill-based",
        rules: [
            { pattern: /code|program|function/, agent: "coder" },
            { pattern: /research|find|search/, agent: "researcher" },
            { pattern: /review|check|verify/, agent: "reviewer" }
        ]
    }
});

// Handle complex request
const result = await orchestrator.execute({
    task: "Research the latest ML algorithms and implement a simple example"
});
```

## 7. Implementation Priorities

### Phase 1: Core Traits and Base Implementation
1. BaseAgent, Agent, Tool traits
2. Basic state management
3. Simple workflow types
4. Core hook system

### Phase 2: Providers and Tools
1. LLM provider abstraction (wrapping rig)
2. 10-15 essential built-in tools
3. Basic agent templates
4. Event system foundation

### Phase 3: Script Integration
1. Lua bindings for core types
2. JavaScript bindings
3. Async coordination
4. Script-friendly APIs

### Phase 4: Advanced Features
1. Complete built-in library
2. Advanced workflows
3. Full orchestration patterns
4. Production optimizations

## Conclusion

This design provides a complete, extensible component ecosystem that:
- Implements go-llms patterns adapted for Rust
- Provides rich built-in functionality
- Enables flexible composition and orchestration
- Integrates seamlessly with scripting languages
- Supports production-grade observability and control

The modular architecture allows incremental implementation while maintaining consistency across all components.