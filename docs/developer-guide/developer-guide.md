# The Complete rs-llmspell Developer Guide

✅ **CURRENT**: Phase 7 - Single source of truth for developers

**Quick Navigation**: [Setup](#setup) | [First Contribution](#your-first-contribution) | [Common Tasks](#common-tasks) | [Architecture](#architecture) | [Testing](#testing) | [Deep Dives](#deep-dives)

---

## 🎯 Developer Quick Start (5 minutes)

### Setup
```bash
# 1. Clone and build
git clone <repository-url> && cd rs-llmspell
cargo build

# 2. Verify setup
./scripts/quality-check-minimal.sh  # Should pass in <5 seconds

# 3. Run a simple test
cargo test -p llmspell-core --lib
```

### Your First Contribution

**Choose your path:**

| I want to... | Start here | Time |
|-------------|------------|------|
| Add a new tool | [Tool Development](#developing-a-tool) | 30 min |
| Fix a bug | [Bug Fix Workflow](#bug-fix-workflow) | 15 min |
| Add a test | [Testing Guide](#testing) | 10 min |
| Improve docs | [Documentation](#documentation) | 5 min |

---

## 📚 Essential Knowledge

### Core Concepts (Must Know)

1. **BaseAgent Trait**: Everything implements BaseAgent (tools, agents, workflows)
2. **Sync Bridge Pattern**: Lua/JS are sync, Rust is async - we bridge with `block_on_async()`
3. **17 Crates**: Each has a specific purpose, don't mix concerns
4. **Security Levels**: Safe, Restricted, Privileged - every tool must declare
5. **Test Categories**: unit, integration, external - always categorize

### Architecture at a Glance

```
Lua/JS Scripts (Synchronous)
       ↓
Bridge Layer (sync_utils::block_on_async)
       ↓
Rust Core (Async/Await)
       ↓
17 Specialized Crates
```

**Key Crates:**
- `llmspell-core`: Traits and types (foundation)
- `llmspell-bridge`: Script language integration
- `llmspell-tools`: 37 built-in tools
- `llmspell-agents`: Agent infrastructure
- `llmspell-workflows`: Workflow patterns

---

## 🛠️ Common Tasks

### Developing a Tool

**30-minute guide to add a new tool:**

1. **Create tool file** in `llmspell-tools/src/<category>/`

```rust
use llmspell_core::traits::{BaseAgent, Tool, SecurityLevel};
use serde::{Deserialize, Serialize};

#[derive(Clone)]
pub struct MyTool {
    metadata: ComponentMetadata,
}

impl BaseAgent for MyTool {
    fn metadata(&self) -> &ComponentMetadata { &self.metadata }
    
    async fn execute(&self, input: AgentInput, _ctx: ExecutionContext) 
        -> Result<AgentOutput> {
        // Parse input
        let params: MyToolInput = serde_json::from_value(input.data)?;
        
        // Do work
        let result = self.do_something(&params)?;
        
        // Return output
        Ok(AgentOutput::tool_result(result))
    }
}

impl Tool for MyTool {
    fn category(&self) -> ToolCategory { ToolCategory::Utility }
    fn security_level(&self) -> SecurityLevel { SecurityLevel::Safe }
}
```

2. **Register in bridge** at `llmspell-bridge/src/tools.rs`:
```rust
registry.register_tool("my_tool", Arc::new(MyTool::new()));
```

3. **Add tests** with proper categorization:
```rust
#[test]
#[cfg_attr(test_category = "unit")]
#[cfg_attr(test_category = "tool")]
fn test_my_tool_basic() {
    // Test using llmspell_testing helpers
}
```

4. **Verify**:
```bash
cargo test -p llmspell-tools
./scripts/quality-check-fast.sh
```

**Need more details?** See [tool-development-guide.md](tool-development-guide.md) (683 lines)

### Bug Fix Workflow

**15-minute guide to fix a bug:**

1. **Find the bug location**:
```bash
# Search for error message
rg "error message" --type rust

# Find related tests
./scripts/test-by-tag.sh <component>
```

2. **Write failing test first**:
```rust
#[test]
#[cfg_attr(test_category = "unit")]
fn test_bug_reproduction() {
    // This should fail before fix
}
```

3. **Fix and verify**:
```bash
cargo test -p <crate> --lib
./scripts/quality-check-minimal.sh
```

4. **Update TODO.md** if fixing a tracked issue

### Adding an Agent

**Quick template:**

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
}
```

Register in `llmspell-bridge/src/agents.rs`.

### Working with Workflows

**Sequential workflow example:**

```rust
let workflow = Workflow::sequential({
    name: "my_workflow",
    steps: vec![
        WorkflowStep::tool("file_reader", params),
        WorkflowStep::agent("processor", params),
        WorkflowStep::tool("file_writer", params),
    ]
});
```

**Need patterns?** See [workflow-bridge-guide.md](workflow-bridge-guide.md) (499 lines)

---

## 🧪 Testing

### Quick Test Commands

```bash
# During development (seconds)
cargo test -p <crate> --lib          # Unit tests only
./scripts/test-by-tag.sh unit        # All unit tests

# Before commit (1 minute)
./scripts/quality-check-fast.sh      # Format + clippy + unit tests

# Before PR (5 minutes)
./scripts/quality-check.sh           # Everything including integration
```

### Test Categorization (MANDATORY)

Every test needs TWO categories:

```rust
#[test]
#[cfg_attr(test_category = "unit")]     // Speed: unit/integration/external
#[cfg_attr(test_category = "tool")]     // Component: tool/agent/workflow/etc
fn test_something() { }
```

### Using Test Helpers

Always use `llmspell-testing` helpers:

```rust
use llmspell_testing::{
    tool_helpers::create_test_tool,
    agent_helpers::create_mock_agent,
};
```

**Complete testing guide:** [test-development-guide.md](test-development-guide.md) (748 lines)

---

## 🏗️ Architecture

### Crate Dependency Hierarchy

```
Foundation Layer (cannot use llmspell-testing):
├── llmspell-core      - Traits and types
├── llmspell-utils     - Shared utilities
├── llmspell-storage   - Storage abstractions
├── llmspell-security  - Security primitives
└── llmspell-config    - Configuration

Application Layer (use llmspell-testing):
├── llmspell-tools     - 37 tools
├── llmspell-agents    - Agent infrastructure
├── llmspell-workflows - Workflow patterns
├── llmspell-bridge    - Script integration
├── llmspell-hooks     - Hook system
├── llmspell-events    - Event bus
├── llmspell-sessions  - Session management
└── llmspell-cli       - Command line interface
```

### Synchronous Bridge Pattern

**Problem**: Lua/JS are synchronous, Rust async operations need bridging

**Solution**: `sync_utils::block_on_async()`

```rust
// In bridge layer
let result = block_on_async(
    "operation_name",
    async move { 
        // Async Rust operation
        bridge.do_something().await 
    },
    Some(Duration::from_secs(30)),
)?;
```

**Details:** [synchronous-api-patterns.md](synchronous-api-patterns.md) (202 lines)

### Security Model

Every tool declares security requirements:

```rust
impl Tool for MyTool {
    fn security_level(&self) -> SecurityLevel {
        SecurityLevel::Restricted  // or Safe, Privileged
    }
    
    fn security_requirements(&self) -> SecurityRequirements {
        SecurityRequirements::restricted()
            .with_file_access("/tmp")
            .with_network_access("api.example.com")
    }
}
```

**Security guide:** [security-guide.md](security-guide.md) (660 lines)

---

## 📋 Development Workflow

### Daily Development

1. **Start work**:
```bash
git checkout -b feature/my-feature
```

2. **Make changes with fast feedback**:
```bash
# Quick test while coding
cargo test -p <crate> test_function_name

# Before commit
./scripts/quality-check-minimal.sh
```

3. **Commit with confidence**:
```bash
./scripts/quality-check-fast.sh
git commit -m "feat: add awesome feature"
```

4. **Before PR**:
```bash
./scripts/quality-check.sh
```

### Performance Targets

Your code must meet these targets:

| Operation | Target | How to Measure |
|-----------|--------|----------------|
| Tool init | <10ms | `cargo bench -p llmspell-tools` |
| Agent creation | <50ms | `cargo bench -p llmspell-agents` |
| State operations | <5ms | `cargo bench -p llmspell-state-persistence` |
| Hook overhead | <2% | Performance tests in hooks |

### Code Quality Standards

- **Zero warnings**: `cargo clippy -- -D warnings`
- **Formatted**: `cargo fmt --all`
- **Documented**: Public APIs need docs
- **Tested**: New features need tests
- **Secure**: Tools declare security level

---

## 🔍 Deep Dives

### Specialized Guides

Only read these when you need deep knowledge:

| Topic | Guide | When to Read |
|-------|-------|--------------|
| Custom Hooks | [hook-development-guide.md](hook-development-guide.md) | Building hook plugins |
| Session System | [session-artifact-implementation.md](session-artifact-implementation.md) | Working on sessions |
| Advanced Testing | [test-development-guide.md](test-development-guide.md) | Test infrastructure work |
| Workflow Patterns | [workflow-bridge-guide.md](workflow-bridge-guide.md) | Complex workflows |
| Security Hardening | [security-guide.md](security-guide.md) | Security features |
| Tool Development | [tool-development-guide.md](tool-development-guide.md) | Advanced tool patterns |

### Finding Your Way

```bash
# Find where something is implemented
rg "function_name" --type rust

# Find tests for a component
./scripts/test-by-tag.sh <component>

# Check documentation
cargo doc --open -p <crate>

# Get performance metrics
cargo bench -p <crate>
```

---

## 🚫 Common Pitfalls

### Don't Do This

1. **Creating test helpers** - Use `llmspell-testing`
2. **Skipping categorization** - Always add test categories
3. **Ignoring security** - Every tool needs security level
4. **Raw unwrap()** - Use proper error handling
5. **Blocking in async** - Use `spawn_blocking` for CPU work

### Do This Instead

1. **Use existing patterns** - Copy from similar code
2. **Test first** - Write failing test before fix
3. **Small commits** - One logical change per commit
4. **Update docs** - Keep documentation in sync
5. **Ask questions** - Use discussions for design questions

---

## 💡 Tips for Success

### For New Contributors

1. Start with a "good first issue"
2. Read similar code before implementing
3. Use quality check scripts early and often
4. Don't skip test categorization
5. Ask questions in discussions

### For Regular Contributors  

1. Keep TODO.md updated
2. Write ADRs for significant decisions
3. Benchmark performance-critical code
4. Help review PRs
5. Update this guide when things change

---

## 📞 Getting Help

- **Questions**: GitHub Discussions
- **Bugs**: GitHub Issues
- **Security**: security@llmspell.org
- **Quick answers**: Search codebase with `rg`

---

## Summary

**The 80% Path** (what most developers need):

1. Clone, build, test
2. Pick a task (tool/bug/test/docs)
3. Follow the pattern from similar code
4. Test with categorization
5. Run quality checks
6. Submit PR

**Remember**:
- Everything is BaseAgent
- Lua/JS are sync, Rust is async
- Use llmspell-testing helpers
- Categorize all tests
- Security matters

---

## 📐 API Design Guidelines

### Naming Conventions

**Constructors**:
```rust
// ✅ Simple constructors
impl MyTool {
    pub fn new() -> Self { }
}

// ✅ Complex constructors with parameters
impl MyAgent {
    pub fn with_config(config: Config) -> Self { }
    pub fn builder() -> MyAgentBuilder { }  // For complex initialization
}
```

**Accessors & Mutators**:
```rust
// ✅ Getters
fn get_status(&self) -> Status { }
fn status(&self) -> &Status { }        // Immutable borrow
fn name(&self) -> &str { }             // Simple field access

// ✅ Setters  
fn set_status(&mut self, status: Status) { }
fn update_config(&mut self, config: Config) { }
```

**Service Components**:
```rust
// ✅ Manager suffix for coordination services
pub struct SessionManager { }
pub struct StateManager { }
pub struct WorkflowManager { }

// ✅ Registry suffix for collections
pub struct ToolRegistry { }
pub struct AgentRegistry { }

// ✅ Service suffix for single-purpose services
pub struct ValidationService { }
pub struct NotificationService { }
```

### Error Handling

**Standard Result Pattern**:
```rust
use llmspell_core::Result;  // Always use project Result type

// ✅ All fallible operations return Result
pub async fn execute(&self, input: Input) -> Result<Output> {
    // Provide context with errors
    let data = self.load_data().context("Failed to load input data")?;
    
    // Use error chaining
    self.process(data)
        .await
        .with_context(|| format!("Failed to process data for tool {}", self.name()))?
}

// ✅ Convert external errors with context
use anyhow::Context;

fn parse_config(path: &Path) -> Result<Config> {
    let content = std::fs::read_to_string(path)
        .with_context(|| format!("Failed to read config from {}", path.display()))?;
    
    serde_json::from_str(&content)
        .with_context(|| format!("Failed to parse config from {}", path.display()))
}
```

**Error Categories**:
```rust
// ✅ Use appropriate error types
use llmspell_core::error::{LLMSpellError, ValidationError, ResourceError};

// Input validation
if input.is_empty() {
    return Err(ValidationError::empty_input("input cannot be empty").into());
}

// Resource constraints
if size > MAX_SIZE {
    return Err(ResourceError::limit_exceeded("file size", size, MAX_SIZE).into());
}

// Security violations
if !self.security_check(&path) {
    return Err(LLMSpellError::security_violation("path access denied").into());
}
```

### Async Patterns

**Trait Definitions**:
```rust
// ✅ Mark async traits properly
#[async_trait]
pub trait Tool: BaseAgent + Send + Sync {
    async fn execute(&self, input: AgentInput) -> Result<AgentOutput>;
}

// ✅ Document cancellation safety
/// Executes the tool with the given input.
/// 
/// # Cancellation Safety
/// This method is cancellation safe. If cancelled, no persistent state is modified.
async fn execute(&self, input: Input) -> Result<Output>;
```

**Sync Wrappers for Scripts**:
```rust
// ✅ Use sync_utils for bridge layer
use llmspell_bridge::sync_utils::block_on_async;

pub fn execute_sync(&self, input: Input) -> Result<Output> {
    block_on_async(
        "tool_execute",
        async move { self.execute(input).await },
        Some(Duration::from_secs(30)),
    )
}
```

---

## 🤝 Contributing Guide

### Code Style Requirements

**Formatting**:
```bash
# Before every commit
cargo fmt --all

# Check formatting in CI
cargo fmt --all -- --check
```

**Linting**:
```bash
# Zero warnings policy
cargo clippy --workspace --all-targets --all-features -- -D warnings

# Fix common issues
cargo clippy --fix
```

**Documentation**:
```rust
// ✅ Document all public APIs
/// Creates a new tool instance with the given configuration.
/// 
/// # Arguments
/// * `config` - Tool configuration parameters
/// 
/// # Errors
/// Returns an error if configuration is invalid or resources cannot be allocated.
/// 
/// # Examples
/// ```rust
/// let tool = MyTool::with_config(Config::default())?;
/// ```
pub fn with_config(config: Config) -> Result<Self> { }

// ✅ Warn on missing docs
#![warn(missing_docs)]
```

### Testing Requirements

**Test Coverage**:
- **Unit tests**: Every public function
- **Integration tests**: Cross-component functionality
- **External tests**: Real API interactions (marked with `#[ignore = "external"]`)

**Categorization (MANDATORY)**:
```rust
#[test]
#[cfg_attr(test_category = "unit")]        // Speed category
#[cfg_attr(test_category = "tool")]        // Component category
fn test_tool_basic_functionality() {
    // Use llmspell-testing helpers
    let tool = create_test_tool("my_tool", "Test tool", vec![]);
    // Test logic
}
```

**Performance Requirements**:
- Unit tests: <5s per crate
- Integration tests: <30s per crate
- External tests: Can be slow, run separately

### Documentation Standards

**API Documentation**:
- All public APIs documented
- Include examples for complex functions
- Document error conditions
- Explain async behavior and cancellation safety

**Guide Updates**:
- Update relevant guides when adding features
- Keep examples current with API changes
- Update TODO.md for tracked work

### PR Process

**Before Submitting**:
```bash
# 1. Run quality checks
./scripts/quality-check-fast.sh

# 2. Update documentation
# 3. Add/update tests
# 4. Check TODO.md for related tasks
```

**PR Description**:
```markdown
## Summary
Brief description of changes

## Changes
- [ ] Feature implementation
- [ ] Tests added/updated  
- [ ] Documentation updated
- [ ] Performance impact considered

## Testing
Describe testing performed

## Breaking Changes
List any breaking changes
```

**Review Process**:
1. Automated checks must pass
2. Code review by maintainer
3. Performance impact assessment
4. Documentation review

---

## 🔧 Common Patterns

### Registry Pattern

**Implementation**:
```rust
use std::collections::HashMap;
use std::sync::Arc;

pub struct ToolRegistry {
    tools: HashMap<String, Arc<dyn Tool>>,
}

impl ToolRegistry {
    pub fn new() -> Self {
        Self {
            tools: HashMap::new(),
        }
    }
    
    pub fn register<T>(&mut self, name: &str, tool: T) 
    where 
        T: Tool + 'static 
    {
        self.tools.insert(name.to_string(), Arc::new(tool));
    }
    
    pub fn get(&self, name: &str) -> Option<Arc<dyn Tool>> {
        self.tools.get(name).cloned()
    }
    
    pub fn list_names(&self) -> Vec<String> {
        self.tools.keys().cloned().collect()
    }
}
```

**Usage**:
```rust
// Register tools
let mut registry = ToolRegistry::new();
registry.register("file_reader", FileReaderTool::new());
registry.register("calculator", CalculatorTool::new());

// Use tools
let tool = registry.get("file_reader").unwrap();
let result = tool.execute(input).await?;
```

### Factory Pattern

**Builder Pattern for Complex Objects**:
```rust
pub struct AgentBuilder {
    name: Option<String>,
    model: Option<String>,
    temperature: Option<f32>,
    tools: Vec<Arc<dyn Tool>>,
}

impl AgentBuilder {
    pub fn new() -> Self {
        Self {
            name: None,
            model: None,  
            temperature: None,
            tools: Vec::new(),
        }
    }
    
    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }
    
    pub fn model(mut self, model: impl Into<String>) -> Self {
        self.model = Some(model.into());
        self
    }
    
    pub fn add_tool(mut self, tool: Arc<dyn Tool>) -> Self {
        self.tools.push(tool);
        self
    }
    
    pub async fn build(self) -> Result<Agent> {
        let name = self.name.ok_or_else(|| ValidationError::required_field("name"))?;
        let model = self.model.ok_or_else(|| ValidationError::required_field("model"))?;
        
        Agent::create(name, model, self.temperature, self.tools).await
    }
}
```

### State Management Patterns

**State Manager with Persistence**:
```rust
#[async_trait]
pub trait StateManager: Send + Sync {
    async fn get<T>(&self, key: &str) -> Result<Option<T>> 
    where T: DeserializeOwned;
    
    async fn set<T>(&self, key: &str, value: T) -> Result<()>
    where T: Serialize;
    
    async fn delete(&self, key: &str) -> Result<bool>;
}

// Usage with error handling
async fn save_session_state(
    state_manager: &dyn StateManager,
    session_id: &str, 
    state: &SessionState
) -> Result<()> {
    let key = format!("session:{}", session_id);
    state_manager.set(&key, state)
        .await
        .with_context(|| format!("Failed to save state for session {}", session_id))
}
```

### Hook Integration Patterns

**Implementing a Hook**:
```rust
#[async_trait]
impl Hook for MyHook {
    fn name(&self) -> &str { "my_hook" }
    
    fn hook_points(&self) -> Vec<HookPoint> {
        vec![HookPoint::ToolExecute, HookPoint::AgentResponse]
    }
    
    async fn execute(&self, context: &mut HookContext) -> Result<HookAction> {
        match context.hook_point() {
            HookPoint::ToolExecute => {
                // Log tool execution
                tracing::info!("Tool executed: {}", context.tool_name());
                Ok(HookAction::Continue)
            },
            HookPoint::AgentResponse => {
                // Validate response
                if self.validate_response(context.response()) {
                    Ok(HookAction::Continue)
                } else {
                    Ok(HookAction::Block("Invalid response".to_string()))
                }
            },
            _ => Ok(HookAction::Continue),
        }
    }
}
```

**Registering Hooks**:
```rust
// In your application setup
let hook_manager = HookManager::new();
hook_manager.register(Arc::new(MyHook::new())).await?;
hook_manager.register(Arc::new(LoggingHook::new())).await?;
```

---

**When in doubt**: 
- Check similar code
- Run quality-check-fast.sh
- Read the specialized guide

---

*This guide consolidates 4,455 lines from 8 separate guides and adds comprehensive API guidelines (naming, error handling, async patterns), contributing standards (style, testing, PR process), and common patterns (registry, factory, state, hooks) for a complete developer experience.*