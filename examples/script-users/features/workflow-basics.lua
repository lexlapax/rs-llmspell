-- ============================================================
-- LLMSPELL FEATURES SHOWCASE  
-- ============================================================
-- Feature ID: 03 - Workflow Basics v0.7.0
-- Complexity Level: INTERMEDIATE
-- Real-World Use Case: Automating multi-step data processing pipelines
-- Feature Category: Workflows
--
-- Purpose: Introduction to workflow orchestration patterns
-- Architecture: Builder pattern for workflow construction
-- Key Capabilities:
--   • Workflow.builder() - Fluent workflow creation
--   • Sequential execution - Step-by-step processing
--   • Parallel execution - Concurrent step execution
--   • Tool integration - Orchestrating multiple tools
--   • Data flow between steps
--
-- Prerequisites: None (uses local tools)
--
-- HOW TO RUN:
-- ./target/debug/llmspell run examples/script-users/features/workflow-basics.lua
--
-- EXPECTED OUTPUT:
-- Two workflow examples: sequential and parallel execution
-- Execution time: <2 seconds
--
-- Time to Complete: 2 seconds
-- Next Steps: See advanced-patterns/complex-workflows.lua
-- ============================================================

print("=== Workflow Basics - Orchestration Patterns ===\n")

-- 1. SEQUENTIAL WORKFLOW
print("1. Sequential Workflow")
print("-" .. string.rep("-", 21))

-- Create a sequential workflow that processes data step by step
local seq_workflow = Workflow.builder()
    :name("data_processing_pipeline")
    :description("Process data through multiple sequential steps")
    :sequential()  -- Sequential execution mode
    :add_step({
        name = "generate_id",
        type = "tool",
        tool = "uuid_generator",
        input = {
            operation = "generate",
            version = "v4"
        }
    })
    :add_step({
        name = "get_timestamp",
        type = "tool",
        tool = "date_time_handler",
        input = {
            operation = "now"
        }
    })
    :add_step({
        name = "create_message",
        type = "tool",
        tool = "template_engine",
        input = {
            input = "Workflow {{id}} completed at {{time}}",
            context = {
                id = "{{step.generate_id.result}}",
                time = "{{step.get_timestamp.result}}"
            }
        }
    })
    :build()

print("   Executing sequential pipeline...")
local success, seq_result = pcall(function()
    return seq_workflow:execute({})
end)

if success and seq_result and seq_result.text then
    -- Workflow returns AgentOutput with text field containing summary
    if seq_result.text:match("completed successfully") then
        print("   ✓ Sequential workflow completed")
        print("   Summary: " .. seq_result.text:sub(1, 60) .. "...")
    else
        print("   ✗ Sequential workflow failed: " .. seq_result.text)
    end
else
    local error_msg = success and "No result" or tostring(seq_result)
    print("   ✗ Sequential workflow failed: " .. error_msg)
end

-- 2. PARALLEL WORKFLOW
print("\n2. Parallel Workflow")
print("-" .. string.rep("-", 19))

-- Create a parallel workflow that runs multiple operations concurrently
local par_workflow = Workflow.builder()
    :name("parallel_data_gathering")
    :description("Gather multiple data points in parallel")
    :parallel()  -- Parallel execution mode
    :add_step({
        name = "generate_uuid",
        type = "tool",
        tool = "uuid_generator",
        input = {
            operation = "generate",
            version = "v4"
        }
    })
    :add_step({
        name = "calculate_hash",
        type = "tool",
        tool = "hash_calculator",
        input = {
            operation = "hash",
            algorithm = "sha256",
            input = "workflow_data"
        }
    })
    :add_step({
        name = "encode_data",
        type = "tool",
        tool = "base64_encoder",
        input = {
            operation = "encode",
            input = "parallel_test"
        }
    })
    :build()

print("   Executing parallel operations...")
local success, par_result = pcall(function()
    return par_workflow:execute({})
end)

if success and par_result and par_result.text then
    if par_result.text:match("completed successfully") then
        print("   ✓ Parallel workflow completed")
        print("   All 3 operations ran concurrently")
    else
        print("   ✗ Parallel workflow failed: " .. par_result.text)
    end
else
    local error_msg = success and "No result" or tostring(par_result)
    print("   ✗ Parallel workflow failed: " .. error_msg)
end

-- 3. WORKFLOW WITH INPUT PARAMETERS
print("\n3. Parameterized Workflow")
print("-" .. string.rep("-", 24))

-- Create a workflow that accepts input parameters
local param_workflow = Workflow.builder()
    :name("text_processor")
    :description("Process text with configurable operations")
    :sequential()
    :add_step({
        name = "uppercase",
        type = "tool",
        tool = "text_manipulator",
        input = {
            operation = "uppercase",
            input = "{{input.text}}"  -- Reference to workflow input
        }
    })
    :add_step({
        name = "hash_text",
        type = "tool",
        tool = "hash_calculator",
        input = {
            operation = "hash",
            algorithm = "{{input.algorithm}}",  -- Configurable algorithm
            input = "{{step.uppercase.result}}"  -- Use previous step result
        }
    })
    :build()

print("   Executing with parameters...")
local success, param_result = pcall(function()
    return param_workflow:execute({
        text = "hello workflow",
        algorithm = "md5"
    })
end)

if success and param_result and param_result.text then
    if param_result.text:match("completed successfully") then
        print("   ✓ Parameterized workflow completed")
    else
        print("   ✗ Parameterized workflow failed: " .. param_result.text)
    end
else
    local error_msg = success and "No result" or tostring(param_result)
    print("   ✗ Parameterized workflow failed: " .. error_msg)
end

-- 4. WORKFLOW STATE AND DATA FLOW
print("\n4. Data Flow Patterns")
print("-" .. string.rep("-", 20))

print("   Step results accessible via: {{step.STEP_NAME.result}}")
print("   Workflow input via: {{input.PARAM_NAME}}")
print("   Conditional execution: See advanced-patterns/")
print("   Error handling: Each step can define retry policies")

-- 5. WORKFLOW DISCOVERY
print("\n5. Workflow Management")
print("-" .. string.rep("-", 21))

-- List available workflows (if any registered)
local workflows = Workflow.list()
print("   Registered workflows: " .. #workflows)

-- Workflow execution modes
print("   Execution modes:")
print("     • sequential() - Steps run one after another")
print("     • parallel() - Steps run concurrently")
print("     • conditional() - See advanced patterns")

-- 6. BEST PRACTICES
print("\n6. Best Practices")
print("-" .. string.rep("-", 16))

print("   • Name steps clearly for result references")
print("   • Use parallel mode for independent operations")
print("   • Validate tool availability before workflow creation")
print("   • Handle errors at workflow and step level")
print("   • Keep workflows focused on single responsibility")

print("\n=== Workflow Basics Complete ===")
print("Next: Explore state-persistence.lua for data management")