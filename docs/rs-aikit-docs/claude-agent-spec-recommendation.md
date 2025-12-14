# Agent Specification for rs-aikit: A Comprehensive Architecture and Recommendation

**Document Status**: Final Recommendation
**Date**: December 13, 2025
**Authors**: Synthesis of industry research, rs-llmspell learnings, and architectural analysis

## Executive Summary

This document presents a comprehensive agent specification architecture for **rs-aikit**, a next-generation Rust-native agent framework. Drawing from four major architectural analyses, current industry standards (2025), and deep learnings from the rs-llmspell project, we propose a **specification-first, protocol-compliant architecture** that achieves true portability while maintaining Rust's performance and safety guarantees.

**Key Recommendation**: Adopt a **6-layer recursive architecture** with a custom ADL (Agent Definition Language) that extends the best patterns from Oracle's Open Agent Specification and Eclipse LMOS ADL, while ensuring protocol compliance with A2A and MCP at runtime boundaries.

## 1. The Current Landscape: Convergence and Fragmentation

### 1.1 Industry Standards Consolidation (2025)

Recent developments show significant standardization momentum:

1. **Linux Foundation A2A Protocol** (June 23, 2025): Agent-to-agent communication protocol backed by 50+ companies including Google, AWS, Salesforce, SAP, and Microsoft. IBM's ACP is merging into A2A, creating a unified standard. [^1] [^2]

2. **Eclipse LMOS ADL** (October 28, 2025): Industry's first open Agent Definition Language, already in production powering Deutsche Telekom's Frag Magenta assistant across 10 European countries—one of Europe's largest multiagent deployments. [^3] [^4]

3. **Oracle Open Agent Specification**: Framework-agnostic declarative language with WayFlow reference runtime and adapters for LangGraph/AutoGen. Used across Oracle AI products. [^5] [^6]

4. **MCP + A2A Complementarity**: Model Context Protocol (MCP, Anthropic) for tool/data access and A2A for agent collaboration are explicitly designed as complementary standards, not competitors. Both use JSON-RPC 2.0 over HTTP/SSE. [^7]

### 1.2 The Critical Insight: Three Distinct Layers

Analysis across all standards reveals **three orthogonal concerns** that must be separated:

```
┌─────────────────────────────────────────────────────────┐
│  DEFINITION LAYER: What an agent IS                    │
│  Standards: Open Agent Spec, Eclipse LMOS ADL          │
│  Format: YAML/JSON declarative schema                  │
└─────────────────────────────────────────────────────────┘
                          ↓
┌─────────────────────────────────────────────────────────┐
│  PROTOCOL LAYER: How agents COMMUNICATE                │
│  Standards: A2A (agent↔agent), MCP (agent↔tools)       │
│  Format: JSON-RPC 2.0, Agent Cards, Tool Manifests     │
└─────────────────────────────────────────────────────────┘
                          ↓
┌─────────────────────────────────────────────────────────┐
│  RUNTIME LAYER: How agents EXECUTE                     │
│  Implementations: LangGraph, Temporal, rs-aikit         │
│  Format: Code (Python, Rust, etc.)                     │
└─────────────────────────────────────────────────────────┘
```

**Critical Finding**: Portability requires separating WHAT (definition) from HOW (communication) from WHERE (execution). Current frameworks conflate these concerns.

### 1.3 The Four Architectural Perspectives

Our analysis reviewed four comprehensive architectural documents:

| Document | Key Contribution | Architecture Model |
|----------|------------------|-------------------|
| **Gemini-1** | 6-layer recursive hierarchy, context engineering as Layer 2, GraphRAG via MCP, fractal workflows | **6 Layers**: Interface, Context, Persistence, Cognitive, Orchestration, Collaborative |
| **Gemini-2** | Holonic framework, Open Agent Spec + SCXML, 5-layer Rust-focused | **5 Layers**: Interface, Context, Reasoning, Orchestration, Integration |
| **ChatGPT** | ADL + Agent Spec survey, practical integration patterns | **Layered Definition**: ADL for definition, A2A for communication, MCP for tools |
| **Claude (prior)** | Strategic analysis, 7-layer architecture, protocol compliance at boundaries | **7 Layers**: Presentation, Orchestration, Agent Core, Context, Memory, Knowledge, Integration |

**Convergence**: All four documents independently arrive at similar conclusions:
- Context engineering belongs in a dedicated layer (Layer 2-4)
- Memory must be bifurcated (user vs. agent, episodic vs. semantic)
- Workflows should support recursive composition
- Standards compliance (A2A, MCP) is non-negotiable
- Separation of definition from execution enables portability

## 2. The 6-Layer Recursive Architecture for rs-aikit

Building on the strongest insights from all sources, we propose a **6-layer architecture** that explicitly separates concerns while enabling recursive composition.

### 2.1 Layer Overview

