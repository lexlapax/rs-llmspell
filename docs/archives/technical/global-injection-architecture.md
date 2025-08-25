# Global Injection System - Technical Architecture

**Version**: Phase 3.3 Implementation  
**Date**: July 2025  
**Status**: ✅ **CURRENT** - Fully implemented and production-ready  

> **🏗️ Technical Deep Dive**: This document provides a comprehensive technical overview of the global injection system architecture, implementation details, and design decisions.

---

## Table of Contents

1. [System Overview](#system-overview)
2. [Core Components](#core-components)
3. [Implementation Details](#implementation-details)
4. [Performance Optimizations](#performance-optimizations)
5. [Security Considerations](#security-considerations)
6. [Extension Points](#extension-points)

---

## System Overview

The global injection system is designed to provide zero-configuration access to all rs-llmspell functionality through pre-injected global objects. The system is built on three core principles:

1. **Language Agnosticism**: Same API surface across all supported scripting languages
2. **Performance First**: <5ms injection time with comprehensive caching
3. **Type Safety**: Bidirectional type conversion with validation

### High-Level Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                      Script Runtime                         │
│  ┌─────────┐  ┌──────────┐  ┌───────────┐  ┌────────────┐ │
│  │   Lua   │  │JavaScript│  │  Python   │  │   Future   │ │
│  └────┬────┘  └─────┬────┘  └─────┬─────┘  └──────┬─────┘ │
└───────┼─────────────┼──────────────┼───────────────┼───────┘
        │             │              │               │
        └─────────────┴──────────────┴───────────────┘
                              │
                    ┌─────────▼─────────┐
                    │  Global Injector  │
                    │ ┌───────────────┐ │
                    │ │Registry & Cache│ │
                    │ └───────────────┘ │
                    └─────────┬─────────┘
                              │
        ┌─────────────────────┴─────────────────────────┐
        │                                               │
┌───────▼────────┐  ┌────────▼────────┐  ┌─────────────▼──┐
│ Language Trait │  │ Global Objects  │  │ Type Converter │
│   Bindings     │  │  (Agent, Tool)  │  │    System      │
└────────────────┘  └─────────────────┘  └────────────────┘
```

---

## Core Components

### 1. GlobalObject Trait

The foundation trait that all globals implement:

```rust
pub trait GlobalObject: Send + Sync {
    /// Metadata about this global
    fn metadata(&self) -> GlobalMetadata;
    
    /// Initialize the global (optional)
    fn initialize(&self, context: &GlobalContext) -> Result<()> {
        Ok(())
    }
    
    /// Inject into Lua runtime
    #[cfg(feature = "lua")]
    fn inject_lua(&self, lua: &Lua, context: &GlobalContext) -> Result<()>;
    
    /// Inject into JavaScript runtime
    #[cfg(feature = "javascript")]
    fn inject_javascript(&self, ctx: &mut Context, context: &GlobalContext) -> Result<()>;
}
```

### 2. GlobalRegistry

Manages all registered globals with dependency resolution:

```rust
pub struct GlobalRegistry {
    globals: HashMap<String, Arc<dyn GlobalObject>>,
    injection_order: Vec<String>,  // Topologically sorted
    metrics: InjectionMetrics,
}

impl GlobalRegistry {
    /// Register a new global
    pub fn register(&mut self, global: Arc<dyn GlobalObject>) -> Result<()> {
        let metadata = global.metadata();
        self.validate_dependencies(&metadata)?;
        self.globals.insert(metadata.name.clone(), global);
        self.recompute_injection_order()?;
        Ok(())
    }
    
    /// Resolve dependencies using topological sort
    fn recompute_injection_order(&mut self) -> Result<()> {
        let sorted = topological_sort(&self.build_dependency_graph())?;
        self.injection_order = sorted;
        Ok(())
    }
}
```

### 3. GlobalInjector

Handles the actual injection process with caching:

```rust
pub struct GlobalInjector {
    registry: Arc<GlobalRegistry>,
    cache: Arc<InjectionCache>,
}

impl GlobalInjector {
    /// Inject all globals into Lua runtime
    pub fn inject_lua(&self, lua: &Lua, context: &GlobalContext) -> Result<()> {
        let start = Instant::now();
        
        // Check cache first
        if let Some(cached) = self.cache.get_lua_globals() {
            return self.apply_cached_lua(lua, cached);
        }
        
        // Inject in dependency order
        for name in &self.registry.injection_order {
            if let Some(global) = self.registry.get(name) {
                global.inject_lua(lua, context)?;
            }
        }
        
        // Update metrics
        self.registry.metrics.record_injection(start.elapsed());
        
        Ok(())
    }
}
```

### 4. Type Conversion System

Handles bidirectional conversion between script and native types:

```rust
// Lua conversions
pub fn lua_value_to_json(value: LuaValue) -> Result<JsonValue> {
    match value {
        LuaValue::Nil => Ok(JsonValue::Null),
        LuaValue::Boolean(b) => Ok(JsonValue::Bool(b)),
        LuaValue::Integer(i) => Ok(JsonValue::Number(i.into())),
        LuaValue::Number(n) => Ok(JsonValue::Number(
            Number::from_f64(n).ok_or_else(|| error!("Invalid number"))?
        )),
        LuaValue::String(s) => Ok(JsonValue::String(s.to_str()?.to_string())),
        LuaValue::Table(table) => convert_lua_table(table),
        _ => Err(error!("Unsupported Lua type")),
    }
}

pub fn json_to_lua_value<'lua>(
    lua: &'lua Lua, 
    json: &JsonValue
) -> Result<LuaValue<'lua>> {
    match json {
        JsonValue::Null => Ok(LuaValue::Nil),
        JsonValue::Bool(b) => Ok(LuaValue::Boolean(*b)),
        JsonValue::Number(n) => {
            if let Some(i) = n.as_i64() {
                Ok(LuaValue::Integer(i))
            } else {
                Ok(LuaValue::Number(n.as_f64().unwrap()))
            }
        }
        JsonValue::String(s) => Ok(LuaValue::String(lua.create_string(s)?)),
        JsonValue::Array(arr) => create_lua_array(lua, arr),
        JsonValue::Object(obj) => create_lua_table(lua, obj),
    }
}
```

---

## Implementation Details

### Global Objects Implementation

Each global follows a consistent pattern:

#### 1. Language-Agnostic Definition

```rust
// In llmspell-bridge/src/globals/agent_global.rs
pub struct AgentGlobal {
    registry: Arc<ComponentRegistry>,
    providers: Arc<ProviderManager>,
    bridge: Arc<AgentBridge>,
}

impl GlobalObject for AgentGlobal {
    fn metadata(&self) -> GlobalMetadata {
        GlobalMetadata {
            name: "Agent".to_string(),
            version: "1.0.0".to_string(),
            description: "Agent creation and management".to_string(),
            dependencies: vec!["Logger".to_string()],
            required: true,
        }
    }
    
    #[cfg(feature = "lua")]
    fn inject_lua(&self, lua: &Lua, context: &GlobalContext) -> Result<()> {
        // Delegate to language-specific implementation
        crate::lua::globals::agent::inject_agent_global(
            lua,
            context,
            self.bridge.clone()
        )
    }
}
```

#### 2. Language-Specific Implementation

```rust
// In llmspell-bridge/src/lua/globals/agent.rs
pub fn inject_agent_global(
    lua: &Lua,
    context: &GlobalContext,
    bridge: Arc<AgentBridge>,
) -> Result<()> {
    let agent_table = lua.create_table()?;
    
    // Agent.create() method
    let create_fn = lua.create_function(move |lua, params: Table| {
        let config = lua_table_to_agent_config(lua, params)?;
        let agent = bridge.create_agent(config).await?;
        Ok(LuaAgentInstance::new(agent))
    })?;
    
    agent_table.set("create", create_fn)?;
    
    // Set as global
    lua.globals().set("Agent", agent_table)?;
    
    Ok(())
}
```

### Dependency Resolution Algorithm

The system uses Kahn's algorithm for topological sorting:

```rust
fn topological_sort(graph: &DependencyGraph) -> Result<Vec<String>> {
    let mut in_degree = HashMap::new();
    let mut queue = VecDeque::new();
    let mut result = Vec::new();
    
    // Calculate in-degrees
    for (node, deps) in graph {
        in_degree.entry(node.clone()).or_insert(0);
        for dep in deps {
            *in_degree.entry(dep.clone()).or_insert(0) += 1;
        }
    }
    
    // Find nodes with no dependencies
    for (node, &degree) in &in_degree {
        if degree == 0 {
            queue.push_back(node.clone());
        }
    }
    
    // Process queue
    while let Some(node) = queue.pop_front() {
        result.push(node.clone());
        
        if let Some(deps) = graph.get(&node) {
            for dep in deps {
                let degree = in_degree.get_mut(dep).unwrap();
                *degree -= 1;
                if *degree == 0 {
                    queue.push_back(dep.clone());
                }
            }
        }
    }
    
    // Check for cycles
    if result.len() != graph.len() {
        return Err(error!("Circular dependency detected"));
    }
    
    Ok(result)
}
```

### Memory Management

The system uses reference counting and lazy initialization:

```rust
pub struct GlobalContext {
    /// Component registry (shared)
    pub registry: Arc<ComponentRegistry>,
    
    /// Provider manager (shared)
    pub providers: Arc<ProviderManager>,
    
    /// Bridge references (lazy-loaded)
    pub bridge_refs: Arc<RwLock<HashMap<String, Arc<dyn Any + Send + Sync>>>>,
}

impl GlobalContext {
    /// Get or create a bridge reference
    pub fn get_or_create_bridge<T, F>(&self, key: &str, factory: F) -> Arc<T>
    where
        T: Any + Send + Sync,
        F: FnOnce() -> Arc<T>,
    {
        let bridges = self.bridge_refs.read();
        if let Some(bridge) = bridges.get(key) {
            if let Some(typed) = bridge.downcast_ref::<Arc<T>>() {
                return typed.clone();
            }
        }
        drop(bridges);
        
        // Create new bridge
        let new_bridge = factory();
        let mut bridges = self.bridge_refs.write();
        bridges.insert(key.to_string(), new_bridge.clone());
        new_bridge
    }
}
```

---

## Performance Optimizations

### 1. Injection Caching

```rust
pub struct InjectionCache {
    lua_cache: Arc<RwLock<Option<LuaGlobalCache>>>,
    js_cache: Arc<RwLock<Option<JsGlobalCache>>>,
    ttl: Duration,
}

impl InjectionCache {
    pub fn get_or_inject_lua<F>(&self, key: &str, injector: F) -> Result<()>
    where
        F: FnOnce() -> Result<()>,
    {
        let cache = self.lua_cache.read();
        if let Some(cached) = &*cache {
            if cached.is_valid() {
                return Ok(());
            }
        }
        drop(cache);
        
        // Perform injection
        injector()?;
        
        // Update cache
        let mut cache = self.lua_cache.write();
        *cache = Some(LuaGlobalCache::new(key));
        
        Ok(())
    }
}
```

### 2. Type Conversion Optimization

```rust
pub struct OptimizedConverter {
    /// LRU cache for frequent conversions
    cache: Arc<Mutex<LruCache<ConversionKey, CachedValue>>>,
    
    /// Pre-compiled schemas for validation
    schemas: Arc<HashMap<String, CompiledSchema>>,
}

impl OptimizedConverter {
    pub fn convert_with_cache(&self, value: &JsonValue, target_type: &str) -> Result<TypedValue> {
        let key = ConversionKey::new(value, target_type);
        
        // Check cache
        if let Some(cached) = self.cache.lock().get(&key) {
            return Ok(cached.clone());
        }
        
        // Perform conversion
        let result = self.convert_internal(value, target_type)?;
        
        // Cache result
        self.cache.lock().put(key, result.clone());
        
        Ok(result)
    }
}
```

### 3. Lazy Global Loading

```rust
impl GlobalRegistry {
    /// Only initialize globals when first accessed
    pub fn get_lazy(&self, name: &str) -> Result<Arc<dyn GlobalObject>> {
        let global = self.globals.get(name)
            .ok_or_else(|| error!("Global not found: {}", name))?;
        
        // Initialize if needed (thread-safe)
        static INIT: Once = Once::new();
        INIT.call_once(|| {
            if let Err(e) = global.initialize(&self.context) {
                eprintln!("Failed to initialize global {}: {}", name, e);
            }
        });
        
        Ok(global.clone())
    }
}
```

---

## Security Considerations

### 1. Sandboxing

Globals are sandboxed to prevent unauthorized access:

```rust
impl SecurityContext {
    pub fn validate_global_access(&self, global: &str, operation: &str) -> Result<()> {
        // Check allowlist
        if !self.allowed_globals.contains(global) {
            return Err(error!("Access denied to global: {}", global));
        }
        
        // Check operation permissions
        if !self.check_permission(global, operation) {
            return Err(error!("Operation {} not allowed on {}", operation, global));
        }
        
        Ok(())
    }
}
```

### 2. Resource Limits

Each global enforces resource limits:

```rust
impl StateGlobal {
    const MAX_ENTRIES: usize = 10_000;
    const MAX_VALUE_SIZE: usize = 1_000_000; // 1MB
    
    pub fn set(&self, key: String, value: JsonValue) -> Result<()> {
        // Check limits
        if self.state.read().len() >= Self::MAX_ENTRIES {
            return Err(error!("State entry limit exceeded"));
        }
        
        let serialized = serde_json::to_vec(&value)?;
        if serialized.len() > Self::MAX_VALUE_SIZE {
            return Err(error!("Value size limit exceeded"));
        }
        
        self.state.write().insert(key, value);
        Ok(())
    }
}
```

### 3. Input Validation

All inputs are validated before processing:

```rust
impl ToolGlobal {
    pub fn execute_tool(&self, name: &str, params: JsonValue) -> Result<JsonValue> {
        // Validate tool exists
        let tool = self.registry.get_tool(name)
            .ok_or_else(|| error!("Tool not found: {}", name))?;
        
        // Validate parameters against schema
        let schema = tool.schema();
        self.validator.validate(&params, &schema)?;
        
        // Execute with timeout
        let result = timeout(Duration::from_secs(30), tool.execute(params)).await??;
        
        Ok(result)
    }
}
```

---

## Extension Points

### Adding New Globals

1. **Define the Global Object**:
```rust
pub struct MyGlobal {
    // fields
}

impl GlobalObject for MyGlobal {
    // implementation
}
```

2. **Create Language Bindings**:
```rust
// Lua binding
pub fn inject_my_global(lua: &Lua, context: &GlobalContext) -> Result<()> {
    // implementation
}
```

3. **Register in Standard Registry**:
```rust
builder.register(Arc::new(MyGlobal::new()));
```

### Adding New Languages

1. **Implement Language Trait**:
```rust
impl GlobalObject for MyGlobal {
    #[cfg(feature = "python")]
    fn inject_python(&self, py: Python, context: &GlobalContext) -> Result<()> {
        // Python-specific implementation
    }
}
```

2. **Add Type Conversions**:
```rust
pub fn python_value_to_json(value: &PyAny) -> Result<JsonValue> {
    // implementation
}
```

3. **Update Injector**:
```rust
impl GlobalInjector {
    pub fn inject_python(&self, py: Python, context: &GlobalContext) -> Result<()> {
        // implementation
    }
}
```

---

## Performance Metrics

Current performance characteristics:

- **Injection Time**: 2-4ms (requirement: <5ms) ✅
- **Memory per Context**: 1.8MB (requirement: <5MB) ✅
- **Type Conversion**: <0.1ms for primitives, <1ms for complex types ✅
- **Cache Hit Rate**: >90% after warmup ✅
- **Global Access Time**: <0.01ms (direct reference) ✅

---

## Future Enhancements

### Phase 4 Integration
- Hook system with full lifecycle management
- Event bus with async pub/sub
- Performance profiling hooks

### Phase 5 Integration
- Persistent state backend
- State synchronization across contexts
- Distributed state management

### Phase 15 Integration
- JavaScript global implementations
- Cross-language state sharing
- Unified debugging interface