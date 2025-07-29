# Phase 6: Session and Artifact Management - Design Document

**Version**: 1.0  
**Date**: July 2025  
**Status**: DESIGN  
**Phase**: 6 (Session and Artifact Management)  
**Timeline**: Weeks 21-22  
**Priority**: MEDIUM (Production Enhancement)  
**Dependencies**: Phase 5 Persistent State Management ✅, Phase 4 Hook System ✅, Phase 3.3 Storage Infrastructure ✅  
**Crate Structure**: New `llmspell-sessions` crate

> **📋 Detailed Implementation Guide**: This document provides complete specifications for implementing Phase 6 session and artifact management for rs-llmspell, leveraging Phase 5's state persistence, Phase 4's hook system, and Phase 3.3's storage infrastructure.

---

## Phase Overview

### Goal
Implement comprehensive session management and artifact storage that enables users to save, restore, and replay their AI interaction sessions with full context preservation and artifact versioning.

### Core Principles
- **State-First Design**: Leverage Phase 5's StateManager for all persistence needs
- **Hook-Driven Lifecycle**: Use Phase 4's hooks for session boundaries and events  
- **Storage Abstraction**: Build on Phase 3.3's StorageBackend for artifact storage
- **Correlation-Based Tracking**: Link all session activities via event correlation IDs
- **Zero Additional Dependencies**: Use only existing infrastructure from previous phases
- **Performance Preservation**: Maintain <5ms operation latency from Phase 5

### Success Criteria
- [ ] Sessions can be created, saved, and restored with full context
- [ ] Artifacts are stored with proper metadata and versioning
- [ ] Session context preserved across application restarts
- [ ] Artifact versioning and history tracking works reliably
- [ ] Session replay functionality operational via ReplayableHook
- [ ] Session hooks fire at appropriate boundaries (start/end/suspend/resume)
- [ ] Artifacts are automatically collected via hooks
- [ ] Event correlation links all session activities

---

## 1. Implementation Specifications

### 1.1 Crate Architecture

**New Crate Structure:**
```toml
[package]
name = "llmspell-sessions"
version = "0.1.0"
edition = "2021"

[dependencies]
# Core dependencies from previous phases
llmspell-state-persistence = { path = "../llmspell-state-persistence" }
llmspell-storage = { path = "../llmspell-storage" }
llmspell-hooks = { path = "../llmspell-hooks" }
llmspell-events = { path = "../llmspell-events" }
llmspell-state-traits = { path = "../llmspell-state-traits" }

# Standard dependencies
async-trait = "0.1"
tokio = { version = "1.0", features = ["full"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
anyhow = "1.0"
thiserror = "1.0"
uuid = { version = "1.5", features = ["v4", "serde"] }
chrono = { version = "0.4", features = ["serde"] }
tracing = "0.1"
bincode = "1.3"
```

**Module Organization:**
```rust
llmspell-sessions/src/
├── lib.rs                 // Public API exports
├── error.rs               // Session-specific error types
├── types.rs               // Core types (Session, Artifact, etc.)
├── manager.rs             // SessionManager implementation
├── lifecycle.rs           // Session lifecycle management
├── storage/
│   ├── mod.rs            // Storage module exports
│   ├── artifact.rs       // Artifact storage implementation
│   ├── metadata.rs       // Metadata management
│   └── versioning.rs     // Version control system
├── replay/
│   ├── mod.rs            // Replay module exports
│   ├── replayer.rs       // Session replay engine
│   └── timeline.rs       // Timeline reconstruction
├── hooks/
│   ├── mod.rs            // Hook module exports
│   ├── lifecycle.rs      // Lifecycle hook definitions
│   └── collectors.rs     // Artifact collection hooks
└── bridge/
    ├── mod.rs            // Script bridge exports
    └── lua.rs            // Lua Session global
```

### 1.2 Core Types and Structures

