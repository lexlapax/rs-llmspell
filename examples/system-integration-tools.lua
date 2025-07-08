-- system-integration-tools.lua
-- Examples for system integration tools with security controls

print("🖥️ System Integration Tools Examples")
print("====================================")

local Agent = require("llmspell.agent")
local agent = Agent.create("claude-3-sonnet-20240229")

print("\n1. Environment Reader Tool")
print("--------------------------")

-- Get specific environment variable
local path_var = agent:use_tool("environment_reader", {
    operation = "get",
    variable = "PATH"
})
print("PATH variable:", path_var)

-- Get home directory
local home_var = agent:use_tool("environment_reader", {
    operation = "get",
    variable = "HOME"
})
print("HOME variable:", home_var)

-- List environment variables (filtered for security)
local env_list = agent:use_tool("environment_reader", {
    operation = "list",
    pattern = "SHELL*"
})
print("Shell variables:", env_list)

-- Get system information
local system_info = agent:use_tool("environment_reader", {
    operation = "system_info"
})
print("System info:", system_info)

-- Find executable
local executable_path = agent:use_tool("environment_reader", {
    operation = "which",
    command = "ls"
})
print("ls executable:", executable_path)

-- Check if variable exists
local var_exists = agent:use_tool("environment_reader", {
    operation = "exists",
    variable = "USER"
})
print("USER variable exists:", var_exists)

print("\n2. Process Executor Tool")
print("------------------------")

-- Execute simple command
local simple_command = agent:use_tool("process_executor", {
    command = "echo",
    args = {"Hello", "from", "LLMSpell!"}
})
print("Echo command:", simple_command)

-- List files
local list_files = agent:use_tool("process_executor", {
    command = "ls",
    args = {"-la", "/tmp"}
})
print("List files:", list_files)

-- Get current user
local whoami = agent:use_tool("process_executor", {
    command = "whoami"
})
print("Current user:", whoami)

-- Execute with timeout
local timeout_command = agent:use_tool("process_executor", {
    command = "sleep",
    args = {"2"},
    timeout_ms = 1000
})
print("Timeout command:", timeout_command)

-- Execute with working directory
local pwd_command = agent:use_tool("process_executor", {
    command = "pwd",
    working_dir = "/tmp"
})
print("Working directory:", pwd_command)

-- Execute with environment variables
local env_command = agent:use_tool("process_executor", {
    command = "env",
    env_vars = {
        CUSTOM_VAR = "LLMSpell Test"
    }
})
print("Environment command:", env_command)

print("\n3. Service Checker Tool")
print("-----------------------")

-- Check TCP port
local tcp_check = agent:use_tool("service_checker", {
    operation = "check_tcp",
    host = "127.0.0.1",
    port = 22
})
print("SSH port check:", tcp_check)

-- Check HTTP service
local http_check = agent:use_tool("service_checker", {
    operation = "check_http",
    url = "http://127.0.0.1:8080"
})
print("HTTP service check:", http_check)

-- Check HTTPS service
local https_check = agent:use_tool("service_checker", {
    operation = "check_https",
    url = "https://google.com"
})
print("HTTPS service check:", https_check)

-- DNS resolution check
local dns_check = agent:use_tool("service_checker", {
    operation = "check_dns",
    hostname = "localhost"
})
print("DNS check:", dns_check)

-- Multiple service checks
local multi_check = agent:use_tool("service_checker", {
    operation = "check_multiple",
    targets = {
        {type = "tcp", host = "127.0.0.1", port = 22},
        {type = "http", url = "http://127.0.0.1:8080"},
        {type = "dns", hostname = "localhost"}
    }
})
print("Multiple checks:", multi_check)

-- Service discovery
local service_discovery = agent:use_tool("service_checker", {
    operation = "discover",
    host = "127.0.0.1",
    port_range = {start = 20, ["end"] = 25}
})
print("Service discovery:", service_discovery)

print("\n4. System Monitor Tool")
print("----------------------")

