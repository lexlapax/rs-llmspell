# **The Unified Cognitive Architecture: A Holistic Specification for Agentic Systems**

## **1\. Introduction: The Crisis of Monoliths in Agentic Design**

The trajectory of Artificial Intelligence has shifted decisively from the development of isolated, stateless models toward the engineering of persistent, stateful, and autonomous agentic systems. This transition necessitates a fundamental re-evaluation of the specifications used to define, configure, and orchestrate these entities. Early specifications, often monolithic in nature, conflated the distinct concerns of cognitive processing, memory management, context curation, and workflow orchestration into singular, brittle configuration files. As agents evolve from simple chatbots into complex adaptive systems capable of long-horizon planning and recursive self-improvement, the architectural frameworks supporting them must mature in parallel.

The prevailing inquiry within the field addresses a critical architectural dilemma: where do the emergent capabilities of declarative context engineering, bifurcated memory architectures, structured knowledge retrieval (GraphRAG), and compositional workflows reside within the agent stack? The historical tendency has been to append these features to the "Cognitive Layer" (often designated as Layer 5), effectively treating the reasoning engine as a "God Object" responsible for its own state, input hygiene, and execution logic. This report argues that such a consolidation is an anti-pattern that inhibits scalability, observability, and interoperability.

We propose a radical restructuring of the agent specification, moving away from a flat definition toward a **Recursive Component Architecture**. In this paradigm, the monolithic "Layer 5" is deconstructed. We advocate for a vertical integration strategy where **Memory** becomes a persistent substrate (Layer 3), **Context** evolves into a dynamic interface (Layer 2), and **Workflows** function as recursive control structures (Layer 5). This holistic research report details the theoretical underpinnings, architectural layers, and implementation strategies required to achieve this vision, drawing upon the latest advancements in Graph Retrieval-Augmented Generation (GraphRAG), the Model Context Protocol (MCP), and stateful orchestration frameworks like LangGraph and Letta.

## ---

**2\. Architectural Re-evaluation: Beyond the Cognitive Monolith**

The initial conceptualization of an AI agent typically centers on the Large Language Model (LLM) as the primary locus of activity. In this view, "Layer 5" serves as the container for all reasoning, memory, and tool use. However, our analysis suggests that placing context engineering, memory systems, and workflow logic entirely within this cognitive layer leads to architectural brittleness. It fails to distinguish between the *state* of the system, the *process* of reasoning, and the *preparation* of inputs.

### **2.1 The Limitations of Layer 5 Consolidation**

Consolidating these distinct functional areas into a single cognitive layer creates significant friction in system design. Specifically, it conflates three orthogonal concerns:

1. **State vs. Process:** Memory is a persistent resource that exists independently of the reasoning process. An agent's "User Memory" (containing preferences and historical interactions) typically has a lifecycle that exceeds the duration of any single inference task. Conversely, "Working Memory" is ephemeral and scoped to the specific thread of execution. Binding both inextricably to the cognitive layer complicates the implementation of persistence mechanisms like checkpointing and "time-travel" debugging.1  
2. **Input vs. Reasoning:** Context engineering is fundamentally a pre-cognitive filter—an input processing task designed to optimize the signal-to-noise ratio before the data ever reaches the LLM. By treating context management as a sub-function of the reasoning layer, we force the model to expend valuable tokens on filtering noise, rather than reasoning about signal.3  
3. **Orchestration vs. Execution:** Workflows represent the *coordination* of intelligence, not the intelligence itself. A complex workflow involving loops, conditionals, and parallel branches is structurally distinct from the atomic cognitive step of generating a response.

### **2.2 The Proposed 6-Layer Recursive Hierarchy**

To address these limitations, we recommend a **6-Layer Recursive Architecture**. This model distributes responsibilities across specialized layers, ensuring separation of concerns and enabling modular upgrades.

