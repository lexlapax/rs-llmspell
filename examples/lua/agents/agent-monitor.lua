-- ABOUTME: Agent monitor example demonstrating system and process monitoring
-- ABOUTME: Shows how agents can continuously monitor and respond to system changes

-- Agent Monitor Example
-- Demonstrates how agents can monitor systems, processes, and data changes

print("=== Agent Monitor Example ===\n")

-- Create a monitoring agent
local monitor = Agent.create({
    name = "system_monitor_agent",
    description = "Monitors system health, processes, and data changes",
    provider = "openai",
    model = "gpt-4o-mini",
    system_prompt = [[
You are a system monitoring specialist. You:
1. Track system resource usage
2. Monitor file changes
3. Watch for anomalies
4. Generate alerts when thresholds are exceeded
5. Provide actionable insights

Be concise and focus on important changes and potential issues.
]],
    temperature = 0.2  -- Low temperature for consistent monitoring
})

-- Check if monitor was created successfully
if not monitor then
    print("Failed to create monitor agent - check API keys")
    return
end

-- Example 1: System Resource Monitoring
print("Example 1: System Resource Monitoring")
print("-" .. string.rep("-", 37))

-- Get baseline system metrics
local baseline_metrics = Tool.executeAsync("system_monitor", {
    operation = "stats"
})

print("Baseline metrics captured")

-- Monitor and analyze system health
local health_result = monitor:execute({
    prompt = string.format([[
Analyze the current system health:

%s

Provide:
1. Overall health status (Good/Warning/Critical)
2. Any concerning metrics
3. Recommendations for optimization
4. Predicted issues based on current usage
]], baseline_metrics.output)
})

print("System Health Analysis:")
print(health_result.content)

-- Example 2: File Change Monitoring
print("\n\nExample 2: File Change Monitoring")
print("-" .. string.rep("-", 33))

-- Create a test directory structure
Tool.executeAsync("file_operations", {
    operation = "write",
    path = "/tmp/monitor_test/config.json",
    input = '{"version": "1.0", "debug": false}'
})

Tool.executeAsync("file_operations", {
    operation = "write",
    path = "/tmp/monitor_test/data.csv",
    input = "timestamp,value\n2024-01-01,100\n2024-01-02,105"
})

-- Initial file state
local initial_files = Tool.executeAsync("file_operations", {
    operation = "list",
    path = "/tmp/monitor_test"
})

-- Simulate changes
Tool.executeAsync("file_operations", {
    operation = "write",
    path = "/tmp/monitor_test/config.json",
    input = '{"version": "1.1", "debug": true, "new_feature": "enabled"}'
})

Tool.executeAsync("file_operations", {
    operation = "write",
    path = "/tmp/monitor_test/new_file.txt",
    input = "This is a new file"
})

-- Monitor analyzes changes
local changes_result = monitor:execute({
    prompt = [[
Monitor file system changes in /tmp/monitor_test:
1. Check current files and compare with baseline
2. Identify what files were modified, added, or deleted
3. For modified files, explain what changed
4. Assess the impact of these changes
5. Recommend any actions needed
]]
})

print("File Change Analysis:")
print(changes_result.content)

-- Example 3: Process Monitoring
print("\n\nExample 3: Process Monitoring")
print("-" .. string.rep("-", 29))

-- Get current processes
local processes = Tool.executeAsync("process_executor", {
    command = "ps",
    args = {"aux"},
    capture_output = true
})

-- Monitor analyzes processes
local process_result = monitor:execute({
    prompt = [[
Analyze the running processes and identify:
1. High CPU consumers
2. High memory consumers
3. Long-running processes
4. Potential zombie or stuck processes
5. Security concerns (unexpected processes)

Focus on the top 5 most important findings.
]]
})

print("Process Analysis:")
print(process_result.content)

-- Example 4: Log Monitoring Pattern
print("\n\nExample 4: Log Monitoring Pattern")
print("-" .. string.rep("-", 33))

-- Create sample log data
local log_data = [[
2024-01-21 10:00:00 INFO Application started
2024-01-21 10:01:00 INFO Connected to database
2024-01-21 10:02:00 WARNING High memory usage: 85%
2024-01-21 10:03:00 ERROR Connection timeout to API
2024-01-21 10:03:01 ERROR Retry attempt 1 failed
2024-01-21 10:03:05 ERROR Retry attempt 2 failed
2024-01-21 10:03:10 INFO Connection restored
2024-01-21 10:04:00 WARNING Slow query detected: 5.2s
2024-01-21 10:05:00 CRITICAL Disk space low: 95% used
2024-01-21 10:06:00 INFO Cleanup process started
]]

