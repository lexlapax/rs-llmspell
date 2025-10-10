-- ============================================================
-- LLMSPELL ADVANCED PATTERNS SHOWCASE
-- ============================================================
-- Pattern ID: 03 - Tool Integration Patterns v0.7.0
-- Complexity Level: ADVANCED
-- Real-World Use Case: Enterprise system integration and automation
-- Pattern Category: Tool Integration
--
-- Purpose: Demonstrates advanced tool usage and integration patterns
-- Architecture: Tool chaining, system integration, external services
-- Key Capabilities:
--   • Tool composition and chaining
--   • External service integration (email, database)
--   • System tool usage with security
--   • Rate limiting and circuit breakers
--   • Error handling in tool chains
--   • Custom tool patterns
--   • Tool result transformation
--
-- Prerequisites:
--   • Understanding of tool basics (see features/tool-basics.lua)
--   • Optional: External service credentials for full functionality
--
-- HOW TO RUN:
-- ./target/debug/llmspell run examples/script-users/advanced-patterns/tool-integration-patterns.lua
--
-- EXPECTED OUTPUT:
-- Tool integration patterns with system and external services
-- Execution time: 1-3 seconds (mostly local operations)
--
-- Time to Complete: 3 seconds
-- Next Steps: See monitoring-security.lua for production patterns
-- ============================================================

print("=== Tool Integration Patterns ===\n")

-- Helper function for safe tool invocation
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

-- Helper to print tool results
local function print_result(label, result, indent)
    indent = indent or ""
    if result.error then
        print(indent .. "   ✗ " .. label .. ": " .. tostring(result.error))
    elseif result.success == false then
        print(indent .. "   ✗ " .. label .. ": " .. (result.message or "Failed"))
    else
        local r = result.result or result
        if r.text then
            print(indent .. "   ✓ " .. label .. ": " .. r.text:sub(1, 50) .. (string.len(r.text or "") > 50 and "..." or ""))
        elseif r.uuid then
            print(indent .. "   ✓ " .. label .. ": " .. r.uuid)
        elseif r.hash then
            print(indent .. "   ✓ " .. label .. ": " .. r.hash:sub(1, 16) .. "...")
        else
            print(indent .. "   ✓ " .. label .. ": Success")
        end
    end
end

-- 1. TOOL CHAINING PATTERN
print("1. Tool Chaining Pattern")
print("-" .. string.rep("-", 23))

-- Chain: Generate → Hash → Encode → Save
print("   Chain: UUID → Hash → Encode → Save")

-- Step 1: Generate UUID
local uuid_result = use_tool("uuid-generator", {
    operation = "generate",
    version = "v4"
})

if uuid_result and uuid_result.result and uuid_result.result.uuid then
    print_result("Generated UUID", uuid_result)
    
    -- Step 2: Hash the UUID
    local hash_result = use_tool("hash-calculator", {
        operation = "hash",
        algorithm = "sha256",
        input = uuid_result.result.uuid
    })
    
    if hash_result and hash_result.result and hash_result.result.hash then
        print_result("Hashed UUID", hash_result)
        
        -- Step 3: Encode the hash
        local encode_result = use_tool("base64-encoder", {
            operation = "encode",
            input = hash_result.result.hash
        })
        
        if encode_result and encode_result.result and encode_result.result.encoded then
            print_result("Encoded hash", encode_result)
            
            -- Step 4: Save to file
            local save_result = use_tool("file-operations", {
                operation = "write",
                path = "/tmp/tool_chain_result.txt",
                input = encode_result.result.encoded
            })
            print_result("Saved to file", save_result)
        end
    end
end

-- 2. PARALLEL TOOL EXECUTION
print("\n2. Parallel Tool Pattern")
print("-" .. string.rep("-", 23))

print("   Executing multiple tools independently...")

-- Execute multiple independent tools
local tools_to_execute = {
    {name = "UUID", tool = "uuid-generator", params = {operation = "generate", version = "v4"}},
    {name = "Timestamp", tool = "datetime-handler", params = {operation = "now"}},
    {name = "Calculator", tool = "calculator", params = {operation = "evaluate", input = "42 * 17 + 89"}},
    {name = "Hash", tool = "hash-calculator", params = {operation = "hash", algorithm = "md5", input = "test"}}
}

local results = {}
for _, tool_config in ipairs(tools_to_execute) do
    results[tool_config.name] = use_tool(tool_config.tool, tool_config.params)
    print_result(tool_config.name, results[tool_config.name])
end

-- 3. SYSTEM INTEGRATION PATTERN
print("\n3. System Integration Pattern")
print("-" .. string.rep("-", 29))

-- Environment reading
print("   Reading environment variables...")
local env_result = use_tool("environment_reader", {
    operation = "get",
    variable_name = "HOME"
})
print_result("HOME directory", env_result)

-- Safe process execution (echo is whitelisted)
print("   Executing safe process...")
local process_result = use_tool("process_executor", {
    executable = "echo",
    arguments = {"Tool integration test"}
})
print_result("Process output", process_result)

-- Service checking
print("   Checking service availability...")
local service_result = use_tool("service_checker", {
    check_type = "tcp",
    target = "127.0.0.1:8080",
    timeout_ms = 1000
})
print_result("Port 8080 check", service_result)

-- 4. DATABASE INTEGRATION PATTERN
print("\n4. Database Integration Pattern (Demo)")
print("-" .. string.rep("-", 38))

