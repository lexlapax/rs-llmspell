//! ABOUTME: Tool registry for discovery, validation, and management
//! ABOUTME: Provides thread-safe tool registration and capability-based discovery

use crate::lifecycle::{ExecutionMetrics, ToolExecutor, ToolLifecycleConfig};
use llmspell_core::{
    error::LLMSpellError,
    traits::tool::{ResourceLimits, SecurityLevel, SecurityRequirements, Tool, ToolCategory},
    types::{AgentInput, AgentOutput},
    ExecutionContext, Result,
};
use llmspell_hooks::{HookExecutor, HookRegistry};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

// Type alias to simplify the complex tool storage type
type ToolStorage = Arc<RwLock<HashMap<String, Arc<Box<dyn Tool>>>>>;
type MetadataCache = Arc<RwLock<HashMap<String, ToolInfo>>>;
type CategoryIndex = Arc<RwLock<HashMap<ToolCategory, Vec<String>>>>;

/// Metadata about a registered tool
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolInfo {
    /// Tool name
    pub name: String,
    /// Tool description
    pub description: String,
    /// Tool category
    pub category: ToolCategory,
    /// Security level required
    pub security_level: SecurityLevel,
    /// Security requirements
    pub security_requirements: SecurityRequirements,
    /// Resource limits
    pub resource_limits: ResourceLimits,
    /// Tool version
    pub version: String,
    /// Custom metadata
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Capability matcher for tool discovery
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CapabilityMatcher {
    /// Required categories (any of these)
    pub categories: Option<Vec<ToolCategory>>,
    /// Maximum security level allowed
    pub max_security_level: Option<SecurityLevel>,
    /// Required capabilities (custom key-value pairs)
    pub capabilities: HashMap<String, serde_json::Value>,
    /// Text-based search terms
    pub search_terms: Vec<String>,
}

impl CapabilityMatcher {
    /// Create a new capability matcher
    pub fn new() -> Self {
        Self::default()
    }

    /// Match tools by category
    pub fn with_categories(mut self, categories: Vec<ToolCategory>) -> Self {
        self.categories = Some(categories);
        self
    }

    /// Set maximum security level
    pub fn with_max_security_level(mut self, level: SecurityLevel) -> Self {
        self.max_security_level = Some(level);
        self
    }

    /// Add capability requirement
    pub fn with_capability(mut self, key: String, value: serde_json::Value) -> Self {
        self.capabilities.insert(key, value);
        self
    }

    /// Add search terms
    pub fn with_search_terms(mut self, terms: Vec<String>) -> Self {
        self.search_terms = terms;
        self
    }

    /// Check if a tool matches this capability matcher
    pub fn matches(&self, tool_info: &ToolInfo) -> bool {
        // Check category match
        if let Some(ref categories) = self.categories {
            if !categories.contains(&tool_info.category) {
                return false;
            }
        }

        // Check security level
        if let Some(ref max_level) = self.max_security_level {
            if tool_info.security_level > *max_level {
                return false;
            }
        }

        // Check custom capabilities
        for (key, expected_value) in &self.capabilities {
            if let Some(actual_value) = tool_info.metadata.get(key) {
                if actual_value != expected_value {
                    return false;
                }
            } else {
                return false;
            }
        }

        // Check search terms (case-insensitive search in name and description)
        if !self.search_terms.is_empty() {
            let searchable_text =
                format!("{} {}", tool_info.name, tool_info.description).to_lowercase();
            for term in &self.search_terms {
                if !searchable_text.contains(&term.to_lowercase()) {
                    return false;
                }
            }
        }

        true
    }
}

/// Thread-safe tool registry for managing tool instances
pub struct ToolRegistry {
    /// Storage for tool instances
    tools: ToolStorage,
    /// Cached tool metadata for fast lookups
    metadata_cache: MetadataCache,
    /// Category index for fast category-based lookups
    category_index: CategoryIndex,
    /// Hook-enabled tool executor
    tool_executor: Option<Arc<ToolExecutor>>,
    /// Hook executor configuration
    hook_config: ToolLifecycleConfig,
}

impl ToolRegistry {
    /// Create a new tool registry
    pub fn new() -> Self {
        Self {
            tools: Arc::new(RwLock::new(HashMap::new())),
            metadata_cache: Arc::new(RwLock::new(HashMap::new())),
            category_index: Arc::new(RwLock::new(HashMap::new())),
            tool_executor: None,
            hook_config: ToolLifecycleConfig::default(),
        }
    }

    /// Create a new tool registry with hook support
    pub fn with_hooks(
        hook_executor: Option<Arc<HookExecutor>>,
        hook_registry: Option<Arc<HookRegistry>>,
        hook_config: ToolLifecycleConfig,
    ) -> Self {
        let tool_executor = if hook_config.enable_hooks {
            Some(Arc::new(ToolExecutor::new(
                hook_config.clone(),
                hook_executor,
                hook_registry,
            )))
        } else {
            None
        };

        Self {
            tools: Arc::new(RwLock::new(HashMap::new())),
            metadata_cache: Arc::new(RwLock::new(HashMap::new())),
            category_index: Arc::new(RwLock::new(HashMap::new())),
            tool_executor,
            hook_config,
        }
    }

    /// Register a tool in the registry
    pub async fn register<T>(&self, name: String, tool: T) -> Result<()>
    where
        T: Tool + 'static,
    {
        // Validate the tool before registration
        self.validate_tool(&tool).await?;

        let tool_arc = Arc::new(Box::new(tool) as Box<dyn Tool>);

        // Extract metadata
        let metadata = tool_arc.metadata();
        let schema = tool_arc.schema();
        let category = tool_arc.category();
        let security_level = tool_arc.security_level();
        let security_requirements = tool_arc.security_requirements();
        let resource_limits = tool_arc.resource_limits();

        let tool_info = ToolInfo {
            name: name.clone(),
            description: schema.description.clone(),
            category: category.clone(),
            security_level,
            security_requirements,
            resource_limits,
            version: metadata.version.to_string(),
            metadata: HashMap::new(), // TODO: Extract custom metadata from component metadata
        };

        // Register the tool
        {
            let mut tools = self.tools.write().await;
            tools.insert(name.clone(), tool_arc);
        }

        // Cache metadata
        {
            let mut cache = self.metadata_cache.write().await;
            cache.insert(name.clone(), tool_info);
        }

        // Update category index
        {
            let mut index = self.category_index.write().await;
            index.entry(category).or_insert_with(Vec::new).push(name);
        }

        Ok(())
    }

    /// Validate a tool before registration
    async fn validate_tool(&self, tool: &dyn Tool) -> Result<()> {
        // Check that the tool has a valid schema
        let schema = tool.schema();
        if schema.name.is_empty() {
            return Err(LLMSpellError::Validation {
                message: "Tool schema name cannot be empty".to_string(),
                field: Some("name".to_string()),
            });
        }

        if schema.description.is_empty() {
            return Err(LLMSpellError::Validation {
                message: "Tool schema description cannot be empty".to_string(),
                field: Some("description".to_string()),
            });
        }

        // Check that metadata is valid
        let metadata = tool.metadata();
        if metadata.name.is_empty() {
            return Err(LLMSpellError::Validation {
                message: "Tool metadata name cannot be empty".to_string(),
                field: Some("metadata.name".to_string()),
            });
        }

        // Validate security requirements
        let security_reqs = tool.security_requirements();
        if security_reqs.level != tool.security_level() {
            return Err(LLMSpellError::Validation {
                message: "Security requirements level must match tool security level".to_string(),
                field: Some("security_requirements.level".to_string()),
            });
        }

        // TODO: Add more validation as needed

        Ok(())
    }

    /// Get a tool by name
    pub async fn get_tool(&self, name: &str) -> Option<Arc<Box<dyn Tool>>> {
        let tools = self.tools.read().await;
        tools.get(name).cloned()
    }

    /// Get tool metadata by name
    pub async fn get_tool_info(&self, name: &str) -> Option<ToolInfo> {
        let cache = self.metadata_cache.read().await;
        cache.get(name).cloned()
    }

    /// List all registered tools
    pub async fn list_tools(&self) -> Vec<String> {
        let tools = self.tools.read().await;
        tools.keys().cloned().collect()
    }

    /// Get tools by category
    pub async fn get_tools_by_category(&self, category: &ToolCategory) -> Vec<String> {
        let index = self.category_index.read().await;
        index.get(category).cloned().unwrap_or_default()
    }

    /// Discover tools by capabilities
    pub async fn discover_tools(&self, matcher: &CapabilityMatcher) -> Vec<ToolInfo> {
        let cache = self.metadata_cache.read().await;
        cache
            .values()
            .filter(|tool_info| matcher.matches(tool_info))
            .cloned()
            .collect()
    }

    /// Get tools compatible with a security level
    pub async fn get_tools_for_security_level(&self, level: SecurityLevel) -> Vec<ToolInfo> {
        let matcher = CapabilityMatcher::new().with_max_security_level(level);
        self.discover_tools(&matcher).await
    }

    /// Check if a tool is registered
    pub async fn contains_tool(&self, name: &str) -> bool {
        let tools = self.tools.read().await;
        tools.contains_key(name)
    }

    /// Unregister a tool
    pub async fn unregister_tool(&self, name: &str) -> Result<()> {
        // Get tool info before removal for category cleanup
        let tool_info = {
            let cache = self.metadata_cache.read().await;
            cache.get(name).cloned()
        };

        // Remove from tools storage
        {
            let mut tools = self.tools.write().await;
            tools.remove(name);
        }

        // Remove from metadata cache
        {
            let mut cache = self.metadata_cache.write().await;
            cache.remove(name);
        }

        // Update category index
        if let Some(info) = tool_info {
            let mut index = self.category_index.write().await;
            if let Some(tools_in_category) = index.get_mut(&info.category) {
                tools_in_category.retain(|tool_name| tool_name != name);
                if tools_in_category.is_empty() {
                    index.remove(&info.category);
                }
            }
        }

        Ok(())
    }

    /// Get registry statistics
    pub async fn get_statistics(&self) -> RegistryStatistics {
        let tools = self.tools.read().await;
        let category_index = self.category_index.read().await;

        let mut category_counts = HashMap::new();
        let mut security_level_counts = HashMap::new();

        {
            let cache = self.metadata_cache.read().await;
            for tool_info in cache.values() {
                *category_counts
                    .entry(tool_info.category.clone())
                    .or_insert(0) += 1;
                *security_level_counts
                    .entry(tool_info.security_level.clone())
                    .or_insert(0) += 1;
            }
        }

        RegistryStatistics {
            total_tools: tools.len(),
            total_categories: category_index.len(),
            category_counts,
            security_level_counts,
        }
    }

    /// Execute a tool with hook integration (if enabled)
    pub async fn execute_tool_with_hooks(
        &self,
        tool_name: &str,
        input: AgentInput,
        context: ExecutionContext,
    ) -> Result<AgentOutput> {
        let tool = self
            .get_tool(tool_name)
            .await
            .ok_or_else(|| LLMSpellError::Component {
                message: format!("Tool '{}' not found in registry", tool_name),
                source: None,
            })?;

        if let Some(ref executor) = self.tool_executor {
            // Execute with hooks
            executor
                .execute_tool_with_hooks(tool.as_ref().as_ref(), input, context)
                .await
        } else {
            // Execute directly without hooks
            tool.execute(input, context).await
        }
    }

    /// Execute a tool by name (with or without hooks based on configuration)
    pub async fn execute_tool(
        &self,
        tool_name: &str,
        input: AgentInput,
        context: ExecutionContext,
    ) -> Result<AgentOutput> {
        self.execute_tool_with_hooks(tool_name, input, context)
            .await
    }

    /// Check if hook integration is enabled
    pub fn has_hook_integration(&self) -> bool {
        self.tool_executor.is_some()
    }

    /// Get the tool executor (if hook integration is enabled)
    pub fn get_tool_executor(&self) -> Option<Arc<ToolExecutor>> {
        self.tool_executor.clone()
    }

    /// Get hook configuration
    pub fn get_hook_config(&self) -> &ToolLifecycleConfig {
        &self.hook_config
    }

    /// Enable or disable hook integration
    pub fn set_hook_integration_enabled(&mut self, enabled: bool) {
        self.hook_config.enable_hooks = enabled;
        // Note: This only changes the config - to create/destroy the ToolExecutor
        // would require recreating the registry with the new configuration
    }

    /// Get execution metrics from the tool executor
    pub fn get_execution_metrics(&self) -> Option<ExecutionMetrics> {
        self.tool_executor
            .as_ref()
            .map(|executor| executor.get_execution_metrics())
    }

    /// Get resource usage statistics for all tool executions
    pub async fn get_resource_usage_stats(&self) -> ResourceUsageStats {
        let stats = self.get_statistics().await;
        let execution_metrics = self.get_execution_metrics().unwrap_or_default();

        ResourceUsageStats {
            total_tools: stats.total_tools,
            tools_with_hooks: if self.has_hook_integration() {
                stats.total_tools
            } else {
                0
            },
            total_executions: execution_metrics.total_executions,
            total_hook_overhead_ms: execution_metrics.hook_overhead_ms,
            average_memory_usage: execution_metrics.average_memory_usage,
            average_cpu_time: execution_metrics.average_cpu_time,
            resource_limits_hit: execution_metrics.resource_limits_hit,
        }
    }
}

impl Default for ToolRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Statistics about the tool registry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistryStatistics {
    /// Total number of registered tools
    pub total_tools: usize,
    /// Total number of categories with tools
    pub total_categories: usize,
    /// Count of tools per category
    pub category_counts: HashMap<ToolCategory, usize>,
    /// Count of tools per security level
    pub security_level_counts: HashMap<SecurityLevel, usize>,
}

/// Resource usage statistics for tools with hook integration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceUsageStats {
    /// Total number of tools in registry
    pub total_tools: usize,
    /// Number of tools with hook integration enabled
    pub tools_with_hooks: usize,
    /// Total tool executions
    pub total_executions: u64,
    /// Total overhead from hook execution in milliseconds
    pub total_hook_overhead_ms: u64,
    /// Average memory usage across all executions
    pub average_memory_usage: usize,
    /// Average CPU time across all executions
    pub average_cpu_time: u64,
    /// Number of times resource limits were hit
    pub resource_limits_hit: u64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;
    use llmspell_core::{
        traits::{
            base_agent::BaseAgent,
            tool::{ParameterDef, ParameterType, ToolSchema},
        },
        types::{AgentInput, AgentOutput},
        ComponentMetadata, ExecutionContext,
    };

    // Mock tool for testing
    struct MockTool {
        metadata: ComponentMetadata,
        category: ToolCategory,
        security_level: SecurityLevel,
        name: String,
    }

    impl MockTool {
        fn new(name: String, category: ToolCategory, security_level: SecurityLevel) -> Self {
            Self {
                metadata: ComponentMetadata::new(name.clone(), format!("Mock tool: {}", name)),
                category,
                security_level,
                name,
            }
        }
    }

    #[async_trait]
    impl BaseAgent for MockTool {
        fn metadata(&self) -> &ComponentMetadata {
            &self.metadata
        }

        async fn execute(
            &self,
            _input: AgentInput,
            _context: ExecutionContext,
        ) -> Result<AgentOutput> {
            Ok(AgentOutput::text(format!("Executed {}", self.name)))
        }

        async fn validate_input(&self, _input: &AgentInput) -> Result<()> {
            Ok(())
        }

        async fn handle_error(&self, error: LLMSpellError) -> Result<AgentOutput> {
            Ok(AgentOutput::text(format!("Error: {}", error)))
        }
    }

    #[async_trait]
    impl Tool for MockTool {
        fn category(&self) -> ToolCategory {
            self.category.clone()
        }

        fn security_level(&self) -> SecurityLevel {
            self.security_level.clone()
        }

        fn schema(&self) -> ToolSchema {
            ToolSchema::new(
                self.name.clone(),
                format!("A mock tool named {}", self.name),
            )
            .with_parameter(ParameterDef {
                name: "input".to_string(),
                param_type: ParameterType::String,
                description: "Input parameter".to_string(),
                required: true,
                default: None,
            })
            .with_returns(ParameterType::String)
        }
    }
    #[tokio::test]
    async fn test_tool_registration() {
        let registry = ToolRegistry::new();

        let tool = MockTool::new(
            "test_tool".to_string(),
            ToolCategory::Utility,
            SecurityLevel::Safe,
        );
        let result = registry.register("test_tool".to_string(), tool).await;
        assert!(result.is_ok());

        // Check that tool is registered
        assert!(registry.contains_tool("test_tool").await);

        // Check that we can retrieve it
        let retrieved = registry.get_tool("test_tool").await;
        assert!(retrieved.is_some());

        // Check metadata
        let info = registry.get_tool_info("test_tool").await;
        assert!(info.is_some());
        let info = info.unwrap();
        assert_eq!(info.name, "test_tool");
        assert_eq!(info.category, ToolCategory::Utility);
        assert_eq!(info.security_level, SecurityLevel::Safe);
    }
    #[tokio::test]
    async fn test_tool_discovery() {
        let registry = ToolRegistry::new();

        // Register tools with different categories and security levels
        let tools = vec![
            (
                "file_tool",
                ToolCategory::Filesystem,
                SecurityLevel::Restricted,
            ),
            ("web_tool", ToolCategory::Web, SecurityLevel::Safe),
            ("data_tool", ToolCategory::Data, SecurityLevel::Privileged),
            ("util_tool", ToolCategory::Utility, SecurityLevel::Safe),
        ];

        for (name, category, level) in tools {
            let tool = MockTool::new(name.to_string(), category, level);
            registry.register(name.to_string(), tool).await.unwrap();
        }

        // Test category-based discovery
        let filesystem_tools = registry
            .get_tools_by_category(&ToolCategory::Filesystem)
            .await;
        assert_eq!(filesystem_tools.len(), 1);
        assert!(filesystem_tools.contains(&"file_tool".to_string()));

        // Test capability-based discovery
        let safe_tools = registry
            .get_tools_for_security_level(SecurityLevel::Safe)
            .await;
        assert_eq!(safe_tools.len(), 2); // web_tool and util_tool

        // Test search with matcher
        let matcher = CapabilityMatcher::new()
            .with_categories(vec![ToolCategory::Web, ToolCategory::Utility])
            .with_max_security_level(SecurityLevel::Safe);

        let discovered = registry.discover_tools(&matcher).await;
        assert_eq!(discovered.len(), 2);

        let names: Vec<String> = discovered.iter().map(|t| t.name.clone()).collect();
        assert!(names.contains(&"web_tool".to_string()));
        assert!(names.contains(&"util_tool".to_string()));
    }
    #[tokio::test]
    async fn test_tool_validation() {
        let registry = ToolRegistry::new();

        // Tool with empty name should fail validation
        struct InvalidTool {
            metadata: ComponentMetadata,
        }

        impl InvalidTool {
            fn new() -> Self {
                // Create invalid metadata with empty name
                let mut metadata = ComponentMetadata::new("".to_string(), "".to_string());
                metadata.name = "".to_string(); // Force empty name
                Self { metadata }
            }
        }

        #[async_trait]
        impl BaseAgent for InvalidTool {
            fn metadata(&self) -> &ComponentMetadata {
                &self.metadata
            }

            async fn execute(
                &self,
                _input: AgentInput,
                _context: ExecutionContext,
            ) -> Result<AgentOutput> {
                Ok(AgentOutput::text("test".to_string()))
            }

            async fn validate_input(&self, _input: &AgentInput) -> Result<()> {
                Ok(())
            }

            async fn handle_error(&self, error: LLMSpellError) -> Result<AgentOutput> {
                Ok(AgentOutput::text(format!("Error: {}", error)))
            }
        }

        #[async_trait]
        impl Tool for InvalidTool {
            fn category(&self) -> ToolCategory {
                ToolCategory::Utility
            }

            fn security_level(&self) -> SecurityLevel {
                SecurityLevel::Safe
            }

            fn schema(&self) -> ToolSchema {
                ToolSchema::new("".to_string(), "".to_string()) // Empty name and description
            }
        }

        let invalid_tool = InvalidTool::new();
        let result = registry.register("invalid".to_string(), invalid_tool).await;
        assert!(result.is_err());
    }
    #[tokio::test]
    async fn test_tool_unregistration() {
        let registry = ToolRegistry::new();

        let tool = MockTool::new(
            "temp_tool".to_string(),
            ToolCategory::Utility,
            SecurityLevel::Safe,
        );
        registry
            .register("temp_tool".to_string(), tool)
            .await
            .unwrap();

        // Verify it's registered
        assert!(registry.contains_tool("temp_tool").await);

        // Unregister it
        let result = registry.unregister_tool("temp_tool").await;
        assert!(result.is_ok());

        // Verify it's gone
        assert!(!registry.contains_tool("temp_tool").await);
        assert!(registry.get_tool("temp_tool").await.is_none());
        assert!(registry.get_tool_info("temp_tool").await.is_none());
    }
    #[tokio::test]
    async fn test_registry_statistics() {
        let registry = ToolRegistry::new();

        let tools = vec![
            ("file1", ToolCategory::Filesystem, SecurityLevel::Safe),
            ("file2", ToolCategory::Filesystem, SecurityLevel::Restricted),
            ("web1", ToolCategory::Web, SecurityLevel::Safe),
            ("data1", ToolCategory::Data, SecurityLevel::Privileged),
        ];

        for (name, category, level) in tools {
            let tool = MockTool::new(name.to_string(), category, level);
            registry.register(name.to_string(), tool).await.unwrap();
        }

        let stats = registry.get_statistics().await;
        assert_eq!(stats.total_tools, 4);
        assert_eq!(stats.total_categories, 3);
        assert_eq!(stats.category_counts[&ToolCategory::Filesystem], 2);
        assert_eq!(stats.category_counts[&ToolCategory::Web], 1);
        assert_eq!(stats.category_counts[&ToolCategory::Data], 1);
        assert_eq!(stats.security_level_counts[&SecurityLevel::Safe], 2);
        assert_eq!(stats.security_level_counts[&SecurityLevel::Restricted], 1);
        assert_eq!(stats.security_level_counts[&SecurityLevel::Privileged], 1);
    }
    #[tokio::test]
    async fn test_capability_matcher() {
        let tool_info = ToolInfo {
            name: "test_tool".to_string(),
            description: "A test tool for testing".to_string(),
            category: ToolCategory::Utility,
            security_level: SecurityLevel::Safe,
            security_requirements: SecurityRequirements::safe(),
            resource_limits: ResourceLimits::strict(),
            version: "1.0.0".to_string(),
            metadata: {
                let mut map = HashMap::new();
                map.insert("supports_async".to_string(), serde_json::json!(true));
                map
            },
        };

        // Test category match
        let matcher = CapabilityMatcher::new()
            .with_categories(vec![ToolCategory::Utility, ToolCategory::Web]);
        assert!(matcher.matches(&tool_info));

        let matcher = CapabilityMatcher::new().with_categories(vec![ToolCategory::Filesystem]);
        assert!(!matcher.matches(&tool_info));

        // Test security level match
        let matcher = CapabilityMatcher::new().with_max_security_level(SecurityLevel::Safe);
        assert!(matcher.matches(&tool_info));

        let matcher = CapabilityMatcher::new().with_max_security_level(SecurityLevel::Restricted);
        assert!(matcher.matches(&tool_info)); // Safe <= Restricted

        // Test capability match
        let matcher = CapabilityMatcher::new()
            .with_capability("supports_async".to_string(), serde_json::json!(true));
        assert!(matcher.matches(&tool_info));

        let matcher = CapabilityMatcher::new()
            .with_capability("supports_async".to_string(), serde_json::json!(false));
        assert!(!matcher.matches(&tool_info));

        // Test search terms
        let matcher = CapabilityMatcher::new().with_search_terms(vec!["test".to_string()]);
        assert!(matcher.matches(&tool_info));

        let matcher = CapabilityMatcher::new().with_search_terms(vec!["nonexistent".to_string()]);
        assert!(!matcher.matches(&tool_info));
    }
    #[tokio::test]
    async fn test_registry_with_hooks() {
        use llmspell_hooks::{HookExecutor as HookExecutorImpl, HookRegistry};

        let hook_registry = Arc::new(HookRegistry::new());
        let hook_executor = Arc::new(HookExecutorImpl::new());
        let hook_config = ToolLifecycleConfig {
            enable_hooks: true,
            ..Default::default()
        };

        let registry = ToolRegistry::with_hooks(
            Some(hook_executor.clone()),
            Some(hook_registry),
            hook_config,
        );

        // Check that hook integration is enabled
        assert!(registry.has_hook_integration());
        assert!(registry.get_tool_executor().is_some());

        // Register a tool
        let tool = MockTool::new(
            "hook_test_tool".to_string(),
            ToolCategory::Utility,
            SecurityLevel::Safe,
        );
        registry
            .register("hook_test_tool".to_string(), tool)
            .await
            .unwrap();

        // Execute tool with hooks
        let input = AgentInput::text("test input");
        let context = ExecutionContext::default();
        let result = registry
            .execute_tool_with_hooks("hook_test_tool", input.clone(), context.clone())
            .await;

        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.text.contains("Executed hook_test_tool"));

        // Test direct execute_tool method as well
        let result = registry
            .execute_tool("hook_test_tool", input, context)
            .await;
        assert!(result.is_ok());
    }
    #[tokio::test]
    async fn test_registry_without_hooks() {
        let registry = ToolRegistry::new();

        // Check that hook integration is disabled
        assert!(!registry.has_hook_integration());
        assert!(registry.get_tool_executor().is_none());

        // Register a tool
        let tool = MockTool::new(
            "no_hook_tool".to_string(),
            ToolCategory::Utility,
            SecurityLevel::Safe,
        );
        registry
            .register("no_hook_tool".to_string(), tool)
            .await
            .unwrap();

        // Execute tool without hooks
        let input = AgentInput::text("test input");
        let context = ExecutionContext::default();
        let result = registry.execute_tool("no_hook_tool", input, context).await;

        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.text.contains("Executed no_hook_tool"));
    }
    #[tokio::test]
    async fn test_tool_execution_error_handling() {
        let registry = ToolRegistry::new();

        let input = AgentInput::text("test input");
        let context = ExecutionContext::default();

        // Try to execute non-existent tool
        let result = registry
            .execute_tool("nonexistent_tool", input, context)
            .await;

        assert!(result.is_err());
        if let Err(LLMSpellError::Component { message, .. }) = result {
            assert!(message.contains("Tool 'nonexistent_tool' not found"));
        } else {
            panic!("Expected Component error");
        }
    }
    #[tokio::test]
    async fn test_resource_usage_stats_with_hooks() {
        use llmspell_hooks::{HookExecutor as HookExecutorImpl, HookRegistry};

        let hook_registry = Arc::new(HookRegistry::new());
        let hook_executor = Arc::new(HookExecutorImpl::new());
        let hook_config = ToolLifecycleConfig {
            enable_hooks: true,
            ..Default::default()
        };

        let registry =
            ToolRegistry::with_hooks(Some(hook_executor), Some(hook_registry), hook_config);

        // Register multiple tools
        for i in 0..3 {
            let tool = MockTool::new(
                format!("test_tool_{}", i),
                ToolCategory::Utility,
                SecurityLevel::Safe,
            );
            registry
                .register(format!("test_tool_{}", i), tool)
                .await
                .unwrap();
        }

        // Get resource usage stats
        let stats = registry.get_resource_usage_stats().await;

        assert_eq!(stats.total_tools, 3);
        assert_eq!(stats.tools_with_hooks, 3); // All tools have hooks when enabled
        assert_eq!(stats.total_executions, 0); // No executions yet
        assert_eq!(stats.total_hook_overhead_ms, 0);
        assert_eq!(stats.average_memory_usage, 0);
        assert_eq!(stats.average_cpu_time, 0);
        assert_eq!(stats.resource_limits_hit, 0);
    }
    #[tokio::test]
    async fn test_resource_usage_stats_without_hooks() {
        let registry = ToolRegistry::new();

        // Register a tool
        let tool = MockTool::new(
            "test_tool".to_string(),
            ToolCategory::Utility,
            SecurityLevel::Safe,
        );
        registry
            .register("test_tool".to_string(), tool)
            .await
            .unwrap();

        // Get resource usage stats
        let stats = registry.get_resource_usage_stats().await;

        assert_eq!(stats.total_tools, 1);
        assert_eq!(stats.tools_with_hooks, 0); // No hooks enabled
        assert_eq!(stats.total_executions, 0);
    }
    #[tokio::test]
    async fn test_execution_metrics_from_registry() {
        use llmspell_hooks::{HookExecutor as HookExecutorImpl, HookRegistry};

        let hook_registry = Arc::new(HookRegistry::new());
        let hook_executor = Arc::new(HookExecutorImpl::new());
        let hook_config = ToolLifecycleConfig::default();

        let registry =
            ToolRegistry::with_hooks(Some(hook_executor), Some(hook_registry), hook_config);

        // Test that we can get execution metrics (even if empty)
        let metrics = registry.get_execution_metrics();
        assert!(metrics.is_some());

        let metrics = metrics.unwrap();
        assert_eq!(metrics.total_executions, 0);
        assert_eq!(metrics.hook_overhead_ms, 0);
    }
}
