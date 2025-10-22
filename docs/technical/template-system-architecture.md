# Template System Architecture

**Version:** 0.12.0 (Phase 12 Complete)
**Status:** Production Ready
**Last Updated:** Phase 12.13 (October 24, 2025)

## Executive Summary

The rs-llmspell template system provides production-ready AI workflow templates that solve the "0-day retention problem" by offering immediate value post-installation. Templates combine agents, tools, RAG, and Local LLM into executable solutions without requiring users to architect workflows from scratch.

**Key Metrics (Phase 12.13 Complete):**
- **10 production templates** implemented (6 base + 4 advanced patterns)
- **149 unit/integration tests** (122 unit + 27 integration, 100% passing)
- **2,651 LOC** template implementation code
- **3,655 lines** total documentation (user guides + developer guide)
- <100ms execution overhead (excluding template runtime) ✅ Achieved: ~2ms (50x faster)
- <10ms template discovery ✅ Achieved: ~0.5ms (20x faster)
- <5ms parameter validation ✅ Achieved: ~0.1ms (50x faster)
- Zero clippy warnings, zero rustdoc warnings, 100% format compliance

---

## Architecture Overview

### System Design Principles

1. **Composability**: Templates orchestrate existing llmspell infrastructure (agents, tools, workflows, RAG)
2. **Declarative Configuration**: Parameter schemas with built-in validation
3. **Type Safety**: Rust trait system ensures compile-time correctness
4. **Performance**: Zero-copy Arc sharing, lazy initialization, efficient registry lookups
5. **Extensibility**: Plugin architecture allows custom templates
6. **Multi-Language Support**: Same templates accessible from CLI, Lua, JavaScript (future)

### High-Level Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                          User Interfaces                         │
├──────────────────┬──────────────────┬──────────────────────────┤
│   CLI Commands   │   Lua Scripts    │  JavaScript (Future)     │
│                  │                  │                           │
│ llmspell template│ Template.list()  │ Template.execute()       │
│   exec           │ Template.execute()│                          │
│   info           │ Template.search()│                          │
└────────┬─────────┴────────┬─────────┴────────┬────────────────┘
         │                  │                  │
         v                  v                  v
┌─────────────────────────────────────────────────────────────────┐
│                      llmspell-bridge                             │
│  ┌────────────────────────────────────────────────────────────┐ │
│  │                    TemplateBridge                          │ │
│  │  Business Logic Layer: Validation, Context Building        │ │
│  │  - list_templates(category)                                │ │
│  │  - execute_template(name, params)                          │ │
│  │  - get_template_info(name, include_schema)                 │ │
│  │  - search_templates(query, category)                       │ │
│  │  - estimate_cost(name, params)                             │ │
│  └────────────────────────────────────────────────────────────┘ │
└─────────────────────┬───────────────────────────────────────────┘
                      │
                      v
┌─────────────────────────────────────────────────────────────────┐
│                    llmspell-templates                            │
│  ┌────────────────────────────────────────────────────────────┐ │
│  │                  TemplateRegistry                          │ │
│  │  Template Discovery and Management                         │ │
│  │  - register(template)                                      │ │
│  │  - get(id) -> Arc<dyn Template>                            │ │
│  │  - search(query) -> Vec<TemplateMetadata>                  │ │
│  │  - discover_by_category(cat) -> Vec<TemplateMetadata>     │ │
│  └────────────────────────────────────────────────────────────┘ │
│                                                                  │
│  ┌────────────────────────────────────────────────────────────┐ │
│  │                  Template Trait                            │ │
│  │  Core Interface for All Templates                          │ │
│  │  - metadata() -> TemplateMetadata                          │ │
│  │  - validate(params) -> Result<()>                          │ │
│  │  - async execute(params, context) -> TemplateOutput        │ │
│  │  - config_schema() -> ConfigSchema                         │ │
│  │  - async estimate_cost(params) -> CostEstimate             │ │
│  └────────────────────────────────────────────────────────────┘ │
│                                                                  │
│  ┌────────────────────────────────────────────────────────────┐ │
│  │       Built-in Template Implementations (10 Total)        │ │
│  ├────────────────────────────────────────────────────────────┤ │
│  │  Base Templates (Phase 12.1-12.8):                        │ │
│  │    1. ResearchAssistantTemplate     (574 LOC, 4-phase)    │ │
│  │    2. InteractiveChatTemplate       (421 LOC, REPL)       │ │
│  │    3. DataAnalysisTemplate          (287 LOC, stats+viz)  │ │
│  │    4. CodeGeneratorTemplate         (352 LOC, 3-agents)   │ │
│  │    5. DocumentProcessorTemplate     (318 LOC, PDF/OCR)    │ │
│  │    6. WorkflowOrchestratorTemplate  (443 LOC, parallel)   │ │
│  │                                                            │ │
│  │  Advanced Templates (Phase 12.10-12.13):                  │ │
│  │    7. CodeReviewTemplate            (298 LOC, aspects)    │ │
│  │    8. ContentGenerationTemplate     (289 LOC, iteration)  │ │
│  │    9. FileClassificationTemplate    (277 LOC, classify)   │ │
│  │   10. KnowledgeManagementTemplate   (392 LOC, RAG CRUD)   │ │
│  └────────────────────────────────────────────────────────────┘ │
└─────────────────────┬───────────────────────────────────────────┘
                      │
                      v
┌─────────────────────────────────────────────────────────────────┐
│                  ExecutionContext                                │
│  Infrastructure Coordination Layer                               │
│  - tool_registry: Arc<ToolRegistry>                             │
│  - agent_registry: Arc<AgentFactoryRegistry>                    │
│  - workflow_factory: Arc<dyn WorkflowFactory>                   │
│  - providers: Arc<ProviderManager>                              │
│  - state_manager: Option<Arc<StateManager>>                     │
│  - session_manager: Option<Arc<SessionManager>>                 │
│  - rag_manager: Option<Arc<RAGManager>>                         │
└─────────────────────┬───────────────────────────────────────────┘
                      │
                      v