| Layer | Name | Primary Function | New Specification Component |
| :---- | :---- | :---- | :---- |
| **Layer 1** | **Interface Layer** | Perception, Action, and API Surfaces. | interface definitions |
| **Layer 2** | **Context Layer** | **Dynamic Gating.** Rolling windows, compression, and policy-driven input curation. | context\_policy (New) |
| **Layer 3** | **Persistence Layer** | **Memory & Knowledge.** GraphRAG, Vector Stores, and User/Agent state bifurcation. | memory, knowledge\_bases (New) |
| **Layer 4** | **Cognitive Layer** | **Reasoning.** The LLM configuration, system prompts, and persona definitions. | intelligence (Existing) |
| **Layer 5** | **Orchestration Layer** | **Control Flow.** State machines, loops, branching, and sub-agent composition. | workflow, behavior (Expanded) |
| **Layer 6** | **Collaborative Layer** | **Network Protocol.** Inter-agent communication standards (DACP, Agent Protocol). | protocol (New) |

### **2.3 The Concept of Fractal Agency**

A pivotal aspect of this recommendation is the treatment of workflows. Rather than viewing a workflow as a linear script executed *by* an agent, the specification should define workflows as **Recursive Agents**. A complex workflow—comprising multiple sub-agents, loops, and conditional logic—must expose the same input/output interface as an atomic agent. This "Fractal Agency" allows for infinite composition: a high-level "CEO Agent" can invoke a "Research Department Agent," which is, in reality, a complex graph of sub-agents performing searches, summarization, and critique, unbeknownst to the caller.5

## ---

**3\. Layer 2: Context Engineering as a Deterministic Science**

Context Engineering has transitioned from the ad-hoc art of "prompt stuffing" to a rigorous architectural requirement. As agents operate over longer horizons and ingest vast quantities of data, the context window—despite recent expansions to 1M+ tokens—remains a finite and precious resource. It must be managed with the same discipline as Random Access Memory (RAM) in an operating system.

### **3.1 Declarative Context Policies**

To formalize this, the Complete Agent Specification (CAS) must include a context\_policy section. This moves the logic of what to remember and what to forget out of the application code and into the declarative schema.

#### **3.1.1 Window Management and Eviction Strategies**

The specification must define explicit strategies for managing the sliding window of context. Research indicates that simple First-In-First-Out (FIFO) buffers are often insufficient for complex tasks, as they may discard critical initial instructions or long-term goals in favor of recent, trivial chatter.3

We recommend supporting three distinct eviction policies within the spec:

1. **Rolling Window:** The standard approach, retaining the last $N$ tokens.  
2. **Selective Retention (Pinned Context):** This allows specific message types (e.g., system\_instructions, user\_memory\_block) to be exempted from eviction. The spec should allow tagging specific blocks as immutable, ensuring that core directives persist regardless of conversation length.7  
3. **Semantic Prioritization:** An advanced policy where context is retained based on vector similarity to the current active goal, rather than mere recency.

#### **3.1.2 Compression and Summarization**

Context compression is vital for maintaining performance and reducing costs. The specification should allow the definition of **Summarization Triggers**. For example, a auto\_summarize flag could trigger a background job when the context buffer reaches 80% capacity. This job uses a secondary, lightweight model (e.g., GPT-4o-mini) to condense the oldest message blocks into a concise "Memory Summary," which is then reinjected into the prompt.4

**Table 1: Context Management Strategies**

| Strategy | Mechanism | Use Case | Specification Impact |
| :---- | :---- | :---- | :---- |
| **FIFO Buffer** | Truncates oldest messages when limit is reached. | Simple chatbots, short sessions. | window\_size: 128k, strategy: rolling |
| **Summarization** | Condenses message history into narrative summaries. | Long-running tasks, infinite sessions. | compression: true, summary\_model: gpt-4o-mini |
| **Context Quarantine** | Isolates sub-agent context to prevent pollution. | Complex multi-agent swarms. | isolation\_level: strict 9 |
| **Selective Punning** | Evicts tool outputs first, retains user inputs. | Tool-heavy workflows. | eviction\_priority: \[tool\_outputs, assistant\_msgs\] |

### **3.2 Preventing Context Poisoning**

A critical insight from recent research is the danger of "Context Poisoning," where a hallucinated fact or an error in a tool output enters the context window and is subsequently treated as ground truth by the model.4 By elevating context engineering to a distinct layer, the specification can enforce **Context Hygiene**.

The context\_policy can define "Quarantine Zones." For instance, the output of a "Drafting Agent" might be kept in a temporary scratchpad and only promoted to the main context *after* it has passed a validation check by a "Critic Agent." This architectural pattern, enforced via the spec, prevents errors from propagating through the system state.

## ---

