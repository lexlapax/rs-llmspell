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

```rust
// Register all standard globals
engine.register_globals(vec![
    ("Agent", Box::new(AgentGlobal)),
    ("Tool", Box::new(ToolGlobal)),
    ("Workflow", Box::new(WorkflowGlobal)),
    ("State", Box::new(StateGlobal)),
    ("Session", Box::new(SessionGlobal)),
    ("RAG", Box::new(RAGGlobal)),
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
- [llmspell-cli](llmspell-cli.md) - CLI using the bridge