// ABOUTME: Agent state persistence structures and serialization
// ABOUTME: Implements StorageSerialize for agent state with Phase 4 hook integration

use crate::sensitive_data::SensitiveDataConfig;
use llmspell_state_traits::StateResult;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::SystemTime;
use uuid::Uuid;

/// Persistent agent state structure with full serialization support
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersistentAgentState {
    pub agent_id: String,
    pub agent_type: String,
    pub state: AgentStateData,
    pub metadata: AgentMetadata,
    pub creation_time: SystemTime,
    pub last_modified: SystemTime,
    pub schema_version: u32,

    // Hook integration fields (Phase 4)
    pub hook_registrations: Vec<String>,
    pub last_hook_execution: Option<SystemTime>,
    pub correlation_context: Option<Uuid>,
}

/// Core agent state data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentStateData {
    pub conversation_history: Vec<ConversationMessage>,
    pub context_variables: HashMap<String, serde_json::Value>,
    pub tool_usage_stats: ToolUsageStats,
    pub execution_state: ExecutionState,
    pub custom_data: HashMap<String, serde_json::Value>,
}

/// Agent metadata for persistence
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentMetadata {
    pub name: String,
    pub description: Option<String>,
    pub version: String,
    pub capabilities: Vec<String>,
    pub provider_config: Option<serde_json::Value>,
    pub tags: Vec<String>,
}

/// Conversation message structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversationMessage {
    pub role: MessageRole,
    pub content: String,
    pub timestamp: SystemTime,
    pub metadata: Option<HashMap<String, serde_json::Value>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MessageRole {
    System,
    User,
    Assistant,
    Tool,
}

/// Tool usage statistics
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ToolUsageStats {
    pub total_invocations: u64,
    pub successful_invocations: u64,
    pub failed_invocations: u64,
    pub tool_performance: HashMap<String, ToolPerformance>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolPerformance {
    pub invocation_count: u64,
    pub total_duration_ms: u64,
    pub average_duration_ms: f64,
    pub last_used: SystemTime,
}

/// Agent execution state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ExecutionState {
    Idle,
    Processing,
    WaitingForInput,
    WaitingForTool,
    Suspended,
    Completed,
    Failed(String),
}

impl PersistentAgentState {
    /// Create a new persistent agent state
    pub fn new(agent_id: String, agent_type: String) -> Self {
        Self {
            agent_id,
            agent_type,
            state: AgentStateData::default(),
            metadata: AgentMetadata::default(),
            creation_time: SystemTime::now(),
            last_modified: SystemTime::now(),
            schema_version: 1,
            hook_registrations: Vec::new(),
            last_hook_execution: None,
            correlation_context: None,
        }
    }

    /// Update the last modified timestamp
    pub fn touch(&mut self) {
        self.last_modified = SystemTime::now();
    }

    /// Add a conversation message
    pub fn add_message(&mut self, role: MessageRole, content: String) {
        self.state.conversation_history.push(ConversationMessage {
            role,
            content,
            timestamp: SystemTime::now(),
            metadata: None,
        });
        self.touch();
    }

    /// Update tool usage statistics
    pub fn record_tool_usage(&mut self, tool_name: &str, duration_ms: u64, success: bool) {
        self.state.tool_usage_stats.total_invocations += 1;
        if success {
            self.state.tool_usage_stats.successful_invocations += 1;
        } else {
            self.state.tool_usage_stats.failed_invocations += 1;
        }

        let performance = self
            .state
            .tool_usage_stats
            .tool_performance
            .entry(tool_name.to_string())
            .or_insert(ToolPerformance {
                invocation_count: 0,
                total_duration_ms: 0,
                average_duration_ms: 0.0,
                last_used: SystemTime::now(),
            });

        performance.invocation_count += 1;
        performance.total_duration_ms += duration_ms;
        performance.average_duration_ms =
            performance.total_duration_ms as f64 / performance.invocation_count as f64;
        performance.last_used = SystemTime::now();

        self.touch();
    }
}

impl Default for AgentStateData {
    fn default() -> Self {
        Self {
            conversation_history: Vec::new(),
            context_variables: HashMap::new(),
            tool_usage_stats: ToolUsageStats::default(),
            execution_state: ExecutionState::Idle,
            custom_data: HashMap::new(),
        }
    }
}

