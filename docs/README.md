# Rs-LLMSpell Documentation Hub

**Complete documentation for scriptable LLM interactions**

**🔗 Navigation**: [← Project Home](../README.md) | [Examples](../examples/) | [Contributing](../CONTRIBUTING.md)

> **📖 Documentation Hub**: All documentation for rs-llmspell v0.6.0 (Phase 7 Complete). Consolidated, validated, and organized for clarity.

---

## Documentation Structure (Post-Consolidation)

### 📘 [User Guide](user-guide/) - *For Script Writers*
**Purpose**: Practical guides for using rs-llmspell to build LLM-driven scripts and workflows.

**Status**: ✅ Consolidated from 38→7 essential files  
**Contents**: Getting started, concepts, configuration, API reference (Lua/Rust), troubleshooting  
**Start here if**: You want to write Lua scripts that use LLMs and tools

---

### 🔧 [Developer Guide](developer-guide/) - *For Contributors*  
**Purpose**: Technical guides for developers contributing to or extending rs-llmspell.

**Status**: ✅ Complete and current  
**Contents**: Tool development, security implementation, testing strategies, contribution workflows  
**Start here if**: You want to create custom tools, contribute code, or understand development

---

### 🏗️ [Technical](technical/) - *For Architects*
**Purpose**: Core architectural documentation and implementation decisions.

**Status**: ✅ Consolidated from 35→7 essential files  
**Contents**: Current architecture, decisions (ADRs), security model, performance benchmarks, API standards  
**Start here if**: You need to understand system architecture, security, or performance

---

### 📚 [Archives](archives/) - *Historical Reference*
**Purpose**: Preserve historical documents for reference.

**Status**: 📦 50+ documents archived  
**Contents**: Superseded technical docs (18), consolidated user guides (32), research notes  
**Note**: These documents may be outdated but provide historical context

---

### 🚧 [In-Progress](in-progress/) - *Development Tracking*
**Purpose**: Track planning and implementation toward version 1.0.

**Status**: 📋 Active development  
**Contents**: Phase completion documents, implementation roadmaps, design documents  
**For**: Core team tracking progress

---

## What Rs-LLMSpell Actually Is

**Production-Ready Features** (v0.6.0):
- ✅ **37+ tools** across 9 categories (file, web, data, media, system)
- ✅ **Lua scripting** with 15 zero-import globals
- ✅ **Agent infrastructure** with builder patterns and templates
- ✅ **4 workflow patterns** (Sequential, Parallel, Conditional, Loop)
- ✅ **State persistence** with 3 backends (Memory, Sled, RocksDB)
- ✅ **Hook system** with 40+ points and circuit breakers
- ✅ **Event bus** with 90K+ events/sec throughput
- ✅ **3-level security** model with sandboxing

**What it doesn't do**:
- ❌ GUI or web interface (CLI and library only)
- ❌ JavaScript support (Lua only currently)
- ❌ Python support (planned for Phase 9)
- ❌ Distributed execution (planned for Phase 12)

---

## Quick Start Paths

### 🚀 **I want to use rs-llmspell**
1. **[Getting Started](user-guide/getting-started.md)** - 5-minute setup
2. **[Core Concepts](user-guide/concepts.md)** - Understand the basics
3. **[Lua API Reference](user-guide/api/lua/)** - Complete API docs
4. **[Examples](../examples/)** - Working code examples

### 🔨 **I want to extend rs-llmspell**
1. **[Developer Setup](developer-guide/README.md)** - Development environment
2. **[Tool Development](developer-guide/tool-development-guide.md)** - Create tools
3. **[API Style Guide](technical/api-style-guide.md)** - Coding standards
4. **[Testing Guide](developer-guide/test-organization.md)** - Test your code

### 🏛️ **I need architectural understanding**
1. **[Current Architecture](technical/current-architecture.md)** - What exists now
2. **[Architecture Decisions](technical/architecture-decisions.md)** - Why it exists
3. **[Security Model](technical/security-model.md)** - Security implementation
4. **[Performance Benchmarks](technical/performance-benchmarks.md)** - Actual metrics

---

## Phase 7 Documentation Achievements

### Consolidation Results
- **User Guide**: 38 files → 7 essential files (82% reduction)
- **Technical Docs**: 35 files → 7 essential files (80% reduction)
- **Total Archives**: 50+ documents moved to archives
- **Validation**: All content verified against actual implementation

### Key Improvements
1. ✅ **Single source of truth** for architecture (current-architecture.md)
2. ✅ **Consolidated ADRs** from all phases (architecture-decisions.md)
3. ✅ **Unified security documentation** (security-model.md)
4. ✅ **Actual performance metrics** (performance-benchmarks.md)
5. ✅ **Official API standards** (api-style-guide.md)
6. ✅ **Clean navigation** with clear purpose for each document

---

## Documentation Quality Standards

### Accuracy ✅
- All code examples tested and working
- API documentation matches v0.6.0 implementation
- Performance metrics from actual measurements
- Architecture validated against codebase

### Organization ✅
- Clear separation: What (architecture) vs How (guides)
- No duplicate content between files
- Appropriate archival of outdated content
- Cross-references validated

### Maintenance 📋
- Version tracking (v0.6.0)
- Phase status clearly marked
- Update dates on all documents
- Clear deprecation notices

---

## Getting Help

**📋 Documentation Issues**: File GitHub issues for corrections  
**❓ Usage Questions**: Check [Troubleshooting](user-guide/troubleshooting.md)  
**🐛 Bug Reports**: Use GitHub issues with reproduction steps  
**💡 Feature Requests**: Review [roadmap](in-progress/implementation-phases.md) first  
**🤝 Contributing**: Start with [Developer Guide](developer-guide/)  

---

**Last Updated**: August 2025 | **Version**: 0.6.0 (Phase 7 Complete) | **Next**: Phase 8 (GUI)