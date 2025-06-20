# Rs-LLMSpell Architecture Refinement TODO

## Overview
Comprehensive refinement of rs-llmspell architecture based on go-llms and Google ADK patterns, focusing on proper agent/tool/workflow hierarchy, hooks/events system, and built-in components.

## Multi-Step Strategy

### Phase 1: Research Foundation (🔍 Research)
- [x] **Task 1.1**: Deep study of go-llms architecture
  - [x] Study core concepts: BaseAgent, Agent, Tool, Workflow relationships
  - [x] Understand tool-wrapped agents pattern
  - [x] Analyze hooks and events system implementation
  - [x] Document component hierarchy patterns

- [x] **Task 1.2**: Study Google Agent Development Kit (ADK) 
  - [x] Core agent/tool/workflow concepts
  - [x] Hook and event patterns
  - [x] Built-in component strategies
  - [x] Orchestration patterns

- [x] **Task 1.3**: **CRITICAL** Research state management and agent handoff patterns
  - [x] Study go-llms state structure design and usage
  - [x] Understand agent-to-agent handoff without workflows
  - [x] Analyze state-driven execution vs message-driven execution
  - [x] Document state preservation and context passing patterns
  - [x] Understand state-based debugging and observability

- [x] **Task 1.4**: Research existing Rust patterns for similar systems
  - [x] Event systems (tokio, async-std patterns)
  - [x] Hook/plugin architectures
  - [x] Tool composition patterns
  - [x] State management patterns in Rust

### Phase 2: Analyze Current State (🔬 Analyze)
- [x] **Task 2.1**: Map current rs-llmspell concepts to go-llms/ADK
  - [x] Identify gaps in current architecture
  - [x] Map existing traits to new hierarchy
  - [x] Identify breaking changes needed

- [x] **Task 2.2**: Analyze scripting interface implications
  - [x] How BaseAgent/Agent/Tool hierarchy exposes to Lua/JS
  - [x] Hook registration patterns in scripts
  - [x] Event handling in scripting context

### Phase 3: Synthesize Core Architecture (⚡ Synthesize)
- [x] **Task 3.1**: Design BaseAgent/Agent/Tool/Workflow hierarchy
  - [x] BaseAgent trait - fundamental tool-handling capabilities
  - [x] Agent trait - LLM wrapper with specialized prompts
  - [x] Tool trait - callable functions for LLMs
  - [x] Workflow trait - deterministic agent types
  - [x] Tool-wrapped agent pattern

- [x] **Task 3.2**: Design hooks and events system
  - [x] Hook points: pre-llm, post-llm, pre-tool, post-tool, etc.
  - [x] Event emit/publish/subscribe system
  - [x] Built-in hooks for logging, metrics, debugging
  - [x] Script-accessible hook registration

### Phase 4: Research Implementation Patterns (🔍 Research)
- [x] **Task 4.1**: Built-in components research
  - [x] Identify 30-40 essential built-in tools
  - [x] Design built-in agent patterns
  - [x] Custom workflow definition patterns

- [x] **Task 4.2**: Composition and orchestration research
  - [x] Agent composition patterns
  - [x] Tool chaining strategies
  - [x] Workflow nesting capabilities

### Phase 5: Analyze Integration Points (🔬 Analyze)
- [x] **Task 5.1**: Bridge integration analysis
  - [x] How new hierarchy maps to bridge layer
  - [x] Script engine implications
  - [x] Testing strategy updates needed

- [x] **Task 5.2**: Performance and security analysis
  - [x] Hook overhead considerations
  - [x] Event system performance impact
  - [x] Security implications of tool-wrapped agents

