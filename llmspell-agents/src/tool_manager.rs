//! ABOUTME: ToolManager for managing tool discovery, invocation, and composition
//! ABOUTME: Core implementation that enables ToolCapable components to interact with tools

use llmspell_core::traits::tool::{SecurityLevel, ToolCategory};
use llmspell_core::{
    traits::tool_capable::{
        ContextMode, ErrorStrategy, ToolComposition, ToolCompositionStep, ToolInfo, ToolQuery,
    },
    types::{AgentInput, AgentOutput},
    ExecutionContext, LLMSpellError, Result,
};
use llmspell_tools::registry::{CapabilityMatcher, ToolInfo as RegistryToolInfo, ToolRegistry};
use serde_json::{Map, Value as JsonValue};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Tool manager that provides concrete implementation of tool integration capabilities.
///
/// This component bridges the gap between ToolCapable components and the actual tool
/// ecosystem. It handles tool discovery, invocation, validation, and composition.
///
/// # Architecture
///
/// ```text
/// ToolCapable Component
///         ↓
///   ToolManager (this)
///         ↓
///   ToolRegistry → Actual Tools
/// ```
///
/// # Usage
///
/// ```
/// use llmspell_agents::tool_manager::ToolManager;
/// use llmspell_tools::registry::ToolRegistry;
/// use llmspell_core::traits::tool_capable::ToolQuery;
/// use serde_json::json;
/// use std::sync::Arc;
///
/// # async fn example() -> llmspell_core::Result<()> {
/// let registry = Arc::new(ToolRegistry::new());
/// let manager = ToolManager::new(registry);
///
/// // Discover tools
/// let query = ToolQuery::new().with_category("filesystem");
/// let tools = manager.discover_tools(&query).await?;
///
/// // Invoke a tool
/// let params = json!({"pattern": "*.txt"});
/// let context = llmspell_core::ExecutionContext::new();
/// let result = manager.invoke_tool("file_search", params, context).await?;
/// # Ok(())
/// # }
/// ```
pub struct ToolManager {
    /// Reference to the tool registry
    registry: Arc<ToolRegistry>,
    /// Cache for tool metadata to improve performance
    metadata_cache: Arc<RwLock<HashMap<String, ToolInfo>>>,
    /// Cache for tool availability checks
    availability_cache: Arc<RwLock<HashMap<String, bool>>>,
    /// Configuration for tool manager behavior
    config: ToolManagerConfig,
}

/// Configuration for ToolManager behavior
#[derive(Debug, Clone)]
pub struct ToolManagerConfig {
    /// Maximum execution time for tool invocation (milliseconds)
    pub max_execution_time_ms: u64,
    /// Whether to cache tool metadata
    pub enable_metadata_cache: bool,
    /// Whether to cache availability checks
    pub enable_availability_cache: bool,
    /// Maximum number of parallel tool executions in composition
    pub max_parallel_executions: usize,
    /// Whether to validate tool parameters before invocation
    pub validate_parameters: bool,
}

impl Default for ToolManagerConfig {
    fn default() -> Self {
        Self {
            max_execution_time_ms: 30_000, // 30 seconds
            enable_metadata_cache: true,
            enable_availability_cache: true,
            max_parallel_executions: 4,
            validate_parameters: true,
        }
    }
}

impl ToolManager {
    /// Create a new ToolManager with the given registry
    pub fn new(registry: Arc<ToolRegistry>) -> Self {
        Self {
            registry,
            metadata_cache: Arc::new(RwLock::new(HashMap::new())),
            availability_cache: Arc::new(RwLock::new(HashMap::new())),
            config: ToolManagerConfig::default(),
        }
    }

    /// Create a new ToolManager with custom configuration
    pub fn with_config(registry: Arc<ToolRegistry>, config: ToolManagerConfig) -> Self {
        Self {
            registry,
            metadata_cache: Arc::new(RwLock::new(HashMap::new())),
            availability_cache: Arc::new(RwLock::new(HashMap::new())),
            config,
        }
    }

