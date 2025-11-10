//! Template registry for discovery and management

use crate::{
    core::{Template, TemplateCategory, TemplateMetadata},
    error::{Result, TemplateError},
};
use dashmap::DashMap;
use std::sync::{Arc, LazyLock};

/// Template registry for storing and discovering templates
///
/// Provides thread-safe template storage with discovery capabilities by ID, category, and tags.
pub struct TemplateRegistry {
    /// Templates by ID
    templates: DashMap<String, Arc<dyn Template>>,
}

impl Default for TemplateRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl TemplateRegistry {
    /// Create a new empty template registry
    pub fn new() -> Self {
        Self {
            templates: DashMap::new(),
        }
    }

    /// Create a registry with built-in templates
    ///
    /// # Errors
    ///
    /// Returns error if built-in template registration fails
    pub fn with_builtin_templates() -> Result<Self> {
        let registry = Self::new();
        registry.register_builtin_templates()?;
        Ok(registry)
    }

    /// Register a template
    ///
    /// # Errors
    ///
    /// Returns error if template with same ID already exists
    pub fn register(&self, template: Arc<dyn Template>) -> Result<()> {
        let id = template.metadata().id.clone();

        if self.templates.contains_key(&id) {
            return Err(TemplateError::AlreadyRegistered(id));
        }

        self.templates.insert(id, template);
        Ok(())
    }

    /// Register a template, replacing existing if present
    pub fn register_or_replace(&self, template: Arc<dyn Template>) {
        let id = template.metadata().id.clone();
        self.templates.insert(id, template);
    }

    /// Unregister a template by ID
    ///
    /// # Errors
    ///
    /// Returns error if template not found
    pub fn unregister(&self, id: &str) -> Result<Arc<dyn Template>> {
        self.templates
            .remove(id)
            .map(|(_, template)| template)
            .ok_or_else(|| TemplateError::NotFound(id.to_string()))
    }

    /// Get a template by ID
    ///
    /// # Errors
    ///
    /// Returns error if template not found
    pub fn get(&self, id: &str) -> Result<Arc<dyn Template>> {
        self.templates
            .get(id)
            .map(|entry| Arc::clone(entry.value()))
            .ok_or_else(|| TemplateError::NotFound(id.to_string()))
    }

    /// Check if template exists
    pub fn contains(&self, id: &str) -> bool {
        self.templates.contains_key(id)
    }

    /// Get all template IDs
    pub fn list_ids(&self) -> Vec<String> {
        self.templates
            .iter()
            .map(|entry| entry.key().clone())
            .collect()
    }

    /// Get all template metadata
    pub fn list_metadata(&self) -> Vec<TemplateMetadata> {
        self.templates
            .iter()
            .map(|entry| entry.value().metadata().clone())
            .collect()
    }

    /// Discover templates by category
    pub fn discover_by_category(&self, category: &TemplateCategory) -> Vec<TemplateMetadata> {
        self.templates
            .iter()
            .filter(|entry| &entry.value().metadata().category == category)
            .map(|entry| entry.value().metadata().clone())
            .collect()
    }

    /// Search templates by query (searches name, description, and tags)
    pub fn search(&self, query: &str) -> Vec<TemplateMetadata> {
        let query_lower = query.to_lowercase();

        self.templates
            .iter()
            .filter(|entry| {
                let metadata = entry.value().metadata();
                metadata.name.to_lowercase().contains(&query_lower)
                    || metadata.description.to_lowercase().contains(&query_lower)
                    || metadata
                        .tags
                        .iter()
                        .any(|tag| tag.to_lowercase().contains(&query_lower))
            })
            .map(|entry| entry.value().metadata().clone())
            .collect()
    }

    /// Find templates by tag
    pub fn find_by_tag(&self, tag: &str) -> Vec<TemplateMetadata> {
        let tag_lower = tag.to_lowercase();

        self.templates
            .iter()
            .filter(|entry| {
                entry
                    .value()
                    .metadata()
                    .tags
                    .iter()
                    .any(|t| t.to_lowercase() == tag_lower)
            })
            .map(|entry| entry.value().metadata().clone())
            .collect()
    }

    /// Get number of registered templates
    pub fn count(&self) -> usize {
        self.templates.len()
    }

