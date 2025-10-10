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
use std::time::Instant;
use tokio::sync::RwLock;
use tracing::{debug, error, info, instrument, trace};

// Type alias to simplify the complex tool storage type
type ToolStorage = Arc<RwLock<HashMap<String, Arc<Box<dyn Tool>>>>>;
type MetadataCache = Arc<RwLock<HashMap<String, ToolInfo>>>;
type CategoryIndex = Arc<RwLock<HashMap<ToolCategory, Vec<String>>>>;
type AliasIndex = Arc<RwLock<HashMap<String, String>>>; // alias -> primary_name

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
    /// Aliases for backward compatibility (alternative names for the same tool)
    pub aliases: Vec<String>,
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
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Match tools by category
    #[must_use]
    pub fn with_categories(mut self, categories: Vec<ToolCategory>) -> Self {
        self.categories = Some(categories);
        self
    }

    /// Set maximum security level
    #[must_use]
    pub const fn with_max_security_level(mut self, level: SecurityLevel) -> Self {
        self.max_security_level = Some(level);
        self
    }

    /// Add capability requirement
    #[must_use]
    pub fn with_capability(mut self, key: String, value: serde_json::Value) -> Self {
        self.capabilities.insert(key, value);
        self
    }

    /// Add search terms
    #[must_use]
    pub fn with_search_terms(mut self, terms: Vec<String>) -> Self {
        self.search_terms = terms;
        self
    }

    /// Check if a tool matches this capability matcher
    #[must_use]
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
    /// Alias index for backward compatibility (alias -> `primary_name`)
    alias_index: AliasIndex,
    /// Hook-enabled tool executor
    tool_executor: Option<Arc<ToolExecutor>>,
    /// Hook executor configuration
    hook_config: ToolLifecycleConfig,
}

impl ToolRegistry {
    /// Create a new tool registry
    #[must_use]
    pub fn new() -> Self {
        debug!("Creating new tool registry without hooks");
        Self {
            tools: Arc::new(RwLock::new(HashMap::new())),
            metadata_cache: Arc::new(RwLock::new(HashMap::new())),
            category_index: Arc::new(RwLock::new(HashMap::new())),
            alias_index: Arc::new(RwLock::new(HashMap::new())),
            tool_executor: None,
            hook_config: ToolLifecycleConfig::default(),
        }
    }

    /// Create a new tool registry with hook support
    #[must_use]
    pub fn with_hooks(
        hook_executor: Option<Arc<HookExecutor>>,
        hook_registry: Option<Arc<HookRegistry>>,
        hook_config: ToolLifecycleConfig,
    ) -> Self {
        debug!(
            hooks_enabled = hook_config.features.hooks_enabled,
            "Creating new tool registry with hook configuration"
        );
        let tool_executor = if hook_config.features.hooks_enabled {
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
            alias_index: Arc::new(RwLock::new(HashMap::new())),
            tool_executor,
            hook_config,
        }
    }

    /// Register a tool with aliases in the registry
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Tool validation fails
    /// - A tool with the same name already exists
    /// - Any alias conflicts with an existing tool name or alias
    /// - Primary name is included in aliases
    #[instrument(skip(self, tool))]
    pub async fn register_with_aliases<T>(
        &self,
        name: String,
        aliases: Vec<String>,
        tool: T,
    ) -> Result<()>
    where
        T: Tool + 'static,
    {
        info!(
            tool_name = %name,
            alias_count = aliases.len(),
            "Registering tool with aliases in registry"
        );

        // Validate aliases before registration
        {
            let tools = self.tools.read().await;
            let alias_index = self.alias_index.read().await;

            // Check that primary name doesn't already exist
            if tools.contains_key(&name) {
                return Err(LLMSpellError::Validation {
                    message: format!("Tool with name '{name}' already exists"),
                    field: Some("name".to_string()),
                });
            }

            // Check that primary name is not in aliases list
            if aliases.contains(&name) {
                return Err(LLMSpellError::Validation {
                    message: "Primary name cannot be included in aliases list".to_string(),
                    field: Some("aliases".to_string()),
                });
            }

            // Check for alias conflicts with existing tools and aliases
            for alias in &aliases {
                // Check if alias conflicts with existing primary name
                if tools.contains_key(alias) {
                    return Err(LLMSpellError::Validation {
                        message: format!("Alias '{alias}' conflicts with existing tool name"),
                        field: Some("aliases".to_string()),
                    });
                }
            }

            // Drop tools lock early since we're done with it
            drop(tools);

            // Check for alias conflicts with existing aliases
            for alias in &aliases {
                if alias_index.contains_key(alias) {
                    return Err(LLMSpellError::Validation {
                        message: format!("Alias '{alias}' is already registered as an alias"),
                        field: Some("aliases".to_string()),
                    });
                }
            }

            // Check for duplicate aliases in the input
            let mut unique_aliases = std::collections::HashSet::new();
            for alias in &aliases {
                if !unique_aliases.insert(alias) {
                    return Err(LLMSpellError::Validation {
                        message: format!("Duplicate alias '{alias}' in aliases list"),
                        field: Some("aliases".to_string()),
                    });
                }
            }
        }

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
            aliases: aliases.clone(),
        };