    /// Discover tools based on query criteria
    pub async fn discover_tools(&self, query: &ToolQuery) -> Result<Vec<ToolInfo>> {
        // Convert ToolQuery to CapabilityMatcher
        let mut matcher = CapabilityMatcher::new();

        // Add category filters
        if !query.categories.is_empty() {
            let categories: Vec<ToolCategory> = query
                .categories
                .iter()
                .filter_map(|cat| self.string_to_tool_category(cat))
                .collect();
            if !categories.is_empty() {
                matcher = matcher.with_categories(categories);
            }
        }

        // Add capability filters
        for capability in &query.capabilities {
            matcher = matcher.with_capability(capability.clone(), JsonValue::Bool(true));
        }

        // Add security level filters
        if let Some(max_level) = &query.max_security_level {
            if let Some(security_level) = self.string_to_security_level(max_level) {
                matcher = matcher.with_max_security_level(security_level);
            }
        }

        // Note: min_security_level is not supported by CapabilityMatcher
        // We'll filter it manually after discovery

        // Get tools from registry
        let tool_info_list = self.registry.discover_tools(&matcher).await;

        let mut tools = Vec::new();
        for registry_info in tool_info_list {
            // Apply text search filter if specified
            if let Some(search_text) = &query.text_search {
                let text_lower = search_text.to_lowercase();
                if !registry_info.name.to_lowercase().contains(&text_lower)
                    && !registry_info
                        .description
                        .to_lowercase()
                        .contains(&text_lower)
                {
                    continue;
                }
            }

            // Apply min_security_level filter manually
            if let Some(min_level) = &query.min_security_level {
                if let Some(min_security) = self.string_to_security_level(min_level) {
                    if registry_info.security_level < min_security {
                        continue;
                    }
                }
            }

            // Convert registry ToolInfo to our ToolInfo
            let tool_info = self.registry_info_to_tool_info(&registry_info);
            tools.push(tool_info);
        }

        Ok(tools)
    }

    /// Invoke a tool by name with given parameters
    pub async fn invoke_tool(
        &self,
        tool_name: &str,
        parameters: JsonValue,
        context: ExecutionContext,
    ) -> Result<AgentOutput> {
        // Check if tool is available
        if !self.tool_available(tool_name).await {
            return Err(LLMSpellError::Component {
                message: format!("Tool '{}' not found or not available", tool_name),
                source: None,
            });
        }

        // Get tool instance from registry
        let tool =
            self.registry
                .get_tool(tool_name)
                .await
                .ok_or_else(|| LLMSpellError::Component {
                    message: format!("Tool '{}' not found in registry", tool_name),
                    source: None,
                })?;

        // Validate parameters if enabled
        if self.config.validate_parameters {
            // Note: Tool trait doesn't have validate_parameters method
            // We'll skip validation for now or implement basic JSON schema validation
        }

        // Create AgentInput with parameters
        let input = AgentInput::text("Tool invocation".to_string())
            .with_parameter("parameters".to_string(), parameters);

        // Execute tool with timeout
        let result = tokio::time::timeout(
            std::time::Duration::from_millis(self.config.max_execution_time_ms),
            tool.execute(input, context),
        )
        .await
        .map_err(|_| LLMSpellError::Component {
            message: format!("Tool '{}' execution timed out", tool_name),
            source: None,
        })??;

        Ok(result)
    }

    /// List all available tools
    pub async fn list_available_tools(&self) -> Result<Vec<String>> {
        let all_tools = self.registry.list_tools().await;
        Ok(all_tools)
    }

