-- ABOUTME: Comprehensive workflow examples showing all four patterns with Hook/Event/State integration
-- ABOUTME: Demonstrates sequential, conditional, loop, and parallel workflows with full features

-- Example 1: Sequential Workflow with State Management
print("=== Sequential Workflow with State ===")

local seq_workflow = Workflow.sequential({
    name = "data_processing_pipeline",
    description = "Process data through multiple stages",
    error_strategy = "continue", -- continue even if a step fails
    timeout_ms = 30000, -- 30 second timeout
    steps = {
        {
            name = "load_data",
            type = "tool",
            tool = "file_reader",
            input = { path = "/tmp/input.json" }
        },
        {
            name = "transform_data", 
            type = "tool",
            tool = "json_transformer",
            input = { transform = "uppercase_keys" }
        },
        {
            name = "save_data",
            type = "tool", 
            tool = "file_writer",
            input = { path = "/tmp/output.json" }
        }
    }
})

-- Set up hooks
seq_workflow:onBeforeExecute(function(context)
    print("Starting sequential workflow...")
end)

seq_workflow:onAfterExecute(function(context)
    print("Sequential workflow completed!")
end)

seq_workflow:onError(function(error)
    print("Error in workflow: " .. tostring(error))
end)

-- Store workflow state
seq_workflow:setState("status", "initialized")
seq_workflow:setState("start_time", os.time())

-- Example 2: Conditional Workflow with Branches
print("\n=== Conditional Workflow ===")

local cond_workflow = Workflow.conditional({
    name = "user_verification", 
    description = "Verify user based on conditions",
    error_strategy = "fail_fast",
    branches = {
        {
            name = "admin_branch",
            condition = {
                type = "shared_data_equals",
                key = "user_role",
                expected = "admin"
            },
            steps = {
                {
                    name = "grant_admin_access",
                    type = "tool",
                    tool = "access_controller",
                    input = { level = "admin" }
                }
            }
        },
        {
            name = "user_branch", 
            condition = {
                type = "shared_data_equals",
                key = "user_role",
                expected = "user"
            },
            steps = {
                {
                    name = "grant_user_access",
                    type = "tool",
                    tool = "access_controller", 
                    input = { level = "user" }
                }
            }
        }
    },
    default_branch = {
        name = "guest_branch",
        steps = {
            {
                name = "grant_guest_access",
                type = "tool",
                tool = "access_controller",
                input = { level = "guest" }
            }
        }
    }
})

-- Emit events during execution
cond_workflow:emit("workflow_created", { 
    workflow_type = "conditional",
    branch_count = 3
})

-- Example 3: Loop Workflow with Different Iterators
print("\n=== Loop Workflow Examples ===")

-- Range-based loop
local range_loop = Workflow.loop({
    name = "batch_processor",
    description = "Process items in batches",
    iterator = "range",
    start = 1,
    ["end"] = 10, -- end is a reserved word in Lua
    step = 2,
    body = {
        {
            name = "process_batch",
            type = "tool",
            tool = "batch_processor",
            input = { batch_size = 100 }
        }
    },
    break_conditions = {
        {
            type = "shared_data_equals",
            key = "stop_processing",
            expected = true
        }
    }
})

-- Collection-based loop
local collection_loop = Workflow.loop({
    name = "file_processor",
    description = "Process multiple files",
    iterator = "collection",
    items = {"file1.txt", "file2.txt", "file3.txt"},
    body = {
        {
            name = "process_file",
            type = "tool",
            tool = "file_processor",
            input = { operation = "analyze" }
        }
    },
    error_strategy = "continue"
})

-- While condition loop
local while_loop = Workflow.loop({
    name = "monitor_system",
    description = "Monitor system while condition is true",
    iterator = "while",
    condition = {
        type = "shared_data_equals",
        key = "monitoring_active",
        expected = true
    },
    max_iterations = 100,
    body = {
        {
            name = "check_system_health",
            type = "tool",
            tool = "system_monitor",
            input = { metrics = {"cpu", "memory", "disk"} }
        }
    }
})

-- Example 4: Parallel Workflow
print("\n=== Parallel Workflow ===")

