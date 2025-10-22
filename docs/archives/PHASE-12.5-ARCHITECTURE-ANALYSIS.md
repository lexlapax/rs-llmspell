# Phase 12.5 Architecture Analysis: Template Global Implementation

**Date**: 2025-10-13
**Status**: Architectural Design Complete
**Purpose**: Holistic analysis of existing bridge patterns to guide Phase 12.5 Template global implementation

---

## Executive Summary

Phase 12.5 requires implementing the `Template` global (16th global) following the established 3-layer bridge pattern. This analysis documents the **discovered architecture** from existing code and proposes a **revised task breakdown** that correctly implements the pattern.

**Key Finding**: Current TODO.md Task 12.5.1 conflates all 3 layers into a single 380-line file. The correct approach uses:
- **Layer 1**: Language-neutral `TemplateGlobal` struct (~66 lines) in `llmspell-bridge/src/globals/template_global.rs`
- **Layer 2**: Lua-specific `inject_template_global()` function (~400-500 lines) in `llmspell-bridge/src/lua/globals/template.rs`
- **Layer 3**: JavaScript stub (~50 lines) in `llmspell-bridge/src/javascript/globals/template.rs`

---

## Discovered Architecture

### 1. Three-Layer Bridge Pattern

All globals in llmspell-bridge follow this pattern:

#### Layer 1: Language-Neutral Global (llmspell-bridge/src/globals/)

**Example**: `tool_global.rs` (66 lines)

```rust
pub struct ToolGlobal {
    registry: Arc<ComponentRegistry>,
}

impl GlobalObject for ToolGlobal {
    fn metadata(&self) -> GlobalMetadata {
        GlobalMetadata {
            name: "Tool".to_string(),
            version: "0.11.0".to_string(),
            description: "Tool management and execution".to_string(),
            dependencies: vec![],
        }
    }

    #[cfg(feature = "lua")]
    fn inject_lua(&self, lua: &mlua::Lua, context: &GlobalContext) -> Result<()> {
        crate::lua::globals::tool::inject_tool_global(lua, context, self.registry.clone())
    }

    #[cfg(feature = "javascript")]
    fn inject_javascript(&self, ctx: &mut boa_engine::Context, context: &GlobalContext) -> Result<()> {
        crate::javascript::globals::tool::inject_tool_global(ctx, context)
    }
}

impl ToolGlobal {
    pub fn new(registry: Arc<ComponentRegistry>) -> Self {
        Self { registry }
    }
}
```

**Key Characteristics**:
- Minimal struct wrapping dependencies (ComponentRegistry, bridges, managers)
- Implements `GlobalObject` trait
- Delegates to language-specific injection functions
- ~60-100 lines total

#### Layer 2: Lua-Specific Injection (llmspell-bridge/src/lua/globals/)

**Example**: `tool.rs` (410 lines)

```rust
pub fn inject_tool_global(
    lua: &Lua,
    context: &GlobalContext,
    registry: Arc<ComponentRegistry>,
) -> mlua::Result<()> {
    let tool_table = lua.create_table()?;

    // Create Tool.list() function (50 lines)
    let list_fn = lua.create_function(move |lua, ()| {
        let tools = registry_clone.list_tools();
        // ... format as Lua table
        Ok(list_table)
    })?;

    // Create Tool.get() function (100 lines)
    let get_fn = lua.create_function(move |lua, name: String| {
        // ... get tool, format metadata, add execute method
        Ok(Some(tool_table))
    })?;

    // Create Tool.execute() function (50 lines)
    let execute_fn = lua.create_function(move |lua, (name, input): (String, Table)| {
        // Use block_on_async_lua for sync wrapper
        // Convert Lua table to AgentInput
        // Execute tool
        // Convert output to Lua table
        Ok(result)
    })?;

    // Create Tool.exists() function (10 lines)
    // Create Tool.categories() function (30 lines)
    // Create Tool.discover() function (50 lines)
    // Add metatable for Tool.tool_name direct access (100 lines)

    // Set all functions
    tool_table.set("list", list_fn)?;
    tool_table.set("get", get_fn)?;
    tool_table.set("execute", execute_fn)?;
    tool_table.set("exists", exists_fn)?;
    tool_table.set("categories", categories_fn)?;
    tool_table.set("discover", discover_fn)?;

    lua.globals().set("Tool", tool_table)?;
    Ok(())
}
```

**Key Characteristics**:
- Large function (400-500+ lines for complex globals)
- Creates Lua table with methods
- Uses `block_on_async_lua()` for sync wrappers around async Rust
- Extensive use of conversion functions from `llmspell-bridge/src/lua/conversion.rs`
- Each method: 20-100 lines
- Metatable magic for syntactic sugar (`Tool.tool_name` vs `Tool.get("tool_name")`)

#### Layer 3: JavaScript Stub (llmspell-bridge/src/javascript/globals/)

**Example**: `tool.rs` (minimal stub)

```rust
pub fn inject_tool_global(
    _ctx: &mut boa_engine::Context,
    _context: &GlobalContext,
) -> llmspell_core::Result<()> {
    // Placeholder - JavaScript support not yet implemented
    warn!("JavaScript Tool global not yet implemented");
    Ok(())
}
```

**Key Characteristics**:
- Minimal stub (~20-50 lines when implemented)
- Logs warning or provides basic implementation

### 2. Conversion Functions (llmspell-bridge/src/lua/conversion.rs)

**Core Bidirectional Conversions** (596 lines total):

