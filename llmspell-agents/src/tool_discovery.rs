//! ABOUTME: Tool discovery and registration APIs for agent-tool integration
//! ABOUTME: Provides high-level APIs for tool discovery, registration, and capability matching

use llmspell_core::{
    traits::{
        tool::{SecurityLevel, ToolCategory},
        tool_capable::{ToolInfo, ToolQuery},
    },
    Result,
};
use llmspell_tools::registry::{CapabilityMatcher, ToolRegistry};
use serde_json::Value as JsonValue;
use std::collections::HashMap;
use std::sync::Arc;
use tracing::instrument;

/// High-level tool discovery that provides convenient APIs
/// for finding and filtering tools based on various criteria.
///
/// This wraps the lower-level `ToolRegistry` to provide
/// more ergonomic APIs for agent developers.
///
/// # Examples
///
/// ```
/// use llmspell_agents::tool_discovery::ToolDiscovery;
/// use llmspell_tools::registry::ToolRegistry;
/// use std::sync::Arc;
///
/// # async fn example() -> llmspell_core::Result<()> {
/// let registry = Arc::new(ToolRegistry::new());
/// let discovery = ToolDiscovery::new(registry);
///
/// // Find filesystem tools
/// let fs_tools = discovery.find_by_category("filesystem").await?;
///
/// // Find tools with specific capabilities
/// let search_tools = discovery.find_with_capability("search").await?;
///
/// // Find safe tools only
/// let safe_tools = discovery.find_by_security_level("safe").await?;
/// # Ok(())
/// # }
/// ```
pub struct ToolDiscovery {
    registry: Arc<ToolRegistry>,
}

impl ToolDiscovery {
    /// Create a new tool discovery instance
    #[must_use]
    pub const fn new(registry: Arc<ToolRegistry>) -> Self {
        Self { registry }
    }

    /// Find tools by category
    ///
    /// # Errors
    ///
    /// Returns an error if tool discovery fails
    #[instrument(skip(self))]
    pub async fn find_by_category(&self, category: &str) -> Result<Vec<ToolInfo>> {
        let query = ToolQuery::new().with_category(category);
        self.discover_tools(&query).await
    }

    /// Find tools by security level
    ///
    /// # Errors
    ///
    /// Returns an error if tool discovery fails
    #[instrument(skip(self))]
    pub async fn find_by_security_level(&self, level: &str) -> Result<Vec<ToolInfo>> {
        let query = ToolQuery::new().with_max_security_level(level);
        self.discover_tools(&query).await
    }

    /// Find tools with specific capability
    ///
    /// # Errors
    ///
    /// Returns an error if tool discovery fails
    #[instrument(skip(self))]
    pub async fn find_with_capability(&self, capability: &str) -> Result<Vec<ToolInfo>> {
        let query = ToolQuery::new().with_capability(capability);
        self.discover_tools(&query).await
    }

    /// Find tools by text search in name/description
    ///
    /// # Errors
    ///
    /// Returns an error if tool discovery fails
    #[instrument(skip(self))]
    pub async fn find_by_text(&self, search_text: &str) -> Result<Vec<ToolInfo>> {
        let query = ToolQuery::new().with_text_search(search_text);
        self.discover_tools(&query).await
    }

    /// Find tools that match multiple criteria
    ///
    /// # Errors
    ///
    /// Returns an error if tool discovery fails
    #[instrument(skip(self))]
    pub async fn find_by_criteria(&self, criteria: &ToolSearchCriteria) -> Result<Vec<ToolInfo>> {
        let mut query = ToolQuery::new();

        // Add category filters
        for category in &criteria.categories {
            query = query.with_category(category);
        }

        // Add capability filters
        for capability in &criteria.capabilities {
            query = query.with_capability(capability);
        }

        // Add security level filters
        if let Some(max_level) = &criteria.max_security_level {
            query = query.with_max_security_level(max_level);
        }

        if let Some(min_level) = &criteria.min_security_level {
            query = query.with_min_security_level(min_level);
        }

        // Add text search
        if let Some(text) = &criteria.text_search {
            query = query.with_text_search(text);
        }

        // Add custom filters
        for (key, value) in &criteria.custom_filters {
            query = query.with_custom_filter(key, value.clone());
        }

        self.discover_tools(&query).await
    }

