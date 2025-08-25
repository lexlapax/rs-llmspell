# Configuration Guide

**Version**: 0.6.0  
**Last Updated**: August 2025

> **📋 Quick Reference**: Complete configuration guide for LLMSpell including providers, security, resources, and external APIs.

**🔗 Navigation**: [← User Guide](README.md) | [Core Concepts](concepts.md) | [Getting Started](getting-started.md)

---

## Table of Contents

1. [Quick Start](#quick-start)
2. [Configuration Files](#configuration-files)
3. [LLM Providers](#llm-providers)
4. [Security Settings](#security-settings)
5. [Resource Limits](#resource-limits)
6. [Tool Configuration](#tool-configuration)
7. [External API Setup](#external-api-setup)
8. [Deployment Profiles](#deployment-profiles)
9. [Environment Variables](#environment-variables)
10. [Troubleshooting](#troubleshooting)

---

## Quick Start

Minimal configuration to get started:

```bash
# Set at least one LLM provider
export OPENAI_API_KEY="sk-..."

# Optional: Additional providers
export ANTHROPIC_API_KEY="sk-ant-..."

# Run with default configuration
./target/release/llmspell run script.lua
```

Using a configuration file:

```bash
# Run with specific config
./target/release/llmspell -c config.toml run script.lua
```

---

## Configuration Files

### Main Configuration Structure

```toml
# config.toml - Complete configuration example

[global]
debug = false
log_level = "info"
working_directory = "/workspace"

[providers]
default = "openai/gpt-4o-mini"

[security]
enabled = true
mode = "enforce"  # enforce, permissive, disabled

[resources]
default_limits = "default"  # strict, default, relaxed, unlimited

[tools]
enabled = ["*"]  # Enable all tools
disabled = []    # Disable specific tools

[state]
backend = "memory"  # memory, sled, rocksdb
persistence_enabled = false

[hooks]
enabled = true
builtin = ["rate_limit", "security"]

[events]
enabled = true
buffer_size = 10000
```

### Configuration Hierarchy

Configuration is loaded in order (later overrides earlier):
1. Built-in defaults
2. System config: `/etc/llmspell/config.toml`
3. User config: `~/.config/llmspell/config.toml`
4. Project config: `./llmspell.toml`
5. CLI specified: `-c custom.toml`
6. Environment variables
7. Command-line arguments

---

## LLM Providers

### Provider/Model Syntax

Use hierarchical naming: `provider/model`

```lua
local agent = Agent.builder()
    :model("openai/gpt-4")  -- Provider/model format
    :build()
```

### OpenAI

**Models:**
- `openai/gpt-4` - Most capable
- `openai/gpt-4-turbo` - Faster GPT-4
- `openai/gpt-4o` - Optimized variant
- `openai/gpt-4o-mini` - Smaller, faster
- `openai/gpt-3.5-turbo` - Cost-effective

**Configuration:**
```toml
[providers.openai]
api_key = "${OPENAI_API_KEY}"
base_url = "https://api.openai.com/v1"
default_model = "gpt-4o-mini"
organization_id = "org-..."  # Optional

[providers.openai.defaults]
temperature = 0.7
max_tokens = 2000
```

### Anthropic

**Models:**
- `anthropic/claude-3-opus` - Most capable
- `anthropic/claude-3-sonnet` - Balanced
- `anthropic/claude-3-haiku` - Fast
- `anthropic/claude-2.1` - Previous gen
- `anthropic/claude-instant` - Fastest

**Configuration:**
```toml
[providers.anthropic]
api_key = "${ANTHROPIC_API_KEY}"
base_url = "https://api.anthropic.com"
default_model = "claude-3-sonnet"
version = "2024-02-15"

[providers.anthropic.defaults]
max_tokens = 4000
temperature = 0.5
```

### Ollama (Local)

**Models:**
- `ollama/llama2` - Llama 2
- `ollama/mistral` - Mistral
- `ollama/codellama` - Code Llama
- `ollama/mixtral` - Mixtral

**Configuration:**
```toml
[providers.ollama]
base_url = "http://localhost:11434"
default_model = "llama2"
timeout = 300  # Seconds

[providers.ollama.defaults]
temperature = 0.8
num_predict = 2048
```

### Groq

**Models:**
- `groq/llama3-70b` - Llama 3 70B
- `groq/mixtral-8x7b` - Mixtral
- `groq/gemma-7b` - Gemma

**Configuration:**
```toml
[providers.groq]
api_key = "${GROQ_API_KEY}"
base_url = "https://api.groq.com/openai/v1"
default_model = "llama3-70b"
```

### Custom Providers

```toml
[providers.custom]
api_key = "${CUSTOM_API_KEY}"
base_url = "https://your-api.com"
auth_header = "X-API-Key"  # Or "Authorization"
auth_prefix = "Bearer"      # For Authorization header
default_model = "your-model"

[providers.custom.headers]
"X-Custom-Header" = "value"
"X-API-Version" = "2024-01"
```

---

## Security Settings

### Authentication

```toml
[security.authentication]
require_api_key = true
api_key_min_length = 32
api_key_rotation_days = 90
session_timeout_minutes = 30
max_failed_attempts = 5
lockout_duration_minutes = 15

[security.authentication.api_keys]
hash_algorithm = "argon2"
rate_limit_per_key = 1000  # per hour
```

### Sandboxing

```toml
[security.sandboxing]
enabled = true
implementation = "native"  # native, docker, firecracker

[security.sandboxing.filesystem]
enabled = true
allowed_paths = [
    "/workspace",
    "/tmp/llmspell"
]
denied_patterns = ["*.exe", "*.sh"]
max_file_size = "10MB"
max_open_files = 100

[security.sandboxing.network]
enabled = true
allowed_domains = [
    "api.openai.com",
    "*.anthropic.com"
]
deny_local_addresses = true
max_connections = 10
```

### Rate Limiting

```toml
[security.rate_limiting]
enabled = true
algorithm = "token_bucket"

[security.rate_limiting.global]
rate = 10000  # per minute
burst = 100

[security.rate_limiting.per_user]
rate = 1000
burst = 20

[[security.rate_limiting.tools]]
name = "web-search"
rate = 10
burst = 2
```

---

## Resource Limits

### Profiles

```toml
[resources.profiles.strict]
memory_limit = "256MB"
cpu_time_limit = "10s"
file_size_limit = "1MB"
max_operations = 100
max_concurrent = 2
operation_timeout = "5s"

[resources.profiles.default]
memory_limit = "512MB"
cpu_time_limit = "30s"
file_size_limit = "10MB"
max_operations = 1000
max_concurrent = 5
operation_timeout = "30s"

[resources.profiles.relaxed]
memory_limit = "2GB"
cpu_time_limit = "5m"
file_size_limit = "100MB"
max_operations = 10000
max_concurrent = 20
operation_timeout = "5m"
```

### Per-Tool Limits

```toml
[[resources.tool_limits]]
tool = "web-scraper"
memory_limit = "1GB"
timeout = "60s"
max_concurrent = 3

[[resources.tool_limits]]
tool = "file-operations"
file_size_limit = "50MB"
max_operations = 500
```

---

## Tool Configuration

### File Operations

```toml
[tools.file_operations]
enabled = true
allowed_paths = ["/workspace", "/tmp"]
max_file_size = "10MB"
allowed_extensions = ["txt", "json", "csv", "md"]
denied_extensions = ["exe", "dll", "so"]
```

### Web Tools

```toml
[tools.web]
enabled = true
timeout = 30
max_redirects = 5
user_agent = "LLMSpell/0.6.0"

[tools.web.scraper]
max_depth = 3
max_pages = 100
respect_robots_txt = true

[tools.web.search]
provider = "brave"  # google, brave, serpapi, serperdev
max_results = 10
safe_search = true
```

### Database Tools

```toml
[tools.database]
enabled = false  # Disabled by default for security

[[tools.database.connections]]
name = "primary"
type = "postgresql"
host = "localhost"
port = 5432
database = "llmspell"
username = "${DB_USER}"
password = "${DB_PASS}"
ssl_mode = "require"
max_connections = 10
```

---

## External API Setup

### Web Search APIs

#### Google Custom Search

1. **Create Search Engine:**
   - Visit [Google Programmable Search](https://programmablesearchengine.google.com/)
   - Create new search engine
   - Note the Search Engine ID (cx)

2. **Get API Key:**
   - Go to [Google Cloud Console](https://console.cloud.google.com/)
   - Enable Custom Search API
   - Create API key

3. **Configure:**
```toml
[tools.web.search.google]
api_key = "${GOOGLE_API_KEY}"
search_engine_id = "${GOOGLE_SEARCH_ENGINE_ID}"
# Free: 100/day, Paid: $5/1000 queries
```

#### Brave Search

1. **Sign Up:**
   - Visit [Brave Search API](https://brave.com/search/api/)
   - Create account

2. **Configure:**
```toml
[tools.web.search.brave]
api_key = "${BRAVE_API_KEY}"
# Free: 2000/month
```

#### SerpAPI

```toml
[tools.web.search.serpapi]
api_key = "${SERPAPI_KEY}"
# Free trial: 100 searches
```

#### SerperDev

```toml
[tools.web.search.serperdev]
api_key = "${SERPERDEV_KEY}"
# Free: 2500 searches
```

### Email Services

#### SendGrid

```toml
[tools.email.sendgrid]
api_key = "${SENDGRID_API_KEY}"
from_email = "noreply@example.com"
from_name = "LLMSpell"
# Free: 100/day
```

#### AWS SES

```toml
[tools.email.aws_ses]
access_key_id = "${AWS_ACCESS_KEY_ID}"
secret_access_key = "${AWS_SECRET_ACCESS_KEY}"
region = "us-east-1"
from_email = "noreply@example.com"
```

---

## Deployment Profiles

### Development

```toml
# config.dev.toml
[global]
debug = true
log_level = "debug"

[security]
mode = "permissive"

[resources]
default_limits = "relaxed"
```

### Staging

```toml
# config.staging.toml
[global]
debug = false
log_level = "info"

[security]
mode = "enforce"

[resources]
default_limits = "default"

[monitoring]
enabled = true
metrics_port = 9090
```

### Production

```toml
# config.prod.toml
[global]
debug = false
log_level = "warn"

[security]
mode = "enforce"
audit_level = "verbose"

[resources]
default_limits = "strict"
enforce_limits = true

[monitoring]
enabled = true
metrics_port = 9090
alerts_enabled = true

[backup]
enabled = true
interval = "1h"
retention_days = 30
```

---

## Environment Variables

### Core Variables

```bash
# LLM Providers
OPENAI_API_KEY="sk-..."
ANTHROPIC_API_KEY="sk-ant-..."
GROQ_API_KEY="gsk_..."

# Configuration
LLMSPELL_CONFIG="/path/to/config.toml"
LLMSPELL_LOG_LEVEL="info"
LLMSPELL_DEBUG="false"

# Security
LLMSPELL_SECURITY_MODE="enforce"
LLMSPELL_API_KEY="your-api-key"

# Resources
LLMSPELL_RESOURCE_LIMITS="default"
LLMSPELL_MAX_MEMORY="512MB"

# State
LLMSPELL_STATE_BACKEND="sled"
LLMSPELL_STATE_PATH="/var/lib/llmspell"
```

### Provider-Specific

```bash
# OpenAI
OPENAI_API_KEY="sk-..."
OPENAI_ORG_ID="org-..."
OPENAI_BASE_URL="https://api.openai.com/v1"

# Anthropic
ANTHROPIC_API_KEY="sk-ant-..."
ANTHROPIC_BASE_URL="https://api.anthropic.com"

# Ollama
OLLAMA_BASE_URL="http://localhost:11434"

# Custom
CUSTOM_API_KEY="..."
CUSTOM_BASE_URL="https://your-api.com"
```

### Tool APIs

```bash
# Web Search
GOOGLE_API_KEY="..."
GOOGLE_SEARCH_ENGINE_ID="..."
BRAVE_API_KEY="..."
SERPAPI_KEY="..."
SERPERDEV_KEY="..."

# Email
SENDGRID_API_KEY="..."
AWS_ACCESS_KEY_ID="..."
AWS_SECRET_ACCESS_KEY="..."

# Database
DB_HOST="localhost"
DB_PORT="5432"
DB_NAME="llmspell"
DB_USER="..."
DB_PASS="..."
```

---

## Troubleshooting

### Common Issues

#### "No API key found"
```bash
# Check environment
echo $OPENAI_API_KEY

# Set if missing
export OPENAI_API_KEY="sk-..."

# Or use config file
echo 'api_key = "sk-..."' >> config.toml
```

#### "Rate limit exceeded"
```toml
# Adjust rate limits
[security.rate_limiting.per_user]
rate = 2000  # Increase limit
burst = 50
```

#### "Memory limit exceeded"
```toml
# Increase memory limits
[resources.profiles.default]
memory_limit = "1GB"  # Increase from 512MB
```

#### "Connection timeout"
```toml
# Increase timeouts
[providers.openai]
timeout = 60  # Seconds

[tools.web]
timeout = 45
```

### Debug Mode

Enable detailed logging:

```bash
# Via environment
export LLMSPELL_DEBUG=true
export LLMSPELL_LOG_LEVEL=debug

# Via config
[global]
debug = true
log_level = "debug"

# Via CLI
./llmspell --debug --log-level debug run script.lua
```

### Validation

Check configuration:

```bash
# Validate config file
./llmspell validate -c config.toml

# Test provider connection
./llmspell test-provider openai

# List available tools
./llmspell list-tools

# Check resource usage
./llmspell stats
```

---

## Best Practices

1. **Use Environment Variables for Secrets**
   - Never commit API keys to version control
   - Use `.env` files with `.gitignore`

2. **Start with Strict Limits**
   - Begin with `strict` resource profile
   - Increase limits as needed

3. **Enable Security in Production**
   - Always use `mode = "enforce"`
   - Enable audit logging
   - Rotate API keys regularly

4. **Monitor Resource Usage**
   - Track memory and CPU usage
   - Set up alerts for limits
   - Review logs regularly

5. **Use Deployment Profiles**
   - Separate configs for dev/staging/prod
   - Test configuration changes in staging
   - Keep production config minimal

---

## See Also

- [Core Concepts](concepts.md) - Understanding configuration context
- [Getting Started](getting-started.md) - Quick setup guide
- [Security Guide](advanced/security.md) - Detailed security configuration
- [API Documentation](api/README.md) - Provider-specific APIs
- [Examples](../../examples/) - Configuration examples