-- Get CPU information
local cpu_info = agent:use_tool("system_monitor", {
    operation = "collect",
    metrics = {"cpu"}
})
print("CPU info:", cpu_info)

-- Get memory information
local memory_info = agent:use_tool("system_monitor", {
    operation = "collect",
    metrics = {"memory"}
})
print("Memory info:", memory_info)

-- Get disk information
local disk_info = agent:use_tool("system_monitor", {
    operation = "collect",
    metrics = {"disk"}
})
print("Disk info:", disk_info)

-- Get all system stats
local all_stats = agent:use_tool("system_monitor", {
    operation = "collect",
    metrics = {"cpu", "memory", "disk", "network"}
})
print("All system stats:", all_stats)

-- Monitor system for a period
local monitor_result = agent:use_tool("system_monitor", {
    operation = "monitor",
    metrics = {"cpu", "memory"},
    duration_ms = 5000,
    interval_ms = 1000
})
print("Monitor result:", monitor_result)

-- Get system uptime
local uptime = agent:use_tool("system_monitor", {
    operation = "uptime"
})
print("System uptime:", uptime)

-- Get load average
local load_avg = agent:use_tool("system_monitor", {
    operation = "load_average"
})
print("Load average:", load_avg)

-- Get process information
local process_info = agent:use_tool("system_monitor", {
    operation = "processes",
    limit = 10
})
print("Process info:", process_info)

print("\n🔒 Security Controls Demonstrated")
print("=================================")

-- These examples show secure system integration:
print("✅ Command whitelisting - only approved executables allowed")
print("✅ Environment filtering - sensitive variables hidden")
print("✅ Network restrictions - only safe hosts/ports accessible")
print("✅ Resource limits - CPU, memory, and time limits enforced")
print("✅ Sandboxing - operations contained within security boundaries")
print("✅ Audit logging - all operations logged for security review")

print("\n⚠️ Security Considerations")
print("=========================")

local security_notes = [[
1. Process execution is restricted to approved commands
2. Environment variables are filtered to prevent information leakage
3. Network connections are limited to safe hosts and ports
4. All operations are logged for security auditing
5. Resource limits prevent denial of service attacks
6. Timeouts prevent hanging operations
]]

print("Security notes:")
print(security_notes)

print("\n📊 Performance Monitoring")
print("========================")

-- Example of monitoring system performance
local performance_example = [[
1. Collect baseline metrics
2. Monitor during operation
3. Alert on threshold violations
4. Generate performance reports
5. Optimize based on metrics
]]

print("Performance monitoring workflow:")
print(performance_example)

print("\n🔧 Common Administrative Tasks")
print("=============================")

-- Examples of common system administration tasks
local admin_tasks = [[
1. Check system health
2. Monitor resource usage
3. Verify service availability
4. Execute maintenance commands
5. Collect diagnostic information
]]

print("Common admin tasks:")
print(admin_tasks)

print("\n🚨 Error Handling Examples")
print("==========================")

-- Demonstrate error handling for various scenarios
local error_examples = {
    {
        description = "Invalid command",
        operation = "process_executor",
        params = {command = "nonexistent_command"}
    },
    {
        description = "Connection timeout",
        operation = "service_checker",
        params = {operation = "check_tcp", host = "192.168.1.999", port = 80}
    },
    {
        description = "Permission denied",
        operation = "environment_reader",
        params = {operation = "get", variable = "SECRET_KEY"}
    }
}

for i, example in ipairs(error_examples) do
    print(string.format("%d. %s", i, example.description))
    local result = agent:use_tool(example.operation, example.params)
    print("Result:", result)
end

print("\n✅ System Integration Tools Examples Complete!")
print("All operations performed with appropriate security controls.")

return {
    tools_demonstrated = 4,
    categories = "system_integration",
    security_features = {
        "command_whitelisting",
        "environment_filtering",
        "network_restrictions",
        "resource_limits",
        "sandboxing",
        "audit_logging"
    },
    performance_monitoring = true,
    error_handling = true,
    status = "success"
}