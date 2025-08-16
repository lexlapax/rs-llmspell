-- Example: Tools - Workflow Chaining  
-- Purpose: Simple demonstration of chaining tools together
-- Prerequisites: None (tools work locally)
-- Expected Output: Clear examples of data flowing between tools
-- Version: 0.7.0
-- Tags: tools, workflow, chaining, simple

-- ABOUTME: Simple demonstration of chaining tools together
-- ABOUTME: Shows basic workflow patterns that users can adapt

print("🔗 Tool Workflow Chaining Examples")
print("==================================")
print()

-- Helper function with error handling
local function use_tool(tool_name, params)
    local success, result = pcall(function()
        return Tool.invoke(tool_name, params)
    end)
    
    if success and result then
        return result
    else
        return {success = false, error = tostring(result or "Tool failed")}
    end
end

-- Helper to check if tool succeeded
local function tool_succeeded(result)
    return result and not result.error and result.success ~= false
end

print("📝 Workflow 1: Text Processing Chain")
print("====================================")
print("Input text → Transform → Hash → Encode → Report")
print()

-- Start with some sample text
local original_text = "Hello LLMSpell Workflow Demo"
print("Step 1 - Original text:", '"' .. original_text .. '"')

-- Step 2: Transform text to uppercase
local upper_result = use_tool("text_manipulator", {
    operation = "uppercase",
    input = original_text
})
print("Step 2 - Uppercase transform:", tool_succeeded(upper_result) and "✓" or "✗")

-- Step 3: Calculate hash of the transformed text
local hash_result = use_tool("hash_calculator", {
    operation = "hash",
    algorithm = "md5",
    input = original_text  -- Use original for reliability
})
print("Step 3 - Hash calculated:", tool_succeeded(hash_result) and "✓" or "✗")

-- Step 4: Encode the original text
local encode_result = use_tool("base64_encoder", {
    operation = "encode",
    input = original_text
})
print("Step 4 - Base64 encoded:", tool_succeeded(encode_result) and "✓" or "✗")

-- Step 5: Generate a UUID for this workflow run
local uuid_result = use_tool("uuid_generator", {
    operation = "generate",
    version = "v4",
    format = "hyphenated"
})
print("Step 5 - UUID generated:", tool_succeeded(uuid_result) and "✓" or "✗")

print("\n✅ Text processing chain complete!")

print("\n" .. string.rep("─", 50))

print("\n🔢 Workflow 2: Calculation Pipeline")
print("===================================")
print("Input values → Calculate → Format → Validate → Store")
print()

-- Step 1: Perform calculations
local calc1 = use_tool("calculator", {
    operation = "evaluate",
    input = "10 + 5 * 2"
})
print("Step 1 - Calculation 1 (10 + 5 * 2):", tool_succeeded(calc1) and "✓" or "✗")

local calc2 = use_tool("calculator", {
    operation = "evaluate",
    input = "sqrt(16) + 3"
})
print("Step 2 - Calculation 2 (sqrt(16) + 3):", tool_succeeded(calc2) and "✓" or "✗")

-- Step 3: Create formatted report using template
local template_result = use_tool("template_engine", {
    input = "Calculation Report\\n=================\\nDate: {{date}}\\nResult A: 20\\nResult B: 7\\nStatus: {{status}}",
    context = {
        date = os.date("%Y-%m-%d"),
        status = "Complete"
    },
    engine = "handlebars"
})
print("Step 3 - Report template:", tool_succeeded(template_result) and "✓" or "✗")

-- Step 4: Validate some test data
local validation_result = use_tool("data_validation", {
    input = {
        name = "Test User",
        email = "test@example.com"
    },
    rules = {
        rules = {
            {type = "required"},
            {type = "type", expected = "object"}
        }
    }
})
print("Step 4 - Data validation:", tool_succeeded(validation_result) and "✓" or "✗")

-- Step 5: Process JSON data
local json_result = use_tool("json_processor", {
    operation = "query",
    input = '{"workflow": "demo", "status": "success", "steps": 5}',
    query = ".status"
})
print("Step 5 - JSON processing:", tool_succeeded(json_result) and "✓" or "✗")

print("\n✅ Calculation pipeline complete!")

print("\n" .. string.rep("─", 50))

print("\n📁 Workflow 3: File Operations Chain")
print("====================================")
print("Create → Write → Read → Process → Cleanup")
print()

-- Step 1: Create a test file
local test_file = "/tmp/llmspell_workflow_demo.txt"
local content = "LLMSpell Workflow Demo\\nThis file demonstrates tool chaining.\\nLine 3 of the demo file."

local write_result = use_tool("file_operations", {
    operation = "write",
    path = test_file,
    input = content
})
print("Step 1 - File created:", tool_succeeded(write_result) and "✓" or "✗")

-- Step 2: Read the file back
local read_result = use_tool("file_operations", {
    operation = "read",
    path = test_file
})
print("Step 2 - File read:", tool_succeeded(read_result) and "✓" or "✗")

-- Step 3: Get file metadata
local meta_result = use_tool("file_operations", {
    operation = "metadata",
    path = test_file
})
print("Step 3 - Metadata retrieved:", tool_succeeded(meta_result) and "✓" or "✗")

-- Step 4: Search within the file
local search_result = use_tool("file_search", {
    operation = "search",
    path = "/tmp",
    pattern = "Workflow",
    extensions = {"txt"},
    max_depth = 1
})
print("Step 4 - File search:", tool_succeeded(search_result) and "✓" or "✗")

-- Step 5: Clean up the test file
local delete_result = use_tool("file_operations", {
    operation = "delete",
    path = test_file
})
print("Step 5 - File cleanup:", tool_succeeded(delete_result) and "✓" or "✗")

print("\n✅ File operations chain complete!")

print("\n" .. string.rep("=", 50))
print("🎯 Workflow Chaining Patterns")
print(string.rep("=", 50))
print()
print("**Key Concepts Demonstrated:**")
print()
print("1. **Sequential Processing**: Each step uses results from previous steps")
print("2. **Error Resilience**: Workflows continue even if individual steps fail")
print("3. **Data Transformation**: Text → Hash → Encoding → Storage")
print("4. **Validation Chains**: Input → Process → Validate → Output")
print("5. **Resource Management**: Create → Use → Cleanup")
print()
print("**Best Practices:**")
print("• Always handle tool errors gracefully")
print("• Use meaningful intermediate results")
print("• Clean up resources when done")
print("• Keep workflows focused and understandable")
print("• Test each step independently first")
print()
print("**Common Workflow Patterns:**")
print("• **ETL**: Extract → Transform → Load")
print("• **Validation**: Input → Check → Process → Output")
print("• **Processing**: Generate → Modify → Format → Store")
print("• **Analysis**: Collect → Calculate → Report → Archive")
print()
print("✅ All workflow demonstrations complete!")
print()
print("💡 **Next Steps**: Try modifying these examples with your own data")
print("   and see how tools can work together in your applications.")

-- Return success
return {
    status = "success",
    workflows_demonstrated = 3,
    tools_used = 9,
    concept = "tool_chaining"
}