**4\. Layer 3: The Knowledge & Memory Substrate**

The user's request to "spec out... graph and rag based storage and retrieval, and agent memory as well as user memory" touches upon the most significant deficiency in current agent frameworks: the lack of a unified persistence layer. We argue for a **Bifurcated Memory Architecture**, explicitly separating the agent's working state from the user's persistent profile.

### **4.1 Agent Memory vs. User Memory**

It is imperative to distinguish between the memory required for the agent to function (Agent Memory) and the memory required for the agent to know the user (User Memory).

#### **4.1.1 User Memory: The Persistent Profile**

User Memory represents the **Semantic and Episodic** history of the user. It functions as the "User Model." This data must persist across sessions, across different agents, and potentially across different applications.

* **Mechanism:** We draw inspiration from **Letta (formerly MemGPT)**, which utilizes "Memory Blocks"—dedicated segments of the system prompt that contain editable text fields (e.g., "Human Persona," "User Facts").10  
* **Specification:** The spec should define a user\_memory object. This object specifies the storage backend (e.g., letta\_store) and the schema of the blocks (e.g., label: user\_preferences, limit: 2000 chars).  
* **Insight:** Unlike vector storage, which is probabilistic, Memory Blocks are deterministic. The agent *always* knows the user's name if it is in the Core Memory block. This provides a stability that RAG alone cannot achieve.12

#### **4.1.2 Agent Memory: The Working State**

Agent Memory is the **Procedural and Working** memory. It tracks the current state of the workflow: "What step am I on?", "What tools have I called?", "What is the intermediate result?"

* **Mechanism:** This is best implemented via **Checkpointing**, a concept popularized by LangGraph. A checkpointer saves the entire state of the execution graph (variable values, instruction pointer) to a database (e.g., Postgres, Redis) after every "super-step".1  
* **Specification:** The spec should include an agent\_state definition, outlining the schema of the state object (e.g., a Pydantic model or TypedDict) and the persistence policy (e.g., checkpoint\_frequency: every\_step).  
* **Benefit:** This enables "Time Travel." If an agent hallucinates or crashes during a complex workflow, the developer can rewind the state to a previous checkpoint, modify the state, and resume execution. This is critical for debugging autonomous systems.2

### **4.2 GraphRAG: Structured Knowledge Integration**

Standard Retrieval-Augmented Generation (RAG) relies on vector similarity search. While effective for finding distinct facts, vector RAG struggles with "multi-hop reasoning" or "global summarization"—queries that require traversing relationships between entities (e.g., "How does the conflict in Region A affect the supply chain in Region B?").13

To address this, the specification must integrate **GraphRAG**.

#### **4.2.1 The Graph Schema Definition**

The agent spec should not just point to a database; it should define the **Ontology** of the knowledge it expects to interact with.

* **Entity Extraction:** The spec should define entity\_types (e.g., Person, Organization, Event) and relationship\_types (e.g., AFFILIATED\_WITH, LOCATED\_IN). This guides the LLM in extracting structured data from unstructured text.15  
* **Community Detection:** Advanced GraphRAG implementations, such as Microsoft's, utilize hierarchical community detection (e.g., Leiden algorithm) to group related nodes. The spec should allow configuration of community\_level summaries, enabling the agent to zoom in (local search) or zoom out (global search) depending on the query.14

#### **4.2.2 Connectivity via MCP**

Integrating GraphRAG requires a standardized connection protocol. The **Model Context Protocol (MCP)** serves as the ideal "USB-C for AI," providing a uniform interface for agents to connect to knowledge sources.18

* **Implementation:** The spec's knowledge\_bases section should define resources as MCP endpoints. For example, a Neo4j database is exposed as an MCP server offering tools like read\_graph\_schema and execute\_cypher\_query.  
* **Insight:** This decoupling means the agent specification remains agnostic to the underlying database technology. Whether the graph lives in Neo4j, Memgraph, or Oracle Database 23ai, the agent interacts with it through the standardized MCP tool interface.20

**Table 2: Vector RAG vs. GraphRAG in Agent Specifications**

