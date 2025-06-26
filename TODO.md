# Rs-LLMSpell Architecture Refinement TODO

## Overview
Comprehensive refinement of rs-llmspell architecture based on go-llms and Google ADK patterns, focusing on proper agent/tool/workflow hierarchy, hooks/events system, and built-in components.

## Multi-Step Strategy

### Phase 1: Research Foundation (🔍 Research)
- [x] **Task 1.1**: Deep study of go-llms architecture
- [x] **Task 1.2**: Study Google Agent Development Kit (ADK) 
- [x] **Task 1.3**: **CRITICAL** Research state management and agent handoff patterns
- [x] **Task 1.4**: Research existing Rust patterns for similar systems
### Phase 2: Analyze Current State (🔬 Analyze)
- [x] **Task 2.1**: Map current rs-llmspell concepts to go-llms/ADK
- [x] **Task 2.2**: Analyze scripting interface implications
### Phase 3: Synthesize Core Architecture (⚡ Synthesize)
- [x] **Task 3.1**: Design BaseAgent/Agent/Tool/Workflow hierarchy
- [x] **Task 3.2**: Design hooks and events system
### Phase 4: Research Implementation Patterns (🔍 Research)
- [x] **Task 4.1**: Built-in components research
- [x] **Task 4.2**: Composition and orchestration research
### Phase 5: Analyze Integration Points (🔬 Analyze)
- [x] **Task 5.1**: Bridge integration analysis
- [x] **Task 5.2**: Performance and security analysis
- [x] **Task 5.3**: **CRITICAL** Scripting engine concurrency and async patterns research
### Phase 5B: Research Existing Crate Ecosystem (🔍 Research)
- [x] **Task 5B.1**: **CRITICAL** LLM Provider Layer Crates Research - 2025-06-20T08:15:00-08:00
- [x] **Task 5B.2**: Scripting Engine Crates Evaluation - 2025-06-20T09:30:00-08:00
- [x] **Task 5B.3**: Workflow and State Management Crates - 2025-06-20T10:45:00-08:00
- [x] **Task 5B.4**: Supporting Infrastructure Crates - 2025-06-20T11:00:00-08:00
- [x] **Task 5B.1b**: **LLM Provider Layer Decision Summary** - 2025-06-20T11:15:00-08:00
- [x] **Task 5B.5**: Build vs Buy Decision Matrix - 2025-06-20T11:30:00-08:00
### Phase 6: Synthesize Complete System (⚡ Synthesize)
- [x] **Task 6.1**: Complete component ecosystem design - 2025-06-20T11:45:00-08:00
- [x] **Task 6.2**: Script interface design - 2025-06-20T12:00:00-08:00
### Phase 7: Collate Architecture (📋 Collate)
- [x] **Task 7.1**: Organize all concepts into coherent architecture - 2025-06-20T12:15:00-08:00
- [x] **Task 7.2**: Validate against use cases - 2025-06-20T12:30:00-08:00
### Phase 8: Update Architecture Document (📝 Update)
- [x] **Task 8.1**: Update core concepts and philosophy - 2025-06-20T12:45:00-08:00
- [x] **Task 8.2**: Update component architecture sections - 2025-06-20T13:00:00-08:00
- [x] **Task 8.3**: Update directory structure - 2025-06-20T13:15:00-08:00
- [x] **Task 8.4**: Update examples section - 2025-06-20T14:00:00-08:00
### Phase 9: Research Advanced Patterns (🔍 Research)
- [x] **Task 9.1**: Advanced orchestration patterns - 2025-06-20T14:30:00-08:00
- [x] **Task 9.2**: Performance optimization patterns - 2025-06-20T15:00:00-08:00
- [x] **Task 9.3**: Model Control Protocol Support (MCP) - 2025-06-20T15:30:00-08:00
- [x] **Task 9.4**: Agent to Agent Protocol Support (A2A) - 2025-06-20T16:00:00-08:00
- [x] **Task 9.5**: scripting language module support - 2025-06-20T16:30:00-08:00
### Phase 10: Analyze Testing Strategy (🔬 Analyze)
- [x] **Task 10.1**: Testing strategy for new concepts - 2025-06-20T17:00:00-08:00
- [x] **Task 10.2**: Cross-engine compatibility analysis - 2025-06-20T17:30:00-08:00
### Phase 11: Synthesize Final Architecture (⚡ Synthesize)
- [x] **Task 11.1**: Complete architecture integration - 2025-06-20T17:45:00-08:00
- [x] **Task 11.2**: Future evolution strategy - 2025-06-20T18:15:00-08:00
### Phase 12: Collate Final Documentation (📋 Collate)
- [x] **Task 12.1**: Final documentation review - 2025-06-20T18:30:00-08:00