    /// Check if a specific tool is available
    pub async fn tool_available(&self, tool_name: &str) -> bool {
        // Check cache first if enabled
        if self.config.enable_availability_cache {
            let cache = self.availability_cache.read().await;
            if let Some(&available) = cache.get(tool_name) {
                return available;
            }
        }

        // Check registry
        let available = self.registry.get_tool(tool_name).await.is_some();

        // Update cache if enabled
        if self.config.enable_availability_cache {
            let mut cache = self.availability_cache.write().await;
            cache.insert(tool_name.to_string(), available);
        }

        available
    }

    /// Get information about a specific tool
    pub async fn get_tool_info(&self, tool_name: &str) -> Result<Option<ToolInfo>> {
        // Check cache first if enabled
        if self.config.enable_metadata_cache {
            let cache = self.metadata_cache.read().await;
            if let Some(info) = cache.get(tool_name) {
                return Ok(Some(info.clone()));
            }
        }

        // Get tool info from registry
        let registry_info = match self.registry.get_tool_info(tool_name).await {
            Some(info) => info,
            None => return Ok(None),
        };

        // Convert to our ToolInfo format
        let tool_info = self.registry_info_to_tool_info(&registry_info);

        // Update cache if enabled
        if self.config.enable_metadata_cache {
            let mut cache = self.metadata_cache.write().await;
            cache.insert(tool_name.to_string(), tool_info.clone());
        }

        Ok(Some(tool_info))
    }

    /// Compose multiple tools into a workflow
    pub async fn compose_tools(
        &self,
        composition: &ToolComposition,
        mut context: ExecutionContext,
    ) -> Result<AgentOutput> {
        let mut previous_output: Option<AgentOutput> = None;
        let mut results = Vec::new();

        for (step_index, step) in composition.steps.iter().enumerate() {
            let step_result = self
                .execute_composition_step(step, step_index, &mut context, previous_output.as_ref())
                .await;

            match step_result {
                Ok(output) => {
                    previous_output = Some(output.clone());
                    results.push(output);
                }
                Err(error) => {
                    match step.error_strategy {
                        ErrorStrategy::Fail => return Err(error),
                        ErrorStrategy::Continue => {
                            // Use a default output and continue
                            let default_output =
                                AgentOutput::text(format!("Step {} failed: {}", step_index, error));
                            previous_output = Some(default_output.clone());
                            results.push(default_output);
                        }
                        ErrorStrategy::Retry(max_attempts) => {
                            // Implement retry logic
                            let mut attempts = 0;
                            let mut _last_error = error;

                            while attempts < max_attempts {
                                attempts += 1;
                                match self
                                    .execute_composition_step(
                                        step,
                                        step_index,
                                        &mut context,
                                        previous_output.as_ref(),
                                    )
                                    .await
                                {
                                    Ok(output) => {
                                        previous_output = Some(output.clone());
                                        results.push(output);
                                        break;
                                    }
                                    Err(e) => {
                                        _last_error = e;
                                        if attempts >= max_attempts {
                                            return Err(_last_error);
                                        }
                                    }
                                }
                            }
                        }
                        ErrorStrategy::Skip => {
                            // Skip this step and continue with previous output
                            continue;
                        }
                    }
                }
            }
        }

        // Return the final result
        if let Some(final_output) = results.last() {
            Ok(final_output.clone())
        } else {
            Ok(AgentOutput::text(
                "Composition completed with no output".to_string(),
            ))
        }
    }

    /// Execute a single composition step
    async fn execute_composition_step(
        &self,
        step: &ToolCompositionStep,
        _step_index: usize,
        context: &mut ExecutionContext,
        previous_output: Option<&AgentOutput>,
    ) -> Result<AgentOutput> {
        // Prepare parameters for this step
        let parameters = self.prepare_step_parameters(
            &step.parameters,
            &step.context_mode,
            context,
            previous_output,
        )?;

        // Invoke the tool
        self.invoke_tool(&step.tool_name, parameters, context.clone())
            .await
    }

