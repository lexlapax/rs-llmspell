# 01 - Hello World

The simplest possible LLMSpell script to verify your installation is working.

## What You'll Learn

- How to run an LLMSpell script
- Basic Lua syntax
- How to check for LLMSpell globals

## Running the Example

```bash
llmspell run examples/script-users/getting-started/01-hello-world/main.lua
```

## Expected Output

```
Hello from LLMSpell! 🎉

Lua version: Lua 5.4
LLMSpell globals available: Tool, Agent, Workflow, State, Provider

✅ Your LLMSpell installation is working!

Next steps:
  1. Try '02-first-tool' to use your first tool
  2. Run 'llmspell --help' to see all commands
  3. Explore the examples directory for more
```

## Key Concepts

1. **LLMSpell Scripts**: Are Lua scripts with access to special global objects
2. **Global Objects**: Tool, Agent, Workflow, State, Provider are injected automatically
3. **print()**: Standard Lua function for output

## Common Issues

- **"command not found: llmspell"**: Make sure LLMSpell is in your PATH
- **"No LLMSpell globals detected"**: Run the script with `llmspell run`, not plain `lua`

## Next Steps

Continue to [02-first-tool](../02-first-tool/) to learn how to use tools.