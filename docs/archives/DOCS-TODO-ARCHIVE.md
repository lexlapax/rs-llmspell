# Rs-LLMSpell Architecture Refinement TODO

## Overview
Comprehensive refinement of rs-llmspell architecture based on go-llms and Google ADK patterns, focusing on proper agent/tool/workflow hierarchy, hooks/events system, and built-in components.

**Status**: Phase 13 COMPLETED 2025-06-26T17:30:00-08:00  
# TODO-DONE: rs-llmspell - Completed Tasks

This file tracks completed tasks for the rs-llmspell multi-engine architecture library

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

- [x] **Task 5B.4**: Supporting Infrastructure Crates - 2025-06-20T11:00:00-08:00
  - [x] Serialization and data handling
    - [x] serde ecosystem integration
      - Industry standard, de facto serialization framework
      - Format agnostic (JSON, YAML, TOML, MessagePack, etc.)
      - **ESSENTIAL** for LLM JSON communication and config parsing
    - [x] rkyv for zero-copy deserialization
      - 10-100x faster than serde for some use cases
      - True zero-copy access without parsing
      - **USE CASE**: State snapshots, high-performance caching
      - **LIMITATION**: Binary format only, more complex API
    - [x] bincode for efficient binary formats
      - Simple serde-based binary serialization
      - Fast and compact, deterministic output
      - **USE CASE**: Internal message passing between components
  - [x] Testing and mocking frameworks
    - [x] mockall for trait mocking
      - Most feature-rich Rust mocking library
      - #[automock] for automatic mock generation
      - Supports async, static methods, complex expectations
      - **PRIMARY CHOICE** for unit testing with mocks
    - [x] proptest for property-based testing
      - Automatically discovers edge cases
      - Shrinking to minimal failing inputs
      - **USE CASE**: Test invariants, serialization roundtrips
    - [x] criterion for benchmarking
      - Statistical rigor with confidence intervals
      - Beautiful HTML reports with graphs
      - **USE CASE**: Performance tracking over time
  - [x] Logging and observability
    - [x] tracing ecosystem integration
      - Structured logging with spans for async context
      - Zero overhead when disabled
      - **SUPERIOR** to log crate for async applications
      - Tracks context across async boundaries
    - [x] metrics-rs for performance monitoring
      - Lightweight metrics facade
      - Multiple backends (Prometheus, StatsD)
      - **USE CASE**: Runtime performance metrics
    - [x] opentelemetry-rust for distributed tracing
      - Full observability stack (traces, metrics, logs)
      - Industry standard protocol
      - **USE CASE**: Production distributed systems
  - [x] **RECOMMENDATIONS**:
    - **Serialization**: serde (general) + rkyv (performance) + bincode (internal)
    - **Testing**: mockall + proptest + criterion comprehensive stack
    - **Observability**: tracing (mandatory) + metrics (production) + opentelemetry (optional)
  - [x] Created comprehensive research document at /docs/technical/supporting_infrastructure_crates_research.md

- [x] **Task 5B.1b**: **LLM Provider Layer Decision Summary** - 2025-06-20T11:15:00-08:00
  - [x] Based on research, recommended approach:
    - [x] Use **rig** as the foundation for LLM provider abstraction
    - [x] Extend with custom BaseAgent/Agent/Tool hierarchy on top
    - [x] Integrate **candle** for local model support
    - [x] Key advantages: production-ready, extensible, streaming support, modular design
    - [x] Implementation strategy: Hybrid approach leveraging rig's providers
  - [x] Created decision document at /docs/technical/llm_provider_decision_summary.md

- [x] **Task 5B.5**: Build vs Buy Decision Matrix - 2025-06-20T11:30:00-08:00
  - [x] Core components analysis
    - [x] What must be built custom (bridge layer, agent hierarchy)
      - Bridge layer: 100% custom, core differentiator
      - Agent hierarchy: 100% custom, implements go-llms patterns
      - Script API: 100% custom, unique experience
    - [x] What can be wrapped (LLM providers, script engines)
      - LLM providers: Wrap rig (70% reuse, 30% custom)
      - State storage: Wrap sled/rocksdb behind traits
      - Local models: Wrap candle for inference
    - [x] What can be used as-is (serialization, logging)
      - Serialization: serde, rkyv, bincode (95% as-is)
      - Testing: mockall, proptest, criterion
      - Observability: tracing, metrics-rs
  - [x] Integration complexity assessment
    - [x] API compatibility requirements
      - Low: serde, tracing, standard infrastructure
      - Medium: mlua async, state storage, rig wrapping
      - High: JS engine, cross-language async, events
    - [x] Performance overhead considerations
      - Acceptable: serde, tracing, storage abstractions
      - Critical: Bridge conversions, event routing, streaming
      - Optimization: rkyv snapshots, batch processing
    - [x] Maintenance burden evaluation
      - Low risk: serde, tokio, tracing (stable)
      - Medium risk: rig, mlua, statig (active)
      - High risk: quickjs, experimental crates
  - [x] Dependency risk analysis
    - [x] Crate maturity and maintenance status
      - Industry standard: serde, tokio ecosystem
      - Production ready: rocksdb, criterion
      - Growing: rig, statig, rkyv
    - [x] License compatibility checks
      - All recommended crates MIT/Apache-2.0 compatible
    - [x] Community support evaluation
      - Strong: serde, tokio, tracing communities
      - Moderate: rig, mlua communities
      - Growing: candle (Hugging Face support)
  - [x] Final recommendations document
    - [x] Recommended crates for each component
      - Decision matrix with BUILD/WRAP/USE classifications
      - 14 components analyzed with clear decisions
    - [x] Integration patterns and best practices
      - Abstraction layers for flexibility
      - Feature flags for optional dependencies
      - Phased implementation roadmap
    - [x] Risk mitigation strategies
      - Facade patterns, fallback options, regular updates
  - [x] Created comprehensive decision matrix at /docs/technical/build_vs_buy_decision_matrix.md

### Phase 6: Synthesize Complete System (⚡ Synthesize)
- [x] **Task 6.1**: Complete component ecosystem design - 2025-06-20T11:45:00-08:00
  - [x] Full trait hierarchy with relationships
    - BaseAgent as foundation with tool handling
    - Agent extends BaseAgent with LLM capabilities
    - Tool trait for LLM-callable functions
    - Workflow trait for deterministic patterns
    - Composition traits: Composable, Observable, Hookable, Scriptable
  - [x] Built-in component library structure
    - 40+ built-in tools across 9 categories
    - 6 agent templates (Chat, Research, Code, Data, Planner, Orchestrator)
    - 6 workflow types (Sequential, Parallel, Conditional, Loop, MapReduce, Pipeline)
  - [x] Hook/event system integration
    - 20+ hook points throughout lifecycle
    - Comprehensive event types and handlers
    - Built-in hooks for logging, metrics, tracing, rate limiting, caching
    - Hybrid EventBus using tokio + crossbeam
  - [x] Composition and orchestration patterns
    - Tool-wrapped agent pattern
    - Composite, Pipeline, and Hierarchical agents
    - Agent pools, mesh, and saga patterns
    - State handoff and synchronization
  - [x] Created comprehensive design at /docs/technical/component_ecosystem_design.md

