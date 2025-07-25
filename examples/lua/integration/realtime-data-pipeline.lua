-- ABOUTME: Real-time data pipeline using hooks and events for end-to-end data processing
-- ABOUTME: Demonstrates practical integration of hooks, events, agents, tools, and workflows for data pipeline automation

print("=== Real-time Data Pipeline Integration Example ===")
print("Demonstrates: Complete data pipeline with hooks, events, agents, tools, and workflows")
print()

-- Pipeline state management
local pipeline_state = {
    active_jobs = {},
    completed_jobs = {},
    failed_jobs = {},
    metrics = {
        jobs_processed = 0,
        total_records = 0,
        processing_time = 0,
        error_count = 0,
        throughput_rps = 0
    },
    configuration = {
        batch_size = 100,
        max_concurrent_jobs = 5,
        retry_attempts = 3,
        timeout_seconds = 30
    }
}

local subscriptions = {}
local hook_handles = {}

-- Helper function to generate job ID
local function generate_job_id()
    return "job_" .. os.time() .. "_" .. math.random(1000, 9999)
end

-- Helper function to log pipeline events
local function log_pipeline_event(level, job_id, message, data)
    local timestamp = os.date("%Y-%m-%d %H:%M:%S", os.time())
    print(string.format("   [%s] %s [%s] %s", timestamp, level, job_id or "SYSTEM", message))
    
    -- Publish monitoring event
    Event.publish("pipeline.monitoring.log", {
        timestamp = os.time(),
        level = level,
        job_id = job_id,
        message = message,
        data = data or {}
    })
end

print("1. Setting up data pipeline infrastructure:")

print("   📡 Creating pipeline event subscriptions:")

-- Set up event subscriptions for pipeline coordination
local pipeline_patterns = {
    data_ingestion = "pipeline.data.ingestion.*",
    processing_jobs = "pipeline.jobs.*",
    monitoring = "pipeline.monitoring.*",
    errors = "pipeline.error.*",
    metrics = "pipeline.metrics.*",
    workflow_control = "pipeline.workflow.*",
    alerts = "pipeline.alert.*"
}

for pattern_name, pattern in pairs(pipeline_patterns) do
    subscriptions[pattern_name] = Event.subscribe(pattern)
    print(string.format("   • %s: %s", pattern_name, pattern))
end

print("   ✅ Pipeline event infrastructure ready")

print()
print("2. Setting up pipeline monitoring hooks:")

print("   🪝 Registering pipeline monitoring hooks:")

-- Hook to monitor data ingestion
hook_handles.ingestion_monitor = Hook.register("BeforeToolExecution", function(context)
    local tool_name = context.component_id.name
    
    -- Only monitor data ingestion tools
    if tool_name:find("ingest") or tool_name:find("fetch") or tool_name:find("collect") then
        local job_id = context.correlation_id
        
        log_pipeline_event("INFO", job_id, "Data ingestion started", {
            tool = tool_name,
            timestamp = os.time()
        })
        
        -- Track job start
        if not pipeline_state.active_jobs[job_id] then
            pipeline_state.active_jobs[job_id] = {
                job_id = job_id,
                start_time = os.time(),
                current_stage = "ingestion",
                tool_name = tool_name,
                status = "running",
                records_processed = 0
            }
        end
        
        Event.publish("pipeline.jobs.started", {
            job_id = job_id,
            stage = "ingestion",
            tool_name = tool_name,
            start_time = os.time(),
            expected_duration = math.random(5, 15)
        })
        
        return "continue"
    end
    
    return "continue"
end, "high")

