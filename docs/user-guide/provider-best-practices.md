# Provider Usage Best Practices

**Version**: 0.13.0 (Phase 13 - Provider System Integration)
**Status**: Production Ready
**Last Updated**: October 2025

## Overview

LLMSpell supports a **dual-path provider architecture** that allows you to choose between:

1. **`provider_name`** (RECOMMENDED): Reference centrally-defined provider configuration
2. **`model`** (AD-HOC): Specify model directly for quick experiments

This guide explains when to use each approach and how to migrate between them.

## Quick Comparison

| Aspect | `provider_name` | `model` |
|--------|----------------|---------|
| **Use Case** | Production workflows | Quick experiments |
| **Configuration** | Centralized in config.toml | Inline in script/command |
| **Repeatability** | ✅ Consistent across runs | ⚠️ Must specify each time |
| **Version Control** | ✅ Tracked in config | ❌ Scattered in scripts |
| **Rotation** | ✅ Change once, affects all | ❌ Update each script |
| **Testing** | ✅ Swap provider easily | ⚠️ Manual substitution |
| **Parameters** | Full control (temp, tokens, timeout, retries) | Limited (model only) |

## When to Use `provider_name` (RECOMMENDED)

### ✅ Production Workflows

**Why**: Centralized configuration, easy rotation, version control

**Example**:
```toml
# config.toml
[providers.production-llm]
provider_type = "openai"
default_model = "gpt-4"
temperature = 0.3
max_tokens = 2000
timeout_seconds = 60
max_retries = 3
```

```lua
-- script.lua
local result = Template.execute("code-generator", {
    provider_name = "production-llm",  -- References config
    description = "Write a function to parse JSON"
})
```

**Benefits**:
- Change model by updating config.toml, not 50 scripts
- Consistent parameters across all invocations
- Easy A/B testing (switch provider, measure results)

### ✅ Repeated Invocations

**Why**: Avoids repeating model string in every call

**Example**:
```bash
# Bad: Repeating model in every call
llmspell template exec code-generator --param model="ollama/llama3.2:3b" ...
llmspell template exec data-analysis --param model="ollama/llama3.2:3b" ...
llmspell template exec research-assistant --param model="ollama/llama3.2:3b" ...

# Good: Use provider once, reference by name
llmspell template exec code-generator --param provider_name="default" ...
llmspell template exec data-analysis --param provider_name="default" ...
llmspell template exec research-assistant --param provider_name="default" ...
```

### ✅ Version-Controlled Configurations

**Why**: Config file tracks LLM choices over time

**Example**:
```toml
# config.toml (committed to git)
[providers]
default_provider = "staging-llm"

[providers.staging-llm]
provider_type = "ollama"
default_model = "llama3.2:3b"
temperature = 0.5

[providers.production-llm]
provider_type = "openai"
default_model = "gpt-4"
temperature = 0.3
```

**Benefits**:
- See LLM config changes in git history
- Environment-specific configs (dev, staging, prod)
- Team shares same LLM configuration

### ✅ Centralized LLM Settings Management

**Why**: Single source of truth for temperature, tokens, timeouts

**Example**:
```toml
[providers.fast-llm]
provider_type = "openai"
default_model = "gpt-3.5-turbo"
temperature = 0.7
max_tokens = 1000
timeout_seconds = 10  # Quick responses

[providers.quality-llm]
provider_type = "openai"
default_model = "gpt-4"
temperature = 0.2
max_tokens = 4096
timeout_seconds = 120  # Allow complex reasoning
```

```lua
-- Use fast-llm for prototyping
Template.execute("code-generator", { provider_name = "fast-llm", ... })

-- Use quality-llm for production
Template.execute("code-generator", { provider_name = "quality-llm", ... })
```

**Benefits**:
- Tune performance/cost trade-offs in one place
- Experiment with different temperature/token settings
- Consistent behavior across templates/agents

## When to Use `model` (AD-HOC)

### ✅ Quick Experiments

**Why**: Fastest way to try a different model without config changes

**Example**:
```bash
# Quick test: Does llama3.1 work better than llama3.2?
llmspell template exec code-generator \
  --param model="ollama/llama3.1:8b" \
  --param description="factorial function"

# Compare with different model
llmspell template exec code-generator \
  --param model="ollama/llama3.2:3b" \
  --param description="factorial function"
```

**When**: One-off tests, throwaway experiments

### ✅ Model Comparison

**Why**: Directly compare outputs from different models