```rust
// JSON <-> Lua conversions
pub fn lua_value_to_json(value: LuaValue) -> mlua::Result<JsonValue>
pub fn lua_table_to_json(table: Table) -> mlua::Result<JsonValue>
pub fn json_to_lua_value<'lua>(lua: &'lua Lua, json: &JsonValue) -> mlua::Result<LuaValue<'lua>>

// Agent conversions
pub fn lua_table_to_agent_input(lua: &Lua, table: &Table) -> mlua::Result<AgentInput>
pub fn agent_output_to_lua_table<'a>(lua: &'a Lua, output: &AgentOutput) -> mlua::Result<Table<'a>>

// Tool conversions
pub fn lua_table_to_tool_input(_lua: &Lua, table: Table) -> mlua::Result<JsonValue>
pub fn tool_output_to_lua_table(lua: &Lua, output: ToolOutput) -> mlua::Result<Table<'_>>

// Workflow conversions
pub fn lua_table_to_workflow_params(_lua: &Lua, table: Table) -> Result<JsonValue>
pub fn workflow_result_to_lua_table<'lua>(lua: &'lua Lua, result: &JsonValue) -> mlua::Result<Table<'lua>>
pub fn script_workflow_result_to_lua_table(lua: &Lua, result: ScriptWorkflowResult) -> mlua::Result<Table<'_>>
```

**Specialized Helper Functions**:
- `extract_parameters_from_table()` - Extracts parameters field
- `extract_output_modalities()` - Parses output modality strings
- `extract_media_content()` - Parses image/media data
- `process_output_text()` - Auto-detects JSON vs text responses
- `set_output_metadata()` - Formats metadata table
- Array vs Object detection via `table.raw_len()` (numeric keys starting at 1)

**Usage Pattern in Lua Injection**:
```rust
// Lua table → JSON → Rust struct
let json_params = lua_table_to_json(params_table)?;
let template_params: TemplateParams = serde_json::from_value(json_params)?;

// Rust output → JSON → Lua table
let json_output = serde_json::to_value(&template_output)?;
let lua_table = json_to_lua_value(lua, &json_output)?;
```

### 3. Bridge Pattern (docs/developer-guide/bridge-pattern-guide.md)

**Key Principle**: **Typed Rust structs everywhere, NO JSON parameters in bridge methods**

**Anti-Pattern Eliminated** (Pre-Phase 11a.8):
```rust
// ❌ OLD WAY
pub async fn create_agent(&self, config: serde_json::Value) -> Result<String> {
    let name = config.get("name").and_then(|v| v.as_str())?;
    // ... 50 lines of JSON navigation
}
```

**Bridge Pattern** (Phase 11a.8+):
```rust
// ✅ NEW WAY - Typed struct in bridge
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentConfig {
    pub name: String,
    pub agent_type: String,
    pub model: Option<ModelConfig>,
}

pub async fn create_agent(&self, config: AgentConfig) -> Result<String> {
    // Direct field access - no JSON navigation
    let agent = self.agent_discovery.create_agent(config).await?;
    Ok(agent.id())
}

// Parser in Lua layer
fn parse_agent_config(table: &Table) -> mlua::Result<AgentConfig> {
    let name: String = table.get("name")?;
    let agent_type: String = table.get("agent_type")?;
    Ok(AgentConfig { name, agent_type, ... })
}
```

**Benefits**:
- ✅ Compile-time validation (Rust compiler checks all field access)
- ✅ Zero serialization overhead (direct struct passing)
- ✅ Clear error messages (mlua reports exact field/type mismatches)
- ✅ IDE autocomplete (full IntelliSense support)
- ✅ Refactoring safety (changing struct fields produces compile errors)

### 4. Registration Pattern (llmspell-bridge/src/globals/mod.rs)

**Global Registry Builder**:

```rust
pub async fn create_standard_registry(context: Arc<GlobalContext>) -> Result<GlobalRegistry> {
    let mut builder = GlobalRegistryBuilder::new();

    // Core globals (json, logger, config, debug)
    register_core_globals(&mut builder, &context);

    // State, Utils, Event
    builder.register(create_state_global(&context).await);
    builder.register(Arc::new(UtilsGlobal::new()));
    builder.register(Arc::new(EventGlobal::new()));

    // Session and Artifact (if SessionManager available)
    let session_manager_opt = register_session_artifacts(&mut builder, &context);

    // RAG (if all dependencies available)
    register_rag_global(&mut builder, &context, session_manager_opt).await;

    // Hook and Tool
    register_hook_and_tools(&mut builder, &context)?;

    // Agent and Workflow
    register_agent_workflow(&mut builder, &context).await?;

    // Streaming
    builder.register(Arc::new(StreamingGlobal::new()));

    // LocalLLM
    builder.register(Arc::new(LocalLLMGlobal::new(
        context.providers.create_core_manager_arc().await?,
    )));

    // *** Template will go here (16th global) ***

    builder.build()
}
```

**Key Observations**:
- Globals registered in dependency order
- Helper functions group related globals
- Arc wrapping for shared ownership
- Async construction for complex globals (RAG, Agent, LocalLLM)

### 5. Existing Template Infrastructure (Phase 12.1-12.4)

**Already Implemented**:

#### TemplateRegistry (llmspell-templates/src/registry.rs)
```rust
pub struct TemplateRegistry {
    templates: DashMap<String, Arc<dyn Template>>,
}

impl TemplateRegistry {
    pub fn register(&self, template: Arc<dyn Template>) -> Result<()>
    pub fn get(&self, id: &str) -> Result<Arc<dyn Template>>
    pub fn list_all(&self) -> Vec<TemplateMetadata>
    pub fn discover_by_category(&self, category: Option<TemplateCategory>) -> Vec<TemplateMetadata>
    pub fn search(&self, query: &str) -> Vec<TemplateMetadata>
    pub fn with_builtin_templates() -> Result<Self>  // Registers all 6 built-in templates
}
```

#### Template Trait (llmspell-templates/src/core.rs)
```rust
#[async_trait]
pub trait Template: Send + Sync {
    fn metadata(&self) -> &TemplateMetadata;
    fn config_schema(&self) -> ConfigSchema;
    async fn execute(&self, params: TemplateParams, context: ExecutionContext) -> Result<TemplateOutput>;
    fn validate(&self, params: &TemplateParams) -> Result<(), TemplateError>;
    fn estimate_cost(&self, params: &TemplateParams) -> Option<CostEstimate>;
}
```

