# Agent Specification Standards: A Strategic Analysis for Portable Agent Definitions

The landscape of agent specification standards is consolidating around a **complementary three-layer stack**: MCP for tool integration, A2A for agent communication, and emerging ADLs for static agent definitions. For a Rust-based framework seeking portability with flexibility, the optimal strategy combines **A2A/MCP protocol compliance** at the communication layer with a **custom schema extending Eclipse LMOS ADL patterns** at the definition layer, while leveraging **OpenAPI as the universal tool specification format**.

## The standards landscape is fragmenting into three distinct layers

Agent specifications have evolved from monolithic frameworks into a **layered protocol stack**, each addressing different concerns. The most significant development is the **December 2024 donation of both MCP and A2A to the Linux Foundation**, signaling industry consensus on these complementary protocols as foundational standards.

**Model Context Protocol (MCP)**, created by Anthropic in November 2024, standardizes how AI models connect to tools, data sources, and resources—what can be termed "vertical integration." Its specification uses **TypeScript as the source of truth** with auto-generated JSON Schema, communicating via **JSON-RPC 2.0** over stdio or streamable HTTP. MCP defines three server primitives (Tools, Resources, Prompts) and two client primitives (Roots, Sampling), with current version **2025-06-18**. Adoption is broad: OpenAI, Google DeepMind, Cloudflare, Block, and thousands of community-built servers.

**Agent-to-Agent Protocol (A2A)**, launched by Google in April 2025, addresses "horizontal integration"—how autonomous agents communicate and collaborate. Its normative source is **Protocol Buffers** (`spec/a2a.proto`) with JSON-RPC, gRPC, and REST bindings. Core abstractions include **Agent Cards** (JSON capability discovery at well-known URLs), **Tasks** (lifecycle-managed work units), and **Messages with Parts**. Over 50 partners support A2A including Salesforce, Atlassian, SAP, and ServiceNow.

The official A2A documentation explicitly states their relationship: *"An agentic application might use A2A to communicate with other agents, while each agent internally uses MCP to interact with its specific tools and resources."*

## Eclipse LMOS ADL provides the most mature agent definition approach

Among agent definition languages, **Eclipse LMOS ADL** stands as the most architecturally comprehensive and production-proven specification. Deployed at Deutsche Telekom's "Frag Magenta" assistant, it provides a **structured, model-neutral DSL** for defining agent behavior with business-friendly syntax.

| Standard | Maturity | Format | LLM-Suitable | Portability |
|----------|----------|--------|--------------|-------------|
| **Eclipse LMOS ADL** | Production | Custom DSL | Excellent | High |
| **MCP** | Production | JSON Schema/TypeScript | Excellent | Very High |
| **A2A** | Production | Protocol Buffers → JSON | Excellent | Very High |
| **AgentML** | Alpha | XML (SCXML extension) | Good | Medium |
| **NextMoca ADL** | Early | JSON Schema | Good | High |
| **LangChain/LangGraph** | Production | Python code | Good | Low |
| **Semantic Kernel** | Production | YAML/OpenAPI | Excellent | Medium |

LangChain/LangGraph agents are fundamentally **code-defined, not schema-specified**—graphs use TypedDict state schemas with Python-based node functions. While they support JSON serialization for checkpoints and OpenAPI generation via LangServe, **no standalone declarative agent interchange format exists**. This limits cross-platform portability.

Semantic Kernel offers more structured specifications through **YAML prompt templates** with execution settings (temperature, model selection) and the **Process Framework** for workflow orchestration, but remains tightly coupled to Microsoft's runtime ecosystem.

## Architectural layering should separate seven distinct concerns

Based on analysis across all major frameworks and cloud provider architectures (Azure AI Foundry, Google Vertex AI Agent Builder, AWS Bedrock AgentCore), the optimal layered architecture for agent specifications separates these concerns:

```
┌─────────────────────────────────────────────────────────┐
│  Layer 7: Presentation (User interfaces, A2A clients)   │
├─────────────────────────────────────────────────────────┤
│  Layer 6: Orchestration (Workflow composition, routing) │
├─────────────────────────────────────────────────────────┤
│  Layer 5: Agent Core (LLM + reasoning loop + tools)     │
├─────────────────────────────────────────────────────────┤
│  Layer 4: Context Engineering (prompts, few-shot, RAG)  │
├─────────────────────────────────────────────────────────┤
│  Layer 3: Memory (short-term/long-term/user/agent)      │
├─────────────────────────────────────────────────────────┤
│  Layer 2: Knowledge (vector stores, knowledge graphs)   │
├─────────────────────────────────────────────────────────┤
│  Layer 1: Integration (MCP tools, external APIs)        │
└─────────────────────────────────────────────────────────┘
```

**Context engineering belongs at Layer 4**, sitting between the agent core and memory systems. It assembles the prompt from: system instructions (persistent behavior), retrieved context (semantic memory via RAG), few-shot examples (episodic memory), and tool definitions (procedural memory). The CoALA framework's three-type memory taxonomy (semantic, episodic, procedural) is becoming standard across implementations.

