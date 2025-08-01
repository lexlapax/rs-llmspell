# Session and Artifact Implementation Guide

This guide covers the implementation details of the Session and Artifact management system for developers working on or extending llmspell.

## Architecture Overview

The Session and Artifact system follows a three-layer architecture:

```
┌─────────────────────────────────────────────────────────┐
│                    Script Layer (Lua)                    │
│         Session.create(), Artifact.store(), etc.         │
└──────────────────────────┬──────────────────────────────┘
                           │
┌─────────────────────────────────────────────────────────┐
│                GlobalObject Layer (Sync)                 │
│      SessionGlobal, ArtifactGlobal (thread-safe)        │
└──────────────────────────┬──────────────────────────────┘
                           │
┌─────────────────────────────────────────────────────────┐
│                  Bridge Layer (Async)                    │
│      SessionBridge, ArtifactBridge (async ops)          │
└──────────────────────────┬──────────────────────────────┘
                           │
┌─────────────────────────────────────────────────────────┐
│                    Core Layer (Rust)                     │
│   SessionManager, SessionArtifact, ArtifactStorage       │
└─────────────────────────────────────────────────────────┘
```

## Core Components

### SessionManager

Located in `llmspell-sessions/src/manager.rs`, the SessionManager is the central coordinator:

```rust
pub struct SessionManager {
    // Active sessions in memory
    active_sessions: Arc<RwLock<HashMap<SessionId, Session>>>,
    
    // Persistent storage backend
    storage_backend: Arc<dyn StorageBackend>,
    
    // State management
    state_manager: Arc<StateManager>,
    
    // Hook system integration
    hook_registry: Arc<HookRegistry>,
    hook_executor: Arc<HookExecutor>,
    
    // Event system
    event_bus: Arc<EventBus>,
    
    // Configuration
    config: SessionManagerConfig,
}
```

Key responsibilities:
- Session lifecycle management (create, suspend, resume, complete)
- Artifact storage and retrieval
- Integration with hooks and events
- Persistence to storage backend

### Session Types

```rust
pub struct Session {
    pub id: SessionId,
    pub metadata: Arc<RwLock<SessionMetadata>>,
    pub artifacts: Arc<RwLock<Vec<ArtifactId>>>,
    pub state: Arc<RwLock<SessionState>>,
}

pub struct SessionMetadata {
    pub id: SessionId,
    pub name: String,
    pub description: Option<String>,
    pub tags: Vec<String>,
    pub status: SessionStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub parent_session_id: Option<SessionId>,
    pub custom_metadata: HashMap<String, serde_json::Value>,
}

pub enum SessionStatus {
    Active,
    Suspended,
    Completed,
    Failed,
}
```

### Artifact System

Artifacts use content-addressed storage with automatic compression:

```rust
pub struct SessionArtifact {
    pub id: ArtifactId,
    pub metadata: ArtifactMetadata,
    content: Vec<u8>,  // May be compressed
}

pub struct ArtifactId {
    pub content_hash: String,  // BLAKE3 hash
    pub session_id: SessionId,
    pub sequence: u64,
}

pub struct ArtifactMetadata {
    pub name: String,
    pub artifact_type: ArtifactType,
    pub mime_type: String,
    pub size: usize,
    pub is_compressed: bool,
    pub tags: Vec<String>,
    pub custom: HashMap<String, serde_json::Value>,
    // ... other fields
}
```

## Bridge Layer Implementation

The bridge layer provides async operations that are converted to sync for script access:

### SessionBridge

```rust
impl SessionBridge {
    pub async fn create_session(
        &self, 
        options: CreateSessionOptions
    ) -> Result<SessionId> {
        self.session_manager.create_session(options).await
    }
    
    pub async fn get_session(&self, id: &SessionId) -> Result<Session> {
        self.session_manager.get_session(id).await
    }
    
    // Thread-local current session
    pub fn get_current_session() -> Option<SessionId> {
        CURRENT_SESSION.with(|cell| cell.borrow().clone())
    }
}
```

### ArtifactBridge

```rust
impl ArtifactBridge {
    pub async fn store_artifact(
        &self,
        session_id: &SessionId,
        artifact_type: ArtifactType,
        name: String,
        content: Vec<u8>,
        metadata: Option<HashMap<String, serde_json::Value>>,
    ) -> Result<ArtifactId> {
        self.session_manager
            .store_artifact(session_id, artifact_type, name, content, metadata)
            .await
    }
}
```

## GlobalObject Layer

The GlobalObject layer provides thread-safe, synchronous wrappers following the HookBridge pattern:

### Pattern Implementation

```rust
// In globals/mod.rs - Bridges created externally
let session_bridge = Arc::new(SessionBridge::new(session_manager.clone()));
let artifact_bridge = Arc::new(ArtifactBridge::new(session_manager));

builder.register(Arc::new(SessionGlobal::new(session_bridge)));
builder.register(Arc::new(ArtifactGlobal::new(artifact_bridge)));
```

### SessionGlobal