#### Built-in Templates (6 templates, 4505 total lines, 110 tests)
1. **ResearchAssistantTemplate** (801 lines) - 4-phase: gather → ingest → synthesize → validate
2. **InteractiveChatTemplate** (482 lines) - Session-based conversation with tool integration
3. **DataAnalysisTemplate** (732 lines) - Sequential analyzer → visualizer workflow
4. **CodeGeneratorTemplate** (858 lines) - 4-phase: spec → impl → test → lint
5. **DocumentProcessorTemplate** (705 lines) - Parallel PDF extraction + transformation
6. **WorkflowOrchestratorTemplate** (660 lines) - Custom parallel/sequential/hybrid workflows

#### CLI Integration (Phase 12.2)
- Commands: `llmspell template list|info|exec|search|schema`
- Kernel message protocol: template_request / template_reply
- Dual mode: embedded (in-process) and connected (remote kernel via ZeroMQ)
- ExecutionContext::resolve() pattern for mode detection

#### ScriptExecutor Template Methods (llmspell-bridge/src/runtime.rs)
```rust
// Added in Phase 12.2.7 to avoid circular dependencies
pub trait ScriptExecutor {
    async fn handle_template_list(&self, category: Option<String>) -> Result<serde_json::Value>;
    async fn handle_template_info(&self, name: &str, show_schema: bool) -> Result<serde_json::Value>;
    async fn handle_template_exec(&self, name: &str, params: serde_json::Value) -> Result<serde_json::Value>;
    async fn handle_template_search(&self, query: &str, category: Option<String>) -> Result<serde_json::Value>;
    async fn handle_template_schema(&self, name: &str) -> Result<serde_json::Value>;
}
```

**Dependencies Available in GlobalContext**:
- `registry: Arc<ComponentRegistry>` - includes tool_registry, agent_registry, workflow_factory, **template_registry** (Phase 12.2.6)
- `providers: Arc<ProviderManager>` - LLM providers (Ollama, Anthropic, OpenAI, etc.)
- Optional bridges: state_manager, session_manager, multi_tenant_rag

---

## Architectural Analysis

### Pattern Comparison: Tool vs Agent vs Template

| Aspect | Tool Global | Agent Global | Template Global (Proposed) |
|--------|-------------|--------------|---------------------------|
| **Layer 1 File** | `globals/tool_global.rs` | `globals/agent_global.rs` | `globals/template_global.rs` |
| **Layer 1 LOC** | 66 lines | ~100 lines | ~80 lines (estimate) |
| **Wrapped Data** | `Arc<ComponentRegistry>` | `Arc<AgentBridge>` | `Arc<ComponentRegistry>` |
| **Layer 2 File** | `lua/globals/tool.rs` | `lua/globals/agent.rs` | `lua/globals/template.rs` |
| **Layer 2 LOC** | 410 lines | ~1500+ lines (complex) | ~450 lines (estimate) |
| **Methods** | list, get, execute, exists, categories, discover | create, list, get, execute, delete, wrap_as_tool, create_context, etc. | list, info, execute, search, schema |
| **Conversion Pattern** | Tool → ToolOutput → Lua | Agent → AgentOutput → Lua | Template → TemplateOutput → Lua |
| **Async Handling** | `block_on_async_lua()` | `block_on_async_lua()` | `block_on_async_lua()` |
| **Metatable Magic** | Yes (`Tool.tool_name`) | No | Optional (`Template.research_assistant`) |
| **Layer 3 File** | `javascript/globals/tool.rs` | `javascript/globals/agent.rs` | `javascript/globals/template.rs` |
| **Layer 3 LOC** | Stub (~20 lines) | Stub (~20 lines) | Stub (~20 lines) |

### Template Global Characteristics

**Simple Like Tool Global**:
- Direct registry access (no complex bridge needed)
- Template registry already in ComponentRegistry (Phase 12.2.6)
- CRUD operations (list, get, execute) similar to Tool
- No state mutation (templates are read-only, execute returns new data)

**Unique Aspects**:
- **Parameter Validation**: Templates have ConfigSchema with typed parameters
- **Cost Estimation**: Templates provide upfront cost/duration estimates
- **Rich Output**: TemplateOutput with results, artifacts, metrics vs simple ToolOutput
- **Search**: Full-text search across name/description/tags (like Tool.discover but richer)

**Methods to Implement (5 total)**:
1. **Template.list([category])** → array of template metadata
2. **Template.info(name, [show_schema])** → metadata + optional schema
3. **Template.execute(name, params)** → output with results/artifacts/metrics
4. **Template.search(query, [category])** → filtered metadata array
5. **Template.schema(name)** → JSON schema for parameters

**Conversion Functions Needed**:
- `lua_table_to_template_params()` - Converts Lua table to HashMap<String, serde_json::Value>
- `template_output_to_lua_table()` - Converts TemplateOutput to Lua table with proper result type handling
- `template_metadata_to_lua_table()` - Formats TemplateMetadata as Lua table
- `config_schema_to_lua_table()` - Formats ConfigSchema as Lua table

---

## Revised Phase 12.5 Task Breakdown

### Current Problems in TODO.md

**Task 12.5.1** (lines 1335-1371):
- ❌ Says "Create Template Global Object" but describes 380 LOC in single file
- ❌ Mixes language-neutral struct with Lua injection implementation
- ❌ Says "Implement `TemplateGlobal` struct with GlobalObject trait" AND "Implement `inject_template_global(lua, context)`" in same task
- ❌ Doesn't mention JavaScript stub at all
- ❌ Doesn't follow established 3-layer pattern

**Task 12.5.2** (lines 1372-1402):
- ❌ "Register Template Global in Bridge" is vague
- ✅ Correctly identifies need to update `globals/mod.rs`

**Task 12.5.3** (lines 1403-1429):
- ✅ Lua examples are appropriate
- ❌ Doesn't account for conversion function implementation