-- Hook to monitor processing completion
hook_handles.processing_monitor = Hook.register("AfterToolExecution", function(context)
    local tool_name = context.component_id.name
    local job_id = context.correlation_id
    
    if pipeline_state.active_jobs[job_id] then
        local job = pipeline_state.active_jobs[job_id]
        local stage_duration = os.time() - job.start_time
        
        -- Simulate processing results
        local records_processed = math.random(50, 500)
        local processing_success = math.random() > 0.1 -- 90% success rate
        
        job.records_processed = job.records_processed + records_processed
        pipeline_state.metrics.total_records = pipeline_state.metrics.total_records + records_processed
        
        if processing_success then
            log_pipeline_event("INFO", job_id, string.format("Stage completed: %d records processed", records_processed))
            
            Event.publish("pipeline.jobs.stage_complete", {
                job_id = job_id,
                stage = job.current_stage,
                tool_name = tool_name,
                records_processed = records_processed,
                stage_duration = stage_duration,
                success = true
            })
            
            -- Move to next stage or complete
            if job.current_stage == "ingestion" then
                job.current_stage = "transformation"
                job.start_time = os.time() -- Reset for next stage
            elseif job.current_stage == "transformation" then
                job.current_stage = "validation"
                job.start_time = os.time()
            elseif job.current_stage == "validation" then
                job.current_stage = "output"
                job.start_time = os.time()
            else
                -- Job completed
                job.status = "completed"
                job.end_time = os.time()
                job.total_duration = job.end_time - (job.start_time - stage_duration)
                
                pipeline_state.completed_jobs[job_id] = job
                pipeline_state.active_jobs[job_id] = nil
                pipeline_state.metrics.jobs_processed = pipeline_state.metrics.jobs_processed + 1
                
                log_pipeline_event("SUCCESS", job_id, string.format("Job completed: %d records in %ds", 
                                  job.records_processed, job.total_duration))
                
                Event.publish("pipeline.jobs.completed", {
                    job_id = job_id,
                    total_records = job.records_processed,
                    total_duration = job.total_duration,
                    success = true
                })
            end
        else
            -- Processing failed
            pipeline_state.metrics.error_count = pipeline_state.metrics.error_count + 1
            job.status = "failed"
            job.error_reason = "processing_failure"
            
            pipeline_state.failed_jobs[job_id] = job
            pipeline_state.active_jobs[job_id] = nil
            
            log_pipeline_event("ERROR", job_id, "Job failed during processing")
            
            Event.publish("pipeline.error.job_failed", {
                job_id = job_id,
                stage = job.current_stage,
                error_reason = "processing_failure",
                records_processed = job.records_processed
            })
        end
    end
    
    return "continue"
end, "high")

-- Hook to monitor system resource usage
hook_handles.resource_monitor = Hook.register("BeforeAgentExecution", function(context)
    -- Simulate resource monitoring
    local cpu_usage = math.random(20, 80)
    local memory_usage = math.random(30, 70)
    local active_jobs = 0
    for _ in pairs(pipeline_state.active_jobs) do
        active_jobs = active_jobs + 1
    end
    
    Event.publish("pipeline.metrics.resources", {
        timestamp = os.time(),
        cpu_usage_percent = cpu_usage,
        memory_usage_percent = memory_usage,
        active_jobs = active_jobs,
        queue_size = math.random(0, 10)
    })
    
    -- Alert if resources are high
    if cpu_usage > 75 or memory_usage > 65 then
        Event.publish("pipeline.alert.high_resource_usage", {
            timestamp = os.time(),
            cpu_usage = cpu_usage,
            memory_usage = memory_usage,
            severity = cpu_usage > 85 and "critical" or "warning"
        })
        
        log_pipeline_event("WARN", nil, string.format("High resource usage: CPU=%d%%, Memory=%d%%", 
                          cpu_usage, memory_usage))
    end
    
    return "continue"
end, "low")

print("   ✅ Pipeline monitoring hooks registered")

print()
print("3. Implementing data pipeline workflow:")

print("   🔄 Starting data pipeline workflow:")

-- Simulate incoming data batches
local data_batches = {
    {
        batch_id = "batch_001",
        source = "user_events",
        record_count = 150,
        format = "json",
        size_mb = 2.4
    },
    {
        batch_id = "batch_002", 
        source = "system_logs",
        record_count = 300,
        format = "csv",
        size_mb = 1.8
    },
    {
        batch_id = "batch_003",
        source = "api_metrics",
        record_count = 75,
        format = "parquet",
        size_mb = 0.9
    },
    {
        batch_id = "batch_004",
        source = "sensor_data",
        record_count = 500,
        format = "binary",
        size_mb = 4.2
    }
}

