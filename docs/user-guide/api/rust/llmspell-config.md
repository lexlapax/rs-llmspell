# llmspell-config

**Configuration system with validation**

**🔗 Navigation**: [← Rust API](README.md) | [Crate Docs](https://docs.rs/llmspell-config) | [Source](../../../../llmspell-config)

---

## Overview

`llmspell-config` provides the configuration infrastructure including schema definition, validation, environment variable support, and provider configuration management.

**Key Features:**
- 📝 TOML/JSON/YAML configuration
- ✅ Schema validation
- 🔄 Environment variable overrides
- 🔐 Secure credential management
- 📊 Configuration merging
- 🎯 Provider configurations
- 💾 Config persistence
- 🔍 Config discovery

## Configuration Schema

```rust
#[derive(Debug, Deserialize, Serialize)]
pub struct Config {
    pub providers: ProvidersConfig,
    pub security: SecurityConfig,
    pub storage: StorageConfig,
    pub tenancy: Option<TenancyConfig>,
    pub rag: Option<RAGConfig>,
    pub hooks: Option<HooksConfig>,
    pub events: Option<EventsConfig>,
    pub debug: DebugConfig,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ProvidersConfig {
    pub openai: Option<OpenAIConfig>,
    pub anthropic: Option<AnthropicConfig>,
    pub ollama: Option<OllamaConfig>,
    pub groq: Option<GroqConfig>,
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

## Example Configuration

```toml
# config.toml

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

[rag]
enabled = true
default_collection = "knowledge"
embedding_provider = "openai"
vector_dimensions = 1536
```

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