        // Register the tool
        {
            let mut tools = self.tools.write().await;
            tools.insert(name.clone(), tool_arc);
        }

        // Register aliases
        {
            let mut alias_index = self.alias_index.write().await;
            for alias in &aliases {
                alias_index.insert(alias.clone(), name.clone());
            }
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

    /// Register a tool in the registry (without aliases)
    ///
    /// # Errors
    ///
    /// Returns an error if tool validation fails or if a tool with the same name already exists
    #[instrument(skip(self, tool))]
    pub async fn register<T>(&self, name: String, tool: T) -> Result<()>
    where
        T: Tool + 'static,
    {
        // Delegate to register_with_aliases with empty aliases vec
        self.register_with_aliases(name, Vec::new(), tool).await
    }

    /// Resolve a tool name to its primary name (checking aliases if needed)
    /// Returns the primary name if found, None otherwise
    #[instrument(skip(self))]
    async fn resolve_tool_name(&self, name: &str) -> Option<String> {
        trace!(
            tool_name = %name,
            "Resolving tool name (checking for aliases)"
        );

        // First check if it's a primary name
        {
            let tools = self.tools.read().await;
            if tools.contains_key(name) {
                return Some(name.to_string());
            }
        }

        // Check if it's an alias
        {
            let aliases = self.alias_index.read().await;
            if let Some(primary_name) = aliases.get(name) {
                return Some(primary_name.clone());
            }
        }

        None
    }

    /// Validate a tool before registration
    #[allow(clippy::unused_async)]
    #[instrument(skip(self, tool))]
    async fn validate_tool(&self, tool: &dyn Tool) -> Result<()> {
        debug!(
            tool_name = %tool.metadata().name,
            "Validating tool before registration"
        );
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

    /// Get a tool by name (supports both primary names and aliases)
    #[instrument(skip(self))]
    pub async fn get_tool(&self, name: &str) -> Option<Arc<Box<dyn Tool>>> {
        debug!(
            tool_name = %name,
            "Looking up tool in registry"
        );

        // Resolve name to primary name (handles aliases)
        let primary_name = self.resolve_tool_name(name).await?;

        let tools = self.tools.read().await;
        tools.get(&primary_name).cloned()
    }

    /// Get tool metadata by name (supports both primary names and aliases)
    #[instrument(skip_all)]
    pub async fn get_tool_info(&self, name: &str) -> Option<ToolInfo> {
        trace!(
            tool_name = %name,
            "Getting tool metadata from registry"
        );

        // Resolve name to primary name (handles aliases)
        let primary_name = self.resolve_tool_name(name).await?;

        let cache = self.metadata_cache.read().await;
        cache.get(&primary_name).cloned()
    }

    /// List all registered tools
    #[instrument(skip(self))]
    pub async fn list_tools(&self) -> Vec<String> {
        trace!("Listing all registered tools");
        let tools = self.tools.read().await;
        tools.keys().cloned().collect()
    }

    /// Get tools by category
    #[instrument(skip(self))]
    pub async fn get_tools_by_category(&self, category: &ToolCategory) -> Vec<String> {
        debug!(
            category = ?category,
            "Searching tools by category"
        );
        let index = self.category_index.read().await;
        index.get(category).cloned().unwrap_or_default()
    }

    /// Discover tools by capabilities
    #[instrument(skip_all)]
    pub async fn discover_tools(&self, matcher: &CapabilityMatcher) -> Vec<ToolInfo> {
        debug!(
            categories = ?matcher.categories,
            search_terms = ?matcher.search_terms,
            "Discovering tools by capabilities"
        );
        let cache = self.metadata_cache.read().await;
        cache
            .values()
            .filter(|tool_info| matcher.matches(tool_info))
            .cloned()
            .collect()
    }

    /// Get tools compatible with a security level
    #[instrument(skip_all)]
    pub async fn get_tools_for_security_level(&self, level: SecurityLevel) -> Vec<ToolInfo> {
        debug!(
            security_level = ?level,
            "Getting tools for security level"
        );
        let matcher = CapabilityMatcher::new().with_max_security_level(level);
        self.discover_tools(&matcher).await
    }

    /// Check if a tool is registered (supports both primary names and aliases)
    #[instrument(skip(self))]
    pub async fn contains_tool(&self, name: &str) -> bool {
        trace!(
            tool_name = %name,
            "Checking if tool exists in registry"
        );
        self.resolve_tool_name(name).await.is_some()
    }

    /// Unregister a tool (removes tool and all its aliases)
    ///
    /// # Errors
    ///
    /// Returns an error if the tool is not found in the registry
    #[instrument(skip(self))]
    pub async fn unregister_tool(&self, name: &str) -> Result<()> {
        info!(
            tool_name = %name,
            "Unregistering tool from registry"
        );
        // Get tool info before removal for category cleanup and alias removal
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

        // Remove aliases from alias index
        if let Some(ref info) = tool_info {
            let mut alias_index = self.alias_index.write().await;
            for alias in &info.aliases {
                alias_index.remove(alias);
            }
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
    #[instrument(skip(self))]
    pub async fn get_statistics(&self) -> RegistryStatistics {
        trace!("Getting registry statistics");
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

    /// Get tool from registry with error handling
    #[instrument(skip(self))]
    async fn get_tool_or_error(&self, tool_name: &str) -> Result<Arc<Box<dyn Tool>>> {
        self.get_tool(tool_name)
            .await
            .ok_or_else(|| LLMSpellError::Component {
                message: format!("Tool '{tool_name}' not found in registry"),
                source: None,
            })
    }

    /// Execute tool with or without hooks based on configuration
    #[instrument(skip(self, tool))]
    async fn execute_with_optional_hooks(
        &self,
        tool: Arc<Box<dyn Tool>>,
        tool_name: &str,
        input: AgentInput,
        context: ExecutionContext,
    ) -> Result<AgentOutput> {
        if let Some(ref executor) = self.tool_executor {
            debug!(
                tool_name = %tool_name,
                "Executing tool with hooks"
            );
            executor
                .execute_tool_with_hooks(tool.as_ref().as_ref(), input, context)
                .await
        } else {
            debug!(
                tool_name = %tool_name,
                "Executing tool without hooks"
            );
            tool.execute(input, context).await
        }
    }

    /// Log tool execution result
    fn log_execution_result(tool_name: &str, result: &Result<AgentOutput>, elapsed_ms: u128) {
        match result {
            Ok(output) => {
                debug!(
                    tool_name = %tool_name,
                    duration_ms = elapsed_ms,
                    output_size = output.text.len(),
                    "Tool execution completed successfully"
                );
            }
            Err(e) => {
                error!(
                    tool_name = %tool_name,
                    duration_ms = elapsed_ms,
                    error = %e,
                    "Tool execution failed"
                );
            }
        }
    }

    /// Execute a tool with hook integration (if enabled)
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Tool not found in registry
    /// - Hook execution fails
    /// - Tool execution fails
    #[instrument(skip(self))]
    pub async fn execute_tool_with_hooks(
        &self,
        tool_name: &str,
        input: AgentInput,
        context: ExecutionContext,
    ) -> Result<AgentOutput> {
        let start = Instant::now();
        info!(
            tool_name = %tool_name,
            input_size = input.text.len(),
            has_hooks = self.tool_executor.is_some(),
            "Executing tool from registry"
        );

        let tool = self.get_tool_or_error(tool_name).await?;
        let result = self
            .execute_with_optional_hooks(tool, tool_name, input, context)
            .await;

        let elapsed_ms = start.elapsed().as_millis();
        Self::log_execution_result(tool_name, &result, elapsed_ms);

        result
    }

    /// Execute a tool by name (with or without hooks based on configuration)
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Tool not found in registry
    /// - Tool execution fails
    /// - Hook execution fails (if hooks are enabled)
    #[instrument(skip(self))]
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
    #[must_use]
    pub const fn has_hook_integration(&self) -> bool {
        self.tool_executor.is_some()
    }

    /// Get the tool executor (if hook integration is enabled)
    #[must_use]
    pub fn get_tool_executor(&self) -> Option<Arc<ToolExecutor>> {
        self.tool_executor.clone()
    }

    /// Get hook configuration
    #[must_use]
    pub const fn get_hook_config(&self) -> &ToolLifecycleConfig {
        &self.hook_config
    }

    /// Enable or disable hook integration
    pub const fn set_hook_integration_enabled(&mut self, enabled: bool) {
        self.hook_config.features.hooks_enabled = enabled;
        // Note: This only changes the config - to create/destroy the ToolExecutor
        // would require recreating the registry with the new configuration
    }

    /// Get execution metrics from the tool executor
    #[must_use]
    pub fn get_execution_metrics(&self) -> Option<ExecutionMetrics> {
        self.tool_executor
            .as_ref()
            .map(|executor| executor.get_execution_metrics())
    }

    /// Get resource usage statistics for all tool executions
    #[instrument(skip(self))]
    pub async fn get_resource_usage_stats(&self) -> ResourceUsageStats {
        trace!("Getting resource usage statistics");
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
        info!(
            tool_name = "tool-registry",
            category = "Tool",
            "Initializing ToolRegistry"
        );
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
    use crate::lifecycle::hook_integration::HookFeatures;
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
                metadata: ComponentMetadata::new(name.clone(), format!("Mock tool: {name}")),
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

        #[instrument(skip(self))]
        async fn execute_impl(
            &self,
            _input: AgentInput,
            _context: ExecutionContext,
        ) -> Result<AgentOutput> {
            Ok(AgentOutput::text(format!("Executed {}", self.name)))
        }

        #[instrument(skip(self))]
        async fn validate_input(&self, _input: &AgentInput) -> Result<()> {
            Ok(())
        }

        #[instrument(skip(self))]
        async fn handle_error(&self, error: LLMSpellError) -> Result<AgentOutput> {
            Ok(AgentOutput::text(format!("Error: {error}")))
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
        // Tool with empty name should fail validation
        struct InvalidTool {
            metadata: ComponentMetadata,
        }

        impl InvalidTool {
            fn new() -> Self {
                // Create invalid metadata with empty name
                let mut metadata = ComponentMetadata::new(String::new(), String::new());
                metadata.name = String::new(); // Force empty name
                Self { metadata }
            }
        }

        #[async_trait]
        impl BaseAgent for InvalidTool {
            fn metadata(&self) -> &ComponentMetadata {
                &self.metadata
            }

            #[instrument(skip(self))]
            async fn execute_impl(
                &self,
                _input: AgentInput,
                _context: ExecutionContext,
            ) -> Result<AgentOutput> {
                Ok(AgentOutput::text("test".to_string()))
            }

            #[instrument(skip(self))]
            async fn validate_input(&self, _input: &AgentInput) -> Result<()> {
                Ok(())
            }

            #[instrument(skip(self))]
            async fn handle_error(&self, error: LLMSpellError) -> Result<AgentOutput> {
                Ok(AgentOutput::text(format!("Error: {error}")))
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
                ToolSchema::new(String::new(), String::new()) // Empty name and description
            }
        }

        let registry = ToolRegistry::new();
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
            aliases: Vec::new(),
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
            features: HookFeatures {
                hooks_enabled: true,
                ..Default::default()
            },
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
        let result = Box::pin(registry.execute_tool("hook_test_tool", input, context)).await;
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
        let result = Box::pin(registry.execute_tool("no_hook_tool", input, context)).await;

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
        let result = Box::pin(registry.execute_tool("nonexistent_tool", input, context)).await;

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
            features: HookFeatures {
                hooks_enabled: true,
                ..Default::default()
            },
            ..Default::default()
        };

        let registry =
            ToolRegistry::with_hooks(Some(hook_executor), Some(hook_registry), hook_config);

        // Register multiple tools
        for i in 0..3 {
            let tool = MockTool::new(
                format!("test_tool_{i}"),
                ToolCategory::Utility,
                SecurityLevel::Safe,
            );
            registry
                .register(format!("test_tool_{i}"), tool)
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

    #[tokio::test]
    async fn test_tool_alias_resolution() {
        let registry = ToolRegistry::new();

        // Register a tool with aliases
        let tool = MockTool::new(
            "primary_tool".to_string(),
            ToolCategory::Utility,
            SecurityLevel::Safe,
        );
        let aliases = vec!["alias1".to_string(), "alias2".to_string()];
        registry
            .register_with_aliases("primary_tool".to_string(), aliases, tool)
            .await
            .unwrap();

        // Test that we can get the tool by primary name
        let tool_by_primary = registry.get_tool("primary_tool").await;
        assert!(tool_by_primary.is_some());

        // Test that we can get the tool by alias1
        let tool_by_alias1 = registry.get_tool("alias1").await;
        assert!(tool_by_alias1.is_some());

        // Test that we can get the tool by alias2
        let tool_by_alias2 = registry.get_tool("alias2").await;
        assert!(tool_by_alias2.is_some());

        // Test that all three return the same tool (unwrap and compare pointers)
        let primary = tool_by_primary.as_ref().unwrap();
        let alias1 = tool_by_alias1.as_ref().unwrap();
        let alias2 = tool_by_alias2.as_ref().unwrap();

        assert!(Arc::ptr_eq(primary, alias1));
        assert!(Arc::ptr_eq(alias1, alias2));

        // Test that contains_tool works for both primary and aliases
        assert!(registry.contains_tool("primary_tool").await);
        assert!(registry.contains_tool("alias1").await);
        assert!(registry.contains_tool("alias2").await);
        assert!(!registry.contains_tool("nonexistent").await);

        // Test that get_tool_info works for both primary and aliases
        let info_primary = registry.get_tool_info("primary_tool").await;
        let info_alias1 = registry.get_tool_info("alias1").await;
        let info_alias2 = registry.get_tool_info("alias2").await;

        assert!(info_primary.is_some());
        assert!(info_alias1.is_some());
        assert!(info_alias2.is_some());

        let info_primary = info_primary.unwrap();
        let info_alias1 = info_alias1.unwrap();
        let info_alias2 = info_alias2.unwrap();

        // All should have the same primary name
        assert_eq!(info_primary.name, "primary_tool");
        assert_eq!(info_alias1.name, "primary_tool");
        assert_eq!(info_alias2.name, "primary_tool");

        // All should have the same aliases
        assert_eq!(info_primary.aliases.len(), 2);
        assert!(info_primary.aliases.contains(&"alias1".to_string()));
        assert!(info_primary.aliases.contains(&"alias2".to_string()));
    }

    #[tokio::test]
    async fn test_tool_registration_with_aliases() {
        let registry = ToolRegistry::new();

        // Test registration with empty aliases
        let tool1 = MockTool::new(
            "tool1".to_string(),
            ToolCategory::Utility,
            SecurityLevel::Safe,
        );
        let result = registry
            .register_with_aliases("tool1".to_string(), Vec::new(), tool1)
            .await;
        assert!(result.is_ok());

        // Test registration with multiple aliases
        let tool2 = MockTool::new(
            "tool2".to_string(),
            ToolCategory::Utility,
            SecurityLevel::Safe,
        );
        let aliases = vec![
            "tool2_alias1".to_string(),
            "tool2_alias2".to_string(),
            "tool2_alias3".to_string(),
        ];
        let result = registry
            .register_with_aliases("tool2".to_string(), aliases, tool2)
            .await;
        assert!(result.is_ok());

        // Verify all aliases work
        assert!(registry.contains_tool("tool2").await);
        assert!(registry.contains_tool("tool2_alias1").await);
        assert!(registry.contains_tool("tool2_alias2").await);
        assert!(registry.contains_tool("tool2_alias3").await);

        // Test that the standard register() method works (uses empty aliases internally)
        let tool3 = MockTool::new(
            "tool3".to_string(),
            ToolCategory::Utility,
            SecurityLevel::Safe,
        );
        let result = registry.register("tool3".to_string(), tool3).await;
        assert!(result.is_ok());

        let info = registry.get_tool_info("tool3").await.unwrap();
        assert_eq!(info.aliases.len(), 0);
    }

    #[tokio::test]
    async fn test_alias_conflict_detection() {
        let registry = ToolRegistry::new();

        // Register first tool with aliases
        let tool1 = MockTool::new(
            "tool1".to_string(),
            ToolCategory::Utility,
            SecurityLevel::Safe,
        );
        registry
            .register_with_aliases(
                "tool1".to_string(),
                vec!["alias1".to_string(), "alias2".to_string()],
                tool1,
            )
            .await
            .unwrap();

        // Try to register a tool with a name that conflicts with existing primary name
        let tool2 = MockTool::new(
            "tool1".to_string(),
            ToolCategory::Utility,
            SecurityLevel::Safe,
        );
        let result = registry
            .register_with_aliases("tool1".to_string(), Vec::new(), tool2)
            .await;
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            LLMSpellError::Validation { .. }
        ));

        // Try to register a tool with an alias that conflicts with existing primary name
        let tool3 = MockTool::new(
            "tool3".to_string(),
            ToolCategory::Utility,
            SecurityLevel::Safe,
        );
        let result = registry
            .register_with_aliases("tool3".to_string(), vec!["tool1".to_string()], tool3)
            .await;
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            LLMSpellError::Validation { .. }
        ));

        // Try to register a tool with an alias that conflicts with existing alias
        let tool4 = MockTool::new(
            "tool4".to_string(),
            ToolCategory::Utility,
            SecurityLevel::Safe,
        );
        let result = registry
            .register_with_aliases("tool4".to_string(), vec!["alias1".to_string()], tool4)
            .await;
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            LLMSpellError::Validation { .. }
        ));

        // Try to register a tool with primary name in aliases list
        let tool5 = MockTool::new(
            "tool5".to_string(),
            ToolCategory::Utility,
            SecurityLevel::Safe,
        );
        let result = registry
            .register_with_aliases("tool5".to_string(), vec!["tool5".to_string()], tool5)
            .await;
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            LLMSpellError::Validation { .. }
        ));

