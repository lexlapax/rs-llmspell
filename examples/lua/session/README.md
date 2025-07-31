# Session and Artifact Examples

This directory contains comprehensive examples demonstrating the Session and Artifact management APIs in rs-llmspell.

## ✅ Runtime Integration Complete

The Session and Artifact globals are now fully integrated into the CLI runtime. All examples in this directory are ready to run!

## Running the Examples

First, ensure you have a configuration file with sessions enabled. Create `llmspell.toml`:

```toml
[runtime.sessions]
enabled = true
max_sessions = 100
max_artifacts_per_session = 1000
artifact_compression_threshold = 10240
session_timeout_seconds = 3600
storage_backend = "memory"
```

Then run any example:

```bash
# Build the CLI if not already built
cargo build --bin llmspell

# Run examples
./target/debug/llmspell -c llmspell.toml run examples/lua/session/basic.lua
./target/debug/llmspell -c llmspell.toml run examples/lua/session/artifacts.lua
./target/debug/llmspell -c llmspell.toml run examples/lua/session/replay.lua
./target/debug/llmspell -c llmspell.toml run examples/lua/session/advanced.lua
./target/debug/llmspell -c llmspell.toml run examples/lua/session/integration.lua
```

## Examples

### 1. basic.lua
Demonstrates fundamental session operations:
- Creating sessions with metadata
- Session lifecycle management (suspend, resume, complete)
- Session persistence (save/load)
- Thread-local session context
- Querying and filtering sessions

### 2. artifacts.lua
Shows artifact storage and retrieval:
- Storing text, JSON, and binary artifacts
- Metadata and tagging
- Content-addressed storage with BLAKE3
- Automatic compression for large artifacts
- File storage from disk
- Artifact deletion

### 3. replay.lua
Illustrates session replay and recovery:
- Multi-step processing with checkpoints
- Failure simulation and recovery
- Session history analysis
- Recovery strategies
- Checkpoint management

### 4. advanced.lua
Covers advanced patterns:
- Complex session hierarchies
- Session templates
- Bulk operations
- Performance metrics
- Session analytics
- Advanced querying

### 5. integration.lua
Demonstrates integration with other globals:
- Session + State integration
- Session + Event integration
- Session + Hook integration (API update needed)
- Session + Agent integration
- Session + Tool integration
- Session + Workflow integration
- Cross-component patterns

## Implementation Status

- ✅ Session and Artifact globals implemented
- ✅ All tests passing in test environment
- ✅ Examples created following best practices
- ✅ Runtime integration complete
- ✅ Configuration support implemented
- ✅ All examples functional in CLI

## Configuration Options

The session system can be configured in your `llmspell.toml`:

```toml
[runtime.sessions]
enabled = true                           # Enable session management
max_sessions = 100                       # Maximum concurrent sessions
max_artifacts_per_session = 1000        # Max artifacts per session
artifact_compression_threshold = 10240   # Compress artifacts > 10KB
session_timeout_seconds = 3600          # Session timeout (1 hour)
storage_backend = "memory"              # Storage backend (memory or sled)
```

## Storage Backends

- **memory**: In-memory storage (default), fast but not persistent
- **sled**: Embedded database, provides persistence across restarts

For persistent sessions, use the sled backend and set the storage path:

```bash
export LLMSPELL_SESSION_PATH="./my_sessions"
```

## Best Practices

1. **Session Lifecycle**: Always complete sessions when done to free resources
2. **Artifact Storage**: Use meaningful artifact types and metadata for easy retrieval
3. **Error Handling**: Wrap session operations in pcall for proper error handling
4. **Thread-Local Context**: Use Session.setCurrent() for implicit session context
5. **Cleanup**: Clean up state keys when sessions complete

## Known Limitations

- Hook integration example needs update for the new Hook API (register instead of create)
- Agent integration requires valid API keys configured for the provider