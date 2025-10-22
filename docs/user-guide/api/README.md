# LLMSpell API Documentation

**Complete API reference for scripting and extending LLMSpell**

**🔗 Navigation**: [← User Guide](../) | [Project Home](../../../) | [Examples](../../../examples/)

---

## Overview

> **🔍 API Hub**: Comprehensive documentation for both Lua scripting APIs and Rust extension APIs. Choose your path based on whether you're writing scripts or building components.

**Version**: 0.12.0 | **Status**: Phase 12 Complete | **Last Updated**: October 2025

## Documentation Structure

### 📘 [Lua API](./lua/README.md)
**Complete Lua API Reference** - Comprehensive documentation for all Lua scripting capabilities.

**Coverage:**
- 18 Global objects (Agent, Tool, Workflow, State, Session, Hook, Event, Config, Provider, Debug, JSON, Template, ARGS, Streaming, Artifact, Replay, RAG, Metrics)
- 200+ Methods with full type signatures
- Complete parameter and return type documentation
- Error conditions and handling patterns
- Practical examples for every API method

**Key Globals:**
- `Agent` - LLM agent creation and execution (25+ methods)
- `Tool` - Tool invocation and management (15+ methods)
- `Workflow` - Workflow orchestration patterns (20+ methods)
- `Template` - Production-ready AI workflow templates (6 methods) **Phase 12**
- `RAG` - Retrieval-Augmented Generation (9+ methods) **Phase 8**
- `State` - Persistent state management (15+ methods)
- `Session` - Session and artifact handling (20+ methods)
- `Hook` - Lifecycle hooks and interception (10+ methods)
- `Event` - Event emission and subscription (15+ methods)
- `Debug` - Debugging and profiling utilities (12+ methods)

### 📙 [Rust API](./rust/README.md)
**Complete Rust API Reference** - Comprehensive documentation for extending LLMSpell with Rust.

**Coverage:**
- 18 crates fully documented with traits, implementations, and examples
- Core traits (BaseAgent, Executable, Agent, Tool, Workflow, Template)
- Builder patterns for all components
- Component Registry system
- Complete error type hierarchy
- Bridge APIs for language integration
- Testing utilities and macros
- Performance optimization guidelines

**Key Crates by Phase:**

**Phase 12 - Production-Ready AI Workflow Templates:**
- `llmspell-templates` - Template system with 10 built-in workflows
  - Template trait and registry for discovery and execution
  - ExecutionContext for infrastructure dependency injection
  - TemplateParams and TemplateOutput for standardized interfaces
  - 6 template categories (Research, Chat, Analysis, CodeGen, Document, Workflow)
  - Cost estimation and parameter validation
  - CLI and Lua API integration via TemplateBridge

**Phase 10 - Production Kernel with Daemon Support:**
- `llmspell-kernel` - Unified kernel with integrated state, sessions, debugging, and daemon support
  - IntegratedKernel architecture with global IO runtime
  - Daemon management with double-fork and signal handling
  - KernelHandle with template protocol support (Phase 12)
  - Protocol servers (Jupyter, DAP) with multi-client support
  - Consolidated state persistence and session management
  - Debug infrastructure with DAP bridge
  - Service deployment (systemd/launchd)

**Phase 8 - Enhanced RAG & Multi-Tenancy:**
- `llmspell-storage` - HNSW vector storage with 70% cost optimization
- `llmspell-rag` - RAG pipeline with session collections and bi-temporal queries
- `llmspell-tenancy` - Multi-tenant isolation with resource quotas and billing

**Core Infrastructure:**
- `llmspell-core` - Foundation traits and types
- `llmspell-utils` - Security and utilities
- `llmspell-testing` - Test framework

**Security:**
- `llmspell-security` - Security and access control

**AI & Execution:**
- `llmspell-agents` - Agent framework
- `llmspell-providers` - LLM provider integrations
- `llmspell-workflows` - Workflow orchestration
- `llmspell-tools` - 37+ built-in tools
- `llmspell-hooks` - Lifecycle hooks (40+ hook points)
- `llmspell-events` - Event bus (90K events/sec)

**Integration:**
- `llmspell-bridge` - Lua/script integration (<1% overhead)
- `llmspell-config` - Configuration system
- `llmspell-cli` - Command-line interface with daemon support

## 🆕 What's New in Phase 12

