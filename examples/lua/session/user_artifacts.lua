-- ABOUTME: Example demonstrating storing user files and datasets as artifacts
-- ABOUTME: Shows how to use the Artifact API for managing user-provided content

-- CONFIG: Requires session-enabled configuration (see examples/configs/session-enabled.toml)
-- WHY: Artifacts provide content-addressed storage for user data with rich metadata
-- STATUS: Session/Artifact globals fully integrated and functional

print("=== User Artifacts Example ===\n")

-- This example demonstrates:
-- 1. Storing various types of user content (documents, datasets, configs)
-- 2. Adding rich metadata for organization and search
-- 3. Storing files from disk using storeFile()
-- 4. Querying artifacts by type, tags, and time
-- 5. Building searchable indexes
-- 6. Managing binary/media content metadata

-- Create a session for our knowledge base
local session_id = Session.create({
    name = "User Data Session",
    description = "Example of storing user files and datasets",
    tags = {"user-data", "file-storage", "example"}
})

print("Created session: " .. session_id)
Session.setCurrent(session_id)

-- Example 1: Store a user document
print("\n1. Storing user documents:")

local doc_content = [[
# Project Documentation

## Overview
This is an example user document that might be uploaded by a user.
It contains important project information that needs to be preserved.

## Key Features
- Feature 1: Advanced data processing
- Feature 2: Real-time analytics
- Feature 3: Multi-format support

## Technical Details
The system uses state-of-the-art algorithms for processing.
All data is encrypted and stored securely.
]]

local doc_id = Artifact.store(
    session_id,
    "user_input",
    "project_documentation.md",
    doc_content,
    {
        author = "john_doe",
        category = "documentation",
        tags = {"project", "technical", "markdown"},
        version = "1.0",
        upload_timestamp = os.time()
    }
)

print("  Stored document: " .. doc_id.content_hash:sub(1, 16) .. "...")

-- Example 2: Store a dataset (CSV format)
print("\n2. Storing datasets:")

local csv_data = [[
timestamp,temperature,humidity,location
2024-01-15T10:00:00,22.5,65,Room A
2024-01-15T10:15:00,22.7,64,Room A
2024-01-15T10:30:00,23.1,63,Room A
2024-01-15T10:45:00,23.3,62,Room A
2024-01-15T11:00:00,23.5,61,Room A
]]

local dataset_id = Artifact.store(
    session_id,
    "user_input",
    "sensor_data.csv",
    csv_data,
    {
        dataset_type = "timeseries",
        format = "csv",
        rows = 5,
        columns = 4,
        tags = {"sensor", "temperature", "humidity"},
        collection_date = "2024-01-15"
    }
)

print("  Stored dataset: " .. dataset_id.content_hash:sub(1, 16) .. "...")

-- Example 3: Store JSON configuration
print("\n3. Storing configuration files:")

local config_data = {
    application = {
        name = "DataProcessor",
        version = "2.1.0",
        environment = "production"
    },
    database = {
        host = "db.example.com",
        port = 5432,
        name = "analytics_db"
    },
    features = {
        real_time_processing = true,
        batch_size = 1000,
        retry_attempts = 3
    }
}

-- Simple JSON encoding
local function encode_json(data)
    local function encode_value(v)
        if type(v) == "string" then
            return '"' .. v .. '"'
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

local config_json = {result = encode_json(config_data)}

local config_id = Artifact.store(
    session_id,
    "user_input",
    "app_config.json",
    config_json.result,
    {
        config_type = "application",
        environment = "production",
        tags = {"config", "json", "production"},
        validated = true
    }
)

print("  Stored configuration: " .. config_id.content_hash:sub(1, 16) .. "...")

-- Example 4: Store file from disk (if available)
print("\n4. Storing files from disk:")

-- Create a test file
local test_file_path = "/tmp/llmspell_test_upload.txt"
local file_op = Tool.invoke("file_operations", {
    operation = "write",
    path = test_file_path,
    input = "This is a test file that simulates user upload.\nIt contains multiple lines of text.\nThis would typically be uploaded through a UI."
})

