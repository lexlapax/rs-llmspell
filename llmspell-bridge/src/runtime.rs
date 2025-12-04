//! ABOUTME: Language-agnostic script runtime using `ScriptEngineBridge` abstraction
//! ABOUTME: Central execution orchestrator supporting multiple script engines

use crate::{
    engine::{ScriptEngineBridge, ScriptOutput, ScriptStream},
    providers::ProviderManager,
    registry::ComponentRegistry,
};

#[cfg(any(feature = "lua", feature = "javascript"))]
use crate::engine::EngineFactory;
#[cfg(any(feature = "lua", feature = "javascript"))]
use crate::tools::register_all_tools;

#[cfg(feature = "lua")]
use crate::engine::LuaConfig;

#[cfg(feature = "javascript")]
use crate::engine::JSConfig;
use async_trait::async_trait;
use llmspell_config::LLMSpellConfig;
use llmspell_core::error::LLMSpellError;
use llmspell_core::traits::component_lookup::ComponentLookup;
use llmspell_core::traits::debug_context::DebugContext;
use llmspell_core::traits::script_executor::{
    ScriptExecutionMetadata, ScriptExecutionOutput, ScriptExecutor,
};
use std::collections::HashMap;
use std::fmt::Write;
use std::sync::{Arc, RwLock};
use std::time::Instant;
use tracing::{debug, info, instrument};

/// Central script runtime that uses `ScriptEngineBridge` abstraction
///
/// The `ScriptRuntime` is the main entry point for executing scripts in `LLMSpell`.
/// It provides a language-agnostic interface that can work with multiple script
/// engines (Lua, JavaScript, Python, etc.) through the `ScriptEngineBridge` trait.
///
/// # Examples
///
/// ## Basic Script Execution
///
/// ```rust,no_run
/// use llmspell_bridge::ScriptRuntime;
/// use llmspell_config::LLMSpellConfig;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// // Create a runtime with default configuration
/// let config = LLMSpellConfig::default();
/// let runtime = ScriptRuntime::new(config).await?;
///
/// // Execute a simple Lua script
/// let output = runtime.execute_script("return 42").await?;
/// println!("Result: {:?}", output.output);
/// # Ok(())
/// # }
/// ```
///
/// ## Working with Agents (Placeholder - Phase 2)
///
/// ```rust,no_run
/// use llmspell_bridge::ScriptRuntime;
/// use llmspell_config::LLMSpellConfig;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let runtime = ScriptRuntime::new(LLMSpellConfig::default()).await?;
///
/// let script = r#"
///     -- Create an agent (placeholder functionality)
///     local agent = Agent.create({
///         name = "assistant",
///         system_prompt = "You are a helpful assistant"
///     })
///     
///     -- Execute the agent (returns placeholder response)
///     local response = agent:execute("Hello!")
///     return response.text
/// "#;
///
/// let output = runtime.execute_script(script).await?;
/// # Ok(())
/// # }
/// ```
///
/// ## Streaming Execution
///
/// ```rust,no_run
/// use llmspell_bridge::ScriptRuntime;
/// use llmspell_config::LLMSpellConfig;
/// use futures::StreamExt;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let runtime = ScriptRuntime::new(LLMSpellConfig::default()).await?;
///
/// // Check if streaming is supported
/// if runtime.supports_streaming() {
///     let mut stream = runtime.execute_script_streaming("return 'streaming output'").await?;
///     
///     // Process chunks as they arrive
///     while let Some(chunk) = stream.stream.next().await {
///         let chunk = chunk?;
///         println!("Received chunk: {:?}", chunk);
///     }
/// }
/// # Ok(())
/// # }
/// ```
///
/// # Dual-Layer Registry Architecture
///
/// `ScriptRuntime` maintains two parallel registry layers for different purposes.
/// This architecture was introduced in Phase 12.7.1 to fix template execution
/// infrastructure gaps while maintaining optimal performance for script access.
///
/// ## Layer 1: Script Access (`ComponentRegistry`)
///
/// - **Purpose**: Fast tool/agent/workflow lookups for Lua/JavaScript scripts
/// - **Implementation**: Lightweight `HashMap<String, Arc<dyn Trait>>` pattern
/// - **Used by**: Script engines via bridge APIs (`Tool.execute()` in Lua)
/// - **Size**: 266 lines of code, optimized for speed
/// - **Features**: Simple name→component mapping, O(1) lookup
///
/// ## Layer 2: Infrastructure (`ToolRegistry` + `FactoryRegistry` + `WorkflowFactory`)
///
/// - **Purpose**: Template execution with discovery, validation, hooks, metrics
/// - **Implementation**: Full-featured registries with caching, indexing, categorization
/// - **Used by**: Template system via `ExecutionContext`
/// - **Size**: 1571 lines (`ToolRegistry` alone), comprehensive features
/// - **Features**: Discovery by category, validation hooks, execution metrics, dependency tracking
///
/// ## Why Both Layers Are Necessary
///
/// The two layers cannot be merged or converted between each other because:
///
/// 1. **Different Data Structures**:
///    - `ComponentRegistry`: Simple `HashMap` for O(1) lookups
///    - `ToolRegistry`: Complex indexes, hooks, validation chains
///
/// 2. **Different Use Cases**:
///    - Scripts need: Fast `get_tool("calculator")` → execute
///    - Templates need: Discovery, validation, `list_tools_by_category()`, metrics
///
/// 3. **Performance vs Features Trade-off**:
///    - Scripts require minimal overhead (<1ms lookup)
///    - Templates require comprehensive infrastructure (hooks, validation, discovery)
///
/// 4. **Memory Cost is Minimal**:
///    - Both layers hold `Arc` references to the same tool instances
///    - Only the index structures are duplicated (~few KB)
///
/// ## Dual-Registration Pattern
///
/// Tools are registered to both layers simultaneously during runtime initialization:
///
/// ```rust,ignore
/// // Step 1: Create both registries
/// let tool_registry = Arc::new(ToolRegistry::new());         // Infrastructure
/// let component_registry = Arc::new(ComponentRegistry::new()); // Script access
///
/// // Step 2: Register tools to BOTH (dual-registration)
/// let calculator = CalculatorTool::new();
/// tool_registry.register("calculator", calculator.clone()).await?;
/// component_registry.register_tool("calculator", Arc::new(calculator))?;
/// ```
///
/// This pattern ensures:
/// - Scripts get fast `HashMap` lookups via `ComponentRegistry`
/// - Templates get full infrastructure via `ToolRegistry`
/// - Both share the same tool instances (`Arc`), no memory waste
///
/// ## Data Flow
///
/// ```text
/// ┌─────────────────────────────────────────────┐
/// │ CLI Command / User Script                   │
/// └────────────┬────────────────────────────────┘
///              │
///              ├──► Scripts (Lua/JS)
///              │    └──► ComponentRegistry (HashMap)
///              │         └──► Tool.execute() [Fast path]
///              │
///              └──► Templates
///                   └──► ExecutionContext
///                        ├──► ToolRegistry (full-featured)
///                        ├──► AgentRegistry (factories)
///                        ├──► WorkflowFactory (creation)
///                        └──► ProviderManager (LLMs)
/// ```
///
/// ## Historical Context (Phase 12.7.1)
///
/// **Problem**: Template execution failed with "`tool_registry` is required" error
/// because `ExecutionContext::builder()` expected 4 infrastructure components
/// (`ToolRegistry`, `AgentRegistry`, `WorkflowFactory`, `ProviderManager`) but
/// `ScriptRuntime` only had `ComponentRegistry`.
///
/// **Root Cause Analysis**: `ComponentRegistry` (266-line `HashMap` wrapper) serves
/// a fundamentally different purpose than `ToolRegistry` (1571-line infrastructure).
/// They cannot be converted or merged without losing critical features.
///
/// **Solution**: Add infrastructure registries alongside `ComponentRegistry`,
/// implement dual-registration for all tools, wire 4 components into
/// `ExecutionContext` builder. See TODO.md Phase 12.7.1 for 180+ line analysis.
///
/// This is not a temporary workaround but the correct architectural design,
/// following the same pattern as `provider_manager` which also exists separately
/// from `ComponentRegistry`.
pub struct ScriptRuntime {
    /// Language-agnostic script engine
    engine: Box<dyn ScriptEngineBridge>,

