# The Complete rs-llmspell Developer Guide

✅ **CURRENT**: Phase 10 Complete - Service Integration & IDE Connectivity
**Version**: 0.10.0 | **Crates**: 17 | **Tools**: 40+ | **Examples**: 60+ | **Feature Flags**: Modular builds (19-35MB)

**Quick Navigation**: [Setup](#setup) | [Architecture](#architecture) | [Core Patterns](#core-patterns) | [Testing](#testing) | [Common Tasks](#common-tasks) | [Deep Dives](#deep-dives)

---

## 🎯 Developer Quick Start (5 minutes)

### Setup & Build Options

rs-llmspell uses a **feature-based build system** to optimize binary size and dependencies. Choose your build based on your needs:

```bash
# 1. Clone repository
git clone <repository-url> && cd rs-llmspell

# 2. Choose your build configuration:

# OPTION A: Minimal Build (19MB) - Recommended for production containers
cargo build --release --bin llmspell
# Includes: Core functionality, Lua scripting, essential tools

# OPTION B: Common Build (25MB) - Recommended for most developers
cargo build --release --bin llmspell --features common
# Adds: Template engines (Tera, Handlebars), PDF processing

# OPTION C: Full Build (35MB) - All features for complete development
cargo build --release --bin llmspell --features full
# Adds: CSV/Parquet, Excel, archives, email, database support

# 3. Verify setup - MANDATORY quality checks
./scripts/quality/quality-check-minimal.sh  # <5 seconds - format, clippy, compile
./scripts/quality/quality-check-fast.sh     # ~1 min - adds unit tests & docs
./scripts/quality/quality-check.sh          # 5+ min - full validation

# 4. Run example to verify
./target/release/llmspell run examples/script-users/getting-started/00-hello-world.lua
```

### Quick Feature Reference

| Feature | Size Impact | Tools Added | Use Case |
|---------|------------|-------------|----------|
| (minimal) | 19MB base | Core tools only | Production, containers |
| `templates` | +400KB | TemplateEngine | Document generation |
| `pdf` | +300KB | PdfProcessor | PDF analysis/extraction |
| `csv-parquet` | +2.8MB | CsvAnalyzer | Data analytics |
| `excel` | +1MB | ExcelHandler | Spreadsheet processing |
| `json-query` | +600KB | JsonQuery (JQ) | Complex JSON ops |
| `archives` | +400KB | ArchiveHandler | ZIP/TAR handling |
| `email` | +500KB | EmailTool (SMTP) | Email notifications |
| `database` | +2MB | Database ops | SQL connectivity |

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

### Phase 10 Architecture (17 Crates)

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
│                  17 Specialized Crates                  │
└─────────────────────────────────────────────────────────┘

Foundation Layer (8 crates):
├── llmspell-core         - BaseAgent trait, types
├── llmspell-utils        - Parameter extraction, error builders, response
├── llmspell-storage      - HNSW vector storage (Phase 8)
├── llmspell-security     - 3-level security model
├── llmspell-config       - Configuration management
├── llmspell-rag          - RAG pipeline (Phase 8)
├── llmspell-tenancy      - Multi-tenant isolation (Phase 8)
└── llmspell-testing      - Centralized test utilities

Application Layer (9 crates):
├── llmspell-kernel       - Daemon, signals, Jupyter, DAP (Phase 10)
├── llmspell-tools        - 40+ built-in tools (feature flags)
├── llmspell-agents       - Agent infrastructure
├── llmspell-workflows    - Sequential/Parallel/Conditional/Loop
├── llmspell-bridge       - Script language integration
├── llmspell-hooks        - 40+ hook points, <2% overhead
├── llmspell-events       - Event bus system
├── llmspell-providers    - LLM provider integration
└── llmspell-cli          - Command line interface + tool commands
```

### Core Concepts (Must Know)

1. **BaseAgent Trait**: Everything implements BaseAgent (tools, agents, workflows)
2. **Sync Bridge Pattern**: Lua/JS are sync, Rust is async - bridge with `block_on_async()`
3. **llmspell-utils Patterns**: Parameter extraction, error builders, response builders
4. **Security Levels**: Safe, Restricted, Privileged - every tool must declare
5. **Test Categories**: unit, integration, external - always categorize
6. **RAG System (Phase 8)**: Vector storage, embeddings, multi-tenant isolation

---

## 📦 Build Configuration & Features

### Feature-Based Build System

rs-llmspell uses **Cargo feature flags** to create optimized binaries. This system reduces binary size from 33.6MB (old default) to as small as 19MB (minimal), while allowing developers to include only the dependencies they need.

### Build Configurations

#### 1. Minimal Build (19MB) - Production Ready
```bash
cargo build --release --bin llmspell
# Or explicitly:
cargo build --release --bin llmspell --no-default-features --features lua
```

**Includes:**
- Core functionality (llmspell-core, utils, bridge)
- Lua scripting support
- Essential tools (file operations, HTTP, shell, text processing)
- State management and persistence
- Hook system and events
- Session management

**Excludes:** Heavy dependencies like Apache Arrow, template engines, PDF processing

**Use Cases:** Production deployments, containers, embedded systems, CI/CD pipelines

#### 2. Common Build (25MB) - Developer Friendly
```bash
cargo build --release --bin llmspell --features common
```

**Adds to Minimal:**
- Template engines (Tera, Handlebars) - document generation
- PDF processing (pdf-extract) - document analysis

**Use Cases:** Most development work, documentation generation, report creation

#### 3. Full Build (35MB) - Complete Toolkit
```bash
cargo build --release --bin llmspell --features full
```

**Adds Everything:**
- CSV/Parquet support (Apache Arrow) - data analytics
- Excel processing (calamine, xlsxwriter)
- Archive handling (ZIP, TAR, GZ)
- Email support (SMTP, AWS SES)
- Database connectivity (PostgreSQL, MySQL, SQLite)
- JSON query engine (JQ implementation)

**Use Cases:** Data science, full-featured development, all examples

### Custom Feature Selection

Mix and match features based on your specific needs:

```bash
# Just add template support to minimal
cargo build --release --features templates

# Data processing focus
cargo build --release --features csv-parquet,excel

# Communication tools
cargo build --release --features email,database

# Multiple features
cargo build --release --features templates,pdf,archives
```

### Feature Flags Reference

| Feature | Dependencies | Binary Impact | Tools Enabled |
|---------|-------------|---------------|---------------|
| `templates` | tera, handlebars | +400KB | TemplateEngineTool |
| `pdf` | pdf-extract | +300KB | PdfProcessorTool |
| `csv-parquet` | arrow, parquet | +2.8MB | CsvAnalyzerTool |
| `excel` | calamine, xlsxwriter | +1MB | ExcelHandlerTool |
| `json-query` | jaq-* crates | +600KB | JsonQueryTool |
| `archives` | zip, tar, flate2 | +400KB | ArchiveHandlerTool |
| `email` | lettre | +500KB | EmailTool (SMTP) |
| `email-aws` | aws-sdk-ses | +1.5MB | EmailTool (AWS) |
| `database` | sqlx | +2MB | Database operations |

### Testing with Features

```bash
# Test minimal configuration
cargo test --no-default-features --features lua

# Test specific feature
cargo test --features templates

# Test everything
cargo test --all-features

# Clippy with features
cargo clippy --features common --all-targets
```

### CI/CD Configuration

#### GitHub Actions
```yaml
strategy:
  matrix:
    features: [minimal, common, full]
steps:
  - name: Build ${{ matrix.features }}
    run: |
      if [ "${{ matrix.features }}" = "minimal" ]; then
        cargo build --release --bin llmspell
      else
        cargo build --release --bin llmspell --features ${{ matrix.features }}
      fi
```

#### Docker Multi-Stage
```dockerfile
# Minimal image (19MB binary)
FROM rust:1.76 as minimal
WORKDIR /app
COPY . .
RUN cargo build --release --bin llmspell

# Common image (25MB binary)
FROM rust:1.76 as common
WORKDIR /app
COPY . .
RUN cargo build --release --features common --bin llmspell

# Runtime
FROM debian:bookworm-slim
COPY --from=minimal /app/target/release/llmspell /usr/local/bin/
```

### Development Workflow

1. **Start with minimal** for core development
2. **Add features as needed** when working on specific tools
3. **Test with minimal** to ensure core functionality
4. **Test with full** before releases
5. **Document feature requirements** in your code

### Feature-Gated Code

When developing tools that require optional dependencies:

```rust
// In llmspell-tools/src/lib.rs
#[cfg(feature = "templates")]
pub mod template_engine;

// In your code
#[cfg(feature = "templates")]
use llmspell_tools::template_engine::TemplateEngineTool;

// Conditional registration
pub fn register_tools(registry: &mut ToolRegistry) {
    #[cfg(feature = "templates")]
    registry.register("template-creator", TemplateEngineTool::new);
}
```

### Runtime Tool Discovery

Tool availability is **automatic** - the runtime discovers available tools:

```lua
-- This always works, showing only available tools
local tools = Tool.list()
for _, name in ipairs(tools) do
    print("Available: " .. name)
end

-- Graceful handling of optional tools
local template = Tool.try_get("template-creator")
if template then
    -- Use template engine
else
    -- Fallback to simple string formatting
end
```

### Performance Comparison

| Build Type | Binary Size | Startup Time | Memory Usage | Tool Count |
|------------|------------|--------------|--------------|------------|
| Minimal | 19MB | 15ms | 12MB | 25 tools |
| Common | 25MB | 18ms | 14MB | 27 tools |
| Full | 35MB | 25ms | 18MB | 37+ tools |
| Old Default | 33.6MB | 23ms | 17MB | 37+ tools |

### Migration from Pre-Feature System

If upgrading from versions before the feature system:

1. **Assess tool usage** - Run with minimal, note missing tools
2. **Choose configuration** - minimal, common, or full
3. **Update build scripts** - Add feature flags to cargo commands
4. **No code changes needed** - API remains identical

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

### Workflow Pattern: Automatic Output Collection

**Problem**: Users need to manually collect agent outputs from state using complex key construction.

**Solution**: All workflow types automatically collect agent outputs during execution.

**Implementation** (Rust):
```rust
// In execute_impl(), after workflow completes:
let mut agent_outputs = serde_json::Map::new();
if let Some(ref state) = context.state {
    for step in &self.steps {
        if let StepType::Agent { agent_id, .. } = &step.step_type {
            let key = format!("workflow:{}:agent:{}:output", execution_id, agent_id);
            if let Ok(Some(output)) = state.read(&key).await {
                agent_outputs.insert(agent_id.clone(), output);
            }
        }
    }
}
if !agent_outputs.is_empty() {
    metadata.extra.insert("agent_outputs".to_string(),
                         serde_json::Value::Object(agent_outputs));
}
```

**Benefits**:
- No manual state key construction
- Batch retrieval (single state access per agent)
- Consistent API across workflow types
- Type-safe access via `result.metadata.extra.agent_outputs`

**Lua Usage**:
```lua
local result = workflow:execute(input)
local outputs = result.metadata and result.metadata.extra
    and result.metadata.extra.agent_outputs or {}

for agent_id, output in pairs(outputs) do
    -- Process agent output
end
```

### Test Development Patterns

#### Test Categorization (MANDATORY)

```rust
#[test]
#[cfg_attr(test_category = "unit")]        // Speed: unit/integration/external
#[cfg_attr(test_category = "tool")]        // Component: tool/agent/workflow/etc
fn test_file_reader_basic() {
    // Use llmspell-testing helpers
    use llmspell_testing::tool_helpers::create_test_tool;

    let tool = create_test_tool("file-reader", "Reads files", vec![
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
        let response = ResponseBuilder::success("my-tool")
            .with_result(json!(result))
            .build()
;
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
./scripts/quality/quality-check-minimal.sh

# 4. Update TODO.md if tracked
```

---

## 🧪 Testing

### Quick Test Commands

```bash
# During development (seconds)
cargo test -p <crate> --lib          # Unit tests only
./scripts/testing/test-by-tag.sh unit        # All unit tests

# Before commit (1 minute) - MANDATORY
./scripts/quality/quality-check-fast.sh      # Format + clippy + unit tests

# Before PR (5 minutes) - MANDATORY
./scripts/quality/quality-check.sh           # Everything including integration

# External tests (when needed)
./scripts/testing/test-by-tag.sh external    # Real API calls
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
| Vector search | <8ms @ 100K vectors | `cargo bench -p llmspell-storage` |
| Multi-tenant | 3% overhead | Integration tests |

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
./scripts/testing/test-by-tag.sh <component>

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