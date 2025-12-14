# **RS-AIKIT: Foundations for a Declarative, Type-Safe Agent Architecture**

## **1\. The Imperative for Rigorous Agent Engineering**

The maturation of generative artificial intelligence has precipitated a paradigm shift in software architecture, moving from deterministic, instruction-based execution to probabilistic, intent-based orchestration. In this transition, the concept of the "AI Agent" has evolved from experimental scripts—typified by early iterations of AutoGPT or basic LangChain loops—into complex, stateful software entities requiring robust definition, reliable execution, and standardized interoperability. As we undertake the specification of rs-aikit, a next-generation agent framework built in Rust, we are presented with a unique opportunity to address the systemic fragilities observed in the current ecosystem.

Drawing from the architectural constraints and "learnings" of the rs-llmspell project, it is evident that the prevailing "code-first" approach to agent definition—where an agent's identity, capabilities, and cognitive architecture are tightly coupled with runtime logic—is insufficient for production-grade systems. This approach leads to brittleness, lack of portability, and significant barriers to observability. To engineer rs-aikit as a superior alternative, we must adopt a **Specification-First** philosophy. This entails treating the agent not as a collection of functions, but as a rigorously defined *service* described by a declarative manifest.

This report provides an exhaustive analysis and recommendation for the rs-aikit Agent Specification. It synthesizes critical industry standards—specifically the **Model Context Protocol (MCP)** 1, the **Agent-to-Agent (A2A) Protocol** 3, and emerging declarative context standards like AGENTS.md 5—into a unified, holistic architecture. By aligning with the "Gemini-style" direction referenced in internal documentation, which prioritizes multimodal context and declarative schemas, this specification positions rs-aikit to leverage Rust’s distinct advantages: memory safety, strict type systems, and high-performance concurrency.

### **1.1 The "Learn" from rs-llmspell: Constraint as a Virtue**

The development of rs-llmspell likely illuminated the friction points inherent in managing Large Language Models (LLMs) within a systems programming language. Unlike Python, where dynamic typing allows for loose coupling between LLM outputs and internal data structures, Rust demands explicit contracts. A key learning from this experience is that **ambiguity is the enemy**. When an agent's tool definitions or output expectations are implicit (buried in prompt text), the Rust compiler cannot assist in validation.

Therefore, the foundational requirement for rs-aikit is **Type Safety at the Edge**. The agent specification must define inputs and outputs not merely as text descriptions, but as rigorous schemas (compatible with schemars and JSON Schema) 7 that the runtime can validate *before* invoking the probabilistic model. This shift from "Prompt Engineering" to "Schema Engineering" ensures that rs-aikit agents are deterministic containers for non-deterministic logic.

### **1.2 The Landscape of Fragmentation**

The current agent landscape is characterized by a "Tower of Babel" problem. Frameworks like LangChain, Semantic Kernel, and AutoGen each employ proprietary definition formats. An agent defined in LangGraph 8 cannot easily communicate with an agent built in Semantic Kernel.9 This fragmentation impedes the creation of "Agent Swarms" or multi-agent meshes.

The rs-aikit specification addresses this by adopting a **Protocol-Native** approach. Rather than inventing a new proprietary format, the recommended specification acts as a "Meta-Manifest" that compiles down to established open standards:

* **Connectivity**: It implements the **Model Context Protocol (MCP)** 2 to standardize tool use, replacing bespoke integrations.  
* **Interoperability**: It implements the **A2A Protocol** 3 to standardize agent-to-agent messaging.  
* **Observability**: It aligns with **OpenTelemetry GenAI Semantic Conventions** 10 to ensure transparent monitoring.

By acting as a unifying layer, rs-aikit moves beyond being just another framework and becomes a robust infrastructure for the decentralized agent web.

## ---

**2\. Deconstructing the Declarative Agent Manifest**

To construct a comprehensive specification for rs-aikit, we must first analyze the state-of-the-art in declarative agent definitions. The industry is converging on the idea that an agent should be defined by a static configuration file (YAML/JSON) rather than executable code. This "Manifest" approach allows for portability, versioning, and static analysis.

### **2.1 The Gemini/Google Influence: Multimodality and Context**

The user's preference for the new-agent-spec-gemini-1.md direction suggests an affinity for the architectural patterns often associated with Google's Gemini and Vertex AI Agent ecosystem. These patterns emphasize two critical capabilities: **Multimodality** and **Massive Context Management**.

