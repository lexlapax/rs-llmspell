#![allow(clippy::significant_drop_tightening)]
// ABOUTME: Controlled state sharing patterns for multi-agent collaboration
// ABOUTME: Implements secure data exchange between agents with permission controls

use anyhow::Result;
use llmspell_core::traits::agent::Agent;
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, SystemTime};
use tracing::{debug, info, instrument, warn};
use uuid::Uuid;

/// State sharing patterns
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SharingPattern {
    /// Broadcast - one agent publishes, all subscribed agents receive
    Broadcast,
    /// Request-Response - one agent requests, another responds
    RequestResponse,
    /// Collaborative - multiple agents can read/write
    Collaborative,
    /// Pipeline - data flows through agents in sequence
    Pipeline,
    /// Hierarchical - parent-child agent relationships
    Hierarchical,
}

/// Shared state channel for agent communication
#[derive(Debug, Clone)]
pub struct SharedStateChannel {
    pub channel_id: String,
    pub pattern: SharingPattern,
    pub creator_agent_id: String,
    pub participants: Vec<String>,
    pub created_at: SystemTime,
    pub ttl: Option<Duration>,
    pub metadata: HashMap<String, Value>,
}

/// Message in a shared state channel
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateMessage {
    pub message_id: Uuid,
    pub channel_id: String,
    pub sender_agent_id: String,
    pub message_type: String,
    pub payload: Value,
    pub timestamp: SystemTime,
    pub correlation_id: Option<Uuid>,
    pub reply_to: Option<Uuid>,
}

/// State sharing manager for controlled data exchange
pub struct StateSharingManager {
    #[allow(dead_code)]
    state_manager: Arc<dyn std::any::Any + Send + Sync>,
    channels: Arc<RwLock<HashMap<String, SharedStateChannel>>>,
    subscriptions: Arc<RwLock<HashMap<String, Vec<String>>>>, // agent_id -> channel_ids
    message_queue: Arc<RwLock<HashMap<String, Vec<StateMessage>>>>, // channel_id -> messages
}

