-- ============================================================
-- LLMSPELL ADVANCED PATTERNS SHOWCASE
-- ============================================================
-- Pattern ID: 01 - Multi-Agent Orchestration v0.7.0
-- Complexity Level: ADVANCED
-- Real-World Use Case: Complex AI systems with specialized agents
-- Pattern Category: Agent Orchestration
--
-- Purpose: Demonstrates complex multi-agent coordination patterns
-- Architecture: Multiple specialized agents with delegation and error recovery
-- Key Capabilities:
--   • Agent specialization and role definition
--   • Task delegation between agents
--   • Error recovery and fallback strategies
--   • Performance monitoring across agents
--   • Agent communication patterns
--
-- Prerequisites:
--   • API keys: OPENAI_API_KEY or ANTHROPIC_API_KEY
--   • Understanding of agent basics (see features/agent-basics.lua)
--
-- HOW TO RUN:
-- OPENAI_API_KEY=$OPENAI_API_KEY ./target/debug/llmspell run examples/script-users/advanced-patterns/multi-agent-orchestration.lua
--
-- EXPECTED OUTPUT:
-- Multiple agents coordinating on complex tasks with delegation
-- Execution time: 15-30 seconds with API calls
--
-- Time to Complete: 30 seconds
-- Next Steps: See cookbook/research-assistant.lua for production example
-- ============================================================

print("=== Multi-Agent Orchestration Patterns ===\n")

-- Helper function for safe agent creation
local function create_agent(config)
    local success, agent = pcall(function()
        return Agent.builder()
            :name(config.name)
            :model(config.model or "openai/gpt-3.5-turbo")
            :temperature(config.temperature or 0.7)
            :max_tokens(config.max_tokens or 500)
            :system_prompt(config.system_prompt)
            :build()
    end)
    if success then
        return agent
    else
        print("   ✗ Failed to create " .. config.name .. ": " .. tostring(agent))
        return nil
    end
end

-- 1. CREATE SPECIALIZED AGENTS
print("1. Creating Specialized Agent Team")
print("-" .. string.rep("-", 34))

-- Research Agent - Information gathering
local researcher = create_agent({
    name = "researcher",
    temperature = 0.3,
    system_prompt = [[You are a research specialist. Your role is to:
1. Gather relevant information
2. Verify facts and sources
3. Summarize findings concisely
Always be accurate and cite your reasoning.]]
})

-- Analyst Agent - Data processing and insights
local analyst = create_agent({
    name = "analyst", 
    temperature = 0.4,
    system_prompt = [[You are a data analyst. Your role is to:
1. Process and analyze information
2. Identify patterns and trends
3. Generate actionable insights
Focus on quantitative analysis when possible.]]
})

-- Writer Agent - Content generation
local writer = create_agent({
    name = "writer",
    temperature = 0.8,
    system_prompt = [[You are a content writer. Your role is to:
1. Create clear, engaging content
2. Adapt tone for the audience
3. Structure information logically
Make complex topics accessible.]]
})

-- Coordinator Agent - Task orchestration
local coordinator = create_agent({
    name = "coordinator",
    model = "openai/gpt-4o-mini",
    temperature = 0.5,
    max_tokens = 1000,
    system_prompt = [[You are the team coordinator. Your role is to:
1. Break down complex tasks into subtasks
2. Delegate to appropriate specialists
3. Synthesize results from multiple agents
4. Ensure quality and completeness
You coordinate between researcher, analyst, and writer agents.]]
})

-- Quality Agent - Review and validation
local quality = create_agent({
    name = "quality_checker",
    temperature = 0.2,
    system_prompt = [[You are a quality assurance specialist. Your role is to:
1. Review work for accuracy and completeness
2. Identify errors or inconsistencies
3. Suggest improvements
4. Validate against requirements
Be thorough and critical in your review.]]
})

local agent_count = 0
for _, agent in ipairs({researcher, analyst, writer, coordinator, quality}) do
    if agent then agent_count = agent_count + 1 end
end
print("   ✓ Created " .. agent_count .. " specialized agents")

-- 2. DELEGATION PATTERN
print("\n2. Task Delegation Pattern")
print("-" .. string.rep("-", 25))

