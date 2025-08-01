# Session and Artifact API Guide

This guide covers the Session and Artifact management APIs available in llmspell scripts.

## Table of Contents
- [Session API](#session-api)
  - [Creating Sessions](#creating-sessions)
  - [Managing Session Lifecycle](#managing-session-lifecycle)
  - [Querying Sessions](#querying-sessions)
  - [Session Persistence](#session-persistence)
  - [Current Session Context](#current-session-context)
- [Artifact API](#artifact-api)
  - [Storing Artifacts](#storing-artifacts)
  - [Retrieving Artifacts](#retrieving-artifacts)
  - [Managing Artifacts](#managing-artifacts)
  - [Working with Files](#working-with-files)

## Session API

The Session API provides comprehensive session management capabilities for tracking conversations, agent interactions, and workflow executions.

### Creating Sessions

```lua
-- Create a new session with optional metadata
local session_id = Session.create({
    name = "Customer Support Chat",
    description = "Support session for ticket #12345",
    tags = {"support", "priority-high", "customer-123"},
    metadata = {
        ticket_id = "12345",
        customer_id = "123",
        department = "technical"
    }
})

-- Create a child session
local child_id = Session.create({
    name = "Technical Investigation",
    parent_session_id = session_id
})
```

### Managing Session Lifecycle

Sessions have four states: `active`, `suspended`, `completed`, and `failed`.

```lua
-- Get session metadata
local metadata = Session.get(session_id)
print("Session name:", metadata.name)
print("Status:", metadata.status)
print("Created at:", metadata.created_at)

-- Suspend a session (can be resumed later)
Session.suspend(session_id)

-- Resume a suspended session
Session.resume(session_id)

-- Complete a session (marks as finished, removes from active)
Session.complete(session_id)

-- Delete a session entirely
Session.delete(session_id)
```

### Querying Sessions

```lua
-- List all active sessions
local sessions = Session.list()

-- Query sessions with filters
local filtered = Session.list({
    status = "active",
    tags = {"support"},
    limit = 10,
    sort_by = "created_at"
})

-- Find sessions created in the last hour
local recent = Session.list({
    created_after = os.date("!%Y-%m-%dT%H:%M:%SZ", os.time() - 3600)
})
```

### Session Persistence

Sessions can be saved to persistent storage and loaded later.

```lua
-- Save a session to storage
Session.save(session_id)

-- Load a session from storage
-- This restores it to active sessions
local loaded_id = Session.load(session_id)
```

### Current Session Context

Manage thread-local session context for implicit operations.

```lua
-- Set current session
Session.setCurrent(session_id)

-- Get current session
local current = Session.getCurrent()

-- Clear current session
Session.setCurrent(nil)

-- Operations that use current session if not specified
local artifacts = Artifact.list("")  -- Uses current session
```

## Artifact API

The Artifact API manages content storage within sessions, including conversation history, tool outputs, and generated content.

### Storing Artifacts

```lua
-- Store text content
local artifact_id = Artifact.store(
    session_id,
    "tool_result",        -- Type: tool_result, agent_output, user_input, system_generated
    "analysis.txt",       -- Name
    "Analysis results...", -- Content
    {
        mime_type = "text/plain",
        tags = {"analysis", "important"},
        source = "data_analyzer_tool",
        custom_field = "custom_value"
    }
)

-- Store binary content (images, PDFs, etc.)
local image_data = io.open("chart.png", "rb"):read("*a")
local image_id = Artifact.store(
    session_id,
    "agent_output",
    "chart.png",
    image_data,
    {
        mime_type = "image/png",
        description = "Performance metrics chart"
    }
)

-- Store structured data
local json_id = Artifact.store(
    session_id,
    "system_generated",
    "config.json",
    '{"setting": "value", "enabled": true}',
    {
        mime_type = "application/json",
        version = "1.0"
    }
)
```

### Retrieving Artifacts

```lua
-- Get artifact by ID
local artifact = Artifact.get(session_id, artifact_id)

-- Access content (automatically decompressed if needed)
print("Content:", artifact.content)

-- Access metadata
print("Name:", artifact.metadata.name)
print("Type:", artifact.metadata.artifact_type)
print("MIME type:", artifact.metadata.mime_type)
print("Size:", artifact.metadata.size)
print("Tags:", table.concat(artifact.metadata.tags, ", "))

-- Access custom metadata
if artifact.metadata.custom.source then
    print("Source:", artifact.metadata.custom.source)
end
```

### Managing Artifacts

```lua
-- List all artifacts in a session
local artifacts = Artifact.list(session_id)
for i, artifact in ipairs(artifacts) do
    print(i .. ". " .. artifact.name .. " (" .. artifact.artifact_type .. ")")
end

-- Delete an artifact
Artifact.delete(session_id, artifact_id)

-- Use current session context
Session.setCurrent(session_id)
local current_artifacts = Artifact.list("")  -- Empty string uses current session
```

### Working with Files

```lua
-- Store a file directly from disk
local file_id = Artifact.storeFile(
    session_id,
    "/path/to/document.pdf",
    "tool_result",
    {
        description = "Generated report",
        tags = {"report", "monthly"},
        generated_by = "report_generator"
    }
)

-- The file name is automatically extracted
local file_artifact = Artifact.get(session_id, file_id)
print("Stored file:", file_artifact.metadata.name)  -- "document.pdf"
```

## Best Practices

### Session Management

1. **Use descriptive names**: Help identify sessions in listings
2. **Tag appropriately**: Use consistent tags for easy filtering
3. **Clean up**: Complete or delete sessions when done
4. **Save important sessions**: Use `Session.save()` for persistence
5. **Use parent-child relationships**: Organize related sessions

### Artifact Storage

1. **Choose appropriate types**: Use the correct artifact type for clarity
2. **Set MIME types**: Always specify MIME type for proper handling
3. **Use compression**: Large artifacts (>10KB) are automatically compressed
4. **Add metadata**: Rich metadata helps with later retrieval
5. **Binary data handling**: Lua strings can hold binary data safely

### Performance Considerations

1. **Batch operations**: Store multiple related artifacts together
2. **Query limits**: Use `limit` parameter to avoid loading too much data
3. **Content size**: Large artifacts are automatically compressed
4. **Cleanup**: Delete unneeded artifacts to save storage

## Error Handling

```lua
-- Wrap operations in pcall for error handling
local success, result = pcall(Session.create, {
    name = "New Session"
})

if not success then
    print("Failed to create session:", tostring(result))
    return
end

local session_id = result

-- Handle artifact storage errors
local ok, artifact_id = pcall(Artifact.store, 
    session_id, "tool_result", "output.txt", content)
    
if not ok then
    print("Failed to store artifact:", tostring(artifact_id))
end
```

## Example: Complete Workflow

```lua
-- Create a session for a analysis task
local session_id = Session.create({
    name = "Data Analysis - Q4 2024",
    description = "Quarterly performance analysis",
    tags = {"analysis", "q4-2024", "performance"}
})

-- Set as current session
Session.setCurrent(session_id)

-- Store input data
local input_id = Artifact.store(
    session_id,
    "user_input",
    "requirements.txt",
    "Analyze sales data for Q4 2024...",
    {mime_type = "text/plain"}
)

-- Run analysis (hypothetical)
local results = analyze_data()

-- Store results
local result_id = Artifact.store(
    session_id,
    "agent_output", 
    "analysis_results.json",
    json.encode(results),
    {
        mime_type = "application/json",
        tags = {"results", "final"}
    }
)

-- Store generated chart
local chart_id = Artifact.storeFile(
    session_id,
    "/tmp/chart.png",
    "agent_output",
    {
        mime_type = "image/png",
        description = "Q4 sales chart"
    }
)

-- Save session for later reference
Session.save(session_id)

-- Mark complete
Session.complete(session_id)

print("Analysis complete. Session:", session_id)
```