**Example**:
```lua
-- Compare models side-by-side
local models = {
    "ollama/llama3.2:3b",
    "ollama/mistral:7b",
    "ollama/phi:3b"
}

for _, model in ipairs(models) do
    print("Testing model:", model)
    local result = Template.execute("code-generator", {
        model = model,
        description = "fibonacci function"
    })
    print("Result:", result.result.value)
    print("---")
end
```

**When**: Benchmarking, choosing best model for task

### ✅ Scripts with Explicit Control

**Why**: Script author wants full control over model choice

**Example**:
```lua
-- Standalone script that doesn't rely on config.toml
local result = Template.execute("research-assistant", {
    model = "ollama/llama3.2:3b",  -- Explicit choice
    topic = "Rust async programming",
    max_sources = 5
})
```

**When**: Portable scripts, tutorials, examples that work anywhere

### ✅ One-Off Testing

**Why**: Testing specific model behavior without polluting config

**Example**:
```bash
# Test if new model handles specific task
llmspell template exec data-analysis \
  --param model="ollama/codellama:13b" \
  --param data_file="sales.csv" \
  --param analysis_type="descriptive"
```

**When**: Validating new model, debugging model-specific issues

## Parameter Precedence

When both `provider_name` and `model` are specified, the system follows this precedence:

1. **`provider_name`** takes precedence (if provided)
2. **`model`** used only if `provider_name` not provided
3. **`default_provider`** from config.toml (fallback)
4. **Error** if none provided

**Example**:
```lua
-- ERROR: Cannot specify both
Template.execute("code-generator", {
    provider_name = "production-llm",
    model = "ollama/llama3.2:3b"  -- CONFLICT!
})
-- Error: "Cannot specify both provider_name and model - use one or the other"

-- OK: Use provider_name
Template.execute("code-generator", {
    provider_name = "production-llm"
})

-- OK: Use model
Template.execute("code-generator", {
    model = "ollama/llama3.2:3b"
})

-- OK: Use default_provider from config
Template.execute("code-generator", {
    description = "factorial function"
})
-- Uses config.toml: providers.default_provider
```

## Migration Guide

### From `model` to `provider_name`

**Before** (ad-hoc model strings):
```lua
Template.execute("code-generator", {
    model = "ollama/llama3.2:3b",
    temperature = 0.5,  -- Ignored! (model path doesn't support inline params)
    description = "factorial function"
})
```

**After** (centralized provider):
```toml
# config.toml
[providers.default]
provider_type = "ollama"
default_model = "llama3.2:3b"
temperature = 0.5  # Now respected!
max_tokens = 2000
timeout_seconds = 30
```

```lua
Template.execute("code-generator", {
    provider_name = "default",
    description = "factorial function"
})
```

**Benefits of Migration**:
- Temperature/max_tokens now work (ignored with `model` path)
- Easy model rotation (update config once)
- Consistent parameters across scripts

### From Multiple `model` Calls to Single Provider

**Before**:
```bash
#!/bin/bash
MODEL="ollama/llama3.2:3b"

llmspell template exec code-generator --param model="$MODEL" --param description="factorial"
llmspell template exec data-analysis --param model="$MODEL" --param data_file="sales.csv"
llmspell template exec research-assistant --param model="$MODEL" --param topic="Rust"
```

**After**:
```toml
# config.toml
[providers]
default_provider = "default"

[providers.default]
provider_type = "ollama"
default_model = "llama3.2:3b"
```

```bash
#!/bin/bash
# No model parameter needed - uses default_provider
llmspell template exec code-generator --param description="factorial"
llmspell template exec data-analysis --param data_file="sales.csv"
llmspell template exec research-assistant --param topic="Rust"
```

## Common Patterns

### Pattern 1: Environment-Specific Providers

```toml
# config.dev.toml
[providers]
default_provider = "dev-llm"

[providers.dev-llm]
provider_type = "ollama"
default_model = "llama3.2:3b"  # Fast local model for dev
temperature = 0.7

# config.prod.toml
[providers]
default_provider = "prod-llm"

[providers.prod-llm]
provider_type = "openai"
default_model = "gpt-4"  # High-quality model for prod
temperature = 0.3
max_retries = 5
```

```bash
# Development
llmspell --config config.dev.toml template exec code-generator ...

# Production
llmspell --config config.prod.toml template exec code-generator ...
```

### Pattern 2: Task-Specific Providers

