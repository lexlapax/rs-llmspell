# llmspell-config

Configuration management for Rs-LLMSpell framework.

## Features
- Layered configuration (defaults, file, environment, runtime)
- Type-safe configuration with serde
- Hot-reloading support for dynamic updates

## Usage
```rust
use llmspell_config::{Config, ConfigBuilder};

let config = ConfigBuilder::new()
    .add_source(File::from("llmspell.toml"))
    .add_source(Environment::with_prefix("LLMSPELL"))
    .build()?;
let api_key: String = config.get("providers.openai.api_key")?;
```

## Dependencies
- `llmspell-core` - Configuration trait definitions
- External: `config`, `serde`, `toml`