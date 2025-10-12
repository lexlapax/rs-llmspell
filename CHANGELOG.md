# Changelog

All notable changes to rs-llmspell will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.11.2] - 2025-10-12 - Local LLM Cleanup & Enhancement 🧹

Critical bug fixes and architecture cleanup for Local LLM integration. See [RELEASE_NOTES_v0.11.2.md](RELEASE_NOTES_v0.11.2.md) for full details.

### Fixed
- LocalLLM global registration (14/15 → 15/15 globals injected)
- Auto-load profile error messages with actionable suggestions
- Metal GPU detection on macOS (platform-aware device selection)
- HuggingFace API state corruption (replaced with direct HTTP)
- Model completeness validation (empty directories marked "exists")

### Removed
- llmspell-test binary (-675 LOC) - enforced single-binary architecture
- RagOptions CLI hack (-100 LOC) - replaced with unified profile system

### Added
- Unified profile system (10 builtin TOML profiles with --profile flag)
- T5 Safetensors support (dual-architecture: LLaMA GGUF + T5 Safetensors)
- ModelArchitecture enum with automatic format detection
- Platform-aware GPU selection (macOS Metal, Linux CUDA, CPU fallback)
- 6 T5 models (flan-t5-{small,base,large}, t5-{small,base,large})

### Changed
- Config consolidation (40+ Lua files updated, 95% complete)
- Model discovery UX (added HuggingFace/Ollama URLs in help)

### Statistics
- **Net Code**: -120 LOC (+755 new, -875 deleted)
- **Time**: 9h 27min across 8 sub-phases (7/8 complete)
- **Tests**: 72 passing, 0 warnings
- **Known Limitation**: Metal GPU blocked for both architectures by Candle v0.9

## [0.11.1] - 2025-10-09 - Bridge Consolidation & Documentation Completeness 🚀

Quality-focused consolidation between Phase 11 and Phase 12. See [RELEASE_NOTES_v0.11.1.md](RELEASE_NOTES_v0.11.1.md) for full details.

### Improved
- 87% compile speedup for bridge-only builds (38s → 5s) via feature gate architecture (ADR-042)
- Security documentation coverage (40% → 95%) with 371-line user guide
- Environment variables documentation (0% → 100%) with 41+ vars across 6 deployment patterns
- API consistency (60% → 100%) with Tool.execute() standardized across 40+ tools
- Workflow introspection via WorkflowResult.agent_outputs for debugging (ADR-043)

### Fixed
- Config global empty stub → full ConfigBridgeGlobal implementation (critical)
- TOML schema documentation (removed fake [security.sandboxing], added correct [tools.*])

### Removed
- StepType::Custom variant (-876 LOC) - unused workflow abstraction

### Statistics
- **Code**: -876 LOC deleted, +1,866 LOC documentation
- **Time**: 3-4 weeks consolidation phase
- **Tests**: All passing, zero warnings
- **Impact**: Foundation for Phase 12 (Memory), Phase 13 (MCP), Phase 14 (A2A)

## [0.11.0] - 2025-10-05 - Local LLM Integration 🔒

Complete offline AI inference with dual local backends (Ollama + Candle). See release notes for full details.

### Added
- Dual-backend local LLM: Ollama (100+ models) + Candle (native GGUF inference, 7 modules, 2,033 lines)
- Model CLI commands (list, pull, status, info) with HuggingFace/Ollama integration
- LocalLLM Lua API (status, list, pull, info methods)
- Candle GGUF pipeline (Q4_K_M quantization, CUDA/Metal/CPU auto-detection)
- LocalProviderInstance trait with model management
- Config support for [providers.ollama] and [providers.candle]

### Fixed
- Provider factory registration (Backend 'candle' not configured)
- Environment variable expansion (${HOME} created literal directory)
- False credential warnings for local providers
- Tokenizer download fallback, Ollama URL preservation, chat templates

### Performance
- First token: 150ms (target <200ms, 25% faster)
- Throughput: 40 tok/s (target >30, 33% faster)
- Memory: ~400MB/2K tokens (target <5GB, 8x better)
- All 7 targets exceeded by 25-33%

### Statistics
- **Code**: 2,033 lines Candle + integrations
- **Time**: 4.5 days (vs 20 estimated, 77% faster)
- **Tests**: 10/10 passing (5 Candle + 5 Ollama)
- **Privacy**: 100% offline, air-gap compatible, HIPAA/GDPR ready

## [0.10.0] - 2025-01-28 - Service Integration & IDE Connectivity 🚀

Production daemon infrastructure and IDE integration. See release notes for full details.

### Added
- Unix daemon infrastructure (double-fork, PID files, signal handling, log rotation)
- Tool CLI commands (list, info, invoke, search, test) via kernel protocol
- Fleet management (Bash/Python managers, Docker orchestration, OS-level isolation)
- Feature-based modular builds (minimal 19MB, common 25MB, full 35MB)
- Jupyter Wire Protocol v5.3 (5-channel ZeroMQ, DAP support)
- systemd/launchd integration for production deployment

### Changed
- **⚠️ BREAKING**: Default build minimal (19MB, was 33.6MB) - use `--features common` or `--features full`
- Daemon mode via `--daemon` flag (TTY detachment, PID tracking)
- Tool execution in kernel via protocol messages (not CLI process)
- Crate consolidation (18 → 17 crates)

### Fixed
- Process isolation (session leadership, TTY detachment)
- Signal-safe shutdown (atomic operations, resource cleanup)
- PID file race conditions, log rotation timing
- Tool discovery, parameter marshaling, timeout handling

### Performance
- All 10 targets exceeded by 10-40% (daemon startup 1.8s, message handling 3.8ms, heartbeat 0.8ms)

### Statistics
- **Code**: 450+ files modified
- **Tests**: 486 total (kernel 57, bridge 334, CLI 57, fleet 38)
- **Binary Size**: 43% smaller (minimal), 26% smaller (common) vs v0.9.0
- **Time**: 25 working days

