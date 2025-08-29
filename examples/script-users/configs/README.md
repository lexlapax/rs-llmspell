# LLMSpell Configuration Guide

**Status**: 🚀 **Phase 8.10.6** - Complete configuration set with RAG, state persistence, providers, and applications

This directory contains 15 pre-configured templates for various LLMSpell use cases, from minimal tool-only setups to production RAG deployments.

## 📊 Configuration Overview

**15 Configuration Files** organized by category:

### 📁 Files in this Directory

| File | Category | Purpose |
|------|----------|---------|
| `minimal.toml` | Core | Minimal setup for tool-only scripts |
| `basic.toml` | Core | Basic state operations without providers |
| `example-providers.toml` | Providers | OpenAI + Anthropic provider setup |
| `applications.toml` | Providers | Production application configuration |
| `cookbook.toml` | Patterns | Configuration for cookbook examples |
| `llmspell.toml` | System | Main LLMSpell system configuration |
| `state-enabled.toml` | State | State persistence with memory backend |
| `session-enabled.toml` | State | Session management and artifacts |
| `migration-enabled.toml` | State | State migration and schema evolution |
| `backup-enabled.toml` | State | Backup and recovery configuration |
| `rag-basic.toml` | RAG | Getting started with RAG |
| `rag-development.toml` | RAG | Local development with mock backend |
| `rag-production.toml` | RAG | Production RAG deployment |
| `rag-performance.toml` | RAG | High-performance RAG configuration |
| `rag-multi-tenant.toml` | RAG | Multi-tenant SaaS configuration |

### 📄 Additional Files
- `MIGRATION.md` - Guide for migrating between configurations
- `validate-rag-configs.sh` - Script to validate RAG configurations

## Available Configurations

### Core Configurations

### `minimal.toml`
- **Use Case**: Minimal setup for tool-only scripts
- **Best For**: Testing tools without LLM providers
- **Key Features**:
  - No providers configured
  - File access enabled
  - Network and process spawn disabled
  - Lightweight and fast startup

### `basic.toml`
- **Use Case**: Basic state operations
- **Best For**: Learning state persistence
- **Key Features**:
  - In-memory state backend
  - No providers configured
  - 10MB state size limit
  - Migration and backup disabled

### `example-providers.toml`
- **Use Case**: Agent and workflow examples
- **Best For**: Learning LLM integration
- **Key Features**:
  - OpenAI provider (GPT-4)
  - Anthropic provider (Claude-3)
  - API keys from environment
  - Safe security level

### `applications.toml`
- **Use Case**: Production applications
- **Best For**: Running complete application examples
- **Key Features**:
  - Multiple providers configured
  - Rate limiting enabled
  - Retry logic configured
  - Production-ready settings

### `cookbook.toml`
- **Use Case**: Cookbook pattern examples
- **Best For**: Production patterns and best practices
- **Key Features**:
  - Comprehensive provider setup
  - State persistence enabled
  - Caching configured
  - Error handling settings

### `llmspell.toml`
- **Use Case**: Main LLMSpell configuration
- **Best For**: Default system configuration
- **Key Features**:
  - Complete system settings
  - All features enabled
  - Production defaults
  - Extensible structure

### State Management Configurations

### `state-enabled.toml`
- **Use Case**: State persistence examples
- **Best For**: Scripts requiring state between runs
- **Key Features**:
  - State global enabled
  - Memory backend
  - Full Lua stdlib
  - No providers (state-only)

### `session-enabled.toml`
- **Use Case**: Session management
- **Best For**: Conversational applications
- **Key Features**:
  - Session tracking enabled
  - Artifact storage
  - Session replay capability
  - Access control settings

### `migration-enabled.toml`
- **Use Case**: State migration testing
- **Best For**: Schema evolution examples
- **Key Features**:
  - Migration system enabled
  - Version tracking
  - Rollback support
  - Schema validation

### `backup-enabled.toml`
- **Use Case**: State backup and recovery
- **Best For**: Production state management
- **Key Features**:
  - Automatic backups
  - Configurable retention
  - Recovery procedures
  - Backup validation

### RAG Configurations

### `rag-basic.toml`
- **Use Case**: Getting started with RAG functionality
- **Best For**: Learning, prototyping, small projects
- **Key Features**:
  - Simple HNSW vector storage setup
  - Moderate resource limits
  - Basic caching enabled
  - 384-dimensional vectors (suitable for lightweight models)

### `rag-development.toml`
- **Use Case**: Local development and testing
- **Best For**: Rapid iteration, debugging, feature development
- **Key Features**:
  - Mock backend option for fast testing
  - Debug logging enabled
  - Minimal resource usage (512MB limit)
  - Small cache sizes for testing fresh results
  - Debug dump functionality

### `rag-production.toml`
- **Use Case**: Production deployments
- **Best For**: Live services, customer-facing applications
- **Key Features**:
  - Multi-tenant support enabled
  - Persistent storage with backups
  - Conservative resource limits (4GB)
  - Comprehensive monitoring and alerting
  - Security features (audit logging, rate limiting)
  - Automatic backup and recovery

