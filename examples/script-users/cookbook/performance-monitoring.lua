-- Cookbook: Performance Monitoring - Track and Optimize System Performance
-- Purpose: Implement patterns for monitoring, measuring, and optimizing performance
-- Prerequisites: Performance monitoring tools (optional for enhanced metrics)
-- Expected Output: Demonstration of performance monitoring patterns
-- Version: 0.7.0
-- Tags: cookbook, performance, monitoring, optimization, metrics

print("=== Performance Monitoring Patterns ===\n")

-- ============================================================
-- Pattern 1: Real-time Performance Metrics
-- ============================================================

print("1. Real-time Performance Metrics Collection")
print("-" .. string.rep("-", 40))

local PerformanceMonitor = {}
PerformanceMonitor.__index = PerformanceMonitor

function PerformanceMonitor:new()
    return setmetatable({
        metrics = {},
        start_times = {},
        counters = {},
        gauges = {},
        histograms = {}
    }, self)
end

function PerformanceMonitor:start_timer(operation_name)
    self.start_times[operation_name] = os.clock()
end

function PerformanceMonitor:end_timer(operation_name)
    local start_time = self.start_times[operation_name]
    if not start_time then
        print("   ⚠️  No start time found for: " .. operation_name)
        return 0
    end
    
    local duration = (os.clock() - start_time) * 1000 -- Convert to milliseconds
    self:record_histogram(operation_name .. "_duration", duration)
    self.start_times[operation_name] = nil
    return duration
end

function PerformanceMonitor:increment_counter(name, value)
    value = value or 1
    self.counters[name] = (self.counters[name] or 0) + value
end

function PerformanceMonitor:set_gauge(name, value)
    self.gauges[name] = value
end

function PerformanceMonitor:record_histogram(name, value)
    if not self.histograms[name] then
        self.histograms[name] = {
            values = {},
            count = 0,
            sum = 0,
            min = math.huge,
            max = -math.huge
        }
    end
    
    local hist = self.histograms[name]
    table.insert(hist.values, value)
    hist.count = hist.count + 1
    hist.sum = hist.sum + value
    hist.min = math.min(hist.min, value)
    hist.max = math.max(hist.max, value)
end

function PerformanceMonitor:get_histogram_stats(name)
    local hist = self.histograms[name]
    if not hist or hist.count == 0 then
        return nil
    end
    
    -- Sort values for percentile calculation
    local sorted_values = {}
    for _, v in ipairs(hist.values) do
        table.insert(sorted_values, v)
    end
    table.sort(sorted_values)
    
    local function percentile(p)
        local index = math.ceil(p * hist.count / 100)
        return sorted_values[math.max(1, index)]
    end
    
    return {
        count = hist.count,
        sum = hist.sum,
        min = hist.min,
        max = hist.max,
        avg = hist.sum / hist.count,
        p50 = percentile(50),
        p90 = percentile(90),
        p95 = percentile(95),
        p99 = percentile(99)
    }
end

function PerformanceMonitor:get_metrics_report()
    local report = {
        timestamp = os.date("%Y-%m-%d %H:%M:%S"),
        counters = self.counters,
        gauges = self.gauges,
        histograms = {}
    }
    
    for name, _ in pairs(self.histograms) do
        report.histograms[name] = self:get_histogram_stats(name)
    end
    
    return report
end

-- Test performance monitoring
local monitor = PerformanceMonitor:new()

-- Simulate various operations
print("   Simulating operations...")

for i = 1, 10 do
    monitor:start_timer("api_call")
    
    -- Simulate API call with variable latency
    local latency = math.random(10, 200) / 1000 -- 10-200ms
    local end_time = os.clock() + latency
    while os.clock() < end_time do end
    
    local duration = monitor:end_timer("api_call")
    monitor:increment_counter("api_calls_total")
    
    if duration > 100 then
        monitor:increment_counter("slow_api_calls")
    end
    
    -- Simulate concurrent users
    monitor:set_gauge("active_users", math.random(50, 200))
    
    -- Simulate memory usage
    monitor:set_gauge("memory_usage_mb", math.random(100, 500))
