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
- **[Architecture Overview](../technical/master-architecture-vision.md)** - System design and component relationships

### **Implementation Guides**
- **[Synchronous API Patterns](synchronous-api-patterns.md)** ✅ *Current & Accurate* - Lua/JS bridge patterns
- **[Resource Limits Implementation](implementing-resource-limits.md)** ⚠️ *API Fixed* - Resource tracking in tools
- **[Security Development Guide](security-guide.md)** ⚠️ *API Fixed* - Security best practices

### **Testing & Quality**
- **[Test Organization](test-organization.md)** ✅ *Current & Accurate* - Test categories and execution
- **[Agent Testing Guide](agent-testing-guide.md)** ✅ *Current & Accurate* - Testing agents with mocks and scenarios
- **[Quality Check Scripts](quality-check-scripts.md)** ✅ *Current & Accurate* - Running quality checks

### **Development Areas**
- **[Tool Development Guide](tool-development-guide.md)** ✅ *Complete* - Developing new tools
- **[Agent Development Guide](agent-development-guide.md)** ✅ *Complete* - Agent infrastructure guide
- **[Bridge Development Guide](bridge-development-guide.md)** ✅ *Complete* - Script language integration
- **[Hook Development Guide](hook-development-guide.md)** ✅ *Complete* - Creating custom hooks in Rust
- **[State Development Guide](../state-management/)** ✅ *Phase 5* - State persistence development

---

## 🏗️ Current Architecture (v0.5.0 - Phase 5 Complete)

### **Core Components**
```
rs-llmspell/
├── llmspell-core/          # Core traits and types
├── llmspell-bridge/        # Lua/JS script engine bridge  
├── llmspell-tools/         # 34 built-in tools
├── llmspell-utils/         # Shared utilities (resource limits, etc.)
├── llmspell-security/      # Security framework
└── llmspell-agents/        # Agent infrastructure with state persistence
```

### **Development Status by Component**
- **✅ llmspell-core**: Stable trait definitions and core types
- **✅ llmspell-bridge**: Synchronous API wrappers over async Rust
- **✅ llmspell-tools**: 34 production-ready tools with security
- **✅ llmspell-utils**: Resource limits and shared functionality  
- **✅ llmspell-security**: Security levels, sandboxing, validation
- **✅ llmspell-agents**: Production-ready agent infrastructure
- **✅ llmspell-hooks**: Hook system with 40+ hook points
- **✅ llmspell-events**: High-performance event bus (90K+ events/sec)
- **✅ llmspell-state-persistence**: Multi-backend state management

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

### **Working on Agents**
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

### **3. Phase 5 Development Achievements**
- **Persistent State Management**: Multi-backend support (Memory, Sled, RocksDB)
- **Enterprise Features**: Schema migrations, atomic backups, retention policies
- **Performance Excellence**: <5ms state operations, 2.07μs/item migrations
- **Hook Integration**: All state changes trigger hooks with <2% overhead
- **Security Enhancements**: Circular reference detection, sensitive data protection
- **Testing Revolution**: 7 test categories with quality check scripts

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
- **Agent Tests**: `./scripts/test-by-tag.sh agent` - Agent infrastructure
- **External Tests**: `./scripts/test-by-tag.sh external` - Network-dependent tests

### **Performance Benchmarks**
```bash
# Benchmark tool performance
cargo bench -p llmspell-tools

# Benchmark state operations
cargo bench -p llmspell-state-persistence

# Benchmark agent operations
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

#### **Common Development Issues**
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
- **Agent Operations**: <50ms overhead (achieved)
- **State Operations**: <5ms read/write (achieved)
- **Hook Overhead**: <2% performance impact (achieved)
- **Migration Performance**: 2.07μs per item (achieved)
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

### **Completed: Phases 0-5 (v0.5.0 Released)**
**Achievements**:
- ✅ Foundation infrastructure with 12-crate workspace
- ✅ 34 production-ready tools across 9 categories
- ✅ Agent infrastructure with multi-agent coordination
- ✅ Hook system with 40+ hook points (<1% overhead)
- ✅ Event bus with 90K+ events/sec throughput
- ✅ Persistent state with multi-backend support
- ✅ Enterprise features: migrations, backups, retention

### **Next: Phase 6 - Session Management (Q3 2025)**
**Focus Areas**:
- Session lifecycle and persistence
- Session-scoped state management
- Agent session upgrades
- Multi-session coordination

**Contribution Opportunities**:
- Session manager implementation
- Session persistence strategies
- Session migration utilities
- Cross-session communication

### **Future: Phase 7+ - Advanced Features**
**Planned Features**:
- GUI interface (Phase 7)
- Python support (Phase 9)
- Vector storage integration (Phase 10)
- Enterprise deployment tools (Phase 11+)

---

## 🎯 Current Development Priorities

### **High Priority (Phase 6 Preparation)**
1. **Session Management Design**: Architecture for session lifecycle
2. **Session Persistence**: Storage strategies for sessions
3. **Agent Session Upgrades**: Enhance agents with session awareness
4. **Documentation**: Update guides for session features

### **Medium Priority**
1. **Performance Monitoring**: Maintain achieved performance targets
2. **Bug Fixes**: Address any v0.5.0 issues reported
3. **Test Coverage**: Increase coverage in agent infrastructure
4. **Security Hardening**: Complete security review of agent components

### **Ongoing**
1. **Code Quality**: Maintain zero warnings and high documentation coverage
2. **User Experience**: Keep user documentation synchronized with features
3. **Community**: Respond to issues and support contributors

---

**Happy contributing to rs-llmspell! 🚀**

*For technical architecture details, see [master-architecture-vision.md](../technical/master-architecture-vision.md)*  
*For user-facing features, see [User Guide](../user-guide/README.md)*