if coordinator and researcher and analyst then
    -- Complex task requiring multiple agents
    local task = "Analyze the impact of AI on software development productivity"
    
    print("   Task: " .. task)
    print("   Delegating to team...")
    
    -- Step 1: Coordinator plans approach
    local plan_response = coordinator:execute({
        text = "Create a research plan for: " .. task .. ". List 3 specific areas to investigate."
    })
    
    if plan_response and plan_response.text then
        print("   Coordinator's plan created")
        
        -- Step 2: Researcher gathers information
        local research_response = researcher:execute({
            text = "Research this topic: " .. task .. ". Focus on: productivity metrics, tool adoption, and developer satisfaction."
        })
        
        if research_response and research_response.text then
            print("   Research completed")
            
            -- Step 3: Analyst processes findings
            local analysis_response = analyst:execute({
                text = "Analyze these research findings and identify key trends: " .. research_response.text:sub(1, 500)
            })
            
            if analysis_response and analysis_response.text then
                print("   Analysis completed")
                print("   ✓ Multi-agent delegation successful")
            end
        end
    end
else
    print("   ✗ Insufficient agents for delegation")
end

-- 3. CONSENSUS PATTERN
print("\n3. Agent Consensus Pattern") 
print("-" .. string.rep("-", 26))

if researcher and analyst and writer then
    local question = "What is the most important programming paradigm?"
    
    print("   Question: " .. question)
    print("   Gathering perspectives...")
    
    -- Get perspectives from different agents
    local perspectives = {}
    
    local r_response = researcher:execute({text = question .. " Provide a research-based answer."})
    if r_response and r_response.text then
        perspectives.researcher = r_response.text:sub(1, 200)
        print("   ✓ Researcher perspective gathered")
    end
    
    local a_response = analyst:execute({text = question .. " Provide an analytical answer."})
    if a_response and a_response.text then
        perspectives.analyst = a_response.text:sub(1, 200)
        print("   ✓ Analyst perspective gathered")
    end
    
    local w_response = writer:execute({text = question .. " Provide a creative answer."})
    if w_response and w_response.text then
        perspectives.writer = w_response.text:sub(1, 200)
        print("   ✓ Writer perspective gathered")
    end
    
    -- Synthesize consensus
    if coordinator and next(perspectives) then
        local synthesis = coordinator:execute({
            text = "Synthesize these perspectives into a consensus: " .. 
                   "Researcher: " .. (perspectives.researcher or "N/A") .. 
                   " Analyst: " .. (perspectives.analyst or "N/A") ..
                   " Writer: " .. (perspectives.writer or "N/A")
        })
        if synthesis and synthesis.text then
            print("   ✓ Consensus achieved")
        end
    end
else
    print("   ✗ Insufficient agents for consensus")
end

-- 4. ERROR RECOVERY PATTERN
print("\n4. Error Recovery with Fallback Agents")
print("-" .. string.rep("-", 38))

-- Simulate primary agent failure
local primary_task = "Calculate the fibonacci sequence up to 10"
print("   Task: " .. primary_task)

-- Try primary agent (simulate failure with invalid prompt)
local primary_success = false
if analyst then
    local result = analyst:execute({
        text = primary_task .. " [This might fail - testing recovery]"
    })
    primary_success = result and result.text and not result.text:match("error")
end

if not primary_success then
    print("   Primary agent failed, activating fallback...")
    
    -- Fallback to simpler agent
    if researcher then
        local fallback_result = researcher:execute({
            text = "Help with this task: " .. primary_task .. ". Provide step-by-step calculation."
        })
        if fallback_result and fallback_result.text then
            print("   ✓ Fallback agent succeeded")
        else
            print("   ✗ Fallback also failed")
        end
    end
else
    print("   ✓ Primary agent succeeded")
end

-- 5. PIPELINE PATTERN
print("\n5. Agent Pipeline Pattern")
print("-" .. string.rep("-", 24))

