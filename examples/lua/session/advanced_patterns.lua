-- ABOUTME: Example demonstrating advanced session and artifact patterns
-- ABOUTME: Shows hierarchies, bulk operations, caching, and enterprise patterns

-- CONFIG: Requires session-enabled configuration (see examples/configs/session-enabled.toml)
-- WHY: Enterprise applications need advanced patterns for scalability and management
-- STATUS: Session/Artifact globals fully integrated and functional

print("=== Advanced Session Patterns Example ===\n")

-- This example demonstrates:
-- 1. Session hierarchies and templates
-- 2. Cross-session artifact sharing
-- 3. Bulk operations and batch processing
-- 4. Advanced querying and analytics
-- 5. State synchronization between sessions
-- 6. Performance optimization (caching)
-- 7. Resource management and cleanup
-- 8. Session hierarchy export

-- Simple JSON encoding function
local function encode_json(data)
    local function encode_value(v)
        if type(v) == "string" then
            return '"' .. v:gsub('"', '\\"') .. '"'
        elseif type(v) == "number" then
            return tostring(v)
        elseif type(v) == "boolean" then
            return tostring(v)
        elseif type(v) == "table" then
            local is_array = #v > 0
            local parts = {}
            if is_array then
                for i, item in ipairs(v) do
                    table.insert(parts, encode_value(item))
                end
                return "[" .. table.concat(parts, ",") .. "]"
            else
                for k, item in pairs(v) do
                    table.insert(parts, '"' .. k .. '":' .. encode_value(item))
                end
                return "{" .. table.concat(parts, ",") .. "}"
            end
        end
        return "null"
    end
    return encode_value(data)
end

-- 1. Session Hierarchies and Templates
print("1. Creating session hierarchies:")

-- Create a master session template
local master_template = {
    name_prefix = "Project-",
    tags = {"project", "managed"},
    metadata = {
        template_version = "1.0",
        organization = "Engineering",
        compliance = "SOC2"
    },
    config = {
        auto_save_interval = 300,  -- 5 minutes
        max_artifacts = 500,
        retention_days = 90
    }
}

-- Function to create project session from template
local function createProjectSession(project_name, owner)
    local session_config = {
        name = master_template.name_prefix .. project_name,
        description = "Project session for " .. project_name,
        tags = master_template.tags,
        metadata = {}
    }
    
    -- Merge metadata
    for k, v in pairs(master_template.metadata) do
        session_config.metadata[k] = v
    end
    session_config.metadata.project_name = project_name
    session_config.metadata.owner = owner
    session_config.metadata.created_from_template = true
    
    return Session.create(session_config)
end

-- Create parent project session
local parent_project = createProjectSession("DataAnalytics", "john.doe@company.com")
print("  Created parent project: " .. parent_project)

-- Create child sessions for different components
local sessions = {
    etl = Session.create({
        name = "ETL Pipeline",
        parent_session_id = parent_project,
        tags = {"etl", "data-processing"}
    }),
    ml = Session.create({
        name = "ML Training",
        parent_session_id = parent_project,
        tags = {"ml", "training"}
    }),
    reporting = Session.create({
        name = "Reporting Module",
        parent_session_id = parent_project,
        tags = {"reporting", "visualization"}
    })
}

print("  Created child sessions:")
for name, id in pairs(sessions) do
    print("    - " .. name .. ": " .. id)
end

-- 2. Cross-Session Artifact Sharing
print("\n2. Cross-session artifact sharing:")

-- Store shared configuration in parent session
Session.setCurrent(parent_project)
local shared_config = Artifact.store(
    parent_project,
    "system_generated",
    "shared_config.json",
    encode_json({
        database = {
            host = "analytics.db.internal",
            port = 5432,
            name = "analytics_prod"
        },
        api = {
            endpoint = "https://api.analytics.internal",
            version = "v2"
        },
        features = {
            real_time = true,
            batch_size = 1000
        }
    }),
    {
        scope = "shared",
        access = "read-only",
        tags = {"config", "shared"}
    }
)