```rust
use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize};
use uuid::Uuid;

/// Session configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionConfig {
    pub name: Option<String>,
    pub description: Option<String>,
    pub auto_save_interval: Option<Duration>,
    pub max_artifacts: Option<usize>,
    pub retention_policy: RetentionPolicy,
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Main session structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    pub id: SessionId,
    pub config: SessionConfig,
    pub state: SessionState,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub correlation_id: Uuid,
    pub parent_session: Option<SessionId>,
}

/// Session state enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SessionState {
    Active,
    Suspended,
    Completed,
    Failed,
    Archived,
}

/// Session artifact
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionArtifact {
    pub id: ArtifactId,
    pub session_id: SessionId,
    pub artifact_type: ArtifactType,
    pub name: String,
    pub content_hash: String, // SHA256 of content
    pub metadata: ArtifactMetadata,
    pub created_at: DateTime<Utc>,
    pub version: u32,
}

/// Artifact types
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ArtifactType {
    AgentOutput,
    ToolResult,
    UserInput,
    SystemLog,
    StateSnapshot,
    Custom(String),
}

/// Artifact metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArtifactMetadata {
    pub size_bytes: usize,
    pub mime_type: Option<String>,
    pub encoding: Option<String>,
    pub tags: Vec<String>,
    pub source_agent: Option<String>,
    pub source_tool: Option<String>,
    pub correlation_id: Uuid,
    pub custom: HashMap<String, serde_json::Value>,
}

/// Session replay event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionEvent {
    pub timestamp: DateTime<Utc>,
    pub event_type: SessionEventType,
    pub correlation_id: Uuid,
    pub data: serde_json::Value,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SessionEventType {
    Started,
    Suspended,
    Resumed,
    Completed,
    Failed,
    ArtifactCreated,
    ArtifactUpdated,
    StateChanged,
    HookExecuted,
    Custom(String),
}
```

### 1.3 SessionManager Implementation

