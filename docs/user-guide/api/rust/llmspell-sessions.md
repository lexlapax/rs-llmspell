# llmspell-sessions

**Session management with artifact storage and replay**

**🔗 Navigation**: [← Rust API](README.md) | [Crate Docs](https://docs.rs/llmspell-sessions) | [Source](../../../../llmspell-sessions)

---

## Overview

`llmspell-sessions` provides comprehensive session management including lifecycle control, artifact storage, replay capabilities, and security contexts. Sessions encapsulate the entire execution context of LLMSpell scripts, enabling persistence, debugging, and reproducibility.

**Key Features:**
- 🎯 Session lifecycle management
- 📦 Artifact storage and retrieval
- 🔄 Session replay and debugging
- 🔐 Security context isolation
- 📊 Session metrics and analytics
- 🏷️ Tagging and categorization
- 🔍 Session search and filtering
- ⏱️ Time-travel debugging

## Core Components

### Session Trait

The fundamental session interface:

```rust
use async_trait::async_trait;
use serde_json::Value;
use uuid::Uuid;

#[async_trait]
pub trait Session: Send + Sync {
    /// Get session ID
    fn id(&self) -> &Uuid;
    
    /// Get session metadata
    fn metadata(&self) -> &SessionMetadata;
    
    /// Store an artifact in the session
    async fn store_artifact(&self, artifact: Artifact) -> Result<ArtifactId>;
    
    /// Retrieve an artifact from the session
    async fn get_artifact(&self, id: &ArtifactId) -> Result<Option<Artifact>>;
    
    /// List all artifacts in the session
    async fn list_artifacts(&self) -> Result<Vec<ArtifactMetadata>>;
    
    /// Record an event in the session
    async fn record_event(&self, event: SessionEvent) -> Result<()>;
    
    /// Get session events
    async fn get_events(&self, filter: Option<EventFilter>) -> Result<Vec<SessionEvent>>;
    
    /// Update session state
    async fn update_state(&self, key: &str, value: Value) -> Result<()>;
    
    /// Get session state
    async fn get_state(&self, key: &str) -> Result<Option<Value>>;
    
    /// End the session
    async fn end(&self) -> Result<()>;
}
```

### SessionManager

Manages session lifecycle and storage:

```rust
pub struct SessionManager {
    storage: Arc<dyn SessionStorage>,
    config: SessionConfig,
    active_sessions: Arc<RwLock<HashMap<Uuid, Arc<dyn Session>>>>,
    metrics: Arc<SessionMetrics>,
}

impl SessionManager {
    /// Create a new session manager
    pub fn new(config: SessionConfig) -> Result<Self> {
        let storage = Self::create_storage(&config)?;
        
        Ok(Self {
            storage: Arc::new(storage),
            config,
            active_sessions: Arc::new(RwLock::new(HashMap::new())),
            metrics: Arc::new(SessionMetrics::default()),
        })
    }
    
    /// Create a new session
    pub async fn create_session(&self, options: CreateSessionOptions) -> Result<Arc<dyn Session>> {
        let id = Uuid::new_v4();
        let metadata = SessionMetadata {
            id,
            name: options.name,
            description: options.description,
            tags: options.tags,
            created_at: SystemTime::now(),
            updated_at: SystemTime::now(),
            status: SessionStatus::Active,
            parent_id: options.parent_id,
            user_id: options.user_id,
            security_context: options.security_context,
        };
        
        let session = Arc::new(SessionImpl {
            metadata,
            storage: self.storage.clone(),
            state: Arc::new(RwLock::new(HashMap::new())),
            events: Arc::new(RwLock::new(Vec::new())),
        });
        
        // Register session
        self.active_sessions.write().await.insert(id, session.clone());
        self.storage.save_session(&metadata).await?;
        self.metrics.sessions_created.fetch_add(1, Ordering::Relaxed);
        
        Ok(session)
    }
    
    /// Load an existing session
    pub async fn load_session(&self, id: Uuid) -> Result<Arc<dyn Session>> {
        // Check active sessions first
        if let Some(session) = self.active_sessions.read().await.get(&id) {
            return Ok(session.clone());
        }
        
        // Load from storage
        let metadata = self.storage.load_session_metadata(id).await?
            .ok_or_else(|| Error::SessionNotFound(id))?;
        
        let session = Arc::new(SessionImpl {
            metadata,
            storage: self.storage.clone(),
            state: Arc::new(RwLock::new(HashMap::new())),
            events: Arc::new(RwLock::new(Vec::new())),
        });
        
        // Load session state
        session.restore_state().await?;
        
        self.active_sessions.write().await.insert(id, session.clone());
        Ok(session)
    }
    
    /// List all sessions
    pub async fn list_sessions(&self, filter: Option<SessionFilter>) -> Result<Vec<SessionMetadata>> {
        self.storage.list_sessions(filter).await
    }
    
    /// Delete a session
    pub async fn delete_session(&self, id: Uuid) -> Result<()> {
        self.active_sessions.write().await.remove(&id);
        self.storage.delete_session(id).await?;
        self.metrics.sessions_deleted.fetch_add(1, Ordering::Relaxed);
        Ok(())
    }
}
```

### Artifact Storage

Store and manage session artifacts:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Artifact {
    pub id: ArtifactId,
    pub session_id: Uuid,
    pub artifact_type: ArtifactType,
    pub name: String,
    pub content: ArtifactContent,
    pub metadata: HashMap<String, Value>,
    pub created_at: SystemTime,
    pub size_bytes: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ArtifactType {
    Code,
    Data,
    Model,
    Document,
    Image,
    Audio,
    Video,
    Log,
    Config,
    Custom(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ArtifactContent {
    Text(String),
    Binary(Vec<u8>),
    Json(Value),
    Path(PathBuf),
    Url(String),
}

impl Artifact {
    /// Create a text artifact
    pub fn text(session_id: Uuid, name: String, content: String) -> Self {
        Self {
            id: ArtifactId(Uuid::new_v4()),
            session_id,
            artifact_type: ArtifactType::Document,
            name,
            size_bytes: content.len(),
            content: ArtifactContent::Text(content),
            metadata: HashMap::new(),
            created_at: SystemTime::now(),
        }
    }
    
    /// Create a JSON artifact
    pub fn json(session_id: Uuid, name: String, content: Value) -> Self {
        let size = serde_json::to_string(&content).unwrap_or_default().len();
        Self {
            id: ArtifactId(Uuid::new_v4()),
            session_id,
            artifact_type: ArtifactType::Data,
            name,
            size_bytes: size,
            content: ArtifactContent::Json(content),
            metadata: HashMap::new(),
            created_at: SystemTime::now(),
        }
    }
    
    /// Create a binary artifact
    pub fn binary(session_id: Uuid, name: String, content: Vec<u8>) -> Self {
        let size = content.len();
        Self {
            id: ArtifactId(Uuid::new_v4()),
            session_id,
            artifact_type: ArtifactType::Data,
            name,
            size_bytes: size,
            content: ArtifactContent::Binary(content),
            metadata: HashMap::new(),
            created_at: SystemTime::now(),
        }
    }
}
```

### Session Events

Track and replay session events:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionEvent {
    pub id: EventId,
    pub session_id: Uuid,
    pub timestamp: SystemTime,
    pub event_type: SessionEventType,
    pub data: Value,
    pub correlation_id: Option<Uuid>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SessionEventType {
    // Lifecycle events
    SessionCreated,
    SessionEnded,
    SessionPaused,
    SessionResumed,
    
    // Execution events
    AgentExecuted(AgentExecutionEvent),
    ToolInvoked(ToolInvocationEvent),
    WorkflowStarted(WorkflowEvent),
    WorkflowCompleted(WorkflowEvent),
    
    // State events
    StateChanged(StateChangeEvent),
    ArtifactStored(ArtifactMetadata),
    ArtifactRetrieved(ArtifactId),
    
    // Error events
    ErrorOccurred(ErrorEvent),
    WarningRaised(WarningEvent),
    
    // Custom events
    Custom(String, Value),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentExecutionEvent {
    pub agent_id: String,
    pub input: Value,
    pub output: Value,
    pub duration: Duration,
    pub tokens_used: Option<TokenUsage>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolInvocationEvent {
    pub tool_name: String,
    pub parameters: Value,
    pub result: Value,
    pub duration: Duration,
}
```

### Session Replay

Replay sessions for debugging and analysis:

```rust
pub struct SessionReplay {
    session: Arc<dyn Session>,
    events: Vec<SessionEvent>,
    position: usize,
    speed: f32,
}

impl SessionReplay {
    /// Create a replay from a session
    pub async fn from_session(session: Arc<dyn Session>) -> Result<Self> {
        let events = session.get_events(None).await?;
        
        Ok(Self {
            session,
            events,
            position: 0,
            speed: 1.0,
        })
    }
    
    /// Start replay
    pub async fn start(&mut self) -> Result<()> {
        self.position = 0;
        self.replay_from_position().await
    }
    
    /// Step forward one event
    pub async fn step(&mut self) -> Result<Option<SessionEvent>> {
        if self.position >= self.events.len() {
            return Ok(None);
        }
        
        let event = self.events[self.position].clone();
        self.position += 1;
        
        // Apply event effects
        self.apply_event(&event).await?;
        
        Ok(Some(event))
    }
    
    /// Jump to a specific time
    pub async fn seek(&mut self, timestamp: SystemTime) -> Result<()> {
        self.position = self.events.iter()
            .position(|e| e.timestamp >= timestamp)
            .unwrap_or(self.events.len());
        
        self.replay_from_position().await
    }
    
    /// Set replay speed
    pub fn set_speed(&mut self, speed: f32) {
        self.speed = speed.max(0.1).min(10.0);
    }
    
    /// Apply an event's effects
    async fn apply_event(&self, event: &SessionEvent) -> Result<()> {
        match &event.event_type {
            SessionEventType::StateChanged(change) => {
                self.session.update_state(&change.key, change.new_value.clone()).await?;
            }
            SessionEventType::ArtifactStored(metadata) => {
                // Artifact already stored, just log
                debug!("Replaying artifact storage: {}", metadata.name);
            }
            _ => {
                // Other events are informational
            }
        }
        
        Ok(())
    }
    
    /// Replay from current position
    async fn replay_from_position(&mut self) -> Result<()> {
        while self.position < self.events.len() {
            let event = &self.events[self.position];
            let delay = if self.position > 0 {
                let prev = &self.events[self.position - 1];
                event.timestamp.duration_since(prev.timestamp).unwrap_or_default()
            } else {
                Duration::from_secs(0)
            };
            
            // Apply speed multiplier
            let adjusted_delay = Duration::from_secs_f32(
                delay.as_secs_f32() / self.speed
            );
            
            tokio::time::sleep(adjusted_delay).await;
            self.step().await?;
        }
        
        Ok(())
    }
}
```

### Security Context

Isolate sessions with security contexts:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityContext {
    pub user_id: Option<String>,
    pub tenant_id: Option<String>,
    pub permissions: HashSet<Permission>,
    pub resource_limits: ResourceLimits,
    pub allowed_tools: Option<HashSet<String>>,
    pub denied_tools: Option<HashSet<String>>,
    pub network_policy: NetworkPolicy,
}

#[derive(Debug, Clone, Serialize, Deserialize, Hash, Eq, PartialEq)]
pub enum Permission {
    ReadState,
    WriteState,
    ExecuteAgent,
    InvokeTool,
    CreateArtifact,
    DeleteArtifact,
    ModifySession,
    Custom(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceLimits {
    pub max_memory_mb: usize,
    pub max_cpu_percent: f32,
    pub max_storage_mb: usize,
    pub max_execution_time: Duration,
    pub max_artifacts: usize,
    pub max_artifact_size_mb: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NetworkPolicy {
    AllowAll,
    DenyAll,
    AllowList(Vec<String>),
    DenyList(Vec<String>),
}

impl SecurityContext {
    /// Check if a permission is granted
    pub fn has_permission(&self, permission: &Permission) -> bool {
        self.permissions.contains(permission)
    }
    
    /// Check if a tool is allowed
    pub fn is_tool_allowed(&self, tool_name: &str) -> bool {
        if let Some(ref denied) = self.denied_tools {
            if denied.contains(tool_name) {
                return false;
            }
        }
        
        if let Some(ref allowed) = self.allowed_tools {
            return allowed.contains(tool_name);
        }
        
        true
    }
    
    /// Check if a network request is allowed
    pub fn is_network_allowed(&self, url: &str) -> bool {
        match &self.network_policy {
            NetworkPolicy::AllowAll => true,
            NetworkPolicy::DenyAll => false,
            NetworkPolicy::AllowList(list) => {
                list.iter().any(|pattern| url.contains(pattern))
            }
            NetworkPolicy::DenyList(list) => {
                !list.iter().any(|pattern| url.contains(pattern))
            }
        }
    }
}
```

### Session Analytics

Analyze session metrics and patterns:

```rust
pub struct SessionAnalytics {
    storage: Arc<dyn SessionStorage>,
}

impl SessionAnalytics {
    /// Get session statistics
    pub async fn get_statistics(&self, filter: Option<SessionFilter>) -> Result<SessionStatistics> {
        let sessions = self.storage.list_sessions(filter).await?;
        
        let total_sessions = sessions.len();
        let active_sessions = sessions.iter().filter(|s| s.status == SessionStatus::Active).count();
        let total_artifacts = self.count_artifacts(&sessions).await?;
        let total_events = self.count_events(&sessions).await?;
        
        let avg_duration = self.calculate_average_duration(&sessions);
        let avg_artifacts_per_session = total_artifacts as f64 / total_sessions.max(1) as f64;
        let avg_events_per_session = total_events as f64 / total_sessions.max(1) as f64;
        
        Ok(SessionStatistics {
            total_sessions,
            active_sessions,
            total_artifacts,
            total_events,
            avg_duration,
            avg_artifacts_per_session,
            avg_events_per_session,
            sessions_by_status: self.group_by_status(&sessions),
            sessions_by_tag: self.group_by_tags(&sessions),
        })
    }
    
    /// Analyze session patterns
    pub async fn analyze_patterns(&self, session_ids: Vec<Uuid>) -> Result<SessionPatterns> {
        let mut tool_usage = HashMap::new();
        let mut agent_usage = HashMap::new();
        let mut error_patterns = Vec::new();
        
        for session_id in session_ids {
            let events = self.storage.load_session_events(session_id).await?;
            
            for event in events {
                match event.event_type {
                    SessionEventType::ToolInvoked(ref tool_event) => {
                        *tool_usage.entry(tool_event.tool_name.clone()).or_insert(0) += 1;
                    }
                    SessionEventType::AgentExecuted(ref agent_event) => {
                        *agent_usage.entry(agent_event.agent_id.clone()).or_insert(0) += 1;
                    }
                    SessionEventType::ErrorOccurred(ref error) => {
                        error_patterns.push(error.clone());
                    }
                    _ => {}
                }
            }
        }
        
        Ok(SessionPatterns {
            tool_usage,
            agent_usage,
            error_patterns,
            common_workflows: self.identify_common_workflows(&session_ids).await?,
        })
    }
}
```

## Usage Examples

### Creating and Managing Sessions

```rust
use llmspell_sessions::{SessionManager, CreateSessionOptions, SecurityContext};

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize session manager
    let config = SessionConfig {
        storage_backend: StorageBackend::Sqlite("./sessions.db".into()),
        auto_save_interval: Duration::from_secs(30),
        retention_policy: RetentionPolicy::Days(30),
    };
    
    let manager = SessionManager::new(config)?;
    
    // Create a new session with security context
    let session = manager.create_session(CreateSessionOptions {
        name: Some("Data Analysis".to_string()),
        description: Some("Analyzing customer data".to_string()),
        tags: vec!["analysis".to_string(), "production".to_string()],
        security_context: Some(SecurityContext {
            user_id: Some("user123".to_string()),
            permissions: hashset![
                Permission::ReadState,
                Permission::WriteState,
                Permission::InvokeTool,
            ],
            resource_limits: ResourceLimits {
                max_memory_mb: 512,
                max_cpu_percent: 50.0,
                max_storage_mb: 100,
                max_execution_time: Duration::from_mins(10),
                max_artifacts: 100,
                max_artifact_size_mb: 10,
            },
            ..Default::default()
        }),
        ..Default::default()
    }).await?;
    
    println!("Created session: {}", session.id());
    
    Ok(())
}
```

### Storing and Retrieving Artifacts

```rust
async fn artifact_operations(session: Arc<dyn Session>) -> Result<()> {
    // Store text artifact
    let text_artifact = Artifact::text(
        *session.id(),
        "analysis_report.md".to_string(),
        "# Analysis Report\n\n## Summary\n...".to_string(),
    );
    let text_id = session.store_artifact(text_artifact).await?;
    
    // Store JSON data artifact
    let data_artifact = Artifact::json(
        *session.id(),
        "results.json".to_string(),
        json!({
            "total_customers": 1000,
            "average_value": 250.50,
            "top_products": ["A", "B", "C"]
        }),
    );
    let data_id = session.store_artifact(data_artifact).await?;
    
    // Store binary artifact (e.g., generated chart)
    let chart_bytes = generate_chart_image()?;
    let binary_artifact = Artifact::binary(
        *session.id(),
        "revenue_chart.png".to_string(),
        chart_bytes,
    );
    let binary_id = session.store_artifact(binary_artifact).await?;
    
    // List all artifacts
    let artifacts = session.list_artifacts().await?;
    for artifact in artifacts {
        println!("Artifact: {} ({:?}, {} bytes)", 
            artifact.name, artifact.artifact_type, artifact.size_bytes);
    }
    
    // Retrieve specific artifact
    if let Some(artifact) = session.get_artifact(&text_id).await? {
        match artifact.content {
            ArtifactContent::Text(content) => {
                println!("Report content: {}", content);
            }
            _ => {}
        }
    }
    
    Ok(())
}
```

### Recording Session Events

```rust
async fn record_execution(session: Arc<dyn Session>) -> Result<()> {
    // Record agent execution
    let agent_event = SessionEvent {
        id: EventId(Uuid::new_v4()),
        session_id: *session.id(),
        timestamp: SystemTime::now(),
        event_type: SessionEventType::AgentExecuted(AgentExecutionEvent {
            agent_id: "analyzer".to_string(),
            input: json!({"query": "analyze sales data"}),
            output: json!({"insights": ["trend up", "seasonal pattern"]}),
            duration: Duration::from_secs(2),
            tokens_used: Some(TokenUsage {
                prompt_tokens: 50,
                completion_tokens: 100,
                total_tokens: 150,
            }),
        }),
        correlation_id: None,
        data: json!({}),
    };
    session.record_event(agent_event).await?;
    
    // Record tool invocation
    let tool_event = SessionEvent {
        id: EventId(Uuid::new_v4()),
        session_id: *session.id(),
        timestamp: SystemTime::now(),
        event_type: SessionEventType::ToolInvoked(ToolInvocationEvent {
            tool_name: "data-processor".to_string(),
            parameters: json!({"operation": "aggregate", "field": "revenue"}),
            result: json!({"total": 1000000}),
            duration: Duration::from_millis(500),
        }),
        correlation_id: None,
        data: json!({}),
    };
    session.record_event(tool_event).await?;
    
    // Query events
    let events = session.get_events(Some(EventFilter {
        event_types: Some(vec![SessionEventType::AgentExecuted]),
        time_range: Some((SystemTime::now() - Duration::from_hours(1), SystemTime::now())),
        ..Default::default()
    })).await?;
    
    println!("Found {} agent execution events", events.len());
    
    Ok(())
}
```

### Session Replay

```rust
async fn replay_session(manager: &SessionManager, session_id: Uuid) -> Result<()> {
    // Load session
    let session = manager.load_session(session_id).await?;
    
    // Create replay
    let mut replay = SessionReplay::from_session(session).await?;
    
    // Set replay speed (2x)
    replay.set_speed(2.0);
    
    // Start replay
    println!("Starting replay...");
    replay.start().await?;
    
    // Or step through events
    while let Some(event) = replay.step().await? {
        println!("Event at {:?}: {:?}", event.timestamp, event.event_type);
        
        // Pause at interesting events
        if matches!(event.event_type, SessionEventType::ErrorOccurred(_)) {
            println!("Error detected, pausing...");
            tokio::time::sleep(Duration::from_secs(5)).await;
        }
    }
    
    Ok(())
}
```

### Session Analytics

```rust
async fn analyze_sessions(manager: &SessionManager) -> Result<()> {
    let analytics = SessionAnalytics::new(manager.storage());
    
    // Get overall statistics
    let stats = analytics.get_statistics(Some(SessionFilter {
        tags: Some(vec!["production".to_string()]),
        time_range: Some((SystemTime::now() - Duration::from_days(7), SystemTime::now())),
        ..Default::default()
    })).await?;
    
    println!("Session Statistics:");
    println!("  Total: {}", stats.total_sessions);
    println!("  Active: {}", stats.active_sessions);
    println!("  Avg Duration: {:?}", stats.avg_duration);
    println!("  Avg Artifacts: {:.1}", stats.avg_artifacts_per_session);
    
    // Analyze patterns
    let session_ids = manager.list_sessions(None).await?
        .iter()
        .map(|s| s.id)
        .collect();
    
    let patterns = analytics.analyze_patterns(session_ids).await?;
    
    println!("\nMost Used Tools:");
    for (tool, count) in patterns.tool_usage.iter().take(5) {
        println!("  {}: {} invocations", tool, count);
    }
    
    println!("\nError Patterns:");
    for error in patterns.error_patterns.iter().take(3) {
        println!("  {}: {}", error.code, error.message);
    }
    
    Ok(())
}
```

### Child Sessions

```rust
async fn create_child_session(manager: &SessionManager, parent_id: Uuid) -> Result<()> {
    // Create child session
    let child = manager.create_session(CreateSessionOptions {
        name: Some("Sub-task".to_string()),
        parent_id: Some(parent_id),
        // Child inherits security context from parent
        ..Default::default()
    }).await?;
    
    println!("Created child session: {} (parent: {})", child.id(), parent_id);
    
    // Access shared state from parent
    let parent = manager.load_session(parent_id).await?;
    if let Some(config) = parent.get_state("shared_config").await? {
        child.update_state("inherited_config", config).await?;
    }
    
    Ok(())
}
```

## Testing

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    
    #[tokio::test]
    async fn test_session_lifecycle() {
        let temp_dir = TempDir::new().unwrap();
        let config = SessionConfig {
            storage_backend: StorageBackend::Sqlite(
                temp_dir.path().join("test.db")
            ),
            ..Default::default()
        };
        
        let manager = SessionManager::new(config).unwrap();
        
        // Create session
        let session = manager.create_session(Default::default()).await.unwrap();
        let id = *session.id();
        
        // Store artifact
        let artifact = Artifact::text(id, "test.txt".to_string(), "content".to_string());
        let artifact_id = session.store_artifact(artifact).await.unwrap();
        
        // End session
        session.end().await.unwrap();
        
        // Reload session
        let loaded = manager.load_session(id).await.unwrap();
        assert_eq!(loaded.id(), &id);
        
        // Verify artifact persisted
        let artifact = loaded.get_artifact(&artifact_id).await.unwrap();
        assert!(artifact.is_some());
    }
}
```

## Performance Considerations

1. **Lazy Loading**: Events and artifacts are loaded on demand
2. **Batching**: Batch event recording for high-frequency operations
3. **Compression**: Large artifacts are compressed automatically
4. **Indexing**: Sessions indexed by tags and time for fast queries
5. **Cleanup**: Automatic cleanup based on retention policy

## Related Documentation

- [llmspell-state-persistence](llmspell-state-persistence.md) - State persistence backend
- [llmspell-security](llmspell-security.md) - Security contexts and permissions
- [llmspell-events](llmspell-events.md) - Event system integration