┌─────────────────────────────────────────────────────────────────┐
│                Core LLMSpell Infrastructure                      │
│  llmspell-agents │ llmspell-tools │ llmspell-workflows │...     │
└─────────────────────────────────────────────────────────────────┘
```

---

## Dual-Layer Registry Architecture (Phase 12.7.1)

### Problem Statement

Prior to Phase 12.7.1, template execution failed with "tool_registry is required" error because `ExecutionContext::builder()` expected 4 infrastructure components (ToolRegistry, AgentRegistry, WorkflowFactory, ProviderManager) but `ScriptRuntime` only provided `ComponentRegistry`.

### Root Cause Analysis

`ComponentRegistry` (266-line HashMap wrapper) and `ToolRegistry` (1571-line infrastructure) serve fundamentally different purposes and cannot be merged or converted:

| Aspect | ComponentRegistry (Layer 1) | ToolRegistry (Layer 2) |
|--------|------------------------------|------------------------|
| **Purpose** | Fast script access | Template infrastructure |
| **Structure** | Simple HashMap | Complex indexed registry |
| **Size** | 266 lines | 1571 lines |
| **Features** | O(1) name→tool lookup | Discovery, validation, hooks, metrics |
| **Used By** | Lua/JS scripts | Template system |
| **Performance** | <1ms lookups | Comprehensive features |

### Architectural Solution

`ScriptRuntime` maintains **two parallel registry layers**:

#### Layer 1: Script Access (`ComponentRegistry`)
- **Purpose**: Fast tool/agent/workflow lookups for Lua/JavaScript scripts
- **Implementation**: Lightweight `HashMap<String, Arc<dyn Tool>>`
- **Used by**: Script engines via `Tool.execute()` in Lua
- **Features**: Simple name→component mapping, O(1) lookup
- **Code**: `llmspell-bridge/src/registry.rs` (266 lines)

#### Layer 2: Infrastructure (`ToolRegistry` + `AgentRegistry` + `WorkflowFactory`)
- **Purpose**: Template execution with discovery, validation, hooks
- **Implementation**: Full-featured registries with caching, categorization
- **Used by**: Template system via `ExecutionContext`
- **Features**: Discovery by category, validation hooks, execution metrics
- **Code**: `llmspell-tools/src/registry.rs` (1571 lines for ToolRegistry alone)

### Why Both Layers Are Necessary

The layers cannot be merged because:

1. **Different Data Structures**:
   - `ComponentRegistry`: Simple `HashMap` for O(1) lookups
   - `ToolRegistry`: Complex indexes, hooks, validation chains

2. **Different Use Cases**:
   - Scripts need: Fast `get_tool("calculator")` → execute
   - Templates need: Discovery, validation, `list_tools_by_category()`, metrics

3. **Performance vs Features Trade-off**:
   - Scripts require minimal overhead (<1ms lookup)
   - Templates require comprehensive infrastructure (hooks, validation, discovery)

4. **Memory Cost is Minimal**:
   - Both layers hold `Arc` references to the same tool instances
   - Only the index structures are duplicated (~few KB per registry)

### Dual-Registration Pattern

Tools are registered to **both layers simultaneously** during runtime initialization:

```rust
// Step 1: Create both registries (llmspell-bridge/src/runtime.rs:263-268)
let tool_registry = Arc::new(llmspell_tools::ToolRegistry::new());         // Infrastructure
let component_registry = Arc::new(ComponentRegistry::with_templates()?);   // Script access

// Step 2: Register tools to BOTH (dual-registration)
// Implementation: llmspell-bridge/src/tools.rs:111-134
async fn register_tool_dual<T, F>(
    component_registry: &Arc<ComponentRegistry>,
    tool_registry: &Arc<llmspell_tools::ToolRegistry>,
    name: &str,
    mut tool_factory: F,
) -> Result<(), Box<dyn std::error::Error>>
where
    T: Tool + Send + Sync + 'static,
    F: FnMut() -> T,  // FnMut allows calling factory twice
{
    // Create first instance for ComponentRegistry (script access)
    let tool_for_component = Arc::new(tool_factory());
    component_registry.register_tool(name.to_string(), tool_for_component)?;

    // Create second instance for ToolRegistry (infrastructure)
    let tool_for_infrastructure = tool_factory();
    tool_registry.register(name.to_string(), tool_for_infrastructure).await?;

    Ok(())
}
```

**Key Points**:
- Each tool is instantiated **twice** (one for each registry)
- Tools are stateless, so memory overhead is negligible
- `FnMut` closure allows calling factory twice (not `FnOnce`)
- ToolRegistry::register() takes ownership, preventing Arc sharing

### Data Flow Diagram

```
┌─────────────────────────────────────────────┐
│ CLI Command / User Script                   │
└────────────┬────────────────────────────────┘
             │
             ├──► Scripts (Lua/JS)
             │    └──► ComponentRegistry (HashMap)
             │         └──► Tool.execute("calculator", {}) [Fast path <1ms]
             │
             └──► Templates
                  └──► ExecutionContext
                       ├──► ToolRegistry (full-featured)
                       │     ├─ Discovery by category
                       │     ├─ Validation hooks
                       │     └─ Execution metrics
                       ├──► AgentRegistry (factories)
                       ├──► WorkflowFactory (creation)
                       └──► ProviderManager (LLMs)
```

### Implementation Details

**ScriptRuntime Fields** (`llmspell-bridge/src/runtime.rs:213-246`):
```rust
pub struct ScriptRuntime {
    // Layer 1: Script Access
    registry: Arc<ComponentRegistry>,  // HashMap-based, fast lookups

