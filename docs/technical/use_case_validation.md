# Use Case Validation

## Overview

This document validates the rs-llmspell architecture against real-world use cases, ensuring the design supports both simple and complex scenarios effectively.

## 1. Simple Tool Execution Scenarios

### 1.1 Basic Calculator Tool

**Use Case**: User asks agent to perform a calculation

**Lua Implementation**:
```lua
-- Create agent with calculator tool
local agent = Agent.new({
    system_prompt = "You are a helpful math assistant",
    tools = { Calculator.new() }
})

-- Simple calculation
local result = agent:chat("What is 15% of 2500?")
-- Agent internally calls calculator tool with expression "2500 * 0.15"
-- Returns: "15% of 2500 is 375"
```

**Validation**:
- ✅ Tool automatically registered with agent
- ✅ Agent determines when to use tool
- ✅ Tool execution transparent to user
- ✅ Natural language response

**JavaScript Implementation**:
```javascript
// Create agent with calculator tool
const agent = new Agent({
    systemPrompt: "You are a helpful math assistant",
    tools: [new Calculator()]
});

// Async calculation
const result = await agent.chat("Calculate compound interest on $1000 at 5% for 3 years");
// Agent calls calculator with compound interest formula
// Returns formatted result
```

**Validation**:
- ✅ Async/await pattern natural for JS
- ✅ Tool parameters automatically extracted
- ✅ Error handling for invalid expressions

### 1.2 File Operations with Permissions

**Use Case**: Agent reads and processes files with safety checks

**Lua Implementation**:
```lua
local agent = Agent.new({
    system_prompt = "You help analyze text files",
    tools = {
        FileSystemTool.new({
            allowed_paths = { "/tmp", "./data" },
            read_only = true
        })
    }
})

-- With hooks for safety
agent:add_hook("before_tool_call", function(context)
    if context.tool_name == "filesystem" then
        print("Accessing file:", context.params.path)
        -- Additional validation
        if not is_safe_path(context.params.path) then
            error("Unsafe file access attempted")
        end
    end
end)

local analysis = agent:chat("Analyze the contents of ./data/report.txt")
```

**Validation**:
- ✅ Tool configuration for safety
- ✅ Hook system intercepts tool calls
- ✅ Path validation before execution
- ✅ Clear error messages

### 1.3 Web Search with Rate Limiting

**Use Case**: Agent searches web with rate limiting and caching

**JavaScript Implementation**:
```javascript
const agent = new Agent({
    systemPrompt: "You are a research assistant",
    tools: [
        new WebSearch({
            apiKey: process.env.SEARCH_API_KEY,
            rateLimit: { requests: 10, per: "minute" }
        })
    ],
    hooks: {
        beforeToolCall: async (context) => {
            if (context.toolName === "web_search") {
                // Check cache first
                const cached = await cache.get(context.params.query);
                if (cached) {
                    context.skipExecution = true;
                    context.result = cached;
                }
            }
        },
        afterToolCall: async (context) => {
            if (context.toolName === "web_search" && context.success) {
                // Cache results
                await cache.set(context.params.query, context.result, { ttl: 3600 });
            }
        }
    }
});

const research = await agent.chat("Find recent developments in quantum computing");
```

**Validation**:
- ✅ Rate limiting built into tool
- ✅ Hooks enable caching layer
- ✅ Async operations handled cleanly
- ✅ Environment variable configuration

## 2. Complex Multi-Agent Workflows

### 2.1 Blog Post Creation Pipeline

**Use Case**: Multiple specialized agents collaborate to create content

**Lua Implementation**:
```lua
-- Define specialized agents
local researcher = ResearchAgent.new({
    search_depth = 3,
    sources = { "academic", "news", "blogs" }
})

local writer = Agent.new({
    system_prompt = "You are an engaging technical writer",
    tools = { MarkdownFormatter.new() }
})

local editor = Agent.new({
    system_prompt = "You are a meticulous editor focusing on clarity and accuracy",
    tools = { GrammarChecker.new(), FactChecker.new() }
})

-- Create workflow
local blog_workflow = Workflow.sequential({
    name = "blog_creation",
    steps = {
        {
            name = "research",
            agent = researcher,
            input = "{{topic}}",
            output = "research_notes",
            timeout = 120  -- 2 minute timeout
        },
        {
            name = "write_draft",
            agent = writer,
            input = {
                research = "{{research_notes}}",
                style = "technical but accessible",
                length = "1500 words"
            },
            output = "draft"
        },
        {
            name = "edit",
            agent = editor,
            input = "{{draft}}",
            output = "edited_draft"
        },
        {
            name = "final_review",
            agent = writer,
            input = {
                draft = "{{edited_draft}}",
                feedback = "{{edit.metadata.suggestions}}"
            },
            output = "final_post"
        }
    },
    on_step_complete = function(step, result)
        print(string.format("Completed %s in %.2fs", 
            step.name, 
            step.duration))
    end
})

-- Execute workflow
local blog_post = blog_workflow:run({
    topic = "The Future of WebAssembly"
})
```