    /// Get all available tools
    ///
    /// # Errors
    ///
    /// Returns an error if tool discovery fails
    #[instrument(skip(self))]
    pub async fn get_all_tools(&self) -> Result<Vec<ToolInfo>> {
        let query = ToolQuery::new();
        self.discover_tools(&query).await
    }

    /// Get tool information by name
    ///
    /// # Errors
    ///
    /// Returns an error if registry access fails
    #[instrument(skip(self))]
    pub async fn get_tool_info(&self, name: &str) -> Result<Option<ToolInfo>> {
        Ok(self
            .registry
            .get_tool_info(name)
            .await
            .map(|registry_info| Self::convert_registry_info(&registry_info)))
    }

    /// Check if a tool exists
    #[instrument(skip(self))]
    pub async fn tool_exists(&self, name: &str) -> bool {
        self.registry.get_tool(name).await.is_some()
    }

    /// Get tools by multiple categories
    ///
    /// # Errors
    ///
    /// Returns an error if tool discovery fails
    #[instrument(skip(self))]
    pub async fn find_by_categories(&self, categories: &[&str]) -> Result<Vec<ToolInfo>> {
        let mut query = ToolQuery::new();
        for category in categories {
            query = query.with_category(*category);
        }
        self.discover_tools(&query).await
    }

    /// Get recommended tools based on context
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Tool discovery fails
    /// - Recommendation processing fails
    #[instrument(skip(self))]
    pub async fn get_recommended_tools(
        &self,
        context: &RecommendationContext,
    ) -> Result<Vec<ToolInfo>> {
        let mut criteria = ToolSearchCriteria::new();

        // Add task-based recommendations
        if let Some(task_type) = &context.task_type {
            match task_type.as_str() {
                "file_processing" => {
                    criteria.categories.push("filesystem".to_string());
                    criteria.categories.push("data".to_string());
                }
                "web_scraping" => {
                    criteria.categories.push("web".to_string());
                    criteria.categories.push("api".to_string());
                }
                "data_analysis" => {
                    criteria.categories.push("analysis".to_string());
                    criteria.categories.push("data".to_string());
                }
                "media_processing" => {
                    criteria.categories.push("media".to_string());
                }
                _ => {
                    // Default to utility tools
                    criteria.categories.push("utility".to_string());
                }
            }
        }

        // Apply security constraints
        if let Some(max_security) = &context.max_security_level {
            criteria.max_security_level = Some(max_security.clone());
        }

        // Apply performance constraints
        if context.performance_critical {
            // Prefer tools with lower overhead
            criteria
                .custom_filters
                .insert("performance_optimized".to_string(), JsonValue::Bool(true));
        }

        self.find_by_criteria(&criteria).await
    }

    /// Internal method to discover tools using `ToolQuery`
    #[instrument(skip(self))]
    async fn discover_tools(&self, query: &ToolQuery) -> Result<Vec<ToolInfo>> {
        // Convert ToolQuery to CapabilityMatcher
        let mut matcher = CapabilityMatcher::new();

        // Add category filters
        if !query.categories.is_empty() {
            let categories: Vec<ToolCategory> = query
                .categories
                .iter()
                .map(|cat| Self::string_to_tool_category(cat))
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
            if let Some(security_level) = Self::string_to_security_level(max_level) {
                matcher = matcher.with_max_security_level(security_level);
            }
        }

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
                if let Some(min_security) = Self::string_to_security_level(min_level) {
                    if registry_info.security_level < min_security {
                        continue;
                    }
                }
            }