- [x] **Task 6.2**: Script interface design - 2025-06-20T12:00:00-08:00
  - [x] Lua/JavaScript API for new concepts
    - Language-idiomatic APIs (tables for Lua, objects for JS)
    - Agent creation with builders and configuration objects
    - Tool definition with handlers and validation
    - Workflow definitions (sequential, conditional, parallel)
  - [x] Hook registration in scripts
    - Global and agent-specific hooks
    - Priority-based execution
    - Conditional hooks with filters
    - Middleware-style composition in JS
  - [x] Event handling in scripts
    - Event emitter patterns for both languages
    - Async event handlers
    - Pattern matching and wildcards
    - Event aggregation and replay
  - [x] Built-in component access
    - Organized tool library by category
    - Agent template instantiation
    - Dynamic loading and extension
    - Custom component registration
  - [x] **ASYNC PATTERNS**: Promise/Future-like abstractions for scripts
    - Lua Promise implementation with then/catch
    - Native JS promises and combinators
    - Unified error handling
  - [x] **ASYNC PATTERNS**: Cooperative scheduling API design
    - Lua coroutine-based scheduler
    - JS async generators with backpressure
    - Yield points for long operations
  - [x] **ASYNC PATTERNS**: Cross-engine async compatibility layer
    - Stream processing abstractions
    - Batch processing with flow control
    - Timeout and cancellation support
  - [x] Created comprehensive design at /docs/technical/script_interface_design.md

### Phase 7: Collate Architecture (📋 Collate)
- [x] **Task 7.1**: Organize all concepts into coherent architecture - 2025-06-20T12:15:00-08:00
  - [x] Layer architecture: Script → Bridge → Application → Infrastructure
  - [x] Component relationships and dependencies
  - [x] Architectural patterns: Bridge-first, State-first, Composable, Observable
  - [x] Critical paths and bottlenecks identification
  - [x] Created comprehensive architecture at /docs/technical/architecture.md

- [x] **Task 7.2**: Integration and testing strategy - 2025-06-20T12:30:00-08:00
  - [x] Component integration patterns
    - Trait-based boundaries with testing seams
    - Dependency injection for testability
    - Mock implementations for isolation
  - [x] Testing strategy across all layers
    - Unit tests for individual components
    - Integration tests for layer interactions
    - End-to-end tests for script scenarios
    - Property-based testing for invariants
  - [x] Performance testing framework
    - Benchmark suites for critical paths
    - Load testing for concurrent scenarios
    - Memory usage analysis
    - Profiling and optimization guidelines
  - [x] **ASYNC PATTERNS**: Testing async patterns
    - Cooperative scheduling verification
    - Cross-engine async behavior validation
    - Resource cleanup testing
    - Timeout and cancellation testing
  - [x] Created comprehensive testing strategy at /docs/technical/testing_strategy.md

### Phase 8: Analyze Implementation Plan (🔬 Analyze)
- [x] **Task 8.1**: Break down implementation into phases - 2025-06-20T12:45:00-08:00
  - [x] Phase 0: Foundation and scaffolding
    - Crate structure and workspace setup
    - Core traits and basic implementations
    - Bridge layer foundation
  - [x] Phase 1: Core components
    - BaseAgent and Agent implementations
    - Tool trait and basic built-in tools
    - Script engine integration (Lua first)
  - [x] Phase 2: Orchestration and workflows
    - Workflow trait and implementations
    - Tool-wrapped agent pattern
    - Hook and event system
  - [x] Phase 3: Advanced features
    - JavaScript engine integration
    - Built-in component library
    - Advanced orchestration patterns
  - [x] Phase 4: Production features
    - Security and sandboxing
    - Performance optimization
    - Observability and monitoring
  - [x] **ASYNC PATTERNS**: Implementation phases
    - Phase 1: Basic coroutine-based async for Lua
    - Phase 2: JavaScript async/await simulation
    - Phase 3: Cross-engine async compatibility
    - Phase 4: Advanced async patterns (streaming, backpressure)
  - [x] Created detailed implementation plan at /docs/technical/implementation_plan.md

- [x] **Task 8.2**: Risk analysis and mitigation - 2025-06-20T13:00:00-08:00
  - [x] Technical risks and mitigation strategies
    - Dependency risks and alternatives
    - Performance risks and optimization plans
    - Security risks and countermeasures
  - [x] Implementation risks and management
    - Complexity management through phases
    - Testing strategy to catch regressions
    - Documentation and knowledge sharing
  - [x] **ASYNC PATTERNS**: Async-specific risks
    - Deadlock prevention in cooperative scheduling
    - Memory leak prevention in long-running async operations
    - Cross-engine consistency maintenance
  - [x] Created risk analysis document at /docs/technical/risk_analysis.md

### Phase 9: Synthesize Final Vision (⚡ Synthesize)
- [x] **Task 9.1**: Create comprehensive architecture document - 2025-06-20T13:15:00-08:00
  - [x] Executive summary and vision
  - [x] Detailed component specifications
  - [x] Implementation guidelines and best practices
  - [x] Examples and usage patterns
  - [x] **ASYNC PATTERNS**: Comprehensive async patterns documentation
    - Cooperative scheduling patterns
    - Cross-engine async compatibility
    - Performance and fairness considerations
  - [x] Updated main architecture document at /docs/technical/architecture.md

- [x] **Task 9.2**: Validate against requirements - 2025-06-20T13:30:00-08:00
  - [x] Go-llms pattern compliance verification
    - BaseAgent/Agent/Tool/Workflow hierarchy ✓
    - Tool-wrapped agent pattern ✓
    - Hooks and events system ✓
    - State-based execution ✓
  - [x] Google ADK pattern alignment
    - Built-in component strategy ✓
    - Orchestration patterns ✓
    - Hook system design ✓
  - [x] Scriptable interface requirements
    - Multi-language support (Lua, JavaScript, Python) ✓
    - Consistent API across languages ✓
    - Performance optimizations ✓
  - [x] **ASYNC PATTERNS**: Async pattern validation
    - Cooperative scheduling ✓
    - Cross-engine compatibility ✓
    - Non-blocking execution ✓
  - [x] Created validation report at /docs/technical/requirements_validation.md

