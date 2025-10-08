-- ============================================================
-- LLMSPELL ADVANCED PATTERNS SHOWCASE
-- ============================================================
-- Pattern ID: 04 - Monitoring & Security Patterns v0.7.0
-- Complexity Level: ADVANCED
-- Real-World Use Case: Production system monitoring and security
-- Pattern Category: Operations & Security
--
-- Purpose: Demonstrates production monitoring and security patterns
-- Architecture: Agent-based monitoring, security controls, audit logging
-- Key Capabilities:
--   • System health monitoring with agents
--   • File system security and sandboxing
--   • Process execution controls
--   • Anomaly detection patterns
--   • Security audit logging
--   • Threshold-based alerting
--   • Encryption and hashing operations
--   • Access control patterns
--
-- Prerequisites:
--   • Optional: API keys for agent-based monitoring
--   • Understanding of security principles
--
-- HOW TO RUN:
-- ./target/debug/llmspell run examples/script-users/advanced-patterns/monitoring-security.lua
--
-- EXPECTED OUTPUT:
-- Monitoring and security patterns with audit trails
-- Execution time: 2-5 seconds (with optional API calls)
--
-- Time to Complete: 5 seconds
-- Next Steps: See cookbook/webapp-creator.lua for production application
-- ============================================================

print("=== Monitoring & Security Patterns ===\n")

-- Helper for safe tool execution
local function use_tool(tool_name, params)
    local success, result = pcall(function()
        return Tool.execute(tool_name, params)
    end)
    
    if success and result then
        return result
    elseif success then
        return {success = false, error = "Tool returned no result"}
    else
        return {success = false, error = tostring(result)}
    end
end

-- Helper for security test results
local function print_security_test(label, result, should_fail)
    if result.error or result.success == false then
        if should_fail then
            print("   ✓ " .. label .. ": Correctly blocked")
        else
            print("   ✗ " .. label .. ": Unexpected failure")
        end
    else
        if should_fail then
            print("   ✗ " .. label .. ": SECURITY FAILURE - should have been blocked")
        else
            print("   ✓ " .. label .. ": Allowed as expected")
        end
    end
end

-- 1. SYSTEM MONITORING PATTERN
print("1. System Health Monitoring")
print("-" .. string.rep("-", 27))

-- Create monitoring agent if API key available
local monitor_agent = nil
local success, agent = pcall(function()
    return Agent.builder()
        :name("system_monitor")
        :model("openai/gpt-3.5-turbo")
        :temperature(0.2)
        :max_tokens(200)
        :system_prompt([[You are a system monitor. Analyze metrics and identify:
1. Health status (Good/Warning/Critical)
2. Potential issues
3. Optimization recommendations
Be concise and actionable.]])
        :build()
end)

if success and agent then
    monitor_agent = agent
    print("   ✓ Monitor agent created")
else
    print("   ⚠ Monitor agent unavailable (no API key)")
end

-- Simulate system metrics
local metrics = {
    cpu_usage = math.random(10, 90),
    memory_usage = math.random(40, 85),
    disk_usage = math.random(30, 80),
    active_connections = math.random(10, 100),
    error_rate = math.random(0, 5) / 100
}

print(string.format("   Metrics: CPU=%d%% MEM=%d%% DISK=%d%%", 
    metrics.cpu_usage, metrics.memory_usage, metrics.disk_usage))

-- Threshold-based alerting
if metrics.cpu_usage > 80 then
    print("   🚨 ALERT: High CPU usage detected!")
end
if metrics.memory_usage > 75 then
    print("   ⚠️ WARNING: Memory usage approaching limit")
end
if metrics.error_rate > 0.02 then
    print("   🚨 ALERT: Error rate above threshold")
end

-- Agent-based analysis if available
if monitor_agent then
    local analysis = monitor_agent:execute({
        text = string.format("Analyze: CPU=%d%%, Memory=%d%%, Disk=%d%%, Errors=%.2f%%",
            metrics.cpu_usage, metrics.memory_usage, metrics.disk_usage, metrics.error_rate * 100)
    })
    if analysis and analysis.text then
        print("   Agent analysis: " .. analysis.text:sub(1, 100) .. "...")
    end
end

-- 2. FILE SYSTEM SECURITY
print("\n2. File System Security Controls")
print("-" .. string.rep("-", 31))

print("   Testing path traversal prevention...")

-- Test 1: Path traversal attempt
local traversal = use_tool("file_operations", {
    operation = "read",
    path = "/tmp/../../../etc/passwd"
})
print_security_test("Path traversal", traversal, true)

-- Test 2: Restricted directory access
local restricted = use_tool("file_operations", {
    operation = "list_dir",
    path = "/etc"
})
print_security_test("Access /etc", restricted, true)

