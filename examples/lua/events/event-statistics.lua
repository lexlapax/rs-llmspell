-- ABOUTME: Event system monitoring and statistics collection for operational insights
-- ABOUTME: Demonstrates comprehensive metrics, monitoring dashboards, and performance analytics

print("=== Event Statistics Example ===")
print("Demonstrates: System monitoring, metrics collection, performance analytics, and operational insights")
print()

local subscriptions = {}
local statistics_collector = {
    events_published = 0,
    events_received = 0,
    bytes_transferred = 0,
    subscription_count = 0,
    error_count = 0,
    
    -- Time-based metrics
    hourly_stats = {},
    daily_stats = {},
    
    -- Performance metrics
    latency_samples = {},
    throughput_samples = {},
    
    -- Event categorization
    event_types = {},
    pattern_usage = {},
    subscription_categories = {},
    
    -- System health metrics
    memory_usage = {},
    cpu_utilization = {},
    
    -- Error tracking
    error_types = {},
    recovery_stats = {},
    
    -- Custom metrics
    custom_counters = {},
    custom_gauges = {},
    custom_histograms = {}
}

-- Helper function to estimate data size
local function estimate_size(data)
    local function sizeof(obj)
        local size = 0
        if type(obj) == "table" then
            for k, v in pairs(obj) do
                size = size + sizeof(k) + sizeof(v)
            end
        elseif type(obj) == "string" then
            size = size + string.len(obj)
        elseif type(obj) == "number" then
            size = size + 8
        elseif type(obj) == "boolean" then
            size = size + 1
        end
        return size
    end
    return sizeof(data)
end

-- Helper function to record metric
local function record_metric(category, metric_name, value, timestamp)
    timestamp = timestamp or os.time()
    
    if not statistics_collector[category] then
        statistics_collector[category] = {}
    end
    
    if not statistics_collector[category][metric_name] then
        statistics_collector[category][metric_name] = {}
    end
    
    table.insert(statistics_collector[category][metric_name], {
        value = value,
        timestamp = timestamp
    })
    
    print(string.format("   📊 [%s] %s: %s", category, metric_name, tostring(value)))
end

-- Helper function to increment counter
local function increment_counter(counter_name, amount)
    amount = amount or 1
    statistics_collector.custom_counters[counter_name] = 
        (statistics_collector.custom_counters[counter_name] or 0) + amount
end

-- Helper function to set gauge
local function set_gauge(gauge_name, value)
    statistics_collector.custom_gauges[gauge_name] = {
        value = value,
        timestamp = os.time()
    }
end

-- Helper function to record histogram sample
local function record_histogram(histogram_name, value)
    if not statistics_collector.custom_histograms[histogram_name] then
        statistics_collector.custom_histograms[histogram_name] = {}
    end
    table.insert(statistics_collector.custom_histograms[histogram_name], {
        value = value,
        timestamp = os.time()
    })
end

print("1. Setting up monitoring infrastructure:")

print("   📡 Creating monitored subscriptions:")

-- Create subscriptions with monitoring
local monitoring_patterns = {
    {name = "user_activity", pattern = "user.*", category = "user_events"},
    {name = "system_health", pattern = "system.*", category = "health_monitoring"},
    {name = "application_errors", pattern = "*.error", category = "error_tracking"},
    {name = "performance_metrics", pattern = "metrics.*", category = "performance"},
    {name = "business_events", pattern = "business.*", category = "analytics"},
    {name = "security_events", pattern = "security.*", category = "security"},
    {name = "audit_logs", pattern = "audit.*", category = "compliance"}
}

for i, sub_config in ipairs(monitoring_patterns) do
    local start_time = os.clock()
    local sub_id = Event.subscribe(sub_config.pattern)
    local creation_time = (os.clock() - start_time) * 1000
    
    if sub_id then
        subscriptions[sub_config.name] = {
            id = sub_id,
            pattern = sub_config.pattern,
            category = sub_config.category,
            created_at = os.time(),
            events_received = 0,
            bytes_received = 0
        }
        
        statistics_collector.subscription_count = statistics_collector.subscription_count + 1
        statistics_collector.pattern_usage[sub_config.pattern] = 0
        statistics_collector.subscription_categories[sub_config.category] = 
            (statistics_collector.subscription_categories[sub_config.category] or 0) + 1
        
        record_metric("subscription_performance", "creation_time_ms", creation_time)
        increment_counter("subscriptions_created")
        
        print(string.format("   %d. ✅ %s (%s) - %.2fms", 
              i, sub_config.name, sub_config.pattern, creation_time))
    else
        statistics_collector.error_count = statistics_collector.error_count + 1
        record_metric("errors", "subscription_creation_failed", 1)
        increment_counter("subscription_errors")
        print(string.format("   %d. ❌ %s (failed)", i, sub_config.name))
    end
