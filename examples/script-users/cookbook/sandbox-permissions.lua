-- Recommended profile: development
-- Run with: llmspell -p development run sandbox-permissions.lua
-- Development environment with debug logging

-- ============================================================
-- LLMSPELL COOKBOOK SHOWCASE
-- ============================================================
-- Pattern ID: 11 - Sandbox Permissions v0.11.0
-- Complexity Level: INTERMEDIATE
-- Real-World Use Case: Configuring security sandbox permissions
-- Pattern Category: Security & Configuration
--
-- Purpose: Demonstrate how to configure and work with security sandbox
--          permissions including network access, process execution, and
--          file system access. Essential for production deployments.
-- Architecture: Defense-in-depth with explicit permission configuration
-- Crates Showcased: llmspell-security, llmspell-config, llmspell-tools
-- Key Features:
--   • Network domain allowlisting
--   • Process execution control
--   • File system path restrictions
--   • Permission error handling
--   • Security best practices
--
-- Prerequisites:
--   • LLMSpell installed and built
--   • No API keys required
--   • config.toml with security settings (see below)
--
-- Configuration Required (config.toml):
--   [tools.http_request]
--   allowed_hosts = ["httpbin.org"]
--   blocked_hosts = ["localhost", "127.0.0.1"]
--
--   [tools.system]
--   allow_process_execution = true
--   allowed_commands = "echo,date,pwd"
--
--   [tools.file_operations]
--   allowed_paths = ["/tmp"]
--   blocked_extensions = ["exe", "dll"]
--
-- HOW TO RUN:
-- ./target/debug/llmspell run examples/script-users/cookbook/sandbox-permissions.lua
--
-- EXPECTED OUTPUT:
-- 4 permission scenarios demonstrated with success/failure handling
--
-- Time to Complete: <5 seconds
-- Production Notes: Always use principle of least privilege,
--                   configure allowlists not denylists,
--                   monitor security violation logs.
-- ============================================================

print("=== Sandbox Permissions Demo ===")
print("Pattern 11: INTERMEDIATE - Security sandbox configuration\n")

-- ============================================================
-- Scenario 1: Network Access Permissions
-- ============================================================

print("1. Network Access Permissions")
print("-" .. string.rep("-", 40))

if Config.isNetworkAccessAllowed() then
    print("   Network access: ENABLED\n")

    -- Test 1: Try allowed domain (httpbin.org)
    print("   Test 1: Allowed domain (httpbin.org)")
    local success, result = pcall(function()
        return Tool.execute("http-request", {
            method = "GET",
            url = "https://httpbin.org/get",
            timeout_ms = 5000
        })
    end)

    if success and result and result.success then
        print("     ✅ SUCCESS: Request to httpbin.org allowed")
    elseif success and result then
        print("     ⚠️  ALLOWED but failed: " .. tostring(result.error or "Network error"))
    else
        print("     ❌ BLOCKED: " .. tostring(result))
        if string.match(tostring(result), "Host blocked") or
           string.match(tostring(result), "not in allowed") then
            print("     Fix: Add 'httpbin.org' to [tools.http_request].allowed_hosts")
        end
    end

    -- Test 2: Try blocked domain (example.com - not in allowlist)
    print("\n   Test 2: Blocked domain (example.com)")
    local blocked_success, blocked_result = pcall(function()
        return Tool.execute("http-request", {
            method = "GET",
            url = "https://example.com",
            timeout_ms = 5000
        })
    end)

    if not blocked_success or (blocked_result and not blocked_result.success) then
        print("     ✅ CORRECTLY BLOCKED: Domain not in allowlist")
        print("     Note: Add to [tools.http_request].allowed_hosts to allow")
    else
        print("     ⚠️  WARNING: Should have been blocked")
    end

else
    print("   ❌ Network access: DISABLED")
    print("   Enable with:")
    print("   [tools.http_request]")
    print("   allowed_hosts = [\"httpbin.org\", \"*.example.com\"]")
end

-- ============================================================
-- Scenario 2: Process Execution Permissions
-- ============================================================

print("\n2. Process Execution Permissions")
print("-" .. string.rep("-", 40))

