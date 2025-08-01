# Session Management Guide

This comprehensive guide covers session and artifact management in rs-llmspell, including best practices, patterns, and real-world examples.

## Table of Contents

1. [Introduction](#introduction)
2. [Core Concepts](#core-concepts)
3. [Session Lifecycle](#session-lifecycle)
4. [Artifact Storage](#artifact-storage)
5. [Configuration](#configuration)
6. [Best Practices](#best-practices)
7. [Advanced Patterns](#advanced-patterns)
8. [Troubleshooting](#troubleshooting)

## Introduction

Sessions in rs-llmspell provide a way to group related operations, maintain state, and track artifacts (data) produced during script execution. They are essential for:

- **Context Management**: Maintaining state across multiple operations
- **Data Organization**: Storing and retrieving artifacts with rich metadata
- **Audit Trails**: Tracking what happened during execution
- **Recovery**: Replaying sessions from checkpoints
- **Collaboration**: Sharing data between components

## Core Concepts

### Sessions

A session represents a logical unit of work with:
- Unique identifier (UUID)
- Lifecycle states (active, suspended, completed, failed)
- Metadata and tags for organization
- Parent-child relationships for hierarchies
- Automatic persistence capabilities

### Artifacts

Artifacts are data items stored within sessions:
- Content-addressed using BLAKE3 hashing
- Automatic deduplication
- Compression for large artifacts (>10KB)
- Rich metadata support
- Multiple types (user_input, tool_result, agent_output, system_generated)

## Session Lifecycle

### Creating Sessions

```lua
-- Basic session creation
local session_id = Session.create()

-- With metadata
local session_id = Session.create({
    name = "Data Analysis Session",
    description = "Processing Q4 sales data",
    tags = {"analysis", "sales", "q4-2024"},
    metadata = {
        department = "Sales",
        analyst = "jane.doe@company.com",
        priority = "high"
    }
})

-- Child session
local child_id = Session.create({
    name = "Data Validation",
    parent_session_id = session_id
})
```

### Managing Session State

Sessions have four lifecycle states:

1. **Active**: Normal operating state
2. **Suspended**: Temporarily paused, can be resumed
3. **Completed**: Successfully finished
4. **Failed**: Terminated due to error

```lua
-- Suspend session (e.g., for maintenance)
Session.suspend(session_id)

-- Resume suspended session
Session.resume(session_id)

-- Complete session (success)
Session.complete(session_id)

-- Mark session as failed
Session.fail(session_id, "Processing error occurred")
```

### Session Context

Set a current session to avoid passing IDs repeatedly:

```lua
-- Set current session
Session.setCurrent(session_id)

-- Get current session
local current = Session.getCurrent()

-- Clear current session
Session.setCurrent(nil)
```

### Persistence

Sessions can be saved and loaded:

```lua
-- Save session state
Session.save(session_id)

-- Load a saved session
local loaded_id = Session.load(session_id)

-- List all sessions
local all_sessions = Session.list()

-- Filter sessions
local active_sessions = Session.list({
    status = "active",
    tags = {"analysis"}
})
```

## Artifact Storage

### Storing Artifacts

```lua
-- Store text content
local artifact_id = Artifact.store(
    session_id,
    "user_input",              -- Type
    "report.txt",              -- Name
    "Quarterly report data",   -- Content
    {                          -- Metadata (optional)
        author = "John Doe",
        department = "Sales",
        tags = {"report", "q4"},
        version = "1.0"
    }
)

-- Store JSON data
local json_data = Tool.execute("json-processor", {
    operation = "stringify",
    input = {key = "value"}
}).result

local json_id = Artifact.store(
    session_id,
    "tool_result",
    "config.json",
    json_data
)

-- Store file from disk
local file_id = Artifact.storeFile(
    session_id,
    "/path/to/file.csv",
    "user_input",
    {
        uploaded_by = "user@example.com",
        purpose = "data_import"
    }
)
```

### Retrieving Artifacts

```lua
-- Get specific artifact
local artifact = Artifact.get(session_id, artifact_id)
if artifact then
    print("Content: " .. artifact.content)
    print("Metadata: " .. artifact.metadata.author)
end

-- List all artifacts in session
local artifacts = Artifact.list(session_id)
for _, artifact in ipairs(artifacts) do
    print(artifact.name .. " (" .. artifact.size .. " bytes)")
end
```

### Querying Artifacts

```lua
-- Query with filters
local results = Artifact.query({
    session_id = session_id,
    type = "user_input",                    -- Filter by type
    tags = {"report", "important"},         -- Filter by tags
    name_pattern = "*.csv",                 -- Filename pattern
    created_after = os.time() - 3600,       -- Last hour
    created_before = os.time(),             -- Until now
    min_size = 1024,                        -- At least 1KB
    max_size = 1048576,                     -- At most 1MB
    limit = 20                              -- Return max 20 results
})
```

### Deleting Artifacts

```lua
-- Delete specific artifact
Artifact.delete(session_id, artifact_id)

-- Bulk delete by query
local old_artifacts = Artifact.query({
    session_id = session_id,
    created_before = os.time() - 7 * 24 * 60 * 60  -- Older than 7 days
})

for _, artifact in ipairs(old_artifacts) do
    Artifact.delete(session_id, artifact.id)
end
```

## Configuration

### Runtime Configuration

Configure sessions in your `llmspell.toml`:

```toml
[runtime.sessions]
enabled = true                           # Enable session management
max_sessions = 100                       # Maximum concurrent sessions
max_artifacts_per_session = 1000        # Per-session artifact limit
artifact_compression_threshold = 10240   # Compress artifacts > 10KB
session_timeout_seconds = 3600          # Auto-cleanup after 1 hour
storage_backend = "sled"                # Use persistent storage
```

### Storage Backends

#### Memory Backend (Default)
- Fast, in-memory storage
- Data lost on restart
- Good for development and testing

#### Sled Backend
- Embedded database
- Persistent across restarts
- Production-ready

Set storage location:
```bash
export LLMSPELL_SESSION_PATH="/var/lib/llmspell/sessions"
```

## Best Practices

### 1. Always Complete Sessions

```lua
local function safeExecute()
    local session_id = Session.create({name = "Safe Operation"})
    
    local success, err = pcall(function()
        -- Your operations here
        processData()
    end)
    
    if success then
        Session.complete(session_id)
    else
        Session.fail(session_id, tostring(err))
    end
    
    return success, err
end
```

### 2. Use Meaningful Metadata

```lua
-- Good: Rich metadata for easy searching
local artifact_id = Artifact.store(
    session_id,
    "tool_result",
    "analysis_2024_q4.json",
    result_data,
    {
        analysis_type = "quarterly_revenue",
        fiscal_year = 2024,
        quarter = 4,
        department = "sales",
        tags = {"revenue", "quarterly", "financial"},
        processed_by = "revenue_analyzer_v2",
        processing_time_ms = 1234
    }
)

-- Bad: Minimal metadata
local artifact_id = Artifact.store(
    session_id,
    "tool_result", 
    "result.json",
    result_data
)
```

### 3. Implement Checkpointing

```lua
local function longRunningProcess(session_id)
    local steps = {"validate", "transform", "analyze", "report"}
    
    for i, step in ipairs(steps) do
        print("Processing step: " .. step)
        
        -- Process step
        local result = processStep(step)
        
        -- Store checkpoint
        Artifact.store(
            session_id,
            "system_generated",
            "checkpoint_" .. step .. ".json",
            Tool.execute("json-processor", {
                operation = "stringify",
                input = {
                    step = step,
                    step_number = i,
                    result = result,
                    timestamp = os.time()
                }
            }).result,
            {
                checkpoint = true,
                step_name = step,
                tags = {"checkpoint", step}
            }
        )
        
        -- Save session after each step
        Session.save(session_id)
    end
end
```

### 4. Handle Errors Gracefully

```lua
local function robustOperation()
    local session_id = Session.create({
        name = "Robust Operation",
        metadata = {retry_count = 0}
    })
    
    local max_retries = 3
    local retry_count = 0
    
    while retry_count < max_retries do
        local success, err = pcall(function()
            -- Attempt operation
            performOperation(session_id)
        end)
        
        if success then
            Session.complete(session_id)
            return true
        else
            retry_count = retry_count + 1
            
            -- Log error as artifact
            Artifact.store(
                session_id,
                "system_generated",
                string.format("error_attempt_%d.txt", retry_count),
                tostring(err),
                {
                    error_type = "operation_failure",
                    attempt = retry_count,
                    timestamp = os.time()
                }
            )
            
            if retry_count < max_retries then
                print("Retrying... (attempt " .. retry_count .. ")")
                -- Wait before retry
                os.execute("sleep 1")
            end
        end
    end
    
    Session.fail(session_id, "Max retries exceeded")
    return false
end
```

### 5. Clean Up Resources

```lua
-- Cleanup pattern
local function cleanupOldSessions(days_to_keep)
    local cutoff = os.time() - (days_to_keep * 24 * 60 * 60)
    local sessions = Session.list()
    
    for _, session in ipairs(sessions) do
        local metadata = Session.get(session.id)
        
        -- Only cleanup completed or failed sessions
        if (metadata.status == "completed" or metadata.status == "failed") 
           and metadata.updated_at < cutoff then
            
            -- Delete artifacts first
            local artifacts = Artifact.list(session.id)
            for _, artifact in ipairs(artifacts) do
                Artifact.delete(session.id, artifact.id)
            end
            
            -- Then delete session
            -- Note: Session deletion API would need to be implemented
            print("Would delete session: " .. session.id)
        end
    end
end
```

## Advanced Patterns

### Session Templates

```lua
-- Define reusable session templates
local templates = {
    data_pipeline = {
        name_prefix = "DataPipeline-",
        tags = {"pipeline", "automated"},
        metadata = {
            pipeline_version = "2.0",
            retry_policy = "exponential_backoff",
            max_retries = 3
        }
    },
    
    ml_training = {
        name_prefix = "MLTraining-",
        tags = {"ml", "training"},
        metadata = {
            framework = "tensorflow",
            gpu_required = true,
            checkpoint_interval = 1000
        }
    }
}

local function createFromTemplate(template_name, custom_name)
    local template = templates[template_name]
    if not template then
        error("Unknown template: " .. template_name)
    end
    
    return Session.create({
        name = template.name_prefix .. custom_name,
        tags = template.tags,
        metadata = template.metadata
    })
end
```

### Cross-Session Communication

```lua
-- Publish-subscribe pattern using artifacts
local function publishMessage(topic, message)
    local pub_session = Session.create({
        name = "Publisher-" .. topic,
        tags = {"publisher", topic}
    })
    
    local msg_id = Artifact.store(
        pub_session,
        "system_generated",
        "message_" .. os.time() .. ".json",
        Tool.execute("json-processor", {
            operation = "stringify",
            input = {
                topic = topic,
                message = message,
                timestamp = os.time(),
                publisher = Session.getCurrent()
            }
        }).result,
        {
            message_type = "broadcast",
            topic = topic,
            tags = {"message", topic}
        }
    )
    
    Session.complete(pub_session)
    return msg_id
end

local function subscribeToTopic(topic, since_time)
    -- Find all publisher sessions for topic
    local publishers = Session.list({
        tags = {"publisher", topic}
    })
    
    local messages = {}
    
    for _, pub in ipairs(publishers) do
        local artifacts = Artifact.query({
            session_id = pub.id,
            tags = {topic},
            created_after = since_time or 0
        })
        
        for _, artifact in ipairs(artifacts) do
            local msg = Artifact.get(pub.id, artifact.id)
            if msg then
                table.insert(messages, {
                    artifact = artifact,
                    content = Tool.execute("json-processor", {
                        operation = "parse",
                        input = msg.content
                    }).result
                })
            end
        end
    end
    
    return messages
end
```

### Performance Monitoring

```lua
-- Session performance tracking
local function trackPerformance(session_id, operation_name, func)
    local start_time = os.clock()
    local start_memory = collectgarbage("count")
    
    local success, result = pcall(func)
    
    local end_time = os.clock()
    local end_memory = collectgarbage("count")
    
    -- Store performance metrics
    Artifact.store(
        session_id,
        "system_generated",
        "perf_" .. operation_name .. "_" .. os.time() .. ".json",
        Tool.execute("json-processor", {
            operation = "stringify",
            input = {
                operation = operation_name,
                success = success,
                duration_seconds = end_time - start_time,
                memory_delta_kb = end_memory - start_memory,
                timestamp = os.time()
            }
        }).result,
        {
            metric_type = "performance",
            operation = operation_name,
            tags = {"metrics", "performance"}
        }
    )
    
    if not success then
        error(result)
    end
    
    return result
end
```

## Troubleshooting

### Common Issues

#### 1. Session Not Found
```lua
-- Problem: Session.get() returns nil
-- Solution: Check if session exists and is loaded
local session_id = "some-uuid"
local metadata = Session.get(session_id)

if not metadata then
    -- Try loading the session first
    local loaded = Session.load(session_id)
    if loaded then
        metadata = Session.get(session_id)
    else
        print("Session does not exist: " .. session_id)
    end
end
```

#### 2. Artifact Storage Fails
```lua
-- Problem: Artifact.store() fails with size error
-- Solution: Check size limits and compress if needed
local large_content = string.rep("data", 100000)  -- ~400KB

-- Compress before storing
local compressed = Tool.execute("compression", {
    operation = "compress",
    algorithm = "gzip",
    input = large_content
})

if compressed.success then
    Artifact.store(
        session_id,
        "user_input",
        "large_file.gz",
        compressed.result,
        {
            original_size = #large_content,
            compressed_size = #compressed.result,
            compression_ratio = #compressed.result / #large_content
        }
    )
end
```

#### 3. Session Replay Issues
```lua
-- Problem: Session replay fails
-- Solution: Ensure proper checkpointing
local function replayableOperation(session_id)
    -- Enable replay metadata
    local metadata = Session.get(session_id)
    metadata.replay_enabled = true
    
    -- Store operation context
    Artifact.store(
        session_id,
        "system_generated",
        "replay_context.json",
        Tool.execute("json-processor", {
            operation = "stringify",
            input = {
                start_time = os.time(),
                environment = {
                    -- Capture relevant environment
                },
                parameters = {
                    -- Capture input parameters
                }
            }
        }).result,
        {
            replay_metadata = true,
            tags = {"replay", "context"}
        }
    )
    
    -- Perform operations with checkpoints...
end
```

### Performance Tips

1. **Batch Operations**: Store multiple artifacts in a loop rather than one at a time
2. **Use Compression**: Enable compression for artifacts > 10KB
3. **Implement Caching**: Cache frequently accessed artifacts in memory
4. **Cleanup Regularly**: Remove old sessions and artifacts to maintain performance
5. **Monitor Metrics**: Track session counts, artifact sizes, and operation times

### Security Considerations

1. **Session Isolation**: Sessions are isolated by default - one session cannot access another's artifacts
2. **Access Control**: Implement access checks in your application logic
3. **Sensitive Data**: Don't store passwords or API keys directly in artifacts
4. **Audit Trails**: Use session metadata to track who did what and when
5. **Retention Policies**: Implement automatic cleanup for compliance

## See Also

- [Session and Artifact API Reference](./session-artifact-api.md)
- [Examples Directory](../../examples/lua/session/)
- [Configuration Guide](./configuration.md)