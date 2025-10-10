-- Application: Instrumented Agent Debugger
-- Purpose: Demonstrate debugging and tracing capabilities for agent applications
-- Prerequisites: OPENAI_API_KEY or ANTHROPIC_API_KEY environment variables
-- Expected Output: Demonstrates Debug logging, State persistence, and REPL inspection techniques
-- Version: 1.0.0
-- Tags: application, debugging, instrumented-agent, repl, state-inspection
--
-- HOW TO RUN:
-- 1. Basic: ./target/debug/llmspell run examples/script-users/applications/instrumented-agent/main.lua
-- 2. With config: ./target/debug/llmspell -c examples/script-users/applications/instrumented-agent/config.toml run examples/script-users/applications/instrumented-agent/main.lua
-- 3. Debug trace: ./target/debug/llmspell --trace debug -c examples/script-users/applications/instrumented-agent/config.toml run examples/script-users/applications/instrumented-agent/main.lua
--
-- TO INSPECT WITH REPL:
-- After running this script, use: ./target/debug/llmspell repl
-- Then try commands shown at the end of this script's output

print("=== Instrumented Agent Debugger ===")
print("Demonstrating debugging and tracing capabilities\n")

-- ============================================================
-- Step 1: Setup and Configuration
-- ============================================================

local timestamp = os.time()
local module_name = "instrumented"

-- Sample code to analyze (for demonstration)
local code_input = [[
function calculate_total(items)
    local total = 0
    for i, item in ipairs(items) do
        if item.price > 0 then
            total = total + item.price * item.quantity
        end
    end
    return total
end
]]

-- Set debug level for this demonstration
Debug.setLevel("debug")
Debug.info("Starting instrumented agent demonstration", module_name)
Debug.debug("Timestamp: " .. timestamp, module_name)

-- ============================================================
-- Step 2: Create Agents with Debug Instrumentation
-- ============================================================

print("📍 Step 1: Creating instrumented agents...")

-- Start timing agent creation
local creation_timer = Debug.timer("agent_creation")

-- Create code analyzer agent
local analyzer = Agent.builder()
    :name("code_analyzer_" .. timestamp)
    :description("Analyzes code for issues and improvements")
    :type("llm")
    :model("openai/gpt-4o-mini")
    :temperature(0.3)
    :max_tokens(500)
    :custom_config({
        system_prompt = "You are a code analysis expert. Analyze the provided code for potential issues, improvements, and best practices. Be concise and specific."
    })
    :build()

-- Create code reviewer agent (using OpenAI instead of Anthropic due to API issues)
local reviewer = Agent.builder()
    :name("code_reviewer_" .. timestamp)
    :description("Reviews code for quality")
    :type("llm")
    :model("openai/gpt-4o-mini")
    :temperature(0.2)
    :max_tokens(400)
    :custom_config({
        system_prompt = "You are a code reviewer. Review the code for readability, maintainability, and potential bugs. Provide actionable feedback."
    })
    :build()

-- Stop creation timer and log
local creation_duration = creation_timer:stop()
Debug.info("Agents created in " .. tostring(creation_duration) .. "ms", module_name)

-- Check if agents were created successfully
if not analyzer and not reviewer then
    Debug.error("No agents created - check API keys", module_name)
    print("❌ No API keys configured. Set OPENAI_API_KEY or ANTHROPIC_API_KEY")
    print("\n📝 Demo mode: Showing debugging features without real LLM calls")

    -- Create demo checkpoint for illustration
    State.save("custom", ":demo:checkpoint", {
        timestamp = timestamp,
        mode = "demo",
        reason = "no_api_keys"
    })

    print("\n🔍 Even in demo mode, you can inspect state in REPL:")
    print("  1. Run: llmspell repl")
    print("  2. Type: State.load('custom', ':demo:checkpoint')")
    print("  3. Type: Debug.getCapturedEntries(10)")
    return
end

print("✅ Agents created successfully")

-- ============================================================
-- Step 3: Execute Agent with Checkpointing
-- ============================================================

print("\n📍 Step 2: Executing agent with checkpointing...")

-- Save pre-execution checkpoint
Debug.debug("Saving pre-execution checkpoint", module_name)
State.save("custom", ":checkpoint:pre_analysis", {
    timestamp = timestamp,
    input_size = string.len(code_input),
    agents_available = {
        analyzer = analyzer ~= nil,
        reviewer = reviewer ~= nil
    }
})

-- Execute analysis if analyzer exists
local analysis_result = nil
if analyzer then
    -- Start execution timer
    local exec_timer = Debug.timer("analysis_execution")
    Debug.info("Starting code analysis", module_name)

    -- Execute agent
    analysis_result = analyzer:execute({
        text = code_input,
        instruction = "Analyze this Lua function for potential issues and improvements"
    })

    -- Stop timer and log
    local exec_duration = exec_timer:stop()
    Debug.info("Analysis completed in " .. tostring(exec_duration) .. "ms", module_name)

    -- Save result for REPL inspection
    if analysis_result then
        State.save("custom", ":last_analysis", analysis_result)
        Debug.debug("Analysis result saved to State", module_name)

        print("✅ Analysis completed successfully")
        print("\n📋 Analysis Result:")
        print("-------------------")
        if type(analysis_result) == "string" then
            print(analysis_result)
        elseif type(analysis_result) == "table" and analysis_result.response then
            print(analysis_result.response)
        else
            print("Result type: " .. type(analysis_result))
        end
        print("-------------------")
    else
        Debug.warn("No result from analyzer", module_name)
        print("⚠️ Analyzer returned no result")

        -- Load checkpoint for recovery
        local checkpoint = State.load("custom", ":checkpoint:pre_analysis")
        if checkpoint then
            Debug.info("Loaded checkpoint from timestamp: " .. tostring(checkpoint.timestamp), module_name)
            print("📌 Checkpoint loaded for potential retry")
        end
    end