end

print("   📊 Initial monitoring setup complete")

print()
print("2. Event publishing with metrics collection:")

print("   📤 Publishing monitored events:")

-- Generate diverse events for statistical analysis
local test_events = {
    -- User activity events
    {name = "user.login", data = {user_id = "u001", method = "password", timestamp = os.time()}},
    {name = "user.logout", data = {user_id = "u001", session_duration = 3600}},
    {name = "user.profile.update", data = {user_id = "u002", fields = {"email", "name"}}},
    {name = "user.purchase", data = {user_id = "u003", amount = 99.99, currency = "USD"}},
    
    -- System health events
    {name = "system.cpu", data = {usage_percent = 45.2, cores = 8, timestamp = os.time()}},
    {name = "system.memory", data = {used_gb = 12.4, total_gb = 32.0, available_gb = 19.6}},
    {name = "system.disk", data = {used_percent = 67.8, free_gb = 458.2}},
    {name = "system.network", data = {bytes_in = 1048576, bytes_out = 2097152}},
    
    -- Error events
    {name = "application.error", data = {error_code = "E001", message = "Database connection failed"}},
    {name = "user.error", data = {error_code = "E002", user_id = "u004", action = "login_failed"}},
    {name = "system.error", data = {error_code = "E003", component = "payment_processor"}},
    
    -- Performance metrics
    {name = "metrics.response_time", data = {endpoint = "/api/users", duration_ms = 245}},
    {name = "metrics.throughput", data = {requests_per_second = 1250, endpoint = "/api/orders"}},
    {name = "metrics.error_rate", data = {percentage = 2.1, total_requests = 10000}},
    
    -- Business events
    {name = "business.sale", data = {product_id = "p001", revenue = 49.99, quantity = 1}},
    {name = "business.signup", data = {plan = "premium", user_id = "u005", value = 29.99}},
    {name = "business.churn", data = {user_id = "u006", reason = "price", lost_revenue = 99.99}},
    
    -- Security events
    {name = "security.login_attempt", data = {user_id = "u007", ip = "192.168.1.100", success = true}},
    {name = "security.suspicious_activity", data = {user_id = "u008", activity_type = "multiple_failed_logins"}},
    
    -- Audit events
    {name = "audit.user.created", data = {admin_id = "admin001", new_user_id = "u009"}},
    {name = "audit.permission.granted", data = {admin_id = "admin001", user_id = "u010", permission = "admin"}}
}

for i, event in ipairs(test_events) do
    local publish_start = os.clock()
    local data_size = estimate_size(event.data)
    
    local published = Event.publish(event.name, event.data)
    local publish_time = (os.clock() - publish_start) * 1000
    
    if published then
        statistics_collector.events_published = statistics_collector.events_published + 1
        statistics_collector.bytes_transferred = statistics_collector.bytes_transferred + data_size
        
        -- Track event types
        local event_type = event.name:match("^([^.]+)") or "unknown"
        statistics_collector.event_types[event_type] = 
            (statistics_collector.event_types[event_type] or 0) + 1
        
        record_metric("publish_performance", "publish_time_ms", publish_time)
        record_metric("data_volume", "event_size_bytes", data_size)
        increment_counter("events_published")
        increment_counter("events_by_type_" .. event_type)
        record_histogram("publish_latency", publish_time)
        
        print(string.format("   %2d. ✅ %s (~%d bytes, %.2fms)", 
              i, event.name, data_size, publish_time))
    else
        statistics_collector.error_count = statistics_collector.error_count + 1
        record_metric("errors", "publish_failed", 1)
        increment_counter("publish_errors")
        print(string.format("   %2d. ❌ %s (failed)", i, event.name))
    end
end

print(string.format("   📊 Published %d events, %d bytes total", 
      statistics_collector.events_published, statistics_collector.bytes_transferred))

print()
print("3. Event consumption with performance tracking:")

print("   📥 Consuming events with metrics:")

