# llmspell-config

Comprehensive configuration management for Rs-LLMSpell framework with specialized RAG, multi-tenant, and performance tuning support.

## Features

### Core Configuration Management
- **Layered Configuration**: Hierarchical config (defaults → file → environment → runtime)
- **Type-Safe Validation**: Compile-time configuration validation with serde
- **Hot-Reloading**: Dynamic configuration updates without service restart
- **Environment Integration**: Seamless development, staging, production configurations

### RAG Configuration (Phase 8)
- **HNSW Tuning**: Advanced vector search optimization with preset profiles
- **Embedding Configuration**: Multi-provider embedding settings with fallback chains
- **Pipeline Configuration**: End-to-end RAG pipeline tuning with performance profiles
- **Multi-Tenant Settings**: Tenant-specific configuration with inheritance and overrides

## Usage

### Basic Configuration
```rust
use llmspell_config::{Config, ConfigBuilder};

let config = ConfigBuilder::new()
    .add_source(File::from("llmspell.toml"))
    .add_source(Environment::with_prefix("LLMSPELL"))
    .build()?;

let api_key: String = config.get("providers.openai.api_key")?;
```

### RAG Configuration (Phase 8)
```rust
use llmspell_config::{RAGConfig, HNSWConfig, EmbeddingConfig};

// Load RAG-specific configuration
let rag_config: RAGConfig = config.get("rag")?;

// Access HNSW performance tuning
let hnsw_config = &rag_config.vector_storage.hnsw;
println!("HNSW config: ef_construction={}, max_connections={}", 
    hnsw_config.ef_construction, hnsw_config.max_connections);

// Access embedding provider configuration
let embedding_config = &rag_config.embeddings;
match &embedding_config.primary_provider {
    EmbeddingProvider::OpenAI { model, api_key } => {
        println!("Using OpenAI model: {}", model);
    },
    EmbeddingProvider::Local { model_path, dimensions } => {
        println!("Using local model: {} ({}D)", model_path, dimensions);
    },
}
```

### Production RAG Configuration File
```toml
# llmspell.toml
[rag]
# Document processing configuration
[rag.chunking]
strategy = "semantic"
chunk_size = 512
overlap = 50
min_chunk_size = 100

# Embedding provider configuration
[rag.embeddings]
primary_provider = { type = "openai", model = "text-embedding-ada-002" }
fallback_providers = [
    { type = "local", model_path = "./models/all-MiniLM-L6-v2", dimensions = 384 }
]
cache_enabled = true
cache_size_mb = 256

# Vector storage configuration with HNSW tuning
[rag.vector_storage]
backend = "hnsw"

[rag.vector_storage.hnsw]
ef_construction = 200        # Build-time accuracy/speed tradeoff
max_connections = 16         # Memory vs accuracy tradeoff
distance_metric = "cosine"
enable_pruning = true
parallel_batch_size = 1000
enable_mmap = true
mmap_sync_interval = 60

# Retrieval configuration
[rag.retrieval]
default_top_k = 10
similarity_threshold = 0.8
enable_reranking = true
context_window_size = 8192
```

### Multi-Tenant Configuration
```rust
use llmspell_config::{TenantConfig, ConfigManager};

// Load tenant-specific configuration with inheritance
let config_manager = ConfigManager::new(base_config).await?;

// Register tenant configuration template
config_manager.register_template("enterprise", TenantConfig {
    rag: RAGConfig {
        chunking: ChunkingConfig {
            strategy: ChunkingStrategy::Semantic,
            chunk_size: 1024,  // Larger chunks for enterprise
            overlap: 100,
        },
        embeddings: EmbeddingConfig {
            primary_provider: EmbeddingProvider::OpenAI {
                model: "text-embedding-3-large".to_string(),
            },
            cache_size_mb: 512,  // Larger cache
        },
        vector_storage: VectorStorageConfig {
            hnsw: HNSWConfig::production(),  // High-performance preset
        },
        retrieval: RetrievalConfig {
            default_top_k: 20,  // More results for enterprise
            similarity_threshold: 0.85,
            enable_reranking: true,
        },
    },
}).await?;

// Apply tenant-specific overrides
let tenant_config = config_manager
    .get_tenant_config("company-123")
    .with_override("rag.retrieval.default_top_k", 15)
    .with_override("rag.vector_storage.hnsw.max_connections", 32)
    .build().await?;
```