| Feature | Vector RAG | GraphRAG | Specification Requirement |
| :---- | :---- | :---- | :---- |
| **Data Structure** | Flat list of text chunks. | Interconnected nodes and edges. | knowledge\_base\_type: graph |
| **Retrieval Logic** | Cosine similarity (Semantic Match). | Graph Traversal (Relationships). | retrieval\_policy: traversal |
| **Context Quality** | High precision for specific facts. | High context for complex systems. | depth: 2 (hops) |
| **Query Types** | "What does X say about Y?" | "How are X, Y, and Z related?" | tools: \[graph\_query\] |

## ---

**5\. Layer 5: Workflows as Recursive Orchestration**

The user asks: "Where would workflows fit in, if we were to consider workflows as a composition of tasks and agents... with a complex workflow itself being presented as an agent." This aligns perfectly with the **Recursive Orchestration** model located in Layer 5\.

### **5.1 Flows as State Machines**

Modern agentic workflows are best modeled not as linear chains (DAGs) but as **Cyclic Graphs** or State Machines. This allows for loops (retries, refinement), conditional branching (decision trees), and parallel execution (map-reduce).22

The specification must adopt a schema capable of defining these topologies. We draw upon **AgentML** (SCXML-based) and **LangGraph** for structural inspiration.24

#### **5.1.1 Core Flow Components**

* **Nodes:** These represent units of work. A node can be a simple Task (calling a tool), a cognitive Agent (invoking an LLM), or—crucially—a nested SubFlow.  
* **Edges:** These define the transitions. Edges can be conditional (checking the state to decide the next node) or direct.  
* **State:** The workflow must possess a defined state\_schema. This schema acts as the "bus" that carries data between nodes. All nodes read from and write to this shared state.26

### **5.2 The Fractal Agent Pattern**

The requirement that a "complex workflow itself be presented as an agent" implies a fractal design.

* **Interface Parity:** A Workflow definition in the spec must define inputs and outputs identical to a standard Agent definition.  
* **Encapsulation:** To the caller, the internal complexity of the workflow is hidden. A "Master Orchestrator" might delegate a task to a "Coder Agent." In reality, the "Coder Agent" is a complex workflow involving a "Writer Node," a "Linter Node," and a "Test Node" looping until success.  
* **Implementation:** The spec should support a type field in the node definition. type: agent invokes an LLM directly; type: flow invokes a nested graph definition. This supports infinite recursion, allowing for the construction of immensely complex systems from simple, composable blocks.5

## ---

**6\. Holistic Recommendation: The Complete Agent Specification (v2.0)**

Based on the synthesis of the research above, we present the **Revised Open Agent Specification (v2.0)** structure. This YAML-based blueprint satisfies all the user's requirements: context engineering, memory bifurcation, GraphRAG integration, and recursive workflows.

### **6.1 The Unified Schema Blueprint**

YAML

spec\_version: "2.0.0"  
metadata:  
  name: "Strategic\_Research\_Architect"  
  version: "1.0"  
  description: "A recursive agent that uses GraphRAG to analyze corporate data and updates user preferences."

\# \--- LAYER 1: INTERFACE \---  
\# Defines the input/output contract.  
interface:  
  inputs:  
    query:   
      type: string  
      description: "The research topic or question."  
      validation: "max\_length=1000"  
  outputs:  
    report:  
      type: markdown  
      description: "A comprehensive analysis report."

\# \--- LAYER 2: CONTEXT POLICY (New Requirement) \---  
\# Defines how the 'Prompt Window' is managed.  
context\_policy:  
  window\_management:  
    strategy: "rolling\_window"  
    max\_tokens: 128000  
    retention\_priority: "semantic\_relevance"  
    
  \# Compression Logic \[8\]  
  compression:  
    enabled: true  
    trigger\_threshold: 0.85 \# Compress when 85% full  
    method: "recursive\_summary"  
    model\_ref: "gpt-4o-mini"  
    
  \# Immutable Context   
  pinned\_context:  
    \- type: "system\_instructions"  
    \- type: "user\_memory\_block" \# Injects the Letta memory block  
    \- type: "current\_task\_state"