- [x] **Task 12.2**: **CRITICAL** Create Complete Standalone Architecture Document - **COMPLETED** 
- [ ] **Task 12.3**: Manual Review of Final documentation 
  - [ ] Manual human review and correction lists - my prompt for this
    **now I'm going to ask you a series of questions about the architecture documented in @docs/rs-llmspell-complete-architecture.md . for each question, document the question in Section 12.3 of Phase 2. for each question, document the question under the Task/question 12.3.1 heading. for that question review the architecture document and find the answer and document the answer under - [ ] Answer: . if you do not find the answer, document it as such. if you figure out what needs to be done, document in - [ ] Todo: subtask. Once that's done, prompt me for another answer. To understand this read the TODO.md document and make sure you understand the section **Task 12.3** and what I'm saying here. think hard and reason. do what I'm asking in this prompt and come back and tell me that you understand.** 
  - [x] **Task/question 12.3.1** Does the architecture allow for agent handoff from one agent to another without resorting to workflows or resorting to the A2A protocol?
    - [x] Answer: Yes, the architecture now explicitly supports LLM-Driven Delegation (Agent Transfer). This is a first-class feature enabling dynamic, intelligent handoffs.
      1.  **`HandoffRequest`**: The `AgentOutput` struct contains an optional `handoff_request` field. An agent can return this to signal its intent to transfer control.
      2.  **`AgentRuntime`**: A dedicated runtime engine inspects agent outputs. If a `HandoffRequest` is found, the runtime manages the seamless transfer of control and state to the specified target agent.
      3.  **State Management**: The existing state management architecture ensures context is preserved during the handoff.
      4.  This pattern is documented in the "LLM-Driven Delegation (Agent Transfer)" section of the main architecture document.
    - [x] Todo: ~~Consider documenting these handoff patterns more explicitly in a dedicated section of the architecture document~~ **Done**. Future work could include adding more complex handoff examples to the `Real-World Examples` section.
  - [x] **Task/question 12.3.2** Does the architecture allow for rs-llmspell to be used as a module in lua or in javascript to take advantage of tools, agents, workflows, events?
    - [x] Answer: Yes, the architecture is explicitly designed to support this through its **Library Mode**. Key features enabling this are:
      1.  **Dual Usage Paradigms**: The architecture document now clearly defines both "Embedded Mode" (running scripts within `rs-llmspell`) and "Library Mode" (importing `rs-llmspell` into existing applications).
      2.  **Stable C API**: The bridge layer is designed to expose a stable C API, which is the foundation for creating native modules.
      3.  **Native Module Support**: The plan includes creating native modules for Lua (as a LuaRock) and JavaScript (as an NPM package), allowing `require("llmspell")` and `require('@rs/llmspell')` respectively.
      4.  **Build System**: The build system is designed to produce these native modules for distribution.
    - [x] Todo: ~~Clarify in architecture doc that module support is future work, not current capability~~ **Done**. The architecture now fully incorporates this capability.
  - [x] **Task/question 12.3.3** Is the architecture laid out in a modular way such that implementation can occur in verticals or layers across the top to bottom stacks as suggested in the MVP roadmap?
    - [x] Answer: Yes, the architecture is explicitly designed for modular, layered implementation. This is evident through several key design decisions:
      1.  **Four-Layer Architecture**: The document defines clear boundaries between the Script, Bridge, Application, and Infrastructure layers.
      2.  **Modular Crate Structure**: The system is broken into independent crates (e.g., `llmspell-core`, `llmspell-agents`, `llmspell-tools`), allowing features to be built and tested in isolation.
      3.  **Trait-Based Design**: Core functionality is defined by traits (`BaseAgent`, `Tool`, `Workflow`), allowing for different components to be implemented and added incrementally.
      4.  **Component Hierarchy**: The `BaseAgent` -> `Agent` -> `Tool` hierarchy allows for building foundational capabilities first and layering more complex components on top.
      5.  **Feature Flags**: The design supports feature flags, enabling the creation of minimal builds (MVPs) that can be expanded over time.
      6.  **Phased Roadmap**: The architecture document's implementation roadmap is structured in phases, directly supporting the layered MVP approach (e.g., Foundation -> Components -> Advanced Features).
    - [x] Todo: ~~The "Implementation Roadmap" section in the main architecture document should be fully detailed...~~ **Done**. These items have been moved to and expanded in **Phase 13: Implementation Roadmap**.
  - [x] **Task/question 12.3.4** If we were to create a command line executable with a script argument, would the architecture be able to detect the script type (say lua, javascript) and instantiate the right engine and run the script? If not what changes to the architecture to support that?
    - [x] Answer: Yes, the architecture fully supports this. The `llmspell-cli` is designed to automatically detect the script type based on file extension (e.g., `.lua`, `.js`) and can also use shebangs for explicit engine selection. This is now documented in the "Quick Start Guide" and "Build System and Tooling" sections.
    - [x] Todo: ~~Ensure CLI implementation includes robust file extension detection...~~ **Done**. The architecture document has been updated to reflect these capabilities.
  - [x] **Task/question 12.3.5** Does the architecture allow for constructs to run a script in a cron job perhaps as a long running/always running daemon using perhaps workflow loop agent + mcp server, or a2a server or a tcp listener tool etc?
    - [x] Answer: Yes, the architecture has been updated to explicitly support this. It now includes:
      1.  **Scheduler Component**: A first-class `Scheduler` for managing jobs with `Cron`, `Interval`, and `Event` triggers.
      2.  **Listener Tools**: `WebhookListenerTool` and `SocketListenerTool` have been added to the built-in tool catalog to enable external triggers.
      3.  **Daemon/Service Mode**: A `llmspell serve` command is specified for running the framework as a persistent service.
      4.  These additions are documented in the "Scheduling and Automation", "Built-in Tools Catalog", and "Deployment Strategies" sections.
    - [x] Todo: ~~Consider adding scheduled/timer-based workflow triggers...~~ **Done**. The architecture now includes a full `Scheduler` component and listener tools.
  - [x] **Task/question 12.3.6** Does the framework allow me to easily create a new tool in a script or an app using rust? Or create a new tool to add to the library of builtin tools into rs-llmspell directly?
    - [x] Answer: Yes, the framework provides multiple ways to create custom tools:
      1. **Script-level tools**: Easy creation using `Tools.create()` in Lua/JavaScript with configuration object
      2. **Rust applications**: Implement the `Tool` trait with required methods (name, schema, execute_tool)
      3. **Built-in library**: Add tools directly to rs-llmspell by implementing in `llmspell-tools/src/category/`
      4. **Tool registration**: `Tools.register()` in scripts or `agent.register_tool()` in Rust
      5. **Tool composition**: Chain tools together with pipelines and sequential execution
      6. **Plugin system**: Dynamic tool loading for advanced extensibility
      The architecture fully supports tool creation at all levels from simple script functions to complex Rust implementations
    - [x] Todo: ~~Architecture has been updated with "Tool Development Architecture" section covering:~~
      - [x] ~~Tool creation patterns for script-level, native, and plugin approaches~~ **Done**
      - [x] ~~Tool trait architecture and best practices~~ **Done**
      - [x] ~~Tool template system and scaffolding architecture~~ **Done**
      - [x] ~~Simplified patterns via common templates (HTTP API, File Processor, etc.)~~ **Done**
  - [x] **Task/question 12.3.7** Does the framework allow me to easily create a new agent to add to the library of builtin agents into rs-llmspell directly?
    - [x] Answer: Yes, the framework provides clear mechanisms for adding built-in agents:
      1. **Location**: Create in `crates/builtin/src/agents/` directory
      2. **Templates**: Extend existing templates (Chat, Research, Code, Content Creator)
      3. **From scratch**: Implement BaseAgent and Agent traits directly
      4. **Registration**: Add to `register_builtin_agents()` in module
      5. **Factory pattern**: Use AgentTemplateFactory for dynamic creation
      6. **Auto-availability**: Once registered, automatically available in Lua/JS scripts
      Example: Extend ResearchAgent template for MarketResearchAgent with specialized tools and prompts
    - [x] Todo: ~~Architecture has been enhanced with comprehensive agent development patterns:~~
      - [x] ~~Agent template scaffolding via CLI commands (llmspell generate agent)~~ **Done**
      - [x] ~~Advanced template customization options and inheritance patterns~~ **Done**
      - [x] ~~Added 3 specialized agent templates (DataAnalyst, CustomerService, API Integration)~~ **Done**
      - [x] ~~Comprehensive agent testing framework with template-specific tests~~ **Done**
  - [x] **Task/question 12.3.8** Does the architecture security allow for configurable security for a script engine, say lua, where in one script we may want to allow io library access, and in another both io, os and a few other "insecure" libraries? Does it allow for templated security profiles like low, medium, high? If not what are the todos to change the architecture to allow that?
    - [x] Answer: Yes, the architecture now supports both configurable security AND pre-defined profiles:
      1. **Per-script security**: Each agent/script can have its own SecurityConfig
      2. **Library access control**: `stdlib_access` in LuaEngineConfig controls which Lua modules are allowed
      3. **Fine-grained controls**: Filesystem, network, system calls all configurable
      4. **Sandboxing**: Can be enabled/disabled per script via `script_sandbox_mode`
      5. **Resource limits**: Memory, CPU, execution time configurable
      6. **Pre-defined security profiles**: None/Low/Medium/High profiles with preset configurations
      7. **Custom profiles**: Configurable with specific library and access options
      8. **Per-script overrides**: Ability to override security profile for specific scripts with audit trail
    - [x] Todo: ~~Architecture has been comprehensively enhanced with security profile system:~~
      - [x] ~~Created pre-defined security profiles (None/Low/Medium/High/Custom) with Medium as default~~ **Done**
      - [x] ~~Added SecurityProfile enum with preset configurations and CustomSecurityProfile support~~ **Done**
      - [x] ~~Created profile builder/factory pattern with CustomSecurityProfileBuilder~~ **Done**
      - [x] ~~Added SecurityProfilePresets for common use cases (development, testing, production, data_analysis)~~ **Done**
      - [x] ~~Added per-script security override examples in configuration with audit trail~~ **Done**
      - [x] ~~Integrated profile-based library access control in Script Sandbox Security~~ **Done**
  - [ ] **Task/question 12.3.9** Does the architecture spell out configuration for prompts, api keys, security profiles etc via configuration files like yaml or environment variables? If not what todos do we have to add to spell that out?
    - [ ] Answer: Yes, the architecture has comprehensive configuration management:
      1. **File formats**: TOML (default), YAML, JSON all supported
      2. **Main config file**: `llmspell.toml` with complete schema documented
      3. **Environment variables**: `LLMSPELL_` prefix with `__` for nesting
      4. **API keys**: Via env vars (`OPENAI_API_KEY`) or encrypted storage
      5. **Hierarchical loading**: Default → Environment-specific → User → Env vars
      6. **Hot reload**: Configuration changes without restart
      7. **Security profiles**: Configurable via `[security]` and `[security.sandbox]` sections
      8. **Validation**: All configs validated with helpful error messages
      However, prompt templates configuration is not explicitly shown in config files
    - [ ] Todo: 
      - [ ] Add `[prompts]` section to config for system/agent prompt templates
      - [ ] Document prompt template variables and interpolation
      - [ ] Add examples of security profile configurations in config file
      - [ ] Create config generator/validator CLI tool
      - [ ] Add configuration migration tool for version upgrades
  - [ ] **Task/question 12.3.10** Does the framework or architecture spell out standard library for methods in rs-llmspell for respective script engine to load e.g. for lua promise.lua, llm.lua or provider.lua, agent.lua etc, or are they all assumed to be available? Can the scripts load other lua modules? Same goes for javascript. If not what todos do we need to add to have the architecture spell that out?
    - [ ] Answer: No standard library modules exist. The architecture uses global object injection:
      1. **Global APIs**: Agent, Tool, Tools, Workflow, etc. are pre-injected as globals
      2. **No module loading**: Cannot use `require()` or `import` for rs-llmspell APIs
      3. **External modules restricted**: Sandboxing prevents loading arbitrary modules
      4. **Limited npm support**: JavaScript can import pre-approved packages only
      5. **No promise.lua**: Lua uses native coroutines, not a promise library
      6. **Security-first design**: Module loading is intentionally restricted for security
    - [ ] Todo: 
      - [ ] Document the global API injection model explicitly
      - [ ] Create a "Script API Reference" section listing all global objects
      - [ ] Consider adding controlled module loading with whitelist
      - [ ] Add custom module path configuration for trusted environments
      - [ ] Document how to package reusable script code without modules
      - [ ] Consider implementing require() with sandboxed module loader
      - [ ] Add examples of code organization patterns without modules