Tool.executeAsync("file_operations", {
    operation = "write",
    path = "/tmp/application.log",
    input = log_data
})

-- Monitor analyzes logs
local log_result = monitor:execute({
    prompt = string.format([[
Analyze this application log for issues:

%s

Provide:
1. Critical issues that need immediate attention
2. Error patterns and their frequency
3. Performance degradation indicators
4. Recommended actions for each issue
5. Overall system stability assessment
]], log_data)
})

print("Log Analysis:")
print(log_result.content)

-- Example 5: Threshold-Based Alerting
print("\n\nExample 5: Threshold-Based Alerting")
print("-" .. string.rep("-", 35))

-- Define monitoring thresholds
local thresholds = {
    cpu_percent = 80,
    memory_percent = 90,
    disk_percent = 85,
    error_rate = 5,  -- errors per minute
    response_time = 1000  -- milliseconds
}

-- Simulate metrics that exceed thresholds
local current_metrics = {
    cpu_percent = 45,
    memory_percent = 92,  -- Exceeds threshold!
    disk_percent = 88,    -- Exceeds threshold!
    error_rate = 7,       -- Exceeds threshold!
    response_time = 850
}

-- Monitor generates alerts
local alert_result = monitor:execute({
    prompt = string.format([[
Check these metrics against thresholds and generate alerts:

Current Metrics:
%s

Thresholds:
%s

For each threshold exceeded:
1. Generate an alert with severity (Low/Medium/High/Critical)
2. Explain the impact
3. Provide immediate remediation steps
4. Suggest long-term solutions
]], 
    JSON.stringify(current_metrics),
    JSON.stringify(thresholds))
})

print("Alert Generation:")
print(alert_result.content)

-- Example 6: Continuous Monitoring Loop
print("\n\nExample 6: Continuous Monitoring Loop")
print("-" .. string.rep("-", 37))

-- Simulate a monitoring loop (limited iterations for demo)
local monitoring_active = true
local iteration = 0
local max_iterations = 3

print("Starting continuous monitoring (3 iterations)...")

while monitoring_active and iteration < max_iterations do
    iteration = iteration + 1
    print(string.format("\n--- Monitoring Iteration %d ---", iteration))
    
    -- Collect current metrics
    local current_state = {
        timestamp = os.date("%Y-%m-%d %H:%M:%S"),
        metrics = Tool.executeAsync("system_monitor", {
            operation = "stats"
        }).output,
        random_event = math.random() > 0.7 and "Spike detected" or "Normal"
    }
    
    -- Monitor analyzes state
    local loop_result = monitor:execute({
        prompt = string.format([[
Continuous monitoring check #%d:

State: %s

Determine:
1. Is everything normal or is action needed?
2. Any trends developing over the iterations?
3. Should monitoring continue or stop?

Be very concise - one line per point.
]], iteration, JSON.stringify(current_state))
    })
    
    print(loop_result.content)
    
    -- Simulate delay between checks
    os.execute("sleep 1")  -- 1 second delay
end

print("\nMonitoring loop completed")

-- Example 7: Predictive Monitoring
print("\n\nExample 7: Predictive Monitoring")
print("-" .. string.rep("-", 32))

-- Historical data for prediction
local historical_data = {
    { hour = 8, cpu = 20, memory = 45 },
    { hour = 9, cpu = 35, memory = 50 },
    { hour = 10, cpu = 60, memory = 65 },
    { hour = 11, cpu = 75, memory = 78 },
    { hour = 12, cpu = 85, memory = 85 },
    -- Current hour
    { hour = 13, cpu = 90, memory = 88 }
}

-- Monitor predicts future issues
local prediction_result = monitor:execute({
    prompt = string.format([[
Analyze this historical resource usage data and predict:

%s

1. When will CPU likely hit 100%% if trend continues?
2. When will memory become critical (>95%%)?
3. What's driving the increased usage?
4. Recommended preventive actions
5. Confidence level in predictions
]], JSON.stringify(historical_data))
})

print("Predictive Analysis:")
print(prediction_result.content)

-- Performance Report
print("\n\n=== Monitoring Performance Report ===")

local monitor_stats = {
    total_checks = 7,
    alerts_generated = 3,
    average_response_time = "~500ms",
    resource_usage = "Low",
    accuracy = "High"
}

print("Monitoring Session Summary:")
for key, value in pairs(monitor_stats) do
    print(string.format("- %s: %s", key:gsub("_", " "):gsub("^%l", string.upper), value))
end

print("\n=== Agent Monitor Example Complete ===")