-- Reference shared artifact in child sessions
for name, session_id in pairs(sessions) do
    Session.setCurrent(session_id)
    
    -- Store reference to parent artifact
    Artifact.store(
        session_id,
        "system_generated",
        "config_reference.json",
        encode_json({
            reference_type = "parent_artifact",
            parent_session = parent_project,
            artifact_id = shared_config,
            purpose = "shared_configuration"
        }),
        {
            reference = true,
            parent_artifact = shared_config.content_hash
        }
    )
end

-- 3. Bulk Operations and Batch Processing
print("\n3. Bulk operations:")

Session.setCurrent(sessions.etl)

-- Batch store multiple data files
local batch_data = {}
for i = 1, 20 do
    batch_data[i] = {
        filename = string.format("data_batch_%03d.csv", i),
        content = string.format("timestamp,value\n%d,%f", os.time() + i, math.random() * 100),
        metadata = {
            batch_number = i,
            record_count = math.random(900, 1100),
            processing_time = math.random() * 2
        }
    }
end

-- Store artifacts in batch
local batch_start = os.clock()
local batch_artifacts = {}

for _, data in ipairs(batch_data) do
    local artifact_id = Artifact.store(
        sessions.etl,
        "tool_result",
        data.filename,
        data.content,
        data.metadata
    )
    table.insert(batch_artifacts, artifact_id)
end

