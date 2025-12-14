The Unified Cognitive Architecture: A Holistic Specification for Agentic Systems1. Introduction: The Crisis of Monoliths in Agentic DesignThe trajectory of Artificial Intelligence has shifted decisively from the development of isolated, stateless models toward the engineering of persistent, stateful, and autonomous agentic systems. This transition necessitates a fundamental re-evaluation of the specifications used to define, configure, and orchestrate these entities. Early specifications, often monolithic in nature, conflated the distinct concerns of cognitive processing, memory management, context curation, and workflow orchestration into singular, brittle configuration files. As agents evolve from simple chatbots into complex adaptive systems capable of long-horizon planning and recursive self-improvement, the architectural frameworks supporting them must mature in parallel.The prevailing inquiry within the field addresses a critical architectural dilemma: where do the emergent capabilities of declarative context engineering, bifurcated memory architectures, structured knowledge retrieval (GraphRAG), and compositional workflows reside within the agent stack? The historical tendency has been to append these features to the "Cognitive Layer" (often designated as Layer 5), effectively treating the reasoning engine as a "God Object" responsible for its own state, input hygiene, and execution logic. This report argues that such a consolidation is an anti-pattern that inhibits scalability, observability, and interoperability.We propose a radical restructuring of the agent specification, moving away from a flat definition toward a Recursive Component Architecture. In this paradigm, the monolithic "Layer 5" is deconstructed. We advocate for a vertical integration strategy where Memory becomes a persistent substrate (Layer 3), Context evolves into a dynamic interface (Layer 2), and Workflows function as recursive control structures (Layer 5). This holistic research report details the theoretical underpinnings, architectural layers, and implementation strategies required to achieve this vision, drawing upon the latest advancements in Graph Retrieval-Augmented Generation (GraphRAG), the Model Context Protocol (MCP), and stateful orchestration frameworks like LangGraph and Letta.2. Architectural Re-evaluation: Beyond the Cognitive MonolithThe initial conceptualization of an AI agent typically centers on the Large Language Model (LLM) as the primary locus of activity. In this view, "Layer 5" serves as the container for all reasoning, memory, and tool use. However, our analysis suggests that placing context engineering, memory systems, and workflow logic entirely within this cognitive layer leads to architectural brittleness. It fails to distinguish between the state of the system, the process of reasoning, and the preparation of inputs.2.1 The Limitations of Layer 5 ConsolidationConsolidating these distinct functional areas into a single cognitive layer creates significant friction in system design. Specifically, it conflates three orthogonal concerns:State vs. Process: Memory is a persistent resource that exists independently of the reasoning process. An agent's "User Memory" (containing preferences and historical interactions) typically has a lifecycle that exceeds the duration of any single inference task. Conversely, "Working Memory" is ephemeral and scoped to the specific thread of execution. Binding both inextricably to the cognitive layer complicates the implementation of persistence mechanisms like checkpointing and "time-travel" debugging.1Input vs. Reasoning: Context engineering is fundamentally a pre-cognitive filter—an input processing task designed to optimize the signal-to-noise ratio before the data ever reaches the LLM. By treating context management as a sub-function of the reasoning layer, we force the model to expend valuable tokens on filtering noise, rather than reasoning about signal.3Orchestration vs. Execution: Workflows represent the coordination of intelligence, not the intelligence itself. A complex workflow involving loops, conditionals, and parallel branches is structurally distinct from the atomic cognitive step of generating a response.2.2 The Proposed 6-Layer Recursive HierarchyTo address these limitations, we recommend a 6-Layer Recursive Architecture. This model distributes responsibilities across specialized layers, ensuring separation of concerns and enabling modular upgrades.LayerNamePrimary FunctionNew Specification ComponentLayer 1Interface LayerPerception, Action, and API Surfaces.interface definitionsLayer 2Context LayerDynamic Gating. Rolling windows, compression, and policy-driven input curation.context_policy (New)Layer 3Persistence LayerMemory & Knowledge. GraphRAG, Vector Stores, and User/Agent state bifurcation.memory, knowledge_bases (New)Layer 4Cognitive LayerReasoning. The LLM configuration, system prompts, and persona definitions.intelligence (Existing)Layer 5Orchestration LayerControl Flow. State machines, loops, branching, and sub-agent composition.workflow, behavior (Expanded)Layer 6Collaborative LayerNetwork Protocol. Inter-agent communication standards (DACP, Agent Protocol).protocol (New)2.3 The Concept of Fractal AgencyA pivotal aspect of this recommendation is the treatment of workflows. Rather than viewing a workflow as a linear script executed by an agent, the specification should define workflows as Recursive Agents. A complex workflow—comprising multiple sub-agents, loops, and conditional logic—must expose the same input/output interface as an atomic agent. This "Fractal Agency" allows for infinite composition: a high-level "CEO Agent" can invoke a "Research Department Agent," which is, in reality, a complex graph of sub-agents performing searches, summarization, and critique, unbeknownst to the caller.53. Layer 2: Context Engineering as a Deterministic ScienceContext Engineering has transitioned from the ad-hoc art of "prompt stuffing" to a rigorous architectural requirement. As agents operate over longer horizons and ingest vast quantities of data, the context window—despite recent expansions to 1M+ tokens—remains a finite and precious resource. It must be managed with the same discipline as Random Access Memory (RAM) in an operating system.3.1 Declarative Context PoliciesTo formalize this, the Complete Agent Specification (CAS) must include a context_policy section. This moves the logic of what to remember and what to forget out of the application code and into the declarative schema.3.1.1 Window Management and Eviction StrategiesThe specification must define explicit strategies for managing the sliding window of context. Research indicates that simple First-In-First-Out (FIFO) buffers are often insufficient for complex tasks, as they may discard critical initial instructions or long-term goals in favor of recent, trivial chatter.3We recommend supporting three distinct eviction policies within the spec:Rolling Window: The standard approach, retaining the last $N$ tokens.Selective Retention (Pinned Context): This allows specific message types (e.g., system_instructions, user_memory_block) to be exempted from eviction. The spec should allow tagging specific blocks as immutable, ensuring that core directives persist regardless of conversation length.7Semantic Prioritization: An advanced policy where context is retained based on vector similarity to the current active goal, rather than mere recency.3.1.2 Compression and SummarizationContext compression is vital for maintaining performance and reducing costs. The specification should allow the definition of Summarization Triggers. For example, a auto_summarize flag could trigger a background job when the context buffer reaches 80% capacity. This job uses a secondary, lightweight model (e.g., GPT-4o-mini) to condense the oldest message blocks into a concise "Memory Summary," which is then reinjected into the prompt.4Table 1: Context Management StrategiesStrategyMechanismUse CaseSpecification ImpactFIFO BufferTruncates oldest messages when limit is reached.Simple chatbots, short sessions.window_size: 128k, strategy: rollingSummarizationCondenses message history into narrative summaries.Long-running tasks, infinite sessions.compression: true, summary_model: gpt-4o-miniContext QuarantineIsolates sub-agent context to prevent pollution.Complex multi-agent swarms.isolation_level: strict 9Selective PunningEvicts tool outputs first, retains user inputs.Tool-heavy workflows.eviction_priority: [tool_outputs, assistant_msgs]3.2 Preventing Context PoisoningA critical insight from recent research is the danger of "Context Poisoning," where a hallucinated fact or an error in a tool output enters the context window and is subsequently treated as ground truth by the model.4 By elevating context engineering to a distinct layer, the specification can enforce Context Hygiene.The context_policy can define "Quarantine Zones." For instance, the output of a "Drafting Agent" might be kept in a temporary scratchpad and only promoted to the main context after it has passed a validation check by a "Critic Agent." This architectural pattern, enforced via the spec, prevents errors from propagating through the system state.4. Layer 3: The Knowledge & Memory SubstrateThe user's request to "spec out... graph and rag based storage and retrieval, and agent memory as well as user memory" touches upon the most significant deficiency in current agent frameworks: the lack of a unified persistence layer. We argue for a Bifurcated Memory Architecture, explicitly separating the agent's working state from the user's persistent profile.4.1 Agent Memory vs. User MemoryIt is imperative to distinguish between the memory required for the agent to function (Agent Memory) and the memory required for the agent to know the user (User Memory).4.1.1 User Memory: The Persistent ProfileUser Memory represents the Semantic and Episodic history of the user. It functions as the "User Model." This data must persist across sessions, across different agents, and potentially across different applications.Mechanism: We draw inspiration from Letta (formerly MemGPT), which utilizes "Memory Blocks"—dedicated segments of the system prompt that contain editable text fields (e.g., "Human Persona," "User Facts").10Specification: The spec should define a user_memory object. This object specifies the storage backend (e.g., letta_store) and the schema of the blocks (e.g., label: user_preferences, limit: 2000 chars).Insight: Unlike vector storage, which is probabilistic, Memory Blocks are deterministic. The agent always knows the user's name if it is in the Core Memory block. This provides a stability that RAG alone cannot achieve.124.1.2 Agent Memory: The Working StateAgent Memory is the Procedural and Working memory. It tracks the current state of the workflow: "What step am I on?", "What tools have I called?", "What is the intermediate result?"Mechanism: This is best implemented via Checkpointing, a concept popularized by LangGraph. A checkpointer saves the entire state of the execution graph (variable values, instruction pointer) to a database (e.g., Postgres, Redis) after every "super-step".1Specification: The spec should include an agent_state definition, outlining the schema of the state object (e.g., a Pydantic model or TypedDict) and the persistence policy (e.g., checkpoint_frequency: every_step).Benefit: This enables "Time Travel." If an agent hallucinates or crashes during a complex workflow, the developer can rewind the state to a previous checkpoint, modify the state, and resume execution. This is critical for debugging autonomous systems.24.2 GraphRAG: Structured Knowledge IntegrationStandard Retrieval-Augmented Generation (RAG) relies on vector similarity search. While effective for finding distinct facts, vector RAG struggles with "multi-hop reasoning" or "global summarization"—queries that require traversing relationships between entities (e.g., "How does the conflict in Region A affect the supply chain in Region B?").13To address this, the specification must integrate GraphRAG.4.2.1 The Graph Schema DefinitionThe agent spec should not just point to a database; it should define the Ontology of the knowledge it expects to interact with.Entity Extraction: The spec should define entity_types (e.g., Person, Organization, Event) and relationship_types (e.g., AFFILIATED_WITH, LOCATED_IN). This guides the LLM in extracting structured data from unstructured text.15Community Detection: Advanced GraphRAG implementations, such as Microsoft's, utilize hierarchical community detection (e.g., Leiden algorithm) to group related nodes. The spec should allow configuration of community_level summaries, enabling the agent to zoom in (local search) or zoom out (global search) depending on the query.144.2.2 Connectivity via MCPIntegrating GraphRAG requires a standardized connection protocol. The Model Context Protocol (MCP) serves as the ideal "USB-C for AI," providing a uniform interface for agents to connect to knowledge sources.18Implementation: The spec's knowledge_bases section should define resources as MCP endpoints. For example, a Neo4j database is exposed as an MCP server offering tools like read_graph_schema and execute_cypher_query.Insight: This decoupling means the agent specification remains agnostic to the underlying database technology. Whether the graph lives in Neo4j, Memgraph, or Oracle Database 23ai, the agent interacts with it through the standardized MCP tool interface.20Table 2: Vector RAG vs. GraphRAG in Agent SpecificationsFeatureVector RAGGraphRAGSpecification RequirementData StructureFlat list of text chunks.Interconnected nodes and edges.knowledge_base_type: graphRetrieval LogicCosine similarity (Semantic Match).Graph Traversal (Relationships).retrieval_policy: traversalContext QualityHigh precision for specific facts.High context for complex systems.depth: 2 (hops)Query Types"What does X say about Y?""How are X, Y, and Z related?"tools: [graph_query]5. Layer 5: Workflows as Recursive OrchestrationThe user asks: "Where would workflows fit in, if we were to consider workflows as a composition of tasks and agents... with a complex workflow itself being presented as an agent." This aligns perfectly with the Recursive Orchestration model located in Layer 5.5.1 Flows as State MachinesModern agentic workflows are best modeled not as linear chains (DAGs) but as Cyclic Graphs or State Machines. This allows for loops (retries, refinement), conditional branching (decision trees), and parallel execution (map-reduce).22The specification must adopt a schema capable of defining these topologies. We draw upon AgentML (SCXML-based) and LangGraph for structural inspiration.245.1.1 Core Flow ComponentsNodes: These represent units of work. A node can be a simple Task (calling a tool), a cognitive Agent (invoking an LLM), or—crucially—a nested SubFlow.Edges: These define the transitions. Edges can be conditional (checking the state to decide the next node) or direct.State: The workflow must possess a defined state_schema. This schema acts as the "bus" that carries data between nodes. All nodes read from and write to this shared state.265.2 The Fractal Agent PatternThe requirement that a "complex workflow itself be presented as an agent" implies a fractal design.Interface Parity: A Workflow definition in the spec must define inputs and outputs identical to a standard Agent definition.Encapsulation: To the caller, the internal complexity of the workflow is hidden. A "Master Orchestrator" might delegate a task to a "Coder Agent." In reality, the "Coder Agent" is a complex workflow involving a "Writer Node," a "Linter Node," and a "Test Node" looping until success.Implementation: The spec should support a type field in the node definition. type: agent invokes an LLM directly; type: flow invokes a nested graph definition. This supports infinite recursion, allowing for the construction of immensely complex systems from simple, composable blocks.56. Holistic Recommendation: The Complete Agent Specification (v2.0)Based on the synthesis of the research above, we present the Revised Open Agent Specification (v2.0) structure. This YAML-based blueprint satisfies all the user's requirements: context engineering, memory bifurcation, GraphRAG integration, and recursive workflows.6.1 The Unified Schema BlueprintYAMLspec_version: "2.0.0"
metadata:
  name: "Strategic_Research_Architect"
  version: "1.0"
  description: "A recursive agent that uses GraphRAG to analyze corporate data and updates user preferences."