impl Default for AgentMetadata {
    fn default() -> Self {
        Self {
            name: String::new(),
            description: None,
            version: "1.0.0".to_string(),
            capabilities: Vec::new(),
            provider_config: None,
            tags: Vec::new(),
        }
    }
}

// StorageSerialize is automatically implemented via blanket implementation
// for all types that implement Serialize + Deserialize

impl PersistentAgentState {
    /// Serialize with circular reference check and sensitive data protection
    pub fn safe_to_storage_bytes(&self) -> StateResult<Vec<u8>> {
        // Use unified serializer for single-pass serialization
        use crate::performance::UnifiedSerializer;

        let serializer = UnifiedSerializer::new(SensitiveDataConfig::default());
        serializer.serialize(self)
    }

    /// Deserialize from storage bytes (no special handling needed on read)
    pub fn safe_from_storage_bytes(bytes: &[u8]) -> StateResult<Self> {
        // Use unified serializer for deserialization
        use crate::performance::UnifiedSerializer;

        let serializer = UnifiedSerializer::new(SensitiveDataConfig::default());
        serializer.deserialize(bytes)
    }

    /// Fast serialization for benchmarks (no protection)
    pub fn fast_to_bytes(&self) -> StateResult<Vec<u8>> {
        use crate::performance::UnifiedSerializer;

        let serializer = UnifiedSerializer::fast();
        serializer.serialize(self)
    }

    /// Fast deserialization for benchmarks
    pub fn fast_from_bytes(bytes: &[u8]) -> StateResult<Self> {
        use crate::performance::UnifiedSerializer;

        let serializer = UnifiedSerializer::fast();
        serializer.deserialize(bytes)
    }
}

/// Agent state operations trait
#[async_trait::async_trait]
pub trait PersistentAgent {
    /// Get the agent's ID
    fn agent_id(&self) -> &str;

    /// Get the current persistent state
    fn get_persistent_state(&self) -> StateResult<PersistentAgentState>;

    /// Apply a persistent state to the agent
    fn apply_persistent_state(&self, state: PersistentAgentState) -> StateResult<()>;

    /// Save the agent's state
    async fn save_state(&self, state_manager: &crate::manager::StateManager) -> StateResult<()> {
        let state = self.get_persistent_state()?;
        state_manager.save_agent_state(&state).await
    }

    /// Load the agent's state
    async fn load_state(
        &mut self,
        state_manager: &crate::manager::StateManager,
    ) -> StateResult<()> {
        if let Some(state) = state_manager.load_agent_state(self.agent_id()).await? {
            self.apply_persistent_state(state)?;
        }
        Ok(())
    }

    /// Delete the agent's state
    async fn delete_state(&self, state_manager: &crate::manager::StateManager) -> StateResult<()> {
        state_manager.delete_agent_state(self.agent_id()).await?;
        Ok(())
    }
}

#[cfg(test)]
#[cfg_attr(test_category = "state")]
mod tests {
    use super::*;

    #[cfg_attr(test_category = "unit")]
    #[test]
    fn test_persistent_agent_state_creation() {
        let state = PersistentAgentState::new("agent_123".to_string(), "assistant".to_string());

        assert_eq!(state.agent_id, "agent_123");
        assert_eq!(state.agent_type, "assistant");
        assert_eq!(state.schema_version, 1);
        assert!(state.hook_registrations.is_empty());
        assert!(state.correlation_context.is_none());
    }

    #[cfg_attr(test_category = "unit")]
    #[test]
    fn test_agent_state_serialization() {
        use llmspell_storage::StorageSerialize;

        let mut state =
            PersistentAgentState::new("agent_456".to_string(), "researcher".to_string());

        // Add some data
        state.add_message(MessageRole::User, "Hello".to_string());
        state.add_message(MessageRole::Assistant, "Hi there!".to_string());
        state.record_tool_usage("web_search", 150, true);

        // Serialize and deserialize using StorageSerialize trait
        let bytes = state.to_storage_bytes().unwrap();
        let restored = PersistentAgentState::from_storage_bytes(&bytes).unwrap();

        assert_eq!(restored.agent_id, state.agent_id);
        assert_eq!(restored.state.conversation_history.len(), 2);
        assert_eq!(restored.state.tool_usage_stats.total_invocations, 1);
    }
}