    /// Component registry for agents, tools, workflows (**script access layer** - Layer 1)
    ///
    /// Lightweight `HashMap`-based registry providing O(1) lookups for script engines.
    /// Used by: Lua/JS scripts via `Tool.execute()`, `Agent.create()`, `Workflow.run()`
    ///
    /// **Not used by templates** - templates use the infrastructure registries below.
    registry: Arc<ComponentRegistry>,

    /// Provider manager for LLM access (infrastructure layer)
    ///
    /// Shared by both scripts and templates. Provides access to configured LLM providers
    /// (`OpenAI`, Anthropic, Ollama, etc.)
    provider_manager: Arc<ProviderManager>,

    /// Tool registry for template infrastructure (**infrastructure layer** - Layer 2)
    ///
    /// Full-featured registry with hooks, discovery, validation, and metrics.
    /// Separate from `ComponentRegistry` which serves script access.
    /// Used by: Template system via `ExecutionContext`, CLI discovery commands.
    ///
    /// Tools exist in BOTH this registry AND `ComponentRegistry` (dual-registration).
    tool_registry: Arc<llmspell_tools::ToolRegistry>,

    /// Agent factory registry for template infrastructure (**infrastructure layer** - Layer 2)
    ///
    /// Provides agent creation and discovery capabilities for templates.
    /// Used by: Template system via `ExecutionContext` for dynamic agent instantiation.
    agent_registry: Arc<llmspell_agents::FactoryRegistry>,

    /// Workflow factory for template infrastructure (**infrastructure layer** - Layer 2)
    ///
    /// Provides workflow creation capabilities for templates.
    /// Used by: Template system via `ExecutionContext` for workflow orchestration.
    workflow_factory: Arc<dyn llmspell_workflows::WorkflowFactory>,

    /// State manager for persistent state (Phase 13b.16.2)
    ///
    /// Created from `Infrastructure` module during initialization.
    /// Provides persistent state operations for session management.
    #[allow(dead_code)]
    // Used by SessionManager infrastructure, not accessed directly in runtime.rs
    state_manager: Arc<llmspell_kernel::state::StateManager>,

    /// Session manager for template infrastructure (Phase 13b.16.2)
    ///
    /// Created from `Infrastructure` module during initialization.
    /// Direct ownership replaces `Arc<RwLock<Option<...>>>` pattern.
    ///
    /// **Why separate from `ComponentRegistry`:**
    /// - `SessionManager` is kernel-specific infrastructure (lifecycle, persistence, hooks)
    /// - Templates need full session management (create, save, restore, artifacts)
    /// - Scripts don't directly access sessions (use state instead)
    session_manager: Arc<llmspell_kernel::sessions::SessionManager>,

    /// RAG (Retrieval-Augmented Generation) infrastructure for template execution (Phase 13b.16.2)
    ///
    /// Created from `Infrastructure` module if `config.rag.enabled`.
    /// Direct ownership replaces `Arc<RwLock<Option<...>>>` pattern.
    /// Optional to support configurations without RAG.
    ///
    /// **Why needed:**
    /// - research-assistant template requires RAG for document ingestion and synthesis
    /// - Multi-tenant vector storage for knowledge base operations
    /// - Templates need RAG access via `ExecutionContext`
    rag: Option<Arc<llmspell_rag::multi_tenant_integration::MultiTenantRAG>>,

    /// Memory manager for adaptive memory system (Phase 13b.16.2)
    ///
    /// Created from `Infrastructure` module if `config.runtime.memory.enabled`.
    /// Direct ownership replaces `Arc<RwLock<Option<...>>>` pattern.
    /// Optional to support configurations without memory.
    memory_manager: Option<Arc<dyn llmspell_memory::MemoryManager>>,

    /// Execution context
    execution_context: Arc<RwLock<crate::engine::ExecutionContext>>,
    /// Debug context for debugging support (uses interior mutability)
    debug_context: Arc<RwLock<Option<Arc<dyn DebugContext>>>>,
    /// Runtime configuration
    _config: LLMSpellConfig,
}

