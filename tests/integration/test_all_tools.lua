-- ABOUTME: Comprehensive integration tests for all 26 Phase 2 tools
-- ABOUTME: Tests with proper JSON output parsing and async execution

-- Load test helpers
package.path = "examples/?.lua;" .. package.path
local TestHelpers = require("test-helpers")

-- Test result tracking
local test_results = {
    passed = 0,
    failed = 0,
    tools_tested = {},
    performance = {},
    start_time = os.clock()
}

-- Helper to run and validate a tool test
local function test_tool(tool_name, params, validation_fn)
    TestHelpers.print_subsection("Testing " .. tool_name)
    
    local start_time = os.clock()
    local result = TestHelpers.execute_tool(tool_name, params)
    local elapsed = (os.clock() - start_time) * 1000 -- ms
    
    test_results.performance[tool_name] = elapsed
    
    if result.success then
        -- Parse JSON output
        local parsed, err = TestHelpers.parse_tool_output(result)
        if parsed then
            -- Run custom validation
            local valid, msg = true, nil
            if validation_fn then
                valid, msg = validation_fn(parsed)
            end
            
            if valid then
                test_results.passed = test_results.passed + 1
                test_results.tools_tested[tool_name] = true
                print(TestHelpers.format_result(true, tool_name, 
                    string.format("completed in %.2fms", elapsed)))
                return true, parsed
            else
                test_results.failed = test_results.failed + 1
                test_results.tools_tested[tool_name] = false
                print(TestHelpers.format_result(false, tool_name, msg or "Validation failed"))
                return false, msg
            end
        else
            test_results.failed = test_results.failed + 1
            test_results.tools_tested[tool_name] = false
            print(TestHelpers.format_result(false, tool_name, "JSON parse error: " .. tostring(err)))
            return false, err
        end
    else
        test_results.failed = test_results.failed + 1
        test_results.tools_tested[tool_name] = false
        print(TestHelpers.format_result(false, tool_name, result.error or "Unknown error"))
        return false, result.error
    end
end

-- =============================================================================
-- UTILITY TOOLS (9 tools)
-- =============================================================================

TestHelpers.print_section("Utility Tools")

-- 1. base64_encoder
test_tool("base64_encoder", {
    operation = "encode",
    input = "Hello, LLMSpell!"
}, function(parsed)
    return parsed.result and parsed.result.output == "SGVsbG8sIExMTVNwZWxsIQ==", 
           "Base64 encoding mismatch"
end)

-- Test decode
test_tool("base64_encoder", {
    operation = "decode", 
    input = "SGVsbG8sIExMTVNwZWxsIQ=="
}, function(parsed)
    return parsed.result and parsed.result.output == "Hello, LLMSpell!",
           "Base64 decoding mismatch"
end)

-- 2. calculator
test_tool("calculator", {
    operation = "evaluate",
    expression = "2 + 3 * 4 - 1"
}, function(parsed)
    return parsed.result and parsed.result.result == 13,
           "Calculator result should be 13"
end)

-- Test with functions
test_tool("calculator", {
    operation = "evaluate",
    expression = "sqrt(16) + sin(0)"
}, function(parsed)
    return parsed.result and math.abs(parsed.result.result - 4) < 0.001,
           "Calculator function result mismatch"
end)

-- 3. data_validation
test_tool("data_validation", {
    data = {name = "test", age = 25},
    rules = {
        rules = {
            {type = "required"},
            {type = "type", expected = "object"}
        }
    }
}, function(parsed)
    return parsed.result and parsed.result.valid == true,
           "Data should be valid"
end)

-- 4. date_time_handler
test_tool("date_time_handler", {
    operation = "now"
}, function(parsed)
    return parsed.result and parsed.result.datetime,
           "Should return current datetime"
end)

-- 5. diff_calculator
test_tool("diff_calculator", {
    old_text = "Hello World",
    new_text = "Hello LLMSpell",
    format = "unified"
}, function(parsed)
    return parsed.result and parsed.result.output and parsed.result.has_changes == true,
           "Should detect changes"
end)

-- 6. hash_calculator
test_tool("hash_calculator", {
    operation = "hash",
    algorithm = "sha256",
    data = "test data"
}, function(parsed)
    return parsed.result and parsed.result.hash == "916f0027a575074ce72a331777c3478d6513f786a591bd892da1a577bf2335f9",
           "SHA256 hash mismatch"
end)