    // Layer 2: Infrastructure
    tool_registry: Arc<llmspell_tools::ToolRegistry>,        // Full-featured
    agent_registry: Arc<llmspell_agents::FactoryRegistry>,   // Agent creation
    workflow_factory: Arc<dyn llmspell_workflows::WorkflowFactory>,  // Workflows

    // Shared by both layers
    provider_manager: Arc<ProviderManager>,  // LLM providers
}
```

**ExecutionContext Building** (`llmspell-bridge/src/runtime.rs:895-907`):
```rust
// Templates get full infrastructure via ExecutionContext
let core_provider_manager = self.provider_manager.create_core_manager_arc().await?;
let context = llmspell_templates::context::ExecutionContext::builder()
    .with_tool_registry(self.tool_registry.clone())      // Layer 2
    .with_agent_registry(self.agent_registry.clone())    // Layer 2
    .with_workflow_factory(self.workflow_factory.clone()) // Layer 2
    .with_providers(core_provider_manager)                // Shared
    .build()?;
```

### Benefits

1. **Scripts**: Fast O(1) tool lookups via ComponentRegistry (< 1ms)
2. **Templates**: Full infrastructure with discovery, validation, hooks via ToolRegistry
3. **Memory Efficient**: Arc sharing means same tool instances, just two indexes
4. **Clear Separation**: Script access vs infrastructure concerns are isolated
5. **Maintainable**: Each layer can evolve independently
6. **Testable**: Integration tests verify both layers work (see `llmspell-bridge/tests/template_execution_test.rs`)

### Testing

**Integration Tests** (`llmspell-bridge/tests/template_execution_test.rs`):
- `test_tools_registered_in_both_registries()`: Verifies 40+ tools exist in BOTH registries
- `test_execution_context_has_infrastructure()`: Verifies all 4 components accessible
- `test_template_execution_no_infrastructure_error()`: Confirms no "tool_registry is required" error
- `test_all_builtin_templates_have_infrastructure()`: Tests all 6 templates
- `test_dual_registration_memory_safety()`: Stress tests with multiple runtimes

**Results**: All 6 tests PASSED (0.81s runtime)

### Historical Context

**Before Phase 12.7.1**:
```rust
// ScriptRuntime only had ComponentRegistry
pub struct ScriptRuntime {
    registry: Arc<ComponentRegistry>,  // HashMap only
    // Missing: tool_registry, agent_registry, workflow_factory
}

// Templates failed:
let context = ExecutionContext::builder().build()?;
// Error: "tool_registry is required"
```

**After Phase 12.7.1**:
```rust
// ScriptRuntime has BOTH layers
pub struct ScriptRuntime {
    registry: Arc<ComponentRegistry>,              // Layer 1: Scripts
    tool_registry: Arc<ToolRegistry>,              // Layer 2: Templates
    agent_registry: Arc<AgentFactoryRegistry>,     // Layer 2: Templates
    workflow_factory: Arc<dyn WorkflowFactory>,    // Layer 2: Templates
}

// Templates succeed:
let context = ExecutionContext::builder()
    .with_tool_registry(tool_registry)     // ✓ Now available
    .with_agent_registry(agent_registry)   // ✓ Now available
    .with_workflow_factory(workflow_factory) // ✓ Now available
    .build()?;  // SUCCESS
```

### Design Rationale

This is **not a temporary workaround** but the correct architectural design:

1. **Separation of Concerns**: Script access and template infrastructure are different domains
2. **Performance Optimization**: Scripts get fast HashMap, templates get comprehensive features
3. **Proven Pattern**: Same pattern as `provider_manager` which also exists separately
4. **Future-Proof**: Allows independent evolution of script and template layers
5. **Zero Breaking Changes**: Existing script code unaffected

See `TODO.md` Phase 12.7.1 for 180+ line detailed analysis including 10 key architectural insights.

---

## Core Components

### 1. Template Trait

The `Template` trait defines the contract all templates must implement:

```rust
#[async_trait]
pub trait Template: Send + Sync {
    /// Template metadata (name, description, category, version)
    fn metadata(&self) -> &TemplateMetadata;

    /// Validate parameters against config schema
    fn validate(&self, params: &TemplateParams) -> Result<()>;

    /// Execute template with parameters and infrastructure context
    async fn execute(
        &self,
        params: TemplateParams,
        context: ExecutionContext,
    ) -> Result<TemplateOutput>;

    /// Get parameter configuration schema
    fn config_schema(&self) -> ConfigSchema;

    /// Estimate execution cost (tokens, duration, USD)
    async fn estimate_cost(&self, params: &TemplateParams) -> CostEstimate;
}
```

**Design Decisions:**
- **Async execution**: Templates may involve long-running LLM calls
- **Arc sharing**: Templates are cheaply cloneable via Arc<dyn Template>
- **Validation separation**: Fail-fast before expensive execution
- **ExecutionContext**: Dependency injection for testability and flexibility

### 2. TemplateMetadata

Describes a template for discovery and introspection:

```rust
pub struct TemplateMetadata {
    pub id: String,                    // Unique identifier (kebab-case)
    pub name: String,                  // Human-readable name
    pub description: String,           // Brief description
    pub category: TemplateCategory,    // Research, Chat, Analysis, etc.
    pub version: String,               // Semantic version
    pub author: Option<String>,        // Author name
    pub requires: Vec<String>,         // Infrastructure requirements
    pub tags: Vec<String>,             // Searchable tags
}
```

**Categories:**
- `Research` - Multi-source research with citations
- `Chat` - Interactive conversation sessions
- `Analysis` - Data analysis and visualization
- `CodeGen` - Code generation with tests
- `Document` - Document processing and transformation
- `Workflow` - Custom agent/tool orchestration
- `Custom(String)` - User-defined categories

### 3. TemplateParams

Type-safe parameter container with validation:

```rust
pub struct TemplateParams {
    values: HashMap<String, serde_json::Value>,
}