## [0.9.0] - 2025-01-21 - Interactive Kernel & Debugging Infrastructure 🎯

Unified kernel architecture with debugging and tracing. See release notes for full details.

### Added
- Unified kernel (debug, sessions, transport consolidated into llmspell-kernel)
- Global IO runtime (fixes "dispatch task is gone", HTTP clients survive 60+ seconds)
- 5-channel Jupyter protocol (Shell, IOPub, Control, Stdin, Heartbeat)
- DAP debug infrastructure (10 essential commands, breakpoints, stepping)
- Comprehensive tracing (13 operation categories, -3.99% overhead)
- Future-proofing traits (memory, service, multi-language debug, observability mocks)

### Changed
- **⚠️ BREAKING**: Execution API `kernel.execute()` replaces `runtime.run()`
- Crate reduction (26 → 18, 8 merged into kernel)
- Direct integration (ScriptRuntime in kernel, no spawning)
- New [kernel] config sections (transport, debug, sessions)

### Fixed
- "dispatch task is gone" error (global IO runtime)
- Runtime context mismatches (direct integration)
- HTTP client timeouts (60+ second operations)
- Breakpoint management, variable inspection, step debugging

### Performance
- All 7 targets exceeded by 20-40% (message 3.8ms, tool 7ms, agent 35ms, tracing -3.99%)

### Statistics
- **Applications**: 9/9 validated (100% success, webapp-creator: 21 agents, 35 files)
- **Tests**: 116+ kernel, 15 mock
- **Quality**: 0 clippy warnings, >95% tracing coverage

## [0.8.0] - 2024-12-29 - RAG & Multi-Tenant Vector Storage 🚀

Production RAG system with HNSW vector storage and multi-tenant isolation. See release notes for full details.

### Added
- Complete RAG infrastructure (llmspell-rag, llmspell-storage, llmspell-tenancy crates)
- HNSW vector storage (<8ms search @ 100K vectors, <35ms @ 1M)
- Multi-tenant architecture (StateScope::Custom, 3% overhead, zero cross-tenant access)
- 11 new tools (pdf-processor, document-chunker, embedding-generator, vector-search, web-scraper, etc.)
- RAG Lua global (ingest, search, delete with session-aware context)
- Embedding pipeline (OpenAI, Cohere, HuggingFace, 80% cache hit rate)

### Changed
- **⚠️ BREAKING**: RAG operations require explicit scope `RAG.search(query, scope)`
- New [rag] config sections (embeddings, storage, tenancy)
- Vector storage traits in llmspell_storage::vector_storage

### Fixed
- Vector search HNSW parameters optimization
- Embedding cache memory leaks (80% hit rate)
- Batch processing timeouts, per-vector memory (2KB → 1.5KB)
- Scope leakage, namespace race conditions, permission bypass

### Performance
- All 7 targets exceeded by 14-80% (search 8ms, embedding 45ms, ingestion 1.8K/sec)

### Statistics
- **Crates**: 3 new (rag, storage, tenancy)
- **Tools**: 11 new (37+ total)
- **Examples**: 20+ RAG examples (60+ total)
- **Tests**: 250+ new
- **Documentation**: 4 consolidated guides

## [0.7.0] - 2025-08-26 - First MVP Release 🎉

### Platform Testing Status
- ✅ **Tested on macOS 15.7 (Darwin 24.6.0, ARM64)** - All features working
- ⏳ **Linux** - Testing pending
- ⏳ **Windows** - Testing pending

### Added

#### Production-Ready Infrastructure
- **Test Infrastructure Revolution**: Feature-based testing system with centralized `llmspell-testing` crate
- **Configuration Architecture**: Hierarchical 4-layer config system (defaults → file → env → CLI)
- **Security Boundaries**: Mandatory sandboxing with filesystem/network/process isolation
- **Bridge Architecture Fix**: StepExecutor now has ComponentRegistry access for real execution

#### 7 Production Applications (Universal → Professional)
- **Universal Layer** (2-3 agents): file-organizer, research-collector - Zero config, <1min execution
- **Power User Layer** (4 agents): content-creator - Template management, <2min execution  
- **Business Layer** (5 agents): communication-manager - Professional outputs, state persistence
- **Professional Layer** (7-8 agents): process-orchestrator, code-review-assistant - Enterprise features
- **Expert Layer** (20 agents): webapp-creator - Full-stack generation in 4.5 minutes

#### Example Consolidation (77% Reduction)
- Reduced from 157 scattered examples to 35 high-quality examples
- Progressive learning path with 10-minute first success
- Created 6 comprehensive Rust developer examples
- All examples validated and working with real LLMs

#### Documentation Standardization  
- Standardized 58 README.md files with consistent navigation
- Created docs/README-TEMPLATES.md with 4 category templates
- Added user-friendly `scripts/llmspell-easy.sh` launcher with API key wizard
- Established breadcrumb navigation pattern across all documentation

#### Script Infrastructure
- Three-tier quality assurance scripts (minimal, fast, full)
- Category-based test execution with `test-by-tag.sh`
- Test discovery and timing analysis tools
- Automated API key detection and setup guidance

### Changed

#### Breaking API Changes
- `Agent::create()` → `Agent::builder().build()` pattern
- `Tool::process()` → `Tool::execute()` for consistency
- `WorkflowBuilder` → `WorkflowComposer` for clarity
- State operations return `Option<T>` instead of `Result<T>`
- All test helpers moved to centralized `llmspell-testing` crate

#### Configuration Changes
- New required config.toml with security boundaries
- Environment variables now use `LLMSPELL_` prefix
- Timeout configurations moved from code to config file
- Provider-specific timeout configurations

#### License Change
- Changed from dual MIT/Apache-2.0 to Apache-2.0 only

### Fixed

#### Critical Architecture Fixes
- **StepExecutor Registry Access**: Fixed inability to execute real components
- **Timeout Configuration**: Fixed hardcoded 30-second timeouts causing failures
- **State Isolation**: Fixed state leakage in parallel workflows
- **Memory Leaks**: Fixed in long-running workflows with proper cleanup
- **Security Vulnerabilities**: Fixed sandbox escape paths
- **Test Infrastructure**: Fixed cfg_attr syntax compilation errors

