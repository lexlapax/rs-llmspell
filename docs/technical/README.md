# Technical Architecture Documentation

**Version**: Phase 3.3 (95%+ Complete)  
**Last Updated**: July 2025  
**Audience**: System architects, core contributors, and technical integrators

> **🏗️ Technical Deep Dive**: This directory contains comprehensive technical documentation for rs-llmspell's architecture, implementation patterns, and system design decisions. All documents reflect the current Phase 3.3 implementation status unless explicitly marked as future work.

**🔗 Navigation**: [← Documentation Hub](../README.md) | [Project Home](../../README.md) | [User Guide](../user-guide/) | [Developer Guide](../developer-guide/)

---

## Core Architecture Documents

### Master Reference
- **[rs-llmspell-final-architecture.md](rs-llmspell-final-architecture.md)** - Complete standalone architecture document containing ALL architectural, implementation, and operational details. No external references required.

### Implementation Deep Dives
- **[global-injection-architecture.md](global-injection-architecture.md)** - ✅ **Current** - Detailed technical architecture of the global injection system with performance optimizations, dependency resolution, and multi-language support
- **[tool-bridge-architecture.md](tool-bridge-architecture.md)** - ✅ **Current** - Tool integration patterns, async execution handling, and bridge layer architecture connecting scripts to native tools

---

## Security & State Management

### Security Architecture
- **[security-architecture.md](security-architecture.md)** - ✅ **Current** - Comprehensive defense-in-depth security model with STRIDE threat analysis, Phase 3.2 security hardening details, and tool-specific security controls

### State Management
- **[state-architecture.md](state-architecture.md)** - ✅ **Current** - Workflow state management architecture with thread-safe operations, state scoping, and Phase 5 persistence integration points

---

## Current Architecture (Phase 4)

### Hook and Event System
- **[hook-event-architecture.md](hook-event-architecture.md)** - ✅ **Phase 4 Complete** - Comprehensive hook and event system architecture with 40+ hook points, CircuitBreaker protection, and cross-language support
- **[hook-implementation.md](hook-implementation.md)** - 📋 **Phase 4 Design Reference** - Original hook system design document (see hook-event-architecture.md for current implementation)

---

## Quick Reference Guide

### For System Architects
1. Start with [rs-llmspell-final-architecture.md](rs-llmspell-final-architecture.md) for complete system overview
2. Review [security-architecture.md](security-architecture.md) for threat model and defense layers
3. Study [global-injection-architecture.md](global-injection-architecture.md) for runtime architecture

### For Core Contributors  
1. Read [tool-bridge-architecture.md](tool-bridge-architecture.md) for integration patterns
2. Understand [state-architecture.md](state-architecture.md) for workflow coordination
3. Plan future work using [hook-implementation.md](hook-implementation.md)

### For Technical Integrators
1. Focus on [global-injection-architecture.md](global-injection-architecture.md) for API surface
2. Reference [security-architecture.md](security-architecture.md) for compliance requirements
3. Use [tool-bridge-architecture.md](tool-bridge-architecture.md) for custom tool development

---

## Implementation Status

### ✅ **Phase 3.3 Complete** (Current)
- **Global Injection System**: 2-4ms injection time, >90% cache hit rate
- **Tool Bridge Architecture**: 34 production tools, async execution, security hardening
- **Security Architecture**: Defense-in-depth with STRIDE analysis, comprehensive hardening
- **State Management**: Thread-safe workflow state with scoping and isolation

### ✅ **Phase 4 Complete** (July 2025)
- **Hook System**: 40+ hook points with CircuitBreaker protection (<5% overhead)
- **Event Architecture**: High-throughput pub/sub system (90K+ events/sec)
- **Cross-Language Support**: Hooks and events work across Lua/JS/Rust
- **Built-in Hooks**: 18+ production-ready hooks for common patterns

### 🚀 **Phase 5+ Future**
- **Persistent Storage**: Sled/RocksDB backends with crash recovery
- **Distributed State**: Multi-node synchronization and conflict resolution
- **Advanced Security**: HSM integration and quantum-resistant cryptography

---

## Performance Characteristics

Current system performance (Phase 3.3):

| Component | Metric | Requirement | Actual | Status |
|-----------|--------|-------------|---------|---------|
| Global Injection | Initialization | <5ms | 2-4ms | ✅ |
| Tool Execution | Average latency | <10ms | 0.5-2.1ms | ✅ |
| State Operations | Read/Write | <1ms | <1ms | ✅ |
| Security Validation | Input processing | <5ms | <2ms | ✅ |
| Memory Usage | Per context | <5MB | 1.8MB | ✅ |

---

## Architecture Principles

### Core Design Philosophy
1. **Bridge-First Design**: Scripts as first-class interfaces, not add-ons
2. **Composition Over Inheritance**: BaseAgent → Agent → Tool → Workflow hierarchy
3. **Security by Design**: Defense-in-depth with multiple validation layers
4. **Performance First**: <10ms tool initialization, async-everywhere architecture
5. **Language Agnostic**: Consistent API across Lua, JavaScript, and future languages

### Technical Patterns
- **Global Injection**: Zero-configuration access to all functionality
- **Async Everywhere**: Non-blocking I/O with efficient resource utilization  
- **State-First Communication**: Shared state for agent/workflow coordination
- **Type-Safe Bridging**: Bidirectional conversion with validation
- **Resource Isolation**: Sandboxing and limits for safe execution

---

## Related Documentation

### User Documentation
- [User Guide](../user-guide/) - End-user scripting guide and API reference
- [Getting Started](../user-guide/getting-started.md) - Quick start for new users

### Developer Documentation  
- [Developer Guide](../developer-guide/) - Tool development and contribution guide
- [Tool Development](../developer-guide/tool-development-guide.md) - Creating custom tools

### Project Documentation
- [Implementation Phases](../in-progress/implementation-phases.md) - Project roadmap and phase planning
- [Phase Completion Status](../in-progress/) - Current progress and handoff documents

---

**📞 Questions or Issues?**
- Technical architecture questions: Review master architecture document first
- Implementation details: Check component-specific deep dive documents  
- Security concerns: Consult security architecture and threat model
- Performance questions: Reference benchmarks and optimization guides

*This documentation reflects the rs-llmspell system as of Phase 3.3 completion (July 2025).*