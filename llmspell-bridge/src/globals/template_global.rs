//! ABOUTME: Template global object implementation for script engines
//! ABOUTME: Provides template discovery, inspection, and execution functionality

#[cfg(any(feature = "lua", feature = "javascript"))]
use super::types::GlobalContext;
use super::types::{GlobalMetadata, GlobalObject};
use crate::template_bridge::TemplateBridge;
#[cfg(any(feature = "lua", feature = "javascript"))]
use llmspell_core::Result;
use std::sync::Arc;

/// Template global object for script engines
///
/// Wraps `TemplateBridge` to provide template operations through script engines.
/// Following the Agent/Workflow pattern, this is a thin wrapper around the bridge
/// which contains all business logic.
pub struct TemplateGlobal {
    /// Template bridge for business logic
    bridge: Arc<TemplateBridge>,
}

impl TemplateGlobal {
    /// Create a new Template global
    ///
    /// # Arguments
    ///
    /// * `bridge` - Template bridge instance containing business logic
    #[must_use]
    pub const fn new(bridge: Arc<TemplateBridge>) -> Self {
        Self { bridge }
    }

    /// Get the template bridge
    ///
    /// # Returns
    ///
    /// Reference to the `Arc<TemplateBridge>` for use in injection functions
    #[must_use]
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
        crate::javascript::globals::template::inject_template_global(ctx, context).map_err(|e| {
            llmspell_core::LLMSpellError::Component {
                message: format!("Failed to inject Template global for JavaScript: {e}"),
                source: None,
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_template_global_metadata() {
        // Create a minimal bridge for testing metadata
        // (Real bridge creation tested in template_bridge.rs)
        let template_registry = Arc::new(
            llmspell_templates::TemplateRegistry::with_builtin_templates()
                .expect("Failed to create template registry"),
        );
        let component_registry = Arc::new(crate::registry::ComponentRegistry::new());

        // Use core provider manager for tests
        let providers = Arc::new(llmspell_providers::ProviderManager::new());

        // Create test infrastructure registries
        let infra = crate::template_bridge::InfraConfig {
            tool_registry: Arc::new(llmspell_tools::ToolRegistry::new()),
            agent_registry: Arc::new(llmspell_agents::FactoryRegistry::new()),
            workflow_factory: Arc::new(llmspell_workflows::factory::DefaultWorkflowFactory::new()),
        };

        // Create default provider config for testing
        let provider_config =
            Arc::new(llmspell_config::providers::ProviderManagerConfig::default());

        let bridge = Arc::new(TemplateBridge::new(
            template_registry,
            component_registry,
            providers,
            provider_config,
            infra,
        ));

        let global = TemplateGlobal::new(bridge);
        let metadata = global.metadata();

        assert_eq!(metadata.name, "Template");
        assert_eq!(metadata.version, "1.0.0");
        assert!(metadata.required);
        assert!(metadata.dependencies.is_empty());
        assert!(!metadata.description.is_empty());
    }

    #[test]
    fn test_template_global_bridge_access() {
        let template_registry = Arc::new(
            llmspell_templates::TemplateRegistry::with_builtin_templates()
                .expect("Failed to create template registry"),
        );
        let component_registry = Arc::new(crate::registry::ComponentRegistry::new());
        let providers = Arc::new(llmspell_providers::ProviderManager::new());

        // Create test infrastructure registries
        let infra = crate::template_bridge::InfraConfig {
            tool_registry: Arc::new(llmspell_tools::ToolRegistry::new()),
            agent_registry: Arc::new(llmspell_agents::FactoryRegistry::new()),
            workflow_factory: Arc::new(llmspell_workflows::factory::DefaultWorkflowFactory::new()),
        };

        // Create default provider config for testing
        let provider_config =
            Arc::new(llmspell_config::providers::ProviderManagerConfig::default());

        let bridge = Arc::new(TemplateBridge::new(
            template_registry,
            component_registry,
            providers,
            provider_config,
            infra,
        ));

        let bridge_clone = bridge.clone();
        let global = TemplateGlobal::new(bridge);

        // Verify bridge access returns the same Arc
        assert!(Arc::ptr_eq(global.bridge(), &bridge_clone));
    }
}