local batch_time = os.clock() - batch_start
print(string.format("  Stored %d artifacts in %.3f seconds (%.1f/sec)", 
    #batch_artifacts, batch_time, #batch_artifacts / batch_time))

-- 4. Advanced Querying and Analytics
print("\n4. Advanced artifact analytics:")

-- Complex query with multiple filters
local recent_large_artifacts = Artifact.query({
    session_id = sessions.etl,
    type = "tool_result",
    created_after = os.time() - 3600,  -- Last hour
    min_size = 50,  -- At least 50 bytes
    limit = 10
})

print("  Recent large artifacts: " .. #recent_large_artifacts)

-- Analyze artifact patterns
local artifact_stats = {
    by_hour = {},
    by_size_range = {
        small = 0,    -- < 1KB
        medium = 0,   -- 1KB - 10KB
        large = 0     -- > 10KB
    },
    total_size = 0,
    compression_ratio = 0
}

local all_artifacts = Artifact.list(sessions.etl)
for _, artifact in ipairs(all_artifacts) do
    -- Group by hour (extract from ISO string)
    local hour = artifact.created_at:match("T(%d%d):") or "00"
    artifact_stats.by_hour[hour] = (artifact_stats.by_hour[hour] or 0) + 1
    
    -- Categorize by size
    if artifact.size < 1024 then
        artifact_stats.by_size_range.small = artifact_stats.by_size_range.small + 1
    elseif artifact.size < 10240 then
        artifact_stats.by_size_range.medium = artifact_stats.by_size_range.medium + 1
    else
        artifact_stats.by_size_range.large = artifact_stats.by_size_range.large + 1
    end
    
    artifact_stats.total_size = artifact_stats.total_size + artifact.size
end

print("\n  Artifact statistics:")
print("    Total size: " .. string.format("%.2f KB", artifact_stats.total_size / 1024))
print("    Size distribution:")
for range, count in pairs(artifact_stats.by_size_range) do
    print(string.format("      %s: %d", range, count))
end

-- 5. Session State Synchronization
print("\n5. Session state synchronization:")

-- Function to sync state between sessions
local function syncSessionState(from_session, to_session, keys)
    Session.setCurrent(from_session)
    local sync_data = {}
    
    -- Collect state from source session
    for _, key in ipairs(keys) do
        sync_data[key] = State.get(key)
    end
    
    -- Apply to target session
    Session.setCurrent(to_session)
    for key, value in pairs(sync_data) do
        State.set(key, value)
    end
    
    -- Create sync record
    Artifact.store(
        to_session,
        "system_generated",
        "state_sync.json",
        encode_json({
            sync_time = os.time(),
            from_session = from_session,
            to_session = to_session,
            keys_synced = keys,
            data = sync_data
        }),
        {
            operation = "state_sync",
            tags = {"sync", "state"}
        }
    )
    
    return sync_data
end

-- Set some state in ETL session
Session.setCurrent(sessions.etl)
State.set("processing_status", "completed")
State.set("records_processed", 10000)
State.set("last_error", nil)

-- Sync to reporting session
local synced = syncSessionState(
    sessions.etl,
    sessions.reporting,
    {"processing_status", "records_processed"}
)

print("  Synced state to reporting session:")
for k, v in pairs(synced) do
    print("    " .. k .. " = " .. tostring(v))
end

-- 6. Performance Optimization Patterns
print("\n6. Performance optimization:")

-- Artifact caching pattern
local artifact_cache = {}
local cache_hits = 0
local cache_misses = 0

local function getCachedArtifact(session_id, artifact_id)
    local cache_key = session_id .. ":" .. artifact_id.content_hash
    
    if artifact_cache[cache_key] then
        cache_hits = cache_hits + 1
        return artifact_cache[cache_key]
    else
        cache_misses = cache_misses + 1
        local artifact = Artifact.get(session_id, artifact_id)
        if artifact then
            artifact_cache[cache_key] = artifact
        end
        return artifact
    end
end

-- Test cache performance
for i = 1, 10 do
    -- Access same artifact multiple times
    local artifact_id = batch_artifacts[1]
    getCachedArtifact(sessions.etl, artifact_id)
end

print(string.format("  Cache performance: %d hits, %d misses (%.1f%% hit rate)",
    cache_hits, cache_misses, (cache_hits / (cache_hits + cache_misses)) * 100))

-- 7. Cleanup and Resource Management
print("\n7. Resource management:")

-- Function to clean up old artifacts
local function cleanupOldArtifacts(session_id, days_old)
    local cutoff_time = os.time() - (days_old * 24 * 60 * 60)
    local artifacts = Artifact.list(session_id)
    local deleted_count = 0
    local freed_space = 0
    
    for _, artifact in ipairs(artifacts) do
        if artifact.created_at < cutoff_time then
            -- Only delete if not referenced by other sessions
            if not artifact.metadata.reference then
                Artifact.delete(session_id, artifact.id)
                deleted_count = deleted_count + 1
                freed_space = freed_space + artifact.size
            end
        end
    end
    
    return deleted_count, freed_space
end

-- Simulate cleanup (we won't actually delete our test data)
print("  Cleanup simulation: Would delete artifacts older than 30 days")

-- 8. Export session hierarchy
print("\n8. Exporting session hierarchy:")

-- Build hierarchy structure
local hierarchy = {
    root = {
        id = parent_project,
        metadata = Session.get(parent_project),
        children = {}
    }
}

for name, session_id in pairs(sessions) do
    local session_data = {
        id = session_id,
        name = name,
        metadata = Session.get(session_id),
        artifact_count = #Artifact.list(session_id),
        state_keys = State.list()
    }
    table.insert(hierarchy.root.children, session_data)
end

-- Save hierarchy
Session.setCurrent(parent_project)
Artifact.store(
    parent_project,
    "system_generated",
    "session_hierarchy.json",
    encode_json(hierarchy),
    {
        export_type = "hierarchy",
        timestamp = os.time(),
        tags = {"export", "hierarchy"}
    }
)

print("  Exported session hierarchy")

-- Complete all sessions
Session.save(parent_project)
for _, session_id in pairs(sessions) do
    Session.save(session_id)
end

print("\n✓ Advanced patterns example completed!")

-- Summary
print("\n=== Summary ===")
print("This example demonstrated advanced patterns:")
print("  • Session hierarchies and templates")
print("  • Cross-session artifact sharing")
print("  • Bulk operations and batch processing")
print("  • Advanced querying and analytics")
print("  • State synchronization between sessions")
print("  • Performance optimization (caching)")
print("  • Resource management and cleanup")
print("  • Session hierarchy export")
print("\nUse these patterns for:")
print("  • Large-scale data processing")
print("  • Complex multi-component systems")
print("  • Performance-critical applications")
print("  • Enterprise session management")