# Session and Artifact Global API Design

## Overview

This document defines the API design for the Session and Artifact global objects that will be exposed to Lua, JavaScript, and Python scripts. The design follows the established GlobalObject pattern from Phase 5 and leverages existing infrastructure.

## Architecture

### Three-Layer Design

1. **Core Bridge Layer** (`SessionBridge`, `ArtifactBridge`)
   - Language-agnostic wrappers around SessionManager and ArtifactStorage
   - Handle async-to-sync conversion using block_on pattern
   - Provide type conversions and error handling

2. **GlobalObject Layer** (`SessionGlobal`, `ArtifactGlobal`)
   - Implement the GlobalObject trait
   - Manage bridge lifecycle and dependencies
   - Provide metadata for registration

3. **Language Binding Layer** (Lua/JS/Python specific)
   - Create language-specific tables/objects
   - Handle type conversions
   - Provide idiomatic APIs

## Session Global API

### Core Operations

```lua
-- Session lifecycle management
Session.create(options)              -- Create new session
Session.get(session_id)              -- Get existing session
Session.list(query)                  -- List sessions with filtering
Session.getCurrent()                 -- Get current active session
Session.setCurrent(session_id)       -- Set current active session

-- Session state management
Session.suspend(session_id)          -- Suspend session
Session.resume(session_id)           -- Resume session
Session.complete(session_id)         -- Mark session complete
Session.delete(session_id)           -- Delete session

-- Session data access
Session.getMetadata(session_id)      -- Get session metadata
Session.updateMetadata(session_id, updates) -- Update metadata
Session.getTags(session_id)          -- Get session tags
Session.setTags(session_id, tags)    -- Set session tags

-- Session persistence
Session.save(session_id)             -- Save session to storage
Session.load(session_id)             -- Load session from storage
Session.saveAll()                    -- Save all active sessions
Session.restoreRecent(count)         -- Restore recent sessions

-- Session replay
Session.canReplay(session_id)        -- Check if replayable
Session.replay(session_id, options)  -- Replay session
Session.getTimeline(session_id)      -- Get event timeline
Session.getReplayMetadata(session_id) -- Get replay metadata
```

### Session Object Methods

```lua
-- When getting a session object
local session = Session.get(session_id)

-- Session object properties
session.id                           -- Session ID
session.metadata                     -- Metadata table
session.state                        -- Current state
session.created_at                   -- Creation timestamp
session.updated_at                   -- Last update timestamp

-- Session object methods
session:suspend()                    -- Suspend this session
session:resume()                     -- Resume this session
session:complete()                   -- Complete this session
session:save()                       -- Save this session
session:addTag(tag)                  -- Add a tag
session:removeTag(tag)               -- Remove a tag
session:hasTag(tag)                  -- Check for tag
```

## Artifact Global API

### Core Operations

```lua
-- Artifact storage
Artifact.store(session_id, artifact) -- Store artifact
Artifact.get(session_id, artifact_id) -- Get artifact
Artifact.getContent(session_id, artifact_id) -- Get content only
Artifact.list(session_id)            -- List session artifacts
Artifact.delete(session_id, artifact_id) -- Delete artifact

-- Artifact queries
Artifact.query(query)                -- Query artifacts
Artifact.findByType(session_id, type) -- Find by type
Artifact.findByTag(session_id, tag)  -- Find by tag
Artifact.search(session_id, text)    -- Text search

-- File artifacts
Artifact.storeFile(session_id, path, metadata) -- Store file
Artifact.saveToFile(session_id, artifact_id, path) -- Save to file

-- Access control
Artifact.grantPermission(session_id, artifact_id, user, permission)
Artifact.revokePermission(session_id, artifact_id, user, permission)
Artifact.getPermissions(session_id, artifact_id)
Artifact.checkAccess(session_id, artifact_id, user, permission)

-- Audit
Artifact.getAuditLog(session_id, artifact_id)
```

### Artifact Object Structure

```lua
-- Artifact structure when storing
local artifact = {
    content = "...",                 -- Content (string or binary)
    type = "text",                   -- Type: text, code, data, model, etc.
    metadata = {
        name = "example.txt",
        description = "Example file",
        tags = {"example", "test"},
        custom = {...}               -- Custom metadata
    }
}

-- Artifact object when retrieved
local artifact = Artifact.get(session_id, artifact_id)
artifact.id                          -- Artifact ID
artifact.session_id                  -- Session ID
artifact.content                     -- Content
artifact.type                        -- Type
artifact.metadata                    -- Metadata table
artifact.content_hash                -- Content hash
artifact.size                        -- Size in bytes
artifact.created_at                  -- Creation timestamp
artifact.updated_at                  -- Last update
artifact.version                     -- Version number
```

