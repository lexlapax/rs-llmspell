# Developer Documentation

Welcome to the rs-llmspell developer documentation! This guide helps contributors understand the codebase, implement new features, and maintain the project.

**🔗 Navigation**: [← Documentation Hub](../README.md) | [Project Home](../../README.md) | [User Guide](../user-guide/) | [Technical Docs](../technical/)

## 🚀 Quick Start for Contributors

### Prerequisites
- **Rust**: 1.70+ with `cargo` and `rustup`
- **Git**: For version control
- **IDE**: VS Code with rust-analyzer recommended

### 5-Minute Setup
```bash
# 1. Clone the repository
git clone <repository-url>
cd rs-llmspell

# 2. Install dependencies and build
cargo build

# 3. Run tests to verify setup
./scripts/quality-check-minimal.sh

# 4. Make a small change and test
cargo test -p llmspell-core --lib
```

**✅ If all commands succeed, you're ready to contribute!**

---

## 📚 Documentation Index

### **Essential Reading**
- **[Contributing Guidelines](../../CONTRIBUTING.md)** ⚠️ *Missing - needs creation*
- **[Architecture Overview](../technical/rs-llmspell-final-architecture.md)** - System design and component relationships

### **Implementation Guides**
- **[Synchronous API Patterns](synchronous-api-patterns.md)** ✅ *Current & Accurate* - Lua/JS bridge patterns
- **[Resource Limits Implementation](implementing-resource-limits.md)** ⚠️ *API Fixed* - Resource tracking in tools
- **[Security Development Guide](security-guide.md)** ⚠️ *API Fixed* - Security best practices

### **Testing & Quality**
- **[Test Organization](test-organization.md)** ✅ *Current & Accurate* - Test categories and execution
- **[Agent Testing Guide](agent-testing-guide.md)** ✅ *Current & Accurate* - Testing agents with mocks and scenarios
- **[Quality Check Scripts](quality-check-scripts.md)** ✅ *Current & Accurate* - Running quality checks

### **Development Areas**
- **[Tool Development Guide](tool-development-guide.md)** 📋 *Missing - Phase 3 Priority*
- **[Agent Development Guide](agent-development-guide.md)** 📋 *Missing - Phase 3 Priority*
- **[Bridge Development Guide](bridge-development-guide.md)** 📋 *Missing - Phase 3 Priority*
- **[Hook Development Guide](hook-development-guide.md)** ✅ *Phase 4* - Creating custom hooks in Rust

---

## 🏗️ Current Architecture (Phase 3.3)

### **Core Components**
```
rs-llmspell/
├── llmspell-core/          # Core traits and types
├── llmspell-bridge/        # Lua/JS script engine bridge  
├── llmspell-tools/         # 34 built-in tools
├── llmspell-utils/         # Shared utilities (resource limits, etc.)
├── llmspell-security/      # Security framework
└── llmspell-agents/        # Agent infrastructure (Phase 3.3)
```

### **Development Status by Component**
- **✅ llmspell-core**: Stable trait definitions and core types
- **✅ llmspell-bridge**: Synchronous API wrappers over async Rust
- **✅ llmspell-tools**: 34 production-ready tools with security
- **✅ llmspell-utils**: Resource limits and shared functionality  
- **✅ llmspell-security**: Security levels, sandboxing, validation
- **🚧 llmspell-agents**: Agent infrastructure in active development

---

## 🔧 Development Workflows

### **Working on Tools**
```bash
# 1. Create new tool in llmspell-tools/src/
# 2. Implement Tool trait with security requirements
# 3. Add tests with appropriate tags
cargo test -p llmspell-tools --test test_your_tool
# 4. Register tool in llmspell-bridge/src/tools.rs
# 5. Test integration
cargo test --test integration_test_tools
```

### **Working on Agents (Phase 3.3)**
```bash
# 1. Implement Agent trait in llmspell-agents/
# 2. Add to agent registry
# 3. Test with Lua bridge
./scripts/test-by-tag.sh agent
# 4. Integration testing
cargo test -p llmspell-bridge --lib
```