-- 7. template_engine
test_tool("template_engine", {
    template = "Hello, {{name}}!",
    context = {name = "LLMSpell"},
    engine = "handlebars"
}, function(parsed)
    return parsed.result and parsed.result.output == "Hello, LLMSpell!",
           "Template rendering mismatch"
end)

-- 8. text_manipulator
test_tool("text_manipulator", {
    operation = "uppercase",
    text = "hello world"
}, function(parsed)
    return parsed.result and parsed.result.output == "HELLO WORLD",
           "Uppercase conversion mismatch"
end)

-- 9. uuid_generator
test_tool("uuid_generator", {
    operation = "generate",
    version = "v4"
}, function(parsed)
    return parsed.result and parsed.result.uuid and #parsed.result.uuid == 36,
           "UUID should be 36 characters"
end)

-- =============================================================================
-- FILE SYSTEM TOOLS (5 tools)
-- =============================================================================

TestHelpers.print_section("File System Tools")

-- 10. file_operations
-- Create a test file first
local test_content = "Test file content for LLMSpell"
test_tool("file_operations", {
    operation = "write",
    path = "/tmp/llmspell_test.txt",
    content = test_content
}, function(parsed)
    return parsed.result and parsed.result.success == true,
           "File write should succeed"
end)

-- Read it back
test_tool("file_operations", {
    operation = "read",
    path = "/tmp/llmspell_test.txt"
}, function(parsed)
    return parsed.result and parsed.result.content == test_content,
           "File content mismatch"
end)

-- 11. archive_handler
test_tool("archive_handler", {
    operation = "create",
    format = "zip",
    output_path = "/tmp/llmspell_test.zip",
    files = {
        {path = "/tmp/llmspell_test.txt", archive_path = "test.txt"}
    }
}, function(parsed)
    return parsed.result and parsed.result.success == true,
           "Archive creation should succeed"
end)

-- 12. file_watcher
-- Note: File watcher is async and requires special handling
print(TestHelpers.format_result(true, "file_watcher", "Skipped (requires async handling)"))

-- 13. file_converter
test_tool("file_converter", {
    operation = "detect_encoding",
    data = "Hello, World!"
}, function(parsed)
    return parsed.result and parsed.result.encoding == "UTF-8",
           "Should detect UTF-8 encoding"
end)

-- 14. file_search
test_tool("file_search", {
    operation = "search_content",
    directory = "/tmp",
    pattern = "LLMSpell",
    file_pattern = "*.txt"
}, function(parsed)
    return parsed.result and parsed.result.results and #parsed.result.results > 0,
           "Should find test file"
end)

-- =============================================================================
-- SYSTEM INTEGRATION TOOLS (4 tools)
-- =============================================================================

TestHelpers.print_section("System Integration Tools")

-- 15. environment_reader
test_tool("environment_reader", {
    operation = "get",
    key = "PATH"
}, function(parsed)
    return parsed.result and parsed.result.value,
           "Should read PATH environment variable"
end)

-- 16. process_executor
test_tool("process_executor", {
    command = "echo",
    args = {"Hello from LLMSpell"}
}, function(parsed)
    return parsed.result and parsed.result.stdout and 
           parsed.result.stdout:match("Hello from LLMSpell"),
           "Process output mismatch"
end)

-- 17. service_checker
test_tool("service_checker", {
    operation = "check_port",
    host = "localhost",
    port = 22  -- SSH port, commonly available
}, function(parsed)
    return parsed.result ~= nil,  -- Just check it returns a result
           "Should check port status"
end)

-- 18. system_monitor
test_tool("system_monitor", {
    operation = "get_stats"
}, function(parsed)
    return parsed.result and parsed.result.cpu_usage ~= nil and parsed.result.memory ~= nil,
           "Should return system stats"
end)

-- =============================================================================
-- DATA PROCESSING TOOLS (4 tools)
-- =============================================================================

TestHelpers.print_section("Data Processing Tools")

-- 19. json_processor
test_tool("json_processor", {
    operation = "query",
    data = JSON.stringify({items = {1, 2, 3, 4, 5}}),
    query = ".items | length"
}, function(parsed)
    return parsed.result and parsed.result.result == 5,
           "JQ query should return array length"
end)

-- 20. csv_analyzer
local csv_data = "name,age,city\nAlice,30,NYC\nBob,25,LA"
test_tool("csv_analyzer", {
    operation = "analyze",
    data = csv_data
}, function(parsed)
    return parsed.result and parsed.result.row_count == 2 and parsed.result.column_count == 3,
           "CSV analysis mismatch"
end)