    /// Prepare parameters for a composition step
    fn prepare_step_parameters(
        &self,
        base_parameters: &JsonValue,
        context_mode: &ContextMode,
        _context: &ExecutionContext,
        previous_output: Option<&AgentOutput>,
    ) -> Result<JsonValue> {
        let mut parameters = base_parameters.clone();

        // Handle parameter substitution based on context mode
        match context_mode {
            ContextMode::Full => {
                // Parameters can reference full context - this would need template resolution
                // For now, just use the base parameters
            }
            ContextMode::Previous => {
                // Replace ${previous.output} with actual previous output
                if let Some(prev_output) = previous_output {
                    parameters = self.substitute_previous_output(parameters, prev_output)?;
                }
            }
            ContextMode::Selective(_fields) => {
                // Replace specific fields - implementation would depend on context structure
                // For now, treat as Full mode
            }
        }

        Ok(parameters)
    }

    /// Substitute ${previous.output} references with actual output
    fn substitute_previous_output(
        &self,
        mut parameters: JsonValue,
        previous_output: &AgentOutput,
    ) -> Result<JsonValue> {
        // Simple substitution - replace "${previous.output}" with the output text
        if let JsonValue::Object(ref mut map) = parameters {
            for (_, value) in map.iter_mut() {
                if let JsonValue::String(s) = value {
                    if s == "${previous.output}" {
                        *value = JsonValue::String(previous_output.text.clone());
                    }
                }
            }
        }
        Ok(parameters)
    }

    /// Convert registry ToolInfo to our ToolInfo format
    fn registry_info_to_tool_info(&self, registry_info: &RegistryToolInfo) -> ToolInfo {
        // Convert SecurityLevel to string
        let security_level_str = match registry_info.security_level {
            SecurityLevel::Safe => "safe",
            SecurityLevel::Restricted => "restricted",
            SecurityLevel::Privileged => "privileged",
        }
        .to_string();

        ToolInfo {
            name: registry_info.name.clone(),
            description: registry_info.description.clone(),
            category: registry_info.category.to_string(),
            security_level: security_level_str,
            schema: JsonValue::Object(Map::new()), // Schema would need to be generated from the tool itself
            capabilities: Vec::new(), // Registry ToolInfo doesn't have capabilities field
            requirements: JsonValue::Object(Map::new()), // Could be expanded with security requirements
        }
    }

    /// Convert string to ToolCategory
    fn string_to_tool_category(&self, category_str: &str) -> Option<ToolCategory> {
        match category_str.to_lowercase().as_str() {
            "filesystem" => Some(ToolCategory::Filesystem),
            "web" => Some(ToolCategory::Web),
            "api" => Some(ToolCategory::Api),
            "analysis" => Some(ToolCategory::Analysis),
            "data" => Some(ToolCategory::Data),
            "system" => Some(ToolCategory::System),
            "media" => Some(ToolCategory::Media),
            "utility" => Some(ToolCategory::Utility),
            _ => Some(ToolCategory::Custom(category_str.to_string())),
        }
    }

    /// Convert string to SecurityLevel
    fn string_to_security_level(&self, level_str: &str) -> Option<SecurityLevel> {
        match level_str.to_lowercase().as_str() {
            "safe" => Some(SecurityLevel::Safe),
            "restricted" => Some(SecurityLevel::Restricted),
            "privileged" => Some(SecurityLevel::Privileged),
            _ => None,
        }
    }

    /// Clear caches
    pub async fn clear_caches(&self) {
        if self.config.enable_metadata_cache {
            self.metadata_cache.write().await.clear();
        }
        if self.config.enable_availability_cache {
            self.availability_cache.write().await.clear();
        }
    }

    /// Get configuration
    pub fn config(&self) -> &ToolManagerConfig {
        &self.config
    }