### Phase 10: Collate Documentation (📋 Collate)
- [x] **Task 10.1**: Organize technical documentation - 2025-06-20T13:45:00-08:00
  - [x] Architecture documents organization
  - [x] Implementation guides structuring
  - [x] API documentation preparation
  - [x] Examples and tutorials outline
  - [x] **ASYNC PATTERNS**: Async patterns documentation organization
  - [x] Created documentation index at /docs/technical/README.md

- [x] **Task 10.2**: Create quick reference materials - 2025-06-20T14:00:00-08:00
  - [x] Component hierarchy quick reference
  - [x] API quick reference for scripts
  - [x] Common patterns and recipes
  - [x] Troubleshooting guide framework
  - [x] **ASYNC PATTERNS**: Async patterns quick reference
  - [x] Created quick reference at /docs/technical/quick_reference.md

### Phase 11: Final Synthesis (⚡ Synthesize)
- [x] **Task 11.1**: Integration testing strategy - 2025-06-20T14:15:00-08:00
  - [x] Component integration test plans
  - [x] End-to-end scenario testing
  - [x] Performance benchmark definitions
  - [x] Security testing framework
  - [x] **ASYNC PATTERNS**: Async integration testing
  - [x] Updated testing strategy at /docs/technical/testing_strategy.md

- [x] **Task 11.2**: Final architecture validation - 2025-06-20T14:30:00-08:00
  - [x] Completeness check against original requirements
  - [x] Consistency verification across all documents
  - [x] Implementation feasibility assessment
  - [x] Performance and scalability analysis
  - [x] **ASYNC PATTERNS**: Final async patterns validation
  - [x] Created final validation report at /docs/technical/final_validation.md

### Phase 11B: Complete Final Synthesis (⚡ Synthesize)
- [x] **Task 11B.1**: Final architecture synthesis - 2025-06-20T14:45:00-08:00
  - [x] Integrate all research findings into coherent architecture
  - [x] Resolve any remaining conflicts or gaps
  - [x] Create definitive implementation roadmap
  - [x] **ASYNC PATTERNS**: Final async patterns integration
  - [x] Updated final architecture at /docs/technical/final_architecture_synthesis.md

- [x] **Task 11B.2**: Documentation completeness verification - 2025-06-20T15:00:00-08:00
  - [x] All required topics covered
  - [x] Cross-references and consistency maintained
  - [x] Implementation guidance sufficient
  - [x] Examples and use cases complete
  - [x] **ASYNC PATTERNS**: Async documentation completeness
  - [x] Created documentation completeness report

### Phase 11C: Create Standalone Guide (📋 Collate)
- [x] **Task 11C.1**: Comprehensive standalone architecture document - 2025-06-20T15:30:00-08:00
  - [x] Consolidate all architecture decisions into single document
  - [x] Include implementation details and examples
  - [x] Provide complete API reference
  - [x] Add troubleshooting and best practices
  - [x] **ASYNC PATTERNS**: Complete async patterns documentation
  - [x] Created standalone guide at /docs/technical/rs_llmspell_complete_guide.md

- [x] **Task 11C.2**: Validate standalone guide completeness - 2025-06-20T16:00:00-08:00
  - [x] Verify all concepts are self-contained
  - [x] Ensure no external references required
  - [x] Test examples and code snippets
  - [x] Review for clarity and completeness
  - [x] **ASYNC PATTERNS**: Async patterns standalone validation
  - [x] Created validation checklist

  # TODO-DONE: rs-llmspell - Completed Tasks

This file tracks completed tasks for the rs-llmspell multi-engine architecture library

**Last Updated**: 2025-06-26T16:45:00-08:00

## Phase 1-11: Research and Architecture Foundation (COMPLETED)
All tasks from Phase 1 through Phase 11 were completed between 2025-06-20T08:15:00-08:00 and 2025-06-20T18:15:00-08:00.

For detailed completion history of Phases 1-11, see previous TODO-DONE.md entries (historical record preserved).

## Phase 12: Collate Final Documentation (📋 Collate and ReEdit) - COMPLETED 2025-06-26T16:45:00-08:00

### Phase 12: Final Documentation Completion 
- [x] **Task 12.1**: Final documentation review - **COMPLETED** 2025-06-20T18:30:00-08:00

- [x] **Task 12.2**: **CRITICAL** Create Complete Standalone Architecture Document - **COMPLETED** 2025-06-20T22:00:00-08:00

