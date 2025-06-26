# Script Interface Design

## Overview

This document defines the complete script interface for rs-llmspell, enabling Lua and JavaScript developers to leverage the full power of the agent ecosystem with idiomatic APIs for each language.

## 1. Core API Design Principles

### 1.1 Language-Idiomatic APIs
- **Lua**: Table-based configuration, colon syntax for methods, coroutines for async
- **JavaScript**: Object/class syntax, promises/async-await, event emitters
- **Consistency**: Same functionality across languages, different syntax

### 1.2 Progressive Disclosure
- Simple things should be simple
- Complex things should be possible
- Default behaviors that make sense
- Advanced features available when needed

### 1.3 Type Safety Where Possible
- TypeScript definitions for JavaScript
- LuaLS annotations for Lua
- Runtime validation for critical operations
- Clear error messages

## 2. Lua API Design

### 2.1 Agent Creation and Configuration

```lua
-- Simple agent creation
local agent = Agent.new("You are a helpful assistant")

-- Full configuration
local agent = Agent.new({
    system_prompt = "You are a helpful assistant",
    provider = "openai",
    model = "gpt-4",
    temperature = 0.7,
    max_tokens = 1000,
    tools = {
        Calculator.new(),
        WebSearch.new({ api_key = "..." })
    },
    hooks = {
        before_llm_call = function(context)
            print("Calling LLM:", context.prompt)
        end,
        after_llm_call = function(context)
            print("Tokens used:", context.tokens_used)
        end
    }
})

-- Builder pattern alternative
local agent = Agent.builder()
    :system_prompt("You are a helpful assistant")
    :provider("anthropic")
    :model("claude-3")
    :add_tool(Calculator.new())
    :add_tool(WebSearch.new())
    :on_event("tool_called", function(event)
        print("Tool called:", event.tool_name)
    end)
    :build()
```

### 2.2 Synchronous and Asynchronous Operations

```lua
-- Synchronous style (blocking)
local response = agent:chat("What is 2+2?")
print(response)

-- Asynchronous style (coroutines)
coroutine.wrap(function()
    local response = agent:chat_async("Tell me a story")
    print(response)
end)()

-- Streaming responses
coroutine.wrap(function()
    local stream = agent:stream_chat("Tell me a long story")
    for chunk in stream do
        io.write(chunk)
        io.flush()
    end
end)()

-- Multiple agents in parallel
local function parallel_agents()
    local agent1 = Agent.new("You are a poet")
    local agent2 = Agent.new("You are a scientist")
    
    local co1 = coroutine.create(function()
        return agent1:chat_async("Write a haiku")
    end)
    
    local co2 = coroutine.create(function()
        return agent2:chat_async("Explain quantum physics")
    end)
    
    -- Run both coroutines
    local poem = select(2, coroutine.resume(co1))
    local science = select(2, coroutine.resume(co2))
    
    return poem, science
end
```

### 2.3 Tool Definition

```lua
-- Simple tool
local my_tool = Tool.new({
    name = "get_weather",
    description = "Get current weather for a location",
    parameters = {
        type = "object",
        properties = {
            location = {
                type = "string",
                description = "City name"
            },
            units = {
                type = "string",
                enum = {"celsius", "fahrenheit"},
                default = "celsius"
            }
        },
        required = {"location"}
    },
    handler = function(params)
        -- Tool implementation
        local weather = fetch_weather(params.location, params.units)
        return weather
    end
})

-- Class-based tool
local CalculatorTool = Tool:extend()

function CalculatorTool:init()
    self.name = "calculator"
    self.description = "Perform mathematical calculations"
end

function CalculatorTool:execute(params)
    local expr = params.expression
    -- Safe evaluation of math expression
    local result = evaluate_math(expr)
    return { result = result }
end

-- Async tool
local AsyncSearchTool = Tool:extend()

function AsyncSearchTool:execute_async(params)
    return coroutine.wrap(function()
        local results = http.get_async("https://api.search.com", {
            query = params.query
        })
        return { results = results }
    end)
end
```

### 2.4 Hook Registration

