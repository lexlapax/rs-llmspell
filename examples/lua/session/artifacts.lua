-- ABOUTME: Example demonstrating artifact storage and retrieval within sessions
-- ABOUTME: Shows text, JSON, binary storage, metadata handling, and compression

-- CONFIG: Requires runtime integration (see README.md for current status)
-- WHY: Artifacts store conversation outputs, tool results, and generated content
-- STATUS: Session/Artifact globals implemented but not yet integrated into CLI runtime
-- TODO: Runtime needs to initialize SessionManager - see llmspell-bridge/src/runtime.rs

print("📦 Artifact Management Example")
print("==============================")

-- This example demonstrates:
-- 1. Storing different artifact types (text, JSON, binary)
-- 2. Using metadata and tags for organization
-- 3. Content-addressed storage with BLAKE3
-- 4. Binary data handling
-- 5. Automatic compression for large artifacts
-- 6. Thread-local session context
-- 7. File storage from disk
-- 8. Artifact deletion

-- Helper to create sample binary data
local function create_sample_image_data()
    -- Simulate a small PNG header and some data
    local png_header = string.char(0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A)
    local data = png_header .. string.rep("X", 1000) -- Simulated image data
    return data
end

-- Step 1: Create a session for our artifacts
print("\n1. Session Setup")
print(string.rep("-", 40))
local session_id = Session.create({
    name = "Artifact Demo Session",
    description = "Demonstrating artifact storage capabilities",
    tags = {"demo", "artifacts"}
})
print("✅ Session created:", session_id)

-- Set as current session for convenience
Session.setCurrent(session_id)
print("🎯 Current session set")

-- Step 2: Store a simple text artifact
print("\n2. Text Artifact Storage")
print(string.rep("-", 40))
local text_id = Artifact.store(
    session_id,
    "tool_result",
    "analysis_output.txt",
    "Sales Analysis Results\n\nQ4 2024 showed 15% growth compared to Q3.\nTop performing regions: EMEA, APAC",
    {
        mime_type = "text/plain",
        tags = {"analysis", "q4-2024"},
        tool = "sales_analyzer",
        execution_time = 2.5
    }
)
print("✅ Stored text artifact")
print("  Content hash:", text_id.content_hash:sub(1, 16) .. "...")
print("  Sequence:", text_id.sequence)

-- Step 3: Store JSON data
print("\n3. JSON Artifact Storage")
print(string.rep("-", 40))
local json_data = {
    metrics = {
        revenue = 1500000,
        growth = 0.15,
        regions = {"EMEA", "APAC", "AMERICAS"}
    },
    timestamp = os.date("!%Y-%m-%dT%H:%M:%SZ")
}
local json_id = Artifact.store(
    session_id,
    "agent_output",
    "metrics.json",
    JSON.stringify(json_data),
    {
        mime_type = "application/json",
        tags = {"metrics", "json"},
        version = "1.0"
    }
)
print("✅ Stored JSON artifact")
print("  Content hash:", json_id.content_hash:sub(1, 16) .. "...")

