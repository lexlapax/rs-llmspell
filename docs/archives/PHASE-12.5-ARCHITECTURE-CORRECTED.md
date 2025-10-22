# Phase 12.5 Architecture - CORRECTED
**Date**: 2025-10-13
**Status**: Corrected after ultrathink re-analysis

## Critical Correction

**PREVIOUS ANALYSIS WAS WRONG**: I initially thought templates would follow ToolGlobal pattern (no bridge).

**CORRECT PATTERN**: Templates must follow AgentGlobal/WorkflowGlobal pattern (WITH bridge).

## Pattern Discovery

### Tool Pattern (Simple - NO Bridge)
```rust
// llmspell-bridge/src/globals/tool_global.rs
pub struct ToolGlobal {
    registry: Arc<ComponentRegistry>,  // Direct registry access
}

// Lua injection takes registry
inject_tool_global(lua, context, registry: Arc<ComponentRegistry>)
```

### Agent/Workflow Pattern (Complex - WITH Bridge)
```rust
// llmspell-bridge/src/globals/agent_global.rs
pub struct AgentGlobal {
    registry: Arc<ComponentRegistry>,
    providers: Arc<ProviderManager>,
    bridge: Arc<AgentBridge>,  // <-- WRAPS BRIDGE!
}

impl GlobalObject for AgentGlobal {
    fn inject_lua(&self, lua, context) -> Result {
        // Passes BRIDGE to Lua, not registry
        crate::lua::globals::agent::inject_agent_global(lua, context, self.bridge.clone())
    }
}

// llmspell-bridge/src/agent_bridge.rs (2000+ lines)
pub struct AgentBridge {
    discovery: Arc<AgentDiscovery>,
    registry: Arc<ComponentRegistry>,
    // ... 20+ fields for complex state management
}

impl AgentBridge {
    pub async fn create_agent(&self, config: AgentConfig) -> Result<String> { ... }
    pub async fn execute_agent(&self, id: &str, input: AgentInput) -> Result<AgentOutput> { ... }
    // ... 50+ methods
}
```

### Workflow Pattern (Same - WITH Bridge)
```rust
// llmspell-bridge/src/globals/workflow_global.rs
pub struct WorkflowGlobal {
    registry: Arc<ComponentRegistry>,
    bridge: Arc<WorkflowBridge>,  // <-- WRAPS BRIDGE!
}

// llmspell-bridge/src/workflows.rs (900+ lines)
pub struct WorkflowBridge {
    discovery: Arc<WorkflowDiscovery>,
    registry: Arc<ComponentRegistry>,
    state_manager: Option<Arc<StateManager>>,
    // ... complex workflow management
}
```

## Why Templates Need Bridge

**Template Complexity** (like Agent, not like Tool):
- CodeGeneratorTemplate: 860 lines, 4-phase orchestration (spec → impl → test → lint)
- ResearchAssistantTemplate: 801 lines, 4-phase research workflow
- Parameter validation with ConfigSchema constraints
- ExecutionContext building with state/session/RAG
- Cost estimation logic
- Artifact management
- Multi-agent coordination

**Bridge Business Logic Needed**:
1. Template instantiation with parameter validation
2. ExecutionContext building from GlobalContext
3. State/session/RAG integration
4. Template discovery with category filtering
5. Search with full-text matching
6. Cost estimation
7. Schema generation

## Corrected 4-Layer Architecture

### Layer 0: Business Logic Bridge
**File**: `llmspell-bridge/src/template_bridge.rs` (NEW - 400-600 LOC)

