# External Tools Quick Reference

## Web Tools

### 🔗 UrlAnalyzerTool
```lua
Tool.get("url-analyzer").execute({
    input = "https://example.com/path?q=value",
    decode_params = true
})
```
**Purpose**: Validate and parse URLs  
**Key Output**: `valid`, `host`, `path`, `query_params`

### 🌐 WebScraperTool
```lua
Tool.get("web-scraper").execute({
    input = "https://example.com",
    selector = "h1",           -- Optional CSS selector
    extract_links = true,      -- Extract all links
    extract_meta = true,       -- Extract meta tags
    timeout = 30              -- Timeout in seconds
})
```
**Purpose**: Extract content from web pages  
**Key Output**: `text`, `title`, `links`, `meta_tags`

### 🧪 ApiTesterTool
```lua
Tool.get("api-tester").execute({
    input = "https://api.example.com/endpoint",
    method = "POST",
    headers = { ["Authorization"] = "Bearer token" },
    body = { key = "value" },
    expected_status = 201
})
```
**Purpose**: Test REST APIs  
**Key Output**: `status_code`, `response`, `timing`

### 🪝 WebhookCallerTool
```lua
Tool.get("webhook-caller").execute({
    input = "https://hooks.example.com/webhook",
    payload = { event = "user.created" },
    retry_count = 3,
    retry_delay = 1000
})
```
**Purpose**: Call webhooks with retry logic  
**Key Output**: `status_code`, `response_time_ms`, `retries_attempted`

### 👁️ WebpageMonitorTool
```lua
Tool.get("webpage-monitor").execute({
    input = "https://example.com/status",
    previous_content = "old content",
    ignore_whitespace = true
})
```
**Purpose**: Detect webpage changes  
**Key Output**: `has_changes`, `changes`, `current_content`

### 🗺️ SitemapCrawlerTool
```lua
Tool.get("sitemap-crawler").execute({
    input = "https://example.com/sitemap.xml",
    follow_sitemaps = true,
    max_urls = 100
})
```
**Purpose**: Parse sitemaps and discover URLs  
**Key Output**: `urls`, `count`, `has_more`

## Communication Tools

### 📧 EmailSenderTool
```lua
-- SMTP
Tool.get("email-sender").execute({
    provider = "smtp",
    from = "sender@example.com",
    to = "recipient@example.com",
    subject = "Test",
    body = "Message",
    smtp_host = "smtp.gmail.com",
    smtp_port = 587,
    smtp_username = "user",
    smtp_password = "pass"
})

-- SendGrid (requires LLMSPELL_API_KEY_SENDGRID)
Tool.get("email-sender").execute({
    provider = "sendgrid",
    from = "sender@example.com",
    to = "recipient@example.com",
    subject = "Test",
    body = "Plain text",
    html_body = "<h1>HTML</h1>"
})
```
**Purpose**: Send emails via multiple providers  
**Providers**: smtp, sendgrid, ses

### 🗄️ DatabaseConnectorTool
```lua
-- Query
Tool.get("database-connector").execute({
    provider = "postgresql",
    connection_string = "postgresql://user:pass@localhost/db",
    operation = "query",
    query = "SELECT * FROM users WHERE status = $1",
    params = {"active"}
})

-- Execute
Tool.get("database-connector").execute({
    provider = "sqlite",
    connection_string = ":memory:",
    operation = "execute",
    query = "CREATE TABLE test (id INTEGER PRIMARY KEY)"
})
```
**Purpose**: Database operations  
**Providers**: sqlite, postgresql, mysql  
**Operations**: query (SELECT), execute (INSERT/UPDATE/DELETE/DDL)

## Environment Variables

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

## Common Patterns

### Error Handling
```lua
local result = tool.execute(params)
if not result.success then
    print("Error:", result.error.message)
    -- Handle error
end
```

### Rate Limiting
```lua
-- Automatic retry with backoff
local result = tool.execute({
    input = url,
    retry_count = 5,
    retry_delay = 2000  -- ms
})
```

### Connection Pooling
```lua
-- Database connection pooling
local result = tool.execute({
    provider = "postgresql",
    use_pool = true,
    pool_size = 10,
    -- ... other params
})
```

## Tool Names Reference

| Tool | Name in Code |
|------|--------------|
| URL Analyzer | `url-analyzer` |
| Web Scraper | `web-scraper` |
| API Tester | `api-tester` |
| Webhook Caller | `webhook-caller` |
| Webpage Monitor | `webpage-monitor` |
| Sitemap Crawler | `sitemap-crawler` |
| Email Sender | `email-sender` |
| Database Connector | `database-connector` |

## Performance Tips

1. **Set appropriate timeouts** - Default 30s may be too long
2. **Use connection pooling** - For repeated database queries
3. **Cache web content** - Avoid repeated scraping
4. **Batch operations** - Group similar requests
5. **Handle rate limits** - Implement exponential backoff

## Security Reminders

- ⚠️ Never hardcode credentials
- ⚠️ Always use parameterized queries
- ⚠️ Validate URLs before scraping
- ⚠️ Sanitize email inputs
- ⚠️ Use HTTPS endpoints only
- ⚠️ Rotate API keys regularly