## Integration with Other Globals

### State Global Integration

```lua
-- Session state is automatically scoped
State.set("session:current", "key", value)  -- Session-scoped state
State.get("session:current", "key")         -- Get session state

-- Or use session ID directly
State.set("session:" .. session_id, "key", value)
```

### Hook Global Integration

```lua
-- Session lifecycle hooks
Hook.register("session.created", function(context)
    print("Session created:", context.session_id)
end)

Hook.register("session.completed", function(context)
    -- Archive session artifacts
end)

-- Artifact hooks
Hook.register("artifact.stored", function(context)
    print("Artifact stored:", context.artifact_id)
end)
```

### Event Global Integration

```lua
-- Session events are automatically correlated
Event.emit("custom_event", {
    session_id = Session.getCurrent(),
    data = {...}
})

-- Listen for session events
Event.on("session.state_changed", function(event)
    print("Session state changed:", event.data.new_state)
end)
```

## Error Handling

All methods follow consistent error handling:

```lua
-- Success case
local session_id, err = Session.create({name = "test"})
if err then
    print("Error creating session:", err)
    return
end

-- Or use protected calls
local ok, session_id = pcall(Session.create, {name = "test"})
if not ok then
    print("Error:", session_id)  -- session_id contains error message
end
```

## Implementation Notes

### Dependencies

- SessionGlobal depends on:
  - StateGlobal (for session state storage)
  - GlobalContext (for accessing bridges)
  
- ArtifactGlobal depends on:
  - SessionGlobal (artifacts belong to sessions)
  - GlobalContext (for accessing bridges)

### Registration Order

1. StateGlobal (already registered)
2. HookGlobal (already registered)  
3. EventGlobal (already registered)
4. SessionGlobal (new)
5. ArtifactGlobal (new)

### Performance Considerations

- Use caching for frequently accessed sessions
- Lazy-load artifact content
- Batch operations where possible
- Stream large artifacts

### Security Considerations

- Validate all session IDs
- Check permissions before operations
- Sanitize file paths
- Limit resource usage
- Prevent session hijacking

## Example Usage

### Basic Session Management

```lua
-- Create a new session
local session_id = Session.create({
    name = "Data Analysis",
    tags = {"analysis", "production"},
    metadata = {
        project = "Q4 Report",
        user = "analyst1"
    }
})

-- Set as current session
Session.setCurrent(session_id)

-- Store an artifact
Artifact.store(session_id, {
    content = results_data,
    type = "data",
    metadata = {
        name = "q4_results.json",
        format = "json",
        tags = {"results", "q4"}
    }
})

-- Complete the session
Session.complete(session_id)
```

### Advanced Replay Example

```lua
-- Check if session can be replayed
if Session.canReplay(session_id) then
    -- Start replay with progress tracking
    local replay_id = Session.replay(session_id, {
        speed = 2.0,  -- 2x speed
        on_progress = function(progress)
            print(string.format("Replay progress: %.1f%%", progress * 100))
        end,
        on_hook = function(hook_event)
            print("Replaying hook:", hook_event.hook_point)
        end
    })
    
    -- Get replay results
    local timeline = Session.getTimeline(session_id)
    for _, event in ipairs(timeline) do
        print(event.timestamp, event.type, event.description)
    end
end
```

### Artifact Access Control

```lua
-- Grant read access to a user
Artifact.grantPermission(session_id, artifact_id, "user123", "read")

-- Check access before operations
if Artifact.checkAccess(session_id, artifact_id, current_user, "write") then
    -- Update artifact
    Artifact.update(session_id, artifact_id, new_content)
end

-- View audit log
local audit_log = Artifact.getAuditLog(session_id, artifact_id)
for _, entry in ipairs(audit_log) do
    print(entry.timestamp, entry.user, entry.action)
end
```

## Migration Path

For users migrating from direct API usage:

1. Replace `manager:create_session()` with `Session.create()`
2. Replace `manager:store_artifact()` with `Artifact.store()`
3. Use `Session.getCurrent()` for context management
4. Leverage integration with State, Hook, and Event globals

## Future Extensions

- Session templates and cloning
- Artifact versioning and diffing
- Session collaboration features
- Artifact transformation pipelines
- Session analytics and insights