# Getting Started with LLMSpell

Progressive examples to learn LLMSpell from scratch. Each example builds on the previous one.

## 🎯 Learning Path

### Step 1: Hello World (2 minutes)
**File**: `00-hello-world.lua`  
**Learn**: Basic script structure, running LLMSpell scripts  
**Prerequisites**: None  

```bash
llmspell run 00-hello-world.lua
```

### Step 2: Your First Tool (5 minutes)
**File**: `01-first-tool.lua`  
**Learn**: Using built-in tools, tool parameters, handling results  
**Prerequisites**: Completed Step 1  

```bash
llmspell run 01-first-tool.lua
```

### Step 3: Your First Agent (10 minutes)
**File**: `02-first-agent.lua`  
**Learn**: Creating agents, sending prompts, handling responses  
**Prerequisites**: API key (OpenAI or Anthropic)  

```bash
OPENAI_API_KEY=your-key llmspell run 02-first-agent.lua
```

### Step 4: Your First Workflow (10 minutes)
**File**: `03-first-workflow.lua`  
**Learn**: Combining tools and agents, sequential execution  
**Prerequisites**: Completed Steps 2 and 3  

```bash
OPENAI_API_KEY=your-key llmspell run 03-first-workflow.lua
```

### Step 5: Saving State (5 minutes)
**File**: `04-save-state.lua`  
**Learn**: Persisting data, loading previous state  
**Prerequisites**: Completed Step 1  

```bash
llmspell run 04-save-state.lua
```

### Step 6: Handling Errors (10 minutes)
**File**: `05-handle-errors.lua`  
**Learn**: Error handling, retries, graceful degradation  
**Prerequisites**: Completed Steps 1-5  

```bash
llmspell run 05-handle-errors.lua
```

## 💡 Tips for Success

### Running Examples
1. Always run examples from the project root directory
2. Check output for helpful error messages
3. Use `--debug` flag for verbose output

### Common Issues
- **"API key not found"**: Set environment variable or add to config
- **"Tool not found"**: Ensure you're using the correct tool name
- **"Agent creation failed"**: Check your API key is valid

### Next Steps
After completing these examples:
1. Explore [features](../features/) for specific capabilities
2. Study [cookbook](../cookbook/) for production patterns
3. Build your own scripts!

## 📚 Key Concepts

### Tools
Built-in functions for file operations, web requests, data processing, etc.
- 34+ tools available
- Synchronous execution in Lua
- Automatic error handling

### Agents
LLM-powered assistants that can use tools and follow instructions.
- Multiple provider support
- Tool integration
- Conversation management

### Workflows
Orchestration of tools and agents in complex patterns.
- Sequential execution
- Parallel processing
- Conditional logic
- Loops and iteration

### State
Persistent data storage across script executions.
- Multiple backends (memory, file, database)
- Automatic serialization
- Scoped isolation

### Error Handling
Robust error management for production use.
- Try-catch patterns
- Automatic retries
- Circuit breakers
- Graceful degradation

## 🔗 Resources

- [LLMSpell User Guide](../../../docs/user-guide/getting-started.md)
- [Tool Reference](../../../docs/user-guide/tool-reference.md)
- [API Documentation](../../../docs/api-reference.md)
- [Troubleshooting Guide](../../../docs/user-guide/troubleshooting.md)