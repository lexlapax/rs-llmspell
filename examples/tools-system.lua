-- tools-system.lua
-- Examples for system integration tools with security controls
-- Using direct Tool API

print("🖥️ System Integration Tools Examples")
print("====================================")

-- Load test helpers
local TestHelpers = dofile("examples/test-helpers.lua")

-- Helper function to execute tool
local function use_tool(tool_name, params)
    return TestHelpers.execute_tool(tool_name, params)
end

-- Helper to print clean results
local function print_result(label, result)
    if result.error then
        print("  ❌ " .. label .. ": " .. result.error)
    elseif result.success == false then
        print("  ❌ " .. label .. ": " .. (result.message or "Failed"))
    else
        -- Extract relevant value from result
        local r = result.result or result
        if r.value then
            print("  ✅ " .. label .. ": " .. tostring(r.value))
        elseif r.available then
            print("  ✅ " .. label .. ": Port " .. (r.port or "?") .. " is available")
        elseif r.cpu_usage then
            print("  ✅ " .. label .. ": CPU: " .. string.format("%.1f%%", r.cpu_usage))
        elseif r.memory then
            print("  ✅ " .. label .. ": Memory usage: " .. string.format("%.1f%%", r.memory.percentage or 0))
        elseif result.stdout then
            local output = result.stdout:gsub("\n", " "):sub(1, 50)
            print("  ✅ " .. label .. ": " .. output .. (string.len(result.stdout) > 50 and "..." or ""))
        elseif result.operation then
            print("  ✅ " .. label .. ": " .. result.operation .. " completed")
        else
            print("  ✅ " .. label .. ": Success")
        end
    end
end

TestHelpers.print_section("Environment Reader Tool")

print("\nEnvironment operations:")

-- Get specific environment variable
local path_var = use_tool("environment_reader", {
    operation = "get",
    variable = "PATH"
})
print_result("PATH variable", path_var)

-- Get home directory
local home_var = use_tool("environment_reader", {
    operation = "get",
    variable = "HOME"
})
print_result("HOME variable", home_var)

-- List environment variables (filtered for security)
local env_list = use_tool("environment_reader", {
    operation = "list",
    pattern = "SHELL*"
})
print_result("Shell variables", env_list)

-- Get system information
local system_info = use_tool("environment_reader", {
    operation = "system_info"
})
print_result("System info", system_info)

TestHelpers.print_section("Process Executor Tool")

print("\nProcess execution (sandboxed):")

-- Execute simple command
local simple_command = use_tool("process_executor", {
    command = "echo Hello from LLMSpell!"
})
print_result("Echo command", simple_command)

-- List files (if ls is whitelisted)
local list_files = use_tool("process_executor", {
    command = "ls -la /tmp",
    timeout_ms = 5000
})
print_result("List files", list_files)

-- Get current user (if whoami is whitelisted)
local whoami = use_tool("process_executor", {
    command = "whoami"
})
print_result("Current user", whoami)

-- Execute with timeout
local timeout_command = use_tool("process_executor", {
    command = "sleep 2",
    timeout_ms = 1000
})
print_result("Timeout test", timeout_command)

TestHelpers.print_section("Service Checker Tool")

print("\nService availability checks:")

-- Check TCP port
local tcp_check = use_tool("service_checker", {
    target = "127.0.0.1:22",
    timeout = 1
})
print_result("SSH port (22)", tcp_check)

-- Check common web port
local http_check = use_tool("service_checker", {
    target = "127.0.0.1:80",
    timeout = 1
})
print_result("HTTP port (80)", http_check)

-- Check HTTPS port
local https_check = use_tool("service_checker", {
    target = "127.0.0.1:443",
    timeout = 1
})
print_result("HTTPS port (443)", https_check)

-- Check custom service
local custom_check = use_tool("service_checker", {
    target = "localhost:8080",
    timeout = 1
})
print_result("Custom port (8080)", custom_check)

TestHelpers.print_section("System Monitor Tool")

print("\nSystem resource monitoring:")

-- Get CPU information
local cpu_info = use_tool("system_monitor", {
    operation = "collect",
    metrics = {"cpu"}
})
print_result("CPU usage", cpu_info)

-- Get memory information
local memory_info = use_tool("system_monitor", {
    operation = "collect",
    metrics = {"memory"}
})
print_result("Memory usage", memory_info)

-- Get disk information
local disk_info = use_tool("system_monitor", {
    operation = "collect",
    metrics = {"disk"}
})
print_result("Disk usage", disk_info)

-- Get all system stats
local all_stats = use_tool("system_monitor", {
    operation = "collect",
    metrics = {"cpu", "memory", "disk"}
})
if all_stats.success ~= false then
    print("  ✅ All stats: Collected successfully")
    if all_stats.result then
        local r = all_stats.result
        if r.cpu_usage then
            print("     CPU: " .. string.format("%.1f%%", r.cpu_usage))
        end
        if r.memory and r.memory.percentage then
            print("     Memory: " .. string.format("%.1f%%", r.memory.percentage))
        end
        if r.disks and #r.disks > 0 then
            print("     Disks: " .. #r.disks .. " mounted")
        end
    end
else
    print_result("All stats", all_stats)
end

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

print(security_notes)

print("\n🚨 Error Handling Examples")
print("==========================")

-- Demonstrate error handling for various scenarios
print("\nTesting error conditions:")

-- Invalid command (should be blocked)
local invalid_cmd = use_tool("process_executor", {
    command = "rm -rf /",  -- This dangerous command should be blocked
    timeout_ms = 1000
})
print_result("Dangerous command", invalid_cmd)

-- Invalid network target
local invalid_network = use_tool("service_checker", {
    target = "999.999.999.999:80",
    timeout = 1
})
print_result("Invalid IP", invalid_network)

-- Restricted environment variable
local restricted_var = use_tool("environment_reader", {
    operation = "get",
    variable = "AWS_SECRET_ACCESS_KEY"  -- Should be filtered
})
print_result("Sensitive var", restricted_var)

print("\n✅ System Integration Tools Examples Complete!")
print("All operations performed with appropriate security controls.")

-- Summary
local tools_tested = {
    "environment_reader",
    "process_executor",
    "service_checker",
    "system_monitor"
}

print("\n📊 Summary:")
print("  Tools tested: " .. #tools_tested)
for _, tool in ipairs(tools_tested) do
    print("    - " .. tool)
end

return {
    tools_demonstrated = #tools_tested,
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