```rust
use std::sync::Arc;
use tokio::sync::RwLock;
use llmspell_state_persistence::{StateManager, StateScope};
use llmspell_storage::StorageBackend;
use llmspell_hooks::{HookExecutor, HookContext, HookPoint};
use llmspell_events::{EventBus, UniversalEvent, EventCorrelationTracker};

pub struct SessionManager {
    // Core infrastructure from previous phases
    state_manager: Arc<StateManager>,
    storage_backend: Arc<dyn StorageBackend>,
    hook_executor: Arc<HookExecutor>,
    event_bus: Arc<EventBus>,
    correlation_tracker: Arc<EventCorrelationTracker>,
    
    // Session-specific state
    active_sessions: Arc<RwLock<HashMap<SessionId, Session>>>,
    artifact_storage: Arc<ArtifactStorage>,
    replay_engine: Arc<ReplayEngine>,
    
    // Configuration
    config: SessionManagerConfig,
}

impl SessionManager {
    /// Create a new session manager
    pub fn new(
        state_manager: Arc<StateManager>,
        storage_backend: Arc<dyn StorageBackend>,
        hook_executor: Arc<HookExecutor>,
        event_bus: Arc<EventBus>,
        config: SessionManagerConfig,
    ) -> Result<Self> {
        let correlation_tracker = Arc::new(EventCorrelationTracker::new());
        let artifact_storage = Arc::new(ArtifactStorage::new(
            storage_backend.clone(),
            config.artifact_config.clone(),
        )?);
        let replay_engine = Arc::new(ReplayEngine::new(
            state_manager.clone(),
            hook_executor.clone(),
        ));
        
        Ok(Self {
            state_manager,
            storage_backend,
            hook_executor,
            event_bus,
            correlation_tracker,
            active_sessions: Arc::new(RwLock::new(HashMap::new())),
            artifact_storage,
            replay_engine,
            config,
        })
    }
    
    /// Create a new session
    pub async fn create_session(&self, config: SessionConfig) -> Result<Session> {
        let session_id = SessionId::new();
        let correlation_id = self.correlation_tracker.create_correlation_id();
        
        // Create session object
        let session = Session {
            id: session_id.clone(),
            config: config.clone(),
            state: SessionState::Active,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            correlation_id,
            parent_session: None,
        };
        
        // Fire session:start hook
        let mut hook_context = HookContext::new();
        hook_context.insert_metadata("session_id", session_id.to_string());
        hook_context.insert_metadata("correlation_id", correlation_id.to_string());
        hook_context.insert_metadata("config", serde_json::to_value(&config)?);
        
        self.hook_executor.execute_hooks(
            HookPoint::Custom("session:start"),
            hook_context.clone(),
        ).await?;
        
        // Initialize session state
        let scope = StateScope::Session(session_id.to_string());
        self.state_manager.set_with_hooks(
            scope.clone(),
            "metadata",
            serde_json::to_value(&session)?,
        ).await?;
        
        // Store in active sessions
        {
            let mut sessions = self.active_sessions.write().await;
            sessions.insert(session_id.clone(), session.clone());
        }
        
        // Emit session started event
        let event = UniversalEvent::new("session.started")
            .with_correlation_id(correlation_id)
            .with_data(serde_json::to_value(&session)?);
        self.event_bus.publish(event).await?;
        
        Ok(session)
    }
    
    /// Save session state
    pub async fn save_session(&self, session_id: &SessionId) -> Result<()> {
        let session = self.get_session(session_id).await?;
        let scope = StateScope::Session(session_id.to_string());
        
        // Update timestamp
        let mut session = session;
        session.updated_at = Utc::now();
        
        // Save session metadata
        self.state_manager.set_with_hooks(
            scope.clone(),
            "metadata",
            serde_json::to_value(&session)?,
        ).await?;
        
        // Collect all session artifacts
        let artifacts = self.artifact_storage.list_artifacts(session_id).await?;
        self.state_manager.set_with_hooks(
            scope.clone(),
            "artifacts",
            serde_json::to_value(&artifacts)?,
        ).await?;
        
        // Fire session:save hook
        let mut hook_context = HookContext::new();
        hook_context.insert_metadata("session_id", session_id.to_string());
        hook_context.insert_metadata("artifact_count", artifacts.len().to_string());
        
        self.hook_executor.execute_hooks(
            HookPoint::Custom("session:save"),
            hook_context,
        ).await?;
        
        Ok(())
    }
    
    /// Restore session from storage
    pub async fn restore_session(&self, session_id: &SessionId) -> Result<Session> {
        let scope = StateScope::Session(session_id.to_string());
        
        // Load session metadata
        let metadata_value = self.state_manager.get(scope.clone(), "metadata").await?
            .ok_or_else(|| anyhow::anyhow!("Session not found: {}", session_id))?;
        
        let session: Session = serde_json::from_value(metadata_value)?;
        
        // Fire session:restore hook
        let mut hook_context = HookContext::new();
        hook_context.insert_metadata("session_id", session_id.to_string());
        hook_context.insert_metadata("state", session.state.to_string());
        
        self.hook_executor.execute_hooks(
            HookPoint::Custom("session:restore"),
            hook_context,
        ).await?;
        
        // Add to active sessions if not archived
        if session.state != SessionState::Archived {
            let mut sessions = self.active_sessions.write().await;
            sessions.insert(session_id.clone(), session.clone());
        }
        
        Ok(session)
    }
    
    /// Complete a session
    pub async fn complete_session(&self, session_id: &SessionId) -> Result<()> {
        let mut session = self.get_session(session_id).await?;
        session.state = SessionState::Completed;
        session.updated_at = Utc::now();
        
        // Save final state
        self.save_session(session_id).await?;
        
        // Fire session:end hook
        let mut hook_context = HookContext::new();
        hook_context.insert_metadata("session_id", session_id.to_string());
        hook_context.insert_metadata("correlation_id", session.correlation_id.to_string());
        
        self.hook_executor.execute_hooks(
            HookPoint::Custom("session:end"),
            hook_context,
        ).await?;
        
        // Remove from active sessions
        {
            let mut sessions = self.active_sessions.write().await;
            sessions.remove(session_id);
        }
        
        Ok(())
    }
}
```

### 1.4 Artifact Storage System

