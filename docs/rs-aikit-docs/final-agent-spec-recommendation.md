# The rs-aikit Agent Specification: A Unified, Storable, and Streamable Format

**Document Status**: Final Synthesis Recommendation
**Date**: December 13, 2025
**Purpose**: Define a complete agent specification format for rs-aikit that is storable, retrievable, streamable, and platform-agnostic

## Executive Summary

This document synthesizes three comprehensive agent specification analyses into a single, unified specification format for rs-aikit. The specification is designed to be:

1. **Format-Agnostic**: While examples use YAML/JSON, the spec is fundamentally a data structure that can be serialized in any format (YAML, JSON, MessagePack, Protocol Buffers, etc.)
2. **Storable**: Can be persisted to databases, files, or object stores with full fidelity
3. **Retrievable**: Supports partial loading, querying, and indexing of spec components
4. **Streamable**: Can be transmitted incrementally over network protocols (HTTP/2, gRPC, WebSocket)
5. **Versionable**: Includes versioning metadata for evolution and compatibility
6. **Portable**: Can be executed on any compliant runtime (rs-aikit Rust, Python adapters, WASM, etc.)

## 1. Architectural Convergence: The 6-Layer Specification Model

All three source documents converge on a layered architecture, though with slight variations (5-7 layers). This synthesis adopts a **6-layer model** that captures all concerns:

```
┌─────────────────────────────────────────────────────────────┐
│  Layer 6: NETWORK & COLLABORATION                          │
│  Purpose: Agent-to-agent communication, discovery, delegation
│  Protocols: A2A, Agent Cards, multi-agent mesh
│  Spec Fields: network, a2a, discovery, delegation
├─────────────────────────────────────────────────────────────┤
│  Layer 5: ORCHESTRATION & WORKFLOW                         │
│  Purpose: Control flow, state machines, task composition
│  Patterns: Sequential, parallel, conditional, loops, recursion
│  Spec Fields: behavior, workflow, orchestration, state_schema
├─────────────────────────────────────────────────────────────┤
│  Layer 4: COGNITION & REASONING                            │
│  Purpose: LLM configuration, prompting, output parsing
│  Components: Model selection, temperature, system prompts
│  Spec Fields: intelligence, cognition, model, prompts
├─────────────────────────────────────────────────────────────┤
│  Layer 3: PERSISTENCE & MEMORY                             │
│  Purpose: Long-term storage, knowledge retention, state
│  Types: User memory, agent memory, episodic, semantic
│  Spec Fields: memory, storage, knowledge_bases
├─────────────────────────────────────────────────────────────┤
│  Layer 2: CONTEXT ENGINEERING                              │
│  Purpose: Dynamic context assembly, RAG, compression
│  Strategies: Sliding window, pinning, semantic priority
│  Spec Fields: context_policy, context_mounts, rag
├─────────────────────────────────────────────────────────────┤
│  Layer 1: INTEGRATION & TOOLS                              │
│  Purpose: External system connectivity, tool execution
│  Standards: MCP, OpenAPI, custom tools
│  Spec Fields: capabilities, tools, mcp_servers, integration
└─────────────────────────────────────────────────────────────┘
```

### Cross-Cutting Concerns (Meta-Layers)

In addition to the 6 functional layers, the spec includes cross-cutting metadata:

- **Identity**: Unique ID, name, version, author, license, tags
- **Interface**: Input/output schemas (JSON Schema), type contracts
- **Governance**: Security policies, rate limits, data retention, audit trails
- **Observability**: Tracing, metrics, logging, OpenTelemetry configuration

## 2. The Complete Agent Specification Schema

The following schema represents the complete agent specification, synthesized from all three source documents. It is presented in a canonical data structure format that can be serialized to YAML, JSON, or binary formats.

### 2.1 Top-Level Structure

```typescript
// Conceptual TypeScript representation (format-agnostic)
interface AgentSpecification {
  // Spec metadata
  spec_version: string;           // e.g., "1.0.0"
  format_version: string;         // e.g., "aikit-spec-v1"

  // Core sections (6 layers + cross-cutting)
  identity: IdentityBlock;
  interface: InterfaceBlock;
  integration: IntegrationBlock;          // Layer 1
  context_policy: ContextPolicyBlock;     // Layer 2
  memory: MemoryBlock;                    // Layer 3
  intelligence: IntelligenceBlock;        // Layer 4
  behavior: BehaviorBlock;                // Layer 5
  network: NetworkBlock;                  // Layer 6

  // Cross-cutting
  governance: GovernanceBlock;
  observability: ObservabilityBlock;
}
```

### 2.2 Identity Block (Metadata)

**Purpose**: Defines the agent's identity for discovery, governance, and version control.

```typescript
interface IdentityBlock {
  // Unique identifier (UUID, DID, or reverse-domain)
  id: string;                     // e.g., "com.lexlapax.research-analyst"

  // Human-readable metadata
  name: string;                   // Display name
  version: string;                // Semantic version (e.g., "1.2.3")
  description: string;            // Multi-line description

  // Authorship and licensing
  author?: string;
  organization?: string;
  license?: string;               // e.g., "MIT", "Apache-2.0"

  // Taxonomy and classification
  tags?: string[];                // ["research", "rag", "multimodal"]
  category?: string;              // e.g., "assistant", "tool-use", "orchestrator"

  // Multimodality support
  input_modalities?: Modality[];  // ["text", "image", "audio", "video"]
  output_modalities?: Modality[];
}

type Modality = "text" | "image" | "audio" | "video" | "json" | "binary";
```

**Key Insights**:
- **Gemini document** emphasizes multimodality as first-class
- **ChatGPT document** includes category/taxonomy for discovery
- **Claude document** adds organization for enterprise governance

### 2.3 Interface Block (Type Contracts)

**Purpose**: Defines strict input/output schemas for type safety and interoperability.

```typescript
interface InterfaceBlock {
  // Input schema (JSON Schema)
  inputs: {
    type: "object";
    properties: Record<string, JSONSchemaProperty>;
    required?: string[];
    additionalProperties?: boolean;
  };

  // Output schema (enforced on LLM via structured output)
  outputs: {
    type: "object";
    properties: Record<string, JSONSchemaProperty>;
    required?: string[];
  };

  // Conversation starters (optional)
  conversation_starters?: ConversationStarter[];
}

interface ConversationStarter {
  title: string;
  prompt: string;
  context?: Record<string, any>;
}

interface JSONSchemaProperty {
  type: string;
  description?: string;
  enum?: any[];
  default?: any;
  format?: string;
  items?: JSONSchemaProperty;
  properties?: Record<string, JSONSchemaProperty>;
}
```

