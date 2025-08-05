//! ABOUTME: Dependency injection container for agent dependencies
//! ABOUTME: Manages tools, providers, and other dependencies for agents

use anyhow::Result;
use llmspell_core::traits::tool::Tool;
#[cfg(test)]
use llmspell_core::traits::tool::{SecurityLevel, ToolCategory, ToolSchema};
use std::{
    any::{Any, TypeId},
    collections::HashMap,
    sync::Arc,
};
use tokio::sync::RwLock;

/// Type-safe dependency injection container
pub struct DIContainer {
    /// Tool registry
    tools: Arc<RwLock<HashMap<String, Arc<dyn Tool>>>>,

    /// Generic service registry
    services: Arc<RwLock<HashMap<TypeId, Box<dyn Any + Send + Sync>>>>,

    /// Named instances registry
    named_instances: Arc<RwLock<HashMap<String, Box<dyn Any + Send + Sync>>>>,
}

impl DIContainer {
    /// Create a new dependency injection container
    #[must_use]
    pub fn new() -> Self {
        Self {
            tools: Arc::new(RwLock::new(HashMap::new())),
            services: Arc::new(RwLock::new(HashMap::new())),
            named_instances: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Register a tool
    ///
    /// # Errors
    ///
    /// Returns an error if a tool with the same ID is already registered
    pub async fn register_tool(&self, id: String, tool: Arc<dyn Tool>) -> Result<()> {
        let mut tools = self.tools.write().await;
        if tools.contains_key(&id) {
            anyhow::bail!("Tool with id '{}' already registered", id);
        }
        tools.insert(id, tool);
        Ok(())
    }

    /// Get a tool by ID
    pub async fn get_tool(&self, id: &str) -> Option<Arc<dyn Tool>> {
        let tools = self.tools.read().await;
        tools.get(id).cloned()
    }

    /// List all registered tool IDs
    pub async fn list_tools(&self) -> Vec<String> {
        let tools = self.tools.read().await;
        tools.keys().cloned().collect()
    }

    /// Register a service by type
    ///
    /// # Errors
    ///
    /// Returns an error if a service of the same type is already registered
    pub async fn register_service<T: Any + Send + Sync + 'static>(&self, service: T) -> Result<()> {
        let type_id = TypeId::of::<T>();
        let mut services = self.services.write().await;
        if services.contains_key(&type_id) {
            anyhow::bail!(
                "Service of type {} already registered",
                std::any::type_name::<T>()
            );
        }
        services.insert(type_id, Box::new(service));
        Ok(())
    }

    /// Get a service by type
    pub async fn get_service<T: Any + Send + Sync + Clone + 'static>(&self) -> Option<Arc<T>> {
        let type_id = TypeId::of::<T>();
        let services = self.services.read().await;
        services
            .get(&type_id)
            .and_then(|service| service.downcast_ref::<T>().map(|s| Arc::new(s.clone())))
    }

    /// Register a named instance
    ///
    /// # Errors
    ///
    /// Returns an error if a named instance with the same name is already registered
    pub async fn register_named<T: Any + Send + Sync + 'static>(
        &self,
        name: String,
        instance: T,
    ) -> Result<()> {
        let mut instances = self.named_instances.write().await;
        if instances.contains_key(&name) {
            anyhow::bail!("Named instance '{}' already registered", name);
        }
        instances.insert(name, Box::new(instance));
        Ok(())
    }

    /// Get a named instance
    pub async fn get_named<T: Any + Send + Sync + Clone + 'static>(
        &self,
        name: &str,
    ) -> Option<Arc<T>> {
        let instances = self.named_instances.read().await;
        instances
            .get(name)
            .and_then(|instance| instance.downcast_ref::<T>().map(|i| Arc::new(i.clone())))
    }

    /// Create a scoped container with additional dependencies
    #[must_use]
    pub fn create_scope(&self) -> ScopedDIContainer {
        ScopedDIContainer {
            parent: self,
            scoped_services: Arc::new(RwLock::new(HashMap::new())),
            _scoped_instances: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

impl Default for DIContainer {
    fn default() -> Self {
        Self::new()
    }
}

/// Scoped dependency injection container
pub struct ScopedDIContainer<'a> {
    parent: &'a DIContainer,
    scoped_services: Arc<RwLock<HashMap<TypeId, Box<dyn Any + Send + Sync>>>>,
    _scoped_instances: Arc<RwLock<HashMap<String, Box<dyn Any + Send + Sync>>>>,
}

impl ScopedDIContainer<'_> {
    /// Register a scoped service
    ///
    /// # Errors
    ///
    /// Returns an error if service registration fails
    pub async fn register_scoped<T: Any + Send + Sync + 'static>(&self, service: T) -> Result<()> {
        let type_id = TypeId::of::<T>();
        let mut services = self.scoped_services.write().await;
        services.insert(type_id, Box::new(service));
        Ok(())
    }

    /// Get a service (checks scope first, then parent)
    pub async fn get_service<T: Any + Send + Sync + Clone + 'static>(&self) -> Option<Arc<T>> {
        let type_id = TypeId::of::<T>();

        // Check scoped services first
        let scoped = self.scoped_services.read().await;
        if let Some(service) = scoped.get(&type_id) {
            return service.downcast_ref::<T>().map(|s| Arc::new(s.clone()));
        }

        // Fall back to parent
        self.parent.get_service::<T>().await
    }

    /// Get a tool (delegates to parent)
    pub async fn get_tool(&self, id: &str) -> Option<Arc<dyn Tool>> {
        self.parent.get_tool(id).await
    }
}

/// Builder for dependency injection container
pub struct DIContainerBuilder {
    container: DIContainer,
}