### Production-Ready AI Workflow Templates
- **10 Built-in Templates**: research-assistant, interactive-chat, data-analysis, code-generator, document-processor, workflow-orchestrator, code-review, content-generation, file-classification, knowledge-management
- **Template Registry**: Discovery, search, and execution infrastructure
- **ExecutionContext**: Dependency injection for templates (tools, agents, workflows, RAG, state, sessions)
- **Template Global**: 16th Lua global with 6 methods (list, info, execute, search, schema, estimate_cost)
- **CLI Commands**: `llmspell template list|info|exec|search|schema`
- **KernelHandle Integration**: Interactive/REPL support via kernel protocol (template_request/template_reply)

### Template System Architecture
- **llmspell-templates**: New 18th crate (2,651 LOC template code, 149 tests)
- **6 Template Categories**: Research, Chat, Analysis, CodeGen, Document, Workflow
- **Performance**: 20-50x faster than targets (template init <2ms, registry lookup <1ms)
- **Zero-Day Productivity**: Solve "0-day retention problem" with immediate-use templates
- **Cost Estimation**: Pre-execution cost estimation for LLM usage budgeting

### Phase 12 Statistics
- **Timeline**: 20 days (Oct 5-24, 2025) vs 10 planned
- **Code**: 2,651 LOC template implementations
- **Tests**: 149 total (122 unit + 27 integration)
- **Documentation**: 3,655 lines (Rust API + Lua API + user guides)
- **Templates**: 10 production-ready workflows solving real problems

---

## 🏛️ Previous Milestones

### Phase 10: Production-Ready Daemon Support
- **System Service Deployment**: Deploy as systemd (Linux) or launchd (macOS) service
- **Signal Handling**: SIGTERM/SIGINT for graceful shutdown, SIGHUP for config reload
- **Integrated Kernel Architecture**: Unified crate with global IO runtime
- **18 Crates Total**: Now includes llmspell-templates (Phase 12)

## Quick Start

### Running as Service

**systemd (Linux):**
```bash
# Install service
llmspell kernel install-service --type systemd

# Start service
sudo systemctl start llmspell-kernel
sudo systemctl enable llmspell-kernel

# Check status
sudo systemctl status llmspell-kernel
```

**launchd (macOS):**
```bash
# Install service
llmspell kernel install-service --type launchd

# Load and start
launchctl load ~/Library/LaunchAgents/com.llmspell.kernel.plist
launchctl start com.llmspell.kernel
```

### Rust Development
```rust
use llmspell_kernel::{IntegratedKernel, ExecutionConfig};
use llmspell_agents::{Agent, AgentBuilder};
use llmspell_tools::ToolRegistry;

// Create agent with integrated kernel
let agent = AgentBuilder::new()
    .name("assistant")
    .model("openai/gpt-4")
    .build()?;

let response = agent.execute("Hello, world!").await?;
```

### Lua Scripting
```lua
local agent = Agent.builder()
    :name("assistant")
    :model("openai/gpt-4")
    :build()

local response = agent:execute({
    prompt = "Hello, world!"
})
```

## API Parity

Both Rust and Lua APIs provide identical functionality:

| Feature | Rust API | Lua API |
|---------|----------|---------|
| Agent Creation | `AgentBuilder` | `Agent.builder()` |
| Tool Invocation | `ToolRegistry::invoke()` | `Tool.execute()` |
| Workflows | `WorkflowBuilder` | `Workflow.new()` |
| State Management | `KernelState` | `State` module |
| Session Management | `SessionManager` | `Session` module |
| Debug Support | `ExecutionManager` | `Debug` module |
| Async Operations | `async/await` | Promises/callbacks |
| Error Handling | `Result<T, E>` | `nil, error` returns |

## Design Principles

### 1. Production-Ready
- Daemon mode for service deployment
- Signal handling for graceful operations
- PID management for process control
- Health monitoring endpoints

### 2. Unified Architecture
- Single kernel crate for all runtime needs
- Integrated state and session management
- Consolidated debugging infrastructure
- Global IO runtime for stability

### 3. Multi-Protocol Support
- Jupyter protocol for notebooks
- DAP for IDE debugging
- Extensible protocol/transport traits
- Multi-client message routing

### 4. Performance
- Zero-cost abstractions in Rust
- Minimal overhead in script bridge (<1%)
- Efficient state management
- Fast-path operations for ephemeral data

## Common Patterns

### Service Deployment
Deploy LLMSpell kernel as a production service:

**Daemon Mode:**
```bash
# Start as daemon
llmspell kernel start --daemon --port 9555 --pid-file /var/run/llmspell/kernel.pid

# Check status
llmspell kernel status

# Send signal
llmspell kernel signal SIGHUP  # Reload config
```

