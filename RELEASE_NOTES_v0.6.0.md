# Release Notes - v0.6.0

**Release Date**: August 1, 2025  
**Phase**: 6 - Session and Artifact Management  
**Status**: Production Ready

## Executive Summary

rs-llmspell v0.6.0 introduces comprehensive session and artifact management capabilities that enable long-running interactions, artifact storage, and session replay. This release adds a sophisticated session lifecycle system with suspend/resume capabilities, content-addressed artifact storage, and full replay functionality. Building on Phase 5's persistent state foundation, this release completes the infrastructure needed for production AI agent workflows.

## Major Features

### 🎯 Session Lifecycle Management

- **Complete Session System**: Create, suspend, resume, and complete long-running sessions
- **Session States**: Active, Suspended, Completed, Failed, Archived with validated transitions
- **Performance Excellence**: Session creation in 24.5μs (2000x better than 50ms target)
- **Session Persistence**: Save and restore full session context across restarts
- **Thread-Local Context**: Active session tracking for simplified API usage

### 📦 Artifact Storage System

- **Content-Addressed Storage**: Blake3 hashing (10x faster than SHA2) for deduplication
- **Automatic Collection**: Tool outputs and agent responses captured automatically
- **User Storage API**: Direct artifact storage for user files and data
- **Rich Metadata**: Tags, MIME types, custom fields with preservation
- **Compression**: LZ4 compression for artifacts >10KB with 50%+ ratio
- **Versioning**: Automatic version management for same-named artifacts

### 🔄 Session Replay Engine

- **Full Replay Capability**: Replay entire sessions using hook execution history
- **Timeline Reconstruction**: Query session events with correlation tracking
- **Debug Support**: Step through session execution with state inspection
- **Performance**: Minimal overhead leveraging existing replay infrastructure

### 🪝 Hook and Event Integration

- **Session Lifecycle Hooks**: session:start, end, suspend, resume with contexts
- **Artifact Collectors**: ToolResultCollector and AgentOutputCollector
- **Event Correlation**: All session events correlated with timeline support
- **Replayable Hooks**: Session hooks implement ReplayableHook trait
- **Performance**: 11μs hook overhead (90x better than 1ms target)

### 📋 Session Policies and Middleware

- **Timeout Policies**: Session duration and idle timeout enforcement
- **Resource Limits**: Memory, token, operation, and cost tracking
- **Rate Limiting**: Global, per-session, and per-operation limits
- **Middleware Patterns**: Sequential, Parallel, and Voting execution
- **Policy Composition**: Combine policies with different strategies

### 🌐 Script Bridge Implementation

- **Session Global**: Complete Lua API for session management
- **Artifact Global**: Store, retrieve, list, and delete artifacts
- **5 Example Suites**: Basic, artifacts, replay, advanced, integration
- **Runtime Integration**: Full CLI support with configuration

## API Enhancements

### Lua Session API
```lua
-- Create and manage sessions
local session = Session.create({
    name = "research_session",
    max_duration = 3600,  -- 1 hour
    tags = {"research", "analysis"}
})

-- Suspend and resume sessions
Session.suspend(session.id)
-- ... later ...
Session.resume(session.id)

-- Save and restore sessions
Session.save(session.id)
local restored = Session.load(session.id)

-- Get current session context
Session.setCurrent(session.id)
local current = Session.getCurrent()
```

### Lua Artifact API
```lua
-- Store artifacts with metadata
local artifact_id = Artifact.store(session.id, "analysis.json", {
    summary = "Market analysis complete",
    revenue = 1000000,
    growth = 0.15
}, {
    mime_type = "application/json",
    tags = {"quarterly", "finance"}
})

-- Retrieve artifacts
local artifact = Artifact.get(session.id, artifact_id)
local content = Artifact.getContent(session.id, artifact_id)

-- Query artifacts
local artifacts = Artifact.list(session.id, {
    artifact_type = "UserInput",
    tags = {"finance"}
})

-- Store files directly
local file_id = Artifact.storeFile(session.id, "/path/to/report.pdf", {
    description = "Q4 financial report"
})
```

## Performance Achievements

| Component | Target | Achieved | Improvement |
|-----------|--------|----------|-------------|
| Session Creation | <50ms | 24.5μs | ✅ 2000x better |
| Session Save | <50ms | 15.3μs | ✅ 3200x better |
| Session Load | <50ms | 3.4μs | ✅ 14700x better |
| Artifact Store | <5ms | <1ms | ✅ 5x better |
| Hook Overhead | <1ms | 11μs | ✅ 90x better |
| Memory Overhead | <20% | <10% | ✅ 2x better |