-- Process each data batch through the pipeline
for i, batch in ipairs(data_batches) do
    local job_id = generate_job_id()
    
    log_pipeline_event("INFO", job_id, string.format("Processing batch %s (%d records, %.1fMB)", 
                      batch.batch_id, batch.record_count, batch.size_mb))
    
    -- Publish data ingestion event
    Event.publish("pipeline.data.ingestion.started", {
        job_id = job_id,
        batch_id = batch.batch_id,
        source = batch.source,
        record_count = batch.record_count,
        format = batch.format,
        size_mb = batch.size_mb,
        timestamp = os.time()
    })
    
    print(string.format("   %d. 📥 Ingesting batch: %s (%d records)", i, batch.batch_id, batch.record_count))
    
    -- Simulate pipeline stages
    local stages = {"ingestion", "transformation", "validation", "output"}
    local job_success = true
    
    for stage_idx, stage in ipairs(stages) do
        local stage_start = os.time()
        local stage_duration = math.random(1, 3) -- 1-3 seconds per stage
        
        Event.publish("pipeline.jobs.stage_started", {
            job_id = job_id,
            batch_id = batch.batch_id,
            stage = stage,
            stage_index = stage_idx,
            total_stages = #stages,
            start_time = stage_start
        })
        
        print(string.format("     %d.%d 🔄 Stage: %s", i, stage_idx, stage))
        
        -- Simulate stage processing time
        os.execute("sleep " .. stage_duration)
        
        -- Simulate stage success/failure
        local stage_success = math.random() > 0.05 -- 95% success rate per stage
        
        if stage_success then
            local records_processed = math.floor(batch.record_count * (0.8 + math.random() * 0.2))
            
            Event.publish("pipeline.jobs.stage_completed", {
                job_id = job_id,
                batch_id = batch.batch_id,
                stage = stage,
                stage_index = stage_idx,
                records_processed = records_processed,
                duration_seconds = stage_duration,
                success = true
            })
            
            print(string.format("     %d.%d ✅ Stage completed: %d records (%ds)", 
                  i, stage_idx, records_processed, stage_duration))
        else
            -- Stage failed
            job_success = false
            
            Event.publish("pipeline.error.stage_failed", {
                job_id = job_id,
                batch_id = batch.batch_id,
                failed_stage = stage,
                stage_index = stage_idx,
                error_type = "processing_error",
                timestamp = os.time()
            })
            
            log_pipeline_event("ERROR", job_id, string.format("Stage failed: %s", stage))
            print(string.format("     %d.%d ❌ Stage failed: %s", i, stage_idx, stage))
            break
        end
    end
    
    -- Complete or fail the job
    if job_success then
        Event.publish("pipeline.jobs.batch_completed", {
            job_id = job_id,
            batch_id = batch.batch_id,
            total_records = batch.record_count,
            completion_time = os.time(),
            success = true
        })
        
        print(string.format("   %d. ✅ Batch completed: %s", i, batch.batch_id))
    else
        Event.publish("pipeline.jobs.batch_failed", {
            job_id = job_id,
            batch_id = batch.batch_id,
            failure_time = os.time(),
            success = false
        })
        
        print(string.format("   %d. ❌ Batch failed: %s", i, batch.batch_id))
    end
    
    -- Small delay between batches
    os.execute("sleep 1")
end

print()
print("4. Real-time pipeline monitoring:")

print("   📊 Processing pipeline monitoring events:")

-- Monitor pipeline events in real-time
local monitoring_results = {}

for pattern_name, sub_id in pairs(subscriptions) do
    local events_received = 0
    
    print(string.format("   🔍 Monitoring %s:", pattern_name))
    
    -- Receive monitoring events
    for attempt = 1, 5 do
        local received = Event.receive(sub_id, 200) -- 200ms timeout
        if received then
            events_received = events_received + 1
            
            local event_type = received.event_type or "unknown"
            local timestamp = received.data and received.data.timestamp and 
                             os.date("%H:%M:%S", received.data.timestamp) or "unknown"
            
            print(string.format("     %d. [%s] %s", events_received, timestamp, event_type))
            
            -- Analyze specific event types
            if received.data then
                if received.data.job_id then
                    print(string.format("        Job: %s", received.data.job_id))
                end
                if received.data.batch_id then
                    print(string.format("        Batch: %s", received.data.batch_id))
                end
                if received.data.records_processed then
                    print(string.format("        Records: %d", received.data.records_processed))
                end
                if received.data.stage then
                    print(string.format("        Stage: %s", received.data.stage))
                end
                if received.data.error_type then
                    print(string.format("        Error: %s", received.data.error_type))
                end
            end
        else
            break
        end
    end
    
    monitoring_results[pattern_name] = events_received
    
    if events_received > 0 then
        print(string.format("   📊 %s: %d events monitored", pattern_name, events_received))
    else
        print(string.format("   ⏰ %s: no events received", pattern_name))
    end
end

print()
print("5. Pipeline performance analytics:")

print("   📈 Pipeline Performance Analytics:")

-- Calculate performance metrics
local total_events = 0
for _, count in pairs(monitoring_results) do
    total_events = total_events + count
end

local processing_time = #data_batches * 15 -- Estimated total processing time
local throughput = total_events > 0 and total_events / processing_time or 0

