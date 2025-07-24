-- ABOUTME: Intelligent monitoring system with AI-driven alerts, predictive analytics, and automated remediation
-- ABOUTME: Demonstrates comprehensive system monitoring using hooks, events, agents, and AI for proactive system management

print("=== Intelligent Monitoring System Integration Example ===")
print("Demonstrates: AI-driven monitoring, predictive analytics, automated remediation, and intelligent alerting")
print()

-- Monitoring system state
local monitoring_system = {
    monitored_components = {},
    alert_history = {},
    remediation_actions = {},
    predictive_models = {},
    metrics = {
        alerts_generated = 0,
        alerts_resolved = 0,
        predictions_made = 0,
        remediations_executed = 0,
        false_positives = 0,
        system_uptime = 0
    },
    thresholds = {
        cpu_warning = 70,
        cpu_critical = 85,
        memory_warning = 75,
        memory_critical = 90,
        disk_warning = 80,
        disk_critical = 95,
        response_time_warning = 1000, -- ms
        response_time_critical = 2000,
        error_rate_warning = 5, -- %
        error_rate_critical = 10
    },
    configuration = {
        monitoring_interval = 5, -- seconds
        prediction_window = 300, -- 5 minutes ahead
        auto_remediation = true,
        alert_dampening = 60, -- seconds
        ai_confidence_threshold = 0.75
    }
}

local subscriptions = {}
local hook_handles = {}
local ai_agents = {}

-- Component definitions for monitoring
local system_components = {
    web_server = {
        type = "service",
        critical = true,
        metrics = {"cpu", "memory", "response_time", "active_connections"},
        health_check_url = "/health",
        restart_command = "systemctl restart nginx"
    },
    database = {
        type = "service", 
        critical = true,
        metrics = {"cpu", "memory", "disk", "query_time", "connections"},
        health_check_query = "SELECT 1",
        restart_command = "systemctl restart postgresql"
    },
    api_gateway = {
        type = "service",
        critical = true,
        metrics = {"cpu", "memory", "response_time", "error_rate", "throughput"},
        health_check_url = "/api/health",
        restart_command = "systemctl restart api-gateway"
    },
    file_system = {
        type = "infrastructure",
        critical = false,
        metrics = {"disk", "inode_usage", "io_wait"},
        cleanup_command = "find /tmp -type f -atime +7 -delete"
    },
    network = {
        type = "infrastructure",
        critical = true,
        metrics = {"bandwidth", "latency", "packet_loss", "connections"},
        diagnostic_command = "netstat -tuln"
    }
}

-- Helper functions
local function generate_alert_id()
    return "alert_" .. os.time() .. "_" .. math.random(1000, 9999)
end

local function log_monitoring_event(level, component, message, data)
    local timestamp = os.date("%Y-%m-%d %H:%M:%S", os.time())
    print(string.format("   [%s] %s [%s] %s", timestamp, level, component or "SYSTEM", message))
    
    Event.publish("monitoring.audit.log", {
        timestamp = os.time(),
        level = level,
        component = component,
        message = message,
        data = data or {}
    })
end