-- Monitor event consumption across all subscriptions
local consumption_stats = {}

for sub_name, sub_info in pairs(subscriptions) do
    local events_consumed = 0
    local total_receive_time = 0
    local total_bytes_received = 0
    
    print(string.format("   🔍 Processing %s events:", sub_name))
    
    -- Consume available events
    for attempt = 1, 5 do
        local receive_start = os.clock()
        local received = Event.receive(sub_info.id, 200) -- 200ms timeout
        local receive_time = (os.clock() - receive_start) * 1000
        
        if received then
            events_consumed = events_consumed + 1
            total_receive_time = total_receive_time + receive_time
            
            local event_size = estimate_size(received.data or {})
            total_bytes_received = total_bytes_received + event_size
            
            statistics_collector.events_received = statistics_collector.events_received + 1
            sub_info.events_received = sub_info.events_received + 1
            sub_info.bytes_received = sub_info.bytes_received + event_size
            
            record_metric("receive_performance", "receive_time_ms", receive_time)
            increment_counter("events_received")
            record_histogram("receive_latency", receive_time)
            
            print(string.format("     %d. %s (~%d bytes, %.2fms)", 
                  attempt, received.event_type or "unknown", event_size, receive_time))
        else
            break -- No more events available
        end
    end
    
    consumption_stats[sub_name] = {
        events_consumed = events_consumed,
        total_receive_time = total_receive_time,
        total_bytes_received = total_bytes_received,
        avg_receive_time = events_consumed > 0 and (total_receive_time / events_consumed) or 0
    }
    
    -- Update pattern usage statistics
    if statistics_collector.pattern_usage[sub_info.pattern] ~= nil then
        statistics_collector.pattern_usage[sub_info.pattern] = 
            statistics_collector.pattern_usage[sub_info.pattern] + events_consumed
    end
    
    print(string.format("   📊 %s: %d events, %d bytes, %.2fms avg", 
          sub_name, events_consumed, total_bytes_received, 
          consumption_stats[sub_name].avg_receive_time))
end

print()
print("4. Real-time statistics dashboard:")

print("   📈 Real-time Statistics Dashboard:")

-- System overview
print("   🖥️  System Overview:")
print(string.format("   • Events Published: %d", statistics_collector.events_published))
print(string.format("   • Events Received: %d", statistics_collector.events_received))
print(string.format("   • Data Transferred: %.2f KB", statistics_collector.bytes_transferred / 1024))
print(string.format("   • Active Subscriptions: %d", statistics_collector.subscription_count))
print(string.format("   • Error Count: %d", statistics_collector.error_count))

-- Event delivery rate
local delivery_rate = statistics_collector.events_published > 0 and
                     (statistics_collector.events_received / statistics_collector.events_published) * 100 or 0
print(string.format("   • Event Delivery Rate: %.1f%%", delivery_rate))

set_gauge("system_delivery_rate", delivery_rate)
set_gauge("active_subscriptions", statistics_collector.subscription_count)
set_gauge("total_events_published", statistics_collector.events_published)

print()
print("   📊 Event Type Distribution:")
for event_type, count in pairs(statistics_collector.event_types) do
    local percentage = (count / statistics_collector.events_published) * 100
    print(string.format("   • %s: %d events (%.1f%%)", event_type, count, percentage))
end

print()
print("   🎯 Pattern Usage Statistics:")
for pattern, usage_count in pairs(statistics_collector.pattern_usage) do
    print(string.format("   • %s: %d events received", pattern, usage_count))
end

print()
print("   📈 Performance Metrics:")

-- Calculate performance statistics
local publish_times = statistics_collector.publish_performance and 
                     statistics_collector.publish_performance.publish_time_ms or {}
local receive_times = statistics_collector.receive_performance and
                     statistics_collector.receive_performance.receive_time_ms or {}

if #publish_times > 0 then
    local total_publish_time = 0
    local min_publish_time = math.huge
    local max_publish_time = 0
    
    for _, time_record in ipairs(publish_times) do
        local time_val = time_record.value
        total_publish_time = total_publish_time + time_val
        min_publish_time = math.min(min_publish_time, time_val)
        max_publish_time = math.max(max_publish_time, time_val)
    end
    
    local avg_publish_time = total_publish_time / #publish_times
    
    print(string.format("   • Publish Latency - Avg: %.2fms, Min: %.2fms, Max: %.2fms", 
          avg_publish_time, min_publish_time, max_publish_time))
    
    set_gauge("avg_publish_latency", avg_publish_time)
    set_gauge("max_publish_latency", max_publish_time)
