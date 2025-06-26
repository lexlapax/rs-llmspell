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
  - [ ] **Task/question 12.3.2** Does the architecture allow for rs-llmspell to be used as a module in lua or in javascript to take advantage of tools, agents, workflows, events?
    - [ ] Answer: No, not in the current implementation. Module support is comprehensively planned but not yet implemented. Currently:
      - Rs-llmspell only supports **embedded scripting** where it hosts Lua/JS runtimes internally
      - Cannot be imported as `require("llmspell")` in external Lua or `import from '@rs/llmspell'` in Node.js
      - The architecture research includes detailed plans for future module support via:
        1. Native modules (Lua C API, Node.js N-API)
        2. C API bridge approach
        3. WebAssembly modules
        4. IPC approach
      - Would require new crates: llmspell-c-api, llmspell-module-core, language bindings
    - [ ] Todo: 
      - [ ] Clarify in architecture doc that module support is future work, not current capability
      - [ ] Consider prioritizing module support in Phase 13 implementation roadmap
      - [ ] Document that current usage is embedded-only (scripts run inside rs-llmspell)
  - [ ] **Task/question 12.3.3** Is the architecture laid out in a modular way such that implementation can occur in verticals or layers across the top to bottom stacks as suggested in the MVP roadmap?
    - [ ] Answer: Yes, the architecture is explicitly designed for modular, layered implementation. Key evidence:
      1. **Four-Layer Architecture**: Script → Bridge → Application → Infrastructure layers with clear boundaries
      2. **Trait-based design**: Composable traits enable incremental implementation
      3. **Bridge-first philosophy**: Build on existing crates, allowing gradual capability addition
      4. **Explicit phased roadmap**: 4-phase implementation plan (Foundation → Scripting/Tools → Advanced → Production)
      5. **Component hierarchy**: BaseAgent/Agent/Tool/Workflow supports independent development
      6. **Production infrastructure at each layer**: Hooks, events, logging built-in from start
      The architecture fully supports the suggested MVP approach: providers+bridge+scripting → add tools → add built-ins → add agents/workflows
    - [ ] Todo: 
      - [ ] Ensure Phase 13 implementation roadmap aligns with the 4-phase plan in the architecture doc
      - [ ] Consider documenting which traits/interfaces are minimal for MVP vs later phases
  - [ ] **Task/question 12.3.4** If we were to create a command line executable with a script argument, would the architecture be able to detect the script type (say lua, javascript) and instantiate the right engine and run the script? If not what changes to the architecture to support that?
    - [ ] Answer: Yes, the architecture already includes this capability. Evidence:
      1. **llmspell-cli**: A dedicated CLI crate is part of the architecture
      2. **Script execution**: `llmspell run hello_world.lua` or `llmspell run script.js` 
      3. **ScriptEngine enum**: Supports Lua, JavaScript, Python (future)
      4. **Automatic detection**: Based on file extensions (.lua, .js)
      5. **Unified execution**: `execute_sandboxed_script` with engine type parameter
      6. **Type conversion**: TypeConverter handles cross-engine type mapping
      The architecture includes all necessary components for CLI-based script execution with automatic type detection
    - [ ] Todo: 
      - [ ] Ensure CLI implementation includes robust file extension detection (.lua, .js, .mjs, etc.)
      - [ ] Consider supporting shebang detection for script type (#!/usr/bin/env lua)
      - [ ] Document CLI usage patterns in the Quick Start section
  - [ ] **Task/question 12.3.5** Does the architecture allow for constructs to run a script in a cron job perhaps as a long running/always running daemon using perhaps workflow loop agent + mcp server, or a2a server or a tcp listener tool etc?
    - [ ] Answer: Yes, the architecture supports daemon/long-running service modes through multiple mechanisms:
      1. **Server Mode**: `rs-llmspell serve` command for running as daemon
      2. **LoopWorkflow**: Continuous workflow execution component
      3. **Cooperative Scheduler**: Manages long-running async tasks with configurable tick duration
      4. **Protocol Servers**: Both MCP and A2A can run as servers
      5. **Container Deployment**: Dockerfile shows service deployment with exposed ports
      6. **Event-driven core**: Built for continuous operation with event bus
      However, explicit cron-like scheduling and built-in TCP server tools are not documented (though infrastructure supports adding them)
    - [ ] Todo: 
      - [ ] Consider adding scheduled/timer-based workflow triggers for cron-like functionality
      - [ ] Document daemon mode usage patterns and examples
      - [ ] Consider adding TCP/Unix socket server tools to built-in catalog
      - [ ] Add examples of long-running workflow patterns
  - [ ] **Task/question 12.3.6** Does the framework allow me to easily create a new tool in a script or an app using rust? Or create a new tool to add to the library of builtin tools into rs-llmspell directly?
    - [ ] Answer: Yes, the framework provides multiple ways to create custom tools:
      1. **Script-level tools**: Easy creation using `Tools.create()` in Lua/JavaScript with configuration object
      2. **Rust applications**: Implement the `Tool` trait with required methods (name, schema, execute_tool)
      3. **Built-in library**: Add tools directly to rs-llmspell by implementing in `llmspell-tools/src/category/`
      4. **Tool registration**: `Tools.register()` in scripts or `agent.register_tool()` in Rust
      5. **Tool composition**: Chain tools together with pipelines and sequential execution
      6. **Plugin system**: Dynamic tool loading for advanced extensibility
      The architecture fully supports tool creation at all levels from simple script functions to complex Rust implementations
    - [ ] Todo: 
      - [ ] Create tool creation tutorial with examples for each approach
      - [ ] Document tool trait requirements and best practices
      - [ ] Add tool template/scaffolding generator
      - [ ] Consider simplified tool creation API for common patterns
  - [ ] **Task/question 12.3.7** Does the framework allow me to easily create a new agent to add to the library of builtin agents into rs-llmspell directly?
    - [ ] Answer: Yes, the framework provides clear mechanisms for adding built-in agents:
      1. **Location**: Create in `crates/builtin/src/agents/` directory
      2. **Templates**: Extend existing templates (Chat, Research, Code, Content Creator)
      3. **From scratch**: Implement BaseAgent and Agent traits directly
      4. **Registration**: Add to `register_builtin_agents()` in module
      5. **Factory pattern**: Use AgentTemplateFactory for dynamic creation
      6. **Auto-availability**: Once registered, automatically available in Lua/JS scripts
      Example: Extend ResearchAgent template for MarketResearchAgent with specialized tools and prompts
    - [ ] Todo: 
      - [ ] Create agent template scaffolding tool
      - [ ] Document agent template customization options
      - [ ] Add more specialized agent templates (e.g., DataAnalyst, CustomerService)
      - [ ] Create agent testing framework and examples
  - [ ] **Task/question 12.3.8** Does the architecture security allow for configurable security for a script engine, say lua, where in one script we may want to allow io library access, and in another both io, os and a few other "insecure" libraries? Does it allow for templated security profiles like low, medium, high? If not what are the todos to change the architecture to allow that?
    - [ ] Answer: Yes, the architecture supports configurable security but not pre-defined profiles:
      1. **Per-script security**: Each agent/script can have its own SecurityConfig
      2. **Library access control**: `stdlib_access` in LuaEngineConfig controls which Lua modules are allowed
      3. **Fine-grained controls**: Filesystem, network, system calls all configurable
      4. **Sandboxing**: Can be enabled/disabled per script via `script_sandbox_mode`
      5. **Resource limits**: Memory, CPU, execution time configurable
      However, pre-defined security profiles (low/medium/high) are NOT currently implemented
    - [ ] Todo: 
      - [ ] Create pre-defined security profiles (low/medium/high):
        - Low: All libraries, minimal restrictions
        - Medium: No os.execute, io.popen, limited filesystem
        - High: Only safe libraries (math, string, table)
      - [ ] Add SecurityProfile enum with preset configurations
      - [ ] Create profile builder/factory for easy security setup
      - [ ] Document security best practices and profile usage
      - [ ] Add per-script security override examples
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
- [ ] **Task 13.1**: Layered implementation roadmap (suggested layers below - need to )
  - [ ] MVP - should be just providers, bridging for providers, scripting engine and one scripting engine that can call providers (mlua)
    - [ ] MVP should have events, logging, metrics hooks
  - [ ] MVP2 - then add tools - and propagate throughout stack upwards
  - [ ] MVP3 - then add builtin-tools - and propagate 
  - [ ] MVP3 - add agents and workflows - and propagate
- [ ]  **Task 13.2** Based on Layered architecture - the actual plan below
  - [ ] Priority order for implementation
  - [ ] Breaking change migration strategy
  - [ ] Testing milestones

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