\# \--- LAYER 3: MEMORY & KNOWLEDGE (New Requirement) \---  
storage:  
  \# 1\. User Memory (Persistent, Letta-style) \[10\]  
  user\_profile:  
    backend: "letta\_store"  
    blocks:  
      \- label: "user\_facts"  
        limit: 2000 chars  
        description: "Biographical facts and preferences."  
      \- label: "persona"  
        limit: 1000 chars  
        description: "How the user wants the agent to behave."  
    auto\_update: true \# Agent can self-edit these blocks via tools

  \# 2\. Agent Memory (Working State, LangGraph-style) \[1\]  
  working\_state:  
    backend: "langgraph\_checkpoint"  
    persistence\_scope: "thread"  
    storage\_provider: "postgres"

  \# 3\. Knowledge Base (GraphRAG via MCP) \[15, 20\]  
  knowledge\_bases:  
    \- id: "corp\_graph"  
      type: "graph\_rag"  
      access\_protocol:  
        type: "mcp"  
        server\_url: "http://neo4j-mcp:8080"  
        
      \# Schema-Guided Extraction \[16\]  
      ontology:  
        entities:  
        relationships:  
        
      retrieval\_policy:  
        method: "hybrid" \# Graph traversal \+ Vector search  
        depth: 2         \# Hop depth for context gathering

\# \--- LAYER 4: COGNITIVE CORE \---  
intelligence:  
  model\_provider: "openai"  
  model\_name: "gpt-4-turbo"  
  temperature: 0.2  
  system\_prompt: "prompts/architect\_system.md"

\# \--- LAYER 5: WORKFLOW / BEHAVIOR (New Requirement) \---  
\# Defines the recursive orchestration logic.  
behavior:  
  type: "flow" \# Indicates this is a composite agent (Workflow)  
    
  \# Shared State Schema   
  state\_schema:  
    keys: \["query", "graph\_data", "draft", "critique\_score"\]

  nodes:  
    \# Node 1: Parallel Execution (Scatter-Gather)  
    \- id: "research\_phase"  
      type: "parallel"  
      branches:  
        \- id: "vector\_search"  
          task: "search\_docs"  
        \- id: "graph\_traversal"  
          task: "query\_relationships" \# Uses GraphRAG tool  
        
    \# Node 2: Sub-Agent Invocation (Recursive)  
    \- id: "drafting\_agent"  
      type: "agent\_ref"  
      source: "./agents/writer\_agent.yaml" \# Calls another full agent spec  
      inputs:  
        context: "{state.graph\_data}"  
        
    \# Node 3: Self-Correction Loop \[23\]  
    \- id: "quality\_check"  
      type: "llm\_call"  
      prompt: "Rate the draft on a scale of 1-10."  
      output\_key: "critique\_score"

  \# Control Flow (Edges)  
  edges:  
    \- from: "START"           to: "research\_phase"  
    \- from: "research\_phase"  to: "drafting\_agent"  
    \- from: "drafting\_agent"  to: "quality\_check"  
      
    \# Conditional Logic  
    \- from: "quality\_check"   to: "END"  
      condition: "state.critique\_score \>= 8"  
    \- from: "quality\_check"   to: "drafting\_agent" \# Loop back for retry  
      condition: "state.critique\_score \< 8"

\# \--- LAYER 6: COLLABORATION \---  
protocol:  
  standard: "dacp" \# Declarative Agent Communication Protocol  
  role: "specialist"

### **6.2 Implementation Guidelines**

To realize this specification, a "Reference Runtime" must be constructed that aggregates the capabilities of current best-in-class frameworks. No single existing tool supports this entire spec out-of-the-box, but they can be integrated via adapters.

1. **Workflow Engine (LangGraph):** The behavior section should be compiled into a LangGraph StateGraph. LangGraph natively supports the cyclic graph topology, conditional edges, and parallel execution defined in the spec. Its checkpointing system directly satisfies the working\_state requirement in Layer 3\.26  
2. **Memory Manager (Letta):** The user\_profile and context\_policy sections should be managed by a Letta (MemGPT) instance running as a sidecar. The runtime should query Letta to construct the optimized system\_prompt (containing the relevant memory blocks) before passing control to the workflow engine.11  
3. **Knowledge Connector (MCP):** The knowledge\_bases section should be implemented using MCP Clients. The runtime establishes connections to MCP Servers (e.g., for Neo4j or Qdrant) to expose the graph\_rag tools to the agent. This ensures that the agent is decoupled from the specific database drivers.20  
4. **Orchestrator (DACP):** For multi-agent systems where this agent must collaborate with others, the protocol layer should implement DACP interfaces, allowing the agent to broadcast its capabilities and accept tasks from a mesh network.6

