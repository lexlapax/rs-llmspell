//! ABOUTME: Business logic bridge for template operations
//! ABOUTME: Centralizes `ExecutionContext` building, parameter validation, and template discovery

use llmspell_core::LLMSpellError;
use llmspell_templates::{
    ConfigSchema, TemplateCategory, TemplateMetadata, TemplateOutput, TemplateParams,
    TemplateRegistry,
};
use std::sync::Arc;

/// Bridge between scripts and template system
///
/// Provides business logic layer for template operations including:
/// - Template discovery and search
/// - Parameter validation with schema constraints
/// - `ExecutionContext` building from infrastructure components
/// - Template execution with proper context
/// - Cost estimation
pub struct TemplateBridge {
    /// Template registry (from llmspell-templates)
    template_registry: Arc<TemplateRegistry>,
    /// Component registry for `ExecutionContext` building
    #[allow(dead_code)] // Reserved for future enhancement: component discovery
    registry: Arc<crate::registry::ComponentRegistry>,
    /// Provider manager for LLM access
    providers: Arc<llmspell_providers::ProviderManager>,
    /// Optional state manager for stateful templates
    state_manager: Option<Arc<llmspell_kernel::state::StateManager>>,
    /// Optional session manager for session-based templates
    session_manager: Option<Arc<llmspell_kernel::sessions::manager::SessionManager>>,
}

impl TemplateBridge {
    /// Create new template bridge
    ///
    /// # Arguments
    ///
    /// * `template_registry` - Registry containing available templates
    /// * `registry` - Component registry for infrastructure access
    /// * `providers` - Provider manager for LLM operations
    #[must_use]
    pub const fn new(
        template_registry: Arc<TemplateRegistry>,
        registry: Arc<crate::registry::ComponentRegistry>,
        providers: Arc<llmspell_providers::ProviderManager>,
    ) -> Self {
        Self {
            template_registry,
            registry,
            providers,
            state_manager: None,
            session_manager: None,
        }
    }

    /// Create with state manager support
    ///
    /// # Arguments
    ///
    /// * `template_registry` - Registry containing available templates
    /// * `registry` - Component registry for infrastructure access
    /// * `providers` - Provider manager for LLM operations
    /// * `state_manager` - State manager for persistent storage
    #[must_use]
    pub const fn with_state_manager(
        template_registry: Arc<TemplateRegistry>,
        registry: Arc<crate::registry::ComponentRegistry>,
        providers: Arc<llmspell_providers::ProviderManager>,
        state_manager: Arc<llmspell_kernel::state::StateManager>,
    ) -> Self {
        Self {
            template_registry,
            registry,
            providers,
            state_manager: Some(state_manager),
            session_manager: None,
        }
    }

    /// Create with both state and session manager support
    ///
    /// # Arguments
    ///
    /// * `template_registry` - Registry containing available templates
    /// * `registry` - Component registry for infrastructure access
    /// * `providers` - Provider manager for LLM operations
    /// * `state_manager` - State manager for persistent storage
    /// * `session_manager` - Session manager for session-based operations
    #[must_use]
    pub const fn with_state_and_session(
        template_registry: Arc<TemplateRegistry>,
        registry: Arc<crate::registry::ComponentRegistry>,
        providers: Arc<llmspell_providers::ProviderManager>,
        state_manager: Arc<llmspell_kernel::state::StateManager>,
        session_manager: Arc<llmspell_kernel::sessions::manager::SessionManager>,
    ) -> Self {
        Self {
            template_registry,
            registry,
            providers,
            state_manager: Some(state_manager),
            session_manager: Some(session_manager),
        }
    }

    /// List templates by optional category
    ///
    /// # Arguments
    ///
    /// * `category` - Optional category filter (Research, Chat, Analysis, etc.)
    ///
    /// # Returns
    ///
    /// Vector of template metadata matching the category filter
    #[must_use]
    pub fn list_templates(&self, category: Option<TemplateCategory>) -> Vec<TemplateMetadata> {
        category.map_or_else(
            || self.template_registry.list_metadata(),
            |cat| self.template_registry.discover_by_category(&cat),
        )
    }