end

-- Generate metrics report
local report = monitor:get_metrics_report()

print(string.format("   📊 Performance Report (%s):", report.timestamp))
print("   Counters:")
for name, value in pairs(report.counters) do
    print(string.format("     %s: %d", name, value))
end

print("   Gauges:")
for name, value in pairs(report.gauges) do
    print(string.format("     %s: %.1f", name, value))
end

print("   Histograms:")
for name, stats in pairs(report.histograms) do
    print(string.format("     %s:", name))
    print(string.format("       Count: %d, Avg: %.2fms", stats.count, stats.avg))
    print(string.format("       P50: %.2fms, P95: %.2fms, P99: %.2fms", 
        stats.p50, stats.p95, stats.p99))
end

print()

-- ============================================================
-- Pattern 2: Performance Alerting System
-- ============================================================

print("2. Performance Alerting System")
print("-" .. string.rep("-", 40))

local AlertingSystem = {}
AlertingSystem.__index = AlertingSystem

function AlertingSystem:new()
    return setmetatable({
        alert_rules = {},
        alert_history = {},
        alert_states = {}
    }, self)
end

function AlertingSystem:add_rule(name, rule_config)
    self.alert_rules[name] = {
        name = name,
        metric_name = rule_config.metric_name,
        condition = rule_config.condition, -- "greater_than", "less_than", "equals"
        threshold = rule_config.threshold,
        duration = rule_config.duration or 0, -- How long condition must persist
        severity = rule_config.severity or "warning", -- "info", "warning", "critical"
        message = rule_config.message or ("Alert: " .. name),
        cooldown = rule_config.cooldown or 300 -- 5 minutes
    }
    
    self.alert_states[name] = {
        triggered = false,
        first_triggered = nil,
        last_triggered = nil,
        last_resolved = nil
    }
    
    print(string.format("   📋 Added alert rule: %s", name))
end

function AlertingSystem:check_alerts(metrics)
    local current_time = os.time()
    
    for rule_name, rule in pairs(self.alert_rules) do
        local metric_value = metrics[rule.metric_name]
        local state = self.alert_states[rule_name]
        
        if metric_value then
            local condition_met = self:evaluate_condition(
                metric_value, rule.condition, rule.threshold)
            
            if condition_met then
                if not state.triggered then
                    state.first_triggered = current_time
                end
                
                -- Check if duration requirement is met
                local duration_met = (current_time - (state.first_triggered or current_time)) >= rule.duration
                
                if duration_met and not state.triggered then
                    self:trigger_alert(rule_name, rule, metric_value)
                    state.triggered = true
                    state.last_triggered = current_time
                end
            else
                if state.triggered then
                    self:resolve_alert(rule_name, rule)
                    state.triggered = false
                    state.last_resolved = current_time
                    state.first_triggered = nil
                end
            end
        end
    end
end

function AlertingSystem:evaluate_condition(value, condition, threshold)
    if condition == "greater_than" then
        return value > threshold
    elseif condition == "less_than" then
        return value < threshold
    elseif condition == "equals" then
        return value == threshold
    else
        return false
    end
end

function AlertingSystem:trigger_alert(rule_name, rule, metric_value)
    local alert = {
        rule_name = rule_name,
        severity = rule.severity,
        message = rule.message,
        metric_value = metric_value,
        threshold = rule.threshold,
        timestamp = os.time()
    }
    
    table.insert(self.alert_history, alert)
    
    local severity_icon = {
        info = "ℹ️",
        warning = "⚠️",
        critical = "🚨"
    }
    
    print(string.format("   %s ALERT: %s (value: %.2f, threshold: %.2f)", 
        severity_icon[rule.severity], rule.message, metric_value, rule.threshold))