### `rag-performance.toml`
- **Use Case**: Maximum performance scenarios
- **Best For**: High-throughput applications, large-scale deployments
- **Key Features**:
  - High-dimensional embeddings (1536d)
  - Aggressive caching (100K embeddings)
  - Large memory allocation (8GB)
  - Optimized HNSW parameters
  - Support for 50M vectors

### `rag-multi-tenant.toml`
- **Use Case**: Multi-tenant applications
- **Best For**: SaaS platforms, shared environments
- **Key Features**:
  - Complete tenant isolation
  - Per-tenant resource limits
  - Automatic tenant cleanup
  - Larger caches for efficiency
  - Strict isolation enforcement

## Configuration Options

### Core Settings

```toml
[rag]
enabled = true           # Enable/disable RAG functionality
multi_tenant = false     # Enable tenant isolation
```

### Vector Storage

```toml
[rag.vector_storage]
dimensions = 384         # Vector dimensions (model-dependent)
backend = "hnsw"        # Options: "hnsw", "mock"
persistence_path = "./data/rag"  # Storage location
max_memory_mb = 1024    # Memory limit

[rag.vector_storage.hnsw]
m = 16                  # Bi-directional links per node
ef_construction = 200   # Construction quality
ef_search = 50         # Search quality
max_elements = 1000000  # Max vectors
metric = "cosine"       # Distance metric
num_threads = 4         # Parallelism
```

### Embedding Configuration

```toml
[rag.embedding]
default_provider = "openai"  # Embedding provider
cache_enabled = true         # Cache embeddings
cache_size = 10000          # Cache entries
cache_ttl_seconds = 3600    # Cache lifetime
batch_size = 32             # Processing batch size
timeout_seconds = 30        # Request timeout
max_retries = 3             # Retry attempts
```

### Document Processing

```toml
[rag.chunking]
strategy = "sliding_window"  # Chunking strategy
chunk_size = 512            # Tokens per chunk
overlap = 64                # Overlap between chunks
max_chunk_size = 2048       # Maximum chunk size
min_chunk_size = 100        # Minimum chunk size
```

### Caching

```toml
[rag.cache]
search_cache_enabled = true      # Cache search results
search_cache_size = 1000         # Cached queries
search_cache_ttl_seconds = 300   # Cache lifetime
document_cache_enabled = true     # Cache documents
document_cache_size_mb = 100     # Document cache size
```

## 🚀 Quick Start Guide

### Choose Your Configuration

```bash
# For simple tool scripts (no LLM needed)
./target/debug/llmspell -c examples/script-users/configs/minimal.toml run script.lua

# For agent/LLM examples
./target/debug/llmspell -c examples/script-users/configs/example-providers.toml run script.lua

# For state persistence
./target/debug/llmspell -c examples/script-users/configs/state-enabled.toml run script.lua

# For RAG examples
./target/debug/llmspell -c examples/script-users/configs/rag-basic.toml run script.lua

# For production applications
./target/debug/llmspell -c examples/script-users/configs/applications.toml run app/main.lua
```

## Usage

### Using a Configuration File

```bash
# Set via environment variable
export LLMSPELL_CONFIG=examples/script-users/configs/rag-production.toml
llmspell run my-script.lua

# Or pass directly
llmspell --config examples/script-users/configs/rag-development.toml run my-script.lua
```

### Overriding Settings via CLI

```bash
# Enable RAG explicitly
llmspell run --rag my-script.lua

# Use custom configuration with overrides
llmspell run --rag-config custom.toml --rag-dims 768 --rag-backend hnsw my-script.lua
```

### Programmatic Configuration

```lua
-- Check if RAG is available
if RAG then
    -- Ingest documents
    local doc_id = RAG.ingest({
        content = "Your document content here",
        metadata = { source = "example.txt" }
    })
    
    -- Search for similar content
    local results = RAG.search({
        query = "search query",
        top_k = 5
    })
    
    for _, result in ipairs(results) do
        print("Score:", result.score, "Content:", result.content)
    end
end
```

## 📋 Configuration Selection Guide

### By Use Case

| Use Case | Recommended Config | Key Features |
|----------|-------------------|--------------|
| **Learning LLMSpell** | `minimal.toml` | Tools only, no setup needed |
| **First agent** | `example-providers.toml` | OpenAI + Anthropic ready |
| **State examples** | `state-enabled.toml` | State API with memory backend |
| **RAG prototyping** | `rag-development.toml` | Mock backend for fast iteration |
| **RAG production** | `rag-production.toml` | Full RAG with persistence |
| **SaaS platform** | `rag-multi-tenant.toml` | Tenant isolation, quotas |
| **Complete apps** | `applications.toml` | All features configured |
| **Best practices** | `cookbook.toml` | Production patterns |

### By Example Type

| Example Directory | Use This Config | Why |
|------------------|-----------------|-----|
| `getting-started/00-04` | `minimal.toml` | No providers needed |
| `getting-started/05` | `rag-basic.toml` | RAG introduction |
| `features/` | `example-providers.toml` | Provider features |
| `cookbook/` | `cookbook.toml` | Production patterns |
| `applications/` | Each has own config | App-specific needs |
| `advanced-patterns/` | `example-providers.toml` | Multi-agent patterns |