```
┌─────────────────────────────────────────────────────────────────────┐
│  Layer 6: COLLABORATIVE NETWORK                                     │
│  ├─ A2A Protocol Interface (JSON-RPC 2.0)                          │
│  ├─ Agent Cards (Capability Discovery)                             │
│  └─ Inter-Agent Task Delegation                                    │
├─────────────────────────────────────────────────────────────────────┤
│  Layer 5: ORCHESTRATION & WORKFLOW                                  │
│  ├─ Graph-based Control Flow (StateGraph)                          │
│  ├─ Recursive Subgraphs (Fractal Agency)                           │
│  ├─ Parallel/Sequential/Conditional/Loop Execution                 │
│  └─ Checkpointing & Time-Travel Debugging                          │
├─────────────────────────────────────────────────────────────────────┤
│  Layer 4: COGNITIVE REASONING                                       │
│  ├─ LLM Provider Abstraction (OpenAI, Anthropic, Ollama)          │
│  ├─ Prompt Rendering & Templating                                  │
│  ├─ Structured Output Parsing (JSON Schema validation)             │
│  └─ Model Configuration (temperature, max_tokens, etc.)            │
├─────────────────────────────────────────────────────────────────────┤
│  Layer 3: PERSISTENCE & MEMORY                                      │
│  ├─ User Memory (Persistent Profile, Preferences)                  │
│  ├─ Agent Memory (Working State, Procedural Knowledge)             │
│  ├─ Temporal Knowledge Graph (Graphiti-style)                      │
│  └─ Checkpointed State (Workflow Resume/Rewind)                    │
├─────────────────────────────────────────────────────────────────────┤
│  Layer 2: CONTEXT ENGINEERING                                       │
│  ├─ Dynamic Context Assembly Pipeline                              │
│  ├─ Selective Retention & Pinned Context                           │
│  ├─ Compression & Summarization Triggers                           │
│  ├─ RAG Coordination (Vector + Graph Retrieval)                    │
│  └─ Context Hygiene & Quarantine                                   │
├─────────────────────────────────────────────────────────────────────┤
│  Layer 1: INTEGRATION & TOOLS                                       │
│  ├─ MCP Client/Server Implementation                               │
│  ├─ Tool Execution Sandbox (WASM/Process Isolation)                │
│  ├─ Knowledge Base Connectors (Vector, Graph, SQL)                 │
│  └─ External API Bridges                                           │
└─────────────────────────────────────────────────────────────────────┘
```

### 2.2 Why Six Layers? The Architectural Justification

**Layer 1 (Integration)** exists independently because tools, data sources, and external systems are infrastructure concerns. MCP standardizes this interface. Unlike Layer 2 (Context), which *orchestrates* what data to retrieve, Layer 1 *executes* the retrieval.

**Layer 2 (Context Engineering)** is the critical innovation. Moving context management out of the cognitive layer (Layer 4) transforms it from an implicit prompt-stuffing exercise into a **deterministic, engineered pipeline**. Context is prepared *before* reasoning, not during it. [^8] [^9]

**Layer 3 (Persistence)** separates *state* from *process*. Memory has a lifecycle independent of any single inference. The temporal knowledge graph architecture (inspired by Zep [^10]) maintains both episodic (events) and semantic (entities/relationships) memory with explicit temporal ordering.

**Layer 4 (Cognitive)** is purely functional: `Context → Decision`. No side effects, no state management, no tool execution. Just reasoning. This separation enables swapping LLM providers without touching orchestration logic.

**Layer 5 (Orchestration)** implements control flow. LangGraph's StateGraph model [^11] with subgraphs enables the **fractal agency pattern**: complex workflows are themselves agents. A "Research Agent" might internally be a graph of 10 sub-agents, but externally presents a simple interface.

**Layer 6 (Collaborative)** handles the agent-to-agent protocol layer. Unlike Layer 1 (tool calls), Layer 6 involves negotiation, delegation, and asynchronous multi-turn interactions with peer agents via A2A.

### 2.3 The Memory Bifurcation Model

Drawing from Zep's temporal knowledge graph architecture [^10] and CoALA framework patterns:

```yaml
memory_architecture:
  # USER MEMORY: Cross-session, cross-agent persistent profile
  user_profile:
    storage: temporal_knowledge_graph
    schema:
      entities:
        - type: User
          attributes: [name, email, preferences, goals]
        - type: UserFact
          attributes: [statement, confidence, source, timestamp]
      relationships:
        - PREFERS
        - INTERESTED_IN
        - ASKED_ABOUT
    temporal_tracking: true  # All edges have valid_from/valid_to

  # AGENT MEMORY: Agent-specific learned behaviors
  agent_state:
    working_memory:
      type: sliding_window
      size: 10_messages
      eviction_policy: semantic_priority

    procedural_memory:
      type: vector_store
      content: [examples, patterns, successful_strategies]
      retrieval: similarity_search

    episodic_memory:
      type: event_log
      retention: session_scoped
      checkpoint_enabled: true  # For time-travel debugging
```

**Key Innovation**: Temporal knowledge graphs (inspired by Zep/Graphiti [^10]) track *when* facts were learned, enabling the agent to reason about information freshness and resolve conflicts ("User said X yesterday but Y today").

### 2.4 Context Engineering as a Deterministic Pipeline

Eclipse LMOS ADL's breakthrough was recognizing that **prompt engineering → context engineering** [^4]. The context pipeline is Layer 2's core:

