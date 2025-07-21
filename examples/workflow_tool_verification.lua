-- ABOUTME: Comprehensive tool integration verification for workflows
-- ABOUTME: Tests all 33+ tools from Phases 3.0-3.2 with workflow patterns

print("=== Workflow Tool Integration Verification ===\n")

-- Enable debugging for detailed output
Workflow.enableDebug(true)

-- Track test results
local test_results = {
    passed = 0,
    failed = 0,
    errors = {}
}

-- Helper function to test a tool in a workflow
function test_tool_in_workflow(tool_name, workflow_type, test_input)
    print("\n--- Testing " .. tool_name .. " in " .. workflow_type .. " workflow ---")
    
    local success, result = pcall(function()
        local workflow
        
        if workflow_type == "sequential" then
            workflow = Workflow.sequential({
                name = "test_" .. tool_name .. "_sequential",
                steps = {
                    {
                        name = "test_step",
                        type = "tool",
                        tool = tool_name,
                        input = test_input
                    }
                }
            })
        elseif workflow_type == "parallel" then
            workflow = Workflow.parallel({
                name = "test_" .. tool_name .. "_parallel",
                branches = {
                    {
                        name = "branch1",
                        steps = {{
                            name = "test_step",
                            type = "tool",
                            tool = tool_name,
                            input = test_input
                        }}
                    }
                }
            })
        end
        
        -- Execute workflow
        local exec_result = workflow:execute({})
        
        -- Validate execution
        if exec_result and exec_result.success ~= false then
            print("✓ " .. tool_name .. " executed successfully in " .. workflow_type)
            test_results.passed = test_results.passed + 1
            return true
        else
            print("✗ " .. tool_name .. " failed in " .. workflow_type)
            test_results.failed = test_results.failed + 1
            table.insert(test_results.errors, {
                tool = tool_name,
                workflow = workflow_type,
                error = exec_result and exec_result.error or "Unknown error"
            })
            return false
        end
    end)
    
    if not success then
        print("✗ " .. tool_name .. " error: " .. tostring(result))
        test_results.failed = test_results.failed + 1
        table.insert(test_results.errors, {
            tool = tool_name,
            workflow = workflow_type,
            error = tostring(result)
        })
        return false
    end
    
    return result
end

-- Test File System Tools (8 tools)
print("\n=== Testing File System Tools ===")

test_tool_in_workflow("file_operations", "sequential", {
    operation = "read",
    path = "/tmp/test_workflow_file.txt"
})

test_tool_in_workflow("file_search", "sequential", {
    pattern = "*.txt",
    directory = "/tmp"
})

test_tool_in_workflow("file_watcher", "sequential", {
    path = "/tmp",
    events = {"create", "modify"},
    duration = 1
})

test_tool_in_workflow("archive_handler", "sequential", {
    operation = "list",
    archive_path = "/tmp/test.zip"
})

test_tool_in_workflow("file_converter", "sequential", {
    input_path = "/tmp/test.md",
    output_format = "html"
})

-- Test Data Processing Tools (4 tools)
print("\n=== Testing Data Processing Tools ===")

test_tool_in_workflow("json_processor", "sequential", {
    input = '{"name": "test", "value": 42}',
    operation = "parse"
})

test_tool_in_workflow("csv_analyzer", "sequential", {
    input = "name,value\ntest,42\n",
    operation = "analyze"
})

-- Test Utility Tools (9 tools)
print("\n=== Testing Utility Tools ===")

test_tool_in_workflow("calculator", "sequential", {
    input = "2 + 2"
})

test_tool_in_workflow("text_manipulator", "sequential", {
    input = "Hello World",
    operation = "uppercase"
})

test_tool_in_workflow("base64_encoder", "sequential", {
    input = "test data",
    operation = "encode"
})

test_tool_in_workflow("hash_calculator", "sequential", {
    input = "test data",
    algorithm = "sha256"
})