### Builder Pattern
Both APIs use builder pattern for complex object creation:

**Rust:**
```rust
let agent = AgentBuilder::new()
    .name("agent")
    .temperature(0.7)
    .build()?;
```

**Lua:**
```lua
local agent = Agent.builder()
    :name("agent")
    :temperature(0.7)
    :build()
```

### Error Handling
Consistent error handling across languages:

**Rust:**
```rust
match agent.execute(prompt).await {
    Ok(response) => println!("{}", response),
    Err(e) => eprintln!("Error: {}", e),
}
```

**Lua:**
```lua
local response, err = agent:execute({prompt = prompt})
if not response then
    print("Error:", err)
end
```

### Async Operations
Both APIs support asynchronous operations:

**Rust:**
```rust
let response = agent.execute_async(prompt).await?;
```

**Lua:**
```lua
agent:execute_async({prompt = prompt})
    :then(function(response)
        print(response)
    end)
```

## Architecture Overview

### Crate Organization (17 Total)

```
llmspell-kernel (Phase 10)
├── State Management (merged)
├── Session Management (merged)
├── Debug Infrastructure
├── Daemon Support
├── Protocol Servers
└── Global IO Runtime

llmspell-core
├── BaseAgent trait
├── Execution Context
└── Component Metadata

llmspell-bridge
├── Lua Integration
├── Type Conversion
└── Global Injection

llmspell-storage → llmspell-rag → llmspell-tenancy
    (Vector Storage)  (RAG Pipeline)  (Multi-tenant)
```

## Performance Characteristics

### Phase 10 Achievements
- **17 crates** (consolidated from 20)
- **37+ integration tests** validating all features
- **<5ms message handling** latency
- **<50MB memory overhead** for kernel
- **100% test coverage** for daemon operations
- **5 comprehensive daemon tests** for signals

### Rust API
- Zero-cost abstractions
- Compile-time optimizations
- Direct memory management
- Global IO runtime (no dispatch errors)

### Lua API
- JIT compilation with LuaJIT
- Minimal bridge overhead (<1%)
- Efficient C bindings
- Fast global injection pattern

## Testing

### Rust Testing
```bash
# Run all tests including integration
cargo test --workspace --all-features

# Run quality checks
./scripts/quality-check-minimal.sh  # Quick checks
./scripts/quality-check-fast.sh     # With tests
./scripts/quality-check.sh          # Comprehensive
```

### Lua Testing
```lua
-- Run test suite
require("llmspell.test").run_all()
```

## Examples

### Complete Examples
- [Rust Examples](../../../examples/rust-developers/)
- [Lua Examples](../../../examples/script-users/)
- [Application Examples](../../../examples/script-users/applications/)

### Production Deployment
- [Service Deployment Guide](../../service-deployment.md)
- [IDE Integration Guide](../../ide-integration.md)

### Code Snippets
Both API documentations include extensive code examples for every function and pattern.

## Support

### Resources
- [User Guide](../../README.md)
- [Configuration Guide](../../configuration.md)
- [Service Deployment](../../service-deployment.md)
- [IDE Integration](../../ide-integration.md)
- [Troubleshooting](../../troubleshooting.md)

### Community
- GitHub Issues: Report bugs and request features
- Discussions: Ask questions and share experiences
- Contributing: See [CONTRIBUTING.md](../../../CONTRIBUTING.md)

## Version Compatibility

| LLMSpell Version | Rust Edition | Lua Version | API Stability |
|-----------------|--------------|-------------|---------------|
| 0.9.0 | 2021 | 5.1+ / LuaJIT | Production |
| 0.8.x | 2021 | 5.1+ / LuaJIT | Stable |
| 0.7.x | 2021 | 5.1+ | Beta |
| 0.6.x | 2021 | 5.1+ / LuaJIT | Beta |

## Migration Notes

### From 0.8.x to 0.9.0
- **Kernel Consolidation**: `llmspell-state-persistence`, `llmspell-state-traits`, and `llmspell-sessions` merged into `llmspell-kernel`
- **Import Changes**: Update imports from separate crates to `llmspell_kernel`
- **API Unchanged**: All public APIs remain compatible
- **New Features**: Daemon mode, signal handling, service integration

## License

See [LICENSE](../../../LICENSE) for licensing information.

---

**Need Help?** Check the [Troubleshooting Guide](../../troubleshooting.md) or [open an issue](https://github.com/yourusername/rs-llmspell/issues).