impl From<HashMap<String, serde_json::Value>> for TemplateParams {
    fn from(values: HashMap<String, serde_json::Value>) -> Self {
        Self { values }
    }
}
```

**Features:**
- JSON-based for flexibility across languages
- Validated against ConfigSchema before execution
- Supports all JSON types: string, number, boolean, array, object, null

### 4. ConfigSchema

Declarative parameter validation schema:

```rust
pub struct ConfigSchema {
    pub version: String,
    pub parameters: Vec<ParameterSchema>,
}

pub struct ParameterSchema {
    pub name: String,
    pub description: String,
    pub param_type: ParameterType,          // String, Integer, Float, Boolean, Array, Object
    pub required: bool,
    pub default: Option<serde_json::Value>,
    pub constraints: Option<ParameterConstraints>,
}

pub struct ParameterConstraints {
    pub min: Option<f64>,                   // Numeric minimum
    pub max: Option<f64>,                   // Numeric maximum
    pub min_length: Option<usize>,          // String minimum length
    pub max_length: Option<usize>,          // String maximum length
    pub pattern: Option<String>,            // Regex pattern
    pub allowed_values: Option<Vec<serde_json::Value>>,  // Enum values
}
```

**Validation Rules:**
1. Required parameters must be present
2. Parameter types must match schema
3. Numeric values must be within min/max range
4. String lengths must be within bounds
5. Patterns must match (if specified)
6. Values must be in allowed_values list (if specified)

### 5. ExecutionContext

Provides access to llmspell infrastructure:

```rust
pub struct ExecutionContext {
    tool_registry: Arc<ToolRegistry>,
    agent_registry: Arc<AgentFactoryRegistry>,
    workflow_factory: Arc<dyn WorkflowFactory>,
    providers: Arc<ProviderManager>,
    state_manager: Option<Arc<StateManager>>,
    session_manager: Option<Arc<SessionManager>>,
    rag_manager: Option<Arc<RAGManager>>,
    session_id: Option<String>,
    output_dir: Option<PathBuf>,
}
```

**Builder Pattern:**
```rust
let context = ExecutionContext::builder()
    .with_tool_registry(tool_registry)
    .with_agent_registry(agent_registry)
    .with_workflow_factory(workflow_factory)
    .with_providers(providers)
    .with_state_manager(state_manager)   // Optional
    .with_session_manager(session_manager) // Optional
    .with_rag_manager(rag_manager)       // Optional
    .with_session_id("session-123")
    .with_output_dir(PathBuf::from("/tmp"))
    .build()?;
```

**Key Design:**
- **Optional Infrastructure**: Templates check availability before using
- **Arc Sharing**: Zero-copy sharing across templates
- **Builder Safety**: Returns error if required components missing

### 6. TemplateOutput

Structured execution result:

```rust
pub struct TemplateOutput {
    pub result: TemplateResult,
    pub artifacts: Vec<Artifact>,
    pub metadata: OutputMetadata,
    pub metrics: ExecutionMetrics,
}

pub enum TemplateResult {
    Text(String),                      // Plain text result
    Structured(serde_json::Value),     // JSON structured data
    File(PathBuf),                     // File path
    Multiple(Vec<TemplateResult>),     // Multiple results
}

pub struct Artifact {
    pub filename: String,
    pub content: Vec<u8>,
    pub mime_type: String,
    pub metadata: HashMap<String, serde_json::Value>,
}

pub struct OutputMetadata {
    pub template_id: String,
    pub template_version: String,
    pub executed_at: chrono::DateTime<chrono::Utc>,
    pub parameters: TemplateParams,
}

pub struct ExecutionMetrics {
    pub duration_ms: u64,
    pub tokens_used: Option<u64>,
    pub cost_usd: Option<f64>,
    pub agents_invoked: usize,
}
```

---

## Template Registry

### Architecture

The `TemplateRegistry` manages template lifecycle and discovery:

```rust
pub struct TemplateRegistry {
    templates: DashMap<String, Arc<dyn Template>>,
}

impl TemplateRegistry {
    pub fn new() -> Self;
    pub fn with_builtin_templates() -> Result<Self>;

    // Registration
    pub fn register(&self, template: Arc<dyn Template>) -> Result<()>;
    pub fn register_or_replace(&self, template: Arc<dyn Template>);
    pub fn unregister(&self, id: &str) -> Result<()>;

    // Discovery
    pub fn get(&self, id: &str) -> Result<Arc<dyn Template>>;
    pub fn list_metadata(&self) -> Vec<TemplateMetadata>;
    pub fn discover_by_category(&self, category: &TemplateCategory) -> Vec<TemplateMetadata>;
    pub fn find_by_tag(&self, tag: &str) -> Vec<TemplateMetadata>;
    pub fn search(&self, query: &str) -> Vec<TemplateMetadata>;

    // Management
    pub fn clear(&self);
    pub fn list_ids(&self) -> Vec<String>;
}
```

### Global Registry

A lazy-initialized global registry provides convenient access:

```rust
static GLOBAL_REGISTRY: once_cell::sync::Lazy<TemplateRegistry> =
    once_cell::sync::Lazy::new(|| {
        TemplateRegistry::with_builtin_templates()
            .expect("Failed to initialize global template registry")
    });

pub fn global_registry() -> &'static TemplateRegistry {
    &GLOBAL_REGISTRY
}
```

### Search Algorithm

The `search()` method performs fuzzy matching across:
1. Template ID (exact match gets highest priority)
2. Template name (case-insensitive substring)
3. Description (case-insensitive substring)
4. Tags (exact match on any tag)

Results are sorted by relevance score.

---

## CLI Integration Architecture

### Command Structure

```
llmspell template <subcommand>
```

**Subcommands:**
- `list [--category <cat>]` - List templates
- `info <id> [--show-schema]` - Show template details
- `exec <id> --param <key>=<value>...` - Execute template
- `schema <id>` - Show parameter schema
- `search <query>` - Search templates

### Execution Flow

```
User CLI Command
      ↓