    /// Update configuration
    pub fn update_config(&mut self, config: ToolManagerConfig) {
        self.config = config;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use llmspell_core::traits::tool_capable::{ToolComposition, ToolQuery};
    use llmspell_tools::registry::ToolRegistry;
    use serde_json::json;
    #[tokio::test]
    async fn test_tool_manager_creation() {
        let registry = Arc::new(ToolRegistry::new());
        let manager = ToolManager::new(registry);

        assert_eq!(manager.config.max_execution_time_ms, 30_000);
        assert!(manager.config.enable_metadata_cache);
        assert!(manager.config.enable_availability_cache);
    }
    #[tokio::test]
    async fn test_tool_manager_with_config() {
        let registry = Arc::new(ToolRegistry::new());
        let config = ToolManagerConfig {
            max_execution_time_ms: 5_000,
            enable_metadata_cache: false,
            enable_availability_cache: false,
            max_parallel_executions: 2,
            validate_parameters: false,
        };

        let manager = ToolManager::with_config(registry, config);
        assert_eq!(manager.config.max_execution_time_ms, 5_000);
        assert!(!manager.config.enable_metadata_cache);
        assert!(!manager.config.enable_availability_cache);
        assert_eq!(manager.config.max_parallel_executions, 2);
        assert!(!manager.config.validate_parameters);
    }
    #[tokio::test]
    async fn test_tool_discovery() {
        let registry = Arc::new(ToolRegistry::new());
        let manager = ToolManager::new(registry);

        // Test empty discovery
        let query = ToolQuery::new().with_category("nonexistent");
        let tools = manager.discover_tools(&query).await.unwrap();
        assert_eq!(tools.len(), 0);
    }
    #[tokio::test]
    async fn test_tool_availability_checks() {
        let registry = Arc::new(ToolRegistry::new());
        let manager = ToolManager::new(registry);

        // Test non-existent tool
        assert!(!manager.tool_available("nonexistent_tool").await);

        // Test that result is cached (second call should use cache)
        assert!(!manager.tool_available("nonexistent_tool").await);
    }
    #[tokio::test]
    async fn test_tool_listing() {
        let registry = Arc::new(ToolRegistry::new());
        let manager = ToolManager::new(registry);

        let tools = manager.list_available_tools().await.unwrap();
        // Should be empty for a new registry
        assert_eq!(tools.len(), 0);
    }
    #[tokio::test]
    async fn test_tool_info_retrieval() {
        let registry = Arc::new(ToolRegistry::new());
        let manager = ToolManager::new(registry);

        // Test non-existent tool
        let info = manager.get_tool_info("nonexistent").await.unwrap();
        assert!(info.is_none());
    }
    #[tokio::test]
    async fn test_tool_composition_empty() {
        let registry = Arc::new(ToolRegistry::new());
        let manager = ToolManager::new(registry);

        let composition = ToolComposition::new("empty_test", "Empty composition");
        let context = ExecutionContext::new();

        let result = manager.compose_tools(&composition, context).await.unwrap();
        assert!(result.text.contains("no output"));
    }
    #[tokio::test]
    async fn test_parameter_substitution() {
        let registry = Arc::new(ToolRegistry::new());
        let manager = ToolManager::new(registry);

        let parameters = json!({
            "input": "${previous.output}",
            "other": "value"
        });

        let previous_output = AgentOutput::text("test_output".to_string());
        let result = manager
            .substitute_previous_output(parameters, &previous_output)
            .unwrap();

        assert_eq!(
            result["input"],
            JsonValue::String("test_output".to_string())
        );
        assert_eq!(result["other"], JsonValue::String("value".to_string()));
    }
    #[tokio::test]
    async fn test_cache_clearing() {
        let registry = Arc::new(ToolRegistry::new());
        let manager = ToolManager::new(registry);

        // Add something to cache by checking availability
        manager.tool_available("test").await;

        // Clear caches
        manager.clear_caches().await;

        // Verify caches are empty
        assert_eq!(manager.metadata_cache.read().await.len(), 0);
        assert_eq!(manager.availability_cache.read().await.len(), 0);
    }
}