Unlike text-only agents, a "Gemini-style" agent spec treats non-text inputs (images, audio, video) as first-class citizens. The specification must therefore include explicit fields for input\_modalities and context\_mounting. In traditional frameworks, adding a file to an agent's context is often an imperative action (calling a function to read and paste text). In a declarative "Gemini-style" spec, this becomes a configuration: the manifest declares a context\_source (e.g., a file glob or MCP resource), and the runtime handles the ingestion, tokenization, and caching.

This declarative context management is crucial for rs-aikit. It allows the runtime to optimize data loading—perhaps pre-caching large documents or using vector databases—without the agent developer writing boilerplate code. It transforms the agent from a script that *reads* data into a system that is *imbued* with data.

### **2.2 Microsoft’s Declarative Agent Schema: A Reference Implementation**

Microsoft has arguably produced the most mature declarative schema to date for its Copilot ecosystem.11 Analyzing their agent.json/agent.yaml structure reveals several rigorous design choices that rs-aikit should emulate:

* **Strict Separation of Capabilities**: Microsoft separates "Instructions" (the system prompt) from "Capabilities" (web search, OneDrive access).12 This prevents the prompt from becoming a "God Object" containing configuration data.  
* **Typed Conversation Starters**: Rather than a simple list of strings, conversation starters are objects with titles and optional context, enabling a richer UI experience.12  
* **Action Definitions**: Actions (plugins) are defined via references to OpenAPI specs, decoupling the API definition from the agent logic.12

However, the Microsoft schema is deeply tied to the M365 ecosystem. rs-aikit must adopt the *structure* of this approach—the strict typing and separation of concerns—but replace the proprietary M365 capabilities with open standards like MCP.

### **2.3 The AGENTS.md Standard: Context Injection**

While Microsoft’s manifest defines *what* the agent is, the emerging AGENTS.md standard 5 defines *where* the agent is operating. An AGENTS.md file acts as a "README for Robots," providing environment-specific context (build commands, coding styles, architectural patterns).

For rs-aikit, integrating AGENTS.md support is a strategic differentiator. The specification should include a mechanism to strictly "mount" the AGENTS.md file found in the working directory. This solves the "Tabula Rasa" problem where an agent enters a new codebase and lacks context. By formally integrating this into the spec, rs-aikit agents become immediately "context-aware" upon instantiation, reading the environment's rules before attempting any actions.

### **2.4 The rs-aikit Manifest Strategy: YAML over TOML**

A critical decision for the rs-aikit specification is the file format. While Rust developers have a strong affinity for TOML due to Cargo 14, **YAML** is the superior choice for agent definitions.

* **Block Scalars for Prompts**: Agent specifications are dominated by large text blocks (System Prompts, Few-Shot Examples). YAML’s block scalar syntax (| for literal, \> for folded) handles multi-line prose with significantly better readability than TOML’s triple-quoted strings.  
* **Hierarchical Depth**: Advanced agent architectures (State Machines, DAGs) require deep nesting. TOML’s flat structure, which relies on dotted keys or \[\[array\_of\_tables\]\], becomes unwieldy for deeply nested logic trees.15 YAML’s indentation-based hierarchy mirrors the logical structure of the agent's cognition.  
* **Industry Alignment**: The broader AI and DevOps ecosystem (Kubernetes, Semantic Kernel, OpenAPI) has standardized on YAML. Adopting YAML ensures rs-aikit interoperability with existing tooling.

Therefore, the rs-aikit specification will be a **YAML-based** Declarative Manifest, rigorously validated against a JSON Schema derived from Rust structs.

## ---

**3\. The Connectivity Layer: Model Context Protocol (MCP) Integration**

The greatest challenge in agent engineering is the "Integration Tax"—the effort required to connect an agent to external systems (databases, APIs, filesystems). In the rs-llmspell era, this likely involved writing bespoke Rust traits and implementations for each tool. For rs-aikit, we must eliminate this tax by adopting the **Model Context Protocol (MCP)** 1 as the exclusive standard for connectivity.

### **3.1 Architecture of an MCP-Native Framework**

MCP fundamentally alters the agent architecture by decoupling the "Host" (the rs-aikit runtime) from the "Server" (the tool provider). Instead of the agent knowing how to query a PostgreSQL database, the agent simply declares a capability requirement for an MCP Server that exposes database tools.

