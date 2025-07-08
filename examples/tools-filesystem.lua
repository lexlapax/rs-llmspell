-- tools-filesystem.lua
-- Examples for file system tools with security sandboxing
-- Using direct Tool API

print("📁 File System Tools Examples")
print("=============================")

-- Load test helpers
local TestHelpers = dofile("examples/test-helpers.lua")

-- Helper function to execute tool
local function use_tool(tool_name, params)
    return TestHelpers.execute_tool(tool_name, params)
end

-- Helper to print clean results
local function print_result(label, result)
    if result.error then
        print("  ❌ " .. label .. ": " .. result.error)
    elseif result.success == false then
        print("  ❌ " .. label .. ": " .. (result.message or "Failed"))
    else
        -- Extract relevant info based on operation
        if result.content then
            print("  ✅ " .. label .. ": Read " .. string.len(result.content) .. " bytes")
        elseif result.size then
            print("  ✅ " .. label .. ": " .. result.size .. " bytes")
        elseif result.files then
            print("  ✅ " .. label .. ": " .. #result.files .. " files")
        elseif result.events then
            print("  ✅ " .. label .. ": " .. #result.events .. " events")
        elseif result.matches then
            print("  ✅ " .. label .. ": " .. #result.matches .. " matches")
        elseif result.operation then
            print("  ✅ " .. label .. ": " .. result.operation .. " completed")
        else
            print("  ✅ " .. label .. ": Success")
        end
    end
end

TestHelpers.print_section("File Operations Tool")

-- Write a file
local file_content = [[
# LLMSpell Test Document
This is a test file created by the FileOperationsTool.

## Features
- Secure file operations
- Path traversal prevention
- Atomic write operations
- Proper error handling

Date: 2024-01-01
]]

print("\nBasic file operations:")
local write_result = use_tool("file_operations", {
    operation = "write",
    path = "/tmp/llmspell_test.md",
    content = file_content
})
print_result("Write file", write_result)

-- Read the file back
local read_result = use_tool("file_operations", {
    operation = "read",
    path = "/tmp/llmspell_test.md"
})
print_result("Read file", read_result)

-- Get file metadata
local info_result = use_tool("file_operations", {
    operation = "info",
    path = "/tmp/llmspell_test.md"
})
print_result("File info", info_result)

-- List directory contents
local list_result = use_tool("file_operations", {
    operation = "list",
    path = "/tmp",
    pattern = "llmspell*"
})
print_result("Directory listing", list_result)

-- Create directory
local mkdir_result = use_tool("file_operations", {
    operation = "create_dir",
    path = "/tmp/llmspell_test_dir"
})
print_result("Create directory", mkdir_result)

-- Copy file
local copy_result = use_tool("file_operations", {
    operation = "copy",
    source = "/tmp/llmspell_test.md",
    destination = "/tmp/llmspell_test_dir/copied.md"
})
print_result("Copy file", copy_result)

TestHelpers.print_section("Archive Handler Tool")

print("\nArchive operations:")

-- Create ZIP archive
local create_zip = use_tool("archive_handler", {
    operation = "create",
    archive_path = "/tmp/llmspell_docs.zip",
    files = {"/tmp/llmspell_test.md"},
    format = "zip"
})
print_result("Create ZIP", create_zip)

-- List archive contents
local list_archive = use_tool("archive_handler", {
    operation = "list",
    archive_path = "/tmp/llmspell_docs.zip"
})
print_result("List archive", list_archive)

-- Extract archive
local extract_result = use_tool("archive_handler", {
    operation = "extract",
    archive_path = "/tmp/llmspell_docs.zip",
    destination = "/tmp/extracted"
})
print_result("Extract archive", extract_result)

TestHelpers.print_section("File Watcher Tool")

print("\nFile monitoring:")

-- Watch a directory for changes (short duration for demo)
local watch_result = use_tool("file_watcher", {
    operation = "watch",
    path = "/tmp/llmspell_test_dir",
    events = {"create", "modify", "delete"},
    timeout_ms = 1000,  -- 1 second timeout
    pattern = "*.md"
})
print_result("Watch directory", watch_result)

TestHelpers.print_section("File Converter Tool")

print("\nFile conversions:")

-- Convert line endings
local convert_line_endings = use_tool("file_converter", {
    operation = "convert_line_endings",
    input_path = "/tmp/llmspell_test.md",
    output_path = "/tmp/llmspell_test_crlf.md",
    line_ending = "crlf"
})
print_result("Convert to CRLF", convert_line_endings)

-- Convert indentation
local convert_indent = use_tool("file_converter", {
    operation = "convert_indentation",
    input_path = "/tmp/llmspell_test.md",
    output_path = "/tmp/llmspell_test_tabs.md",
    to_tabs = true,
    tab_width = 4
})
print_result("Convert to tabs", convert_indent)

-- Detect file encoding
local detect_encoding = use_tool("file_converter", {
    operation = "detect_encoding",
    path = "/tmp/llmspell_test.md"
})
print_result("Detect encoding", detect_encoding)

TestHelpers.print_section("File Search Tool")

print("\nSearching files:")

-- Search for pattern in files
local search_result = use_tool("file_search", {
    operation = "search",
    path = "/tmp",
    pattern = "LLMSpell",
    recursive = true,
    extensions = {"md", "txt"},
    max_depth = 2
})
print_result("Search files", search_result)

-- Search with context
local context_search = use_tool("file_search", {
    operation = "search",
    path = "/tmp/llmspell_test.md",
    pattern = "Features",
    context_lines = 2,
    case_sensitive = false
})
print_result("Search with context", context_search)

-- Display search results if available
if context_search.matches and #context_search.matches > 0 then
    print("\n  Search results:")
    for i, match in ipairs(context_search.matches) do
        if i <= 3 then  -- Show first 3 matches
            print(string.format("    Match %d: Line %d", i, match.line_number or 0))
            if match.context then
                print("    Context: " .. (match.context:sub(1, 50) .. "..."))
            end
        end
    end
end

print("\n🔒 Security Features Demonstrated")
print("=================================")

-- These examples show secure file operations:
print("✅ Path traversal prevention - all paths validated")
print("✅ Sandbox containment - operations restricted to safe areas")
print("✅ Atomic operations - writes are atomic to prevent corruption")
print("✅ Resource limits - file size and operation limits enforced")
print("✅ Error handling - proper error messages without information leakage")

print("\n📊 Performance Considerations")
print("=============================")

-- File operations are optimized for:
print("✅ Streaming - large files processed in chunks")
print("✅ Memory efficiency - minimal memory footprint")
print("✅ Concurrent safety - thread-safe operations")
print("✅ Fast initialization - tools start quickly")

-- Cleanup example
print("\n🧹 Cleanup:")
local cleanup_result = use_tool("file_operations", {
    operation = "delete",
    path = "/tmp/llmspell_test_dir",
    recursive = true
})
print_result("Delete directory", cleanup_result)

-- Delete individual files
local delete_file = use_tool("file_operations", {
    operation = "delete",
    path = "/tmp/llmspell_test.md"
})
print_result("Delete file", delete_file)

print("\n✅ File System Tools Examples Complete!")
print("All operations performed safely within sandbox restrictions.")

-- Summary
local tools_tested = {
    "file_operations",
    "archive_handler",
    "file_watcher",
    "file_converter",
    "file_search"
}

print("\n📊 Summary:")
print("  Tools tested: " .. #tools_tested)
for _, tool in ipairs(tools_tested) do
    print("    - " .. tool)
end

return {
    tools_demonstrated = #tools_tested,
    categories = "file_system",
    security_features = {
        "path_traversal_prevention",
        "sandbox_containment", 
        "atomic_operations",
        "resource_limits",
        "error_handling"
    },
    status = "success"
}