**Validation**:
- ✅ Specialized agents with different prompts/tools
- ✅ Data flows between steps via outputs
- ✅ Timeout and error handling per step
- ✅ Progress tracking with callbacks
- ✅ Metadata passed between agents

### 2.2 Parallel Analysis with Aggregation

**Use Case**: Multiple agents analyze data from different perspectives simultaneously

**JavaScript Implementation**:
```javascript
// Create analyzer agents
const sentimentAnalyzer = new Agent({
    systemPrompt: "Analyze sentiment and emotional tone",
    tools: [new SentimentTool()]
});

const technicalAnalyzer = new Agent({
    systemPrompt: "Analyze technical accuracy and complexity",
    tools: [new ComplexityScorer(), new JargonDetector()]
});

const audienceAnalyzer = new Agent({
    systemPrompt: "Analyze target audience and accessibility",
    tools: [new ReadabilityTool()]
});

// Parallel workflow with aggregation
const analysisWorkflow = new ParallelWorkflow({
    name: "comprehensive_analysis",
    branches: {
        sentiment: {
            agent: sentimentAnalyzer,
            input: "{{document}}"
        },
        technical: {
            agent: technicalAnalyzer,
            input: "{{document}}"
        },
        audience: {
            agent: audienceAnalyzer,
            input: "{{document}}"
        }
    },
    aggregator: async (results) => {
        // Custom aggregation logic
        const summary = new SummaryAgent();
        return await summary.chat({
            message: "Synthesize these analyses into a comprehensive report",
            context: results
        });
    },
    options: {
        maxConcurrency: 3,
        failureStrategy: "continue" // Continue even if one fails
    }
});

// With progress tracking
const analysis = await analysisWorkflow.run(
    { document: documentText },
    {
        onBranchComplete: (branch, result) => {
            console.log(`${branch} analysis complete`);
        }
    }
);
```

**Validation**:
- ✅ True parallel execution
- ✅ Different agents with specialized tools
- ✅ Aggregation step combines results
- ✅ Failure handling strategies
- ✅ Progress callbacks for UI updates

### 2.3 Conditional Routing Workflow

**Use Case**: Route requests to appropriate specialists based on content

**Lua Implementation**:
```lua
-- Routing agent determines intent
local router = Agent.new({
    system_prompt = "Classify user requests and route to appropriate specialist"
})

-- Specialist agents
local code_helper = CodeAgent.new({
    languages = { "rust", "python", "javascript" }
})

local math_expert = Agent.new({
    system_prompt = "You are a mathematics expert",
    tools = { Calculator.new(), SymbolicMath.new(), Plotter.new() }
})

local general_assistant = ChatAgent.new()

-- Conditional workflow
local smart_assistant = Workflow.conditional({
    name = "smart_routing",
    
    -- First, classify the request
    classifier = function(input)
        local classification = router:chat(
            "Classify this request: " .. input.message
        )
        return classification
    end,
    
    conditions = {
        {
            name = "is_code_request",
            test = function(classification)
                return classification:match("programming") or 
                       classification:match("code")
            end,
            then_branch = "code_specialist"
        },
        {
            name = "is_math_request",
            test = function(classification)
                return classification:match("mathematical") or
                       classification:match("calculation")
            end,
            then_branch = "math_specialist"
        }
    },
    
    branches = {
        code_specialist = code_helper,
        math_specialist = math_expert,
        default = general_assistant
    },
    
    -- Track routing decisions
    on_branch_selected = function(branch, classification)
        metrics:increment("routing", { branch = branch })
    end
})

-- Usage
local response = smart_assistant:run({
    message = "Write a Python function to calculate fibonacci numbers"
})
-- Routes to code_specialist
```

