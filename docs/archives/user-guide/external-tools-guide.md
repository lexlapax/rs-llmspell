# External Tools Guide - Phase 3.1

✅ **Production Ready**: All 8 external integration tools are fully implemented, tested, and available for use.

This guide provides comprehensive documentation for the 8 external integration tools added in Phase 3.1.

## Table of Contents

1. [Overview](#overview)
2. [Web Tools](#web-tools)
   - [UrlAnalyzerTool](#urlanalyzertool)
   - [WebScraperTool](#webscrapertool)
   - [ApiTesterTool](#apitestertool)
   - [WebhookCallerTool](#webhookcallertool)
   - [WebpageMonitorTool](#webpagemonitortool)
   - [SitemapCrawlerTool](#sitemapecrawlertool)
3. [Communication Tools](#communication-tools)
   - [EmailSenderTool](#emailsendertool)
   - [DatabaseConnectorTool](#databaseconnectortool)
4. [Configuration](#configuration)
   - [API Key Setup](#api-key-setup)
   - [Rate Limiting](#rate-limiting)
   - [Error Handling](#error-handling)
5. [Integration Examples](#integration-examples)
6. [Troubleshooting](#troubleshooting)

## Overview

Phase 3.1 introduces 8 new external integration tools that extend LLMSpell's capabilities to interact with web services, send emails, and connect to databases. These tools follow the same security and performance standards as Phase 2 tools.

### Key Features

- **Standardized Interfaces**: All tools use the `input` parameter as primary data
- **ResponseBuilder Pattern**: Consistent response format across all tools
- **Security First**: Input validation, rate limiting, and secure credential handling
- **Performance**: <10ms initialization time for all tools
- **Error Handling**: Comprehensive error messages with actionable solutions

## Web Tools

### UrlAnalyzerTool

Analyzes and validates URLs, extracting metadata and components.

#### Configuration

```lua
local tool = Tool.get("url-analyzer")
```

#### Parameters

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| input | string | Yes | URL to analyze |
| decode_params | boolean | No | Decode URL parameters (default: true) |
| extract_links | boolean | No | Extract links from page (default: false) |
| extract_images | boolean | No | Extract images from page (default: false) |
| extract_meta | boolean | No | Extract meta tags (default: false) |

#### Example Usage

```lua
local result = tool.execute({
    input = "https://example.com/path?query=value#section",
    decode_params = true
})

-- Result includes:
-- {
--   valid: true,
--   scheme: "https",
--   host: "example.com",
--   path: "/path",
--   query: "query=value",
--   query_params: { query: "value" },
--   fragment: "section"
-- }
```

#### Error Scenarios

- Invalid URL format: Returns validation error
- Relative URLs: Not supported, must be absolute
- Non-HTTP(S) schemes: FTP, file:// not supported

### WebScraperTool

Extracts content from web pages with CSS selector support.

#### Configuration

```lua
local tool = Tool.get("web-scraper")
```

#### Parameters

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| input | string | Yes | URL to scrape |
| selector | string | No | CSS selector for specific content |
| extract_links | boolean | No | Extract all links (default: false) |
| extract_images | boolean | No | Extract all images (default: false) |
| extract_meta | boolean | No | Extract meta tags (default: false) |
| timeout | number | No | Timeout in seconds (default: 30) |

#### Example Usage

```lua
-- Extract specific content
local result = tool.execute({
    input = "https://example.com",
    selector = "h1",
    extract_links = true,
    timeout = 10
})

-- Full page extraction
local full_result = tool.execute({
    input = "https://example.com",
    extract_links = true,
    extract_images = true,
    extract_meta = true
})
```

#### Rate Limiting

- Default: 10 requests per minute per domain
- Respects robots.txt when configured
- Automatic retry with exponential backoff

### ApiTesterTool

Tests REST APIs with comprehensive request/response validation.

#### Configuration

```lua
local tool = Tool.get("api-tester")
```

#### Parameters

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| input | string | Yes | API endpoint URL |
| method | string | No | HTTP method (default: GET) |
| headers | object | No | Request headers |
| body | any | No | Request body (JSON/form data) |
| expected_status | number | No | Expected HTTP status code |
| timeout | number | No | Timeout in seconds (default: 30) |

#### Example Usage

```lua
-- GET request
local get_result = tool.execute({
    input = "https://api.example.com/users",
    headers = {
        ["Authorization"] = "Bearer token123"
    }
})

-- POST request with JSON
local post_result = tool.execute({
    input = "https://api.example.com/users",
    method = "POST",
    headers = {
        ["Content-Type"] = "application/json"
    },
    body = {
        name = "John Doe",
        email = "john@example.com"
    },
    expected_status = 201
})
```

#### Response Validation

- Automatic JSON parsing when Content-Type is application/json
- Status code validation against expected_status
- Response time tracking
- Header extraction

### WebhookCallerTool

Invokes webhooks with retry logic and delivery guarantees.

#### Configuration

```lua
local tool = Tool.get("webhook-caller")
```

#### Parameters

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| input | string | Yes | Webhook URL |
| method | string | No | HTTP method (default: POST) |
| headers | object | No | Request headers |
| payload | any | No | Webhook payload |
| retry_count | number | No | Number of retries (default: 3) |
| retry_delay | number | No | Delay between retries in ms (default: 1000) |
| timeout | number | No | Timeout in seconds (default: 30) |

#### Example Usage

```lua
local result = tool.execute({
    input = "https://hooks.example.com/webhook/123",
    headers = {
        ["X-Webhook-Secret"] = "secret123"
    },
    payload = {
        event = "user.created",
        user_id = 12345,
        timestamp = os.time()
    },
    retry_count = 5,
    retry_delay = 2000
})
```

#### Retry Logic

- Exponential backoff with jitter
- Retries on network errors and 5xx status codes
- Does not retry on 4xx client errors
- Tracks retry attempts in response

### WebpageMonitorTool

Monitors web pages for changes with diff detection.

#### Configuration

```lua
local tool = Tool.get("webpage-monitor")
```

#### Parameters

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| input | string | Yes | URL to monitor |
| previous_content | string | No | Previous content for comparison |
| selector | string | No | CSS selector for specific monitoring |
| ignore_whitespace | boolean | No | Ignore whitespace changes (default: true) |

#### Example Usage

```lua
-- Initial check
local initial = tool.execute({
    input = "https://example.com/status"
})
local baseline = initial.result.current_content

-- Check for changes
local monitor = tool.execute({
    input = "https://example.com/status",
    previous_content = baseline,
    ignore_whitespace = true
})

if monitor.result.has_changes then
    print("Changes detected:", #monitor.result.changes)
end
```

#### Change Detection

- Line-by-line diff comparison
- Change types: addition, deletion, modification
- Grouped changes for better readability
- Percentage change calculation

### SitemapCrawlerTool

Parses XML sitemaps and discovers URLs.

#### Configuration

```lua
local tool = Tool.get("sitemap-crawler")
```

#### Parameters

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| input | string | Yes | Sitemap URL |
| follow_sitemaps | boolean | No | Follow nested sitemaps (default: true) |
| max_urls | number | No | Maximum URLs to return (default: 1000) |
| timeout | number | No | Timeout in seconds (default: 30) |

#### Example Usage

```lua
local result = tool.execute({
    input = "https://example.com/sitemap.xml",
    follow_sitemaps = true,
    max_urls = 100
})

-- Result includes:
-- {
--   urls: ["https://example.com/page1", ...],
--   count: 100,
--   has_more: true
-- }
```

## Communication Tools

### EmailSenderTool

Multi-provider email sending with SMTP, SendGrid, and AWS SES support.

#### Configuration

```lua
local tool = Tool.get("email-sender")
```

#### Parameters

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| provider | string | Yes | Email provider (smtp/sendgrid/ses) |
| from | string | Yes | Sender email address |
| to | string | Yes | Recipient email address |
| subject | string | Yes | Email subject |
| body | string | Yes | Plain text body |
| html_body | string | No | HTML body |
| cc | array | No | CC recipients |
| bcc | array | No | BCC recipients |
| attachments | array | No | File attachments |

#### Provider-Specific Parameters

**SMTP:**
- smtp_host: SMTP server hostname
- smtp_port: SMTP server port
- smtp_username: SMTP username
- smtp_password: SMTP password
- smtp_tls: Use TLS (default: true)

**SendGrid:**
- Requires LLMSPELL_API_KEY_SENDGRID environment variable

**AWS SES:**
- region: AWS region (default: us-east-1)
- Requires AWS_ACCESS_KEY_ID and AWS_SECRET_ACCESS_KEY

#### Example Usage

```lua
-- SMTP example
local smtp_result = tool.execute({
    provider = "smtp",
    from = "sender@example.com",
    to = "recipient@example.com",
    subject = "Test Email",
    body = "This is a test email.",
    smtp_host = "smtp.gmail.com",
    smtp_port = 587,
    smtp_username = "user@gmail.com",
    smtp_password = "app-specific-password"
})

-- SendGrid example
local sendgrid_result = tool.execute({
    provider = "sendgrid",
    from = "sender@example.com",
    to = "recipient@example.com",
    subject = "Newsletter",
    body = "Plain text version",
    html_body = "<h1>HTML Newsletter</h1><p>Content here</p>"
})
```

### DatabaseConnectorTool

Multi-database support for SQLite, PostgreSQL, and MySQL.

#### Configuration

```lua
local tool = Tool.get("database-connector")
```

#### Parameters

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| provider | string | Yes | Database type (sqlite/postgresql/mysql) |
| operation | string | Yes | Operation type (query/execute) |
| query | string | Yes | SQL query |
| params | array | No | Query parameters |
| connection_string | string | No* | Connection string |
| host | string | No* | Database host |
| port | number | No* | Database port |
| database | string | No* | Database name |
| username | string | No* | Database username |
| password | string | No* | Database password |
| use_pool | boolean | No | Use connection pooling (default: true) |
| pool_size | number | No | Connection pool size (default: 10) |

*Either connection_string OR individual connection parameters required

#### Example Usage

```lua
-- SQLite in-memory
local sqlite_result = tool.execute({
    provider = "sqlite",
    connection_string = ":memory:",
    operation = "query",
    query = "SELECT datetime('now') as current_time"
})

-- PostgreSQL with parameters
local pg_result = tool.execute({
    provider = "postgresql",
    host = "localhost",
    port = 5432,
    database = "myapp",
    username = "user",
    password = "pass",
    operation = "query",
    query = "SELECT * FROM users WHERE status = $1",
    params = {"active"}
})

-- MySQL execute
local mysql_result = tool.execute({
    provider = "mysql",
    connection_string = "mysql://user:pass@localhost/mydb",
    operation = "execute",
    query = "UPDATE users SET last_login = NOW() WHERE id = ?",
    params = {123}
})
```

#### Security Considerations

- SQL injection protection through parameterized queries
- Connection string credentials should use environment variables
- DDL operations restricted by default
- Query timeout enforcement

## Configuration

### API Key Setup

External tools requiring API keys use environment variables with the prefix `LLMSPELL_API_KEY_`:

```bash
# Web Search Providers
export LLMSPELL_API_KEY_GOOGLE="your-google-custom-search-api-key"
export LLMSPELL_API_KEY_BRAVE="your-brave-search-api-key"
export LLMSPELL_API_KEY_SERPAPI="your-serpapi-key"
export LLMSPELL_API_KEY_SERPERDEV="your-serperdev-api-key"

# Email Provider
export LLMSPELL_API_KEY_SENDGRID="your-sendgrid-api-key"

# AWS Credentials (for SES)
export AWS_ACCESS_KEY_ID="your-aws-access-key"
export AWS_SECRET_ACCESS_KEY="your-aws-secret-key"
export AWS_REGION="us-east-1"

# Database URLs
export DATABASE_URL="postgresql://user:pass@localhost:5432/mydb"
```

#### Managing API Keys

Use the built-in API key management system:

```bash
# Add a key
llmspell keys add google "your-api-key"

# List configured keys
llmspell keys list

# Rotate a key
llmspell keys rotate google "new-api-key"

# Remove a key
llmspell keys remove google
```

### Rate Limiting

All external tools implement rate limiting to prevent API abuse:

#### Default Limits

| Provider | Requests/Minute | Requests/Hour |
|----------|----------------|---------------|
| DuckDuckGo | 60 | 1000 |
| Google | 100 | 1000 |
| Brave | 50 | 500 |
| SendGrid | 100 | 3000 |
| Generic API | 60 | 600 |

#### Custom Rate Limits

Configure custom limits in `llmspell.toml`:

```toml
[tools.rate_limits]
google = { per_minute = 50, per_hour = 500 }
sendgrid = { per_minute = 200, per_hour = 5000 }
custom_api = { per_minute = 30, per_hour = 300 }
```

#### Rate Limit Headers

Tools automatically parse and respect rate limit headers:
- `X-RateLimit-Limit`
- `X-RateLimit-Remaining`
- `X-RateLimit-Reset`
- `Retry-After`

### Error Handling

All tools follow consistent error handling patterns:

#### Error Types

1. **Validation Errors**: Invalid parameters or input
2. **Network Errors**: Connection failures, timeouts
3. **API Errors**: Authentication, rate limits, server errors
4. **Configuration Errors**: Missing API keys or credentials

#### Error Response Format

```json
{
  "success": false,
  "operation": "tool_operation",
  "error": {
    "code": "ERROR_CODE",
    "message": "Human-readable error message",
    "details": {
      "field": "parameter_name",
      "reason": "validation_failed"
    }
  }
}
```

#### Common Error Codes

| Code | Description | Solution |
|------|-------------|----------|
| VALIDATION_ERROR | Invalid input parameters | Check parameter types and requirements |
| MISSING_REQUIRED | Required parameter missing | Provide all required parameters |
| NETWORK_ERROR | Network connection failed | Check connectivity and retry |
| TIMEOUT | Request timed out | Increase timeout or retry |
| RATE_LIMITED | API rate limit exceeded | Wait and retry with backoff |
| AUTH_FAILED | Authentication failed | Check API keys and credentials |
| NOT_FOUND | Resource not found | Verify URL or identifier |
| SERVER_ERROR | Server returned 5xx | Retry with exponential backoff |

## Integration Examples

### Web Scraping Pipeline

```lua
-- 1. Analyze URL
local url_tool = Tool.get("url-analyzer")
local url_result = url_tool.execute({
    input = "https://example.com/products"
})

if not url_result.result.valid then
    error("Invalid URL")
end

-- 2. Scrape content
local scraper_tool = Tool.get("web-scraper")
local content = scraper_tool.execute({
    input = "https://example.com/products",
    selector = ".product",
    extract_links = true
})

-- 3. Store in database
local db_tool = Tool.get("database-connector")
for _, product in ipairs(content.result.products) do
    db_tool.execute({
        provider = "postgresql",
        connection_string = os.getenv("DATABASE_URL"),
        operation = "execute",
        query = "INSERT INTO products (name, price, url) VALUES ($1, $2, $3)",
        params = {product.name, product.price, product.url}
    })
end
```

### API Monitoring System

```lua
-- Monitor API endpoints
local api_tool = Tool.get("api-tester")
local webhook_tool = Tool.get("webhook-caller")

local endpoints = {
    {url = "https://api.example.com/health", expected = 200},
    {url = "https://api.example.com/v1/users", expected = 200}
}

for _, endpoint in ipairs(endpoints) do
    local result = api_tool.execute({
        input = endpoint.url,
        expected_status = endpoint.expected,
        timeout = 5
    })
    
    if not result.success or result.result.response.status_code ~= endpoint.expected then
        -- Send alert webhook
        webhook_tool.execute({
            input = "https://hooks.slack.com/services/xxx/yyy/zzz",
            payload = {
                text = string.format("API Alert: %s returned %d", 
                    endpoint.url, 
                    result.result.response.status_code or 0)
            }
        })
    end
end
```

### Email Report Generator

```lua
-- Generate report from database
local db_tool = Tool.get("database-connector")
local email_tool = Tool.get("email-sender")

-- Query data
local report_data = db_tool.execute({
    provider = "postgresql",
    connection_string = os.getenv("DATABASE_URL"),
    operation = "query",
    query = [[
        SELECT 
            COUNT(*) as total_users,
            COUNT(CASE WHEN created_at > NOW() - INTERVAL '7 days' THEN 1 END) as new_users
        FROM users
    ]]
})

-- Format report
local report_body = string.format([[
Weekly User Report

Total Users: %d
New Users (Last 7 Days): %d

Generated: %s
]], 
    report_data.result.rows[1].total_users,
    report_data.result.rows[1].new_users,
    os.date("%Y-%m-%d %H:%M:%S")
)

-- Send email
email_tool.execute({
    provider = "sendgrid",
    from = "reports@example.com",
    to = "admin@example.com",
    subject = "Weekly User Report",
    body = report_body
})
```

## Troubleshooting

### Common Issues and Solutions

#### Tool Not Found

**Error**: `Tool 'tool-name' not found`

**Solutions**:
1. Verify tool name (use hyphens, not underscores)
2. Check if llmspell-tools is properly installed
3. Ensure tool is registered in llmspell-bridge

#### API Key Not Configured

**Error**: `Email provider 'sendgrid' not configured`

**Solutions**:
1. Set environment variable: `export LLMSPELL_API_KEY_SENDGRID="your-key"`
2. Use `llmspell keys add sendgrid "your-key"`
3. Check variable name spelling (case-sensitive)

#### Network Timeouts

**Error**: `error sending request for url (https://...): operation timed out`

**Solutions**:
1. Increase timeout parameter: `timeout = 60`
2. Check network connectivity
3. Verify URL is accessible
4. Consider retry logic for transient failures

#### Rate Limiting

**Error**: `Rate limit exceeded`

**Solutions**:
1. Implement exponential backoff
2. Use different API provider
3. Request rate limit increase
4. Cache responses when possible

#### Database Connection Failed

**Error**: `Database 'mydb' not configured`

**Solutions**:
1. Verify connection string format
2. Check database server is running
3. Ensure network access to database
4. Verify credentials are correct

### Debug Mode

Enable debug logging for detailed troubleshooting:

```lua
-- In your script
local tool = Tool.get("web-scraper")
tool.set_debug(true)

-- Or via environment
export LLMSPELL_LOG_LEVEL=debug
```

### Performance Tips

1. **Connection Pooling**: Always use for databases
2. **Timeouts**: Set appropriate timeouts for your use case
3. **Batch Operations**: Group database queries when possible
4. **Caching**: Cache web scraping results
5. **Async Operations**: Use coroutines for parallel requests

### Security Best Practices

1. **Never hardcode credentials** - Use environment variables
2. **Validate all inputs** - Especially URLs and SQL queries
3. **Use parameterized queries** - Prevent SQL injection
4. **Implement rate limiting** - Protect against abuse
5. **Monitor usage** - Track API calls and errors
6. **Rotate keys regularly** - Use key rotation features
7. **Restrict permissions** - Minimum required access

## Quick Reference

### Web Tools Summary

| Tool | Code Name | Purpose | Key Parameters |
|------|-----------|---------|----------------|
| 🔗 **UrlAnalyzerTool** | `url-analyzer` | Validate and parse URLs | `input`, `decode_params` |
| 🌐 **WebScraperTool** | `web-scraper` | Extract web content | `input`, `selector`, `timeout` |
| 🧪 **ApiTesterTool** | `api-tester` | Test REST APIs | `input`, `method`, `headers`, `body` |
| 🪝 **WebhookCallerTool** | `webhook-caller` | Call webhooks with retry | `input`, `payload`, `retry_count` |
| 👁️ **WebpageMonitorTool** | `webpage-monitor` | Detect page changes | `input`, `previous_content` |
| 🗺️ **SitemapCrawlerTool** | `sitemap-crawler` | Parse sitemaps | `input`, `max_urls` |

### Communication Tools Summary

| Tool | Code Name | Purpose | Key Parameters |
|------|-----------|---------|----------------|
| 📧 **EmailSenderTool** | `email-sender` | Send emails | `provider`, `from`, `to`, `subject` |
| 🗄️ **DatabaseConnectorTool** | `database-connector` | Database operations | `provider`, `query`, `params` |

### Quick Examples

```lua
-- URL Analysis
Tool.get("url-analyzer"):execute({input = "https://example.com/path?q=value"})

-- Web Scraping
Tool.get("web-scraper"):execute({
    input = "https://example.com",
    selector = "h1",
    extract_links = true
})

-- API Testing
Tool.get("api-tester"):execute({
    input = "https://api.example.com/endpoint",
    method = "POST",
    body = {key = "value"}
})

-- Send Email (SendGrid)
Tool.get("email-sender"):execute({
    provider = "sendgrid",
    from = "sender@example.com",
    to = "recipient@example.com",
    subject = "Test",
    body = "Message"
})

-- Database Query
Tool.get("database-connector"):execute({
    provider = "postgresql",
    connection_string = "postgresql://user:pass@localhost/db",
    operation = "query",
    query = "SELECT * FROM users WHERE status = $1",
    params = {"active"}
})
```

### Environment Variables Quick Reference

```bash
# Web Search
export LLMSPELL_API_KEY_GOOGLE="..."
export LLMSPELL_API_KEY_BRAVE="..."
export LLMSPELL_API_KEY_SERPAPI="..."
export LLMSPELL_API_KEY_SERPERDEV="..."

# Email
export LLMSPELL_API_KEY_SENDGRID="..."
export AWS_ACCESS_KEY_ID="..."
export AWS_SECRET_ACCESS_KEY="..."

# Database
export DATABASE_URL="postgresql://..."
```

### Common Patterns

#### Error Handling
```lua
local result = tool:execute(params)
if not result.success then
    Logger.error("Operation failed", {error = result.error.message})
end
```

#### Retry with Backoff
```lua
local result = tool:execute({
    input = url,
    retry_count = 5,
    retry_delay = 2000  -- ms
})
```

#### Connection Pooling
```lua
local result = tool:execute({
    provider = "postgresql",
    use_pool = true,
    pool_size = 10,
    -- other params
})
```

### Performance Tips

1. **Set appropriate timeouts** - Default 30s may be too long
2. **Use connection pooling** - For repeated database queries
3. **Cache web content** - Avoid repeated scraping
4. **Batch operations** - Group similar requests
5. **Handle rate limits** - Implement exponential backoff

### Security Reminders

- ⚠️ Never hardcode credentials
- ⚠️ Always use parameterized queries
- ⚠️ Validate URLs before scraping
- ⚠️ Sanitize email inputs
- ⚠️ Use HTTPS endpoints only
- ⚠️ Rotate API keys regularly

## Additional Resources

- [LLMSpell Examples](../../examples/README.md)
- [API Reference](../technical/tool-integration-architecture.md)
- [Security Guide](../security/SECURITY_BEST_PRACTICES.md)
- [Performance Tips](performance-tips.md)

---

For feature requests or bug reports, please file an issue in the project repository.