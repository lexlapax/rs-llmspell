-- tools-utility-reference.lua
-- Reference implementation for utility tools using the correct Tool API

print("🔧 Utility Tools Working Examples")
print("=================================")

-- Get list of available tools
local tools = Tool.list()
print("Available tools:", table.concat(tools, ", "))

-- Helper function to safely use a tool
local function use_tool(tool_name, params)
    local tool = Tool.get(tool_name)
    if not tool then
        return {error = "Tool not found: " .. tool_name}
    end
    
    local success, result = pcall(function()
        return tool.execute(params)
    end)
    
    if success then
        return result
    else
        return {error = result}
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

if table.concat(tools, ","):find("web_search") then
    local search_result = use_tool("web_search", {
        input = "LLMSpell documentation",
        max_results = 3
    })
    print_result("web_search", "search query", search_result)
else
    print("  ❌ Web search tool not available")
end

print("\n3. Tool Schema Information")
print("-------------------------")

for _, tool_name in ipairs(tools) do
    local tool = Tool.get(tool_name)
    if tool and tool.getSchema then
        local schema_success, schema = pcall(function()
            return tool.getSchema()
        end)
        
        if schema_success then
            print(string.format("\n%s schema:", tool_name))
            print(string.format("  Name: %s", schema.name or "unknown"))
            print(string.format("  Description: %s", schema.description or "no description"))
            
            if schema.parameters then
                print("  Parameters:")
                for param_name, param_info in pairs(schema.parameters) do
                    print(string.format("    %s: %s", param_name, param_info.type or "unknown"))
                end
            end
        else
            print(string.format("\n%s schema: Error - %s", tool_name, schema))
        end
    else
        print(string.format("\n%s schema: Not available", tool_name))
    end
end

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
    if table.concat(tools, ","):find(test_case.tool) then
        local result = use_tool(test_case.tool, test_case.params)
        print(string.format("\nTest %d - %s:", i, test_case.description))
        print_result(test_case.tool, test_case.description, result)
    end
end

print("\n✅ Utility Tools Working Examples Complete!")
print("==================================================")
print("This demonstrates the actual working API for tools.")
print("Tools are accessed via Tool.get() and executed directly.")

return {
    available_tools = tools,
    examples_tested = true,
    api_working = true,
    status = "success"
}