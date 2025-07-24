-- ABOUTME: Master runner script to execute all hook and event examples in sequence
-- ABOUTME: Provides comprehensive testing and demonstration of the complete llmspell hook/event system

print("=== LLMSpell Hook & Event Examples Runner ===")
print("Running all hook, event, and integration examples in sequence")
print()

-- Configuration
local config = {
    pause_between_examples = true,
    pause_duration = 2, -- seconds
    show_detailed_output = true,
    run_performance_tests = true,
    cleanup_between_examples = true,
    generate_summary_report = true
}

-- Runner state
local runner_state = {
    examples_run = 0,
    examples_passed = 0,
    examples_failed = 0,
    total_execution_time = 0,
    start_time = os.time(),
    results = {},
    errors = {}
}

-- Helper function to run an example with error handling and timing
local function run_example(category, filename, description)
    local full_path = string.format("examples/lua/%s/%s", category, filename)
    
    print(string.format("🚀 Running: %s", description))
    print(string.format("   File: %s", full_path))
    
    local start_time = os.clock()
    local success = true
    local error_message = nil
    
    -- Execute the example file
    local status, result = pcall(function()
        return dofile(full_path)
    end)
    
    local execution_time = (os.clock() - start_time) * 1000 -- Convert to milliseconds
    
    if status then
        runner_state.examples_passed = runner_state.examples_passed + 1
        print(string.format("✅ Completed: %s (%.2fms)", description, execution_time))
    else
        success = false
        error_message = tostring(result)
        runner_state.examples_failed = runner_state.examples_failed + 1
        print(string.format("❌ Failed: %s (%.2fms)", description, execution_time))
        print(string.format("   Error: %s", error_message))
        table.insert(runner_state.errors, {
            example = description,
            file = full_path,
            error = error_message,
            execution_time = execution_time
        })
    end
    
    -- Store result
    table.insert(runner_state.results, {
        category = category,
        filename = filename,
        description = description,
        success = success,
        execution_time = execution_time,
        error = error_message
    })
    
    runner_state.examples_run = runner_state.examples_run + 1
    runner_state.total_execution_time = runner_state.total_execution_time + execution_time
    
    -- Cleanup between examples if configured
    if config.cleanup_between_examples then
        -- Force garbage collection
        collectgarbage("collect")
        
        -- Small pause for cleanup
        if config.pause_between_examples then
            print(string.format("   ⏸️  Pausing for %ds...", config.pause_duration))
            os.execute("sleep " .. config.pause_duration)
        end
    end
    
    print() -- Add blank line for readability
    
    return success
end

-- Function to display section header
local function section_header(title, description)
    print(string.rep("=", 60))
    print(string.format("🎯 %s", title))
    print(string.format("   %s", description))
    print(string.rep("=", 60))
    print()
end

-- Function to display progress
local function show_progress()
    local progress_percent = runner_state.examples_run > 0 and 
                            (runner_state.examples_passed / runner_state.examples_run) * 100 or 0
    print(string.format("📊 Progress: %d/%d examples (%.1f%% success rate)", 
          runner_state.examples_passed, runner_state.examples_run, progress_percent))
    print()
end

print("Starting comprehensive examples execution...")
print(string.format("Configuration: Pause=%s, Cleanup=%s, Performance=%s", 
      config.pause_between_examples and "ON" or "OFF",
      config.cleanup_between_examples and "ON" or "OFF", 
      config.run_performance_tests and "ON" or "OFF"))
print()

-- Section 1: Hook Examples
section_header("HOOK EXAMPLES", "Demonstrating hook system capabilities from basic to advanced")

local hook_examples = {
    {"hook-basic.lua", "Basic Hook Registration and Unregistration"},
    {"hook-priorities.lua", "Hook Priority System and Execution Order"},
    {"hook-lifecycle.lua", "Complete Agent Lifecycle Hook Integration"},
    {"hook-tool-integration.lua", "Tool Execution Hooks and Validation"},
    {"hook-workflow-integration.lua", "Workflow Stage Hooks and Coordination"},
    {"hook-data-modification.lua", "Hook Result Types and Data Modification"},
    {"hook-error-handling.lua", "Comprehensive Error Handling in Hooks"},
    {"hook-cross-language.lua", "Cross-Language Hook Coordination"},
    {"hook-filtering-listing.lua", "Hook Listing and Filtering Capabilities"},
    {"hook-advanced-patterns.lua", "Advanced Hook Patterns and Complex Logic"}
}

