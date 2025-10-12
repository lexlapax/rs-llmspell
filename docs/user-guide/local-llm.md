# Local LLM Integration Guide

**Version**: 1.0 (Phase 11)
**Status**: Production Ready
**Last Updated**: 2025-10-04

## Overview

LLMSpell supports local LLM inference through two backends:
- **Ollama**: Production LLM server (recommended for most users)
- **Candle**: Embedded GGUF inference (for portable/offline use)

Both backends provide identical APIs for model management and inference.

## Quick Start

### Option 1: Ollama (Recommended)

**1. Install Ollama**
```bash
# macOS
brew install ollama

# Linux
curl -fsSL https://ollama.com/install.sh | sh

# Start Ollama server
ollama serve
```

**2. Pull a model**
```bash
# Via Ollama CLI
ollama pull llama3.1:8b

# Or via LLMSpell
llmspell model pull llama3.1:8b@ollama
```

**3. Use in Lua script**
```lua
local agent = Agent.create({
    model = "local/llama3.1:8b@ollama"
})

local response = agent:complete("Explain Rust ownership in one sentence")
print(response)
```

### Option 2: Candle (Embedded)

**1. Set HuggingFace API key** (optional, for higher rate limits)
```bash
export HFHUB_API_KEY=hf_...
```

**2. Pull a model**
```bash
llmspell model pull tinyllama:Q4_K_M@candle
```

**3. Use in Lua script**
```lua
local agent = Agent.create({
    model = "local/tinyllama:Q4_K_M@candle"
})

local response = agent:complete("Write a haiku about code")
print(response)
```

## Model Management

### Listing Models

**CLI:**
```bash
# List all local models
llmspell model list

# List Ollama models only
llmspell model list --backend ollama

# List Candle models only
llmspell model list --backend candle
```

**Lua API:**
```lua
-- List all models
local models = LocalLLM.list()
for _, model in ipairs(models) do
    print(model.id, model.backend, model.size_bytes)
end

-- List Ollama models only
local ollama_models = LocalLLM.list("ollama")

-- List Candle models only
local candle_models = LocalLLM.list("candle")
```

### Pulling Models

**CLI:**
```bash
# Pull from Ollama
llmspell model pull llama3.1:8b@ollama
llmspell model pull mistral:7b@ollama

# Pull from HuggingFace (Candle)
llmspell model pull tinyllama:Q4_K_M@candle
llmspell model pull phi-2:Q4_K_M@candle
```

**Lua API:**
```lua
-- Pull Ollama model
LocalLLM.pull("llama3.1:8b@ollama")

-- Pull Candle model
LocalLLM.pull("tinyllama:Q4_K_M@candle")
```

### Model Information

**CLI:**
```bash
llmspell model info llama3.1:8b@ollama
llmspell model info tinyllama:Q4_K_M@candle
```

**Lua API:**
```lua
local info = LocalLLM.info("llama3.1:8b@ollama")
print("Model:", info.id)
print("Backend:", info.backend)
print("Size:", info.size_bytes, "bytes")
print("Format:", info.format)
```

### Backend Status

**CLI:**
```bash
# Check all backends
llmspell model status

# Check specific backend
llmspell model status --backend ollama
```

**Lua API:**
```lua
-- Get status for all backends
local status = LocalLLM.status()

-- Check Ollama status
print("Ollama running:", status.ollama.running)
print("Ollama models:", status.ollama.models)

-- Check Candle status
print("Candle ready:", status.candle.ready)
print("Candle models:", status.candle.models)
```

## Configuration

### Ollama Configuration

`~/.llmspell/config.toml`:
```toml
[providers.ollama]
enabled = true

[providers.ollama.options]
# Ollama server URL (default: http://localhost:11434)
url = "http://localhost:11434"

# Connection timeout in seconds
timeout = 30
```

### Candle Configuration

`~/.llmspell/config.toml`:
```toml
[providers.candle]
enabled = true

[providers.candle.options]
# Model storage directory (default: ~/.llmspell/models/candle)
model_directory = "~/.llmspell/models/candle"

# Device selection: "auto", "cpu", "cuda", "metal"
device = "auto"

# HuggingFace API key (for downloads)
# Can also set via HFHUB_API_KEY environment variable
hf_api_key = "hf_..."
```

## Model Specification Syntax

### Full Syntax
```
local/MODEL_NAME:VARIANT@BACKEND
```

### Examples
```lua
-- Ollama with explicit backend
"local/llama3.1:8b@ollama"

-- Candle with explicit backend
"local/tinyllama:Q4_K_M@candle"

-- Auto-detect backend (defaults to Ollama)
"local/llama3.1:8b"
```

### Backend Auto-Detection
If `@backend` is omitted, LLMSpell tries backends in order:
1. Ollama (if available)
2. Candle (fallback)

## Supported Models

### Ollama
Any model available via `ollama pull`:
- llama3.1:8b, llama3.1:70b
- mistral:7b, mistral:7b-instruct
- phi-3:mini, phi-3:medium
- qwen2:7b, qwen2:72b
- gemma2:2b, gemma2:9b

See: https://ollama.com/library