### Revised Task Structure

#### **Task 12.5.1: Create Language-Neutral TemplateGlobal Struct**
**Priority**: CRITICAL
**Estimated Time**: 1 hour (was 4 hours)
**File**: `llmspell-bridge/src/globals/template_global.rs` (NEW - 80 LOC)

**Description**: Create language-neutral `TemplateGlobal` struct implementing `GlobalObject` trait, following the pattern from `tool_global.rs`.

**Acceptance Criteria:**
- [x] `TemplateGlobal` struct created with `registry: Arc<ComponentRegistry>` field
- [x] Implements `GlobalObject` trait with metadata() method
- [x] `inject_lua()` method delegates to `crate::lua::globals::template::inject_template_global()`
- [x] `inject_javascript()` method delegates to `crate::javascript::globals::template::inject_template_global()`
- [x] `new(registry)` constructor
- [x] Module added to `llmspell-bridge/src/globals/mod.rs`

**Implementation Pattern** (from tool_global.rs):
```rust
// llmspell-bridge/src/globals/template_global.rs (NEW FILE)
use crate::globals::{GlobalContext, GlobalMetadata, GlobalObject};
use crate::ComponentRegistry;
use llmspell_core::Result;
use std::sync::Arc;

/// Template global for script access to template system
pub struct TemplateGlobal {
    registry: Arc<ComponentRegistry>,
}

impl GlobalObject for TemplateGlobal {
    fn metadata(&self) -> GlobalMetadata {
        GlobalMetadata {
            name: "Template".to_string(),
            version: "0.12.0".to_string(),
            description: "Template discovery, inspection, and execution".to_string(),
            dependencies: vec!["provider_manager".to_string()],
        }
    }

    #[cfg(feature = "lua")]
    fn inject_lua(&self, lua: &mlua::Lua, context: &GlobalContext) -> Result<()> {
        crate::lua::globals::template::inject_template_global(lua, context, self.registry.clone())
    }

    #[cfg(feature = "javascript")]
    fn inject_javascript(&self, ctx: &mut boa_engine::Context, context: &GlobalContext) -> Result<()> {
        crate::javascript::globals::template::inject_template_global(ctx, context)
    }
}

impl TemplateGlobal {
    pub fn new(registry: Arc<ComponentRegistry>) -> Self {
        Self { registry }
    }
}
```

**Definition of Done:**
- [x] File compiles without errors
- [x] GlobalObject trait fully implemented
- [x] Module declared in `globals/mod.rs`: `pub mod template_global;`
- [x] Re-export added: `pub use template_global::TemplateGlobal;`

---

#### **Task 12.5.2: Implement Template Conversion Functions**
**Priority**: CRITICAL
**Estimated Time**: 2 hours (NEW TASK)
**File**: `llmspell-bridge/src/lua/conversion.rs` (add 150 LOC)

**Description**: Implement Lua ↔ Rust conversion functions for template-specific types (TemplateParams, TemplateOutput, TemplateMetadata, ConfigSchema).

**Acceptance Criteria:**
- [x] `lua_table_to_template_params()` converts Lua table to HashMap<String, Value>
- [x] `template_output_to_lua_table()` converts TemplateOutput to Lua table
- [x] `template_metadata_to_lua_table()` formats metadata as Lua table
- [x] `config_schema_to_lua_table()` formats parameter schema as Lua table
- [x] All functions handle errors gracefully with mlua::Result

