# Release Notes - v0.3.0

**Release Date**: 2025-01-30  
**Codename**: "Tool Enhancement & Agent Infrastructure"  
**Status**: Pre-release (Breaking Changes Expected)

## 🎉 What's New in v0.3.0

Rs-LLMSpell v0.3.0 delivers a massive expansion of capabilities with **34 production-ready tools**, complete **agent infrastructure**, and **workflow orchestration**. This release represents the completion of Phase 3 and establishes the foundation for advanced LLM-driven automation.

### Quick Highlights

- **🔧 34 Production Tools** - Up from 26, organized in 9 categories
- **🤖 Agent Infrastructure** - Complete lifecycle management and LLM integration
- **🔄 Workflow Patterns** - Sequential, conditional, loop, and parallel execution
- **🚀 Exceptional Performance** - 52,600x faster than requirements
- **🔒 Security Hardened** - Zero known vulnerabilities, comprehensive protection
- **📊 95% Consistency** - Standardized parameters across all tools

## Major Features

### 1. Expanded Tool Ecosystem (34 Tools)

**New Tools Added (8)**:
- `web_scraper` - Extract content from web pages with CSS selectors
- `url_analyzer` - Parse and validate URLs
- `api_tester` - Test REST APIs with validation
- `webhook_caller` - Call webhooks with retry logic
- `webpage_monitor` - Monitor pages for changes
- `sitemap_crawler` - Parse and crawl sitemaps
- `email_sender` - Send emails via SMTP/SendGrid/SES
- `database_connector` - Query PostgreSQL, MySQL, SQLite

**Tool Categories**:
- **File System** (5): file_operations, archive_handler, file_watcher, file_converter, file_search
- **Data Processing** (4): json_processor, csv_analyzer, http_request, graphql_query
- **Web & Network** (7): web_search, web_scraper, url_analyzer, api_tester, webhook_caller, webpage_monitor, sitemap_crawler
- **System Integration** (4): environment_reader, process_executor, service_checker, system_monitor
- **Utilities** (10): calculator, text_manipulator, hash_calculator, uuid_generator, base64_encoder, date_time_handler, diff_calculator, data_validation, template_engine, database_connector
- **Media Processing** (3): audio_processor, video_processor, image_processor
- **Communication** (1): email_sender

### 2. Agent Infrastructure

Create and manage LLM agents with full lifecycle support:

```lua
-- Create an agent with any supported model
local agent = Agent.create({
    name = "research-agent",
    model = "openai/gpt-4",  -- or "anthropic/claude-3", etc.
    system_prompt = "You are a research assistant",
    temperature = 0.7
})

-- Execute tasks
local response = agent:execute({
    prompt = "Analyze this data and provide insights"
})

-- Use tools from within agents
local tools = agent:discoverTools()
local result = agent:invokeTool("web_search", {
    operation = "search",
    input = "latest AI developments"
})
```

**Features**:
- Multiple agent types (Basic, LLM, Tool-Orchestrator)
- Complete lifecycle management (7 states)
- Tool discovery and invocation
- Metrics and monitoring
- Agent composition patterns

### 3. Workflow Orchestration

Build complex multi-step workflows:

```lua
-- Sequential workflow
local workflow = Workflow.sequential({
    name = "data-pipeline",
    steps = {
        {type = "tool", name = "http_request", params = {...}},
        {type = "agent", name = "analyzer", params = {...}},
        {type = "tool", name = "file_operations", params = {...}}
    }
})

-- Conditional workflow
local conditional = Workflow.conditional({
    condition = function(context) return context.data.score > 0.8 end,
    if_true = {type = "workflow", name = "high-score-flow"},
    if_false = {type = "workflow", name = "low-score-flow"}
})

-- Parallel execution
local parallel = Workflow.parallel({
    branches = {...},
    join_strategy = "wait_all"
})
```

### 4. Enhanced Security

Comprehensive security hardening across all components:

- **Path Traversal Protection**: Canonical resolution, symlink blocking
- **Command Injection Prevention**: No shell interpretation, sanitized arguments
- **DoS Protection**: Resource limits, timeout enforcement
- **SSRF Prevention**: URL validation, private IP blocking
- **Rate Limiting**: Token bucket algorithm with configurable limits
- **Input Validation**: Length limits, format validation, injection prevention

## Breaking Changes

⚠️ **This is a pre-1.0 release with breaking changes**

### Parameter Standardization

All tools now use consistent parameter names:
- Primary data: `input` (was: text, content, data, expression, query, etc.)
- File paths: `path` for single files, `source_path`/`target_path` for operations
- All multi-function tools require `operation` parameter

**Before**:
```lua
calculator:execute({expression = "2+2"})
hash:execute({data = "hello", algorithm = "sha256"})
```

**After**:
```lua
calculator:execute({operation = "evaluate", input = "2+2"})
hash:execute({operation = "hash", input = "hello", algorithm = "sha256"})
```

### API Changes

- Removed all `*Async` methods - everything is now synchronous
- Agent creation requires model specification
- Tool.get() returns tool instances, not direct execution

See [CHANGELOG.md](CHANGELOG.md) for complete breaking changes documentation.

## Performance

Exceptional performance maintained and improved:
- **Tool initialization**: 107-190ns (52,600x faster than 10ms requirement)
- **Agent creation**: <50ms including provider setup
- **Workflow execution**: <20ms overhead per step
- **Memory usage**: Reduced by 40% through shared utilities

## Getting Started

### Installation

```bash
git clone https://github.com/lexlapax/rs-llmspell
cd rs-llmspell
cargo build --release
```

### Quick Example

```lua
-- Create an agent
local agent = Agent.create({
    model = "openai/gpt-4",
    system_prompt = "You are a helpful assistant"
})

-- Use tools
local scraper = Tool.get("web_scraper")
local content = scraper:execute({
    operation = "scrape",
    input = "https://example.com",
    selector = "article"
})

-- Process with agent
local analysis = agent:execute({
    prompt = "Summarize this content: " .. content.data.text
})

-- Save results
local file_ops = Tool.get("file_operations")
file_ops:execute({
    operation = "write",
    path = "summary.txt",
    input = analysis.content
})
```

## Migration from v0.2.x

Key changes to update your scripts:

1. **Update parameter names**: Replace old parameter names with standardized ones
2. **Add operation parameters**: Tools now require explicit operations
3. **Remove async patterns**: All operations are now synchronous
4. **Update response parsing**: Use new standardized response format

Example migration:
```lua
-- Old (v0.2.x)
local result = Tool.executeAsync("calculator", {expression = "2+2"})

-- New (v0.3.0)
local calc = Tool.get("calculator")
local result = calc:execute({operation = "evaluate", input = "2+2"})
```

## Known Issues

- Tool invocation from agents requires wrapped parameters format (workaround available)
- JavaScript support planned for future release (currently Lua only)

## What's Next

**Phase 4** (Weeks 17-18) will add:
- Hook system for lifecycle events
- Event bus for component communication
- Custom hook registration from scripts
- Built-in hooks for logging, metrics, and debugging

## Documentation

- [User Guide](docs/user-guide/README.md) - Getting started and tutorials
- [Tool Reference](docs/user-guide/tool-reference.md) - All 34 tools documented
- [Developer Guide](docs/developer-guide/README.md) - Contributing and architecture
- [Examples](examples/) - Working code for all features

## Support

- **Issues**: [GitHub Issues](https://github.com/lexlapax/rs-llmspell/issues)
- **Discussions**: [GitHub Discussions](https://github.com/lexlapax/rs-llmspell/discussions)
- **Documentation**: [Complete Docs](docs/README.md)

---

**Thank you** to all contributors who made this release possible! Rs-LLMSpell v0.3.0 represents a major milestone in our journey toward scriptable LLM orchestration.