# --- LAYER 1: INTERFACE ---
# Defines the input/output contract.
interface:
  inputs:
    query: 
      type: string
      description: "The research topic or question."
      validation: "max_length=1000"
  outputs:
    report:
      type: markdown
      description: "A comprehensive analysis report."

# --- LAYER 2: CONTEXT POLICY (New Requirement) ---
# Defines how the 'Prompt Window' is managed.
context_policy:
  window_management:
    strategy: "rolling_window"
    max_tokens: 128000
    retention_priority: "semantic_relevance"
  
  # Compression Logic [8]
  compression:
    enabled: true
    trigger_threshold: 0.85 # Compress when 85% full
    method: "recursive_summary"
    model_ref: "gpt-4o-mini"
  
  # Immutable Context 
  pinned_context:
    - type: "system_instructions"
    - type: "user_memory_block" # Injects the Letta memory block
    - type: "current_task_state"

# --- LAYER 3: MEMORY & KNOWLEDGE (New Requirement) ---
storage:
  # 1. User Memory (Persistent, Letta-style) [10]
  user_profile:
    backend: "letta_store"
    blocks:
      - label: "user_facts"
        limit: 2000 chars
        description: "Biographical facts and preferences."
      - label: "persona"
        limit: 1000 chars
        description: "How the user wants the agent to behave."
    auto_update: true # Agent can self-edit these blocks via tools

  # 2. Agent Memory (Working State, LangGraph-style) [1]
  working_state:
    backend: "langgraph_checkpoint"
    persistence_scope: "thread"
    storage_provider: "postgres"

  # 3. Knowledge Base (GraphRAG via MCP) [15, 20]
  knowledge_bases:
    - id: "corp_graph"
      type: "graph_rag"
      access_protocol:
        type: "mcp"
        server_url: "http://neo4j-mcp:8080"
      
      # Schema-Guided Extraction [16]
      ontology:
        entities:
        relationships:
      
      retrieval_policy:
        method: "hybrid" # Graph traversal + Vector search
        depth: 2         # Hop depth for context gathering