- [x] **Task 12.3**: Manual Review of Final documentation - **COMPLETED** 2025-06-26T16:00:00-08:00
  - [x] Manual human review and correction lists - my prompt for this
    **now I'm going to ask you a series of questions about the architecture documented in @docs/rs-llmspell-complete-architecture.md . for each question, document the question in Section 12.3 of Phase 2. for each question, document the question under the Task/question 12.3.1 heading. for that question review the architecture document and find the answer and document the answer under - [ ] Answer: . if you do not find the answer, document it as such. if you figure out what needs to be done, document in - [ ] Todo: subtask. Once that's done, prompt me for another answer. To understand this read the TODO.md document and make sure you understand the section **Task 12.3** and what I'm saying here. think hard and reason. do what I'm asking in this prompt and come back and tell me that you understand.** 
  - [x] **Task/question 12.3.1** Does the architecture allow for agent handoff from one agent to another without resorting to workflows or resorting to the A2A protocol? - **COMPLETED** 2025-06-26T10:00:00-08:00
    - [x] Answer: Yes, the architecture now explicitly supports LLM-Driven Delegation (Agent Transfer). This is a first-class feature enabling dynamic, intelligent handoffs.
      1.  **`HandoffRequest`**: The `AgentOutput` struct contains an optional `handoff_request` field. An agent can return this to signal its intent to transfer control.
      2.  **`AgentRuntime`**: A dedicated runtime engine inspects agent outputs. If a `HandoffRequest` is found, the runtime manages the seamless transfer of control and state to the specified target agent.
      3.  **State Management**: The existing state management architecture ensures context is preserved during the handoff.
      4.  This pattern is documented in the "LLM-Driven Delegation (Agent Transfer)" section of the main architecture document.
    - [x] Todo: ~~Consider documenting these handoff patterns more explicitly in a dedicated section of the architecture document~~ **Done** 2025-06-26T10:00:00-08:00. Future work could include adding more complex handoff examples to the `Real-World Examples` section.
  - [x] **Task/question 12.3.2** Does the architecture allow for rs-llmspell to be used as a module in lua or in javascript to take advantage of tools, agents, workflows, events? - **COMPLETED** 2025-06-26T10:15:00-08:00
    - [x] Answer: Yes, the architecture is explicitly designed to support this through its **Library Mode**. Key features enabling this are:
      1.  **Dual Usage Paradigms**: The architecture document now clearly defines both "Embedded Mode" (running scripts within `rs-llmspell`) and "Library Mode" (importing `rs-llmspell` into existing applications).
      2.  **Stable C API**: The bridge layer is designed to expose a stable C API, which is the foundation for creating native modules.
      3.  **Native Module Support**: The plan includes creating native modules for Lua (as a LuaRock) and JavaScript (as an NPM package), allowing `require("llmspell")` and `require('@rs/llmspell')` respectively.
      4.  **Build System**: The build system is designed to produce these native modules for distribution.
    - [x] Todo: ~~Clarify in architecture doc that module support is future work, not current capability~~ **Done** 2025-06-26T10:15:00-08:00. The architecture now fully incorporates this capability.
  - [x] **Task/question 12.3.3** Is the architecture laid out in a modular way such that implementation can occur in verticals or layers across the top to bottom stacks as suggested in the MVP roadmap? - **COMPLETED** 2025-06-26T10:30:00-08:00
    - [x] Answer: Yes, the architecture is explicitly designed for modular, layered implementation. This is evident through several key design decisions:
      1.  **Four-Layer Architecture**: The document defines clear boundaries between the Script, Bridge, Application, and Infrastructure layers.
      2.  **Modular Crate Structure**: The system is broken into independent crates (e.g., `llmspell-core`, `llmspell-agents`, `llmspell-tools`), allowing features to be built and tested in isolation.
      3.  **Trait-Based Design**: Core functionality is defined by traits (`BaseAgent`, `Tool`, `Workflow`), allowing for different components to be implemented and added incrementally.
      4.  **Component Hierarchy**: The `BaseAgent` -> `Agent` -> `Tool` hierarchy allows for building foundational capabilities first and layering more complex components on top.
      5.  **Feature Flags**: The design supports feature flags, enabling the creation of minimal builds (MVPs) that can be expanded over time.
      6.  **Phased Roadmap**: The architecture document's implementation roadmap is structured in phases, directly supporting the layered MVP approach (e.g., Foundation -> Components -> Advanced Features).
    - [x] Todo: ~~The "Implementation Roadmap" section in the main architecture document should be fully detailed...~~ **Done** 2025-06-26T10:30:00-08:00. These items have been moved to and expanded in **Phase 13: Implementation Roadmap**.
  - [x] **Task/question 12.3.4** If we were to create a command line executable with a script argument, would the architecture be able to detect the script type (say lua, javascript) and instantiate the right engine and run the script? If not what changes to the architecture to support that? - **COMPLETED** 2025-06-26T11:00:00-08:00
    - [x] Answer: Yes, the architecture fully supports this. The `llmspell-cli` is designed to automatically detect the script type based on file extension (e.g., `.lua`, `.js`) and can also use shebangs for explicit engine selection. This is now documented in the "Quick Start Guide" and "Build System and Tooling" sections.
    - [x] Todo: ~~Ensure CLI implementation includes robust file extension detection...~~ **Done** 2025-06-26T11:00:00-08:00. The architecture document has been updated to reflect these capabilities.
  - [x] **Task/question 12.3.5** Does the architecture allow for constructs to run a script in a cron job perhaps as a long running/always running daemon using perhaps workflow loop agent + mcp server, or a2a server or a tcp listener tool etc? - **COMPLETED** 2025-06-26T11:15:00-08:00
    - [x] Answer: Yes, the architecture has been updated to explicitly support this. It now includes:
      1.  **Scheduler Component**: A first-class `Scheduler` for managing jobs with `Cron`, `Interval`, and `Event` triggers.
      2.  **Listener Tools**: `WebhookListenerTool` and `SocketListenerTool` have been added to the built-in tool catalog to enable external triggers.
      3.  **Daemon/Service Mode**: A `llmspell serve` command is specified for running the framework as a persistent service.
      4.  These additions are documented in the "Scheduling and Automation", "Built-in Tools Catalog", and "Deployment Strategies" sections.
    - [x] Todo: ~~Consider adding scheduled/timer-based workflow triggers...~~ **Done** 2025-06-26T11:15:00-08:00. The architecture now includes a full `Scheduler` component and listener tools.
  - [x] **Task/question 12.3.6** Does the framework allow me to easily create a new tool in a script or an app using rust? Or create a new tool to add to the library of builtin tools into rs-llmspell directly? - **COMPLETED** 2025-06-26T11:30:00-08:00
    - [x] Answer: Yes, the framework provides multiple ways to create custom tools:
      1. **Script-level tools**: Easy creation using `Tools.create()` in Lua/JavaScript with configuration object
      2. **Rust applications**: Implement the `Tool` trait with required methods (name, schema, execute_tool)
      3. **Built-in library**: Add tools directly to rs-llmspell by implementing in `llmspell-tools/src/category/`
      4. **Tool registration**: `Tools.register()` in scripts or `agent.register_tool()` in Rust
      5. **Tool composition**: Chain tools together with pipelines and sequential execution
      6. **Plugin system**: Dynamic tool loading for advanced extensibility
      The architecture fully supports tool creation at all levels from simple script functions to complex Rust implementations
    - [x] Todo: ~~Architecture has been updated with "Tool Development Architecture" section covering:~~ **Done** 2025-06-26T11:30:00-08:00
      - [x] ~~Tool creation patterns for script-level, native, and plugin approaches~~ **Done**
      - [x] ~~Tool trait architecture and best practices~~ **Done**
      - [x] ~~Tool template system and scaffolding architecture~~ **Done**
      - [x] ~~Simplified patterns via common templates (HTTP API, File Processor, etc.)~~ **Done**
  - [x] **Task/question 12.3.7** Does the framework allow me to easily create a new agent to add to the library of builtin agents into rs-llmspell directly? - **COMPLETED** 2025-06-26T11:45:00-08:00
    - [x] Answer: Yes, the framework provides clear mechanisms for adding built-in agents:
      1. **Location**: Create in `crates/builtin/src/agents/` directory
      2. **Templates**: Extend existing templates (Chat, Research, Code, Content Creator)
      3. **From scratch**: Implement BaseAgent and Agent traits directly
      4. **Registration**: Add to `register_builtin_agents()` in module
      5. **Factory pattern**: Use AgentTemplateFactory for dynamic creation
      6. **Auto-availability**: Once registered, automatically available in Lua/JS scripts
      Example: Extend ResearchAgent template for MarketResearchAgent with specialized tools and prompts
    - [x] Todo: ~~Architecture has been enhanced with comprehensive agent development patterns:~~ **Done** 2025-06-26T11:45:00-08:00
      - [x] ~~Agent template scaffolding via CLI commands (llmspell generate agent)~~ **Done**
      - [x] ~~Advanced template customization options and inheritance patterns~~ **Done**
      - [x] ~~Added 3 specialized agent templates (DataAnalyst, CustomerService, API Integration)~~ **Done**
      - [x] ~~Comprehensive agent testing framework with template-specific tests~~ **Done**
  - [x] **Task/question 12.3.8** Does the architecture security allow for configurable security for a script engine, say lua, where in one script we may want to allow io library access, and in another both io, os and a few other "insecure" libraries? Does it allow for templated security profiles like low, medium, high? If not what are the todos to change the architecture to allow that? - **COMPLETED** 2025-06-26T12:00:00-08:00
    - [x] Answer: Yes, the architecture now supports both configurable security AND pre-defined profiles:
      1. **Per-script security**: Each agent/script can have its own SecurityConfig
      2. **Library access control**: `stdlib_access` in LuaEngineConfig controls which Lua modules are allowed
      3. **Fine-grained controls**: Filesystem, network, system calls all configurable
      4. **Sandboxing**: Can be enabled/disabled per script via `script_sandbox_mode`
      5. **Resource limits**: Memory, CPU, execution time configurable
      6. **Pre-defined security profiles**: None/Low/Medium/High profiles with preset configurations
      7. **Custom profiles**: Configurable with specific library and access options
      8. **Per-script overrides**: Ability to override security profile for specific scripts with audit trail
    - [x] Todo: ~~Architecture has been comprehensively enhanced with security profile system:~~ **Done** 2025-06-26T12:00:00-08:00
      - [x] ~~Created pre-defined security profiles (None/Low/Medium/High/Custom) with Medium as default~~ **Done**
      - [x] ~~Added SecurityProfile enum with preset configurations and CustomSecurityProfile support~~ **Done**
      - [x] ~~Created profile builder/factory pattern with CustomSecurityProfileBuilder~~ **Done**
      - [x] ~~Added SecurityProfilePresets for common use cases (development, testing, production, data_analysis)~~ **Done**
      - [x] ~~Added per-script security override examples in configuration with audit trail~~ **Done**
      - [x] ~~Integrated profile-based library access control in Script Sandbox Security~~ **Done**
  - [x] **Task/question 12.3.9** Does the architecture spell out configuration for prompts, api keys, security profiles etc via configuration files like yaml or environment variables? If not what todos do we have to add to spell that out? - **COMPLETED** 2025-06-26T12:15:00-08:00
    - [x] Answer: Yes, the architecture has comprehensive configuration management:
      1. **File formats**: TOML (default), YAML, JSON all supported
      2. **Main config file**: `llmspell.toml` with complete schema documented
      3. **Environment variables**: `LLMSPELL_` prefix with `__` for nesting
      4. **API keys**: Via env vars (`OPENAI_API_KEY`) or encrypted storage
      5. **Hierarchical loading**: Default → Environment-specific → User → Env vars
      6. **Hot reload**: Configuration changes without restart
      7. **Security profiles**: Configurable via `[security]` and `[security.sandbox]` sections
      8. **Validation**: All configs validated with helpful error messages
      9. **Prompt templates**: Full `[prompts]` section with system/agent templates and variable interpolation
      10. **Template variables**: Environment variable integration and dynamic substitution
    - [x] Todo: ~~Architecture has been comprehensively enhanced with configuration management:~~ **Done** 2025-06-26T12:15:00-08:00
      - [x] ~~Added `[prompts]` section to config with system/agent prompt templates and variable interpolation~~ **Done**
      - [x] ~~Documented prompt template variables, filters, and Handlebars/Jinja2 syntax support~~ **Done**
      - [x] ~~Added comprehensive security profile configuration examples~~ **Done** (completed in 12.3.8)
      - [x] ~~Created extensive config generator/validator CLI toolset with migration support~~ **Done**
      - [x] ~~Added configuration migration tools with backup/restore and version management~~ **Done**
  - [x] **Task/question 12.3.10** Does the framework or architecture spell out standard library for methods in rs-llmspell for respective script engine to load e.g. for lua promise.lua, llm.lua or provider.lua, agent.lua etc, or are they all assumed to be available? Can the scripts load other lua modules? Same goes for javascript. If not what todos do we need to add to have the architecture spell that out? - **COMPLETED** 2025-06-26T12:30:00-08:00
    - [x] Answer: The architecture now fully documents module loading behavior:
      1. **Global APIs**: Agent, Tool, Tools, Workflow, etc. are pre-injected as globals
      2. **No module loading**: Cannot use `require()` or `import` for rs-llmspell APIs
      3. **Module loading by security profile**: None=All, Low=Whitelist, Medium/High=None
      4. **Sandboxed require/import**: Controlled module loading with verification when allowed
      5. **Library mode exception**: Full module access when used as native module
      6. **Code organization patterns**: Documented patterns for module-less development
    - [x] Todo: ~~Architecture has been comprehensively enhanced with module loading support:~~ **Done** 2025-06-26T12:30:00-08:00
      - [x] ~~Documented global API injection model with complete list of globals~~ **Done**
      - [x] ~~Added module loading behavior table by security profile~~ **Done**
      - [x] ~~Implemented controlled module loading with whitelist and verification~~ **Done**
      - [x] ~~Added custom module path configuration in ModuleLoadingConfig~~ **Done**
      - [x] ~~Created "Code Organization Patterns Without Modules" section~~ **Done**
      - [x] ~~Implemented sandboxed require() with security verification~~ **Done**
      - [x] ~~Added 6 code organization patterns with examples~~ **Done**
  - [x] **Task/question 12.3.11** Does the architecture spell out how to enable logging of different levels or debugging of framework/library itself, or logging debuging of scripts? does Log/Logging need to be a global level entity like Agent, Tools or workflows? how about Debug? are they the same thing or different things? - **COMPLETED** 2025-06-26T12:45:00-08:00
    - [x] Answer: The architecture now FULLY addresses logging and debugging:
      1. **Framework Logging**: SystemConfig has a `log_level` field accepting "trace", "debug", "info", "warn", "error"
      2. **Script Logging**: `Logger` IS a pre-injected global entity like Agent, Tools, Workflow
      3. **Configuration**: Log levels configured via TOML config file or LLMSPELL_SYSTEM__LOG_LEVEL env var
      4. **Debug Mode**: Separate concept - LuaEngineConfig and JavaScriptEngineConfig have `debug_mode` boolean
      5. **Observability**: Comprehensive ObservabilityManager with TracingManager for distributed tracing
      6. **Logging vs Debug**: Different concerns - logging for observability, debug for script development
    - [x] Todo: ~~Architecture has been comprehensively enhanced with logging support:~~ **Done** 2025-06-26T12:45:00-08:00
      - [x] ~~Document Logger global API methods (Logger.info(), Logger.debug(), Logger.error(), etc.)~~ **Done**
      - [x] ~~Clarify what `debug_mode` enables in script engines (debugger attachment, breakpoints, etc.)~~ **Done**
      - [x] ~~Add per-script log level configuration capability~~ **Done**
      - [x] ~~Document how to enable different log levels for framework vs scripts~~ **Done**
      - [x] ~~Add configuration examples showing logging setup in llmspell.toml~~ **Done**
      - [x] ~~Document environment variable options for runtime log level changes~~ **Done** 
  - [x] **Task/question 12.3.12** In the **embedded** mode, the vision is to allow to modes for the command line - runner mode that runs scripts, and repl mode to enter an interactive mode. Does the architecture allow for that? what's missing? - **COMPLETED** 2025-06-26T13:00:00-08:00
    - [x] Answer: The architecture now FULLY supports all command-line modes:
      1. **Runner Mode**: YES - `llmspell run <script>` with full Unix pipeline support
      2. **REPL Mode**: YES - `llmspell repl` with state persistence, tab completion, multi-line input
      3. **Serve Mode**: YES - `llmspell serve` for daemon/service mode with API endpoints
      4. **Eval Mode**: YES - `llmspell eval <expr>` for one-shot evaluation
      5. **Debug Mode**: YES - `llmspell debug <script>` for interactive debugging
    - [x] Todo: ~~Architecture has been comprehensively enhanced with CLI/REPL support:~~ **Done** 2025-06-26T13:00:00-08:00
      - [x] ~~Document interactive REPL mode architecture (`llmspell repl` or `llmspell interactive`)~~ **Done**
      - [x] ~~Define REPL features: multi-line input, history, tab completion, state persistence~~ **Done**
      - [x] ~~Specify how REPL maintains agent/tool state between commands~~ **Done**
      - [x] ~~Document `llmspell serve` command for daemon/service mode~~ **Done**
      - [x] ~~Add CLI usage patterns showing: run, repl, serve modes~~ **Done**
      - [x] ~~Define REPL-specific globals or commands (e.g., .help, .exit, .save, .load)~~ **Done** 
  - [x] **Task/question 12.3.13** Does the architecture allow for and say how the `llmspell` command can be run in a unix shell as a pipe component with stdin and stdout and stderr redirection etc just like a regular unix command , with or without args? - **COMPLETED** 2025-06-26T13:15:00-08:00
    - [x] Answer: Yes, the architecture fully documents Unix pipe support in multiple sections:
      1. **Script Execution Mode** (Build System and Tooling): Shows comprehensive Unix pipeline examples
      2. **Unix Pipeline Integration** (Quick Start Guide): Demonstrates practical pipe usage
      3. **Features documented**: 
         - stdin/stdout/stderr redirection: `llmspell process.lua < input.txt > output.txt 2> errors.log`
         - Pipe chaining: `echo "text" | llmspell analyze.lua | jq '.summary'`
         - Exit codes: `llmspell validate.lua && echo "passed" || echo "failed"`
         - Background execution: `llmspell long_task.lua &`
         - JSON data flow: `cat data.json | llmspell transform.js | jq`
         - Integration with Unix tools: curl, mail, jq, etc.
      4. **With/without args**: Both modes supported - can pipe data OR use --param arguments
    - [x] Todo: Architecture is complete for Unix pipe support - **Done** 2025-06-26T13:15:00-08:00
  - [x] **Task/question 12.3.14** Does the architecture have enough thought process given to allow the entire project to be cross-platform, with different os specific components modularly laid out in a nice architecture pattern? the os's i'm concerned with in priority order would be linux, macosx and windows. - **COMPLETED** 2025-06-26T13:30:00-08:00
    - [x] Answer: The architecture NOW has EXPLICIT and comprehensive cross-platform support:
      1. **Platform Bridge Philosophy**: Added to Bridge-First Design principle with platform abstraction traits
      2. **Platform-aware Configuration**: SystemConfig enhanced with PlatformConfig for OS-specific settings
      3. **Cross-platform Build System**: Platform-specific build configurations and scripts
      4. **Service Integration**: Platform-specific serve mode with SystemD/LaunchD/Windows Service support
      5. **Storage Path Handling**: Platform-aware file handling with proper directory resolution
      6. **Testing Matrix**: Comprehensive platform testing across Linux/macOS/Windows with CI/CD matrix
      7. **Development Guidelines**: Detailed cross-platform development best practices
    - [x] Todo: ~~All platform support requirements have been comprehensively addressed in the architecture:~~ **Done** 2025-06-26T13:30:00-08:00
      - [x] ~~Add explicit OS abstraction layer for platform-specific operations~~ **Done** - PlatformServices trait added
      - [x] ~~Document platform-specific path handling (PathBuf usage, directory separators)~~ **Done** - PlatformPaths implementation
      - [x] ~~Add conditional compilation examples (#[cfg(target_os = "windows")])~~ **Done** - Throughout architecture
      - [x] ~~Document Windows service vs Unix daemon differences for serve mode~~ **Done** - Platform-specific service integration
      - [x] ~~Add platform-specific configuration defaults (data directories, config paths)~~ **Done** - In PlatformConfig
      - [x] ~~Document IPC mechanism differences (Unix sockets vs Named Pipes)~~ **Done** - Implicit in service architecture
      - [x] ~~Add platform testing matrix to ensure Linux/macOS/Windows compatibility~~ **Done** - CIPlatformMatrix added 
  - [x] **Task/question 12.3.15** Based on the architecture, can you tell me component by component, what happens when I run a lua script using the command line runner - which component it hits first etc. assume that the script uses all globals Agents, tools, worklfows etc. What's missing in the architectural document for you not to be able to trace that? what components are missing or integration layers between components? - **COMPLETED** 2025-06-26T14:00:00-08:00
    - [x] Answer: **COMPLETED** - Full execution flow now documented in architecture. Complete component-by-component trace:
      1. **CLI Entry** (`main()`) → Command parsing via clap → Script type detection by extension
      2. **ScriptRuntime** creation → ComponentLifecycleManager initialization in phases:
         - Infrastructure (storage, config)
         - Providers (LLM connections)  
         - Core (registries, factories)
         - ScriptEngine (Lua/JS context)
         - Globals (API injection)
      3. **Bridge Layer** (ScriptEngineBridge trait) → Script execution → Result handling → Cleanup
      **Architecture Updated With:**
      - ScriptRuntime as central orchestrator (Architecture Overview section)
      - ComponentLifecycleManager with 5-phase initialization
      - CLI entry point implementation (Build System section)
      - ScriptEngineBridge trait for unified script interface (Bridge-First Design)
      - Complete execution flow sequence diagram
      - Execution flow testing methodology (Testing Strategy)
    - [x] Todo: **ALL COMPLETED** - Architecture fully documents execution flow from CLI to cleanup - **Done** 2025-06-26T14:00:00-08:00
  - [x] **Task/question 12.3.16** Based on the architecture, can you tell me component by component, what happens when I load llmspell as a lua module from an external lua runtime  - which component it hits first etc. assume that the script uses all globals Agents, tools, worklfows etc. What's missing in the architectural document for you not to be able to trace that? what components are missing or integration layers between components? what about an alternate scenario where I only want to run Tools ? - **COMPLETED** 2025-06-26T15:00:00-08:00
    - [x] Answer: **COMPLETED** - Full library mode flow documented through component reuse architecture:
      1. **External Entry**: `require("llmspell")` → C API (llmspell_init_library) → ScriptRuntime(Library mode)
      2. **Dual-Mode ScriptRuntime**: RuntimeMode enum extends existing ScriptRuntime for library mode
      3. **Selective Initialization**: ComponentLifecycleManager with SelectiveInitStrategy (Full/ToolsOnly/AgentsOnly/Custom)
      4. **External Runtime Integration**: ExternalRuntimeBridge trait extends ScriptEngineBridge
      5. **C API Layer**: Complete specification for require() support (llmspell_init_library, llmspell_inject_globals, etc.)
      6. **Tools-Only Mode**: SelectiveInitStrategy::ToolsOnly for partial component loading
      **Architecture Extended With:**
      - RuntimeMode in ScriptRuntime (dual embedded/library capability)
      - SelectiveInitStrategy in ComponentLifecycleManager (tools-only, agents-only patterns)
      - C API layer with complete FFI specification (Bridge-First Design section)
      - ExternalRuntimeBridge trait for external Lua/Node.js integration
      - Library mode build commands (Build System section)
      - Library mode testing methodology (Testing Strategy section)
    - [x] Todo: **ALL COMPLETED** - Architecture fully documents library mode integration - **Done** 2025-06-26T15:00:00-08:00
      - [x] Document C API/FFI interface specification for native module creation
      - [x] Add LibraryModeRuntime component architecture (vs ScriptRuntime)
      - [x] Detail selective initialization patterns (tools-only, agents-only, etc.)
      - [x] Document external runtime integration patterns (memory, threading, errors)
      - [x] Add library mode specific configuration and resource management
      - [x] Document component lifecycle for library mode vs embedded mode

- [x] **Task 12.4**: Architecture document `docs/rs-llmspell-complete-architecture.md` Readiness Review - **COMPLETED** 2025-06-26T16:15:00-08:00
  - [x] **Task/question 12.4.1**: Review the document for architecture, component, dependency consistency. you may need to read and re-read the document multiple times. Does the document ensure no feature overlaps, ensure no implementation overlaps, ensures reuse of common code etc., ensure no feature or api or naming conflicts, ensure clean integration between namespaces, modules, crates etc, ensure no circular dependencies, ensures clear unambiguous start for detailed designs to come later - **COMPLETED** 2025-06-26T15:30:00-08:00
    - [x] Answer: **MIXED ASSESSMENT** - Architecture is comprehensive with strong foundations but has several consistency issues:
      **❌ Issues Found:**
      1. **Feature Overlaps**: Storage systems (sled/rocksdb) have duplicate functionality; ScriptRuntime/AgentRuntime orchestration overlap
      2. **Implementation Overlaps**: Event system + Hook system solve similar problems; BaseAgent trait implementation duplication across all components
      3. **Naming Conflicts**: Table of contents has two section "16"s; Tool creation API inconsistency (Tools.create() vs Tools.register())
      4. **Crate Boundaries**: Unclear which components live in which crates (llmspell-core vs llmspell-agents etc.)
      5. **Potential Circular Dependencies**: AgentRegistry↔ToolRegistry through AgentWrappedTool; Hook system component access patterns
      **✅ Well Designed**: BaseAgent trait foundation, Bridge-first philosophy, Security-first design, Multi-language consistency, Production-ready infrastructure
      **Overall**: Ready for detailed design after addressing consistency issues
    - [x] Todo: **ALL COMPLETED** - Architecture consistency issues resolved - **Done** 2025-06-26T15:30:00-08:00
      - [x] Fix table of contents numbering conflict (immediate priority)
      - [x] Define clear crate boundaries with dependency diagram
      - [x] Consolidate Event system and Hook system into unified pattern
      - [x] Standardize API naming patterns (create vs register vs factory)
      - [x] Add dependency injection strategy to prevent circular dependencies
      - [x] Clarify ScriptRuntime vs AgentRuntime separation of concerns 
  - [x] **Task/question 12.4.2**: Review the document for implementation plan readiness. Does the document have enough content to give us a phased plan, each phase complete in itself to be built, tested, compiled and run? I'm not asking for an in-depth readiness plan, just enough architectural content and readiness to derive a plan for it. do you have enough information to phase components etc for different releases such as (they don't have to be in this order or number of phases but they give a sense of priority). - **COMPLETED** 2025-06-26T16:15:00-08:00
    - [x] Answer: **STRONG IMPLEMENTATION READINESS** - Architecture provides sufficient detail for phased implementation:
      **✅ WELL DOCUMENTED PHASES (Ready for Implementation):**
      - **Phase 0**: Complete crate structure (line 17852), workspace config, build scripts, technology dependencies
      - **Phase 1**: CLI entry point (line 17396), Lua bridge, ScriptRuntime architecture, command structure
      - **Phase 3**: Agent/Tool/Workflow architecture fully specified, CLI integration patterns
      - **Phase 4**: Comprehensive REPL architecture (line 3726+), state management, commands, multi-language support
      - **Phase 5**: JavaScript bridge documented, cross-engine compatibility matrix, unified API patterns
      - **Phase 7**: Serve mode architecture (line 13824+), daemon integration, platform service patterns
      - **Phase 11**: Library Mode architecture complete, C API layer, native module patterns
      - **Phase 12**: Cross-platform builds, Windows service integration, platform-specific configurations
      **🔧 PARTIALLY DOCUMENTED (Need Implementation Details):**
      - **Phase 2**: Debug command exists but needs debug protocol/stepping implementation details
      - **Phase 6/8**: MCP mentioned in protocols crate but needs specific MCP client/server implementation
      - **Phase 9/10**: A2A protocol mentioned but needs specific client/server implementation patterns
      **Overall Assessment**: 8/13 phases have complete implementation guidance, 5 phases need additional detail
    - [x] Todo: **ALL COMPLETED** - **Done** 2025-06-26T16:15:00-08:00
      - [x] Add detailed debug protocol implementation to CLI debug command specification - **COMPLETED** 2025-06-26T15:45:00-08:00
      - [x] Expand MCP client/server implementation details in Protocol Integration section - **COMPLETED** 2025-06-26T16:00:00-08:00
      - [x] Add A2A client/server implementation patterns and examples - **COMPLETED** 2025-06-26T16:05:00-08:00
      - [x] Add phase-specific testing strategies for each implementation phase - **COMPLETED** 2025-06-26T16:10:00-08:00
      - [x] Document component integration testing between phases - **COMPLETED** 2025-06-26T16:15:00-08:00 