end

function AlertingSystem:resolve_alert(rule_name, rule)
    print(string.format("   ✅ RESOLVED: %s", rule.message))
end

function AlertingSystem:get_alert_summary()
    local summary = {
        total_alerts = #self.alert_history,
        by_severity = {},
        active_alerts = 0
    }
    
    for _, alert in ipairs(self.alert_history) do
        summary.by_severity[alert.severity] = (summary.by_severity[alert.severity] or 0) + 1
    end
    
    for _, state in pairs(self.alert_states) do
        if state.triggered then
            summary.active_alerts = summary.active_alerts + 1
        end
    end
    
    return summary
end

-- Test alerting system
local alerting = AlertingSystem:new()

-- Define alert rules
alerting:add_rule("high_latency", {
    metric_name = "avg_latency",
    condition = "greater_than",
    threshold = 100,
    duration = 1,
    severity = "warning",
    message = "API latency is high"
})

alerting:add_rule("low_memory", {
    metric_name = "available_memory",
    condition = "less_than", 
    threshold = 100,
    severity = "critical",
    message = "Available memory is critically low"
})

alerting:add_rule("high_error_rate", {
    metric_name = "error_rate_percent",
    condition = "greater_than",
    threshold = 5,
    severity = "critical",
    message = "Error rate is too high"
})

-- Simulate metric changes and alert checking
print("   Monitoring metrics for alerts...")

local test_metrics = {
    {avg_latency = 50, available_memory = 200, error_rate_percent = 1},
    {avg_latency = 120, available_memory = 150, error_rate_percent = 2},
    {avg_latency = 150, available_memory = 80, error_rate_percent = 7},
    {avg_latency = 90, available_memory = 120, error_rate_percent = 3}
}

for i, metrics in ipairs(test_metrics) do
    print(string.format("\n   Metric check %d:", i))
    for name, value in pairs(metrics) do
        print(string.format("     %s: %.1f", name, value))
    end
    alerting:check_alerts(metrics)
end

local summary = alerting:get_alert_summary()
print(string.format("\n   Alert Summary: %d total, %d active", 
    summary.total_alerts, summary.active_alerts))

print()

-- ============================================================
-- Pattern 3: Performance Profiling
-- ============================================================

print("3. Performance Profiling")
print("-" .. string.rep("-", 40))

local Profiler = {}
Profiler.__index = Profiler

function Profiler:new()
    return setmetatable({
        profiles = {},
        active_profiles = {},
        call_stack = {}
    }, self)
end

function Profiler:start_profile(name)
    local profile = {
        name = name,
        start_time = os.clock(),
        calls = {},
        memory_start = collectgarbage("count")
    }
    
    self.active_profiles[name] = profile
    print(string.format("   🔍 Started profiling: %s", name))
end

function Profiler:profile_function_call(func_name, func)
    local active_profile = nil
    for _, profile in pairs(self.active_profiles) do
        active_profile = profile
        break
    end
    
    if not active_profile then
        return func()
    end
    
    -- Track call
    table.insert(self.call_stack, func_name)
    local call_start = os.clock()
    
    if not active_profile.calls[func_name] then
        active_profile.calls[func_name] = {
            count = 0,
            total_time = 0,
            max_time = 0,
            min_time = math.huge
        }
    end
    
    local result = func()
    
    local call_time = (os.clock() - call_start) * 1000
    local call_info = active_profile.calls[func_name]
    call_info.count = call_info.count + 1
    call_info.total_time = call_info.total_time + call_time
    call_info.max_time = math.max(call_info.max_time, call_time)
    call_info.min_time = math.min(call_info.min_time, call_time)
    
    table.remove(self.call_stack)
    return result
end