## Validation

### RAG Configurations

Run the validation script to test RAG configurations:

```bash
./examples/script-users/configs/validate-rag-configs.sh
```

This will verify that each configuration:
1. Parses correctly
2. Initializes RAG successfully
3. Provides the expected API surface

## 🔑 Environment Variables

### Required for Providers
- `OPENAI_API_KEY`: OpenAI API access (GPT models, embeddings)
- `ANTHROPIC_API_KEY`: Anthropic API access (Claude models)

### System Configuration
- `LLMSPELL_CONFIG`: Path to configuration file
- `RUST_LOG`: Logging level (`trace`, `debug`, `info`, `warn`, `error`)
- `LLMSPELL_HOME`: LLMSpell home directory (defaults to `~/.llmspell`)

### Optional Overrides
- `LLMSPELL_STATE_BACKEND`: Override state backend (`memory`, `file`, `sqlite`)
- `LLMSPELL_RAG_ENABLED`: Enable/disable RAG (`true`, `false`)
- `LLMSPELL_MAX_MEMORY_MB`: Override memory limits

## Performance Tuning

### Memory vs Speed Trade-offs

- **Higher `m` value**: Better recall, more memory
- **Higher `ef_construction`**: Better index quality, slower builds
- **Higher `ef_search`**: Better search accuracy, slower queries
- **Larger caches**: Faster repeated operations, more memory

### Recommended Settings by Scale

| Scale | Vectors | Memory | m | ef_construction | ef_search |
|-------|---------|--------|---|-----------------|-----------|
| Small | <10K | 512MB | 8 | 50 | 25 |
| Medium | <100K | 2GB | 16 | 200 | 50 |
| Large | <1M | 4GB | 32 | 400 | 100 |
| XLarge | <10M | 8GB+ | 48 | 500 | 200 |

## 🔧 Troubleshooting

### Common Issues and Solutions

#### Configuration Not Loading
```bash
# Verify config is being used
RUST_LOG=debug ./target/debug/llmspell -c config.toml run script.lua
# Look for "Loading configuration from" in output
```

#### Provider Issues
1. **"No providers available"**
   - Check API keys are set: `echo $OPENAI_API_KEY`
   - Verify provider section in config
   - Use `example-providers.toml` as reference

2. **"Rate limit exceeded"**
   - Add rate limiting to config
   - Use `cookbook.toml` for retry patterns
   - Implement exponential backoff

#### State Issues
1. **"State not available"**
   - Use `state-enabled.toml` or similar
   - Ensure `[runtime.state_persistence]` section exists
   - Set `enabled = true`

2. **"State too large"**
   - Increase `max_state_size_bytes`
   - Use file or sqlite backend instead of memory
   - Implement state cleanup patterns

#### RAG Issues
1. **"RAG not available"**
   - Ensure `rag.enabled = true` in config
   - Check embedding provider is configured
   - Verify API key for embeddings

2. **Out of memory with RAG**
   - Reduce `max_memory_mb` in vector storage
   - Lower cache sizes
   - Use smaller vector dimensions (384 vs 1536)

3. **Slow RAG search**
   - Increase `ef_search` for parallelism
   - Enable search caching
   - Use `rag-performance.toml` settings

4. **Multi-tenant not working**
   - Set `multi_tenant = true`
   - Pass tenant ID in API calls
   - Enable `strict_isolation = true`

## 📈 Configuration Migration

When moving from development to production, follow the migration path:

```
minimal.toml → basic.toml → example-providers.toml → applications.toml
                     ↓
              state-enabled.toml → session-enabled.toml
                     ↓
              rag-development.toml → rag-basic.toml → rag-production.toml
                                            ↓
                                    rag-multi-tenant.toml (for SaaS)
```

See [MIGRATION.md](MIGRATION.md) for detailed migration steps between configurations.

## 🔗 Related Documentation

### In This Directory
- [MIGRATION.md](MIGRATION.md) - Detailed guide for migrating between configurations
- [validate-rag-configs.sh](validate-rag-configs.sh) - Script to validate RAG configurations

### Project Documentation
- [Lua API Reference](../../../docs/user-guide/api/lua/README.md) - Complete Lua API
- [Tool Catalog](../../../docs/user-guide/tools-catalog.md) - All available tools
- [Architecture Guide](../../../docs/technical/master-architecture-vision.md) - System design

### Example Usage
- [Getting Started](../getting-started/) - Basic examples using these configs
- [Features](../features/) - Feature demonstrations with configs
- [Cookbook](../cookbook/) - Production patterns using these configs
- [Applications](../applications/) - Complete apps with custom configs

## 🆕 Phase 8 Enhancements

This release adds comprehensive RAG support with:
- 5 specialized RAG configurations
- Multi-tenant isolation support
- Session-based RAG for conversations
- Cost optimization configurations
- Performance tuning for millions of vectors
- HNSW vector storage with persistence
- Embedding caching and batching