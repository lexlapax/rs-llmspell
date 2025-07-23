//! ABOUTME: Workflow global object implementation for script engines
//! ABOUTME: Provides Workflow creation and orchestration functionality

use super::types::{GlobalContext, GlobalMetadata, GlobalObject};
use crate::workflows::WorkflowBridge;
use crate::ComponentRegistry;
use llmspell_core::Result;
use std::sync::Arc;

/// Workflow global object for script engines
pub struct WorkflowGlobal {
    registry: Arc<ComponentRegistry>,
    bridge: Arc<WorkflowBridge>,
}

impl WorkflowGlobal {
    /// Create a new Workflow global
    pub fn new(registry: Arc<ComponentRegistry>) -> Self {
        let bridge = Arc::new(WorkflowBridge::new(registry.clone()));
        Self { registry, bridge }
    }

    /// Get the component registry
    pub fn registry(&self) -> &Arc<ComponentRegistry> {
        &self.registry
    }

    /// Get the workflow bridge
    pub fn bridge(&self) -> &Arc<WorkflowBridge> {
        &self.bridge
    }
}

impl GlobalObject for WorkflowGlobal {
    fn metadata(&self) -> GlobalMetadata {
        GlobalMetadata {
            name: "Workflow".to_string(),
            description: "Workflow creation and orchestration".to_string(),
            dependencies: vec!["Agent".to_string(), "Tool".to_string()],
            required: true,
            version: "1.0.0".to_string(),
        }
    }

    #[cfg(feature = "lua")]
    fn inject_lua(&self, lua: &mlua::Lua, context: &GlobalContext) -> Result<()> {
        crate::lua::globals::workflow::inject_workflow_global(lua, context, self.bridge.clone())
            .map_err(|e| llmspell_core::LLMSpellError::Component {
                message: format!("Failed to inject Workflow global: {}", e),
                source: None,
            })
    }

    #[cfg(feature = "javascript")]
    fn inject_javascript(
        &self,
        ctx: &mut boa_engine::Context,
        context: &GlobalContext,
    ) -> Result<()> {
        crate::javascript::globals::workflow::inject_workflow_global(ctx, context).map_err(|e| {
            llmspell_core::LLMSpellError::Component {
                message: format!("Failed to inject Workflow global for JavaScript: {}", e),
                source: None,
            }
        })
    }
}
