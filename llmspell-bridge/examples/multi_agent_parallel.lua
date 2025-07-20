--[[
ABOUTME: Multi-agent parallel coordination example  
ABOUTME: Demonstrates fork-join pattern where agents work concurrently on different aspects

This example shows parallel agent coordination for comprehensive document analysis:
- Multiple specialist agents analyze different aspects simultaneously
- Results are aggregated by a coordinator agent
- Demonstrates efficiency gains from parallel processing
]]

-- Create specialized analysis agents
local sentiment_agent = Agent.create({
    name = "sentiment_analyzer",
    model = "mock/sentiment-v1",
    capabilities = {"sentiment_analysis", "emotion_detection"},
    system_prompt = "Analyze emotional tone and sentiment in text."
})

local fact_checker = Agent.create({
    name = "fact_checker", 
    model = "mock/factcheck-v1",
    capabilities = {"fact_verification", "source_validation"},
    system_prompt = "Verify facts and check sources for accuracy."
})

local style_analyzer = Agent.create({
    name = "style_analyzer",
    model = "mock/style-v1", 
    capabilities = {"writing_style_analysis", "readability_scoring"},
    system_prompt = "Analyze writing style, clarity, and readability."
})

local topic_extractor = Agent.create({
    name = "topic_extractor",
    model = "mock/topics-v1",
    capabilities = {"topic_modeling", "keyword_extraction"},
    system_prompt = "Extract main topics and key themes from text."
})

-- Coordinator agent to synthesize results
local coordinator = Agent.create({
    name = "analysis_coordinator",
    model = "mock/coordinator-v1",
    capabilities = {"result_synthesis", "report_generation"},
    system_prompt = "Synthesize multiple analysis results into coherent insights."
})

-- Sample document to analyze
local document = [[
The rapid advancement of artificial intelligence has transformed industries worldwide. 
Recent studies show that AI adoption has increased by 47% in the past year alone.
While this brings unprecedented opportunities for innovation and efficiency, it also
raises important questions about job displacement and ethical considerations.
Companies like TechCorp have reported 30% productivity gains after implementing
AI-driven automation, though critics argue these benefits come at a social cost.
]]

-- Create parallel analysis workflow
local parallel_analysis = Workflow.parallel({
    name = "document_analysis_parallel",
    steps = {
        {
            name = "sentiment_analysis",
            agent = "sentiment_analyzer",
            parameters = {
                text = document,
                granularity = "sentence_level",
                include_confidence = true
            }
        },
        {
            name = "fact_checking",
            agent = "fact_checker",
            parameters = {
                text = document,
                verify_statistics = true,
                check_sources = true
            }
        },
        {
            name = "style_analysis", 
            agent = "style_analyzer",
            parameters = {
                text = document,
                metrics = {"readability", "complexity", "tone"}
            }
        },
        {
            name = "topic_extraction",
            agent = "topic_extractor",
            parameters = {
                text = document,
                num_topics = 5,
                extract_entities = true
            }
        }
    },
    max_concurrency = 4, -- All agents work simultaneously
    fail_fast = false    -- Continue even if one agent fails
})

-- Execute parallel analysis
print("Starting parallel document analysis with 4 specialist agents...")
local start_time = os.clock()
local parallel_result = Workflow.execute(parallel_analysis)
local parallel_time = (os.clock() - start_time) * 1000

-- Now create a sequential workflow to coordinate and synthesize results
local synthesis_workflow = Workflow.sequential({
    name = "result_synthesis",
    steps = {
        {
            name = "coordinate_results",
            agent = "analysis_coordinator",
            parameters = {
                sentiment_data = parallel_result.outputs.sentiment_analysis,
                fact_check_data = parallel_result.outputs.fact_checking,
                style_data = parallel_result.outputs.style_analysis,
                topic_data = parallel_result.outputs.topic_extraction,
                synthesis_type = "comprehensive_report"
            }
        }
    }
})

-- Execute synthesis
local final_result = Workflow.execute(synthesis_workflow)

-- Display results
print("\n=== Parallel Analysis Results ===")
print("Execution time: " .. parallel_time .. "ms")
print("\nIndividual agent outputs:")

for step_name, output in pairs(parallel_result.outputs) do
    print("\n" .. step_name .. ":")
    print(JSON.stringify(output, 2))
end

print("\n=== Coordinated Synthesis ===")
if final_result.success then
    print(JSON.stringify(final_result.outputs.coordinate_results, 2))
else
    print("Synthesis failed: " .. final_result.error)
end

-- Compare with sequential execution time
print("\n=== Performance Comparison ===")
local sequential_time = 0
for step_name, _ in pairs(parallel_result.outputs) do
    -- Simulate sequential execution time (sum of individual times)
    sequential_time = sequential_time + (parallel_result.step_timings[step_name] or 250)
end

print("Parallel execution: " .. parallel_time .. "ms")
print("Estimated sequential: " .. sequential_time .. "ms")
print("Speedup factor: " .. string.format("%.2fx", sequential_time / parallel_time))

-- Advanced example: Dynamic fork-join based on initial analysis
local dynamic_analysis = Workflow.sequential({
    name = "dynamic_fork_join",
    steps = {
        {
            name = "quick_scan",
            agent = "topic_extractor",
            parameters = { text = document, mode = "quick" }
        },
        {
            name = "dynamic_fork",
            type = "conditional",
            condition = "$quick_scan.output.num_topics > 3",
            then_branch = {
                -- Complex document: Use all specialists
                type = "parallel",
                steps = {
                    { agent = "sentiment_analyzer" },
                    { agent = "fact_checker" },
                    { agent = "style_analyzer" },
                    { agent = "legal_analyzer" },    -- Additional specialist
                    { agent = "technical_analyzer" } -- Additional specialist
                }
            },
            else_branch = {
                -- Simple document: Use subset of specialists
                type = "parallel",
                steps = {
                    { agent = "sentiment_analyzer" },
                    { agent = "style_analyzer" }
                }
            }
        },
        {
            name = "final_synthesis",
            agent = "analysis_coordinator",
            parameters = { mode = "adaptive" }
        }
    }
})

return {
    parallel_workflow = parallel_analysis,
    synthesis_workflow = synthesis_workflow,
    dynamic_workflow = dynamic_analysis,
    performance_metrics = {
        parallel_time = parallel_time,
        sequential_estimate = sequential_time,
        speedup = sequential_time / parallel_time
    }
}