# Rs-LLMSpell Architecture (v2)

## Table of Contents

1. [Introduction](#introduction)
2. [Why Rs-LLMSpell Exists](#why-rs-llmspell-exists)
3. [Core Philosophy](#core-philosophy)
4. [Architecture Overview](#architecture-overview)
5. [Component Hierarchy](#component-hierarchy)
6. [Bridge-First Design](#bridge-first-design)
7. [Script Interface](#script-interface)
8. [Hook and Event System](#hook-and-event-system)
9. [State Management](#state-management)
10. [Built-in Components](#built-in-components)
11. [Advanced Orchestration Patterns](#advanced-orchestration-patterns)
12. [Protocol Integration](#protocol-integration)
13. [Cross-Engine Compatibility](#cross-engine-compatibility)
14. [Error Handling Strategy](#error-handling-strategy)
15. [Testing Architecture](#testing-architecture)
16. [Security Model](#security-model)
17. [Examples](#examples)
18. [Future Evolution](#future-evolution)
19. [Implementation Roadmap](#implementation-roadmap)

---

**Quick Links:**
- [TODO.md](../../TODO.md) - Current implementation tasks
- [Component Ecosystem Design](./component_ecosystem_design.md) - Detailed component specifications
- [Script Interface Design](./script_interface_design.md) - Complete scripting API reference
- [Build vs Buy Decisions](./build_vs_buy_decision_matrix.md) - Technology choices

## Introduction

Rs-LLMSpell is a **scriptable LLM interaction framework** that enables developers to orchestrate AI agents, tools, and workflows through expressive scripts in Lua, JavaScript, and other languages. It provides a bridge between high-performance Rust implementations and flexible scripting environments.

### What Makes Rs-LLMSpell Unique

1. **Go-llms Inspired Architecture**: Implements the proven BaseAgent/Agent/Tool/Workflow patterns from go-llms, adapted for Rust
2. **Multi-Language Scripting**: Write AI orchestration logic in Lua, JavaScript, or other supported languages
3. **Production-Ready Infrastructure**: Built-in hooks, events, state management, and observability
4. **Extensible Component Library**: 40+ built-in tools, multiple agent templates, and workflow patterns
5. **Bridge-First Philosophy**: Leverages existing Rust crates (rig, mlua, etc.) rather than reimplementing

### What is a Spell?

A spell is a script that orchestrates AI capabilities through rs-llmspell's unified API:

```lua
-- Example: Multi-agent research workflow (Lua)
local workflow = Workflow.sequential({
    name = "deep_research",
    steps = {
        {
            agent = ResearchAgent.new({ sources = {"academic", "news"} }),
            action = "research",
            output = "raw_research"
        },
        {
            agent = AnalysisAgent.new({ tools = {StatisticsTool.new()} }),
            action = "analyze", 
            input = "{{raw_research}}",
            output = "analysis"
        },
        {
            agent = WriterAgent.new({ style = "technical" }),
            action = "write_report",
            input = { research = "{{raw_research}}", analysis = "{{analysis}}" },
            output = "report"
        }
    }
})

local report = workflow:run({ topic = "AI Safety advances in 2025" })
```

## Why Rs-LLMSpell Exists

### The Problem Space

1. **Compilation Barrier**: Rust's compile-edit-test cycle slows AI experimentation
2. **Complex Orchestration**: Multi-agent workflows require sophisticated coordination
3. **Language Diversity**: Teams have different scripting language preferences
4. **Production Requirements**: Real systems need hooks, events, state management
5. **Tool Integration**: AI agents need access to diverse external capabilities

### Our Solution

Rs-LLMSpell addresses these challenges through:

- **Scriptable Orchestration**: Define complex AI workflows without recompilation
- **Unified Component Model**: BaseAgent/Agent/Tool/Workflow hierarchy for all AI components
- **Production Infrastructure**: Built-in hooks, events, state management, and observability
- **Language Agnostic**: Consistent API across Lua, JavaScript, and future languages
- **Bridge Architecture**: Leverages best-in-class Rust crates rather than reimplementing

### Key Benefits

🚀 **Rapid Iteration**: Test AI behaviors instantly without compilation  
🔧 **Production Ready**: Hooks, events, and state management built-in  
🌐 **Multi-Language**: Use Lua, JavaScript, or extend to other languages  
🧩 **Composable**: Build complex systems from simple components  
⚡ **High Performance**: Rust core with efficient script bridges  
🔒 **Secure**: Sandboxed execution with resource limits  

## Core Philosophy

### 1. Bridge, Don't Build

**Fundamental Principle**: We bridge existing Rust crates rather than reimplementing functionality.

- LLM Providers: Bridge to `rig` for multi-provider support
- Scripting: Bridge to `mlua` and JavaScript engines
- State Storage: Bridge to `sled`/`rocksdb` behind traits
- Observability: Bridge to `tracing`, `metrics-rs`

This ensures:
- Leverage battle-tested implementations
- Automatic compatibility with upstream updates
- Minimal maintenance burden
- Focus on our unique value: scriptable orchestration

### 2. Go-llms Patterns, Rust Implementation

We adopt the proven patterns from go-llms while leveraging Rust's strengths:

- **BaseAgent**: Foundation for any tool-handling component
- **Agent**: LLM-powered extension of BaseAgent
- **Tool**: Discrete, callable functions for agents
- **Workflow**: Deterministic orchestration patterns
- **Tool-Wrapped Agents**: Agents callable as tools

### 3. Production-First Design

Every component is designed for production use:

- **Hooks**: 20+ lifecycle interception points
- **Events**: Comprehensive event system for monitoring
- **State**: Distributed state management with handoff
- **Observability**: Built-in logging, metrics, tracing
- **Error Handling**: Graceful degradation and recovery

### 4. Language-Idiomatic APIs

Each scripting language gets an API that feels native:

```lua
-- Lua: Table-based, colon syntax
agent:add_tool(Calculator.new())
agent:on("tool_called", handler)
```

```javascript
// JavaScript: Object-oriented, promises
agent.addTool(new Calculator());
agent.on("toolCalled", handler);
```

## Architecture Overview

### High-Level Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                    Script Layer (Lua/JS)                    │
│        User-written spells and orchestration logic          │
├─────────────────────────────────────────────────────────────┤
│                    Bridge Layer (FFI)                       │
│         Type conversion, async adaptation, safety           │
├─────────────────────────────────────────────────────────────┤
│                 Core Components (Rust)                      │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐        │
│  │   Agents    │  │   Tools     │  │  Workflows  │        │
│  └─────────────┘  └─────────────┘  └─────────────┘        │
├─────────────────────────────────────────────────────────────┤
│                 Infrastructure Layer                        │
│  ┌────────┐  ┌────────┐  ┌────────┐  ┌────────┐          │
│  │  LLM   │  │ State  │  │ Events │  │ Hooks  │          │
│  └────────┘  └────────┘  └────────┘  └────────┘          │
├─────────────────────────────────────────────────────────────┤
│                 External Dependencies                       │
│      rig, mlua, sled/rocksdb, tokio, tracing              │
└─────────────────────────────────────────────────────────────┘
```

### Data Flow

```
User Script → Bridge (Type Conversion) → Agent/Workflow → LLM Provider
     ↑                                          ↓
     └──────── Bridge (Result Conversion) ← Tool Execution
                                                ↓
                                          State/Events/Hooks
```

## Component Hierarchy

### Core Trait Hierarchy

```rust
/// Foundation: Any component that can handle tools
#[async_trait]
trait BaseAgent {
    fn id(&self) -> &str;
    fn tools(&self) -> &[Box<dyn Tool>];
    async fn execute(&mut self, input: AgentInput) -> Result<AgentOutput>;
    fn state(&self) -> &AgentState;
}

/// LLM-powered agent extending BaseAgent
#[async_trait] 
trait Agent: BaseAgent {
    fn llm_provider(&self) -> &dyn LLMProvider;
    fn system_prompt(&self) -> &str;
    async fn chat(&mut self, message: &str) -> Result<String>;
}

/// Callable tool for agents
#[async_trait]
trait Tool {
    fn name(&self) -> &str;
    fn description(&self) -> &str;
    fn parameters_schema(&self) -> Value;
    async fn execute(&self, params: Value) -> Result<ToolOutput>;
}

/// Deterministic execution patterns
#[async_trait]
trait Workflow: BaseAgent {
    fn workflow_type(&self) -> WorkflowType;
    async fn run(&mut self, input: WorkflowInput) -> Result<WorkflowOutput>;
}
```

### Composition Patterns

1. **Tool-Wrapped Agent**: Any agent can be wrapped as a tool
2. **Composite Agent**: Multiple agents working together
3. **Hierarchical Agent**: Supervisor-worker patterns
4. **Pipeline Agent**: Sequential processing stages

## Bridge-First Design

### Bridge Architecture

We bridge to best-in-class Rust crates:

| Component | Bridge To | Why |
|-----------|-----------|-----|
| LLM Providers | `rig` | Comprehensive multi-provider support |
| Lua Scripting | `mlua` | Mature, async-capable Lua binding |
| JS Scripting | `rquickjs` | Lightweight, embeddable JS engine |
| State Storage | `sled`/`rocksdb` | Embedded persistence options |
| Observability | `tracing` | Structured, async-aware logging |

### Type Conversion

Unified value type for cross-language communication:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
enum ScriptValue {
    Null,
    Bool(bool),
    Number(f64), 
    String(String),
    Array(Vec<ScriptValue>),
    Object(HashMap<String, ScriptValue>),
    Function(FunctionRef),
    Promise(PromiseRef),
}
```

### Async Patterns

Different languages require different async approaches:

- **Lua**: Coroutines with Promise-like wrappers
- **JavaScript**: Native promises and async/await
- **Rust**: Tokio-based async runtime

## Script Interface

### Lua API Highlights

```lua
-- Agent creation
local agent = Agent.new({
    system_prompt = "You are a helpful assistant",
    provider = "openai",
    tools = { Calculator.new(), WebSearch.new() }
})

-- Async operations with coroutines
coroutine.wrap(function()
    local response = agent:chat_async("Hello")
    print(response)
end)()

-- Event handling
agent:on("tool_called", function(event)
    print("Tool used:", event.tool_name)
end)

-- Workflow definition
local workflow = Workflow.sequential({
    steps = { ... }
})
```

### JavaScript API Highlights

```javascript
// Agent creation  
const agent = new Agent({
    systemPrompt: "You are a helpful assistant",
    provider: "anthropic",
    tools: [new Calculator(), new WebSearch()]
});

// Native async/await
const response = await agent.chat("Hello");

// Streaming
for await (const chunk of agent.streamChat("Tell me a story")) {
    process.stdout.write(chunk);
}

// Event handling
agent.on("toolCalled", (event) => {
    console.log("Tool used:", event.toolName);
});
```

## Hook and Event System

### Hook Points

20+ hook points throughout the lifecycle:

- `beforeInit`, `afterInit`
- `beforeLLMCall`, `afterLLMCall`
- `beforeToolCall`, `afterToolCall`
- `beforeStateChange`, `afterStateChange`
- `onError`, `onRetry`

### Event System

Comprehensive event emission and handling:

```javascript
// Global event bus
eventBus.on("agent:started", (event) => {
    metrics.increment("agents.active");
});

// Agent-specific events
agent.on("tool:*", (event) => {
    audit.log(event);
});
```

### Built-in Hooks

- **LoggingHook**: Structured logging with context
- **MetricsHook**: Performance and usage metrics
- **RateLimitHook**: Request throttling
- **CacheHook**: Response caching
- **RetryHook**: Automatic retry with backoff

## State Management

### State Hierarchy

1. **Agent State**: Local to each agent instance
2. **Workflow State**: Managed by workflows, includes child states
3. **Global State**: Shared via StateRepository with access control

### State Storage

Abstracted behind traits for flexibility:

```rust
trait StateStore {
    async fn get(&self, key: &str) -> Result<Option<Vec<u8>>>;
    async fn set(&self, key: &str, value: Vec<u8>) -> Result<()>;
    async fn watch(&self, prefix: &str) -> Result<StateWatcher>;
}
```

- Development: `sled` for fast iteration
- Production: `rocksdb` for reliability

### State Handoff

Agents can transfer state during handoffs:

```lua
local handoff = StateHandoff.new({
    from = "researcher",
    to = "writer", 
    filter = { include = {"research_notes", "sources"} }
})
```

## Built-in Components

### Tools (40+)

Organized by category:

- **System**: FileSystem, Shell, Environment, Process
- **Data**: JSON, XML, CSV, SQLite, Regex
- **Web**: HTTP, WebScraper, RSS, WebSocket
- **AI**: Embedding, VectorSearch, ImageGen, Transcribe
- **Communication**: Email, Slack, Discord, SMS
- **Math**: Calculator, Statistics, LinearAlgebra
- **Time**: DateTime, Timer, Scheduler
- **Crypto**: Hash, Encryption, Signature

### Agent Templates

Pre-built agents for common use cases:

- **ChatAgent**: Conversational with memory
- **ResearchAgent**: Search and synthesis
- **CodeAgent**: Code generation and review
- **DataAnalyst**: Data processing and visualization
- **PlannerAgent**: Task decomposition
- **OrchestratorAgent**: Multi-agent coordination

### Workflow Types

- **Sequential**: Steps in order
- **Parallel**: Concurrent execution
- **Conditional**: Branching logic
- **Loop**: Iteration patterns
- **MapReduce**: Distributed processing
- **Pipeline**: Stream transformation

## Testing Architecture

### Testing Strategy

Multi-layer testing approach:

1. **Unit Tests**: Mock LLM providers with `mockall`
2. **Property Tests**: Invariants with `proptest`
3. **Integration Tests**: Real component interaction
4. **Performance Tests**: Benchmarks with `criterion`

### Test Infrastructure

```rust
// Mock LLM for testing
#[automock]
trait LLMProvider {
    async fn complete(&self, prompt: &str) -> Result<String>;
}

// Property testing
proptest! {
    #[test]
    fn agent_state_serialization(state: AgentState) {
        let serialized = serde_json::to_string(&state)?;
        let deserialized: AgentState = serde_json::from_str(&serialized)?;
        prop_assert_eq!(state, deserialized);
    }
}
```

## Security Model

### Script Sandboxing

- Resource limits (CPU, memory, time)
- Filesystem access restrictions
- Network access control
- No access to process/system APIs

### Tool Permissions

```lua
local tool = FileSystemTool.new({
    allowed_paths = {"/tmp", "./output"},
    read_only = true,
    max_file_size = 10 * 1024 * 1024  -- 10MB
})
```

### Audit Trail

All actions logged with context:

```javascript
{
    timestamp: "2025-01-20T10:30:00Z",
    agentId: "agent_123",
    action: "tool_call",
    tool: "file_write",
    params: { path: "./output/report.md" },
    result: "success",
    userId: "user_456",
    sessionId: "session_789"
}
```

## Advanced Orchestration Patterns

Rs-llmspell supports sophisticated multi-agent orchestration patterns for complex AI workflows.

### Dynamic Workflow Creation

```rust
// Dynamic workflow builder
pub struct DynamicWorkflowBuilder {
    steps: Vec<WorkflowStep>,
    conditions: HashMap<String, Box<dyn Condition>>,
    loop_handlers: HashMap<String, Box<dyn LoopHandler>>,
}

impl DynamicWorkflowBuilder {
    pub fn conditional(mut self, condition: impl Condition + 'static, 
                      true_branch: WorkflowStep, 
                      false_branch: WorkflowStep) -> Self {
        let condition_id = format!("condition_{}", self.conditions.len());
        self.conditions.insert(condition_id.clone(), Box::new(condition));
        self.steps.push(WorkflowStep::Conditional {
            condition_id,
            true_branch: Box::new(true_branch),
            false_branch: Box::new(false_branch),
        });
        self
    }
    
    pub fn parallel_fan_out(mut self, steps: Vec<WorkflowStep>, 
                           aggregator: Box<dyn OutputAggregator>) -> Self {
        self.steps.push(WorkflowStep::ParallelFanOut { steps, aggregator });
        self
    }
    
    pub fn adaptive_loop(mut self, condition: impl LoopCondition + 'static,
                        body: WorkflowStep, max_iterations: usize) -> Self {
        let loop_id = format!("loop_{}", self.loop_handlers.len());
        self.loop_handlers.insert(loop_id.clone(), Box::new(AdaptiveLoopHandler::new(
            Box::new(condition), max_iterations
        )));
        self.steps.push(WorkflowStep::AdaptiveLoop { loop_id, body: Box::new(body) });
        self
    }
}
```

### Multi-Agent Collaboration

```lua
-- Advanced collaboration pattern in Lua
local CollaborationWorkflow = Workflow.create({
    name = "multi_expert_analysis",
    
    steps = {
        -- Parallel expert analysis
        {
            type = "parallel_fan_out",
            agents = {
                { agent = "TechnicalExpert", role = "technical_analysis" },
                { agent = "MarketExpert", role = "market_analysis" },
                { agent = "RiskExpert", role = "risk_assessment" }
            },
            aggregator = function(results)
                return {
                    technical = results[1],
                    market = results[2],
                    risk = results[3],
                    correlation_score = Analyzers.calculate_correlation(results)
                }
            end
        },
        
        -- Synthesis by senior agent
        {
            agent = "SeniorAnalyst",
            action = "synthesize_analysis",
            input_transform = function(parallel_results)
                return {
                    expert_inputs = parallel_results,
                    synthesis_instructions = "Provide comprehensive analysis synthesis"
                }
            end
        },
        
        -- Adaptive refinement loop
        {
            type = "adaptive_loop",
            condition = function(context)
                return context.confidence_score < 0.85 and context.iteration < 3
            end,
            body = {
                agent = "QualityReviewer",
                action = "identify_gaps",
                on_output = function(gaps, context)
                    if #gaps > 0 then
                        -- Request specific expert clarification
                        for _, gap in ipairs(gaps) do
                            local expert = Experts.get_for_domain(gap.domain)
                            expert:clarify(gap.question)
                        end
                    end
                end
            }
        }
    }
})
```

## Protocol Integration

Rs-llmspell provides built-in support for modern AI protocols and standards.

### Model Control Protocol (MCP) Support

```rust
// MCP client implementation
pub struct MCPClient {
    connection: Box<dyn MCPConnection>,
    tools: HashMap<String, MCPTool>,
    resources: HashMap<String, MCPResource>,
    prompts: HashMap<String, MCPPrompt>,
}

impl MCPClient {
    pub async fn discover_capabilities(&mut self) -> Result<MCPCapabilities> {
        let request = MCPRequest::Initialize {
            protocol_version: "2024-11-05".to_string(),
            capabilities: ClientCapabilities {
                roots: Some(RootsCapability { list_changed: true }),
                sampling: None,
            },
            client_info: ClientInfo {
                name: "rs-llmspell".to_string(),
                version: env!("CARGO_PKG_VERSION").to_string(),
            },
        };
        
        let response = self.connection.send_request(request).await?;
        
        match response {
            MCPResponse::Initialize(init_response) => {
                // Discover available tools
                if init_response.capabilities.tools.is_some() {
                    let tools_response = self.connection.send_request(
                        MCPRequest::ListTools
                    ).await?;
                    
                    if let MCPResponse::ListTools(tools) = tools_response {
                        for tool in tools.tools {
                            self.tools.insert(tool.name.clone(), MCPTool::from(tool));
                        }
                    }
                }
                
                // Discover available resources
                if init_response.capabilities.resources.is_some() {
                    let resources_response = self.connection.send_request(
                        MCPRequest::ListResources
                    ).await?;
                    
                    if let MCPResponse::ListResources(resources) = resources_response {
                        for resource in resources.resources {
                            self.resources.insert(resource.uri.clone(), MCPResource::from(resource));
                        }
                    }
                }
                
                Ok(init_response.capabilities)
            },
            _ => Err(anyhow!("Unexpected response to initialize request"))
        }
    }
    
    pub async fn call_mcp_tool(&self, tool_name: &str, arguments: serde_json::Value) -> Result<ToolResult> {
        let tool = self.tools.get(tool_name)
            .ok_or_else(|| anyhow!("MCP tool not found: {}", tool_name))?;
        
        let request = MCPRequest::CallTool {
            name: tool_name.to_string(),
            arguments: Some(arguments),
        };
        
        let response = self.connection.send_request(request).await?;
        
        match response {
            MCPResponse::CallTool(result) => Ok(result),
            _ => Err(anyhow!("Unexpected response to tool call"))
        }
    }
}

// Integration with rs-llmspell tools
#[async_trait]
impl Tool for MCPTool {
    fn name(&self) -> &str { &self.mcp_tool.name }
    fn description(&self) -> &str { &self.mcp_tool.description }
    
    async fn execute(&self, input: ToolInput) -> Result<ToolOutput> {
        let mcp_result = self.client.call_mcp_tool(&self.mcp_tool.name, input.parameters).await?;
        
        Ok(ToolOutput {
            result: mcp_result.content,
            metadata: HashMap::new(),
            artifacts: vec![],
        })
    }
}
```

### Agent to Agent (A2A) Protocol

```rust
// A2A protocol implementation for distributed agent networks
pub struct A2AProtocolHandler {
    local_agents: AgentRegistry,
    remote_connections: HashMap<String, A2AConnection>,
    message_router: MessageRouter,
    security_manager: SecurityManager,
}

impl A2AProtocolHandler {
    pub async fn register_agent(&mut self, agent: Box<dyn BaseAgent>, 
                               capabilities: AgentCapabilities) -> Result<()> {
        // Register locally
        self.local_agents.register(agent.id().to_string(), agent).await?;
        
        // Announce to network
        let announcement = A2AMessage::AgentAnnouncement {
            agent_id: agent.id().to_string(),
            capabilities,
            endpoint: self.get_local_endpoint(),
            authentication: self.security_manager.create_agent_token(agent.id())?,
        };
        
        self.broadcast_to_network(announcement).await?;
        Ok(())
    }
    
    pub async fn invoke_remote_agent(&self, agent_id: &str, 
                                   request: AgentRequest) -> Result<AgentResponse> {
        // Find remote agent
        let connection = self.find_agent_connection(agent_id).await?;
        
        // Create authenticated request
        let a2a_request = A2AMessage::AgentInvocation {
            target_agent: agent_id.to_string(),
            request,
            authentication: self.security_manager.create_request_token()?,
            correlation_id: Uuid::new_v4().to_string(),
        };
        
        // Send request and await response
        let response = connection.send_request(a2a_request).await?;
        
        match response {
            A2AMessage::AgentResponse { response, .. } => Ok(response),
            A2AMessage::Error { error, .. } => Err(anyhow!("Remote agent error: {}", error)),
            _ => Err(anyhow!("Unexpected A2A response"))
        }
    }
}

// A2A usage in workflow
```lua
-- Cross-network agent collaboration
local NetworkWorkflow = Workflow.create({
    name = "distributed_research",
    
    steps = {
        {
            agent = "local://research_coordinator",
            action = "plan_research",
            output = "research_plan"
        },
        {
            type = "parallel",
            steps = {
                {
                    agent = "a2a://university_network/literature_agent",
                    action = "search_literature",
                    input = "{{research_plan.literature_query}}"
                },
                {
                    agent = "a2a://corporate_network/market_agent", 
                    action = "analyze_market_trends",
                    input = "{{research_plan.market_query}}"
                },
                {
                    agent = "local://web_scraper",
                    action = "scrape_recent_news",
                    input = "{{research_plan.news_query}}"
                }
            }
        },
        {
            agent = "local://synthesis_agent",
            action = "synthesize_findings",
            input = "{{step_2.*}}"
        }
    }
})
```

## Cross-Engine Compatibility

Rs-llmspell ensures consistent behavior across different scripting engines through unified abstractions.

### Unified Async Interface

```rust
// Cross-engine promise abstraction
pub enum AsyncHandle {
    LuaCoroutine(LuaCoroutineHandle),
    JavaScriptPromise(JSPromiseHandle),
    PythonAwaitable(PyAwaitableHandle),
}

impl AsyncHandle {
    pub async fn await_result(&self) -> Result<ScriptValue> {
        match self {
            AsyncHandle::LuaCoroutine(handle) => {
                handle.resume_until_complete().await
            },
            AsyncHandle::JavaScriptPromise(handle) => {
                handle.await_promise().await
            },
            AsyncHandle::PythonAwaitable(handle) => {
                handle.await_coroutine().await
            }
        }
    }
    
    pub fn is_complete(&self) -> bool {
        match self {
            AsyncHandle::LuaCoroutine(handle) => handle.is_complete(),
            AsyncHandle::JavaScriptPromise(handle) => handle.is_resolved(),
            AsyncHandle::PythonAwaitable(handle) => handle.is_done(),
        }
    }
}

// Unified hook execution across engines
pub struct CrossEngineHookExecutor {
    lua_executor: LuaHookExecutor,
    js_executor: JSHookExecutor,
    python_executor: PyHookExecutor,
}

impl CrossEngineHookExecutor {
    pub async fn execute_hook(&self, hook: &dyn Hook, context: &HookContext) -> Result<HookResult> {
        match hook.engine_type() {
            ScriptEngine::Lua => {
                self.lua_executor.execute_lua_hook(hook, context).await
            },
            ScriptEngine::JavaScript => {
                self.js_executor.execute_js_hook(hook, context).await
            },
            ScriptEngine::Python => {
                self.python_executor.execute_py_hook(hook, context).await
            },
            ScriptEngine::Native => {
                // Direct Rust hook execution
                hook.execute(context).await
            }
        }
    }
}
```

### Consistent Error Handling

```lua
-- Lua error handling with rs-llmspell patterns
local success, result = pcall(function()
    return Agent.execute("research_agent", {
        query = "AI safety research trends",
        timeout = 30000
    })
end)

if not success then
    local error_info = ErrorHandler.parse_error(result)
    
    if error_info.type == "timeout" then
        print("Research timed out, using cached results")
        result = Cache.get_cached_research("ai_safety_trends")
    elseif error_info.type == "rate_limit" then
        print("Rate limited, waiting and retrying...")
        Async.sleep(error_info.retry_after or 5000)
        result = Agent.execute("research_agent", { query = "AI safety research trends" })
    else
        error("Unhandled error: " .. error_info.message)
    end
end
```

```javascript
// JavaScript error handling with consistent patterns
try {
    const result = await Agent.execute('research_agent', {
        query: 'AI safety research trends',
        timeout: 30000
    });
    
    return result;
} catch (error) {
    const errorInfo = ErrorHandler.parseError(error);
    
    switch (errorInfo.type) {
        case 'timeout':
            console.log('Research timed out, using cached results');
            return await Cache.getCachedResearch('ai_safety_trends');
            
        case 'rate_limit':
            console.log('Rate limited, waiting and retrying...');
            await Async.sleep(errorInfo.retryAfter || 5000);
            return await Agent.execute('research_agent', { 
                query: 'AI safety research trends' 
            });
            
        default:
            throw new Error(`Unhandled error: ${errorInfo.message}`);
    }
}
```

## Error Handling Strategy

Rs-llmspell implements comprehensive error handling with recovery strategies and cross-engine consistency.

### Hierarchical Error Recovery

```rust
// Error recovery coordinator
pub struct ErrorRecoveryCoordinator {
    recovery_strategies: HashMap<ErrorCategory, Vec<Box<dyn RecoveryStrategy>>>,
    circuit_breaker: CircuitBreaker,
    retry_policies: HashMap<String, RetryPolicy>,
    fallback_agents: HashMap<String, String>,
}

impl ErrorRecoveryCoordinator {
    pub async fn handle_error(&self, error: &LLMSpellError, 
                             context: &ErrorContext) -> Result<RecoveryAction> {
        let error_category = self.categorize_error(error);
        
        // Check circuit breaker state
        if self.circuit_breaker.should_block(&error_category) {
            return Ok(RecoveryAction::CircuitBreakerBlock {
                reason: "Too many recent failures".to_string(),
                retry_after: self.circuit_breaker.retry_after(),
            });
        }
        
        // Try recovery strategies in order of priority
        if let Some(strategies) = self.recovery_strategies.get(&error_category) {
            for strategy in strategies {
                match strategy.attempt_recovery(error, context).await {
                    Ok(RecoveryResult::Recovered(action)) => {
                        return Ok(action);
                    },
                    Ok(RecoveryResult::PartialRecovery(action)) => {
                        // Log partial recovery and continue
                        warn!("Partial recovery for {}: {:?}", error_category, action);
                        continue;
                    },
                    Ok(RecoveryResult::CannotRecover) => {
                        continue; // Try next strategy
                    },
                    Err(_) => {
                        continue; // Strategy failed, try next
                    }
                }
            }
        }
        
        // No recovery possible
        Ok(RecoveryAction::PropagateError {
            enhanced_error: self.enhance_error_info(error, context),
        })
    }
}

// Agent fallback strategy
pub struct AgentFallbackStrategy {
    fallback_mappings: HashMap<String, String>,
    capability_matcher: CapabilityMatcher,
}

#[async_trait]
impl RecoveryStrategy for AgentFallbackStrategy {
    async fn attempt_recovery(&self, error: &LLMSpellError, 
                            context: &ErrorContext) -> Result<RecoveryResult> {
        if let LLMSpellError::Agent(AgentError::ExecutionFailed { agent_id, .. }) = error {
            // Try direct fallback mapping first
            if let Some(fallback_id) = self.fallback_mappings.get(agent_id) {
                return Ok(RecoveryResult::Recovered(RecoveryAction::UseFallbackAgent {
                    original_agent: agent_id.clone(),
                    fallback_agent: fallback_id.clone(),
                }));
            }
            
            // Try capability-based fallback
            if let Some(original_agent) = context.get_agent(agent_id) {
                let required_capabilities = original_agent.capabilities();
                let fallback_agent = self.capability_matcher
                    .find_compatible_agent(required_capabilities)?;
                
                return Ok(RecoveryResult::Recovered(RecoveryAction::UseFallbackAgent {
                    original_agent: agent_id.clone(),
                    fallback_agent: fallback_agent.id().to_string(),
                }));
            }
        }
        
        Ok(RecoveryResult::CannotRecover)
    }
}
```

## Future Evolution

Rs-llmspell is designed for extensibility and future growth through well-defined extension points.

### Plugin System Architecture

```rust
// Plugin extension framework
pub trait Plugin: Send + Sync {
    fn plugin_info(&self) -> PluginInfo;
    fn compatible_versions(&self) -> VersionRange;
    fn required_features(&self) -> &[String];
    
    async fn initialize(&mut self, context: PluginInitContext) -> Result<()>;
    async fn shutdown(&mut self) -> Result<()>;
    
    fn health_check(&self) -> PluginHealth;
    fn metrics(&self) -> PluginMetrics;
}

// Example: Third-party LLM provider plugin
pub struct CustomLLMProviderPlugin {
    provider: CustomLLMProvider,
    config: ProviderConfig,
    metrics: ProviderMetrics,
}

impl Plugin for CustomLLMProviderPlugin {
    fn plugin_info(&self) -> PluginInfo {
        PluginInfo {
            name: "custom-llm-provider".to_string(),
            version: Version::new(1, 0, 0),
            description: "Custom LLM provider with advanced features".to_string(),
            author: "Third Party Developer".to_string(),
            license: "MIT".to_string(),
        }
    }
    
    fn compatible_versions(&self) -> VersionRange {
        VersionRange::new(
            Version::new(1, 0, 0),
            Version::new(2, 0, 0)
        )
    }
    
    async fn initialize(&mut self, context: PluginInitContext) -> Result<()> {
        // Register custom provider
        context.llm_registry.register_provider(
            "custom-llm",
            Box::new(self.provider.clone())
        )?;
        
        // Register custom tools
        for tool in self.provider.get_tools() {
            context.tool_registry.register_tool(tool)?;
        }
        
        Ok(())
    }
}
```

### Extension Points

- **New Script Engines**: Add support for additional programming languages
- **Protocol Extensions**: Implement new communication protocols and standards  
- **LLM Provider Extensions**: Integrate emerging AI providers and models
- **Tool Categories**: Create new categories of built-in tools and capabilities
- **Workflow Patterns**: Define new orchestration and coordination patterns
- **Storage Backends**: Add support for different persistence and caching systems

## Examples

### Simple Chat Agent

```lua
-- Lua
local agent = Agent.new("You are a helpful assistant")
local response = agent:chat("What is the weather like?")
print(response)
```

### Multi-Tool Research Agent

```javascript
// JavaScript
const researcher = new Agent({
    systemPrompt: "You are a research assistant",
    tools: [
        new WebSearch({ apiKey: process.env.SEARCH_KEY }),
        new Calculator(),
        new FileWriter({ baseDir: "./research" })
    ]
});

const report = await researcher.chat(
    "Research quantum computing and write a summary"
);
```

### Complex Workflow

```lua
-- Multi-agent blog creation
local workflow = Workflow.sequential({
    name = "blog_pipeline",
    steps = {
        { agent = researcher, action = "research", output = "notes" },
        { agent = writer, action = "draft", input = "{{notes}}" },
        { agent = editor, action = "edit", output = "final" }
    }
})

local blog = workflow:run({ topic = "AI Safety" })
```

## Implementation Roadmap

### Phase 1: Core Foundation (Weeks 1-2)
- Basic trait definitions
- Bridge layer for Lua/JS
- Simple agent implementation
- Core tool abstractions

### Phase 2: Infrastructure (Weeks 3-4)
- Hook system implementation
- Event bus architecture
- State management layer
- LLM provider integration (rig)

### Phase 3: Built-in Components (Weeks 5-6)
- Essential tools (10-15)
- Agent templates (3-5)
- Basic workflows
- Script APIs

### Phase 4: Production Features (Weeks 7-8)
- Full tool library
- Advanced workflows
- Performance optimization
- Security hardening

### Phase 5: Polish and Launch (Weeks 9-10)
- Documentation
- Examples and tutorials
- Performance benchmarks
- Community tooling

## Detailed Component Architecture

### BaseAgent/Agent/Tool/Workflow Hierarchy

#### BaseAgent: The Foundation

BaseAgent is the fundamental trait that defines any component capable of handling tools and maintaining state:

```rust
#[async_trait]
pub trait BaseAgent: Send + Sync + Observable + Hookable {
    /// Unique identifier for this agent
    fn id(&self) -> &str;
    
    /// Human-readable name
    fn name(&self) -> &str;
    
    /// Get available tools
    fn tools(&self) -> &[Box<dyn Tool>];
    
    /// Add a tool to this agent
    fn add_tool(&mut self, tool: Box<dyn Tool>) -> Result<()>;
    
    /// Execute with the given input
    async fn execute(&mut self, input: AgentInput) -> Result<AgentOutput>;
    
    /// Get current state
    fn state(&self) -> &AgentState;
    
    /// Handle state transition
    async fn transition(&mut self, event: StateEvent) -> Result<()>;
    
    /// Get agent metadata
    fn metadata(&self) -> &AgentMetadata;
}
```

**Key Characteristics**:
- Tool management capabilities
- State tracking and transitions
- Hook and event integration
- Async execution model
- Error handling and recovery

#### Agent: LLM-Powered Intelligence

Agent extends BaseAgent with LLM-specific capabilities:

```rust
#[async_trait]
pub trait Agent: BaseAgent {
    /// Get the LLM provider
    fn llm_provider(&self) -> &dyn LLMProvider;
    
    /// Get system prompt
    fn system_prompt(&self) -> &str;
    
    /// Update system prompt
    fn set_system_prompt(&mut self, prompt: String);
    
    /// Process a conversation turn
    async fn chat(&mut self, message: &str) -> Result<String>;
    
    /// Stream response
    async fn stream_chat(&mut self, message: &str) -> Result<ResponseStream>;
    
    /// Chat with additional context
    async fn chat_with_context(&mut self, message: &str, context: AgentContext) -> Result<String>;
    
    /// Get conversation history
    fn conversation_history(&self) -> &[ConversationTurn];
    
    /// Clear conversation history
    fn clear_history(&mut self);
}
```

**Implementation Example**:
```rust
pub struct ChatAgent {
    id: String,
    name: String,
    llm_provider: Box<dyn LLMProvider>,
    system_prompt: String,
    tools: Vec<Box<dyn Tool>>,
    state: AgentState,
    conversation_history: Vec<ConversationTurn>,
    hooks: HookManager,
    events: EventEmitter,
}

#[async_trait]
impl Agent for ChatAgent {
    async fn chat(&mut self, message: &str) -> Result<String> {
        // Execute pre-chat hooks
        self.execute_hooks(HookPoint::BeforeLLMCall, &mut HookContext {
            agent_id: self.id.clone(),
            message: message.to_string(),
            ..Default::default()
        }).await?;
        
        // Prepare completion request
        let request = CompletionRequest {
            messages: self.build_message_history(message),
            tools: self.tools.iter().map(|t| t.schema()).collect(),
            system: Some(self.system_prompt.clone()),
            ..Default::default()
        };
        
        // Call LLM provider
        let response = self.llm_provider.complete(request).await?;
        
        // Handle tool calls if any
        if let Some(tool_calls) = response.tool_calls {
            for tool_call in tool_calls {
                let result = self.execute_tool(&tool_call).await?;
                // Continue conversation with tool results...
            }
        }
        
        // Execute post-chat hooks
        self.execute_hooks(HookPoint::AfterLLMCall, &mut HookContext {
            agent_id: self.id.clone(),
            response: response.content.clone(),
            token_usage: response.usage,
            ..Default::default()
        }).await?;
        
        // Update conversation history
        self.conversation_history.push(ConversationTurn {
            user_message: message.to_string(),
            assistant_response: response.content.clone(),
            tool_calls: response.tool_calls,
            timestamp: Utc::now(),
        });
        
        Ok(response.content)
    }
}
```

#### Tool: Discrete Capabilities

Tools provide specific capabilities that agents can invoke:

```rust
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
    fn validate_params(&self, params: &Value) -> Result<()> {
        // Default implementation validates against schema
        validate_against_schema(params, &self.parameters_schema())
    }
    
    /// Tool metadata (version, author, etc.)
    fn metadata(&self) -> ToolMetadata {
        ToolMetadata::default()
    }
    
    /// Resource requirements
    fn resource_requirements(&self) -> ResourceRequirements {
        ResourceRequirements::default()
    }
}
```

**Tool Implementation Example**:
```rust
pub struct CalculatorTool;

#[async_trait]
impl Tool for CalculatorTool {
    fn name(&self) -> &str {
        "calculator"
    }
    
    fn description(&self) -> &str {
        "Perform mathematical calculations and return the result"
    }
    
    fn parameters_schema(&self) -> Value {
        json!({
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
    
    async fn execute(&self, params: Value) -> Result<ToolOutput> {
        let expression = params.get("expression")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow!("Missing expression parameter"))?;
            
        // Safe mathematical evaluation
        let result = evaluate_math_expression(expression)?;
        
        Ok(ToolOutput {
            content: json!({"result": result}),
            metadata: HashMap::from([
                ("expression".to_string(), Value::String(expression.to_string())),
                ("result_type".to_string(), Value::String("number".to_string())),
            ]),
        })
    }
}
```

#### Workflow: Orchestration Patterns

Workflows provide deterministic orchestration of agents and tools:

```rust
#[async_trait]
pub trait Workflow: BaseAgent {
    /// Workflow type identifier
    fn workflow_type(&self) -> WorkflowType;
    
    /// Get workflow steps
    fn steps(&self) -> &[WorkflowStep];
    
    /// Execute workflow
    async fn run(&mut self, input: WorkflowInput) -> Result<WorkflowOutput>;
    
    /// Pause workflow execution
    async fn pause(&mut self) -> Result<WorkflowCheckpoint>;
    
    /// Resume from checkpoint
    async fn resume(&mut self, checkpoint: WorkflowCheckpoint) -> Result<()>;
    
    /// Get execution progress
    fn progress(&self) -> WorkflowProgress;
}
```

**Sequential Workflow Example**:
```rust
pub struct SequentialWorkflow {
    id: String,
    name: String,
    steps: Vec<WorkflowStep>,
    state: WorkflowState,
    current_step: usize,
}

#[async_trait]
impl Workflow for SequentialWorkflow {
    async fn run(&mut self, input: WorkflowInput) -> Result<WorkflowOutput> {
        let mut context = WorkflowContext::new(input);
        
        for (index, step) in self.steps.iter().enumerate() {
            self.current_step = index;
            
            // Execute pre-step hooks
            self.execute_hooks(HookPoint::BeforeWorkflowStep, &mut HookContext {
                workflow_id: self.id.clone(),
                step_name: step.name.clone(),
                step_index: index,
                ..Default::default()
            }).await?;
            
            // Resolve step input from context
            let step_input = step.resolve_input(&context)?;
            
            // Execute step
            let step_output = match &step.executor {
                StepExecutor::Agent(agent) => {
                    agent.execute(step_input).await?
                },
                StepExecutor::Tool(tool) => {
                    tool.execute(step_input.into()).await?.into()
                },
                StepExecutor::Workflow(workflow) => {
                    workflow.run(step_input.into()).await?.into()
                },
            };
            
            // Store output in context
            if let Some(output_key) = &step.output_key {
                context.set(output_key, step_output.clone());
            }
            
            // Execute post-step hooks
            self.execute_hooks(HookPoint::AfterWorkflowStep, &mut HookContext {
                workflow_id: self.id.clone(),
                step_name: step.name.clone(),
                step_output: step_output.clone(),
                ..Default::default()
            }).await?;
        }
        
        Ok(WorkflowOutput {
            result: context.get_final_output(),
            metadata: context.metadata(),
            steps_executed: self.current_step + 1,
        })
    }
}
```

### Tool-Wrapped Agents

One of the most powerful patterns is wrapping agents as tools, enabling composition:

```rust
pub struct AgentAsTool<A: Agent> {
    agent: A,
    tool_name: String,
    tool_description: String,
}

impl<A: Agent> AgentAsTool<A> {
    pub fn new(agent: A, name: String, description: String) -> Self {
        Self {
            agent,
            tool_name: name,
            tool_description: description,
        }
    }
}

#[async_trait]
impl<A: Agent> Tool for AgentAsTool<A> {
    fn name(&self) -> &str {
        &self.tool_name
    }
    
    fn description(&self) -> &str {
        &self.tool_description
    }
    
    fn parameters_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "message": {
                    "type": "string",
                    "description": "Message to send to the agent"
                },
                "context": {
                    "type": "object",
                    "description": "Additional context for the agent"
                }
            },
            "required": ["message"]
        })
    }
    
    async fn execute(&self, params: Value) -> Result<ToolOutput> {
        let message = params.get("message")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow!("Missing message parameter"))?;
            
        let response = self.agent.chat(message).await?;
        
        Ok(ToolOutput {
            content: json!({"response": response}),
            metadata: HashMap::from([
                ("agent_id".to_string(), Value::String(self.agent.id().to_string())),
                ("agent_type".to_string(), Value::String("wrapped_agent".to_string())),
            ]),
        })
    }
}
```

**Usage Example**:
```lua
-- Lua: Create specialist agents and wrap them as tools
local code_expert = CodeAgent.new({
    languages = {"rust", "python"},
    style = "professional"
})

local research_expert = ResearchAgent.new({
    sources = {"academic", "documentation"}
})

-- Create orchestrator with specialist agents as tools
local orchestrator = Agent.new({
    system_prompt = "You coordinate specialist agents to solve complex problems",
    tools = {
        AgentTool.wrap(code_expert, "code_expert", "Expert in code generation and review"),
        AgentTool.wrap(research_expert, "research_expert", "Expert in research and analysis")
    }
})

-- The orchestrator can now use specialist agents as tools
local result = orchestrator:chat([[
    I need to implement a Rust web server. First research the best frameworks,
    then generate the code using best practices.
]])
```

### Composition Patterns

#### Hierarchical Agents
```rust
pub struct HierarchicalAgent {
    supervisor: Box<dyn Agent>,
    workers: HashMap<String, Box<dyn Agent>>,
    delegation_strategy: Box<dyn DelegationStrategy>,
    coordination_state: CoordinationState,
}

impl HierarchicalAgent {
    async fn delegate_task(&mut self, task: Task) -> Result<AgentOutput> {
        // Supervisor decides which worker should handle the task
        let worker_id = self.delegation_strategy.select_worker(&task, &self.workers).await?;
        
        let worker = self.workers.get_mut(&worker_id)
            .ok_or_else(|| anyhow!("Worker not found: {}", worker_id))?;
            
        // Execute task with selected worker
        let result = worker.execute(task.into()).await?;
        
        // Supervisor reviews and potentially refines the result
        self.supervisor.chat(&format!(
            "Review this result from {}: {}",
            worker_id, result.content
        )).await
    }
}
```

#### Pipeline Agents
```rust
pub struct PipelineAgent {
    stages: Vec<PipelineStage>,
    transformers: Vec<Box<dyn Transformer>>,
}

struct PipelineStage {
    agent: Box<dyn Agent>,
    transformer: Option<Box<dyn Transformer>>,
}

impl PipelineAgent {
    async fn process(&mut self, input: AgentInput) -> Result<AgentOutput> {
        let mut current_data = input;
        
        for stage in &mut self.stages {
            // Process with agent
            let result = stage.agent.execute(current_data).await?;
            
            // Transform for next stage
            current_data = if let Some(transformer) = &stage.transformer {
                transformer.transform(result).await?
            } else {
                result.into()
            };
        }
        
        Ok(current_data.into())
    }
}
```

## Comprehensive Examples

### BaseAgent Usage in Scripts

#### Custom BaseAgent Implementation (Lua)

```lua
-- Define a custom workflow agent that handles tools
local DataProcessingWorkflow = BaseAgent:extend()

function DataProcessingWorkflow:init(config)
    self.id = config.id or "data_processor"
    self.name = config.name or "Data Processing Workflow"
    self.tools = config.tools or {}
    self.state = {
        status = "ready",
        processed_items = 0,
        errors = {}
    }
    self.steps = config.steps or {}
end

function DataProcessingWorkflow:execute(input)
    return coroutine.wrap(function()
        self:set_state("processing")
        
        for i, step in ipairs(self.steps) do
            local step_result = self:execute_step(step, input)
            if step_result.error then
                table.insert(self.state.errors, step_result.error)
            else
                input = step_result.output  -- Chain outputs
                self.state.processed_items = self.state.processed_items + 1
            end
        end
        
        self:set_state("completed")
        return {
            processed_items = self.state.processed_items,
            errors = self.state.errors,
            final_output = input
        }
    end)
end

-- Usage
local processor = DataProcessingWorkflow.new({
    id = "csv_processor",
    tools = {
        CsvReader.new(),
        DataCleaner.new(),
        StatisticsCalculator.new(),
        ChartGenerator.new()
    },
    steps = {
        { tool = "csv-reader", action = "read" },
        { tool = "data-cleaner", action = "clean" },
        { tool = "statistics-calculator", action = "analyze" },
        { tool = "chart-generator", action = "visualize" }
    }
})

local result = processor:execute({ file_path = "data.csv" })
```

#### BaseAgent Composition (JavaScript)

```javascript
// Custom BaseAgent for orchestrating multiple agents
class MultiAgentOrchestrator extends BaseAgent {
    constructor(config) {
        super(config);
        this.agents = new Map();
        this.delegationStrategy = config.delegationStrategy || 'round_robin';
        this.activeJobs = new Map();
    }
    
    addAgent(name, agent) {
        this.agents.set(name, agent);
        // Wrap agent as a tool for this orchestrator
        this.addTool(new AgentTool(agent, name));
    }
    
    async execute(input) {
        // Determine which agents should handle this input
        const selectedAgents = await this.selectAgents(input);
        
        if (selectedAgents.length === 1) {
            // Single agent execution
            return await this.executeSingleAgent(selectedAgents[0], input);
        } else {
            // Multi-agent coordination
            return await this.executeMultiAgent(selectedAgents, input);
        }
    }
    
    async selectAgents(input) {
        // Custom logic to determine appropriate agents
        if (input.task_type === 'research') {
            return ['researcher', 'fact_checker'];
        } else if (input.task_type === 'development') {
            return ['code_generator', 'tester', 'reviewer'];
        }
        return ['general_assistant'];
    }
    
    async executeMultiAgent(agentNames, input) {
        const jobs = agentNames.map(async (name) => {
            const agent = this.agents.get(name);
            const jobId = this.generateJobId();
            
            this.activeJobs.set(jobId, { agent: name, status: 'running' });
            
            try {
                const result = await agent.execute(input);
                this.activeJobs.set(jobId, { agent: name, status: 'completed' });
                return { agent: name, result, jobId };
            } catch (error) {
                this.activeJobs.set(jobId, { agent: name, status: 'failed', error });
                throw error;
            }
        });
        
        const results = await Promise.all(jobs);
        return this.aggregateResults(results);
    }
}

// Usage
const orchestrator = new MultiAgentOrchestrator({
    id: 'main_orchestrator',
    delegationStrategy: 'skill_based'
});

orchestrator.addAgent('researcher', new ResearchAgent());
orchestrator.addAgent('writer', new WriterAgent());
orchestrator.addAgent('reviewer', new ReviewAgent());

const result = await orchestrator.execute({
    task_type: 'research',
    query: 'Latest developments in quantum computing'
});
```

### Hook Registration and Usage

#### Global Hook Registration (Lua)

```lua
-- Global hook for performance monitoring
Hooks.register("before_llm_call", function(context)
    context.start_time = os.clock()
    context.request_id = uuid.generate()
    
    print(string.format("[%s] Starting LLM call for agent %s",
        context.request_id, context.agent_id))
    
    -- Add to metrics
    metrics:increment("llm_calls_started", {
        agent_type = context.agent_type,
        provider = context.provider
    })
end)

Hooks.register("after_llm_call", function(context)
    local duration = os.clock() - context.start_time
    
    print(string.format("[%s] LLM call completed in %.3fs, tokens: %d",
        context.request_id, duration, context.token_usage.total))
    
    -- Add to metrics
    metrics:histogram("llm_call_duration", duration, {
        agent_type = context.agent_type,
        provider = context.provider
    })
    
    metrics:histogram("llm_tokens_used", context.token_usage.total, {
        agent_type = context.agent_type,
        provider = context.provider
    })
end)

-- Error handling hook
Hooks.register("on_error", function(context)
    -- Log error with full context
    logger:error({
        message = "Agent execution failed",
        agent_id = context.agent_id,
        error = context.error,
        stack_trace = context.stack_trace,
        input = context.input,
        request_id = context.request_id
    })
    
    -- Implement retry logic
    if should_retry(context.error) and (context.retry_count or 0) < 3 then
        context.retry = true
        context.retry_count = (context.retry_count or 0) + 1
        context.retry_delay = math.pow(2, context.retry_count) * 1000  -- Exponential backoff
        
        print(string.format("Retrying in %dms (attempt %d/3)",
            context.retry_delay, context.retry_count))
    end
end)
```

#### Agent-Specific Hooks (JavaScript)

```javascript
// Create agent with comprehensive hooks
const agent = new ChatAgent({
    systemPrompt: "You are a helpful assistant",
    provider: "openai",
    hooks: {
        // Input validation and sanitization
        beforeExecute: async (context) => {
            // Sanitize input
            context.input.message = sanitizeInput(context.input.message);
            
            // Check rate limits
            const rateLimitKey = `user:${context.userId}`;
            if (await rateLimiter.isExceeded(rateLimitKey)) {
                throw new RateLimitError("Too many requests");
            }
            
            // Add user context
            context.userProfile = await getUserProfile(context.userId);
        },
        
        // Content filtering
        beforeLLMCall: async (context) => {
            // Check for inappropriate content
            const contentFlags = await contentModerator.check(context.prompt);
            if (contentFlags.inappropriate) {
                throw new ContentViolationError("Inappropriate content detected");
            }
            
            // Add safety instructions to prompt
            context.prompt = addSafetyInstructions(context.prompt);
        },
        
        // Response processing
        afterLLMCall: async (context) => {
            // Check response safety
            const responseFlags = await contentModerator.check(context.response);
            if (responseFlags.inappropriate) {
                context.response = "I can't assist with that request.";
            }
            
            // Add citations if needed
            if (context.toolCalls.some(tc => tc.name === 'web_search')) {
                context.response = await addCitations(context.response);
            }
        },
        
        // Audit logging
        afterExecute: async (context) => {
            await auditLogger.log({
                userId: context.userId,
                agentId: context.agentId,
                input: context.input,
                output: context.output,
                toolsUsed: context.toolCalls.map(tc => tc.name),
                duration: context.duration,
                tokensUsed: context.tokenUsage,
                timestamp: new Date().toISOString()
            });
        },
        
        // Error handling with user notification
        onError: async (context) => {
            // Log error
            console.error('Agent execution failed:', {
                agentId: context.agentId,
                error: context.error.message,
                userId: context.userId
            });
            
            // Notify user of generic error
            context.userMessage = "I encountered an error. Please try again.";
            
            // Send error to monitoring system
            await errorReporter.report(context.error, {
                agentId: context.agentId,
                userId: context.userId,
                context: context.sanitizedContext
            });
        }
    }
});
```

### Event-Driven Workflow Examples

#### Event-Based Multi-Agent Coordination (Lua)

```lua
-- Event-driven research workflow
local research_system = EventSystem.new()

-- Define agents
local search_agent = ResearchAgent.new({ sources = {"web", "academic"} })
local analysis_agent = AnalysisAgent.new({ tools = {StatisticsTool.new()} })
local writing_agent = WriterAgent.new({ style = "academic" })
local review_agent = ReviewAgent.new({ criteria = {"accuracy", "clarity"} })

-- Event handlers for workflow coordination
research_system:on("research_requested", function(event)
    print("Starting research on: " .. event.query)
    
    -- Start search
    search_agent:search_async(event.query, function(results)
        research_system:emit("search_completed", {
            query = event.query,
            results = results,
            request_id = event.request_id
        })
    end)
end)

research_system:on("search_completed", function(event)
    print("Search completed, starting analysis...")
    
    analysis_agent:analyze_async(event.results, function(analysis)
        research_system:emit("analysis_completed", {
            query = event.query,
            raw_results = event.results,
            analysis = analysis,
            request_id = event.request_id
        })
    end)
end)

research_system:on("analysis_completed", function(event)
    print("Analysis completed, generating report...")
    
    writing_agent:write_async({
        topic = event.query,
        research = event.raw_results,
        analysis = event.analysis
    }, function(draft)
        research_system:emit("draft_completed", {
            query = event.query,
            draft = draft,
            request_id = event.request_id
        })
    end)
end)

research_system:on("draft_completed", function(event)
    print("Draft completed, starting review...")
    
    review_agent:review_async(event.draft, function(review)
        if review.approved then
            research_system:emit("research_completed", {
                query = event.query,
                final_report = event.draft,
                review = review,
                request_id = event.request_id
            })
        else
            -- Send back for revision
            research_system:emit("revision_needed", {
                query = event.query,
                draft = event.draft,
                feedback = review.feedback,
                request_id = event.request_id
            })
        end
    end)
end)

research_system:on("revision_needed", function(event)
    print("Revision needed, sending back to writer...")
    
    writing_agent:revise_async({
        draft = event.draft,
        feedback = event.feedback
    }, function(revised_draft)
        research_system:emit("draft_completed", {
            query = event.query,
            draft = revised_draft,
            request_id = event.request_id
        })
    end)
end)

-- Start the research process
research_system:emit("research_requested", {
    query = "Impact of AI on software development",
    request_id = uuid.generate()
})
```

#### Real-Time Event Streaming (JavaScript)

```javascript
// Real-time agent activity monitoring
class AgentActivityMonitor {
    constructor() {
        this.eventStream = new EventEmitter();
        this.activeAgents = new Map();
        this.metrics = new MetricsCollector();
    }
    
    // Monitor all agents in the system
    monitorAgent(agent) {
        const agentId = agent.id;
        this.activeAgents.set(agentId, {
            agent,
            status: 'idle',
            lastActivity: Date.now()
        });
        
        // Subscribe to all agent events
        agent.on('*', (eventType, data) => {
            this.handleAgentEvent(agentId, eventType, data);
        });
    }
    
    handleAgentEvent(agentId, eventType, data) {
        const timestamp = Date.now();
        
        // Update agent status
        const agentInfo = this.activeAgents.get(agentId);
        agentInfo.lastActivity = timestamp;
        
        switch (eventType) {
            case 'execution_started':
                agentInfo.status = 'executing';
                this.metrics.increment('agent_executions_started');
                break;
                
            case 'tool_called':
                this.metrics.increment('tool_calls', {
                    tool: data.toolName,
                    agent: agentId
                });
                break;
                
            case 'llm_call_started':
                agentInfo.status = 'waiting_for_llm';
                this.metrics.increment('llm_calls');
                break;
                
            case 'execution_completed':
                agentInfo.status = 'idle';
                this.metrics.histogram('execution_duration', data.duration);
                break;
                
            case 'error':
                agentInfo.status = 'error';
                this.metrics.increment('agent_errors', {
                    agent: agentId,
                    error_type: data.error.name
                });
                break;
        }
        
        // Emit aggregated event
        this.eventStream.emit('agent_activity', {
            agentId,
            eventType,
            data,
            timestamp,
            agentStatus: agentInfo.status
        });
    }
    
    // Real-time dashboard data
    getDashboardData() {
        const agentStats = Array.from(this.activeAgents.entries()).map(([id, info]) => ({
            id,
            status: info.status,
            lastActivity: info.lastActivity,
            name: info.agent.name
        }));
        
        return {
            agents: agentStats,
            metrics: this.metrics.getSnapshot(),
            timestamp: Date.now()
        };
    }
    
    // Set up real-time streaming to client
    streamToClient(websocket) {
        const handler = (data) => {
            websocket.send(JSON.stringify({
                type: 'agent_activity',
                data
            }));
        };
        
        this.eventStream.on('agent_activity', handler);
        
        // Send periodic dashboard updates
        const dashboardInterval = setInterval(() => {
            websocket.send(JSON.stringify({
                type: 'dashboard_update',
                data: this.getDashboardData()
            }));
        }, 1000);
        
        // Cleanup on disconnect
        websocket.on('close', () => {
            this.eventStream.off('agent_activity', handler);
            clearInterval(dashboardInterval);
        });
    }
}

// Usage
const monitor = new AgentActivityMonitor();

// Monitor all agents
monitor.monitorAgent(chatAgent);
monitor.monitorAgent(researchAgent);
monitor.monitorAgent(codeAgent);

// Set up real-time monitoring
monitor.eventStream.on('agent_activity', (activity) => {
    console.log(`Agent ${activity.agentId}: ${activity.eventType}`);
    
    // Trigger alerts for errors
    if (activity.eventType === 'error') {
        alertSystem.notify(`Agent ${activity.agentId} encountered an error`);
    }
});
```

### Built-in Tool Usage Examples

#### Tool Composition and Chaining (Lua)

```lua
-- Advanced tool usage with composition
local research_assistant = Agent.new({
    system_prompt = "You are a research assistant with access to various tools",
    tools = {
        -- Web and data tools
        WebSearch.new({ 
            api_key = config.search_api_key,
            max_results = 10 
        }),
        WebScraper.new({ 
            rate_limit = { requests = 5, per = "minute" },
            user_agent = "ResearchBot/1.0"
        }),
        
        -- Data processing tools
        CsvTool.new(),
        JsonTool.new(),
        StatisticsTool.new(),
        
        -- File and document tools
        FileSystemTool.new({ 
            allowed_paths = {"./research", "./data"},
            read_only = false 
        }),
        MarkdownTool.new(),
        
        -- AI-powered tools
        EmbeddingTool.new({ 
            model = "text-embedding-ada-002",
            api_key = config.openai_api_key 
        }),
        VectorSearchTool.new({ 
            index_path = "./vector_index",
            similarity_threshold = 0.8 
        }),
        
        -- Communication tools
        EmailTool.new({ 
            smtp_server = config.smtp_server,
            from_address = config.email_from 
        }),
        SlackTool.new({ 
            token = config.slack_token,
            channel = "#research-updates" 
        })
    }
})

-- Complex research workflow using multiple tools
local function conduct_research(topic)
    return research_assistant:chat(string.format([[
        Research the topic "%s" using the following process:
        
        1. Use web_search to find recent articles and papers
        2. Use web_scraper to extract full content from promising sources
        3. Save raw research data to "./research/%s_raw.json"
        4. Use statistics_tool to analyze any numerical data found
        5. Generate embeddings for key findings
        6. Search for related content in our vector database
        7. Create a comprehensive markdown report at "./research/%s_report.md"
        8. Send a summary to the #research-updates Slack channel
        
        Be thorough and cite all sources properly.
    ]], topic, topic:gsub("%s+", "_"), topic:gsub("%s+", "_")))
end

-- Execute research
local result = conduct_research("Quantum Computing Applications in Finance")
print("Research completed:", result)
```

#### Tool Security and Validation (JavaScript)

```javascript
// Secure tool usage with validation and sandboxing
class SecureAgent extends Agent {
    constructor(config) {
        super(config);
        this.setupSecureTools();
    }
    
    setupSecureTools() {
        // File system tool with strict security
        this.addTool(new FileSystemTool({
            allowedPaths: ['/tmp/agent_workspace', './output'],
            maxFileSize: 10 * 1024 * 1024, // 10MB
            allowedExtensions: ['.txt', '.json', '.csv', '.md'],
            readOnly: false,
            hooks: {
                beforeExecute: (context) => {
                    // Additional path validation
                    const path = context.params.path;
                    if (path.includes('..') || path.startsWith('/')) {
                        throw new SecurityError('Invalid path detected');
                    }
                    
                    // Log all file operations
                    this.auditLogger.log('file_operation', {
                        operation: context.params.operation,
                        path: path,
                        agentId: this.id,
                        timestamp: new Date().toISOString()
                    });
                }
            }
        }));
        
        // HTTP tool with URL validation
        this.addTool(new HttpRequestTool({
            allowedDomains: [
                'api.openweathermap.org',
                'jsonplaceholder.typicode.com',
                'httpbin.org'
            ],
            timeout: 30000,
            maxResponseSize: 5 * 1024 * 1024, // 5MB
            hooks: {
                beforeExecute: async (context) => {
                    const url = context.params.url;
                    
                    // Validate URL against allow list
                    const domain = new URL(url).hostname;
                    if (!this.isAllowedDomain(domain)) {
                        throw new SecurityError(`Domain not allowed: ${domain}`);
                    }
                    
                    // Rate limiting
                    await this.rateLimiter.checkLimit(`http:${domain}`, {
                        requests: 10,
                        per: 'minute'
                    });
                }
            }
        }));
        
        // Shell command tool with command whitelist
        this.addTool(new ShellCommandTool({
            allowedCommands: [
                'ls', 'cat', 'head', 'tail', 'wc',
                'grep', 'awk', 'sed',
                'python3 ./scripts/*.py',
                'node ./scripts/*.js'
            ],
            timeout: 60000,
            workingDirectory: '/tmp/agent_workspace',
            hooks: {
                beforeExecute: (context) => {
                    const command = context.params.command;
                    
                    if (!this.isCommandAllowed(command)) {
                        throw new SecurityError(`Command not allowed: ${command}`);
                    }
                    
                    // Log all command executions
                    this.auditLogger.log('shell_command', {
                        command: command,
                        agentId: this.id,
                        timestamp: new Date().toISOString()
                    });
                }
            }
        }));
        
        // Email tool with template validation
        this.addTool(new EmailTool({
            smtpConfig: {
                host: process.env.SMTP_HOST,
                port: 587,
                secure: false,
                auth: {
                    user: process.env.SMTP_USER,
                    pass: process.env.SMTP_PASS
                }
            },
            allowedRecipients: process.env.ALLOWED_EMAIL_DOMAINS?.split(',') || [],
            templatePath: './email_templates',
            hooks: {
                beforeExecute: async (context) => {
                    // Validate recipient
                    const recipient = context.params.to;
                    if (!this.isEmailAllowed(recipient)) {
                        throw new SecurityError(`Email recipient not allowed: ${recipient}`);
                    }
                    
                    // Content filtering
                    const subject = context.params.subject;
                    const body = context.params.body;
                    
                    const contentCheck = await this.contentModerator.check(subject + ' ' + body);
                    if (contentCheck.flagged) {
                        throw new ContentViolationError('Email content violates policy');
                    }
                }
            }
        }));
    }
    
    isAllowedDomain(domain) {
        return this.tools.find(t => t.name === 'http_request')
            .config.allowedDomains.includes(domain);
    }
    
    isCommandAllowed(command) {
        const allowedCommands = this.tools.find(t => t.name === 'shell_command')
            .config.allowedCommands;
            
        return allowedCommands.some(pattern => {
            if (pattern.includes('*')) {
                // Simple glob pattern matching
                const regex = new RegExp('^' + pattern.replace(/\*/g, '.*') + '$');
                return regex.test(command);
            }
            return command === pattern;
        });
    }
    
    isEmailAllowed(email) {
        const allowedDomains = this.tools.find(t => t.name === 'email')
            .config.allowedRecipients;
            
        const domain = email.split('@')[1];
        return allowedDomains.includes(domain);
    }
}

// Usage with comprehensive security
const secureAgent = new SecureAgent({
    systemPrompt: "You are a secure assistant with restricted tool access",
    auditLogger: new AuditLogger('./logs/agent_audit.log'),
    rateLimiter: new RateLimiter(),
    contentModerator: new ContentModerator()
});
```

### Async Pattern Examples

#### Cooperative Scheduling (Lua)

```lua
-- Lua coroutine-based async patterns
local AsyncScheduler = {}
AsyncScheduler.__index = AsyncScheduler

function AsyncScheduler.new()
    return setmetatable({
        tasks = {},
        running = false
    }, AsyncScheduler)
end

function AsyncScheduler:add_task(name, task_func)
    local task = coroutine.create(task_func)
    self.tasks[name] = {
        coroutine = task,
        status = "ready"
    }
end

function AsyncScheduler:run()
    self.running = true
    
    while self.running and next(self.tasks) do
        for name, task in pairs(self.tasks) do
            if task.status == "ready" or task.status == "running" then
                local success, result = coroutine.resume(task.coroutine)
                
                if success then
                    if coroutine.status(task.coroutine) == "dead" then
                        task.status = "completed"
                        task.result = result
                        print(string.format("Task %s completed: %s", name, tostring(result)))
                        self.tasks[name] = nil  -- Remove completed task
                    else
                        task.status = "running"
                    end
                else
                    task.status = "error"
                    task.error = result
                    print(string.format("Task %s error: %s", name, result))
                    self.tasks[name] = nil  -- Remove failed task
                end
            end
        end
        
        -- Yield control briefly
        coroutine.yield()
    end
end

-- Example: Multiple agents running concurrently
local scheduler = AsyncScheduler.new()

-- Research task
scheduler:add_task("research", function()
    local research_agent = ResearchAgent.new()
    
    for i = 1, 5 do
        print("Research step", i)
        local result = research_agent:search("AI topic " .. i)
        
        -- Yield periodically to allow other tasks to run
        coroutine.yield()
        
        -- Process result
        research_agent:process(result)
    end
    
    return "Research completed"
end)

-- Analysis task  
scheduler:add_task("analysis", function()
    local analysis_agent = AnalysisAgent.new()
    
    for i = 1, 3 do
        print("Analysis step", i)
        local data = analysis_agent:fetch_data("dataset_" .. i)
        
        -- Yield to other tasks
        coroutine.yield()
        
        local insights = analysis_agent:analyze(data)
        analysis_agent:store_insights(insights)
    end
    
    return "Analysis completed"
end)

-- Writing task
scheduler:add_task("writing", function()
    local writer_agent = WriterAgent.new()
    
    local sections = {"intro", "body", "conclusion"}
    for _, section in ipairs(sections) do
        print("Writing", section)
        local content = writer_agent:write_section(section)
        
        -- Yield after each section
        coroutine.yield()
        
        writer_agent:save_section(section, content)
    end
    
    return "Writing completed"
end)

-- Run all tasks cooperatively
print("Starting cooperative execution...")
scheduler:run()
print("All tasks completed!")
```

#### Promise-Style Async (JavaScript)

```javascript
// Advanced async patterns with Promise coordination
class AsyncAgentCoordinator {
    constructor() {
        this.agents = new Map();
        this.taskQueue = [];
        this.activeJobs = new Map();
        this.maxConcurrency = 3;
    }
    
    // Register agents for coordination
    registerAgent(name, agent) {
        this.agents.set(name, agent);
    }
    
    // Execute multiple agents with different concurrency patterns
    async executeParallel(tasks) {
        console.log(`Executing ${tasks.length} tasks in parallel...`);
        
        const promises = tasks.map(async (task) => {
            const agent = this.agents.get(task.agentName);
            if (!agent) {
                throw new Error(`Agent not found: ${task.agentName}`);
            }
            
            try {
                const result = await agent.execute(task.input);
                return { task: task.name, result, success: true };
            } catch (error) {
                return { task: task.name, error, success: false };
            }
        });
        
        return await Promise.allSettled(promises);
    }
    
    // Execute tasks with controlled concurrency
    async executeWithConcurrencyLimit(tasks) {
        console.log(`Executing ${tasks.length} tasks with concurrency limit ${this.maxConcurrency}...`);
        
        const results = [];
        const executing = new Set();
        
        for (const task of tasks) {
            // Wait if we've hit the concurrency limit
            if (executing.size >= this.maxConcurrency) {
                await Promise.race(executing);
            }
            
            const promise = this.executeTask(task).finally(() => {
                executing.delete(promise);
            });
            
            executing.add(promise);
            results.push(promise);
        }
        
        return await Promise.all(results);
    }
    
    // Execute task with timeout and retry
    async executeTask(task) {
        const agent = this.agents.get(task.agentName);
        let lastError;
        
        for (let attempt = 1; attempt <= (task.retries || 1); attempt++) {
            try {
                console.log(`Executing ${task.name} (attempt ${attempt})`);
                
                const timeoutPromise = new Promise((_, reject) => {
                    setTimeout(() => reject(new Error('Task timeout')), task.timeout || 30000);
                });
                
                const taskPromise = agent.execute(task.input);
                
                const result = await Promise.race([taskPromise, timeoutPromise]);
                
                console.log(`Task ${task.name} completed successfully`);
                return { task: task.name, result, attempt, success: true };
                
            } catch (error) {
                console.warn(`Task ${task.name} failed on attempt ${attempt}:`, error.message);
                lastError = error;
                
                if (attempt < (task.retries || 1)) {
                    // Exponential backoff
                    const delay = Math.pow(2, attempt - 1) * 1000;
                    console.log(`Retrying ${task.name} in ${delay}ms...`);
                    await new Promise(resolve => setTimeout(resolve, delay));
                }
            }
        }
        
        return { task: task.name, error: lastError, success: false };
    }
    
    // Stream results as they complete
    async *executeStream(tasks) {
        const promises = tasks.map(task => this.executeTask(task));
        
        // Yield results as they complete
        while (promises.length > 0) {
            const { value, index } = await this.raceWithIndex(promises);
            promises.splice(index, 1);
            yield value;
        }
    }
    
    // Helper to get Promise.race result with index
    async raceWithIndex(promises) {
        return new Promise((resolve) => {
            promises.forEach((promise, index) => {
                promise.then((value) => resolve({ value, index }));
            });
        });
    }
    
    // Pipeline execution - output of one task becomes input to next
    async executePipeline(pipeline) {
        console.log(`Executing pipeline with ${pipeline.steps.length} steps...`);
        
        let currentInput = pipeline.initialInput;
        const results = [];
        
        for (const [index, step] of pipeline.steps.entries()) {
            console.log(`Pipeline step ${index + 1}: ${step.name}`);
            
            const agent = this.agents.get(step.agentName);
            if (!agent) {
                throw new Error(`Agent not found: ${step.agentName}`);
            }
            
            try {
                // Transform input if transformer is provided
                const stepInput = step.inputTransformer 
                    ? step.inputTransformer(currentInput)
                    : currentInput;
                
                const result = await agent.execute(stepInput);
                results.push({ step: step.name, result });
                
                // Output becomes input for next step
                currentInput = step.outputTransformer 
                    ? step.outputTransformer(result)
                    : result;
                    
            } catch (error) {
                console.error(`Pipeline failed at step ${step.name}:`, error);
                
                if (step.continueOnError) {
                    results.push({ step: step.name, error, skipped: true });
                    // Use fallback or previous input
                    currentInput = step.fallbackOutput || currentInput;
                } else {
                    throw error;
                }
            }
        }
        
        return {
            finalOutput: currentInput,
            stepResults: results
        };
    }
}

// Usage examples
const coordinator = new AsyncAgentCoordinator();

coordinator.registerAgent('research', new ResearchAgent());
coordinator.registerAgent('analysis', new AnalysisAgent());
coordinator.registerAgent('writer', new WriterAgent());
coordinator.registerAgent('reviewer', new ReviewAgent());

// Parallel execution
const parallelTasks = [
    { name: 'search_papers', agentName: 'research', input: { query: 'AI safety' } },
    { name: 'search_news', agentName: 'research', input: { query: 'AI regulation' } },
    { name: 'search_blogs', agentName: 'research', input: { query: 'AI ethics' } }
];

const parallelResults = await coordinator.executeParallel(parallelTasks);

// Streaming execution with real-time updates
console.log('Streaming results:');
for await (const result of coordinator.executeStream(parallelTasks)) {
    console.log(`Completed: ${result.task}`, result.success ? '✓' : '✗');
    if (result.success) {
        console.log('Result:', result.result);
    } else {
        console.error('Error:', result.error.message);
    }
}

// Pipeline execution
const researchPipeline = {
    initialInput: { topic: 'Quantum Computing in Finance' },
    steps: [
        {
            name: 'research',
            agentName: 'research',
            inputTransformer: (input) => ({ query: input.topic }),
            outputTransformer: (output) => ({ researchData: output })
        },
        {
            name: 'analyze',
            agentName: 'analysis',
            inputTransformer: (input) => input.researchData,
            outputTransformer: (output) => ({ analysis: output })
        },
        {
            name: 'write',
            agentName: 'writer',
            inputTransformer: (input) => ({ 
                topic: 'Quantum Computing in Finance',
                analysis: input.analysis 
            }),
            outputTransformer: (output) => ({ draft: output })
        },
        {
            name: 'review',
            agentName: 'reviewer',
            inputTransformer: (input) => input.draft,
            continueOnError: true,
            fallbackOutput: { approved: true, feedback: 'Auto-approved due to review error' }
        }
    ]
};

const pipelineResult = await coordinator.executePipeline(researchPipeline);
console.log('Pipeline completed:', pipelineResult.finalOutput);
```

These comprehensive examples demonstrate the full power and flexibility of the rs-llmspell architecture, showing how components can be composed, coordinated, and extended to build sophisticated AI applications.

## Directory Structure and Crate Organization

### Crate Architecture

Rs-LLMSpell follows a modular crate structure that mirrors the component architecture:

```
rs-llmspell/
├── Cargo.toml                     # Workspace definition
├── README.md
├── TODO.md
├── docs/
│   └── technical/                  # Architecture documentation
│
├── llmspell-core/                  # Core traits and types
│   ├── Cargo.toml
│   └── src/
│       ├── traits/
│       │   ├── base_agent.rs       # BaseAgent trait
│       │   ├── agent.rs            # Agent trait  
│       │   ├── tool.rs             # Tool trait
│       │   ├── workflow.rs         # Workflow trait
│       │   ├── observable.rs       # Observable trait
│       │   ├── hookable.rs         # Hookable trait
│       │   └── mod.rs
│       ├── types/
│       │   ├── script_value.rs     # ScriptValue enum
│       │   ├── agent_state.rs      # AgentState types
│       │   ├── tool_types.rs       # Tool input/output types
│       │   ├── workflow_types.rs   # Workflow types
│       │   └── mod.rs
│       ├── error.rs                # Error types
│       └── lib.rs
│
├── llmspell-bridge/                # Language bindings
│   ├── Cargo.toml
│   └── src/
│       ├── lua/
│       │   ├── agent_binding.rs    # Lua Agent bindings
│       │   ├── tool_binding.rs     # Lua Tool bindings
│       │   ├── workflow_binding.rs # Lua Workflow bindings
│       │   ├── async_support.rs    # Lua coroutine/Promise
│       │   ├── event_binding.rs    # Lua event handling
│       │   └── mod.rs
│       ├── js/
│       │   ├── agent_binding.rs    # JS Agent bindings
│       │   ├── tool_binding.rs     # JS Tool bindings
│       │   ├── workflow_binding.rs # JS Workflow bindings
│       │   ├── async_support.rs    # JS Promise integration
│       │   ├── event_binding.rs    # JS event handling
│       │   └── mod.rs
│       ├── convert.rs              # Type conversions
│       ├── async_bridge.rs         # Async coordination
│       ├── error_bridge.rs         # Error handling
│       └── lib.rs
│
├── llmspell-infra/                 # Infrastructure layer
│   ├── Cargo.toml
│   └── src/
│       ├── llm/
│       │   ├── provider.rs         # LLMProvider trait
│       │   ├── rig_wrapper.rs      # Rig integration
│       │   ├── local_provider.rs   # Candle integration
│       │   └── mod.rs
│       ├── state/
│       │   ├── store.rs            # StateStore trait
│       │   ├── sled_store.rs       # Sled implementation
│       │   ├── rocksdb_store.rs    # RocksDB implementation
│       │   ├── handoff.rs          # State handoff
│       │   └── mod.rs
│       ├── events/
│       │   ├── bus.rs              # EventBus implementation
│       │   ├── types.rs            # Event types
│       │   ├── handler.rs          # EventHandler trait
│       │   ├── stream.rs           # Event streaming
│       │   └── mod.rs
│       ├── hooks/
│       │   ├── manager.rs          # HookManager
│       │   ├── context.rs          # HookContext
│       │   ├── builtin/            # Built-in hooks
│       │   │   ├── logging.rs      # LoggingHook
│       │   │   ├── metrics.rs      # MetricsHook
│       │   │   ├── tracing.rs      # TracingHook
│       │   │   ├── rate_limit.rs   # RateLimitHook
│       │   │   ├── cache.rs        # CacheHook
│       │   │   ├── retry.rs        # RetryHook
│       │   │   └── mod.rs
│       │   └── mod.rs
│       ├── security/
│       │   ├── sandbox.rs          # Script sandboxing
│       │   ├── audit.rs            # Audit logging
│       │   ├── permissions.rs      # Permission system
│       │   └── mod.rs
│       └── lib.rs
│
├── llmspell-agents/                # Agent implementations
│   ├── Cargo.toml
│   └── src/
│       ├── chat.rs                 # ChatAgent
│       ├── research.rs             # ResearchAgent
│       ├── code.rs                 # CodeAgent
│       ├── data.rs                 # DataAnalyst
│       ├── planner.rs              # PlannerAgent
│       ├── orchestrator.rs         # OrchestratorAgent
│       ├── base_impl.rs            # Common implementations
│       └── lib.rs
│
├── llmspell-tools/                 # Built-in tools
│   ├── Cargo.toml
│   └── src/
│       ├── system/
│       │   ├── filesystem.rs       # FileSystemTool
│       │   ├── shell.rs            # ShellCommandTool
│       │   ├── environment.rs      # EnvironmentTool
│       │   ├── process.rs          # ProcessTool
│       │   └── mod.rs
│       ├── data/
│       │   ├── json.rs             # JsonTool
│       │   ├── xml.rs              # XmlTool
│       │   ├── csv.rs              # CsvTool
│       │   ├── sqlite.rs           # SqliteTool
│       │   ├── regex.rs            # RegexTool
│       │   └── mod.rs
│       ├── web/
│       │   ├── http.rs             # HttpRequestTool
│       │   ├── scraper.rs          # WebScraperTool
│       │   ├── rss.rs              # RssFeedTool
│       │   ├── websocket.rs        # WebSocketTool
│       │   └── mod.rs
│       ├── ai/
│       │   ├── embedding.rs        # EmbeddingTool
│       │   ├── vector_search.rs    # VectorSearchTool
│       │   ├── image_gen.rs        # ImageGenerationTool
│       │   ├── transcribe.rs       # AudioTranscribeTool
│       │   ├── code_interpreter.rs # CodeInterpreterTool
│       │   └── mod.rs
│       ├── communication/
│       │   ├── email.rs            # EmailTool
│       │   ├── slack.rs            # SlackTool
│       │   ├── discord.rs          # DiscordTool
│       │   ├── twilio.rs           # TwilioTool
│       │   └── mod.rs
│       ├── math/
│       │   ├── calculator.rs       # CalculatorTool
│       │   ├── statistics.rs       # StatisticsTool
│       │   ├── linear_algebra.rs   # LinearAlgebraTool
│       │   ├── symbolic.rs         # SymbolicMathTool
│       │   └── mod.rs
│       ├── time/
│       │   ├── datetime.rs         # DateTimeTool
│       │   ├── timer.rs            # TimerTool
│       │   ├── scheduler.rs        # SchedulerTool
│       │   ├── timezone.rs         # TimeZoneTool
│       │   └── mod.rs
│       ├── crypto/
│       │   ├── hash.rs             # HashTool
│       │   ├── encryption.rs       # EncryptionTool
│       │   ├── signature.rs        # SignatureTool
│       │   ├── random.rs           # RandomTool
│       │   └── mod.rs
│       ├── integration/
│       │   ├── github.rs           # GitHubTool
│       │   ├── jira.rs             # JiraTool
│       │   ├── aws.rs              # AwsTool
│       │   ├── kubernetes.rs       # KubernetesTool
│       │   └── mod.rs
│       └── lib.rs
│
├── llmspell-workflows/             # Workflow implementations
│   ├── Cargo.toml
│   └── src/
│       ├── sequential.rs           # SequentialWorkflow
│       ├── parallel.rs             # ParallelWorkflow
│       ├── conditional.rs          # ConditionalWorkflow
│       ├── loop_flow.rs            # LoopWorkflow
│       ├── map_reduce.rs           # MapReduceWorkflow
│       ├── pipeline.rs             # PipelineWorkflow
│       ├── base_impl.rs            # Common implementations
│       └── lib.rs
│
├── llmspell-mcp/                   # Model Control Protocol support
│   ├── Cargo.toml
│   └── src/
│       ├── client.rs               # MCP client implementation
│       ├── server.rs               # MCP server implementation
│       ├── tool_adapter.rs         # Tool → MCP adaptation
│       ├── agent_adapter.rs        # Agent → MCP adaptation
│       └── lib.rs
│
├── llmspell-a2a/                   # Agent-to-Agent Protocol support
│   ├── Cargo.toml
│   └── src/
│       ├── client.rs               # A2A client implementation
│       ├── server.rs               # A2A server implementation
│       ├── protocol.rs             # A2A protocol definition
│       └── lib.rs
│
├── llmspell-testing/               # Testing utilities
│   ├── Cargo.toml
│   └── src/
│       ├── mocks/
│       │   ├── llm_provider.rs     # Mock LLM providers
│       │   ├── tools.rs            # Mock tools
│       │   ├── agents.rs           # Mock agents
│       │   └── mod.rs
│       ├── fixtures/
│       │   ├── agents.rs           # Test agent fixtures
│       │   ├── workflows.rs        # Test workflow fixtures
│       │   ├── data.rs             # Test data fixtures
│       │   └── mod.rs
│       ├── harness/
│       │   ├── script_test.rs      # Script testing harness
│       │   ├── integration.rs      # Integration test helpers
│       │   ├── performance.rs      # Performance test utilities
│       │   └── mod.rs
│       └── lib.rs
│
├── llmspell-cli/                   # CLI application
│   ├── Cargo.toml
│   └── src/
│       ├── commands/
│       │   ├── run.rs              # Run spell command
│       │   ├── test.rs             # Test spell command
│       │   ├── init.rs             # Initialize project
│       │   ├── validate.rs         # Validate spell syntax
│       │   └── mod.rs
│       ├── config.rs               # Configuration management
│       ├── cli.rs                  # CLI interface
│       └── main.rs
│
├── llmspell/                       # Main library crate
│   ├── Cargo.toml
│   └── src/
│       ├── prelude.rs              # Common imports
│       ├── runtime.rs              # Runtime management
│       ├── spell.rs                # Spell execution
│       └── lib.rs
│
├── examples/                       # Example spells
│   ├── lua/
│   │   ├── simple_chat.lua
│   │   ├── multi_agent.lua
│   │   ├── research_workflow.lua
│   │   └── tool_composition.lua
│   ├── javascript/
│   │   ├── simple_chat.js
│   │   ├── multi_agent.js
│   │   ├── research_workflow.js
│   │   └── tool_composition.js
│   └── README.md
│
└── tests/
    ├── integration/                # Integration tests
    ├── performance/                # Performance benchmarks
    └── scripts/                    # Test scripts
```

### Feature Flags and Conditional Compilation

Each crate includes feature flags for optional dependencies:

```toml
# llmspell/Cargo.toml
[features]
default = ["lua", "javascript", "all-tools", "all-agents"]

# Script engines
lua = ["llmspell-bridge/lua", "mlua"]
javascript = ["llmspell-bridge/js", "rquickjs"]

# Tool categories
tools-system = ["llmspell-tools/system"]
tools-web = ["llmspell-tools/web"]
tools-ai = ["llmspell-tools/ai"]
tools-communication = ["llmspell-tools/communication"]
tools-math = ["llmspell-tools/math"]
tools-time = ["llmspell-tools/time"]
tools-crypto = ["llmspell-tools/crypto"]
tools-integration = ["llmspell-tools/integration"]
all-tools = [
    "tools-system", "tools-web", "tools-ai", "tools-communication",
    "tools-math", "tools-time", "tools-crypto", "tools-integration"
]

# Agent types
agents-chat = ["llmspell-agents/chat"]
agents-research = ["llmspell-agents/research"]
agents-code = ["llmspell-agents/code"]
agents-data = ["llmspell-agents/data"]
agents-planner = ["llmspell-agents/planner"]
agents-orchestrator = ["llmspell-agents/orchestrator"]
all-agents = [
    "agents-chat", "agents-research", "agents-code",
    "agents-data", "agents-planner", "agents-orchestrator"
]

# Workflow types
workflows-sequential = ["llmspell-workflows/sequential"]
workflows-parallel = ["llmspell-workflows/parallel"]
workflows-conditional = ["llmspell-workflows/conditional"]
workflows-loop = ["llmspell-workflows/loop"]
workflows-mapreduce = ["llmspell-workflows/mapreduce"]
workflows-pipeline = ["llmspell-workflows/pipeline"]
all-workflows = [
    "workflows-sequential", "workflows-parallel", "workflows-conditional",
    "workflows-loop", "workflows-mapreduce", "workflows-pipeline"
]

# Storage backends
storage-sled = ["llmspell-infra/sled"]
storage-rocksdb = ["llmspell-infra/rocksdb"]

# LLM providers (via rig)
providers-openai = ["llmspell-infra/rig-openai"]
providers-anthropic = ["llmspell-infra/rig-anthropic"]
providers-local = ["llmspell-infra/candle"]

# Protocols
mcp = ["llmspell-mcp"]
a2a = ["llmspell-a2a"]

# Optional infrastructure
metrics = ["llmspell-infra/metrics"]
tracing = ["llmspell-infra/tracing"]
opentelemetry = ["llmspell-infra/opentelemetry"]

# Development/testing
testing = ["llmspell-testing"]
mocks = ["llmspell-testing/mocks"]
```

### Testing Strategy Integration

The modular structure enables comprehensive testing:

#### Unit Testing
```rust
// llmspell-tools/src/math/calculator.rs
#[cfg(test)]
mod tests {
    use super::*;
    use llmspell_testing::mocks::*;

    #[tokio::test]
    async fn test_calculator_basic_operations() {
        let calc = CalculatorTool;
        let result = calc.execute(json!({"expression": "2 + 2"})).await?;
        assert_eq!(result.content["result"], 4);
    }
}
```

#### Integration Testing
```rust
// tests/integration/agent_tool_integration.rs
use llmspell_testing::{fixtures::*, harness::*};

#[tokio::test]
async fn test_agent_with_calculator() {
    let mut agent = test_chat_agent();
    agent.add_tool(Box::new(CalculatorTool));
    
    let response = agent.chat("What is 15% of 2500?").await?;
    assert!(response.contains("375"));
}
```

#### Script Testing
```rust
// llmspell-bridge/src/lua/tests.rs
#[tokio::test]
async fn test_lua_agent_creation() {
    let lua_code = r#"
        local agent = Agent.new({
            system_prompt = "Test assistant",
            tools = { Calculator.new() }
        })
        return agent:chat("Calculate 10 + 5")
    "#;
    
    let result = execute_lua_script(lua_code).await?;
    assert!(result.contains("15"));
}
```

### Build and Development Workflow

#### Development Commands
```bash
# Run all tests
cargo test --workspace

# Test specific component
cargo test -p llmspell-tools

# Test with specific features
cargo test --features "lua,tools-math"

# Benchmark performance
cargo bench -p llmspell-testing

# Check all feature combinations
cargo hack check --feature-powerset

# Build CLI
cargo build -p llmspell-cli --release

# Run examples
cargo run --example lua/simple_chat.lua
```

#### CI/CD Integration
```yaml
# .github/workflows/test.yml
strategy:
  matrix:
    features:
      - "lua,tools-system"
      - "javascript,tools-web"
      - "all-tools,all-agents"
      - "minimal"  # Only core traits
```

This modular structure provides:
- **Clear separation of concerns** between different component types
- **Feature-based compilation** for minimal deployments
- **Comprehensive testing** at all levels
- **Easy extension** for new tools, agents, and workflows
- **Protocol support** for MCP and A2A integration

## Conclusion

Rs-LLMSpell provides a unique combination of:

1. **Proven Patterns**: Go-llms architecture adapted for Rust
2. **Scriptable Interface**: Multi-language support with idiomatic APIs
3. **Production Infrastructure**: Hooks, events, state management built-in
4. **Extensible Components**: 40+ tools, multiple agents, workflow patterns
5. **Bridge Philosophy**: Leverage best-in-class Rust crates

This architecture enables rapid AI development while maintaining production-grade reliability and performance.