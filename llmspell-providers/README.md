# llmspell-providers

LLM provider integrations for Rs-LLMSpell.

## Features
- Multi-provider support via rig (OpenAI, Anthropic, Google, Cohere, Perplexity)
- Local model support through candle integration
- Unified provider interface with automatic retry and fallback

## Usage
```rust
use llmspell_providers::{ProviderManager, OpenAIProvider};

let manager = ProviderManager::new()
    .add_provider("openai", OpenAIProvider::new(api_key))
    .with_fallback_chain(vec!["openai", "anthropic"]);
let response = manager.complete(prompt).await?;
```

## Dependencies
- `llmspell-core` - Provider trait definitions
- `llmspell-config` - API key and model configuration
- External: `rig`, `candle`