**Key Insights**:
- **Gemini document** emphasizes schema-driven I/O for Rust type safety
- **ChatGPT document** includes conversation starters (Microsoft pattern)
- **Claude document** enforces strict validation before execution

### 2.4 Integration Block (Layer 1: Tools & Connectivity)

**Purpose**: Defines connections to external systems, tools, and data sources.

```typescript
interface IntegrationBlock {
  // MCP (Model Context Protocol) servers
  mcp_servers?: MCPServer[];

  // Legacy/custom tools (OpenAPI-based)
  tools?: Tool[];

  // Allowed tools (security sandbox)
  allowed_tools?: string[];       // e.g., ["web_browser.search", "filesystem.read_file"]

  // Tool execution policies
  tool_execution?: {
    sandbox?: "wasm" | "docker" | "none";
    timeout_seconds?: number;
    max_retries?: number;
    rate_limit?: RateLimit;
  };
}

interface MCPServer {
  id: string;                     // e.g., "neo4j_graph"
  name?: string;
  transport: "stdio" | "http" | "sse" | "docker";

  // Transport-specific config
  command?: string[];             // For stdio (e.g., ["npx", "mcp-server-filesystem"])
  args?: string[];
  url?: string;                   // For HTTP/SSE
  image?: string;                 // For Docker

  // Authentication
  auth?: {
    type: "none" | "bearer" | "basic" | "apikey";
    token_env?: string;           // Environment variable name
    credentials?: Record<string, string>;
  };

  // Tool filtering
  expose_tools?: string[];        // If omitted, expose all
}

interface Tool {
  name: string;
  type: "openapi" | "wasm_sandbox" | "native" | "mcp";

  // Type-specific fields
  spec_url?: string;              // For OpenAPI
  operations?: string[];
  module?: string;                // For WASM
  permissions?: string[];         // e.g., ["network:deny", "filesystem:readonly"]
  function_ref?: string;          // For native Rust functions
}

interface RateLimit {
  max_calls_per_second?: number;
  max_calls_per_minute?: number;
  burst_size?: number;
}
```

**Key Insights**:
- **Gemini document** emphasizes MCP-native architecture with multiple transports
- **Claude document** adds security sandboxing with WASM/Docker
- **ChatGPT document** includes comprehensive tool filtering

### 2.5 Context Policy Block (Layer 2: Context Engineering)

**Purpose**: Declarative context assembly, RAG coordination, compression strategies.

```typescript
interface ContextPolicyBlock {
  // Window management strategy
  window_management: {
    strategy: "rolling_window" | "semantic_prioritization" | "custom";
    max_tokens: number;           // e.g., 128000
    retention_priority?: "recency" | "semantic_relevance" | "manual";
  };

  // Context layers (assembled in order)
  layers: ContextLayer[];

  // Compression/summarization
  compression?: {
    enabled: boolean;
    trigger_threshold: number;    // 0.0-1.0 (e.g., 0.85 = 85% full)
    method: "recursive_summary" | "extract_key_points" | "custom";
    model_ref?: string;           // Reference to a model in intelligence block
    preserve_pinned: boolean;
  };

  // Context quarantine (prevent context poisoning)
  quarantine?: {
    enabled: boolean;
    isolated_contexts: string[];  // e.g., ["draft_zone", "tool_outputs"]
    promotion_policy: "manual" | "auto_validated" | "never";
  };

  // AGENTS.md integration (Gemini-specific)
  context_mounts?: ContextMount[];
}

interface ContextLayer {
  name: string;
  source: "file" | "temporal_graph" | "agent_memory" | "rag_hybrid" | "mcp_resource" | "custom";
  priority: "critical" | "high" | "medium" | "low";
  pinned?: boolean;               // Never evicted

  // Source-specific config
  path?: string;                  // For file source
  storage_ref?: string;           // For temporal_graph, agent_memory
  query_template?: string;        // Cypher, SQL, etc.

  // For RAG sources
  trigger?: "always" | "on_demand" | "conditional";
  max_tokens?: number;
  retrieval_config?: {
    vector_weight?: number;       // 0.0-1.0
    graph_weight?: number;
    traversal_depth?: number;
    community_summary?: boolean;  // GraphRAG hierarchical summaries
  };

  // Window config (for sliding window sources)
  type?: "sliding_window" | "full_history";
  window_size?: number;
  eviction_strategy?: "fifo" | "preserve_user_messages" | "semantic_priority";
}

interface ContextMount {
  type: "file" | "mcp_resource" | "url";
  path?: string;                  // e.g., "AGENTS.md"
  uri?: string;                   // For MCP resources
  strategy: "inject_start" | "inject_end" | "replace_section";
}
```

**Key Insights**:
- **Claude document** introduces layered context assembly with priorities
- **Gemini document** adds AGENTS.md mounting for environment context
- **ChatGPT document** emphasizes quarantine for context hygiene

### 2.6 Memory Block (Layer 3: Persistence & Knowledge)

**Purpose**: Defines long-term memory stores, knowledge bases, and temporal reasoning.

