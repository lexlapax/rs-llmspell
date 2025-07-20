-- ABOUTME: Advanced example showing agent and workflow integration
-- ABOUTME: Demonstrates multi-agent coordination using workflows and state

-- Setup: Initialize shared state for agent coordination
State.set("task_queue", {})
State.set("results", {})

print("=== Multi-Agent Workflow Example ===")

-- Step 1: Create specialized agents
local analyst_agent = Agent.create({
    name = "data_analyst",
    provider = "mock",
    model = "mock-model",
    system_prompt = "You are a data analyst. Analyze data and provide insights."
})

local writer_agent = Agent.create({
    name = "content_writer", 
    provider = "mock",
    model = "mock-model",
    system_prompt = "You are a content writer. Create clear, engaging content."
})

print("Created agents:", analyst_agent.id, writer_agent.id)

-- Step 2: Create a workflow that coordinates agents
local report_workflow = Workflow.sequential({
    name = "automated_report_generation",
    description = "Analyzes data and generates a report",
    steps = {
        -- Step 1: Use calculator tool to process data
        {
            name = "calculate_metrics",
            tool = "calculator",
            parameters = {
                operation = "multiply",
                a = 150,
                b = 0.85  -- 85% efficiency
            }
        },
        -- Step 2: Store calculation in state
        {
            name = "store_metrics",
            tool = "state_manager",  -- Hypothetical tool wrapping State global
            parameters = {
                action = "set",
                key = "efficiency_score",
                value = "$calculate_metrics.result"
            }
        },
        -- Step 3: Analyze with agent (would use real agent in production)
        {
            name = "analyze_data",
            tool = "agent_executor",  -- Hypothetical tool wrapping Agent
            parameters = {
                agent_id = analyst_agent.id,
                prompt = "Analyze efficiency score: $calculate_metrics.result"
            }
        },
        -- Step 4: Generate report with writer agent
        {
            name = "write_report",
            tool = "agent_executor",
            parameters = {
                agent_id = writer_agent.id,
                prompt = "Write a brief report about: $analyze_data.output"
            }
        }
    }
})

-- Step 3: Execute the workflow
print("\nExecuting report generation workflow...")
local report_result = Workflow.execute(report_workflow)

-- Step 4: Store final report in state
State.set("latest_report", {
    timestamp = os.date(),
    content = report_result.output,
    workflow_id = report_workflow.id
})

print("Report generated and stored!")

-- Step 5: Demonstrate conditional workflow based on state
local review_workflow = Workflow.conditional({
    name = "report_review",
    description = "Reviews and potentially revises the report",
    condition = {
        check = function()
            local report = State.get("latest_report")
            -- In real scenario, would check report quality
            return report ~= nil
        end,
        true_branch = {
            name = "approve_report",
            tool = "logger",
            parameters = {
                message = "Report approved for distribution"
            }
        },
        false_branch = {
            name = "request_revision",
            tool = "logger", 
            parameters = {
                message = "Report needs revision"
            }
        }
    }
})

-- Step 6: Create a parallel workflow for multi-agent analysis
local parallel_analysis = Workflow.parallel({
    name = "multi_perspective_analysis",
    description = "Multiple agents analyze the same data simultaneously",
    branches = {
        {
            name = "technical_analysis",
            tool = "agent_executor",
            parameters = {
                agent_id = analyst_agent.id,
                prompt = "Provide technical analysis"
            }
        },
        {
            name = "business_analysis",
            tool = "agent_executor",
            parameters = {
                agent_id = writer_agent.id,
                prompt = "Provide business perspective"
            }
        }
    },
    merge_strategy = "combine"  -- Combine all outputs
})

-- Step 7: Demonstrate workflow composition
print("\n=== Workflow Composition Example ===")

-- Store workflow references for reuse
State.set("workflow_library", {
    report_generation = report_workflow.id,
    report_review = review_workflow.id,
    parallel_analysis = parallel_analysis.id
})

-- Create a master workflow that uses other workflows
local master_workflow = Workflow.sequential({
    name = "complete_analysis_pipeline",
    steps = {
        {
            name = "generate",
            workflow = "report_generation"  -- Reference to sub-workflow
        },
        {
            name = "analyze", 
            workflow = "parallel_analysis"
        },
        {
            name = "review",
            workflow = "report_review"
        }
    }
})

print("Created master workflow with nested workflows")

-- Step 8: Demonstrate event hooks (placeholders for Phase 4)
Hook.register("workflow_start", function(data)
    print("Workflow started:", data.workflow_name)
    State.set("workflow_start_time", os.time())
end)

Hook.register("workflow_complete", function(data)
    local start_time = State.get("workflow_start_time") or os.time()
    local duration = os.time() - start_time
    print("Workflow completed in", duration, "seconds")
end)

-- Step 9: Clean up and summary
print("\n=== Integration Summary ===")
local active_agents = Agent.list()
print("Active agents:", #active_agents)

local state_keys = State.list()
print("State entries:", #state_keys)

-- Demonstrate state persistence across script execution
local saved_data = {
    agents = {},
    workflows = State.get("workflow_library"),
    report = State.get("latest_report")
}

-- Save to JSON for potential export
local export_json = JSON.stringify(saved_data)
print("Exportable data size:", #export_json, "bytes")

print("\nThis example demonstrated:")
print("- Creating and coordinating multiple agents")
print("- Building complex workflows with sequential, conditional, and parallel patterns")
print("- Using State for cross-component communication")
print("- Workflow composition and reuse")
print("- Integration patterns for production systems")