-- Test 3: Safe sandbox operation
local safe_write = use_tool("file_operations", {
    operation = "write",
    path = "/tmp/security_test.txt",
    input = "Safe content in sandbox"
})
print_security_test("Write to /tmp", safe_write, false)

-- Test 4: System directory write attempt
local system_write = use_tool("file_operations", {
    operation = "write",
    path = "/usr/bin/malicious",
    input = "malicious content"
})
print_security_test("Write to /usr/bin", system_write, true)

-- 3. PROCESS EXECUTION SECURITY
print("\n3. Process Execution Controls")
print("-" .. string.rep("-", 29))

print("   Testing command whitelisting...")

-- Test 1: Whitelisted command (echo)
local safe_cmd = use_tool("process_executor", {
    executable = "echo",
    arguments = {"Security test"}
})
print_security_test("Echo command", safe_cmd, false)

-- Test 2: Dangerous command (rm)
local dangerous_cmd = use_tool("process_executor", {
    executable = "rm",
    arguments = {"-rf", "/"},
    timeout_ms = 1000
})
print_security_test("rm -rf command", dangerous_cmd, true)

-- Test 3: Network command (curl)
local network_cmd = use_tool("process_executor", {
    executable = "curl",
    arguments = {"http://malicious.site"},
    timeout_ms = 1000
})
print_security_test("curl command", network_cmd, true)

-- Test 4: Script execution (sh)
local script_cmd = use_tool("process_executor", {
    executable = "sh",
    arguments = {"-c", "echo 'arbitrary code'"},
    timeout_ms = 1000
})
print_security_test("Shell script", script_cmd, true)

-- 4. ANOMALY DETECTION PATTERN
print("\n4. Anomaly Detection Pattern")
print("-" .. string.rep("-", 28))

-- Track file changes for anomaly detection
local baseline_files = {}

-- Create baseline
print("   Creating baseline...")
for i = 1, 3 do
    local filename = "/tmp/monitor_" .. i .. ".txt"
    use_tool("file_operations", {
        operation = "write",
        path = filename,
        input = "Baseline content " .. i
    })
    
    -- Hash file content for integrity checking
    local hash_result = use_tool("hash_calculator", {
        operation = "hash",
        algorithm = "sha256",
        input = "Baseline content " .. i
    })
    
    if hash_result and hash_result.result and hash_result.result.hash then
        baseline_files[filename] = hash_result.result.hash
        print("   Baseline: " .. filename .. " -> " .. hash_result.result.hash:sub(1, 16) .. "...")
    end
end

-- Simulate file modification (anomaly)
print("   Simulating file modification...")
use_tool("file_operations", {
    operation = "write",
    path = "/tmp/monitor_2.txt",
    input = "Modified content - ANOMALY!"
})

-- Check for anomalies
print("   Checking for anomalies...")
for filename, original_hash in pairs(baseline_files) do
    local read_result = use_tool("file_operations", {
        operation = "read",
        path = filename
    })
    
    if read_result and read_result.result and read_result.result.text then
        local new_hash_result = use_tool("hash_calculator", {
            operation = "hash",
            algorithm = "sha256",
            input = read_result.result.text
        })
        
        if new_hash_result and new_hash_result.result and new_hash_result.result.hash then
            if new_hash_result.result.hash ~= original_hash then
                print("   🚨 ANOMALY: " .. filename .. " has been modified!")
            else
                print("   ✓ " .. filename .. " unchanged")
            end
        end
    end
end

-- 5. AUDIT LOGGING PATTERN
print("\n5. Security Audit Logging")
print("-" .. string.rep("-", 24))

-- Create audit log
local audit_log = {}
local function log_security_event(event_type, details, severity)
    local timestamp = os.date("%Y-%m-%d %H:%M:%S")
    local entry = {
        timestamp = timestamp,
        event_type = event_type,
        severity = severity,
        details = details
    }
    table.insert(audit_log, entry)
    
    -- Also write to file for persistence
    local log_entry = string.format("[%s] %s: %s - %s\n", 
        timestamp, severity, event_type, details)
    
    use_tool("file_operations", {
        operation = "append",
        path = "/tmp/security_audit.log",
        input = log_entry
    })
    
    -- Display based on severity
    if severity == "CRITICAL" then
        print("   🚨 " .. event_type .. ": " .. details)
    elseif severity == "WARNING" then
        print("   ⚠️ " .. event_type .. ": " .. details)
    else
        print("   ℹ️ " .. event_type .. ": " .. details)
    end
end

-- Log various security events
log_security_event("ACCESS_ATTEMPT", "Unauthorized /etc access blocked", "WARNING")
log_security_event("FILE_MODIFIED", "/tmp/monitor_2.txt changed", "WARNING")
log_security_event("PROCESS_BLOCKED", "rm command execution denied", "CRITICAL")
log_security_event("LOGIN_SUCCESS", "User authenticated successfully", "INFO")