```typescript
interface MemoryBlock {
  // User memory (cross-session, cross-agent persistent profile)
  user_profile?: UserMemory;

  // Agent memory (working state, procedural knowledge)
  agent_state?: AgentMemory;

  // Knowledge bases (external data sources)
  knowledge_bases?: KnowledgeBase[];
}

interface UserMemory {
  backend: "temporal_knowledge_graph" | "letta_store" | "vector_store" | "custom";
  storage_provider: "neo4j" | "surrealdb" | "qdrant" | "memory" | "sled" | "postgres";

  // Connection details
  connection?: {
    url_env?: string;             // Environment variable for URL
    auth_env?: string;
    host?: string;
    port?: number;
    credentials?: Record<string, string>;
  };

  // Schema definition (for graph stores)
  schema?: {
    entities?: EntityDefinition[];
    relationships?: RelationshipDefinition[];
  };

  // Temporal configuration
  temporal_config?: {
    track_validity: boolean;      // Add valid_from/valid_to timestamps
    conflict_resolution: "latest_wins" | "merge" | "manual";
    history_retention_days?: number;
  };

  // Memory blocks (Letta-style)
  blocks?: MemoryBlock[];

  // Auto-update permissions
  auto_update?: boolean;          // Can agent modify user memory?
}

interface AgentMemory {
  backend: "checkpointed_state" | "vector_store" | "event_log" | "memory";
  storage_provider: string;
  persistence_scope: "thread" | "session" | "global";
  checkpoint_frequency?: "every_step" | "every_n_steps" | "manual";

  // Working memory (short-term)
  working_memory?: {
    type: "sliding_window" | "summary_buffer";
    size: number;                 // Number of messages/tokens
    serialization?: "json" | "messagepack" | "protobuf";
    eviction_policy?: "fifo" | "semantic_priority" | "lru";
  };

  // Procedural memory (learned patterns)
  procedural_memory?: {
    type: "vector_store" | "graph" | "none";
    provider?: string;
    collection?: string;
    embedding_model?: string;
    content_types?: string[];     // e.g., ["successful_examples", "error_patterns"]
  };

  // Episodic memory (event log)
  episodic_memory?: {
    type: "event_log" | "none";
    backend?: string;
    table?: string;
    indexed_fields?: string[];
    retention?: "session_scoped" | "persistent";
  };
}

interface MemoryBlock {
  label: string;                  // e.g., "user_facts", "preferences"
  max_chars?: number;
  description?: string;
}

interface KnowledgeBase {
  id: string;
  type: "graphrag" | "vector_store" | "sql_database" | "document_store" | "custom";
  access_protocol: "mcp" | "native" | "http" | "grpc";
  server_ref?: string;            // Reference to MCP server

  // For GraphRAG
  ontology?: {
    entities?: string[];          // ["Document", "Section", "Entity", "Concept"]
    relationships?: string[];     // ["CONTAINS", "MENTIONS", "RELATES_TO"]
  };

  indexing?: {
    text_embedding?: string;      // Model name
    entity_extraction?: "llm_based" | "ner" | "manual";
    community_detection?: "leiden_algorithm" | "louvain" | "none";
    hierarchy_levels?: number;
  };

  retrieval_policy?: {
    method: "vector" | "graph" | "hybrid" | "sql";
    local_search_threshold?: number;
    global_search_communities?: number;
    max_depth?: number;           // For graph traversal
    top_k?: number;               // For vector search
  };

  // For vector stores
  collection?: string;
  embedding_model?: string;
  distance_metric?: "cosine" | "euclidean" | "dot_product";
}

interface EntityDefinition {
  type: string;                   // e.g., "User", "UserFact"
  properties?: string[];
}

interface RelationshipDefinition {
  type: string;                   // e.g., "HAS_PREFERENCE", "KNOWS"
  temporal?: boolean;
  properties?: string[];
}
```

**Key Insights**:
- **Claude document** emphasizes temporal knowledge graphs (Zep/Graphiti pattern)
- **ChatGPT document** includes Letta-style memory blocks
- **Gemini document** adds GraphRAG with community detection

### 2.7 Intelligence Block (Layer 4: Cognition & Reasoning)

**Purpose**: LLM configuration, prompt architecture, output parsing.

```typescript
interface IntelligenceBlock {
  // Primary model configuration
  primary_model: ModelConfig;

  // Fallback models (for reliability)
  fallback_models?: FallbackModel[];

  // Specialized models
  compression_model?: ModelConfig;
  embedding_model?: ModelConfig;

  // Prompt architecture
  prompts?: {
    system?: PromptDefinition;
    templates?: Record<string, PromptDefinition>;
  };

  // Output schema enforcement
  output_schema?: {
    type: "json_schema" | "pydantic" | "typescript";
    schema_path?: string;
    schema_inline?: JSONSchemaProperty;
    validation: "strict" | "lenient" | "none";
    retry_on_invalid?: boolean;
    max_retries?: number;
  };
}

interface ModelConfig {
  provider: string;               // "openai" | "anthropic" | "ollama" | "local" | "custom"
  model_id: string;               // e.g., "gpt-4-turbo", "claude-sonnet-4-5"
  deployment?: string;            // For Azure, custom endpoints

  // Hyperparameters
  parameters?: {
    temperature?: number;
    max_tokens?: number;
    top_p?: number;
    top_k?: number;
    frequency_penalty?: number;
    presence_penalty?: number;
    stop_sequences?: string[];
  };

  // Fine-tuning (LoRA adapters)
  adapters?: Adapter[];

  // Quantization (for local models)
  quantization?: "int4" | "int8" | "fp16" | "fp32" | "q4_k_m";

  // Structured output mode
  response_format?: "text" | "json_object" | "json_schema";
}

interface FallbackModel extends ModelConfig {
  trigger_on?: ("rate_limit" | "service_unavailable" | "timeout" | "error")[];
}

interface Adapter {
  id: string;
  path?: string;                  // File path to adapter weights
  url?: string;                   // Remote URL
  scale?: number;                 // Adapter mixing weight
}

interface PromptDefinition {
  source: "inline" | "file" | "template";
  content?: string;               // Inline content
  path?: string;                  // File path
  template?: string;              // Template engine (j2, handlebars)

  // Variable substitution
  variables?: Record<string, string>;

  // Context mounting (Gemini pattern)
  context_mounts?: ContextMount[];
}
```

**Key Insights**:
- **Gemini document** emphasizes provider abstraction with rig integration
- **ChatGPT document** includes fine-tuning parameters (LoRA adapters)
- **Claude document** adds structured output enforcement

### 2.8 Behavior Block (Layer 5: Orchestration & Workflow)

**Purpose**: Defines the agent's execution logic, state machines, and control flow.