impl ScriptRuntime {
    /// Create script runtime from configuration (Phase 13b.16.2 - Engine-Agnostic API)
    ///
    /// Uses `config.default_engine` to determine which engine to initialize.
    /// This is the primary entry point for engine-agnostic runtime creation.
    ///
    /// Creates ALL infrastructure internally via `Infrastructure::from_config()`:
    /// - `ProviderManager`
    /// - `StateManager`
    /// - `SessionManager`
    /// - RAG (if `config.rag.enabled`)
    /// - `MemoryManager` (if `config.runtime.memory.enabled`)
    /// - `ToolRegistry`
    /// - `AgentRegistry`
    /// - `WorkflowFactory`
    /// - `ComponentRegistry`
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use llmspell_bridge::ScriptRuntime;
    /// use llmspell_config::LLMSpellConfig;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let mut config = LLMSpellConfig::default();
    /// config.default_engine = "lua".to_string();
    ///
    /// // Creates runtime with Lua engine
    /// let runtime = ScriptRuntime::new(config).await?;
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Engine initialization fails
    /// - Infrastructure creation fails
    /// - `config.default_engine` is unsupported or not compiled in
    #[cfg(any(feature = "lua", feature = "javascript"))]
    #[instrument(level = "info", skip(config), fields(
        default_engine = %config.default_engine,
        rag_enabled = config.rag.enabled,
        memory_enabled = config.runtime.memory.enabled
    ))]
    pub async fn new(config: LLMSpellConfig) -> Result<Self, LLMSpellError> {
        info!(
            "Creating ScriptRuntime with engine: {}",
            config.default_engine
        );
        Box::pin(Self::with_engine(config.clone(), &config.default_engine)).await
    }

    /// Create script runtime with specific engine override (Phase 13b.16.2 - Engine-Agnostic API)
    ///
    /// Allows overriding `config.default_engine` to use a specific engine.
    /// Creates ALL infrastructure internally via `Infrastructure::from_config()`.
    ///
    /// # Arguments
    ///
    /// * `config` - Runtime configuration
    /// * `engine_name` - Engine to use ("lua", "javascript", etc.)
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use llmspell_bridge::ScriptRuntime;
    /// use llmspell_config::LLMSpellConfig;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let config = LLMSpellConfig::default();
    ///
    /// // Create runtime with Lua engine (override config.default_engine)
    /// let runtime = ScriptRuntime::with_engine(config, "lua").await?;
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Engine initialization fails
    /// - Infrastructure creation fails
    /// - `engine_name` is unsupported or not compiled in
    #[cfg(any(feature = "lua", feature = "javascript"))]
    #[instrument(level = "info", skip(config), fields(
        engine_name = %engine_name,
        rag_enabled = config.rag.enabled,
        memory_enabled = config.runtime.memory.enabled
    ))]
    pub async fn with_engine(
        config: LLMSpellConfig,
        engine_name: &str,
    ) -> Result<Self, LLMSpellError> {
        info!("Creating ScriptRuntime with engine: {engine_name}");

        // Step 1: Create ALL infrastructure from config
        let infrastructure = crate::infrastructure::Infrastructure::from_config(&config).await?;

        // Step 2: Create engine
        let engine = Self::create_engine_by_name(engine_name, &config)?;

        // Step 3: Initialize runtime with engine and infrastructure
        Self::initialize_with_infrastructure(engine, config, infrastructure).await
    }

    /// Create a new runtime with Lua engine
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use llmspell_bridge::ScriptRuntime;
    /// use llmspell_config::LLMSpellConfig;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// // With default configuration
    /// let runtime = ScriptRuntime::new(LLMSpellConfig::default()).await?;
    ///
    /// // With custom configuration
    /// let mut config = LLMSpellConfig::default();
    /// config.runtime.security.allow_file_access = true;
    /// let runtime = ScriptRuntime::new(config).await?;
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// Get list of compiled script engines
    ///
    /// Returns a list of engine names that were compiled into this binary
    /// based on enabled features. Useful for error messages and diagnostics.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use llmspell_bridge::ScriptRuntime;
    ///
    /// let engines = ScriptRuntime::available_engines();
    /// println!("Available engines: {}", engines.join(", "));
    /// ```
    #[must_use]
    #[allow(clippy::vec_init_then_push)] // Cannot use vec![] with #[cfg] attributes
    #[allow(clippy::missing_const_for_fn)] // Vec::push is not const
    pub fn available_engines() -> Vec<&'static str> {
        #[allow(unused_mut)] // mut needed when at least one feature enabled
        let mut engines = Vec::new();
        #[cfg(feature = "lua")]
        engines.push("lua");
        #[cfg(feature = "javascript")]
        engines.push("javascript");
        engines
    }

    /// Create engine by name (Phase 13b.16.2 - Engine Factory Helper)
    ///
    /// Helper for `with_engine()` that creates engine based on name.
    ///
    /// # Errors
    ///
    /// Returns an error if engine is unsupported or not compiled in
    #[cfg(any(feature = "lua", feature = "javascript"))]
    fn create_engine_by_name(
        engine_name: &str,
        config: &LLMSpellConfig,
    ) -> Result<Box<dyn ScriptEngineBridge>, LLMSpellError> {
        match engine_name {
            #[cfg(feature = "lua")]
            "lua" => {
                let lua_config = LuaConfig::default();
                EngineFactory::create_lua_engine_with_runtime(
                    &lua_config,
                    Some(Arc::new(config.clone())),
                )
            }
            #[cfg(feature = "javascript")]
            "javascript" | "js" => {
                let js_config = JSConfig::default();
                EngineFactory::create_javascript_engine(&js_config)
            }
            _ => Err(LLMSpellError::Validation {
                field: Some("engine".to_string()),
                message: format!(
                    "Unsupported or disabled engine: '{}'. Available: {}",
                    engine_name,
                    Self::available_engines().join(", ")
                ),
            }),
        }
    }

    /// Initialize runtime with engine and infrastructure (Phase 13b.16.2)
    ///
    /// Core initialization logic that wires engine with all infrastructure components.
    /// Called by both `new()` and `with_engine()` after infrastructure creation.
    ///
    /// # Errors
    ///
    /// Returns an error if initialization fails
    #[cfg(any(feature = "lua", feature = "javascript"))]
    #[instrument(level = "debug", skip(engine, config, infrastructure), fields(
        engine_name = engine.get_engine_name(),
        rag_enabled = infrastructure.rag.is_some(),
        memory_enabled = infrastructure.memory_manager.is_some()
    ))]
    async fn initialize_with_infrastructure(
        mut engine: Box<dyn ScriptEngineBridge>,
        config: LLMSpellConfig,
        infrastructure: crate::infrastructure::Infrastructure,
    ) -> Result<Self, LLMSpellError> {
        debug!("Initializing runtime with infrastructure");

        // Destructure infrastructure
        let crate::infrastructure::Infrastructure {
            provider_manager,
            state_manager,
            session_manager,
            rag,
            vector_storage,
            memory_manager,
            tool_registry,
            agent_registry,
            workflow_factory,
            component_registry,
        } = infrastructure;

        // Convert memory_manager to trait object (Phase 13b.16.2)
        let memory_manager: Option<Arc<dyn llmspell_memory::MemoryManager>> =
            memory_manager.map(|m| m as Arc<dyn llmspell_memory::MemoryManager>);

        // Register all Phase 2 tools with BOTH registries using dual-registration
        register_all_tools(&component_registry, &tool_registry, &config.tools)
            .await
            .map_err(|e| LLMSpellError::Component {
                message: format!("Failed to register tools: {e}"),
                source: None,
            })?;

        // Register default agent factory with `AgentRegistry`
        debug!("Registering default agent factory");
        let core_provider_manager = provider_manager.create_core_manager_arc().await?;
        let default_agent_factory = Arc::new(llmspell_agents::DefaultAgentFactory::new(
            core_provider_manager,
        ));

        agent_registry
            .register_factory("default".to_string(), default_agent_factory)
            .await
            .map_err(|e| LLMSpellError::Component {
                message: format!("Failed to register default agent factory: {e}"),
                source: None,
            })?;

        debug!("Default agent factory registered successfully");

        // Create RAGInfrastructure if RAG is enabled
        let rag_infrastructure = if let (Some(rag_ref), Some(vs_ref)) = (&rag, &vector_storage) {
            debug!("Creating RAGInfrastructure with vector_storage");
            Some(Arc::new(
                crate::globals::rag_infrastructure::RAGInfrastructure {
                    multi_tenant_rag: rag_ref.clone(),
                    vector_storage: Some(vs_ref.clone() as Arc<dyn llmspell_storage::VectorStorage>),
                    sqlite_storage: Some(vs_ref.clone()),
                },
            ))
        } else {
            debug!(
                "RAG infrastructure not created - rag: {}, vector_storage: {}",
                rag.is_some(),
                vector_storage.is_some()
            );
            None
        };

        // Create API dependencies struct for clean injection
        let api_deps = crate::engine::bridge::ApiDependencies::new(
            component_registry.clone(),
            provider_manager.clone(),
            tool_registry.clone(),
            agent_registry.clone(),
            workflow_factory.clone(),
        )
        .with_session_manager(session_manager.clone())
        .with_state_manager(state_manager.clone());

        // Add RAG infrastructure if available
        let api_deps = if let Some(rag_infra) = rag_infrastructure {
            api_deps.with_rag(rag_infra as Arc<dyn std::any::Any + Send + Sync>)
        } else {
            api_deps
        };

        // Inject APIs into the engine with full infrastructure
        engine.inject_apis(&api_deps)?;

        // Create execution context
        let execution_context = Arc::new(RwLock::new(crate::engine::ExecutionContext {
            working_directory: std::env::current_dir()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string(),
            environment: std::env::vars().collect(),
            state: serde_json::Value::Object(serde_json::Map::new()),
            security: crate::engine::SecurityContext {
                allow_file_access: config.runtime.security.allow_file_access,
                allow_network_access: config.runtime.security.allow_network_access,
                allow_process_spawn: config.runtime.security.allow_process_spawn,
                max_memory_bytes: config.runtime.security.max_memory_bytes,
                max_execution_time_ms: config.runtime.security.max_execution_time_ms,
            },
        }));

        Ok(Self {
            engine,
            registry: component_registry,
            provider_manager,
            tool_registry,
            agent_registry,
            workflow_factory,
            state_manager,
            session_manager,
            rag,
            memory_manager,
            execution_context,
            debug_context: Arc::new(RwLock::new(None)),
            _config: config,
        })
    }

    /// Execute a script and return the output
    ///
    /// # Errors
    ///
    /// Returns an error if script execution fails
    #[instrument(level = "info", skip(self, script), fields(
        engine_name = self.engine.get_engine_name(),
        script_size = script.len(),
        execution_id = %uuid::Uuid::new_v4()
    ))]
    pub async fn execute_script(&self, script: &str) -> Result<ScriptOutput, LLMSpellError> {
        info!("Executing script with {} bytes", script.len());
        self.engine.execute_script(script).await
    }

    /// Get completion candidates for the given context
    ///
    /// This method is used for REPL tab completion to suggest available
    /// variables, functions, and other completable elements.
    ///
    /// # Arguments
    ///
    /// * `context` - The completion context containing the line and cursor position
    ///
    /// # Returns
    ///
    /// A vector of completion candidates suitable for the current context
    #[must_use]
    pub fn get_completion_candidates(
        &self,
        context: &crate::engine::bridge::CompletionContext,
    ) -> Vec<crate::engine::bridge::CompletionCandidate> {
        self.engine.get_completion_candidates(context)
    }

    /// Execute a script with streaming output
    ///
    /// # Errors
    ///
    /// Returns an error if the engine doesn't support streaming or script execution fails
    #[instrument(level = "debug", skip(self, script), fields(
        engine_name = self.engine.get_engine_name(),
        script_size = script.len(),
        execution_id = %uuid::Uuid::new_v4(),
        streaming_supported = self.engine.supports_streaming()
    ))]
    pub async fn execute_script_streaming(
        &self,
        script: &str,
    ) -> Result<ScriptStream, LLMSpellError> {
        debug!("Executing script with streaming output");
        if !self.engine.supports_streaming() {
            return Err(LLMSpellError::Component {
                message: format!(
                    "{} engine does not support streaming execution",
                    self.engine.get_engine_name()
                ),
                source: None,
            });
        }
        self.engine.execute_script_streaming(script).await
    }

    /// Set script arguments to be passed to the script
    ///
    /// These arguments will be made available to the script in a language-specific way:
    /// - Lua: Available as global `ARGS` table
    /// - JavaScript: Available as global `args` object
    /// - Python: Available as `sys.argv` equivalent
    ///
    /// # Errors
    ///
    /// Returns an error if the engine fails to set arguments
    #[instrument(level = "debug", skip(self, args), fields(
        engine_name = self.engine.get_engine_name(),
        argument_count = args.len()
    ))]
    pub async fn set_script_args(
        &mut self,
        args: HashMap<String, String>,
    ) -> Result<(), LLMSpellError> {
        debug!("Setting {} script arguments", args.len());
        self.engine.set_script_args(args).await
    }

    /// Get the name of the current engine
    #[must_use]
    pub fn get_engine_name(&self) -> &'static str {
        self.engine.get_engine_name()
    }

    /// Check if the engine supports streaming
    #[must_use]
    pub fn supports_streaming(&self) -> bool {
        self.engine.supports_streaming()
    }

    /// Check if the engine supports multimodal content
    #[must_use]
    pub fn supports_multimodal(&self) -> bool {
        self.engine.supports_multimodal()
    }

    /// Get the engine's supported features
    #[must_use]
    pub fn get_engine_features(&self) -> crate::engine::EngineFeatures {
        self.engine.supported_features()
    }

    /// Get the component registry
    #[must_use]
    pub const fn registry(&self) -> &Arc<ComponentRegistry> {
        &self.registry
    }

    /// Get the provider manager
    #[must_use]
    pub const fn provider_manager(&self) -> &Arc<ProviderManager> {
        &self.provider_manager
    }

    /// Get the tool registry (Phase 12.7.1.2 Step 8)
    /// Used by template execution infrastructure for hooks, discovery, validation
    #[must_use]
    pub const fn tool_registry(&self) -> &Arc<llmspell_tools::ToolRegistry> {
        &self.tool_registry
    }

    /// Get the agent factory registry (Phase 12.7.1.2 Step 8)
    /// Used by template execution infrastructure for agent creation
    #[must_use]
    pub const fn agent_registry(&self) -> &Arc<llmspell_agents::FactoryRegistry> {
        &self.agent_registry
    }

    /// Get the workflow factory (Phase 12.7.1.2 Step 8)
    /// Used by template execution infrastructure for workflow creation
    #[must_use]
    pub fn workflow_factory(&self) -> &Arc<dyn llmspell_workflows::WorkflowFactory> {
        &self.workflow_factory
    }

    /// Downcast support for kernel to access concrete `ScriptRuntime` methods (Phase 12.8.2.5)
    ///
    /// Enables kernel to downcast `Arc<dyn ScriptExecutor>` to `&ScriptRuntime` for
    /// calling methods not in the trait (like `set_session_manager()`).
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// // In kernel initialization:
    /// if let Some(runtime) = script_executor.as_any().downcast_ref::<ScriptRuntime>() {
    ///     runtime.set_session_manager(session_manager);
    /// }
    /// ```
    #[must_use]
    pub fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    /// Get the current execution context
    ///
    /// # Panics
    ///
    /// Panics if the execution context lock is poisoned
    #[must_use]
    pub fn get_execution_context(&self) -> crate::engine::ExecutionContext {
        self.execution_context.read().unwrap().clone()
    }

    /// Update the execution context
    ///
    /// # Errors
    ///
    /// Returns an error if the context lock is poisoned
    ///
    /// # Panics
    ///
    /// Panics if the write lock cannot be acquired
    pub fn set_execution_context(
        &self,
        context: crate::engine::ExecutionContext,
    ) -> Result<(), LLMSpellError> {
        {
            let mut ctx = self.execution_context.write().unwrap();
            *ctx = context;
        } // Explicitly drop the lock here
        Ok(())
    }
}