- [x] **Task 12.5** Finalize Architecture Document - **COMPLETED** 2025-06-26T16:30:00-08:00
  - [x] move all research document from `/docs/technical/*` to `/docs/archives/research/` - **COMPLETED** 2025-06-26T16:25:00-08:00
  - [x] move `docs/rs-llmspell-complete-architecture.md` to `/docs/technical/master-architecture-vision.md` - **COMPLETED** 2025-06-26T16:30:00-08:00
### Phase 13: Implementation Roadmap - **COMPLETED** 2025-06-26T17:30:00-08:00
- [x] **Task 13.1**: Define all the phases of the implementation with goals for each phase and success critera for each phase - **COMPLETED**
  - [x] Created comprehensive 16-phase implementation roadmap at `/docs/technical/implementation-phases.md`
  - [x] The roadmap is structured in layers/phases (MVP Foundation, Production Features, Advanced Integration, Platform Support, Production Optimization)
  - [x] **CRITICAL**: Explicitly defined the scope for a Minimal Viable Product (MVP) - **COMPLETED**
    - [x] Detailed the specific traits required for the MVP (`BaseAgent`, `Agent`, `Tool`, `Workflow`)
    - [x] Listed the essential components for the MVP (`ScriptRuntime`, `mlua` script engine, ComponentLifecycleManager)
    - [x] Specified a small, core set of built-in tools for the initial release (12+ core tools across File System, HTTP, Utilities, System categories)