local can_execute = Config.get("tools.system.allow_process_execution")
if can_execute then
    print("   Process execution: ENABLED\n")

    -- Test 1: Try allowed command (echo)
    print("   Test 1: Allowed command (echo)")
    local success, result = pcall(function()
        return Tool.execute("process-executor", {
            executable = "echo",
            arguments = {"Hello from sandbox!"},
            timeout_ms = 1000
        })
    end)

    if success and result and result.success then
        print("     ✅ SUCCESS: echo command executed")
        if result.result and result.result.output then
            print("     Output: " .. result.result.output)
        end
    else
        local error_msg = tostring(result)
        print("     ❌ BLOCKED: " .. error_msg)
        if string.match(error_msg, "Command blocked") or
           string.match(error_msg, "not allowed") then
            print("     Fix: Add 'echo' to [tools.system].allowed_commands")
        end
    end

    -- Test 2: Try blocked command (curl - dangerous)
    print("\n   Test 2: Blocked command (curl)")
    local curl_success, curl_result = pcall(function()
        return Tool.execute("process-executor", {
            executable = "curl",
            arguments = {"https://example.com"},
            timeout_ms = 1000
        })
    end)

    if not curl_success or (curl_result and not curl_result.success) then
        print("     ✅ CORRECTLY BLOCKED: curl is dangerous (network access)")
        print("     Note: Use http-request tool instead for safety")
    else
        print("     ⚠️  WARNING: Should have been blocked")
    end

    -- Test 3: Try another blocked command (rm - very dangerous)
    print("\n   Test 3: Blocked command (rm)")
    local rm_success, rm_result = pcall(function()
        return Tool.execute("process-executor", {
            executable = "rm",
            arguments = {"-rf", "/tmp/test"},
            timeout_ms = 1000
        })
    end)

    if not rm_success or (rm_result and not rm_result.success) then
        print("     ✅ CORRECTLY BLOCKED: rm is dangerous (file deletion)")
        print("     Note: NEVER add 'rm' to allowed_commands")
    else
        print("     ❌ SECURITY FAILURE: rm should ALWAYS be blocked")
    end

else
    print("   ❌ Process execution: DISABLED")
    print("   Enable with (use caution):")
    print("   [tools.system]")
    print("   allow_process_execution = true")
    print("   allowed_commands = \"echo,date,pwd\"  # Minimal safe commands")
end

-- ============================================================
-- Scenario 3: File System Permissions
-- ============================================================

print("\n3. File System Permissions")
print("-" .. string.rep("-", 40))

if Config.isFileAccessAllowed() then
    print("   File access: ENABLED\n")

    -- Test 1: Try allowed path (/tmp)
    print("   Test 1: Allowed path (/tmp)")
    local success, result = pcall(function()
        return Tool.execute("file-operations", {
            operation = "write",
            path = "/tmp/sandbox-test.txt",
            input = "Sandbox permission test at " .. os.date()
        })
    end)

    if success and result and result.success then
        print("     ✅ SUCCESS: Write to /tmp allowed")

        -- Read back to verify
        local read_success, read_result = pcall(function()
            return Tool.execute("file-operations", {
                operation = "read",
                path = "/tmp/sandbox-test.txt"
            })
        end)

        if read_success and read_result and read_result.success then
            print("     ✅ Read back successful")
        end
    else
        print("     ❌ BLOCKED: " .. tostring(result))
        if string.match(tostring(result), "not in allowlist") or
           string.match(tostring(result), "Path not allowed") then
            print("     Fix: Add '/tmp' to [tools.file_operations].allowed_paths")
        end
    end

    -- Test 2: Try blocked path (/etc/passwd)
    print("\n   Test 2: Blocked path (/etc/passwd)")
    local etc_success, etc_result = pcall(function()
        return Tool.execute("file-operations", {
            operation = "read",
            path = "/etc/passwd"
        })
    end)

    if not etc_success or (etc_result and not etc_result.success) then
        print("     ✅ CORRECTLY BLOCKED: /etc/passwd is system file")
        print("     Note: NEVER add system paths to allowed_paths")
    else
        print("     ❌ SECURITY FAILURE: System files should be blocked")
    end

    -- Test 3: Try path traversal attack
    print("\n   Test 3: Path traversal attack (../etc/passwd)")
    local traversal_success, traversal_result = pcall(function()
        return Tool.execute("file-operations", {
            operation = "read",
            path = "/tmp/../etc/passwd"
        })
    end)

    if not traversal_success or (traversal_result and not traversal_result.success) then
        print("     ✅ CORRECTLY BLOCKED: Path traversal detected")
        print("     Note: Sandbox prevents ../ attacks automatically")
    else
        print("     ❌ SECURITY FAILURE: Path traversal should be blocked")
    end