```rust
// Conceptual Rust trait for context pipeline
trait ContextPipeline {
    fn assemble(&self,
                request: &AgentRequest,
                user_memory: &TemporalKnowledgeGraph,
                agent_memory: &AgentMemory) -> Result<ContextWindow>;
}

// Example implementation
impl ContextPipeline for LayeredContextEngine {
    fn assemble(&self, request, user_memory, agent_memory) -> Result<ContextWindow> {
        let mut context = ContextWindow::new(self.max_tokens);

        // Layer 1: System instructions (pinned, never evicted)
        context.pin(self.system_prompt)?;

        // Layer 2: User profile (from temporal KG)
        let user_facts = user_memory.retrieve_facts_for_user(request.user_id)?;
        context.add_with_priority(user_facts, Priority::Critical)?;

        // Layer 3: Episodic memory (recent conversation)
        let recent_history = agent_memory.get_sliding_window(10)?;
        context.add_with_priority(recent_history, Priority::High)?;

        // Layer 4: RAG retrieval (triggered by request content)
        if self.rag_enabled {
            let retrieved = self.retrieve_knowledge(request, 2000)?;
            context.add_with_priority(retrieved, Priority::Medium)?;
        }

        // Layer 5: Compression if needed
        if context.is_over_threshold(0.85) {
            context.compress_oldest_with_model(self.summary_model)?;
        }

        Ok(context)
    }
}
```

**Declarative Specification** (Layer 2 in agent YAML):

```yaml
context_policy:
  max_tokens: 128000

  layers:
    - name: system_instructions
      source: file
      path: ./prompts/system.md
      priority: critical
      pinned: true

    - name: user_profile
      source: temporal_graph
      query: |
        MATCH (u:User {id: $user_id})-[r]->(f:Fact)
        WHERE r.valid_to IS NULL OR r.valid_to > now()
        RETURN f ORDER BY r.valid_from DESC LIMIT 20
      priority: critical

    - name: conversation_history
      source: agent_memory
      type: sliding_window
      window_size: 10
      priority: high

    - name: knowledge_retrieval
      source: rag_hybrid
      trigger: always
      max_tokens: 2000
      retrieval_config:
        vector_weight: 0.6
        graph_weight: 0.4
        depth: 2  # GraphRAG traversal depth
      priority: medium

  compression:
    enabled: true
    threshold: 0.85
    method: recursive_summary
    model: gpt-4o-mini
```

## 3. The rs-aikit Agent Definition Language (ADL)

### 3.1 Design Principles

Based on analysis of Open Agent Spec [^5], Eclipse LMOS ADL [^3], and rs-llmspell learnings:

1. **YAML-first, code-optional**: Specifications are human-readable YAML, validated against JSON Schema
2. **Trait-based extensibility**: Components implement Rust traits, enabling custom implementations
3. **Recursive composition**: Agents and workflows are interchangeable (fractal agency)
4. **Protocol-agnostic internals**: Specification doesn't mandate A2A/MCP, but runtime provides adapters
5. **Temporal awareness**: All memory/knowledge constructs support temporal reasoning
6. **Observability by default**: All state transitions emit structured events

### 3.2 Complete Specification Schema

