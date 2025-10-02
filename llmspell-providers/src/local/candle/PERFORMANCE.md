# Candle Provider Performance Characteristics

**Date**: 2025-10-02
**Candle Version**: 0.9.1
**Implementation**: Phase 11.7

## Performance Metrics

The Candle provider tracks the following performance metrics during inference:

### 1. Model Loading Time
- **Metric**: Time to load GGUF file and initialize model
- **Logged as**: "Model loaded in {seconds}s"
- **Typical Range**:
  - TinyLlama-1.1B Q4_K_M: 1-3s (CPU), 0.5-1.5s (GPU)
  - Phi-2 Q4_K_M: 2-5s (CPU), 1-3s (GPU)

### 2. Tokenization Time
- **Metric**: Time to encode prompt text to token IDs
- **Logged as**: "Prompt tokenized: {tokens} tokens in {ms}ms"
- **Typical Range**: <10ms for prompts <1000 tokens

### 3. First Token Latency (TTFT - Time To First Token)
- **Metric**: Time from prompt submission to first generated token
- **Includes**: Prompt processing through model forward pass
- **Logged as**: "First token latency: {ms}ms ({prompt_tokens} prompt tokens)"
- **Typical Range**:
  - TinyLlama-1.1B Q4_K_M CPU: 200-500ms
  - TinyLlama-1.1B Q4_K_M Metal/CUDA: 50-150ms

### 4. Generation Speed
- **Metric**: Tokens generated per second (subsequent tokens)
- **Logged as**: "Generated {tokens} tokens in {ms}ms ({tokens/sec} tokens/sec)"
- **Typical Range**:
  - CPU: 5-15 tokens/sec
  - Metal/CUDA: 20-50 tokens/sec

### 5. Decoding Time
- **Metric**: Time to decode token IDs back to text
- **Logged as**: Part of "Total generation" breakdown
- **Typical Range**: <5ms for outputs <500 tokens

### 6. Total Generation Time
- **Metric**: End-to-end time from request to response
- **Logged as**: "Total generation: {ms}ms (tokenize: {ms}ms, first token: {ms}ms, generation: {ms}ms, decode: {ms}ms)"
- **Components**:
  - Tokenize: Input text → token IDs
  - First token: Prompt processing
  - Generation: Autoregressive token generation
  - Decode: Token IDs → output text

## Device Performance Comparison

### CPU (Apple M-series, Intel i7/i9, AMD Ryzen)
- **Model Loading**: 2-5s
- **First Token**: 200-500ms
- **Generation**: 5-15 tokens/sec
- **Memory**: ~4-8GB for 7B Q4_K_M

### Metal (Apple Silicon)
- **Model Loading**: 1-3s
- **First Token**: 50-150ms
- **Generation**: 20-50 tokens/sec
- **Memory**: ~4-8GB for 7B Q4_K_M

### CUDA (NVIDIA GPUs)
- **Model Loading**: 1-3s
- **First Token**: 50-150ms (depends on GPU)
- **Generation**: 30-80 tokens/sec (depends on GPU)
- **Memory**: ~4-8GB VRAM for 7B Q4_K_M

## Optimization Techniques

### 1. Built-in KV Cache
- Candle's `quantized_llama::ModelWeights` includes automatic KV cache
- Cache is preserved across tokens in a single generation
- No manual cache management needed
- Memory usage grows with sequence length (~400MB per 2048 tokens)

### 2. Quantization Formats
- **Q4_K_M**: Best quality/size tradeoff (recommended)
- **Q5_K_M**: Higher quality, slightly larger
- **Q8_0**: Maximum quality, 2x larger than Q4
- **Q4_0**: Smallest, lower quality

### 3. Batch Size
- Current implementation: batch_size = 1 (single request)
- Optimal for chat/interactive use cases
- Batching not yet implemented

### 4. Context Length
- Default: Model-dependent (usually 2048-4096)
- Longer contexts increase first token latency linearly
- KV cache memory grows with context

## Performance Validation

To validate performance:

```bash
# Enable trace-level logging
RUST_LOG=llmspell_providers=info cargo run --release

# Run inference
llmspell run test_candle.lua

# Check logs for timing metrics
```

Expected log output:
```
[INFO] Model loaded in 2.34s
[INFO] Prompt tokenized: 12 tokens in 2.15ms
[INFO] First token latency: 156.42ms (12 prompt tokens)
[INFO] Generated 50 tokens in 1234.56ms (40.51 tokens/sec)
[INFO] Total generation: 1395.28ms (tokenize: 2.15ms, first token: 156.42ms, generation: 1234.56ms, decode: 2.15ms)
```

## Known Limitations

1. **No Streaming**: Current implementation generates all tokens before returning
2. **No Batching**: Single request at a time
3. **No Speculative Decoding**: Sequential token generation only
4. **Memory**: Models stay in memory until provider is dropped
5. **Context Window**: Limited by model's max sequence length

## Future Optimizations

- [ ] Implement streaming responses
- [ ] Add batch inference support
- [ ] Explore Flash Attention integration
- [ ] Add model pooling/caching between requests
- [ ] Implement speculative decoding
- [ ] Add dynamic quantization options
- [ ] Support longer context via RoPE scaling

## Benchmarking

For formal benchmarking, see integration tests:
- `tests/candle_integration_test.rs`
- Run with: `RUN_EXPENSIVE_TESTS=1 cargo test --release candle_performance`

## References

- Candle Performance: https://github.com/huggingface/candle/discussions/performance
- GGUF Quantization: https://github.com/ggerganov/ggml/blob/master/docs/gguf.md
- LLM Inference Optimization: https://lilianweng.github.io/posts/2023-01-10-inference-optimization/