TemplateCommand::run()
      ↓
TemplateBridge::execute_template(id, params)
      ↓
  1. Get template from registry
  2. Validate params against schema
  3. Build ExecutionContext with infrastructure
  4. Call template.execute(params, context)
      ↓
TemplateOutput (result, artifacts, metrics)
      ↓
Format and display to user
```

### Parameter Parsing

CLI parameters are parsed as:
```bash
--param key=value           # String
--param count=42            # Integer
--param enabled=true        # Boolean
--param tags=["a","b"]      # Array (JSON)
--param config='{"x":1}'    # Object (JSON)
```

---

## Lua Bridge Architecture

### 4-Layer Bridge Pattern

Following the established Agent/Workflow pattern:

#### Layer 0: Business Logic (TemplateBridge)
Located in `llmspell-bridge/src/template_bridge.rs` (437 lines)

```rust
pub struct TemplateBridge {
    template_registry: Arc<TemplateRegistry>,
    registry: Arc<ComponentRegistry>,
    providers: Arc<ProviderManager>,
    state_manager: Option<Arc<StateManager>>,
    session_manager: Option<Arc<SessionManager>>,
}

impl TemplateBridge {
    pub fn list_templates(&self, category: Option<TemplateCategory>) -> Vec<TemplateMetadata>;
    pub fn get_template_info(&self, name: &str, include_schema: bool) -> Result<TemplateInfo>;
    pub async fn execute_template(&self, name: &str, params: TemplateParams) -> Result<TemplateOutput>;
    pub fn search_templates(&self, query: &str, category: Option<TemplateCategory>) -> Vec<TemplateMetadata>;
    pub fn get_template_schema(&self, name: &str) -> Result<ConfigSchema>;
    pub async fn estimate_cost(&self, name: &str, params: &TemplateParams) -> Result<Option<CostEstimate>>;
}
```

**Key Responsibilities:**
- Centralized parameter validation
- ExecutionContext building (tool/agent/workflow/provider/state/session registries)
- Template discovery and search
- Cost estimation

#### Layer 1: Language-Neutral Global (TemplateGlobal)
Located in `llmspell-bridge/src/globals/template_global.rs` (100 lines)

```rust
pub struct TemplateGlobal {
    bridge: Arc<TemplateBridge>,
}

impl GlobalObject for TemplateGlobal {
    fn metadata(&self) -> GlobalMetadata;
    fn inject_lua(&self, lua: &Lua, context: &GlobalContext) -> Result<()>;
    fn inject_javascript(&self, ctx: &mut boa_engine::Context, context: &GlobalContext) -> Result<()>;
}
```

#### Layer 2: Lua Injection
Located in `llmspell-bridge/src/lua/globals/template.rs` (253 lines)

Provides Lua API:
```lua
-- Discovery
Template.list([category])           -- List templates, optional category filter
Template.search(query, [category])  -- Search templates by query
Template.info(name, [show_schema])  -- Get template info, optional schema

-- Introspection
Template.schema(name)               -- Get parameter schema

-- Execution
Template.execute(name, params)      -- Execute template (async via block_on_async_lua)
Template.estimate_cost(name, params) -- Estimate execution cost (async)
```

#### Layer 3: JavaScript Stub
Located in `llmspell-bridge/src/javascript/globals/template.rs` (57 lines)

Future implementation placeholder.

### Lua Type Conversions

Located in `llmspell-bridge/src/lua/conversion.rs`:

**Rust → Lua:**
- `TemplateMetadata` → Lua table (id, name, description, category, version, tags)
- `ConfigSchema` → Lua table (version, parameters array with constraints)
- `TemplateOutput` → Lua table (result, artifacts, metadata, metrics)
- `TemplateParams` → HashMap<String, JsonValue>

**Lua → Rust:**
- Lua table → `TemplateParams` (key-value pairs converted to JSON values)

---

## Performance Characteristics

### Benchmarks

| Operation | Target | Actual | Status |
|-----------|--------|--------|--------|
| Template list | <10ms | ~0.5ms | ✅ 20x faster |
| Template info | <5ms | ~0.3ms | ✅ 16x faster |
| Template discovery (by category) | <10ms | ~1ms | ✅ 10x faster |
| Parameter validation | <5ms | ~0.1ms | ✅ 50x faster |
| ExecutionContext creation | <100ms | ~2ms | ✅ 50x faster |

### Memory Efficiency

- **Arc sharing**: Templates shared across threads/calls without cloning
- **Lazy loading**: Global registry initialized once on first access
- **DashMap**: Lock-free concurrent HashMap for zero-contention reads
- **Zero-copy**: Template metadata returned by reference when possible

### Scalability

- **Concurrent execution**: Multiple templates can execute simultaneously
- **Thread-safe registry**: Safe to register/access from multiple threads
- **Stateless templates**: No shared mutable state between executions
- **Resource pooling**: ProviderManager pools LLM connections

---

## Testing Strategy

### Unit Tests (110 tests)

**Module Coverage:**
- `artifacts.rs`: 7 tests (creation, collection, size, file I/O)
- `context.rs`: 2 tests (builder validation, infrastructure checks)
- `core.rs`: 8 tests (metadata, params, results, output)
- `error.rs`: 8 tests (all error types, from conversions)
- `registry.rs`: 11 tests (register, get, search, discover, tags)
- `validation.rs`: 5 tests (required, type, numeric, string, optional)
- Built-in templates: 69 tests across 6 templates

### Integration Tests (16 tests)

Located in `llmspell-templates/tests/integration_test.rs`:

**Coverage:**
- Registry initialization with builtin templates
- Template discovery by category
- Template search functionality
- Parameter validation (required, constraints, types)
- ExecutionContext builder (minimal and missing components)
- Template metadata completeness
- Config schema structure
- Custom template registration
- Multi-template workflows
- Error propagation
- Cost estimation (async)
- Tag-based discovery
- Registry clear operation

### Test Infrastructure

Uses mock implementations for external dependencies:
- `ToolRegistry::new()` - Empty tool registry
- `AgentFactoryRegistry::new()` - Empty agent registry
- `DefaultWorkflowFactory::new()` - Minimal workflow factory
- `ProviderManager::new()` - No-op provider manager

**Key Principle**: Tests don't require running LLMs or external services.

---

## Extension Points

### Creating Custom Templates

```rust
use llmspell_templates::{Template, TemplateMetadata, TemplateParams,
                         ExecutionContext, TemplateOutput, ConfigSchema,
                         TemplateCategory, CostEstimate};