The rs-aikit specification must reflect this decoupling. The capabilities section of the manifest should not contain inline tool logic. Instead, it should define **MCP Connections**:

1. **Server Reference**: A URI or command to start the MCP server (e.g., npx \-y @modelcontextprotocol/server-postgres).  
2. **Tool Selection**: An allow-list of specific tools from that server to expose to the agent (e.g., postgres.query\_readonly, but *not* postgres.drop\_table).

This architecture delegates the complexity of API interactions to the MCP ecosystem, allowing rs-aikit to focus purely on orchestration and reasoning.

### **3.2 Implementing the MCP Host in Rust**

To support this specification, rs-aikit must implement a full MCP Host runtime. Leveraging the mcp-rust-sdk 16 or the production-ready mcp-protocol-sdk 17, the framework needs to manage the lifecycle of these connections.

The specification must allow for distinct transport types supported by MCP:

* **Stdio Transport**: For local tools (e.g., filesystem access, git operations). The spec will define the executable command and arguments.18  
* **SSE (Server-Sent Events)**: For remote agents or managed services. The spec will define the endpoint URL.

This distinction allows rs-aikit agents to be **Hybrid**: capable of manipulating local files via Stdio while simultaneously querying a remote knowledge base via SSE, all defined declaratively in the YAML manifest.

### **3.3 Security Sandboxing via Specification**

A major advantage of a declarative MCP spec is security. By explicitly listing allowed tools in the manifest, rs-aikit can enforce a **Capabilities-Based Security Model**.

* The runtime parses the manifest *before* the agent starts.  
* It establishes connections only to the specified MCP servers.  
* It filters the exposed tools against the allow-list.

This "Least Privilege" approach addresses a critical security gap in current Python frameworks, where agents often have implicit, unrestricted access to the environment. In rs-aikit, if a tool isn't in the spec, the agent cannot call it.

## ---

**4\. The Social Layer: Agent-to-Agent (A2A) Protocol Implementation**

As agents proliferate, they will inevitably need to collaborate. The **Agent-to-Agent (A2A) Protocol** 3 provides the standardized handshake for this collaboration, allowing agents to discover, task, and query one another regardless of their underlying implementation.

### **4.1 Identity and Discovery**

For an rs-aikit agent to be a good citizen of the A2A network, its specification must serve as its passport. The metadata section of the rs-aikit spec must map 1:1 to the fields required for the **A2A Agent Card**.4

* **Identity**: Unique ID (DID or UUID), Name, and Description.  
* **Topics**: A list of topics/tags the agent specializes in.  
* **Interface Definition**: The exact MIME types of inputs and outputs the agent accepts.

When the rs-aikit runtime loads an agent, it should automatically spin up the .well-known/agent.json endpoint required by A2A, populating it directly from the YAML manifest. This zero-configuration discovery mechanism ensures that every rs-aikit agent is "born" ready to participate in a swarm.

### **4.2 The Task Lifecycle State Machine**

A2A defines a rigorous state machine for tasks: Submitted $\\rightarrow$ Working $\\rightarrow$ InputRequired $\\rightarrow$ Completed (or Failed).19 The rs-aikit runtime must abstract this complexity away from the developer.

The specification should define **Task Handlers**—logic flows that trigger when a task enters a specific state. For example, the manifest might define an on\_input\_required policy that automatically routes clarification requests to a human-in-the-loop (HITL) interface or a delegated "Manager Agent." By baking A2A compliance into the spec, rs-aikit ensures that long-running tasks are resilient and observable.

### **4.3 radkit Integration**

The Rust ecosystem already possesses a robust A2A implementation in radkit.3 rs-aikit should integrate radkit as a core dependency for its networking layer. The specification will effectively act as a configuration layer for radkit's primitives. For instance, defining a "Skill" in the rs-aikit YAML should automatically register a radkit::Skill with the associated A2A metadata. This synergy allows rs-aikit to piggyback on radkit's protocol correctness while providing a higher-level, more ergonomic developer experience.

## ---

**5\. Cognitive Architecture: Defining the "Brain"**

The core of an agent is its cognitive loop—the iterative process of perceiving, reasoning, and acting. Most frameworks hardcode this loop (e.g., the standard "ReAct" loop). rs-aikit should aim higher, allowing the cognitive architecture itself to be defined declaratively.

