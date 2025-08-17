# WebApp Creator

A powerful web application generator that uses AI agents to design, architect, and implement complete web applications based on your requirements.

## Features

- **UX Research & Design**: AI agents analyze requirements and design user experience
- **Architecture Planning**: Intelligent system design based on requirements
- **Full Stack Generation**: Creates frontend, backend, database, and deployment configs
- **Testing & Documentation**: Generates tests and comprehensive documentation
- **Multi-Agent Collaboration**: Multiple specialized agents work together

## Usage

### Command-Line Arguments (Recommended - New!)

The WebApp Creator now supports command-line arguments for easier configuration:

```bash
# Using named arguments
./target/debug/llmspell run examples/script-users/applications/webapp-creator/main.lua \
  --input user-input-ecommerce.lua \
  --debug true \
  --max-cost 20

# Using positional arguments
./target/debug/llmspell run examples/script-users/applications/webapp-creator/main.lua \
  user-input-ecommerce.lua

# Mix of positional and named
./target/debug/llmspell run examples/script-users/applications/webapp-creator/main.lua \
  user-input-ecommerce.lua --debug true
```

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
- `ARGS.debug` - Enable debug mode ("true" or "false")
- `ARGS["max-cost"]` - Maximum API cost limit
- `ARGS[1], ARGS[2], ...` - Positional arguments

Example usage in Lua:
```lua
local input_file = ARGS and ARGS.input or "user-input.lua"
local debug_mode = ARGS and ARGS.debug == "true"
local max_cost = tonumber(ARGS and ARGS["max-cost"] or "10")
```

## Output

Generated applications are saved to:
- `/tmp/webapp-creator-generated/` - Temporary output directory
- `generated/` - Saved examples (e.g., `generated/ecommerce-app/`)

## Requirements

- llmspell CLI with Lua support
- Optional: API keys for OpenAI/Anthropic for full AI features
- Optional: Configuration file for advanced settings

## Troubleshooting

If the script doesn't recognize arguments:
1. Ensure you're using the latest llmspell version
2. Check that arguments are properly formatted: `--key value`
3. For hyphenated keys, access them with quotes: `ARGS["max-cost"]`

## Examples

### Basic Task Management App
```bash
./target/debug/llmspell run main.lua
```

### E-commerce Platform
```bash
./target/debug/llmspell run main.lua --input user-input-ecommerce.lua
```

### With Debug Output
```bash
./target/debug/llmspell run main.lua --input user-input-ecommerce.lua --debug true
```

### With Cost Limit
```bash
./target/debug/llmspell run main.lua --input user-input-ecommerce.lua --max-cost 25
```