else
    print("   ❌ File access: DISABLED")
    print("   Enable with:")
    print("   [tools.file_operations]")
    print("   allowed_paths = [\"/tmp\", \"/workspace\"]")
    print("   blocked_extensions = [\"exe\", \"dll\", \"so\"]")
end

-- ============================================================
-- Scenario 4: Permission Error Handling Pattern
-- ============================================================

print("\n4. Permission Error Handling Pattern")
print("-" .. string.rep("-", 40))

print("   Demonstrating comprehensive error handling:\n")

-- Helper function: Safe tool execution with detailed error reporting
local function safe_tool_execute(tool_name, params)
    local success, result = pcall(function()
        return Tool.execute(tool_name, params)
    end)

    if not success then
        local error_msg = tostring(result)

        -- Network errors
        if error_msg:match("Domain not in allowed") or
           error_msg:match("Host blocked") then
            print("     ❌ Network Permission Error")
            print("     Solution: Add domain to [tools.http_request].allowed_hosts")
            return false, "network_permission"

        -- File access errors
        elseif error_msg:match("Path not in allowlist") or
               error_msg:match("Path not allowed") then
            print("     ❌ File Permission Error")
            print("     Solution: Add path to [tools.file_operations].allowed_paths")
            return false, "file_permission"

        -- Process execution errors
        elseif error_msg:match("Command blocked") or
               error_msg:match("Executable not allowed") then
            print("     ❌ Process Permission Error")
            print("     Solution: Add command to [tools.system].allowed_commands")
            return false, "process_permission"

        -- Generic permission error
        elseif error_msg:match("Permission denied") then
            print("     ❌ Generic Permission Error")
            print("     Solution: Check security settings in config.toml")
            return false, "generic_permission"

        -- Other error
        else
            print("     ❌ Tool Error: " .. error_msg)
            return false, "tool_error"
        end

    elseif result and not result.success then
        print("     ⚠️  Tool failed: " .. tostring(result.error or "Unknown"))
        return false, "tool_failed"
    else
        return true, result
    end
end

-- Example: Try operation that might fail
print("   Testing safe_tool_execute helper:")
local worked, result_or_type = safe_tool_execute("file-operations", {
    operation = "read",
    path = "/tmp/sandbox-test.txt"
})

if worked then
    print("     ✅ Operation succeeded")
else
    print("     Error type: " .. result_or_type)
end

-- ============================================================
-- Security Best Practices Summary
-- ============================================================

print("\n=== Security Best Practices ===")
print("1. Always check permissions before use:")
print("   if Config.isNetworkAccessAllowed() then ... end")
print("")
print("2. Use pcall() to catch and handle permission errors:")
print("   local success, result = pcall(function() ... end)")
print("")
print("3. Configure minimal permissions (principle of least privilege):")
print("   - Only paths you need")
print("   - Only domains you access")
print("   - Only commands you use")
print("")
print("4. Use allowlists, not denylists:")
print("   allowed_hosts = [\"trusted.com\"]  # Good")
print("   # NOT: blocked_hosts = [\"evil.com\"]  # Incomplete")
print("")
print("5. Monitor security violation logs:")
print("   Check logs for unexpected permission requests")
print("")
print("6. Document required permissions in script header:")
print("   -- REQUIRED CONFIG:")
print("   -- [tools.network]")
print("   -- allowed_domains = [\"api.example.com\"]")

-- ============================================================
-- Configuration Reference
-- ============================================================

print("\n=== Configuration Reference ===")
print("Network access:")
print("  [tools.http_request]")
print("  allowed_hosts = [\"api.example.com\", \"*.github.com\"]")
print("")
print("Process execution:")
print("  [tools.system]")
print("  allow_process_execution = true")
print("  allowed_commands = \"echo,cat,ls,pwd\"")
print("")
print("File system:")
print("  [tools.file_operations]")
print("  allowed_paths = [\"/tmp\", \"/workspace\"]")
print("  blocked_extensions = [\"exe\", \"dll\", \"so\"]")
print("")
print("See docs/user-guide/security-and-permissions.md for complete guide")

-- ============================================================
-- Pattern Complete
-- ============================================================

print("\n=== Sandbox Permissions Demo Complete ===")
print("Demonstrated:")
print("  • Network domain allowlisting")
print("  • Process execution control")
print("  • File system path restrictions")
print("  • Permission error handling")
print("  • Security best practices")
print("")
print("Next: Configure your config.toml with minimal required permissions")