end

-- ============================================================
-- Step 4: Workflow with Multiple Agents
-- ============================================================

print("\n📍 Step 3: Creating debugging workflow...")

-- Only create workflow if we have agents
if analyzer or reviewer then
    local workflow = Workflow.builder()
        :name("debug_workflow_" .. timestamp)
        :description("Parallel code analysis and review")
        :parallel()

    -- Add analyzer step if available
    if analyzer then
        workflow = workflow:add_step({
            name = "analyze",
            type = "agent",
            agent = "code_analyzer_" .. timestamp,
            input = "Analyze this code for issues: {{code_input}}"
        })
    end

    -- Add reviewer step if available
    if reviewer then
        workflow = workflow:add_step({
            name = "review",
            type = "agent",
            agent = "code_reviewer_" .. timestamp,
            input = "Review this code for quality: {{code_input}}"
        })
    end

    local debug_workflow = workflow:build()

    -- Execute workflow with timing
    local workflow_timer = Debug.timer("workflow_execution")
    Debug.info("Executing parallel workflow", module_name)

    local workflow_result = debug_workflow:execute({
        code_input = code_input
    })

    local workflow_duration = workflow_timer:stop()
    Debug.info("Workflow completed in " .. tostring(workflow_duration) .. "ms", module_name)

    -- Check for workflow outputs (automatic collection)
    if analyzer then
        local agent_outputs = workflow_result.metadata and workflow_result.metadata.extra
            and workflow_result.metadata.extra.agent_outputs or {}
        local analysis_output = agent_outputs["code_analyzer_" .. timestamp]
        if analysis_output then
            Debug.debug("Workflow analysis output retrieved from metadata.extra.agent_outputs", module_name)
        end
    end

    print("✅ Workflow execution completed")
end

-- ============================================================
-- Step 5: Tool Usage with Debug Logging
-- ============================================================

print("\n📍 Step 4: Writing results with tool...")

-- Format results for file
local formatted_results = [[
# Code Analysis Report
Generated: ]] .. os.date("%Y-%m-%d %H:%M:%S") .. [[


## Input Code
```lua
]] .. code_input .. [[
```

## Analysis Results
]] .. (analysis_result and tostring(analysis_result) or "No analysis available") .. [[


## Debug Information
- Timestamp: ]] .. timestamp .. [[
- Agents Created: ]] .. ((analyzer and 1 or 0) + (reviewer and 1 or 0)) .. [[
- Execution Mode: ]] .. (analyzer and "Live" or "Demo")

-- Write to file with debug logging
Debug.debug("Writing results to file", module_name)
local write_result = Tool.execute("file-operations", {
    operation = "write",
    path = "/tmp/instrumented-analysis-" .. timestamp .. ".md",
    input = formatted_results
})

if write_result then
    print("✅ Results written to: /tmp/instrumented-analysis-" .. timestamp .. ".md")
else
    Debug.warn("Failed to write results file", module_name)
end

-- ============================================================
-- Step 6: Debug Summary and REPL Instructions
-- ============================================================

print("\n📊 Debug Summary")
print("===============")

-- Get debug level
local current_level = Debug.getLevel()
print("• Debug Level: " .. current_level)

-- Check if debug is enabled
local debug_enabled = Debug.isEnabled()
print("• Debug Enabled: " .. tostring(debug_enabled))

-- Get some captured entries
local entries = Debug.getCapturedEntries(5)
if entries and #entries > 0 then
    print("• Recent Debug Entries: " .. #entries)
end

-- List State keys for inspection
print("\n📦 State Keys Available for Inspection:")
print("• custom::checkpoint:pre_analysis")
print("• custom::last_analysis")
if analyzer then
    print("• result.metadata.extra.agent_outputs[\"code_analyzer_" .. timestamp .. "\"] (automatic collection)")
end

print("\n🔍 To inspect state in REPL:")
print("=====================================")
print("1. Run: ./target/debug/llmspell repl")
print("2. Try these commands:")
print("   > State.load('custom', ':last_analysis')")
print("   > State.load('custom', ':checkpoint:pre_analysis')")
print("   > Debug.getCapturedEntries(10)")
print("   > Debug.getLevel()")
print("   > Debug.isEnabled()")
print("   > State.list_keys('custom:')")
print("   > Session.get_current()")
print("=====================================")

print("\n✨ Instrumented Agent Debugger demonstration complete!")
print("Use the REPL commands above to explore the saved state and debug information.")

-- Final debug log
Debug.info("Demonstration completed successfully", module_name)