for i, example in ipairs(hook_examples) do
    print(string.format("[%d/%d] Hook Example:", i, #hook_examples))
    run_example("hooks", example[1], example[2])
    show_progress()
end

print(string.format("🎉 Hook examples section completed: %d/%d successful", 
      runner_state.examples_passed, #hook_examples))
print()

-- Section 2: Event Examples  
section_header("EVENT EXAMPLES", "Demonstrating event system capabilities from basic to advanced")

local event_examples = {
    {"event-basic.lua", "Basic Event Publish/Subscribe Patterns"},
    {"event-patterns.lua", "Event Pattern Matching with Wildcards"},
    {"event-cross-language.lua", "Cross-Language Event Communication"},
    {"event-data-structures.lua", "Complex Nested Event Data Structures"},
    {"event-subscription-management.lua", "Event Subscription Lifecycle Management"},
    {"event-performance.lua", "High-Throughput Event Performance Scenarios"},
    {"event-timeout-handling.lua", "Event Timeouts and Error Recovery"},
    {"event-statistics.lua", "Event System Monitoring and Statistics"},
    {"event-workflow-coordination.lua", "Events for Workflow Coordination"},
    {"event-hook-integration.lua", "Events Triggered by Hooks Integration"}
}

local event_start_count = runner_state.examples_run

for i, example in ipairs(event_examples) do
    print(string.format("[%d/%d] Event Example:", i, #event_examples))
    run_example("events", example[1], example[2])
    show_progress()
end

local events_completed = runner_state.examples_run - event_start_count
print(string.format("🎉 Event examples section completed: %d/%d successful", 
      runner_state.examples_passed - #hook_examples, events_completed))
print()

-- Section 3: Integration Examples
section_header("INTEGRATION EXAMPLES", "Demonstrating real-world integration scenarios")

local integration_examples = {
    {"realtime-data-pipeline.lua", "Real-time Data Pipeline with End-to-End Processing"},
    {"user-workflow-automation.lua", "User Workflow Automation with Intelligent Routing"},
    {"intelligent-monitoring-system.lua", "AI-Driven Monitoring with Predictive Analytics"}
}

local integration_start_count = runner_state.examples_run

for i, example in ipairs(integration_examples) do
    print(string.format("[%d/%d] Integration Example:", i, #integration_examples))
    run_example("integration", example[1], example[2])
    show_progress()
end

local integrations_completed = runner_state.examples_run - integration_start_count
print(string.format("🎉 Integration examples section completed: %d/%d successful", 
      runner_state.examples_passed - #hook_examples - #event_examples, integrations_completed))
print()

-- Final Summary Report
section_header("EXECUTION SUMMARY", "Complete results of all examples execution")

local total_runtime = os.time() - runner_state.start_time
local success_rate = runner_state.examples_run > 0 and 
                    (runner_state.examples_passed / runner_state.examples_run) * 100 or 0

print("📊 Overall Statistics:")
print(string.format("   • Total examples run: %d", runner_state.examples_run))
print(string.format("   • Successful: %d", runner_state.examples_passed))
print(string.format("   • Failed: %d", runner_state.examples_failed))
print(string.format("   • Success rate: %.1f%%", success_rate))
print(string.format("   • Total execution time: %.2fms", runner_state.total_execution_time))
print(string.format("   • Total runtime: %ds", total_runtime))
print(string.format("   • Average execution time: %.2fms", 
      runner_state.examples_run > 0 and runner_state.total_execution_time / runner_state.examples_run or 0))

print()
print("📈 Performance Breakdown:")
print(string.format("   • Hook examples: %d (avg: %.2fms)", #hook_examples, 
      (function()
          local total = 0
          local count = 0
          for _, result in ipairs(runner_state.results) do
              if result.category == "hooks" then
                  total = total + result.execution_time
                  count = count + 1
              end
          end
          return count > 0 and total / count or 0
      end)()))
      
print(string.format("   • Event examples: %d (avg: %.2fms)", #event_examples,
      (function()
          local total = 0
          local count = 0
          for _, result in ipairs(runner_state.results) do
              if result.category == "events" then
                  total = total + result.execution_time
                  count = count + 1
              end
          end
          return count > 0 and total / count or 0
      end)()))
      
print(string.format("   • Integration examples: %d (avg: %.2fms)", #integration_examples,
      (function()
          local total = 0
          local count = 0
          for _, result in ipairs(runner_state.results) do
              if result.category == "integration" then
                  total = total + result.execution_time
                  count = count + 1
              end
          end
          return count > 0 and total / count or 0
      end)()))

-- Error Summary
if #runner_state.errors > 0 then
    print()
    print("❌ Errors Encountered:")
    for i, error in ipairs(runner_state.errors) do
        print(string.format("   %d. %s", i, error.example))
        print(string.format("      File: %s", error.file))
        print(string.format("      Error: %s", error.error))
        print(string.format("      Time: %.2fms", error.execution_time))
    end
else
    print()
    print("✅ No errors encountered - all examples executed successfully!")
end

-- System Resource Summary
print()
print("💾 System Resources:")
print(string.format("   • Lua memory usage: %.2f KB", collectgarbage("count")))
print(string.format("   • Examples per second: %.2f", 
      total_runtime > 0 and runner_state.examples_run / total_runtime or 0))

-- Generate recommendations
print()
print("💡 Recommendations:")

if success_rate < 100 then
    print("   • Some examples failed - check error details above")
    print("   • Consider running failed examples individually for debugging")
end

if runner_state.total_execution_time > 60000 then -- More than 1 minute
    print("   • Long execution time detected - consider running examples in parallel")
end

if success_rate >= 95 then
    print("   • Excellent success rate - hook and event system is working well")
    print("   • Consider integrating these patterns into your applications")
end

-- Final status
print()
print(string.rep("=", 60))
if success_rate == 100 then
    print("🎉 ALL EXAMPLES COMPLETED SUCCESSFULLY!")
    print("   The llmspell hook and event system is fully functional")
elseif success_rate >= 90 then
    print("✅ EXAMPLES MOSTLY SUCCESSFUL")
    print(string.format("   %d examples passed with %d minor issues", 
          runner_state.examples_passed, runner_state.examples_failed))
else
    print("⚠️  SOME EXAMPLES FAILED")
    print(string.format("   %d/%d examples successful - check error details", 
          runner_state.examples_passed, runner_state.examples_run))
end

print()
print("🚀 Hook & Event Examples Runner Complete")
print("   Ready for integration into your llmspell applications!")
print(string.rep("=", 60))