**Validation**:
- ✅ Dynamic routing based on content
- ✅ Classification step before routing
- ✅ Multiple conditional branches
- ✅ Default fallback handling
- ✅ Metrics tracking for routing

## 3. Hook and Event Driven Automation

### 3.1 Automatic Error Recovery

**Use Case**: Automatically retry failed operations with exponential backoff

**JavaScript Implementation**:
```javascript
class ResilientAgent extends Agent {
    constructor(config) {
        super(config);
        
        // Add retry hook
        this.addHook({
            point: "afterExecute",
            handler: async (context) => {
                if (context.error && this.shouldRetry(context.error)) {
                    const retryCount = context.metadata.retryCount || 0;
                    
                    if (retryCount < 3) {
                        // Exponential backoff
                        const delay = Math.pow(2, retryCount) * 1000;
                        await new Promise(resolve => setTimeout(resolve, delay));
                        
                        context.retry = true;
                        context.metadata.retryCount = retryCount + 1;
                        
                        // Emit event for monitoring
                        this.emit("retry", {
                            attempt: retryCount + 1,
                            error: context.error,
                            delay
                        });
                    }
                }
            }
        });
    }
    
    shouldRetry(error) {
        return error.code === "RATE_LIMIT" || 
               error.code === "TIMEOUT" ||
               error.message.includes("temporary");
    }
}

// Usage with event monitoring
const agent = new ResilientAgent({ provider: "openai" });

agent.on("retry", (event) => {
    console.log(`Retry attempt ${event.attempt} after ${event.delay}ms`);
});

// Automatic retries happen transparently
const result = await agent.chat("Complex query that might fail");
```

**Validation**:
- ✅ Hook intercepts failures
- ✅ Exponential backoff implemented
- ✅ Event emission for monitoring
- ✅ Transparent to caller
- ✅ Configurable retry logic

### 3.2 Real-time Progress Streaming

**Use Case**: Stream progress updates during long operations

**Lua Implementation**:
```lua
-- Agent with progress streaming
local analyst = Agent.new({
    system_prompt = "You perform detailed analysis"
})

-- Add progress hooks
analyst:add_hook("before_tool_call", function(context)
    context.event_emitter:emit("progress", {
        stage = "tool_execution",
        tool = context.tool_name,
        status = "starting"
    })
end)

analyst:add_hook("after_tool_call", function(context)
    context.event_emitter:emit("progress", {
        stage = "tool_execution",
        tool = context.tool_name,
        status = "completed",
        duration = context.duration
    })
end)

-- Stream progress to client
local progress_stream = analyst:create_event_stream("progress")

-- In another coroutine, handle progress
coroutine.wrap(function()
    for event in progress_stream do
        update_ui(event)
    end
end)()

-- Long running analysis
local report = analyst:chat("Analyze this 10MB dataset: " .. dataset_path)
```

**Validation**:
- ✅ Real-time progress events
- ✅ Streaming without blocking
- ✅ Detailed stage information
- ✅ UI update capability
- ✅ Coroutine-based streaming

### 3.3 Compliance and Audit Logging

**Use Case**: Comprehensive audit trail for all agent activities

**JavaScript Implementation**:
```javascript
// Audit logger
class AuditLogger {
    constructor(storage) {
        this.storage = storage;
    }
    
    createAuditHook() {
        return {
            point: "afterExecute",
            priority: -1000, // Run last
            handler: async (context) => {
                const auditEntry = {
                    timestamp: new Date().toISOString(),
                    agentId: context.agentId,
                    action: context.action,
                    input: this.sanitize(context.input),
                    output: this.sanitize(context.output),
                    toolsUsed: context.toolCalls,
                    duration: context.duration,
                    tokenUsage: context.tokenUsage,
                    error: context.error,
                    userId: context.metadata.userId,
                    sessionId: context.metadata.sessionId
                };
                
                await this.storage.append("audit_log", auditEntry);
                
                // Emit for real-time monitoring
                eventBus.emit("audit", auditEntry);
            }
        };
    }
    
    sanitize(data) {
        // Remove sensitive information
        return sanitizeHelper.clean(data, ["password", "api_key", "token"]);
    }
}

// Global audit setup
const auditLogger = new AuditLogger(secureStorage);
Hooks.register("afterExecute", auditLogger.createAuditHook());

// Usage - all agents automatically audited
const agent = new Agent({ 
    systemPrompt: "Financial advisor",
    metadata: { userId: "user123", sessionId: "session456" }
});

const advice = await agent.chat("Investment recommendations for $50k");
// Automatically logged with full context
```

