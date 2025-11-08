# Kernel Execution Paths

**Version**: 0.13.0
**Phase**: 13b.16
**Last Updated**: January 2025

> **🎯 Purpose**: Document the unified kernel creation path via Infrastructure module (Phase 13b.16)

**🔗 Navigation**: [← Technical Docs](README.md) | [Storage Architecture →](storage-architecture.md)

---

## Table of Contents

1. [Overview](#overview)
2. [Architecture Principles](#architecture-principles)
3. [Execution Modes](#execution-modes)
4. [Infrastructure Creation Flow](#infrastructure-creation-flow)
5. [Component Initialization Order](#component-initialization-order)
6. [Dependency Graph](#dependency-graph)
7. [Lifecycle Management](#lifecycle-management)
8. [Code References](#code-references)
9. [Migration Guide](#migration-guide)
10. [Troubleshooting](#troubleshooting)

---

## Overview

Phase 13b.16 introduced the **Infrastructure module** as the single creation path for all kernel infrastructure components. This unified approach replaced multiple fragmented initialization patterns with one config-driven factory.

### Single Creation Path

```text
┌─────────────────────────────────────────────┐
│  Infrastructure::from_config(config)        │
│                                             │
│  Creates ALL 9 components from config:      │
│  ├─ ProviderManager                         │
│  ├─ StateManager                            │
│  ├─ SessionManager                          │
│  ├─ RAG (optional, if config.rag.enabled)   │
│  ├─ MemoryManager (optional, if enabled)    │
│  ├─ ToolRegistry                            │
│  ├─ AgentRegistry                           │
│  ├─ WorkflowFactory                         │
│  └─ ComponentRegistry                       │
└─────────────────────────────────────────────┘
         ↓
┌─────────────────────────────────────────────┐
│  ScriptRuntime::new(config)                 │
│  ├─ Calls Infrastructure::from_config()     │
│  ├─ Stores all 9 components                 │
│  └─ Sets up script engine bridges           │
└─────────────────────────────────────────────┘
         ↓
┌─────────────────────────────────────────────┐
│  Kernel Creation                            │
│  ├─ CLI: ExecutionContext::resolve()        │
│  ├─ Programmatic: start_embedded_kernel()   │
│  └─ Both use same ScriptRuntime path        │
└─────────────────────────────────────────────┘
```

### Key Improvements

**Before Phase 13b.16** (Fragmented):
- CLI created components directly
- Services duplicated initialization code
- Multiple creation paths (embedded vs daemon)
- 200+ lines of initialization logic in CLI
- No conditional component creation

**After Phase 13b.16** (Unified):
- Single `Infrastructure::from_config()` entry point
- CLI uses ~12 lines: `ScriptRuntime::new()` + `start_embedded_kernel()`
- Conditional creation (RAG, Memory only if enabled)
- Config-driven architecture
- Zero duplication across modes

---

## Architecture Principles

### 1. Self-Contained Kernel (Phase 9/10)

The kernel must be **self-contained** - it owns all infrastructure creation:

```rust
// ✅ CORRECT (Phase 13b.16)
let script_executor = Arc::new(ScriptRuntime::new(config).await?);
let kernel = start_embedded_kernel_with_executor(config, script_executor).await?;

// ❌ INCORRECT (Pre-13b.16 pattern)
let provider_manager = ProviderManager::new(...).await?;
let state_manager = StateManager::new(...).await?;
// ... manual creation of each component
```

**Why**: Services, CLIs, and tests should all use the **same** creation path.

### 2. CLI Layer is Thin

CLI commands should **never** create infrastructure directly:

```rust
// llmspell-cli/src/execution_context.rs:136-146
let script_executor = Arc::new(
    llmspell_bridge::ScriptRuntime::new(config.clone()).await?
) as Arc<dyn ScriptExecutor>;

let handle = start_embedded_kernel_with_executor(
    config.clone(),
    script_executor,
).await?;
```

**Total CLI infrastructure code**: ~12 lines
**Infrastructure creation code**: 0 lines (delegated to `Infrastructure`)

### 3. Config-Driven Creation

Optional components (RAG, Memory) are created based on configuration:

```rust
// llmspell-bridge/src/infrastructure.rs:120-125
let rag = if config.rag.enabled {
    Some(create_rag(config))
} else {
    debug!("RAG disabled in config, skipping creation");
    None
};
```

**Benefits**:
- Production services enable PostgreSQL storage + RAG + Memory
- Development/testing disables expensive components
- Single config controls entire stack

### 4. Dependency Injection

All component dependencies are satisfied during creation:

```rust
// SessionManager depends on StateManager
let session_manager = create_session_manager(
    state_manager.clone(),  // ← Injected dependency
    config
)?;
```

**Enforced at compile time** - impossible to create components with missing dependencies.

---

## Execution Modes

### Embedded Mode (In-Process Kernel)

CLI or service runs kernel in the same process:

```rust
use llmspell_bridge::ScriptRuntime;
use llmspell_config::LLMSpellConfig;
use llmspell_kernel::api::start_embedded_kernel_with_executor;

let config = LLMSpellConfig::load_from_file("config.toml").await?;

// Phase 13b.16.3: ScriptRuntime creates ALL infrastructure
let script_executor = Arc::new(ScriptRuntime::new(config.clone()).await?)
    as Arc<dyn ScriptExecutor>;

let kernel_handle = start_embedded_kernel_with_executor(
    config,
    script_executor,
).await?;

// Execute scripts
let result = kernel_handle.execute("return 42").await?;
```

**Used by**:
- `llmspell run` command
- `llmspell exec` command
- `llmspell repl` command
- Embedded Rust applications
- HTTP services

### Connected Mode (Remote Kernel)

Client connects to external kernel server via TCP:

```rust
use llmspell_kernel::api::connect_to_kernel;

// Connect to running kernel
let client_handle = connect_to_kernel("localhost:9555").await?;

// Same API as embedded mode
let result = client_handle.execute("return 42").await?;
```

**Used by**:
- `llmspell run --connect localhost:9555`
- Multi-client scenarios
- Jupyter kernels
- VS Code integration

### Auto-Detection Mode

CLI automatically finds running kernel or falls back to embedded:

```rust
// llmspell-cli/src/execution_context.rs:155-187
match (connect, kernel, config) {
    // Explicit --connect flag: use remote kernel
    (Some(addr), _, _) => { /* connect to addr */ }

    // Explicit --kernel ID: find by ID
    (_, Some(kernel_id), _) => { /* find_kernel_by_id */ }

    // Explicit --config: use embedded with that config
    (_, _, Some(config_path)) => { /* embedded with config */ }

    // Auto-detect: search for running kernels
    (None, None, None) => {
        if let Some(addr) = find_running_kernel().await? {
            // Found running kernel, connect to it
        } else {
            // No kernel found, start embedded
        }
    }
}
```

**Priority order**:
1. `--connect <address>` (explicit remote)
2. `--kernel <id>` (find by ID)
3. `--config <path>` (embedded with config)
4. Auto-detect (search → fallback to embedded)

---

## Infrastructure Creation Flow

### Entry Point: `Infrastructure::from_config()`

**Location**: `llmspell-bridge/src/infrastructure.rs:107-161`

```rust
pub async fn from_config(config: &LLMSpellConfig) -> Result<Self, LLMSpellError> {
    info!("Creating infrastructure from config");

    // 1. Create provider manager
    let provider_manager = create_provider_manager(config).await?;

    // 2. Create state manager
    let state_manager = create_state_manager(config).await?;

    // 3. Create session manager (depends on state_manager)
    let session_manager = create_session_manager(state_manager.clone(), config)?;

    // 4. Create RAG if enabled
    let rag = if config.rag.enabled {
        Some(create_rag(config))
    } else {
        None
    };

    // 5. Create memory manager if enabled
    let memory_manager = if config.runtime.memory.enabled {
        Some(create_memory_manager(config).await?)
    } else {
        None
    };

    // 6. Create tool registry
    let tool_registry = Arc::new(llmspell_tools::ToolRegistry::new());

    // 7. Create agent registry
    let agent_registry = Arc::new(llmspell_agents::FactoryRegistry::new());

    // 8. Create workflow factory
    let workflow_factory: Arc<dyn llmspell_workflows::WorkflowFactory> =
        Arc::new(llmspell_workflows::factory::DefaultWorkflowFactory::new());

    // 9. Create component registry (with EventBus if enabled)
    let component_registry = create_component_registry(config)?;

    info!("Infrastructure created successfully");

    Ok(Self {
        provider_manager,
        state_manager,
        session_manager,
        rag,
        memory_manager,
        tool_registry,
        agent_registry,
        workflow_factory,
        component_registry,
    })
}
```

### Component Creation Functions

Each component has a dedicated creation function:

#### 1. ProviderManager

**Location**: `llmspell-bridge/src/infrastructure.rs:169-175`

```rust
async fn create_provider_manager(
    config: &LLMSpellConfig,
) -> Result<Arc<ProviderManager>, LLMSpellError> {
    debug!("Creating provider manager");
    let manager = ProviderManager::new(config.providers.clone()).await?;
    Ok(Arc::new(manager))
}
```

**Creates**: LLM provider connections (OpenAI, Anthropic, Ollama, Candle)

#### 2. StateManager

**Location**: `llmspell-bridge/src/infrastructure.rs:182-193`

```rust
async fn create_state_manager(
    _config: &LLMSpellConfig,
) -> Result<Arc<llmspell_kernel::state::StateManager>, LLMSpellError> {
    debug!("Creating state manager");
    let manager = llmspell_kernel::state::StateManager::new(None)
        .await
        .map_err(|e| LLMSpellError::Component {
            message: format!("Failed to create state manager: {e}"),
            source: None,
        })?;
    Ok(Arc::new(manager))
}
```

**Creates**: Persistent key-value state storage

#### 3. SessionManager

**Location**: `llmspell-bridge/src/infrastructure.rs:200-248`

```rust
fn create_session_manager(
    state_manager: Arc<llmspell_kernel::state::StateManager>,
    config: &LLMSpellConfig,
) -> Result<Arc<llmspell_kernel::sessions::SessionManager>, LLMSpellError> {
    // Select storage backend (memory vs sled)
    let session_storage_backend: Arc<dyn llmspell_storage::StorageBackend> =
        if config.runtime.sessions.storage_backend.as_str() == "memory" {
            Arc::new(llmspell_storage::MemoryBackend::new())
        } else {
            Arc::new(llmspell_storage::SledBackend::new_with_path("./sessions")?)
        };

    // Create hook infrastructure
    let hook_registry = Arc::new(llmspell_hooks::HookRegistry::new());
    let hook_executor = Arc::new(llmspell_hooks::HookExecutor::new());
    let event_bus = Arc::new(llmspell_events::bus::EventBus::new());

    // Create session manager with all dependencies
    let manager = llmspell_kernel::sessions::SessionManager::new(
        state_manager,
        session_storage_backend,
        hook_registry,
        hook_executor,
        &event_bus,
        session_config,
    )?;

    Ok(Arc::new(manager))
}
```

**Creates**: Session lifecycle manager with hooks and events

#### 4. RAG (Conditional)

**Location**: `llmspell-bridge/src/infrastructure.rs:254-302`

```rust
fn create_rag(
    config: &LLMSpellConfig,
) -> Arc<llmspell_rag::multi_tenant_integration::MultiTenantRAG> {
    // Convert config to storage config
    let storage_hnsw_config = llmspell_storage::HNSWConfig {
        m: config.rag.vector_storage.hnsw.m,
        ef_construction: config.rag.vector_storage.hnsw.ef_construction,
        // ... full HNSW config mapping
    };

    // Create vector storage
    let mut vector_storage = HNSWVectorStorage::new(
        config.rag.vector_storage.dimensions,
        storage_hnsw_config
    );

    // Enable persistence if configured
    if let Some(ref path) = config.rag.vector_storage.persistence_path {
        vector_storage = vector_storage.with_persistence(path.clone());
    }

    // Create tenant manager and RAG
    let tenant_manager = Arc::new(MultiTenantVectorManager::new(
        Arc::new(vector_storage)
    ));
    Arc::new(MultiTenantRAG::new(tenant_manager))
}
```

**Creates**: Multi-tenant RAG with HNSW vector storage
**Conditional**: Only if `config.rag.enabled == true`

#### 5. MemoryManager (Conditional)

**Location**: `llmspell-bridge/src/infrastructure.rs:311-328`

```rust
async fn create_memory_manager(
    _config: &LLMSpellConfig,
) -> Result<Arc<llmspell_memory::DefaultMemoryManager>, LLMSpellError> {
    // Use in-memory implementation (testing/development)
    let manager = llmspell_memory::DefaultMemoryManager::new_in_memory()
        .await
        .map_err(|e| LLMSpellError::Component {
            message: format!("Failed to create memory manager: {e}"),
            source: None,
        })?;
    Ok(Arc::new(manager))
}
```

**Creates**: 3-tier memory system (Episodic, Semantic, Procedural)
**Conditional**: Only if `config.runtime.memory.enabled == true`

#### 6-9. Registries

**Location**: `llmspell-bridge/src/infrastructure.rs:136-146`

```rust
// 6. Tool registry
let tool_registry = Arc::new(llmspell_tools::ToolRegistry::new());

// 7. Agent registry
let agent_registry = Arc::new(llmspell_agents::FactoryRegistry::new());

// 8. Workflow factory
let workflow_factory: Arc<dyn llmspell_workflows::WorkflowFactory> =
    Arc::new(llmspell_workflows::factory::DefaultWorkflowFactory::new());

// 9. Component registry (with EventBus if enabled)
let component_registry = create_component_registry(config)?;
```

**Creates**: Tool discovery, agent factories, workflow creation, script access layer

---

## Component Initialization Order

### Dependency-Aware Ordering

Components are created in dependency order to ensure all requirements are satisfied:

```text
Initialization Order (9 steps):

1. ProviderManager          [No dependencies]
   └─ OpenAI, Anthropic, Ollama, Candle providers

2. StateManager             [No dependencies]
   └─ Persistent key-value storage

3. SessionManager           [Depends on: StateManager]
   ├─ State manager (injected)
   ├─ Storage backend (memory or sled)
   ├─ Hook registry
   ├─ Hook executor
   └─ Event bus

4. RAG                      [No dependencies, optional]
   ├─ HNSW vector storage
   ├─ Tenant manager
   └─ Multi-tenant RAG

5. MemoryManager            [No dependencies, optional]
   ├─ Episodic memory (HNSW)
   ├─ Semantic memory (graph)
   └─ Procedural memory

6. ToolRegistry             [No dependencies]
   └─ Tool discovery and execution

7. AgentRegistry            [No dependencies]
   └─ Agent factory registration

8. WorkflowFactory          [No dependencies]
   └─ Workflow creation patterns

9. ComponentRegistry        [Depends on: EventBus if enabled]
   ├─ Script access layer
   ├─ Event bus (optional)
   └─ Template registry
```

### Dependency Graph

```text
┌──────────────────┐
│ ProviderManager  │ (Independent)
└──────────────────┘

┌──────────────────┐
│  StateManager    │ (Independent)
└────────┬─────────┘
         │
         ├──► ┌──────────────────┐
         │    │ SessionManager   │
         │    ├──────────────────┤
         │    │ + StateManager   │
         │    │ + StorageBackend │
         │    │ + HookRegistry   │
         │    │ + HookExecutor   │
         │    │ + EventBus       │
         └────┴──────────────────┘

┌──────────────────┐
│      RAG         │ (Optional, independent)
└──────────────────┘

┌──────────────────┐
│  MemoryManager   │ (Optional, independent)
└──────────────────┘

┌──────────────────┐
│  ToolRegistry    │ (Independent)
└──────────────────┘

┌──────────────────┐
│  AgentRegistry   │ (Independent)
└──────────────────┘

┌──────────────────┐
│ WorkflowFactory  │ (Independent)
└──────────────────┘

┌──────────────────┐
│ ComponentRegistry│ (Independent, EventBus optional)
└──────────────────┘
```

**Critical Path**: StateManager → SessionManager
**All other components**: Parallel creation possible (though currently sequential)

---

## Lifecycle Management

### Creation Phase

```rust
// 1. Load configuration
let config = LLMSpellConfig::load_from_file("config.toml").await?;

// 2. Create infrastructure (all 9 components)
let infrastructure = Infrastructure::from_config(&config).await?;

// 3. Create ScriptRuntime (wraps infrastructure)
let script_executor = Arc::new(ScriptRuntime::new(config.clone()).await?);

// 4. Start kernel
let kernel_handle = start_embedded_kernel_with_executor(
    config,
    script_executor,
).await?;
```

**Timeline**:
- Config load: <1ms (cached)
- Infrastructure creation: 50-200ms (depends on providers)
- ScriptRuntime creation: 10-50ms (engine initialization)
- Kernel start: <10ms (in-process transport)

**Total startup**: ~100-300ms (measured on M1 MacBook Pro)

### Execution Phase

```rust
// Execute script
let result = kernel_handle.execute(r#"
    local result = Agent.query("What is Rust?")
    return result
"#).await?;
```

**Component access**:
- Script → ComponentRegistry → Tool/Agent (O(1) HashMap lookup)
- Template → ToolRegistry → Tool (indexed lookup with hooks)

### Shutdown Phase

```rust
// Kernel shutdown
drop(kernel_handle);  // Graceful shutdown via Drop trait

// Infrastructure cleanup
drop(script_executor);
drop(infrastructure);
```

**Cleanup order**:
1. Kernel stops accepting requests
2. In-flight scripts complete (timeout: 30s)
3. Components shut down (Drop trait)
4. Storage backends flush to disk
5. Connections close

**Graceful shutdown**: <5s typical, <30s maximum

---

## Code References

### Primary Implementation Files

| Component | Location | Lines | Purpose |
|-----------|----------|-------|---------|
| Infrastructure | `llmspell-bridge/src/infrastructure.rs` | 385 | Central creation factory |
| ScriptRuntime | `llmspell-bridge/src/runtime.rs` | 800+ | Script execution orchestrator |
| ExecutionContext | `llmspell-cli/src/execution_context.rs` | 296 | CLI mode resolution |
| Kernel API | `llmspell-kernel/src/api.rs` | 1200+ | Embedded/client handles |

### Key Function References

```rust
// Infrastructure creation
llmspell-bridge/src/infrastructure.rs:107-161
pub async fn from_config(config: &LLMSpellConfig) -> Result<Self, LLMSpellError>

// Provider manager creation
llmspell-bridge/src/infrastructure.rs:169-175
async fn create_provider_manager(config: &LLMSpellConfig)

// State manager creation
llmspell-bridge/src/infrastructure.rs:182-193
async fn create_state_manager(_config: &LLMSpellConfig)

// Session manager creation
llmspell-bridge/src/infrastructure.rs:200-248
fn create_session_manager(state_manager, config)

// RAG creation (conditional)
llmspell-bridge/src/infrastructure.rs:254-302
fn create_rag(config: &LLMSpellConfig)

// Memory manager creation (conditional)
llmspell-bridge/src/infrastructure.rs:311-328
async fn create_memory_manager(_config: &LLMSpellConfig)

// Component registry creation
llmspell-bridge/src/infrastructure.rs:336-369
fn create_component_registry(config: &LLMSpellConfig)

// CLI execution context resolution
llmspell-cli/src/execution_context.rs:102-189
pub async fn resolve(connect, kernel, config, default_config)

// Embedded kernel creation
llmspell-kernel/src/api.rs:1093-1150
pub async fn start_embedded_kernel_with_executor(config, script_executor)

// Client kernel connection
llmspell-kernel/src/api.rs:720-780
pub async fn connect_to_kernel(address: &str)
```

---

## Migration Guide

### From Pre-13b.16 Pattern

**Before** (Manual component creation):

```rust
// ❌ OLD PATTERN - DO NOT USE
let provider_manager = ProviderManager::new(config.providers.clone()).await?;
let state_manager = StateManager::new(None).await?;
let session_manager = SessionManager::new(
    state_manager.clone(),
    storage_backend,
    hook_registry,
    hook_executor,
    &event_bus,
    session_config,
)?;

// ... manual creation of 6 more components
let script_runtime = ScriptRuntime::new_with_components(
    config,
    provider_manager,
    state_manager,
    session_manager,
    // ... pass all components
).await?;
```

**After** (Infrastructure pattern):

```rust
// ✅ NEW PATTERN (Phase 13b.16)
let script_executor = Arc::new(ScriptRuntime::new(config.clone()).await?)
    as Arc<dyn ScriptExecutor>;

let kernel_handle = start_embedded_kernel_with_executor(
    config,
    script_executor,
).await?;
```

**Lines of code**: 50+ → ~5 lines

### Component Access

**Before** (Direct field access):

```rust
// ❌ OLD PATTERN
let provider_manager = runtime.provider_manager.clone();
let state_manager = runtime.state_manager.clone();
```

**After** (Infrastructure accessor methods):

```rust
// ✅ NEW PATTERN
let infrastructure = Infrastructure::from_config(&config).await?;

let provider_manager = infrastructure.provider_manager();
let state_manager = infrastructure.state_manager();
let session_manager = infrastructure.session_manager();

// Optional components
if let Some(rag) = infrastructure.rag() {
    // RAG is enabled
}

if let Some(memory) = infrastructure.memory_manager() {
    // Memory is enabled
}
```

**Accessor methods** (llmspell-bridge/src/infrastructure.rs:250-310):
- `provider_manager()` → `Arc<ProviderManager>`
- `state_manager()` → `Arc<StateManager>`
- `session_manager()` → `Arc<SessionManager>`
- `rag()` → `Option<Arc<MultiTenantRAG>>`
- `memory_manager()` → `Option<Arc<DefaultMemoryManager>>`
- `tool_registry()` → `Arc<ToolRegistry>`
- `agent_registry()` → `Arc<FactoryRegistry>`
- `workflow_factory()` → `Arc<dyn WorkflowFactory>`
- `component_registry()` → `Arc<ComponentRegistry>`

---

## Troubleshooting

### Issue: Component Not Found

**Error**: `"Component 'rag' not found"`

**Cause**: RAG or Memory not enabled in config

**Solution**:
```toml
# config.toml
[rag]
enabled = true

[memory]
enable_memory = true
enable_rag = true
```

### Issue: Startup Slow

**Symptom**: Infrastructure creation takes >5s

**Diagnosis**:
```rust
// Enable debug logging
export RUST_LOG=llmspell_bridge=debug

// Check which component is slow
2025-01-15 14:30:00 DEBUG llmspell_bridge::infrastructure: Creating provider manager
2025-01-15 14:30:03 DEBUG llmspell_bridge::infrastructure: Creating state manager  ← 3s delay!
```

**Common causes**:
- Network timeout connecting to Ollama/Candle
- Disk I/O for large HNSW indexes
- PostgreSQL connection pool exhaustion

**Solutions**:
- Use `memory` storage backend for development
- Reduce HNSW `max_elements` for testing
- Check PostgreSQL connection limits

### Issue: Memory Leak

**Symptom**: Memory usage grows over time

**Diagnosis**:
- Check component `Arc` reference counts
- Use `cargo-flamegraph` for heap profiling

**Common causes**:
- Script engines holding onto old contexts
- Session artifacts not being garbage collected
- HNSW index growing without bounds

**Solutions**:
```toml
[runtime.sessions]
max_session_age_hours = 24  # Auto-cleanup old sessions

[rag.vector_storage.hnsw]
max_elements = 100000  # Limit HNSW growth
```

### Issue: Component Initialization Fails

**Error**: `"Failed to create session manager: ..."`

**Cause**: Dependency not satisfied (e.g., StateManager failed first)

**Debug steps**:
1. Check logs for first failure
2. Verify config file is valid TOML
3. Test component creation in isolation:

```rust
// Test StateManager creation
let state_manager = llmspell_kernel::state::StateManager::new(None).await?;
```

---

**🔗 See Also**:
- [Storage Architecture](storage-architecture.md) - Backend selection and optimization
- [Service Deployment](../user-guide/service-deployment.md) - Production deployment patterns
- [Configuration Guide](../user-guide/configuration.md) - Complete config reference