```yaml
# rs-aikit Agent Definition Language (ADL) v1.0
spec_version: "1.0.0"

# ============================================================================
# METADATA: Agent Identity and Contract
# ============================================================================
metadata:
  id: "research-analyst-v1"
  name: "Research Analyst Agent"
  version: "1.0.0"
  description: |
    Multi-source research agent that combines vector search, graph traversal,
    and recursive sub-agent composition to produce comprehensive analysis.
  author: "rs-aikit team"
  license: "MIT"
  tags: [research, analysis, graphrag, multi-agent]

# ============================================================================
# LAYER 1: INTEGRATION & TOOLS
# ============================================================================
integration:
  # MCP Tool Servers
  mcp_servers:
    - id: filesystem
      protocol: stdio
      command: ["npx", "@modelcontextprotocol/server-filesystem"]
      args: ["/workspace"]

    - id: neo4j_graph
      protocol: http
      url: "http://localhost:8080/mcp"
      auth:
        type: bearer
        token_env: NEO4J_MCP_TOKEN

    - id: qdrant_vectors
      protocol: http
      url: "http://localhost:6333/mcp"

  # Legacy Tool Definitions (OpenAPI-based)
  tools:
    - name: web_search
      type: openapi
      spec_url: "https://api.tavily.com/openapi.json"
      operations: [search]

    - name: code_executor
      type: wasm_sandbox
      module: ./tools/python_executor.wasm
      permissions: [network:deny, filesystem:readonly]

# ============================================================================
# LAYER 2: CONTEXT ENGINEERING
# ============================================================================
context_policy:
  window_management:
    strategy: semantic_prioritization
    max_tokens: 128000

  layers:
    - name: system_instructions
      source: file
      path: ./prompts/system.md
      priority: critical
      pinned: true

    - name: user_profile
      source: temporal_graph
      storage_ref: user_memory
      query_template: |
        MATCH (u:User {id: $user_id})-[r:HAS_PREFERENCE|KNOWS|ASKED_ABOUT]->(n)
        WHERE r.valid_to IS NULL OR r.valid_to > timestamp()
        RETURN n, r ORDER BY r.valid_from DESC LIMIT 15
      priority: critical

    - name: conversation_context
      source: agent_memory
      type: sliding_window
      window_size: 10
      eviction_strategy: preserve_user_messages
      priority: high

    - name: knowledge_base
      source: hybrid_rag
      trigger: always
      max_tokens: 3000
      config:
        vector_similarity: 0.6
        graph_traversal: 0.4
        traversal_depth: 2
        community_summary: true  # GraphRAG hierarchical summaries
      priority: medium

  compression:
    enabled: true
    trigger_threshold: 0.85
    method: recursive_summary
    model_ref: compression_model
    preserve_pinned: true

  quarantine:
    enabled: true
    isolated_contexts: [draft_zone, tool_outputs]
    promotion_policy: manual  # Requires validation before main context

# ============================================================================
# LAYER 3: PERSISTENCE & MEMORY
# ============================================================================
memory:
  # User Memory: Cross-session persistent profile
  user_profile:
    backend: temporal_knowledge_graph
    storage_provider: neo4j
    connection:
      url_env: NEO4J_URL
      auth_env: NEO4J_AUTH

    schema:
      entities:
        - User: {properties: [name, email, created_at]}
        - UserFact: {properties: [statement, confidence, source]}
        - Preference: {properties: [category, value, strength]}
        - Goal: {properties: [description, status, deadline]}

      relationships:
        - HAS_PREFERENCE: {temporal: true}
        - KNOWS: {temporal: true, properties: [learned_at, confidence]}
        - INTERESTED_IN: {temporal: true}
        - ASKED_ABOUT: {temporal: true, properties: [timestamp, context]}

    temporal_config:
      track_validity: true
      conflict_resolution: latest_wins
      history_retention: 90_days

    auto_update: true  # Agent can modify via specialized tools

  # Agent Memory: Working state and procedural knowledge
  agent_state:
    backend: checkpointed_state
    storage_provider: postgres
    persistence_scope: thread  # Per-conversation thread
    checkpoint_frequency: every_step

    working_memory:
      type: sliding_window
      size: 10
      serialization: json

    procedural_memory:
      type: vector_store
      provider: qdrant
      collection: agent_procedures
      embedding_model: text-embedding-3-small
      content_types: [successful_examples, error_patterns, optimization_hints]

    episodic_memory:
      type: event_log
      backend: postgres
      table: agent_episodes
      indexed_fields: [timestamp, user_id, outcome]
      retention: session_scoped

  # Knowledge Bases: External knowledge sources
  knowledge_bases:
    - id: corporate_docs
      type: graphrag
      access_protocol: mcp
      server_ref: neo4j_graph

      ontology:
        entities: [Document, Section, Entity, Concept, Event]
        relationships: [CONTAINS, MENTIONS, RELATES_TO, CAUSED_BY]

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
# LAYER 4: COGNITIVE REASONING
# ============================================================================
intelligence:
  primary_model:
    provider: anthropic
    model_id: claude-sonnet-4-5
    parameters:
      temperature: 0.2
      max_tokens: 4096
      top_p: 0.95

  fallback_models:
    - provider: openai
      model_id: gpt-4-turbo
      trigger_on: [rate_limit, service_unavailable]

  compression_model:
    provider: openai
    model_id: gpt-4o-mini
    parameters:
      temperature: 0.1
      max_tokens: 2048

  system_prompt:
    source: file
    path: ./prompts/system.md
    variables:
      agent_name: "{{metadata.name}}"
      capabilities: "{{tools | join(', ')}}"

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
  type: state_graph  # Workflow is a graph of nodes/edges

  # Shared state schema across workflow
  state_schema:
    type: typescript  # Or JSON Schema
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

  # Node definitions
  nodes:
    # Parallel retrieval phase
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

    # Recursive sub-agent invocation
    - id: draft_generation
      type: agent_ref
      description: "Invoke Writer sub-agent"
      source: ./agents/writer_agent.yaml  # Another full agent spec
      input_mapping:
        context: "{{state.vector_results + state.graph_context}}"
        instructions: "Write a comprehensive analysis"
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

    # Conditional refinement
    - id: refine_draft
      type: llm_call
      model_ref: primary_model
      prompt: |
        Improve this draft addressing: {{state.critique_score.issues}}
        {{state.draft_report}}
      output_key: draft_report

    # Finalization
    - id: finalize
      type: transform
      operation: format_output
      input: "{{state.draft_report}}"
      output_key: final_output

  # Edge definitions (control flow)
  edges:
    - from: START
      to: parallel_retrieval

    - from: parallel_retrieval
      to: draft_generation

    - from: draft_generation
      to: quality_check

    # Conditional: good enough?
    - from: quality_check
      to: finalize
      condition: "{{state.critique_score.score >= 8}}"

    # Loop back for refinement
    - from: quality_check
      to: refine_draft
      condition: "{{state.critique_score.score < 8 && state.iterations < 3}}"

    - from: refine_draft
      to: quality_check

    # Max iterations reached, use what we have
    - from: quality_check
      to: finalize
      condition: "{{state.iterations >= 3}}"

    - from: finalize
      to: END

  # Workflow-level config
  max_iterations: 10  # Circuit breaker
  timeout: 300_seconds
  checkpoint_enabled: true  # Enable time-travel debugging

# ============================================================================
# LAYER 6: COLLABORATIVE NETWORK
# ============================================================================
network:
  # A2A Protocol Configuration
  a2a:
    enabled: true
    agent_card:
      url: /.well-known/agent.json
      auto_generate: true

    capabilities:
      - id: research_analysis
        description: "Conduct multi-source research and produce comprehensive analysis"
        input_schema:
          type: object
          properties:
            query: {type: string}
            depth: {type: string, enum: [shallow, medium, deep]}
        output_schema:
          type: object
          properties:
            report: {type: string, format: markdown}
            sources: {type: array, items: {type: string}}
            confidence: {type: number}

    delegation:
      can_delegate_to:
        - agent_id: writer-agent
          capabilities: [draft_generation, content_formatting]
        - agent_id: fact-checker-agent
          capabilities: [verification, source_validation]

    discovery:
      registry_url: "https://agent-registry.example.com"
      auto_register: true
      heartbeat_interval: 60_seconds

# ============================================================================
# GOVERNANCE & OBSERVABILITY
# ============================================================================
governance:
  safety_constraints:
    - type: content_policy
      policy_ref: ./policies/content_safety.yaml
      enforcement: strict

    - type: data_retention
      user_data_ttl: 90_days
      conversation_data_ttl: 30_days

    - type: rate_limiting
      max_requests_per_user: 100
      window: 1_hour

  observability:
    tracing:
      enabled: true
      provider: opentelemetry
      export_endpoint: "http://localhost:4318"

    metrics:
      enabled: true
      include: [latency, token_usage, error_rate, cache_hit_rate]

    logging:
      level: info
      structured: true
      include_state_transitions: true
```

