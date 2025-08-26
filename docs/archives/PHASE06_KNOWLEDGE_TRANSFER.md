# Phase 6 Knowledge Transfer - Session and Artifact Management

**Date**: July 29, 2025  
**From Phase**: 5 (Persistent State Management)  
**To Phase**: 6 (Session and Artifact Management)  
**Prepared By**: Phase 5 Implementation Team

---

## Quick Start for Phase 6 Development

### 1. Understanding What You're Building

Phase 6 focuses on **Session and Artifact Management**, building directly on Phase 5's state persistence infrastructure. Your main deliverables will be:

- **Session Lifecycle Management**: Create, save, restore sessions with hooks
- **Artifact Storage System**: Store and retrieve artifacts with metadata
- **Session Context Preservation**: Maintain context across restarts
- **Session Replay**: Leverage ReplayableHook for session reconstruction

### 2. Key Infrastructure You'll Use

#### From Phase 5 (State Management):
```rust
// Session state operations
use llmspell_state_persistence::{StateManager, StateScope};

// Use the Session scope variant
let scope = StateScope::Session(session_id.to_string());
state_manager.set(scope, "last_command", json!(command)).await?;
```

#### From Phase 4 (Hooks):
```rust
// Session lifecycle hooks
use llmspell_hooks::{HookExecutor, HookContext, HookPoint};

let context = HookContext::new()
    .with_metadata("session_id", session_id)
    .with_metadata("user_id", user_id);
    
hook_executor.execute_hooks(
    HookPoint::Custom("session:start"),
    context
).await?;
```

#### From Phase 3.3 (Storage):
```rust
// Artifact storage
use llmspell_storage::{StorageBackend, StorageSerialize};

impl StorageSerialize for SessionArtifact {
    fn serialize_for_storage(&self) -> Result<Vec<u8>> {
        // Use existing patterns
    }
}
```

---

## Critical Design Decisions from Phase 5

### 1. Crate Architecture Pattern
**Learning**: Create separate crates to avoid circular dependencies
```toml
# Recommended structure for Phase 6
[package]
name = "llmspell-sessions"

[dependencies]
llmspell-state-persistence = { path = "../llmspell-state-persistence" }
llmspell-storage = { path = "../llmspell-storage" }
llmspell-hooks = { path = "../llmspell-hooks" }
llmspell-events = { path = "../llmspell-events" }
```

### 2. Performance Considerations
- **State operations**: <5ms (validated in Phase 5)
- **Hook overhead**: <2% (measured and acceptable)
- **Use StateClass**: Consider `StateClass::Archive` for old sessions

### 3. Security Model
- **Session isolation**: Already enforced via StateScope
- **Sensitive data**: Protection mechanisms in place
- **Consider**: Session-specific encryption keys

---

## Implementation Recommendations

### Session Manager Architecture
```rust
pub struct SessionManager {
    state_manager: Arc<StateManager>,
    storage_backend: Arc<dyn StorageBackend>,
    hook_executor: Arc<HookExecutor>,
    correlation_tracker: Arc<EventCorrelationTracker>,
}

impl SessionManager {
    pub async fn create_session(&self, config: SessionConfig) -> Result<Session> {
        let session_id = Uuid::new_v4().to_string();
        
        // Fire session:start hook
        let context = HookContext::new()
            .with_metadata("session_id", &session_id)
            .with_metadata("config", serde_json::to_value(&config)?);
            
        self.hook_executor.execute_hooks(
            HookPoint::Custom("session:start"),
            context
        ).await?;
        
        // Initialize session state
        let scope = StateScope::Session(session_id.clone());
        self.state_manager.set(scope, "created_at", json!(Utc::now())).await?;
        
        Ok(Session { id: session_id, config })
    }
}
```

### Artifact Storage Pattern
```rust
#[derive(Debug, Serialize, Deserialize)]
pub struct SessionArtifact {
    pub id: String,
    pub session_id: String,
    pub artifact_type: ArtifactType,
    pub content: Vec<u8>,
    pub metadata: ArtifactMetadata,
    pub created_at: DateTime<Utc>,
}

impl StorageSerialize for SessionArtifact {
    fn serialize_for_storage(&self) -> Result<Vec<u8>> {
        bincode::serialize(self)
            .map_err(|e| StorageError::SerializationFailed(e.to_string()))
    }
    
    fn storage_key(&self) -> String {
        format!("artifact:{}:{}", self.session_id, self.id)
    }
}
```

---

## Testing Strategy

### Test Categories to Add
```rust
#[cfg_attr(test_category = "session")]
#[tokio::test]
async fn test_session_lifecycle() {
    // Your session tests here
}
```

### Running Tests
```bash
# Run session-specific tests
./scripts/test-by-tag.sh session

# Quality checks
./scripts/quality-check-fast.sh
```

---

## Common Patterns from Phase 5

### 1. Async Everything
All state operations are async - embrace it:
```rust
// Good
let value = state_manager.get(scope, key).await?;

// Bad - trying to block
let value = tokio::task::block_in_place(|| {
    Handle::current().block_on(state_manager.get(scope, key))
});
```

### 2. Error Context
Always add context to errors:
```rust
state_manager.set(scope, key, value)
    .await
    .context("Failed to save session state")?;
```

### 3. Correlation IDs
Link everything through correlation:
```rust
let correlation_id = self.correlation_tracker.create_correlation_id();
// Use this ID across all related operations
```

---

## Gotchas and Solutions

### 1. State Scope Mixing
**Problem**: Mixing global and session state
**Solution**: Always use `StateScope::Session(id)` for session data

### 2. Hook Performance
**Problem**: Too many hooks slowing operations
**Solution**: Monitor with performance benchmarks, use CircuitBreaker if needed

### 3. Large Artifacts
**Problem**: Storing large artifacts in state
**Solution**: Store artifacts in StorageBackend, only metadata in state

---

## Resources and Support

### Documentation
- State system: `/docs/state-management/README.md`
- Best practices: `/docs/state-management/best-practices.md`
- Architecture: `/docs/technical/state-architecture.md`

### Examples
- Basic state: `/examples/state_persistence/basic_operations.rs`
- Lua examples: `/examples/lua/state/`

### Key Files to Study
- `llmspell-state-persistence/src/manager.rs` - StateManager implementation
- `llmspell-state-persistence/src/scope.rs` - StateScope with Session variant
- `llmspell-hooks/src/executor.rs` - Hook execution patterns
- `llmspell-events/src/correlation.rs` - Event correlation

---

## Phase 6 Success Criteria Alignment

Based on Phase 5's infrastructure, you can achieve:
- ✅ Sessions can be created, saved, and restored (StateManager ready)
- ✅ Artifacts stored with metadata (StorageBackend available)
- ✅ Session context preserved (StateScope::Session implemented)
- ✅ Session replay functionality (ReplayableHook integration complete)
- ✅ Session hooks at boundaries (Hook infrastructure ready)
- ✅ Automatic artifact collection (Hook + correlation ready)
- ✅ Event correlation (UniversalEvent + correlation IDs ready)

---

## Questions?

The Phase 5 implementation provides a solid foundation for Phase 6. All the infrastructure you need is in place:
- State persistence for sessions
- Hook system for lifecycle management
- Storage backend for artifacts
- Correlation system for linking
- Testing framework for validation

Good luck with Phase 6! The groundwork is solid - build confidently on top of it.