impl From<llmspell_config::SecurityConfig> for crate::engine::SecurityContext {
    fn from(config: llmspell_config::SecurityConfig) -> Self {
        Self {
            allow_file_access: config.allow_file_access,
            allow_network_access: config.allow_network_access,
            allow_process_spawn: config.allow_process_spawn,
            max_memory_bytes: config.max_memory_bytes,
            max_execution_time_ms: config.max_execution_time_ms,
        }
    }
}

/// Implementation of `ScriptExecutor` trait for `ScriptRuntime`
///
/// This allows the kernel to execute scripts without directly depending on
/// the bridge crate, avoiding cyclic dependencies.
#[async_trait]
impl ScriptExecutor for ScriptRuntime {
    #[instrument(skip(self, script))]
    async fn execute_script(&self, script: &str) -> Result<ScriptExecutionOutput, LLMSpellError> {
        let start = Instant::now();

        // Execute using the underlying engine
        let engine_output = self.engine.execute_script(script).await?;

        // Convert ScriptOutput to ScriptExecutionOutput
        let output = ScriptExecutionOutput {
            output: engine_output.output,
            console_output: engine_output.console_output,
            metadata: ScriptExecutionMetadata {
                duration: start.elapsed(),
                language: engine_output.metadata.engine.clone(),
                exit_code: None, // ScriptMetadata doesn't have exit_code
                warnings: engine_output.metadata.warnings,
            },
        };

        Ok(output)
    }