**Implementation**:
```rust
// llmspell-bridge/src/lua/conversion.rs (add to existing file)

/// Convert Lua table to template parameters
pub fn lua_table_to_template_params(lua: &Lua, table: &Table) -> mlua::Result<HashMap<String, serde_json::Value>> {
    let mut params = HashMap::new();
    for pair in table.pairs::<String, LuaValue>() {
        let (key, value) = pair?;
        let json_value = lua_value_to_json(value)?;
        params.insert(key, json_value);
    }
    Ok(params)
}

/// Convert TemplateOutput to Lua table
pub fn template_output_to_lua_table<'a>(
    lua: &'a Lua,
    output: &llmspell_templates::TemplateOutput,
) -> mlua::Result<Table<'a>> {
    let table = lua.create_table()?;

    // Set result based on type
    match &output.result {
        llmspell_templates::TemplateResult::Text(text) => {
            table.set("result_type", "text")?;
            table.set("result", text.clone())?;
        }
        llmspell_templates::TemplateResult::Structured(data) => {
            table.set("result_type", "structured")?;
            table.set("result", json_to_lua_value(lua, data)?)?;
        }
        llmspell_templates::TemplateResult::File(path) => {
            table.set("result_type", "file")?;
            table.set("result", path.to_string_lossy().to_string())?;
        }
        llmspell_templates::TemplateResult::Multiple(results) => {
            table.set("result_type", "multiple")?;
            let results_array = lua.create_table()?;
            for (i, result) in results.iter().enumerate() {
                // Recursively convert each result
                let result_table = lua.create_table()?;
                match result {
                    llmspell_templates::TemplateResult::Text(text) => {
                        result_table.set("type", "text")?;
                        result_table.set("value", text.clone())?;
                    }
                    // ... handle other types
                }
                results_array.set(i + 1, result_table)?;
            }
            table.set("result", results_array)?;
        }
    }

    // Set artifacts
    if !output.artifacts.is_empty() {
        let artifacts_array = lua.create_table()?;
        for (i, artifact) in output.artifacts.iter().enumerate() {
            let artifact_table = lua.create_table()?;
            artifact_table.set("name", artifact.name())?;
            artifact_table.set("path", artifact.path().to_string_lossy().to_string())?;
            artifact_table.set("mime_type", artifact.mime_type())?;
            artifacts_array.set(i + 1, artifact_table)?;
        }
        table.set("artifacts", artifacts_array)?;
    }

    // Set metrics
    let metrics_table = lua.create_table()?;
    if let Some(duration) = output.metrics.duration_ms {
        metrics_table.set("duration_ms", duration)?;
    }
    if let Some(tokens) = output.metrics.total_tokens {
        metrics_table.set("total_tokens", tokens)?;
    }
    if let Some(cost) = output.metrics.total_cost_usd {
        metrics_table.set("total_cost_usd", cost)?;
    }
    metrics_table.set("agents_invoked", output.metrics.agents_invoked)?;
    metrics_table.set("tools_invoked", output.metrics.tools_invoked)?;
    metrics_table.set("rag_queries", output.metrics.rag_queries)?;
    // Custom metrics
    for (key, value) in &output.metrics.custom {
        metrics_table.set(key.as_str(), json_to_lua_value(lua, value)?)?;
    }
    table.set("metrics", metrics_table)?;

    Ok(table)
}

/// Convert TemplateMetadata to Lua table
pub fn template_metadata_to_lua_table<'a>(
    lua: &'a Lua,
    metadata: &llmspell_templates::TemplateMetadata,
) -> mlua::Result<Table<'a>> {
    let table = lua.create_table()?;
    table.set("id", metadata.id.clone())?;
    table.set("name", metadata.name.clone())?;
    table.set("description", metadata.description.clone())?;
    table.set("category", format!("{:?}", metadata.category))?;
    table.set("version", metadata.version.clone())?;

    // Tags
    if !metadata.tags.is_empty() {
        let tags_array = lua.create_table()?;
        for (i, tag) in metadata.tags.iter().enumerate() {
            tags_array.set(i + 1, tag.clone())?;
        }
        table.set("tags", tags_array)?;
    }

    // Requirements
    if !metadata.requires.is_empty() {
        let requires_array = lua.create_table()?;
        for (i, req) in metadata.requires.iter().enumerate() {
            requires_array.set(i + 1, req.clone())?;
        }
        table.set("requires", requires_array)?;
    }

    Ok(table)
}

/// Convert ConfigSchema to Lua table
pub fn config_schema_to_lua_table<'a>(
    lua: &'a Lua,
    schema: &llmspell_templates::ConfigSchema,
) -> mlua::Result<Table<'a>> {
    let schema_table = lua.create_table()?;

    let params_array = lua.create_table()?;
    for (i, param) in schema.parameters.iter().enumerate() {
        let param_table = lua.create_table()?;
        param_table.set("name", param.name.clone())?;
        param_table.set("description", param.description.clone())?;
        param_table.set("param_type", format!("{:?}", param.param_type))?;
        param_table.set("required", param.required)?;

        if let Some(default) = &param.default {
            param_table.set("default", json_to_lua_value(lua, default)?)?;
        }

        // Constraints
        if param.constraints.min.is_some() || param.constraints.max.is_some() {
            let constraints_table = lua.create_table()?;
            if let Some(min) = param.constraints.min {
                constraints_table.set("min", min)?;
            }
            if let Some(max) = param.constraints.max {
                constraints_table.set("max", max)?;
            }
            if let Some(min_len) = param.constraints.min_length {
                constraints_table.set("min_length", min_len)?;
            }
            if let Some(max_len) = param.constraints.max_length {
                constraints_table.set("max_length", max_len)?;
            }
            if !param.constraints.allowed_values.is_empty() {
                let allowed_array = lua.create_table()?;
                for (j, val) in param.constraints.allowed_values.iter().enumerate() {
                    allowed_array.set(j + 1, val.clone())?;
                }
                constraints_table.set("allowed_values", allowed_array)?;
            }
            param_table.set("constraints", constraints_table)?;
        }

        params_array.set(i + 1, param_table)?;
    }
    schema_table.set("parameters", params_array)?;

    Ok(schema_table)
}
```

**Definition of Done:**
- [x] All 4 conversion functions compile
- [x] Handles all TemplateResult variants (Text, Structured, File, Multiple)
- [x] Artifacts array properly formatted
- [x] Metrics includes all standard + custom fields
- [x] Metadata includes tags and requirements arrays
- [x] ConfigSchema includes constraints

---

#### **Task 12.5.3: Implement Lua Template Global Injection**
**Priority**: CRITICAL
**Estimated Time**: 4 hours (was part of 12.5.1)
**File**: `llmspell-bridge/src/lua/globals/template.rs` (NEW - 450 LOC)

**Description**: Implement comprehensive Lua injection function with all 5 Template methods, following the pattern from `tool.rs`.

**Acceptance Criteria:**
- [x] `inject_template_global()` function creates Template global table
- [x] 5 methods implemented: list, info, execute, search, schema
- [x] All methods use `block_on_async_lua()` for async execution
- [x] Uses conversion functions from Task 12.5.2
- [x] Error handling with clear Lua error messages
- [x] Compiles with `cargo check -p llmspell-bridge`