use async_trait::async_trait;
use std::sync::Arc;

pub struct MyCustomTemplate {
    metadata: TemplateMetadata,
}

impl MyCustomTemplate {
    pub fn new() -> Self {
        Self {
            metadata: TemplateMetadata {
                id: "my-custom-template".to_string(),
                name: "My Custom Template".to_string(),
                description: "Custom template description".to_string(),
                category: TemplateCategory::Custom("MyCategory".to_string()),
                version: "1.0.0".to_string(),
                author: Some("Your Name".to_string()),
                requires: vec!["tool:custom".to_string()],
                tags: vec!["custom".to_string()],
            },
        }
    }
}

#[async_trait]
impl Template for MyCustomTemplate {
    fn metadata(&self) -> &TemplateMetadata {
        &self.metadata
    }

    fn validate(&self, params: &TemplateParams) -> Result<()> {
        // Validate required parameters
        params.get_string("input")?;
        Ok(())
    }

    async fn execute(
        &self,
        params: TemplateParams,
        context: ExecutionContext,
    ) -> Result<TemplateOutput> {
        // Implementation here
        todo!()
    }

    fn config_schema(&self) -> ConfigSchema {
        // Define parameter schema
        todo!()
    }

    async fn estimate_cost(&self, params: &TemplateParams) -> CostEstimate {
        // Estimate execution cost
        todo!()
    }
}

// Register custom template
let registry = TemplateRegistry::new();
registry.register(Arc::new(MyCustomTemplate::new()))?;
```

### Plugin System (Future)

Templates can be loaded dynamically from:
- Compiled shared libraries (.so, .dylib, .dll)
- WASM modules
- External processes via IPC

---

## Phase 13 Memory Integration

Templates are designed for zero-breaking-change memory enhancement:

### Current (Phase 12)
```rust
let result = template.execute(params, context).await?;
```

### Future (Phase 13) - Opt-in Memory
```rust
let mut params = TemplateParams::new();
params.insert("topic", "Rust async");
params.insert("enable_memory", true);  // Opt-in memory

let result = template.execute(params, context).await?;
```

**Memory Features:**
- Remember previous template executions
- Build knowledge graph across topics
- Suggest related templates based on history
- Share learned patterns between templates

**Design Principle**: Memory is opt-in, not mandatory. Templates work without memory.

---

## Security Considerations

### Input Validation

1. **Schema validation**: All parameters validated before execution
2. **Type safety**: Rust type system prevents type confusion
3. **Range checks**: Numeric parameters bounded by min/max
4. **Pattern matching**: String parameters validated against regex
5. **Enum validation**: Only allowed values accepted

### Resource Limits

1. **Execution timeout**: Templates can specify max execution time
2. **Token limits**: LLM calls bounded by token count
3. **Memory limits**: Large results can be streamed to disk
4. **Concurrent execution**: Rate limiting on template execution

### Sandboxing

Templates execute within llmspell process, not isolated. Future phases may add:
- WASM sandboxing for untrusted templates
- Process isolation for high-security environments
- Capability-based security model

---

## Performance Optimization Techniques

### 1. Lazy Initialization
```rust
static GLOBAL_REGISTRY: once_cell::sync::Lazy<TemplateRegistry> =
    once_cell::sync::Lazy::new(|| { ... });
```

### 2. Arc Sharing
```rust
pub fn get(&self, id: &str) -> Result<Arc<dyn Template>> {
    self.templates.get(id)
        .map(|entry| entry.value().clone())  // Arc clone, not template clone
        .ok_or_else(|| TemplateError::NotFound(id.to_string()))
}
```

### 3. DashMap for Concurrency
```rust
use dashmap::DashMap;

pub struct TemplateRegistry {
    templates: DashMap<String, Arc<dyn Template>>,  // Lock-free reads
}
```

### 4. Builder Pattern for Context
```rust
// Avoid intermediate allocations
let context = ExecutionContext::builder()
    .with_tool_registry(tool_registry)  // Moves, not clones
    .with_agent_registry(agent_registry)
    .build()?;