```typescript
interface BehaviorBlock {
  type: "state_machine" | "state_graph" | "dag" | "sequential" | "custom";

  // Shared state schema across workflow
  state_schema?: StateSchema;

  // Node definitions
  nodes?: Node[];

  // Edge definitions (control flow)
  edges?: Edge[];

  // Workflow-level configuration
  initial_state?: string;         // For state machines
  max_iterations?: number;        // Circuit breaker
  timeout_seconds?: number;
  checkpoint_enabled?: boolean;   // Enable time-travel debugging

  // Guardrails (validation gates)
  guardrails?: Guardrail[];
}

interface StateSchema {
  type: "typescript" | "json_schema" | "pydantic";
  definition?: string;            // Inline schema definition
  path?: string;                  // External schema file
  keys?: string[];                // For simple key-value schemas
}

interface Node {
  id: string;
  type: "llm_call" | "tool_call" | "agent_ref" | "parallel" | "transform" | "conditional" | "custom";
  description?: string;

  // For llm_call
  model_ref?: string;             // Reference to model in intelligence block
  prompt?: string;
  prompt_template?: string;
  output_key?: string;            // State key to write result
  output_parser?: "json" | "text" | "structured";

  // For tool_call
  tool_ref?: string;              // e.g., "neo4j_graph"
  tool_operation?: string;
  input_mapping?: Record<string, string>;

  // For agent_ref (recursive sub-agent)
  source?: string;                // Path to another agent spec
  input_mapping?: Record<string, string>;
  output_mapping?: Record<string, string>;

  // For parallel
  branches?: Branch[];

  // For transform
  operation?: string;
  input?: string;

  // For conditional
  condition?: string;             // Expression to evaluate
  if_branch?: string;             // Node ID
  else_branch?: string;
}

interface Branch {
  id: string;
  type: "tool_call" | "llm_call" | "agent_ref";
  tool_ref?: string;
  tool_operation?: string;
  input_mapping?: Record<string, string>;
  output_key?: string;
}

interface Edge {
  from: string;                   // "START" or node ID
  to: string;                     // "END" or node ID
  condition?: string;             // Optional guard condition
  priority?: number;              // For multiple edges from same node
}

interface Guardrail {
  name: string;
  type: "input_validation" | "output_validation" | "rate_limit" | "content_policy";
  policy_ref?: string;
  enforcement: "strict" | "warn" | "log";
}
```

**Key Insights**:
- **Claude document** emphasizes StateGraph with recursive composition
- **Gemini document** adds SCXML-style state machines for determinism
- **ChatGPT document** includes guardrails for validation

### 2.9 Network Block (Layer 6: Collaboration & Discovery)

**Purpose**: Agent-to-agent communication, discovery, delegation, swarm coordination.

```typescript
interface NetworkBlock {
  // A2A Protocol configuration
  a2a?: A2AConfig;

  // Agent role in multi-agent systems
  role?: "specialist" | "orchestrator" | "worker" | "supervisor" | "custom";

  // Communication protocols
  protocols?: string[];           // e.g., ["a2a", "dacp", "custom"]
}

interface A2AConfig {
  enabled: boolean;

  // Agent card configuration
  agent_card?: {
    url?: string;                 // e.g., "/.well-known/agent.json"
    auto_generate?: boolean;
  };

  // Capabilities (advertised skills)
  capabilities?: Capability[];

  // Delegation (which agents this agent can call)
  delegation?: {
    can_delegate_to?: DelegationTarget[];
    max_delegation_depth?: number;
  };

  // Discovery mechanism
  discovery?: {
    registry_url?: string;
    auto_register?: boolean;
    heartbeat_interval_seconds?: number;
    dns_discovery?: boolean;
  };

  // Task lifecycle management
  task_handlers?: {
    on_input_required?: "prompt_user" | "delegate" | "fail";
    timeout_policy?: "retry" | "cancel" | "escalate";
  };
}

interface Capability {
  id: string;
  name?: string;
  description: string;

  // Input/output schemas
  input_schema?: JSONSchemaProperty;
  output_schema?: JSONSchemaProperty;

  // Metadata
  tags?: string[];
  complexity?: "simple" | "medium" | "complex";
  estimated_duration_seconds?: number;
}

interface DelegationTarget {
  agent_id: string;
  capabilities?: string[];        // Specific capabilities to delegate
  priority?: number;
}
```

**Key Insights**:
- **Claude document** emphasizes A2A protocol with auto-discovery
- **Gemini document** adds radkit integration for Rust implementation
- **ChatGPT document** includes task lifecycle state machine

### 2.10 Governance Block (Cross-Cutting)

**Purpose**: Security, compliance, multi-tenancy, audit trails.

```typescript
interface GovernanceBlock {
  // Security policies
  security?: {
    sandbox?: "strict" | "moderate" | "none";
    allowed_operations?: string[];
    denied_operations?: string[];
    require_approval?: string[];  // Operations requiring human approval
  };

  // Safety constraints
  safety_constraints?: SafetyConstraint[];

  // Multi-tenancy and scope
  tenancy?: {
    scope: "global" | "tenant" | "user" | "session";
    tenant_id?: string;
    isolation_level?: "strict" | "shared_resources" | "none";
  };

  // Data retention
  data_retention?: {
    user_data_ttl_days?: number;
    conversation_data_ttl_days?: number;
    memory_data_ttl_days?: number;
    purge_on_delete?: boolean;
  };

  // Rate limiting
  rate_limiting?: {
    max_requests_per_user?: number;
    window_seconds?: number;
    max_tokens_per_day?: number;
    cost_limit_usd?: number;
  };

  // Audit logging
  audit?: {
    enabled: boolean;
    log_inputs?: boolean;
    log_outputs?: boolean;
    log_tool_calls?: boolean;
    retention_days?: number;
  };
}

interface SafetyConstraint {
  type: "content_policy" | "data_policy" | "usage_policy";
  policy_ref?: string;            // Path to policy file
  enforcement: "strict" | "warn" | "log";
  on_violation?: "block" | "redact" | "flag";
}
```

**Key Insights**:
- **ChatGPT document** emphasizes multi-tenancy with StateScope
- **Gemini document** adds capabilities-based security model
- **Claude document** includes comprehensive audit trails

### 2.11 Observability Block (Cross-Cutting)

**Purpose**: Tracing, metrics, logging, debugging, performance monitoring.

```typescript
interface ObservabilityBlock {
  // Distributed tracing
  tracing?: {
    enabled: boolean;
    provider: "opentelemetry" | "jaeger" | "datadog" | "custom";
    export_endpoint?: string;
    sample_rate?: number;         // 0.0-1.0

    // OpenTelemetry GenAI semantic conventions
    gen_ai_conventions?: boolean;
  };

  // Metrics collection
  metrics?: {
    enabled: boolean;
    provider?: "prometheus" | "statsd" | "custom";
    include?: MetricType[];
    export_interval_seconds?: number;
  };

  // Structured logging
  logging?: {
    level: "trace" | "debug" | "info" | "warn" | "error";
    format: "json" | "text" | "compact";
    structured: boolean;
    include_state_transitions?: boolean;
    include_tool_calls?: boolean;
    include_llm_calls?: boolean;
  };

  // Debug capabilities
  debugging?: {
    enabled: boolean;
    protocol?: "dap" | "custom";  // Debug Adapter Protocol
    breakpoints_enabled?: boolean;
    step_through?: boolean;
    state_inspection?: boolean;
  };
}

type MetricType =
  | "latency"
  | "token_usage"
  | "error_rate"
  | "cache_hit_rate"
  | "tool_invocations"
  | "agent_invocations"
  | "cost";
```

