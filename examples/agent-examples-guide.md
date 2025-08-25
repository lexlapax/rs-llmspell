# Agent Examples Developer Guide

**Version**: Phase 3.3 Examples  
**Status**: ✅ **CURRENT** - Production-ready agent patterns  
**Last Updated**: July 2025

> **🔧 EXAMPLES GUIDE**: Comprehensive guide to agent implementation patterns, demonstrating real-world use cases with the 34 production tools and multi-agent coordination.

**🔗 Navigation**: [← Developer Guide](README.md) | [Documentation Hub](../README.md) | [Agent API](../user-guide/agent-api.md)

---

## Table of Contents

1. [Overview](#overview)
2. [Running Examples](#running-examples)
3. [Example Patterns](#example-patterns)
   - [Tool Orchestrator Agent](#1-tool-orchestrator-agent)
   - [Multi-Agent Coordinator](#2-multi-agent-coordinator)
   - [Monitoring Agent](#3-monitoring-agent)
   - [Data Pipeline Agent](#4-data-pipeline-agent)
   - [Research Agent](#5-research-agent)
   - [Code Generation Agent](#6-code-generation-agent)
   - [Decision-Making Agent](#7-decision-making-agent)
   - [Agent Library Catalog](#8-agent-library-catalog)
   - [Context-Aware Agent](#9-context-aware-agent)
   - [Performance Optimized Agent](#10-performance-optimized-agent)
4. [Common Patterns](#common-patterns)
5. [Best Practices](#best-practices)
6. [Testing Examples](#testing-examples)

## Overview

These examples demonstrate production-ready agent patterns using:
- ✅ Different agent architectures and coordination patterns
- ✅ Integration with all 34 standardized tools
- ✅ Multi-agent workflows and communication
- ✅ Performance optimization techniques
- ✅ Error handling and recovery strategies
- ✅ State management approaches

## Running Examples

### From Rust (Development)
```bash
# Run specific example
cargo run --example tool_orchestrator

# Run with logging
RUST_LOG=info cargo run --example multi_agent_coordinator

# Run all examples
for example in tool_orchestrator multi_agent_coordinator monitoring_agent; do
    cargo run --example $example
done
```

### From Lua Scripts (Production)
```lua
-- Load and run agent patterns
local agent_patterns = require("agent_patterns")

-- Use tool orchestrator pattern
local orchestrator = agent_patterns.create_tool_orchestrator({
    name = "data_processor",
    tools = {"file_operations", "json_processor", "csv_analyzer"}
})

local result = orchestrator:execute({
    task = "Process sales data",
    input_file = "sales.csv"
})
```

## Example Patterns

### 1. Tool Orchestrator Agent

**Purpose**: Coordinate multiple tools to solve complex tasks through intelligent sequencing.

**Use Cases**:
- Data processing pipelines
- Multi-step file transformations
- Complex analysis workflows

**Implementation Pattern**:
```rust
// Rust implementation structure
pub struct ToolOrchestratorAgent {
    tool_manager: Arc<ToolManager>,
    execution_plan: ExecutionPlan,
    error_strategy: ErrorStrategy,
}

impl Agent for ToolOrchestratorAgent {
    async fn execute(&self, input: AgentInput) -> Result<AgentOutput> {
        // 1. Analyze task requirements
        let required_tools = self.analyze_requirements(&input)?;
        
        // 2. Create execution plan
        let plan = self.create_execution_plan(required_tools)?;
        
        // 3. Execute tools in sequence with error handling
        let mut context = ExecutionContext::new();
        for step in plan.steps {
            match self.execute_tool_step(&step, &mut context).await {
                Ok(result) => context.add_result(step.name, result),
                Err(e) => self.handle_error(e, &step, &mut context)?,
            }
        }
        
        // 4. Aggregate and return results
        Ok(self.aggregate_results(context))
    }
}
```

**Lua Usage Example**:
```lua
-- ✅ WORKING EXAMPLE
local orchestrator = Agent.create({
    model = "openai/gpt-4",
    system_prompt = "You are a data processing orchestrator. Use tools efficiently.",
    tools = {"file_operations", "json_processor", "csv_analyzer"}
})

local result = orchestrator:execute({
    prompt = "Read sales.csv, analyze the data, and create a JSON summary"
})

-- The agent will automatically:
-- 1. Use file_operations to read the CSV
-- 2. Use csv_analyzer to process the data
-- 3. Use json_processor to format the output
```

### 2. Multi-Agent Coordinator

**Purpose**: Coordinate multiple specialized agents working together on complex tasks.

**Use Cases**:
- Research projects with specialized domains
- Complex decision-making requiring multiple perspectives
- Hierarchical task decomposition

**Coordination Patterns**:

#### Hierarchical Pattern
```lua
-- ✅ WORKING EXAMPLE
local coordinator = Agent.create({
    model = "anthropic/claude-3",
    system_prompt = "You coordinate specialist agents to complete complex tasks."
})

local specialists = {
    researcher = Agent.create({
        model = "openai/gpt-4",
        system_prompt = "You are a research specialist."
    }),
    analyst = Agent.create({
        model = "openai/gpt-4", 
        system_prompt = "You analyze data and find patterns."
    }),
    writer = Agent.create({
        model = "openai/gpt-4",
        system_prompt = "You write clear, concise reports."
    })
}

-- Coordinator delegates tasks
local workflow = Workflow.sequential({
    name = "research_project",
    steps = {
        {type = "agent", id = "coordinator", params = {task = "plan_research"}},
        {type = "agent", id = "researcher", params = {task = "gather_data"}},
        {type = "agent", id = "analyst", params = {task = "analyze_findings"}},
        {type = "agent", id = "writer", params = {task = "write_report"}}
    }
})
```

#### Consensus Pattern
```lua
-- ✅ WORKING EXAMPLE
local consensus_workflow = Workflow.multiAgentConsensus({
    name = "investment_decision",
    evaluators = {"risk_analyst", "market_expert", "tech_specialist"},
    consensus_threshold = 0.66,
    options = {
        {id = "startup_a", description = "AI healthcare startup"},
        {id = "startup_b", description = "Green energy platform"}
    }
})
```

### 3. Monitoring Agent

**Purpose**: Monitor system health, agent performance, and trigger alerts.

**Use Cases**:
- System observability
- Performance tracking
- Anomaly detection
- Alert generation

**Implementation**:
```lua
-- ✅ WORKING EXAMPLE
local monitor = Agent.create({
    model = "openai/gpt-3.5-turbo",  -- Lightweight model for monitoring
    system_prompt = [[
        You are a system monitor. Track metrics, identify anomalies, 
        and generate alerts for critical issues.
    ]],
    tools = {"system_monitor", "service_checker"}
})

-- Monitoring loop
local function monitoring_loop()
    while true do
        -- Collect metrics
        local metrics = Tool.get("system_monitor"):execute({
            operation = "get_metrics"
        })
        
        -- Check for anomalies
        local analysis = monitor:execute({
            prompt = "Analyze these metrics for anomalies: " .. 
                     JSON.stringify(metrics.output)
        })
        
        -- Generate alerts if needed
        if analysis.output:match("ALERT") then
            Logger.error("System alert", {
                analysis = analysis.output,
                metrics = metrics.output
            })
        end
        
        -- Wait before next check
        os.execute("sleep 60")
    end
end
```

### 4. Data Pipeline Agent

**Purpose**: Create intelligent ETL pipelines with decision-making capabilities.

**Use Cases**:
- Data transformation workflows
- ETL operations
- Data quality validation
- Adaptive processing based on data characteristics

**Pipeline Example**:
```lua
-- ✅ WORKING EXAMPLE
local pipeline_agent = Agent.create({
    model = "openai/gpt-4",
    system_prompt = [[
        You manage data pipelines. Analyze data quality, 
        choose appropriate transformations, and ensure data integrity.
    ]],
    tools = {"csv_analyzer", "json_processor", "data_validation", "file_operations"}
})

-- Adaptive pipeline that adjusts based on data
local function process_data_file(filepath)
    -- Analyze file first
    local analysis = Tool.get("csv_analyzer"):execute({
        operation = "analyze",
        path = filepath
    })
    
    -- Let agent decide processing strategy
    local strategy = pipeline_agent:execute({
        prompt = string.format([[
            Based on this CSV analysis, create a processing strategy:
            %s
            
            Consider: data quality, missing values, format issues
        ]], JSON.stringify(analysis.output))
    })
    
    -- Execute the strategy
    -- Agent will use tools to clean, transform, and validate data
    return pipeline_agent:execute({
        prompt = "Execute the data processing strategy on " .. filepath
    })
end
```

### 5. Research Agent

**Purpose**: Conduct comprehensive research using web and local resources.

**Use Cases**:
- Market research
- Academic research
- Competitive analysis
- Report generation

**Research Workflow**:
```lua
-- ✅ WORKING EXAMPLE
local research_agent = Agent.create({
    model = "anthropic/claude-3",
    system_prompt = [[
        You are a research specialist. Gather information from multiple sources,
        verify facts, synthesize findings, and create comprehensive reports.
    ]],
    tools = {"web_search", "web_scraper", "file_operations", "text_manipulator"}
})

local research_workflow = Workflow.sequential({
    name = "comprehensive_research",
    steps = {
        -- Phase 1: Initial search
        {
            type = "tool",
            name = "web_search",
            params = {input = "latest AI safety research 2025"}
        },
        -- Phase 2: Deep dive on top results
        {
            type = "agent",
            id = "research_agent",
            params = {task = "analyze_and_expand_search"}
        },
        -- Phase 3: Fact verification
        {
            type = "parallel",
            branches = {
                {type = "tool", name = "web_scraper", params = {verify_sources = true}},
                {type = "agent", id = "fact_checker", params = {cross_reference = true}}
            }
        },
        -- Phase 4: Report generation
        {
            type = "agent",
            id = "research_agent",
            params = {task = "synthesize_findings_into_report"}
        }
    }
})
```

### 6. Code Generation Agent

**Purpose**: Generate, test, and refine code through iterative development.

**Use Cases**:
- Automated code generation
- Test-driven development
- Code refactoring
- Bug fixing

**Generate-Test-Refine Pattern**:
```lua
-- ✅ WORKING EXAMPLE
local code_gen_agent = Agent.create({
    model = "openai/gpt-4",
    system_prompt = [[
        You are an expert programmer. Generate clean, tested code.
        Follow TDD principles: write tests first, then implementation.
    ]],
    tools = {"file_operations", "process_executor", "text_manipulator"}
})

local function generate_tested_code(requirements)
    -- Step 1: Generate tests
    local test_code = code_gen_agent:execute({
        prompt = "Write comprehensive tests for: " .. requirements
    })
    
    -- Save test file
    Tool.get("file_operations"):execute({
        operation = "write",
        path = "test_generated.lua",
        content = test_code.output
    })
    
    -- Step 2: Generate implementation
    local implementation = code_gen_agent:execute({
        prompt = "Implement code to pass these tests: " .. test_code.output
    })
    
    -- Step 3: Run tests and refine
    local test_result = Tool.get("process_executor"):execute({
        operation = "run",
        command = "lua test_generated.lua"
    })
    
    if not test_result.output:match("PASS") then
        -- Refine based on test failures
        implementation = code_gen_agent:execute({
            prompt = string.format([[
                The tests failed with: %s
                Fix the implementation: %s
            ]], test_result.output, implementation.output)
        })
    end
    
    return implementation
end
```

### 7. Decision-Making Agent

**Purpose**: Make complex decisions using multiple criteria and confidence scoring.

**Use Cases**:
- Business strategy decisions
- Investment analysis
- Risk assessment
- Resource allocation

**Multi-Criteria Decision Pattern**:
```lua
-- ✅ WORKING EXAMPLE
local decision_agent = Agent.create({
    model = "anthropic/claude-3",
    system_prompt = [[
        You are a decision analyst. Evaluate options using multiple criteria,
        assign weights, calculate scores, and provide confidence levels.
    ]]
})

local function make_decision(options, criteria)
    -- Structured decision matrix
    local decision_matrix = {
        options = options,
        criteria = criteria,
        weights = {cost = 0.3, risk = 0.3, potential = 0.4}
    }
    
    -- Evaluate each option
    local evaluation = decision_agent:execute({
        prompt = string.format([[
            Evaluate these options using the decision matrix:
            %s
            
            For each option, score each criterion (1-10) and calculate weighted score.
            Provide confidence level (0-1) for each evaluation.
        ]], JSON.stringify(decision_matrix))
    })
    
    -- Parse results and find best option
    return {
        evaluation = evaluation.output,
        recommended = extract_recommendation(evaluation.output),
        confidence = extract_confidence(evaluation.output)
    }
end
```

### 8. Agent Library Catalog

**Purpose**: Create reusable agent templates and manage agent libraries.

**Use Cases**:
- Agent template management
- Configuration-driven agents
- Agent factory patterns
- Reusable agent components

**Template Pattern**:
```lua
-- ✅ WORKING EXAMPLE
local AgentLibrary = {}

-- Define reusable templates
AgentLibrary.templates = {
    researcher = {
        model = "anthropic/claude-3",
        system_prompt = "You are a research specialist...",
        default_tools = {"web_search", "file_operations"}
    },
    analyst = {
        model = "openai/gpt-4",
        system_prompt = "You analyze data and find patterns...",
        default_tools = {"csv_analyzer", "json_processor"}
    },
    writer = {
        model = "openai/gpt-4",
        system_prompt = "You write clear, engaging content...",
        default_tools = {"text_manipulator", "file_operations"}
    }
}

-- Factory function
function AgentLibrary.create_from_template(template_name, customizations)
    local template = AgentLibrary.templates[template_name]
    if not template then
        error("Unknown template: " .. template_name)
    end
    
    -- Merge customizations
    local config = {
        model = customizations.model or template.model,
        system_prompt = template.system_prompt .. (customizations.prompt_suffix or ""),
        tools = customizations.tools or template.default_tools
    }
    
    return Agent.create(config)
end

-- Usage
local custom_researcher = AgentLibrary.create_from_template("researcher", {
    prompt_suffix = " Focus on AI safety research.",
    tools = {"web_search", "web_scraper", "file_operations"}
})
```

### 9. Context-Aware Agent

**Purpose**: Maintain and utilize context across conversations and tasks.

**Use Cases**:
- Conversational agents
- Long-running tasks
- Stateful workflows
- Memory-enhanced agents

**Context Management Pattern**:
```lua
-- ✅ WORKING EXAMPLE
local ContextAwareAgent = {}

function ContextAwareAgent.create(config)
    local agent = Agent.create(config)
    local context_key = "agent_context_" .. config.name
    
    -- Initialize context
    State.set(context_key, {
        conversation_history = {},
        learned_facts = {},
        user_preferences = {},
        task_memory = {}
    })
    
    -- Wrap execute to include context
    local original_execute = agent.execute
    agent.execute = function(self, input)
        local context = State.get(context_key)
        
        -- Add context to prompt
        local enriched_input = {
            prompt = string.format([[
                Context:
                - Previous conversations: %d
                - Known facts: %d
                - Current task memory: %s
                
                User input: %s
            ]], 
                #context.conversation_history,
                #context.learned_facts,
                JSON.stringify(context.task_memory),
                input.prompt
            )
        }
        
        -- Execute with context
        local result = original_execute(self, enriched_input)
        
        -- Update context
        table.insert(context.conversation_history, {
            input = input.prompt,
            output = result.output,
            timestamp = os.time()
        })
        
        -- Extract and store new facts
        local facts = extract_facts(result.output)
        for _, fact in ipairs(facts) do
            table.insert(context.learned_facts, fact)
        end
        
        State.set(context_key, context)
        return result
    end
    
    return agent
end
```

### 10. Performance Optimized Agent

**Purpose**: Build high-performance agents for throughput-critical applications.

**Use Cases**:
- High-volume data processing
- Real-time analysis
- Batch operations
- Resource-constrained environments

**Optimization Patterns**:
```lua
-- ✅ WORKING EXAMPLE
local PerformanceAgent = {}

function PerformanceAgent.create(config)
    local agent = Agent.create(config)
    
    -- Add caching layer
    local cache_key = "perf_cache_" .. config.name
    State.set(cache_key, {})
    
    -- Add batching capability
    local batch_queue = {}
    local batch_size = config.batch_size or 10
    
    -- Optimized execute with caching and batching
    agent.execute_optimized = function(self, input)
        -- Check cache first
        local cache = State.get(cache_key)
        local input_hash = hash_input(input)
        
        if cache[input_hash] then
            Logger.debug("Cache hit", {hash = input_hash})
            return cache[input_hash]
        end
        
        -- Add to batch queue
        table.insert(batch_queue, input)
        
        -- Process batch if full
        if #batch_queue >= batch_size then
            local results = process_batch(self, batch_queue)
            batch_queue = {}
            
            -- Cache results
            for i, result in ipairs(results) do
                local hash = hash_input(batch_queue[i])
                cache[hash] = result
            end
            State.set(cache_key, cache)
            
            return results[#results]  -- Return last result
        end
        
        -- For immediate processing
        local result = self:execute(input)
        cache[input_hash] = result
        State.set(cache_key, cache)
        
        return result
    end
    
    -- Parallel tool execution
    agent.execute_parallel_tools = function(self, tool_calls)
        local workflow = Workflow.parallel({
            name = "parallel_tools",
            branches = {}
        })
        
        for _, call in ipairs(tool_calls) do
            table.insert(workflow.branches, {
                type = "tool",
                name = call.tool,
                params = call.params
            })
        end
        
        return Workflow.execute(workflow, {})
    end
    
    return agent
end
```

## Common Patterns

### Error Handling Strategy
```lua
-- Implement retry with exponential backoff
local function retry_with_backoff(fn, max_retries)
    max_retries = max_retries or 3
    local delay = 1000  -- Start with 1 second
    
    for attempt = 1, max_retries do
        local success, result = pcall(fn)
        if success then
            return result
        end
        
        if attempt < max_retries then
            Logger.warn("Attempt failed, retrying", {
                attempt = attempt,
                delay_ms = delay,
                error = result
            })
            os.execute("sleep " .. (delay / 1000))
            delay = delay * 2  -- Exponential backoff
        else
            error("All retry attempts failed: " .. tostring(result))
        end
    end
end
```

### Resource Management
```lua
-- Resource pooling for tools
local ToolPool = {}

function ToolPool.create(tool_name, pool_size)
    local pool = {
        available = {},
        in_use = {},
        tool_name = tool_name
    }
    
    -- Initialize pool
    for i = 1, pool_size do
        table.insert(pool.available, Tool.get(tool_name))
    end
    
    function pool:acquire()
        if #self.available == 0 then
            -- Wait or create new instance
            Logger.warn("Tool pool exhausted", {tool = self.tool_name})
            return Tool.get(self.tool_name)  -- Fallback
        end
        
        local tool = table.remove(self.available)
        self.in_use[tool] = true
        return tool
    end
    
    function pool:release(tool)
        if self.in_use[tool] then
            self.in_use[tool] = nil
            table.insert(self.available, tool)
        end
    end
    
    return pool
end
```

### State Persistence
```lua
-- Checkpoint state for long-running agents
local function checkpoint_agent_state(agent_id, state)
    local checkpoint = {
        agent_id = agent_id,
        timestamp = os.time(),
        state = state,
        version = "1.0"
    }
    
    Tool.get("file_operations"):execute({
        operation = "write",
        path = string.format("/tmp/agent_checkpoint_%s.json", agent_id),
        content = JSON.stringify(checkpoint)
    })
end

local function restore_agent_state(agent_id)
    local result = Tool.get("file_operations"):execute({
        operation = "read",
        path = string.format("/tmp/agent_checkpoint_%s.json", agent_id)
    })
    
    if result.success then
        return JSON.parse(result.output)
    end
    return nil
end
```

## Best Practices

### 1. Agent Design
- **Single Responsibility**: Each agent should have a clear, focused purpose
- **Composability**: Design agents that can work together
- **Stateless When Possible**: Use external state management for persistence

### 2. Tool Usage
- **Validate Before Use**: Check tool availability and parameters
- **Handle Tool Failures**: Always have fallback strategies
- **Batch Operations**: Group similar tool calls for efficiency

### 3. Performance
- **Cache Expensive Operations**: Use State API for caching
- **Parallel Execution**: Use workflow patterns for concurrent operations
- **Resource Limits**: Set appropriate timeouts and memory limits

### 4. Testing
- **Unit Test Agent Logic**: Test decision-making separately from execution
- **Mock External Dependencies**: Use mock tools for testing
- **Performance Benchmarks**: Monitor agent execution times

## Testing Examples

### Unit Testing Pattern
```lua
-- Test agent logic without actual LLM calls
local function test_agent_logic()
    local mock_agent = {
        execute = function(self, input)
            -- Deterministic responses for testing
            if input.prompt:match("analyze") then
                return {output = "Analysis complete: positive trends"}
            elseif input.prompt:match("summarize") then
                return {output = "Summary: key points extracted"}
            end
        end
    }
    
    -- Test orchestration logic
    local result = orchestrate_task(mock_agent, "analyze and summarize data")
    assert(result:match("Analysis complete"))
    assert(result:match("Summary"))
end
```

### Integration Testing
```lua
-- Test with real tools but mock LLM
local function integration_test()
    local test_file = "/tmp/test_data.json"
    
    -- Create test data
    Tool.get("file_operations"):execute({
        operation = "write",
        path = test_file,
        content = JSON.stringify({test = "data"})
    })
    
    -- Test agent with real tools
    local agent = create_test_agent()
    local result = agent:process_file(test_file)
    
    -- Verify results
    assert(result.success)
    assert(result.output.processed)
    
    -- Cleanup
    Tool.get("file_operations"):execute({
        operation = "delete",
        path = test_file
    })
end
```

## Contributing New Examples

When adding new agent examples:

1. **Demonstrate Unique Patterns**: Show something not covered by existing examples
2. **Include Documentation**: Explain the pattern and use cases
3. **Provide Both Rust and Lua**: Show implementation and usage
4. **Add Tests**: Include unit and integration tests
5. **Update This Guide**: Add your example to the appropriate section

---

**See Also**:
- [Agent API Reference](../user-guide/agent-api.md) - Complete API documentation
- [Tool Reference](../user-guide/tool-reference.md) - Available tools
- [Workflow Patterns](workflow-bridge-guide.md) - Multi-agent workflows
- [Example Scripts](../../examples/lua/agents/) - Runnable Lua examples