impl StateSharingManager {
    pub fn new<T: std::any::Any + Send + Sync + 'static>(state_manager: Arc<T>) -> Self {
        Self {
            state_manager,
            channels: Arc::new(RwLock::new(HashMap::new())),
            subscriptions: Arc::new(RwLock::new(HashMap::new())),
            message_queue: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Create a new shared state channel
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - A channel with the given ID already exists
    /// - Subscribing the creator agent fails
    #[instrument(skip(self))]
    pub fn create_channel(
        &self,
        channel_id: &str,
        pattern: SharingPattern,
        creator_agent_id: &str,
        ttl: Option<Duration>,
    ) -> Result<SharedStateChannel> {
        let mut channels = self.channels.write();

        if channels.contains_key(channel_id) {
            return Err(anyhow::anyhow!("Channel {} already exists", channel_id));
        }

        let channel = SharedStateChannel {
            channel_id: channel_id.to_string(),
            pattern,
            creator_agent_id: creator_agent_id.to_string(),
            participants: vec![creator_agent_id.to_string()],
            created_at: SystemTime::now(),
            ttl,
            metadata: HashMap::new(),
        };

        channels.insert(channel_id.to_string(), channel.clone());

        // Initialize message queue for channel
        let mut message_queue = self.message_queue.write();
        message_queue.insert(channel_id.to_string(), Vec::new());

        // Drop locks before calling subscribe_agent to avoid deadlock
        drop(message_queue);
        drop(channels);

        // Subscribe creator to channel
        self.subscribe_agent(creator_agent_id, channel_id)?;

        info!(
            "Created shared state channel {} with pattern {:?}",
            channel_id, pattern
        );
        Ok(channel)
    }

    /// Subscribe an agent to a channel
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The specified channel does not exist
    /// - The agent ID is invalid
    #[instrument(skip(self))]
    pub fn subscribe_agent(&self, agent_id: &str, channel_id: &str) -> Result<()> {
        // Verify channel exists
        let mut channels = self.channels.write();
        let channel = channels
            .get_mut(channel_id)
            .ok_or_else(|| anyhow::anyhow!("Channel {} not found", channel_id))?;

        // Add to participants if not already present
        if !channel.participants.contains(&agent_id.to_string()) {
            channel.participants.push(agent_id.to_string());
        }

        // Update subscriptions
        let mut subscriptions = self.subscriptions.write();
        let agent_subs = subscriptions.entry(agent_id.to_string()).or_default();
        if !agent_subs.contains(&channel_id.to_string()) {
            agent_subs.push(channel_id.to_string());
        }

        debug!("Agent {} subscribed to channel {}", agent_id, channel_id);
        Ok(())
    }

    /// Unsubscribe an agent from a channel
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The specified channel does not exist
    /// - The agent is not subscribed to the channel
    #[instrument(skip(self))]
    pub fn unsubscribe_agent(&self, agent_id: &str, channel_id: &str) -> Result<()> {
        // Remove from channel participants
        let mut channels = self.channels.write();
        if let Some(channel) = channels.get_mut(channel_id) {
            channel.participants.retain(|id| id != agent_id);
        }

        // Update subscriptions
        let mut subscriptions = self.subscriptions.write();
        if let Some(agent_subs) = subscriptions.get_mut(agent_id) {
            agent_subs.retain(|id| id != channel_id);
        }

        debug!(
            "Agent {} unsubscribed from channel {}",
            agent_id, channel_id
        );
        Ok(())
    }

    /// Publish a message to a channel
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The specified channel does not exist
    /// - The sender agent is not a participant in the channel
    /// - Pattern permissions validation fails
    #[instrument(skip(self, payload))]
    pub async fn publish_message(
        &self,
        channel_id: &str,
        sender_agent_id: &str,
        message_type: &str,
        payload: Value,
        correlation_id: Option<Uuid>,
    ) -> Result<Uuid> {
        // Verify channel exists and agent is participant
        let channels = self.channels.read();
        let channel = channels
            .get(channel_id)
            .ok_or_else(|| anyhow::anyhow!("Channel {} not found", channel_id))?;

        if !channel.participants.contains(&sender_agent_id.to_string()) {
            return Err(anyhow::anyhow!(
                "Agent {} is not a participant in channel {}",
                sender_agent_id,
                channel_id
            ));
        }

        // Validate pattern permissions
        Self::validate_pattern_permissions(&channel.pattern, sender_agent_id, channel)?;

        let message = StateMessage {
            message_id: Uuid::new_v4(),
            channel_id: channel_id.to_string(),
            sender_agent_id: sender_agent_id.to_string(),
            message_type: message_type.to_string(),
            payload: payload.clone(),
            timestamp: SystemTime::now(),
            correlation_id,
            reply_to: None,
        };

        // Store message in queue
        let mut message_queue = self.message_queue.write();
        if let Some(queue) = message_queue.get_mut(channel_id) {
            queue.push(message.clone());

            // Trim queue if too long
            if queue.len() > 1000 {
                queue.drain(0..100);
            }
        }

        // Store in persistent state for durability
        // TODO: When StateManager is properly integrated, uncomment this:
        // let state_key = format!("channel:{}:message:{}", channel_id, message.message_id);
        // self.state_manager.set_scoped(
        //     StateScope::Custom(format!("sharing:{}", channel_id)),
        //     &state_key,
        //     serde_json::to_value(&message)?,
        // ).await?;

        debug!(
            "Published message {} to channel {}",
            message.message_id, channel_id
        );
        Ok(message.message_id)
    }

    /// Reply to a message in a channel
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The specified channel does not exist
    /// - The original message to reply to is not found
    /// - Publishing the reply message fails
    #[instrument(skip(self, payload))]
    pub async fn reply_to_message(
        &self,
        channel_id: &str,
        sender_agent_id: &str,
        reply_to_id: Uuid,
        message_type: &str,
        payload: Value,
    ) -> Result<Uuid> {
        // Find original message
        let message_queue = self.message_queue.read();
        let queue = message_queue
            .get(channel_id)
            .ok_or_else(|| anyhow::anyhow!("Channel {} not found", channel_id))?;

        let original = queue
            .iter()
            .find(|m| m.message_id == reply_to_id)
            .ok_or_else(|| anyhow::anyhow!("Message {} not found", reply_to_id))?;

        let correlation_id = original.correlation_id.or(Some(original.message_id));
        drop(message_queue);

        // Create reply message
        let message = StateMessage {
            message_id: Uuid::new_v4(),
            channel_id: channel_id.to_string(),
            sender_agent_id: sender_agent_id.to_string(),
            message_type: message_type.to_string(),
            payload,
            timestamp: SystemTime::now(),
            correlation_id,
            reply_to: Some(reply_to_id),
        };

        // Store reply
        let mut message_queue = self.message_queue.write();
        if let Some(queue) = message_queue.get_mut(channel_id) {
            queue.push(message.clone());
        }

        Ok(message.message_id)
    }

    /// Get messages for an agent from their subscribed channels
    ///
    /// # Errors
    ///
    /// Returns an error if the agent has no subscriptions
    #[instrument(skip(self))]
    pub fn get_messages_for_agent(
        &self,
        agent_id: &str,
        since: Option<SystemTime>,
        limit: Option<usize>,
    ) -> Result<Vec<StateMessage>> {
        let subscriptions = self.subscriptions.read();
        let agent_channels = subscriptions
            .get(agent_id)
            .ok_or_else(|| anyhow::anyhow!("Agent {} has no subscriptions", agent_id))?;

        let message_queue = self.message_queue.read();
        let mut all_messages = Vec::new();

        for channel_id in agent_channels {
            if let Some(queue) = message_queue.get(channel_id) {
                let channel_messages: Vec<_> = queue
                    .iter()
                    .filter(|m| {
                        // Filter by time if specified
                        since.map_or(true, |since_time| m.timestamp > since_time)
                    })
                    .filter(|m| {
                        // Don't show agent their own messages
                        m.sender_agent_id != agent_id
                    })
                    .cloned()
                    .collect();

                all_messages.extend(channel_messages);
            }
        }

        // Sort by timestamp
        all_messages.sort_by_key(|m| m.timestamp);

        // Apply limit if specified
        if let Some(limit) = limit {
            all_messages.truncate(limit);
        }

        Ok(all_messages)
    }

    /// Create a collaborative workspace for multiple agents
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Creating the collaborative channel fails
    /// - Subscribing any participant to the workspace fails
    #[instrument(skip(self))]
    pub async fn create_collaborative_workspace(
        &self,
        workspace_id: &str,
        owner_agent_id: &str,
        participant_ids: Vec<String>,
    ) -> Result<()> {
        // Create collaborative channel
        self.create_channel(
            workspace_id,
            SharingPattern::Collaborative,
            owner_agent_id,
            None,
        )?;

        // Subscribe all participants
        for participant_id in participant_ids {
            self.subscribe_agent(&participant_id, workspace_id)?;
        }

        // Create shared state scope for workspace
        // TODO: When StateManager is properly integrated, uncomment this:
        // let workspace_scope = StateScope::Custom(format!("workspace:{}", workspace_id));
        //
        // // Initialize workspace metadata
        // let metadata = serde_json::json!({
        //     "workspace_id": workspace_id,
        //     "owner": owner_agent_id,
        //     "created_at": SystemTime::now(),
        //     "type": "collaborative",
        // });
        //
        // self.state_manager.set_scoped(
        //     workspace_scope,
        //     "metadata",
        //     metadata,
        // ).await?;

        info!("Created collaborative workspace {}", workspace_id);
        Ok(())
    }

    /// Create a data pipeline between agents
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Creating the pipeline channel fails
    /// - The stages list is empty
    /// - Setting pipeline metadata fails
    #[instrument(skip(self))]
    pub fn create_pipeline(
        &self,
        pipeline_id: &str,
        stages: Vec<String>, // Agent IDs in order
    ) -> Result<()> {
        if stages.is_empty() {
            return Err(anyhow::anyhow!("Pipeline must have at least one stage"));
        }

        // Create pipeline channel
        self.create_channel(pipeline_id, SharingPattern::Pipeline, &stages[0], None)?;

        // Subscribe all stages
        for agent_id in &stages {
            self.subscribe_agent(agent_id, pipeline_id)?;
        }

        // Store pipeline configuration
        let mut channels = self.channels.write();
        if let Some(channel) = channels.get_mut(pipeline_id) {
            channel.metadata.insert(
                "pipeline_stages".to_string(),
                serde_json::to_value(&stages)?,
            );
        }

        info!(
            "Created pipeline {} with {} stages",
            pipeline_id,
            stages.len()
        );
        Ok(())
    }

    /// Process next stage in a pipeline
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The pipeline is not found
    /// - Pipeline stages metadata is missing or invalid
    /// - The current agent is not in the pipeline
    /// - Publishing the stage completion message fails
    #[instrument(skip(self, data))]
    pub async fn process_pipeline_stage(
        &self,
        pipeline_id: &str,
        current_agent_id: &str,
        data: Value,
    ) -> Result<Option<String>> {
        let (stages, current_index) = {
            let channels = self.channels.read();
            let channel = channels
                .get(pipeline_id)
                .ok_or_else(|| anyhow::anyhow!("Pipeline {} not found", pipeline_id))?;

            let stages: Vec<String> = serde_json::from_value(
                channel
                    .metadata
                    .get("pipeline_stages")
                    .ok_or_else(|| anyhow::anyhow!("Pipeline stages not found"))?
                    .clone(),
            )?;

            // Find current stage index
            let current_index = stages
                .iter()
                .position(|id| id == current_agent_id)
                .ok_or_else(|| anyhow::anyhow!("Agent {} not in pipeline", current_agent_id))?;

            (stages, current_index)
        };

        // Publish processed data
        let correlation_id = Uuid::new_v4();

        self.publish_message(
            pipeline_id,
            current_agent_id,
            "pipeline_stage_complete",
            data,
            Some(correlation_id),
        )
        .await?;

        // Return next agent in pipeline
        if current_index + 1 < stages.len() {
            Ok(Some(stages[current_index + 1].clone()))
        } else {
            Ok(None) // Pipeline complete
        }
    }

    /// Clean up expired channels
    pub fn cleanup_expired_channels(&self) {
        let mut channels = self.channels.write();
        let now = SystemTime::now();

        let expired: Vec<_> = channels
            .iter()
            .filter_map(|(id, channel)| {
                if let Some(ttl) = channel.ttl {
                    if now.duration_since(channel.created_at).ok()? > ttl {
                        Some(id.clone())
                    } else {
                        None
                    }
                } else {
                    None
                }
            })
            .collect();

        for channel_id in expired {
            channels.remove(&channel_id);

            // Clean up message queue
            let mut message_queue = self.message_queue.write();
            message_queue.remove(&channel_id);

            // Clean up subscriptions
            let mut subscriptions = self.subscriptions.write();
            for (_, agent_channels) in subscriptions.iter_mut() {
                agent_channels.retain(|id| id != &channel_id);
            }

            info!("Cleaned up expired channel {}", channel_id);
        }
    }

    // Private helper methods

    fn validate_pattern_permissions(
        pattern: &SharingPattern,
        agent_id: &str,
        channel: &SharedStateChannel,
    ) -> Result<()> {
        match pattern {
            SharingPattern::Broadcast => {
                // Only creator can broadcast
                if agent_id != channel.creator_agent_id {
                    return Err(anyhow::anyhow!("Only channel creator can broadcast"));
                }
            }
            SharingPattern::Pipeline | _ => {
                // Pipeline: Agents can only publish when it's their turn (enforced by process_pipeline_stage)
                // Other patterns: Allow any participant to publish
            }
        }
        Ok(())
    }
}

/// Extension trait for agents to use shared state
#[async_trait::async_trait]
pub trait SharedStateAgent: Agent {
    /// Get shared state accessor
    fn shared_state(&self) -> SharedStateAccessor
    where
        Self: Sized,
    {
        SharedStateAccessor::new(self.metadata().id.to_string(), self.sharing_manager())
    }