```rust
use llmspell_storage::{StorageBackend, StorageSerialize};
use sha2::{Sha256, Digest};

pub struct ArtifactStorage {
    storage_backend: Arc<dyn StorageBackend>,
    config: ArtifactConfig,
    version_manager: VersionManager,
}

impl ArtifactStorage {
    /// Store a new artifact
    pub async fn store_artifact(
        &self,
        session_id: &SessionId,
        artifact_type: ArtifactType,
        name: String,
        content: Vec<u8>,
        metadata: ArtifactMetadata,
    ) -> Result<SessionArtifact> {
        let artifact_id = ArtifactId::new();
        
        // Calculate content hash
        let mut hasher = Sha256::new();
        hasher.update(&content);
        let content_hash = format!("{:x}", hasher.finalize());
        
        // Determine version
        let version = self.version_manager.next_version(session_id, &name).await?;
        
        // Create artifact object
        let artifact = SessionArtifact {
            id: artifact_id.clone(),
            session_id: session_id.clone(),
            artifact_type,
            name,
            content_hash,
            metadata,
            created_at: Utc::now(),
            version,
        };
        
        // Store content
        let content_key = format!("artifact_content:{}:{}", session_id, artifact_id);
        self.storage_backend.store(&content_key, &content).await?;
        
        // Store metadata
        let metadata_key = format!("artifact_metadata:{}:{}", session_id, artifact_id);
        self.storage_backend.store(&metadata_key, &artifact).await?;
        
        // Update version index
        self.version_manager.record_version(
            session_id,
            &artifact.name,
            version,
            artifact_id.clone(),
        ).await?;
        
        Ok(artifact)
    }
    
    /// Retrieve an artifact
    pub async fn get_artifact(
        &self,
        session_id: &SessionId,
        artifact_id: &ArtifactId,
    ) -> Result<(SessionArtifact, Vec<u8>)> {
        // Load metadata
        let metadata_key = format!("artifact_metadata:{}:{}", session_id, artifact_id);
        let artifact: SessionArtifact = self.storage_backend
            .load(&metadata_key)
            .await?;
        
        // Load content
        let content_key = format!("artifact_content:{}:{}", session_id, artifact_id);
        let content: Vec<u8> = self.storage_backend
            .load(&content_key)
            .await?;
        
        // Verify content hash
        let mut hasher = Sha256::new();
        hasher.update(&content);
        let actual_hash = format!("{:x}", hasher.finalize());
        
        if actual_hash != artifact.content_hash {
            return Err(anyhow::anyhow!(
                "Artifact content hash mismatch: expected {}, got {}",
                artifact.content_hash,
                actual_hash
            ));
        }
        
        Ok((artifact, content))
    }
    
    /// List all artifacts for a session
    pub async fn list_artifacts(&self, session_id: &SessionId) -> Result<Vec<SessionArtifact>> {
        let prefix = format!("artifact_metadata:{}:", session_id);
        let keys = self.storage_backend.list_keys(&prefix).await?;
        
        let mut artifacts = Vec::new();
        for key in keys {
            let artifact: SessionArtifact = self.storage_backend.load(&key).await?;
            artifacts.push(artifact);
        }
        
        // Sort by creation time
        artifacts.sort_by_key(|a| a.created_at);
        
        Ok(artifacts)
    }
}

impl StorageSerialize for SessionArtifact {
    fn serialize_for_storage(&self) -> Result<Vec<u8>, llmspell_storage::StorageError> {
        bincode::serialize(self)
            .map_err(|e| llmspell_storage::StorageError::SerializationFailed(e.to_string()))
    }
    
    fn storage_key(&self) -> String {
        format!("artifact:{}:{}", self.session_id, self.id)
    }
    
    fn storage_namespace(&self) -> String {
        "sessions".to_string()
    }
}
```

### 1.5 Session Replay Engine