**Graph and RAG storage belong at Layer 2 (Knowledge)**, providing retrieval services consumed by Layer 4. The emerging **GraphRAG pattern** combines knowledge graph traversal with vector search—the graph identifies structural context while vectors retrieve semantic details.

**Memory differentiation** is critical:
- **Agent memory** (Layer 3): Agent-specific learned behaviors, preferences, persistent state across sessions
- **User memory** (Layer 3): User-specific information persisted across interactions for personalization
- **Conversation memory** (Layer 4): Thread-scoped context within a single interaction, managed via checkpointers

## Workflow-as-agent composition patterns vary significantly across standards

Standards handle the critical question of **presenting complex workflows as agents** through different mechanisms:

**LangGraph** provides the most flexible workflow composition via its **StateGraph** abstraction supporting cycles (unlike DAGs), parallel execution, conditional edges, and nested subgraphs. Workflows compile to `CompiledStateGraph` objects that can be deployed as standalone agents via LangServe, effectively achieving workflow-as-agent composition through code.

**Semantic Kernel's Process Framework** offers structured workflow patterns:
- Sequential execution
- Parallel/concurrent processing  
- Fan-in/fan-out configurations
- Map-reduce strategies
- Conditional branching
- Human-in-the-loop

Each step invokes KernelFunctions and can represent complex sub-processes, with Microsoft Orleans and Dapr providing distributed execution runtimes.

**A2A handles workflow composition** through its Task lifecycle and multi-turn context management. Tasks maintain state (`submitted`, `working`, `completed`, `failed`, `cancelled`, `input_required`) and can reference other tasks via `referenceTaskIds`, enabling workflow orchestration where an orchestrator agent delegates to specialized agents.

**BPMN extensions for agentic workflows** are emerging with new constructs like `AgenticLane`, `AgenticTask`, `AgenticOR`, and `AgenticAND`, including trustworthiness scores for agent reliability. Camunda's implementation displays tools as ad-hoc sub-processes with heatmaps showing tool execution performance.

| Framework | Workflow Format | Composability | Runtime |
|-----------|-----------------|---------------|---------|
| LangGraph | Python code + StateGraph | Subgraphs as nodes | LangServe/Python |
| Semantic Kernel | YAML + Process Builder | Steps as KernelFunctions | Orleans/Dapr |
| A2A | Protocol + Agent Cards | Task delegation | Any A2A-compliant |
| Temporal | Code-first (any SDK) | Activities + Child Workflows | Temporal Server |
| BPMN 2.0 | XML | Sub-processes | Camunda/etc. |

**Temporal** deserves special attention for production agent orchestration—it provides **durable execution** with automatic state persistence, indefinite workflow duration, and native support for human-in-the-loop via Signals. Its December 2025 OpenAI Agents SDK integration demonstrates growing agent ecosystem support.

## Standard overlaps reveal a coherent integration pattern

The standards exhibit clear **complementary boundaries** with minimal harmful overlap:

**MCP and A2A are explicitly designed as complementary layers**:
- MCP: Agent → Tools/Resources (structured, stateless calls)
- A2A: Agent → Agent (stateful, multi-turn collaboration)
- Both use JSON-RPC 2.0, HTTP transport, SSE streaming
- Both donated to Linux Foundation governance

**OpenAPI serves as the universal tool specification language**:
- Semantic Kernel imports OpenAPI plugins natively
- Azure Bedrock Action Groups use OpenAPI 3.0 with Lambda handlers
- Tools like `openapi-mcp` auto-convert OpenAPI specs to MCP servers
- LangServe generates OpenAPI from LangChain runnables

**Agent definition standards (ADL) complement protocol standards**:
- ADL defines *what* an agent is (static definition)
- MCP/A2A define *how* agents interact (dynamic runtime)
- OpenAPI defines *what* tools are available (capability description)

The integration pattern that emerges:

```
Agent Card (A2A) ←→ Agent Definition (ADL) ←→ Tool Specs (OpenAPI) ←→ Tool Access (MCP)
```

## Portability requires separating structural definition from runtime execution

Only certain standards support **both structural portability and runtime execution**:

| Standard | Structural Portability | Runtime Execution | Notes |
|----------|------------------------|-------------------|-------|
| **A2A** | ✅ Agent Cards (JSON) | ✅ Protocol bindings | Cross-platform by design |
| **MCP** | ✅ JSON Schema tools | ✅ JSON-RPC runtime | Cross-platform by design |
| **OpenAPI** | ✅ YAML/JSON specs | ⚠️ Via runtime adapters | Universal definition, varied execution |
| **Eclipse LMOS ADL** | ✅ DSL definitions | ✅ Kotlin runtime | Coupled to LMOS platform |
| **LangChain/LangGraph** | ⚠️ Python code | ✅ Python runtime | Low portability |
| **Semantic Kernel** | ✅ YAML prompts | ✅ .NET/Python/Java | Medium portability |

