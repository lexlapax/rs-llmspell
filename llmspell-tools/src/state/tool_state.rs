// ABOUTME: Tool state persistence integration for automated state management
// ABOUTME: Provides state save/load capabilities for tool execution context and results

use anyhow::{Context, Result};
use async_trait::async_trait;
use llmspell_core::traits::tool::Tool;
use llmspell_core::ComponentMetadata;
use llmspell_state_traits::{StateManager, StateScope};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, SystemTime};
use tracing::{debug, info, warn};

/// Tool execution state for persistence
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolState {
    /// Tool identifier
    pub tool_id: String,
    /// Tool metadata
    pub metadata: ComponentMetadata,
    /// Execution statistics
    pub execution_stats: ToolExecutionStats,
    /// Cached results by input hash
    pub result_cache: HashMap<String, CachedResult>,
    /// State timestamp
    pub last_updated: SystemTime,
    /// Tool-specific custom state
    pub custom_state: HashMap<String, Value>,
}

/// Tool execution statistics
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ToolExecutionStats {
    /// Total number of executions
    pub total_executions: u64,
    /// Number of successful executions
    pub successful_executions: u64,
    /// Number of failed executions
    pub failed_executions: u64,
    /// Total execution time in milliseconds
    pub total_execution_time_ms: u64,
    /// Average execution time in milliseconds
    pub average_execution_time_ms: f64,
    /// Last execution timestamp
    pub last_execution: Option<SystemTime>,
    /// Cache hit ratio percentage
    pub cache_hit_ratio: f64,
    /// Resource usage statistics
    pub resource_usage: ResourceUsageStats,
}

/// Resource usage statistics
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ResourceUsageStats {
    /// Peak memory usage in bytes
    pub peak_memory_bytes: u64,
    /// Average memory usage in bytes
    pub average_memory_bytes: u64,
    /// Total CPU time in milliseconds
    pub total_cpu_time_ms: u64,
    /// Number of file operations
    pub file_operations: u64,
    /// Number of network requests
    pub network_requests: u64,
}

/// Cached tool result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CachedResult {
    /// Input hash for cache key
    pub input_hash: String,
    /// Serialized result
    pub result: Value,
    /// Execution time for this result
    pub execution_time_ms: u64,
    /// When this result was cached
    pub cached_at: SystemTime,
    /// TTL for cache entry in seconds
    pub ttl_seconds: u64,
    /// Whether this result contains sensitive data
    pub contains_sensitive_data: bool,
}

impl ToolState {
    /// Create new tool state
    pub fn new(tool_id: String, metadata: ComponentMetadata) -> Self {
        Self {
            tool_id,
            metadata,
            execution_stats: ToolExecutionStats::default(),
            result_cache: HashMap::new(),
            last_updated: SystemTime::now(),
            custom_state: HashMap::new(),
        }
    }

    /// Update execution statistics
    pub fn record_execution(&mut self, success: bool, duration: Duration) {
        self.execution_stats.total_executions += 1;
        let duration_ms = duration.as_millis() as u64;
        self.execution_stats.total_execution_time_ms += duration_ms;

        if success {
            self.execution_stats.successful_executions += 1;
        } else {
            self.execution_stats.failed_executions += 1;
        }

        // Update average execution time
        self.execution_stats.average_execution_time_ms =
            self.execution_stats.total_execution_time_ms as f64
                / self.execution_stats.total_executions as f64;

        self.execution_stats.last_execution = Some(SystemTime::now());
        self.last_updated = SystemTime::now();
    }

    /// Add result to cache
    pub fn cache_result(
        &mut self,
        input_hash: String,
        result: Value,
        execution_time: Duration,
        ttl_seconds: u64,
        contains_sensitive_data: bool,
    ) {
        let cached_result = CachedResult {
            input_hash: input_hash.clone(),
            result,
            execution_time_ms: execution_time.as_millis() as u64,
            cached_at: SystemTime::now(),
            ttl_seconds,
            contains_sensitive_data,
        };

        self.result_cache.insert(input_hash, cached_result);
        self.last_updated = SystemTime::now();
    }

