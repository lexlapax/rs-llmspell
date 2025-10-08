-- ============================================================
-- LLMSPELL GETTING STARTED SHOWCASE
-- ============================================================
-- Example ID: 01 - First Tool v0.7.0
-- Complexity Level: BEGINNER
-- Real-World Use Case: File management and basic I/O operations
--
-- Purpose: Learn fundamental tool invocation patterns with file-operations tool.
--          Demonstrates how to call tools, pass parameters, and handle results.
--          This is the foundation for all tool-based automation in LLMSpell.
-- Architecture: Synchronous tool invocation via Tool.execute()
-- Crates Showcased: llmspell-tools (file-operations), llmspell-bridge
-- Key Features:
--   • Tool invocation syntax
--   • Parameter passing patterns
--   • Result handling (success and error cases)
--   • File creation, reading, and existence checking
--
-- Prerequisites:
--   • LLMSpell installed and built
--   • Write access to /tmp directory
--   • No API keys required
--
-- HOW TO RUN:
-- ./target/debug/llmspell run examples/script-users/getting-started/01-first-tool.lua
--
-- EXPECTED OUTPUT:
-- File created: /tmp/my_first_file.txt (50 bytes written)
-- File content: "Hello from LLMSpell! This is my first tool usage."
-- File exists: true
-- All operations completed successfully
--
-- Time to Complete: <5 seconds
-- ============================================================

print("=== LLMSpell: Your First Tool ===")
print("Example 01: BEGINNER - Learning tool invocation patterns")
print("Showcasing: file-operations tool for basic I/O\n")

-- LLMSpell provides 34+ tools out of the box
-- We'll start with file-operations for create, read, and check operations

print("1. Creating a test file...")
local result = Tool.execute("file-operations", {
    operation = "write",
    path = "/tmp/my_first_file.txt",
    input = "Hello from LLMSpell! This is my first tool usage."
})

if result.text then
    print("✅ File created successfully!")
    print("   Result: " .. result.text)
else
    print("❌ Error creating file: " .. (result.error or "Unknown error"))
    return
end

print()
print("2. Reading the file back...")
local read_result = Tool.execute("file-operations", {
    operation = "read",
    path = "/tmp/my_first_file.txt"
})

if read_result.text then
    print("✅ File content read successfully!")
    print("   Content: " .. read_result.text)
else
    print("❌ Error reading file: " .. (read_result.error or "Unknown error"))
end

print()
print("3. Checking if file exists...")
local exists_result = Tool.execute("file-operations", {
    operation = "exists",
    path = "/tmp/my_first_file.txt"
})

if exists_result.text then
    print("✅ File exists check: " .. exists_result.text)
else
    print("❌ Error checking file: " .. (exists_result.error or "Unknown error"))
end

print()
print("🎉 Congratulations! You've successfully:")
print("   - Invoked your first tool")
print("   - Created a file")
print("   - Read a file")
print("   - Checked file existence")
print()
print("Next: Try 02-first-agent.lua to learn about agents!")