# Bridge Pattern Guide

**Typed Rust Structs for Script-to-Rust Configuration Passing**

**Version**: 1.1 | **Phase**: 11a.8 + 13b.16 | **Status**: ✅ Complete
**Last Updated**: November 2025 (Phase 13b.16 Infrastructure Module)

**🔗 Navigation**: [← Developer Guide](README.md) | [Extending LLMSpell](extending-llmspell.md) | [Tracing Guide](tracing-best-practices.md)

---

## Table of Contents

1. [Overview & Purpose](#overview--purpose)
2. [Core Principles](#core-principles)
3. [Anti-Patterns Eliminated](#anti-patterns-eliminated)
4. [Pattern Components](#pattern-components)
5. [Infrastructure Module Pattern (Phase 13b.16)](#infrastructure-module-pattern-phase-13b16)
6. [Implementation Checklist](#implementation-checklist)
7. [Common Reusable Parsers](#common-reusable-parsers)
8. [Complete Examples](#complete-examples)
9. [Testing Requirements](#testing-requirements)
10. [Troubleshooting](#troubleshooting)
11. [Design Decisions Reference](#design-decisions-reference)

---

## Overview & Purpose

### The Problem

Prior to Phase 11a.8, llmspell-bridge methods accepted `serde_json::Value` or `HashMap<String, Value>` parameters for configuration, creating a multi-layer anti-pattern:

```rust
// ❌ ANTI-PATTERN (Pre-11a.8)
pub async fn create_agent(&self, config: serde_json::Value) -> Result<String> {
    let name = config.get("name")
        .and_then(|v| v.as_str())
        .ok_or(...)?;
    let model = config.get("model")
        .and_then(|v| v.as_str())
        .ok_or(...)?;
    // ... 50 more lines of JSON navigation
}
```

**Lua binding**:
```rust
// ❌ ANTI-PATTERN (Lua layer)
let config_json = lua_table_to_json(config_table)?;  // Lua table → JSON
bridge.create_agent(config_json).await?;             // JSON → HashMap in bridge
```

**Problems**:
- **No compile-time validation**: Typos in field names only caught at runtime
- **JSON serialization overhead**: Lua table → JSON → HashMap navigation
- **Poor error messages**: "missing field 'model'" doesn't indicate which layer failed
- **No IDE support**: No autocomplete for config fields
- **Maintenance burden**: Changing a field requires updating JSON keys as strings everywhere

### The Solution

**Bridge Pattern**: Use typed Rust structs for all configuration parameters, with parsing exclusively in the Lua layer.

```rust
// ✅ BRIDGE PATTERN (Phase 11a.8+)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentConfig {
    pub name: String,
    pub agent_type: String,
    pub model: Option<ModelConfig>,
    // ... typed fields
}

pub async fn create_agent(&self, config: AgentConfig) -> Result<String> {
    // Direct field access - no JSON navigation
    let agent = self.agent_discovery.create_agent(config).await?;
    Ok(agent.id())
}
```

**Lua binding**:
```rust
// ✅ Parser in Lua layer
fn parse_agent_config(table: &Table) -> mlua::Result<AgentConfig> {
    let name: String = table.get("name")?;
    let agent_type: String = table.get("agent_type")?;
    // ... typed extraction
    Ok(AgentConfig { name, agent_type, ... })
}

// Binding calls parser
let config = parse_agent_config(&config_table)?;
bridge.create_agent(config).await?;  // Zero serialization
```

**Benefits**:
- ✅ **Compile-time validation**: Rust compiler checks all field access
- ✅ **Zero serialization overhead**: Direct struct passing
- ✅ **Clear error messages**: mlua errors indicate exact Lua table field that's missing/wrong type
- ✅ **IDE autocomplete**: Full IntelliSense support for config construction
- ✅ **Refactoring safety**: Changing struct fields produces compile errors, not silent runtime failures
- ✅ **Self-documenting**: Struct fields show required vs optional parameters

---

## Core Principles

### 1. **Typed Structs in Bridge Layer**

All bridge methods accepting configuration MUST use typed structs, never `serde_json::Value` or `HashMap`:

```rust
// ✅ CORRECT
pub async fn create_context(&self, config: ExecutionContextConfig) -> Result<ContextId>

// ❌ WRONG
pub async fn create_context(&self, config: serde_json::Value) -> Result<ContextId>
pub async fn create_context(&self, config: HashMap<String, Value>) -> Result<ContextId>
```

### 2. **Parsing in Lua Layer Only**

All Lua table → Rust struct conversion happens in `llmspell-bridge/src/lua/globals/*.rs`, never in bridge methods:

```rust
// ✅ CORRECT (in lua/globals/agent.rs)
fn parse_agent_config(table: &Table) -> mlua::Result<AgentConfig> {
    // Parsing logic here
}

// Bridge binding
let config = parse_agent_config(&config_table)?;
bridge.create_agent(config).await?;

// ❌ WRONG (in agent_bridge.rs)
pub async fn create_agent(&self, config: Table) -> Result<String> {
    let name = config.get("name")?;  // Parsing in bridge - WRONG layer
    // ...
}
```

### 3. **Separation of Concerns**

- **Lua layer**: Responsible for format conversion (Lua tables/strings → typed Rust structs)
- **Bridge layer**: Responsible for business logic only (struct field access, method calls)

### 4. **Reuse Core Types When Available**

Before creating bridge-specific types, check if llmspell-core or llmspell-agents already defines the type:

```rust
// ✅ REUSE existing core types
use llmspell_core::execution_context::{ContextScope, InheritancePolicy};

pub struct ChildContextConfig {
    pub scope: ContextScope,           // ✅ Reused from core
    pub inheritance: InheritancePolicy, // ✅ Reused from core
}

// ⚠️ CREATE bridge-specific types when core types are too complex for Lua
// Example: llmspell-agents::AlertCondition has Arc<dyn AlertEvaluator>
// which cannot be constructed from Lua, so we create BridgeAlertConfig
pub enum AlertConditionConfig {
    MetricThreshold { ... },
    HealthStatus { ... },
    ErrorRate { ... },
}
```

### 5. **Serde Attributes for Clean JSON**

Use Serde attributes to ensure clean serialization and good defaults:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]  // ✅ "MetricThreshold" → "metric_threshold"
pub enum AlertConditionConfig {
    MetricThreshold { ... },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolWrapperConfig {
    pub tool_name: String,

    #[serde(skip_serializing_if = "Option::is_none")]  // ✅ Omit None fields
    pub category: Option<ToolCategory>,

    #[serde(default = "default_enabled")]  // ✅ Use function for defaults
    pub enabled: bool,
}

const fn default_enabled() -> bool { true }  // ✅ const fn per clippy
```

### 6. **Optional Fields with Sensible Defaults**

Use `Option<T>` for optional fields and provide defaults in bridge implementation:

```rust
pub struct ToolWrapperConfig {
    pub tool_name: String,
    pub category: Option<ToolCategory>,      // ✅ Optional with default
    pub security_level: Option<SecurityLevel>, // ✅ Optional with default
}

// In bridge method
let category = config.category.unwrap_or(ToolCategory::Utility);
let security_level = config.security_level.unwrap_or(SecurityLevel::Restricted);
```

---

## Anti-Patterns Eliminated

### ❌ Anti-Pattern 1: JSON in Bridge Signatures

**Before**:
```rust
pub async fn wrap_agent_as_tool(
    &self,
    agent_name: &str,
    wrapper_config: serde_json::Value,  // ❌ Opaque JSON
) -> Result<String>
```

**After**:
```rust
pub async fn wrap_agent_as_tool(
    &self,
    agent_name: &str,
    config: ToolWrapperConfig,  // ✅ Typed struct
) -> Result<String>
```

### ❌ Anti-Pattern 2: lua_table_to_json Conversion

**Before**:
```rust
// In Lua binding
let config_json = lua_table_to_json(config_table)?;  // ❌ Lua → JSON
bridge.wrap_agent_as_tool(&agent_name, config_json).await?;
```

**After**:
```rust
// In Lua binding
let config = parse_tool_wrapper_config(&config_table);  // ✅ Lua → struct
bridge.wrap_agent_as_tool(&agent_name, config).await?;
```

### ❌ Anti-Pattern 3: JSON Navigation in Bridge

**Before**:
```rust
pub async fn create_context(&self, config: serde_json::Value) -> Result<ContextId> {
    let conversation_id = config
        .get("conversation_id")  // ❌ String keys
        .and_then(|v| v.as_str())
        .map(String::from);
    let user_id = config
        .get("user_id")
        .and_then(|v| v.as_str())
        .map(String::from);
    // ... 40 more lines
}
```

**After**:
```rust
pub async fn create_context(&self, config: ExecutionContextConfig) -> Result<ContextId> {
    let mut builder = ExecutionContextBuilder::new();
    if let Some(conv_id) = config.conversation_id {  // ✅ Direct field access
        builder = builder.with_conversation_id(conv_id);
    }
    if let Some(user_id) = config.user_id {
        builder = builder.with_user_id(user_id);
    }
    // ... clean builder pattern
}
```

### ❌ Anti-Pattern 4: Ignoring JSON Parameters

**Before**:
```rust
pub async fn replay_session(
    &self,
    session_id: &SessionId,
    _options: serde_json::Value,  // ❌ Ignored! Using default config instead
) -> Result<serde_json::Value>
```

**After**:
```rust
pub async fn replay_session(
    &self,
    session_id: &SessionId,
    config: SessionReplayConfig,  // ✅ Actually used
) -> Result<serde_json::Value>
```

---

## Pattern Components

### Component 1: Typed Struct Definition (Bridge Layer)

**Location**: `llmspell-bridge/src/agent_bridge.rs` (or relevant bridge file)

**Guidelines**:
- Place near top of file (after imports, before impl blocks)
- Group related structs together (e.g., ModelConfig + AgentConfig)
- Use clear, descriptive names ending in `Config` for configuration structs
- Document all fields with `///` comments

**Example**:
```rust
/// Configuration for wrapping an agent as a tool
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolWrapperConfig {
    /// Name for the wrapped tool
    pub tool_name: String,

    /// Tool category (defaults to Utility)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub category: Option<llmspell_core::traits::tool::ToolCategory>,

    /// Security level (defaults to Restricted)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub security_level: Option<llmspell_core::traits::tool::SecurityLevel>,
}
```

### Component 2: Parser Function (Lua Layer)

**Location**: `llmspell-bridge/src/lua/globals/*.rs` (e.g., `agent.rs` for agent-related parsers)

**Guidelines**:
- Function name: `parse_<struct_name_snake_case>`
- Place parsers near top of file, before bindings that use them
- Document expected Lua table structure
- Return `mlua::Result<T>` for failable parsing, or just `T` if defaults cover all cases
- Provide helpful error messages

**Example**:
```rust
/// Parse `ToolWrapperConfig` from a Lua table
///
/// Expected fields:
/// - `tool_name`: string (required)
/// - category: string (optional) - "filesystem", "web", "api", etc.
/// - `security_level`: string (optional) - "safe", "restricted", "privileged"
///
/// Supports both minimal and full configuration:
/// - Minimal: { `tool_name` = `"my_tool"` }
/// - Full: { `tool_name` = `"my_tool"`, category = "api", `security_level` = "restricted" }
fn parse_tool_wrapper_config(table: &Table) -> crate::agent_bridge::ToolWrapperConfig {
    use crate::agent_bridge::ToolWrapperConfig;
    use llmspell_core::traits::tool::{SecurityLevel, ToolCategory};

    let tool_name: String = table.get("tool_name").unwrap_or_else(|_| String::new());

    let category = table
        .get::<_, Option<String>>("category")
        .unwrap_or(None)
        .map(|cat_str| match cat_str.as_str() {
            "filesystem" => ToolCategory::Filesystem,
            "web" => ToolCategory::Web,
            "api" => ToolCategory::Api,
            "analysis" => ToolCategory::Analysis,
            "data" => ToolCategory::Data,
            "system" => ToolCategory::System,
            "media" => ToolCategory::Media,
            "utility" => ToolCategory::Utility,
            custom => ToolCategory::Custom(custom.to_string()),
        });

    let security_level = table
        .get::<_, Option<String>>("security_level")
        .unwrap_or(None)
        .and_then(|level_str| match level_str.as_str() {
            "safe" => Some(SecurityLevel::Safe),
            "restricted" => Some(SecurityLevel::Restricted),
            "privileged" => Some(SecurityLevel::Privileged),
            _ => None,
        });

    ToolWrapperConfig {
        tool_name,
        category,
        security_level,
    }
}
```

### Component 3: Bridge Method Signature Update

**Location**: `llmspell-bridge/src/agent_bridge.rs` (or relevant bridge file)

**Guidelines**:
- Replace `serde_json::Value` or `HashMap` parameter with typed struct
- Update method body to use direct field access
- Remove JSON navigation logic
- Simplify error handling (no parse errors needed)

**Example**:
```rust
/// Wrap an agent as a tool
///
/// # Errors
///
/// Returns an error if the agent is not found or tool registration fails
pub async fn wrap_agent_as_tool(
    &self,
    agent_name: &str,
    config: ToolWrapperConfig,  // ✅ Typed struct
) -> Result<String> {
    use llmspell_agents::agent_wrapped_tool::AgentWrappedTool;
    use llmspell_core::traits::tool::{SecurityLevel, ToolCategory};

    // Get the agent instance
    let agent = self
        .get_agent(agent_name)
        .await
        .ok_or_else(|| LLMSpellError::Component {
            message: format!("Agent '{agent_name}' not found"),
            source: None,
        })?;

    // ✅ Direct field access with defaults
    let tool_name = if config.tool_name.is_empty() {
        format!("{agent_name}_tool")
    } else {
        config.tool_name
    };
    let category = config.category.unwrap_or(ToolCategory::Utility);
    let security_level = config.security_level.unwrap_or(SecurityLevel::Restricted);

    // Create and register the wrapped tool
    let wrapped_tool = AgentWrappedTool::new(agent.clone(), category, security_level);
    self.registry.register_tool(tool_name.clone(), Arc::new(wrapped_tool))?;

    Ok(tool_name)
}
```

### Component 4: Lua Binding Update

**Location**: `llmspell-bridge/src/lua/globals/*.rs`

**Guidelines**:
- Find the Lua binding that calls the bridge method
- Replace `lua_table_to_json()` call with parser call
- Update signature if needed (e.g., `Table` → `Value` for flexible parsing)
- Remove `.unwrap()` if method no longer returns `Result`

**Example**:
```rust
// Create Agent.wrap_as_tool() function
let bridge_clone = bridge.clone();
let wrap_as_tool_fn = lua.create_function(move |_lua, args: (String, Table)| {
    let (agent_name, config) = args;
    let bridge = bridge_clone.clone();

    // ✅ Parse Lua table to typed config
    let config_typed = parse_tool_wrapper_config(&config);

    // ✅ Call bridge with typed struct
    let tool_name = block_on_async(
        "agent_wrapAsTool",
        bridge.wrap_agent_as_tool(&agent_name, config_typed),
        None,
    )
    .map_err(|e| mlua::Error::RuntimeError(format!("Failed to wrap agent as tool: {e}")))?;

    Ok(tool_name)
})?;
```

---

## Infrastructure Module Pattern (Phase 13b.16)

**Purpose**: Config-driven component creation pattern introduced in Phase 13b.16 to unify kernel initialization, simplify CLI, and provide a single creation path for all infrastructure components.

### The Evolution: From Manual Construction to Config-Driven Creation

**Before Phase 13b.16** (Manual component creation in CLI):
```rust
// ❌ ANTI-PATTERN: 200+ LOC manual initialization
pub async fn run_command(config: LLMSpellConfig) -> Result<()> {
    // Manual provider manager creation
    let provider_manager = Arc::new(ProviderManager::new(&config.providers));

    // Manual state manager creation
    let state_backend = match config.storage.backend {
        "memory" => Arc::new(MemoryBackend::new()) as Arc<dyn StorageBackend>,
        "sqlite" => Arc::new(SqliteBackend::new(&config.storage.sqlite.path).await?),
        "postgres" => Arc::new(PostgreSQLBackend::new(&config.storage.postgres.url).await?),
        _ => return Err(...)
    };
    let state_manager = Arc::new(StateManager::new(state_backend));

    // Manual session manager creation (with dependency)
    let session_manager = Arc::new(SessionManager::new(
        state_manager.clone(),
        config.sessions.clone()
    ));

    // Manual RAG creation (with multiple dependencies)
    let rag = Arc::new(RAG::new(
        provider_manager.clone(),
        state_manager.clone(),
        config.rag.clone()
    )?);

    // ... 150+ more lines for remaining 5 components

    // Manual ScriptRuntime creation with all dependencies
    let script_runtime = ScriptRuntime::new_with_lua(config.clone())
        .await?
        .with_provider_manager(provider_manager.clone())
        .with_state_manager(state_manager.clone())
        .with_session_manager(session_manager.clone())
        .with_rag(rag.clone())
        .with_memory_manager(memory_manager.clone())
        .with_tool_registry(tool_registry.clone())
        .with_agent_registry(agent_registry.clone())
        .with_workflow_factory(workflow_factory.clone())
        .with_component_registry(component_registry.clone());

    Ok(())
}
```

**After Phase 13b.16** (Config-driven Infrastructure module):
```rust
// ✅ INFRASTRUCTURE PATTERN: 12 LOC single creation path
pub async fn run_command(config: LLMSpellConfig) -> Result<()> {
    // Single unified creation call
    let infrastructure = Infrastructure::from_config(&config).await?;

    // ScriptRuntime automatically receives all components
    let script_runtime = ScriptRuntime::new(config.clone())
        .with_infrastructure(infrastructure)
        .with_engine("lua")
        .await?;

    Ok(())
}
```

**Impact**: 200+ LOC → 12 LOC (94% reduction), zero component creation boilerplate

### Infrastructure Module Core API

**Location**: `llmspell-bridge/src/infrastructure.rs`

**Purpose**: Unified factory for creating all 9 infrastructure components from `LLMSpellConfig` with correct dependency order and configuration.

```rust
use llmspell_bridge::Infrastructure;
use llmspell_core::config::LLMSpellConfig;

/// Infrastructure module encapsulating all core components
pub struct Infrastructure {
    provider_manager: Arc<ProviderManager>,
    state_manager: Arc<StateManager>,
    session_manager: Arc<SessionManager>,
    rag: Arc<RAG>,
    memory_manager: Arc<MemoryManager>,
    tool_registry: Arc<ToolRegistry>,
    agent_registry: Arc<AgentDiscovery>,
    workflow_factory: Arc<WorkflowFactory>,
    component_registry: Arc<ComponentRegistry>,
}

impl Infrastructure {
    /// Create all components from config with correct dependency order
    pub async fn from_config(config: &LLMSpellConfig) -> Result<Self, LLMSpellError> {
        // 1. Create provider manager (no dependencies)
        let provider_manager = create_provider_manager(config).await?;

        // 2. Create state manager (no dependencies)
        let state_manager = create_state_manager(config).await?;

        // 3. Create session manager (depends on state_manager)
        let session_manager = create_session_manager(config, state_manager.clone()).await?;

        // 4. Create RAG (depends on provider_manager, state_manager)
        let rag = create_rag(config, provider_manager.clone(), state_manager.clone()).await?;

        // 5. Create memory manager (depends on state_manager, provider_manager)
        let memory_manager = create_memory_manager(config, state_manager.clone(), provider_manager.clone()).await?;

        // 6. Create tool registry (no dependencies)
        let tool_registry = create_tool_registry(config).await?;

        // 7. Create agent registry (depends on provider_manager, tool_registry)
        let agent_registry = create_agent_registry(config, provider_manager.clone(), tool_registry.clone()).await?;

        // 8. Create workflow factory (depends on agent_registry)
        let workflow_factory = create_workflow_factory(config, agent_registry.clone()).await?;

        // 9. Create component registry (aggregates all components)
        let component_registry = create_component_registry(
            config,
            provider_manager.clone(),
            state_manager.clone(),
            session_manager.clone(),
            rag.clone(),
            memory_manager.clone(),
            tool_registry.clone(),
            agent_registry.clone(),
            workflow_factory.clone(),
        ).await?;

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

    /// Get provider manager
    pub fn provider_manager(&self) -> Arc<ProviderManager> {
        self.provider_manager.clone()
    }

    /// Get state manager
    pub fn state_manager(&self) -> Arc<StateManager> {
        self.state_manager.clone()
    }

    // ... accessors for all 9 components
}
```

### Component Factory Functions

Each component has a dedicated factory function that encapsulates creation logic:

#### 1. ProviderManager Factory

**Location**: `llmspell-bridge/src/infrastructure.rs:45-72`

```rust
async fn create_provider_manager(config: &LLMSpellConfig) -> Result<Arc<ProviderManager>, LLMSpellError> {
    use llmspell_providers::{ProviderConfig, ProviderManager, ProviderType};

    let mut provider_configs = Vec::new();

    // Ollama providers
    for (name, ollama_config) in &config.providers.ollama {
        provider_configs.push(ProviderConfig {
            name: name.clone(),
            provider_type: ProviderType::Ollama,
            config_json: serde_json::to_value(ollama_config)?,
        });
    }

    // Candle providers
    for (name, candle_config) in &config.providers.candle {
        provider_configs.push(ProviderConfig {
            name: name.clone(),
            provider_type: ProviderType::Candle,
            config_json: serde_json::to_value(candle_config)?,
        });
    }

    let manager = ProviderManager::from_configs(&provider_configs).await?;
    Ok(Arc::new(manager))
}
```

#### 2. StateManager Factory (Hot-Swappable Backend)

**Location**: `llmspell-bridge/src/infrastructure.rs:74-112`

```rust
async fn create_state_manager(config: &LLMSpellConfig) -> Result<Arc<StateManager>, LLMSpellError> {
    use llmspell_storage::{MemoryBackend, SqliteBackend, PostgreSQLBackend, StorageBackend};

    // Backend selection from config
    let backend: Arc<dyn StorageBackend> = match config.storage.backend.as_str() {
        "memory" => {
            info!("Using in-memory storage backend");
            Arc::new(MemoryBackend::new())
        }
        "sqlite" => {
            info!("Using SQLite embedded database backend");
            let sqlite_path = &config.storage.sqlite.path;
            Arc::new(SqliteBackend::new_with_path(sqlite_path).await?)
        }
        "postgres" => {
            info!("Using PostgreSQL backend");
            let pg_backend = PostgreSQLBackend::new(&config.storage.postgres.url).await?;

            // Run migrations if enabled
            if config.storage.postgres.run_migrations {
                pg_backend.migrate().await?;
            }

            // Enable RLS if configured
            if config.storage.postgres.enforce_tenant_isolation {
                Arc::new(pg_backend.with_tenant(config.storage.postgres.default_tenant.clone()))
            } else {
                Arc::new(pg_backend)
            }
        }
        backend_type => {
            return Err(LLMSpellError::Config(format!(
                "Unknown storage backend: {}. Supported: memory, sqlite, postgres",
                backend_type
            )));
        }
    };

    Ok(Arc::new(StateManager::new(backend)))
}
```

#### 3. RAG Factory (Multi-Dependency Initialization)

**Location**: `llmspell-bridge/src/infrastructure.rs:142-178`

```rust
async fn create_rag(
    config: &LLMSpellConfig,
    provider_manager: Arc<ProviderManager>,
    state_manager: Arc<StateManager>,
) -> Result<Arc<RAG>, LLMSpellError> {
    use llmspell_rag::{RAG, RAGConfig, ChunkingStrategy};

    // Extract RAG config or use defaults
    let rag_config = RAGConfig {
        default_provider: config.rag.default_provider.clone(),
        default_chunking: ChunkingStrategy::Sliding {
            size: config.rag.chunk_size.unwrap_or(500),
            overlap: config.rag.chunk_overlap.unwrap_or(50),
        },
        default_k: config.rag.default_k.unwrap_or(5),
        default_threshold: config.rag.similarity_threshold,
    };

    let rag = RAG::new(provider_manager, state_manager, rag_config)?;
    Ok(Arc::new(rag))
}
```

### Config-Driven Component Selection

**Example**: Per-component backend override in `config.toml`:

```toml
[storage]
# Global default backend
backend = "postgres"

[storage.postgres]
url = "postgresql://llmspell:pass@localhost:5432/llmspell_prod"
pool_size = 20
enforce_tenant_isolation = true
run_migrations = true

# Component-specific backend overrides
[storage.components.vector_embeddings]
backend = "postgres"  # Use PostgreSQL HNSW for vectors

[storage.components.episodic_memory]
backend = "postgres"  # Use PostgreSQL for episodic memory

[storage.components.agent_state]
backend = "sqlite"    # Use SQLite for fast local agent state

[storage.components.session_data]
backend = "memory"    # Use in-memory for ephemeral sessions
```

**Implementation**:
```rust
// Infrastructure module automatically applies component overrides
let state_manager = create_state_manager(config).await?;

// Each component queries its specific backend from config
let vector_backend = state_manager.backend_for_component("vector_embeddings");
let agent_backend = state_manager.backend_for_component("agent_state");
```

### Dependency Order Guarantees

The `Infrastructure::from_config()` method enforces correct dependency order:

```
Dependency Graph:
=================

Level 1 (No dependencies):
- ProviderManager
- StateManager
- ToolRegistry

Level 2 (1 dependency):
- SessionManager (→ StateManager)

Level 3 (2 dependencies):
- RAG (→ ProviderManager, StateManager)
- MemoryManager (→ StateManager, ProviderManager)
- AgentRegistry (→ ProviderManager, ToolRegistry)

Level 4 (3 dependencies):
- WorkflowFactory (→ AgentRegistry)

Level 5 (All dependencies):
- ComponentRegistry (aggregates all 8 components)
```

**Validation**: If dependency order is violated, compilation fails with clear error:

```rust
// ❌ COMPILE ERROR: Cannot create RAG before its dependencies
let rag = create_rag(config, provider_manager, state_manager).await?;
let provider_manager = create_provider_manager(config).await?;  // ERROR: Used before definition
```

### ScriptRuntime Integration

**API Location**: `llmspell-bridge/src/script_runtime.rs`

**Pattern**: ScriptRuntime delegates component creation to Infrastructure module:

```rust
use llmspell_bridge::{ScriptRuntime, Infrastructure};

impl ScriptRuntime {
    /// Create ScriptRuntime with Infrastructure module
    pub async fn new(config: LLMSpellConfig) -> Result<Self> {
        // Infrastructure handles all component creation
        let infrastructure = Infrastructure::from_config(&config).await?;

        let mut runtime = Self {
            config,
            engine: None,
            provider_manager: infrastructure.provider_manager(),
            state_manager: infrastructure.state_manager(),
            session_manager: infrastructure.session_manager(),
            rag: infrastructure.rag(),
            memory_manager: infrastructure.memory_manager(),
            tool_registry: infrastructure.tool_registry(),
            agent_registry: infrastructure.agent_registry(),
            workflow_factory: infrastructure.workflow_factory(),
            component_registry: infrastructure.component_registry(),
        };

        Ok(runtime)
    }

    /// Add script engine (Lua/JavaScript/Python)
    pub async fn with_engine(mut self, engine_type: &str) -> Result<Self> {
        match engine_type {
            "lua" => {
                let lua_engine = LuaEngine::new(self.clone()).await?;
                self.engine = Some(Box::new(lua_engine));
            }
            "javascript" => {
                let js_engine = JavaScriptEngine::new(self.clone()).await?;
                self.engine = Some(Box::new(js_engine));
            }
            _ => return Err(LLMSpellError::Config(format!("Unknown engine: {}", engine_type))),
        }
        Ok(self)
    }

    /// Legacy constructor (deprecated in Phase 13b.16)
    #[deprecated(since = "0.14.0", note = "Use `ScriptRuntime::new()` instead")]
    pub async fn new_with_lua(config: LLMSpellConfig) -> Result<Self> {
        Self::new(config).await?.with_engine("lua").await
    }
}
```

### Usage Patterns

#### Pattern 1: Embedded Service Deployment

```rust
use llmspell_bridge::{ScriptRuntime, Infrastructure};
use llmspell_core::config::LLMSpellConfig;

#[tokio::main]
async fn main() -> Result<()> {
    // Load config
    let config = LLMSpellConfig::from_file("config.toml")?;

    // Create runtime with all infrastructure
    let runtime = ScriptRuntime::new(config.clone())
        .await?
        .with_engine("lua")
        .await?;

    // Execute script
    runtime.execute_file("script.lua").await?;

    Ok(())
}
```

#### Pattern 2: HTTP Service with Axum

```rust
use axum::{Router, routing::post};
use llmspell_bridge::ScriptRuntime;

#[tokio::main]
async fn main() -> Result<()> {
    let config = LLMSpellConfig::from_file("config.toml")?;

    // Create runtime once, share across requests
    let runtime = Arc::new(
        ScriptRuntime::new(config)
            .await?
            .with_engine("lua")
            .await?
    );

    let app = Router::new()
        .route("/execute", post(execute_handler))
        .layer(Extension(runtime));

    axum::Server::bind(&"0.0.0.0:3000".parse()?)
        .serve(app.into_make_service())
        .await?;

    Ok(())
}
```

#### Pattern 3: CLI Kernel Mode

```rust
use llmspell_bridge::ScriptRuntime;
use llmspell_kernel::{start_embedded_kernel_with_executor, KernelHandle};

pub async fn run_command(config: LLMSpellConfig) -> Result<()> {
    // Create ScriptRuntime as ScriptExecutor
    let script_executor = Arc::new(
        ScriptRuntime::new(config.clone())
            .await?
            .with_engine("lua")
            .await?
    ) as Arc<dyn ScriptExecutor>;

    // Start kernel with executor
    let kernel_handle = start_embedded_kernel_with_executor(
        config,
        script_executor
    ).await?;

    // Kernel lifecycle management
    kernel_handle.wait_for_shutdown().await?;

    Ok(())
}
```

### Component Lifecycle Management

**Initialization Order**:
```
1. Infrastructure::from_config()
   ├─ 1. ProviderManager (parallel init)
   ├─ 2. StateManager (parallel init)
   ├─ 3. ToolRegistry (parallel init)
   ├─ 4. SessionManager (depends on StateManager)
   ├─ 5. RAG (depends on ProviderManager, StateManager)
   ├─ 6. MemoryManager (depends on StateManager, ProviderManager)
   ├─ 7. AgentRegistry (depends on ProviderManager, ToolRegistry)
   ├─ 8. WorkflowFactory (depends on AgentRegistry)
   └─ 9. ComponentRegistry (aggregates all)

2. ScriptRuntime::new()
   └─ Receives Infrastructure components

3. ScriptRuntime::with_engine()
   └─ Attaches script engine (Lua/JS)
```

**Shutdown Order** (reverse of initialization):
```
1. ScriptRuntime drops
2. ComponentRegistry drops (releases all component refs)
3. WorkflowFactory drops
4. AgentRegistry drops
5. MemoryManager drops
6. RAG drops
7. SessionManager drops
8. ToolRegistry drops
9. StateManager drops (flushes to disk if SQLite/Postgres)
10. ProviderManager drops (closes connections)
```

### Extension Points

#### Adding New Components

To add a new component to Infrastructure:

**Step 1**: Define factory function
```rust
// llmspell-bridge/src/infrastructure.rs
async fn create_new_component(
    config: &LLMSpellConfig,
    dependency1: Arc<Dependency1>,
    dependency2: Arc<Dependency2>,
) -> Result<Arc<NewComponent>, LLMSpellError> {
    let component_config = NewComponentConfig::from_config(config)?;
    let component = NewComponent::new(dependency1, dependency2, component_config)?;
    Ok(Arc::new(component))
}
```

**Step 2**: Add to Infrastructure struct
```rust
pub struct Infrastructure {
    // ... existing components
    new_component: Arc<NewComponent>,
}
```

**Step 3**: Initialize in `from_config()`
```rust
impl Infrastructure {
    pub async fn from_config(config: &LLMSpellConfig) -> Result<Self> {
        // ... existing initialization

        // 10. Create new component
        let new_component = create_new_component(
            config,
            dependency1.clone(),
            dependency2.clone()
        ).await?;

        Ok(Self {
            // ... existing fields
            new_component,
        })
    }

    pub fn new_component(&self) -> Arc<NewComponent> {
        self.new_component.clone()
    }
}
```

**Step 4**: Add to ScriptRuntime
```rust
pub struct ScriptRuntime {
    // ... existing fields
    new_component: Arc<NewComponent>,
}

impl ScriptRuntime {
    pub async fn new(config: LLMSpellConfig) -> Result<Self> {
        let infrastructure = Infrastructure::from_config(&config).await?;

        Ok(Self {
            // ... existing fields
            new_component: infrastructure.new_component(),
        })
    }
}
```

### Benefits of Infrastructure Pattern

1. **Single Source of Truth**: Config file drives all component creation
2. **Dependency Safety**: Correct initialization order enforced at compile time
3. **Zero Boilerplate**: CLI goes from 200+ LOC to 12 LOC
4. **Hot-Swappable Backends**: Change storage backend without code changes
5. **Component-Specific Config**: Per-component backend overrides
6. **Testability**: Easy to mock Infrastructure for unit tests
7. **Extensibility**: Add new components without touching CLI
8. **Documentation**: Single place to document component creation

### Related Documentation

- **Kernel Execution Paths**: `/docs/technical/kernel-execution-paths.md` (detailed initialization flow)
- **Storage Architecture**: `/docs/technical/storage-architecture.md` (backend selection patterns)
- **Configuration Guide**: `/docs/user-guide/configuration.md` (config schema)
- **Service Deployment**: `/docs/user-guide/service-deployment.md` (deployment patterns)

---

## Implementation Checklist

Use this checklist when implementing the bridge pattern for a new method:

### Phase 1: Analysis & Design

- [ ] **Identify the method** - Find bridge method with `serde_json::Value` or `HashMap` parameter
- [ ] **Check for existing types** - Search llmspell-core and llmspell-agents for reusable types
- [ ] **Document current JSON structure** - Note what fields are expected/used
- [ ] **Design typed struct** - Determine required vs optional fields, nested structs
- [ ] **Check name conflicts** - Ensure struct name doesn't conflict with imports

### Phase 2: Struct Implementation

- [ ] **Create typed struct(s)** in bridge file with:
  - [ ] Clear doc comments
  - [ ] `#[derive(Debug, Clone, Serialize, Deserialize)]`
  - [ ] Appropriate Serde attributes (`skip_serializing_if`, `default`, `rename_all`)
  - [ ] Default implementation if all fields have defaults
- [ ] **Run `cargo check`** - Ensure struct compiles

### Phase 3: Parser Implementation

- [ ] **Create parser function** in appropriate `lua/globals/*.rs` file:
  - [ ] Name: `parse_<struct_name_snake_case>`
  - [ ] Doc comment with expected Lua table structure
  - [ ] Return type: `mlua::Result<T>` or `T` if infallible
  - [ ] Support flexible input (e.g., String or Table) where appropriate
  - [ ] Clear error messages for missing/invalid fields
- [ ] **Run `cargo clippy`** - Check for warnings
- [ ] **Fix any clippy warnings** - Especially `unnecessary_wraps`, `bind_instead_of_map`

### Phase 4: Bridge Method Update

- [ ] **Update method signature** - Replace JSON param with typed struct
- [ ] **Simplify implementation** - Replace JSON navigation with direct field access
- [ ] **Update error handling** - Remove parse-related errors if no longer applicable
- [ ] **Update doc comments** - Reflect new signature
- [ ] **Run `cargo check`** - Ensure bridge compiles

### Phase 5: Lua Binding Update

- [ ] **Find Lua binding** that calls the bridge method
- [ ] **Replace `lua_table_to_json()`** with parser call
- [ ] **Update signature** if needed (e.g., `Table` → `Value`)
- [ ] **Remove `.unwrap()`** if method no longer returns `Result`
- [ ] **Remove unused imports** (e.g., `lua_table_to_json`)
- [ ] **Run `cargo clippy`** - Check for warnings

### Phase 6: Test Updates

- [ ] **Update test fixtures** to use typed structs instead of JSON
- [ ] **Run `cargo test`** - Ensure all tests pass
- [ ] **Check for dead code** - Remove any leftover JSON construction
- [ ] **Add new tests** if coverage gaps exist

### Phase 7: Validation

- [ ] **`cargo clippy -p llmspell-bridge --all-targets --all-features -- -D warnings`** - 0 warnings
- [ ] **`cargo test -p llmspell-bridge --all-features`** - All tests pass
- [ ] **Review diff** - Ensure changes follow pattern consistently
- [ ] **Document insights** - Note any design decisions or patterns for future reference

---

## Common Reusable Parsers

These parsers are defined in `llmspell-bridge/src/lua/globals/agent.rs` and can be reused across multiple bridge methods:

### 1. `parse_context_scope()` - Context Scoping

**Location**: `llmspell-bridge/src/lua/globals/agent.rs:168-222`

**Purpose**: Parse Lua string or table to `ContextScope` enum from llmspell-core

**Usage**: Context operations (create_context, create_child_context, set_shared_memory, get_shared_memory)

**Signature**:
```rust
fn parse_context_scope(value: &Value) -> mlua::Result<ContextScope>
```

**Supported Formats**:
```lua
-- String format (Global only)
scope = "global"

-- Table format (all variants)
scope = { type = "session", id = "session-123" }
scope = { type = "workflow", id = "workflow-456" }
scope = { type = "agent", id = "agent-789" }
scope = { type = "user", id = "user-abc" }
```

**Implementation Pattern**:
```rust
fn parse_context_scope(value: &Value) -> mlua::Result<ContextScope> {
    match value {
        Value::String(s) => {
            let scope_str = s.to_str()?;
            if scope_str == "global" {
                Ok(ContextScope::Global)
            } else {
                Err(mlua::Error::RuntimeError(
                    "Invalid simple scope. Use table for session/workflow/agent/user scopes".to_string(),
                ))
            }
        }
        Value::Table(table) => {
            let scope_type: String = table.get("type")?;
            match scope_type.as_str() {
                "global" => Ok(ContextScope::Global),
                "session" => {
                    let id: String = table.get("id")?;
                    Ok(ContextScope::Session(id))
                }
                "workflow" => {
                    let id: String = table.get("id")?;
                    Ok(ContextScope::Workflow(id))
                }
                "agent" => {
                    let id: String = table.get("id")?;
                    Ok(ContextScope::Agent(ComponentId::from_name(&id)))
                }
                "user" => {
                    let id: String = table.get("id")?;
                    Ok(ContextScope::User(id))
                }
                _ => Err(mlua::Error::RuntimeError(format!(
                    "Unknown scope type: {scope_type}. Expected: global, session, workflow, agent, user"
                ))),
            }
        }
        _ => Err(mlua::Error::RuntimeError(
            "Scope must be a string ('global') or table with 'type' field".to_string(),
        )),
    }
}
```

**Reuse Count**: Used by 4 methods (create_context, create_child_context, set_shared_memory, get_shared_memory)

### 2. `parse_inheritance_policy()` - Context Inheritance

**Location**: `llmspell-bridge/src/lua/globals/agent.rs:224-234`

**Purpose**: Parse string to `InheritancePolicy` enum from llmspell-core

**Usage**: Context creation operations

**Signature**:
```rust
fn parse_inheritance_policy(policy_str: &str) -> InheritancePolicy
```

**Supported Values**:
```lua
inheritance = "inherit"  -- Default
inheritance = "isolate"  -- No parent data
inheritance = "copy"     -- Copy parent data
inheritance = "share"    -- Share parent data
```

**Implementation**:
```rust
fn parse_inheritance_policy(policy_str: &str) -> InheritancePolicy {
    match policy_str {
        "isolate" => InheritancePolicy::Isolate,
        "copy" => InheritancePolicy::Copy,
        "share" => InheritancePolicy::Share,
        _ => InheritancePolicy::Inherit, // Default
    }
}
```

### 3. `parse_model_config()` - LLM Model Configuration

**Location**: `llmspell-bridge/src/lua/globals/agent.rs` (exact line varies)

**Purpose**: Parse Lua table to `ModelConfig` struct from llmspell-agents

**Usage**: Agent creation

**Signature**:
```rust
fn parse_model_config(table: &Table) -> mlua::Result<ModelConfig>
```

**Expected Fields**:
```lua
model_config = {
    provider = "openai",  -- or "anthropic", "ollama@tinyllama", etc.
    model_id = "gpt-4",
    temperature = 0.7,    -- Optional
    max_tokens = 2000,    -- Optional
    -- Provider-specific settings
}
```

---

## Complete Examples

### Example 1: Simple Config (ToolWrapperConfig)

**From Task 11a.8.6**: Wrapping agents as tools

#### 1. Struct Definition
```rust
// llmspell-bridge/src/agent_bridge.rs:133-152
/// Configuration for wrapping an agent as a tool
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolWrapperConfig {
    /// Name for the wrapped tool
    pub tool_name: String,
    /// Tool category (defaults to Utility)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub category: Option<llmspell_core::traits::tool::ToolCategory>,
    /// Security level (defaults to Restricted)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub security_level: Option<llmspell_core::traits::tool::SecurityLevel>,
}
```

#### 2. Parser Implementation
```rust
// llmspell-bridge/src/lua/globals/agent.rs:375-417
fn parse_tool_wrapper_config(table: &Table) -> crate::agent_bridge::ToolWrapperConfig {
    use crate::agent_bridge::ToolWrapperConfig;
    use llmspell_core::traits::tool::{SecurityLevel, ToolCategory};

    let tool_name: String = table.get("tool_name").unwrap_or_else(|_| String::new());

    let category = table
        .get::<_, Option<String>>("category")
        .unwrap_or(None)
        .map(|cat_str| match cat_str.as_str() {
            "filesystem" => ToolCategory::Filesystem,
            "web" => ToolCategory::Web,
            "api" => ToolCategory::Api,
            "analysis" => ToolCategory::Analysis,
            "data" => ToolCategory::Data,
            "system" => ToolCategory::System,
            "media" => ToolCategory::Media,
            "utility" => ToolCategory::Utility,
            custom => ToolCategory::Custom(custom.to_string()),
        });

    let security_level = table
        .get::<_, Option<String>>("security_level")
        .unwrap_or(None)
        .and_then(|level_str| match level_str.as_str() {
            "safe" => Some(SecurityLevel::Safe),
            "restricted" => Some(SecurityLevel::Restricted),
            "privileged" => Some(SecurityLevel::Privileged),
            _ => None,
        });

    ToolWrapperConfig {
        tool_name,
        category,
        security_level,
    }
}
```

#### 3. Bridge Method
```rust
// llmspell-bridge/src/agent_bridge.rs:1397-1431
pub async fn wrap_agent_as_tool(
    &self,
    agent_name: &str,
    config: ToolWrapperConfig,
) -> Result<String> {
    let agent = self.get_agent(agent_name).await
        .ok_or_else(|| LLMSpellError::Component {
            message: format!("Agent '{agent_name}' not found"),
            source: None,
        })?;

    let tool_name = if config.tool_name.is_empty() {
        format!("{agent_name}_tool")
    } else {
        config.tool_name
    };

    let category = config.category.unwrap_or(ToolCategory::Utility);
    let security_level = config.security_level.unwrap_or(SecurityLevel::Restricted);

    let wrapped_tool = AgentWrappedTool::new(agent.clone(), category, security_level);
    self.registry.register_tool(tool_name.clone(), Arc::new(wrapped_tool))?;

    Ok(tool_name)
}
```

#### 4. Lua Binding
```rust
// llmspell-bridge/src/lua/globals/agent.rs:1697-1716
let wrap_as_tool_fn = lua.create_function(move |_lua, args: (String, Table)| {
    let (agent_name, config) = args;
    let bridge = bridge_clone.clone();

    let config_typed = parse_tool_wrapper_config(&config);

    let tool_name = block_on_async(
        "agent_wrapAsTool",
        bridge.wrap_agent_as_tool(&agent_name, config_typed),
        None,
    )
    .map_err(|e| mlua::Error::RuntimeError(format!("Failed to wrap agent as tool: {e}")))?;

    Ok(tool_name)
})?;
```

#### 5. Lua Usage
```lua
-- Minimal usage
local tool_name = Agent.wrap_as_tool("my_agent", { tool_name = "agent_tool" })

-- Full usage
local tool_name = Agent.wrap_as_tool("my_agent", {
    tool_name = "advanced_agent",
    category = "api",
    security_level = "restricted"
})
```

### Example 2: Nested Config (ExecutionContextConfig)

**From Task 11a.8.4**: Creating execution contexts with nested security config

#### 1. Struct Definitions
```rust
// llmspell-bridge/src/agent_bridge.rs:69-111
/// Security context configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SecurityContextConfig {
    /// Security permissions
    #[serde(default)]
    pub permissions: Vec<String>,
    /// Security level
    #[serde(default = "default_security_level")]
    pub level: String,
}

const fn default_security_level() -> String {
    String::from("default")
}

/// Configuration for execution context creation
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ExecutionContextConfig {
    /// Conversation ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub conversation_id: Option<String>,

    /// User ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_id: Option<String>,

    /// Session ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session_id: Option<String>,

    /// Context scope
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scope: Option<ContextScope>,

    /// Inheritance policy
    #[serde(skip_serializing_if = "Option::is_none")]
    pub inheritance: Option<InheritancePolicy>,

    /// Context data
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<HashMap<String, Value>>,

    /// Security configuration
    #[serde(skip_serializing_if = "Option::is_none")]
    pub security: Option<SecurityContextConfig>,
}
```

#### 2. Parser Implementation
```rust
// llmspell-bridge/src/lua/globals/agent.rs:236-301
fn parse_execution_context_config(table: &Table) -> mlua::Result<ExecutionContextConfig> {
    let conversation_id: Option<String> = table.get("conversation_id").ok();
    let user_id: Option<String> = table.get("user_id").ok();
    let session_id: Option<String> = table.get("session_id").ok();

    // Parse scope using reusable parser
    let scope = table
        .get::<_, Option<Value>>("scope")
        .ok()
        .flatten()
        .map(|v| parse_context_scope(&v))
        .transpose()?;

    // Parse inheritance using reusable parser
    let inheritance = table
        .get::<_, Option<String>>("inheritance")
        .ok()
        .flatten()
        .map(|s| parse_inheritance_policy(&s));

    // Parse data (Lua table → HashMap)
    let data = table
        .get::<_, Option<Table>>("data")
        .ok()
        .flatten()
        .map(|data_table| {
            let mut map = HashMap::new();
            for pair in data_table.pairs::<String, Value>() {
                let (key, value) = pair?;
                let json_value = lua_value_to_json(value)?;
                map.insert(key, json_value);
            }
            Ok::<_, mlua::Error>(map)
        })
        .transpose()?;

    // Parse security (nested struct)
    let security = table
        .get::<_, Option<Table>>("security")
        .ok()
        .flatten()
        .map(|sec_table| {
            let permissions: Vec<String> = sec_table.get("permissions").unwrap_or_default();
            let level: String = sec_table.get("level").unwrap_or_else(|_| "default".to_string());
            SecurityContextConfig { permissions, level }
        });

    Ok(ExecutionContextConfig {
        conversation_id,
        user_id,
        session_id,
        scope,
        inheritance,
        data,
        security,
    })
}
```

#### 3. Bridge Method
```rust
// llmspell-bridge/src/agent_bridge.rs:1060-1114
pub async fn create_context(&self, config: ExecutionContextConfig) -> Result<ContextId> {
    let mut builder = ExecutionContextBuilder::new();

    if let Some(conv_id) = config.conversation_id {
        builder = builder.with_conversation_id(conv_id);
    }
    if let Some(user_id) = config.user_id {
        builder = builder.with_user_id(user_id);
    }
    if let Some(session_id) = config.session_id {
        builder = builder.with_session_id(session_id);
    }
    if let Some(scope) = config.scope {
        builder = builder.with_scope(scope);
    }
    if let Some(inheritance) = config.inheritance {
        builder = builder.with_inheritance(inheritance);
    }
    if let Some(data) = config.data {
        for (key, value) in data {
            builder = builder.with_data(key, value);
        }
    }
    if let Some(security) = config.security {
        builder = builder.with_security_permissions(security.permissions);
        builder = builder.with_security_level(&security.level);
    }

    let context = builder.build();
    let context_id = context.id().clone();
    self.contexts.write().await.insert(context_id.clone(), context);

    Ok(context_id)
}
```

#### 4. Lua Usage
```lua
-- Full context creation
local context_id = Agent.create_context({
    conversation_id = "conv-123",
    user_id = "user-456",
    session_id = "session-789",
    scope = { type = "session", id = "session-789" },
    inheritance = "inherit",
    data = {
        temperature = 0.7,
        max_tokens = 2000
    },
    security = {
        permissions = { "read", "write" },
        level = "restricted"
    }
})
```

### Example 3: Enum Config (RoutingConfig)

**From Task 11a.8.3**: Creating composite agents with routing strategies

#### 1. Enum + Struct Definitions
```rust
// llmspell-bridge/src/agent_bridge.rs:28-67
/// Routing strategy for composite agents
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum RoutingStrategy {
    /// Execute delegates in order
    Sequential,
    /// Execute delegates concurrently
    Parallel,
    /// Consensus-based execution
    Vote {
        /// Minimum votes required
        #[serde(skip_serializing_if = "Option::is_none")]
        threshold: Option<usize>,
    },
    /// Custom user-defined strategy
    Custom {
        /// Strategy name
        name: String,
    },
}

impl Default for RoutingStrategy {
    fn default() -> Self {
        Self::Sequential
    }
}

/// Routing configuration for composite agents
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RoutingConfig {
    /// Routing strategy
    #[serde(default)]
    pub strategy: RoutingStrategy,

    /// Fallback agent name
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fallback_agent: Option<String>,

    /// Timeout in milliseconds
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timeout_ms: Option<u64>,
}
```

#### 2. Parser Implementation (Flexible: String or Table)
```rust
// llmspell-bridge/src/lua/globals/agent.rs:168-230
fn parse_routing_config(value: &Value) -> mlua::Result<RoutingConfig> {
    match value {
        // String format: "sequential", "parallel", "vote"
        Value::String(s) => {
            let strategy_str = s.to_str()?;
            let strategy = match strategy_str {
                "sequential" => RoutingStrategy::Sequential,
                "parallel" => RoutingStrategy::Parallel,
                "vote" => RoutingStrategy::Vote { threshold: None },
                custom => RoutingStrategy::Custom {
                    name: custom.to_string(),
                },
            };
            Ok(RoutingConfig {
                strategy,
                fallback_agent: None,
                timeout_ms: None,
            })
        }
        // Table format: { strategy = "...", threshold = 3, fallback_agent = "...", timeout_ms = 5000 }
        Value::Table(table) => {
            let strategy_str: String = table.get("strategy").unwrap_or_else(|_| "sequential".to_string());

            let strategy = match strategy_str.as_str() {
                "sequential" => RoutingStrategy::Sequential,
                "parallel" => RoutingStrategy::Parallel,
                "vote" => {
                    let threshold: Option<usize> = table.get("threshold").ok();
                    RoutingStrategy::Vote { threshold }
                }
                custom => RoutingStrategy::Custom {
                    name: custom.to_string(),
                },
            };

            let fallback_agent: Option<String> = table.get("fallback_agent").ok();
            let timeout_ms: Option<u64> = table.get("timeout_ms").ok();

            Ok(RoutingConfig {
                strategy,
                fallback_agent,
                timeout_ms,
            })
        }
        _ => Err(mlua::Error::RuntimeError(
            "Routing config must be a string or table".to_string(),
        )),
    }
}
```

#### 3. Lua Usage
```lua
-- Simple string format
Agent.create_composite("comp1", delegates, "sequential")
Agent.create_composite("comp2", delegates, "parallel")

-- Table format with options
Agent.create_composite("comp3", delegates, {
    strategy = "vote",
    threshold = 2,
    fallback_agent = "default_agent",
    timeout_ms = 5000
})

-- Custom strategy
Agent.create_composite("comp4", delegates, {
    strategy = "round_robin",  -- Becomes Custom { name = "round_robin" }
    timeout_ms = 3000
})
```

---

## Testing Requirements

### 1. Unit Tests for Parsers

**Location**: Add tests in the same file as parsers

**Example**:
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use mlua::Lua;

    #[test]
    fn test_parse_tool_wrapper_config_minimal() {
        let lua = Lua::new();
        let table = lua.create_table().unwrap();
        table.set("tool_name", "test_tool").unwrap();

        let config = parse_tool_wrapper_config(&table);

        assert_eq!(config.tool_name, "test_tool");
        assert!(config.category.is_none());
        assert!(config.security_level.is_none());
    }

    #[test]
    fn test_parse_tool_wrapper_config_full() {
        let lua = Lua::new();
        let table = lua.create_table().unwrap();
        table.set("tool_name", "test_tool").unwrap();
        table.set("category", "api").unwrap();
        table.set("security_level", "restricted").unwrap();

        let config = parse_tool_wrapper_config(&table);

        assert_eq!(config.tool_name, "test_tool");
        assert_eq!(config.category, Some(ToolCategory::Api));
        assert_eq!(config.security_level, Some(SecurityLevel::Restricted));
    }
}
```

### 2. Integration Tests for Bridge Methods

**Location**: Update existing tests in `llmspell-bridge/src/agent_bridge.rs`

**Pattern**: Replace JSON fixtures with typed structs

**Example**:
```rust
#[tokio::test]
async fn test_wrap_agent_as_tool() {
    let bridge = create_test_bridge();

    // Create test agent first
    let agent_config = create_test_agent_config("test_agent");
    bridge.create_agent(agent_config).await.unwrap();

    // ✅ Use typed config instead of JSON
    let config = ToolWrapperConfig {
        tool_name: "test_tool".to_string(),
        category: Some(ToolCategory::Api),
        security_level: Some(SecurityLevel::Restricted),
    };

    let tool_name = bridge.wrap_agent_as_tool("test_agent", config).await.unwrap();
    assert_eq!(tool_name, "test_tool");

    // Verify tool was registered
    let tools = bridge.list_tools();
    assert!(tools.contains(&"test_tool".to_string()));
}
```

### 3. End-to-End Lua Tests

**Location**: `llmspell-bridge/tests/*.rs` integration tests

**Example**:
```rust
#[tokio::test]
async fn test_lua_agent_wrapping() {
    let lua = create_test_lua_with_bridge().await;

    lua.load(r#"
        -- Create agent
        local agent_config = {
            name = "test_agent",
            agent_type = "basic",
            allowed_tools = {}
        }
        Agent.create(agent_config)

        -- Wrap as tool with typed config
        local tool_name = Agent.wrap_as_tool("test_agent", {
            tool_name = "wrapped_tool",
            category = "api",
            security_level = "restricted"
        })

        assert(tool_name == "wrapped_tool", "Tool name mismatch")

        -- Verify tool exists
        local tools = Tool.list()
        local found = false
        for _, name in ipairs(tools) do
            if name == "wrapped_tool" then
                found = true
                break
            end
        end
        assert(found, "Wrapped tool not found in tool list")
    "#).exec().unwrap();
}
```

### 4. Validation Checklist

Before marking a task complete, verify:

- [ ] **Clippy**: `cargo clippy -p llmspell-bridge --all-targets --all-features -- -D warnings` → 0 warnings
- [ ] **Tests**: `cargo test -p llmspell-bridge --all-features` → All tests pass
- [ ] **Parser tests**: Unit tests for all parser functions added
- [ ] **Bridge tests**: Integration tests updated to use typed structs
- [ ] **No regressions**: Test count stays same or increases
- [ ] **Dead code removed**: No leftover JSON construction in tests
- [ ] **Error messages**: Lua error messages are clear and helpful

---

## Troubleshooting

### Issue 1: "Type `X` is defined multiple times"

**Symptom**:
```
error[E0255]: the name `AlertConfig` is defined multiple times
   |
11 |     AgentMetrics, AlertConfig, AlertManager, ...
   |                   ----------- previous import of the type `AlertConfig` here
...
181 | pub struct AlertConfig {
    | ^^^^^^^^^^^^^^^^^^^^^^ `AlertConfig` redefined here
```

**Cause**: Bridge-specific struct name conflicts with imported type from llmspell-agents or llmspell-core

**Solution**: Rename bridge-specific struct with `Bridge` prefix

```rust
// ✅ SOLUTION
pub struct BridgeAlertConfig {  // Renamed to avoid conflict
    // ...
}
```

**Example**: Task 11a.8.6 renamed `AlertConfig` → `BridgeAlertConfig` to avoid conflict with `llmspell_agents::AlertConfig`

### Issue 2: "This function's return value is unnecessarily wrapped by `Result`"

**Symptom**:
```
error: this function's return value is unnecessarily wrapped by `Result`
   |
379 | fn parse_tool_wrapper_config(table: &Table) -> mlua::Result<ToolWrapperConfig> {
    | ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
```

**Cause**: Parser provides defaults for all fields, so it cannot fail

**Solution**: Return type directly instead of `Result`

```rust
// ❌ BEFORE
fn parse_tool_wrapper_config(table: &Table) -> mlua::Result<ToolWrapperConfig> {
    let tool_name = table.get("tool_name").unwrap_or_else(|_| String::new());
    // ... all fields have defaults
    Ok(ToolWrapperConfig { ... })
}

// ✅ AFTER
fn parse_tool_wrapper_config(table: &Table) -> ToolWrapperConfig {
    let tool_name = table.get("tool_name").unwrap_or_else(|_| String::new());
    // ... all fields have defaults
    ToolWrapperConfig { ... }  // Direct return, no Ok()
}
```

### Issue 3: "Using `Option.and_then(|x| Some(y))`, use `map` instead"

**Symptom**:
```
error: using `Option.and_then(|x| Some(y))`, which is more succinctly expressed as `map(|x| y)`
    |
386 |       let category = table
    |  ____________________^
387 | |         .get::<_, Option<String>>("category")
388 | |         .unwrap_or(None)
389 | |         .and_then(|cat_str| match cat_str.as_str() {
...   |
398 | |             custom => Some(ToolCategory::Custom(custom.to_string())),
399 | |         });
```

**Cause**: Using `and_then` when all match arms return `Some(...)`

**Solution**: Use `map` instead of `and_then`

```rust
// ❌ BEFORE
.and_then(|cat_str| match cat_str.as_str() {
    "api" => Some(ToolCategory::Api),
    "web" => Some(ToolCategory::Web),
    // ... all arms return Some
})

// ✅ AFTER
.map(|cat_str| match cat_str.as_str() {
    "api" => ToolCategory::Api,
    "web" => ToolCategory::Web,
    // ... no Some() wrapping needed
})
```

### Issue 4: "Item in documentation is missing backticks"

**Symptom**:
```
error: item in documentation is missing backticks
   |
377 | /// - Minimal: { tool_name = "my_tool" }
    |                  ^^^^^^^^^
```

**Cause**: Doc comment references code identifiers without backticks

**Solution**: Wrap code identifiers in backticks

```rust
// ❌ BEFORE
/// - tool_name: string (required)
/// - Minimal: { tool_name = "my_tool" }

// ✅ AFTER
/// - `tool_name`: string (required)
/// - Minimal: { `tool_name` = `"my_tool"` }
```

### Issue 5: "This could be a `const fn`"

**Symptom**:
```
error: this could be a `const fn`
   |
196 | fn default_enabled() -> bool {
    | ^^^^^^^^^^^^^^^^^^^^^^^^^^^^
```

**Cause**: Simple default function that could be const

**Solution**: Add `const` keyword

```rust
// ❌ BEFORE
fn default_enabled() -> bool {
    true
}

// ✅ AFTER
const fn default_enabled() -> bool {
    true
}
```

### Issue 6: Parser Not Found When Calling

**Symptom**:
```
error[E0425]: cannot find function `parse_tool_wrapper_config` in this scope
```

**Cause**: Parser defined in wrong file or not in scope

**Solution**: Ensure parser is in same file as bindings OR import it

```rust
// ✅ SOLUTION 1: Define parser in same file as bindings
// llmspell-bridge/src/lua/globals/agent.rs

fn parse_tool_wrapper_config(table: &Table) -> ToolWrapperConfig {
    // ... parser implementation
}

// Later in same file:
let config = parse_tool_wrapper_config(&table);  // ✅ In scope

// ✅ SOLUTION 2: If parser needs to be in different file, import it
use crate::lua::parsers::parse_tool_wrapper_config;
```

### Issue 7: Unused Import After Refactoring

**Symptom**:
```
error: unused import: `lua_table_to_json`
 |
9 |     agent_output_to_lua_table, json_to_lua_value, lua_table_to_json,
  |                                                    ^^^^^^^^^^^^^^^^^
```

**Cause**: Replaced `lua_table_to_json()` with typed parser, but didn't remove import

**Solution**: Remove unused import

```rust
// ❌ BEFORE
use crate::lua::conversion::{
    agent_output_to_lua_table, json_to_lua_value, lua_table_to_json,
    lua_value_to_json,
};

// ✅ AFTER
use crate::lua::conversion::{
    agent_output_to_lua_table, json_to_lua_value, lua_value_to_json,
};
```

---

## Design Decisions Reference

### When to Create Bridge-Specific Types vs Reuse Core Types

**Reuse core types when**:
- Type is simple and fully constructible from Lua (e.g., `ContextScope`, `InheritancePolicy`)
- Type already exists in llmspell-core or llmspell-agents
- Type has no Rust-specific constructs (Arc, Box, dyn traits)

**Create bridge-specific types when**:
- Core type is too complex for Lua API surface (e.g., `ExecutionPattern` with nested types)
- Core type contains Rust-specific constructs (e.g., `Arc<dyn AlertEvaluator>`)
- Bridge needs simpler API than core provides (e.g., `RoutingStrategy` vs `ExecutionPattern`)

**Example from Task 11a.8.6**:
```rust
// ❌ Can't reuse llmspell_agents::AlertCondition directly
pub enum AlertCondition {
    MetricThreshold { ... },
    HealthStatus { ... },
    ErrorRate { ... },
    Custom {
        evaluator: Arc<dyn AlertEvaluator>,  // ❌ Cannot construct from Lua
    },
}

// ✅ Created bridge-specific simplified version
pub enum AlertConditionConfig {
    MetricThreshold { ... },
    HealthStatus { ... },
    ErrorRate { ... },
    // No Custom variant - not needed for Lua API
}
```

### When to Make Parsers Failable vs Infallible

**Return `mlua::Result<T>` when**:
- Required fields must be present (e.g., "name" is required)
- Field values need validation (e.g., threshold must be > 0)
- Parsing can fail (e.g., invalid enum variant string)

**Return `T` directly when**:
- All fields have sensible defaults
- Parser uses `unwrap_or()` or `unwrap_or_else()` for all fields
- No validation required

**Example**:
```rust
// ✅ Failable parser (name is required)
fn parse_agent_config(table: &Table) -> mlua::Result<AgentConfig> {
    let name: String = table.get("name")?;  // ❌ Fails if missing
    // ...
    Ok(AgentConfig { name, ... })
}

// ✅ Infallible parser (all fields have defaults)
fn parse_tool_wrapper_config(table: &Table) -> ToolWrapperConfig {
    let tool_name: String = table.get("tool_name").unwrap_or_else(|_| String::new());
    let category = table.get("category").ok();  // ✅ Option, no failure
    ToolWrapperConfig { tool_name, category }
}
```

### When to Support Flexible Input (String or Table)

**Support both formats when**:
- Simple cases are common (e.g., `scope = "global"`)
- Advanced cases need configuration (e.g., `scope = { type = "session", id = "123" }`)
- API convenience is important

**Example from Task 11a.8.4**:
```rust
fn parse_context_scope(value: &Value) -> mlua::Result<ContextScope> {
    match value {
        Value::String(s) => {
            // ✅ Simple format: scope = "global"
            if s.to_str()? == "global" {
                Ok(ContextScope::Global)
            } else {
                Err(...)
            }
        }
        Value::Table(table) => {
            // ✅ Advanced format: scope = { type = "session", id = "..." }
            let scope_type: String = table.get("type")?;
            match scope_type.as_str() {
                "session" => Ok(ContextScope::Session(table.get("id")?)),
                // ...
            }
        }
        _ => Err(...)
    }
}
```

### When to Use Default Trait vs Custom Default Function

**Use `#[derive(Default)]` when**:
- All fields have `Default` implementations
- Default behavior is "all fields default"

**Use custom `Default` impl when**:
- Need specific default values (not type defaults)
- Want to document what defaults mean

**Use default functions for Serde when**:
- Need const-compatible defaults
- Want to share defaults across structs

**Example**:
```rust
// ✅ Derived Default (all fields have Default)
#[derive(Default)]
pub struct ExecutionContextConfig {
    pub conversation_id: Option<String>,  // defaults to None
    pub user_id: Option<String>,          // defaults to None
    // ...
}

// ✅ Custom Default impl (specific defaults)
impl Default for RoutingStrategy {
    fn default() -> Self {
        Self::Sequential  // ✅ Specific default, not unit variant
    }
}

// ✅ Default function for Serde (const-compatible)
#[derive(Serialize, Deserialize)]
pub struct BridgeAlertConfig {
    #[serde(default = "default_enabled")]  // ✅ Uses function
    pub enabled: bool,
}

const fn default_enabled() -> bool {
    true
}
```

---

## Summary

The bridge pattern provides:

1. **Compile-time validation** - Rust compiler catches configuration errors
2. **Zero serialization overhead** - Direct struct passing, no JSON
3. **Clear error messages** - mlua reports exact field/type mismatches
4. **Self-documenting code** - Struct fields show API contract
5. **Refactoring safety** - Breaking changes caught at compile time
6. **IDE support** - Full autocomplete and type checking
7. **Config-driven creation** - Infrastructure module pattern (Phase 13b.16)
8. **Dependency safety** - Correct initialization order enforced

**Completed Coverage** (Phase 11a.8):
- ✅ Task 11a.8.1: `AgentConfig` (create_agent)
- ✅ Task 11a.8.2: `AgentConfig` refinement with `ModelConfig` + `ResourceLimits`
- ✅ Task 11a.8.3: `RoutingConfig` (create_composite_agent)
- ✅ Task 11a.8.4: `ExecutionContextConfig` + `ChildContextConfig` (create_context, create_child_context)
- ✅ Task 11a.8.5: `ContextScope` reuse (set_shared_memory, get_shared_memory)
- ✅ Task 11a.8.6: `ToolWrapperConfig` + `BridgeAlertConfig` (wrap_agent_as_tool, configure_agent_alerts)

**Infrastructure Module Pattern** (Phase 13b.16):
- ✅ `Infrastructure::from_config()` - Single creation path for 9 components
- ✅ Dependency graph enforcement (5-level initialization order)
- ✅ Hot-swappable storage backends (memory/sqlite/postgres)
- ✅ Per-component backend configuration
- ✅ CLI simplification (200+ LOC → 12 LOC, 94% reduction)
- ✅ `ScriptRuntime::new()` - Replaces `new_with_lua()` (deprecated)

**Pattern applies to**: All future bridge methods accepting configuration parameters + all infrastructure component creation

**Evolution Path**:
1. **Phase 11a.8**: Typed structs replace JSON in bridge method signatures
2. **Phase 13b.16**: Infrastructure module unifies component creation from config
3. **Future**: Extend pattern to plugin architecture and dynamic component loading

---

**Version History**:
- **1.0** (2025-10-07): Initial guide documenting Phase 11a.8 bridge pattern consolidation
- **1.1** (2025-11-08): Added Infrastructure Module Pattern from Phase 13b.16 (config-driven component creation, dependency order enforcement, hot-swappable backends)
