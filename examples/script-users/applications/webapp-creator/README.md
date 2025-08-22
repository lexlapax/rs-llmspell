# WebApp Creator

A powerful web application generator that uses AI agents to design, architect, and implement complete web applications based on your requirements.

## 🎉 Production Ready - Framework Validated

WebApp Creator has been comprehensively tested and validated as part of Task 7.3.10, successfully orchestrating 20 AI agents to generate complete web applications. This application serves as both a powerful tool and a validation suite for the entire llmspell framework.

### Key Achievements (2025-08-22)
- ✅ **20/20 Agents Successfully Executed** - All agents complete without failures
- ✅ **Timeout Configuration Fixed** - Long-running LLM operations now supported
- ✅ **Single Execution Path Validated** - BaseAgent trait unification working correctly
- ✅ **State Persistence Robust** - All outputs correctly saved and retrieved
- ✅ **Production Performance** - ~170 seconds to generate complete applications

## Features

- **UX Research & Design**: AI agents analyze requirements and design user experience
- **Architecture Planning**: Intelligent system design based on requirements
- **Full Stack Generation**: Creates frontend, backend, database, and deployment configs
- **Testing & Documentation**: Generates tests and comprehensive documentation
- **Multi-Agent Collaboration**: 20 specialized agents working in orchestrated sequence
- **Configurable Timeouts**: Support for long-running LLM operations
- **Model Flexibility**: Different models for different agent types

## Usage

### Command-Line Arguments (Recommended - New!)

The WebApp Creator now supports command-line arguments for easier configuration:

```bash
# Basic usage (note the -- before arguments)
./target/debug/llmspell run examples/script-users/applications/webapp-creator/main.lua -- \
  --input user-input-ecommerce.lua

# Specify custom output directory
./target/debug/llmspell run examples/script-users/applications/webapp-creator/main.lua -- \
  --input user-input-ecommerce.lua \
  --output ~/my-projects

# Using all options
./target/debug/llmspell run examples/script-users/applications/webapp-creator/main.lua -- \
  --input user-input-ecommerce.lua \
  --output-dir ./generated \
  --debug true \
  --max-cost 20

# Using positional arguments
./target/debug/llmspell run examples/script-users/applications/webapp-creator/main.lua -- \
  user-input-ecommerce.lua
```

#### Output Directory Structure

The output directory parameter controls where projects are generated:
- Default: `/tmp/<project-name>` (e.g., `/tmp/shopeasy/`)
- Custom: `<output-dir>/<project-name>` (e.g., `~/projects/shopeasy/`)

The project name is automatically converted to a filesystem-safe format (lowercase, spaces replaced with hyphens).

### Environment Variables (Backward Compatible)

The previous environment variable method still works:

```bash
# Select input file via environment variable
WEBAPP_INPUT_FILE=user-input-ecommerce.lua ./target/debug/llmspell run \
  examples/script-users/applications/webapp-creator/main.lua

# With API keys configured
export OPENAI_API_KEY="sk-..."
export ANTHROPIC_API_KEY="sk-ant-..."
./target/debug/llmspell run examples/script-users/applications/webapp-creator/main.lua
```

### Available Input Files

- `user-input.lua` - Default task management app template
- `user-input-ecommerce.lua` - E-commerce platform template
- Create your own by copying and modifying these templates

## Migration Guide

If you have scripts or CI/CD pipelines using the old environment variable method:

**Old Method:**
```bash
WEBAPP_INPUT_FILE=user-input-ecommerce.lua ./target/debug/llmspell run main.lua
```

**New Method (Recommended):**
```bash
./target/debug/llmspell run main.lua --input user-input-ecommerce.lua
```

Both methods work, so you can migrate at your convenience.

## Customization

### Creating Your Own Input File

1. Copy an existing input file:
   ```bash
   cp user-input.lua user-input-myapp.lua
   ```