**Validation**:
- ✅ Automatic audit for all agents
- ✅ Sensitive data sanitization
- ✅ Comprehensive context capture
- ✅ Real-time event emission
- ✅ Persistent storage

## 4. Built-in Component Usage

### 4.1 Tool Composition

**Use Case**: Combine multiple tools for complex operations

**Lua Implementation**:
```lua
-- Create composite tool
local DataAnalysisTool = Tool:extend()

function DataAnalysisTool:init()
    self.name = "data_analysis"
    self.description = "Comprehensive data analysis with visualization"
    
    -- Compose from built-in tools
    self.csv_reader = tools.data.CsvTool.new()
    self.stats = tools.math.StatisticsTool.new()
    self.plotter = tools.visualization.Plotter.new()
end

function DataAnalysisTool:execute(params)
    -- Read data
    local data = self.csv_reader:execute({
        file = params.file,
        parse_numbers = true
    })
    
    -- Calculate statistics
    local stats = self.stats:execute({
        data = data.values,
        operations = ["mean", "median", "std", "correlation"]
    })
    
    -- Create visualization
    local plot = self.plotter:execute({
        data = data,
        type = "scatter",
        title = params.title or "Data Analysis",
        stats_overlay = stats
    })
    
    return {
        statistics = stats,
        visualization = plot,
        summary = self:generate_summary(stats)
    }
end

-- Use in agent
local analyst = Agent.new({
    system_prompt = "You are a data analyst",
    tools = { DataAnalysisTool.new() }
})
```

**Validation**:
- ✅ Tool composition pattern
- ✅ Reuse of built-in tools
- ✅ Custom tool interface
- ✅ Rich return values
- ✅ Natural integration with agents

### 4.2 Agent Template Extension

**Use Case**: Extend built-in agent templates with custom behavior

**JavaScript Implementation**:
```javascript
// Extend built-in research agent
class AcademicResearchAgent extends agents.ResearchAgent {
    constructor(config) {
        super({
            ...config,
            sources: ["academic", "arxiv", "pubmed"],
            citationStyle: config.citationStyle || "APA"
        });
        
        // Add specialized tools
        this.addTool(new CitationFormatter({
            style: this.config.citationStyle
        }));
        
        this.addTool(new PaperSummarizer());
        
        // Custom hooks
        this.addHook({
            point: "afterSearch",
            handler: async (context) => {
                // Filter for peer-reviewed only
                context.results = context.results.filter(
                    r => r.metadata.peerReviewed
                );
            }
        });
    }
    
    async analyzePaper(paperUrl) {
        const paper = await this.tools.paperFetcher.execute({ url: paperUrl });
        const summary = await this.tools.paperSummarizer.execute({ 
            content: paper 
        });
        
        return {
            summary,
            citation: this.tools.citationFormatter.execute({
                paper: paper.metadata
            }),
            relatedWorks: await this.findRelated(paper)
        };
    }
}

// Usage
const researcher = new AcademicResearchAgent({
    citationStyle: "MLA",
    llmProvider: "anthropic"
});

const analysis = await researcher.analyzePaper("https://arxiv.org/...");
```

**Validation**:
- ✅ Inherits base functionality
- ✅ Adds specialized tools
- ✅ Custom methods for domain
- ✅ Configuration extension
- ✅ Hook integration

### 4.3 Workflow Template Customization

**Use Case**: Customize built-in workflow templates