```rust
use llmspell_hooks::{ReplayableHook, HookHistory};

pub struct ReplayEngine {
    state_manager: Arc<StateManager>,
    hook_executor: Arc<HookExecutor>,
}

impl ReplayEngine {
    /// Replay a session from its history
    pub async fn replay_session(
        &self,
        session_id: &SessionId,
        target_state: Option<DateTime<Utc>>,
    ) -> Result<ReplayResult> {
        let scope = StateScope::Session(session_id.to_string());
        
        // Load session metadata
        let session = self.load_session_metadata(session_id).await?;
        
        // Load hook history
        let hook_history = self.load_hook_history(&session.correlation_id).await?;
        
        // Create new replay session
        let replay_session_id = SessionId::new();
        let replay_correlation_id = Uuid::new_v4();
        
        // Initialize replay result
        let mut replay_result = ReplayResult {
            original_session_id: session_id.clone(),
            replay_session_id: replay_session_id.clone(),
            events_replayed: 0,
            errors: Vec::new(),
        };
        
        // Replay hooks in order
        for hook_execution in hook_history {
            // Check if we've reached target state
            if let Some(target) = target_state {
                if hook_execution.timestamp > target {
                    break;
                }
            }
            
            // Replay the hook
            match self.replay_hook_execution(&hook_execution, replay_correlation_id).await {
                Ok(_) => replay_result.events_replayed += 1,
                Err(e) => replay_result.errors.push(ReplayError {
                    timestamp: hook_execution.timestamp,
                    hook_id: hook_execution.hook_id,
                    error: e.to_string(),
                }),
            }
        }
        
        Ok(replay_result)
    }
    
    /// Reconstruct session timeline
    pub async fn reconstruct_timeline(
        &self,
        session_id: &SessionId,
    ) -> Result<SessionTimeline> {
        let session = self.load_session_metadata(session_id).await?;
        
        // Collect all events
        let mut events = Vec::new();
        
        // Load state changes
        let state_changes = self.load_state_changes(&session.correlation_id).await?;
        for change in state_changes {
            events.push(TimelineEvent {
                timestamp: change.timestamp,
                event_type: TimelineEventType::StateChange(change),
                correlation_id: session.correlation_id,
            });
        }
        
        // Load artifact creations
        let artifacts = self.load_artifact_events(session_id).await?;
        for artifact in artifacts {
            events.push(TimelineEvent {
                timestamp: artifact.created_at,
                event_type: TimelineEventType::ArtifactCreated(artifact),
                correlation_id: session.correlation_id,
            });
        }
        
        // Sort by timestamp
        events.sort_by_key(|e| e.timestamp);
        
        Ok(SessionTimeline {
            session_id: session_id.clone(),
            start_time: session.created_at,
            end_time: if session.state == SessionState::Completed {
                Some(session.updated_at)
            } else {
                None
            },
            events,
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplayResult {
    pub original_session_id: SessionId,
    pub replay_session_id: SessionId,
    pub events_replayed: usize,
    pub errors: Vec<ReplayError>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionTimeline {
    pub session_id: SessionId,
    pub start_time: DateTime<Utc>,
    pub end_time: Option<DateTime<Utc>>,
    pub events: Vec<TimelineEvent>,
}
```

---

## 2. Hook Integration

### 2.1 Session Lifecycle Hooks

```rust
use llmspell_hooks::{Hook, HookContext, HookResult, HookPoint};

/// Built-in session lifecycle hooks
pub fn register_session_hooks(hook_executor: &mut HookExecutor) -> Result<()> {
    // Session start hook
    hook_executor.register_hook(
        HookPoint::Custom("session:start"),
        SessionStartHook::new(),
    )?;
    
    // Session end hook
    hook_executor.register_hook(
        HookPoint::Custom("session:end"),
        SessionEndHook::new(),
    )?;
    
    // Artifact collection hook
    hook_executor.register_hook(
        HookPoint::Custom("artifact:collect"),
        ArtifactCollectionHook::new(),
    )?;
    
    Ok(())
}

#[derive(Debug)]
struct SessionStartHook {
    start_time: Instant,
}

#[async_trait]
impl Hook for SessionStartHook {
    fn id(&self) -> &str {
        "builtin:session:start"
    }
    
    async fn execute(&mut self, context: HookContext) -> Result<HookResult> {
        self.start_time = Instant::now();
        
        // Log session start
        tracing::info!(
            session_id = context.get_metadata("session_id").unwrap_or("unknown"),
            "Session started"
        );
        
        Ok(HookResult::Continue)
    }
}

#[derive(Debug)]
struct ArtifactCollectionHook {
    artifact_storage: Arc<ArtifactStorage>,
}

#[async_trait]
impl Hook for ArtifactCollectionHook {
    fn id(&self) -> &str {
        "builtin:artifact:collect"
    }
    
    async fn execute(&mut self, context: HookContext) -> Result<HookResult> {
        // Extract artifact data from context
        if let Some(artifact_data) = context.get_metadata("artifact_data") {
            let session_id = context.get_metadata("session_id")
                .ok_or_else(|| anyhow::anyhow!("Missing session_id"))?;
            
            // Store artifact
            let artifact = self.artifact_storage.store_artifact(
                &SessionId::from_str(&session_id)?,
                ArtifactType::AgentOutput,
                "auto_collected".to_string(),
                artifact_data.as_bytes().to_vec(),
                ArtifactMetadata::default(),
            ).await?;
            
            tracing::debug!("Auto-collected artifact: {}", artifact.id);
        }
        
        Ok(HookResult::Continue)
    }
}
```

