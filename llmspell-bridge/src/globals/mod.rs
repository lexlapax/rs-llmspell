//! ABOUTME: Global object injection infrastructure for script engines
//! ABOUTME: Provides centralized management of all script-accessible globals

pub mod agent_global;
pub mod core;
pub mod injection;
pub mod registry;
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
pub fn create_standard_registry(context: Arc<GlobalContext>) -> Result<GlobalRegistry> {
    let mut builder = GlobalRegistryBuilder::new();

    // Register core globals in dependency order
    // TODO: Add JSON global when implemented
    builder.register(Arc::new(core::LoggerGlobal::new()));
    builder.register(Arc::new(core::ConfigGlobal::new(serde_json::json!({}))));
    // TODO: Add State global when implemented
    builder.register(Arc::new(core::UtilsGlobal::new()));
    // TODO: Add Security global when implemented
    // TODO: Add Event global when implemented
    // TODO: Add Hook global when implemented
    builder.register(Arc::new(core::ToolGlobal::new(context.registry.clone())));
    builder.register(Arc::new(core::AgentGlobal::new(
        context.registry.clone(),
        context.providers.clone(),
    )));
    builder.register(Arc::new(workflow_global::WorkflowGlobal::new(
        context.registry.clone(),
    )));

    builder.build()
}
