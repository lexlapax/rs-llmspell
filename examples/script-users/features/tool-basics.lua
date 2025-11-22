-- Recommended profile: minimal
-- Run with: llmspell -p minimal run tool-basics.lua
-- No LLM required

-- ============================================================
-- LLMSPELL FEATURES SHOWCASE  
-- ============================================================
-- Feature ID: 02 - Tool Basics v0.7.0
-- Complexity Level: INTERMEDIATE
-- Real-World Use Case: Automating file operations and data processing
-- Feature Category: Tools
--
-- Purpose: Essential tool usage patterns for common operations
-- Architecture: Synchronous Tool.execute() API with structured results
-- Key Capabilities:
--   • File operations (read, write, exists)
--   • Data encoding (Base64, JSON)
--   • Utility functions (UUID, hash, calculations)
--   • Error handling patterns
--   • Tool discovery and listing
--
-- Prerequisites: None (all tools work locally)
--
-- HOW TO RUN:
-- ./target/debug/llmspell run examples/script-users/features/tool-basics.lua
--
-- EXPECTED OUTPUT:
-- Demonstrates 6 tool categories with success indicators
-- Execution time: <3 seconds
--
-- Time to Complete: 3 seconds
-- Next Steps: See advanced-patterns/tool-integration-patterns.lua
-- ============================================================

print("=== Tool Basics - Essential Operations ===\n")

-- Helper function for tool invocation with error handling
local function use_tool(tool_name, params)
    local result = Tool.execute(tool_name, params)
    if result then
        return result
    end
    return {success = false, error = "Tool returned no result"}
end

-- 1. FILE OPERATIONS
print("1. File Operations")
print("-" .. string.rep("-", 17))

-- Write a file
local content = "# LLMSpell Test\nThis is a test file.\nCreated: " .. os.date()
local write_result = use_tool("file-operations", {
    operation = "write",
    path = "/tmp/llmspell_test.txt",
    input = content
})
print("   Write file: " .. (write_result.success ~= false and "✓" or "✗"))

-- Read it back
local read_result = use_tool("file-operations", {
    operation = "read",
    path = "/tmp/llmspell_test.txt"
})
print("   Read file: " .. (read_result.text and "✓" or "✗"))

-- Check existence
local exists_result = use_tool("file-operations", {
    operation = "exists",
    path = "/tmp/llmspell_test.txt"
})
print("   Check exists: " .. ((exists_result.success ~= false) and "✓" or "✗"))

-- 2. UUID GENERATION
print("\n2. UUID Generation")
print("-" .. string.rep("-", 17))

local uuid_v4 = use_tool("uuid-generator", {
    operation = "generate",
    version = "v4",
    format = "hyphenated"
})
print("   UUID v4: " .. (uuid_v4.result and uuid_v4.result.uuid and "✓" or "✗"))

local component_id = use_tool("uuid-generator", {
    operation = "component_id",
    prefix = "tool"
})
print("   Component ID: " .. (component_id.result and component_id.result.id and "✓" or "✗"))

-- 3. ENCODING OPERATIONS
print("\n3. Encoding Operations")
print("-" .. string.rep("-", 21))

local encode_result = use_tool("base64-encoder", {
    operation = "encode",
    input = "Hello LLMSpell"
})
print("   Base64 encode: " .. (encode_result.result and encode_result.result.output and "✓" or "✗"))

if encode_result.result and encode_result.result.output then
    local decode_result = use_tool("base64-encoder", {
        operation = "decode",
        input = encode_result.result.output
    })
    print("   Base64 decode: " .. (decode_result.result and decode_result.result.output == "Hello LLMSpell" and "✓" or "✗"))
end

-- 4. HASHING
print("\n4. Hash Calculations")
print("-" .. string.rep("-", 19))

local hash_result = use_tool("hash-calculator", {
    operation = "hash",
    algorithm = "sha256",
    input = "test data"
})
print("   SHA256 hash: " .. (hash_result.result and hash_result.result.hash and "✓" or "✗"))

-- 5. TEXT MANIPULATION
print("\n5. Text Manipulation")
print("-" .. string.rep("-", 19))

local text_result = use_tool("text-manipulator", {
    operation = "uppercase",
    input = "hello llmspell"
})
print("   Uppercase: " .. (text_result.result and text_result.result.result == "HELLO LLMSPELL" and "✓" or "✗"))

local replace_result = use_tool("text-manipulator", {
    operation = "replace",
    input = "hello world",
    options = {
        from = "world",
        to = "llmspell"
    }
})
print("   Replace: " .. (replace_result.result and replace_result.result.result == "hello llmspell" and "✓" or "✗"))

-- 6. CALCULATOR
print("\n6. Calculator")
print("-" .. string.rep("-", 12))

local calc_result = use_tool("calculator", {
    operation = "evaluate",
    input = "2 + 2 * 3"
})
print("   Calculate 2+2*3: " .. (calc_result.result and calc_result.result.result == 8 and "✓ = 8" or "✗"))

-- 7. TOOL DISCOVERY
print("\n7. Tool Discovery")
print("-" .. string.rep("-", 16))

local tools = Tool.list()
print("   Available tools: " .. #tools)
print("   Categories found:")

-- Count tool categories
local categories = {}
for _, tool in ipairs(tools) do
    if tool and tool.category then
        categories[tool.category] = true
    end
end

for category, _ in pairs(categories) do
    print("     • " .. category)
end

-- 8. ERROR HANDLING PATTERN
print("\n8. Error Handling")
print("-" .. string.rep("-", 16))

-- Intentionally cause an error
local error_result = use_tool("file-operations", {
    operation = "read",
    path = "/nonexistent/file.txt"
})

if error_result.success == false or error_result.error then
    print("   Error handling: ✓ (caught expected error)")
else
    print("   Error handling: ✗ (should have failed)")
end

print("\n=== Tool Basics Complete ===")
print("Next: Explore workflow-basics.lua for tool orchestration")