### **5.1 The rig Runtime Abstraction**

To execute the cognitive functions defined in the spec, rs-aikit should leverage the rig library.20 rig provides high-performance, async-native abstractions for LLMs (CompletionModel), Vector Stores (VectorStoreIndex), and basic agent loops.

The rs-aikit specification acts as a "Hydrator" for rig.

* The cognition.model section of the YAML maps to rig's provider configuration (instantiating an OpenAI or Cohere client).  
* The cognition.memory section configures rig's vector store connections.

This separation protects the agent definition from churn in the underlying model providers. Switching from GPT-4 to Claude 3.5 Sonnet becomes a one-line change in the YAML, with the rig runtime handling the API differences transparently.

### **5.2 Declarative Orchestration: State Machines over Loops**

While simple agents operate in a loop, complex enterprise agents operate as state machines. The rs-aikit spec should support a **Declarative Finite State Machine (FSM)** definition within the orchestration section.22

Inspired by LangGraph 8 but serialized in YAML, this approach allows developers to define:

* **States**: Distinct modes of operation (e.g., "Researching", "Drafting", "Reviewing").  
* **Transitions**: Logic gates that determine movement between states (e.g., "If research\_quality \> 0.8, move to Drafting").  
* **Guardrails**: Validation steps that must pass before a transition occurs.

