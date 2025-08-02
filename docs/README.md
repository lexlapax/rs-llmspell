# Rs-LLMSpell Documentation

**Scriptable LLM interactions via Lua, JavaScript** - Cast scripting spells to animate LLM golems

> **📖 Documentation Hub**: Complete documentation for rs-llmspell, a Rust library that enables scriptable LLM interactions through embedded scripting engines. This documentation covers everything from quick start guides to deep architectural analysis.

**🔗 Back to Project**: [← Main README](../README.md) | **Current Status**: Phase 5 Complete (v0.5.0) | **Target**: Version 1.0

---

## What Rs-LLMSpell Actually Does

Rs-LLMSpell is a **focused scripting framework** for LLM interactions. It provides:

✅ **What it delivers:**
- 34 production-ready tools across 9 categories (file, network, data processing, etc.)
- Embedded Lua and JavaScript runtimes with zero-configuration global APIs
- Agent creation and coordination through scripts
- Workflow orchestration (Sequential, Conditional, Loop, Parallel patterns)
- Thread-safe state management for complex interactions
- Comprehensive security with sandboxing and resource limits

❌ **What it doesn't do:**
- Replace general-purpose automation tools (not a Zapier alternative)
- Provide GUI or web interfaces (command-line and library focused)
- Include ML/AI models (integrates with external LLM providers)
- Solve every workflow automation need (focused on LLM-centric use cases)

**Target Audience**: Developers who need scriptable LLM interactions, researchers building LLM workflows, and teams automating LLM-driven processes.

---

## Documentation Structure

### 📘 [User Guide](user-guide/) - *For End Users*
**Purpose**: Practical guides for using rs-llmspell to build LLM-driven scripts and workflows.

**Contents**: Getting started, API reference, tool documentation, workflow patterns, best practices  
**Audience**: Script writers, workflow builders, integration developers  
**Status**: ✅ Complete and current (11 documents)

**Start here if**: You want to write Lua/JavaScript scripts that use LLMs and tools

---

### 🔧 [Developer Guide](developer-guide/) - *For Library Developers*  
**Purpose**: Technical guides for developers contributing to or extending the rs-llmspell library itself.

**Contents**: Tool development, security implementation, testing strategies, contribution workflows  
**Audience**: Rust developers, library contributors, custom tool creators  
**Status**: ✅ Complete and current (6 documents)

**Start here if**: You want to create custom tools, contribute code, or understand the development process

---

### 🏗️ [Technical](technical/) - *For System Architects*
**Purpose**: Deep architectural documentation for system design, integration patterns, and technical decisions.

**Contents**: Complete architecture reference, security model, performance analysis, implementation deep dives  
**Audience**: System architects, technical leads, integration specialists  
**Status**: ✅ Complete and current (6 documents + master architecture)

**Start here if**: You need to understand system architecture, security model, or integration patterns

---

### 📚 [Archives](archives/) - *Historical Reference*
**Purpose**: Preserve historical documents, research notes, and deprecated documentation for reference.

**Contents**: Historical design decisions, deprecated guides, research documentation, migration notes  
**Audience**: Project historians, researchers, anyone needing historical context  
**Status**: 🗂️ Repository for historical materials

**Contains**: Documents that are no longer current but may be valuable for understanding project evolution

---

### 🚧 [In-Progress](in-progress/) - *Development Tracking*
**Purpose**: Track planning, implementation, and progress toward version 1.0 release.

**Contents**: Phase completion documents, implementation roadmaps, handoff packages, TODOs  
**Audience**: Core team, project managers, contributors tracking progress  
**Status**: 📋 Active development tracking

**Contains**: Phase completion status, implementation plans, progress tracking until 1.0 release

---

## Quick Navigation

### 🚀 **I want to use rs-llmspell**
1. **[Getting Started Guide](user-guide/getting-started.md)** - 5-minute setup and first script
2. **[Tutorial: Agents & Workflows](user-guide/tutorial-agents-workflows.md)** - Step-by-step learning
3. **[Tool Reference](user-guide/tool-reference.md)** - Browse all 34 available tools  
4. **[Agent API Guide](user-guide/agent-api.md)** - Create and coordinate LLM agents
5. **[Workflow Patterns](user-guide/workflow-api.md)** - Build complex multi-step processes
6. **[Examples](../examples/)** - Working code examples

