# 08. Operations & Performance

**Guide #8 of Developer Guide Series**

**Status**: Production-Ready
**Last Updated**: Phase 13 (v0.13.0)
**Phase Coverage**: 0-13 Complete
**Purpose**: Unified guide for performance, security, and operational concerns

> **📋 Single Source**: This guide consolidates performance benchmarks and security model for operational deployment, providing complete operational reference.

---

## Table of Contents

1. [Performance Overview](#performance-overview)
2. [Security Overview](#security-overview)
3. [Performance Benchmarks](#performance-benchmarks)
4. [Security Implementation](#security-implementation)
5. [Performance Tuning](#performance-tuning)
6. [Security Operations](#security-operations)
7. [Data Migration Operations](#data-migration-operations-phase-13c32)
8. [Monitoring & Observability](#monitoring--observability)
9. [Operational Checklists](#operational-checklists)

---

## Performance Overview

### Achievement Summary

All performance targets met or exceeded across Phases 0-13:

| Metric | Target | Actual | Status |
|--------|--------|--------|--------|
| **Tool Initialization** | <10ms | <10ms | ✅ |
| **Agent Creation** | <50ms | ~10ms | ✅ 5x better |
| **Hook Overhead** | <5% | <2% | ✅ |
| **State Operations** | <5ms write, <1ms read | <3ms, <1ms | ✅ |
| **Event Throughput** | 50K/sec | 90K/sec | ✅ 1.8x better |
| **Vector Search (100K)** | <10ms | 8ms | ✅ Phase 8 |
| **Embedding Generation** | <100ms | ~80ms | ✅ Phase 8 |
| **Multi-tenant Overhead** | <5% | 3% | ✅ Phase 8 |
| **Memory Baseline** | <50MB | 12-15MB | ✅ 3x better |
| **Template Operations** | <10ms | <2ms | ✅ Phase 12 5x better |
| **Memory Storage** | <5ms | <2ms | ✅ Phase 13 2.5x better |
| **Context Assembly** | <50ms | <2ms | ✅ Phase 13 25x better |

---

## Security Overview

### Three-Level Security Model

```rust
pub enum SecurityLevel {
    Safe,       // No file/network access (pure computation)
    Restricted, // Limited, validated access (default)
    Privileged, // Full system access (requires review)
}
```

### Phase 8 Security Enhancements

- **Multi-tenant Isolation**: Complete namespace separation via `StateScope::Custom("tenant:id")`
- **Row-Level Security**: Vector operations filtered by tenant context
- **Audit Logging**: All RAG operations tracked with tenant attribution
- **No Cross-Tenant Leakage**: Compile-time and runtime guarantees

---

## Performance Benchmarks

### Core Components (Phase 0-7)

#### Script Bridge Layer
```
Global Injection: 2-4ms (all 17 globals including RAG)
Sync Bridge Overhead: <1ms per call
Maximum Observed: 5ms (complex operations)
```

#### Tool Execution
| Category | Tools | Init Time | Exec Overhead |
|----------|-------|-----------|---------------|
| Utilities | 10 | <5ms | <1ms |
| File System | 5 | <8ms | <3ms |
| Web | 8 | <10ms | <5ms (local) |
| Data Processing | 3 | <6ms | <2ms |

#### State Persistence
```
Write: <3ms (Memory: <0.1ms, SQLite: <3ms, PostgreSQL: <5ms)
Read: <1ms (Memory: <0.1ms, SQLite: <1ms, PostgreSQL: <2ms)
Migration: 2.07μs/item (483K items/sec)
```

### RAG System Performance (Phase 8)

#### Vector Operations
```
Search Latency (100K vectors, 384 dims):
- Speed-optimized (m=8): 0.5ms @ 85% recall
- Balanced (m=16): 2ms @ 95% recall
- Accuracy-optimized (m=48): 10ms @ 99% recall

Insertion Throughput:
- Batch 32: 5,000 vectors/sec
- Batch 128: 15,000 vectors/sec
- Batch 256: 20,000 vectors/sec
```

#### Memory Usage
```
Per Vector: ~2KB (including metadata)
100K vectors: ~450MB
1M vectors: ~4.5GB
Index overhead: 16 bytes × m × vector_count
```

#### Multi-Tenant Performance
```
Tenant Isolation Overhead: 3%
Namespace Resolution: <0.1ms
Per-tenant Search (10K vectors): <5ms
Concurrent Tenant Operations: Linear scaling to 100+ tenants
```

#### Embedding Performance
```
OpenAI text-embedding-3-small (384 dims):
- Single document: ~80ms
- Batch 32 documents: ~400ms
- Cache hit: <1ms
- Rate limiting: Automatic backoff
```

### Template System Performance (Phase 12)

```
Template Registry Lookup: <2ms (DashMap concurrent access)
Template Instantiation: <2ms (ExecutionContext builder)
Template Execution: <2ms overhead (actual workflow time depends on LLM calls)
```

### Memory System Performance (Phase 13)

```
Memory Storage: <2ms (2.5x better than 5ms target)
Memory Retrieval: <2ms (5x better than 10ms target)
Context Assembly: <2ms (25x better than 50ms target)

HNSW Search (1000 vectors): 8.47ms (8.47x faster than InMemory baseline)

Backend Performance:
- InMemory: 71.68ms (baseline, development/testing)
- HNSW: 8.47ms (8.47x speedup, production)
- SQLite/PostgreSQL: Production backends for bi-temporal graph queries
```

### Resource Usage

#### Memory Profile
```
Startup: 12-15MB baseline
Per Agent: ~500KB
Per Tool: ~100KB
Per Session: ~50KB
Per 1K Vectors: ~2MB
Peak (100 operations): <50MB
```

#### CPU Usage
```
Idle: <0.1%
Active (10 ops/sec): ~2%
Peak (100 ops/sec): ~15%
Vector indexing: Scales with thread count
```

---

## Security Implementation

### Defense Layers

#### 1. Input Validation
**Location**: `llmspell-utils/src/security/validation.rs`
```rust
Features:
- Path traversal prevention (../ detection)
- Input sanitization (encoding normalization)
- Parameter type validation
- Pattern-based malicious input detection
```

#### 2. Sandboxing
**Location**: `llmspell-security/src/sandbox.rs`
```rust
Lua Restrictions:
- Removed: os.execute, io.popen, loadfile, dofile
- File I/O only through Tool interface
- No raw network access

File System Sandbox:
- Whitelist-based path validation
- Symlink resolution
- Size limits enforcement
```

#### 3. Resource Control
**Location**: `llmspell-utils/src/resource_tracker.rs`
```rust
pub struct ResourceLimits {
    pub memory_limit: Option<usize>,      // Bytes
    pub cpu_limit: Option<Duration>,      // Time limit
    pub file_size_limit: Option<usize>,   // Bytes
    pub operation_limit: Option<usize>,   // Count
}
```

### Multi-Tenant Security (Phase 8)

#### Tenant Isolation Architecture
```rust
// Every vector operation includes tenant context
pub struct VectorSecurityContext {
    tenant_id: String,
    namespace: String,  // "tenant:acme-corp"
    policies: Vec<SecurityPolicy>,
    audit_logger: AuditLogger,
}

// Row-level security enforcement
pub enum AccessDecision {
    Allow,
    Deny(String),
    AllowWithFilters(Vec<SecurityFilter>),
}
```

#### Security Boundaries
- **Namespace Separation**: Physical isolation of vector data
- **Query Filtering**: Automatic tenant filter injection
- **Result Masking**: Cross-tenant results filtered out
- **Audit Trail**: Complete operation history per tenant

---

## Performance Tuning

### HNSW Vector Search Tuning

#### Preset Configurations

**Speed-Optimized** (Real-time, ~85% recall):
```toml
[rag.vector_storage.hnsw]
m = 8
ef_construction = 50
ef_search = 25
```

**Balanced** (Default, ~95% recall):
```toml
[rag.vector_storage.hnsw]
m = 16
ef_construction = 200
ef_search = 50
```

**Accuracy-Optimized** (High-stakes, ~99% recall):
```toml
[rag.vector_storage.hnsw]
m = 48
ef_construction = 500
ef_search = 300
```

### State Persistence Tuning

#### Backend Selection
- **Memory**: Development, <1K items
- **SQLite**: Production embedded, <1M items
- **PostgreSQL**: Production scale, multi-tenant deployments

#### Migration Optimization
```rust
// Batch size for optimal throughput
const OPTIMAL_BATCH_SIZE: usize = 1000;
// Achieves 483K items/sec
```

### Memory System Tuning (Phase 13)

#### Backend Selection
- **InMemory**: Development/testing, unlimited scale for experiments
- **HNSW**: Production, 1M+ vectors with 8.47x speedup
- **SQLite/PostgreSQL**: Bi-temporal graph queries, relationship-rich data

#### Configuration
```toml
[memory]
backend = "hnsw"  # or "inmemory",  "sqlite", "postgres"
max_entries = 10000
ttl_seconds = 3600

[memory.hnsw]
m = 16
ef_construction = 200
ef_search = 50
```

### Resource Limit Guidelines

#### By Security Level
```rust
// Safe Level (computation only)
ResourceLimits {
    memory_limit: Some(10 * MB),
    cpu_limit: Some(Duration::from_secs(1)),
    file_size_limit: None,  // No file access
}

// Restricted Level (default)
ResourceLimits {
    memory_limit: Some(100 * MB),
    cpu_limit: Some(Duration::from_secs(5)),
    file_size_limit: Some(10 * MB),
}

// Privileged Level
ResourceLimits {
    memory_limit: Some(1 * GB),
    cpu_limit: Some(Duration::from_secs(30)),
    file_size_limit: Some(100 * MB),
}
```

---

## Security Operations

### Threat Mitigation

#### STRIDE Analysis & Mitigations

| Threat | Mitigation | Implementation |
|--------|------------|----------------|
| **Spoofing** | API key protection | Environment variables, no logging |
| **Tampering** | Input validation | Path sanitization, type checking |
| **Repudiation** | Audit logging | Hook-based event tracking |
| **Information Disclosure** | Error sanitization | Generic messages, path obfuscation |
| **Denial of Service** | Resource limits | Memory/CPU/operation limits |
| **Elevation of Privilege** | Sandboxing | Three-level model, bridge enforcement |

### Security Monitoring

#### Hook-Based Security Events
```rust
pub enum SecurityHooks {
    SecurityViolation,      // Policy violation detected
    ResourceLimitExceeded,  // Limits hit
    SandboxEscape,         // Escape attempt
    AuthenticationFailed,   // Auth failure
    TenantBoundaryViolation, // Cross-tenant attempt
}
```

#### Audit Log Format
```rust
pub struct AuditEntry {
    timestamp: DateTime<Utc>,
    operation: String,          // "vector-searcher", "document-ingest"
    tenant_id: Option<String>,  // Multi-tenant context
    principal: String,          // User/service identity
    resource: String,           // Resource accessed
    decision: AccessDecision,   // Allow/Deny/Filtered
    latency_ms: u32,           // Performance tracking
}
```

---

## Data Migration Operations (Phase 13c.3.2)

### Migration Overview

**Purpose**: Bidirectional PostgreSQL ↔ SQLite data migration for deployment flexibility

**Use Cases**:
- **Development → Production**: SQLite (local) → PostgreSQL (scaled deployment)
- **Production → Development**: PostgreSQL → SQLite (local debugging with prod data)
- **Backend Switching**: Scale up/down based on deployment needs
- **Disaster Recovery**: Restore from JSON export after database failure

### Migration Architecture

**Components**:
- **Export**: `SqliteExporter` / `PostgresExporter` → `ExportFormat` (JSON)
- **Import**: `SqliteImporter` / `PostgresImporter` ← `ExportFormat` (JSON)
- **CLI**: `llmspell storage export/import` commands

**Data Coverage**:
- V3: Vector embeddings (all 4 dimensions: 384, 768, 1536, 3072)
- V4: Knowledge graph (entities + relationships with bi-temporal data)
- V5: Procedural memory patterns
- V6: Agent state
- V7: KV store
- V8: Workflow states
- V9: Sessions
- V10: Artifacts (including binary content)
- V11: Event log
- V13: Hook history

### Migration Procedures

#### Pre-Migration Checklist

**Before starting any migration:**

```bash
# 1. Verify source backend health
llmspell database health --backend <source>

# 2. Check database size
# SQLite:
du -sh /path/to/llmspell.db
# PostgreSQL:
psql $DATABASE_URL -c "SELECT pg_size_pretty(pg_database_size('llmspell_prod'));"

# 3. Ensure sufficient disk space (2x database size)
df -h /target/path

# 4. Backup current data
llmspell storage export --backend <source> --output pre-migration-backup.json

# 5. Verify target backend is ready
llmspell database health --backend <target>
```

#### SQLite → PostgreSQL Migration

**Typical scenario**: Moving from local development to production deployment

```bash
# 1. Export from SQLite
llmspell storage export --backend sqlite --output dev-export.json

# 2. Transfer to production server (if needed)
scp dev-export.json prod-server:/tmp/

# 3. Set PostgreSQL connection
export DATABASE_URL="postgresql://llmspell:password@localhost:5432/llmspell_prod"

# 4. Import to PostgreSQL
llmspell storage import --backend postgres --input dev-export.json

# 5. Verify import statistics
# Expected output:
# ✅ Imported X total records:
#   - Vectors: Y
#   - Entities: Z
#   - Sessions: W
#   ...

# 6. Validation: Export from target and compare
llmspell storage export --backend postgres --output verify-export.json
diff <(jq -S .data dev-export.json) <(jq -S .data verify-export.json)
# Should show no differences (timestamps may vary)
```

#### PostgreSQL → SQLite Migration

**Typical scenario**: Debugging production issues locally

```bash
# 1. Export from PostgreSQL
export DATABASE_URL="postgresql://llmspell:password@localhost:5432/llmspell_prod"
llmspell storage export --backend postgres --output prod-export.json

# 2. Transfer to dev machine
scp prod-server:/tmp/prod-export.json ~/Downloads/

# 3. Import to local SQLite
llmspell storage import --backend sqlite --input ~/Downloads/prod-export.json

# 4. Verify data integrity
llmspell storage export --backend sqlite --output verify-export.json
diff <(jq -S .data prod-export.json) <(jq -S .data verify-export.json)
```

### Migration Performance

**Benchmarks** (Phase 13c.3.2):

| Operation | Dataset Size | Time | Throughput |
|-----------|--------------|------|------------|
| SQLite Export | 10K vectors + 1K entities | <5s | 2.2K records/sec |
| PostgreSQL Export | 10K vectors + 1K entities | <8s | 1.4K records/sec |
| SQLite Import | 10K vectors + 1K entities | <6s | 1.8K records/sec |
| PostgreSQL Import | 10K vectors + 1K entities | <10s | 1.1K records/sec |
| JSON Serialization | 10K records | <2s | 5K records/sec |
| JSON Deserialization | 10K records | <3s | 3.3K records/sec |

**Optimization Tips**:
- Use SSD storage for import/export operations
- Ensure sufficient memory (2x dataset size)
- Disable unnecessary database logging during import
- Use batch operations where possible

### Migration Troubleshooting

#### Issue: Export File Too Large

**Problem**: Disk space exhausted during export

**Solution**:
```bash
# 1. Check available space
df -h /export/path

# 2. Compress export directly
llmspell storage export --backend postgres --output - | gzip > export.json.gz

# 3. Or export to larger partition
llmspell storage export --backend postgres --output /mnt/large-disk/export.json
```

#### Issue: Import Fails with JSON Parse Error

**Problem**: Corrupted or incomplete JSON file

**Solution**:
```bash
# 1. Validate JSON
jq . export.json > /dev/null && echo "Valid" || echo "Invalid"

# 2. Check file completeness
tail -1 export.json  # Should end with }

# 3. Re-export if corrupted
llmspell storage export --backend <source> --output fresh-export.json
```

#### Issue: Import Fails Midway

**Problem**: Database constraint violation or connection timeout

**Solution**:
```bash
# Imports are transaction-safe - automatically rolled back on failure

# 1. Check database logs
# PostgreSQL:
docker logs llmspell_postgres_dev --tail 100
# SQLite:
Check application logs for SQL errors

# 2. Verify migrations applied
llmspell database migrations list

# 3. Retry import (safe due to rollback)
llmspell storage import --backend <target> --input export.json
```

#### Issue: Data Missing After Import

**Problem**: Import succeeded but some data is missing

**Solution**:
```bash
# 1. Check import statistics
# Ensure counts match source database

# 2. Verify tenant filtering (if using multi-tenancy)
# Check config.toml for tenant_id settings

# 3. Export from target and compare
llmspell storage export --backend <target> --output verify.json
jq '.data | keys' verify.json  # Check which tables have data

# 4. Check for PostgreSQL RLS policies (if applicable)
psql $DATABASE_URL -c "SELECT tablename, rowsecurity FROM pg_tables WHERE schemaname = 'public';"
```

### Migration Best Practices

**1. Test with Subset First**:
```bash
# Export small dataset for testing
jq '.data.sessions = .data.sessions[:5]' full-export.json > test-export.json
llmspell storage import --backend <target> --input test-export.json
```

**2. Schedule During Maintenance Window**:
- Production migrations should occur during low-traffic periods
- Allow 2-3x estimated migration time
- Have rollback plan ready

**3. Monitor Resource Usage**:
```bash
# Watch memory and disk during migration
watch -n 1 'free -h && df -h /target/path'

# Monitor database connections (PostgreSQL)
psql $DATABASE_URL -c "SELECT count(*) FROM pg_stat_activity;"
```

**4. Verify Data Integrity**:
```bash
# Always verify after migration
llmspell storage export --backend <target> --output post-migration.json
diff <(jq -S .data pre-migration.json) <(jq -S .data post-migration.json)
```

**5. Keep Export Files for Rollback**:
```bash
# Archive export files with timestamps
mv export.json backups/export-$(date +%Y%m%d-%H%M%S).json

# Compress old backups
gzip backups/export-*.json

# Keep last 7 days of exports
find backups/ -name "export-*.json.gz" -mtime +7 -delete
```

### See Also

- **[Data Migration Guide](../../user-guide/11-data-migration.md)** - Complete migration workflows
- **[Storage Backends](reference/storage-backends.md)** - Export/Import API details
- **[PostgreSQL Guide](../technical/postgresql-guide.md)** - PostgreSQL-specific migration
- **[CLI Reference](../../user-guide/05-cli-reference.md#storage)** - Storage commands

---

## Monitoring & Observability

### Key Metrics

#### System Health
- Memory usage (baseline, peak, per-component)
- CPU utilization (idle, active, peak)
- Event throughput (events/sec)
- Hook execution time (P50, P95, P99)

#### RAG Performance
- Vector search latency (P50, P95, P99)
- Embedding generation time
- Index size and memory usage
- Cache hit rates

#### Memory System Performance (Phase 13)
- Memory storage latency (P50, P95, P99)
- Memory retrieval latency
- Context assembly time
- Backend-specific metrics (HNSW index size, graph query time)

#### Template System Performance (Phase 12)
- Template lookup latency
- Template instantiation time
- Workflow execution time (excluding LLM calls)

#### Security Metrics
- Security violations per hour
- Resource limit hits
- Sandbox escape attempts
- Authentication failures
- Cross-tenant violation attempts

### Performance Regression Detection

```yaml
thresholds:
  latency_increase: 10%    # Investigation required
  memory_increase: 5%      # Review required
  throughput_decrease: 20% # PR blocked
```

---

## Operational Checklists

### Deployment Checklist

#### Performance Validation
- [ ] All benchmarks within targets
- [ ] Memory baseline <15MB
- [ ] Vector search <10ms for expected dataset
- [ ] No memory leaks detected
- [ ] Resource limits configured

#### Security Validation
- [ ] All tools declare security level
- [ ] Sandboxing enabled and tested
- [ ] Input validation active
- [ ] Audit logging configured
- [ ] Multi-tenant isolation verified

### Production Configuration

#### Recommended Settings
```toml
[performance]
event_buffer_size = 10000
state_backend = "sqlite"
migration_batch_size = 1000

[security]
default_level = "restricted"
audit_enabled = true
sandbox_enabled = true
path_validation = true

[rag]
enabled = true
embedding_provider = "openai"
embedding_cache_size = 10000

[rag.vector_storage.hnsw]
m = 16
ef_construction = 200
ef_search = 50
max_elements = 1000000

[memory]
backend = "hnsw"
max_entries = 10000
ttl_seconds = 3600

[templates]
enable_caching = true
max_templates = 100
```

### Incident Response

#### Performance Degradation
1. Check resource limits (memory, CPU)
2. Review hook circuit breakers
3. Analyze vector index fragmentation
4. Check embedding API rate limits
5. Review tenant usage patterns
6. Check memory backend performance (switch InMemory→HNSW if needed)
7. Review template cache hit rates

#### Security Incident
1. Check audit logs for violation patterns
2. Review sandbox escape attempts
3. Analyze cross-tenant access attempts
4. Check for resource exhaustion attacks
5. Review API key exposure risks

---

## Performance Evolution

### Historical Improvements

| Phase | Key Achievement | Performance Impact |
|-------|-----------------|-------------------|
| Phase 1 | Async foundation | Baseline established |
| Phase 2 | Lazy loading | 50% faster startup |
| Phase 3 | Tool standardization | 20% faster validation |
| Phase 4 | Lock-free events | 10x throughput |
| Phase 5 | Batch migrations | 100x faster |
| Phase 6 | Blake3 hashing | 10x faster |
| Phase 7 | Feature tests | 50% faster CI |
| Phase 8 | HNSW vectors | 8ms @ 100K vectors |
| Phase 12 | Template system | <2ms overhead (50x target) |
| Phase 13 | Memory system | <2ms operations (2.5-25x targets) |

### Future Optimization Opportunities

- **LuaJIT Integration**: 2-5x Lua performance
- **SIMD Operations**: 2-3x data processing
- **GPU Acceleration**: 10-100x for embeddings
- **Memory Mapping**: Support for 10M+ vectors
- **Distributed Indexing**: Horizontal scaling
- **Template Pre-compilation**: Faster instantiation
- **Memory Index Sharding**: Multi-backend parallelization

---

## Security Evolution

### Security Maturity Progress

| Phase | Security Enhancement | Impact |
|-------|---------------------|--------|
| Phase 3 | Three-level model | Clear boundaries |
| Phase 4 | Hook monitoring | Real-time detection |
| Phase 5 | State encryption | Data protection |
| Phase 6 | Session isolation | User separation |
| Phase 7 | API standardization | Consistent validation |
| Phase 8 | Multi-tenant RAG | Enterprise-ready |

### Future Security Enhancements

- **Advanced Authentication**: OAuth2, SAML
- **Network Traffic Inspection**: TLS verification
- **Behavioral Analysis**: Anomaly detection
- **Compliance Features**: GDPR, HIPAA support
- **Zero-Trust Architecture**: Complete verification

---

## Related Documentation

- **[Performance Guide](../technical/performance-guide.md)**: Detailed performance targets, benchmarking methodology, profiling tools
- **[User Guide: Configuration](../user-guide/02-configuration.md)**: User-facing configuration reference
- **[Developer Guide: Testing](05-testing.md)**: Testing strategies and quality gates

---

*This operational guide consolidates performance and security documentation for llmspell, validated against Phase 13 implementation.*
