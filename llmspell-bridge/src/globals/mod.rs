//! ABOUTME: Global object injection infrastructure for script engines
//! ABOUTME: Provides centralized management of all script-accessible globals

pub mod agent_global;
pub mod artifact_global;
pub mod config_global;
pub mod context_global;
pub mod core;
pub mod debug_global;
pub mod event_global;
pub mod hook_global;
pub mod injection;
pub mod json_global;
pub mod local_llm_global;
pub mod memory_global;
pub mod provider_global;
pub mod rag_global;
pub mod rag_infrastructure;
pub mod registry;
pub mod replay_global;
pub mod session_global;
pub mod session_infrastructure;
pub mod state_global;
pub mod state_infrastructure;
pub mod streaming_global;
pub mod template_global;
pub mod tool_api_standard;
pub mod tool_global;
pub mod types;
pub mod workflow_global;

// Re-exports
pub use injection::{GlobalInjector, InjectionCache};
pub use registry::{GlobalRegistry, GlobalRegistryBuilder};
pub use template_global::TemplateGlobal;
pub use types::{GlobalContext, GlobalMetadata, GlobalObject};

use llmspell_core::Result;
use std::sync::Arc;
use tracing::{debug, warn};

/// Register core globals (json, logger, config, debug)
fn register_core_globals(builder: &mut GlobalRegistryBuilder, context: &Arc<GlobalContext>) {
    builder.register(Arc::new(json_global::JsonGlobal::new()));
    builder.register(Arc::new(core::LoggerGlobal::new()));

    // Register ConfigBridgeGlobal if runtime_config is available
    if let Some(runtime_config) =
        context.get_bridge::<llmspell_config::LLMSpellConfig>("runtime_config")
    {
        let permissions = crate::config_bridge::ConfigPermissions::standard();
        builder.register(Arc::new(config_global::ConfigBridgeGlobal::new(
            (*runtime_config).clone(),
            permissions,
        )));
    } else {
        // Fallback to empty Config global if no runtime config
        warn!("No runtime_config available, using empty Config global");
        builder.register(Arc::new(core::ConfigGlobal::new(serde_json::json!({}))));
    }

    builder.register(Arc::new(debug_global::DebugGlobal::new()));
}

/// Register session and artifact globals if `SessionManager` is available
fn register_session_artifacts(
    builder: &mut GlobalRegistryBuilder,
    context: &Arc<GlobalContext>,
) -> Option<Arc<llmspell_kernel::sessions::manager::SessionManager>> {
    let session_manager_opt =
        context.get_bridge::<llmspell_kernel::sessions::manager::SessionManager>("session_manager");

    if let Some(session_manager) = session_manager_opt.clone() {
        let session_bridge = Arc::new(crate::session_bridge::SessionBridge::new(
            session_manager.clone(),
        ));
        let artifact_bridge =
            Arc::new(crate::artifact_bridge::ArtifactBridge::new(session_manager));

        builder.register(Arc::new(session_global::SessionGlobal::new(session_bridge)));
        builder.register(Arc::new(artifact_global::ArtifactGlobal::new(
            artifact_bridge,
        )));
    }

    session_manager_opt
}

/// Extract memory manager from context if available
fn extract_memory_manager(
    context: &Arc<GlobalContext>,
) -> Option<Arc<dyn llmspell_memory::MemoryManager>> {
    context
        .get_bridge::<Arc<dyn llmspell_memory::MemoryManager>>("memory_manager")
        .map(|arc_arc| (*arc_arc).clone())
}

/// Create in-memory fallback memory manager
fn create_fallback_memory_manager() -> Option<Arc<dyn llmspell_memory::MemoryManager>> {
    use llmspell_memory::DefaultMemoryManager;
    use tracing::{debug, info, warn};

    info!("No memory_manager in context, creating in-memory fallback");
    match DefaultMemoryManager::new_in_memory() {
        Ok(manager) => {
            debug!("Created in-memory MemoryManager successfully");
            Some(Arc::new(manager) as Arc<dyn llmspell_memory::MemoryManager>)
        }
        Err(e) => {
            warn!("Failed to create in-memory MemoryManager: {}, Memory/Context globals will not be available", e);
            None
        }
    }
}