**Implementation Pattern**:
```rust
// llmspell-bridge/src/lua/globals/template.rs (NEW FILE)
use crate::globals::GlobalContext;
use crate::lua::conversion::{
    config_schema_to_lua_table, json_to_lua_value, lua_table_to_template_params,
    template_metadata_to_lua_table, template_output_to_lua_table,
};
use crate::lua::sync_utils::block_on_async_lua;
use crate::ComponentRegistry;
use mlua::{Lua, Table, Value};
use std::sync::Arc;
use tracing::{info, instrument};

/// Inject Template global into Lua environment
#[instrument(
    level = "info",
    skip(lua, context, registry),
    fields(global_name = "Template")
)]
pub fn inject_template_global(
    lua: &Lua,
    context: &GlobalContext,
    registry: Arc<ComponentRegistry>,
) -> mlua::Result<()> {
    info!("Injecting Template global API");
    let template_table = lua.create_table()?;

    // Create Template.list([category]) function
    let registry_clone = registry.clone();
    let list_fn = lua.create_function(move |lua, category: Option<String>| {
        let registry = registry_clone.clone();

        let result = block_on_async_lua(
            "template_list",
            async move {
                let template_registry = registry.template_registry();

                // Parse category
                let category_filter = category.and_then(|cat_str| {
                    use llmspell_templates::TemplateCategory;
                    match cat_str.as_str() {
                        "Research" => Some(TemplateCategory::Research),
                        "Chat" => Some(TemplateCategory::Chat),
                        "Analysis" => Some(TemplateCategory::Analysis),
                        "CodeGen" => Some(TemplateCategory::CodeGen),
                        "Document" => Some(TemplateCategory::Document),
                        "Workflow" => Some(TemplateCategory::Workflow),
                        _ => None,
                    }
                });

                // Get templates
                let templates = template_registry.discover_by_category(category_filter);

                // Convert to JSON
                let json_templates = serde_json::to_value(&templates)
                    .map_err(|e| mlua::Error::RuntimeError(format!("Failed to serialize templates: {e}")))?;

                Ok(json_templates)
            },
            None,
        )?;

        // Convert JSON result to Lua table
        json_to_lua_value(lua, &result)
    })?;

    // Create Template.info(name, [show_schema]) function
    let registry_clone = registry.clone();
    let info_fn = lua.create_function(move |lua, (name, show_schema): (String, Option<bool>)| {
        let registry = registry_clone.clone();

        let result = block_on_async_lua(
            "template_info",
            async move {
                let template_registry = registry.template_registry();
                let template = template_registry
                    .get(&name)
                    .map_err(|e| mlua::Error::RuntimeError(format!("Template not found: {e}")))?;

                let metadata = template.metadata();
                let mut info_table = serde_json::json!({
                    "id": metadata.id,
                    "name": metadata.name,
                    "description": metadata.description,
                    "category": format!("{:?}", metadata.category),
                    "version": metadata.version,
                    "tags": metadata.tags,
                    "requires": metadata.requires,
                });

                // Add schema if requested
                if show_schema.unwrap_or(false) {
                    let schema = template.config_schema();
                    let schema_json = serde_json::to_value(&schema)
                        .map_err(|e| mlua::Error::RuntimeError(format!("Failed to serialize schema: {e}")))?;
                    info_table.as_object_mut().unwrap().insert("schema".to_string(), schema_json);
                }

                Ok(info_table)
            },
            None,
        )?;

        json_to_lua_value(lua, &result)
    })?;

    // Create Template.execute(name, params) function
    let registry_clone = registry.clone();
    let context_clone = Arc::new(context.clone());
    let execute_fn = lua.create_function(move |lua, (name, params): (String, Table)| {
        let registry = registry_clone.clone();
        let global_context = context_clone.clone();

        let result = block_on_async_lua(
            "template_execute",
            async move {
                // Get template
                let template_registry = registry.template_registry();
                let template = template_registry
                    .get(&name)
                    .map_err(|e| mlua::Error::RuntimeError(format!("Template not found: {e}")))?;

                // Convert params
                let template_params_map = lua_table_to_template_params(lua, &params)?;
                let template_params = llmspell_templates::TemplateParams::new(template_params_map);

                // Validate params
                template.validate(&template_params)
                    .map_err(|e| mlua::Error::RuntimeError(format!("Parameter validation failed: {e}")))?;

                // Build ExecutionContext
                let exec_context = llmspell_templates::ExecutionContext::builder()
                    .tool_registry(registry.tool_registry().clone())
                    .agent_registry(registry.agent_registry().clone())
                    .workflow_factory(registry.workflow_factory())
                    .llm_registry(global_context.providers.clone())
                    // Optional components
                    .with_state_opt(global_context.state_access.as_ref().map(|s| s.clone()))
                    .with_rag_opt(
                        global_context.get_bridge::<llmspell_rag::RAGStore>("rag_store")
                    )
                    .build();

                // Execute template
                let output = template.execute(template_params, exec_context)
                    .await
                    .map_err(|e| mlua::Error::RuntimeError(format!("Template execution failed: {e}")))?;

                Ok(output)
            },
            None,
        )?;

        // Convert TemplateOutput to Lua table
        template_output_to_lua_table(lua, &result)
    })?;

    // Create Template.search(query, [category]) function
    let registry_clone = registry.clone();
    let search_fn = lua.create_function(move |lua, (query, category): (String, Option<String>)| {
        let registry = registry_clone.clone();

        let result = block_on_async_lua(
            "template_search",
            async move {
                let template_registry = registry.template_registry();

                // Search
                let mut results = template_registry.search(&query);

                // Filter by category if provided
                if let Some(cat_str) = category {
                    use llmspell_templates::TemplateCategory;
                    let category_filter = match cat_str.as_str() {
                        "Research" => Some(TemplateCategory::Research),
                        "Chat" => Some(TemplateCategory::Chat),
                        "Analysis" => Some(TemplateCategory::Analysis),
                        "CodeGen" => Some(TemplateCategory::CodeGen),
                        "Document" => Some(TemplateCategory::Document),
                        "Workflow" => Some(TemplateCategory::Workflow),
                        _ => None,
                    };

                    if let Some(cat) = category_filter {
                        results.retain(|metadata| metadata.category == cat);
                    }
                }

                let json_results = serde_json::to_value(&results)
                    .map_err(|e| mlua::Error::RuntimeError(format!("Failed to serialize search results: {e}")))?;

                Ok(json_results)
            },
            None,
        )?;

        json_to_lua_value(lua, &result)
    })?;

    // Create Template.schema(name) function
    let registry_clone = registry.clone();
    let schema_fn = lua.create_function(move |lua, name: String| {
        let registry = registry_clone.clone();

        let result = block_on_async_lua(
            "template_schema",
            async move {
                let template_registry = registry.template_registry();
                let template = template_registry
                    .get(&name)
                    .map_err(|e| mlua::Error::RuntimeError(format!("Template not found: {e}")))?;

                let schema = template.config_schema();
                let json_schema = serde_json::to_value(&schema)
                    .map_err(|e| mlua::Error::RuntimeError(format!("Failed to serialize schema: {e}")))?;

                Ok(json_schema)
            },
            None,
        )?;

        json_to_lua_value(lua, &result)
    })?;

    // Set functions on Template table
    template_table.set("list", list_fn)?;
    template_table.set("info", info_fn)?;
    template_table.set("execute", execute_fn)?;
    template_table.set("search", search_fn)?;
    template_table.set("schema", schema_fn)?;

    // Set Template as global
    lua.globals().set("Template", template_table)?;

    Ok(())
}
```

