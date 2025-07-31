//! ABOUTME: Global object injection infrastructure for script engines
//! ABOUTME: Provides centralized management of all script-accessible globals

pub mod agent_global;
pub mod artifact_global;
pub mod core;
pub mod event_global;
pub mod hook_global;
pub mod injection;
pub mod json_global;
pub mod registry;
pub mod replay_global;
pub mod session_global;
pub mod state_global;
pub mod state_infrastructure;
pub mod streaming_global;
pub mod tool_global;
pub mod types;
pub mod workflow_global;

// Re-exports
pub use injection::{GlobalInjector, InjectionCache};
pub use registry::{GlobalRegistry, GlobalRegistryBuilder};
pub use types::{GlobalContext, GlobalMetadata, GlobalObject};

use llmspell_core::Result;
use std::sync::Arc;

/// Initialize the standard global registry with all core globals
pub async fn create_standard_registry(context: Arc<GlobalContext>) -> Result<GlobalRegistry> {
    let mut builder = GlobalRegistryBuilder::new();

    // Register core globals in dependency order
    builder.register(Arc::new(json_global::JsonGlobal::new()));
    builder.register(Arc::new(core::LoggerGlobal::new()));
    builder.register(Arc::new(core::ConfigGlobal::new(serde_json::json!({}))));

    // Create StateGlobal with migration support if configured
    let state_global = if let Some(runtime_config) =
        context.get_bridge::<crate::runtime::RuntimeConfig>("runtime_config")
    {
        if runtime_config.runtime.state_persistence.enabled {
            // Initialize state infrastructure
            use crate::globals::state_infrastructure::get_or_create_state_infrastructure;
            match get_or_create_state_infrastructure(
                &context,
                &runtime_config.runtime.state_persistence,
            )
            .await
            {
                Ok(infrastructure) => Arc::new(state_global::StateGlobal::with_full_support(
                    infrastructure.state_manager,
                    infrastructure.migration_engine,
                    infrastructure.schema_registry,
                    infrastructure.backup_manager,
                )),
                Err(e) => {
                    tracing::warn!(
                        "Failed to initialize state infrastructure: {}, falling back to in-memory",
                        e
                    );
                    Arc::new(state_global::StateGlobal::new())
                }
            }
        } else {
            Arc::new(state_global::StateGlobal::new())
        }
    } else {
        Arc::new(state_global::StateGlobal::new())
    };

    builder.register(state_global);
    builder.register(Arc::new(core::UtilsGlobal::new()));
    // TODO: Add Security global when implemented
    builder.register(Arc::new(event_global::EventGlobal::new()));

    // Register Session and Artifact globals if SessionManager is available
    if let Some(session_manager) =
        context.get_bridge::<llmspell_sessions::manager::SessionManager>("session_manager")
    {
        // Create bridges externally for consistency with HookBridge pattern
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

    // Create HookBridge for hook system integration
    let hook_bridge = Arc::new(crate::hook_bridge::HookBridge::new(context.clone()).await?);
    builder.register(Arc::new(hook_global::HookGlobal::new(hook_bridge.clone())));

    // Register replay global for hook debugging
    builder.register(Arc::new(replay_global::ReplayGlobal::new()));

    builder.register(Arc::new(tool_global::ToolGlobal::new(
        context.registry.clone(),
    )));

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

    builder.register(Arc::new(workflow_global::WorkflowGlobal::new(
        context.registry.clone(),
    )));

    builder.register(Arc::new(streaming_global::StreamingGlobal::new()));

    builder.build()
}
