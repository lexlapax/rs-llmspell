# LLM Provider Configuration Guide

This guide explains how to configure and use different LLM providers in rs-llmspell.

## Provider/Model Syntax

As of v0.3.0, rs-llmspell uses a hierarchical naming convention for specifying providers and models:

```lua
-- Format: "provider/model"
local agent = Agent.create({
    model = "openai/gpt-4",
    system_prompt = "You are a helpful assistant"
})
```

This syntax allows for:
- Clear identification of which provider to use
- Automatic provider configuration
- Easy switching between providers
- Support for multiple models per provider

## Supported Providers

### OpenAI

**Models**:
- `openai/gpt-4` - Most capable model
- `openai/gpt-4-turbo` - Faster, more cost-effective GPT-4
- `openai/gpt-4o` - Optimized GPT-4
- `openai/gpt-4o-mini` - Smaller, faster variant
- `openai/gpt-3.5-turbo` - Fast, cost-effective model

**Configuration**:
```bash
# Set API key via environment variable
export OPENAI_API_KEY="sk-..."

# Optional: Custom endpoint
export OPENAI_BASE_URL="https://api.openai.com/v1"
```

**Example**:
```lua
local agent = Agent.create({
    model = "openai/gpt-4",
    temperature = 0.7,
    max_tokens = 2000
})
```

### Anthropic

**Models**:
- `anthropic/claude-3-opus` - Most capable Claude model
- `anthropic/claude-3-sonnet` - Balanced performance
- `anthropic/claude-3-haiku` - Fast, efficient model
- `anthropic/claude-2.1` - Previous generation
- `anthropic/claude-instant` - Fastest option

**Configuration**:
```bash
# Set API key via environment variable
export ANTHROPIC_API_KEY="sk-ant-..."

# Optional: Custom endpoint
export ANTHROPIC_BASE_URL="https://api.anthropic.com"
```

**Example**:
```lua
local agent = Agent.create({
    model = "anthropic/claude-3-sonnet",
    temperature = 0.5,
    max_tokens = 4000
})
```

### Google (Gemini)

**Models**:
- `google/gemini-pro` - General purpose model
- `google/gemini-pro-vision` - Multimodal support
- `google/gemini-ultra` - Most capable (when available)

**Configuration**:
```bash
# Set API key via environment variable
export GOOGLE_API_KEY="..."

# Optional: Custom endpoint
export GOOGLE_BASE_URL="https://generativelanguage.googleapis.com"
```

### Cohere

**Models**:
- `cohere/command` - Instruction following
- `cohere/command-light` - Faster variant
- `cohere/command-nightly` - Latest features

**Configuration**:
```bash
export COHERE_API_KEY="..."
```

### Local/Custom Providers

**Models**:
- `local/llama2` - Local Llama 2 instance
- `local/mistral` - Local Mistral instance
- `custom/your-model` - Any custom provider

**Configuration**:
```bash
# For local models
export LOCAL_MODEL_PATH="/path/to/model"
export LOCAL_BASE_URL="http://localhost:8080"

# For custom providers
export CUSTOM_API_KEY="..."
export CUSTOM_BASE_URL="https://your-api.com"
```

## Configuration Methods

### 1. Environment Variables (Recommended)

The recommended way to configure providers is through environment variables:

```bash
# In your .env file or shell configuration
export OPENAI_API_KEY="sk-..."
export ANTHROPIC_API_KEY="sk-ant-..."
export GOOGLE_API_KEY="..."
```

### 2. Runtime Configuration

You can override provider settings at runtime:

```lua
local agent = Agent.create({
    model = "openai/gpt-4",
    api_key = "sk-...",  -- Override default API key
    base_url = "https://custom-endpoint.com/v1"  -- Custom endpoint
})
```

### 3. Configuration File

Create a `providers.toml` configuration file:

```toml
[providers.openai]
api_key = "sk-..."
base_url = "https://api.openai.com/v1"
default_model = "gpt-4"

[providers.anthropic]
api_key = "sk-ant-..."
base_url = "https://api.anthropic.com"
default_model = "claude-3-sonnet"

[providers.custom]
api_key = "..."
base_url = "https://your-api.com"
headers = { "X-Custom-Header" = "value" }
```

## Advanced Configuration

### Custom Headers

Some providers require custom headers:

```lua
local agent = Agent.create({
    model = "custom/model",
    headers = {
        ["X-API-Version"] = "2024-01",
        ["X-Custom-Auth"] = "Bearer token"
    }
})
```

### Proxy Configuration

For enterprise environments with proxy requirements:

```bash
export HTTP_PROXY="http://proxy.company.com:8080"
export HTTPS_PROXY="http://proxy.company.com:8080"
export NO_PROXY="localhost,127.0.0.1"
```

### Rate Limiting

Configure rate limits per provider:

```lua
local agent = Agent.create({
    model = "openai/gpt-4",
    rate_limit = {
        requests_per_minute = 60,
        tokens_per_minute = 40000
    }
})
```

## Provider-Specific Features

### OpenAI Functions

OpenAI models support function calling:

```lua
local agent = Agent.create({
    model = "openai/gpt-4",
    functions = {
        {
            name = "get_weather",
            description = "Get current weather",
            parameters = {
                type = "object",
                properties = {
                    location = {type = "string"}
                }
            }
        }
    }
})
```

### Anthropic System Prompts

Anthropic models have specific system prompt handling:

```lua
local agent = Agent.create({
    model = "anthropic/claude-3-sonnet",
    system_prompt = "You are Claude, an AI assistant created by Anthropic."
})
```

### Google Safety Settings

Google models support safety configurations:

```lua
local agent = Agent.create({
    model = "google/gemini-pro",
    safety_settings = {
        harassment = "BLOCK_MEDIUM_AND_ABOVE",
        hate_speech = "BLOCK_MEDIUM_AND_ABOVE"
    }
})
```

## Error Handling

Handle provider-specific errors gracefully:

```lua
local success, result = pcall(function()
    return Agent.create({
        model = "openai/gpt-4",
        system_prompt = "Hello"
    })
end)

if not success then
    if result:match("API key") then
        print("Error: Missing or invalid API key")
    elseif result:match("rate limit") then
        print("Error: Rate limit exceeded")
    elseif result:match("model not found") then
        print("Error: Invalid model name")
    else
        print("Error: " .. result)
    end
end
```

## Best Practices

1. **API Key Security**:
   - Never hardcode API keys in scripts
   - Use environment variables or secure vaults
   - Rotate keys regularly
   - Use different keys for dev/prod

2. **Model Selection**:
   - Use smaller models for simple tasks
   - Reserve larger models for complex reasoning
   - Consider cost vs performance tradeoffs
   - Test with multiple models

3. **Error Recovery**:
   - Implement retry logic with backoff
   - Have fallback models configured
   - Log errors for monitoring
   - Handle rate limits gracefully

4. **Performance Optimization**:
   - Cache responses when appropriate
   - Batch requests where possible
   - Use streaming for long responses
   - Monitor token usage

## Provider Comparison

| Provider | Strengths | Best For | Pricing |
|----------|-----------|----------|---------|
| OpenAI | Most mature API, function calling | General purpose, code generation | Per-token |
| Anthropic | Long context, safety-focused | Analysis, writing, research | Per-token |
| Google | Multimodal, large context | Visual tasks, long documents | Per-character |
| Cohere | Retrieval augmented | Search, classification | Per-request |
| Local | Privacy, no API limits | Sensitive data, offline use | Infrastructure |

## Troubleshooting

### Common Issues

**"Provider not found"**
- Check the provider name spelling
- Ensure the provider is supported
- Verify the model name format

**"Invalid API key"**
- Check environment variable name
- Verify key hasn't expired
- Ensure correct key format

**"Model not available"**
- Verify model name and availability
- Check if you have access to the model
- Try a different model variant

**"Connection timeout"**
- Check network connectivity
- Verify proxy settings if applicable
- Try custom base URL if behind firewall

### Debug Mode

Enable debug logging for provider issues:

```bash
export LLMSPELL_LOG_LEVEL=debug
export LLMSPELL_LOG_PROVIDERS=true
```

## Future Enhancements

Upcoming provider features:
- Automatic provider fallback
- Load balancing across providers
- Provider-specific caching
- Unified streaming interface
- Cost tracking and optimization

## Related Documentation

- [Agent API Reference](agent-api.md) - Complete agent API documentation
- [Getting Started](getting-started.md) - Quick start guide
- [Configuration Guide](configuration.md) - Full configuration options