```lua
-- Global hooks
Hooks.register("before_llm_call", function(context)
    context.metadata.request_id = generate_uuid()
    print("Request ID:", context.metadata.request_id)
end)

-- Agent-specific hooks
agent:add_hook("after_tool_call", function(context)
    metrics:increment("tool_calls", {
        tool = context.tool_name,
        agent = context.agent_id
    })
end)

-- Hook with priority
agent:add_hook({
    point = "before_execute",
    priority = 10, -- Lower numbers run first
    handler = function(context)
        -- Rate limiting logic
        if rate_limiter:check(context.agent_id) then
            error("Rate limit exceeded")
        end
    end
})

-- Conditional hooks
agent:add_hook("on_error", {
    condition = function(context)
        return context.error:match("timeout")
    end,
    handler = function(context)
        -- Retry with longer timeout
        context.retry = true
        context.timeout = context.timeout * 2
    end
})
```

### 2.5 Event Handling

```lua
-- Simple event handler
agent:on("tool_called", function(event)
    print(string.format("Tool %s called with params: %s",
        event.tool_name,
        json.encode(event.params)))
end)

-- Pattern matching events
agent:on_pattern("tool_*", function(event)
    -- Handles all tool-related events
    log_event(event)
end)

-- Event emitter pattern
local emitter = EventEmitter.new()

emitter:on("agent_ready", function(agent)
    print("Agent ready:", agent.id)
end)

emitter:emit("agent_ready", agent)

-- Async event handling
agent:on_async("long_operation", coroutine.wrap(function(event)
    -- Perform async operation
    local result = fetch_external_data(event.data)
    event:reply(result)
end))
```

### 2.6 Workflow Definition

```lua
-- Sequential workflow
local workflow = Workflow.sequential({
    name = "research_workflow",
    steps = {
        {
            name = "search",
            agent = ResearchAgent.new(),
            input = "{{query}}",
            output = "search_results"
        },
        {
            name = "analyze",
            agent = AnalysisAgent.new(),
            input = "{{search_results}}",
            output = "analysis"
        },
        {
            name = "summarize",
            agent = SummaryAgent.new(),
            input = {
                content = "{{analysis}}",
                max_length = 500
            },
            output = "summary"
        }
    }
})

-- Conditional workflow
local workflow = Workflow.conditional({
    name = "smart_router",
    conditions = {
        {
            name = "is_code_question",
            test = function(input)
                return input.query:match("code") or
                       input.query:match("program")
            end,
            then_branch = "code_path"
        },
        {
            name = "is_math_question",
            test = function(input)
                return input.query:match("calculate") or
                       input.query:match("math")
            end,
            then_branch = "math_path"
        }
    },
    branches = {
        code_path = CodeAgent.new(),
        math_path = MathAgent.new(),
        default = GeneralAgent.new()
    }
})

-- Parallel workflow with aggregation
local workflow = Workflow.parallel({
    name = "multi_perspective",
    branches = {
        optimist = Agent.new("You are an optimist"),
        pessimist = Agent.new("You are a pessimist"),
        realist = Agent.new("You are a realist")
    },
    aggregator = function(results)
        -- Combine perspectives
        return {
            optimistic = results.optimist,
            pessimistic = results.pessimist,
            realistic = results.realist,
            consensus = find_consensus(results)
        }
    end
})
```

## 3. JavaScript API Design

### 3.1 Agent Creation and Configuration

```javascript
// Simple agent creation
const agent = new Agent("You are a helpful assistant");

// Full configuration
const agent = new Agent({
    systemPrompt: "You are a helpful assistant",
    provider: "openai",
    model: "gpt-4",
    temperature: 0.7,
    maxTokens: 1000,
    tools: [
        new Calculator(),
        new WebSearch({ apiKey: "..." })
    ],
    hooks: {
        beforeLLMCall: (context) => {
            console.log("Calling LLM:", context.prompt);
        },
        afterLLMCall: (context) => {
            console.log("Tokens used:", context.tokensUsed);
        }
    }
});

// Builder pattern
const agent = Agent.builder()
    .systemPrompt("You are a helpful assistant")
    .provider("anthropic")
    .model("claude-3")
    .addTool(new Calculator())
    .addTool(new WebSearch())
    .onEvent("toolCalled", (event) => {
        console.log("Tool called:", event.toolName);
    })
    .build();

// Class-based extension
class CustomAgent extends Agent {
    constructor(config) {
        super(config);
        this.customState = {};
    }
    
    async beforeChat(message) {
        // Custom preprocessing
        this.customState.lastMessage = message;
        return message.toLowerCase();
    }
}
```

### 3.2 Async/Await and Streaming

