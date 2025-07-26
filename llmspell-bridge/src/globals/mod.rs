//! ABOUTME: Global object injection infrastructure for script engines
//! ABOUTME: Provides centralized management of all script-accessible globals

pub mod agent_global;
pub mod core;
pub mod event_global;
pub mod hook_global;
pub mod injection;
pub mod json_global;
pub mod registry;
pub mod replay_global;
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
                Ok(infrastructure) => {
                    if let (Some(migration_engine), Some(schema_registry)) = (
                        infrastructure.migration_engine,
                        infrastructure.schema_registry,
                    ) {
                        Arc::new(state_global::StateGlobal::with_migration_support(
                            infrastructure.state_manager,
                            migration_engine,
                            schema_registry,
                        ))
                    } else {
                        Arc::new(state_global::StateGlobal::with_state_manager(
                            infrastructure.state_manager,
                        ))
                    }
                }
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

    // Create HookBridge for hook system integration
    let hook_bridge = Arc::new(crate::hook_bridge::HookBridge::new(context.clone()).await?);
    builder.register(Arc::new(hook_global::HookGlobal::new(hook_bridge.clone())));

    // Register replay global for hook debugging
    builder.register(Arc::new(replay_global::ReplayGlobal::new()));

    builder.register(Arc::new(tool_global::ToolGlobal::new(
        context.registry.clone(),
    )));

    // Create agent global asynchronously
    let agent_global =
        agent_global::AgentGlobal::new(context.registry.clone(), context.providers.clone()).await?;
    builder.register(Arc::new(agent_global));

    builder.register(Arc::new(workflow_global::WorkflowGlobal::new(
        context.registry.clone(),
    )));

    builder.register(Arc::new(streaming_global::StreamingGlobal::new()));

    builder.build()
}
