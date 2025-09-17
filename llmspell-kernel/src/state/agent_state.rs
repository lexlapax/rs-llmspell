// ABOUTME: Agent state persistence structures and serialization
// ABOUTME: Implements StorageSerialize for agent state with Phase 4 hook integration

use super::sensitive_data::SensitiveDataConfig;
use super::StateResult;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::SystemTime;
use uuid::Uuid;

/// Persistent agent state structure with full serialization support
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersistentAgentState {
    /// Unique identifier for the agent instance
    pub agent_id: String,
    /// Type classification of the agent (e.g., "assistant", "reviewer", "analyzer")
    pub agent_type: String,
    /// Core operational state data for the agent
    pub state: AgentStateData,
    /// Descriptive metadata about agent capabilities and configuration
    pub metadata: AgentMetadata,
    /// Timestamp of when this agent state was initially created
    pub creation_time: SystemTime,
    /// Timestamp of the most recent state modification
    pub last_modified: SystemTime,
    /// Version number of the state schema for migration compatibility
    pub schema_version: u32,

    // Hook integration fields (Phase 4)
    /// List of registered hook identifiers for state change notifications
    pub hook_registrations: Vec<String>,
    /// Timestamp of the last successful hook execution
    pub last_hook_execution: Option<SystemTime>,
    /// Correlation UUID for tracking related operations across the system
    pub correlation_context: Option<Uuid>,
}

/// Core agent state data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentStateData {
    /// Historical record of conversation messages and interactions
    pub conversation_history: Vec<ConversationMessage>,
    /// Dynamic context variables for agent decision-making
    pub context_variables: HashMap<String, serde_json::Value>,
    /// Aggregated statistics on tool usage patterns and performance
    pub tool_usage_stats: ToolUsageStats,
    /// Current execution state and workflow position
    pub execution_state: ExecutionState,
    /// Application-specific custom data storage
    pub custom_data: HashMap<String, serde_json::Value>,
}

/// Agent metadata for persistence
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentMetadata {
    /// Human-readable name for the agent
    pub name: String,
    /// Optional detailed description of agent purpose and behavior
    pub description: Option<String>,
    /// Semantic version string for agent implementation
    pub version: String,
    /// List of capabilities this agent can perform
    pub capabilities: Vec<String>,
    /// Provider-specific configuration (e.g., model settings, API keys)
    pub provider_config: Option<serde_json::Value>,
    /// Categorization tags for agent organization and discovery
    pub tags: Vec<String>,
}

/// Conversation message structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversationMessage {
    /// Role of the message sender in the conversation
    pub role: MessageRole,
    /// Actual message content text
    pub content: String,
    /// When this message was created
    pub timestamp: SystemTime,
    /// Optional metadata for tool calls, attachments, or other context
    pub metadata: Option<HashMap<String, serde_json::Value>>,
}

/// Role of a participant in the conversation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MessageRole {
    /// System-level instructions or prompts
    System,
    /// Human user input
    User,
    /// AI assistant response
    Assistant,
    /// Tool execution result or output
    Tool,
}

/// Tool usage statistics
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ToolUsageStats {
    /// Total number of tool invocations attempted
    pub total_invocations: u64,
    /// Number of successful tool executions
    pub successful_invocations: u64,
    /// Number of failed tool executions
    pub failed_invocations: u64,
    /// Per-tool performance metrics indexed by tool name
    pub tool_performance: HashMap<String, ToolPerformance>,
}

/// Performance metrics for individual tool usage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolPerformance {
    /// Number of times this tool has been invoked
    pub invocation_count: u64,
    /// Cumulative execution time in milliseconds
    pub total_duration_ms: u64,
    /// Average execution time per invocation in milliseconds
    pub average_duration_ms: f64,
    /// Most recent usage timestamp
    pub last_used: SystemTime,
}

/// Agent execution state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ExecutionState {
    /// Agent is ready but not actively processing
    Idle,
    /// Agent is actively processing a request
    Processing,
    /// Agent is waiting for user input to continue
    WaitingForInput,
    /// Agent is waiting for a tool to complete execution
    WaitingForTool,
    /// Agent execution is temporarily suspended
    Suspended,
    /// Agent has completed its task successfully
    Completed,
    /// Agent execution failed with an error message
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
        #[allow(clippy::cast_precision_loss)]
        let total_duration_f64 = performance.total_duration_ms as f64;
        #[allow(clippy::cast_precision_loss)]
        let invocation_count_f64 = performance.invocation_count as f64;
        performance.average_duration_ms = total_duration_f64 / invocation_count_f64;
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
        use super::performance::UnifiedSerializer;

        let serializer = UnifiedSerializer::new(SensitiveDataConfig::default());
        serializer.serialize(self)
    }

    /// Deserialize from storage bytes (no special handling needed on read)
    pub fn safe_from_storage_bytes(bytes: &[u8]) -> StateResult<Self> {
        // Use unified serializer for deserialization
        use super::performance::UnifiedSerializer;

        let serializer = UnifiedSerializer::new(SensitiveDataConfig::default());
        serializer.deserialize(bytes)
    }

    /// Fast serialization for benchmarks (no protection)
    pub fn fast_to_bytes(&self) -> StateResult<Vec<u8>> {
        use super::performance::UnifiedSerializer;

        let serializer = UnifiedSerializer::fast();
        serializer.serialize(self)
    }

    /// Fast deserialization for benchmarks
    pub fn fast_from_bytes(bytes: &[u8]) -> StateResult<Self> {
        use super::performance::UnifiedSerializer;

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
    async fn save_state(
        &self,
        state_manager: &crate::state::manager::StateManager,
    ) -> StateResult<()> {
        let state = self.get_persistent_state()?;
        state_manager.save_agent_state(&state).await
    }

    /// Load the agent's state
    async fn load_state(
        &mut self,
        state_manager: &crate::state::manager::StateManager,
    ) -> StateResult<()> {
        if let Some(state) = state_manager.load_agent_state(self.agent_id()).await? {
            self.apply_persistent_state(state)?;
        }
        Ok(())
    }

    /// Delete the agent's state
    async fn delete_state(
        &self,
        state_manager: &crate::state::manager::StateManager,
    ) -> StateResult<()> {
        state_manager.delete_agent_state(self.agent_id()).await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_persistent_agent_state_creation() {
        let state = PersistentAgentState::new("agent_123".to_string(), "assistant".to_string());

        assert_eq!(state.agent_id, "agent_123");
        assert_eq!(state.agent_type, "assistant");
        assert_eq!(state.schema_version, 1);
        assert!(state.hook_registrations.is_empty());
        assert!(state.correlation_context.is_none());
    }
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
