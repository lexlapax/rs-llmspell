--[[
ABOUTME: Multi-agent pipeline coordination example
ABOUTME: Demonstrates sequential agent collaboration where each agent processes and enriches data

This example shows a research pipeline where multiple specialized agents work together:
1. Research Agent - Gathers initial information
2. Analysis Agent - Analyzes the gathered data
3. Summary Agent - Creates a concise summary of findings
]]

-- Create specialized agents (mock agents for demonstration)
local research_agent = Agent.create({
    name = "research_agent",
    model = "mock/research-v1",
    capabilities = {"web_search", "data_gathering"},
    system_prompt = "You are a research specialist focused on gathering comprehensive information."
})

local analysis_agent = Agent.create({
    name = "analysis_agent", 
    model = "mock/analyzer-v1",
    capabilities = {"data_analysis", "pattern_recognition"},
    system_prompt = "You are an analysis expert who identifies patterns and insights in data."
})

local summary_agent = Agent.create({
    name = "summary_agent",
    model = "mock/summarizer-v1", 
    capabilities = {"summarization", "key_point_extraction"},
    system_prompt = "You are a summarization expert who creates concise, actionable summaries."
})

-- Create a pipeline workflow where agents process data sequentially
local pipeline_workflow = Workflow.sequential({
    name = "research_pipeline",
    steps = {
        {
            name = "research_phase",
            agent = "research_agent",
            parameters = {
                task = "Research recent developments in quantum computing",
                output_format = "structured_data"
            }
        },
        {
            name = "analysis_phase",
            agent = "analysis_agent",
            parameters = {
                -- Reference output from previous step
                input = "$research_phase.output",
                analysis_type = "trend_analysis",
                focus_areas = {"commercial_applications", "technical_breakthroughs"}
            }
        },
        {
            name = "summary_phase",
            agent = "summary_agent",
            parameters = {
                -- Reference outputs from both previous steps
                research_data = "$research_phase.output",
                analysis_results = "$analysis_phase.output",
                summary_type = "executive_brief",
                max_length = 500
            }
        }
    },
    error_strategy = "stop" -- Stop pipeline if any agent fails
})

-- Execute the pipeline
print("Starting multi-agent research pipeline...")
local result = Workflow.execute(pipeline_workflow)

-- Display results from each phase
if result.success then
    print("\nPipeline completed successfully!")
    print("\nResearch Phase Output:")
    print(JSON.stringify(result.outputs.research_phase, 2))
    
    print("\nAnalysis Phase Output:")
    print(JSON.stringify(result.outputs.analysis_phase, 2))
    
    print("\nFinal Summary:")
    print(JSON.stringify(result.outputs.summary_phase, 2))
    
    print("\nExecution time: " .. result.execution_time .. "ms")
else
    print("\nPipeline failed: " .. result.error)
    print("Failed at step: " .. result.last_completed_step)
end

-- Example of creating a more complex pipeline with conditional routing
local adaptive_pipeline = Workflow.sequential({
    name = "adaptive_research_pipeline",
    steps = {
        {
            name = "initial_research",
            agent = "research_agent",
            parameters = { task = "Assess topic complexity", quick_scan = true }
        },
        {
            name = "route_decision",
            type = "conditional",
            condition = {
                type = "expression",
                expression = "$initial_research.output.complexity_score > 7"
            },
            then_branch = {
                -- High complexity: Use multiple specialized agents
                type = "parallel",
                branches = {
                    { agent = "technical_expert_agent", task = "deep_technical_analysis" },
                    { agent = "business_expert_agent", task = "market_analysis" },
                    { agent = "legal_expert_agent", task = "regulatory_analysis" }
                }
            },
            else_branch = {
                -- Low complexity: Use single analysis agent
                agent = "analysis_agent",
                parameters = { mode = "standard_analysis" }
            }
        },
        {
            name = "final_summary",
            agent = "summary_agent",
            parameters = { 
                inputs = "$previous_outputs",
                format = "comprehensive_report"
            }
        }
    }
})

return {
    simple_pipeline = pipeline_workflow,
    adaptive_pipeline = adaptive_pipeline,
    agents_created = { research_agent.id, analysis_agent.id, summary_agent.id }
}