```javascript
// Simple async/await
const response = await agent.chat("What is 2+2?");
console.log(response);

// Streaming responses
const stream = await agent.streamChat("Tell me a long story");
for await (const chunk of stream) {
    process.stdout.write(chunk);
}

// Parallel execution
const [poem, science] = await Promise.all([
    poetAgent.chat("Write a haiku"),
    scientistAgent.chat("Explain quantum physics")
]);

// Async iteration over events
const eventStream = agent.events();
for await (const event of eventStream) {
    console.log("Event:", event.type, event.data);
}

// Timeout handling
const response = await agent.chat("Complex question", {
    timeout: 30000, // 30 seconds
    onTimeout: () => {
        console.log("Request timed out, using fallback");
        return "I need more time to think about that.";
    }
});
```

### 3.3 Tool Definition

```javascript
// Function-based tool
const weatherTool = new Tool({
    name: "get_weather",
    description: "Get current weather for a location",
    parameters: {
        type: "object",
        properties: {
            location: {
                type: "string",
                description: "City name"
            },
            units: {
                type: "string",
                enum: ["celsius", "fahrenheit"],
                default: "celsius"
            }
        },
        required: ["location"]
    },
    handler: async (params) => {
        const weather = await fetchWeather(params.location, params.units);
        return weather;
    }
});

// Class-based tool
class CalculatorTool extends Tool {
    constructor() {
        super({
            name: "calculator",
            description: "Perform mathematical calculations"
        });
    }
    
    async execute(params) {
        const { expression } = params;
        const result = evaluateMath(expression);
        return { result };
    }
}

// Tool with validation
class DatabaseTool extends Tool {
    async validate(params) {
        if (!params.query) {
            throw new Error("Query is required");
        }
        if (params.query.toLowerCase().includes("drop")) {
            throw new Error("Destructive operations not allowed");
        }
    }
    
    async execute(params) {
        this.validate(params);
        return await this.db.query(params.query);
    }
}
```

### 3.4 Hook Registration

```javascript
// Global hooks
Hooks.register("beforeLLMCall", async (context) => {
    context.metadata.requestId = generateUUID();
    console.log("Request ID:", context.metadata.requestId);
});

// Agent-specific hooks
agent.addHook("afterToolCall", async (context) => {
    metrics.increment("tool_calls", {
        tool: context.toolName,
        agent: context.agentId
    });
});

// Hook with options
agent.addHook({
    point: "beforeExecute",
    priority: 10,
    handler: async (context) => {
        if (await rateLimiter.check(context.agentId)) {
            throw new Error("Rate limit exceeded");
        }
    }
});

// Middleware-style hooks
agent.use(async (context, next) => {
    console.log("Before:", context.action);
    const start = Date.now();
    
    await next();
    
    const duration = Date.now() - start;
    console.log("After:", context.action, "took", duration, "ms");
});
```

### 3.5 Event Handling

```javascript
// EventEmitter pattern
agent.on("toolCalled", (event) => {
    console.log(`Tool ${event.toolName} called with params:`, event.params);
});

// Once listeners
agent.once("ready", () => {
    console.log("Agent is ready");
});

// Wildcard events
agent.on("tool:*", (event) => {
    // Handles all tool-related events
    logEvent(event);
});

// Async event handlers
agent.on("longOperation", async (event) => {
    const result = await fetchExternalData(event.data);
    event.reply(result);
});

// Event aggregation
const events = new EventAggregator();
events.collect(agent, "tool:*");

// Later...
const toolEvents = events.getEvents("tool:*");
const stats = analyzeToolUsage(toolEvents);
```

### 3.6 Workflow Definition

```javascript
// Sequential workflow
const workflow = new SequentialWorkflow({
    name: "researchWorkflow",
    steps: [
        {
            name: "search",
            agent: new ResearchAgent(),
            input: "{{query}}",
            output: "searchResults"
        },
        {
            name: "analyze",
            agent: new AnalysisAgent(),
            input: "{{searchResults}}",
            output: "analysis"
        },
        {
            name: "summarize",
            agent: new SummaryAgent(),
            input: {
                content: "{{analysis}}",
                maxLength: 500
            },
            output: "summary"
        }
    ]
});

// Conditional workflow
const workflow = new ConditionalWorkflow({
    name: "smartRouter",
    conditions: [
        {
            name: "isCodeQuestion",
            test: (input) => /code|program/.test(input.query),
            thenBranch: "codePath"
        },
        {
            name: "isMathQuestion",
            test: (input) => /calculate|math/.test(input.query),
            thenBranch: "mathPath"
        }
    ],
    branches: {
        codePath: new CodeAgent(),
        mathPath: new MathAgent(),
        default: new GeneralAgent()
    }
});

// Parallel workflow with map-reduce
const workflow = new MapReduceWorkflow({
    name: "distributedAnalysis",
    mapper: new DataChunkAnalyzer(),
    reducer: new ResultAggregator(),
    partitioner: (data) => {
        // Split data into chunks
        return chunkArray(data, 100);
    }
});

// Custom workflow class
class CustomWorkflow extends Workflow {
    async execute(input) {
        // Pre-processing
        const prepared = await this.prepare(input);
        
        // Dynamic step generation
        const steps = this.generateSteps(prepared);
        
        // Execute with monitoring
        return await this.executeSteps(steps, {
            onStepComplete: (step, result) => {
                this.emit("stepComplete", { step, result });
            }
        });
    }
}
```

