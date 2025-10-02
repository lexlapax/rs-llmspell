# Candle 0.9 GGUF API Research

**Date**: 2025-10-02
**Purpose**: Document Candle 0.9 API for GGUF model loading and inference
**Status**: Complete research for llmspell-providers implementation

## Key API Components

### 1. GGUF File Loading

**Module**: `candle::quantized::gguf_file`

```rust
use candle::quantized::gguf_file;
use std::fs::File;

// Open GGUF file
let mut file = File::open(path)?;

// Read GGUF content (metadata + tensor info)
let content = gguf_file::Content::read(&mut file)?;
```

**Content struct**:
```rust
pub struct Content {
    pub magic: VersionedMagic,                     // GGUF version
    pub metadata: HashMap<String, Value>,           // Model metadata
    pub tensor_infos: HashMap<String, TensorInfo>,  // Tensor information
    pub tensor_data_offset: u64,                   // Offset to tensor data
}
```

**Metadata access**:
```rust
// Extract metadata values
let head_count = content.metadata
    .get("llama.attention.head_count")?
    .to_u32()? as usize;

let block_count = content.metadata
    .get("llama.block_count")?
    .to_u32()? as usize;

let embedding_length = content.metadata
    .get("llama.embedding_length")?
    .to_u32()? as usize;
```

**Tensor loading**:
```rust
// Load individual tensors by name
let tensor = content.tensor(&mut file, "token_embd.weight", &device)?;
// Returns QTensor (quantized tensor)
```

### 2. Quantized LLaMA Model

**Module**: `candle_transformers::models::quantized_llama`

**Model loading**:
```rust
use candle_transformers::models::quantized_llama;
use candle::Device;

// Load model from GGUF
let device = Device::cuda_if_available(0)?; // or Device::Cpu
let model = quantized_llama::ModelWeights::from_gguf(
    content,        // gguf_file::Content
    &mut file,      // mutable file reference
    &device         // target device
)?;
```

**ModelWeights struct**:
```rust
pub struct ModelWeights {
    tok_embeddings: Embedding,
    layers: Vec<LayerWeights>,
    norm: RmsNorm,
    output: QMatMul,
    masks: HashMap<usize, Tensor>,
    // Internal spans for tracing
}
```

**Forward pass**:
```rust
// Generate next token logits
let logits = model.forward(&input_tensor, index_pos)?;

// Parameters:
// - input_tensor: Tensor with shape (batch_size, seq_len) containing token IDs
// - index_pos: Current position for KV cache (0 for first token, increments)
```

### 3. KV Cache (Built-in)

**Automatic KV cache management**:
- Each `LayerWeights` has `kv_cache: Option<(Tensor, Tensor)>`
- Cache automatically updated in `forward_attn()`
- Concatenates previous k/v with new k/v on each forward pass
- **No manual KV cache implementation needed**

**Cache behavior**:
```rust
// On first token (index_pos = 0):
// - Creates new K, V tensors
// - Stores in kv_cache

// On subsequent tokens:
// - Loads previous k_cache, v_cache
// - Concatenates with new k, v
// - Updates kv_cache

// Code from quantized_llama.rs:
let (k, v) = match &self.kv_cache {
    None => (k, v),
    Some((k_cache, v_cache)) => {
        if index_pos == 0 {
            (k, v)
        } else {
            let k = Tensor::cat(&[k_cache, &k], 2)?;
            let v = Tensor::cat(&[v_cache, &v], 2)?;
            (k, v)
        }
    }
};
self.kv_cache = Some((k.clone(), v.clone()));
```

### 4. Token Sampling

**Available in candle-nn**:
- `gumbel_softmax()` - Gumbel-Softmax sampling with temperature

**Custom sampling needed**:
- Top-p (nucleus) sampling
- Top-k sampling
- Repeat penalty
- Combined sampling strategy

**Basic sampling pattern**:
```rust
// Get logits from model
let logits = model.forward(&input_tensor, pos)?;

// Apply temperature
let logits = (logits / temperature)?;

// Sample token
let next_token = sample_token(&logits)?; // Custom implementation needed
```

## Complete GGUF Inference Flow