### 3.3 Recursive Composition: Workflows as Agents

The fractal agency pattern (inspired by multi-agent research [^12]) means:

```yaml
# parent_agent.yaml
behavior:
  nodes:
    - id: research_subtask
      type: agent_ref  # Invokes another agent
      source: ./agents/research_agent.yaml
      input_mapping:
        query: "{{state.subtask_query}}"
```

```yaml
# research_agent.yaml (could itself have sub-agents)
behavior:
  type: state_graph
  nodes:
    - id: deep_analysis
      type: agent_ref
      source: ./agents/analyst_agent.yaml
```

**Interface Parity**: Every agent exposes the same interface:
- Input: `AgentRequest { user_id, query, context }`
- Output: `AgentResponse { result, artifacts, metadata }`

Whether the agent is a simple LLM call or a 50-node workflow with 10 sub-agents is **implementation detail**, invisible to the caller.

## 4. rs-aikit Runtime Architecture

### 4.1 Core Rust Components

Drawing from rs-llmspell's trait-first philosophy:

```rust
// Core trait definitions (conceptual)

/// Layer 6: Collaborative Network
pub trait A2AProtocol {
    fn generate_agent_card(&self, agent_spec: &AgentSpec) -> AgentCard;
    fn handle_task_request(&self, request: TaskRequest) -> TaskResponse;
    fn discover_agents(&self, capability: &str) -> Vec<AgentEndpoint>;
}

/// Layer 5: Orchestration
pub trait WorkflowEngine {
    fn compile_graph(&self, spec: &BehaviorSpec) -> CompiledGraph;
    fn execute_node(&self, node: &Node, state: &mut State) -> NodeResult;
    fn checkpoint(&self, state: &State) -> CheckpointId;
    fn restore(&self, checkpoint_id: CheckpointId) -> State;
}

/// Layer 4: Cognitive Reasoning
pub trait CognitiveEngine {
    fn infer(&self, context: &ContextWindow) -> LLMResponse;
    fn parse_output(&self, response: &str, schema: &JsonSchema) -> Result<Value>;
}

/// Layer 3: Memory & Persistence
pub trait MemoryStore {
    fn store_user_fact(&mut self, user_id: &str, fact: UserFact) -> Result<()>;
    fn query_temporal_graph(&self, cypher: &str, params: Params) -> Result<Graph>;
    fn checkpoint_state(&mut self, state: &State) -> Result<CheckpointId>;
}

/// Layer 2: Context Engineering
pub trait ContextEngine {
    fn assemble_context(&self,
                       request: &AgentRequest,
                       user_memory: &dyn MemoryStore,
                       agent_memory: &AgentMemory) -> Result<ContextWindow>;
    fn compress(&self, context: &mut ContextWindow, model: &dyn CognitiveEngine) -> Result<()>;
}

/// Layer 1: Integration
pub trait MCPClient {
    fn connect(&mut self, server_config: &MCPServerConfig) -> Result<()>;
    fn list_tools(&self) -> Result<Vec<ToolDefinition>>;
    fn call_tool(&self, tool_name: &str, args: Value) -> Result<Value>;
}

/// Top-level agent runtime
pub struct AgentRuntime {
    spec: AgentSpec,
    network: Box<dyn A2AProtocol>,
    workflow: Box<dyn WorkflowEngine>,
    cognitive: Box<dyn CognitiveEngine>,
    memory: Box<dyn MemoryStore>,
    context: Box<dyn ContextEngine>,
    integration: Vec<Box<dyn MCPClient>>,
}
```

