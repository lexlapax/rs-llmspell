# llmspell-providers

**LLM provider integrations**

**🔗 Navigation**: [← Rust API](README.md) | [Crate Docs](https://docs.rs/llmspell-providers) | [Source](../../../../llmspell-providers)

---

## Overview

`llmspell-providers` implements integrations with various LLM providers including OpenAI, Anthropic, Ollama, Groq, and more. It provides a unified interface for different providers while preserving provider-specific features.

**Key Features:**
- 🔌 Multiple provider support (OpenAI, Anthropic, Ollama, Groq)
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

### Ollama (Local)

```rust
use llmspell_providers::ollama::OllamaProvider;

let provider = OllamaProvider::new(OllamaConfig {
    base_url: "http://localhost:11434".to_string(),
})?;

let models = provider.list_models().await?;
println!("Available local models: {:?}", models);
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