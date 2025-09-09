# llmspell-config

**Configuration system with validation** **🆕 UPDATED Phase 9**

**🔗 Navigation**: [← Rust API](README.md) | [Crate Docs](https://docs.rs/llmspell-config) | [Source](../../../../llmspell-config)

---

## Overview

`llmspell-config` provides the configuration infrastructure including schema definition, validation, environment variable support, kernel configuration, and RAG profiles introduced in Phase 9.

**Key Features:**
- 📝 TOML/JSON/YAML configuration
- ✅ Schema validation
- 🔄 Environment variable overrides
- 🔐 Secure credential management
- ⚙️ Kernel configuration (NEW Phase 9)
- 🎯 RAG profiles (NEW Phase 9)
- 📊 Configuration merging
- 💾 Config persistence
- 🔍 Config discovery

## Configuration Schema (v0.9.0)

```rust
#[derive(Debug, Deserialize, Serialize)]
pub struct LLMSpellConfig {
    pub providers: ProvidersConfig,
    pub security: SecurityConfig,
    pub storage: StorageConfig,
    pub runtime: RuntimeConfig,       // NEW: Contains kernel config
    pub tenancy: Option<TenancyConfig>,
    pub rag: RAGConfig,               // Enhanced with profiles
    pub hooks: Option<HooksConfig>,
    pub events: Option<EventsConfig>,
    pub debug: DebugConfig,
    pub default_engine: String,       // Script engine (lua, javascript, python)
}

#[derive(Debug, Deserialize, Serialize)]
pub struct RuntimeConfig {
    pub kernel: KernelConfig,         // NEW Phase 9
    pub security: SecurityConfig,
    pub memory_limit: Option<usize>,
    pub timeout_ms: Option<u64>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct KernelConfig {
    pub enabled: bool,
    pub auth_enabled: bool,
    pub max_clients: usize,
    pub heartbeat_interval_ms: u64,
    pub execution_timeout_ms: u64,
    pub transport: TransportConfig,
    pub security: KernelSecurityConfig,
}
```

## Loading Configuration

```rust
use llmspell_config::{Config, ConfigLoader};

// Load from file
let config = ConfigLoader::from_file("config.toml")?;

// Load with environment overrides
let config = ConfigLoader::new()
    .file("config.toml")
    .env_prefix("LLMSPELL")
    .load()?;

// Validate configuration
config.validate()?;
```

## Kernel Configuration (NEW Phase 9)

```toml
[runtime.kernel]
enabled = true
auth_enabled = false
max_clients = 50
heartbeat_interval_ms = 5000
execution_timeout_ms = 30000

[runtime.kernel.transport]
type = "tcp"
ip = "127.0.0.1"
shell_port = 50501
iopub_port = 50502
stdin_port = 50503
control_port = 50504
hb_port = 50505

[runtime.kernel.security]
signature_scheme = "hmac-sha256"
key = ""  # Auto-generated if empty
```

## RAG Profiles (NEW Phase 9)

RAG profiles simplify configuration by bundling common settings:

```toml
[rag]
enabled = true
default_collection = "knowledge"
embedding_provider = "openai"
vector_dimensions = 1536

# Define RAG profiles
[rag.profiles.production]
name = "production"
enabled = true
embedding_provider = "openai"
vector_dimensions = 1536
max_results = 10
chunk_size = 500
chunk_overlap = 50
threshold = 0.7

[rag.profiles.development]
name = "development"
enabled = true
embedding_provider = "ollama"
vector_dimensions = 768
max_results = 5
chunk_size = 1000
chunk_overlap = 100
threshold = 0.5

[rag.profiles.testing]
name = "testing"
enabled = false  # Disable RAG for testing
```

### Using RAG Profiles

```rust
use llmspell_config::{LLMSpellConfig, RAGProfile};

// Load config with profile
let mut config = LLMSpellConfig::from_file("config.toml")?;

// Apply profile from CLI
if let Some(profile) = config.rag.profiles.get("production") {
    profile.apply_to_config(&mut config.rag);
}

// In CLI usage
// llmspell run script.lua --rag-profile production
```

## Example Configuration

```toml
# config.toml
default_engine = "lua"

[providers.openai]
api_key = "${OPENAI_API_KEY}"
organization = "org-123"
model_aliases = { "gpt4" = "gpt-4-turbo-preview" }

[providers.anthropic]
api_key = "${ANTHROPIC_API_KEY}"
default_model = "claude-3-opus"

[security]
enable_audit = true
audit_path = "./audit.log"
rate_limiting = { enabled = true, requests_per_minute = 60 }

[storage]
backend = "sqlite"
path = "./llmspell.db"
encryption = true

[tenancy]
enabled = true
isolation_level = "strict"

## Environment Variables

```bash
# Override configuration via environment
export LLMSPELL_PROVIDERS_OPENAI_API_KEY="sk-..."
export LLMSPELL_SECURITY_ENABLE_AUDIT="false"
export LLMSPELL_STORAGE_PATH="/var/lib/llmspell/data.db"
```

## Configuration Validation

```rust
impl Config {
    pub fn validate(&self) -> Result<()> {
        // Check required fields
        if self.providers.is_empty() {
            return Err(ConfigError::NoProviders);
        }
        
        // Validate API keys
        if let Some(openai) = &self.providers.openai {
            if openai.api_key.is_empty() {
                return Err(ConfigError::MissingApiKey("openai"));
            }
        }
        
        // Validate paths
        if let Some(path) = &self.storage.path {
            if !path.parent().map_or(false, |p| p.exists()) {
                return Err(ConfigError::InvalidPath(path));
            }
        }
        
        Ok(())
    }
}
```

## Dynamic Configuration

```rust
use llmspell_config::DynamicConfig;

let mut config = DynamicConfig::new();

// Update configuration at runtime
config.set("providers.openai.temperature", 0.7)?;
config.set("security.rate_limiting.enabled", false)?;

// Watch for changes
config.watch("config.toml", |event| {
    println!("Config changed: {:?}", event);
})?;
```

## Related Documentation

- [Configuration Guide](../../configuration.md) - Detailed configuration guide
- [llmspell-cli](llmspell-cli.md) - CLI configuration handling