### Candle (GGUF)
Quantized models from HuggingFace:
- `tinyllama:Q4_K_M` - TinyLlama 1.1B (638MB, validated)
- `phi-2:Q4_K_M` - Microsoft Phi-2 2.7B
- `qwen2-0.5b:Q4_K_M` - Qwen2 0.5B

**Supported Quantizations:**
- Q4_K_M (recommended, best quality/size balance)
- Q5_K_M (higher quality, larger)
- Q8_0 (highest quality, largest)

## Performance Characteristics

### Ollama
**Strengths:**
- Production-ready server architecture
- Multi-model support
- GPU acceleration (CUDA, Metal, ROCm)
- Model caching and pooling
- <100ms first token latency (typical)
- >20 tokens/sec for 7B models

**Best For:**
- Development workflows
- Long-running services
- Multiple concurrent requests
- Large models (>7B parameters)

### Candle
**Strengths:**
- Embedded (no server required)
- Portable (single binary)
- Works offline
- ~150ms first token latency (GPU)
- ~40 tokens/sec for small models (GPU)
- Low memory overhead (~400MB)

**Best For:**
- Portable scripts
- Offline environments
- Small models (<3B parameters)
- CI/CD integration
- Air-gapped systems

**Limitations:**
- No streaming (generates full response)
- No model caching (reloads each request)
- No batching (one request at a time)
- LLaMA architecture only (Mistral/Qwen compatible)

## Troubleshooting

### Ollama: Connection Refused
```
Error: Failed to connect to Ollama server
```

**Solutions:**
1. Ensure Ollama is running: `ollama serve`
2. Check URL in config: `url = "http://localhost:11434"`
3. Verify Ollama installed: `ollama --version`

### Candle: Tokenizer Not Found
```
Error: Tokenizer file not found
```

**Solutions:**
1. Re-download model: `llmspell model pull MODEL:VARIANT@candle`
2. Check model directory: `~/.llmspell/models/candle/`
3. Set HFHUB_API_KEY for authenticated downloads

### Candle: Out of Memory
```
Error: Failed to allocate tensor
```

**Solutions:**
1. Use smaller model: `tinyllama:Q4_K_M` instead of larger models
2. Switch to CPU: Set `device = "cpu"` in config
3. Use Ollama for large models instead

### Model Pull Fails
```
Error: Failed to download model
```

**Solutions:**
1. Check internet connection
2. Verify model name correct
3. For Candle: Set `HFHUB_API_KEY` environment variable
4. Try pulling via backend CLI directly:
   - Ollama: `ollama pull MODEL`
   - Candle: Manual download from HuggingFace

### Slow Inference
**Ollama:**
- Ensure GPU available: `ollama ps` should show GPU
- Use quantized models: 7B-Q4 instead of 70B-FP16

**Candle:**
- Check device selection: `device = "auto"` (should detect GPU)
- Use Q4_K_M quantization (fastest)
- Try smaller models (TinyLlama 1.1B vs Phi-2 2.7B)

## Examples

### Chat Loop
```lua
local agent = Agent.create({
    model = "local/llama3.1:8b@ollama",
    system_prompt = "You are a helpful coding assistant."
})

while true do
    io.write("You: ")
    local user_input = io.read()

    if user_input == "exit" then break end

    local response = agent:complete(user_input)
    print("Assistant:", response)
end
```

### Backend Comparison
```lua
local ollama_agent = Agent.create({
    model = "local/llama3.1:8b@ollama"
})

local candle_agent = Agent.create({
    model = "local/tinyllama:Q4_K_M@candle"
})

local prompt = "Explain async/await in JavaScript"

print("=== Ollama ===")
print(ollama_agent:complete(prompt))

print("\n=== Candle ===")
print(candle_agent:complete(prompt))
```

### Model Information Script
```lua
print("=== Local LLM Status ===\n")

-- Get status for all backends
local status = LocalLLM.status()

-- Check Ollama
print("ollama:")
print("  Running: " .. tostring(status.ollama.running))
print("  Models: " .. status.ollama.models)
if status.ollama.error then
    print("  Error: " .. status.ollama.error)
end
print()

-- Check Candle
print("candle:")
print("  Ready: " .. tostring(status.candle.ready))
print("  Models: " .. status.candle.models)
if status.candle.error then
    print("  Error: " .. status.candle.error)
end
print()

-- List all models
print("=== Available Models ===\n")
local models = LocalLLM.list()
for _, model in ipairs(models) do
    print(model.id .. " (" .. model.backend .. ")")
    print("  Size: " .. math.floor(model.size_bytes / 1024 / 1024) .. " MB")
    print()
end
```

## Best Practices

1. **Use Ollama for production**: Server architecture handles concurrent requests better
2. **Use Candle for portability**: Embedded inference works in air-gapped environments
3. **Start with smaller models**: TinyLlama (1.1B) or Phi-3 Mini (3.8B) for testing
4. **Set explicit backend**: Use `@ollama` or `@candle` to avoid auto-detection overhead
5. **Monitor memory**: Large models (>7B) need significant RAM/VRAM
6. **Use quantization**: Q4_K_M provides good quality at 1/4 the size of FP16

## See Also

- [Getting Started Guide](getting-started.md)
- [Configuration Guide](configuration.md)
- [Lua API Reference](api/lua/README.md)
- [Performance Tuning](performance-tuning.md)
- [Troubleshooting](troubleshooting.md)