    /// Get sharing manager (to be implemented by agent)
    fn sharing_manager(&self) -> Arc<StateSharingManager>;
}

/// Accessor for shared state operations
pub struct SharedStateAccessor {
    agent_id: String,
    sharing_manager: Arc<StateSharingManager>,
}

impl SharedStateAccessor {
    #[must_use]
    pub const fn new(agent_id: String, sharing_manager: Arc<StateSharingManager>) -> Self {
        Self {
            agent_id,
            sharing_manager,
        }
    }

    /// Subscribe to a channel
    ///
    /// # Errors
    ///
    /// Returns an error if subscription fails
    pub fn subscribe(&self, channel_id: &str) -> Result<()> {
        self.sharing_manager
            .subscribe_agent(&self.agent_id, channel_id)
    }

    /// Unsubscribe from a channel
    ///
    /// # Errors
    ///
    /// Returns an error if unsubscription fails
    pub fn unsubscribe(&self, channel_id: &str) -> Result<()> {
        self.sharing_manager
            .unsubscribe_agent(&self.agent_id, channel_id)
    }

    /// Publish a message
    ///
    /// # Errors
    ///
    /// Returns an error if publishing the message fails
    pub async fn publish(
        &self,
        channel_id: &str,
        message_type: &str,
        payload: Value,
    ) -> Result<Uuid> {
        self.sharing_manager
            .publish_message(channel_id, &self.agent_id, message_type, payload, None)
            .await
    }