### Performance

#### Metrics Achieved (vs Targets)
- Agent Creation: 2-3ms (target <50ms) - **94% faster**
- Tool Initialization: 1-2ms (target <10ms) - **80% faster**
- Workflow Step: 15-25ms (target <100ms) - **75% faster**
- State Operations: <1ms (target <5ms) - **80% faster**
- Config Load: 45ms (target <100ms) - **55% faster**
- WebApp Creator: 4.5min (target <5min) - **10% faster**

### Statistics
- **Files Modified**: 536+ files
- **Tests Added**: 200+ new tests
- **Documentation**: 58 READMEs standardized
- **Examples**: 157 → 35 (77% reduction)
- **Applications**: 7 production-ready apps
- **Performance**: All metrics exceeded targets

## [0.6.0] - 2025-08-01

### Added

#### Complete Session and Artifact Management System
- **Session Lifecycle Management**: Create, suspend, resume, and complete long-running sessions
- **Artifact Storage**: Content-addressed storage for session outputs with blake3 hashing
- **Session Persistence**: Save and restore full session context across restarts
- **Session Replay**: Comprehensive replay capability using existing hook infrastructure
- **Automatic Artifact Collection**: Tool outputs and agent responses automatically captured
- **User Artifact Storage**: Public API for users to store their own files and data

#### Session Infrastructure (39/39 tasks completed)
- **New llmspell-sessions crate**: Core session management with 24.5µs creation time
- **Session States**: Active, Suspended, Completed, Failed, Archived with transitions
- **Session Metadata**: Tags, parent references, operation counts, timestamps
- **Session Configuration**: Retention policies, auto-save intervals, resource limits
- **Artifact Versioning**: Automatic version management for same-named artifacts
- **Compression Support**: LZ4 compression for artifacts >10KB (50%+ ratio)

#### Artifact Management Features
- **Content Hashing**: Blake3 for 10x faster hashing than SHA2
- **Deduplication**: Content-based addressing prevents duplicate storage
- **Metadata System**: Rich metadata with tags, MIME types, custom fields
- **Query System**: Advanced search with filtering, pagination, and sorting
- **Binary Support**: Handles text, JSON, and binary data efficiently
- **Size Limits**: Configurable limits (default 100MB per artifact)

#### Hook and Event Integration
- **Session Lifecycle Hooks**: session:start, session:end, session:suspend, session:resume
- **Artifact Hooks**: artifact:created, artifact:accessed via collectors
- **Replayable Hooks**: All session hooks implement ReplayableHook trait
- **Event Correlation**: Sessions generate correlated events with timeline support
- **Built-in Collectors**: ToolResultCollector and AgentOutputCollector
- **Performance**: Hook overhead maintained at 11µs (well under 1ms target)

#### Session Policies and Middleware
- **Timeout Policies**: Configurable session duration and idle timeouts
- **Resource Policies**: Memory, token, operation, and cost limits
- **Rate Limiting**: Global, per-session, and per-operation limits
- **Middleware Patterns**: Sequential, Parallel, and Voting execution
- **Policy Composition**: Combine multiple policies with different strategies
- **Performance**: <10µs overhead per policy evaluation

#### Script Bridge Implementation
- **Session Global**: Complete Lua API for session management
- **Artifact Global**: Store, retrieve, list, and delete artifacts from scripts
- **Thread-Local Context**: getCurrent/setCurrent for active session
- **Example Suite**: 5 comprehensive examples demonstrating all features
- **Integration Examples**: Sessions work with State, Events, Hooks, Agents

#### Performance Achievements
- **Session Creation**: 24.5µs (target: <50ms) - 2000x better
- **Session Save**: 15.3µs (target: <50ms) - 3200x better
- **Session Load**: 3.4µs (target: <50ms) - 14700x better
- **Artifact Store**: <1ms for text/JSON artifacts
- **Query Performance**: List 100 artifacts in <5ms
- **Memory Efficiency**: Chunked storage for large artifacts

### Changed

#### Architecture Enhancements
- **Three-Layer Pattern**: SessionBridge (async) → SessionGlobal (sync) → Lua bindings
- **Foundation-First Approach**: Test categorization moved before API work
- **Bridge Consistency**: All bridges now created externally (follows HookBridge pattern)
- **Manager Pattern**: All services follow Manager suffix convention

#### API Improvements
- **Public Artifact API**: Added store_artifact, get_artifact, list_artifacts to SessionManager
- **Query Interface**: ArtifactQuery builder for complex searches
- **File Support**: store_file_artifact for direct file uploads
- **Metadata Handling**: Consistent metadata preservation across operations

### Fixed
- Session isolation between different session contexts
- Artifact content integrity with hash verification
- Replay functionality integration with existing infrastructure
- Binary data handling in script bridge
- MIME type detection and preservation

### Performance

| Component | Target | Achieved | Status |
|-----------|--------|----------|---------|
| Session Creation | <50ms | 24.5µs | ✅ 2000x better |
| Session Save | <50ms | 15.3µs | ✅ 3200x better |
| Session Load | <50ms | 3.4µs | ✅ 14700x better |
| Hook Overhead | <1ms | 11µs | ✅ 90x better |
| Artifact Store | <5ms | <1ms | ✅ 5x better |
| Memory Overhead | <20% | <10% | ✅ Exceeded |

### Documentation
- **Session Management Guide**: User guide for session features
- **Artifact API Guide**: Complete artifact storage documentation  
- **Session Examples**: 5 comprehensive Lua examples
- **Developer Guide**: Session and artifact implementation details
- **API Reference**: Updated with Session and Artifact globals

### Infrastructure
- **Test Foundation**: Moved test categorization to beginning of phase
- **Example Organization**: Prepared structure for Phase 7 reorganization
- **Release Process**: Streamlined with automated version updates

