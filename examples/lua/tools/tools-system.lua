-- tools-system.lua
-- Examples for system integration tools with security controls
-- Using direct Tool API

print("🖥️ System Integration Tools Examples")
print("====================================")

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

-- Helper for security tests
local function print_security_test(label, result, should_fail)
    if result.error then
        if should_fail then
            print("  ✅ " .. label .. ": Correctly blocked - " .. result.error)
        else
            print("  ❌ " .. label .. ": Unexpected error - " .. result.error)
        end
    elseif result.success == false then
        if should_fail then
            print("  ✅ " .. label .. ": Correctly blocked - " .. (result.message or "Access denied"))
        else
            print("  ❌ " .. label .. ": Unexpected failure - " .. (result.message or "Failed"))
        end
    else
        if should_fail then
            print("  ❌ " .. label .. ": Security failure - operation should have been blocked")
        else
            print("  ✅ " .. label .. ": Success")
        end
    end
end

print("Environment Reader Tool")

print("\nEnvironment operations:")

-- Get specific environment variable
local path_var = use_tool("environment_reader", {
    operation = "get",
    variable_name = "PATH"
})
print_result("PATH variable", path_var)

-- Get home directory
local home_var = use_tool("environment_reader", {
    operation = "get",
    variable_name = "HOME"
})
print_result("HOME variable", home_var)

-- List environment variables (filtered for security)
local env_list = use_tool("environment_reader", {
    operation = "list",
    pattern = "SHELL*"
})
print_result("Shell variables", env_list)

-- List all allowed environment variables
local all_vars = use_tool("environment_reader", {
    operation = "list"
})
print_result("All allowed vars", all_vars)

print("Process Executor Tool")

print("\nProcess execution (sandboxed):")

-- Execute simple command
local simple_command = use_tool("process_executor", {
    executable = "echo",
    arguments = {"Hello", "from", "LLMSpell!"}
})
print_result("Echo command", simple_command)

-- List files (if ls is whitelisted)
local list_files = use_tool("process_executor", {
    executable = "ls",
    arguments = {"-la", "/tmp"},
    timeout_ms = 5000
})
print_result("List files", list_files)

-- Get current user (if whoami is whitelisted)
local whoami = use_tool("process_executor", {
    executable = "whoami"
})
print_result("Current user", whoami)

-- Execute with timeout
local timeout_command = use_tool("process_executor", {
    executable = "sleep",
    arguments = {"2"},
    timeout_ms = 1000
})
-- Note: sleep is not whitelisted so this tests security blocking, not timeout
print_security_test("Sleep command blocked", timeout_command, true)

print("Service Checker Tool")

print("\nService availability checks:")

-- Check TCP port
local tcp_check = use_tool("service_checker", {
    check_type = "tcp",
    target = "127.0.0.1:22",
    timeout_ms = 1000
})
print_result("SSH port (22)", tcp_check)

-- Check common web port
local http_check = use_tool("service_checker", {
    check_type = "tcp",
    target = "127.0.0.1:80",
    timeout_ms = 1000
})
print_result("HTTP port (80)", http_check)

-- Check HTTPS port
local https_check = use_tool("service_checker", {
    check_type = "tcp",
    target = "127.0.0.1:443",
    timeout_ms = 1000
})
print_result("HTTPS port (443)", https_check)

-- Check custom service
local custom_check = use_tool("service_checker", {
    check_type = "tcp",
    target = "localhost:8080",
    timeout_ms = 1000
})
print_result("Custom port (8080)", custom_check)

print("System Monitor Tool")

print("\nSystem resource monitoring:")

-- Get CPU information
local cpu_info = use_tool("system_monitor", {
    operation = "cpu"
})
print_result("CPU usage", cpu_info)

-- Get memory information
local memory_info = use_tool("system_monitor", {
    operation = "memory"
})
print_result("Memory usage", memory_info)

-- Get disk information
local disk_info = use_tool("system_monitor", {
    operation = "disk"
})
print_result("Disk usage", disk_info)

-- Get all system stats
local all_stats = use_tool("system_monitor", {
    operation = "all"
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
print("\nTesting security controls (these should be blocked):")

-- Invalid command (should be blocked)
local invalid_cmd = use_tool("process_executor", {
    executable = "rm",
    arguments = {"-rf", "/"},  -- This dangerous command should be blocked
    timeout_ms = 1000
})
print_security_test("Dangerous command blocked", invalid_cmd, true)

-- Invalid network target
local invalid_network = use_tool("service_checker", {
    check_type = "tcp",
    target = "999.999.999.999:80",
    timeout_ms = 1000
})
print_result("Invalid IP", invalid_network)  -- This is an actual error, not security

-- Restricted environment variable
local restricted_var = use_tool("environment_reader", {
    operation = "get",
    variable_name = "AWS_SECRET_ACCESS_KEY"  -- Should be filtered
})
print_security_test("Sensitive var blocked", restricted_var, true)

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