## ---

**7\. Conclusion**

The evolution of agentic AI demands a specification that is as rigorous as it is flexible. By rejecting the monolithic "Layer 5" model and adopting a **6-Layer Recursive Architecture**, we create a framework where **Context** is engineered, **Memory** is bifurcated and persistent, **Knowledge** is structured and traversable, and **Workflows** are fractal.

This holistic specification transforms the agent from a transient, stateless query processor into a persistent, evolving digital worker. It bridges the gap between the "stateless" world of LLMs and the "stateful" world of enterprise applications, providing a robust foundation for the next generation of autonomous systems. The integration of GraphRAG ensures these agents are grounded in complex realities, while the recursive workflow definition ensures they can scale to meet any procedural challenge. This is not merely a configuration file; it is the blueprint for a cognitive operating system.

#### **Works cited**

1. LangGraph Tutorial: Building LLM Agents with LangChain's ... \- Zep, accessed December 13, 2025, [https://www.getzep.com/ai-agents/langgraph-tutorial/](https://www.getzep.com/ai-agents/langgraph-tutorial/)  
2. Building Scalable LangChain Agents with Neo4j GraphQL ... \- Medium, accessed December 13, 2025, [https://medium.com/@bauerflo77/building-scalable-langchain-agents-with-neo4j-graphql-checkpointers-6ddd7e05649a](https://medium.com/@bauerflo77/building-scalable-langchain-agents-with-neo4j-graphql-checkpointers-6ddd7e05649a)  
3. Effective context engineering for AI agents \- Anthropic, accessed December 13, 2025, [https://www.anthropic.com/engineering/effective-context-engineering-for-ai-agents](https://www.anthropic.com/engineering/effective-context-engineering-for-ai-agents)  
4. Context Engineering \- LangChain Blog, accessed December 13, 2025, [https://blog.langchain.com/context-engineering-for-agents/](https://blog.langchain.com/context-engineering-for-agents/)  
5. Multi-agent patterns in LlamaIndex, accessed December 13, 2025, [https://developers.llamaindex.ai/python/framework/understanding/agent/multi\_agent/](https://developers.llamaindex.ai/python/framework/understanding/agent/multi_agent/)  
6. DACP: Declarative Agent Communication Protocol \- Medium, accessed December 13, 2025, [https://medium.com/@andrewswhitehouse/dacp-declarative-agent-communication-protocol-4ce579ec4407](https://medium.com/@andrewswhitehouse/dacp-declarative-agent-communication-protocol-4ce579ec4407)  
7. Context Engineering for Lens Studio | Snap for Developers, accessed December 13, 2025, [https://developers.snap.com/lens-studio/features/lens-studio-ai/developer-mode-context-engineering](https://developers.snap.com/lens-studio/features/lens-studio-ai/developer-mode-context-engineering)  
8. Optimizing the context abilities of LLMs by using a rolling summary, accessed December 13, 2025, [https://www.reddit.com/r/LocalLLaMA/comments/151rxol/optimizing\_the\_context\_abilities\_of\_llms\_by\_using/](https://www.reddit.com/r/LocalLLaMA/comments/151rxol/optimizing_the_context_abilities_of_llms_by_using/)  
9. How to Master Context Engineering | Digital Bricks, accessed December 13, 2025, [https://www.digitalbricks.ai/blog-posts/how-to-master-context-engineering](https://www.digitalbricks.ai/blog-posts/how-to-master-context-engineering)  
10. letta-ai/letta: Letta is the platform for building stateful agents ... \- GitHub, accessed December 13, 2025, [https://github.com/cpacker/MemGPT](https://github.com/cpacker/MemGPT)  
11. How Letta builds production-ready AI agents with Amazon Aurora ..., accessed December 13, 2025, [https://aws.amazon.com/blogs/database/how-letta-builds-production-ready-ai-agents-with-amazon-aurora-postgresql/](https://aws.amazon.com/blogs/database/how-letta-builds-production-ready-ai-agents-with-amazon-aurora-postgresql/)  
12. Towards AGI: \[Part 1\] Agents with Memory \- DEV Community, accessed December 13, 2025, [https://dev.to/akkiprime/towards-agi-part-1-agents-with-memory-4cp5](https://dev.to/akkiprime/towards-agi-part-1-agents-with-memory-4cp5)  
13. Beyond Vector RAG: How Document GraphRAG and Oracle 26ai ..., accessed December 13, 2025, [https://medium.com/@DatabaseDoug/beyond-vector-rag-how-document-graphrag-and-oracle-26ai-are-unlocking-business-knowledge-c1dbaf1381b5](https://medium.com/@DatabaseDoug/beyond-vector-rag-how-document-graphrag-and-oracle-26ai-are-unlocking-business-knowledge-c1dbaf1381b5)  
14. The Future of AI: GraphRAG – A better way to query interlinked ..., accessed December 13, 2025, [https://techcommunity.microsoft.com/blog/azure-ai-foundry-blog/the-future-of-ai-graphrag-%E2%80%93-a-better-way-to-query-interlinked-documents/4287182](https://techcommunity.microsoft.com/blog/azure-ai-foundry-blog/the-future-of-ai-graphrag-%E2%80%93-a-better-way-to-query-interlinked-documents/4287182)  
15. Custom Graphs \- GraphRAG \- Microsoft Open Source, accessed December 13, 2025, [https://microsoft.github.io/graphrag/index/byog/](https://microsoft.github.io/graphrag/index/byog/)  
16. What's New in the Neo4j GraphRAG Package for Python, accessed December 13, 2025, [https://neo4j.com/blog/developer/unleashing-the-power-of-schema/](https://neo4j.com/blog/developer/unleashing-the-power-of-schema/)  
17. How GraphRAG Works Step-By-Step \- Towards AI, accessed December 13, 2025, [https://pub.towardsai.net/how-microsofts-graphrag-works-step-by-step-b15cada5c209](https://pub.towardsai.net/how-microsofts-graphrag-works-step-by-step-b15cada5c209)  
18. What is Model Context Protocol (MCP)? A guide \- Google Cloud, accessed December 13, 2025, [https://cloud.google.com/discover/what-is-model-context-protocol](https://cloud.google.com/discover/what-is-model-context-protocol)  
19. What is Model Context Protocol (MCP)? \- IBM, accessed December 13, 2025, [https://www.ibm.com/think/topics/model-context-protocol](https://www.ibm.com/think/topics/model-context-protocol)  
20. A No-Code MCP Approach \- Graph Database & Analytics \- Neo4j, accessed December 13, 2025, [https://neo4j.com/blog/developer/knowledge-graphs-claude-neo4j-mcp/](https://neo4j.com/blog/developer/knowledge-graphs-claude-neo4j-mcp/)  
21. Knowledge Graph Memory Server | MCP Servers \- LobeHub, accessed December 13, 2025, [https://lobehub.com/mcp/deanacus-knowledge-graph-mcp](https://lobehub.com/mcp/deanacus-knowledge-graph-mcp)  
22. Tutorial: Building a Flow-based Ops Assistant for Incident ... \- Medium, accessed December 13, 2025, [https://medium.com/oracledevs/tutorial-building-a-flow-based-ops-assistant-for-incident-investigation-in-open-agent-spec-and-d44664d0da0a](https://medium.com/oracledevs/tutorial-building-a-flow-based-ops-assistant-for-incident-investigation-in-open-agent-spec-and-d44664d0da0a)  
23. Building Intelligent Agentic Workflows with LangGraph: A Human ..., accessed December 13, 2025, [https://medium.com/@sabita2025/building-intelligent-agentic-workflows-with-langgraph-a-human-friendly-guide-3dd295465092](https://medium.com/@sabita2025/building-intelligent-agentic-workflows-with-langgraph-a-human-friendly-guide-3dd295465092)  
24. agentflare-ai/agentml: Agent Markdown Language \- GitHub, accessed December 13, 2025, [https://github.com/agentflare-ai/agentml](https://github.com/agentflare-ai/agentml)  
25. Graph API overview \- Docs by LangChain, accessed December 13, 2025, [https://docs.langchain.com/oss/python/langgraph/graph-api](https://docs.langchain.com/oss/python/langgraph/graph-api)  
26. LangChain Vs LangGraph: Best, Definitive 2025 Agents Guide, accessed December 13, 2025, [https://binaryverseai.com/langchain-vs-langgraph-decision-guide-framework/](https://binaryverseai.com/langchain-vs-langgraph-decision-guide-framework/)