### Phase 13: Implementation Roadmap 
- [ ] **Task 13.1**: Define a detailed, phased implementation roadmap in the main architecture document.
  - [ ] The roadmap should be structured in layers/phases (e.g., Phase 1: Core Foundation, Phase 2: Built-in Components, etc.).
  - [ ] **CRITICAL**: Explicitly define the scope for a Minimal Viable Product (MVP).
    - [ ] Detail the specific traits required for the MVP (e.g., `BaseAgent`, `Tool`).
    - [ ] List the essential components for the MVP (e.g., `AgentRuntime`, `mlua` script engine).
    - [ ] Specify a small, core set of built-in tools for the initial release.
- [ ] **Task 13.2**: Align the implementation plan with the architectural design.
  - [ ] Establish a priority order for component implementation.
  - [ ] Outline a strategy for handling breaking changes between phases.
  - [ ] Define key testing milestones for each phase.
  - [ ] **Note**: Scheduling features (Scheduler component, cron/interval triggers, daemon mode) should be implemented in a later phase after core functionality is stable
  - [ ] **Agent Development Priority Notes**:
    - Core agent templates (Chat, Research, Code) should be MVP priority
    - Advanced agent templates (DataAnalyst, CustomerService, API Integration) are post-MVP
    - CLI scaffolding commands (`llmspell generate agent/tool`) are enhancement phase, not MVP
    - Template inheritance and dynamic generation are advanced features for later phases
    - Agent testing framework should align with overall testing infrastructure development
  - [ ] **Security Profile Implementation Notes**:
    - Basic security profiles (Medium/High) should be MVP priority for production safety
    - Custom profile builder and presets are enhancement features for later phases
    - Per-script security overrides are advanced features, implement after core security is stable
    - SecurityProfile enum and basic factory methods are core infrastructure needed early