### 4.2 Implementation Roadmap (Phased, rs-llmspell Style)

Based on rs-llmspell's phase-driven approach:

**Phase 1: Foundation (Weeks 1-4)**
- [ ] 1.1: Core ADL schema definition (YAML parser, JSON Schema validation)
- [ ] 1.2: Trait hierarchy (all 6 layers as Rust traits)
- [ ] 1.3: Basic workflow engine (sequential, conditional edges only)
- [ ] 1.4: SQLite-based memory (user facts, conversation history)
- [ ] 1.5: LLM provider abstraction (OpenAI, Anthropic)

**Phase 2: Context & Memory (Weeks 5-8)**
- [ ] 2.1: Context pipeline implementation (layered assembly)
- [ ] 2.2: Temporal knowledge graph (Neo4j or SurrealDB backend)
- [ ] 2.3: Vector store integration (Qdrant client)
- [ ] 2.4: GraphRAG retrieval (hybrid vector + graph traversal)
- [ ] 2.5: Compression & summarization

**Phase 3: Orchestration (Weeks 9-12)**
- [ ] 3.1: StateGraph implementation (petgraph-based)
- [ ] 3.2: Parallel execution (tokio tasks, join/barrier patterns)
- [ ] 3.3: Recursive subgraphs (agent-ref nodes)
- [ ] 3.4: Checkpointing (Postgres-backed state snapshots)
- [ ] 3.5: Time-travel debugging UI

**Phase 4: Protocols (Weeks 13-16)**
- [ ] 4.1: MCP client implementation (stdio + HTTP transports)
- [ ] 4.2: MCP server (expose agent capabilities as MCP tools)
- [ ] 4.3: A2A protocol (JSON-RPC 2.0 server)
- [ ] 4.4: Agent Card generation (auto-generate from spec)
- [ ] 4.5: Agent discovery registry

**Phase 5: Production Hardening (Weeks 17-20)**
- [ ] 5.1: Observability (OpenTelemetry, structured logging)
- [ ] 5.2: Error handling & retry logic
- [ ] 5.3: Rate limiting & quotas
- [ ] 5.4: Security audit (input validation, sandbox escapes)
- [ ] 5.5: Performance optimization (caching, connection pooling)

### 4.3 Rust Crate Ecosystem

Leveraging proven crates (avoiding dependency bloat, rs-llmspell principle):

| Component | Recommended Crate | Justification |
|-----------|-------------------|---------------|
| **ADL Parsing** | `serde`, `serde_yaml`, `schemars` | Industry standard, zero-copy deserialization |
| **Async Runtime** | `tokio` | Required for parallel workflows, HTTP servers |
| **HTTP Server** | `axum` | High-performance, tower-based, A2A endpoint |
| **Graph Engine** | `petgraph` | StateGraph representation, traversal algorithms |
| **Vector Search** | `qdrant-client` or embed `vectorlite-rs` | Reuse rs-llmspell's HNSW impl for local search |
| **Temporal Graph** | `neo4j-rs` or `surrealdb` | Neo4j for production, SurrealDB for embedded |
| **LLM Clients** | `reqwest` (APIs), `candle` (local) | Candle for edge/local inference |
| **MCP** | `mcp-sdk-rs` or build custom | Rust SDK exists, may need extensions |
| **State Machine** | Custom (via petgraph) | SCXML too heavyweight, graph-based is cleaner |
| **Tracing** | `tracing`, `opentelemetry` | Structured logging, production observability |

### 4.4 Example: Executing an Agent Spec

```rust
use rs_aikit::{AgentRuntime, AgentSpec};

#[tokio::main]
async fn main() -> Result<()> {
    // Load agent specification
    let spec = AgentSpec::from_file("./agents/research_analyst.yaml")?;

    // Initialize runtime with default implementations
    let runtime = AgentRuntime::builder()
        .with_spec(spec)
        .with_memory_backend(PostgresMemory::new("postgres://..."))
        .with_vector_store(QdrantStore::new("http://localhost:6333"))
        .with_temporal_graph(Neo4jGraph::new("bolt://..."))
        .with_llm_provider(AnthropicProvider::new(api_key))
        .with_a2a_server("0.0.0.0:8080")
        .build()?;

    // Execute agent
    let request = AgentRequest {
        user_id: "user-123".into(),
        query: "Analyze the impact of GraphRAG on enterprise AI".into(),
        context: Default::default(),
    };

    let response = runtime.execute(request).await?;

    println!("Result: {}", response.result);
    println!("Sources: {:?}", response.metadata.sources);

    Ok(())
}
```

## 5. Key Differentiators: Why rs-aikit?

### 5.1 Specification-First, Not Code-First

Unlike LangGraph (code-defined graphs), CrewAI (Python classes), or AutoGen (Python functions), rs-aikit agents are **declaratively specified** in YAML/JSON. The specification is the source of truth.