### 2.2 Automatic Artifact Collection

```rust
/// Hook integration for automatic artifact collection
impl SessionManager {
    /// Register artifact collection hooks for agents and tools
    pub async fn register_artifact_collectors(&self) -> Result<()> {
        // Agent output collector
        self.hook_executor.register_hook(
            HookPoint::AfterAgentExecution,
            AgentOutputCollector::new(self.artifact_storage.clone()),
        ).await?;
        
        // Tool result collector
        self.hook_executor.register_hook(
            HookPoint::AfterToolExecution,
            ToolResultCollector::new(self.artifact_storage.clone()),
        ).await?;
        
        Ok(())
    }
}

struct AgentOutputCollector {
    artifact_storage: Arc<ArtifactStorage>,
}

#[async_trait]
impl Hook for AgentOutputCollector {
    fn id(&self) -> &str {
        "builtin:agent:output:collector"
    }
    
    async fn execute(&mut self, context: HookContext) -> Result<HookResult> {
        // Check if we're in a session context
        if let Some(session_id) = context.get_metadata("session_id") {
            // Extract agent output
            if let Some(output) = context.get_result() {
                let metadata = ArtifactMetadata {
                    source_agent: context.get_metadata("agent_id"),
                    correlation_id: context.correlation_id(),
                    ..Default::default()
                };
                
                // Store as artifact
                self.artifact_storage.store_artifact(
                    &SessionId::from_str(&session_id)?,
                    ArtifactType::AgentOutput,
                    format!("agent_output_{}", Utc::now().timestamp()),
                    serde_json::to_vec(&output)?,
                    metadata,
                ).await?;
            }
        }
        
        Ok(HookResult::Continue)
    }
}
```

---

## 3. Script Integration

### 3.1 Lua Session API

```lua
-- Session global API
Session = {
    -- Session management
    create = function(config) end,
    save = function(session_id) end,
    restore = function(session_id) end,
    complete = function(session_id) end,
    list = function() end,
    
    -- Artifact operations
    saveArtifact = function(session_id, name, content, metadata) end,
    loadArtifact = function(session_id, artifact_id) end,
    listArtifacts = function(session_id) end,
    
    -- Session context
    getCurrent = function() end,
    setCurrent = function(session_id) end,
    
    -- Replay functionality
    replay = function(session_id, target_time) end,
    timeline = function(session_id) end,
}

-- Example usage
local config = {
    name = "Research Session",
    description = "Analyzing AI trends",
    retention_policy = {
        type = "days",
        value = 30
    }
}

-- Create new session
local session = Session.create(config)
print("Created session:", session.id)

-- Set as current session
Session.setCurrent(session.id)

-- Do some work...
local agent = Agent.create("researcher")
local result = agent:execute("Find recent AI breakthroughs")

-- Save an artifact
Session.saveArtifact(session.id, "research_results.json", result, {
    tags = {"research", "ai", "breakthroughs"},
    source_agent = agent.id
})

-- Save session state
Session.save(session.id)

-- Later, restore the session
local restored = Session.restore(session.id)
print("Restored session from:", restored.created_at)

-- List artifacts
local artifacts = Session.listArtifacts(session.id)
for _, artifact in ipairs(artifacts) do
    print("Artifact:", artifact.name, "Version:", artifact.version)
end

-- Complete the session
Session.complete(session.id)
```

### 3.2 Bridge Implementation

