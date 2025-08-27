//! ABOUTME: Global object injection infrastructure for script engines
//! ABOUTME: Provides centralized management of all script-accessible globals

pub mod agent_global;
pub mod artifact_global;
pub mod config_global;
pub mod core;
pub mod debug_global;
pub mod event_global;
pub mod hook_global;
pub mod injection;
pub mod json_global;
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
pub mod tool_api_standard;
pub mod tool_global;
pub mod types;
pub mod workflow_global;

// Re-exports
pub use injection::{GlobalInjector, InjectionCache};
pub use registry::{GlobalRegistry, GlobalRegistryBuilder};
pub use types::{GlobalContext, GlobalMetadata, GlobalObject};

use llmspell_core::Result;
use std::sync::Arc;

/// Register core globals (json, logger, config, debug)
fn register_core_globals(builder: &mut GlobalRegistryBuilder) {
    builder.register(Arc::new(json_global::JsonGlobal::new()));
    builder.register(Arc::new(core::LoggerGlobal::new()));
    builder.register(Arc::new(core::ConfigGlobal::new(serde_json::json!({}))));
    builder.register(Arc::new(debug_global::DebugGlobal::new()));
}

/// Register session and artifact globals if `SessionManager` is available
fn register_session_artifacts(
    builder: &mut GlobalRegistryBuilder,
    context: &Arc<GlobalContext>,
) -> Option<Arc<llmspell_sessions::manager::SessionManager>> {
    let session_manager_opt =
        context.get_bridge::<llmspell_sessions::manager::SessionManager>("session_manager");

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

/// Register RAG global if all dependencies are available
async fn register_rag_global(
    builder: &mut GlobalRegistryBuilder,
    context: &Arc<GlobalContext>,
    session_manager_opt: Option<Arc<llmspell_sessions::manager::SessionManager>>,
) {
    // Try to get vector storage from infrastructure
    let vector_storage = context
        .get_bridge::<crate::globals::rag_infrastructure::RAGInfrastructure>("rag_infrastructure")
        .and_then(|infra| infra.vector_storage.clone());

    if let (Some(state_manager), Some(session_manager), Some(multi_tenant_rag)) = (
        context.get_bridge::<llmspell_state_persistence::StateManager>("state_manager"),
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
                tracing::warn!("Failed to initialize RAG global: {}", e);
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
        context.get_bridge::<llmspell_state_persistence::StateManager>("state_manager")
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

    // Create workflow global with state manager if available
    let workflow_global = context
        .get_bridge::<llmspell_state_persistence::StateManager>("state_manager")
        .map_or_else(
            || workflow_global::WorkflowGlobal::new(context.registry.clone()),
            |state_manager| {
                workflow_global::WorkflowGlobal::with_state_manager(
                    context.registry.clone(),
                    state_manager,
                )
            },
        );
    builder.register(Arc::new(workflow_global));

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
                    tracing::warn!(
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
    register_core_globals(&mut builder);

    // Create and register StateGlobal
    let state_global = create_state_global(&context).await;

    builder.register(state_global);
    builder.register(Arc::new(core::UtilsGlobal::new()));
    builder.register(Arc::new(event_global::EventGlobal::new()));

    // Register session and artifact globals
    let session_manager_opt = register_session_artifacts(&mut builder, &context);

    // Register RAG global if dependencies available
    register_rag_global(&mut builder, &context, session_manager_opt).await;

    // Register hook and tool globals
    register_hook_and_tools(&mut builder, &context)?;

    // Register agent and workflow globals
    register_agent_workflow(&mut builder, &context).await?;

    builder.register(Arc::new(streaming_global::StreamingGlobal::new()));

    builder.build()
}