```rust
use llmspell_templates::{Template, TemplateRegistry, TemplateMetadata, TemplateOutput, TemplateParams, ConfigSchema};
use std::sync::Arc;

/// Bridge between scripts and template system
/// Provides business logic layer for template operations
pub struct TemplateBridge {
    /// Template registry (from llmspell-templates)
    template_registry: Arc<TemplateRegistry>,
    /// Component registry for ExecutionContext building
    registry: Arc<ComponentRegistry>,
    /// Optional state manager for stateful templates
    state_manager: Option<Arc<llmspell_kernel::state::StateManager>>,
    /// Optional session manager for session-based templates
    session_manager: Option<Arc<llmspell_kernel::sessions::manager::SessionManager>>,
}

impl TemplateBridge {
    /// Create new template bridge
    pub fn new(
        template_registry: Arc<TemplateRegistry>,
        registry: Arc<ComponentRegistry>,
    ) -> Self {
        Self {
            template_registry,
            registry,
            state_manager: None,
            session_manager: None,
        }
    }

    /// Create with state manager support
    pub fn with_state_manager(
        template_registry: Arc<TemplateRegistry>,
        registry: Arc<ComponentRegistry>,
        state_manager: Arc<llmspell_kernel::state::StateManager>,
    ) -> Self {
        Self {
            template_registry,
            registry,
            state_manager: Some(state_manager),
            session_manager: None,
        }
    }

    /// List templates by optional category
    pub fn list_templates(&self, category: Option<llmspell_templates::TemplateCategory>) -> Vec<TemplateMetadata> {
        self.template_registry.discover_by_category(category)
    }

    /// Get template info with optional schema
    pub fn get_template_info(&self, name: &str, include_schema: bool) -> Result<TemplateInfo> {
        let template = self.template_registry.get(name)?;
        let metadata = template.metadata().clone();
        let schema = if include_schema {
            Some(template.config_schema())
        } else {
            None
        };
        Ok(TemplateInfo { metadata, schema })
    }

    /// Execute template with parameters
    pub async fn execute_template(
        &self,
        name: &str,
        params: TemplateParams,
    ) -> Result<TemplateOutput> {
        // Get template
        let template = self.template_registry.get(name)?;

        // Validate parameters
        template.validate(&params)?;

        // Build ExecutionContext from available infrastructure
        let mut context_builder = llmspell_templates::ExecutionContext::builder()
            .tool_registry(self.registry.tool_registry().clone())
            .agent_registry(self.registry.agent_registry().clone())
            .workflow_factory(self.registry.workflow_factory())
            .llm_registry(/* from providers */);

        // Add optional components
        if let Some(state_mgr) = &self.state_manager {
            let state_adapter = Arc::new(crate::state_adapter::NoScopeStateAdapter::new(state_mgr.clone()));
            context_builder = context_builder.with_state(state_adapter);
        }

        if let Some(session_mgr) = &self.session_manager {
            context_builder = context_builder.with_session_manager(session_mgr.clone());
        }

        let exec_context = context_builder.build();

        // Execute template
        template.execute(params, exec_context).await
    }

    /// Search templates by query and optional category
    pub fn search_templates(
        &self,
        query: &str,
        category: Option<llmspell_templates::TemplateCategory>,
    ) -> Vec<TemplateMetadata> {
        let mut results = self.template_registry.search(query);

        // Filter by category if provided
        if let Some(cat) = category {
            results.retain(|metadata| metadata.category == cat);
        }

        results
    }

    /// Get template parameter schema
    pub fn get_template_schema(&self, name: &str) -> Result<ConfigSchema> {
        let template = self.template_registry.get(name)?;
        Ok(template.config_schema())
    }

    /// Estimate template execution cost
    pub async fn estimate_cost(&self, name: &str, params: &TemplateParams) -> Result<Option<CostEstimate>> {
        let template = self.template_registry.get(name)?;
        Ok(template.estimate_cost(params).await)
    }
}

/// Template info with optional schema
pub struct TemplateInfo {
    pub metadata: TemplateMetadata,
    pub schema: Option<ConfigSchema>,
}
```

**Estimate**: 400-600 lines (similar to WorkflowBridge complexity)

### Layer 1: Language-Neutral Global Wrapper
**File**: `llmspell-bridge/src/globals/template_global.rs` (NEW - 100 LOC)