- [x] **Task 13.2**: Align the implementation plan with the architectural design - **COMPLETED**
  - [x] Established a priority order for component implementation (MVP Phases 0-3, then Production 4-7, then Advanced 8-15)
  - [x] Outlined a strategy for handling breaking changes between phases (Pre-1.0 allows breaking changes, post-1.0 major version boundaries only)
  - [x] Defined key testing milestones for each phase (Unit, Integration, Performance, Security, Cross-Platform tests per phase)
  - [x] **Implementation Priority Documentation**: All implementation notes documented in phases:
    - [x] Scheduling features (Scheduler component, cron/interval triggers, daemon mode) assigned to Phase 8 (post-MVP)
    - [x] Core agent templates (Chat, Research, Code) assigned to MVP priority, advanced templates to post-MVP
    - [x] Basic security profiles (Medium/High) assigned to MVP priority for production safety
    - [x] Global API injection (Agent, Tool, Tools, Workflow, etc.) assigned to MVP priority (Phase 1)
    - [x] Logger global API assigned to MVP priority with basic logging levels and structured output
    - [x] Basic `llmspell run <script>` command assigned to MVP priority (Phase 1)
    - [x] Unix pipeline support (stdin/stdout/stderr) assigned to MVP for shell integration
    - [x] PlatformServices trait and basic Linux/macOS support assigned to MVP priority
    - [x] ScriptRuntime as central orchestrator assigned to MVP priority (Phase 1)
    - [x] ComponentLifecycleManager with phased initialization assigned to MVP critical infrastructure
    - [x] Library mode C API design assigned to Phase 13 with early C API considerations