        // Try to register a tool with duplicate aliases
        let tool6 = MockTool::new(
            "tool6".to_string(),
            ToolCategory::Utility,
            SecurityLevel::Safe,
        );
        let result = registry
            .register_with_aliases(
                "tool6".to_string(),
                vec!["dup".to_string(), "dup".to_string()],
                tool6,
            )
            .await;
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            LLMSpellError::Validation { .. }
        ));
    }

    #[tokio::test]
    async fn test_unregister_removes_aliases() {
        let registry = ToolRegistry::new();

        // Register a tool with aliases
        let tool = MockTool::new(
            "tool_with_aliases".to_string(),
            ToolCategory::Utility,
            SecurityLevel::Safe,
        );
        let aliases = vec![
            "alias_a".to_string(),
            "alias_b".to_string(),
            "alias_c".to_string(),
        ];
        registry
            .register_with_aliases("tool_with_aliases".to_string(), aliases, tool)
            .await
            .unwrap();

        // Verify tool and aliases exist
        assert!(registry.contains_tool("tool_with_aliases").await);
        assert!(registry.contains_tool("alias_a").await);
        assert!(registry.contains_tool("alias_b").await);
        assert!(registry.contains_tool("alias_c").await);

        // Unregister the tool
        let result = registry.unregister_tool("tool_with_aliases").await;
        assert!(result.is_ok());

        // Verify tool and all aliases are removed
        assert!(!registry.contains_tool("tool_with_aliases").await);
        assert!(!registry.contains_tool("alias_a").await);
        assert!(!registry.contains_tool("alias_b").await);
        assert!(!registry.contains_tool("alias_c").await);

        // Verify we can now reuse the alias names
        let new_tool = MockTool::new(
            "new_tool".to_string(),
            ToolCategory::Utility,
            SecurityLevel::Safe,
        );
        let result = registry
            .register_with_aliases(
                "new_tool".to_string(),
                vec!["alias_a".to_string()],
                new_tool,
            )
            .await;
        assert!(result.is_ok());
        assert!(registry.contains_tool("new_tool").await);
        assert!(registry.contains_tool("alias_a").await);
    }
}
