# The Complete rs-llmspell Developer Guide

✅ **CURRENT**: Phase 8 Complete - RAG & Vector Storage
**Version**: 0.8.0 | **Crates**: 20 | **Tools**: 37+ | **Examples**: 60+

**Quick Navigation**: [Setup](#setup) | [Architecture](#architecture) | [Core Patterns](#core-patterns) | [Testing](#testing) | [Common Tasks](#common-tasks) | [Deep Dives](#deep-dives)

---

## 🎯 Developer Quick Start (5 minutes)

### Setup
```bash
# 1. Clone and build
git clone <repository-url> && cd rs-llmspell
cargo build --release

# 2. Verify setup - MANDATORY quality checks
./scripts/quality-check-minimal.sh  # <5 seconds - format, clippy, compile
./scripts/quality-check-fast.sh     # ~1 min - adds unit tests & docs
./scripts/quality-check.sh          # 5+ min - full validation

# 3. Run example to verify
./target/debug/llmspell run examples/script-users/getting-started/00-hello-world.lua
```

### Your First Contribution

**Choose your path based on the 60+ examples:**

| I want to... | Start here | Example to Study | Time |
|-------------|------------|------------------|------|
| Add a new tool | [Tool Development](#developing-a-tool) | `examples/rust-developers/custom-tool-example/` | 30 min |
| Create an agent | [Agent Development](#developing-an-agent) | `examples/rust-developers/custom-agent-example/` | 45 min |
| Build RAG features | [RAG Extension](#rag-system-phase-8) | `examples/script-users/cookbook/rag-multi-tenant.lua` | 60 min |
| Fix a bug | [Bug Fix Workflow](#bug-fix-workflow) | Test patterns in `llmspell-testing` | 15 min |
| Add tests | [Testing Guide](#testing) | `examples-standards.md` | 10 min |

---

## 📚 Essential Knowledge

### Phase 8 Architecture (20 Crates)

```
┌─────────────────────────────────────────────────────────┐
│                   Lua/JS Scripts (Sync)                 │
│            60+ Examples in examples/*                   │
└─────────────────────────────────────────────────────────┘
                            ↓
┌─────────────────────────────────────────────────────────┐
│          Bridge Layer (sync_utils::block_on_async)      │
│                    llmspell-bridge                      │
└─────────────────────────────────────────────────────────┘
                            ↓
┌─────────────────────────────────────────────────────────┐
│              Rust Core (Async/Await)                    │
│                  20 Specialized Crates                  │
└─────────────────────────────────────────────────────────┘

Foundation Layer (10 crates):
├── llmspell-core         - BaseAgent trait, types
├── llmspell-utils        - Parameter extraction, error builders, response
├── llmspell-storage      - HNSW vector storage (Phase 8)
├── llmspell-security     - 3-level security model
├── llmspell-config       - Configuration management
├── llmspell-state-traits - State abstractions
├── llmspell-state-persistence - Persistent state
├── llmspell-rag          - RAG pipeline (Phase 8)
├── llmspell-tenancy      - Multi-tenant isolation (Phase 8)
└── llmspell-testing      - Centralized test utilities

Application Layer (10 crates):
├── llmspell-tools        - 37+ built-in tools
├── llmspell-agents       - Agent infrastructure
├── llmspell-workflows    - Sequential/Parallel/Conditional/Loop
├── llmspell-bridge       - Script language integration
├── llmspell-hooks        - 40+ hook points, <2% overhead
├── llmspell-events       - Event bus system
├── llmspell-sessions     - Session management
├── llmspell-providers    - LLM provider integration
├── llmspell-cli          - Command line interface
└── llmspell-examples     - Example utilities
```

### Core Concepts (Must Know)

1. **BaseAgent Trait**: Everything implements BaseAgent (tools, agents, workflows)
2. **Sync Bridge Pattern**: Lua/JS are sync, Rust is async - bridge with `block_on_async()`
3. **llmspell-utils Patterns**: Parameter extraction, error builders, response builders
4. **Security Levels**: Safe, Restricted, Privileged - every tool must declare
5. **Test Categories**: unit, integration, external - always categorize
6. **RAG System (Phase 8)**: Vector storage, embeddings, multi-tenant isolation

---

## 🛠️ Core Patterns

### llmspell-utils: The Foundation (NEW SECTION)

**CRITICAL**: These patterns are used EVERYWHERE but were previously undocumented.

#### Parameter Extraction Pattern

```rust
use llmspell_utils::params::{
    extract_parameters,
    extract_required_string,
    extract_optional_string,
    extract_required_bool,
    extract_optional_u64,
    extract_required_object,
};

impl Tool for MyTool {
    async fn execute(&self, input: AgentInput, _ctx: ExecutionContext) 
        -> Result<AgentOutput> {
        // 1. Extract parameters object
        let params = extract_parameters(&input)?;
        
        // 2. Extract specific parameters with validation
        let operation = extract_required_string(params, "operation")?;
        let path = extract_required_string(params, "path")?;
        let recursive = extract_bool_with_default(params, "recursive", false);
        let timeout = extract_optional_u64(params, "timeout_ms");
        
        // 3. Parameters are validated with proper error messages
        // No need for manual validation boilerplate!
        
        // Your tool logic here...
    }
}
```

#### Error Building Pattern

```rust
use llmspell_utils::error_builders::llmspell::{
    component_error,
    validation_error,
    security_error,
};

// Component-level errors
return Err(component_error(format!("Failed to connect: {}", e)));

// Validation errors with field context
return Err(validation_error(
    "Path must be absolute",
    Some("path".to_string())
));

// Security violations
return Err(security_error("Access denied to system directory"));
```

#### Response Building Pattern

```rust
use llmspell_utils::response::ResponseBuilder;

// Success response
let response = ResponseBuilder::success("file_read")
    .with_result(json!({ "content": file_content }))
    .with_metadata("file_size", json!(file_size))
    .with_duration_ms(elapsed.as_millis() as u64)
    .build();

// Error response
let response = ResponseBuilder::error("file_read", "File not found")
    .with_error_details(ErrorDetails::new("ENOENT")
        .with_code("FILE_NOT_FOUND")
        .with_details(json!({ "path": path })))
    .build();
```

### Synchronous Bridge Pattern

**The heart of script integration - from `sync_utils.rs`:**

```rust
use llmspell_bridge::sync_utils::{block_on_async, block_on_async_lua};

// For general async operations returning Result<T, E>
let result = block_on_async::<_, AgentInstance, LLMSpellError>(
    "agent_create",  // Operation name for debugging
    async move {
        bridge.create_agent(model_spec, config_json).await
    },
    Some(Duration::from_secs(30)),  // Optional timeout
)?;

// For operations returning LuaResult<LuaValue>
let result = block_on_async_lua(
    &format!("tool_execute_{}", tool_name),
    async move {
        // Tool execution logic that returns LuaResult
    },
    timeout,
)?;
```

**Key Features:**
- Panic safety with `catch_unwind`
- Timeout support for long operations
- Consistent error messages
- Multi-threaded tokio runtime required

### Test Development Patterns

#### Test Categorization (MANDATORY)

```rust
#[test]
#[cfg_attr(test_category = "unit")]        // Speed: unit/integration/external
#[cfg_attr(test_category = "tool")]        // Component: tool/agent/workflow/etc
fn test_file_reader_basic() {
    // Use llmspell-testing helpers
    use llmspell_testing::tool_helpers::create_test_tool;
    
    let tool = create_test_tool("file_reader", "Reads files", vec![
        ("path", "string"),
    ]);
    // Test logic...
}

#[tokio::test]
#[cfg_attr(test_category = "external")]
#[cfg_attr(test_category = "tool")]
#[ignore = "external"]  // Skip in CI
async fn test_real_api_call() {
    // Real external dependency test
}
```

#### Using llmspell-testing Helpers (NO DUPLICATES)

```rust
use llmspell_testing::{
    // Tool testing
    tool_helpers::{create_test_tool, create_test_tool_input, MockTool},
    
    // Agent testing  
    agent_helpers::{AgentTestBuilder, create_mock_provider_agent},
    
    // Workflow testing
    workflow_helpers::{create_test_workflow_step, create_test_sequential_workflow},
    
    // State testing
    state_helpers::{create_test_state_manager, create_test_memory_backend},
};
```

---

## 🚀 Common Tasks

### Developing a Tool

**Study**: `examples/rust-developers/custom-tool-example/` and `llmspell-tools/src/web/web_scraper.rs`

```rust
use llmspell_core::traits::{BaseAgent, Tool, SecurityLevel, ToolCategory};
use llmspell_utils::params::{extract_parameters, extract_required_string};
use llmspell_utils::response::ResponseBuilder;

#[derive(Clone)]
pub struct MyTool {
    metadata: ComponentMetadata,
}

impl BaseAgent for MyTool {
    fn metadata(&self) -> &ComponentMetadata { &self.metadata }
    
    async fn execute(&self, input: AgentInput, _ctx: ExecutionContext) 
        -> Result<AgentOutput> {
        // Use llmspell-utils for parameter extraction
        let params = extract_parameters(&input)?;
        let operation = extract_required_string(params, "operation")?;
        
        // Do work
        let result = self.do_operation(operation).await?;
        
        // Use ResponseBuilder for consistent output
        let response = ResponseBuilder::success("my_tool")
            .with_result(json!(result))
            .build();
            
        Ok(AgentOutput::tool_result(response))
    }
}

impl Tool for MyTool {
    fn category(&self) -> ToolCategory { ToolCategory::Utility }
    fn security_level(&self) -> SecurityLevel { SecurityLevel::Safe }
}
```

### Developing an Agent

**Study**: `examples/rust-developers/custom-agent-example/src/main.rs`

```rust
use llmspell_core::traits::BaseAgent;

pub struct MyAgent {
    metadata: ComponentMetadata,
}

impl BaseAgent for MyAgent {
    fn metadata(&self) -> &ComponentMetadata { &self.metadata }
    
    async fn execute(&self, input: AgentInput, ctx: ExecutionContext) 
        -> Result<AgentOutput> {
        // Agent logic here
        Ok(AgentOutput::text("Response"))
    }
    
    async fn validate_input(&self, input: &AgentInput) -> Result<()> {
        // Input validation
        Ok(())
    }
    
    async fn handle_error(&self, error: LLMSpellError) -> Result<AgentOutput> {
        // Error recovery
        Ok(AgentOutput::text(format!("Error: {}", error)))
    }
}
```

### Bug Fix Workflow

```bash
# 1. Find the bug location
rg "error message" --type rust

# 2. Write failing test first (with categorization!)
#[test]
#[cfg_attr(test_category = "unit")]
#[cfg_attr(test_category = "tool")]
fn test_bug_reproduction() {
    // Should fail before fix
}

# 3. Fix and verify
cargo test -p <crate> --lib
./scripts/quality-check-minimal.sh

# 4. Update TODO.md if tracked
```

---

## 🧪 Testing

### Quick Test Commands

```bash
# During development (seconds)
cargo test -p <crate> --lib          # Unit tests only
./scripts/test-by-tag.sh unit        # All unit tests

# Before commit (1 minute) - MANDATORY
./scripts/quality-check-fast.sh      # Format + clippy + unit tests

# Before PR (5 minutes) - MANDATORY
./scripts/quality-check.sh           # Everything including integration

# External tests (when needed)
./scripts/test-by-tag.sh external    # Real API calls
```

### Test Organization Best Practices

1. **Always categorize** (speed + component)
2. **Use descriptive names**: `test_unit_file_reader_handles_empty_path`
3. **Group related tests** in modules
4. **Mock external dependencies** for integration tests
5. **Use llmspell-testing helpers** - never duplicate

### Performance Requirements

Your code MUST meet these targets:

| Operation | Target | Measure Command |
|-----------|--------|-----------------|
| Tool init | <10ms | `cargo bench -p llmspell-tools` |
| Agent creation | <50ms | `cargo bench -p llmspell-agents` |
| Hook overhead | <2% | Performance tests |
| State operations | <5ms write, <1ms read | `cargo bench -p llmspell-state-persistence` |
| Vector search | <8ms @ 100K vectors | `cargo bench -p llmspell-storage` |

---

## 🆕 RAG System (Phase 8)

### Overview

Phase 8 added comprehensive RAG (Retrieval-Augmented Generation) support:

```rust
use llmspell_rag::prelude::*;

// Multi-tenant vector entry
let entry = VectorEntry::new("doc-1", embeddings)
    .with_scope(StateScope::Custom("tenant:tenant-123".to_string()))
    .with_metadata(metadata);

// Scoped vector query
let query = VectorQuery::new(query_embedding, 10)
    .with_scope(StateScope::Custom("tenant:tenant-123".to_string()))
    .with_threshold(0.8);

// HNSW configuration
let config = HNSWConfig::balanced()  // or ::performance() or ::accuracy()
    .with_m(16)                      // Connectivity parameter
    .with_ef_construction(200)       // Build-time accuracy
    .with_ef_search(100);            // Query-time accuracy
```

### Key Components

1. **llmspell-storage**: HNSW vector storage, <8ms search @ 100K vectors
2. **llmspell-rag**: RAG pipeline, embeddings, chunking
3. **llmspell-tenancy**: Multi-tenant isolation, 3% overhead

### Learning Path

1. Study `examples/script-users/getting-started/05-first-rag.lua`
2. Review `examples/script-users/cookbook/rag-multi-tenant.lua`
3. Examine `llmspell-rag/src/embeddings/` for provider patterns
4. See `llmspell-storage/src/hnsw/` for vector storage

---

## 🔍 Deep Dives

### Example-Driven Learning

**60+ production-quality examples** in `examples/`:

| Category | Count | Start With |
|----------|-------|------------|
| Getting Started | 6 | `00-hello-world.lua` → `05-first-rag.lua` |
| Features | 5 | `tool-basics.lua`, `agent-basics.lua` |
| Cookbook | 11 | `rag-multi-tenant.lua`, `error-handling.lua` |
| Applications | 9 | `webapp-creator/`, `knowledge-base/` |
| Rust Examples | 6 | `custom-tool-example/`, `custom-agent-example/` |

### Specialized Guides

Read these for deep expertise:

| Topic | Guide | When to Read |
|-------|-------|--------------|
| Extending Tools/Agents/RAG | [extending-llmspell.md](extending-llmspell.md) | Building extensions |
| Production Deployment | [production-guide.md](production-guide.md) | Going to production |
| Example Standards | [examples-standards.md](examples-standards.md) | Writing examples |
| Security Model | [security-guide.md](security-guide.md) | Security features |

### Finding Your Way

```bash
# Find implementation
rg "function_name" --type rust

# Find tests
./scripts/test-by-tag.sh <component>

# Check docs
cargo doc --open -p <crate>

# List examples
ls -la examples/script-users/
```

---

## 💡 Best Practices

### DO This
1. **Use llmspell-utils** for parameters, errors, responses
2. **Study examples** before implementing
3. **Categorize all tests** (speed + component)
4. **Run quality checks** before commits
5. **Follow patterns** from similar code

### DON'T Do This
1. **Create test helpers** - Use `llmspell-testing`
2. **Skip categorization** - Always add test attributes
3. **Ignore security** - Declare levels properly
4. **Use unwrap()** - Proper error handling
5. **Duplicate utilities** - Check llmspell-utils first

---

## 📞 Getting Help

- **Examples**: 60+ in `examples/` directory
- **Questions**: GitHub Discussions
- **Bugs**: GitHub Issues
- **Quick search**: `rg "pattern" --type rust`

---

## Summary

**The Fast Path** (what most developers need):

1. Clone, build, run quality checks
2. Study relevant examples (60+ available)
3. Use llmspell-utils patterns (params, errors, responses)
4. Test with proper categorization
5. Run quality checks before commit
6. Submit PR

**Key Phase 8 Additions**:
- 20 crates (not 17) - added llmspell-rag, llmspell-storage, llmspell-tenancy
- HNSW vector storage with <8ms search
- Multi-tenant RAG with 3% overhead
- 60+ production examples including RAG patterns

**Remember**:
- Everything is BaseAgent
- Lua/JS sync ↔ Rust async via block_on_async
- llmspell-utils for ALL parameter/error/response handling
- Categorize ALL tests
- Study examples first

---

*This guide consolidates best practices from 8 separate guides, incorporates Phase 8 RAG features, and provides direct links to the 60+ production examples.*