print(string.format("   • Total batches processed: %d", #data_batches))
print(string.format("   • Total monitoring events: %d", total_events))
print(string.format("   • Estimated processing time: %ds", processing_time))
print(string.format("   • Event throughput: %.2f events/second", throughput))

-- Analyze monitoring patterns
print("   📊 Monitoring Pattern Analysis:")
for pattern_name, event_count in pairs(monitoring_results) do
    if event_count > 0 then
        local percentage = (event_count / total_events) * 100
        print(string.format("   • %s: %d events (%.1f%%)", pattern_name, event_count, percentage))
    end
end

-- Simulate pipeline health metrics
local pipeline_health = {
    success_rate = 85.0 + math.random() * 10, -- 85-95%
    avg_latency = 8.5 + math.random() * 3, -- 8.5-11.5 seconds
    error_rate = math.random() * 5, -- 0-5%
    throughput_mbps = 2.5 + math.random() * 2 -- 2.5-4.5 MB/s
}

print("   🏥 Pipeline Health Metrics:")
print(string.format("   • Success rate: %.1f%%", pipeline_health.success_rate))
print(string.format("   • Average latency: %.1fs", pipeline_health.avg_latency))
print(string.format("   • Error rate: %.2f%%", pipeline_health.error_rate))
print(string.format("   • Throughput: %.1f MB/s", pipeline_health.throughput_mbps))

-- Health status
local overall_health = (pipeline_health.success_rate * 0.4 + 
                       (100 - pipeline_health.error_rate) * 0.3 +
                       math.min(pipeline_health.throughput_mbps / 4 * 100, 100) * 0.3)

print(string.format("   • Overall health score: %.1f/100", overall_health))

if overall_health >= 85 then
    print("   ✅ Pipeline is healthy")
elseif overall_health >= 70 then
    print("   ⚠️  Pipeline has minor issues")
else
    print("   ❌ Pipeline needs attention")
end

print()
print("6. Error handling and recovery:")

print("   🚑 Implementing error recovery mechanisms:")

-- Simulate error recovery for failed batches
local recovery_scenarios = {
    {
        job_id = "recovery_job_001",
        error_type = "timeout",
        recovery_strategy = "retry_with_backoff",
        max_attempts = 3
    },
    {
        job_id = "recovery_job_002", 
        error_type = "data_corruption",
        recovery_strategy = "skip_corrupted_records",
        max_attempts = 1
    },
    {
        job_id = "recovery_job_003",
        error_type = "resource_exhaustion",
        recovery_strategy = "queue_for_later",
        max_attempts = 2
    }
}

for i, scenario in ipairs(recovery_scenarios) do
    log_pipeline_event("WARN", scenario.job_id, string.format("Initiating recovery: %s", scenario.error_type))
    
    Event.publish("pipeline.error.recovery_started", {
        job_id = scenario.job_id,
        error_type = scenario.error_type,
        recovery_strategy = scenario.recovery_strategy,
        max_attempts = scenario.max_attempts,
        start_time = os.time()
    })
    
    print(string.format("   %d. 🔄 Recovery for %s: %s", i, scenario.job_id, scenario.recovery_strategy))
    
    -- Simulate recovery attempts
    for attempt = 1, scenario.max_attempts do
        local recovery_duration = math.random(2, 5)
        os.execute("sleep " .. recovery_duration)
        
        local recovery_success = math.random() > (0.3 * attempt) -- Increasing success probability
        
        Event.publish("pipeline.error.recovery_attempt", {
            job_id = scenario.job_id,
            attempt = attempt,
            max_attempts = scenario.max_attempts,
            duration = recovery_duration,
            success = recovery_success
        })
        
        if recovery_success then
            log_pipeline_event("SUCCESS", scenario.job_id, string.format("Recovery successful on attempt %d", attempt))
            
            Event.publish("pipeline.error.recovery_completed", {
                job_id = scenario.job_id,
                successful_attempt = attempt,
                total_recovery_time = recovery_duration * attempt,
                success = true
            })
            
            print(string.format("   %d. ✅ Recovery successful on attempt %d", i, attempt))
            break
        else
            print(string.format("   %d. ❌ Recovery attempt %d failed", i, attempt))
            
            if attempt == scenario.max_attempts then
                log_pipeline_event("ERROR", scenario.job_id, "Recovery exhausted - manual intervention required")
                
                Event.publish("pipeline.error.recovery_exhausted", {
                    job_id = scenario.job_id,
                    total_attempts = scenario.max_attempts,
                    requires_manual_intervention = true
                })
                
                print(string.format("   %d. ❌ Recovery exhausted for %s", i, scenario.job_id))
            end
        end
    end
end

print()
print("7. Pipeline optimization and scaling:")

print("   ⚡ Pipeline optimization recommendations:")

-- Analyze pipeline performance and provide optimization recommendations
local optimization_recommendations = {}

if pipeline_health.success_rate < 90 then
    table.insert(optimization_recommendations, {
        area = "Reliability",
        recommendation = "Implement better error handling and retry mechanisms",
        priority = "high",
        estimated_improvement = "5-10% success rate increase"
    })
end

if pipeline_health.avg_latency > 10 then
    table.insert(optimization_recommendations, {
        area = "Performance", 
        recommendation = "Optimize processing stages and implement parallel processing",
        priority = "medium",
        estimated_improvement = "20-30% latency reduction"
    })
end

if pipeline_health.throughput_mbps < 3 then
    table.insert(optimization_recommendations, {
        area = "Throughput",
        recommendation = "Increase batch sizes and implement stream processing",
        priority = "medium",
        estimated_improvement = "50-100% throughput increase"
    })
end

if pipeline_health.error_rate > 2 then
    table.insert(optimization_recommendations, {
        area = "Error Rate",
        recommendation = "Improve data validation and preprocessing",
        priority = "high",
        estimated_improvement = "50% error rate reduction"
    })
end

if #optimization_recommendations > 0 then
    for i, rec in ipairs(optimization_recommendations) do
        print(string.format("   %d. [%s] %s: %s", i, rec.priority:upper(), rec.area, rec.recommendation))
        print(string.format("      Expected improvement: %s", rec.estimated_improvement))
        
        Event.publish("pipeline.monitoring.optimization", {
            recommendation_id = "opt_" .. i,
            area = rec.area,
            recommendation = rec.recommendation,
            priority = rec.priority,
            estimated_improvement = rec.estimated_improvement,
            timestamp = os.time()
        })
    end
else
    print("   ✅ Pipeline is well-optimized - no immediate recommendations")
end

print()
print("8. Integration best practices demonstrated:")

print("   💡 Integration Best Practices Demonstrated:")
print("   • Event-driven architecture for loose coupling")
print("   • Hook-based monitoring for comprehensive observability")
print("   • Real-time event processing for immediate feedback")
print("   • Error recovery mechanisms with multiple strategies")
print("   • Performance monitoring and health metrics")
print("   • Scalable batch processing with configurable parameters")
print("   • Integration between hooks, events, and workflow orchestration")
print("   • Resource monitoring and alerting")
print("   • Comprehensive logging and audit trail")
print("   • Optimization recommendations based on performance data")

print()
print("9. Cleanup and final statistics:")

-- Final pipeline statistics
local final_stats = {
    batches_processed = #data_batches,
    monitoring_events = total_events,
    recovery_scenarios = #recovery_scenarios,
    optimization_recommendations = #optimization_recommendations,
    hooks_registered = 0,
    subscriptions_created = 0
}

-- Count hooks and subscriptions
for _ in pairs(hook_handles) do
    final_stats.hooks_registered = final_stats.hooks_registered + 1
end

for _ in pairs(subscriptions) do
    final_stats.subscriptions_created = final_stats.subscriptions_created + 1
end

print("   📊 Final Pipeline Statistics:")
print(string.format("   • Data batches processed: %d", final_stats.batches_processed))
print(string.format("   • Monitoring events captured: %d", final_stats.monitoring_events))
print(string.format("   • Recovery scenarios handled: %d", final_stats.recovery_scenarios))
print(string.format("   • Optimization recommendations: %d", final_stats.optimization_recommendations))
print(string.format("   • Hooks registered: %d", final_stats.hooks_registered))
print(string.format("   • Event subscriptions: %d", final_stats.subscriptions_created))
print(string.format("   • Overall health score: %.1f/100", overall_health))

-- Cleanup hooks
print("   🧹 Cleaning up pipeline infrastructure:")
local hooks_cleaned = 0
for hook_name, handle in pairs(hook_handles) do
    if handle and handle:id() then
        Hook.unregister(handle)
        hooks_cleaned = hooks_cleaned + 1
        print(string.format("   • Unregistered hook: %s", hook_name))
    end
end

-- Cleanup subscriptions
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
print("✨ Real-time Data Pipeline Integration Example Complete!")
print("   Real-world integration demonstrated:")
print("   • End-to-end data pipeline with batch processing")
print("   • Hook-based monitoring and observability")
print("   • Event-driven coordination and state management")
print("   • Real-time performance analytics and health monitoring")
print("   • Automated error recovery and retry mechanisms")
print("   • Resource monitoring and alerting")
print("   • Pipeline optimization recommendations")
print("   • Comprehensive logging and audit capabilities")
print("   • Integration of agents, tools, workflows, hooks, and events")
print("   • Production-ready patterns for scalable data processing")