### Phase 14: Final Update (📝 Update)
- [ ] **Task 14.1**: Complete architecture.md update
  - [ ] All sections reflect new architecture
  - [ ] Examples demonstrate new concepts
  - [ ] Testing strategy updated

- [ ] **Task 14.2**: Supporting documentation
  - [ ] Update README.md if needed
  - [ ] Create migration guide outline
  - [ ] Update TODO-DONE.md when complete

## Key Concepts to Address

### Component Hierarchy
- **BaseAgent**: Fundamental agent with tool-handling capabilities
- **Agent**: LLM wrapper with specialized prompts, uses multiple tools
- **Tool**: Special functions LLMs can call, runnable independently
- **Workflow**: Deterministic agents (sequential, parallel, conditional, loop)
- **Tool-wrapped Agent**: Agents wrapped as tools for other agents

### Built-in Components
- **30-40 Built-in Tools**: Ready-to-use tool library
- **Built-in Agents**: Common agent patterns
- **Custom Workflows**: User-defined workflow types

### Hooks and Events
- **Hook Points**: pre-llm, post-llm, pre-tool, post-tool, pre-workflow, post-workflow
- **Event System**: emit/publish/subscribe for orchestration events
- **Built-in Hooks**: logging, metrics, debugging, tracing
- **Script Hooks**: User-defined hooks in Lua/JavaScript

