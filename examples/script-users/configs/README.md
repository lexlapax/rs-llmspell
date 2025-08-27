# RAG Configuration Guide

This directory contains pre-configured RAG (Retrieval-Augmented Generation) configuration templates for various use cases in llmspell.

## Available Configurations

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

## Validation

Run the validation script to test configurations:

```bash
./validate-rag-configs.sh
```

This will verify that each configuration:
1. Parses correctly
2. Initializes RAG successfully
3. Provides the expected API surface

## Environment Variables

The following environment variables affect RAG behavior:

- `OPENAI_API_KEY`: Required for OpenAI embeddings
- `LLMSPELL_CONFIG`: Path to configuration file
- `RUST_LOG`: Logging level (e.g., `debug`, `info`)

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

## Troubleshooting

### Common Issues

1. **"RAG not available" in scripts**
   - Ensure `rag.enabled = true` in config
   - Check that config file is being loaded

2. **Out of memory errors**
   - Reduce `max_memory_mb`
   - Lower cache sizes
   - Use smaller vector dimensions

3. **Slow search performance**
   - Increase `ef_search` for better parallelism
   - Enable caching
   - Consider using performance config

4. **Multi-tenant isolation issues**
   - Ensure `multi_tenant = true`
   - Check tenant ID is being passed correctly
   - Verify `strict_isolation = true`

## See Also

- [Migration Guide](MIGRATION.md) - Moving from development to production
- [API Documentation](../../../docs/api/rag.md) - Full RAG API reference
- [Performance Benchmarks](../../../benchmarks/rag/README.md) - Detailed performance analysis