    /// Check if registry is empty
    pub fn is_empty(&self) -> bool {
        self.templates.is_empty()
    }

    /// Clear all templates
    pub fn clear(&self) {
        self.templates.clear();
    }

    /// Register all built-in templates
    ///
    /// This will be populated with actual templates in later implementation
    fn register_builtin_templates(&self) -> Result<()> {
        crate::builtin::register_builtin_templates(self)
    }
}

/// Global template registry
static GLOBAL_REGISTRY: LazyLock<TemplateRegistry> = LazyLock::new(|| {
    TemplateRegistry::with_builtin_templates()
        .expect("Failed to initialize global template registry")
});

/// Get the global template registry
pub fn global_registry() -> &'static TemplateRegistry {
    &GLOBAL_REGISTRY
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::context::ExecutionContext;
    use crate::core::{CostEstimate, TemplateOutput, TemplateParams};
    use crate::validation::ConfigSchema;
    use async_trait::async_trait;

    // Mock template for testing
    #[derive(Debug)]
    struct MockTemplate {
        metadata: TemplateMetadata,
    }

    impl MockTemplate {
        fn new(id: &str, name: &str, category: TemplateCategory) -> Self {
            Self {
                metadata: TemplateMetadata {
                    id: id.to_string(),
                    name: name.to_string(),
                    description: format!("Test template: {}", name),
                    category,
                    version: "0.1.0".to_string(),
                    author: Some("Test".to_string()),
                    requires: vec![],
                    tags: vec!["test".to_string()],
                },
            }
        }
    }

    #[async_trait]
    impl Template for MockTemplate {
        fn metadata(&self) -> &TemplateMetadata {
            &self.metadata
        }

        fn config_schema(&self) -> ConfigSchema {
            ConfigSchema::new(vec![])
        }

        async fn execute(
            &self,
            _params: TemplateParams,
            _context: ExecutionContext,
        ) -> Result<TemplateOutput> {
            unimplemented!("Mock template")
        }

        fn validate(&self, _params: &TemplateParams) -> Result<()> {
            Ok(())
        }

        async fn estimate_cost(&self, _params: &TemplateParams) -> CostEstimate {
            CostEstimate::unknown()
        }
    }

    #[test]
    fn test_registry_register_and_get() {
        let registry = TemplateRegistry::new();
        let template = Arc::new(MockTemplate::new(
            "test-template",
            "Test Template",
            TemplateCategory::Research,
        ));

        registry.register(template.clone()).unwrap();
        assert!(registry.contains("test-template"));
        assert_eq!(registry.count(), 1);

        let retrieved = registry.get("test-template").unwrap();
        assert_eq!(retrieved.metadata().id, "test-template");
    }

    #[test]
    fn test_registry_duplicate_registration() {
        let registry = TemplateRegistry::new();
        let template = Arc::new(MockTemplate::new(
            "test-template",
            "Test Template",
            TemplateCategory::Research,
        ));

        registry.register(template.clone()).unwrap();

        // Second registration should fail
        let result = registry.register(template);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            TemplateError::AlreadyRegistered(_)
        ));
    }

    #[test]
    fn test_registry_register_or_replace() {
        let registry = TemplateRegistry::new();
        let template1 = Arc::new(MockTemplate::new(
            "test-template",
            "Test Template 1",
            TemplateCategory::Research,
        ));
        let template2 = Arc::new(MockTemplate::new(
            "test-template",
            "Test Template 2",
            TemplateCategory::Chat,
        ));

        registry.register_or_replace(template1);
        assert_eq!(
            registry.get("test-template").unwrap().metadata().name,
            "Test Template 1"
        );

        registry.register_or_replace(template2);
        assert_eq!(
            registry.get("test-template").unwrap().metadata().name,
            "Test Template 2"
        );
        assert_eq!(registry.count(), 1); // Still only one template
    }

    #[test]
    fn test_registry_unregister() {
        let registry = TemplateRegistry::new();
        let template = Arc::new(MockTemplate::new(
            "test-template",
            "Test Template",
            TemplateCategory::Research,
        ));

        registry.register(template).unwrap();
        assert_eq!(registry.count(), 1);

        registry.unregister("test-template").unwrap();
        assert_eq!(registry.count(), 0);
        assert!(!registry.contains("test-template"));
    }

    #[test]
    fn test_registry_get_not_found() {
        let registry = TemplateRegistry::new();
        let result = registry.get("nonexistent");
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), TemplateError::NotFound(_)));
    }

    #[test]
    fn test_registry_list_ids() {
        let registry = TemplateRegistry::new();
        registry
            .register(Arc::new(MockTemplate::new(
                "template1",
                "Template 1",
                TemplateCategory::Research,
            )))
            .unwrap();
        registry
            .register(Arc::new(MockTemplate::new(
                "template2",
                "Template 2",
                TemplateCategory::Chat,
            )))
            .unwrap();

        let ids = registry.list_ids();
        assert_eq!(ids.len(), 2);
        assert!(ids.contains(&"template1".to_string()));
        assert!(ids.contains(&"template2".to_string()));
    }

    #[test]
    fn test_registry_discover_by_category() {
        let registry = TemplateRegistry::new();
        registry
            .register(Arc::new(MockTemplate::new(
                "research1",
                "Research 1",
                TemplateCategory::Research,
            )))
            .unwrap();
        registry
            .register(Arc::new(MockTemplate::new(
                "research2",
                "Research 2",
                TemplateCategory::Research,
            )))
            .unwrap();
        registry
            .register(Arc::new(MockTemplate::new(
                "chat1",
                "Chat 1",
                TemplateCategory::Chat,
            )))
            .unwrap();

        let research_templates = registry.discover_by_category(&TemplateCategory::Research);
        assert_eq!(research_templates.len(), 2);

        let chat_templates = registry.discover_by_category(&TemplateCategory::Chat);
        assert_eq!(chat_templates.len(), 1);
    }

    #[test]
    fn test_registry_search() {
        let registry = TemplateRegistry::new();

        let mut template1 = MockTemplate::new(
            "template1",
            "Research Assistant",
            TemplateCategory::Research,
        );
        template1.metadata.tags = vec!["research".to_string(), "assistant".to_string()];

        let mut template2 = MockTemplate::new("template2", "Chat Bot", TemplateCategory::Chat);
        template2.metadata.tags = vec!["chat".to_string(), "conversation".to_string()];

        registry.register(Arc::new(template1)).unwrap();
        registry.register(Arc::new(template2)).unwrap();

        // Search by name
        let results = registry.search("research");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].id, "template1");

        // Search by tag
        let results = registry.search("chat");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].id, "template2");

        // Case-insensitive search
        let results = registry.search("RESEARCH");
        assert_eq!(results.len(), 1);
    }

    #[test]
    fn test_registry_find_by_tag() {
        let registry = TemplateRegistry::new();

        let mut template1 =
            MockTemplate::new("template1", "Template 1", TemplateCategory::Research);
        template1.metadata.tags = vec!["research".to_string(), "advanced".to_string()];

        let mut template2 = MockTemplate::new("template2", "Template 2", TemplateCategory::Chat);
        template2.metadata.tags = vec!["chat".to_string(), "advanced".to_string()];

        registry.register(Arc::new(template1)).unwrap();
        registry.register(Arc::new(template2)).unwrap();

        // Find by single tag
        let results = registry.find_by_tag("research");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].id, "template1");

        // Find by shared tag
        let results = registry.find_by_tag("advanced");
        assert_eq!(results.len(), 2);
    }

    #[test]
    fn test_registry_clear() {
        let registry = TemplateRegistry::new();
        registry
            .register(Arc::new(MockTemplate::new(
                "template1",
                "Template 1",
                TemplateCategory::Research,
            )))
            .unwrap();
        registry
            .register(Arc::new(MockTemplate::new(
                "template2",
                "Template 2",
                TemplateCategory::Chat,
            )))
            .unwrap();

        assert_eq!(registry.count(), 2);

        registry.clear();
        assert_eq!(registry.count(), 0);
        assert!(registry.is_empty());
    }

    #[test]
    fn test_global_registry() {
        let registry = global_registry();
        // Global registry should be initialized (even if empty for now)
        // Just verify we can access it without panic
        let _count = registry.count();
    }
}