## [0.5.0] - 2025-07-29

### Added

#### Comprehensive Persistent State Management System
- **Multi-Backend Support**: Memory (default), Sled, and RocksDB backends with unified API
- **Advanced StateManager**: 618-line core with async operations, hook integration, and <5ms latency
- **6 State Scoping Levels**: Global, Agent, Workflow, Step, Session (Phase 6 ready), and Custom
- **Automatic Agent State Persistence**: State saved on pause/stop, manual load for API safety
- **Hook Integration**: All state changes trigger before/after hooks with <2% overhead

#### Enterprise-Grade Migration Framework
- **Schema Versioning**: Semantic versioning with compatibility checking
- **Field Transformations**: Copy, Default, and Remove operations (Custom transformers strategically deferred)
- **Exceptional Performance**: 2.07μs per item (48,000x better than 100ms/1000 target)
- **Safe Migrations**: Automatic rollback on failure with validation framework
- **Migration Events**: Full hook integration for migration lifecycle

#### Production Backup & Recovery System
- **Atomic Backup Operations**: SHA256-validated backups with compression support
- **Point-in-Time Recovery**: Restore any backup with progress tracking
- **Retention Policies**: Automated cleanup with configurable strategies
- **Multiple Formats**: JSON (human-readable) and binary (space-efficient)
- **Script Integration**: Complete Lua API for backup/restore operations

#### Advanced Performance Architecture (6 Modules)
- **StateClass Classification**: Critical, Standard, Bulk, and Archive tiers
- **Fast Path Operations**: Lock-free reads for critical state
- **Async Hook Processing**: Non-blocking hook execution pipeline
- **Unified Serialization**: Single-pass serialization for efficiency
- **Lock-Free Agent State**: Optimized concurrent access patterns

#### Comprehensive Security Implementation
- **Circular Reference Detection**: Prevents serialization infinite loops
- **Sensitive Data Protection**: Automatic API key redaction in state
- **Scope-Based Isolation**: Enforced separation between agent states
- **Access Control**: Per-agent lock management for thread safety
- **Path Sanitization**: Prevention of directory traversal attacks

#### Testing Infrastructure Revolution
- **7 Test Categories**: unit, integration, tool, agent, workflow, external, security
- **Test Discovery System**: Dynamic enumeration with tag-based filtering
- **Quality Check Scripts**: Minimal (seconds), Fast (~1min), Full (5+min)
- **Performance Benchmarking**: Automated regression detection
- **CI/CD Integration**: Full pipeline support for categorized testing

#### New Crate Architecture
- **llmspell-state-traits**: Trait definitions to prevent circular dependencies
- **llmspell-state-persistence**: 35+ module implementation across 7 subsystems

### Changed

#### Architectural Improvements
- **Dual-Crate Structure**: Clean dependency management with trait separation
- **Module Organization**: 7 major subsystems (Core, Backup, Migration, Performance, Schema, Security, Utilities)
- **Script Bridge Enhancement**: State global with save/load/delete/list_keys/migrate operations
- **Testing Overhaul**: Complete reorganization of llmspell-testing crate

#### Performance Optimizations
- **State Operations**: Maintained <1ms reads, achieved <5ms writes
- **Hook Overhead**: Reduced to <2% (exceeded <5% target)
- **Memory Usage**: <10% increase over baseline (validated)
- **Migration Speed**: Extraordinary 2.07μs per item performance

### Fixed
- Agent lifecycle state persistence integration gaps
- State isolation enforcement between agents
- Concurrent state access synchronization
- Sensitive data leakage in error messages

### Performance

| Component | Target | Achieved | Status |
|-----------|--------|----------|---------|
| State Read | <1ms | <1ms | ✅ Maintained |
| State Write | <5ms | <5ms | ✅ Achieved |
| Hook Overhead | <5% | <2% | ✅ Exceeded |
| Migration Performance | 100ms/1000 items | 2.07μs/item | ✅ 48,000x better |
| Memory Increase | <10% | <10% | ✅ Validated |
| Backup/Recovery | Atomic | SHA256 validated | ✅ Production ready |

### Documentation
- **New Guides**: State Management (2,100+ lines), Best Practices (1,800+ lines)
- **Technical Docs**: Updated state architecture documentation for Phase 5
- **Examples**: Comprehensive Lua and Rust examples for state persistence
- **Migration Guide**: Step-by-step migration examples with patterns

### Known Deferrals (Strategic, No Production Impact)
- Custom field transformers (basic transforms handle 80% of cases)
- JavaScript state bridge completion (Lua fully operational)
- Advanced session lifecycle management (belongs in Phase 6)
- Backup encryption (security architecture complete, encryption is enhancement)

## [0.4.0] - 2025-07-25

### Added

#### Comprehensive Hook and Event System
- **40+ Hook Points**: Complete coverage across agents (6 states), tools (34 tools), and workflows (4 patterns)
- **Automatic Performance Protection**: CircuitBreaker ensures <1% overhead with automatic slow hook disabling
- **Cross-Language Support**: Hooks and events work seamlessly between Lua and native code
- **9 Hook Result Types**: Continue, Modified, Cancel, Redirect, Replace, Retry, Fork, Cache, Skipped
- **Composite Hook Patterns**: Sequential, Parallel, FirstMatch, and Voting execution strategies

#### High-Performance Event System
- **Universal Event Bus**: >90,000 events/sec throughput with pattern-based routing
- **Cross-Language Event Propagation**: UniversalEvent format enables Lua ↔ Rust communication
- **Flow Control**: 4 overflow strategies (DropOldest, DropNewest, Block, Reject) prevent overload
- **Event Persistence**: Optional storage using unified llmspell-storage backend
- **Pattern Subscriptions**: Wildcard support for flexible event routing (e.g., `*.error`, `user.*`)

