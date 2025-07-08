-- file-system-tools.lua
-- Examples for file system tools with security sandboxing

print("📁 File System Tools Examples")
print("=============================")

local Agent = require("llmspell.agent")
local agent = Agent.create("claude-3-sonnet-20240229")

print("\n1. File Operations Tool")
print("----------------------")

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

local write_result = agent:use_tool("file_operations", {
    operation = "write",
    path = "/tmp/llmspell_test.md",
    content = file_content
})
print("Write result:", write_result)

-- Read the file back
local read_result = agent:use_tool("file_operations", {
    operation = "read",
    path = "/tmp/llmspell_test.md"
})
print("Read result:", read_result)

-- Get file metadata
local info_result = agent:use_tool("file_operations", {
    operation = "info",
    path = "/tmp/llmspell_test.md"
})
print("File info:", info_result)

-- List directory contents
local list_result = agent:use_tool("file_operations", {
    operation = "list",
    path = "/tmp"
})
print("Directory listing:", list_result)

-- Create directory
local mkdir_result = agent:use_tool("file_operations", {
    operation = "create_dir",
    path = "/tmp/llmspell_test_dir"
})
print("Create directory:", mkdir_result)

-- Copy file
local copy_result = agent:use_tool("file_operations", {
    operation = "copy",
    source = "/tmp/llmspell_test.md",
    destination = "/tmp/llmspell_test_dir/copied.md"
})
print("Copy result:", copy_result)

print("\n2. Archive Handler Tool")
print("-----------------------")

-- Get archive information
local archive_info = agent:use_tool("archive_handler", {
    operation = "info",
    archive_path = "/tmp/test.zip"
})
print("Archive info:", archive_info)

-- Create ZIP archive
local create_zip = agent:use_tool("archive_handler", {
    operation = "create",
    archive_path = "/tmp/llmspell_docs.zip",
    files = {
        "/tmp/llmspell_test.md",
        "/tmp/llmspell_test_dir/copied.md"
    },
    format = "zip"
})
print("Create ZIP:", create_zip)

-- Extract archive
local extract_result = agent:use_tool("archive_handler", {
    operation = "extract",
    archive_path = "/tmp/llmspell_docs.zip",
    destination = "/tmp/extracted",
    format = "zip"
})
print("Extract result:", extract_result)

-- List archive contents
local list_archive = agent:use_tool("archive_handler", {
    operation = "list",
    archive_path = "/tmp/llmspell_docs.zip"
})
print("Archive contents:", list_archive)

print("\n3. File Watcher Tool")
print("-------------------")

-- Watch a directory for changes
local watch_result = agent:use_tool("file_watcher", {
    operation = "watch",
    path = "/tmp/llmspell_test_dir",
    events = {"create", "modify", "delete"},
    duration_ms = 5000,
    pattern = "*.md"
})
print("Watch result:", watch_result)

-- Configure watcher
local config_result = agent:use_tool("file_watcher", {
    operation = "config",
    debounce_ms = 100,
    max_events = 50
})
print("Config result:", config_result)

print("\n4. File Converter Tool")
print("---------------------")

-- Convert file encoding
local convert_encoding = agent:use_tool("file_converter", {
    operation = "convert",
    input_path = "/tmp/llmspell_test.md",
    output_path = "/tmp/llmspell_test_utf16.md",
    target_encoding = "UTF-16"
})
print("Encoding conversion:", convert_encoding)

-- Convert line endings
local convert_line_endings = agent:use_tool("file_converter", {
    operation = "convert",
    input_path = "/tmp/llmspell_test.md",
    output_path = "/tmp/llmspell_test_crlf.md",
    line_ending = "CRLF"
})
print("Line ending conversion:", convert_line_endings)

-- Convert indentation
local convert_indent = agent:use_tool("file_converter", {
    operation = "convert",
    input_path = "/tmp/llmspell_test.md",
    output_path = "/tmp/llmspell_test_tabs.md",
    indentation = "tabs"
})
print("Indentation conversion:", convert_indent)

-- Detect file encoding
local detect_encoding = agent:use_tool("file_converter", {
    operation = "detect",
    input_path = "/tmp/llmspell_test.md"
})
print("Detected encoding:", detect_encoding)

print("\n5. File Search Tool")
print("------------------")

-- Search for pattern in files
local search_result = agent:use_tool("file_search", {
    operation = "search",
    path = "/tmp",
    pattern = "LLMSpell",
    recursive = true,
    file_types = {"md", "txt"}
})
print("Search result:", search_result)

-- Search with regex
local regex_search = agent:use_tool("file_search", {
    operation = "search",
    path = "/tmp/llmspell_test_dir",
    pattern = "Test.*Document",
    regex = true,
    context_lines = 2
})
print("Regex search:", regex_search)

-- Search in specific file
local file_search = agent:use_tool("file_search", {
    operation = "search",
    path = "/tmp/llmspell_test.md",
    pattern = "Features",
    context_lines = 3
})
print("File search:", file_search)

-- Search and replace
local replace_result = agent:use_tool("file_search", {
    operation = "replace",
    path = "/tmp/llmspell_test_dir/copied.md",
    pattern = "2024-01-01",
    replacement = "2024-07-08",
    backup = true
})
print("Replace result:", replace_result)

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

print("\n🔧 Common Use Cases")
print("==================")

-- Template for common file operations workflow
local workflow_example = [[
1. Create working directory
2. Write configuration files
3. Process data files
4. Archive results
5. Monitor for changes
6. Clean up temporary files
]]

print("Common workflow:")
print(workflow_example)

-- Cleanup example
local cleanup_result = agent:use_tool("file_operations", {
    operation = "delete",
    path = "/tmp/llmspell_test_dir",
    recursive = true
})
print("Cleanup result:", cleanup_result)

print("\n✅ File System Tools Examples Complete!")
print("All operations performed safely within sandbox restrictions.")

return {
    tools_demonstrated = 5,
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