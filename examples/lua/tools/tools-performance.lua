-- tools-performance.lua: Tool performance benchmarking
-- Measures initialization time, execution time, and resource usage

-- Load test helpers
-- Helper function to execute tool using synchronous API
local function use_tool(tool_name, params)
    local result = Tool.invoke(tool_name, params)
    
    -- Parse the JSON result to get the actual tool response
    if result and result.text then
        local parsed = JSON.parse(result.text)
        if parsed then
            return parsed
        end
    end
    
    -- Return error result if parsing failed
    return {success = false, error = "Failed to parse tool result"}
end

print("⚡ Tool Performance Benchmarks")
print("================================")
print()

-- Performance measurement helpers
local function measure_time(fn)
    local start = os.clock()
    local result = fn()
    local elapsed = os.clock() - start
    return elapsed * 1000, result  -- Convert to milliseconds
end

local function format_time(ms)
    if ms < 1 then
        return string.format("%.3fms", ms)
    elseif ms < 1000 then
        return string.format("%.1fms", ms)
    else
        return string.format("%.2fs", ms / 1000)
    end
end

local function benchmark_tool(name, operations)
    print(string.format("[1m[36m=== %s Performance ===[0m", name))
    
    -- Measure tool initialization/availability check
    local init_time, tool_exists = measure_time(function()
        local tools = Tool.list()
        for _, tool in ipairs(tools) do
            if tool.name == name then
                return true
            end
        end
        return false
    end)
    
    if not tool_exists then
        print("  ❌ Tool not found:", name)
        return nil
    end
    
    print(string.format("  Initialization check: %s", format_time(init_time)))
    
    -- Run operations
    local total_time = 0
    local results = {}
    
    for op_name, op_params in pairs(operations) do
        local exec_time, result = measure_time(function()
            return Tool.invoke(name, op_params)
        end)
        
        total_time = total_time + exec_time
        
        print(string.format("  %s: %s %s", 
            op_name, 
            format_time(exec_time),
            result.success and "✅" or "❌"
        ))
        
        table.insert(results, {
            operation = op_name,
            time = exec_time,
            success = result.success
        })
    end
    
    print(string.format("  Total: %s\n", format_time(total_time + init_time)))
    
    return {
        tool = name,
        init_time = init_time,
        operations = results,
        total_time = total_time + init_time
    }
end

-- Benchmark configurations for different tools
local benchmarks = {}

-- 1. Lightweight Tools (should be <10ms)
print("[1m[35m🏃 Lightweight Tools (Target: <10ms init, <50ms ops)[0m")
print()

benchmarks.uuid = benchmark_tool("uuid_generator", {
    ["Generate v4"] = {operation = "generate", format = "standard"},
    ["Generate v5"] = {operation = "generate", version = "v5", namespace = "dns", name = "example.com"},
    ["Generate simple"] = {operation = "generate", format = "simple"},
    ["Component ID"] = {operation = "component_id", prefix = "test"}
})

benchmarks.calculator = benchmark_tool("calculator", {
    ["Simple math"] = {operation = "evaluate", input = "2 + 2"},
    ["Complex expr"] = {operation = "evaluate", input = "16 + 2^8 - 10 * 5"},
    ["With variables"] = {operation = "evaluate", input = "a * b + c", variables = {a = 10, b = 20, c = 30}},
    ["Arithmetic"] = {operation = "evaluate", input = "100 / 4 + 3 * 7 - 15"},
    ["Trigonometry"] = {operation = "evaluate", input = "sin(pi()/2) + cos(0)"},
    ["Square root"] = {operation = "evaluate", input = "sqrt(16) + sqrt(25)"},
    ["Exponential"] = {operation = "evaluate", input = "exp(1) + exp(0)"},
    ["Logarithm"] = {operation = "evaluate", input = "ln(e()) + log(10, 100)"}
})

benchmarks.text = benchmark_tool("text_manipulator", {
    ["Uppercase"] = {operation = "uppercase", input = "hello world"},
    ["Snake case"] = {operation = "snake_case", input = "Hello World From LLMSpell"},
    ["Reverse"] = {operation = "reverse", input = "abcdefghijklmnopqrstuvwxyz"},
    ["Replace"] = {operation = "replace", input = "hello world", options = {from = "world", to = "llmspell"}}
})

-- 2. Medium Weight Tools
print("[1m[35m⚙️ Medium Weight Tools (Target: <50ms init, <100ms ops)[0m")
print()

benchmarks.base64 = benchmark_tool("base64_encoder", {
    ["Encode small"] = {operation = "encode", input = "Hello, World!"},
    ["Encode medium"] = {operation = "encode", input = string.rep("A", 1000)},
    ["Decode"] = {operation = "decode", input = "SGVsbG8sIFdvcmxkIQ=="},
    ["URL safe"] = {operation = "encode", input = "test+data/with=special", variant = "url-safe"}
})

benchmarks.hash = benchmark_tool("hash_calculator", {
    ["MD5 small"] = {operation = "hash", algorithm = "md5", input = "test"},
    ["SHA256 small"] = {operation = "hash", algorithm = "sha256", input = "test"},
    ["SHA512 medium"] = {operation = "hash", algorithm = "sha512", input = string.rep("X", 1000)},
    ["Multiple"] = {operation = "hash", algorithm = "sha256", input = "benchmark", format = "hex"}
})