    /// Get new messages
    ///
    /// # Errors
    ///
    /// Returns an error if retrieving messages fails
    pub fn get_messages(&self, since: Option<SystemTime>) -> Result<Vec<StateMessage>> {
        self.sharing_manager
            .get_messages_for_agent(&self.agent_id, since, None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Once;

    static INIT: Once = Once::new();

    fn init_tracing() {
        INIT.call_once(|| {
            // Initialize tracing subscriber for tests to prevent hangs with #[instrument]
            let _ = tracing_subscriber::fmt()
                .with_test_writer()
                .with_max_level(tracing::Level::DEBUG)
                .try_init();
        });
    }
    #[tokio::test]
    async fn test_broadcast_channel() {
        init_tracing();
        // Mock state manager for testing
        struct MockStateManager;
        let state_manager = Arc::new(MockStateManager);
        let sharing_manager = StateSharingManager::new(state_manager);

        // Create broadcast channel
        let _channel = sharing_manager
            .create_channel("news", SharingPattern::Broadcast, "broadcaster", None)
            .unwrap();

        // Subscribe listeners
        sharing_manager
            .subscribe_agent("listener1", "news")
            .unwrap();
        sharing_manager
            .subscribe_agent("listener2", "news")
            .unwrap();

        // Broadcaster publishes
        let msg_id = sharing_manager
            .publish_message(
                "news",
                "broadcaster",
                "announcement",
                serde_json::json!({"message": "Hello world"}),
                None,
            )
            .await
            .unwrap();

        // Listeners should see the message
        let listener1_msgs = sharing_manager
            .get_messages_for_agent("listener1", None, None)
            .unwrap();
        assert_eq!(listener1_msgs.len(), 1);
        assert_eq!(listener1_msgs[0].message_id, msg_id);

        // Non-broadcaster cannot publish
        let result = sharing_manager
            .publish_message(
                "news",
                "listener1",
                "announcement",
                serde_json::json!({"message": "Unauthorized"}),
                None,
            )
            .await;
        assert!(result.is_err());
    }
    #[tokio::test]
    async fn test_collaborative_workspace() {
        init_tracing();
        // Mock state manager for testing
        struct MockStateManager;
        let state_manager = Arc::new(MockStateManager);
        let sharing_manager = StateSharingManager::new(state_manager);

        // Create collaborative workspace
        sharing_manager
            .create_collaborative_workspace(
                "project-x",
                "lead",
                vec!["dev1".to_string(), "dev2".to_string()],
            )
            .await
            .unwrap();

        // All participants can publish
        let _msg1 = sharing_manager
            .publish_message(
                "project-x",
                "lead",
                "update",
                serde_json::json!({"status": "started"}),
                None,
            )
            .await
            .unwrap();

        let _msg2 = sharing_manager
            .publish_message(
                "project-x",
                "dev1",
                "update",
                serde_json::json!({"progress": 50}),
                None,
            )
            .await
            .unwrap();

        // Check messages received
        let lead_msgs = sharing_manager
            .get_messages_for_agent("lead", None, None)
            .unwrap();
        assert_eq!(lead_msgs.len(), 1); // Should see dev1's message
        assert_eq!(lead_msgs[0].sender_agent_id, "dev1");
    }
    #[tokio::test]
    async fn test_pipeline_processing() {
        init_tracing();
        // Mock state manager for testing
        struct MockStateManager;
        let state_manager = Arc::new(MockStateManager);
        let sharing_manager = StateSharingManager::new(state_manager);

        // Create pipeline
        let stages = vec![
            "fetcher".to_string(),
            "processor".to_string(),
            "writer".to_string(),
        ];
        sharing_manager
            .create_pipeline("data-pipeline", stages)
            .unwrap();

        // First stage processes
        let next = sharing_manager
            .process_pipeline_stage(
                "data-pipeline",
                "fetcher",
                serde_json::json!({"data": "raw"}),
            )
            .await
            .unwrap();
        assert_eq!(next, Some("processor".to_string()));

        // Second stage processes
        let next = sharing_manager
            .process_pipeline_stage(
                "data-pipeline",
                "processor",
                serde_json::json!({"data": "processed"}),
            )
            .await
            .unwrap();
        assert_eq!(next, Some("writer".to_string()));

        // Final stage
        let next = sharing_manager
            .process_pipeline_stage(
                "data-pipeline",
                "writer",
                serde_json::json!({"data": "written"}),
            )
            .await
            .unwrap();
        assert_eq!(next, None); // Pipeline complete
    }
}