    async fn execute_script_with_args(
        &self,
        script: &str,
        args: std::collections::HashMap<String, String>,
    ) -> Result<ScriptExecutionOutput, LLMSpellError> {
        let start = Instant::now();

        debug!("Executing script with {} arguments", args.len());

        // We need to temporarily set the args and then execute
        // Since we can't mutate self, we need to use a different approach
        // Create a new script with args injected as a preamble
        let script_with_args = if args.is_empty() {
            script.to_string()
        } else {
            let mut preamble = String::from("-- Injected script arguments\nARGS = {}\n");
            for (key, value) in &args {
                // Escape the value for Lua string
                let escaped_value = value.replace('\\', "\\\\").replace('"', "\\\"");
                writeln!(preamble, "ARGS[\"{key}\"] = \"{escaped_value}\"")
                    .expect("String write should never fail");
            }
            preamble.push_str("\n-- Original script\n");
            preamble.push_str(script);
            preamble
        };

        // Execute using the underlying engine
        let engine_output = self.engine.execute_script(&script_with_args).await?;

        // Convert ScriptOutput to ScriptExecutionOutput
        let output = ScriptExecutionOutput {
            output: engine_output.output,
            console_output: engine_output.console_output,
            metadata: ScriptExecutionMetadata {
                duration: start.elapsed(),
                language: engine_output.metadata.engine.clone(),
                exit_code: None,
                warnings: engine_output.metadata.warnings,
            },
        };

        Ok(output)
    }

    fn supports_streaming(&self) -> bool {
        self.engine.supports_streaming()
    }

