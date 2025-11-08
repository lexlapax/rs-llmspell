# PostgreSQL Schema Reference

**Phase 13b Storage Backend**: Complete schema documentation for all 15 tables + 15 migrations

## Table of Contents

- [Overview](#overview)
- [Schema Structure](#schema-structure)
- [Migration History](#migration-history)
- [Tables](#tables)
  - [Vector Embeddings](#vector-embeddings)
  - [Temporal Graph](#temporal-graph)
  - [Procedural Memory](#procedural-memory)
  - [Agent State](#agent-state)
  - [Workflow State](#workflow-state)
  - [Sessions](#sessions)
  - [Artifacts](#artifacts)
  - [Event Log](#event-log)
  - [Hook History](#hook-history)
  - [API Keys](#api-keys)
- [Index Strategy](#index-strategy)
- [RLS Policies](#rls-policies)
- [Entity-Relationship Diagram](#entity-relationship-diagram)

---

## Overview

rs-llmspell uses PostgreSQL 18 with VectorChord 0.5.3 for production storage. The schema implements:

- **15 migrations** (V1-V15): 2,434 lines of SQL DDL
- **15+ tables**: 10 storage backends across 12+ tables (vector embeddings split into 4 dimension tables)
- **Bi-temporal tracking**: Valid time + transaction time for entities and relationships
- **Content addressing**: Deduplication via blake3 hashing for artifacts
- **Partitioning**: Monthly range partitioning for event log
- **Row-Level Security**: Multi-tenant isolation on all tables via RLS
- **Index optimization**: HNSW (vectors), GiST (temporal ranges), GIN (JSONB)

**Schema**: `llmspell`
**Extensions**: `vchord 0.5.3`, `vector 0.8.1`, `uuid-ossp`, `pgcrypto`
**Roles**: `llmspell` (admin), `llmspell_app` (non-superuser for RLS)

---

## Schema Structure

```
llmspell (schema)
├── Extensions
│   ├── vchord 0.5.3 (Vector similarity search)
│   ├── vector 0.8.1 (pgvector dependency)
│   ├── uuid-ossp (UUID generation)
│   └── pgcrypto (Encryption functions)
│
├── Vector Storage (4 tables)
│   ├── vector_embeddings_384   (384-dim: All-MiniLM, small models)
│   ├── vector_embeddings_768   (768-dim: sentence-transformers, BGE)
│   ├── vector_embeddings_1536  (1536-dim: OpenAI text-embedding-3-small)
│   └── vector_embeddings_3072  (3072-dim: OpenAI text-embedding-3-large)
│
├── Temporal Graph (2 tables)
│   ├── entities (Bi-temporal nodes with versioning)
│   └── relationships (Bi-temporal edges with versioning)
│
├── State Storage (3 tables)
│   ├── procedural_patterns (Learned state transition patterns)
│   ├── agent_states (Persistent agent state with versioning)
│   └── workflow_states (Workflow execution lifecycle)
│
├── Session Management (3 tables)
│   ├── sessions (Session snapshots with expiration)
│   ├── artifact_content (Content-addressed storage with deduplication)
│   └── artifacts (Artifact metadata and references)
│
├── Event Sourcing (2 tables)
│   ├── event_log (Partitioned event log, monthly partitions)
│   └── hook_history (Hook execution history with compression)
│
├── Security (1 table)
│   └── api_keys (Encrypted API keys with pgcrypto)
│
└── Testing (1 table)
    └── test_table (RLS testing infrastructure)
```

---

## Migration History

### Summary

| Version | Name | Lines | Purpose |
|---------|------|-------|---------|
| **V1** | `initial_setup` | 38 | Extensions (vchord, pgcrypto, uuid-ossp), llmspell schema |
| **V2** | `test_table_rls` | 51 | RLS testing infrastructure with test_table |
| **V3** | `vector_embeddings` | 215 | Vector storage (4 dimension tables: 384, 768, 1536, 3072) |
| **V4** | `temporal_graph` | 225 | Bi-temporal entities + relationships |
| **V5** | `procedural_memory` | 106 | Procedural patterns table |
| **V6** | `agent_state` | 138 | Agent state with versioning + integrity checks |
| **V7** | `kv_store` | 105 | Key-value store (deprecated, kept for compatibility) |
| **V8** | `workflow_states` | 159 | Workflow execution lifecycle tracking |
| **V9** | `sessions` | 157 | Session snapshots with expiration |
| **V10** | `artifacts` | 322 | Content-addressed artifacts with deduplication |
| **V11** | `event_log` | 256 | Partitioned event log (monthly range partitions) |
| **V12** | `application_role_rls_enforcement` | 151 | llmspell_app role + RLS enforcement |
| **V13** | `hook_history` | 200 | Hook execution history with compression |
| **V14** | `api_keys` | 219 | Encrypted API keys with pgcrypto |
| **V15** | `bitemporal_composite_keys` | 92 | Fix bi-temporal primary keys (composite keys) |
| **TOTAL** | | **2,434** | |

### Detailed Migration Timeline

**Phase 13b.2-13b.3: Foundation**
- **V1**: Extension installation and schema creation
- **V2**: RLS testing framework and validation

**Phase 13b.4: Vector Search**
- **V3**: 4 vector embedding tables with HNSW indexes (dimensions: 384, 768, 1536, 3072)

**Phase 13b.5: Temporal Graph**
- **V4**: Bi-temporal entities + relationships with GiST indexes
- **V15**: Fixed composite primary keys for proper bi-temporal versioning

**Phase 13b.6-13b.8: State Storage**
- **V5**: Procedural memory patterns
- **V6**: Agent state with SHA-256 checksums
- **V7**: KV store (deprecated but kept for backward compatibility)
- **V8**: Workflow execution states

**Phase 13b.9-13b.10: Session Management**
- **V9**: Session lifecycle tracking
- **V10**: Artifact storage with content addressing

**Phase 13b.11: Event Sourcing**
- **V11**: Partitioned event log with automatic partition management
- **V12**: Application role (llmspell_app) with strict RLS enforcement

**Phase 13b.12-13b.13: Security & Observability**
- **V13**: Hook history with compression
- **V14**: Encrypted API keys using pgcrypto

---

## Tables

### Vector Embeddings

**Tables:** `vector_embeddings_{384,768,1536,3072}`
**Migration:** V3
**Purpose:** Multi-dimensional vector storage for episodic memory with VectorChord HNSW indexes

#### Schema (384-dimensional example)

```sql
CREATE TABLE llmspell.vector_embeddings_384 (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    tenant_id VARCHAR(255) NOT NULL,
    scope VARCHAR(255) NOT NULL,
    embedding VECTOR(384) NOT NULL,
    metadata JSONB NOT NULL DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
```

**Columns:**
- `id`: Unique embedding identifier (UUID v4)
- `tenant_id`: Tenant isolation key (RLS filter)
- `scope`: Logical grouping (e.g., "session:abc123", "agent:xyz")
- `embedding`: Vector data (384/768/1536/3072 dimensions)
- `metadata`: Flexible JSONB storage for embedding context
- `created_at`, `updated_at`: Lifecycle timestamps

#### Dimension Tables

| Table | Dimensions | Use Case | HNSW Parameters |
|-------|------------|----------|-----------------|
| `vector_embeddings_384` | 384 | All-MiniLM, small models | `m=16, ef_construction=64` |
| `vector_embeddings_768` | 768 | sentence-transformers, BGE | `m=16, ef_construction=128` |
| `vector_embeddings_1536` | 1536 | OpenAI text-embedding-3-small | `m=24, ef_construction=256` |
| `vector_embeddings_3072` | 3072 | OpenAI text-embedding-3-large | **No HNSW** (pgvector 2000-dim limit) |

**Note on 3072 dimensions:** VectorChord/pgvector have a 2000-dimension limit for HNSW/IVFFlat indexes. For 3072-dimensional vectors:
- Linear scan for small datasets (<10K vectors)
- External vector DB (Qdrant, Milvus) for large-scale search
- Dimension reduction (PCA, UMAP) if needed

#### Indexes

```sql
-- B-tree indexes for filtering
CREATE INDEX idx_vector_384_tenant ON vector_embeddings_384(tenant_id);
CREATE INDEX idx_vector_384_scope ON vector_embeddings_384(scope);
CREATE INDEX idx_vector_384_created ON vector_embeddings_384(created_at);

-- HNSW index for similarity search (cosine distance)
CREATE INDEX idx_vector_384_hnsw ON vector_embeddings_384
    USING hnsw (embedding vector_cosine_ops)
    WITH (m = 16, ef_construction = 64);
```

**Index Strategy:**
- **B-tree indexes**: Tenant isolation, scope filtering, temporal queries
- **HNSW index**: Vector similarity search (O(log n) vs O(n) linear scan)
- **Performance**: 8.47x speedup at 10K vectors (validated Phase 13)

**HNSW Parameters:**
- `m`: Graph connectivity (higher = better recall, more memory)
- `ef_construction`: Build-time search depth (higher = slower build, better index quality)

#### RLS Policies

```sql
-- All 4 CRUD operations isolated by tenant_id
CREATE POLICY tenant_isolation_select ON vector_embeddings_384
    FOR SELECT
    USING (tenant_id = current_setting('app.current_tenant_id', true));

CREATE POLICY tenant_isolation_insert ON vector_embeddings_384
    FOR INSERT
    WITH CHECK (tenant_id = current_setting('app.current_tenant_id', true));

CREATE POLICY tenant_isolation_update ON vector_embeddings_384
    FOR UPDATE
    USING (tenant_id = current_setting('app.current_tenant_id', true))
    WITH CHECK (tenant_id = current_setting('app.current_tenant_id', true));

CREATE POLICY tenant_isolation_delete ON vector_embeddings_384
    FOR DELETE
    USING (tenant_id = current_setting('app.current_tenant_id', true));
```

**RLS Performance:** <5% overhead (4.9% measured in Phase 13b.3)

---

### Temporal Graph

**Tables:** `entities`, `relationships`
**Migrations:** V4 (initial), V15 (composite keys)
**Purpose:** Bi-temporal knowledge graph for semantic memory

#### Entities Table

```sql
CREATE TABLE llmspell.entities (
    entity_id UUID NOT NULL,
    tenant_id VARCHAR(255) NOT NULL,
    entity_type VARCHAR(255) NOT NULL,
    name VARCHAR(500) NOT NULL,
    properties JSONB NOT NULL DEFAULT '{}',

    -- Bi-temporal timestamps
    valid_time_start TIMESTAMPTZ NOT NULL,
    valid_time_end TIMESTAMPTZ NOT NULL DEFAULT 'infinity',
    transaction_time_start TIMESTAMPTZ NOT NULL DEFAULT now(),
    transaction_time_end TIMESTAMPTZ NOT NULL DEFAULT 'infinity',

    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),

    PRIMARY KEY (entity_id, transaction_time_start),

    CONSTRAINT valid_time_range CHECK (valid_time_start < valid_time_end),
    CONSTRAINT tx_time_range CHECK (transaction_time_start < transaction_time_end)
);
```

**Bi-Temporal Semantics:**
- **Valid Time**: When data was true in the real world
  - `valid_time_start`: Entity became valid
  - `valid_time_end`: Entity became invalid (or 'infinity' if still valid)
- **Transaction Time**: When data was recorded in the database
  - `transaction_time_start`: Record created
  - `transaction_time_end`: Record superseded (or 'infinity' if current version)

**Primary Key:** `(entity_id, transaction_time_start)` allows multiple temporal versions

**Example Use Cases:**
- **Point-in-time queries**: "What was known at time T?"
- **Historical reconstruction**: "What did we know about X on date Y?"
- **Audit trails**: Immutable transaction history
- **Temporal joins**: Relationships valid at specific times

#### Relationships Table

```sql
CREATE TABLE llmspell.relationships (
    relationship_id UUID NOT NULL,
    tenant_id VARCHAR(255) NOT NULL,
    from_entity UUID NOT NULL,
    to_entity UUID NOT NULL,
    relationship_type VARCHAR(255) NOT NULL,
    properties JSONB NOT NULL DEFAULT '{}',

    -- Bi-temporal timestamps (same as entities)
    valid_time_start TIMESTAMPTZ NOT NULL,
    valid_time_end TIMESTAMPTZ NOT NULL DEFAULT 'infinity',
    transaction_time_start TIMESTAMPTZ NOT NULL DEFAULT now(),
    transaction_time_end TIMESTAMPTZ NOT NULL DEFAULT 'infinity',

    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),

    PRIMARY KEY (relationship_id, transaction_time_start),

    CONSTRAINT valid_time_range CHECK (valid_time_start < valid_time_end),
    CONSTRAINT tx_time_range CHECK (transaction_time_start < transaction_time_end)
);
```

**Note on Foreign Keys (V15 Change):**
- V4 initially included `FOREIGN KEY (from_entity) REFERENCES entities(entity_id)`
- V15 removed foreign keys because `entity_id` is no longer unique (multiple temporal versions)
- **Referential integrity is application-enforced** (standard practice for bi-temporal databases)
- Application code ensures relationships only reference valid `entity_id` values

#### Indexes

```sql
-- Entities indexes
CREATE INDEX idx_entities_tenant ON entities(tenant_id);
CREATE INDEX idx_entities_type ON entities(entity_type);
CREATE INDEX idx_entities_name ON entities(name);
CREATE INDEX idx_entities_id_lookup ON entities(entity_id); -- V15: All versions

-- GiST indexes for temporal range queries (critical for performance)
CREATE INDEX idx_entities_valid_time ON entities
    USING GIST (tstzrange(valid_time_start, valid_time_end));

CREATE INDEX idx_entities_tx_time ON entities
    USING GIST (tstzrange(transaction_time_start, transaction_time_end));

-- GIN index for JSONB property queries
CREATE INDEX idx_entities_properties ON entities USING GIN(properties);

-- Relationships indexes
CREATE INDEX idx_relationships_tenant ON relationships(tenant_id);
CREATE INDEX idx_relationships_from ON relationships(from_entity, valid_time_start);
CREATE INDEX idx_relationships_to ON relationships(to_entity, valid_time_start);
CREATE INDEX idx_relationships_type ON relationships(relationship_type);
CREATE INDEX idx_relationships_id_lookup ON relationships(relationship_id); -- V15

-- GiST temporal indexes for relationships
CREATE INDEX idx_relationships_valid_time ON relationships
    USING GIST (tstzrange(valid_time_start, valid_time_end));

CREATE INDEX idx_relationships_tx_time ON relationships
    USING GIST (tstzrange(transaction_time_start, transaction_time_end));

-- GIN index for relationship properties
CREATE INDEX idx_relationships_properties ON relationships USING GIN(properties);
```

**Query Patterns:**
```sql
-- Current version of all entities (transaction_time_end = 'infinity')
SELECT * FROM entities WHERE transaction_time_end = 'infinity';

-- Currently valid entities (valid_time_end = 'infinity')
SELECT * FROM entities WHERE valid_time_end = 'infinity';

-- Point-in-time query (what was known at 2025-01-01?)
SELECT * FROM entities
WHERE valid_time_start <= '2025-01-01'
  AND valid_time_end > '2025-01-01'
  AND transaction_time_start <= '2025-01-01'
  AND transaction_time_end > '2025-01-01';

-- Historical reconstruction (all versions of entity X)
SELECT * FROM entities
WHERE entity_id = '...'
ORDER BY transaction_time_start;
```

---

### Procedural Memory

**Table:** `procedural_patterns`
**Migration:** V5
**Purpose:** Learned state transition patterns for procedural memory

#### Schema

```sql
CREATE TABLE llmspell.procedural_patterns (
    pattern_id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    tenant_id VARCHAR(255) NOT NULL,

    -- Pattern identity
    scope VARCHAR(500) NOT NULL,
    key VARCHAR(500) NOT NULL,
    value TEXT NOT NULL,

    -- Pattern tracking
    frequency INTEGER NOT NULL DEFAULT 1,
    first_seen TIMESTAMPTZ NOT NULL DEFAULT now(),
    last_seen TIMESTAMPTZ NOT NULL DEFAULT now(),

    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT unique_procedural_pattern UNIQUE (tenant_id, scope, key, value),
    CONSTRAINT positive_frequency CHECK (frequency > 0)
);
```

**Purpose:** Track recurring state transitions. When a transition `scope:key → value` occurs ≥3 times, it becomes a learned pattern.

**Example Pattern:**
```
scope: "llm_provider_config"
key: "default_model"
value: "ollama/llama3.2:3b"
frequency: 47
```

#### Indexes

```sql
CREATE INDEX idx_procedural_patterns_tenant ON procedural_patterns(tenant_id);
CREATE INDEX idx_procedural_patterns_scope_key ON procedural_patterns(scope, key);
CREATE INDEX idx_procedural_patterns_frequency ON procedural_patterns(frequency DESC);
CREATE INDEX idx_procedural_patterns_last_seen ON procedural_patterns(last_seen DESC);

-- Partial index for learned patterns (frequency >= 3)
CREATE INDEX idx_procedural_patterns_lookup ON procedural_patterns(tenant_id, scope, key, value)
    WHERE frequency >= 3;
```

#### Triggers

```sql
-- Auto-update updated_at on modifications
CREATE TRIGGER trigger_procedural_patterns_updated_at
    BEFORE UPDATE ON procedural_patterns
    FOR EACH ROW
    EXECUTE FUNCTION update_procedural_patterns_updated_at();
```

---

### Agent State

**Table:** `agent_states`
**Migration:** V6
**Purpose:** Persistent agent state with versioning and integrity checks

#### Schema

```sql
CREATE TABLE llmspell.agent_states (
    state_id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    tenant_id VARCHAR(255) NOT NULL,

    -- Agent identification
    agent_id VARCHAR(255) NOT NULL,
    agent_type VARCHAR(100) NOT NULL,

    -- State data
    state_data JSONB NOT NULL,

    -- Versioning and integrity
    schema_version INTEGER NOT NULL DEFAULT 1,
    data_version INTEGER NOT NULL DEFAULT 1,
    checksum VARCHAR(64) NOT NULL, -- SHA-256 of state_data

    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT unique_agent_state UNIQUE (tenant_id, agent_id),
    CONSTRAINT positive_schema_version CHECK (schema_version > 0),
    CONSTRAINT positive_data_version CHECK (data_version > 0)
);
```

**Columns:**
- `state_data`: Full `PersistentAgentState` as JSONB (conversation history, tools, metadata)
- `checksum`: SHA-256 hash for integrity verification
- `schema_version`: For schema evolution (future-proofing)
- `data_version`: Auto-incremented on state changes

#### Indexes

```sql
CREATE INDEX idx_agent_states_tenant ON agent_states(tenant_id);
CREATE INDEX idx_agent_states_type ON agent_states(agent_type);
CREATE INDEX idx_agent_states_updated ON agent_states(updated_at DESC);

-- GIN index for JSONB queries
CREATE INDEX idx_agent_states_data_gin ON agent_states USING GIN(state_data);

-- Specific JSONB path indexes
CREATE INDEX idx_agent_states_execution_state ON agent_states((state_data->'state'->>'execution_state'));
CREATE INDEX idx_agent_states_metadata_name ON agent_states((state_data->'metadata'->>'name'));
```

#### Triggers

```sql
-- Auto-update updated_at
CREATE TRIGGER trigger_agent_states_updated_at
    BEFORE UPDATE ON agent_states
    FOR EACH ROW
    EXECUTE FUNCTION update_agent_states_updated_at();

-- Auto-increment data_version on state changes
CREATE TRIGGER trigger_agent_state_version
    BEFORE UPDATE ON agent_states
    FOR EACH ROW
    EXECUTE FUNCTION increment_agent_state_version();
```

---

### Workflow State

**Table:** `workflow_states`
**Migration:** V8
**Purpose:** Workflow execution lifecycle tracking with checkpoints

#### Schema

```sql
CREATE TABLE llmspell.workflow_states (
    tenant_id VARCHAR(255) NOT NULL,
    workflow_id VARCHAR(255) NOT NULL,
    workflow_name VARCHAR(500),

    state_data JSONB NOT NULL,

    -- Execution tracking
    current_step INTEGER NOT NULL DEFAULT 0,
    status VARCHAR(20) NOT NULL DEFAULT 'pending',

    -- Lifecycle timestamps
    started_at TIMESTAMPTZ,
    completed_at TIMESTAMPTZ,
    last_updated TIMESTAMPTZ NOT NULL DEFAULT now(),
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),

    PRIMARY KEY (tenant_id, workflow_id),

    CONSTRAINT valid_workflow_status CHECK (
        status IN ('pending', 'running', 'completed', 'failed', 'cancelled')
    ),
    CONSTRAINT positive_step_index CHECK (current_step >= 0)
);
```

**Status Values:**
- `pending`: Workflow created, not started
- `running`: Execution in progress
- `completed`: Successfully finished
- `failed`: Execution error
- `cancelled`: User-initiated cancellation

#### Indexes

```sql
CREATE INDEX idx_workflow_states_tenant ON workflow_states(tenant_id);
CREATE INDEX idx_workflow_states_tenant_workflow ON workflow_states(tenant_id, workflow_id);
CREATE INDEX idx_workflow_states_status ON workflow_states(status);
CREATE INDEX idx_workflow_states_started ON workflow_states(started_at DESC);
CREATE INDEX idx_workflow_states_completed ON workflow_states(completed_at DESC)
    WHERE completed_at IS NOT NULL;

-- GIN index for JSONB state queries
CREATE INDEX idx_workflow_states_data_gin ON workflow_states USING GIN(state_data);
CREATE INDEX idx_workflow_states_execution_stats ON workflow_states((state_data->'execution_stats'));

-- Composite index for tenant + status queries
CREATE INDEX idx_workflow_states_tenant_status ON workflow_states(tenant_id, status);
```

---

### Sessions

**Table:** `sessions`
**Migration:** V9
**Purpose:** Session lifecycle tracking with expiration management

#### Schema

```sql
CREATE TABLE llmspell.sessions (
    tenant_id VARCHAR(255) NOT NULL,
    session_id UUID NOT NULL,

    session_data JSONB NOT NULL,

    -- Extracted fields for queries
    status VARCHAR(20) NOT NULL DEFAULT 'active',
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    last_accessed_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    expires_at TIMESTAMPTZ,
    artifact_count INTEGER NOT NULL DEFAULT 0,

    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),

    PRIMARY KEY (tenant_id, session_id),

    CONSTRAINT valid_session_status CHECK (
        status IN ('active', 'archived', 'expired')
    ),
    CONSTRAINT non_negative_artifact_count CHECK (artifact_count >= 0)
);
```

**Status Values:**
- `active`: Session in use
- `archived`: Manually archived (preserved)
- `expired`: TTL exceeded (cleanup eligible)

#### Indexes

```sql
CREATE INDEX idx_sessions_tenant ON sessions(tenant_id);
CREATE INDEX idx_sessions_tenant_session ON sessions(tenant_id, session_id);
CREATE INDEX idx_sessions_status ON sessions(status);

-- Partial index for expiration cleanup
CREATE INDEX idx_sessions_expires ON sessions(expires_at)
    WHERE expires_at IS NOT NULL;

CREATE INDEX idx_sessions_created ON sessions(created_at DESC);
CREATE INDEX idx_sessions_accessed ON sessions(last_accessed_at DESC);

-- GIN index for JSONB queries
CREATE INDEX idx_sessions_data_gin ON sessions USING GIN(session_data);

-- Composite indexes
CREATE INDEX idx_sessions_tenant_status ON sessions(tenant_id, status);
CREATE INDEX idx_sessions_tenant_expires ON sessions(tenant_id, expires_at)
    WHERE expires_at IS NOT NULL;
```

#### Triggers

```sql
-- Auto-update updated_at
CREATE TRIGGER trigger_sessions_updated_at
    BEFORE UPDATE ON sessions
    FOR EACH ROW
    EXECUTE FUNCTION update_sessions_updated_at();

-- Auto-update last_accessed_at (throttled to 1 minute)
CREATE TRIGGER trigger_sessions_accessed_at
    BEFORE UPDATE ON sessions
    FOR EACH ROW
    EXECUTE FUNCTION update_sessions_accessed_at();
```

---

### Artifacts

**Tables:** `artifact_content`, `artifacts`
**Migration:** V10
**Purpose:** Content-addressed artifact storage with deduplication

#### Artifact Content Table (Content Addressing)

```sql
CREATE TABLE llmspell.artifact_content (
    tenant_id VARCHAR(255) NOT NULL,
    content_hash VARCHAR(64) NOT NULL, -- blake3 hash

    -- Storage strategy
    storage_type VARCHAR(20) NOT NULL,
    data BYTEA, -- <1MB
    large_object_oid OID, -- >=1MB

    -- Metadata
    size_bytes BIGINT NOT NULL,
    is_compressed BOOLEAN NOT NULL DEFAULT false,
    original_size_bytes BIGINT,

    -- Deduplication tracking
    reference_count INTEGER NOT NULL DEFAULT 1,

    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    last_accessed_at TIMESTAMPTZ NOT NULL DEFAULT now(),

    PRIMARY KEY (tenant_id, content_hash),

    CONSTRAINT valid_storage_type CHECK (storage_type IN ('bytea', 'large_object')),
    CONSTRAINT positive_reference_count CHECK (reference_count > 0),
    CONSTRAINT bytea_storage_valid CHECK (
        (storage_type = 'bytea' AND data IS NOT NULL AND large_object_oid IS NULL)
        OR (storage_type = 'large_object' AND large_object_oid IS NOT NULL AND data IS NULL)
    ),
    CONSTRAINT max_artifact_size CHECK (size_bytes <= 104857600) -- 100MB
);
```

**Deduplication Strategy:**
- **Content hash**: blake3 (faster than SHA-256, cryptographically secure)
- **Reference counting**: Track how many artifacts point to this content
- **Garbage collection**: Delete content when `reference_count = 0`

**Storage Strategies:**
- **BYTEA** (<1MB): Stored inline in table
- **Large Objects** (>=1MB): PostgreSQL Large Objects (TOAST)

#### Artifacts Table (Metadata)

```sql
CREATE TABLE llmspell.artifacts (
    tenant_id VARCHAR(255) NOT NULL,
    artifact_id VARCHAR(512) NOT NULL, -- Format: "{session_id}:{sequence}:{content_hash}"
    session_id UUID NOT NULL,
    sequence BIGINT NOT NULL,
    content_hash VARCHAR(64) NOT NULL,

    metadata JSONB NOT NULL,

    -- Extracted fields
    name VARCHAR(255) NOT NULL,
    artifact_type VARCHAR(50) NOT NULL,
    mime_type VARCHAR(255) NOT NULL,
    size_bytes BIGINT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    created_by VARCHAR(255),

    -- Versioning
    version INTEGER NOT NULL DEFAULT 1,
    parent_artifact_id VARCHAR(512),

    tags TEXT[],

    stored_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),

    PRIMARY KEY (tenant_id, artifact_id),

    FOREIGN KEY (tenant_id, content_hash)
        REFERENCES artifact_content(tenant_id, content_hash)
        ON DELETE RESTRICT, -- Cannot delete content while artifacts reference it

    FOREIGN KEY (tenant_id, session_id)
        REFERENCES sessions(tenant_id, session_id)
        ON DELETE CASCADE, -- Delete artifacts when session deleted

    CONSTRAINT unique_session_sequence UNIQUE (tenant_id, session_id, sequence),
    CONSTRAINT positive_version CHECK (version > 0),
    CONSTRAINT non_negative_sequence CHECK (sequence >= 0)
);
```

#### Indexes

```sql
-- Content table
CREATE INDEX idx_artifact_content_tenant ON artifact_content(tenant_id);
CREATE INDEX idx_artifact_content_refcount ON artifact_content(reference_count)
    WHERE reference_count = 0; -- For garbage collection
CREATE INDEX idx_artifact_content_large_objects ON artifact_content(large_object_oid)
    WHERE large_object_oid IS NOT NULL;
CREATE INDEX idx_artifact_content_accessed ON artifact_content(last_accessed_at DESC);

-- Artifacts table
CREATE INDEX idx_artifacts_session ON artifacts(tenant_id, session_id, created_at DESC);
CREATE INDEX idx_artifacts_type ON artifacts(artifact_type, created_at DESC);
CREATE INDEX idx_artifacts_content ON artifacts(tenant_id, content_hash);
CREATE INDEX idx_artifacts_name ON artifacts(name);
CREATE INDEX idx_artifacts_created ON artifacts(created_at DESC);
CREATE INDEX idx_artifacts_size ON artifacts(size_bytes DESC);
CREATE INDEX idx_artifacts_tags ON artifacts USING GIN(tags);
CREATE INDEX idx_artifacts_metadata ON artifacts USING GIN(metadata);
CREATE INDEX idx_artifacts_tenant_type ON artifacts(tenant_id, artifact_type, created_at DESC);
```

#### Triggers

```sql
-- Auto-update updated_at on artifacts
CREATE TRIGGER trigger_artifacts_updated_at
    BEFORE UPDATE ON artifacts
    FOR EACH ROW
    EXECUTE FUNCTION update_artifacts_updated_at();

-- Auto-increment reference count when artifact created
CREATE TRIGGER trigger_increment_refcount
    AFTER INSERT ON artifacts
    FOR EACH ROW
    EXECUTE FUNCTION increment_content_refcount();

-- Auto-decrement reference count when artifact deleted
CREATE TRIGGER trigger_decrement_refcount
    AFTER DELETE ON artifacts
    FOR EACH ROW
    EXECUTE FUNCTION decrement_content_refcount();

-- Auto-update last_accessed_at on content (throttled to 1 minute)
CREATE TRIGGER trigger_content_accessed_at
    BEFORE UPDATE ON artifact_content
    FOR EACH ROW
    EXECUTE FUNCTION update_content_accessed_at();
```

---

### Event Log

**Table:** `event_log` (partitioned)
**Migration:** V11
**Purpose:** Time-series event storage with monthly range partitioning

#### Schema

```sql
CREATE TABLE llmspell.event_log (
    tenant_id VARCHAR(255) NOT NULL,
    event_id UUID NOT NULL,

    -- Extracted columns for hot queries
    event_type VARCHAR(255) NOT NULL,
    correlation_id UUID NOT NULL,
    timestamp TIMESTAMPTZ NOT NULL,
    sequence BIGINT NOT NULL,
    language VARCHAR(50) NOT NULL,

    -- Full event as JSONB
    payload JSONB NOT NULL,

    PRIMARY KEY (tenant_id, timestamp, event_id)
) PARTITION BY RANGE (timestamp);
```

**Partitioning Strategy:** Monthly RANGE partitions on `timestamp`

**Partition Naming:** `event_log_YYYY_MM` (e.g., `event_log_2025_01`)

#### Partition Management Functions

```sql
-- Create partition for specific month
SELECT llmspell.create_event_log_partition('2025-01-01'::timestamptz, '2025-02-01'::timestamptz);

-- Ensure future partitions exist (current + next 3 months)
SELECT llmspell.ensure_future_event_log_partitions();
```

**Maintenance:** Run `ensure_future_event_log_partitions()` periodically (daily cron) to maintain partition coverage.

#### Indexes

```sql
CREATE INDEX idx_event_log_correlation ON event_log(correlation_id, timestamp DESC);
CREATE INDEX idx_event_log_type ON event_log(event_type, timestamp DESC);
CREATE INDEX idx_event_log_sequence ON event_log(sequence DESC);
CREATE INDEX idx_event_log_tenant_time ON event_log(tenant_id, timestamp DESC);
CREATE INDEX idx_event_log_payload ON event_log USING GIN(payload);
```

**Index Inheritance:** All indexes on parent table are automatically created on each partition.

---

### Hook History

**Table:** `hook_history`
**Migration:** V13
**Purpose:** Hook execution history with compression for replay capabilities

#### Schema

```sql
CREATE TABLE llmspell.hook_history (
    execution_id UUID PRIMARY KEY,
    tenant_id VARCHAR(255) NOT NULL,

    -- Hook identification
    hook_id VARCHAR(255) NOT NULL,
    hook_type VARCHAR(255) NOT NULL,
    correlation_id UUID NOT NULL,

    -- Execution data (compressed)
    hook_context BYTEA NOT NULL, -- Compressed HookContext
    result_data JSONB NOT NULL, -- HookResult

    -- Execution metrics
    timestamp TIMESTAMPTZ NOT NULL,
    duration_ms INTEGER NOT NULL,

    -- Component metadata
    triggering_component VARCHAR(255) NOT NULL,
    component_id VARCHAR(255) NOT NULL,
    modified_operation BOOLEAN NOT NULL DEFAULT false,

    -- Categorization
    tags TEXT[] DEFAULT ARRAY[]::TEXT[],
    retention_priority INTEGER NOT NULL DEFAULT 0,

    -- Storage metadata
    context_size INTEGER NOT NULL,
    contains_sensitive_data BOOLEAN NOT NULL DEFAULT false,
    metadata JSONB NOT NULL DEFAULT '{}'::JSONB,

    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
```

**Compression:** `hook_context` stored as compressed BYTEA to reduce storage (context can be large)

**Retention:** `retention_priority` determines cleanup order (lower priority deleted first)

#### Indexes

```sql
CREATE INDEX idx_hook_history_hook_time ON hook_history(hook_id, timestamp DESC);
CREATE INDEX idx_hook_history_correlation ON hook_history(correlation_id);
CREATE INDEX idx_hook_history_type ON hook_history(hook_type);
CREATE INDEX idx_hook_history_tenant_time ON hook_history(tenant_id, timestamp DESC);
CREATE INDEX idx_hook_history_retention ON hook_history(retention_priority, timestamp);
CREATE INDEX idx_hook_history_metadata ON hook_history USING GIN(metadata);
CREATE INDEX idx_hook_history_tags ON hook_history USING GIN(tags);
```

#### Cleanup Function

```sql
-- Delete old executions based on retention policy
SELECT llmspell.cleanup_old_hook_executions(
    before_date := now() - INTERVAL '90 days',
    min_retention_priority := 5
);
```

---

### API Keys

**Table:** `api_keys`
**Migration:** V14
**Purpose:** Encrypted API key storage using pgcrypto

#### Schema

```sql
CREATE TABLE llmspell.api_keys (
    key_id VARCHAR(255) PRIMARY KEY,
    tenant_id VARCHAR(255) NOT NULL,

    service VARCHAR(255) NOT NULL, -- e.g., "openai", "anthropic"

    encrypted_key BYTEA NOT NULL, -- pgp_sym_encrypt result

    key_metadata JSONB NOT NULL DEFAULT '{}'::JSONB,

    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    last_used_at TIMESTAMPTZ,
    expires_at TIMESTAMPTZ,

    is_active BOOLEAN NOT NULL DEFAULT true,
    usage_count BIGINT NOT NULL DEFAULT 0,

    -- Audit fields
    rotated_from VARCHAR(255),
    deactivated_at TIMESTAMPTZ,

    CONSTRAINT unique_active_service_key UNIQUE (tenant_id, service, is_active)
);
```

**Encryption:** Uses PostgreSQL `pgcrypto` extension:
```sql
-- Encrypt key
INSERT INTO api_keys (key_id, tenant_id, service, encrypted_key)
VALUES (
    'key-123',
    'tenant-1',
    'openai',
    pgp_sym_encrypt('sk-...', current_setting('app.encryption_key'))
);

-- Decrypt key
SELECT pgp_sym_decrypt(encrypted_key, current_setting('app.encryption_key')) AS decrypted_key
FROM api_keys
WHERE key_id = 'key-123';
```

**Security:**
- Encryption passphrase stored in `app.encryption_key` session variable (never logged)
- Only one active key per tenant/service combination
- Key rotation via `rotated_from` field

#### Indexes

```sql
CREATE INDEX idx_api_keys_tenant_service ON api_keys(tenant_id, service);
CREATE INDEX idx_api_keys_expiration ON api_keys(expires_at)
    WHERE expires_at IS NOT NULL;
CREATE INDEX idx_api_keys_active ON api_keys(is_active)
    WHERE is_active = true;
CREATE INDEX idx_api_keys_metadata ON api_keys USING GIN(key_metadata);
```

#### Cleanup Function

```sql
-- Delete expired keys for current tenant
SELECT * FROM llmspell.cleanup_expired_api_keys();
```

---

## Index Strategy

### Index Types Used

| Index Type | Purpose | Performance Characteristic | Used On |
|------------|---------|----------------------------|---------|
| **B-tree** | Equality and range queries | O(log n) lookup | Primary keys, foreign keys, tenant_id, timestamps |
| **HNSW** | Vector similarity search | O(log n) approximate nearest neighbor | Vector embeddings (384, 768, 1536) |
| **GiST** | Temporal range queries | O(log n) range overlap | Bi-temporal timestamps (valid_time, transaction_time) |
| **GIN** | JSONB/array containment | Fast for `@>`, `?`, `?&` operators | JSONB columns (metadata, state_data), TEXT[] (tags) |
| **Partial** | Filtered subsets | Smaller index, faster queries | `expires_at IS NOT NULL`, `reference_count = 0`, `is_active = true` |

### Index Sizing

**Total Index Count:** 150+ indexes across 15 tables

**Example Index Sizes (after 10K rows per table):**
- **B-tree** (tenant_id): ~100 KB
- **HNSW** (384-dim vectors): ~25 MB (m=16, ef_construction=64)
- **GiST** (temporal range): ~500 KB
- **GIN** (JSONB): ~2 MB (depends on data cardinality)

### Index Maintenance

```sql
-- Rebuild indexes after bulk inserts
REINDEX TABLE llmspell.vector_embeddings_384;

-- Update statistics for query planner
ANALYZE llmspell.sessions;

-- Vacuum table and analyze (reclaim space + update stats)
VACUUM ANALYZE llmspell.event_log;
```

---

## RLS Policies

### RLS Architecture

**All 12+ tables use identical RLS pattern:**

```sql
ALTER TABLE <table> ENABLE ROW LEVEL SECURITY;
ALTER TABLE <table> FORCE ROW LEVEL SECURITY; -- Enforces RLS even for table owner

-- 4 policies per table: SELECT, INSERT, UPDATE, DELETE
CREATE POLICY tenant_isolation_select ON <table>
    FOR SELECT
    USING (tenant_id = current_setting('app.current_tenant_id', true));

CREATE POLICY tenant_isolation_insert ON <table>
    FOR INSERT
    WITH CHECK (tenant_id = current_setting('app.current_tenant_id', true));

CREATE POLICY tenant_isolation_update ON <table>
    FOR UPDATE
    USING (tenant_id = current_setting('app.current_tenant_id', true))
    WITH CHECK (tenant_id = current_setting('app.current_tenant_id', true));

CREATE POLICY tenant_isolation_delete ON <table>
    FOR DELETE
    USING (tenant_id = current_setting('app.current_tenant_id', true));
```

### Setting Tenant Context

**Rust:**
```rust
use llmspell_storage::postgres::set_tenant_context;

let conn = pool.get().await?;
set_tenant_context(&conn, "tenant-123").await?;
```

**SQL:**
```sql
SET LOCAL app.current_tenant_id = 'tenant-123';
```

### RLS Performance

**Measured overhead:** 4.9% (Phase 13b.3 validation)

**Benchmark:**
```
Without RLS:  1000 queries in 245ms (4,082 qps)
With RLS:     1000 queries in 257ms (3,892 qps)
Overhead:     4.9%
```

**Optimization:** Composite indexes with `tenant_id` as first column:
```sql
CREATE INDEX idx_sessions_tenant_status ON sessions(tenant_id, status);
```

PostgreSQL can use this index for both tenant isolation (RLS) and status filtering (application query).

### Cross-Tenant Access Prevention

**RLS prevents cross-tenant access even with SQL injection:**

```sql
-- Malicious query attempt
SELECT * FROM sessions WHERE tenant_id = 'other_tenant';

-- RLS enforces AND clause:
SELECT * FROM sessions
WHERE tenant_id = 'other_tenant'
  AND tenant_id = current_setting('app.current_tenant_id', true);
-- Result: 0 rows (cannot see other tenant's data)
```

**Security guarantee:** RLS is enforced at PostgreSQL kernel level, not application layer.

---

## Entity-Relationship Diagram

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                         rs-llmspell PostgreSQL Schema                        │
│                         15 Tables + Partitions + RLS                        │
└─────────────────────────────────────────────────────────────────────────────┘

┌───────────────────────────────────────────────────────────────────────────┐
│                          VECTOR STORAGE (4 tables)                         │
├──────────────────────┬──────────────────────┬──────────────────────┬──────┤
│vector_embeddings_384 │vector_embeddings_768 │vector_embeddings_1536│ 3072 │
│  id (UUID)           │  id (UUID)           │  id (UUID)           │ ...  │
│  tenant_id           │  tenant_id           │  tenant_id           │      │
│  scope               │  scope               │  scope               │      │
│  embedding VECTOR    │  embedding VECTOR    │  embedding VECTOR    │      │
│  metadata JSONB      │  metadata JSONB      │  metadata JSONB      │      │
│  created_at          │  created_at          │  created_at          │      │
└──────────────────────┴──────────────────────┴──────────────────────┴──────┘
  Indexes: HNSW (cosine similarity), B-tree (tenant, scope, created_at)

┌───────────────────────────────────────────────────────────────────────────┐
│                       TEMPORAL GRAPH (2 tables)                            │
├──────────────────────────────────┬────────────────────────────────────────┤
│ entities                         │ relationships                          │
│  entity_id UUID                  │  relationship_id UUID                  │
│  tenant_id                       │  tenant_id                             │
│  entity_type                     │  from_entity UUID ───┐                 │
│  name                            │  to_entity UUID ─────┼────┐            │
│  properties JSONB                │  relationship_type   │    │            │
│  valid_time_start                │  properties JSONB    │    │            │
│  valid_time_end                  │  valid_time_start    │    │            │
│  transaction_time_start          │  valid_time_end      │    │            │
│  transaction_time_end            │  transaction_time_start   │            │
│  PK: (entity_id, tx_time_start)  │  transaction_time_end│    │            │
└────┬─────────────────────────────┴──────────────────────┴────┴────────────┘
     └───────────────────────────────────────────────────────────┘
  Indexes: GiST (temporal ranges), GIN (properties), B-tree (tenant, type)
  Note: V15 removed foreign keys (bi-temporal versioning requires app enforcement)

┌───────────────────────────────────────────────────────────────────────────┐
│                        STATE STORAGE (3 tables)                            │
├──────────────────────┬───────────────────────┬───────────────────────────┤
│ procedural_patterns  │ agent_states          │ workflow_states           │
│  pattern_id UUID     │  state_id UUID        │  workflow_id              │
│  tenant_id           │  tenant_id            │  tenant_id                │
│  scope               │  agent_id             │  workflow_name            │
│  key                 │  agent_type           │  state_data JSONB         │
│  value               │  state_data JSONB     │  current_step             │
│  frequency           │  schema_version       │  status (enum)            │
│  first_seen          │  data_version         │  started_at               │
│  last_seen           │  checksum (SHA-256)   │  completed_at             │
│  UNIQUE (tenant,     │  UNIQUE (tenant,      │  last_updated             │
│    scope, key, val)  │    agent_id)          │  PK: (tenant, workflow)   │
└──────────────────────┴───────────────────────┴───────────────────────────┘
  Indexes: B-tree (tenant, scope/type, frequency), GIN (state_data JSONB)
  Triggers: Auto-update updated_at, auto-increment data_version

┌───────────────────────────────────────────────────────────────────────────┐
│                   SESSION MANAGEMENT (3 tables)                            │
├──────────────────┬────────────────────────┬─────────────────────────────┤
│ sessions         │ artifact_content       │ artifacts                   │
│  session_id UUID │  content_hash (blake3) │  artifact_id                │
│  tenant_id       │  tenant_id             │  session_id UUID ───┐       │
│  session_data    │  storage_type (enum)   │  tenant_id          │       │
│  status (enum)   │  data BYTEA (<1MB)     │  sequence           │       │
│  created_at      │  large_object_oid      │  content_hash ──────┼────┐  │
│  last_accessed   │  size_bytes            │  metadata JSONB     │    │  │
│  expires_at      │  is_compressed         │  name, type, mime   │    │  │
│  artifact_count  │  reference_count       │  version            │    │  │
│  PK: (tenant,    │  created_at            │  tags TEXT[]        │    │  │
│    session_id)   │  last_accessed_at      │  FK: (tenant,       │    │  │
└────┬─────────────┴──────┬─────────────────┴───content_hash) ────┴────┴──┘
     │                    │                       FK: (tenant, session_id)
     │                    └───────────────────────────────────────┘
     └────────────────────────────────────────────────────────────┘
  Indexes: B-tree (tenant, session, status), GIN (session_data, metadata, tags)
  Triggers: Auto-update timestamps, reference counting for deduplication
  Foreign Keys: artifacts -> sessions (CASCADE), artifacts -> content (RESTRICT)

┌───────────────────────────────────────────────────────────────────────────┐
│                     EVENT SOURCING (2 tables)                              │
├──────────────────────────────────┬────────────────────────────────────────┤
│ event_log (PARTITIONED)          │ hook_history                           │
│  event_id UUID                   │  execution_id UUID                     │
│  tenant_id                       │  tenant_id                             │
│  event_type                      │  hook_id                               │
│  correlation_id UUID             │  hook_type                             │
│  timestamp                       │  correlation_id UUID                   │
│  sequence                        │  hook_context BYTEA (compressed)       │
│  language                        │  result_data JSONB                     │
│  payload JSONB                   │  timestamp                             │
│  PK: (tenant, timestamp, id)     │  duration_ms                           │
│  PARTITION BY RANGE (timestamp)  │  triggering_component                  │
│  Partitions: event_log_YYYY_MM   │  modified_operation                    │
│  Management: ensure_future_...   │  tags TEXT[], retention_priority       │
└──────────────────────────────────┴────────────────────────────────────────┘
  Indexes: B-tree (tenant, time, correlation, type), GIN (payload, metadata)
  Partitioning: Monthly ranges, auto-managed via ensure_future_partitions()

┌───────────────────────────────────────────────────────────────────────────┐
│                           SECURITY (1 table)                               │
├───────────────────────────────────────────────────────────────────────────┤
│ api_keys                                                                   │
│  key_id (PK)                                                               │
│  tenant_id                                                                 │
│  service (e.g., "openai", "anthropic")                                    │
│  encrypted_key BYTEA (pgp_sym_encrypt)                                    │
│  key_metadata JSONB                                                        │
│  created_at, last_used_at, expires_at                                     │
│  is_active, usage_count                                                   │
│  rotated_from, deactivated_at                                             │
│  UNIQUE (tenant, service, is_active) -- One active key per service        │
└───────────────────────────────────────────────────────────────────────────┘
  Indexes: B-tree (tenant+service, expiration, active), GIN (metadata)
  Encryption: pgcrypto (pgp_sym_encrypt/decrypt with app.encryption_key)

┌───────────────────────────────────────────────────────────────────────────┐
│                        CROSS-CUTTING CONCERNS                              │
├───────────────────────────────────────────────────────────────────────────┤
│ Row-Level Security (RLS):                                                  │
│   - ALL tables have 4 policies: SELECT, INSERT, UPDATE, DELETE            │
│   - Filter: tenant_id = current_setting('app.current_tenant_id', true)    │
│   - Performance: <5% overhead (4.9% measured)                             │
│   - Security: Enforced at PostgreSQL kernel level                         │
│                                                                            │
│ Indexing:                                                                  │
│   - B-tree: tenant_id, foreign keys, timestamps (150+ indexes)            │
│   - HNSW: Vector similarity (384, 768, 1536 dimensions)                   │
│   - GiST: Temporal ranges (bi-temporal entities/relationships)            │
│   - GIN: JSONB containment, array searches                                │
│   - Partial: Filtered indexes (expires_at NOT NULL, refcount=0)           │
│                                                                            │
│ Triggers:                                                                  │
│   - Auto-update: updated_at, last_accessed_at (15+ triggers)              │
│   - Versioning: Auto-increment data_version on state changes              │
│   - Referencing: Auto-increment/decrement artifact reference counts       │
│                                                                            │
│ Constraints:                                                               │
│   - CHECK: Valid status enums, positive counts, time range validity       │
│   - UNIQUE: Composite keys (tenant+id), patterns, active service keys     │
│   - FOREIGN KEY: Artifacts -> Sessions (CASCADE), Content (RESTRICT)      │
└───────────────────────────────────────────────────────────────────────────┘

Legend:
  UUID = PostgreSQL UUID type (generated via uuid_generate_v4())
  JSONB = Binary JSON with GIN indexing
  VECTOR = pgvector/VectorChord vector type
  BYTEA = Binary data (compression, encryption, large objects)
  TIMESTAMPTZ = Timestamp with timezone
  TEXT[] = PostgreSQL array type
  HNSW = Hierarchical Navigable Small World (vector similarity index)
  GiST = Generalized Search Tree (temporal range queries)
  GIN = Generalized Inverted Index (JSONB, full-text search)
  RLS = Row-Level Security (multi-tenant isolation)
  PK = Primary Key
  FK = Foreign Key
```

---

## Query Examples

### Vector Similarity Search

```sql
-- Find 10 nearest neighbors to query vector (cosine distance)
SELECT id, scope, metadata, 1 - (embedding <=> $1::vector) AS similarity
FROM llmspell.vector_embeddings_768
WHERE tenant_id = current_setting('app.current_tenant_id', true)
  AND scope = 'session:abc123'
ORDER BY embedding <=> $1::vector
LIMIT 10;
```

### Bi-Temporal Point-in-Time Query

```sql
-- What entities were valid and known at 2025-01-01 00:00:00?
SELECT entity_id, name, properties
FROM llmspell.entities
WHERE tenant_id = current_setting('app.current_tenant_id', true)
  AND valid_time_start <= '2025-01-01 00:00:00+00'
  AND valid_time_end > '2025-01-01 00:00:00+00'
  AND transaction_time_start <= '2025-01-01 00:00:00+00'
  AND transaction_time_end > '2025-01-01 00:00:00+00';
```

### Session with Artifacts

```sql
-- Get session with all artifacts (JOIN)
SELECT s.session_id, s.status, s.artifact_count,
       a.artifact_id, a.name, a.artifact_type, a.size_bytes
FROM llmspell.sessions s
LEFT JOIN llmspell.artifacts a
  ON s.tenant_id = a.tenant_id AND s.session_id = a.session_id
WHERE s.tenant_id = current_setting('app.current_tenant_id', true)
  AND s.status = 'active'
ORDER BY a.sequence;
```

### Event Correlation

```sql
-- Find all events for a correlation ID (across partitions)
SELECT event_id, event_type, timestamp, payload
FROM llmspell.event_log
WHERE tenant_id = current_setting('app.current_tenant_id', true)
  AND correlation_id = $1::uuid
ORDER BY timestamp DESC;
```

### Procedural Pattern Frequency

```sql
-- Top 10 most frequent procedural patterns
SELECT scope, key, value, frequency, last_seen
FROM llmspell.procedural_patterns
WHERE tenant_id = current_setting('app.current_tenant_id', true)
  AND frequency >= 3 -- Learned threshold
ORDER BY frequency DESC
LIMIT 10;
```

---

## Next Steps

- **Performance Tuning**: See [performance-tuning.md](./performance-tuning.md) for HNSW optimization and query tuning
- **Setup Guide**: See [postgresql-setup.md](./postgresql-setup.md) for installation and configuration
- **Backup/Restore**: See [backup-restore.md](./backup-restore.md) for disaster recovery procedures
- **Migration Guide**: See [migration-guide.md](./migration-guide.md) for version upgrades