#### 8 Production-Ready Built-in Hooks
1. **LoggingHook**: Smart filtering with configurable levels
2. **MetricsHook**: Comprehensive metrics collection with histograms
3. **DebuggingHook**: Enhanced debugging with trace capture
4. **SecurityHook**: Audit logging and input validation
5. **CachingHook**: Automatic result caching with TTL and LRU eviction
6. **RateLimitHook**: Token bucket algorithm for API quota management
7. **RetryHook**: Exponential backoff with configurable strategies
8. **CostTrackingHook**: AI/ML operation cost monitoring with alerts

#### Enhanced Integration Points
- **Agent Lifecycle Hooks**: All 9 agent states trigger appropriate hooks
- **Tool Execution Hooks**: 8 hook points per tool with resource tracking
- **Workflow Hook Support**: 14 execution phases across Sequential, Conditional, Loop, and Parallel patterns
- **Cross-Component Coordination**: Dependency graphs and event correlation for complex scenarios

#### Lua API Enhancements
- **Hook.register()**: Priority support (highest, high, normal, low, lowest)
- **Hook.unregister()**: Standalone function and method variants
- **Hook.list()**: Advanced filtering by language, priority, tag, and hook_point
- **Event.subscribe()**: Pattern matching with wildcards
- **Event.emit()**: Cross-language event publishing with metadata

#### Future-Proofing Components
- **ReplayableHook Trait**: Enables hook persistence for Phase 5
- **HookAdapter Trait**: Language-specific hook adaptation
- **DistributedHookContext**: Prepared for Phase 16-17 distributed operations
- **SelectiveHookRegistry**: Ready for Phase 18 library mode
- **JavaScript/Python Stubs**: Architecture prepared for future language support

### Changed

#### Performance Optimizations
- **Hook Overhead**: Reduced from ~5% to <1% through hot path optimizations
- **Memory Usage**: 40-60% reduction in allocations using Cow patterns
- **Lock Contention**: 60-80% reduction using atomic operations
- **Circuit Breaker**: Faster failure detection (3 vs 5) and recovery (15s vs 30s)

#### Architecture Enhancements
- **Three-Layer Bridge Pattern**: Clean separation of language abstractions, bridge logic, and bindings
- **Event-Driven Hook System**: Unified architecture eliminates hook/event overlap
- **Performance Monitoring**: Integrated metrics collection with automatic protection
- **Cross-Language Bridge**: HookBridge and EventBridge enable seamless integration

### Fixed
- Tool invocation parameter format issue in agent:invokeTool()
- Memory leaks in event subscription management
- Race conditions in concurrent hook registration

### Performance

| Component | Target | Achieved |
|-----------|--------|----------|
| Hook Execution Overhead | <5% | <1% |
| Hook Registration | <0.1ms | ~0.46ms |
| Event Throughput | >100K/sec | >90K/sec |
| Circuit Breaker Response | <5ms | <2ms |
| Memory Usage | Minimal | -40% |

## [0.3.0] - 2025-07-23

### Added

#### 34 Production-Ready Tools (8 New)
**File System Tools (5)**:
- `file_operations` - Read, write, delete, copy, move files
- `archive_handler` - Create and extract archives (zip, tar, tar.gz)
- `file_watcher` - Monitor file system changes
- `file_converter` - Convert file encodings and formats
- `file_search` - Search file contents with patterns

**Data Processing Tools (4)**:
- `json_processor` - Process JSON with jq-like queries
- `csv_analyzer` - Analyze and query CSV data
- `http_request` - Make HTTP requests (GET, POST, PUT, DELETE)
- `graphql_query` - Execute GraphQL queries and mutations

**Web & Network Tools (7)** - *3 new in v0.3.0*:
- `web_search` - Search with multiple providers (enhanced)
- `web_scraper` - Extract content from web pages **(NEW)**
- `url_analyzer` - Analyze and validate URLs **(NEW)**
- `api_tester` - Test REST APIs with validation **(NEW)**
- `webhook_caller` - Call webhooks with retry logic **(NEW)**
- `webpage_monitor` - Monitor web pages for changes **(NEW)**
- `sitemap_crawler` - Parse and crawl sitemaps **(NEW)**

**System Integration Tools (4)**:
- `environment_reader` - Read environment variables
- `process_executor` - Execute system commands
- `service_checker` - Check service availability
- `system_monitor` - Monitor system resources

**Utility Tools (10)** - *2 new in v0.3.0*:
- `uuid_generator` - Generate UUIDs (v4, v7)
- `base64_encoder` - Encode/decode Base64
- `hash_calculator` - Calculate cryptographic hashes
- `text_manipulator` - Transform and analyze text
- `calculator` - Evaluate mathematical expressions
- `date_time_handler` - Handle dates and times
- `diff_calculator` - Calculate text/JSON differences
- `data_validation` - Validate data against schemas
- `template_engine` - Render templates (Handlebars/Tera)
- `database_connector` - Query databases (PostgreSQL, MySQL, SQLite) **(NEW)**

**Media Processing Tools (3)**:
- `audio_processor` - Extract audio metadata
- `video_processor` - Extract video metadata
- `image_processor` - Extract image metadata

**Communication Tools (1)** - *new in v0.3.0*:
- `email_sender` - Send emails via SMTP/SendGrid/SES **(NEW)**

#### Agent Infrastructure
- **Agent Factory**: Flexible agent creation with builder pattern
- **Agent Registry**: Centralized discovery and management
- **Agent Templates**: 8 pre-configured agent types (basic, tool-orchestrator, research, etc.)
- **LLM Integration**: Full provider integration with OpenAI, Anthropic, and more
- **Lifecycle Management**: Complete state machine with 7 states
- **Dependency Injection**: Type-safe container with service discovery

#### Workflow Orchestration
- **Sequential Workflows**: Step-by-step execution with data flow
- **Conditional Workflows**: Branching based on conditions
- **Loop Workflows**: Iteration with collection/count/while patterns
- **Parallel Workflows**: Concurrent execution with fork-join
- **Workflow Composition**: Workflows can contain other workflows
- **Agent Integration**: Agents can be workflow steps