/// Register Memory and Context bridges with the given memory manager
fn register_bridges(
    builder: &mut GlobalRegistryBuilder,
    memory_manager: Arc<dyn llmspell_memory::MemoryManager>,
) {
    use tracing::debug;

    // Register Memory global (17th global)
    let memory_bridge = Arc::new(crate::memory_bridge::MemoryBridge::new(
        memory_manager.clone(),
    ));
    builder.register(Arc::new(memory_global::MemoryGlobal::new(memory_bridge)));
    debug!("Registered Memory global (17th)");

    // Register Context global (18th global) - depends on Memory
    let context_bridge = Arc::new(crate::context_bridge::ContextBridge::new(memory_manager));
    builder.register(Arc::new(context_global::ContextGlobal::new(context_bridge)));
    debug!("Registered Context global (18th)");
}

/// Register Memory and Context globals (always available with in-memory fallback)
fn register_memory_context_globals(
    builder: &mut GlobalRegistryBuilder,
    context: &Arc<GlobalContext>,
) {
    use tracing::warn;

    // Try to get memory_manager from context, or create in-memory fallback
    let memory_manager_from_context = extract_memory_manager(context);

    let memory_manager_opt = if memory_manager_from_context.is_some() {
        memory_manager_from_context
    } else {
        create_fallback_memory_manager()
    };

    if let Some(memory_manager) = memory_manager_opt {
        register_bridges(builder, memory_manager);
    } else {
        warn!("Skipping Memory/Context global registration due to initialization failure");
    }
}

/// Register RAG global if all dependencies are available
async fn register_rag_global(
    builder: &mut GlobalRegistryBuilder,
    context: &Arc<GlobalContext>,
    session_manager_opt: Option<Arc<llmspell_kernel::sessions::manager::SessionManager>>,
) {
    // Try to get vector storage from infrastructure
    let vector_storage = context
        .get_bridge::<crate::globals::rag_infrastructure::RAGInfrastructure>("rag_infrastructure")
        .and_then(|infra| infra.vector_storage.clone());

    if let (Some(state_manager), Some(session_manager), Some(multi_tenant_rag)) = (
        context.get_bridge::<llmspell_kernel::state::StateManager>("state_manager"),
        session_manager_opt,
        context.get_bridge::<llmspell_rag::multi_tenant_integration::MultiTenantRAG>(
            "multi_tenant_rag",
        ),
    ) {
        match rag_global::RAGGlobal::with_managers(
            context.registry.clone(),
            context.providers.clone(),
            state_manager,
            session_manager,
            multi_tenant_rag,
            vector_storage,
        )
        .await
        {
            Ok(rag_global) => {
                builder.register(Arc::new(rag_global));
            }
            Err(e) => {
                warn!("Failed to initialize RAG global: {}", e);
            }
        }
    }
}

/// Register hook and tool related globals
fn register_hook_and_tools(
    builder: &mut GlobalRegistryBuilder,
    context: &Arc<GlobalContext>,
) -> Result<()> {
    let hook_bridge = Arc::new(crate::hook_bridge::HookBridge::new(context.clone())?);
    builder.register(Arc::new(hook_global::HookGlobal::new(hook_bridge)));
    builder.register(Arc::new(replay_global::ReplayGlobal::new()));
    builder.register(Arc::new(tool_global::ToolGlobal::new(
        context.registry.clone(),
    )));
    builder.register(Arc::new(provider_global::ProviderGlobal::new(
        context.providers.clone(),
    )));
    Ok(())
}