- [x] **Task 5.3**: **CRITICAL** Scripting engine concurrency and async patterns research
  - [x] Lua threading limitations and workarounds
    - [x] mlua async support capabilities and limitations
    - [x] Lua coroutines vs true async patterns
    - [x] Cooperative scheduling implementation strategies
    - [x] Yield-based programming models for long operations
  - [x] JavaScript async patterns in embedded engines
    - [x] Promise implementation in Rust JS engines (boa, v8)
    - [x] Event loop integration with Tokio runtime
    - [x] async/await simulation patterns
    - [x] Worker thread alternatives for CPU-intensive tasks
  - [x] Cross-engine async pattern standardization
    - [x] Common async interface for Lua and JavaScript
    - [x] Promise/Future-like abstractions for scripts
    - [x] Error handling in async script contexts
    - [x] Resource cleanup in interrupted async operations
  - [x] Agent orchestration async patterns
    - [x] Parallel agent execution without true threading
    - [x] Tool execution scheduling and queuing
    - [x] Stream processing with cooperative yielding
    - [x] Hook system non-blocking execution
  - [x] Workflow engine async design
    - [x] Parallel workflow step execution strategies
    - [x] Sequential workflow with async steps
    - [x] Conditional workflows with async predicates
    - [x] Loop workflows with async conditions and bodies
  - [x] Performance and fairness considerations
    - [x] Script execution time slicing
    - [x] Resource allocation between concurrent scripts
    - [x] Memory management in long-running async operations
    - [x] Debugging and profiling async script execution