if researcher and analyst and writer and quality then
    local topic = "Benefits of test-driven development"
    print("   Topic: " .. topic)
    print("   Running through pipeline...")
    
    -- Pipeline: Research → Analyze → Write → Review
    local pipeline_data = topic
    local stage_results = {}
    
    -- Stage 1: Research
    local research = researcher:execute({text = "Research: " .. pipeline_data})
    if research and research.text then
        pipeline_data = research.text:sub(1, 300)
        stage_results.research = "✓"
    end
    
    -- Stage 2: Analyze
    local analysis = analyst:execute({text = "Analyze this research: " .. pipeline_data})
    if analysis and analysis.text then
        pipeline_data = analysis.text:sub(1, 300)
        stage_results.analysis = "✓"
    end
    
    -- Stage 3: Write
    local content = writer:execute({text = "Write a brief summary about: " .. pipeline_data})
    if content and content.text then
        pipeline_data = content.text:sub(1, 300)
        stage_results.writing = "✓"
    end
    
    -- Stage 4: Quality Review
    local review = quality:execute({text = "Review this content for quality: " .. pipeline_data})
    if review and review.text then
        stage_results.review = "✓"
    end
    
    print("   Pipeline stages: Research[" .. (stage_results.research or "✗") .. 
          "] → Analyze[" .. (stage_results.analysis or "✗") ..
          "] → Write[" .. (stage_results.writing or "✗") ..
          "] → Review[" .. (stage_results.review or "✗") .. "]")
else
    print("   ✗ Insufficient agents for pipeline")
end

-- 6. PARALLEL PROCESSING PATTERN
print("\n6. Parallel Agent Processing")
print("-" .. string.rep("-", 28))

if researcher and analyst and writer then
    local topic = "Future of programming languages"
    print("   Topic: " .. topic)
    print("   Launching parallel analysis...")
    
    -- All agents work on the same topic simultaneously
    local start_time = os.clock()
    
    -- Simulate parallel execution (in practice, would use async)
    local parallel_results = {}
    
    local r_result = researcher:execute({text = "Research trends in: " .. topic})
    if r_result and r_result.text then
        parallel_results.research = true
    end
    
    local a_result = analyst:execute({text = "Analyze market data for: " .. topic})
    if a_result and a_result.text then
        parallel_results.analysis = true
    end
    
    local w_result = writer:execute({text = "Write predictions about: " .. topic})
    if w_result and w_result.text then
        parallel_results.writing = true
    end
    
    local duration = (os.clock() - start_time) * 1000
    
    local completed = 0
    for _, v in pairs(parallel_results) do
        if v then completed = completed + 1 end
    end
    
    print(string.format("   ✓ Parallel execution: %d agents completed in %.2f ms", completed, duration))
else
    print("   ✗ Insufficient agents for parallel processing")
end

-- 7. PERFORMANCE MONITORING
print("\n7. Multi-Agent Performance Monitoring")
print("-" .. string.rep("-", 37))

local metrics = {
    agents_created = agent_count,
    successful_executions = 0,
    failed_executions = 0,
    total_duration = 0
}

-- Quick performance test
if coordinator then
    local test_start = os.clock()
    local test_result = coordinator:execute({
        text = "Quickly respond: What is 2+2?"
    })
    local test_duration = (os.clock() - test_start) * 1000
    
    if test_result and test_result.text then
        metrics.successful_executions = metrics.successful_executions + 1
        metrics.total_duration = test_duration
        print(string.format("   Response time: %.2f ms", test_duration))
    else
        metrics.failed_executions = metrics.failed_executions + 1
    end
end

print("   Metrics:")
print("   • Agents created: " .. metrics.agents_created)
print("   • Successful executions: " .. metrics.successful_executions)
print("   • Failed executions: " .. metrics.failed_executions)
if metrics.total_duration > 0 then
    print(string.format("   • Avg response time: %.2f ms", metrics.total_duration))
end

-- 8. BEST PRACTICES
print("\n8. Multi-Agent Best Practices")
print("-" .. string.rep("-", 29))

print("   • Define clear agent roles and responsibilities")
print("   • Use appropriate temperature for each agent's task")
print("   • Implement fallback strategies for critical paths")
print("   • Monitor performance across all agents")
print("   • Design communication protocols between agents")
print("   • Handle partial failures gracefully")
print("   • Use coordinator agents for complex orchestration")
print("   • Cache agent results when appropriate")

print("\n=== Multi-Agent Orchestration Complete ===")
print("Demonstrated: Delegation, Consensus, Recovery, Pipeline, Parallel patterns")
print("Next: Explore complex-workflows.lua for workflow orchestration")