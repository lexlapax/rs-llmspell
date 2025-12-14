# rs-aikit: Master Architecture Vision

**Version**: 1.0.0
**Date**: December 2025
**Status**: Architecture Design Document
**Project Phase**: Pre-Implementation (Reference Design)

> **Vision Statement**: rs-aikit is a production-grade AI agent runtime and specification platform that transforms agent definitions from declarative specifications into executable, observable, and composable services. Built on lessons from rs-llmspell's 14-phase evolution, rs-aikit provides a kernel-centric architecture with native multi-tenancy, comprehensive agent lifecycle management, and seamless scalability from single-user SQLite to enterprise PostgreSQL deployments.

---

## Table of Contents

### Part I: Foundation and Vision
1. [Executive Summary](#executive-summary)
2. [Project Identity and Mission](#project-identity-and-mission)
3. [Architectural Principles](#architectural-principles)
4. [Key Innovations](#key-innovations)

### Part II: Agent Specification Architecture
5. [6-Layer Agent Specification Model](#6-layer-agent-specification-model)
6. [Agent Specification Storage](#agent-specification-storage)
7. [Specification Lifecycle Management](#specification-lifecycle-management)
8. [Specification Validation and Schema](#specification-validation-and-schema)

### Part III: Core System Architecture
9. [Kernel-Centric Architecture](#kernel-centric-architecture)
10. [5-Crate Workspace Structure](#5-crate-workspace-structure)
11. [Protocol and Transport Abstraction](#protocol-and-transport-abstraction)
12. [Execution Modes](#execution-modes)

### Part IV: Multi-Tenancy Architecture
13. [Multi-Tenancy Design Principles](#multi-tenancy-design-principles)
14. [SQLite Multi-Tenancy (Default)](#sqlite-multi-tenancy-default)
15. [PostgreSQL Multi-Tenancy (Production)](#postgresql-multi-tenancy-production)
16. [Tenant Isolation and Security](#tenant-isolation-and-security)

### Part V: Storage and Persistence
17. [3-Tier Storage Architecture](#3-tier-storage-architecture)
18. [Vector Storage and RAG](#vector-storage-and-rag)
19. [Graph Storage and Temporal Knowledge](#graph-storage-and-temporal-knowledge)
20. [Migration and Backup](#migration-and-backup)

### Part VI: Scripting and Integration
21. [Multi-Language Scripting Bridge](#multi-language-scripting-bridge)
22. [Lua Integration](#lua-integration)
23. [JavaScript Integration](#javascript-integration)
24. [Agent Specification DSL](#agent-specification-dsl)

### Part VII: Infrastructure and Operations
25. [Configuration System](#configuration-system)
26. [Messaging and Communication](#messaging-and-communication)
27. [Security Architecture](#security-architecture)
28. [Observability and Monitoring](#observability-and-monitoring)

### Part VIII: Implementation and Evolution
29. [Technology Stack](#technology-stack)
30. [Performance Targets](#performance-targets)
31. [Implementation Roadmap](#implementation-roadmap)
32. [Migration from rs-llmspell](#migration-from-rs-llmspell)

---

## Executive Summary

### The rs-aikit Vision

rs-aikit reimagines AI agent development by treating **agent specifications as first-class citizens** stored, versioned, and executed like code. Unlike rs-llmspell's experimental rapid-iteration platform, rs-aikit is purpose-built for **production deployment** with:

- **Declarative Agent Specifications**: 6-layer specification model (Identity, Interface, Integration, Context, Memory, Intelligence, Behavior, Network) that can be stored in databases, versioned in git, and streamed over networks
- **Kernel-Centric Runtime**: Single integrated kernel handling multiple execution modes (in-process, daemon, cluster) with protocol/transport abstraction
- **Native Multi-Tenancy**: Built-in tenant isolation from day one, scaling from single-user SQLite to enterprise PostgreSQL with Row-Level Security
- **Specification-Driven Configuration**: Agent specs define not just behavior but complete infrastructure (memory backends, storage choices, network topology)
- **Hot-Swappable Backends**: Swap between in-memory, SQLite, and PostgreSQL storage without code changes
- **Zero-Trust Security**: Multi-layer security from specification validation through runtime sandboxing and database-level RLS

### What Makes rs-aikit Different

| Capability | rs-llmspell (Experimental) | rs-aikit (Production) |
|------------|---------------------------|----------------------|
| **Primary Focus** | Rapid AI experimentation via scripts | Production agent deployment via specs |
| **Agent Definition** | Lua/JS code | Declarative YAML/JSON specifications |
| **Multi-Tenancy** | Opt-in via StateScope | Built-in from architecture foundation |
| **Storage** | SQLite unified (Phase 13c) | SQLite (default) + PostgreSQL (production) with automatic tenant isolation |
| **Scaling** | Single instance | Horizontal scaling via kernel clustering |
| **Configuration** | TOML profiles | Agent spec + layered profiles + tenant config |
| **Security** | Application-level | Database RLS + application + specification validation |
| **Target User** | AI researchers, experimenters | SaaS developers, enterprises, multi-tenant platforms |

### Core Architecture Pillars

1. **Agent Specification as Database Entity**: Agent definitions stored in `agent_specifications` table with full CRUD, versioning, and query capabilities
2. **Kernel as Service**: Unified kernel runtime deployable as daemon, library, or cluster node with ZeroMQ messaging
3. **Tenant-First Design**: Every table has `tenant_id`, every query filtered by RLS policies, zero data leakage
4. **Specification-Driven Infrastructure**: Agent spec declares memory backend (InMemory/HNSW/SQLite/PostgreSQL), storage choices, network topology
5. **Language-Agnostic Bridge**: Lua/JavaScript wrappers for Rust implementations, maintaining script-first philosophy while enforcing safety
6. **Comprehensive Observability**: OpenTelemetry integration, distributed tracing, metrics, and logs correlated by tenant and agent spec version

### Quantitative Targets

**Performance**:
- Agent spec load: <20ms (from database)
- Agent instantiation: <100ms (from spec)
- Tenant isolation overhead: <5% (via RLS)
- Vector search (1M vectors): <50ms P95
- Spec validation: <10ms

**Scalability**:
- Concurrent tenants: 10,000+ (PostgreSQL mode)
- Agents per tenant: 1,000+
- Concurrent agent executions: 100+ (per kernel instance)
- Kernel instances: Horizontally scalable (ZeroMQ mesh)

**Reliability**:
- Agent spec schema validation: 100%
- Tenant data isolation: 100% (enforced by RLS)
- Zero-downtime spec updates: Yes (versioned specs)
- Automatic failover: Yes (kernel clustering)

---

## Project Identity and Mission

### Mission Statement

**Enable any organization to deploy, manage, and scale AI agent workloads with production-grade multi-tenancy, declarative specifications, and zero-trust security.**

### Identity: rs-aikit vs rs-llmspell

**rs-llmspell Philosophy**:
> "Cast scripting spells to explore AI concepts, extract proven patterns to production-ready Rust"
- **Audience**: AI researchers, rapid experimenters
- **Workflow**: Write Lua → Test immediately → Extract to Rust when validated
- **Goal**: Velocity and iteration speed

**rs-aikit Philosophy**:
> "Deploy agents as specifications, not scripts. Scale with confidence, not hope."
- **Audience**: SaaS developers, platform engineers, enterprises
- **Workflow**: Define agent spec → Store in database → Deploy at scale → Monitor and iterate
- **Goal**: Production reliability and multi-tenant isolation

### Design Lineage

rs-aikit inherits proven patterns from rs-llmspell's 14-phase evolution:

**Phase 0-3 (Foundation)**: Core traits, execution runtime, tools library → **rs-aikit adopts**: BaseAgent, Tool, Workflow traits, 40+ production tools

**Phase 4-6 (Infrastructure)**: Hooks, state persistence, sessions → **rs-aikit adopts**: Event-driven hooks, multi-backend state, artifact storage

**Phase 7 (Consolidation)**: API standardization, builder patterns → **rs-aikit adopts**: Consistent API surface, universal builders

**Phase 8 (RAG)**: HNSW vector storage, multi-tenant namespaces → **rs-aikit extends**: Spec-driven RAG configuration, per-agent vector stores

**Phase 9-10 (Kernel)**: Integrated kernel, daemon mode, signal handling → **rs-aikit adopts**: Kernel-centric design, ZeroMQ messaging, graceful shutdown

**Phase 11 (Local LLM)**: Ollama + Candle integration → **rs-aikit adopts**: Dual-backend local LLM support

**Phase 12 (Templates)**: 10 workflow templates → **rs-aikit transforms**: Templates become agent specification presets

**Phase 13 (Memory)**: Adaptive memory with hot-swappable backends → **rs-aikit adopts**: 3-tier memory architecture, context engineering

**Phase 13b (Storage)**: PostgreSQL with VectorChord and RLS → **rs-aikit adopts**: Multi-tenancy foundation, RLS patterns

**Phase 13c (Consolidation)**: Unified SQLite storage, vectorlite-rs HNSW → **rs-aikit adopts**: Single-file storage for dev mode

**Phase 14 (Web Interface)**: Mission Control UI → **rs-aikit extends**: Spec editor, tenant dashboard, agent deployment UI

### rs-aikit's New Contributions

1. **Agent Specification Format**: Declarative 6-layer spec (Identity/Interface/Integration/Context/Memory/Intelligence/Behavior/Network)
2. **Specification Storage**: Database-first agent definitions with versioning and history
3. **Native Multi-Tenancy**: Built into architecture from day one (not retrofitted)
4. **Tenant-Aware Configuration**: Layered config (system → tenant → agent → execution)
5. **Specification Validation**: JSON Schema validation, semantic checks, resource limits
6. **Kernel Clustering**: Horizontal scaling via ZeroMQ mesh topology
7. **Specification Streaming**: Large agent specs transmitted via chunked protocol
8. **Cross-Tenant Analytics**: Aggregated metrics while maintaining isolation

---

## Architectural Principles

### 1. Specification-Driven Everything

**Principle**: If it can be declared, it should be declared. Code is runtime, specs are design time.

```yaml
# Agent specification defines infrastructure choices
spec_version: "1.0.0"
identity:
  name: "customer-support-agent"
  version: "2.3.1"
  tenant_id: "acme-corp"  # Multi-tenancy from spec

memory:
  backend: "postgresql"  # Hot-swappable based on tenant
  episodic:
    provider: "hnsw"
    namespace: "{{tenant_id}}/{{agent_id}}/episodic"  # Tenant isolation
  semantic:
    provider: "postgresql_graph"
    rls_enabled: true  # Database-level isolation

storage:
  artifacts:
    backend: "postgresql"
    retention_days: 90
    tenant_quota_gb: 100  # Per-tenant limits

network:
  a2a_enabled: true
  discovery:
    namespace: "{{tenant_id}}/agents"  # Tenant-scoped discovery
```

**Implementation**: Agent specifications are validated at multiple levels:
1. **Schema validation** (JSON Schema): Structure correctness
2. **Semantic validation**: Logical consistency (e.g., can't use PostgreSQL backend with sqlite:// connection string)
3. **Resource validation**: Tenant quotas, rate limits
4. **Security validation**: Permission checks, sandbox constraints

### 2. Tenant Isolation as Foundation

**Principle**: Multi-tenancy is not a feature—it's the architecture. Every design decision considers tenant boundaries.

**Enforcement Layers**:
1. **Database RLS**: PostgreSQL Row-Level Security blocks queries across tenants
2. **Application Logic**: Tenant context in every request (SessionVariable pattern)
3. **Specification Scope**: Agent specs scoped to tenant namespace
4. **Network Segmentation**: Tenant-specific discovery and messaging
5. **Resource Quotas**: CPU, memory, storage, and API rate limits per tenant

**SQLite Fallback**: Even in single-user SQLite mode, tenant_id filtering is enforced to maintain compatibility when migrating to PostgreSQL.

### 3. Kernel as Universal Runtime

**Principle**: One kernel, many execution modes. Protocol/transport abstraction enables diverse deployment patterns.

**Execution Modes**:
- **In-Process**: Kernel embedded in application (library mode)
- **Daemon**: Long-running background service (systemd/launchd)
- **Cluster Node**: Horizontal scaling via ZeroMQ mesh
- **Serverless**: Kubernetes pod with auto-scaling

**Protocol Support**:
- **Jupyter**: Interactive notebook integration
- **LSP**: IDE integration for agent spec editing
- **DAP**: Debugging agent execution
- **HTTP/WebSocket**: Web UI and REST API
- **ZeroMQ**: Inter-kernel communication

### 4. Progressive Complexity

**Principle**: Simple for single users, powerful for enterprises. Default to simplicity, opt into complexity.

**Deployment Tiers**:

| Tier | Storage | Tenancy | Deployment | Use Case |
|------|---------|---------|------------|----------|
| **Developer** | SQLite (single file) | Default tenant (current user) | In-process | Local development, testing |
| **Team** | SQLite (shared) | Manual tenant assignment | Daemon (single host) | Small teams, startups |
| **Production** | PostgreSQL (managed) | Automatic tenant isolation (RLS) | Daemon cluster | SaaS platforms |
| **Enterprise** | PostgreSQL (distributed) | Hierarchical tenants (org/team/user) | Kubernetes (auto-scale) | Large enterprises |

**Configuration Progression**:
```toml
# Developer: Zero config (all defaults)
# ~/.aikit/config.toml created automatically

# Team: Minimal config
[database]
backend = "sqlite"
path = "/shared/aikit/storage.db"

[tenancy]
default_tenant = "team-alpha"

# Production: Full config with RLS
[database]
backend = "postgresql"
connection_string = "postgres://aikit:***@db.example.com/aikit"
pool_size = 20

[tenancy]
mode = "automatic"  # Enforce tenant_id on all queries
rls_enabled = true  # Use PostgreSQL RLS policies
default_tenant_id = "system"  # Fallback for admin operations

[security]
tenant_isolation = "strict"  # Fail on missing tenant context
cross_tenant_queries = "forbidden"
```

### 5. Observable by Default

**Principle**: If you can't measure it, you can't improve it. Every operation is traceable.

**Observability Stack**:
- **Distributed Tracing**: OpenTelemetry with trace_id correlated to tenant_id and agent_spec_id
- **Structured Logging**: JSON logs with tenant context, agent metadata
- **Metrics**: Prometheus exposition with per-tenant cardinality limits
- **Events**: Event sourcing for agent lifecycle, specification changes

**Example Trace**:
```rust
#[tracing::instrument(
    name = "agent.execute",
    skip(self, input),
    fields(
        tenant_id = %self.spec.identity.tenant_id,
        agent_id = %self.spec.identity.agent_id,
        agent_version = %self.spec.identity.version,
        execution_id = %execution_id,
    )
)]
async fn execute(&self, input: AgentInput) -> Result<AgentOutput> {
    // All database queries, LLM calls, tool invocations automatically
    // inherit trace context with tenant isolation
}
```

### 6. Security in Depth

**Principle**: Zero-trust architecture. Validate at every boundary.

**Security Layers**:
1. **Input Validation**: Agent spec schema validation before storage
2. **Authorization**: Tenant-scoped permissions (can user X modify spec Y?)
3. **Specification Sandbox**: Declared resources only (no filesystem access unless spec declares `tools.file_operations`)
4. **Runtime Sandbox**: Lua/JS execution in controlled environment
5. **Database RLS**: PostgreSQL policies prevent cross-tenant data access
6. **Network Policies**: Agent-to-agent communication restricted by spec
7. **Audit Trail**: All operations logged with tenant attribution

---

## 6-Layer Agent Specification Model

### Overview

rs-aikit adopts the comprehensive 6-layer agent specification model synthesized from industry standards (Oracle Open Agent Spec, Eclipse LMOS ADL, Microsoft Copilot Manifest) with enhancements for multi-tenancy and production deployment.

```
┌─────────────────────────────────────────────────────────────┐
│  Layer 6: NETWORK & COLLABORATION                          │
│  Agent-to-agent communication, discovery, delegation        │
│  Fields: network, a2a, discovery, delegation                │
│  Multi-Tenant Enhancement: Tenant-scoped discovery          │
├─────────────────────────────────────────────────────────────┤
│  Layer 5: ORCHESTRATION & WORKFLOW                         │
│  Control flow, state machines, task composition             │
│  Fields: behavior, workflow, orchestration, state_schema    │
│  Multi-Tenant Enhancement: Tenant resource quotas           │
├─────────────────────────────────────────────────────────────┤
│  Layer 4: COGNITION & REASONING                            │
│  LLM configuration, prompting, output parsing               │
│  Fields: intelligence, cognition, model, prompts            │
│  Multi-Tenant Enhancement: Per-tenant model routing         │
├─────────────────────────────────────────────────────────────┤
│  Layer 3: PERSISTENCE & MEMORY                             │
│  Long-term storage, knowledge retention, state              │
│  Fields: memory, storage, knowledge_bases                   │
│  Multi-Tenant Enhancement: Tenant-isolated backends         │
├─────────────────────────────────────────────────────────────┤
│  Layer 2: CONTEXT ENGINEERING                              │
│  Dynamic context assembly, RAG, compression                 │
│  Fields: context_policy, context_mounts, rag                │
│  Multi-Tenant Enhancement: Tenant-scoped RAG namespaces     │
├─────────────────────────────────────────────────────────────┤
│  Layer 1: INTEGRATION & TOOLS                              │
│  External system connectivity, tool execution               │
│  Fields: capabilities, tools, mcp_servers, integration      │
│  Multi-Tenant Enhancement: Tenant permission policies       │
└─────────────────────────────────────────────────────────────┘

Cross-Cutting Concerns (Metadata):
├─ Identity: Tenant-scoped unique IDs, versioning
├─ Interface: Input/output schemas with tenant context
├─ Governance: Per-tenant security policies, rate limits
└─ Observability: Tenant-attributed tracing and metrics
```

### Specification Structure

```typescript
// TypeScript schema (portable to Rust via serde and schemars)
interface AgentSpecification {
  // Spec metadata
  spec_version: string;           // e.g., "aikit-v1.0.0"
  format_version: string;         // Serialization format compatibility

  // Cross-cutting metadata
  identity: IdentityBlock;
  interface: InterfaceBlock;
  governance: GovernanceBlock;
  observability: ObservabilityBlock;

  // 6-layer specification
  integration: IntegrationBlock;          // Layer 1: Tools and MCP
  context_policy: ContextPolicyBlock;     // Layer 2: RAG and context
  memory: MemoryBlock;                    // Layer 3: Memory backends
  intelligence: IntelligenceBlock;        // Layer 4: LLM configuration
  behavior: BehaviorBlock;                // Layer 5: Workflow orchestration
  network: NetworkBlock;                  // Layer 6: A2A communication
}

interface IdentityBlock {
  agent_id: string;               // Unique within tenant (UUID or slug)
  tenant_id: string;              // Multi-tenancy foundation
  name: string;
  version: string;                // Semantic versioning
  description: string;
  author: string;
  license: string;
  tags: string[];
  created_at: string;             // ISO 8601 timestamp
  updated_at: string;
}

interface MemoryBlock {
  enabled: boolean;
  backend: "inmemory" | "sqlite" | "postgresql";  // Hot-swappable
  episodic: {
    provider: "inmemory" | "hnsw";
    namespace: string;            // Template: "{{tenant_id}}/{{agent_id}}/episodic"
    retention_days?: number;
    max_entries?: number;
  };
  semantic: {
    provider: "sqlite_graph" | "postgresql_graph";
    namespace: string;
    enable_temporal: boolean;     // Bi-temporal graph modeling
    rls_enabled?: boolean;        // PostgreSQL RLS for tenant isolation
  };
  procedural: {
    provider: "sqlite" | "postgresql";
    table_prefix?: string;
  };
}

interface GovernanceBlock {
  security: {
    tenant_isolation: "strict" | "relaxed";
    allowed_tools: string[];      // Whitelist of permitted tools
    denied_tools: string[];       // Blacklist (takes precedence)
    resource_limits: {
      max_concurrent_executions: number;
      max_memory_mb: number;
      max_cpu_percent: number;
      max_storage_mb: number;
    };
  };
  compliance: {
    data_retention_days: number;
    gdpr_compliant: boolean;
    audit_level: "none" | "basic" | "detailed";
  };
  rate_limits: {
    requests_per_minute: number;
    llm_tokens_per_day: number;
    cost_limit_usd_per_day?: number;
  };
}
```

### Specification Validation Levels

**1. Schema Validation (Compile-time)**:
```rust
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct AgentSpecification {
    pub spec_version: String,
    pub identity: IdentityBlock,
    pub memory: MemoryBlock,
    // ... other blocks
}

// Automatic JSON Schema generation
let schema = schemars::schema_for!(AgentSpecification);
```

**2. Semantic Validation (Load-time)**:
```rust
impl AgentSpecification {
    pub fn validate(&self, tenant_context: &TenantContext) -> Result<(), ValidationError> {
        // Check tenant_id matches context
        if self.identity.tenant_id != tenant_context.tenant_id {
            return Err(ValidationError::TenantMismatch);
        }

        // Validate memory backend matches connection string
        if self.memory.backend == MemoryBackend::PostgreSQL {
            if !tenant_context.has_postgres_connection() {
                return Err(ValidationError::BackendUnavailable);
            }
        }

        // Check tool permissions
        for tool_name in &self.integration.tools {
            if !tenant_context.is_tool_permitted(tool_name) {
                return Err(ValidationError::ToolNotPermitted(tool_name.clone()));
            }
        }

        // Validate resource limits within tenant quotas
        if self.governance.security.resource_limits.max_storage_mb >
           tenant_context.quota_storage_mb {
            return Err(ValidationError::ExceedsQuota("storage"));
        }

        Ok(())
    }
}
```

**3. Runtime Validation (Execution-time)**:
```rust
impl AgentRuntime {
    async fn execute(&self, input: AgentInput) -> Result<AgentOutput> {
        // Enforce rate limits
        self.rate_limiter.check_limit(&self.spec.identity.tenant_id).await?;

        // Enforce resource limits (memory, CPU)
        self.resource_monitor.check_limits(&self.spec.governance.security.resource_limits)?;

        // Execute with tenant context
        let output = self.execute_with_isolation(input).await?;

        // Update metrics (tenant-attributed)
        self.metrics.record_execution(
            &self.spec.identity.tenant_id,
            &self.spec.identity.agent_id,
            output.duration,
        );

        Ok(output)
    }
}
```

---

## Agent Specification Storage

### Database Schema

```sql
-- Core agent specifications table
CREATE TABLE agent_specifications (
    -- Identity
    agent_id UUID NOT NULL,
    tenant_id UUID NOT NULL,  -- Multi-tenancy foundation
    version VARCHAR(50) NOT NULL,

    -- Specification content (JSONB for efficient querying)
    spec JSONB NOT NULL,

    -- Metadata
    status VARCHAR(20) NOT NULL DEFAULT 'draft',  -- draft | active | deprecated
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_by VARCHAR(255),

    -- Indexes
    PRIMARY KEY (tenant_id, agent_id, version),

    -- Efficient JSON querying
    CONSTRAINT spec_valid_json CHECK (jsonb_typeof(spec) = 'object')
);

-- Indexes for efficient queries
CREATE INDEX idx_agent_specs_tenant ON agent_specifications(tenant_id);
CREATE INDEX idx_agent_specs_status ON agent_specifications(tenant_id, status);
CREATE INDEX idx_agent_specs_tags ON agent_specifications USING GIN ((spec->'identity'->'tags'));

-- PostgreSQL RLS for tenant isolation
ALTER TABLE agent_specifications ENABLE ROW LEVEL SECURITY;

CREATE POLICY agent_spec_tenant_isolation ON agent_specifications
    USING (tenant_id::text = current_setting('aikit.current_tenant_id', true));

-- Version history tracking
CREATE TABLE agent_specification_history (
    id BIGSERIAL PRIMARY KEY,
    tenant_id UUID NOT NULL,
    agent_id UUID NOT NULL,
    version VARCHAR(50) NOT NULL,
    spec JSONB NOT NULL,
    change_type VARCHAR(20) NOT NULL,  -- created | updated | deprecated
    changed_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    changed_by VARCHAR(255),
    change_reason TEXT,

    FOREIGN KEY (tenant_id, agent_id, version)
        REFERENCES agent_specifications(tenant_id, agent_id, version)
);

-- SQLite schema (compatible subset)
-- Note: No native RLS, enforced at application layer
CREATE TABLE agent_specifications (
    agent_id TEXT NOT NULL,
    tenant_id TEXT NOT NULL,  -- Always "default" for single-user mode
    version TEXT NOT NULL,
    spec TEXT NOT NULL,  -- JSON as TEXT
    status TEXT NOT NULL DEFAULT 'draft',
    created_at TEXT NOT NULL,  -- ISO 8601
    updated_at TEXT NOT NULL,
    created_by TEXT,

    PRIMARY KEY (tenant_id, agent_id, version),
    CHECK (json_valid(spec))
);

CREATE INDEX idx_agent_specs_tenant_sqlite ON agent_specifications(tenant_id);
CREATE INDEX idx_agent_specs_status_sqlite ON agent_specifications(tenant_id, status);
```

### Specification CRUD Operations

```rust
use sqlx::{PgPool, SqlitePool};
use uuid::Uuid;

pub struct AgentSpecRepository {
    pool: StorageBackend,
    tenant_id: Uuid,
}

pub enum StorageBackend {
    Sqlite(SqlitePool),
    Postgresql(PgPool),
}

impl AgentSpecRepository {
    // Create new agent specification
    pub async fn create(&self, spec: &AgentSpecification) -> Result<()> {
        // Validate before storage
        spec.validate(&TenantContext { tenant_id: self.tenant_id })?;

        match &self.pool {
            StorageBackend::Postgresql(pool) => {
                // Set session tenant context for RLS
                sqlx::query("SELECT set_config('aikit.current_tenant_id', $1, false)")
                    .bind(self.tenant_id.to_string())
                    .execute(pool)
                    .await?;

                sqlx::query(
                    "INSERT INTO agent_specifications
                     (agent_id, tenant_id, version, spec, status, created_by)
                     VALUES ($1, $2, $3, $4, $5, $6)"
                )
                .bind(spec.identity.agent_id)
                .bind(spec.identity.tenant_id)
                .bind(&spec.identity.version)
                .bind(serde_json::to_value(spec)?)  // JSONB
                .bind("draft")
                .bind(&spec.identity.author)
                .execute(pool)
                .await?;
            }
            StorageBackend::Sqlite(pool) => {
                // Application-level tenant filtering
                if spec.identity.tenant_id != self.tenant_id {
                    return Err(Error::TenantMismatch);
                }

                sqlx::query(
                    "INSERT INTO agent_specifications
                     (agent_id, tenant_id, version, spec, status, created_by, created_at, updated_at)
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)"
                )
                .bind(&spec.identity.agent_id)
                .bind(&spec.identity.tenant_id)
                .bind(&spec.identity.version)
                .bind(serde_json::to_string(spec)?)  // JSON as TEXT
                .bind("draft")
                .bind(&spec.identity.author)
                .bind(chrono::Utc::now().to_rfc3339())
                .bind(chrono::Utc::now().to_rfc3339())
                .execute(pool)
                .await?;
            }
        }

        Ok(())
    }

    // Retrieve specification
    pub async fn get(&self, agent_id: &str, version: &str) -> Result<AgentSpecification> {
        match &self.pool {
            StorageBackend::Postgresql(pool) => {
                // RLS automatically filters by tenant
                sqlx::query("SELECT set_config('aikit.current_tenant_id', $1, false)")
                    .bind(self.tenant_id.to_string())
                    .execute(pool)
                    .await?;

                let row = sqlx::query(
                    "SELECT spec FROM agent_specifications
                     WHERE agent_id = $1 AND version = $2"
                )
                .bind(agent_id)
                .bind(version)
                .fetch_one(pool)
                .await?;

                let spec: AgentSpecification = serde_json::from_value(row.get("spec"))?;
                Ok(spec)
            }
            StorageBackend::Sqlite(pool) => {
                // Explicit tenant filtering
                let row = sqlx::query(
                    "SELECT spec FROM agent_specifications
                     WHERE tenant_id = ?1 AND agent_id = ?2 AND version = ?3"
                )
                .bind(&self.tenant_id.to_string())
                .bind(agent_id)
                .bind(version)
                .fetch_one(pool)
                .await?;

                let spec: AgentSpecification = serde_json::from_str(row.get("spec"))?;
                Ok(spec)
            }
        }
    }

    // List specifications with filtering
    pub async fn list(&self, filters: SpecFilters) -> Result<Vec<AgentSpecification>> {
        // PostgreSQL: Efficient JSONB queries with GIN indexes
        // SQLite: JSON extraction with json_extract()
        // Both: Tenant isolation enforced
        todo!()
    }

    // Update specification (creates new version)
    pub async fn update(&self, agent_id: &str, spec: &AgentSpecification) -> Result<()> {
        // Semantic versioning: increment patch version
        // Record in history table
        // Maintain RLS context
        todo!()
    }

    // Promote specification (draft → active)
    pub async fn promote(&self, agent_id: &str, version: &str) -> Result<()> {
        // Change status to 'active'
        // Optionally deprecate previous active version
        todo!()
    }
}
```

---

## Kernel-Centric Architecture

### Integrated Kernel Design

rs-aikit kernel is a unified runtime managing agent lifecycle, specification execution, and tenant isolation.

```rust
pub struct AiKitKernel<P: Protocol> {
    // Core execution
    spec_repository: Arc<AgentSpecRepository>,
    agent_registry: Arc<AgentRegistry>,
    protocol: P,
    transport: Option<Box<dyn Transport>>,

    // I/O and messaging
    io_runtime: &'static Runtime,  // Global IO runtime (no "dispatch task is gone")
    message_router: Arc<MessageRouter>,
    event_bus: Arc<EventBus>,

    // Tenant context
    tenant_manager: Arc<TenantManager>,
    current_tenant: Option<TenantContext>,

    // Storage backends (tenant-aware)
    storage: Arc<StorageBackend>,
    memory_backend: Arc<dyn MemoryBackend>,
    vector_store: Arc<dyn VectorStore>,

    // Scripting bridges
    lua_engine: Arc<LuaEngine>,
    js_engine: Arc<JsEngine>,

    // Security and observability
    security_context: Arc<SecurityContext>,
    tracer: Arc<Tracer>,
    metrics: Arc<MetricsCollector>,

    // Production features
    shutdown_coordinator: Arc<ShutdownCoordinator>,
    health_monitor: Arc<HealthMonitor>,
}

impl<P: Protocol> AiKitKernel<P> {
    /// Load and instantiate agent from specification
    pub async fn load_agent(&self, agent_id: &str, version: &str) -> Result<AgentHandle> {
        // Set tenant context
        let tenant_id = self.current_tenant
            .as_ref()
            .ok_or(Error::MissingTenantContext)?
            .tenant_id;

        // Retrieve spec from database (RLS enforced)
        let spec = self.spec_repository.get(agent_id, version).await?;

        // Validate spec for current tenant
        spec.validate(&self.current_tenant.as_ref().unwrap())?;

        // Initialize agent infrastructure from spec
        let memory = self.initialize_memory_from_spec(&spec.memory).await?;
        let tools = self.initialize_tools_from_spec(&spec.integration).await?;
        let llm_client = self.initialize_llm_from_spec(&spec.intelligence).await?;

        // Create agent runtime
        let agent = Agent::from_spec(spec, memory, tools, llm_client)?;

        // Register agent (tenant-scoped)
        let handle = self.agent_registry.register(tenant_id, agent).await?;

        Ok(handle)
    }

    /// Execute agent with input
    pub async fn execute_agent(&self, handle: AgentHandle, input: AgentInput) -> Result<AgentOutput> {
        // Retrieve agent (tenant isolation enforced)
        let agent = self.agent_registry.get(&handle)?;

        // Execute with tracing (tenant-attributed)
        let span = tracing::info_span!(
            "agent.execute",
            tenant_id = %agent.spec.identity.tenant_id,
            agent_id = %agent.spec.identity.agent_id,
        );

        let output = agent.execute(input).instrument(span).await?;

        Ok(output)
    }

    /// Main kernel loop
    pub async fn run(&mut self) -> Result<()> {
        loop {
            tokio::select! {
                // Protocol messages (Jupyter, LSP, HTTP)
                Some(msg) = self.transport.as_ref().unwrap().recv("shell") => {
                    self.handle_message(msg).await?;
                }

                // Shutdown signal
                _ = self.shutdown_coordinator.wait_for_shutdown() => {
                    self.graceful_shutdown().await?;
                    break;
                }

                // Health checks
                _ = tokio::time::sleep(Duration::from_secs(30)) => {
                    self.health_monitor.check_health().await?;
                }
            }
        }

        Ok(())
    }
}
```

### Protocol/Transport Abstraction

```rust
// Protocol layer: Message semantics
pub trait Protocol: Send + Sync {
    fn parse_message(&self, data: &[u8]) -> Result<Message>;
    fn create_response(&self, msg: &Message, content: Value) -> Result<Vec<u8>>;
}

// Transport layer: Message delivery
pub trait Transport: Send + Sync {
    async fn bind(&mut self, config: &TransportConfig) -> Result<()>;
    async fn recv(&self, channel: &str) -> Result<Option<Vec<Vec<u8>>>>;
    async fn send(&self, channel: &str, parts: Vec<Vec<u8>>) -> Result<()>;
}

// Implementations
pub struct JupyterProtocol;  // Jupyter message spec
pub struct LSPProtocol;      // Language Server Protocol
pub struct HTTPProtocol;     // REST API

pub struct ZeroMQTransport;  // ZMQ sockets
pub struct WebSocketTransport;  // WS for web UI
pub struct InProcessTransport;  // Channels for library mode
```

---

## Multi-Tenancy Architecture

### SQLite Multi-Tenancy (Default)

**Design**: Application-level tenant filtering with single-user default.

```rust
pub struct SqliteTenantManager {
    pool: SqlitePool,
    default_tenant_id: Uuid,  // Current user by default
}

impl SqliteTenantManager {
    pub fn new(pool: SqlitePool) -> Self {
        // Default tenant: current OS user
        let default_tenant_id = Uuid::new_v5(
            &Uuid::NAMESPACE_DNS,
            whoami::username().as_bytes()
        );

        Self { pool, default_tenant_id }
    }

    // Every query enforces tenant_id filtering
    async fn execute_with_tenant<T>(&self, query: &str, tenant_id: Uuid) -> Result<T> {
        // Validate tenant_id present in query
        if !query.to_lowercase().contains("tenant_id") {
            return Err(Error::MissingTenantFilter);
        }

        sqlx::query(query)
            .bind(tenant_id.to_string())
            .fetch_one(&self.pool)
            .await
    }
}

// Example: Agent specification retrieval
async fn get_agent_spec(tenant_id: Uuid, agent_id: &str) -> Result<AgentSpecification> {
    sqlx::query_as(
        "SELECT * FROM agent_specifications
         WHERE tenant_id = ?1 AND agent_id = ?2"
    )
    .bind(tenant_id.to_string())
    .bind(agent_id)
    .fetch_one(&pool)
    .await
}
```

**Tenant Isolation Verification**:
```rust
#[cfg(test)]
mod tests {
    #[tokio::test]
    async fn test_tenant_isolation() {
        let pool = setup_sqlite_pool().await;

        // Create specs for different tenants
        let tenant_a = Uuid::new_v4();
        let tenant_b = Uuid::new_v4();

        create_spec(&pool, tenant_a, "agent-1", spec_a).await;
        create_spec(&pool, tenant_b, "agent-1", spec_b).await;

        // Retrieve with tenant_a context
        let result_a = get_agent_spec(&pool, tenant_a, "agent-1").await.unwrap();
        assert_eq!(result_a, spec_a);

        // Attempt cross-tenant access
        let result_b = get_agent_spec(&pool, tenant_a, "agent-1").await;
        assert!(result_b.is_err());  // Tenant B's agent not visible
    }
}
```

**Migration to PostgreSQL**:
- Schema compatible (tenant_id column exists)
- Application logic unchanged (queries already filter by tenant_id)
- Enable RLS for database-level enforcement

### PostgreSQL Multi-Tenancy (Production)

**Design**: Row-Level Security (RLS) with session variables.

```sql
-- Enable RLS on all tenant-scoped tables
ALTER TABLE agent_specifications ENABLE ROW LEVEL SECURITY;
ALTER TABLE agent_executions ENABLE ROW LEVEL SECURITY;
ALTER TABLE agent_memories ENABLE ROW LEVEL SECURITY;
ALTER TABLE artifacts ENABLE ROW LEVEL SECURITY;

-- Policy: Users can only access their tenant's data
CREATE POLICY tenant_isolation_policy ON agent_specifications
    USING (tenant_id::text = current_setting('aikit.current_tenant_id', true));

CREATE POLICY tenant_isolation_policy ON agent_executions
    USING (tenant_id::text = current_setting('aikit.current_tenant_id', true));

-- Similar policies for all tenant-scoped tables
```

**Rust Implementation**:
```rust
pub struct PostgresTenantManager {
    pool: PgPool,
}

impl PostgresTenantManager {
    // Set tenant context for session
    pub async fn set_tenant_context(&self, tenant_id: Uuid) -> Result<()> {
        sqlx::query("SELECT set_config('aikit.current_tenant_id', $1, false)")
            .bind(tenant_id.to_string())
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    // Execute query with tenant context
    pub async fn with_tenant<F, T>(&self, tenant_id: Uuid, f: F) -> Result<T>
    where
        F: FnOnce(&PgPool) -> BoxFuture<'_, Result<T>>,
    {
        // Set tenant context
        self.set_tenant_context(tenant_id).await?;

        // Execute function
        let result = f(&self.pool).await?;

        Ok(result)
    }
}

// Usage: Tenant context automatically enforced by RLS
async fn get_agent_spec(manager: &PostgresTenantManager, tenant_id: Uuid, agent_id: &str) -> Result<AgentSpecification> {
    manager.with_tenant(tenant_id, |pool| {
        Box::pin(async move {
            // No need for WHERE tenant_id = ... (RLS handles it)
            sqlx::query_as("SELECT * FROM agent_specifications WHERE agent_id = $1")
                .bind(agent_id)
                .fetch_one(pool)
                .await
        })
    }).await
}
```

**Performance Optimization**:
```sql
-- Tenant-partitioned tables for large deployments
CREATE TABLE agent_executions (
    id BIGSERIAL,
    tenant_id UUID NOT NULL,
    agent_id UUID NOT NULL,
    -- ... other fields
) PARTITION BY HASH (tenant_id);

-- Create partitions (automated via script)
CREATE TABLE agent_executions_p0 PARTITION OF agent_executions
    FOR VALUES WITH (MODULUS 10, REMAINDER 0);

CREATE TABLE agent_executions_p1 PARTITION OF agent_executions
    FOR VALUES WITH (MODULUS 10, REMAINDER 1);

-- ... partitions p2-p9

-- RLS policies on partitioned table automatically apply to partitions
```

**Cross-Tenant Analytics** (Admin Operations):
```rust
impl PostgresTenantManager {
    // Bypass RLS for admin aggregations (requires BYPASSRLS privilege)
    pub async fn admin_query<F, T>(&self, f: F) -> Result<T>
    where
        F: FnOnce(&PgPool) -> BoxFuture<'_, Result<T>>,
    {
        // Admin user with BYPASSRLS privilege
        sqlx::query("SET ROLE admin_user")
            .execute(&self.pool)
            .await?;

        let result = f(&self.pool).await?;

        // Reset to application user
        sqlx::query("RESET ROLE")
            .execute(&self.pool)
            .await?;

        Ok(result)
    }
}

// Example: Cross-tenant metrics
async fn get_total_agent_count(manager: &PostgresTenantManager) -> Result<i64> {
    manager.admin_query(|pool| {
        Box::pin(async move {
            let row: (i64,) = sqlx::query_as(
                "SELECT COUNT(*) FROM agent_specifications"
            )
            .fetch_one(pool)
            .await?;

            Ok(row.0)
        })
    }).await
}
```

---

## Configuration System

### Layered Configuration Architecture

rs-aikit configuration system supports agent specifications while maintaining backward compatibility with TOML profiles.

```
┌─────────────────────────────────────────────────────────────┐
│  Layer 4: Execution Context (Runtime)                       │
│  Source: CLI flags, environment variables                   │
│  Example: --tenant-id, AIKIT_LOG_LEVEL                      │
│  Priority: HIGHEST                                           │
├─────────────────────────────────────────────────────────────┤
│  Layer 3: Agent Specification (Per-Agent)                   │
│  Source: agent_specifications table                         │
│  Example: spec.memory.backend = "postgresql"                │
│  Priority: HIGH (agent-specific overrides)                  │
├─────────────────────────────────────────────────────────────┤
│  Layer 2: Tenant Configuration (Per-Tenant)                 │
│  Source: tenant_configs table                               │
│  Example: default_memory_backend, resource_quotas           │
│  Priority: MEDIUM (tenant defaults)                         │
├─────────────────────────────────────────────────────────────┤
│  Layer 1: Profile Configuration (System)                    │
│  Source: TOML files (~/.aikit/profiles/*.toml)              │
│  Example: postgres.toml, ollama-production.toml             │
│  Priority: LOW (system defaults)                            │
└─────────────────────────────────────────────────────────────┘

Configuration Resolution: Higher layers override lower layers
```

### Profile TOML Format (Legacy Compatibility)

```toml
# ~/.aikit/profiles/production.toml
[metadata]
name = "production"
description = "Production deployment with PostgreSQL and multi-tenancy"
version = "1.0.0"

[database]
backend = "postgresql"
connection_string_env = "AIKIT_DATABASE_URL"  # Read from env
pool_size = 20
max_connections = 100

[tenancy]
mode = "automatic"
rls_enabled = true
default_tenant_id = "system"
tenant_discovery = "header"  # Read from X-Tenant-ID header

[memory]
default_backend = "postgresql"
episodic_provider = "hnsw"
semantic_provider = "postgresql_graph"

[security]
tenant_isolation = "strict"
require_tenant_context = true
audit_level = "detailed"

[observability]
tracing_enabled = true
metrics_enabled = true
log_level = "info"
```

### Agent Specification Configuration

```yaml
# Agent spec can override profile defaults
spec_version: "aikit-v1.0.0"

identity:
  agent_id: "customer-support-v2"
  tenant_id: "acme-corp"

memory:
  backend: "hnsw"  # Override profile default (postgresql)
  episodic:
    provider: "hnsw"
    namespace: "{{tenant_id}}/{{agent_id}}/episodic"
    hnsw_config:
      m: 16
      ef_construction: 200
      ef_search: 64

governance:
  security:
    resource_limits:
      max_memory_mb: 2048  # Override tenant default
      max_concurrent_executions: 10
```

### Configuration Resolution Logic

```rust
pub struct ConfigResolver {
    profile_loader: ProfileLoader,
    tenant_config_repo: TenantConfigRepository,
    spec_repository: AgentSpecRepository,
}

impl ConfigResolver {
    pub async fn resolve_config(
        &self,
        agent_id: &str,
        version: &str,
        tenant_id: Uuid,
        cli_overrides: &CliConfig,
    ) -> Result<ResolvedConfig> {
        // Layer 1: Load profile
        let profile = self.profile_loader.load(&cli_overrides.profile)?;

        // Layer 2: Load tenant config
        let tenant_config = self.tenant_config_repo.get(tenant_id).await?;

        // Layer 3: Load agent spec
        let spec = self.spec_repository.get(agent_id, version).await?;

        // Layer 4: CLI overrides

        // Merge layers (higher priority overrides lower)
        let resolved = ResolvedConfig {
            database: cli_overrides.database
                .or(spec.storage.backend)
                .or(tenant_config.default_storage_backend)
                .unwrap_or(profile.database.backend),

            memory_backend: cli_overrides.memory_backend
                .or(spec.memory.backend)
                .or(tenant_config.default_memory_backend)
                .unwrap_or(profile.memory.default_backend),

            // ... resolve all config fields
        };

        Ok(resolved)
    }
}
```

---

## Technology Stack

### Core Rust Crates

| Component | Crate | Version | Purpose |
|-----------|-------|---------|---------|
| **Async Runtime** | `tokio` | 1.42+ | Global IO runtime, no-spawn execution |
| **Database** | `sqlx` | 0.8+ | SQLite + PostgreSQL with compile-time query checks |
| **Serialization** | `serde` + `serde_json` | 1.0+ | Agent spec serialization |
| **Schema Validation** | `schemars` | 0.8+ | JSON Schema generation for specs |
| **LLM Providers** | `rig` | 0.26+ | OpenAI, Anthropic, Gemini, Cohere |
| **Local LLM** | `candle-core` | 0.9+ | Embedded inference (Candle backend) |
| **Ollama Client** | `rig-ollama` | 0.26+ | Local LLM via Ollama |
| **Vector Search** | `vectorlite-rs` | 0.2+ | Pure Rust HNSW for SQLite |
| **PostgreSQL Vector** | `pgvector` | 0.3+ | PostgreSQL vector extension |
| **Lua Runtime** | `mlua` | 0.10+ | Lua scripting bridge |
| **JavaScript Runtime** | `boa` | 0.20+ | JavaScript scripting bridge |
| **Messaging** | `zeromq` | 0.4+ | Inter-kernel communication |
| **Tracing** | `tracing` + `opentelemetry` | 0.1+ | Distributed tracing |
| **Metrics** | `prometheus` | 0.13+ | Metrics collection |
| **Web Server** | `axum` | 0.7+ | HTTP API and WebSocket |
| **CLI** | `clap` | 4.5+ | Command-line interface |

### External Dependencies

- **PostgreSQL 16+**: Production database with RLS and VectorChord
- **SQLite 3.45+**: Development database with vectorlite extension
- **ZeroMQ 4.3+**: Messaging infrastructure
- **Ollama** (optional): Local LLM serving
- **Redis** (optional): Distributed rate limiting and caching

---

## Performance Targets

| Metric | Target | Measurement Method |
|--------|--------|-------------------|
| **Agent Spec Load** | <20ms P95 | Database query + deserialization |
| **Agent Instantiation** | <100ms P95 | From spec to executable agent |
| **Tenant Isolation Overhead** | <5% | RLS vs non-RLS query comparison |
| **Vector Search (1M vectors)** | <50ms P95 | HNSW with ef_search=64 |
| **Spec Validation** | <10ms P95 | Schema + semantic + resource checks |
| **Memory Overhead** | <100MB per agent | Including memory backends |
| **Concurrent Tenants** | 10,000+ | PostgreSQL with RLS |
| **Agents Per Tenant** | 1,000+ | In-memory registry |
| **Kernel Startup** | <2s | Cold start with PostgreSQL connection |
| **Graceful Shutdown** | <10s | 6-phase shutdown with state persistence |

---

## Implementation Roadmap

### Phase 0: Foundation (Weeks 1-2)
- 5-crate workspace structure
- Agent specification schema (Rust types + JSON Schema)
- Database schema (PostgreSQL + SQLite)
- Basic kernel skeleton

**Deliverables**:
- `aikit-core`: AgentSpecification struct with serde + schemars
- `aikit-storage`: AgentSpecRepository with SQLite backend
- `aikit-kernel`: Basic kernel with in-process transport
- Schema migration scripts (SQLite + PostgreSQL)

### Phase 1: Specification Management (Weeks 3-4)
- Agent spec CRUD operations
- Versioning and history
- Validation pipeline (schema → semantic → resource)
- Tenant context enforcement (SQLite application-level)

**Deliverables**:
- Spec repository with full CRUD
- Validation error messages with actionable feedback
- CLI commands: `aikit spec create/get/list/update/promote`
- 100% test coverage for spec operations

### Phase 2: Multi-Tenancy Foundation (Weeks 5-6)
- PostgreSQL RLS policies
- Session variable tenant context
- Tenant configuration table
- Cross-database compatibility (SQLite ↔ PostgreSQL)

**Deliverables**:
- RLS policies for all tenant-scoped tables
- TenantManager with set_tenant_context()
- Tenant isolation tests (verify no cross-tenant leakage)
- Migration guide (SQLite → PostgreSQL)

### Phase 3: Agent Runtime (Weeks 7-10)
- Agent loading from spec
- Memory backend initialization from spec
- Tool initialization from spec
- LLM client initialization from spec
- Agent execution with tenant context

**Deliverables**:
- AgentRuntime::from_spec() implementation
- Integration tests with real specs
- Performance benchmarks (load <20ms, instantiate <100ms)
- Example agent specs (10+ demonstrating all layers)

### Phase 4: Scripting Bridges (Weeks 11-12)
- Lua bridge with agent spec support
- JavaScript bridge with agent spec support
- Spec DSL (simplified YAML for common patterns)

**Deliverables**:
- Lua `AgentSpec` global for runtime spec access
- JS `AgentSpec` global with Promise-based API
- DSL compiler (YAML → full spec)
- Migration guide (rs-llmspell scripts → rs-aikit specs)

### Phase 5: Web Interface (Weeks 13-14)
- Spec editor UI (Monaco with JSON Schema)
- Tenant dashboard (agent inventory, metrics)
- Agent deployment UI (spec → running agent)
- Spec marketplace (share/discover specs)

**Deliverables**:
- React UI with embedded Vite build
- WebSocket API for real-time updates
- Spec validation in browser (same JSON Schema)
- Dark mode + responsive design

### Phase 6: Production Features (Weeks 15-16)
- Kernel clustering (ZeroMQ mesh)
- Graceful shutdown with state persistence
- Health checks and readiness probes
- Kubernetes manifests

**Deliverables**:
- Multi-kernel deployment working
- Systemd/launchd service files
- Kubernetes Helm chart
- Production deployment guide

### Phase 7: Observability (Weeks 17-18)
- OpenTelemetry tracing integration
- Prometheus metrics with tenant labels
- Structured logging with tenant context
- Grafana dashboards

**Deliverables**:
- Trace every agent execution with tenant_id
- Metrics: agent_executions_total{tenant_id, agent_id, version}
- Logs: JSON with tenant_id, agent_id fields
- Pre-built Grafana dashboard

---

## Migration from rs-llmspell

### Conceptual Mapping

| rs-llmspell | rs-aikit | Migration Strategy |
|-------------|----------|-------------------|
| **Lua scripts** | **Agent specs** | Convert script logic → declarative spec |
| **TOML profiles** | **Layered config** | Profiles remain, specs add agent layer |
| **llmspell-core traits** | **aikit-core traits** | Direct port (minimal changes) |
| **SQLite unified storage** | **SQLite + PostgreSQL** | Add PostgreSQL, maintain SQLite compat |
| **StateScope (tenant)** | **Native multi-tenancy** | Refactor to RLS-first design |
| **Templates** | **Spec presets** | Templates become reusable spec snippets |
| **Tools (40+)** | **Tools (same)** | Tools unchanged, spec declares usage |

### Migration Path

**Step 1: Parallel Development**
- rs-llmspell remains active for experimentation
- rs-aikit developed in separate repository
- Share common traits (aikit-core based on llmspell-core)

**Step 2: Spec Conversion Tools**
- CLI tool: `aikit migrate script.lua` → `agent-spec.yaml`
- Analyze Lua script:
  - Detect Agent.create() calls → intelligence block
  - Detect Tool.invoke() calls → integration block
  - Detect Memory.store() calls → memory block
- Generate spec with annotations for manual review

**Step 3: Backward Compatibility Bridge**
- rs-aikit supports loading Lua scripts as legacy mode
- Automatic spec generation from script execution
- Warning: "Consider migrating to declarative spec for production"

**Step 4: Data Migration**
- Export rs-llmspell SQLite data
- Import into rs-aikit with tenant assignment
- Maintain file format compatibility (artifacts, sessions)

**Step 5: Deprecation Timeline**
- rs-llmspell: Maintenance mode (bug fixes only)
- rs-aikit: Active development
- Timeline: 6-month parallel support, then full transition

---

## Conclusion

rs-aikit represents the evolution from experimental AI platform (rs-llmspell) to production-grade agent runtime. By treating agent specifications as first-class database entities with native multi-tenancy and kernel-centric architecture, rs-aikit enables organizations to deploy AI agents at scale with confidence.

**Key Differentiators**:
1. **Declarative Specifications**: Agents defined in YAML/JSON, stored in databases, versioned like code
2. **Native Multi-Tenancy**: Built into architecture from day one, not retrofitted
3. **Hot-Swappable Backends**: Switch between SQLite and PostgreSQL without code changes
4. **Specification-Driven Infrastructure**: Agent specs declare memory backends, storage, networking
5. **Production-Ready**: RLS, distributed tracing, graceful shutdown, horizontal scaling

**Next Steps**:
1. Review architecture with stakeholders
2. Validate agent specification schema against use cases
3. Prototype Phase 0 (foundation) in 2 weeks
4. Iterate based on feedback
5. Begin full implementation roadmap

---

## Research Sources

### Multi-Tenancy Patterns
- [Multi-tenant data isolation with PostgreSQL Row Level Security](https://aws.amazon.com/blogs/database/multi-tenant-data-isolation-with-postgresql-row-level-security/)
- [Mastering PostgreSQL Row-Level Security for Multi-Tenancy](https://ricofritzsche.me/mastering-postgresql-row-level-security-rls-for-rock-solid-multi-tenancy/)
- [Row-Level Security | SQLite Cloud Docs](https://docs.sqlitecloud.io/docs/rls)
- [Multi-Tenant Database Design: Complete Guide for 2025](https://sqlcheat.com/blog/multi-tenant-database-design-2025/)

### Agent Specification Standards
- [Open Agent Spec - GitHub](https://github.com/prime-vector/open-agent-spec)
- [Open Agent Specification Technical Report](https://arxiv.org/html/2510.04173v1)
- [Using YAML Files To Define Tasks For AI Agents](https://empathyfirstmedia.com/yaml-files-ai-agents/)
- [TOON vs JSON: Token-Optimized Data Formats](https://jduncan.io/blog/2025-11-11-toon-vs-json-agent-optimized-data/)

### Kernel Architecture
- [Backend.AI Kernel Architecture](https://docs.backend.ai/en/latest/dev/adding-kernels.html)
- [ZeroMQ Architecture](https://aosabook.org/en/v2/zeromq.html)
- [ØMQ Lightweight Messaging Kernel](http://wiki.zeromq.org/whitepapers:design-v05)

---

**Document Status**: Ready for Review
**Next Revision**: After Phase 0 prototype validation
**Maintainer**: Architecture Team
**Last Updated**: December 2025