```rust
use mlua::{Lua, Result as LuaResult, Table, Value};
use llmspell_bridge::lua::helpers::*;

pub fn register_session_global(lua: &Lua, session_manager: Arc<SessionManager>) -> LuaResult<()> {
    let session_table = lua.create_table()?;
    
    // Create session
    let manager = session_manager.clone();
    session_table.set("create", lua.create_async_function(move |lua, config: Value| {
        let manager = manager.clone();
        async move {
            let config = lua_value_to_session_config(&config)?;
            let session = manager.create_session(config).await
                .map_err(|e| mlua::Error::external(e))?;
            
            session_to_lua_value(lua, &session)
        }
    })?)?;
    
    // Save session
    let manager = session_manager.clone();
    session_table.set("save", lua.create_async_function(move |_lua, session_id: String| {
        let manager = manager.clone();
        async move {
            let session_id = SessionId::from_str(&session_id)
                .map_err(|e| mlua::Error::external(e))?;
            
            manager.save_session(&session_id).await
                .map_err(|e| mlua::Error::external(e))?;
            
            Ok(())
        }
    })?)?;
    
    // Save artifact
    let manager = session_manager.clone();
    session_table.set("saveArtifact", lua.create_async_function(move |lua, args: (String, String, Value, Table)| {
        let manager = manager.clone();
        async move {
            let (session_id, name, content, metadata) = args;
            
            let session_id = SessionId::from_str(&session_id)
                .map_err(|e| mlua::Error::external(e))?;
            
            let content_bytes = match content {
                Value::String(s) => s.as_bytes().to_vec(),
                _ => serde_json::to_vec(&lua_value_to_json(&content)?)
                    .map_err(|e| mlua::Error::external(e))?,
            };
            
            let metadata = lua_table_to_artifact_metadata(&metadata)?;
            
            let artifact = manager.artifact_storage.store_artifact(
                &session_id,
                ArtifactType::UserInput,
                name,
                content_bytes,
                metadata,
            ).await.map_err(|e| mlua::Error::external(e))?;
            
            artifact_to_lua_value(lua, &artifact)
        }
    })?)?;
    
    // Register global
    lua.globals().set("Session", session_table)?;
    
    Ok(())
}
```

---

## 4. Testing Strategy

### 4.1 Test Categories

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[cfg_attr(test_category = "session")]
    #[tokio::test]
    async fn test_session_lifecycle() {
        let manager = create_test_session_manager().await;
        
        // Create session
        let config = SessionConfig::default();
        let session = manager.create_session(config).await.unwrap();
        assert_eq!(session.state, SessionState::Active);
        
        // Save session
        manager.save_session(&session.id).await.unwrap();
        
        // Complete session
        manager.complete_session(&session.id).await.unwrap();
        
        // Verify state change
        let completed = manager.restore_session(&session.id).await.unwrap();
        assert_eq!(completed.state, SessionState::Completed);
    }
    
    #[cfg_attr(test_category = "session")]
    #[tokio::test]
    async fn test_artifact_storage_and_retrieval() {
        let storage = create_test_artifact_storage().await;
        let session_id = SessionId::new();
        
        // Store artifact
        let content = b"Test artifact content";
        let metadata = ArtifactMetadata {
            size_bytes: content.len(),
            tags: vec!["test".to_string()],
            ..Default::default()
        };
        
        let artifact = storage.store_artifact(
            &session_id,
            ArtifactType::UserInput,
            "test.txt".to_string(),
            content.to_vec(),
            metadata,
        ).await.unwrap();
        
        // Retrieve artifact
        let (retrieved_artifact, retrieved_content) = storage
            .get_artifact(&session_id, &artifact.id)
            .await.unwrap();
        
        assert_eq!(retrieved_artifact.id, artifact.id);
        assert_eq!(retrieved_content, content);
    }
    
    #[cfg_attr(test_category = "session")]
    #[tokio::test]
    async fn test_session_replay() {
        let manager = create_test_session_manager().await;
        
        // Create and populate session
        let session = manager.create_session(SessionConfig::default()).await.unwrap();
        
        // Perform some operations...
        
        // Complete session
        manager.complete_session(&session.id).await.unwrap();
        
        // Replay session
        let replay_result = manager.replay_engine
            .replay_session(&session.id, None)
            .await.unwrap();
        
        assert!(replay_result.errors.is_empty());
        assert!(replay_result.events_replayed > 0);
    }
    
    #[cfg_attr(test_category = "session")]
    #[tokio::test]
    async fn test_hook_integration() {
        let manager = create_test_session_manager().await;
        let hook_called = Arc::new(AtomicBool::new(false));
        
        // Register test hook
        let called = hook_called.clone();
        manager.hook_executor.register_hook(
            HookPoint::Custom("session:start"),
            TestHook::new(called),
        ).await.unwrap();
        
        // Create session (should trigger hook)
        let _session = manager.create_session(SessionConfig::default()).await.unwrap();
        
        // Verify hook was called
        assert!(hook_called.load(Ordering::Relaxed));
    }
}
```

### 4.2 Performance Tests

```rust
#[cfg(test)]
mod performance_tests {
    use criterion::{criterion_group, criterion_main, Criterion};
    