**Benefit**: True portability. The same YAML can theoretically be executed by:
- rs-aikit Rust runtime (production, high-performance)
- Python adapter (prototyping, integration with existing tools)
- JavaScript/WASM runtime (browser-based agents)

### 5.2 Protocol-Compliant at Boundaries

A2A and MCP compliance isn't optional—it's built into Layer 1 and Layer 6. This ensures:
- rs-aikit agents can call any MCP server (Anthropic's, OpenAI's, community servers)
- Other agents (built with LangChain, CrewAI, etc.) can delegate to rs-aikit agents via A2A
- Interoperability is guaranteed, not aspirational

### 5.3 Temporal Knowledge Graphs for Memory

Inspired by Zep [^10] (18.5% accuracy improvement, 90% lower latency on temporal reasoning), rs-aikit's Layer 3 uses **temporal knowledge graphs** where every relationship has `valid_from`/`valid_to` timestamps.

**Example**: "User preferred Python in 2023, switched to Rust in 2024" is represented as two temporal edges, enabling the agent to reason about preference evolution.

### 5.4 Context Engineering as Infrastructure

Layer 2 makes context assembly a **first-class engineering concern** [^8] [^9], not an afterthought. The context pipeline is:
- Declaratively specified (YAML config)
- Independently testable (mock memory stores)
- Optimizable (caching, pre-computation)
- Auditable (observability into what was included/excluded)

### 5.5 Rust-Native Performance & Safety

Benchmarks show LangGraph as fastest Python framework [^11], but Rust offers:
- 10-100x faster execution (no GIL, zero-cost abstractions)
- Memory safety (no segfaults, data races)
- Fearless concurrency (tokio for parallel workflows)
- Small binary size (embed agents in edge devices)

**Target**: <10ms tool initialization, <50ms agent creation, <1ms state read/write (matching rs-llmspell performance targets).

### 5.6 Fractal Agency Pattern

Complex workflows are agents. Simple agents are nodes. Everything composes recursively via the `agent_ref` node type. This eliminates the artificial distinction between "single agent" and "multi-agent system."

### 5.7 Production-Grade from Day One

Learning from rs-llmspell's quality gates:
- >90% test coverage (using `llmspell-testing` patterns)
- >95% API documentation
- Zero clippy warnings
- Structured observability (OpenTelemetry)
- Checkpointing & time-travel debugging (for complex workflow debugging)

## 6. Comparison with Existing Standards

### 6.1 rs-aikit ADL vs. Eclipse LMOS ADL

| Feature | Eclipse LMOS ADL | rs-aikit ADL |
|---------|------------------|--------------|
| **Language** | Kotlin/JVM | Rust/native |
| **Focus** | Business-friendly DSL | Engineering-first, type-safe |
| **Memory Model** | Implicit | Explicit temporal knowledge graphs |
| **Workflow** | Sequential + conditions | StateGraph (cycles, parallel, subgraphs) |
| **Protocols** | Custom | A2A + MCP native |
| **Production Use** | Deutsche Telekom (millions of interactions) | Greenfield, targeting similar scale |

**Takeaway**: Eclipse ADL proved the concept at scale. rs-aikit extends it with GraphRAG, temporal reasoning, and protocol compliance.

### 6.2 rs-aikit ADL vs. Open Agent Spec

| Feature | Open Agent Spec | rs-aikit ADL |
|---------|-----------------|--------------|
| **Origin** | Oracle Labs | rs-llmspell learnings |
| **Execution** | WayFlow + adapters (Python) | Native Rust runtime |
| **Memory** | Roadmap feature | Core Layer 3 with temporal graphs |
| **Context** | Implicit in prompts | Explicit Layer 2 pipeline |
| **GraphRAG** | Not specified | First-class via hybrid retrieval |
| **Maturity** | Stable (used in Oracle products) | V1.0 design, pre-implementation |

**Takeaway**: Open Agent Spec is more mature. rs-aikit extends it with memory/context as first-class layers.

### 6.3 Runtime Comparison

| Framework | Language | Portability | GraphRAG | Temporal Memory | A2A/MCP |
|-----------|----------|-------------|----------|----------------|---------|
| **LangGraph** | Python | Low (code-defined) | Via LangChain | No | Via extensions |
| **CrewAI** | Python | Low (class-based) | No | Basic | No |
| **AutoGen** | Python | Medium (JSON config) | No | No | Roadmap |
| **Temporal.io** | Multi-language | High (workflow as code) | No | Durable execution | Custom |
| **rs-aikit** | Rust (spec in YAML) | **Very High** | **Native** | **Yes (temporal KG)** | **Native** |

## 7. Strategic Recommendations

### 7.1 For rs-aikit Implementation

1. **Start with Layer 1-4** (Phases 1-2): Build the core runtime without A2A/MCP initially. Validate the memory and context architecture with real use cases.

2. **Dogfood with rs-llmspell migration**: Migrate 1-2 rs-llmspell agents to rs-aikit specs. This validates the ADL and exposes gaps.