    fn language(&self) -> &'static str {
        // Return the configured engine type
        // TODO: Add a method to get current engine language
        "lua" // Default for now since we use Lua primarily
    }

    async fn is_ready(&self) -> bool {
        // Engine is ready if it's been initialized
        // TODO: Add proper readiness check to ScriptEngineBridge trait
        true
    }

    fn set_debug_context(&self, context: Option<Arc<dyn DebugContext>>) {
        // Use interior mutability to set debug context
        self.debug_context.write().unwrap().clone_from(&context);

        // Also set it on the underlying engine if it supports debugging
        self.engine.set_debug_context(context);
    }

    fn supports_debugging(&self) -> bool {
        // Check if the underlying engine supports debugging
        self.engine.supports_debugging()
    }

    fn get_debug_context(&self) -> Option<Arc<dyn DebugContext>> {
        // Return the stored debug context
        let debug_context = self.debug_context.read().unwrap();
        debug_context.clone()
    }

    fn get_session_manager_any(&self) -> Option<Arc<dyn std::any::Any + Send + Sync>> {
        Some(self.session_manager.clone() as Arc<dyn std::any::Any + Send + Sync>)
    }

    fn component_registry(&self) -> Option<Arc<dyn ComponentLookup>> {
        // Return the component registry as ComponentLookup trait
        Some(Arc::clone(&self.registry) as Arc<dyn ComponentLookup>)
    }

    fn template_registry_any(&self) -> Option<Arc<dyn std::any::Any + Send + Sync>> {
        // Return the template registry as type-erased Any to avoid circular dependencies
        self.registry
            .template_registry()
            .map(|reg| reg as Arc<dyn std::any::Any + Send + Sync>)
    }

    fn get_completion_candidates(&self, line: &str, cursor_pos: usize) -> Vec<(String, String)> {
        // Create a CompletionContext from the provided line and cursor position
        let context = crate::engine::bridge::CompletionContext::new(line, cursor_pos);

        // Get completions from the underlying engine
        let candidates = self.engine.get_completion_candidates(&context);

        // Convert CompletionCandidate to tuple format expected by ScriptExecutor trait
        candidates
            .into_iter()
            .map(|candidate| {
                let display = format!("{:?}", candidate.kind).to_lowercase();
                (candidate.text, display)
            })
            .collect()
    }

    // === Template Operations (JSON-based API to avoid circular dependencies) ===

    fn handle_template_list(
        &self,
        category: Option<&str>,
    ) -> Result<serde_json::Value, LLMSpellError> {
        use serde_json::json;

        // Get TemplateRegistry via type erasure
        let registry_any =
            self.template_registry_any()
                .ok_or_else(|| LLMSpellError::Component {
                    message: "Template registry not available".to_string(),
                    source: None,
                })?;

        // Downcast to concrete TemplateRegistry
        let template_registry = std::sync::Arc::downcast::<
            llmspell_templates::registry::TemplateRegistry,
        >(registry_any)
        .map_err(|_| LLMSpellError::Component {
            message: "Failed to access template registry".to_string(),
            source: None,
        })?;

        // Get templates by category or all
        let templates_metadata = if let Some(cat_str) = category {
            // Parse category string
            use llmspell_templates::core::TemplateCategory;
            let category = match cat_str.to_lowercase().as_str() {
                "research" => TemplateCategory::Research,
                "chat" => TemplateCategory::Chat,
                "analysis" => TemplateCategory::Analysis,
                "codegen" => TemplateCategory::CodeGen,
                "document" => TemplateCategory::Document,
                "workflow" => TemplateCategory::Workflow,
                _ => {
                    return Err(LLMSpellError::Validation {
                        field: Some("category".to_string()),
                        message: format!("Invalid category: {cat_str}"),
                    });
                }
            };
            template_registry.discover_by_category(&category)
        } else {
            template_registry.list_metadata()
        };

        // Convert to JSON
        let templates_json: Vec<serde_json::Value> = templates_metadata
            .iter()
            .map(|meta| {
                json!({
                    "id": meta.id,
                    "name": meta.name,
                    "description": meta.description,
                    "category": format!("{:?}", meta.category),
                    "version": meta.version,
                    "author": meta.author,
                    "tags": meta.tags,
                })
            })
            .collect();

        Ok(json!(templates_json))
    }

    fn handle_template_info(
        &self,
        template_id: &str,
        with_schema: bool,
    ) -> Result<serde_json::Value, LLMSpellError> {
        use serde_json::json;

        // Get TemplateRegistry via type erasure
        let registry_any =
            self.template_registry_any()
                .ok_or_else(|| LLMSpellError::Component {
                    message: "Template registry not available".to_string(),
                    source: None,
                })?;

        // Downcast to concrete TemplateRegistry
        let template_registry = std::sync::Arc::downcast::<
            llmspell_templates::registry::TemplateRegistry,
        >(registry_any)
        .map_err(|_| LLMSpellError::Component {
            message: "Failed to access template registry".to_string(),
            source: None,
        })?;

        // Get template
        let template =
            template_registry
                .get(template_id)
                .map_err(|e| LLMSpellError::Component {
                    message: format!("Template not found: {e}"),
                    source: None,
                })?;

        let metadata = template.metadata();
        let mut info_json = json!({
            "id": metadata.id,
            "name": metadata.name,
            "description": metadata.description,
            "category": format!("{:?}", metadata.category),
            "version": metadata.version,
            "author": metadata.author,
            "requires": metadata.requires,
            "tags": metadata.tags,
        });

        // Add schema if requested
        if with_schema {
            let schema = template.config_schema();
            if let Ok(schema_json) = serde_json::to_value(schema) {
                info_json["schema"] = schema_json;
            }
        }

        Ok(info_json)
    }

    /// Handle template exec command
    ///
    /// Builds `ExecutionContext` with all required infrastructure:
    /// - Tool/Agent/Workflow registries (always required)
    /// - Provider manager + `provider_config` (always required, Phase 13.5.7d)
    /// - Session manager (optional, if wired from kernel)
    /// - RAG infrastructure (optional, if wired from kernel)
    ///
    /// # Errors
    ///
    /// Returns error if:
    /// - Template not found in registry
    /// - Parameter validation fails
    /// - `ExecutionContext` build fails (missing required infrastructure)
    /// - Template execution fails
    async fn handle_template_exec(
        &self,
        template_id: &str,
        params: serde_json::Value,
    ) -> Result<serde_json::Value, LLMSpellError> {
        use serde_json::json;

        // Get template from registry
        let template = self.get_template_from_registry(template_id)?;

        // Convert and validate params
        let template_params = Self::convert_and_validate_params(&template, &params)?;

        // Build execution context with infrastructure registries (Phase 12.7.1.3 + 12.8.2.5)
        // Wire in the 5 required components for template execution
        let core_provider_manager = self.provider_manager.create_core_manager_arc().await?;

        // Get session manager (Phase 13b.16.2: direct ownership)
        let session_manager = Some(self.session_manager.clone());

        // Get RAG if available (Phase 13b.16.2: direct ownership)
        let rag = self.rag.clone();
        if rag.is_some() {
            debug!("RAG available for template execution");
        } else {
            debug!("RAG NOT available for template execution - check config.rag.enabled");
        }

        // Get provider configuration for ExecutionContext (Task 13b.1.7 - Phase 13.5.7d regression fix)
        //
        // Phase 13.5.7d made provider_config REQUIRED in ExecutionContext to enable
        // smart dual-path LLM provider resolution:
        //   1. provider_name param → centralized config lookup (RECOMMENDED)
        //   2. model param → ephemeral provider with inline overrides (backward compat)
        //   3. Default provider → fallback from ProviderManagerConfig
        //
        // Without provider_config, ExecutionContext::build() fails with:
        // "Required infrastructure not available: provider_config is required"
        //
        // See: llmspell-templates/src/context.rs:706-709 (validation)
        //      llmspell-templates/src/context.rs:160-230 (smart resolution)
        let provider_config = Arc::new(self.provider_manager.config().clone());

        let mut builder = llmspell_templates::context::ExecutionContext::builder()
            .with_tool_registry(self.tool_registry.clone())
            .with_agent_registry(self.agent_registry.clone())
            .with_workflow_factory(self.workflow_factory.clone())
            .with_providers(core_provider_manager)
            .with_provider_config(provider_config);

        // Add session manager if wired from kernel (Phase 12.8.2.5)
        if let Some(sm) = session_manager {
            builder = builder.with_session_manager(sm);
            debug!("Session manager added to template execution context");
        }

        // Add RAG if wired from kernel (Phase 12.8.fix)
        if let Some(r) = rag {
            builder = builder.with_rag(r);
            debug!("RAG infrastructure added to template execution context");
        }

        let context = builder.build().map_err(|e| LLMSpellError::Component {
            message: format!("Failed to build execution context: {e}"),
            source: None,
        })?;

        // Check if any providers are initialized and fail fast if none available (Task 13b.15)
        // Prevents hanging when templates need LLM but all providers disabled/missing
        // Most templates use agents which require providers, so check unconditionally
        let providers = self.provider_manager.list_providers().await;
        if providers.is_empty() {
            return Err(LLMSpellError::Configuration {
                message: format!(
                    "Template '{template_id}' execution requires LLM providers, but none are configured/enabled. \
                     Enable at least one provider in config or use --profile with valid providers."
                ),
                source: None,
            });
        }

        // Execute template
        let output = template
            .execute(template_params, context)
            .await
            .map_err(|e| LLMSpellError::Component {
                message: format!("Template execution failed: {e}"),
                source: None,
            })?;

        // Convert output to JSON response
        Ok(json!({
            "result": Self::convert_template_result_to_json(&output.result),
            "artifacts": Self::convert_artifacts_to_json(&output.artifacts),
            "metrics": {
                "duration_ms": output.metrics.duration_ms,
                "tokens_used": output.metrics.tokens_used,
                "cost_usd": output.metrics.cost_usd,
                "agents_invoked": output.metrics.agents_invoked,
                "tools_invoked": output.metrics.tools_invoked,
                "rag_queries": output.metrics.rag_queries,
                "custom_metrics": output.metrics.custom_metrics,
            }
        }))
    }

    fn handle_template_search(
        &self,
        query: &str,
        category: Option<&str>,
    ) -> Result<serde_json::Value, LLMSpellError> {
        use serde_json::json;

        // Get TemplateRegistry via type erasure
        let registry_any =
            self.template_registry_any()
                .ok_or_else(|| LLMSpellError::Component {
                    message: "Template registry not available".to_string(),
                    source: None,
                })?;

        // Downcast to concrete TemplateRegistry
        let template_registry = std::sync::Arc::downcast::<
            llmspell_templates::registry::TemplateRegistry,
        >(registry_any)
        .map_err(|_| LLMSpellError::Component {
            message: "Failed to access template registry".to_string(),
            source: None,
        })?;

        // Search templates
        let mut results = template_registry.search(query);

        // Filter by category if specified
        if let Some(cat_str) = category {
            use llmspell_templates::core::TemplateCategory;
            let category = match cat_str.to_lowercase().as_str() {
                "research" => TemplateCategory::Research,
                "chat" => TemplateCategory::Chat,
                "analysis" => TemplateCategory::Analysis,
                "codegen" => TemplateCategory::CodeGen,
                "document" => TemplateCategory::Document,
                "workflow" => TemplateCategory::Workflow,
                _ => {
                    return Err(LLMSpellError::Validation {
                        field: Some("category".to_string()),
                        message: format!("Invalid category: {cat_str}"),
                    });
                }
            };
            results.retain(|m| m.category == category);
        }

        // Convert to JSON
        let results_json: Vec<serde_json::Value> = results
            .iter()
            .map(|meta| {
                json!({
                    "id": meta.id,
                    "name": meta.name,
                    "description": meta.description,
                    "category": format!("{:?}", meta.category),
                    "tags": meta.tags,
                })
            })
            .collect();

        Ok(json!(results_json))
    }

    fn handle_template_schema(
        &self,
        template_id: &str,
    ) -> Result<serde_json::Value, LLMSpellError> {
        // Get TemplateRegistry via type erasure
        let registry_any =
            self.template_registry_any()
                .ok_or_else(|| LLMSpellError::Component {
                    message: "Template registry not available".to_string(),
                    source: None,
                })?;

        // Downcast to concrete TemplateRegistry
        let template_registry = std::sync::Arc::downcast::<
            llmspell_templates::registry::TemplateRegistry,
        >(registry_any)
        .map_err(|_| LLMSpellError::Component {
            message: "Failed to access template registry".to_string(),
            source: None,
        })?;

        // Get template
        let template =
            template_registry
                .get(template_id)
                .map_err(|e| LLMSpellError::Component {
                    message: format!("Template not found: {e}"),
                    source: None,
                })?;

        let schema = template.config_schema();
        serde_json::to_value(schema).map_err(|e| LLMSpellError::Component {
            message: format!("Failed to serialize schema: {e}"),
            source: None,
        })
    }

    // === Memory Operations (Phase 13.12.1) ===

    fn handle_memory_add(
        &self,
        session_id: &str,
        role: &str,
        content: &str,
        metadata: serde_json::Value,
    ) -> Result<serde_json::Value, LLMSpellError> {
        use serde_json::json;

        // Get memory manager
        let memory_manager =
            self.memory_manager
                .clone()
                .ok_or_else(|| LLMSpellError::Component {
                    message: "Memory manager not available".to_string(),
                    source: None,
                })?;

        // Create episodic entry with metadata
        let mut entry = llmspell_memory::EpisodicEntry::new(
            session_id.to_string(),
            role.to_string(),
            content.to_string(),
        );
        entry.metadata = metadata;

        // Add to episodic memory
        let entry_id = tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current()
                .block_on(async { memory_manager.episodic().add(entry).await })
        })
        .map_err(|e| LLMSpellError::Component {
            message: format!("Failed to add episodic memory: {e}"),
            source: None,
        })?;

        Ok(json!({"status": "success", "entry_id": entry_id}))
    }

    fn handle_memory_search(
        &self,
        session_id: Option<&str>,
        query: &str,
        limit: usize,
    ) -> Result<serde_json::Value, LLMSpellError> {
        use serde_json::json;

        // Get memory manager
        let memory_manager =
            self.memory_manager
                .clone()
                .ok_or_else(|| LLMSpellError::Component {
                    message: "Memory manager not available".to_string(),
                    source: None,
                })?;

        // Search episodic memory
        let mut results = tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current()
                .block_on(async { memory_manager.episodic().search(query, limit).await })
        })
        .map_err(|e| LLMSpellError::Component {
            message: format!("Failed to search episodic memory: {e}"),
            source: None,
        })?;

        // Filter by session if specified
        if let Some(sid) = session_id {
            results.retain(|entry| entry.session_id == sid);
        }

        // Convert results to JSON
        let results_json: Vec<serde_json::Value> = results
            .into_iter()
            .map(|entry| {
                json!({
                    "id": entry.id,
                    "session_id": entry.session_id,
                    "role": entry.role,
                    "content": entry.content,
                    "timestamp": entry.timestamp.to_rfc3339(),
                    "metadata": entry.metadata,
                })
            })
            .collect();

        Ok(json!(results_json))
    }

    fn handle_memory_query(
        &self,
        query: &str,
        limit: usize,
    ) -> Result<serde_json::Value, LLMSpellError> {
        use serde_json::json;

        // Get memory manager
        let _memory_manager =
            self.memory_manager
                .clone()
                .ok_or_else(|| LLMSpellError::Component {
                    message: "Memory manager not available".to_string(),
                    source: None,
                })?;

        // Query semantic memory (knowledge graph)
        // Note: SemanticMemory trait doesn't have text search, so we return empty for now
        // Full implementation will come with dedicated context pipeline integration
        Ok(json!({
            "message": "Semantic memory query not yet implemented (requires context pipeline)",
            "query": query,
            "limit": limit,
            "entities": []
        }))
    }

    fn handle_memory_stats(&self) -> Result<serde_json::Value, LLMSpellError> {
        use serde_json::json;

        // Get memory manager
        let memory_manager =
            self.memory_manager
                .clone()
                .ok_or_else(|| LLMSpellError::Component {
                    message: "Memory manager not available".to_string(),
                    source: None,
                })?;

        // Get stats from episodic and semantic memory
        // Note: Memory traits don't have count methods, so we get session lists as proxy
        let sessions_with_unprocessed = tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {
                memory_manager
                    .episodic()
                    .list_sessions_with_unprocessed()
                    .await
            })
        })
        .map_err(|e| LLMSpellError::Component {
            message: format!("Failed to get episodic stats: {e}"),
            source: None,
        })?;

        Ok(json!({
            "episodic": {
                "sessions_with_unprocessed": sessions_with_unprocessed.len(),
                "sessions": sessions_with_unprocessed
            },
            "semantic": {
                "message": "Entity/relationship counts not exposed in current API"
            },
            "procedural": {
                "patterns": 0
            },
            "consolidation": {
                "enabled": memory_manager.has_consolidation()
            }
        }))
    }

    fn handle_memory_consolidate(
        &self,
        session_id: Option<&str>,
        force: bool,
    ) -> Result<serde_json::Value, LLMSpellError> {
        use serde_json::json;

        // Get memory manager
        let memory_manager =
            self.memory_manager
                .clone()
                .ok_or_else(|| LLMSpellError::Component {
                    message: "Memory manager not available".to_string(),
                    source: None,
                })?;

        // Determine consolidation mode
        let mode = if force {
            llmspell_memory::ConsolidationMode::Immediate
        } else {
            llmspell_memory::ConsolidationMode::Background
        };

        // Consolidate episodic to semantic memory
        let result = tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {
                let sid = session_id.unwrap_or("");
                memory_manager.consolidate(sid, mode, None).await
            })
        })
        .map_err(|e| LLMSpellError::Component {
            message: format!("Consolidation failed: {e}"),
            source: None,
        })?;

        Ok(json!({
            "status": "success",
            "entries_processed": result.entries_processed,
            "entities_added": result.entities_added,
            "entities_updated": result.entities_updated,
            "entities_deleted": result.entities_deleted,
            "entries_skipped": result.entries_skipped,
            "entries_failed": result.entries_failed,
            "duration_ms": result.duration_ms,
        }))
    }

    // === Context Operations (Phase 13.12.3) ===

    fn handle_context_assemble(
        &self,
        query: &str,
        strategy: &str,
        budget: usize,
        session_id: Option<&str>,
    ) -> Result<serde_json::Value, LLMSpellError> {
        use serde_json::json;

        // Check if memory manager is available (Phase 13b.16.2: direct ownership check)
        if self.memory_manager.is_none() {
            return Err(LLMSpellError::Component {
                message: "Context operations not available (memory manager not set)".to_string(),
                source: None,
            });
        }

        // Get memory manager
        let memory_manager =
            self.memory_manager
                .clone()
                .ok_or_else(|| LLMSpellError::Component {
                    message: "Memory manager not available".to_string(),
                    source: None,
                })?;

        // Simple context assembly based on strategy
        let chunks = tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {
                match strategy {
                    "episodic" => {
                        // Get from episodic memory only
                        let mut results = memory_manager.episodic().search(query, budget).await?;

                        // Filter by session if specified
                        if let Some(sid) = session_id {
                            results.retain(|entry| entry.session_id == sid);
                        }

                        let chunks: Vec<serde_json::Value> = results
                            .into_iter()
                            .map(|entry| {
                                json!({
                                    "type": "episodic",
                                    "content": entry.content,
                                    "role": entry.role,
                                    "timestamp": entry.timestamp.to_rfc3339(),
                                    "session_id": entry.session_id,
                                })
                            })
                            .collect();
                        Ok::<_, llmspell_memory::MemoryError>(chunks)
                    }
                    "semantic" => {
                        // Semantic memory text search not yet available in current API
                        Ok(vec![json!({
                            "type": "info",
                            "content": "Semantic memory text search requires context pipeline (Phase 13.12.3 full implementation)",
                        })])
                    }
                    _ => {
                        // Use episodic only for now (semantic text search not available)
                        // This includes "hybrid" and any other strategy
                        let mut results = memory_manager.episodic().search(query, budget).await?;

                        // Filter by session if specified
                        if let Some(sid) = session_id {
                            results.retain(|entry| entry.session_id == sid);
                        }

                        let chunks: Vec<serde_json::Value> = results
                            .into_iter()
                            .map(|entry| {
                                json!({
                                    "type": "episodic",
                                    "content": entry.content,
                                    "role": entry.role,
                                    "timestamp": entry.timestamp.to_rfc3339(),
                                    "session_id": entry.session_id,
                                })
                            })
                            .collect();

                        Ok(chunks)
                    }
                }
            })
        })
        .map_err(|e| LLMSpellError::Component {
            message: format!("Context assembly failed: {e}"),
            source: None,
        })?;

        Ok(json!({
            "strategy": strategy,
            "chunks": chunks,
            "total_chunks": chunks.len(),
        }))
    }

    fn handle_context_strategies(&self) -> Result<serde_json::Value, LLMSpellError> {
        // Return list of available strategies (default implementation from trait)
        use serde_json::json;
        Ok(json!([
            {
                "name": "hybrid",
                "description": "Combines episodic and semantic memory (recommended)"
            },
            {
                "name": "episodic",
                "description": "Conversation history only"
            },
            {
                "name": "semantic",
                "description": "Knowledge graph entities only"
            }
        ]))
    }

    fn handle_context_analyze(
        &self,
        query: &str,
        budget: usize,
    ) -> Result<serde_json::Value, LLMSpellError> {
        use serde_json::json;

        // Check if memory manager is available (Phase 13b.16.2: direct ownership check)
        if self.memory_manager.is_none() {
            return Err(LLMSpellError::Component {
                message: "Context operations not available (memory manager not set)".to_string(),
                source: None,
            });
        }

        // Get memory manager
        let memory_manager =
            self.memory_manager
                .clone()
                .ok_or_else(|| LLMSpellError::Component {
                    message: "Memory manager not available".to_string(),
                    source: None,
                })?;

        // Analyze token usage for each strategy
        let analysis = tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {
                // Episodic strategy
                let episodic_results = memory_manager.episodic().search(query, budget).await?;
                let episodic_tokens: usize = episodic_results
                    .iter()
                    .map(|entry| entry.content.split_whitespace().count())
                    .sum();

                // Semantic strategy analysis not available (requires text search)
                let semantic_tokens: usize = 0;
                let semantic_count: usize = 0;

                // Hybrid uses episodic only for now
                let hybrid_tokens = episodic_tokens;

                Ok::<_, llmspell_memory::MemoryError>(json!([
                    {
                        "strategy": "episodic",
                        "estimated_tokens": episodic_tokens,
                        "chunks": episodic_results.len(),
                    },
                    {
                        "strategy": "semantic",
                        "estimated_tokens": semantic_tokens,
                        "chunks": semantic_count,
                        "message": "Requires context pipeline for text search"
                    },
                    {
                        "strategy": "hybrid",
                        "estimated_tokens": hybrid_tokens,
                        "chunks": episodic_results.len(),
                        "message": "Currently episodic-only (semantic search pending)"
                    }
                ]))
            })
        })
        .map_err(|e| LLMSpellError::Component {
            message: format!("Context analysis failed: {e}"),
            source: None,
        })?;

        Ok(analysis)
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