By defining the orchestration graph in data rather than code, rs-aikit enables powerful capabilities like **Time-Travel Debugging** (replaying a session step-by-step) and **Visual Graph Rendering** (generating a diagram of the agent's logic from its spec).

### **5.3 Memory Architectures**

The specification must distinguish between **Short-Term Memory** (Context Window) and **Long-Term Memory** (Vector Store/Database).24

* **Short-Term**: The spec should define the context\_window strategy (e.g., "sliding\_window", "summary\_buffer"). rig handles the mechanics of truncating conversation history.  
* **Long-Term**: The spec should define memory\_namespaces. An agent might have access to a "Global" namespace (shared facts) and a "Session" namespace (user-specific details). The runtime mounts these stores via MCP or native rig vector stores.

## ---

**6\. The Interface Contract: Ensuring Type Safety**

The defining feature of a Rust-based framework must be type safety. In rs-aikit, the boundary between the deterministic runtime and the probabilistic LLM must be guarded by strict schemas.

### **6.1 Schema-Driven I/O**

The interface section of the rs-aikit specification is not optional documentation; it is a binding contract.

* **Inputs**: Defined via JSON Schema properties. The runtime validates all incoming requests against this schema before the agent is even instantiated.  
* **Outputs**: Defined similarly. This schema is critical for modern LLMs that support "Structured Output" or "JSON Mode".25

By feeding the output schema directly to the LLM (e.g., via OpenAI's response\_format), rs-aikit can guarantee that the agent returns valid, parseable JSON. This solves the "retry loop" problem where frameworks waste tokens begging the model to fix its syntax.

### **6.2 schemars Integration**

In the Rust implementation, the schemars crate 7 is the bridge.

* **Compile Time**: Developers can define Rust structs and derive JsonSchema.  
* **Runtime**: The rs-aikit runtime can generate the JSON Schema from these structs to populate the A2A Agent Card.  
* **Validation**: Conversely, the runtime can read the YAML spec, generate a validator, and ensure dynamic inputs match the definition.

This creates a "Contract-First" development lifecycle: Define the spec $\\rightarrow$ Generate the Types $\\rightarrow$ Implement the Logic.

## ---

**7\. Observability and Governance**

In professional environments, an agent is a "Black Box" that must be monitored. The rs-aikit specification plays a crucial role in enabling observability.

### **7.1 OpenTelemetry GenAI Conventions**

The specification should map directly to **OpenTelemetry (OTel) GenAI Semantic Conventions**.10

* The metadata.name and metadata.version fields in the YAML become the gen\_ai.system.name and gen\_ai.system.version attributes in every trace.  
* The cognition.model field populates gen\_ai.request.model.  
* Tool definitions in capabilities map to gen\_ai.tool.name.

By adhering to these conventions, rs-aikit agents instantly integrate with observability platforms like Honeycomb, Datadog, or Arize Phoenix 27, providing traces that visualize the agent's thought process, tool usage, and latency without custom instrumentation code.

### **7.2 The "Agent Compiler": Static Analysis**

Because the agent is defined declaratively, rs-aikit can introduce a novel tool: the **Agent Compiler** (aikit check). This CLI tool performs static analysis on the YAML manifest:

* **Integrity Check**: Do all referenced prompt templates exist?  
* **Capability Check**: Are all tools used in the orchestration graph actually declared in the capabilities section?  
* **Schema Check**: Do the prompt variables match the input schema fields?

This pre-flight validation prevents a vast class of runtime errors, moving rs-aikit closer to the reliability of compiled software.

## ---

**8\. Strategic Recommendation: The rs-aikit Specification**

Based on the preceding analysis, the following section outlines the concrete recommendation for the rs-aikit agent specification. This format is designed to be comprehensive, modular, and strictly typed.

### **8.1 Specification Structure Overview**

The specification is defined in a single YAML file (e.g., aikit.yaml), composed of five primary blocks:

1. **identity**: Metadata for A2A discovery and observability.  
2. **interface**: Strict I/O schemas defining the agent's contract.  
3. **cognition**: Configuration of the LLM, system prompts, and memory.  
4. **capabilities**: Declarative connections to MCP servers and local tools.  
5. **orchestration**: The logic flow (State Machine) governing execution.

### **8.2 Detailed Schema Reference**

#### **8.2.1 Block 1: Identity (A2A Compliance)**

This block ensures the agent is discoverable and governable.

YAML

identity:  
  \# Unique identifier (UUID or reverse-domain)  
  id: "com.lexlapax.researcher"  
  \# Human-readable display name  
  name: "Deep Research Specialist"  
  \# Semantic versioning  
  version: "1.0.0"  
  \# Detailed description for A2A discovery/router selection  
  description: \>  
    An autonomous research agent capable of performing deep technical analysis,  
    synthesizing web sources, and producing academic-grade reports.  
  \# Authorship and license for governance  
  author: "rs-aikit Engineering Team"  
  license: "MIT"  
  \# Taxonomy tags  
  tags: \["research", "summarization", "technical-writing"\]

#### **8.2.2 Block 2: Interface (Type Safety)**

This block defines the contract. It maps to JSON Schema.

YAML

interface:  
  \# Input Schema: Validated before execution  
  inputs:  
    properties:  
      topic:  
        type: string  
        description: "The primary subject of research."  
      depth:  
        type: string  
        enum: \["brief", "comprehensive", "academic"\]  
        default: "comprehensive"  
    required: \["topic"\]

  \# Output Schema: Enforced on the LLM via Structured Output  
  outputs:  
    type: object  
    properties:  
      executive\_summary:  
        type: string  
      key\_findings:  
        type: array  
        items:  
          type: string  
      citations:  
        type: array  
        items:  
          type: object  
          properties:  
            source\_id: { type: string }  
            url: { type: string }

#### **8.2.3 Block 3: Cognition (Runtime Configuration)**

This block configures the rig runtime.

YAML

cognition:  
  \# Model Selection  
  model:  
    provider: "openai" \# Maps to rig::providers::openai  
    deployment: "gpt-4-turbo"  
    parameters:  
      temperature: 0.2  
      max\_tokens: 4096  
      top\_p: 0.95

  \# Prompt Architecture  
  prompts:  
    system:  
      \# Use external template for maintainability  
      template: "prompts/system.j2"  
        
      \# Context Mounting: The AGENTS.md integration  
      context\_mounts:  
        \- type: "file"  
          path: "AGENTS.md"  
          strategy: "inject\_start" \# Inject at start of system prompt  
        \- type: "mcp\_resource"  
          uri: "postgres://knowledge\_base/schema"  
          strategy: "inject\_end"

#### **8.2.4 Block 4: Capabilities (MCP Integration)**

This block defines the agent's reach.

YAML

capabilities:  
  \# MCP Server Connections  
  mcp\_servers:  
    \- name: "web\_browser"  
      transport: "docker"  
      image: "mcp/browser-server:latest"  
        
    \- name: "filesystem"  
      transport: "stdio"  
      command: "npx"  
      args: \["-y", "@modelcontextprotocol/server-filesystem", "./workspace"\]

  \# Explicit Tool Allow-list (Security Sandbox)  
  allowed\_tools:  
    \- "web\_browser.search"  
    \- "web\_browser.read\_page"  
    \- "filesystem.write\_file"  
    \# Note: 'filesystem.delete\_file' is omitted, effectively sandboxing the agent

#### **8.2.5 Block 5: Orchestration (State Machine)**

This block defines the logic flow.

YAML

orchestration:  
  \# Declarative Finite State Machine  
  type: "state\_machine"  
  initial\_state: "analyze"  
    
  states:  
    analyze:  
      action: "call\_llm"  
      prompt\_template: "prompts/analyze\_intent.j2"  
      next:  
        \- if: "output.needs\_info"  
          target: "research"  
        \- target: "draft"  
      
    research:  
      action: "call\_tool"  
      tool: "web\_browser.search"  
      next:  
        \- target: "analyze"  
          
    draft:  
      action: "call\_llm"  
      prompt\_template: "prompts/write\_report.j2"  
      \# Enforce output schema here  
      response\_format: "json\_object"  
      next:  
        \- target: "end"

### **8.3 Comparison with Alternatives**

The following table summarizes why this specific architecture is superior for rs-aikit.

| Feature | rs-aikit Spec | Microsoft Copilot Manifest | LangChain (Python) | AutoGen |
| :---- | :---- | :---- | :---- | :---- |
| **Format** | YAML (Declarative) | JSON/YAML (Declarative) | Python Code (Imperative) | Python Code (Imperative) |
| **Connectivity** | **MCP Native** | Proprietary Plugins | Proprietary Tools | Proprietary Skills |
| **Interoperability** | **A2A Protocol** | M365 Ecosystem | LangGraph Protocol | AutoGen Protocol |
| **Type Safety** | **Strict (JSON Schema)** | Strict | Loose (Dynamic) | Loose |
| **Context** | **AGENTS.md Mounting** | OneDrive/SharePoint | Manual Loading | Manual Loading |
| **Observability** | **OTel GenAI SemConv** | Proprietary Telemetry | LangSmith (Proprietary) | Custom Logging |

## ---

**9\. Conclusion**

The rs-aikit project represents a necessary evolution in agent engineering. By synthesizing the rigorous connectivity of MCP, the interoperability of A2A, and the contextual awareness of AGENTS.md into a single, type-safe YAML specification, we create a framework that is robust by default.

This specification does not merely define an agent; it defines a verifiable contract for an autonomous service. It leverages Rust’s strengths to deliver safety and performance while providing a developer experience that is declarative, composable, and future-proof. Implementing this specification will position rs-aikit as a reference architecture for high-reliability Agentic AI.

#### **Works cited**

1. Specification \- Model Context Protocol （MCP）, accessed December 13, 2025, [https://modelcontextprotocol.info/specification/](https://modelcontextprotocol.info/specification/)  
2. Specification \- Model Context Protocol, accessed December 13, 2025, [https://modelcontextprotocol.io/specification/2025-11-25](https://modelcontextprotocol.io/specification/2025-11-25)  
3. agents-sh/radkit: Rust Agent Development Kit \- GitHub, accessed December 13, 2025, [https://github.com/agents-sh/radkit](https://github.com/agents-sh/radkit)  
4. a2a-client \- crates.io: Rust Package Registry, accessed December 13, 2025, [https://crates.io/crates/a2a-client](https://crates.io/crates/a2a-client)  
5. AGENT.md: The Universal Agent Configuration File \- GitHub, accessed December 13, 2025, [https://github.com/agentmd/agent.md](https://github.com/agentmd/agent.md)  
6. AGENTS.md \- Factory Documentation, accessed December 13, 2025, [https://docs.factory.ai/cli/configuration/agents-md](https://docs.factory.ai/cli/configuration/agents-md)  
7. agent-client-protocol-schema \- crates.io: Rust Package Registry, accessed December 13, 2025, [https://crates.io/crates/agent-client-protocol-schema/0.6.1/dependencies](https://crates.io/crates/agent-client-protocol-schema/0.6.1/dependencies)  
8. A Beginner's Guide to Getting Started in Agent State in LangGraph, accessed December 13, 2025, [https://dev.to/aiengineering/a-beginners-guide-to-getting-started-in-agent-state-in-langgraph-3bkj](https://dev.to/aiengineering/a-beginners-guide-to-getting-started-in-agent-state-in-langgraph-3bkj)  
9. YAML schema reference for Semantic Kernel prompts, accessed December 13, 2025, [https://learn.microsoft.com/en-us/semantic-kernel/concepts/prompts/yaml-schema](https://learn.microsoft.com/en-us/semantic-kernel/concepts/prompts/yaml-schema)  
10. Semantic conventions for generative AI systems | OpenTelemetry, accessed December 13, 2025, [https://opentelemetry.io/docs/specs/semconv/gen-ai/](https://opentelemetry.io/docs/specs/semconv/gen-ai/)  
11. Agent manifest | Microsoft Learn, accessed December 13, 2025, [https://learn.microsoft.com/en-us/copilot/security/developer/agent-manifest](https://learn.microsoft.com/en-us/copilot/security/developer/agent-manifest)  
12. Declarative agent schema 1.2 for Microsoft 365 Copilot, accessed December 13, 2025, [https://learn.microsoft.com/en-us/microsoft-365-copilot/extensibility/declarative-agent-manifest-1.2](https://learn.microsoft.com/en-us/microsoft-365-copilot/extensibility/declarative-agent-manifest-1.2)  
13. AGENTS.md, accessed December 13, 2025, [https://agents.md/](https://agents.md/)  
14. Ruler Config: Centralize Your AI Coding Assistant Instructions, accessed December 13, 2025, [https://medium.com/@devonsunml/ruler-config-centralize-your-ai-coding-assistant-instructions-960e4be5a95f](https://medium.com/@devonsunml/ruler-config-centralize-your-ai-coding-assistant-instructions-960e4be5a95f)  
15. TOML \- Docs by LangChain, accessed December 13, 2025, [https://docs.langchain.com/oss/python/integrations/document\_loaders/toml](https://docs.langchain.com/oss/python/integrations/document_loaders/toml)  
16. Rust SDK for the Model Context Protocol (MCP) \- GitHub, accessed December 13, 2025, [https://github.com/Derek-X-Wang/mcp-rust-sdk](https://github.com/Derek-X-Wang/mcp-rust-sdk)  
17. mcp-protocol-sdk \- crates.io: Rust Package Registry, accessed December 13, 2025, [https://crates.io/crates/mcp-protocol-sdk](https://crates.io/crates/mcp-protocol-sdk)  
18. Model Context Protocol (MCP) \- Docs by LangChain, accessed December 13, 2025, [https://docs.langchain.com/oss/python/langchain/mcp](https://docs.langchain.com/oss/python/langchain/mcp)  
19. a2a-types — Rust auth library // Lib.rs, accessed December 13, 2025, [https://lib.rs/crates/a2a-types](https://lib.rs/crates/a2a-types)  
20. rigs \- crates.io: Rust Package Registry, accessed December 13, 2025, [https://crates.io/crates/rigs](https://crates.io/crates/rigs)  
21. rig \- Rust \- Docs.rs, accessed December 13, 2025, [https://docs.rs/rig-core/latest/rig/](https://docs.rs/rig-core/latest/rig/)  
22. Building Scalable Disaster Recovery Platforms for Microservices, accessed December 13, 2025, [https://dzone.com/articles/building-scalable-disaster-recovery-platform-for-m](https://dzone.com/articles/building-scalable-disaster-recovery-platform-for-m)  
23. Temporal: Beyond State Machines for Reliable Distributed ..., accessed December 13, 2025, [https://temporal.io/blog/temporal-replaces-state-machines-for-distributed-applications](https://temporal.io/blog/temporal-replaces-state-machines-for-distributed-applications)  
24. langchain-ai/langgraph: Build resilient language agents as ... \- GitHub, accessed December 13, 2025, [https://github.com/langchain-ai/langgraph](https://github.com/langchain-ai/langgraph)  
25. Structured model outputs | OpenAI API, accessed December 13, 2025, [https://platform.openai.com/docs/guides/structured-outputs](https://platform.openai.com/docs/guides/structured-outputs)  
26. Ensuring JSON Response Format in OpenAI Assistant API \- Scorpil, accessed December 13, 2025, [https://scorpil.com/post/ensuring-json-response-format-in-openai-assistant-api/](https://scorpil.com/post/ensuring-json-response-format-in-openai-assistant-api/)  
27. Pydantic AI Tracing | Integrations | Arize Docs, accessed December 13, 2025, [https://arize.com/docs/ax/observe/tracing-integrations-auto/pydantic-ai](https://arize.com/docs/ax/observe/tracing-integrations-auto/pydantic-ai)