### Integration Points
- **Bridge Layer**: How new concepts map to script engines
- **Testing Strategy**: Comprehensive testing for new concepts
- **Performance**: Efficient hook/event execution
- **Security**: Safe tool-wrapped agent execution
- **Async Patterns**: Concurrency and parallelism in single-threaded scripting engines
- **Cooperative Scheduling**: Non-blocking execution patterns for long-running operations
- **Cross-Engine Compatibility**: Consistent async behavior across Lua and JavaScript

## Success Criteria
- [ ] Architecture reflects go-llms/ADK patterns
- [ ] Clear BaseAgent/Agent/Tool/Workflow hierarchy
- [ ] Comprehensive hooks and events system
- [ ] Built-in component strategy defined
- [ ] Tool-wrapped agent pattern implemented
- [ ] Script interface supports all new concepts
- [ ] **ASYNC PATTERNS**: Cooperative scheduling and async patterns implemented
- [ ] **ASYNC PATTERNS**: Cross-engine async compatibility achieved
- [ ] **ASYNC PATTERNS**: Non-blocking execution for long-running operations
- [ ] Testing strategy covers all components
- [ ] Examples demonstrate real-world usage
- [ ] Performance and security considerations addressed
- [ ] Migration path from current design clear

## Notes
- Reference: https://github.com/lexlapax/go-llms/
- Reference: https://google.github.io/adk-docs/
- Focus on scriptable interface with Rust performance
- Maintain bridge-first philosophy
- Ensure testing infrastructure covers new concepts
- Keep examples practical and realistic