if file_op.success then
    local file_artifact_id = Artifact.storeFile(
        session_id,
        test_file_path,
        "user_input",
        {
            original_filename = "user_upload.txt",
            upload_method = "cli",
            user = "test_user",
            tags = {"upload", "text", "user-file"}
        }
    )
    print("  Stored file from disk: " .. file_artifact_id.content_hash:sub(1, 16) .. "...")
    
    -- Clean up
    Tool.invoke("file_operations", {
        operation = "delete",
        path = test_file_path
    })
else
    print("  (Skipping file upload - could not create test file)")
end

-- Example 5: Store binary data (simulated image metadata)
print("\n5. Storing binary/media artifacts:")

-- Simulate image metadata (in real use, this would be actual image data)
local image_metadata = {
    filename = "screenshot.png",
    width = 1920,
    height = 1080,
    format = "PNG",
    size_bytes = 2458624,
    color_space = "sRGB",
    dpi = 72,
    thumbnail = "base64_encoded_thumbnail_here"
}

local metadata_json = {result = encode_json(image_metadata)}

local image_id = Artifact.store(
    session_id,
    "user_input",
    "screenshot_metadata.json",
    metadata_json.result,
    {
        media_type = "image",
        actual_file = "screenshot.png",
        tags = {"image", "screenshot", "metadata"},
        dimensions = "1920x1080"
    }
)

print("  Stored image metadata: " .. image_id.content_hash:sub(1, 16) .. "...")

-- Example 6: Query and retrieve user artifacts
print("\n6. Querying user artifacts:")

-- Query all user input artifacts
local user_artifacts = Artifact.query({
    session_id = session_id,
    type = "user_input"
})

print("  Found " .. #user_artifacts .. " user artifacts")

-- Query by tags
local doc_artifacts = Artifact.query({
    session_id = session_id,
    tags = {"documentation"}
})

print("  Found " .. #doc_artifacts .. " documentation artifacts")

-- Query recent uploads
local recent_artifacts = Artifact.query({
    session_id = session_id,
    created_after = os.time() - 3600, -- Last hour
    limit = 5
})

print("  Found " .. #recent_artifacts .. " recent artifacts")

-- Example 7: Retrieve and display artifact content
print("\n7. Retrieving artifact content:")

local retrieved = Artifact.get(session_id, doc_id)
if retrieved then
    print("  Retrieved document '" .. retrieved.metadata.name .. "'")
    print("  Size: " .. #retrieved.content .. " bytes")
    print("  First line: " .. retrieved.content:match("^[^\n]+"))
end

-- Example 8: Building a searchable index
print("\n8. Building artifact index:")

local all_artifacts = Artifact.list(session_id)
local index = {}

for _, artifact in ipairs(all_artifacts) do
    -- Index by type
    if not index[artifact.artifact_type] then
        index[artifact.artifact_type] = {}
    end
    table.insert(index[artifact.artifact_type], artifact)
    
    -- Index by tags
    if artifact.metadata and artifact.metadata.tags then
        for _, tag in ipairs(artifact.metadata.tags) do
            if not index[tag] then
                index[tag] = {}
            end
            table.insert(index[tag], artifact)
        end
    end
end

print("  Indexed " .. #all_artifacts .. " artifacts")
for key, items in pairs(index) do
    print("    " .. key .. ": " .. #items .. " items")
end

-- Complete the session
Session.complete(session_id)
print("\n✓ User artifacts example completed!")
print("  Session " .. session_id .. " contains " .. #all_artifacts .. " artifacts")

-- Summary of capabilities demonstrated:
print("\n=== Summary ===")
print("This example demonstrated:")
print("  • Storing various types of user content (documents, datasets, configs)")
print("  • Adding rich metadata for organization and search")
print("  • Storing files from disk using storeFile()")
print("  • Querying artifacts by type, tags, and time")
print("  • Building searchable indexes")
print("  • Managing binary/media content metadata")
print("\nUse these patterns to build knowledge bases, document stores,")
print("and data management systems with the Artifact API.")