    /// Get template info with optional schema
    ///
    /// # Arguments
    ///
    /// * `name` - Template identifier
    /// * `include_schema` - Whether to include parameter schema in response
    ///
    /// # Returns
    ///
    /// Template metadata and optional schema
    ///
    /// # Errors
    ///
    /// Returns error if template not found
    pub fn get_template_info(
        &self,
        name: &str,
        include_schema: bool,
    ) -> Result<TemplateInfo, LLMSpellError> {
        let template = self
            .template_registry
            .get(name)
            .map_err(|e| LLMSpellError::Component {
                message: format!("Template not found: {e}"),
                source: None,
            })?;

        let metadata = template.metadata().clone();
        let schema = if include_schema {
            Some(template.config_schema())
        } else {
            None
        };

        Ok(TemplateInfo { metadata, schema })
    }

    /// Execute template with parameters
    ///
    /// This is the CORE method that centralizes `ExecutionContext` building.
    /// It validates parameters, builds a complete `ExecutionContext` with all
    /// infrastructure components, and executes the template.
    ///
    /// # Arguments
    ///
    /// * `name` - Template identifier
    /// * `params` - Template parameters (validated against schema)
    ///
    /// # Returns
    ///
    /// Template execution output including result, artifacts, and metrics
    ///
    /// # Errors
    ///
    /// Returns error if:
    /// - Template not found
    /// - Parameter validation fails
    /// - `ExecutionContext` building fails
    /// - Template execution fails
    pub async fn execute_template(
        &self,
        name: &str,
        params: TemplateParams,
    ) -> Result<TemplateOutput, LLMSpellError> {
        // Get template
        let template = self
            .template_registry
            .get(name)
            .map_err(|e| LLMSpellError::Component {
                message: format!("Template not found: {e}"),
                source: None,
            })?;

        // Validate parameters against schema
        template
            .validate(&params)
            .map_err(|e| LLMSpellError::Validation {
                field: Some("params".to_string()),
                message: format!("Parameter validation failed: {e}"),
            })?;

        // Build ExecutionContext from available infrastructure
        // NOTE: For Phase 12.5, we create minimal registries. Future phases can enhance this.
        let mut context_builder = llmspell_templates::ExecutionContext::builder()
            .with_tool_registry(Arc::new(llmspell_tools::ToolRegistry::new()))
            .with_agent_registry(Arc::new(llmspell_agents::FactoryRegistry::new()))
            .with_workflow_factory(Arc::new(
                llmspell_workflows::factory::DefaultWorkflowFactory::new(),
            ))
            .with_providers(self.providers.clone());

        // Add optional components
        if let Some(state_mgr) = &self.state_manager {
            context_builder = context_builder.with_state_manager(state_mgr.clone());
        }

        if let Some(session_mgr) = &self.session_manager {
            context_builder = context_builder.with_session_manager(session_mgr.clone());
        }

        let exec_context = context_builder
            .build()
            .map_err(|e| LLMSpellError::Component {
                message: format!("Failed to build execution context: {e}"),
                source: None,
            })?;

        // Execute template
        template
            .execute(params, exec_context)
            .await
            .map_err(|e| LLMSpellError::Component {
                message: format!("Template execution failed: {e}"),
                source: None,
            })
    }

    /// Search templates by query and optional category
    ///
    /// # Arguments
    ///
    /// * `query` - Search query (matches name, description, tags)
    /// * `category` - Optional category filter
    ///
    /// # Returns
    ///
    /// Vector of matching template metadata
    #[must_use]
    pub fn search_templates(
        &self,
        query: &str,
        category: Option<TemplateCategory>,
    ) -> Vec<TemplateMetadata> {
        let mut results = self.template_registry.search(query);

        // Filter by category if provided
        if let Some(cat) = category {
            results.retain(|metadata| metadata.category == cat);
        }

        results
    }

    /// Get template parameter schema
    ///
    /// # Arguments
    ///
    /// * `name` - Template identifier
    ///
    /// # Returns
    ///
    /// Parameter schema with constraints
    ///
    /// # Errors
    ///
    /// Returns error if template not found
    pub fn get_template_schema(&self, name: &str) -> Result<ConfigSchema, LLMSpellError> {
        let template = self
            .template_registry
            .get(name)
            .map_err(|e| LLMSpellError::Component {
                message: format!("Template not found: {e}"),
                source: None,
            })?;

        Ok(template.config_schema())
    }