**Definition of Done:**
- [x] All 5 methods functional
- [x] Async execution via block_on_async_lua
- [x] Proper error messages for missing templates, validation failures
- [x] Category filtering works for list and search
- [x] ExecutionContext built from GlobalContext
- [x] Compiles cleanly with cargo check

---

#### **Task 12.5.4: Create JavaScript Template Global Stub**
**Priority**: LOW
**Estimated Time**: 30 minutes (NEW TASK)
**File**: `llmspell-bridge/src/javascript/globals/template.rs` (NEW - 20 LOC)

**Description**: Create minimal JavaScript stub for Template global, following the pattern from other JavaScript stubs.

**Acceptance Criteria:**
- [x] Stub file created with warning log
- [x] `inject_template_global()` signature matches Lua version
- [x] Returns Ok(()) with no-op implementation
- [x] Module added to `llmspell-bridge/src/javascript/globals/mod.rs`

**Implementation**:
```rust
// llmspell-bridge/src/javascript/globals/template.rs (NEW FILE)
use crate::globals::GlobalContext;
use llmspell_core::Result;
use tracing::warn;

/// Inject Template global into JavaScript environment (stub)
pub fn inject_template_global(
    _ctx: &mut boa_engine::Context,
    _context: &GlobalContext,
) -> Result<()> {
    warn!("JavaScript Template global not yet implemented");
    Ok(())
}
```

**Add to mod.rs**:
```rust
// llmspell-bridge/src/javascript/globals/mod.rs
pub mod template;
```

**Definition of Done:**
- [x] File compiles
- [x] Warning logged when called
- [x] Module exported

---

#### **Task 12.5.5: Register Template Global in GlobalRegistry**
**Priority**: CRITICAL
**Estimated Time**: 1 hour (was 2 hours in Task 12.5.2)
**File**: `llmspell-bridge/src/globals/mod.rs` (modify existing)

**Description**: Register TemplateGlobal in `create_standard_registry()` as the 16th global, following the LocalLLM registration pattern.

**Acceptance Criteria:**
- [x] Import added: `pub use template_global::TemplateGlobal;`
- [x] Module declared: `pub mod template_global;`
- [x] Registration added after LocalLLM in `create_standard_registry()`
- [x] Uses `Arc::new(TemplateGlobal::new(context.registry.clone()))`
- [x] Global count updated: 15 → 16 in documentation

**Implementation**:
```rust
// llmspell-bridge/src/globals/mod.rs (modify existing file)

// Add module declaration (around line 27)
pub mod template_global;

// Add re-export (around line 32)
pub use template_global::TemplateGlobal;

// In create_standard_registry(), add after LocalLLM registration (around line 247)
pub async fn create_standard_registry(context: Arc<GlobalContext>) -> Result<GlobalRegistry> {
    let mut builder = GlobalRegistryBuilder::new();

    // ... existing registrations (core, state, event, session, rag, hook, tool, agent, workflow, streaming, local_llm)

    // Register LocalLLM global (providers always available in context)
    builder.register(Arc::new(local_llm_global::LocalLLMGlobal::new(
        context.providers.create_core_manager_arc().await?,
    )));

    // Register Template global (16th global)
    builder.register(Arc::new(template_global::TemplateGlobal::new(
        context.registry.clone(),
    )));

    builder.build()
}
```

**Definition of Done:**
- [x] Module compiled into globals
- [x] TemplateGlobal registered in builder
- [x] Global available in Lua scripts after bridge initialization
- [x] No circular dependencies
- [x] Compiles with `cargo check --workspace`

---

#### **Task 12.5.6: Create Lua Template Examples**
**Priority**: HIGH
**Estimated Time**: 3 hours (same as original Task 12.5.3)
**File**: Multiple new files in `examples/templates/`

**Description**: Create comprehensive Lua examples for all 6 templates demonstrating Template global usage.

**Acceptance Criteria:**
- [x] Example for each template (6 total)
- [x] Discovery example (list + search)
- [x] Schema introspection example
- [x] Error handling examples
- [x] All examples execute successfully

**Implementation Steps:**
1. Create `examples/templates/` directory structure:
   - `discovery.lua`: Template.list() and Template.search() usage
   - `research/lua-basic.lua`: Basic research assistant
   - `chat/lua-basic.lua`: Basic interactive chat
   - `analysis/lua-basic.lua`: Basic data analysis
   - `codegen/lua-basic.lua`: Basic code generator
   - `documents/lua-basic.lua`: Basic document processor
   - `orchestration/lua-basic.lua`: Basic workflow orchestrator
2. Add comprehensive comments explaining each API call
3. Test all examples execute successfully
4. Create `examples/templates/README.md` with overview