    fn bench_session_operations(c: &mut Criterion) {
        let rt = tokio::runtime::Runtime::new().unwrap();
        let manager = rt.block_on(create_test_session_manager());
        
        let mut group = c.benchmark_group("session_operations");
        
        // Benchmark session creation
        group.bench_function("create_session", |b| {
            b.to_async(&rt).iter(|| async {
                manager.create_session(SessionConfig::default()).await.unwrap()
            });
        });
        
        // Benchmark artifact storage
        let session_id = rt.block_on(async {
            manager.create_session(SessionConfig::default()).await.unwrap().id
        });
        
        group.bench_function("store_artifact", |b| {
            b.to_async(&rt).iter(|| async {
                manager.artifact_storage.store_artifact(
                    &session_id,
                    ArtifactType::UserInput,
                    "bench.txt".to_string(),
                    vec![0u8; 1024], // 1KB artifact
                    ArtifactMetadata::default(),
                ).await.unwrap()
            });
        });
        
        group.finish();
    }
    
    criterion_group!(benches, bench_session_operations);
    criterion_main!(benches);
}
```

---

## 5. Implementation Timeline

### Phase 6.1: Core Session Management (Days 1-3)
- Implement SessionManager with basic CRUD operations
- Session lifecycle management with state transitions
- Integration with StateManager for persistence
- Basic hook integration for lifecycle events

### Phase 6.2: Artifact Storage System (Days 4-6)
- Implement ArtifactStorage with versioning
- Content hashing and integrity verification
- Metadata management and search capabilities
- Storage backend integration

### Phase 6.3: Hook Integration (Days 7-8)
- Implement session lifecycle hooks
- Automatic artifact collection hooks
- Hook registration and configuration
- Event correlation setup

### Phase 6.4: Session Replay (Days 9-10)
- Implement ReplayEngine
- Timeline reconstruction from events
- Hook replay functionality
- Progress tracking and error handling

### Phase 6.5: Script Integration (Days 11-12)
- Implement Lua Session global
- Bridge implementation for all operations
- JavaScript API preparation
- Example scripts and documentation

### Phase 6.6: Testing and Validation (Days 13-14)
- Comprehensive test suite
- Performance benchmarking
- Integration testing
- Documentation completion

---

## 6. Performance Targets

Based on Phase 5 baselines:

| Operation | Target | Rationale |
|-----------|--------|-----------|
| Session Creation | <10ms | StateManager overhead + metadata |
| Session Save | <20ms | Multiple state operations |
| Artifact Store | <15ms | Hashing + storage write |
| Artifact Retrieve | <10ms | Direct storage read |
| Session Restore | <25ms | Multiple state reads |
| Hook Overhead | <2% | Maintained from Phase 5 |
| Timeline Reconstruction | <100ms | For 1000 events |
| Session Replay | <500ms | For typical session |

---

## 7. Security Considerations

### 7.1 Session Isolation
- Sessions cannot access each other's state
- Artifact access restricted by session ownership
- Correlation IDs prevent cross-session data leakage

### 7.2 Data Protection
- Content hashing for integrity verification
- Optional encryption for sensitive artifacts
- Audit trail via hook system

### 7.3 Resource Limits
- Maximum artifacts per session
- Storage quota enforcement
- Retention policy enforcement

---

## 8. Future Considerations

### Phase 7 Preparation (Vector Storage)
- Design artifacts to support embedding generation
- Consider semantic search over session artifacts
- Prepare for RAG over session history

### Phase 8 Preparation (Advanced Workflows)
- Session-aware workflow execution
- Workflow state tied to sessions
- Cross-session workflow patterns

---

This design document provides a comprehensive blueprint for implementing Phase 6's session and artifact management system, building seamlessly on the robust foundation established in Phases 3-5.