### **Working on Bridge (Lua/JS)**
```bash
# 1. Modify global implementations in llmspell-bridge/src/lua/globals/
# 2. Update synchronous wrappers using sync_utils patterns
# 3. Test Lua integration
cargo test --test lua_integration_tests
# 4. Validate with quality check
./scripts/quality-check-fast.sh
```

### **Quality Gates**
```bash
# Before committing (5 seconds)
./scripts/quality-check-minimal.sh

# Before pushing (1 minute)  
./scripts/quality-check-fast.sh

# Before PRs (5+ minutes)
./scripts/quality-check.sh
```

---

## 📖 Key Concepts for Contributors

### **1. Synchronous API Over Async Rust**
- **Challenge**: Lua/JS are synchronous, Rust LLM operations are async
- **Solution**: `sync_utils.rs` provides `block_on_async()` wrappers
- **Pattern**: All bridge methods use synchronous wrappers internally
- **See**: [Synchronous API Patterns](synchronous-api-patterns.md)

### **2. Security-First Tool Development**
- **Every tool** must declare `security_level()`: Safe, Restricted, or Privileged
- **Resource limits** are enforced automatically via ResourceTracker
- **Sandboxing** provides file/network access control
- **See**: [Security Development Guide](security-guide.md)

### **3. Phase 3.3 Development Focus**
- **Agent Infrastructure**: Multi-agent coordination and composition
- **Tool Integration**: Agents discovering and invoking tools
- **Workflow Enhancement**: Advanced workflow patterns with agent support
- **Bridge Completion**: Full Lua API coverage for agents and workflows

---

## 🛠️ Common Development Tasks

### **Adding a New Tool**
1. **Create tool struct** in `llmspell-tools/src/your_category/`
2. **Implement Tool trait** with execute method and security requirements
3. **Add tests** with `#[cfg(test)]` and appropriate tags (`#[test]` for unit, `#[cfg(feature = "integration")]` for integration)
4. **Register in bridge** at `llmspell-bridge/src/tools.rs`
5. **Document in user guide** at `docs/user-guide/`

### **Modifying Agent Behavior**
1. **Update agent implementation** in `llmspell-agents/`
2. **Modify Lua bindings** in `llmspell-bridge/src/lua/globals/agent.rs`
3. **Test agent registry** and execution patterns
4. **Update documentation** in `docs/user-guide/agent-api.md`

### **Adding Bridge APIs**
1. **Define synchronous wrapper** using patterns from `sync_utils.rs`
2. **Implement Lua binding** in appropriate global module
3. **Add integration tests** with Lua script examples
4. **Update API documentation** and user guides

---

## 🧪 Testing Strategy

### **Test Categories** (See [Test Organization](test-organization.md))
- **Unit Tests**: `cargo test --lib` - Individual component testing
- **Integration Tests**: `./scripts/test-by-tag.sh integration` - Component interaction
- **Tool Tests**: `./scripts/test-by-tag.sh tool` - Tool-specific functionality
- **Agent Tests**: `./scripts/test-by-tag.sh agent` - Agent infrastructure (Phase 3.3)
- **External Tests**: `./scripts/test-by-tag.sh external` - Network-dependent tests

### **Performance Benchmarks**
```bash
# Benchmark tool performance
cargo bench -p llmspell-tools

# Benchmark agent operations (Phase 3.3)
cargo bench -p llmspell-agents

# Benchmark bridge overhead
cargo bench -p llmspell-bridge
```

---

## 🐛 Debugging Guide

### **Common Issues**

#### **Lua Bridge Compilation Errors**
```bash
# Check Lua bindings compilation
cargo check -p llmspell-bridge --features lua

# Test specific global
cargo test -p llmspell-bridge --test lua_globals_test -- --nocapture
```

#### **Tool Registration Issues**
```bash
# Verify tool is registered
cargo test --test tool_registry_test

# Check tool discovery
cargo test -p llmspell-bridge --lib -- tool_discovery
```