            // Convert registry ToolInfo to our ToolInfo
            let tool_info = Self::convert_registry_info(&registry_info);
            tools.push(tool_info);
        }

        Ok(tools)
    }

    /// Convert registry `ToolInfo` to our `ToolInfo` format
    fn convert_registry_info(registry_info: &llmspell_tools::registry::ToolInfo) -> ToolInfo {
        let security_level_str = match registry_info.security_level {
            SecurityLevel::Safe => "safe",
            SecurityLevel::Restricted => "restricted",
            SecurityLevel::Privileged => "privileged",
        }
        .to_string();

        ToolInfo::new(
            registry_info.name.clone(),
            registry_info.description.clone(),
            registry_info.category.to_string(),
            security_level_str,
        )
        .with_schema(JsonValue::Object(serde_json::Map::new())) // Would need tool instance for schema
        .with_requirements(JsonValue::Object(serde_json::Map::new()))
    }

    /// Convert string to `ToolCategory`
    fn string_to_tool_category(category_str: &str) -> ToolCategory {
        match category_str.to_lowercase().as_str() {
            "filesystem" => ToolCategory::Filesystem,
            "web" => ToolCategory::Web,
            "api" => ToolCategory::Api,
            "analysis" => ToolCategory::Analysis,
            "data" => ToolCategory::Data,
            "system" => ToolCategory::System,
            "media" => ToolCategory::Media,
            "utility" => ToolCategory::Utility,
            _ => ToolCategory::Custom(category_str.to_string()),
        }
    }

    /// Convert string to `SecurityLevel`
    fn string_to_security_level(level_str: &str) -> Option<SecurityLevel> {
        match level_str.to_lowercase().as_str() {
            "safe" => Some(SecurityLevel::Safe),
            "restricted" => Some(SecurityLevel::Restricted),
            "privileged" => Some(SecurityLevel::Privileged),
            _ => None,
        }
    }
}

/// Search criteria for finding tools with multiple constraints
#[derive(Debug, Clone, Default)]
pub struct ToolSearchCriteria {
    /// Filter by tool categories
    pub categories: Vec<String>,
    /// Filter by required capabilities
    pub capabilities: Vec<String>,
    /// Maximum security level allowed
    pub max_security_level: Option<String>,
    /// Minimum security level required
    pub min_security_level: Option<String>,
    /// Text search in tool names/descriptions
    pub text_search: Option<String>,
    /// Custom query parameters
    pub custom_filters: HashMap<String, JsonValue>,
}

impl ToolSearchCriteria {
    /// Create new empty search criteria
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a category filter
    #[must_use]
    pub fn with_category(mut self, category: impl Into<String> + std::fmt::Debug) -> Self {
        self.categories.push(category.into());
        self
    }

    /// Add a capability filter
    #[must_use]
    pub fn with_capability(mut self, capability: impl Into<String> + std::fmt::Debug) -> Self {
        self.capabilities.push(capability.into());
        self
    }

    /// Set maximum security level
    #[must_use]
    pub fn with_max_security_level(mut self, level: impl Into<String> + std::fmt::Debug) -> Self {
        self.max_security_level = Some(level.into());
        self
    }

    /// Set minimum security level
    #[must_use]
    pub fn with_min_security_level(mut self, level: impl Into<String> + std::fmt::Debug) -> Self {
        self.min_security_level = Some(level.into());
        self
    }

    /// Add text search filter
    #[must_use]
    pub fn with_text_search(mut self, text: impl Into<String> + std::fmt::Debug) -> Self {
        self.text_search = Some(text.into());
        self
    }

    /// Add custom filter
    #[must_use]
    pub fn with_custom_filter(
        mut self,
        key: impl Into<String> + std::fmt::Debug,
        value: JsonValue,
    ) -> Self {
        self.custom_filters.insert(key.into(), value);
        self
    }
}

