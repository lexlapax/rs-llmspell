# LLMSpell Configuration Guide

**Status**: 🚀 **Phase 8.10.6** - Complete configuration set with RAG, state persistence, providers, and applications

## 🎯 Primary Approach: Builtin Profiles

**Most users should use builtin profiles** (`-p profile-name`) instead of custom config files:

```bash
# Recommended: Use builtin profiles
llmspell -p providers run script.lua        # Agents with OpenAI/Anthropic
llmspell -p state run script.lua            # State persistence
llmspell -p rag-dev run script.lua          # RAG development
llmspell -p rag-prod run script.lua         # RAG production
```

See [Builtin Profiles](#builtin-profiles-recommended) below for complete list.

## 🔧 When to Use Custom Configs

This directory contains **custom configuration templates** for advanced scenarios:
- Multi-tenant RAG with unique isolation requirements
- Application-specific security policies
- Custom resource limits and performance tuning
- Migration and backup strategies
- Demonstrating configuration patterns for learning

**15 Custom Configuration Templates** available:

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

## 🎯 Builtin Profiles (Recommended)

**Use builtin profiles for zero-config quick start:**

| Profile | Purpose | When to Use |
|---------|---------|-------------|
| `minimal` | Tools only, no LLM | Testing, learning tools |
| `development` | Dev mode with debug | Development, debugging |
| `providers` | OpenAI + Anthropic | Agent examples, LLM scripts |
| `state` | State persistence | State management examples |
| `sessions` | Sessions + state + events | Conversational apps |
| `ollama` | Local Ollama LLM | Local LLM with Ollama |
| `candle` | Local Candle LLM | Local LLM with Candle |
| `rag-dev` | RAG development | Learning RAG, prototyping |
| `rag-prod` | RAG production | Production RAG deployment |
| `rag-perf` | RAG performance | High-performance RAG |

**Examples:**
```bash
# For agent examples (requires API key)
llmspell -p providers run script.lua

# For state persistence
llmspell -p state run script.lua

# For RAG examples
llmspell -p rag-dev run script.lua
llmspell -p rag-prod run script.lua

# For local LLM
llmspell -p ollama run script.lua
```

## 🔧 Custom Configurations (Advanced)

### When to Use Custom Config Files

Use custom configs when you need:
- Unique multi-tenant isolation policies
- Custom resource limits beyond profile defaults
- Application-specific security settings
- Advanced migration or backup strategies

### Choose Your Configuration

```bash
# Multi-tenant RAG with custom isolation
./target/debug/llmspell -c examples/script-users/configs/rag-multi-tenant.toml run script.lua

# Custom application configuration
./target/debug/llmspell -c examples/script-users/configs/applications.toml run app/main.lua

# Migration testing
./target/debug/llmspell -c examples/script-users/configs/migration-enabled.toml run script.lua
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

### By Use Case (Builtin Profiles First)

| Use Case | Recommended Approach | Alternative (Custom Config) |
|----------|---------------------|---------------------------|
| **Learning LLMSpell** | `-p minimal` | `minimal.toml` |
| **First agent** | `-p providers` | `example-providers.toml` |
| **State examples** | `-p state` | `state-enabled.toml` |
| **RAG prototyping** | `-p rag-dev` | `rag-development.toml` |
| **RAG production** | `-p rag-prod` | `rag-production.toml` |
| **SaaS platform** | `-p rag-prod` + custom | `rag-multi-tenant.toml` |
| **Complete apps** | `-p development` | `applications.toml` |
| **Local LLM** | `-p ollama` or `-p candle` | Custom provider config |

**Note**: Use builtin profiles unless you need custom resource limits, multi-tenant isolation, or unique security policies.

### By Example Type

| Example Directory | Use Builtin Profile | Custom Config (if needed) |
|------------------|---------------------|--------------------------|
| `getting-started/` | `-p minimal`, `-p providers`, `-p rag-dev` | See individual examples |
| `features/` | `-p providers`, `-p state` | N/A |
| `cookbook/` | `-p providers`, `-p sessions`, `-p rag-prod` | `cookbook.toml` for patterns |
| `applications/` | `-p development`, `-p rag-prod` | Each app has own config.toml |
| `advanced-patterns/` | `-p providers`, `-p development` | N/A |

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
   - Use builtin profile: `llmspell -p providers run script.lua`
   - Check API keys are set: `echo $OPENAI_API_KEY`
   - Alternative: Use `example-providers.toml` as reference for custom config

2. **"Rate limit exceeded"**
   - Builtin profiles include sensible rate limits
   - For custom limits: Use `cookbook.toml` as reference
   - Implement exponential backoff in your script

#### State Issues
1. **"State not available"**
   - Use builtin profile: `llmspell -p state run script.lua`
   - Alternative: Use `state-enabled.toml` custom config
   - Ensure `[runtime.state_persistence]` section exists with `enabled = true`

2. **"State too large"**
   - Increase `max_state_size_bytes` in custom config
   - Use file or sqlite backend instead of memory
   - Implement state cleanup patterns

#### RAG Issues
1. **"RAG not available"**
   - Use builtin profile: `llmspell -p rag-dev run script.lua` (development)
   - Or: `llmspell -p rag-prod run script.lua` (production)
   - Verify API key for embeddings: `echo $OPENAI_API_KEY`
   - Alternative: Ensure `rag.enabled = true` in custom config

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