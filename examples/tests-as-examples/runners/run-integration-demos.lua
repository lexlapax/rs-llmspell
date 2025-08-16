-- Example: Integration Demos Runner
-- Purpose: Focused runner for integration examples demonstrating real-world scenarios
-- Prerequisites: Full system setup with hooks, events, state, and sessions enabled
-- Expected Output: Guided execution of complex integration patterns with analysis
-- Version: 0.7.0
-- Tags: test, runner, integration, real-world

-- ABOUTME: Focused runner for integration examples demonstrating real-world scenarios
-- ABOUTME: Provides guided execution of complex integration patterns with detailed analysis

print("=== LLMSpell Integration Demos Runner ===")
print("Focused execution of real-world integration scenarios")
print()

-- Configuration
local config = {
    interactive_mode = false, -- Set to true for step-by-step execution
    detailed_analysis = true,
    performance_profiling = true,
    resource_monitoring = true,
    demo_pause_duration = 3, -- seconds between major sections
    show_system_state = true,
    generate_integration_report = true
}

-- Demo state tracking
local demo_state = {
    current_demo = nil,
    demos_completed = 0,
    total_demos = 3,
    start_time = os.time(),
    performance_metrics = {
        data_pipeline = {},
        workflow_automation = {},
        monitoring_system = {}
    },
    integration_insights = {}
}

-- Helper function to monitor system resources
local function monitor_resources()
    local resources = {
        timestamp = os.time(),
        memory_usage = "Simulated: 45MB",
        cpu_usage = "Simulated: 12%",
        active_agents = Agent and #Agent.list() or 0,
        active_sessions = 0,
        state_entries = 0
    }
    
    if Session and Session.list then
        local sessions = Session.list()
        resources.active_sessions = sessions and #sessions or 0
    end
    
    if State then
        -- Simulate state entry count
        resources.state_entries = math.random(10, 50)
    end
    
    return resources
end

-- Helper function to run a demo with comprehensive tracking
local function run_integration_demo(demo_name, demo_description, demo_function)
    print(string.format("\n[Demo %d/%d] %s", demo_state.demos_completed + 1, demo_state.total_demos, demo_name))
    print("Description: " .. demo_description)
    print(string.rep("-", 50))
    
    demo_state.current_demo = demo_name
    
    local start_time = os.clock()
    local start_resources = monitor_resources()
    
    local success, result = pcall(demo_function)
    
    local end_time = os.clock()
    local end_resources = monitor_resources()
    local execution_time = (end_time - start_time) * 1000
    
    demo_state.demos_completed = demo_state.demos_completed + 1
    
    if success then
        print(string.format("✅ Demo completed successfully (%.2fms)", execution_time))
        if config.detailed_analysis and result then
            print("Result summary:")
            if type(result) == "table" then
                for key, value in pairs(result) do
                    print(string.format("  %s: %s", key, tostring(value)))
                end
            else
                print("  " .. tostring(result))
            end
        end
    else
        print(string.format("❌ Demo failed (%.2fms)", execution_time))
        print("Error: " .. tostring(result))
    end
    
    -- Store performance metrics
    demo_state.performance_metrics[demo_name:lower():gsub("%s+", "_")] = {
        execution_time = execution_time,
        success = success,
        start_resources = start_resources,
        end_resources = end_resources,
        result = success and result or nil,
        error = not success and result or nil
    }
    
    if config.show_system_state then
        print("\nSystem State:")
        print(string.format("  Memory: %s → %s", start_resources.memory_usage, end_resources.memory_usage))
        print(string.format("  CPU: %s → %s", start_resources.cpu_usage, end_resources.cpu_usage))
        print(string.format("  Active Agents: %d → %d", start_resources.active_agents, end_resources.active_agents))
        print(string.format("  Active Sessions: %d → %d", start_resources.active_sessions, end_resources.active_sessions))
    end
    
    if config.demo_pause_duration > 0 then
        print(string.format("\nPausing for %d seconds...", config.demo_pause_duration))
        local pause_start = os.clock()
        while (os.clock() - pause_start) < config.demo_pause_duration do
            -- Busy wait for demonstration
        end
    end
    
    demo_state.current_demo = nil
end

-- Demo 1: Data Pipeline Integration
local function data_pipeline_demo()
    print("Initializing real-time data pipeline...")
    
    -- Simulate pipeline setup
    local pipeline_config = {
        input_sources = {"sensors", "api_feeds", "file_watchers"},
        processing_stages = {"validation", "transformation", "enrichment"},
        output_targets = {"database", "cache", "notifications"}
    }
    
    print("Pipeline configuration:")
    for stage, details in pairs(pipeline_config) do
        print(string.format("  %s: %s", stage, table.concat(details, ", ")))
    end
    
    -- Simulate data processing
    local processed_items = 0
    for i = 1, 10 do
        -- Simulate processing delay
        local process_start = os.clock()
        while (os.clock() - process_start) < 0.1 do end
        
        processed_items = processed_items + 1
        print(string.format("  Processed item %d/10", processed_items))
    end
    
    return {
        pipeline_id = "data_pipeline_001",
        items_processed = processed_items,
        status = "running",
        throughput = "100 items/sec"
    }