/// Helper methods for template execution
impl ScriptRuntime {
    /// Helper to get template from registry with type erasure
    fn get_template_from_registry(
        &self,
        template_id: &str,
    ) -> Result<std::sync::Arc<dyn llmspell_templates::core::Template>, LLMSpellError> {
        let registry_any =
            self.template_registry_any()
                .ok_or_else(|| LLMSpellError::Component {
                    message: "Template registry not available".to_string(),
                    source: None,
                })?;

        let template_registry = std::sync::Arc::downcast::<
            llmspell_templates::registry::TemplateRegistry,
        >(registry_any)
        .map_err(|_| LLMSpellError::Component {
            message: "Failed to access template registry".to_string(),
            source: None,
        })?;

        template_registry
            .get(template_id)
            .map_err(|e| LLMSpellError::Component {
                message: format!("Template not found: {e}"),
                source: None,
            })
    }

    /// Helper to convert and validate JSON params to `TemplateParams`
    fn convert_and_validate_params(
        template: &std::sync::Arc<dyn llmspell_templates::core::Template>,
        params: &serde_json::Value,
    ) -> Result<llmspell_templates::core::TemplateParams, LLMSpellError> {
        let params_obj = params
            .as_object()
            .ok_or_else(|| LLMSpellError::Validation {
                field: Some("params".to_string()),
                message: "Parameters must be a JSON object".to_string(),
            })?;

        let mut template_params = llmspell_templates::core::TemplateParams::new();
        for (key, value) in params_obj {
            template_params.insert(key.clone(), value.clone());
        }

        template
            .validate(&template_params)
            .map_err(|e| LLMSpellError::Validation {
                field: Some("params".to_string()),
                message: format!("Parameter validation failed: {e}"),
            })?;

        Ok(template_params)
    }