test_tool_in_workflow("uuid_generator", "sequential", {
    version = "v4"
})

test_tool_in_workflow("date_time_handler", "sequential", {
    operation = "now",
    format = "iso"
})

test_tool_in_workflow("template_engine", "sequential", {
    template = "Hello {{name}}!",
    data = {name = "World"}
})

test_tool_in_workflow("diff_calculator", "sequential", {
    text1 = "Hello World",
    text2 = "Hello Lua"
})

test_tool_in_workflow("data_validation", "sequential", {
    input = '{"name": "test"}',
    schema = {
        type = "object",
        properties = {
            name = {type = "string"}
        }
    }
})

-- Test System Integration Tools (4 tools)
print("\n=== Testing System Integration Tools ===")

test_tool_in_workflow("environment_reader", "sequential", {
    variable = "PATH"
})

test_tool_in_workflow("process_executor", "sequential", {
    command = "echo",
    args = {"test"}
})

test_tool_in_workflow("system_monitor", "sequential", {
    metrics = {"cpu", "memory"}
})

test_tool_in_workflow("service_checker", "sequential", {
    services = {"http://localhost:8080"}
})

-- Test API/Web Tools (8 tools)
print("\n=== Testing API/Web Tools ===")

test_tool_in_workflow("http_request", "sequential", {
    url = "https://httpbin.org/get",
    method = "GET"
})

test_tool_in_workflow("graphql_query", "sequential", {
    endpoint = "https://api.github.com/graphql",
    query = "{ viewer { login } }"
})

test_tool_in_workflow("web_search", "sequential", {
    query = "test",
    limit = 1
})

test_tool_in_workflow("url_analyzer", "sequential", {
    url = "https://example.com"
})

test_tool_in_workflow("api_tester", "sequential", {
    url = "https://httpbin.org/status/200",
    method = "GET",
    expected_status = 200
})

test_tool_in_workflow("webhook_caller", "sequential", {
    url = "https://httpbin.org/post",
    method = "POST",
    data = {test = "data"}
})

test_tool_in_workflow("web_scraper", "sequential", {
    url = "https://example.com",
    selector = "h1"
})

test_tool_in_workflow("sitemap_crawler", "sequential", {
    url = "https://example.com/sitemap.xml"
})

-- Test Media Tools (3 tools)
print("\n=== Testing Media Tools ===")

test_tool_in_workflow("image_processor", "sequential", {
    operation = "info",
    path = "/tmp/test.jpg"
})

test_tool_in_workflow("audio_processor", "sequential", {
    operation = "info",
    path = "/tmp/test.mp3"
})

test_tool_in_workflow("video_processor", "sequential", {
    operation = "info",
    path = "/tmp/test.mp4"
})

-- Test Communication Tools (2 tools)
print("\n=== Testing Communication Tools ===")

test_tool_in_workflow("email_sender", "sequential", {
    to = ["test@example.com"],
    subject = "Test",
    body = "Test email",
    provider = "mock"
})

test_tool_in_workflow("database_connector", "sequential", {
    connection = "test",
    query = "SELECT 1"
})

-- Test Tool Composition in Workflows
print("\n\n=== Testing Tool Composition Patterns ===")

-- Sequential composition
local seq_composition = Workflow.sequential({
    name = "tool_composition_sequential",
    steps = {
        {
            name = "generate_uuid",
            type = "tool",
            tool = "uuid_generator",
            input = {version = "v4"}
        },
        {
            name = "encode_uuid",
            type = "tool",
            tool = "base64_encoder",
            input = {
                input = "{{step:generate_uuid}}",
                operation = "encode"
            }
        },
        {
            name = "hash_encoded",
            type = "tool",
            tool = "hash_calculator",
            input = {
                input = "{{step:encode_uuid}}",
                algorithm = "sha256"
            }
        }
    }
})

local comp_result = seq_composition:execute({})
if comp_result and comp_result.success ~= false then
    print("✓ Sequential tool composition successful")
    test_results.passed = test_results.passed + 1
