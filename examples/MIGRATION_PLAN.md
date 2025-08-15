# Example Migration Plan

## Overview
This plan outlines the migration of 125 examples into a new organized structure, to be executed in Task 7.3.3 after the directory structure is created in Task 7.3.2.

## New Directory Structure (from Task 7.3.2)
```
examples/
├── README.md                    # Main navigation
├── STANDARDS.md                 # Example standards
├── script-users/               # Lua examples for end users
│   ├── getting-started/        # Progressive learning path
│   ├── features/               # Feature demonstrations
│   ├── cookbook/               # Patterns and recipes
│   └── applications/           # Complete applications
├── rust-developers/            # Rust examples for developers
│   ├── getting-started/        # Basics of embedding
│   ├── api-usage/              # API demonstrations
│   ├── patterns/               # Design patterns
│   └── extensions/             # Extending llmspell
└── tests-as-examples/          # Test files that serve as examples
    ├── integration/            # Integration test examples
    └── benchmarks/             # Performance examples
```

## Migration Priorities

### Priority 1: Getting Started (Critical Path)
These examples form the critical learning path and must be migrated first.

#### Script Users Getting Started
| Current Location | New Location | Action |
|-----------------|--------------|--------|
| examples/hello.lua | script-users/getting-started/00-hello-world.lua | Move & enhance |
| examples/lua/tools/tools-showcase.lua | script-users/getting-started/01-first-tool.lua | Simplify & move |
| examples/lua/agents/agent-simple.lua | script-users/getting-started/02-first-agent.lua | Move & document |
| examples/lua/workflows/workflow-simple.lua | script-users/getting-started/03-first-workflow.lua | Move & enhance |
| examples/lua/state/state-basic.lua | script-users/getting-started/04-save-state.lua | Move & enhance |
| NEW | script-users/getting-started/05-handle-errors.lua | Create new |

#### Rust Developers Getting Started
| Current Location | New Location | Action |
|-----------------|--------------|--------|
| llmspell-bridge/examples/lua_script.rs | rust-developers/getting-started/00-embed-llmspell.rs | Move & enhance |
| llmspell-tools/examples/calculator_tool.rs | rust-developers/getting-started/01-custom-tool.rs | Move |
| llmspell-agents/examples/basic_agent.rs | rust-developers/getting-started/02-custom-agent.rs | Move |
| llmspell-workflows/examples/sequential_workflow.rs | rust-developers/getting-started/03-workflows.rs | Move |
| NEW | rust-developers/getting-started/04-testing.rs | Create new |

### Priority 2: Feature Demonstrations
Core feature examples that show capabilities.

#### Agents
| Current Location | New Location | Action |
|-----------------|--------------|--------|
| examples/lua/agents/agent-api-comprehensive.lua | script-users/features/agents-comprehensive.lua | Move |
| examples/lua/agents/agent-async-example.lua | script-users/features/agents-async.lua | Move & fix |
| examples/lua/agents/agent-composition.lua | script-users/cookbook/agent-composition.lua | Move |
| examples/lua/agents/agent-coordinator.lua | script-users/cookbook/multi-agent-coordination.lua | Move |

#### Tools
| Current Location | New Location | Action |
|-----------------|--------------|--------|
| examples/lua/tools/tools-filesystem.lua | script-users/features/tools-filesystem.lua | Move |
| examples/lua/tools/tools-communication.lua | script-users/features/tools-communication.lua | Move |
| examples/lua/tools/tools-web.lua | script-users/features/tools-web.lua | Move |
| examples/lua/tools/tools-data.lua | script-users/features/tools-data.lua | Move |

#### Workflows
| Current Location | New Location | Action |
|-----------------|--------------|--------|
| examples/lua/workflows/workflow-sequential.lua | script-users/features/workflow-sequential.lua | Move |
| examples/lua/workflows/workflow-parallel.lua | script-users/features/workflow-parallel.lua | Move |
| examples/lua/workflows/workflow-conditional.lua | script-users/features/workflow-conditional.lua | Move |
| examples/lua/workflows/workflow-loop.lua | script-users/features/workflow-loop.lua | Move |

### Priority 3: Cookbook Examples
Common patterns and best practices.

| Current Location | New Location | Action |
|-----------------|--------------|--------|
| examples/lua/workflows/workflow-error-handling.lua | script-users/cookbook/error-handling.lua | Move & enhance |
| examples/lua/workflows/workflow-retries.lua | script-users/cookbook/retry-strategies.lua | Move |
| examples/lua/hooks/hook-circuit-breaker.lua | script-users/cookbook/circuit-breaker.lua | Move |
| examples/lua/events/event-rate-limit.lua | script-users/cookbook/rate-limiting.lua | Move |
| examples/lua/state/state-shared.lua | script-users/cookbook/state-sharing.lua | Move |

### Priority 4: Applications
Complete example applications.

| Current Location | New Location | Action |
|-----------------|--------------|--------|
| examples/llmspell-demo.lua | script-users/applications/demo-app/ | Convert to full app |
| NEW | script-users/applications/research-assistant/ | Create new |
| NEW | script-users/applications/data-pipeline/ | Create new |
| NEW | script-users/applications/monitoring-system/ | Create new |

## Consolidation Plan

### Examples to Merge
1. **Agent Examples**:
   - Merge: agent-simple.lua + agent-simple-demo.lua → getting-started series
   - Keep best parts of each

2. **Tool Examples**:
   - Merge: tools-showcase.lua + tools-run-all.lua + tools-api.lua → comprehensive tool guide
   - Split by feature area

3. **State Examples**:
   - Merge: state-basic.lua + state-persistence.lua → progressive state series

### Examples to Remove
1. Outdated examples (using old APIs)
2. Broken examples that can't be fixed
3. True duplicates with no unique value

## Enhancement Requirements

### All Migrated Examples Must Have:
1. **Header Documentation**:
   ```lua
   -- Example: [Name]
   -- Purpose: [What this demonstrates]
   -- Audience: [Script Users/Rust Developers]
   -- Prerequisites: [What to know before running]
   -- Expected Output: [What should happen]
   ```

2. **Error Handling**:
   - Wrap main logic in pcall/try-catch
   - Show error recovery patterns
   - Validate inputs

3. **Configuration**:
   - No hard-coded API keys
   - Configurable paths
   - Environment variables

4. **Testing**:
   - Include test script
   - Document how to verify success
   - Add to CI

## Timeline

### Phase 1: Structure & Getting Started (Day 1)
- Create directory structure (Task 7.3.2)
- Migrate Priority 1 examples
- Create missing getting-started examples

### Phase 2: Features & Cookbook (Day 2)
- Migrate Priority 2 & 3 examples
- Consolidate duplicates
- Enhance with documentation

### Phase 3: Applications & Polish (Day 3)
- Create application examples
- Add testing for all examples
- Update main documentation

## Success Criteria

### Quantitative
- [ ] All 125 examples migrated or consolidated
- [ ] 0 duplicate examples remain
- [ ] 100% examples have documentation
- [ ] 100% examples handle errors
- [ ] Getting-started series complete for both audiences

### Qualitative
- [ ] Clear navigation structure
- [ ] Progressive learning path
- [ ] Self-contained examples
- [ ] Production-ready patterns demonstrated

## Post-Migration Tasks
1. Update all documentation references
2. Set up CI for example testing
3. Create example search/discovery tool
4. Get user feedback on new structure
5. Iterate based on feedback