impl DIContainerBuilder {
    /// Create a new builder
    #[must_use]
    pub fn new() -> Self {
        Self {
            container: DIContainer::new(),
        }
    }

    /// Add a tool to the container
    pub fn with_tool(self, id: String, tool: Arc<dyn Tool>) -> Self {
        // Use blocking to register during building
        tokio::task::block_in_place(move || {
            tokio::runtime::Handle::current().block_on(async {
                self.container.register_tool(id, tool).await.unwrap();
                self
            })
        })
    }

    /// Build the container
    #[must_use]
    pub fn build(self) -> DIContainer {
        self.container
    }
}

impl Default for DIContainerBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use llmspell_core::{
        traits::base_agent::BaseAgent, types::*, ComponentMetadata, ExecutionContext,
    };

    // Mock tool for testing
    #[derive(Clone)]
    struct MockTool {
        id: String,
    }

    #[async_trait::async_trait]
    impl BaseAgent for MockTool {
        async fn execute(
            &self,
            _input: AgentInput,
            _context: ExecutionContext,
        ) -> Result<AgentOutput, llmspell_core::LLMSpellError> {
            Ok(AgentOutput::text("Mock output"))
        }

        fn metadata(&self) -> &ComponentMetadata {
            // In a real implementation, this would return actual metadata
            unimplemented!("Mock metadata")
        }

        async fn validate_input(
            &self,
            _input: &AgentInput,
        ) -> Result<(), llmspell_core::LLMSpellError> {
            Ok(())
        }

        async fn handle_error(
            &self,
            error: llmspell_core::LLMSpellError,
        ) -> Result<AgentOutput, llmspell_core::LLMSpellError> {
            Err(error)
        }
    }

    #[async_trait::async_trait]
    impl Tool for MockTool {
        fn category(&self) -> ToolCategory {
            ToolCategory::Utility
        }

        fn security_level(&self) -> SecurityLevel {
            SecurityLevel::Safe
        }

        fn schema(&self) -> ToolSchema {
            ToolSchema::new(self.id.clone(), "Mock tool for testing".to_string())
        }
    }
    #[tokio::test]
    async fn test_tool_registration() {
        let container = DIContainer::new();
        let tool = Arc::new(MockTool {
            id: "test-tool".to_string(),
        });

        // Register tool
        container
            .register_tool("test-tool".to_string(), tool.clone())
            .await
            .unwrap();

        // Get tool
        let retrieved = container.get_tool("test-tool").await;
        assert!(retrieved.is_some());

        // List tools
        let tools = container.list_tools().await;
        assert_eq!(tools, vec!["test-tool"]);

        // Duplicate registration should fail
        let result = container.register_tool("test-tool".to_string(), tool).await;
        assert!(result.is_err());
    }

    #[derive(Clone)]
    struct TestService {
        #[allow(dead_code)]
        value: String,
    }
    #[tokio::test]
    async fn test_service_registration() {
        let container = DIContainer::new();
        let service = TestService {
            value: "test".to_string(),
        };

        // Register service
        container.register_service(service.clone()).await.unwrap();

        // Get service
        let retrieved = container.get_service::<TestService>().await;
        assert!(retrieved.is_some());

        // Duplicate registration should fail
        let result = container.register_service(service).await;
        assert!(result.is_err());
    }
    #[tokio::test]
    async fn test_scoped_container() {
        let parent = DIContainer::new();
        let parent_service = TestService {
            value: "parent".to_string(),
        };
        parent.register_service(parent_service).await.unwrap();

        let scope = parent.create_scope();
        let scoped_service = TestService {
            value: "scoped".to_string(),
        };
        scope.register_scoped(scoped_service).await.unwrap();

        // Scoped service should override parent
        let service = scope.get_service::<TestService>().await;
        assert!(service.is_some());
    }
    #[tokio::test]
    async fn test_named_instances() {
        let container = DIContainer::new();

        let service1 = TestService {
            value: "service1".to_string(),
        };
        let service2 = TestService {
            value: "service2".to_string(),
        };

        // Register named instances
        container
            .register_named("svc1".to_string(), service1)
            .await
            .unwrap();
        container
            .register_named("svc2".to_string(), service2)
            .await
            .unwrap();

        // Get named instances
        let retrieved1 = container.get_named::<TestService>("svc1").await;
        let retrieved2 = container.get_named::<TestService>("svc2").await;

        assert!(retrieved1.is_some());
        assert!(retrieved2.is_some());

        // Non-existent instance
        let none = container.get_named::<TestService>("non-existent").await;
        assert!(none.is_none());

        // Duplicate registration should fail
        let service3 = TestService {
            value: "service3".to_string(),
        };
        let result = container.register_named("svc1".to_string(), service3).await;
        assert!(result.is_err());
    }
    #[tokio::test]
    async fn test_di_builder() {
        let tool = Arc::new(MockTool {
            id: "builder-tool".to_string(),
        });

        // DIContainerBuilder uses blocking, so we need to spawn it
        let container = tokio::task::spawn_blocking(move || {
            DIContainerBuilder::new()
                .with_tool("builder-tool".to_string(), tool)
                .build()
        })
        .await
        .unwrap();

        let retrieved = container.get_tool("builder-tool").await;
        assert!(retrieved.is_some());
    }
    #[tokio::test]
    async fn test_scoped_tool_access() {
        let parent = DIContainer::new();
        let tool = Arc::new(MockTool {
            id: "parent-tool".to_string(),
        });

        parent
            .register_tool("parent-tool".to_string(), tool)
            .await
            .unwrap();

        let scope = parent.create_scope();

        // Scoped container should have access to parent's tools
        let retrieved = scope.get_tool("parent-tool").await;
        assert!(retrieved.is_some());
    }
}