# --- LAYER 4: COGNITIVE CORE ---
intelligence:
  model_provider: "openai"
  model_name: "gpt-4-turbo"
  temperature: 0.2
  system_prompt: "prompts/architect_system.md"

# --- LAYER 5: WORKFLOW / BEHAVIOR (New Requirement) ---
# Defines the recursive orchestration logic.
behavior:
  type: "flow" # Indicates this is a composite agent (Workflow)
  
  # Shared State Schema 
  state_schema:
    keys: ["query", "graph_data", "draft", "critique_score"]

  nodes:
    # Node 1: Parallel Execution (Scatter-Gather)
    - id: "research_phase"
      type: "parallel"
      branches:
        - id: "vector_search"
          task: "search_docs"
        - id: "graph_traversal"
          task: "query_relationships" # Uses GraphRAG tool
      
    # Node 2: Sub-Agent Invocation (Recursive)
    - id: "drafting_agent"
      type: "agent_ref"
      source: "./agents/writer_agent.yaml" # Calls another full agent spec
      inputs:
        context: "{state.graph_data}"
      
    # Node 3: Self-Correction Loop [23]
    - id: "quality_check"
      type: "llm_call"
      prompt: "Rate the draft on a scale of 1-10."
      output_key: "critique_score"

  # Control Flow (Edges)
  edges:
    - from: "START"           to: "research_phase"
    - from: "research_phase"  to: "drafting_agent"
    - from: "drafting_agent"  to: "quality_check"
    
    # Conditional Logic
    - from: "quality_check"   to: "END"
      condition: "state.critique_score >= 8"
    - from: "quality_check"   to: "drafting_agent" # Loop back for retry
      condition: "state.critique_score < 8"