```rust
use super::types::{GlobalContext, GlobalMetadata, GlobalObject};
use crate::template_bridge::TemplateBridge;
use llmspell_core::Result;
use std::sync::Arc;

/// Template global object for script engines
pub struct TemplateGlobal {
    bridge: Arc<TemplateBridge>,
}

impl TemplateGlobal {
    /// Create a new Template global
    pub fn new(bridge: Arc<TemplateBridge>) -> Self {
        Self { bridge }
    }

    /// Get the template bridge
    pub const fn bridge(&self) -> &Arc<TemplateBridge> {
        &self.bridge
    }
}

impl GlobalObject for TemplateGlobal {
    fn metadata(&self) -> GlobalMetadata {
        GlobalMetadata {
            name: "Template".to_string(),
            description: "Template discovery, inspection, and execution".to_string(),
            dependencies: vec![],
            required: true,
            version: "1.0.0".to_string(),
        }
    }

    #[cfg(feature = "lua")]
    fn inject_lua(&self, lua: &mlua::Lua, context: &GlobalContext) -> Result<()> {
        crate::lua::globals::template::inject_template_global(lua, context, self.bridge.clone())
            .map_err(|e| llmspell_core::LLMSpellError::Component {
                message: format!("Failed to inject Template global: {e}"),
                source: None,
            })
    }

    #[cfg(feature = "javascript")]
    fn inject_javascript(
        &self,
        ctx: &mut boa_engine::Context,
        context: &GlobalContext,
    ) -> Result<()> {
        crate::javascript::globals::template::inject_template_global(ctx, context)
            .map_err(|e| llmspell_core::LLMSpellError::Component {
                message: format!("Failed to inject Template global for JavaScript: {e}"),
                source: None,
            })
    }
}
```

### Layer 2: Lua Injection Function
**File**: `llmspell-bridge/src/lua/globals/template.rs` (NEW - 450 LOC)

```rust
use crate::globals::GlobalContext;
use crate::lua::conversion::{/* conversion functions */};
use crate::lua::sync_utils::block_on_async_lua;
use crate::template_bridge::TemplateBridge;
use mlua::{Lua, Table};
use std::sync::Arc;

/// Inject Template global into Lua environment
pub fn inject_template_global(
    lua: &Lua,
    _context: &GlobalContext,
    bridge: Arc<TemplateBridge>,  // <-- Takes BRIDGE not registry!
) -> mlua::Result<()> {
    let template_table = lua.create_table()?;

    // Template.list([category])
    let bridge_clone = bridge.clone();
    let list_fn = lua.create_function(move |lua, category: Option<String>| {
        let bridge = bridge_clone.clone();
        let result = block_on_async_lua(
            "template_list",
            async move {
                let category_enum = /* parse category string */;
                let templates = bridge.list_templates(category_enum);
                Ok(templates)
            },
            None,
        )?;
        // Convert to Lua table
        Ok(result)
    })?;

    // Template.info(name, [show_schema])
    let bridge_clone = bridge.clone();
    let info_fn = lua.create_function(move |lua, (name, show_schema): (String, Option<bool>)| {
        let bridge = bridge_clone.clone();
        let result = block_on_async_lua(
            "template_info",
            async move {
                let info = bridge.get_template_info(&name, show_schema.unwrap_or(false))?;
                Ok(info)
            },
            None,
        )?;
        // Convert to Lua table
        Ok(result)
    })?;

    // Template.execute(name, params)
    let bridge_clone = bridge.clone();
    let execute_fn = lua.create_function(move |lua, (name, params): (String, Table)| {
        let bridge = bridge_clone.clone();
        let result = block_on_async_lua(
            "template_execute",
            async move {
                let template_params = /* convert Lua table to TemplateParams */;
                let output = bridge.execute_template(&name, template_params).await?;
                Ok(output)
            },
            None,
        )?;
        // Convert TemplateOutput to Lua table
        Ok(result)
    })?;

    // Template.search(query, [category])
    // Template.schema(name)

    template_table.set("list", list_fn)?;
    template_table.set("info", info_fn)?;
    template_table.set("execute", execute_fn)?;
    // ... set other methods

    lua.globals().set("Template", template_table)?;
    Ok(())
}
```

