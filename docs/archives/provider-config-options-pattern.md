# Provider Configuration Options Extraction Pattern

**Status**: Phase 11 Implementation Pattern
**Purpose**: Document how to extract backend-specific options from `ProviderConfig.options` HashMap
**Related**: `llmspell-config/src/providers.rs:137-138`

## Overview

The `ProviderConfig` struct uses a flattened `HashMap<String, serde_json::Value>` for backend-specific options, allowing new provider types (like Ollama and Candle) to be added without struct changes.

## ProviderConfig Structure

```rust
// llmspell-config/src/providers.rs:104-139
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(default)]
pub struct ProviderConfig {
    // Standard fields
    pub name: String,
    pub provider_type: String,
    pub enabled: bool,
    pub base_url: Option<String>,
    pub api_key: Option<String>,
    pub default_model: Option<String>,
    pub max_tokens: Option<u32>,
    pub timeout_seconds: Option<u64>,

    // Backend-specific options (FLATTENED into config)
    #[serde(flatten)]
    pub options: HashMap<String, serde_json::Value>,
}
```

**Key Point**: The `#[serde(flatten)]` attribute means all extra TOML fields are automatically captured in the `options` HashMap without explicit field definitions.

## TOML Configuration Format

### Ollama Example

```toml
[providers.ollama]
provider_type = "ollama"
enabled = true
base_url = "http://localhost:11434"
timeout_seconds = 120

# Backend-specific fields (auto-captured in options)
auto_start = true
health_check_interval_seconds = 60
default_backend = "ollama"
```

### Candle Example

```toml
[providers.candle]
provider_type = "candle"
enabled = true
timeout_seconds = 300

# Backend-specific fields (auto-captured in options)
model_directory = "${HOME}/.llmspell/models/candle"
device = "auto"
max_concurrent = 1
default_quantization = "Q4_K_M"
cpu_threads = 0
context_size = 4096
batch_size = 512
use_flash_attention = true
```

## Extraction Pattern

### Basic Pattern

```rust
impl OllamaProvider {
    pub fn new(config: ProviderConfig) -> Result<Self> {
        // Extract backend-specific options with defaults
        let auto_start = config.options.get("auto_start")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        let health_check_interval = config.options.get("health_check_interval_seconds")
            .and_then(|v| v.as_u64())
            .unwrap_or(60);

        let default_backend = config.options.get("default_backend")
            .and_then(|v| v.as_str())
            .unwrap_or("ollama");

        // Use extracted values
        // ...
    }
}
```

### Advanced Pattern with Type Safety

```rust
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct CandleOptions {
    #[serde(default = "default_model_directory")]
    model_directory: String,
    #[serde(default = "default_device")]
    device: String,
    #[serde(default = "default_max_concurrent")]
    max_concurrent: usize,
    #[serde(default = "default_quantization")]
    default_quantization: String,
    #[serde(default)]
    cpu_threads: usize,
    #[serde(default = "default_context_size")]
    context_size: usize,
    #[serde(default = "default_batch_size")]
    batch_size: usize,
    #[serde(default = "default_flash_attention")]
    use_flash_attention: bool,
}

fn default_model_directory() -> String {
    dirs::home_dir()
        .map(|p| p.join(".llmspell/models/candle").to_string_lossy().to_string())
        .unwrap_or_else(|| "./models/candle".to_string())
}

fn default_device() -> String { "auto".to_string() }
fn default_max_concurrent() -> usize { 1 }
fn default_quantization() -> String { "Q4_K_M".to_string() }
fn default_context_size() -> usize { 4096 }
fn default_batch_size() -> usize { 512 }
fn default_flash_attention() -> bool { true }

impl CandleProvider {
    pub fn new(config: ProviderConfig) -> Result<Self> {
        // Convert HashMap to structured options using serde
        let options_value = serde_json::to_value(&config.options)
            .map_err(|e| anyhow!("Failed to serialize options: {}", e))?;

        let options: CandleOptions = serde_json::from_value(options_value)
            .map_err(|e| anyhow!("Failed to deserialize Candle options: {}", e))?;

        // Use type-safe, validated options
        let model_directory = PathBuf::from(options.model_directory);
        let device = Device::from_str(&options.device)?;

        // ...
    }
}
```

## Value Type Extraction Helpers