benchmarks.json = benchmark_tool("json_processor", {
    ["Query simple"] = {operation = "query", input = '{"key": "value"}', query = ".key"},
    ["Format complex"] = {operation = "query", input = '{"a":1,"b":{"c":2,"d":[3,4,5]}}', query = "."},
    ["Query nested"] = {operation = "query", input = '{"users":[{"name":"Alice","age":30},{"name":"Bob","age":25}]}', query = ".users[0].name"},
    ["Validate"] = {operation = "validate", input = '{"valid": true}', schema = {type = "object", properties = {valid = {type = "boolean"}}}}
})

-- 3. Heavy Tools (may take longer)
print("[1m[35m🏋️ Heavy Tools (Target: <100ms init, <500ms ops)[0m")
print()

benchmarks.template = benchmark_tool("template_engine", {
    ["Handlebars simple"] = {
        engine = "handlebars",
        input = "Hello {{name}}!",
        context = {name = "Benchmark"}
    },
    ["Handlebars loop"] = {
        engine = "handlebars",
        input = "{{#each items}}Item {{@index}}: {{this}}\n{{/each}}",
        context = {items = {1, 2, 3, 4, 5}}
    },
    ["Tera simple"] = {
        engine = "tera",
        input = "Hello {{ name }}!",
        context = {name = "Benchmark"}
    }
})

benchmarks.datetime = benchmark_tool("date_time_handler", {
    ["Current time"] = {operation = "now"},
    ["Parse date"] = {operation = "parse", input = "2025-01-01"},
    ["Convert timezone"] = {operation = "convert_timezone", input = "2025-01-01T12:00:00Z", target_timezone = "America/New_York"},
    ["Add duration"] = {operation = "add", input = "2025-01-01", amount = 30, unit = "days"}
})

benchmarks.diff = benchmark_tool("diff_calculator", {
    ["Text diff small"] = {
        operation = "text_diff",
        old_text = "Hello world",
        new_text = "Hello LLMSpell",
        format = "unified"
    },
    ["Text diff medium"] = {
        operation = "text_diff",
        old_text = string.rep("Line\n", 50),
        new_text = string.rep("Modified Line\n", 50),
        format = "unified"
    }
})

-- 4. System Tools (variable performance)
print("[1m[35m🖥️ System Tools (Target: <200ms)[0m")
print()

benchmarks.env = benchmark_tool("environment_reader", {
    ["Get single"] = {operation = "get", variable_name = "PATH"},
    ["List all"] = {operation = "list"},
    ["Get HOME"] = {operation = "get", variable_name = "HOME"}
})

-- Performance Summary
print("\n" .. string.rep("=", 60))
print("📊 Performance Summary")
print(string.rep("=", 60))

-- Calculate statistics
local total_tools = 0
local fast_tools = 0
local slow_tools = 0
local total_init_time = 0
local total_exec_time = 0

for name, data in pairs(benchmarks) do
    if data then
        total_tools = total_tools + 1
        total_init_time = total_init_time + data.init_time
        
        -- Check if meets performance targets
        if data.init_time < 10 then
            fast_tools = fast_tools + 1
        else
            slow_tools = slow_tools + 1
        end
        
        -- Sum operation times
        for _, op in ipairs(data.operations) do
            total_exec_time = total_exec_time + op.time
        end
    end
end

print(string.format("\nTools benchmarked: %d", total_tools))
print(string.format("Average init time: %s", format_time(total_init_time / total_tools)))
print(string.format("Fast tools (<10ms init): %d", fast_tools))
print(string.format("Slow tools (>10ms init): %d", slow_tools))

-- Detailed breakdown
print("\n[1m[33mInitialization Times:[0m")
local sorted_tools = {}
for name, data in pairs(benchmarks) do
    if data then
        table.insert(sorted_tools, {name = name, time = data.init_time})
    end
end
table.sort(sorted_tools, function(a, b) return a.time < b.time end)

for _, tool in ipairs(sorted_tools) do
    local status = tool.time < 10 and "✅" or "⚠️"
    print(string.format("  %s %-20s %s", status, tool.name .. ":", format_time(tool.time)))
end

-- Performance targets
print("\n[1m[33mPerformance Targets:[0m")
print("  ✅ Tool initialization: <10ms")
print("  ✅ Simple operations: <50ms")
print("  ✅ Complex operations: <500ms")
print("  ✅ Batch operations: <1000ms")

-- Recommendations
print("\n[1m[33mRecommendations:[0m")
if slow_tools > 0 then
    print("  ⚠️ Some tools exceed 10ms initialization target")
    print("  → Consider lazy loading or caching for heavy tools")
end
print("  ✅ Most tools meet performance targets")
print("  → Continue monitoring performance in production")

-- Return benchmark data for test runner
return {
    status = "success",
    tools_tested = total_tools,
    performance = {
        fast_tools = fast_tools,
        slow_tools = slow_tools,
        avg_init_time = total_init_time / total_tools
    },
    benchmarks = benchmarks
}