## 4. Async Pattern Abstractions

### 4.1 Unified Promise/Future Interface

```lua
-- Lua Promise-like interface
local Promise = require("llmspell.promise")

local promise = Promise.new(function(resolve, reject)
    agent:chat_async("Hello", function(result, err)
        if err then
            reject(err)
        else
            resolve(result)
        end
    end)
end)

promise:then_(function(result)
    print("Success:", result)
end):catch(function(err)
    print("Error:", err)
end)

-- Promise.all equivalent
Promise.all({
    agent1:chat_async("Question 1"),
    agent2:chat_async("Question 2")
}):then_(function(results)
    print("Both completed:", results[1], results[2])
end)
```

```javascript
// JavaScript native promises
const promise = agent.chat("Hello");

// Promise combinators
const results = await Promise.all([
    agent1.chat("Question 1"),
    agent2.chat("Question 2")
]);

// Promise race
const fastest = await Promise.race([
    fastAgent.chat("Quick question"),
    slowAgent.chat("Complex question")
]);

// Promise with timeout
const withTimeout = Promise.race([
    agent.chat("Question"),
    new Promise((_, reject) => 
        setTimeout(() => reject(new Error("Timeout")), 5000)
    )
]);
```

### 4.2 Cooperative Scheduling

```lua
-- Lua cooperative scheduler
local scheduler = Scheduler.new()

-- Add tasks
scheduler:add(function()
    for i = 1, 10 do
        print("Task 1:", i)
        scheduler:yield() -- Give control to other tasks
    end
end)

scheduler:add(function()
    for i = 1, 10 do
        print("Task 2:", i)
        scheduler:yield()
    end
end)

-- Run scheduler
scheduler:run()

-- Async agent with yielding
local function long_running_agent()
    local agent = Agent.new("Process large data")
    
    return coroutine.wrap(function()
        for chunk in data:chunks(1000) do
            local result = agent:process_chunk(chunk)
            coroutine.yield(result) -- Yield control
        end
    end)
end
```

```javascript
// JavaScript async generator for cooperative execution
async function* longRunningAgent(data) {
    const agent = new Agent("Process large data");
    
    for (const chunk of data.chunks(1000)) {
        const result = await agent.processChunk(chunk);
        yield result; // Yield control
    }
}

// Consume with backpressure
const processor = longRunningAgent(bigData);
for await (const result of processor) {
    console.log("Processed chunk:", result);
    // Natural backpressure - won't request next until ready
}

// Manual scheduling
class CooperativeScheduler {
    constructor() {
        this.tasks = [];
    }
    
    add(task) {
        this.tasks.push(task);
    }
    
    async run() {
        while (this.tasks.length > 0) {
            for (const task of this.tasks) {
                if (!task.done) {
                    const { value, done } = await task.next();
                    task.done = done;
                    if (value) console.log(value);
                }
            }
            this.tasks = this.tasks.filter(t => !t.done);
            await new Promise(resolve => setImmediate(resolve));
        }
    }
}
```

### 4.3 Stream Processing

```lua
-- Lua stream interface
local stream = agent:stream_chat("Tell me a story")

-- Process stream with transformation
stream
    :map(function(chunk)
        return chunk:upper()
    end)
    :filter(function(chunk)
        return #chunk > 0
    end)
    :for_each(function(chunk)
        print(chunk)
    end)

-- Batch processing
stream
    :batch(10) -- Collect 10 chunks
    :for_each(function(batch)
        process_batch(batch)
    end)
```