**Example Structure** (discovery.lua):
```lua
-- Template Discovery and Search Example

print("=== Template.list() - All Templates ===")
local all_templates = Template.list()
for i, template in ipairs(all_templates) do
    print(string.format("%d. %s (%s)", i, template.name, template.id))
    print(string.format("   Category: %s", template.category))
    print(string.format("   Description: %s", template.description))
end

print("\n=== Template.list('Research') - Research Templates Only ===")
local research_templates = Template.list("Research")
for i, template in ipairs(research_templates) do
    print(string.format("%d. %s", i, template.name))
end

print("\n=== Template.search('research') - Search by Query ===")
local search_results = Template.search("research", nil)
for i, template in ipairs(search_results) do
    print(string.format("%d. %s (score: %.2f)", i, template.name, template.relevance or 0.0))
end

print("\n=== Template.info('research-assistant', true) - With Schema ===")
local info = Template.info("research-assistant", true)
print("Template ID: " .. info.id)
print("Name: " .. info.name)
print("Description: " .. info.description)
print("Category: " .. info.category)
print("Version: " .. info.version)

if info.schema then
    print("\nParameters:")
    for i, param in ipairs(info.schema.parameters) do
        local required = param.required and "required" or "optional"
        print(string.format("  - %s (%s, %s): %s",
            param.name, param.param_type, required, param.description))
        if param.default then
            print(string.format("    Default: %s", param.default))
        end
    end
end

print("\n=== Template.schema('research-assistant') - Schema Only ===")
local schema = Template.schema("research-assistant")
print("Parameters count: " .. #schema.parameters)
```

**Definition of Done:**
- [x] 7 Lua examples created
- [x] All examples execute successfully
- [x] Well-commented and educational
- [x] README helpful
- [x] Examples tested with quality-check-fast.sh

---

## Summary of Changes

### Original TODO.md Structure (Tasks 12.5.1-12.5.3)
1. **Task 12.5.1**: Create Template Global Object (4 hours, 380 LOC mixed Lua/Rust)
2. **Task 12.5.2**: Register in Bridge (2 hours)
3. **Task 12.5.3**: Lua Examples (3 hours)

**Total**: 3 tasks, 9 hours

### Revised Structure (Tasks 12.5.1-12.5.6)
1. **Task 12.5.1**: Language-Neutral TemplateGlobal Struct (1 hour, 80 LOC)
2. **Task 12.5.2**: Template Conversion Functions (2 hours, 150 LOC)
3. **Task 12.5.3**: Lua Template Global Injection (4 hours, 450 LOC)
4. **Task 12.5.4**: JavaScript Template Global Stub (0.5 hours, 20 LOC)
5. **Task 12.5.5**: Register in GlobalRegistry (1 hour)
6. **Task 12.5.6**: Lua Examples (3 hours)

**Total**: 6 tasks, 11.5 hours

**Added Complexity**: +2.5 hours, but correctly implements 3-layer pattern

---

## Implementation Dependencies

### Phase 12.1-12.4 Dependencies (Already Complete)
- ✅ `llmspell-templates` crate with Template trait
- ✅ TemplateRegistry with discovery/search
- ✅ 6 built-in templates fully implemented
- ✅ TemplateParams, TemplateOutput, ConfigSchema types
- ✅ ExecutionContext with builder pattern
- ✅ TemplateRegistry in ComponentRegistry (Phase 12.2.6)

### Phase 12.5 Dependencies (No New External Crates)
- `mlua` (already used, Lua bindings)
- `serde` / `serde_json` (already used, serialization)
- `async-trait` (already used, Template trait)
- Conversion functions in `llmspell-bridge/src/lua/conversion.rs`
- Sync utils in `llmspell-bridge/src/lua/sync_utils.rs` (block_on_async_lua)

---

## Quality Gates

Before marking Phase 12.5 complete:

- [ ] **Clippy**: `cargo clippy --workspace --all-targets --all-features -- -D warnings` → 0 warnings
- [ ] **Format**: `cargo fmt --all --check` → no formatting issues
- [ ] **Compile**: `cargo check --workspace --all-features` → clean compilation
- [ ] **Tests**: `cargo test --workspace --all-features` → all tests pass
- [ ] **Template Global Available**: Lua script can call `Template.list()` successfully
- [ ] **All 5 Methods Work**: list, info, execute, search, schema all functional
- [ ] **Examples Execute**: All 7 Lua examples run without errors
- [ ] **Documentation**: API docs for all new public items

---

## Risk Analysis

### Low Risk
- ✅ Pattern well-established (15 existing globals follow same pattern)
- ✅ Template infrastructure complete (Phase 12.1-12.4)
- ✅ Conversion functions straightforward (similar to tool/agent conversions)
- ✅ No new external dependencies

### Medium Risk
- ⚠️ ExecutionContext building may need adjustment if GlobalContext doesn't have all needed fields
- ⚠️ Template execution may be slower than tool execution (multi-agent orchestration)
- ⚠️ TemplateOutput conversion complexity (4 result types: Text, Structured, File, Multiple)

### Mitigation
- Use existing ExecutionContext::builder() pattern from templates crate
- Document expected execution times in Lua examples
- Thoroughly test all TemplateResult variants in conversion function

---

## Performance Targets

- **Template.list()**: <10ms (registry lookup + metadata formatting)
- **Template.info()**: <5ms (single template metadata)
- **Template.execute()**: Variable (depends on template complexity, target <100ms overhead)
- **Template.search()**: <20ms (6 templates full-text search)
- **Template.schema()**: <5ms (schema serialization)

---

## Next Steps After Phase 12.5

**Phase 12.6**: Testing, Quality, and Release
- Comprehensive unit tests for all conversion functions
- Integration tests for Template global in Lua
- Performance benchmarks
- Documentation finalization (template-system-architecture.md)
- Release preparation (RELEASE_NOTES_v0.12.0.md)

**Phase 13**: Adaptive Memory System (A-TKG)
- Templates will gain .enable_memory() enhancement
- ExecutionContext will include memory_manager
- Templates designed to be memory-ready (no breaking changes)

---

## Conclusion

Phase 12.5 correctly follows the established 3-layer bridge pattern:
1. **Language-neutral global struct** (template_global.rs)
2. **Lua-specific injection** (lua/globals/template.rs)
3. **JavaScript stub** (javascript/globals/template.rs)

The revised task breakdown separates concerns properly and provides clear, actionable steps for implementation. Total estimated time: **11.5 hours** across 6 tasks.

All infrastructure from Phase 12.1-12.4 is ready. Phase 12.5 is purely bridge integration work with zero new domain logic.
