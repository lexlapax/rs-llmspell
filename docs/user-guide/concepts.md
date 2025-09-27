# Core Concepts

**Version**: 0.8.10  
**Last Updated**: December 2024

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
7. [RAG (Retrieval-Augmented Generation)](#rag-retrieval-augmented-generation) ⭐ **Phase 8.10.6**
8. [Vector Storage & HNSW](#vector-storage--hnsw) ⭐ **Phase 8.10.6**
9. [Multi-Tenancy](#multi-tenancy) ⭐ **Phase 8.10.6**
10. [Hooks & Events](#hooks--events)
11. [Sessions & Artifacts](#sessions--artifacts)
12. [Execution Context](#execution-context)
13. [Security Model](#security-model)

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

LLMSpell includes 100+ built-in tools across 12 categories:
- **File System** (8): read, write, move, copy, delete, watch, compress
- **Web** (12): fetch, scrape, search, monitor, sitemap crawler, robots.txt
- **API** (6): REST tester, webhook caller, GraphQL, OpenAPI
- **Data** (8): JSON/CSV processing, XML, YAML, database connectors
- **System** (6): command execution, environment, process management
- **Media** (8): image/audio/video processing, PDF, document conversion
- **Utility** (15): hash, encrypt, datetime, regex, text manipulation
- **Communication** (12): email, SMS, chat integrations
- **Security** (8): encryption, validation, sanitization
- **RAG** (6): document ingestion, vector search, embedding generation ⭐
- **Multi-Tenant** (4): tenant management, isolation, usage tracking ⭐
- **Custom**: User-defined tools

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

## RAG (Retrieval-Augmented Generation) ⭐ **Phase 8.10.6**

RAG enhances LLM responses by combining vector search with generation, enabling knowledge-grounded responses with 70% cost optimization through intelligent caching.

### Core RAG Components

```rust
pub struct RAGPipeline {
    pub vector_storage: Arc<dyn VectorStorage>,
    pub embeddings: Arc<dyn EmbeddingProvider>,
    pub chunking: Arc<dyn ChunkingStrategy>,
    pub cache: Arc<dyn RAGCache>,
}
```

### RAG Workflow

1. **Document Ingestion**
   - Documents chunked into semantic pieces (sliding window, sentence, semantic)
   - Chunks converted to embeddings via providers (OpenAI, local models)
   - Vectors stored in HNSW index with metadata

2. **Query Processing**
   - User query converted to embedding
   - Vector similarity search (cosine, euclidean, inner product)
   - Results ranked and filtered by relevance threshold

3. **Context Augmentation**
   - Retrieved chunks combined with original query
   - LLM generates response with grounded knowledge
   - Citations and sources preserved in metadata

### Document Chunking

```lua
-- Lua API for document ingestion
local chunks = Tool.invoke("document-chunker", {
    strategy = "sliding_window",
    chunk_size = 512,
    overlap = 64,
    content = document_text
})

for _, chunk in ipairs(chunks) do
    RAG.ingest({
        content = chunk.text,
        metadata = {
            source = "document.pdf",
            page = chunk.page,
            chunk_id = chunk.id
        }
    })
end
```

### Vector Search

```lua
-- Multi-dimensional vector search
local results = RAG.search("explain quantum computing", {
    collection = "physics_papers",
    limit = 5,
    threshold = 0.8,
    include_metadata = true
})

for _, result in ipairs(results) do
    print(f"Score: {result.score}, Source: {result.metadata.source}")
end
```

### Cost Optimization (70% Reduction)

1. **Embedding Cache**: Pre-computed embeddings cached with TTL
2. **Search Cache**: Query results cached to avoid re-computation
3. **Document Cache**: Frequently accessed content cached in memory
4. **Batch Processing**: Multiple embeddings generated in single API call

```lua
-- Enable aggressive caching
RAG.configure({
    embedding_cache_enabled = true,
    search_cache_enabled = true,
    cache_ttl_seconds = 3600  -- 1 hour
})
```

---

## Vector Storage & HNSW ⭐ **Phase 8.10.6**

High-performance vector storage using Hierarchical Navigable Small World (HNSW) algorithm for sub-10ms similarity search at million-vector scale.

### HNSW Architecture

HNSW creates a multi-layer graph structure for efficient approximate nearest neighbor search:

- **Layer 0**: Contains all vectors with local connections
- **Higher Layers**: Sparse subsets with long-range connections
- **Search**: Starts from top layer, narrows down to target neighborhood

### Performance Characteristics

| Vectors | Build Time | Memory | Search Time | Accuracy |
|---------|------------|--------|-------------|----------|
| 10K | 5s | 50MB | <1ms | 99%+ |
| 100K | 45s | 400MB | <5ms | 98%+ |
| 1M | 8min | 3.2GB | <10ms | 97%+ |
| 10M | 90min | 28GB | <20ms | 95%+ |

### Configuration Profiles

```rust
// Speed-optimized (lower accuracy, faster search)
let config = HNSWConfig {
    m: 8,                    // Fewer connections
    ef_construction: 50,     // Faster build
    ef_search: 25,          // Faster search
    max_elements: 1_000_000,
    metric: DistanceMetric::Cosine,
};

// Accuracy-optimized (slower, higher recall)
let config = HNSWConfig {
    m: 48,                   // More connections
    ef_construction: 500,    // Better quality
    ef_search: 300,         // Thorough search
    max_elements: 1_000_000,
    metric: DistanceMetric::Cosine,
};
```

### Multi-Dimensional Support

Vector storage supports multiple embedding dimensions with automatic routing:

| Dimensions | Model Example | Use Case |
|------------|---------------|----------|
| 384 | all-MiniLM-L6-v2 | Fast retrieval, limited memory |
| 768 | all-mpnet-base-v2 | Balanced performance |
| 1536 | text-embedding-3-small | OpenAI standard |
| 3072 | text-embedding-3-large | Maximum accuracy |

### Scoped Storage

Vectors are scoped for multi-tenant isolation:

```rust
pub enum StateScope {
    Global,                              // System-wide access
    User(String),                       // Per-user isolation
    Session(String),                    // Session-scoped vectors
    Workflow(String),                   // Workflow-scoped context
    Custom(String),                     // "tenant:id" for multi-tenancy
}
```

### Metadata Filtering

Vectors support rich metadata for hybrid search:

```lua
-- Store with metadata
RAG.ingest({
    content = "Quantum computers use qubits...",
    metadata = {
        source = "physics-textbook.pdf",
        chapter = 3,
        difficulty = "advanced",
        tags = {"quantum", "computing", "physics"},
        date = "2024-01-15"
    }
})

-- Search with metadata filters
local results = RAG.search("quantum algorithms", {
    filters = {
        difficulty = "advanced",
        tags_contains = "quantum",
        date_after = "2023-01-01"
    }
})
```

---

## Multi-Tenancy ⭐ **Phase 8.10.6**

Complete tenant isolation with resource quotas, usage tracking, and billing integration for SaaS deployments.

### Tenant Isolation Levels

```rust
pub enum IsolationMode {
    None,           // No isolation (single tenant)
    Namespace,      // Logical separation (shared resources)
    Physical,       // Separate storage per tenant
    Strict,         // Complete isolation + validation
}
```

### Tenant Configuration

```rust
pub struct TenantConfig {
    pub tenant_id: String,
    pub isolation_mode: IsolationMode,
    pub resource_limits: TenantLimits,
    pub retention_policy: RetentionPolicy,
    pub billing_config: Option<BillingConfig>,
}

pub struct TenantLimits {
    pub max_vectors: Option<usize>,
    pub max_storage_mb: Option<usize>,
    pub max_queries_per_minute: Option<u32>,
    pub max_concurrent_operations: Option<u32>,
    pub max_embedding_tokens_per_day: Option<u64>,
}
```

### Tenant Lifecycle

```lua
-- Create tenant with quotas
local tenant_id = Tenant.create({
    name = "acme-corp",
    plan = "enterprise",
    limits = {
        max_vectors = 100000,
        max_queries_per_minute = 1000,
        max_storage_mb = 5000
    },
    retention_days = 365
})

-- Scope operations to tenant
State.setScope("tenant:" .. tenant_id)
RAG.setScope("tenant:" .. tenant_id)

-- All operations now isolated to this tenant
```

### Usage Tracking & Billing

```rust
pub struct UsageMetrics {
    pub vectors_stored: u64,
    pub queries_executed: u64,
    pub embedding_tokens: u64,
    pub storage_bytes: u64,
    pub compute_seconds: f64,
}

pub struct CostEstimate {
    pub storage_cost: f64,      // Per GB/month
    pub query_cost: f64,        // Per 1K queries
    pub embedding_cost: f64,    // Per 1K tokens
    pub total_monthly: f64,
}
```

### Cross-Tenant Operations

Strict isolation prevents accidental data leaks:

```lua
-- This will fail with isolation error
State.setScope("tenant:acme")
local data = State.get("secret_data")  -- From different tenant

-- Admin operations require explicit override
if User.hasRole("admin") then
    local stats = Tenant.getUsageStats("acme-corp", {
        admin_override = true
    })
end
```

### Resource Quotas

```lua
-- Check current usage against quotas
local usage = Tenant.getCurrentUsage()
local quotas = Tenant.getQuotas()

if usage.queries_per_minute > quotas.max_queries_per_minute then
    error("Rate limit exceeded: " .. usage.queries_per_minute .. "/" .. quotas.max_queries_per_minute)
end

-- Soft limits with warnings
if usage.storage_mb > quotas.max_storage_mb * 0.8 then
    Event.publish("tenant.storage.warning", {
        tenant_id = tenant_id,
        usage_percent = usage.storage_mb / quotas.max_storage_mb * 100
    })
end
```

### Session Collections

For conversational AI with memory:

```lua
-- Create session-scoped RAG collection
local session_id = Session.create({name = "support-chat"})
RAG.createCollection(session_id, {
    ttl_seconds = 3600,  -- 1 hour conversation memory
    max_vectors = 1000,
    auto_cleanup = true
})

-- Ingest conversation history
RAG.ingestSession(session_id, {
    content = "User asked about pricing plans",
    metadata = {type = "user_query", timestamp = os.time()}
})

-- Search conversation context
local context = RAG.searchSession(session_id, "what did we discuss about pricing?")
```

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

## Service Deployment ⭐ **Phase 10**

LLMSpell can be deployed as a production service with proper Unix daemon behavior.

### Daemon Mode

```bash
# Start as daemon
./llmspell kernel start --daemon --port 9555

# With custom paths
./llmspell kernel start --daemon \
  --log-file /var/log/llmspell/kernel.log \
  --pid-file /var/run/llmspell/kernel.pid
```

**Features:**
- **Double-fork technique**: Proper daemon detachment
- **PID file management**: Prevents multiple instances
- **Signal handling**: SIGTERM, SIGHUP, SIGUSR1, SIGUSR2
- **Log rotation**: Automatic log management
- **TTY detachment**: True background operation

### systemd Integration (Linux)

```ini
[Service]
Type=forking
ExecStart=/usr/local/bin/llmspell kernel start --daemon --port 9555
PIDFile=/var/run/llmspell/kernel.pid
Restart=on-failure
```

### launchd Integration (macOS)

```xml
<key>ProgramArguments</key>
<array>
    <string>/usr/local/bin/llmspell</string>
    <string>kernel</string>
    <string>start</string>
    <string>--daemon</string>
</array>
<key>RunAtLoad</key><true/>
<key>KeepAlive</key><true/>
```

### Signal Handling

- **SIGTERM/SIGINT**: Graceful shutdown
- **SIGHUP**: Reload configuration
- **SIGUSR1**: Dump statistics
- **SIGUSR2**: Toggle debug logging

---

## Debug & IDE Integration ⭐ **Phase 9-10**

### Debug Adapter Protocol (DAP)

Phase 9 added full DAP support for IDE debugging:

```lua
-- Enable DAP in script
Debug.enableDAP({
    port = 9556,
    wait_for_debugger = true
})

-- Programmatic breakpoints
Debug.breakpoint("function_name", line_number)
```

**DAP Features:**
- **Breakpoints**: Set/clear/conditional
- **Stepping**: Step in/over/out
- **Variables**: Inspect locals and globals
- **Call Stack**: View execution stack
- **REPL**: Evaluate expressions

### VS Code Integration

```json
{
  "type": "llmspell",
  "request": "attach",
  "name": "Debug LLMSpell",
  "port": 9556
}
```

### Jupyter Lab Integration

The kernel implements full Jupyter protocol:

```bash
# Connect Jupyter Lab
jupyter console --existing /var/lib/llmspell/kernel.json

# Or use notebook
jupyter notebook --kernel llmspell
```

### Performance Monitoring

```bash
# Health check
curl http://localhost:9555/health

# Metrics endpoint
curl http://localhost:9555/metrics

# Event correlation
curl http://localhost:9555/events
```

---

## Security Model

### Kernel Security

Phase 9-10 enhanced security with kernel-level isolation:

**Process Isolation:**
- Dedicated service user
- Restricted file permissions
- Resource limits (memory, CPU, files)
- Namespace isolation (Linux)

**Network Security:**
- HMAC message signing
- TLS support for remote connections
- IP whitelisting
- Rate limiting

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

| Operation | Target | Actual | Phase |
|-----------|--------|--------|----|
| Agent Creation | <50ms | ~10ms | Core |
| Tool Invocation | <10ms | <5ms | Core |
| State Read | <1ms | <1ms | Core |
| State Write | <5ms | <3ms | Core |
| Hook Overhead | <1% | <0.5% | Phase 4 |
| Event Throughput | >50K/sec | 90K/sec | Phase 4 |
| Workflow Step | <5ms | <5ms | Phase 3 |
| Vector Search (1M) | <10ms | <8ms | Phase 8 |
| RAG Query (E2E) | <100ms | <75ms | Phase 8 |
| Embedding Cache Hit | <1ms | <0.5ms | Phase 8 |
| Tenant Isolation Check | <1ms | <0.3ms | Phase 8 |
| HNSW Index Build (100K) | <60s | <45s | Phase 8 |
| **Kernel Startup** | **<200ms** | **<100ms** | **Phase 9** |
| **Message Processing** | **<10ms** | **<5ms** | **Phase 9** |
| **Protocol Parsing** | **<5ms** | **<1ms** | **Phase 9** |
| **DAP Stepping** | **<20ms** | **<10ms** | **Phase 9** |
| **Daemon Fork** | **<50ms** | **<30ms** | **Phase 10** |
| **Signal Handling** | **<5ms** | **<2ms** | **Phase 10** |

---

## See Also

- [Getting Started](getting-started.md) - Quick start with kernel setup
- [Configuration](configuration.md) - Kernel, RAG, and service configuration
- [Service Deployment](service-deployment.md) - Production deployment guide
- [Troubleshooting](troubleshooting.md) - Common issues and solutions
- [Lua API Reference](api/lua/README.md) - Complete API including Debug globals
- [Rust API Reference](api/rust/README.md) - Complete traits including kernel architecture
- [Kernel Examples](../../examples/script-users/kernel/) - Kernel and service examples
- [Configuration Examples](../../examples/script-users/configs/) - Kernel, RAG, and daemon configs
- [Examples Index](../../examples/EXAMPLE-INDEX.md) - 60+ examples including Phase 9-10 features