For maximum portability, **separate concerns into three artifacts**:

1. **Agent Definition** (portable): JSON/YAML schema describing agent identity, capabilities, tools, memory requirements
2. **Tool Specifications** (portable): OpenAPI 3.x documents defining available actions
3. **Runtime Implementation** (platform-specific): Code implementing the agent behavior using portable definitions

## Recommendations for rs-llmspell Rust framework

Based on this analysis, here are strategic recommendations for building your Rust-based agent framework:

### Adopt MCP and A2A as communication protocols

Implement **MCP client and server support** for tool integration—this provides access to the growing ecosystem of MCP servers. The Rust ecosystem already has foundational support via the `mcp-rs` crate and `rmcp` implementations. For agent-to-agent communication, implement **A2A protocol bindings** (JSON-RPC 2.0 over HTTP is the required binding; gRPC is optional).

### Design a custom ADL extending industry patterns

Create a **YAML/JSON agent definition schema** that combines the best elements:

```yaml
# Proposed rs-llmspell Agent Definition Schema (example structure)
agent:
  name: "research-assistant"
  version: "1.0.0"
  description: "Multi-source research agent"
  
model:
  provider: "anthropic"  # or "openai", "ollama", etc.
  model_id: "claude-3-sonnet"
  parameters:
    temperature: 0.7
    max_tokens: 4096
    top_p: 0.95

capabilities:
  tools:
    - source: "openapi"
      spec_url: "https://api.example.com/openapi.json"
    - source: "mcp"
      server: "filesystem"
  memory:
    short_term: { type: "sliding_window", size: 10 }
    long_term: { type: "vector", provider: "qdrant" }
  
prompts:
  system: |
    You are a research assistant...
  templates:
    - name: "search_query"
      template: "Generate search terms for: {{topic}}"

workflow:
  type: "state_machine"  # or "dag", "sequential"
  states: [...]
  transitions: [...]

a2a:
  card_url: "/.well-known/agent.json"
  skills:
    - id: "research"
      input_modes: ["text/plain"]
      output_modes: ["application/json", "text/markdown"]
```

### Implement a seven-layer architecture with clear interfaces

Define **Rust traits for each architectural layer**, enabling pluggable implementations:

```rust
// Layer interfaces (conceptual)
trait PresentationLayer { /* A2A server, CLI, UI */ }
trait OrchestrationLayer { /* Workflow engine */ }
trait AgentCore { /* LLM interaction, reasoning loop */ }
trait ContextEngine { /* Prompt assembly, RAG integration */ }
trait MemoryStore { /* Short/long-term memory */ }
trait KnowledgeLayer { /* Vector stores, graphs */ }
trait IntegrationLayer { /* MCP client, tool execution */ }
```

### Leverage existing Rust ecosystem components

Several production-ready Rust crates align with this architecture:

- **rig-rs**: LLM integration with RAG and multi-agent support
- **rs-graph-llm**: High-performance graph-based workflow execution
- **AutoAgents** (liquidos-ai): Multi-agent framework with YAML workflows and MCP integration
- **pgvecto.rs**: Vector database (20x faster than pgvector)
- **Swiftide**: RAG pipeline framework

### Prioritize OpenAPI as the tool specification format

Use **OpenAPI 3.x as the canonical format for tool definitions**, then:
1. Auto-generate MCP tool servers from OpenAPI specs
2. Generate A2A skill definitions from OpenAPI operations
3. Create Rust bindings via code generation from OpenAPI

This maintains a **single source of truth** for tool capabilities while supporting multiple protocols.

### Support hybrid workflow composition

Implement multiple workflow patterns to match use-case requirements:

- **Sequential**: Simple pipelines using iterator patterns
- **Parallel**: Tokio-based concurrent execution with `FanOutTask` patterns
- **State Machine**: For complex conditional flows (similar to LangGraph's StateGraph)
- **DAG**: For dependency-based task scheduling

Enable **workflows-as-agents** by allowing workflow definitions to be wrapped with A2A Agent Cards, making complex orchestrations discoverable and invocable through standard protocols.

## The path forward balances standardization with innovation

The agent specification landscape is **consolidating around a coherent stack**: MCP for tools, A2A for agents, OpenAPI for definitions, with ADL-style schemas for static configuration. The Linux Foundation's governance of both MCP and A2A signals long-term stability.

For rs-llmspell, the strategic approach is **protocol compliance at boundaries, innovation at the core**. Implement MCP and A2A for interoperability with the broader ecosystem. Create a Rust-native agent definition schema that extends Eclipse LMOS ADL patterns with explicit support for context engineering, memory layering, and workflow composition. Use OpenAPI as the tool specification format to maximize compatibility.

The **key architectural insight** is that portability comes from separating what an agent *is* (static definition) from what an agent *does* (runtime behavior) from how agents *communicate* (protocols). Standards are converging on this separation, and frameworks that embrace it will integrate most smoothly with the emerging agentic ecosystem.