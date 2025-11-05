# Storage Documentation

**rs-llmspell Storage System** - Multi-backend storage architecture with hot-swappable backends

## Overview

The llmspell storage system provides trait-based abstraction for multiple backend implementations:

- **Embedded Backends**: Sled (key-value), InMemory (testing), HNSW (vector search)
- **Centralized Backends**: PostgreSQL (production), SurrealDB (graph)
- **Specialized Storage**: Large Object streaming, Partitioned event logs, Encrypted API keys

---

## Documentation Index

### User Guides

#### [Migration Guide](./migration-guide.md) - **Phase 1: Sled→PostgreSQL**
**Status**: Production Ready
**Components**: agent_state, workflow_state, sessions
**Updated**: January 2025

Comprehensive guide for migrating from embedded Sled backend to centralized PostgreSQL:
- **Quick Start**: 5-minute migration walkthrough
- **Step-by-Step**: Plan → Review → Dry-run → Execute → Validate
- **Architecture**: MigrationEngine, BackupManager, Validator components
- **Troubleshooting**: 7 common errors with solutions
- **FAQ**: 9 comprehensive questions (batch tuning, rollback, zero-downtime, etc.)
- **Phase 2/3 Preview**: Episodic/Semantic memory migrations (Q1 2025)

**Start here** if you need to migrate production workloads to PostgreSQL.

---

## Storage Architecture

### Backend Abstraction

```
┌──────────────────────────────────────────────┐
│ llmspell-storage Trait Layer                 │
│  ├── StorageBackend (KV operations)          │
│  ├── VectorStorage (HNSW, VectorChord)       │
│  ├── GraphStorage (Bi-temporal CTEs)         │
│  └── MigrationSource/Target (data migration) │
└──────────────────────────────────────────────┘
                     ↓
┌──────────────────────────────────────────────┐
│ Backend Implementations                      │
│  ├── SledBackend (embedded KV)               │
│  ├── PostgresBackend (centralized)           │
│  ├── HNSWStorage (vector files)              │
│  ├── SurrealDBStorage (graph)                │
│  └── InMemory (testing)                      │
└──────────────────────────────────────────────┘
```

### Storage Components

| Component | Sled | PostgreSQL | HNSW | SurrealDB | InMemory |
|-----------|------|------------|------|-----------|----------|
| Agent State | ✅ | ✅ (V7) | - | - | ✅ |
| Workflow State | ✅ | ✅ (V8) | - | - | ✅ |
| Sessions | ✅ | ✅ (V9) | - | - | ✅ |
| Episodic Memory | - | ✅ (V3) | ✅ | - | ✅ |
| Semantic Memory | - | ✅ (V4/V15) | - | ✅ | ✅ |
| Procedural Memory | - | ✅ (V6) | - | - | ✅ |
| Artifacts | Filesystem | ✅ (V10) | - | - | ✅ |
| Events | Custom | ✅ (V11) | - | - | ✅ |
| Hooks | Filesystem | ✅ (V12) | - | - | - |
| API Keys | Filesystem | ✅ (V13) | - | - | - |

**Key**: Migration version shown in parentheses (e.g., V7 = Migration V7 schema)

---

## Migration Roadmap

### Phase 1: Production State ✅ Available Now

**Status**: Production Ready (v0.13.0)
**Timeline**: January 2025

**Components**:
- Agent State (Sled → PostgreSQL)
- Workflow State (Sled → PostgreSQL)
- Sessions (Sled → PostgreSQL)

**Migration Path**: Simple key-value migrations
**Documentation**: [Migration Guide](./migration-guide.md)
**Performance**: ~1000 records/second

---

### Phase 2: Memory Systems 📋 Planned

**Status**: In Design
**Timeline**: Q1 2025

**Components**:
- Episodic Memory (HNSW files → PostgreSQL VectorChord)
- Semantic Memory (SurrealDB → PostgreSQL bi-temporal graph)

**Technical Challenges**:
- HNSW binary format parsing
- Vector dimension routing (384/768/1024/1536)
- SurrealDB query translation
- Bi-temporal graph preservation

**Estimated Effort**: 2-3 days per component

---

### Phase 3: Specialized Storage 📋 Planned

**Status**: Planned
**Timeline**: Q1 2025

**Components**:
- Artifacts (Filesystem → PostgreSQL Large Objects)
- Events (Custom adapter → PostgreSQL partitioned log)
- Hooks (Filesystem → PostgreSQL with LZ4 compression)
- API Keys (Encrypted files → PostgreSQL pgcrypto)

**Technical Challenges**:
- Large file streaming (MB-GB artifacts)
- Monthly partition management
- LZ4 compression/decompression
- Encryption key rotation

**Estimated Effort**: 1-2 days per component

---

## Quick Links

### Configuration

**Sled Backend** (default for development):
```toml
[storage]
backend = "sled"
path = "~/.local/share/llmspell/sled"
```

**PostgreSQL Backend** (recommended for production):
```toml
[storage]
backend = "postgres"
connection = "postgresql://llmspell_app:password@localhost:5432/llmspell_dev"
```

**Hybrid Configuration** (Phase 13 default):
```toml
[storage]
backend = "sled"  # Default KV

[storage.memory]
episodic_backend = "hnsw"  # Vector search
semantic_backend = "postgres"  # Bi-temporal graph
procedural_backend = "memory"  # In-memory patterns
```

### CLI Commands

```bash
# Migration
llmspell storage migrate plan --from sled --to postgres --components agent_state --output plan.yaml
llmspell storage migrate execute --plan plan.yaml --dry-run
llmspell storage migrate execute --plan plan.yaml

# Validation
llmspell storage validate --backend sled --components agent_state,workflow_state,sessions

# Info
llmspell storage info --backend postgres
```

---

## Developer Guide

### Implementing New Backends

1. **Implement `StorageBackend` trait** (llmspell-storage/src/traits/storage.rs)
2. **Add backend module** (llmspell-storage/src/backends/)
3. **Create migration schema** (llmspell-storage/migrations/)
4. **Implement `MigrationSource`/`MigrationTarget` traits** (for data migration support)
5. **Add tests** (llmspell-storage/tests/)
6. **Update configuration** (llmspell-config/)

**Example**: See `PostgresBackend` implementation (llmspell-storage/src/backends/postgres/)

---

## Support

- **Migration Issues**: See [Migration Guide Troubleshooting](./migration-guide.md#troubleshooting-guide)
- **Backend Configuration**: Check llmspell-config documentation
- **Performance Tuning**: See PostgreSQL-specific guides (coming in Phase 13b.17)
- **GitHub Issues**: https://github.com/anthropics/llmspell/issues

---

**Last Updated**: January 2025
**Version**: Phase 13b (Storage Migration - Phase 1)
**Next Update**: Phase 2 memory migrations (Q1 2025)
