# Script Engine Architecture Analysis

**Date**: 2025-06-26  
**Context**: Phase 1.2 implementation planning  
**Issue**: Current design has architectural concerns for multi-language support

---

## Current Design Analysis

From `phase-01-design-doc.md`, the current design shows:

```rust
// llmspell-bridge/src/runtime.rs
pub struct ScriptRuntime {
    lua: Arc<Mutex<Lua>>,  // ⚠️ Directly Lua-coupled!
    registry: Arc<ComponentRegistry>,
    provider_manager: Arc<ProviderManager>,
    execution_context: Arc<RwLock<ExecutionContext>>,
}
```

**Issue Identified**: The current design has `ScriptRuntime` directly coupled to Lua, which violates the multi-language goal mentioned in Phase 5.

---

## Recommended Directory Structure

Based on the Phase 5 mention of `ScriptEngineBridge` trait, here's how it should be restructured:

```
llmspell-bridge/
├── src/
│   ├── lib.rs
│   ├── runtime.rs                    # Language-agnostic ScriptRuntime
│   ├── engine/                       # Language abstraction layer
│   │   ├── mod.rs
│   │   ├── bridge.rs                 # ScriptEngineBridge trait
│   │   ├── types.rs                  # Common script types
│   │   └── executor.rs               # Common execution patterns
│   ├── lua/                          # Lua-specific implementation
│   │   ├── mod.rs
│   │   ├── engine.rs                 # LuaEngine: ScriptEngineBridge
│   │   ├── api/
│   │   │   ├── mod.rs
│   │   │   ├── agent.rs              # Agent.create(), agent:execute()
│   │   │   ├── tool.rs               # Tool.get(), tool:execute()
│   │   │   ├── workflow.rs           # Workflow patterns
│   │   │   └── streaming.rs          # Coroutine-based streaming
│   │   └── types.rs                  # Lua ↔ Rust type conversions
│   ├── javascript/                   # Future JS implementation
│   │   ├── mod.rs
│   │   ├── engine.rs                 # JSEngine: ScriptEngineBridge
│   │   ├── api/                      # Same API structure as Lua
│   │   │   ├── agent.rs              # Promise-based agents
│   │   │   ├── tool.rs
│   │   │   ├── workflow.rs
│   │   │   └── streaming.rs          # Async generator streaming
│   │   └── types.rs                  # JS ↔ Rust type conversions
│   └── python/                       # Future Python (via pyo3)
│       └── ...
```

---

## Language-Agnostic Architecture

The `ScriptRuntime` should be redesigned as:

```rust
// llmspell-bridge/src/runtime.rs
pub struct ScriptRuntime {
    engine: Box<dyn ScriptEngineBridge>,  // Language-agnostic!
    registry: Arc<ComponentRegistry>,
    provider_manager: Arc<ProviderManager>,
    execution_context: Arc<RwLock<ExecutionContext>>,
}

// llmspell-bridge/src/engine/bridge.rs
#[async_trait]
pub trait ScriptEngineBridge: Send + Sync {
    async fn execute_script(&self, script: &str) -> Result<ScriptOutput>;
    async fn execute_script_streaming(&self, script: &str) -> Result<ScriptStream>;
    fn inject_apis(&mut self, registry: &ComponentRegistry, providers: &ProviderManager) -> Result<()>;
    fn get_engine_name(&self) -> &'static str;
    fn supports_streaming(&self) -> bool;
}
```

---

## Language-Specific Implementations

**Lua Engine**:
```rust
// llmspell-bridge/src/lua/engine.rs
pub struct LuaEngine {
    lua: Arc<Mutex<Lua>>,
}

impl ScriptEngineBridge for LuaEngine {
    async fn execute_script(&self, script: &str) -> Result<ScriptOutput> {
        // Lua-specific execution logic
    }
    
    fn inject_apis(&mut self, registry: &ComponentRegistry, providers: &ProviderManager) -> Result<()> {
        // Call lua/api/* modules to inject APIs
    }
}
```

**JavaScript Engine** (Future):
```rust
// llmspell-bridge/src/javascript/engine.rs
pub struct JSEngine {
    context: boa::Context,  // or quickjs context
}

impl ScriptEngineBridge for JSEngine {
    // Same interface, different implementation
}
```

---

## API Injection Separation

Each language has its own API injection modules:

```rust
// llmspell-bridge/src/lua/api/agent.rs
pub fn inject_agent_api(lua: &Lua, registry: &ComponentRegistry) -> Result<()> {
    // Lua-specific Agent API injection
}

// llmspell-bridge/src/javascript/api/agent.rs  
pub fn inject_agent_api(ctx: &mut boa::Context, registry: &ComponentRegistry) -> Result<()> {
    // JavaScript-specific Agent API injection
}
```

---

## Runtime Factory Pattern

```rust
// llmspell-bridge/src/runtime.rs
impl ScriptRuntime {
    pub async fn new_with_lua(config: RuntimeConfig) -> Result<Self> {
        let engine = Box::new(LuaEngine::new()?);
        Self::new_with_engine(engine, config).await
    }
    
    pub async fn new_with_javascript(config: RuntimeConfig) -> Result<Self> {
        let engine = Box::new(JSEngine::new()?);
        Self::new_with_engine(engine, config).await
    }
    
    async fn new_with_engine(mut engine: Box<dyn ScriptEngineBridge>, config: RuntimeConfig) -> Result<Self> {
        let registry = Arc::new(ComponentRegistry::new());
        let provider_manager = Arc::new(ProviderManager::new(config.providers)?);
        
        engine.inject_apis(&registry, &provider_manager)?;
        
        Ok(Self {
            engine,
            registry,
            provider_manager,
            execution_context: Arc::new(RwLock::new(ExecutionContext::new())),
        })
    }
}
```

---

## Third-Party Extensibility

**Plugin Interface**:
```rust
// llmspell-bridge/src/engine/plugin.rs
pub trait ScriptEnginePlugin {
    fn engine_name() -> &'static str;
    fn create_engine(config: Value) -> Result<Box<dyn ScriptEngineBridge>>;
    fn supported_features() -> EngineFeatures;
}

// Third parties can implement:
// my-ruby-engine/src/lib.rs
pub struct RubyEngine { ... }
impl ScriptEnginePlugin for RubyEngine { ... }
```

---

## Assessment: Architecture Needs Modification

**Current Issue**: The existing design documents show `ScriptRuntime` directly coupled to Lua, which would make adding JavaScript/Python difficult.

**Recommendation**: 
1. **Refactor ScriptRuntime** to use `ScriptEngineBridge` trait
2. **Separate language implementations** into dedicated modules
3. **Abstract API injection** through the bridge pattern
4. **Use factory pattern** for runtime creation with different engines

This restructuring would make the crate architecture properly extensible for multi-language support while maintaining clean separation of concerns.

---

## Implementation Strategy for Phase 1.2

**For Phase 1.2 (immediate)**: Start with the proper abstraction layer even though only implementing Lua:

1. Create `ScriptEngineBridge` trait first
2. Implement `LuaEngine` as the first concrete implementation
3. Design API injection to be language-agnostic from day one
4. Use factory pattern even for single engine

This ensures Phase 5 (JavaScript) can be added cleanly without major refactoring.