print("   Database operations (requires configuration)...")
print("   Note: Database connector requires proper configuration")
print("   Example operations that would work with config:")
print("   • CREATE TABLE for schema setup")
print("   • INSERT for data persistence")
print("   • SELECT for data retrieval")
print("   • UPDATE for modifications")
print("   • DELETE for cleanup")

-- 5. EMAIL INTEGRATION PATTERN
print("\n5. Email Integration Pattern (Demo)")
print("-" .. string.rep("-", 35))

print("   Email operations (requires configuration)...")
print("   Note: Email sender requires SMTP/API credentials")
print("   Supported providers:")
print("   • SMTP - Standard email protocol")
print("   • SendGrid - Cloud email service")
print("   • AWS SES - Amazon email service")
print("   • Mailgun - Developer-friendly API")

-- 6. ERROR RECOVERY PATTERN
print("\n6. Error Recovery in Tool Chains")
print("-" .. string.rep("-", 32))

print("   Testing error recovery...")

-- Try to read non-existent file
local read_result = use_tool("file-operations", {
    operation = "read",
    path = "/tmp/nonexistent_file.txt"
})

if read_result.error or read_result.success == false then
    print("   File not found, creating with default content...")
    
    -- Recover by creating the file
    local create_result = use_tool("file-operations", {
        operation = "write",
        path = "/tmp/nonexistent_file.txt",
        input = "Default content created during error recovery"
    })
    
    if create_result and create_result.success ~= false then
        -- Now read the created file
        local retry_result = use_tool("file-operations", {
            operation = "read",
            path = "/tmp/nonexistent_file.txt"
        })
        print_result("Recovery successful", retry_result)
    end
else
    print_result("File exists", read_result)
end

-- 7. RATE LIMITING PATTERN
print("\n7. Rate Limiting Pattern (Simulated)")
print("-" .. string.rep("-", 36))

print("   Simulating rate limiting...")

-- Simulate rate limiting with local logic
local call_count = 0
local max_calls = 5
local window_start = os.time()

for i = 1, 7 do
    call_count = call_count + 1
    
    if call_count <= max_calls then
        print("   Call " .. i .. ": ✓ Allowed (" .. call_count .. "/" .. max_calls .. ")")
    else
        print("   Call " .. i .. ": ✗ Rate limited (exceeded " .. max_calls .. " calls)")
    end
end

print("   Rate limiting prevents API abuse")

-- 8. CIRCUIT BREAKER PATTERN
print("\n8. Circuit Breaker Pattern (Simulated)")
print("-" .. string.rep("-", 38))

print("   Simulating circuit breaker...")

-- Simulate circuit breaker logic
local failure_count = 0
local failure_threshold = 3
local circuit_open = false

for i = 1, 5 do
    -- Simulate failures on calls 2, 3, 4
    local should_fail = (i >= 2 and i <= 4)
    
    if circuit_open then
        print("   Call " .. i .. ": ✗ Circuit OPEN - request blocked")
    elseif should_fail then
        failure_count = failure_count + 1
        print("   Call " .. i .. ": ✗ Failed (" .. failure_count .. "/" .. failure_threshold .. ")")
        
        if failure_count >= failure_threshold then
            circuit_open = true
            print("   Circuit breaker OPENED after " .. failure_threshold .. " failures")
        end
    else
        print("   Call " .. i .. ": ✓ Success")
        failure_count = 0  -- Reset on success
    end
end

-- 9. TOOL COMPOSITION PATTERN
print("\n9. Tool Composition Pattern")
print("-" .. string.rep("-", 27))

-- Create a composite operation using multiple tools
local function create_secure_document(content, filename)
    print("   Creating secure document: " .. filename)
    
    -- Generate document ID
    local id_result = use_tool("uuid-generator", {
        operation = "generate",
        version = "v4"
    })
    
    if not (id_result and id_result.result and id_result.result.uuid) then
        return {success = false, error = "Failed to generate ID"}
    end
    
    -- Get timestamp
    local time_result = use_tool("datetime-handler", {
        operation = "now"
    })
    
    -- Create document with metadata
    local doc_content = string.format(
        "Document ID: %s\nCreated: %s\n---\n%s",
        id_result.result.uuid,
        time_result and time_result.result and time_result.result.timestamp or "unknown",
        content
    )
    
    -- Hash for integrity
    local hash_result = use_tool("hash-calculator", {
        operation = "hash",
        algorithm = "sha256",
        input = doc_content
    })
    
    -- Add hash to document
    doc_content = doc_content .. "\n---\nSHA256: " .. (hash_result and hash_result.result and hash_result.result.hash or "error")
    
    -- Save document
    local save_result = use_tool("file-operations", {
        operation = "write",
        path = "/tmp/" .. filename,
        input = doc_content
    })
    
    return save_result
end

local doc_result = create_secure_document("This is a secure document with integrity checking.", "secure_doc.txt")
print_result("Secure document created", doc_result)

-- 10. BEST PRACTICES
print("\n10. Tool Integration Best Practices")
print("-" .. string.rep("-", 35))

print("   • Always handle tool failures gracefully")
print("   • Chain tools for complex operations")
print("   • Use parallel execution for independent operations")
print("   • Implement rate limiting for external services")
print("   • Use circuit breakers for unreliable services")
print("   • Validate tool inputs and outputs")
print("   • Create composite tools for reusable patterns")
print("   • Monitor tool performance and errors")
print("   • Document tool dependencies clearly")
print("   • Test error recovery paths")

print("\n=== Tool Integration Patterns Complete ===")
print("Demonstrated: Chaining, Parallel, System, Database, Email, Recovery patterns")
print("Next: Explore monitoring-security.lua for production monitoring")