### Phase 5B: Research Existing Crate Ecosystem (🔍 Research)
- [x] **Task 5B.1**: **CRITICAL** LLM Provider Layer Crates Research - 2025-06-20T08:15:00-08:00
  - [x] rust-genai evaluation
    - [x] Review architecture and design patterns
      - Multi-provider abstraction with "common and ergonomic single API"
      - Native implementation without per-service SDKs
      - Trait-based adapter pattern with static dispatch
      - Supports OpenAI, Anthropic, Gemini, Ollama, Groq, xAI, DeepSeek, Cohere
    - [x] Analyze provider abstraction approach
      - Uses `Adapter` trait with static methods (no &self)
      - `AdapterKind` enum for provider identification
      - `AdapterDispatcher` for routing to specific implementations
      - Model name to provider mapping (e.g., "gpt" -> OpenAI, "claude" -> Anthropic)
      - **LIMITATION**: Fixed enum-based providers, not easily extensible for custom providers
    - [x] Check async/streaming support
      - Fully async with tokio runtime
      - Streaming support via futures::Stream trait
      - Event-source streaming for real-time responses
      - Inter-stream abstraction for normalized streaming across providers
      - Tool use streaming support (recent addition)
    - [x] Evaluate extensibility for custom providers
      - Custom endpoints via `ServiceTargetResolver`
      - Custom authentication via `AuthResolver`
      - Model mapping via `ModelMapper`
      - **LIMITATION**: Cannot add new provider types without modifying the crate
      - **LIMITATION**: AdapterKind is a fixed enum, not trait-based
      - Fallback to Ollama for unknown models provides some flexibility
    - [x] Test performance and overhead
      - Minimal dependencies: tokio, futures, reqwest, serde
      - No unsafe code (forbid unsafe_code lint)
      - Lightweight abstraction layer
      - Arc-based cloning for shared state
      - Comment mentions "overhead is minimal" for data cloning
  - [x] Alternative LLM crates comparison - 2025-06-20T08:30:00-08:00
    - [x] llm_api_crate evaluation
      - Architecture: Enum-based LLM abstraction with trait `Access`
      - Providers: OpenAI, Gemini, Anthropic (fixed enum)
      - **LIMITATION**: No streaming support
      - **LIMITATION**: Fixed provider set, not extensible
      - Python bindings via PyO3
      - Simple async interface but limited features
    - [x] langchain-rust capabilities and limitations
      - Full LangChain port to Rust with composability focus
      - Providers: OpenAI, Azure OpenAI, Ollama, Anthropic, MistralAI
      - Builder patterns and macro-heavy design
      - Supports agents, tools, chains, and vector stores
      - Document loaders (PDF, HTML, CSV, Git commits)
      - **STRENGTH**: Most feature-complete LangChain implementation
      - **COMPLEXITY**: Heavy framework with many abstractions
    - [x] llm-chain architecture review
      - Focused on prompt chaining and multi-step workflows
      - Supports cloud and local models via llm.rs
      - Three chain types: Sequential, Map-reduce, Conversational
      - Tool integration (Bash, Python, web search)
      - **STRENGTH**: Good for complex multi-step workflows
      - **LIMITATION**: Less provider coverage than others
    - [x] async-openai for OpenAI-specific needs
      - OpenAI-specific with Azure OpenAI support
      - Full SSE streaming support
      - Trait-based Config for extensibility
      - BYOT (Bring Your Own Types) feature
      - Exponential backoff retry mechanism
      - **STRENGTH**: Best for OpenAI-specific applications
      - **LIMITATION**: Single provider focus
    - [x] rllm evaluation
      - Architecture: Thin wrapper around "llm" crate v1.2.6
      - Providers: OpenAI, Anthropic, Ollama, DeepSeek, xAI, Phind, Groq, Google
      - Builder pattern with LLMBuilder for configuration
      - Two main traits: ChatProvider and CompletionProvider
      - **FEATURES**: Multi-step chains, prompt templates, parallel evaluation
      - **FEATURES**: Function calling, vision, reasoning, structured output
      - **FEATURES**: Speech-to-text transcription support
      - **LIMITATION**: Streaming disabled in all examples (.stream(false))
      - **DESIGN**: Feature flags for conditional compilation
      - Multi-backend registry for managing different providers
      - **NOTE**: Wraps another crate "llm" which appears to be by same author
    - [x] rig evaluation
      - Architecture: Trait-based provider abstraction with companion crates
      - Core traits: CompletionModel, EmbeddingModel, VectorStoreIndex
      - Providers: OpenAI, Anthropic, Gemini, xAI, Perplexity, Cohere, DeepSeek, and more
      - **STRENGTH**: Full streaming support with StreamingCompletion trait
      - **STRENGTH**: Modular design - each provider/vector store in separate crate
      - **STRENGTH**: High-level Agent abstraction for RAG and tools
      - **STRENGTH**: Extensive vector store integrations (MongoDB, Neo4j, LanceDB, etc.)
      - **EXTENSIBILITY**: Implement CompletionModel trait for custom providers
      - **FEATURES**: Tool calling, embeddings, RAG, multi-agent support
      - **FEATURES**: Audio generation, image generation, transcription
      - **DESIGN**: Clean separation between completion and embedding models
      - **DESIGN**: Builder pattern for agents and requests
      - Async-first with tokio runtime
    - [x] candle for local model support
      - Architecture: Minimalist ML framework focused on serverless inference
      - Core goal: Remove Python from production, create lightweight binaries
      - **BACKENDS**: CPU (with MKL/Accelerate), CUDA, Metal, WASM
      - **MODELS**: LLaMA, Mistral, Mixtral, Gemma, Phi, Falcon, Whisper, etc.
      - **QUANTIZATION**: Supports GGML/GGUF formats like llama.cpp
      - **STRENGTH**: PyTorch-like syntax, easy to use
      - **STRENGTH**: WASM support for browser-based inference
      - **STRENGTH**: Extensive model support with examples
      - **INTEGRATION**: Could serve as local model backend for rs-llmspell
      - **DESIGN**: Modular crates (core, nn, transformers, examples)
      - **PERFORMANCE**: Optimized for small binary size and fast inference
      - Maintained by Hugging Face team
    - [x] **COMPARISON SUMMARY**:
      - **rust-genai**: Best multi-provider abstraction, good streaming, limited extensibility
      - **llm_api_crate**: Simplest API, no streaming, very limited
      - **langchain-rust**: Most features, heavy framework, good for complex apps
      - **llm-chain**: Good for workflows/chains, moderate complexity
      - **async-openai**: Best for OpenAI-only apps, excellent streaming
      - **rllm**: Good multi-provider support, rich features, unclear streaming status
      - **rig**: Most extensible, excellent streaming, modular architecture, best for production
      - **candle**: Best for local model inference, WASM support, no API provider abstraction
  - [x] Provider abstraction requirements - 2025-06-20T09:00:00-08:00
    - [x] Multi-provider support patterns
      - **FINDING**: Two main approaches - enum-based (rust-genai, llm_api_crate) vs trait-based (rig)
      - **RECOMMENDATION**: Use trait-based approach like rig for extensibility
      - **REQUIREMENT**: Support dynamic provider registration
      - **REQUIREMENT**: Allow custom provider implementations
      - **PATTERN**: Separate crates for each provider (like rig's modular design)
    - [x] Streaming response handling
      - **FINDING**: Critical feature - most modern crates support it
      - **BEST PRACTICE**: rig's StreamingCompletion trait pattern
      - **REQUIREMENT**: Unified streaming interface across providers
      - **REQUIREMENT**: Support both streaming and non-streaming modes
      - **CONSIDERATION**: Handle provider-specific streaming formats (SSE, WebSocket, etc.)
    - [x] Token counting and rate limiting
      - **FINDING**: Often overlooked but important for production
      - **REQUIREMENT**: Provider-agnostic token counting interface
      - **REQUIREMENT**: Rate limiting with exponential backoff
      - **CONSIDERATION**: Different tokenizers per provider
      - **BEST PRACTICE**: async-openai's retry mechanism
    - [x] Error handling and retries
      - **FINDING**: Most crates use custom error types with thiserror
      - **REQUIREMENT**: Unified error type that can wrap provider-specific errors
      - **REQUIREMENT**: Automatic retry with exponential backoff
      - **BEST PRACTICE**: async-openai's approach with configurable retry
      - **CONSIDERATION**: Different error types per provider (rate limits, auth, network)
    - [x] Authentication and configuration
      - **FINDING**: Various approaches - env vars, builders, config structs
      - **REQUIREMENT**: Flexible auth (API keys, OAuth, custom headers)
      - **REQUIREMENT**: Per-provider configuration with defaults
      - **BEST PRACTICE**: rig's Config trait for extensible configuration
      - **PATTERN**: rust-genai's AuthResolver for dynamic auth
  - [x] Integration feasibility analysis - 2025-06-20T09:10:00-08:00
    - [x] Compatibility with BaseAgent/Agent design
      - **FINDING**: rig's Agent abstraction aligns well with go-llms BaseAgent concept
      - **COMPATIBILITY**: rig's CompletionModel trait maps to LLM provider interface
      - **INTEGRATION**: Can wrap rig agents as Tools for tool-wrapped agent pattern
      - **CHALLENGE**: Need to bridge rig's trait-based design with rs-llmspell's hierarchy
    - [x] Bridge pattern implementation options
      - **OPTION 1**: Wrap rig directly - least work, most features
      - **OPTION 2**: Create custom trait inspired by rig - more control, more work
      - **OPTION 3**: Hybrid - use rig for providers, custom for agent hierarchy
      - **RECOMMENDATION**: Option 3 - leverage rig's providers with custom agents
    - [x] Custom provider extension points
      - **REQUIREMENT**: Allow users to implement custom CompletionModel trait
      - **REQUIREMENT**: Support local models via candle integration
      - **PATTERN**: Provider registry for dynamic provider loading
      - **CONSIDERATION**: Plugin system for community providers

- [x] **Task 5B.2**: Scripting Engine Crates Evaluation - 2025-06-20T09:30:00-08:00
  - [x] Lua embedding options
    - [x] mlua features and limitations review
      - **FEATURES**: Async/await support via coroutines
      - **FEATURES**: Multiple Lua versions (5.1-5.4, LuaJIT, Luau)
      - **FEATURES**: Module and standalone modes
      - **SAFETY**: Not absolute - contains significant unsafe code
      - **THREADING**: Optional Send + Sync with "send" feature
      - **ASYNC**: Works with any executor (Tokio, async-std)
      - **LIMITATION**: Cannot guarantee complete safety
    - [x] rlua comparison for safety guarantees
      - **STATUS**: Deprecated in favor of mlua
      - **CURRENT**: Now a thin wrapper around mlua
      - **MIGRATION**: Provides compatibility traits
      - **RECOMMENDATION**: Use mlua directly for new projects
    - [x] lua-sys for low-level control needs
      - **mlua-sys**: Raw FFI bindings used by mlua
      - **PURPOSE**: Direct Lua C API access
      - **SAFETY**: Requires careful unsafe handling
      - **USE CASE**: Only if mlua abstractions insufficient
    - [x] Performance benchmarks and memory usage
      - **OVERHEAD**: Safety mechanisms add some overhead
      - **OPTIMIZATION**: Feature flags to reduce dependencies
      - **MEMORY**: Efficient coroutine-based async
  - [x] JavaScript engine alternatives
    - [x] boa maturity and compliance assessment
      - **STATUS**: Experimental but progressing
      - **COMPLIANCE**: 90% ECMAScript spec compliance
      - **FEATURES**: Module support, single-threaded
      - **STRENGTHS**: Pure Rust, memory safe, embeddable
      - **LIMITATIONS**: Still experimental, not production-ready
    - [x] v8 rust bindings complexity analysis
      - **rusty_v8**: Now stable (v129.0.0+)
      - **COMPLEXITY**: 600K+ lines C++, 30min compile
      - **BUILD**: Complex (gn + ninja), but automated via cargo
      - **FEATURES**: Full V8 API, WebAssembly, Inspector, Fast API
      - **CHALLENGES**: Scopes, isolates, memory management
    - [x] quickjs-rs for lightweight embedding
      - **SIZE**: 210 KiB for hello world
      - **PERFORMANCE**: <300μs runtime lifecycle
      - **ALTERNATIVES**: rquickjs more feature-complete
      - **FEATURES**: ES2020, async/await, modules, bytecode
      - **LIMITATION**: Single-threaded (mutex-locked)
    - [x] deno_core for modern JS features
      - **FOUNDATION**: Built on rusty_v8
      - **FEATURES**: TypeScript, JSX, web standards
      - **SECURITY**: Permission system, sandboxing
      - **USE CASE**: Custom JS/TS runtimes
      - **PRODUCTION**: Stable and widely used
  - [x] Cross-language considerations - 2025-06-20T10:15:00-08:00
    - [x] Unified value conversion strategies
      - ScriptValue enum as common type representation
      - Bidirectional conversion traits for mlua and JavaScript engines
      - Function proxy pattern for cross-language callbacks
      - Promise/Coroutine interop for async operations
    - [x] Shared memory management approaches
      - Arc/Weak reference counting for shared objects
      - CrossLangObject wrapper with vtable for method dispatch
      - Coordinated GC between Lua and JavaScript runtimes
      - Lazy conversion to minimize overhead
    - [x] Consistent error handling patterns
      - Unified ScriptError type with language-specific variants
      - ErrorContext preservation with stack traces
      - Cross-boundary error propagation
      - Cause chain tracking for debugging

- [x] **Task 5B.3**: Workflow and State Management Crates - 2025-06-20T10:45:00-08:00
  - [x] Workflow engine crates
    - [x] temporal-sdk-rust capabilities
      - Production-ready distributed workflow orchestration
      - Requires external Temporal server infrastructure
      - **LIMITATION**: Too heavyweight for embedded scripting
      - Good patterns but not directly usable for rs-llmspell
    - [x] flowrs for lightweight workflows
      - Early development (0.1.x), lightweight and embeddable
      - Builder pattern with async native design
      - **POTENTIAL**: Could be wrapped but needs enhancement
      - Limited features: no persistence, signals, or queries
    - [x] state-machine crates comparison
      - **sm**: Compile-time safety, zero overhead, typestate pattern
      - **statig**: Hierarchical states, async support, event-driven
      - **finny**: Actor-based, queue management, builder API
      - **RECOMMENDATION**: statig for async hierarchical agent states
  - [x] State management solutions
    - [x] sled for embedded persistence
      - Lock-free, log-structured design, beta status (0.34.x)
      - Fast reads, ACID transactions, watch subscriptions
      - **PROS**: Rust native, embedded, good performance
      - **CONS**: Beta status, memory intensive, higher space usage
    - [x] rocksdb for high-performance needs
      - Production proven, stable (0.21.x), battle-tested
      - Column families, compaction, snapshots
      - **PROS**: Excellent for large datasets, stable API
      - **CONS**: C++ dependency, large binary size
    - [x] async-std storage patterns
      - Actor-based state management patterns
      - Channel-based coordination strategies
      - Not a storage solution but patterns for async state
  - [x] Event system crates
    - [x] tokio-stream for async event streams
      - Part of tokio ecosystem, async-first design
      - Stream combinators, broadcast channels
      - **PROS**: Perfect tokio integration, zero-cost abstractions
      - **CONS**: Tokio lock-in, learning curve
    - [x] crossbeam-channel for multi-producer patterns
      - Sync channels for thread-to-thread communication
      - Multiple channel types, select! macro support
      - **PROS**: Excellent performance, runtime agnostic
      - **CONS**: Not async native, needs adapter
    - [x] event-emitter-rs for pub/sub models
      - Simple Node.js style event emitter
      - Limited documentation and features
      - Less commonly used than alternatives
  - [x] **RECOMMENDATIONS**:
    - **Workflow**: Custom engine inspired by flowrs patterns
    - **State**: sled (dev) / rocksdb (prod) behind trait abstraction
    - **State Machines**: statig for hierarchical async states
    - **Events**: tokio-stream + crossbeam hybrid approach
  - [x] Created comprehensive research document at /docs/technical/workflow_state_crates_research.md

- [ ] **Task 5B.4**: Supporting Infrastructure Crates
  - [ ] Serialization and data handling
    - [ ] serde ecosystem integration
    - [ ] rkyv for zero-copy deserialization
    - [ ] bincode for efficient binary formats
  - [ ] Testing and mocking frameworks
    - [ ] mockall for trait mocking
    - [ ] proptest for property-based testing
    - [ ] criterion for benchmarking
  - [ ] Logging and observability
    - [ ] tracing ecosystem integration
    - [ ] metrics-rs for performance monitoring
    - [ ] opentelemetry-rust for distributed tracing

- [ ] **Task 5B.1b**: **LLM Provider Layer Decision Summary**
  - [ ] Based on research, recommended approach:
    - [ ] Use **rig** as the foundation for LLM provider abstraction
    - [ ] Extend with custom BaseAgent/Agent/Tool hierarchy on top
    - [ ] Integrate **candle** for local model support
    - [ ] Key advantages: production-ready, extensible, streaming support, modular design
    - [ ] Implementation strategy: Hybrid approach leveraging rig's providers

- [ ] **Task 5B.5**: Build vs Buy Decision Matrix
  - [ ] Core components analysis
    - [ ] What must be built custom (bridge layer, agent hierarchy)
    - [ ] What can be wrapped (LLM providers, script engines)
    - [ ] What can be used as-is (serialization, logging)
  - [ ] Integration complexity assessment
    - [ ] API compatibility requirements
    - [ ] Performance overhead considerations
    - [ ] Maintenance burden evaluation
  - [ ] Dependency risk analysis
    - [ ] Crate maturity and maintenance status
    - [ ] License compatibility checks
    - [ ] Community support evaluation
  - [ ] Final recommendations document
    - [ ] Recommended crates for each component
    - [ ] Integration patterns and best practices
    - [ ] Risk mitigation strategies

### Phase 6: Synthesize Complete System (⚡ Synthesize)
- [ ] **Task 6.1**: Complete component ecosystem design
  - [ ] Full trait hierarchy with relationships
  - [ ] Built-in component library structure
  - [ ] Hook/event system integration
  - [ ] Composition and orchestration patterns

- [ ] **Task 6.2**: Script interface design
  - [ ] Lua/JavaScript API for new concepts
  - [ ] Hook registration in scripts
  - [ ] Event handling in scripts
  - [ ] Built-in component access
  - [ ] **ASYNC PATTERNS**: Promise/Future-like abstractions for scripts
  - [ ] **ASYNC PATTERNS**: Cooperative scheduling API design
  - [ ] **ASYNC PATTERNS**: Cross-engine async compatibility layer

### Phase 7: Collate Architecture (📋 Collate)
- [ ] **Task 7.1**: Organize all concepts into coherent architecture
  - [ ] Resolve conflicts between concepts
  - [ ] Ensure consistent terminology
  - [ ] Validate against go-llms/ADK patterns
  - [ ] Create comprehensive component map

- [ ] **Task 7.2**: Validate against use cases
  - [ ] Simple tool execution scenarios
  - [ ] Complex multi-agent workflows
  - [ ] Hook/event driven automation
  - [ ] Built-in component usage

### Phase 8: Update Architecture Document (📝 Update)
- [ ] **Task 8.1**: Update core concepts and philosophy
  - [ ] Revise core philosophy section
  - [ ] Update architecture overview
  - [ ] Refine bridge-first design section

- [ ] **Task 8.2**: Update component architecture sections
  - [ ] BaseAgent/Agent/Tool/Workflow hierarchy
  - [ ] Hooks and events system
  - [ ] Built-in components
  - [ ] Tool-wrapped agents

- [ ] **Task 8.3**: Update directory structure
  - [ ] Add built-in components crates
  - [ ] Add hooks/events system crates
  - [ ] Update testing strategy for new concepts

- [ ] **Task 8.4**: Update examples section
  - [ ] Show BaseAgent usage in scripts
  - [ ] Demonstrate hook registration
  - [ ] Event-driven workflow examples
  - [ ] Built-in tool usage examples
  - [ ] **ASYNC PATTERNS**: Parallel agent execution examples
  - [ ] **ASYNC PATTERNS**: Long-running tool execution with yielding
  - [ ] **ASYNC PATTERNS**: Stream processing examples
  - [ ] **ASYNC PATTERNS**: Error handling in async contexts

### Phase 9: Research Advanced Patterns (🔍 Research)
- [ ] **Task 9.1**: Advanced orchestration patterns
  - [ ] Multi-agent collaboration patterns
  - [ ] Dynamic workflow composition
  - [ ] Real-time event-driven automation

- [ ] **Task 9.2**: Performance optimization patterns
  - [ ] Efficient hook execution
  - [ ] Event system optimization
  - [ ] Tool execution pooling

### Phase 10: Analyze Testing Strategy (🔬 Analyze)
- [ ] **Task 10.1**: Testing strategy for new concepts
  - [ ] Hook system testing patterns
  - [ ] Event system testing
  - [ ] Tool-wrapped agent testing
  - [ ] Built-in component testing
  - [ ] **ASYNC PATTERNS**: Async script execution testing
  - [ ] **ASYNC PATTERNS**: Cooperative scheduling test scenarios
  - [ ] **ASYNC PATTERNS**: Resource cleanup in interrupted operations
  - [ ] **ASYNC PATTERNS**: Performance testing for async patterns

- [ ] **Task 10.2**: Cross-engine compatibility analysis
  - [ ] Hook registration across engines
  - [ ] Event handling differences
  - [ ] Tool execution consistency
  - [ ] **ASYNC PATTERNS**: Async pattern compatibility across Lua/JS
  - [ ] **ASYNC PATTERNS**: Promise/Future behavior consistency
  - [ ] **ASYNC PATTERNS**: Error propagation in async contexts

### Phase 11: Synthesize Final Architecture (⚡ Synthesize)
- [ ] **Task 11.1**: Complete architecture integration
  - [ ] Finalize all trait relationships
  - [ ] Complete hook/event integration
  - [ ] Finalize built-in component strategy
  - [ ] **ASYNC PATTERNS**: Integrate async patterns into core architecture
  - [ ] **ASYNC PATTERNS**: Finalize cooperative scheduling design
  - [ ] **ASYNC PATTERNS**: Complete async error handling strategy

- [ ] **Task 11.2**: Future evolution strategy
  - [ ] Extension points for new concepts
  - [ ] Backward compatibility strategy
  - [ ] Migration path from current design

### Phase 12: Collate Final Documentation (📋 Collate)
- [ ] **Task 12.1**: Final documentation review
  - [ ] Ensure all concepts are covered
  - [ ] Validate examples work with new design
  - [ ] Check consistency across sections

- [ ] **Task 12.2**: Create implementation roadmap
  - [ ] Priority order for implementation
  - [ ] Breaking change migration strategy
  - [ ] Testing milestones

### Phase 13: Final Update (📝 Update)
- [ ] **Task 13.1**: Complete architecture.md update
  - [ ] All sections reflect new architecture
  - [ ] Examples demonstrate new concepts
  - [ ] Testing strategy updated

- [ ] **Task 13.2**: Supporting documentation
  - [ ] Update README.md if needed
  - [ ] Create migration guide outline
  - [ ] Update TODO-DONE.md when complete

## Key Concepts to Address

### Component Hierarchy
- **BaseAgent**: Fundamental agent with tool-handling capabilities
- **Agent**: LLM wrapper with specialized prompts, uses multiple tools
- **Tool**: Special functions LLMs can call, runnable independently
- **Workflow**: Deterministic agents (sequential, parallel, conditional, loop)
- **Tool-wrapped Agent**: Agents wrapped as tools for other agents

### Built-in Components
- **30-40 Built-in Tools**: Ready-to-use tool library
- **Built-in Agents**: Common agent patterns
- **Custom Workflows**: User-defined workflow types

### Hooks and Events
- **Hook Points**: pre-llm, post-llm, pre-tool, post-tool, pre-workflow, post-workflow
- **Event System**: emit/publish/subscribe for orchestration events
- **Built-in Hooks**: logging, metrics, debugging, tracing
- **Script Hooks**: User-defined hooks in Lua/JavaScript

### Integration Points
- **Bridge Layer**: How new concepts map to script engines
- **Testing Strategy**: Comprehensive testing for new concepts
- **Performance**: Efficient hook/event execution
- **Security**: Safe tool-wrapped agent execution
- **Async Patterns**: Concurrency and parallelism in single-threaded scripting engines
- **Cooperative Scheduling**: Non-blocking execution patterns for long-running operations
- **Cross-Engine Compatibility**: Consistent async behavior across Lua and JavaScript

## Success Criteria
- [ ] Architecture reflects go-llms/ADK patterns
- [ ] Clear BaseAgent/Agent/Tool/Workflow hierarchy
- [ ] Comprehensive hooks and events system
- [ ] Built-in component strategy defined
- [ ] Tool-wrapped agent pattern implemented
- [ ] Script interface supports all new concepts
- [ ] **ASYNC PATTERNS**: Cooperative scheduling and async patterns implemented
- [ ] **ASYNC PATTERNS**: Cross-engine async compatibility achieved
- [ ] **ASYNC PATTERNS**: Non-blocking execution for long-running operations
- [ ] Testing strategy covers all components
- [ ] Examples demonstrate real-world usage
- [ ] Performance and security considerations addressed
- [ ] Migration path from current design clear

## Notes
- Reference: https://github.com/lexlapax/go-llms/
- Reference: https://google.github.io/adk-docs/
- Focus on scriptable interface with Rust performance
- Maintain bridge-first philosophy
- Ensure testing infrastructure covers new concepts
- Keep examples practical and realistic