    /// Helper to convert `TemplateResult` to JSON
    fn convert_template_result_to_json(
        result: &llmspell_templates::core::TemplateResult,
    ) -> serde_json::Value {
        use serde_json::json;

        match result {
            llmspell_templates::core::TemplateResult::Text(text) => {
                json!({"type": "text", "value": text})
            }
            llmspell_templates::core::TemplateResult::Structured(value) => {
                json!({"type": "structured", "value": value})
            }
            llmspell_templates::core::TemplateResult::File(path) => {
                json!({"type": "file", "path": path.display().to_string()})
            }
            llmspell_templates::core::TemplateResult::Multiple(results) => {
                let results_json: Vec<serde_json::Value> = results
                    .iter()
                    .map(|r| match r {
                        llmspell_templates::core::TemplateResult::Text(t) => {
                            json!({"type": "text", "value": t})
                        }
                        llmspell_templates::core::TemplateResult::File(p) => {
                            json!({"type": "file", "path": p.display().to_string()})
                        }
                        llmspell_templates::core::TemplateResult::Structured(v) => {
                            json!({"type": "structured", "value": v})
                        }
                        llmspell_templates::core::TemplateResult::Multiple(_) => {
                            json!({"type": "nested_multiple"})
                        }
                    })
                    .collect();
                json!({"type": "multiple", "results": results_json})
            }
        }
    }

    /// Helper to convert artifacts to JSON
    fn convert_artifacts_to_json(
        artifacts: &[llmspell_templates::artifacts::Artifact],
    ) -> Vec<serde_json::Value> {
        use serde_json::json;

        artifacts
            .iter()
            .map(|a| {
                json!({
                    "filename": a.filename,
                    "mime_type": a.mime_type,
                    "size": a.size()
                })
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_runtime_config_default() {
        let config = LLMSpellConfig::default();
        assert_eq!(config.default_engine, "lua");
        assert!(config.supports_engine("lua"));
        assert!(config.supports_engine("javascript"));
        assert!(!config.supports_engine("python"));
    }
    #[test]
    fn test_security_config_conversion() {
        let config = llmspell_config::SecurityConfig::default();
        let context: crate::engine::SecurityContext = config.into();
        assert!(!context.allow_file_access);
        assert!(context.allow_network_access);
        assert!(!context.allow_process_spawn);
    }
}