# --- LAYER 6: COLLABORATION ---
protocol:
  standard: "dacp" # Declarative Agent Communication Protocol
  role: "specialist"
6.2 Implementation GuidelinesTo realize this specification, a "Reference Runtime" must be constructed that aggregates the capabilities of current best-in-class frameworks. No single existing tool supports this entire spec out-of-the-box, but they can be integrated via adapters.Workflow Engine (LangGraph): The behavior section should be compiled into a LangGraph StateGraph. LangGraph natively supports the cyclic graph topology, conditional edges, and parallel execution defined in the spec. Its checkpointing system directly satisfies the working_state requirement in Layer 3.26Memory Manager (Letta): The user_profile and context_policy sections should be managed by a Letta (MemGPT) instance running as a sidecar. The runtime should query Letta to construct the optimized system_prompt (containing the relevant memory blocks) before passing control to the workflow engine.11Knowledge Connector (MCP): The knowledge_bases section should be implemented using MCP Clients. The runtime establishes connections to MCP Servers (e.g., for Neo4j or Qdrant) to expose the graph_rag tools to the agent. This ensures that the agent is decoupled from the specific database drivers.20Orchestrator (DACP): For multi-agent systems where this agent must collaborate with others, the protocol layer should implement DACP interfaces, allowing the agent to broadcast its capabilities and accept tasks from a mesh network.67. ConclusionThe evolution of agentic AI demands a specification that is as rigorous as it is flexible. By rejecting the monolithic "Layer 5" model and adopting a 6-Layer Recursive Architecture, we create a framework where Context is engineered, Memory is bifurcated and persistent, Knowledge is structured and traversable, and Workflows are fractal.This holistic specification transforms the agent from a transient, stateless query processor into a persistent, evolving digital worker. It bridges the gap between the "stateless" world of LLMs and the "stateful" world of enterprise applications, providing a robust foundation for the next generation of autonomous systems. The integration of GraphRAG ensures these agents are grounded in complex realities, while the recursive workflow definition ensures they can scale to meet any procedural challenge. This is not merely a configuration file; it is the blueprint for a cognitive operating system.