### Environment-Specific Configuration
```rust
use llmspell_config::{Environment, PerformanceProfile};

// Development configuration
let dev_config = ConfigBuilder::new()
    .add_source(File::from("configs/rag-development.toml"))
    .set_performance_profile(PerformanceProfile::Development)
    .build()?;

// Production configuration with environment overrides
let prod_config = ConfigBuilder::new()
    .add_source(File::from("configs/rag-production.toml"))
    .add_source(Environment::with_prefix("LLMSPELL"))
    .set_performance_profile(PerformanceProfile::Production)
    .validate_required_fields()
    .build()?;

// Configuration validation
match prod_config.validate() {
    Ok(_) => println!("Configuration valid"),
    Err(errors) => {
        for error in errors {
            eprintln!("Config error: {}", error);
        }
        return Err("Invalid configuration".into());
    }
}
```

### Hot-Reloading RAG Configuration
```rust
use llmspell_config::{ConfigWatcher, ConfigEvent};

// Watch for configuration changes
let mut watcher = ConfigWatcher::new("llmspell.toml").await?;

while let Some(event) = watcher.next_event().await? {
    match event {
        ConfigEvent::RagConfigChanged { section, old_value, new_value } => {
            println!("RAG config changed: {} = {} -> {}", section, old_value, new_value);
            
            // Apply hot reload for safe changes
            match section.as_str() {
                "rag.retrieval.similarity_threshold" => {
                    rag_pipeline.update_similarity_threshold(new_value.parse()?).await?;
                },
                "rag.retrieval.default_top_k" => {
                    rag_pipeline.update_default_top_k(new_value.parse()?).await?;
                },
                _ => {
                    println!("Configuration change requires restart: {}", section);
                }
            }
        },
        ConfigEvent::ValidationError { errors } => {
            eprintln!("Configuration validation failed: {:?}", errors);
        },
    }
}
```

## Configuration Presets (Phase 8)

### Development Profile
```rust
// Optimized for fast iteration and debugging
HNSWConfig::development()  // Low accuracy, fast builds
ChunkingConfig { chunk_size: 256, strategy: FixedSize }
EmbeddingConfig { cache_enabled: false }  // No caching for dev
```

### Balanced Profile  
```rust  
// Good accuracy/performance tradeoff for staging
HNSWConfig::balanced()     // ef_construction=100, max_connections=16
ChunkingConfig { chunk_size: 512, strategy: Semantic }
EmbeddingConfig { cache_size_mb: 128 }
```

### Production Profile
```rust
// Optimized for accuracy and throughput
HNSWConfig::production()   // ef_construction=200, max_connections=32
ChunkingConfig { chunk_size: 1024, strategy: Semantic }
EmbeddingConfig { cache_size_mb: 512, enable_persistence: true }
```

## Phase 8 Architecture
```
llmspell-config
├── rag.rs              # RAG-specific configuration structures
├── engines.rs          # Script engine configuration  
├── providers.rs        # LLM provider configuration
├── tools.rs            # Tool configuration management
├── validation.rs       # Configuration validation rules
├── env.rs              # Environment variable handling
├── env_registry.rs     # Environment variable registry
└── debug.rs            # Debug configuration utilities
```

## Configuration Examples

Production configurations are available in `examples/script-users/configs/`:
- `rag-basic.toml` - Simple RAG setup for getting started
- `rag-development.toml` - Development environment with fast iteration
- `rag-multi-tenant.toml` - Multi-tenant production setup
- `rag-performance.toml` - High-performance configuration for large deployments
- `rag-production.toml` - Full production configuration with all features

## Dependencies
- `llmspell-core` - Configuration trait definitions and error types
- `llmspell-rag` - RAG configuration structures and validation
- `llmspell-tenancy` - Multi-tenant configuration management
- `config` - Configuration file loading and merging
- `serde` - Serialization and deserialization
- `toml` - TOML configuration file format
- `notify` - File system watching for hot-reload
- `tokio` - Async configuration loading and validation