-- Step 4: Store binary data (simulated image)
print("\n4. Binary Artifact Storage")
print(string.rep("-", 40))
local image_data = create_sample_image_data()
local image_id = Artifact.store(
    session_id,
    "system_generated",
    "chart.png",
    image_data,
    {
        mime_type = "image/png",
        tags = {"visualization", "chart"},
        description = "Q4 2024 sales chart"
    }
)
print("✅ Stored binary artifact")
print("  Content hash:", image_id.content_hash:sub(1, 16) .. "...")
print("  Binary size:", #image_data, "bytes")

-- Step 5: List all artifacts in the session
print("\n5. Artifact Listing")
print(string.rep("-", 40))
local artifacts = Artifact.list(session_id)
print(string.format("Found %d artifacts:", #artifacts))
for i, artifact in ipairs(artifacts) do
    print(string.format("  %d. %s (%s, %d bytes)", 
        i, 
        artifact.name, 
        artifact.artifact_type,
        artifact.size
    ))
end

-- Step 6: Retrieve and verify artifacts
print("\n6. Artifact Retrieval and Verification")
print(string.rep("-", 40))

-- Get text artifact
local text_artifact = Artifact.get(session_id, text_id)
print("\nText artifact:")
print("  Name:", text_artifact.metadata.name)
print("  Type:", text_artifact.metadata.artifact_type)
print("  MIME:", text_artifact.metadata.mime_type)
print("  Content preview:", text_artifact.content:sub(1, 50) .. "...")

-- Get JSON artifact and decode
local json_artifact = Artifact.get(session_id, json_id)
local decoded = JSON.parse(json_artifact.content)
print("\nJSON artifact:")
print("  Revenue:", decoded.metrics.revenue)
print("  Growth:", decoded.metrics.growth * 100 .. "%")
print("  Regions:", table.concat(decoded.metrics.regions, ", "))

-- Get binary artifact
local binary_artifact = Artifact.get(session_id, image_id)
print("\nBinary artifact:")
print("  Name:", binary_artifact.metadata.name)
print("  Size:", binary_artifact.metadata.size, "bytes")
print("  First 8 bytes (PNG header):", 
    string.format("%02X %02X %02X %02X %02X %02X %02X %02X",
        binary_artifact.content:byte(1, 8)
    )
)

-- Step 7: Using current session context
print("\n7. Thread-Local Session Context")
print(string.rep("-", 40))
local current_artifacts = Artifact.list("")  -- Empty string uses current session
print("Artifacts in current session:", #current_artifacts)

-- Step 8: Store a large artifact (tests compression)
print("\n8. Large Artifact Compression")
print(string.rep("-", 40))
local large_content = string.rep("This is test data for compression. ", 1000)
print("Original size:", #large_content, "bytes")

local large_id = Artifact.store(
    session_id,
    "system_generated",
    "large_file.txt",
    large_content,
    {
        mime_type = "text/plain",
        tags = {"large", "test"}
    }
)

local large_artifact = Artifact.get(session_id, large_id)
print("Stored size:", large_artifact.metadata.size, "bytes")
print("Content matches:", large_artifact.content == large_content)
-- Note: Compression happens transparently at storage layer

-- Step 9: Store file directly from disk
print("\n9. File Storage from Disk")
print(string.rep("-", 40))
-- First create a test file
local test_file = "/tmp/test_artifact.txt"
local file = io.open(test_file, "w")
file:write("This is content from a file on disk.")
file:close()

local file_id = Artifact.storeFile(
    session_id,
    test_file,
    "user_input",
    {
        source = "filesystem",
        original_path = test_file
    }
)
print("Stored file artifact:", file_id.content_hash)

-- Clean up test file
os.remove(test_file)

-- Step 10: Delete an artifact
print("\n10. Artifact Deletion")
print(string.rep("-", 40))
Artifact.delete(session_id, text_id)
print("Deleted text artifact")

-- Verify it's gone
local success, err = pcall(Artifact.get, session_id, text_id)
if not success then
    print("Confirmed deletion:", tostring(err))
end

-- List remaining artifacts
local remaining = Artifact.list(session_id)
print(string.format("\nRemaining artifacts: %d", #remaining))

-- Clean up
Session.complete(session_id)

-- Summary
print("\n\n🎉 Artifact Management Completed!")
print("=================================")
print("\nDemonstrated capabilities:")
print("  ✓ Text, JSON, and binary artifact storage")
print("  ✓ Rich metadata and tagging support")
print("  ✓ Content-addressed storage (BLAKE3)")
print("  ✓ Automatic compression for large artifacts")
print("  ✓ Thread-local session context usage")
print("  ✓ Direct file storage from disk")
print("  ✓ Artifact listing and deletion")
print("\nKey takeaways:")
print("  • Artifacts use content-addressed storage (deduplication)")
print("  • Binary data is handled transparently as Lua strings")
print("  • Large artifacts are compressed automatically (>10KB)")
print("  • Metadata enables rich querying and organization")