```javascript
// JavaScript stream interface
const stream = await agent.streamChat("Tell me a story");

// Transform stream
const upperStream = stream
    .map(chunk => chunk.toUpperCase())
    .filter(chunk => chunk.length > 0);

// Async transformation
const processedStream = stream.mapAsync(async (chunk) => {
    const enhanced = await enhanceText(chunk);
    return enhanced;
});

// Stream aggregation
const batches = stream.batch(10);
for await (const batch of batches) {
    await processBatch(batch);
}
```

## 5. Built-in Component Access

### 5.1 Tool Library Access

```lua
-- Lua tool library
local tools = require("llmspell.tools")

-- Access built-in tools
local calculator = tools.math.Calculator.new()
local web_search = tools.web.WebSearch.new({
    api_key = config.search_api_key
})

-- List available tools
for category, tools in pairs(tools) do
    print("Category:", category)
    for tool_name, tool_class in pairs(tools) do
        print("  Tool:", tool_name)
    end
end

-- Dynamic tool loading
local tool = tools.load("math.Calculator")
agent:add_tool(tool)
```

```javascript
// JavaScript tool library
import { tools } from 'llmspell';

// Access built-in tools
const calculator = new tools.math.Calculator();
const webSearch = new tools.web.WebSearch({
    apiKey: config.searchApiKey
});

// List available tools
for (const [category, categoryTools] of Object.entries(tools)) {
    console.log("Category:", category);
    for (const [toolName, ToolClass] of Object.entries(categoryTools)) {
        console.log("  Tool:", toolName);
    }
}

// Dynamic tool loading
const ToolClass = await tools.load("math.Calculator");
const tool = new ToolClass();
agent.addTool(tool);
```

### 5.2 Agent Template Access

```lua
-- Lua agent templates
local agents = require("llmspell.agents")

-- Use built-in agent
local chat_agent = agents.ChatAgent.new({
    provider = "openai",
    memory_size = 10
})

local research_agent = agents.ResearchAgent.new({
    search_depth = 3,
    sources = {"web", "academic", "news"}
})

-- Extend built-in agent
local CustomResearcher = agents.ResearchAgent:extend()

function CustomResearcher:init(config)
    agents.ResearchAgent.init(self, config)
    self.custom_sources = config.custom_sources or {}
end

function CustomResearcher:before_search(query)
    -- Custom preprocessing
    return enhance_query(query)
end
```

```javascript
// JavaScript agent templates
import { agents } from 'llmspell';

// Use built-in agents
const chatAgent = new agents.ChatAgent({
    provider: "openai",
    memorySize: 10
});

const researchAgent = new agents.ResearchAgent({
    searchDepth: 3,
    sources: ["web", "academic", "news"]
});

// Extend built-in agent
class CustomResearcher extends agents.ResearchAgent {
    constructor(config) {
        super(config);
        this.customSources = config.customSources || [];
    }
    
    async beforeSearch(query) {
        // Custom preprocessing
        return enhanceQuery(query);
    }
}
```

## 6. Error Handling and Debugging

### 6.1 Error Handling Patterns

```lua
-- Lua error handling
local ok, result = pcall(function()
    return agent:chat("Question")
end)

if not ok then
    print("Error:", result)
    -- Handle error
end

-- Protected async call
local function safe_chat_async(agent, message)
    return coroutine.wrap(function()
        local ok, result = pcall(function()
            return agent:chat_async(message)
        end)
        
        if ok then
            return result, nil
        else
            return nil, result
        end
    end)
end

-- Error context
agent:on_error(function(err, context)
    print("Error in agent:", context.agent_id)
    print("During:", context.action)
    print("Error:", err)
    
    -- Retry logic
    if err:match("timeout") and context.retries < 3 then
        context.retry = true
        context.retries = context.retries + 1
    end
end)
```

```javascript
// JavaScript error handling
try {
    const result = await agent.chat("Question");
} catch (error) {
    console.error("Error:", error);
    // Handle error
}

// With error context
agent.onError((error, context) => {
    console.error("Error in agent:", context.agentId);
    console.error("During:", context.action);
    console.error("Error:", error);
    
    // Retry logic
    if (error.message.includes("timeout") && context.retries < 3) {
        context.retry = true;
        context.retries++;
    }
});

// Async error boundaries
class SafeAgent extends Agent {
    async chat(message) {
        try {
            return await super.chat(message);
        } catch (error) {
            this.emit("error", error);
            return this.fallbackResponse(error);
        }
    }
}
```