end

if #receive_times > 0 then
    local total_receive_time = 0
    local min_receive_time = math.huge
    local max_receive_time = 0
    
    for _, time_record in ipairs(receive_times) do
        local time_val = time_record.value
        total_receive_time = total_receive_time + time_val
        min_receive_time = math.min(min_receive_time, time_val)
        max_receive_time = math.max(max_receive_time, time_val)
    end
    
    local avg_receive_time = total_receive_time / #receive_times
    
    print(string.format("   • Receive Latency - Avg: %.2fms, Min: %.2fms, Max: %.2fms", 
          avg_receive_time, min_receive_time, max_receive_time))
    
    set_gauge("avg_receive_latency", avg_receive_time)
    set_gauge("max_receive_latency", max_receive_time)
end

print()
print("5. Historical trend analysis:")

print("   📅 Historical Trend Analysis:")

-- Simulate historical data collection
local current_hour = os.date("%H", os.time())
local current_day = os.date("%Y-%m-%d", os.time())

-- Record hourly statistics
if not statistics_collector.hourly_stats[current_hour] then
    statistics_collector.hourly_stats[current_hour] = {
        events_published = 0,
        events_received = 0,
        errors = 0,
        avg_latency = 0
    }
end

statistics_collector.hourly_stats[current_hour].events_published = 
    statistics_collector.hourly_stats[current_hour].events_published + statistics_collector.events_published
statistics_collector.hourly_stats[current_hour].events_received = 
    statistics_collector.hourly_stats[current_hour].events_received + statistics_collector.events_received
statistics_collector.hourly_stats[current_hour].errors = 
    statistics_collector.hourly_stats[current_hour].errors + statistics_collector.error_count

-- Record daily statistics
if not statistics_collector.daily_stats[current_day] then
    statistics_collector.daily_stats[current_day] = {
        total_events = 0,
        total_bytes = 0,
        peak_throughput = 0,
        avg_delivery_rate = 0
    }
end

statistics_collector.daily_stats[current_day].total_events = 
    statistics_collector.events_published + statistics_collector.events_received
statistics_collector.daily_stats[current_day].total_bytes = statistics_collector.bytes_transferred
statistics_collector.daily_stats[current_day].avg_delivery_rate = delivery_rate

print("   📊 Current Hour Stats (" .. current_hour .. ":00):")
print(string.format("   • Published: %d, Received: %d, Errors: %d", 
      statistics_collector.hourly_stats[current_hour].events_published,
      statistics_collector.hourly_stats[current_hour].events_received,
      statistics_collector.hourly_stats[current_hour].errors))

print("   📊 Current Day Stats (" .. current_day .. "):")
print(string.format("   • Total Events: %d, Bytes: %d, Delivery Rate: %.1f%%", 
      statistics_collector.daily_stats[current_day].total_events,
      statistics_collector.daily_stats[current_day].total_bytes,
      statistics_collector.daily_stats[current_day].avg_delivery_rate))

print()
print("6. Custom metrics and KPIs:")

print("   🎯 Custom Business Metrics:")

-- Calculate custom business KPIs
local business_metrics = {
    user_engagement_score = 0,
    system_health_score = 0,
    error_rate_percentage = 0,
    throughput_efficiency = 0
}

-- User engagement score (based on user events)
local user_events = statistics_collector.event_types["user"] or 0
business_metrics.user_engagement_score = math.min(100, user_events * 10)

-- System health score (based on system events vs errors)
local system_events = statistics_collector.event_types["system"] or 0
local error_events = statistics_collector.event_types["application"] or 0
business_metrics.system_health_score = system_events > 0 and 
    math.max(0, 100 - ((error_events / system_events) * 100)) or 100

-- Error rate percentage
business_metrics.error_rate_percentage = statistics_collector.events_published > 0 and
    (statistics_collector.error_count / statistics_collector.events_published) * 100 or 0

-- Throughput efficiency (events per second estimate)
local session_duration = 10 -- Assume 10 second session for demo
business_metrics.throughput_efficiency = statistics_collector.events_published / session_duration

-- Set custom gauges
set_gauge("user_engagement_score", business_metrics.user_engagement_score)
set_gauge("system_health_score", business_metrics.system_health_score)
set_gauge("error_rate_percentage", business_metrics.error_rate_percentage)
set_gauge("throughput_efficiency", business_metrics.throughput_efficiency)