else
    print("✗ Sequential tool composition failed")
    test_results.failed = test_results.failed + 1
end

-- Parallel composition with data aggregation
local par_composition = Workflow.parallel({
    name = "tool_composition_parallel",
    branches = {
        {
            name = "branch_calc",
            steps = {{
                name = "calculate",
                type = "tool",
                tool = "calculator",
                input = {input = "10 * 5"}
            }}
        },
        {
            name = "branch_time",
            steps = {{
                name = "get_time",
                type = "tool",
                tool = "date_time_handler",
                input = {operation = "now", format = "unix"}
            }}
        },
        {
            name = "branch_uuid",
            steps = {{
                name = "get_uuid",
                type = "tool",
                tool = "uuid_generator",
                input = {version = "v4"}
            }}
        }
    }
})

par_result = par_composition:execute({})
if par_result and par_result.success ~= false then
    print("✓ Parallel tool composition successful")
    test_results.passed = test_results.passed + 1
else
    print("✗ Parallel tool composition failed")
    test_results.failed = test_results.failed + 1
end

-- Test Error Handling
print("\n\n=== Testing Error Handling ===")

-- Test with invalid tool
local error_workflow = Workflow.sequential({
    name = "error_test",
    error_strategy = "continue",
    steps = {
        {
            name = "invalid_tool",
            type = "tool",
            tool = "non_existent_tool",
            input = {}
        },
        {
            name = "valid_tool",
            type = "tool",
            tool = "calculator",
            input = {input = "1 + 1"}
        }
    }
})

local error_result = error_workflow:execute({})
if error_result then
    print("✓ Error handling test completed")
    test_results.passed = test_results.passed + 1
else
    print("✗ Error handling test failed")
    test_results.failed = test_results.failed + 1
end

-- Test timeout behavior
print("\n\n=== Testing Timeout Behavior ===")

local timeout_workflow = Workflow.sequential({
    name = "timeout_test",
    timeout_ms = 100,
    steps = {
        {
            name = "slow_operation",
            type = "tool",
            tool = "process_executor",
            input = {
                command = "sleep",
                args = {"0.2"}
            }
        }
    }
})

local timeout_result = timeout_workflow:execute({})
if timeout_result and timeout_result.error and timeout_result.error:find("timeout") then
    print("✓ Timeout behavior working correctly")
    test_results.passed = test_results.passed + 1
else
    print("✗ Timeout behavior test failed")
    test_results.failed = test_results.failed + 1
end

-- Summary
print("\n\n=== Test Summary ===")
print("Total tests: " .. (test_results.passed + test_results.failed))
print("Passed: " .. test_results.passed)
print("Failed: " .. test_results.failed)

if #test_results.errors > 0 then
    print("\nErrors:")
    for _, error in ipairs(test_results.errors) do
        print("  - " .. error.tool .. " in " .. error.workflow .. ": " .. error.error)
    end
end

-- Performance test
print("\n\n=== Performance Testing ===")

local start_time = os.clock()
local perf_workflow = Workflow.sequential({
    name = "performance_test",
    steps = {
        {name = "step1", type = "tool", tool = "calculator", input = {input = "1 + 1"}},
        {name = "step2", type = "tool", tool = "text_manipulator", input = {input = "test", operation = "uppercase"}},
        {name = "step3", type = "tool", tool = "uuid_generator", input = {version = "v4"}}
    }
})

-- Execute multiple times
for i = 1, 10 do
    perf_workflow:execute({})
end

local elapsed = os.clock() - start_time
local avg_time = elapsed / 10 * 1000 -- Convert to ms
print(string.format("Average workflow execution time: %.2f ms", avg_time))

if avg_time < 50 then
    print("✓ Performance requirement met (<50ms)")
else
    print("✗ Performance requirement not met (>50ms)")
end

print("\n=== Tool Integration Verification Complete ===")