**Key Insights**:
- **Gemini document** emphasizes OpenTelemetry GenAI conventions
- **Claude document** adds comprehensive tracing with correlation IDs
- **ChatGPT document** includes DAP (Debug Adapter Protocol) support

## 3. Serialization, Storage, and Streaming

### 3.1 Serialization Formats

The specification is **format-agnostic** and can be serialized to multiple formats:

| Format | Use Case | Pros | Cons |
|--------|----------|------|------|
| **YAML** | Human editing, Git versioning | Readable, comments, multi-line | Slow parsing, whitespace-sensitive |
| **JSON** | API transmission, web clients | Fast parsing, ubiquitous | No comments, verbose for prompts |
| **MessagePack** | Binary storage, performance | Compact, fast | Not human-readable |
| **Protocol Buffers** | gRPC, high-performance | Strongly typed, compact, schema evolution | Requires .proto definitions |
| **CBOR** | IoT, constrained environments | Compact, self-describing | Less common |

**Recommendation for rs-aikit**:
- **Primary format**: YAML (for human editing and version control)
- **Internal format**: MessagePack (for database storage and performance)
- **Wire format**: JSON or Protobuf (for network transmission)

### 3.2 Storage Strategies

**Database Storage** (for retrieval and querying):

```sql
-- Relational schema (PostgreSQL example)
CREATE TABLE agent_specs (
  id UUID PRIMARY KEY,
  name TEXT NOT NULL,
  version TEXT NOT NULL,
  spec_data JSONB NOT NULL,       -- Full spec as JSON
  created_at TIMESTAMPTZ DEFAULT NOW(),
  updated_at TIMESTAMPTZ DEFAULT NOW(),

  -- Indexed metadata for fast queries
  author TEXT,
  tags TEXT[],
  category TEXT,

  -- Version tracking
  parent_version UUID REFERENCES agent_specs(id),

  UNIQUE(name, version)
);

-- Indexes for fast retrieval
CREATE INDEX idx_agent_specs_name ON agent_specs(name);
CREATE INDEX idx_agent_specs_tags ON agent_specs USING GIN(tags);
CREATE INDEX idx_agent_specs_category ON agent_specs(category);
CREATE INDEX idx_agent_specs_spec_data ON agent_specs USING GIN(spec_data jsonb_path_ops);
```

**Document Storage** (for flexibility):

```javascript
// MongoDB example
{
  _id: ObjectId("..."),
  identity: {
    id: "com.lexlapax.research-analyst",
    name: "Research Analyst",
    version: "1.0.0",
    // ... rest of identity
  },
  // ... rest of spec (full document)

  // Metadata for indexing
  _metadata: {
    created_at: ISODate("..."),
    updated_at: ISODate("..."),
    hash: "sha256:...",           // For integrity checking
  }
}
```

**Object Storage** (for large specs):

```
s3://aikit-specs/
├── by-id/
│   └── com.lexlapax.research-analyst/
│       └── 1.0.0.msgpack
├── by-name/
│   └── research-analyst/
│       └── 1.0.0.yaml
└── manifests/
    └── registry.json              # Index of all specs
```

### 3.3 Streaming Protocol

For large agent specifications (e.g., with embedded prompts, extensive ontologies), support **chunked streaming**:

```typescript
// Streaming protocol (over HTTP/2 or gRPC)
interface SpecStream {
  // Header chunk (always first)
  header: {
    spec_version: string;
    total_size_bytes: number;
    chunk_count: number;
    compression?: "gzip" | "brotli" | "none";
  };

  // Content chunks
  chunks: SpecChunk[];
}

interface SpecChunk {
  sequence: number;
  path: string;                   // JSON path to this section
  data: any;                      // Partial spec data
  checksum: string;               // For integrity
}
```

**Example streaming sequence**:
1. **Chunk 0** (header): `{ spec_version: "1.0.0", total_size_bytes: 524288, chunk_count: 5 }`
2. **Chunk 1** (identity + interface): Minimal data needed to start validation
3. **Chunk 2** (integration + context): Tool and context configuration
4. **Chunk 3** (memory + intelligence): Heavy sections with embeddings
5. **Chunk 4** (behavior): Workflow graph
6. **Chunk 5** (network + governance + observability): Final sections

**Benefit**: Client can begin processing (e.g., validating schemas, initializing connections) while later chunks are still being transmitted.

## 4. Versioning and Evolution

### 4.1 Specification Versioning

The specification uses **semantic versioning** at two levels:

1. **Spec Format Version** (`spec_version`): Version of the specification schema itself
   - Example: `"1.0.0"` → `"1.1.0"` (added optional fields) → `"2.0.0"` (breaking changes)

2. **Agent Version** (`identity.version`): Version of the specific agent implementation
   - Example: `"1.2.3"` → `"1.2.4"` (bug fix) → `"1.3.0"` (new feature)

### 4.2 Schema Evolution

**Forward Compatibility** (new runtime can read old specs):
- New optional fields can be added without breaking old specs
- Deprecation warnings for fields marked for removal

**Backward Compatibility** (old runtime can read new specs):
- New required fields break backward compatibility (major version bump)
- Runtimes should ignore unknown fields gracefully

**Example evolution**:

```yaml
# Version 1.0.0 spec
spec_version: "1.0.0"
identity:
  name: "MyAgent"

# Version 1.1.0 spec (backward compatible)
spec_version: "1.1.0"
identity:
  name: "MyAgent"
  organization: "Acme Corp"  # New optional field

# Version 2.0.0 spec (breaking change)
spec_version: "2.0.0"
identity:
  id: "required-unique-id"   # New required field
  name: "MyAgent"
```

### 4.3 Migration Tools

rs-aikit should provide migration utilities:

```bash
# Upgrade spec from 1.0 to 2.0
aikit spec migrate --from 1.0 --to 2.0 ./agent.yaml

# Validate spec against schema
aikit spec validate ./agent.yaml

# Show diff between versions
aikit spec diff ./agent-v1.yaml ./agent-v2.yaml
```

## 5. Complete Example: Research Analyst Agent

Here's a complete agent specification demonstrating all layers:

```yaml
# Example: Research Analyst with GraphRAG, Temporal Memory, Multi-Agent
spec_version: "1.0.0"
format_version: "aikit-spec-v1"

# ============================================================================
# IDENTITY & METADATA
# ============================================================================
identity:
  id: "com.lexlapax.research-analyst"
  name: "Deep Research Analyst"
  version: "1.2.0"
  description: |
    Multi-source research agent combining vector search, graph traversal,
    and recursive sub-agent composition for comprehensive analysis.
  author: "rs-aikit Engineering"
  organization: "Lexlapax Labs"
  license: "MIT"
  tags: ["research", "graphrag", "multimodal", "temporal-memory"]
  category: "assistant"
  input_modalities: ["text", "image"]
  output_modalities: ["text", "json"]

# ============================================================================
# INTERFACE (TYPE CONTRACT)
# ============================================================================
interface:
  inputs:
    type: object
    properties:
      query:
        type: string
        description: "Research question or topic"
      depth:
        type: string
        enum: ["shallow", "medium", "deep"]
        default: "medium"
      sources:
        type: array
        items:
          type: string
        description: "Specific sources to prioritize"
    required: ["query"]

  outputs:
    type: object
    properties:
      executive_summary:
        type: string
      key_findings:
        type: array
        items:
          type: string
      citations:
        type: array
        items:
          type: object
          properties:
            source_id:
              type: string
            url:
              type: string
            relevance_score:
              type: number
      confidence:
        type: number
        minimum: 0
        maximum: 1
    required: ["executive_summary", "key_findings", "citations"]

  conversation_starters:
    - title: "Research a technical topic"
      prompt: "Provide a comprehensive analysis of quantum computing breakthroughs in 2025"
    - title: "Compare technologies"
      prompt: "Compare GraphRAG vs traditional vector RAG for enterprise AI"

# ============================================================================
# LAYER 1: INTEGRATION & TOOLS
# ============================================================================
integration:
  mcp_servers:
    - id: filesystem
      transport: stdio
      command: ["npx", "@modelcontextprotocol/server-filesystem"]
      args: ["/workspace"]

    - id: neo4j_graph
      transport: http
      url: "http://localhost:8080/mcp"
      auth:
        type: bearer
        token_env: NEO4J_MCP_TOKEN

    - id: qdrant_vectors
      transport: http
      url: "http://localhost:6333/mcp"

  tools:
    - name: web_search
      type: openapi
      spec_url: "https://api.tavily.com/openapi.json"
      operations: ["search"]

    - name: code_executor
      type: wasm_sandbox
      module: ./tools/python_executor.wasm
      permissions: ["network:deny", "filesystem:readonly"]

  allowed_tools:
    - "filesystem.read_file"
    - "filesystem.write_file"
    - "neo4j_graph.cypher_query"
    - "qdrant_vectors.search"
    - "web_search.search"

  tool_execution:
    sandbox: wasm
    timeout_seconds: 30
    max_retries: 2

# ============================================================================
# LAYER 2: CONTEXT ENGINEERING
# ============================================================================
context_policy:
  window_management:
    strategy: semantic_prioritization
    max_tokens: 128000
    retention_priority: semantic_relevance

  layers:
    - name: system_instructions
      source: file
      path: ./prompts/system.md
      priority: critical
      pinned: true

    - name: agents_md_context
      source: file
      path: AGENTS.md
      priority: critical
      pinned: true

    - name: user_profile
      source: temporal_graph
      storage_ref: user_memory
      query_template: |
        MATCH (u:User {id: $user_id})-[r:PREFERS|KNOWS|ASKED_ABOUT]->(n)
        WHERE r.valid_to IS NULL OR r.valid_to > timestamp()
        RETURN n, r ORDER BY r.valid_from DESC LIMIT 15
      priority: critical

    - name: conversation_history
      source: agent_memory
      type: sliding_window
      window_size: 10
      eviction_strategy: preserve_user_messages
      priority: high

    - name: knowledge_retrieval
      source: rag_hybrid
      trigger: always
      max_tokens: 3000
      retrieval_config:
        vector_weight: 0.6
        graph_weight: 0.4
        traversal_depth: 2
        community_summary: true
      priority: medium

  compression:
    enabled: true
    trigger_threshold: 0.85
    method: recursive_summary
    model_ref: compression_model
    preserve_pinned: true

  quarantine:
    enabled: true
    isolated_contexts: ["draft_zone", "tool_outputs"]
    promotion_policy: manual

# ============================================================================
# LAYER 3: PERSISTENCE & MEMORY
# ============================================================================
memory:
  user_profile:
    backend: temporal_knowledge_graph
    storage_provider: neo4j
    connection:
      url_env: NEO4J_URL
      auth_env: NEO4J_AUTH

    schema:
      entities:
        - type: User
          properties: ["name", "email", "created_at"]
        - type: UserFact
          properties: ["statement", "confidence", "source"]
        - type: Preference
          properties: ["category", "value", "strength"]

      relationships:
        - type: HAS_PREFERENCE
          temporal: true
        - type: KNOWS
          temporal: true
          properties: ["learned_at", "confidence"]
        - type: ASKED_ABOUT
          temporal: true
          properties: ["timestamp", "context"]

    temporal_config:
      track_validity: true
      conflict_resolution: latest_wins
      history_retention_days: 90

    auto_update: true

  agent_state:
    backend: checkpointed_state
    storage_provider: postgres
    persistence_scope: thread
    checkpoint_frequency: every_step

    working_memory:
      type: sliding_window
      size: 10
      serialization: json
      eviction_policy: semantic_priority

    procedural_memory:
      type: vector_store
      provider: qdrant
      collection: agent_procedures
      embedding_model: text-embedding-3-small
      content_types: ["successful_examples", "error_patterns"]

    episodic_memory:
      type: event_log
      backend: postgres
      table: agent_episodes
      indexed_fields: ["timestamp", "user_id", "outcome"]
      retention: session_scoped

  knowledge_bases:
    - id: corporate_docs
      type: graphrag
      access_protocol: mcp
      server_ref: neo4j_graph

      ontology:
        entities: ["Document", "Section", "Entity", "Concept", "Event"]
        relationships: ["CONTAINS", "MENTIONS", "RELATES_TO", "CAUSED_BY"]

      indexing:
        text_embedding: text-embedding-3-small
        entity_extraction: llm_based
        community_detection: leiden_algorithm
        hierarchy_levels: 3

      retrieval_policy:
        method: hybrid
        local_search_threshold: 0.7
        global_search_communities: 5
        max_depth: 2

    - id: vector_docs
      type: vector_store
      access_protocol: mcp
      server_ref: qdrant_vectors
      collection: documents
      embedding_model: text-embedding-3-small
      distance_metric: cosine
      top_k: 10

# ============================================================================
# LAYER 4: COGNITION & REASONING
# ============================================================================
intelligence:
  primary_model:
    provider: anthropic
    model_id: claude-sonnet-4-5
    parameters:
      temperature: 0.2
      max_tokens: 4096
      top_p: 0.95
    response_format: json_schema

  fallback_models:
    - provider: openai
      model_id: gpt-4-turbo
      trigger_on: ["rate_limit", "service_unavailable"]

  compression_model:
    provider: openai
    model_id: gpt-4o-mini
    parameters:
      temperature: 0.1
      max_tokens: 2048

  prompts:
    system:
      source: file
      path: ./prompts/system.md
      variables:
        agent_name: "{{identity.name}}"
        capabilities: "{{tools | join(', ')}}"

    templates:
      analyze_intent:
        source: file
        path: ./prompts/analyze_intent.j2

      write_report:
        source: file
        path: ./prompts/write_report.j2

  output_schema:
    type: json_schema
    schema_path: ./schemas/output.json
    validation: strict
    retry_on_invalid: true
    max_retries: 3

# ============================================================================
# LAYER 5: ORCHESTRATION & WORKFLOW
# ============================================================================
behavior:
  type: state_graph

  state_schema:
    type: typescript
    definition: |
      interface AgentState {
        query: string;
        vector_results?: Document[];
        graph_context?: KnowledgeGraph;
        draft_report?: string;
        critique_score?: number;
        iterations: number;
        final_output?: Report;
      }

  nodes:
    # Parallel retrieval
    - id: parallel_retrieval
      type: parallel
      description: "Concurrent vector + graph search"
      branches:
        - id: vector_search
          type: tool_call
          tool_ref: qdrant_vectors
          tool_operation: search
          input_mapping:
            query: "{{state.query}}"
            top_k: 10
          output_key: vector_results

        - id: graph_traversal
          type: tool_call
          tool_ref: neo4j_graph
          tool_operation: cypher_query
          input_mapping:
            query: |
              MATCH path = (start)-[*1..2]-(related)
              WHERE start.content CONTAINS $query_terms
              RETURN path, related
            query_terms: "{{state.query | extract_keywords}}"
          output_key: graph_context

    # Recursive sub-agent
    - id: draft_generation
      type: agent_ref
      description: "Invoke Writer sub-agent"
      source: ./agents/writer_agent.yaml
      input_mapping:
        context: "{{state.vector_results + state.graph_context}}"
        instructions: "Write comprehensive analysis"
      output_key: draft_report

    # Self-critique loop
    - id: quality_check
      type: llm_call
      model_ref: primary_model
      prompt: |
        Rate this draft on accuracy (1-10):
        {{state.draft_report}}

        Output JSON: {"score": <number>, "issues": [<string>]}
      output_key: critique_score
      output_parser: json

    - id: refine_draft
      type: llm_call
      model_ref: primary_model
      prompt: |
        Improve addressing: {{state.critique_score.issues}}
        {{state.draft_report}}
      output_key: draft_report

    - id: finalize
      type: transform
      operation: format_output
      input: "{{state.draft_report}}"
      output_key: final_output

  edges:
    - from: START
      to: parallel_retrieval
    - from: parallel_retrieval
      to: draft_generation
    - from: draft_generation
      to: quality_check
    - from: quality_check
      to: finalize
      condition: "{{state.critique_score.score >= 8}}"
    - from: quality_check
      to: refine_draft
      condition: "{{state.critique_score.score < 8 && state.iterations < 3}}"
    - from: refine_draft
      to: quality_check
    - from: quality_check
      to: finalize
      condition: "{{state.iterations >= 3}}"
    - from: finalize
      to: END

  max_iterations: 10
  timeout_seconds: 300
  checkpoint_enabled: true

  guardrails:
    - name: input_validation
      type: input_validation
      enforcement: strict
    - name: output_schema_check
      type: output_validation
      enforcement: strict

# ============================================================================
# LAYER 6: NETWORK & COLLABORATION
# ============================================================================
network:
  role: specialist
  protocols: ["a2a"]

  a2a:
    enabled: true
    agent_card:
      url: /.well-known/agent.json
      auto_generate: true

    capabilities:
      - id: research_analysis
        name: "Deep Research Analysis"
        description: "Conduct multi-source research with GraphRAG and temporal reasoning"
        input_schema:
          type: object
          properties:
            query:
              type: string
            depth:
              type: string
              enum: ["shallow", "medium", "deep"]
        output_schema:
          type: object
          properties:
            report:
              type: string
              format: markdown
            sources:
              type: array
            confidence:
              type: number
        complexity: complex
        estimated_duration_seconds: 60

    delegation:
      can_delegate_to:
        - agent_id: writer-agent
          capabilities: ["draft_generation"]
          priority: 1
        - agent_id: fact-checker-agent
          capabilities: ["verification"]
          priority: 2
      max_delegation_depth: 2

    discovery:
      registry_url: "https://agent-registry.example.com"
      auto_register: true
      heartbeat_interval_seconds: 60
      dns_discovery: true

    task_handlers:
      on_input_required: delegate
      timeout_policy: retry

# ============================================================================
# GOVERNANCE (CROSS-CUTTING)
# ============================================================================
governance:
  security:
    sandbox: strict
    allowed_operations: ["read", "write", "search"]
    denied_operations: ["delete", "drop_table"]
    require_approval: ["write_file"]

  safety_constraints:
    - type: content_policy
      policy_ref: ./policies/content_safety.yaml
      enforcement: strict
      on_violation: block

  tenancy:
    scope: user
    isolation_level: strict

  data_retention:
    user_data_ttl_days: 90
    conversation_data_ttl_days: 30
    memory_data_ttl_days: 180
    purge_on_delete: true

  rate_limiting:
    max_requests_per_user: 100
    window_seconds: 3600
    max_tokens_per_day: 1000000
    cost_limit_usd: 50.0

  audit:
    enabled: true
    log_inputs: true
    log_outputs: true
    log_tool_calls: true
    retention_days: 365

# ============================================================================
# OBSERVABILITY (CROSS-CUTTING)
# ============================================================================
observability:
  tracing:
    enabled: true
    provider: opentelemetry
    export_endpoint: "http://localhost:4318"
    sample_rate: 1.0
    gen_ai_conventions: true

  metrics:
    enabled: true
    provider: prometheus
    include:
      - latency
      - token_usage
      - error_rate
      - cache_hit_rate
      - tool_invocations
      - cost
    export_interval_seconds: 10

  logging:
    level: info
    format: json
    structured: true
    include_state_transitions: true
    include_tool_calls: true
    include_llm_calls: true

  debugging:
    enabled: true
    protocol: dap
    breakpoints_enabled: true
    step_through: true
    state_inspection: true
```