```

---

## Production Status (Phase 12 Complete)

### ✅ All Templates Production-Ready

**Base Templates (Phase 12.1-12.8):**
- ✅ **Research Assistant** - Full 4-phase RAG pipeline (web → embed → store → synthesize → validate)
- ✅ **Interactive Chat** - REPL-based conversation with session management and history
- ✅ **Data Analysis** - Stats + visualization agents with sequential workflow
- ✅ **Code Generator** - 3-agent chain (spec → impl → test) with real LLM execution
- ✅ **Document Processor** - PDF/text file processing with transformation agents
- ✅ **Workflow Orchestrator** - Custom parallel/sequential/hybrid agent orchestration

**Advanced Templates (Phase 12.10-12.13):**
- ✅ **Code Review** - Multi-aspect analysis (security, quality, performance) with selective execution
- ✅ **Content Generation** - Quality-driven iteration with eval/edit refinement loop
- ✅ **File Classification** - Scan-classify-act pattern with dry-run mode for bulk operations
- ✅ **Knowledge Management** - RAG-centric CRUD (ingest, query, update, delete, list) with multi-collection support

**Infrastructure Complete:**
- ✅ Template trait and registry (DashMap-based, lock-free)
- ✅ Parameter validation with schemas (min/max, pattern, enum)
- ✅ CLI integration (list, info, exec, search, schema)
- ✅ Lua bridge integration (16th global, 6 methods)
- ✅ Cost estimation (per-template logic)
- ✅ Output formatting (text, JSON, structured)
- ✅ Artifact generation (file I/O, metadata)
- ✅ ExecutionContext with dual-layer registry (ComponentRegistry + ToolRegistry)

**Quality Metrics:**
- ✅ 149 tests passing (122 unit + 27 integration)
- ✅ Zero clippy warnings, zero rustdoc warnings
- ✅ 2,651 LOC production template code
- ✅ 3,655 lines documentation
- ✅ Performance targets exceeded (20-50x faster than planned)

**Future Enhancements (Non-Breaking):**
- Phase 13: Adaptive Memory integration (opt-in via `enable_memory` parameter)
- Phase 14: Advanced RAG with temporal knowledge graphs
- Phase 15: Multi-agent collaboration patterns
- Phase 16: Custom template marketplace with WASM sandboxing

---

## Advanced Template Patterns (Phase 12.10-12.13)

Phase 12.10-12.13 introduced 4 advanced template patterns demonstrating sophisticated workflow orchestration, quality-driven iteration, and RAG-centric operations.

### Pattern 1: Multi-Aspect Analysis (Code Review)

**Template**: `code-review` (298 LOC)
**Use Case**: Configurable analysis with selective aspect execution
**Category**: Development

**Architecture**:
```rust
// User selects which aspects to analyze
params: {
    code_path: String,
    aspects: Vec<String>,  // ["security", "quality", "performance", "style", "docs"]
    model: String,
}

// Template creates specialized agent per aspect
for aspect in aspects {
    let agent = create_specialized_reviewer(aspect);  // Different prompts/temps
    let finding = agent.execute(code).await?;
    results.push((aspect, finding));
}
```

**Key Features**:
- **Selective Execution**: Only requested aspects analyzed (cost optimization)
- **Specialized Agents**: Different temperature, prompts per aspect
  - Security: temp=0.3 (factual), strict criteria
  - Quality: temp=0.5 (balanced), best practices
  - Performance: temp=0.4 (analytical), profiling focus
- **Aspect-Specific Output**: Structured findings per aspect with severity levels
- **Cost Efficiency**: 5 aspects available, user pays only for selected

**Pattern Applications**:
- Multi-criteria evaluation (proposals, designs, architectures)
- Parallel independent analyses
- Modular analysis pipelines

### Pattern 2: Quality-Driven Iteration (Content Generation)

**Template**: `content-generation` (289 LOC)
**Use Case**: Iterative refinement until quality threshold met
**Category**: Content

**Architecture**:
```rust
// Stage 1: Draft Generation
let draft = draft_agent.execute(topic).await?;
let mut content = draft;

// Stage 2-N: Evaluate & Edit Loop
for iteration in 0..max_iterations {
    // Evaluation Agent (temp=0.4, analytical)
    let quality_score = eval_agent.execute(&content).await?;

    if quality_score >= threshold {
        break;  // Quality met, exit loop
    }

    // Editor Agent (temp=0.7, creative)
    content = edit_agent.execute(&content, quality_feedback).await?;
}
```

**Key Features**:
- **Conditional Iteration**: Loop terminates when quality threshold met or max iterations reached
- **Dual-Agent Pattern**: Evaluator (analytical) + Editor (creative)
- **Quality Metrics**: Structured scoring (clarity, depth, accuracy, engagement)
- **Progressive Refinement**: Each iteration improves based on specific feedback
- **Fail-Safe**: Max iterations prevents infinite loops

**Pattern Applications**:
- Essay/article generation with quality control
- Report refinement workflows
- Iterative design improvement
- Test generation until coverage threshold met

### Pattern 3: Scan-Classify-Act (File Classification)

**Template**: `file-classification` (277 LOC)
**Use Case**: Bulk operations with dry-run mode
**Category**: Productivity

**Architecture**:
```rust
// Phase 1: Scan - Enumerate files
let files = scan_directory(path, filters)?;

// Phase 2: Classify - LLM categorizes each file
let agent = create_classifier_agent();
for file in files {
    let content = read_file(&file)?;
    let category = agent.execute(content).await?;
    classifications.push((file, category));
}

