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
7. [Monitoring & Observability](#monitoring--observability)
8. [Operational Checklists](#operational-checklists)

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
Write: <3ms (Memory: <0.1ms, SQLite: <3ms, RocksDB: <2ms)
Read: <1ms (Memory: <0.1ms, SQLite: <1ms, RocksDB: <0.5ms)
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
- SurrealDB: Not yet benchmarked (graph queries, bi-temporal)
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
- **Sled**: Production, <100K items
- **RocksDB**: High-scale, >100K items

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
- **SurrealDB**: Bi-temporal graph queries, relationship-rich data

#### Configuration
```toml
[memory]
backend = "hnsw"  # or "inmemory", "surrealdb"
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
- Backend-specific metrics (HNSW index size, SurrealDB query time)

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
state_backend = "rocksdb"
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
