# Core Concepts

**Version**: 0.6.0  
**Last Updated**: August 2025

> **📚 Quick Reference**: This guide explains the fundamental concepts behind LLMSpell's architecture. For API details, see [Lua API](api/lua/README.md) or [Rust API](api/rust/README.md).

**🔗 Navigation**: [← User Guide](README.md) | [Getting Started →](getting-started.md) | [Configuration →](configuration.md)

---

## Table of Contents

1. [Overview](#overview)
2. [Component Model](#component-model)
3. [Agents](#agents)
4. [Tools](#tools)
5. [Workflows](#workflows)
6. [State Management](#state-management)
7. [Hooks & Events](#hooks--events)
8. [Sessions & Artifacts](#sessions--artifacts)
9. [Execution Context](#execution-context)
10. [Security Model](#security-model)

---

## Overview

LLMSpell is built on a **trait-based component architecture** where everything is a component that implements the `BaseAgent` trait. This provides a unified interface for agents, tools, and workflows while maintaining type safety and extensibility.

### Design Principles

1. **Component-First**: Everything is a component with metadata and execution capabilities
2. **Trait-Based**: Rust traits define behavior, enabling composition and extension
3. **State-Centric**: Components communicate through shared state rather than direct coupling
4. **Event-Driven**: Cross-cutting concerns handled through hooks and events
5. **Script-Friendly**: Identical functionality exposed to Lua/JS through bridge layer

---

## Component Model

### BaseAgent Trait

The foundational trait that all components implement (from `llmspell-core/src/traits/base_agent.rs`):

```rust
#[async_trait]
pub trait BaseAgent: Send + Sync {
    fn metadata(&self) -> &ComponentMetadata;
    async fn execute(&self, input: AgentInput, context: ExecutionContext) -> Result<AgentOutput>;
    async fn execute_impl(&self, input: AgentInput, context: ExecutionContext) -> Result<AgentOutput>;
    async fn validate_input(&self, input: &AgentInput) -> Result<()>;
    async fn handle_error(&self, error: LLMSpellError) -> Result<AgentOutput>;
    
    // Optional capabilities
    async fn stream_execute(&self, input: AgentInput, context: ExecutionContext) -> Result<AgentStream>;
    fn supports_streaming(&self) -> bool;
    fn supports_multimodal(&self) -> bool;
    fn supported_media_types(&self) -> Vec<MediaType>;
}
```

### Component Metadata

Every component has metadata that identifies and describes it:

```rust
pub struct ComponentMetadata {
    pub id: Uuid,
    pub name: String,
    pub version: String,
    pub description: String,
    pub component_type: ComponentType,
    pub created_at: DateTime<Utc>,
}
```

### Execution Flow

1. **Input Validation**: `validate_input()` ensures parameters are correct
2. **Event Emission**: Start event emitted if events are enabled
3. **Implementation**: `execute_impl()` runs the actual logic
4. **Event Completion**: Success/failure event emitted
5. **Error Handling**: `handle_error()` provides recovery options

---

## Agents

Agents are LLM-powered components that process natural language prompts. They extend `BaseAgent` with conversation management.

### Agent Trait

From `llmspell-core/src/traits/agent.rs`:

```rust
#[async_trait]
pub trait Agent: BaseAgent {
    fn config(&self) -> &AgentConfig;
    async fn get_conversation(&self) -> Result<Vec<ConversationMessage>>;
    async fn clear_conversation(&self) -> Result<()>;
    async fn add_message(&self, message: ConversationMessage) -> Result<()>;
}
```

### Conversation Management

Agents maintain conversation history with role-based messages:

```rust
pub enum MessageRole {
    System,    // System instructions
    User,      // User input
    Assistant, // Agent responses
}
```

### Builder Pattern

Both Rust and Lua use builders for agent creation:

**Rust:**
```rust
let agent = AgentBuilder::new()
    .name("assistant")
    .model("openai/gpt-4")
    .temperature(0.7)
    .build()?;
```

**Lua:**
```lua
local agent = Agent.builder()
    :name("assistant")
    :model("openai/gpt-4")
    :temperature(0.7)
    :build()
```

### Provider Integration

Agents support multiple LLM providers through a unified interface:
- OpenAI (GPT-3.5, GPT-4, GPT-4o)
- Anthropic (Claude 3 family)
- Ollama (local models)
- Groq (fast inference)

---

## Tools

Tools are functional components that perform specific operations. They extend `BaseAgent` with parameter schemas and security requirements.

### Tool Trait

From `llmspell-core/src/traits/tool.rs`:

```rust
#[async_trait]
pub trait Tool: BaseAgent {
    fn category(&self) -> ToolCategory;
    fn parameter_schema(&self) -> &ParameterSchema;
    fn security_requirements(&self) -> &SecurityRequirements;
    async fn invoke(&self, params: Value) -> Result<Value>;
}
```

### Tool Categories

Tools are organized by function:

```rust
pub enum ToolCategory {
    Filesystem,  // File operations
    Web,         // Web scraping, API calls
    Api,         // External API integration
    Analysis,    // Data analysis
    Data,        // Data processing
    System,      // System operations
    Media,       // Image/audio/video
    Utility,     // General utilities
    Custom(String),
}
```

### Security Levels

Tools have three security levels with increasing permissions:

```rust
pub enum SecurityLevel {
    Safe,        // No file/network access
    Restricted,  // Limited access with validation
    Privileged,  // Full system access
}
```

### Built-in Tools

LLMSpell includes 37+ built-in tools across 9 categories:
- **File System** (5): read, write, move, copy, delete
- **Web** (6): fetch, scrape, search, monitor
- **API** (2): REST tester, webhook caller
- **Data** (2): JSON/CSV processing
- **System** (4): command execution, environment
- **Media** (3): image/audio/video processing
- **Utility** (9): hash, encrypt, datetime, etc.

---

## Workflows

Workflows orchestrate multiple components into complex processes. They also implement `BaseAgent`, making them composable.

### Workflow Types

1. **Sequential**: Step-by-step execution
2. **Conditional**: Branching based on conditions
3. **Loop**: Iterative processing
4. **Parallel**: Concurrent execution

### Workflow Trait

From `llmspell-core/src/traits/workflow.rs`:

```rust
#[async_trait]
pub trait Workflow: BaseAgent {
    fn workflow_type(&self) -> WorkflowType;
    async fn add_step(&mut self, step: WorkflowStep) -> Result<()>;
    async fn get_state(&self, key: &str) -> Result<Option<Value>>;
    async fn set_state(&self, key: &str, value: Value) -> Result<()>;
}
```

### Variable References

Workflows support variable references between steps:
- `$stepName` - Output of a previous step
- `$stepName.field` - Specific field from step output
- `$$` - Original workflow input
- `$$.field` - Field from workflow input

### Example Workflow

```lua
local workflow = Workflow.sequential({
    name = "data_pipeline",
    steps = {
        {name = "fetch", tool = "web-fetch", input = {url = "$$.source_url"}},
        {name = "parse", tool = "json-processor", input = {data = "$fetch"}},
        {name = "analyze", agent = analyst, prompt = "Analyze: $parse.summary"},
        {name = "save", tool = "file-write", input = {
            path = "results.txt",
            content = "$analyze"
        }}
    }
})
```

---

## State Management

State provides thread-safe data sharing between components.

### State Scopes

```rust
pub enum StateScope {
    Global,     // Shared across all components
    Session,    // Scoped to a session
    Workflow,   // Scoped to a workflow instance
    Component,  // Scoped to a single component
}
```

### State Operations

**Lua API:**
```lua
State.set("counter", 0)
local value = State.get("counter")
State.delete("counter")
local keys = State.list()
```

### Persistence Backends

State can be persisted using different backends:
- **Memory**: Fast, ephemeral (default)
- **Sled**: Embedded database
- **RocksDB**: High-performance embedded
- **Redis**: Distributed caching (planned)

---

## Hooks & Events

### Hooks

Hooks intercept and modify component execution:

```lua
Hook.register("BeforeAgentExecution", function(context)
    -- Modify input, cancel, or continue
    return {
        action = "modified",
        modified_data = {input = {text = "prefixed: " .. context.data.input.text}}
    }
end, "high")
```

**Hook Points** (40+):
- `BeforeAgentExecution` / `AfterAgentExecution`
- `BeforeToolInvocation` / `AfterToolInvocation`
- `BeforeWorkflowStep` / `AfterWorkflowStep`
- State, session, and artifact hooks

**Hook Results**:
- `continue` - Proceed normally
- `modified` - Continue with modified data
- `cancel` - Stop execution
- `retry` - Retry with backoff

### Events

Events provide async notifications without blocking:

```lua
-- Publish
Event.publish("user.action.completed", {
    action = "analysis",
    duration = 1234
})

-- Subscribe
local sub = Event.subscribe("user.*")
local event = Event.receive(sub, 1000) -- 1s timeout
```

**Event Format** (UniversalEvent):
```json
{
    "id": "uuid",
    "event_type": "user.action.completed",
    "timestamp": "2025-01-01T00:00:00Z",
    "version": "1.0",
    "source": {...},
    "data": {...},
    "metadata": {...}
}
```

---

## Sessions & Artifacts

### Sessions

Sessions group related operations with lifecycle management:

```lua
local session_id = Session.create({
    name = "analysis_session",
    description = "Data analysis task",
    tags = {"analysis", "priority-high"}
})

-- Operations happen in session context
Session.setCurrent(session_id)

-- Later...
Session.suspend(session_id)  -- Pause
Session.resume(session_id)   -- Continue
Session.complete(session_id) -- Finish
```

### Artifacts

Artifacts store content associated with sessions:

```lua
local artifact_id = Artifact.store(
    session_id,
    "tool_result",
    "analysis.json",
    JSON.stringify(results),
    {mime_type = "application/json"}
)

-- Retrieve later
local artifact = Artifact.get(session_id, artifact_id)
```

**Artifact Types**:
- `tool_result` - Tool execution outputs
- `agent_output` - Agent responses
- `user_input` - User-provided content
- `system_generated` - System artifacts

---

## Execution Context

The `ExecutionContext` carries environment information through component execution:

```rust
pub struct ExecutionContext {
    pub session_id: Option<String>,
    pub conversation_id: Option<String>,
    pub metadata: Metadata,
    pub state: Option<Arc<dyn StateManager>>,
    pub events: Option<Arc<dyn EventEmitter>>,
    pub hooks: Option<Arc<dyn HookManager>>,
}
```

### Context Propagation

Context automatically flows through:
1. Agent executions
2. Tool invocations
3. Workflow steps
4. Nested workflows

### Metadata

Metadata provides correlation and tracking:
```rust
pub struct Metadata {
    pub correlation_id: Option<String>,
    pub causation_id: Option<String>,
    pub user_id: Option<String>,
    pub trace_id: Option<String>,
    pub span_id: Option<String>,
}
```

---

## Security Model

### Sandboxing

Scripts run in sandboxed environments:
- **Lua**: Restricted stdlib, no file/network access by default
- **JavaScript**: V8 isolates with permission model
- **Resource Limits**: CPU, memory, execution time

### Permission System

Components declare required permissions:

```rust
pub struct SecurityRequirements {
    pub level: SecurityLevel,
    pub file_permissions: Vec<String>,    // Paths
    pub network_permissions: Vec<String>, // Domains
    pub env_permissions: Vec<String>,     // Variables
}
```

### Validation

All inputs are validated:
1. **Parameter Schemas**: JSON Schema validation
2. **Type Checking**: Runtime type verification
3. **Range Limits**: Max sizes, counts, durations
4. **Sanitization**: Path traversal, injection prevention

---

## Performance Characteristics

Target performance metrics (validated in benchmarks):

| Operation | Target | Actual |
|-----------|--------|--------|
| Agent Creation | <50ms | ~10ms |
| Tool Invocation | <10ms | <10ms |
| State Read | <1ms | <1ms |
| State Write | <5ms | <3ms |
| Hook Overhead | <1% | <0.5% |
| Event Throughput | >50K/sec | 90K/sec |
| Workflow Step | <5ms | <5ms |

---

## See Also

- [Getting Started](getting-started.md) - Quick start guide
- [Lua API Reference](api/lua/README.md) - Complete Lua API
- [Rust API Reference](api/rust/README.md) - Complete Rust API
- [Examples](../../examples/EXAMPLE-INDEX.md) - Working examples
- [Configuration](configuration.md) - System configuration