## Summary of Phase 12 Achievements - 2025-06-26T16:45:00-08:00

### Major Accomplishments:
1. **Comprehensive Architecture Document**: Created 15,034+ line standalone architecture guide
2. **Manual Review Process**: Completed detailed Q&A review of 16 architecture questions  
3. **Architectural Enhancement**: Added missing components for debug protocol, MCP/A2A integration, testing framework
4. **Consistency Resolution**: Fixed naming conflicts, crate boundaries, circular dependencies
5. **Implementation Readiness**: Achieved sufficient detail for 13-phase implementation roadmap
6. **Documentation Organization**: Properly archived research documents and finalized architecture location

### Key Technical Additions:
- **Debug Adapter Protocol (DAP)**: Complete IDE integration with breakpoints, variable inspection, step execution
- **MCP Protocol Integration**: Full client/server implementation with circuit breakers, health monitoring
- **A2A Protocol Integration**: Agent discovery, task delegation, conversation management with reputation system
- **Phase-Specific Testing Framework**: 13-phase validation with cross-platform testing matrix
- **ScriptRuntime Architecture**: Dual-mode support (embedded/library) with selective initialization
- **ComponentLifecycleManager**: 5-phase initialization strategy with dependency management

### Documentation Status:
- **Final Architecture**: `/docs/technical/master-architecture-vision.md` (15,034+ lines)
- **Research Archive**: 25 documents moved to `/docs/archives/research/`
- **Implementation Ready**: All 13 phases have sufficient architectural guidance
- **Quality Assurance**: Comprehensive testing strategy, security model, performance optimization

**Phase 12 is COMPLETE** - All architectural work finished, documentation organized, ready for Phase 13 Implementation Roadmap.

# Rs-LLMSpell Architecture Refinement TODO

## Overview
Comprehensive refinement of rs-llmspell architecture based on go-llms and Google ADK patterns, focusing on proper agent/tool/workflow hierarchy, hooks/events system, and built-in components.

**Status**: Phase 13 COMPLETED 2025-06-26T17:30:00-08:00  
**Current Phase**: Phase 14 - Final Update (IN PROGRESS)

## Completed Phases Summary

### Phases 1-12: Research and Architecture Foundation (COMPLETED)
All tasks from Phase 1 through Phase 12 were completed between 2025-06-20T08:15:00-08:00 and 2025-06-26T16:45:00-08:00.

**Major Achievements:**
- Complete 15,034+ line standalone architecture document
- Manual review of 16 architecture questions with architectural enhancements  
- Debug Adapter Protocol, MCP/A2A integration, Phase-specific testing framework
- Documentation organization and finalization

For detailed completion history, see TODO-DONE.md.

## Remaining Implementation Tasks



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