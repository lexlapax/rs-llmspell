# Tool Reference

**Version**: Phase 7 Complete  
**Status**: ✅ **CURRENT** - All 37 tools production-ready  
**Last Updated**: August 2025

> **🔧 COMPREHENSIVE REFERENCE**: Complete documentation for all 37 built-in tools across 10 categories. Each tool includes usage examples, parameters, and security considerations.

**🔗 Navigation**: [← User Guide](README.md) | [Documentation Hub](../README.md) | [Example Index](example-index.md) | [Examples](../../examples/) | [External Tools Guide](external-tools-guide.md)

## Example References
- **Basic Tool Usage**: [`05-use-tools.lua`](../../examples/script-users/getting-started/05-use-tools.lua)
- **Filesystem Tools**: [`filesystem-tools.lua`](../../examples/script-users/features/filesystem-tools.lua)
- **Utility Tools**: [`utility-tools.lua`](../../examples/script-users/features/utility-tools.lua)
- **Tool Integration**: [`tools-integration.lua`](../../examples/script-users/advanced/tools-integration.lua)
- **Tool-Workflow Chaining**: [`tools-workflow-chaining.lua`](../../examples/script-users/features/tools-workflow-chaining.lua)

---

## Table of Contents

1. [Overview](#overview)
2. [Tool Categories](#tool-categories)
3. [Quick Reference Table](#quick-reference-table)
4. [Detailed Tool Documentation](#detailed-tool-documentation)
   - [File System Tools (5)](#file-system-tools)
   - [Data Processing Tools (4)](#data-processing-tools)
   - [Web & Network Tools (7)](#web--network-tools)
   - [System Integration Tools (4)](#system-integration-tools)
   - [Utility Tools (10)](#utility-tools)
   - [Media Processing Tools (3)](#media-processing-tools)
   - [Document & Academic Tools (3)](#document--academic-tools)
   - [Search Tools (1)](#search-tools)
5. [Security Considerations](#security-considerations)
6. [Performance Characteristics](#performance-characteristics)
7. [Common Patterns](#common-patterns)

## Overview

Rs-llmspell provides 37 production-ready tools that can be used standalone or integrated with agents and workflows. All tools follow consistent patterns:

```lua
-- Basic tool usage pattern
local tool = Tool.get("tool_name")
local result = tool:execute({
    operation = "operation_name",
    -- tool-specific parameters
})

if result.success then
    print("Output:", result.output)
else
    print("Error:", result.error)
end
```

### Tool Discovery

```lua
-- List all available tools
local tools = Tool.list()
for _, tool_name in ipairs(tools) do
    local tool = Tool.get(tool_name)
    print(tool_name .. ": " .. tool.description)
end

-- Get tool schema
local schema = Tool.get_schema("file_operations")
print(JSON.stringify(schema, true))
```

## Tool Categories

### 📁 File System Tools (5)
Tools for file and directory operations with security sandboxing.

### 📊 Data Processing Tools (5)
Tools for processing JSON, CSV, HTTP requests, GraphQL queries, and graph data structures.

### 🌐 Web & Network Tools (7)
Tools for web scraping, API testing, webhooks, and web search.

### 🖥️ System Integration Tools (4)
Tools for system monitoring, process execution, and service checks.

### 🔧 Utility Tools (10)
General-purpose tools for text manipulation, calculations, and data validation.

### 🎬 Media Processing Tools (3)
Tools for audio, video, and image file processing (metadata extraction).

### 📚 Document & Academic Tools (3)
Tools for PDF processing, citation formatting, and academic document handling.

### 🔍 Search Tools (1)
Web search integration with multiple providers.

## Quick Reference Table

| Tool Name | Category | Primary Operations | Security Level |
|-----------|----------|-------------------|----------------|
| `file_operations` | File System | read, write, delete, list, copy, move | Restricted |
| `archive_handler` | File System | create, extract, list | Restricted |
| `file_watcher` | File System | watch, unwatch | Restricted |
| `file_converter` | File System | convert_encoding, convert_format | Safe |
| `file_search` | File System | search, search_regex | Restricted |
| `json_processor` | Data Processing | parse, stringify, query, transform | Safe |
| `csv_analyzer` | Data Processing | analyze, parse, write | Safe |
| `http_request` | Data Processing | get, post, put, delete | Restricted |
| `graphql_query` | Data Processing | query, mutation | Restricted |
| `web_scraper` | Web & Network | scrape, extract | Restricted |
| `url_analyzer` | Web & Network | analyze, validate | Safe |
| `api_tester` | Web & Network | test, validate | Restricted |
| `webhook_caller` | Web & Network | call, call_with_retry | Restricted |
| `webpage_monitor` | Web & Network | monitor, check_changes | Restricted |
| `sitemap_crawler` | Web & Network | crawl, extract_urls | Restricted |
| `email_sender` | Web & Network | send | Restricted |
| `environment_reader` | System | read, list | Restricted |
| `process_executor` | System | run, run_with_timeout | Privileged |
| `service_checker` | System | check, check_batch | Safe |
| `system_monitor` | System | get_metrics, get_all | Safe |
| `uuid_generator` | Utility | v4, v7, validate | Safe |
| `base64_encoder` | Utility | encode, decode | Safe |
| `hash_calculator` | Utility | hash, verify | Safe |
| `text_manipulator` | Utility | transform, analyze | Safe |
| `calculator` | Utility | evaluate | Safe |
| `date_time_handler` | Utility | now, parse, format | Safe |
| `diff_calculator` | Utility | text, json | Safe |
| `data_validation` | Utility | validate | Safe |
| `template_engine` | Utility | render | Safe |
| `database_connector` | Utility | query, execute | Restricted |
| `audio_processor` | Media | get_info, extract_metadata | Safe |
| `video_processor` | Media | get_info, extract_metadata | Safe |
| `image_processor` | Media | get_info, extract_metadata | Safe |
| `web_search` | Search | search | Restricted |
| `pdf_processor` | Document | extract_text, extract_metadata, extract_pages | Restricted |
| `citation_formatter` | Academic | format_citation, validate_bibliography, list_styles | Safe |
| `graph_builder` | Data Processing | create_graph, add_node, add_edge, analyze, export_json | Safe |

## Detailed Tool Documentation

### File System Tools

#### `file_operations`
Secure file system operations with path sandboxing.

**Operations:**
- `read` - Read file contents
- `write` - Write content to file
- `append` - Append content to file
- `delete` - Delete file
- `exists` - Check if file exists
- `list` - List directory contents
- `create_directory` - Create directory
- `copy` - Copy file
- `move` - Move/rename file

**Example:**
```lua
local file_tool = Tool.get("file_operations")

-- Read a file
local result = file_tool:execute({
    operation = "read",
    path = "/data/config.json"
})

-- Write a file
local result = file_tool:execute({
    operation = "write",
    path = "/output/report.txt",
    content = "Analysis complete"
})

-- List directory
local result = file_tool:execute({
    operation = "list",
    path = "/data",
    recursive = true
})
```

#### `archive_handler`
Create and extract archives (zip, tar, tar.gz).

**Operations:**
- `create` - Create archive from files
- `extract` - Extract archive contents
- `list` - List archive contents

**Example:**
```lua
local archive_tool = Tool.get("archive_handler")

-- Create a zip archive
local result = archive_tool:execute({
    operation = "create",
    format = "zip",
    source_path = "/data/reports",
    output_path = "/backups/reports.zip"
})

-- Extract archive
local result = archive_tool:execute({
    operation = "extract",
    archive_path = "/downloads/data.tar.gz",
    output_path = "/data/extracted"
})
```

#### `file_watcher`
Monitor file system changes in real-time.

**Operations:**
- `watch` - Start watching path
- `unwatch` - Stop watching path
- `get_changes` - Get accumulated changes

**Example:**
```lua
local watcher = Tool.get("file_watcher")

-- Start watching
watcher:execute({
    operation = "watch",
    path = "/data/incoming"
})

-- Check for changes
local result = watcher:execute({
    operation = "get_changes"
})
-- Returns: {changes = [{path = "...", kind = "create|modify|delete"}, ...]}
```

#### `file_converter`
Convert file encodings and formats.

**Operations:**
- `convert_encoding` - Change text encoding
- `convert_format` - Convert between formats

**Example:**
```lua
local converter = Tool.get("file_converter")

-- Convert encoding
local result = converter:execute({
    operation = "convert_encoding",
    source_path = "/data/legacy.txt",
    target_path = "/data/utf8.txt",
    from_encoding = "latin1",
    to_encoding = "utf8"
})
```

#### `file_search`
Search file contents with patterns.

**Operations:**
- `search` - Search with text pattern
- `search_regex` - Search with regex

**Example:**
```lua
local search_tool = Tool.get("file_search")

-- Search for text
local result = search_tool:execute({
    operation = "search",
    path = "/docs",
    pattern = "TODO",
    recursive = true,
    file_pattern = "*.md"
})
```

### Data Processing Tools

#### `json_processor`
Process JSON data with jq-like queries.

**Operations:**
- `parse` - Parse JSON string
- `stringify` - Convert to JSON string
- `query` - Query with jq syntax
- `transform` - Transform structure

**Example:**
```lua
local json_tool = Tool.get("json_processor")

-- Parse and query
local result = json_tool:execute({
    operation = "query",
    input = '{"users": [{"name": "Alice", "age": 30}]}',
    query = ".users[0].name"
})
-- Returns: "Alice"

-- Transform JSON
local result = json_tool:execute({
    operation = "transform",
    input = data,
    transformation = {
        name = "$.user.name",
        email = "$.user.contact.email"
    }
})
```

#### `csv_analyzer`
Analyze and process CSV files.

**Operations:**
- `analyze` - Get statistics and structure
- `parse` - Parse to JSON
- `write` - Write CSV from data
- `query` - Query with SQL-like syntax

**Example:**
```lua
local csv_tool = Tool.get("csv_analyzer")

-- Analyze CSV
local result = csv_tool:execute({
    operation = "analyze",
    path = "/data/sales.csv"
})
-- Returns: {rows = 1000, columns = [...], stats = {...}}

-- Parse CSV
local result = csv_tool:execute({
    operation = "parse",
    path = "/data/sales.csv",
    headers = true
})
```

#### `http_request`
Make HTTP requests with full control.

**Operations:**
- `get` - HTTP GET request
- `post` - HTTP POST request
- `put` - HTTP PUT request
- `delete` - HTTP DELETE request
- `head` - HTTP HEAD request

**Example:**
```lua
local http_tool = Tool.get("http_request")

-- GET request
local result = http_tool:execute({
    operation = "get",
    url = "https://api.example.com/data",
    headers = {
        ["Authorization"] = "Bearer token"
    }
})

-- POST request
local result = http_tool:execute({
    operation = "post",
    url = "https://api.example.com/users",
    body = JSON.stringify({name = "Alice"}),
    headers = {
        ["Content-Type"] = "application/json"
    }
})
```

#### `graphql_query`
Execute GraphQL queries and mutations.

**Operations:**
- `query` - Execute GraphQL query
- `mutation` - Execute GraphQL mutation

**Example:**
```lua
local graphql = Tool.get("graphql_query")

-- Query
local result = graphql:execute({
    operation = "query",
    endpoint = "https://api.example.com/graphql",
    query = [[
        query GetUser($id: ID!) {
            user(id: $id) {
                name
                email
            }
        }
    ]],
    variables = {id = "123"}
})
```

### Web & Network Tools

#### `web_scraper`
Extract content from web pages.

**Operations:**
- `scrape` - Extract with CSS selectors
- `extract` - Extract specific elements

**Example:**
```lua
local scraper = Tool.get("web_scraper")

-- Scrape page
local result = scraper:execute({
    operation = "scrape",
    url = "https://example.com",
    selectors = {
        title = "h1",
        paragraphs = "p",
        links = "a[href]"
    }
})
```

#### `url_analyzer`
Analyze and validate URLs.

**Operations:**
- `analyze` - Get URL components
- `validate` - Check if URL is valid

**Example:**
```lua
local analyzer = Tool.get("url_analyzer")

local result = analyzer:execute({
    operation = "analyze",
    url = "https://example.com/path?query=value#anchor"
})
-- Returns: {scheme = "https", host = "example.com", path = "/path", ...}
```

#### `api_tester`
Test REST APIs with validation.

**Operations:**
- `test` - Test endpoint
- `validate` - Validate response

**Example:**
```lua
local tester = Tool.get("api_tester")

local result = tester:execute({
    operation = "test",
    url = "https://api.example.com/health",
    method = "GET",
    expected_status = 200,
    timeout = 5000
})
```

#### `webhook_caller`
Call webhooks with retry logic.

**Operations:**
- `call` - Single webhook call
- `call_with_retry` - Call with retries

**Example:**
```lua
local webhook = Tool.get("webhook_caller")

local result = webhook:execute({
    operation = "call_with_retry",
    url = "https://hooks.example.com/notify",
    payload = {event = "user_signup", user_id = "123"},
    max_retries = 3,
    retry_delay = 1000
})
```

#### `webpage_monitor`
Monitor web pages for changes.

**Operations:**
- `monitor` - Check for changes
- `check_changes` - Compare snapshots

**Example:**
```lua
local monitor = Tool.get("webpage_monitor")

-- Initial snapshot
local result = monitor:execute({
    operation = "monitor",
    url = "https://example.com/status",
    selector = ".status-message"
})

-- Check for changes later
local result = monitor:execute({
    operation = "check_changes",
    url = "https://example.com/status",
    selector = ".status-message",
    previous_hash = result.output.hash
})
```

#### `sitemap_crawler`
Parse and crawl sitemaps.

**Operations:**
- `crawl` - Crawl sitemap
- `extract_urls` - Extract all URLs

**Example:**
```lua
local crawler = Tool.get("sitemap_crawler")

local result = crawler:execute({
    operation = "crawl",
    url = "https://example.com/sitemap.xml",
    max_depth = 2
})
-- Returns: {urls = [...], sitemap_count = N}
```

#### `email_sender`
Send emails via multiple providers.

**Operations:**
- `send` - Send email

**Example:**
```lua
local email = Tool.get("email_sender")

local result = email:execute({
    operation = "send",
    provider = "smtp",  -- or "sendgrid", "ses"
    to = "user@example.com",
    subject = "Report Ready",
    body = "Your analysis report is complete.",
    from = "noreply@example.com"
})
```

### System Integration Tools

#### `environment_reader`
Read environment variables safely.

**Operations:**
- `read` - Read single variable
- `list` - List all variables

**Example:**
```lua
local env = Tool.get("environment_reader")

-- Read variable
local result = env:execute({
    operation = "read",
    key = "API_ENDPOINT"
})

-- List all
local result = env:execute({
    operation = "list",
    filter = "^APP_"  -- Only vars starting with APP_
})
```

#### `process_executor`
Execute system commands securely.

**Operations:**
- `run` - Run command
- `run_with_timeout` - Run with timeout

**Example:**
```lua
local executor = Tool.get("process_executor")

local result = executor:execute({
    operation = "run_with_timeout",
    command = "python",
    args = ["analyze.py", "--input", "data.csv"],
    timeout = 30000,  -- 30 seconds
    cwd = "/scripts"
})
```

#### `service_checker`
Check service availability.

**Operations:**
- `check` - Check single service
- `check_batch` - Check multiple services

**Example:**
```lua
local checker = Tool.get("service_checker")

local result = checker:execute({
    operation = "check_batch",
    services = [
        {name = "api", url = "https://api.example.com/health"},
        {name = "db", host = "db.example.com", port = 5432}
    ]
})
```

#### `system_monitor`
Monitor system resources.

**Operations:**
- `get_metrics` - Get specific metrics
- `get_all` - Get all metrics

**Example:**
```lua
local monitor = Tool.get("system_monitor")

local result = monitor:execute({
    operation = "get_metrics",
    metrics = ["cpu", "memory", "disk"]
})
-- Returns: {cpu = {usage = 45.2}, memory = {used = 8.5, total = 16}, ...}
```

### Utility Tools

#### `uuid_generator`
Generate and validate UUIDs.

**Operations:**
- `v4` - Generate UUID v4
- `v7` - Generate UUID v7 (time-ordered)
- `validate` - Validate UUID

**Example:**
```lua
local uuid = Tool.get("uuid_generator")

local result = uuid:execute({
    operation = "v4"
})
-- Returns: "550e8400-e29b-41d4-a716-446655440000"
```

#### `base64_encoder`
Encode and decode Base64.

**Operations:**
- `encode` - Encode to Base64
- `decode` - Decode from Base64

**Example:**
```lua
local base64 = Tool.get("base64_encoder")

local result = base64:execute({
    operation = "encode",
    input = "Hello, World!"
})
-- Returns: "SGVsbG8sIFdvcmxkIQ=="
```

#### `hash_calculator`
Calculate cryptographic hashes.

**Operations:**
- `hash` - Calculate hash
- `verify` - Verify hash

**Example:**
```lua
local hasher = Tool.get("hash_calculator")

local result = hasher:execute({
    operation = "hash",
    input = "password123",
    algorithm = "sha256"
})
```

#### `text_manipulator`
Transform and analyze text.

**Operations:**
- `transform` - Apply transformations
- `analyze` - Get text statistics
- `extract` - Extract patterns

**Example:**
```lua
local text = Tool.get("text_manipulator")

-- Transform text
local result = text:execute({
    operation = "transform",
    input = "hello world",
    transformations = ["uppercase", "reverse"]
})
-- Returns: "DLROW OLLEH"

-- Analyze text
local result = text:execute({
    operation = "analyze",
    input = "Long text here..."
})
-- Returns: {words = 100, characters = 500, sentences = 10, ...}
```

#### `calculator`
Evaluate mathematical expressions.

**Operations:**
- `evaluate` - Evaluate expression

**Example:**
```lua
local calc = Tool.get("calculator")

local result = calc:execute({
    operation = "evaluate",
    input = "sqrt(16) + 2^3 - 1"
})
-- Returns: 11
```

#### `date_time_handler`
Handle dates and times.

**Operations:**
- `now` - Current timestamp
- `parse` - Parse date string
- `format` - Format date
- `add` - Add duration
- `diff` - Calculate difference

**Example:**
```lua
local datetime = Tool.get("date_time_handler")

-- Format current time
local result = datetime:execute({
    operation = "format",
    timestamp = os.time(),
    format = "%Y-%m-%d %H:%M:%S"
})

-- Calculate difference
local result = datetime:execute({
    operation = "diff",
    from = "2025-01-01",
    to = "2025-12-31",
    unit = "days"
})
```

#### `diff_calculator`
Calculate differences between texts/data.

**Operations:**
- `text` - Text diff
- `json` - JSON diff

**Example:**
```lua
local diff = Tool.get("diff_calculator")

local result = diff:execute({
    operation = "text",
    old_text = "Hello world",
    new_text = "Hello World!",
    format = "unified"
})
```

#### `data_validation`
Validate data against schemas.

**Operations:**
- `validate` - Validate with schema

**Example:**
```lua
local validator = Tool.get("data_validation")

local result = validator:execute({
    operation = "validate",
    input = {name = "Alice", age = 30},
    schema = {
        type = "object",
        properties = {
            name = {type = "string", minLength = 1},
            age = {type = "number", minimum = 0, maximum = 150}
        },
        required = ["name", "age"]
    }
})
```

#### `template_engine`
Render templates with data.

**Operations:**
- `render` - Render template

**Example:**
```lua
local template = Tool.get("template_engine")

local result = template:execute({
    operation = "render",
    template = "Hello {{name}}! You have {{count}} messages.",
    data = {name = "Alice", count = 5},
    engine = "handlebars"  -- or "tera"
})
```

#### `database_connector`
Connect to databases (SQLite, PostgreSQL, MySQL).

**Operations:**
- `query` - Execute SELECT query
- `execute` - Execute INSERT/UPDATE/DELETE

**Example:**
```lua
local db = Tool.get("database_connector")

-- Query data
local result = db:execute({
    operation = "query",
    connection_string = "sqlite:///data.db",
    query = "SELECT * FROM users WHERE active = ?",
    params = [true]
})

-- Insert data
local result = db:execute({
    operation = "execute",
    connection_string = "postgresql://localhost/mydb",
    query = "INSERT INTO logs (message, timestamp) VALUES (?, ?)",
    params = ["User logged in", os.time()]
})
```

### Media Processing Tools

#### `audio_processor`
Extract audio file metadata.

**Operations:**
- `get_info` - Get file information
- `extract_metadata` - Extract detailed metadata

**Example:**
```lua
local audio = Tool.get("audio_processor")

local result = audio:execute({
    operation = "get_info",
    path = "/media/podcast.mp3"
})
-- Returns: {duration = 3600, format = "mp3", bitrate = 128000, ...}
```

#### `video_processor`
Extract video file metadata.

**Operations:**
- `get_info` - Get file information
- `extract_metadata` - Extract detailed metadata

**Example:**
```lua
local video = Tool.get("video_processor")

local result = video:execute({
    operation = "get_info",
    path = "/media/presentation.mp4"
})
-- Returns: {duration = 1800, format = "mp4", resolution = "1920x1080", ...}
```

#### `image_processor`
Extract image file metadata.

**Operations:**
- `get_info` - Get file information
- `extract_metadata` - Extract detailed metadata

**Example:**
```lua
local image = Tool.get("image_processor")

local result = image:execute({
    operation = "get_info",
    path = "/images/photo.jpg"
})
-- Returns: {width = 1920, height = 1080, format = "jpeg", ...}
```

### Search Tools

#### `web_search`
Search the web using multiple providers.

**Operations:**
- `search` - Perform web search

**Example:**
```lua
local search = Tool.get("web_search")

local result = search:execute({
    operation = "search",
    query = "rust programming language tutorial",
    provider = "duckduckgo",  -- or "google", "bing", "brave"
    limit = 10
})
-- Returns: {results = [{title = "...", url = "...", snippet = "..."}, ...]}
```

### Document & Academic Tools

#### `pdf_processor`
Extract text and metadata from PDF documents with security controls.

**Operations:**
- `extract_text` - Extract all text from PDF
- `extract_metadata` - Get PDF metadata (size, creation date, etc.)
- `extract_pages` - Extract text from specific pages

**Security:** Restricted (file system access)
- Maximum file size: 10MB
- Path validation and sandboxing
- Timeout protection (30 seconds)

**Example:**
```lua
local pdf = Tool.get("pdf_processor")
local result = pdf:execute({
    operation = "extract_text",
    input = "/path/to/document.pdf"
})
-- Returns: {text = "extracted text...", file_path = "...", length = 12345}
```

#### `citation_formatter`
Format citations and bibliographies in APA, MLA, Chicago and 2,600+ academic styles.

**Operations:**
- `format_citation` - Format citations in specified style
- `validate_bibliography` - Validate bibliography format and entries
- `list_styles` - List available citation styles

**Security:** Safe (no file system or network access)
- Maximum 1000 bibliography entries
- Maximum 5KB per entry

**Example:**
```lua
local citations = Tool.get("citation_formatter")
local result = citations:execute({
    operation = "format_citation",
    input = "test-entry:\n  type: Article\n  author: Smith, John\n  title: Test Article\n  date: 2024",
    format = "yaml",  -- or "bibtex"
    style = "apa"      -- or "mla", "chicago", "harvard", "ieee", etc.
})
-- Returns: {citations = [...], reference_list = [...], style = "apa"}
```

#### `graph_builder`
Build and analyze graph data structures with JSON serialization.

**Operations:**
- `create_graph` - Create new graph (directed/undirected)
- `add_node` - Add node to existing graph
- `add_edge` - Add edge between nodes
- `analyze` - Analyze graph structure (degree, density, connectivity)
- `export_json` - Export graph to JSON format
- `import_json` - Import graph from JSON

**Security:** Safe (no file system or network access)
- Maximum 10,000 nodes
- Maximum 50,000 edges
- Maximum 10MB JSON size

**Example:**
```lua
local graph = Tool.get("graph_builder")
local result = graph:execute({
    operation = "create_graph",
    graph_type = "directed"  -- or "undirected"
})
-- Returns: {graph_type = "directed", nodes = [], edges = [], metadata = {...}}

-- Add nodes and edges
local updated = graph:execute({
    operation = "add_node",
    graph = JSON.stringify(result),
    node_id = "node1",
    label = "First Node",
    data = {value = 42}
})
```

## Security Considerations

### Security Levels

- **Safe**: No security risks, operates on provided data only
- **Restricted**: Limited access to system resources, sandboxed
- **Privileged**: Full system access, use with caution

### Sandboxing

File system tools are sandboxed by default:
```lua
-- Paths are restricted to allowed directories
local result = file_tool:execute({
    operation = "read",
    path = "../../../etc/passwd"  -- Will be rejected
})
```

### Resource Limits

All tools respect configured resource limits:
- Memory usage caps
- Execution timeouts
- Rate limiting for network operations
- File size restrictions

## Performance Characteristics

| Tool Category | Initialization | Operation Overhead |
|--------------|----------------|-------------------|
| File System | <5ms | <10ms |
| Data Processing | <3ms | <5ms |
| Web & Network | <10ms | Network dependent |
| System Integration | <5ms | System dependent |
| Utility | <2ms | <5ms |
| Media Processing | <5ms | File size dependent |
| Search | <5ms | Network dependent |

### Best Practices

1. **Reuse tool instances** - Tools are cached after first use
2. **Batch operations** - Use batch methods when available
3. **Handle errors** - Always check result.success
4. **Set timeouts** - For network and system operations
5. **Validate inputs** - Tools validate but pre-validation helps

## Common Patterns

### Tool Chaining
```lua
-- Read → Process → Write pipeline
local data = Tool.get("file_operations"):execute({
    operation = "read",
    path = "/input.json"
})

local processed = Tool.get("json_processor"):execute({
    operation = "transform",
    input = data.output,
    transformation = {...}
})

Tool.get("file_operations"):execute({
    operation = "write",
    path = "/output.json",
    content = processed.output
})
```

### Error Handling
```lua
local function safe_execute(tool_name, params)
    local tool = Tool.get(tool_name)
    if not tool then
        return {success = false, error = "Tool not found"}
    end
    
    local result = tool:execute(params)
    if not result.success then
        Logger.error("Tool failed", {
            tool = tool_name,
            error = result.error
        })
    end
    
    return result
end
```

### Tool Discovery
```lua
-- Find tools by capability
local function find_tools_by_operation(operation)
    local matching = {}
    for _, name in ipairs(Tool.list()) do
        local schema = Tool.get_schema(name)
        if schema.operations and schema.operations[operation] then
            table.insert(matching, name)
        end
    end
    return matching
end
```

---

**See Also**:
- [Agent API](agent-api.md) - Using tools with agents
- [Workflow API](workflow-api.md) - Tools in workflows
- [External Tools Guide](external-tools-guide.md) - Phase 3.1 tools detail
- [Examples](../../examples/) - Working tool examples