# LLMSpell Example Index

## Quick Start Guide

All examples use the `-c` flag for configuration:
```bash
# ✅ CORRECT - Use -c flag for configurations
./target/debug/llmspell -c config.toml run example.lua

# ❌ AVOID - Environment variables cause permission prompts
LLMSPELL_CONFIG=config.toml ./target/debug/llmspell run example.lua
```

## Example Categories

### 🌍 Universal Layer Applications (Beginner)
Simple applications that solve everyday problems with minimal configuration.

| Application | Agents | Config Lines | Purpose | Run Command |
|------------|--------|--------------|---------|-------------|
| **[file-organizer](script-users/applications/file-organizer/)** | 3 | 35 | Organize messy files | `./target/debug/llmspell -c examples/script-users/applications/file-organizer/config.toml run examples/script-users/applications/file-organizer/main.lua` |
| **[research-collector](script-users/applications/research-collector/)** | 2 | 39 | Research automation | `./target/debug/llmspell -c examples/script-users/applications/research-collector/config.toml run examples/script-users/applications/research-collector/main.lua` |

**Key Features**: Single provider, no state persistence, immediate results

### ⚡ Power User Applications (Intermediate)
Productivity-focused applications with quality control and customization.

| Application | Agents | Config Lines | Purpose | Run Command |
|------------|--------|--------------|---------|-------------|
| **[content-creator](script-users/applications/content-creator/)** | 4 | 69 | Content creation with quality control | `./target/debug/llmspell -c examples/script-users/applications/content-creator/config.toml run examples/script-users/applications/content-creator/main.lua` |

**Key Features**: Multiple providers, quality thresholds, memory-only state

### 💼 Business Applications (Advanced)
Enterprise-ready applications with state persistence and session management.

| Application | Agents | Config Lines | Purpose | Run Command |
|------------|--------|--------------|---------|-------------|
| **[communication-manager](script-users/applications/communication-manager/)** | 5 | 109 | Business communication automation | `./target/debug/llmspell -c examples/script-users/applications/communication-manager/config.toml run examples/script-users/applications/communication-manager/main.lua` |

**Key Features**: State persistence, session management, webhooks, SLAs

### 🏢 Professional Applications (Expert)
Enterprise process orchestration with full platform capabilities.

| Application | Agents | Config Lines | Purpose | Run Command |
|------------|--------|--------------|---------|-------------|
| **[process-orchestrator](script-users/applications/process-orchestrator/)** | 8 | 164 | Enterprise process automation | `./target/debug/llmspell -c examples/script-users/applications/process-orchestrator/config.toml run examples/script-users/applications/process-orchestrator/main.lua` |
| **[code-review-assistant](script-users/applications/code-review-assistant/)** | 3 | - | Code quality automation | `./target/debug/llmspell run examples/script-users/applications/code-review-assistant/main.lua` |
| **[webapp-creator](script-users/applications/webapp-creator/)** | 20 | - | AI application generation | `./target/debug/llmspell run examples/script-users/applications/webapp-creator/main.lua` |

**Key Features**: PostgreSQL, Kafka, OAuth2, monitoring, security

## Core Concept Examples

### Getting Started (Beginner)
- **[01-hello-llmspell.lua](script-users/getting-started/01-hello-llmspell.lua)** - First script with basic agent
- **[02-first-agent.lua](script-users/getting-started/02-first-agent.lua)** - Creating and using agents
- **[03-first-workflow.lua](script-users/getting-started/03-first-workflow.lua)** - Introduction to workflows
- **[04-save-state.lua](script-users/getting-started/04-save-state.lua)** - State persistence basics
- **[05-use-tools.lua](script-users/getting-started/05-use-tools.lua)** - Working with tools

### Agent Examples
- **[agent-creation.lua](script-users/features/agent-creation.lua)** - Agent builder patterns
- **[agent-data-processor.lua](script-users/features/agent-data-processor.lua)** - Data processing with agents
- **[agent-api-comprehensive.lua](script-users/features/agent-api-comprehensive.lua)** - Complete agent API usage
- **[agent-monitor.lua](script-users/advanced/agent-monitor.lua)** - Agent monitoring
- **[agent-orchestrator.lua](script-users/advanced/agent-orchestrator.lua)** - Multi-agent orchestration

### Workflow Examples
- **[workflow-sequential-basics.lua](script-users/workflows/workflow-sequential-basics.lua)** - Sequential execution
- **[workflow-parallel-basics.lua](script-users/workflows/workflow-parallel-basics.lua)** - Parallel execution
- **[workflow-conditional.lua](script-users/workflows/workflow-conditional.lua)** - Conditional logic
- **[workflow-loop.lua](script-users/workflows/workflow-loop.lua)** - Loop patterns
- **[workflow-nested.lua](script-users/workflows/workflow-nested.lua)** - Nested workflows
- **[workflow-complex.lua](script-users/workflows/workflow-complex.lua)** - Complex orchestration

### Tool Examples
- **[filesystem-tools.lua](script-users/features/filesystem-tools.lua)** - File system operations
- **[utility-tools.lua](script-users/features/utility-tools.lua)** - Utility tool usage
- **[tools-workflow-chaining.lua](script-users/features/tools-workflow-chaining.lua)** - Chaining tools in workflows
- **[tools-integration.lua](script-users/advanced/tools-integration.lua)** - External tool integration
- **[tools-media.lua](script-users/advanced/tools-media.lua)** - Media processing tools
- **[tools-security.lua](script-users/advanced/tools-security.lua)** - Security tools

### State Management
- **[state-persistence-basics.lua](script-users/features/state-persistence-basics.lua)** - State fundamentals
- **[state-workflow-integration.lua](script-users/cookbook/state-workflow-integration.lua)** - State in workflows

