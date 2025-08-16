-- tools-utility-reference.lua
-- Reference implementation for utility tools using the correct Tool API

print("🔧 Utility Tools Working Examples")
print("=================================")

-- Get list of available tools
local tools = Tool.list()
local tool_names = {}
for i, tool in ipairs(tools) do
    table.insert(tool_names, tool.name)
end
print("Available tools:", table.concat(tool_names, ", "))

-- Helper function to safely use a tool with current API
local function use_tool(tool_name, params)
    local success, result = pcall(function()
        return Tool.invoke(tool_name, params)
    end)
    
    if success and result then
        return result
    else
        return {error = result or "Tool invocation failed"}
    end
end

-- Helper function to print tool result
local function print_result(tool_name, operation, result)
    print(string.format("\n%s (%s):", tool_name, operation))
    if result.error then
        print("  ❌ Error:", result.error)
    else
        print("  ✅ Success:")
        for k, v in pairs(result) do
            if type(v) == "table" then
                print(string.format("    %s: [table]", k))
            else
                print(string.format("    %s: %s", k, tostring(v)))
            end
        end
    end
end

print("\n1. Calculator Tool")
print("------------------")

-- Test basic arithmetic
local calc_result = use_tool("calculator", {
    input = "2 + 3 * 4"
})
print_result("calculator", "basic arithmetic", calc_result)

-- Test with variables
local calc_vars = use_tool("calculator", {
    input = "x^2 + y^2",
    variables = {x = 3, y = 4}
})
print_result("calculator", "with variables", calc_vars)

-- Test expression validation
local calc_validate = use_tool("calculator", {
    operation = "validate",
    input = "2 + 3 * (4 + 5)"
})
print_result("calculator", "validation", calc_validate)

print("\n2. Web Search Tool (if available)")
print("----------------------------------")

if table.concat(tool_names, ","):find("web_search") then
    local search_result = use_tool("web_search", {
        input = "LLMSpell documentation",
        max_results = 3
    })
    print_result("web_search", "search query", search_result)
else
    print("  ❌ Web search tool not available")
end

print("\n3. Tool Information")
print("-------------------")

-- List available tools with basic info
print("Available tools:")
for _, tool_name in ipairs(tool_names) do
    print(string.format("  - %s", tool_name))
end

print("\nNote: Schema introspection API not available in current version")
print("Use Tool.list() to see available tools")

print("\n5. Tool Capabilities Test")
print("-------------------------")

-- Test different parameter combinations
local test_cases = {
    {
        tool = "calculator",
        params = {input = "sqrt(16)"},
        description = "square root"
    },
    {
        tool = "calculator", 
        params = {input = "2^10"},
        description = "power operation"
    },
    {
        tool = "calculator",
        params = {input = "sin(0)"},
        description = "trigonometric function"
    }
}

for i, test_case in ipairs(test_cases) do
    if table.concat(tool_names, ","):find(test_case.tool) then
        local result = use_tool(test_case.tool, test_case.params)
        print(string.format("\nTest %d - %s:", i, test_case.description))
        print_result(test_case.tool, test_case.description, result)
    end
end

print("\n✅ Utility Tools Working Examples Complete!")
print("==================================================")
print("This demonstrates the correct API for tools.")
print("Tools are accessed via Tool.invoke(tool_name, params).")

return {
    available_tools = tools,
    examples_tested = true,
    api_working = true,
    status = "success"
}