3. **Contribute to standards**: Engage with Open Agent Spec and LMOS communities. Share rs-aikit's temporal memory and context engineering patterns.

4. **Build adapters, not monoliths**: Provide:
   - `rs-aikit → LangGraph` adapter (execute rs-aikit specs in Python)
   - `Open Agent Spec → rs-aikit` converter (import Oracle specs)
   - `LMOS ADL → rs-aikit` converter (import Deutsche Telekom patterns)

5. **Reference implementations**: Build 3-5 canonical agents demonstrating each pattern:
   - Simple chatbot (Layers 1-4 only)
   - Research analyst (with GraphRAG, Layer 2-3 heavy)
   - Multi-agent orchestrator (fractal agency, Layer 5-6)
   - Edge agent (WASM compilation, minimal dependencies)

### 7.2 For Community Adoption

1. **Publish ADL schema**: Make JSON Schema publicly available, solicit feedback
2. **Open-source runtime**: Apache 2.0 license, crates.io publication
3. **Documentation-first**: Every layer has:
   - Architectural rationale (why this design?)
   - API reference (Rust traits)
   - User guide (YAML examples)
   - Migration guide (from LangGraph, CrewAI, etc.)

4. **Interoperability demos**: Show rs-aikit agents calling:
   - OpenAI's MCP servers
   - LangGraph agents via A2A
   - Anthropic's Claude via MCP

### 7.3 For Research & Innovation

1. **Temporal reasoning benchmarks**: Validate that temporal KGs improve on LongMemEval (Zep showed 18.5% gains [^10])

2. **Context engineering ablation studies**: Measure impact of each context layer on task performance

3. **Fractal agency efficiency**: Does recursive composition reduce latency vs. monolithic workflows?

4. **Rust vs. Python performance**: Quantify speedup on representative agent workloads

## 8. Conclusion: The Path Forward

The convergence of industry standards (A2A, MCP), production-validated ADLs (Eclipse LMOS, Open Agent Spec), and architectural insights from four comprehensive analyses points to a clear path:

**rs-aikit should be a specification-first, protocol-compliant, Rust-native agent framework** that:

1. **Defines agents declaratively** via a 6-layer YAML-based ADL
2. **Separates concerns** (context, memory, reasoning, orchestration) into independent, testable layers
3. **Enables fractal composition** where workflows are agents and agents are workflows
4. **Mandates protocol compliance** (A2A for agents, MCP for tools) at runtime boundaries
5. **Prioritizes temporal reasoning** via knowledge graphs for memory
6. **Targets production quality** from day one (>90% test coverage, observability, security)

The specification above provides a **complete blueprint** for implementation. The roadmap is aggressive but achievable in 20 weeks with a small, focused team.

Most importantly, rs-aikit is positioned to **bridge the gap** between:
- Research (LangChain, academic frameworks) and production (Deutsche Telekom scale)
- Flexibility (declarative specs) and performance (Rust runtime)
- Innovation (GraphRAG, temporal memory) and standardization (A2A, MCP, Open Agent Spec)

The future of agentic AI requires interoperability. rs-aikit, by embracing standards at boundaries while innovating at the core, can be a leading implementation of this vision.

---

## References

[^1]: [Linux Foundation Launches A2A Protocol](https://www.linuxfoundation.org/press/linux-foundation-launches-the-agent2agent-protocol-project-to-enable-secure-intelligent-communication-between-ai-agents)
[^2]: [ACP Merges with A2A Under Linux Foundation](https://github.com/orgs/i-am-bee/discussions/5)
[^3]: [Eclipse LMOS ADL Announcement](https://newsroom.eclipse.org/news/announcements/eclipse-lmos-redefines-agentic-ai-industry%E2%80%99s-first-open-agent-definition)
[^4]: [Eclipse LMOS Production Deployment](https://www.globenewswire.com/news-release/2025/10/28/3175143/0/en/Eclipse-LMOS-Redefines-Agentic-AI-with-Industry-s-First-Open-Agent-Definition-Language-ADL-for-Enterprises.html)
[^5]: [Oracle Open Agent Specification](https://oracle.github.io/agent-spec/index.html)
[^6]: [Open Agent Spec GitHub](https://github.com/oracle/agent-spec)
[^7]: [MCP and A2A Complementarity](https://akka.io/blog/mcp-a2a-acp-what-does-it-all-mean)
[^8]: [Context Engineering for AI Agents - Anthropic](https://www.anthropic.com/engineering/effective-context-engineering-for-ai-agents)
[^9]: [Eclipse LMOS ADL Context Engineering](https://eclipse.dev/lmos/docs/arc/adl/)
[^10]: [Zep: Temporal Knowledge Graph for Agent Memory](https://arxiv.org/abs/2501.13956)
[^11]: [LangGraph Multi-Agent Orchestration 2025](https://latenode.com/blog/langgraph-multi-agent-orchestration-complete-framework-guide-architecture-analysis-2025)
[^12]: [Multi-Agent Patterns in LlamaIndex](https://developers.llamaindex.ai/python/framework/understanding/agent/multi_agent/)

---

**Document Changelog**:
- 2025-12-13: Initial comprehensive specification based on 4-document synthesis + 2025 research