2. Edit the configuration:
   ```lua
   return {
       project = {
           name = "MyApp",
           description = "My custom application",
           version = "1.0.0"
       },
       requirements = [[
           Your detailed requirements here...
       ]],
       -- ... rest of configuration
   }
   ```

3. Run with your custom input:
   ```bash
   ./target/debug/llmspell run main.lua --input user-input-myapp.lua
   ```

### Available Arguments

When using command-line arguments, the following are available in the Lua script as `ARGS`:

- `ARGS.input` - Input file to use (e.g., "user-input-ecommerce.lua")
- `ARGS.output` or `ARGS["output-dir"]` - Base output directory for generated projects
- `ARGS.debug` - Enable debug mode ("true" or "false")
- `ARGS["max-cost"]` - Maximum API cost limit
- `ARGS[1], ARGS[2], ...` - Positional arguments

Example usage in Lua:
```lua
local input_file = ARGS and ARGS.input or "user-input.lua"
local base_output_dir = ARGS and (ARGS.output or ARGS["output-dir"]) or "/tmp"
local debug_mode = ARGS and ARGS.debug == "true"
local max_cost = tonumber(ARGS and ARGS["max-cost"] or "10")
```

## Output

Generated applications are saved to:
- Default: `/tmp/<project-name>/` (e.g., `/tmp/shopeasy/` for ShopEasy project)
- Custom: `<output-dir>/<project-name>/` when using `--output` or `--output-dir`
- Examples: `./generated/shopeasy/`, `~/projects/taskflow/`, etc.

Each project directory contains:
- `requirements.json` - Analyzed requirements
- `ux-design.json` - UX research and design decisions
- `architecture.json` - System architecture
- `frontend-code.tar.gz` - Frontend application code
- `backend-code.tar.gz` - Backend API code
- `deployment.yaml` - Deployment configuration
- `documentation.md` - Complete project documentation

## Configuration

### Tool Security Configuration (config.toml)

The WebApp Creator uses the llmspell configuration system to control tool behavior and security settings. The `config.toml` file allows you to configure:

#### File Operations Security
```toml
[tools.file_operations]
allowed_paths = [
    "/tmp",
    "/tmp/webapp-projects", 
    "/Users/username/projects",
    "/home/user/web-projects"
]
max_file_size = 52428800  # 50MB
atomic_writes = true
```

#### Web Tools Configuration
```toml
[tools.web_search]
rate_limit_per_minute = 30
max_results = 10
timeout_seconds = 30

[tools.http_request]
timeout_seconds = 30
max_redirects = 5
[tools.http_request.default_headers]
"User-Agent" = "llmspell-webapp-creator/1.0"
```

#### Usage with Configuration
```bash
# Use custom configuration file
LLMSPELL_CONFIG=examples/script-users/applications/webapp-creator/config.toml \
  ./target/debug/llmspell run main.lua -- \
  --input user-input-ecommerce.lua \
  --output /home/user/projects
```

**Security Note**: The `allowed_paths` setting controls where the WebApp Creator can write files. Only directories listed in `allowed_paths` will be accessible for output. This prevents accidental writes to system directories.

### Provider Configuration

Configure AI providers in your config.toml:

```toml
[providers]
default_provider = "openai"

[providers.providers.openai]
provider_type = "openai"
api_key_env = "OPENAI_API_KEY"
model = "gpt-4o-mini"
timeout_seconds = 60

[providers.providers.anthropic]
provider_type = "anthropic"
api_key_env = "ANTHROPIC_API_KEY"
model = "claude-3-haiku-20240307"
timeout_seconds = 60
```

## Requirements

- llmspell CLI with Lua support
- Optional: API keys for OpenAI/Anthropic for full AI features
- Optional: Configuration file (config.toml) for security and tool settings

## Troubleshooting