```rust
/// Helper trait for safe option extraction
trait ConfigOptionExt {
    fn get_string(&self, key: &str) -> Option<String>;
    fn get_bool(&self, key: &str) -> Option<bool>;
    fn get_u64(&self, key: &str) -> Option<u64>;
    fn get_f64(&self, key: &str) -> Option<f64>;
}

impl ConfigOptionExt for HashMap<String, serde_json::Value> {
    fn get_string(&self, key: &str) -> Option<String> {
        self.get(key).and_then(|v| v.as_str()).map(String::from)
    }

    fn get_bool(&self, key: &str) -> Option<bool> {
        self.get(key).and_then(|v| v.as_bool())
    }

    fn get_u64(&self, key: &str) -> Option<u64> {
        self.get(key).and_then(|v| v.as_u64())
    }

    fn get_f64(&self, key: &str) -> Option<f64> {
        self.get(key).and_then(|v| v.as_f64())
    }
}

// Usage
impl OllamaProvider {
    pub fn new(config: ProviderConfig) -> Result<Self> {
        let auto_start = config.options.get_bool("auto_start").unwrap_or(false);
        let health_interval = config.options.get_u64("health_check_interval_seconds").unwrap_or(60);
        let backend = config.options.get_string("default_backend").unwrap_or_else(|| "ollama".to_string());

        // ...
    }
}
```

## Environment Variable Expansion

```rust
/// Expand environment variables in string values
fn expand_env_vars(value: &str) -> String {
    if value.starts_with("${") && value.ends_with("}") {
        let var_name = &value[2..value.len()-1];
        std::env::var(var_name).unwrap_or_else(|_| value.to_string())
    } else {
        value.to_string()
    }
}

// Usage
let model_directory = config.options.get_string("model_directory")
    .map(|s| expand_env_vars(&s))
    .unwrap_or_else(default_model_directory);
```

## Validation Pattern

```rust
impl CandleProvider {
    pub fn new(config: ProviderConfig) -> Result<Self> {
        // Extract options
        let options: CandleOptions = /* ... */;

        // Validate before use
        Self::validate_options(&options)?;

        // Use validated options
        // ...
    }

    fn validate_options(options: &CandleOptions) -> Result<()> {
        // Validate device
        if !["auto", "cpu", "cuda", "metal"].contains(&options.device.as_str()) {
            return Err(anyhow!("Invalid device: {}", options.device));
        }

        // Validate model directory exists or can be created
        let model_dir = PathBuf::from(&options.model_directory);
        if !model_dir.exists() {
            std::fs::create_dir_all(&model_dir)
                .map_err(|e| anyhow!("Cannot create model directory: {}", e))?;
        }

        // Validate quantization level
        if !["Q4_K_M", "Q5_K_M", "Q8_0"].contains(&options.default_quantization.as_str()) {
            return Err(anyhow!("Invalid quantization: {}", options.default_quantization));
        }

        Ok(())
    }
}
```

## Testing

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_candle_options_extraction() {
        let mut config = ProviderConfig::default();
        config.provider_type = "candle".to_string();

        // Add backend-specific options
        config.options.insert("device".to_string(), json!("cuda"));
        config.options.insert("max_concurrent".to_string(), json!(2));
        config.options.insert("context_size".to_string(), json!(8192));

        let provider = CandleProvider::new(config).unwrap();

        // Verify extracted values
        assert_eq!(provider.device, Device::Cuda);
        assert_eq!(provider.max_concurrent, 2);
        assert_eq!(provider.context_size, 8192);
    }

    #[test]
    fn test_options_with_defaults() {
        let config = ProviderConfig {
            provider_type: "candle".to_string(),
            ..Default::default()
        };

        let provider = CandleProvider::new(config).unwrap();

        // Verify defaults applied
        assert_eq!(provider.device, Device::Auto);
        assert_eq!(provider.max_concurrent, 1);
        assert_eq!(provider.default_quantization, "Q4_K_M");
    }
}
```

## Summary

**No Struct Changes Required**: The `#[serde(flatten)]` attribute on `ProviderConfig.options` allows unlimited backend-specific fields without modifying the core struct.

**Two Extraction Approaches**:
1. **Manual**: Use `HashMap::get()` with type conversions (simple, direct)
2. **Structured**: Define a typed options struct and use serde deserialization (type-safe, validated)

**Best Practices**:
- Always provide sensible defaults
- Validate extracted values before use
- Use environment variable expansion for paths
- Create helper traits for common extraction patterns
- Write tests for both extraction and defaults

This pattern allows Phase 11 to add Ollama and Candle providers without any changes to `llmspell-config`.