### Advanced Features
- **[streaming-responses.lua](script-users/features/streaming-responses.lua)** - Streaming agent responses
- **[multimodal.lua](script-users/features/multimodal.lua)** - Image and media processing
- **[debug-globals.lua](script-users/features/debug-globals.lua)** - Debug infrastructure usage
- **[comprehensive-demo.lua](script-users/features/comprehensive-demo.lua)** - Full feature demonstration

### Cookbook Patterns
- **[multi-agent-coordination.lua](script-users/cookbook/multi-agent-coordination.lua)** - Agent coordination
- **[agent-composition.lua](script-users/cookbook/agent-composition.lua)** - Composing agent behaviors
- **[workflow-composition.lua](script-users/cookbook/workflow-composition.lua)** - Composing workflows
- **[error-recovery.lua](script-users/cookbook/error-recovery.lua)** - Error recovery patterns

## Learning Path

### Week 1: Universal Foundation
1. Start with `file-organizer` - understand basic agents and workflows
2. Try `research-collector` - learn parallel search patterns
3. Read configs to understand minimal setup

### Week 2: Power User Transition
1. Explore `content-creator` - see quality control in action
2. Study the 69-line config for customization options
3. Experiment with thresholds and parameters

### Week 3: Business Integration
1. Deploy `communication-manager` - understand state persistence
2. Configure webhooks and SLAs
3. Test session management features

### Week 4: Professional Mastery
1. Implement `process-orchestrator` - full enterprise features
2. Study the 164-line config for production deployment
3. Integrate with existing infrastructure

## Configuration Progression

The examples demonstrate a natural configuration complexity progression:

| Layer | Lines | Key Additions |
|-------|-------|--------------|
| Universal | 35-39 | Basic provider, minimal tools |
| Power User | 69 | Quality thresholds, multiple models |
| Business | 109 | State persistence, sessions, webhooks |
| Professional | 164 | PostgreSQL, Kafka, OAuth2, monitoring |

## Quick Commands

### Build the project
```bash
cargo build --release
```

### Run any example with config
```bash
./target/debug/llmspell -c path/to/config.toml run path/to/example.lua
```

### Run with debug output
```bash
./target/debug/llmspell --debug -c config.toml run example.lua
```

### Check available tools
```bash
./target/debug/llmspell exec 'for i, tool in ipairs(Tool.list()) do print(i, tool) end'
```

## Environment Setup

### Required API Keys
Set these environment variables before running examples:
```bash
export OPENAI_API_KEY="sk-..."
export ANTHROPIC_API_KEY="sk-ant-..."
```

### Optional Services
For professional features:
```bash
export DATABASE_URL="postgresql://..."
export OAUTH_TOKEN_ENDPOINT="https://..."
export AZURE_OPENAI_API_KEY="..."
```

## Troubleshooting

### Common Issues

1. **"No API key found"**
   - Set `OPENAI_API_KEY` or `ANTHROPIC_API_KEY` environment variables
   - Check your config.toml has correct `api_key_env` settings

2. **"Config file not found"**
   - Always use absolute or relative paths with `-c` flag
   - Ensure config.toml exists in the application directory

3. **"Agent creation failed"**
   - Verify API keys are valid
   - Check model names in config match available models

4. **"Workflow timeout"**
   - Increase `timeout_seconds` in config
   - Check network connectivity to API providers

## Contributing Examples

When adding new examples:
1. Follow the naming pattern: `category-name.lua`
2. Include comprehensive header comments
3. Provide a matching `config.toml` if needed
4. Update this index with your example
5. Test with `-c` flag usage

## Example Search by Tag

### Difficulty Tags
- **#beginner**: Getting started examples (01-05)
- **#intermediate**: Features and basic workflows
- **#advanced**: Complex patterns and integrations
- **#expert**: Production applications (20+ agents)

### Feature Tags
- **#agents**: Agent creation and orchestration
- **#tools**: Tool usage and integration
- **#workflows**: Workflow patterns
- **#state**: State management
- **#streaming**: Real-time responses
- **#multimodal**: Image/media processing
- **#debug**: Debugging features
- **#production**: Complete applications

### Quick Example Finder

| Need | Example | Location |
|------|---------|----------|
| First script | `01-hello-llmspell.lua` | `getting-started/` |
| Create agent | `agent-creation.lua` | `features/` |
| Build workflow | `workflow-sequential-basics.lua` | `workflows/` |
| Use tools | `filesystem-tools.lua` | `features/` |
| Save state | `state-persistence-basics.lua` | `features/` |
| Multi-agent | `agent-orchestrator.lua` | `advanced/` |
| Complex workflow | `workflow-complex.lua` | `workflows/` |
| Production app | `webapp-creator` | `applications/` |

### Finding Examples with Shell

```bash
# Find all agent examples
find examples -name "*agent*.lua" -type f

# Find beginner examples
ls examples/script-users/getting-started/*.lua

# Find by content
grep -r "Tool.invoke" examples --include="*.lua"

# Use the example finder tool
./tools/find-examples.sh agent
./tools/find-examples.sh --tag beginner
./tools/find-examples.sh --feature workflow
```

## See Also

- [CONFIG-PROGRESSION.md](script-users/applications/CONFIG-PROGRESSION.md) - Configuration architecture
- [README.md](script-users/applications/README.md) - Application overview
- [User Guide](../docs/user-guide/) - Complete documentation
- [Getting Started](../docs/user-guide/getting-started.md) - New user guide
- [Agent API](../docs/user-guide/agent-api.md) - Agent reference
- [Workflow API](../docs/user-guide/workflow-api.md) - Workflow reference
- [Tool Reference](../docs/user-guide/tool-reference.md) - Tool documentation