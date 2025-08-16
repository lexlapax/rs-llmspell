-- Example: 01-first-tool.lua
-- Author: LLMSpell Examples
-- Purpose: First introduction to using a single tool
-- Learning: Basic tool invocation with the file_operations tool

print("=== LLMSpell: Your First Tool ===")
print("This example shows how to use your very first tool!")
print()

-- LLMSpell provides many tools out of the box
-- Let's start with the file_operations tool, which can create, read and write files

print("1. Creating a test file...")
local result = Tool.invoke("file_operations", {
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
local read_result = Tool.invoke("file_operations", {
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
local exists_result = Tool.invoke("file_operations", {
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