### 6.2 Debugging Support

```lua
-- Lua debugging
Debug.enable("llmspell:*")

local agent = Agent.new({
    debug = true,
    system_prompt = "Test agent"
})

-- Trace execution
agent:trace(function(event)
    print(string.format("[%s] %s: %s",
        os.date("%H:%M:%S"),
        event.type,
        json.encode(event.data)
    ))
end)

-- Inspection
print("Agent state:", inspect(agent:get_state()))
print("Tools:", inspect(agent:get_tools()))

-- Step-through debugging
agent:set_breakpoint("before_tool_call", function(context)
    print("About to call tool:", context.tool_name)
    print("Press Enter to continue...")
    io.read()
end)
```

```javascript
// JavaScript debugging
import { Debug } from 'llmspell';

Debug.enable("llmspell:*");

const agent = new Agent({
    debug: true,
    systemPrompt: "Test agent"
});

// Trace execution
agent.trace((event) => {
    console.log(`[${new Date().toISOString()}] ${event.type}:`, event.data);
});

// Inspection
console.log("Agent state:", agent.getState());
console.log("Tools:", agent.getTools());

// Step-through debugging
agent.setBreakpoint("beforeToolCall", async (context) => {
    console.log("About to call tool:", context.toolName);
    await waitForUserInput();
});

// Performance profiling
const profiler = new Profiler();
agent.use(profiler.middleware());

// Later...
console.log("Performance stats:", profiler.getStats());
```

## 7. Integration Examples

### 7.1 Complex Multi-Agent System

```lua
-- Lua multi-agent system
local system = MultiAgentSystem.new()

-- Add specialized agents
system:add_agent("researcher", ResearchAgent.new())
system:add_agent("writer", WriterAgent.new())
system:add_agent("critic", CriticAgent.new())

-- Define collaboration workflow
local workflow = system:create_workflow({
    name = "blog_post_creation",
    steps = {
        {
            agent = "researcher",
            action = "research",
            input = "{{topic}}",
            output = "research"
        },
        {
            agent = "writer",
            action = "write",
            input = {
                research = "{{research}}",
                style = "engaging"
            },
            output = "draft"
        },
        {
            agent = "critic",
            action = "review",
            input = "{{draft}}",
            output = "feedback"
        },
        {
            agent = "writer",
            action = "revise",
            input = {
                draft = "{{draft}}",
                feedback = "{{feedback}}"
            },
            output = "final"
        }
    }
})

-- Execute workflow
local result = workflow:run({ topic = "AI Safety" })
```

```javascript
// JavaScript multi-agent system
const system = new MultiAgentSystem();

// Add specialized agents
system.addAgent("researcher", new ResearchAgent());
system.addAgent("writer", new WriterAgent());
system.addAgent("critic", new CriticAgent());

// Define collaboration workflow
const workflow = system.createWorkflow({
    name: "blogPostCreation",
    steps: [
        {
            agent: "researcher",
            action: "research",
            input: "{{topic}}",
            output: "research"
        },
        {
            agent: "writer",
            action: "write",
            input: {
                research: "{{research}}",
                style: "engaging"
            },
            output: "draft"
        },
        {
            agent: "critic",
            action: "review",
            input: "{{draft}}",
            output: "feedback"
        },
        {
            agent: "writer",
            action: "revise",
            input: {
                draft: "{{draft}}",
                feedback: "{{feedback}}"
            },
            output: "final"
        }
    ]
});

// Execute workflow
const result = await workflow.run({ topic: "AI Safety" });
```

## 8. Best Practices and Guidelines

### 8.1 Resource Management
- Always close agents when done
- Use connection pooling for LLM providers
- Implement proper cleanup in hooks
- Monitor memory usage with large contexts

### 8.2 Error Handling
- Always handle async errors
- Implement retry logic for transient failures
- Log errors with context
- Provide fallback behaviors

### 8.3 Performance
- Use streaming for long responses
- Batch tool calls when possible
- Cache frequently used results
- Profile agent execution

### 8.4 Security
- Validate all tool inputs
- Sanitize LLM outputs
- Implement rate limiting
- Use secure credential storage

## Conclusion

This script interface design provides:
- Idiomatic APIs for both Lua and JavaScript
- Comprehensive async/streaming support
- Flexible hook and event systems
- Easy access to built-in components
- Strong debugging and error handling

The design enables both simple and complex use cases while maintaining consistency across scripting languages.