## Architecture Innovations

### Session Infrastructure
- **llmspell-sessions crate**: 39 completed tasks across 10 subsystems
- **Three-Layer Bridge**: SessionBridge → SessionGlobal → Lua bindings
- **Foundation-First**: Test categorization implemented before API work

### Module Organization
```
llmspell-sessions/
├── Core: session.rs, manager.rs, types.rs, config.rs
├── artifact/: storage.rs, versioning.rs, metadata.rs, search.rs
├── replay/: session_adapter.rs, controls.rs, debug.rs
├── hooks/: collectors.rs, session_hooks.rs, context_extensions.rs
├── policies/: timeout.rs, resource_limit.rs, rate_limit.rs
├── middleware/: session_middleware.rs
├── analytics/: session_metrics.rs
└── bridge/: operations.rs, conversions.rs, errors.rs
```

## Breaking Changes

None - v0.6.0 maintains full backward compatibility with v0.5.0 APIs.

## Migration Guide

No migration required. To leverage new features:

1. **Enable Sessions**:
   ```toml
   # llmspell.toml
   [session]
   enabled = true
   storage_path = "./sessions"
   max_active_sessions = 100
   artifact_compression = true
   ```

2. **Use Sessions in Scripts**:
   ```lua
   -- Create session with auto-suspend
   local session = Session.create({name = "task"})
   Session.setCurrent(session.id)
   
   -- Artifacts automatically associated
   local agent = Agent.create({model = "gpt-4"})
   local result = agent:execute({prompt = "Analyze data"})
   -- Result automatically collected as artifact
   ```

3. **Setup Session Policies**:
   ```lua
   -- Add timeout policy
   local session = Session.create({
       name = "limited_session",
       max_duration = 600,  -- 10 minutes
       max_idle_time = 120  -- 2 minute idle timeout
   })
   ```

## What's Next (Phase 7)

Phase 7 will focus on API consistency and standardization:
- Comprehensive API inventory and inconsistency analysis
- Standardization of naming conventions across all crates
- Builder patterns for all configuration objects
- Workflow-Agent trait integration (Google ADK pattern)
- Test organization leveraging categorization system
- Documentation polish for 1.0 release

## Key Implementation Highlights

### Performance Engineering
- **Blake3 Hashing**: 10x faster than SHA2 for content addressing
- **LZ4 Compression**: Fast compression for large artifacts
- **Chunked Storage**: Efficient handling of large binary data
- **Lock-Free Reads**: Optimized concurrent session access

### Security Features
- **Session Isolation**: Enforced boundaries between sessions
- **Access Control**: Session-based artifact permissions
- **Content Integrity**: Hash verification on all operations
- **Audit Logging**: All session operations tracked

### Developer Experience
- **Thread-Local Context**: Simplified API with getCurrent/setCurrent
- **Rich Examples**: 5 comprehensive example suites
- **Automatic Collection**: Zero-configuration artifact capture
- **Consistent APIs**: Follows established patterns from previous phases

## Installation

Update your `Cargo.toml`:
```toml
[dependencies]
llmspell = "0.6.0"
```

Or use the CLI:
```bash
cargo install llmspell-cli --version 0.6.0
```

## Documentation

- [Session Management Guide](/docs/user-guide/session-management.md)
- [Session-Artifact API Guide](/docs/user-guide/session-artifact-api.md)
- [Session Examples](/examples/lua/session/) - 5 comprehensive examples
- [Developer Guide](/docs/developer-guide/session-artifact-implementation.md)
- [API Reference](/docs/user-guide/api-reference.md) - Updated with Session/Artifact

## Support

For issues or questions:
- GitHub Issues: https://github.com/lexlapax/rs-llmspell/issues
- Documentation: https://docs.rs/llmspell/0.6.0
- Discord: https://discord.gg/llmspell

## Phase 6 Metrics

- **Total Tasks**: 39/39 ✅ Complete
- **New Crate**: llmspell-sessions with 30+ modules
- **Performance**: All targets exceeded by 5-14700x
- **Test Coverage**: Comprehensive with proper categorization
- **Examples**: 5 example suites + runtime integration
- **Production Ready**: Session management ready for deployment

## Acknowledgments

Phase 6 successfully delivers production-ready session management that seamlessly integrates with the existing state persistence, hook, and event systems. The exceptional performance metrics demonstrate the power of the architectural decisions made throughout the project.

Special recognition to:
- The hook infrastructure that enabled automatic artifact collection
- The replay system that provided session replay with minimal new code
- The test categorization initiative that improved development velocity

This release brings rs-llmspell significantly closer to the 1.0 milestone, with only API standardization remaining before the stable release.