/// Register agent and workflow globals
async fn register_agent_workflow(
    builder: &mut GlobalRegistryBuilder,
    context: &Arc<GlobalContext>,
) -> Result<()> {
    // Create agent global with state manager if available
    let agent_global = if let Some(state_manager) =
        context.get_bridge::<llmspell_kernel::state::StateManager>("state_manager")
    {
        agent_global::AgentGlobal::with_state_manager(
            context.registry.clone(),
            context.providers.clone(),
            state_manager,
        )
        .await?
    } else {
        agent_global::AgentGlobal::new(context.registry.clone(), context.providers.clone()).await?
    };
    builder.register(Arc::new(agent_global));

    // Get template_executor from context if available
    let template_executor = context
        .get_bridge::<crate::template_bridge::TemplateBridge>("template_bridge")
        .map(|bridge| {
            Arc::clone(&bridge)
                as Arc<dyn llmspell_core::traits::template_executor::TemplateExecutor>
        });

    // Create workflow global with state manager if available
    let template_executor_clone = template_executor.clone();
    let workflow_global = context
        .get_bridge::<llmspell_kernel::state::StateManager>("state_manager")
        .map_or_else(
            || {
                workflow_global::WorkflowGlobal::new(
                    context.registry.clone(),
                    template_executor.clone(),
                )
            },
            |state_manager| {
                workflow_global::WorkflowGlobal::with_state_manager(
                    context.registry.clone(),
                    state_manager,
                    template_executor_clone,
                )
            },
        );
    builder.register(Arc::new(workflow_global));

    Ok(())
}

/// Register template global
async fn register_template_global(
    builder: &mut GlobalRegistryBuilder,
    context: &Arc<GlobalContext>,
) -> Result<()> {
    // Create template registry with builtin templates
    let template_registry = Arc::new(
        llmspell_templates::TemplateRegistry::with_builtin_templates().map_err(|e| {
            llmspell_core::LLMSpellError::Component {
                message: format!("Failed to create template registry: {e}"),
                source: None,
            }
        })?,
    );

    // Get core provider manager (TemplateBridge needs core, not bridge wrapper)
    let core_providers = context.providers.create_core_manager_arc().await?;

    // Get provider configuration for smart dual-path resolution (Task 13.5.7d)
    let provider_config = Arc::new(context.providers.config().clone());

    // Get infrastructure registries from GlobalContext (Phase 12.8.2.13)
    // These were registered by inject_apis and contain the fully-configured infrastructure
    let infra = crate::template_bridge::InfraConfig {
        tool_registry: context
            .get_bridge::<llmspell_tools::ToolRegistry>("tool_registry")
            .expect("tool_registry must be available in GlobalContext"),
        agent_registry: context
            .get_bridge::<llmspell_agents::FactoryRegistry>("agent_registry")
            .expect("agent_registry must be available in GlobalContext"),
        // Note: workflow_factory is stored as Arc<T> in GlobalContext, so get_bridge returns Arc<Arc<T>>
        // We need to extract the inner Arc
        workflow_factory: context
            .get_bridge::<Arc<dyn llmspell_workflows::WorkflowFactory>>("workflow_factory")
            .map(|arc_arc| (*arc_arc).clone())
            .expect("workflow_factory must be available in GlobalContext"),
        // Wire RAG from ScriptRuntime if available (Task 13b.15.6)
        rag: context.get_bridge::<llmspell_rag::multi_tenant_integration::MultiTenantRAG>(
            "multi_tenant_rag",
        ),
    };

    // Create template bridge with optional state and session managers
    let template_bridge = if let (Some(state_manager), Some(session_manager)) = (
        context.get_bridge::<llmspell_kernel::state::StateManager>("state_manager"),
        context.get_bridge::<llmspell_kernel::sessions::manager::SessionManager>("session_manager"),
    ) {
        let managers = crate::template_bridge::Managers {
            state_manager,
            session_manager,
        };
        Arc::new(
            crate::template_bridge::TemplateBridge::with_state_and_session(
                template_registry,
                context.registry.clone(),
                core_providers,
                provider_config,
                infra,
                managers,
            ),
        )
    } else if let Some(state_manager) =
        context.get_bridge::<llmspell_kernel::state::StateManager>("state_manager")
    {
        // Need to clone infra for this branch
        let infra_clone = crate::template_bridge::InfraConfig {
            tool_registry: infra.tool_registry.clone(),
            agent_registry: infra.agent_registry.clone(),
            workflow_factory: infra.workflow_factory.clone(),
            rag: infra.rag.clone(),
        };
        Arc::new(crate::template_bridge::TemplateBridge::with_state_manager(
            template_registry,
            context.registry.clone(),
            core_providers,
            provider_config,
            infra_clone,
            state_manager,
        ))
    } else {
        Arc::new(crate::template_bridge::TemplateBridge::new(
            template_registry,
            context.registry.clone(),
            core_providers,
            provider_config,
            infra,
        ))
    };

    // Add template_bridge to context so workflow registration can access it
    context.set_bridge("template_bridge", template_bridge.clone());

    // Register template global
    builder.register(Arc::new(template_global::TemplateGlobal::new(
        template_bridge,
    )));

    Ok(())
}