    /// Get cached result if valid
    pub fn get_cached_result(&self, input_hash: &str) -> Option<&CachedResult> {
        if let Some(cached) = self.result_cache.get(input_hash) {
            // Check if cache entry is still valid
            if let Ok(elapsed) = cached.cached_at.elapsed() {
                if elapsed.as_secs() <= cached.ttl_seconds {
                    return Some(cached);
                }
            }
        }
        None
    }

    /// Clean expired cache entries
    pub fn clean_expired_cache(&mut self) {
        let now = SystemTime::now();
        self.result_cache.retain(|_, cached| {
            if let Ok(elapsed) = cached.cached_at.elapsed() {
                elapsed.as_secs() <= cached.ttl_seconds
            } else {
                false // Remove entries we can't check
            }
        });

        if !self.result_cache.is_empty() {
            self.last_updated = now;
        }
    }

    /// Calculate cache hit ratio
    pub fn update_cache_hit_ratio(&mut self, was_cache_hit: bool) {
        let total_requests = self.execution_stats.total_executions as f64;
        if total_requests > 0.0 {
            let current_hits =
                self.execution_stats.cache_hit_ratio * (total_requests - 1.0) / 100.0;
            let new_hits = if was_cache_hit {
                current_hits + 1.0
            } else {
                current_hits
            };
            self.execution_stats.cache_hit_ratio = (new_hits / total_requests) * 100.0;
        }
    }

    /// Set custom state value
    pub fn set_custom_state(&mut self, key: String, value: Value) {
        self.custom_state.insert(key, value);
        self.last_updated = SystemTime::now();
    }

    /// Get custom state value
    pub fn get_custom_state(&self, key: &str) -> Option<&Value> {
        self.custom_state.get(key)
    }
}

/// Extension trait for tools to add state persistence capabilities
#[async_trait]
pub trait ToolStatePersistence: Tool {
    /// Get the state manager for this tool
    fn state_manager(&self) -> Option<Arc<dyn StateManager>>;

    /// Set the state manager for this tool
    fn set_state_manager(&self, state_manager: Arc<dyn StateManager>);

    /// Save the tool's current state
    async fn save_state(&self) -> Result<()> {
        if let Some(state_manager) = self.state_manager() {
            let tool_state = self.create_tool_state().await?;
            let state_scope = StateScope::Custom(format!("tool_{}", self.metadata().id));

            state_manager
                .set(state_scope, "state", serde_json::to_value(&tool_state)?)
                .await
                .map_err(|e| anyhow::anyhow!("Failed to save tool state: {}", e))?;

            info!("Saved state for tool {}", self.metadata().id);
            Ok(())
        } else {
            Err(anyhow::anyhow!("No state manager configured"))
        }
    }

    /// Load the tool's state from storage
    async fn load_state(&self) -> Result<bool> {
        if let Some(state_manager) = self.state_manager() {
            let tool_id = self.metadata().id.to_string();
            let state_scope = StateScope::Custom(format!("tool_{}", tool_id));

            match state_manager.get(state_scope, "state").await {
                Ok(Some(state_value)) => {
                    let tool_state: ToolState = serde_json::from_value(state_value)
                        .context("Failed to deserialize tool state")?;
                    self.restore_from_tool_state(tool_state).await?;
                    info!("Loaded state for tool {}", tool_id);
                    Ok(true)
                }
                Ok(None) => {
                    debug!("No saved state found for tool {}", tool_id);
                    Ok(false)
                }
                Err(e) => Err(anyhow::anyhow!("Failed to load tool state: {}", e)),
            }
        } else {
            Err(anyhow::anyhow!("No state manager configured"))
        }
    }