function Profiler:end_profile(name)
    local profile = self.active_profiles[name]
    if not profile then
        print(string.format("   ⚠️  No active profile: %s", name))
        return nil
    end
    
    profile.end_time = os.clock()
    profile.total_duration = (profile.end_time - profile.start_time) * 1000
    profile.memory_end = collectgarbage("count")
    profile.memory_used = profile.memory_end - profile.memory_start
    
    self.profiles[name] = profile
    self.active_profiles[name] = nil
    
    print(string.format("   ✅ Completed profiling: %s (%.2fms)", 
        name, profile.total_duration))
    
    return profile
end

function Profiler:get_profile_report(name)
    local profile = self.profiles[name]
    if not profile then
        return nil
    end
    
    local report = {
        name = profile.name,
        total_duration = profile.total_duration,
        memory_used = profile.memory_used,
        function_calls = {}
    }
    
    for func_name, call_info in pairs(profile.calls) do
        table.insert(report.function_calls, {
            name = func_name,
            count = call_info.count,
            total_time = call_info.total_time,
            avg_time = call_info.total_time / call_info.count,
            max_time = call_info.max_time,
            min_time = call_info.min_time,
            percentage = (call_info.total_time / profile.total_duration) * 100
        })
    end
    
    -- Sort by total time
    table.sort(report.function_calls, function(a, b)
        return a.total_time > b.total_time
    end)
    
    return report
end

-- Test profiling
local profiler = Profiler:new()

profiler:start_profile("data_processing")

-- Simulate profiled functions
local function expensive_operation()
    local end_time = os.clock() + 0.05 -- 50ms
    while os.clock() < end_time do end
    return "expensive_result"
end

local function fast_operation()
    local end_time = os.clock() + 0.01 -- 10ms
    while os.clock() < end_time do end
    return "fast_result"
end

local function data_transformation()
    local end_time = os.clock() + 0.02 -- 20ms
    while os.clock() < end_time do end
    return "transformed_data"
end

-- Execute profiled operations
for i = 1, 5 do
    profiler:profile_function_call("expensive_operation", expensive_operation)
    
    for j = 1, 3 do
        profiler:profile_function_call("fast_operation", fast_operation)
    end
    
    profiler:profile_function_call("data_transformation", data_transformation)
end

local profile = profiler:end_profile("data_processing")
local report = profiler:get_profile_report("data_processing")

print(string.format("   📈 Profile Report: %s", report.name))
print(string.format("   Total duration: %.2fms", report.total_duration))
print(string.format("   Memory used: %.2f KB", report.memory_used))
print("   Function breakdown:")

for _, func_call in ipairs(report.function_calls) do
    print(string.format("     %s: %.2fms (%.1f%%) - %d calls, avg: %.2fms", 
        func_call.name, func_call.total_time, func_call.percentage,
        func_call.count, func_call.avg_time))
end

print()

-- ============================================================
-- Pattern 4: Performance Optimization Recommendations
-- ============================================================

print("4. Performance Optimization Recommendations")
print("-" .. string.rep("-", 40))

local OptimizationAdvisor = {}
OptimizationAdvisor.__index = OptimizationAdvisor

function OptimizationAdvisor:new()
    return setmetatable({
        performance_data = {},
        optimization_rules = {},
        recommendations = {}
    }, self)
end

function OptimizationAdvisor:add_optimization_rule(rule)
    table.insert(self.optimization_rules, rule)
end

function OptimizationAdvisor:analyze_performance(performance_report)
    self.recommendations = {}
    
    for _, rule in ipairs(self.optimization_rules) do
        local recommendation = rule.analyzer(performance_report)
        if recommendation then
            recommendation.priority = rule.priority or "medium"
            recommendation.category = rule.category or "general"
            table.insert(self.recommendations, recommendation)
        end
    end
    
    -- Sort by priority (critical > high > medium > low)
    local priority_order = {critical = 4, high = 3, medium = 2, low = 1}
    table.sort(self.recommendations, function(a, b)
        return (priority_order[a.priority] or 0) > (priority_order[b.priority] or 0)
    end)
    
    return self.recommendations
