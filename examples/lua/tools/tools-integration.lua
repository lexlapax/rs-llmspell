-- tools-integration.lua
-- Examples for external integration tools (Email, Database, Rate Limiting, Circuit Breaker)
-- Using direct Tool API

print("🔌 External Integration Tools Examples")
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
    -- Check if the tool returned an error in the output
    if result.output and type(result.output) == "string" then
        local ok, parsed = pcall(function() return JSON.parse(result.output) end)
        if ok and parsed.success == false then
            print("  ❌ " .. label .. ": " .. (parsed.error and parsed.error.message or "Failed"))
            return
        end
    end
    
    if result.error then
        print("  ❌ " .. label .. ": " .. (result.error.message or result.error))
    elseif result.success == false then
        print("  ❌ " .. label .. ": " .. (result.message or result.error or "Failed"))
    else
        -- Extract relevant output
        local r = result.result or result
        if r.message_id then
            print("  ✅ " .. label .. ": Message ID=" .. r.message_id)
        elseif r.rows_affected ~= nil then
            print("  ✅ " .. label .. ": Rows affected=" .. r.rows_affected)
        elseif r.rows then
            print("  ✅ " .. label .. ": Retrieved " .. #r.rows .. " rows")
        elseif result.message then
            print("  ✅ " .. label .. ": " .. result.message)
        else
            print("  ✅ " .. label .. ": Success")
            if type(r) == "table" then
                print(r, 2)
            end
        end
    end
end

print("Email Sender Tool")

print("\nNote: Email examples require configuration of SMTP or API credentials")
print("Set environment variables like SENDGRID_API_KEY, AWS_ACCESS_KEY_ID, etc.\n")

-- SMTP Email example
local smtp_result = use_tool("email-sender", {
    provider = "smtp",
    from = "sender@example.com",
    to = "recipient@example.com",
    subject = "Test Email from LLMSpell",
    body = "This is a test email sent via SMTP.",
    smtp_host = "smtp.gmail.com",
    smtp_port = 587,
    smtp_username = "your-email@gmail.com",
    smtp_password = "your-app-password"
})
print_result("SMTP email", smtp_result)

-- SendGrid email example
local sendgrid_result = use_tool("email-sender", {
    provider = "sendgrid",
    to = "recipient@example.com",
    from = "sender@example.com",
    subject = "Test Email via SendGrid",
    body = "This email was sent using SendGrid API.",
    html_body = "<h1>Hello from LLMSpell!</h1><p>This email was sent using SendGrid API.</p>"
})
print_result("SendGrid email", sendgrid_result)

-- AWS SES email example
local ses_result = use_tool("email-sender", {
    provider = "ses",
    to = "recipient@example.com",
    from = "verified-sender@example.com",
    subject = "Test Email via AWS SES",
    body = "This email was sent using AWS Simple Email Service.",
    region = "us-east-1"
})
print_result("AWS SES email", ses_result)

-- Email with attachments (conceptual - implementation pending)
local attachment_result = use_tool("email-sender", {
    provider = "smtp",
    from = "sender@example.com",
    to = "recipient@example.com",
    subject = "Email with Attachment",
    body = "Please find the report attached.",
    attachments = {
        {
            filename = "report.pdf",
            content_type = "application/pdf",
            path = "/path/to/report.pdf"
        }
    }
})
print_result("Email with attachment", attachment_result)

print("Database Connector Tool")

print("\nNote: Database examples require connection strings or credentials")
print("Set DATABASE_URL or individual connection parameters\n")

-- SQLite example (in-memory for testing)
local sqlite_result = use_tool("database-connector", {
    provider = "sqlite",
    connection_string = ":memory:",
    query = "SELECT 1 as test_value",
    operation = "query"
})
print_result("SQLite query", sqlite_result)

-- PostgreSQL example
local postgres_query = use_tool("database-connector", {
    provider = "postgresql",
    host = "localhost",
    port = 5432,
    database = "test_db",
    username = "postgres",
    password = "password",
    query = "SELECT version()",
    operation = "query"
})
print_result("PostgreSQL version", postgres_query)

-- MySQL example with parameterized query
local mysql_result = use_tool("database-connector", {
    provider = "mysql",
    connection_string = "mysql://user:pass@localhost/mydb",
    query = "SELECT * FROM users WHERE status = ?",
    params = {"active"},
    operation = "query"
})
print_result("MySQL parameterized query", mysql_result)

-- Execute DDL (CREATE TABLE)
local create_table_result = use_tool("database-connector", {
    provider = "sqlite",
    connection_string = ":memory:",
    query = [[
        CREATE TABLE users (
            id INTEGER PRIMARY KEY,
            name TEXT NOT NULL,
            email TEXT UNIQUE,
            created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
        )
    ]],
    operation = "execute"
})
print_result("Create table", create_table_result)

-- Insert data
local insert_result = use_tool("database-connector", {
    provider = "sqlite",
    connection_string = ":memory:",
    query = "INSERT INTO users (name, email) VALUES (?, ?)",
    params = {"John Doe", "john@example.com"},
    operation = "execute"
})
print_result("Insert user", insert_result)

print("Rate Limiting Examples")

print("\nDemonstrating rate limiting behavior:")

-- Simulate rate limiting with web search
print("\nTesting rate limits with rapid requests:")
for i = 1, 5 do
    local result = use_tool("web_search", {
        input = "test query " .. i,
        provider = "duckduckgo",
        max_results = 1
    })
    if result.error and result.error:match("rate") then
        print("  ⏸️  Request " .. i .. ": Rate limited")
    else
        print("  ✅ Request " .. i .. ": Success")
    end
    -- Small delay to demonstrate rate limiting
    os.execute("sleep 0.1")
end

-- Using API with built-in rate limiting
print("\nAPI calls with rate limiting:")
local api_results = {}
for i = 1, 3 do
    local result = use_tool("api-tester", {
        input = "https://httpbin.org/delay/1",
        method = "GET",
        timeout = 2
    })
    table.insert(api_results, result)
    print_result("API call " .. i, result)
end

print("Circuit Breaker Examples")

print("\nDemonstrating circuit breaker pattern:")

-- Simulate failures to trigger circuit breaker
print("\nTesting circuit breaker with failing endpoint:")
local failing_endpoint = "https://httpbin.org/status/500"

for i = 1, 6 do
    local result = use_tool("api-tester", {
        input = failing_endpoint,
        method = "GET",
        expected_status = 200
    })
    
    if result.error and result.error:match("circuit") then
        print("  🔴 Request " .. i .. ": Circuit breaker OPEN")
    elseif result.success == false then
        print("  ⚠️  Request " .. i .. ": Request failed (circuit counting)")
    else
        print("  ✅ Request " .. i .. ": Success")
    end
end

-- Test recovery after circuit opens
print("\nWaiting for circuit breaker recovery...")
os.execute("sleep 2")

local recovery_result = use_tool("api-tester", {
    input = "https://httpbin.org/status/200",
    method = "GET"
})
print_result("Recovery attempt", recovery_result)

print("API Key Management Examples")

print("\nDemonstrating API key management:")

-- Check if API keys are configured
print("\nChecking configured API keys:")
local providers = {"openai", "anthropic", "google", "sendgrid", "aws"}
for _, provider in ipairs(providers) do
    local env_var = "LLMSPELL_API_KEY_" .. provider:upper()
    if os.getenv(env_var) then
        print("  ✅ " .. provider .. ": Configured")
    else
        print("  ⚫ " .. provider .. ": Not configured")
    end
end

-- Using tools that require API keys
print("\nUsing tools with API key requirements:")

-- Web search with API key
local api_search_result = use_tool("web_search", {
    input = "artificial intelligence",
    provider = "google",  -- Requires LLMSPELL_API_KEY_GOOGLE
    max_results = 3
})
if api_search_result.error and api_search_result.error:match("API key") then
    print("  ℹ️  Google search: API key required")
else
    print_result("Google search with API", api_search_result)
end

-- Email with API key
local email_api_result = use_tool("email-sender", {
    provider = "sendgrid",  -- Requires LLMSPELL_API_KEY_SENDGRID
    to = "test@example.com",
    from = "sender@example.com",
    subject = "API Key Test",
    body = "Testing SendGrid with API key"
})
if email_api_result.error and email_api_result.error:match("API key") then
    print("  ℹ️  SendGrid email: API key required")
else
    print_result("SendGrid with API", email_api_result)
end

print("Connection Pooling Examples")

print("\nDemonstrating connection pooling:")

-- Multiple database queries using connection pool
print("\nExecuting multiple queries with connection pooling:")
local pool_queries = {
    "SELECT 1 as num",
    "SELECT 2 as num",
    "SELECT 3 as num",
    "SELECT 4 as num",
    "SELECT 5 as num"
}

for i, query in ipairs(pool_queries) do
    local result = use_tool("database-connector", {
        provider = "sqlite",
        connection_string = ":memory:",
        query = query,
        operation = "query",
        use_pool = true,  -- Enable connection pooling
        pool_size = 3     -- Max 3 connections
    })
    print_result("Pooled query " .. i, result)
end

print("\n✨ Integration tools examples completed!")
print("\nNote: Many examples require API keys or service configuration.")
print("Set environment variables to enable full functionality.")