    /// Create a tool state representation from current tool state
    async fn create_tool_state(&self) -> Result<ToolState> {
        let mut tool_state =
            ToolState::new(self.metadata().id.to_string(), self.metadata().clone());

        // Add execution statistics if available
        if let Some(stats) = self.execution_statistics() {
            tool_state.execution_stats = stats;
        }

        // Add cached results if available
        if let Some(cache) = self.result_cache() {
            tool_state.result_cache = cache;
        }

        // Add custom state if available
        if let Some(custom) = self.custom_state() {
            tool_state.custom_state = custom;
        }

        Ok(tool_state)
    }

    /// Restore tool state from saved state
    async fn restore_from_tool_state(&self, state: ToolState) -> Result<()> {
        let cache_count = state.result_cache.len();
        let tool_id = state.tool_id.clone();

        // Restore execution statistics
        self.restore_execution_statistics(state.execution_stats)?;

        // Restore result cache
        self.restore_result_cache(state.result_cache)?;

        // Restore custom state
        self.restore_custom_state(state.custom_state)?;

        info!(
            "Restored tool state for '{}' with {} cached results",
            tool_id, cache_count
        );

        Ok(())
    }

    /// Get execution statistics (optional override)
    fn execution_statistics(&self) -> Option<ToolExecutionStats> {
        None
    }

    /// Get result cache (optional override)
    fn result_cache(&self) -> Option<HashMap<String, CachedResult>> {
        None
    }

    /// Get custom state (optional override)
    fn custom_state(&self) -> Option<HashMap<String, Value>> {
        None
    }

    /// Restore execution statistics (optional override)
    fn restore_execution_statistics(&self, _stats: ToolExecutionStats) -> Result<()> {
        Ok(())
    }

    /// Restore result cache (optional override)
    fn restore_result_cache(&self, _cache: HashMap<String, CachedResult>) -> Result<()> {
        Ok(())
    }

    /// Restore custom state (optional override)
    fn restore_custom_state(&self, _state: HashMap<String, Value>) -> Result<()> {
        Ok(())
    }
}

/// State manager holder trait for concrete implementations
pub trait ToolStateManagerHolder {
    fn state_manager(&self) -> Option<Arc<dyn StateManager>>;
    fn set_state_manager(&self, state_manager: Arc<dyn StateManager>);
}

/// Tool state registry for managing multiple tool states
pub struct ToolStateRegistry {
    state_manager: Arc<dyn StateManager>,
    tool_states: HashMap<String, ToolState>,
}

impl ToolStateRegistry {
    /// Create new tool state registry
    pub fn new(state_manager: Arc<dyn StateManager>) -> Self {
        Self {
            state_manager,
            tool_states: HashMap::new(),
        }
    }

    /// Register a tool for state management
    pub async fn register_tool<T: ToolStatePersistence>(&mut self, tool: T) -> Result<T> {
        tool.set_state_manager(self.state_manager.clone());

        // Try to load existing state
        match tool.load_state().await {
            Ok(true) => info!("Restored state for tool {}", tool.metadata().id),
            Ok(false) => debug!("No existing state for tool {}", tool.metadata().id),
            Err(e) => warn!(
                "Failed to load state for tool {}: {}",
                tool.metadata().id,
                e
            ),
        }

        Ok(tool)
    }

    /// Save state for all registered tools
    pub async fn save_all_states(&self) -> Result<()> {
        for (tool_id, tool_state) in &self.tool_states {
            let state_scope = StateScope::Custom(format!("tool_{}", tool_id));
            self.state_manager
                .set(state_scope, "state", serde_json::to_value(tool_state)?)
                .await
                .context(format!("Failed to save state for tool {}", tool_id))?;
        }
        Ok(())
    }

    /// Load state for a specific tool
    pub async fn load_tool_state(&self, tool_id: &str) -> Result<Option<ToolState>> {
        let state_scope = StateScope::Custom(format!("tool_{}", tool_id));

        match self.state_manager.get(state_scope, "state").await {
            Ok(Some(state_value)) => {
                let tool_state: ToolState = serde_json::from_value(state_value)
                    .context("Failed to deserialize tool state")?;
                Ok(Some(tool_state))
            }
            Ok(None) => Ok(None),
            Err(e) => Err(anyhow::anyhow!("Failed to load tool state: {}", e)),
        }
    }