```rust
use candle::{Device, Tensor};
use candle::quantized::gguf_file;
use candle_transformers::models::quantized_llama;
use tokenizers::Tokenizer;
use std::fs::File;

// 1. Setup
let device = Device::cuda_if_available(0)?;
let mut file = File::open("model.gguf")?;
let tokenizer = Tokenizer::from_file("tokenizer.json")?;

// 2. Load model
let content = gguf_file::Content::read(&mut file)?;
let mut model = quantized_llama::ModelWeights::from_gguf(content, &mut file, &device)?;

// 3. Tokenize prompt
let encoding = tokenizer.encode(prompt, true)?;
let tokens = encoding.get_ids();
let input_tensor = Tensor::new(tokens, &device)?;

// 4. Process prompt (index_pos = 0)
let input_tensor = input_tensor.unsqueeze(0)?; // Add batch dimension
let logits = model.forward(&input_tensor, 0)?;

// 5. Generate tokens (index_pos increments)
let mut generated = Vec::new();
for pos in 1..max_tokens {
    // Sample next token from logits
    let next_token = sample(&logits)?;

    // Check for EOS
    if next_token == eos_token {
        break;
    }

    generated.push(next_token);

    // Forward pass with single token
    let input_tensor = Tensor::new(&[next_token], &device)?.unsqueeze(0)?;
    let logits = model.forward(&input_tensor, pos)?;
}

// 6. Decode generated tokens
let text = tokenizer.decode(&generated, true)?;
```

## Critical API Differences from 0.7

### VarBuilder removed from from_gguf
**0.7**: `from_gguf(var_builder, config)` - required manual VarBuilder creation
**0.9**: `from_gguf(content, reader, device)` - Content handles everything

### Config extraction
**0.7**: Config passed as parameter
**0.9**: Config extracted from GGUF metadata automatically

### Tensor loading
**0.7**: Used VarBuilder.get()
**0.9**: Uses `content.tensor(reader, name, device)` directly

## Metadata Keys Reference

Common GGUF metadata keys for LLaMA models:

```rust
// Architecture
"llama.attention.head_count"          // Number of attention heads
"llama.attention.head_count_kv"       // Number of KV heads (for GQA/MQA)
"llama.block_count"                   // Number of transformer layers
"llama.embedding_length"              // Hidden dimension size
"llama.rope.dimension_count"          // RoPE dimension
"llama.rope.freq_base"                // RoPE frequency base (default: 10000)

// Normalization
"llama.attention.layer_norm_rms_epsilon"  // RMS norm epsilon

// MoE (if present)
"llama.expert_count"                  // Number of experts (0 if not MoE)
"llama.expert_used_count"             // Number of experts used per token

// General
"general.alignment"                   // Tensor data alignment (default: 32)
"general.architecture"                // Model architecture (e.g., "llama")
"general.name"                        // Model name
```

## Supported Quantization Formats

From `candle::quantized::GgmlDType`:
- Q4_0, Q4_1, Q4_K
- Q5_0, Q5_1, Q5_K
- Q6_K
- Q8_0, Q8_K
- F16, F32
- And more (see candle-core/src/quantized/mod.rs)

**Recommended for llmspell**:
- Q4_K_M - Best quality/size tradeoff
- Q5_K_M - Higher quality, slightly larger
- Q8_0 - Maximum quality, larger size

## Performance Considerations

### Device Selection
```rust
// Try CUDA first, fall back to Metal, then CPU
let device = if let Ok(cuda) = Device::cuda_if_available(0) {
    cuda
} else if let Ok(metal) = Device::new_metal(0) {
    metal
} else {
    Device::Cpu
};
```

### Memory Management
- Model loading: ~4-8GB for 7B Q4_K_M model
- KV cache grows with sequence length: ~400MB per 2048 tokens
- Unload model by dropping ModelWeights

### Optimization Tips
1. **Batch size = 1** for chat/interactive use
2. **Use GPU** for <200ms first token latency
3. **CPU inference** works but slower (~1-2 sec first token)
4. **Cache model** in memory between requests
5. **Clear KV cache** between unrelated prompts

## Testing Resources

### Test Models (Small, Fast Download)
- TinyLlama-1.1B-Chat-v1.0 GGUF (~600MB)
- Phi-2 GGUF (~1.5GB)
- Qwen2-0.5B GGUF (~400MB)

### HuggingFace GGUF Repos
- TheBloke/* - Most models have GGUF versions
- bartowski/* - Newer models
- lmstudio-community/* - Well-organized

## Implementation Checklist for llmspell

- [x] Research Candle 0.9 API
- [x] Understand GGUF loading flow
- [x] Understand KV cache (built-in, automatic)
- [ ] Implement GGUFLoader wrapper
- [ ] Implement TokenizerWrapper
- [ ] Implement custom sampling (top-p, top-k, repeat penalty)
- [ ] Implement inference loop in CandleProvider
- [ ] Implement HuggingFace download (hf-hub crate)
- [ ] Test with TinyLlama GGUF
- [ ] Benchmark performance
- [ ] Document usage

## Next Steps

1. Create proof-of-concept loader in llmspell-providers
2. Test with small GGUF model (TinyLlama)
3. Implement production-ready CandleProvider
4. Add sampling strategies
5. Integrate with existing llmspell architecture

## References

- Candle repo: https://github.com/huggingface/candle
- Candle docs: https://docs.rs/candle-core/0.9.1
- GGUF spec: https://github.com/ggerganov/ggml/blob/master/docs/gguf.md
- Source code: `~/.cargo/registry/src/.../candle-*/`
