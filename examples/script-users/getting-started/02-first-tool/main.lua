-- Example: Your First Tool - Working with Files
-- Purpose: Learn how to use tools to perform actions
-- Audience: Script Users (Beginners)
-- Prerequisites: Completed 01-hello-world
-- Expected Output: File operations demonstration
-- Version: 0.7.0
-- Tags: getting-started, tools, file-operations, beginner

print("=== Your First Tool: File Operations ===")
print("")

-- Tools are pre-built functions that perform specific tasks.
-- The file_operations tool can read, write, and manage files.

-- Step 1: Write a file
print("1. Writing a message to a file...")

local write_result = Tool.invoke("file_operations", {
    operation = "write",
    path = "/tmp/my_message.txt",
    input = "Hello from LLMSpell!\nThis file was created using the file_operations tool.\nTimestamp: " .. os.date()
})

if write_result and write_result.success then
    print("   ✅ File written successfully!")
    print("   Path: /tmp/my_message.txt")
else
    print("   ❌ Failed to write file: " .. (write_result and write_result.error or "Unknown error"))
end

print("")

-- Step 2: Check if file exists
print("2. Checking if our file exists...")

local exists_result = Tool.invoke("file_operations", {
    operation = "exists",
    path = "/tmp/my_message.txt"
})

if exists_result and exists_result.text then
    print("   ✅ File exists!")
else
    print("   ❌ File not found")
end

print("")

-- Step 3: Read the file back
print("3. Reading the file contents...")

local read_result = Tool.invoke("file_operations", {
    operation = "read",
    path = "/tmp/my_message.txt"
})

if read_result and read_result.text then
    print("   File contents:")
    print("   " .. string.rep("-", 40))
    -- Indent each line for better formatting
    for line in read_result.text:gmatch("[^\n]+") do
        print("   " .. line)
    end
    print("   " .. string.rep("-", 40))
else
    print("   ❌ Failed to read file")
end

print("")

-- Step 4: Get file information
print("4. Getting file information...")

local info_result = Tool.invoke("file_operations", {
    operation = "info",
    path = "/tmp/my_message.txt"
})

if info_result and info_result.text then
    print("   " .. info_result.text)
else
    print("   ❌ Failed to get file info")
end

print("")

-- Step 5: List available tools
print("5. Other tools you can explore:")
print("")

-- Tool.list() returns all available tools
local tools = Tool.list()
local tool_count = 0
for _, tool in ipairs(tools) do
    tool_count = tool_count + 1
    if tool_count <= 5 then  -- Show first 5 tools
        print("   - " .. tostring(tool))
    end
end

print("   ... and " .. (#tools - 5) .. " more tools!")

print("")
print("🎉 Congratulations! You've successfully:")
print("   - Used the Tool.invoke() function")
print("   - Performed file operations (write, exists, read, info)")
print("   - Discovered available tools with Tool.list()")
print("")
print("💡 Key Concepts:")
print("   - Tools are invoked with Tool.invoke(name, parameters)")
print("   - Each tool has different operations and parameters")
print("   - Results contain success/error status and output")
print("")
print("Next: Continue to '03-simple-agent' to create your first AI agent!")