    /// Get registry statistics
    pub async fn get_registry_stats(&self) -> RegistryStatistics {
        let total_tools = self.tool_states.len();
        let total_cached_results: usize = self
            .tool_states
            .values()
            .map(|state| state.result_cache.len())
            .sum();

        let total_executions: u64 = self
            .tool_states
            .values()
            .map(|state| state.execution_stats.total_executions)
            .sum();

        let average_cache_hit_ratio: f64 = if !self.tool_states.is_empty() {
            self.tool_states
                .values()
                .map(|state| state.execution_stats.cache_hit_ratio)
                .sum::<f64>()
                / self.tool_states.len() as f64
        } else {
            0.0
        };

        RegistryStatistics {
            total_tools,
            total_cached_results,
            total_executions,
            average_cache_hit_ratio,
        }
    }
}

/// Registry statistics
#[derive(Debug, Clone)]
pub struct RegistryStatistics {
    pub total_tools: usize,
    pub total_cached_results: usize,
    pub total_executions: u64,
    pub average_cache_hit_ratio: f64,
}

/// Macro to implement ToolStatePersistence trait for types that implement Tool + ToolStateManagerHolder
#[macro_export]
macro_rules! impl_tool_state_persistence {
    ($tool_type:ty) => {
        #[async_trait::async_trait]
        impl $crate::state::tool_state::ToolStatePersistence for $tool_type {
            fn state_manager(
                &self,
            ) -> Option<&std::sync::Arc<llmspell_state_persistence::StateManager>> {
                self.state_manager()
            }

            fn set_state_manager(
                &mut self,
                state_manager: std::sync::Arc<llmspell_state_persistence::StateManager>,
            ) {
                self.set_state_manager(state_manager)
            }
        }
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    use llmspell_core::types::{AgentInput, AgentOutput};
    use llmspell_core::{BaseAgent, ExecutionContext, LLMSpellError};
    use llmspell_state_persistence::config::PersistenceConfig;
    use llmspell_state_persistence::config::StorageBackendType;
    use std::sync::Mutex;

    // Mock tool for testing
    struct MockTool {
        metadata: ComponentMetadata,
        state_manager: Arc<parking_lot::RwLock<Option<Arc<dyn StateManager>>>>,
        execution_stats: Arc<Mutex<ToolExecutionStats>>,
        result_cache: Arc<Mutex<HashMap<String, CachedResult>>>,
        custom_state: Arc<Mutex<HashMap<String, Value>>>,
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
        ) -> llmspell_core::Result<AgentOutput> {
            Ok(AgentOutput::text("Mock output"))
        }

        async fn validate_input(&self, _input: &AgentInput) -> llmspell_core::Result<()> {
            Ok(())
        }

        async fn handle_error(&self, error: LLMSpellError) -> llmspell_core::Result<AgentOutput> {
            Ok(AgentOutput::text(format!("Error: {}", error)))
        }
    }

    #[async_trait]
    impl Tool for MockTool {
        fn category(&self) -> llmspell_core::traits::tool::ToolCategory {
            llmspell_core::traits::tool::ToolCategory::Utility
        }

        fn security_level(&self) -> llmspell_core::traits::tool::SecurityLevel {
            llmspell_core::traits::tool::SecurityLevel::Safe
        }

        fn schema(&self) -> llmspell_core::traits::tool::ToolSchema {
            // Return minimal schema for testing
            llmspell_core::traits::tool::ToolSchema::new(
                "mock_tool".to_string(),
                "A mock tool for testing".to_string(),
            )
        }
    }

    impl ToolStateManagerHolder for MockTool {
        fn state_manager(&self) -> Option<Arc<dyn StateManager>> {
            self.state_manager.read().clone()
        }

        fn set_state_manager(&self, state_manager: Arc<dyn StateManager>) {
            *self.state_manager.write() = Some(state_manager);
        }
    }

    #[async_trait]
    impl ToolStatePersistence for MockTool {
        fn state_manager(&self) -> Option<Arc<dyn StateManager>> {
            ToolStateManagerHolder::state_manager(self)
        }

        fn set_state_manager(&self, state_manager: Arc<dyn StateManager>) {
            ToolStateManagerHolder::set_state_manager(self, state_manager)
        }

        fn execution_statistics(&self) -> Option<ToolExecutionStats> {
            self.execution_stats.lock().ok().map(|stats| stats.clone())
        }

        fn result_cache(&self) -> Option<HashMap<String, CachedResult>> {
            self.result_cache.lock().ok().map(|cache| cache.clone())
        }

        fn custom_state(&self) -> Option<HashMap<String, Value>> {
            self.custom_state.lock().ok().map(|state| state.clone())
        }

        fn restore_execution_statistics(&self, stats: ToolExecutionStats) -> Result<()> {
            if let Ok(mut current_stats) = self.execution_stats.lock() {
                *current_stats = stats;
            }
            Ok(())
        }

        fn restore_result_cache(&self, cache: HashMap<String, CachedResult>) -> Result<()> {
            if let Ok(mut current_cache) = self.result_cache.lock() {
                *current_cache = cache;
            }
            Ok(())
        }

        fn restore_custom_state(&self, state: HashMap<String, Value>) -> Result<()> {
            if let Ok(mut current_state) = self.custom_state.lock() {
                *current_state = state;
            }
            Ok(())
        }
    }

    async fn create_test_state_manager() -> Arc<dyn StateManager> {
        let config = PersistenceConfig {
            enabled: true,
            ..Default::default()
        };

        Arc::new(
            llmspell_state_persistence::StateManager::with_backend(
                StorageBackendType::Memory,
                config,
            )
            .await
            .unwrap(),
        )
    }

    #[cfg_attr(test_category = "unit")]
    #[tokio::test]
    async fn test_tool_state_creation() {
        let metadata = ComponentMetadata::new("test-tool".to_string(), "Test tool".to_string());

        let tool_state = ToolState::new(metadata.id.to_string(), metadata.clone());

        assert_eq!(tool_state.tool_id, metadata.id.to_string());
        assert_eq!(tool_state.execution_stats.total_executions, 0);
        assert!(tool_state.result_cache.is_empty());
    }

    #[cfg_attr(test_category = "unit")]
    #[tokio::test]
    async fn test_tool_state_persistence() {
        let state_manager = create_test_state_manager().await;

        let metadata = ComponentMetadata::new("test-tool".to_string(), "Test tool".to_string());
        let tool = MockTool {
            metadata,
            state_manager: Arc::new(parking_lot::RwLock::new(None)),
            execution_stats: Arc::new(Mutex::new(ToolExecutionStats::default())),
            result_cache: Arc::new(Mutex::new(HashMap::new())),
            custom_state: Arc::new(Mutex::new(HashMap::new())),
        };

        ToolStateManagerHolder::set_state_manager(&tool, state_manager);

        // Should save and load successfully
        tool.save_state().await.unwrap();
        let loaded = tool.load_state().await.unwrap();
        assert!(loaded);
    }

    #[cfg_attr(test_category = "unit")]
    #[tokio::test]
    async fn test_tool_state_registry() {
        let state_manager = create_test_state_manager().await;
        let mut registry = ToolStateRegistry::new(state_manager);

        let metadata = ComponentMetadata::new("test-tool".to_string(), "Test tool".to_string());
        let tool = MockTool {
            metadata,
            state_manager: Arc::new(parking_lot::RwLock::new(None)),
            execution_stats: Arc::new(Mutex::new(ToolExecutionStats::default())),
            result_cache: Arc::new(Mutex::new(HashMap::new())),
            custom_state: Arc::new(Mutex::new(HashMap::new())),
        };

        let _registered_tool = registry.register_tool(tool).await.unwrap();

        let stats = registry.get_registry_stats().await;
        assert_eq!(stats.total_tools, 0); // MockTool doesn't add itself to registry states
    }
}