local parallel_workflow = Workflow.parallel({
    name = "multi_analysis",
    description = "Analyze data in parallel",
    max_concurrency = 3,
    timeout_ms = 20000,
    error_strategy = "continue",
    branches = {
        {
            name = "sentiment_analysis",
            steps = {
                {
                    name = "analyze_sentiment",
                    type = "tool",
                    tool = "sentiment_analyzer",
                    input = { model = "bert-base" }
                }
            }
        },
        {
            name = "entity_extraction",
            steps = {
                {
                    name = "extract_entities",
                    type = "tool",
                    tool = "entity_extractor",
                    input = { types = {"person", "location", "organization"} }
                }
            }
        },
        {
            name = "topic_modeling",
            steps = {
                {
                    name = "model_topics",
                    type = "tool",
                    tool = "topic_modeler",
                    input = { num_topics = 5 }
                }
            }
        }
    }
})

-- Example 5: Nested Workflows (Workflow Composition)
print("\n=== Workflow Composition ===")

-- Create a master workflow that uses other workflows
local master_workflow = Workflow.sequential({
    name = "complete_pipeline",
    description = "Master pipeline combining multiple workflows",
    steps = {
        {
            name = "prepare_data",
            type = "tool",
            tool = "data_preparer",
            input = { format = "json" }
        },
        -- Note: Workflow steps would be supported in full implementation
        -- For now, we demonstrate the pattern
        {
            name = "parallel_analysis",
            type = "tool", -- Would be "workflow" when fully implemented
            tool = "workflow_executor",
            input = { 
                workflow_id = parallel_workflow:getInfo().id,
                inherit_state = true
            }
        },
        {
            name = "post_process",
            type = "tool",
            tool = "result_aggregator",
            input = { format = "report" }
        }
    }
})

-- Example 6: Workflow Registry Operations
print("\n=== Workflow Registry ===")

-- List all workflows
local all_workflows = Workflow.list()
print("Total workflows registered: " .. #all_workflows)

for i, wf in ipairs(all_workflows) do
    print(string.format("%d. %s (%s) - %s", 
        i, wf.id, wf.type, wf.description or "No description"))
end

-- Get a specific workflow
local workflow_id = seq_workflow:getInfo().id
local retrieved_workflow = Workflow.get(workflow_id)
if retrieved_workflow then
    print("Retrieved workflow: " .. retrieved_workflow:getInfo().name)
end

-- Example 7: State Management Across Workflows
print("\n=== Cross-Workflow State ===")

-- Store shared state that workflows can access
if State then
    State.set("shared:processing_mode", "batch")
    State.set("shared:max_retries", 3)
    State.set("shared:monitoring_active", true)
end

-- Workflows can read shared state
local processing_mode = seq_workflow:getState("processing_mode")
print("Processing mode: " .. tostring(processing_mode))

-- Example 8: Event-Driven Workflow Coordination
print("\n=== Event-Driven Coordination ===")

-- Emit workflow lifecycle events
parallel_workflow:emit("analysis_started", {
    timestamp = os.time(),
    branch_count = 3
})

-- Other workflows could listen to these events (in Phase 4)
-- For now, we demonstrate the emission pattern

-- Example 9: Error Handling Patterns
print("\n=== Error Handling ===")

local error_handling_workflow = Workflow.sequential({
    name = "robust_pipeline",
    description = "Pipeline with comprehensive error handling",
    error_strategy = "retry", -- Will retry failed steps
    steps = {
        {
            name = "risky_operation",
            type = "tool",
            tool = "network_caller",
            input = { 
                url = "https://api.example.com/data",
                timeout = 5000,
                retry_count = 3
            }
        }
    }
})

-- Set up error hook
error_handling_workflow:onError(function(error)
    -- Log error and emit event
    print("Workflow error: " .. tostring(error))
    error_handling_workflow:emit("workflow_error", {
        error = error,
        timestamp = os.time()
    })
    
    -- Store error in state for analysis
    error_handling_workflow:setState("last_error", error)
    error_handling_workflow:setState("error_count", 
        (error_handling_workflow:getState("error_count") or 0) + 1)
end)

-- Example 10: Workflow Execution
print("\n=== Executing Workflows ===")

-- Execute sequential workflow
-- local result = seq_workflow:execute({
--     input_data = { key = "value" }
-- })
-- print("Sequential result: " .. tostring(result))

-- Note: Actual execution would require the workflow runtime to be running
-- This example demonstrates the API patterns

print("\n=== Workflow Types ===")
local types = Workflow.types()
for i, wf_type in ipairs(types) do
    print(i .. ". " .. wf_type)
end

print("\n=== Comprehensive Workflow Examples Complete ===")