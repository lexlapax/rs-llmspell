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

### Agents
- **[agent-simple.lua](lua/agents/)** - Basic agent creation
- **[agent-memory.lua](lua/agents/)** - Agents with conversation memory
- **[agent-templates.lua](lua/agents/)** - Using agent templates

### Workflows
- **[workflow-sequential.lua](lua/workflows/)** - Sequential execution
- **[workflow-parallel.lua](lua/workflows/)** - Parallel execution
- **[workflow-conditional.lua](lua/workflows/)** - Conditional branching

### Tools
- **[tools-showcase.lua](lua/tools/)** - All available tools
- **[tools-filesystem.lua](lua/tools/)** - File operations
- **[tools-web.lua](lua/tools/)** - Web tools (search, fetch)

### State Management
- **[state-basic.lua](lua/state/)** - Basic state operations
- **[state-persistence.lua](lua/state/)** - Persistent state
- **[state-sessions.lua](lua/state/)** - Session management

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

## See Also

- [CONFIG-PROGRESSION.md](script-users/applications/CONFIG-PROGRESSION.md) - Configuration architecture
- [README.md](script-users/applications/README.md) - Application overview
- [User Guide](../docs/user-guide/) - Complete documentation
- [API Reference](../docs/api/) - Detailed API docs