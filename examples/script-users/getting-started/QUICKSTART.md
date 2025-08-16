# Quick Start Guide - Script Users

Get started with LLMSpell in 5 minutes!

## Prerequisites

1. Install LLMSpell:
```bash
cargo install llmspell
```

2. Set up an API key (optional, for AI features):
```bash
export OPENAI_API_KEY="your-key-here"
# or
export ANTHROPIC_API_KEY="your-key-here"
```

## Your First Script

Create a file `hello.lua`:

```lua
-- hello.lua
print("Hello from LLMSpell!")

-- Use a tool
local result = Tool.invoke("uuid_generator", {
    operation = "generate",
    version = "v4"
})

print("Generated UUID: " .. (result.text or "error"))
```

Run it:
```bash
llmspell run hello.lua
```

## Learning Path

Follow these examples in order:

1. **01-hello-world** - Verify installation and explore globals
2. **02-first-tool** - Learn to use tools for file operations
3. **03-simple-agent** - Create your first AI assistant
4. **04-basic-workflow** - Chain operations together
5. **05-state-persistence** - Save data between runs
6. **06-error-handling** - Build robust scripts

## Common Patterns

### Using Tools
```lua
local result = Tool.invoke("tool_name", {
    operation = "operation_name",
    input = "your data"
})
```

### Creating Agents
```lua
local agent = Agent.builder()
    :name("assistant")
    :model("openai/gpt-3.5-turbo")
    :system_prompt("You are helpful")
    :build()

local response = agent:invoke({ text = "Hello!" })
```

### Building Workflows
```lua
local workflow = Workflow.builder()
    :name("my_workflow")
    :sequential()
    :add_step({
        name = "step1",
        type = "tool",
        tool = "tool_name",
        input = { ... }
    })
    :build()

workflow:execute({})
```

### Error Handling
```lua
local success, result = pcall(function()
    return Tool.invoke("tool_name", {...})
end)

if success then
    print("Success: " .. tostring(result))
else
    print("Error: " .. tostring(result))
end
```

## Next Steps

- Explore the [advanced examples](../advanced/)
- Read the [User Guide](../../../docs/user-guide/)
- Check the [API Reference](../../../docs/developer-guide/api-reference.md)
- Join our community on Discord

## Troubleshooting

### No tools available
Make sure LLMSpell is properly installed and initialized.

### Agent errors
Check that your API keys are set correctly:
```bash
echo $OPENAI_API_KEY
echo $ANTHROPIC_API_KEY
```

### Script errors
Use error handling patterns and check logs:
```bash
RUST_LOG=debug llmspell run your_script.lua
```

## Getting Help

- GitHub Issues: https://github.com/yourusername/llmspell/issues
- Documentation: https://docs.llmspell.dev
- Examples: This directory!