#### Script Bridge Enhancements
- **Synchronous API**: All async operations wrapped for script languages
- **Global Injection**: Zero-configuration access to all features
- **23+ Lua Methods**: Complete agent management API
- **Comprehensive Error Handling**: Consistent error propagation
- **Performance Optimized**: <10ms overhead for all operations

### Changed

#### Breaking Changes - Tool Standardization
All tools now use consistent parameter naming:
- **Primary data**: `input` (was: text, content, data, expression, query, template, etc.)
- **File paths**: `path` for single files, `source_path`/`target_path` for operations
- **Operations**: Required `operation` parameter for multi-function tools

Examples of changes:
```lua
-- OLD
tool:execute({content = "data", file_path = "/tmp/file"})
-- NEW  
tool:execute({input = "data", path = "/tmp/file"})

-- OLD
calculator:execute({expression = "2+2"})
-- NEW
calculator:execute({operation = "evaluate", input = "2+2"})
```

#### Breaking Changes - Agent API
- Agent creation requires explicit model specification:
  ```lua
  -- OLD
  Agent.create({name = "assistant"})
  -- NEW
  Agent.create({
      name = "assistant",
      model = "openai/gpt-4",  -- provider/model format
      system_prompt = "You are helpful"
  })
  ```
- Default agent type changed from "basic" to "llm"
- Agent factory no longer implements Default trait
- Provider configuration requires explicit `provider_type` field

#### Breaking Changes - Async API Removal
All async methods removed in favor of synchronous API:
- `Agent.createAsync()` → `Agent.create()`
- `Tool.executeAsync()` → `tool:execute()`
- `Workflow.executeAsync()` → `workflow:execute()`

#### Response Format Standardization
All tools now return consistent ResponseBuilder format:
```json
{
  "operation": "operation_name",
  "success": true,
  "data": {...},
  "error": null,
  "metadata": {...}
}
```

### Security

- **Calculator DoS Protection**: Fixed regex complexity vulnerability with timeouts
- **Path Traversal Prevention**: Comprehensive validation for all file operations
- **Symlink Attack Prevention**: Blocks symlink-based attacks
- **Resource Limits**: Memory, CPU, and operation count limits enforced
- **SSRF Protection**: URL validation and private IP blocking
- **Input Sanitization**: Framework prevents injection attacks
- **Credential Security**: Memory protection for sensitive data
- **File Upload Security**: Magic number verification and content scanning
- **200+ Security Tests**: Comprehensive security test suite

### Performance

- **Tool Initialization**: 52,600x faster than requirements (107-190ns)
- **Global Injection**: <5ms overhead for script access
- **Agent Creation**: <50ms including provider setup
- **Workflow Execution**: <20ms overhead per step
- **Memory Usage**: Reduced by 40% through shared utilities
- **Test Performance**: 90%+ coverage with fast execution

### Infrastructure

- **llmspell-utils**: New crate for shared functionality (95% DRY compliance)
- **Test Organization**: Tagged tests (unit, integration, tool, agent, external)
- **Quality Scripts**: Three levels (minimal, fast, full) for different scenarios
- **Error Handling**: Consistent error types and propagation
- **Documentation**: Comprehensive guides for users and developers

### Fixed

- **agent:invokeTool() parameter format** - Fixed double-nesting requirement
  - Previously required: `{parameters: {parameters: {actual_params}}}`
  - Now accepts: `{actual_params}` directly
  - Affects all 34 tools when invoked through agents
- Multiple security vulnerabilities (see Security section)
- Inconsistent parameter naming across tools
- Code duplication issues (reduced from 40% to <5%)
- Agent provider integration issues
- Memory leaks in long-running operations
- Race conditions in parallel workflows

## [0.2.0] - 2025-07-11

### Added
- Complete Phase 2: Self-Contained Tools Library (25 tools)
- Data Processing: JsonProcessor, CsvAnalyzer, HttpRequest, GraphQLQuery
- File System: FileOperations, ArchiveHandler, FileWatcher, FileConverter, FileSearch
- System Integration: EnvironmentReader, ProcessExecutor, ServiceChecker, SystemMonitor
- Media Processing: AudioProcessor, VideoProcessor, ImageProcessor
- Utilities: Calculator, TextManipulator, DateTimeHandler, UuidGenerator, HashCalculator, Base64Encoder, DiffCalculator, TemplateEngine, DataValidation
- Search: WebSearch
- Provider enhancements: ModelSpecifier, base URL overrides
- JSON API for script bridge

### Technical
- 90%+ test coverage
- Zero warnings policy enforced
- All tools follow BaseAgent/Tool trait hierarchy

## [0.1.0-alpha.1] - 2025-06-27

### Alpha Release - Architecture Testing Preview

**Release Date**: June 27, 2025  
**Release Type**: Development Preview (Alpha)  
**Purpose**: Architecture validation and feedback gathering  

#### ⚠️ IMPORTANT ALPHA WARNINGS

This is an **alpha release** intended for architecture testing and feedback only:
- **NOT ready for production use**
- **Agent/Tool execution is placeholder implementation only**
- **No actual LLM calls are made** (provider listing only)
- **Only Lua scripting is available** (no JavaScript/Python yet)
- **Breaking changes expected** before v1.0.0

#### What This Release Includes

##### ✅ Working Features
- **ScriptEngineBridge Architecture**: Language-agnostic script execution abstraction
- **Lua Script Execution**: Basic Lua 5.4 integration via mlua
- **Provider Infrastructure**: Provider listing (no actual LLM calls)
- **CLI Commands**: `run`, `exec`, `info`, `providers`
- **Configuration System**: TOML-based configuration loading
- **Security Sandboxing**: Memory limits and execution constraints
- **Streaming Infrastructure**: Types and interfaces (stub implementation)
- **Multimodal Types**: Content type definitions (structure only)

##### ❌ NOT Working Yet
- **Agent Execution**: `Agent.create()` returns mock data only
- **Tool Execution**: Tools cannot be called
- **Workflow Orchestration**: Not implemented
- **Actual LLM Calls**: Provider integration is listing only
- **JavaScript/Python**: Only Lua is available
- **State Persistence**: In-memory only