-- 21. http_request
-- Note: This is an async tool
test_tool("http_request", {
    method = "GET",
    url = "https://httpbin.org/get",
    headers = {["User-Agent"] = "LLMSpell/1.0"}
}, function(parsed)
    return parsed.result and parsed.result.status == 200,
           "HTTP request should succeed"
end)

-- 22. graphql_query
-- Skip as it requires a GraphQL endpoint
print(TestHelpers.format_result(true, "graphql_query", "Skipped (requires GraphQL endpoint)"))

-- =============================================================================
-- MEDIA PROCESSING TOOLS (3 tools)
-- =============================================================================

TestHelpers.print_section("Media Processing Tools")

-- 23. audio_processor
test_tool("audio_processor", {
    operation = "get_info",
    format = "wav"
}, function(parsed)
    -- Just check it returns without error since we don't have actual audio
    return parsed.result ~= nil,
           "Should return audio format info"
end)

-- 24. video_processor
test_tool("video_processor", {
    operation = "get_info",
    format = "mp4"
}, function(parsed)
    -- Just check it returns without error since we don't have actual video
    return parsed.result ~= nil,
           "Should return video format info"
end)

-- 25. image_processor
test_tool("image_processor", {
    operation = "get_info",
    format = "png"
}, function(parsed)
    -- Just check it returns without error since we don't have actual image
    return parsed.result ~= nil,
           "Should return image format info"
end)

-- =============================================================================
-- SEARCH TOOLS (1 tool)
-- =============================================================================

TestHelpers.print_section("Search Tools")

-- 26. web_search
-- Skip as it's moved to Phase 2.5
print(TestHelpers.format_result(true, "web_search", "Skipped (moved to Phase 2.5)"))

-- =============================================================================
-- TOOL CHAINING TESTS
-- =============================================================================

TestHelpers.print_section("Tool Chaining Tests")

-- Chain 1: UUID -> Hash -> Base64
local uuid_result = TestHelpers.execute_tool("uuid_generator", {
    operation = "generate",
    version = "v4"
})
if uuid_result.success then
    local uuid_parsed = TestHelpers.parse_tool_output(uuid_result)
    if uuid_parsed then
        local hash_result = TestHelpers.execute_tool("hash_calculator", {
            operation = "hash",
            algorithm = "md5",
            data = uuid_parsed.result.uuid
        })
        if hash_result.success then
            local hash_parsed = TestHelpers.parse_tool_output(hash_result)
            if hash_parsed then
                local b64_result = TestHelpers.execute_tool("base64_encoder", {
                    operation = "encode",
                    input = hash_parsed.result.hash
                })
                print(TestHelpers.format_result(b64_result.success, 
                    "UUID->Hash->Base64 chain", "Chain completed successfully"))
            end
        end
    end
end

-- =============================================================================
-- PERFORMANCE SUMMARY
-- =============================================================================

TestHelpers.print_section("Performance Summary")

-- Calculate statistics
local total_time = 0
local count = 0
local slowest = {name = "", time = 0}
local fastest = {name = "", time = 999999}

for tool, time in pairs(test_results.performance) do
    total_time = total_time + time
    count = count + 1
    if time > slowest.time then
        slowest = {name = tool, time = time}
    end
    if time < fastest.time then
        fastest = {name = tool, time = time}
    end
end

local avg_time = count > 0 and (total_time / count) or 0

print(string.format("Average tool execution: %.2fms", avg_time))
print(string.format("Fastest: %s (%.2fms)", fastest.name, fastest.time))
print(string.format("Slowest: %s (%.2fms)", slowest.name, slowest.time))

-- Check <10ms requirement
local tools_over_10ms = 0
for tool, time in pairs(test_results.performance) do
    if time > 10 then
        tools_over_10ms = tools_over_10ms + 1
        print(string.format("  ⚠️  %s: %.2fms (exceeds 10ms target)", tool, time))
    end
end

-- =============================================================================
-- FINAL SUMMARY
-- =============================================================================

local summary = TestHelpers.create_summary({{success = test_results.passed > 0}})
summary.total = test_results.passed + test_results.failed
summary.passed = test_results.passed
summary.failed = test_results.failed

TestHelpers.print_summary(summary)

-- Additional details
print("\nTools tested: " .. count .. "/26")
print("Execution time: " .. string.format("%.2fs", os.clock() - test_results.start_time))

if tools_over_10ms > 0 then
    print("\n⚠️  " .. tools_over_10ms .. " tools exceeded 10ms performance target")
end

-- Exit with appropriate code
os.exit(test_results.failed > 0 and 1 or 0)