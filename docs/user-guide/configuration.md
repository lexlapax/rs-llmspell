# Configuration Guide

**Version**: 0.8.10  
**Last Updated**: December 2024

> **📋 Quick Reference**: Complete configuration guide for LLMSpell including providers, security, resources, and external APIs.

**🔗 Navigation**: [← User Guide](README.md) | [Core Concepts](concepts.md) | [Getting Started](getting-started.md)

---

## Table of Contents

1. [Quick Start](#quick-start)
2. [Configuration Files](#configuration-files)
3. [LLM Providers](#llm-providers)
4. [RAG Configuration](#rag-configuration) ⭐ **Phase 8.10.6**
5. [Multi-Tenancy](#multi-tenancy) ⭐ **Phase 8.10.6**
6. [State & Sessions](#state--sessions)
7. [Security Settings](#security-settings)
8. [Tool Configuration](#tool-configuration)
9. [External API Setup](#external-api-setup)
10. [Deployment Profiles](#deployment-profiles)
11. [Environment Variables](#environment-variables)
12. [Troubleshooting](#troubleshooting)

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

# Main runtime configuration
[runtime]
max_concurrent_scripts = 10
script_timeout_seconds = 300
enable_streaming = true

# Provider configuration
[providers]
default_provider = "openai"

[providers.providers.openai]
provider_type = "openai"
api_key_env = "OPENAI_API_KEY"
default_model = "gpt-4o-mini"
temperature = 0.7
max_tokens = 2000

# RAG (Retrieval-Augmented Generation) - Phase 8.10.6
[rag]
enabled = false  # Enable for RAG functionality
multi_tenant = false

[rag.vector_storage]
dimensions = 384  # 384, 768, 1536, 3072
backend = "hnsw"
persistence_path = "./data/rag/vectors"
max_memory_mb = 500

[rag.vector_storage.hnsw]
m = 16
ef_construction = 200
ef_search = 50
max_elements = 1000000
metric = "cosine"

# Security configuration
[runtime.security]
allow_file_access = false
allow_network_access = true
allow_process_spawn = false
max_memory_bytes = 50000000

# State persistence
[runtime.state_persistence]
enabled = true
backend_type = "memory"
migration_enabled = false
backup_enabled = false

# Events system
[events]
enabled = true
buffer_size = 10000
emit_timing_events = true
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

## RAG Configuration ⭐ **Phase 8.10.6**

LLMSpell includes comprehensive RAG (Retrieval-Augmented Generation) capabilities with HNSW vector storage, multi-tenant isolation, and cost optimization.

### Basic RAG Setup

```toml
[rag]
enabled = true
multi_tenant = false  # Enable for tenant isolation

# Vector storage configuration
[rag.vector_storage]
dimensions = 768      # 384, 768, 1536, 3072 supported
backend = "hnsw"      # Only HNSW supported currently
persistence_path = "./data/rag/vectors"
max_memory_mb = 1024

# HNSW algorithm parameters
[rag.vector_storage.hnsw]
m = 16                      # Connections per node
ef_construction = 200       # Build-time search width
ef_search = 100            # Query-time search width
max_elements = 1000000     # Maximum vectors
metric = "cosine"          # cosine, euclidean, inner_product
allow_replace_deleted = true
num_threads = 4
```

### Embedding Configuration

```toml
[rag.embedding]
default_provider = "openai"  # Provider for embeddings
cache_enabled = true         # 70% cost reduction
cache_size = 20000          # Cached embeddings
cache_ttl_seconds = 3600    # 1 hour cache
batch_size = 32             # Batch processing
timeout_seconds = 30
max_retries = 3
```

### Document Chunking

```toml
[rag.chunking]
strategy = "sliding_window"  # sliding_window, semantic, sentence
chunk_size = 512            # Tokens per chunk
overlap = 64               # Overlap between chunks
max_chunk_size = 2048      # Hard limit
min_chunk_size = 100       # Quality threshold
```

### RAG Caching (70% Cost Reduction)

```toml
[rag.cache]
# Search result caching
search_cache_enabled = true
search_cache_size = 5000
search_cache_ttl_seconds = 600

# Document caching
document_cache_enabled = true
document_cache_size_mb = 200
```

### HNSW Optimization Profiles

**Small Dataset (<10K vectors):**
```toml
[rag.vector_storage.hnsw]
m = 12
ef_construction = 100
ef_search = 50
max_elements = 10000
```

**Large Dataset (100K-1M vectors):**
```toml
[rag.vector_storage.hnsw]
m = 32
ef_construction = 400
ef_search = 200
max_elements = 1000000
num_threads = 4
```

**Speed Optimized:**
```toml
[rag.vector_storage.hnsw]
m = 8
ef_construction = 50
ef_search = 25
```

**Accuracy Optimized:**
```toml
[rag.vector_storage.hnsw]
m = 48
ef_construction = 500
ef_search = 300
```

### Session Collections

For conversational memory:

```toml
[rag.sessions]
enable_session_collections = true
session_collection_ttl = 3600  # 1 hour
max_session_vectors = 1000
auto_cleanup = true
```

### Supported Vector Dimensions

| Dimensions | Model Example | Use Case |
|------------|---------------|----------|
| 384 | all-MiniLM-L6-v2 | Fast, small memory |
| 768 | all-mpnet-base-v2 | Balanced |
| 1536 | text-embedding-3-small | OpenAI standard |
| 3072 | text-embedding-3-large | Maximum accuracy |

---

## Multi-Tenancy ⭐ **Phase 8.10.6**

Complete tenant isolation for RAG and state data.

### Basic Multi-Tenant Setup

```toml
[rag]
enabled = true
multi_tenant = true  # IMPORTANT: Enable tenant isolation

[rag.multi_tenant_settings]
max_vectors_per_tenant = 100000
tenant_ttl_hours = 168        # 7 days retention
auto_cleanup = true
strict_isolation = true       # No cross-tenant access
max_concurrent_operations = 10
rate_limit_per_minute = 100
```

### Tenant Resource Quotas

```toml
[tenancy.quotas]
# Storage limits per tenant
max_storage_mb = 1000
max_vectors = 50000
max_collections = 10

# Compute limits
max_queries_per_minute = 100
max_ingestion_rate = 50      # documents per minute
max_concurrent_operations = 5

# Cost control
max_embedding_tokens_per_day = 100000
billing_enabled = true
```

### Tenant Lifecycle

```toml
[tenancy.lifecycle]
default_retention_days = 30
auto_suspend_inactive_days = 7
purge_deleted_after_days = 90
backup_before_deletion = true
```

---

## State & Sessions

### State Persistence

```toml
[runtime.state_persistence]
enabled = true
backend_type = "sled"        # memory, sled, file
migration_enabled = true
backup_enabled = true
backup_on_migration = true
schema_directory = "./schemas"
max_state_size_bytes = 10000000

[runtime.state_persistence.backup]
backup_dir = "./backups"
compression_enabled = true
compression_type = "zstd"
compression_level = 3
incremental_enabled = true
max_backups = 10
max_backup_age = 2592000     # 30 days
```

### Session Management

```toml
[runtime.sessions]
enabled = true
max_sessions = 100
max_artifacts_per_session = 1000
artifact_compression_threshold = 10240  # 10KB
session_timeout_seconds = 3600
storage_backend = "memory"   # memory, sled
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
# rag-development.toml
[rag]
enabled = true
multi_tenant = false

[rag.vector_storage]
dimensions = 384
backend = "hnsw" 
max_memory_mb = 512

[rag.vector_storage.hnsw]
m = 8
ef_construction = 50
ef_search = 25
max_elements = 10000
num_threads = 2

[rag.embedding]
default_provider = "openai"
cache_enabled = false      # Test fresh each time
batch_size = 4            # Small batches for debugging
timeout_seconds = 10

[rag.cache]
search_cache_enabled = false
document_cache_enabled = false
```

### Production

```toml
# rag-production.toml
[rag]
enabled = true
multi_tenant = true       # Enable tenant isolation

[rag.vector_storage]
dimensions = 768          # Better accuracy
backend = "hnsw"
persistence_path = "/var/lib/llmspell/rag/vectors"
max_memory_mb = 4096

[rag.vector_storage.hnsw]
m = 16                    # Balanced performance
ef_construction = 200
ef_search = 50
max_elements = 5000000    # 5M vectors
num_threads = 4

[rag.embedding]
default_provider = "openai"
cache_enabled = true      # 70% cost reduction
cache_size = 20000
cache_ttl_seconds = 1800  # 30 minutes
batch_size = 32
timeout_seconds = 45
max_retries = 3

[rag.cache]
search_cache_enabled = true
search_cache_size = 5000
search_cache_ttl_seconds = 600

# Multi-tenant quotas
[rag.multi_tenant_settings]
max_vectors_per_tenant = 100000
tenant_ttl_hours = 168
strict_isolation = true
max_concurrent_operations = 10
```

### Multi-Tenant SaaS

```toml
# rag-multi-tenant.toml
[rag]
enabled = true
multi_tenant = true

[rag.vector_storage]
dimensions = 768
backend = "hnsw"
persistence_path = "./data/rag/vectors"

[rag.vector_storage.hnsw]
m = 32                    # More connections for better recall
ef_construction = 400     # High quality
ef_search = 100
max_elements = 10000000   # 10M vectors for many tenants
num_threads = 8

[rag.embedding]
cache_enabled = true
cache_size = 50000        # Large cache for multiple tenants
cache_ttl_seconds = 7200  # 2 hours
batch_size = 64
max_retries = 5

[tenancy.quotas]
max_vectors = 50000
max_queries_per_minute = 100
max_embedding_tokens_per_day = 100000
billing_enabled = true
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

# RAG Configuration (Phase 8.10.6)
LLMSPELL_RAG_ENABLED="true"
LLMSPELL_RAG_MULTI_TENANT="false"
LLMSPELL_RAG_DIMENSIONS="768"
LLMSPELL_RAG_BACKEND="hnsw"
LLMSPELL_RAG_PERSISTENCE_PATH="/var/lib/llmspell/rag"
LLMSPELL_RAG_MAX_MEMORY_MB="1024"

# HNSW Configuration
LLMSPELL_HNSW_M="16"
LLMSPELL_HNSW_EF_CONSTRUCTION="200"
LLMSPELL_HNSW_EF_SEARCH="50"
LLMSPELL_HNSW_MAX_ELEMENTS="1000000"
LLMSPELL_HNSW_METRIC="cosine"

# Embedding Configuration
LLMSPELL_EMBEDDING_PROVIDER="openai"
LLMSPELL_EMBEDDING_CACHE_ENABLED="true"
LLMSPELL_EMBEDDING_CACHE_SIZE="20000"
LLMSPELL_EMBEDDING_BATCH_SIZE="32"

# Multi-Tenancy (Phase 8.10.6)
LLMSPELL_TENANT_MAX_VECTORS="50000"
LLMSPELL_TENANT_RATE_LIMIT="100"
LLMSPELL_TENANT_TTL_HOURS="168"

# State & Sessions
LLMSPELL_STATE_ENABLED="true"
LLMSPELL_STATE_BACKEND="sled"
LLMSPELL_STATE_PATH="/var/lib/llmspell"
LLMSPELL_SESSIONS_ENABLED="false"
LLMSPELL_SESSIONS_BACKEND="memory"
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
[providers.providers.openai]
timeout_seconds = 60

[rag.embedding]
timeout_seconds = 45
```

#### "RAG not enabled"
```bash
# Check RAG configuration
export LLMSPELL_RAG_ENABLED="true"

# Or in config file
[rag]
enabled = true
```

#### "Vector storage error"
```toml
# Check storage path permissions
[rag.vector_storage]
persistence_path = "./data/rag/vectors"  # Ensure writable

# Reduce memory if needed
max_memory_mb = 256  # Lower limit
```

#### "HNSW index build failed"
```toml
# Use smaller parameters for limited memory
[rag.vector_storage.hnsw]
m = 8                    # Reduce connections
ef_construction = 50     # Lower quality
max_elements = 10000     # Smaller dataset
```

#### "Multi-tenant isolation error"
```toml
# Ensure proper tenant configuration
[rag]
multi_tenant = true      # Must be enabled

[rag.multi_tenant_settings]
strict_isolation = true  # Enforce separation
```

#### "Embedding cache miss"
```bash
# Check cache status
[rag.embedding]
cache_enabled = true
cache_size = 20000      # Increase if needed
cache_ttl_seconds = 3600 # Extend TTL
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

# Test RAG functionality
./llmspell test-rag --config rag-development.toml

# Check vector storage status
./llmspell exec 'local stats = RAG.get_stats("default", nil); print(JSON.stringify(stats))'

# List available tools
./llmspell list-tools

# Check resource usage and tenant quotas
./llmspell stats

# Validate RAG configuration
./scripts/validate-rag-configs.sh
```

---

## Best Practices

1. **Use Environment Variables for Secrets**
   - Never commit API keys to version control
   - Use `.env` files with `.gitignore`

2. **Start with RAG Development Profile**
   - Use `rag-development.toml` for testing
   - Small datasets and fast iteration
   - Disable caching for fresh results

3. **Enable Multi-Tenancy in Production**
   - Always set `multi_tenant = true` in production
   - Configure proper tenant quotas
   - Enable strict isolation

4. **Optimize HNSW for Your Use Case**
   - Small datasets: Use `speed_optimized` profile
   - Large datasets: Use `accuracy_optimized` profile
   - Monitor memory usage and vector counts

5. **Enable RAG Caching (70% Cost Reduction)**
   - Set `cache_enabled = true` for embeddings
   - Use appropriate cache sizes and TTL
   - Monitor cache hit rates

6. **Monitor Resource Usage**
   - Track vector storage growth
   - Monitor tenant quotas and usage
   - Set up alerts for HNSW memory limits

7. **Use Configuration Profiles**
   - `rag-development.toml` for development
   - `rag-production.toml` for production
   - `rag-multi-tenant.toml` for SaaS deployments

---

## See Also

- [Core Concepts](concepts.md) - Understanding RAG and multi-tenancy
- [Getting Started](getting-started.md) - Quick setup with RAG
- [API Documentation](api/README.md) - RAG and provider APIs
- [Configuration Examples](../../examples/script-users/configs/) - 15+ config files
- [RAG Examples](../../examples/script-users/applications/) - RAG-powered applications