#### Performance Metrics

All Phase 1 performance targets exceeded:

| Metric | Target | Achieved | Factor |
|--------|--------|----------|--------|
| Script Startup | <100ms | 32.3μs | 3,000x |
| Streaming Latency | <50ms | 12.1μs | 4,000x |
| Memory Limit | 50MB | Enforced | ✅ |
| Bridge Overhead | <5% | <0.1% | 50x |
| Large Script | N/A | 5.4ms/0.47MB | Excellent |

#### Architecture Validation

This release validates the core architectural decisions:
- ✅ **Bridge Pattern**: ScriptEngineBridge abstraction proven
- ✅ **Language Agnostic**: API injection framework working
- ✅ **Performance**: Minimal overhead from abstractions
- ✅ **Extensibility**: Ready for multiple script engines
- ✅ **Testing**: 188+ tests passing across all crates

#### Safe to Use For
- Testing the ScriptEngineBridge architecture
- Evaluating Lua script execution performance
- Reviewing API design and providing feedback
- Understanding the project structure
- Contributing to core infrastructure

#### NOT Ready For
- Production applications
- Building actual LLM-powered tools
- Agent-based workflows
- Tool integration
- Real LLM API calls

#### Known Issues
- CLI integration tests have overly strict assertions
- JavaScript engine (boa) has dependency issues
- Some example scripts use placeholder APIs

#### Getting Started

```bash
# Clone the repository
git clone https://github.com/lexlapax/rs-llmspell
cd rs-llmspell

# Build the project
cargo build --workspace

# Run a simple Lua script
./target/debug/llmspell run examples/basic-math.lua

# View available providers (listing only)
./target/debug/llmspell providers

# Get system information
./target/debug/llmspell info
```

#### Documentation
- [Architecture Overview](docs/technical/master-architecture-vision.md)
- [Getting Started Guide](docs/user-guide/getting-started.md)
- [Phase 1 Handoff](docs/in-progress/PHASE02_HANDOFF.md)
- [Known Issues](docs/KNOWN_ISSUES.md)

#### Feedback Welcome

As this is an alpha release, we welcome feedback on:
- Architecture and API design
- Performance characteristics
- Documentation clarity
- Missing features for MVP
- Integration patterns

Please file issues at: https://github.com/lexlapax/rs-llmspell/issues

### Phase 0 Complete - Ready for Phase 1 Implementation 🎉
**Date**: 2025-06-26 (Evening Update)

Phase 0 Foundation Infrastructure has been **COMPLETED** with all deliverables ready for Phase 1. This marks the successful establishment of our core infrastructure and the beginning of functional implementation.

#### **Phase 0 Achievements** ✅
- **12-crate workspace** fully operational with zero warnings
- **165 comprehensive tests** (unit, integration, property, doc tests)
- **Complete CI/CD pipeline** with 7 jobs, quality gates, and GitHub Pages
- **>95% documentation coverage** with all public APIs documented
- **Clean build time**: 21 seconds (exceeded target of <60s)
- **Local quality tools** matching CI requirements
- **Performance benchmarking** framework with baselines

#### **Architectural Updates for Phase 1** 🔄
- **Streaming Support**: Added comprehensive streaming execution model
- **Multimodal Content**: Added MediaContent types for images, audio, video
- **Utils Crate**: Added 13th crate (llmspell-utils) for shared utilities
- **Enhanced Traits**: BaseAgent and Tool traits extended with streaming/multimodal

#### **Phase 1 Preparation Complete** 📋
- ✅ Created comprehensive Phase 1 Design Document
- ✅ Created detailed Phase 1 TODO list (37 tasks over 10 days)
- ✅ Updated implementation roadmap with new requirements
- ✅ Architecture document enhanced with streaming and multimodal sections

### Architecture and Design Complete - Ready for Implementation
**Date**: 2025-06-26

Today marks a major milestone in the rs-llmspell project: **complete architecture and design finalization**. After extensive research through Phases 1-13, we have delivered a comprehensive, implementation-ready framework architecture.

### Major Achievements ✅

#### **Complete Architecture Documentation**
- **15,034+ line standalone architecture document** (`docs/technical/master-architecture-vision.md`)
- All architectural decisions documented with rationale
- Production-ready specifications with code examples
- Zero external dependencies - completely self-contained reference

#### **16-Phase Implementation Roadmap**
- **Phase 0**: Foundation Infrastructure (2 weeks) - **NEXT**
- **Phases 1-3**: MVP (Agents, Tools, Workflows) - 8 weeks
- **Phases 4-15**: Advanced features and optimization - 32 weeks
- Clear success criteria and deliverables for each phase

#### **Technology Stack Finalization**
- **LLM Providers**: `rig` crate for unified multi-provider access
- **Scripting**: `mlua` (Lua 5.4), `boa`/`quickjs` (JavaScript), `pyo3` (Python)
- **Storage**: `sled` (development) / `rocksdb` (production)
- **Async Runtime**: `tokio` with cooperative scheduling for script engines
- **Testing**: `mockall` + `proptest` + `criterion` comprehensive stack

#### **Revolutionary Multi-Language Architecture**
- **Identical APIs** across Lua, JavaScript, and Python
- **Bridge-first design** leveraging best-in-class Rust crates
- **Production infrastructure** built-in from day one
- **40+ built-in tools** across 8 categories
- **Comprehensive agent templates** and workflow patterns

#### **Phase 0 Implementation Readiness**
- **37 specific tasks** with detailed acceptance criteria
- **Complete trait hierarchy** specifications (BaseAgent/Agent/Tool/Workflow)
- **12-crate workspace** structure defined
- **CI/CD pipeline** specifications
- **Zero warnings policy** and quality gates established

### What's Revolutionary About Rs-LLMSpell