print(string.format("   • User Engagement Score: %.1f/100", business_metrics.user_engagement_score))
print(string.format("   • System Health Score: %.1f/100", business_metrics.system_health_score))
print(string.format("   • Error Rate: %.2f%%", business_metrics.error_rate_percentage))
print(string.format("   • Throughput Efficiency: %.1f events/sec", business_metrics.throughput_efficiency))

print()
print("7. Alerting and threshold monitoring:")

print("   🚨 Alerting System:")

-- Define alert thresholds
local alert_thresholds = {
    max_error_rate = 5.0, -- 5% error rate
    min_delivery_rate = 90.0, -- 90% delivery rate
    max_avg_latency = 100.0, -- 100ms average latency
    min_system_health = 80.0 -- 80% system health score
}

-- Check thresholds and generate alerts
local alerts = {}

if business_metrics.error_rate_percentage > alert_thresholds.max_error_rate then
    table.insert(alerts, {
        severity = "WARNING",
        metric = "Error Rate",
        current = business_metrics.error_rate_percentage,
        threshold = alert_thresholds.max_error_rate,
        message = "Error rate exceeds acceptable threshold"
    })
end

if delivery_rate < alert_thresholds.min_delivery_rate then
    table.insert(alerts, {
        severity = "CRITICAL",
        metric = "Delivery Rate", 
        current = delivery_rate,
        threshold = alert_thresholds.min_delivery_rate,
        message = "Event delivery rate below minimum threshold"
    })
end

if business_metrics.system_health_score < alert_thresholds.min_system_health then
    table.insert(alerts, {
        severity = "WARNING",
        metric = "System Health",
        current = business_metrics.system_health_score,
        threshold = alert_thresholds.min_system_health,
        message = "System health score indicates potential issues"
    })
end

if #alerts > 0 then
    print("   🚨 Active Alerts:")
    for i, alert in ipairs(alerts) do
        print(string.format("   %d. [%s] %s: %.2f (threshold: %.2f) - %s", 
              i, alert.severity, alert.metric, alert.current, alert.threshold, alert.message))
        increment_counter("alerts_" .. alert.severity:lower())
    end
else
    print("   ✅ No active alerts - all metrics within thresholds")
end

print()
print("8. Resource utilization monitoring:")

print("   💾 Resource Utilization:")

-- Simulate resource monitoring
local resource_metrics = {
    memory_usage_mb = 45.2,
    cpu_usage_percent = 23.7,
    disk_io_ops_sec = 156,
    network_bytes_sec = 2048000,
    event_queue_size = 12,
    subscription_memory_kb = 89.4
}

for metric_name, value in pairs(resource_metrics) do
    set_gauge("resource_" .. metric_name, value)
    print(string.format("   • %s: %s", metric_name:gsub("_", " "):gsub("^%l", string.upper), tostring(value)))
end

-- Calculate resource efficiency
local events_per_mb = resource_metrics.memory_usage_mb > 0 and
                     statistics_collector.events_published / resource_metrics.memory_usage_mb or 0
local events_per_cpu_percent = resource_metrics.cpu_usage_percent > 0 and
                              statistics_collector.events_published / resource_metrics.cpu_usage_percent or 0

print(string.format("   📊 Resource Efficiency - Events per MB: %.1f, Events per CPU%%: %.1f", 
      events_per_mb, events_per_cpu_percent))

print()
print("9. Comprehensive statistics export:")

print("   📋 Statistics Export:")

-- Generate comprehensive statistics report
local statistics_report = {
    timestamp = os.time(),
    session_duration = 60, -- estimated session duration in seconds
    
    -- Core metrics
    core_metrics = {
        events_published = statistics_collector.events_published,
        events_received = statistics_collector.events_received,
        bytes_transferred = statistics_collector.bytes_transferred,
        subscription_count = statistics_collector.subscription_count,
        error_count = statistics_collector.error_count,
        delivery_rate_percent = delivery_rate
    },
    
    -- Performance metrics
    performance_metrics = {
        avg_publish_latency = statistics_collector.custom_gauges["avg_publish_latency"] and 
                             statistics_collector.custom_gauges["avg_publish_latency"].value or 0,
        avg_receive_latency = statistics_collector.custom_gauges["avg_receive_latency"] and
                             statistics_collector.custom_gauges["avg_receive_latency"].value or 0,
        throughput_eps = business_metrics.throughput_efficiency
    },
    
    -- Business metrics
    business_metrics = business_metrics,
    
    -- Resource metrics
    resource_metrics = resource_metrics,
    
    -- Event distribution
    event_type_distribution = statistics_collector.event_types,
    pattern_usage = statistics_collector.pattern_usage,
    
    -- Custom counters and gauges
    custom_counters = statistics_collector.custom_counters,
    custom_gauges = (function()
        local gauges = {}
        for name, gauge in pairs(statistics_collector.custom_gauges) do
            gauges[name] = gauge.value
        end
        return gauges
    end)()
}

