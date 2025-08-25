# Technical Documentation

**Version**: v0.6.0 (Phase 7 Complete)  
**Last Updated**: August 2025  
**Status**: Production Architecture - Fully Consolidated

> **📋 Technical Reference**: Consolidated technical documentation for LLMSpell's architecture. All content validated against actual implementation with 35→7 files consolidation achieved.

**🔗 Navigation**: [← Documentation Hub](../README.md) | [User Guide](../user-guide/) | [Developer Guide](../developer-guide/)

---

## Core Documentation (7 Essential Files)

### 📊 What We Built
- **[current-architecture.md](current-architecture.md)** - **SINGLE SOURCE OF TRUTH** - The actual implementation architecture (17 crates, 71K LOC, Phase 0-7 evolution)

### 🎯 Why We Built It
- **[architecture-decisions.md](architecture-decisions.md)** - All architectural decisions (28 ADRs) showing evolution and reversals across phases

### 🛡️ How It's Secured
- **[security-model.md](security-model.md)** - Complete security architecture (3-level model, STRIDE analysis, actual implementation)

### ⚡ How It Performs
- **[performance-benchmarks.md](performance-benchmarks.md)** - Actual measured performance (all targets met/exceeded, validated metrics)

### 📐 How to Build It
- **[api-style-guide.md](api-style-guide.md)** - Official API standards (Phase 7 - naming, patterns, documentation requirements)

### 🔮 Historical Reference
- **[master-architecture-vision.md](master-architecture-vision.md)** - Original aspirational architecture (historical reference only, NOT current state)

## Component Architecture References

### 🐛 Debug Infrastructure
- **[debug-infrastructure-architecture.md](debug-infrastructure-architecture.md)** - Complete debug system architecture (global manager, thread safety, performance optimization)

---

## Quick Navigation Guide

### Understanding the System
1. **Start**: [current-architecture.md](current-architecture.md) - What exists
2. **Learn**: [architecture-decisions.md](architecture-decisions.md) - Why it exists
3. **Secure**: [security-model.md](security-model.md) - How it's protected
4. **Measure**: [performance-benchmarks.md](performance-benchmarks.md) - How fast it runs
5. **Build**: [api-style-guide.md](api-style-guide.md) - How to extend it

---

## Key Architecture Facts

### System Scale (Validated)
- **17 crates** in Cargo workspace
- **71,000+ lines** of production Rust
- **37+ tools** (evolved from 26→33→37)
- **15 Lua globals** with zero-import pattern
- **40+ hook points** with circuit breakers
- **4 workflow types** (Sequential/Parallel/Conditional/Loop)

### Performance Achievements
| Metric | Target | Actual | Status |
|--------|--------|--------|--------|
| Tool Init | <10ms | <10ms | ✅ |
| Agent Creation | <50ms | ~10ms | ✅ 5x better |
| Hook Overhead | <5% | <2% | ✅ |
| Event Throughput | 50K/sec | 90K/sec | ✅ 1.8x |
| State Migration | - | 2.07μs/item | ✅ |

### Architecture Highlights
- **BaseAgent Foundation**: Universal trait for all components
- **Sync Bridge Pattern**: block_on() for Lua/JS integration  
- **Global Injection**: 2-4ms for all 15 globals
- **3-Level Security**: Safe/Restricted/Privileged
- **Multi-Backend State**: Memory/Sled/RocksDB
- **Builder Pattern**: Universal object creation

---

## Phase 7 Documentation Achievements

### Consolidation Complete ✅
- **Before**: 35 scattered technical files
- **After**: 7 organized essential files
- **Reduction**: 80% file count reduction
- **Archives**: 18 files archived (13 technical + 5 API planning)

### What Was Done
1. ✅ Created `current-architecture.md` - validated against all phase docs
2. ✅ Created `architecture-decisions.md` - 28 ADRs from 7 phases
3. ✅ Created `security-model.md` - merged 2 security docs
4. ✅ Created `performance-benchmarks.md` - actual measured metrics
5. ✅ Kept `api-style-guide.md` - Phase 7 official standards
6. ✅ Archived 18 redundant/outdated files
7. ✅ Updated all cross-references

### Content Validation
- All architecture validated against phase design docs
- All performance metrics from actual measurements
- All security features verified in implementation
- All API patterns confirmed in codebase

---

## Related Documentation

### For Users
- **[User Guide](../user-guide/)** - How to use LLMSpell
- **[Getting Started](../user-guide/getting-started.md)** - 5-minute quickstart
- **[API Reference](../user-guide/api/)** - Lua and Rust APIs

### For Developers
- **[Developer Guide](../developer-guide/)** - Contributing guide
- **[Tool Development](../developer-guide/tool-development-guide.md)** - Create tools
- **[Test Organization](../developer-guide/test-organization.md)** - Testing strategy

### Archives
- **[Technical Archives](../archives/technical/)** - 18 historical documents
- **[User Guide Archives](../archives/user-guide/)** - 32 consolidated files

---

## Document Status Table

| File | Lines | Purpose | Validation |
|------|-------|---------|------------|
| current-architecture.md | 362 | Actual implementation | ✅ Code + Phases |
| architecture-decisions.md | 484 | Design rationale (28 ADRs) | ✅ Phase docs |
| security-model.md | 334 | Security implementation | ✅ Security crates |
| performance-benchmarks.md | 257 | Measured performance | ✅ Test outputs |
| api-style-guide.md | 529 | API standards | ✅ Phase 7 work |
| master-architecture-vision.md | 20K+ | Original vision | 📚 Historical |
| **README.md** | This file | Navigation hub | ✅ Current |

---

*Documentation reflects LLMSpell v0.6.0 after Phase 7 completion. Reduced from 35→7 files with full validation.*