### 🔨 **I want to extend rs-llmspell**
1. **[Developer Setup](developer-guide/README.md)** - 5-minute development environment
2. **[Tool Development](developer-guide/tool-development-guide.md)** - Create custom tools
3. **[Security Requirements](developer-guide/security-guide.md)** - Implement security correctly
4. **[Testing Guide](developer-guide/testing-guide.md)** - Test your contributions

### 🏛️ **I need architectural understanding**
1. **[Master Architecture](technical/master-architecture-vision.md)** - Complete system reference
2. **[Security Architecture](technical/security-architecture.md)** - Threat model and defenses
3. **[Global Injection System](technical/global-injection-architecture.md)** - API injection deep dive
4. **[Performance Characteristics](technical/README.md#performance-characteristics)** - Benchmarks and limits

### 📊 **I want to track progress**
1. **[Implementation Phases](in-progress/implementation-phases.md)** - Roadmap to 1.0
2. **[Phase Status](in-progress/)** - Current completion status  
3. **[Known Limitations](user-guide/README.md#current-limitations--workarounds)** - What's not yet available
4. **[Hook & Event Architecture](technical/hook-event-architecture.md)** - Phase 4 extensibility system
5. **[State Management Guide](state-management/)** - Phase 5 persistent state system

---

## Current Project Status

### ✅ **Phase 5 Complete** (July 2025 - v0.5.0)
- **Persistent State Management**: Multi-backend support (Memory, Sled, RocksDB)
- **State Scoping**: 6 levels (Global, Agent, Workflow, Step, Session, Custom)
- **Enterprise Features**: Schema migrations, atomic backups, retention policies
- **Performance Achievements**: <5ms state operations, 2.07μs/item migrations
- **Security Enhancements**: Circular reference detection, sensitive data protection
- **Testing Revolution**: 7 test categories with quality check scripts
- **Hook Integration**: All state changes trigger hooks with <2% overhead

### ✅ **Previous Phases Complete**
- **Phase 4**: Hook & Event System (40+ hooks, 90K events/sec)
- **Phase 3**: 34 Production Tools & Workflow Patterns
- **Phase 2**: Agent Infrastructure & Bridge Architecture
- **Phase 1**: Core Foundation & Script Execution

### 📋 **Working Toward 1.0** (Target: Q4 2025)
- **Phase 6**: Session Management & Agent Upgrades (Next)
- **Phase 7-9**: GUI, Testing Tools, Python Support
- **Phase 10-16**: Advanced Features & Enterprise Capabilities
- **Version 1.0**: Stable API with backward compatibility guarantees

### 🎯 **What 1.0 Means For You**
- **API Stability**: No breaking changes without major version bump
- **Production Ready**: Full enterprise deployment support
- **Complete Documentation**: All features fully documented
- **Long-term Support**: Maintenance and security updates

---

## Documentation Quality Standards

This documentation follows strict quality standards:

### **Accuracy Requirements**
- ✅ All code examples tested and working
- ✅ API documentation matches implementation
- ✅ Status indicators reflect actual implementation
- ✅ No aspirational features presented as current

### **User Experience Focus**
- 🎯 Clear navigation for different user types
- 🎯 Realistic expectations about capabilities
- 🎯 Practical examples over theoretical concepts
- 🎯 Progressive disclosure (basic → advanced)

### **Maintenance Standards**
- 📝 Regular updates as features are implemented
- 📝 Version tracking for all major changes
- 📝 Cross-references validated and current
- 📝 Deprecated content moved to archives

---

## Getting Help

**📋 Documentation Issues**: File issues on GitHub if you find outdated or incorrect documentation

**❓ Usage Questions**: 
- Check [User Guide FAQ](user-guide/getting-started.md#troubleshooting)
- Browse [examples directory](../examples/) for patterns
- Review [troubleshooting guides](user-guide/README.md)

**🐛 Bug Reports**: Use GitHub issues with specific reproduction steps

**💡 Feature Requests**: Review [planned features](in-progress/implementation-phases.md) first, then file GitHub issues

**🤝 Contributing**: Start with [Developer Guide](developer-guide/README.md) for contribution workflow

---

## Documentation Navigation

- **🏠 Project Home**: [Main README](../README.md)
- **📁 Examples**: [Examples Directory](../examples/)
- **🔧 Source Code**: [Library Source](../llmspell-*)
- **🧪 Tests**: [Integration Tests](../tests/)

**Last Updated**: July 29, 2025 | **Documentation Version**: Phase 5 Complete (v0.5.0) | **Next Update**: Phase 6 Features