```rust
pub struct SessionGlobal {
    pub session_bridge: Arc<SessionBridge>,
}

impl GlobalObject for SessionGlobal {
    fn inject_lua(&self, lua: &mlua::Lua, context: &GlobalContext) -> Result<()> {
        crate::lua::globals::session::inject_session_global(
            lua,
            context,
            self.session_bridge.clone(),
        )
    }
}
```

## Script Layer Integration

The Lua bindings use `block_on_async` to convert async operations:

```rust
// In lua/globals/session.rs
pub fn inject_session_global(
    lua: &Lua,
    _context: &GlobalContext,
    session_bridge: Arc<SessionBridge>,
) -> mlua::Result<()> {
    let session_table = lua.create_table()?;
    
    // Create method
    let create_fn = lua.create_function(move |_lua, options: Option<Table>| {
        let options = table_to_create_options(options)?;
        let session_id = block_on_async(
            "session_create",
            async move { bridge.create_session(options).await },
            None,
        )?;
        Ok(session_id.to_string())
    })?;
    session_table.set("create", create_fn)?;
    
    // ... other methods
    
    lua.globals().set("Session", session_table)?;
    Ok(())
}
```

## Storage and Persistence

### Content-Addressed Storage

Artifacts use BLAKE3 hashing for content addressing:

```rust
impl ArtifactStorage {
    async fn store_content(&self, content: &[u8]) -> Result<String> {
        let hash = blake3::hash(content).to_hex().to_string();
        let key = format!("content:{}", hash);
        
        // Check if already exists (deduplication)
        if !self.storage_backend.exists(&key).await? {
            self.storage_backend.set(&key, content).await?;
        }
        
        Ok(hash)
    }
}
```

### Compression

Large artifacts (>10KB) are automatically compressed:

```rust
const COMPRESSION_THRESHOLD: usize = 10 * 1024;

impl SessionArtifact {
    pub fn compress(&mut self) -> Result<()> {
        if self.metadata.is_compressed {
            return Ok(());
        }
        
        let compressed = compress_prepend_size(&self.content);
        if compressed.len() < self.metadata.size {
            self.content = compressed;
            self.metadata.is_compressed = true;
            self.metadata.original_size = Some(self.metadata.size);
            self.metadata.size = self.content.len();
        }
        Ok(())
    }
}
```

## Hook Integration

Sessions fire hooks at key lifecycle points:

```rust
pub enum HookPoint {
    SessionCreate,
    SessionSuspend,
    SessionResume,
    SessionComplete,
    SessionSave,
    SessionLoad,
    ArtifactStore,
    ArtifactRetrieve,
    ArtifactDelete,
}
```

Example hook implementation:

```rust
impl SessionManager {
    async fn fire_session_hook(&self, point: HookPoint, session: &Session) -> Result<()> {
        let hooks = self.hook_registry.get_hooks(&point);
        for hook in hooks {
            let mut ctx = SessionHookContext::new(point, session);
            self.hook_executor.execute_hook(&hook, &mut ctx).await?;
        }
        Ok(())
    }
}
```

## Event System Integration

Sessions publish events for monitoring and correlation:

```rust
pub struct SessionEvent {
    pub session_id: SessionId,
    pub event_type: SessionEventType,
    pub timestamp: DateTime<Utc>,
    pub metadata: HashMap<String, serde_json::Value>,
}

pub enum SessionEventType {
    Created,
    StateChanged { from: SessionStatus, to: SessionStatus },
    ArtifactAdded { artifact_id: ArtifactId },
    Completed,
}
```

## Best Practices for Extending

### Adding New Session Operations

1. Add method to `SessionManager`
2. Add async method to `SessionBridge`
3. Add sync wrapper in `lua/globals/session.rs`
4. Add appropriate hooks and events
5. Update tests

### Adding New Artifact Types

1. Extend `ArtifactType` enum
2. Update `parse_artifact_type` in conversions
3. Add any special handling in `store_artifact`
4. Update documentation

### Performance Optimization

1. **Batch Operations**: Group related operations
2. **Async Patterns**: Use tokio effectively
3. **Caching**: Consider caching frequently accessed data
4. **Compression**: Tune `COMPRESSION_THRESHOLD`

## Testing

### Unit Tests

```rust
#[tokio::test]
async fn test_session_lifecycle() {
    let manager = create_test_manager().await;
    let session_id = manager.create_session(Default::default()).await?;
    
    // Test operations
    manager.suspend_session(&session_id).await?;
    manager.resume_session(&session_id).await?;
    manager.complete_session(&session_id).await?;
}
```

### Integration Tests

```lua
-- Test through Lua API
local session_id = Session.create({name = "Test"})
Session.save(session_id)
Session.complete(session_id)
```

## Security Considerations

1. **Session Isolation**: Sessions cannot access each other's artifacts without permission
2. **Input Validation**: All inputs validated at bridge layer
3. **Resource Limits**: Configurable limits on session/artifact counts
4. **Sanitization**: Paths and metadata sanitized

## Future Enhancements

1. **Artifact Permissions**: Fine-grained access control
2. **Session Templates**: Predefined session configurations
3. **Artifact Versioning**: Track artifact history
4. **Search**: Full-text search across artifacts
5. **Replication**: Multi-backend storage support