If the script doesn't recognize arguments:
1. Ensure you're using the latest llmspell version
2. Check that arguments are properly formatted: `--key value`
3. For hyphenated keys, access them with quotes: `ARGS["max-cost"]`

## Examples

### Basic Task Management App (default output to /tmp)
```bash
./target/debug/llmspell run main.lua
```

### E-commerce Platform with Custom Output Directory
```bash
./target/debug/llmspell run main.lua -- \
  --input user-input-ecommerce.lua \
  --output ./my-projects
```

### Generate in Current Directory's 'generated' Folder
```bash
./target/debug/llmspell run main.lua -- \
  --input user-input-ecommerce.lua \
  --output-dir ./generated
```

### With All Options
```bash
./target/debug/llmspell run main.lua -- \
  --input user-input-ecommerce.lua \
  --output ~/web-apps \
  --debug true \
  --max-cost 25
```

### Using Environment Variables (backward compatible)
```bash
WEBAPP_INPUT_FILE=user-input-ecommerce.lua ./target/debug/llmspell run main.lua
```

## Lessons Learned from Task 7.3.10

### Critical Architectural Insights

Through the comprehensive rebuild and validation of WebApp Creator, we discovered and resolved several critical framework issues:

#### 1. Timeout Configuration Bug
**Problem**: Workflow steps were hardcoded to 30-second timeout, insufficient for LLM operations.

**Solution**: Fixed in `llmspell-bridge/src/lua/globals/workflow.rs` to properly pass timeout configuration from Lua scripts:
```lua
-- Now configurable per step
step_config.timeout_ms = 120000  -- 2 minutes for code generation
```

#### 2. Single Execution Path Architecture
**Validation**: Successfully unified all component execution through BaseAgent trait:
- All components implement `execute()` → `execute_impl()` pattern
- State and events handled uniformly across agents, tools, and workflows
- ComponentRegistry properly threaded through execution chain

#### 3. Framework as Validator
WebApp Creator exercises the entire llmspell stack:
- **Agent Orchestration**: 20 agents in sequence
- **State Management**: Persistence across workflow
- **Tool Integration**: File operations for code generation
- **Event System**: Lifecycle events properly emitted
- **Component Registry**: Dynamic component lookup

### Performance Benchmarks

Based on successful production runs:
- **E-commerce App (ShopEasy)**: 168 seconds, 20 files
- **Task Management (TaskFlow)**: 174 seconds, 20 files
- **Average per Agent**: 8-9 seconds including LLM latency

### Best Practices Discovered

1. **Timeout Configuration**: Always configure timeouts based on agent complexity:
   - Code generation agents: 120-180 seconds
   - Analysis agents: 60-90 seconds
   - Simple agents: 30-45 seconds

2. **Model Selection**: Use appropriate models for different tasks:
   - Claude Sonnet for code generation (better structured output)
   - GPT-4 for complex reasoning and architecture
   - GPT-3.5 for simple analysis (cost-effective)

3. **State Management**: Enable persistence for recovery:
   ```toml
   [state]
   enabled = true
   persistence = true
   ```

4. **Error Recovery**: Workflow state persists across failures, allowing resumption

### Production Readiness

WebApp Creator demonstrates llmspell is production-ready for:
- ✅ Complex multi-agent workflows
- ✅ Long-running LLM operations
- ✅ State persistence and recovery
- ✅ Tool integration and file generation
- ✅ Event-driven architectures

## Additional Documentation

For comprehensive configuration and troubleshooting:
- [CONFIG.md](CONFIG.md) - Detailed configuration guide with timeout management
- [OUTPUT-STRUCTURE.md](OUTPUT-STRUCTURE.md) - Expected file structure and outputs
- [minimal-input.lua](minimal-input.lua) - Simple starting template

## Support

For issues or questions:
- Check [CONFIG.md](CONFIG.md) troubleshooting section
- Review workflow logs with `RUST_LOG=debug`
- Ensure API keys are properly configured
- Verify timeout settings for your use case