## 6. Comparison with Industry Standards

### 6.1 vs. Eclipse LMOS ADL

| Feature | Eclipse LMOS ADL | rs-aikit Spec |
|---------|------------------|---------------|
| **Language** | Kotlin/JVM | Format-agnostic (YAML/JSON/MessagePack) |
| **Runtime** | LMOS platform | Any compliant runtime (Rust, Python, WASM) |
| **Memory** | Implicit | Explicit temporal knowledge graphs |
| **Workflow** | Sequential + conditions | Full state graphs with recursion |
| **Context** | Basic | Layered pipeline with quarantine |
| **Protocols** | Custom | A2A + MCP native |
| **Production** | Deutsche Telekom (millions of interactions) | Designed for similar scale |

### 6.2 vs. Oracle Open Agent Spec

| Feature | Open Agent Spec | rs-aikit Spec |
|---------|-----------------|---------------|
| **Maturity** | Production (Oracle products) | Greenfield design |
| **Component Model** | Flows, Tools, Resources | 6-layer architecture |
| **Memory** | Roadmap feature | Core Layer 3 with temporal graphs |
| **Context** | Implicit in prompts | Explicit Layer 2 pipeline |
| **GraphRAG** | Not specified | First-class hybrid retrieval |
| **Streaming** | Not specified | Built-in chunked streaming |

