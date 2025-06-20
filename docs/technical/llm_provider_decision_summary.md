# LLM Provider Layer Decision Summary

## Executive Summary

Based on comprehensive research of existing Rust LLM crates, we recommend adopting **rig** as the foundation for rs-llmspell's LLM provider abstraction, with **candle** for local model support and a custom BaseAgent/Agent/Tool hierarchy built on top.

## Decision Rationale

### Why rig?

1. **Most Extensible Architecture**
   - Trait-based `CompletionModel` allows custom provider implementations
   - Modular design with separate crates for each provider
   - Clean separation between completion and embedding models

2. **Production-Ready Features**
   - Full streaming support with `StreamingCompletion` trait
   - Comprehensive provider coverage (OpenAI, Anthropic, Gemini, etc.)
   - High-level Agent abstraction that aligns with our design
   - Extensive vector store integrations

3. **Superior Design Patterns**
   - Builder pattern for requests and agents
   - Async-first with tokio runtime
   - Tool calling and RAG support built-in
   - Audio generation and transcription capabilities

### Why Not Others?

1. **rust-genai**: Good multi-provider support but limited extensibility (fixed enum)
2. **langchain-rust**: Too heavy and opinionated for our bridge-first approach
3. **llm-chain**: Good for workflows but limited provider coverage
4. **async-openai**: Excellent but single-provider only
5. **rllm**: Good features but unclear streaming status

### Integration with candle

For local model support, **candle** provides:
- Minimalist ML framework for serverless inference
- WASM support for browser-based inference
- Extensive model support (LLaMA, Mistral, Gemma, etc.)
- Maintained by Hugging Face team

## Implementation Strategy

### Phase 1: Provider Abstraction
```rust
// Wrap rig's CompletionModel for our needs
trait LLMProvider: Send + Sync {
    type Stream: Stream<Item = Result<StreamChunk>>;
    
    async fn complete(&self, request: CompletionRequest) -> Result<CompletionResponse>;
    async fn stream(&self, request: CompletionRequest) -> Result<Self::Stream>;
    fn count_tokens(&self, text: &str) -> Result<usize>;
}

// Implement for rig providers
impl<T: CompletionModel> LLMProvider for RigProviderWrapper<T> {
    // Bridge implementation
}
```

### Phase 2: Agent Hierarchy Integration
```rust
// Our BaseAgent uses rig's completion models
struct BaseAgent {
    provider: Box<dyn LLMProvider>,
    tools: Vec<Box<dyn Tool>>,
    hooks: HookManager,
    state: AgentState,
}

// rig agents can be wrapped as tools
impl Tool for RigAgentWrapper {
    async fn execute(&self, input: ToolInput) -> Result<ToolOutput> {
        // Delegate to rig agent
    }
}
```

### Phase 3: Custom Extensions
```rust
// Provider registry for dynamic loading
struct ProviderRegistry {
    providers: HashMap<String, Box<dyn ProviderFactory>>,
}

// Local model provider using candle
struct CandleProvider {
    model: candle::Model,
    tokenizer: Tokenizer,
}

impl LLMProvider for CandleProvider {
    // Implementation using candle
}
```

## Key Design Decisions

### 1. Hybrid Approach
- Use rig for provider abstraction and communication
- Build custom BaseAgent/Agent/Tool hierarchy on top
- Leverage rig's streaming and tool infrastructure

### 2. Extensibility Points
- Custom providers via `CompletionModel` trait
- Local models through candle integration
- Provider registry for dynamic loading
- Plugin system for community providers

### 3. Bridge Pattern
```rust
// Script-friendly wrapper
struct ScriptableLLM {
    inner: Arc<dyn LLMProvider>,
    config: LLMConfig,
}

// Lua/JS bindings
impl UserData for ScriptableLLM {
    // Methods exposed to scripts
}
```

## Migration Path

### From Current Design
1. Replace existing LLM traits with rig-based abstraction
2. Wrap rig providers in our LLMProvider trait
3. Implement BaseAgent/Agent hierarchy using providers
4. Add script bindings for new components

### Future Enhancements
1. Custom provider implementations
2. Local model support via candle
3. Advanced features (fine-tuning, embeddings)
4. Performance optimizations

## Risk Mitigation

### Dependencies
- rig is actively maintained with regular updates
- Modular design limits impact of changes
- Abstraction layer isolates our code

### Performance
- Streaming support ensures low latency
- Token counting for cost management
- Configurable timeout and retry logic

### Compatibility
- Support for all major providers
- Extensible for future providers
- Backward compatibility through traits

## Recommendations

### Immediate Actions
1. Create rig provider wrapper trait
2. Implement basic provider tests
3. Design script bindings interface
4. Document provider configuration

### Long-term Strategy
1. Build provider plugin system
2. Add candle local model support
3. Implement provider-specific optimizations
4. Create provider selection logic

## Conclusion

The combination of rig's production-ready provider abstraction with our custom agent hierarchy provides the best balance of:
- **Functionality**: Comprehensive provider support
- **Extensibility**: Custom providers and local models
- **Performance**: Streaming and async support
- **Maintainability**: Clean architecture and modular design

This approach allows us to leverage existing work while maintaining flexibility for rs-llmspell's unique requirements.