/// Create `StateGlobal` with migration support if configured
async fn create_state_global(context: &Arc<GlobalContext>) -> Arc<state_global::StateGlobal> {
    if let Some(runtime_config) =
        context.get_bridge::<llmspell_config::LLMSpellConfig>("runtime_config")
    {
        if runtime_config.runtime.state_persistence.enabled {
            use crate::globals::state_infrastructure::get_or_create_state_infrastructure;
            match get_or_create_state_infrastructure(
                context,
                &runtime_config.runtime.state_persistence,
            )
            .await
            {
                Ok(infrastructure) => {
                    return Arc::new(state_global::StateGlobal::with_full_support(
                        infrastructure.state_manager,
                        infrastructure.migration_engine,
                        infrastructure.schema_registry,
                        infrastructure.backup_manager,
                    ));
                }
                Err(e) => {
                    warn!(
                        "Failed to initialize state infrastructure: {}, falling back to in-memory",
                        e
                    );
                }
            }
        }
    }
    Arc::new(state_global::StateGlobal::new())
}

/// Initialize the standard global registry with all core globals
///
/// # Errors
///
/// Returns an error if:
/// - Global registration fails
/// - Registry building fails
pub async fn create_standard_registry(context: Arc<GlobalContext>) -> Result<GlobalRegistry> {
    let mut builder = GlobalRegistryBuilder::new();

    // Register core globals
    register_core_globals(&mut builder, &context);

    // Create and register StateGlobal
    let state_global = create_state_global(&context).await;

    builder.register(state_global);
    builder.register(Arc::new(core::UtilsGlobal::new()));
    builder.register(Arc::new(event_global::EventGlobal::new()));

    // Register session and artifact globals
    let session_manager_opt = register_session_artifacts(&mut builder, &context);

    // Register Memory and Context globals (always available with in-memory fallback)
    register_memory_context_globals(&mut builder, &context);

    // Register RAG global if dependencies available
    register_rag_global(&mut builder, &context, session_manager_opt).await;

    // Register hook and tool globals
    register_hook_and_tools(&mut builder, &context)?;

    // Register template global FIRST (must happen before agent/workflow registration)
    // because register_agent_workflow needs template_bridge from context
    register_template_global(&mut builder, &context).await?;

    // Register agent and workflow globals (depends on template_bridge being in context)
    register_agent_workflow(&mut builder, &context).await?;

    builder.register(Arc::new(streaming_global::StreamingGlobal::new()));

    // Register LocalLLM global (always - ProviderManager always exists)
    // LocalLLM methods handle empty provider lists gracefully at runtime
    builder.register(Arc::new(local_llm_global::LocalLLMGlobal::new(
        context.providers.create_core_manager_arc().await?,
    )));

    builder.build()
}