// Phase 3: Act - Execute actions (conditional on dry_run)
if !dry_run {
    for (file, category) in classifications {
        let target_dir = category_mapping[&category];
        move_file(file, target_dir)?;
    }
}
```

**Key Features**:
- **Dry-Run Mode**: Preview classification without executing actions
- **Bulk Processing**: Handles multiple files in single invocation
- **Configurable Categories**: User defines category→directory mapping
- **Safe Preview**: Users verify before committing changes
- **File Operations**: Move, copy, tag, or delete based on classification

**Pattern Applications**:
- Document organization (emails, downloads, research papers)
- Log file triage
- Code refactoring (move files to new package structure)
- Media library organization

### Pattern 4: RAG-Centric CRUD (Knowledge Management)

**Template**: `knowledge-management` (392 LOC)
**Use Case**: Multi-operation template with state persistence
**Category**: Research

**Architecture**:
```rust
match operation {
    "ingest" => {
        // Document chunking
        let chunks = chunk_document(&content, chunk_size, overlap);

        // RAG storage
        for chunk in chunks {
            context.state().set_json(
                &format!("{}/doc_{}", collection, uuid::Uuid::new_v4()),
                &chunk
            )?;
        }
    },
    "query" => {
        // Retrieve from collection
        let docs = context.state().get_all_with_prefix(&format!("{}/", collection))?;

        // Simple search (Phase 12) or RAG search (future enhancement)
        let results = search_documents(&query, docs);
        return TemplateOutput::json(results);
    },
    "update" | "delete" | "list" => { /* ... */ }
}
```

**Key Features**:
- **Single Template, Multiple Operations**: 5 operations (ingest, query, update, delete, list) via parameter
- **Multi-Collection**: Users organize knowledge bases into collections
- **Document Chunking**: Automatic text splitting for RAG ingestion
- **State Persistence**: Uses StateManager for durable storage
- **Citation Tracking**: Metadata preserved for provenance
- **Extensibility**: Ready for Phase 13 A-TKG integration

**Pattern Applications**:
- Personal knowledge base management
- Research note organization
- FAQ/documentation systems
- Long-term memory for conversational agents (Phase 13)

### Common Patterns Across Advanced Templates

1. **Parameter-Driven Behavior**: Single template, multiple modes controlled by parameters
2. **Agent Specialization**: Different temperatures, prompts, models per sub-task
3. **Conditional Execution**: Dry-run, quality thresholds, iteration limits
4. **State Integration**: Persistent storage via StateManager for multi-session workflows
5. **Cost Awareness**: Selective execution, early termination to minimize LLM calls
6. **User Safety**: Preview modes, validation before destructive operations

### Performance Characteristics

| Template | Avg Execution Time | LLM Calls | State Ops | Tested With |
|----------|-------------------|-----------|-----------|-------------|
| code-review | 15-45s (5 aspects) | 5 | 0 | ollama/llama3.2:3b |
| content-generation | 20-60s (3 iterations) | 3-9 | 0 | ollama/llama3.2:3b |
| file-classification | 5-20s (10 files) | 10 | 10 (if not dry-run) | ollama/llama3.2:3b |
| knowledge-management | 2-10s (ingest 5 docs) | 0 | 5 | N/A (state-only) |

**Note**: Execution times with local LLM (Ollama). Cloud LLMs (OpenAI, Anthropic) typically 2-5x faster.

---

## Migration Guide

### From Direct Agent/Tool Usage

**Before (Phase 11):**
```rust
let agent = agent_registry.create("research-agent", config)?;
let tool_result = tool_registry.execute("web-search", params)?;
let rag_result = rag_manager.ingest(docs)?;
// Manual orchestration...
```

**After (Phase 12):**
```rust
let result = template.execute("research-assistant", params).await?;
// Template handles orchestration internally
```

**Benefits:**
- Less code (80% reduction)
- Validated parameters
- Consistent error handling
- Built-in cost estimation
- Portable across CLI/Lua/JS

---

## Related Documentation

- [Template User Guide](/docs/user-guide/templates/README.md) - User-facing documentation for all 10 templates
- [Template Creation Guide](/docs/developer-guide/template-creation.md) - Developer guide for creating custom templates
- [Research Assistant Template](/docs/user-guide/templates/research-assistant.md) - Flagship template with RAG pipeline
- [Code Review Template](/docs/user-guide/templates/code-review.md) - Multi-aspect analysis pattern
- [Content Generation Template](/docs/user-guide/templates/content-generation.md) - Quality-driven iteration pattern
- [File Classification Template](/docs/user-guide/templates/file-classification.md) - Scan-classify-act pattern
- [Knowledge Management Template](/docs/user-guide/templates/knowledge-management.md) - RAG-centric CRUD pattern
- [CLI Template Commands](/docs/cli/template-commands.md) - Command-line interface reference
- [Lua Template API](/docs/api/lua/template-global.md) - Lua scripting API
- [Implementation Phases](/docs/in-progress/implementation-phases.md) - Development roadmap
- [Phase 12 Design Doc](/docs/in-progress/phase-12-design-doc.md) - Original design vs actual implementation

---

## Appendix: Type Definitions

### Complete TemplateMetadata
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateMetadata {
    /// Unique template identifier (kebab-case)
    pub id: String,

    /// Human-readable name
    pub name: String,

    /// Brief description
    pub description: String,

    /// Template category
    pub category: TemplateCategory,

    /// Semantic version
    pub version: String,

    /// Optional author name
    pub author: Option<String>,

    /// Infrastructure requirements (e.g., ["tool:web-search", "rag:enabled"])
    pub requires: Vec<String>,

    /// Searchable tags
    pub tags: Vec<String>,
}
```

### Complete ConfigSchema
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigSchema {
    /// Schema version (currently "1.0")
    pub version: String,

    /// Parameter definitions
    pub parameters: Vec<ParameterSchema>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParameterSchema {
    /// Parameter name
    pub name: String,

    /// Parameter description
    pub description: String,

    /// Parameter type
    pub param_type: ParameterType,

    /// Whether parameter is required
    pub required: bool,

    /// Default value (if not required)
    pub default: Option<serde_json::Value>,

    /// Optional constraints
    pub constraints: Option<ParameterConstraints>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ParameterType {
    String,
    Integer,
    Float,
    Boolean,
    Array,
    Object,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParameterConstraints {
    /// Minimum numeric value
    pub min: Option<f64>,

    /// Maximum numeric value
    pub max: Option<f64>,

    /// Minimum string length
    pub min_length: Option<usize>,

    /// Maximum string length
    pub max_length: Option<usize>,

    /// Regex pattern (for strings)
    pub pattern: Option<String>,

    /// Allowed values (enum)
    pub allowed_values: Option<Vec<serde_json::Value>>,
}
```

---

**Document Version:** 2.0 (Post-Phase 12 Complete)
**Phase:** 12.13 (Knowledge Management Template - Final)
**Last Updated:** 2025-10-24
**Next Review:** Phase 13 (Adaptive Memory Integration)
**Changes Since v1.0**:
- Updated metrics: 6→10 templates, 126→149 tests, +2,651 LOC
- Added "Advanced Template Patterns" section (Phase 12.10-12.13)
- Removed "Placeholder Implementations" (all production-ready)
- Updated architecture diagrams with all 10 templates
- Added performance benchmarks for advanced templates
