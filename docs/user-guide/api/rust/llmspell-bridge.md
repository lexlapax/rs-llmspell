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