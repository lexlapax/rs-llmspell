//! ABOUTME: Global object injection infrastructure for script engines
//! ABOUTME: Provides centralized management of all script-accessible globals

pub mod agent_global;
pub mod core;
pub mod event_global;
pub mod hook_global;
pub mod injection;
pub mod json_global;
pub mod registry;
pub mod state_global;
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
    builder.register(Arc::new(state_global::StateGlobal::new()));
    builder.register(Arc::new(core::UtilsGlobal::new()));
    // TODO: Add Security global when implemented
    builder.register(Arc::new(event_global::EventGlobal::new()));
    builder.register(Arc::new(hook_global::HookGlobal::new()));
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