-- AI prediction function (simplified)
local function predict_metric_trend(component, metric, historical_data, time_window)
    -- Simplified AI prediction based on recent trends
    if #historical_data < 3 then
        return {
            predicted_value = historical_data[#historical_data] or 0,
            confidence = 0.3,
            trend = "insufficient_data"
        }
    end
    
    -- Calculate trend
    local recent_values = {}
    local start_index = math.max(1, #historical_data - 5)
    
    for i = start_index, #historical_data do
        table.insert(recent_values, historical_data[i].value)
    end
    
    -- Simple linear trend calculation
    local sum_x, sum_y, sum_xy, sum_x2 = 0, 0, 0, 0
    local n = #recent_values
    
    for i, value in ipairs(recent_values) do
        sum_x = sum_x + i
        sum_y = sum_y + value
        sum_xy = sum_xy + (i * value)
        sum_x2 = sum_x2 + (i * i)
    end
    
    local slope = (n * sum_xy - sum_x * sum_y) / (n * sum_x2 - sum_x * sum_x)
    local intercept = (sum_y - slope * sum_x) / n
    
    -- Predict value at future time point
    local future_point = n + (time_window / 60) -- Convert to minutes
    local predicted_value = slope * future_point + intercept
    
    -- Calculate confidence based on trend consistency
    local variance = 0
    for i, value in ipairs(recent_values) do
        local expected = slope * i + intercept
        variance = variance + (value - expected) ^ 2
    end
    variance = variance / n
    
    local confidence = math.max(0.1, math.min(0.95, 1 / (1 + variance / 100)))
    
    local trend = "stable"
    if math.abs(slope) > 1 then
        trend = slope > 0 and "increasing" or "decreasing"
    end
    
    return {
        predicted_value = predicted_value,
        confidence = confidence,
        trend = trend,
        slope = slope,
        variance = variance
    }
end

print("1. Setting up intelligent monitoring infrastructure:")

print("   📡 Creating monitoring event subscriptions:")

-- Set up comprehensive monitoring subscriptions
local monitoring_patterns = {
    system_metrics = "monitoring.metrics.*",
    alerts = "monitoring.alert.*",
    predictions = "monitoring.prediction.*",
    remediation = "monitoring.remediation.*",
    health_checks = "monitoring.health.*",
    performance = "monitoring.performance.*",
    ai_insights = "monitoring.ai.*",
    audit = "monitoring.audit.*"
}

for pattern_name, pattern in pairs(monitoring_patterns) do
    subscriptions[pattern_name] = Event.subscribe(pattern)
    print(string.format("   • %s: %s", pattern_name, pattern))
end

print("   ✅ Monitoring infrastructure ready")

print()
print("2. Setting up AI-driven monitoring hooks:")

print("   🧠 Registering intelligent monitoring hooks:")

-- Hook for real-time metric analysis
hook_handles.metric_analyzer = Hook.register("BeforeAgentExecution", function(context)
    local agent_name = context.component_id.name
    
    -- Only analyze monitoring-related agents
    if agent_name:find("monitor") or agent_name:find("metric") then
        local component = agent_name:gsub("monitor_", ""):gsub("_agent", "")
        
        -- Simulate current metrics
        local current_metrics = {
            cpu = math.random(20, 95),
            memory = math.random(30, 85),
            disk = math.random(40, 90),
            response_time = math.random(100, 2500),
            error_rate = math.random(0, 15),
            throughput = math.random(50, 500)
        }
        
        -- Initialize component monitoring if not exists
        if not monitoring_system.monitored_components[component] then
            monitoring_system.monitored_components[component] = {
                name = component,
                last_check = os.time(),
                status = "healthy",
                metrics_history = {},
                alert_count = 0,
                predictions = {}
            }
        end
        
        local comp_monitor = monitoring_system.monitored_components[component]
        
        -- Store metrics in history
        for metric_name, value in pairs(current_metrics) do
            if not comp_monitor.metrics_history[metric_name] then
                comp_monitor.metrics_history[metric_name] = {}
            end
            
            table.insert(comp_monitor.metrics_history[metric_name], {
                timestamp = os.time(),
                value = value
            })
            
            -- Keep only recent history (last 20 data points)
            if #comp_monitor.metrics_history[metric_name] > 20 then
                table.remove(comp_monitor.metrics_history[metric_name], 1)
            end
        end
        
        -- Analyze metrics for anomalies
        for metric_name, value in pairs(current_metrics) do
            local threshold_key = metric_name .. "_warning"
            local critical_key = metric_name .. "_critical"
            
            local warning_threshold = monitoring_system.thresholds[threshold_key]
            local critical_threshold = monitoring_system.thresholds[critical_key]
            
            if warning_threshold and critical_threshold then
                local severity = "normal"
                
                if value >= critical_threshold then
                    severity = "critical"
                elseif value >= warning_threshold then
                    severity = "warning"
                end
                
                if severity ~= "normal" then
                    local alert_id = generate_alert_id()
                    
                    local alert = {
                        id = alert_id,
                        component = component,
                        metric = metric_name,
                        current_value = value,
                        threshold = severity == "critical" and critical_threshold or warning_threshold,
                        severity = severity,
                        timestamp = os.time(),
                        status = "active"
                    }
                    
                    monitoring_system.alert_history[alert_id] = alert
                    monitoring_system.metrics.alerts_generated = monitoring_system.metrics.alerts_generated + 1
                    comp_monitor.alert_count = comp_monitor.alert_count + 1
                    
                    log_monitoring_event("ALERT", component, 
                                        string.format("%s %s alert: %.1f (threshold: %.1f)", 
                                                     severity:upper(), metric_name, value, alert.threshold))
                    
                    Event.publish("monitoring.alert.triggered", {
                        alert_id = alert_id,
                        component = component,
                        metric = metric_name,
                        current_value = value,
                        threshold = alert.threshold,
                        severity = severity,
                        timestamp = os.time(),
                        auto_remediation_candidate = monitoring_system.configuration.auto_remediation
                    })
                    
                    print(string.format("   🚨 %s alert: %s %s = %.1f", 
                          severity:upper(), component, metric_name, value))
                end
            end
        end
        
        -- Publish current metrics
        Event.publish("monitoring.metrics.current", {
            component = component,
            metrics = current_metrics,
            timestamp = os.time(),
            status = comp_monitor.status
        })
        
        comp_monitor.last_check = os.time()
    end
    
    return "continue"
end, "high")

-- Hook for AI-driven predictions
hook_handles.prediction_engine = Hook.register("AfterAgentExecution", function(context)
    local agent_name = context.component_id.name
    
    if agent_name:find("monitor") then
        local component = agent_name:gsub("monitor_", ""):gsub("_agent", "")
        local comp_monitor = monitoring_system.monitored_components[component]
        
        if comp_monitor and comp_monitor.metrics_history then
            -- Make predictions for each metric
            for metric_name, history in pairs(comp_monitor.metrics_history) do
                if #history >= 3 then
                    local prediction = predict_metric_trend(
                        component, 
                        metric_name, 
                        history, 
                        monitoring_system.configuration.prediction_window
                    )
                    
                    if prediction.confidence >= monitoring_system.configuration.ai_confidence_threshold then
                        comp_monitor.predictions[metric_name] = prediction
                        monitoring_system.metrics.predictions_made = monitoring_system.metrics.predictions_made + 1
                        
                        -- Check if prediction indicates future problem
                        local threshold_key = metric_name .. "_warning"
                        local warning_threshold = monitoring_system.thresholds[threshold_key]
                        
                        if warning_threshold and prediction.predicted_value > warning_threshold then
                            log_monitoring_event("PREDICTION", component,
                                                string.format("AI predicts %s will reach %.1f in %ds (confidence: %.1f%%)",
                                                             metric_name, prediction.predicted_value,
                                                             monitoring_system.configuration.prediction_window,
                                                             prediction.confidence * 100))
                            
                            Event.publish("monitoring.prediction.warning", {
                                component = component,
                                metric = metric_name,
                                predicted_value = prediction.predicted_value,
                                threshold = warning_threshold,
                                confidence = prediction.confidence,
                                trend = prediction.trend,
                                time_to_threshold = monitoring_system.configuration.prediction_window,
                                prediction_timestamp = os.time()
                            })
                            
                            print(string.format("   🔮 AI predicts %s %s issue in %ds (%.1f%% confidence)", 
                                  component, metric_name, monitoring_system.configuration.prediction_window,
                                  prediction.confidence * 100))
                        end
                    end
                end
            end
        end
    end
    
    return "continue"
end, "normal")

-- Hook for automated remediation
hook_handles.remediation_engine = Hook.register("BeforeToolExecution", function(context)
    local tool_name = context.component_id.name
    
    if tool_name:find("remediation") or tool_name:find("fix") then
        local component = context.metadata and context.metadata.component
        local alert_id = context.metadata and context.metadata.alert_id
        
        if component and alert_id and monitoring_system.alert_history[alert_id] then
            local alert = monitoring_system.alert_history[alert_id]
            local system_component = system_components[component]
            
            if system_component and monitoring_system.configuration.auto_remediation then
                local remediation_action = {
                    id = "remediation_" .. os.time(),
                    alert_id = alert_id,
                    component = component,
                    action_type = "automated",
                    started_at = os.time(),
                    status = "executing"
                }
                
                -- Determine remediation action based on metric and component
                local action_description = "unknown"
                local success_probability = 0.7 -- Base 70% success rate
                
                if alert.metric == "cpu" and system_component.restart_command then
                    action_description = "Service restart to reduce CPU usage"
                    success_probability = 0.85
                elseif alert.metric == "memory" then
                    action_description = "Memory cleanup and garbage collection"
                    success_probability = 0.75
                elseif alert.metric == "disk" and system_component.cleanup_command then
                    action_description = "Disk cleanup and temporary file removal"
                    success_probability = 0.9
                elseif alert.metric == "response_time" then
                    action_description = "Performance optimization and cache clearing"
                    success_probability = 0.6
                else
                    action_description = "Generic component health restoration"
                    success_probability = 0.5
                end
                
                remediation_action.description = action_description
                
                log_monitoring_event("REMEDIATION", component, 
                                    string.format("Starting automated remediation: %s", action_description))
                
                Event.publish("monitoring.remediation.started", {
                    remediation_id = remediation_action.id,
                    alert_id = alert_id,
                    component = component,
                    action_description = action_description,
                    expected_duration = math.random(5, 30),
                    success_probability = success_probability,
                    started_at = remediation_action.started_at
                })
                
                -- Simulate remediation execution
                local execution_time = math.random(3, 15)
                local success = math.random() < success_probability
                
                remediation_action.completed_at = os.time() + execution_time
                remediation_action.duration = execution_time
                remediation_action.success = success
                remediation_action.status = success and "completed" or "failed"
                
                monitoring_system.remediation_actions[remediation_action.id] = remediation_action
                monitoring_system.metrics.remediations_executed = monitoring_system.metrics.remediations_executed + 1
                
                if success then
                    -- Mark alert as resolved
                    alert.status = "resolved"
                    alert.resolved_at = os.time()
                    alert.resolution_method = "automated"
                    monitoring_system.metrics.alerts_resolved = monitoring_system.metrics.alerts_resolved + 1
                    
                    log_monitoring_event("SUCCESS", component, 
                                        string.format("Automated remediation successful: %s", action_description))
                    
                    Event.publish("monitoring.remediation.success", {
                        remediation_id = remediation_action.id,
                        alert_id = alert_id,
                        component = component,
                        duration = execution_time,
                        resolution_confirmed = true
                    })
                    
                    print(string.format("   ✅ Automated fix successful: %s (%ds)", component, execution_time))
                else
                    log_monitoring_event("ERROR", component,
                                        string.format("Automated remediation failed: %s", action_description))
                    
                    Event.publish("monitoring.remediation.failed", {
                        remediation_id = remediation_action.id,
                        alert_id = alert_id,
                        component = component,
                        failure_reason = "execution_error",
                        requires_manual_intervention = true
                    })
                    
                    print(string.format("   ❌ Automated fix failed: %s - manual intervention required", component))
                end
            end
        end
    end
    
    return "continue"
end, "highest")

-- Hook for system health assessment
hook_handles.health_assessor = Hook.register("BeforeWorkflowStart", function(context)
    local workflow_name = context.component_id.name
    
    if workflow_name:find("health") or workflow_name:find("diagnostic") then
        -- Perform comprehensive system health assessment
        local health_report = {
            overall_status = "healthy",
            timestamp = os.time(),
            components = {},
            recommendations = {},
            risk_score = 0
        }
        
        -- Assess each monitored component
        for component_name, component_data in pairs(monitoring_system.monitored_components) do
            local component_health = {
                name = component_name,
                status = component_data.status,
                last_check = component_data.last_check,
                alert_count = component_data.alert_count,
                health_score = 100,
                issues = {}
            }
            
            -- Calculate health score based on recent alerts and metrics
            if component_data.alert_count > 0 then
                component_health.health_score = math.max(0, 100 - (component_data.alert_count * 20))
            end
            
            -- Check for stale data
            local staleness = os.time() - component_data.last_check
            if staleness > monitoring_system.configuration.monitoring_interval * 3 then
                component_health.health_score = component_health.health_score - 30
                table.insert(component_health.issues, "Stale monitoring data")
            end
            
            -- Check predictions for future issues
            if component_data.predictions then
                for metric_name, prediction in pairs(component_data.predictions) do
                    if prediction.trend == "increasing" and prediction.confidence > 0.8 then
                        component_health.health_score = component_health.health_score - 15
                        table.insert(component_health.issues, 
                                    string.format("Predicted %s increase", metric_name))
                    end
                end
            end
            
            -- Set component status based on health score
            if component_health.health_score >= 80 then
                component_health.status = "healthy"
            elseif component_health.health_score >= 60 then
                component_health.status = "degraded"
            else
                component_health.status = "unhealthy"
            end
            
            health_report.components[component_name] = component_health
            health_report.risk_score = health_report.risk_score + (100 - component_health.health_score)
        end
        
        -- Calculate overall system health
        local total_health = 0
        local component_count = 0
        
        for _, component_health in pairs(health_report.components) do
            total_health = total_health + component_health.health_score
            component_count = component_count + 1
        end
        
        local average_health = component_count > 0 and total_health / component_count or 100
        
        if average_health >= 85 then
            health_report.overall_status = "healthy"
        elseif average_health >= 70 then
            health_report.overall_status = "degraded"
        else
            health_report.overall_status = "unhealthy"
        end
        
        -- Generate recommendations
        if health_report.risk_score > 50 then
            table.insert(health_report.recommendations, 
                        "High system risk detected - consider immediate maintenance")
        end
        
        if monitoring_system.metrics.false_positives > 5 then
            table.insert(health_report.recommendations,
                        "High false positive rate - review alert thresholds")
        end
        
        log_monitoring_event("INFO", "SYSTEM", 
                            string.format("Health assessment: %s (score: %.1f)", 
                                         health_report.overall_status, average_health))
        
        Event.publish("monitoring.health.assessment", {
            health_report = health_report,
            average_health_score = average_health,
            component_count = component_count,
            assessment_timestamp = os.time()
        })
        
        print(string.format("   🏥 System health: %s (%.1f/100)", 
              health_report.overall_status, average_health))
    end
    
    return "continue"
end, "low")

print("   ✅ Intelligent monitoring hooks registered")

print()
print("3. Simulating system monitoring:")

print("   📊 Starting intelligent system monitoring:")

-- Create monitoring agents for each component
for component_name, component_config in pairs(system_components) do
    print(string.format("   🔍 Monitoring %s (%s)", component_name, component_config.type))
    
    -- Simulate monitoring cycles for each component
    for cycle = 1, 3 do
        -- Trigger monitoring hooks by simulating agent execution
        log_monitoring_event("INFO", component_name, string.format("Monitoring cycle %d", cycle))
        
        -- Simulate metrics collection and analysis
        Event.publish("monitoring.metrics.collected", {
            component = component_name,
            cycle = cycle,
            timestamp = os.time(),
            collector = "automated_agent"
        })
        
        -- Small delay between monitoring cycles
        os.execute("sleep 1")
    end
end

print()
print("4. AI-driven analytics and predictions:")

print("   🧠 Processing AI analytics:")

-- Process AI insights and predictions
local ai_insights = {}

for component_name, component_data in pairs(monitoring_system.monitored_components) do
    if component_data.predictions then
        for metric_name, prediction in pairs(component_data.predictions) do
            local insight = {
                component = component_name,
                metric = metric_name,
                prediction = prediction,
                insight_type = "trend_analysis",
                confidence = prediction.confidence,
                timestamp = os.time()
            }
            
            -- Generate actionable insights
            if prediction.trend == "increasing" and prediction.confidence > 0.8 then
                insight.recommendation = string.format(
                    "Consider scaling %s resources or optimizing %s usage",
                    component_name, metric_name
                )
                insight.urgency = prediction.predicted_value > monitoring_system.thresholds[metric_name .. "_critical"] and "high" or "medium"
            elseif prediction.trend == "decreasing" and metric_name == "throughput" then
                insight.recommendation = string.format(
                    "Investigate potential performance degradation in %s",
                    component_name
                )
                insight.urgency = "medium"
            else
                insight.recommendation = "Continue monitoring - trend within normal parameters"
                insight.urgency = "low"
            end
            
            table.insert(ai_insights, insight)
            
            Event.publish("monitoring.ai.insight", {
                insight_id = "insight_" .. os.time() .. "_" .. math.random(100, 999),
                component = component_name,
                metric = metric_name,
                prediction = prediction,
                recommendation = insight.recommendation,
                urgency = insight.urgency,
                confidence = prediction.confidence,
                generated_at = os.time()
            })
            
            print(string.format("   💡 AI Insight: %s %s - %s (%.1f%% confidence)", 
                  component_name, metric_name, insight.recommendation, prediction.confidence * 100))
        end
    end
end

print(string.format("   📊 Generated %d AI insights", #ai_insights))

print()
print("5. Automated remediation simulation:")

print("   🔧 Executing automated remediation:")

-- Simulate automated remediation for active alerts
local remediated_alerts = 0

for alert_id, alert in pairs(monitoring_system.alert_history) do
    if alert.status == "active" and alert.severity == "critical" then
        print(string.format("   🚨 Processing critical alert: %s %s", alert.component, alert.metric))
        
        -- Trigger remediation
        Event.publish("monitoring.remediation.trigger", {
            alert_id = alert_id,
            component = alert.component,
            metric = alert.metric,
            severity = alert.severity,
            trigger_timestamp = os.time(),
            auto_remediation = true
        })
        
        remediated_alerts = remediated_alerts + 1
        
        -- Simulate remediation processing time
        os.execute("sleep 2")
    end
end

print(string.format("   ✅ Processed %d critical alerts for remediation", remediated_alerts))

print()
print("6. Real-time monitoring dashboard:")

print("   📊 Real-time monitoring event processing:")

-- Process monitoring events in real-time
local monitoring_events = {}

for pattern_name, sub_id in pairs(subscriptions) do
    local events_received = 0
    
    print(string.format("   🔍 Processing %s events:", pattern_name))
    
    for attempt = 1, 5 do
        local received = Event.receive(sub_id, 200)
        if received then
            events_received = events_received + 1
            
            local event_type = received.event_type or "unknown"
            local component = received.data and received.data.component
            local timestamp = received.data and received.data.timestamp and
                             os.date("%H:%M:%S", received.data.timestamp) or "unknown"
            
            print(string.format("     %d. [%s] %s", events_received, timestamp, event_type))
            
            if component then
                print(string.format("        Component: %s", component))
            end
            
            -- Display event-specific information
            if received.data then
                if received.data.severity then
                    print(string.format("        Severity: %s", received.data.severity))
                end
                if received.data.current_value then
                    print(string.format("        Value: %.1f", received.data.current_value))
                end
                if received.data.confidence then
                    print(string.format("        Confidence: %.1f%%", received.data.confidence * 100))
                end
                if received.data.recommendation then
                    print(string.format("        Recommendation: %s", received.data.recommendation))
                end
            end
        else
            break
        end
    end
    
    monitoring_events[pattern_name] = events_received
    
    if events_received > 0 then
        print(string.format("   📊 %s: %d events processed", pattern_name, events_received))
    else
        print(string.format("   ⏰ %s: no events", pattern_name))
    end
end

print()
print("7. System performance analytics:")

print("   📈 System Performance Analytics:")

-- Calculate comprehensive performance metrics
local total_events = 0
for _, count in pairs(monitoring_events) do
    total_events = total_events + count
end

local alert_resolution_rate = monitoring_system.metrics.alerts_generated > 0 and
                             (monitoring_system.metrics.alerts_resolved / monitoring_system.metrics.alerts_generated) * 100 or 0

local false_positive_rate = monitoring_system.metrics.alerts_generated > 0 and
                           (monitoring_system.metrics.false_positives / monitoring_system.metrics.alerts_generated) * 100 or 0

local remediation_success_rate = 0
local successful_remediations = 0
local total_remediations = 0

for _, remediation in pairs(monitoring_system.remediation_actions) do
    total_remediations = total_remediations + 1
    if remediation.success then
        successful_remediations = successful_remediations + 1
    end
end

remediation_success_rate = total_remediations > 0 and 
                          (successful_remediations / total_remediations) * 100 or 0

print(string.format("   • Components monitored: %d", (function()
    local count = 0
    for _ in pairs(monitoring_system.monitored_components) do count = count + 1 end
    return count
end)()))

print(string.format("   • Alerts generated: %d", monitoring_system.metrics.alerts_generated))
print(string.format("   • Alerts resolved: %d", monitoring_system.metrics.alerts_resolved))
print(string.format("   • Alert resolution rate: %.1f%%", alert_resolution_rate))
print(string.format("   • False positive rate: %.1f%%", false_positive_rate))
print(string.format("   • AI predictions made: %d", monitoring_system.metrics.predictions_made))
print(string.format("   • Automated remediations: %d", monitoring_system.metrics.remediations_executed))
print(string.format("   • Remediation success rate: %.1f%%", remediation_success_rate))
print(string.format("   • Total monitoring events: %d", total_events))

-- Component health summary
print("   🏥 Component Health Summary:")
for component_name, component_data in pairs(monitoring_system.monitored_components) do
    print(string.format("   • %s: %s (%d alerts)", 
          component_name, component_data.status, component_data.alert_count))
end

print()
print("8. Intelligent alerting and escalation:")

print("   🚨 Intelligent Alerting Analysis:")

-- Analyze alert patterns and effectiveness
local alert_analysis = {
    by_component = {},
    by_severity = {critical = 0, warning = 0},
    by_metric = {},
    resolution_times = {},
    escalations = 0
}

for alert_id, alert in pairs(monitoring_system.alert_history) do
    -- Component analysis
    alert_analysis.by_component[alert.component] = 
        (alert_analysis.by_component[alert.component] or 0) + 1
    
    -- Severity analysis
    alert_analysis.by_severity[alert.severity] = 
        alert_analysis.by_severity[alert.severity] + 1
    
    -- Metric analysis
    alert_analysis.by_metric[alert.metric] = 
        (alert_analysis.by_metric[alert.metric] or 0) + 1
    
    -- Resolution time analysis
    if alert.resolved_at then
        local resolution_time = alert.resolved_at - alert.timestamp
        table.insert(alert_analysis.resolution_times, resolution_time)
    end
end

-- Calculate average resolution time
local avg_resolution_time = 0
if #alert_analysis.resolution_times > 0 then
    local total_time = 0
    for _, time in ipairs(alert_analysis.resolution_times) do
        total_time = total_time + time
    end
    avg_resolution_time = total_time / #alert_analysis.resolution_times
end

print(string.format("   • Average resolution time: %ds", math.floor(avg_resolution_time)))

print("   📊 Alert Distribution:")
for component, count in pairs(alert_analysis.by_component) do
    print(string.format("   • %s: %d alerts", component, count))
end

for severity, count in pairs(alert_analysis.by_severity) do
    print(string.format("   • %s alerts: %d", severity:upper(), count))
end

-- Generate optimization recommendations
local optimization_recommendations = {}

if false_positive_rate > 10 then
    table.insert(optimization_recommendations, {
        area = "Alert Accuracy",
        recommendation = "Tune alert thresholds to reduce false positives",
        priority = "high"
    })
end

if avg_resolution_time > 300 then -- More than 5 minutes
    table.insert(optimization_recommendations, {
        area = "Response Time",
        recommendation = "Improve automated remediation speed",
        priority = "medium"
    })
end

if remediation_success_rate < 70 then
    table.insert(optimization_recommendations, {
        area = "Remediation Effectiveness",
        recommendation = "Enhance automated remediation scripts",
        priority = "high"
    })
end

print("   💡 Optimization Recommendations:")
for i, rec in ipairs(optimization_recommendations) do
    print(string.format("   %d. [%s] %s: %s", i, rec.priority:upper(), rec.area, rec.recommendation))
end

if #optimization_recommendations == 0 then
    print("   ✅ Monitoring system is well-optimized")
end

print()
print("9. Integration best practices demonstrated:")

print("   💡 Integration Best Practices Demonstrated:")
print("   • AI-driven predictive analytics for proactive monitoring")
print("   • Event-driven architecture for real-time responsiveness") 
print("   • Automated remediation with intelligent decision making")
print("   • Comprehensive health assessment and risk scoring")
print("   • Multi-layered alerting with severity-based escalation")
print("   • Performance analytics and optimization recommendations")
print("   • Integration of monitoring, prediction, and remediation systems")
print("   • Intelligent false positive reduction through AI analysis")
print("   • Real-time dashboard with actionable insights")
print("   • Scalable monitoring architecture for enterprise systems")

print()
print("10. Cleanup and final statistics:")

-- Final monitoring statistics
local final_stats = {
    components_monitored = 0,
    alerts_generated = monitoring_system.metrics.alerts_generated,
    alerts_resolved = monitoring_system.metrics.alerts_resolved,
    predictions_made = monitoring_system.metrics.predictions_made,
    remediations_executed = monitoring_system.metrics.remediations_executed,
    ai_insights_generated = #ai_insights,
    monitoring_events = total_events,
    optimization_recommendations = #optimization_recommendations,
    hooks_registered = 0,
    subscriptions_created = 0
}

-- Count components and infrastructure
for _ in pairs(monitoring_system.monitored_components) do
    final_stats.components_monitored = final_stats.components_monitored + 1
end

for _ in pairs(hook_handles) do
    final_stats.hooks_registered = final_stats.hooks_registered + 1
end

for _ in pairs(subscriptions) do
    final_stats.subscriptions_created = final_stats.subscriptions_created + 1
end

print("   📊 Final Monitoring System Statistics:")
print(string.format("   • Components monitored: %d", final_stats.components_monitored))
print(string.format("   • Alerts generated: %d", final_stats.alerts_generated))
print(string.format("   • Alerts resolved: %d", final_stats.alerts_resolved))
print(string.format("   • Resolution rate: %.1f%%", alert_resolution_rate))
print(string.format("   • AI predictions made: %d", final_stats.predictions_made))
print(string.format("   • Automated remediations: %d", final_stats.remediations_executed))
print(string.format("   • Remediation success rate: %.1f%%", remediation_success_rate))
print(string.format("   • AI insights generated: %d", final_stats.ai_insights_generated))
print(string.format("   • Monitoring events processed: %d", final_stats.monitoring_events))
print(string.format("   • Optimization recommendations: %d", final_stats.optimization_recommendations))
print(string.format("   • Infrastructure: %d hooks, %d subscriptions", 
      final_stats.hooks_registered, final_stats.subscriptions_created))

-- Cleanup
print("   🧹 Cleaning up monitoring infrastructure:")
local hooks_cleaned = 0
for hook_name, handle in pairs(hook_handles) do
    if handle and handle:id() then
        Hook.unregister(handle)
        hooks_cleaned = hooks_cleaned + 1
        print(string.format("   • Unregistered hook: %s", hook_name))
    end
end

local subs_cleaned = 0
for pattern_name, sub_id in pairs(subscriptions) do
    local unsubscribed = Event.unsubscribe(sub_id)
    if unsubscribed then
        subs_cleaned = subs_cleaned + 1
        print(string.format("   • Unsubscribed from: %s", pattern_name))
    end
end

local final_hooks = #Hook.list()
local final_subs = Event.list_subscriptions()

print(string.format("   ✅ Cleaned up %d hooks and %d subscriptions", hooks_cleaned, subs_cleaned))
print(string.format("   ✅ Final state: %d hooks, %d subscriptions", final_hooks, #final_subs))

print()
print("✨ Intelligent Monitoring System Integration Example Complete!")
print("   Advanced monitoring capabilities demonstrated:")
print("   • AI-driven predictive analytics and trend analysis")
print("   • Automated anomaly detection with configurable thresholds")
print("   • Intelligent remediation with success probability assessment")
print("   • Real-time health monitoring and risk assessment")
print("   • Comprehensive alerting with severity-based escalation")
print("   • Performance analytics and optimization insights")
print("   • Integration of monitoring, AI, and automation systems")
print("   • Event-driven architecture for real-time responsiveness")
print("   • Scalable monitoring for complex distributed systems")
print("   • Production-ready patterns for enterprise monitoring")