end

function OptimizationAdvisor:get_recommendations_report()
    local report = {
        total_recommendations = #self.recommendations,
        by_priority = {},
        by_category = {}
    }
    
    for _, rec in ipairs(self.recommendations) do
        report.by_priority[rec.priority] = (report.by_priority[rec.priority] or 0) + 1
        report.by_category[rec.category] = (report.by_category[rec.category] or 0) + 1
    end
    
    return report
end

-- Create optimization advisor with rules
local advisor = OptimizationAdvisor:new()

-- Add optimization rules
advisor:add_optimization_rule({
    category = "latency",
    priority = "high",
    analyzer = function(report)
        if report.histograms and report.histograms.api_call_duration then
            local stats = report.histograms.api_call_duration
            if stats.p95 > 200 then
                return {
                    title = "High API Latency Detected",
                    description = string.format("95th percentile latency is %.2fms, target is <200ms", stats.p95),
                    impact = "high",
                    effort = "medium",
                    suggestions = {
                        "Implement response caching",
                        "Optimize database queries",
                        "Add connection pooling",
                        "Consider CDN for static assets"
                    }
                }
            end
        end
        return nil
    end
})

advisor:add_optimization_rule({
    category = "memory",
    priority = "critical",
    analyzer = function(report)
        if report.gauges and report.gauges.memory_usage_mb then
            if report.gauges.memory_usage_mb > 400 then
                return {
                    title = "High Memory Usage",
                    description = string.format("Memory usage is %.1fMB, consider optimization", 
                        report.gauges.memory_usage_mb),
                    impact = "high",
                    effort = "high",
                    suggestions = {
                        "Profile memory allocation patterns",
                        "Implement object pooling",
                        "Optimize data structures",
                        "Add garbage collection tuning"
                    }
                }
            end
        end
        return nil
    end
})

advisor:add_optimization_rule({
    category = "error_rate",
    priority = "critical",
    analyzer = function(report)
        if report.counters then
            local total_calls = report.counters.api_calls_total or 0
            local slow_calls = report.counters.slow_api_calls or 0
            
            if total_calls > 0 then
                local slow_percentage = (slow_calls / total_calls) * 100
                if slow_percentage > 20 then
                    return {
                        title = "High Slow Request Rate",
                        description = string.format("%.1f%% of requests are slow (>100ms)", slow_percentage),
                        impact = "medium",
                        effort = "medium",
                        suggestions = {
                            "Implement request timeouts",
                            "Add retry logic with backoff",
                            "Optimize slow endpoints",
                            "Consider async processing"
                        }
                    }
                end
            end
        end
        return nil
    end
})

-- Analyze performance using previous metrics
local recommendations = advisor:analyze_performance(report)
local rec_report = advisor:get_recommendations_report()

print(string.format("   🎯 Optimization Analysis: %d recommendations", 
    rec_report.total_recommendations))

if #recommendations > 0 then
    for i, rec in ipairs(recommendations) do
        local priority_icon = {critical = "🚨", high = "⚠️", medium = "📋", low = "💡"}
        print(string.format("\n   %s %s [%s priority]", 
            priority_icon[rec.priority], rec.title, rec.priority))
        print(string.format("     %s", rec.description))
        print(string.format("     Impact: %s, Effort: %s", rec.impact, rec.effort))
        print("     Suggestions:")
        for _, suggestion in ipairs(rec.suggestions) do
            print(string.format("       • %s", suggestion))
        end
    end
else
    print("   ✅ No optimization recommendations at this time")
end

print()
print("🎯 Key Takeaways:")
print("   • Monitor key performance metrics continuously")
print("   • Set up automated alerting for critical thresholds")
print("   • Profile code to identify bottlenecks")
print("   • Generate actionable optimization recommendations")
print("   • Track performance trends over time")
print("   • Balance monitoring overhead with insight value")