    /// Estimate template execution cost
    ///
    /// # Arguments
    ///
    /// * `name` - Template identifier
    /// * `params` - Template parameters for cost estimation
    ///
    /// # Returns
    ///
    /// Optional cost estimate (None if template doesn't support estimation)
    ///
    /// # Errors
    ///
    /// Returns error if template not found
    pub async fn estimate_cost(
        &self,
        name: &str,
        params: &TemplateParams,
    ) -> Result<Option<llmspell_templates::CostEstimate>, LLMSpellError> {
        let template = self
            .template_registry
            .get(name)
            .map_err(|e| LLMSpellError::Component {
                message: format!("Template not found: {e}"),
                source: None,
            })?;

        let estimate = template.estimate_cost(params).await;
        Ok(Some(estimate))
    }
}

/// Template info with optional schema
///
/// Returned by `get_template_info()` to provide template metadata
/// and optionally the parameter schema.
#[derive(Debug, Clone)]
pub struct TemplateInfo {
    /// Template metadata (name, description, category, etc.)
    pub metadata: TemplateMetadata,
    /// Parameter schema (optional, only if requested)
    pub schema: Option<ConfigSchema>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_template_bridge_creation() {
        let template_registry = Arc::new(
            TemplateRegistry::with_builtin_templates().expect("Failed to create template registry"),
        );
        let component_registry = Arc::new(crate::registry::ComponentRegistry::new());
        let providers = Arc::new(llmspell_providers::ProviderManager::new());

        let bridge = TemplateBridge::new(template_registry, component_registry, providers);

        // Verify we can list templates
        let templates = bridge.list_templates(None);
        assert!(!templates.is_empty(), "Should have built-in templates");
    }

    #[test]
    fn test_list_templates_by_category() {
        let template_registry = Arc::new(
            TemplateRegistry::with_builtin_templates().expect("Failed to create template registry"),
        );
        let component_registry = Arc::new(crate::registry::ComponentRegistry::new());
        let providers = Arc::new(llmspell_providers::ProviderManager::new());

        let bridge = TemplateBridge::new(template_registry, component_registry, providers);

        // List all templates
        let all_templates = bridge.list_templates(None);
        assert!(!all_templates.is_empty());

        // List by category (should be subset or equal)
        let research_templates = bridge.list_templates(Some(TemplateCategory::Research));
        assert!(research_templates.len() <= all_templates.len());
    }

    #[test]
    fn test_get_template_info() {
        let template_registry = Arc::new(
            TemplateRegistry::with_builtin_templates().expect("Failed to create template registry"),
        );
        let component_registry = Arc::new(crate::registry::ComponentRegistry::new());
        let providers = Arc::new(llmspell_providers::ProviderManager::new());

        let bridge = TemplateBridge::new(template_registry, component_registry, providers);

        // Get first template
        let templates = bridge.list_templates(None);
        if let Some(first_template) = templates.first() {
            let info = bridge
                .get_template_info(&first_template.id, true)
                .expect("Should get template info");
            assert_eq!(info.metadata.id, first_template.id);
            assert!(info.schema.is_some(), "Schema should be included");

            // Test without schema
            let info_no_schema = bridge
                .get_template_info(&first_template.id, false)
                .expect("Should get template info");
            assert!(
                info_no_schema.schema.is_none(),
                "Schema should not be included"
            );
        }
    }

    #[test]
    fn test_search_templates() {
        let template_registry = Arc::new(
            TemplateRegistry::with_builtin_templates().expect("Failed to create template registry"),
        );
        let component_registry = Arc::new(crate::registry::ComponentRegistry::new());
        let providers = Arc::new(llmspell_providers::ProviderManager::new());

        let bridge = TemplateBridge::new(template_registry, component_registry, providers);

        // Search for "research" - should find research assistant template
        let results = bridge.search_templates("research", None);
        assert!(
            !results.is_empty(),
            "Should find templates matching 'research'"
        );
    }

    #[test]
    fn test_get_template_schema() {
        let template_registry = Arc::new(
            TemplateRegistry::with_builtin_templates().expect("Failed to create template registry"),
        );
        let component_registry = Arc::new(crate::registry::ComponentRegistry::new());
        let providers = Arc::new(llmspell_providers::ProviderManager::new());

        let bridge = TemplateBridge::new(template_registry, component_registry, providers);

        // Get first template's schema
        let templates = bridge.list_templates(None);
        if let Some(first_template) = templates.first() {
            let schema = bridge
                .get_template_schema(&first_template.id)
                .expect("Should get template schema");
            assert!(
                !schema.parameters.is_empty(),
                "Schema should have parameters"
            );
        }
    }
}