### Layer 3: JavaScript Stub
**File**: `llmspell-bridge/src/javascript/globals/template.rs` (NEW - 20 LOC)

```rust
pub fn inject_template_global(
    _ctx: &mut boa_engine::Context,
    _context: &GlobalContext,
) -> llmspell_core::Result<()> {
    warn!("JavaScript Template global not yet implemented");
    Ok(())
}
```

## Revised Phase 12.5 Tasks

### Task 12.5.1: Create TemplateBridge (NEW)
**Time**: 4 hours
**File**: `llmspell-bridge/src/template_bridge.rs` (400-600 LOC)
- Business logic for template operations
- Similar complexity to WorkflowBridge
- Methods: list, get_info, execute, search, get_schema, estimate_cost

### Task 12.5.2: Create TemplateGlobal
**Time**: 1 hour
**File**: `llmspell-bridge/src/globals/template_global.rs` (100 LOC)
- Wraps `Arc<TemplateBridge>`
- Implements GlobalObject trait
- Passes bridge to Lua/JS injection

### Task 12.5.3: Implement Conversion Functions
**Time**: 2 hours
**File**: `llmspell-bridge/src/lua/conversion.rs` (+150 LOC)
- template_params_to_lua, lua_to_template_params
- template_output_to_lua, template_metadata_to_lua
- config_schema_to_lua

### Task 12.5.4: Implement Lua Injection
**Time**: 4 hours
**File**: `llmspell-bridge/src/lua/globals/template.rs` (450 LOC)
- inject_template_global takes Arc<TemplateBridge>
- 5 methods: list, info, execute, search, schema

### Task 12.5.5: JavaScript Stub
**Time**: 0.5 hours
**File**: `llmspell-bridge/src/javascript/globals/template.rs` (20 LOC)

### Task 12.5.6: Register TemplateGlobal
**Time**: 1 hour
**File**: `llmspell-bridge/src/globals/mod.rs` (modifications)
- Create TemplateBridge instance
- Wrap in TemplateGlobal
- Register in create_standard_registry()

### Task 12.5.7: Lua Examples
**Time**: 3 hours
**Files**: `examples/templates/*.lua`

**Total**: 7 tasks, 15.5 hours

## Key Differences from Previous Analysis

| Aspect | WRONG Analysis | CORRECT Analysis |
|--------|---------------|------------------|
| **Pattern** | ToolGlobal (no bridge) | AgentGlobal/WorkflowGlobal (with bridge) |
| **TemplateGlobal wraps** | `Arc<ComponentRegistry>` | `Arc<TemplateBridge>` |
| **Lua injection takes** | `Arc<ComponentRegistry>` | `Arc<TemplateBridge>` |
| **Business logic** | In Lua injection | In TemplateBridge |
| **File structure** | 2 new files | 3 new files (+ bridge) |
| **Total LOC** | ~650 | ~1070 (includes bridge) |
| **Time estimate** | 11.5 hours | 15.5 hours |

## Why This Matters

**With Bridge Pattern**:
- ✅ Business logic in Rust (type-safe, testable)
- ✅ Lua layer is thin wrapper (no complex async logic)
- ✅ Consistent with Agent/Workflow patterns
- ✅ Can add JavaScript support easily (reuses bridge)
- ✅ ExecutionContext building centralized in bridge
- ✅ State/session integration handled properly

**Without Bridge (WRONG)**:
- ❌ Business logic scattered in Lua injection
- ❌ Hard to test (mlua tests complex)
- ❌ Inconsistent with existing patterns
- ❌ JavaScript implementation duplicates logic
- ❌ ExecutionContext building in wrong layer

## Conclusion

Templates ARE complex enough to need a bridge. The corrected architecture follows the established Agent/Workflow pattern:

1. **TemplateBridge** (src/template_bridge.rs) - Business logic
2. **TemplateGlobal** (src/globals/template_global.rs) - Wraps bridge
3. **Lua injection** (src/lua/globals/template.rs) - Calls bridge methods
4. **JavaScript stub** (src/javascript/globals/template.rs) - Placeholder

This is the CORRECT architecture for Phase 12.5.