**Lua Implementation**:
```lua
-- Extend map-reduce workflow for distributed analysis
local DistributedAnalysis = workflows.MapReduceWorkflow:extend()

function DistributedAnalysis:init(config)
    workflows.MapReduceWorkflow.init(self, config)
    
    -- Custom partitioner for documents
    self.partitioner = function(documents)
        local chunks = {}
        local chunk_size = math.ceil(#documents / self.config.workers)
        
        for i = 1, #documents, chunk_size do
            table.insert(chunks, {
                documents = table.slice(documents, i, i + chunk_size - 1),
                chunk_id = #chunks + 1
            })
        end
        
        return chunks
    end
    
    -- Progress tracking
    self.progress = {
        total_chunks = 0,
        completed_chunks = 0
    }
end

function DistributedAnalysis:on_chunk_complete(chunk_id, result)
    self.progress.completed_chunks = self.progress.completed_chunks + 1
    
    self:emit("progress", {
        completed = self.progress.completed_chunks,
        total = self.progress.total_chunks,
        percentage = (self.progress.completed_chunks / self.progress.total_chunks) * 100
    })
end

-- Usage
local analyzer = DistributedAnalysis.new({
    workers = 4,
    mapper = DocumentAnalyzer.new(),
    reducer = ResultAggregator.new()
})

analyzer:on("progress", function(p)
    print(string.format("Progress: %.1f%%", p.percentage))
end)

local results = analyzer:run(large_document_set)
```

**Validation**:
- ✅ Extends built-in workflow
- ✅ Custom partitioning logic
- ✅ Progress tracking added
- ✅ Event emission
- ✅ Maintains base functionality

## 5. Performance and Scale Validation

### 5.1 High-Throughput Scenario

**Use Case**: Process 1000+ requests per minute

```javascript
// Connection pooling and request batching
const highThroughputAgent = new Agent({
    provider: new PooledProvider({
        baseProvider: "openai",
        poolSize: 10,
        queueSize: 1000
    }),
    batchingEnabled: true,
    batchSize: 20,
    batchDelay: 100 // ms
});

// Automatic batching of similar requests
const promises = [];
for (let i = 0; i < 1000; i++) {
    promises.push(
        highThroughputAgent.chat(`Question ${i}`)
    );
}

// Internally batched into ~50 API calls
const results = await Promise.all(promises);
```

**Validation**:
- ✅ Connection pooling
- ✅ Request batching
- ✅ Queue management
- ✅ Automatic optimization

### 5.2 Memory-Efficient Streaming

**Use Case**: Process large documents without loading entirely into memory

```lua
-- Streaming document processor
local processor = Agent.new({
    system_prompt = "Summarize documents",
    streaming = true
})

-- Process large file in chunks
local summary_parts = {}
local stream = FileStream.new("large_document.txt", { chunk_size = 1024 })

for chunk in stream do
    local partial = processor:stream_chat({
        message = "Summarize this section",
        context = { 
            chunk = chunk,
            previous_summary = summary_parts[#summary_parts]
        }
    })
    
    table.insert(summary_parts, partial)
    
    -- Free memory
    if #summary_parts > 10 then
        -- Consolidate older summaries
        local consolidated = processor:chat({
            message = "Consolidate these summaries",
            context = table.slice(summary_parts, 1, 5)
        })
        summary_parts = { consolidated, table.unpack(summary_parts, 6) }
    end
end
```

**Validation**:
- ✅ Streaming file processing
- ✅ Memory-bounded operation
- ✅ Progressive summarization
- ✅ Chunk consolidation

## 6. Error Scenarios

### 6.1 Graceful Degradation

**Use Case**: System continues functioning when services fail

```javascript
// Multi-provider fallback
const resilientAgent = new Agent({
    providers: [
        { name: "openai", priority: 1 },
        { name: "anthropic", priority: 2 },
        { name: "local_llm", priority: 3 }
    ],
    fallbackBehavior: "next_provider",
    
    onProviderFailure: (error, provider) => {
        console.error(`Provider ${provider} failed:`, error);
        metrics.increment("provider_failure", { provider });
    }
});

// Automatic fallback to next provider
const response = await resilientAgent.chat("Query");
```

**Validation**:
- ✅ Multi-provider support
- ✅ Automatic fallback
- ✅ Error tracking
- ✅ Graceful degradation

## Conclusion

The architecture successfully validates against:

1. **Simple Scenarios**: Tool execution is straightforward and safe
2. **Complex Workflows**: Multi-agent orchestration is flexible and powerful
3. **Event-Driven**: Hooks and events enable sophisticated automation
4. **Built-in Components**: Extensible and composable
5. **Performance**: Scales to high-throughput scenarios
6. **Reliability**: Graceful error handling and recovery

All use cases demonstrate that the architecture is:
- Practical for real-world applications
- Flexible enough for diverse scenarios
- Performant at scale
- Safe and reliable in production