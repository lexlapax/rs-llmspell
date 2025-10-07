# llmspell-providers

**LLM provider integrations**

**🔗 Navigation**: [← Rust API](README.md) | [Crate Docs](https://docs.rs/llmspell-providers) | [Source](../../../../llmspell-providers)

---

## Overview

`llmspell-providers` implements integrations with various LLM providers including OpenAI, Anthropic, Ollama, Groq, and more. It provides a unified interface for different providers while preserving provider-specific features.

**Key Features:**
- 🔌 Multiple provider support (OpenAI, Anthropic, Groq)
- 🏠 **Local LLM support (Ollama, Candle)** ⭐ **Phase 11**
- 📦 **Model management** (pull, list, inspect) ⭐ **Phase 11**
- 🏥 **Health checks** for local backends ⭐ **Phase 11**
- 🔄 Streaming responses
- 💰 Token usage tracking
- ⚡ Rate limiting and retries
- 🔧 Function calling support
- 📊 Provider metrics
- 🎯 Model discovery
- 🔐 Secure API key management

## Provider Trait

```rust
#[async_trait]
pub trait Provider: Send + Sync {
    /// Get provider name
    fn name(&self) -> &str;
    
    /// List available models
    async fn list_models(&self) -> Result<Vec<Model>>;
    
    /// Create completion
    async fn complete(&self, request: CompletionRequest) -> Result<CompletionResponse>;
    
    /// Stream completion
    async fn complete_stream(&self, request: CompletionRequest) -> Result<CompletionStream>;
    
    /// Create embeddings
    async fn embed(&self, request: EmbeddingRequest) -> Result<EmbeddingResponse>;
}
```

## Supported Providers

### OpenAI

```rust
use llmspell_providers::openai::OpenAIProvider;

let provider = OpenAIProvider::new(OpenAIConfig {
    api_key: env::var("OPENAI_API_KEY")?,
    organization: None,
    base_url: None,
})?;

let response = provider.complete(CompletionRequest {
    model: "gpt-4".to_string(),
    messages: vec![Message::user("Hello!")],
    temperature: Some(0.7),
    max_tokens: Some(1000),
    ..Default::default()
}).await?;
```

### Anthropic

```rust
use llmspell_providers::anthropic::AnthropicProvider;

let provider = AnthropicProvider::new(AnthropicConfig {
    api_key: env::var("ANTHROPIC_API_KEY")?,
    base_url: None,
})?;

let response = provider.complete(CompletionRequest {
    model: "claude-3-opus".to_string(),
    messages: vec![Message::user("Explain quantum computing")],
    ..Default::default()
}).await?;
```

### Ollama (Local Backend) ⭐ **Phase 11**

```rust
use llmspell_providers::ollama::OllamaProvider;
use llmspell_providers::local::LocalProviderInstance;

let provider = OllamaProvider::new(OllamaConfig {
    base_url: "http://localhost:11434".to_string(),
})?;

// Health check
let status = provider.health_check().await?;
match status {
    HealthStatus::Healthy { available_models, version } => {
        println!("Ollama healthy: {} models, version {:?}", available_models, version);
    }
    HealthStatus::Unhealthy { reason } => {
        println!("Ollama unhealthy: {}", reason);
    }
    _ => {}
}

// List local models
let models = provider.list_local_models().await?;
for model in models {
    println!("Model: {} ({})", model.id, model.backend);
    println!("  Size: {} bytes", model.size_bytes);
    if let Some(quant) = model.quantization {
        println!("  Quantization: {}", quant);
    }
}

// Pull a model
use llmspell_providers::local::ModelSpec;
let spec = ModelSpec::parse("llama3.1:8b@ollama")?;
let progress = provider.pull_model(&spec).await?;
println!("Downloaded: {}%", progress.percent_complete);

// Get model info
let info = provider.model_info("llama3.1:8b").await?;
println!("Format: {}, Loaded: {}", info.format, info.loaded);
```

### Candle (Embedded Inference) ⭐ **Phase 11**

```rust
use llmspell_providers::candle::CandleProvider;
use llmspell_providers::local::LocalProviderInstance;

let provider = CandleProvider::new(CandleConfig {
    device: "cpu".to_string(), // or "cuda", "metal"
    cache_dir: Some("/path/to/models".into()),
})?;

// Pull and cache model
let spec = ModelSpec::parse("tinyllama@candle")?;
let progress = provider.pull_model(&spec).await?;

// Run inference
let response = provider.complete(CompletionRequest {
    model: "tinyllama".to_string(),
    messages: vec![Message::user("Hello!")],
    max_tokens: Some(100),
    ..Default::default()
}).await?;
```

## LocalProviderInstance Trait ⭐ **Phase 11**

```rust
#[async_trait]
pub trait LocalProviderInstance: ProviderInstance {
    /// Check backend health
    async fn health_check(&self) -> Result<HealthStatus>;

    /// List locally available models
    async fn list_local_models(&self) -> Result<Vec<LocalModelInfo>>;

    /// Pull/download a model
    async fn pull_model(&self, spec: &ModelSpec) -> Result<DownloadProgress>;

    /// Get detailed model information
    async fn model_info(&self, model_id: &str) -> Result<ModelInfo>;
}

// Health status
pub enum HealthStatus {
    Healthy {
        available_models: usize,
        version: Option<String>,
    },
    Unhealthy {
        reason: String,
    },
    Unknown,
}

// Download progress
pub struct DownloadProgress {
    pub model_id: String,
    pub status: DownloadStatus,
    pub percent_complete: f32,
    pub bytes_downloaded: u64,
    pub bytes_total: Option<u64>,
}

pub enum DownloadStatus {
    Starting,
    Downloading,
    Verifying,
    Complete,
    Failed { error: String },
}
```

## Model Specification ⭐ **Phase 11**

Models use unified format: `model_name[:tag][@backend]`

```rust
use llmspell_providers::local::ModelSpec;

// Parse model specifications
let spec1 = ModelSpec::parse("llama3.1:8b@ollama")?;
assert_eq!(spec1.model_name, "llama3.1");
assert_eq!(spec1.tag, Some("8b".to_string()));
assert_eq!(spec1.backend, Some("ollama".to_string()));

let spec2 = ModelSpec::parse("mistral:7b@candle")?;
let spec3 = ModelSpec::parse("tinyllama@candle")?; // Default tag

// Use with provider
let progress = provider.pull_model(&spec1).await?;
```

## Streaming Responses

```rust
let mut stream = provider.complete_stream(request).await?;

while let Some(chunk) = stream.next().await {
    match chunk {
        Ok(delta) => print!("{}", delta.content),
        Err(e) => eprintln!("Stream error: {}", e),
    }
}
```

## Related Documentation

- [llmspell-agents](llmspell-agents.md) - Agent framework using providers
- [llmspell-config](llmspell-config.md) - Provider configuration