# llmspell-bridge

**Script language integration and Lua bridge**

**🔗 Navigation**: [← Rust API](README.md) | [Crate Docs](https://docs.rs/llmspell-bridge) | [Source](../../../../llmspell-bridge)

---

## Overview

`llmspell-bridge` provides the integration layer between Rust components and scripting languages, primarily Lua. It handles type conversion, global injection, and performance optimization.

**Key Features:**
- 🌉 Lua <-> Rust bridging
- 🔄 Automatic type conversion
- 📦 Global object injection
- ⚡ <1% performance overhead
- 🎯 Error propagation
- 📊 Bridge metrics
- 🔐 Sandboxed execution
- 🧩 Extensible for other languages

## ScriptRuntime (Phase 13b.16)

**Primary API**: Self-contained script execution with unified infrastructure creation.

```rust
use llmspell_bridge::ScriptRuntime;
use llmspell_config::LLMSpellConfig;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Load configuration
    let config = LLMSpellConfig::from_file("config.toml")?;

    // Create ScriptRuntime (creates ALL infrastructure internally)
    let runtime = ScriptRuntime::new(config.clone()).await?;

    // Execute script
    let output = runtime.execute_script(r#"
        local result = Agent.query("What is Rust?")
        return result
    "#).await?;

    println!("Result: {:?}", output.output);
    Ok(())
}
```

### ScriptRuntime API

| Method | Description | Phase |
|--------|-------------|-------|
| `ScriptRuntime::new(config)` | Create with Infrastructure module | 13b.16 |
| `execute_script(code)` | Execute inline code | 1.0 |
| `execute_file(path)` | Execute script file | 1.0 |
| `supports_streaming()` | Check streaming support | 1.0 |
| `execute_script_streaming(code)` | Stream execution | 1.0 |

**Migration from old API**:
```rust
// ❌ DEPRECATED (Pre-13b.16)
let runtime = ScriptRuntime::new_with_lua(config).await?;

// ✅ NEW API (Phase 13b.16+)
let runtime = ScriptRuntime::new(config).await?;
```

## Infrastructure Module (Phase 13b.16)

**Purpose**: Unified component creation from configuration.

```rust
use llmspell_bridge::infrastructure::Infrastructure;
use llmspell_config::LLMSpellConfig;

let config = LLMSpellConfig::from_file("config.toml")?;

// Create all 9 components from config
let infrastructure = Infrastructure::from_config(&config).await?;

// Access components
let provider_manager = infrastructure.provider_manager();
let state_manager = infrastructure.state_manager();
let session_manager = infrastructure.session_manager();

// Optional components (if enabled in config)
if let Some(rag) = infrastructure.rag() {
    println!("RAG enabled");
}

if let Some(memory_manager) = infrastructure.memory_manager() {
    println!("Memory enabled");
}

// Registries
let tool_registry = infrastructure.tool_registry();
let agent_registry = infrastructure.agent_registry();
let workflow_factory = infrastructure.workflow_factory();
let component_registry = infrastructure.component_registry();
```

### Infrastructure Components (9 total)

| Component | Required | Created If | Purpose |
|-----------|----------|-----------|---------|
| `ProviderManager` | ✅ | Always | LLM provider connections |
| `StateManager` | ✅ | Always | Persistent key-value state |
| `SessionManager` | ✅ | Always | Session lifecycle management |
| `RAG` | ❌ | `config.rag.enabled` | Vector similarity search |
| `MemoryManager` | ❌ | `config.runtime.memory.enabled` | Adaptive memory system |
| `ToolRegistry` | ✅ | Always | Tool discovery |
| `AgentRegistry` | ✅ | Always | Agent factories |
| `WorkflowFactory` | ✅ | Always | Workflow creation |
| `ComponentRegistry` | ✅ | Always | Script access layer |

**Code Reference**: `llmspell-bridge/src/infrastructure.rs:107-161`

## ScriptEngine Trait

```rust
#[async_trait]
pub trait ScriptEngine: Send + Sync {
    /// Execute script code
    async fn execute(&self, code: &str) -> Result<Value>;

    /// Execute script file
    async fn execute_file(&self, path: &Path) -> Result<Value>;

    /// Register global object
    fn register_global(&self, name: &str, object: Box<dyn ScriptObject>) -> Result<()>;

    /// Get script context
    fn context(&self) -> &ScriptContext;
}
```

## Lua Integration

```rust
use llmspell_bridge::lua::{LuaEngine, LuaGlobals};

// Create Lua engine
let engine = LuaEngine::new(LuaConfig {
    memory_limit: Some(512 * 1024 * 1024), // 512MB
    globals: LuaGlobals::all(),
    sandbox: true,
})?;

// Execute Lua code
let result = engine.execute(r#"
    local agent = Agent.builder()
        :model("openai/gpt-4")
        :build()
    
    return agent:execute({prompt = "Hello!"})
"#).await?;
```

## Global Registration

**Phase 12**: Template global added as 16th global (now 18 total with ARGS and Streaming).

```rust
// Register all standard globals (18 total - Phase 12)
engine.register_globals(vec![
    ("Agent", Box::new(AgentGlobal)),
    ("Tool", Box::new(ToolGlobal)),
    ("Workflow", Box::new(WorkflowGlobal)),
    ("State", Box::new(StateGlobal)),
    ("Session", Box::new(SessionGlobal)),
    ("RAG", Box::new(RAGGlobal)),
    ("Hook", Box::new(HookGlobal)),
    ("Event", Box::new(EventGlobal)),
    ("Config", Box::new(ConfigGlobal)),
    ("Provider", Box::new(ProviderGlobal)),
    ("Debug", Box::new(DebugGlobal)),
    ("JSON", Box::new(JSONGlobal)),
    ("Template", Box::new(TemplateGlobal)),  // Phase 12
    ("ARGS", Box::new(ARGSGlobal)),
    ("Streaming", Box::new(StreamingGlobal)),
    ("Artifact", Box::new(ArtifactGlobal)),
    ("Replay", Box::new(ReplayGlobal)),
    ("Metrics", Box::new(MetricsGlobal)),
])?;

// Register custom global
engine.register_global("Custom", Box::new(CustomGlobal {
    // implementation
}))?;
```

## Type Conversion

```rust
use llmspell_bridge::lua::conversion::{ToLua, FromLua};

// Rust to Lua
impl ToLua for MyStruct {
    fn to_lua(self, lua: &Lua) -> LuaResult<LuaValue> {
        let table = lua.create_table()?;
        table.set("field", self.field)?;
        Ok(LuaValue::Table(table))
    }
}

// Lua to Rust
impl FromLua for MyStruct {
    fn from_lua(value: LuaValue, lua: &Lua) -> LuaResult<Self> {
        match value {
            LuaValue::Table(table) => {
                Ok(MyStruct {
                    field: table.get("field")?,
                })
            }
            _ => Err("Expected table".into())
        }
    }
}
```

## Error Handling

```rust
// Errors propagate with stack traces
let result = engine.execute(r#"
    local function inner()
        error("Something went wrong")
    end
    
    local function outer()
        inner()
    end
    
    outer()
"#).await;

match result {
    Err(e) => {
        // Full Lua stack trace available
        println!("Error: {}", e);
        println!("Stack trace:\n{}", e.stack_trace());
    }
    Ok(_) => {}
}
```

## Performance Optimization

- **JIT Compilation**: LuaJIT for native performance
- **Lazy Loading**: Globals loaded on-demand
- **Memory Pooling**: Reuse Lua states
- **Type Caching**: Cache converted types
- **Minimal Overhead**: <1% vs native Rust

## RAG Bridge with Temporal Support

The RAG bridge enables vector storage operations from Lua with full temporal metadata support:

```rust
use llmspell_bridge::rag_bridge::{RAGBridge, RAGSearchParams, RAGIngestParams};

// Initialize RAG bridge
let rag_bridge = RAGBridge::new(
    state_manager,
    session_manager,
    multi_tenant_rag,
    provider_manager,
    vector_storage, // HNSWVectorStorage with temporal support
);

// Search with temporal awareness (from Rust)
let params = RAGSearchParams {
    query: "recent updates".to_string(),
    k: Some(10),
    scope: Some("tenant".to_string()),
    scope_id: Some("org-123".to_string()),
    filters: None,
    threshold: Some(0.8),
    context: None,
};

let results = rag_bridge.search(params).await?;

// Ingest with temporal metadata
let ingest_params = RAGIngestParams {
    documents: vec![
        Document {
            content: "Important update".to_string(),
            metadata: hashmap! {
                "timestamp" => json!(1699564800), // Unix timestamp for event_time
                "ttl" => json!(86400),            // Expire after 24 hours
                "source" => json!("api"),
            },
        }
    ],
    options: IngestOptions {
        chunk_size: 500,
        chunk_overlap: 50,
        tenant_id: Some("org-123".to_string()),
    },
};

rag_bridge.ingest(ingest_params).await?;
```

### Temporal Metadata Extraction

The RAG bridge automatically extracts temporal metadata from documents:

```rust
// In rag_bridge.rs - temporal field extraction
if let Some(timestamp_val) = metadata.get("timestamp") {
    if let Some(timestamp_num) = timestamp_val.as_u64() {
        // Convert Unix timestamp to SystemTime
        let duration = std::time::Duration::from_secs(timestamp_num);
        if let Some(event_time) = std::time::UNIX_EPOCH.checked_add(duration) {
            entry = entry.with_event_time(event_time);
        }
    } else if let Some(timestamp_str) = timestamp_val.as_str() {
        // Parse ISO 8601 timestamp
        if let Ok(parsed) = chrono::DateTime::parse_from_rfc3339(timestamp_str) {
            let event_time = std::time::SystemTime::from(parsed);
            entry = entry.with_event_time(event_time);
        }
    }
}

// TTL extraction from multiple possible fields
if let Some(ttl_val) = metadata.get("ttl")
    .or_else(|| metadata.get("ttl_seconds"))
    .or_else(|| metadata.get("expires_in")) 
{
    if let Some(ttl_seconds) = ttl_val.as_u64() {
        entry = entry.with_ttl(ttl_seconds);
    }
}
```

### Lua RAG Global with Temporal Support

From Lua scripts, temporal metadata is transparently handled:

```lua
-- Ingest with temporal metadata
RAG.ingest({
    content = "System update notification",
    metadata = {
        timestamp = os.time() - 3600,  -- Event from 1 hour ago
        ttl = 7200,                    -- Expires in 2 hours
        source = "monitoring"
    }
})

-- Search (temporal queries coming in future update)
local results = RAG.search("recent updates", {
    limit = 5,
    scope = "tenant",
    scope_id = "org-123",
    -- Future: event_time_range, exclude_expired
})
```

## Template Bridge ⭐ **Phase 12**

**Production-Ready AI Workflow Templates** - The Template global provides Lua access to 10 built-in templates and custom template execution.

### TemplateBridge Implementation

The Template bridge follows the 4-layer bridge pattern (Agent/Workflow style, not Tool style):

```rust
use llmspell_bridge::lua::globals::template::TemplateBridge;
use llmspell_templates::{TemplateRegistry, TemplateParams, ExecutionContext};

pub struct TemplateBridge {
    registry: Arc<TemplateRegistry>,
}

impl TemplateBridge {
    /// List templates, optionally filtered by category
    pub fn list_templates(&self, category: Option<TemplateCategory>) -> Vec<TemplateMetadata> {
        self.registry.list(category)
    }

    /// Get template information
    pub fn get_template_info(&self, name: &str, show_schema: bool) -> Result<serde_json::Value> {
        let template = self.registry.get(name)?;
        let mut info = json!({
            "name": template.metadata().name,
            "description": template.metadata().description,
            "category": template.metadata().category,
            "version": template.metadata().version,
        });

        if show_schema {
            info["schema"] = json!(template.config_schema());
        }

        Ok(info)
    }

    /// Execute template
    pub async fn execute_template(
        &self,
        name: &str,
        params: TemplateParams,
        context: ExecutionContext,
    ) -> Result<TemplateOutput> {
        let template = self.registry.get(name)?;
        template.execute(params, context).await
    }
}
```

### Lua Template Global

From Lua scripts, templates are accessed via the `Template` global (16th global):

```lua
-- List all templates
local templates = Template.list()
for _, t in ipairs(templates) do
    print(t.name .. " - " .. t.description)
end

-- List by category
local research_templates = Template.list("research")

-- Get template info with schema
local info = Template.info("research-assistant", true)
print("Parameters: " .. JSON.encode(info.schema.parameters))

-- Execute template
local result = Template.execute("research-assistant", {
    topic = "Rust async programming",
    max_sources = 10,
    model = "ollama/llama3.2:3b"
})

-- Search templates
local matches = Template.search("code", "codegen")
for _, t in ipairs(matches) do
    print("Found: " .. t.name)
end

-- Get parameter schema
local schema = Template.schema("data-analysis")
print("Required params: " .. JSON.encode(schema.required))

-- Estimate cost before execution
local estimate = Template.estimate_cost("research-assistant", {
    topic = "AI research",
    max_sources = 20
})
print("Estimated cost: $" .. estimate.estimated_cost_usd)
```

### Template Categories

The bridge supports 6 template categories plus custom:

```rust
pub enum TemplateCategory {
    Research,    // research-assistant, knowledge-management
    Chat,        // interactive-chat
    Analysis,    // data-analysis
    CodeGen,     // code-generator, code-review
    Document,    // document-processor, content-generation
    Workflow,    // workflow-orchestrator, file-classification
    Custom(String),
}
```

### Template Methods

| Method | Signature | Description |
|--------|-----------|-------------|
| `list` | `([category]) -> table[]` | List templates, optionally filtered |
| `info` | `(name, [show_schema]) -> table` | Get template metadata and schema |
| `execute` | `(name, params) -> table` | Execute template with parameters |
| `search` | `(query, [category]) -> table[]` | Search templates by text |
| `schema` | `(name) -> table` | Get parameter schema |
| `estimate_cost` | `(name, params) -> table` | Estimate execution cost |

## Typed Configuration Pattern ⭐ **Phase 11a.8**

**Anti-pattern eliminated**: JSON parameters for configuration (pre-11a.8)
**Pattern established**: Typed Rust structs with Lua layer parsers

### The Pattern

1. **Bridge layer**: Define typed Rust structs for configuration
2. **Lua layer**: Create parser functions that convert Lua tables to structs
3. **Method signatures**: Accept typed structs directly, not `serde_json::Value`

### Example: Session Replay Configuration

**Bridge Layer** (`session_bridge.rs`):
```rust
use llmspell_kernel::sessions::replay::session_adapter::SessionReplayConfig;

impl SessionBridge {
    pub async fn replay_session(
        &self,
        session_id: &SessionId,
        config: SessionReplayConfig,  // ✅ Typed struct, not JSON
    ) -> Result<serde_json::Value> {
        let result = convert_err!(
            self.session_manager.replay_session(session_id, config).await
        )?;
        // ... convert result to JSON for Lua
    }
}
```

**Lua Layer** (`lua/globals/session.rs`):
```rust
fn parse_session_replay_config(table: &Table) -> mlua::Result<SessionReplayConfig> {
    let mode_str: String = table.get("mode").unwrap_or_else(|_| "exact".to_string());
    let mode = match mode_str.as_str() {
        "exact" => ReplayMode::Exact,
        "modified" => ReplayMode::Modified,
        "simulate" => ReplayMode::Simulate,
        "debug" => ReplayMode::Debug,
        _ => return Err(mlua::Error::RuntimeError(format!("Unknown mode: {mode_str}")))
    };

    Ok(SessionReplayConfig {
        mode,
        compare_results: table.get("compare_results").unwrap_or(false),
        timeout_seconds: table.get("timeout_seconds").ok(),
        stop_on_error: table.get("stop_on_error").unwrap_or(false),
        metadata: table.get::<_, Option<Table>>("metadata").ok()
            .and_then(|t| lua_table_to_json(&t).ok())
            .unwrap_or_default(),
    })
}

// Lua binding
let replay_fn = lua.create_async_function(move |lua, (session_id_str, options_table): (String, Table)| {
    async move {
        let session_id = SessionId::new_v4(); // Parse ID
        let config = parse_session_replay_config(&options_table)?;  // ✅ Parse to struct

        let result = bridge.replay_session(&session_id, config).await?;  // ✅ Pass struct
        Ok(result)
    }
})?;
```

### Pattern Benefits

1. **Compile-Time Validation**: Rust compiler catches all config field errors
2. **Zero Serialization Overhead**: Direct struct passing, no JSON intermediate
3. **Clear Error Messages**: mlua reports exact Lua field issues
4. **IDE Support**: Full autocomplete for struct construction
5. **Refactoring Safety**: Breaking changes caught at compile time
6. **Self-Documentation**: Struct fields show API contract explicitly

### Bridge Pattern Implementations (Phase 11a.8)

**Converted Methods** (7 total):
- `AgentBridge::create_agent` - Uses `AgentConfig`
- `AgentBridge::create_composite_agent` - Uses `RoutingConfig`
- `AgentBridge::create_context` - Uses `ExecutionContextConfig`
- `AgentBridge::create_child_context` - Uses `ChildContextConfig`
- `AgentBridge::set_shared_memory` - Uses `ContextScope`
- `AgentBridge::wrap_agent_as_tool` - Uses `ToolWrapperConfig`
- `SessionBridge::replay_session` - Uses `SessionReplayConfig`

**Reusable Parsers** (11 total):
Located in `lua/globals/agent.rs`:
- `parse_context_scope()` - Parses "global" | "session" | "tenant" | custom scopes
- `parse_inheritance_policy()` - Parses "inherit" | "replace" | "merge"
- `parse_model_config()` - Parses model, temperature, max_tokens, etc.
- `parse_resource_limits()` - Parses timeout, max_memory, retries
- `parse_agent_config()` - Full agent configuration
- `parse_routing_config()` - Composite agent routing
- `parse_execution_context_config()` - Execution context settings
- `parse_child_context_config()` - Child context creation
- `parse_tool_wrapper_config()` - Tool wrapping settings
- `parse_alert_condition()` - Alert condition variants
- `parse_alert_config()` - Full alert configuration

Located in `lua/globals/session.rs`:
- `parse_session_replay_config()` - Session replay options

### Anti-Patterns Eliminated

**Before Phase 11a.8** ❌:
```rust
// DON'T DO THIS - Anti-pattern
pub async fn create_agent(&self, options: serde_json::Value) -> Result<AgentId> {
    let name = options["name"].as_str().unwrap();  // ❌ Runtime errors
    let model = options["model"].as_str().unwrap();  // ❌ No type safety
    // ...
}
```

**After Phase 11a.8** ✅:
```rust
// DO THIS - Typed pattern
pub struct AgentConfig {
    pub name: String,
    pub model_config: ModelConfig,
    pub resource_limits: Option<ResourceLimits>,
}

pub async fn create_agent(&self, config: AgentConfig) -> Result<AgentId> {
    let name = config.name;  // ✅ Compile-time validated
    let model = config.model_config.model;  // ✅ Type-safe access
    // ...
}
```

For complete bridge pattern guidance, see [Bridge Pattern Guide](../../../developer-guide/bridge-pattern-guide.md).

## Sandboxing

```rust
let sandbox_config = SandboxConfig {
    allow_io: false,
    allow_network: false,
    allow_process: false,
    memory_limit: Some(100 * 1024 * 1024), // 100MB
    cpu_limit: Some(Duration::from_secs(10)),
};

let engine = LuaEngine::with_sandbox(sandbox_config)?;
```

## Related Documentation

- [Lua API](../lua/README.md) - Lua scripting API
- [Bridge Pattern Guide](../../../developer-guide/bridge-pattern-guide.md) - Comprehensive pattern documentation
- [llmspell-cli](llmspell-cli.md) - CLI using the bridge