### 6.3 vs. Microsoft Copilot Manifest

| Feature | Microsoft Copilot | rs-aikit Spec |
|---------|-------------------|---------------|
| **Platform** | M365 ecosystem | Platform-agnostic |
| **Capabilities** | OneDrive, SharePoint, Graph | MCP-based, extensible |
| **Actions** | Proprietary plugins | OpenAPI + MCP tools |
| **Context** | M365 data sources | AGENTS.md + temporal graphs + RAG |
| **Observability** | Proprietary telemetry | OpenTelemetry standard |

## 7. Implementation Considerations for rs-aikit

### 7.1 Rust Type System Mapping

The specification maps cleanly to Rust structs:

```rust
// Core spec structure
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct AgentSpecification {
    pub spec_version: String,
    pub format_version: String,

    pub identity: IdentityBlock,
    pub interface: InterfaceBlock,
    pub integration: IntegrationBlock,
    pub context_policy: ContextPolicyBlock,
    pub memory: MemoryBlock,
    pub intelligence: IntelligenceBlock,
    pub behavior: BehaviorBlock,
    pub network: NetworkBlock,
    pub governance: GovernanceBlock,
    pub observability: ObservabilityBlock,
}

// Example block
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct IdentityBlock {
    pub id: String,
    pub name: String,
    pub version: String,
    pub description: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub author: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub organization: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub license: Option<String>,

    #[serde(default)]
    pub tags: Vec<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub category: Option<String>,

    #[serde(default)]
    pub input_modalities: Vec<Modality>,

    #[serde(default)]
    pub output_modalities: Vec<Modality>,
}
```

### 7.2 Validation Pipeline

```rust
// Spec validation trait
pub trait SpecValidator {
    fn validate(&self, spec: &AgentSpecification) -> Result<(), ValidationError>;
}

// Validation pipeline
pub struct ValidationPipeline {
    validators: Vec<Box<dyn SpecValidator>>,
}

impl ValidationPipeline {
    pub fn validate(&self, spec: &AgentSpecification) -> Result<(), Vec<ValidationError>> {
        let mut errors = Vec::new();

        for validator in &self.validators {
            if let Err(e) = validator.validate(spec) {
                errors.push(e);
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

// Example validators
struct SchemaValidator;
struct SecurityValidator;
struct ResourceReferenceValidator;
```

### 7.3 Integration with rs-aikit Kernel

The specification integrates with the rs-aikit-kernel architecture:

```rust
// In aikit-kernel
pub struct IntegratedKernel {
    spec: AgentSpecification,
    runtime: ScriptRuntime,
    event_bus: EventBus,
    hook_registry: HookRegistry,
    // ... other components
}

impl IntegratedKernel {
    pub fn from_spec(spec: AgentSpecification) -> Result<Self> {
        // Validate spec
        let validator = ValidationPipeline::default();
        validator.validate(&spec)?;

        // Initialize components from spec
        let runtime = ScriptRuntime::from_spec(&spec)?;
        let event_bus = EventBus::new();
        let hook_registry = HookRegistry::from_spec(&spec.observability)?;

        Ok(Self {
            spec,
            runtime,
            event_bus,
            hook_registry,
        })
    }
}
```

## 8. Conclusion: A Specification for the Future

This unified agent specification represents the synthesis of three comprehensive architectural analyses, combining:

1. **Oracle's component-based modularity** (Open Agent Spec)
2. **Eclipse's production-validated patterns** (LMOS ADL)
3. **Microsoft's type-safe interface contracts** (Copilot Manifest)
4. **Google's multimodal context awareness** (Gemini patterns)
5. **Anthropic's protocol standards** (MCP)
6. **Linux Foundation's interoperability** (A2A)

**Key Achievements**:
- **Format-Agnostic**: Can be serialized as YAML, JSON, MessagePack, or Protobuf
- **Storable**: Designed for database storage with queryable metadata
- **Retrievable**: Supports partial loading and indexed access
- **Streamable**: Chunked transmission for large specifications
- **Portable**: Can execute on any compliant runtime
- **Comprehensive**: Covers all 6 layers + cross-cutting concerns
- **Production-Ready**: Includes governance, security, observability

**For rs-aikit**, this specification provides:
- A clear target for implementation
- Compatibility with rs-llmspell consolidation plans
- A foundation for the kernel-centric architecture
- Protocol compliance for ecosystem integration
- A path to production deployment at scale

The specification is not just a configuration format—it is a **complete cognitive operating system blueprint** that transforms agents from ephemeral scripts into durable, observable, and composable services.

---

**Document Changelog**:
- 2025-12-13: Final synthesis of chatgpt, claude, and gemini agent spec recommendations