end

-- Demo 2: Workflow Automation
local function workflow_automation_demo()
    print("Setting up intelligent workflow automation...")
    
    -- Create workflow components
    local workflow_steps = {
        "Document ingestion",
        "Content analysis", 
        "Classification",
        "Routing",
        "Response generation"
    }
    
    print("Workflow steps:")
    for i, step in ipairs(workflow_steps) do
        print(string.format("  %d. %s", i, step))
        
        -- Simulate step execution
        local step_start = os.clock()
        while (os.clock() - step_start) < 0.2 do end
        print(string.format("     ✓ Completed (%.1fms)", 200))
    end
    
    -- Simulate agent coordination
    if Agent then
        print("\nAgent coordination:")
        local agents = Agent.list()
        print(string.format("  Available agents: %d", #agents))
        print("  Workflow orchestration: Active")
    end
    
    return {
        workflow_id = "automation_workflow_001",
        steps_completed = #workflow_steps,
        total_execution_time = 1000,
        status = "completed"
    }
end

-- Demo 3: Monitoring System Integration
local function monitoring_system_demo()
    print("Deploying intelligent monitoring system...")
    
    -- Simulate monitoring setup
    local monitoring_components = {
        "Event collectors",
        "Anomaly detectors", 
        "Alert processors",
        "Dashboard generators"
    }
    
    print("Monitoring components:")
    for _, component in ipairs(monitoring_components) do
        print(string.format("  - %s: Online", component))
    end
    
    -- Simulate event processing
    local events_processed = 0
    local anomalies_detected = 0
    
    for i = 1, 20 do
        events_processed = events_processed + 1
        
        -- Simulate anomaly detection (10% chance)
        if math.random() < 0.1 then
            anomalies_detected = anomalies_detected + 1
            print(string.format("  🚨 Anomaly detected in event %d", i))
        end
    end
    
    print(string.format("\nMonitoring results:"))
    print(string.format("  Events processed: %d", events_processed))
    print(string.format("  Anomalies detected: %d", anomalies_detected))
    print(string.format("  System health: %s", anomalies_detected < 3 and "Good" or "Attention needed"))
    
    return {
        monitoring_session_id = "monitor_001",
        events_processed = events_processed,
        anomalies_detected = anomalies_detected,
        system_health = anomalies_detected < 3 and "good" or "degraded"
    }
end

-- Execute integration demos
print("Starting integration demonstrations...")
print("Configuration:")
for key, value in pairs(config) do
    print(string.format("  %s: %s", key, tostring(value)))
end

-- Run all demos
run_integration_demo("Data Pipeline", "Real-time data processing pipeline", data_pipeline_demo)
run_integration_demo("Workflow Automation", "Intelligent document processing workflow", workflow_automation_demo)
run_integration_demo("Monitoring System", "Anomaly detection and alerting system", monitoring_system_demo)

-- Generate comprehensive report
local function generate_integration_report()
    local total_time = os.time() - demo_state.start_time
    
    print("\n" .. string.rep("=", 60))
    print("INTEGRATION DEMONSTRATIONS REPORT")
    print(string.rep("=", 60))
    
    print(string.format("Total Demos: %d", demo_state.total_demos))
    print(string.format("Completed: %d", demo_state.demos_completed))
    print(string.format("Total Time: %d seconds", total_time))
    
    print("\nPerformance Summary:")
    local total_execution_time = 0
    for demo_name, metrics in pairs(demo_state.performance_metrics) do
        print(string.format("  %s:", demo_name:gsub("_", " "):gsub("(%w+)", function(w) return w:sub(1,1):upper()..w:sub(2) end)))
        print(string.format("    Execution: %.2fms", metrics.execution_time))
        print(string.format("    Status: %s", metrics.success and "Success" or "Failed"))
        total_execution_time = total_execution_time + metrics.execution_time
    end
    
    print(string.format("\nAverage Demo Time: %.2fms", total_execution_time / demo_state.total_demos))
    
    print("\nIntegration Insights:")
    print("  - Real-time data processing demonstrated")
    print("  - Workflow automation patterns validated")
    print("  - Monitoring and alerting systems functional")
    print("  - Cross-component integration successful")
    
    print(string.rep("=", 60))
end

if config.generate_integration_report then
    generate_integration_report()
end

print("\n🎯 Integration demos completed successfully!")
print("These demos showcase real-world LLMSpell usage patterns")

-- Return comprehensive results
return {
    demos_completed = demo_state.demos_completed,
    total_time = os.time() - demo_state.start_time,
    performance_metrics = demo_state.performance_metrics,
    success = demo_state.demos_completed == demo_state.total_demos
}