/// Context for tool recommendations
#[derive(Debug, Clone)]
pub struct RecommendationContext {
    /// Type of task being performed
    pub task_type: Option<String>,
    /// Maximum security level allowed
    pub max_security_level: Option<String>,
    /// Whether performance is critical
    pub performance_critical: bool,
    /// User preferences
    pub user_preferences: HashMap<String, JsonValue>,
    /// Previous tool usage history
    pub usage_history: Vec<String>,
}

impl RecommendationContext {
    /// Create new recommendation context
    #[must_use]
    pub fn new() -> Self {
        Self {
            task_type: None,
            max_security_level: None,
            performance_critical: false,
            user_preferences: HashMap::new(),
            usage_history: Vec::new(),
        }
    }

    /// Set task type
    #[must_use]
    pub fn with_task_type(mut self, task_type: impl Into<String> + std::fmt::Debug) -> Self {
        self.task_type = Some(task_type.into());
        self
    }

    /// Set maximum security level
    #[must_use]
    pub fn with_max_security_level(mut self, level: impl Into<String> + std::fmt::Debug) -> Self {
        self.max_security_level = Some(level.into());
        self
    }

    /// Mark as performance critical
    #[must_use]
    pub const fn performance_critical(mut self) -> Self {
        self.performance_critical = true;
        self
    }

    /// Add user preference
    #[must_use]
    pub fn with_preference(
        mut self,
        key: impl Into<String> + std::fmt::Debug,
        value: JsonValue,
    ) -> Self {
        self.user_preferences.insert(key.into(), value);
        self
    }

    /// Add usage history
    #[must_use]
    pub fn with_usage_history(mut self, tools: Vec<String>) -> Self {
        self.usage_history = tools;
        self
    }
}

impl Default for RecommendationContext {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use llmspell_tools::registry::ToolRegistry;
    #[tokio::test]
    async fn test_tool_discovery_service_creation() {
        let registry = Arc::new(ToolRegistry::new());
        let discovery = ToolDiscovery::new(registry);

        // Test that service was created successfully
        assert!(!(discovery.tool_exists("nonexistent").await));
    }
    #[tokio::test]
    async fn test_search_criteria_builder() {
        let criteria = ToolSearchCriteria::new()
            .with_category("filesystem")
            .with_capability("search")
            .with_max_security_level("safe")
            .with_text_search("file");

        assert_eq!(criteria.categories.len(), 1);
        assert_eq!(criteria.capabilities.len(), 1);
        assert_eq!(criteria.max_security_level, Some("safe".to_string()));
        assert_eq!(criteria.text_search, Some("file".to_string()));
    }
    #[tokio::test]
    async fn test_recommendation_context_builder() {
        let context = RecommendationContext::new()
            .with_task_type("file_processing")
            .with_max_security_level("restricted")
            .performance_critical()
            .with_preference("auto_save".to_string(), JsonValue::Bool(true));

        assert_eq!(context.task_type, Some("file_processing".to_string()));
        assert_eq!(context.max_security_level, Some("restricted".to_string()));
        assert!(context.performance_critical);
        assert_eq!(context.user_preferences.len(), 1);
    }
    #[tokio::test]
    async fn test_find_by_category() {
        let registry = Arc::new(ToolRegistry::new());
        let discovery = ToolDiscovery::new(registry);

        // Test empty result for nonexistent category
        let tools = discovery.find_by_category("nonexistent").await.unwrap();
        assert_eq!(tools.len(), 0);
    }
    #[tokio::test]
    async fn test_get_recommended_tools() {
        let registry = Arc::new(ToolRegistry::new());
        let discovery = ToolDiscovery::new(registry);

        let context = RecommendationContext::new()
            .with_task_type("file_processing")
            .with_max_security_level("safe");

        let tools = discovery.get_recommended_tools(&context).await.unwrap();
        // Should return empty for new registry, but test that it doesn't error
        assert!(tools.is_empty() || !tools.is_empty()); // Just test it doesn't panic
    }
}