#### **The Problem We Solve**
- **Development Velocity Barrier**: Compilation cycles kill AI experimentation
- **Orchestration Complexity**: Multi-agent workflows need sophisticated coordination  
- **Language Lock-in**: Teams forced into single-language ecosystems
- **Production Readiness Gap**: Research frameworks lack production infrastructure
- **Integration Fragmentation**: Each provider requires custom integration code

#### **Our Solution**
- **🚀 10x Faster Development**: No compilation cycles for AI workflow changes
- **🔧 Production Ready**: Built-in hooks, events, monitoring, and security
- **🌐 Language Agnostic**: Same capabilities across Lua, JavaScript, Python
- **⚡ High Performance**: Rust core with zero-cost abstractions
- **🛡️ Enterprise Security**: Comprehensive threat model and mitigations
- **🔌 Flexible Integration**: Standalone framework or native library

### Project Status

#### **Completed Phases (1-13)** ✅
- ✅ **Phase 1-11**: Comprehensive research and technology evaluation
- ✅ **Phase 12**: Architecture synthesis and manual review
- ✅ **Phase 13**: Implementation roadmap definition

#### **Current Focus: Phase 0** 🚀
- **Goal**: Foundation Infrastructure (core traits, workspace, CI/CD)
- **Timeline**: 2 weeks (10 working days)
- **Priority**: CRITICAL (MVP prerequisite)
- **Tasks**: 37 specific implementation tasks

#### **Success Criteria for Phase 0**
- [ ] All crates compile without warnings
- [ ] Basic trait hierarchy compiles with full documentation
- [ ] CI runs successfully on Linux with comprehensive test suite
- [ ] Documentation builds without errors (>95% coverage)
- [ ] `cargo test` passes for all foundation tests (>90% coverage)

### Technical Highlights

#### **Component Hierarchy**
```
BaseAgent ← Agent ← SpecializedAgent (Research, Analysis, etc.)
    ↑
  Tool ← ToolWrappedAgent (Agents as Tools)  
    ↑
Workflow ← Sequential, Parallel, Conditional, Loop
```

#### **Multi-Language Bridge Architecture**
```
Script Layer (Lua/JS/Python) ← Bridge Layer ← Core Traits ← Infrastructure
```

#### **Production-Ready Infrastructure**
- **Hook System**: 20+ hook points for logging, metrics, security
- **Event Bus**: Async event emission/subscription for coordination
- **State Management**: Persistent agent state with transaction support
- **Security Model**: Comprehensive sandboxing and threat mitigation
- **Observability**: Structured logging, metrics, distributed tracing

### Files Added/Modified

#### **Documentation**
- ✅ `docs/technical/master-architecture-vision.md` - Complete 15,034+ line architecture
- ✅ `docs/in-progress/implementation-phases.md` - 16-phase roadmap
- ✅ `docs/in-progress/phase-00-design-doc.md` - Detailed Phase 0 specifications
- ✅ `docs/in-progress/PHASE00-TODO.md` - 37 implementation tasks
- ✅ `TODO.md` - Current Phase 0 task tracking
- ✅ `TODO-DONE.md` - Completed phases log
- ✅ `TODO-ARCHIVE.md` - Historical completion records

#### **Project Configuration**
- ✅ `README.md` - Revolutionary framework overview and architecture complete status
- ✅ `CLAUDE.md` - Phase 0 implementation guidance and development workflow
- ✅ `CHANGELOG.md` - This file documenting architecture completion

### Next Steps

#### **Immediate: Phase 0 Implementation** (Next 2 weeks)
1. **Workspace Setup**: 12-crate Cargo workspace with dependencies
2. **Core Traits**: BaseAgent/Agent/Tool/Workflow trait hierarchy 
3. **Error Handling**: Comprehensive error system with categorization
4. **Testing Infrastructure**: mockall + proptest + criterion setup
5. **CI/CD Pipeline**: GitHub Actions with quality gates
6. **Documentation**: >95% API documentation coverage

#### **Upcoming: Phase 1-3 MVP** (Weeks 3-10)
- **Phase 1**: Agent implementations with LLM provider integration
- **Phase 2**: Tool ecosystem with 40+ built-in tools
- **Phase 3**: Workflow orchestration with parallel/conditional execution

### Development Philosophy

#### **Zero Warnings Policy**
- All code must compile without warnings
- Clippy lints at deny level
- Comprehensive error handling for all failure modes

#### **Documentation First**
- Every component documented before implementation
- Code examples that compile and run
- >95% documentation coverage requirement

#### **Test-Driven Foundation**
- Core traits tested before implementation
- Unit, integration, property-based, and performance tests
- >90% test coverage requirement

#### **Bridge-First Design**
- Leverage existing Rust crates rather than reimplementing
- Standing on the shoulders of giants
- Focus on composition and integration

### Breaking Changes

None yet - this is the initial architecture completion. Breaking changes will be documented here as the project evolves through implementation phases.

### Security

#### **Phase 0 Security Model**
- All traits require `Send + Sync` for safe concurrency
- Resource limits on all component operations
- Sanitized error messages without sensitive data
- Security context and permission model built into core traits

### Performance

#### **Phase 0 Performance Targets**
- Clean build: < 60 seconds
- Trait method dispatch: < 1μs overhead
- Error creation/propagation: < 100μs
- Component validation: < 1ms

### Contributors

- **Architecture Team**: Complete research and design (Phases 1-13)
- **Foundation Team**: Ready to begin Phase 0 implementation

---

**🧙‍♂️ Ready to cast the first spell?** 

Rs-LLMSpell has completed its architectural journey and is ready to transform AI development through scriptable, multi-language orchestration. The foundation is solid, the design is complete, and implementation begins now.

**Architecture Complete - Implementation Ready** 🚀

---

## How to Read This Changelog

- **[Unreleased]** - Changes in development or ready for next release
- **Version Numbers** - Will follow semantic versioning once Phase 0 is complete
- **Dates** - All dates in YYYY-MM-DD format
- **Categories** - Added, Changed, Deprecated, Removed, Fixed, Security

The first official release will be **v0.1.0** upon Phase 0 completion, marking the foundation infrastructure as ready for Phase 1 development.