#### **Agent Infrastructure Issues (Phase 3.3)**
```bash
# Test agent creation and registry
cargo test -p llmspell-agents --lib

# Test agent-tool integration
cargo test --test agent_tool_integration
```

### **Debugging Tools**
- **Rust Analyzer**: IDE integration with error highlighting
- **cargo check**: Fast compilation checking
- **cargo clippy**: Linting and common mistake detection
- **RUST_LOG=debug**: Enable debug logging for runtime issues

---

## 📋 Code Quality Standards

### **Mandatory Requirements**
- **Zero Warnings**: `cargo clippy -- -D warnings` must pass
- **Formatting**: `cargo fmt --all` applied before commits
- **Documentation**: All public APIs must have docs (`#[warn(missing_docs)]`)
- **Security**: All tools must declare appropriate security levels
- **Testing**: New features require both unit and integration tests

### **Performance Requirements**
- **Tool Initialization**: <10ms (measured in benchmarks)
- **Agent Operations**: <50ms overhead (Phase 3.3 target)
- **Bridge Overhead**: <5ms for global method calls
- **Memory Usage**: Tools must use ResourceTracker for memory >1MB

### **Code Review Checklist**
- [ ] Compiles without warnings
- [ ] Tests pass (`./scripts/quality-check-fast.sh`)
- [ ] Security requirements declared
- [ ] Documentation updated
- [ ] Breaking changes documented
- [ ] Performance impact considered

---

## 📞 Getting Help

### **When Stuck**
1. **Search Issues**: Check if similar problems exist
2. **Ask Questions**: Use discussions for architectural questions
3. **Code Review**: Request early feedback on complex changes
4. **Documentation**: Check if implementation guides cover your use case

### **Contributing Process**
1. **Fork & Branch**: Create feature branch from `main`
2. **Implement**: Follow development workflows above
3. **Test**: Pass all quality gates
4. **Document**: Update relevant documentation
5. **PR**: Submit with clear description of changes

---

## 🗺️ Phase Roadmap for Contributors

### **Current: Phase 3.3 - Agent Infrastructure (Weeks 15-16)**
**Focus Areas**:
- Agent factory and registry completion
- Multi-agent coordination patterns  
- Agent-tool integration
- Workflow-agent composition

**Contribution Opportunities**:
- Agent implementation patterns
- Tool integration with agent discovery
- Advanced workflow patterns
- Lua API completion for agents

### **Next: Phase 4 - Hooks & Events (Weeks 17-18)**
**Focus Areas**:
- Hook system implementation (20+ lifecycle points)
- Event bus with async subscriptions
- Performance monitoring hooks
- Integration with existing workflows

### **Future: Phase 5 - Persistent State (Weeks 19-20)**
**Focus Areas**:
- State backed by sled/rocksdb
- Automatic persistence and recovery
- State migrations and versioning
- Cross-session state management

---

## 🎯 Current Development Priorities

### **High Priority (Phase 3.3)**
1. **Agent Infrastructure Completion**: Factory, registry, lifecycle management
2. **Tool-Agent Integration**: Agents discovering and invoking tools
3. **Workflow Enhancement**: Agent-aware workflow patterns
4. **Bridge API Completion**: Full Lua coverage for agents

### **Medium Priority**
1. **Documentation Updates**: Keep development guides current with implementation
2. **Performance Optimization**: Meet Phase 3.3 performance targets
3. **Test Coverage**: Increase coverage in agent infrastructure
4. **Security Hardening**: Complete security review of agent components

### **Ongoing**
1. **Code Quality**: Maintain zero warnings and high documentation coverage
2. **User Experience**: Keep user documentation synchronized with features
3. **Community**: Respond to issues and support contributors

---

**Happy contributing to rs-llmspell! 🚀**

*For technical architecture details, see [rs-llmspell-final-architecture.md](../technical/rs-llmspell-final-architecture.md)*  
*For user-facing features, see [User Guide](../user-guide/README.md)*