```toml
[providers]
default_provider = "general-llm"

[providers.general-llm]
provider_type = "ollama"
default_model = "llama3.2:3b"
temperature = 0.7

[providers.code-llm]
provider_type = "ollama"
default_model = "codellama:13b"
temperature = 0.2  # Low temp for deterministic code

[providers.creative-llm]
provider_type = "openai"
default_model = "gpt-4"
temperature = 0.9  # High temp for creative writing
```

```lua
-- Use code-specific provider for code generation
Template.execute("code-generator", {
    provider_name = "code-llm",
    description = "factorial function"
})

-- Use creative provider for content generation
Template.execute("content-generation", {
    provider_name = "creative-llm",
    topic = "Future of AI"
})
```

### Pattern 3: Cost-Optimized Providers

```toml
[providers.cheap-llm]
provider_type = "ollama"
default_model = "llama3.2:3b"  # Free local model
max_tokens = 1000

[providers.expensive-llm]
provider_type = "openai"
default_model = "gpt-4"
max_tokens = 4096
```

```lua
-- Use cheap-llm for prototyping
Template.execute("code-generator", {
    provider_name = "cheap-llm",
    description = "simple function"
})

-- Use expensive-llm for production
Template.execute("code-generator", {
    provider_name = "expensive-llm",
    description = "complex algorithm with edge cases"
})
```

## Internal API Changes (For Developers)

### Memory System

**Old** (hardcoded model):
```rust
let llm_config = LlmEngineConfig {
    model: "ollama/llama3.2:3b".to_string(),
    temperature: 0.0,
    max_tokens: 2000,
    // ...
};
```

**New** (provider-based):
```rust
let provider = config.providers.get_provider("consolidation-llm")?;
let llm_config = LlmEngineConfig::from_provider(provider)?;
```

### Templates

**Old** (model string):
```rust
let model: String = params.get_or("model", "ollama/llama3.2:3b".to_string());
let req = LLMRequestBuilder::new(model)
    .temperature(Some(0.3))
    .build()?;
```

**New** (smart dual-path):
```rust
let provider_config = context.resolve_llm_config(&params)?;
let model = provider_config.default_model.as_ref()?;
let req = LLMRequestBuilder::new(model.clone())
    .temperature(provider_config.temperature)
    .max_tokens(provider_config.max_tokens.map(|t| t as usize))
    .build()?;
```

**Benefits**:
- Single `resolve_llm_config()` handles both `provider_name` and `model`
- Automatic fallback to `default_provider`
- Consistent temperature/tokens behavior

## FAQ

### Q: Should I always use `provider_name`?

**A**: For production code, yes. For quick experiments or model comparison, `model` is fine.

### Q: Can I override temperature when using `provider_name`?

**A**: Not directly. Temperature is part of the provider config. To use different temperatures:
1. Create separate providers (e.g., `low-temp-llm`, `high-temp-llm`)
2. Or use `model` path with inline temperature (limited support)

### Q: What if my provider doesn't exist?

**A**: You'll get error: "provider 'xyz' not found". Check:
1. Provider defined in `config.toml`: `[providers.xyz]`
2. Spelling matches exactly
3. Config file loaded correctly: `llmspell config show`

### Q: Can I use environment-specific providers?

**A**: Yes! Use different config files:
```bash
llmspell --config config.dev.toml ...   # Uses dev providers
llmspell --config config.prod.toml ...  # Uses prod providers
```

### Q: Do all templates support both paths?

**A**: Yes. All 10 builtin templates (Phase 12) support both `provider_name` and `model` parameters.

### Q: What about agents and workflows?

**A**: Provider system integration for agents/workflows is planned for Phase 13.7 (Kernel Integration).

## Related Documentation

- [Memory Configuration](memory-configuration.md) - Using providers with memory system
- [Configuration Guide](configuration.md) - Full provider configuration reference
- [Template User Guides](templates/README.md) - Template-specific examples
- [Local LLM Integration](local-llm.md) - Ollama and Candle provider setup

## Recommendations

**For Production**:
1. ✅ Use `provider_name` exclusively
2. ✅ Define providers in `config.toml` (version controlled)
3. ✅ Use environment-specific configs (dev/staging/prod)
4. ✅ Set `default_provider` for convenience

**For Development**:
1. ✅ Use `provider_name` with `default` provider for most work
2. ✅ Use `model` for quick model comparisons
3. ✅ Create task-specific providers (code, creative, cheap)

**For Scripts/Examples**:
1. ✅ Use `model` for standalone portability
2. ✅ Document required model in script comments
3. ✅ Provide both `provider_name` and `model` examples