print("   📊 Statistics Report Generated:")
print(string.format("   • Report timestamp: %s", os.date("%Y-%m-%d %H:%M:%S", statistics_report.timestamp)))
print(string.format("   • Core metrics: %d fields", (function()
    local count = 0
    for _ in pairs(statistics_report.core_metrics) do count = count + 1 end
    return count
end)()))
print(string.format("   • Performance metrics: %d fields", (function()
    local count = 0
    for _ in pairs(statistics_report.performance_metrics) do count = count + 1 end
    return count
end)()))
print(string.format("   • Business metrics: %d fields", (function()
    local count = 0
    for _ in pairs(statistics_report.business_metrics) do count = count + 1 end
    return count
end)()))
print(string.format("   • Custom counters: %d", (function()
    local count = 0
    for _ in pairs(statistics_report.custom_counters) do count = count + 1 end
    return count
end)()))

print()
print("10. Statistics best practices:")

print("   💡 Event Statistics Best Practices:")
print("   • Collect metrics at multiple granularities (real-time, hourly, daily)")
print("   • Monitor both technical and business metrics")
print("   • Set up alerting for critical thresholds")
print("   • Track resource utilization alongside event metrics")
print("   • Use histograms for latency and duration measurements")
print("   • Implement custom counters for domain-specific metrics")
print("   • Export statistics for external analysis and dashboards")
print("   • Monitor event delivery rates and error patterns")
print("   • Track subscription lifecycle and usage patterns")
print("   • Correlate performance metrics with system load")

print()
print("11. Final statistics summary:")

print("   📊 Session Summary:")
print("   • Total events published:", statistics_collector.events_published)
print("   • Total events received:", statistics_collector.events_received)
print(string.format("   • Total data transferred: %.2f KB", statistics_collector.bytes_transferred / 1024))
print("   • Active subscriptions:", statistics_collector.subscription_count)
print("   • Errors encountered:", statistics_collector.error_count)
print(string.format("   • Overall delivery rate: %.1f%%", delivery_rate))
print("   • Custom counters defined:", (function()
    local count = 0
    for _ in pairs(statistics_collector.custom_counters) do count = count + 1 end
    return count
end)())
print("   • Custom gauges tracked:", (function()
    local count = 0
    for _ in pairs(statistics_collector.custom_gauges) do count = count + 1 end
    return count
end)())
print("   • Alert conditions checked:", #alert_thresholds)

print()
print("12. Cleaning up monitoring subscriptions:")

local cleanup_count = 0
for name, sub_info in pairs(subscriptions) do
    local unsubscribed = Event.unsubscribe(sub_info.id)
    if unsubscribed then
        cleanup_count = cleanup_count + 1
        increment_counter("subscriptions_cleaned")
        print(string.format("   🧹 Unsubscribed from %s (%d events, %d bytes)", 
              name, sub_info.events_received, sub_info.bytes_received))
    end
end

local final_subs = Event.list_subscriptions()
print("   ✅ Cleaned up", cleanup_count, "monitoring subscriptions")
print("   ✅ Final subscription count:", #final_subs)

print()
print("✨ Event statistics example complete!")
print("   Key concepts demonstrated:")
print("   • Comprehensive metrics collection and monitoring")
print("   • Real-time dashboard with system overview")
print("   • Historical trend analysis and data retention")
print("   • Custom business metrics and KPI calculation")
print("   • Alerting system with threshold monitoring")
print("   • Resource utilization tracking and efficiency metrics")
print("   • Performance analysis with latency and throughput")
print("   • Event distribution and pattern usage analytics")
print("   • Statistics export for external analysis")
print("   • Best practices for operational monitoring")