-- 6. ENCRYPTION PATTERN
print("\n6. Data Encryption Pattern")
print("-" .. string.rep("-", 25))

-- Sensitive data to protect
local sensitive_data = "SSN: 123-45-6789, Credit Card: 4111-1111-1111-1111"
print("   Original: " .. sensitive_data:sub(1, 30) .. "...")

-- Hash sensitive data (one-way)
local hash_result = use_tool("hash_calculator", {
    operation = "hash",
    algorithm = "sha256",
    input = sensitive_data
})

if hash_result and hash_result.result and hash_result.result.hash then
    print("   SHA256: " .. hash_result.result.hash:sub(1, 32) .. "...")
end

-- Base64 encode (not encryption, but obfuscation)
local encode_result = use_tool("base64_encoder", {
    operation = "encode",
    input = sensitive_data
})

if encode_result and encode_result.result and encode_result.result.encoded then
    print("   Encoded: " .. encode_result.result.encoded:sub(1, 32) .. "...")
    
    -- Decode to verify
    local decode_result = use_tool("base64_encoder", {
        operation = "decode",
        input = encode_result.result.encoded
    })
    
    if decode_result and decode_result.result and decode_result.result.decoded then
        print("   ✓ Decode verified")
    end
end

-- 7. RATE LIMITING FOR SECURITY
print("\n7. Rate Limiting Security")
print("-" .. string.rep("-", 24))

print("   Implementing login rate limiting...")

-- Simulate login attempts
local login_attempts = 0
local blocked_until = 0

for i = 1, 8 do
    local current_time = os.time()
    
    if current_time < blocked_until then
        print("   Attempt " .. i .. ": ✗ Blocked (cooldown)")
    else
        login_attempts = login_attempts + 1
        
        if login_attempts > 3 then
            blocked_until = current_time + 30  -- 30 second cooldown
            print("   Attempt " .. i .. ": ✗ Too many attempts - blocking for 30s")
            login_attempts = 0
        else
            print("   Attempt " .. i .. ": ✓ Login attempt allowed")
        end
    end
end

-- 8. ENVIRONMENT SECURITY
print("\n8. Environment Variable Security")
print("-" .. string.rep("-", 31))

print("   Checking for sensitive variables...")

-- Check for sensitive environment variables
local sensitive_vars = {"API_KEY", "PASSWORD", "SECRET", "TOKEN", "PRIVATE"}

for _, var_pattern in ipairs(sensitive_vars) do
    local env_result = use_tool("environment_reader", {
        operation = "list",
        pattern = var_pattern .. "*"
    })
    
    if env_result and env_result.result then
        -- In production, would mask or redact these
        print("   ⚠️ Found sensitive pattern: " .. var_pattern .. "*")
    end
end

-- Safe environment variables
local safe_result = use_tool("environment_reader", {
    operation = "get",
    variable_name = "PATH"
})

if safe_result and safe_result.result and safe_result.result.value then
    print("   ✓ PATH variable accessible (safe)")
end

-- 9. SECURITY METRICS
print("\n9. Security Metrics Dashboard")
print("-" .. string.rep("-", 28))

local security_metrics = {
    blocked_attempts = 5,
    successful_auth = 12,
    file_violations = 3,
    process_blocks = 4,
    anomalies_detected = 1,
    audit_entries = #audit_log
}

print("   Security Dashboard:")
print("   • Blocked attempts: " .. security_metrics.blocked_attempts)
print("   • Successful auth: " .. security_metrics.successful_auth)
print("   • File violations: " .. security_metrics.file_violations)
print("   • Process blocks: " .. security_metrics.process_blocks)
print("   • Anomalies: " .. security_metrics.anomalies_detected)
print("   • Audit entries: " .. security_metrics.audit_entries)

-- Calculate security score
local security_score = 100
security_score = security_score - (security_metrics.file_violations * 5)
security_score = security_score - (security_metrics.anomalies_detected * 10)
security_score = math.max(0, security_score)

print(string.format("   Overall Security Score: %d/100", security_score))

-- 10. BEST PRACTICES
print("\n10. Monitoring & Security Best Practices")
print("-" .. string.rep("-", 39))

print("   • Implement defense in depth")
print("   • Use threshold-based alerting")
print("   • Maintain comprehensive audit logs")
print("   • Monitor for anomalies continuously")
print("   • Enforce strict sandboxing")
print("   • Whitelist allowed operations")
print("   • Rate limit sensitive operations")
print("   • Encrypt sensitive data at rest")
print("   • Regular security metric reviews")
print("   • Automate incident response")

print("\n=== Monitoring & Security Patterns Complete ===")
print("Demonstrated: Monitoring, Sandboxing, Auditing, Anomaly Detection, Encryption")
print("Security Score: " .. security_score .. "/100")
print("Next: See cookbook examples for production applications")