# Global Object Injection Infrastructure Design

## Current State Analysis

### Existing Architecture
The current llmspell-bridge has partial global injection:
- Individual `inject_*_api` functions for each component (Agent, Tool, Workflow, JSON, Streaming)
- Each API has a definition struct in `engine/types.rs`
- APIs are injected during engine initialization via `inject_apis()`
- Language-specific implementations in `lua/globals/*` and `javascript/globals/*`

### Issues with Current Approach
1. **Scattered Implementation**: Each API requires separate injection function
2. **Manual Management**: Adding new globals requires modifying multiple files
3. **No Performance Optimization**: Each injection is done separately
4. **Limited Extensibility**: Hard to add custom globals or plugins
5. **Inconsistent Patterns**: Different APIs have different injection patterns

## Proposed Architecture

### Core Design Principles
1. **Centralized Management**: Single global registry for all injectable objects
2. **Performance Optimized**: Batch injection with <5ms overhead
3. **Extensible**: Plugin-based system for adding new globals
4. **Type Safe**: Strong typing for script-to-native conversions
5. **Cross-Engine**: Consistent API across Lua and JavaScript

### Component Structure

```
llmspell-bridge/
├── src/
│   ├── conversion.rs           # Common conversion traits (ToScriptValue, FromScriptValue)
│   ├── globals/
│   │   ├── mod.rs              # Global injection framework
│   │   ├── registry.rs         # GlobalRegistry implementation
│   │   ├── injection.rs        # GlobalInjector trait and impls
│   │   ├── types.rs            # Common types and traits
│   │   └── core/               # Core globals
│   │       ├── mod.rs
│   │       ├── agent.rs        # Agent global
│   │       ├── tool.rs         # Tool/Tools globals
│   │       ├── workflow.rs     # Workflow global
│   │       ├── hook.rs         # Hook global
│   │       ├── event.rs        # Event global
│   │       ├── state.rs        # State global
│   │       ├── logger.rs       # Logger global
│   │       ├── config.rs       # Config global
│   │       ├── security.rs     # Security global
│   │       └── utils.rs        # Utils global
│   ├── lua/
│   │   ├── conversion.rs       # Consolidated Lua-specific conversions
│   │   ├── global_helpers.rs   # Lua-specific helpers for global injection
│   │   └── api/                # Existing API modules
│   └── javascript/
│       ├── conversion.rs       # Consolidated JS-specific conversions
│       ├── global_helpers.rs   # JS-specific helpers for global injection
│       └── api/                # Future API modules
```

This structure follows the existing pattern in the codebase where:
- Common/core functionality is at the bridge root
- Language-specific code stays in language directories
- Globals focus on injection coordination, not conversion ownership

### Key Components

#### 1. GlobalRegistry
Central registry for all injectable globals:
```rust
pub struct GlobalRegistry {
    globals: HashMap<String, Arc<dyn GlobalObject>>,
    injection_order: Vec<String>,
    performance_metrics: PerformanceMetrics,
}
```

#### 2. GlobalObject Trait
Common interface for all globals:
```rust
pub trait GlobalObject: Send + Sync {
    fn name(&self) -> &str;
    fn dependencies(&self) -> Vec<&str>;
    fn inject_lua(&self, lua: &Lua, context: &GlobalContext) -> Result<()>;
    fn inject_javascript(&self, ctx: &mut Context, context: &GlobalContext) -> Result<()>;
}
```

#### 3. GlobalContext
Shared context for all globals:
```rust
pub struct GlobalContext {
    registry: Arc<ComponentRegistry>,
    providers: Arc<ProviderManager>,
    bridge_refs: HashMap<String, Arc<dyn Any + Send + Sync>>,
}
```

#### 4. GlobalInjector
High-performance injection system:
```rust
pub struct GlobalInjector {
    registry: Arc<GlobalRegistry>,
    cache: InjectionCache,
    metrics: Arc<PerformanceMetrics>,
}
```

## Implementation Plan

### Phase 1: Core Infrastructure (Steps 1-3)
1. Create `globals/` module structure
2. Implement GlobalRegistry and GlobalObject trait
3. Create GlobalInjector with performance optimization

### Phase 2: Migrate Existing Globals (Steps 4-5)
4. Refactor existing APIs to use GlobalObject trait
5. Maintain backward compatibility during migration

### Phase 3: New Globals Implementation (Steps 6-11)
6. Implement Hook global
7. Implement Event global
8. Implement State global
9. Implement Logger global
10. Implement Config, Security, Utils globals
11. Enhance JSON global

### Phase 4: Integration & Optimization (Steps 12-14)
12. Integrate with script engines
13. Performance optimization and caching
14. Cross-engine consistency verification

## Performance Optimization Strategy

### 1. Batch Injection
- Pre-compile all globals into single injection operation
- Use lazy initialization for heavy objects
- Cache compiled globals per engine instance

### 2. Dependency Resolution
- Topological sort for injection order
- Parallel injection where possible
- Minimize cross-global dependencies

### 3. Memory Optimization
- Share immutable data between globals
- Use Arc for reference counting
- Implement cleanup on engine disposal

## Type Conversion System

### Unified Conversion Traits
```rust
pub trait ToScriptValue<T> {
    fn to_script_value(&self) -> Result<T>;
}

pub trait FromScriptValue<T>: Sized {
    fn from_script_value(value: T) -> Result<Self>;
}
```

### Automatic Conversions
- Primitive types (bool, numbers, strings)
- Collections (Vec, HashMap)
- JSON values
- Custom types with derive macros

## Migration Strategy

### Step 1: Parallel Implementation
- Build new system alongside existing
- No breaking changes initially

### Step 2: Gradual Migration
- Migrate one API at a time
- Maintain tests throughout

### Step 3: Deprecation
- Mark old APIs as deprecated
- Provide migration guide

### Step 4: Removal
- Remove old injection functions
- Clean up redundant code

## Testing Strategy

### Unit Tests
- Test each global in isolation
- Verify type conversions
- Check dependency resolution

### Integration Tests
- Test all globals together
- Verify cross-global interactions
- Check performance requirements

### Cross-Engine Tests
- Ensure Lua/JS consistency
- Verify same behavior across engines

## Example Usage

### Before (Current)
```rust
// In engine.rs
inject_agent_api(&lua, &api_surface.agent_api, registry, providers)?;
inject_tool_api(&lua, &api_surface.tool_api, registry)?;
inject_workflow_api(&lua, &api_surface.workflow_api, registry, workflow_bridge)?;
// ... more individual injections
```

### After (New System)
```rust
// In engine.rs
let global_injector = GlobalInjector::new(global_registry);
global_injector.inject_all(&mut engine, &global_context)?;
```

### Script Usage (Unchanged)
```lua
-- All globals available without require()
local agent = Agent.create({name = "assistant"})
local tool = Tool.get("calculator")
local workflow = Workflow.sequential({steps = {...}})
Hook.register("before_execute", function() ... end)
Event.emit("custom_event", {data = ...})
State.set("key", "value")
Logger.info("Message")
```

## Benefits

1. **Developer Experience**: Easier to add new globals
2. **Performance**: Batch injection reduces overhead
3. **Maintainability**: Centralized global management
4. **Extensibility**: Plugin system for custom globals
5. **Type Safety**: Strong typing throughout
6. **Consistency**: Same API across all engines