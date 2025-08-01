# llmspell-sessions

Session and Artifact management for rs-llmspell, providing comprehensive session lifecycle management, artifact storage, and replay capabilities.

## Features

- **Session Management**: Create, suspend, resume, and complete sessions with full lifecycle tracking
- **Artifact Storage**: Store and retrieve arbitrary data associated with sessions
- **Content-Addressed Storage**: BLAKE3-based content hashing for deduplication
- **Automatic Compression**: LZ4 compression for artifacts over 10KB
- **Session Replay**: Replay session events with checkpoint and recovery support
- **Thread-Safe Operations**: Arc/RwLock patterns for concurrent access
- **Flexible Storage**: Support for multiple storage backends (memory, sled)

## Architecture

The crate is organized into several key modules:

- `manager`: Core SessionManager orchestrating all operations
- `session`: Session struct and lifecycle management
- `artifact`: Artifact types and storage system
- `replay`: Session replay engine with checkpoint support
- `bridge`: Integration with llmspell-bridge for script access
- `security`: Session isolation and access control

## Usage

### Basic Session Operations

```rust
use llmspell_sessions::{SessionManager, SessionManagerConfig, CreateSessionOptions};

// Create a session manager
let config = SessionManagerConfig::default();
let manager = SessionManager::new(config).await?;

// Create a new session
let options = CreateSessionOptions {
    name: Some("My Session".to_string()),
    description: Some("Example session".to_string()),
    tags: vec!["example".to_string(), "demo".to_string()],
    ..Default::default()
};
let session_id = manager.create_session(options).await?;

// Work with the session
// ...

// Complete the session
manager.complete_session(&session_id).await?;
```

### Artifact Storage

```rust
use llmspell_sessions::artifact::{ArtifactType, ArtifactMetadata};

// Store an artifact
let artifact_id = manager.store_artifact(
    &session_id,
    "report.txt".to_string(),
    b"Analysis report content".to_vec(),
    ArtifactType::UserInput,
    Some(ArtifactMetadata {
        description: Some("Analysis report".to_string()),
        tags: vec!["report".to_string()],
        mime_type: Some("text/plain".to_string()),
        ..Default::default()
    })
).await?;

// Retrieve the artifact
let (artifact, content) = manager.get_artifact(&session_id, &artifact_id).await?;
```

### Session Persistence

```rust
// Save session state
manager.save_session(&session_id).await?;

// Load a saved session
let loaded_id = manager.load_session(&session_id).await?;

// List all sessions
let sessions = manager.list_sessions(None).await?;
```

## Configuration

Sessions are configured via `SessionManagerConfig`:

```rust
use llmspell_sessions::config::SessionManagerConfig;
use std::time::Duration;

let config = SessionManagerConfig {
    max_sessions: 100,
    max_artifacts_per_session: 1000,
    session_timeout: Duration::from_secs(3600),
    artifact_compression_threshold: 10240, // 10KB
    storage_backend: "memory".to_string(),
    ..Default::default()
};
```

## Storage Backends

- **Memory**: Fast in-memory storage, data lost on restart
- **Sled**: Embedded database providing persistence

Set the storage path via environment variable:
```bash
export LLMSPELL_SESSION_PATH="/path/to/sessions"
```

## Performance

Achieved performance metrics:
- Session creation: 24.5µs
- Session save: 15.3µs  
- Session load: 3.4µs
- Hook overhead: 11µs
- Artifact storage with compression: <1ms

## Security

- **Session Isolation**: Each session can only access its own resources
- **Access Control**: SessionSecurityManager enforces strict isolation
- **Size Limits**: 100MB max artifact size
- **Resource Limits**: Configurable per-session limits

## Integration

The crate integrates with:
- `llmspell-state-persistence`: For session state management
- `llmspell-hooks`: For lifecycle event handling
- `llmspell-events`: For event correlation and tracking
- `llmspell-storage`: For backend storage abstraction
- `llmspell-bridge`: For script language access

## Examples

See the `examples/lua/session/` directory for comprehensive Lua examples:
- `basic.lua`: Fundamental session operations
- `artifacts.lua`: Artifact storage and retrieval
- `replay.lua`: Session replay and recovery
- `advanced.lua`: Advanced patterns and hierarchies
- `integration.lua`: Integration with other components

## License

This crate is part of the rs-llmspell project and follows the same license terms.